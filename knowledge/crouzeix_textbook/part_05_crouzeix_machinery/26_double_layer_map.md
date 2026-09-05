---
id: cft-chapter-26-double-layer-map
title: The double-layer map
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-28
tags: [crouzeix-textbook, crouzeix-machinery, mathematics, lean]
confidence: high
canonical: 26_double_layer_map.md
chapter: 26
part: 5
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 26: The double-layer map

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-v-crouzeix-machinery|Part V — Crouzeix machinery]]
Previous: [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/25_convex_boundaries_and_cauchy_layers|Chapter 25 — Convex boundaries and Cauchy layers]]
Next: [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/27_one_plus_sqrt_two_barrier|Chapter 27 — The one-plus-square-root-two barrier]]

## Opening problem

Cauchy's formula represents `f(B)` with the resolvent `(σI-B)^{-1}`. For a nonnormal matrix that resolvent can be large even far from the spectrum. The remedy is structural: pair the analytic resolvent with its adjoint. A supporting line of the numerical range becomes a positive semidefinite matrix, and a resolvent congruence transports that positivity to an operator-valued boundary density.

The six links are
$$
\text{support geometry}\Rightarrow\sigma\notin W(B)\Rightarrow(\sigma I-B)^{-1}\text{ exists}
\Rightarrow S_{\sigma,\nu}(B)\succeq0
\Rightarrow R_\sigma^*S_{\sigma,\nu}(B)R_\sigma=D_{\sigma,\nu}(B)\succeq0
\Rightarrow\int D\,d\mu\succeq0.
$$
At the boundary the supporting inequality is non-strict. Invertibility there uses openness of the containing domain and `spectrum(B)⊆W(B)`, not an imaginary strict inequality.

## Conceptual model

The construction has three layers that should not be conflated. The support
line is scalar geometry: it controls every Rayleigh quotient. The support
matrix packages exactly that scalar inequality as a Hermitian quadratic form.
The resolvent congruence then changes coordinates from the support matrix to
the double-layer density. Positivity survives because a congruence evaluates
the old quadratic form at a new vector; no commutativity is used.

### Running nonnormal calculation

Let
$$N=\begin{pmatrix}0&2\\0&0\end{pmatrix},\qquad N^2=0.$$
In the prose, the inner product is linear in its first entry, as is customary
in operator theory. Mathlib is conjugate-linear in its first entry, so Lean
writes `⟪x,Bx⟫_ℂ` for the same scalar that the prose writes as `⟨Bx,x⟩`.
For a unit vector `x=(a,b)`, `⟨Nx,x⟩=2b\bar a` and `|2b\bar a|≤|a|²+|b|²=1`; every phase at modulus one is attained when `|a|=|b|=1/√2`. The numerical range is convex by Toeplitz--Hausdorff and contains `0`, so containing the unit circle while remaining in the closed unit disk forces `W(N)` to be that disk. On `σ=Re^{it}`, `R>1`, with outward normal `ν=e^{it}=σ/R`, nilpotence gives
$$
R_\sigma=(\sigma I-N)^{-1}=\sigma^{-1}I+\sigma^{-2}N
=\begin{pmatrix}\sigma^{-1}&2\sigma^{-2}\\0&\sigma^{-1}\end{pmatrix}.
$$
The unnormalised density is
$$
D(t)=νR_\sigma+\barνR_\sigma^*
=\begin{pmatrix}2/R&2/(R\sigma)\\2/(R\bar\sigma)&2/R\end{pmatrix}.
$$
Its eigenvalues are `2/R±2/R²`. It is positive definite for `R>1`, and singular PSD at `R=1`. This explicit example audits every scalar factor below.

For the integration audit, the parameter domain is $[0,2π]$. We must keep
the unnormalized measure $dμ(t)=dt$ distinct from the normalized angular
measure $d\widehat{μ}(t)=dt/(2π)$. Define the raw analytic factor
$$
A(t)=σR_\sigma
=\begin{pmatrix}1&2/\sigma\\0&1\end{pmatrix}.
$$
The scalar Cauchy factor is already built into the analytic first part:
$$
F(t)=\frac{1}{2π}A(t)
=\frac{1}{2π}\begin{pmatrix}1&2/\sigma\\0&1\end{pmatrix}.
$$
The adjoint-symmetrized parametric density has the same built-in factor:
$$
K(t)=F(t)+F(t)^*=\frac{R}{2π}D(t)
=\frac{1}{2π}\begin{pmatrix}2&2/\sigma\\2/\bar\sigma&2\end{pmatrix}.
$$
Thus, with Lebesgue measure `dt`, the oscillatory entries vanish and
$$
\int_0^{2π}F(t)\,dt=I,\qquad
\int_0^{2π}K(t)\,dt=2I.
$$
Equivalently, with normalized angular measure one integrates the *raw*
factor: $\int_0^{2π}A(t)\,d\widehat{μ}(t)=I$, while its symmetrization
has mass `2I`. One must not combine `d\widehat{μ}` with the built-in
`1/(2π)` in `F` or `K` and still claim these masses. The analytic first part has identity mass; the adjoint-symmetrized density has mass $2I$.

## Formal development

### CFT-26-001 — support point outside the numerical range {#cft-26-001}

#### Purpose

Translate planar support geometry into a scalar inequality for every Rayleigh quotient, and distinguish a boundary point of an open domain from a member of the numerical range.

#### Definitions and notation

An outward support datum `(σ,ν)` for `Ω⊆ℂ` has `|ν|=1`, `σ∈∂Ω`, and
$$\operatorname{Re}(\barν(σ-z))\ge0\quad(z\in Ω).$$
Also `W(B)={⟨Bx,x⟩:‖x‖=1}`.

