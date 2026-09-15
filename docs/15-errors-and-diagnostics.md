# Errors, analysis outcomes, and diagnostics

## Core distinction

An operation can complete successfully while determining that code intelligence is ambiguous, unknown, or unsupported. That is an analysis outcome, not necessarily an operational error.

```text
operation
├── completed → resolved | ambiguous | unknown | unsupported
└── failed    → typed operational error
```

For example, a resolver contract can return `Result<Resolution<T>, ResolverError>`. `Resolution::Unknown` means the resolver worked but available evidence was insufficient. `ResolverError` means it could not perform its job.

## Subsystem-owned errors

Scanning, parsing, resolution, storage, engine orchestration, and transport each own meaningful typed errors. Do not create one bottom-level `RootlineError` containing every possible failure. Library callers must be able to match failures that affect recovery or publication.

Illustrative categories:

- scanner: missing repository, permission denial, boundary violation, I/O failure;
- parser: unavailable grammar, encoding failure, parser failure;
- resolver: configuration read/parse failure, internal invariant failure;
- store: open, migration, transaction, corruption;
- protocol/server: malformed request, authentication failure, incompatible version.

Dedicated typed errors may use a derive helper. Application boundaries such as CLI startup or server bootstrap may use context-rich general error composition when callers do not need to match variants. Do not return a general opaque error from a public scanner or resolver contract that requires typed recovery.

## Analysis statuses

The domain model must distinguish succeeded, partial, ambiguous, unknown, unsupported, failed, cancelled, and stale. A parser returning an empty structure is not proof that a file has no declarations unless its coverage and success state establish that claim.

Resolution outcomes should retain candidate targets and reasons:

```text
resolved(target)
ambiguous(candidates, reason)
unknown(reason)
unsupported(language or feature)
```

Operational errors are mapped separately.

## Structured diagnostics

Diagnostics are stable, searchable product data—not only log strings. A diagnostic should carry a code, severity, component, safe message, repository/revision, optional artifact and source range, and help or recovery guidance.

Example shape:

```text
Diagnostic
  code: RL-PY-RESOLVE-003
  severity: warning
  component: python-resolver
  message: ambiguous module target
  artifact: importer path
  range: import source range
  candidates: two repository artifacts
  help: inspect package roots or configuration
```

Codes support the UI, logs, tests, issue reports, and documentation. Diagnostic text should avoid exposing source content or unrelated absolute paths by default.

## Public protocol errors

Never send raw Rust error chains, backtraces, database errors, or filesystem internals to the browser. Map internal failures to stable public envelopes such as code, safe message, and request ID. Restricted logs keep the detailed cause.

Protocol version mismatches and authentication failures have distinct codes. A user-facing analysis diagnostic is not conflated with an HTTP 500 response.

## Observability

Use structured tracing and spans instead of scattered prints. Track the causal chain of indexing:

```text
index_repository
├── scan_repository      files, bytes, elapsed time
├── parse_files          files, symbols, failures, elapsed time
├── resolve_imports      resolved, ambiguous, unknown, elapsed time
├── publish_graph        nodes, relations, revision, elapsed time
└── queue_enrichment     tasks, cache hits
```

Do not log source code by default. Logs should be useful for profiling without leaking repository content. Record operation IDs and revision identity so partial, cancelled, stale, and failed runs can be diagnosed.

## Testing requirements

Tests should verify typed error recovery, analysis-status distinctions, stable diagnostic codes, safe protocol mapping, and that public responses do not expose raw paths, SQL, backtraces, or source text. Incremental publication failures must preserve the previous valid graph.
