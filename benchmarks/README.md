# Benchmark corpus

Benchmark repositories are referenced by exact source revision rather than copied into Rootline. Do not evaluate a dirty live checkout as a golden snapshot. Materialize a pinned commit in an isolated temporary directory before running inventory or future code-intelligence checks.

## Corpus definition

`corpus.yaml` is the machine-readable source of truth for pinned revisions, source locations, and expected inventory counts. `expected/` holds derived oracles (exact path lists), never benchmark source. `runners/` holds the portable scripts that acquire, analyze, and score each benchmark. `results/` keeps local machine-readable run records (`*.json` is ignored by Git; only `.gitkeep` is committed).

## R0: Audio Tensor Lab

- Source: https://github.com/Tarun516/audio-tensor-lab.git (pinned in `corpus.yaml`; this is the designated benchmark corpus, so naming it here is intentional).
- Pinned revision: `e750f443fd1f732e73e9cf9612d0e3b57c053a81`
- Expected committed artifacts: 40 regular tracked files at that revision.
- Verified clean-archive inventory: 40 files, 337,358 bytes, 0 skipped; categories are 30 code, 5 config, 3 documentation, 1 data, and 1 other.
- Current local checkout: dirty as observed on 2026-09-15; do not use it for golden counts.
- Initial oracle: exact inventory paths (`expected/r0-inventory.txt`), deterministic ordering, category counts, and no repository-boundary escape.
- Later oracle: Python symbols, import resolution, and selected comprehension questions documented in [evaluation](../docs/11-benchmark-and-evaluation.md).

## Reproducing the R0 inventory benchmark

From a clean checkout with `bash`, `git`, `cargo`, and network access:

```sh
benchmarks/runners/r0-inventory.sh
```

The runner clones the pinned revision into an isolated temporary directory (removed afterwards unless `--keep-workdir` is passed), verifies the materialized `HEAD` equals the pinned revision and the worktree is clean, runs `cargo run --locked --bin rootline -- index <materialized-repo>`, and checks scanner mode, revision, file/byte/skip counts, deterministic path ordering, and exact paths against `expected/r0-inventory.txt`. It writes a machine-readable record to `benchmarks/results/r0-inventory-<UTC-timestamp>.json` containing the pinned and analyzed revisions, scanner mode, counts, duration, Rootline revision and CLI version, and host information. A nonzero exit means the oracle failed; the result file records the failure count.

The first inventory CLI does not itself materialize Git revisions. Earlier results came from a clean archive of the pinned commit in an isolated temporary directory; the scanner mode was the non-Git filesystem fallback for archives and is the Git ignore-aware mode for the clone this runner produces. Every result records the analyzed revision and scanner mode.

## Initial comprehension questions

These are evaluation prompts, not yet scored implementation claims:

1. Where does the command-line execution flow begin?
2. Which modules decode WAV data and convert it into tensor representations?
3. How do dataset loading and variable-length collation connect to training?
4. Which module owns the classifier and loss behavior?
5. Which files manage checkpoint persistence and inference?

Expected expert-reviewed answers will be committed only after the Python graph milestone provides source-range evidence.
