---
id: cft-chapter-08-positive-operators-and-gram-geometry
title: Positive operators and Gram geometry
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-09
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 08_positive_operators_and_gram_geometry.md
chapter: 8
part: 2
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 8: Positive operators and Gram geometry

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/07_inner_product_spaces|Chapter 7]]
Next: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/09_operator_norms_and_singular_values|Chapter 9 — Operator norms and singular values]]

## Opening problem

Chapter 7's ML bridge ended on an observation left unexplained: the Gram matrix
of $(1,0)$ and $(3,4)$ had determinant $16$, exactly the squared length of the
residual after projection. That was not luck. A Gram matrix is built from a
pairing, so it inherits the pairing's positivity, and everything measured by
that positivity — squared volumes, feasible directions, the existence of a
square root — follows from one inequality.

**Motivation.** Positivity is the property that survives the change of
coordinates a metric permits. It is also the property most often asserted
loosely: three different conditions travel under the word *positive*, and the
later chapters break if they are conflated. This chapter pins down which one is
meant, proves the closure properties actually used downstream, and constructs
the positive square root that Part VI's completion argument consumes.

## Conceptual model

Fix the Hermitian pairing of Chapter 7. A square matrix $A$ is *positive
semidefinite* when it is self-adjoint and $\langle x,Ax\rangle\ge0$ for every
$x$; it is *positive definite* when the inequality is strict for every $x\neq0$.
Over $\mathbb R$ the self-adjointness condition is symmetry.

The generating example is the Gram matrix. Given $S$, set $G=S^{\mathsf H}S$.
Its quadratic form is
$$
\langle x,Gx\rangle=\langle Sx,Sx\rangle=\|Sx\|^2\ge0 ,
$$
so *every* Gram matrix is positive semidefinite, with no hypothesis on $S$ at
all. It is positive definite exactly when $Sx=0$ forces $x=0$, that is when $S$
is invertible. Positivity of the Gram matrix is therefore not an extra
assumption; it is a restatement of the fact that a squared length is
nonnegative.

Three conditions must be kept apart, and this chapter separates them with
compiled examples rather than warnings.

1. *Positive semidefinite*: the quadratic form is nonnegative. A singular Gram
   matrix has a nonzero null vector and is semidefinite but not definite.
2. *Positive definite*: strict on nonzero vectors. This is what makes the
   square root invertible.
3. *Entrywise positive*: every entry is a positive number. This is unrelated to
   the other two. A matrix with all entries positive can have a strictly
   negative quadratic form.

**Historical context.** The word "positive" is overloaded across matrix
analysis, order theory and probability, and different subfields fix different
defaults. We state the definition we use rather than relying on the reader's
background; no claim is made about which usage came first.

## Formal development

### CFT-08-001: instantiating a positivity hypothesis {#cft-08-001}

**Statement.** Let $A$ be a symmetric real matrix and suppose
$\langle y,Ay\rangle>0$ for every nonzero $y$. Then for any explicitly supplied
nonzero $x$, $\langle x,Ax\rangle>0$.

**Proof.** The hypothesis is universally quantified over nonzero vectors, and
$x$ is one; instantiate it at $x$. There is no further content, and in
particular the symmetry hypothesis is not consumed anywhere.

**Boundary and Lean provider.** This card records the shape of the
positive-definiteness assumption used downstream, not a new inequality: its
conclusion is its own hypothesis instantiated. Saying so is the honest reading.
[`positive_definite_quadratic_positive`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L19)
is proved here, and its checked proof is exactly that single instantiation. The
symmetry argument is carried as an explicitly unused binder so that the
statement matches the interface the later chapters expect. The real work of the
chapter is in the Gram cards that follow, where positivity is *derived* rather
than assumed.

### CFT-08-002: every Gram matrix is positive semidefinite {#cft-08-002}

**Statement.** For every complex square matrix $S$, the Gram matrix
$G=S^{\mathsf H}S$ is positive semidefinite.

