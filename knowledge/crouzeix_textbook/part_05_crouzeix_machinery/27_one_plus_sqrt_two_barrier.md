---
id: cft-chapter-27-one-plus-sqrt-two-barrier
title: The one-plus-square-root-two barrier
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-28
tags: [crouzeix-textbook, crouzeix-machinery, mathematics, lean]
confidence: high
canonical: 27_one_plus_sqrt_two_barrier.md
chapter: 27
part: 5
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 27: The one-plus-square-root-two barrier

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-v-crouzeix-machinery|Part V — Crouzeix machinery]]
Previous: [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/26_double_layer_map|Chapter 26 — The double-layer map]]
Next: [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/28_complete_power_family|Chapter 28 — The complete power family]]

## Opening problem

Chapter 26 built a positive double-layer map. Why did its classical one-step use
stop at `1+√2≈2.414` instead of the sharp constant `2`? The analytic argument
produces coupled operators, but a triangle inequality replaces them by two
independent lengths. Solving the resulting quadratic is exact; the relaxation
that produced it is not.

Write `T` for the target, `S` for the positive-map term, and `H` for a controlled
companion product. The one-step algebra has the form
$$T^*T=S^*T-H,\qquad \|S\|≤2,\qquad \|H\|≤1.$$
For `κ=‖T‖`,
$$κ^2=\|T^*T\|=\|S^*T-H\|≤\|S\|\,\|T\|+\|H\|≤2κ+1.$$
This chapter audits every step, including the correlation discarded by the
triangle inequality.

