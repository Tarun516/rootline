//! Python language adapter: Tree-sitter syntax into owned Code Intelligence IR.
//!
//! The adapter boundary owns every Tree-sitter type. Callers receive only
//! `rootline-core` IR values plus a typed adapter error, so the grammar and
//! parser runtime can evolve without touching downstream contracts.
//!
//! Scope of this revision:
//!
//! - module-level and nested `def` (including `async def`) and `class`
//!   declarations with lexical owners and source ranges;
//! - `import` / `from ... import` syntax facts, normalized to one fact per
//!   imported module;
//! - call sites with callee names and identifier receivers (no target
//!   resolution; that belongs to the graph stage);
//! - identifier and attribute base classes from class headers;
//! - partial recovery: files with syntax errors still yield their intact
//!   symbols with a [`rootline_core::ir::AnalysisStatus::Partial`] status.
//!
//! Known limits (tracked for the next adapter slice, not hidden): parameters,
//! return annotations, decorators (including calls inside decorators),
//! parametrized bases (`Generic[T]`) and class keywords, and column
//! coordinates are not extracted yet.

use rootline_core::RepoPath;
use rootline_core::ir::{
    AnalysisStatus, AnalyzerInfo, ByteOffset, ByteRange, CallReceiver, CallSite, CoordinateError,
    ImportStatement, ImportedName, Inheritance, IrValidationError, Language, LineNumber, LineRange,
    ModuleBody, ParseError, ParsedModule, SourceRange, Symbol, SymbolId, SymbolKind, SymbolName,
};
use std::fmt;
use std::path::Path;
use tree_sitter::{LanguageError, Node, Parser};

/// Identity of this analyzer for provenance records.
pub const ADAPTER_NAME: &str = "rootline-python-adapter";
/// Adapter version follows the engine crate; parser/grammar upgrades must bump it.
pub const ADAPTER_VERSION: &str = env!("CARGO_PKG_VERSION");

/// File extensions this adapter claims. Stubs (`.pyi`) parse with the same
/// grammar but have no fixture coverage yet, so they stay explicitly
/// unsupported rather than silently half-supported.
pub fn supports_extension(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("py")
}

/// Operational failures that prevent the adapter from doing its job. These
/// are distinct from [`AnalysisStatus`]: a syntax error in the analyzed file
/// yields `Ok` with a `Partial` status, while these variants mean the adapter
/// itself could not run.
#[derive(Debug)]
pub enum PythonAdapterError {
    GrammarUnavailable { source: LanguageError },
    ParseUnavailable,
    SourceTooLarge { bytes: u64 },
    Coordinate { source: CoordinateError },
    InvalidNode { detail: &'static str },
    InvalidSymbol { source: IrValidationError },
}

impl fmt::Display for PythonAdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GrammarUnavailable { source } => {
                write!(f, "Python grammar could not be loaded: {source}")
            }
            Self::ParseUnavailable => f.write_str("Python parser produced no syntax tree"),
            Self::SourceTooLarge { bytes } => {
                write!(
                    f,
                    "Python source ({bytes} bytes) exceeds the IR address space"
                )
            }
            Self::Coordinate { source } => write!(f, "unrepresentable source range: {source}"),
            Self::InvalidNode { detail } => {
                write!(f, "unexpected Python syntax shape: {detail}")
            }
            Self::InvalidSymbol { source } => write!(f, "invalid extracted symbol: {source}"),
        }
    }
}

impl std::error::Error for PythonAdapterError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GrammarUnavailable { source } => Some(source),
            Self::Coordinate { source } => Some(source),
            Self::InvalidSymbol { source } => Some(source),
            _ => None,
        }
    }
}

impl From<CoordinateError> for PythonAdapterError {
    fn from(source: CoordinateError) -> Self {
        Self::Coordinate { source }
    }
}

impl From<IrValidationError> for PythonAdapterError {
    fn from(source: IrValidationError) -> Self {
        Self::InvalidSymbol { source }
    }
}

/// Python adapter holding one reusable Tree-sitter parser.
///
/// The parser is stateful and neither `Clone` nor shareable across threads,
/// so it lives inside this owning context instead of being rebuilt per file.
/// Repository-scale parallelism stays a future, explicitly bounded design;
/// this revision parses one file per adapter sequentially.
pub struct PythonAdapter {
    parser: Parser,
}

