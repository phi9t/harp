---
id: cft-chapter-10-multilinear-maps-and-tensors
title: Multilinear maps and tensors
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-09
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 10_multilinear_maps_and_tensors.md
chapter: 10
part: 2
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 10: Multilinear maps and tensors

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/09_operator_norms_and_singular_values|Chapter 9]]
Next: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/11_differentiation_as_linear_approximation|Chapter 11 — Differentiation as linear approximation]]

## Opening problem

Chapter 5 built the determinant from a function of several vector arguments that
is linear in each one separately. Chapter 7 built geometry from a pairing that
is linear in each of two arguments. Chapter 8's Gram matrix is the component
array of such a pairing. The same shape keeps recurring, and this chapter names
it: a multilinear map, and the object that classifies all of them.

**Motivation.** Two things get conflated whenever indices multiply. One is the
*tensor product*, a universal object through which every bilinear map factors
uniquely. The other is a *reshape*, a relabeling of an index set. They are not
the same, and the difference is visible in one line: reshaping is a bijection of
indices, whereas most arrays of the product shape are not products of anything.
This chapter separates them, and separates congruence from similarity while it
is at it, because both confusions surface again in Part IV.

## Conceptual model

A map of several vector arguments is *multilinear* when it is linear in each
argument with the others held fixed. Two instances are already in hand: the
determinant as a function of the columns, from Chapter 5, and the pairing
$\langle\cdot,\cdot\rangle$ from Chapter 7.

The tensor product $M\otimes N$ is the object that turns bilinear maps into
linear ones: every bilinear $f$ factors as a *unique* linear map on
$M\otimes N$, computing on pure tensors by $x\otimes y\mapsto f(x,y)$. That
universal property is the definition. It says nothing about arrays, and it does
not require a basis.

Choosing bases turns a bilinear form into an array of components, and this is
where reshape enters. An array indexed by $\iota\times\kappa$ can be relabeled
by a bijection $\iota\times\kappa\simeq\{1,\dots,mn\}$, and that relabeling is
all a reshape is. It is not the tensor product, and the clean way to see the
difference is that an element of $M\otimes N$ need not be a pure tensor
$x\otimes y$. The identity matrix is such an element: it is a legitimate array
of the product shape and no outer product of two vectors.

*Contraction* sums a repeated index. Which index is summed is carried by the
types, not by the notation, and shape checking is the whole content of "the
axes must line up". Contracting the outer product of two matrices over their
shared axis is exactly matrix multiplication.

**Historical context.** Index notation and basis-free notation for these objects
developed alongside each other and remain in parallel use. We give both and say
which statements need a basis; no priority is claimed for either convention.

## Formal development

### A disclosure about this chapter's roster

