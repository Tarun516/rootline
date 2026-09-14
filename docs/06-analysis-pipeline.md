# Analysis pipeline

## Pipeline goals

Rootline's pipeline must be deterministic at its base, progressively useful, observable, cancellable, and safe to update incrementally. LLM work is optional enrichment scheduled by the engine rather than orchestration glue.

## Full analysis path

```text
repository
  -> safe inventory and revision snapshot
  -> manifest/config discovery
  -> language detection and parser planning
  -> deterministic parsing into Code IR
  -> import/module resolution
  -> symbol/reference/call resolution
  -> fact graph construction and validation
  -> initial persistence/publication
  -> graph signals, importance, and cluster candidates
  -> optional semantic enrichment
  -> system concepts, flow candidates, and learning guidance
  -> projection and UI updates throughout
```

## Stage 1: repository snapshot and inventory

Inputs:

- repository root;
- revision or dirty-worktree policy;
- ignore configuration;
- analysis configuration.

Outputs:

- repository/revision identity;
- normalized artifact inventory;
- language and manifest candidates;
- content hashes;
- scanner diagnostics and timing.

The inventory becomes useful immediately in the UI and forms the boundary for safe file access.

## Stage 2: manifest and project-context analysis

Read relevant project metadata such as package manifests, workspace configuration, compiler configuration, module definitions, and build files. Produce typed configuration facts and resolution contexts rather than only a prose description.

Configuration inputs must be hashed and tracked as dependency roots because changing them can invalidate many imports or symbols.

## Stage 3: deterministic structure extraction

For each supported file:

1. select a language adapter;
2. parse once where possible;
3. record parse status and coverage;
4. extract declarations, ownership, imports/exports, and source ranges;
5. extract call/reference candidates supported by the adapter;
6. translate into Code IR;
7. validate adapter output.

For unsupported files, preserve the artifact and its unsupported state. Non-code adapters progressively add configurations, schemas, endpoints, services, resources, pipelines, and data definitions.

## Stage 4: resolution

Resolution transforms syntax-level references into graph candidates:

```text
raw import/reference/call
  + importer context
  + workspace/module configuration
  + symbol table
  -> resolved target | candidate set | unresolved reason
```

Resolution is language-specific behind a shared result contract. External dependencies remain distinguishable from missing internal targets.

## Stage 5: fact graph construction

Create and validate:

- artifacts and symbols;
- containment;
- imports and exports;
- supported inheritance/implementation;
- references and calls;
- non-code relationships;
- evidence and analyzer lineage.

Publish a consistent fact-graph revision transactionally. The UI can begin structural navigation as soon as the first valid projection is available.

## Stage 6: deterministic graph analysis

Compute signals that help the product prioritize and organize the graph:

- entry-point candidates;
- fan-in/fan-out;
- centrality and bridge nodes;
- strongly connected components;
- community candidates;
- directory and package cohesion;
- cross-boundary dependencies;
- critical or representative paths;
- initial significance scores.

Each score keeps its contributing signals.

## Stage 7: semantic enrichment

Semantic workers receive bounded, evidence-rich graph projections rather than arbitrary repository chunks. Possible work units include a cohesive module community, a candidate subsystem, a flow path, or a selected explanation request.

Outputs may include:

- summaries and purpose statements;
- cluster labels;
- capability and subsystem proposals;
- flow interpretations;
- semantic relationships;
- pedagogical explanations and prerequisites;
- embeddings.

Before persistence, outputs are schema-validated, normalized, checked against allowed references, and marked as inferences with model lineage.

## Progressive publication

The UI should receive stage updates through a stable event or subscription contract:

```text
inventory-ready
structure-partition-ready
resolution-progress
fact-graph-ready
signals-ready
semantic-concept-ready
analysis-complete
analysis-warning or analysis-failed
```

Progressive publication must not expose half-applied database transactions. Projections should identify the graph revision and completeness state they represent.

## Scheduling and batching

Deterministic work should prioritize artifacts likely to accelerate understanding:

- manifests and entry points;
- high-level packages/modules;
- files with high dependency connectivity;
- user-selected targets;
- current viewport or query needs.

Semantic batching can use dependency communities, directory cohesion, shared symbols, and runtime boundaries. Small isolated tasks may be pooled to avoid scheduling overhead. User-requested explanations can preempt background enrichment.

## Error model

Each stage reports structured outcomes:

- success;
- partial success with coverage;
- unsupported;
- ambiguous;
- failed with retry classification;
- cancelled;
- stale because the repository changed during analysis.

A single file failure should not normally abort the repository. Failures that compromise revision identity, storage consistency, or publication invariants should stop publication.

## Reproducibility and caching

Deterministic cache keys should include content, relevant configuration, adapter/analyzer versions, and repository context where resolution depends on it.

Semantic cache keys should include:

- the exact fact projection or stable digest;
- prompt/template version;
- model/provider identity and parameters;
- output-language configuration;
- semantic task type.

Changing a model should not invalidate deterministic facts. Changing a parser or resolver may invalidate only outputs within its declared coverage.

## Security and privacy considerations

- Default to local processing for repository content.
- Restrict file access to the indexed repository boundary.
- Require explicit configuration before sending source-derived context to external model providers.
- Show which data a semantic task will transmit.
- Avoid persisting unrelated absolute paths or credentials.
- Treat repository files, generated text, and model output as untrusted input.
