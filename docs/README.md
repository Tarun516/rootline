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
| [glossary.md](glossary.md) | Shared terminology |
| [adrs/README.md](adrs/README.md) | Architecture decision record process and index |
| [engineering/README.md](engineering/README.md) | Engineering learning notebook and proposed topics |

## Authority and naming

- **Rootline** is the canonical project name.
- Earlier source material used a different working name; **Rootline** supersedes it everywhere in this documentation.
- Rootline's architecture and implementation are independently defined by its product principles and validated through reproducible benchmarks.
- When documents disagree, the non-negotiable principles in [02-principles-and-boundaries.md](02-principles-and-boundaries.md) take precedence over roadmap sequencing.
- The roadmap is expected to change when experiments and measurements reveal better choices.

## Current phase

Rootline is currently in its planning stage. Implementation should begin only after the initial benchmark corpus, evaluation questions, minimal IR contract, and first architecture decisions are explicitly accepted.

## Documentation maintenance

Documentation is part of the engineering system:

- Consequential technical decisions receive an ADR.
- New concepts receive engineering notebook entries before or during integration.
- Every capability phase defines measurable exit criteria.
- Unsupported behavior and unresolved analysis are documented rather than hidden.
- Documents should distinguish current behavior, committed design, hypotheses, and future possibilities.
