---
id: cft-chapter-28-complete-power-family
title: The complete power family
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-29
tags: [crouzeix-textbook, crouzeix-machinery, mathematics, lean]
confidence: high
canonical: 28_complete_power_family.md
chapter: 28
part: 5
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 28: The complete power family

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-v-crouzeix-machinery|Part V — Crouzeix machinery]]
Previous: [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/27_one_plus_sqrt_two_barrier|Chapter 27 — The one-plus-square-root-two barrier]]
Next: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/29_crouzeix_problem_and_sharpness|Chapter 29 — The Crouzeix problem and sharpness]]

## Opening problem

Chapter 27 showed where `1+√2` loses information: a triangle inequality treats
two coupled terms as independent lengths. Here we retain multiplication. The
phrase *complete power family* means that the Cauchy identity holds for every `m : ℕ` on one fixed contour and one fixed measure.

The Cayley transform has the expansion
$$(1+zw)/(1-zw)=1+2∑_{m≥0}(zw)^{m+1}.$$
Transporting this series through a boundary integral requires every
coefficient. Knowing one function at `m=1` is insufficient: it says nothing
about whether the integral of `f^m` is the `m`th power of the integral of `f`.

The power-family construction through CFT-28-005 does not invoke a Jin,
Lorist--Schwenninger, or Harp terminal provider. The final card is deliberately
different: it previews the Jin completion-to-two consequence and labels that
dependency rather than presenting it as common machinery. Calling
this a cross-route complete-power mechanism is the Harp comparison inference
`CC-040`, not terminology attributed to either source.

## Conceptual model

Let `Γ` be a positively oriented boundary of a convex outer domain `Ω`, let
`μ` be its finite parameter measure, let `W(B)⊆Ω`, and let `f` be a scalar
boundary function with `|f|≤1`. Write `F_Γ(B,x)` for the analytic first part of
the double-layer density. The family is
$$\int f(x)^mF_Γ(B,x)\,dμ(x)=T^m,\qquad m=0,1,2,\ldots.$$
Here the `m=0` identity fixes mass one. The positive powers supply all coefficients
of one analytic Cayley function.

For the running matrix
$$A_{0,2}=\begin{pmatrix}0&2\\0&0\end{pmatrix},$$
the algebra is
$$A_{0,2}^0=I,\qquad A_{0,2}^1=A_{0,2},\qquad A_{0,2}^m=0\quad(m≥2).$$
Higher members record nilpotence, information that a norm bound on the first
member cannot recover.

This truncation is checked independently in Lean: supporting theorem
`CrouzeixTextbook.Part05.a_zero_two_power_truncation`
([code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L264)).
Its focused compiler receipt records the standard project axioms
`Classical.choice`, `Quot.sound`, and `propext`; it is supporting evidence for
the example, not an additional CFT correspondence row.

## Formal development

### CFT-28-001 — one contour for every analytic power {#cft-28-001}

#### Purpose

This card states the power-indexed Cauchy formula with domain, orientation,
and analyticity. It identifies the analytic input needed to exchange a
boundary integral with the uniformly convergent Cayley series.

#### Definitions and notation

An oriented radial convex boundary `G` parametrizes `∂Ω` counterclockwise.
The finite measure is the contour-parameter measure; the coefficient
`k=-i/(2π)` and the period carry the normalization. A simple diagonalization
`h_B` defines `f(B)` on eigenvalues, and `F_Γ(B,x)` abbreviates
`parametricBoundaryFirstPart Γ B x`.

#### Statement

Assume `Ω̄` is compact and contained in an open set `V`, `f` is complex
differentiable on `V`, `|f(z)|≤1` on `Ω̄`, and `W(B)⊆Ω`. For a positively
oriented radial boundary and simple diagonalization of `B`,
$$\int f(Γ(x))^mF_Γ(B,x)\,dμ(x)=f(B)^m
\qquad\text{for every }m∈\mathbb N.$$

#### Hypothesis ledger

Compactness creates a positive buffer inside `V`; analyticity then passes to
every `f^m`. Convexity and numerical-range containment control the resolvent.
Orientation fixes the sign and constant. Simple diagonalization identifies the
integral with the matrix functional calculus.

#### Proof roadmap

Thicken `Ω̄` inside `V`, apply the matrix Cauchy-resolvent formula to `f^m`,
rewrite the interval integral using the shared parameter measure, cancel the
period and orientation constants, then use multiplicativity of diagonal
functional calculus.

#### Proof

We state the power-indexed Cauchy formula with domain, orientation, and analyticity.
Choose `ε>0` so the closed `ε`-thickening of `Ω̄` lies in `V`.
For each `m`, `z↦f(z)^m` is holomorphic there. The oriented formula gives
$$\int_0^{2π}k f(Γ(t))^mΓ'(t)(Γ(t)I-B)^{-1}\,dt
=h_B.\operatorname{functionEval}(f^m),$$
where `k=-i/(2π)`. The normalization is exact:
$$k(2π i)=1.$$
Since the diagonal calculus is an algebra homomorphism,
$$h_B.\operatorname{functionEval}(f^m)
=\bigl(h_B.\operatorname{functionEval}(f)\bigr)^m,$$
which proves the one-contour family.

#### Worked instance

For the scalar matrix `[b]`, this becomes the ordinary identity
`∮f(ζ)^m(ζ-b)^{-1}dζ/(2πi)=f(b)^m`. The matrix proof applies this at every
eigenvalue and conjugates back with the same change of basis.

#### Boundary case

If the contour or measure changes with `m`, the individual formulas no longer
come from one bounded linear evaluation map. Termwise application to the
Cayley series is then unjustified even if each displayed equality is true.

#### Historical context

Source boundary: the scalar engine is Cauchy's integral formula; the oriented radial matrix package is maintained Harp proof code, not a claim of historical priority.

Review status: the buffer, orientation constant, common measure, and power multiplicativity were checked against the compiler-validated provider.

#### ML analogy

Mathematical object: one linear evaluation map preserving every power of a bounded scalar function.

ML counterpart: one transition model whose multi-step operators equal repeated composition of its one-step operator.

Exact transfer: the coherence equation `T_m=T_1^m` is algebraically identical in both settings.

Non-transfer: finitely many sampled rollout agreements do not establish an analytic contour formula or an infinite family.

Diagnostic: estimate several horizon operators on one held-out split and compare each with the corresponding power of the first.

#### Pedagogical prerequisites

