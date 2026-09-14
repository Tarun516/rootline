# Incremental correctness

## Why this is foundational

Incremental analysis is not a cache optimization. It is the process of maintaining a valid, explainable graph while the repository, configuration, analyzers, and semantic models evolve.

A naive changed-file cache fails because one change can affect:

- import resolution in many files;
- symbol ownership and references;
- schema or generated-code relationships;
- architecture clusters and importance rankings;
- execution-flow candidates;
- semantic summaries and learning paths;
- stable identity across renames or moves.

## Required persisted baselines

The first persistent implementation should retain enough information to compare and repair safely:

- repository and revision identity;
- complete artifact inventory;
- content hashes;
- structural fingerprints;
- relevant config/manifest hashes;
- adapter and analyzer versions;
- symbol identities and ownership evidence;
- import/reference resolution inputs and outcomes;
- published fact-graph revision;
- independently versioned semantic outputs;
- previous architecture and learning projections where appropriate.

## Change classes

At minimum, classify each artifact as:

- unchanged;
- content-changed but structurally equivalent under the current analyzer;
- structurally changed;
- new;
- deleted;
- newly ignored or newly included;
- analyzer-invalidated because tooling or configuration changed;
- unknown because the comparison is not trustworthy.

The term `cosmetic` should be used cautiously. Internal logic can change behavior without changing declarations or imports. Such a change may be structurally equivalent but can still invalidate summaries, flow semantics, or runtime-risk information.

## Invalidation model

Invalidation should follow dependencies, not only file membership.

Examples:

- A function signature change invalidates its symbol fact, references, calls, summaries, and dependent flow explanations.
- A `tsconfig` path change invalidates resolution for importers in its scope.
- A `pyproject.toml` or package-layout change may invalidate Python module resolution.
- A schema change can invalidate generated clients, endpoints, data flows, and semantic concepts.
- A file move invalidates path identity but may preserve semantic identity if evidence supports a move mapping.
- An analyzer upgrade may invalidate all facts produced by that analyzer version while preserving unrelated facts.

The graph should support reverse dependency lookup for invalidation planning.

## Safe update sequence

```text
1. Snapshot repository/revision and configuration
2. Compare inventory and fingerprints
3. Build an invalidation plan
4. Analyze affected current artifacts
5. Construct a candidate graph revision
6. Reconcile identities and relationships
7. Validate preservation and deletion claims
8. Recompute affected global projections
9. Publish facts, fingerprints, and metadata atomically
10. Invalidate or refresh dependent semantic outputs separately
```

The previously published graph remains available if candidate publication fails.

## Symbol-loss safety

A symbol missing from new analyzer output is not automatically deleted.

Deletion is safe only when Rootline can establish that:

- the relevant language and syntax are supported by a strict analyzer;
- parsing completed without a state that compromises declaration coverage;
- the previous symbol identity is sufficiently known;
- the current source has no matching declaration under that identity;
- dynamic or generated declaration mechanisms do not create compatible uncertainty;
- the graph candidate has not merely omitted an existing source declaration.

Otherwise, mark the result unknown and preserve or quarantine the previous information according to an explicit policy. Never silently erase durable graph knowledge because of a transient parser or enrichment failure.

## Identity reconciliation

Cross-revision symbol identity should consider:

- artifact move/rename evidence from Git when available;
- symbol kind, name, and explicit owner;
- lexical scope;
- structural signature;
- surrounding stable declarations;
- source range only as within-revision evidence;
- ambiguity when multiple candidates match.

Opaque graph IDs and identical names are not enough. Same-named methods in different classes must remain distinct.

## Global recomputation

Some changes require more than local graph repair. Escalate recomputation when evidence indicates:

- directory/package boundaries changed;
- a resolution-root configuration changed;
- many structural entities changed;
- major entry points appeared or disappeared;
- community topology changed materially;
- system concepts lost significant evidence;
- analyzer schema or semantics changed.

Thresholds should be measured on the benchmark corpus rather than copied permanently from another project.

## Semantic invalidation

Semantic outputs use their own dependency graph. A source change should invalidate only the affected summaries, labels, flows, embeddings, and learning guidance when possible.

Semantic output must record the fact revision or fact projection digest it used. Stale semantic information can then be hidden, marked stale, or refreshed without blocking valid deterministic navigation.

## Correctness properties

Tests should assert:

- no dangling endpoints after publication;
- unchanged facts keep stable identities;
- genuine deletions are removed when proven;
- still-present symbols omitted by a candidate block or trigger repair;
- config-root changes invalidate dependent resolution;
- removed files cannot leave active outgoing relations;
- inbound relationships reconcile to valid replacement identities;
- a failed update does not advance graph, fingerprint, or revision metadata;
- repeated analysis of the same revision is idempotent;
- full and incremental analysis converge to equivalent deterministic graphs.

The last property is critical: incremental results should be compared against clean rebuilds in continuous testing.

## Observability

Every update should report:

- changed files by class;
- invalidation reasons;
- files and symbols reanalyzed;
- relationships added, removed, preserved, or left unresolved;
- global computations rerun or preserved;
- semantic tasks invalidated or queued;
- duration, CPU, memory, and I/O;
- publication success or retained previous revision.
