---
id: cft-chapter-02-vector-spaces-and-subspaces
title: Vector spaces and subspaces
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-07
tags: [crouzeix-textbook, linear-structure, mathematics, lean]
confidence: high
canonical: 02_vector_spaces_and_subspaces.md
chapter: 2
part: 1
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 2: Vector spaces and subspaces

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-i-linear-structure|Part I -- Linear structure]]
Previous: [[knowledge/crouzeix_textbook/part_01_linear_structure/01_objects_and_representations|Chapter 1]]
Next: [[knowledge/crouzeix_textbook/part_01_linear_structure/03_linear_maps_and_exact_structure|Chapter 3 — Linear maps and exact structure]]

## Opening problem

### Motivation

Three parameters can encode only two effective coefficients. Consider

$$
T:\mathbb R^3\longrightarrow\mathbb R^2,
\qquad T(x,y,z)=(x+z,y+z).
$$

The parameters $(1,2,0)$ and $(0,1,1)$ both give $(1,2)$. Their difference is
$(1,1,-1)$, and adding any multiple of $(-1,-1,1)$ leaves the output unchanged.
Should we keep all three parameters, choose one representative, or make a new
space whose elements already identify equivalent parameters?

A span describes the redundant directions, a direct sum chooses complementary
representatives, and a quotient records exactly what survives forgetting the
redundancy. Each construction has a different proof obligation.

**Historical context.** This chapter follows the structural language of vector
spaces, bases, and quotients used in modern linear algebra. This is a
pedagogical orientation, not a claim of historical priority. The proofs below
need no historical attribution.

## Conceptual model

Work over a field $\mathbb K$, especially $\mathbb R$ or $\mathbb C$. A vector
space $V$ has an abelian-group addition and scalar multiplication satisfying
$a(x+y)=ax+ay$, $(a+b)x=ax+bx$, $(ab)x=a(bx)$, and $1x=x$.
A subspace $W\subseteq V$ contains $0$ and is closed under addition and scalar
multiplication. These conditions also give subtraction, since
$x-y=x+(-1)y$. A nonempty subset with the two closure laws contains zero by
multiplying any member by zero; requiring zero explicitly avoids accidentally
accepting the empty set.

A span keeps everything finite linear combinations can produce. An
intersection keeps vectors satisfying every specified subspace constraint.
A direct sum separates a vector into recoverable components. A quotient
identifies vectors whose difference lies in a specified subspace.

A family $(b_i)_{i\in I}$ is independent if every finitely supported relation
$\sum_i c_i b_i=0$ has all $c_i=0$. It spans if every vector has such an
expansion. A basis has both properties. Existence comes from spanning;
uniqueness comes from independence. For infinite $I$, only finitely many
coefficients of each vector may be nonzero. No infinite series or convergence
is part of an algebraic basis.

## Formal development

All spaces in this section share a field $\mathbb K$. Each indexed statement
has a public Lean declaration. Supporting construction results are named
separately: a narrow checkpoint does not silently stand for a larger theorem.

### CFT-02-001: span minimality {#cft-02-001}

**Statement.** If $S\subseteq V$ and $W$ is a subspace containing $S$, then
$\operatorname{span}(S)\subseteq W$.

**Proof.** Construct $C(S)$ as all sums $\sum_{i=1}^n a_i s_i$ with
$s_i\in S$, $a_i\in\mathbb K$, and finite $n$. Allow $n=0$, giving zero
even when $S$ is empty. Addition concatenates the two finite lists; scalar
multiplication scales every coefficient. Thus $C(S)$ is a subspace. Each
$s\in S$ occurs as $1s$, so $C(S)$ contains $S$.

If $W$ contains $S$, every summand $a_i s_i$ lies in $W$. Induction on the
number of terms, starting with $0\in W$ and adding one term at a time, puts
the whole sum in $W$. Hence $C(S)\subseteq W$. Let $J$ be the intersection
of all subspaces containing $S$. The preceding argument gives $C(S)\subseteq J$.
Conversely $J\subseteq C(S)$ because $C(S)$ is itself one of those subspaces.
Therefore $C(S)=J$; either construction defines $\operatorname{span}(S)$,
and the stated minimality follows. $\square$

