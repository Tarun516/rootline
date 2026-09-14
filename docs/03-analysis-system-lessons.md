# Analysis-system engineering lessons

## Purpose

Mature repository-comprehension systems reveal a consistent set of difficult engineering problems. Rootline should use these lessons to:

- establish capability and performance baselines;
- identify hard engineering areas early;
- study successful conceptual patterns;
- reveal failure modes in whole-repository analysis and graph visualization;
- test whether Rootline's proposed differentiation is real.

Rootline should independently define its schemas, APIs, engine, orchestration, UI composition, and implementation. Prior-art behavior is useful for comparison, but it does not define Rootline's architecture.

## A representative mature pipeline

A capable repository-comprehension pipeline is more than a dashboard wrapped around an LLM:

```text
repository
  -> project scanner
  -> deterministic import resolution
  -> file/import graph
  -> community detection and semantic batching
  -> deterministic extraction + LLM enrichment
  -> merged knowledge graph
  -> architecture, tour, and domain analysis
  -> validation
  -> knowledge-graph.json
  -> dashboard and graph-grounded chat
```

Full analysis is naturally multi-phase. Incremental updates need a separate, cheaper reconciliation path.

## Deterministic analysis already present

A proven approach uses Tree-sitter with reusable parser instances and language-specific extractors. Structural analysis can cover TypeScript/JavaScript, Python, Go, Rust, Java, Ruby, PHP, C/C++, C#, Dart, Kotlin, Swift, and Scala when each language has a complete adapter.

The normalized analysis contract includes:

- functions and methods;
- classes and ownership;
- imports and exports;
- call-graph entries;
- parameters and return types where available;
- source line ranges;
- non-code definitions, services, endpoints, steps, and resources.

Its analyzer plugin abstraction exposes operations equivalent to:

```text
analyzeFile
resolveImports
extractCallGraph
extractReferences
analyzeFileFull
```

The `analyzeFileFull` path parses once and derives both structure and calls, avoiding duplicate syntax-tree work. Parser instances are cached by language.

### Rootline lesson

Keep a stable language-adapter boundary and a normalized Code Intelligence IR. Treat parser lifecycle, grammar loading, and parser reuse as engine responsibilities rather than per-file implementation details.

## Non-code analysis

A complete comprehension model must represent software beyond source files. Dedicated parsers should eventually cover formats and artifacts including:

- Markdown and configuration files;
- Dockerfiles and shell/build scripts;
- SQL, GraphQL, and Protobuf;
- Terraform;
- YAML, JSON, TOML, and environment files.

The graph can represent services, pipelines, resources, tables, schemas, endpoints, configuration, and documentation.

### Rootline lesson

Architecture is the relationship among code, data, contracts, deployment, infrastructure, and configuration. Rootline's initial vertical slice can be narrow, but its IR and evidence model must allow these artifacts later.

## Import resolution

A project-aware resolver must not map raw import strings directly to paths. A mature resolver may support, with language-specific limitations:

- TypeScript/JavaScript relative imports, extension/index probing, CommonJS requires, multiple `tsconfig.json` files, `baseUrl`, and path aliases;
- Python relative and absolute package imports;
- Go module prefixes and multiple `go.mod` files;
- Rust crate/self/super paths and module declarations;
- Java, Kotlin, Scala, and C# package-style paths;
- Ruby `require` and `require_relative`;
- PHP Composer PSR-4 mappings;
- C/C++ include probes;
- Swift module targets.

### Rootline lesson

Import and reference resolution is a first-class subsystem. Each language requires explicit resolution semantics, configuration context, measurable coverage, and unresolved states.

## Graph-aware batching

Graph-aware batching can build an import graph and use Louvain community detection to group related code files for semantic analysis. It should have deterministic fallbacks, merge tiny communities to reduce task startup overhead, and group non-code artifacts by relevant categories.

### Rootline lesson

When semantic enrichment is used, context should follow repository topology rather than arbitrary file counts. In Rootline this should be an internal scheduling concern behind stable engine APIs, not the product's primary orchestration model.

## Incremental analysis

A robust incremental system maintains content and structural fingerprints, separates unchanged, structurally equivalent, structural, new, and deleted files, refreshes import maps, prunes stale graph fragments, and selectively reanalyzes changed areas.

Symbol-loss validation must be deliberately conservative. A symbol is not considered deleted merely because a best-effort parser or semantic analyzer omitted it. Unsupported languages, parse failures, ambiguous identities, and incomplete ownership evidence can block publication rather than corrupting the durable graph.

### Rootline lesson

Incremental analysis is correctness-preserving graph maintenance under repository evolution. Rootline should design stable identities, dependency roots, analyzer-version tracking, transactional publication, and evidence-aware deletion into the first persisted model.

## Architecture and tours

Architecture and tour generation can combine deterministic signals with optional semantic interpretation. Relevant signals include:

- directory and node-type groupings;
- fan-in and fan-out;
- inter-directory dependencies and cohesion;
- entry points and dependency traversal;
- deployment and data topology;
- clusters and non-code inventory.

A guided-learning system can convert these signals into a learning sequence. A domain view can model domains, flows, and steps that point back to implementation locations.

### Rootline lesson

Use graph algorithms and heuristics to surface candidates; use semantic models to label and explain them. Rootline should turn the system/capability/flow hierarchy into the persistent navigation model, not only a generated alternative view.

## Visualization lessons

A modern graph client may combine React, a graph-canvas library, a state store, layered layout engines, graph algorithms, and syntax highlighting. Useful capabilities include overview layers, drill-down, containers, source viewing, breadcrumbs, filtering, search, change overlays, tours, domain views, and path finding.

Large-graph interfaces commonly encounter wide ranks, unreadable labels, and edge spaghetti. Effective responses include:

- ELK-based layout;
- container aggregation;
- aggregated cross-container edges;
- a two-stage layout that positions containers first and lays out children lazily;
- automatic expansion for focus, search, and tour targets;
- viewport preservation during expansion.

### Rootline lesson

Progressive disclosure and aggregation are mandatory. Rootline should extend this into true semantic zoom where each level changes the represented entity type and question being answered.

## Schema limitation Rootline must avoid

A graph may have broad node and edge taxonomies, validation, alias normalization, and sanitization while still failing to make provenance and supporting evidence first-class for every statement. Rootline must avoid that gap.

Rootline should not treat these as equivalent:

```text
calls: proven by supported deterministic analysis at source line 74
part_of_subsystem: inferred from clustering and semantic labeling at confidence 0.82
grouped_with: created by the user for a personal view
```

This distinction belongs in the schema, persistence layer, query API, and interface.

## What Rootline should keep conceptually

- deterministic extraction followed by optional semantic interpretation;
- Tree-sitter language adapters and reusable parser runtimes;
- non-code artifacts as architectural entities;
- project-aware import resolution;
- graph-aware semantic batching;
- structural fingerprints and conservative incremental reconciliation;
- graph validation and treating generated data as untrusted;
- domain/flow views and pedagogical tours;
- container aggregation and lazy layout for large graphs;
- deterministic benchmarks separated from semantic-provider timing.

## What Rootline should deliberately change

- Replace agent-led deterministic orchestration with a stable native engine.
- Make provenance and uncertainty mandatory in the graph model.
- Make the system/capability/flow hierarchy the primary experience.
- Make semantic enrichment optional, queued, cached, and replaceable.
- Stream usable results as stages complete.
- Persist learning state and user-created views separately from source truth.
- Build combined lexical, structural, graph, and eventually ANN retrieval rather than relying on semantic vector search alone.
- Treat prior-art comparisons as design inputs, never as architectural constraints.
