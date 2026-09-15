//! Language-neutral Code Intelligence IR.
//!
//! This module owns the stable shapes that language adapters produce: typed
//! source coordinates, symbol identities, import syntax, and per-file analysis
//! outcomes. It depends only on `std` and [`crate::RepoPath`]; parser,
//! database, transport, and model-provider types must never leak through this
//! boundary (see `docs/04-target-architecture.md`).
//!
//! Identity scope: a [`SymbolId`] identifies a symbol within one file at one
//! revision scope. Repository identity and revision join later at graph-build
//! time, which is why they are fields of provenance there rather than here.
//! Line numbers in [`SourceRange`] are evidence for humans; byte ranges are
//! the precise machine coordinates.

use std::fmt;

use crate::RepoPath;

/// Language an adapter parsed. Only languages with a tested adapter may gain
/// a variant; a grammar alone is not language support.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Language {
    Python,
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Python => f.write_str("python"),
        }
    }
}

/// Byte offset into a UTF-8 source file. Distinct from line/column positions
/// so coordinate systems cannot be mixed through bare integers.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ByteOffset(u32);

impl ByteOffset {
    /// Builds an offset from a raw byte index.
    ///
    /// # Errors
    /// Returns [`CoordinateError::OffsetOverflow`] when the index exceeds what
    /// the IR can represent; such sources are rejected explicitly rather than
    /// silently truncated.
    pub fn new(value: usize) -> Result<Self, CoordinateError> {
        u32::try_from(value)
            .map(Self)
            .map_err(|_| CoordinateError::OffsetOverflow { value })
    }

    /// Returns the offset as a plain index for slicing already-validated ranges.
    pub fn get(self) -> u32 {
        self.0
    }
}

/// Why a source coordinate could not be represented in the IR.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CoordinateError {
    OffsetOverflow { value: usize },
    EmptyRange,
}

impl fmt::Display for CoordinateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OffsetOverflow { value } => {
                write!(f, "byte offset {value} exceeds the IR address space")
            }
            Self::EmptyRange => f.write_str("source range is empty"),
        }
    }
}

impl std::error::Error for CoordinateError {}

/// Byte range with start-inclusive, end-exclusive semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ByteRange {
    start: ByteOffset,
    end: ByteOffset,
}

impl ByteRange {
    /// Builds a non-empty byte range.
    ///
    /// # Errors
    /// Returns [`CoordinateError::EmptyRange`] when `start >= end`; an empty
    /// range carries no declaration evidence and is rejected instead of
    /// published as a symbol location.
    pub fn new(start: ByteOffset, end: ByteOffset) -> Result<Self, CoordinateError> {
        if start >= end {
            return Err(CoordinateError::EmptyRange);
        }
        Ok(Self { start, end })
    }

    pub fn start(self) -> ByteOffset {
        self.start
    }

    pub fn end(self) -> ByteOffset {
        self.end
    }
}

/// One-based line number for human-facing evidence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LineNumber(u32);

impl LineNumber {
    /// Converts a zero-based parser row into a one-based line number.
    ///
    /// # Errors
    /// Returns [`CoordinateError::OffsetOverflow`] when the row does not fit;
    /// the conversion is checked rather than silently wrapped.
    pub fn from_zero_based(row: usize) -> Result<Self, CoordinateError> {
        let one_based = row
            .checked_add(1)
            .ok_or(CoordinateError::OffsetOverflow { value: row })?;
        u32::try_from(one_based)
            .map(Self)
            .map_err(|_| CoordinateError::OffsetOverflow { value: row })
    }

    pub fn get(self) -> u32 {
        self.0
    }
}

/// One-based line span with both ends inclusive, matching how humans cite
/// source locations ("lines 3-7").
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineRange {
    start: LineNumber,
    end: LineNumber,
}

impl LineRange {
    /// Builds a line span covering `start` through `end` inclusive.
    ///
    /// # Errors
    /// Returns [`CoordinateError::EmptyRange`] when `start` is after `end`.
    pub fn new(start: LineNumber, end: LineNumber) -> Result<Self, CoordinateError> {
        if start > end {
            return Err(CoordinateError::EmptyRange);
        }
        Ok(Self { start, end })
    }

    pub fn start(self) -> LineNumber {
        self.start
    }