**Proof.** Self-adjointness is immediate:
$(S^{\mathsf H}S)^{\mathsf H}=S^{\mathsf H}S^{{\mathsf H}{\mathsf H}}=S^{\mathsf H}S$.
For the inequality, move one factor across the pairing using CFT-07-004: the
adjoint of the operator induced by $S$ is induced by $S^{\mathsf H}$, so
$$
\langle x,S^{\mathsf H}Sx\rangle=\langle Sx,Sx\rangle=\|Sx\|^{2}\ge0 .
$$
No hypothesis on $S$ is needed, and no invertibility is assumed.

**Boundary and Lean provider.**
[`gram_matrix_positive`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L67)
re-exports `CrouzeixConjecture.completionGramMatrix_posSemidef`, whose checked
proof is the single library result `Matrix.posSemidef_conjTranspose_mul_self`
— that is, the whole displayed argument is packaged once in the library and the
provider applies it. The real-scalar version of the same computation is proved
here as
[`real_gram_quadratic_form`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L28)
and
[`real_gram_nonneg`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L34),
where the transpose replaces the conjugate transpose and the two steps are
visible separately.

### CFT-08-003: an invertible generator has an invertible Gram matrix {#cft-08-003}

**Statement.** If $S$ is a unit of the matrix ring, so is $G=S^{\mathsf H}S$.

**Proof.** The conjugate transpose is a ring anti-automorphism that carries
units to units, so $S^{\mathsf H}$ is a unit; and a product of two units is a
unit. That is the entire argument — it does not compute $G^{-1}$, and it does
not go through the determinant.

**Boundary and Lean provider.**
[`invertible_gram_matrix_invertible`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L70)
re-exports `CrouzeixConjecture.completionGramMatrix_isUnit`, whose checked proof
unfolds the Gram matrix and applies `IsUnit.star` followed by `IsUnit.mul`.
Chapter 5's determinant criterion would give an alternative route through
$\det G=\overline{\det S}\det S$; the maintained proof does not take it, and
the prose above follows the proof that exists.

### CFT-08-004: invertible Gram matrices are positive definite {#cft-08-004}

**Statement.** If $S$ is a unit, then $G=S^{\mathsf H}S$ is positive definite.

**Proof.** $G$ is positive semidefinite by CFT-08-002 and invertible by
CFT-08-003. For a positive semidefinite matrix these two facts are equivalent
to definiteness: semidefiniteness gives $\langle x,Gx\rangle\ge0$ always, and
if the value were $0$ for some $x\neq0$ then $\|Sx\|=0$, so $Sx=0$ and $G$ would
annihilate a nonzero vector, contradicting invertibility.

**Boundary and Lean provider.**
[`invertible_gram_matrix_positive_definite`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L73)
re-exports `CrouzeixConjecture.completionGramMatrix_posDef`. Its checked proof
is the composition the paragraph describes: it feeds CFT-08-002's conclusion
and CFT-08-003's conclusion into the library equivalence
`PosSemidef.posDef_iff_isUnit`. The equivalence is what carries the null-vector
argument; the card assembles the two inputs.

### CFT-08-005: congruence preserves positivity {#cft-08-005}

**Statement.** If $A$ is positive semidefinite then so is
$S^{\mathsf H}AS$, for every $S$.

**Proof.** Move $S$ across the pairing as in CFT-08-002:
$\langle x,S^{\mathsf H}ASx\rangle=\langle Sx,A\,Sx\rangle\ge0$, because $A$ is
nonnegative at the vector $Sx$. Self-adjointness of the conjugated matrix
follows from that of $A$ by the same anti-automorphism used in CFT-08-003.

**Boundary and Lean provider.** Note what is *not* claimed: congruence
preserves semidefiniteness for every $S$, but preserves definiteness only when
$S$ is invertible, since a singular $S$ sends some nonzero $x$ to $0$.
Congruence is also not similarity: $S^{\mathsf H}AS$ and $S^{-1}AS$ agree for
*every* $A$ exactly when $S^{\mathsf H}=S^{-1}$, that is when $S$ is unitary.
For a particular $A$ they can of course coincide without that — at $A=0$ both
are $0$ — so what fails in general is the preservation of the spectrum, which
congruence does not give even though it preserves positivity.
[`positivity_preserved_by_congruence`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L76)
re-exports `CrouzeixConjecture.posSemidef_congruence`, whose checked proof is
the single library application `PosSemidef.conjTranspose_mul_mul_same`.

