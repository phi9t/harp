---
id: cft-chapter-15-complex-differentiability
title: Complex differentiability
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-11
tags: [crouzeix-textbook, analysis, mathematics, lean]
confidence: high
canonical: 15_complex_differentiability.md
chapter: 15
part: 3
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 15: Complex differentiability

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iii-analysis-and-complex-functions|Part III -- Analysis and complex functions]]
Previous: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/14_sequences_and_series_of_operators|Chapter 14 — Sequences and series of operators]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/16_consequences_of_cauchy_theory|Chapter 16 — Consequences of Cauchy theory]]

## Opening problem

A holomorphic scalar function can be evaluated at a matrix. For a diagonalizable
matrix the recipe is obvious — apply `f` to each eigenvalue and reassemble — and
it is also useless as a definition, because it presupposes a diagonalization and
says nothing about matrices that have none.

The repair is the Cauchy integral. Write the value as an integral of `f` against a
resolvent over a contour surrounding the spectrum, and the formula makes sense for
every matrix, diagonalizable or not. What has to be proved is that the two agree
where both are defined, and that the integral is stable enough to be reached by
approximation everywhere else.

That is this chapter: a contour integral, its continuity in the matrix, its
agreement with the eigenvalue recipe on diagonalizable matrices, and a density
argument carrying the agreement to every matrix.

## What this chapter compiles, and what it does not

The chapter is titled for complex differentiability and develops none of it.
**Not compiled here:** the difference quotient and its independence of direction,
the Cauchy–Riemann equations, the theorem that holomorphic implies analytic,
Cauchy's integral formula itself, Cauchy's estimate, and any example of a function
satisfying Cauchy–Riemann at a point without being differentiable there. The
earlier sketch of this chapter set five exercises on those topics and all five are
withdrawn; its sixth, a reading of the functional-calculus identity's hypotheses,
survives below as E04.

What is compiled is the functional calculus Cauchy's formula makes possible, in
the concrete form Part VI consumes: a parametrized boundary integral, continuity of
that integral in its matrix argument, its agreement with the simple-diagonalization
evaluation on an oriented radial contour, and the convergence of the
simple-spectrum approximants to it.

Cauchy's theorem itself enters through a maintained declaration named in
CFT-15-003's proof,
`integral_holomorphic_cauchy_resolvent_of_simpleDiagonalization`, which this
chapter re-uses rather than proves.

## Conceptual model

Fix a convex domain `\Omega` and a matrix `B` whose numerical range lies inside
it. The boundary integral
$$
\Phi(f,B)=\int f\bigl(\Gamma(x)\bigr)\,K_{\Gamma}(B,x)\;d\mu(x)
$$
pairs the scalar values of `f` along the contour with a matrix-valued kernel built
from the resolvent. Three facts make it a functional calculus rather than just an
integral.

It is *continuous in `B`*, on the set where the numerical range stays inside
`\Omega`, needing only continuity of `f` on the closure.

It *agrees with the obvious recipe* when `B` is simply diagonalizable: the
integral equals `f` applied eigenvalue-by-eigenvalue. This is where Cauchy's
theorem is used, and where holomorphy rather than mere continuity is required.

And the simply diagonalizable matrices are *dense*, by Chapter 6's simple-spectrum
density. Continuity plus agreement on a dense set plus convergence of the
approximants gives the calculus everywhere.

## Running example: a matrix with a repeated eigenvalue

The running family `A_{\lambda,\alpha}` of Chapter 6 has a single eigenvalue
`\lambda` of multiplicity two and, for `\alpha \neq 0`, no diagonalization. The
eigenvalue recipe cannot be applied to it at all.

The contour integral can. And Chapter 6's density theorem supplies matrices
`A_k \to A_{\lambda,\alpha}` with distinct eigenvalues, each of which the recipe
does handle; CFT-15-006 says the recipe's values along that sequence converge to
the integral's value at `A_{\lambda,\alpha}`. That is the whole argument in one
example, and the chapter compiles the general statement rather than this instance.

## Formal development

### CFT-15-001 — the parametric boundary integral {#cft-15-001}

#### Purpose

Name the integral once. Every later card is a property of this object.

#### Statement

This is a definition, not a theorem. For a parametrized convex boundary `\Gamma`
of a domain `\Omega`, a measure `\mu` on the parameter space, a scalar function
`f` and a matrix `B`,
$$
\Phi(f,B)=\int f\bigl(\Gamma(x)\bigr)\cdot K_{\Gamma}(B,x)\;d\mu(x),
$$
where `K_{\Gamma}(B,x)` is the analytic half of the boundary density. Nothing is
asserted about convergence, about `f`, or about `B`.

#### Hypothesis ledger

None on `f` or `B`, and fewer on the ambient context than the surrounding chapter
needs. Writing the integral requires only a topological, measurable parameter space
and a finite index type; the normalized type below carries no `CompactSpace`, no
`IsFiniteMeasure` and no `Nonempty`, and exercise E01 omits all of them. Those
become load-bearing first in CFT-15-002. The card's
docstring in the provider is explicit that this becomes the normalized functional
calculus **only** for the oriented radial boundary and parameter measure used
later — for a general `\Gamma` and `\mu` it is an integral and nothing more.

#### Proof roadmap

Unfold the definition.

#### Proof

The integrand is the scalar `f(\Gamma(x))` acting on the matrix-valued
`parametricBoundaryFirstPart \Gamma B x`, which is itself
`(2\pi)^{-1}\,\text{speed}(x)` times the normal times the resolvent. The
definition is that product integrated against `\mu`. Exercise E01 performs the
unfolding.

Calling this a functional calculus is premature and the definition does not. The
normalization constant `(2\pi)^{-1}` sits in the kernel so that the oriented
radial case comes out normalized, and CFT-15-003 is what actually checks that.

#### Worked instance

At `f \equiv 0` the integrand vanishes and the integral is the zero matrix, for
any contour, measure and `B`. Exercise E02 compiles this.

#### Boundary case

For a general `\Gamma` and `\mu` the value need not be a functional calculus of
anything. Only the oriented radial boundary with `contourParameterMeasure` is
shown to give the normalized evaluation, and only in CFT-15-003.

#### Historical context

