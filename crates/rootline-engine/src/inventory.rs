use rootline_core::{Artifact, ArtifactCategory, RepoPath, RepoPathError};
use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// How candidate paths were listed before filesystem safety checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListingMode {
    GitIgnoreAware,
    FilesystemFallback,
}

impl fmt::Display for ListingMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GitIgnoreAware => f.write_str("git-ignore-aware"),
            Self::FilesystemFallback => {
                f.write_str("filesystem-fallback (ignore rules not applied)")
            }
        }
    }
}

/// Why a listed entry was not accepted as a regular-file artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScanIssueKind {
    Symlink,
    NotRegularFile,
    Vanished,
    Unreadable,
}

/// An entry skipped after listing; issues remain visible instead of becoming empty success.
#[derive(Debug, Eq, PartialEq)]
pub struct ScanIssue {
    pub path: RepoPath,
    pub kind: ScanIssueKind,
}

/// Snapshot of accepted files and skipped entries from one inventory pass.
///
/// `git_revision` identifies `HEAD` when available, not the contents of a dirty
/// worktree. `None` can mean the directory is not a Git repository or revision
/// lookup did not succeed.
#[derive(Debug)]
pub struct Inventory {
    pub root: PathBuf,
    pub mode: ListingMode,
    pub git_revision: Option<String>,
    pub artifacts: Vec<Artifact>,
    pub issues: Vec<ScanIssue>,
}

impl Inventory {
    pub fn total_bytes(&self) -> u64 {
        self.artifacts
            .iter()
            .map(|artifact| artifact.size_bytes)
            .sum()
    }
}

/// Operational failures that prevent the inventory pass from completing.
#[derive(Debug)]
pub enum ScanError {
    InvalidRoot {
        path: PathBuf,
        source: io::Error,
    },
    NotDirectory {
        path: PathBuf,
    },
    NotRepositoryRoot {
        path: PathBuf,
        repository_root: PathBuf,
    },
    GitFailed {
        operation: &'static str,
    },
    InvalidGitPath {
        source: RepoPathError,
    },
    PathEncoding,
    Io {
        path: PathBuf,
        source: io::Error,
    },
}

impl fmt::Display for ScanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRoot { path, source } => {
                write!(f, "cannot open {}: {source}", path.display())
            }
            Self::NotDirectory { path } => write!(f, "{} is not a directory", path.display()),
            Self::NotRepositoryRoot {
                path,
                repository_root,
            } => write!(
                f,
                "{} is inside a Git repository; index its root at {}",
                path.display(),
                repository_root.display()
            ),
            Self::GitFailed { operation } => write!(f, "Git {operation} failed"),
            Self::InvalidGitPath { source } => write!(f, "Git returned an unsafe path: {source}"),
            Self::PathEncoding => {
                f.write_str("Git returned a path that cannot be decoded on this platform")
            }
            Self::Io { path, source } => write!(f, "I/O at {} failed: {source}", path.display()),
        }
    }
}

impl std::error::Error for ScanError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidRoot { source, .. } | Self::Io { source, .. } => Some(source),
            Self::InvalidGitPath { source } => Some(source),
            _ => None,
        }
    }
}

/// Builds a safe, sorted inventory of regular files within a repository root.
/// Git repositories use Git's tracked + untracked, ignore-aware listing. Other
/// directories use a conservative walk that skips symlinks and `.git` but does
/// not claim to implement `.gitignore` semantics.
///
/// # Errors
/// Fails if the root cannot be opened, is not a directory, is a Git subdirectory,
/// Git listing is invalid, or filesystem traversal cannot complete.
pub fn scan(root: &Path) -> Result<Inventory, ScanError> {
    let root = fs::canonicalize(root).map_err(|source| ScanError::InvalidRoot {
        path: root.to_path_buf(),
        source,
    })?;
    if !root.is_dir() {
        return Err(ScanError::NotDirectory { path: root });
    }

    let git_root = discover_git_root(&root)?;
    let (mode, paths) = if let Some(repository_root) = git_root {
        if repository_root != root {
            return Err(ScanError::NotRepositoryRoot {
                path: root,
                repository_root,
            });
        }
        (ListingMode::GitIgnoreAware, git_paths(&root)?)
    } else {
        (ListingMode::FilesystemFallback, walk_paths(&root)?)
    };

    // Revision lookup is descriptive metadata; listing still succeeds without HEAD.
    let git_revision = if mode == ListingMode::GitIgnoreAware {
        git_revision(&root)
    } else {
        None
    };
    let mut artifacts = Vec::with_capacity(paths.len());
    let mut issues = Vec::new();
    for path in paths {
        match inspect(&root, &path) {
            Ok(Some(size_bytes)) => artifacts.push(Artifact {
                category: classify(path.as_path()),
                path,
                size_bytes,
            }),
            Ok(None) => issues.push(ScanIssue {
                path,
                kind: ScanIssueKind::Symlink,
            }),
            Err(kind) => issues.push(ScanIssue { path, kind }),
        }
    }
    artifacts.sort_by(|a, b| a.path.cmp(&b.path));
    issues.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(Inventory {
        root,
        mode,
        git_revision,
        artifacts,
        issues,
    })
}