**Boundary.** The span of the empty set is $\{0\}$, not the empty set.
Spanning does not make coefficients unique: with generators $1,2$ in
$\mathbb R$, the vector $2$ is both $2\cdot1+0\cdot2$ and $0\cdot1+1\cdot2$.

**Formal correspondence.** `span_minimality` uses `Submodule.span_le`.
`span_as_intersection` records Lean's intersection definition;
`span_finite_combination` uses `Finsupp.mem_span_iff_linearCombination` to
identify it with finitely supported sums. `finite_combination_closure` checks
addition and scaling of those coefficients. The indexed theorem allows
semirings and semimodules; its field specialization is exactly the statement
above. [Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean).

### CFT-02-002: intersections {#cft-02-002}

**Statement.** For subspaces $U,W\subseteq V$ and a vector $x$,
$x\in U\cap W$ if and only if $x\in U$ and $x\in W$.

**Proof.** This is the definition of set intersection, and it also explains
why the result is a subspace. Zero lies in both. If $x,y$ lie in both, then
$x+y$ lies in each by its addition law. If $x$ lies in both, so does $ax$ by
each scalar law. For an arbitrary family $(W_i)$, perform each check for every
$i$. For an empty family there are no constraints, so its intersection is $V$.
$\square$

**Boundary.** Union expresses “at least one constraint,” which addition need
not preserve. Both $(1,0)$ and $(0,1)$ belong to the union of the coordinate
axes, but their sum $(1,1)$ does not. Exercise E05 checks this failure.

**Formal correspondence.** `mem_subspace_intersection_iff` proves both
implications locally; `intersection_closure` checks the three subspace laws.
The indexed semiring/semimodule theorem specializes to the field statement
here. [Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean).

### CFT-02-003: basis coordinates {#cft-02-003}

**Statement.** For a basis $b=(b_i)_{i\in I}$ and $x,y\in V$,
$\operatorname{repr}_b(x)=\operatorname{repr}_b(y)$ if and only if $x=y$.

**Proof.** Spanning first provides a finitely supported coefficient function
$c:I\to\mathbb K$ with $x=\sum_i c_i b_i$. If $d$ is another such function,
subtract the expansions over the finite union of their supports:
$\sum_i(c_i-d_i)b_i=0$. Independence gives $c_i=d_i$ everywhere, including
outside the supports where both values are zero. Thus each vector has exactly
one coefficient function, defining $\operatorname{repr}_b$. Reconstructing
from equal coefficients gives equal vectors; substituting equal vectors gives
equal coefficients. $\square$

Reconstruction $c\mapsto\sum_i c_i b_i$ preserves addition and scaling by
distributing finite sums. Its inverse does too: the sum or scalar multiple
of expansions is an expansion of the corresponding vector, and uniqueness
identifies its coefficients. Therefore $V\simeq\mathbb K^{(I)}$ linearly,
where the parentheses mean finite support. For finite $I$, this is the usual
coordinate space $\mathbb K^{|I|}$.

**Boundary.** An independent list need not span: $(1,0)$ cannot represent
$(0,1)$ in $\mathbb R^2$. A spanning list may be redundant. For infinite $I$,
replacing $\mathbb K^{(I)}$ by all functions $I\to\mathbb K$ would admit
coefficient lists whose sums have no algebraic meaning.

**Formal correspondence.** `basis_coordinates_unique` uses injectivity of
`b.repr`. The stronger construction is separately compiled as
`basis_expansion_exists_unique`, using `Module.Basis.linearCombination_repr`
and `Module.Basis.repr_linearCombination`. It states existence and uniqueness
of `ι →₀ 𝕜` coefficients. The indexed semiring theorem is specialized here
to a field. [Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean).

### CFT-02-004: dimension and transport {#cft-02-004}

**Statement.** If $e:V\simeq W$ is a linear equivalence, then
$\operatorname{finrank}_{\mathbb K}V=\operatorname{finrank}_{\mathbb K}W$.
For finite-dimensional spaces this says their dimensions are equal.

**Proof.** Given a basis $(b_i)$ of $V$, the family $(e(b_i))$ spans $W$:
expand $e^{-1}(w)=\sum_i c_i b_i$ and apply $e$ to obtain
$w=\sum_i c_i e(b_i)$. It is independent: applying $e^{-1}$ to a relation
$\sum_i c_i e(b_i)=0$ gives $\sum_i c_i b_i=0$, so every coefficient vanishes.
The transported basis has exactly the same index set. For finite bases,
their cardinality is the dimension, so transport preserves it.

