# Risks and open questions

## Cross-language name and reference resolution

### Why it is hard

Parsing syntax is easier than identifying the actual target of a name. Module systems, build configuration, overloads, dynamic behavior, generated code, and runtime injection complicate resolution.

### Response

- build language-specific resolvers behind a shared outcome contract;
- use compiler or LSP metadata selectively where it improves evidence;
- retain candidate sets and unresolved states;
- measure precision and coverage independently;
- never use semantic guesses as deterministic facts.

### Open questions

- Which resolution tiers should Python and TypeScript support in the first 90 days?
- When should LSP/compiler integrations supplement Tree-sitter?
- How should cross-language service calls be represented before runtime evidence exists?

## Dynamic languages and runtime wiring

### Why it is hard

Reflection, monkey patching, decorators, dependency injection, dynamic imports, and configuration-driven registration can defeat complete static analysis.

### Response

- model partial deterministic facts;
- distinguish probable edges from resolved edges;
- consider runtime traces as a later evidence source;
- document framework-specific limitations.

## Call-graph explosion

### Why it is hard

Over-approximation creates noisy graphs; under-approximation hides important flows.

### Response

- retain evidence and resolution status;
- build task-specific projections;
- support confidence/coverage filters;
- evaluate important flows rather than maximizing edge count.

## Stable identity across revisions

### Why it is hard

Symbols move, rename, overload, change owners, or are generated dynamically. Paths and line numbers are insufficient stable identities.

### Response

- combine Git move evidence, owner, kind, name, signature, and structural context;
- expose ambiguity;
- test transition matrices;
- separate within-revision identity from cross-revision mapping.

### Open questions

- Which identities must remain stable for user learning state and saved views?
- When should user state follow a probable move versus wait for confirmation?

## Incremental invalidation

### Why it is hard

One configuration, schema, workspace, or analyzer change can invalidate a large area.

### Response

- treat manifests/configs as dependency roots;
- maintain reverse dependencies;
- compare incremental output with clean rebuilds;
- use conservative escalation when proof is weak;
- publish transactionally.

## Architecture labeling

### Why it is hard

Structural clusters do not necessarily match human conceptual architecture. Labels may be plausible but wrong.

### Response

- separate cluster discovery from labeling;
- show supporting signals and confidence;
- allow corrections in the user-view layer;
- evaluate against expert diagrams and golden questions;
- monitor stability across revisions.

## Flow reconstruction

### Why it is hard

Important transitions cross frameworks, configuration, queues, schemas, external services, and runtime dispatch.

### Response

- start with narrow, high-evidence flow families;
- combine static paths with typed non-code artifacts;
- preserve branches and unresolved transitions;
- add runtime evidence later without replacing static lineage.

## Semantic zoom and graph visualization

### Why it is hard

Large graphs cannot be rendered directly, and changing abstraction levels can destroy orientation. Layout instability reduces trust and learning.

### Response

- query projections rather than render storage;
- use hierarchical aggregation and lazy layout;
- preserve selection, breadcrumb path, and viewport intent;
- evaluate comprehension, not only layout time;
- treat semantic zoom as a research problem with prototypes.

### Open questions

- Which graph canvas or rendering strategy best supports future scale?
- How stable should layout be across repository revisions?
- What information belongs in each semantic level?

## Model cost, latency, and variability

### Why it is hard

Whole-repository semantic work can dominate runtime and produce non-reproducible results.

### Response

- keep enrichment optional and progressive;
- cache by exact facts, template, provider, and model;
- prioritize user-visible or high-value tasks;
- support local providers where practical;
- separate semantic benchmarks from deterministic engine performance.

## User trust

### Why it is hard

A polished incorrect architecture is worse than a visibly incomplete analysis.

### Response

- evidence inspector;
- distinct visual treatment for facts and inferences;
- visible coverage and freshness;
- correction and feedback paths;
- explicit unknown states.

## Scope expansion

### Why it is hard

Code intelligence, education, chat, agents, visualization, and architecture simulation can each become separate products.

### Response

Use the north-star workflow as a filter: does this help users understand an unfamiliar system faster and more accurately? Delay work that does not strengthen the current vertical slice or a measured user need.

## Storage and graph representation

### Open questions

- How far can indexed SQLite plus in-memory adjacency structures scale on target hardware?
- Which queries require precomputed projections?
- When do compact integer IDs, CSR structures, arenas, or memory mapping become worthwhile?
- How should graph-schema migrations preserve provenance and user state?

These should be answered through benchmarks rather than architectural fashion.

## Local API and application packaging

### Open questions

- Local HTTP, IPC, or a hybrid protocol?
- Desktop wrapper, browser-launched local app, or both?
- How should authentication and repository path allowlists work locally?
- How are long-running indexing tasks cancelled and resumed?
- How are grammar binaries packaged cross-platform?

## Privacy and source handling

### Risks

- accidental transmission of proprietary source to external providers;
- absolute path or credential leakage in persisted graph data;
- following links outside repository boundaries;
- treating prompt-like repository content as trusted instructions.

### Response

- local processing by default;
- explicit provider data boundaries;
- repository-scoped access controls;
- path sanitization and secret-aware handling;
- treat repository and model content as untrusted data.
