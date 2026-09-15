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
//! - partial recovery: files with syntax errors still yield their intact
//!   symbols with a [`rootline_core::ir::AnalysisStatus::Partial`] status.
//!
//! Known limits (tracked for the next adapter slice, not hidden): parameters,
//! return annotations, decorators, call/reference extraction, and column
//! coordinates are not extracted yet.

use rootline_core::RepoPath;
use rootline_core::ir::{
    AnalysisStatus, AnalyzerInfo, ByteOffset, ByteRange, CoordinateError, ImportStatement,
    IrValidationError, Language, LineNumber, LineRange, ParseError, ParsedModule, SourceRange,
    Symbol, SymbolId, SymbolKind, SymbolName,
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
    /// Lexical scope chain from observed nesting, e.g. `Outer.Inner` for a
    /// method of a nested class. Ownership always comes from this nesting,
    /// never from textual naming conventions.
    scope: Vec<ScopeFrame>,
    symbols: Vec<Symbol>,
    imports: Vec<ImportStatement>,
    parse_errors: Vec<ParseError>,
}

/// One lexically enclosing named block around the node being visited.
struct ScopeFrame {
    name: String,
    is_class: bool,
}

impl Extraction {
    fn new(file: RepoPath) -> Self {
        Self {
            file,
            scope: Vec::new(),
            symbols: Vec::new(),
            imports: Vec::new(),
            parse_errors: Vec::new(),
        }
    }

