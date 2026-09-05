---
id: cft-chapter-22-positive-and-completely-positive-maps
title: Positive and completely positive maps
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, operator-theory, mathematics, lean]
confidence: high
canonical: 22_positive_and_completely_positive_maps.md
chapter: 22
part: 4
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 22: Positive and completely positive maps

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iv-finite-dimensional-operator-theory|Part IV — Finite-dimensional operator theory]]
Previous: [[knowledge/crouzeix_textbook/part_04_operator_theory/21_spectral_sets|Chapter 21 — Spectral sets]]
Next: [[knowledge/crouzeix_textbook/part_04_operator_theory/23_compression_and_dilation|Chapter 23 — Compression and dilation]]

## Opening problem

An integral of positive matrices is positive. But when a linear map acts entrywise on a block matrix, does positivity still survive? This question separates positivity from complete positivity and explains why matrix amplification is not automatic.

## Conceptual model

A linear map Φ between matrix algebras is positive if X≥0 implies Φ(X)≥0. It is completely positive if every amplification id_k⊗Φ is positive. Positivity is enough for scalar-level double-layer estimates; complete positivity is the structure needed for matrix-valued amplification. A unital positive map between C*-algebras is contractive on self-adjoint inputs, while Stinespring represents a completely positive map as compression of a *-representation.

## Formal development

### The six-item spine

### CFT-22-001 — boundary positive map {#cft-22-001}

### CFT-22-002 — boundary positive linear map {#cft-22-002}

### CFT-22-003 — boundary positive map norm {#cft-22-003}

### CFT-22-004 — boundary positive map unital {#cft-22-004}

### CFT-22-005 — boundary positive map star {#cft-22-005}

### CFT-22-006 — boundary positive map preserves psd {#cft-22-006}

The checked items are CFT-22-001, CFT-22-002, CFT-22-003, CFT-22-004, CFT-22-005, CFT-22-006.

**Definition 22.1.** X≥0 means X=X* and ⟨Xv,v⟩≥0 for all v. Congruence preserves positivity: V*XV≥0 because its quadratic form at u equals that of X at Vu.

**Theorem 22.2 (integral positivity).** If D(t)≥0 almost everywhere and is integrable, then ∫D(t)dμ(t)≥0. Test against each vector x, move the bounded linear quadratic functional through the Bochner integral, and integrate a nonnegative scalar function.

**Theorem 22.3 (boundary map).** Define Φ(h)=∫h(t)D(t)dμ(t). When h is real nonnegative, Φ(h)≥0. With normalized mass, Φ(1)=I. The adjoint law Φ(conj h)=Φ(h)* follows from conjugate transpose commuting with the integral.

**Definition 22.4 (complete positivity).** For every k, the map on M_k-valued inputs must preserve block positivity. A merely positive map need not satisfy this; matrix transposition on M_n is the standard warning.

The six Lean items expose the boundary map as bounded linear data, its norm estimate, unitality, star compatibility, and positivity.

## Worked examples

**Example 1.** If D_j≥0 and weights w_j≥0, then Φ(h)=∑w_jh_jD_j is positive for h_j≥0. This finite sum is the discrete model of the boundary integral.

**Example 2.** Transposition preserves eigenvalues and hence positivity of individual matrices, but id_2⊗transpose fails on an entangled rank-one projector. Thus positive does not imply completely positive.

## ML bridge

Kernel layers and covariance operators often preserve PSD matrices by congruence or averaging. Complete positivity is a stronger, compositional guarantee: it remains valid after coupling the system to an auxiliary feature space. The distinction mirrors why a scalar theorem may not survive batched matrix-valued amplification.

## Lean translation

The formal boundary used here is #check CrouzeixTextbook.Part04.boundary_positive_map_preserves_psd. It proves the exact positivity needed by the double-layer construction. The chapter deliberately does not infer complete positivity unless an amplification theorem is present.

## Exercises

### CFT-22-E01 -- retrieval {#exercise-cft-22-e01}

Define positive and completely positive maps.

### CFT-22-E02 -- calculation {#exercise-cft-22-e02}

Prove V*XV≥0 when X≥0.

### CFT-22-E03 -- written-proof {#exercise-cft-22-e03}

Prove a positive weighted sum of PSD matrices is PSD.

### CFT-22-E04 -- written-proof {#exercise-cft-22-e04}

Show that a unital *-homomorphism is completely positive.

### CFT-22-E05 -- boundary {#exercise-cft-22-e05}

Explain why transpose is a useful boundary example.

### CFT-22-E06 -- lean-proof {#exercise-cft-22-e06}

Identify which hypotheses of boundary_positive_map_unital normalize the integral mass.

### Solution sketches

For (2), evaluate the quadratic form. For (3), sum nonnegative quadratic forms. For (4), amplifications remain *-homomorphisms and send Y*Y to Φ(Y)*Φ(Y). For (5), positivity survives but an amplification fails. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part04.boundary_positive_map_unital`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter’s sixth contract declaration is a navigation handoff to Chapter 23. The compiled interfaces expose related formal statements; because correspondence remains summary/checkpoint or unmapped, they do not certify every analytic step above.
