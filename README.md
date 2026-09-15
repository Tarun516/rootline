# Rootline

Rootline is a local-first software comprehension engine. It progressively turns a repository into a navigable mental model—from system and subsystem views through capabilities and flows to modules, files, symbols, and source lines—while preserving evidence, uncertainty, context, and a developer's learning state.

The first Rust implementation slice is underway: a repository-inventory engine and CLI exist, along with Python code intelligence (a Tree-sitter adapter producing a language-neutral Code Intelligence IR, inspectable via `rootline symbols`). Import resolution and symbol-graph construction with explicit uncertainty are implemented; graph persistence and the product client have not been implemented yet.

## Product principles

- Optimize time to understanding, not only indexing completion.
- Make useful deterministic analysis available progressively.
- Keep facts, semantic inferences, and user-created views separate.
- Show provenance and uncertainty for important relationships.
- Provide basic code intelligence without an LLM.
- Preserve global context as users drill into source.
- Treat incremental correctness and local privacy as foundational.

## Planned architecture

One Git monorepo will contain a Cargo workspace for the Rust analysis engine and a pnpm workspace for a React/TypeScript/Vite client in `apps/web`. The client will communicate with the Rust-owned runtime through a local, versioned protocol and receive bounded graph projections rather than internal engine structs.

The initial languages are Python and TypeScript. Tree-sitter adapters will normalize syntax into a Code Intelligence IR; language-aware resolvers will produce a provenance-aware fact graph persisted in SQLite. Semantic enrichment is optional and cannot mutate deterministic facts.
