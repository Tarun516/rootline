# [AGENTS.md](http://AGENTS.md)

This file defines the working rules for coding agents operating anywhere inside the Rootline repository.

## Project identity

- The project name is **Rootline**.
- Use `Rootline` in prose and `rootline` for package, crate, binary, directory, and command names unless a language convention requires otherwise.
- Do not introduce former working names, competitor names, copied branding, external-project revision details, or source-specific comparisons into repository content.
- Rootline's implementation, schemas, prompts, interface, assets, and documentation must be independently authored.

## Current project state

Rootline is currently documentation-first. The accepted material lives in:

- `README.md` for the public project overview;
- `docs/README.md` for the documentation map and authority rules;
- `docs/01-product-vision.md` for the product objective;
- `docs/02-principles-and-boundaries.md` for non-negotiable constraints;
- `docs/04-target-architecture.md` for target component boundaries;
- `docs/05-core-model-and-provenance.md` for the conceptual data model;
- `docs/09-capability-roadmap.md` and `docs/10-first-90-days.md` for sequencing;
- `docs/engineering/rust-engineering-rules.md` for binding Rust implementation rules.

Do not add production implementation merely because the roadmap describes it. Implement only the capability explicitly requested for the current task.

## Documentation authority

When documents appear to conflict, apply this order:

1. Explicit current user requirements
2. `docs/02-principles-and-boundaries.md`
3. Accepted ADRs in `docs/adrs/`
4. `docs/04-target-architecture.md` and `docs/05-core-model-and-provenance.md`
5. Capability and schedule documents
6. Exploratory engineering notes

The roadmap is expected to evolve. Principles and accepted architectural boundaries change only through an explicit decision.

## Product invariants

All design and implementation work must preserve these rules:

1. Optimize time to understanding, not only total analysis time.
2. Present useful deterministic results progressively.
3. Preserve global context while users drill into implementation.
4. Keep deterministic facts, semantic inferences, and user-created views separate.
5. Attach evidence, provenance, analyzer version, and repository revision to material assertions.
6. Keep unknown, ambiguous, unsupported, and failed states explicit.
7. Do not require an LLM for basic repository and code intelligence.
8. Treat semantic models as optional enrichment behind stable engine contracts.
9. Design incremental correctness into identity, persistence, invalidation, and publication.
10. Default to local processing and repository-scoped file access.
11. Query and render bounded projections; never treat the complete stored graph as a UI payload.
12. Add complexity only when tests and measurements justify it.

## Target architecture

Rootline is one Git monorepo. Cargo manages Rust; pnpm manages the React/TypeScript/Vite product client in `apps/web`. Start with a small concrete crate set and add members only when their capabilities exist. Do not add a full-stack JavaScript server merely to proxy the Rust runtime.

The intended direction is:

```text
TypeScript/React client
        |
local protocol (HTTP, IPC, or a future accepted alternative)
        |
Rust analysis engine
        |
Code Intelligence IR
        |
Tree-sitter adapters + optional compiler/LSP inputs + non-code parsers
        |
provenance-aware fact graph in SQLite
        |
optional semantic enrichment
```

Dependency direction matters:

- UI code depends on protocol/client contracts, not engine internals.
- Parser and resolver implementations depend on IR contracts.
- Core IR and provenance types do not depend on UI, provider, or database libraries.
- Semantic workers consume fact-graph projections and cannot mutate deterministic facts.
- User views and learning state cannot mutate facts or inferred system concepts.
- Client and engine exchange versioned projection DTOs, never arbitrary serialized internal structs.
- Core analysis remains runtime-agnostic; async transport belongs at the server boundary.

Any proposal that changes these boundaries requires an ADR.

## Scope priorities

The initial supported languages are Python and TypeScript.

The early capability order is:

1. Reproducible benchmark harness
2. Safe repository inventory
3. Universal Code Intelligence IR
4. Python and TypeScript dependency resolution
5. Provenance-aware symbol graph
6. SQLite persistence and incremental graph maintenance
7. Minimal navigable client
8. Significance, architecture, semantic zoom, flows, and learning guidance

