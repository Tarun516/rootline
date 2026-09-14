# Engineering learning notebook

This area is for durable technical learning: mental models, experiments, limitations, and implementation implications. These notes differ from ADRs. An ADR records what Rootline decided; a learning note explains the technical territory that informed decisions.

## Recommended note format

Each note should include:

- the question being studied;
- the underlying technical model;
- a small reproducible experiment;
- observed failure modes;
- relevance to Rootline;
- conclusions and remaining questions;
- links to any resulting ADRs or benchmarks.

## Proposed sequence

1. `001-ast-cst-and-tree-sitter.md`
2. `002-parser-lifecycle-and-grammar-packaging.md`
3. `003-code-intelligence-ir.md`
4. `004-symbol-tables-and-lexical-scope.md`
5. `005-python-module-resolution.md`
6. `006-typescript-module-resolution.md`
7. `007-reference-and-call-graphs.md`
8. `008-incremental-computation.md`
9. `009-symbol-identity-across-revisions.md`
10. `010-sqlite-for-local-graph-storage.md`
11. `011-community-detection.md`
12. `012-significance-and-centrality.md`
13. `013-execution-flow-reconstruction.md`
14. `014-hierarchical-graph-layout.md`
15. `015-semantic-zoom.md`
16. `016-search-and-retrieval.md`
17. `017-llm-grounding-and-evaluation.md`

## Notebook rules

- Prefer small experiments over library summaries.
- Document incorrect assumptions and failed approaches.
- Keep benchmark inputs and environment details reproducible.
- Distinguish language semantics from library behavior.
- Record unsupported and ambiguous cases.
- Do not turn hypotheses into architecture merely because a spike succeeded on one fixture.
