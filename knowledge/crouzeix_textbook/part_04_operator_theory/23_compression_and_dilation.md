---
id: cft-chapter-23-compression-and-dilation
title: Compression and dilation
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, operator-theory, mathematics, lean]
confidence: high
canonical: 23_compression_and_dilation.md
chapter: 23
part: 4
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 23: Compression and dilation

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iv-finite-dimensional-operator-theory|Part IV — Finite-dimensional operator theory]]
Previous: [[knowledge/crouzeix_textbook/part_04_operator_theory/22_positive_and_completely_positive_maps|Chapter 22 — Positive and completely positive maps]]
Next: [[knowledge/crouzeix_textbook/part_04_operator_theory/24_gramians_and_ordered_matrix_inequalities|Chapter 24 — Gramians and ordered matrix inequalities]]

## Opening problem

Can a difficult operator be understood as the visible corner of a simpler operator on a larger space? Dilation theory answers yes in carefully structured situations, and the constant two appears when a contraction’s powers are compressed and doubled.

## Conceptual model

If V:E→K is an isometry and Q acts on K, the compression is V*QV. A dilation matches not just one operator but a family of moments: T^n=V*Q^nV. Compression cannot increase norm. Dilation turns algebra on E into geometry on K, where Q may be contractive, unitary, or multiplicative.

## Formal development

### The six-item spine

### CFT-23-001 — compressed adjoint power {#cft-23-001}

### CFT-23-002 — compressed adjoint power contractive {#cft-23-002}

### CFT-23-003 — doubled compression norm two {#cft-23-003}

### CFT-23-004 — adjacent power defect factorization {#cft-23-004}

### CFT-23-005 — perturbation commutes {#cft-23-005}

### CFT-23-006 — target power bound {#cft-23-006}

The checked items are CFT-23-001, CFT-23-002, CFT-23-003, CFT-23-004, CFT-23-005, CFT-23-006.

**Theorem 23.1 (compression bound).** If V is an isometry, then ‖V*QV‖≤‖Q‖. This follows from ‖V‖=‖V*‖=1 and submultiplicativity.

**Definition 23.2 (power dilation).** Q dilates T through V when T^n=V*Q^nV for all n≥0. Then ‖p(T)‖≤‖p(Q)‖ for polynomials p by linearity.

**Theorem 23.3 (doubled compression).** If ‖Q‖≤1, then ‖2V*(Q*)^nV‖≤2. This is the rigid part of the Lorist–Schwenninger decomposition.

**Definition 23.4 (adjacent-power defect).** When the target powers are represented as doubled compressed powers minus perturbations E_n, compare consecutive n. The defect factors through the dilation displacement and supports a recurrence after testing on a norm-attaining vector.

The formal items expose compressed adjoint powers, their one and two bounds, the adjacent defect factorization, commutation of perturbations with the target, and the preliminary target-power estimate.

## Worked examples

**Example 1.** Let $E$ be the first coordinate in $K=\mathbb C^2$, $Vx=(x,0)$, and $Q=\begin{pmatrix}a&b\\c&d\end{pmatrix}$. Then $V^*QV$ is scalar multiplication by $a$. The full operator can be complicated while the compression sees one corner.

**Example 2.** The unilateral shift dilates strict contractions in classical Sz.-Nagy theory. In finite matrix language, a block unitary can encode a contraction so that powers of the contraction arise as compressed powers.

## ML bridge

Latent-state augmentation is a computational analogue of dilation: a non-Markovian or constrained visible map can become simple on an expanded state. The theorem is stricter than architecture augmentation, however—it requires exact moment identities and norm control, not just expressive equivalence.

## Lean translation

CrouzeixTextbook.Part04.adjacent_power_defect_factorization is an exact operator identity in the Lorist–Schwenninger data structure. The structure bundles isometry, contraction, uniform perturbation bound, and commutation, preventing later recurrence steps from silently assuming any of them.

## Exercises

### CFT-23-E01 -- retrieval {#exercise-cft-23-e01}

Define compression through an isometry.

### CFT-23-E02 -- calculation {#exercise-cft-23-e02}

Prove the compression norm bound.

### CFT-23-E03 -- written-proof {#exercise-cft-23-e03}

Show that an exact power dilation transfers polynomial norms.

### CFT-23-E04 -- written-proof {#exercise-cft-23-e04}

Explain why V*V=I but usually VV*≠I.

### CFT-23-E05 -- boundary {#exercise-cft-23-e05}

Give a compression whose numerical range is strictly smaller than the original operator’s.

### CFT-23-E06 -- lean-proof {#exercise-cft-23-e06}

Inspect perturbation_commutes and explain why commutation matters when moving E_n past powers of T.

### Solution sketches

For (2), apply submultiplicativity and ‖V‖=‖V*‖=1. For (3), sum the moment identities. For (4), VV* projects onto range V. For (5), compress diag(-1,1) to one coordinate. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part04.perturbation_commutes`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named field does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter’s sixth contract declaration is a navigation handoff to Chapter 24. The compiled interfaces expose related formal statements; because correspondence remains summary/checkpoint or unmapped, they do not certify every analytic step above.