#### Statement

If `(σ,ν)` is outward support data for an open domain `Ω` and `W(B)⊆Ω`, then `σ∉W(B)`. Every unit `x` obeys
$$0\le\operatorname{Re}(\barν(σ-\langle Bx,x\rangle)).$$

#### Hypothesis ledger

Containment puts Rayleigh quotients in `Ω`; support supplies the sign. Openness and `σ∈∂Ω` give `σ∉Ω`, hence exclusion from `W(B)`. Convexity is construction context for obtaining support data in applications, not a hypothesis of this theorem.

#### Proof roadmap

Insert a Rayleigh quotient into the scalar support inequality. Then use the open-boundary distinction, separately from the non-strict inequality, to exclude `σ`.

#### Proof

We prove boundary exclusion together with unit-vector scalar half-plane separation from numerical-range containment. For `‖x‖=1`, set `z_x=⟨Bx,x⟩`. Then `z_x∈W(B)⊆Ω`, so
$$0\le\operatorname{Re}(\barν(σ-z_x))
=\operatorname{Re}(\barν(σ-\langle Bx,x\rangle)).$$
This can be equality. Exclusion uses `σ∈∂Ω` and openness: `σ∉Ω`; therefore `σ∉W(B)`. For the exterior point `ζ_t=σ+tν`, `t>0`, one does get strictness:
$$\operatorname{Re}(\barν(ζ_t-z_x))=t+\operatorname{Re}(\barν(σ-z_x))\ge t>0.$$

#### Worked instance

For `N` and `|σ|=R>1`,
$$\operatorname{Re}(\barν(σ-\langle Nx,x\rangle))
=R-\operatorname{Re}(\barν\langle Nx,x\rangle)\ge R-1>0.$$

#### Boundary case

At `R=1`, a unit vector realizes `⟨Nx,x⟩=σ`, so the support residual is zero. Here `W(N)` is not contained in the open unit disk, so this is an illustration of the lost margin, not an instance of the theorem's strict-containment hypotheses. Boundary support alone does not promise a positive margin.

#### Historical context

Supporting hyperplanes are classical convex analysis; Rayleigh quotients make them operator inequalities.
Source boundary: registered Crouzeix geometry declarations and standard finite-dimensional supporting-line theory only.
Review status: scalar sign, open-boundary exclusion, exterior strictness, and Lean provider were jointly checked.

#### ML analogy

Mathematical object: a half-plane certificate for every matrix Rayleigh quotient.
ML counterpart: a separator for all latent quadratic responses of a learned linear operator.
Exact transfer: verified response containment yields the same scalar residual inequality.
Non-transfer: finitely sampled directions do not certify the full numerical range.
Diagnostic: minimize the support residual and compare it with a Hermitian-part eigenvalue computation.

#### Pedagogical prerequisites

Complex inner products, unit vectors, numerical range, open-set boundary, and supporting lines.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.support_point_outside_numerical_range`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixConjecture.EuclideanVector, CrouzeixConjecture.OutwardBoundarySupport, CrouzeixConjecture.OutwardBoundarySupport.sigma_not_mem, CrouzeixConjecture.OutwardBoundarySupport.support_inequality, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.euclideanOperator, CrouzeixConjecture.numericalRange, CrouzeixConjecture.numericalRange._proof_1, CrouzeixConjecture.numericalRange._proof_2, CrouzeixConjecture.numericalRange._proof_3, CrouzeixConjecture.numericalRange._proof_4, CrouzeixConjecture.numericalRange._proof_5`.
Readable type map: `Omega,sigma,nu,B,x` are the domain, supported point, outward normal, matrix, and unit vector; the paired conclusion is `sigma ∉ numericalRange B` together with the scalar support inequality for every unit `x`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L15).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] {Omega : Set.{0} Complex} {sigma nu : Complex}, CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu → ∀ (B : CrouzeixConjecture.SquareMatrix.{u_1} n), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} B) Omega → And (Not (Membership.mem.{0, 0} (CrouzeixConjecture.numericalRange.{u_1} B) sigma)) (∀ (x : CrouzeixConjecture.EuclideanVector.{u_1} n), Eq.{1} (Norm.norm.{u_1} x) (OfNat.ofNat.{0} 1) → LE.le.{0} (OfNat.ofNat.{0} 0) (DFunLike.coe.{1, 1, 1} RCLike.re.{0} (HMul.hMul.{0, 0, 0} (Star.star.{0} nu) (HSub.hSub.{0, 0, 0} sigma (Inner.inner.{0, u_1} Complex x (DFunLike.coe.{u_1 + 1, u_1 + 1, u_1 + 1} (DFunLike.coe.{u_1 + 1, u_1 + 1, u_1 + 1} CrouzeixConjecture.euclideanOperator.{u_1} B) x))))))`.
Type SHA-256: `c81aa6c33f155d60b3ea289c9683b19f669b9b37f5b0fa0e1afbe8be12f42086`.
Direct maintained dependencies: `CrouzeixConjecture.EuclideanVector, CrouzeixConjecture.OutwardBoundarySupport, CrouzeixConjecture.OutwardBoundarySupport.sigma_not_mem, CrouzeixConjecture.OutwardBoundarySupport.support_inequality, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.euclideanOperator, CrouzeixConjecture.numericalRange, CrouzeixConjecture.numericalRange._proof_1, CrouzeixConjecture.numericalRange._proof_2, CrouzeixConjecture.numericalRange._proof_3, CrouzeixConjecture.numericalRange._proof_4, CrouzeixConjecture.numericalRange._proof_5`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: finite decidable matrix index, outward support data, and numerical-range containment.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:c81aa6c33f155d60b3ea289c9683b19f669b9b37f5b0fa0e1afbe8be12f42086`.

