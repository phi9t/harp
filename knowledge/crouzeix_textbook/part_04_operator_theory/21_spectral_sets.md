---
id: cft-chapter-21-spectral-sets
title: Spectral sets
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, operator-theory, mathematics, lean]
confidence: high
canonical: 21_spectral_sets.md
chapter: 21
part: 4
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 21: Spectral sets

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iv-finite-dimensional-operator-theory|Part IV — Finite-dimensional operator theory]]
Previous: [[knowledge/crouzeix_textbook/part_04_operator_theory/20_numerical_range|Chapter 20 — The numerical range]]
Next: [[knowledge/crouzeix_textbook/part_04_operator_theory/22_positive_and_completely_positive_maps|Chapter 22 — Positive and completely positive maps]]

## Opening problem

Suppose a compact set K contains the spectrum of A. When does every rational function bounded on K remain controlled after substituting A? The answer packages an entire functional-calculus inequality into one geometric phrase.

## Conceptual model

A compact K is a K₀-spectral set for A if σ(A)⊆K and ‖r(A)‖≤K₀ sup_K|r| for every rational r with poles off K. The case K₀=1 is contractive; K₀=2 is the Crouzeix endpoint for K=closure W(A). Polynomial bounds are the entry point, rational approximation extends them, and closure matters in infinite-dimensional Hilbert spaces.

## Formal development

### The six-item spine

### CFT-21-001 — operator numerical range convex {#cft-21-001}

### CFT-21-002 — closed operator numerical range {#cft-21-002}

### CFT-21-003 — closed operator numerical range nonempty {#cft-21-003}

### CFT-21-004 — closed operator numerical range compact {#cft-21-004}

### CFT-21-005 — hilbert rational spectral set statement {#cft-21-005}

### CFT-21-006 — closed numerical range two spectral set {#cft-21-006}

The checked items are CFT-21-001, CFT-21-002, CFT-21-003, CFT-21-004, CFT-21-005, CFT-21-006.

**Definition 21.1 (spectral set).** The pole condition is essential: if a denominator vanishes on σ(A), r(A) is not defined by inversion. The supremum is finite because K is compact and r continuous on K.

**Theorem 21.2 (normal case).** If A is normal, σ(A) is a 1-spectral set. Unitary diagonalization reduces the operator norm to max_{λ∈σ(A)}|r(λ)|.

**Theorem 21.3 (polynomial-to-rational passage).** Let r be pole-free near K. Approximate r uniformly on K by polynomials p_j using an appropriate approximation theorem. If ‖p_j(A)-p_k(A)‖≤C‖p_j-p_k‖_K, then p_j(A) is Cauchy. Completeness defines r(A), and passing to the limit preserves the bound. One must then identify this limit with numerator times inverse denominator.

**Theorem 21.4 (Hilbert-space transport).** Finite-dimensional compression plus strong approximation transfers a uniform finite-matrix polynomial bound to bounded operators. The closure of W(A) is nonempty, convex, bounded, and compact only in the scalar plane, even when the Hilbert space is infinite-dimensional.

The formal spine names the closed operator numerical range, proves its geometry, and exposes the exact rational and two-spectral-set predicates.

## Worked examples

**Example 1.** For normal A=diag(0,i), the unit disk is a 1-spectral set because it contains σ(A), though it is not minimal.

**Example 2.** For $J=\begin{pmatrix}0&1\\0&0\end{pmatrix}$, $\sigma(J)=\{0\}$ cannot be a finite-$K$ spectral set for polynomials: $p(z)=z$ vanishes on $\sigma(J)$ but $p(J)=J\ne0$. The numerical range disk repairs the missing derivative information.

## ML bridge

A spectral-set bound is a robust API for matrix functions: once established, every admissible filter inherits a norm guarantee. This resembles a uniform generalization bound over a function class, but it is deterministic and exact. The function class and pole exclusions are part of the theorem, not afterthoughts.

## Lean translation

CrouzeixTextbook.Part04.hilbert_rational_spectral_set_statement is a proposition-valued interface, while closed_numerical_range_two_spectral_set specializes it to one operator. The declaration names make the difference between a theorem schema and a pointwise bound explicit.

## Exercises

### CFT-21-E01 -- retrieval {#exercise-cft-21-e01}

Define a K₀-spectral set.

### CFT-21-E02 -- calculation {#exercise-cft-21-e02}

Show that enlarging K preserves a K₀-spectral-set inequality when poles remain excluded.

### CFT-21-E03 -- written-proof {#exercise-cft-21-e03}

Prove the normal 1-spectral-set result.

### CFT-21-E04 -- written-proof {#exercise-cft-21-e04}

Explain the Cauchy-sequence step in polynomial-to-rational extension.

### CFT-21-E05 -- boundary {#exercise-cft-21-e05}

Give an example showing σ(A) alone fails for a Jordan block.

### CFT-21-E06 -- lean-proof {#exercise-cft-21-e06}

Inspect closed_numerical_range_two_spectral_set and list its quantified rational data.

### Solution sketches

For (2), sup over the larger set dominates. For (3), diagonalize unitarily. For (4), apply the polynomial bound to p_j-p_k. For (5), use p(z)=z on a nilpotent block. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part04.closed_numerical_range_two_spectral_set`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter’s sixth contract declaration is a navigation handoff to Chapter 22. The compiled interfaces expose related formal statements; because correspondence remains summary/checkpoint or unmapped, they do not certify every analytic step above.
