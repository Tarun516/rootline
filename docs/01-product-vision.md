# Product vision

## Working thesis

Rootline is a local-first, high-performance software comprehension engine that progressively reconstructs a codebase into a navigable mental model—from product architecture and execution flows down to individual symbols and source lines—while preserving evidence, global context, and the user's learning state.

## The user problem

The difficult part of joining, revisiting, debugging, or changing a codebase is not locating text. It is constructing and maintaining a stable mental model:

- What does this system do?
- Where does execution begin?
- What are its major subsystems and capabilities?
- How does a request, event, command, or data item move through it?
- Which components are important, and why?
- What depends on a component and what may be affected if it changes?
- How do infrastructure, deployment, configuration, schemas, and data stores connect to source code?
- Where does a local implementation detail fit in the global system?
- What changed since the previous revision, and how should the mental model change?
- What should a developer learn before or after the current topic?

File explorers and text search expose artifacts. They do not directly construct this model. Rootline should make that model an inspectable, queryable, and continuously maintained product object.

## Product promise

A developer should be able to open a repository and begin learning within seconds. As analysis progresses, Rootline should move from physical facts to deeper system understanding without forcing the user to wait for the final semantic result.

Illustrative progressive availability:

```text
T+0        Repository opened
T+0.5s     Files, languages, manifests, and repository metadata
T+1-2s     Physical and module structure
T+2-4s     Symbols begin appearing
T+3-6s     Import and dependency graph becomes useful
T+6-12s    Initial architecture candidates and importance signals
T+later    Semantic labels, explanations, and inferred flows continue enriching
```

These are goals to measure, not promises detached from repository size and hardware.

## North-star outcome

Optimize **time to understanding**, not only time to complete analysis.

A faster full index is useful, but the more meaningful measurement is how quickly a user can accurately answer a real question about the system.

## Primary user journeys

### Orientation

1. Open an unfamiliar repository.
2. See the system purpose and major subsystems.
3. Identify true entry points and important paths.
4. Drill into one capability without losing global context.
5. Reach the relevant source with an evidence-backed explanation.

### Flow tracing

1. Select or ask about a request, event, command, or data transformation.
2. View the critical path across code, contracts, services, storage, and infrastructure.
3. Inspect uncertainty and evidence at each transition.

### Change comprehension

1. Compare repository revisions or inspect a working change.
2. Identify directly changed entities and affected relationships.
3. Update only the necessary parts of the model.
4. Explain which architectural or learning-model concepts changed.

### Guided learning

1. Track explored and unexplored concepts.
2. See prerequisites for the selected topic.
3. Follow an adaptive learning sequence grounded in graph topology and system structure.
4. Preserve learning state across sessions and repository revisions.

## Product differentiation

Rootline's differentiation is the combination of:

- system-first navigation rather than implementation-first navigation;
- semantic zoom that changes abstraction, not merely node size;
- deterministic code intelligence that remains useful without an LLM;
- provenance-aware truth and explicit uncertainty;
- progressive results and incremental maintenance;
- persistent personal learning state and user-created views;
- evidence-backed execution and data-flow reconstruction.

## Long-term direction

Once the observed model is trustworthy, Rootline may support a separate hypothetical design model. Users could explore architectural changes and their consequences while keeping observed truth and simulation clearly separated. This is a later capability, not part of the initial product.