Defining `f(A)` by a contour integral rather than by a spectral decomposition is
the Riesz–Dunford construction; its advantage is exactly that it never mentions
eigenvectors, which is why it survives the non-diagonalizable case.
Source boundary: Mathlib 4.32.1 measure-theoretic declarations, and the registered Crouzeix geometry packet, which owns the provider.
Review status: the definition's generality and the normalization caveat reviewed together.

#### ML analogy

Mathematical object: a matrix-valued integral pairing scalar values on a contour with a resolvent kernel.
ML counterpart: a spectral filter applied to an operator through a contour or quadrature rule rather than through an eigendecomposition.
Exact transfer: the construction needs no eigenvectors, so it is defined for operators that cannot be diagonalized.
Non-transfer: nothing here is a quadrature scheme, a discretization, or a cost model; the integral is exact and no numerical method is described.
Diagnostic: a filter that fails on a defective matrix is using an eigendecomposition somewhere, which this construction does not.

#### Pedagogical prerequisites

Chapter 12's oriented boundary and Chapter 14's analytic functions on a disk.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.parametric_boundary_integral`.
Formal mode: `definition`.
Underlying declaration: `CrouzeixConjecture.parametricBoundaryIntegral`.
Substantive provider: `none`; a definition has no proof obligation. The named declaration is an alias of the definition, which the underlying-declaration line names.
Readable type map: `Gamma` is the parametrized boundary, `mu` the parameter measure, `f` the scalar function, `B` the matrix.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L9).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `{i : Type u_1} → {n : Type u_2} → [inst : TopologicalSpace.{u_1} i] → [inst_1 : MeasurableSpace.{u_1} i] → [Fintype.{u_2} n] → [DecidableEq.{u_2 + 1} n] → {Omega : Set.{0} Complex} → CrouzeixConjecture.ParametricConvexBoundary.{u_1} Omega → MeasureTheory.Measure.{u_1} i → (Complex → Complex) → CrouzeixConjecture.SquareMatrix.{u_2} n → CrouzeixConjecture.SquareMatrix.{u_2} n`.
Type SHA-256: `db42d2cc78e2548dc4e32779200458704022a64a442f32f34441ec2a39f9a3b6`.
Direct maintained dependencies: `CrouzeixConjecture.parametricBoundaryIntegral`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: none on `f` or `B`; a topological, measurable parameter space and a finite index type. Compactness, finiteness of the measure and nonemptiness are not used.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:db42d2cc78e2548dc4e32779200458704022a64a442f32f34441ec2a39f9a3b6`.

#### Exercises and solutions

CFT-15-E01 unfolds the definition and CFT-15-E02 evaluates it at the zero
function, which needs no hypothesis on the contour.

### CFT-15-002 — the integral is continuous in the matrix {#cft-15-002}

#### Purpose

The analytic engine of the density argument. Without continuity in `B` the
approximation step below has nothing to stand on.

#### Statement

For a fixed contour and a scalar `f` continuous on `\overline{\Omega}`, the map
`B \mapsto \Phi(f,B)` is continuous on the set of matrices whose numerical range
is contained in `\Omega`.

#### Hypothesis ledger

Continuity of `f` on the *closure* is what is used, and holomorphy is not needed
here — the provider's docstring says so explicitly. The numerical-range condition
is what keeps the resolvent invertible along the whole contour: it is not a
technicality but the reason the integrand exists.

The restriction to that set is what the provider can establish invertibility from,
and it is sufficient rather than necessary. The chain is `W(B) \subseteq \Omega`,
so a boundary point lies outside `\Omega` and hence outside `W(B)`, and the
spectrum is contained in the numerical range — but that containment is strict in
general, so a matrix can fail the condition and still have an invertible resolvent
along the whole contour. Take `\Omega` the open unit disk and
`B = \begin{bmatrix}0&3\\0&0\end{bmatrix}`: its spectrum is `\{0\}` but its
numerical range is the disk of radius `3/2`, so `W(B) \not\subseteq \Omega` while
the resolvent exists at every point of the unit circle.

Nor does the integrand cease to exist outside the set. The resolvent is Mathlib's
total matrix inverse, which returns `0` on a singular argument, so
`\Phi(f,B)` is a total function of `B` — as CFT-15-001's ledger says. What fails
outside the numerical-range set is not definedness but meaning.

#### Proof roadmap

Apply a parametrized continuity-of-integral lemma, then check the integrand is
jointly continuous on the relevant product set.

#### Proof

The provider applies `continuousOn_integral_of_compact_support` with the whole
parameter space as the compact support, which is available because that space is
compact.

The remaining obligation is joint continuity of the integrand on
`\{B : W(B) \subseteq \Omega\} \times \text{parameter space}`. That splits.
The scalar factor `(B,x) \mapsto f(\Gamma(x))` is continuous because `\Gamma` is
continuous, its values lie in `\overline{\Omega}` — the provider gets this from
`frontier_subset_closure` applied to the boundary-support witness — and `f` is
continuous there by hypothesis. The matrix factor is a private lemma of the
provider module establishing that the uncurried first part is continuous on the
same product, whose own content is that the resolvent is invertible when the
numerical range is inside `\Omega`.

The two are combined with `.smul` and the result matched to the integrand with a
`congr` on `Function.uncurry`.

#### Worked instance

For `f` a polynomial, continuity on the closure is automatic and the card applies
to every matrix whose numerical range sits inside `\Omega`. The polynomial case is
not compiled here; it is the general card instantiated by hand.

#### Boundary case

Continuity holds on the numerical-range set, not on all matrices. A matrix whose
numerical range touches the boundary is outside the domain of the statement, and
nothing here describes the behaviour there.

#### Historical context

That the Riesz–Dunford integral depends continuously on the operator is what makes
the calculus robust; the numerical-range formulation used here is the one Crouzeix's
problem needs, since the numerical range rather than the spectrum is what the
theorem controls.
Source boundary: Mathlib 4.32.1 integration declarations, and the registered Crouzeix geometry packet, which owns the provider.
Review status: the continuity-only hypothesis on `f` reviewed against the holomorphy needed later.

#### ML analogy

