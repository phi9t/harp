---
id: cft-chapter-03-linear-maps-and-exact-structure
title: Linear maps and exact structure
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-07
tags: [crouzeix-textbook, linear-structure, mathematics, lean]
confidence: high
canonical: 03_linear_maps_and_exact_structure.md
chapter: 3
part: 1
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 3: Linear maps and exact structure

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-i-linear-structure|Part I -- Linear structure]]
Previous: [[knowledge/crouzeix_textbook/part_01_linear_structure/02_vector_spaces_and_subspaces|Chapter 2]]
Next: [[knowledge/crouzeix_textbook/part_01_linear_structure/04_coordinates_and_duality|Chapter 4 — Coordinates and duality]]

## Opening problem

The predictor from Chapter 2 has three parameters but only two effective
coefficients:
$$
T(x,y,z)=(x+z,y+z),\qquad
f_{(x,y,z)}(a,b)=(x+z)a+(y+z)b.
$$
Which parameters are invisible? Can every coefficient pair be reached? Can we
replace the parameters by their equivalence classes without losing any output?
These are three faces of one question: what does a linear map preserve?

**Motivation.** Solving one system $Tv=w$ gives a particular answer. Understanding
the kernel and range describes every possible right-hand side and every
ambiguity at once. We will extract two coordinates from this three-parameter
model, then explain why the same construction works for any linear map.

## Conceptual model

Work over $\mathbb F=\mathbb R$ or $\mathbb C$ unless stated otherwise. A map
$T:V\to W$ is linear when $T(au+bv)=aT(u)+bT(v)$. In particular $T0=0$,
$T(-v)=-Tv$, and $T(u-v)=Tu-Tv$. Its kernel collects inputs producing zero;
its range collects attainable outputs. A fiber $T^{-1}(w)$, when nonempty,
is a translate $v+\ker T$: subtract two preimages to see their difference is
in the kernel, and add a kernel vector to obtain another preimage.

The quotient $V/\ker T$ identifies exactly these fibers. It records an input's
effective identity without choosing a preferred representative. A complement
of the kernel supplies representatives, but that is an additional choice.
For an endomorphism $A:V\to V$, an invariant subspace $L$ satisfies
$A(L)\subseteq L$; it supports dynamics with both inputs and outputs in $L$.

**Historical context.** Kernel, image, quotient, and the first isomorphism
theorem are standard structural language for linear algebra. Here their
purpose is concrete: organize solution sets independently of coordinates.

## Formal development

### CFT-03-001: kernel {#cft-03-001}

First construct $K=\{x\in V:Tx=0\}$. It contains zero. If $x,y\in K$, then
$T(x+y)=Tx+Ty=0$; if $a\in\mathbb F$, then $T(ax)=aTx=0$. Thus $K$ is a
subspace, denoted $\ker T$, before any membership theorem is used.

**Statement.** For a linear map $T:V\to W$ and $x\in V$,
$x\in\ker T\iff Tx=0$.

**Proof.** Membership is exactly the defining zero-output equation. The closure
argument above justifies treating the set as a vector space.

**Boundary and Lean provider.** No finite dimension is needed. The retained
`mem_kernel_iff` allows semirings and semimodules, so it specializes to our
real or complex spaces. `kernel_range_closure` separately checks closure.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean#L9).

### CFT-03-002: range {#cft-03-002}

Construct $R=\{Tx:x\in V\}$. Zero has preimage zero. If $u=Tx$ and $v=Ty$,
then $u+v=T(x+y)$ and $au=T(ax)$. These witnesses establish subspace closure;
we write this subspace as $\operatorname{ran}T$.

**Statement.** For a linear map $T:V\to W$ and $w\in W$,
$w\in\operatorname{ran}T\iff\exists x\in V,\ Tx=w$.

**Proof.** The forward direction extracts the preimage in the definition;
the reverse direction inserts that preimage into the range.

**Boundary and Lean provider.** The range need not equal the codomain.
`mem_range_iff` allows semirings and semimodules without finite dimension;
`kernel_range_closure` checks the two subspaces separately.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean#L15).

### CFT-03-003: rank--nullity {#cft-03-003}

