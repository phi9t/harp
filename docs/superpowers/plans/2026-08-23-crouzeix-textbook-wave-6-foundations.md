# Crouzeix Textbook Wave 6 Lax-Style Foundations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete Chapters 1–12 as an engaging, self-contained Peter Lax-style route through linear structure, Euclidean geometry, multilinearity, and calculus, calibrated for mathematically mature ML researchers.

**Architecture:** Use Chapter 1's object-before-coordinates voice as the baseline, then deepen each six-item CFT slice with topic-specific motivation, exact proofs, counterexamples, the cumulative matrix family `A_{λ,α}`, and distinct Lean exercises. The foundations must support later chapters without reading like a compressed prerequisites list.

**Tech Stack:** Markdown, Lean 4.32.1 and Mathlib 4.32.1, existing MathematicalFoundations/Crouzeix modules, Rust publisher, JSON v2 contracts, mise.

---

## Task 1: Freeze the foundations editorial and formal contract

- [ ] Add RED tests requiring 72 unique CFT anchors, 72 distinct exercise
  solutions, exact theorem cards, and no repeated generic motivation/ML
  paragraphs across Chapters 1–12.
- [ ] Preserve all public names and Chapter 1's existing six solution theorems;
  require new local solution declarations for Chapters 2–12.
- [ ] Require `A_{λ,α}` calculations in every chapter, each tied to that
  chapter's mathematical object rather than repeated as decoration.
- [ ] Add a foundations editorial fixture that checks labeled Motivation,
  Historical context, and the four ML analogy fields without imposing word
  counts.
- [ ] Commit as `test(crouzeix-textbook): freeze Lax-style foundations depth`.

## Task 2: Chapter 1 — objects before representations

- [ ] Retain the chapter's successful exposition but upgrade CFT-01-001..006
  into exact theorem cards: linear transformation, basis-image columns,
  coordinate action, change of basis, characteristic-polynomial invariance,
  and the nonunitary-similarity norm counterexample.
- [ ] Prove every coordinate identity from the universal property of a basis;
  distinguish invariant operator facts from basis-dependent matrix norms.
- [ ] Introduce `A_{λ,α}` as one operator represented in different bases and
  compute how nonunitary similarity changes Euclidean lengths.
- [ ] Verify existing `Exercises.Chapter01.exercise_01_solution` through 06
  match distinct exercise statements and update fingerprints.

## Task 3: Chapter 2 — vector spaces, subspaces, quotients, direct sums

- [ ] Prove span minimality, intersection membership, coordinate uniqueness,
  dimension invariance, direct-sum coordinate uniqueness, and quotient
  projection surjectivity CFT-02-001..006.
- [ ] Build the proofs from maps and universal properties; explain quotient
  objects before quotient coordinates.
- [ ] For `A_{λ,α}`, identify invariant eigenspace, generalized eigenspace, a
  complementary subspace when available, and the quotient action.
- [ ] Add six distinct Chapter 2 solutions; the Lean exercise proves a direct
  sum uniqueness statement from intersection-zero.

## Task 4: Chapter 3 — kernels, ranges, and exact structure

- [ ] Prove kernel/range membership, rank--nullity, injectivity iff trivial
  kernel, invariant restriction well-definedness, and composition range
  containment CFT-03-001..006.
- [ ] Give the exact-sequence reading of rank--nullity without assuming
  category theory; show every finite-dimensional hypothesis.
- [ ] Calculate kernels/ranges of `A_{0,α}`, its powers, and an invariant
  restriction.
- [ ] Add six distinct solutions and verify `chapter_03_`.

## Task 5: Chapter 4 — coordinates and duality

- [ ] Prove dual-map evaluation, contravariant composition, coordinate
  conjugacy, characteristic-polynomial, determinant, and trace invariance
  CFT-04-001..006.
- [ ] Derive inverse-transpose/conjugate-transpose coordinate rules from the
  pairing, with field assumptions explicit.
- [ ] Compare primal activation propagation and dual covector/backpropagation
  for `A_{λ,α}`; exact transfer is the algebraic transpose rule, not a claim
  about gradient statistics.
- [ ] Add six distinct solutions and verify `chapter_04_`.

## Task 6: Chapter 5 — determinant, trace, and exterior algebra

- [ ] Prove determinant multiplicativity/diagonal/similarity and trace
  cyclicity/similarity/diagonal CFT-05-001..006.
- [ ] Motivate determinant through top exterior power and trace through
  infinitesimal volume scaling; give a complete finite-dimensional bridge from
  those invariant descriptions to coordinates.
- [ ] Compute both invariants for `A_{λ,α}` and explain why neither detects
  nonnormal coupling α.
- [ ] Add six distinct solutions and verify `chapter_05_`.

## Task 7: Chapter 6 — eigenvalues and polynomial algebra

- [ ] Prove polynomial action on a diagonal matrix, diagonalizable functional
  calculus, spectrum membership, generated-algebra containment and
  representation, and density of simple spectrum CFT-06-001..006.
- [ ] Distinguish algebraic/geometric multiplicity and show the Jordan
  derivative term `p(A_{λ,α})` explicitly.
- [ ] Add a boundary example where eigenvalues alone fail to control norms.
- [ ] Add six distinct solutions and verify `chapter_06_`.

## Task 8: Part I checkpoint