impl PythonAdapter {
    /// Creates an adapter with the pinned Python grammar loaded.
    ///
    /// # Errors
    /// Returns [`PythonAdapterError::GrammarUnavailable`] when the grammar
    /// fails to load into the parser.
    pub fn new() -> Result<Self, PythonAdapterError> {
        let mut parser = Parser::new();
        parser
            .set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|source| PythonAdapterError::GrammarUnavailable { source })?;
        Ok(Self { parser })
    }

    /// Parses one UTF-8 Python source into normalized IR.
    ///
    /// File reading stays with the caller: this function borrows already
    /// loaded source so tests and future pipeline stages control I/O,
    /// encoding policy, and cancellation.
    ///
    /// # Errors
    /// Returns [`PythonAdapterError`] when the source cannot be parsed at
    /// all (oversized input, missing syntax tree, or coordinates outside the
    /// IR address space). Recoverable syntax errors instead yield `Ok` with a
    /// `Partial` status and per-error locations.
    pub fn parse(
        &mut self,
        file: &RepoPath,
        source: &str,
    ) -> Result<ParsedModule, PythonAdapterError> {
        let bytes = u64::try_from(source.len()).unwrap_or(u64::MAX);
        if bytes > u64::from(u32::MAX) {
            return Err(PythonAdapterError::SourceTooLarge { bytes });
        }
        let tree = self
            .parser
            .parse(source, None)
            .ok_or(PythonAdapterError::ParseUnavailable)?;
        let root = tree.root_node();

        let mut extraction = Extraction::new(file.clone());
        extraction.visit_node(root, source)?;
        extraction.finish()
    }
}

/// Accumulates adapter output during one syntax-tree walk.
struct Extraction {
    file: RepoPath,
    /// Lexical scope chain from observed nesting. Ownership always comes
    /// from this nesting, never from textual naming conventions.
    scope: Vec<ScopeFrame>,
    /// Declarations seen per scope path, kind, and name: the source of
    /// declaration indices, so legal redefinitions keep distinct identities.
    seen: std::collections::BTreeMap<(String, SymbolKind, String), u32>,
    symbols: Vec<Symbol>,
    imports: Vec<ImportStatement>,
    calls: Vec<CallSite>,
    inheritances: Vec<Inheritance>,
    parse_errors: Vec<ParseError>,
}

/// One lexically enclosing named block around the node being visited. The
/// frame carries the scope's symbol identity so call sites can name their
/// caller without re-deriving it from text.
struct ScopeFrame {
    id: SymbolId,
    is_class: bool,
}

impl Extraction {
    fn new(file: RepoPath) -> Self {
        Self {
            file,
            scope: Vec::new(),
            seen: std::collections::BTreeMap::new(),
            symbols: Vec::new(),
            imports: Vec::new(),
            calls: Vec::new(),
            inheritances: Vec::new(),
            parse_errors: Vec::new(),
        }
    }

    /// Dotted path of enclosing scope names for declaration counting.
    fn scope_key(&self) -> String {
        self.scope
            .iter()
            .map(|frame| frame.id.name().as_str())
            .collect::<Vec<_>>()
            .join(".")
    }

    /// Structural identity of the innermost enclosing scope (`None` at
    /// module top level).
    fn owner(&self) -> Option<SymbolId> {
        self.scope.last().map(|frame| frame.id.clone())
    }

    /// Declaration index for one more same-name declaration in the current
    /// scope: 0 for the first, counting up through legal redefinitions.
    fn declare(&mut self, kind: SymbolKind, name: &str) -> u32 {
        let key = (self.scope_key(), kind, name.to_owned());
        let index = self.seen.get(&key).copied().unwrap_or(0);
        self.seen.insert(key, index.saturating_add(1));
        index
    }

    /// Identity of the innermost enclosing scope (`None` for module top-level
    /// code), used as the caller of recorded call sites.
    fn caller(&self) -> Option<SymbolId> {
        self.scope.last().map(|frame| frame.id.clone())
    }

