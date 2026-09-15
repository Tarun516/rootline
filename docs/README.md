# Rootline documentation

This directory is the consolidated product and engineering plan for Rootline. It incorporates the supplied architecture roadmap, detailed analysis-system research, and the decisions made in discussion.

## Documentation map

| Document | Purpose |
| --- | --- |
| [01-product-vision.md](01-product-vision.md) | Product thesis, user problem, promise, and north-star metric |
| [02-principles-and-boundaries.md](02-principles-and-boundaries.md) | Non-negotiable principles, product boundaries, and decision rules |
| [03-analysis-system-lessons.md](03-analysis-system-lessons.md) | Engineering lessons from mature repository-comprehension systems |
| [04-target-architecture.md](04-target-architecture.md) | Recommended system boundaries, components, and runtime relationships |
| [05-core-model-and-provenance.md](05-core-model-and-provenance.md) | Universal entities, relationships, evidence, confidence, and truth layers |
| [06-analysis-pipeline.md](06-analysis-pipeline.md) | Progressive deterministic analysis and optional semantic enrichment |
| [07-incremental-correctness.md](07-incremental-correctness.md) | Change classification, invalidation, graph repair, and safe deletion |
| [08-semantic-zoom-and-ux.md](08-semantic-zoom-and-ux.md) | Navigation levels, interaction model, learning state, and large-graph behavior |
| [09-capability-roadmap.md](09-capability-roadmap.md) | Capability checkpoints from benchmark harness through simulation mode |
| [10-first-90-days.md](10-first-90-days.md) | Practical initial execution sequence and expected deliverables |
| [11-benchmark-and-evaluation.md](11-benchmark-and-evaluation.md) | Corpus, golden questions, correctness metrics, and performance budgets |
| [12-engineering-operating-model.md](12-engineering-operating-model.md) | How the project should be built, measured, documented, and changed |
| [13-risks-and-open-questions.md](13-risks-and-open-questions.md) | Known hard problems, uncertainty, mitigation, and likely pivots |
| [14-repository-organization.md](14-repository-organization.md) | Cargo/pnpm monorepo, crate boundaries, frontend organization, and protocol DTOs |
| [15-errors-and-diagnostics.md](15-errors-and-diagnostics.md) | Typed operational errors, analysis outcomes, diagnostics, public errors, and tracing |
| [16-configuration-security-portability.md](16-configuration-security-portability.md) | Explicit configuration, local security, source privacy, and cross-platform behavior |
| [implementation-tracker.md](implementation-tracker.md) | Dated capability status, implementation history, verification, and next gates |
| [glossary.md](glossary.md) | Shared terminology |
| [adrs/README.md](adrs/README.md) | Architecture decision record process and index |
| [engineering/README.md](engineering/README.md) | Engineering learning notebook and proposed topics |
| [engineering/rust-engineering-rules.md](engineering/rust-engineering-rules.md) | Binding Rust implementation rules for ownership, APIs, errors, concurrency, determinism, dependencies, testing, and coding agents |

## Authority and naming

- **Rootline** is the canonical project name.
- Earlier source material used a different working name; **Rootline** supersedes it everywhere in this documentation.
- Rootline's architecture and implementation are independently defined by its product principles and validated through reproducible benchmarks.
- When documents disagree, the non-negotiable principles in [02-principles-and-boundaries.md](02-principles-and-boundaries.md) take precedence over roadmap sequencing.
- The roadmap is expected to change when experiments and measurements reveal better choices.
- Rust implementation work must also follow [engineering/rust-engineering-rules.md](engineering/rust-engineering-rules.md) unless an accepted ADR explicitly changes a rule or boundary.

## Current phase

Rootline has begun its first Rust vertical slice. The minimal workspace, inventory CLI, Python parsing into a normalized Code Intelligence IR, a `rootline symbols` inspection command, and Python import resolution with symbol-graph construction are implemented; graph persistence and the product client remain future capabilities. See [implementation-tracker.md](implementation-tracker.md) for dated status and verification. Design checkpoints still govern each new slice.

## Documentation maintenance

Documentation is part of the engineering system:

- Consequential technical decisions receive an ADR.
- New concepts receive engineering notebook entries before or during integration.
- Every capability phase defines measurable exit criteria.
- Unsupported behavior and unresolved analysis are documented rather than hidden.
- Documents should distinguish current behavior, committed design, hypotheses, and future possibilities.

## Accepted implementation direction

The product will be one Git monorepo with Cargo and pnpm workspaces. The product client will use React, TypeScript, and Vite in `apps/web`. Rust begins with a small concrete crate set and splits only at real boundaries. Core analysis remains runtime-agnostic; local transport, versioned DTOs, typed errors, analysis outcomes, diagnostics, and structured tracing have explicit boundaries. These decisions do not authorize placeholder scaffolding before a capability is requested.