Cauchy's integral formula, compact subsets of open sets, diagonal matrix
functional calculus, and the oriented double-layer first part from Chapter 26.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.power_cauchy_formula`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixConjecture.ContourParameter, CrouzeixConjecture.HasParametricPowerCauchyFormula, CrouzeixConjecture.PositivePeriodicRadialData, CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary, CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.hasParametricPowerCauchyFormula_of_holomorphic_of_simpleDiagonalization, CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundary, CrouzeixConjecture.SimpleDiagonalization, CrouzeixConjecture.SimpleDiagonalization.functionEval, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.contourParameterMeasure, CrouzeixConjecture.contourPeriod, CrouzeixConjecture.numericalRange, CrouzeixConjecture.parametricContractiveBoundaryFunctionOfContinuousOn`.
Readable type map: the radial data, domain, open neighborhood, compact closure, holomorphic scalar function, bound, matrix, diagonalization, and numerical-range containment match the statement above.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L11).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] (R : CrouzeixConjecture.PositivePeriodicRadialData) (c : Complex) {Omega V : Set.{0} Complex} (G : R.OrientedRadialConvexBoundary c Omega), IsOpen.{0} V → IsCompact.{0} (closure.{0} Omega) → ∀ (hclosure : LE.le.{0} (closure.{0} Omega) V) {f : Complex → Complex} (hf : DifferentiableOn.{0, 0, 0} Complex f V) (hbound : ∀ (z : Complex), Membership.mem.{0, 0} (closure.{0} Omega) z → LE.le.{0} (Norm.norm.{0} (f z)) (OfNat.ofNat.{0} 1)) (B : CrouzeixConjecture.SquareMatrix.{u_1} n) (hB : CrouzeixConjecture.SimpleDiagonalization.{u_1} B), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} B) Omega → CrouzeixConjecture.HasParametricPowerCauchyFormula.{0, u_1} (CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundary R c G) CrouzeixConjecture.contourParameterMeasure B (CrouzeixConjecture.parametricContractiveBoundaryFunctionOfContinuousOn.{0} (CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundary R c G) f ⋯ hbound) (CrouzeixConjecture.SimpleDiagonalization.functionEval.{u_1} hB f)`.
Type SHA-256: `258dcd40effc3e1c87075f12ed8e7b33594385efb646067ebd90517af2e3238f`.
Direct maintained dependencies: `CrouzeixConjecture.ContourParameter, CrouzeixConjecture.HasParametricPowerCauchyFormula, CrouzeixConjecture.PositivePeriodicRadialData, CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary, CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.hasParametricPowerCauchyFormula_of_holomorphic_of_simpleDiagonalization, CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundary, CrouzeixConjecture.SimpleDiagonalization, CrouzeixConjecture.SimpleDiagonalization.functionEval, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.contourParameterMeasure, CrouzeixConjecture.contourPeriod, CrouzeixConjecture.numericalRange, CrouzeixConjecture.parametricContractiveBoundaryFunctionOfContinuousOn`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: exactly the geometric, analytic, diagonalization, and containment binders in the readable type map.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:258dcd40effc3e1c87075f12ed8e7b33594385efb646067ebd90517af2e3238f`.

#### Exercises and solutions

CFT-28-E01 exposes the definition of the common-contour family and checks that
the universal power quantifier sits inside the package.

### CFT-28-002 — zeroth power and mass one {#cft-28-002}

#### Purpose

This card applies scalar Cauchy to the constant function and tracks
normalization. The identity is what turns the symmetrized double-layer density
into a positive density of total mass `2I`.

#### Definitions and notation

For matrices `T^0=I`; for scalars `f(x)^0=1`, even at a zero of `f`. Put
`F(x)=F_Γ(B,x)` and `D(x)=F(x)+F(x)^*`. Their masses differ by a factor two.

#### Statement

If the complete family holds, then
$$\int F_Γ(B,x)\,dμ(x)=I,$$
and hence, when adjoint commutes with the finite-dimensional integral,
`∫D(x)dμ(x)=2I`.

#### Hypothesis ledger

The only mathematical premise is the common power Cauchy formula. The ambient
topology and measurable-space structure make the integral expression
well-typed; no compactness or finiteness premise enters this direct
specialization at zero.

#### Proof roadmap

Specialize at `m=0`, simplify scalar and matrix zeroth powers, and distinguish
the identity mass of `F` from the doubled mass of `D`.

#### Proof

We apply scalar Cauchy to the constant function and track normalization. The
`m=0` member is
$$\int f(x)^0F_Γ(B,x)\,dμ(x)=T^0.$$
Thus `f(x)^0=1` and `T^0=I` give
$$\int F_Γ(B,x)\,dμ(x)=I.$$
Consequently
$$\int D\,dμ=\int F\,dμ+\left(\int F\,dμ\right)^*=I+I=2I.$$

#### Worked instance

At the center of the unit disk, the scalar analytic first part is constant
one under normalized angular measure. It has mass one; its symmetrization is
constant two and has mass two.

#### Boundary case

Starting the family at `m=1` loses the normalization. Every positive-power
moment may be specified while the mass of the density remains undetermined,
so positivity cannot later repair the missing scale.

#### Historical context

Source boundary: mass one is the constant-function case of Cauchy's formula; the matrix statement is local common machinery rather than a historical attribution.

Review status: direct specialization at zero and the separate `I` versus `2I` normalizations were checked in prose and Lean.

#### ML analogy

Mathematical object: a zeroth moment fixing the scale of an operator-valued density.

ML counterpart: a normalization constraint requiring learned mixture weights to sum to one before comparing higher moments.

Exact transfer: the zeroth equation rules out arbitrary common rescaling in both settings.

Non-transfer: total mass one does not imply matrix positivity or correctness of any higher moment.

Diagnostic: compute the empirical zeroth moment separately and report its operator-norm residual from the identity.

#### Pedagogical prerequisites

Natural powers, the matrix identity, finite-dimensional Bochner integration,
and the distinction between an analytic first part and its symmetrization.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.power_cauchy_mass_one`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixConjecture.ContractiveBoundaryFunction, CrouzeixConjecture.ContractiveBoundaryFunction.function, CrouzeixConjecture.HasParametricPowerCauchyFormula, CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.parametricBoundaryFirstPart`.
Readable type map: `hCauchy` is the common power formula and the conclusion is the identity mass of its analytic first part.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L35).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {i : Type u_1} {n : Type u_2} [inst : TopologicalSpace.{u_1} i] [inst_1 : MeasurableSpace.{u_1} i] [inst_2 : Fintype.{u_2} n] [inst_3 : DecidableEq.{u_2 + 1} n] {mu : MeasureTheory.Measure.{u_1} i} {Omega : Set.{0} Complex} {Gamma : CrouzeixConjecture.ParametricConvexBoundary.{u_1} Omega} {B : CrouzeixConjecture.SquareMatrix.{u_2} n} {f : CrouzeixConjecture.ContractiveBoundaryFunction.{u_1} i} {T : CrouzeixConjecture.SquareMatrix.{u_2} n}, CrouzeixConjecture.HasParametricPowerCauchyFormula.{u_1, u_2} Gamma mu B f T → Eq.{u_2 + 1} (MeasureTheory.integral.{u_1, u_2} mu fun x => CrouzeixConjecture.parametricBoundaryFirstPart.{u_1, u_2} Gamma B x) (OfNat.ofNat.{u_2} 1)`.
Type SHA-256: `af7ffea07579ffc73817d257712c98a60596ed2a7df38403d75cee67224d96dd`.
Direct maintained dependencies: `CrouzeixConjecture.ContractiveBoundaryFunction, CrouzeixConjecture.ContractiveBoundaryFunction.function, CrouzeixConjecture.HasParametricPowerCauchyFormula, CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.parametricBoundaryFirstPart`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: the common-contour formula and ambient topology, measure, and finite-matrix typeclasses in the Lean type.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:af7ffea07579ffc73817d257712c98a60596ed2a7df38403d75cee67224d96dd`.

#### Exercises and solutions

CFT-28-E02 proves the companion fact that every scalar power remains
contractive when the original boundary function is contractive.

### CFT-28-003 — normalized boundary data stay contractive {#cft-28-003}

#### Purpose

This card defines the normalized boundary function and proves its supremum
norm is at most one. The uniform bound controls every scalar Cayley series on
compact subdisks.

#### Definitions and notation

Let `f` be continuous on `Ω̄` with `|f(z)|≤1` there. Its boundary restriction
is `h(x)=f(Γ(x))`. Lean packages `h` as a bounded continuous function together
with its pointwise norm proof.

#### Statement

There exists a contractive boundary function `h` such that
$$h(x)=f(Γ(x)),\qquad \|h(x)\|≤1,$$
for every `x`, and its bounded-continuous-function norm satisfies `‖h‖≤1`.
Consequently `‖h(x)^m‖≤1` for every natural `m`.

