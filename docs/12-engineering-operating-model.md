# Engineering operating model

## The roadmap is disposable; principles are not

Rootline should be willing to replace storage, libraries, algorithms, UI frameworks, graph representations, and model providers when evidence supports the change. Product invariants—trust, progressive understanding, context preservation, fact/inference separation, local-first behavior, and explicit uncertainty—should remain stable.

## Build vertical slices

Keep an end-to-end path alive:

```text
repository
  -> scan
  -> parse one supported language
  -> normalize symbol
  -> persist fact and evidence
  -> query projection
  -> display in UI
```

Deepen and widen this slice rather than building isolated infrastructure for months. A narrow working product exposes contract mistakes earlier than disconnected subsystems.

## Run learning spikes

Before integrating a consequential unfamiliar concept:

```text
learn the mental model
  -> build a small isolated experiment
  -> observe limitations and failure modes
  -> document findings
  -> decide through an ADR
  -> integrate
```

Likely spike topics include Tree-sitter, AST/CST distinctions, scopes, symbol tables, name resolution, call graphs, LSPs, control/data-flow analysis, clustering, incremental computation, graph layout, SQLite indexing, and retrieval.

## Do not use AI to hide missing code intelligence

If a reference cannot be resolved deterministically, store the unresolved state. A model may produce a separately labeled likely explanation, but it cannot rewrite the fact as resolved.

This rule applies equally to imports, calls, architecture boundaries, flows, and change impact.

## Measure from the first phase

Each subsystem should emit correctness, coverage, runtime, I/O, and memory measurements. Optimization begins with a reproducible profile and an explicit budget.

Avoid premature complexity such as custom allocators, memory mapping, distributed execution, specialized graph storage, or ANN indexes without benchmark evidence.

## Architecture decision records

Record consequential decisions, including:

- why Rust and where it is used;
- Tree-sitter library and grammar packaging;
- Code IR boundaries and versioning;
- SQLite schema and migration strategy;
- symbol identity and move/rename policy;
- resolver semantics;
- incremental invalidation and publication;
- engine/client protocol;
- graph projection and layout;
- semantic provider boundary and cache keys.

Use [adrs/README.md](adrs/README.md) and [adrs/template.md](adrs/template.md).

## Engineering learning notebook

Maintain technical notes that explain concepts, experiments, and failure cases independently from product-facing documentation. See [engineering/README.md](engineering/README.md).

## Definition of done for a capability

A capability is not complete until it has:

- an explicit contract and scope;
- fixtures representing normal and adversarial cases;
- correctness tests;
- declared limitations and unsupported states;
- structured diagnostics;
- evidence and analyzer lineage;
- performance measurement;
- incremental behavior where applicable;
- documentation and an ADR when the choice is consequential;
- integration into the working vertical slice.

## Language support policy

A language is supported only when the intended support tier is explicit and tested.

A credible structural tier may require:

- files and grammar load correctly;
- functions, classes, methods, types, and ownership as applicable;
- imports/exports;
- source ranges;
- fixtures and coverage metrics.

A credible code-intelligence tier additionally requires:

- language/project-aware module resolution;
- references/calls at a documented precision level;
- inheritance/implementation where applicable;
- incremental identity and deletion evidence;
- limitations documented end to end.

Do not advertise complete language support solely because a grammar exists.

## Failure and uncertainty policy

- Never silently drop analyzer errors.
- Do not convert `unsupported` into an empty successful result.
- Keep ambiguity sets when multiple targets are valid candidates.
- Preserve the last valid published graph when updates fail.
- Show partial coverage in the product.
- Prefer a smaller trusted graph over a visually complete fabricated one.

## Testing strategy

Use multiple layers:

- adapter unit fixtures;
- resolver conformance matrices;
- IR and schema property tests;
- graph invariant tests;
- incremental transition tests;
- full-versus-incremental convergence tests;
- golden repository snapshots;
- performance regression tests;
- UI projection and interaction tests;
- semantic evaluation separated from deterministic correctness.

## Review checkpoints

At each roadmap phase:

1. Demonstrate the capability on pinned repositories.
2. Review correctness and unknown/unsupported rates.
3. Review runtime and memory measurements.
4. Inspect whether the vertical slice became more useful.
5. Record architectural changes and new risks.
6. Adjust the next phase based on evidence.

## Documentation rules

- Use Rootline consistently as the project name.
- Label current implementation, committed design, hypothesis, and future idea distinctly.
- Link claims to tests, benchmarks, ADRs, or evidence where available.
- Avoid documenting unsupported behavior as planned certainty.
- Update architecture documents when stable boundaries change.
