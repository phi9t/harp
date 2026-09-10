---
id: cft-chapter-07-inner-product-spaces
title: Inner-product spaces
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-09
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 07_inner_product_spaces.md
chapter: 7
part: 2
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 7: Inner-product spaces

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_01_linear_structure/06_eigenvalues_and_polynomial_algebra|Chapter 6]]
Next: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/08_positive_operators_and_gram_geometry|Chapter 8 — Positive operators and Gram geometry]]

## Opening problem

Part I measured almost nothing. A linear map was compared with another linear
map, a subspace with another subspace, but no vector had a length and no two
vectors had an angle; the single exception was CFT-06-006, which stated a
density result in a norm it borrowed from this chapter. Chapter 6 closed with a warning that depends on exactly
that gap: the spectrum of $A_{\lambda,\alpha}$ is blind to $\alpha$, yet the
matrix visibly stretches. To say *how much* it stretches we need a pairing.

**Motivation.** One extra piece of structure — a pairing that is linear,
symmetric in the appropriate sense, and positive on nonzero vectors — buys
lengths, angles, orthogonal projection, a best-approximation principle, and a
canonical vector representative for each covector. It is also where Chapter 4's
transpose stops being the right notion: over $\mathbb C$ the pairing needs a
conjugate, and the adjoint parts company with the transpose. Chapter 6 already
met a place where the two fields differ — `planeRotation` has no real
eigenvector but diagonalizes over $\mathbb C$ — so what is new here is not that
they differ but that the difference reaches the pairing itself.

## Conceptual model

On $\mathbb R^n$ take the coordinate pairing $x\cdot y=\sum_i x_iy_i$. It is
symmetric, additive and homogeneous in each argument, and satisfies $x\cdot
x\ge0$ with equality only at $x=0$. Those four properties are all that the real
half of this chapter uses; none of them mentions a basis after the pairing is
fixed.

Given a nonzero $u$, the *line projection* of $x$ is
$$
P_ux=\frac{x\cdot u}{u\cdot u}\,u ,
$$
the unique point of the line $\mathbb Ru$ whose residual $x-P_ux$ is orthogonal
to $u$. The coefficient is forced: requiring $(x-tu)\cdot u=0$ gives
$t=(x\cdot u)/(u\cdot u)$, which is legal exactly because $u\cdot u\neq0$ when
$u\neq0$. Everything else in the real half — Pythagoras, best approximation,
Cauchy–Schwarz and its equality case — is read off this one splitting.

Over $\mathbb C$ the pairing must be changed, not merely copied. If $\langle
x,y\rangle$ were bilinear then $\langle ix,ix\rangle=-\langle x,x\rangle$ and
positivity would fail. The fix is conjugate-linearity in one slot, and its
matrix consequence is that the adjoint is the *conjugate* transpose. Chapter 4's
pullback $T^\vee$ needed no metric and involved no conjugation; the adjoint
$T^\dagger$ needs a metric and does. They are different operations that agree
only over $\mathbb R$.

**Historical context.** Presenting length and angle as consequences of a single
pairing is a pedagogical convention we adopt because it makes the hypotheses of
each later inequality visible. No attribution of the axioms or of the projection
formula to any particular author is intended.

## Formal development

### CFT-07-001: the projection splitting {#cft-07-001}

**Statement.** For any $u$ and $x$ in $\mathbb R^n$,
$x=P_ux+(x-P_ux)$.

**Proof.** The right-hand side is $P_ux+x-P_ux$, and addition on
$\mathbb R^n$ is commutative and associative with additive inverses, so the two
copies of $P_ux$ cancel and the expression reduces to $x$. Nothing else is
used: the identity is stated for every $u$, including $u=0$, because it is a
fact about the abelian group structure and not about the projection formula.

**Boundary and Lean provider.** No nonvanishing hypothesis appears, which is
what makes this card weaker than the next one and safe to state first.
[`projection_residual_decomposition`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L42)
is the public declaration; its checked proof discharges
$P_ux+(x-P_ux)=x$ by `abel` and then applies `.symm`, `abel` being exactly the
abelian-group cancellation the paragraph describes. The
same statement is available in the maintained foundations library as
`MathematicalFoundations.Orthogonality.line_projection_residual_decomposition`;
this chapter reproves it locally so that Part II owns its own development.

### CFT-07-002: the residual is orthogonal {#cft-07-002}