#### Hypothesis ledger

Continuity on `Ω̄` makes the restriction continuous on the compact parameter
space. Every boundary point lies in `∂Ω⊆Ω̄`, so the closed-domain bound applies.
No matrix assumption is needed for this scalar construction.

#### Proof roadmap

Compose `f` with the continuous boundary map, package the result as a bounded
continuous function, transport the closed-domain bound pointwise, then use the
characterization of its supremum norm.

#### Proof

We define normalized boundary data and prove its supremum norm is at most one.
Set `h=f∘Γ`. Since
$$Γ(x)∈∂Ω⊆\overline Ω,$$
the hypothesis gives `‖h(x)‖=‖f(Γ(x))‖≤1`. The bounded-continuous-function norm
is the supremum of the pointwise norms, hence `‖h‖≤1`. For powers,
$$\|h(x)^m\|=\|h(x)\|^m≤1^m=1,$$
including `m=0`.

#### Worked instance

For the unit disk and `f(z)=z`, the boundary function is `h(e^{it})=e^{it}`.
Its norm and every power norm equal one, so contractivity is sharp and does not
degrade with the power index.

#### Boundary case

A bound on finitely many sampled boundary points is not a uniform bound. The
function may exceed one between samples, and the scalar Cayley series then has
no certified common convergence disk.

#### Historical context

Source boundary: this is the boundary restriction used by the maintained holomorphic double-layer construction, with no source-priority or trained-model claim.

Review status: boundary containment, the application identity, pointwise contractivity, and the actual supremum-norm conclusion were compiler checked.

#### ML analogy

Mathematical object: a uniformly contractive scalar function on the whole boundary parameter space.

ML counterpart: a recurrent scalar gate constrained in magnitude before repeated composition.

Exact transfer: a pointwise magnitude bound by one prevents every scalar power from exceeding one.

Non-transfer: bounded scalar gates alone do not control nonnormal matrix products or establish statistical stability.

Diagnostic: maximize the gate magnitude over an adversarial validation mesh and repeat the measurement for several powers.

#### Pedagogical prerequisites

Continuous functions on compact spaces, supremum norms, boundary containment,
and the elementary norm identity for scalar powers.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.contractive_boundary_function`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixConjecture.ContractiveBoundaryFunction, CrouzeixConjecture.ContractiveBoundaryFunction.function, CrouzeixConjecture.ContractiveBoundaryFunction.norm_le_one, CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.ParametricConvexBoundary.point, CrouzeixConjecture.parametricContractiveBoundaryFunctionOfContinuousOn`.
Readable type map: the constructed boundary object is paired with its exact application identity and its bounded-continuous-function norm estimate.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L50).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {i : Type u_1} [inst : TopologicalSpace.{u_1} i] [CompactSpace.{u_1} i] {Omega : Set.{0} Complex} (Gamma : CrouzeixConjecture.ParametricConvexBoundary.{u_1} Omega) (f : Complex → Complex), ContinuousOn.{0, 0} f (closure.{0} Omega) → (∀ (z : Complex), Membership.mem.{0, 0} (closure.{0} Omega) z → LE.le.{0} (Norm.norm.{0} (f z)) (OfNat.ofNat.{0} 1)) → Exists.{u_1 + 1} fun h => And (∀ (x : i), Eq.{1} (DFunLike.coe.{u_1 + 1, u_1 + 1, 1} (CrouzeixConjecture.ContractiveBoundaryFunction.function.{u_1} h) x) (f (DFunLike.coe.{u_1 + 1, u_1 + 1, 1} (CrouzeixConjecture.ParametricConvexBoundary.point.{u_1} Gamma) x))) (LE.le.{0} (Norm.norm.{u_1} (CrouzeixConjecture.ContractiveBoundaryFunction.function.{u_1} h)) (OfNat.ofNat.{0} 1))`.
Type SHA-256: `89cd7ac55f8eef42107167ebd2bc6310d339d9a3a319ed58ca3ceba31f31e907`.
Direct maintained dependencies: `CrouzeixConjecture.ContractiveBoundaryFunction, CrouzeixConjecture.ContractiveBoundaryFunction.function, CrouzeixConjecture.ContractiveBoundaryFunction.norm_le_one, CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.ParametricConvexBoundary.point, CrouzeixConjecture.parametricContractiveBoundaryFunctionOfContinuousOn`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: continuity and the uniform bound on the closed domain, plus the boundary data's topology.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:89cd7ac55f8eef42107167ebd2bc6310d339d9a3a319ed58ca3ceba31f31e907`.

#### Exercises and solutions

CFT-28-E03 derives the scalar Cayley power series and checks convergence under
the strict inequality `‖zw‖<1`.

### CFT-28-004 — Cayley companions for the entire family {#cft-28-004}

#### Purpose

This card constructs every Cayley companion/power, not a single function, and
proves the algebraic identity used by positivity. It is where the complete
power family becomes one positive-real analytic package.

#### Definitions and notation

For `z∈𝔻`, put `c_z(x)=(1+zf(x))/(1-zf(x))` and
`C_z(T)=(I+zT)(I-zT)^{-1}`. Let `D=F+F^*`, `H(z)=Φ_D(c_z)`, and let `g(z)` be
the companion formed from the adjoint analytic half.

#### Statement

There are functions `H` and `g` such that, for every `z∈𝔻`,
$$g(z)∈\operatorname{alg}(B),\qquad \operatorname{Re}H(z)\succeq0,$$
and
$$2H(z)=C_z(T)+g(z)^*.$$
This is one universal statement over the disk, not a checkpoint at one `z`.

#### Hypothesis ledger

Contractivity gives `|zf(x)|<1` and uniform convergence on each closed subdisk.
The full power family identifies every Taylor coefficient. Mass one normalizes
`D`. Numerical-range containment gives density positivity and companion
algebra membership. Spectral containment makes `C_z(T)` analytic.

#### Proof roadmap

Expand the scalar Cayley transform, apply the boundary integral term by term,
replace each coefficient by `T^m`, and identify the matrix series by spectral
radius rather than an operator-norm contraction. Pass from finite partial sums
to the integral through the bounded linear integration map. Then combine the
analytic and adjoint halves of the positive double-layer density.

#### Proof

We construct every Cayley companion power and prove the positivity identity.
For a scalar `w` with `|zw|<1`, direct rationalization gives
`Re((1+zw)/(1-zw))=(1-|zw|²)/|1-zw|²`. In particular, for
`|zf(x)|<1`, the geometric series converges uniformly when `|z|≤r<1`:
$$c_z(x)=1+2\sum_{m≥0}z^{m+1}f(x)^{m+1}.$$
The shared integral map and power family give
the finite identities
$$
\int\!\left(1+2\sum_{m=0}^{N}z^{m+1}f^{m+1}\right)F\,dμ
=I+2\sum_{m=0}^{N}z^{m+1}T^{m+1}.
$$
The limit on the left may pass through the integral because the scalar series
converges uniformly and integration against the fixed matrix-valued density is
a bounded linear integration map.

The right side needs a different argument. Spectral mapping and the hypothesis
`σ(T)⊆\overline{𝔻}` give
$$σ(zT)=zσ(T),\qquad r(zT)≤|z|<1.$$
Finite-dimensional spectral-radius theory now implies that
$$\sum_{m≥0}(zT)^m$$
converges in operator norm and `(zT)^{N+1}→0`. The finite telescoping identity
$$
(I-zT)\sum_{m=0}^{N}(zT)^m=I-(zT)^{N+1}
$$
therefore yields
$$
(I-zT)^{-1}=\sum_{m≥0}(zT)^m.
$$
Consequently
$$
\int c_zF\,dμ
=I+2\sum_{m≥0}z^{m+1}T^{m+1}
=2(I-zT)^{-1}-I
=C_z(T).
$$
The positive map built from `D` has
$$\operatorname{Re}c_z(x)=\frac{1-|zf(x)|^2}{|1-zf(x)|^2}≥0,$$
so `Re H(z)≽0`. Splitting `D=F+F^*` yields the exact direct identity
$$2Φ_D(c_z)=C_z(T)+g(z)^*.$$

