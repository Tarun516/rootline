//! CLI boundary tests: argument validation, exit codes, and output
//! behavior through the built binary. Engine semantics stay in the engine
//! test suites; these assert only what the process contract promises.

//! Fixture setup failure must fail the test immediately.
#![expect(
    clippy::expect_used,
    reason = "fixture setup failure must fail the test immediately"
)]

use std::path::{Path, PathBuf};
use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_rootline"))
}

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("rootline-cli-test-{}-{name}", std::process::id()));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("clear stale scratch dir");
    }
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

fn write(dir: &Path, name: &str, contents: &[u8]) -> PathBuf {
    let path = dir.join(name);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("create parent dirs");
    }
    std::fs::write(&path, contents).expect("write fixture");
    path
}

#[test]
fn rejects_missing_arguments_with_usage() {
    let output = binary().output().expect("run binary");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("usage: rootline index"), "{stderr}");
}

#[test]
fn rejects_unknown_commands_with_usage() {
    let output = binary()
        .args(["frobnicate", "."])
        .output()
        .expect("run binary");
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn indexes_a_directory() {
    let dir = scratch_dir("index");
    write(&dir, "main.py", b"def run():\n    return 1\n");
    write(&dir, "notes.md", b"# notes\n");
    let output = binary()
        .args(["index", dir.to_str().expect("scratch is UTF-8")])
        .output()
        .expect("run binary");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("files: 2"), "{stdout}");
    assert!(stdout.contains("main.py"), "{stdout}");
    std::fs::remove_dir_all(&dir).expect("remove scratch dir");
}

#[test]
fn reports_symbols_for_python_files() {
    let dir = scratch_dir("symbols");
    write(&dir, "service.py", b"def serve():\n    return True\n");
    // Symbol identity is repository-relative, so invoke from the fixture
    // directory with a relative path, as real usage does.
    let output = binary()
        .current_dir(&dir)
        .args(["symbols", "service.py"])
        .output()
        .expect("run binary");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("status: succeeded"), "{stdout}");
    assert!(stdout.contains("serve"), "{stdout}");
    std::fs::remove_dir_all(&dir).expect("remove scratch dir");
}

#[test]
fn reports_unsupported_for_other_extensions() {
    let dir = scratch_dir("unsupported");
    let file = write(&dir, "config.toml", b"[tool]\n");
    let output = binary()
        .args(["symbols", file.to_str().expect("scratch is UTF-8")])
        .output()
        .expect("run binary");
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("stdout is UTF-8");
    assert!(stdout.contains("unsupported"), "{stdout}");
    std::fs::remove_dir_all(&dir).expect("remove scratch dir");
}

#[test]
fn fails_cleanly_for_missing_and_non_utf8_files() {
    let dir = scratch_dir("failures");
    let missing = dir.join("ghost.py");
    let output = binary()
        .args(["symbols", missing.to_str().expect("scratch is UTF-8")])
        .output()
        .expect("run binary");
    assert_eq!(output.status.code(), Some(1));

    let bad = write(&dir, "bad.py", b"def f():\n    return \xff\n");
    let output = binary()
        .args(["symbols", bad.to_str().expect("scratch is UTF-8")])
        .output()
        .expect("run binary");
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8(output.stderr).expect("stderr is UTF-8");
    assert!(stderr.contains("not valid UTF-8"), "{stderr}");
    std::fs::remove_dir_all(&dir).expect("remove scratch dir");
}