    fn finish(self) -> Result<ParsedModule, PythonAdapterError> {
        let status = if self.parse_errors.is_empty() {
            AnalysisStatus::Succeeded
        } else {
            AnalysisStatus::Partial {
                detail: format!(
                    "recovered {} syntax error(s); extracted symbols exclude unparseable regions",
                    self.parse_errors.len()
                ),
            }
        };
        Ok(ParsedModule::new(
            self.file.clone(),
            Language::Python,
            AnalyzerInfo::new(ADAPTER_NAME, ADAPTER_VERSION),
            status,
            ModuleBody {
                symbols: self.symbols,
                imports: self.imports,
                calls: self.calls,
                inheritances: self.inheritances,
                parse_errors: self.parse_errors,
            },
        ))
    }

    /// Walks one node: records error locations, extracts declarations and
    /// imports, and recurses so definitions inside `if`/`try`/decorated blocks
    /// are found wherever they nest.
    fn visit_node(&mut self, node: Node<'_>, source: &str) -> Result<(), PythonAdapterError> {
        if node.is_missing() {
            self.parse_errors.push(ParseError::new(
                enclosing_range(node)?,
                format!("missing {}", node.kind()),
            ));
            return Ok(());
        }
        if node.kind() == "ERROR" {
            self.parse_errors.push(ParseError::new(
                node_range(node)?,
                "unparseable syntax".to_owned(),
            ));
        }
        match node.kind() {
            "function_definition" => self.visit_function(node, source),
            "class_definition" => self.visit_class(node, source),
            "decorated_definition" => self.visit_decorated(node, source),
            "import_statement" => self.visit_plain_import(node, source),
            "import_from_statement" => self.visit_from_import(node, source),
            "call" => self.visit_call(node, source),
            _ => self.visit_children(node, source),
        }
    }

