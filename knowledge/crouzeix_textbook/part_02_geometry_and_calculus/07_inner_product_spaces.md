---
id: cft-chapter-07-inner-product-spaces
title: Inner-product spaces
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 07_inner_product_spaces.md
chapter: 7
part: 2
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 7: Inner-product spaces

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_01_linear_structure/06_eigenvalues_and_polynomial_algebra|Chapter 6]]
Next: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/08_positive_operators_and_gram_geometry|Chapter 8 — Positive operators and Gram geometry]]

## Opening problem

How do we turn linear structure into quantitative geometry? This chapter develops projection, orthogonality, adjoints, and Riesz representation. The goal is not a catalogue of formulas but a chain of proofs whose hypotheses remain visible.

## Conceptual model

Algebra says which combinations are legal. Geometry adds a rule for measuring size, angle, orientation, or first-order change. Each new identification is extra structure: a covector is not automatically a vector, a transpose is not automatically an adjoint, and a coordinate formula is not automatically basis-free.

## Formal development

### The six-item spine

### CFT-07-001 — projection residual decomposition {#cft-07-001}

### CFT-07-002 — projection residual orthogonal {#cft-07-002}

### CFT-07-003 — adjoint coordinate identity {#cft-07-003}

### CFT-07-004 — adjoint matrix is conjugate transpose {#cft-07-004}

### CFT-07-005 — matrix norm is operator norm {#cft-07-005}

### CFT-07-006 — norm is nonnegative {#cft-07-006}

The chapter tracks six formal items, `CFT-07-001`, `CFT-07-002`, `CFT-07-003`, `CFT-07-004`, `CFT-07-005`, `CFT-07-006`. The identity $x=P_Ux+(x-P_Ux)$ becomes useful only after proving the residual is orthogonal to $U$. Cauchy--Schwarz follows by minimizing $\|x-ty\|^2$ over scalars $t$; equality means linear dependence. In finite dimensions, the same estimate makes every linear functional continuous, and expansion in an orthonormal basis supplies its unique Riesz vector. The adjoint is then defined by $\langle Tx,y\rangle=\langle x,T^*y\rangle$.

The proof discipline is consistent: expand the relevant definition, isolate the term controlled by positivity, orthogonality, multilinearity, or the small-o remainder, and only then invoke finite-dimensional compactness or coordinates. This prevents a later operator inequality from silently assuming normality or commutativity.

### Proof workshop: projection and Cauchy--Schwarz

For a nonzero vector $u$, minimize $q(t)=\lVert x-tu\rVert^2$. Over
$\mathbb C$ the minimizing coefficient is
$t=\langle u,x\rangle/\langle u,u\rangle$ (with the book's inner-product
convention). Expanding at that value gives
$0\le \lVert x\rVert^2-|\langle u,x\rangle|^2/\lVert u\rVert^2$, hence
$|\langle u,x\rangle|\le\lVert u\rVert\lVert x\rVert$. Equality holds
exactly when the residual vanishes, that is, when $x$ lies on the line
spanned by $u$.

For a subspace $U$ with orthonormal basis $e_1,\ldots,e_r$, set
$P_Ux=\sum_j\langle e_j,x\rangle e_j$. Testing $x-P_Ux$ against each basis
vector proves it is orthogonal to all of $U$. If also $x=u+v$ with $u\in U$
and $v\perp U$, subtract the two decompositions: $u-P_Ux$ lies in both $U$
and $U^\perp$, so its squared norm is zero. This proves uniqueness, not just
the coordinate formula.

The adjoint is constructed by applying finite-dimensional Riesz
representation to the functional $x\mapsto\langle Tx,y\rangle$ for each
fixed $y$. Uniqueness of the Riesz vector proves linearity in $y$ and the
identity $(ST)^*=T^*S^*$.

## Worked examples

**Example 1.** Project $x=(2,1)$ onto the line spanned by
$u=(1,1)$. The coefficient is
$\langle u,x\rangle/\langle u,u\rangle=3/2$, so
$P_Ux=(3/2,3/2)$ and the residual $(1/2,-1/2)$ is orthogonal to $u$.

**Example 2.** For
$A=\begin{pmatrix}1&i\\2&0\end{pmatrix}$ with the standard complex inner
product, the adjoint matrix is
$A^*=\begin{pmatrix}1&2\\-i&0\end{pmatrix}$. Direct multiplication checks
$\langle Ax,y\rangle=\langle x,A^*y\rangle$. In a nonorthonormal basis the
coordinate matrix of the adjoint is not obtained by naïve conjugate
transpose; the Gram matrix must also be inserted.

## ML bridge

Euclidean gradients identify covectors with vectors through the chosen inner product; changing that metric changes the gradient even when the derivative covector is unchanged.

## Lean translation

`CrouzeixTextbook.Part02.Chapter07` exposes six checked declarations. They reuse maintained Harp or Mathlib results only at matching statement boundaries. For Chapters 11--12, the formal scope is deliberately finite-coordinate or planar; it does not pretend to formalize a JAX runtime or the full manifold-level Stokes theorem.

## Exercises

### CFT-07-E01 -- retrieval {#exercise-cft-07-e01}

State the conjugate-symmetry and positivity axioms for a complex inner product.

### CFT-07-E02 -- calculation {#exercise-cft-07-e02}

Project $(1,2,3)$ onto the line spanned by $(1,1,1)$.

### CFT-07-E03 -- written-proof {#exercise-cft-07-e03}

Prove Cauchy--Schwarz by minimizing $\lVert x-tu\rVert^2$.

### CFT-07-E04 -- written-proof {#exercise-cft-07-e04}

Prove uniqueness of the orthogonal decomposition $x=u+v$.

### CFT-07-E05 -- boundary {#exercise-cft-07-e05}

Give a projection onto a line along a nonorthogonal complement and show it can increase norm.

### CFT-07-E06 -- lean-proof {#exercise-cft-07-e06}

In Lean, inspect projection_residual_orthogonal and identify the nonzero hypothesis.

### Solution sketches

For (2), the coefficient is $2$, so the projection is $(2,2,2)$. For (3), substitute the minimizing scalar and use nonnegativity. For (4), the difference belongs to $U\cap U^\perp$. For (5), project onto the first coordinate along a line almost parallel to it. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part02.projection_residual_orthogonal`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter contributes its six-item spine to the dependency path from inner-product geometry through differentiation and oriented boundaries. The next chapter uses only the structures introduced so far.
