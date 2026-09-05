---
id: cft-chapter-03-linear-maps-and-exact-structure
title: Linear maps and exact structure
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, linear-structure, mathematics, lean]
confidence: high
canonical: 03_linear_maps_and_exact_structure.md
chapter: 3
part: 1
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 3: Linear maps and exact structure

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-i-linear-structure|Part I -- Linear structure]]
Previous: [[knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces|Chapter 2]]
Next: [[knowledge/crouzeix_textbook/part_01_linear_structure/04_coordinates_and_duality|Chapter 4 — Coordinates and duality]]

## Opening problem

A linear layer maps many inputs to outputs. Which input directions disappear,
which outputs are attainable, and what happens when layers are composed? The
kernel and range answer these questions without choosing coordinates.

## Conceptual model

The kernel is lost information; the range is attainable information.
Rank--nullity is a conservation law: in finite dimensions, the dimension of
the input is partitioned into directions that survive and directions that
vanish. An invariant subspace is a subsystem closed under the dynamics.

## Formal development

### CFT-03-001 and CFT-03-002: kernel and range {#cft-03-001}

### CFT-03-002 locator {#cft-03-002}

For $T:V\to W$,
$\ker T=\{x:T x=0\}$ and
$\operatorname{ran}T=\{T x:x\in V\}$. Both are subspaces: linearity sends
linear combinations of kernel elements to zero and linear combinations of
images to images of the corresponding combinations.

### CFT-03-003: rank--nullity {#cft-03-003}

Choose a basis $k_1,\ldots,k_r$ of $\ker T$ and extend it to a basis
$k_1,\ldots,k_r,v_1,\ldots,v_s$ of $V$. The vectors $T v_i$ span the range.
They are independent: if $\sum a_iT v_i=0$, then $\sum a_iv_i$ lies in the
kernel, and uniqueness in the extended basis forces every $a_i=0$. Therefore
$\dim V=\dim\ker T+\dim\operatorname{ran}T$.

### CFT-03-004: injectivity {#cft-03-004}

$T$ is injective exactly when $\ker T=\{0\}$. Indeed, $T x=T y$ is equivalent
to $T(x-y)=0$. This one-line reduction is a recurring proof pattern.

### CFT-03-005: invariant restriction {#cft-03-005}

If $T(W)\subseteq W$, the restriction $T|_W:W\to W$ is an endomorphism. Its
codomain is genuinely $W$, not merely $V$; Lean forces us to provide the
invariance proof that ordinary notation often hides.

### CFT-03-006: composition rank bound {#cft-03-006}

For $U\xrightarrow{S}V\xrightarrow{T}W$,
$\operatorname{ran}(T\circ S)\subseteq\operatorname{ran}T$, hence
$\operatorname{rank}(T\circ S)\le\operatorname{rank}T$. Symmetrically, its
rank is at most the rank of $S$.

## Worked examples

For $T(x,y,z)=(x+y,y+z)$, solving $T(x,y,z)=0$ gives
$(x,y,z)=t(1,-1,1)$, so nullity is one. The two columns $(1,0)$ and $(1,1)$
are independent, so rank is two; $3=1+2$.

For a square-zero map $N$ with $N^2=0$, every vector in the range lies in the
kernel because $N(Nx)=0$. Thus $\operatorname{ran}N\subseteq\ker N$ and
$\operatorname{rank}N\le\frac12\dim V$.

## ML bridge

The Jacobian kernel contains locally invisible perturbations; its range
contains locally reachable output velocities. Rank deficiency is structural,
not merely numerical, although floating-point thresholds are needed to infer
it from data.

## Lean translation

`T.ker` and `T.range` are submodules. The rank--nullity theorem uses the
typeclass `Module.Finite 𝕜 V`. `invariantRestriction` returns a map between
subtypes, so the invariance premise appears explicitly in its type.

## Exercises

### CFT-03-E01 -- retrieval {#exercise-cft-03-e01}

Interpret the kernel of a feature projection.

### CFT-03-E02 -- calculation {#exercise-cft-03-e02}

Find the rank and kernel of $\begin{pmatrix}1&2\\2&4\end{pmatrix}$.

### CFT-03-E03 -- written-proof {#exercise-cft-03-e03}

Prove the injectivity/kernel equivalence.

### CFT-03-E04 -- written-proof {#exercise-cft-03-e04}

Prove $\operatorname{ran}(TS)\subseteq\operatorname{ran}T$.

### CFT-03-E05 -- boundary {#exercise-cft-03-e05}

Give maps for which the composition rank inequality is strict.

### CFT-03-E06 -- lean-proof {#exercise-cft-03-e06}

Formalize the range inclusion in Lean.

Solutions: (1) invisible feature directions; (2) rank one and kernel
$\operatorname{span}(-2,1)$; (3) subtract two equal-output inputs; (4) write
$TSx=T(Sx)$; (5) take $S=0$ and $T=I$; (6) use
`composition_range_le`. Related compiled navigation checkpoints are
`CFT-03-001` through `CFT-03-006`. No exercise in this chapter currently has a
distinct checked Lean solution declaration; the named declarations do not
claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

Kernel, range, restriction, and composition are now basis-free. Chapter 4
shows how coordinates and dual functionals represent this structure.
