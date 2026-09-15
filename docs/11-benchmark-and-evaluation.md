# Benchmark and evaluation system

## Purpose

Rootline should be developed against reproducible evidence rather than attractive demonstrations. The evaluation system must measure both code-intelligence correctness and the product outcome: how quickly and accurately a user understands a repository.

## Repository corpus

| ID | Repository class | Purpose |
| --- | --- | --- |
| R0 | Audio Tensor Lab | Small Python repository understood deeply; daily correctness oracle |
| R1 | Medium TypeScript product | Aliases, API, persistence, and frontend/backend boundaries |
| R2 | Native/systems repository, initially Redis or a comparable project | Lower-level language and module behavior; different structural patterns |
| R3 | Large Python framework/application, initially FastAPI or a comparable project | Dynamic-language limitations and scale |
| R4 | Large TypeScript monorepo, potentially VS Code or a smaller controlled precursor | Workspaces, packages, resolution complexity, and layout pressure |
| R5 | Multilingual repository | Cross-language and service boundaries after initial maturity |

Every corpus entry must pin a repository URL or local source identity, exact commit, submodule state, fixture configuration, and any exclusions.

## Golden comprehension questions

Each repository should have expert-reviewed expected answers for:

1. What is this project?
2. What are its five major subsystems?
3. Where are its true entry points?
4. Trace one important request, event, command, or data flow.
5. What are the ten most important files and why?
6. What are the ten most important symbols and why?
7. Which components interact with external systems?
8. Where is persistent state stored?
9. Which boundaries are architectural, and which are only directory organization?
10. What changed between two pinned commits?
11. Which fact, subsystem, flow, or learning-model entities should update after that change?
12. Where does static analysis remain unsupported, ambiguous, or incomplete?

Answers should include acceptable variants and evidence, not only prose strings.

## Structural correctness metrics

### Inventory

- files found versus expected;
- ignored/re-included files;
- repository-boundary violations;
- language and artifact classification;
- manifest/workspace discovery;
- stability across repeated runs.

### Parsing and symbols

- declaration precision and recall by symbol kind;
- ownership correctness;
- source-range correctness;
- parse failures and recovery rate;
- unsupported-file rate;
- deterministic snapshot stability.

### Resolution

- resolved internal import precision and recall;
- external dependency classification;
- ambiguous and unresolved rate;
- false definite resolutions;
- resolver latency;
- configuration-scope correctness.

### Relationships

- containment correctness;
- calls/reference precision and coverage by language;
- inheritance/implementation correctness;
- dangling-edge count, which should be zero after publication;
- evidence completeness.

### Incremental analysis

- affected-region size;
- false invalidation and missed invalidation;
- identity preservation;
- deletion correctness;
- failed-publication rollback correctness;
- convergence with clean full rebuild;
- semantic staleness detection.

## System-understanding quality

Architecture, significance, and flow evaluation require expert comparison rather than exact text matching.

Measure:

- subsystem cluster precision, recall, and stability;
- agreement with expert boundaries;
- explanation evidence quality;
- entry-point ranking;
- top-k important file/symbol agreement;
- end-to-end flow step correctness and ordering;
- false confident inference rate;
- visibility and accuracy of uncertainty;
- user correction rate and correction persistence.

## Time-to-understanding evaluation

Give participants repository comprehension tasks and compare tools or Rootline versions using:

- time to first correct system summary;
- time to locate an entry point;
- time to trace a selected flow;
- time to identify persistent state or an external boundary;
- answer correctness and confidence calibration;
- navigation actions and backtracking;
- retained understanding after a delay;
- perceived orientation and trust.

This evaluation may begin informally with the project owner on Audio Tensor Lab, then become a structured study.

## Performance metrics

| Area | Metrics |
| --- | --- |
| Scan | files/sec, MB/sec, ignored count, bytes reread, hash throughput |
| Parse | ms/file, symbols/sec, parser initialization, failures, unsupported count |
| Resolve | imports/sec, resolved/ambiguous/unresolved percentages, config-loading time |
| Graph | nodes/edges, build time, persisted size, query latency, adjacency build time |
| Incremental | detection time, invalidated entities, reanalysis time, publication time |
| UI | time to first useful map, projection latency, layout latency, FPS, rendered count |
| Semantic | tasks, tokens/cost where relevant, queue latency, cache hit rate, stale outputs |
| Memory | peak RSS, retained index memory, client heap, graph projection size |

Cold analysis, warm analysis, incremental analysis, and UI-only projection benchmarks must be reported separately.

## Benchmark methodology

- Pin hardware characteristics or record them with every run.
- Pin repository revisions and Rootline analyzer version.
- Run enough iterations to report median and dispersion.
- Separate filesystem cache effects where meaningful.
- Separate deterministic engine time from semantic-provider latency.
- Record configuration and exclusions.
- Store machine-readable results.
- Fail benchmarks on invalid graph output, not only performance regressions.
- Compare incremental results to clean rebuilds regularly.

## Reference comparisons

Compare Rootline with selected repository-comprehension and code-graph tools on the same repository revisions and questions. The comparison should identify capability and architecture differences, not manufacture a single score across unlike systems.

Useful comparison dimensions:

- time to first usable output;
- full deterministic and full enriched completion time;
- structural coverage;
- import/call correctness;
- architecture and flow quality;
- uncertainty presentation;
- incremental work and correctness;
- large-graph usability;
- model calls and semantic cost;
- ability to work without semantic enrichment.

Additional tools may be included when they expose relevant baselines.

## Benchmark artifacts

Proposed future layout:

```text
benchmarks/
  corpus.yaml
  questions/
  expected/
  fixtures/
  runners/
  results/
  reports/
```

Do not commit proprietary benchmark source. Store only allowed metadata, derived fixtures, and instructions for acquiring repositories.

Implemented so far: `corpus.yaml` (pinned R0 revision and expected inventory counts), `expected/` (exact R0 path oracle), `runners/r0-inventory.sh` (portable acquisition, analysis, and scoring), and `results/` (local machine-readable run records). Golden questions, expected answers, fixtures, and reports remain future work; see `benchmarks/README.md` and the implementation tracker for current status.