### CFT-08-006: the positive square root {#cft-08-006}

**Statement.** For an invertible $S$, write $G=S^{\mathsf H}S$. Then the
specific matrices $H=G^{1/2}$ and $H^{-1}$, produced by the continuous
functional calculus, satisfy $H^{\mathsf H}=H$, $(H^{-1})^{\mathsf H}=H^{-1}$,
$HH=G$, and $HH^{-1}=H^{-1}H=I$. The Lean statement is this stronger, named
form rather than a bare existential: it names `completionPositiveSquareRoot S`
and its inverse and asserts the five properties of them.

**Proof.** $G$ is positive definite by CFT-08-004, so it is nonnegative in the
matrix order and invertible. Continuous functional calculus applied to the
square-root function on the nonnegative reals produces $H=G^{1/2}$: it is
nonnegative, hence self-adjoint, and squares to $G$ by the functional identity
$(\sqrt{t})^2=t$ on the spectrum. Invertibility of $G$ makes $H$ invertible,
because the square root of a unit is a unit, and the inverse of a self-adjoint
matrix is self-adjoint.

**Boundary and Lean provider.** All five conditions are *conclusions*; none
remains an assumption, which is what makes this card usable as an input to the
Part VI completion argument.
[`positive_square_root_data`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L79)
re-exports `CrouzeixConjecture.completionSquareRootData_of_isUnit`, whose
checked proof discharges the five fields in the order above, using
`CFC.sqrt` for the construction, `CFC.isUnit_sqrt_iff` for invertibility, and
`Matrix.nonneg_iff_posSemidef` to recover self-adjointness from nonnegativity.
The three conclusions this book uses most are also exposed separately as
[`completion_square_root_squares`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L89),
[`completion_square_root_selfAdjoint`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L96)
and
[`completion_square_root_inverse`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L101).
Uniqueness of the nonnegative square root is true and is not claimed here; the
card supplies the named square root with the listed properties. The continuous
functional calculus it rests on is library machinery, used here ahead of the
functional-calculus development in Part III.

### Three meanings of "positive", separated

The distinctions promised in the conceptual model are compiled, not asserted.

Take $S=\begin{bmatrix}1&1\\0&0\end{bmatrix}$ and $x=(1,-1)$. Then $Sx=0$, so
the real Gram matrix $S^{\mathsf T}S$ satisfies $\langle x,Gx\rangle=0$ at a
nonzero $x$: semidefinite, not definite. This is
[`real_gram_semidefinite_not_definite`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L41),
and it is exactly the failure mode CFT-08-004 excludes by assuming $S$
invertible.

Take $A=\begin{bmatrix}1&2\\2&1\end{bmatrix}$, all of whose entries are
positive. At the same $x=(1,-1)$ the quadratic form is
$1-2-2+1=-2<0$. Entrywise positivity therefore does not imply semidefiniteness
in any direction; this is
[`entrywise_positive_not_semidefinite`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L53).
The converse fails too: the identity is positive definite and has zero entries
off the diagonal.

## Worked examples

Return to Chapter 7's pair $v_1=(1,0)$, $v_2=(3,4)$. Writing them as the
columns of $S$, the Gram matrix is
$$
G=S^{\mathsf T}S=\begin{bmatrix}1&3\\3&25\end{bmatrix},
\qquad \det G=25-9=16 .
$$
The projection computation of Chapter 7 gave residual $(0,4)$ with squared
length $16$. The agreement is the two-dimensional case of the general fact that
$\det G$ is the squared volume of the parallelepiped on the columns: base
$\|v_1\|^2=1$ times squared height $16$. Since $S$ is invertible here, CFT-08-004
predicts $G$ is positive definite, and indeed its quadratic form is
$a^2+6ab+25b^2=(a+3b)^2+16b^2$.

