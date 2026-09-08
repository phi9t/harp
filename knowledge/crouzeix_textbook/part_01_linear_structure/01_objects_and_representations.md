---
id: cft-chapter-01-objects-and-representations
title: Objects and representations
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-07
tags: [crouzeix-textbook, linear-structure, mathematics, lean]
confidence: high
canonical: 01_objects_and_representations.md
chapter: 1
part: 1
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 1: Objects and representations

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-i-linear-structure|Part I -- Linear structure]]
Previous: Start of book
Next: [[knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces|Chapter 2]]

## Opening problem

### Motivation

Suppose a linear operation has matrix

$$
\Lambda=\begin{bmatrix}0&0\\0&1\end{bmatrix}
$$

in one basis. In a second basis, a computation produces

$$
T=\begin{bmatrix}0&1\\0&1\end{bmatrix}.
$$

Are these different operations? They have the same eigenvalues, but one has
Euclidean operator norm $1$ and the other has norm $\sqrt{2}$. Which facts
belong to the underlying transformation, which facts belong to the chosen
coordinates, and which facts also depend on the geometry placed on those
coordinates?

This distinction is the first load-bearing idea of the book. Crouzeix's
problem is stated for matrices and their numerical ranges, yet many proof
moves change coordinates, compare norms, or pass through functional calculi.
If object and representation are blurred at the start, later similarities can
look unitary when they are not.

## Historical context

This opening follows an object-before-coordinates teaching order. It makes no
claim about who first introduced linear maps, bases, or similarity. Historical
priority is not needed for the mathematical distinction developed here.

## Conceptual model

A vector is not intrinsically a column of numbers. A vector becomes a column
after a basis is chosen. Likewise, a linear transformation $T:V\to W$ is a
function preserving addition and scalar multiplication. It becomes a matrix
after bases are chosen for $V$ and $W$.

It helps to keep three layers separate:

1. **Algebraic object:** spaces, vectors, and the transformation itself.
2. **Coordinate representation:** tuples and matrices obtained from bases.
3. **Geometry:** a norm or inner product used to measure those tuples.

A basis change alters layer 2. If the new basis is not orthonormal, the usual
Euclidean norm on the new coordinate tuples also changes how layer 3 is being
interpreted. Similarity preserves algebraic invariants such as the
characteristic polynomial. Arbitrary similarity does not preserve Euclidean
operator norm.

## Formal development

Throughout the coordinate arguments, $V$ and $W$ are vector spaces over a
field $\mathbb K$, with finite ordered bases. Real and complex spaces are the
geometric specializations. We use the basis universal property explicitly:
every $x\in V$ has exactly one expansion $x=\sum_jx_jb_j$; prescribing the
images of the $b_j$ therefore determines a unique linear map. To see the
second assertion directly, set $T(\sum_jx_jb_j)=\sum_jx_jw_j$ for prescribed
$w_j\in W$. Uniqueness of coordinates makes this well-defined; adding or
scaling coordinate expansions proves linearity; evaluating at $b_j$ gives
$w_j$. Every other linear map with those basis images has the same formula.
Chapter 2 develops the span and independence criteria for obtaining a basis;
this chapter assumes that the displayed lists already are bases.

### Linear transformations before coordinates {#cft-01-001}

**Definition CFT-01-001 (linear transformation).** **Statement.** Let $V$ and $W$ be vector
spaces over the same scalar field $\mathbb K$. A map $T:V\to W$ is linear when

$$
T(x+y)=T(x)+T(y),\qquad T(ax)=aT(x)
$$

for every $x,y\in V$ and scalar $a\in\mathbb K$.

No basis occurs in this definition. That omission is mathematical content,
not a refusal to compute. It tells us which statements should survive every
change of coordinates. For example, linearity forces $T(0)=0$: since
$0=0+0$,

$$
T(0)=T(0+0)=T(0)+T(0),
$$

and cancellation gives $T(0)=0$. Lean packages the map and its preservation
laws into `V →ₗ[𝕜] W`; the textbook alias `LinearTransformation` mentions no
basis.

