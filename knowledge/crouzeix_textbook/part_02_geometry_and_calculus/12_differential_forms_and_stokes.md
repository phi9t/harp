---
id: cft-chapter-12-differential-forms-and-stokes
title: Differential forms and Stokes
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 12_differential_forms_and_stokes.md
chapter: 12
part: 2
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 12: Differential forms and Stokes

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/11_differentiation_as_linear_approximation|Chapter 11 — Differentiation as linear approximation]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/13_metric_and_normed_spaces|Chapter 13 — Metric and normed spaces]]

## Opening problem

How do we turn linear structure into quantitative geometry? This chapter develops pullbacks, exterior derivatives, orientation, and the supported boundary calculus. The goal is not a catalogue of formulas but a chain of proofs whose hypotheses remain visible.

## Conceptual model

Algebra says which combinations are legal. Geometry adds a rule for measuring size, angle, orientation, or first-order change. Each new identification is extra structure: a covector is not automatically a vector, a transpose is not automatically an adjoint, and a coordinate formula is not automatically basis-free.

## Formal development

### The six-item spine

### CFT-12-001 — real inner complex coordinates {#cft-12-001}

### CFT-12-002 — area form quarter turn {#cft-12-002}

### CFT-12-003 — oriented radial tangent positive {#cft-12-003}

### CFT-12-004 — oriented tangent formula {#cft-12-004}

### CFT-12-005 — boundary parameter is smooth {#cft-12-005}

### CFT-12-006 — boundary slice derivative {#cft-12-006}

The chapter tracks six formal items, `CFT-12-001`, `CFT-12-002`, `CFT-12-003`, `CFT-12-004`, `CFT-12-005`, `CFT-12-006`. A differential $k$-form assigns an alternating $k$-covector smoothly at each point. Pullback composes the form with derivatives and is functorial. Antisymmetry makes $d\circ d=0$. Stokes says that integrating $d\omega$ over an oriented domain equals integrating $\omega$ over its induced boundary. For this book the formal surface is the exact planar complex orientation and smooth radial-boundary derivative later used in the double-layer contour; the broader manifold theorem is explanatory context, not an imported proof claim.

The proof discipline is consistent: expand the relevant definition, isolate the term controlled by positivity, orthogonality, multilinearity, or the small-o remainder, and only then invoke finite-dimensional compactness or coordinates. This prevents a later operator inequality from silently assuming normality or commutativity.

### Proof workshop: from the fundamental theorem to Stokes

A smooth differential $k$-form assigns an alternating $k$-covector
$\omega_x$ at each point. A smooth map $F$ pulls it back by inserting
$DF(x)$ into every slot:
$(F^*\omega)_x(v_1,\ldots,v_k)
=\omega_{F(x)}(DF(x)v_1,\ldots,DF(x)v_k)$.
The exterior derivative is the unique local operator extending the scalar
differential, satisfying $d^2=0$ and the graded Leibniz rule.

For a rectangle in $\mathbb R^2$ and
$\omega=P\,dx+Q\,dy$, direct one-variable fundamental-theorem
calculations give
$\int_{\partial R}\omega
=\iint_R(\partial_xQ-\partial_yP)\,dx\,dy
=\int_Rd\omega$.
Triangulate a smooth planar domain: integrals over shared interior edges
cancel because the induced orientations oppose, leaving only the outer
boundary. Partitions of unity and charts generalize this mechanism to
manifolds.

In the complex plane, multiplication by $i$ rotates a unit outward normal
through a quarter turn into the positively oriented tangent. The sign is not
cosmetic: reversing the boundary orientation negates the contour integral.
The maintained Lean surface proves the exact complex area-form and radial
tangent identities, plus smoothness and the boundary-slice derivative needed
by the later parametric contour.

## Worked examples

**Example 1.** For the unit disk with counterclockwise boundary,
the outward normal at $e^{it}$ is $e^{it}$ and the tangent is
$ie^{it}$. Their oriented area is positive. Clockwise parametrization uses
$-ie^{it}$ and reverses every boundary integral.

**Example 2.** Let $\omega=-y\,dx+x\,dy$. Then
$d\omega=2\,dx\wedge dy$. On the unit disk, Stokes gives
$\int_{\partial D}\omega=2\operatorname{area}(D)=2\pi$, matching the
direct parametrization $x=\cos t,y=\sin t$.

## ML bridge

Continuous normalizing flows use divergence and change of variables; this chapter supplies the geometric vocabulary while keeping numerical ODE and estimator claims outside the theorem boundary.

## Lean translation

`CrouzeixTextbook.Part02.Chapter12` exposes six checked declarations. They reuse maintained Harp or Mathlib results only at matching statement boundaries. For Chapters 11--12, the formal scope is deliberately finite-coordinate or planar; it does not pretend to formalize a JAX runtime or the full manifold-level Stokes theorem.

## Exercises

### CFT-12-E01 -- retrieval {#exercise-cft-12-e01}

Define pullback of a differential form.

### CFT-12-E02 -- calculation {#exercise-cft-12-e02}

Compute $d(P\,dx+Q\,dy)$.

### CFT-12-E03 -- written-proof {#exercise-cft-12-e03}

Prove Stokes on a rectangle from the one-variable fundamental theorem.

### CFT-12-E04 -- written-proof {#exercise-cft-12-e04}

Explain cancellation of interior edges in a triangulation.

### CFT-12-E05 -- boundary {#exercise-cft-12-e05}

Show that reversing boundary orientation negates the integral.

### CFT-12-E06 -- lean-proof {#exercise-cft-12-e06}

Inspect oriented_tangent_formula and match its normal, tangent, speed, and sign hypotheses.

### Solution sketches

For (2), obtain $(\partial_xQ-\partial_yP)dx\wedge dy$. For (3), integrate the four edges and group the two fundamental-theorem identities. For (4), each shared edge appears twice with opposite direction. For (5), change variables under the reversed parametrization. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part02.oriented_tangent_formula`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter contributes its six-item spine to the dependency path from inner-product geometry through differentiation and oriented boundaries. The next chapter uses only the structures introduced so far.
