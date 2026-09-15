//! Python module resolution over an explicit file set.
//!
//! Resolution maps import syntax (adapter output) to repository artifacts
//! without touching the filesystem: the caller supplies the analyzed files
//! and the source roots, so tests and future pipeline stages control exactly
//! what "the repository" means. Filesystem reads, root discovery, and
//! cancellation stay outside this boundary.
//!
//! Outcome contract, kept distinct from operational failure:
//!
//! - `Resolved`: exactly one analyzed file provides the module;
//! - `Ambiguous`: several roots provide it; every candidate is listed, and no
//!   silent shadowing picks a winner;
//! - `External`: no analyzed file provides an absolute module (stdlib,
//!   third-party, or absent — recorded, never errored);
//! - `Unknown`: a relative import names no analyzed target, escapes its
//!   package, or is otherwise malformed.
//!
//! Deliberate limits of this revision:
//!
//! - Any directory may act as a package (namespace-package semantics).
//!   `__init__.py` presence is not required to resolve, which mirrors modern
//!   packaging but can over-accept legacy layouts; over-acceptance surfaces
//!   as resolved edges, never as silent drops.
//! - Absolute imports consult only the analyzed file set, so a partially
//!   analyzed repository may classify a missing-internal module as external.
//!   The classification stays visible in graph diagnostics for exactly this
//!   reason; explicit repository-boundary tracking is the follow-up.
//! - Relative imports are repo-internal by construction: `from . import x`
//!   can never resolve to `External`.

use std::collections::BTreeSet;
use std::fmt;
use std::path::{Component, PathBuf};

use rootline_core::RepoPath;

/// What module resolution established for one import statement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Resolution {
    Resolved { target: RepoPath },
    Ambiguous { candidates: Vec<RepoPath> },
    External { module: String },
    Unknown { reason: String },
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolved { target } => {
                write!(f, "resolved to {}", target.as_path().display())
            }
            Self::Ambiguous { candidates } => {
                write!(f, "ambiguous across {} candidates", candidates.len())
            }
            Self::External { module } => write!(f, "external module {module}"),
            Self::Unknown { reason } => write!(f, "unknown: {reason}"),
        }
    }
}

/// The file set resolution may target, plus the source roots absolute
/// imports are anchored at. Both are owned here; nothing is read from disk.
#[derive(Clone, Debug)]
pub struct ModuleIndex {
    roots: Vec<RepoPath>,
    files: BTreeSet<RepoPath>,
}

impl ModuleIndex {
    /// Builds an index over analyzed files and source roots (repository
    /// relative, e.g. `src` for a src-layout project).
    ///
    /// Absolute imports probe `<root>/a/b.py` and `<root>/a/b/__init__.py`
    /// under every root, plus the repository top itself: a flat project with
    /// no source roots still resolves its top-level modules, and a src-layout
    /// project keeps working when tooling also exposes the repository top.
    pub fn new(roots: Vec<RepoPath>, files: Vec<RepoPath>) -> Self {
        Self {
            roots,
            files: files.into_iter().collect(),
        }
    }

    pub fn files(&self) -> &BTreeSet<RepoPath> {
        &self.files
    }

    /// Resolves an absolute module path such as `os.path` or `pkg.sub`.
    pub fn resolve_absolute(&self, module: &str) -> Resolution {
        let segments: Vec<&str> = module
            .split('.')
            .filter(|segment| !segment.is_empty())
            .collect();
        if segments.is_empty() {
            return Resolution::Unknown {
                reason: "absolute import names an empty module path".to_owned(),
            };
        }
        let mut candidates: Vec<RepoPath> =
            self.absolute_candidates(&segments).into_iter().collect();
        if candidates.len() > 1 {
            return Resolution::Ambiguous { candidates };
        }
        if let Some(target) = candidates.pop() {
            return Resolution::Resolved { target };
        }
        Resolution::External {
            module: module.to_owned(),
        }
    }

    /// Candidate files for absolute module segments: `<base>/<segments>.py`
    /// and `<base>/<segments>/__init__.py` for the repository top plus every
    /// configured root, restricted to analyzed files. Collection order is
    /// normalized so ambiguity reports are deterministic.
    fn absolute_candidates(&self, segments: &[&str]) -> BTreeSet<RepoPath> {
        let mut candidates = BTreeSet::new();
        let mut bases: Vec<std::path::PathBuf> = vec![PathBuf::new()];
        bases.extend(self.roots.iter().map(|root| root.as_path().to_path_buf()));
        for base in &bases {
            for candidate in module_files(base, segments) {
                if self.files.contains(&candidate) {
                    candidates.insert(candidate);
                }
            }
        }
        candidates
    }

