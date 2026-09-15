# Architecture decision records

Architecture decision records explain consequential choices and preserve the context that made them reasonable.

## Process

1. Copy [template.md](template.md) to `NNNN-short-title.md`.
2. Set the status to `Proposed` while the decision is under review.
3. Include alternatives, evidence, risks, and validation criteria.
4. Change the status to `Accepted` when the decision is adopted.
5. Do not rewrite accepted history to hide a changed decision. Add a new ADR that supersedes it.

## Initial ADR backlog

- Rust engine boundaries and workspace structure
- Tree-sitter runtime and grammar distribution
- Code Intelligence IR schema and versioning
- SQLite persistence and migrations
- Repository snapshot and dirty-worktree policy
- Symbol identity across revisions
- Python module resolution scope
- TypeScript module resolution scope
- Incremental invalidation and transactional publication
- Local engine/client protocol
- Graph projection and semantic-zoom state
- Semantic provider and cache boundaries

## Index

| ADR | Decision |
| --- | --- |
| [0001-dual-workspace-monorepo.md](0001-dual-workspace-monorepo.md) | One Git repository with Cargo and pnpm workspaces; grow members only when needed |
| [0002-product-client-and-versioned-protocol.md](0002-product-client-and-versioned-protocol.md) | React/TypeScript/Vite product client and versioned projection DTOs |
| [0003-outcomes-errors-and-observability.md](0003-outcomes-errors-and-observability.md) | Typed operational errors, explicit analysis outcomes, diagnostics, and tracing |
| [0004-code-intelligence-ir.md](0004-code-intelligence-ir.md) | Owned language-neutral IR in `rootline-core` with typed coordinates, identities, and outcomes |
| [0005-tree-sitter-packaging.md](0005-tree-sitter-packaging.md) | Tree-sitter runtime plus crates.io grammar crates contained inside language adapters |