Why is the finite count independent of the initial basis? Express $m$
independent vectors using a spanning list of length $n$. Replace one spanning
vector at a time by the next independent vector. At each step the new vector
has a nonzero coefficient on an unreplaced vector: otherwise it would lie in
the span of its independent predecessors. Solving for that spanning vector
preserves the span. After $n$ replacements no further independent vector can
be inserted, so $m\le n$. Applying this in both directions to finite bases
gives equal counts.

In Lean's general statement, `finrank` is the natural-number extraction from
cardinal dimension. An infinite-dimensional space has `finrank = 0`; that
value does not say the space is zero. A linear equivalence transports bases
in both directions, hence preserves whether dimension is finite and its
cardinal value. Both infinite-dimensional sides therefore have `finrank`
zero. $\square$

**Boundary.** Equality of general `finrank` values does not imply equivalence:
the zero space and an infinite-dimensional space both have value zero.
Interpreting `finrank` as a finite number of basis vectors requires finite
dimension.

**Formal correspondence.** `dimension_invariant_under_linear_equiv` uses
`LinearEquiv.finrank_eq` with a division ring and no finite-dimensional
typeclass. `basis_transport` uses `Module.Basis.map_apply` to record the
transported family. Real and complex spaces are field specializations.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean).

### CFT-02-005: direct-sum uniqueness {#cft-02-005}

**Statement.** Suppose $U\cap W=\{0\}$, $u_1,u_2\in U$, $w_1,w_2\in W$,
and $u_1+w_1=u_2+w_2$. Then $u_1=u_2$ and $w_1=w_2$.

**Proof.** Subtract and rearrange to get $u_1-u_2=w_2-w_1$. Closure under
subtraction puts the left side in $U$ and the right side in $W$. Their common
value lies in $U\cap W=\{0\}$; both differences are therefore zero.
Cancellation gives the two equalities. $\square$

The sum $U+W=\{u+w:u\in U,w\in W\}$ is a subspace: add or scale the
components separately. The assertion $U+W=V$ gives existence of a
decomposition for every vector. Together with zero intersection it gives
$V=U\oplus W$, an internal direct sum. Existence and uniqueness have separate
hypotheses with separate jobs.

**Boundary.** In $\mathbb R^3$, the first two coordinate axes intersect only
at zero but cannot sum to $(0,0,1)$. Conversely $U=W=V\ne\{0\}$ gives
existence but not uniqueness: $v=v+0=0+v$ when $v\ne0$.

**Formal correspondence.** `direct_sum_coordinates_unique` implements the
subtraction and intersection proof and also allows division rings.
`direct_sum_existence` separately assumes `U ⊔ W = ⊤` and extracts summands
using `Submodule.mem_sup`. The symbol `⊔` denotes subspace sum, not union.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean).

### CFT-02-006: quotient representatives {#cft-02-006}

**Statement.** For a subspace $W\subseteq V$, the projection
$q:V\to V/W$, $v\mapsto[v]$, is surjective.

**Proof.** Define $x\sim y$ when $x-y\in W$. Reflexivity holds because
$x-x=0\in W$. Symmetry follows from $y-x=-(x-y)\in W$. Transitivity follows
from $x-z=(x-y)+(y-z)\in W$. Hence equivalence classes exist; $V/W$ is their
set. Every class is $[v]$ for some $v$, precisely surjectivity of $q$. $\square$

The vector operations still need justification. Define $[x]+[y]=[x+y]$ and
$a[x]=[ax]$. If $x\sim x'$ and $y\sim y'$, then

$$
(x+y)-(x'+y')=(x-x')+(y-y')\in W,
\qquad ax-ax'=a(x-x')\in W.
$$

Changing representatives therefore changes neither output class. The
vector-space laws descend from $V$: both ways of adding three classes give
the class of $x+y+z$, and $a([x]+[y])=[a(x+y)]=[ax+ay]$; the other scalar
laws follow by the same substitution. Zero is $[0]$ and the inverse of $[x]$
is $[-x]$. The defining formulas make $q$ linear, and
$q(x)=0\iff[x]=[0]\iff x-0\in W\iff x\in W$, proving $\ker q=W$.

