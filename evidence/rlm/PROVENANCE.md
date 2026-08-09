# Harp provenance: a1zhang RLM harness generalization

## Identity and scope

This bundle supports the RSI knowledge capture for Alex L. Zhang and Omar
Khattab's July 2026 blog, *Language model harnesses are compositional
generalizers*. The local X clipping supplied the seed, but the canonical blog
is the substantive argument. The bundle also captures the mechanism's
predecessor paper, *Recursive Language Models*, at arXiv revision
`2512.24601v3`, and immutable metadata for the official implementation at Git
commit `72d6940142ddfb84ee6be573dc999a37e633e671`.

The RLM paper → implementation → training/generalization blog lineage is a
first-class mechanism anchor for the RSI topic. It remains historically outside
the Weng-rooted citation closure: Weng's anchor was published on 2026-07-04,
while Zhang and Khattab's blog was published on 2026-07-20. The topic therefore
uses multiple anchors without rewriting either source's bibliography.

The current RLM-anchor capture stops at the blog, its predecessor paper, the
official implementation metadata, the blog's 11-entry bibliography, and 51
structured outgoing references from the paper. Those outgoing references are
parsed but not automatically admitted or fetched in this pass.

Harp later added narrow source snapshots of the full and minimal official
repositories under `evidence/implementations/`. Those files are governed by
`evidence/implementations/manifest.tsv` and do not change this bundle's
original artifact inventory or capture receipt.

## Captured representations

- The complete blog HTML, a plain-text sidecar, its eight first-party figures,
  and the blog's 11-entry BibTeX file.
- The arXiv v3 PDF, abstract/license metadata, semantic HTML, a plain-text
  sidecar, and 51 structured `ltx_bibitem` citation records.
- The official repository README and MIT license at the pinned commit. No
  source tree was materialized in this bundle and no upstream benchmark was
  executed.
- The announcement identity is retained in `sources.tsv`; the substantive
  evidence is the captured first-party blog.

`manifest.tsv` records one row per source identity. `artifact_inventory.tsv`
records 19 bundle files totaling 15,388,190 bytes and their SHA-256 digests.
`sources.tsv` records relationships and claim ceilings. `acquire.sh` is the
idempotent capture workflow, and `run-receipt.tsv` records its capture time,
tool versions, source-table digest, and workflow digest.

## Evidence boundary

The blog is an author report, not an independent reproduction. Its plots and
configuration support claims about what the authors ran and observed; they do
not establish that every RLM harness generalizes, that the compared systems
received equal total compute, or that trajectory similarity proves equality of
model-internal computation. The knowledge note preserves these distinctions as
`CLAIM`, `EVIDENCE`, `INFERENCE`, and `MISSING`.

The term “Recursive Language Model” names a system-level inference strategy:
a base language model externalizes context into a REPL and can make recursive
model calls. It is not a new recursive Transformer layer, and the reported
training experiment is not itself recursive self-improvement of the harness.

## Copyright and licenses

There is no bundle-wide license. The blog displays a 2026 copyright notice but
no reuse license was located, so its HTML and figures remain copyright-bound.
The arXiv metadata identifies the paper as CC BY 4.0. The implementation's
captured `LICENSE` is MIT. Harp-authored manifests and commentary do not
relicense any upstream artifact.
