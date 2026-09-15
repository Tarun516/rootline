# ADR 0001: Dual-workspace monorepo

- Status: Accepted
- Date: 2026-09-15
- Owners: Rootline project
- Supersedes: None
- Superseded by: None

## Context

Rootline combines a native Rust analysis engine with a TypeScript product client. They share product contracts and release coordination but have different build and dependency systems. The repository is currently documentation-first; this ADR chooses organization, not immediate scaffolding.

## Decision drivers

- Preserve one source of truth for architecture, protocol, tests, and docs.
- Let each ecosystem use its native workspace tooling.
- Avoid empty packages and a premature cross-language build platform.
- Keep dependency direction and ownership clear.

## Considered options

Separate repositories would isolate tooling but complicate protocol changes and coordinated tests. A JavaScript-centric monorepo would add orchestration without replacing Cargo's Rust workspace model. One Git repository with Cargo and pnpm workspaces supports both ecosystems directly.

## Decision

Use one Git repository. Cargo manages Rust crates; pnpm manages TypeScript applications/packages. Begin with a small concrete Rust crate set and add pnpm when the client begins. Do not add Nx, Turborepo, or similar orchestration unless a measured workflow need appears. Do not create all target crates or directories up front.

## Consequences

The repository can coordinate protocol and end-to-end changes. Tooling remains ecosystem-specific. Some cross-workspace commands may initially require separate invocations; a small command runner may be introduced later. CI and release configuration must respect both workspace systems.

## Validation

The first vertical slice should build and test with Cargo alone. When the client arrives, both workspaces should run focused and integrated checks without duplicating dependencies or introducing a second server runtime.

## Revisit triggers

Measured build/test orchestration pain, independent release requirements, or a demonstrated need for distributed ownership.

## References

- [Repository organization](../14-repository-organization.md)
- [Target architecture](../04-target-architecture.md)
