//! Symbol-graph integration tests: fixture packages parsed, resolved,
//! and built into validated graphs through the public engine boundary.

//! Fixture setup failure must fail the test immediately.
#![expect(
    clippy::expect_used,
    reason = "fixture setup failure must fail the test immediately"
)]

use rootline_core::RepoPath;
use rootline_core::graph::{GraphMetadata, NodeId, Relation, RelationKind, RelationTarget};
use rootline_core::ir::ParsedModule;
use rootline_engine::{ADAPTER_NAME, ModuleIndex, PythonAdapter, ResolvedGraph, build};
use std::path::{Path, PathBuf};

const GRAPH_ROOT: &str = "fixtures/python/graph";

fn fixture_file(name: &str) -> RepoPath {
    RepoPath::new(&Path::new(GRAPH_ROOT).join(name)).expect("fixture path is relative")
}

fn parse_report(path: &RepoPath, root: &Path) -> ParsedModule {
    let absolute = root.join(path.as_path());
    let source = std::fs::read_to_string(&absolute)
        .unwrap_or_else(|_| panic!("read fixture {}", absolute.display()));
    PythonAdapter::new()
        .expect("adapter builds")
        .parse(path, &source)
        .expect("fixture parses without adapter failure")
}

fn test_metadata() -> GraphMetadata {
    GraphMetadata::new(None, None)
}

fn fixture_graph() -> ResolvedGraph {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let names = [
        "__init__.py",
        "helper.py",
        "main.py",
        "shop/__init__.py",
        "shop/cart.py",
        "shop/store/__init__.py",
        "shop/store/shelf.py",
    ];
    let modules: Vec<ParsedModule> = names
        .iter()
        .map(|name| parse_report(&fixture_file(name), &root))
        .collect();
    let files: Vec<RepoPath> = names.iter().map(|name| fixture_file(name)).collect();
    let index = ModuleIndex::new(
        vec![RepoPath::new(Path::new(GRAPH_ROOT)).expect("root")],
        files,
    );
    build(&modules, &index, test_metadata()).expect("fixture graph validates")
}

fn edge_summary(relation: &Relation) -> (String, String, String, String) {
    let target = match relation.target() {
        RelationTarget::Resolved(node) => {
            node.to_canonical_string().expect("fixture paths are UTF-8")
        }
        RelationTarget::Ambiguous(candidates) => format!(
            "ambiguous[{}]",
            candidates
                .iter()
                .map(|node| node.to_canonical_string().expect("fixture paths are UTF-8"))
                .collect::<Vec<_>>()
                .join("|")
        ),
    };
    (
        relation
            .from()
            .to_canonical_string()
            .expect("fixture paths are UTF-8"),
        relation.kind().to_string(),
        target,
        relation.confidence().to_string(),
    )
}

#[test]
fn builds_containment_for_files_and_owners() {
    let resolved = fixture_graph();
    let edges: Vec<_> = resolved
        .graph()
        .relations()
        .iter()
        .filter(|relation| relation.kind() == RelationKind::Contains)
        .map(edge_summary)
        .collect();
    let root = GRAPH_ROOT;
    assert_eq!(
        edges,
        [
            (
                format!("file:{root}/helper.py"),
                "contains".to_owned(),
                "function:assist".to_owned(),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/main.py"),
                "contains".to_owned(),
                "function:run".to_owned(),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/cart.py"),
                "contains".to_owned(),
                "class:Cart".to_owned(),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/store/shelf.py"),
                "contains".to_owned(),
                "function:locate".to_owned(),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/store/shelf.py"),
                "contains".to_owned(),
                "class:Base".to_owned(),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/store/shelf.py"),
                "contains".to_owned(),
                "class:Broken".to_owned(),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/store/shelf.py"),
                "contains".to_owned(),
                "class:Posix".to_owned(),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/store/shelf.py"),
                "contains".to_owned(),
                "class:Shelf".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "class:Cart".to_owned(),
                "contains".to_owned(),
                "method:Cart.__init__".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "class:Cart".to_owned(),
                "contains".to_owned(),
                "method:Cart.add".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "class:Cart".to_owned(),
                "contains".to_owned(),
                "method:Cart.checkout".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "class:Cart".to_owned(),
                "contains".to_owned(),
                "method:Cart.total".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "class:Base".to_owned(),
                "contains".to_owned(),
                "method:Base.move".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "class:Shelf".to_owned(),
                "contains".to_owned(),
                "method:Shelf.place".to_owned(),
                "deterministic".to_owned()
            ),
        ]
    );
}

