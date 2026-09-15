# Implementation tracker

This file records what has been implemented, when, how it was verified, and what remains open. Use local date and time with timezone for each entry. Append a new entry for a material milestone or correction; do not silently rewrite history. `Done` means the listed scope and verification are complete, not that the whole roadmap phase is complete.

## Current capability status

| Capability | Status | Evidence or next gate |
| --- | --- | --- |
| Pinned R0 inventory oracle | Done | Clean archive of pinned revision: 40 files, 337,358 bytes, 0 skipped; portable acquisition and scored comprehension answers remain |
| Minimal Rust workspace | Done | Three real crates; pinned toolchain, lockfile, lint policy, CI, formatting/check/Clippy/tests passed locally |
| First repository inventory CLI | Done with declared limits | Git ignore-aware listing, non-Git fallback, symlink checks, deterministic output; 4 tests and clean-archive benchmark passed |
| Python Code Intelligence IR | Not started | Next implementation slice |
| Python import resolution and symbol graph | Not started | Requires IR |
| SQLite and incremental engine | Not started | Requires fact graph |
| React/TypeScript/Vite client | Not started | Requires local query protocol |

## Chronological log

### 2026-09-15 09:42 IST — Implementation kickoff

- Confirmed Rootline was documentation-only and its worktree was clean.
- Read `AGENTS.md` and the complete binding Rust engineering rules.
- Located Audio Tensor Lab and observed its live checkout was dirty.
- Pinned benchmark revision `e750f443fd1f732e73e9cf9612d0e3b57c053a81`; its committed tree contains 40 files.
- Scope chosen: benchmark metadata, minimal Cargo workspace, safe inventory CLI, and focused tests. Python parsing and UI are not part of this entry.

### 2026-09-15 09:48 IST — Inventory workspace implementation and verification

- Added three functional crates: `rootline-core`, `rootline-engine`, and `rootline-cli`.
- Added a typed lexical `RepoPath`, artifact categories, inventory results, and component-local scan errors.
- Added `rootline index <repository-root>` with Git tracked+untracked ignore-aware enumeration, a clearly labeled non-Git fallback, no symlink following, stable path ordering, size totals, and skip counts.
- Added focused tests for path escape, ordering, symlink behavior, and Git ignore rules.
- Added a pinned Rust toolchain, workspace lints, committed Cargo lockfile, and Linux/macOS/Windows CI quality gates.
- Verification: `cargo fmt --all -- --check`, `cargo check --workspace --all-targets --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, and `cargo test --workspace --all-features --locked` passed locally; 4 unit tests passed.
- Ran the CLI against Rootline's current Git worktree: 43 files, 0 skipped, Git ignore-aware mode.
- Ran the CLI against a clean archive of the pinned Audio Tensor Lab commit: 40 files, 337,358 bytes, 0 skipped; category counts were 30 code, 5 config, 3 documentation, 1 data, 1 other.
- Limits: non-Git fallback does not apply ignore rules; file categories are preliminary; no content hashing, source parsing, graph, persistence, or revision materialization exists yet. The local toolchain is task-specific and not part of the repository.
- Next gate: Python Tree-sitter adapter and owned Code Intelligence IR with fixture-backed symbol/source-range tests.

### 2026-09-15 09:50 IST — Repository-wide code quality rule

- Added a standing `AGENTS.md` rule for meaningful names, useful and maintained comments/API documentation, explicit error handling, idiomatic and secure practices, and verification before code is considered complete.
- This is a documentation and working-rule change; it does not add or modify production code.

### 2026-09-15 09:53 IST — First-slice code documentation

- Added public type documentation and targeted comments to the existing inventory crates and CLI for path safety, listing and skip semantics, preliminary classification, Git metadata limits, and presentation boundaries.
- No runtime behavior or public signatures changed. Verification: Rust formatting, check, Clippy, and tests rerun after the comment update.

## Log entry template

```text
### YYYY-MM-DD HH:MM TZ — Milestone

- Implemented: exact capability and boundaries.
- Evidence: tests, benchmark revision/results, commands, and outcome.
- Limitations: unsupported or incomplete behavior.
- Next gate: the specific condition required before advancing.
```
