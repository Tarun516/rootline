# Rootline

Rootline is a local-first software comprehension engine. It progressively reconstructs a repository into a navigable mental model that connects product architecture and execution flows to modules, files, symbols, and source lines.

The product is designed around one outcome: help a developer understand an unfamiliar or forgotten system faster and more accurately.

Rootline is not primarily a code search tool, a code-generation agent, a static-analysis replacement, or a graph visualization. Those may be capabilities. The product is the continuously maintained comprehension model beneath them.

## Core idea

Traditional code navigation begins with implementation:

```text
directory -> file -> symbol -> source
```

Rootline begins with the system:

```text
system
  -> subsystem
    -> capability
      -> execution or data flow
        -> module
          -> file
            -> symbol
              -> source
```

At every level, users should retain their global context, see why an entity matters, and inspect the evidence behind a relationship or inference.

## Foundational decisions

- A deterministic Rust engine owns scanning, parsing, resolution, graph construction, persistence, incremental maintenance, and queries.
- Python and TypeScript are the first supported languages.
- Tree-sitter provides syntax structure; language adapters normalize it into a universal Code Intelligence IR.
- SQLite is the initial persistent store.
- A TypeScript/React client provides semantic zoom, graph navigation, source inspection, search, and learning state.
- LLMs enrich the model but do not replace deterministic code intelligence or orchestrate the core engine.
- Facts, semantic inferences, and user-created views are separate data layers.
- Provenance, evidence, confidence, analyzer version, and repository revision are first-class.
- Unknown or ambiguous analysis results remain explicitly unknown or ambiguous.
- Incremental correctness is a foundational subsystem.
- The first useful model appears progressively; users do not wait for full analysis before learning.

## Design foundations

Rootline builds on established lessons from static analysis, code intelligence, graph systems, incremental computation, information retrieval, semantic enrichment, and large-graph interaction. Its schemas, APIs, engine, orchestration, interface, and implementation remain independently designed around Rootline's product principles.

## Documentation

Start with [docs/README.md](docs/README.md).

The recommended reading path is:

1. [Product vision](docs/01-product-vision.md)
2. [Principles and boundaries](docs/02-principles-and-boundaries.md)
3. [Analysis-system engineering lessons](docs/03-analysis-system-lessons.md)
4. [Target architecture](docs/04-target-architecture.md)
5. [Core model and provenance](docs/05-core-model-and-provenance.md)
6. [Analysis pipeline](docs/06-analysis-pipeline.md)
7. [Incremental correctness](docs/07-incremental-correctness.md)
8. [Semantic zoom and user experience](docs/08-semantic-zoom-and-ux.md)
9. [Capability roadmap](docs/09-capability-roadmap.md)
10. [First 90-day execution plan](docs/10-first-90-days.md)
11. [Benchmark and evaluation system](docs/11-benchmark-and-evaluation.md)
12. [Engineering operating model](docs/12-engineering-operating-model.md)
13. [Risks and open questions](docs/13-risks-and-open-questions.md)

## Current status

Rootline is in the product-definition and architecture stage. No production implementation exists yet. The first engineering milestone is a benchmarked vertical slice from repository scanning through normalized Code IR to a deterministic symbol/import graph.
