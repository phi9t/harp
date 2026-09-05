---
id: cft-chapter-25-convex-boundaries-and-cauchy-layers
title: Convex boundaries and Cauchy layers
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-28
tags: [crouzeix-textbook, crouzeix-machinery, mathematics, lean]
confidence: high
canonical: 25_convex_boundaries_and_cauchy_layers.md
chapter: 25
part: 5
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 25: Convex boundaries and Cauchy layers

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-v-crouzeix-machinery|Part V — Crouzeix machinery]]
Previous: [[knowledge/crouzeix_textbook/part_04_operator_theory/24_gramians_and_ordered_matrix_inequalities|Chapter 24]]
Next: [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/26_double_layer_map|Chapter 26]]

## Opening problem

The numerical range is compact and convex, but its boundary may have corners. We therefore enlarge it by a positive radius, do contour analysis on an exact oriented radial `C¹` boundary package, and let the radius tend to zero. Throughout, `⟨u,v⟩_ℝ=\operatorname{Re}⟨u,v⟩_ℂ`. Our running nonnormal example is

$$A_{λ,α}=\begin{pmatrix}\lambda&\alpha\\0&-\lambda\end{pmatrix},\qquad
h_{W(A)}(\theta)=\tfrac12\sqrt{4\operatorname{Re}(e^{-i\theta}\lambda)^2+|\alpha|^2}.$$

We derive this formula below rather than treating it as a fact. It identifies a centered ellipse, a disk, or a degenerate segment, and positive thickening shifts its support function by the outer radius.

## Conceptual model

Projection converts convex geometry into a stable point-valued map. Parallel thickening then replaces a possibly cornered numerical range by controlled outer domains: their Hausdorff error is governed by one scalar radius, while their oriented radial `C¹` data supplies exactly the contour interface used in Chapter 26.

## Running example: derive the ellipse and its projection

For a unit vector `u`, the support function of a numerical range is the largest Rayleigh quotient of the Hermitian part:
$$
h_{W(A)}(\theta)
=\max_{\|u\|=1}\operatorname{Re}\langle e^{-i\theta}Au,u\rangle
=\lambda_{\max}(\operatorname{Re}(e^{-i\theta}A)).
$$
For `A=A_{\lambda,\alpha}`, put `a_\theta=\operatorname{Re}(e^{-i\theta}\lambda)`. Direct matrix algebra gives
$$
\operatorname{Re}(e^{-i\theta}A_{\lambda,\alpha})
=\frac{e^{-i\theta}A_{\lambda,\alpha}+e^{i\theta}A_{\lambda,\alpha}^{*}}2
=\begin{pmatrix}
a_\theta&\tfrac12e^{-i\theta}\alpha\\
\tfrac12e^{i\theta}\overline\alpha&-a_\theta
\end{pmatrix}.
$$
Its characteristic polynomial is
$$\mu^2-\left(a_\theta^2+\frac{|\alpha|^2}{4}\right),$$
so
$$
h_{W(A)}(\theta)
=\lambda_{\max}(\operatorname{Re}(e^{-i\theta}A_{\lambda,\alpha}))
=\sqrt{a_\theta^2+\frac{|\alpha|^2}{4}}.
$$
If `\lambda\ne0` and `\phi=\arg\lambda`, then `a_\theta=|\lambda|\cos(\theta-\phi)`. Hence the ellipse is centered at zero, its principal axis is oriented by `e^{i\phi}`, and its semiaxes are
$$a=\sqrt{|\lambda|^2+\frac{|\alpha|^2}{4}},\qquad b=\frac{|\alpha|}{2}.$$
Only `|\alpha|` appears: the complex phase of `\alpha` changes a matrix representation but not this numerical-range ellipse. If `\lambda=0`, the result is a disk of radius `|\alpha|/2` and orientation is irrelevant. If `\alpha=0`, it is the degenerate segment `[-\lambda,\lambda]`; if both vanish, it is one point.

