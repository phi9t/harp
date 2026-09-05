---
id: cft-chapter-08-positive-operators-and-gram-geometry
title: Positive operators and Gram geometry
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 08_positive_operators_and_gram_geometry.md
chapter: 8
part: 2
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 8: Positive operators and Gram geometry

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/07_inner_product_spaces|Chapter 7 — Inner-product spaces]]
Next: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/09_operator_norms_and_singular_values|Chapter 9 — Operator norms and singular values]]

## Opening problem

How do we turn linear structure into quantitative geometry? This chapter develops Gram matrices, congruence, positive square roots, and Cholesky factors. The goal is not a catalogue of formulas but a chain of proofs whose hypotheses remain visible.

## Conceptual model

Algebra says which combinations are legal. Geometry adds a rule for measuring size, angle, orientation, or first-order change. Each new identification is extra structure: a covector is not automatically a vector, a transpose is not automatically an adjoint, and a coordinate formula is not automatically basis-free.

## Formal development

### The six-item spine

### CFT-08-001 — positive definite quadratic positive {#cft-08-001}

### CFT-08-002 — gram matrix positive {#cft-08-002}

### CFT-08-003 — invertible gram matrix invertible {#cft-08-003}

### CFT-08-004 — invertible gram matrix positive definite {#cft-08-004}

### CFT-08-005 — positivity preserved by congruence {#cft-08-005}

### CFT-08-006 — positive square root data {#cft-08-006}

The chapter tracks six formal items, `CFT-08-001`, `CFT-08-002`, `CFT-08-003`, `CFT-08-004`, `CFT-08-005`, `CFT-08-006`. For columns $s_i$, the Gram matrix $G=S^*S$ satisfies $c^*Gc=\|Sc\|^2\ge0$. If $S$ is invertible this is positive for nonzero $c$. Congruence preserves positivity because $x^*(R^*GR)x=(Rx)^*G(Rx)$. Diagonalizing a positive Hermitian matrix and taking nonnegative scalar square roots constructs the unique positive square root; on its support this yields Cholesky and inverse-square-root factors.

The proof discipline is consistent: expand the relevant definition, isolate the term controlled by positivity, orthogonality, multilinearity, or the small-o remainder, and only then invoke finite-dimensional compactness or coordinates. This prevents a later operator inequality from silently assuming normality or commutativity.

### Proof workshop: positivity as geometry

A Hermitian matrix $G$ is positive semidefinite when
$x^*Gx\ge0$ for every $x$. If $G=S^*S$, then
$x^*Gx=\lVert Sx\rVert^2$, so every Gram matrix is PSD. If $S$ is
invertible and $x\ne0$, then $Sx\ne0$ and the inequality is strict; hence
$S^*S$ is positive definite and invertible.

Conversely, the spectral theorem writes a Hermitian PSD matrix as
$G=U\operatorname{diag}(\lambda_i)U^*$ with $\lambda_i\ge0$. Define
$G^{1/2}=U\operatorname{diag}(\sqrt{\lambda_i})U^*$. Then
$G^{1/2}$ is PSD and squares to $G$. Uniqueness follows because any PSD
square root commutes with $G$ and agrees on its spectral subspaces.

Congruence is the natural transport law. From $G\ge0$,
$S^*GS\ge0$ because $x^*S^*GSx=(Sx)^*G(Sx)$. Similarity and congruence must
not be confused: similarity preserves spectrum, while congruence preserves
quadratic-form order. The completion proof will repeatedly move between these
two roles.

## Worked examples

**Example 1.** For vectors $v_1=(1,0)$ and $v_2=(1,1)$, the Gram
matrix is
$G=\begin{pmatrix}1&1\\1&2\end{pmatrix}$. Its determinant is $1$ and its
leading diagonal entry is positive, so $G$ is positive definite. The
quadratic form is exactly $\lVert a_1v_1+a_2v_2\rVert^2$.

**Example 2.** The Hermitian matrix
$\begin{pmatrix}1&2\\2&1\end{pmatrix}$ is not PSD: testing on $(1,-1)$
gives $-2$. Positive diagonal entries alone are therefore insufficient.

## ML bridge

Feature covariance and kernel Gram matrices are positive by construction. Regularization changes the operator before factorization; it is not part of the exact identity.

## Lean translation

`CrouzeixTextbook.Part02.Chapter08` exposes six checked declarations. They reuse maintained Harp or Mathlib results only at matching statement boundaries. For Chapters 11--12, the formal scope is deliberately finite-coordinate or planar; it does not pretend to formalize a JAX runtime or the full manifold-level Stokes theorem.

## Exercises

### CFT-08-E01 -- retrieval {#exercise-cft-08-e01}

Define PSD and positive definite matrices by quadratic forms.

### CFT-08-E02 -- calculation {#exercise-cft-08-e02}

Compute the Gram matrix of $(1,0)$ and $(1,1)$.

### CFT-08-E03 -- written-proof {#exercise-cft-08-e03}

Prove every Gram matrix is PSD.

### CFT-08-E04 -- written-proof {#exercise-cft-08-e04}

Prove $S^*S$ is positive definite when $S$ is invertible.

### CFT-08-E05 -- boundary {#exercise-cft-08-e05}

Give a Hermitian matrix with positive diagonal that is not PSD.

### CFT-08-E06 -- lean-proof {#exercise-cft-08-e06}

Use positivity_preserved_by_congruence to explain why no invertibility of $S$ is needed.

### Solution sketches

For (2), obtain $\begin{pmatrix}1&1\\1&2\end{pmatrix}$. For (3), rewrite the quadratic form as a squared norm. For (4), strict positivity follows from $Sx\ne0$. For (5), use $\begin{pmatrix}1&2\\2&1\end{pmatrix}$. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part02.positivity_preserved_by_congruence`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter contributes its six-item spine to the dependency path from inner-product geometry through differentiation and oriented boundaries. The next chapter uses only the structures introduced so far.