Mathematical object: continuity of an operator-valued integral in its operator argument.
ML counterpart: stability of a spectral filter under perturbation of the operator.
Exact transfer: on the region where the numerical range stays inside the domain, the output moves continuously with the input.
Non-transfer: no modulus of continuity is given, so this is not a perturbation bound and supports no error estimate.
Diagnostic: output that jumps under a small perturbation indicates the numerical range left the domain, not a discontinuity of the construction.

#### Pedagogical prerequisites

CFT-15-001, and the numerical range.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.boundary_integral_continuous`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.continuousOn_parametricBoundaryIntegral`.
Substantive provider: `CrouzeixConjecture.continuousOn_parametricBoundaryIntegral`, a genuine multi-step proof: `continuousOn_integral_of_compact_support`, then joint continuity of the integrand split into its scalar and matrix factors.
Readable type map: `Gamma` is the contour, `f` the scalar function continuous on the closure, and the set is the matrices whose numerical range lies in `Omega`.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L11).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {i : Type u_1} {n : Type u_2} [inst : TopologicalSpace.{u_1} i] [CompactSpace.{u_1} i] [inst_2 : MeasurableSpace.{u_1} i] [OpensMeasurableSpace.{u_1} i] [inst_4 : Fintype.{u_2} n] [inst_5 : DecidableEq.{u_2 + 1} n] {mu : MeasureTheory.Measure.{u_1} i} [MeasureTheory.IsFiniteMeasure.{u_1} mu] {Omega : Set.{0} Complex} (Gamma : CrouzeixConjecture.ParametricConvexBoundary.{u_1} Omega) (f : Complex → Complex), ContinuousOn.{0, 0} f (closure.{0} Omega) → ContinuousOn.{u_2, u_2} (CrouzeixConjecture.parametricBoundaryIntegral.{u_1, u_2} Gamma mu f) (setOf.{u_2} fun B => LE.le.{0} (CrouzeixConjecture.numericalRange.{u_2} B) Omega)`.
Type SHA-256: `963da44a67bc224363d41d3a83934eefd3ce72483cb4703321565ec0e2fa8f8b`.
Direct maintained dependencies: `CrouzeixConjecture.continuousOn_parametricBoundaryIntegral`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: `f` continuous on the closure of `Omega`; the numerical-range condition defines the domain of continuity.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:963da44a67bc224363d41d3a83934eefd3ce72483cb4703321565ec0e2fa8f8b`.

#### Exercises and solutions

CFT-15-E03 pairs this continuity with the convergence it makes possible.

### CFT-15-003 — the integral is the eigenvalue recipe, when there is one {#cft-15-003}

#### Purpose

The card that makes the construction a *functional calculus* rather than an
unrelated integral: on the matrices where the obvious answer exists, the integral
gives it.

#### Statement

Let `R` be positive periodic radial data, `G` an oriented radial convex boundary
for it around `c` with domain `\Omega`, and `V` an open convex set containing
`\overline{\Omega}`. If `f` is holomorphic on `V`, `B` is simply diagonalizable
and its numerical range lies in `\Omega`, then
$$
\Phi_{G}(f,B)=\text{functionEval}_{B}(f),
$$
the evaluation of `f` eigenvalue-by-eigenvalue through the diagonalization. The
notation hides a data dependence the Lean makes explicit: `functionEval` consumes
the `SimpleDiagonalization` structure itself, so the right-hand side depends on
the witness `hB` and not merely on `B`.

#### Hypothesis ledger

This is the chapter's heaviest ledger and every clause is used.

Holomorphy of `f` on `V` is genuinely required, not merely used: continuity on the
closure, which suffices for CFT-15-002, does not give the eigenvalue recipe, and
the primitive the Cauchy step constructs is exactly what fails without it.

Convexity of `V` is a different matter — the proof uses it, to obtain that
primitive, but the statement does not need it. Under the card's other hypotheses
the closure of `\Omega` is compact and convex, so a thickening of it inside `V` is
an open convex set on which `f` is still holomorphic; the hypothesis could be
dropped and recovered. It is assumed because the provider's route wants it.

Simple diagonalizability is what makes the right-hand side exist at all;
`functionEval` is defined through the diagonalization.

The oriented radial structure is what makes the normalization come out to `1`,
and the orientation is doing real work in that cancellation. The period is
`2\pi`, so the kernel's `(2\pi)^{-1}` against Cauchy's `\text{period}\cdot i`
would leave `i`. What removes it is the positive-orientation identity
`\text{tangent} = i\cdot\text{normal}\cdot\text{speed}` carried by the boundary
structure, which turns the kernel's scalar into `-i(2\pi)^{-1}` times the
tangent. The compiled cancellation is therefore
`(-i(2\pi)^{-1})\cdot(2\pi i) = 1`, which is the maintained
`cauchyNormalization_mul_period_I`.

#### Proof roadmap

Rewrite the integrand into Cauchy form, convert the contour integral to an
interval integral, pull out the constant, apply Cauchy's resolvent theorem, and
cancel the normalization.

#### Proof

The provider's proof is a five-step `calc`.

First the integrand is replaced: pointwise, `f(\Gamma(x))\cdot K(B,x)` equals
`k\cdot f(R(c,t))\cdot(\text{tangent}(t)\cdot \text{doubleLayerResolvent}(B,\cdot))`
with `k = -i/(2\pi)`, via
`parametricBoundaryFirstPart_eq_cauchyIntegrand` and an entrywise `simp` with
`ring`. The rewrite is fed to `integral_congr_ae`, but the provider proves it for *every*
`x` and injects that with `Eventually.of_forall`; the almost-everywhere framing is
the congruence lemma's interface, not the strength of what is checked. What the
step does need is `parametricBoundaryFirstPart_eq_cauchyIntegrand` and a `ring`,
so it is not definitional.

Second, the integral over the contour parameter becomes an interval integral over
`[0,\text{period}]`, by `integral_contourParameter_eq_intervalIntegral`.

Third, the constant `k` comes out of the interval integral.

Fourth — and this is where the analysis lives — the remaining integral is
`(\text{period}\cdot i)\cdot \text{functionEval}_{B}(f)` by the maintained
`integral_holomorphic_cauchy_resolvent_of_simpleDiagonalization`, which is
Cauchy's theorem for this resolvent and is **not proved in this chapter**. The
boundary points lie in `\overline{\Omega}` by `frontier_subset_closure`, which is
what lets that theorem's hypotheses be discharged.

