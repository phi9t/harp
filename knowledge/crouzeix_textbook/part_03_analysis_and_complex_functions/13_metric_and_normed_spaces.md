---
id: cft-chapter-13-metric-and-normed-spaces
title: Metric and normed spaces
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, analysis-and-complex-functions, mathematics, lean]
confidence: high
canonical: 13_metric_and_normed_spaces.md
chapter: 13
part: 3
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 13: Metric and normed spaces

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iii-analysis-and-complex-functions|Part III — Analysis and complex functions]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/12_differential_forms_and_stokes|Chapter 12 — Differential forms and Stokes]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/14_sequences_and_series_of_operators|Chapter 14 — Sequences and series of operators]]

## Opening problem

A sequence of matrices converges entrywise. May we immediately pass to its operator norms, inverses, spectra, and contour integrals? The answer is yes for some operations, no for others, and only under a quantitative hypothesis for inversion. This chapter builds the topology needed to tell those cases apart.

## Conceptual model

A metric turns “near” into an inequality; a norm additionally respects vector addition and scalar multiplication. On a finite-dimensional vector space all norms generate the same convergent sequences, but their numerical values and conditioning can differ dramatically. This distinction is central in machine learning: changing parametrization preserves finite-dimensional topology while changing optimization geometry.

A sequence is Cauchy when its tail points become mutually close. A space is complete when every Cauchy sequence converges inside it. Finite-dimensional normed spaces over ℝ or ℂ are complete because coordinates reduce the claim to completeness of the scalar field. Compactness is stronger than boundedness in general, but in finite dimension a set is compact exactly when it is closed and bounded. That theorem is the quiet engine behind “a maximum is attained.”

## Formal development

### The six-item spine

### CFT-13-001 — max modulus on compact set {#cft-13-001}

### CFT-13-002 — compact maximum exists {#cft-13-002}

### CFT-13-003 — pointwise norm le maximum {#cft-13-003}

### CFT-13-004 — compact maximum nonnegative {#cft-13-004}

### CFT-13-005 — compact maximum monotone {#cft-13-005}

### CFT-13-006 — outer maxima converge {#cft-13-006}

The formal items are CFT-13-001, CFT-13-002, CFT-13-003, CFT-13-004, CFT-13-005, CFT-13-006.

**Definition 13.1 (operator norm).** For a linear map T, set ‖T‖ = sup_{‖x‖=1} ‖Tx‖. Homogeneity gives ‖Tx‖ ≤ ‖T‖‖x‖, and submultiplicativity follows by applying this estimate twice.

**Theorem 13.2 (equivalence of finite-dimensional norms).** If ‖·‖ₐ and ‖·‖ᵦ are norms on V with dim V < ∞, then constants c,C>0 satisfy c‖x‖ₐ ≤ ‖x‖ᵦ ≤ C‖x‖ₐ. Proof: restrict x ↦ ‖x‖ᵦ to the ‖·‖ₐ-unit sphere. The sphere is compact, so the continuous function attains a finite maximum C and a positive minimum c. Positivity of the minimum uses that zero is not on the sphere.

**Theorem 13.3 (finite-dimensional Heine–Borel).** Closed bounded subsets are compact. In coordinates, bounded sequences have coordinatewise bounded subsequences; iterated Bolzano–Weierstrass extraction gives a convergent subsequence whose limit stays in the closed set.

**Theorem 13.4 (extreme value principle).** A continuous real-valued function on a nonempty compact set attains its maximum and minimum. In this book it produces max_{z∈W(A)} |p(z)| rather than an unverified supremum.

The six compiled items CFT-13-001 through CFT-13-006 expose exactly the compact maximum object, attainment, its pointwise bound, nonnegativity, monotonicity, and convergence under controlled outer approximation.

## Worked examples

**Example 1.** On ℂ², ‖x‖∞ ≤ ‖x‖₂ ≤ √2‖x‖∞. The constants come directly from max(|x₁|,|x₂|)² ≤ |x₁|²+|x₂|² ≤ 2max(|x₁|,|x₂|)². Thus convergence is identical, although the unit balls have different shapes.

**Example 2.** Let K={z:|z|≤1} and f(z)=z²+1. Compactness guarantees a maximizer. On |z|=1, |z²+1|≤2 with equality at z=±1; inside, the maximum cannot exceed the boundary maximum once complex theory is available. The elementary compactness claim and the later maximum-modulus claim are logically distinct.

## ML bridge

Finite-dimensional parameter spaces make norm choice topologically harmless but algorithmically decisive. Gradient clipping in ℓ₂, coordinatewise clipping in ℓ∞, and natural-gradient geometry can converge to the same local topology while following very different paths. The book uses equivalence of norms only for qualitative passage to limits; every quantitative constant remains tied to the Euclidean operator norm.

## Lean translation

The module CrouzeixTextbook.Part03.Chapter13 checks six declarations. For example, #check CrouzeixTextbook.Part03.compact_maximum_exists exposes the hypotheses that the set is compact and nonempty and the function continuous. Lean prevents the common paper shortcut of writing max over an empty or noncompact set.

## Exercises

### CFT-13-E01 -- retrieval {#exercise-cft-13-e01}

Define a Cauchy sequence and completeness without coordinates.

### CFT-13-E02 -- calculation {#exercise-cft-13-e02}

Prove ‖x‖∞ ≤ ‖x‖₂ ≤ √n‖x‖∞ on ℂⁿ.

### CFT-13-E03 -- written-proof {#exercise-cft-13-e03}

Prove that a finite-dimensional linear map is continuous by bounding its coordinate matrix.

### CFT-13-E04 -- written-proof {#exercise-cft-13-e04}

Prove that the unit sphere is compact and explain where finite dimensionality enters.

### CFT-13-E05 -- boundary {#exercise-cft-13-e05}

Give a bounded closed set in an infinite-dimensional normed space that is not compact.

### CFT-13-E06 -- lean-proof {#exercise-cft-13-e06}

Use compact_maximum_monotone to formalize why enlarging a compact domain cannot decrease the maximum modulus.

### Solution sketches

For (2), square both sides and compare the maximum coordinate with the sum. For (3), bound each output coordinate by a finite sum and combine the estimates. For (4), identify V with ℂⁿ and invoke closed-and-bounded compactness. For (5), use the standard orthonormal sequence in ℓ²: its elements remain pairwise √2 apart. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part03.compact_maximum_monotone`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

Chapter 13 turns the preceding finite-dimensional geometry into a reusable analytic interface. Its last compiled item feeds directly into Chapter 14, while the distinctions between algebraic identity, convergence, and positivity remain visible for both constant-two routes.
