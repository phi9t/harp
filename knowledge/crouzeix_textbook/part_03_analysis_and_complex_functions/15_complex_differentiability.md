---
id: cft-chapter-15-complex-differentiability
title: Complex differentiability
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, analysis-and-complex-functions, mathematics, lean]
confidence: high
canonical: 15_complex_differentiability.md
chapter: 15
part: 3
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 15: Complex differentiability

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iii-analysis-and-complex-functions|Part III — Analysis and complex functions]]
Previous: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/14_sequences_and_series_of_operators|Chapter 14 — Sequences and series of operators]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/16_consequences_of_cauchy_theory|Chapter 16 — Consequences of Cauchy theory]]

## Opening problem

A real-differentiable map ℝ²→ℝ² has a 2×2 derivative. What extra rigidity appears when that derivative must be multiplication by a complex number? That single constraint produces Cauchy’s integral formula and, eventually, a functional calculus for matrices.

## Conceptual model

Complex differentiability asks whether [f(z+h)-f(z)]/h has one limit for every complex direction h→0. The derivative is therefore ℂ-linear, far more restrictive than an arbitrary ℝ-linear derivative. In coordinates f=u+iv, this rigidity is encoded by the Cauchy–Riemann equations u_x=v_y and u_y=-v_x.

A holomorphic function is complex differentiable on an open set. Analytic means locally equal to a convergent power series. The deep theorem is that holomorphic implies analytic; Cauchy’s integral formula is the bridge.

## Formal development

### The six-item spine

### CFT-15-001 — parametric boundary integral {#cft-15-001}

### CFT-15-002 — boundary integral continuous {#cft-15-002}

### CFT-15-003 — boundary integral is function eval {#cft-15-003}

### CFT-15-004 — simple spectrum holomorphic eval {#cft-15-004}

### CFT-15-005 — simple spectrum boundary limit {#cft-15-005}

### CFT-15-006 — simple spectrum eval limit {#cft-15-006}

The formal items are CFT-15-001, CFT-15-002, CFT-15-003, CFT-15-004, CFT-15-005, CFT-15-006.

**Theorem 15.1 (Cauchy–Riemann necessity).** If f is complex differentiable at z, compare increments along h=t and h=it. Both directional limits must equal f′(z), yielding u_x=v_y and u_y=-v_x.

**Theorem 15.2 (Cauchy theorem, disk form).** If f is holomorphic on a neighborhood of a disk, its integral around the boundary is zero. One proof applies Green’s theorem to u and v and cancels terms using Cauchy–Riemann. More robust versions remove auxiliary smoothness by local primitives or triangle subdivision.

**Theorem 15.3 (Cauchy integral formula).** For a inside a positively oriented contour Γ,
f(a)=(1/(2πi))∮_Γ f(z)/(z-a) dz.
Subtract f(a) in the numerator. The quotient extends continuously at a, and Cauchy’s theorem kills its integral; the remaining integral of 1/(z-a) is 2πi.

**Corollary 15.4 (derivative formula).** Differentiating under the fixed contour gives f⁽ᵐ⁾(a)=m!/(2πi)∮f(z)/(z-a)^{m+1}dz. Hence |f⁽ᵐ⁾(a)|≤m! M/rᵐ on a circle of radius r.

CFT-15-001--006 formalize the fixed parametrized contour integral, its continuity in the matrix, agreement with simple-spectrum evaluation, and the approximation limit.

## Worked examples

**Example 1.** f(z)=conj(z) has directional quotient 1 along the real axis and -1 along the imaginary axis, so it is nowhere complex differentiable despite being smooth as a real map.

**Example 2.** For f(z)=1/(1-z) and |z|<1, Cauchy’s coefficient formula yields f(z)=∑zᵏ. The same series, with z replaced by a matrix T under a convergence hypothesis, is the resolvent series of Chapter 14.

## ML bridge

Wirtinger calculus treats z and conj(z) as formally independent coordinates. It is useful for real losses on complex parameters, but it does not make a nonholomorphic loss holomorphic. Holomorphic structure matters here because contour deformation and Cauchy estimates require genuine complex differentiability, not merely automatic differentiation through complex-valued code.

## Lean translation

The copyable checkpoint is #check CrouzeixTextbook.Part03.boundary_integral_is_function_eval. Its long type is pedagogically valuable: it records openness, convexity, closure containment, differentiability, diagonalization, and numerical-range containment. Those are precisely the hypotheses often compressed into “by functional calculus.”

## Exercises

### CFT-15-E01 -- retrieval {#exercise-cft-15-e01}

Define complex differentiability at a point.

### CFT-15-E02 -- calculation {#exercise-cft-15-e02}

Derive the Cauchy–Riemann equations by using real and imaginary increments.

### CFT-15-E03 -- written-proof {#exercise-cft-15-e03}

Prove that z↦z² has derivative 2z directly from the difference quotient.

### CFT-15-E04 -- written-proof {#exercise-cft-15-e04}

Derive Cauchy’s estimate from the derivative formula.

### CFT-15-E05 -- boundary {#exercise-cft-15-e05}

Give a function satisfying Cauchy–Riemann at one point but not complex differentiable there.

### CFT-15-E06 -- lean-proof {#exercise-cft-15-e06}

Read boundary_integral_is_function_eval and group its hypotheses into geometry, analyticity, and matrix structure.

### Solution sketches

For (2), identify the two directional derivatives with the same complex number. For (3), expand (z+h)²-z². For (4), take norms, bound the contour length by 2πr, and divide by r^{m+1}. For (5), f(z)=|z|² satisfies the equations at zero but has quotient conj(h), which depends on direction. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part03.boundary_integral_is_function_eval`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

Chapter 15 turns the preceding finite-dimensional geometry into a reusable analytic interface. Its last compiled item feeds directly into Chapter 16, while the distinctions between algebraic identity, convergence, and positivity remain visible for both constant-two routes.
