---
id: cft-chapter-05-determinants-trace-and-exterior-algebra
title: Determinants, trace, and exterior algebra
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, linear-structure, mathematics, lean]
confidence: high
canonical: 05_determinants_trace_and_exterior_algebra.md
chapter: 5
part: 1
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 5: Determinants, trace, and exterior algebra

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-i-linear-structure|Part I -- Linear structure]]
Previous: [[knowledge/crouzeix_textbook/part_01_linear_structure/04_coordinates_and_duality|Chapter 4]]
Next: [[knowledge/crouzeix_textbook/part_01_linear_structure/06_eigenvalues_and_polynomial_algebra|Chapter 6 — Eigenvalues and polynomial algebra]]

## Opening problem

Which scalar summaries of a linear operator survive every change of basis?
Determinant records oriented volume scaling; trace records the first-order
change of volume near the identity. Their formulas look coordinate-bound, but
their meanings are structural.

## Conceptual model

An alternating multilinear form vanishes when two inputs coincide. Swapping
two inputs changes its sign. In dimension $n$, the space of alternating
$n$-forms is one-dimensional; normalizing its value on a basis produces the
determinant. Exterior powers package this idea: $T$ acts on
$\bigwedge^n V$ by the scalar $\det T$.

## Formal development

### CFT-05-001: multiplicativity {#cft-05-001}

Both $\det(AB)$ and $\det A\det B$ describe the volume factor obtained by
first applying $B$ and then $A$. Algebraically, fix $A$ and view
$\det(AB)$ as an alternating multilinear function of the columns of $B$;
uniqueness of the normalized determinant gives
$\det(AB)=\det A\det B$.

### CFT-05-002: diagonal matrices {#cft-05-002}

For $D=\operatorname{diag}(d_1,\ldots,d_n)$, only the identity permutation
survives in the Leibniz formula, so $\det D=\prod_i d_i$.

### CFT-05-003: similarity {#cft-05-003}

If $S$ is invertible,
$\det(SAS^{-1})=\det S\det A\det S^{-1}=\det A$.
The cancellation is why determinant belongs to the operator rather than its
matrix presentation.

### CFT-05-004 and CFT-05-005: trace {#cft-05-004}

### CFT-05-005 locator {#cft-05-005}

Direct expansion gives
$\operatorname{tr}(AB)=\sum_{i,j}A_{ij}B_{ji}
=\operatorname{tr}(BA)$. Cyclicity, not arbitrary commutativity, then yields
$\operatorname{tr}(SAS^{-1})=\operatorname{tr}A$.

### CFT-05-006: diagonal trace {#cft-05-006}

$\operatorname{tr}(\operatorname{diag}d)=\sum_i d_i$. Later, after
triangularization, this becomes “trace is the sum of eigenvalues” and
“determinant is their product,” with algebraic multiplicity.

## Worked examples

For $A=\begin{pmatrix}1&2\\3&4\end{pmatrix}$,
$\det A=-2$ and $\operatorname{tr}A=5$. For
$S=\begin{pmatrix}1&1\\0&1\end{pmatrix}$, direct multiplication verifies
that $SAS^{-1}$ has the same two scalars although different entries.

For $I+tA$, the determinant expansion begins
$\det(I+tA)=1+t\operatorname{tr}A+O(t^2)$. In dimension two this follows by
expanding the four entries; the general statement follows from multilinearity.

## ML bridge

Log-determinants measure volume change and appear in normalizing flows and
Gaussian likelihoods. Trace estimators approximate expensive sums. The exact
identities here concern mathematical matrices; stochastic estimators and
floating-point stability require additional probabilistic and numerical
analysis.

## Lean translation

Mathlib's determinant is already built from an alternating construction.
This chapter exposes six consequences as stable textbook declarations:
`determinant_multiplicative`, `determinant_diagonal`,
`determinant_similarity`, `trace_cyclic`, `trace_similarity`, and
`trace_diagonal`.

## Exercises

### CFT-05-E01 -- retrieval {#exercise-cft-05-e01}

Explain why an alternating map vanishes on repeated inputs.

### CFT-05-E02 -- calculation {#exercise-cft-05-e02}

Compute determinant and trace of a triangular $3\times3$ matrix.

### CFT-05-E03 -- written-proof {#exercise-cft-05-e03}

Derive determinant similarity invariance.

### CFT-05-E04 -- written-proof {#exercise-cft-05-e04}

Prove trace cyclicity by interchanging two finite sums.

### CFT-05-E05 -- boundary {#exercise-cft-05-e05}

Show $\operatorname{tr}(ABC)$ is cyclic but not invariant under arbitrary permutations.

### CFT-05-E06 -- lean-proof {#exercise-cft-05-e06}

Reproduce `trace_similarity` in Lean from `trace_mul_comm`.

Solutions: (1) swapping equal inputs changes the sign but not the value; (2)
product and sum of diagonal entries; (3) multiplicativity and
$\det S^{-1}=(\det S)^{-1}$; (4) rename $i,j$; (5) $ABC,BCA,CAB$ agree in
trace, while small matrix units give a counterexample to reversal; (6) rotate
the two factors and cancel the unit. Related compiled navigation checkpoints
are `CFT-05-001` through `CFT-05-006`. No exercise in this chapter currently
has a distinct checked Lean solution declaration; the named declarations do
not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

Determinant detects singularity and produces the characteristic polynomial;
trace summarizes spectral mass. Chapter 6 turns polynomials into operators.
