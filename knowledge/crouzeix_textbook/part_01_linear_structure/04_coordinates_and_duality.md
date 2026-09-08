---
id: cft-chapter-04-coordinates-and-duality
title: Coordinates and duality
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-07
tags: [crouzeix-textbook, linear-structure, mathematics, lean]
confidence: high
canonical: 04_coordinates_and_duality.md
chapter: 4
part: 1
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 4: Coordinates and duality

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-i-linear-structure|Part I -- Linear structure]]
Previous: [[knowledge/crouzeix_textbook/part_01_linear_structure/03_linear_maps_and_exact_structure|Chapter 3]]
Next: [[knowledge/crouzeix_textbook/part_01_linear_structure/05_determinants_trace_and_exterior_algebra|Chapter 5 — Determinants, trace, and exterior algebra]]

## Opening problem

The feature map from Chapters 2 and 3 sends three parameters to two effective
coefficients: $T(x,y,z)=(x+z,y+z)$. Suppose an output measurement assigns
$ap+bq$ to the coefficient pair $(p,q)$. Which measurement does it induce
on the parameters? Substitution gives
$$
a(x+z)+b(y+z)=ax+by+(a+b)z.
$$
Every such measurement vanishes on $(-1,-1,1)$, the direction the predictor
cannot see. This is the dual version of the kernel calculation.

**Motivation.** A measurement on outputs becomes a measurement on inputs
without choosing lengths or angles. This operation explains why transpose
matrices appear, why two maps reverse order, and which extra choice turns a
differential into a gradient vector.

## Conceptual model

Work over $\mathbb F=\mathbb R$ or $\mathbb C$ unless otherwise stated.
The algebraic dual space $V^*=\operatorname{Hom}_{\mathbb F}(V,\mathbb F)$
consists of linear functionals, also called covectors. Its operations are
pointwise: $(\phi+\psi)(v)=\phi(v)+\psi(v)$ and $(a\phi)(v)=a\phi(v)$.
The constructions work over any commutative field. Finite dimension enters
the dual-basis and dimension arguments below.

For $T:V\to W$, define the algebraic pullback $T^\vee:W^*\to V^*$ by
$T^\vee\phi=\phi\circ T$. The star denotes the dual space; $\vee$ denotes
the pullback map. Reserve $T^\dagger$ for a metric adjoint. Over $\mathbb C$,
our functionals are complex-linear: $\phi(av)=a\phi(v)$, with no conjugation.
The conjugate-linear dual is a different convention.

**Historical context.** Dual spaces organize the familiar substitution of
variables in a linear form. We use that connection as a pedagogical route
from coordinates to intrinsic statements; no inventor or historical date
is being attributed here.

## Formal development

### CFT-04-001: pullback evaluation {#cft-04-001}

**Statement.** For a linear map $T:V\to W$, a covector $\phi\in W^*$,
and $x\in V$, $(T^\vee\phi)(x)=\phi(Tx)$.

**Proof.** The right side defines a covector because
$\phi(T(ax+by))=\phi(aTx+bTy)=a\phi(Tx)+b\phi(Ty)$.
It also depends linearly on the measurement:
$T^\vee(a\phi+b\psi)(x)=a\phi(Tx)+b\psi(Tx)$ for every $x$.
Thus $T^\vee$ is a linear map between dual spaces. The stated evaluation
equation is its definition.

**Boundary and Lean provider.** No metric or finite dimension is
needed. [`dual_map_apply`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L11)
checks evaluation; Mathlib's `LinearMap.dualMap` supplies the linearity data.
The public declaration permits a commutative semiring and semimodules,
specializing to our real or complex vector spaces.

### CFT-04-002: composition reversal {#cft-04-002}

**Statement.** For linear maps $S:U\to V$ and $T:V\to W$,
$S^\vee\circ T^\vee=(T\circ S)^\vee$ as maps $W^*\to U^*$.