#### Exercises and solutions

CFT-26-E01 reconstructs the scalar support inequality directly without invoking the public exclusion theorem.

### CFT-26-002 — supported boundary resolvent {#cft-26-002}

#### Purpose

Prove that `(σI-B)^{-1}` exists before using inverse cancellation, while making the boundary argument honest.

#### Definitions and notation

The spectrum contains shifts for which `σI-B` is not invertible. In finite complex dimension, `spectrum(B)⊆W(B)`. Write `R_σ=(σI-B)^{-1}` after invertibility is established.

#### Statement

Outward support for an open convex domain `Ω` and containment `W(B)⊆Ω` imply that `σI-B` is an invertible matrix, represented in Lean by `IsUnit (σI-B)`.

#### Hypothesis ledger

Finite dimension supplies spectral inclusion. Openness excludes the supported boundary point from `Ω`. No resolvent norm estimate is asserted.

#### Proof roadmap

Give a kernel contradiction for an exterior shift, then isolate the boundary limit and replace lost strictness by open-domain exclusion plus spectral inclusion.

#### Proof

We contradict a kernel vector using strict separation; isolate the boundary limit. Let `ζ=σ+tν`, `t>0`, and suppose `(ζI-B)x=0` with `x≠0`. For `y=x/‖x‖`, `By=ζy`, whence
$$0<t\le\operatorname{Re}(\barν(ζ-\langle By,y\rangle))=0,$$
a contradiction. At `t=0` the strict term disappears, so this proof is not passed to the limit. Instead,
$$σ\notin Ω,\quad W(B)\subseteq Ω\quad\Longrightarrow\quad σ\notin W(B).$$
Then `spectrum(B)⊆W(B)` gives `σ∉spectrum(B)`, exactly the invertibility of `σI-B`.

#### Worked instance

For `N²=0`,
$$ (σI-N)(σ^{-1}I+σ^{-2}N)=I-σ^{-2}N^2=I,$$
which verifies the displayed resolvent whenever `σ≠0`.

#### Boundary case

On `|σ|=1`, the density for `N` is singular, but `σI-N` remains invertible because `spectrum(N)={0}`. These are different properties.

#### Historical context

An eigenvalue belongs to the numerical range by normalizing its eigenvector; this is the spectral fact used here.
Source boundary: registered numerical-range and spectrum declarations for finite complex matrices, with no pseudospectral estimate.
Review status: exterior kernel contradiction and boundary spectral argument were reviewed as separate logical cases.

#### ML analogy

Mathematical object: invertibility of a shifted nonnormal operator.
ML counterpart: solvability of a shifted linear system in an implicit layer or transfer function.
Exact transfer: spectral exclusion certifies a unique exact solve.
Non-transfer: invertibility alone does not control conditioning or floating-point error.
Diagnostic: report the smallest singular value together with the geometric support margin.

#### Pedagogical prerequisites

Eigenvectors and eigenvalues, the finite-matrix spectrum, inverse matrices, numerical-range containment, and the elementary topology of open domains and their boundaries.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.support_resolvent_invertible`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.scalar_sub_matrix_isUnit_of_outwardBoundarySupport`.
Substantive provider: `CrouzeixConjecture.scalar_sub_matrix_isUnit_of_outwardBoundarySupport`.
Readable type map: support and containment hypotheses return `IsUnit (sigma • 1 - B)`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L29).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] {Omega : Set.{0} Complex} {sigma nu : Complex}, CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu → ∀ (B : CrouzeixConjecture.SquareMatrix.{u_1} n), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} B) Omega → IsUnit.{u_1} (HSub.hSub.{u_1, u_1, u_1} (HSMul.hSMul.{0, u_1, u_1} sigma (OfNat.ofNat.{u_1} 1)) B)`.
Type SHA-256: `ba925bdd2b8af209941855dc71402f2fc0d602598e38bb205a1282955c8e5866`.
Direct maintained dependencies: `CrouzeixConjecture.scalar_sub_matrix_isUnit_of_outwardBoundarySupport`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: finite decidable matrix index, outward support data, and containment in the open domain.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:ba925bdd2b8af209941855dc71402f2fc0d602598e38bb205a1282955c8e5866`.

#### Exercises and solutions

CFT-26-E02 performs the scalar quadratic-form calculation, while CFT-26-E05 reconstructs this invertibility result.

### CFT-26-003 — positivity of the support matrix {#cft-26-003}

#### Purpose

Package scalar support geometry as a Hermitian PSD matrix, the form preserved by congruence and integration.

#### Definitions and notation

Set
$$S_{σ,ν}(B)=\barν(σI-B)+ν(\barσ I-B^*).$$
Write `M\succeq0` when `M` is Hermitian and `Re⟨Mx,x⟩≥0` for every `x`.

#### Statement

Supported-boundary geometry for an open convex domain, together with `W(B)⊆Ω`, implies that the Hermitian support matrix satisfies `S_{σ,ν}(B)\succeq0`.

#### Hypothesis ledger

The support inequality applies to unit vectors. Nonzero vectors are normalized and rescaled by `‖x‖²`; zero is separate. The adjoint term supplies Hermitianity.

#### Proof roadmap

Expand the quadratic form, identify a complex number plus its conjugate, apply scalar support, and restore arbitrary vector scale.

#### Proof

