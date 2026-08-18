---
id: autodiff-geometry-source-acquisition-and-parsing
title: Autodiff geometry source acquisition and parsing
type: source-parsing-report
status: active
created: 2026-08-18
updated: 2026-08-18
tags: [autodiff, jax, parsing, sources, curriculum]
confidence: high
---

# Autodiff geometry source acquisition and parsing

Back to the [[knowledge/autodiff_geometry/autodiff_geometry_index|packet index]].

This report is the durable source-ingestion handoff for the autodiff geometry
program. It records what Harp has captured, what was parsed into text or
structured inventories, and which sources remain gated before they can support
direct formalization claims.

## Mechanism

The acquisition and parser entrypoint is
[[evidence/autodiff_geometry/acquire.py|`evidence/autodiff_geometry/acquire.py`]].

Use:

```sh
python3 evidence/autodiff_geometry/acquire.py --verify
```

for offline verification of the checked-in bundle. A networked refresh is:

```sh
python3 evidence/autodiff_geometry/acquire.py
```

Run a networked refresh only when intentionally updating source observations;
it rewrites capture timestamps, manifests, and derived parser artifacts.

## Checked-In Parser Outputs

**EVIDENCE -- local parser bundle.** The capture receipt is
[[evidence/autodiff_geometry/capture_receipt.tsv|`capture_receipt.tsv`]], the
source manifest is [[evidence/autodiff_geometry/manifest.tsv|`manifest.tsv`]],
and the parse ledger is
[[evidence/autodiff_geometry/source_parse_report.tsv|`source_parse_report.tsv`]].
Every checked-in artifact is bound by
[[evidence/autodiff_geometry/artifact_inventory.tsv|`artifact_inventory.tsv`]].

| Source | Parser result | Local artifact | Formalization use |
|---|---|---|---|
| JAX autodiff cookbook | Parsed notebook cells and rendered-page text. | [[evidence/autodiff_geometry/parsed/jax_autodiff_cookbook_sections.tsv|section inventory]] and [[evidence/autodiff_geometry/text/jax_autodiff_cookbook_cells.txt|cell text]] | Primary notation and API surface for `grad`, `argnums`, `jvp`, `vjp`, `jacfwd`, `jacrev`, and Hessian-vector products. |
| JAX API source | Parsed public autodiff definition locations. | [[evidence/autodiff_geometry/parsed/jax_api_public_autodiff_defs.tsv|API definition inventory]] | API-surface locator only; not an implementation-correctness proof. |
| Spivak public repository | Parsed root TeX structure; full book text not imported. | [[evidence/autodiff_geometry/parsed/spivak_tex_outline.tsv|TeX outline]] | Curriculum alignment and notation lineage only until direct rights review permits claim-specific text use. |
| SICM open-access HTML | Parsed open-access HTML book units into text. | [[evidence/autodiff_geometry/text/sicm_edition_2_html_text.txt|SICM extracted text]] and [[evidence/autodiff_geometry/parsed/sicm_html_units.tsv|unit inventory]] | Mechanics notation and variational-calculus bridge after claim-specific review. |
| FDG open-access PDF | Captured PDF and extracted text with `pdftotext -layout`. | [[evidence/autodiff_geometry/text/functional_differential_geometry_9580.txt|FDG extracted text]] and [[evidence/autodiff_geometry/parsed/fdg_text_markers.tsv|marker inventory]] | Prologue and notation appendix become the first direct source for operator notation beyond JAX's bibliographic claim. |
| Tao analysis repository | Parsed top-level Lean imports and project metadata. | [[evidence/autodiff_geometry/parsed/tao_analysis_imports.tsv|import inventory]] | Proof-engineering guidance; no dependency or theorem reuse claim. |

## Rights and Evidence Boundaries

**EVIDENCE -- JAX and Tao repositories.** The JAX notebook, JAX API source, and
Tao analysis files are captured at immutable Git commits. They are appropriate
for source identity, API-surface, and proof-engineering guidance claims.

**EVIDENCE -- SICM and FDG open-access routes.** The SICM text was extracted
from the MIT content-server open-access HTML zip, but the raw zip is not
vendored because it contains fonts and images outside the current text-parser
need. The FDG PDF is captured directly because the JAX cookbook specifically
points to the book's notation discussion and the MIT content server exposes the
PDF as an open-access object.

**MISSING -- Spivak theorem text.** The public Spivak repository README warns
that it is for personal use only. Harp therefore records repository metadata
and the TeX outline, but it does not vendor the full book PDF or chapter text.
Future agents may use the outline to decide what to request or inspect, but
must not promote direct Spivak theorem claims without a rights-cleared source
locator.

## Parser Contract For Future Agents

Before starting a new formalization phase:

1. Run `python3 evidence/autodiff_geometry/acquire.py --verify`.
2. Read `source_parse_report.tsv` and select only rows whose parser status is
   `parsed` or explicitly appropriate for the task.
3. Record every new mathematical claim in
   [[knowledge/autodiff_geometry/claim_evidence_ledger|the claim ledger]] before
   translating it into Lean.
4. If a source needs fresh network acquisition, rerun the acquisition script,
   inspect the diff, and update the claim ceiling before committing.
5. Keep source bytes, parsed artifacts, curriculum docs, and Lean theorem
   declarations in separate commits when the phase is large enough to split.

The parser is not a theorem prover. It makes text and structure available for
careful reading; every formal statement still needs an explicit source claim,
mathlib survey, and compiled Lean declaration before it becomes a Harp proof
claim.
