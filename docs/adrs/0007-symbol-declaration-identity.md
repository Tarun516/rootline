# ADR 0007: Declaration identity for symbols

- Status: Accepted
- Date: 2026-09-15
- Owners: Rootline project
- Supersedes: 0004 (identity section only)
- Superseded by: None

## Context

ADR 0004 defined symbol identity as file, kind, owner, and declared name, and simultaneously required adapters to preserve legal Python redefinitions. Both cannot hold: two `def load` declarations in one scope share every field of that identity, and the publication check rejected the second one. Dotted owner strings (`Outer.Inner`) have the same flaw for nested scopes: a method of the second `class Model` cannot name its parent.

## Decision drivers

- Persistable identities must distinguish every source declaration.
- Ownership must be structural, not textual.
- Display names and cross-revision mapping are separate concerns with
  different stability requirements.

## Considered options

Byte offsets as identity components would be stable within a revision but contradict the documented rule that ranges are evidence, not identity. First-wins deduplication would silently drop the rebinding that Python actually executes. Overload-style signatures do not exist for Python and would not generalize.

## Decision

Three layers, modeled separately:

- declaration identity (`SymbolId`): file, kind, structural owner (the parent
  declaration's identity, boxed for recursion), declared name, and a
  declaration index counting same-name declarations in source order (0 for
  the first);
- semantic name: file, kind, owner names, and declared name without the
  index, via `scope_path()` / `full_path()` for display and lookup;
- cross-revision identity: explicitly future work, recorded here so no one
  mistakes declaration indices (which shift when declarations are added
  above) for stable cross-revision keys.

`validate_module` now collides only on full declaration identities, which
indicates an adapter defect rather than a legal program. Containment edges
derive from structural owners directly, removing the parent-lookup map and
its silently skipped miss branch.

## Consequences

Adapters assign declaration indices in walk order; redefinitions validate
cleanly and address distinctly. Persistence may store declaration
identities, but must not treat the index as stable across revisions.
`NodeId` display keeps the dotted semantic name, so existing snapshots are
unaffected.

## Validation

Unit tests pin distinct identities for redefined names, structural owner
paths, and validation acceptance; the fixture package asserts unchanged
containment output; a redefinition fixture pair pins indices 0 and 1
end to end.

## Revisit triggers

Cross-revision identity design (rename/move tracking), or a language whose
declarations need more than source order to disambiguate.

## References

- [ADR 0004](0004-code-intelligence-ir.md)
- [Core model and provenance](../05-core-model-and-provenance.md)
