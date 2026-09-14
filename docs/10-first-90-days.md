# First 90-day execution plan

## Purpose

This plan creates a serious vertical slice without treating the schedule as more authoritative than engineering evidence. Windows may move. Deliverables and learning goals should remain explicit.

## Days 1–10: reference and measurement foundation

### Deliverables

- Freeze initial benchmark repository revisions.
- Confirm Audio Tensor Lab as the primary correctness oracle.
- Select a medium TypeScript repository with API, persistence, and frontend/backend boundaries.
- Define golden comprehension questions and expected answers.
- Capture competing-tool baseline results on the same revisions.
- Create the Rootline CLI/workspace skeleton only after core ADRs are drafted.
- Define benchmark output formats and repeatability rules.
- Create ADR and engineering-notebook conventions.
- Run focused spikes for safe scanning, Rust Tree-sitter integration, and SQLite access.

### Decisions to record

- workspace and crate boundaries;
- supported operating systems for the first slice;
- repository snapshot/dirty-worktree policy;
- Tree-sitter Rust library and grammar packaging approach;
- initial SQLite library and migration strategy;
- benchmark fixture policy.

### Exit checkpoint

The corpus and revisions are reproducible, the golden questions are reviewable, and a minimal benchmark can record scanner timing and correctness.

## Days 11–25: Python vertical slice

### Deliverables

- Safe Rust repository scanner.
- Ignore, path, symlink, and repository-boundary behavior.
- Content hashing and repository metadata.
- Python Tree-sitter adapter.
- Initial Code Intelligence IR.
- File, function, class, method, ownership, import syntax, and source-range extraction.
- Initial SQLite schema for repository, revision, artifact, symbol, relation, evidence, and analyzer run.
- CLI inspection commands for inventory and symbols.
- Deterministic snapshot tests on Audio Tensor Lab.

### Learning spikes

- CST versus AST and Tree-sitter recovery behavior;
- Python declaration and scope semantics;
- stable identity under line movement;
- parser reuse and safe parallelism;
- transaction and migration behavior in SQLite.

### Exit checkpoint

Audio Tensor Lab produces a stable, inspectable artifact and symbol graph with source ranges and analyzer evidence.

## Days 26–40: resolution and first end-to-end interface

### Deliverables

- Python absolute and relative module resolution.
- Explicit external, unresolved, ambiguous, unsupported, and failed states.
- Supported call/reference candidates with conservative target resolution.
- Local query API or IPC boundary.
- Minimal TypeScript/React interface.
- Repository, module/directory, file, and symbol navigation.
- Source panel, breadcrumbs, and `you are here` context.
- Analysis status and evidence display.

### Learning spikes

- Python package and namespace-package resolution;
- symbol tables and lexical scopes;
- call-graph precision versus coverage;
- local HTTP versus IPC tradeoffs;
- graph projection rather than full-graph transfer.

### Exit checkpoint

A user can index Audio Tensor Lab, navigate from repository structure to a symbol, inspect its source and evidence, and answer supported dependency questions without an LLM.

## Days 41–55: incremental correctness

### Deliverables

- Content and structural fingerprints.
- Git-aware and direct-filesystem change detection.
- Dependency-aware invalidation plan.
- Candidate graph construction and transactional publication.
- Symbol preservation/deletion validation.
- Analyzer-version invalidation.
- Cold versus incremental benchmarks.
- Full-rebuild convergence tests.
- Visible update diagnostics.

### Learning spikes

- incremental computation models;
- identity across rename and move;
- reverse dependency indexing;
- safe graph repair;
- config-root invalidation.

### Exit checkpoint

A localized function change updates the smallest safe graph region, a failed update preserves the last valid graph, and incremental output matches a clean rebuild for the fixtures.

## Days 56–70: TypeScript and significance

### Deliverables

- TypeScript/JavaScript adapter.
- Relative resolution, extension/index probing, `tsconfig` aliases, and basic workspace boundaries.
- Cross-language-independent IR conformance tests.
- Deterministic importance signals and explanations.
- Lexical/fuzzy search and basic graph traversal queries.
- Comparative benchmark against selected repository-comprehension tools on Python and TypeScript repositories.

### Learning spikes

- TypeScript module-resolution modes and package exports;
- monorepo/project configuration boundaries;
- useful centrality metrics versus misleading graph popularity;
- ranking evaluation.

### Exit checkpoint

The TypeScript benchmark has credible symbol/import coverage, and important entities rank plausibly with explainable factors.

## Days 71–90: architecture inference prototype

### Deliverables

- Deterministic community and boundary signals.
- Entry-point and subsystem candidate computation.
- Optional semantic labeling behind a provider-neutral task contract.
- Provenance-aware inferred concepts and memberships.
- Evidence inspector for subsystem assignments.
- First semantic-zoom prototype between repository, subsystem candidate, module/file, and symbol.
- Evaluation against a manually drawn Audio Tensor Lab architecture.
- Refined post-90-day roadmap based on measurements.

### Learning spikes

- Louvain, Leiden, and alternative community algorithms;
- cohesion, coupling, and bridge detection;
- cluster stability across revisions;
- semantic labeling evaluation;
- hierarchical graph layout and context-preserving transitions.

### Exit checkpoint

Rootline produces inspectable architecture candidates on the golden repository, the user can descend into implementation without losing context, and disagreements with the manual architecture can be explained.

## Work deliberately excluded from the first 90 days

- autonomous coding agents;
- a full chat experience;
- broad language coverage;
- a hosted platform;
- distributed graph processing;
- a specialized graph database without measurements;
- architecture simulation;
- custom ANN or GPU infrastructure;
- extensive collaboration/organization features;
- visual polish that does not improve comprehension.

## Ongoing practices throughout the 90 days

- Keep one working vertical slice alive.
- Record consequential decisions as ADRs.
- Write learning notes for unfamiliar concepts.
- Benchmark the same pinned revisions.
- Track performance, coverage, and correctness from the first scanner.
- Treat failures and unknowns as product-visible data.
- Review the roadmap at each exit checkpoint.
