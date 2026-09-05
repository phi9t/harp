# Crouzeix Textbook Wave 5 Analysis and Operator Theory Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Chapters 13–24 a self-contained analytic and operator-theoretic bridge from normed-space convergence through holomorphic functional calculus, numerical ranges, positivity, dilation, and Gramians.

**Architecture:** Upgrade twelve chapters in two six-chapter batches. Each preserved six-item CFT slice gets exact prose, truthful Lean provenance, six distinct formal solutions, and the running nonnormal example. Chapter dependencies are authored pedagogically; kernel dependencies continue to come only from compiled receipts.

**Tech Stack:** Markdown, Lean 4.32.1 and Mathlib 4.32.1, existing Crouzeix analysis/operator modules, Rust publisher, JSON v2 contracts, mise.

---

## Task 1: Freeze the Part III–IV completion tests

- [ ] Add RED tests requiring 72 unique CFT anchors, 72 distinct solution
  declarations, all theorem-card fields, and chapter-specific motivation rather
  than repeated generic paragraphs.
- [ ] Add semantic correspondence tests for Chapter 15: the displayed
  Cauchy--Riemann, Cauchy formula, and derivative estimate claims must have
  exact Lean rows or be explicitly marked context; downstream contour aliases
  cannot satisfy them.
- [ ] Require the running example `A_{λ,α}` in Chapters 13, 14, 16, 17, 19,
  20, 21, 23, and 24, with a different calculation in each.
- [ ] Commit as `test(crouzeix-textbook): freeze analysis and operator depth`.

## Task 2: Chapter 13 — metric, normed, and compact foundations

- [ ] Prove compact maximum existence, pointwise domination, nonnegativity,
  monotonicity, and convergence of outer maxima for CFT-13-001..006.
- [ ] Define sequences, Cauchy sequences, completeness, compactness, and uniform
  convergence before use; prove the finite-dimensional compactness facts the
  book relies on or link exact Mathlib theorems.
- [ ] Compute `max_{z∈W(A_{λ,α})}|p(z)|` for a simple affine `p`.
- [ ] Add six distinct `Exercises.Chapter13` solutions and verify `chapter_13_`.

## Task 3: Chapter 14 — series of operators

- [ ] Define matrix power series and radius; prove coefficient, radius,
  analytic-sum, and convergence claims CFT-14-001..006 with explicit norm
  majorants and completeness.
- [ ] Separate pointwise, uniform-on-compacts, and operator-norm convergence.
  Show exactly which mode permits limits through addition, multiplication, and
  evaluation.
- [ ] Sum the resolvent Neumann series for the nilpotent running example and
  show where nonnormal amplification appears despite spectral radius zero.
- [ ] Replace aliases with exact reexports/local bridges, add six solutions,
  and verify `chapter_14_`.

## Task 4: Chapter 15 — complex differentiability and Cauchy theory

- [ ] Repair the audit mismatch: give exact CFT cards and Lean declarations for
  complex differentiability, the Cauchy--Riemann implication actually used,
  contour integration, Cauchy's formula, and derivative estimates. If an
  existing CFT row currently names a downstream claim, keep its ID but change
  its exact statement/anchor and preserve the public Lean name through a local
  theorem of the new type only when compatibility is possible; otherwise
  record a design-spec migration note before changing a public type.
- [ ] Derive the parametric boundary integral from ordinary contour notation,
  audit orientation and `2πi`, and prove continuity with a uniform majorant.
- [ ] Include a complete circle calculation and a boundary integral for
  `A_{λ,α}`.
- [ ] Add six distinct solutions and verify both `chapter_15_` and type
  fingerprint migration tests.

## Task 5: Chapter 16 — consequences of Cauchy theory

- [ ] Define holomorphic matrix evaluation; prove contour agreement,
  polynomial compatibility, locality, additivity, and multiplicativity for
  CFT-16-001..006.
- [ ] Show the resolvent identity and double-contour calculation underlying
  multiplicativity instead of calling it standard.
- [ ] Compute `f(A_{λ,α})=[[f(λ),αf'(λ)],[0,f(λ)]]` from the contour formula.
- [ ] Add six distinct solutions and verify `chapter_16_`.

## Task 6: Chapter 17 — rational and holomorphic operator functions

- [ ] Define finite pole sets and pole-free domains; prove finiteness,
  openness, scalar evaluation, and matrix evaluation CFT-17-001..006.
- [ ] Prove independence from a chosen numerator/denominator representation and
  connect rational evaluation to the holomorphic calculus.
- [ ] Evaluate a resolvent rational function on `A_{λ,α}` and compare spectral
  distance with operator norm.
- [ ] Add six distinct solutions and verify `chapter_17_`.

## Task 7: Chapter 18 — positive-real analytic functions

- [ ] Define the disk, circle, sampled kernel, and matrix-kernel positivity for
  CFT-18-001..004.
