---
id: cft-chapter-20-numerical-range
title: The numerical range
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, operator-theory, mathematics, lean]
confidence: high
canonical: 20_numerical_range.md
chapter: 20
part: 4
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 20: The numerical range

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iv-finite-dimensional-operator-theory|Part IV — Finite-dimensional operator theory]]
Previous: [[knowledge/crouzeix_textbook/part_04_operator_theory/19_normality_and_nonnormality|Chapter 19 — Normality and nonnormality]]
Next: [[knowledge/crouzeix_textbook/part_04_operator_theory/21_spectral_sets|Chapter 21 — Spectral sets]]

## Opening problem

The spectrum of a nonnormal matrix can be too small to control its action. What is the smallest elementary geometric set that still records every quadratic observation ⟨Ax,x⟩ made by a unit vector?

## Conceptual model

The numerical range is W(A)={⟨Ax,x⟩:‖x‖=1}. It is basis-independent under unitary changes, compact in finite dimension, contains the spectrum, and—surprisingly—is convex. It is usually larger than the convex hull of the spectrum, exactly because it remembers nonnormal geometry. Its radius w(A)=max{|z|:z∈W(A)} is equivalent to operator norm: w(A)≤‖A‖≤2w(A).

## Formal development

### The six-item spine

### CFT-20-001 — numerical range nonempty {#cft-20-001}

### CFT-20-002 — numerical range as sphere image {#cft-20-002}

### CFT-20-003 — numerical range compact {#cft-20-003}

### CFT-20-004 — numerical range convex {#cft-20-004}

### CFT-20-005 — numerical range perturbation bound {#cft-20-005}

### CFT-20-006 — spectrum lies in numerical range {#cft-20-006}

The checked items are CFT-20-001, CFT-20-002, CFT-20-003, CFT-20-004, CFT-20-005, CFT-20-006.

**Theorem 20.1 (Toeplitz–Hausdorff).** W(A) is convex. Fix two unit vectors realizing points z₀,z₁. Their span has dimension at most two, so it is enough to prove the theorem for 2×2 compressions. In two complex dimensions, the unit sphere modulo phase maps to a real two-sphere, and the quadratic form is an affine image of that sphere; the image equals the image of the filled ball and is convex. This proof sequence is an informal roadmap. The related compiled theorem is a navigation checkpoint; no reviewed exact correspondence with this prose argument is claimed.

**Theorem 20.2 (spectral inclusion).** σ(A)⊆W(A). For an eigenpair Ax=λx with ‖x‖=1, ⟨Ax,x⟩=λ. Over ℂ every finite matrix has an eigenvalue, so W(A) is nonempty.

**Theorem 20.3 (support-line description).** For θ∈ℝ,
max_{z∈W(A)} Re(e^{-iθ}z)=λ_max(Re(e^{-iθ}A)).
This follows by rewriting the left side as the maximum Rayleigh quotient of a Hermitian matrix. Thus W(A) is the intersection of its supporting half-planes.

**Theorem 20.4 (perturbation).** Every z∈W(A) lies within ‖A-B‖ of some point of W(B): use the same unit vector x and bound |⟨(A-B)x,x⟩|≤‖A-B‖.

CFT-20-001--006 formalize nonemptiness, sphere-image form, compactness, convexity, perturbation stability, and spectral inclusion.

## Worked examples

**Example 1.** For $J=\begin{pmatrix}0&1\\0&0\end{pmatrix}$, writing $x=(\cos t,e^{i\phi}\sin t)$ gives $\langle Jx,x\rangle=e^{i\phi}\sin t\cos t$. Hence $W(J)$ is the closed disk of radius $1/2$, although $\sigma(J)=\{0\}$.

**Example 2.** For Hermitian A, every Rayleigh quotient lies between the smallest and largest eigenvalues, and every intermediate value occurs. Thus W(A)=[λ_min,λ_max].

## ML bridge

The field of values bounds transient dynamics and polynomial filters without assuming normality. It also appears in convergence guarantees for Krylov solvers. In ML terms, W(A) retains directional quadratic responses that an eigenvalue-only summary discards, while remaining a tractable convex object.

## Lean translation

The copyable declaration is #check CrouzeixTextbook.Part04.numerical_range_convex. Lean’s theorem is finite-dimensional and complex, with the exact Euclidean inner product. The perturbation theorem separately records quantitative stability, which is needed when passing through simple-spectrum approximants.

## Exercises

### CFT-20-E01 -- retrieval {#exercise-cft-20-e01}

Define W(A) and numerical radius.

### CFT-20-E02 -- calculation {#exercise-cft-20-e02}

Compute W(diag(λ,μ)).

### CFT-20-E03 -- written-proof {#exercise-cft-20-e03}

Prove σ(A)⊆W(A).

### CFT-20-E04 -- written-proof {#exercise-cft-20-e04}

Prove W(U*AU)=W(A) for unitary U.

### CFT-20-E05 -- boundary {#exercise-cft-20-e05}

Show by example that W(A) need not equal conv σ(A).

### CFT-20-E06 -- lean-proof {#exercise-cft-20-e06}

Use numerical_range_perturbation_bound to state the Hausdorff-style stability estimate.

### Solution sketches

For (2), the value is |x₁|²λ+|x₂|²μ, the line segment. For (3), normalize an eigenvector. For (4), change variables x=Uy. For (5), use the nilpotent Jordan block. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part04.numerical_range_perturbation_bound`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named theorem does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter’s sixth contract declaration is a navigation handoff to Chapter 21. The compiled interfaces expose related formal statements; because correspondence remains summary/checkpoint or unmapped, they do not certify every analytic step above.