The universal property is stronger than surjectivity. Let $f:V\to Z$ be
linear with $W\subseteq\ker f$. Define $\bar f([x])=f(x)$. If $[x]=[y]$,
then $x-y\in W$, so $f(x)-f(y)=f(x-y)=0$. This proves well-definedness.
Moreover,

$$
\bar f([x]+[y])=f(x+y)=f(x)+f(y),\qquad
\bar f(a[x])=f(ax)=a f(x),
$$

so $\bar f$ is linear and $\bar f\circ q=f$. For any other such map $g$,
every class has a representative and
$g([x])=g(q(x))=f(x)=\bar f([x])$. Hence $g=\bar f$. Conversely a
factorization through $q$ must annihilate $W$, since $q(w)=0$ there.

**Boundary.** This quotient is by a subspace. An arbitrary subset to “forget”
need not give equivalence or well-defined operations. Even for a subspace, a
map not annihilating it cannot factor: $f(x,y)=x$ distinguishes $(0,0)$ from
$(1,1)$ although they have the same class modulo the diagonal.

**Formal correspondence.** The indexed `quotient_projection_surjective`
proves only surjectivity and permits rings/modules. Separate support is
provided by `quotient_relation_laws`, `quotient_operations_well_defined`,
`quotient_class_eq_iff`, `quotient_projection_kernel`,
`quotient_factor_well_defined`, and `quotient_factor_exists_unique`. The last
constructs a linear map using `Submodule.liftQ` and `Submodule.liftQ_mkQ`,
then proves uniqueness using representatives. Linearity is part of its `→ₗ`
type, not a consequence of surjectivity. These helpers use fields, including
$\mathbb R$ and $\mathbb C$.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean).

## Worked examples

### Choosing representatives for the running map

For $T(x,y,z)=(x+z,y+z)$, solving $T(x,y,z)=0$ gives $x=-z$, $y=-z$.
Thus $\ker T=\operatorname{span}(-1,-1,1)$. Let
$H=\{(a,b,0):a,b\in\mathbb R\}$. Every vector decomposes as

$$
(x,y,z)=(x+z,y+z,0)+z(-1,-1,1).
$$

The first term lies in $H$, the second in $\ker T$. Their intersection is
zero: the last coordinate of $a(-1,-1,1)$ is $a$. The decomposition is unique.
Choosing $H$ gives representatives; the quotient $\mathbb R^3/\ker T$
itself requires no choice of complement. The induced map sends $[(x,y,z)]$
to $(x+z,y+z)$. It is onto since $(a,b)$ comes from $(a,b,0)$, and one-to-one
since equal outputs have difference in the kernel. This directly identifies
this quotient with $\mathbb R^2$, without a later isomorphism theorem.

The kernel calculation is `redundantPredictor_kernel`; the equal-output
criterion is E06 in the [Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean).

### A diagonal quotient

For $D=\{(t,t):t\in\mathbb R\}$, $f(x,y)=x-y$ vanishes on $D$, so it
induces $\bar f([(x,y)])=x-y$. An inverse sends $r$ to $[(r,0)]$:
$\bar f([(r,0)])=r$, and $(x,y)-(x-y,0)=(y,y)\in D$ gives
$[(x-y,0)]=[(x,y)]$. Exercise E04 proves the unique linear factorization;
the inverse calculation here is an additional prose consequence.

### Invariant subspaces in the triangular family

Return to

$$
A_{\lambda,\alpha}=\begin{bmatrix}\lambda&\alpha\\0&\lambda\end{bmatrix},
\qquad A_{\lambda,\alpha}(x,y)=(\lambda x+\alpha y,\lambda y).
$$

Take real parameters here, as in the compiled example; the same algebra works
over $\mathbb C$. A subspace is invariant if $A$ keeps each of its vectors
inside it. The line $L=\operatorname{span}(1,0)$ is invariant since
$A(x,0)=(\lambda x,0)$. Put $N=A-\lambda I$. Then $N(x,y)=(\alpha y,0)$
and $N^2(x,y)=0$. If $\alpha\ne0$, the ordinary eigenspace $\ker N$ is
exactly $L$, whereas $\ker N^2=\mathbb R^2$. The latter is the generalized
eigenspace at exponent two: vectors killed by two applications of
$A-\lambda I$. This definition needs no canonical-form theory. If $\alpha=0$,
already $N=0$ and the ordinary eigenspace is the whole plane.