The support formula also gives a usable projection characterization. In the nondegenerate case `\alpha\ne0`, let
$$n(\theta)=e^{i\theta},\qquad
x(\theta)=n(\theta)\bigl(h(\theta)+i h'(\theta)\bigr).$$
Then `x(\theta)` is the unique supported boundary point with outward unit normal `n(\theta)`. A point `z` outside the ellipse has projection `x(\theta)` exactly when the unique pair `(\theta,t)` satisfies
$$z=x(\theta)+t\,n(\theta),\qquad t\ge0.$$
This follows from the variational inequality: `z-x(\theta)` must be an outward normal, and conversely a nonnegative normal residual has nonpositive inner product with every feasible chord. Solving this single scalar normal equation is usually clearer and more stable than printing the equivalent quartic formula.

The special cases are explicit. For the disk of radius `b`, `P(z)=z` when `|z|\le b` and `P(z)=b z/|z|` otherwise. For the degenerate segment with `\lambda\ne0`, set `u=\lambda/|\lambda|`; then
$$P(z)=u\,\operatorname{clip}\!\left(\operatorname{Re}(\overline u z),- |\lambda|,|\lambda|\right).$$
Finally, thickening by `\varepsilon_k` replaces `h` by `h_k=h+\varepsilon_k`; in the smooth case it replaces the boundary point by `x_k(\theta)=x(\theta)+\varepsilon_k n(\theta)`. Thus the same outward-normal rays describe both projection and outer-offset geometry.

## Formal development

### CFT-25-001 — convex projection {#cft-25-001}

#### Purpose

Metric projection turns closed convex geometry into a point-valued operation. The proof keeps two ideas separate: finite-dimensional properness and closedness supply a minimizer, while convexity supplies uniqueness.

#### Definitions and notation

For a nonempty closed real-convex set `K⊆ℂ` and `z∈ℂ`, a projection is `p∈K` such that `‖z-p‖≤‖z-w‖` for all `w∈K`. We denote this unique point by `P_Kz`.

#### Statement

There exists exactly one nearest point: `∃!p, p∈K ∧ ∀w∈K, ‖z-p‖≤‖z-w‖`. Every topological and convexity hypothesis is explicit in both prose and Lean.

#### Hypothesis ledger

Nonemptiness makes the distance infimum finite. Finite-dimensional properness makes a bounded closed sublevel compact, closedness keeps its limit in `K`, and convexity excludes distinct minimizers. No boundedness of `K` and no boundary differentiability are assumed.

#### Proof roadmap

Choose a bounded minimizing sequence, pass to a convergent subsequence in a compact sublevel, use closedness at the limit, and then exclude two minimizers with the midpoint and parallelogram identity.

#### Proof

We prove closed-convex nearest-point existence and separate existence from uniqueness. Put
$$d=\inf_{w\in K}\|z-w\|.$$
Choose `a∈K`. Since `0≤d≤‖z-a‖`, `d` is finite. For each `n≥1`, the definition of infimum gives `w_n∈K` with
$$d\le \|z-w_n\|<d+\frac1n\le d+1.$$
Thus the tail of `(w_n)` lies in the closed ball `\overline B(z,d+1)`. The complex plane is finite-dimensional, hence proper: closed bounded sets are compact. Therefore
$$K\cap\overline B(z,d+1)$$
is a proper-space compact sublevel (closedness of `K` is used here). A subsequence converges to some `p` in this intersection. Closedness gives `p∈K`—in other words, closedness keeps the limit in K—and continuity of the norm gives
$$\|z-p\|=\lim_j\|z-w_{n_j}\|=d.$$
This proves existence even though `K` itself may be unbounded.

For uniqueness, suppose `p,q∈K` both attain the same minimum `d`. Convexity puts the midpoint `m=(p+q)/2` in `K`. The parallelogram identity gives
$$\|z-m\|^2=\tfrac12\|z-p\|^2+\tfrac12\|z-q\|^2-\tfrac14\|p-q\|^2
=d^2-\tfrac14\|p-q\|^2.$$
Minimality also gives `d²≤‖z-m‖²`, so `‖p-q‖²≤0`; hence `p=q`. Thus existence and uniqueness use different hypotheses: closedness plus finite-dimensional properness for existence, convexity for uniqueness. Lean packages the existence step through completeness (`IsClosed.isComplete`) and Mathlib's complete-convex minimizer theorem, then reconstructs uniqueness locally from the equivalent pair of variational inequalities.

#### Worked instance

For the disk `|w|≤r`, an interior point projects to itself and an exterior point projects to `(r/|z|)z`. Exercise E02 certifies this radial picture by inequalities.

#### Boundary case

An open disk may fail to attain the distance infimum, showing why closedness or compactness matters. Two isolated equidistant points show why existence alone does not imply uniqueness without convexity.

#### Historical context

This is the finite-dimensional Hilbert projection theorem. In an arbitrary Hilbert space, closedness supplies completeness but closed bounded balls need not be compact; the usual proof instead shows a minimizing sequence is Cauchy by the parallelogram law.
Source boundary: Mathlib 4.32.1 projection declarations and the registered Crouzeix geometry packet.
Review status: existence/uniqueness split, hypotheses, compiler locator, and exercise proof jointly reviewed.

#### ML analogy

Mathematical object: the unique Euclidean nearest point in a nonempty closed convex feasible set.
ML counterpart: an exact projected-gradient update onto a convex parameter constraint.
Exact transfer: exact convex projection is single-valued and satisfies the same distance minimization.
Non-transfer: projection onto a nonconvex model family may be multivalued or discontinuous.
Diagnostic: two distinct exact outputs indicate a failed convexity or optimality assumption.

#### Pedagogical prerequisites

The proof uses continuous functions on compact sets, infima, convex segments, and the real inner product on the complex plane. It requires no operator calculus or measure theory.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.convex_projection`.
Formal mode: `proved-here`.
Substantive provider: `none` among maintained Crouzeix declarations; the proof uses Mathlib `exists_norm_eq_iInf_of_complete_convex` with `IsClosed.isComplete`, and proves uniqueness locally.
Readable type map: `K,z,p,w` are exactly the set, input, unique projection, and competitors above.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L14).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ (K : Set.{0} Complex), Set.Nonempty.{0} K → IsClosed.{0} K → Convex.{0, 0} Real K → ∀ (z : Complex), ExistsUnique.{1} fun p => And (Membership.mem.{0, 0} K p) (∀ (w : Complex), Membership.mem.{0, 0} K w → LE.le.{0} (Norm.norm.{0} (HSub.hSub.{0, 0, 0} z p)) (Norm.norm.{0} (HSub.hSub.{0, 0, 0} z w)))`.
Type SHA-256: `4ffef37c950b39279846aeb028511a0d8f18a945e1d8888379d849d8096a6100`.
Direct maintained dependencies: `none`; the proof descends directly to Mathlib's complete-convex minimizer theorem and elementary inner-product algebra.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: nonempty closed real-convex `K`, with arbitrary `z∈ℂ`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:4ffef37c950b39279846aeb028511a0d8f18a945e1d8888379d849d8096a6100`.

#### Exercises and solutions

CFT-25-E01 reconstructs closed-set existence and the minimizing inequality without calling CFT-25-001. Its distinct checked theorem and complete written proof occur below.

### CFT-25-002 — projection variational inequality {#cft-25-002}

#### Purpose

The variational inequality replaces nonlinear norm comparison by a signed inner product. It is the precise outward-normal statement consumed by later support geometry.

#### Definitions and notation

At the mathematical level, let `p∈K` be any attained minimizer of distance
from `z` to a convex set `K`. Choose `w∈K`, use the feasible segment
`p+t(w-p)`, and define `g(t)=‖z-p-t(w-p)‖²` for `0≤t≤1`. The exact Lean
theorem below assumes that `K` is nonempty, compact, and real-convex because
its named `convexProjection` provider is defined with those data.

#### Statement

For every `w∈K`, `⟨z-p,w-p⟩_ℝ≤0`. The segment argument itself needs only
convexity and an attained minimizer. CFT-25-001 supplies the broader
closed-convex existence theorem in `ℂ`; the attached public Lean declaration
checks the compact-provider specialization. Thus the residual `z-p` is an
outward normal in the real two-dimensional geometry underlying `ℂ`.

#### Hypothesis ledger

Convexity makes the segment feasible, minimality gives `g(0)≤g(t)`, and the
squared norm has an elementary quadratic expansion. Compactness constructs the
formal provider's minimizer but is not used after that minimizer is fixed. Zero
is an endpoint, not an interior point.

#### Proof roadmap

Expand `g(t)-g(0)`, divide by positive `t`, and send `t` down to zero. The resulting one-sided derivative has a sign rather than being zero.

#### Proof

We differentiate squared distance along a convex segment and handle the endpoint. For `0<t≤1`,
$$0\le g(t)-g(0)=-2t\langle z-p,w-p\rangle_{\mathbb R}+t^2\|w-p\|^2.$$
Division by `t` gives `2⟨z-p,w-p⟩_ℝ≤t‖w-p‖²`; letting `t↓0` proves the claim. Equivalently `g'_+(0)≥0`; asserting an endpoint derivative equals zero would be wrong.

#### Worked instance

For an exterior point of the radius-`r` disk, `p=(r/|z|)z`; Cauchy–Schwarz and `|w|≤r` verify the inequality and hence the radial formula.

#### Boundary case

If `z∈K`, then `p=z` and every inequality is equality. On a flat face, nonzero tangential directions may also give equality, so strict curvature is unnecessary.

#### Historical context

This is the classical normal-cone characterization of a convex projection, specialized to the real Hilbert structure on `ℂ`.
Source boundary: Mathlib 4.32.1 variational theorem and registered finite-dimensional projection foundations.
Review status: segment expansion, endpoint sign, exact provider, and radial application reviewed.

#### ML analogy

Mathematical object: a first-order variational inequality for a constrained quadratic minimum.
ML counterpart: the normal-cone or KKT condition for an exact proximal subproblem.
Exact transfer: every feasible direction has the required signed directional derivative.
Non-transfer: approximate projection introduces an error term that must be bounded.
Diagnostic: sample feasible directions and check the signed residual inner products.

#### Pedagogical prerequisites

Readers need convex segments, the polarization expansion of a squared norm, and one-sided limits. Optimization terminology is optional because all algebra is displayed.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.convex_projection_variational`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.convexProjection_variational`.
Substantive provider: `CrouzeixConjecture.convexProjection_variational`.
Readable type map: feasible `w` produces the real-inner-product inequality at the chosen projection.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L47).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ (K : Set.{0} Complex) (hKne : Set.Nonempty.{0} K) (hKcompact : IsCompact.{0} K) (hKconvex : Convex.{0, 0} Real K) (z w : Complex), Membership.mem.{0, 0} K w → LE.le.{0} (Inner.inner.{0, 0} Real (HSub.hSub.{0, 0, 0} z (CrouzeixConjecture.convexProjection K hKne hKcompact hKconvex z)) (HSub.hSub.{0, 0, 0} w (CrouzeixConjecture.convexProjection K hKne hKcompact hKconvex z))) (OfNat.ofNat.{0} 0)`.
Type SHA-256: `e5ff9225c14b24a0bc391ce2d438e1f6d80e2f0e1fe4075becb75d5ac2fe160a`.
Direct maintained dependencies: `CrouzeixConjecture.convexProjection_variational`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: nonempty compact real-convex `K`, arbitrary `z`, and feasible `w`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:e5ff9225c14b24a0bc391ce2d438e1f6d80e2f0e1fe4075becb75d5ac2fe160a`.

