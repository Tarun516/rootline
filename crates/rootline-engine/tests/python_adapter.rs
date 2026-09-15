//! Python adapter conformance tests: fixture files and inline sources
//! exercised through the public adapter boundary.

//! Fixture setup failure must fail the test immediately.
#![expect(
    clippy::expect_used,
    reason = "fixture setup failure must fail the test immediately"
)]

use rootline_core::RepoPath;
use rootline_core::ir::{
    AnalysisStatus, Language, ParsedModule, SourceRange, Symbol, SymbolKind, validate_module,
};
use rootline_engine::{ADAPTER_NAME, ADAPTER_VERSION, PythonAdapter, supports_extension};
use std::path::{Path, PathBuf};

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/python")
        .join(name)
}

fn parse_fixture(name: &str) -> ParsedModule {
    let source = std::fs::read_to_string(fixture_path(name)).expect("read Python fixture");
    parse_source(name, &source)
}

fn parse_source(name: &str, source: &str) -> ParsedModule {
    let repo_path = RepoPath::new(Path::new(name)).expect("fixture name is relative");
    PythonAdapter::new()
        .expect("adapter builds")
        .parse(&repo_path, source)
        .expect("fixture parses without adapter failure")
}

fn symbol_summary(symbol: &Symbol) -> (SymbolKind, Option<String>, &str) {
    (
        symbol.id().kind(),
        symbol.id().owner().map(|owner| owner.full_path()),
        symbol.id().name().as_str(),
    )
}

fn range_summary(symbol: &Symbol) -> (u32, u32, u32, u32) {
    range_of(symbol.range())
}

fn range_of(range: SourceRange) -> (u32, u32, u32, u32) {
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
    assert_eq!(
        module.file().to_canonical_string().expect("path is UTF-8"),
        "symbols_basic.py"
    );
    let summary: Vec<_> = module.symbols().iter().map(symbol_summary).collect();
    assert_eq!(
        summary,
        [
            (SymbolKind::Function, None, "train"),
            (SymbolKind::Class, None, "Model"),
            (SymbolKind::Method, Some("Model".to_owned()), "fit"),
            (SymbolKind::Method, Some("Model".to_owned()), "predict"),
            (SymbolKind::Function, None, "outer"),
            (SymbolKind::Function, Some("outer".to_owned()), "inner"),
            (SymbolKind::Function, None, "served"),
            (SymbolKind::Class, None, "Outer"),
            (SymbolKind::Class, Some("Outer".to_owned()), "Inner"),
            (SymbolKind::Method, Some("Outer.Inner".to_owned()), "method"),
            (SymbolKind::Function, None, "load"),
            (SymbolKind::Function, None, "load"),
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
            (386, 410, 34, 35), // load (first declaration)
            (413, 437, 38, 39), // load (second declaration)
        ]
    );
    // Byte ranges arrive in deterministic declaration order.
    let starts: Vec<u32> = ranges.iter().map(|range| range.0).collect();
    let mut ordered = starts.clone();
    ordered.sort();
    assert_eq!(starts, ordered);
}

#[test]
fn redefinitions_keep_distinct_identities() {
    let module = parse_fixture("symbols_basic.py");
    let loads: Vec<_> = module
        .symbols()
        .iter()
        .filter(|symbol| symbol.id().name().as_str() == "load")
        .collect();
    assert_eq!(loads.len(), 2);
    assert_eq!(loads[0].id().declaration_index(), 0);
    assert_eq!(loads[1].id().declaration_index(), 1);
    assert_ne!(loads[0].id(), loads[1].id());
    assert!(validate_module(&module).is_ok());
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
                    .map(|name| (name.imported().to_owned(), name.bound().as_str().to_owned()))
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
                vec![("os".to_owned(), "os".to_owned())],
                false,
                (47, 56, 2, 2)
            ),
            (
                0,
                Some("sys".to_owned()),
                vec![("sys".to_owned(), "system".to_owned())],
                false,
                (57, 77, 3, 3)
            ),
            (
                0,
                Some("pkg.sub".to_owned()),
                vec![("pkg.sub".to_owned(), "alias".to_owned())],
                false,
                (78, 101, 4, 4)
            ),
            (
                0,
                Some("pathlib".to_owned()),
                vec![("Path".to_owned(), "Path".to_owned())],
                false,
                (103, 127, 6, 6)
            ),
            (
                1,
                None,
                vec![("sibling".to_owned(), "sibling".to_owned())],
                false,
                (128, 149, 7, 7)
            ),
            (
                2,
                Some("pkg".to_owned()),
                vec![("thing".to_owned(), "renamed".to_owned())],
                false,
                (150, 184, 8, 8)
            ),
            (
                0,
                Some("package".to_owned()),
                vec![
                    ("first".to_owned(), "first".to_owned()),
                    ("second".to_owned(), "two".to_owned())
                ],
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

#[test]
fn extracts_call_shapes_with_callers() {
    use rootline_core::ir::CallReceiver;
    let source = "import os\n\
        \n\
        \n\
        class Base:\n\
        \x20   pass\n\
        \n\
        \n\
        class Child(Base, mix.Mixin, Generic[T], metaclass=Meta):\n\
        \x20   def run(self):\n\
        \x20       train()\n\
        \x20       self.run()\n\
        \x20       os.remove(item)\n\
        \x20       factory().make()\n\
        \n\
        \n\
        configure()\n";
    let module = parse_source("calls.py", source);
    let calls: Vec<_> = module
        .calls()
        .iter()
        .map(|call| {
            (
                call.caller()
                    .map(|caller| caller.name().as_str().to_owned()),
                match call.receiver() {
                    CallReceiver::Absent => "-".to_owned(),
                    CallReceiver::Named(name) => name.as_str().to_owned(),
                    CallReceiver::Opaque => "?".to_owned(),
                },
                call.name().as_str().to_owned(),
            )
        })
        .collect();
    assert_eq!(
        calls,
        [
            (Some("run".to_owned()), "-".to_owned(), "train".to_owned()),
            (Some("run".to_owned()), "self".to_owned(), "run".to_owned()),
            (Some("run".to_owned()), "os".to_owned(), "remove".to_owned()),
            // `factory().make()` yields both calls: the inner bare
            // `factory()` and the outer opaquely-qualified `make()`.
            (Some("run".to_owned()), "-".to_owned(), "factory".to_owned()),
            (Some("run".to_owned()), "?".to_owned(), "make".to_owned()),
            (None, "-".to_owned(), "configure".to_owned()),
        ]
    );
}

#[test]
fn extracts_plain_bases_and_skips_exotic_ones() {
    let source = "class Child(Base, mix.Mixin, Generic[T], metaclass=Meta):\n    pass\n";
    let module = parse_source("bases.py", source);
    let bases: Vec<_> = module
        .inheritances()
        .iter()
        .map(|inheritance| {
            (
                inheritance.class().name().as_str().to_owned(),
                inheritance.base().to_owned(),
            )
        })
        .collect();
    // `Generic[T]` (subscript) and `metaclass=Meta` (keyword) are
    // recorded nowhere: raw text would manufacture bogus externals.
    assert_eq!(
        bases,
        [
            ("Child".to_owned(), "Base".to_owned()),
            ("Child".to_owned(), "mix.Mixin".to_owned()),
        ]
    );
}