The cumulative family gives the degenerate comparison. For
$$
A_{\lambda,\alpha}=\begin{bmatrix}\lambda&\alpha\\0&\lambda\end{bmatrix},
\qquad
A_{\lambda,\alpha}^{\mathsf T}A_{\lambda,\alpha}
=\begin{bmatrix}\lambda^{2}&\lambda\alpha\\\lambda\alpha&\alpha^{2}+\lambda^{2}\end{bmatrix},
$$
whose determinant is $\lambda^{2}(\alpha^{2}+\lambda^{2})-\lambda^{2}\alpha^{2}
=\lambda^{4}$, matching $(\det A_{\lambda,\alpha})^2$ as it must. At
$\lambda=0$ the family becomes singular and its Gram matrix is semidefinite but
not definite — the boundary case above, reached from the running example.

## ML bridge

**Mathematical object.** A kernel matrix is a Gram matrix of feature vectors,
whether or not the feature map is written down. A covariance matrix is a Gram
matrix of centred data divided by a count. Both are positive semidefinite for
the same reason: their quadratic form is a squared length.

**Exact transfer.** CFT-08-002 is why a kernel matrix is always positive
semidefinite when it genuinely comes from an inner product of features, with no
extra assumption. CFT-08-004 is the condition under which it is invertible and
a ridgeless solve is well posed. CFT-08-005 is why whitening, reparametrization
and any other congruence $S^{\mathsf H}AS$ cannot destroy positivity, and
CFT-08-006 is what makes the whitening transform $G^{-1/2}$ exist in the first
place.

**Non-transfer.** These are statements about exact matrices. A kernel matrix
assembled in floating point is routinely *not* positive semidefinite: rounding
produces small negative eigenvalues, and the usual repair of clipping them is a
numerical decision this chapter does not license. Nothing here says a learned
similarity is a kernel, that a positive semidefinite matrix is well conditioned,
or that $G^{-1/2}$ can be computed stably when $G$ is near singular. The
distinction the chapter compiles — semidefinite versus definite — is exactly the
distinction that separates a solvable system from an ill-posed one, and
invertibility in exact arithmetic says nothing about the conditioning.

**Calculation.** For the running Gram matrix
$G=\begin{bmatrix}1&3\\3&25\end{bmatrix}$ the eigenvalues are
$13\pm\sqrt{153}$, both positive, and the smaller is about $0.63$ while the
larger is about $25.4$. So $G$ is positive definite with condition number about
$40$: exact positivity and numerical comfort are different questions. Here the
$40$ comes mostly from the lengths, not the angle. The two vectors have norms
$1$ and $5$ and make an angle of about $53^\circ$; rescaling them to equal
length, which leaves the angle untouched, gives
$\begin{bmatrix}1&0.6\\0.6&1\end{bmatrix}$ with eigenvalues $1.6$ and $0.4$
and condition number $4$. A disparity in scale inflates the conditioning of a
Gram matrix just as near-parallelism does, and this example is dominated by the
former.

## Lean translation

`Matrix.PosSemidef` and `Matrix.PosDef` are the two positivity predicates;
`PosSemidef` bundles self-adjointness with the nonnegativity of the quadratic
form. The Gram matrix of this development is
`CrouzeixConjecture.completionGramMatrix S`, definitionally `Sᴴ * S`, and its
positive square root is `completionPositiveSquareRoot S`, built by
`CFC.sqrt`. The five conclusions of CFT-08-006 are the fields of the
`CompletionSquareRootData` structure.

Five of this chapter's six cards are exact re-exports of maintained
`CrouzeixConjecture` theorems and name their underlying declaration in the
contract. CFT-08-001 is proved here, because its previous provider lived in a
namespace the receipt exporter does not treat as maintained and therefore could
not be checked as an alias target.

The six exercise theorems live in
`CrouzeixTextbook.Part02.Exercises.Chapter08` in the
[chapter Lean source](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean).

## Exercises

### CFT-08-E01 -- retrieval {#exercise-cft-08-e01}

Show that the identity matrix satisfies the positive-definiteness interface of
CFT-08-001 on $\mathbb R^2$: its quadratic form is strictly positive at every
nonzero vector.