#### Worked instance

For scalar `T=t` with `|t|≤1`, the matrix series reduces to
`C_z(t)=(1+zt)/(1-zt)`. If `t=0`, all positive coefficients vanish and the
Cayley transform is the constant one.

#### Boundary case

When `|zw|=1`, the denominator may vanish. The proof works on the open unit
disk and uses uniform convergence only on closed subdisks `|z|≤r<1`; it makes
no unproved boundary-limit claim. For a nonnormal matrix the proof cannot use `‖zT‖<1`:
if `T=[[0,M],[0,0]]`, then `σ(T)={0}` but `‖zT‖=|z|M` can exceed
one. Spectral radius, not operator norm, is the load-bearing convergence input.

#### Historical context

Source boundary: Cayley transforms and geometric series are classical; the all-parameter companion package is the maintained common formalization, and `CC-040` is Harp's comparison inference.

Review status: uniform convergence, the real-part numerator, companion algebra, adjoint placement, and factor two were checked against the proof surface.

#### ML analogy

Mathematical object: a generating function packaging an infinite coherent sequence of operator moments.

ML counterpart: a resolvent-style generating function for every rollout horizon of one transition model.

Exact transfer: equality of all coefficients implies equality of analytic generating functions within a shared convergence disk.

Non-transfer: matching finitely many rollout horizons does not justify the infinite series or positivity everywhere in the disk.

Diagnostic: compare truncated generating functions at several radii and report the residual as the number of shared coefficients increases.

#### Pedagogical prerequisites

Geometric series, uniform convergence, bounded linear maps, adjoints, generated
matrix algebras, and Chapter 26's positive double-layer density.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.power_cayley_companion`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixConjecture.ContractiveBoundaryFunction, CrouzeixConjecture.HasParametricPowerCauchyFormula, CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.closedUnitDisk, CrouzeixConjecture.doubleLayerCayleySeries, CrouzeixConjecture.doubleLayerCayleySeries_rePart_posSemidef, CrouzeixConjecture.generatedAlgebra, CrouzeixConjecture.matrixCayleyTransform, CrouzeixConjecture.matrixSpectrum, CrouzeixConjecture.numericalRange, CrouzeixConjecture.parametricPositiveBoundaryDensityOfMass, CrouzeixConjecture.parametricPowerCayleyCompanion, CrouzeixConjecture.parametricPowerCayleyCompanion_mem_generatedAlgebra, CrouzeixConjecture.parametric_direct_cayley_identity_of_powerCauchy, CrouzeixConjecture.rePart, CrouzeixConjecture.unitDisk, CrouzeixTextbook.Part05.power_cauchy_mass_one`.
Readable type map: the theorem universally constructs the companion and positive function, then records algebra membership, positive semidefinite real part, and the direct Cayley identity for each disk point.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L68).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {i : Type u_1} {n : Type u_2} [inst : TopologicalSpace.{u_1} i] [inst_1 : CompactSpace.{u_1} i] [inst_2 : MeasurableSpace.{u_1} i] [inst_3 : OpensMeasurableSpace.{u_1} i] [inst_4 : Fintype.{u_2} n] [inst_5 : DecidableEq.{u_2 + 1} n] [Nonempty.{u_2 + 1} n] {mu : MeasureTheory.Measure.{u_1} i} [inst_7 : MeasureTheory.IsFiniteMeasure.{u_1} mu] {Omega : Set.{0} Complex} (Gamma : CrouzeixConjecture.ParametricConvexBoundary.{u_1} Omega) (B : CrouzeixConjecture.SquareMatrix.{u_2} n) (hWB : LE.le.{0} (CrouzeixConjecture.numericalRange.{u_2} B) Omega) (f : CrouzeixConjecture.ContractiveBoundaryFunction.{u_1} i) (T : CrouzeixConjecture.SquareMatrix.{u_2} n) (hCauchy : CrouzeixConjecture.HasParametricPowerCauchyFormula.{u_1, u_2} Gamma mu B f T), LE.le.{0} (CrouzeixConjecture.matrixSpectrum.{u_2} T) CrouzeixConjecture.closedUnitDisk → Exists.{u_2 + 1} fun g => And (∀ (z : Complex), Membership.mem.{0, 0} CrouzeixConjecture.unitDisk z → Membership.mem.{u_2, u_2} (CrouzeixConjecture.generatedAlgebra.{u_2} B) (g z)) (And (∀ (z : Complex), Membership.mem.{0, 0} CrouzeixConjecture.unitDisk z → Matrix.PosSemidef.{u_2, 0} (CrouzeixConjecture.rePart.{u_2} (CrouzeixConjecture.doubleLayerCayleySeries.{u_1, u_2} (CrouzeixConjecture.parametricPositiveBoundaryDensityOfMass.{u_1, u_2} Gamma B hWB ⋯) f z))) (∀ (z : Complex), Membership.mem.{0, 0} CrouzeixConjecture.unitDisk z → Eq.{u_2 + 1} (HSMul.hSMul.{0, u_2, u_2} (OfNat.ofNat.{0} 2) (CrouzeixConjecture.doubleLayerCayleySeries.{u_1, u_2} (CrouzeixConjecture.parametricPositiveBoundaryDensityOfMass.{u_1, u_2} Gamma B hWB ⋯) f z)) (HAdd.hAdd.{u_2, u_2, u_2} (CrouzeixConjecture.matrixCayleyTransform.{u_2} z T) (Matrix.conjTranspose.{0, u_2, u_2} (g z)))))`.
Type SHA-256: `b8f10ac0cc5db54f484a749ec7b303f346a9d9803d1a9bab62062bbefc2099dc`.
Direct maintained dependencies: `CrouzeixConjecture.ContractiveBoundaryFunction, CrouzeixConjecture.HasParametricPowerCauchyFormula, CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.closedUnitDisk, CrouzeixConjecture.doubleLayerCayleySeries, CrouzeixConjecture.doubleLayerCayleySeries_rePart_posSemidef, CrouzeixConjecture.generatedAlgebra, CrouzeixConjecture.matrixCayleyTransform, CrouzeixConjecture.matrixSpectrum, CrouzeixConjecture.numericalRange, CrouzeixConjecture.parametricPositiveBoundaryDensityOfMass, CrouzeixConjecture.parametricPowerCayleyCompanion, CrouzeixConjecture.parametricPowerCayleyCompanion_mem_generatedAlgebra, CrouzeixConjecture.parametric_direct_cayley_identity_of_powerCauchy, CrouzeixConjecture.rePart, CrouzeixConjecture.unitDisk, CrouzeixTextbook.Part05.power_cauchy_mass_one`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: boundary geometry, numerical-range containment, contractivity, the complete family, and spectral containment.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:b8f10ac0cc5db54f484a749ec7b303f346a9d9803d1a9bab62062bbefc2099dc`.

#### Exercises and solutions

CFT-28-E04 reconstructs mass one directly from the zeroth member of the
family, without invoking a named mass theorem.

### CFT-28-005 — positive-real completion from all powers {#cft-28-005}

#### Purpose

This card assembles pointwise positivity into a positive-real completion and
preserves the complete power family. It packages the common analytic work
needed by the later norm extraction.

#### Definitions and notation

