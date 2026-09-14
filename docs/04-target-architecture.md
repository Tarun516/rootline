# Target architecture

## Architectural objective

Rootline should separate deterministic repository analysis, semantic interpretation, and interactive presentation through stable contracts. The deterministic engine owns truth production and maintenance. Semantic workers consume facts and produce explicitly identified inferences. The UI queries projections suited to the user's current abstraction level.

## High-level system

```text
+------------------------------------------------------+
| TypeScript client                                    |
|                                                      |
| semantic zoom | graph canvas | source viewer         |
| search | evidence inspector | learning state | views |
+--------------------------+---------------------------+
                           | local HTTP / IPC
+--------------------------v---------------------------+
| Rust engine                                           |
|                                                       |
| scanner | hashing | parser runtime | language adapters|
| symbol table | import/reference resolvers | call graph|
| graph store | incremental engine | graph algorithms   |
| flow engine | significance | query/search APIs        |
+--------------------------+---------------------------+
                           | Code Intelligence IR
          +----------------+----------------+
          |                |                |
     Tree-sitter     optional LSP/compiler  config/data/infra
       syntax             semantics             parsers
          +----------------+----------------+
                           |
                     persisted fact graph
                           |
+--------------------------v---------------------------+
| Semantic engine                                      |
|                                                      |
| subsystem discovery | architecture labeling          |
| flow inference | explanations | guided learning      |
| embeddings | optional provider adapters              |
+------------------------------------------------------+
```

## Component responsibilities

### Repository scanner

Owns safe and repeatable repository inventory:

- repository-bound path normalization;
- symlink and traversal safety;
- ignore semantics;
- Git revision and worktree metadata;
- file type, language, size, and manifest detection;
- bounded parallel reads;
- content hashes and change candidates;
- stable file identities.

The scanner must never escape the repository boundary or silently follow unsafe links.

### Parser runtime

Owns parser and grammar lifecycle:

- Tree-sitter grammar loading;
- reusable parser instances;
- bounded concurrency;
- syntax errors and recovery status;
- source encoding handling;
- per-language coverage metadata;
- analyzer versioning and timing.

Parsing success, partial recovery, failure, and unsupported language are distinct outcomes.

### Language adapters

Translate language-specific syntax into a universal IR. Each adapter defines:

- supported file patterns;
- symbol extraction and ownership rules;
- import/export syntax;
- module-resolution integration;
- call/reference extraction capabilities;
- fixtures, conformance tests, and declared limitations.

A grammar alone does not constitute language support.

### Symbol table and identity service

Maintains entity identities within and across analyses:

- file, module, namespace, class, method, function, and other symbol identities;
- lexical and declaring ownership;
- definitions and references;
- stable keys independent of incidental line movement where possible;
- ambiguity sets when a unique target is not provable.

### Import and reference resolvers

Implement language- and project-aware resolution:

- relative and absolute modules;
- aliases and configuration files;
- package and workspace boundaries;
- project/module roots;
- external versus internal dependencies;
- zero, one, or multiple candidate targets;
- explicit failure and ambiguity reasons.

Configuration and manifests are dependency roots: changing them may invalidate many resolution results.

### Fact graph builder

Converts normalized analysis into versioned entities, relations, and evidence. It should:

- enforce schema invariants;
- deduplicate by stable identity;
- reject dangling or invalid relationships;
- preserve unsupported and unresolved outcomes;
- maintain revision and analyzer lineage;
- publish changes transactionally.

### Persistent graph store

Use SQLite initially. Store normalized facts rather than one opaque JSON document as the source of truth. Candidate tables include repositories, revisions, artifacts, symbols, relations, evidence, analyzer runs, fingerprints, semantic concepts, concept memberships, views, and learning state.

Graph algorithms can operate on compact in-memory adjacency structures derived from persisted rows. A graph database should be introduced only when measurements show SQLite plus derived structures cannot satisfy actual workloads.

### Incremental engine

Owns classification, invalidation, recomputation, reconciliation, and publication. See [07-incremental-correctness.md](07-incremental-correctness.md).

### Graph algorithms

Provide deterministic signals such as:

- fan-in and fan-out;
- centrality and bridge scores;
- strongly connected components;
- community detection;
- dependency direction and boundary crossings;
- entry-point proximity;
- reachability and critical paths;
- cohesion and coupling;
- change-impact candidates.

Algorithms surface evidence. They do not assign semantic truth by themselves.

### Significance engine

Ranks artifacts and symbols using measurable factors:

- incoming and outgoing dependencies;
- public/exported status;
- proximity to entry points;
- flow participation;
- boundary crossings;
- persistence, network, filesystem, or other I/O;
- state mutation;
- architectural centrality;
- test and documentation coverage signals.

It must expose the factors behind every score.

### Flow engine

Builds evidence-backed candidates for HTTP, CLI, event, job, data, audio, and other execution flows. It combines entry points, calls/references, middleware, schemas, queues, services, configuration, storage access, and semantic interpretation.

Static flows are necessarily incomplete for many languages and frameworks. Candidate paths must retain evidence and uncertainty.

### Query and search engine

Combines multiple retrieval modes:

- exact identifier and path lookup;
- lexical/fuzzy search;
- structural filters and relationships;
- graph traversal and path search;
- semantic search when embeddings exist;
- projections by abstraction level, subsystem, flow, revision, confidence, or evidence source.

### Semantic engine

Consumes fact-graph projections and emits inferred concepts or explanations. Responsibilities may include:

- labeling structural clusters;
- subsystem and capability proposals;
- execution-flow interpretation;
- summaries and explanations;
- guided-learning narratives;
- semantic relationships and embeddings.

All output is validated, versioned, attributable to a provider/model/prompt, and stored separately from facts.

### Client application

The client renders query projections rather than loading the entire graph. It owns:

- semantic zoom and navigation history;
- breadcrumbs and `you are here` context;
- progressive result updates;
- evidence and uncertainty presentation;
- graph aggregation and lazy layout;
- code/source display;
- filtering, path tracing, and change overlays;
- personal learning state and user-created views.

## Runtime deployment for the initial product

The expected first deployment is a local process:

```text
rootline CLI or desktop shell
  -> starts/connects to local Rust engine
  -> indexes a user-selected repository
  -> persists data in a repository-specific or user-managed local store
  -> serves authenticated local HTTP/IPC APIs
  -> opens the TypeScript client
```

The exact desktop wrapper is intentionally undecided. The engine-client protocol should not depend on a particular wrapper.

## Dependency direction

```text
UI -> protocol/client SDK -> engine APIs
semantic workers -> query projections -> fact graph
engine orchestration -> adapters/parsers/resolvers
adapters -> Code IR contracts
persistence implementations -> storage interfaces
```

Core IR and provenance types should not depend on UI libraries, LLM providers, or a specific database API.

## Architectural constraints to test early

- Can partial facts stream without exposing inconsistent graph states?
- Which identities survive renames and moves, and which require history-aware mapping?
- Can SQLite transactions and indexes support expected local workloads?
- How will Rust Tree-sitter grammar packaging work across target platforms?
- What should run in-process versus worker threads or child processes?
- What is the minimal protocol that supports progressive projections and evidence inspection?
- How are model-generated results cached and invalidated independently from facts?

These are hypotheses for spikes and ADRs, not assumptions to bury in implementation.