    fn visit_children(&mut self, node: Node<'_>, source: &str) -> Result<(), PythonAdapterError> {
        // All children, not only named ones: missing tokens such as a absent
        // `)` are anonymous, and skipping them would turn a recoverable syntax
        // error into a silent success.
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if !child.is_named() {
                if child.is_missing() {
                    self.parse_errors.push(ParseError::new(
                        enclosing_range(child)?,
                        format!("missing {}", child.kind()),
                    ));
                }
                continue;
            }
            self.visit_node(child, source)?;
        }
        Ok(())
    }

    fn visit_function(&mut self, node: Node<'_>, source: &str) -> Result<(), PythonAdapterError> {
        // A definition without a name carries no symbol identity; the
        // covering ERROR/MISSING nodes already record the syntax failure, so
        // the body is still searched for nested declarations.
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = SymbolName::new(node_text(name_node, source)?)?;
            // Only a `def` directly inside a class body is a method; deeper
            // nesting (a closure inside a method) stays a function owned by
            // its dotted scope path.
            let symbol_kind = if self.scope.last().is_some_and(|frame| frame.is_class) {
                SymbolKind::Method
            } else {
                SymbolKind::Function
            };
            let index = self.declare(symbol_kind, name.as_str());
            let id = SymbolId::new(self.file.clone(), symbol_kind, self.owner(), name, index);
            self.symbols.push(Symbol::new(id, node_range(node)?)?);
            let frame_id = self
                .symbols
                .last()
                .map(|symbol| symbol.id().clone())
                .ok_or(PythonAdapterError::InvalidNode {
                    detail: "just-pushed function symbol is missing",
                })?;
            self.scope.push(ScopeFrame {
                id: frame_id,
                is_class: false,
            });
            self.visit_children(node, source)?;
            self.scope.pop();
            Ok(())
        } else {
            self.visit_children(node, source)
        }
    }

    fn visit_class(&mut self, node: Node<'_>, source: &str) -> Result<(), PythonAdapterError> {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = SymbolName::new(node_text(name_node, source)?)?;
            let index = self.declare(SymbolKind::Class, name.as_str());
            let id = SymbolId::new(
                self.file.clone(),
                SymbolKind::Class,
                self.owner(),
                name,
                index,
            );
            self.symbols.push(Symbol::new(id, node_range(node)?)?);
            let frame_id = self
                .symbols
                .last()
                .map(|symbol| symbol.id().clone())
                .ok_or(PythonAdapterError::InvalidNode {
                    detail: "just-pushed class symbol is missing",
                })?;
            self.visit_superclasses(node, source, &frame_id)?;
            self.scope.push(ScopeFrame {
                id: frame_id,
                is_class: true,
            });
            self.visit_children(node, source)?;
            self.scope.pop();
            Ok(())
        } else {
            self.visit_children(node, source)
        }
    }

    fn visit_decorated(&mut self, node: Node<'_>, source: &str) -> Result<(), PythonAdapterError> {
        // The symbol range covers the `def`/`class` statement itself;
        // decorators stay uninterpreted in this revision (see module docs).
        // Because decorator subtrees are never walked, call expressions
        // inside decorators are not recorded as call sites either.
        if let Some(definition) = node.child_by_field_name("definition") {
            self.visit_node(definition, source)?;
        }
        Ok(())
    }

    /// Records one call expression: the callee's final name segment and how
    /// it was qualified. Bare `f()` calls may resolve through scope and
    /// imports; `receiver.name()` calls resolve through the receiver; complex
    /// qualifiers (`factory().make()`, `a[0]()`) are opaque so the graph
    /// stage never matches their bare name against a local definition.
    fn visit_call(&mut self, node: Node<'_>, source: &str) -> Result<(), PythonAdapterError> {
        let Some(function) = node.child_by_field_name("function") else {
            return self.visit_children(node, source);
        };
        let (receiver, name) = match function.kind() {
            "identifier" => (CallReceiver::Absent, node_text(function, source)?),
            "attribute" => {
                let attribute = function.child_by_field_name("attribute").ok_or(
                    PythonAdapterError::InvalidNode {
                        detail: "attribute call without attribute name",
                    },
                )?;
                let receiver = match function.child_by_field_name("object") {
                    Some(object) if object.kind() == "identifier" => {
                        CallReceiver::Named(SymbolName::new(node_text(object, source)?)?)
                    }
                    _ => CallReceiver::Opaque,
                };
                (receiver, node_text(attribute, source)?)
            }
            _ => return self.visit_children(node, source),
        };
        self.calls.push(CallSite::new(
            self.caller(),
            receiver,
            SymbolName::new(name)?,
            node_range(node)?,
        ));
        self.visit_children(node, source)
    }

    /// Records the identifier and attribute base classes named in a class
    /// header. Parametrized bases (`Generic[T]`), calls, and keywords
    /// (`metaclass=...`) are not extracted in this revision: recording their
    /// raw text as a dependency target would manufacture bogus externals, and
    /// skipping them keeps the gap explicit instead of hidden.
    fn visit_superclasses(
        &mut self,
        node: Node<'_>,
        source: &str,
        class: &SymbolId,
    ) -> Result<(), PythonAdapterError> {
        let Some(superclasses) = node.child_by_field_name("superclasses") else {
            return Ok(());
        };
        let mut cursor = superclasses.walk();
        for child in superclasses.named_children(&mut cursor) {
            if child.kind() != "identifier" && child.kind() != "attribute" {
                continue;
            }
            self.inheritances.push(Inheritance::new(
                class.clone(),
                node_text(child, source)?.to_owned(),
                node_range(child)?,
            ));
        }
        Ok(())
    }

    fn visit_plain_import(
        &mut self,
        node: Node<'_>,
        source: &str,
    ) -> Result<(), PythonAdapterError> {
        let range = node_range(node)?;
        let mut cursor = node.walk();
        // Only `name`-field children are imported items; matching every
        // `dotted_name` would also catch the `module_name` side of the
        // statement and report the module as importing itself.
        for child in node.children_by_field_name("name", &mut cursor) {
            if child.kind() != "dotted_name" && child.kind() != "aliased_import" {
                continue;
            }
            let (module, bound) = plain_import_target(child, source)?;
            self.imports.push(ImportStatement::new(
                range,
                0,
                Some(module.clone()),
                vec![ImportedName::new(module, SymbolName::new(&bound)?)],
                false,
                false,
            ));
        }
        Ok(())
    }

    fn visit_from_import(
        &mut self,
        node: Node<'_>,
        source: &str,
    ) -> Result<(), PythonAdapterError> {
        let range = node_range(node)?;
        let (level, module) = from_import_module(node, source)?;
        let mut names = Vec::new();
        let mut is_wildcard = false;
        // As with plain imports, only `name`-field children are imported
        // items; the `module_name` child must not leak into the name list.
        let mut names_cursor = node.walk();
        for child in node.children_by_field_name("name", &mut names_cursor) {
            match child.kind() {
                "dotted_name" => {
                    let text = node_text(child, source)?;
                    names.push(ImportedName::new(text.to_owned(), SymbolName::new(text)?));
                }
                "aliased_import" => {
                    let target = child.child_by_field_name("name").ok_or(
                        PythonAdapterError::InvalidNode {
                            detail: "aliased from-import without target",
                        },
                    )?;
                    let alias = child.child_by_field_name("alias").ok_or(
                        PythonAdapterError::InvalidNode {
                            detail: "aliased from-import without alias",
                        },
                    )?;
                    names.push(ImportedName::new(
                        node_text(target, source)?.to_owned(),
                        SymbolName::new(node_text(alias, source)?)?,
                    ));
                }
                _ => {}
            }
        }
        let mut wildcard_cursor = node.walk();
        for child in node.named_children(&mut wildcard_cursor) {
            if child.kind() == "wildcard_import" {
                is_wildcard = true;
            }
        }
        self.imports.push(ImportStatement::new(
            range,
            level,
            module,
            names,
            is_wildcard,
            true,
        ));
        Ok(())
    }
}