#### Exercises and solutions

CFT-25-E02 computes disk projection from the variational condition, with inside/outside cases and a distinct compiler-checked proof below.

### CFT-25-003 — nonexpansiveness {#cft-25-003}

#### Purpose

Projection must not amplify input perturbations. We derive this stability from two variational inequalities so the cancellation and zero-distance case remain visible.

#### Definitions and notation

Put `p=P_Kx`, `q=P_Ky`, and `d=p-q`. Use `q` as competitor for `p`, and `p` as competitor for `q`, always in the real inner product.

#### Statement

For all `x,y`, `‖P_Kx-P_Ky‖≤‖x-y‖`. The proof first obtains the stronger firm estimate `‖d‖²≤⟨x-y,d⟩_ℝ`.

#### Hypothesis ledger

The same nonempty compact convex set serves both projections. Variational inequalities provide signs; real Cauchy–Schwarz provides the upper estimate.

#### Proof roadmap

Add the two signed inequalities, rearrange into a squared norm, apply Cauchy–Schwarz, and split `‖d‖=0` before cancelling the factor.

#### Proof

We add both variational inequalities and apply Cauchy--Schwarz:
$$\|p-q\|^2\le\langle x-y,p-q\rangle_{\mathbb R}\le\|x-y\|\,\|p-q\|.$$
If `p=q`, the claim is immediate. Otherwise the positive factor `‖p-q‖` can be cancelled, giving `‖p-q‖≤‖x-y‖`. The explicit branch prevents division by zero.

#### Worked instance

Radial clipping onto a disk remains one-Lipschitz even when one point crosses the circle. The abstract inequality proves this without polar-coordinate case proliferation.

#### Boundary case

Equality holds when both inputs lie in `K`, because projection is the identity there. Hence one-Lipschitz is sharp and generally cannot be strengthened to contraction.

#### Historical context

Firm nonexpansiveness is a standard Hilbert projection property and later became central in monotone-operator splitting.
Source boundary: Mathlib 4.32.1 nonexpansive projection provider and elementary Cauchy–Schwarz.
Review status: sign algebra, zero branch, exact dependency, and exercise separation reviewed.

