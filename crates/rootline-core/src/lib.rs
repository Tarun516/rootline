use std::fmt;
use std::path::{Component, Path, PathBuf};

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
}

impl fmt::Display for RepoPathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => f.write_str("repository path is empty"),
            Self::NotRelative => f.write_str("repository path must be relative"),
            Self::InvalidComponent => {
                f.write_str("repository path contains a non-normal component")
            }
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
}