fn discover_git_root(root: &Path) -> Result<Option<PathBuf>, ScanError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .map_err(|source| ScanError::Io {
            path: root.to_path_buf(),
            source,
        })?;
    // A nonzero `rev-parse` status selects fallback mode; the fallback reports
    // that Git ignore rules were not applied rather than claiming Git coverage.
    if !output.status.success() {
        return Ok(None);
    }
    let text = String::from_utf8(output.stdout).map_err(|_| ScanError::PathEncoding)?;
    let git_root = PathBuf::from(text.trim_end_matches(['\n', '\r']));
    let canonical = fs::canonicalize(&git_root).map_err(|source| ScanError::Io {
        path: git_root,
        source,
    })?;
    Ok(Some(canonical))
}

fn git_paths(root: &Path) -> Result<Vec<RepoPath>, ScanError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .output()
        .map_err(|source| ScanError::Io {
            path: root.to_path_buf(),
            source,
        })?;
    if !output.status.success() {
        return Err(ScanError::GitFailed {
            operation: "file listing",
        });
    }
    // NUL framing preserves filenames with whitespace or newlines; paths retain
    // their platform representation rather than becoming lossy display strings.
    let mut paths = Vec::new();
    for bytes in output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|bytes| !bytes.is_empty())
    {
        let os_path = decode_git_path(bytes)?;
        let path = RepoPath::new(Path::new(&os_path))
            .map_err(|source| ScanError::InvalidGitPath { source })?;
        paths.push(path);
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}

#[cfg(unix)]
fn decode_git_path(bytes: &[u8]) -> Result<OsString, ScanError> {
    use std::os::unix::ffi::OsStringExt;
    Ok(OsString::from_vec(bytes.to_vec()))
}

#[cfg(not(unix))]
fn decode_git_path(bytes: &[u8]) -> Result<OsString, ScanError> {
    String::from_utf8(bytes.to_vec())
        .map(OsString::from)
        .map_err(|_| ScanError::PathEncoding)
}

fn git_revision(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|text| text.trim().to_owned())
}

fn walk_paths(root: &Path) -> Result<Vec<RepoPath>, ScanError> {
    // Keep traversal iterative so deeply nested repositories do not consume
    // call-stack depth; inspect() later rejects symlinks and non-regular entries.
    let mut pending = vec![PathBuf::new()];
    let mut paths = Vec::new();
    while let Some(relative_dir) = pending.pop() {
        let absolute_dir = root.join(&relative_dir);
        let entries = fs::read_dir(&absolute_dir).map_err(|source| ScanError::Io {
            path: absolute_dir,
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| ScanError::Io {
                path: root.join(&relative_dir),
                source,
            })?;
            if entry.file_name() == OsStr::new(".git") {
                continue;
            }
            let relative = relative_dir.join(entry.file_name());
            let kind = entry.file_type().map_err(|source| ScanError::Io {
                path: root.join(&relative),
                source,
            })?;
            if kind.is_dir() {
                pending.push(relative);
            } else {
                let path = RepoPath::new(&relative)
                    .map_err(|source| ScanError::InvalidGitPath { source })?;
                paths.push(path);
            }
        }
    }
    paths.sort();
    Ok(paths)
}

