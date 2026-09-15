# Rootline

Rootline is a local-first software comprehension engine. It progressively turns a repository into a navigable mental model—from system and subsystem views through capabilities and flows to modules, files, symbols, and source lines—while preserving evidence, uncertainty, context, and a developer's learning state.

The project is currently in the design stage. No production engine or client has been implemented yet.

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

## Documentation

Start with the [documentation index](docs/README.md). The most useful entry points are the [product vision](docs/01-product-vision.md), [principles](docs/02-principles-and-boundaries.md), [target architecture](docs/04-target-architecture.md), [repository organization](docs/14-repository-organization.md), [capability roadmap](docs/09-capability-roadmap.md), and [first 90-day plan](docs/10-first-90-days.md).

Agent working rules are in [AGENTS.md](AGENTS.md). Consequential architecture choices belong in [ADRs](docs/adrs/README.md); engineering learning and implementation rules live in [docs/engineering](docs/engineering/README.md).

## Current milestone

The first engineering milestone is a benchmarked repository inventory and Python vertical slice: safe scanning, Tree-sitter structure extraction, normalized symbols and imports, evidence, persistence, CLI queries, and focused correctness tests. Crates, applications, and workspaces will be created only as their capabilities are implemented.
