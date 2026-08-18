---
id: autodiff-geometry-source-registry
title: Autodiff geometry source registry
type: source-registry
status: active
created: 2026-08-18
updated: 2026-08-18
tags: [autodiff, jax, sources, provenance]
confidence: medium
---

# Autodiff geometry source registry

Back to the [[knowledge/autodiff_geometry/autodiff_geometry_index|packet index]].

## Source classes and boundary

The first phase uses public first-party sources and immutable Git identities
where available. It records textbook links as notation-lineage evidence, not as
permission to reproduce book content.

| ID | Identity | Stability | Local artifact | Claim ceiling |
|---|---|---|---|---|
| `JAX-AUTODIFF-COOKBOOK` | JAX documentation notebook `docs/notebooks/autodiff_cookbook.ipynb` at Git commit `0eb0f676ba22b15b9dfe29518ab0d40277b7138d`; rendered `latest` page captured 2026-08-18 | The rendered `latest` page is mutable; the Git commit is immutable. | [[evidence/autodiff_geometry/metadata/jax_autodiff_cookbook.ipynb|notebook]], [[evidence/autodiff_geometry/text/jax_autodiff_cookbook_cells.txt|cell text]], and [[evidence/autodiff_geometry/parsed/jax_autodiff_cookbook_sections.tsv|section inventory]] | Cookbook notation claims, examples, and API-level descriptions of `grad`, `argnums`, `jvp`, `vjp`, `jacfwd`, `jacrev`, and Hessian-vector products. Not implementation verification. |
| `JAX-API-SOURCE` | JAX source file `jax/_src/api.py` at Git commit `0eb0f676ba22b15b9dfe29518ab0d40277b7138d` | Immutable Git commit. | [[evidence/autodiff_geometry/metadata/jax_api.py|source]] and [[evidence/autodiff_geometry/parsed/jax_api_public_autodiff_defs.tsv|definition inventory]] | Existence and location of public transformation definitions such as `grad`, `value_and_grad`, `jacfwd`, `jacrev`, `hessian`, `jvp`, and `vjp`. Not semantic equivalence to Lean. |
| `SPIVAK-CALCULUS-ON-MANIFOLDS-REPO` | Public GitHub repository `zongpingding/Calculus_On_Manifolds_Michael_Spivak`, default branch `main`, observed at `ac453ea3c0a43fe2bc3ee5b144bee5d9d1cf174f` | Repository is mutable; observed commit is immutable. | [[evidence/autodiff_geometry/metadata/spivak_README.md|README]], [[evidence/autodiff_geometry/metadata/spivak_Calculus_On_Manifolds.tex|root TeX]], and [[evidence/autodiff_geometry/parsed/spivak_tex_outline.tsv|outline]] | Bibliographic/source availability and curriculum structure for Spivak notation lineage only. The README warns personal use; Harp does not reproduce the full scan or chapter text. |
| `SICM-COURSE-PAGE` | Gerald Jay Sussman's MIT CSAIL course page for 6.946/6.5160, observed 2026-08-18; page links SICM second edition and an HTML mechanics book mirror. | Mutable dated web observation. | [[evidence/autodiff_geometry/metadata/sicm_course_page.html|course page]] and [[evidence/autodiff_geometry/text/sicm_course_page.txt|course text]] | Public-course pointer to SICM second edition and open-access HTML route. |
| `SICM-OPEN-ACCESS-HTML` | MIT content-server open-access SICM second-edition HTML zip. | Mutable content-server route with local digest receipt. | [[evidence/autodiff_geometry/metadata/sicm_edition_2_zip_digest.txt|zip digest]], [[evidence/autodiff_geometry/metadata/sicm_edition_2_zip_listing.tsv|zip listing]], [[evidence/autodiff_geometry/text/sicm_edition_2_html_text.txt|extracted text]], and [[evidence/autodiff_geometry/parsed/sicm_html_units.tsv|unit inventory]] | Direct local text evidence for SICM notation and mechanics examples after claim-specific review. The raw zip is not vendored. |
| `FDG-OPEN-ACCESS-PDF` | MIT content-server open-access PDF for *Functional Differential Geometry*. | Mutable content-server route; local PDF and extracted text are digest-bound. | [[evidence/autodiff_geometry/artifacts/functional_differential_geometry_9580.pdf|PDF]], [[evidence/autodiff_geometry/text/functional_differential_geometry_9580.txt|extracted text]], and [[evidence/autodiff_geometry/parsed/fdg_text_markers.tsv|marker inventory]] | Direct local text evidence for FDG notation, especially Prologue and notation appendix material. |
| `TAO-ANALYSIS-REPO` | Terry Tao's `teorth/analysis` repository at commit `8f9e0fc5f063d0839f9b2bfc3ed9607b417877fb`. | Repository is mutable; observed commit is immutable. | [[evidence/autodiff_geometry/metadata/tao_analysis_README.md|README]], [[evidence/autodiff_geometry/metadata/tao_analysis_Analysis.lean|top-level Lean file]], and [[evidence/autodiff_geometry/parsed/tao_analysis_imports.tsv|import inventory]] | Proof-engineering guidance and mathlib orientation. Not a dependency and not source evidence for JAX semantics. |

## Capture notes

The JAX cookbook rendered page was available at:

`https://docs.jax.dev/en/latest/notebooks/autodiff_cookbook.html`

The immutable notebook source used for claim locators is:

`https://raw.githubusercontent.com/jax-ml/jax/0eb0f676ba22b15b9dfe29518ab0d40277b7138d/docs/notebooks/autodiff_cookbook.ipynb`

The API source locator used for function definitions is:

`https://raw.githubusercontent.com/jax-ml/jax/0eb0f676ba22b15b9dfe29518ab0d40277b7138d/jax/_src/api.py`

## Cannot-prove boundaries

- The JAX docs describe the public API and notation contract; they do not prove
  Harp's Lean statements.
- The Lean project proves finite-coordinate algebraic contracts; it does not
  validate JAX's Python implementation or floating-point execution.
- The Spivak repository is a public source pointer. This packet does not assert
  redistribution rights for the underlying book scan.
- The MIT Press product pages were not used as the text source. SICM and FDG
  text evidence comes from the MIT content-server open-access routes recorded
  in the local manifest.
- Tao's analysis repository is guidance for proof organization and mathlib
  usage; Harp remains standalone and does not import it.