**Statement.** If $V$ is finite dimensional and $T:V\to W$ is linear, then
$$\dim\operatorname{ran}T+\dim\ker T=\dim V.$$
The rank is the first dimension and the nullity is the second.

**Proof.** Choose a basis $k_1,\ldots,k_r$ of the kernel. Its vectors are
independent in $V$ because an equality between them is the same equality in
the subspace. Extend this family to a basis
$k_1,\ldots,k_r,v_1,\ldots,v_s$ of $V$. In finite dimension, extension can be
performed by repeatedly adjoining a vector outside the current span; such a
vector preserves independence, and the process terminates within $\dim V$
steps. It stops only when the span is all of $V$.

We prove both requirements for $Tv_1,\ldots,Tv_s$ to be a range basis.
For spanning, take any $w\in\operatorname{ran}T$ and choose $x$ with $Tx=w$.
Expand $x=\sum_i c_i k_i+\sum_j a_jv_j$ in the extended basis. Applying $T$
annihilates each $k_i$ and gives $w=\sum_j a_jTv_j$.
For independence, suppose $\sum_j a_jTv_j=0$. Then
$u=\sum_j a_jv_j$ belongs to the kernel, so $u=\sum_i c_i k_i$ for some
coefficients $c_i$. Consequently
$\sum_i(-c_i)k_i+\sum_j a_jv_j=0$. Independence of the entire extended
basis forces every $a_j=0$ (and every $c_i=0$). The range basis has $s$
vectors, the kernel basis has $r$, and the domain basis has $r+s$, as claimed.
This includes empty bases when the kernel or range is zero.

