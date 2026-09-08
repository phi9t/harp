---
id: cft-chapter-05-determinants-trace-and-exterior-algebra
title: Determinants, trace, and exterior algebra
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-08
tags: [crouzeix-textbook, linear-structure, mathematics, lean]
confidence: high
canonical: 05_determinants_trace_and_exterior_algebra.md
chapter: 5
part: 1
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 5: Determinants, trace, and exterior algebra

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-i-linear-structure|Part I -- Linear structure]]
Previous: [[knowledge/crouzeix_textbook/part_01_linear_structure/04_coordinates_and_duality|Chapter 4]]
Next: [[knowledge/crouzeix_textbook/part_01_linear_structure/06_eigenvalues_and_polynomial_algebra|Chapter 6 — Eigenvalues and polynomial algebra]]

## Opening problem

Chapter 4 showed that a change of coordinates replaces the matrix $A$ by
$SAS^{-1}$, so every entry can change while the operator does not. Which
numbers computed from the entries survive that replacement? Two do, and this
chapter proves it. Write $\mathbb F$ for the scalar field and work with
$n\times n$ matrices over a commutative ring unless a stronger hypothesis is
named.

**Motivation.** A quantity attached to the entries is only a property of the
operator if it is unchanged by every invertible change of coordinates. The
determinant and the trace pass that test, and the reason they pass is not an
accident of the formulas but a structural fact: the determinant is the unique
normalized alternating multilinear function of the columns, and the trace is
the derivative of the determinant at the identity. Once that is established,
singularity, volume scaling, and first-order growth all become statements
about the operator rather than about a chosen basis.

## Conceptual model

Fix a finite index set $n$ and a commutative ring $\mathbb F$. A function
$f$ of the $n$ rows of a matrix is *multilinear* when it is linear in each row
with the remaining rows held fixed, and *alternating* when it vanishes as soon
as two rows coincide. Alternation forces antisymmetry: expanding
$f(\ldots,u+v,\ldots,u+v,\ldots)=0$ and cancelling the two vanishing diagonal
terms leaves $f(\ldots,u,\ldots,v,\ldots)=-f(\ldots,v,\ldots,u,\ldots)$.

Alternating multilinear functions of $n$ rows form a module of rank one:
fixing the value on the identity matrix pins the function down completely. The
normalized choice $f(I)=1$ is the determinant. The Leibniz sum
$$
\det A=\sum_{\sigma\in S_n}\operatorname{sgn}(\sigma)\prod_{i}A_{\sigma(i)\,i}
$$
is what that normalization computes; it is a consequence of alternation and
multilinearity, not an independent definition to be memorized.

The trace has a different character. It is linear rather than multiplicative,
and it is invariant for the weaker reason that $\operatorname{tr}(AB)$ and
$\operatorname{tr}(BA)$ are the same double sum read in two orders.

Because transposing leaves the determinant unchanged, every statement about
rows above has an identical statement about columns, and we use whichever is
more convenient.

Rectangular matrices have no determinant in this sense. The construction
consumes exactly $n$ rows of length $n$; for an $m\times n$ array with
$m\neq n$ there is no alternating $n$-linear form on the wrong number of
arguments to normalize. Trace has the same restriction as a single-matrix
invariant, although $\operatorname{tr}(AB)$ is defined for rectangular $A$ and
$B$ of transposed shapes because $AB$ is square.

**Historical context.** Determinants were computed for small systems long
before the alternating-form description existed, and the two presentations are
still taught side by side. We use the alternating-form route because it makes
the invariance proofs short and basis-free; no claim of historical priority or
of original discovery is being made by that choice.

## Formal development

### CFT-05-001: multiplicativity {#cft-05-001}

**Statement.** For square matrices $A$ and $B$ over a commutative ring with a
finite index set and decidable equality, $\det(AB)=\det A\,\det B$.