#### ML analogy

Mathematical object: a firmly nonexpansive map onto a convex feasible set.
ML counterpart: a projection layer in constrained training or inference.
Exact transfer: Euclidean perturbations cannot be amplified by exact projection.
Non-transfer: learned approximate projectors need not preserve the Lipschitz guarantee.
Diagnostic: a measured ratio `‖P(x)-P(y)‖/‖x-y‖>1` exposes a failure.

#### Pedagogical prerequisites

This proof needs the previous variational inequality, bilinearity, Cauchy–Schwarz, and safe cancellation. It is complete at undergraduate linear-algebra level.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.convex_projection_nonexpansive`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.norm_convexProjection_sub_le`.
Substantive provider: `CrouzeixConjecture.norm_convexProjection_sub_le`.
Readable type map: two points are projected onto one set and output distance is bounded by input distance.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L56).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ (K : Set.{0} Complex) (hKne : Set.Nonempty.{0} K) (hKcompact : IsCompact.{0} K) (hKconvex : Convex.{0, 0} Real K) (x y : Complex), LE.le.{0} (Norm.norm.{0} (HSub.hSub.{0, 0, 0} (CrouzeixConjecture.convexProjection K hKne hKcompact hKconvex x) (CrouzeixConjecture.convexProjection K hKne hKcompact hKconvex y))) (Norm.norm.{0} (HSub.hSub.{0, 0, 0} x y))`.
Type SHA-256: `6491d411e634da59fa43f86b95c536274c4f53474e95382ada98b49cc76f6fc5`.
Direct maintained dependencies: `CrouzeixConjecture.norm_convexProjection_sub_le`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: nonempty compact real-convex `K`, with arbitrary inputs `x,y`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:6491d411e634da59fa43f86b95c536274c4f53474e95382ada98b49cc76f6fc5`.

#### Exercises and solutions

CFT-25-E03 isolates uniqueness algebra: two variational inequalities force a squared distance to vanish, without invoking the uniqueness provider.

### CFT-25-004 — outer approximation radius {#cft-25-004}

#### Purpose

Contour work needs a genuine outer neighborhood and one fixed bound for all indices. A canonical positive radius sequence makes both facts explicit.

#### Definitions and notation

Write
$$\varepsilon_k=\operatorname{outerApproximationRadius}(k)=\frac1{k+1}$$
and `Ω_k={z:dist(z,K)<ε_k}`. Here `k` is coerced to a real number in the denominator.

#### Statement

For each natural `k`, `0<ε_k≤1`. Positivity gives strict enlargement; the upper bound places all closures in the closed one-thickening of `K`.

#### Hypothesis ledger

This scalar theorem assumes only `k∈ℕ`; no set hypothesis is hidden. Compactness enters only when the geometric convergence of a specific `K` is considered.

#### Proof roadmap

Pair the independently proved positivity and upper-bound facts. Then interpret them as strict outer containment and a common neighborhood for later compactness arguments.

#### Proof

We define the parallel outer radius and its geometric containment data. Because `k≥0`, we have `k+1≥1>0`; reciprocals reverse the positive inequality and give
$$0<\varepsilon_k\le1.$$
For decay, let `δ>0`. The Archimedean property supplies `N` with `N+1>1/δ`. Thus, whenever `k≥N`,
$$0<\varepsilon_k=\frac1{k+1}\le\frac1{N+1}<\delta.$$
This is the elementary `ε`--`N` proof that
$$\varepsilon_k\longrightarrow0.$$
Geometrically, `dist(z,K)<ε_k` implies `dist(z,K)<1`, while strict positivity prevents the construction from degenerating to the raw boundary.

#### Worked instance

For `W(A_{λ,α})`, thickening shifts every support value: `h_k(θ)=h_W(θ)+ε_k`. For `λ=0,α=2`, the unit disk becomes radius `1+ε_k`.

#### Boundary case

Radius zero leaves corners exposed, while an unbounded radius sequence destroys a single compact dominating neighborhood. Both inequalities perform real proof work.

#### Historical context

Parallel bodies formed by Minkowski addition with Euclidean balls are classical tools of convex approximation.
Source boundary: registered Crouzeix parallel-domain formalization and Mathlib metric thickening primitives.
Review status: scalar bounds, geometric interpretation, ellipse example, and exact compiler receipt reviewed.

#### ML analogy

Mathematical object: positive bounded geometric enlargements of a feasible set.
ML counterpart: a certified robustness-radius or continuation schedule.
Exact transfer: a shared upper radius provides one neighborhood for uniform bounds.
Non-transfer: random augmentation does not necessarily cover a metric thickening.
Diagnostic: audit positivity and a global upper bound separately from decay.

#### Pedagogical prerequisites

Only distance to a set, metric thickenings, and elementary inequalities are needed. Support functions are used solely to interpret the running ellipse.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.outer_approximation_radius`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixConjecture.outerApproximationRadius, CrouzeixConjecture.outerApproximationRadius_le_one, CrouzeixConjecture.outerApproximationRadius_pos`.
Readable type map: every natural index has a strictly positive radius at most one.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L65).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ (k : Nat), And (LT.lt.{0} (OfNat.ofNat.{0} 0) (CrouzeixConjecture.outerApproximationRadius k)) (LE.le.{0} (CrouzeixConjecture.outerApproximationRadius k) (OfNat.ofNat.{0} 1))`.
Type SHA-256: `b2294ee6be90f9259c5d55fcb89f02182b9ba7feeeb8124179e51fabf7056ce8`.
Direct maintained dependencies: `CrouzeixConjecture.outerApproximationRadius, CrouzeixConjecture.outerApproximationRadius_le_one, CrouzeixConjecture.outerApproximationRadius_pos`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: natural index only; no geometric set is suppressed.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:b2294ee6be90f9259c5d55fcb89f02182b9ba7feeeb8124179e51fabf7056ce8`.