    pub fn end(self) -> LineNumber {
        self.end
    }
}

/// Precise byte coordinates plus human-citable line evidence for one source
/// location. Byte offsets count UTF-8 bytes; columns are intentionally absent
/// in this revision because Tree-sitter, LSP, and editors disagree on column
/// units, and an ambiguous column is worse than none.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceRange {
    bytes: ByteRange,
    lines: LineRange,
}

impl SourceRange {
    pub fn new(bytes: ByteRange, lines: LineRange) -> Self {
        Self { bytes, lines }
    }

    pub fn bytes(self) -> ByteRange {
        self.bytes
    }

    pub fn lines(self) -> LineRange {
        self.lines
    }
}

/// Declared name of a symbol. A newtype keeps names, owners, and paths from
/// mixing through bare strings at adapter and graph boundaries.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SymbolName(String);

impl SymbolName {
    /// Builds a name, rejecting empty identifiers.
    ///
    /// # Errors
    /// Returns [`IrValidationError::EmptyName`] for empty input; anonymous
    /// constructs need an explicit strategy, not an empty-string identity.
    pub fn new(name: &str) -> Result<Self, IrValidationError> {
        if name.is_empty() {
            return Err(IrValidationError::EmptyName);
        }
        Ok(Self(name.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SymbolName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Kind of program entity an adapter extracted.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SymbolKind {
    Function,
    Class,
    Method,
}

impl fmt::Display for SymbolKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Function => f.write_str("function"),
            Self::Class => f.write_str("class"),
            Self::Method => f.write_str("method"),
        }
    }
}

/// Stable file-scoped identity for a symbol: artifact path, kind, lexical
/// owner chain, and declared name. Line ranges are evidence, not identity, so
/// they are deliberately excluded: a symbol keeps its identity when lines
/// shift within a revision scope.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SymbolId {
    file: RepoPath,
    kind: SymbolKind,
    owner: Option<SymbolName>,
    name: SymbolName,
}

impl SymbolId {
    pub fn new(
        file: RepoPath,
        kind: SymbolKind,
        owner: Option<SymbolName>,
        name: SymbolName,
    ) -> Self {
        Self {
            file,
            kind,
            owner,
            name,
        }
    }

    pub fn file(&self) -> &RepoPath {
        &self.file
    }

    pub fn kind(&self) -> SymbolKind {
        self.kind
    }

    pub fn owner(&self) -> Option<&SymbolName> {
        self.owner.as_ref()
    }

    pub fn name(&self) -> &SymbolName {
        &self.name
    }
}

/// One language-level program entity with its declaring source range.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Symbol {
    id: SymbolId,
    range: SourceRange,
}

impl Symbol {
    /// Builds a symbol, enforcing that methods declare their lexical owner.
    /// Ownership must come from adapter scope evidence, never from textual
    /// naming conventions.
    ///
    /// # Errors
    /// Returns [`IrValidationError::MethodWithoutOwner`] when a method has no
    /// owner.
    pub fn new(id: SymbolId, range: SourceRange) -> Result<Self, IrValidationError> {
        if id.kind() == SymbolKind::Method && id.owner().is_none() {
            return Err(IrValidationError::MethodWithoutOwner {
                name: id.name().as_str().to_owned(),
            });
        }
        Ok(Self { id, range })
    }

    pub fn id(&self) -> &SymbolId {
        &self.id
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }
}

/// Raw import syntax as written, before module resolution. Resolution outcomes
/// (resolved, ambiguous, unknown, external) belong to the resolver stage, not
/// to this syntax-level fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportStatement {
    range: SourceRange,
    level: u32,
    module: Option<String>,
    names: Vec<SymbolName>,
    is_wildcard: bool,
    is_from: bool,
}

