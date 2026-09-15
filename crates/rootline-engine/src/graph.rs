//! Provenance-aware symbol-graph construction.
//!
//! The builder turns per-file adapter output plus [`ModuleIndex`] resolution
//! into a validated [`Graph`]: containment, resolved imports, inheritance,
//! and conservatively resolved calls. Everything it cannot prove stays an
//! explicit [`GraphDiagnostic`] — a probable guess is never promoted to a
//! deterministic edge.
//!
//! Call-target policy (only these resolve; all else is diagnosed):
//!
//! - bare `f()`: nearest enclosing scope's function/class definition, then
//!   names imported with `from`, then language builtins (skipped silently by
//!   design: they are outside the analyzed universe, not unknown);
//! - `self.m()` / `cls.m()`: methods of the enclosing class only;
//! - `mod.f()`: symbols of the file an import binding proves `mod` names;
//! - `factory().make()`, `a.b.c()`: opaque — diagnosed, never matched
//!   against a bare local definition the qualifier disproves.
//!
//! Base-class policy mirrors it: same-file classes, classes reached through
//! import bindings, or external modules. Bare names that match nothing
//! (including builtins such as `Exception`) are diagnosed; telling builtins,
//! forgotten imports, and undefined names apart is deferred stdlib work.
//!
//! External identity is always a module path (`os`, `pathlib`), never
//! module-plus-attribute: the graph records which outside dependency is
//! used, and source ranges preserve the exact attribute access.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use rootline_core::RepoPath;
use rootline_core::graph::{
    Confidence, Graph, GraphMetadata, GraphValidationError, NodeId, NodeInfo, Relation,
    RelationKind, RelationTarget,
};
use rootline_core::ir::{AnalyzerInfo, ParsedModule, SourceRange, Symbol, SymbolId, SymbolKind};

use crate::python::BUILTIN_NAMES;
use crate::python::resolve::{ModuleIndex, Resolution};

/// Identity of the graph-construction stage for provenance records.
/// Parsing facts carry the adapter's identity; graph facts carry this one.
pub const GRAPH_BUILDER_NAME: &str = "rootline-graph-builder";
/// Builder version follows the engine crate.
pub const GRAPH_BUILDER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// What a diagnostic reports.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticKind {
    Import,
    Call,
    BaseClass,
}

impl fmt::Display for DiagnosticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Import => f.write_str("import"),
            Self::Call => f.write_str("call"),
            Self::BaseClass => f.write_str("base-class"),
        }
    }
}

/// One explicitly unproven fact: what was written, where, and why no edge
/// was published for it. Diagnostics are data, not log strings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphDiagnostic {
    kind: DiagnosticKind,
    file: RepoPath,
    subject: String,
    reason: String,
    range: SourceRange,
    analyzer: AnalyzerInfo,
}

impl GraphDiagnostic {
    pub fn kind(&self) -> DiagnosticKind {
        self.kind
    }

    pub fn file(&self) -> &RepoPath {
        &self.file
    }

    pub fn subject(&self) -> &str {
        &self.subject
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn analyzer(&self) -> AnalyzerInfo {
        self.analyzer
    }
}

/// A built graph plus the diagnostics explaining what was left out of it.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedGraph {
    graph: Graph,
    diagnostics: Vec<GraphDiagnostic>,
}

impl ResolvedGraph {
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// Diagnostics in deterministic file-and-offset order.
    pub fn diagnostics(&self) -> &[GraphDiagnostic] {
        &self.diagnostics
    }
}

