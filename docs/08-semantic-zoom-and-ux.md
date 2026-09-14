# Semantic zoom and user experience

## UX objective

Rootline should help the user build a mental model, not merely inspect a graph. The interface must preserve orientation while changing the represented abstraction.

## Semantic zoom levels

| Level | Primary object | Main question |
| --- | --- | --- |
| 0 | Product/system | What is this software? |
| 1 | Subsystem | What are the major areas? |
| 2 | Capability | What can each area do? |
| 3 | Execution/data flow | How does work move through the system? |
| 4 | Module/package | Where is the capability implemented? |
| 5 | File/artifact | Which artifact owns this behavior? |
| 6 | Symbol | Which function, class, method, or type matters? |
| 7 | Source | What exactly happens here? |

Zoom changes representation and available questions. It is not simply geometric scaling or hiding labels.

## Context preservation

Every detail view should answer `Where am I?`:

```text
Rootline
  -> Analysis engine
    -> Dependency resolution
      -> TypeScript resolver
        -> resolver module
          -> resolve_import()
            -> source lines
```

Required navigation aids include:

- semantic breadcrumbs;
- visible parent and sibling context;
- reversible drill-down;
- back/forward navigation history;
- stable selection across projection changes;
- minimap or contextual overview where useful;
- explanation of why the current node belongs at each higher level.

## Progressive interface states

The interface should clearly represent analysis completeness:

- inventory available;
- structure currently indexing;
- dependencies partially resolved;
- system concepts proposed but not complete;
- semantic enrichment queued or disabled;
- selected results stale because source changed;
- an analysis limitation affects this view.

New results should enrich the current context rather than repeatedly replacing the entire canvas.

## System-first default

The initial overview should prioritize:

- system purpose;
- major subsystems;
- important entry points;
- representative flows;
- external systems and persistent state;
- uncertainty and analysis coverage.

A directory tree remains useful, especially before semantic analysis finishes, but it is a navigation tool rather than the final mental model.

## Flow views

Flow views should reduce the graph to a meaningful path, for example:

```text
HTTP request
  -> middleware
  -> controller
  -> service
  -> repository
  -> database
```

or:

```text
audio upload
  -> decode
  -> normalize
  -> resample
  -> tensorize
  -> batch
```

Each step should show:

- its system-level meaning;
- implementing artifacts/symbols;
- transition evidence;
- confidence or ambiguity;
- omitted branches and filters;
- relevant external systems or state.

## Evidence inspector

Selecting an entity or relation should expose:

- fact versus inference versus user view;
- origin and analyzer/model version;
- source ranges and resolution traces;
- confidence/status and known limitations;
- supporting graph signals;
- repository revision and freshness;
- conflicting or alternative candidates.

The interface should style deterministic facts and inferred relationships differently.

## Significance and explanation

The product should explain not only what a symbol does but why it matters:

- high fan-in or architectural bridge;
- entry-point proximity;
- flow participation;
- cross-subsystem dependency;
- external I/O or persistence;
- public API or exported surface;
- state mutation;
- change impact.

Importance remains a projection with visible factors, not an unexplained universal score.

## Guided learning state

The learning model may track:

- unseen;
- seen;
- explored;
- understood, when explicitly marked or confidently inferred under a transparent policy;
- prerequisite;
- recommended next;
- intentionally skipped.

Example:

```text
YOU ARE HERE
Audio system -> Processing -> Resampling -> resample_audio

ALREADY EXPLORED
Audio loading
PCM representation

USEFUL PREREQUISITE
Anti-aliasing

NEXT
Tensor conversion
```

Learning state belongs to the user layer and must survive regeneration of the inferred architecture through identity reconciliation.

## Chat as a spatial interface

Chat arrives only after the graph and navigation experience are useful. It should act on the model:

- `Show authentication` focuses a system projection.
- `Trace this endpoint to storage` produces a flow view.
- `Why is this important?` opens an evidence-backed explanation.
- `Hide tests` changes the active view.
- `Group by runtime service` changes projection rules.
- `Show only the critical path` filters branches while preserving a way to reveal them.

Answers should cite graph entities and evidence and, when helpful, manipulate the canvas.

## User-created views

Users should eventually be able to:

- group and rename presentation concepts;
- pin, collapse, hide, and annotate;
- save layouts and filters;
- create perspectives such as `Performance path`, `Deployment architecture`, or `Things I am learning`.

These actions never rewrite the fact graph or silently retrain the inferred model.

## Large-graph behavior

Never attempt to render the database. Query and render a projection appropriate to context.

Techniques to validate include:

- containers and hierarchical aggregation;
- edge bundling or count aggregation;
- lazy child layout;
- viewport-driven loading;
- virtualized lists and source views;
- stable layout caches;
- interaction-priority scheduling;
- local neighborhood and path projections;
- summary nodes for hidden subgraphs.

A two-stage layout approach—containers first, children on demand—is a useful baseline. Rootline's additional challenge is switching semantic levels without disorienting the user.

## Early usability exit criterion

The first UI is successful when a developer prefers using Rootline over a conventional file explorer to reorient in the golden Python repository, even before advanced semantic inference or chat exists.
