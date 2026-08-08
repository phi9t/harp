# Standalone import receipt

## Source identity

- Source repository: Tarocco.
- Source commit: `d14d0c5d2f07bfcecebe300d047fbb99e8ee36db`.
- Source tree: `2ad5c149cc4019330be4d1748df092a7bd9883cc`.
- Import method: immutable `git archive` and `git show` reads from that commit;
  no product file was copied from the source working tree.
- Synchronization policy: this is a fixed snapshot. Later source changes require
  a separate explicit import.

## Classified input

`docs/import-map.tsv` classifies all 521 scoped source paths:

- 172 RSI product files;
- 322 evidence files: 291 Weng, 27 RLM, and four SICP;
- five SICP evaluator files;
- six RSI compiler files; and
- 16 auxiliary task, QMD, and maintained specification files.

Allowed dispositions are `ported`, `rewritten`, `generated`, `replaced`, and
`omitted-tarocco-specific`. Source paths and the source-specific omitted
disposition are percent-encoded in the TSV so this receipt remains the only
tracked file that names the source repository.

## Binary evidence

- Forty pointer-backed RSI/RLM PDFs were hydrated from the source repository's
  local Git LFS object store.
- Every hydrated object matched its pointer's SHA-256 and recorded size before
  import.
- The standalone tree contains 49 binary evidence objects: 41 PDFs and eight
  RLM images. The planning estimate named seven images; the immutable source
  tree contains eight, so all eight were preserved.
- Root `.gitattributes` tracks every evidence PDF, PNG, JPG, and JPEG through
  Harp's own Git LFS filter.

## Rewrites and removals

- Canonical material moved from `knowledge/rsi/` to `content/`.
- The dedicated app moved to `atlas/`, with
  `atlas/src/content/generated/corpus.json` and
  `atlas/dist/harp-atlas.html` as derived artifacts.
- Evidence moved from source third-party paths to `evidence/weng/`,
  `evidence/rlm/`, and `evidence/sicp/`; provenance and acquisition metadata
  became Harp-owned while captured third-party bytes remained unchanged.
- Public implementation locators became narrow tracked snapshots under
  `evidence/implementations/`.
- The SICP capstone became a Pi/Hermes/Codex agent-harness architecture
  dossier.
- Source-runtime comparisons, Temporal-preview claims, SFT implementation
  links, old run receipts, old implementation plans, QMD integration, and
  general Knowledge Atlas references were omitted or replaced.
- No repository-wide license was imported or added.

## Standalone payload digest

The final tracked product payload, excluding this self-referential receipt and
ignored build/materialization state, has SHA-256:

`1d3dcad85b57074b940215bca8fccee720423c84df48f31ffe4268870f8440f8`