**Solution.** For the identity the quadratic form is $\langle x,x\rangle$. It
is nonnegative by the pairing axiom of Chapter 7 and vanishes only at $x=0$ by
positive definiteness, so at a nonzero $x$ it is nonzero and nonnegative, hence
strictly positive.
[`exercise_01_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L112)
discharges the universally quantified hypothesis of CFT-08-001 by exactly that
argument and then instantiates the card.

### CFT-08-E02 -- calculation {#exercise-cft-08-e02}

For a real $2\times2$ matrix $S$ and any $x$, show that the Gram quadratic form
is the squared length of $Sx$, and deduce nonnegativity.

**Solution.** Moving one factor across the real pairing gives
$\langle x,S^{\mathsf T}Sx\rangle=\langle Sx,Sx\rangle$, and a self-pairing is
nonnegative.
[`exercise_02_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L122)
states both conjuncts, and the first is proved by the transpose-and-associate
rewriting rather than by citing the complex card, so the real case stands on
its own.

### CFT-08-E03 -- written-proof {#exercise-cft-08-e03}

For an invertible $S$, assemble the three positivity conclusions about
$G=S^{\mathsf H}S$: semidefinite, invertible, definite.

**Solution.** Semidefiniteness is CFT-08-002 and needs no hypothesis;
invertibility is CFT-08-003 and consumes the hypothesis on $S$; definiteness is
CFT-08-004, which combines the two.
[`exercise_03_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L128)
states all three together so the reader can see which of them uses the
invertibility assumption and which does not.

### CFT-08-E04 -- written-proof {#exercise-cft-08-e04}

Show that positivity survives a congruence and then survives a second one, so
that congruence by a composite is no stronger than congruence twice.

**Solution.** Apply CFT-08-005 to $S$, then apply the same card once more with
the matrix $ST$. The checked solution states exactly these two conclusions,
$(S^{\mathsf H}AS)$ and $((ST)^{\mathsf H}A(ST))$ semidefinite, each by one
application. It does *not* state the iterated form
$T^{\mathsf H}(S^{\mathsf H}AS)T$, and no `conjTranspose_mul` step appears in
it, so the identification of the composite reading with the twice-iterated one
is left to the reader rather than checked.
[`exercise_04_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L136)
states both conclusions for arbitrary $S$ and $T$ with no invertibility
assumption, which is the point: definiteness would need one and
semidefiniteness does not.

### CFT-08-E05 -- boundary {#exercise-cft-08-e05}

Exhibit a Gram matrix that is semidefinite but not definite, and a matrix with
all entries positive whose quadratic form takes a negative value.

**Solution.** For $S=\begin{bmatrix}1&1\\0&0\end{bmatrix}$ and $x=(1,-1)$,
$Sx=0$, so the Gram quadratic form vanishes at a nonzero vector. For
$A=\begin{bmatrix}1&2\\2&1\end{bmatrix}$ and the same $x$, the form is $-2$.
[`exercise_05_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L143)
states both. Together they separate the three meanings of "positive" listed in
the conceptual model, and they are the reason this chapter never uses the word
without qualification.

### CFT-08-E06 -- lean-proof {#exercise-cft-08-e06}

For an invertible $S$, extract from the square-root data the three facts the
later completion argument uses: that $H$ squares to $G$, that $H$ is
self-adjoint, and that the supplied inverse works on one side.

**Solution.** All three are fields of the structure produced by CFT-08-006;
extracting them is projection, not further proof.
[`exercise_06_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter08.lean#L152)
states the three together. Reading it beside the card shows that nothing in
the square-root data remains hypothetical: an invertible generator is the only
input.

## Synthesis and forward dependencies

Positivity is a consequence, not an assumption, whenever a matrix is presented
as a Gram matrix: the quadratic form is a squared length. Invertibility of the
generator upgrades semidefinite to definite, congruence preserves the weaker
property unconditionally, and definiteness is exactly what makes the positive
square root invertible.

Chapter 9 applies the square root of this chapter to $A^{\mathsf H}A$ and reads
off the singular values, finally measuring the stretching that Chapter 6 could
only name and Chapter 7 could only illustrate. Part VI uses CFT-08-006 directly:
the completion argument balances a matrix by conjugating with $G^{1/2}$, and
every field of the square-root data is consumed there.
