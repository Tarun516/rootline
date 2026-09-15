# Implementation tracker

This file records what has been implemented, when, how it was verified, and what remains open. Use local date and time with timezone for each entry. Append a new entry for a material milestone or correction; do not silently rewrite history. `Done` means the listed scope and verification are complete, not that the whole roadmap phase is complete.

## Current capability status

| Capability | Status | Evidence or next gate |
| --- | --- | --- |
| Pinned R0 inventory oracle | Done, reproducible | `benchmarks/runners/r0-inventory.sh` clones the pinned revision into an isolated temp dir, checks mode/revision/counts/ordering/exact paths, and writes a machine-readable record to `benchmarks/results/` |
| Minimal Rust workspace | Done | Three real crates; pinned toolchain, lockfile, lint policy, CI, formatting/check/Clippy/tests passed locally |
| First repository inventory CLI | Done with declared limits | Git ignore-aware listing, non-Git fallback, symlink checks, deterministic output; 4 tests and clean-archive benchmark passed |
| Python Code Intelligence IR | Done with declared limits | Owned IR in `rootline-core::ir` (typed coordinates, identities, outcomes, publication validation); Tree-sitter Python adapter with 4 fixtures and 7 tests; `rootline symbols` inspection; ADRs 0004/0005 |
| Python import resolution and symbol graph | Done with declared limits | `ModuleIndex` resolver (absolute/relative, ambiguous/unknown/external) and `graph` builder (containment, imports, inheritance, conservative calls) with confidence, evidence, and diagnostics; 7-file fixture package asserts 23 nodes and 34 relations edge-for-edge; [ADR 0006](adrs/0006-python-module-resolution.md) |
| SQLite and incremental engine | Not started | Requires fact graph |
| React/TypeScript/Vite client | Not started | Requires local query protocol |

## Chronological log

### 2026-09-15 16:48 IST — Pre-persistence hardening slice

- Fixed Windows CI blockers before they could fossilize: `.gitattributes` pins LF bytes for fixtures/benchmarks, and `RepoPath::to_canonical_string()` gives persistence/tests a platform-independent form (`display()` stays human-only). MSRV now honestly reads `1.98` (the proved toolchain) instead of an untested `1.85` claim.
- Hardened identity (P0): structural `SymbolId` owners plus declaration indices make legal redefinitions distinct; `validate_module` collides only on full identities; containment derives from structure with no skippable lookup. [ADR 0007](adrs/0007-symbol-declaration-identity.md) supersedes ADR 0004's identity section.
- Hardened relations (P0/P1): `RelationTarget::{Resolved, Ambiguous}` removes the fake primary (no `to()` to misuse); `Confidence` holds assertion strength only; `Graph::try_new` validates at construction; ambiguous targets require 2+ candidates; nodes carry ranges/analyzers and graphs carry repository/revision metadata. [ADR 0008](adrs/0008-relation-targets-and-validation.md) supersedes the relation/confidence sections of ADR 0004.
- Fixed correctness bugs: from-import aliases now resolve through the imported name (with a dedicated regression test); submodule probing applies only under `__init__.py` packages; non-UTF-8 path components produce explicit `Unknown` instead of silent drops; import self-edges skip only for imports so recursion edges survive (pinned by a recursive fixture).
- Restructured the engine: `python/{parse,resolve,builtins}` modules behind a narrowed root API (`mod` + curated `pub use`); CLI updated to the new paths; fixture-heavy adapter/graph suites moved to `crates/rootline-engine/tests/`, CLI boundary tests added at `crates/rootline-cli/tests/cli.rs` (6 tests: usage, exit codes, unsupported, missing/non-UTF-8 inputs).
- Evidence: 59 tests pass (19 core unit, 14 engine unit, 10 adapter + 10 graph integration, 6 CLI); `cargo fmt --check`, `cargo check`, `cargo clippy -D warnings` clean. Sandbox `/tmp` quota remains exhausted, so tests run with `TMPDIR=~/tmp-rootline`.
- Known Windows status: path-separator and CRLF causes are fixed by construction (canonical form is separator-free by design; fixtures are LF-pinned), but confirmation awaits the next CI run on this branch.
- Next gate: SQLite persistence and incremental engine, now building on persistable identities.

### 2026-09-15 12:45 IST — Python import resolution and symbol graph