    /// Dotted path of enclosing scopes (`None` at module top level).
    fn owner(&self) -> Result<Option<SymbolName>, PythonAdapterError> {
        if self.scope.is_empty() {
            return Ok(None);
        }
        let path = self
            .scope
            .iter()
            .map(|frame| frame.name.as_str())
            .collect::<Vec<_>>()
            .join(".");
        SymbolName::new(&path)
            .map(Some)
            .map_err(PythonAdapterError::from)
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
            Language::Python,
            AnalyzerInfo::new(ADAPTER_NAME, ADAPTER_VERSION),
            status,
            self.symbols,
            self.imports,
            self.parse_errors,
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
            let frame_name = name.as_str().to_owned();
            let id = SymbolId::new(self.file.clone(), symbol_kind, self.owner()?, name);
            self.symbols.push(Symbol::new(id, node_range(node)?)?);
            self.scope.push(ScopeFrame {
                name: frame_name,
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
            let frame_name = name.as_str().to_owned();
            let id = SymbolId::new(self.file.clone(), SymbolKind::Class, self.owner()?, name);
            self.symbols.push(Symbol::new(id, node_range(node)?)?);
            self.scope.push(ScopeFrame {
                name: frame_name,
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
        if let Some(definition) = node.child_by_field_name("definition") {
            self.visit_node(definition, source)?;
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
                Some(module),
                vec![SymbolName::new(&bound)?],
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
                "dotted_name" => names.push(SymbolName::new(node_text(child, source)?)?),
                "aliased_import" => {
                    let alias = child.child_by_field_name("alias").ok_or(
                        PythonAdapterError::InvalidNode {
                            detail: "aliased from-import without alias",
                        },
                    )?;
                    names.push(SymbolName::new(node_text(alias, source)?)?);
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

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "fixture I/O and adapter setup failure must fail the test immediately"
)]
mod tests {
    use super::*;
    use rootline_core::ir::ParsedModule;

    fn fixture_path(name: &str) -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/python")
            .join(name)
    }

    fn parse_fixture(name: &str) -> ParsedModule {
        let source = std::fs::read_to_string(fixture_path(name)).expect("read Python fixture");
        let repo_path = RepoPath::new(Path::new(name)).expect("fixture name is relative");
        PythonAdapter::new()
            .expect("adapter builds")
            .parse(&repo_path, &source)
            .expect("fixture parses without adapter failure")
    }

    fn symbol_summary(symbol: &Symbol) -> (SymbolKind, Option<&str>, &str) {
        (
            symbol.id().kind(),
            symbol.id().owner().map(SymbolName::as_str),
            symbol.id().name().as_str(),
        )
    }

    fn range_summary(symbol: &Symbol) -> (u32, u32, u32, u32) {
        let range = symbol.range();
        (
            range.bytes().start().get(),
            range.bytes().end().get(),
            range.lines().start().get(),
            range.lines().end().get(),
        )
    }

    #[test]
    fn extracts_symbols_with_lexical_owners() {
        let module = parse_fixture("symbols_basic.py");
        assert_eq!(module.status(), &AnalysisStatus::Succeeded);
        let summary: Vec<_> = module.symbols().iter().map(symbol_summary).collect();
        assert_eq!(
            summary,
            [
                (SymbolKind::Function, None, "train"),
                (SymbolKind::Class, None, "Model"),
                (SymbolKind::Method, Some("Model"), "fit"),
                (SymbolKind::Method, Some("Model"), "predict"),
                (SymbolKind::Function, None, "outer"),
                (SymbolKind::Function, Some("outer"), "inner"),
                (SymbolKind::Function, None, "served"),
                (SymbolKind::Class, None, "Outer"),
                (SymbolKind::Class, Some("Outer"), "Inner"),
                (SymbolKind::Method, Some("Outer.Inner"), "method"),
            ]
        );
    }

    #[test]
    fn pins_exact_source_ranges() {
        // Ranges below were verified against the fixture bytes: each range
        // starts at the declaration keyword (excluding indentation and
        // decorators) and ends before the trailing newline, with one-based
        // inclusive line spans.
        let module = parse_fixture("symbols_basic.py");
        let ranges: Vec<_> = module.symbols().iter().map(range_summary).collect();
        assert_eq!(
            ranges,
            [
                (45, 81, 4, 5),     // train
                (84, 191, 8, 13),   // Model
                (101, 141, 9, 10),  // Model.fit
                (147, 191, 12, 13), // Model.predict
                (194, 258, 16, 20), // outer
                (211, 240, 17, 18), // outer.inner
                (272, 301, 24, 25), // served (decorator on line 23 excluded)
                (304, 383, 28, 31), // Outer
                (321, 383, 29, 31), // Outer.Inner
                (342, 383, 30, 31), // Outer.Inner.method
            ]
        );
        // Byte ranges arrive in deterministic declaration order.
        let starts: Vec<u32> = ranges.iter().map(|range| range.0).collect();
        let mut ordered = starts.clone();
        ordered.sort();
        assert_eq!(starts, ordered);
    }

    #[test]
    fn extracts_import_statements_per_module() {
        let module = parse_fixture("imports_basic.py");
        assert_eq!(module.status(), &AnalysisStatus::Succeeded);
        let summary: Vec<_> = module
            .imports()
            .iter()
            .map(|import| {
                (
                    import.level(),
                    import.module().map(str::to_owned),
                    import
                        .names()
                        .iter()
                        .map(|name| name.as_str().to_owned())
                        .collect::<Vec<_>>(),
                    import.is_wildcard(),
                    range_of(import.range()),
                )
            })
            .collect();
        assert_eq!(
            summary,
            [
                (
                    0,
                    Some("os".to_owned()),
                    vec!["os".to_owned()],
                    false,
                    (47, 56, 2, 2)
                ),
                (
                    0,
                    Some("sys".to_owned()),
                    vec!["system".to_owned()],
                    false,
                    (57, 77, 3, 3)
                ),
                (
                    0,
                    Some("pkg.sub".to_owned()),
                    vec!["alias".to_owned()],
                    false,
                    (78, 101, 4, 4)
                ),
                (
                    0,
                    Some("pathlib".to_owned()),
                    vec!["Path".to_owned()],
                    false,
                    (103, 127, 6, 6)
                ),
                (1, None, vec!["sibling".to_owned()], false, (128, 149, 7, 7)),
                (
                    2,
                    Some("pkg".to_owned()),
                    vec!["renamed".to_owned()],
                    false,
                    (150, 184, 8, 8)
                ),
                (
                    0,
                    Some("package".to_owned()),
                    vec!["first".to_owned(), "two".to_owned()],
                    false,
                    (185, 225, 9, 9)
                ),
                (
                    0,
                    Some("module".to_owned()),
                    vec![],
                    true,
                    (226, 246, 10, 10)
                ),
            ]
        );
    }

    #[test]
    fn recovers_partial_status_from_syntax_errors() {
        let module = parse_fixture("syntax_error.py");
        match module.status() {
            AnalysisStatus::Partial { .. } => {}
            status => panic!("expected partial status, got {status}"),
        }
        // Intact declarations survive recovery.
        let names: Vec<_> = module
            .symbols()
            .iter()
            .map(|symbol| symbol.id().name().as_str())
            .collect();
        assert_eq!(names, ["healthy", "broken"]);
        // The error location points at the broken line with real coordinates.
        assert_eq!(module.parse_errors().len(), 1);
        let error = &module.parse_errors()[0];
        assert_eq!(error.range().lines().start().get(), 8);
        assert_eq!(error.range().lines().end().get(), 8);
    }

    #[test]
    fn empty_file_succeeds_with_no_symbols() {
        // An empty successful result is only meaningful with a Succeeded
        // status: status and content are asserted together, never inferred
        // from content alone.
        let module = parse_fixture("empty.py");
        assert_eq!(module.status(), &AnalysisStatus::Succeeded);
        assert!(module.symbols().is_empty());
        assert!(module.imports().is_empty());
        assert!(module.parse_errors().is_empty());
    }

    #[test]
    fn python_extension_support_is_explicit() {
        assert!(supports_extension(Path::new("src/module.py")));
        assert!(!supports_extension(Path::new("src/module.pyi")));
        assert!(!supports_extension(Path::new("src/module.ts")));
        assert!(!supports_extension(Path::new("src/module")));
    }

    #[test]
    fn provenance_identifies_adapter_and_version() {
        let module = parse_fixture("symbols_basic.py");
        assert_eq!(module.language(), Language::Python);
        assert_eq!(module.analyzer().name(), ADAPTER_NAME);
        assert_eq!(module.analyzer().version(), ADAPTER_VERSION);
    }

    fn range_of(range: SourceRange) -> (u32, u32, u32, u32) {
        (
            range.bytes().start().get(),
            range.bytes().end().get(),
            range.lines().start().get(),
            range.lines().end().get(),
        )
    }
}