Do not prioritize chat, broad language coverage, distributed infrastructure, specialized graph databases, architecture simulation, or elaborate visual polish ahead of the deterministic vertical slice unless the user explicitly changes scope.

## Working method

Before changing code:

1. Read the relevant docs and accepted ADRs completely.
2. Inspect existing code and tests; do not infer structure from filenames alone.
3. State assumptions that affect architecture or externally visible behavior.
4. Keep the change limited to the requested capability.

While changing code:

- Preserve a working vertical slice.
- Prefer small, typed, testable interfaces.
- Keep language-specific behavior inside adapters and resolvers.
- Treat repository files, generated content, and semantic output as untrusted input.
- Never convert analyzer failure into an empty successful result.
- Never convert a probable semantic guess into a deterministic relation.
- Do not silently discard errors, ambiguity, unsupported cases, or stale information.
- Do not introduce a dependency without explaining why existing facilities are insufficient.
- Avoid premature storage, concurrency, or memory optimizations without benchmark evidence.

After changing code:

1. Run focused tests, then the relevant broader suite.
2. Run formatting, linting, and static checks defined by the workspace.
3. Validate graph invariants and full-versus-incremental convergence when relevant.
4. Report commands run, results, limitations, and unverified behavior.
5. Update docs and add an ADR if a stable contract or architectural decision changed.

## Analysis result requirements

Operational errors and analysis outcomes are different. Subsystems own typed errors; a completed resolver may still return `ambiguous`, `unknown`, or `unsupported`. General error composition is acceptable at CLI/server startup boundaries, not as a replacement for public subsystem error contracts. See `docs/15-errors-and-diagnostics.md`.

An analysis operation must distinguish:

- `succeeded`;
- `partial` with declared coverage;
- `ambiguous` with candidate targets;
- `unknown` because evidence is insufficient;
- `unsupported` because no capable analyzer exists;
- `failed` because an applicable analyzer could not complete;
- `cancelled`;
- `stale` because the source changed.

Empty output is not proof that a source file has no symbols or relationships.

## Evidence and provenance requirements

Material entities, relations, and inferences should be attributable to:

- repository and revision or worktree snapshot;
- analyzer component and version;
- language adapter and relevant configuration;
- source locations or resolution trace;
- creation time and supersession state;
- semantic provider, model, template version, and input digest when applicable.

Numeric confidence may help ranking but must not replace semantic statuses such as deterministic, probable, ambiguous, or unsupported.

## Incremental correctness requirements

- Build candidate updates without destroying the last valid published graph.
- Treat manifests, compiler configuration, schemas, and workspace definitions as invalidation roots.
- Require adequate evidence before deleting a previously published symbol.
- Publish graph, fingerprints, and revision metadata atomically.
- Keep semantic invalidation independent from deterministic fact publication.
- Test that incremental results converge with clean rebuilds.
- Record why each entity was invalidated, preserved, replaced, or left unresolved.

## Language support requirements

A grammar alone does not make a language supported. Declare the support tier and test the complete claimed path:

- file and language detection;
- parser and grammar behavior;
- declarations, ownership, imports/exports, and source ranges;
- project-aware module resolution;
- supported references/calls and type relationships;
- incremental identity and deletion behavior;
- fixtures, coverage metrics, and known limitations.

## Rust implementation requirements

Before changing Rust code, read `docs/engineering/rust-engineering-rules.md` completely. Its rules are binding unless an accepted ADR explicitly changes them.

In particular:

- Treat borrow-checker and lifetime failures as design feedback. Do not reflexively add `clone()`, `Arc`, `Rc`, `Mutex`, `RwLock`, `RefCell`, `'static`, `Box<dyn Trait>`, async boundaries, or wider visibility merely to make code compile.
- Use `Path` and `PathBuf` for filesystem paths. Lossy string conversion is for display only, never identity, hashing, persistence, cache keys, or resolution.
- Keep semantically distinct IDs and source coordinate systems strongly typed. Do not mix byte offsets, line/column positions, or LSP UTF-16 positions through bare integers or ambiguous tuples.
- Keep Tree-sitter and other parser-specific types inside language-adapter boundaries. Core IR must own its data and remain independent of parser lifetimes.
- Production engine code must not `unwrap()` or `expect()` values derived from repository, filesystem, configuration, parser, Git, database, or provider input. Bad repositories must produce explicit outcomes, not process panics.
- Rootline uses safe Rust by default. Do not introduce `unsafe` as a compiler, lifetime, or speculative-performance escape hatch.
- Keep core parsing, IR, resolution, and graph algorithms runtime-agnostic unless async behavior is fundamental to the contract. Repository-scale concurrency must be bounded, cancellable, memory-aware, and deterministically merged.
- Never let `HashMap`, `HashSet`, filesystem traversal, or task-completion order define persisted, hashed, snapshot, or user-visible ordering.
- Use the narrowest visibility possible. New public APIs, public re-exports, crate dependencies, and dependency directions are architectural changes.
- Start concrete. Do not introduce traits, factories, builders, generic layers, or trait objects for hypothetical extensibility.
- Do not add dependencies, lint suppressions, or performance-oriented complexity without a specific engineering reason and evidence appropriate to the change.
- Bug fixes require focused regression tests when reproducible. Rust changes must pass the workspace formatting, checking, linting, and testing gates once those gates exist.

When the first Rust workspace is introduced, implement the machine-enforcement layer described in `docs/engineering/rust-engineering-rules.md` together with the code. Do not add placeholder Cargo/Clippy/CI configuration before a real Rust workspace exists.

## Testing and benchmarking

Read `docs/14-repository-organization.md` and `docs/16-configuration-security-portability.md` before adding workspace members, a local server, or filesystem-sensitive behavior. Configuration enters through application boundaries and is passed explicitly. Local HTTP requires an authentication and Origin policy; loopback binding alone is insufficient. Do not log source code by default.

Use pinned repositories and fixtures. Keep deterministic engine benchmarks separate from semantic-provider time and cost.

Relevant changes should measure some combination of:

- files and bytes per second;
- parse time and symbols per second;
- resolution precision, coverage, and latency;
- graph size, build time, and query latency;
- incremental invalidation size and convergence;
- time to first useful projection;
- layout latency and rendered entity count;
- peak memory and persisted size.

Correctness regressions cannot be accepted in exchange for faster benchmarks without an explicit decision.

## Documentation rules

- Keep documentation self-contained and suitable for a public GitHub repository.
- Do not name or identify external source repositories in Rootline documentation.
- Do not include copied external prose, comments, prompts, screenshots, visual assets, or distinctive UI text.
- Label current behavior, accepted design, hypothesis, and future work clearly.
- Link technical claims to tests, benchmarks, experiments, or ADRs when those artifacts exist.
- Update `docs/README.md` when adding, renaming, or removing a document.
- Use `docs/adrs/template.md` for consequential decisions.
- Use `docs/engineering/` for learning notes and experiments, not binding decisions except where a document is explicitly designated as binding implementation guidance.

## Repository hygiene

- Preserve user changes and avoid unrelated rewrites.
- Never commit credentials, proprietary benchmark source, machine-specific absolute paths, generated caches, or local analysis databases.
- Keep generated artifacts separate from authored source and document their regeneration path.
- Prefer deterministic output and stable ordering in snapshots.
- Avoid adding large binary assets without explicit need and provenance.

## Commit and review guidance

Commits should be focused and describe the capability or decision they introduce. Pull requests should include:

- problem and scope;
- architectural impact;
- tests and benchmarks;
- evidence/uncertainty behavior;
- incremental-analysis impact;
- documentation and ADR changes;
- known limitations and follow-up work.