Finally `k\cdot(\text{period}\cdot i) = 1` by `cauchyNormalization_mul_period_I`,
and the normalization cancels.

#### Worked instance

For `f(z)=z` the identity says the integral returns `B` itself whenever `B` is
simply diagonalizable and its numerical range is inside the domain. The chapter
does not compile that instance.

#### Boundary case

Drop simple diagonalizability and the right-hand side does not exist; the identity
is not false but unstatable. That gap is exactly what CFT-15-006 repairs, by
approximation rather than by extending this identity.

#### Historical context

The agreement of the Riesz–Dunford integral with the spectral recipe on
diagonalizable operators is the consistency check that justifies calling the
integral a functional calculus; without it the construction would be a definition
with no claim on the name.
Source boundary: the registered Crouzeix geometry packet, which owns both this provider and the Cauchy resolvent theorem it calls.
Review status: the use of Cauchy's theorem as an imported result reviewed explicitly.

#### ML analogy

Mathematical object: agreement of a contour-integral functional calculus with the eigenvalue recipe on diagonalizable operators.
ML counterpart: a spectral filter implemented by quadrature agreeing with the same filter implemented by eigendecomposition, where both are available.
Exact transfer: on diagonalizable inputs the two constructions give the same matrix exactly.
Non-transfer: this is an identity of exact values and says nothing about two numerical implementations agreeing to any tolerance.
Diagnostic: disagreement on a diagonalizable input means one side is not the construction described here, since the exact values coincide.

#### Pedagogical prerequisites

CFT-15-001, Chapter 12's oriented boundary, and simple diagonalization.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.boundary_integral_is_function_eval`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_functionEval`.
Substantive provider: `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_functionEval`, a five-step `calc` whose analytic content is the imported `integral_holomorphic_cauchy_resolvent_of_simpleDiagonalization`.
Readable type map: `R` and `c` fix the radial data, `G` the oriented boundary, `V` the holomorphy domain, `B` the matrix, and `hB` the diagonalization witness that `functionEval` consumes as data.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L13).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] (R : CrouzeixConjecture.PositivePeriodicRadialData) (c : Complex) {Omega V : Set.{0} Complex} (G : R.OrientedRadialConvexBoundary c Omega), IsOpen.{0} V → Convex.{0, 0} Real V → LE.le.{0} (closure.{0} Omega) V → ∀ {f : Complex → Complex}, DifferentiableOn.{0, 0, 0} Complex f V → ∀ (B : CrouzeixConjecture.SquareMatrix.{u_1} n) (hB : CrouzeixConjecture.SimpleDiagonalization.{u_1} B), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} B) Omega → Eq.{u_1 + 1} (CrouzeixConjecture.parametricBoundaryIntegral.{0, u_1} (CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundary R c G) CrouzeixConjecture.contourParameterMeasure f B) (CrouzeixConjecture.SimpleDiagonalization.functionEval.{u_1} hB f)`.
Type SHA-256: `74acdb3127b3924d38eb938c0934826e0ff23e1aa7c6eb4ec4a7e63c496d534e`.
Direct maintained dependencies: `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_functionEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: holomorphy on an open convex `V` containing the closure, simple diagonalizability of `B`, and its numerical range inside `Omega`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:74acdb3127b3924d38eb938c0934826e0ff23e1aa7c6eb4ec4a7e63c496d534e`.

#### Exercises and solutions

CFT-15-E04 states this identity with its hypotheses grouped into geometry,
analyticity and matrix structure — the one exercise of the earlier sketch that
survives.

### CFT-15-004 — the simple-spectrum evaluation sequence {#cft-15-004}

#### Purpose

Name the approximating values. Chapter 6 supplies matrices with distinct
eigenvalues near any given matrix; this card names what the eigenvalue recipe
returns on them.

#### Statement

This is a definition, not a theorem. For a matrix `A`, a scalar `f` and an index
`k`,
$$
S_k(A,f)=\text{functionEval}_{A_k}(f),
$$
where `A_k` is Chapter 6's `k`-th simple-spectrum approximant to `A` and the
evaluation is through the diagonalization its distinct eigenvalues provide.

#### Hypothesis ledger

None. The definition is total in `A`, `f` and `k`: the approximant always has
distinct eigenvalues, by Chapter 6's construction, so the diagonalization always
exists and `functionEval` always applies. No hypothesis on `A` is needed, and in
particular `A` itself may be defective.

#### Proof roadmap

Unfold: read `S_k(A,f)` as `functionEval` of the diagonalization supplied by the
approximant's distinct eigenvalues.

#### Proof

`simpleSpectrumApproximation A k` is Chapter 6's construction and
`simpleSpectrumApproximation_hasDistinctEigenvalues` is the theorem that it has
distinct eigenvalues; together they feed
`simpleDiagonalization_of_hasDistinctEigenvalues`, and `functionEval` is applied
to the result. Exercise E05 performs this unfolding.

The definition being total is the point. The eigenvalue recipe does not apply to
`A`; it applies to every `A_k`, unconditionally, and the convergence card is what
transfers information back.

#### Worked instance

For the running family `A_{\lambda,\alpha}` with `\alpha \neq 0`, which has no
diagonalization, every `S_k` is nonetheless defined.

#### Boundary case

`S_k(A,f)` is a sequence of values, not a value at `A`. Nothing in this card says
it converges, or that its limit deserves to be called `f(A)`; both are CFT-15-006.

#### Historical context

Defining a functional calculus on a dense set and extending by continuity is the
standard route; naming the approximating sequence as an object rather than leaving
it inside a limit is what lets the extension be stated as a convergence theorem.
Source boundary: the registered Crouzeix geometry packet, which owns the provider and Chapter 6's density construction.
Review status: totality of the definition reviewed against the conditional convergence card.

#### ML analogy

Mathematical object: the eigenvalue recipe evaluated along a sequence of perturbed, diagonalizable matrices.
ML counterpart: perturbing a defective matrix to break degenerate eigenvalues before applying an eigendecomposition-based routine.
Exact transfer: the perturbed matrices are genuinely diagonalizable, so the recipe applies exactly to each.
Non-transfer: the perturbation here is a specific exact construction, not a random or floating-point jitter, and no conditioning claim follows.
Diagnostic: a routine that succeeds only after perturbation is relying on diagonalizability, which the contour integral does not.

#### Pedagogical prerequisites

Chapter 6's simple-spectrum density and CFT-15-001.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.simple_spectrum_holomorphic_eval`.
Formal mode: `definition`.
Underlying declaration: `CrouzeixConjecture.simpleSpectrumHolomorphicEval`.
Substantive provider: `none`; a definition has no proof obligation. The named declaration is an alias of the definition, which the underlying-declaration line names.
Readable type map: `A` is the target matrix, `f` the scalar function, `k` the approximation index.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L15).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `{n : Type u_1} → [Fintype.{u_1} n] → [DecidableEq.{u_1 + 1} n] → [Nonempty.{u_1 + 1} n] → CrouzeixConjecture.SquareMatrix.{u_1} n → (Complex → Complex) → Nat → CrouzeixConjecture.SquareMatrix.{u_1} n`.
Type SHA-256: `f74b567c4c68920453b562c528c09954e6ebe2600c879b384eee872eb58b2fea`.
Direct maintained dependencies: `CrouzeixConjecture.simpleSpectrumHolomorphicEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: none; the definition is total.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:f74b567c4c68920453b562c528c09954e6ebe2600c879b384eee872eb58b2fea`.