```sh
cargo test -p harp --test crouzeix_textbook part_01_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
```

- [ ] Editorially reconstruct rank--nullity, dual coordinate action,
  determinant/trace invariance, and polynomial Jordan action without external
  sources. Commit Chapters 1–6 in chapter-scoped commits, then regenerate once.

## Task 9: Chapter 7 — inner-product geometry

- [ ] Prove orthogonal projection decomposition and orthogonality, adjoint
  coordinate identity, conjugate-transpose representation, matrix/operator
  norm agreement, and norm nonnegativity CFT-07-001..006.
- [ ] Derive the projection theorem in finite dimensions and define the adjoint
  by its universal pairing property before writing matrices.
- [ ] Compute `A*`, orthogonal projections, and one norm lower bound for
  `A_{λ,α}`.
- [ ] Add six distinct solutions and verify `chapter_07_`.

## Task 10: Chapter 8 — positivity and Gram geometry

- [ ] Prove quadratic positivity, Gram PSD, invertible Gram matrix properties,
  positivity under congruence, and positive square-root data CFT-08-001..006.
- [ ] Prove equivalence of Gram invertibility and vector independence and show
  precisely when PSD strengthens to positive definite.
- [ ] Compute a two-vector Gram matrix derived from the columns of
  `A_{λ,α}` and diagnose conditioning.
- [ ] Add six distinct solutions and verify `chapter_08_`.

## Task 11: Chapter 9 — operator norms and singular values

- [ ] Prove induced matrix/operator norm identity, polar factor unitarity,
  polar similarity and norm transfer, the quadratic-bound-to-two lemma, and
  the triangle kernel CFT-09-001..006.
- [ ] Derive singular values through `A*A`, prove norm attainment in finite
  dimensions, and explain why spectral radius can be much smaller than norm.
- [ ] Compute the exact singular values of `A_{λ,α}` and specialize to
  `A_{0,2}`.
- [ ] Add six distinct solutions and verify `chapter_09_`.

## Task 12: Chapter 10 — multilinear maps and tensors

- [ ] Prove pairing duality, transpose coordinate action, top-degree
  determinant behavior, alternating volume, contraction/trace cyclicity, and
  wedge-sign identity CFT-10-001..006.
- [ ] Use universal multilinear properties before tensor coordinates; explain
  variance and contraction carefully for readers familiar with ML tensor APIs.
- [ ] The ML analogy maps mathematical tensor contraction to indexed array
  contraction exactly, then states the non-transfer boundary: storage layout,
  broadcasting, and autodiff semantics are implementation conventions.
- [ ] Add six distinct solutions and verify `chapter_10_`.

## Task 13: Chapter 11 — differentiation as linear approximation

- [ ] Prove gradient component, JVP, VJP, Hessian-vector action, JVP/VJP
  duality, and chain-rule kernel CFT-11-001..006.
- [ ] Define Fréchet differentiability with its remainder estimate; derive the
  chain rule and adjoint pullback rather than presenting API mnemonics.
- [ ] Use `x↦A_{λ,α}x` as the exact linear case and a nonlinear perturbation to
  show what a local Jacobian does and does not predict.
- [ ] Add six distinct solutions and verify `chapter_11_`.

## Task 14: Chapter 12 — differential forms, orientation, and Stokes

- [ ] Prove real inner product in complex coordinates, area form under quarter
  turn, radial/tangent positivity, tangent formula, boundary smoothness, and
  slice derivative CFT-12-001..006.
- [ ] Build from alternating forms to line integrals and the planar Stokes
  theorem used for positively oriented convex boundaries. Audit sign and
  orientation conventions against the later Cauchy layers.
- [ ] Parameterize the circular numerical range of `A_{0,2}` and verify the
  tangent/normal formulas directly.
- [ ] Add six distinct solutions and verify `chapter_12_`.

## Task 15: Historical and ML continuity

- [ ] Use exact Lax editions/locators for historical or pedagogical attribution;
  mark broad motivation as context, not as a source claim.
- [ ] Make the cumulative ML thread explicit: representation → invariant
  subspace → dual action → weak invariants → polynomial response → metric
  geometry → conditioning → norm amplification → tensor contraction →
  differentiation → oriented integration.
- [ ] Every analogy includes exact transfer, non-transfer, and a calculation;
  remove any analogy that cannot meet all four fields.

## Task 16: Publish, verify, and declare whole-book completion

- [ ] Run the Part II checkpoint:

```sh
cargo test -p harp --test crouzeix_textbook part_02_ -- --test-threads=1
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
```

- [ ] Run a whole-book contract audit. Only now change overall status to exact
  completion, and only if all 210 theorem rows and all 210 exercise rows meet
  the approved completion contract.
- [ ] Regenerate all ledgers, corpus JSON, and Atlas HTML and run:

```sh
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-jin
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-ls
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise run lean-crouzeix-textbook
PATH=/opt/homebrew/bin:$PATH mise run verify
git diff --check
git status --short
```

- [ ] Conduct an editorial review from Chapter 1 through both endpoints,
  sampling every proof dependency and all 35 ML diagnostics. Compilation is
  not a substitute for this review.
- [ ] Refresh `docs/import-receipt.md` last and commit it separately. Wave 6
  exits only when all 35 chapters are active, all applicable rows are exact,
  all formal exercises have distinct compiled solutions, and both provider
  branches remain independent.
