# ADR 0005: Tree-sitter runtime and grammar distribution

- Status: Accepted
- Date: 2026-09-15
- Owners: Rootline project
- Supersedes: None
- Superseded by: None

## Context

The Python adapter needs a real parser with error recovery. Options differ in build complexity, platform packaging, grammar versioning, and API stability. The choice must not leak parser types into the IR and must reproduce from a clean checkout on Linux, macOS, and Windows.

## Decision drivers

- Reproducible builds from crates.io with a committed lockfile.
- Bounded build-time cost and no manual grammar submodules.
- Recovery behavior that preserves partial facts with locations.
- A containment boundary parser types cannot cross.

## Considered options

Git-submodule grammars with build scripts would pin sources exactly but add checkout tooling, platform-specific compiler setup, and maintenance per grammar. Hand-written parsers would avoid dependencies but cost far more than the first vertical slice allows. The `tree-sitter` runtime plus per-language grammar crates keeps versioning in Cargo, compiles the C grammar through the `cc` crate, and exposes a uniform node API across future languages.

## Decision

Use the `tree-sitter` runtime crate with the `tree-sitter-python` grammar crate, both at major/minor `0.25`, pinned exactly in `Cargo.lock`. The `PythonAdapter` owns the single `tree_sitter::Parser` (one file at a time; bounded parallelism is a later explicit design), and no Tree-sitter type appears in any public signature outside `rootline-engine::python`. Grammar upgrades require re-verifying fixture ranges and bumping adapter provenance output.

## Consequences

CI needs a C toolchain on all three platforms for the grammar build. Parser initialization cost is paid per adapter construction until measurement justifies reuse or pooling. Tree-sitter `0.25` API shapes (field names, anonymous missing tokens) are encoded in the adapter and covered by fixture tests.

## Validation

Fixture-backed tests pin symbol, import, and error extraction against `tree-sitter-python 0.25.0`; quality gates run on Linux, macOS, and Windows; the R0 benchmark reruns against the pinned corpus.

## Revisit triggers

Grammar packaging failures on a supported platform, measured parser-initialization cost dominating file analysis, or a second language needing a shared parser runtime policy.

## References

- [Target architecture](../04-target-architecture.md)
- [Analysis pipeline](../06-analysis-pipeline.md)
- [Benchmark and evaluation](../11-benchmark-and-evaluation.md)
