//! Provenance-aware fact-graph types.
//!
//! These types describe the published graph: addressable nodes, typed
//! relations with assertion strength and evidence, and the validation that
//! keeps dangling references out of published output. Like the IR, this module is
//! dependency-free; storage, transport, and inference layers build on it
//! without leaking their types back in.
//!
//! Three concepts stay separate by construction:
//!
//! - assertion strength ([`Confidence`]): how strongly the assertion itself
//!   is believed — deterministic observation versus ranked inference;
//! - resolution outcome ([`RelationTarget`]): whether the target is proven
//!   ([`RelationTarget::Resolved`]) or an explicit candidate set
//!   ([`RelationTarget::Ambiguous`]) with no fake primary;
//! - operational result: `Ok` versus `Err`, never a relation state. Unknown
//!   and failed analyses surface as diagnostics or typed errors, not edges.
//!
//! Endpoint direction per relation kind:
//!
//! - `Contains`: artifact to symbol, or symbol to symbol (class to method);
//! - `Imports`: artifact to artifact, or artifact to an external module;
//! - `Calls`: symbol (or artifact for module-level code) to a symbol target;
//! - `Inherits`: class symbol to a class symbol or external base.

use std::collections::BTreeMap;
use std::fmt;

use crate::RepoPath;
use crate::RepoPathError;
use crate::ir::{AnalyzerInfo, SourceRange, SymbolId};

/// Strength of a material assertion: deterministic observation versus
/// ranked inference. Resolution state lives in [`RelationTarget`], and
/// operational failure lives in `Err` — neither is a confidence level, so
/// neither appears here.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Confidence {
    Deterministic,
    Probable,
    Possible,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deterministic => f.write_str("deterministic"),
            Self::Probable => f.write_str("probable"),
            Self::Possible => f.write_str("possible"),
        }
    }
}

/// Addressable graph endpoint: a repository artifact, a language-level
/// symbol, or a module outside the analyzed repositories (stdlib or
/// third-party dependency, recorded by dotted path).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum NodeId {
    Artifact(RepoPath),
    Symbol(SymbolId),
    External { module: String },
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Artifact(path) => write!(f, "file:{}", path.as_path().display()),
            Self::Symbol(id) => write!(f, "{}:{}", id.kind(), id.full_path()),
            Self::External { module } => write!(f, "external:{module}"),
        }
    }
}

impl NodeId {
    /// Canonical string for persisted, protocol, and test-comparison use.
    /// Artifact paths render with `/` separators on every platform; symbol
    /// and external forms are already platform-independent.
    ///
    /// # Errors
    /// Propagates [`RepoPathError`] when an artifact path is not valid UTF-8.
    pub fn to_canonical_string(&self) -> Result<String, RepoPathError> {
        match self {
            Self::Artifact(path) => Ok(format!("file:{}", path.to_canonical_string()?)),
            Self::Symbol(_) => Ok(self.to_string()),
            Self::External { module } => Ok(format!("external:{module}")),
        }
    }
}

/// Kind of connection between two nodes.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RelationKind {
    Contains,
    Imports,
    Calls,
    Inherits,
}

impl fmt::Display for RelationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Contains => f.write_str("contains"),
            Self::Imports => f.write_str("imports"),
            Self::Calls => f.write_str("calls"),
            Self::Inherits => f.write_str("inherits"),
        }
    }
}

/// Why a candidate relation violates graph invariants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GraphValidationError {
    EmptyEvidence { kind: RelationKind },
    DanglingEndpoint { endpoint: String },
    AmbiguousTarget { candidates: usize },
}

impl fmt::Display for GraphValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyEvidence { kind } => {
                write!(f, "{kind} relation carries no source evidence")
            }
            Self::DanglingEndpoint { endpoint } => {
                write!(f, "relation references missing endpoint {endpoint}")
            }
            Self::AmbiguousTarget { candidates } => {
                write!(
                    f,
                    "ambiguous target needs at least 2 candidates, got {candidates}"
                )
            }
        }
    }
}

impl std::error::Error for GraphValidationError {}

/// What a relation points at: exactly one proven target, or an explicit
/// candidate set with no primary. There is deliberately no `to()` accessor
/// returning a single node: consumers must match on the target, which makes
/// accidentally treating ambiguity as resolution unrepresentable.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RelationTarget {
    Resolved(NodeId),
    Ambiguous(Vec<NodeId>),
}