**Statement.** For $u\neq0$ in $\mathbb R^n$ and any $x$,
$(x-P_ux)\cdot u=0$.

**Proof.** First, $u\cdot u\neq0$: positive definiteness says $u\cdot u=0$ only
for $u=0$, which is excluded. Now expand the pairing over the subtraction and
pull the scalar out of the projection:
$$
(x-P_ux)\cdot u=x\cdot u-\Bigl(\frac{x\cdot u}{u\cdot u}\Bigr)(u\cdot u).
$$
Since $u\cdot u$ is a nonzero real number it cancels, leaving
$x\cdot u-x\cdot u=0$. The nonvanishing hypothesis is used exactly once, to
license that cancellation.

**Boundary and Lean provider.**
[`projection_residual_orthogonal`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L48)
runs those steps in that order: it derives $u\cdot u\neq0$ from
`dotProduct_self_eq_zero`, rewrites with `sub_dotProduct` and
`smul_dotProduct`, and closes by clearing the denominator. Positive
definiteness itself is
[`real_inner_self_eq_zero_iff`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L38);
the other three pairing axioms are
[`real_inner_comm`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L21),
[`real_inner_add_left`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L25)
and
[`real_inner_smul_left`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L29).

### Pythagoras, best approximation, and the zero line

Two consequences follow from CFT-07-002 together with the splitting CFT-07-001,
and both are used later in the book.

Splitting $x$ by CFT-07-001 and expanding the pairing of the sum with itself,
the two cross terms are $P_ux\cdot(x-P_ux)$ and its mirror image. Writing
$P_ux=cu$ and pulling $c$ out reduces each to a multiple of $(x-P_ux)\cdot u$,
which vanishes. What survives is
$$
x\cdot x=P_ux\cdot P_ux+(x-P_ux)\cdot(x-P_ux),
$$
checked as
[`projection_pythagoras`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L57).

The same cancellation proves best approximation. For any scalar $t$ write
$x-tu=(x-P_ux)+(P_ux-tu)$. The second summand is again a multiple of $u$, so it
pairs to zero with the residual, and expanding gives the residual's own square
plus a nonnegative term. Hence no point of the line beats $P_ux$; this is
[`projection_minimizes`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L77).
Note which hypothesis each result consumes: the splitting needs none,
orthogonality needs $u\neq0$, and both consequences inherit that requirement.

The hypothesis is not decorative.
[`lineProjection_zero`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L95)
records that $P_0x=0$ for every $x$, so with $u=0$ the residual is $x$ itself
and orthogonality would assert $x\cdot0=0$ — true but vacuous, and best
approximation would degenerate to a statement about the single point $0$.

### Cauchy–Schwarz and its equality case

Best approximation is Cauchy–Schwarz in disguise. Pythagoras reads
$x\cdot x=P_ux\cdot P_ux+r\cdot r$ with $P_ux\cdot P_ux=(x\cdot u)^2/(u\cdot u)$.
Discarding the nonnegative *residual* term $r\cdot r$ leaves
$(x\cdot u)^2/(u\cdot u)\le x\cdot x$, which is
$(x\cdot u)^2\le(x\cdot x)(u\cdot u)$, with equality exactly when the residual
vanishes, that is when $x$ lies on the line.

This book does not reprove that inequality here. The maintained foundations
laboratory proves it by four independent routes with a compiled dependency
audit, and this chapter reuses the quadratic route:
[`cauchy_schwarz_real`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L101)
restates it in the pairing notation used here and is discharged by
`TextbookBench.cauchySchwarzQuadratic`. The equality case is
[`cauchy_schwarz_real_equality_iff`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L107),
discharged by `TextbookBench.cauchySchwarzEqualityIffProportional`, and its
conclusion is the proportionality statement $x=0$ or $y=tx$ rather than a
vaguer "linear dependence". Reuse is labeled, not silent: those two
declarations carry the laboratory's proof, not a new one, and the laboratory's
own controls are what establish that its four routes do not borrow from each
other.

### CFT-07-003: the pairing in coordinates {#cft-07-003}

**Statement.** For a complex square matrix $A$ and a Euclidean vector $x$,
$$
\langle x,Ax\rangle=\overline{x}^{\,\mathsf T}(Ax),
$$
where the bar is entrywise conjugation and the product on the right is the
coordinate pairing.