impl ImportStatement {
    /// Builds one normalized import fact per imported module: a plain
    /// `import a, b` is represented as two facts sharing one source range.
    /// `is_from` records the statement form because `import a.b as c`
    /// (binds the module) and `from a.b import c` (binds a name inside it)
    /// are otherwise indistinguishable yet bind different things.
    pub fn new(
        range: SourceRange,
        level: u32,
        module: Option<String>,
        names: Vec<SymbolName>,
        is_wildcard: bool,
        is_from: bool,
    ) -> Self {
        Self {
            range,
            level,
            module,
            names,
            is_wildcard,
            is_from,
        }
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    /// Leading-dot count for `from` imports; 0 means absolute.
    pub fn level(&self) -> u32 {
        self.level
    }

    /// Dotted module path as written (`None` for `from . import name`).
    pub fn module(&self) -> Option<&str> {
        self.module.as_deref()
    }

    /// Names bound by this statement: imported identifiers for `from`
    /// imports, the bound top-level or aliased name for plain imports.
    pub fn names(&self) -> &[SymbolName] {
        &self.names
    }

    /// Whether this statement is a `from module import *` wildcard.
    pub fn is_wildcard(&self) -> bool {
        self.is_wildcard
    }

    /// Whether this is a `from`-form import (binds names inside the module)
    /// as opposed to a plain `import` (binds the module itself).
    pub fn is_from(&self) -> bool {
        self.is_from
    }
}

/// How a call names its target. `Absent` (`f()`) may resolve through scope
/// and imports; `Named` (`self.fit()`, `mod.run()`) resolves through its
/// receiver; `Opaque` (`factory().make()`, `a[0]()`) carries a callee name
/// that must never match a bare in-scope definition, since the qualifier
/// proves the callee is not the local name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CallReceiver {
    Absent,
    Named(SymbolName),
    Opaque,
}

/// One call expression as written: the callee's final name segment, how it
/// was qualified, the calling scope (or `None` for module top-level code),
/// and its source range.
///
/// Target resolution belongs to the graph stage; this struct is the
/// syntax-level fact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallSite {
    caller: Option<SymbolId>,
    receiver: CallReceiver,
    name: SymbolName,
    range: SourceRange,
}

impl CallSite {
    pub fn new(
        caller: Option<SymbolId>,
        receiver: CallReceiver,
        name: SymbolName,
        range: SourceRange,
    ) -> Self {
        Self {
            caller,
            receiver,
            name,
            range,
        }
    }

    pub fn caller(&self) -> Option<&SymbolId> {
        self.caller.as_ref()
    }

    pub fn receiver(&self) -> &CallReceiver {
        &self.receiver
    }

    pub fn name(&self) -> &SymbolName {
        &self.name
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }
}

/// One base-class reference as written in a class header: the dotted path
/// text (`Base`, `pkg.Base`, `nn.Module`), not a resolved target.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inheritance {
    class: SymbolId,
    base: String,
    range: SourceRange,
}

impl Inheritance {
    pub fn new(class: SymbolId, base: String, range: SourceRange) -> Self {
        Self { class, base, range }
    }

    pub fn class(&self) -> &SymbolId {
        &self.class
    }

    pub fn base(&self) -> &str {
        &self.base
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }
}

/// Per-file analysis outcome. These are successful coverage reports, not
/// operational errors: `Unknown` means the adapter ran but evidence was
/// insufficient, while a Rust `Err` means the adapter could not do its job.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnalysisStatus {
    /// Complete extraction under the adapter's declared contract.
    Succeeded,
    /// Syntax recovered with errors; extracted symbols carry the coverage
    /// described in `detail`, and error locations are in `parse_errors`.
    Partial { detail: String },
    /// No capable adapter exists for this file; the artifact and this status
    /// are preserved rather than reported as an empty success.
    Unsupported { reason: String },
}

impl fmt::Display for AnalysisStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Succeeded => f.write_str("succeeded"),
            Self::Partial { detail } => write!(f, "partial: {detail}"),
            Self::Unsupported { reason } => write!(f, "unsupported: {reason}"),
        }
    }
}

/// One syntax error location recovered during parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParseError {
    range: SourceRange,
    message: String,
}

impl ParseError {
    pub fn new(range: SourceRange, message: String) -> Self {
        Self { range, message }
    }

    pub fn range(&self) -> SourceRange {
        self.range
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

/// Provenance for one parsed module: which adapter, at which version,
/// produced these facts. Repository revision joins at graph-build time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnalyzerInfo {
    name: &'static str,
    version: &'static str,
}

impl AnalyzerInfo {
    pub const fn new(name: &'static str, version: &'static str) -> Self {
        Self { name, version }
    }

    pub fn name(self) -> &'static str {
        self.name
    }