A positive-real completion is a matrix-valued analytic `H` on `𝔻` with
`H(0)=I`, positive semidefinite real part, a companion in `alg(B)`, and the
direct Cayley identity linking `H`, `T`, and that companion.

#### Statement

If `W(B)⊆Ω`, the complete Cauchy family holds, and
`σ(T)⊆\overline{𝔻}`, then there exists `H` satisfying
$$\operatorname{Re}H(z)\succeq0\ (z∈𝔻),\qquad H(0)=I,$$
together with the companion and direct-identity fields of
`IsPositiveRealCompletion B T H`.

#### Hypothesis ledger

Numerical-range containment makes the double-layer density positive. The
zeroth power normalizes its mass, all positive powers transport the Cayley
coefficients, contractivity controls convergence, and spectral containment
makes the matrix Cayley transform analytic.

#### Proof roadmap

Construct `D`, `H`, and `g` explicitly. Prove analyticity, the value at zero,
and positive real part from the lower-level boundary-map theorems. Use CFT-28-004
for algebra membership and the direct identity, then fill the completion
structure field by field.

#### Proof

We assemble pointwise positivity into a positive-real completion for all powers,
while preserving the complete power family. Let `D=F+F^*`; then `D(x)≽0` and
`∫D dμ=2I`. Define
$$H(z)=Φ_D(c_z),\qquad g(z)=\text{the common-contour companion}.$$
The formula
$$\operatorname{Re}c_z(x)=\frac{1-|zf(x)|^2}{|1-zf(x)|^2}≥0$$
and positivity of `Φ_D` imply `Re H(z)≽0`; mass gives `H(0)=I`. CFT-28-004
provides `g(z)∈alg(B)` and
$$2H(z)=C_z(T)+g(z)^*.$$
Since `C_z(T)=2(I-zT)^{-1}-I`, rearrangement identifies the completion defect;
in particular `\tfrac12(g(z)^*-I)∈\operatorname{alg}(B^*)`. These verified
fields construct `H` without calling a terminal existence wrapper.

#### Worked instance

For scalar `T=t`, `|t|≤1`, take `H(z)=(1+zt)/(1-zt)`. Its real part is
nonnegative, `H(0)=1`, and its positive Taylor coefficients are `2t^m`.

#### Boundary case

Pointwise positivity without mass normalization produces an unknown positive
matrix at zero rather than `I`. Mass one without pointwise positivity cannot
certify a positive real part. Powers alone still need positivity and spectral
hypotheses.

#### Historical context

Source boundary: positive-real and Herglotz methods are classical; this exact completion is reconstructed in the common Harp module and is not attributed as source terminology.

Review status: the lower-level construction, rather than the former existence shortcut, was checked through compiler proof dependencies.

#### ML analogy

Mathematical object: a positive-real analytic certificate whose coefficients obey one exact multiplicative law.

ML counterpart: a frequency-domain certificate built from rollout operators constrained to be powers of one learned transition.

Exact transfer: a common coefficient sequence prevents independently fitted horizons from being inserted into one generating function.

Non-transfer: positive Hermitian parts on a finite frequency grid do not prove positivity throughout the disk.

Diagnostic: track the smallest sampled Hermitian-part eigenvalue and coefficient-coherence residual separately; neither diagnostic is a proof.

#### Pedagogical prerequisites

Positive operator-valued integrals, analytic matrix functions, generated
algebras, Cayley transforms, and the preceding all-parameter identity.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.positive_completion_from_power_family`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixConjecture.ContractiveBoundaryFunction, CrouzeixConjecture.HasParametricPowerCauchyFormula, CrouzeixConjecture.IsPositiveRealCompletion, CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.PositiveBoundaryDensity, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.closedUnitDisk, CrouzeixConjecture.doubleLayerCayleySeries, CrouzeixConjecture.doubleLayerCayleySeries_analyticOnNhd, CrouzeixConjecture.doubleLayerCayleySeries_zero, CrouzeixConjecture.generatedAlgebra, CrouzeixConjecture.isPositiveRealCompletion_of_direct_cayley_identity, CrouzeixConjecture.matrixCayleyTransform, CrouzeixConjecture.matrixSpectrum, CrouzeixConjecture.numericalRange, CrouzeixConjecture.parametricPositiveBoundaryDensityOfMass, CrouzeixConjecture.rePart, CrouzeixConjecture.unitDisk, CrouzeixTextbook.Part05.power_cauchy_mass_one, CrouzeixTextbook.Part05.power_cayley_companion`.
Readable type map: the contour, matrix, containment, contractive boundary function, full family, and spectral inclusion produce an explicit existential positive-real completion.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L102).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {i : Type u_1} {n : Type u_2} [inst : TopologicalSpace.{u_1} i] [CompactSpace.{u_1} i] [inst_2 : MeasurableSpace.{u_1} i] [OpensMeasurableSpace.{u_1} i] [inst_4 : Fintype.{u_2} n] [inst_5 : DecidableEq.{u_2 + 1} n] [Nonempty.{u_2 + 1} n] {mu : MeasureTheory.Measure.{u_1} i} [MeasureTheory.IsFiniteMeasure.{u_1} mu] {Omega : Set.{0} Complex} (Gamma : CrouzeixConjecture.ParametricConvexBoundary.{u_1} Omega) (B : CrouzeixConjecture.SquareMatrix.{u_2} n), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_2} B) Omega → ∀ (f : CrouzeixConjecture.ContractiveBoundaryFunction.{u_1} i) (T : CrouzeixConjecture.SquareMatrix.{u_2} n), CrouzeixConjecture.HasParametricPowerCauchyFormula.{u_1, u_2} Gamma mu B f T → LE.le.{0} (CrouzeixConjecture.matrixSpectrum.{u_2} T) CrouzeixConjecture.closedUnitDisk → Exists.{u_2 + 1} fun H => CrouzeixConjecture.IsPositiveRealCompletion.{u_2} B T H`.
Type SHA-256: `655425bab338ed249ed27b699808957c6fddbfb80f2a31fdfa72e65852a6ccb0`.
Direct maintained dependencies: `CrouzeixConjecture.ContractiveBoundaryFunction, CrouzeixConjecture.HasParametricPowerCauchyFormula, CrouzeixConjecture.IsPositiveRealCompletion, CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.PositiveBoundaryDensity, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.closedUnitDisk, CrouzeixConjecture.doubleLayerCayleySeries, CrouzeixConjecture.doubleLayerCayleySeries_analyticOnNhd, CrouzeixConjecture.doubleLayerCayleySeries_zero, CrouzeixConjecture.generatedAlgebra, CrouzeixConjecture.isPositiveRealCompletion_of_direct_cayley_identity, CrouzeixConjecture.matrixCayleyTransform, CrouzeixConjecture.matrixSpectrum, CrouzeixConjecture.numericalRange, CrouzeixConjecture.parametricPositiveBoundaryDensityOfMass, CrouzeixConjecture.rePart, CrouzeixConjecture.unitDisk, CrouzeixTextbook.Part05.power_cauchy_mass_one, CrouzeixTextbook.Part05.power_cayley_companion`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: exactly the common geometric, measure, power-family, and spectral hypotheses in the type.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:655425bab338ed249ed27b699808957c6fddbfb80f2a31fdfa72e65852a6ccb0`.

#### Exercises and solutions

CFT-28-E05 extracts all indexed equations and mass one from one package while
keeping the shared contour and measure visible.

### CFT-28-006 — deferred Jin completion-to-two consequence {#cft-28-006}

**DEFERRED JIN COMPLETION-TO-TWO CONSEQUENCE.** This card is a preview. Its
Lean declaration is exact, but the textbook proof of the terminal implication
is intentionally deferred to the Jin route in Chapters 30--32.