/// Splits `import a.b` / `import a.b as c` into the module path and the name
/// it binds (`a` by default, the alias when present).
fn plain_import_target(
    node: Node<'_>,
    source: &str,
) -> Result<(String, String), PythonAdapterError> {
    if node.kind() == "aliased_import" {
        let target = node
            .child_by_field_name("name")
            .ok_or(PythonAdapterError::InvalidNode {
                detail: "aliased import without target",
            })?;
        let alias = node
            .child_by_field_name("alias")
            .ok_or(PythonAdapterError::InvalidNode {
                detail: "aliased import without alias",
            })?;
        Ok((
            node_text(target, source)?.to_owned(),
            node_text(alias, source)?.to_owned(),
        ))
    } else {
        let module = node_text(node, source)?.to_owned();
        let bound = module
            .split('.')
            .next()
            .ok_or(PythonAdapterError::InvalidNode {
                detail: "imported module path is empty",
            })?
            .to_owned();
        Ok((module, bound))
    }
}

/// Reads the module side of `from ... import ...`: the leading-dot level and
/// the dotted path (`None` for bare `from . import name`).
fn from_import_module(
    node: Node<'_>,
    source: &str,
) -> Result<(u32, Option<String>), PythonAdapterError> {
    let module_node = node.child_by_field_name("module_name");
    match module_node {
        None => Ok((0, None)),
        Some(module_node) if module_node.kind() == "dotted_name" => {
            Ok((0, Some(node_text(module_node, source)?.to_owned())))
        }
        Some(relative) => {
            let mut level = 0;
            let mut module = None;
            let mut cursor = relative.walk();
            for child in relative.named_children(&mut cursor) {
                if child.kind() == "import_prefix" {
                    let dots = node_text(child, source)?;
                    level =
                        u32::try_from(dots.len()).map_err(|_| PythonAdapterError::InvalidNode {
                            detail: "relative import level does not fit",
                        })?;
                } else if child.kind() == "dotted_name" {
                    module = Some(node_text(child, source)?.to_owned());
                }
            }
            Ok((level, module))
        }
    }
}

/// Converts a syntax node's byte span and rows into IR coordinates.
fn node_range(node: Node<'_>) -> Result<SourceRange, PythonAdapterError> {
    let bytes = ByteRange::new(
        ByteOffset::new(node.start_byte())?,
        ByteOffset::new(node.end_byte())?,
    )?;
    let lines = LineRange::new(
        LineNumber::from_zero_based(node.start_position().row)?,
        LineNumber::from_zero_based(node.end_position().row)?,
    )?;
    Ok(SourceRange::new(bytes, lines))
}

/// Error location for zero-width missing nodes: the smallest enclosing
/// non-empty ancestor range, since an empty range carries no evidence.
fn enclosing_range(node: Node<'_>) -> Result<SourceRange, PythonAdapterError> {
    let mut current = Some(node);
    while let Some(candidate) = current {
        if candidate.start_byte() < candidate.end_byte() {
            return node_range(candidate);
        }
        current = candidate.parent();
    }
    Err(PythonAdapterError::InvalidNode {
        detail: "syntax error without an enclosing range",
    })
}

/// Borrows the source text covered by a node without lossy conversion.
fn node_text<'a>(node: Node<'_>, source: &'a str) -> Result<&'a str, PythonAdapterError> {
    source
        .get(node.start_byte()..node.end_byte())
        .ok_or(PythonAdapterError::InvalidNode {
            detail: "syntax node range escapes its source",
        })
}