impl RelationTarget {
    /// All endpoints this target touches: one for resolved, every candidate
    /// for ambiguous. Used by publication validation.
    pub fn endpoints(&self) -> Vec<&NodeId> {
        match self {
            Self::Resolved(node) => vec![node],
            Self::Ambiguous(candidates) => candidates.iter().collect(),
        }
    }
}

/// One attributable connection. Every relation carries at least one source
/// range proving where the connection was observed, plus the analyzer that
/// produced it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    from: NodeId,
    target: RelationTarget,
    kind: RelationKind,
    confidence: Confidence,
    evidence: Vec<SourceRange>,
    analyzer: AnalyzerInfo,
}

impl Relation {
    /// Builds a relation, rejecting evidence-free assertions and degenerate
    /// ambiguity: a connection without an observed source location, or an
    /// "ambiguous" target with fewer than two candidates, must not enter
    /// the graph.
    ///
    /// # Errors
    /// Returns [`GraphValidationError::EmptyEvidence`] when `evidence` is
    /// empty, or [`GraphValidationError::AmbiguousTarget`] for an ambiguous
    /// target with fewer than two distinct candidates.
    pub fn new(
        from: NodeId,
        target: RelationTarget,
        kind: RelationKind,
        confidence: Confidence,
        evidence: Vec<SourceRange>,
        analyzer: AnalyzerInfo,
    ) -> Result<Self, GraphValidationError> {
        if evidence.is_empty() {
            return Err(GraphValidationError::EmptyEvidence { kind });
        }
        let target = match target {
            RelationTarget::Ambiguous(candidates) => {
                let mut distinct: Vec<NodeId> = candidates;
                distinct.sort();
                distinct.dedup();
                if distinct.len() < 2 {
                    return Err(GraphValidationError::AmbiguousTarget {
                        candidates: distinct.len(),
                    });
                }
                RelationTarget::Ambiguous(distinct)
            }
            resolved => resolved,
        };
        Ok(Self {
            from,
            target,
            kind,
            confidence,
            evidence,
            analyzer,
        })
    }

    pub fn from(&self) -> &NodeId {
        &self.from
    }

    pub fn target(&self) -> &RelationTarget {
        &self.target
    }

    pub fn kind(&self) -> RelationKind {
        self.kind
    }

    pub fn confidence(&self) -> Confidence {
        self.confidence
    }

    pub fn evidence(&self) -> &[SourceRange] {
        &self.evidence
    }

    pub fn analyzer(&self) -> AnalyzerInfo {
        self.analyzer
    }
}

/// Provenance of one node: what declared it, and which analyzer observed
/// it. Symbols carry their declaration range and the parsing adapter;
/// artifacts and externals carry no range, and artifacts will migrate to
/// scanner provenance once the repository layer owns node assembly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeInfo {
    range: Option<SourceRange>,
    analyzer: AnalyzerInfo,
}

impl NodeInfo {
    pub fn new(range: Option<SourceRange>, analyzer: AnalyzerInfo) -> Self {
        Self { range, analyzer }
    }

    pub fn range(self) -> Option<SourceRange> {
        self.range
    }

    pub fn analyzer(self) -> AnalyzerInfo {
        self.analyzer
    }
}

/// Graph-level provenance: which repository snapshot this graph describes.
/// Both fields are opaque strings on purpose — real repository and revision
/// identity types arrive with the persistence design, and inventing them
/// here would fossilize guesses into stored data.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphMetadata {
    repository: Option<String>,
    revision: Option<String>,
}

impl GraphMetadata {
    pub fn new(repository: Option<String>, revision: Option<String>) -> Self {
        Self {
            repository,
            revision,
        }
    }

    pub fn repository(&self) -> Option<&str> {
        self.repository.as_deref()
    }

    pub fn revision(&self) -> Option<&str> {
        self.revision.as_deref()
    }
}

/// A published fact-graph snapshot: its provenance, node inventory with per-
/// node records, and relations in deterministic order. The only constructor
/// validates, so holding a `Graph` means holding a graph with no dangling
/// endpoints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graph {
    metadata: GraphMetadata,
    nodes: BTreeMap<NodeId, NodeInfo>,
    relations: Vec<Relation>,
}