**Proof.** Fix $A$ and regard $g(B)=\det(AB)$ as a function of the columns
$b_1,\ldots,b_n$ of $B$. Column $j$ of $AB$ is $Ab_j$, so
$g(B)=\det(Ab_1,\ldots,Ab_n)$ where $\det$ is read as a function of columns.
Each $b_j\mapsto Ab_j$ is linear and $\det$ is multilinear in its columns, so
$g$ is multilinear. If $b_j=b_{j'}$ for $j\neq j'$ then $Ab_j=Ab_{j'}$, so
$AB$ has two equal columns and $g(B)=0$; hence $g$ is alternating. By rank-one
uniqueness, $g$ is a scalar multiple of $\det$, and evaluating at $B=I$
identifies the scalar as $g(I)=\det A$. Hence $\det(AB)=\det A\det B$ for
every $B$. Geometrically the two sides are the volume factor of the composite
and the product of the two separate factors.

**Boundary and Lean provider.** No field, inverse, or spectral hypothesis is
used; a commutative ring suffices.
[`determinant_multiplicative`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L10)
is the public declaration, supplied by Mathlib's `Matrix.det_mul`. The
alternating and normalization inputs quoted above are recorded separately by
[`determinant_row_additive`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L64),
[`determinant_row_homogeneous`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L69),
[`determinant_repeated_row`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L54),
and [`determinant_normalized`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L50).

### CFT-05-002: diagonal matrices {#cft-05-002}

**Statement.** For a family of scalars $d:n\to\mathbb F$ over a commutative
ring, $\det(\operatorname{diag} d)=\prod_i d_i$.

**Proof.** In the Leibniz sum a permutation $\sigma$ contributes
$\operatorname{sgn}(\sigma)\prod_i D_{\sigma(i)\,i}$, and
$D_{\sigma(i)\,i}=0$ whenever $\sigma(i)\neq i$. A single index moved by
$\sigma$ therefore kills the whole product, so only $\sigma=\mathrm{id}$
survives, with sign $1$ and product $\prod_i d_i$.

**Boundary and Lean provider.** The statement needs a finite index with
decidable equality so that `Matrix.diagonal` is defined; no invertibility is
assumed, and a zero entry correctly produces determinant zero.
[`determinant_diagonal`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L16)
uses `Matrix.det_diagonal`. The Leibniz expansion the proof reads is
[`determinant_leibniz`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L74).

### CFT-05-003: similarity invariance {#cft-05-003}

**Statement.** For a unit $S$ of the matrix ring and any square $A$ over a
commutative ring, $\det\bigl(SAS^{-1}\bigr)=\det A$.

**Proof.** Apply CFT-05-001 twice:
$\det(SAS^{-1})=\det S\,\det A\,\det(S^{-1})$. Applying it once more to
$SS^{-1}=I$ together with the normalization $\det I=1$ gives
$\det S\,\det(S^{-1})=1$. Since the ring is commutative the two outside
factors can be brought together and cancelled, leaving $\det A$. The
determinant is therefore an invariant of the operator, and the vanishing of
that single scalar detects singularity independently of the coordinates.

**Boundary and Lean provider.** The hypothesis is that $S$ is a unit of the
matrix ring, not merely a matrix with nonzero determinant over an arbitrary
ring; over a field the two conditions agree.
[`determinant_similarity`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L22)
uses `Matrix.det_units_conj`. The separate one-sided form with an explicit
left inverse is proved as E03. This card supplies the derivation that
[[knowledge/crouzeix_textbook/part_01_linear_structure/04_coordinates_and_duality#cft-04-005|CFT-04-005]]
previewed.

### Multilinearity, alternation, and the invertibility criterion

The three support declarations quoted above state exactly the properties used
in CFT-05-001. Additivity in one row is
$\det(A\text{ with row }j\text{ replaced by }u+v)$ equal to the sum of the two
replaced determinants; homogeneity scales such a determinant by $s$;
alternation is the vanishing on a repeated row. Antisymmetry follows and is
recorded as
[`determinant_permuted`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L59):
permuting the rows by $\sigma$ multiplies the determinant by
$\operatorname{sgn}(\sigma)$.

Because
[`determinant_transpose`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L79)
gives $\det A^{\mathsf T}=\det A$, every row statement above has an identical
column statement, which is why the geometric reading in terms of column
vectors and the computational reading in terms of rows agree.

Invertibility is detected by the same scalar. Over a commutative ring,
$A$ is a unit exactly when $\det A$ is a unit; this is
[`determinant_unit_criterion`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L83),
supplied by `Matrix.isUnit_iff_isUnit_det`. Over a field the units are the
nonzero scalars, so the criterion becomes $\det A\neq0$, recorded as
[`determinant_field_criterion`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L88).
The distinction matters: over $\mathbb Z$ the matrix $2I$ has nonzero
determinant and no inverse.

### CFT-05-004: cyclicity of the trace {#cft-05-004}

**Statement.** For $A$ of shape $m\times n$ and $B$ of shape $n\times m$ over
a commutative scalar structure, $\operatorname{tr}(AB)=\operatorname{tr}(BA)$.

**Proof.** Both traces are the same double sum read in two orders. Expanding
the diagonal of the product,
$$
\operatorname{tr}(AB)=\sum_{i}\sum_{j}A_{ij}B_{ji},\qquad
\operatorname{tr}(BA)=\sum_{j}\sum_{i}B_{ji}A_{ij}.
$$
Interchanging the two finite sums and commuting each scalar product turns the
first expression into the second. Nothing beyond finiteness of both index sets
and commutativity of scalar multiplication is used, and in particular $AB$ and
$BA$ need not have the same size.

**Boundary and Lean provider.** The hypotheses are additive commutativity and
a commutative multiplication on the scalars, with both index sets finite; no
ring inverse, determinant, or square shape is required.
[`trace_cyclic`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L28)
uses `Matrix.trace_mul_comm`. The explicit double sums displayed here are the
content of E04.

### CFT-05-005: similarity invariance of the trace {#cft-05-005}

**Statement.** For a unit $S$ of the matrix ring and any square $A$ over a
commutative semiring, $\operatorname{tr}\bigl(SAS^{-1}\bigr)=\operatorname{tr}A$.

**Proof.** Group the product as $S\cdot(AS^{-1})$ and apply CFT-05-004:
$$
\operatorname{tr}\bigl(S(AS^{-1})\bigr)=\operatorname{tr}\bigl((AS^{-1})S\bigr)
=\operatorname{tr}\bigl(A(S^{-1}S)\bigr)=\operatorname{tr}A .
$$
Only cyclicity is used, never a claim that $\operatorname{tr}(XY)$ equals
$\operatorname{tr}(YX)$ for a rearrangement of three or more factors in
arbitrary order. That stronger claim is false, as E05 exhibits.

**Boundary and Lean provider.** A commutative semiring suffices, which is
weaker than the commutative ring needed for the determinant card, because no
subtraction or sign enters.
[`trace_similarity`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L34)
uses `Matrix.trace_units_conj`. E06 reproduces the same conclusion from an
explicit left inverse. This card supplies the derivation that
[[knowledge/crouzeix_textbook/part_01_linear_structure/04_coordinates_and_duality#cft-04-006|CFT-04-006]]
previewed.

### CFT-05-006: the diagonal trace {#cft-05-006}

**Statement.** For a family of scalars $d:n\to\mathbb F$ over an additive
commutative monoid, $\operatorname{tr}(\operatorname{diag} d)=\sum_i d_i$.

**Proof.** The $i$th diagonal entry of $\operatorname{diag} d$ is $d_i$ and
every off-diagonal entry is zero, so the defining sum of diagonal entries is
$\sum_i d_i$ term by term.

**Boundary and Lean provider.** Only an additive commutative monoid and a
finite index with decidable equality are needed; no multiplication is used at
all, which is why this card has the weakest hypotheses in the chapter.
[`trace_diagonal`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L40)
uses `Matrix.trace_diagonal`. Combined with CFT-05-002, a diagonalizable
operator has determinant the product and trace the sum of its diagonal
entries; Chapter 6 supplies the diagonalization and the eigenvalue reading,
and this chapter does not assume that every matrix admits one.

### Top-degree exterior action

The rank-one fact quoted in the conceptual model can be stated without
choosing coordinates. Let $M$ be a module with a finite basis $e$ indexed by
$\iota$, let $f$ be any alternating $\iota$-form on $M$ with scalar values,
and let $T$ be a linear operator on $M$. Then for every family $v$ of inputs
$$
f(T\circ v)=\det(T)\cdot f(v).
$$
The proof is short once uniqueness is available: writing $f=f(e)\cdot e^{\det}$
for the basis determinant form $e^{\det}$ reduces the claim to the single case
$f=e^{\det}$, where it is the definition of the operator determinant. This is
[`alternating_form_scaled_by_determinant`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L102).
Reading $\bigwedge^{n}M$ as the target of the universal alternating form, the
identity says that $T$ acts on the top exterior power by the scalar $\det T$.
The statement above is the compiled form; the exterior-power packaging is
explanatory and is not separately formalized here.

## Worked examples

Take
$$
A=\begin{bmatrix}1&2\\3&4\end{bmatrix},\qquad
S=\begin{bmatrix}1&1\\0&1\end{bmatrix},\qquad
S^{-1}=\begin{bmatrix}1&-1\\0&1\end{bmatrix}.
$$
Then $\det A=1\cdot4-2\cdot3=-2$ and $\operatorname{tr}A=1+4=5$. Conjugating,
$$
SAS^{-1}=\begin{bmatrix}4&2\\3&1\end{bmatrix},
$$
whose determinant is $4-6=-2$ and whose trace is $4+1=5$. Every entry moved;
the two scalars did not.

The first-order expansion is exact in dimension two. For any $t$ and any
$2\times2$ matrix $A$,
$$
\det(I+tA)=1+t\operatorname{tr}A+t^{2}\det A,
$$
which is
[`determinant_one_add_smul_fin_two`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L130).
The coefficient of $t$ is the trace, which is the precise sense in which trace
is the derivative of volume at the identity. In dimension $n$ the same
expansion has $n+1$ terms and the linear coefficient is still the trace.

The cumulative family acts over $\mathbb R$ by
$$
A_{\lambda,\alpha}=\begin{bmatrix}\lambda&\alpha\\0&\lambda\end{bmatrix},
\qquad
\det A_{\lambda,\alpha}=\lambda^{2},\qquad
\operatorname{tr}A_{\lambda,\alpha}=2\lambda .
$$
These are
[`runningFamily_det`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L140)
and
[`runningFamily_trace`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L145).
Neither scalar mentions $\alpha$, so the whole family shares one determinant
and one trace, recorded as
[`runningFamily_scalars_ignore_shear`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L150).
Chapter 1 showed that a diagonal change of basis rescales $\alpha$ freely,
so this blindness is exactly what invariance predicts. It is also a warning:
Chapter 9 will show that $\|A_{\lambda,\alpha}\|$ grows with $\alpha$, so the
two invariants of this chapter do not control the size of the operator.

## ML bridge

**Mathematical object.** The log-determinant $\log|\det J|$ of a Jacobian is
the exact change-of-variables factor for a density under an invertible smooth
map, and the trace is the linear term of that factor near the identity. Both
are scalars attached to a linear map, so both are unchanged when the same map
is written in another basis.

**Exact transfer.** CFT-05-001 is what makes a composition of invertible
layers accumulate log-determinants additively:
$\log|\det(J_kJ_{k-1}\cdots J_1)|=\sum_i\log|\det J_i|$ follows from
multiplicativity and the logarithm of a product. CFT-05-004 is what makes a
trace independent of the order in which a low-rank factorization is applied,
so $\operatorname{tr}(UV^{\mathsf T})$ may be evaluated in whichever of the two
shapes is smaller.

**Non-transfer.** These identities are theorems about matrices over an exact
scalar structure. They do not establish that a floating-point evaluation of
$\det$ or $\log|\det|$ is numerically stable, that a stochastic trace
estimator is unbiased at a given sample count, or that an architecture with an
easy log-determinant is expressive. A determinant near zero is exactly the
regime where a computed inverse is unreliable, and nothing here bounds that
error.

**Calculation.** For $A_{\lambda,\alpha}$ with $\lambda=1$ and $\alpha=10$,
$\det=1$ and $\operatorname{tr}=2$, both identical to the values at
$\alpha=0$. A density pushed through this map is therefore unchanged in total
volume, even though the map stretches the vector $(0,1)$ to $(10,1)$. Volume
preservation and distortion are different statements, and only the first is
what the determinant reports.

## Lean translation

Mathlib writes `Matrix.det` and `Matrix.trace`, and `Matrix.diagonal d` for
the diagonal matrix of a family `d`. The public cards of this chapter carry
the hypotheses their proofs need and no more: `CommRing` for the three
determinant cards, `CommSemiring` for trace similarity, and an additive
commutative monoid for the diagonal trace. `Matrix.updateRow` expresses the
one-row replacements used to state multilinearity, and `Equiv.Perm.sign` gives
the antisymmetry constant.

`LinearMap.det` is the coordinate-free operator determinant, and
`Module.Basis.det` is the normalized alternating form attached to a basis.
The alternating-form statement of the last development section is proved from
`AlternatingMap.eq_smul_basis_det` and `Module.Basis.det_comp`; those two
Mathlib results are the uniqueness and the transformation rule respectively.

The six exercise theorems live in
`CrouzeixTextbook.Part01.Exercises.Chapter05` in the
[chapter Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean).
Each has its own statement rather than an alias to a public card: the two
similarity exercises assume a one-sided inverse $RS=I$ instead of a matrix-ring
unit, and the remaining four fix concrete small matrices.

## Exercises

### CFT-05-E01 -- retrieval {#exercise-cft-05-e01}

Explain why an alternating function vanishes on repeated inputs, then check
both halves concretely: show that the real matrix with two equal rows
$(a,b)$ has determinant zero, and that exchanging the two rows of a
$2\times2$ matrix negates its determinant.

**Solution.** Alternation is the defining hypothesis, and antisymmetry is its
consequence: expanding on the sum of the two arguments and cancelling the two
vanishing terms leaves a sign change. Concretely $ab-ba=0$ for the repeated
row, while
$$
\det\begin{bmatrix}a&b\\1&2\end{bmatrix}=2a-b,\qquad
\det\begin{bmatrix}1&2\\a&b\end{bmatrix}=b-2a,
$$
which are negatives of each other for every $a$ and $b$.
[`exercise_01_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L161)
checks both identities for all real $a$ and $b$.

### CFT-05-E02 -- calculation {#exercise-cft-05-e02}

Compute the determinant and the trace of the upper-triangular matrix
$$
T=\begin{bmatrix}2&5&-1\\0&3&4\\0&0&-2\end{bmatrix}.
$$

**Solution.** Only the identity permutation contributes a nonzero product to
the Leibniz sum for a triangular matrix, so $\det T=2\cdot3\cdot(-2)=-12$, and
the trace is $2+3-2=3$.
[`exercise_02_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L172)
states both numbers for that explicit matrix. Triangular matrices behave like
diagonal ones for these two scalars, which is why CFT-05-002 and CFT-05-006
already predict the answers.

### CFT-05-E03 -- written-proof {#exercise-cft-05-e03}

Let $S$, $R$, and $A$ be square matrices over a commutative ring and assume
only the one-sided identity $RS=I$. Prove $\det(SAR)=\det A$ without assuming
that $S$ is a unit of the matrix ring.

**Solution.** Multiplicativity gives $\det(SAR)=\det S\,\det A\,\det R$, and
applying it to the hypothesis gives $\det R\,\det S=\det I=1$. Commutativity
of the scalars lets the two outside determinants be collected and cancelled.
[`exercise_03_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L181)
records the general statement. The exercise is strictly weaker in hypothesis
than CFT-05-003 and shows that a left inverse already suffices.

### CFT-05-E04 -- written-proof {#exercise-cft-05-e04}

For $A$ of shape $m\times n$ and $B$ of shape $n\times m$, write both
$\operatorname{tr}(AB)$ and $\operatorname{tr}(BA)$ as explicit double sums of
entries and derive cyclicity by interchanging the two finite sums.

**Solution.** Expanding the diagonal of $AB$ gives $\sum_i\sum_j A_{ij}B_{ji}$
and expanding the diagonal of $BA$ gives $\sum_j\sum_i B_{ji}A_{ij}$.
Interchanging the order of the two finite sums and commuting each product
identifies them.
[`exercise_04_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L193)
states the two expansions and the resulting equality as three conjuncts, so
the interchange step is visible in the checked statement rather than hidden
inside a library lemma.

### CFT-05-E05 -- boundary {#exercise-cft-05-e05}

Show that $\operatorname{tr}(ABC)$ is invariant under cyclic rotation of the
three factors, and exhibit three concrete matrices for which
$\operatorname{tr}(ABC)\neq\operatorname{tr}(ACB)$.

**Solution.** Rotation follows from CFT-05-004 applied to the grouping
$A\cdot(BC)$, and once more to $B\cdot(CA)$. For the counterexample take the
matrix units
$$
X=\begin{bmatrix}1&0\\0&0\end{bmatrix},\quad
Y=\begin{bmatrix}0&1\\0&0\end{bmatrix},\quad
Z=\begin{bmatrix}0&0\\1&0\end{bmatrix}.
$$
Then $XYZ=X$ has trace $1$ while $XZY=0$ has trace $0$.
[`exercise_05_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L207)
proves the two rotations for arbitrary square matrices and supplies that
counterexample, so cyclicity and full permutation invariance are separated in
one checked statement.

### CFT-05-E06 -- lean-proof {#exercise-cft-05-e06}

Reproduce the trace similarity result in Lean from cyclicity alone, again
assuming only $RS=I$ rather than that $S$ is a unit.

**Solution.** Cyclicity moves $R$ to the front of $S A R$, associativity
regroups the product as $(RS)A$, and the hypothesis collapses it to $A$.
[`exercise_06_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter05.lean#L219)
is that three-step calculation. Comparing it with E03 shows the same
one-sided hypothesis serving two different invariants, one multiplicative and
one additive.

## Synthesis and forward dependencies

The determinant is the unique normalized alternating multilinear function of
the rows; multiplicativity, the diagonal formula, and similarity invariance
all follow from that characterization, and the same scalar decides
invertibility. The trace is linear and cyclic, invariant for a weaker reason,
and blind to exactly the shear parameter that will later drive the operator
norm.

This chapter completes the scalar-invariant previews recorded in Chapter 4.
Chapter 6 uses the determinant to build the characteristic polynomial and
turns polynomials into operators; the invertibility criterion proved here is
what makes $\det(\lambda I-A)=0$ the right test for an eigenvalue.