We prove support-resolvent positivity by an explicit quadratic-form congruence. The actual resolvent congruence comes in CFT-26-005; here we build its PSD source. For `‖x‖=1` and `q=\barν(σ-⟨Bx,x⟩)`,
$$
\langle Sx,x\rangle=q+\bar q=2\operatorname{Re}q\ge0.
$$
For `x≠0`, put `y=x/‖x‖`; then
$$\operatorname{Re}\langle Sx,x\rangle=\lVert x\rVert^2\operatorname{Re}\langle Sy,y\rangle\ge0.$$
The zero vector gives zero. Finally `S^*=S`, because taking the adjoint swaps the two displayed summands.

#### Worked instance

For `N`, `σ=Re^{it}`, `ν=e^{it}`,
$$S=\begin{pmatrix}2R&-2e^{-it}\\-2e^{it}&2R\end{pmatrix},$$
whose eigenvalues are `2R±2`.

#### Boundary case

Replacing the outward normal by `-ν` replaces `S` by `-S`. Orientation is mathematical data, not decoration.

#### Historical context

Hermitian parts classically turn scalar accretivity into operator positivity.
Source boundary: registered double-layer geometry declarations plus elementary adjoint and quadratic-form identities.
Review status: scalar factors, zero-vector normalization, adjoint convention, and Hermitian sign were checked explicitly.

#### ML analogy

Mathematical object: a PSD support matrix built from a nonnormal operator.
ML counterpart: a covariance-like energy matrix used to certify nonnegative quadratic loss.
Exact transfer: PSD means every exact feature direction has nonnegative energy.
Non-transfer: arbitrary similarity transforms do not preserve PSD; congruences do.
Diagnostic: compare the minimum eigenvalue with twice the minimum scalar support residual.

#### Pedagogical prerequisites

Adjoints, Hermitian matrices, PSD quadratic forms, and normalization of vectors.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.double_layer_support_positive`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.doubleLayerSupportMatrix_posSemidef_of_outwardBoundarySupport`.
Substantive provider: `CrouzeixConjecture.doubleLayerSupportMatrix_posSemidef_of_outwardBoundarySupport`.
Readable type map: support and containment yield `PosSemidef (doubleLayerSupportMatrix B sigma nu)`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L33).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] {Omega : Set.{0} Complex} {sigma nu : Complex}, CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu → ∀ (B : CrouzeixConjecture.SquareMatrix.{u_1} n), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} B) Omega → Matrix.PosSemidef.{u_1, 0} (CrouzeixConjecture.doubleLayerSupportMatrix.{u_1} B sigma nu)`.
Type SHA-256: `669c3561a5e2fa2329dbac180de86d6b8ac4f22a91c3d0ab2b827e9570dbd577`.
Direct maintained dependencies: `CrouzeixConjecture.doubleLayerSupportMatrix_posSemidef_of_outwardBoundarySupport`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: finite decidable index, outward support data, and numerical-range containment.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:669c3561a5e2fa2329dbac180de86d6b8ac4f22a91c3d0ab2b827e9570dbd577`.

#### Exercises and solutions

CFT-26-E03 proves the abstract congruence rule independently of this support-specific theorem.

### CFT-26-004 — oriented double-layer resolvent {#cft-26-004}

#### Purpose

Name the analytic resolvent and fix orientation, speed, and normalization before algebra begins.

#### Definitions and notation

Let `R_σ=(σI-B)^{-1}` and `D_{σ,ν}=νR_σ+\barνR_σ^*`. For positively oriented regular `γ`,
$$ν(t)=-i\frac{γ'(t)}{|γ'(t)|},\qquad |dσ|=|γ'(t)|dt.$$

#### Statement

