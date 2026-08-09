# Source registry template

Use one heading per source so claim-ledger entries can link to the exact source
record.

## EXAMPLE-PAPER: Darwin Godel Machine paper

- Class: `primary paper`
- Title: Example research paper
- Artifact: [Local paper capture](../../evidence/weng/text/dgm.txt)
- Stability: `pinned`
- Immutable identity: `arXiv:2505.22954v3`
- Semantic locators: section, appendix, table, figure, and algorithm.
- Can prove: Paper method, wording, author-reported results, and limitations.
- Cannot prove: Independent reproduction or current released-code behavior.

## EXAMPLE-REPO: Released DGM implementation

- Class: `pinned implementation`
- Title: Example released implementation
- Artifact: [Local implementation snapshot](../../evidence/implementations/dgm/snapshot/README.md)
- Stability: `pinned`
- Immutable identity: `git:a565fd2d1dca504ef5104a7cc0f3bdc4ab9b4fd2`
- Semantic locators: file paths and line anchors.
- Can prove: Source behavior at the pinned revision.
- Cannot prove: Historical experiment identity, runtime safety, or benchmark
  reproduction.

## EXAMPLE-MUTABLE-PAGE: Mutable results page

- Class: `mutable first-party page`
- Title: Example mutable results page
- Artifact: [Local dated capture](../../evidence/weng/metadata/dgm-arxiv.html)
- Stability: `dated observation`
- Observed: `2026-08-08`
- Semantic locators: `Abstract page > Description`.
- Can prove: What the page showed on the observation date.
- Cannot prove: Current state after that date or historical state before it.
