---
id: cft-chapter-02-vector-spaces-and-subspaces
title: Vector spaces and subspaces
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, linear-structure, mathematics, lean]
confidence: high
canonical: 02_vector_spaces_and_subspaces.md
chapter: 2
part: 1
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 2: Vector spaces and subspaces

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-i-linear-structure|Part I -- Linear structure]]
Previous: [[knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations|Chapter 1]]
Next: [[knowledge/crouzeix_textbook/part_01_linear_structure/03_linear_maps_and_exact_structure|Chapter 3 — Linear maps and exact structure]]

## Opening problem

Suppose two parameter vectors produce the same predictor. Is their difference
noise, symmetry, or evidence that the parameterization is too large? Linear
algebra answers by separating a space from its subspaces, decompositions, and
quotients. The central move is to stop treating a vector as a row of numbers:
a vector is an element of a structure in which addition and scalar
multiplication obey fixed laws.

## Conceptual model

A subspace is a closed world inside a vector space. Spans build the smallest
such world from generators; intersections impose several linear constraints
at once; direct sums split a vector into independently recoverable parts; and
quotients deliberately forget a subspace. These are four views of the same
question: which distinctions should the mathematics retain?

Coordinates require a basis. A basis is simultaneously a spanning family and
an independent family. Spanning guarantees existence of coordinates;
independence guarantees uniqueness. Neither property alone is enough.

## Formal development

### CFT-02-001: span minimality {#cft-02-001}

For a set $S\subseteq V$, $\operatorname{span}(S)$ is the intersection of all
subspaces containing $S$. Hence if $S\subseteq W$, then
$\operatorname{span}(S)\subseteq W$. The proof is built into the definition:
every finite linear combination of elements of $S$ remains in $W$ because $W$
is closed under addition and scalar multiplication.

### CFT-02-002: intersections {#cft-02-002}

$x\in U\cap W$ if and only if $x\in U$ and $x\in W$. Arbitrary intersections
of subspaces are therefore subspaces. Unions usually fail: the union of the
coordinate axes in $\mathbb R^2$ does not contain $(1,1)$.

### CFT-02-003 and CFT-02-004: bases and dimension {#cft-02-003}

### CFT-02-004 locator {#cft-02-004}

If $b=(b_i)$ is a basis, the coordinate map
$\operatorname{repr}_b:V\to \mathbb K^{(I)}$ is a linear equivalence. Thus
equal coordinate functions imply equal vectors. Any linear equivalence
$V\simeq W$ transports a basis in either direction, so finite dimension is an
invariant of the vector space rather than of its presentation.

### CFT-02-005: direct sums {#cft-02-005}

If $U\cap W=\{0\}$ and $u_1+w_1=u_2+w_2$, then
$u_1-u_2=w_2-w_1$. The left side lies in $U$ and the right side in $W$; their
common value lies in the intersection and is therefore zero. Hence
$u_1=u_2$ and $w_1=w_2$. This is the load-bearing uniqueness proof.

### CFT-02-006: quotients {#cft-02-006}

The quotient $V/W$ identifies $v$ and $v+w$ for every $w\in W$. Its canonical
map $q:V\to V/W$ is surjective because every equivalence class has a
representative. Its kernel is precisely $W$.

## Worked examples

In $\mathbb R^3$, let $U=\operatorname{span}(e_1,e_2)$ and
$W=\operatorname{span}(e_3)$. Their intersection is zero and every
$(a,b,c)$ has the unique decomposition $(a,b,0)+(0,0,c)$.

For the second example, quotient $\mathbb R^2$ by the diagonal line
$D=\operatorname{span}(1,1)$. The functional $f(x,y)=x-y$ is constant on
cosets of $D$, and it identifies $\mathbb R^2/D$ with $\mathbb R$.

## ML bridge

Feature maps often have redundant directions. Passing from parameter space
to the quotient by the predictor's null directions records functions rather
than parameter representatives. Direct sums describe independent feature
blocks only when the intersection is zero; concatenating arrays does not by
itself establish that independence.

## Lean translation

Lean writes a subspace as `Submodule 𝕜 V`, a basis as
`Module.Basis ι 𝕜 V`, and the quotient projection as `W.mkQ`. The declaration
`direct_sum_coordinates_unique` formalizes every membership step in the
uniqueness proof; `quotient_projection_surjective` checks the representative
argument.

## Exercises

### CFT-02-E01 -- retrieval {#exercise-cft-02-e01}

State the two closure laws that make a nonempty subset a subspace.

### CFT-02-E02 -- calculation {#exercise-cft-02-e02}

Compute the span of $(1,1)$ and $(1,-1)$ in $\mathbb R^2$.

### CFT-02-E03 -- written-proof {#exercise-cft-02-e03}

Prove span minimality from closure under finite linear combinations.

### CFT-02-E04 -- written-proof {#exercise-cft-02-e04}

Prove uniqueness of a direct-sum decomposition.

### CFT-02-E05 -- boundary {#exercise-cft-02-e05}

Give two subspaces whose union is not a subspace.

### CFT-02-E06 -- lean-proof {#exercise-cft-02-e06}

In Lean, prove `Submodule.span 𝕜 s ≤ W` from `s ⊆ W`.

Solutions: (1) closure under addition and scalar multiplication, with zero
following from scalar multiplication; (2) the determinant is $-2$, so the
span is all of $\mathbb R^2$; (3) induct on a finite linear combination; (4)
use the intersection argument above; (5) the two coordinate axes; (6) apply
`span_minimality`. Related compiled navigation checkpoints are the six public
declarations `CFT-02-001` through `CFT-02-006`. No exercise in this chapter
currently has a distinct checked Lean solution declaration; the named
declarations do not claim reviewed exact prose correspondence.

## Synthesis and forward dependencies

Span, kernel, range, direct sum, and quotient are the structural nouns used by
every later chapter. Chapter 3 turns them into a calculus for linear maps.