**Proof.** Fix $\phi\in W^*$ and $u\in U$. Then
$$
\bigl(S^\vee(T^\vee\phi)\bigr)(u)
=(T^\vee\phi)(Su)=\phi(T(Su))
=\bigl((T\circ S)^\vee\phi\bigr)(u).
$$
Equality for every $u$ gives equality of functionals; equality for every
$\phi$ gives equality of the two linear maps. The domains force the order:
the output measurement first meets $T$, then $S$.

**Boundary and Lean provider.**
[`dual_map_composition`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L18)
uses `LinearMap.dualMap_comp_dualMap` over a commutative semiring with no
metric or dimension assumptions. The displayed proof supplies both
extensionality steps independently of that provider name.

### Dual bases and the transpose

Let $(b_i)_{i\in I}$ be a finite basis of $V$. Define $b^i(x)$ to be its
$i$th coordinate. Existence and uniqueness of coordinates make this
well-defined. Addition and scaling of expansions show it is linear, and
$b^i(b_j)=\delta_{ij}$.

These covectors are independent: evaluating $\sum_i a_i b^i=0$ at $b_j$
gives $a_j=0$. They span: if $x=\sum_i x_i b_i$, then
$$
\phi(x)=\sum_i x_i\phi(b_i)
=\left(\sum_i\phi(b_i)b^i\right)(x).
$$
Hence $\phi=\sum_i\phi(b_i)b^i$, and $(b^i)$ is the dual basis.
For an infinite basis, a functional can have infinitely many nonzero values
on basis vectors; finite combinations of coordinate functionals do not
exhaust the algebraic dual. That is why this argument uses finiteness.

**Formal correspondence.**
[`dual_basis_exists`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L50)
uses `Module.Basis.dualBasis` and `dualBasis_apply_self` for existence and
evaluation. [`dual_basis_expansion`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L55)
checks the expansion with `sum_dual_apply_smul_coord`. These named support
declarations assume a field and a finite basis.

For bases $(b_i)$ of $V$ and $(c_j)$ of $W$, define
$A_{ji}=c^j(Tb_i)$. Then $Tb_i=\sum_j A_{ji}c_j$. If
$\phi=\sum_j a_jc^j$, the $i$th dual coordinate of its pullback is
$$
(T^\vee\phi)(b_i)=\phi(Tb_i)
=\sum_j A_{ji}\phi(c_j)=\sum_j A_{ji}a_j=(A^{\mathsf T}a)_i.
$$
This derives transpose action from dual-basis evaluations. It works over
$\mathbb C$ with no conjugation because the functional is complex-linear.
[`pullback_basis_coefficients`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L102)
checks the equality for finite bases over any field.

### Annihilators and their dimension

For $U\le V$, define
$U^\circ=\{\phi\in V^*: \phi(u)=0\text{ for every }u\in U\}$.
Zero belongs to it, and any linear combination of functionals vanishing
on $U$ still vanishes there. Thus the annihilator is a subspace of $V^*$.

Choose a basis $u_1,\ldots,u_r$ of $U$ and extend it to
$u_1,\ldots,u_r,v_1,\ldots,v_s$ of $V$, using basis extension over a field.
Write the corresponding dual basis as $u^1,\ldots,u^r,v^1,\ldots,v^s$.
A covector vanishes on $U$ exactly when it vanishes on all the $u_i$:
necessity is immediate, and sufficiency follows by applying linearity to
every combination of the $u_i$.

The coefficient of $u^i$ in the dual expansion of $\phi$ is $\phi(u_i)$.
For $\phi\in U^\circ$ it is zero, leaving
$\phi=\sum_j\phi(v_j)v^j$. Conversely each $v^j$ vanishes on every $u_i$,
hence on $U$. These remaining covectors therefore span exactly $U^\circ$.
If $\sum_j a_jv^j=0$, evaluating at $v_k$ gives $a_k=0$, so they are
independent. The primal basis has $r+s$ members and this annihilator basis
has $s$, proving
$$
\dim U+\dim U^\circ=\dim V,\qquad
\dim U^\circ=\dim V-\dim U.
$$