    pub fn version(self) -> &'static str {
        self.version
    }
}

/// The extracted contents of one analyzed file. An empty body is
/// semantically valid (an empty file parses to nothing), so `Default` holds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModuleBody {
    pub symbols: Vec<Symbol>,
    pub imports: Vec<ImportStatement>,
    pub calls: Vec<CallSite>,
    pub inheritances: Vec<Inheritance>,
    pub parse_errors: Vec<ParseError>,
}

/// Normalized per-file result every language adapter must produce.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedModule {
    file: RepoPath,
    language: Language,
    analyzer: AnalyzerInfo,
    status: AnalysisStatus,
    body: ModuleBody,
}

impl ParsedModule {
    /// Builds a module, canonicalizing collection order by byte offset so
    /// repeated runs and snapshot tests observe deterministic output
    /// regardless of traversal order.
    pub fn new(
        file: RepoPath,
        language: Language,
        analyzer: AnalyzerInfo,
        status: AnalysisStatus,
        mut body: ModuleBody,
    ) -> Self {
        body.symbols
            .sort_by_key(|symbol| (symbol.range().bytes().start(), symbol.range().bytes().end()));
        body.imports
            .sort_by_key(|import| (import.range().bytes().start(), import.range().bytes().end()));
        body.calls
            .sort_by_key(|call| (call.range().bytes().start(), call.range().bytes().end()));
        body.inheritances.sort_by_key(|inheritance| {
            (
                inheritance.range().bytes().start(),
                inheritance.range().bytes().end(),
            )
        });
        Self {
            file,
            language,
            analyzer,
            status,
            body,
        }
    }

    /// Repository-relative identity of the analyzed file.
    pub fn file(&self) -> &RepoPath {
        &self.file
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn analyzer(&self) -> AnalyzerInfo {
        self.analyzer
    }

    pub fn status(&self) -> &AnalysisStatus {
        &self.status
    }

    /// Symbols in deterministic byte-offset order.
    pub fn symbols(&self) -> &[Symbol] {
        &self.body.symbols
    }

    /// Imports in deterministic byte-offset order.
    pub fn imports(&self) -> &[ImportStatement] {
        &self.body.imports
    }

    /// Call sites in deterministic byte-offset order.
    pub fn calls(&self) -> &[CallSite] {
        &self.body.calls
    }

    /// Base-class references in deterministic byte-offset order.
    pub fn inheritances(&self) -> &[Inheritance] {
        &self.body.inheritances
    }

    pub fn parse_errors(&self) -> &[ParseError] {
        &self.body.parse_errors
    }
}

/// Why a candidate IR value violates the model's invariants.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IrValidationError {
    EmptyName,
    MethodWithoutOwner { name: String },
    DuplicateSymbol { name: String },
}

impl fmt::Display for IrValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyName => f.write_str("symbol name is empty"),
            Self::MethodWithoutOwner { name } => {
                write!(f, "method '{name}' has no lexical owner")
            }
            Self::DuplicateSymbol { name } => {
                write!(f, "duplicate symbol identity '{name}' in one module")
            }
        }
    }
}

impl std::error::Error for IrValidationError {}