#### Exercises and solutions

CFT-15-E05 unfolds this definition and records that it needs no hypothesis on `A`.

### CFT-15-005 — the contour sees the approximants converge {#cft-15-005}

#### Purpose

The first half of the density argument: the *integral* evaluated at the
approximants converges to the integral at the target.

#### Statement

For a fixed contour, `f` continuous on `\overline{\Omega}`, an open `\Omega`
and a matrix `A` whose numerical range lies inside it,
$$
\Phi(f,A_k)\longrightarrow \Phi(f,A).
$$

#### Hypothesis ledger

Openness of `\Omega` is used, and only here: it is what makes the
numerical-range condition an *open* condition, so that the approximants
eventually satisfy it. Continuity of `f` on the closure is inherited from
CFT-15-002. The numerical range of `A` must lie inside `\Omega`, not merely in
its closure, for the same reason.

Note that this card says nothing about the eigenvalue recipe. It is a statement
about the integral alone.

#### Proof roadmap

Show the approximants eventually satisfy the numerical-range condition, upgrade
their convergence to convergence within that set, and compose with CFT-15-002.

#### Proof

Chapter 6's approximants converge to `A`, and
`eventually_simpleSpectrumApproximation_numericalRange_subset_open` says that for
an open `\Omega` they eventually have numerical range inside it — this is the
step openness pays for.

Those two combine through `tendsto_nhdsWithin_of_tendsto_nhds_of_eventually_within`
into convergence *within* the set `\{B : W(B) \subseteq \Omega\}`, which is the
form a `ContinuousOn` statement can consume.

Finally CFT-15-002 at `A` gives continuity there, and its `.tendsto` composed with
the within-convergence is the conclusion.

#### Worked instance

For the running family with `\alpha \neq 0` and a contour whose domain contains
its numerical range, the integral's values at the approximants converge to its
value at the family member itself.

#### Boundary case

Replace `\Omega` by a set that is not open and the eventual-membership step
fails: the approximants may leave the region at every index, and the composition
has nothing to compose with.

#### Historical context

Continuity plus density is the oldest route to extending an operator
construction; the content here is entirely in checking that the approximating
sequence stays inside the region where the construction is continuous.
Source boundary: the registered Crouzeix geometry packet, which owns the provider and Chapter 6's density construction.
Review status: the role of openness reviewed as the hypothesis that makes membership eventual.

#### ML analogy

Mathematical object: convergence of a continuous operator construction along an approximating sequence.
ML counterpart: expecting a stable spectral routine to give consistent answers on a sequence of perturbed inputs approaching a degenerate one.
Exact transfer: the values converge, provided the perturbed inputs remain in the region where the construction is continuous.
Non-transfer: no rate is given, so this licenses no stopping rule for a perturbation schedule.
Diagnostic: values that fail to settle indicate the approximants are leaving the region, which openness of the domain is what prevents.

#### Pedagogical prerequisites