**Boundary and Lean provider.** The argument assumes a field and
finite-dimensional $V$.
[`dualityExtendedBasis`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L60),
[`annihilator_of_basis_span`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L63),
[`annihilator_expansion`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L79),
and [`annihilator_coefficients_unique`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L89)
provide the extension construction, the vanishing criterion on the first
block, the remaining-block expansion, and its coefficient independence.
The extension construction uses `Module.Basis.sumExtend`; the other lemmas
take an extended basis indexed by a sum as input.
[`annihilator_dimension`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L97)
uses `Subspace.finrank_add_finrank_dualAnnihilator_eq` for the numerical
identity. That provider is separate from the explicit basis-extension
argument; a rank-nullity checkpoint alone would not check its spanning
and independence steps.

### CFT-04-003: coordinate conjugacy {#cft-04-003}

Fix $x_{\rm new}=Sx_{\rm old}$. Applying the operator in new coordinates
means converting back, applying the old matrix, and converting forward:
$A_{\rm new}=SA_{\rm old}S^{-1}$.

**Statement.** Let $S,R,A$ be square matrices over a commutative ring with
a finite index set. Assume $RS=I$. For every column $x$,
$(SAR)(Sx)=S(Ax)$. An invertible coordinate change takes $R=S^{-1}$;
the separate left inverse is the exact hypothesis of the public statement.

**Proof.** Associativity gives
$(SAR)(Sx)=SA(RS)x=SAIx=S(Ax)$. Both sides apply the operator to the old
column and express its result in new coordinates. Entries may change even
though the action represents the same operator.

**Boundary and Lean provider.**
[`coordinate_change_conjugacy`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L26)
retains its compatibility definition alias to Chapter 1's `change_basis_action`.
It accepts separate $S,R$ with $RS=I$, and does not require any metric or
spectral result. For covector columns the same convention gives
$a_{\rm new}=S^{-\mathsf T}a_{\rm old}$: substituting $x_{\rm new}=Sx$
in $a_{\rm new}^{\mathsf T}x_{\rm new}=a_{\rm old}^{\mathsf T}x$ proves it.

### CFT-04-004: characteristic polynomial preview {#cft-04-004}

**Statement.** For square $A$ over a commutative ring and an invertible matrix
$S$, a unit in the matrix ring,
$\chi_{SAS^{-1}}(t)=\chi_A(t)$. Here
$\chi_A(t)=\det(tI-A)$ is the characteristic polynomial in a formal
variable $t$. Over a field its roots, in a field containing them, are the
eigenvalues, with their algebraic multiplicities.

**Proof summary — forward reference.** Rewrite
$tI-SAS^{-1}=S(tI-A)S^{-1}$ in the polynomial matrix ring. Determinant
multiplicativity and cancellation of the outside unit determinants give the
identity. Chapter 5 owns the determinant machinery and Chapter 6 the
polynomial-operator development; neither is a prerequisite of the duality core.

**Boundary and Lean provider.**
[`duality_similarity_preserves_charpoly`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L30)
retains the exact alias to `similarity_preserves_charpoly`, using
`Matrix.charpoly_units_conj`. The index is finite with decidable equality
and the ring commutative. This is an exact statement with summary exposition,
not a self-contained proof of characteristic-polynomial theory.

### CFT-04-005: determinant preview {#cft-04-005}

**Statement.** For square $A$ over a commutative ring and an invertible matrix
$S$, a unit in the matrix ring,
$\det(SAS^{-1})=\det A$.

**Proof summary — forward reference.** Multiplicativity gives
$\det S\det A\det(S^{-1})=\det A$, since the outside factors multiply to
one. Chapter 5 supplies the determinant construction and multiplicativity.
Over $\mathbb R$, determinant measures oriented volume scaling; over a
field it vanishes exactly for singular matrices. These meanings are developed
there, rather than used to prove the preceding duality results.

**Boundary and Lean provider.**
[`similarity_preserves_determinant`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L33)
uses `Matrix.det_units_conj` over a commutative ring with a finite index and
decidable equality. Its proof exposition remains a summary preview.

