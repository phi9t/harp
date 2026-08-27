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
- The maintained tree now contains 65 binary evidence objects: 51 PDFs, nine
  PNGs, one WebP, three WOFF2 font files, and one compressed archive. The
  Meta-Harness extension adds one captured WebP, three captured WOFF2 font
  files, and one deterministic raw TRAE archive. The Self-Improving Agents
  Survey extension adds one captured PNG and one arXiv PDF. The Verified
  Coevolution Agenda extension adds five arXiv PDFs. The DeepSeek Harness study
  extension adds one Cordis paper PDF. The Lean proof-engineering extension
  adds one arXiv PDF.
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

## Post-import Verified Coevolution Agenda extension

The maintained product also adds a bounded verified-coevolution agenda packet:

- the supplied research agenda preserved as immutable design input;
- full revision-pinned arXiv records for LADDER `2503.00735v3`, PRIME-RL TTRL
  `2504.16084v3`, NSRSA `2603.21558v1`, SAHOO `2603.06333v1`, and Scrivens
  verification `2603.28650v1`, including Atom metadata, abstract pages,
  semantic HTML, PDFs, and deterministic `pdftotext -raw` text;
- a first-party Nature capture and deterministic text extraction for the
  model-collapse article `s41586-024-07566-y`;
- metadata-only arXiv records for Godel Agent `2410.04444v4`, Mendel Godel
  Machine `2608.07645v1`, and the historical Godel Machine `cs/0309048v5`;
- eight new source-registry rows and three evidence-graph edges; and
- five canonical Markdown packet documents under
  `knowledge/verified_coevolution_agenda/` with regenerated Atlas outputs.

The packet publishes a falsifiable research agenda. It does not reproduce paper
experiments, validate reported benchmark results, prove broad alignment
properties, or treat the supplied agenda as source evidence for paper claims.
This extension does not alter the fixed 521-row initial import account.

## Post-import DeepSeek Harness study extension

The maintained product also adds a bounded DeepSeek Harness design-decision
study:

- a narrow implementation snapshot of
  `https://github.com/deepseek-ai/deepseek-harness.git` at
  `47f943859bef60e4160492346772ded9b24f765a`;
- the cited Cordis paper captured from `https://github.com/cordiverse/paper.git`
  at tag `v8` / commit `948a07b369c62adb3b12e102458be5c18dfb69b9`,
  including PDF, upstream README, derived text, provenance, and artifact
  inventory;
- a local keyless DSH tool-runtime smoke receipt recording `pnpm@11.7.0`
  install and `packages/core/tools/tests/tools.spec.ts` with 136 passing tests;
- two new source-registry rows and three evidence-graph edges for the Cordis
  paper and DSH smoke receipt; and
- thirteen canonical Markdown packet documents under
  `knowledge/deepseek_harness/` with regenerated Atlas outputs.

The packet supports design-decision study of DSH's pinned plugin composition,
events, sessions, tools, sandbox policy, subagent seam, persistence/API
packaging, and Cordis basis. It does not reproduce DeepSWE scores, run a paid
model provider, validate UI behavior, prove platform sandbox enforcement, or
establish autonomous recursive self-improvement. This extension does not alter
the fixed 521-row initial import account.

## Post-import Lean proof-engineering extension

The maintained product also adds bounded Lean proof-engineering evidence for
the Crouzeix formal-validation workstream:

- a revision-pinned Lean4Agent arXiv PDF, abstract page, and deterministic
  `pdftotext -raw` text;
- dated first-party Lean, Mistral, and Lean community documentation captures;
- immutable Lean repository identity metadata plus upstream README and Apache
  2.0 license bytes for the resolved Lean commit;
- a revision-pinned NNG4 introductory proof phase with upstream remote,
  revision, Apache 2.0 license evidence, and a standalone Lean/mathlib
  `v4.32.1` Lake project;
- source lock, closure, capture receipt, manifest, and artifact inventory rows;
  and
- acquisition contract tests for the offline evidence bundle.

The packet supports local Lean proof-engineering context and runtime-workflow
inspection plus Harp-owned proof of the 77 ordinary active NNG4 introductory
statements imported through upstream `Game.lean`. It does not provision Lean,
validate a Crouzeix proof, trust model outputs as formal results, treat the
upstream `xyzzy` Fermat endpoint as proof evidence, cover old/WIP NNG4 side
worlds, or alter the fixed 521-row initial import account.

## Post-import Autodiff Geometry formalization extension

The maintained product also adds a bounded Autodiff Geometry formalization
packet:

- source-backed registry and claim ledger entries for the JAX autodiff
  cookbook, pinned JAX API source, Spivak repository pointer, SICM course page,
  and blocked MIT Press pointers for SICM and FDG;
- a standalone Lean/mathlib `v4.32.1` Lake project proving finite-coordinate
  contracts for gradient component selection, JVPs, VJPs, Hessian-vector
  products, and JVP/VJP dot-product duality;
- no vendored textbook or JAX source bytes in this phase; and
- four canonical Markdown packet documents under `knowledge/autodiff_geometry/`
  with regenerated Atlas outputs.

The packet supports notation-bound finite-coordinate proof contracts and a
roadmap for later differentiability/API-correspondence phases. It does not
verify JAX's Python implementation, reproduce cookbook numerics, claim
manifold-level theorem coverage, or promote textbook claims beyond the
currently captured evidence boundary. This extension does not alter the fixed
521-row initial import account.

## Post-import Jin Crouzeix proof-port extension

The maintained product also adds a Harp-local Lean 4.32 port of Jin's
holomorphic Crouzeix route:

- a tracked `formalization/lean/CrouzeixConjecture/` proof implementation
  derived from Jin commit `565b6a3e0659b6e0785f783b016c3f6d9f171fa5`;
- a shared-root `CrouzeixConjecture` Lean library and a Harp-owned terminal
  assembly theorem under `Crouzeix.Jin.Terminal`;
- receipt-backed source-map status for all three Jin rows, including the
  terminal theorem `CrouzeixConjecture.crouzeixConjecture`; and
- proof-slice audit evidence that the terminal theorem compiles under the
  cached shared Lean root with observed axioms `propext`, `Classical.choice`,
  and `Quot.sound`.

The unmodified upstream archive remains remote-only evidence by URL, byte
count, and digest because the upstream revision had no license file present.
This extension does not alter the fixed 521-row initial import account.

## Post-import Crouzeix editorial extension

The maintained product also adds a reader-facing technical editorial on the
Crouzeix conjecture and Jin's proof:

- one canonical Markdown article under `knowledge/crouzeix_conjecture/` aimed
  at senior ML research-engineering readers;
- stable corpus registration and packet-roster coverage for the article; and
- regenerated Atlas corpus and static export artifacts.

The article synthesizes the existing Crouzeix packet and Harp-local proof-port
receipts. It does not add new upstream source evidence, alter the pinned Jin
source identities, claim peer-review completion, or alter the fixed 521-row
initial import account.

## Standalone payload digest

The final tracked product payload, excluding this self-referential receipt and
ignored build/materialization state, has SHA-256:

`94c933573092a4a823a4a100ccac571e7efb4ecf9301f2f6d0ca7abafd47fdb1`
