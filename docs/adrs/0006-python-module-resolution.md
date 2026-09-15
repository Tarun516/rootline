# ADR 0006: Python module resolution scope

- Status: Accepted
- Date: 2026-09-15
- Owners: Rootline project
- Supersedes: None
- Superseded by: None

## Context

Import edges require mapping `import` / `from ... import` syntax to repository files. Python's import system (absolute and explicit relative imports, packages, namespace packages, `sys.path` composition, dynamic imports) is larger than the first graph slice can cover, so the supported subset and every out-of-scope outcome need a recorded boundary.

## Decision drivers

- Resolve the common static cases with measured precision.
- Keep ambiguous, unknown, and external outcomes visible instead of erroring.
- Resolve without filesystem access so tests control the file set exactly.
- Never promote a probable guess to a deterministic edge.

## Considered options

Shelling out to the interpreter (`importlib`, `sys.path` probing) would give authoritative answers but couples analysis to the host environment, breaks hermetic benchmarks, and executes repository code paths indirectly. A full reimplementation of `sys.path` semantics (`.pth` files, zip imports, meta-path finders) would precede any graph value. The chosen middle path resolves lexical static imports against an explicit file set and source roots, and records everything else as data.

## Decision

`rootline-engine::resolve` maps imports over a caller-supplied `ModuleIndex` (analyzed files plus source roots; the repository top is always probed so flat layouts work without configuration):

- absolute imports probe `<root>/a/b.py` and `<root>/a/b/__init__.py` under every root: one hit resolves, several report `Ambiguous` with all candidates (no silent shadowing), none reports `External`;
- relative imports anchor at the importing file's directory and climb per leading dot: missing targets and climbs past the package top report `Unknown` (relative imports are repo-internal by construction, so they never report `External`);
- intermediate directories need no `__init__.py`, but the resolved endpoint must be a module file;
- `from pkg import name` additionally distinguishes names defined in the package file, submodule files, and external packages; anything else is an import diagnostic;
- resolution is a pure, infallible function returning `Resolution`; there is no `ResolverError` yet because no operational failure mode exists at this boundary (no I/O, no threads, validated inputs).

Call and inheritance reasoning (in `rootline-engine::graph`) resolves only statically credible targets — same-scope definitions, `self`/`cls` methods, and names traced through import bindings — with language builtins skipped by design and every other case kept as a `GraphDiagnostic`. External identity is always a module path, never module-plus-attribute.

## Consequences

A partially analyzed repository may classify a missing-internal module as external; the classification stays visible in graph diagnostics until explicit repository-boundary tracking lands. Builtin base classes (`Exception`) report as unresolved bases, dynamic imports (`importlib`, `__import__`) are not extracted, and `rootline graph` CLI inspection waits for tested root discovery.

## Validation

Nine resolver unit tests pin absolute, relative, package, submodule, ambiguity, escape, and empty-root behavior; eight graph tests pin a 23-node, 34-relation fixture package edge-for-edge plus every diagnostic; `Graph::validate` rejects dangling endpoints at publication.

## Revisit triggers

Measured false-definite resolutions on the R0 corpus, stdlib/third-party classification needs, dynamic-import coverage demands, or a second language forcing a shared resolution contract.

## References

- [Core model and provenance](../05-core-model-and-provenance.md)
- [Analysis pipeline](../06-analysis-pipeline.md)
- [Benchmark and evaluation](../11-benchmark-and-evaluation.md)
