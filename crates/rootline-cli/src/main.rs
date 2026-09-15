use rootline_engine::inventory::{ListingMode, scan};
use std::env;
use std::path::Path;
use std::process::ExitCode;

// The CLI owns user-facing output and exit codes; the engine returns typed
// results so other callers do not inherit presentation or process policy.
fn main() -> ExitCode {
    let mut args = env::args_os();
    let _program = args.next();
    let command = args.next();
    let path = args.next();
    if command.as_deref() != Some(std::ffi::OsStr::new("index"))
        || path.is_none()
        || args.next().is_some()
    {
        eprintln!("usage: rootline index <repository-root>");
        return ExitCode::from(2);
    }
    let path = match path {
        Some(path) => path,
        None => return ExitCode::from(2),
    };
    match scan(Path::new(&path)) {
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
