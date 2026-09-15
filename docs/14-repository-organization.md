# Repository and code organization

## Decision status

The workspace strategy, product-client stack, and dependency direction are accepted design choices. The directories and crates below are a target topology, not permission to create empty scaffolding. Introduce each member when a working capability needs it.

## One repository, two workspaces

```text
Rootline Git repository
├── Cargo workspace: Rust domain model, engine, CLI, and eventual transport
└── pnpm workspace: React/TypeScript/Vite product client and optional shared packages
```

Cargo and pnpm handle their respective ecosystems. Do not add Nx, Turborepo, or another orchestration layer before a measured workflow problem exists. A small cross-workspace command runner may be considered later.

## Initial implementation topology

When implementation is authorized, begin with only the necessary members:

```text
rootline/
├── crates/
│   ├── rootline-core/
│   ├── rootline-engine/
│   └── rootline-cli/
├── fixtures/
├── benchmarks/
├── docs/
├── Cargo.toml
└── AGENTS.md
```

`apps/web` and the pnpm workspace arrive when the first visual product is built. Do not make the browser client a prerequisite for scanner, parser, IR, resolution, or graph correctness.

Initially the engine crate can contain cohesive modules for repository scanning, syntax, Python/TypeScript adapters, resolution, and graph construction. Avoid `utils`, `common`, or `misc` as dumping grounds; name modules for responsibilities.

## Target topology as boundaries mature

```text
rootline/
├── apps/
│   ├── web/                 # product client
│   └── site/                # optional future public site
├── crates/
│   ├── rootline-core/
│   ├── rootline-repo/
│   ├── rootline-analysis/
│   ├── rootline-store/
│   ├── rootline-engine/
│   ├── rootline-protocol/
│   ├── rootline-server/
│   └── rootline-cli/
├── packages/                # only when shared TypeScript packages are real
├── fixtures/
├── benchmarks/
├── docs/
├── scripts/
├── .github/workflows/
├── Cargo.toml
├── pnpm-workspace.yaml
├── AGENTS.md
└── README.md
```

The exact tool configuration files are introduced with the related code and enforcement rules, not as placeholders.

## Rust dependency direction

```text
rootline-cli ───────────────┐
rootline-server ────────────┤
                            ▼
                     rootline-engine
                     /      |      \
                    ▼       ▼       ▼
             rootline-repo analysis store
                    \       |       /
                     \      ▼      /
                      rootline-core
```

`rootline-server` may also depend on `rootline-protocol`. Lower layers must not depend on transport or frontend code. The engine exposes use cases such as opening, indexing, updating, querying, and explaining analysis status; it coordinates components rather than implementing every subsystem internally forever.

### Core

`rootline-core` owns durable domain concepts: typed IDs, repository-relative paths, byte ranges, symbol and relation kinds, analysis statuses, evidence, provenance, symbols, and relations. It has no dependency on Tree-sitter, SQLite, Tokio, Axum, HTTP, React, or model-provider SDKs. Do not turn it into a miscellaneous shared package.

### Repository layer

`rootline-repo`, once extracted, owns repository-bound paths, traversal, ignore behavior, symlink policy, Git snapshots, file classification, hashes, and change detection. It does not know what an AST, symbol, or client component is.

### Analysis layer

`rootline-analysis`, once extracted, owns parsing, adapters, normalized IR, scopes, symbols, imports, module resolution, references, calls, and fact construction. Python and TypeScript start as modules. Language-specific crates appear only if adapter size or isolation makes them necessary.

### Store layer

`rootline-store`, once extracted, owns SQLite schema, migrations, transactions, and persistence queries. Synchronous SQLite access is the initial design direction; an async database layer is not justified merely because HTTP transport is async. The exact library is confirmed through a spike and ADR.

### Protocol and server

`rootline-protocol` owns public DTOs, handshake version, stable error envelopes, and projection contracts. `rootline-server` owns routing, authentication, Origin policy, and transport mapping. Axum/Tokio are candidates at this boundary only. CPU parsing does not become `async` merely because transport is.

## Client framework and code organization

The product client uses React, TypeScript, and Vite in `apps/web`. A separate website may use a different framework if its workload requires it. Do not add Next.js to the product client solely to provide a second server in front of the Rust runtime.

Organize the client by feature, not by a global collection of components/hooks/services:

```text
apps/web/src/
├── app/             # application composition, providers, routing, shell
├── features/
│   ├── repository/
│   ├── graph/
│   ├── search/
│   ├── source/
│   ├── evidence/
│   ├── navigation/
│   ├── flows/
│   └── learning/
├── components/ui/   # genuinely reusable primitives
├── lib/             # narrow API/error/formatting adapters
└── styles/
```

A feature owns its components, hooks, local state, types, and tests. Begin with local React state and shareable URL navigation state. Later, separate fetched engine data from ephemeral UI state; query caching and a lightweight UI store are options only when complexity warrants them. Do not add a large state framework by default.

## Protocol and visualization adapters

```text
internal Rust graph
   → bounded projection
   → versioned protocol DTO
   → Rootline web-domain model
   → graph-library adapter
   → rendered canvas
```

This boundary prevents database or engine refactors from breaking the UI and prevents graph-canvas types from becoming domain types. A startup handshake includes engine and protocol versions; incompatible versions produce a clear message instead of partially rendering invalid data.

## Workspace and test organization

Rust unit tests sit near algorithms; integration tests exercise crate boundaries. Shared fixtures cover Python, TypeScript, repository, and incremental cases. Frontend tests sit near features; end-to-end UI tests are added when the repository-to-symbol navigation flow exists. Cross-platform CI should test filesystem behavior on Linux, macOS, and Windows as soon as the workspace is real.

## Growth triggers

Split a crate or package only when at least one of these becomes concrete:

- a distinct domain responsibility with a stable public contract;
- a dependency that must not leak into lower layers;
- an independent test or build lifecycle;
- a measured compile-time or release boundary;
- multiple consumers that would otherwise duplicate logic.

Do not split solely because the target diagram contains a box.