impl Graph {
    /// Builds a snapshot, canonicalizing relation order so repeated builds
    /// and snapshot tests observe deterministic output, and validating every
    /// endpoint against the node inventory.
    ///
    /// # Errors
    /// Returns [`GraphValidationError::DanglingEndpoint`] for the first
    /// endpoint missing from `nodes`.
    pub fn try_new(
        metadata: GraphMetadata,
        nodes: BTreeMap<NodeId, NodeInfo>,
        mut relations: Vec<Relation>,
    ) -> Result<Self, GraphValidationError> {
        relations.sort_by(|a, b| {
            (a.from(), a.kind() as u8, a.target(), a.confidence() as u8).cmp(&(
                b.from(),
                b.kind() as u8,
                b.target(),
                b.confidence() as u8,
            ))
        });
        let graph = Self {
            metadata,
            nodes,
            relations,
        };
        graph.validate()?;
        Ok(graph)
    }

    pub fn metadata(&self) -> &GraphMetadata {
        &self.metadata
    }

    pub fn nodes(&self) -> &BTreeMap<NodeId, NodeInfo> {
        &self.nodes
    }

    pub fn node_info(&self, id: &NodeId) -> Option<&NodeInfo> {
        self.nodes.get(id)
    }

    pub fn relations(&self) -> &[Relation] {
        &self.relations
    }