The running nonnormal comparison is
$$A_{0,2}=\begin{pmatrix}0&2\\0&0\end{pmatrix}.$$
Write `J=A_{0,2}/2`. Its numerical range has radius `1/2` and its norm is one;
scaling by two gives the unit-disk numerical range and norm two for `A_{0,2}`.
For `p(z)=z`, either normalization has ratio `2`, while the independent-term
proof certifies only `1+√2`. This is method slack, not an attainment claim for
the larger constant. Lean checks the normalized core in
[`jordanNilpotentTwo_attains_two`](../../../formalization/lean/CrouzeixConjecture/Sharpness.lean#L34),
whose canonical receipt is `CrouzeixTextbook:454af67489b07bf9f313151152547157983ebf25dfdb60014fe289a908d9641b`
with axioms `Classical.choice, Quot.sound, propext`.

## Conceptual model

The proof has two layers. The scalar layer solves `κ²≤2κ+1` exactly by
factoring at the roots `1±√2` and auditing their signs. The operator layer is
where information is lost: it replaces a coupled decomposition by separate
norm bounds and then applies the triangle inequality. Thus `1+√2` is the exact
endpoint of an inexact relaxation. To improve the constant, a later method
must retain joint information—such as a signed cross term, positivity across a
whole power family, or a dilation—past the point where this argument discards
it.

## Formal development

### CFT-27-001 — the nonnegative square-root branch {#cft-27-001}

#### Purpose

Fix which solution of `s²=2` is denoted by `√2`; every later factor-sign audit
depends on this branch information rather than on the square equation alone.

#### Definitions and notation

For `r≥0`, `Real.sqrt r` is the unique nonnegative real whose square is `r`.
We write `s=Real.sqrt 2` and deliberately keep its sign and square as two facts.

#### Statement

The real number selected by the square-root operation at two satisfies
$$0≤\sqrt2.$$

#### Hypothesis ledger

There are no variable hypotheses. The conclusion is the order half of the
real square-root specification, specialized to the nonnegative input two.

#### Proof roadmap

Invoke the nonnegative-output theorem directly; do not infer the sign from
`s²=2`, since that equation by itself also permits the negative root.

#### Proof

We establish the nonnegative real square-root branch. Specializing the general
square-root order theorem at two gives the load-bearing fact
$$0≤2\quad\Longrightarrow\quad0≤\operatorname{Real.sqrt}(2).$$

#### Worked instance

The equation `s²=9` admits `±3`, whereas the real-square-root function selects
`3`. The same selection principle chooses `+√2` without decimal arithmetic.

#### Boundary case

At input zero the positive and negative algebraic branches coincide. This is
why nonnegativity holds at zero although strict positivity requires positive input.

#### Historical context

Source boundary: this is standard ordered-real square-root theory from the pinned Mathlib toolchain, not a historical Crouzeix claim.

Review status: the branch choice and its later use in the factor-sign audit were checked against the compiled declaration.

#### ML analogy

Mathematical object: a canonical nonnegative branch selected from two algebraic roots.

ML counterpart: a nonnegative scale represented through its square, such as a standard deviation.

Exact transfer: a declared square-root parameterization permits sign-sensitive inequalities using `s≥0`.

Non-transfer: an unconstrained learned parameter is not nonnegative merely because its square enters a loss.

Diagnostic: inspect the parameterization or constraint instead of inferring a sign from squared observations.

#### Pedagogical prerequisites

Ordered real numbers, squares, and the distinction between an equation's
solution set and a function that canonically selects one solution.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.sqrt_two_nonnegative`.
Formal mode: `proved-here`.
Substantive provider: `none`.
Readable type map: the closed proposition says exactly that the real square root of two is nonnegative.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L12).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `LE.le.{0} (OfNat.ofNat.{0} 0) (Real.sqrt (OfNat.ofNat.{0} 2))`.
Type SHA-256: `0671f6cc626c2d1ec60ba1496a2a0bcfcea215f114a3712a2e6ad0333943c82b`.
Direct maintained dependencies: `none`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: none beyond the ordered-real square-root specification.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:0671f6cc626c2d1ec60ba1496a2a0bcfcea215f114a3712a2e6ad0333943c82b`.

#### Exercises and solutions

CFT-27-E01 reconstructs the endpoint estimate without calling its public
barrier theorem and distinguishes norm context from scalar necessity.

### CFT-27-002 — the square-root equation at two {#cft-27-002}

#### Purpose

Record the algebraic half of the square-root specification so the barrier's
factorization is symbolic and exact rather than a rounded numerical argument.

#### Definitions and notation

Continue to write `s=√2`. The expression `s²` is ordinary real multiplication,
not a norm square, a matrix square, or a complex modulus.

#### Statement

The selected root obeys the exact identity
$$ (\sqrt2)^2=2.$$
This is equality in the real numbers, not a decimal approximation and not an
unspecified choice between the two roots of `t²=2`.

#### Hypothesis ledger

The general theorem requires the input to be nonnegative. The numerical side
condition `0≤2` is discharged explicitly and leaves no variable assumptions.

#### Proof roadmap

Apply the theorem stating that the square of the real square root returns its
input, and provide the elementary proof that two is nonnegative.

#### Proof

We prove the square-root specification at two. The general rule
`0≤r → (√r)²=r`, specialized at `r=2`, gives
$$0≤2\quad\Longrightarrow\quad(\operatorname{Real.sqrt}(2))^2=2.$$

#### Worked instance

Squaring `1.4142` yields only an approximation; the symbolic identity remains
exact and therefore supports contradiction arguments with strict signs.

#### Boundary case

For negative inputs the total real square-root function does not satisfy this
equation. The nonnegative-input side condition is mathematically load-bearing.

#### Historical context

Source boundary: the theorem is standard real analysis supplied by the pinned Mathlib square-root API.

Review status: the numerical side condition, normalized proposition, dependencies, and axioms were compiler checked.

#### ML analogy

Mathematical object: an exact invariant relating a nonnegative parameter to its square.

ML counterpart: a positive scale reparameterized by a square root while downstream code consumes a variance.

Exact transfer: symbolic reasoning may replace `s²` by two when `s` is definitionally `Real.sqrt 2`.

Non-transfer: floating-point `sqrt` does not preserve the identity bit-for-bit under every arithmetic schedule.

Diagnostic: separate symbolic proof obligations from numerical tolerance tests in experiments.

#### Pedagogical prerequisites

Real square roots, exponent notation, and specialization of a theorem carrying
an explicit order side condition.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.sqrt_two_squared`.
Formal mode: `proved-here`.
Substantive provider: `none`.
Readable type map: the proposition identifies the square of `Real.sqrt 2` with the real number two.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L16).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `Eq.{1} (HPow.hPow.{0, 0, 0} (Real.sqrt (OfNat.ofNat.{0} 2)) (OfNat.ofNat.{0} 2)) (OfNat.ofNat.{0} 2)`.
Type SHA-256: `807b03a281f82bdc472d967959ea405a451a428770ceb42484d1cd5bd3c5c85c`.
Direct maintained dependencies: `none`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: the discharged numerical fact that two is nonnegative.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:807b03a281f82bdc472d967959ea405a451a428770ceb42484d1cd5bd3c5c85c`.

#### Exercises and solutions

CFT-27-E02 uses exact squaring and sign checks to place `1+√2` strictly
between two and five halves.

### CFT-27-003 — positivity and the excluded negative root {#cft-27-003}

#### Purpose

Show that the upper endpoint is positive, make the negative branch exclusion
visible, and record irrationality as a supporting fact rather than a premise.

#### Definitions and notation

The equation `t²=2` has roots `±√2`; the function `Real.sqrt` chooses the
nonnegative one. Irrationality is a different arithmetic property of that root.

#### Statement

The endpoint used by the barrier proof is strictly positive:
$$0<1+\sqrt2.$$
Equivalently, the nonnegative branch places the chosen endpoint strictly to
the right of zero; this sign is what later distinguishes it from `1-√2`.

#### Hypothesis ledger

Only `0≤√2` and `0<1` are required. The checked irrationality helper is useful
context, but neither positivity nor the later factor argument assumes it.

#### Proof roadmap

Add the nonnegative square root to one. Then separately identify the negative
root and recall the parity contradiction establishing irrationality.

#### Proof

We exclude the negative root with the sign branch visible. Since `0≤√2`,
$$1≤1+\sqrt 2,$$
so `0<1+√2`. The root `-√2` is nonpositive and is not selected. A nearby Lean
helper records `Irrational (√2)`; the familiar proof makes numerator and
denominator both even in a supposed lowest-terms representation.

#### Worked instance

The roots of `t²-2t-1` are `1±√2`; one is negative and one positive. The
barrier is the right endpoint, not merely a root chosen by notation.

#### Boundary case

Knowing only `(√2)²=2` cannot distinguish the two signs. Every proof that
uses a factor sign must cite branch information explicitly.

#### Historical context

Source boundary: irrationality of `√2` is classical; Lean uses Mathlib's standard theorem rather than introducing new number theory.

Review status: positivity, branch exclusion, and the supporting irrationality declaration were checked independently.

#### ML analogy

Mathematical object: branch data turning a polynomial equation into an ordered conclusion.

ML counterpart: an identifiability constraint selecting one representative among sign-symmetric parameters.

Exact transfer: enforced nonnegative parameterization permits comparisons unavailable from squared output alone.

Non-transfer: optimization dynamics need not respect an intended branch unless the model enforces it.

Diagnostic: test whether sign-flipped parameters represent the same model before interpreting a recovered sign.

#### Pedagogical prerequisites

Order compatibility with addition, quadratic roots, and the elementary
lowest-terms proof that the square root of two is irrational.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.one_plus_sqrt_two_positive`.
Formal mode: `proved-here`.
Substantive provider: `none`.
Readable type map: the public theorem proves positivity; a nearby checked helper records irrationality separately.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L20).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `LT.lt.{0} (OfNat.ofNat.{0} 0) (HAdd.hAdd.{0, 0, 0} (OfNat.ofNat.{0} 1) (Real.sqrt (OfNat.ofNat.{0} 2)))`.
Type SHA-256: `b5d6ab35facd3ff402611c5653d77b68002e38c6a2303ed396fd26262b35ec94`.
Direct maintained dependencies: `none`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: no variable assumptions; positivity follows from the selected branch.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:b5d6ab35facd3ff402611c5653d77b68002e38c6a2303ed396fd26262b35ec94`.

#### Exercises and solutions

CFT-27-E03 reconstructs the upper endpoint by completing the square while
keeping the square-root order step explicit.

### CFT-27-004 — the scalar quadratic barrier {#cft-27-004}

#### Purpose

Solve the exact scalar inequality with every factor sign audited and without
carrying the unnecessary application-specific assumption `κ≥0`.

#### Definitions and notation

Put `s=√2`. The quadratic factors as
$$κ^2-2κ-1=(κ-1-\sqrt 2)(κ-1+\sqrt 2).$$
Both factors are real scalars. Their product sign, rather than a division by
either factor, will decide which side of the upper root can contain `κ`.

#### Statement

For every real `κ`,
$$κ²≤2κ+1\quad\Longrightarrow\quadκ≤1+\sqrt 2.$$
In particular, the scalar upper bound does not require `κ≥0`.

#### Hypothesis ledger

The only variable hypothesis is the quadratic inequality. We use `s²=2` to
factor and `s≥0` to compare factors; norm nonnegativity is not used here.

#### Proof roadmap

Derive a nonpositive product. If `κ` crossed the upper root, both factors would
be positive, contradicting the product sign.

#### Proof

We factor the scalar quadratic and audit both factor signs. From the hypothesis,
$$
(κ-1-s)(κ-1+s)=(κ-1)^2-s^2=κ^2-2κ-1≤0.
$$
If `κ>1+s`, then `κ-1-s>0`, while
`κ-1+s=(κ-1-s)+2s>0` because `s≥0`. Their product would be positive. Hence
`κ≤1+√2`, with no assumption on the sign of `κ`.

#### Worked instance

At `κ=1+√2`, equality holds because `κ²=3+2√2=2κ+1`. The lower root
`1-√2` is another equality case, confirming why negative inputs are allowed.

#### Boundary case

With a strict quadratic inequality, `κ` lies strictly between the roots. The
weak inequality must retain both roots as boundary cases.

#### Historical context

Source boundary: `CROUZEIX-PALENCIA-2017` supports the result-level `1+√2` bound through a publisher metadata receipt; no full text was captured locally.

Review status: this factor proof is a checked textbook reconstruction, not a quotation or independent reproduction of the 2017 paper.

#### ML analogy

Mathematical object: extraction of an explicit endpoint from a self-bounding scalar inequality.

ML counterpart: converting a certified stability relation `r²≤2r+1` into a robustness radius.

Exact transfer: any real diagnostic satisfying that relation is at most `1+√2`, regardless of sign.

Non-transfer: algebra cannot establish the premise for an ML system from empirical samples alone.

Diagnostic: log the quadratic residual and factor product alongside the final constant.

#### Pedagogical prerequisites

Factoring quadratics, contradiction, multiplication of positive reals, and the
two square-root facts already proved.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.one_plus_sqrt_two_barrier`.
Formal mode: `proved-here`.
Substantive provider: `none`.
Readable type map: any real `κ` satisfying the quadratic inequality obeys the upper endpoint, with no sign premise.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L31).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {x : Real}, LE.le.{0} (HPow.hPow.{0, 0, 0} x (OfNat.ofNat.{0} 2)) (HAdd.hAdd.{0, 0, 0} (HMul.hMul.{0, 0, 0} (OfNat.ofNat.{0} 2) x) (OfNat.ofNat.{0} 1)) → LE.le.{0} x (HAdd.hAdd.{0, 0, 0} (OfNat.ofNat.{0} 1) (Real.sqrt (OfNat.ofNat.{0} 2)))`.
Type SHA-256: `b34b7fab7e6f0711708e126020eace13d85ec177642c7a3d3c3172f0dc9bc8f5`.
Direct maintained dependencies: `none`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: only the displayed scalar quadratic inequality.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:b34b7fab7e6f0711708e126020eace13d85ec177642c7a3d3c3172f0dc9bc8f5`.

#### Exercises and solutions

CFT-27-E04 constructs aligned complex summands that attain `1+√2`, showing
that the triangle inequality can be sharp for those two lengths. This does not
claim simultaneous equality in every operator estimate leading to the barrier.

### CFT-27-005 — the independent-term triangle kernel {#cft-27-005}

#### Purpose

Derive the norm estimate replacing coupled operator terms by budgets `2κ` and
one, and expose the aligned equality case that blocks scalar improvement.

#### Definitions and notation

Let `u,v` lie in a normed additive space. In the application `u=S^*T`, `v=H`,
and `κ=‖T‖`; the estimate knows no relative phase or inner product.

#### Statement

If `‖u‖≤2κ` and `‖v‖≤1`, then the triangle inequality gives the exact budget
$$\|u-v\|≤2κ+1.$$
The claim needs no separately stated sign assumption on `κ`; its hypotheses
already express everything used by the proof.

#### Hypothesis ledger

The two norm bounds are independent. No inner product, commutation, positivity,
phase relation, or common generative origin is assumed.

#### Proof roadmap

Apply the triangle inequality to `u+(-v)`, substitute both norm budgets, and
then compare the estimate with the compiled aligned complex witness.

#### Proof

We derive the independent-term triangle estimate and expose its equality case:
$$\|u-v\|≤\|u\|+\|v\|≤2κ+1.$$
The precise checked sharpness witness used here is narrower than this general
normed-group statement. CFT-27-E04 takes the complex witnesses `a=1` and
`b=√2`, for which `‖a‖=1`, `‖b‖=√2`, and `‖a+b‖=1+√2`. Equivalently, set
`κ=√2/2`, `u=b`, and `v=-a` in the displayed difference estimate. Then
`‖u‖=2κ` and `‖v‖=1`, so the kernel is sharp for this compiled scalar witness.
We do not infer a scalar-multiplication equality witness in every normed
additive group, because that structure is absent from the compiled public
theorem.

#### Worked instance

The exercise witness `a=1`, `b=√2` attains `1+√2` over `ℂ`. Complex phase
helps only when the proof controls it; the public kernel itself retains no
phase hypothesis.

#### Boundary case

If `u⊥v`, Pythagoras gives `‖u-v‖²=‖u‖²+‖v‖²`, strictly better than the
squared triangle bound when both vectors are nonzero.

#### Historical context

Source boundary: the triangle inequality is standard normed-space theory; its barrier role is newly authored explanatory analysis of the registered prior-bound context.

Review status: the bound, equality witness, and orthogonal comparison were checked independently of later constant-two routes.

#### ML analogy

Mathematical object: the independent-component estimate `‖u-v‖≤‖u‖+‖v‖`.

ML counterpart: separately bounding two representation-error or gradient components before combining them.

Exact transfer: marginal certificates `‖u‖≤a`, `‖v‖≤b` imply the worst-case sharp bound `‖u-v‖≤a+b`.

Non-transfer: worst-case alignment does not show that components from a trained network actually align.

Diagnostic: estimate their signed cosine on the same examples before treating the independent sum as descriptive.

#### Pedagogical prerequisites

Normed spaces, triangle inequality, scalar multiplication, and the difference
between marginal and joint constraints.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.triangle_barrier_kernel`.
Formal mode: `proved-here`.
Substantive provider: `none`.
Readable type map: Lean states the kernel for `X+Y`; substituting `X=u` and
`Y=-v` gives the displayed difference estimate. The premise `‖X‖≤2κ`
already forces the sign needed by any nontrivial instance.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L46).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {E : Type u_1} [inst : SeminormedAddCommGroup.{u_1} E] (X Y : E) (kappa : Real), LE.le.{0} (Norm.norm.{u_1} X) (HMul.hMul.{0, 0, 0} (OfNat.ofNat.{0} 2) kappa) → LE.le.{0} (Norm.norm.{u_1} Y) (OfNat.ofNat.{0} 1) → LE.le.{0} (Norm.norm.{u_1} (HAdd.hAdd.{u_1, u_1, u_1} X Y)) (HAdd.hAdd.{0, 0, 0} (HMul.hMul.{0, 0, 0} (OfNat.ofNat.{0} 2) kappa) (OfNat.ofNat.{0} 1))`.
Type SHA-256: `4ee8f501079eaa15f32b005c52c5275ec658b9f32af686f89de532b09a890aeb`.
Direct maintained dependencies: `none`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: a normed additive group and the three displayed scalar bounds.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:4ee8f501079eaa15f32b005c52c5275ec658b9f32af686f89de532b09a890aeb`.