Three of this chapter's six indexed cards — CFT-10-003, CFT-10-004 and
CFT-10-005 — are re-exports of Chapter 5's declarations, with underlying
declarations named in the contract and therefore identical type fingerprints to
[[knowledge/crouzeix_textbook/part_01_linear_structure/05_determinants_trace_and_exterior_algebra#cft-05-001|CFT-05-001]],
[[knowledge/crouzeix_textbook/part_01_linear_structure/05_determinants_trace_and_exterior_algebra#cft-05-002|CFT-05-002]]
and
[[knowledge/crouzeix_textbook/part_01_linear_structure/05_determinants_trace_and_exterior_algebra#cft-05-004|CFT-05-004]].
They are not new theorems. They are indexed here because the multilinear reading
of each is what this chapter teaches, and because the reader arriving at tensors
should see the determinant and the trace named as instances of the general
notion rather than as facts about matrices. The chapter's own mathematical
content is in the support declarations below, and the honest summary is that
half of this roster restates Part I.

### CFT-10-001: the pairing moves a matrix across as its transpose {#cft-10-001}

**Statement.** For a real $m\times n$ matrix $A$, a vector $x$ of length $n$ and
a vector $y$ of length $m$,
$$
\langle Ax,\ y\rangle=\langle x,\ A^{\mathsf T}y\rangle .
$$

**Proof.** Expand both sides into double sums over the entries. The left side is
$\sum_i\bigl(\sum_j A_{ij}x_j\bigr)y_i$ and the right side is
$\sum_j x_j\bigl(\sum_i A_{ij}y_i\bigr)$, since the $(j,i)$ entry of
$A^{\mathsf T}$ is $A_{ij}$. Distributing the outer factor into each sum makes
both sides the same finite double sum $\sum_i\sum_j A_{ij}x_jy_i$ up to the
order of summation and the order of each product; interchanging the two finite
sums and commuting each product identifies them.

**Boundary and Lean provider.** Only finiteness of the two index sets and
commutativity of the real product are used; no invertibility, no square shape,
and no metric beyond the coordinate pairing itself.
[`bilinear_pairing_duality`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L17)
is proved here, and its checked proof is exactly the displayed expansion: it
distributes both pairings into double sums, applies `Finset.sum_comm` for the
interchange, and closes by commuting the products. The card previously aliased a
declaration in the `AutodiffGeometry` namespace, which the receipt exporter does
not treat as maintained, so the validator could not check the alias target; the
local proof reproduces that provider's argument step for step.

### CFT-10-002: the covector action is the transpose action {#cft-10-002}

**Statement.** For a real $m\times n$ matrix $A$ and a covector $y$ of length
$m$, the row-vector action $y\,A$ equals $A^{\mathsf T}y$.

**Proof.** Both sides have $j$th component $\sum_i y_iA_{ij}$: on the left by
the definition of the row-vector product, on the right because the $(j,i)$ entry
of $A^{\mathsf T}$ is $A_{ij}$. The two expressions are therefore the same
function of $j$.

**Boundary and Lean provider.** This is a definitional identification, not a
theorem about geometry: it says the two notations for the same computation
agree. It is what makes CFT-10-001 readable as "the pairing moves $A$ across",
and it is the reverse-mode rule of Chapter 4 in coordinates.
[`transpose_coordinate_action`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L26)
is proved here, by rewriting with `Matrix.vecMul_transpose` and cancelling the
double transpose. The component computation displayed above is not what the
checked proof runs: it is the reason the rewrite is available, and no
declaration in this chapter carries out the index-by-index argument. Its previous provider was also in the unmaintained
`AutodiffGeometry` namespace.

### CFT-10-003: multiplicativity of the top-degree form {#cft-10-003}

**Statement.** For square matrices $A$ and $B$ over a commutative ring,
$\det(AB)=\det A\det B$.

**Proof.** This is CFT-05-001. Read multilinearly: $\det$ is the normalized
alternating $n$-linear function of the columns, composition acts on that
function by precomposition, and the scalar by which it rescales is
multiplicative because precomposing twice rescales twice.

**Boundary and Lean provider.**
[`determinant_top_degree_multiplicative`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L33)
re-exports `CrouzeixTextbook.Part01.determinant_multiplicative` and names it as
its underlying declaration, so the two carry the same type. No new content is
claimed. The top-degree exterior reading is
`alternating_form_scaled_by_determinant` in Chapter 5, which this card does not
duplicate.

### CFT-10-004: the alternating form on a diagonal {#cft-10-004}

**Statement.** For a family of scalars $d$,
$\det(\operatorname{diag} d)=\prod_i d_i$.

**Proof.** This is CFT-05-002. Multilinearly it says the volume of the box on
the scaled axes is the product of the scalings, which is the normalization that
pins the alternating form down.

**Boundary and Lean provider.**
[`alternating_diagonal_volume`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L36)
re-exports `CrouzeixTextbook.Part01.determinant_diagonal`, named as its
underlying declaration. No new content is claimed.

### CFT-10-005: contraction is cyclic {#cft-10-005}

**Statement.** For $A$ of shape $m\times n$ and $B$ of shape $n\times m$,
$\operatorname{tr}(AB)=\operatorname{tr}(BA)$.

**Proof.** This is CFT-05-004. Read as contraction: both sides contract the same
two-index array $A_{ij}B_{ji}$ over both of its indices, and the order in which
the two contractions are performed does not change the result.

**Boundary and Lean provider.**
[`contraction_trace_cyclic`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L39)
re-exports `CrouzeixTextbook.Part01.trace_cyclic`, named as its underlying
declaration. The shapes are deliberately rectangular: $AB$ and $BA$ are square
of different sizes, and the equality is between two contractions rather than
between two matrices.

### CFT-10-006: the antisymmetry sign kernel {#cft-10-006}

**Statement.** For integers $a$ and $b$, $-(a-b)=b-a$.

**Proof.** Ring arithmetic on the integers.

**Boundary and Lean provider.** This card is deliberately small: it isolates the
one arithmetic fact behind every sign in an alternating expression, so that the
signs elsewhere are bookkeeping rather than a separate argument. It proves
nothing about wedge products themselves.
[`wedge_sign_kernel`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L42)
is discharged by `ring`.

### The tensor product, and what a reshape is not

The universal property is the definition. Every bilinear $f:M\times N\to P$
factors through a unique linear map on $M\otimes N$, and that factorization
computes on pure tensors:
$$
\widetilde f(x\otimes y)=f(x,y).
$$
Of that statement, the computation rule is what is compiled here:
[`bilinear_factors_through_tensor`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L49),
supplied by Mathlib's `TensorProduct.lift.tmul`, takes $f$ already in curried
linear form and states $\widetilde f(x\otimes y)=f(x)(y)$. Neither the existence
of the factorization nor its uniqueness is stated by that declaration or by any
other in this chapter; both hold, and both are Mathlib results
(`TensorProduct.lift` and `TensorProduct.ext`), but the chapter does not check
them. The compiled rule holds over any commutative ring and any three modules,
with no basis and no finiteness.

Reshaping is something else entirely. It is the relabeling
$\iota\times\kappa\simeq\{1,\dots,mn\}$ and nothing more, recorded as
[`reshape_is_index_relabeling`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L58).
A bijection of index sets carries no algebra with it.

The sharpest way to see that the two notions differ is that an array of the
product shape need not be a product of vectors. The identity matrix is such an
array, and there are no vectors $x,y$ with $I_{ij}=x_iy_j$: any such array has
vanishing two-by-two determinant, since $x_0y_0\,x_1y_1-x_0y_1\,x_1y_0=0$, while
$\det I=1$. That is
[`identity_is_not_a_pure_tensor`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L65),
whose proof detects rank with `Matrix.det_fin_two` and `Matrix.det_one` from the
library rather than with any Chapter 5 declaration.

Read as a statement about $\mathbb R^2\otimes\mathbb R^2$, this says most
elements of a tensor product are not pure tensors — but that reading is a gloss
in two ways the compiled statement does not cover. `TensorProduct` does not
occur in the declaration, which quantifies over $2\times2$ real arrays, so the
bridge to the tensor product runs through the standard identification of
$\mathbb R^2\otimes\mathbb R^2$ with those arrays, which this chapter does not
compile. And "most" is a generalization from one witness; the compiled content
is the single counterexample.

### Contraction, shape-checked

Summing a repeated index is contraction. Which index is summed is carried by the
types:
[`contractMiddle`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L83)
takes a three-index array and sums the middle axis, and no other reading of it
typechecks. Contracting the outer product of two matrices over their shared axis
recovers matrix multiplication exactly, which is
[`contract_middle_eq_mul`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L88);
contraction is additive in the array, which is
[`contract_middle_add`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L94);
scalar homogeneity, the other half of linearity, is not compiled here.

### Basis dependence: congruence, not similarity

Components of a bilinear form depend on the basis, and the dependence is a
*congruence*. Substituting $x\mapsto Sx$ and $y\mapsto Sy$ into
$\langle x,Gy\rangle$ and moving $S$ across the pairing with CFT-10-001 gives
$$
\langle Sx,\ G\,Sy\rangle=\langle x,\ S^{\mathsf T}GS\,y\rangle ,
$$
which is
[`bilinear_components_change_by_congruence`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L108).
This is exactly the transformation whose positivity Chapter 8 proved stable
under.

Congruence is not similarity. Operators transform by $S^{-1}AS$ and forms by
$S^{\mathsf T}GS$. These agree for *every* matrix exactly when
$S^{\mathsf T}=S^{-1}$, that is when $S$ is orthogonal; for a particular matrix
they may coincide without it, as at $G=0$. The running shear is not:
$S^{\mathsf T}S\neq I$ for $S=\begin{bmatrix}1&1\\0&1\end{bmatrix}$, recorded as
[`congruence_is_not_similarity`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L122)
with the transpose written out in
[`shear_transpose`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L115).
So "change of basis" must always say which object is being transported. A
spectrum is preserved by similarity and not by congruence; positivity is
preserved by congruence and not by similarity.

## Worked examples

Contract the outer product of
$$
A=\begin{bmatrix}1&2\\3&4\end{bmatrix},\qquad
B=\begin{bmatrix}5&6\\7&8\end{bmatrix}
$$
over the shared axis: $\sum_j A_{ij}B_{jk}$ gives
$\begin{bmatrix}19&22\\43&50\end{bmatrix}$, which is $AB$. E02 checks that
identity against the explicit product.

The cumulative family shows the congruence rule. Its general member is
$$
A_{\lambda,\alpha}=\begin{bmatrix}\lambda&\alpha\\0&\lambda\end{bmatrix},
$$
and taking $S=A_{1,\alpha}$ with $G=I$,
$$
S^{\mathsf T}GS=S^{\mathsf T}S
=\begin{bmatrix}1&0\\ \alpha&1\end{bmatrix}
 \begin{bmatrix}1&\alpha\\0&1\end{bmatrix}
=\begin{bmatrix}1&\alpha\\ \alpha&\alpha^{2}+1\end{bmatrix},
$$
the Gram matrix of Chapter 8. Its determinant is $\alpha^2+1-\alpha^2=1$, in
agreement with $\det(S^{\mathsf T}GS)=(\det S)^2\det G$, which combines
CFT-10-003 with $\det S^{\mathsf T}=\det S$ — the latter being Chapter 5's
`determinant_transpose`, not part of CFT-10-003. The
same $S$ transports an operator to $S^{-1}AS$, which for $\alpha\neq0$ is a
different matrix; that difference is the content of the boundary result above.

## ML bridge

**Mathematical object.** A tensor in this chapter is an element of a tensor
product, characterized by the universal property. A tensor in an array library
is a rectangular block of numbers with named axes. The second is a
representation of the first *after* a basis is chosen, and only for the cases
where a basis exists.

**Exact transfer.** Contraction is `einsum`, and the shape check is the whole
content of "the axes must line up": `contract_middle_eq_mul` is the statement
that contracting `ij,jk` over `j` is matrix multiplication. CFT-10-001 is the
reverse-mode rule in coordinates — the vector-Jacobian product is the transpose
action — and CFT-10-002 is why the two notations for it agree.

**Non-transfer.** A `reshape` is a relabeling of indices and carries no algebra:
it is not a tensor product, and no statement here licenses treating a flattened
array as an element of one. Broadcasting is not multilinearity, since it is not
linear in a repeated argument. Naming an axis does not make a transformation law
hold: whether an array transforms by $S^{-1}AS$ or $S^{\mathsf T}GS$ depends on
what it represents, and the array itself does not record which. Nothing here
addresses numerical contraction order, which changes cost and rounding but not
the exact result.

**Calculation.** The identity is a legitimate `(2,2)` array and not an outer
product of any two vectors, because every outer product has vanishing
two-by-two determinant and $\det I=1$. In array terms: `I` has rank two, and
`x[:,None] * y[None,:]` always has rank at most one. Any code path that assumes
a rank-two array can be factored into two vectors is wrong on this input, and
the failure is exact rather than numerical.

## Lean translation

`Matrix.mulVec` is `A *ᵥ x`, `Matrix.vecMul` is `y ᵥ* A`, and `dotProduct` is
`⬝ᵥ`. The tensor product is Mathlib's `TensorProduct`, with `x ⊗ₜ[R] y` for
pure tensors and `TensorProduct.lift` for the factorization; `finProdFinEquiv`
is the reshape bijection. `contractMiddle` is defined in this chapter because
the point being made is about which axis the types allow to be summed.

Three of the six cards are re-exports of Chapter 5 declarations and name them as
underlying declarations, so the validator checks the alias identity rather than
accepting an unexamined checkpoint. CFT-10-001 and CFT-10-002 are proved here
because their previous providers were in an unmaintained namespace.

The six exercise theorems live in
`CrouzeixTextbook.Part02.Exercises.Chapter10` in the
[chapter Lean source](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean).

## Exercises

### CFT-10-E01 -- retrieval {#exercise-cft-10-e01}

For a rectangular real matrix, state both coordinate facts of this chapter: that
the pairing moves the matrix across as its transpose, and that the covector
action is the transpose action.

**Solution.** Both are CFT-10-001 and CFT-10-002 at a concrete shape; the point
of stating them together is that the second is what makes the first readable.
[`exercise_01_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L134)
fixes a $2\times3$ matrix, so the two vector arguments have genuinely different
lengths and no square-shape assumption can hide.

### CFT-10-E02 -- calculation {#exercise-cft-10-e02}

Contract the outer product of the two matrices above over their shared axis and
identify the result.

**Solution.** The contraction gives
$\begin{bmatrix}19&22\\43&50\end{bmatrix}$.
[`exercise_02_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L139)
proves it by rewriting the contraction as the matrix product and then evaluating
entrywise, which is the checked form of "einsum `ij,jk->ik` is matmul".

### CFT-10-E03 -- written-proof {#exercise-cft-10-e03}

Prove the two structural facts about contraction: that it recovers matrix
multiplication on an outer product, and that it is additive in the array.

**Solution.** The first is the definition of the matrix product read as a sum
over the shared index. The second is the interchange of a finite sum with
addition.
[`exercise_03_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L148)
states both for arbitrary shapes, so the axis lengths are independent and the
shape check is visible in the statement rather than assumed.

### CFT-10-E04 -- written-proof {#exercise-cft-10-e04}

Derive the transformation law for the components of a bilinear form under a
change of basis.

**Solution.** Substitute $x\mapsto Sx$ and $y\mapsto Sy$ and move the first $S$
across the pairing with CFT-10-001, then associate the remaining product. The
result is congruence by $S$.
[`exercise_04_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L157)
states it for every dimension, every form and every change of basis, with no
invertibility assumption — congruence needs none, which is exactly why it
differs from similarity.

### CFT-10-E05 -- boundary {#exercise-cft-10-e05}

Show that a reshape is a bijection of index sets, and that an array of the
product shape need not be a pure tensor.

**Solution.** The reshape is the equivalence
$\mathrm{Fin}\,2\times\mathrm{Fin}\,3\simeq\mathrm{Fin}\,6$, which is bijective
by construction. For the second, any array of the form $x_iy_j$ has vanishing
two-by-two determinant, while the identity has determinant one.
[`exercise_05_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L163)
states both. Together they are the chapter's central distinction: relabeling
indices is not forming a tensor product.

### CFT-10-E06 -- lean-proof {#exercise-cft-10-e06}

State the universal property's computation rule and the sign kernel.

**Solution.** The factorization of a bilinear map through the tensor product
computes on pure tensors as $\widetilde f(x\otimes y)=f(x,y)$; the sign kernel
is integer ring arithmetic.
[`exercise_06_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter10.lean#L170)
states them together over an arbitrary commutative ring and arbitrary modules,
so the universal property appears without any finiteness or basis assumption.

## Synthesis and forward dependencies

Multilinearity is the pattern behind the determinant, the pairing and the Gram
matrix alike. The tensor product is the object that classifies bilinear maps by
a universal property; a reshape is a relabeling of indices, and the identity
matrix is the one-line proof that the two are different. Contraction sums a
repeated index, with the types carrying which one. Components of a form
transform by congruence, components of an operator by similarity, and confusing
the two is the error that Chapter 8's positivity results and Chapter 6's
spectral results would each detect.

Chapter 11 makes the derivative a linear approximation, which turns the
pullback of Chapter 4 into the reverse mode of automatic differentiation.
Chapter 12 returns to the alternating side of this chapter for forms and
integration.