The line $\operatorname{span}(0,1)$ complements $L$ as a vector space, but
for $\alpha\ne0$ it is not invariant: $A(0,1)=(\alpha,\lambda)$ leaves it.
In fact there is no invariant complement. Any complementary line has a vector
with nonzero second coordinate, which can be normalized to $(t,1)$.
Invariance would require $A(t,1)=c(t,1)$. The second coordinate forces
$c=\lambda$ and the first then forces $\alpha=0$, a contradiction.

Nevertheless $A$ acts on the quotient by $L$: vectors differing in $L$ have
images differing in $L$ by invariance, so $[v]\mapsto[Av]$ is well-defined.
Identify the quotient with its second coordinate, $[(x,y)]\mapsto y$.
The induced action is $y\mapsto\lambda y$, since $(Av)_2=\lambda y$.
The off-diagonal term disappears in the quotient while remaining in the full
map. `triangularAction_structure`, `triangularAction_eigenline`, and
`triangularAction_no_invariant_complement` compile these coordinate identities
and the obstruction for every normalized candidate $(t,1)$.
[Lean source](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean).

## ML bridge

**Mathematical object.** The linear parameter-to-coefficient map
$T(x,y,z)=(x+z,y+z)$ followed by the predictor
$p_{(x,y,z)}(r,s)=(x+z)r+(y+z)s$ on all of $\mathbb R^2$.

**Exact transfer.** Two parameter vectors define the same predictor on every
input exactly when their $T$ outputs agree. Evaluate at $(1,0)$ and $(0,1)$
for the forward direction; substitute equal coefficients for the reverse.
E06 identifies this with the parameter difference being a multiple of
$(-1,-1,1)$. Quotient classes are therefore exactly predictor identities for
this specified linear model and full input domain.

**Non-transfer.** Nonlinear network symmetries need not form a fixed additive
subspace. Equality on a finite training set need not imply equality on every
input. This argument alone supplies neither claim, and it does not imply
that optimization or regularization treats all representatives identically.

**Calculation.** Both $(1,2,0)$ and $(0,1,1)$ give $r+2s$. For every real $a$,
$T((x,y,z)+a(-1,-1,1))=(x+z,y+z)$. Choosing $z=0$ replaces a parameter by
$(x+z,y+z,0)$ while preserving every prediction. Yet the squared parameter
lengths of the two displayed representatives are $5$ and $2$, so a Euclidean
parameter penalty can distinguish them.

## Lean translation

`Submodule 𝕜 V` packages membership and the three closure laws.
`Submodule.span 𝕜 S` is an intersection; the finite-combination helper
justifies the concrete construction. `U ⊓ W` is intersection, `U ⊔ W` is
subspace sum, and `⊥`, `⊤` are the zero and whole subspaces.

`Module.Basis ι 𝕜 V` supplies `b.repr : V ≃ₗ[𝕜] (ι →₀ 𝕜)`. The `→₀`
type encodes finite support even for an infinite basis index.
`Module.finrank` has its basis-count interpretation only under finite
dimension; the indexed theorem retains its general division-ring signature.

`V ⧸ W` is a quotient type and `W.mkQ` a linear projection. `W.liftQ f h`
requires `h : W ≤ LinearMap.ker f`, exactly the condition for the
representative formula to be independent of choices. The local factorization
theorem proves uniqueness by obtaining a representative of each class.

All six indexed correspondences are exact at the stated field specialization
or, for CFT-02-004, with the stated general `finrank` convention. One indexed
declaration does not check every surrounding paragraph; the supporting
declarations are named where used. Concrete exercises use real products, and
Lean's `ℝ × ℝ × ℝ` means `ℝ × (ℝ × ℝ)`.

## Exercises

Each prompt has a distinct theorem in namespace
`CrouzeixTextbook.Part01.Exercises.Chapter02` in the
[Lean solutions](../../../formalization/lean/CrouzeixTextbook/Part01/Chapter02.lean).

### CFT-02-E01 -- retrieval {#exercise-cft-02-e01}

**Prompt.** For the diagonal condition $u_1=u_2$ on $\mathbb R^2$, prove:
zero satisfies it; if $u,v$ satisfy it then $u+v$ does; and if $u$ satisfies
it then $au$ does for every real $a$.