#### Exercises and solutions

CFT-27-E05 proves the Pythagorean replacement under orthogonality, identifying
one precise kind of missing joint structure.

### CFT-27-006 — the positive-map quadratic and discarded cross term {#cft-27-006}

#### Purpose

Assemble the adjoint identity and two norm budgets into the quadratic barrier,
then locate the exact correlation information erased by the triangle step.

#### Definitions and notation

The compiled kernel uses a positive boundary density `D`, a bounded continuous
boundary function `h`, matrices `T,R`, and the positive-map value
`Φ_D(h)=boundaryPhiCLM D h`. Its decomposition is
$$T^*T=Φ_D(h)+R.$$
This is the abstraction consumed after the classical motivating identity has
already supplied the positive-map budget.

#### Statement

Assume
$$T^*T=Φ_D(h)+R,$$
$$\tfrac12\!\left(\int\!\|D(x)\|\,dμ(x)\right)\|h\|≤2\|T\|,
\qquad \|R\|≤1.$$
Then `‖T‖²≤2‖T‖+1`. Combining this card with CFT-27-004 gives
`‖T‖≤1+√2`.

#### Hypothesis ledger

The matrix operator norm supplies `‖T^*T‖=‖T‖²`. The theorem
`boundaryPhi_norm_le` converts the density integral into a norm bound on
`Φ_D(h)`. The two displayed premises control that term and `R` separately.

