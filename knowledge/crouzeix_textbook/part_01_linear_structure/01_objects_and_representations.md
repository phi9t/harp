---
id: cft-chapter-01-objects-and-representations
title: Objects and representations
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-24
tags: [crouzeix-textbook, linear-structure, mathematics, lean]
confidence: high
canonical: 01_objects_and_representations.md
chapter: 1
part: 1
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 0
---

# Chapter 1: Objects and representations

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-i-linear-structure|Part I -- Linear structure]]
Previous: Start of book
Next: [[knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces|Chapter 2]]

## Opening problem

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

### Linear transformations before coordinates {#cft-01-001}

**Definition CFT-01-001 (linear transformation).** Let $V$ and $W$ be vector
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

### Columns are images of basis vectors {#cft-01-002}

Choose an ordered basis $B=(b_1,\ldots,b_n)$ of $V$ and
$C=(c_1,\ldots,c_m)$ of $W$. Write

$$
T(b_j)=\sum_{i=1}^m a_{ij}c_i.
$$

**Theorem CFT-01-002 (column rule).** The $j$th column of the representing
matrix $A=[T]_{C\leftarrow B}$ is the coordinate vector of $T(b_j)$ in basis
$C$.

**Proof.** By definition, entry $a_{ij}$ is the coefficient of $c_i$ in the
unique basis expansion of $T(b_j)$. Collecting those coefficients down the
$j$th column gives $[T(b_j)]_C$. Nothing else is required. The theorem is
almost definitional because a matrix is constructed precisely by recording
these images. $\square$

### Coordinate action is matrix multiplication {#cft-01-003}

Let $x=\sum_j x_jb_j$. Linearity and the column rule give

$$
T(x)=\sum_j x_jT(b_j)
=\sum_i\left(\sum_j a_{ij}x_j\right)c_i.
$$

**Theorem CFT-01-003 (coordinate action).** With the bases above,

$$
[T(x)]_C=[T]_{C\leftarrow B}[x]_B.
$$

**Proof.** The coefficient of $c_i$ in the displayed expansion is
$\sum_j a_{ij}x_j$, exactly the $i$th entry of the matrix-vector product.
$\square$

This proof explains matrix multiplication rather than merely declaring a
rule. The inner sum says: scale each basis image by the input coordinate and
add the contributions.

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

**Theorem CFT-01-004 (change of basis).** For every coordinate vector,
$A'=SAS^{-1}$. Thus matrices representing the same endomorphism in different
bases are similar.

The proof has one important direction convention: $S$ sends old coordinates
to new coordinates. If a text chooses the inverse convention, the displayed
conjugation reverses. Memorizing a bare formula is therefore fragile; deriving
it from the coordinate diagram is safer.

### Similarity invariants {#cft-01-005}

**Theorem CFT-01-005 (characteristic polynomial).** If
$A'=SAS^{-1}$, then