**Solution.** Zero's coordinates are equal. Substitution gives
$u_1+v_1=u_2+v_2$; multiplication by $a$ gives $au_1=au_2$. These are the
three conjuncts of `exercise_01_solution`, which checks this concrete
subspace condition rather than assuming an arbitrary subset is closed.

### CFT-02-E02 -- calculation {#exercise-cft-02-e02}

**Prompt.** For arbitrary $x,y\in\mathbb R$, prove

$$
\frac{x+y}{2}(1,1)+\frac{x-y}{2}(1,-1)=(x,y)
$$

and use it to prove $\operatorname{span}\{(1,1),(1,-1)\}=\mathbb R^2$.

**Solution.** The coordinates are $(x+y+x-y)/2=x$ and $(x+y-x+y)/2=y$.
Each scaled generator lies in the span, and so does their sum. Every pair is
therefore in the span. Both the identity and span equality occur in
`exercise_02_solution`; no determinant result is used.

### CFT-02-E03 -- written-proof {#exercise-cft-02-e03}

**Prompt.** Prove $\operatorname{span}\{(2,0),(0,2)\}\subseteq
\operatorname{span}\{(1,1),(1,-1)\}$ by expressing the two left generators
as combinations of the right generators.

**Solution.** Put $p=(1,1)$, $m=(1,-1)$. Then $(2,0)=p+m$ and $(0,2)=p-m$.
Addition and subtraction closure put both in the right span. Minimality puts
the entire left span inside it. This is the concrete inclusion in
`exercise_03_solution`.

### CFT-02-E04 -- written-proof {#exercise-cft-02-e04}

**Prompt.** For $D=\{(t,t):t\in\mathbb R\}$, prove there exists exactly one
linear map $g:\mathbb R^2/D\to\mathbb R$ satisfying $g(q(x,y))=x-y$ for
every $(x,y)$.

**Solution.** Define $g([(x,y)])=x-y$. If two representatives differ by
$(t,t)$, their differences $x-y$ differ by $t-t=0$, proving well-definedness.
Adding or scaling representatives proves linearity. Every class is $q(x,y)$,
so the required formula determines $g$ uniquely. `exercise_04_solution`
has precisely this unique-existence statement, using the functional's
vanishing on the diagonal to construct the linear factor.

### CFT-02-E05 -- boundary {#exercise-cft-02-e05}

**Prompt.** Find $u,v\in\mathbb R^2$ with
$(u_1=0\text{ or }u_2=0)$ and $(v_1=0\text{ or }v_2=0)$, but neither
coordinate of $u+v$ zero. Thus exhibit failure of addition closure for the
union of the coordinate axes.

**Solution.** Take $u=(1,0)$, $v=(0,1)$. Each has a zero coordinate but
$u+v=(1,1)$ has none. `exercise_05_solution` supplies these witnesses and
checks the negated disjunction for their sum.

### CFT-02-E06 -- lean-proof {#exercise-cft-02-e06}

**Prompt.** For $T(x,y,z)=(x+z,y+z)$ and arbitrary $v,w\in\mathbb R^3$,
prove in Lean $T(v)=T(w)\iff\exists a\in\mathbb R,\ v-w=a(-1,-1,1)$.

**Solution.** Equal outputs give $v_1-w_1=-(v_3-w_3)$ and
$v_2-w_2=-(v_3-w_3)$; take $a=v_3-w_3$. Conversely
$T(a(-1,-1,1))=(0,0)$, so $T(v)-T(w)=T(v-w)=0$. In
`exercise_06_solution`, use `congrArg` to extract the two output equalities,
supply this witness, and prove the three coordinate identities. The reverse
direction uses `map_sub` and `sub_eq_zero`.

## Synthesis and forward dependencies

The opening redundancy is now explicit: its null directions form a span,
the plane $z=0$ supplies unique representatives through a direct sum, and the
quotient records the predictor independently of its representative. The
triangular family adds a distinction needed later: a vector-space complement
can exist without being invariant, while a quotient action still exists.

Chapter 3 develops kernels, ranges, and factorization as a calculus for linear
maps. It can use the quotient construction and universal property proved
here, while keeping existence, uniqueness, and finite-dimensional counting
as separate arguments.
