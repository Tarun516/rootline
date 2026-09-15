# ADR 0003: Analysis outcomes, typed errors, and observability

- Status: Accepted
- Date: 2026-09-15
- Owners: Rootline project
- Supersedes: None
- Superseded by: None

## Context

Static analysis can complete while a target remains ambiguous, unknown, or unsupported. Operational failures require different recovery. Opaque errors and unstructured logs would hide coverage and complicate incremental publication.

## Decision drivers

- Preserve trust through explicit uncertainty.
- Let callers recover from meaningful subsystem failures.
- Give users stable diagnostic codes without leaking internals.
- Measure indexing stages without logging source content.

## Considered options

One catch-all error type would simplify signatures but erase subsystem meaning. Treating unresolved results as exceptions would confuse expected analysis limits with engine faults. Plain printed messages would be difficult to connect to operation and revision context.

## Decision

Subsystems own typed operational errors. Completed analysis returns explicit outcomes such as resolved, ambiguous, unknown, and unsupported. CLI/server composition may use context-rich general errors where variant matching is not needed. Stable diagnostics carry code, severity, component, safe message, location, and guidance. Protocol errors are safe public envelopes. Structured tracing records operation spans and counters, with no source content logged by default. Configuration is read at application boundaries and passed explicitly.

## Consequences

Contracts and tests must cover both error and outcome paths. Diagnostics require a code registry and stable mapping. Logs gain useful causality and performance data while retaining privacy boundaries.

## Validation

Tests must distinguish successful-but-unknown resolution from resolver failure, verify public errors do not expose internal paths/backtraces/source text, and confirm tracing emits stage counters without source content.

## Revisit triggers

An observed domain where an outcome category is insufficient, diagnostic-code collisions, or a measured observability gap.

## References

- [Errors and diagnostics](../15-errors-and-diagnostics.md)
- [Configuration, security, and portability](../16-configuration-security-portability.md)