**Proof.** The Euclidean inner product is *defined* in coordinates by the
conjugated pairing, in the order $\langle x,y\rangle=y^{\mathsf
T}\overline{x}$; the library records this as
`EuclideanSpace.inner_eq_star_dotProduct`. Applying it to $y=Ax$ and then
commuting the coordinate pairing, which is symmetric as an unconjugated bilinear
sum, produces the displayed order. The whole content is bookkeeping between two
conventions for which factor is conjugated and which is written first.

**Boundary and Lean provider.** The index type is finite with decidable
equality and the scalars are complex; no positivity, normality or spectral
hypothesis appears.
[`adjoint_coordinate_identity`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L118)
re-exports `CrouzeixConjecture.inner_euclideanOperator_eq_star_dotProduct`,
whose checked proof is exactly the two steps above:
`EuclideanSpace.inner_eq_star_dotProduct` followed by `dotProduct_comm`.

### CFT-07-004: the adjoint is the conjugate transpose {#cft-07-004}

**Statement.** For a complex square matrix $A$, the operator induced by
$A^{\mathsf H}=\overline{A}^{\,\mathsf T}$ is the Hilbert-space adjoint of the
operator induced by $A$.

**Proof.** The map sending a matrix to its induced Euclidean operator is a
homomorphism of star-algebras: it respects sums and products, and it carries
the matrix star operation, which is conjugate transposition, to the operator
star operation, which is the adjoint. The statement is that homomorphism's
compatibility with the star, applied to $A$.

**Boundary and Lean provider.**
[`adjoint_matrix_is_conjugate_transpose`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L121)
re-exports `CrouzeixConjecture.euclideanOperator_conjTranspose`, whose checked
proof is the single application `map_star (euclideanOperator)`. The prose above
is a description of that one step; the work sits in the star-algebra structure
of `euclideanOperator`, which the maintained development supplies, not in this
card.

Two conventions must be kept apart here. Chapter 4's pullback of a covector
uses no metric and introduces no conjugation; the adjoint of this chapter uses
the Hermitian metric and does. Over $\mathbb R$ they coincide, which is why the
distinction is invisible until complex scalars appear.
[`transpose_ne_conjTranspose_complex`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L134)
settles it with one matrix: for $\operatorname{diag}(i,0)$ the transpose has
$i$ in the corner and the conjugate transpose has $-i$, and $i\neq-i$.

### CFT-07-005: the matrix norm is the operator norm {#cft-07-005}

**Statement.** For a complex square matrix $A$, $\|A\|=\|A_{\mathrm{op}}\|$,
where the left side is this development's matrix norm and the right side is the
operator norm of the induced map on Euclidean space.

**Proof.** There is nothing to compute: the matrix norm used throughout this
book is *defined* to be the operator norm of the induced Euclidean map, so the
two sides are the same object under different notation.

**Boundary and Lean provider.**
[`matrix_norm_is_operator_norm`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L124)
re-exports `CrouzeixConjecture.matrix_norm_eq_euclidean_operator_norm`, whose
checked proof is `rfl` — definitional equality, which is the formal reading of
"nothing to compute". The card is worth stating because later chapters bound
$\|p(A)\|$ and the reader is entitled to know which norm that is. It is not the
Frobenius norm and not an entrywise maximum; those are different functions that
would make the later inequalities false or trivial.

### CFT-07-006: norms are nonnegative {#cft-07-006}

**Statement.** In any seminormed additive group, $0\le\|x\|$.

**Proof.** From the triangle inequality in the form
$\|x-y\|\le\|x\|+\|y\|$, taken at $y=x$, one gets
$0=\|x-x\|\le\|x\|+\|x\|$, hence $0\le\|x\|$.

**Boundary and Lean provider.** The hypothesis is a seminormed additive group,
which is weaker than the normed spaces of the rest of the chapter: it allows
nonzero vectors of zero length.
[`norm_is_nonnegative`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L127)
runs exactly that argument: it applies `norm_sub_le` at $y=x$, rewrites
$\|x-x\|$ to $0$ with `sub_self` and `norm_zero`, and closes with `linarith`.
It is proved locally rather than re-exported, deliberately: Mathlib's
`norm_nonneg` is the same conclusion and is a `simp` lemma, so a proof that
invoked `simp` here would have been a restatement of the card rather than a
derivation of it. This is the weakest card in the chapter and is stated because
every later norm inequality silently assumes it.