### CFT-04-006: trace preview {#cft-04-006}

**Statement.** For square $A$ over a commutative semiring and an invertible
matrix $S$, a unit in the matrix ring, $\operatorname{tr}(SAS^{-1})=\operatorname{tr}A$, where
$\operatorname{tr}A=\sum_i A_{ii}$ is the sum of diagonal entries.

**Proof summary — forward reference.** Cyclicity gives
$\operatorname{tr}(SAS^{-1})=\operatorname{tr}(AS^{-1}S)=\operatorname{tr}A$.
Chapter 5 proves cyclicity by expanding finite sums. Trace is therefore a
scalar attached to the operator despite being computed from one basis's
diagonal entries.

**Boundary and Lean provider.**
[`similarity_preserves_trace`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L39)
uses `Matrix.trace_units_conj`. It needs only a commutative semiring and a
finite index with decidable equality. This third summary preview is outside
the self-contained duality proof path.

## Worked examples

For $T(x,y)=(x+2y,3y)$ and $\phi(p,q)=4p-q$,
$$
A=\begin{bmatrix}1&2\\0&3\end{bmatrix},\qquad
A^{\mathsf T}\begin{bmatrix}4\\-1\end{bmatrix}
=\begin{bmatrix}4\\5\end{bmatrix}.
$$
Substitution confirms $4(x+2y)-3y=4x+5y$. E02 proves equality of these
functionals at every input.

For $U=\operatorname{span}((1,1,0))\le\mathbb R^3$, a covector acts by
$ax+by+cz$. It vanishes on the generator exactly when $a+b=0$, hence
$U^\circ=\{(a,-a,c):a,c\in\mathbb R\}$ in dual coordinates. The covectors
$(1,-1,0)$ and $(0,0,1)$ span this plane and are independent by examining
the first and third coefficients. Its dimension is two, agreeing with
$1+2=3$. E04 checks the criterion for every coefficient triple.

The cumulative family acts over $\mathbb R$ or $\mathbb C$ by
$$
A_{\lambda,\alpha}=\begin{bmatrix}\lambda&\alpha\\0&\lambda\end{bmatrix},
\qquad A_{\lambda,\alpha}\begin{bmatrix}x\\y\end{bmatrix}
=\begin{bmatrix}\lambda x+\alpha y\\\lambda y\end{bmatrix}.
$$
Its pullback sends $\ell(x,y)=ax+by$ to
$a\lambda x+(a\alpha+b\lambda)y$, so its covector column becomes
$$
A_{\lambda,\alpha}^{\mathsf T}\begin{bmatrix}a\\b\end{bmatrix}
=\begin{bmatrix}\lambda a\\\alpha a+\lambda b\end{bmatrix}.
$$
This is the basis-evaluation rule above over either field. If we choose the
standard Hermitian metric $\langle u,v\rangle=\bar u^{\mathsf T}v$ over
$\mathbb C$, the adjoint matrix instead is
$\overline{A_{\lambda,\alpha}}^{\mathsf T}$. Indeed expanding
$\langle Au,v\rangle=\bar u^{\mathsf T}\bar A^{\mathsf T}v$
identifies it from the defining equation
$\langle Au,v\rangle=\langle u,A^\dagger v\rangle$.
For $\lambda=i,\alpha=1$, the algebraic-dual diagonal entries are $i$,
whereas the adjoint diagonal entries are $-i$. The chosen metric explains
the conjugation; algebraic duality itself introduces none.

## ML bridge

**Mathematical object.** A scalar loss's differential $dL_x$ is a covector:
it takes a perturbation $h$ to its first-order scalar change. A gradient is
a vector representing that covector under a chosen metric. Its coordinates
need not equal the differential's coefficient column.

**Exact transfer.** The chain rule composes derivatives as linear maps.
Pulling a scalar differential through these maps uses the reversal in
CFT-04-002. This is the linear algebra underlying reverse-mode products.
A forward derivative product applies a primal map to a tangent vector;
a reverse product pulls an output covector back to an input covector.