// Check every component with symlink_metadata: a safe-looking final filename
// must not hide a symlinked parent directory that escapes the repository root.
fn inspect(root: &Path, path: &RepoPath) -> Result<Option<u64>, ScanIssueKind> {
    let mut current = root.to_path_buf();
    let components: Vec<_> = path.as_path().components().collect();
    for (index, component) in components.iter().enumerate() {
        current.push(component.as_os_str());
        let metadata = match fs::symlink_metadata(&current) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                return Err(ScanIssueKind::Vanished);
            }
            Err(_) => return Err(ScanIssueKind::Unreadable),
        };
        if metadata.file_type().is_symlink() {
            return Ok(None);
        }
        if index + 1 == components.len() {
            return if metadata.is_file() {
                Ok(Some(metadata.len()))
            } else {
                Err(ScanIssueKind::NotRegularFile)
            };
        }
        if !metadata.is_dir() {
            return Err(ScanIssueKind::NotRegularFile);
        }
    }
    Err(ScanIssueKind::NotRegularFile)
}

fn classify(path: &Path) -> ArtifactCategory {
    // Extension/name heuristics are only an inventory hint; language adapters
    // must establish actual support and source facts in later analysis slices.
    let extension = path
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("")
        .to_ascii_lowercase();
    let name = path.file_name().and_then(OsStr::to_str).unwrap_or("");
    if name == "Dockerfile" || name == "Makefile" || extension == "tf" {
        ArtifactCategory::Infrastructure
    } else if matches!(
        extension.as_str(),
        "py" | "pyi" | "ts" | "tsx" | "js" | "jsx" | "rs" | "go"
    ) {
        ArtifactCategory::Code
    } else if matches!(extension.as_str(), "md" | "mdx" | "rst" | "txt") {
        ArtifactCategory::Documentation
    } else if matches!(
        extension.as_str(),
        "jsonl" | "csv" | "sql" | "proto" | "graphql"
    ) {
        ArtifactCategory::Data
    } else if matches!(extension.as_str(), "toml" | "yaml" | "yml" | "json" | "ini")
        || name.starts_with('.')
    {
        ArtifactCategory::Config
    } else {
        ArtifactCategory::Other
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "isolated fixture setup and teardown should fail the unit test immediately"
)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

    fn test_dir() -> PathBuf {
        let id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("rootline-scan-test-{}-{id}", std::process::id()));
        fs::create_dir(&path).expect("create isolated test directory");
        path
    }

    #[test]
    fn fallback_lists_regular_files_in_stable_order() {
        let root = test_dir();
        fs::create_dir(root.join("src")).expect("create source directory");
        fs::write(root.join("z.txt"), b"z").expect("write fixture");
        fs::write(root.join("src/a.py"), b"print(1)\n").expect("write fixture");
        let inventory = scan(&root).expect("scan fixture");
        assert_eq!(inventory.mode, ListingMode::FilesystemFallback);
        assert_eq!(inventory.artifacts.len(), 2);
        assert_eq!(inventory.artifacts[0].path.as_path(), Path::new("src/a.py"));
        assert_eq!(inventory.total_bytes(), 10);
        fs::remove_dir_all(root).expect("remove isolated test directory");
    }

    #[cfg(unix)]
    #[test]
    fn fallback_does_not_follow_symlinks() {
        use std::os::unix::fs::symlink;
        let root = test_dir();
        symlink("/etc", root.join("outside")).expect("create fixture symlink");
        let inventory = scan(&root).expect("scan fixture");
        assert!(inventory.artifacts.is_empty());
        assert_eq!(inventory.issues.len(), 1);
        assert_eq!(inventory.issues[0].kind, ScanIssueKind::Symlink);
        fs::remove_dir_all(root).expect("remove isolated test directory");
    }

    #[test]
    fn git_listing_respects_ignore_rules() {
        let root = test_dir();
        let init = Command::new("git")
            .arg("-C")
            .arg(&root)
            .arg("init")
            .output()
            .expect("run fixture Git");
        assert!(init.status.success());
        fs::write(root.join(".gitignore"), b"ignored.txt\n").expect("write ignore rule");
        fs::write(root.join("included.py"), b"pass\n").expect("write fixture");
        fs::write(root.join("ignored.txt"), b"ignored\n").expect("write fixture");
        let inventory = scan(&root).expect("scan Git fixture");
        assert_eq!(inventory.mode, ListingMode::GitIgnoreAware);
        assert_eq!(inventory.artifacts.len(), 2);
        assert!(
            !inventory
                .artifacts
                .iter()
                .any(|artifact| artifact.path.as_path() == Path::new("ignored.txt"))
        );
        fs::remove_dir_all(root).expect("remove isolated test directory");
    }
}