CFT-15-002, CFT-15-004, and Chapter 6's density theorem.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.simple_spectrum_boundary_limit`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.tendsto_parametricBoundaryIntegral_simpleSpectrumApproximation`.
Substantive provider: `CrouzeixConjecture.tendsto_parametricBoundaryIntegral_simpleSpectrumApproximation`, which chains eventual membership, `tendsto_nhdsWithin_of_tendsto_nhds_of_eventually_within`, and CFT-15-002's continuity.
Readable type map: `Gamma` is the contour, `A` the target matrix, and `A_k` Chapter 6's approximants.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L17).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {i : Type u_1} {n : Type u_2} [inst : TopologicalSpace.{u_1} i] [CompactSpace.{u_1} i] [inst_2 : MeasurableSpace.{u_1} i] [OpensMeasurableSpace.{u_1} i] [inst_4 : Fintype.{u_2} n] [inst_5 : DecidableEq.{u_2 + 1} n] [inst_6 : Nonempty.{u_2 + 1} n] {mu : MeasureTheory.Measure.{u_1} i} [MeasureTheory.IsFiniteMeasure.{u_1} mu] {Omega : Set.{0} Complex} (Gamma : CrouzeixConjecture.ParametricConvexBoundary.{u_1} Omega) (f : Complex → Complex), ContinuousOn.{0, 0} f (closure.{0} Omega) → ∀ (A : CrouzeixConjecture.SquareMatrix.{u_2} n), IsOpen.{0} Omega → LE.le.{0} (CrouzeixConjecture.numericalRange.{u_2} A) Omega → Filter.Tendsto.{0, u_2} (fun k => CrouzeixConjecture.parametricBoundaryIntegral.{u_1, u_2} Gamma mu f (CrouzeixConjecture.simpleSpectrumApproximation.{u_2} A k)) Filter.atTop.{0} (nhds.{u_2} (CrouzeixConjecture.parametricBoundaryIntegral.{u_1, u_2} Gamma mu f A))`.
Type SHA-256: `486e395c4946e9143d989e8d08dbb5ec1cafcfa3b625fe421b5881d3fa179b2e`.
Direct maintained dependencies: `CrouzeixConjecture.tendsto_parametricBoundaryIntegral_simpleSpectrumApproximation`, which calls `continuousOn_parametricBoundaryIntegral`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: `f` continuous on the closure, `Omega` open, and the numerical range of `A` inside `Omega`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:486e395c4946e9143d989e8d08dbb5ec1cafcfa3b625fe421b5881d3fa179b2e`.

#### Exercises and solutions

CFT-15-E03 states this convergence together with the continuity it rests on.

### CFT-15-006 — the eigenvalue recipe converges to the integral {#cft-15-006}

#### Purpose

The chapter's conclusion. It closes the loop: the obvious recipe, applied along
the approximating sequence, converges to the contour integral's value at the
original matrix — which may be defective.

#### Statement

Under the geometry and holomorphy hypotheses of CFT-15-003, and for any matrix `A`
whose numerical range lies in `\Omega`,
$$
S_k(A,f)\longrightarrow \Phi_{G}(f,A).
$$

#### Hypothesis ledger

The union of CFT-15-003's and CFT-15-005's, minus simple diagonalizability of the
target — which is precisely the point. `A` is arbitrary.

Holomorphy on `V` is needed because the identity of CFT-15-003 is applied at each
approximant; continuity on the closure, which CFT-15-005 wants, is derived from it
by `hf.continuousOn.mono hclosure` rather than assumed separately. Openness of
`\Omega` comes from the boundary's domain rather than being a hypothesis of the
card.

#### Proof roadmap

Take the integral's convergence from CFT-15-005, then replace the integral at each
approximant by the eigenvalue recipe using CFT-15-003, eventually.

#### Proof

CFT-15-005 gives `\Phi(f,A_k) \to \Phi(f,A)`, after deriving the continuity of `f`
on the closure from holomorphy on `V`.

The two sequences `\Phi(f,A_k)` and `S_k(A,f)` then agree *eventually*, not
identically. The provider takes the eventual numerical-range membership of the
approximants and, on that filter, applies CFT-15-003 at each `A_k` — legitimate
because each approximant is simply diagonalizable by construction, which is what
supplies that card's missing hypothesis.

Finally `Tendsto.congr'` transfers the limit across an eventual equality. The
`'`-form matters: the two sequences need not agree at small `k`, where an
approximant may not yet be inside `\Omega`.

This is where the chapter's three ingredients meet — continuity from CFT-15-002,
agreement from CFT-15-003, density from Chapter 6 — and none of them alone gives
the conclusion.

#### Worked instance

For `A_{\lambda,\alpha}` with `\alpha \neq 0`, which has no diagonalization, the
eigenvalue recipe applied to its approximants converges to the contour integral's
value. That value is what deserves the name `f(A_{\lambda,\alpha})`.

#### Boundary case

The card gives a limit, not an identity: it does not say `S_k(A,f)` equals
`\Phi_G(f,A)` for any particular `k`, and in general it does not. Nor does it
claim the limit is independent of the contour; that is a separate maintained
theorem, `parametricBoundaryIntegral_eq_of_two_orientedRadialBoundaries`, which
this chapter does not carry as a card.

#### Historical context

Extending a functional calculus from the diagonalizable matrices by density is the
finite-dimensional shadow of extending from a dense subalgebra; the reason it is
worth compiling is that the defective case is exactly the case Crouzeix's problem
cannot avoid.
Source boundary: the registered Crouzeix geometry packet, which owns the provider.
Review status: the eventual-rather-than-identical agreement of the two sequences reviewed explicitly.

#### ML analogy

Mathematical object: extension of a spectral construction from diagonalizable operators to all operators by density.
ML counterpart: justifying a spectral routine on a degenerate input by the limit of its values on nearby non-degenerate ones.
Exact transfer: the limit exists and equals the contour construction's value at the degenerate input.
Non-transfer: no rate, and no claim that any finite perturbation is close enough; the statement is about the limit alone.
Diagnostic: values that converge to something other than the contour value indicate the approximants left the domain, since inside it the two sequences eventually coincide.

#### Pedagogical prerequisites

CFT-15-003 and CFT-15-005.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.simple_spectrum_eval_limit`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.tendsto_simpleSpectrumHolomorphicEval`.
Substantive provider: `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.tendsto_simpleSpectrumHolomorphicEval`, which composes CFT-15-005's convergence with an eventual application of CFT-15-003 through `Tendsto.congr'`.
Readable type map: `A` is the arbitrary target matrix, `S_k` the simple-spectrum evaluations, and the limit is the contour integral at `A`.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L19).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [inst_2 : Nonempty.{u_1 + 1} n] (R : CrouzeixConjecture.PositivePeriodicRadialData) (c : Complex) {Omega V : Set.{0} Complex} (G : R.OrientedRadialConvexBoundary c Omega), IsOpen.{0} V → Convex.{0, 0} Real V → LE.le.{0} (closure.{0} Omega) V → ∀ {f : Complex → Complex}, DifferentiableOn.{0, 0, 0} Complex f V → ∀ (A : CrouzeixConjecture.SquareMatrix.{u_1} n), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} A) Omega → Filter.Tendsto.{0, u_1} (CrouzeixConjecture.simpleSpectrumHolomorphicEval.{u_1} A f) Filter.atTop.{0} (nhds.{u_1} (CrouzeixConjecture.parametricBoundaryIntegral.{0, u_1} (CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundary R c G) CrouzeixConjecture.contourParameterMeasure f A))`.
Type SHA-256: `686e54ee644104f71a77067fd770780a508573e5b613bc07f69b3cf2afef3f11`.
Direct maintained dependencies: `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.tendsto_simpleSpectrumHolomorphicEval`, which calls `tendsto_parametricBoundaryIntegral_simpleSpectrumApproximation` and `parametricBoundaryIntegral_eq_functionEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: the geometry and holomorphy hypotheses of CFT-15-003, with no diagonalizability assumption on `A`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:686e54ee644104f71a77067fd770780a508573e5b613bc07f69b3cf2afef3f11`.

#### Exercises and solutions

CFT-15-E06 states this conclusion at an arbitrary matrix, which is the form the
maintained outer-limit development and Part VI consume.

