---
id: autodiff-geometry-layered-curriculum-tracker
title: Autodiff geometry layered curriculum tracker
type: curriculum-tracker
status: active
created: 2026-08-18
updated: 2026-08-18
tags: [autodiff, curriculum, lean, mathlib, tracker]
confidence: high
---

# Autodiff geometry layered curriculum tracker

Back to the [[knowledge/autodiff_geometry/autodiff_geometry_index|packet index]].

This tracker makes the autodiff geometry program restartable. A fresh agent
should be able to choose the next bounded slice, check its source and parser
state, and know which Lean surface it is allowed to extend.

## Current State

| Area | Status | Evidence | Next agent action |
|---|---|---|---|
| Source acquisition | `captured-and-parsed` | [[knowledge/autodiff_geometry/source_acquisition_and_parsing|source acquisition report]] | Run `python3 evidence/autodiff_geometry/acquire.py --verify` before using the bundle. |
| First Lean phase | `compiled` | [[knowledge/autodiff_geometry/formalization_roadmap|formalization roadmap]] | Keep finite-coordinate operator contracts green while adding analytic layers. |
| Math foundations bridge | `available` | [[knowledge/mathematical_foundations/mathematical_foundations_index|mathematical foundations packet]] | Reuse Bishop/Lax-inspired linear algebra, probability, and optimization vocabulary as prerequisite material. |
| JAX notation bridge | `parsed` | [[evidence/autodiff_geometry/parsed/jax_autodiff_cookbook_sections.tsv|JAX section inventory]] | Promote cookbook examples into curriculum exercises one operator family at a time. |
| Spivak bridge | `structure-parsed-content-gated` | [[evidence/autodiff_geometry/parsed/spivak_tex_outline.tsv|Spivak TeX outline]] | Use chapter structure for alignment only; do not quote or formalize direct claims from gated text. |
| SICM bridge | `parsed` | [[evidence/autodiff_geometry/text/sicm_edition_2_html_text.txt|SICM extracted text]] | Identify notation and variational examples that clarify `argnums` and product-domain derivatives. |
| FDG bridge | `parsed` | [[evidence/autodiff_geometry/text/functional_differential_geometry_9580.txt|FDG extracted text]] | Start with the Prologue and notation appendix before manifold-level theorem statements. |
| Tao analysis guidance | `structure-parsed` | [[evidence/autodiff_geometry/parsed/tao_analysis_imports.tsv|Tao import inventory]] | Survey proof architecture and mathlib import choices; do not add a dependency on the repository. |

## Curriculum Layers

### Layer 0 -- Prerequisite Mathematical Substrate

Status: `available`.

Use the mathematical foundations packet as the prerequisite spine:

- finite-dimensional vector spaces and linear maps;
- inner products, duality, and transpose-like operations;
- probability and optimization vocabulary where the JAX cookbook examples use
  logistic regression or loss functions; and
- matrix calculus patterns already formalized in the first autodiff phase.

Done criteria:

- learner can state gradient, Jacobian, VJP, JVP, Hessian-vector product, and
  pullback in finite coordinates;
- Lean references point to `formalization/lean/MathematicalFoundations` or
  `formalization/lean/AutodiffGeometry`, not to informal textbook prose; and
- no Bishop or Lax source bytes are added to Harp.

### Layer 1 -- JAX Cookbook Notation

Status: `ready`.

Primary source: JAX cookbook parsed notebook and rendered text.

Work items:

- map each cookbook section to one Harp concept card;
- classify examples as `notation`, `operator identity`, `complexity guidance`,
  `container/product structure`, or `runtime behavior`;
- preserve `grad(f)`, `grad(f, i)`, `jvp`, `vjp`, `jacfwd`, `jacrev`, and
  Hessian-vector notation exactly enough that later Lean theorem names stay
  recognizable; and
- keep runtime claims out of Lean unless backed by executable example tests.

First Lean target:

- replace supplied Jacobian/Hessian data with mathlib differentiability objects
  for finite-dimensional real spaces where the API is mature enough.

### Layer 2 -- Spivak/SICM/FDG Notation Lineage

Status: `partially ready`.

Primary sources:

- JAX's direct lineage claim;
- Spivak repository outline, but not full text;
- SICM parsed open-access HTML text; and
- FDG parsed open-access PDF text.

Work items:

- extract a notation glossary from FDG Prologue and appendix material;
- connect SICM product-domain notation to JAX `argnums`;
- record exact locators for every promoted claim in the claim ledger;
- keep Spivak as structure and bibliography until a rights-cleared locator is
  available; and
- mark every synthesis row as `EVIDENCE`, `SOURCE CLAIM`, `INFERENCE`, or
  `MISSING`.

First Lean target:

- product-domain derivative projections: prove that differentiating with
  respect to one argument is composition with a product projection, then align
  that theorem with `grad(f, i)`.

### Layer 3 -- Mathlib Calculus Survey

Status: `not started`.

Primary sources:

- local mathlib dependency in the shared root `formalization/lean`;
- Lean API docs available through the pinned project; and
- Tao analysis repository import inventory as proof-engineering guidance.

Work items:

- survey `HasFDerivAt`, `fderiv`, `HasDerivAt`, `ContDiff`, continuous linear
  maps, finite-dimensional real spaces, and inner-product gradients;
- decide whether gradient should be represented through `fderiv` plus Riesz
  representation, or through a finite-coordinate matrix layer first;
- identify mathlib declarations that already prove chain rule, product rule,
  derivative of projections, bilinear maps, and symmetry of second derivatives;
- record import budget before adding broad `import Mathlib`; and
- create small Lean probes before committing theorem names to the roadmap.

First Lean target:

- a theorem that the derivative of a scalar function on `Fin n -> R` is a
  continuous linear map, then recover the finite-coordinate directional
  derivative used by JVP.

### Layer 4 -- Executable Correspondence Examples

Status: `planned`.

Primary source: JAX cookbook examples.

Work items:

- create dependency-gated examples outside Harp's default runtime until JAX
  dependency policy is approved;
- test scalar `grad`, `grad(f, i)`, JVP/VJP for affine and quadratic maps, and
  Hessian-vector product examples;
- distinguish exact algebraic identities from floating-point tolerance checks;
  and
- keep these tests as correspondence evidence, not proof of JAX internals.

First deliverable:

- a local optional lab that compares JAX numeric output against the finite
  formulas already compiled in Lean.

### Layer 5 -- Manifold and Functional Differential Geometry

Status: `evidence-gated`.

Primary source: FDG and mathlib manifold APIs after source claim extraction.

Work items:

- decide whether the formal target is smooth manifolds, finite-dimensional
  Euclidean spaces with coordinate charts, or functional differential geometry;
- avoid translating FDG notation directly into Lean until the finite-coordinate
  and product-domain layers are stable;
- survey mathlib manifold APIs before designing Harp abstractions; and
- record any missing upstream theorem as a gap rather than proving a weaker
  statement under the same name.

First Lean target:

- chart-local restatement of finite-coordinate JVP/VJP, if mathlib's manifold
  derivative APIs make that statement natural.

## Open Work Queue

| ID | Status | Slice | Done criteria |
|---|---|---|---|
| ADG-Q001 | `ready` | Build a concept card for each JAX cookbook autodiff operator. | New Markdown cards cite parsed JAX sections and update the claim ledger. |
| ADG-Q002 | `ready` | Extract FDG notation glossary from Prologue and appendix. | Glossary entries cite local FDG text locators and classify claims. |
| ADG-Q003 | `ready` | Survey mathlib calculus APIs for finite real spaces. | New survey doc lists exact imports and candidate declarations; no Lean theorem changes yet. |
| ADG-Q004 | `planned` | Formalize product projection derivative for `argnums`. | Lean compiles without `sorry`; theorem appears in the formalization roadmap. |
| ADG-Q005 | `planned` | Add optional JAX correspondence lab. | Lab is dependency-gated and excluded from default `mise run verify` unless policy changes. |
| ADG-Q006 | `blocked` | Promote Spivak direct theorem text. | Requires rights-cleared locator or explicit owner approval for source handling. |

## Fresh-Agent Checklist

1. `git status --short` and confirm you are in an isolated worktree.
2. Run `python3 evidence/autodiff_geometry/acquire.py --verify`.
3. Run `scripts/check_autodiff_geometry_lean.sh` before and after Lean edits.
4. Update this tracker row before stopping work.
5. If a tracked source or parser artifact changes, run `cargo run -p harp --
   sources verify` and refresh `docs/import-receipt.md` only after all tracked
   content has settled.

The program advances only when source evidence, curriculum prose, and compiled
Lean declarations stay synchronized.