The public formal declaration `double_layer_resolvent B sigma` is exactly
`(sigma I-B)⁻¹`. Separately, the registered parametric-boundary definitions
assemble that resolvent into the pulled-back density
$$K_γ(t)=\frac{|γ'(t)|}{2π}D_{γ(t),ν(t)}.$$

#### Hypothesis ledger

The definition takes only a finite square matrix and scalar shift. Inverse laws require the separate unit proof. The geometric wrapper needs positive orientation, nonzero speed, and `1/(2πi)`.

#### Proof roadmap

Rewrite the scalar Cauchy differential using `γ'=i|γ'|ν`, then add its Hermitian partner and keep unnormalised density separate from speed and `2π`.

#### Proof

We define the oriented normalized double-layer resolvent from scalar Cauchy data
in two compiler-checked layers: the public definition names the raw
resolvent, while the separate parametric definitions attach speed,
normalization, and orientation. To identify that normalized density with
scalar Cauchy data, the analytic term is
$$\frac1{2πi}R_{γ(t)}γ'(t)dt.$$
Positive orientation gives `γ'(t)=i|γ'(t)|ν(t)`, hence
$$\frac1{2πi}R_{γ(t)}γ'(t)dt
=\frac{|γ'(t)|}{2π}ν(t)R_{γ(t)}dt.$$
Adding the adjoint partner gives `K_γ(t)dt`. Lean checks the `speed/(2π)`
normalization in `parametricBoundaryFirstPart` and checks the Cauchy-integrand
identity for an `OrientedRadialConvexBoundary`; a generic
`ParametricConvexBoundary` deliberately carries point, normal, and speed but no
tangent equation or Cauchy formula.

#### Worked instance

For the nilpotent circle, speed is `R`, so
$$K_γ(t)=\frac1{2π}\begin{pmatrix}2&2/σ\\2/\barσ&2\end{pmatrix}.$$

#### Boundary case

If `γ'(t)=0`, the unit normal cannot be recovered from `-iγ'/|γ'|`. Chapter 25's positive-speed package rules this out.

#### Historical context

The terminology comes from potential theory; the resolvent term comes from Dunford--Riesz Cauchy calculus.
Source boundary: scalar Cauchy normalization and registered parametric-boundary definitions; no norm-two theorem is imported.
Review status: orientation sign, arclength speed, `2π` factor, and adjoint placement were independently recomputed.

#### ML analogy

Mathematical object: an oriented boundary-indexed resolvent feature.
ML counterpart: frequency-response features of a linear state-space or implicit model.
Exact transfer: the same shifted solve produces the same resolvent feature.
Non-transfer: discretization need not preserve orientation, arclength, or analytic normalization.
Diagnostic: for `B=0` on `[0,2π]` with measure `dt`, verify identity mass for the analytic first part and `2I` mass for its adjoint symmetrization; record the built-in `1/(2π)` exactly once.

#### Pedagogical prerequisites

Matrix inverse, adjoint, scalar Cauchy formula, regular curves, normals, and arclength.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.double_layer_resolvent`.
Formal mode: `definition`.
Underlying declaration: `CrouzeixConjecture.doubleLayerResolvent`.
Substantive provider: `CrouzeixConjecture.doubleLayerResolvent`.
Readable type map: a square matrix and complex shift map to a square-matrix resolvent.
Code: [Lean definition](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L37).
Supporting normalization definitions: [`parametricBoundaryFirstPart`](../../../formalization/lean/CrouzeixConjecture/ParametricBoundary.lean#L72) and [`parametricDoubleLayerDensity`](../../../formalization/lean/CrouzeixConjecture/ParametricBoundary.lean#L79).
Supporting oriented Cauchy bridge: [`parametricBoundaryFirstPart_eq_cauchyIntegrand`](../../../formalization/lean/CrouzeixConjecture/RadialContour.lean#L398).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `{n : Type u_1} → [Fintype.{u_1} n] → [DecidableEq.{u_1 + 1} n] → CrouzeixConjecture.SquareMatrix.{u_1} n → Complex → CrouzeixConjecture.SquareMatrix.{u_1} n`.
Type SHA-256: `2576ccb833b38fce237c0923d56df69609bf656489d5f61e0ab3427e4486ec09`.
Direct maintained dependencies: `CrouzeixConjecture.doubleLayerResolvent`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: finite decidable index; inverse identities are used only after proving the shift is a unit.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:2576ccb833b38fce237c0923d56df69609bf656489d5f61e0ab3427e4486ec09`.

#### Exercises and solutions

CFT-26-E04 expands the exact algebra connecting this resolvent to the support matrix and density.

### CFT-26-005 — full resolvent congruence {#cft-26-005}

#### Purpose

Transport the positive support matrix through the nonnormal resolvent without assuming any commutation or diagonalization.

#### Definitions and notation

Put `Q=σI-B` and `R=Q⁻¹`. The support matrix is `S=\barνQ+νQ*`, while the target double-layer density is the Hermitian sum `D=\barνR*+νR`.

#### Statement

Under support geometry and containment,
$$R^*SR=D.$$
Thus `S\succeq0` implies `D\succeq0` by congruence.

#### Hypothesis ledger

Invertibility gives `QR=I`; its adjoint gives `R*Q*=I`. Matrix order is never exchanged. Scalars commute with matrices, but matrices do not.

#### Proof roadmap

Distribute both support terms, reassociate, cancel the right inverse and its adjoint, then apply PSD preservation under congruence.

#### Proof

We write the matrix congruence in full, including adjoints and scalar factors. From `QR=I`,
$$R^*Q^*=(QR)^*=I.$$
Therefore
$$
\begin{aligned}
R^*SR
&=R^*(\barνQ+νQ^*)R\\
&=\barνR^*QR+νR^*Q^*R\\
&=\barνR^*(QR)+ν(R^*Q^*)R\\
&=\barνR^*+νR=D.
\end{aligned}
$$
For every `x`, `Re⟨R*SRx,x⟩=Re⟨S(Rx),Rx⟩≥0`, and adjoints show the result is Hermitian.

#### Worked instance

Substitution of the explicit `N` resolvent and support matrix yields
$$R_σ^*S R_σ=\begin{pmatrix}2/R&2/(Rσ)\\2/(R\barσ)&2/R\end{pmatrix},$$
including the sensitive off-diagonal powers.

#### Boundary case

For an approximate inverse `QR=I+E`, residual terms in `E` and `E*` remain. Exact PSD no longer follows automatically.

#### Historical context

Congruence preservation is elementary linear algebra; placing it around the resolvent is the double-layer method's structural step.
Source boundary: registered double-layer congruence and Mathlib adjoint algebra, excluding terminal Crouzeix bounds.
Review status: multiplication order, adjoint inverse, scalar conjugation, and both cancellations were checked line by line.

#### ML analogy

Mathematical object: PSD transport under the feature map `x↦R_σx`.
ML counterpart: covariance transport through a linear feature layer.
Exact transfer: `M↦R*MR` is exactly the covariance congruence and preserves PSD.
Non-transfer: the similarity `R⁻¹MR` is different and need not preserve Hermitian positivity.
Diagnostic: compute `‖R*SR-D‖` before interpreting sampled eigenvalues.

#### Pedagogical prerequisites

Associative matrix algebra, conjugate transpose, inverse identities, and PSD quadratic forms.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.double_layer_congruence`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.doubleLayerResolvent_congruence_density`.
Substantive provider: `CrouzeixConjecture.doubleLayerResolvent_congruence_density`.
Readable type map: support and containment imply the exact resolvent-support-resolvent density identity.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L41).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] {Omega : Set.{0} Complex} {sigma nu : Complex}, CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu → ∀ (B : CrouzeixConjecture.SquareMatrix.{u_1} n), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} B) Omega → Eq.{u_1 + 1} (HMul.hMul.{u_1, u_1, u_1} (HMul.hMul.{u_1, u_1, u_1} (Matrix.conjTranspose.{0, u_1, u_1} (CrouzeixConjecture.doubleLayerResolvent.{u_1} B sigma)) (CrouzeixConjecture.doubleLayerSupportMatrix.{u_1} B sigma nu)) (CrouzeixConjecture.doubleLayerResolvent.{u_1} B sigma)) (CrouzeixConjecture.doubleLayerDensity.{u_1} (CrouzeixConjecture.doubleLayerResolvent.{u_1} B sigma) nu)`.
Type SHA-256: `3ca440cf9415bc7043af96c8f7c9292238e56809c47f535ee2f896ccbb9006b1`.
Direct maintained dependencies: `CrouzeixConjecture.doubleLayerResolvent_congruence_density`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: finite decidable index, support data, and containment ensuring inverse laws.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:3ca440cf9415bc7043af96c8f7c9292238e56809c47f535ee2f896ccbb9006b1`.

