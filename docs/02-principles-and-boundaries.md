# Principles and boundaries

## Non-negotiable principles

1. **Graphs should teach a system.** Node count and visual complexity are not success metrics.
2. **Fast to first understanding.** Useful partial results appear before full analysis completes.
3. **Never lose global context.** Drilling into a symbol must preserve the path back to capability, subsystem, and system.
4. **Facts, inference, and user interpretation are distinct.** They have different authority and lifecycle rules.
5. **Every important inference is explainable.** The user can inspect evidence and supporting signals.
6. **Basic code intelligence works without an LLM.** Parsing, resolution, containment, and supported call/reference facts do not depend on model guesses.
7. **Unknown is valid.** Unsupported, failed, ambiguous, and unresolved states remain visible.
8. **Incremental correctness is architectural.** It is designed into identity, persistence, provenance, and invalidation.
9. **Local-first behavior is the default.** Repository content and the core fact graph remain usable locally.
10. **Large repositories influence interfaces, not premature implementation.** Design stable boundaries now; add specialized storage and memory techniques when measurements justify them.
11. **The local single-user experience must be excellent.** Core usefulness cannot depend on future services or collaboration features.
12. **Deterministic work is reproducible.** The same revision, analyzer version, and configuration should yield equivalent facts.

## Product boundaries

Rootline is not primarily:

- a code-generation or autonomous coding agent;
- a general-purpose static analyzer or compiler replacement;
- a text or symbol search interface;
- a chat wrapper over repository files;
- a single enormous file/function graph;
- a visualization created for spectacle;
- a substitute for runtime tracing where static evidence is insufficient.

These capabilities may be integrated where they improve comprehension, but none defines the product.

## Decision hierarchy

When a roadmap item conflicts with a principle, preserve the principle and change the roadmap.

When a tool or library conflicts with a stable boundary, prefer replacing the tool rather than weakening the boundary.

When semantic inference conflicts with deterministic evidence:

- preserve the deterministic fact;
- retain the inference as a separately identified hypothesis if useful;
- record the disagreement;
- never silently promote the inference to fact.

When the analyzer lacks sufficient evidence:

- store `unknown`, `ambiguous`, `unsupported`, or `failed` as appropriate;
- explain the limitation;
- avoid manufacturing a definite edge for visual completeness.

## Scope discipline

The initial scope is repository comprehension for Python and TypeScript. It includes repository inventory, normalized symbols, module/import resolution, supported structural relationships, persistence, incremental updates, and a minimal navigable UI.

The following should not lead the first implementation:

- autonomous coding features;
- a complex conversational interface;
- many-language breadth;
- a hosted platform;
- distributed processing;
- a specialized graph database without evidence;
- architecture simulation;
- perfect visual polish before the deterministic model is useful;
- custom embedding or GPU infrastructure.

## Definitions of quality

### Trustworthy

Users can tell what Rootline knows, why it knows it, and where its knowledge ends.

### Useful

The model helps answer real comprehension questions faster than reading the repository conventionally.

### Progressive

Each completed stage improves the user's view without invalidating already presented facts.

### Maintainable

New languages, parsers, resolvers, semantic enrichers, and UI projections fit through explicit contracts.

### Measurable

Performance and correctness claims are evaluated on fixed repositories and revisions.