#### Exercises and solutions

CFT-25-E04 reconstructs nonexpansiveness from two abstract variational inequalities, the stability input reused by the boundary geometry.

### CFT-25-005 — radius and Hausdorff convergence {#cft-25-005}

#### Purpose

Scalar decay alone cannot justify convergence of maxima on moving domains. We record Hausdorff convergence and one fixed compact neighborhood explicitly.

#### Definitions and notation

For compact `K`, let `Ω_k=parallelOuterDomain K k`, with `ε_k=1/(k+1)`. Hausdorff distance measures the worst nearest-set error, and `cthickening 1 K` is the common closed neighborhood.

#### Statement

The radii satisfy `ε_k→0`, `d_H(closure Ω_k,K)→0`, and every `closure Ω_k` lies in `cthickening 1 K`. These are uniform set-level assertions, not pointwise samples.

#### Hypothesis ledger

Compactness of `K` is explicit and ensures the fixed closed thickening is compact in `ℂ`. Uniform continuity of later functions follows on that set.

#### Proof roadmap

Combine scalar radius decay with the parallel-body Hausdorff estimate, use `ε_k≤1` for common containment, then transfer continuous maxima by uniform continuity.

#### Proof

We prove outer-radius decay with compactness and uniformity explicit. For every `k`, the parallel-body estimate is
$$0\le d_H(\overline{\Omega_k},K)\le\varepsilon_k.$$
The left inequality is nonnegativity of Hausdorff distance. For the right inequality, every `x∈\overline{\Omega_k}` has some `w∈K` with `\|x-w\|≤\varepsilon_k`, while every `w∈K` lies in `\Omega_k` because `\operatorname{dist}(w,K)=0<\varepsilon_k`. These are the two directed Hausdorff inequalities; together they give the displayed upper bound. Since `\varepsilon_k\to0`, the squeeze theorem gives
$$d_H(\overline{\Omega_k},K)\longrightarrow0.$$
The exact closure formula and `0<\varepsilon_k≤1` give the fixed-neighborhood inclusions, for every compact `K` and every `k`:
$$K\subseteq\Omega_k\subseteq\overline{\Omega_k}
=\operatorname{cthickening}(\varepsilon_k,K)
\subseteq\operatorname{cthickening}(1,K).$$
The last set is compact because `K` is compact. If `K` is also nonempty and `f` is continuous on that fixed compact set, then `|f|` is uniformly continuous there. Given `η>0`, choose `δ>0` for uniform continuity and then `k` with `d_H(\overline{\Omega_k},K)<δ`. Every `x∈\overline{\Omega_k}` has `y∈K` with `|x-y|<δ`, so
$$\max_{\overline{\Omega_k}}|f|\le\max_K|f|+\eta;
\qquad
\max_K|f|\le\max_{\overline{\Omega_k}}|f|$$
by `K⊆\overline{\Omega_k}`. Thus the compact-set maxima converge. This maximum-transfer corollary uses nonemptiness in addition to the hypotheses of the Lean theorem displayed in this card.

#### Worked instance

For the running ellipse, `h_k=h+ε_k` uniformly in angle, so the Hausdorff error vanishes uniformly and polynomial-modulus maxima return to `W(A_{λ,α})`.

#### Boundary case

Without a common compact neighborhood, escaping points can defeat convergence of continuous maxima. Polygonal `K` is allowed; corners remain only in the limit.

#### Historical context

Hausdorff convergence is the standard topology on compact convex bodies and corresponds to uniform support-function convergence.
Source boundary: registered parallel-domain Hausdorff theorems plus compact uniform-continuity arguments.
Review status: compactness scope, uniform quantifiers, maximum transfer, and Lean contract reviewed.

#### ML analogy

Mathematical object: Hausdorff-convergent feasible sets under compact domination.
ML counterpart: a vanishing robust-optimization envelope around a compact constraint set.
Exact transfer: uniformly continuous worst-case objectives converge under these hypotheses.
Non-transfer: pointwise sampled perturbations do not imply Hausdorff convergence.
Diagnostic: measure worst nearest-set error and shared boundedness, not average radius.

#### Pedagogical prerequisites

Readers need compactness in the plane, uniform continuity, Hausdorff distance, and supremum comparison. The attached exercise makes the corner obstruction precise.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.outer_radius_tends_to_zero`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixConjecture.outerApproximationRadius, CrouzeixConjecture.parallelOuterDomain, CrouzeixConjecture.parallelOuterDomain_closure_subset_fixedNeighborhood, CrouzeixConjecture.tendsto_hausdorffDist_parallelOuterDomain_closure, CrouzeixConjecture.tendsto_outerApproximationRadius`.
Readable type map: compact `K` receives scalar decay, Hausdorff decay, and fixed-neighborhood containment.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L71).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ (K : Set.{0} Complex), IsCompact.{0} K → And (Filter.Tendsto.{0, 0} CrouzeixConjecture.outerApproximationRadius Filter.atTop.{0} (nhds.{0} (OfNat.ofNat.{0} 0))) (And (Filter.Tendsto.{0, 0} (fun k => Metric.hausdorffDist.{0} (closure.{0} (CrouzeixConjecture.parallelOuterDomain K k)) K) Filter.atTop.{0} (nhds.{0} (OfNat.ofNat.{0} 0))) (∀ (k : Nat), LE.le.{0} (closure.{0} (CrouzeixConjecture.parallelOuterDomain K k)) (Metric.cthickening.{0} (OfNat.ofNat.{0} 1) K)))`.
Type SHA-256: `1f2d7a4b929e101df48496466ae11483b7aa7f7328ffe49b5c0d95c34335b000`.
Direct maintained dependencies: `CrouzeixConjecture.outerApproximationRadius, CrouzeixConjecture.parallelOuterDomain, CrouzeixConjecture.parallelOuterDomain_closure_subset_fixedNeighborhood, CrouzeixConjecture.tendsto_hausdorffDist_parallelOuterDomain_closure, CrouzeixConjecture.tendsto_outerApproximationRadius`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: compact `K`; filters, closures, and containment remain explicit.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:1f2d7a4b929e101df48496466ae11483b7aa7f7328ffe49b5c0d95c34335b000`.

