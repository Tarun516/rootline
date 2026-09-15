# Benchmark corpus

Benchmark repositories are referenced by exact source revision rather than copied into Rootline. Do not evaluate a dirty live checkout as a golden snapshot. Materialize a pinned commit in an isolated temporary directory before running inventory or future code-intelligence checks.

## R0: Audio Tensor Lab

- Source: the Audio Tensor Lab repository, acquired separately. This file deliberately contains no machine-specific checkout path or copied source tree. A portable acquisition procedure remains to be added.
- Pinned revision: `e750f443fd1f732e73e9cf9612d0e3b57c053a81`
- Expected committed artifacts: 40 regular tracked files at that revision.
- Verified clean-archive inventory: 40 files, 337,358 bytes, 0 skipped; categories are 30 code, 5 config, 3 documentation, 1 data, and 1 other.
- Current local checkout: dirty as observed on 2026-09-15; do not use it for golden counts.
- Initial oracle: exact inventory paths, deterministic ordering, category counts, and no repository-boundary escape.
- Later oracle: Python symbols, import resolution, and selected comprehension questions documented in [evaluation](../docs/11-benchmark-and-evaluation.md).

The first inventory CLI does not itself materialize Git revisions. The result above came from a clean archive of the pinned commit in an isolated temporary directory; its scanner mode was the non-Git filesystem fallback. A benchmark runner must acquire the pinned commit separately and record the analyzed revision and scanner mode with every result.

## Initial comprehension questions

These are evaluation prompts, not yet scored implementation claims:

1. Where does the command-line execution flow begin?
2. Which modules decode WAV data and convert it into tensor representations?
3. How do dataset loading and variable-length collation connect to training?
4. Which module owns the classifier and loss behavior?
5. Which files manage checkpoint persistence and inference?

Expected expert-reviewed answers will be committed only after the Python graph milestone provides source-range evidence.
