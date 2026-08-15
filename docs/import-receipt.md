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
- The initial standalone tree contained 49 binary evidence objects: 41 PDFs
  and eight RLM images. The planning estimate named seven images; the immutable
  source tree contains eight, so all eight were preserved.
- The maintained tree now contains 56 binary evidence objects. The
  Meta-Harness extension adds one captured WebP, three captured WOFF2 font
  files, and one deterministic raw TRAE archive. The Self-Improving Agents
  Survey extension adds one captured PNG and one arXiv PDF.
- Root `.gitattributes` tracks evidence PDFs, PNGs, JPGs, JPEGs, WebPs, WOFF2
  fonts, and compressed TAR archives through Harp's own Git LFS filter.

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

## Post-import Meta-Harness extension

The original 521-row import map remains a fixed account of the initial
standalone import and was not reinterpreted.

The maintained product now also contains:

- a 13-file offline-complete capture of the dated Meta-Harness project page;
- 42 narrow files from the main repository at
  `44b9942127847f7421db70d8c7e48407f09a3c70`;
- all five files from the TerminalBench-2 artifact repository at
  `57fefdb2ff84af3fd81b69d67814acbe69bd0743`;
- the preserved MIT license for the main repository and an explicit missing
  license status for the artifact revision;
- one normalized local TRAE proposal receipt and one deterministic raw archive;
  and
- one additional canonical Markdown deep dive under
  `knowledge/meta_harness/` and its regenerated Atlas outputs.

These additions are Harp maintenance after the initial import. They do not
change the source repository identity, source commit, source path counts, or
dispositions recorded above.

## Post-import RLM implementation extension

The maintained product also adds narrow implementation evidence for the RLM
mechanism:

- four selected files from `alexzhang13/rlm-minimal` at
  `973f8d4acf3af2c86dc170af91607bf8b0c4d0ea`;
- six selected files from `alexzhang13/rlm` at
  `72d6940142ddfb84ee6be573dc999a37e633e671`;
- the MIT license and immutable remote/revision sidecars for each snapshot; and
- manifest-bound byte counts and SHA-256 digests under
  `evidence/implementations/manifest.tsv`.

The snapshots support mechanism and present-day implementation inspection.
They do not reproduce the paper benchmarks, later post-training experiments,
cloud-adapter safety, or production behavior. This extension does not alter the
fixed 521-row initial import account.

## Post-import Self-Improving Agents Survey extension

The maintained product also adds a bounded survey-anchor packet for
`https://selfimproving-agent.github.io/`:

- a dated capture of the project hub and same-site overview figure;
- arXiv `2607.13104v1` metadata, abstract page, PDF, and derived reading text;
- pinned GitHub Pages source bytes at
  `d8af6607ced118351108670f823cd106649cb757`;
- update-repository metadata, README, and MIT license text for
  `selfimproving-agent/Awesome-Self-Improving-Agents` at
  `57a1d89e5bafcd65db7feb51e809422700aeb48a`;
- a one-hop link inventory that records hub topology without crawling child
  bibliographies; and
- four canonical Markdown packet documents under
  `knowledge/self_improving_agents_survey/` with regenerated Atlas outputs.

The packet supports survey taxonomy, dated hub topology, and source identity.
It does not validate linked-paper mechanisms, benchmark results, venue status,
or linked-repository implementation behavior. This extension does not alter the
fixed 521-row initial import account.

## Standalone payload digest

The final tracked product payload, excluding this self-referential receipt and
ignored build/materialization state, has SHA-256:

`ba07f795d40afe66a65092e9f168c2d99dbcd212d5655dbf6790dfe9ac808ff1`