**Formal correspondence.** Definition
`CrouzeixTextbook.Part01.LinearTransformation`, whose provider is Mathlib's
`LinearMap`. [Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean).
The alias also allows semirings and semimodules; the definition above is its
field specialization. There is no theorem proof obligation for a definition.
`linear_map_preserves_zero` is an unindexed structural consequence.

### Columns are images of basis vectors {#cft-01-002}

Choose an ordered basis $B=(b_1,\ldots,b_n)$ of $V$ and
$C=(c_1,\ldots,c_m)$ of $W$. Write

$$
T(b_j)=\sum_{i=1}^m a_{ij}c_i.
$$

**Theorem CFT-01-002 (column rule).** **Statement.** For finite ordered bases $B$
of $V$ and $C$ of $W$, and a $\mathbb K$-linear map $T:V\to W$, the $j$th column of the representing
matrix $A=[T]_{C\leftarrow B}$ is the coordinate vector of $T(b_j)$ in basis
$C$.

**Proof.** By definition, entry $a_{ij}$ is the coefficient of $c_i$ in the
unique basis expansion of $T(b_j)$. Collecting those coefficients down the
$j$th column gives $[T(b_j)]_C$. Nothing else is required. The theorem is
almost definitional because a matrix is constructed precisely by recording
these images. $\square$

**Boundary.** The matrix can be rectangular: $m$ output coordinates and $n$
input coordinates. With a spanning list that is not a basis, the coefficients
need not be unique; with an independent list that does not span, the values
do not determine $T$ on the whole space.

**Formal correspondence.** Public theorem
`CrouzeixTextbook.Part01.basis_image_columns`, using
`LinearMap.toMatrix_apply`.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean).
The statement has a field, two vector spaces, finite basis index types,
explicit bases $B,C$, and indices $i,j$. It states
`LinearMap.toMatrix B C T i j = C.repr (T (B j)) i`.
The older `matrix_column_is_basis_image` remains a standard-coordinate helper;
it is not the arbitrary-basis theorem.

### Coordinate action is matrix multiplication {#cft-01-003}

Let $x=\sum_j x_jb_j$. Linearity and the column rule give

$$
T(x)=\sum_j x_jT(b_j)
=\sum_i\left(\sum_j a_{ij}x_j\right)c_i.
$$

**Theorem CFT-01-003 (coordinate action).** **Statement.** For the finite bases and
linear map of CFT-01-002 and every $x\in V$,

$$
[T(x)]_C=[T]_{C\leftarrow B}[x]_B.
$$

**Proof.** The basis universal property gives $x=\sum_jx_jb_j$ uniquely.
Additivity moves $T$ through the finite sum; homogeneity replaces
$T(x_jb_j)$ with $x_jT(b_j)$. Substitute
$T(b_j)=\sum_i a_{ij}c_i$, distribute each scalar, and interchange the two
finite sums. Commutativity of $\mathbb K$ turns $x_ja_{ij}$ into $a_{ij}x_j$.
Uniqueness of the $C$ expansion now identifies the coefficient of $c_i$ as
$\sum_j a_{ij}x_j$, exactly the $i$th entry of the matrix-vector product.
$\square$

This proof explains matrix multiplication rather than merely declaring a
rule. The inner sum says: scale each basis image by the input coordinate and
add the contributions.

**Boundary.** A nonlinear map such as $f(t)=t^2$ on $\mathbb R$ agrees with
the identity on the basis vector $1$, but $f(2)=4\ne2$. Recording basis
images only determines the whole map under the linearity hypothesis.

**Formal correspondence.** Proof written here:
`CrouzeixTextbook.Part01.basis_coordinate_action`; substantive basis provider
`Module.Basis.sum_repr` and coordinate provider `LinearMap.toMatrix_apply`.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean).
It assumes the same field and finite bases as CFT-01-002 and proves
`C.repr (T x) = LinearMap.toMatrix B C T *ᵥ B.repr x` as coordinate functions.
The proof expands $x$ in $B$ and moves the linear maps through the sum.
The preserved `coordinate_action_eq_mulVec` is only a definitional interface
checkpoint and is not this proof.

### Change of basis produces similarity {#cft-01-004}

