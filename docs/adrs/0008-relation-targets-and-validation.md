# ADR 0008: Relation targets, confidence separation, and publication validation

- Status: Accepted
- Date: 2026-09-15
- Owners: Rootline project
- Supersedes: 0004 (relation and confidence sections only)
- Superseded by: None

## Context

Relations carried `to` plus `candidates`, with ambiguity publishing its first candidate as the primary target under an `Ambiguous` confidence. Any consumer reading `to()` received a guess wearing analysis output. The confidence enum simultaneously mixed assertion strength (`probable`), resolution state (`ambiguous`), capability state (`unsupported`), and operational result (`failed`). And `Graph::new` built graphs that `validate()` might later reject, so invalid graphs were representable values.

## Decision drivers

- Ambiguity must be unrepresentable as resolution at the type level.
- Each concept from docs/05 and ADR 0003 needs exactly one home.
- Invalid graphs must fail at construction, before persistence exists.

## Considered options

Keeping `to` plus candidates with stronger documentation would rely on every future consumer reading the docs first — the review correctly identified this as the dangerous option. Splitting relations into separate resolved/ambiguous types would double every consumer match for little gain over one target enum.

## Decision

- `RelationTarget::{Resolved(NodeId), Ambiguous(Vec<NodeId>)}` replaces
  `to` plus `candidates`. There is no single-target accessor; consumers
  match. The constructor sorts, dedups, and rejects ambiguity with fewer
  than two distinct candidates.
- `Confidence::{Deterministic, Probable, Possible}` covers assertion
  strength only. Unknown and failed analyses are diagnostics or `Err`,
  never edges; unsupported files never produce relations.
- `Graph::try_new` validates endpoints against the node inventory and
  returns `Result`; no unchecked graph constructor remains.
- `Relation::new` additionally rejects empty evidence, as before.
- Nodes carry `NodeInfo` (declaration range where one exists, producing
  analyzer), and graphs carry `GraphMetadata` (opaque repository and
  revision strings — real identity types arrive with the persistence
  design, deliberately not invented here).

## Consequences

Builder call sites construct targets explicitly; ambiguous import, call,
and inheritance edges keep full candidate sets with deterministic
confidence. SQLite will persist `Resolved` versus `Ambiguous` distinctly
rather than decoding a confidence flag. External identity stays a module
path (never module-plus-attribute) until ecosystems force refinement.

## Validation

Unit tests pin target validation (evidence, candidate count, dangling
endpoints including candidates), and the fixture package asserts the full
35-relation graph plus diagnostics under the new model, including an
ambiguous-import candidate set with no primary.

## Revisit triggers

Persistence-schema pressure on external identity (multi-ecosystem
graphs), or inference stages needing `Probable`/`Possible` semantics
pinned down beyond ranking.

## References

- [ADR 0003](0003-outcomes-errors-and-observability.md)
- [ADR 0004](0004-code-intelligence-ir.md)
- [Core model and provenance](../05-core-model-and-provenance.md)
