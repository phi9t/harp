---
id: cft-chapter-04-coordinates-and-duality
title: Coordinates and duality
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, linear-structure, mathematics, lean]
confidence: high
canonical: 04_coordinates_and_duality.md
chapter: 4
part: 1
lean_exercise_solution_declarations: 0
lean_exact_correspondences: 0
---

# Chapter 4: Coordinates and duality

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-i-linear-structure|Part I -- Linear structure]]
Previous: [[knowledge/crouzeix_textbook/part_01_linear_structure/03_linear_maps_and_exact_structure|Chapter 3]]
Next: [[knowledge/crouzeix_textbook/part_01_linear_structure/05_determinants_trace_and_exterior_algebra|Chapter 5 — Determinants, trace, and exterior algebra]]

## Opening problem

Backpropagation sends output covectors toward inputs. Why does the order of
maps reverse, and why is a matrix transpose involved? The answer is duality,
not an arbitrary array convention.

## Conceptual model

The dual space $V^*$ consists of linear measurements $V\to\mathbb K$. A map
$T:V\to W$ pulls a measurement $\phi\in W^*$ back to
$T^*\phi=\phi\circ T\in V^*$. Data moves forward; questions move backward.
That reversal explains both transpose matrices and reverse-mode autodiff.

## Formal development

### CFT-04-001 and CFT-04-002: pullback {#cft-04-001}

### CFT-04-002 locator {#cft-04-002}

By definition, $(T^*\phi)(x)=\phi(Tx)$. For $S:U\to V$ and $T:V\to W$,
evaluation at $u$ gives
$((T\circ S)^*\phi)(u)=\phi(T(Su))=(S^*(T^*\phi))(u)$; extensionality yields
$(T\circ S)^*=S^*\circ T^*$.

### Dual bases and annihilators

For a finite basis $(b_i)$, the dual basis $(b^i)$ satisfies
$b^i(b_j)=\delta_{ij}$. If $U\le V$, its annihilator
$U^\circ=\{\phi:\phi(u)=0\ \forall u\in U\}$ has dimension
$\dim V-\dim U$: extend a basis of $U$ to one of $V$ and select the dual
functionals belonging to the added vectors.

### CFT-04-003 and CFT-04-004: coordinate conjugacy {#cft-04-003}

### CFT-04-004 locator {#cft-04-004}

Changing coordinates by $S$ sends $A$ to $SAS^{-1}$. The calculation from
Chapter 1 proves that the new matrix acts consistently. Because similar
matrices describe the same operator, their characteristic polynomials agree.

### CFT-04-005 and CFT-04-006: scalar invariants {#cft-04-005}

### CFT-04-006 locator {#cft-04-006}

Determinant and trace are also unchanged by similarity. These facts will be
proved from multiplicativity and cyclicity in Chapter 5; here they serve as
the first examples of quantities that forget the chosen basis.

## Worked examples

For $T(x,y)=(x+2y,3y)$ and $\phi(a,b)=4a-b$,
$T^*\phi(x,y)=4(x+2y)-3y=4x+5y$. In coordinates, the coefficient column
$(4,-1)$ is multiplied by $T^{\mathsf T}$.

For $U=\operatorname{span}(1,1,0)$ in $\mathbb R^3$, the annihilator is the
plane of functionals $(a,b,c)$ with $a+b=0$, hence has dimension two.

## ML bridge

A gradient is naturally a covector: it eats a perturbation and returns a
first-order change. Identifying it with a vector uses an inner product, which
does not arrive until Chapter 7. Reverse-mode products are pullbacks even when
no Euclidean identification is appropriate.

## Lean translation

Lean calls the pullback `T.dualMap`. The theorem `dual_map_composition`
exposes contravariance. The similarity declarations reuse the already checked
coordinate-action theorem and Mathlib's determinant, trace, and characteristic
polynomial conjugacy lemmas.

## Exercises

### CFT-04-E01 -- retrieval {#exercise-cft-04-e01}

Define the dual space without coordinates.

### CFT-04-E02 -- calculation {#exercise-cft-04-e02}

Compute the pullback in the first example.

### CFT-04-E03 -- written-proof {#exercise-cft-04-e03}

Prove dualization reverses composition.

### CFT-04-E04 -- written-proof {#exercise-cft-04-e04}

Construct the dual basis of the standard basis of $\mathbb R^n$.

### CFT-04-E05 -- boundary {#exercise-cft-04-e05}

Show that transposing in the same order is generally wrong.

### CFT-04-E06 -- lean-proof {#exercise-cft-04-e06}

Check `dual_map_composition` in Lean and explain each type.

Solutions: (1) linear maps to the scalar field; (2) $(4,5)$; (3) evaluate at
an arbitrary input; (4) coordinate projections; (5) rectangular dimensions
already expose the mismatch; (6) the source and target duals reverse. Related
compiled navigation checkpoints are `CFT-04-001` through `CFT-04-006`. No
exercise in this chapter currently has a distinct checked Lean solution
declaration; the named declarations do not claim reviewed exact prose
correspondence.

## Synthesis and forward dependencies

Duality explains transpose and reverse flow. Chapter 5 develops determinant
and trace, the two similarity invariants used constantly in spectral theory.
