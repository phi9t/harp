---
id: cft-chapter-10-multilinear-maps-and-tensors
title: Multilinear maps and tensors
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 10_multilinear_maps_and_tensors.md
chapter: 10
part: 2
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 10: Multilinear maps and tensors

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/09_operator_norms_and_singular_values|Chapter 9 — Operator norms and singular values]]
Next: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/11_differentiation_as_linear_approximation|Chapter 11 — Differentiation as linear approximation]]

## Opening problem

How do we turn linear structure into quantitative geometry? This chapter develops bilinear factorization, tensor coordinates, alternating forms, and contraction. The goal is not a catalogue of formulas but a chain of proofs whose hypotheses remain visible.

## Conceptual model

Algebra says which combinations are legal. Geometry adds a rule for measuring size, angle, orientation, or first-order change. Each new identification is extra structure: a covector is not automatically a vector, a transpose is not automatically an adjoint, and a coordinate formula is not automatically basis-free.

## Formal development

### The six-item spine

### CFT-10-001 — bilinear pairing duality {#cft-10-001}

### CFT-10-002 — transpose coordinate action {#cft-10-002}

### CFT-10-003 — determinant top degree multiplicative {#cft-10-003}

### CFT-10-004 — alternating diagonal volume {#cft-10-004}

### CFT-10-005 — contraction trace cyclic {#cft-10-005}

### CFT-10-006 — wedge sign kernel {#cft-10-006}

The chapter tracks six formal items, `CFT-10-001`, `CFT-10-002`, `CFT-10-003`, `CFT-10-004`, `CFT-10-005`, `CFT-10-006`. A bilinear map is linear in each input separately and therefore factors through a tensor product. Alternation projects away repeated directions: $v\wedge v=0$ and $v\wedge w=-w\wedge v$. The top exterior action is one-dimensional and its scalar is the determinant. Contraction pairs one covariant and one contravariant slot; in coordinates the resulting finite sums explain transpose duality and cyclic trace.

The proof discipline is consistent: expand the relevant definition, isolate the term controlled by positivity, orthogonality, multilinearity, or the small-o remainder, and only then invoke finite-dimensional compactness or coordinates. This prevents a later operator inequality from silently assuming normality or commutativity.

### Proof workshop: multilinearity before coordinates

A bilinear map $B:U\times V\to W$ is linear in each argument while the
other is held fixed. The tensor product $U\otimes V$ is characterized by a
universal property: every bilinear $B$ factors uniquely through a linear map
$\widetilde B:U\otimes V\to W$. This property, not a multidimensional
array layout, defines the tensor product.

Alternating multilinear maps vanish whenever two arguments agree. Swapping
two adjacent arguments changes the sign; any permutation contributes its
sign. The exterior power $\bigwedge^kV$ represents alternating
$k$-linear maps. In top degree, a linear map $A:V\to V$ induces a map on the
one-dimensional space $\bigwedge^nV$, so it must be scalar multiplication.
That scalar is $\det A$. Functoriality immediately gives
$\det(AB)=\det A\det B$.

Contraction pairs a vector slot with a covector slot. Matrix multiplication,
trace, Jacobian-vector products, and vector-Jacobian products are all
contractions with different variance. The equality
$\langle Jv,w\rangle=\langle v,J^*w\rangle$ explains forward/reverse
duality without conflating vectors and covectors.

## Worked examples

**Example 1.** The area form on $\mathbb R^2$ is
$\omega(u,v)=u_1v_2-u_2v_1$. It is bilinear and alternating. Swapping
arguments negates it, and applying a matrix $A$ to both inputs multiplies it
by $\det A$.

**Example 2.** If $J$ is an $m\times n$ Jacobian, a JVP contracts its input
index with $v\in\mathbb R^n$; a VJP contracts its output index with a
covector $w\in(\mathbb R^m)^*$. Their pairing equality is a finite
reindexing of the same double sum.

## ML bridge

Attention scores and second-order models use multilinear contractions, but array axes acquire mathematical meaning only after domain, codomain, and variance are specified.

## Lean translation

`CrouzeixTextbook.Part02.Chapter10` exposes six checked declarations. They reuse maintained Harp or Mathlib results only at matching statement boundaries. For Chapters 11--12, the formal scope is deliberately finite-coordinate or planar; it does not pretend to formalize a JAX runtime or the full manifold-level Stokes theorem.

## Exercises

### CFT-10-E01 -- retrieval {#exercise-cft-10-e01}

Define a bilinear map and an alternating map.

### CFT-10-E02 -- calculation {#exercise-cft-10-e02}

Compute the area form on $(1,2)$ and $(3,4)$.

### CFT-10-E03 -- written-proof {#exercise-cft-10-e03}

Prove an alternating bilinear map changes sign when arguments swap.

### CFT-10-E04 -- written-proof {#exercise-cft-10-e04}

Derive determinant multiplicativity from the top exterior power.

### CFT-10-E05 -- boundary {#exercise-cft-10-e05}

Give a bilinear map that cannot be written as a linear map on the Cartesian product.

### CFT-10-E06 -- lean-proof {#exercise-cft-10-e06}

Use bilinear_pairing_duality to expand the JVP/VJP equality as finite sums.

### Solution sketches

For (2), the value is $-2$. For (3), expand $0=\omega(u+v,u+v)$. For (4), use functoriality of $\bigwedge^n$. For (5), scalar multiplication $(x,y)\mapsto xy$ is bilinear but not linear on the product space. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part02.bilinear_pairing_duality`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter contributes its six-item spine to the dependency path from inner-product geometry through differentiation and oriented boundaries. The next chapter uses only the structures introduced so far.
