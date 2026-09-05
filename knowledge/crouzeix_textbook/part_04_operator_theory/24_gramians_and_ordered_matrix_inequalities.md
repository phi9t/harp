---
id: cft-chapter-24-gramians-and-ordered-matrix-inequalities
title: Gramians and ordered matrix inequalities
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, operator-theory, mathematics, lean]
confidence: high
canonical: 24_gramians_and_ordered_matrix_inequalities.md
chapter: 24
part: 4
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 24: Gramians and ordered matrix inequalities

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iv-finite-dimensional-operator-theory|Part IV — Finite-dimensional operator theory]]
Previous: [[knowledge/crouzeix_textbook/part_04_operator_theory/23_compression_and_dilation|Chapter 23 — Compression and dilation]]
Next: [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/25_convex_boundaries_and_cauchy_layers|Chapter 25 — Convex boundaries and Cauchy layers]]

## Opening problem

How can infinitely many power bounds be condensed into one positive matrix? A weighted Gramian sums the energies of all iterates, and differences of two such sums isolate the constant that will become two.

## Conceptual model

For q>1 and a matrix C, define G_q(C)=∑_{k≥0}q^{-k}(C*)^kC^k. Every term is PSD. If powers are uniformly bounded, the series converges in finite-dimensional matrix norm. The Gramian solves a Lyapunov-type identity and converts power information into ordered matrix inequalities.

## Formal development

### The six-item spine

### CFT-24-001 — gramian term {#cft-24-001}

### CFT-24-002 — gramian term positive {#cft-24-002}

### CFT-24-003 — gramian terms summable {#cft-24-003}

### CFT-24-004 — weighted gramian {#cft-24-004}

### CFT-24-005 — weighted gramian positive {#cft-24-005}

### CFT-24-006 — gramian difference positive {#cft-24-006}

The checked items are CFT-24-001, CFT-24-002, CFT-24-003, CFT-24-004, CFT-24-005, CFT-24-006.

**Theorem 24.1 (term positivity).** Each (C*)^kC^k is PSD because it is X*X with X=C^k. Positive scalar weights preserve PSD.

**Theorem 24.2 (summability).** If ‖C^k‖≤M, then ‖q^{-k}(C*)^kC^k‖≤q^{-k}M². The scalar geometric series converges for q>1, so the matrix series converges absolutely.

**Theorem 24.3 (PSD limit).** A norm limit of PSD matrices is PSD. For each vector x, quadratic evaluation is continuous; the limit of nonnegative real numbers is nonnegative.

**Theorem 24.4 (ordered difference).** Comparing q=2 and q=4 gives coefficients 2^{-k}-4^{-k}≥0. Therefore G₂(C)-G₄(C)≥0, and subtracting selected initial terms reveals sharper inequalities. The coefficient at k=1 is 1/2-1/4=1/4; after rearrangement this is where a factor 4, hence a norm bound 2, emerges.

CFT-24-001--006 compile the term, positivity, summability, total Gramian, positivity of the sum, and the key Gramian difference.

## Worked examples

**Example 1.** For scalar C=c with |c|<√q, G_q(c)=1/[1-|c|²/q]. The matrix construction generalizes this energy denominator without diagonalizing C.

**Example 2.** If C²=0, then G_q(C)=I+q^{-1}C*C. Thus G₂-G₄=(1/4)C*C≥0 exactly, with no tail.

## ML bridge

Controllability and observability Gramians summarize accumulated dynamics; analogous matrices appear in recurrent-network stability and implicit-layer conditioning. Weight choice trades sensitivity to long horizons against convergence. Here the special weights 2 and 4 are algebraically tuned to the sharp endpoint.

## Lean translation

The theorem #check CrouzeixTextbook.Part04.gramian_difference_positive formalizes positivity of an infinite-series difference, not merely a finite truncation. Its hypotheses expose the uniform power bound used to justify summability.

## Exercises

### CFT-24-E01 -- retrieval {#exercise-cft-24-e01}

Define PSD order A≤B.

### CFT-24-E02 -- calculation {#exercise-cft-24-e02}

Compute G_q(c) in the scalar case.

### CFT-24-E03 -- written-proof {#exercise-cft-24-e03}

Prove each Gramian term is PSD.

### CFT-24-E04 -- written-proof {#exercise-cft-24-e04}

Prove absolute convergence from a uniform power bound.

### CFT-24-E05 -- boundary {#exercise-cft-24-e05}

Show G₂-G₄≥0 coefficientwise.

### CFT-24-E06 -- lean-proof {#exercise-cft-24-e06}

Explain how the k=1 coefficient converts a matrix inequality into ‖C‖≤2.

### Solution sketches

For (2), sum a scalar geometric series. For (3), test X*X on vectors. For (4), compare norms with M²∑q^{-k}. For (5), every coefficient is nonnegative. For (6), isolate (1/4)C*C≤I and take the operator norm. A related compiled navigation checkpoint is `CrouzeixTextbook.Part04.gramian_difference_positive`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter’s sixth contract declaration is a navigation handoff to Chapter 25. The compiled interfaces expose related formal statements; because correspondence remains summary/checkpoint or unmapped, they do not certify every analytic step above.