#### Exercises and solutions

CFT-26-E05 reconstructs boundary invertibility; together E03--E05 expose every ingredient used by E06.

### CFT-26-006 — integration of the positive density {#cft-26-006}

#### Purpose

Close the analytic gap between pointwise PSD and a well-defined PSD matrix integral, including entrywise interchange.

#### Definitions and notation

For compact parametric boundary `Γ` and finite measure `μ`, write `K(x)=parametricDoubleLayerDensity Γ B x`. Its Bochner integral lies in the finite-dimensional matrix space.

#### Statement

If `W(B)⊆Ω`, then `K` is integrable, `∫K dμ\succeq0`, and
$$\left(\int K(x)dμ(x)\right)_{ab}=\int K(x)_{ab}dμ(x)$$
for every `a,b`.

#### Hypothesis ledger

Compactness, continuity, and finite measure supply integrability. Numerical-range containment supplies pointwise PSD. Coordinate evaluation is continuous linear, hence commutes with Bochner integration.

#### Proof roadmap

Use the density integrability theorem, integrate pointwise PSD, then factor entry evaluation into two continuous coordinate projections.

#### Proof

We integrate pointwise positivity with integrability and finite-sum interchange justified. Since `K∈L¹(μ)`, finite sums may pass through the integral. For every vector `v`,
$$
\begin{aligned}
\operatorname{Re}\left\langle\left(\int Kdμ\right)v,v\right\rangle
&=\operatorname{Re}\sum_{a,b}\bar v_a\left(\int K_{ab}dμ\right)v_b\\
&=\int\operatorname{Re}\sum_{a,b}\bar v_aK_{ab}(x)v_b\,dμ(x)\\
&=\int\operatorname{Re}\langle K(x)v,v\rangle dμ(x)\ge0.
\end{aligned}
$$
Hermitianity integrates entrywise. Formally, row evaluation `E_a` and coordinate evaluation `e_b` are continuous linear maps, so
$$e_bE_a\left(\int Kdμ\right)=\int e_bE_a(K(x))dμ(x),$$
which is precisely the entrywise equality.

#### Worked instance

For the nilpotent circle on `[0,2π]`, using Lebesgue measure `dt`,
$$K(t)=\frac1{2π}\begin{pmatrix}2&2/(Re^{it})\\2/(Re^{-it})&2\end{pmatrix}.$$
Uniform integration kills both oscillatory off-diagonal entries and yields `2I`.
The unsymmetrized analytic first part has half the diagonal and integrates to
`I`; these two mass statements use the same unnormalized measure and the same
built-in `1/(2π)` factor.

#### Boundary case

Pointwise PSD without measurability or integrability does not define a Bochner integral. A PSD quadrature sum also does not certify exact continuum mass.

#### Historical context

Bochner integration extends integration to Banach-valued functions; continuous linear maps commuting with it explains componentwise matrix integration.
Source boundary: Mathlib Bochner integration and registered parametric double-layer density theorems only.
Review status: integrability, integrated PSD, coordinate projections, and finite-index interchange were compiler checked.

#### ML analogy

Mathematical object: a Bochner average of PSD operator-valued features.
ML counterpart: a population covariance or kernel operator averaged from PSD contributions.
Exact transfer: exact expectations and positive weighted finite sums preserve PSD.
Non-transfer: Monte Carlo samples do not certify continuum integrability, mass, or worst-case behavior.
Diagnostic: track quadrature mass, minimum sampled eigenvalue, and entrywise convergence under refinement.

#### Pedagogical prerequisites