**Non-transfer.** The compiled claims concern linear maps and the concrete
linear loss below. They do not prove an autodifferentiation implementation
correct, prescribe complex-array API conventions, or identify parameters of
a nonlinear network merely because a linear feature map has a kernel.
No software API claim is needed here.

**Calculation.** Define $g(u,h)=2u_1h_1+u_2h_2$, with Gram matrix
$G=\begin{bmatrix}2&0\\0&1\end{bmatrix}$, and $\ell(x,y)=x+2y$.
The real pairing is symmetric and bilinear. If $v\ne0$, at least one
coordinate is nonzero, so $g(v,v)=2v_1^2+v_2^2>0$. The exact increment is
$$
\ell(x+th)-\ell(x)=t(h_1+2h_2).
$$
There is no remainder, so $d\ell_x(h)=h_1+2h_2$ at every base point.
Solve $g(q,h)=d\ell_x(h)$ for every $h$. The two coordinate perturbations
give $2q_1=1$ and $q_2=2$, hence $\nabla_g\ell=(1/2,2)$.
Conversely substitution verifies the pairing for every perturbation. This
vector differs from the differential's coefficients $(1,2)$. It is unique:
if $q,q'$ both represent the covector, their difference pairs to zero with
every $h$; choosing $h=q-q'$ and using positivity forces $q=q'$.

[`weightedMetric_symmetric`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L137),
[`weightedMetric_linear_right`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L141),
and [`weightedMetric_positive`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L146)
check the local metric properties. E05 checks positivity, the exact increment,
universal pairing, and coordinate inequality. The generic derivative chain
rule is background motivation, not an additional compiled theorem here.
For the redundant predictor, $(a,b)$ pulls back to $(a,b,a+b)$ and pairs
to zero with $(-1,-1,1)$.
[`redundant_predictor_pullback`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L161)
checks both equalities using Chapter 2's actual map and null direction.

## Lean translation

`Module.Dual 𝕜 V` denotes $V^*$ and `T.dualMap` denotes $T^\vee$.
`b.dualBasis.repr φ i` is $\phi(b_i)$; `b.coord i` extracts the $i$th
primal coordinate. `U.dualAnnihilator` denotes the vanishing subspace.
Finite sums require finite index types, and `annihilator_dimension` requires
`Module.Finite 𝕜 V`. Concrete exercises use $\mathbb R$, except E03, which
works over any field on the two-dimensional coordinate space.

The coordinate-action card and characteristic-polynomial preview retain
their original definition aliases for compatibility. Their declaration
kind does not change their mathematical statements or the exposition boundary.
The determinant and trace providers retain their different ring and semiring
hypotheses. Compiler-generated correspondence records the exact provider
classification; the explanatory links do not substitute for that receipt.

The six exercise theorems live in
`CrouzeixTextbook.Part01.Exercises.Chapter04` in the
[chapter Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean).
They establish the tasks below with distinct statements rather than aliases
to the public cards.

## Exercises

### CFT-04-E01 -- retrieval {#exercise-cft-04-e01}

Define $V^*$ without coordinates. Verify that $\phi(x,y)=2x-3y$ satisfies
$\phi(v+rw)=\phi(v)+r\phi(w)$ for all real $v,w,r$, and calculate
$T^\vee\phi(v)$ for $T(x,y)=(x+2y,3y)$.

**Solution.** The dual is the space of linear maps to the scalar field.
Expanding coordinates proves the linearity identity; substitution gives
$2(x+2y)-9y=2x-5y$.
[`exercise_01_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L170)
checks both universal identities. The definition is explanatory background,
not an extra theorem claim.

### CFT-04-E02 -- calculation {#exercise-cft-04-e02}

Prove that the pullback of the covector $(4,-1)$ under
$T(x,y)=(x+2y,3y)$ equals the covector $(4,5)$.

**Solution.** At every input the pullback gives $4(x+2y)-3y=4x+5y$.
Equality at every vector proves equality of functionals.
[`exercise_02_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L176)
states this equality of linear maps, not a sample evaluation.

