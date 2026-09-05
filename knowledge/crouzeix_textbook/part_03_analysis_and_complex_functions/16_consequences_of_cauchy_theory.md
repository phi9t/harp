---
id: cft-chapter-16-consequences-of-cauchy-theory
title: Consequences of Cauchy theory
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, analysis-and-complex-functions, mathematics, lean]
confidence: high
canonical: 16_consequences_of_cauchy_theory.md
chapter: 16
part: 3
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 16: Consequences of Cauchy theory

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iii-analysis-and-complex-functions|Part III — Analysis and complex functions]]
Previous: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/15_complex_differentiability|Chapter 15 — Complex differentiability]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/17_functions_of_matrices_and_operators|Chapter 17 — Functions of matrices and operators]]

## Opening problem

Why can a holomorphic function not hide a strict interior maximum, and why does knowing all its values on a tiny accumulating set determine it everywhere on a connected domain? Cauchy’s formula turns local boundary data into global rigidity.

## Conceptual model

Cauchy’s formula says the center value is an average of boundary values. Equality in the triangle inequality is therefore rigid: an interior maximum forces all boundary contributions to align, and the function becomes locally constant. Analytic continuation then propagates that constancy.

Zeros of a nonzero holomorphic function are isolated because the first nonzero Taylor coefficient factors the function as (z-a)^m g(z), with g(a)≠0. This factorization powers the identity theorem and residue calculus.

## Formal development

### The six-item spine

### CFT-16-001 — holomorphic matrix eval {#cft-16-001}

### CFT-16-002 — contour eval agrees {#cft-16-002}

### CFT-16-003 — polynomial compatibility {#cft-16-003}

### CFT-16-004 — locality on neighborhood {#cft-16-004}

### CFT-16-005 — functional calculus additive {#cft-16-005}

### CFT-16-006 — functional calculus multiplicative {#cft-16-006}

The formal items are CFT-16-001, CFT-16-002, CFT-16-003, CFT-16-004, CFT-16-005, CFT-16-006.

**Theorem 16.1 (maximum modulus).** A nonconstant holomorphic function on a connected open set has no interior local maximum of |f|. If |f(a)| dominates a surrounding disk, Cauchy’s formula and the mean-value inequality force equality almost everywhere on every small circle. Equality in the complex triangle inequality makes f constant on that circle, hence in the disk, hence throughout the component.

**Theorem 16.2 (identity theorem).** If zeros of f accumulate inside a connected domain, then f=0. At an accumulation point, an isolated-zero factorization is impossible unless every Taylor coefficient vanishes; local vanishing propagates through the connected domain.

**Theorem 16.3 (open mapping).** A nonconstant holomorphic map sends open sets to open sets. Near z₀, factor f(z)-f(z₀)=(z-z₀)^m g(z). On a small circle the leading behavior controls perturbations, and the argument principle supplies nearby preimages.

**Theorem 16.4 (residue theorem).** For isolated singularities a_j inside Γ, ∮Γ f=2πi∑Res(f,a_j). Remove small disks, apply Cauchy’s theorem on the punctured region, and evaluate each small-circle integral from the Laurent coefficient of (z-a_j)^{-1}.

The Lean spine emphasizes the consequence needed later: a contour-defined matrix evaluation is local to a neighborhood, agrees with polynomial evaluation, and preserves addition and multiplication.

## Worked examples

**Example 1.** If a polynomial p satisfies |p(z)|≤1 for |z|≤1 and |p(0)|=1, maximum modulus makes p constant. Merely knowing |p(0)|<1 gives no such conclusion.

**Example 2.** For f(z)=1/[z(z-1)], the residues at 0 and 1 are -1 and 1. A contour enclosing both has integral zero; a contour enclosing only 0 has integral -2πi. Geometry determines which local coefficients contribute.

## ML bridge

Analytic activations are extremely rigid: exact agreement on a set with an interior accumulation point forces global agreement on a connected domain. That is unlike generic neural networks, where interpolation constraints leave enormous freedom. In this book, the useful payoff is stable approximation on shrinking outer neighborhoods and algebraic compatibility of holomorphic matrix evaluation.

## Lean translation

CrouzeixTextbook.Part03.Chapter16 exposes #check CrouzeixTextbook.Part03.locality_on_neighborhood and the additive/multiplicative laws. The formal theorem does not define f(A) by choosing eigenvectors; it identifies the limit-based calculus with any admissible fixed contour and proves independence through locality.

## Exercises

### CFT-16-E01 -- retrieval {#exercise-cft-16-e01}

State the maximum-modulus principle in local and compact-domain forms.

### CFT-16-E02 -- calculation {#exercise-cft-16-e02}

Find the residues of 1/(z²-1).

### CFT-16-E03 -- written-proof {#exercise-cft-16-e03}

Prove that zeros of a nonzero polynomial cannot accumulate.

### CFT-16-E04 -- written-proof {#exercise-cft-16-e04}

Prove the identity theorem from isolated zeros.

### CFT-16-E05 -- boundary {#exercise-cft-16-e05}

Show why connectedness is necessary in the identity theorem.

### CFT-16-E06 -- lean-proof {#exercise-cft-16-e06}

Use functional_calculus_multiplicative to identify the Lean law corresponding to (fg)(A)=f(A)g(A).

### Solution sketches

For (2), factor (z-1)(z+1) and take simple-pole limits. For (3), use degree or the fundamental theorem of algebra. For (4), factor at an accumulation point and propagate the open-and-closed zero region. For (5), define different constants on two components. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part03.functional_calculus_multiplicative`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

Chapter 16 turns the preceding finite-dimensional geometry into a reusable analytic interface. Its last compiled item feeds directly into Chapter 17, while the distinctions between algebraic identity, convergence, and positivity remain visible for both constant-two routes.