## Worked examples

**A defective matrix.** The running family `A_{\lambda,\alpha}` with
`\alpha \neq 0` has one eigenvalue of multiplicity two and no diagonalization, so
the eigenvalue recipe is not merely inaccurate on it — it is undefined. The
contour integral is defined, and CFT-15-006 says the recipe's values on the
approximants converge to it. Neither the family nor this instance is compiled.

**Where holomorphy is needed and where it is not.** CFT-15-002 asks only that `f`
be continuous on the closure, because it proves continuity of an integral.
CFT-15-003 asks for holomorphy on an open convex neighbourhood, because it invokes
Cauchy's theorem. The chapter keeps the two hypotheses apart on purpose; a reader
who assumes holomorphy throughout will not see which card needs what.

**The normalization.** The period is `2\pi`, so the kernel's `(2\pi)^{-1}`
against Cauchy's `\text{period}\cdot i` leaves `i`, not `1`. The factor that
removes it is `-i`, and it comes from the orientation: the boundary structure
carries `\text{tangent} = i\cdot\text{normal}\cdot\text{speed}`, so the kernel's
scalar is `-i(2\pi)^{-1}` times the tangent rather than `(2\pi)^{-1}` times the
normal. The compiled identity is
`(-i(2\pi)^{-1})\cdot(2\pi i) = 1`. That cancellation is specific to the oriented
radial contour with `contourParameterMeasure`, and CFT-15-001's docstring says so.

**The zero function.** `\Phi(0,B)=0` for every contour, measure and matrix, with
no hypotheses at all. Exercise E02 compiles it, and it is the only value of the
integral this chapter computes in closed form.

## ML bridge

**Mathematical object.** A functional calculus defined by a contour integral,
agreeing with the eigenvalue recipe where that exists, and extended to every
matrix by density.

**ML counterpart.** Applying a scalar function to an operator — a spectral filter,
a matrix function in a preconditioner, a resolvent in an implicit solver — for
operators that may be defective.

**Exact transfer.** The construction never mentions eigenvectors, so it is defined
on defective inputs where an eigendecomposition-based routine is not. On
diagonalizable inputs the two agree exactly.

**Non-transfer.** Nothing here is a numerical method. No quadrature, no
discretization of the contour, no conditioning analysis, and no rate for the
density argument's convergence. A defective matrix is also ill-conditioned for
eigenvalue computation, and this chapter says nothing about that.

**Diagnostic.** A matrix-function routine that fails or degrades sharply on a
near-defective input is relying on diagonalization; the contour construction is
the alternative that does not, and its continuity is in the *matrix*, not in the
eigenvalues.

## Lean translation

`CrouzeixTextbook.Part03.Chapter15` exposes six cards and six checked exercise
solutions. CFT-15-001 and CFT-15-004 are `definition` rows; the other four are
`reexported-proof` rows over `CrouzeixConjecture.HolomorphicFunctionalCalculus`.
Four of the six rows were registered as `checkpoint` with no underlying
declaration and are corrected here. CFT-15-001 and CFT-15-004 alias definitions and
become `definition` rows; CFT-15-003 and CFT-15-006 alias theorems in the nested
namespace `PositivePeriodicRadialData.OrientedRadialConvexBoundary`, which the
receipt exporter treats as maintained, and become `reexported-proof` rows.

All four theorem providers are genuine multi-step proofs. CFT-15-003's is a
five-step `calc` and is the longest in the chapter; its analytic content is an
imported Cauchy theorem, which the card names rather than claims.

The exercises carry the chapter's variable context explicitly and use `omit` to
drop the ambient instances they do not need, which keeps the module warning-free.

Not compiled, and not claimed: complex differentiability itself, the difference
quotient, the Cauchy–Riemann equations, holomorphic implies analytic, Cauchy's
integral formula, Cauchy's estimate, any rate for CFT-15-006's convergence, and
independence of the value from the choice of contour — that last is a maintained
theorem, `parametricBoundaryIntegral_eq_of_two_orientedRadialBoundaries`, which
this chapter does not carry. The earlier sketch set five exercises on complex
differentiability and all five are withdrawn; its sixth, a reading of
CFT-15-003's hypotheses, survives as E04.

## Exercises with complete solutions

### CFT-15-E01 — unfolding the integral {#exercise-cft-15-e01}

Unfold the parametric boundary integral to the integral it names.

#### Complete written solution

By definition the value is the integral, against `\mu`, of the scalar
`f(\Gamma(x))` acting on the matrix-valued first part of the boundary density. The
statement holds by `rfl` — it is a definitional unfolding and nothing more.

The exercise is worth stating because the definition is where the normalization
lives. The `(2\pi)^{-1}` that makes CFT-15-003's identity come out without a
stray constant is inside the kernel, not inserted later, and reading the
definition is the only way to see that.

Nothing in the ambient context beyond the measure is used, which is why the
checked solution omits the compactness, open-measurability, nonemptiness and
finite-measure instances.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter15.exercise_01_solution`.
Formal mode: `proved-here`.
Provider boundary: `rfl`; a definitional unfolding with no provider and no hypothesis on `f`, `B` or the contour.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L33).
Type SHA-256: `05097994711b6fa8226b8a2962e9c3a57029dd64cb0597f63ee051ea7d1e0b33`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:05097994711b6fa8226b8a2962e9c3a57029dd64cb0597f63ee051ea7d1e0b33`.

### CFT-15-E02 — the zero function {#exercise-cft-15-e02}

Evaluate the integral at the zero function.

#### Complete written solution

The integrand is `0 \cdot K(B,x)`, which is the zero matrix pointwise, and the
integral of the zero function is zero. So `\Phi(0,B)=0` for every contour, measure
and matrix.

No hypothesis is needed — not continuity of anything, not a numerical-range
condition, not compactness. That is the contrast worth noticing: every other card
in the chapter carries hypotheses because it says something about a *nonzero* `f`,
and this one shows the hypotheses are about `f` and `B` rather than about the
construction being well posed.

The checked solution unfolds the definition and closes by `simp`.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter15.exercise_02_solution`.
Formal mode: `proved-here`.
Provider boundary: the definition of `parametricBoundaryIntegral` and `simp`; no provider and no hypotheses.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L41).
Type SHA-256: `38e14ac8dd819d33fa238e2c4ed188ad60cbe530a1aafccd03b8624139554ec7`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:38e14ac8dd819d33fa238e2c4ed188ad60cbe530a1aafccd03b8624139554ec7`.

