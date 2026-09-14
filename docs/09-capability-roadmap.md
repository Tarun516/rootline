# Capability roadmap

## How to use this roadmap

This is a sequence of capability checkpoints, not a contractual calendar. Each phase ends when an observable invariant becomes true. Findings may split, reorder, or replace phases while the product principles remain stable.

## Phase 0 — Reference and benchmark harness

Build a fixed corpus, freeze repository revisions, define golden comprehension questions, and capture reproducible baseline behavior from relevant repository-comprehension tools.

Exit criterion: the same repository and revision can be evaluated repeatedly for runtime, memory, structural coverage, and answer quality.

## Phase 1 — Repository inventory

Build a CLI flow such as:

```text
rootline index <path>
```

Implement safe traversal, ignore rules, path normalization, symlink policy, language/manifest detection, content hashes, Git metadata, bounded parallel I/O, and structured diagnostics.

Exit criterion: a 100,000-file repository can be inventoried safely and repeatedly without unnecessary rereads or boundary escape.

## Phase 2 — Universal Code Intelligence IR

Support Python and TypeScript first. Normalize files, functions, classes, methods, imports, exports, owners, parameters, types where available, and source ranges through language adapters.

Exit criterion: the golden Python and TypeScript repositories produce stable symbol inventories and correct ranges across repeated runs.

## Phase 3 — Dependency resolution

Implement Python module resolution and TypeScript/JavaScript resolution, including relative imports, index/extension probing, `tsconfig` paths, and package/workspace boundaries. Record explicit external, ambiguous, unresolved, unsupported, and failed outcomes.

Exit criterion: import edges resolve to actual repository artifacts with measured precision and coverage; unresolved cases remain visible.

## Phase 4 — Symbol graph

Add containment, imports/exports, supported calls, references, inheritance, and implementation relations with evidence and provenance.

Exit criterion: `Who calls this?`, `What does this call?`, and `What depends on this?` work on supported cases without an LLM.

## Phase 5 — Persistence and incremental engine

Persist artifacts, symbols, relations, evidence, analyzer runs, and fingerprints in SQLite. Implement change detection, dependency-aware invalidation, graph repair, and transactional publication.

Exit criterion: changing one function reprocesses the smallest safe affected region; incremental and clean rebuild results converge.

## Phase 6 — First usable visual product

Build repository-to-module-to-file-to-symbol navigation, source display, breadcrumbs, `you are here`, basic search, analysis progress, and evidence status. Avoid advanced AI dependency.

Exit criterion: the golden repository is easier to reorient in through Rootline than through the editor's file explorer.

## Phase 7 — Significance engine

Compute entry-point proximity, fan-in/out, centrality, public/exported status, flow participation, boundary crossing, state mutation, and I/O signals. Explain each ranking.

Exit criterion: important files and symbols rank plausibly across the corpus, and their explanations cite measurable factors.

## Phase 8 — Architecture and subsystem inference

Combine community detection, cohesion/coupling, directory and naming signals, framework conventions, entry points, deployment topology, and semantic labeling.

Exit criterion: generated subsystem candidates are close to expert-drawn architecture on the golden repository, and disagreements are inspectable.

## Phase 9 — Semantic zoom

Represent system, subsystem, capability, flow, module, file, symbol, and source levels with stable transitions and context preservation.

Exit criterion: users can descend from system overview to a source line and return without losing orientation.

## Phase 10 — Execution and data flows

Infer HTTP, CLI, event, data, audio, and job flows using entry points, references/calls, middleware, contracts, configuration, services, and storage evidence.

Exit criterion: at least one important end-to-end flow per benchmark repository is reconstructed accurately with source evidence and visible uncertainty.

## Phase 11 — Guided learning

Track explored areas, prerequisites, learning sequences, and recommended next steps. Generate guidance from graph topology and the inferred system model.

Exit criterion: a new user can follow a coherent onboarding sequence without manually deciding which files to read first.

## Phase 12 — Contextual chat

Add graph-grounded questions and commands that can focus, filter, regroup, or trace the canvas.

Exit criterion: commands such as `trace authentication` or `show only the critical path` produce evidence-backed answers and useful visual transformations.

## Phase 13 — User-created views

Add grouping, pinning, collapsing, hiding, annotation, saved perspectives, and persistent personal mental models.

Exit criterion: users can create durable views while facts and inferred architecture remain separately reproducible.

## Phase 14 — Large-repository runtime

Profile increasingly large repositories and add compact IDs, adjacency representations, arenas, memory mapping, partitioning, parallel parsing, lazy loading, virtualized rendering, or ANN indexes only as evidence requires.

Exit criterion: agreed performance budgets are met at target scale without correctness regression.

## Phase 15 — Additional languages

Add languages through complete, tested adapters. Grammar availability alone is insufficient.

Exit criterion: a supported language has credible parsing, symbols, module resolution, relationships, fixtures, coverage reporting, and limitations documentation.

## Phase 16 — Design and simulation mode

Fork the observed system model into an explicitly hypothetical model. Allow architectural alternatives and impact exploration without changing source truth.

Exit criterion: observed and hypothetical entities remain clearly separated, changes are reversible, and impact explanations cite their assumptions.

## Cross-phase requirements

Every phase must add or preserve:

- measurable correctness and performance;
- structured diagnostics and coverage;
- evidence and analyzer lineage;
- explicit unsupported and unknown states;
- benchmark fixtures;
- relevant ADRs and engineering notes;
- a working vertical slice wherever feasible.
