---
id: cft-chapter-09-operator-norms-and-singular-values
title: Operator norms and singular values
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-09
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 09_operator_norms_and_singular_values.md
chapter: 9
part: 2
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 9: Operator norms and singular values

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/08_positive_operators_and_gram_geometry|Chapter 8]]
Next: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/10_multilinear_maps_and_tensors|Chapter 10 — Multilinear maps and tensors]]

## Opening problem

Three chapters have now circled the same gap. Chapter 5 found two similarity
invariants blind to the shear parameter $\alpha$; Chapter 6 exhibited a matrix
whose spectrum cannot see how far it moves a vector; Chapter 7 measured one
such vector and left the general question open. This chapter closes it: the
induced norm is the exact size of the largest stretching, and for the running
family it equals $|\alpha|$ while every eigenvalue is zero.

**Motivation.** A bound of the form $\|p(A)\|\le C$ is the shape of every
result in Part VI, so the book has to fix which norm is meant and prove the
handful of properties every later estimate silently uses: that the norm bounds
stretching, that it is the least such bound, that it is submultiplicative, and
that unitary factors do not change it. It also has to state plainly that the
spectrum does not control it, because the whole point of the numerical range
is to supply a substitute that does.

## Conceptual model

For a matrix $A$ acting on Euclidean space, the *induced* or *operator* norm is
$$
\|A\|=\sup_{\|x\|\le 1}\|Ax\| ,
$$
characterized by two properties that this chapter proves separately:
$\|Ax\|\le\|A\|\,\|x\|$ for every $x$, and $\|A\|\le C$ whenever $C\ge0$ bounds
every $\|Ax\|/\|x\|$. Those two together say the norm is the least bound, and
they are the only interface the later chapters need.

The singular values enter through the Gram matrix of Chapter 8. Because
$A^{\mathsf H}A$ is positive semidefinite, its eigenvalues are nonnegative; the
singular values $\sigma_i(A)$ are their square roots. The link to the norm is
the C\*-identity
$$
\|A^{\mathsf H}A\|=\|A\|^{2},
$$
which says that $\|A\|$ is the largest singular value. Rank deficiency is the
statement that the smallest singular value is zero, equivalently that $A$ kills
a nonzero vector.

Two measurements must be kept apart. The *spectral radius* is the largest
modulus of an eigenvalue; the norm is the largest singular value. They agree
for normal matrices and can differ arbitrarily otherwise. The chapter proves
the extreme case: a nonzero nilpotent matrix has spectral radius $0$ and norm
$|\alpha|$.

**Historical context.** Naming $\|A\|$ the "induced" norm reflects that it is
determined by the vector norm rather than chosen independently; several other
matrix norms are in common use and none of them is meant here. We fix the
convention and prove what we use; no priority claim is intended.

## Formal development

### CFT-09-001: the norm is the induced operator norm {#cft-09-001}

**Statement.** For a complex square matrix $A$, $\|A\|=\|A_{\mathrm{op}}\|$,
where the right side is the operator norm of the induced map on Euclidean
space.

**Proof.** The matrix norm carried by this development is defined to be that
operator norm, so the two sides denote the same real number.

