# ADR 0004: Code Intelligence IR schema and versioning

- Status: Accepted
- Date: 2026-09-15
- Owners: Rootline project
- Supersedes: None
- Superseded by: None

## Context

Python and TypeScript adapters must produce one stable shape for symbols, imports, source ranges, and per-file outcomes. Parser-specific types (Tree-sitter nodes, grammar bindings) cannot cross into graph, storage, or protocol code without coupling every consumer to a parser implementation.

## Decision drivers

- Keep a single owned IR behind adapter boundaries.
- Make source coordinates, identities, and analysis outcomes unmixable types.
- Distinguish operational failure from successful-but-partial analysis.
- Version analyzer output for provenance and invalidation.

## Considered options

Per-language structs would duplicate graph and storage contracts. Bare string/integer maps would be flexible but erase coordinate systems and identity rules. A protobuf/IDL schema would add tooling before any consumer needs cross-process exchange.

## Decision

`rootline-core::ir` owns the IR with only `std` and `RepoPath` dependencies: `Language`, `ByteOffset`/`ByteRange` (start-inclusive, end-exclusive UTF-8 bytes), one-based `LineNumber`/`LineRange` evidence, `SymbolName`, `SymbolKind` (function, class, method), file-scoped `SymbolId` (path, kind, owner, name; line ranges excluded from identity), owner-validated `Symbol`, normalized per-module `ImportStatement`, `AnalysisStatus` (succeeded, partial, unsupported), `ParseError`, `AnalyzerInfo` (adapter name plus engine crate version), deterministic `ParsedModule`, and a publication-boundary `validate_module`. Revision identity joins at graph-build time, not in the IR.

## Consequences

Adapters convert parser types at their boundary and depend on IR contracts, never the reverse. Columns are absent until one unit is chosen across Tree-sitter, LSP, and editors. Python redefinitions are preserved by adapters and resolved at publication, since duplicate identities are legal source facts.

## Validation

Unit tests pin coordinate validation, owner rules, duplicate detection, and deterministic ordering; adapter conformance is fixture-backed with exact byte/line ranges.

## Revisit triggers

A second adapter forcing schema changes, column-unit requirements from the source viewer, or parameter/type extraction needs from resolution.

## References

- [Core model and provenance](../05-core-model-and-provenance.md)
- [Target architecture](../04-target-architecture.md)
- [Errors and diagnostics](../15-errors-and-diagnostics.md)