#### Purpose

This card shows how the common completion constructed in CFT-28-005 enters a
norm-two conclusion once the Jin completion-to-two theorem is supplied. The
common power-family development ends with the completion; the terminal
extraction is not proved self-containedly here.

#### Definitions and notation

Write `T=S diag(λ)S⁻¹` for the target representation supplied by a simple
diagonalization of `B`. Spectral contractivity means `|λ_j|≤1`. The completion
is the object constructed in CFT-28-005.

#### Statement

If the common-contour power family holds, `W(B)⊆Ω`, `B` has a simple
diagonalization, `T=S diag(λ)S⁻¹`, and `|λ_j|≤1` for every `j`, then
$$\|T\|≤2.$$

#### Hypothesis ledger

The power family constructs the completion; numerical-range containment gives
density positivity; diagonalization gives the eigenvalue model; and the bounds
on `λ_j` put the spectrum in the closed disk. The final implication from a
positive-real completion to `‖T‖≤2` is the Jin-route provider
`CrouzeixConjecture.positiveRealCompletionStatement`.

#### Proof roadmap

Derive spectral containment from the diagonal representation and invoke the
earlier public completion constructed from lower-level common machinery. Then
use CFT-30-002, whose terminal completion-to-two implication is decomposed in
the detailed CFT-30--CFT-32 chain, especially CFT-32-001.

#### Proof

We assemble the common completion and preview the deferred Jin completion-to-two consequence.

The part proved in this chapter is the input assembly.
Similarity and the diagonal spectrum give
$$σ(T)=\{λ_j:j\}\subseteq\overline{𝔻}.$$
CFT-28-005 then constructs `H` with positive real part and the exact
Cayley-companion identity. At this point the common proof stops. The Lean
declaration calls `CrouzeixConjecture.positiveRealCompletionStatement`, the
Jin provider exposed pedagogically as CFT-30-002; CFT-32-001 later reconstructs
the completion-implies-norm-two step. That deferred result yields
$$\|T\|≤2.$$
Thus the exact Lean composition is verified here, while the mathematical
reason the completion forces the constant two belongs to the later Jin
workshop rather than to this summary.

#### Worked instance

For `A_{0,2}`, the coherent sequence is `I,A_{0,2},0,0,…`. Its norm is two,
so the endpoint is attained in the normalized problem whose sharpness is
proved in Chapter 29.

#### Boundary case

Independent bounds `‖T^m‖≤C_m` do not imply that boundary moments are powers
of one operator. Breaking exact coherence breaks the Cayley coefficient
identity and reopens the loss from Chapter 27.

#### Historical context

Source boundary: the complete-power-to-completion construction is maintained
common machinery. The terminal provider
`CrouzeixConjecture.positiveRealCompletionStatement` is the audited Jin-route
result (`JIN-V4-AUDITED`), previewed here and taught in CFT-30-002 through the
Chapter 30--32 chain. “Complete power family” as a cross-route organizing idea
remains Harp inference `CC-040`, not source terminology.

Review status: spectral inclusion and the dependency on public CFT-28-005 are
checked here; the terminal provider provenance and the forward reader links to
CFT-30-002 and CFT-32-001 are explicit.

#### ML analogy

Mathematical object: an infinite family of exact power identities used as one analytic certificate.

ML counterpart: multi-horizon transition estimates constrained to arise by composing one nonnormal dynamics matrix.

Exact transfer: multiplicative coherence is the equality `T_m=T_1^m` in either notation.

Non-transfer: No trained-network claim follows; learned operators also contain estimation error, mismatch, and finite-data effects.

Diagnostic: measure the normalized multiplicative-coherence residual `r_m=‖\hat T_m-\hat T_1^m‖/(1+‖\hat T_1‖^m)` on held-out data.

#### Pedagogical prerequisites

