---
id: crouzeix-textbook-source-registry
title: Crouzeix foundations textbook source registry
type: source-registry
status: active
created: 2026-08-23
updated: 2026-08-25
tags: [crouzeix-textbook, sources, copyright, provenance]
confidence: high
canonical: source_registry.md
---

# Crouzeix foundations textbook source registry

Back to the [[knowledge/crouzeix_textbook/crouzeix_textbook_index|book index]].

This book contains newly authored exposition, examples, exercises, and
solutions. It does not reproduce source prose, figures, scans, or textbook
exercises. The entries below fix topic lineage and claim ceilings.

## LAX-2007

- Identity: Peter D. Lax, *Linear Algebra and Its Applications*, second
  edition, 2007.
- Local authority: [[knowledge/mathematical_foundations/source_registry|Mathematical Foundations source registry]].
- Role: structural linear-algebra ordering, conventional definitions, and
  high-level topic locators.
- Cannot support: a claim that this book copies Lax's presentation, or that a
  Harp theorem is correct without its own proof.

## BISHOP-2006

- Identity: Christopher M. Bishop, *Pattern Recognition and Machine Learning*,
  2006.
- Local authority: [[knowledge/mathematical_foundations/source_registry|Mathematical Foundations source registry]].
- Role: bounded machine-learning motivation and terminology.
- Cannot support: empirical performance claims or a mathematical result not
  proved here.

## SPIVAK-CALCULUS-ON-MANIFOLDS-REPO

- Identity: the public repository record pinned in the
  [[knowledge/autodiff_geometry/source_registry|Autodiff Geometry source registry]].
- Role: curriculum structure and notation lineage only.
- Boundary: source content remains gated. Broader manifold theorem text is not
  promoted from that source. The book proves independently stated
  finite-dimensional results using lawful sources and Mathlib.

## JAX-AUTODIFF-COOKBOOK

- Identity: pinned JAX documentation notebook recorded in the
  [[knowledge/autodiff_geometry/source_registry|Autodiff Geometry source registry]].
- Role: API notation and ML bridge vocabulary for later calculus chapters.
- Cannot support: JAX implementation correctness, floating-point behavior, or
  semantic identity between JAX programs and Lean declarations.

## MATHLIB-4.32.1

- Identity: Mathlib revision pinned by
  `formalization/lean/lakefile.toml` for Lean 4.32.1.
- Role: definitions, theorem interfaces, tactics, and compiled dependency
  artifacts used by the chapter formalizations.
- Boundary: compilation is relative to Lean's kernel, compiler/runtime, and
  the trusted pinned dependency surface.

## CROUZEIX-PACKET

- Identity: [[knowledge/crouzeix_conjecture/crouzeix_conjecture_index|Harp's maintained Crouzeix proof packet]].
- Role: source identities, candidate-proof interfaces, local proof receipts,
  and publication-status boundaries for Parts V--VI.
- Boundary: a locally compiled theorem is not peer review, and the packet's
  comparison language is not automatically terminology of an upstream author.
- Chapter 36 provenance: the local Harp construction in
  [[formalization/lean/Crouzeix/Harp/MainTheorem.lean|Harp MainTheorem.lean]]
  and its finite cubature, atomic dilation, and finite-horizon recurrence
  modules. This is a derived local route sharing lower-level
  Lorist–Schwenninger machinery, not a third source manuscript or a priority
  claim. Its chapter statements and exercises are exposed in
  [[formalization/lean/CrouzeixTextbook/Part06/Chapter36.lean|Chapter36.lean]].

## JIN-V4-AUDITED

- Identity: the formalization-matched Jin manuscript revision registered by
  the maintained Crouzeix proof packet.
- Role: source lineage for the positive-real completion, sample/origin
  cancellation, Gramian bridge, and constant-two endpoint in Chapters 30--32.
- Boundary: Harp's compiled theorem receipts are local verification; they do
  not replace independent publication review.

## LS-ARXIV-V1

- Identity: the pinned Lorist--Schwenninger arXiv v1 source registered by the
  maintained Crouzeix proof packet.
- Role: source lineage for the perturbation recurrence, L² boundary
  realization, and constant-two endpoint in Chapters 33--35.
- Exact source locators copied from the verified LS source graph:
  - Lemma 1: `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99`.
  - Recurrence and source Equation (3):
    `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L74-L90`.
  - Source Equation (4) and the scalar contradiction:
    `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98`.
- Status reconciliation: the
  [[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json|LS source graph]]
  remains the locator authority, but its blocked-node fields are a dated status
  snapshot. They are superseded for current local proof status by the
  maintained compiled `CrouzeixLoristSchwenninger` aggregate and its fresh
  verification gates. This local compilation does not establish peer review or publication status.
- Boundary: the book distinguishes source claims, local formalization, and
  peer-review status.