**Boundary and Lean provider.** Only the domain must be finite dimensional;
the codomain can be infinite dimensional. `rank_nullity` retains Mathlib's
division-ring/module generality. The basis-extension step has its own
[pinned Mathlib provider, `Module.Basis.sumExtend`](https://github.com/leanprover-community/mathlib4/blob/520045ab14e26149ee970e2e617ca04b09bde5d6/Mathlib/LinearAlgebra/Basis/VectorSpace.lean#L75).
The separate local helpers `image_spanning_from_decomposition` and
`image_independence_from_kernel` check the two image-family arguments;
`rank_nullity` alone is not their provider.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean#L21).

### CFT-03-004: injectivity {#cft-03-004}

**Statement.** A linear map $T:V\to W$ is injective if and only if
$\ker T=\{0\}$.

**Proof.** If $T$ is injective and $x\in\ker T$, then $Tx=0=T0$, hence $x=0$;
zero already lies in the kernel. Conversely, suppose the kernel contains
only zero. If $Tx=Ty$, linearity gives $T(x-y)=Tx-Ty=0$. Therefore $x-y=0$,
so $x=y$. Subtraction converts equality of arbitrary outputs into a zero
output and is the essential step.

**Boundary and Lean provider.** No dimension hypothesis is needed. The
retained `injective_iff_kernel_bottom` works over division rings and additive
groups and now follows this difference argument locally. Additive inverses
are used; the proof is not asserted for arbitrary semimodules.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean#L28).

### CFT-03-005: invariant restriction {#cft-03-005}

**Statement.** (Definition.) Given a linear endomorphism $A:V\to V$, a subspace
$L\le V$, and a proof $\forall x\in L,\ Ax\in L$, define
$$A|_L:L\to L,\qquad x\longmapsto Ax.$$

**Construction and verification.** An input in $L$ is a vector together with
a membership proof. Invariance supplies the output membership proof. For
$x,y\in L$ and $a\in\mathbb F$, the equalities
$A|_L(x+y)=Ax+Ay$ and $A|_L(ax)=aAx$ follow from linearity in the ambient
space. Equality in the subspace is equality of its underlying vectors.
Thus this construction is a linear endomorphism, and forgetting the
membership proof gives exactly $Ax$.

**Boundary and Lean provider.** `invariantRestriction` remains a **definition**,
with the invariance hypothesis explicit in its input and linearity fields in
its output. It permits semirings and semimodules. The evaluation theorem is
`invariantRestriction_apply`; E06 separately proves compatibility with powers.
Without invariance one can restrict the domain to $L$, but cannot claim the
codomain is $L$.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean#L44).

### CFT-03-006: composition rank bound {#cft-03-006}

**Statement.** For linear maps $S:U\to V$ and $T:V\to W$,
$\operatorname{ran}(T\circ S)\subseteq\operatorname{ran}T$.

**Proof.** If $w\in\operatorname{ran}(T\circ S)$, choose $u$ with
$T(Su)=w$. The vector $Su\in V$ is a preimage of $w$ under $T$.

**Boundary and Lean provider.** The inclusion needs no finite dimension and
may be strict: take $S=0$ and $T=I$ on $\mathbb R$. The composite range is
$\{0\}$, whereas the final range is $\mathbb R$. In finite dimensions,
subspace dimension monotonicity gives the corresponding rank inequality;
the indexed claim is the range inclusion. `composition_range_le` retains
semiring/semimodule generality, and E05 checks the strict counterexample.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean#L52).

### First isomorphism theorem: the effective input space

**Statement.** Every linear map $T:V\to W$ induces a linear isomorphism
$$\overline T:V/\ker T\longrightarrow\operatorname{ran}T,
\qquad [x]\longmapsto Tx.$$

**Proof.** First check representatives. If $[x]=[y]$, then $x-y\in\ker T$;
hence $Tx-Ty=T(x-y)=0$. Thus the output does not depend on the representative.
It lies in the range by construction. Quotient addition and scaling give
$\overline T([x]+[y])=T(x+y)=Tx+Ty$ and
$\overline T(a[x])=T(ax)=aTx$, proving linearity. This is the factor supplied
by Chapter 2's quotient construction, with its codomain restricted to the range.

For injectivity, equal outputs of $[x]$ and $[y]$ imply
$T(x-y)=0$, hence $x-y\in\ker T$ and $[x]=[y]$.
For surjectivity, every $w\in\operatorname{ran}T$ has a preimage $x$ and
$\overline T([x])=w$. Its inverse sends $w$ to $[x]$ for any preimage $x$.
Another preimage differs by a kernel vector, so this inverse class is
independent of that choice. Both inverse identities follow immediately:
$[x]\mapsto Tx\mapsto[x]$ and $w\mapsto[x]\mapsto w$.
The inverse is linear because, if $Tx=w$ and $Ty=z$, then $x+y$ and $ax$
are preimages of $w+z$ and $aw$. No complement or preferred preimage was chosen.

**Boundary and Lean providers.** Finite dimension is unnecessary. Local
`firstIso`, `firstIso_apply`, `firstIso_inverse_class`, and
`firstIso_bijective` expose construction, evaluation, inverse classes, and
bijectivity. `firstIso_well_defined` checks the difference argument and
`firstIso_linear_operations` checks quotient addition and scaling. The definition uses
[`LinearMap.quotKerEquivRange`](https://github.com/leanprover-community/mathlib4/blob/520045ab14e26149ee970e2e617ca04b09bde5d6/Mathlib/LinearAlgebra/Isomorphisms.lean#L36),
which constructs the equivalence using the quotient lift. Its ring/module
generality includes real and complex vector spaces.
[Local Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean#L103).

## Worked examples

For our fixed map, write
$$T=\begin{bmatrix}1&0&1\\0&1&1\end{bmatrix},\qquad
n=\begin{bmatrix}-1\\-1\\1\end{bmatrix}.$$
The equations $x+z=y+z=0$ yield $(x,y,z)=zn$, so $(n)$ spans the kernel.
It is independent because the third coordinate of $an=0$ gives $a=0$.
Every $(p,q)$ has preimage $(p,q,0)$, so the range is $\mathbb R^2$.
Its basis is $e_1=(1,0),e_2=(0,1)$: $pe_1+qe_2=(p,q)$, and setting
this combination to zero gives $p=q=0$. Thus nullity is one and rank two.
These spanning and independence certificates are checked together in E02.

The first isomorphism is especially visible here:
$$[(x,y,z)]\longmapsto(x+z,y+z),\qquad
(p,q)\longmapsto[(p,q,0)].$$
Indeed $(x,y,z)-(x+z,y+z,0)=zn$ is in the kernel. The plane $z=0$
therefore supplies one convenient representative, and $T$ is injective on
that plane. E03 checks the inverse on every image class and E04 checks this
restricted injectivity. The class itself is intrinsic; the plane is a choice.

Now revisit Chapter 2's family $A_{\lambda,\alpha}$ at $\lambda=0$:
$$A_{0,\alpha}=\begin{bmatrix}0&\alpha\\0&0\end{bmatrix},\qquad
A_{0,\alpha}(x,y)=(\alpha y,0).$$
Let $L=\{(x,0):x\in\mathbb R\}$, the first-coordinate line. If $\alpha\ne0$,
$\alpha y=0$ iff $y=0$, so $\ker A_{0,\alpha}=L$. Every output lies in $L$,
and $(p,0)$ has preimage $(0,p/\alpha)$; hence its range is also $L$.
Applying the map twice gives $(0,0)$, so $A^2=0$ and $A^n=0$ for every
$n\ge2$. Here $A^0=I$ and $A^{n+1}=A\circ A^n$. The kernel and range of
each power are therefore explicit:
$$
\ker A^0=\{0\},\qquad \operatorname{ran}A^0=\mathbb R^2;
\qquad
\ker A^n=\mathbb R^2,\quad \operatorname{ran}A^n=\{0\}\quad(n\ge2).
$$
For the zeroth power, $Ix=0$ forces $x=0$, and each vector is its own preimage.
For powers at least two, every input maps to zero, so every input lies in the
kernel and zero is the only attainable output. Both statements hold for every
$\alpha$, including $\alpha=0$. The remaining power $n=1$ has kernel and
range $L$ when $\alpha\ne0$, as calculated above. Since $A(x,0)=0$,
$L$ is invariant and $A|_L=0$. Its zeroth power is the identity on $L$,
and every positive power is zero. If $\alpha=0$, the whole map is zero:
the kernel becomes the whole plane and the range becomes $\{0\}$.
`nilpotent_kernel_range`, `nilpotent_square`, `nilpotent_powers`, and
`nilpotent_restriction` check these calculations, and `nilpotent_zero_case`
checks the $\alpha=0$ boundary.
`nilpotent_power_zero_kernel_range` and `nilpotent_higher_power_kernel_range`
check the displayed power kernel/range identities in the
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean#L187).
E06 proves the general
restriction/power identity by induction. A map with some zero positive power
is called *nilpotent*; here two steps suffice.

## ML bridge

**Mathematical object.** The real linear parameter-to-coefficient map
$T(x,y,z)=(x+z,y+z)$ for the scalar predictor $f_{(x,y,z)}(a,b)$ above.

**Exact transfer.** Two parameter triples define the same predictor on every
$(a,b)\in\mathbb R^2$ exactly when their $T$ outputs agree. Equality of
predictors implies equality of coefficients by evaluating at $(1,0)$ and
$(0,1)$; the converse follows by substitution. The quotient by $\ker T$
therefore describes precisely these predictor identities.

**Non-transfer.** Equality on a restricted training set need not imply equality
of predictors on the full plane. Nonlinear network symmetries are not covered
by this linear model. An exact kernel computation also makes no numerical
conditioning claim.

**Calculation.** Moving parameters by $tn=t(-1,-1,1)$ leaves both coefficients
unchanged. Choosing $t=-z$ selects $(x+z,y+z,0)$ from the same class.
Thus three stored parameters determine two effective coefficients, with one
invisible direction. This is the computed equality $3=2+1$, not a parameter
counting heuristic.

## Lean translation

The chapter imports Chapter 2 and reuses `redundantPredictor`, `nullDirection`,
`redundantPredictor_kernel`, quotient support, and `triangularAction`.
Real triples are right-nested products `ℝ × ℝ × ℝ`; their coordinates are
`v.1`, `v.2.1`, and `v.2.2`. A range element is a vector with a proof that it
has a preimage. A quotient element is a class, so evaluating on a representative
uses `T.ker.mkQ`.

The six indexed declarations retain their public names and types.
`rank_nullity` uses `Module.Finite 𝕜 V`; without that assumption, natural-number
`finrank` is not an unrestricted infinite-dimensional counting law.
`invariantRestriction` is a definition whose output is a linear map between
subtypes. Its explicit `hW` argument supplies output membership, and its
linearity fields prove addition and scaling compatibility. General helpers
use field or ring/module hypotheses as stated in their types; all concrete
examples and exercises E01–E05 use real products. E06 uses general semimodules.
Source-level use of a Mathlib theorem is not evidence of a newly proved local
theorem body: compiler publication classifies aliases and definitions separately.

## Exercises

### CFT-03-E01 -- retrieval {#exercise-cft-03-e01}

**Prompt.** For every real triple $v$, prove $Tv=0$ iff
$v=(-v_3,-v_3,v_3)$.

**Solution.** The two output coordinates give $v_1=-v_3$ and $v_2=-v_3$.
Conversely those two equations make both outputs zero. The third coordinate
is unchanged. Checked by `Exercises.Chapter03.exercise_01_solution`.

### CFT-03-E02 -- calculation {#exercise-cft-03-e02}

**Prompt.** Give explicit kernel and range basis certificates for $T$: prove
$\ker T=\operatorname{span}\{n\}$ and $\operatorname{ran}T=\mathbb R^2$;
prove $an=0\Rightarrow a=0$, prove every $(p,q)=p(1,0)+q(0,1)$, and prove
$a(1,0)+b(0,1)=0\Rightarrow a=b=0$.

**Solution.** Solve the two kernel equations as above. For the range choose
preimage $(p,q,0)$. The third coordinate proves independence of $n$; the first
and second coordinates prove the stated expansion and independence of the
two range vectors. These are complete basis certificates. Checked by
`Exercises.Chapter03.exercise_02_solution`.

### CFT-03-E03 -- written-proof {#exercise-cft-03-e03}

**Prompt.** For every $v\in\mathbb R^3$, prove that the inverse of the
quotient-to-range isomorphism applied to $Tv$ is both $[v]$ and
$[(v_1+v_3,v_2+v_3,0)]$.

**Solution.** The inverse-class identity gives $[v]$. The difference from the
second representative is $v_3n\in\ker T$, so its class is the same. Checked
by `Exercises.Chapter03.exercise_03_solution`.

### CFT-03-E04 -- written-proof {#exercise-cft-03-e04}

**Prompt.** If real triples $v,w$ have third coordinate zero and $Tv=Tw$,
prove $v=w$.

**Solution.** The first output coordinates give $v_1=w_1$ and the second give
$v_2=w_2$, because both third coordinates vanish. Together with the given
third-coordinate equality, these prove equality of triples. Checked by
`Exercises.Chapter03.exercise_04_solution`.

### CFT-03-E05 -- boundary {#exercise-cft-03-e05}

**Prompt.** With $S=0$ and $T=I$ on $\mathbb R$, prove
$\operatorname{ran}(T\circ S)<\operatorname{ran}T$ as a strict subspace inclusion.

**Solution.** The two ranges are respectively $\{0\}$ and $\mathbb R$.
The scalar $1$ lies in the latter and not the former, so the inclusion is
strict. Checked by `Exercises.Chapter03.exercise_05_solution`.

### CFT-03-E06 -- lean-proof {#exercise-cft-03-e06}

**Prompt.** Over any semiring and semimodule, let $A:V\to V$ be linear and
$L$ invariant with proof $h_L$. Prove for every $n\in\mathbb N$ and $x\in L$
that the underlying ambient vector of $(A|_L)^n x$ equals $A^n x$.

**Solution.** Induct on $n$. At zero both sides are $x$. At the successor,
the restriction evaluates as $A$ on the underlying vector; substitute the
induction hypothesis to obtain $A(A^nx)=A^{n+1}x$. This uses invariance at
every restricted application and requires no finite dimension. Checked by
`Exercises.Chapter03.exercise_06_solution`, whose proof is this induction.

Each exercise has its own theorem with exactly the displayed hypotheses and
conclusion in the [chapter Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter03.lean).

## Synthesis and forward dependencies

Kernel and range answer what disappears and what can be reached; the first
isomorphism identifies the resulting effective input space. Rank–nullity
counts a kernel basis and an image basis obtained from one extended domain
basis. Restriction describes dynamics on an invariant subspace, while
composition can remove attainable outputs. Chapter 4 turns these constructions
into coordinate matrices and dual functionals. The running predictor and the
triangular family will keep the same maps as their representations change.