**Boundary and Lean provider.** This card re-exports the same maintained
theorem as
[[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/07_inner_product_spaces#cft-07-005|CFT-07-005]],
`CrouzeixConjecture.matrix_norm_eq_euclidean_operator_norm`, whose checked
proof is `rfl`. The two cards therefore have identical type fingerprints; this
one is not new mathematics. It is indexed again here because Chapter 9 is where
the identity is *used* — every estimate below is stated for the matrix norm and
proved through the operator norm — and a reader entering at this chapter needs
the convention in front of them.
[`induced_matrix_norm_identity`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L14)
is the declaration.

### The two halves of "least upper bound"

The interface the rest of the book uses is a pair of inequalities, and they are
proved separately because later chapters use them in opposite directions.

Upper control: $\|Ax\|\le\|A\|\,\|x\|$ for every $x$. This is the defining
bound of an operator norm, transported across CFT-09-001; it is
[`matrix_norm_mulVec_bound`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L42),
whose checked proof rewrites by CFT-09-001 and applies the library's
`le_opNorm`.

Minimality: if $C\ge0$ and $\|Ax\|\le C\|x\|$ for every $x$, then $\|A\|\le C$.
This is
[`matrix_norm_le_of_bound`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L48),
discharged by `opNorm_le_bound`. Every "the norm is at most two" statement in
Part VI is an instance of this direction, including CFT-09-005 below.

Two closure properties follow from the same interface.
[`matrix_norm_mul_le`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L55)
is submultiplicativity $\|AB\|\le\|A\|\|B\|$, which is what makes a bound on
$\|A\|$ say anything about $\|A^k\|$; it comes from the normed-algebra structure.
[`matrix_norm_conjTranspose_mul_self`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L61)
is the C\*-identity $\|A^{\mathsf H}A\|=\|A\|\|A\|$, the bridge to the singular
values, supplied by `CStarRing.norm_star_mul_self`.

### CFT-09-002: the polar factor is unitary {#cft-09-002}

**Statement.** For an invertible $S$ with Gram matrix $G=S^{\mathsf H}S$ and
positive square root $G^{1/2}$, the polar factor $U=SG^{-1/2}$ is unitary.

**Proof.** Unitarity is $U^{\mathsf H}U=I$. Expanding and using that
$G^{-1/2}$ is self-adjoint,
$$
U^{\mathsf H}U=G^{-1/2}S^{\mathsf H}SG^{-1/2}=G^{-1/2}GG^{-1/2}
=G^{-1/2}G^{1/2}G^{1/2}G^{-1/2}=I ,
$$
where the middle step replaces $G$ by $G^{1/2}G^{1/2}$ and the last uses the
two-sided inverse property. Every ingredient is a field of the square-root data
supplied by CFT-08-006, so no new hypothesis is introduced.

**Boundary and Lean provider.** Invertibility of $S$ is needed only to obtain
that data.
[`polar_factor_is_unitary`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L17)
re-exports `CrouzeixConjecture.completionPolarUnitary_mem_unitaryGroup`, whose
checked proof runs exactly the displayed calculation: it rewrites the membership
criterion, distributes the conjugate transpose using the self-adjointness field,
regroups with `noncomm_ring`, substitutes the squaring field, and cancels with
the two inverse fields. The chapter does not claim uniqueness of the polar
decomposition, only that this construction yields a unitary factor.

### CFT-09-003: the polar similarity preserves the norm {#cft-09-003}

**Statement.** For invertible $S$ and any family $\lambda$ of scalars,
$$
\bigl\|S\operatorname{diag}(\lambda)S^{-1}\bigr\|
=\bigl\|G^{1/2}\operatorname{diag}(\lambda)G^{-1/2}\bigr\| .
$$

**Proof.** The two matrices are unitarily conjugate. Substituting the polar
factorization $S=UG^{1/2}$ turns the left-hand matrix into
$U\bigl(G^{1/2}\operatorname{diag}(\lambda)G^{-1/2}\bigr)U^{\mathsf H}$, and $U$
is unitary by CFT-09-002. Multiplying by a unitary on either side leaves the
induced norm unchanged, so both factors can be discarded in turn.

**Boundary and Lean provider.** The statement is an equality of norms, not of
matrices: the two matrices are genuinely different unless $U=I$.
[`polar_similarity_norm_transfer`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L20)
re-exports
`CrouzeixConjecture.completionDiagonalizableMatrix_norm_eq_completionSimilarity_norm`,
whose checked proof substitutes the unitary conjugation and then strips the two
unitary factors with `CStarRing.norm_mem_unitary_mul` and
`CStarRing.norm_mul_mem_unitary`. Those are exactly the two invariance
statements recorded here as
[`matrix_norm_unitary_invariant`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L66).
This is the balancing step Part VI relies on: it replaces an arbitrary
invertible conjugation, whose condition number is uncontrolled, by a positive
one, at no cost in norm.

### CFT-09-004: the same transfer on operator norms {#cft-09-004}

**Statement.** The equality of CFT-09-003 holds verbatim with both sides read
as induced operator norms of the corresponding Euclidean maps.

**Proof.** Rewrite both sides by CFT-09-001 and apply CFT-09-003.

**Boundary and Lean provider.**
[`polar_operator_norm_transfer`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L24)
re-exports
`CrouzeixConjecture.completionDiagonalizableMatrix_euclideanOperator_norm_eq`,
whose checked proof is literally those two rewrites followed by CFT-09-003. The
card exists because the downstream argument is stated on operators while the
algebra is done on matrices, and the book should not leave the reader to assume
the translation is free.

### CFT-09-005: a quadratic bound gives norm at most two {#cft-09-005}

**Statement.** If $4I-C^{\mathsf H}C$ is positive semidefinite then
$\|C\|\le2$.

**Proof.** Positivity of $4I-C^{\mathsf H}C$ says
$\langle x,C^{\mathsf H}Cx\rangle\le4\langle x,x\rangle$, that is
$\|Cx\|^{2}\le4\|x\|^{2}$, hence $\|Cx\|\le2\|x\|$ for every $x$. By
minimality, $\|C\|\le2$.

**Boundary and Lean provider.** This is the shape every terminal bound in the
book takes: a positivity certificate converted into a norm bound. The constant
is not special to the argument; $4$ appears because the target constant is $2$.
[`quadratic_bound_implies_norm_two`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L28)
re-exports
`CrouzeixConjecture.matrix_norm_le_two_of_four_sub_conjTranspose_mul_self_posSemidef`,
whose checked proof rewrites by CFT-09-001 and delegates to the operator-level
statement; the positivity-to-bound step itself lives in the maintained
positivity development, not in this card.

### CFT-09-006: the triangle inequality {#cft-09-006}

**Statement.** In any seminormed additive group, $\|x+y\|\le\|x\|+\|y\|$.

**Proof.** Subadditivity is part of the seminorm interface; the library
packages it as `norm_add_le`.

**Boundary and Lean provider.** Like CFT-07-006 this is a weakest-hypothesis
card, stated because every perturbation estimate in Parts V and VI splits a
matrix into a main term and a correction and adds the two bounds.
[`norm_triangle_kernel`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L32)
is proved here rather than aliased, because a bare alias of a non-maintained
library lemma is classified as a direct alias and cannot carry a local-proof
claim.

### Rank deficiency and the spectrum–norm gap

Two boundary facts complete the chapter.

A singular matrix kills a nonzero vector:
[`singular_has_null_vector`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L72)
turns $\det A=0$ into an explicit $x\neq0$ with $Ax=0$, via the library
equivalence `Matrix.exists_mulVec_eq_zero_iff`. In singular-value language the
smallest singular value is then zero, and $A$ has no bounded inverse.

The spectrum does not control the norm. Take the running family at eigenvalue
zero,
$$
N_\alpha=\begin{bmatrix}0&\alpha\\0&0\end{bmatrix}.
$$
It is nilpotent — $N_\alpha^2=0$, checked as
[`shear_nilpotent`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L82)
— so every eigenvalue is zero and the spectral radius is $0$; its determinant
vanishes too, checked as
[`shear_det_zero`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L90).
Yet its norm is exactly $|\alpha|$. The computation is the C\*-identity applied
to a Gram matrix that happens to be diagonal:
$$
N_\alpha^{\mathsf H}N_\alpha=\operatorname{diag}\bigl(0,\ \overline{\alpha}\alpha\bigr),
$$
checked as
[`shear_conjTranspose_mul_self`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L95).
The induced norm of a diagonal matrix is the largest modulus of its entries,
here $|\alpha|^2$, recorded as
[`shear_diagonal_family_norm`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L104);
so $\|N_\alpha\|^2=|\alpha|^2$ and, both sides being nonnegative,
$\|N_\alpha\|=|\alpha|$. That is
[`shear_norm_eq`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L119).

The gap between spectral radius $0$ and norm $|\alpha|$ is therefore unbounded.
This single example is why the rest of the book cannot bound $\|p(A)\|$ by the
values of $p$ on the spectrum, and why Chapter 20 introduces the numerical
range instead.

## Worked examples

For the general running family
$$
A_{\lambda,\alpha}=\begin{bmatrix}\lambda&\alpha\\0&\lambda\end{bmatrix}
=\lambda I+N_\alpha ,
$$
the triangle inequality and the computation above give
$\|A_{\lambda,\alpha}\|\le|\lambda|+|\alpha|$, while the spectral radius is
$|\lambda|$ for every $\alpha$. Chapter 5 computed $\det=\lambda^2$ and
$\operatorname{tr}=2\lambda$, both blind to $\alpha$; the norm is the first
quantity in the book that sees it.

Submultiplicativity is what converts a single bound into a power bound: from
$\|A\|\le c$ it follows that $\|A^k\|\le c^k$ by induction. For $A_{1,\alpha}$
this gives $\|A_{1,\alpha}^k\|\le(1+|\alpha|)^k$, which is exponential; the
true growth is linear, since $A_{1,\alpha}^k=I+k\alpha N_1$. The bound is
correct and badly lossy, and closing that gap for a class of functions rather
than a single power is the subject of Part VI.

Rank deficiency in the same family: at $\lambda=0$ the matrix is $N_\alpha$,
whose determinant vanishes, so it annihilates $(1,0)$ and its smallest singular
value is $0$ while its largest is $|\alpha|$.

## ML bridge

**Mathematical object.** The induced norm is the worst-case gain of a linear
layer, and the largest singular value is the Lipschitz constant of that layer
with respect to the Euclidean metric. Spectral normalization divides a weight
matrix by exactly this quantity.

**Exact transfer.** Submultiplicativity is why the product of per-layer
Lipschitz constants bounds the Lipschitz constant of a composition of linear
layers. Unitary invariance is why an orthogonal reparametrization of a layer
changes nothing measurable about its gain. The C\*-identity is the statement
that the quantity computed by a power iteration on $A^{\mathsf H}A$ is the
squared norm.

**Non-transfer.** The spectral radius is not the Lipschitz constant, and the
compiled example above is the reason: a layer whose eigenvalues all vanish can
still amplify by an arbitrary factor. A composition bound built from spectral
radii is therefore not a bound at all. Nothing here covers nonlinear layers,
where the per-layer Lipschitz product is usually a large overestimate, nor does
it say that a numerically computed largest singular value is accurate for a
near-defective matrix — the running family is precisely the case where it is
not.

**Calculation.** For $A_{1,\alpha}$ with $\alpha=10$ the spectral radius is $1$,
so a spectral-radius argument would predict $\|A^k\|\le1$ for all $k$. The truth
is $A^k=I+10kN_1$, whose norm grows without bound. The submultiplicative bound
$11^k$ is correct but exponentially lossy. Both failures are visible at $k=2$:
predicted $1$, actual $\|I+20N_1\|>20$, bound $121$.

## Lean translation

`euclideanOperator A` is the continuous linear map induced by `A`, so
`ContinuousLinearMap.le_opNorm` and `opNorm_le_bound` supply the two halves of
the least-upper-bound interface once CFT-09-001 transports them.
`CStarRing.norm_star_mul_self` is the C\*-identity, and
`CStarRing.norm_mem_unitary_mul` and `norm_mul_mem_unitary` are the two unitary
invariances. `Matrix.l2_opNorm_diagonal` computes the induced norm of a diagonal
matrix as the supremum norm of its entry family, which is what makes the
`shear_norm_eq` computation short.

Five of this chapter's six cards are exact re-exports of maintained
`CrouzeixConjecture` theorems and name their underlying declaration in the
contract; CFT-09-006 is proved here. CFT-09-001 shares its provider with
CFT-07-005, as its boundary paragraph states.

The six exercise theorems live in
`CrouzeixTextbook.Part02.Exercises.Chapter09` in the
[chapter Lean source](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean).

## Exercises

### CFT-09-E01 -- retrieval {#exercise-cft-09-e01}

State the convention fixing which norm this book uses, and the defining bound
that convention buys.

**Solution.** The matrix norm is the induced operator norm of the Euclidean
map, and it dominates every stretching: $\|Ax\|\le\|A\|\|x\|$.
[`exercise_01_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L134)
states both for every finite index type, every matrix and every vector, so the
convention and the inequality it licenses appear in one checked statement.

### CFT-09-E02 -- calculation {#exercise-cft-09-e02}

Record the two multiplicative facts every later estimate uses:
submultiplicativity and the C\*-identity.

**Solution.** $\|AB\|\le\|A\|\|B\|$ comes from the normed-algebra structure;
$\|A^{\mathsf H}A\|=\|A\|\|A\|$ is the C\*-identity, and it is what identifies
$\|A\|$ as the largest singular value.
[`exercise_02_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L140)
states both. Neither is proved here from scratch; both are named library facts
about the C\*-algebra of matrices, and the exercise records which ones.

### CFT-09-E03 -- written-proof {#exercise-cft-09-e03}

For an invertible $S$, show that the polar factor is unitary and that the polar
similarity leaves the norm unchanged.

**Solution.** Unitarity is CFT-09-002, proved by expanding $U^{\mathsf H}U$ and
collapsing it with the square-root data. The norm equality is CFT-09-003,
proved by writing the left matrix as a unitary conjugate of the right and
stripping the unitary factors.
[`exercise_03_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L146)
states both for every scalar family, which is the form the Part VI balancing
argument consumes.

### CFT-09-E04 -- written-proof {#exercise-cft-09-e04}

Prove that unitary factors leave the norm unchanged on either side, and deduce
that unitary conjugation does too.

**Solution.** The two one-sided invariances are library facts about the
C\*-algebra. For the conjugation, apply the right-hand invariance with
$U^{\mathsf H}$ — which is unitary because the unitary group is closed under
the star — and then the left-hand invariance with $U$.
[`exercise_04_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L156)
states all three conjuncts and derives the third from the first two exactly as
described, rather than citing CFT-09-003.

### CFT-09-E05 -- boundary {#exercise-cft-09-e05}

Exhibit a matrix whose spectral radius is zero and whose norm is as large as
desired.

**Solution.** Take $N_\alpha$. It is nilpotent and singular, so every
eigenvalue is zero; and its norm is exactly $|\alpha|$, by the C\*-identity
applied to the diagonal Gram matrix $\operatorname{diag}(0,|\alpha|^2)$.
[`exercise_05_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L164)
states the vanishing determinant, the nilpotency and the exact norm as three
conjuncts. The exact value matters: a lower bound would show the spectrum is
not an upper bound for the norm, but the equality shows the gap is exactly the
parameter the spectrum discards.

### CFT-09-E06 -- lean-proof {#exercise-cft-09-e06}

Instantiate the two facts a terminal bound needs: a positivity certificate
converted into a norm bound, and the triangle inequality used to add a main
term to a correction.

**Solution.** The first is CFT-09-005 applied to the supplied certificate; the
second is CFT-09-006.
[`exercise_06_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter09.lean#L172)
states them together, which is the exact pair of ingredients the Part VI
perturbation arguments combine.

## Synthesis and forward dependencies

The induced norm is the least bound on stretching, it is submultiplicative, it
is unchanged by unitary factors, and by the C\*-identity it is the largest
singular value. Those four facts are the entire interface the rest of the book
uses. Against them the chapter sets one compiled counterexample: a nilpotent
matrix with spectral radius zero and norm $|\alpha|$, which rules out any bound
on $\|p(A)\|$ in terms of $p$ on the spectrum.

Chapter 10 leaves the metric aside and returns to algebra with multilinear maps
and tensors. The polar-similarity transfer proved here reappears in Part VI as
the balancing step, and the spectrum–norm gap is the gap that Crouzeix's
theorem closes by replacing the spectrum with the numerical range.
