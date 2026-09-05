---
id: cft-chapter-17-functions-of-matrices-and-operators
title: Functions of matrices and operators
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, analysis-and-complex-functions, mathematics, lean]
confidence: high
canonical: 17_functions_of_matrices_and_operators.md
chapter: 17
part: 3
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 17: Functions of matrices and operators

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iii-analysis-and-complex-functions|Part III — Analysis and complex functions]]
Previous: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/16_consequences_of_cauchy_theory|Chapter 16 — Consequences of Cauchy theory]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/18_positive_real_analytic_functions|Chapter 18 — Positive-real analytic functions]]

## Opening problem

What should f(A) mean when A is not diagonalizable? Substituting eigenvalues is insufficient because a Jordan block also sees derivatives of f. The contour definition handles diagonalizable and defective matrices uniformly.

## Conceptual model

Polynomial evaluation is unambiguous: p(A)=∑aₖAᵏ. Rational evaluation r=p/q is p(A)q(A)⁻¹ when q has no zero on the spectrum. Holomorphic evaluation packages all of this as a contour integral of the resolvent:
f(A)=(1/(2πi))∮Γ f(z)(zI-A)⁻¹dz.
The contour surrounds the spectrum and lies inside the holomorphy domain.

The resolvent remembers nonnormality. Its norm may be huge far from the spectrum, so spectral distance alone does not control it without normality.

## Formal development

### The six-item spine

### CFT-17-001 — rational pole set {#cft-17-001}

### CFT-17-002 — rational poles finite {#cft-17-002}

### CFT-17-003 — pole complement open {#cft-17-003}

### CFT-17-004 — rational pole free on {#cft-17-004}

### CFT-17-005 — rational scalar eval {#cft-17-005}

### CFT-17-006 — rational matrix eval {#cft-17-006}

The formal items are CFT-17-001, CFT-17-002, CFT-17-003, CFT-17-004, CFT-17-005, CFT-17-006.

**Theorem 17.1 (resolvent identity).** For z,w outside the spectrum,
(zI-A)⁻¹-(wI-A)⁻¹=(w-z)(zI-A)⁻¹(wI-A)⁻¹.
Multiply both sides by zI-A and wI-A, or use X⁻¹-Y⁻¹=X⁻¹(Y-X)Y⁻¹.

**Theorem 17.2 (polynomial compatibility).** The contour formula gives p(A). It suffices by linearity to treat p(z)=zᵏ. Polynomial division writes zᵏ/(z-λ) as a polynomial plus λᵏ/(z-λ); the polynomial part integrates to zero and Cauchy’s formula returns λᵏ. A Jordan or resolvent argument lifts the scalar identity.

**Theorem 17.3 (algebra homomorphism).** (f+g)(A)=f(A)+g(A), (fg)(A)=f(A)g(A), and 1(A)=I. Additivity is integral linearity. Multiplicativity uses two nested contours and the resolvent identity; Cauchy’s formula collapses one integral.

**Theorem 17.4 (spectral mapping).** σ(f(A))=f(σ(A)) in finite dimension. Factor f(z)-μ around its finitely many zeros near σ(A), then use multiplicativity and invertibility of zero-free factors.

CFT-17-001--006 gives a fail-closed rational interface: explicit finite pole set, open pole complement, pole-freeness predicate, scalar evaluation, and matrix evaluation.

## Worked examples

**Example 1.** For $J=\begin{pmatrix}\lambda&1\\0&\lambda\end{pmatrix}$ and holomorphic $f$, write $J=\lambda I+N$ with $N^2=0$. Taylor expansion gives $f(J)=f(\lambda)I+f\prime(\lambda)N$. Eigenvalues alone miss the derivative term.

**Example 2.** For r(z)=1/(2-z) and ‖A‖<2, r(A)=(2I-A)⁻¹=(1/2)∑(A/2)ᵏ. The rational, contour, and Neumann definitions agree where their hypotheses overlap.

## ML bridge

Matrix functions appear in exponential integrators, covariance transforms, graph filters, and implicit differentiation. Diagonalizing a highly nonnormal matrix can be numerically unstable even when the abstract calculus is sound. Contour quadrature and Krylov methods approximate the same mathematical object with different stability tradeoffs; this chapter proves the object, not a particular numerical method.

## Lean translation

The declarations rational_pole_free_on and rational_matrix_eval make the denominator contract explicit. #check CrouzeixTextbook.Part03.rational_matrix_eval shows that the definition is totalized at the Lean level, while meaningful theorems add pole-freeness hypotheses before asserting inverse or norm properties.

## Exercises

### CFT-17-E01 -- retrieval {#exercise-cft-17-e01}

Define polynomial, rational, and holomorphic matrix evaluation.

### CFT-17-E02 -- calculation {#exercise-cft-17-e02}

Compute f(J) for f(z)=exp z and a 2×2 Jordan block.

### CFT-17-E03 -- written-proof {#exercise-cft-17-e03}

Prove the first resolvent identity.

### CFT-17-E04 -- written-proof {#exercise-cft-17-e04}

Prove that polynomial evaluation commutes with A.

### CFT-17-E05 -- boundary {#exercise-cft-17-e05}

Give a rational expression whose matrix evaluation is invalid because a pole hits the spectrum.

### CFT-17-E06 -- lean-proof {#exercise-cft-17-e06}

Explain the role of rational_poles_finite in choosing a pole-free neighborhood.

### Solution sketches

For (2), f(J)=e^λ(I+N). For (3), apply the inverse-difference identity. For (4), every power Aᵏ commutes with A, hence their finite linear combination does. For (5), use r(z)=1/(z-λ) when λ∈σ(A). Related compiled navigation checkpoints for Exercise (6) are `CrouzeixTextbook.Part03.rational_poles_finite` and `pole_complement_open`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declarations do not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

Chapter 17 turns the preceding finite-dimensional geometry into a reusable analytic interface. Its last compiled item feeds directly into Chapter 18, while the distinctions between algebraic identity, convergence, and positivity remain visible for both constant-two routes.