/// Human-readable rendering of an import statement for diagnostics.
fn import_subject(import: &rootline_core::ir::ImportStatement) -> String {
    let dots = ".".repeat(import.level() as usize);
    let module = import.module().unwrap_or_default();
    let names = import
        .names()
        .iter()
        .map(|name| {
            if name.imported() == name.bound().as_str() {
                name.imported().to_owned()
            } else {
                format!("{} as {}", name.imported(), name.bound().as_str())
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    if import.is_from() {
        if import.is_wildcard() {
            format!("from {dots}{module} import *")
        } else {
            format!("from {dots}{module} import {names}")
        }
    } else if let Some(first) = import.names().first() {
        let bound = first.bound().as_str();
        let top = module.split('.').next().unwrap_or_default();
        if bound == top {
            format!("import {module}")
        } else {
            format!("import {module} as {bound}")
        }
    } else {
        format!("import {module}")
    }
}

/// Names one bound name with both spellings for diagnostic reasons:
/// `'thing'`, or `'thing' bound as 'renamed'` when aliased.
fn name_render(name: &rootline_core::ir::ImportedName) -> String {
    if name.imported() == name.bound().as_str() {
        format!("'{}'", name.imported())
    } else {
        format!("'{}' bound as '{}'", name.imported(), name.bound().as_str())
    }
}

/// Human-readable rendering of a call site for diagnostics.
fn call_subject(call: &rootline_core::ir::CallSite) -> String {
    match call.receiver() {
        rootline_core::ir::CallReceiver::Absent => format!("{}()", call.name().as_str()),
        rootline_core::ir::CallReceiver::Named(receiver) => {
            format!("{}.{}()", receiver.as_str(), call.name().as_str())
        }
        rootline_core::ir::CallReceiver::Opaque => format!("?.{}()", call.name().as_str()),
    }
}
/// Builds a validated fact graph from parsed modules.
///
/// Modules are processed in file order regardless of input order, so the
/// same inputs always produce the same graph.
///
/// # Errors
/// Returns [`GraphValidationError`] when the constructed graph violates
/// publication invariants. With correct builder logic this never fires; it
/// exists so a future builder defect fails loudly instead of publishing a
/// graph with dangling references.
pub fn build(
    modules: &[ParsedModule],
    index: &ModuleIndex,
    metadata: GraphMetadata,
) -> Result<ResolvedGraph, GraphValidationError> {
    let mut builder = Builder::new(index, metadata);
    let mut ordered: Vec<&ParsedModule> = modules.iter().collect();
    ordered.sort_by(|a, b| a.file().cmp(b.file()));
    for module in &ordered {
        builder.register_symbols(module);
    }
    for module in &ordered {
        builder.add_containment(module)?;
        builder.add_imports(module)?;
        builder.add_inheritance(module)?;
        builder.add_calls(module)?;
    }
    builder.finish()
}

/// Owned per-symbol record the builder reasons over.
#[derive(Clone, Debug)]
struct SymbolRecord {
    id: SymbolId,
    kind: SymbolKind,
    /// Dotted owner path, or empty for module top level.
    parent: String,
    /// Full dotted path including the symbol's own name.
    full: String,
    range: SourceRange,
    analyzer: AnalyzerInfo,
}

impl SymbolRecord {
    fn of(symbol: &Symbol, range: SourceRange, analyzer: AnalyzerInfo) -> Self {
        Self {
            id: symbol.id().clone(),
            kind: symbol.id().kind(),
            parent: symbol.id().scope_path(),
            full: symbol.id().full_path(),
            range,
            analyzer,
        }
    }
}

/// How an import binding resolved, cached per file for call and base-class
/// reasoning.
#[derive(Clone, Debug)]
enum Binding {
    /// `import a.b [as c]`: names the module file or external module.
    Module { resolution: Resolution },
    /// `from p import n` where `n` is defined in the resolved package file.
    Symbol { target: SymbolId },
    /// `from p import n` where `n` is a submodule file.
    Submodule { target: RepoPath },
    /// `from p import n` where `p` is an external module.
    ExternalName { module: String },
    /// Anything unresolved (the import itself already produced a diagnostic).
    Broken,
}

/// Per-file import state shared by the import, call, and inheritance passes.
#[derive(Clone, Debug, Default)]
struct FileImports {
    /// Bound name to binding, covering plain-import aliases and from-names.
    bindings: BTreeMap<String, Binding>,
}

struct Builder<'a> {
    index: &'a ModuleIndex,
    metadata: GraphMetadata,
    analyzer: AnalyzerInfo,
    symbols: Vec<SymbolRecord>,
    by_file: BTreeMap<RepoPath, Vec<usize>>,
    imports: BTreeMap<RepoPath, FileImports>,
    relations: Vec<Relation>,
    diagnostics: Vec<GraphDiagnostic>,
    externals: BTreeSet<String>,
}

impl<'a> Builder<'a> {
    fn new(index: &'a ModuleIndex, metadata: GraphMetadata) -> Self {
        Self {
            index,
            metadata,
            analyzer: AnalyzerInfo::new(GRAPH_BUILDER_NAME, GRAPH_BUILDER_VERSION),
            symbols: Vec::new(),
            by_file: BTreeMap::new(),
            imports: BTreeMap::new(),
            relations: Vec::new(),
            diagnostics: Vec::new(),
            externals: BTreeSet::new(),
        }
    }

    /// Indexes every symbol so cross-file lookups see complete tables.
    fn register_symbols(&mut self, module: &ParsedModule) {
        let file = module.file().clone();
        let entry = self.by_file.entry(file).or_default();
        for symbol in module.symbols() {
            let record = SymbolRecord::of(symbol, symbol.range(), module.analyzer());
            self.symbols.push(record);
            if let Some(position) = self.symbols.len().checked_sub(1) {
                entry.push(position);
            }
        }
    }

    /// File-contains-symbol and class/function-contains-member edges.
    /// Ownership is structural adapter evidence, so every symbol's parent is
    /// known directly: there is no lookup to miss and no gap to skip.
    fn add_containment(&mut self, module: &ParsedModule) -> Result<(), GraphValidationError> {
        let file = module.file();
        for symbol in module.symbols() {
            let record = SymbolRecord::of(symbol, symbol.range(), module.analyzer());
            let from = match record.id.owner() {
                None => NodeId::Artifact(file.clone()),
                Some(parent) => NodeId::Symbol(parent.clone()),
            };
            self.push_relation(
                from,
                NodeId::Symbol(record.id),
                RelationKind::Contains,
                Confidence::Deterministic,
                vec![symbol.range()],
            )?;
        }
        Ok(())
    }

    /// Records a resolved relation, tracking external targets for the node
    /// inventory. Evidence is never empty at any call site; a violation
    /// propagates instead of publishing an unattributed edge. A file
    /// importing itself (`from . import x` resolving its own package)
    /// carries no information and is skipped — but only for imports:
    /// recursive calls are legitimate edges.
    ///
    /// Resolved observations are deterministic by construction; inferred
    /// edges will carry their own confidence when that stage exists.
    fn push_relation(
        &mut self,
        from: NodeId,
        to: NodeId,
        kind: RelationKind,
        confidence: Confidence,
        evidence: Vec<SourceRange>,
    ) -> Result<(), GraphValidationError> {
        if kind == RelationKind::Imports && from == to {
            return Ok(());
        }
        if let NodeId::External { module } = &to {
            self.externals.insert(module.clone());
        }
        self.relations.push(Relation::new(
            from,
            RelationTarget::Resolved(to),
            kind,
            confidence,
            evidence,
            self.analyzer,
        )?);
        Ok(())
    }

    fn push_diagnostic(
        &mut self,
        kind: DiagnosticKind,
        file: &RepoPath,
        subject: String,
        reason: String,
        range: SourceRange,
    ) {
        self.diagnostics.push(GraphDiagnostic {
            kind,
            file: file.clone(),
            subject,
            reason,
            range,
            analyzer: self.analyzer,
        });
    }

    /// Symbols of one file named `name` whose parent path is exactly
    /// `scope`. Callers pass scope chains (nearest scope wins elsewhere);
    /// this answers one scope level.
    fn symbols_in_scope(&self, file: &RepoPath, scope: &str, name: &str) -> Vec<SymbolId> {
        let positions = self.by_file.get(file).cloned().unwrap_or_default();
        positions
            .into_iter()
            .filter_map(|position| {
                let record = &self.symbols[position];
                if record.parent == scope && record.id.name().as_str() == name {
                    Some(record.id.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// Top-level (module-attribute) symbols of one file named `name`. Only
    /// these are reachable as `module.name` from another file.
    fn top_level_symbols(&self, file: &RepoPath, name: &str) -> Vec<SymbolId> {
        self.symbols_in_scope(file, "", name)
    }

    /// Import edges plus the binding table calls and bases reason over.
    fn add_imports(&mut self, module: &ParsedModule) -> Result<(), GraphValidationError> {
        let file = module.file().clone();
        for import in module.imports() {
            let resolution = match import.level() {
                0 => match import.module() {
                    Some(target) => self.index.resolve_absolute(target),
                    None => {
                        self.push_diagnostic(
                            DiagnosticKind::Import,
                            &file,
                            import_subject(import),
                            "plain import names no module".to_owned(),
                            import.range(),
                        );
                        continue;
                    }
                },
                level => self.index.resolve_relative(&file, level, import.module()),
            };
            if import.is_from() {
                self.add_from_import(&file, import, resolution)?;
            } else {
                self.add_plain_import(&file, import, resolution)?;
            }
        }
        Ok(())
    }

    fn add_plain_import(
        &mut self,
        file: &RepoPath,
        import: &rootline_core::ir::ImportStatement,
        resolution: Resolution,
    ) -> Result<(), GraphValidationError> {
        let file_node = NodeId::Artifact(file.clone());
        match resolution {
            Resolution::Resolved { target } => {
                self.push_relation(
                    file_node,
                    NodeId::Artifact(target.clone()),
                    RelationKind::Imports,
                    Confidence::Deterministic,
                    vec![import.range()],
                )?;
                self.bind_module_names(file, import, Resolution::Resolved { target });
            }
            Resolution::Ambiguous { candidates } => {
                self.push_ambiguous(
                    file_node,
                    candidates.iter().cloned().map(NodeId::Artifact).collect(),
                    RelationKind::Imports,
                    import.range(),
                )?;
                self.bind_module_names(file, import, Resolution::Ambiguous { candidates });
            }
            Resolution::External { module } => {
                self.push_relation(
                    file_node,
                    NodeId::External {
                        module: module.clone(),
                    },
                    RelationKind::Imports,
                    Confidence::Deterministic,
                    vec![import.range()],
                )?;
                self.bind_module_names(file, import, Resolution::External { module });
            }
            Resolution::Unknown { reason } => {
                self.push_diagnostic(
                    DiagnosticKind::Import,
                    file,
                    import_subject(import),
                    reason,
                    import.range(),
                );
            }
        }
        Ok(())
    }

    /// Records plain-import bindings: each bound name stands for the whole
    /// imported module, never for a symbol inside it.
    fn bind_module_names(
        &mut self,
        file: &RepoPath,
        import: &rootline_core::ir::ImportStatement,
        resolution: Resolution,
    ) {
        let entry = self.imports.entry(file.clone()).or_default();
        for name in import.names() {
            entry.bindings.insert(
                name.bound().as_str().to_owned(),
                Binding::Module {
                    resolution: resolution.clone(),
                },
            );
        }
    }

    fn add_from_import(
        &mut self,
        file: &RepoPath,
        import: &rootline_core::ir::ImportStatement,
        resolution: Resolution,
    ) -> Result<(), GraphValidationError> {
        let file_node = NodeId::Artifact(file.clone());
        match resolution {
            Resolution::Resolved { target } => {
                self.push_relation(
                    file_node,
                    NodeId::Artifact(target.clone()),
                    RelationKind::Imports,
                    Confidence::Deterministic,
                    vec![import.range()],
                )?;
                for name in import.names() {
                    let bound = name.bound().as_str();
                    let defined = self.top_level_symbols(&target, name.imported());
                    if let [only] = defined.as_slice() {
                        self.bind_symbol(file, bound, only.clone());
                    } else if !defined.is_empty() {
                        self.push_diagnostic(
                            DiagnosticKind::Import,
                            file,
                            import_subject(import),
                            format!(
                                "name {} is defined {} times in '{}'",
                                name_render(name),
                                defined.len(),
                                target.as_path().display()
                            ),
                            import.range(),
                        );
                        self.bind_broken(file, bound);
                    } else {
                        match self.index.resolve_submodule(&target, name.imported()) {
                            Ok(Some(submodule)) => {
                                self.push_relation(
                                    NodeId::Artifact(file.clone()),
                                    NodeId::Artifact(submodule.clone()),
                                    RelationKind::Imports,
                                    Confidence::Deterministic,
                                    vec![import.range()],
                                )?;
                                self.bind_submodule(file, bound, submodule);
                            }
                            Ok(None) => {
                                self.push_diagnostic(
                                    DiagnosticKind::Import,
                                    file,
                                    import_subject(import),
                                    format!(
                                        "name {} is neither defined in '{}' nor a submodule of it",
                                        name_render(name),
                                        target.as_path().display()
                                    ),
                                    import.range(),
                                );
                                self.bind_broken(file, bound);
                            }
                            Err(_) => {
                                self.push_diagnostic(
                                    DiagnosticKind::Import,
                                    file,
                                    import_subject(import),
                                    "package path is not valid UTF-8".to_owned(),
                                    import.range(),
                                );
                                self.bind_broken(file, bound);
                            }
                        }
                    }
                }
            }
            Resolution::Ambiguous { candidates } => {
                self.push_ambiguous(
                    file_node,
                    candidates.iter().cloned().map(NodeId::Artifact).collect(),
                    RelationKind::Imports,
                    import.range(),
                )?;
                for name in import.names() {
                    self.bind_broken(file, name.bound().as_str());
                }
            }
            Resolution::External { module } => {
                self.push_relation(
                    file_node,
                    NodeId::External {
                        module: module.clone(),
                    },
                    RelationKind::Imports,
                    Confidence::Deterministic,
                    vec![import.range()],
                )?;
                let entry = self.imports.entry(file.clone()).or_default();
                for name in import.names() {
                    entry.bindings.insert(
                        name.bound().as_str().to_owned(),
                        Binding::ExternalName {
                            module: module.clone(),
                        },
                    );
                }
            }
            Resolution::Unknown { reason } => {
                self.push_diagnostic(
                    DiagnosticKind::Import,
                    file,
                    import_subject(import),
                    reason,
                    import.range(),
                );
                for name in import.names() {
                    self.bind_broken(file, name.bound().as_str());
                }
            }
        }
        Ok(())
    }

    fn bind_symbol(&mut self, file: &RepoPath, name: &str, target: SymbolId) {
        self.imports
            .entry(file.clone())
            .or_default()
            .bindings
            .insert(name.to_owned(), Binding::Symbol { target });
    }

    fn bind_submodule(&mut self, file: &RepoPath, name: &str, target: RepoPath) {
        self.imports
            .entry(file.clone())
            .or_default()
            .bindings
            .insert(name.to_owned(), Binding::Submodule { target });
    }

    fn bind_broken(&mut self, file: &RepoPath, name: &str) {
        self.imports
            .entry(file.clone())
            .or_default()
            .bindings
            .insert(name.to_owned(), Binding::Broken);
    }

    /// Inheritance edges: same-file classes, classes reached through
    /// import bindings, or external modules. Anything else is diagnosed.
    fn add_inheritance(&mut self, module: &ParsedModule) -> Result<(), GraphValidationError> {
        let file = module.file().clone();
        for inheritance in module.inheritances() {
            let base = inheritance.base();
            match base.split_once('.') {
                None => self.add_bare_base(&file, inheritance)?,
                Some((head, _)) => self.add_dotted_base(&file, inheritance, head)?,
            }
        }
        Ok(())
    }

    /// Bare base classes resolve through the defining scope chain,
    /// considering class symbols only: a function is never a base class.
    fn add_bare_base(
        &mut self,
        file: &RepoPath,
        inheritance: &rootline_core::ir::Inheritance,
    ) -> Result<(), GraphValidationError> {
        let scopes = self.definition_scopes(inheritance.class());
        let mut matches = Vec::new();
        for scope in &scopes {
            let found: Vec<SymbolId> = self
                .symbols_in_scope(file, scope, inheritance.base())
                .into_iter()
                .filter(|id| id.kind() == SymbolKind::Class)
                .collect();
            if !found.is_empty() {
                matches = found;
                break;
            }
        }
        match matches.as_slice() {
            [only] => {
                self.push_relation(
                    NodeId::Symbol(inheritance.class().clone()),
                    NodeId::Symbol(only.clone()),
                    RelationKind::Inherits,
                    Confidence::Deterministic,
                    vec![inheritance.range()],
                )?;
            }
            [] => {
                self.push_diagnostic(
                    DiagnosticKind::BaseClass,
                    file,
                    inheritance.base().to_owned(),
                    format!(
                        "base class '{}' resolves to no analyzed class",
                        inheritance.base()
                    ),
                    inheritance.range(),
                );
            }
            many => {
                self.push_ambiguous(
                    NodeId::Symbol(inheritance.class().clone()),
                    many.iter().cloned().map(NodeId::Symbol).collect(),
                    RelationKind::Inherits,
                    inheritance.range(),
                )?;
            }
        }
        Ok(())
    }

    /// Dotted bases resolve their head through import bindings: `os.PathLike`
    /// with `import os` names the external module `os`, while
    /// `store.Shelf` with `from . import store` searches that submodule file.
    fn add_dotted_base(
        &mut self,
        file: &RepoPath,
        inheritance: &rootline_core::ir::Inheritance,
        head: &str,
    ) -> Result<(), GraphValidationError> {
        let tail = inheritance
            .base()
            .rsplit('.')
            .next()
            .unwrap_or(inheritance.base());
        let class_node = NodeId::Symbol(inheritance.class().clone());
        let binding = self
            .imports
            .get(file)
            .and_then(|imports| imports.bindings.get(head))
            .cloned();
        match binding {
            Some(Binding::Module { resolution }) => match resolution {
                Resolution::Resolved { target } => {
                    self.add_class_in_file(&class_node, file, &target, tail, inheritance.range())?;
                }
                Resolution::Ambiguous { candidates } => {
                    self.push_ambiguous(
                        class_node,
                        candidates.into_iter().map(NodeId::Artifact).collect(),
                        RelationKind::Inherits,
                        inheritance.range(),
                    )?;
                }
                Resolution::External { module } => {
                    self.push_relation(
                        class_node,
                        NodeId::External { module },
                        RelationKind::Inherits,
                        Confidence::Deterministic,
                        vec![inheritance.range()],
                    )?;
                }
                Resolution::Unknown { .. } => {
                    self.push_diagnostic(
                        DiagnosticKind::BaseClass,
                        file,
                        inheritance.base().to_owned(),
                        format!(
                            "base class '{}' names an unresolved import",
                            inheritance.base()
                        ),
                        inheritance.range(),
                    );
                }
            },
            Some(Binding::Symbol { target }) => {
                if target.kind() != SymbolKind::Class {
                    self.push_diagnostic(
                        DiagnosticKind::BaseClass,
                        file,
                        inheritance.base().to_owned(),
                        format!(
                            "'{}' names a {}, not a class",
                            inheritance.base(),
                            target.kind()
                        ),
                        inheritance.range(),
                    );
                    return Ok(());
                }
                // `head.tail` where `head` names an imported class searches
                // classes nested directly inside it. Deeper paths use only
                // their final segment; the diagnostic subject keeps the full
                // text visible.
                let parent = target.scope_path();
                let parent = if parent.is_empty() {
                    target.name().as_str().to_owned()
                } else {
                    format!("{}.{}", parent, target.name().as_str())
                };
                let found: Vec<SymbolId> = self
                    .symbols_in_scope(target.file(), &parent, tail)
                    .into_iter()
                    .filter(|id| id.kind() == SymbolKind::Class)
                    .collect();
                self.push_class_edge(
                    class_node,
                    file,
                    found,
                    inheritance.base(),
                    inheritance.range(),
                )?;
            }
            Some(Binding::Submodule { target }) => {
                self.add_class_in_file(&class_node, file, &target, tail, inheritance.range())?;
            }
            Some(Binding::ExternalName { module }) => {
                self.push_relation(
                    class_node,
                    NodeId::External { module },
                    RelationKind::Inherits,
                    Confidence::Deterministic,
                    vec![inheritance.range()],
                )?;
            }
            Some(Binding::Broken) | None => {
                self.push_diagnostic(
                    DiagnosticKind::BaseClass,
                    file,
                    inheritance.base().to_owned(),
                    format!(
                        "base class '{}' names '{}', which is not imported here",
                        inheritance.base(),
                        head
                    ),
                    inheritance.range(),
                );
            }
        }
        Ok(())
    }

    /// Looks for a top-level class in another file (module-attribute access
    /// only reaches top-level names).
    fn add_class_in_file(
        &mut self,
        class_node: &NodeId,
        file: &RepoPath,
        target_file: &RepoPath,
        name: &str,
        range: SourceRange,
    ) -> Result<(), GraphValidationError> {
        let found: Vec<SymbolId> = self
            .top_level_symbols(target_file, name)
            .into_iter()
            .filter(|id| id.kind() == SymbolKind::Class)
            .collect();
        self.push_class_edge(class_node.clone(), file, found, name, range)
    }

    /// Emits one class edge from a candidate set: exactly one target gives
    /// a deterministic edge, none gives a `BaseClass` diagnostic, and
    /// several give an ambiguous edge with every candidate attached.
    fn push_class_edge(
        &mut self,
        from: NodeId,
        file: &RepoPath,
        found: Vec<SymbolId>,
        subject: &str,
        range: SourceRange,
    ) -> Result<(), GraphValidationError> {
        match found.as_slice() {
            [only] => {
                self.push_relation(
                    from,
                    NodeId::Symbol(only.clone()),
                    RelationKind::Inherits,
                    Confidence::Deterministic,
                    vec![range],
                )?;
            }
            [] => {
                self.push_diagnostic(
                    DiagnosticKind::BaseClass,
                    file,
                    subject.to_owned(),
                    format!("'{subject}' resolves to no analyzed class"),
                    range,
                );
            }
            many => {
                self.push_ambiguous(
                    from,
                    many.iter().cloned().map(NodeId::Symbol).collect(),
                    RelationKind::Inherits,
                    range,
                )?;
            }
        }
        Ok(())
    }

    /// Emits an ambiguous edge: the candidate set with no primary target.
    /// Candidates are files or symbols already in the node inventory, so
    /// validation still holds. Ambiguity this precisely observed is a
    /// deterministic fact about the analysis, hence the confidence.
    fn push_ambiguous(
        &mut self,
        from: NodeId,
        candidates: Vec<NodeId>,
        kind: RelationKind,
        range: SourceRange,
    ) -> Result<(), GraphValidationError> {
        for endpoint in &candidates {
            if let NodeId::External { module } = endpoint {
                self.externals.insert(module.clone());
            }
        }
        self.relations.push(Relation::new(
            from,
            RelationTarget::Ambiguous(candidates),
            kind,
            Confidence::Deterministic,
            vec![range],
            self.analyzer,
        )?);
        Ok(())
    }

    /// Enclosing scope chain for a caller, innermost first: the caller's own
    /// path (nested definitions live there), then each enclosing path, then
    /// module top level. Module-level calls search only the top level.
    fn scope_chain(&self, caller: Option<&SymbolId>) -> Vec<String> {
        let full = caller.and_then(|id| {
            self.symbols
                .iter()
                .find(|record| record.id == *id)
                .map(|record| record.full.clone())
        });
        let Some(mut path) = full else {
            return vec![String::new()];
        };
        let mut chain = vec![path.clone()];
        while let Some((parent, _)) = path.rsplit_once('.') {
            chain.push(parent.to_owned());
            path = parent.to_owned();
        }
        chain.push(String::new());
        chain
    }

    /// Owner chain of a defining scope for base-class lookup: the class's
    /// own owner path, then each enclosing path, then module top level.
    fn definition_scopes(&self, defined: &SymbolId) -> Vec<String> {
        let mut scopes = Vec::new();
        let scope = defined.scope_path();
        if !scope.is_empty() {
            let mut path = scope;
            scopes.push(path.clone());
            while let Some((parent, _)) = path.rsplit_once('.') {
                scopes.push(parent.to_owned());
                path = parent.to_owned();
            }
        }
        scopes.push(String::new());
        scopes
    }

    fn add_calls(&mut self, module: &ParsedModule) -> Result<(), GraphValidationError> {
        let file = module.file().clone();
        for call in module.calls() {
            let caller_node = call
                .caller()
                .map(|caller| NodeId::Symbol(caller.clone()))
                .unwrap_or_else(|| NodeId::Artifact(file.clone()));
            match call.receiver() {
                rootline_core::ir::CallReceiver::Absent => {
                    self.add_bare_call(&file, call, &caller_node)?;
                }
                rootline_core::ir::CallReceiver::Named(receiver)
                    if receiver.as_str() == "self" || receiver.as_str() == "cls" =>
                {
                    self.add_self_call(&file, call, &caller_node, receiver.as_str())?;
                }
                rootline_core::ir::CallReceiver::Named(receiver) => {
                    self.add_receiver_call(&file, call, &caller_node, receiver.as_str())?;
                }
                rootline_core::ir::CallReceiver::Opaque => {
                    self.push_diagnostic(
                        DiagnosticKind::Call,
                        &file,
                        call_subject(call),
                        "complex receiver expression; call target unproven".to_owned(),
                        call.range(),
                    );
                }
            }
        }
        Ok(())
    }

    /// Bare `f()` calls: nearest enclosing scope's functions and classes
    /// (methods are class attributes, never bare scope bindings), then
    /// `from`-imported names, then builtins. Anything else is diagnosed.
    fn add_bare_call(
        &mut self,
        file: &RepoPath,
        call: &rootline_core::ir::CallSite,
        caller_node: &NodeId,
    ) -> Result<(), GraphValidationError> {
        let name = call.name().as_str();
        for scope in self.scope_chain(call.caller()) {
            let mut found: Vec<SymbolId> = self
                .symbols_in_scope(file, &scope, name)
                .into_iter()
                .filter(|id| id.kind() != SymbolKind::Method)
                .collect();
            if found.is_empty() {
                continue;
            }
            if found.len() == 1 {
                self.push_relation(
                    caller_node.clone(),
                    NodeId::Symbol(found.remove(0)),
                    RelationKind::Calls,
                    Confidence::Deterministic,
                    vec![call.range()],
                )?;
                return Ok(());
            }
            self.push_ambiguous(
                caller_node.clone(),
                found.into_iter().map(NodeId::Symbol).collect(),
                RelationKind::Calls,
                call.range(),
            )?;
            return Ok(());
        }
        let binding = self
            .imports
            .get(file)
            .and_then(|imports| imports.bindings.get(name))
            .cloned();
        match binding {
            Some(Binding::Symbol { target }) => {
                self.push_relation(
                    caller_node.clone(),
                    NodeId::Symbol(target),
                    RelationKind::Calls,
                    Confidence::Deterministic,
                    vec![call.range()],
                )?;
            }
            Some(Binding::Submodule { .. }) => {
                self.push_diagnostic(
                    DiagnosticKind::Call,
                    file,
                    call_subject(call),
                    format!("'{name}' names an imported submodule, not a callable"),
                    call.range(),
                );
            }
            Some(Binding::ExternalName { module }) => {
                self.push_relation(
                    caller_node.clone(),
                    NodeId::External { module },
                    RelationKind::Calls,
                    Confidence::Deterministic,
                    vec![call.range()],
                )?;
            }
            Some(Binding::Module { .. }) => {
                self.push_diagnostic(
                    DiagnosticKind::Call,
                    file,
                    call_subject(call),
                    format!("'{name}' names an imported module, not a callable"),
                    call.range(),
                );
            }
            Some(Binding::Broken) => {}
            None if BUILTIN_NAMES.contains(&name) => {}
            None => {
                self.push_diagnostic(
                    DiagnosticKind::Call,
                    file,
                    call_subject(call),
                    format!("'{name}' has no in-scope definition or import binding"),
                    call.range(),
                );
            }
        }
        Ok(())
    }

    /// `self.m()` / `cls.m()`: methods of the enclosing class only. Calls
    /// that only work through inheritance (a parent's method) stay unknown:
    /// proving them needs the class hierarchy the graph is still building.
    fn add_self_call(
        &mut self,
        file: &RepoPath,
        call: &rootline_core::ir::CallSite,
        caller_node: &NodeId,
        receiver: &str,
    ) -> Result<(), GraphValidationError> {
        let name = call.name().as_str();
        let class_path = call.caller().and_then(|caller| {
            self.symbols
                .iter()
                .find(|record| record.id == *caller)
                .filter(|record| record.kind == SymbolKind::Method)
                .map(|record| record.parent.clone())
        });
        match class_path {
            Some(class_path) => {
                let found: Vec<SymbolId> = self
                    .symbols_in_scope(file, &class_path, name)
                    .into_iter()
                    .filter(|id| id.kind() == SymbolKind::Method)
                    .collect();
                match found.as_slice() {
                    [only] => {
                        self.push_relation(
                            caller_node.clone(),
                            NodeId::Symbol(only.clone()),
                            RelationKind::Calls,
                            Confidence::Deterministic,
                            vec![call.range()],
                        )?;
                    }
                    [] => {
                        self.push_diagnostic(
                            DiagnosticKind::Call,
                            file,
                            call_subject(call),
                            format!("'{name}' is not a method of '{class_path}'"),
                            call.range(),
                        );
                    }
                    many => {
                        self.push_ambiguous(
                            caller_node.clone(),
                            many.iter().cloned().map(NodeId::Symbol).collect(),
                            RelationKind::Calls,
                            call.range(),
                        )?;
                    }
                }
            }
            None => {
                self.push_diagnostic(
                    DiagnosticKind::Call,
                    file,
                    call_subject(call),
                    format!("'{receiver}' used outside a method"),
                    call.range(),
                );
            }
        }
        Ok(())
    }

    /// `mod.f()` calls resolve `mod` through import bindings: plain-imported
    /// modules, `from`-imported submodules, `from`-imported classes (methods
    /// of the class), or external modules. Parameters and locals are not
    /// bindings, so they are diagnosed instead of guessed.
    fn add_receiver_call(
        &mut self,
        file: &RepoPath,
        call: &rootline_core::ir::CallSite,
        caller_node: &NodeId,
        receiver: &str,
    ) -> Result<(), GraphValidationError> {
        let name = call.name().as_str();
        let binding = self
            .imports
            .get(file)
            .and_then(|imports| imports.bindings.get(receiver))
            .cloned();
        match binding {
            Some(Binding::Module { resolution }) => match resolution {
                Resolution::Resolved { target } => {
                    self.add_call_in_file(file, call, caller_node, &target, name)?;
                }
                Resolution::Ambiguous { candidates } => {
                    self.push_ambiguous(
                        caller_node.clone(),
                        candidates.into_iter().map(NodeId::Artifact).collect(),
                        RelationKind::Calls,
                        call.range(),
                    )?;
                }
                Resolution::External { module } => {
                    self.push_relation(
                        caller_node.clone(),
                        NodeId::External { module },
                        RelationKind::Calls,
                        Confidence::Deterministic,
                        vec![call.range()],
                    )?;
                }
                Resolution::Unknown { .. } => {}
            },
            Some(Binding::Symbol { target }) => {
                if target.kind() != SymbolKind::Class {
                    self.push_diagnostic(
                        DiagnosticKind::Call,
                        file,
                        call_subject(call),
                        format!("'{receiver}' names a {}, not a class", target.kind()),
                        call.range(),
                    );
                    return Ok(());
                }
                let parent = target.scope_path();
                let parent = if parent.is_empty() {
                    target.name().as_str().to_owned()
                } else {
                    format!("{}.{}", parent, target.name().as_str())
                };
                let found: Vec<SymbolId> = self
                    .symbols_in_scope(target.file(), &parent, name)
                    .into_iter()
                    .filter(|id| id.kind() == SymbolKind::Method)
                    .collect();
                match found.as_slice() {
                    [only] => {
                        self.push_relation(
                            caller_node.clone(),
                            NodeId::Symbol(only.clone()),
                            RelationKind::Calls,
                            Confidence::Deterministic,
                            vec![call.range()],
                        )?;
                    }
                    [] => {
                        self.push_diagnostic(
                            DiagnosticKind::Call,
                            file,
                            call_subject(call),
                            format!("'{name}' is not a method of '{receiver}'"),
                            call.range(),
                        );
                    }
                    many => {
                        self.push_ambiguous(
                            caller_node.clone(),
                            many.iter().cloned().map(NodeId::Symbol).collect(),
                            RelationKind::Calls,
                            call.range(),
                        )?;
                    }
                }
            }
            Some(Binding::Submodule { target }) => {
                self.add_call_in_file(file, call, caller_node, &target, name)?;
            }
            Some(Binding::ExternalName { module }) => {
                self.push_relation(
                    caller_node.clone(),
                    NodeId::External { module },
                    RelationKind::Calls,
                    Confidence::Deterministic,
                    vec![call.range()],
                )?;
            }
            Some(Binding::Broken) => {}
            None => {
                self.push_diagnostic(
                    DiagnosticKind::Call,
                    file,
                    call_subject(call),
                    format!("receiver '{receiver}' is not self, cls, or an import binding"),
                    call.range(),
                );
            }
        }
        Ok(())
    }

    /// Calls into another file target top-level functions and classes only:
    /// module-attribute access cannot reach nested definitions.
    fn add_call_in_file(
        &mut self,
        file: &RepoPath,
        call: &rootline_core::ir::CallSite,
        caller_node: &NodeId,
        target_file: &RepoPath,
        name: &str,
    ) -> Result<(), GraphValidationError> {
        let found: Vec<SymbolId> = self
            .top_level_symbols(target_file, name)
            .into_iter()
            .filter(|id| id.kind() == SymbolKind::Function || id.kind() == SymbolKind::Class)
            .collect();
        match found.as_slice() {
            [only] => {
                self.push_relation(
                    caller_node.clone(),
                    NodeId::Symbol(only.clone()),
                    RelationKind::Calls,
                    Confidence::Deterministic,
                    vec![call.range()],
                )?;
            }
            [] => {
                self.push_diagnostic(
                    DiagnosticKind::Call,
                    file,
                    call_subject(call),
                    format!(
                        "'{name}' is not defined in '{}'",
                        target_file.as_path().display()
                    ),
                    call.range(),
                );
            }
            many => {
                self.push_ambiguous(
                    caller_node.clone(),
                    many.iter().cloned().map(NodeId::Symbol).collect(),
                    RelationKind::Calls,
                    call.range(),
                )?;
            }
        }
        Ok(())
    }

    fn finish(mut self) -> Result<ResolvedGraph, GraphValidationError> {
        let mut nodes: BTreeMap<NodeId, NodeInfo> = BTreeMap::new();
        for file in self.index.files() {
            nodes.insert(
                NodeId::Artifact(file.clone()),
                NodeInfo::new(None, self.analyzer),
            );
        }
        for record in &self.symbols {
            nodes.insert(
                NodeId::Symbol(record.id.clone()),
                NodeInfo::new(Some(record.range), record.analyzer),
            );
        }
        for module in self.externals {
            nodes.insert(
                NodeId::External { module },
                NodeInfo::new(None, self.analyzer),
            );
        }
        let graph = Graph::try_new(self.metadata, nodes, std::mem::take(&mut self.relations))?;
        self.diagnostics.sort_by(|a, b| {
            (
                a.file().clone(),
                a.range().bytes().start(),
                a.kind() as u8,
                a.subject().to_owned(),
            )
                .cmp(&(
                    b.file().clone(),
                    b.range().bytes().start(),
                    b.kind() as u8,
                    b.subject().to_owned(),
                ))
        });
        Ok(ResolvedGraph {
            graph,
            diagnostics: self.diagnostics,
        })
    }
}