#### Exercises and solutions

CFT-25-E05 proves that unequal one-sided tangents at a polygon vertex contradict `C¹` differentiability, rather than vaguely saying corners are bad.

### CFT-25-006 — oriented radial C¹ boundary data {#cft-25-006}

#### Purpose

Chapter 26 needs a differentiable, correctly oriented contour with support data. This theorem supplies exactly that package for every canonical outer domain.

#### Definitions and notation

Fix a nonempty compact convex core `K`, a center `c∈K`, and a positive radius `r`. Translate the outer domain to the origin,
$$D=\{v:c+v\in\operatorname{thickening}(r,K)\},$$
write `u(θ)=e^{iθ}`, and define the inverse gauge radius
$$\rho(\theta)=\operatorname{gauge}(D,u(\theta))^{-1}.$$
Then
$$\gamma(\theta)=c+\rho(\theta)u(\theta).$$
At this frontier point let `P_Kγ(θ)` be the metric projection onto the compact core and define the normalized projection residual
$$\nu(\theta)=\frac{\gamma(\theta)-P_K\gamma(\theta)}r,
\qquad s(\theta)=\|\gamma'(\theta)\|.$$
`CanonicalParallelOrientedRadialBoundaryStatement` quantifies this construction over matrices and indices. It does not assert that the contour map is onto the whole frontier.

#### Statement

Every `parallelOuterDomain (numericalRange A) k` has the canonical positively oriented radial `C¹` boundary package required by downstream Cauchy integration.

#### Hypothesis ledger

The dimension type is finite, decidable, and nonempty, making the numerical range nonempty, compact, and convex. Positive radius provides genuine thickening.

#### Proof roadmap

Choose a center in the numerical range; prove inverse-gauge positivity, periodicity, and `C¹` regularity; prove frontier membership; normalize the projection residual; establish support and positive speed; then orient the tangent.

#### Proof

We construct the positively oriented regular radial C¹ boundary package consumed by Cauchy integration. For every `A,k`, choose `c∈W(A)` and put `K=W(A)`, `r=ε_k>0`.

First, the inverse gauge is finite and positive because the translated thickening is a bounded convex neighborhood of zero. The identity `gauge(ρ(θ)u(θ))=1` places `γ(θ)` on the frontier. The squared-distance equation
$$\|\gamma(\theta)-P_K\gamma(\theta)\|^2=r^2$$
has strictly positive derivative in its radial variable. The scalar implicit-function theorem therefore makes `ρ`, and hence `γ`, `C¹`. This is radial C¹ regularity, not a claim about an arbitrary parametrization.

In the packaged notation `R(θ)=ρ(θ)`, the checked frontier field reads
$$\gamma(\theta)=c+R(\theta)e^{i\theta}\in\operatorname{frontier}(\Omega_k).$$

Second, the projection residual has norm `r`, so `ν(θ)` is a unit vector. The projection variational inequality gives, for every `y` in the open thickening,
$$\operatorname{Re}\!\left(\overline{\nu(\theta)}(\gamma(\theta)-y)\right)\ge0.$$
Thus `ν(θ)` is a supporting normal, not merely a perpendicular vector.

Third, differentiating the constant squared-distance equation gives
$$\langle\nu(\theta),\gamma'(\theta)\rangle_{\mathbb R}=0.$$
The radial formula is
$$\gamma'(\theta)=\bigl(\rho'(\theta)+i\rho(\theta)\bigr)e^{i\theta}.$$
Since `ρ(θ)>0`, this tangent cannot vanish. Consequently the speed `s(θ)=‖γ′(θ)‖` has positive speed, `s(θ)>0`. The outward normal has positive component along `e^{iθ}`; together with orthogonality this selects the counterclockwise sign and proves the orientation identity
$$\gamma'(\theta)=i\nu(\theta)s(\theta).$$
In plain contour notation, this is exactly `γ′(θ)=iν(θ)s(θ)`.

Here is the exact Lean boundary. `PositivePeriodicRadialData` has provider fields for `radius`, its displayed derivative, positivity, a derivative proof at every parameter, continuity of the derivative, and periodicity. `OrientedRadialConvexBoundary` has provider fields for the open convex radial domain, continuous `normal` and `speed`, nonnegative speed, the `OutwardBoundarySupport` witness, and `tangent_eq`. The lower lemmas `parallelGaugeRadius_pos`, `contDiff_one_parallelGaugeRadius`, `parallelPositivePeriodicRadialData_point_mem_frontier`, `outwardBoundarySupport_thickening`, and `radialTangent_eq_I_mul_normal_mul_norm` assemble those fields. The textbook lemmas `parallel_radial_tangent_ne_zero` and `parallel_radial_speed_pos` record the strict regularity upgrade for this construction. The public theorem reexports only the final canonical package, truthfully. It never claims the raw body has `C∞` boundary; that stronger property is neither proved nor needed.

#### Worked instance

For the ellipse, support becomes `h+ε_k` and the smooth boundary moves by `x_k(θ)=x(θ)+ε_kn(θ)`. If `α=0` the ellipse is a degenerate segment, yet positive thickening gives a stadium-like domain with the certified C¹ contour; the strict-speed lemma additionally certifies regularity.

#### Boundary case

At radius zero, the constant-speed traversal of a segment or polygon retains unequal one-sided tangents. A polygon can be given a degenerate `C¹` parametrization that pauses at every vertex, so the correct obstruction is to a positive-speed regular radial package, not to every conceivable `C¹` map. At positive radius the contract is exactly regular `C¹`, not an unjustified `C²` or `C∞` assertion.