### CFT-15-E03 — continuity and the convergence it gives {#exercise-cft-15-e03}

State the continuity of the integral in its matrix argument together with the
convergence along the approximants that follows.

#### Complete written solution

The first conjunct is CFT-15-002: for `f` continuous on the closure, the map
`B \mapsto \Phi(f,B)` is continuous on the numerical-range set.

The second is CFT-15-005: for an open `\Omega` and a matrix `A` inside it, the
integral's values at the simple-spectrum approximants converge to its value at
`A`.

Pairing them makes visible that the second is the first composed with an
approximation, and that the extra hypothesis the second carries — openness of
`\Omega` — is what makes the approximants eventually admissible. The checked
solution is the pair of card applications and adds no proof.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter15.exercise_03_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.continuousOn_parametricBoundaryIntegral` and `tendsto_parametricBoundaryIntegral_simpleSpectrumApproximation`, the providers of CFT-15-002 and CFT-15-005; the exercise adds the pairing, not a proof.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L48).
Type SHA-256: `19a55e7da2f7a20491ce66dbb2f0e7bd11148c0703319e21389633a6581e0377`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:19a55e7da2f7a20491ce66dbb2f0e7bd11148c0703319e21389633a6581e0377`.

### CFT-15-E04 — grouping the hypotheses of the identity {#exercise-cft-15-e04}

State the agreement of the integral with the eigenvalue recipe, and group its
hypotheses into geometry, analyticity and matrix structure.

#### Complete written solution

The identity is CFT-15-003. Its hypotheses fall into three groups.

*Geometry*: the radial data `R`, the centre `c`, and the oriented radial convex
boundary `G` with domain `\Omega`. These fix the contour and, with the parameter
measure, the normalization — the identity is stated for this contour and has no
content for an arbitrary one.

*Analyticity*: the open convex `V` with `\overline{\Omega} \subseteq V`, and
holomorphy of `f` on `V`. Convexity is not decoration: the proof reaches Cauchy's
theorem, which needs an antiderivative on the domain. Mere continuity, which
suffices for CFT-15-002, does not suffice here.

*Matrix structure*: simple diagonalizability of `B`, which is what makes the
right-hand side exist, and the numerical range of `B` inside `\Omega`, which is
what keeps the resolvent finite on the contour.

The checked solution states the identity under exactly these hypotheses and
discharges it by the card's provider. This is the one exercise of the earlier
sketch of this chapter that survives.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter15.exercise_04_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_functionEval`, CFT-15-003's provider; the exercise restates it and does not reprove it.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L65).
Type SHA-256: `f731421e4d7dc8f66aca392a1f32a8d0cc0edcddc2cea87693c8a8130f86d565`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:f731421e4d7dc8f66aca392a1f32a8d0cc0edcddc2cea87693c8a8130f86d565`.

### CFT-15-E05 — the approximating evaluations are total {#exercise-cft-15-e05}

Unfold the simple-spectrum evaluation sequence and record that it needs no
hypothesis on the matrix.

#### Complete written solution

`S_k(A,f)` is `functionEval` applied to the diagonalization that the `k`-th
approximant's distinct eigenvalues supply. The statement holds by `rfl`.

The content is in the absence of hypotheses. `A` is an arbitrary matrix — it may
be defective, and for the running family with `\alpha \neq 0` it is — yet every
`S_k(A,f)` is defined, because the approximant rather than `A` is what gets
diagonalized. That totality is what makes the sequence available as a bridge, and
it is the reason the definition is stated separately from the convergence card
that uses it.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter15.exercise_05_solution`.
Formal mode: `proved-here`.
Provider boundary: `rfl`; a definitional unfolding of `simpleSpectrumHolomorphicEval`, with no hypothesis on `A`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L78).
Type SHA-256: `c6cb463a3a9785d0cc83589b23c2fcec35ce67a3814b6121c1e4e87f35f1b503`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:c6cb463a3a9785d0cc83589b23c2fcec35ce67a3814b6121c1e4e87f35f1b503`.

### CFT-15-E06 — the calculus at an arbitrary matrix {#exercise-cft-15-e06}

State the convergence of the eigenvalue recipe along the approximants to the
contour integral's value, for a matrix with no diagonalizability assumption.

#### Complete written solution

This is CFT-15-006. Under the geometry and holomorphy hypotheses of CFT-15-003,
and for any `A` whose numerical range lies in `\Omega`,
`S_k(A,f) \to \Phi_G(f,A)`.

The absent hypothesis is the point: nothing assumes `A` is diagonalizable. The
argument reaches it by applying CFT-15-003 at each approximant, where
diagonalizability does hold, and transferring the limit across an eventual
equality.

The checked solution states this at an arbitrary `A` and discharges it by the
card's provider. It is the form the maintained outer-limit development consumes,
and through that Part VI, where the matrices in question are not assumed to have
distinct eigenvalues.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter15.exercise_06_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.tendsto_simpleSpectrumHolomorphicEval`, CFT-15-006's provider; the exercise restates it and does not reprove it.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter15.lean#L85).
Type SHA-256: `6f0dbf4ed2df6911fe363700cb3a1cb7ebdf8f9081514fb5f4b720c7a99db56a`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:6f0dbf4ed2df6911fe363700cb3a1cb7ebdf8f9081514fb5f4b720c7a99db56a`.

## Synthesis and forward dependencies

The chapter contributes a functional calculus that does not presuppose
diagonalizability: an integral, its continuity in the matrix, its agreement with
the eigenvalue recipe where that exists, and a density argument reaching every
matrix.

Chapter 16 draws the consequences of Cauchy theory that the calculus rests on.

CFT-15-006's provider is consumed by the maintained
`CrouzeixConjecture.HolomorphicOuterLimit` development and, through it, by
Part VI's Chapter 32. Part V is not a consumer: its Chapter 28 takes
`SimpleDiagonalization` as a hypothesis, which is precisely the assumption this
chapter's last card exists to remove, so the extension is used further downstream
than the first application of the calculus.