PSD matrices, measurable integrable functions, finite measures, Bochner integration, and coordinate maps.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.double_layer_density_positive`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.integrable_parametricDoubleLayerDensity, CrouzeixConjecture.integral_posSemidef, CrouzeixConjecture.numericalRange, CrouzeixConjecture.parametricDoubleLayerDensity, CrouzeixConjecture.parametricDoubleLayerDensity_posSemidef`.
Readable type map: `Gamma,B,mu` produce integrability, PSD of the integral, and equality with every entry integral.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L46).
Compiler receipt: fresh canonical compiler output joined to this card.
Normalized type: `∀ {i : Type u_1} {n : Type u_2} [inst : TopologicalSpace.{u_1} i] [CompactSpace.{u_1} i] [inst_2 : MeasurableSpace.{u_1} i] [OpensMeasurableSpace.{u_1} i] [inst_4 : Fintype.{u_2} n] [inst_5 : DecidableEq.{u_2 + 1} n] [Nonempty.{u_2 + 1} n] (mu : MeasureTheory.Measure.{u_1} i) [MeasureTheory.IsFiniteMeasure.{u_1} mu] {Omega : Set.{0} Complex} (Gamma : CrouzeixConjecture.ParametricConvexBoundary.{u_1} Omega) (B : CrouzeixConjecture.SquareMatrix.{u_2} n), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_2} B) Omega → And (MeasureTheory.Integrable.{u_2, u_1} (CrouzeixConjecture.parametricDoubleLayerDensity.{u_1, u_2} Gamma B) mu) (And (Matrix.PosSemidef.{u_2, 0} (MeasureTheory.integral.{u_1, u_2} mu fun x => CrouzeixConjecture.parametricDoubleLayerDensity.{u_1, u_2} Gamma B x)) (∀ (a b : n), Eq.{1} (MeasureTheory.integral.{u_1, u_2} mu (fun x => CrouzeixConjecture.parametricDoubleLayerDensity.{u_1, u_2} Gamma B x) a b) (MeasureTheory.integral.{u_1, 0} mu fun x => CrouzeixConjecture.parametricDoubleLayerDensity.{u_1, u_2} Gamma B x a b)))`.
Type SHA-256: `acfa9886104dc38842086a82b8fe579c01f2b5ebf959a35159b37f1b76cea4b4`.
Direct maintained dependencies: `CrouzeixConjecture.ParametricConvexBoundary, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.integrable_parametricDoubleLayerDensity, CrouzeixConjecture.integral_posSemidef, CrouzeixConjecture.numericalRange, CrouzeixConjecture.parametricDoubleLayerDensity, CrouzeixConjecture.parametricDoubleLayerDensity_posSemidef`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: compact measurable parameter space, finite measure, finite nonempty decidable index, parametric boundary, and numerical-range containment.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:acfa9886104dc38842086a82b8fe579c01f2b5ebf959a35159b37f1b76cea4b4`.

#### Exercises and solutions

CFT-26-E06 reconstructs pointwise density PSD without terminal providers; this theorem adds integrability and integration.

## Worked examples

The nilpotent calculation above is the chapter's normalization audit. It makes
the resolvent finite because `N²=0`, exposes both off-diagonal phases, and
shows that the density eigenvalues are `2/R±2/R²`. Three qualitative facts can
therefore be checked without abstraction: positivity is strict outside the
numerical-range disk, becomes singular at the boundary, and integration over
the circle removes the oscillatory off-diagonal terms. With `dt`, the analytic
first part integrates to `I` and its adjoint-symmetrized density integrates to
`2I`. With `dt/(2π)`, the corresponding raw analytic factor integrates to
`I`. These are dimensionally equivalent statements, not interchangeable
normalization conventions.

## ML bridge

For an ML researcher, the closest exact analogy is a PSD feature operator
transported through a nonnormal preconditioner and then averaged. Congruence
is the same algebra that preserves positivity of covariance matrices under a
linear feature map, while the Bochner integral is a population average of
operator-valued features. The analogy stops at certification: sampled
eigenvalues do not prove support containment, resolvent invertibility, exact
continuum mass, or integrability. A useful numerical diagnostic records the
support residual, the smallest eigenvalue of the sampled density, the
resolvent norm, and the quadrature mass separately; a failure in one quantity
must not be disguised by success in another.

## Lean translation

The six public declarations and six exercise declarations linked below are
compiler-bound surfaces. CFT-26-001 directly proves boundary exclusion together
with scalar support separation. CFT-26-002 through CFT-26-005 expose maintained
inversion, positivity, definition, and congruence results under the textbook
names. CFT-26-006 is proved in this chapter's Lean module by combining
integrability, pointwise PSD, positivity of the Bochner integral, and the
continuous coordinate projections that justify entrywise interchange. The
exercise declarations have distinct types and hashes; E06 reconstructs
pointwise density positivity from the lower-level exercise chain rather than
calling the terminal public density theorem. The visible locators, normalized
types, dependency lists, axioms, verification target, and receipt identities
on each card state exactly what the compiler checked.

## Exercises

### CFT-26-E01 — scalar support transport {#exercise-cft-26-e01}

Write the outward support inequality for W(B).

#### Complete written solution

We transport numerical-range containment into the scalar real-part inequality at every unit vector. For `‖x‖=1`, `z=⟨Bx,x⟩∈W(B)⊆Ω`, hence
$$0\le\operatorname{Re}(\barν(σ-z))=\operatorname{Re}(\barν(σ-\langle Bx,x\rangle)).$$
This is the requested direct proof.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_01_solution` in [Chapter26.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L75).
Formal mode: `proved-here`; the proof calls the scalar support inequality, not CFT-26-001.
Compiler locator: line 75, column 9.
Type SHA-256: `ad9414c12426fa0b86b5d53d95b51c8cc4ab9fa8704f994c534390026b9680cb`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:ad9414c12426fa0b86b5d53d95b51c8cc4ab9fa8704f994c534390026b9680cb`.

### CFT-26-E02 — unit-vector support quadratic form {#exercise-cft-26-e02}

Prove unit-vector nonnegativity of the support quadratic form.

#### Complete written solution

We expand the support quadratic form, insert the geometric support inequality, and obtain nonnegativity on every unit vector; Hermitianity and zero/nonzero rescaling are the separate upgrade to PSD. With `q=\barν(σ-⟨Bx,x⟩)`,
$$\operatorname{Re}\langle Sx,x\rangle=\operatorname{Re}(q+\bar q)=2\operatorname{Re}q\ge0.$$
The last inequality is E01. This is exactly the frozen Lean proposition: it
assumes `‖x‖=1` and proves nonnegativity for that `x`. E02 alone does not assert
that the whole matrix is PSD. In CFT-26-003 and E06, the adjoint identity
`S*=S` supplies Hermitianity; the zero vector is immediate; and for `x≠0`,
normalizing `y=x/‖x‖` and multiplying by `‖x‖²` upgrades the unit-vector result
to the all-vector quadratic-form condition required by PSD.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_02_solution` in [Chapter26.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L88).
Formal mode: `proved-here`; the proof performs the adjoint and real-part expansion explicitly.
Compiler locator: line 88, column 9.
Type SHA-256: `fa517c6187ce2ab93447446f63434b8d3eedaa99135d5b021040e843996e63f0`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:fa517c6187ce2ab93447446f63434b8d3eedaa99135d5b021040e843996e63f0`.

### CFT-26-E03 — PSD congruence {#exercise-cft-26-e03}

Prove matrix congruence preserves positive semidefiniteness.

#### Complete written solution

We establish Hermitianity and rewrite the quadratic form of BᴴMB as the quadratic form of M at Bx. If `M\succeq0`,
$$\operatorname{Re}\langle(B^*MB)x,x\rangle=\operatorname{Re}\langle M(Bx),Bx\rangle\ge0.$$
Also
$$(B^*MB)^*=B^*M^*B=B^*MB,$$
so the congruence is Hermitian as well as nonnegative.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_03_solution` in [Chapter26.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L129).
Formal mode: `proved-here`; its proposition is the abstract PSD congruence rule, distinct from every public card. The Lean proof unfolds the defining Hermitian and quadratic-form obligations and does not call `Matrix.PosSemidef.conjTranspose_mul_mul_same` or the project wrapper.
Compiler locator: line 129, column 9.
Type SHA-256: `a249c97397a895cd7686c23e9d18b972a390f48e7ebe52ee6751d3cfd4140baf`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:a249c97397a895cd7686c23e9d18b972a390f48e7ebe52ee6751d3cfd4140baf`.

### CFT-26-E04 — expanded density identity {#exercise-cft-26-e04}

Expand the double-layer resolvent congruence algebraically.

#### Complete written solution

We use the right-inverse identity, its adjoint, and all scalar factors to identify the double-layer density. If `Q=σI-B` and `QR=I`, then `R*Q*=I`, so
$$R^*(\barνQ+νQ^*)R=\barνR^*(QR)+ν(R^*Q^*)R=\barνR^*+νR.$$
No matrices are commuted.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_04_solution` in [Chapter26.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L142).
Formal mode: `proved-here`; the proof derives the adjoint inverse and cancels both support terms.
Compiler locator: line 142, column 9.
Type SHA-256: `1213c8d0db5a5822300d7344cc97899f0843943f1b0d97310ed8cdaf69452b27`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:1213c8d0db5a5822300d7344cc97899f0843943f1b0d97310ed8cdaf69452b27`.

### CFT-26-E05 — boundary invertibility {#exercise-cft-26-e05}

Explain why a supported boundary point gives resolvent invertibility.

#### Complete written solution

We turn a hypothetical spectral/kernel witness into a numerical-range witness and contradict strict containment. Openness gives `σ∉Ω`; containment gives `σ∉W(B)`. Since `spectrum(B)⊆W(B)`, `σ∉spectrum(B)`, hence `σI-B` is invertible. For an exterior shift a normalized kernel vector contradicts a strict support margin; at the boundary the open-domain argument replaces that lost margin.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_05_solution` in [Chapter26.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L159).
Formal mode: `proved-here`; the proof uses open-boundary exclusion and spectral inclusion directly.
Compiler locator: line 159, column 9.
Type SHA-256: `d80cfa6b4f3279d04d4d2122d027233a345d27ffb43c4177ebdb91cf0a3a0997`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:d80cfa6b4f3279d04d4d2122d027233a345d27ffb43c4177ebdb91cf0a3a0997`.

### CFT-26-E06 — reconstruct density positivity {#exercise-cft-26-e06}

Reconstruct positivity of the double-layer resolvent density.

#### Complete written solution

We compose support positivity, resolvent inversion, the exact congruence, and congruence preservation without calling the terminal density theorem. E02 plus normalization proves `S\succeq0`; E05 supplies `R`; E04 gives `D=R*SR`; E03 gives `R*SR\succeq0`. Therefore `D\succeq0`, with every dependency exposed.

Lean solution: `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_06_solution` in [Chapter26.lean](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L177).
Formal mode: `proved-here`; the proof reconstructs the density result from E02--E05 and avoids terminal density providers.
Compiler locator: line 177, column 9.
Type SHA-256: `ab76766d5649e5bfb7842181fabf6b97389fbb4aa81464c960e16964bcd0a7b8`.
Axioms: `Classical.choice, Quot.sound, propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:ab76766d5649e5bfb7842181fabf6b97389fbb4aa81464c960e16964bcd0a7b8`.

## Synthesis and forward dependencies

Support lines control scalar Rayleigh quotients; openness and spectral inclusion justify the boundary resolvent; the Hermitian support matrix packages the sign; congruence transports it through a nonnormal inverse; and Bochner integration preserves order. Chapter 27 combines this positivity with a triangle estimate, while Chapter 28 uses simultaneous powers to reach the sharp constant two.