    /// Resolves a `from` import: `level` leading dots and the optional module
    /// path, interpreted from the importing file's directory. `level` 1
    /// means the importer's own directory; each further level climbs one
    /// package. Callers route `level` 0 (absolute) to [`Self::resolve_absolute`].
    pub fn resolve_relative(
        &self,
        importer: &RepoPath,
        level: u32,
        module: Option<&str>,
    ) -> Resolution {
        let mut components = path_components(importer.as_path());
        components.pop();
        for _ in 1..level {
            if components.pop().is_none() {
                return Resolution::Unknown {
                    reason: "relative import climbs above its package".to_owned(),
                };
            }
        }
        if let Some(module) = module {
            let segments: Vec<&str> = module
                .split('.')
                .filter(|segment| !segment.is_empty())
                .collect();
            if segments.is_empty() {
                return Resolution::Unknown {
                    reason: "relative import names an empty module path".to_owned(),
                };
            }
            components.extend(segments.iter().map(ToString::to_string));
        }
        let candidates = join_candidates(&components);
        let mut hits: Vec<RepoPath> = candidates
            .into_iter()
            .filter(|candidate| self.files.contains(candidate))
            .collect();
        hits.sort();
        hits.dedup();
        match hits.len() {
            0 => Resolution::Unknown {
                reason: "no analyzed file provides the relative target".to_owned(),
            },
            1 => Resolution::Resolved {
                target: hits.remove(0),
            },
            _ => Resolution::Ambiguous { candidates: hits },
        }
    }

    /// Probes whether `name` is a submodule file next to an already-resolved
    /// package file: for `from pkg import name`, after `pkg` resolves to a
    /// file `F`, this checks `parent(F)/name.py` and
    /// `parent(F)/name/__init__.py`. Names defined *inside* `F` are symbol
    /// references, not submodules, and stay the graph builder's job — this
    /// answers only the file question.
    pub fn resolve_submodule(&self, package_file: &RepoPath, name: &str) -> Option<RepoPath> {
        if name.is_empty() {
            return None;
        }
        let mut components = path_components(package_file.as_path());
        components.pop();
        components.push(name.to_owned());
        join_candidates(&components)
            .into_iter()
            .find(|candidate| self.files.contains(candidate))
    }
}

/// Candidate files for module segments under one base directory:
/// `<base>/<segments>.py` and `<base>/<segments>/__init__.py`. The base is
/// empty for the repository top.
fn module_files(base: &std::path::Path, segments: &[&str]) -> Vec<RepoPath> {
    let mut paths = Vec::with_capacity(2);
    let mut module = base.to_path_buf();
    for segment in segments {
        module.push(segment);
    }
    let mut file = module.clone();
    file.set_extension("py");
    if let Ok(path) = RepoPath::new(&file) {
        paths.push(path);
    }
    module.push("__init__.py");
    if let Ok(path) = RepoPath::new(&module) {
        paths.push(path);
    }
    paths
}

/// Candidate files for already-joined relative components: the components
/// as a module file, or as a package's `__init__.py`. An empty component
/// list means the import climbed past the repository top.
fn join_candidates(components: &[String]) -> Vec<RepoPath> {
    if components.is_empty() {
        return Vec::new();
    }
    let mut paths = Vec::with_capacity(2);
    let mut base = PathBuf::new();
    for component in components {
        base.push(component);
    }
    let mut file = base.clone();
    file.set_extension("py");
    if let Ok(path) = RepoPath::new(&file) {
        paths.push(path);
    }
    base.push("__init__.py");
    if let Ok(path) = RepoPath::new(&base) {
        paths.push(path);
    }
    paths
}

/// Normal path components of a validated repository path.
fn path_components(path: &std::path::Path) -> Vec<String> {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(part) => part.to_str().map(str::to_owned),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "fixture setup failure must fail the test immediately"
)]
mod tests {
    use super::*;
    use std::path::Path;

    fn repo_path(name: &str) -> RepoPath {
        RepoPath::new(Path::new(name)).expect("test path is relative")
    }

    fn index() -> ModuleIndex {
        ModuleIndex::new(
            vec![repo_path("src")],
            vec![
                repo_path("src/shop/__init__.py"),
                repo_path("src/shop/cart.py"),
                repo_path("src/shop/store/__init__.py"),
                repo_path("src/shop/store/shelf.py"),
                repo_path("src/top.py"),
            ],
        )
    }