    /// Verifies publication invariants: `from` plus every target endpoint
    /// resolves to the node inventory.
    ///
    /// # Errors
    /// Returns the first dangling endpoint found.
    pub fn validate(&self) -> Result<(), GraphValidationError> {
        for relation in &self.relations {
            for endpoint in std::iter::once(relation.from()).chain(relation.target().endpoints()) {
                if !self.nodes.contains_key(endpoint) {
                    return Err(GraphValidationError::DanglingEndpoint {
                        endpoint: endpoint.to_string(),
                    });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test fixtures are static inputs; setup failure must fail the test immediately"
)]
mod tests {
    use super::*;
    use crate::ir::{ByteOffset, ByteRange, LineNumber, LineRange, Symbol, SymbolKind, SymbolName};
    use std::path::Path;

    fn test_path(name: &str) -> RepoPath {
        RepoPath::new(Path::new(name)).expect("test path is relative")
    }

    fn test_range() -> SourceRange {
        SourceRange::new(
            ByteRange::new(
                ByteOffset::new(0).expect("offset fits"),
                ByteOffset::new(10).expect("offset fits"),
            )
            .expect("test range is non-empty"),
            LineRange::new(
                LineNumber::from_zero_based(0).expect("line fits"),
                LineNumber::from_zero_based(1).expect("line fits"),
            )
            .expect("test lines are ordered"),
        )
    }

    fn test_symbol(path: &str, kind: SymbolKind, owner: Option<&str>, name: &str) -> Symbol {
        let owner_id = owner.map(|owner| {
            SymbolId::new(
                test_path(path),
                SymbolKind::Class,
                None,
                SymbolName::new(owner).expect("test owner is named"),
                0,
            )
        });
        Symbol::new(
            SymbolId::new(
                test_path(path),
                kind,
                owner_id,
                SymbolName::new(name).expect("test symbol is named"),
                0,
            ),
            test_range(),
        )
        .expect("test symbol satisfies IR invariants")
    }

    fn test_analyzer() -> AnalyzerInfo {
        AnalyzerInfo::new("test", "0.0.0")
    }

    fn test_metadata() -> GraphMetadata {
        GraphMetadata::new(None, None)
    }

    fn test_nodes(ids: Vec<NodeId>) -> BTreeMap<NodeId, NodeInfo> {
        ids.into_iter()
            .map(|id| (id, NodeInfo::new(None, test_analyzer())))
            .collect()
    }

    #[test]
    fn nodes_carry_ranges_and_analyzers() {
        let symbol = test_symbol("m.py", SymbolKind::Method, Some("Model"), "fit");
        let id = NodeId::Symbol(symbol.id().clone());
        let info = NodeInfo::new(Some(test_range()), test_analyzer());
        assert_eq!(info.range(), Some(test_range()));
        assert_eq!(info.analyzer(), test_analyzer());
        let graph = Graph::try_new(
            test_metadata(),
            test_nodes(vec![NodeId::Artifact(test_path("m.py")), id.clone()]),
            Vec::new(),
        )
        .expect("test graph validates");
        assert_eq!(
            graph.node_info(&id).expect("node present").analyzer(),
            test_analyzer()
        );
        assert_eq!(graph.metadata().revision(), None);
    }

    #[test]
    fn relations_require_source_evidence() {
        let from = NodeId::Artifact(test_path("a.py"));
        let to = NodeId::Artifact(test_path("b.py"));
        assert_eq!(
            Relation::new(
                from,
                RelationTarget::Resolved(to),
                RelationKind::Imports,
                Confidence::Deterministic,
                Vec::new(),
                test_analyzer(),
            ),
            Err(GraphValidationError::EmptyEvidence {
                kind: RelationKind::Imports
            })
        );
    }

    #[test]
    fn ambiguous_targets_need_two_distinct_candidates() {
        let from = NodeId::Artifact(test_path("a.py"));
        let only = NodeId::Artifact(test_path("b.py"));
        assert_eq!(
            Relation::new(
                from.clone(),
                RelationTarget::Ambiguous(vec![only.clone(), only.clone()]),
                RelationKind::Imports,
                Confidence::Deterministic,
                vec![test_range()],
                test_analyzer(),
            ),
            Err(GraphValidationError::AmbiguousTarget { candidates: 1 })
        );
        assert!(matches!(
            Relation::new(
                from,
                RelationTarget::Ambiguous(vec![only]),
                RelationKind::Imports,
                Confidence::Deterministic,
                vec![test_range()],
                test_analyzer(),
            ),
            Err(GraphValidationError::AmbiguousTarget { .. })
        ));
    }

    #[test]
    fn published_graph_rejects_dangling_endpoints() {
        let present = NodeId::Artifact(test_path("a.py"));
        let missing = NodeId::Artifact(test_path("ghost.py"));
        let relation = Relation::new(
            present.clone(),
            RelationTarget::Resolved(missing),
            RelationKind::Imports,
            Confidence::Deterministic,
            vec![test_range()],
            test_analyzer(),
        )
        .expect("test relation is evidenced");
        assert!(matches!(
            Graph::try_new(test_metadata(), test_nodes(vec![present]), vec![relation]),
            Err(GraphValidationError::DanglingEndpoint { .. })
        ));
    }

    #[test]
    fn ambiguous_candidates_must_also_resolve() {
        let from = NodeId::Artifact(test_path("a.py"));
        let candidate = NodeId::Artifact(test_path("b.py"));
        let ghost = NodeId::Artifact(test_path("ghost.py"));
        let relation = Relation::new(
            from.clone(),
            RelationTarget::Ambiguous(vec![candidate.clone(), ghost]),
            RelationKind::Imports,
            Confidence::Deterministic,
            vec![test_range()],
            test_analyzer(),
        )
        .expect("test relation is evidenced");
        assert!(matches!(
            Graph::try_new(
                test_metadata(),
                test_nodes(vec![from, candidate]),
                vec![relation]
            ),
            Err(GraphValidationError::DanglingEndpoint { .. })
        ));
    }

    #[test]
    fn node_display_distinguishes_endpoint_families() {
        let symbol = test_symbol("m.py", SymbolKind::Method, Some("Model"), "fit");
        assert_eq!(
            NodeId::Symbol(symbol.id().clone()).to_string(),
            "method:Model.fit"
        );
        assert_eq!(NodeId::Artifact(test_path("m.py")).to_string(), "file:m.py");
        assert_eq!(
            NodeId::External {
                module: "torch.nn".to_owned()
            }
            .to_string(),
            "external:torch.nn"
        );
    }

    #[test]
    fn confidence_states_are_all_addressable() {
        // Assertion strength only: resolution outcome and operational
        // failure live elsewhere by construction, so they cannot be merged
        // back into this enum.
        let states = [
            (Confidence::Deterministic, "deterministic"),
            (Confidence::Probable, "probable"),
            (Confidence::Possible, "possible"),
        ];
        assert_eq!(states.len(), 3);
        for (state, label) in states {
            assert_eq!(state.to_string(), label);
        }
    }
}
