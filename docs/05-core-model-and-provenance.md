# Core model and provenance

## Model objectives

The core model must represent physical repository artifacts, program structure, relationships, higher-order system concepts, supporting evidence, analysis uncertainty, and user interpretation without confusing their authority.

## Three truth layers

```text
SOURCE / FACT GRAPH
parser, resolver, schema, config, and repository facts
              |
              v
INFERRED SYSTEM MODEL
subsystems, capabilities, flows, labels, explanations
              |
              v
USER VIEW
personal groups, layout, annotations, hidden/pinned state
```

### Fact graph

Facts are reproducible outputs from a declared analyzer at a repository revision. A deterministic origin does not imply omniscience; it means the result follows a reproducible procedure with defined coverage.

Examples:

- a file exists at a revision;
- a Python function declaration occupies a source range;
- a TypeScript file imports another file through a resolved alias;
- a call expression has one proven target;
- a Docker Compose service exposes a port.

### Inferred model

Inferences are regeneratable interpretations based on facts and supporting signals.

Examples:

- a group of modules forms an authentication subsystem;
- a symbol is a critical architectural bridge;
- a series of calls and storage accesses represents an upload flow;
- one concept should be learned before another.

### User view

User state changes presentation or personal interpretation without mutating source truth.

Examples:

- a custom group named `Performance path`;
- pinned nodes or saved layouts;
- personal annotations;
- hidden tests;
- explored/understood markers.

## Conceptual entities

| Entity | Purpose | Examples |
| --- | --- | --- |
| Repository | An indexed repository identity | local checkout, remote origin identity |
| Revision | Source state used for analysis | Git commit, dirty-worktree snapshot |
| Artifact | Physical repository object | file, config, schema, Dockerfile |
| Symbol | Language-level program entity | function, class, method, type |
| Relation | Connection between entities | contains, imports, calls, reads, deploys |
| Evidence | Why an entity/relation/assertion exists | source range, resolver trace, heuristic signals |
| Analyzer run | Reproducibility and lifecycle record | scanner v1 at revision X |
| System concept | Higher-order model entity | subsystem, capability, domain, flow |
| Membership | Evidence-backed assignment to a concept | module belongs to audio preprocessing |
| View | Presentation-only grouping and state | saved perspective, filters, layout |
| Learning state | User comprehension history | seen, explored, understood, prerequisite |

## Identity

Identity should be stable enough for incremental maintenance but honest about ambiguity.

Conceptual source identity:

```text
repository + revision scope + artifact path + symbol kind + owner + declared name
```

Line ranges are evidence and disambiguation within a revision, not stable cross-revision identity. Renames, moves, overloads, generated declarations, dynamic installers, and anonymous constructs require explicit strategies.

The model must not infer class ownership from textual naming or opaque IDs alone. Ownership should come from adapter output and lexical scope evidence.

## Relationship model

Relationships should cover structural, behavioral, data, configuration, infrastructure, semantic, and pedagogical connections. Initial deterministic priorities are:

- `contains`;
- `imports` and `exports`;
- `defines` and `references`;
- `calls` where a target is credible;
- `inherits` and `implements` where supported.

Later families include:

- data reads, writes, transformations, and validation;
- routes, middleware, publishes, subscribes, and triggers;
- schema definition and migration;
- deploys, serves, configures, and provisions;
- system-concept membership and flow sequencing;
- learning prerequisites and recommendations.

An edge type should define direction, allowed endpoint types, confidence semantics, and evidence requirements.

## Evidence model

Every material assertion should carry evidence comparable to:

```text
RelationEvidence
  origin
  confidence
  source locations
  supporting signals
  analyzer identity and version
  repository revision
  status and limitations
```

### Origins

- scanner;
- parser;
- import resolver;
- reference/call resolver;
- config, schema, data, or infrastructure parser;
- graph algorithm;
- runtime observation, if introduced later;
- semantic model;
- user.

### Confidence

Confidence must not collapse different meanings into one unexplained floating-point value.

Recommended conceptual states:

- `deterministic`: produced under a supported deterministic contract;
- `probable`: strong but incomplete evidence;
- `possible`: plausible candidate requiring caution;
- `ambiguous`: multiple unresolved candidates;
- `unknown`: insufficient evidence;
- `unsupported`: no capable analyzer;
- `failed`: analyzer was applicable but could not complete.

Numeric scores may supplement these states for ranking but should not replace their semantics.

### Example distinctions

```text
CALLS
origin: reference resolver
status: deterministic
evidence: src/api.ts:74, resolved target src/service.ts:31
```

```text
PART_OF_SUBSYSTEM
origin: graph clustering + semantic labeler
status: probable
score: 0.82
signals: import density, shared types, naming, directory cohesion
```

```text
GROUPED_WITH
origin: user
status: authoritative for this view only
```

## Analysis lineage

Every persisted output should be attributable to:

- repository identity and source revision;
- dirty-worktree snapshot identity where applicable;
- analyzer component and version;
- adapter configuration and relevant project configuration hashes;
- model provider, model, prompt/template version, and parameters for semantic work;
- creation time and superseded-by relationship;
- evidence inputs used to derive it.

This enables reproducibility, selective invalidation, evaluator comparisons, and debugging.

## Persistence sketch

SQLite is the initial target. A normalized conceptual schema may include:

```text
repositories
revisions
artifacts
symbols
relations
evidence
relation_evidence
analyzer_runs
fingerprints
system_concepts
concept_memberships
semantic_runs
views
view_memberships
learning_state
```

This is not a final SQL schema. It should be validated through the first vertical slice and recorded through an ADR before becoming durable.

## Invariants

- No published relation references a missing endpoint.
- Facts and inferences cannot share indistinguishable storage semantics.
- User views cannot mutate fact or inference records.
- A deleted source entity is removed only with adequate evidence or marked unresolved pending repair.
- Repository-relative paths are stored without leaking unrelated absolute filesystem structure.
- Analyzer failures are observable.
- Every semantic result is invalidatable independently of deterministic facts.
- Schema validation occurs at ingestion and publication boundaries.