Now take an endomorphism $T:V\to V$. Suppose coordinate vectors in two bases
are related by

$$
[x]_{B'}=S[x]_B,
$$

where $S$ is invertible. Let $A=[T]_B$ and $A'=[T]_{B'}$. Then

$$
A'[x]_{B'}=[T(x)]_{B'}=S[T(x)]_B=SA[x]_B=SA S^{-1}[x]_{B'}.
$$

**Theorem CFT-01-004 (change of basis).** **Statement.** Let $B,B'$ be finite
ordered bases of the same vector space $V$ over $\mathbb K$, with the same
index set, and let $T:V\to V$ be linear. Set
$S=[\mathrm{id}_V]_{B'\leftarrow B}$ and
$R=[\mathrm{id}_V]_{B\leftarrow B'}$. Then $R=S^{-1}$ and
$[T]_{B'}=S[T]_B R$. Thus matrices representing the same endomorphism in different
bases are similar.

**Proof.** By CFT-01-003 applied to the identity map, $S$ sends $B$
coordinates to $B'$ coordinates, and $R$ reverses that process. The unique
basis expansion reconstructs the same vector after either round trip, so
$RSx=x$ and $SRy=y$ for every coordinate column $x,y$. In particular this
holds for each standard coordinate vector, so the column rule yields
$RS=SR=I$. For any $y$, reconstruct the vector $v$ with $[v]_{B'}=y$.
Its old coordinates are $Ry$, whence
$[T]_{B'}y=[T(v)]_{B'}=S[T(v)]_B=S[T]_BRy$.
Apply this to each standard coordinate vector again to obtain equality of
matrices, rather than just equality at one vector. $\square$

The proof has one important direction convention: $S$ sends old coordinates
to new coordinates. If a text chooses the inverse convention, the displayed
conjugation reverses. Memorizing a bare formula is therefore fragile; deriving
it from the coordinate diagram is safer.

**Boundary.** A noninvertible encoding loses information and need not give a
basis change. For example, $(x,y)\mapsto x$ cannot reconstruct $(0,1)$.

**Formal correspondence.** Proof written here:
`CrouzeixTextbook.Part01.basis_change_conjugacy`, provider
`LinearMap.toMatrix_comp` for composition in specified bases.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean).
The exact formal equality uses the two matrices of the identity, $S$ and $R$,
instead of a separately chosen inverse operation. The inverse conclusion
follows by applying the same composition rule to the identity round trips;
it is also proved in the unindexed `basis_change_inverse` helper.
The old `change_basis_action` remains an algebraic action checkpoint with an
explicit left-inverse assumption.

### Similarity invariants {#cft-01-005}

**Theorem CFT-01-005 (characteristic polynomial).** **Statement.** Let $A$ be a
finite square matrix over a commutative ring $\mathbb K$, and let $S$ be a
unit in that matrix ring (an invertible matrix with a two-sided inverse).
If $A'=SAS^{-1}$, then the following is an equality of polynomials in
$\mathbb K[t]$:

$$
\det(tI-A')=\det(tI-A).
$$

**Proof.** This is a forward-reference proof using determinant
multiplicativity, whose development belongs to Chapter 5. Regard $S$ and $A$
as constant matrices over $\mathbb K[t]$. Since $t$ is central,

$$
tI-SAS^{-1}=S(tI-A)S^{-1}.
$$

Using multiplicativity of determinant,

$$
\det\!\left(S(tI-A)S^{-1}\right)
=\det(S)\det(tI-A)\det(S^{-1})
=\det(tI-A).
$$

Here $\det(S)\det(S^{-1})=\det(SS^{-1})=\det(I)=1$;
commutativity allows those two factors to be moved together. Over a field,
the polynomial equality also gives the same roots and root multiplicities
in any common splitting field. The spectral interpretation is another
forward dependency. This calculation exposes the required determinant facts;
it does not establish determinant theory from Chapter 1's prerequisites.
$\square$

An invariant answers a specific equivalence question. “Same eigenvalues” does
not mean “same geometry.” The Euclidean norm on coordinate columns is
preserved by unitary changes of basis, not by every invertible $S$.

**Formal correspondence.** Public theorem
`CrouzeixTextbook.Part01.similarity_preserves_charpoly`, using
`Matrix.charpoly_units_conj`.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean).
Lean assumes a commutative ring, a finite square index type with decidable
equality, and a matrix unit. The correspondence is exact; the prose proof
scope is a summary with the explicit Chapter 5 dependency. Formal availability
does not make this a self-contained first-chapter determinant proof.

### A nonnormality witness {#cft-01-006}

**Worked theorem CFT-01-006 (sharp norm counterexample).** **Statement.** Over
$\mathbb R$, let

$$
S=\begin{bmatrix}1&1\\0&1\end{bmatrix},\qquad
S^{-1}=\begin{bmatrix}1&-1\\0&1\end{bmatrix},\qquad
\Lambda=\begin{bmatrix}0&0\\0&1\end{bmatrix}.
$$

Then $S^{-1}$ is the displayed two-sided inverse, $T=S\Lambda S^{-1}$,
and $\|\Lambda\|_2=1$ while $\|T\|_2=\sqrt2$.
Here $\|A\|_2=\sup_{\|x\|_2=1}\|Ax\|_2$ is the Euclidean operator norm.

**Proof.** The two products $SS^{-1}$ and $S^{-1}S$ each have diagonal
entries $1,1$ and off-diagonal entries $0,0$, hence both are $I$.
Direct multiplication gives

$$
S\Lambda S^{-1}=\begin{bmatrix}0&1\\0&1\end{bmatrix}=T.
$$

For $e_2=(0,1)^\mathsf{T}$,

$$
\Lambda e_2=(0,1)^\mathsf{T},\qquad
Te_2=(1,1)^\mathsf{T}.
$$

Their squared Euclidean lengths are $1$ and $2$. In fact, for
$x=(x_1,x_2)^\mathsf{T}$,

$$
Tx=(x_2,x_2)^\mathsf{T},\qquad
\lVert Tx\rVert_2=\sqrt{2}|x_2|
\le \sqrt{2}\lVert x\rVert_2.
$$

Equality holds at $e_2$, so $\lVert T\rVert_2=\sqrt{2}$, while
$\|\Lambda x\|_2=|x_2|\le\|x\|_2$ with equality at $e_2$, proving
$\lVert\Lambda\rVert_2=1$. Similarity preserved the characteristic polynomial
$t(t-1)$ but changed the Euclidean operator norm. This is the smallest useful
warning that spectral data alone does not control nonnormal geometry.
Moreover, $Se_2=(1,1)^{\mathsf T}$ has length $\sqrt2$, so $S$ is not
orthogonal. $\square$

**Boundary.** Different output lengths on one unit vector alone would not
prove different operator norms. The uniform upper bounds and their attained
values are both necessary here. The calculation compares the standard
Euclidean norms on the two coordinate spaces; it does not change a fixed
intrinsic norm on the underlying vector space.

**Formal correspondence.** Proof written here:
`CrouzeixTextbook.Part01.nonunitary_similarity_norm_counterexample`.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean).
It proves both inverse identities, similarity, the uniform squared bounds
$\sum_i(\Lambda x)_i^2\le\sum_i x_i^2$ and
$\sum_i(Tx)_i^2\le2\sum_i x_i^2$, and a unit input attaining $1$ and $2$.
Its providers are the local arithmetic lemmas `nonnormalExample_similarity`
and `nonunitary_similarity_changes_output_length_sq`; the bound proof is
written here. Taking nonnegative square roots and the stated supremum gives
the displayed norm values. Lean uses explicit sums of real squares, not the
default norm instance on the function type `Fin 2 → ℝ`.

The compiler records declaration assumptions and foundational axioms in the
generated [[knowledge/crouzeix_textbook/lean_coverage_ledger|Lean coverage ledger]].
Those records concern the formal declarations; they do not certify the
motivation, historical framing, or ML interpretation.

## Worked examples

### Example 1: Build a matrix from a transformation

On polynomials of degree at most two, let $D(p)=p'$. In the ordered basis
$B=(1,t,t^2)$,

$$
D(1)=0,\qquad D(t)=1,\qquad D(t^2)=2t.
$$

The column rule therefore gives

$$
[D]_B=\begin{bmatrix}
0&1&0\\
0&0&2\\
0&0&0
\end{bmatrix}.
$$

For $p(t)=a+bt+ct^2$, multiplication produces $(b,2c,0)$, the coordinate
vector of $p'(t)=b+2ct$. The columns make the action transparent.

### Example 2: Derive, do not guess, a basis change

Let $T(x,y)=(0,y)$ in the standard basis, so
$A=\begin{bmatrix}0&0\\0&1\end{bmatrix}$. Choose
$B'=((1,0),(-1,1))$. If $P$ has the new basis vectors as columns, then standard
coordinates satisfy $[x]_{\mathrm{std}}=P[x]_{B'}$. Thus the matrix in $B'$ is
$P^{-1}AP$. Here

$$
P=\begin{bmatrix}1&-1\\0&1\end{bmatrix},\qquad
P^{-1}AP=
\begin{bmatrix}1&1\\0&1\end{bmatrix}
\begin{bmatrix}0&0\\0&1\end{bmatrix}
=\begin{bmatrix}0&1\\0&1\end{bmatrix}.
$$

The second basis vector maps to $(0,1)=(1,0)+(-1,1)$, so its new
coordinate column is $(1,1)^{\mathsf T}$, independently checking the product.
This identifies the two bases behind the opening problem.

The apparent conflict with CFT-01-004 is only notation: there $S$ sent old
coordinates to new coordinates, whereas $P$ here sends new coordinates to old
coordinates. The diagram determines the conjugation.

### Example 3: One operator, the family $A_{\lambda,\alpha}$

For real $\lambda,\alpha$, let an operator have standard matrix

$$
A_{\lambda,\alpha}=\begin{bmatrix}\lambda&\alpha\\0&\lambda\end{bmatrix}.
$$

For a nonzero real $s$, choose $B_s=((s^{-1},0),(0,1))$. A vector with old
coordinates $(x_1,x_2)$ has new coordinates $(sx_1,x_2)$, because
$(sx_1)(s^{-1},0)+x_2(0,1)=(x_1,x_2)$. Thus the old-to-new matrix is
$D_s=\operatorname{diag}(s,1)$, and

$$
D_s A_{\lambda,\alpha}D_s^{-1}
=\begin{bmatrix}s\lambda&s\alpha\\0&\lambda\end{bmatrix}
\begin{bmatrix}s^{-1}&0\\0&1\end{bmatrix}
=A_{\lambda,s\alpha}.
$$

This is one fixed operator in two bases. On the input $e_2$, whose old and
new coordinates agree, its output coordinates are respectively
$(\alpha,\lambda)$ and $(s\alpha,\lambda)$. Their standard squared lengths
are

$$
\|A_{\lambda,\alpha}e_2\|_2^2=\alpha^2+\lambda^2,
\qquad
\|A_{\lambda,s\alpha}e_2\|_2^2=s^2\alpha^2+\lambda^2.
$$

For $s=2$, $\lambda=0$, $\alpha=1$, these are $1$ and $4$.
The corresponding operator norms are $1$ and $2$, since
$A_{0,a}(x_1,x_2)=(ax_2,0)$ has norm at most $|a|\|(x_1,x_2)\|_2$,
with equality at $e_2$. The basis change is nonorthogonal because it sends
$e_1$ to $2e_1$. For general $s$, the displayed output length changes exactly
when $\alpha\ne0$ and $s^2\ne1$.

If the original geometric norm is transported to the new coordinates, it is
$\|(y_1,y_2)\|_{B_s}^2=s^{-2}y_1^2+y_2^2$.
The new output then has squared length
$s^{-2}(s\alpha)^2+\lambda^2=\alpha^2+\lambda^2$, as it must for the same
vector. The discrepancy arose from imposing a fresh standard Euclidean
metric on the new coordinate array.

Unindexed Lean helpers `runningFamily_change_basis` (with $s\ne0$) and
`runningFamily_output_length_sq` prove the matrix and squared-length
calculations. [Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter01.lean).
They do not add indexed theorem cards.

## ML bridge

**Mathematical object.** A fixed linear hidden-state update $h_{k+1}=Ah_k$
on $\mathbb R^n$, together with an invertible encoding $z=Sh$.

**Exact transfer.** Substitution gives
$z_{k+1}=Sh_{k+1}=SAh_k=SAS^{-1}z_k$. This is exactly CFT-01-004.
For different input and output spaces, separate invertible changes instead
give $A'=S_{\mathrm{out}}AS_{\mathrm{in}}^{-1}$ by the same coordinate rule.

**Non-transfer.** This algebra does not assert that nonlinear activations
commute with $S$, that a dimension-reducing encoding is invertible, or that
finite-precision errors are invariant. It supplies no conclusion about
learning dynamics or gradient statistics. Coordinate Euclidean norms require
an additional geometric choice.

**Calculation.** Take $A=A_{0,1}$, $S=\operatorname{diag}(2,1)$, and $h=e_2$.
Then $Ah=e_1$ and $z=e_2$, but $SAS^{-1}z=2e_1$.
The squared output length changes from $1$ to $4$ in standard coordinates.
With the transported norm $\|z\|_S^2=z_1^2/4+z_2^2$, the latter output has
squared length $1$. The represented update and the metric must both be
specified before interpreting an activation magnitude.

## Lean translation

Lean makes the object/representation split visible. A linear map has its own
type, while a matrix has explicit row and column index types.

```lean
abbrev LinearTransformation
    (𝕜 V W : Type*) [Semiring 𝕜]
    [AddCommMonoid V] [AddCommMonoid W]
    [Module 𝕜 V] [Module 𝕜 W] :=
  V →ₗ[𝕜] W

theorem matrix_column_is_basis_image
    {𝕜 n : Type*} [Semiring 𝕜] [Fintype n] [DecidableEq n]
    (A : Matrix n n 𝕜) (i j : n) :
    (A *ᵥ Pi.single j 1) i = A i j := by
  exact congrFun (Matrix.mulVec_single_one A j) i
```

The theorem says that multiplying by the $j$th standard coordinate vector
returns column $j$. `Fintype n` makes the matrix sum finite;
`DecidableEq n` lets Lean distinguish the selected coordinate; `Semiring 𝕜`
provides zero, one, addition, and multiplication. These hypotheses are not
ceremony: each supports an operation in the statement or proof.

This displayed helper is limited to the standard basis. The arbitrary-basis
claims are `basis_image_columns`, `basis_coordinate_action`, and
`basis_change_conjugacy`, whose explicit `Module.Basis` arguments carry the
reconstruction property. The full chapter module also checks the sharp
squared norm counterexample, the running-family calculations, Mathlib's
characteristic-polynomial invariance theorem, and all six reference solutions.
See the generated
[[knowledge/crouzeix_textbook/lean_coverage_ledger|Lean coverage ledger]].

## Exercises

### CFT-01-E01 -- retrieval {#exercise-cft-01-e01}

State the two laws defining a linear transformation. Derive $T(0)=0$ without
using coordinates.

### CFT-01-E02 -- calculation {#exercise-cft-01-e02}

Let $A=\begin{bmatrix}2&-1\\3&4\end{bmatrix}$. Compute $Ae_1$ and $Ae_2$.
Explain why the answers are the columns of $A$.
In Lean, calculate both displayed products over `ℤ`; expand `mulVec` into its
two dot-product terms rather than citing the generic column checkpoint.

### CFT-01-E03 -- written proof {#exercise-cft-01-e03}

Let $A$ be a finite square matrix over a commutative semiring and $x$ a
coordinate column. Prove that its action is the linear combination of its
columns with coefficients $x_j$. In Lean, prove the column expansion
`coordinateAction A x = ∑ j, x j • fun i => A i j`; expand `mulVec`
coordinatewise rather than closing the goal by definitional equality.
For the field specialization, explain how the basis expansion in CFT-01-003
identifies this coordinate calculation with $[T(x)]_C$.

### CFT-01-E04 -- written proof {#exercise-cft-01-e04}

Assume the determinant identities announced in CFT-01-005. Prove that similar
matrices over a commutative ring have the same characteristic polynomial and
determinant. Mark the
uses of commutation with $tI$, determinant multiplicativity, and
$\det(S^{-1})=\det(S)^{-1}$.
In Lean, package characteristic-polynomial invariance together with determinant
invariance for conjugation by a matrix unit.

### CFT-01-E05 -- boundary problem {#exercise-cft-01-e05}

Prove preservation of characteristic polynomial and trace by arbitrary
similarity, using the announced invariant theorems. Give the displayed
diagonal/nonnormal pair as a counterexample to Euclidean norm preservation.
For a real matrix $U$ satisfying $U^{\mathsf T}U=I$, prove
$(Ux)\cdot(Ux)=x\cdot x$, and explain the resulting stronger norm property.
The Lean solution packages the characteristic polynomial and trace invariants,
the explicit similar diagonal/nonnormal pair whose output length changes, and
the general orthogonal boundary `UᵀU=I`, which preserves `x · x`.

### CFT-01-E06 -- Lean proof {#exercise-cft-01-e06}

In the chapter namespace, prove that the squared output length of
`nonnormalExample` on `secondCoordinateVector` is `2`. Use the compiled
two-part witness rather than redoing matrix arithmetic.

```lean
theorem exercise_06_solution :
    (∑ i, (nonnormalExample *ᵥ secondCoordinateVector) i ^ 2) = 2 :=
  nonunitary_similarity_changes_output_length_sq.1
```

### Solutions and checks

**E01.** The laws are additivity and homogeneity. Additivity gives
$T(0)=T(0)+T(0)$; cancel $T(0)$. Lean checks
`Exercises.Chapter01.exercise_01_solution`.

**E02.** $Ae_1=(2,3)^\mathsf{T}$ and
$Ae_2=(-1,4)^\mathsf{T}$. Matrix-vector multiplication against a standard
coordinate vector kills every term except the selected column. Lean's
`exercise_02_solution` expands and normalizes all four coordinates explicitly.

**E03.** In coordinate $i$, the proposed sum is
$\sum_jx_ja_{ij}=\sum_ja_{ij}x_j=(Ax)_i$. Equality in every coordinate gives
the identity. Commutativity of the semiring is used to exchange the scalar
factors. This is `exercise_03_solution`. Over a field, CFT-01-003 supplies
the separate basis reconstruction and linearity argument.

**E04.** The proof is the calculation in CFT-01-005. Lean's
`exercise_04_solution` pairs `Matrix.charpoly_units_conj` with
`Matrix.det_units_conj`, so its statement records both polynomial and
determinant invariance instead of repeating the chapter checkpoint.

**E05.** Characteristic polynomial and trace are preserved by the two
invariant theorems used in `exercise_05_solution`. Euclidean
operator norm need not be; CFT-01-006 is a counterexample. If $S$ is unitary,
$S^{-1}=S^*$ and both $S$ and $S^{-1}$ preserve Euclidean length, so
$\lVert SAS^{-1}\rVert_2=\lVert A\rVert_2$.
For the exercise's real orthogonal case,
$(Ux)\cdot(Ux)=x\cdot(U^{\mathsf T}Ux)=x\cdot x$.
The formal solution proves this real dot-product identity; it does not
formalize the complex unitary operator-norm corollary in this exercise.

**E06.** Project the first component of the conjunction with `.1`, as in the
displayed Lean proof. The reference theorem is compiled in the chapter
exercise namespace.

## Synthesis and forward dependencies

Keep the following distinctions:

- a linear transformation exists before a basis is chosen;
- a matrix records basis-vector images as columns;
- matrix-vector multiplication is the coordinate form of linearity;
- a basis change conjugates an endomorphism;
- characteristic polynomial is a similarity invariant;
- Euclidean operator norm is invariant under unitary similarity, not arbitrary
  similarity.

Chapter 2 will supply the vector-space, span, independence, and basis theory
used here. Later chapters will return to the nonnormal example when spectra,
norms, numerical ranges, and Crouzeix bounds are all available.