$$
\det(tI-A')=\det(tI-A).
$$

**Proof.** Since scalar matrices commute with $S$,

$$
tI-SAS^{-1}=S(tI-A)S^{-1}.
$$

Using multiplicativity of determinant,

$$
\det\!\left(S(tI-A)S^{-1}\right)
=\det(S)\det(tI-A)\det(S^{-1})
=\det(tI-A).
$$

Hence similar matrices have the same characteristic polynomial and therefore
the same eigenvalues with algebraic multiplicity. Later chapters will prove
the determinant facts used here. At this point, treat them as an announced
dependency whose exact Lean theorem is already available from Mathlib.
$\square$

An invariant answers a specific equivalence question. “Same eigenvalues” does
not mean “same geometry.” The Euclidean norm on coordinate columns is
preserved by unitary changes of basis, not by every invertible $S$.

### A nonnormality witness {#cft-01-006}

**Worked theorem CFT-01-006.** Let

$$
S=\begin{bmatrix}1&1\\0&1\end{bmatrix},\qquad
S^{-1}=\begin{bmatrix}1&-1\\0&1\end{bmatrix},\qquad
\Lambda=\begin{bmatrix}0&0\\0&1\end{bmatrix}.
$$

Then direct multiplication gives

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
$\lVert\Lambda\rVert_2=1$. Similarity preserved the characteristic polynomial
$t(t-1)$ but changed the Euclidean operator norm. This is the smallest useful
warning that spectral data alone does not control nonnormal geometry.

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

Let $T(x,y)=(x+y,y)$ in the standard basis, so
$A=\begin{bmatrix}1&1\\0&1\end{bmatrix}$. Choose
$B'=((1,0),(1,1))$. If $P$ has the new basis vectors as columns, then standard
coordinates satisfy $[x]_{\mathrm{std}}=P[x]_{B'}$. Thus the matrix in $B'$ is
$P^{-1}AP$, not $PAP^{-1}$. Multiplying shows which direction is correct.

The apparent conflict with CFT-01-004 is only notation: there $S$ sent old
coordinates to new coordinates, whereas $P$ here sends new coordinates to old
coordinates. The diagram determines the conjugation.

## ML bridge

A feature vector, hidden state, or parameter update is usually stored as an
array, so it is tempting to treat the array as the object. The distinction in
this chapter gives three useful checks.

First, a linear layer is a map; its weight matrix is a representation after
coordinate systems for input and output have been fixed. Permuting features
or changing a linear basis conjugates an endomorphism, while changing input
and output bases separately gives a two-sided transformation.

Second, conditioning is geometric. A nonorthogonal reparameterization may
leave the represented linear operation unchanged while changing Euclidean
norms, singular values, gradient scales, and numerical stability. Similarity
does not license an operator-norm equality.

Third, the analogy has a boundary. Learned representations may lie on
nonlinear subsets; network layers may include nonlinearities; a coordinate
map may be noninvertible; finite precision introduces error. The results here
apply exactly when the stated linear and invertibility hypotheses hold.

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

The full chapter module also checks the conjugation action, Mathlib's
characteristic-polynomial invariance theorem, the explicit $2\times2$
similarity, and all six reference solutions. See the generated
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

Starting from $x=\sum_jx_jb_j$, prove
$[T(x)]_C=[T]_{C\leftarrow B}[x]_B$. Identify exactly where linearity is used.
In Lean, express the same derivation as the column expansion
`coordinateAction A x = ∑ j, x j • fun i => A i j`; expand `mulVec`
coordinatewise rather than closing the goal by definitional equality.

### CFT-01-E04 -- written proof {#exercise-cft-01-e04}

Prove that similar matrices have the same characteristic polynomial. Mark the
uses of commutation with $tI$, determinant multiplicativity, and
$\det(S^{-1})=\det(S)^{-1}$.
In Lean, package characteristic-polynomial invariance together with determinant
invariance for conjugation by a matrix unit.

### CFT-01-E05 -- boundary problem {#exercise-cft-01-e05}

Give two quantities preserved by arbitrary similarity and one quantity not
preserved by arbitrary similarity. Explain why a unitary change of basis has a
stronger norm-preservation property.
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

**E03.** Apply linearity first to move $T$ through the finite sum and then to
move each scalar outside $T$. Expanding each $T(b_j)$ and collecting the
coefficient of $c_i$ gives $\sum_ja_{ij}x_j$. Lean's
`coordinate_action_eq_mulVec` fixes the final coordinate interface.

**E04.** The proof is the calculation in CFT-01-005. Lean's
`exercise_04_solution` pairs `Matrix.charpoly_units_conj` with
`Matrix.det_units_conj`, so its statement records both polynomial and
determinant invariance instead of repeating the chapter checkpoint.

**E05.** Characteristic polynomial and eigenvalues are preserved. Euclidean
operator norm need not be; CFT-01-006 is a counterexample. If $S$ is unitary,
$S^{-1}=S^*$ and both $S$ and $S^{-1}$ preserve Euclidean length, so
$\lVert SAS^{-1}\rVert_2=\lVert A\rVert_2$.

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