- [ ] Derive the matrix Herglotz kernel and prove positivity CFT-18-005/006,
  distinguishing positive real part from positive sampled Pick matrices.
- [ ] Give a two-point scalar kernel calculation and a 2×2 matrix-valued
  calculation. Identify precisely what finite sampling can and cannot certify.
- [ ] Add six distinct solutions and verify `chapter_18_`.
- [ ] Run the Part III checkpoint:

```sh
cargo test -p harp --test crouzeix_textbook part_03_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
```

- [ ] Commit Chapters 13–18 as focused chapter commits plus one generated-state
  commit.

## Task 8: Chapter 19 — normality, diagonalization, and approximation

- [ ] Define normal and nonnormal operators and prove the simple-spectrum
  approximation properties CFT-19-001..005 with an explicit perturbation and
  norm convergence.
- [ ] Prove diagonal polynomial action CFT-19-006 and contrast unitary
  diagonalization with arbitrary similarity.
- [ ] Use `A_{λ,α}` to calculate eigenvectors, defectiveness, commutator
  `A*A-AA*`, and transient powers.
- [ ] Add six distinct solutions and verify `chapter_19_`.

## Task 9: Chapter 20 — numerical range

- [ ] Define `W(A)`, prove nonemptiness, sphere-image representation,
  compactness, convexity, perturbation stability, and spectral containment for
  CFT-20-001..006.
- [ ] Give a reconstructible Toeplitz--Hausdorff proof at the finite-dimensional
  level used later; do not merely cite convexity.
- [ ] Derive the ellipse/disk numerical range of `A_{λ,α}` and identify the
  normalized Jordan extremizer.
- [ ] Add six distinct solutions and verify `chapter_20_`.

## Task 10: Chapter 21 — spectral sets

- [ ] Define closed numerical range and rational spectral-set constants; prove
  nonemptiness, compactness, and the exact two-spectral-set statement
  CFT-21-001..006.
- [ ] Separate spectrum containment, scalar supremum, rational pole
  restrictions, and operator norm. Give counterexamples showing none may be
  silently omitted.
- [ ] Calculate the spectral-set ratio for `A_{0,2}` and `r(z)=z`.
- [ ] Add six distinct solutions and verify `chapter_21_`.

## Task 11: Chapter 22 — positive and completely positive maps

- [ ] Define the boundary positive map and prove linearity, norm, unitality,
  star preservation, and PSD preservation CFT-22-001..006.
- [ ] State exactly where mere positivity suffices and where matrix-level or
  complete positivity would be stronger; do not attribute an unproved CP
  property to the current declaration.
- [ ] Prove norm control for a unital positive map at the finite-dimensional
  scope actually used by the Crouzeix machinery.
- [ ] Add six distinct solutions and verify `chapter_22_`.

## Task 12: Chapter 23 — compression and dilation

- [ ] Define compressed adjoint powers and prove contraction, doubled
  compression, adjacent-power defect, commutation, and target power bounds
  CFT-23-001..006.
- [ ] Derive `P_H U^n|_H` identities with domains/codomains visible and explain
  why equality of the first moment does not imply equality of all powers.
- [ ] Give a finite block-matrix dilation for a contraction and compare it with
  `A_{λ,α}`'s nonnormal compression.
- [ ] Add six distinct solutions and verify `chapter_23_`.

## Task 13: Chapter 24 — Gramians and matrix order

- [ ] Define Gramian terms and prove positivity, summability, weighted sums,
  weighted positivity, and difference positivity CFT-24-001..006.
- [ ] Show quadratic-form/matrix-order equivalence, PSD preservation under
  congruence, and the exact conditions for removing a positive tail.
- [ ] Compute a finite-horizon Gramian for `A_{λ,α}` and interpret its largest
  direction as transient amplification, not as a theorem about trained models.
- [ ] Add six distinct solutions and verify `chapter_24_`.

## Task 14: Labeled context and editorial integration

- [ ] Every chapter receives source-bound historical context. Textbook
  derivations are not misclassified as historical evidence.
- [ ] Every chapter's four-field ML analogy uses one of: convergence of
  iterative linearizations, Jacobian resolvents, feature-kernel positivity,
  nonnormal transient growth, compression of latent dynamics, or Gramians.
  Each names an exact calculation and a non-transfer boundary.
- [ ] Review prerequisite flow from Chapter 13 to 24 and forward links into
  Chapters 25–35. Remove generic paragraph reuse found by phrase search.

## Task 15: Publish and freeze Wave 5

```sh
cargo test -p harp --test crouzeix_textbook part_03_ -- --test-threads=1
cargo test -p harp --test crouzeix_textbook part_04_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
PATH=/opt/homebrew/bin:$PATH mise run verify
git diff --check
```

Regenerate all outputs, editorially reconstruct the Cauchy/functional-calculus,
numerical-range, dilation, and Gramian proofs, refresh the import receipt last,
and commit it separately. Wave 5 exits only when all 72 rows are
reconstructible/exact.
