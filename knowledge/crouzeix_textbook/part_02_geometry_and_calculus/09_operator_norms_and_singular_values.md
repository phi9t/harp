---
id: cft-chapter-09-operator-norms-and-singular-values
title: Operator norms and singular values
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 09_operator_norms_and_singular_values.md
chapter: 9
part: 2
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 9: Operator norms and singular values

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/08_positive_operators_and_gram_geometry|Chapter 8 — Positive operators and Gram geometry]]
Next: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/10_multilinear_maps_and_tensors|Chapter 10 — Multilinear maps and tensors]]

## Opening problem

How do we turn linear structure into quantitative geometry? This chapter develops induced norms, singular values, polar decomposition, and conditioning. The goal is not a catalogue of formulas but a chain of proofs whose hypotheses remain visible.

## Conceptual model

Algebra says which combinations are legal. Geometry adds a rule for measuring size, angle, orientation, or first-order change. Each new identification is extra structure: a covector is not automatically a vector, a transpose is not automatically an adjoint, and a coordinate formula is not automatically basis-free.

## Formal development

### The six-item spine

### CFT-09-001 — induced matrix norm identity {#cft-09-001}

### CFT-09-002 — polar factor is unitary {#cft-09-002}

### CFT-09-003 — polar similarity norm transfer {#cft-09-003}

### CFT-09-004 — polar operator norm transfer {#cft-09-004}

### CFT-09-005 — quadratic bound implies norm two {#cft-09-005}

### CFT-09-006 — norm triangle kernel {#cft-09-006}

The chapter tracks six formal items, `CFT-09-001`, `CFT-09-002`, `CFT-09-003`, `CFT-09-004`, `CFT-09-005`, `CFT-09-006`. The induced norm is $\|T\|=\sup_{\|x\|=1}\|Tx\|$; compactness of the finite unit sphere gives attainment. Since $\|Tx\|^2=\langle T^*Tx,x\rangle$, the largest eigenvalue of $T^*T$ is $\|T\|^2$. Its square roots are singular values. The spectral theorem gives an SVD, while $T=U(T^*T)^{1/2}$ is the polar decomposition, with $U$ unitary on the supported finite-dimensional space.

The proof discipline is consistent: expand the relevant definition, isolate the term controlled by positivity, orthogonality, multilinearity, or the small-o remainder, and only then invoke finite-dimensional compactness or coordinates. This prevents a later operator inequality from silently assuming normality or commutativity.

### Proof workshop: from $A^*A$ to the SVD

The induced norm obeys $\lVert Ax\rVert\le\lVert A\rVert\lVert x\rVert$
by scaling nonzero $x$ to the unit sphere. Finite-dimensional compactness
makes the supremum a maximum. Because
$\lVert Ax\rVert^2=\langle A^*Ax,x\rangle$, the Rayleigh quotient theorem
for the PSD matrix $A^*A$ gives
$\lVert A\rVert^2=\lambda_{\max}(A^*A)$.

Choose an orthonormal eigenbasis $v_i$ of $A^*A$ with eigenvalues
$\sigma_i^2\ge0$. When $\sigma_i>0$, set $u_i=Av_i/\sigma_i$.
Then the $u_i$ are orthonormal because
$\langle Av_i,Av_j\rangle
=\langle v_i,A^*Av_j\rangle=\sigma_j^2\delta_{ij}$.
Extend them across the zero singular subspace. The resulting bases give
$A=U\Sigma V^*$, the singular-value decomposition.

Set $|A|=(A^*A)^{1/2}=V\Sigma V^*$ and
$Q=UV^*$ on the supported subspace. Then $A=Q|A|$ is the polar
decomposition. For invertible square $A$, $Q$ is unitary. The condition
number $\kappa_2(A)=\sigma_{\max}/\sigma_{\min}$ quantifies how much
solving $Ax=b$ can amplify relative errors.

## Worked examples

**Example 1.** For
$A=\begin{pmatrix}0&3\\0&4\end{pmatrix}$,
$A^*A=\begin{pmatrix}0&0\\0&25\end{pmatrix}$. Its singular values are
$5$ and $0$, so $\lVert A\rVert_2=5$ even though both eigenvalues of $A$
are zero and four.

**Example 2.** For diagonal $A=\operatorname{diag}(\varepsilon,1)$,
$\kappa_2(A)=1/\varepsilon$. The map is invertible for every
$\varepsilon>0$, but inversion becomes arbitrarily sensitive as
$\varepsilon\downarrow0$.

## ML bridge

Gradient amplification, exploding dynamics, and robustness are operator-norm questions. Eigenvalues suffice only for normal operators; singular values remain geometric for nonnormal ones.

## Lean translation

`CrouzeixTextbook.Part02.Chapter09` exposes six checked declarations. They reuse maintained Harp or Mathlib results only at matching statement boundaries. For Chapters 11--12, the formal scope is deliberately finite-coordinate or planar; it does not pretend to formalize a JAX runtime or the full manifold-level Stokes theorem.

## Exercises

### CFT-09-E01 -- retrieval {#exercise-cft-09-e01}

Define the induced operator norm.

### CFT-09-E02 -- calculation {#exercise-cft-09-e02}

Compute the singular values of $\operatorname{diag}(2,-3)$.

### CFT-09-E03 -- written-proof {#exercise-cft-09-e03}

Prove $\lVert AB\rVert\le\lVert A\rVert\lVert B\rVert$.

### CFT-09-E04 -- written-proof {#exercise-cft-09-e04}

Derive $\lVert A\rVert^2=\lambda_{\max}(A^*A)$.

### CFT-09-E05 -- boundary {#exercise-cft-09-e05}

Give a nilpotent matrix whose norm is arbitrarily large.

### CFT-09-E06 -- lean-proof {#exercise-cft-09-e06}

Inspect polar_similarity_norm_transfer and identify the unitary factor being removed.

### Solution sketches

For (2), the singular values are $2$ and $3$. For (3), apply the defining estimate twice. For (4), maximize the Rayleigh quotient. For (5), use $\begin{pmatrix}0&M\\0&0\end{pmatrix}$. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part02.polar_similarity_norm_transfer`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter contributes its six-item spine to the dependency path from inner-product geometry through differentiation and oriented boundaries. The next chapter uses only the structures introduced so far.
