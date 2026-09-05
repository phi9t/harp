---
id: cft-chapter-19-normality-and-nonnormality
title: Normality and nonnormality
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, operator-theory, mathematics, lean]
confidence: high
canonical: 19_normality_and_nonnormality.md
chapter: 19
part: 4
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 19: Normality and nonnormality

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iv-finite-dimensional-operator-theory|Part IV — Finite-dimensional operator theory]]
Previous: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/18_positive_real_analytic_functions|Chapter 18 — Positive-real analytic functions]]
Next: [[knowledge/crouzeix_textbook/part_04_operator_theory/20_numerical_range|Chapter 20 — The numerical range]]

## Opening problem

Two matrices can have the same eigenvalues and radically different transient behavior. The diagonal matrix $\operatorname{diag}(0,0)$ is zero, while the nilpotent Jordan block $\begin{pmatrix}0&M\\0&0\end{pmatrix}$ has the same spectrum and norm $M$. What structure makes eigenvalues trustworthy, and what fails when that structure is absent?

## Conceptual model

An operator A is normal when A*A=AA*. Unitary and self-adjoint operators are normal; a generic trained linear layer is not. The finite-dimensional spectral theorem says normality is exactly what permits unitary diagonalization. Similarity diagonalization alone can use an ill-conditioned change of basis and does not preserve the Euclidean norm. Nonnormality is therefore not a minor technical defect: it is the gap between spectral data and geometric action.

## Formal development

### The six-item spine

### CFT-19-001 — simple spectrum approximation {#cft-19-001}

### CFT-19-002 — simple spectrum approximation distinct {#cft-19-002}

### CFT-19-003 — simple spectrum approximation close {#cft-19-003}

### CFT-19-004 — simple spectrum approximation converges {#cft-19-004}

### CFT-19-005 — distinct spectrum dense {#cft-19-005}

### CFT-19-006 — diagonal polynomial action {#cft-19-006}

The checked items are CFT-19-001, CFT-19-002, CFT-19-003, CFT-19-004, CFT-19-005, CFT-19-006.

**Theorem 19.1 (spectral theorem).** A complex matrix is normal if and only if it admits A=UΛU* with U unitary and Λ diagonal. One proof takes an eigenvector, shows its orthogonal complement is invariant under both A and A*, and inducts on dimension. Normality is used to show an A-eigenvector is also an A*-eigenvector.

**Corollary 19.2.** For normal A, ‖p(A)‖=max_{λ∈σ(A)}|p(λ)|. Indeed p(A)=Up(Λ)U*, unitary conjugation preserves norm, and a diagonal operator norm is its largest diagonal modulus.

**Definition 19.3 (pseudospectrum).** The ε-pseudospectrum consists of z for which ‖(zI-A)⁻¹‖>ε⁻¹, together with σ(A). For normal A it is exactly the ε-neighborhood of the spectrum. For nonnormal A it can bulge much farther.

**Theorem 19.4 (dense simple spectrum).** Every finite complex matrix is a norm limit of matrices with distinct eigenvalues. The proof perturbs along a polynomial pencil and avoids finitely many roots of a nonzero discriminant. This density enables proofs on diagonalizable approximants, but only if the quantity under study is continuous.

Items CFT-19-001--006 expose the maintained simple-spectrum approximation, distinctness, quantitative closeness, convergence, density, and diagonal polynomial calculus.

## Worked examples

**Example 1.** $J=\begin{pmatrix}0&M\\0&0\end{pmatrix}$ satisfies $J^2=0$ and $e^{tJ}=I+tJ$. Its spectrum is $\{0\}$, yet $\lVert e^{tJ}\rVert$ grows initially like $|t|M$. Eigenvalues miss the transient because the eigenvectors have collapsed.

**Example 2.** If A=Udiag(1,i)U* and p(z)=z²+z, then ‖p(A)‖=max(|2|,|-1+i|)=2. Replacing U by a nonunitary similarity changes the norm estimate by its condition number.

## ML bridge

Nonnormal Jacobians arise in recurrent networks, residual dynamics, and optimizer linearizations. Spectral-radius regularization alone does not prevent transient amplification. Pseudospectral or numerical-range control is often closer to the stability question because it retains interaction with the Euclidean geometry.

## Lean translation

The chapter module uses the explicit simpleSpectrumApproximation sequence rather than a vague density assertion. #check CrouzeixTextbook.Part04.simple_spectrum_approximation_converges is the bridge that later transports fixed-contour bounds from distinct-eigenvalue matrices to arbitrary matrices.

## Exercises

### CFT-19-E01 -- retrieval {#exercise-cft-19-e01}

Define normal, self-adjoint, and unitary operators.

### CFT-19-E02 -- calculation {#exercise-cft-19-e02}

Verify directly that $\begin{pmatrix}0&M\\0&0\end{pmatrix}$ is nonnormal for $M\ne0$.

### CFT-19-E03 -- written-proof {#exercise-cft-19-e03}

Prove that unitary conjugation preserves operator norm.

### CFT-19-E04 -- written-proof {#exercise-cft-19-e04}

Derive the normal polynomial norm identity from unitary diagonalization.

### CFT-19-E05 -- boundary {#exercise-cft-19-e05}

Give diagonalizable matrices whose eigenvector condition numbers diverge.

### CFT-19-E06 -- lean-proof {#exercise-cft-19-e06}

Explain why density alone is insufficient without continuity of the evaluated expression.

### Solution sketches

For (2), compare J*J and JJ*. For (3), substitute y=U*x in the supremum. For (4), reduce to the maximum diagonal entry. For (5), use Sε diag(0,1) Sε⁻¹ with nearly parallel columns in Sε. Related compiled navigation checkpoints for Exercise (6) are the convergence and functional-calculus continuity interfaces. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named interfaces do not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter’s sixth contract declaration is a navigation handoff to Chapter 20. The compiled interfaces expose related formal statements; because correspondence remains summary/checkpoint or unmapped, they do not certify every analytic step above.
