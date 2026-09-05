---
id: cft-chapter-14-sequences-and-series-of-operators
title: Sequences and series of operators
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, analysis-and-complex-functions, mathematics, lean]
confidence: high
canonical: 14_sequences_and_series_of_operators.md
chapter: 14
part: 3
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 14: Sequences and series of operators

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iii-analysis-and-complex-functions|Part III — Analysis and complex functions]]
Previous: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/13_metric_and_normed_spaces|Chapter 13 — Metric and normed spaces]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/15_complex_differentiability|Chapter 15 — Complex differentiability]]

## Opening problem

Why does (I-zT)⁻¹ expand as I+zT+z²T²+⋯, and exactly which inequality makes the expansion legal? The formula is algebraically suggestive, but the proof is analytic: partial sums telescope, while a norm estimate forces the remainder to zero.

## Conceptual model

An operator series is handled in two layers. Algebra identifies the finite partial-sum identity. Analysis proves convergence in a complete normed space. Keeping these layers separate is especially useful in noncommutative settings: the geometric series works because every term is a power of the same operator, not because arbitrary matrices commute.

Uniform convergence is the right mode for exchanging a limit with evaluation, integration, or another bounded linear operation. Absolute convergence, expressed as ∑‖Aₖ‖<∞, is a convenient sufficient condition in Banach spaces.

## Formal development

### The six-item spine

### CFT-14-001 — matrix power series {#cft-14-001}

### CFT-14-002 — matrix power series coefficient {#cft-14-002}

### CFT-14-003 — matrix power series radius {#cft-14-003}

### CFT-14-004 — matrix power series sum {#cft-14-004}

### CFT-14-005 — matrix power series analytic {#cft-14-005}

### CFT-14-006 — matrix power series converges {#cft-14-006}

The formal items are CFT-14-001, CFT-14-002, CFT-14-003, CFT-14-004, CFT-14-005, CFT-14-006.

**Theorem 14.1 (Banach-valued absolute convergence).** If X is complete and ∑‖xₖ‖ converges, then ∑xₖ converges. A tail norm is bounded by the scalar tail ∑‖xₖ‖, so partial sums are Cauchy.

**Theorem 14.2 (Neumann series).** If ‖T‖<1, then I-T is invertible and (I-T)⁻¹=∑ₖ₌₀∞Tᵏ. For S_N=∑ₖ₌₀ᴺTᵏ, multiplication gives (I-T)S_N=S_N(I-T)=I-Tᴺ⁺¹. Submultiplicativity yields ‖Tᴺ⁺¹‖≤‖T‖ᴺ⁺¹→0. Passing to the limit provides both inverse identities.

**Corollary 14.3 (resolvent expansion).** If |z|‖T‖<1, then (I-zT)⁻¹=∑zᵏTᵏ. This local expansion later becomes a matrix-valued analytic function.

**Theorem 14.4 (continuity of inversion).** If A is invertible and ‖A⁻¹(B-A)‖<1, write B=A(I+A⁻¹(B-A)) and use the Neumann series. Thus invertibility is open and B⁻¹→A⁻¹ when B→A through this neighborhood.

The compiled spine CFT-14-001--006 packages matrix power series, coefficients, radius, sum, analyticity, and convergence on the unit disk.

## Worked examples

**Example 1.** For $T=\begin{pmatrix}0&1\\0&0\end{pmatrix}$, $T^2=0$. Hence $(I-zT)^{-1}=I+zT$ for every $z$, not merely for $|z|\lVert T\rVert<1$. The norm condition is sufficient, not necessary.

**Example 2.** For T=diag(1/2,1/3), ∑Tᵏ=diag(2,3/2). Multiplying by I-T verifies the result. The slowest scalar mode, 1/2, controls the tail bound ‖∑_{k>N}Tᵏ‖≤(1/2)^{N+1}/(1-1/2).

## ML bridge

Deep equilibrium models and implicit layers differentiate through (I-J)⁻¹. A truncated Neumann series is justified only when the relevant Jacobian is contractive in the chosen norm; spectral radius below one alone may require a different equivalent norm and can hide poor conditioning. This chapter supplies the exact analytic contract behind such approximations.

## Lean translation

CrouzeixTextbook.Part03.Chapter14 re-exports the maintained matrixPowerSeries construction. #check CrouzeixTextbook.Part03.matrix_power_series_converges displays the coefficient bound and disk hypothesis used by the proof; no informal interchange of an infinite sum and matrix evaluation is assumed.

## Exercises

### CFT-14-E01 -- retrieval {#exercise-cft-14-e01}

State absolute convergence for a Banach-valued series.

### CFT-14-E02 -- calculation {#exercise-cft-14-e02}

Compute ∑Tᵏ for T=diag(1/4,-1/2).

### CFT-14-E03 -- written-proof {#exercise-cft-14-e03}

Prove the finite identity (I-T)∑_{k=0}^N T^k=I-T^{N+1}.

### CFT-14-E04 -- written-proof {#exercise-cft-14-e04}

Prove the Neumann-series inverse theorem from that identity.

### CFT-14-E05 -- boundary {#exercise-cft-14-e05}

Give an operator with spectral radius zero but arbitrarily large norm.

### CFT-14-E06 -- lean-proof {#exercise-cft-14-e06}

Inspect matrix_power_series_radius and identify why a coefficient bound yields radius at least one.

### Solution sketches

For (2), sum the two scalar geometric series. For (3), expand and cancel adjacent powers. For (4), use completeness plus $\lVert T^k\rVert\le\lVert T\rVert^k$ and take limits on both sides. For (5), use $T=\begin{pmatrix}0&M\\0&0\end{pmatrix}$. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part03.matrix_power_series_radius`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

Chapter 14 turns the preceding finite-dimensional geometry into a reusable analytic interface. Its last compiled item feeds directly into Chapter 15, while the distinctions between algebraic identity, convergence, and positivity remain visible for both constant-two routes.
