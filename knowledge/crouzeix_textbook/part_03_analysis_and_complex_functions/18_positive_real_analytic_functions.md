---
id: cft-chapter-18-positive-real-analytic-functions
title: Positive-real analytic functions
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, analysis-and-complex-functions, mathematics, lean]
confidence: high
canonical: 18_positive_real_analytic_functions.md
chapter: 18
part: 3
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 18: Positive-real analytic functions

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iii-analysis-and-complex-functions|Part III — Analysis and complex functions]]
Previous: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/17_functions_of_matrices_and_operators|Chapter 17 — Functions of matrices and operators]]
Next: [[knowledge/crouzeix_textbook/part_04_operator_theory/19_normality_and_nonnormality|Chapter 19 — Normality and nonnormality]]

## Opening problem

How can the inequality Re F(z)≥0 for every point in the unit disk be converted into a finite positive-semidefinite matrix inequality? The Herglotz kernel performs exactly this conversion and is the entry point to Jin’s completion argument.

## Conceptual model

For a scalar holomorphic function f on the disk, positive real part is equivalent to positivity of the kernel
K_f(z,w)=[f(z)+conj(f(w))]/[1-z conj(w)].
Kernel positivity means every finite Gram matrix [K_f(z_i,z_j)] is positive semidefinite, including repeated sample points. This is stronger-looking than pointwise positivity but analyticity links the two.

The Cayley transform moves between the Schur class |s(z)|≤1 and positive-real functions: f=(1+s)/(1-s), s=(f-1)/(f+1), provided the denominators are valid. Matrix-valued versions replace nonnegativity by positive semidefiniteness and require careful multiplication order.

## Formal development

### The six-item spine

### CFT-18-001 — open unit disk {#cft-18-001}

### CFT-18-002 — unit circle {#cft-18-002}

### CFT-18-003 — sampled kernel matrix {#cft-18-003}

### CFT-18-004 — positive matrix kernel on {#cft-18-004}

### CFT-18-005 — matrix herglotz kernel {#cft-18-005}

### CFT-18-006 — matrix herglotz kernel positive {#cft-18-006}

The formal items are CFT-18-001, CFT-18-002, CFT-18-003, CFT-18-004, CFT-18-005, CFT-18-006.

**Theorem 18.1 (scalar Herglotz kernel).** If f is holomorphic with Re f≥0 on the disk, then K_f is positive. One route uses the Herglotz representation f(z)=∫(ζ+z)/(ζ-z)dμ(ζ)+iβ. For each boundary point ζ, the kernel factorizes as 2/[ (1-z conj ζ)(1-conj w ζ) ], a rank-one positive kernel; integration preserves positivity.

**Theorem 18.2 (finite sampling).** Kernel positivity is precisely
∑_{i,j} conj(c_i)K(z_i,z_j)c_j≥0
for every finite sample list and coefficient list. Repetitions are allowed; forbidding them would lose derivative information in limiting arguments.

**Theorem 18.3 (matrix-valued kernel).** For F(z)∈M_n(ℂ), define
K_F(z,w)=[F(z)+F(w)*]/[1-z conj(w)].
Positivity now tests block vectors u_i∈ℂⁿ. Equivalently, the sampled block matrix is positive semidefinite.

**Theorem 18.4 (Cayley correspondence).** A contractive analytic S gives positive-real F=(I+S)(I-S)⁻¹. Expanding F+F* and using I-SS*≥0 yields a congruence by (I-S)⁻¹. The order of factors matters.

The six Lean items define the disk, circle, sampled matrix, positive-kernel predicate, Herglotz kernel, and its matrix positivity theorem.

## Worked examples

**Example 1.** f(z)=(1+z)/(1-z) has Re f(z)=(1-|z|²)/|1-z|²>0. Its kernel equals 2/[(1-z)(1-conj w)], visibly rank one and positive.

**Example 2.** For the constant matrix F(z)=P with P=P*≥0, K_F(z,w)=2P/(1-z conj w). The Szegő kernel 1/(1-z conj w) is positive, and tensoring with P preserves positivity.

## ML bridge

Positive kernels are familiar from Gaussian processes and kernel methods, but here the input kernel is operator-valued and its positivity certifies an analytic interpolation/completion problem. The finite Gram tests look like kernel matrices in ML; the crucial difference is that they encode an exact theorem rather than a data-dependent empirical approximation.

## Lean translation

The key checkpoint is #check CrouzeixTextbook.Part03.matrix_herglotz_kernel_positive. It expands the abstract kernel predicate into all finite sample sizes and coefficient vectors. This precise quantification is what later licenses a selected sample/origin quadratic-form cancellation.

## Exercises

### CFT-18-E01 -- retrieval {#exercise-cft-18-e01}

Define a positive scalar kernel by finite quadratic tests.

### CFT-18-E02 -- calculation {#exercise-cft-18-e02}

Compute Re[(1+z)/(1-z)].

### CFT-18-E03 -- written-proof {#exercise-cft-18-e03}

Prove that a rank-one kernel K(z,w)=g(z)conj(g(w)) is positive.

### CFT-18-E04 -- written-proof {#exercise-cft-18-e04}

Show that sums and positive scalar multiples of positive kernels are positive.

### CFT-18-E05 -- boundary {#exercise-cft-18-e05}

Explain why testing only distinct sample points is an artificial restriction.

### CFT-18-E06 -- lean-proof {#exercise-cft-18-e06}

Read matrix_herglotz_kernel_positive and identify the matrix adjoint and disk hypotheses.

### Solution sketches

For (2), multiply numerator and denominator by 1-conj(z). For (3), the quadratic form is |∑conj(c_i)g(z_i)|². For (4), add the nonnegative quadratic forms. For (5), repeated points arise naturally when samples coalesce and should not change the definition. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part03.matrix_herglotz_kernel_positive`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

Chapter 18 turns the preceding finite-dimensional geometry into a reusable analytic interface. Its last compiled item feeds directly into Chapter 19, while the distinctions between algebraic identity, convergence, and positivity remain visible for both constant-two routes.