Spectrum under similarity, diagonal functional calculus, and positive-real
completion from CFT-28-005. For a reconstructible terminal proof, continue to
CFT-30-002 and then the detailed CFT-30--CFT-32 chain, especially CFT-32-001.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.power_family_norm_two`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixConjecture.ContractiveBoundaryFunction, CrouzeixConjecture.HasParametricPowerCauchyFormula, CrouzeixConjecture.IsPositiveRealCompletion, CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.SimpleDiagonalization, CrouzeixConjecture.SimpleDiagonalization.changeBasis, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.closedUnitDisk, CrouzeixConjecture.innerConjugation, CrouzeixConjecture.matrixSpectrum, CrouzeixConjecture.matrixSpectrum.eq_1, CrouzeixConjecture.numericalRange, CrouzeixConjecture.positiveRealCompletionStatement, CrouzeixTextbook.Part05.positive_completion_from_power_family`.
Readable type map: the family, containment, diagonalization, similarity representation, and eigenvalue bound build the Jin provider's hypotheses and then yield `‖T‖≤2`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L129).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {i : Type u_1} {n : Type u_2} [inst : TopologicalSpace.{u_1} i] [CompactSpace.{u_1} i] [inst_2 : MeasurableSpace.{u_1} i] [OpensMeasurableSpace.{u_1} i] [inst_4 : Fintype.{u_2} n] [inst_5 : DecidableEq.{u_2 + 1} n] [Nonempty.{u_2 + 1} n] {mu : MeasureTheory.Measure.{u_1} i} [MeasureTheory.IsFiniteMeasure.{u_1} mu] {Omega : Set.{0} Complex} (Gamma : CrouzeixConjecture.ParametricConvexBoundary.{u_1} Omega) (B T : CrouzeixConjecture.SquareMatrix.{u_2} n), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_2} B) Omega → ∀ (f : CrouzeixConjecture.ContractiveBoundaryFunction.{u_1} i), CrouzeixConjecture.HasParametricPowerCauchyFormula.{u_1, u_2} Gamma mu B f T → ∀ (hB : CrouzeixConjecture.SimpleDiagonalization.{u_2} B) (lambda : n → Complex), Eq.{u_2 + 1} T (DFunLike.coe.{u_2 + 1, u_2 + 1, u_2 + 1} (CrouzeixConjecture.innerConjugation.{u_2} (CrouzeixConjecture.SimpleDiagonalization.changeBasis.{u_2} hB)) (Matrix.diagonal.{0, u_2} lambda)) → (∀ (j : n), LE.le.{0} (Norm.norm.{0} (lambda j)) (OfNat.ofNat.{0} 1)) → LE.le.{0} (Norm.norm.{u_2} T) (OfNat.ofNat.{0} 2)`.
Type SHA-256: `a505f972d0a99bda91d63ea153feeadbea85c3112034679470c5e48b4da9f7bc`.
Direct maintained dependencies: `CrouzeixConjecture.ContractiveBoundaryFunction, CrouzeixConjecture.HasParametricPowerCauchyFormula, CrouzeixConjecture.IsPositiveRealCompletion, CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.SimpleDiagonalization, CrouzeixConjecture.SimpleDiagonalization.changeBasis, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.closedUnitDisk, CrouzeixConjecture.innerConjugation, CrouzeixConjecture.matrixSpectrum, CrouzeixConjecture.matrixSpectrum.eq_1, CrouzeixConjecture.numericalRange, CrouzeixConjecture.positiveRealCompletionStatement, CrouzeixTextbook.Part05.positive_completion_from_power_family`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: the finite-dimensional typeclasses and all common geometric, diagonalization, and spectral binders in the type.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:a505f972d0a99bda91d63ea153feeadbea85c3112034679470c5e48b4da9f7bc`.

#### Exercises and solutions

CFT-28-E06 assembles the deferred consequence with all hypotheses explicit.
Its final line deliberately calls the same audited Jin completion-to-two
provider; it is a Lean assembly exercise, not a self-contained proof of that
provider.

## Worked examples

The running nilpotent matrix `A_{0,2}` separates the three roles of the power
index. At `m=0` its power is `I`, which fixes the mass normalization. At
`m=1` its power is `A_{0,2}`, which retains the nonnormal operator. Only for
`m≥2` does the power vanish. Thus the complete family is not the repeated
assertion that every positive power is zero; its first two members carry the
data later consumed by the completion argument.

The scalar Cayley example supplies the analytic counterpart. For `|zw|<1`,
the geometric series converges to `(1+zw)/(1-zw)` and has positive real part
`(1-|zw|²)/|1-zw|²`. At `|zw|=1` the denominator can vanish, so the boundary
is a genuine limit of the disk argument rather than another point of uniform
convergence.

## ML bridge

For estimated horizon operators, the exact mathematical analogue of the power
family is multiplicative coherence: `\hat T_m=\hat T_1^m`. A scale-aware
held-out diagnostic is
$$r_m=\frac{\|\hat T_m-\hat T_1^m\|}{1+\|\hat T_1\|^m}.$$
Small residuals can reveal inconsistency between separately fitted horizons,
but they do not supply contour analyticity, density positivity, or spectral
containment. No trained-network claim follows from the theorem: estimation
error, model mismatch, and finite-data effects remain outside its hypotheses.

## Lean translation

The six public declarations in
[[formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean|Chapter28.lean]]
are proved in the textbook namespace and have no underlying alias. They expose,
in order, the oriented-radial Cauchy construction, the direct zeroth-power
specialization, the boundary sup-norm bound, the disk-wide Cayley companion,
the reconstructed positive-real completion, and the final norm-two extraction.
The six exercises in the same file have distinct theorem statements and proof
bodies; none is merely the corresponding card declaration under a new name.

## Exercises

### CFT-28-E01 — definition surface {#exercise-cft-28-e01}

Define a common-contour power Cauchy family.

#### Hint

Keep `Γ`, `μ`, `B`, `f`, and `T` outside the universal quantifier over `m`.

#### Complete written solution

We bind one contour, measure, boundary function, matrix, and target to a formula valid for every natural power. By definition,
$$\operatorname{HasParametricPowerCauchyFormula}(Γ,μ,B,f,T)
\iff \forall m∈\mathbb N,\ \int f(x)^mF_Γ(B,x)\,dμ(x)=T^m.$$
The universal quantifier sits inside the package, so all powers use the same
five objects.

#### Lean solution

Declaration: `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_01_solution`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L152).
Normalized type: `∀ (i n : Type) [inst : TopologicalSpace.{0} i] [CompactSpace.{0} i] [inst_2 : MeasurableSpace.{0} i] [OpensMeasurableSpace.{0} i] [inst_4 : Fintype.{0} n] [inst_5 : DecidableEq.{1} n] [Nonempty.{1} n] (mu : MeasureTheory.Measure.{0} i) [MeasureTheory.IsFiniteMeasure.{0} mu] (Omega : Set.{0} Complex) (Gamma : CrouzeixConjecture.ParametricConvexBoundary.{0} Omega) (B : CrouzeixConjecture.SquareMatrix.{0} n) (f : CrouzeixConjecture.ContractiveBoundaryFunction.{0} i) (T : CrouzeixConjecture.SquareMatrix.{0} n), Iff (CrouzeixConjecture.HasParametricPowerCauchyFormula.{0, 0} Gamma mu B f T) (∀ (m : Nat), Eq.{1} (MeasureTheory.integral.{0, 0} mu fun x => HSMul.hSMul.{0, 0, 0} (HPow.hPow.{0, 0, 0} (DFunLike.coe.{1, 1, 1} (CrouzeixConjecture.ContractiveBoundaryFunction.function.{0} f) x) m) (CrouzeixConjecture.parametricBoundaryFirstPart.{0, 0} Gamma B x)) (HPow.hPow.{0, 0, 0} T m))`.
Type SHA-256: `d60e7a320e248d34ac0bc0538f91b9e73f70c4a13a0dfed2b08b86e5b614b87b`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:d60e7a320e248d34ac0bc0538f91b9e73f70c4a13a0dfed2b08b86e5b614b87b`.

### CFT-28-E02 — contractive powers {#exercise-cft-28-e02}

Prove pointwise contractivity of every power of a contractive boundary function.

#### Hint

Use the scalar norm-of-power identity and induction on the exponent.

#### Complete written solution

We use norm multiplicativity and monotonicity from ‖f(x)‖≤1 to every natural power. For fixed `x`,
$$\|f(x)^m\|=\|f(x)\|^m≤1^m=1.$$
At zero both sides equal one; the inductive step multiplies by the nonnegative
number `‖f(x)‖≤1`.

#### Lean solution

Declaration: `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_02_solution`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L170).
Normalized type: `∀ (i : Type) [inst : TopologicalSpace.{0} i] (f : CrouzeixConjecture.ContractiveBoundaryFunction.{0} i) (m : Nat) (x : i), LE.le.{0} (Norm.norm.{0} (HPow.hPow.{0, 0, 0} (DFunLike.coe.{1, 1, 1} (CrouzeixConjecture.ContractiveBoundaryFunction.function.{0} f) x) m)) (OfNat.ofNat.{0} 1)`.
Type SHA-256: `a9aef25e6854b94c3da4a46b2804a3c1f3571142ad88dac47781881686084661`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:a9aef25e6854b94c3da4a46b2804a3c1f3571142ad88dac47781881686084661`.

### CFT-28-E03 — scalar Cayley series {#exercise-cft-28-e03}

Derive the scalar Cayley power series.

#### Hint

Set `q=zw` and rewrite `(1+q)/(1-q)` as `1+2q/(1-q)`.

#### Complete written solution

We expand the named Cayley transform geometrically and justify convergence when ‖zw‖<1. Since `q=zw` has norm below one,
$$\sum_{m≥0}q^{m+1}=q\sum_{m≥0}q^m=\frac q{1-q}.$$
Therefore
$$1+2\sum_{m≥0}(zw)^{m+1}=1+\frac{2zw}{1-zw}
=\frac{1+zw}{1-zw}=\operatorname{cayleyTransform}(z,w).$$

#### Lean solution

Declaration: `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_03_solution`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L178).
Normalized type: `∀ (z w : Complex), LT.lt.{0} (Norm.norm.{0} (HMul.hMul.{0, 0, 0} z w)) (OfNat.ofNat.{0} 1) → Eq.{1} (CrouzeixConjecture.cayleyTransform z w) (HAdd.hAdd.{0, 0, 0} (OfNat.ofNat.{0} 1) (HMul.hMul.{0, 0, 0} (OfNat.ofNat.{0} 2) (tsum.{0, 0} fun m => HPow.hPow.{0, 0, 0} (HMul.hMul.{0, 0, 0} z w) (HAdd.hAdd.{0, 0, 0} m (OfNat.ofNat.{0} 1)))))`.
Type SHA-256: `de80e6e778a5dd402323a69c8ec31582f63fd1794ca2f38e67d19b9f25a5d638`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:de80e6e778a5dd402323a69c8ec31582f63fd1794ca2f38e67d19b9f25a5d638`.

### CFT-28-E04 — mass from the zeroth power {#exercise-cft-28-e04}

Derive mass one from the zeroth power Cauchy identity.

#### Hint

Specialize at `m=0` and simplify directly; do not call a named mass theorem.

#### Complete written solution