## Worked examples

Take $u=(1,0)$ and $x=(3,4)$ in $\mathbb R^2$. Then $x\cdot u=3$ and
$u\cdot u=1$, so $P_ux=(3,0)$ and the residual is $(0,4)$. Orthogonality is
$(0,4)\cdot(1,0)=0$, and Pythagoras reads $25=9+16$. Any other point $tu$ of
the horizontal axis gives $\|x-tu\|^2=(3-t)^2+16\ge16$, with equality only at
$t=3$. E01 and E02 check the projection and the two numbers.

The cumulative family shows why this chapter is needed. For
$$
A_{\lambda,\alpha}=\begin{bmatrix}\lambda&\alpha\\0&\lambda\end{bmatrix},
\qquad
A_{\lambda,\alpha}\begin{bmatrix}0\\1\end{bmatrix}
=\begin{bmatrix}\alpha\\\lambda\end{bmatrix},
$$
the image of a unit vector has squared length $\alpha^2+\lambda^2$, which grows
without bound in $\alpha$ while Chapter 5's determinant stays $\lambda^2$ and
its trace stays $2\lambda$. The pairing sees what the two similarity invariants
cannot. Chapter 9 turns that observation into the singular values.

Over $\mathbb C$ take $A=\operatorname{diag}(i,0)$. Its transpose is itself;
its conjugate transpose is $\operatorname{diag}(-i,0)$. So $A$ is symmetric in
Chapter 4's sense and *not* self-adjoint in this chapter's sense. E05 checks the
inequality of the two matrices.

## ML bridge

**Mathematical object.** A Gram matrix $G_{ij}=\langle v_i,v_j\rangle$ records
every pairwise pairing of a finite family, and a kernel method is a computation
that touches the data only through such a matrix. Least squares is the
projection of this chapter applied to the column space of a design matrix.

**Exact transfer.** CFT-07-002 is the one-dimensional case of the normal
equation: the residual of the fitted value is orthogonal to the single direction
projected onto. The full normal equation, orthogonality to every direction the
model can produce, needs projection onto a subspace, which is the restriction
recorded at the end of this section. The best
approximation statement is why the least-squares solution is the minimizer, not
merely a stationary point, and Cauchy–Schwarz with its equality case is exactly
the statement that a correlation coefficient lies in $[-1,1]$ and reaches an
endpoint only under exact proportionality.

**Non-transfer.** These results concern one fixed pairing on a
finite-dimensional space over an exact scalar field. They do not say that a
learned embedding's dot product is an inner product of anything meaningful,
that a normalized similarity is a metric, or that a floating-point Gram matrix
is positive semidefinite — rounding routinely produces small negative
eigenvalues, and no statement here bounds that. The projection formula also
assumes a single direction; projecting onto a subspace of dimension greater than
one requires an orthogonalization that this book does not develop, in this
chapter or any later one.

**Calculation.** With $v_1=(1,0)$ and $v_2=(3,4)$ the Gram matrix is
$\begin{bmatrix}1&3\\3&25\end{bmatrix}$, whose determinant $25-9=16$ is exactly
the squared length of the residual computed above. That is not a coincidence:
Chapter 8 shows the Gram determinant measures squared volume, and here the
volume of the parallelogram on $v_1,v_2$ is base $1$ times height $4$.

## Lean translation

The real pairing is Mathlib's `dotProduct`, written `x ⬝ᵥ y`. The projection is
`MathematicalFoundations.Orthogonality.lineProjection`, reused rather than
redefined so that this chapter and the foundations laboratory cannot drift
apart. The complex side uses `CrouzeixConjecture.EuclideanVector` and
`euclideanOperator`, with `⟪x, y⟫_ℂ` for the Hermitian pairing, `Aᴴ` for the
conjugate transpose and `ContinuousLinearMap.adjoint` for the operator adjoint.

Three cards of this chapter are exact re-exports of maintained
`CrouzeixConjecture` theorems and name their underlying declaration in the
contract; three are proved here. The Cauchy–Schwarz results are labeled reuse of
the foundations laboratory and are support declarations, not indexed cards.

The six exercise theorems live in
`CrouzeixTextbook.Part02.Exercises.Chapter07` in the
[chapter Lean source](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean).

## Exercises