#### Proof roadmap

Use the C*-identity, substitute the decomposition, apply
`boundaryPhi_norm_le`, and finish with CFT-27-005. Then inspect the exact
Hilbert-space cross-term identity to see what that last step forgets.

#### Proof

We derive the positive-map quadratic barrier and identify the discarded cross term.
First, `boundaryPhi_norm_le` and the assumed coefficient estimate give
$$\|Φ_D(h)\|≤2\|T\|.$$
Therefore
$$
\|T\|^2=\|T^*T\|=\|Φ_D(h)+R\|
≤\|Φ_D(h)\|+\|R\|≤2\|T\|+1.
$$
This proves the compiled operator-norm theorem. The identity below is a
vector-level analogue, not an operator-norm identity. To connect the two
truthfully when the finite matrix space is nonzero, write `U=Φ_D(h)` and
`V=R`. The operator `U+V` has a finite-dimensional norm-attaining unit vector
`x`, so `‖(U+V)x‖=‖U+V‖`. Set `u=Ux` and `v=-Vx`. The vector identity gives
the exact bridge
$$\|U+V\|^2=\|Ux\|^2+\|Vx\|^2+2\operatorname{Re}\langle Ux,Vx\rangle.$$
At the vector level, before replacing joint geometry by separate norms,
$$\|u-v\|^2=\|u\|^2+\|v\|^2-2\operatorname{Re}\langle u,v\rangle.$$
The independent estimate permits the worst case
`Re⟨u,v⟩=-‖u‖‖v‖`; equivalently, for the sum above it permits
`Re⟨Ux,Vx⟩=‖Ux‖‖Vx‖`. It then uses `‖Ux‖≤‖U‖` and `‖Vx‖≤‖V‖`, forgetting
both the actual phase and the common input `x`. Consequently the
independent-budget argument alone cannot lower `1+√2` to two. A favorable
cross term would improve the vector estimate, but this identity by itself does
not prove that the operator pair has one.

