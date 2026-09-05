---
id: cft-chapter-11-differentiation-as-linear-approximation
title: Differentiation as linear approximation
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 11_differentiation_as_linear_approximation.md
chapter: 11
part: 2
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 11: Differentiation as linear approximation

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/10_multilinear_maps_and_tensors|Chapter 10 — Multilinear maps and tensors]]
Next: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/12_differential_forms_and_stokes|Chapter 12 — Differential forms and Stokes]]

## Opening problem

How do we turn linear structure into quantitative geometry? This chapter develops Fréchet derivatives, chain rules, JVPs, VJPs, and implicit differentiation. The goal is not a catalogue of formulas but a chain of proofs whose hypotheses remain visible.

## Conceptual model

Algebra says which combinations are legal. Geometry adds a rule for measuring size, angle, orientation, or first-order change. Each new identification is extra structure: a covector is not automatically a vector, a transpose is not automatically an adjoint, and a coordinate formula is not automatically basis-free.

## Formal development

### The six-item spine

### CFT-11-001 — gradient component contract {#cft-11-001}

### CFT-11-002 — derivative action is jvp {#cft-11-002}

### CFT-11-003 — derivative pullback is vjp {#cft-11-003}

### CFT-11-004 — hessian vector action {#cft-11-004}

### CFT-11-005 — jvp vjp duality {#cft-11-005}

### CFT-11-006 — linear approximation chain kernel {#cft-11-006}

The chapter tracks six formal items, `CFT-11-001`, `CFT-11-002`, `CFT-11-003`, `CFT-11-004`, `CFT-11-005`, `CFT-11-006`. The derivative $Df(x)$ is the unique linear map with $f(x+h)=f(x)+Df(x)h+o(\|h\|)$. Substituting two first-order expansions proves the chain rule: $D(g\circ f)(x)=Dg(f(x))Df(x)$. Bounded linear maps differentiate to themselves; product and inverse rules follow by isolating the linear term. In coordinates, JVP is Jacobian multiplication and VJP is multiplication by the transpose.

The proof discipline is consistent: expand the relevant definition, isolate the term controlled by positivity, orthogonality, multilinearity, or the small-o remainder, and only then invoke finite-dimensional compactness or coordinates. This prevents a later operator inequality from silently assuming normality or commutativity.

### Proof workshop: the derivative is a map

Fréchet differentiability of $f:E\to F$ at $x$ means there is a bounded
linear map $Df(x)$ such that
$f(x+h)=f(x)+Df(x)h+r(h)$ with
$\lVert r(h)\rVert/\lVert h\rVert\to0$. The derivative is unique:
subtract two candidate approximations, evaluate on $h=tv$, divide by $|t|$,
and let $t\to0$.

For the chain rule, write
$g(f(x+h))=g(f(x))+Dg(f(x))(Df(x)h+r_f(h))+r_g(Df(x)h+r_f(h))$.
The first remainder stays small because $Dg$ is bounded. The second is
small relative to its input, whose norm is $O(\lVert h\rVert)$. Collecting
terms leaves $Dg(f(x))\circ Df(x)$ plus $o(\lVert h\rVert)$.

A JVP evaluates $Df(x)v$. Reverse mode applies the pullback
$Df(x)^*$ to an output covector. For a scalar loss the derivative is
intrinsically a covector; an inner product identifies it with the gradient
vector. The Hessian is the derivative of that gradient only after fixing this
geometry.

## Worked examples

**Example 1.** For $f(x)=\lVert x\rVert^2$ on $\mathbb R^n$,
$f(x+h)-f(x)=2\langle x,h\rangle+\lVert h\rVert^2$. Thus
$Df(x)h=2\langle x,h\rangle$, and the remainder divided by
$\lVert h\rVert$ tends to zero.

**Example 2.** For $g(y)=\exp y$ and $f(x)=a^Tx$,
$D(g\circ f)(x)h=e^{a^Tx}a^Th$. Forward mode first computes $a^Th$;
reverse mode pulls the scalar cotangent back to $e^{a^Tx}a$.

## ML bridge

Forward mode transports tangents and reverse mode pulls back covectors. The Lean layer checks this finite-coordinate algebra, not tracing semantics, pytrees, or floating-point execution.

## Lean translation

`CrouzeixTextbook.Part02.Chapter11` exposes six checked declarations. They reuse maintained Harp or Mathlib results only at matching statement boundaries. For Chapters 11--12, the formal scope is deliberately finite-coordinate or planar; it does not pretend to formalize a JAX runtime or the full manifold-level Stokes theorem.

## Exercises

### CFT-11-E01 -- retrieval {#exercise-cft-11-e01}

State Fréchet differentiability with a small-o remainder.

### CFT-11-E02 -- calculation {#exercise-cft-11-e02}

Compute the derivative of $x\mapsto\lVert x\rVert^2$.

### CFT-11-E03 -- written-proof {#exercise-cft-11-e03}

Prove uniqueness of the Fréchet derivative.

### CFT-11-E04 -- written-proof {#exercise-cft-11-e04}

Prove the chain rule by composing remainder estimates.

### CFT-11-E05 -- boundary {#exercise-cft-11-e05}

Give a directionally differentiable function that is not Fréchet differentiable at zero.

### CFT-11-E06 -- lean-proof {#exercise-cft-11-e06}

Use jvp_vjp_duality to explain why forward and reverse mode compute adjoint contractions.

### Solution sketches

For (2), expand the square. For (3), test the difference of candidates on $tv$. For (4), use boundedness of the outer derivative and $Df(x)h+r_f(h)=O(\lVert h\rVert)$. For (5), use $f(x,y)=x^3/(x^2+y^2)$ away from zero, extended by zero. A related compiled navigation checkpoint for Exercise (6) is `CrouzeixTextbook.Part02.jvp_vjp_duality`. No exercise in this chapter currently has a distinct checked Lean solution declaration; the named declaration does not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

This chapter contributes its six-item spine to the dependency path from inner-product geometry through differentiation and oriented boundaries. The next chapter uses only the structures introduced so far.