### CFT-07-E01 -- retrieval {#exercise-cft-07-e01}

State the projection splitting for an arbitrary direction in $\mathbb R^2$, and
compute the projection of $(3,4)$ onto the direction $(1,0)$.

**Solution.** The splitting is the abelian-group identity of CFT-07-001 and
needs no hypothesis. The computation gives $x\cdot u=3$, $u\cdot u=1$, hence
$P_ux=3\,(1,0)=(3,0)$.
[`exercise_01_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L148)
states the general splitting for that direction and the explicit projected
vector; its checked proof is `abel` for the first conjunct and a coordinatewise
expansion of the definition for the second.

### CFT-07-E02 -- calculation {#exercise-cft-07-e02}

For the same data compute the residual pairing $(x-P_ux)\cdot u$ and the
squared length $x\cdot x$.

**Solution.** The residual is $(0,4)$, so its pairing with $(1,0)$ is $0$, and
$x\cdot x=9+16=25$.
[`exercise_02_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L157)
states both numbers; its checked proof expands the two-term sums and evaluates,
which is the arithmetic displayed here rather than an appeal to CFT-07-002.

### CFT-07-E03 -- written-proof {#exercise-cft-07-e03}

For $y\neq0$ in $\mathbb R^n$ and arbitrary $x$, assemble the three facts the
projection splitting yields: the Cauchy–Schwarz bound, orthogonality of the
residual, and the Pythagoras identity.

**Solution.** Cauchy–Schwarz is the reused laboratory result; orthogonality is
CFT-07-002 with $u=y$; Pythagoras is the cancellation of the two cross terms
described above.
[`exercise_03_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L166)
states the three conclusions together for every dimension, every $x$ and every
nonzero $y$, so the shared hypothesis is visible in one statement.

### CFT-07-E04 -- written-proof {#exercise-cft-07-e04}

Prove the equality case of Cauchy–Schwarz in the proportionality form, and
verify directly that every proportional pair attains equality.

**Solution.** The equality case is the laboratory's proportionality theorem:
equality holds exactly when $x=0$ or $y=tx$ for some scalar $t$. For the
converse direction one does not need it: substituting $y=t\,x$ and pulling the
scalar out of both sides gives $t^2(x\cdot x)^2$ on the left and
$t^2(x\cdot x)(x\cdot x)$ on the right.
[`exercise_04_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L175)
states the characterization and that direct verification as two conjuncts; the
second is proved by pulling scalars through the pairing and normalizing, not by
citing the first.

### CFT-07-E05 -- boundary {#exercise-cft-07-e05}

Show that the projection degenerates on the zero direction, and that over
$\mathbb C$ the transpose is not the adjoint.

**Solution.** $P_0x=0$ because the defining scalar has a zero numerator and the
direction is zero. For the second, $\operatorname{diag}(i,0)$ equals its own
transpose but its conjugate transpose is $\operatorname{diag}(-i,0)$, and
$i\neq-i$.
[`exercise_05_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L183)
states both. Together they mark the two hypotheses this chapter cannot drop:
the direction must be nonzero, and the scalars determine which transpose is the
metric one.

### CFT-07-E06 -- lean-proof {#exercise-cft-07-e06}

For an arbitrary complex square matrix, state that its norm is nonnegative and
that it agrees with the operator norm of the induced Euclidean map.

**Solution.** Nonnegativity is the same statement as CFT-07-006, though the
checked solution discharges it with Mathlib's `norm_nonneg` rather than by
citing the card; the agreement is CFT-07-005, which holds definitionally.
[`exercise_06_solution`](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter07.lean#L190)
states both for every finite index type. Reading it beside the two cards shows
which norm every later bound in this book is about.

## Synthesis and forward dependencies

A pairing turns linear structure into geometry. One splitting — projection plus
orthogonal residual — yields Pythagoras, best approximation, Cauchy–Schwarz and
its equality case, and every one of those inherits the single hypothesis that
the direction is nonzero. Over $\mathbb C$ the pairing must be conjugate-linear
in one slot, and the adjoint replaces the transpose.

Chapter 8 keeps the pairing and asks which operators are nonnegative with
respect to it, turning the Gram matrix of this chapter's ML bridge into the
central object. Chapter 9 replaces the single direction by an orthonormal
family and produces the singular values, which finally measure the stretching
that Chapter 6 could only point at.