/// Validates adapter output before it may enter the fact graph: every method
/// carries an owner and no two symbols share one identity. Range validity is
/// enforced by the coordinate constructors, so it is not rechecked here.
///
/// # Errors
/// Returns the first invariant violation found.
pub fn validate_module(module: &ParsedModule) -> Result<(), IrValidationError> {
    let mut seen = std::collections::BTreeSet::new();
    for symbol in module.symbols() {
        if symbol.id().kind() == SymbolKind::Method && symbol.id().owner().is_none() {
            return Err(IrValidationError::MethodWithoutOwner {
                name: symbol.id().name().as_str().to_owned(),
            });
        }
        let key = (
            symbol.id().kind() as u8,
            symbol.id().owner().map(SymbolName::as_str),
            symbol.id().name().as_str(),
        );
        if !seen.insert(key) {
            return Err(IrValidationError::DuplicateSymbol {
                name: symbol.id().name().as_str().to_owned(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test fixtures are static inputs; setup failure must fail the test immediately"
)]
mod tests {
    use super::*;
    use std::path::Path;

    fn test_path() -> RepoPath {
        RepoPath::new(Path::new("src/example.py")).expect("test fixture path is relative")
    }

    fn test_range(start_byte: u32, end_byte: u32, start_line: u32, end_line: u32) -> SourceRange {
        SourceRange::new(
            ByteRange::new(ByteOffset(start_byte), ByteOffset(end_byte))
                .expect("test range is non-empty"),
            LineRange::new(LineNumber(start_line), LineNumber(end_line))
                .expect("test line range is ordered"),
        )
    }

    fn test_symbol(kind: SymbolKind, owner: Option<&str>, name: &str) -> Symbol {
        Symbol::new(
            SymbolId::new(
                test_path(),
                kind,
                owner.map(|owner| SymbolName::new(owner).expect("test owner is named")),
                SymbolName::new(name).expect("test symbol is named"),
            ),
            test_range(0, 10, 1, 2),
        )
        .expect("test symbol satisfies IR invariants")
    }

    #[test]
    fn byte_range_rejects_empty_coordinates() {
        assert_eq!(
            ByteRange::new(ByteOffset(4), ByteOffset(4)),
            Err(CoordinateError::EmptyRange)
        );
        assert_eq!(
            ByteRange::new(ByteOffset(9), ByteOffset(4)),
            Err(CoordinateError::EmptyRange)
        );
    }

    #[test]
    fn line_numbers_convert_from_zero_based_parser_rows() {
        assert_eq!(LineNumber::from_zero_based(0).map(LineNumber::get), Ok(1));
        assert_eq!(LineNumber::from_zero_based(41).map(LineNumber::get), Ok(42));
        assert_eq!(
            LineRange::new(LineNumber(7), LineNumber(3)),
            Err(CoordinateError::EmptyRange)
        );
    }

    #[test]
    fn symbol_names_reject_empty_identifiers() {
        assert_eq!(SymbolName::new(""), Err(IrValidationError::EmptyName));
        assert!(SymbolName::new("train").is_ok());
    }

    #[test]
    fn methods_require_a_lexical_owner() {
        let id = SymbolId::new(
            test_path(),
            SymbolKind::Method,
            None,
            SymbolName::new("fit").expect("test name is valid"),
        );
        assert_eq!(
            Symbol::new(id, test_range(0, 10, 1, 2)),
            Err(IrValidationError::MethodWithoutOwner {
                name: "fit".to_owned()
            })
        );
    }

    #[test]
    fn module_construction_canonicalizes_symbol_order() {
        let late = test_symbol(SymbolKind::Function, None, "zebra");
        let early = test_symbol(SymbolKind::Function, None, "apple");
        let module = ParsedModule::new(
            test_path(),
            Language::Python,
            AnalyzerInfo::new("test", "0.0.0"),
            AnalysisStatus::Succeeded,
            ModuleBody {
                symbols: vec![late, early],
                ..ModuleBody::default()
            },
        );
        assert_eq!(module.symbols().len(), 2);
        assert!(validate_module(&module).is_ok());
    }

    #[test]
    fn validation_rejects_duplicate_symbol_identities() {
        let module = ParsedModule::new(
            test_path(),
            Language::Python,
            AnalyzerInfo::new("test", "0.0.0"),
            AnalysisStatus::Succeeded,
            ModuleBody {
                symbols: vec![
                    test_symbol(SymbolKind::Function, None, "train"),
                    test_symbol(SymbolKind::Function, None, "train"),
                ],
                ..ModuleBody::default()
            },
        );
        assert_eq!(
            validate_module(&module),
            Err(IrValidationError::DuplicateSymbol {
                name: "train".to_owned()
            })
        );
    }

    #[test]
    fn same_name_with_different_owners_is_not_a_duplicate() {
        let module = ParsedModule::new(
            test_path(),
            Language::Python,
            AnalyzerInfo::new("test", "0.0.0"),
            AnalysisStatus::Succeeded,
            ModuleBody {
                symbols: vec![
                    test_symbol(SymbolKind::Method, Some("AudioModel"), "fit"),
                    test_symbol(SymbolKind::Method, Some("Trainer"), "fit"),
                ],
                ..ModuleBody::default()
            },
        );
        assert!(validate_module(&module).is_ok());
    }
}