Supporting Lean identity: `CrouzeixTextbook.Part05.barrier_cross_term_identity`
is checked at [Chapter27.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L168).
Formalization scope: the Lean helper formalizes only this vector identity; the
norm-attaining finite-dimensional bridge and any correlation estimate are not
claimed as part of that helper's formal surface.
Its focused compiler-probe receipt, which is not a canonical publication row,
has type SHA-256
`9b6fcb9d9fddcbd5133e2693f201b84cc8916ea65c2804bebdb1b7afa7409aec`,
axioms `Classical.choice, Quot.sound, propext`, probe target
`CrouzeixTextbook`, and probe identity
`CrouzeixTextbook:9b6fcb9d9fddcbd5133e2693f201b84cc8916ea65c2804bebdb1b7afa7409aec`.

#### Worked instance

For `A_{0,2}=2J` and `p(z)=z`, homogeneity preserves the checked ratio two:
`‖2J‖=2` and the numerical radius scales from `1/2` to one. The method gives
only `2≤1+√2`; the slack `√2-1` diagnoses proof loss rather than operator
behavior. The Lean receipt for the normalized witness `J` is linked above.

#### Boundary case

For the generic difference `u-v`, zero correlation gives Pythagoras and
positive correlation improves the bound. For the `U+V` bridge, the favorable
sign is negative. Neither sign is supplied by the independent norm budgets.