    #[test]
    fn resolves_absolute_modules_and_packages() {
        let index = index();
        assert_eq!(
            index.resolve_absolute("shop.cart"),
            Resolution::Resolved {
                target: repo_path("src/shop/cart.py")
            }
        );
        assert_eq!(
            index.resolve_absolute("shop"),
            Resolution::Resolved {
                target: repo_path("src/shop/__init__.py")
            }
        );
        assert_eq!(
            index.resolve_absolute("shop.store.shelf"),
            Resolution::Resolved {
                target: repo_path("src/shop/store/shelf.py")
            }
        );
    }

    #[test]
    fn unmapped_absolute_modules_are_external_not_errors() {
        let index = index();
        assert_eq!(
            index.resolve_absolute("os.path"),
            Resolution::External {
                module: "os.path".to_owned()
            }
        );
        assert_eq!(
            index.resolve_absolute("thirdparty.lib"),
            Resolution::External {
                module: "thirdparty.lib".to_owned()
            }
        );
    }

    #[test]
    fn duplicate_modules_across_roots_are_ambiguous() {
        let index = ModuleIndex::new(
            vec![repo_path("src"), repo_path("vendor")],
            vec![
                repo_path("src/shop/cart.py"),
                repo_path("vendor/shop/cart.py"),
            ],
        );
        assert_eq!(
            index.resolve_absolute("shop.cart"),
            Resolution::Ambiguous {
                candidates: vec![
                    repo_path("src/shop/cart.py"),
                    repo_path("vendor/shop/cart.py"),
                ]
            }
        );
    }

    #[test]
    fn resolves_relative_imports_from_the_importer_package() {
        let index = index();
        let importer = repo_path("src/shop/cart.py");
        assert_eq!(
            index.resolve_relative(&importer, 1, Some("store.shelf")),
            Resolution::Resolved {
                target: repo_path("src/shop/store/shelf.py")
            }
        );
        assert_eq!(
            index.resolve_relative(&importer, 1, None),
            Resolution::Resolved {
                target: repo_path("src/shop/__init__.py")
            }
        );
        assert_eq!(
            index.resolve_relative(&importer, 2, Some("top")),
            Resolution::Resolved {
                target: repo_path("src/top.py")
            }
        );
    }

    #[test]
    fn relative_imports_climbing_past_the_top_are_unknown() {
        let index = index();
        assert_eq!(
            index.resolve_relative(&repo_path("src/top.py"), 3, Some("x")),
            Resolution::Unknown {
                reason: "relative import climbs above its package".to_owned()
            }
        );
    }

    #[test]
    fn relative_targets_absent_from_the_file_set_are_unknown() {
        let index = index();
        assert_eq!(
            index.resolve_relative(&repo_path("src/shop/cart.py"), 1, Some("ghost")),
            Resolution::Unknown {
                reason: "no analyzed file provides the relative target".to_owned()
            }
        );
    }

    #[test]
    fn submodule_names_resolve_next_to_their_package() {
        let index = index();
        assert_eq!(
            index.resolve_submodule(&repo_path("src/shop/__init__.py"), "cart"),
            Some(repo_path("src/shop/cart.py"))
        );
        assert_eq!(
            index.resolve_submodule(&repo_path("src/shop/__init__.py"), "store"),
            Some(repo_path("src/shop/store/__init__.py"))
        );
        assert_eq!(
            index.resolve_submodule(&repo_path("src/shop/__init__.py"), "ghost"),
            None
        );
        assert_eq!(
            index.resolve_submodule(&repo_path("src/shop/__init__.py"), ""),
            None
        );
    }

    #[test]
    fn empty_module_paths_are_unknown() {
        let index = index();
        assert!(matches!(
            index.resolve_absolute("..."),
            Resolution::Unknown { .. }
        ));
    }

    #[test]
    fn repository_top_resolves_without_configured_roots() {
        // Flat layouts have no source root to name; the repository top is
        // always probed so `import top` still resolves internally.
        let index = ModuleIndex::new(Vec::new(), vec![repo_path("top.py")]);
        assert_eq!(
            index.resolve_absolute("top"),
            Resolution::Resolved {
                target: repo_path("top.py")
            }
        );
        assert_eq!(
            index.resolve_absolute("missing"),
            Resolution::External {
                module: "missing".to_owned()
            }
        );
    }
}