#### Historical context

Radial and support parametrizations are classical convex geometry; positive orientation is what fixes the sign in Cauchy formulas.
Source boundary: canonical parallel radial geometry in the registered Crouzeix formalization packet.
Review status: orientation, exact regularity boundary, no-`C∞` claim, and compiler provider reviewed.

#### ML analogy

Mathematical object: a typed oriented differentiable boundary interface.
ML counterpart: a certified geometric object passed to a downstream integration routine.
Exact transfer: explicit fields prevent consumers from assuming unavailable regularity.
Non-transfer: autodifferentiating samples does not prove global orientation or boundary image.
Diagnostic: check image, tangent, orientation sign, and support property independently.

#### Pedagogical prerequisites

Readers need numerical-range compactness and convexity, radial curves, tangent orientation, and the distinction between `C¹` and higher smoothness.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.parallel_outer_domain_data`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.canonicalParallelOrientedRadialBoundaryStatement`.
Substantive provider: `CrouzeixConjecture.canonicalParallelOrientedRadialBoundaryStatement`; strict speed is checked by `CrouzeixTextbook.Part05.parallel_radial_speed_pos`.
Readable type map: every nonempty finite matrix dimension receives the quantified canonical boundary statement.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L114).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [Nonempty.{u_1 + 1} n], CrouzeixConjecture.CanonicalParallelOrientedRadialBoundaryStatement.{u_1}`.
Type SHA-256: `e0c090a879dce8bda6dbcd5297ac2597904ac6646e0fff4ebdae2e33e4f846b9`.
Direct maintained dependencies: `CrouzeixConjecture.canonicalParallelOrientedRadialBoundaryStatement`.
Axioms: `Classical.choice, Quot.sound, propext`.
Assumptions: `Fintype n`, `DecidableEq n`, and `Nonempty n`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:e0c090a879dce8bda6dbcd5297ac2597904ac6646e0fff4ebdae2e33e4f846b9`.

#### Exercises and solutions

CFT-25-E06 reconstructs the package from lower-level numerical-range and thickening data, avoiding the public card and preassembled existence shortcut.

## Worked examples

The disk calculation in CFT-25-E02 makes projection explicit, and the running matrix `A_{λ,α}` carries the same geometry through the chapter: its elliptical numerical range acquires the shifted support function `h_k=h_W+ε_k`; in the degenerate segment case, positive thickening still produces the certified stadium-like `C¹` contour.

## ML bridge

The exact transfers are projection stability, compact robust envelopes, and typed boundary data. The non-transfers are equally important: learned approximate projectors need not be nonexpansive, sampled perturbations need not converge in Hausdorff distance, and autodifferentiated samples do not certify global orientation. The per-card diagnostics state how to test each boundary.

## Lean translation

The six public declarations at lines 14, 47, 56, 65, 71, and 114 of `Part05/Chapter25.lean` are compiler-bound to the displayed types. CFT-25-002, CFT-25-003, and CFT-25-006 truthfully reexport exact maintained providers; the other three are proved locally. The six exercises are separate theorems in `CrouzeixTextbook.Part05.Exercises.Chapter25` with distinct statement fingerprints and audited proof dependencies.

## Exercises with complete solutions

### CFT-25-E01 — projection witness {#exercise-cft-25-e01}

Prove existence of a nearest point in a nonempty closed convex subset of the complex plane.

#### Complete written solution

We use closedness to obtain completeness, construct a minimizer, and prove its minimizing inequality. Let `d=inf_{u∈K}‖z-u‖` and choose `w_n∈K` with `‖z-w_n‖<d+1/n`. The sequence is bounded. In the complex plane, its tail lies in the compact intersection of `K` with a closed ball, so a subsequence converges to `p`. Closedness gives `p∈K`, and norm continuity gives
$$\|z-p\|=d\le\|z-w\|\quad(w\in K).$$
Lean expresses the same argument through completeness of a closed set and the complete-convex minimizer theorem. It does not call CFT-25-001.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_01_solution`.
Formal mode: `proved-here`.
Provider boundary: Mathlib `exists_norm_eq_iInf_of_complete_convex` via `IsClosed.isComplete`; no maintained Crouzeix provider and no reconstruction shortcut.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L122).
Type SHA-256: `ee6bd57fe174d7239540f948a6a03855a9a066eda912a472dad2534fc28e890b`.
Axioms: `Classical.choice, Quot.sound, propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:ee6bd57fe174d7239540f948a6a03855a9a066eda912a472dad2534fc28e890b`.

### CFT-25-E02 — disk projection {#exercise-cft-25-e02}

Compute metric projection onto a closed disk.

#### Complete written solution

We split inside/outside cases and derive the exact radial formula from the variational inequality. Inside, distance zero gives `P_Kx=x`. Outside set `q=(r/‖x‖)x`; then `‖q‖=r`, and for `‖w‖≤r`,
$$\langle x-q,w-q\rangle_{\mathbb R}=(1-r/\|x\|)(\langle x,w\rangle_{\mathbb R}-\|x\|r)\le0.$$
Comparison with the independently derived condition for the chosen minimizer forces `q=P_Kx`.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_02_solution`.
Formal mode: `proved-here`.
Provider boundary: uses the compact projection definition, membership, and infimum identity; it does not call the public variational card or a disk-projection shortcut.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L136).
Type SHA-256: `c7a7cb6d472ecdae80c1536ce80998d0a8f0357ef6736888c94d3734b1c1fe0a`.
Axioms: `Classical.choice, Quot.sound, propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:c7a7cb6d472ecdae80c1536ce80998d0a8f0357ef6736888c94d3734b1c1fe0a`.

### CFT-25-E03 — variational uniqueness {#exercise-cft-25-e03}

Prove the projection variational inequalities imply uniqueness.

#### Complete written solution

