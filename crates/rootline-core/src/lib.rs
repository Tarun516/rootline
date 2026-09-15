use std::fmt;
use std::path::{Component, Path, PathBuf};

pub mod graph;
pub mod ir;

/// A lexical repository-relative path containing only normal components.
///
/// This establishes a path-shape invariant, not a filesystem-boundary guarantee:
/// the scanner checks each component for symlinks before accepting an artifact.
/// Cloning and hashing are supported because paths serve as symbol and graph
/// identity components.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RepoPath(PathBuf);

/// Why a repository-relative path could not establish its lexical invariant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RepoPathError {
    Empty,
    NotRelative,
    InvalidComponent,
    NonUtf8Component,
}

impl fmt::Display for RepoPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("repository path is empty"),
            Self::NotRelative => f.write_str("repository path must be relative"),
            Self::InvalidComponent => {
                f.write_str("repository path contains a non-normal component")
            }
            Self::NonUtf8Component => f.write_str("repository path component is not valid UTF-8"),
        }
    }
}

impl std::error::Error for RepoPathError {}

impl RepoPath {
    /// Constructs a path containing only normal relative components.
    ///
    /// # Errors
    /// Returns an error for empty, absolute, parent, current-directory, or prefix components.
    pub fn new(path: &Path) -> Result<Self, RepoPathError> {
        if path.as_os_str().is_empty() {
            return Err(RepoPathError::Empty);
        }
        if path.is_absolute() {
            return Err(RepoPathError::NotRelative);
        }
        if path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
        {
            Ok(Self(path.to_path_buf()))
        } else {
            Err(RepoPathError::InvalidComponent)
        }
    }

    /// Borrows the original platform path without lossy text conversion.
    pub fn as_path(&self) -> &Path {
        &self.0
    }

    /// Canonical `/`-separated representation for persisted, protocol, and
    /// test-comparison use. Native separators differ per platform, so neither
    /// `display()` nor `as_path()` may serve as the canonical form: tests
    /// comparing those break on Windows, and persistence would fork by OS.
    ///
    /// Components are joined, never byte-substituted, so a backslash inside
    /// a Unix filename cannot corrupt into a separator.
    ///
    /// # Errors
    /// Returns [`RepoPathError::NonUtf8Component`] when a component is not
    /// valid UTF-8. Lossy conversion would silently merge distinct paths, so
    /// it is refused explicitly instead.
    pub fn to_canonical_string(&self) -> Result<String, RepoPathError> {
        let mut parts = Vec::new();
        for component in self.0.components() {
            match component {
                Component::Normal(part) => {
                    let text = part.to_str().ok_or(RepoPathError::NonUtf8Component)?;
                    parts.push(text);
                }
                _ => return Err(RepoPathError::InvalidComponent),
            }
        }
        Ok(parts.join("/"))
    }
}

/// Preliminary file classification for inventory output, not a parser support claim.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactCategory {
    Code,
    Config,
    Documentation,
    Data,
    Infrastructure,
    Other,
}

impl fmt::Display for ArtifactCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Code => "code",
            Self::Config => "config",
            Self::Documentation => "documentation",
            Self::Data => "data",
            Self::Infrastructure => "infrastructure",
            Self::Other => "other",
        };
        f.write_str(label)
    }
}

/// A regular file accepted by the inventory scanner.
#[derive(Debug, Eq, PartialEq)]
pub struct Artifact {
    pub path: RepoPath,
    pub size_bytes: u64,
    pub category: ArtifactCategory,
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test fixtures are static inputs; setup failure must fail the test immediately"
)]
mod tests {
    use super::*;

    #[test]
    fn path_rejects_escape_and_absolute_input() {
        assert!(RepoPath::new(Path::new("src/main.py")).is_ok());
        assert_eq!(
            RepoPath::new(Path::new("../secret")),
            Err(RepoPathError::InvalidComponent)
        );
        #[cfg(unix)]
        assert_eq!(
            RepoPath::new(Path::new("/tmp/secret")),
            Err(RepoPathError::NotRelative)
        );
        #[cfg(windows)]
        {
            // A Windows absolute path needs both a drive prefix and a root.
            assert_eq!(
                RepoPath::new(Path::new(r"C:\secret")),
                Err(RepoPathError::NotRelative)
            );
            // Rooted-but-drive-relative paths are unsafe, but not absolute.
            assert_eq!(
                RepoPath::new(Path::new(r"\secret")),
                Err(RepoPathError::InvalidComponent)
            );
        }
        assert_eq!(RepoPath::new(Path::new("")), Err(RepoPathError::Empty));
    }

    #[test]
    fn canonical_form_is_platform_independent() {
        // Joining native components with `/` keeps this assertion green on
        // Windows, where `display()` would render backslashes.
        let path = RepoPath::new(Path::new("fixtures/python/graph/shop/cart.py"))
            .expect("fixture path is relative");
        assert_eq!(
            path.to_canonical_string().expect("path is UTF-8"),
            "fixtures/python/graph/shop/cart.py"
        );
    }

    #[cfg(unix)]
    #[test]
    fn canonical_form_refuses_non_utf8_components() {
        use std::os::unix::ffi::OsStringExt;
        let raw = std::ffi::OsString::from_vec(b"src/\xff.py".to_vec());
        let path = RepoPath::new(Path::new(&raw)).expect("bytes are a normal component");
        assert_eq!(
            path.to_canonical_string(),
            Err(RepoPathError::NonUtf8Component)
        );
    }
}