We specialize the simultaneous common-contour formula at m=0 and simplify both zeroth powers. It gives
$$\int f(x)^0F_Γ(B,x)\,dμ(x)=T^0.$$
Since both zeroth powers are one, scalar multiplication disappears and the
right side is `I`. Hence `∫F_Γ(B,x)dμ(x)=I`.

#### Lean solution

Declaration: `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_04_solution`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L201).
Normalized type: `∀ (i n : Type) [inst : TopologicalSpace.{0} i] [CompactSpace.{0} i] [inst_2 : MeasurableSpace.{0} i] [OpensMeasurableSpace.{0} i] [inst_4 : Fintype.{0} n] [inst_5 : DecidableEq.{1} n] [Nonempty.{1} n] (mu : MeasureTheory.Measure.{0} i) [MeasureTheory.IsFiniteMeasure.{0} mu] (Omega : Set.{0} Complex) (Gamma : CrouzeixConjecture.ParametricConvexBoundary.{0} Omega) (B : CrouzeixConjecture.SquareMatrix.{0} n) (f : CrouzeixConjecture.ContractiveBoundaryFunction.{0} i) (T : CrouzeixConjecture.SquareMatrix.{0} n), CrouzeixConjecture.HasParametricPowerCauchyFormula.{0, 0} Gamma mu B f T → Eq.{1} (MeasureTheory.integral.{0, 0} mu fun x => CrouzeixConjecture.parametricBoundaryFirstPart.{0, 0} Gamma B x) (OfNat.ofNat.{0} 1)`.
Type SHA-256: `c52cb4db0682cde318061df6e401cc47f4703d9785021336732f05802bd8c8b4`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:c52cb4db0682cde318061df6e401cc47f4703d9785021336732f05802bd8c8b4`.

### CFT-28-E05 — package and indexed identities {#exercise-cft-28-e05}

Compare the common-contour package with its indexed identities and mass normalization.

#### Hint

Unfold the package for the first conjunct and specialize that family at zero
for the second.

#### Complete written solution

We extract every indexed identity and the zeroth-power mass identity while keeping the shared contour and measure explicit. Unfolding gives
$$\forall m,\ \int f(x)^mF_Γ(B,x)\,dμ(x)=T^m.$$
Retain this family as the first conjunct. At `m=0` it simplifies directly to
`∫F_Γ(B,x)dμ(x)=I`, the second conjunct.

#### Lean solution

Declaration: `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_05_solution`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L215).
Normalized type: `∀ (i n : Type) [inst : TopologicalSpace.{0} i] [CompactSpace.{0} i] [inst_2 : MeasurableSpace.{0} i] [OpensMeasurableSpace.{0} i] [inst_4 : Fintype.{0} n] [inst_5 : DecidableEq.{1} n] [Nonempty.{1} n] (mu : MeasureTheory.Measure.{0} i) [MeasureTheory.IsFiniteMeasure.{0} mu] (Omega : Set.{0} Complex) (Gamma : CrouzeixConjecture.ParametricConvexBoundary.{0} Omega) (B : CrouzeixConjecture.SquareMatrix.{0} n) (f : CrouzeixConjecture.ContractiveBoundaryFunction.{0} i) (T : CrouzeixConjecture.SquareMatrix.{0} n), CrouzeixConjecture.HasParametricPowerCauchyFormula.{0, 0} Gamma mu B f T → And (∀ (m : Nat), Eq.{1} (MeasureTheory.integral.{0, 0} mu fun x => HSMul.hSMul.{0, 0, 0} (HPow.hPow.{0, 0, 0} (DFunLike.coe.{1, 1, 1} (CrouzeixConjecture.ContractiveBoundaryFunction.function.{0} f) x) m) (CrouzeixConjecture.parametricBoundaryFirstPart.{0, 0} Gamma B x)) (HPow.hPow.{0, 0, 0} T m)) (Eq.{1} (MeasureTheory.integral.{0, 0} mu fun x => CrouzeixConjecture.parametricBoundaryFirstPart.{0, 0} Gamma B x) (OfNat.ofNat.{0} 1))`.
Type SHA-256: `cf9f1ed4ba7cbe2b2f52a5ce61b9b0e88bbc7e945c9e30e72ee66da7cb7d9c03`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:cf9f1ed4ba7cbe2b2f52a5ce61b9b0e88bbc7e945c9e30e72ee66da7cb7d9c03`.

### CFT-28-E06 — assemble the deferred norm-two consequence {#exercise-cft-28-e06}

Assemble the deferred Jin completion-to-two consequence with every hypothesis explicit.
Treat the Jin completion-to-two theorem as a named deferred input.

#### Hint

Derive spectral containment from the diagonal representation, build the
completion from earlier common machinery, then apply
`CrouzeixConjecture.positiveRealCompletionStatement`.

#### Complete written solution

We derive spectral containment, construct the common positive-real completion,
and call the named Jin completion-to-two provider.

We retain common-contour power data, numerical-range containment, simple diagonalization, the target representation, and spectral contractivity in the norm-two conclusion. From `T=S diag(λ)S⁻¹` and `|λ_j|≤1`,
$$σ(T)=\{λ_j:j\}\subseteq\overline{𝔻}.$$
The common family constructs a positive-real completion. The final line uses
the Jin provider `positiveRealCompletionStatement` to obtain `‖T‖≤2`; its proof
is deferred to CFT-30-002 and CFT-32-001. Removing the family loses its
coefficient identity; removing eigenvalue bounds loses spectral containment.

#### Lean solution

Declaration: `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_06_solution`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L234).
Normalized type: `∀ (i n : Type) [inst : TopologicalSpace.{0} i] [CompactSpace.{0} i] [inst_2 : MeasurableSpace.{0} i] [OpensMeasurableSpace.{0} i] [inst_4 : Fintype.{0} n] [inst_5 : DecidableEq.{1} n] [Nonempty.{1} n] (mu : MeasureTheory.Measure.{0} i) [MeasureTheory.IsFiniteMeasure.{0} mu] (Omega : Set.{0} Complex) (Gamma : CrouzeixConjecture.ParametricConvexBoundary.{0} Omega) (B T : CrouzeixConjecture.SquareMatrix.{0} n), LE.le.{0} (CrouzeixConjecture.numericalRange.{0} B) Omega → ∀ (f : CrouzeixConjecture.ContractiveBoundaryFunction.{0} i), CrouzeixConjecture.HasParametricPowerCauchyFormula.{0, 0} Gamma mu B f T → ∀ (hB : CrouzeixConjecture.SimpleDiagonalization.{0} B) (lambda : n → Complex), Eq.{1} T (DFunLike.coe.{1, 1, 1} (CrouzeixConjecture.innerConjugation.{0} (CrouzeixConjecture.SimpleDiagonalization.changeBasis.{0} hB)) (Matrix.diagonal.{0, 0} lambda)) → (∀ (j : n), LE.le.{0} (Norm.norm.{0} (lambda j)) (OfNat.ofNat.{0} 1)) → LE.le.{0} (Norm.norm.{0} T) (OfNat.ofNat.{0} 2)`.
Type SHA-256: `8c621531e36aa52df03ccb1d9abbbf215e78182b93e1343e41ae3d90535c6f6a`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:8c621531e36aa52df03ccb1d9abbbf215e78182b93e1343e41ae3d90535c6f6a`.

## Synthesis and forward dependencies

Mass one fixes scale. Contractivity controls the scalar powers. The complete
family transports every Cayley coefficient, and positivity constructs a
completion. This chapter previews, but does not reconstruct, the Jin theorem
that converts that completion into the norm-two estimate. Chapter 29 states
the normalized Crouzeix problem and proves that two is sharp; Chapters 30--32
then supply the deferred route proof.
