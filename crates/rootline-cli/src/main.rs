use rootline_core::RepoPath;
use rootline_core::ir::AnalysisStatus;
use rootline_engine::{ListingMode, PythonAdapter, scan, supports_extension};
use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

// The CLI owns user-facing output and exit codes; the engine returns typed
// results so other callers do not inherit presentation or process policy.
fn main() -> ExitCode {
    let mut args = env::args_os();
    let _program = args.next();
    let command = args.next();
    let path = args.next();
    if path.is_none() || args.next().is_some() {
        eprintln!("usage: rootline index <repository-root>");
        eprintln!("       rootline symbols <python-file>");
        return ExitCode::from(2);
    }
    let path = match path {
        Some(path) => PathBuf::from(path),
        None => return ExitCode::from(2),
    };
    if command.as_deref() == Some(std::ffi::OsStr::new("index")) {
        return index(&path);
    }
    if command.as_deref() == Some(std::ffi::OsStr::new("symbols")) {
        return symbols(&path);
    }
    eprintln!("usage: rootline index <repository-root>");
    eprintln!("       rootline symbols <python-file>");
    ExitCode::from(2)
}
fn index(path: &Path) -> ExitCode {
    match scan(path) {
        Ok(inventory) => {
            println!("repository: {}", inventory.root.display());
            println!("listing: {}", inventory.mode);
            if let Some(revision) = inventory.git_revision.as_deref() {
                println!("git revision: {revision}");
            }
            println!("files: {}", inventory.artifacts.len());
            println!("bytes: {}", inventory.total_bytes());
            println!("skipped: {}", inventory.issues.len());
            for artifact in &inventory.artifacts {
                println!(
                    "{}\t{}\t{}",
                    artifact.category,
                    artifact.size_bytes,
                    artifact.path.as_path().display()
                );
            }
            // Warnings keep coverage limits visible even when some files were
            // successfully inventoried and the process exits successfully.
            if inventory.mode == ListingMode::FilesystemFallback {
                eprintln!("warning: non-Git fallback does not apply ignore rules");
            }
            if !inventory.issues.is_empty() {
                eprintln!(
                    "warning: skipped entries include symlinks, vanished, unreadable, or non-regular files"
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("rootline: {error}");
            ExitCode::from(1)
        }
    }
}

/// Prints the normalized Code Intelligence IR for one Python file.
///
/// An unsupported extension is an analysis outcome, not an operational
/// failure, so it prints the outcome and exits successfully; unreadable
/// files and adapter failures exit nonzero.
fn symbols(path: &Path) -> ExitCode {
    if !supports_extension(path) {
        println!(
            "{}",
            AnalysisStatus::Unsupported {
                reason: "no Python adapter coverage for this file extension".to_owned(),
            }
        );
        return ExitCode::SUCCESS;
    }
    let bytes = match std::fs::read(path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("rootline: cannot read {}: {error}", path.display());
            return ExitCode::from(1);
        }
    };
    let source = match String::from_utf8(bytes) {
        Ok(source) => source,
        Err(_) => {
            eprintln!("rootline: {} is not valid UTF-8", path.display());
            return ExitCode::from(1);
        }
    };
    // Symbol identity is repository-relative; absolutize only against the
    // current directory for display-scoped inspection, never by following
    // symlinks or escaping into unrelated trees.
    let repo_path = match relative_identity(path) {
        Some(repo_path) => repo_path,
        None => {
            eprintln!(
                "rootline: pass a repository-relative path or one inside the current directory"
            );
            return ExitCode::from(1);
        }
    };
    let mut adapter = match PythonAdapter::new() {
        Ok(adapter) => adapter,
        Err(error) => {
            eprintln!("rootline: {error}");
            return ExitCode::from(1);
        }
    };
    match adapter.parse(&repo_path, &source) {
        Ok(module) => {
            println!("file: {}", repo_path.as_path().display());
            println!("language: {}", module.language());
            println!(
                "analyzer: {} {}",
                module.analyzer().name(),
                module.analyzer().version()
            );
            println!("status: {}", module.status());
            println!("symbols: {}", module.symbols().len());
            for symbol in module.symbols() {
                let range = symbol.range();
                match symbol.id().owner() {
                    Some(owner) => println!(
                        "{}\t{}.{}\t{}..{}\t{}..{}",
                        symbol.id().kind(),
                        owner.full_path(),
                        symbol.id().name(),
                        range.bytes().start().get(),
                        range.bytes().end().get(),
                        range.lines().start().get(),
                        range.lines().end().get(),
                    ),
                    None => println!(
                        "{}\t{}\t{}..{}\t{}..{}",
                        symbol.id().kind(),
                        symbol.id().name(),
                        range.bytes().start().get(),
                        range.bytes().end().get(),
                        range.lines().start().get(),
                        range.lines().end().get(),
                    ),
                }
            }
            println!("imports: {}", module.imports().len());
            for import in module.imports() {
                let range = import.range();
                let names = import
                    .names()
                    .iter()
                    .map(|name| {
                        if name.imported() == name.bound().as_str() {
                            name.bound().as_str().to_owned()
                        } else {
                            format!("{} as {}", name.imported(), name.bound().as_str())
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                println!(
                    "level={}\tmodule={}\twildcard={}\tnames={}\t{}..{}\t{}..{}",
                    import.level(),
                    import.module().unwrap_or("-"),
                    import.is_wildcard(),
                    names,
                    range.bytes().start().get(),
                    range.bytes().end().get(),
                    range.lines().start().get(),
                    range.lines().end().get(),
                );
            }
            for error in module.parse_errors() {
                let range = error.range();
                eprintln!(
                    "parse error: {} at {}..{} (lines {}..{})",
                    error.message(),
                    range.bytes().start().get(),
                    range.bytes().end().get(),
                    range.lines().start().get(),
                    range.lines().end().get(),
                );
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("rootline: {error}");
            ExitCode::from(1)
        }
    }
}

/// Resolves a CLI-provided file argument to repository-relative identity:
/// relative inputs are used as given, absolute inputs must sit inside the
/// current directory.
fn relative_identity(path: &Path) -> Option<RepoPath> {
    if let Ok(relative) = RepoPath::new(path) {
        return Some(relative);
    }
    if path.is_absolute()
        && let Ok(current) = env::current_dir()
        && let Ok(stripped) = path.strip_prefix(&current)
        && let Ok(relative) = RepoPath::new(stripped)
    {
        return Some(relative);
    }
    None
}