We test each candidate against the other, add the real-inner-product inequalities, and force zero distance. Expansion gives
$$\|p-q\|^2=\langle x-p,q-p\rangle_{\mathbb R}+\langle x-q,p-q\rangle_{\mathbb R}\le0.$$
A squared norm is nonnegative, hence zero, so `p=q`. The Lean proof is pure inner-product algebra and uses no uniqueness provider.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_03_solution`.
Formal mode: `proved-here`.
Provider boundary: elementary real-inner-product expansion only; there is no direct maintained provider and no uniqueness shortcut.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L204).
Type SHA-256: `a98e82a390fe2b59c39f83f0fb2347b59f1c2dbd20a41d098256a00edbaf3a88`.
Axioms: `Classical.choice, Quot.sound, propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:a98e82a390fe2b59c39f83f0fb2347b59f1c2dbd20a41d098256a00edbaf3a88`.

### CFT-25-E04 — nonexpansiveness algebra {#exercise-cft-25-e04}

Derive nonexpansiveness from two variational inequalities.

#### Complete written solution

We add the two projection inequalities and close with Cauchy--Schwarz, treating the zero-distance case. With `d=p_x-p_y`,
$$\|d\|^2\le\langle x-y,d\rangle_{\mathbb R}\le\|x-y\|\|d\|.$$
If `‖d‖=0` we are done; otherwise cancel the positive factor to obtain `‖p_x-p_y‖≤‖x-y‖`. Lean explicitly follows both branches.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_04_solution`.
Formal mode: `proved-here`.
Provider boundary: elementary inner-product algebra and Cauchy--Schwarz only; it does not call either maintained nonexpansiveness theorem.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L219).
Type SHA-256: `257db097066a44c31c2934e6dadb37742f14c665296df8d49441c382c2533884`.
Axioms: `Classical.choice, Quot.sound, propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:257db097066a44c31c2934e6dadb37742f14c665296df8d49441c382c2533884`.

### CFT-25-E05 — corner obstruction {#exercise-cft-25-e05}

Show why the natural constant-speed polygonal parametrization is incompatible with the positive-speed regular radial C¹ package.

#### Complete written solution

We exhibit unequal one-sided complex tangents, prove nondifferentiability of the constant-speed corner model, and distinguish degenerate C¹ parametrizations that pause. For `f(t)=tv` when `t≤0` and `f(t)=tw` otherwise,
$$\lim_{t\uparrow0}\frac{f(t)-f(0)}t=v\ne w=\lim_{t\downarrow0}\frac{f(t)-f(0)}t.$$
A derivative would equal both. When `‖v‖=‖w‖>0`, this is exactly the constant-speed polygonal parametrization near a genuine corner. Lean proves the stronger statement for any `v≠w`: it computes the two `derivWithin` values on `Iic 0` and `Ici 0`, then contradicts global differentiability.

This must not be inflated into “a polygon has no `C¹` parametrization”: degenerate C¹ parametrizations can pause at each vertex. Reparametrize each edge with derivative tending to zero at both endpoints, so the derivative glues continuously as zero at the corner. Such a map is not regular there. The actual incompatibility is with the positive-speed regular radial C¹ package, whose tangent never vanishes and therefore determines one tangent line at every parameter.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_05_solution`.
Formal mode: `proved-here`.
Provider boundary: one-sided derivative uniqueness from Mathlib calculus; no maintained Crouzeix provider and no polygon regularity shortcut.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L243).
Type SHA-256: `8ae57520a57476a45463626e2493efd5cf74e168730249e50887e59438fff1db`.
Axioms: `Classical.choice, Quot.sound, propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:8ae57520a57476a45463626e2493efd5cf74e168730249e50887e59438fff1db`.

### CFT-25-E06 — radial package {#exercise-cft-25-e06}

Formalize and prove the oriented radial C¹ package required for every canonical parallel outer domain.

#### Complete written solution

We quantify over the numerical range and parallel-body index and produce the oriented radial boundary package consumed by Cauchy integration. Choose `c∈W(A)` and define the inverse-gauge radius `ρ`, `γ(θ)=c+ρ(θ)e^{iθ}`, the projection-residual normal `ν(θ)`, and `s(θ)=‖γ′(θ)‖`. Nonemptiness, compactness, convexity, and `ε_k>0` feed the lower-level topology, implicit-function, support, and orientation lemmas, yielding
$$\forall A\,k,\ \operatorname{HasOrientedRadialConvexBoundary}(\operatorname{parallelOuterDomain}(W(A),k)).$$
Frontier membership comes from gauge normalization; radial `C¹` regularity comes from the positive radial derivative of the distance equation; the supporting normal property comes from metric projection; `ρ>0` makes the tangent nonzero; and the orientation lemma gives `γ′(θ)=iν(θ)s(θ)`. Therefore the constructed witness is exactly the interface required by the canonical statement. Lean packages the radial data and boundary witness directly, without either forbidden canonical shortcut.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_06_solution`.
Formal mode: `proved-here`.
Provider boundary: lower-level numerical-range facts, `parallelPositivePeriodicRadialData`, and `orientedRadialConvexBoundary_thickening`; it does not call `canonicalParallelOrientedRadialBoundaryStatement` or the public CFT-25-006 theorem.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L282).
Type SHA-256: `9a64eacc8aaba37cbac9d770626aac85e112e005a0fd27a38cee7b3ea047fcf8`.
Axioms: `Classical.choice, Quot.sound, propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:9a64eacc8aaba37cbac9d770626aac85e112e005a0fd27a38cee7b3ea047fcf8`.

## Synthesis and forward dependencies

Closedness and finite-dimensional properness give a nearest point, convexity makes it unique, a segment derivative gives the normal inequality, two inequalities give stability, positive radii give controlled outer domains, Hausdorff decay returns maxima, and the oriented regular radial `C¹` package hands Chapter 26 a lawful contour.