#### Historical context

Source boundary: `CROUZEIX-PALENCIA-2017` is a metadata-only receipt for the published `1+√2` result; the symmetrized identity has separate inspected evidence.

Review status: the derivation is a compiler-backed Harp reconstruction; no full-text 2017 reproduction, priority claim, acceptance claim, or attainment claim is made.

#### ML analogy

Mathematical object: a norm bound that forgets the signed cross term between coupled outputs.

ML counterpart: a robustness analysis that bounds two jointly generated error components independently.

Exact transfer: `-2 Re⟨u,v⟩` exactly quantifies the gap between joint and marginal vector analysis.

Non-transfer: No trained-network claim is implied; empirical alignment and distribution shift need separate evidence.

Diagnostic: measure the normalized cross-term residual `ρ=Re⟨u,v⟩/(‖u‖‖v‖)` when defined and report it beside the independent bound.

#### Pedagogical prerequisites

Adjoints, Hilbert-space operator norms, the C*-identity, positive maps from
Chapter 22, and the double-layer construction from Chapter 26.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.positive_map_norm_kernel`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixConjecture.PositiveBoundaryDensity, CrouzeixConjecture.PositiveBoundaryDensity.density, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.boundaryPhi, CrouzeixConjecture.boundaryPhiCLM, CrouzeixConjecture.boundaryPhi_norm_le, CrouzeixTextbook.Part05.triangle_barrier_kernel`.
Readable type map: `D,h,T,R` satisfy the displayed positive-map decomposition,
coefficient budget, and companion bound; the conclusion is exactly the
quadratic norm inequality. The classical `S,H` identity motivates these inputs
but is not falsely presented as the public theorem's signature.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L57).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {i : Type u_1} {n : Type u_2} [inst : TopologicalSpace.{u_1} i] [inst_1 : MeasurableSpace.{u_1} i] [inst_2 : OpensMeasurableSpace.{u_1} i] [inst_3 : Fintype.{u_2} n] [inst_4 : DecidableEq.{u_2 + 1} n] {mu : MeasureTheory.Measure.{u_1} i} (D : CrouzeixConjecture.PositiveBoundaryDensity.{u_1, u_2} mu) (h : BoundedContinuousFunction.{u_1, 0} i Complex) (T R : CrouzeixConjecture.SquareMatrix.{u_2} n), Eq.{u_2 + 1} (HMul.hMul.{u_2, u_2, u_2} (Matrix.conjTranspose.{0, u_2, u_2} T) T) (HAdd.hAdd.{u_2, u_2, u_2} (DFunLike.coe.{max (u_1 + 1) (u_2 + 1), u_1 + 1, u_2 + 1} (CrouzeixConjecture.boundaryPhiCLM.{u_1, u_2} D) h) R) → LE.le.{0} (HMul.hMul.{0, 0, 0} (HMul.hMul.{0, 0, 0} (Inv.inv.{0} (OfNat.ofNat.{0} 2)) (MeasureTheory.integral.{u_1, 0} mu fun x => Norm.norm.{u_2} (CrouzeixConjecture.PositiveBoundaryDensity.density.{u_1, u_2} D x))) (Norm.norm.{u_1} h)) (HMul.hMul.{0, 0, 0} (OfNat.ofNat.{0} 2) (Norm.norm.{u_2} T)) → LE.le.{0} (Norm.norm.{u_2} R) (OfNat.ofNat.{0} 1) → LE.le.{0} (HPow.hPow.{0, 0, 0} (Norm.norm.{u_2} T) (OfNat.ofNat.{0} 2)) (HAdd.hAdd.{0, 0, 0} (HMul.hMul.{0, 0, 0} (OfNat.ofNat.{0} 2) (Norm.norm.{u_2} T)) (OfNat.ofNat.{0} 1))`.
Type SHA-256: `ff5adddf717c380cb4f3b3123f60e9e8569e51e49c18c3d4d3c8de299b91935e`.
Direct maintained dependencies: `CrouzeixConjecture.PositiveBoundaryDensity, CrouzeixConjecture.PositiveBoundaryDensity.density, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.boundaryPhi, CrouzeixConjecture.boundaryPhiCLM, CrouzeixConjecture.boundaryPhi_norm_le, CrouzeixTextbook.Part05.triangle_barrier_kernel`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: complex Hilbert-space structure, the decomposition, and two norm bounds.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:ff5adddf717c380cb4f3b3123f60e9e8569e51e49c18c3d4d3c8de299b91935e`.

#### Exercises and solutions

CFT-27-E06 reconstructs the factor proof without calling the endpoint theorem
and locates norm nonnegativity only in the operator application.

## Worked examples

The aligned scalar pair in CFT-27-005 realizes equality in the triangle step,
so the relaxation can be sharp when only the marginal norms are known. The
nonnormal matrix `A_{0,2}` points in the other direction: its checked
polynomial ratio is two, strictly below `1+√2`. Together the examples separate
sharpness of the local inequality from sharpness of the Crouzeix constant.

## ML bridge

The exact analogy is an analysis that bounds two coupled error components by
their marginal norms and forgets their signed correlation. The concrete
diagnostic is the normalized cross-term residual
`ρ=Re⟨u,v⟩/(‖u‖‖v‖)` on the same samples. A value far from the worst-case sign
shows slack in the independent-component certificate. No trained-network
claim follows from the analogy; empirical alignment and distribution shift
remain separate questions.

## Lean translation

The six public declarations are compiler-bound in the correspondence table of
their cards, and the six exercises have distinct solution theorems in
[`Chapter27.lean`](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L80).
The public scalar barrier has no `κ≥0` hypothesis. Exercise E01 retains that
operator-norm premise pedagogically but reconstructs the factorization without
calling the endpoint theorem; E03 and E06 likewise expose independent proof
terms rather than shortcut aliases.

## Exercises

### CFT-27-E01 — operator-norm barrier {#exercise-cft-27-e01}

Solve κ²≤2κ+1 under the operator-norm hypothesis κ≥0.

#### Complete written solution

We factor at 1±√2, determine the factor signs, and derive the upper bound. The
hypothesis gives `(κ-1-√2)(κ-1+√2)≤0`. If `κ>1+√2`, both factors are
positive, a contradiction. Therefore `κ≤1+√2`; `κ≥0` is valid norm context
but unnecessary scalar input.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_01_solution` in [Chapter27.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L80).
Formal mode: `proved-here`; the proof avoids the public barrier theorem.
Compiler locator: line 80, column 9.
Type SHA-256: `68c6d3b9d3a4994f12a30480fb4fe7a1dd0ad1a6a1bf27b50f896042bff2c944`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:68c6d3b9d3a4994f12a30480fb4fe7a1dd0ad1a6a1bf27b50f896042bff2c944`.

### CFT-27-E02 — locate the constant {#exercise-cft-27-e02}

Numerically compare 1+√2 with 2 and 5/2.

#### Complete written solution

We square positive comparisons and obtain 2<1+√2<5/2. For the lower bound,
subtracting one reduces the claim to `1<√2`. Both sides are nonnegative, and
`1²<2=(√2)²`, so the desired strict inequality follows. For the upper bound,
subtracting one reduces the claim to `√2<3/2`. Again both sides are
nonnegative, while `(√2)²=2<9/4=(3/2)²`. Thus
$$2<1+\sqrt2<\frac52.$$
The sign checks matter: squaring does not preserve order on all real numbers.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_02_solution` in [Chapter27.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L96).
Formal mode: `proved-here`; both strict inequalities are reconstructed.
Compiler locator: line 96, column 9.
Type SHA-256: `e1f3ec1cd602af53c5ff34a3a8a4f02706beff01c46434ba4d0dfa9dc7262563`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:e1f3ec1cd602af53c5ff34a3a8a4f02706beff01c46434ba4d0dfa9dc7262563`.

### CFT-27-E03 — complete the square {#exercise-cft-27-e03}

Prove the quadratic barrier by completing the square.

#### Complete written solution

We rewrite as (κ-1)²≤2 and use the nonnegative square-root branch to derive the upper bound.
Indeed `(κ-1)²=κ²-2κ+1≤2`; if `κ-1>√2`, squaring positive quantities
contradicts this inequality. Hence `κ≤1+√2`, without assuming `κ≥0`.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_03_solution` in [Chapter27.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L104).
Formal mode: `proved-here`; the proof avoids the public barrier theorem.
Compiler locator: line 104, column 9.
Type SHA-256: `bb00d3ff964d18a29c0d722841b2b2763d9a964178d9076cc48ef9ac1c1a4495`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:bb00d3ff964d18a29c0d722841b2b2763d9a964178d9076cc48ef9ac1c1a4495`.

### CFT-27-E04 — aligned equality {#exercise-cft-27-e04}

Show the triangle inequality can attain 1+√2.

#### Complete written solution

We construct aligned complex summands with norms 1 and √2 and verify equality in the triangle inequality.
Take `a=1`, `b=√2` as nonnegative real complex numbers. Then `‖a‖=1`,
`‖b‖=√2`, and `‖a+b‖=1+√2` because their phases coincide.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_04_solution` in [Chapter27.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L122).
Formal mode: `proved-here`; the equality witness is explicit.
Compiler locator: line 122, column 9.
Type SHA-256: `caf6285507ebf86498b027f7cccb42d4576f2d599e2ffcef1e9ff53b0172efa0`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:caf6285507ebf86498b027f7cccb42d4576f2d599e2ffcef1e9ff53b0172efa0`.

### CFT-27-E05 — orthogonality {#exercise-cft-27-e05}

Give a structural condition improving the triangle estimate for vectors in a complex inner-product space.

#### Complete written solution

We assume inner-product orthogonality and replace the triangle bound by the Pythagorean norm identity.
Expanding gives `‖x+y‖²=‖x‖²+‖y‖²+2 Re⟨x,y⟩`; when `⟨x,y⟩=0`,
the cross term vanishes, hence `‖x+y‖²=‖x‖²+‖y‖²`.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_05_solution` in [Chapter27.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L132).
Formal mode: `proved-here`; the complex inner-product expansion is checked.
Compiler locator: line 132, column 9.
Type SHA-256: `3f90e0243253841699e7da931dc016840f9c032b405a9582eac70c4b3ca73173`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:3f90e0243253841699e7da931dc016840f9c032b405a9582eac70c4b3ca73173`.

### CFT-27-E06 — scalar versus operator signs {#exercise-cft-27-e06}

Show the scalar upper barrier does not require κ≥0, and locate where nonnegativity enters the operator application.

#### Complete written solution

We factor the quadratic at 1±√2 and derive κ≤1+√2 without a sign hypothesis; prove separately that κ≥0 when κ is instantiated as an operator norm.
The factor contradiction proves the endpoint. If `κ=‖T‖`, then norm
nonnegativity separately gives `0≤κ`; that fact is application context only.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_06_solution` in [Chapter27.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L140).
Formal mode: `proved-here`; the proof never calls the endpoint theorem.
Compiler locator: line 140, column 9.
Type SHA-256: `988d32168cdb145b0cdb418258040fd63fa105628f3b46203623f494df97e0ff`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:988d32168cdb145b0cdb418258040fd63fa105628f3b46203623f494df97e0ff`.

## Synthesis and forward dependencies

The square-root cards make the scalar endpoint exact. The triangle and
positive-map cards show why that endpoint appears: the method erases a signed
cross term. Chapter 28 keeps a complete power family, allowing later proofs to
carry joint structure farther before taking norms. The arithmetic improves
only because the information reaching it improves.