#[test]
fn resolves_import_edges_and_externals() {
    let resolved = fixture_graph();
    let edges: Vec<_> = resolved
        .graph()
        .relations()
        .iter()
        .filter(|relation| relation.kind() == RelationKind::Imports)
        .map(edge_summary)
        .collect();
    let root = GRAPH_ROOT;
    assert_eq!(
        edges,
        [
            (
                format!("file:{root}/main.py"),
                "imports".to_owned(),
                format!("file:{root}/shop/cart.py"),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/main.py"),
                "imports".to_owned(),
                format!("file:{root}/shop/store/shelf.py"),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/main.py"),
                "imports".to_owned(),
                "external:pathlib".to_owned(),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/__init__.py"),
                "imports".to_owned(),
                format!("file:{root}/shop/cart.py"),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/__init__.py"),
                "imports".to_owned(),
                format!("file:{root}/shop/store/__init__.py"),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/__init__.py"),
                "imports".to_owned(),
                format!("file:{root}/shop/store/shelf.py"),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/cart.py"),
                "imports".to_owned(),
                format!("file:{root}/__init__.py"),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/cart.py"),
                "imports".to_owned(),
                format!("file:{root}/helper.py"),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/cart.py"),
                "imports".to_owned(),
                format!("file:{root}/shop/store/shelf.py"),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/cart.py"),
                "imports".to_owned(),
                "external:os".to_owned(),
                "deterministic".to_owned()
            ),
            (
                format!("file:{root}/shop/store/shelf.py"),
                "imports".to_owned(),
                "external:os".to_owned(),
                "deterministic".to_owned()
            ),
        ]
    );
}

#[test]
fn resolves_calls_conservatively() {
    let resolved = fixture_graph();
    let edges: Vec<_> = resolved
        .graph()
        .relations()
        .iter()
        .filter(|relation| relation.kind() == RelationKind::Calls)
        .map(edge_summary)
        .collect();
    assert_eq!(
        edges,
        [
            (
                "function:assist".to_owned(),
                "calls".to_owned(),
                "function:assist".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "function:run".to_owned(),
                "calls".to_owned(),
                "class:Cart".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "function:run".to_owned(),
                "calls".to_owned(),
                "class:Shelf".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "function:run".to_owned(),
                "calls".to_owned(),
                "external:pathlib".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "method:Cart.add".to_owned(),
                "calls".to_owned(),
                "function:locate".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "method:Cart.checkout".to_owned(),
                "calls".to_owned(),
                "function:assist".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "method:Cart.checkout".to_owned(),
                "calls".to_owned(),
                "method:Cart.total".to_owned(),
                "deterministic".to_owned()
            ),
        ]
    );
}

#[test]
fn resolves_inheritance_with_evidence() {
    let resolved = fixture_graph();
    let edges: Vec<_> = resolved
        .graph()
        .relations()
        .iter()
        .filter(|relation| relation.kind() == RelationKind::Inherits)
        .map(edge_summary)
        .collect();
    assert_eq!(
        edges,
        [
            (
                "class:Broken".to_owned(),
                "inherits".to_owned(),
                "class:Shelf".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "class:Posix".to_owned(),
                "inherits".to_owned(),
                "external:os".to_owned(),
                "deterministic".to_owned()
            ),
            (
                "class:Shelf".to_owned(),
                "inherits".to_owned(),
                "class:Base".to_owned(),
                "deterministic".to_owned()
            ),
        ]
    );
    for relation in resolved
        .graph()
        .relations()
        .iter()
        .filter(|relation| relation.kind() == RelationKind::Inherits)
    {
        assert!(!relation.evidence().is_empty());
    }
}

#[test]
fn unproven_facts_stay_visible_diagnostics() {
    let resolved = fixture_graph();
    let diagnostics: Vec<_> = resolved
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.kind().to_string(),
                diagnostic
                    .file()
                    .to_canonical_string()
                    .expect("fixture paths are UTF-8"),
                diagnostic.subject().to_owned(),
                diagnostic.reason().to_owned(),
            )
        })
        .collect();
    let root = GRAPH_ROOT;
    assert_eq!(
        diagnostics,
        [
            (
                "import".to_owned(),
                format!("{root}/main.py"),
                "from .ghost import missing".to_owned(),
                "no analyzed file provides the relative target".to_owned()
            ),
            (
                "call".to_owned(),
                format!("{root}/main.py"),
                "unknown_thing()".to_owned(),
                "'unknown_thing' has no in-scope definition or import binding".to_owned()
            ),
            (
                "import".to_owned(),
                format!("{root}/shop/cart.py"),
                "from .store.shelf import move, locate".to_owned(),
                format!(
                    "name 'move' is neither defined in '{root}/shop/store/shelf.py' nor a submodule of it"
                )
            ),
            (
                "call".to_owned(),
                format!("{root}/shop/cart.py"),
                "?.append()".to_owned(),
                "complex receiver expression; call target unproven".to_owned()
            ),
            (
                "call".to_owned(),
                format!("{root}/shop/store/shelf.py"),
                "?.join()".to_owned(),
                "complex receiver expression; call target unproven".to_owned()
            ),
            (
                "call".to_owned(),
                format!("{root}/shop/store/shelf.py"),
                "self.move()".to_owned(),
                "'move' is not a method of 'Shelf'".to_owned()
            ),
            (
                "base-class".to_owned(),
                format!("{root}/shop/store/shelf.py"),
                "Missing".to_owned(),
                "base class 'Missing' resolves to no analyzed class".to_owned()
            ),
        ]
    );
}