- Implemented: `rootline-core::graph` (nodes, relations, confidence states, evidence requirement, publication validation) and `rootline-engine::{resolve, graph}` — package-aware import resolution plus a fact-graph builder emitting containment, import, inheritance, and conservatively resolved call edges with per-fact provenance. Unprovable facts stay visible as `GraphDiagnostic`s, never guessed edges.
- Implemented: IR additions that the slice genuinely needed — `CallSite` with a three-way receiver (`Absent`/`Named`/`Opaque`), `Inheritance` facts, plain-vs-`from` form on imports, file identity on `ParsedModule`, and a `ModuleBody` grouping after Clippy flagged a 9-argument constructor.
- Evidence: 42 tests pass (13 core, 29 engine), including 9 resolver unit tests and 8 graph tests over a 7-file fixture package asserting 23 nodes and 34 relations edge-for-edge; `cargo fmt --check`, `cargo check`, `cargo clippy -D warnings` clean. (Note: the sandbox `/tmp` quota was exhausted during this slice, so tests were run with `TMPDIR=~/tmp-rootline`; no repo change was needed.)
- Limitations: no `rootline graph` CLI (needs tested root discovery); builtin base classes diagnose as unknown; dynamic imports uncovered; partial file sets may classify missing-internal modules as external; call targets outside the credible set stay unknown by design.
- Next gate: SQLite persistence and incremental engine over this graph.
- Added [ADR 0006](adrs/0006-python-module-resolution.md).

### 2026-09-15 11:47 IST — Reproducible R0 benchmark, toolchain update, Python IR slice

- Implemented: `benchmarks/corpus.yaml` (R0 source, pinned revision, expected counts), `benchmarks/expected/r0-inventory.txt` (exact 40-path oracle), `benchmarks/runners/r0-inventory.sh` (fresh clone into isolated temp dir, HEAD/cleanliness verification, oracle checks, machine-readable `benchmarks/results/*.json` record), and an updated `benchmarks/README.md` acquisition procedure.
- Implemented: language-neutral IR in `rootline-core::ir` plus a Tree-sitter Python adapter in `rootline-engine::python` (functions/classes/methods with lexical owners, per-module import facts, partial recovery with error locations), 4 fixtures under `fixtures/python/`, and `rootline symbols <python-file>` inspection. Added [ADR 0004](adrs/0004-code-intelligence-ir.md) (IR schema) and [ADR 0005](adrs/0005-tree-sitter-packaging.md) (Tree-sitter packaging).
- Changed: pinned toolchain `1.85.0` → `1.98` to match the working laptop installation while keeping a pinned channel; `rust-version` MSRV stays `1.85`. Added `rootline-core` as a direct `rootline-cli` dependency for presentation types; `Clone`/`Hash` derived on `RepoPath` for symbol identity use.
- Dependencies added with reason: `tree-sitter 0.25` + `tree-sitter-python 0.25` (`std` cannot parse Python; grammar crates keep versioning in Cargo; parser types contained in the adapter per ADR 0005). Pinned exactly in `Cargo.lock`.
- Evidence: `benchmarks/runners/r0-inventory.sh` passed from a clean materialization (40 files, 337,358 bytes, 0 skipped, git-ignore-aware, exact paths, category counts 30/5/3/1/1); `cargo fmt --all -- --check`, `cargo check/clippy/test --workspace --all-targets --all-features --locked` passed (18 tests: 8 core IR, 7 Python adapter, 3 inventory).
- Limitations: no parameters/decorators/calls/columns in the IR yet; `.pyi` stubs explicitly unsupported; adapter parses one file at a time with no bounded-parallelism design; symbol snapshot over the R0 corpus and scored comprehension answers remain future work.
- Next gate: Python import resolution and symbol-graph construction on the IR.

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

### 2026-09-15 10:18 IST — Windows path-test regression

- GitHub Actions Windows reported `path_rejects_escape_and_absolute_input` failing because `/tmp/secret` is rooted but not absolute on Windows; `RepoPath::new` correctly returned `InvalidComponent`.
- Replaced the platform-assuming assertion with Unix and Windows absolute-path cases and a Windows rooted-but-drive-relative case. Production path validation did not change.
- Local Linux quality gates passed after the correction; Windows CI result remains to be confirmed after push.

## Log entry template

```text
### YYYY-MM-DD HH:MM TZ — Milestone

- Implemented: exact capability and boundaries.
- Evidence: tests, benchmark revision/results, commands, and outcome.
- Limitations: unsupported or incomplete behavior.
- Next gate: the specific condition required before advancing.
```