### CFT-04-E03 -- written-proof {#exercise-cft-04-e03}

For a $2\times2$ matrix $A$ over any field, a functional $\phi$ on its
coordinate space, and standard basis vectors $e_j$, set $a_j=\phi(e_j)$.
Prove $(A^\vee\phi)(e_i)=(A^{\mathsf T}a)_i$ for each $i$ by expanding
$Ae_i$. Explain the absence of conjugation over $\mathbb C$.

**Solution.** Since $Ae_i=\sum_j A_{ji}e_j$, evaluation gives
$\sum_j A_{ji}a_j$, the required transpose component. Complex linearity
leaves the scalars unchanged.
[`exercise_03_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L183)
checks all matrices, functionals, and indices using the general basis theorem.
Lean uses zero-based `Fin 2` indices; displayed subscripts are one-based.

### CFT-04-E04 -- written-proof {#exercise-cft-04-e04}

For every real $(a,b,c)$, show that $\phi(x,y,z)=ax+by+cz$ annihilates
$\operatorname{span}((1,1,0))$ exactly when $a+b=0$. Describe the resulting
plane of coefficient triples.

**Solution.** Vanishing on the generator is necessary. It is sufficient
since $\phi(t(1,1,0))=t(a+b)$. The plane is $(a,-a,c)$.
[`exercise_04_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L192)
checks the equivalence for all coefficients. The dual-basis expansion explains
why every functional on $\mathbb R^3$ has such a triple.

### CFT-04-E05 -- boundary {#exercise-cft-04-e05}

For $g(u,h)=2u_1h_1+u_2h_2$ and $\ell(x,y)=x+2y$, prove that $g$ is
positive definite, that $\ell(x+th)-\ell(x)=t(h_1+2h_2)$ for every $x,h,t$,
and that $g((1/2,2),h)=h_1+2h_2$ for every $h$. Prove
$(1/2,2)\ne(1,2)$ and explain the differential/gradient distinction.

**Solution.** A nonzero coordinate makes $2v_1^2+v_2^2>0$. Expanding the
linear loss cancels its base-point terms. The metric pairing is
$2(1/2)h_1+2h_2$, exactly the differential. Finally $1/2\ne1$.
These are the four conjuncts of
[`exercise_05_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L208).
The differential has coefficients $(1,2)$; the gradient under $g$ is $(1/2,2)$.

### CFT-04-E06 -- lean-proof {#exercise-cft-04-e06}

Let $S(x,y)=(2x,x+y)$, $T(p,q)=(p+2q,3q)$, and $\phi(p,q)=4p-q$.
At every input $(x,y)$ calculate $(T\circ S)^\vee\phi$,
$S^\vee(T^\vee\phi)$, and $T^\vee(S^\vee\phi)$. Verify the reversed
order and identify the different coefficient pair produced by the other order.

**Solution.** $T(S(x,y))=(4x+2y,3x+3y)$, so the composite gives $13x+5y$.
Pulling through $T$ gives $(4,5)$, then through $S$ gives $(13,5)$.
Pulling through $S$ first gives $(7,-1)$, then through $T$ gives $(7,11)$,
hence $7x+11y$. These functionals differ, for example at $(1,0)$, although
they agree on some vectors. All three universal calculations are checked by
[`exercise_06_solution`](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter04.lean#L222).

## Synthesis and forward dependencies

A linear map sends vectors forward and measurements backward. Dual bases
express its pullback as a transpose, annihilators describe invisible directions,
and a chosen metric determines a vector representative for a covector.
Coordinate conjugacy keeps these descriptions consistent across bases.

The accepted scope is the duality core and six exercise solutions.
CFT-04-004 through CFT-04-006 retain exact statements with summary previews
whose proof development belongs to Chapters 5 and 6. This slice does not
declare the full Chapter 4 or the later metric and differentiation chapters
complete.