#[test]
fn nodes_carry_ranges_and_provenance() {
    let resolved = fixture_graph();
    let graph = resolved.graph();
    let cart = NodeId::Artifact(fixture_file("shop/cart.py"));
    let cart_class = graph
        .nodes()
        .keys()
        .find(|id| matches!(id, NodeId::Symbol(symbol) if symbol.name().as_str() == "Cart"))
        .expect("Cart node exists")
        .clone();
    let cart_info = graph.node_info(&cart).expect("artifact node exists");
    assert_eq!(cart_info.range(), None);
    let class_info = graph.node_info(&cart_class).expect("symbol node exists");
    assert!(class_info.range().is_some());
    assert_eq!(class_info.analyzer().name(), ADAPTER_NAME);
    assert_eq!(graph.metadata().revision(), None);
}

#[test]
fn published_graph_has_no_dangling_endpoints() {
    let resolved = fixture_graph();
    assert!(resolved.graph().validate().is_ok());
    // 7 artifacts + 14 symbols + 2 externals (os, pathlib).
    assert_eq!(resolved.graph().nodes().len(), 23);
    assert_eq!(resolved.graph().relations().len(), 35);
}

#[test]
fn ambiguous_modules_produce_candidates_not_guesses() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let main_path = RepoPath::new(Path::new("entry.py")).expect("test path");
    let source = "import dup.mod\n";
    let main = PythonAdapter::new()
        .expect("adapter builds")
        .parse(&main_path, source)
        .expect("inline source parses");
    let index = ModuleIndex::new(
        vec![
            RepoPath::new(Path::new("libs/a")).expect("root"),
            RepoPath::new(Path::new("libs/b")).expect("root"),
        ],
        vec![
            main_path.clone(),
            RepoPath::new(Path::new("libs/a/dup/mod.py")).expect("test path"),
            RepoPath::new(Path::new("libs/b/dup/mod.py")).expect("test path"),
        ],
    );
    let _ = root;
    let resolved =
        build(std::slice::from_ref(&main), &index, test_metadata()).expect("graph validates");
    let edges: Vec<_> = resolved
        .graph()
        .relations()
        .iter()
        .map(edge_summary)
        .collect();
    assert_eq!(
        edges,
        [(
            "file:entry.py".to_owned(),
            "imports".to_owned(),
            "ambiguous[file:libs/a/dup/mod.py|file:libs/b/dup/mod.py]".to_owned(),
            "deterministic".to_owned()
        )]
    );
    let relation = &resolved.graph().relations()[0];
    assert!(matches!(
        relation.target(),
        RelationTarget::Ambiguous(candidates) if candidates.len() == 2
    ));
}

#[test]
fn aliased_imports_resolve_through_the_imported_name() {
    // `from provider import thing as renamed` must resolve `thing` in
    // the provider, not `renamed`: the bound name exists only locally.
    let provider_path = RepoPath::new(Path::new("provider.py")).expect("test path");
    let importer_path = RepoPath::new(Path::new("user.py")).expect("test path");
    let mut adapter = PythonAdapter::new().expect("adapter builds");
    let provider = adapter
        .parse(&provider_path, "def thing():\n    return 1\n")
        .expect("provider parses");
    let importer = adapter
        .parse(
            &importer_path,
            "from provider import thing as renamed\n\n\nrenamed()\n",
        )
        .expect("importer parses");
    let index = ModuleIndex::new(
        Vec::new(),
        vec![provider_path.clone(), importer_path.clone()],
    );
    let resolved = build(&[provider, importer], &index, test_metadata()).expect("graph validates");
    let calls: Vec<_> = resolved
        .graph()
        .relations()
        .iter()
        .filter(|relation| relation.kind() == RelationKind::Calls)
        .map(edge_summary)
        .collect();
    assert_eq!(
        calls,
        [(
            "file:user.py".to_owned(),
            "calls".to_owned(),
            "function:thing".to_owned(),
            "deterministic".to_owned()
        )]
    );
    assert!(resolved.diagnostics().is_empty());
}

#[test]
fn build_is_independent_of_input_order() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let names = [
        "main.py",
        "shop/cart.py",
        "helper.py",
        "shop/__init__.py",
        "shop/store/shelf.py",
        "shop/store/__init__.py",
        "__init__.py",
    ];
    let modules: Vec<ParsedModule> = names
        .iter()
        .map(|name| parse_report(&fixture_file(name), &root))
        .collect();
    let files: Vec<RepoPath> = names.iter().map(|name| fixture_file(name)).collect();
    let index = ModuleIndex::new(
        vec![RepoPath::new(Path::new(GRAPH_ROOT)).expect("root")],
        files,
    );
    let _ = PathBuf::new();
    assert_eq!(
        build(&modules, &index, test_metadata()).expect("graph validates"),
        fixture_graph()
    );
}
