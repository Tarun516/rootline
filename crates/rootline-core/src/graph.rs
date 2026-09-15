//! Provenance-aware fact-graph types.
//!
//! These types describe the published graph: addressable nodes, typed
//! relations with confidence and evidence, and the validation that keeps
//! dangling references out of published output. Like the IR, this module is
//! dependency-free; storage, transport, and inference layers build on it
//! without leaking their types back in.
//!
//! Endpoint direction per relation kind:
//!
//! - `Contains`: artifact to symbol, or symbol to symbol (class to method);
//! - `Imports`: artifact to artifact, or artifact to an external module;
//! - `Calls`: symbol (or artifact for module-level code) to a symbol, with
//!   `candidates` populated when the target is ambiguous;
//! - `Inherits`: class symbol to a class symbol or external base.

use std::collections::BTreeSet;
use std::fmt;

use crate::RepoPath;
use crate::ir::{AnalyzerInfo, SourceRange, SymbolId};

/// Confidence of a material assertion. These states must not be collapsed
/// into one unexplained score: `ambiguous` (multiple candidates) and
/// `unknown` (insufficient evidence) demand different user responses.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Confidence {
    Deterministic,
    Probable,
    Possible,
    Ambiguous,
    Unknown,
    Unsupported,
    Failed,
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Deterministic => f.write_str("deterministic"),
            Self::Probable => f.write_str("probable"),
            Self::Possible => f.write_str("possible"),
            Self::Ambiguous => f.write_str("ambiguous"),
            Self::Unknown => f.write_str("unknown"),
            Self::Unsupported => f.write_str("unsupported"),
            Self::Failed => f.write_str("failed"),
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
            Self::Symbol(id) => match id.owner() {
                Some(owner) => write!(f, "{}:{}.{}", id.kind(), owner, id.name()),
                None => write!(f, "{}:{}", id.kind(), id.name()),
            },
            Self::External { module } => write!(f, "external:{module}"),
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
        }
    }
}

impl std::error::Error for GraphValidationError {}

/// One attributable connection. Every relation carries at least one source
/// range proving where the connection was observed, plus the analyzer that
/// produced it. Ambiguous relations name their candidate targets explicitly
/// instead of promoting one guess to a deterministic edge.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Relation {
    from: NodeId,
    to: NodeId,
    kind: RelationKind,
    confidence: Confidence,
    candidates: Vec<NodeId>,
    evidence: Vec<SourceRange>,
    analyzer: AnalyzerInfo,
}

impl Relation {
    /// Builds a relation, rejecting evidence-free assertions: a connection
    /// without an observed source location must not enter the graph.
    ///
    /// # Errors
    /// Returns [`GraphValidationError::EmptyEvidence`] when `evidence` is empty.
    pub fn new(
        from: NodeId,
        to: NodeId,
        kind: RelationKind,
        confidence: Confidence,
        candidates: Vec<NodeId>,
        evidence: Vec<SourceRange>,
        analyzer: AnalyzerInfo,
    ) -> Result<Self, GraphValidationError> {
        if evidence.is_empty() {
            return Err(GraphValidationError::EmptyEvidence { kind });
        }
        Ok(Self {
            from,
            to,
            kind,
            confidence,
            candidates,
            evidence,
            analyzer,
        })
    }

    pub fn from(&self) -> &NodeId {
        &self.from
    }

    pub fn to(&self) -> &NodeId {
        &self.to
    }

    pub fn kind(&self) -> RelationKind {
        self.kind
    }

    pub fn confidence(&self) -> Confidence {
        self.confidence
    }

    pub fn candidates(&self) -> &[NodeId] {
        &self.candidates
    }

    pub fn evidence(&self) -> &[SourceRange] {
        &self.evidence
    }

    pub fn analyzer(&self) -> AnalyzerInfo {
        self.analyzer
    }
}

/// A published fact-graph snapshot: its node inventory plus relations in
/// deterministic order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Graph {
    nodes: BTreeSet<NodeId>,
    relations: Vec<Relation>,
}

impl Graph {
    /// Builds a snapshot, canonicalizing relation order so repeated builds
    /// and snapshot tests observe deterministic output.
    pub fn new(nodes: BTreeSet<NodeId>, mut relations: Vec<Relation>) -> Self {
        relations.sort_by(|a, b| {
            (a.from(), a.kind() as u8, a.to(), a.confidence() as u8).cmp(&(
                b.from(),
                b.kind() as u8,
                b.to(),
                b.confidence() as u8,
            ))
        });
        Self { nodes, relations }
    }

    pub fn nodes(&self) -> &BTreeSet<NodeId> {
        &self.nodes
    }

    pub fn relations(&self) -> &[Relation] {
        &self.relations
    }

    /// Verifies publication invariants: every endpoint (including ambiguous
    /// candidates) resolves to the node inventory.
    ///
    /// # Errors
    /// Returns the first dangling endpoint found.
    pub fn validate(&self) -> Result<(), GraphValidationError> {
        for relation in &self.relations {
            for endpoint in std::iter::once(relation.to())
                .chain(std::iter::once(relation.from()))
                .chain(relation.candidates().iter())
            {
                if !self.nodes.contains(endpoint) {
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
        Symbol::new(
            SymbolId::new(
                test_path(path),
                kind,
                owner.map(|owner| SymbolName::new(owner).expect("test owner is named")),
                SymbolName::new(name).expect("test symbol is named"),
            ),
            test_range(),
        )
        .expect("test symbol satisfies IR invariants")
    }

    fn test_analyzer() -> AnalyzerInfo {
        AnalyzerInfo::new("test", "0.0.0")
    }

    #[test]
    fn relations_require_source_evidence() {
        let from = NodeId::Artifact(test_path("a.py"));
        let to = NodeId::Artifact(test_path("b.py"));
        assert_eq!(
            Relation::new(
                from,
                to,
                RelationKind::Imports,
                Confidence::Deterministic,
                Vec::new(),
                Vec::new(),
                test_analyzer(),
            ),
            Err(GraphValidationError::EmptyEvidence {
                kind: RelationKind::Imports
            })
        );
    }

    #[test]
    fn published_graph_rejects_dangling_endpoints() {
        let present = NodeId::Artifact(test_path("a.py"));
        let missing = NodeId::Artifact(test_path("ghost.py"));
        let relation = Relation::new(
            present.clone(),
            missing,
            RelationKind::Imports,
            Confidence::Deterministic,
            Vec::new(),
            vec![test_range()],
            test_analyzer(),
        )
        .expect("test relation is evidenced");
        let graph = Graph::new(BTreeSet::from([present]), vec![relation]);
        assert!(matches!(
            graph.validate(),
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
            candidate.clone(),
            RelationKind::Imports,
            Confidence::Ambiguous,
            vec![candidate.clone(), ghost],
            vec![test_range()],
            test_analyzer(),
        )
        .expect("test relation is evidenced");
        let graph = Graph::new(BTreeSet::from([from, candidate]), vec![relation]);
        assert!(matches!(
            graph.validate(),
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
        // Pins the domain contract from docs/05: each state renders
        // distinctly, so none can be silently merged into another.
        let states = [
            (Confidence::Deterministic, "deterministic"),
            (Confidence::Probable, "probable"),
            (Confidence::Possible, "possible"),
            (Confidence::Ambiguous, "ambiguous"),
            (Confidence::Unknown, "unknown"),
            (Confidence::Unsupported, "unsupported"),
            (Confidence::Failed, "failed"),
        ];
        assert_eq!(states.len(), 7);
        for (state, label) in states {
            assert_eq!(state.to_string(), label);
        }
    }
}
