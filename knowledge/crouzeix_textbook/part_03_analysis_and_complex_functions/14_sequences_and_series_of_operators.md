---
id: cft-chapter-14-sequences-and-series-of-operators
title: Sequences and series of operators
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-11
tags: [crouzeix-textbook, analysis, mathematics, lean]
confidence: high
canonical: 14_sequences_and_series_of_operators.md
chapter: 14
part: 3
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 14: Sequences and series of operators

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iii-analysis-and-complex-functions|Part III -- Analysis and complex functions]]
Previous: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/13_metric_and_normed_spaces|Chapter 13 — Metric and normed spaces]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/15_complex_differentiability|Chapter 15 — Complex differentiability]]

## Opening problem

Chapter 13 established that a maximum over a compact set exists. This chapter
supplies the other analytic ingredient Part VI needs: a matrix-valued function of
a complex variable, defined by a series, known to be analytic on a disk.

The object is a power series whose coefficients are matrices rather than scalars,
$$
F(z)=\sum_{m\ge0}z^{m}a_{m},
$$
and the question is what has to be true of the coefficient sequence for `F` to be
a well-behaved function on the open unit disk. The answer compiled here is: a
uniform bound on the coefficient norms suffices.

That is exactly the shape the Jin and Lorist–Schwenninger completions need in
Part VI, where `F` is a resolvent-like function built from a contraction and the
coefficient bound comes from the contraction property.

## What this chapter compiles, and what it does not

The chapter is titled for sequences and series of operators, and it develops one
case: matrix-valued power series in a complex variable.

**Not compiled here:** the Neumann series and the inverse `(I-T)^{-1}`, absolute
convergence in a general Banach space, the spectral radius formula, the geometric
series sum for an operator, continuity of inversion and openness of the invertible
set, convergence of operator sequences in any other sense, and anything at all on
the boundary circle. The earlier sketch of this chapter displayed four theorems on
those topics and set six exercises; four exercises are withdrawn. Its finite geometric identity
*is* compiled, as E02, because that identity is pure ring algebra and needs no
analysis.

What is compiled is two definitions and four theorems: the formal series, how its
terms evaluate, that a uniform coefficient bound forces radius of convergence at
least one, the sum function, that the sum is analytic on the open unit disk, and
that inside that disk the sum really is the limit of the expected matrix
monomials. Four of the six cards are `reexported-proof` rows over
`CrouzeixConjecture.MatrixPowerSeries`; two are `definition` rows.

## Conceptual model

A matrix power series is not a new kind of object. It is Mathlib's
`FormalMultilinearSeries` specialized so that the `m`-th multilinear term is the
map sending `(z,\dots,z)` to `z^{m}a_{m}` — a scalar `m`-linear form times a
fixed matrix. Everything about radius of convergence and analyticity is then
inherited from the general theory, and the chapter's work is to check that the
specialization behaves as the notation suggests.

The one quantitative input is the coefficient bound. If `‖a_m‖ ≤ C` for every
`m`, the `m`-th term has norm at most `C|z|^{m}`, which is summable for `|z|<1`.
That is a sufficient condition and not a necessary one: a series with unbounded
coefficients can still have radius at least one, `a_m = m\cdot I` being the
standard example, whose radius is exactly one. Nothing in this chapter claims the
converse.

## Running example: the geometric series of a contraction

Take `a_m = T^{m}` for a matrix `T` with `‖T‖ \le 1`. Then `‖a_m‖ \le 1` for every
`m`: submultiplicativity gives `‖T^{m}‖ \le ‖T‖^{m} \le 1` for `m \ge 1`, and the
`m = 0` term is `I`, whose induced norm is `1`. So the coefficient bound holds with
`C = 1`, the radius is at least one, and
$$
F(z)=\sum_{m\ge0}z^{m}T^{m}
$$
is analytic on the open unit disk. On paper one then writes `F(z)=(I-zT)^{-1}`
and calls it the Neumann series.

Nothing in this chapter instantiates the cards at `a_m = T^{m}`. The bound
`‖T^{m}‖ \le ‖T‖^{m}` is not compiled here either. The one compiled instantiation
is exercise E06, at a *constant* coefficient sequence, where the bound is an
equality rather than an estimate.

**That last step is not compiled.** The chapter gives the series, its radius, its
analyticity and its sum as a limit of monomials; it does not identify the sum with
any inverse, and no declaration here mentions `(I-zT)^{-1}`. Exercise E02 compiles
the finite identity that a proof of the Neumann formula would start from, and
stops there.

## Formal development

### CFT-14-001 — the formal matrix power series {#cft-14-001}

#### Purpose

Name the series object once, as an instance of Mathlib's general formal
multilinear series, so that radius and analyticity come from the existing theory
rather than being rebuilt.

#### Statement

This is a definition, not a theorem. For a coefficient sequence
`a : ℕ → SquareMatrix n`, `matrixPowerSeries a` is the formal multilinear series
whose `m`-th term is the continuous `m`-linear map built from `a m` by
`ContinuousMultilinearMap.mkPiRing`. No convergence is asserted.

#### Hypothesis ledger

None. The definition accepts any coefficient sequence, including wildly unbounded
ones for which the series converges nowhere but at the origin. That totality is
why the later cards carry their own coefficient bound.

#### Proof roadmap

Unfold the definition: read the `m`-th term as `mkPiRing` applied to `a m`.

#### Proof

`ContinuousMultilinearMap.mkPiRing ℂ (Fin m) (a m)` is the `m`-linear map on
`ℂ^{m}` determined by sending the tuple of ones to `a m`; on a repeated argument
`(z,\dots,z)` it therefore produces `z^{m}\cdot a_m`. That evaluation is the
content of CFT-14-002 and is not re-derived here.

#### Worked instance

For the constant sequence `a_m = B`, the series is the one whose terms evaluate to
`z^{m}B` — the matrix geometric series with ratio `z`.

#### Boundary case

The definition says nothing about convergence. For `a_m = m!\cdot I` the radius is zero and no uniform bound exists, so
CFT-14-003, CFT-14-005 and CFT-14-006 all fail to apply. The two definition cards
and CFT-14-002 carry no hypotheses and apply unchanged; they simply say nothing
about convergence.

#### Historical context

Treating a matrix-valued power series as a formal multilinear series rather than
as a sequence of partial sums is the choice that makes the general
radius-of-convergence machinery apply verbatim; the alternative is to reprove the
root test for each new coefficient type.
Source boundary: Mathlib 4.32.1 analytic declarations, and the registered Crouzeix geometry packet, which owns the provider.
Review status: definition and its non-convergent instance reviewed together.

#### ML analogy

Mathematical object: a power series with matrix coefficients, as a formal object.
ML counterpart: a truncated operator expansion, such as a Neumann or Chebyshev series used to approximate a matrix inverse.
Exact transfer: the coefficients are the same matrices, and a truncated expansion in a fixed matrix is this object evaluated at a fixed scalar rather than a different construction.
Non-transfer: the definition alone gives no convergence, so nothing here says a truncation approximates anything.
Diagnostic: an expansion that diverges for small arguments indicates the coefficients are not bounded, which is the hypothesis of the next cards.

#### Pedagogical prerequisites

Chapter 5's matrix algebra and Chapter 9's matrix norm.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.matrix_power_series`.
Formal mode: `definition`.
Underlying declaration: `CrouzeixConjecture.matrixPowerSeries`.
Substantive provider: `none`; a definition has no proof obligation. The named declaration is an alias of the definition, which the underlying-declaration line names.
Readable type map: `a` is the coefficient sequence and `m` the degree of the term.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L10).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `{n : Type u_1} → (Nat → CrouzeixConjecture.SquareMatrix.{u_1} n) → FormalMultilinearSeries.{0, 0, u_1} Complex Complex (CrouzeixConjecture.SquareMatrix.{u_1} n)`.
Type SHA-256: `f150f77a56b65c2b094d4fbe041a0778dd019cd1d9d711145efee98be6ab4bf2`.
Direct maintained dependencies: `CrouzeixConjecture.matrixPowerSeries`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: none; the definition is total.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:f150f77a56b65c2b094d4fbe041a0778dd019cd1d9d711145efee98be6ab4bf2`.

#### Exercises and solutions

CFT-14-E01 records how the terms evaluate together with the value of the sum at
the origin.

### CFT-14-002 — how the terms evaluate {#cft-14-002}

#### Purpose

Confirm the notation. The `m`-th term, applied to a repeated scalar, is the
monomial the notation `\sum z^{m}a_m` promises.

#### Statement

For every coefficient sequence `a`, degree `m` and scalar `z`,
`matrixPowerSeries a m (fun _ => z) = z^{m} \cdot a_m`.

#### Hypothesis ledger

None. The identity is an unfolding and holds for every `a`, `m` and `z`,
convergent or not.

#### Proof roadmap

Unfold `mkPiRing` on a repeated argument.

#### Proof

The `m`-linear map `mkPiRing ℂ (Fin m) (a m)` sends a tuple `(z_1,\dots,z_m)` to
`(\prod_i z_i)\cdot a_m`. On the repeated argument every `z_i` is `z`, so the
product is `z^{m}`.

**What is checked.** The provider's proof is a single `simp` call with
`matrixPowerSeries` and `ContinuousMultilinearMap.mkPiRing_apply` — one unfolding
step, not the product computation displayed above, which is the content of
Mathlib's `mkPiRing_apply` rather than of this card. The `omit` on the finiteness
instances in the provider records that neither `Fintype n` nor `DecidableEq n` is
used.

#### Worked instance

At `m = 0` the term is the constant `a_0` for every `z`, since `z^{0}=1`. That is
why the value of the sum at the origin is `a_0`.

#### Boundary case

The identity is about a *repeated* argument. On a general tuple the term is
`(\prod_i z_i)a_m`, which this card does not state.

#### Historical context

That a matrix-valued monomial is a multilinear form times a fixed matrix is the
observation that lets one theory of analyticity serve scalar and operator-valued
functions alike.
Source boundary: Mathlib 4.32.1 multilinear declarations, and the registered Crouzeix geometry packet, which owns the provider.
Review status: the single-`simp` provider and the displayed product computation reviewed as distinct.

#### ML analogy

Mathematical object: evaluation of a monomial term of an operator-valued series.
ML counterpart: the `k`-th term of a truncated operator expansion, a scalar power times a fixed matrix.
Exact transfer: the term is exactly the scalar power times the coefficient.
Non-transfer: nothing here concerns the cost of forming that term, which for a Neumann expansion is a matrix product per degree.
Diagnostic: a term that does not scale as the stated power of the argument indicates the coefficients depend on the argument, which this form forbids.

#### Pedagogical prerequisites

CFT-14-001.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.matrix_power_series_coefficient`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.matrixPowerSeries_apply`.
Substantive provider: `CrouzeixConjecture.matrixPowerSeries_apply`, whose proof is a single `simp` with `matrixPowerSeries` and `ContinuousMultilinearMap.mkPiRing_apply`.
Readable type map: `a` is the coefficient sequence, `m` the degree, `z` the scalar argument.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L12).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} (a : Nat → CrouzeixConjecture.SquareMatrix.{u_1} n) (m : Nat) (z : Complex), Eq.{u_1 + 1} (DFunLike.coe.{u_1 + 1, 1, u_1 + 1} (CrouzeixConjecture.matrixPowerSeries.{u_1} a m) fun x => z) (HSMul.hSMul.{0, u_1, u_1} (HPow.hPow.{0, 0, 0} z m) (a m))`.
Type SHA-256: `399f5d4543254160f695c0d3eb518b77e0e445221ca737e581c737a2299671bf`.
Direct maintained dependencies: `CrouzeixConjecture.matrixPowerSeries_apply`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: none.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:399f5d4543254160f695c0d3eb518b77e0e445221ca737e581c737a2299671bf`.

#### Exercises and solutions

CFT-14-E01 pairs this evaluation with the value of the sum at the origin.

### CFT-14-003 — a coefficient bound forces radius at least one {#cft-14-003}

#### Purpose

The chapter's one quantitative hypothesis, and the only place a number enters.

#### Statement

If `‖a_m‖ ≤ C` for every `m`, then the radius of convergence of
`matrixPowerSeries a` is at least `1`.

#### Hypothesis ledger

The uniform bound is what is used, and it is *sufficient but not necessary*.
Radius at least one is equivalent to a limsup condition on `‖a_m‖^{1/m}`, which
tolerates unbounded coefficients: `a_m = m\cdot I` has unbounded norms and radius
exactly one. The card assumes the stronger, checkable hypothesis because that is
what Part VI can supply.

No positivity of `C` is assumed. If `C < 0` the hypothesis cannot hold — norms are
nonnegative and `ℕ` is inhabited — so the statement is vacuously true in that
case, which is harmless and not relied on.

#### Proof roadmap

Apply Mathlib's bound criterion for the radius, then discharge the per-term norm
estimate.

#### Proof

Mathlib's `FormalMultilinearSeries.le_radius_of_bound` says that if
`‖p_m‖\,r^{m} \le C` for every `m`, then the radius is at least `r`, with `r`
implicit. The provider applies it with the given `C`; the goal fixes `r = 1`, so
the geometric factor disappears and the remaining obligation is that the `m`-th
term of `matrixPowerSeries a` has norm at most `C`.

That reduces to the hypothesis. The `m`-th term is `mkPiRing` applied to `a m`,
whose operator norm is `‖a_m‖`, so the goal is exactly `‖a_m‖ ≤ C`; the provider
closes it with `simpa [matrixPowerSeries] using ha m`.

Note what the `1` is doing. The lemma's `r` is implicit and is determined by the
goal; here the goal asks for radius at least `1`, which collapses `‖p_m‖\,r^{m}`
to `‖p_m‖` and makes a uniform coefficient bound exactly the right hypothesis. A
bound of the shape `‖a_m‖ \le C\,r^{-m}` would instantiate the same lemma at that
`r` and give radius at least `r`. This card uses only the `r = 1` case.

#### Worked instance

For `a_m = T^{m}` with `‖T‖ \le 1`, submultiplicativity gives `‖T^{m}‖ \le 1` for
`m \ge 1`, and the `m = 0` coefficient is `I` with induced norm `1`; so `C = 1`
works and the radius is at least one. Submultiplicativity alone does not reach
`m = 0`, where the power is empty.

#### Boundary case

"At least one" is not "exactly one". For `a_m = 0` beyond some degree the series
is a polynomial and the radius is infinite. The card gives a lower bound only, and
nothing in this chapter computes a radius exactly.

#### Historical context

The root test gives the exact radius; the bound criterion gives a lower bound from
a hypothesis that is far easier to verify. Formalizations almost always want the
second, because the coefficient bound is what the surrounding argument produces.
Source boundary: Mathlib 4.32.1 analytic declarations, and the registered Crouzeix geometry packet, which owns the provider.
Review status: sufficiency-without-necessity of the bound reviewed explicitly.

#### ML analogy

Mathematical object: a lower bound on radius of convergence from a uniform coefficient bound.
ML counterpart: the guarantee that a Neumann-style expansion converges for a contraction, obtained from a norm bound on the iterated operator.
Exact transfer: a uniform bound on the coefficient norms gives convergence strictly inside the unit disk.
Non-transfer: the bound is sufficient, not necessary, so failing it does not mean the expansion diverges; and no rate of convergence follows from it.
Diagnostic: an expansion that diverges inside the expected region indicates the coefficient bound was never established, not that the criterion is wrong.

#### Pedagogical prerequisites

CFT-14-001 and Chapter 9's matrix norm.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.matrix_power_series_radius`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.one_le_matrixPowerSeries_radius`.
Substantive provider: `CrouzeixConjecture.one_le_matrixPowerSeries_radius`: `FormalMultilinearSeries.le_radius_of_bound` applied at `C`, then `simpa` to reduce the term norm to the coefficient norm.
Readable type map: `a` is the coefficient sequence and `C` the uniform bound.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L14).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] (a : Nat → CrouzeixConjecture.SquareMatrix.{u_1} n) (C : Real), (∀ (m : Nat), LE.le.{0} (Norm.norm.{u_1} (a m)) C) → LE.le.{0} (OfNat.ofNat.{0} 1) (FormalMultilinearSeries.radius.{0, 0, u_1} (CrouzeixConjecture.matrixPowerSeries.{u_1} a))`.
Type SHA-256: `129deb76798151c789bda54a5f52b915a9987b221523bfcbcbd7aee4953675a7`.
Direct maintained dependencies: `CrouzeixConjecture.one_le_matrixPowerSeries_radius`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: a uniform bound `C` on the coefficient norms.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:129deb76798151c789bda54a5f52b915a9987b221523bfcbcbd7aee4953675a7`.

#### Exercises and solutions

CFT-14-E06 instantiates this at a constant coefficient sequence, where the bound
is the coefficient's own norm; CFT-14-E03 pairs the radius bound with the
analyticity it feeds.

### CFT-14-004 — the sum function {#cft-14-004}

#### Purpose

Name the function the series defines, so that analyticity can be asserted of
something.

#### Statement

This is a definition, not a theorem. `matrixPowerSeriesSum a` is the function
`ℂ → SquareMatrix n` given by Mathlib's `FormalMultilinearSeries.sum` of
`matrixPowerSeries a`. No convergence is asserted.

#### Hypothesis ledger

None. Like CFT-14-001 the definition is total: outside the radius of convergence
Mathlib's `sum` returns the `tsum` of a non-summable family, which is a junk
value. The convergence cards are what make the value meaningful.

#### Proof roadmap

Unfold: read `matrixPowerSeriesSum a` as `(matrixPowerSeries a).sum`.

#### Proof

Mathlib defines the sum of a formal multilinear series at `z` as
`\sum'_m p_m (z,\dots,z)`, an unconditional `tsum`. Composing with CFT-14-002,
that is `\sum'_m z^{m}a_m` — the series in the notation of the opening. The
identification of the `tsum` with a genuine limit is CFT-14-006 and holds only
inside the disk.

The one value available without any convergence hypothesis is at the origin:
`matrixPowerSeriesSum a 0 = a_0`, because every term beyond the constant one
carries a factor `0^{m}`. That is a separate compiled declaration,
`matrixPowerSeriesSum_zero`, and is recorded by exercise E01 rather than by a card
of its own.

#### Worked instance

For `a_m = T^{m}` the sum is the function that Part VI reads as a resolvent. This
chapter does not identify it with one.

#### Boundary case

Outside the radius the definition still returns a matrix, produced by `tsum` on a
non-summable family. Nothing in this chapter interprets that value, and no card
asserts anything about `|z| \ge 1`.

#### Historical context

Defining the sum unconditionally and proving convergence separately is the
formalization convention; it avoids a dependent type in which the function exists
only where the series converges.
Source boundary: Mathlib 4.32.1 analytic declarations, and the registered Crouzeix geometry packet, which owns the provider.
Review status: totality of the definition and the junk value outside the radius reviewed together.

#### ML analogy

Mathematical object: the sum function of an operator-valued power series, defined unconditionally.
ML counterpart: the limit an iterative operator expansion is intended to compute.
Exact transfer: inside the radius the definition is the limit of the partial sums, by CFT-14-006.
Non-transfer: outside the radius the definition still returns a value, and that value means nothing; a routine that reports a result regardless of convergence has the same defect.
Diagnostic: a returned value that does not stabilize as more terms are added indicates the argument is on or outside the radius, not a numerical problem.

#### Pedagogical prerequisites

CFT-14-001 and CFT-14-002.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.matrix_power_series_sum`.
Formal mode: `definition`.
Underlying declaration: `CrouzeixConjecture.matrixPowerSeriesSum`.
Substantive provider: `none`; a definition has no proof obligation. The named declaration is an alias of the definition, which the underlying-declaration line names.
Readable type map: `a` is the coefficient sequence; the value is a function of the complex argument.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L16).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `{n : Type u_1} → (Nat → CrouzeixConjecture.SquareMatrix.{u_1} n) → Complex → CrouzeixConjecture.SquareMatrix.{u_1} n`.
Type SHA-256: `344122eb5432b9afc54c26b78cddb716569cf2a0def94923e3239066935c9a46`.
Direct maintained dependencies: `CrouzeixConjecture.matrixPowerSeriesSum`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: none; the definition is total.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:344122eb5432b9afc54c26b78cddb716569cf2a0def94923e3239066935c9a46`.

#### Exercises and solutions

CFT-14-E01 records the value at the origin; CFT-14-E04 identifies the `tsum` with
the sum inside the disk.

### CFT-14-005 — the sum is analytic on the open unit disk {#cft-14-005}

#### Purpose

The conclusion Part VI consumes: the function is analytic where it matters, so
contour integration and the identity theorem apply to it.

#### Statement

If `‖a_m‖ ≤ C` for every `m`, then `matrixPowerSeriesSum a` is analytic on a
neighbourhood of each point of the open unit disk.

#### Hypothesis ledger

The coefficient bound, used exactly as in CFT-14-003 and with the same
sufficiency-not-necessity caveat. The open disk is where the conclusion is
claimed; the card says nothing at radius one, and nothing outside.

`AnalyticOnNhd` is *defined* as analyticity at each point of the set — in Mathlib
it unfolds to `∀ x ∈ s, AnalyticAt ℂ f x` — and `AnalyticAt` already means a power
series converging on some ball around the point. The notion it should be
contrasted with is `AnalyticOn`, which asks only for a series converging within
`s` and is genuinely weaker; the two agree exactly on open sets, and the unit disk
is open. The card asserts the stronger of the two.

#### Proof roadmap

Turn the radius bound into a power-series ball, shrink the ball to radius one, and
read off analyticity at each interior point.

#### Proof

CFT-14-003 gives radius at least one, hence radius strictly positive.
Mathlib's `hasFPowerSeriesOnBall` then produces a power-series representation on
the ball of radius equal to the radius of convergence.

That representation is restricted to the ball of radius one by `.mono`, using
`0 < 1` and the radius bound. Restriction is the step that makes the statement
about the *unit* disk rather than about the possibly larger disk of convergence.

Finally, for a point `z` of the unit disk, `analyticAt_of_mem` applied to the
restricted representation gives analyticity near `z`. The provider's remaining
work is bookkeeping: it rewrites membership in `unitDisk` into membership in
Mathlib's extended-radius ball, through `Metric.eball_coe` and
`mem_ball_zero_iff`.

#### Worked instance

For `a_m = T^{m}` with `‖T‖ \le 1` the sum is analytic on the open unit disk.
Whether it extends past radius one depends on `T` and is not addressed here.

#### Boundary case

Nothing is claimed at `|z| = 1`. For `a_m = I` the series is geometric, the radius
is exactly one, and at `z = 1` the terms do not tend to zero, so no sum exists —
while the definition of CFT-14-004 still returns a value. Exercise E05 records that
`1` is outside the disk and `0` is inside, which is the compiled form of "the
conclusion is about the open disk".

#### Historical context

Analyticity of a power series strictly inside its disk of convergence is classical
for scalar series; the operator-valued case is identical because the proof uses
only completeness of the target, which finite-dimensional matrices have.
Source boundary: Mathlib 4.32.1 analytic declarations, and the registered Crouzeix geometry packet, which owns the provider.
Review status: the `.mono` restriction to radius one reviewed as the step that fixes the disk.

#### ML analogy

Mathematical object: analyticity of an operator-valued function on an open disk.
ML counterpart: smooth dependence of a resolvent-like quantity on a scalar parameter, inside a stability region.
Exact transfer: analyticity holds at every interior point, so all local complex-analytic tools apply there.
Non-transfer: nothing holds on the boundary circle, which is exactly where a parameter is often pushed in practice.
Diagnostic: behaviour that degrades as the parameter approaches the unit circle is consistent with this card, not a contradiction of it.

#### Pedagogical prerequisites

CFT-14-003, CFT-14-004, and the notion of an analytic function.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.matrix_power_series_analytic`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.matrixPowerSeriesSum_analyticOnNhd_unitDisk`.
Substantive provider: `CrouzeixConjecture.matrixPowerSeriesSum_analyticOnNhd_unitDisk`, which chains `one_le_matrixPowerSeries_radius`, `hasFPowerSeriesOnBall`, `.mono` to radius one, and `analyticAt_of_mem`.
Readable type map: `a` is the coefficient sequence, `C` the uniform bound, and `unitDisk` the open ball of radius one.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L18).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] (a : Nat → CrouzeixConjecture.SquareMatrix.{u_1} n) (C : Real), (∀ (m : Nat), LE.le.{0} (Norm.norm.{u_1} (a m)) C) → AnalyticOnNhd.{0, 0, u_1} Complex (CrouzeixConjecture.matrixPowerSeriesSum.{u_1} a) CrouzeixConjecture.unitDisk`.
Type SHA-256: `1d79c9dee1884eee18821311bdb92ce794fb7052bb1309615802488f23fa15d5`.
Direct maintained dependencies: `CrouzeixConjecture.matrixPowerSeriesSum_analyticOnNhd_unitDisk`, which calls `one_le_matrixPowerSeries_radius`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: a uniform bound `C` on the coefficient norms.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:1d79c9dee1884eee18821311bdb92ce794fb7052bb1309615802488f23fa15d5`.

#### Exercises and solutions

CFT-14-E03 states this together with the radius bound it rests on; CFT-14-E05
records that the disk is open.

### CFT-14-006 — inside the disk the sum is the limit of the monomials {#cft-14-006}

#### Purpose

Close the loop. The function named in CFT-14-004 is, inside the disk, genuinely
the sum of `z^{m}a_m` — not merely a `tsum` that happens to be defined.

#### Statement

If `‖a_m‖ ≤ C` for every `m` and `z` lies in the open unit disk, then the family
`m ↦ z^{m}a_m` has sum `matrixPowerSeriesSum a z`.

#### Hypothesis ledger

The coefficient bound and membership in the open disk. Membership cannot be
dropped: for `a_m = I` and `z = 1` the terms are constantly `I` and no sum exists,
while the left side of the statement remains well formed. The bound is again
sufficient rather than necessary.

#### Proof roadmap

Reuse the power-series ball of CFT-14-005, take its `HasSum` at `z`, then rewrite
the terms into monomial form.

#### Proof

The first three steps are those of CFT-14-005: radius at least one, hence
positive; `hasFPowerSeriesOnBall`; restriction to radius one by `.mono`. The
provider repeats them rather than citing CFT-14-005, because what it needs is the
representation itself, not the analyticity conclusion drawn from it.

Membership `z ∈ unitDisk` is rewritten into membership in Mathlib's
extended-radius ball, and `HasFPowerSeriesOnBall.hasSum` at that point gives
$$
\text{HasSum}\ \bigl(m \mapsto p_m(z,\dots,z)\bigr)\ \bigl(\text{sum at } z\bigr),
$$
where the base point is the origin, which is why the provider simplifies a
`zero_add`; the same `simpa` also unfolds `matrixPowerSeriesSum`.

Finally CFT-14-002 rewrites each `p_m(z,\dots,z)` into `z^{m}a_m`, turning the
abstract `HasSum` into the concrete one. Two cards of this chapter are used: the
radius bound of CFT-14-003 at the first step, and CFT-14-002 at this last rewrite.

#### Worked instance

For `a_m = T^{m}` with `‖T‖ \le 1` and `|z| < 1`, the partial sums of
`\sum z^{m}T^{m}` converge to the value of the sum function at `z`. The classical
next step identifies that limit with `(I-zT)^{-1}`; this chapter does not take it.

#### Boundary case

At `|z| = 1` the conclusion can fail, as the `a_m = I`, `z = 1` instance shows;
the hypothesis is genuinely needed and is not an artefact of the proof.

#### Historical context

Separating "the sum function is defined" from "the series converges to it" is a
formalization artefact with a real payoff: the function can be named and reasoned
about before the convergence region is known, and the convergence card then
attaches to a fixed object.
Source boundary: Mathlib 4.32.1 analytic declarations, and the registered Crouzeix geometry packet, which owns the provider.
Review status: the repetition of CFT-14-005's first three steps reviewed as deliberate.

#### ML analogy

Mathematical object: convergence of an operator series to its sum function, inside the radius.
ML counterpart: the guarantee that truncating an operator expansion converges to the intended limit for a contraction.
Exact transfer: inside the disk the partial sums converge to the named function, unconditionally in the summation order, since `HasSum` is an unordered notion.
Non-transfer: no rate is given, so the number of terms needed for a tolerance does not follow; and nothing holds on the boundary.
Diagnostic: partial sums that wander rather than converge indicate an argument on or outside the unit circle.

#### Pedagogical prerequisites

CFT-14-002, CFT-14-003 and CFT-14-004.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.matrix_power_series_converges`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.matrixPowerSeries_hasSum`.
Substantive provider: `CrouzeixConjecture.matrixPowerSeries_hasSum`, which repeats the ball construction of CFT-14-005 and finishes with `HasFPowerSeriesOnBall.hasSum` and a rewrite by `matrixPowerSeries_apply`.
Readable type map: `a` is the coefficient sequence, `C` the bound, and `z` the point of the open disk, taken as an implicit binder.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L20).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] (a : Nat → CrouzeixConjecture.SquareMatrix.{u_1} n) (C : Real), (∀ (m : Nat), LE.le.{0} (Norm.norm.{u_1} (a m)) C) → ∀ {z : Complex}, Membership.mem.{0, 0} CrouzeixConjecture.unitDisk z → HasSum.{u_1, 0} (fun m => HSMul.hSMul.{0, u_1, u_1} (HPow.hPow.{0, 0, 0} z m) (a m)) (CrouzeixConjecture.matrixPowerSeriesSum.{u_1} a z)`.
Type SHA-256: `68ba8eb2341b7082a7ba51f7586b5ce73c76785f9e26ea77ac1a78ee72451fc2`.
Direct maintained dependencies: `CrouzeixConjecture.matrixPowerSeries_hasSum`, which calls `one_le_matrixPowerSeries_radius` and `matrixPowerSeries_apply`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: a uniform bound `C` and membership of `z` in the open unit disk.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:68ba8eb2341b7082a7ba51f7586b5ce73c76785f9e26ea77ac1a78ee72451fc2`.

#### Exercises and solutions

CFT-14-E04 states this together with the `tsum` identification it yields.

## Worked examples

**A contraction's geometric series.** For `‖T‖ \le 1` and `a_m = T^{m}`, the
coefficient norms are bounded by `1` — submultiplicativity for `m \ge 1`, and the
identity's induced norm at `m = 0` — so `C = 1` and every card applies: radius
at least one, analytic on the open unit disk, and inside that disk the sum is the
limit of `\sum z^{m}T^{m}`. What the chapter does *not* do is identify that limit
with `(I-zT)^{-1}`.

**Where the coefficient bound fails.** For `a_m = m!\cdot I` no uniform `C`
exists, the radius is zero, and none of CFT-14-003, CFT-14-005 or CFT-14-006
applies. The definitions of CFT-14-001 and CFT-14-004 still produce objects; the
sum function returns junk away from the origin.

**Unbounded coefficients with radius one.** For `a_m = m\cdot I` over a nonempty
index type the coefficient norms are `m`, unbounded, so CFT-14-003's hypothesis
fails, yet the radius is exactly one. This is the witness that the coefficient
bound is sufficient and not necessary. It is not compiled. Nonemptiness matters
for the witness: over an empty index type the matrix ring is trivial, `I = 0`, and
every coefficient sequence is bounded — the cards themselves hold for any index
type, but the counterexamples do not.

**The boundary.** For `a_m = I` and `z = 1` the terms are constantly `I`, so no
sum exists — the terms do not tend to zero. The sum function of CFT-14-004 is
nonetheless defined at `z = 1`, and its value there means nothing. Exercise E05
compiles the fact that `1` is outside the open disk and `0` is inside.

**The finite geometric identity.** For any `T` and any `N`,
$$
\Bigl(\sum_{i<N}T^{i}\Bigr)(I-T)=I-T^{N},
$$
pure ring algebra with no analysis and no hypothesis on `T`. This is the identity
a proof of the Neumann formula starts from; exercise E02 compiles it, and the
chapter goes no further along that road.

## ML bridge

**Mathematical object.** An operator-valued power series in a scalar parameter,
its radius of convergence, and the analyticity of its sum strictly inside that
radius.

**ML counterpart.** A truncated operator expansion — Neumann, Chebyshev, or a
polynomial filter — used to apply an inverse or a function of a matrix without
forming it.

**Exact transfer.** A uniform bound on the coefficient norms guarantees
convergence strictly inside the unit disk, and the limit is unconditional in the
summation order, since `HasSum` is an unordered notion.

**Non-transfer.** No rate follows, so the number of terms needed for a tolerance
is outside this chapter. Nothing holds on the boundary circle, which is precisely
where an ill-conditioned problem sits. And nothing here identifies the sum with an
inverse, which is the step the application actually wants.

**Diagnostic.** A truncated expansion whose partial sums wander rather than settle
indicates a parameter on or outside the unit circle, or coefficients without a
uniform bound — the two hypotheses the cards carry.

## Lean translation

`CrouzeixTextbook.Part03.Chapter14` exposes six cards and six checked exercise
solutions. CFT-14-001 and CFT-14-004 are `definition` rows naming
`CrouzeixConjecture.matrixPowerSeries` and `matrixPowerSeriesSum`; the other four
are `reexported-proof` rows over `CrouzeixConjecture.MatrixPowerSeries`.

Both definition rows had been registered as `checkpoint` with no underlying
declaration. They alias definitions, not theorems, so `definition` is their
correct mode; the same mis-registration is present in several later chapters and
is corrected as each is completed.

Of the four theorem providers, one is a single `simp` — CFT-14-002's, and its card
says so. The other three are genuine multi-step proofs, and CFT-14-005 and
CFT-14-006 share their first three steps, which the cards note rather than leave
the reader to discover.

One supporting declaration is compiled but carries no card:
`matrixPowerSeriesSum_zero`, the value at the origin. It is recorded by exercise
E01.

Two graphs are recorded separately and should not be read as one. Each card's
*Pedagogical prerequisites* names what a reader needs before the card and is the
edge set the contract's reviewed prerequisite policy constrains; each card's
*Direct maintained dependencies* names what the checked proof actually calls. They
differ on purpose — CFT-14-001, for instance, is preceded pedagogically by
Chapter 6's polynomial actions and Chapter 9's norm while its Lean type depends on
neither.

Not compiled, and not claimed: the Neumann series and any inverse, absolute
convergence in a general Banach space, the spectral radius formula, an exact
radius for any series, any statement at `|z| = 1`, any rate of convergence, and
convergence of operator sequences in any sense other than this one. The earlier
sketch of this chapter set six exercises: Banach-valued absolute convergence, a
scalar geometric computation, the finite geometric identity, the Neumann inverse
theorem, a nilpotent norm example, and a reading of the radius bound. The finite
identity survives as E02 and the radius reading as E06; the other four are
withdrawn. The sketch also displayed four theorems, including continuity of
inversion, none of which is compiled.

## Exercises with complete solutions

### CFT-14-E01 — evaluation and the value at the origin {#exercise-cft-14-e01}

Record how a term of the series evaluates, and what the sum is at the origin.

#### Complete written solution

The first conjunct is CFT-14-002: the `m`-th term on a repeated argument `z` is
`z^{m}a_m`.

The second is the value at the origin. Every term with `m \ge 1` carries a factor
`0^{m} = 0`, so only the constant term survives and the sum is `a_0`. The compiled
proof of this is `matrixPowerSeriesSum_zero`, a declaration of the provider module
that carries no card of its own: it changes the goal into a `tsum` of terms,
rewrites that into the `tsum` of `0^{m}\cdot a_m` using CFT-14-002, and closes
with `(HasSum.hasSum_at_zero a).tsum_eq` — the `tsum_eq` being what turns
Mathlib's `HasSum` statement into the equation the goal wants.

Note that this is the one value of the sum function available with no convergence
hypothesis at all, which is why it is worth recording separately from CFT-14-006.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter14.exercise_01_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.matrixPowerSeries_apply` (CFT-14-002's provider) and `CrouzeixConjecture.matrixPowerSeriesSum_zero`, a supporting declaration with no card.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L27).
Type SHA-256: `3d7636dc759838e43b97859829dc1dc6e049c21ad320aa5de7a055fdaef6e246`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:3d7636dc759838e43b97859829dc1dc6e049c21ad320aa5de7a055fdaef6e246`.

### CFT-14-E02 — the finite geometric identity {#exercise-cft-14-e02}

Prove the finite identity that a derivation of the Neumann series starts from.

#### Complete written solution

For any square `T` and any `N`,
$$
\Bigl(\sum_{i<N}T^{i}\Bigr)(I-T)=I-T^{N}.
$$
Expanding the product telescopes: the `i`-th term contributes `T^{i}-T^{i+1}`, and
adjacent terms cancel, leaving `T^{0}-T^{N}`. No hypothesis on `T` is needed and no
analysis is involved — this is an identity in any ring, and the checked solution is
Mathlib's `geom_sum_mul_neg` at `T`.

What it is *not* is the Neumann series. Letting `N \to \infty` and inverting
requires convergence of `T^{N}` to zero and invertibility of `I-T`, neither of
which is compiled in this chapter. The identity is where such a proof begins, and
the chapter supplies only that starting point.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter14.exercise_02_solution`.
Formal mode: `proved-here`.
Provider boundary: Mathlib's `geom_sum_mul_neg`; a ring identity with no hypothesis on `T`, no analysis, and no Crouzeix provider.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L34).
Type SHA-256: `39bb2fb60ed0c80ad5861444aa103b66e7bafff3a6c3f13c4ace00e7b84902a7`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:39bb2fb60ed0c80ad5861444aa103b66e7bafff3a6c3f13c4ace00e7b84902a7`.

### CFT-14-E03 — bound to analyticity {#exercise-cft-14-e03}

From a uniform coefficient bound, derive both the radius estimate and the
analyticity of the sum.

#### Complete written solution

Both conclusions follow from the single hypothesis `‖a_m‖ \le C`. The radius
estimate is CFT-14-003, by Mathlib's bound criterion. The analyticity is
CFT-14-005, which consumes that radius estimate and restricts the resulting
power-series ball to radius one.

Stating them together makes the dependency visible in the type: the same `C` that
bounds the coefficients is what puts the disk inside the region of convergence.
The checked solution is the pair of card applications and adds no proof of its
own.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter14.exercise_03_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.one_le_matrixPowerSeries_radius` and `matrixPowerSeriesSum_analyticOnNhd_unitDisk`, the providers of CFT-14-003 and CFT-14-005; the exercise adds the pairing, not a proof.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L40).
Type SHA-256: `22dc64980552eec2b2d4058ee85961b2835ea07389bf17be58308b3fefad6d03`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:22dc64980552eec2b2d4058ee85961b2835ea07389bf17be58308b3fefad6d03`.

### CFT-14-E04 — the sum is the tsum {#exercise-cft-14-e04}

Inside the disk, identify the sum function with the unordered sum of the
monomials.

#### Complete written solution

CFT-14-006 gives `HasSum (m ↦ z^{m}a_m) (\text{sum at } z)` for `z` in the open
disk. A family with a sum has that sum as its `tsum`, so
`\sum'_m z^{m}a_m` equals the sum function at `z`; the checked solution obtains the
second conjunct from the first by `HasSum.tsum_eq`.

The distinction matters because CFT-14-004 defines the sum function *as* a `tsum`
unconditionally. Outside the disk that `tsum` is a junk value and this identity,
though still true by definition, carries no information. What the exercise records
is that inside the disk the `tsum` is a genuine limit.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter14.exercise_04_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.matrixPowerSeries_hasSum` (CFT-14-006's provider) and Mathlib's `HasSum.tsum_eq`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L48).
Type SHA-256: `8e5a15bb9fb6175c991a8e8194581df5cfbfc67416a6f561a6b73b405295808e`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:8e5a15bb9fb6175c991a8e8194581df5cfbfc67416a6f561a6b73b405295808e`.

### CFT-14-E05 — the disk is open {#exercise-cft-14-e05}

Record that the region the chapter's conclusions cover excludes the boundary.

#### Complete written solution

`unitDisk` is `Metric.ball 0 1`, the *open* ball. So `1` does not belong to it,
since `‖1‖ = 1` is not strictly less than `1`, while `0` does — that is the order
the checked statement uses. Both conjuncts follow by unfolding the definition.

The point is not the arithmetic but the scope of every other card. CFT-14-005
claims analyticity on this set and CFT-14-006 claims convergence at its points;
neither says anything at `|z| = 1`. That silence is not an oversight: for
`a_m = I` the series is geometric, its radius is exactly one, and at `z = 1` the
terms are constantly `I`, so no sum exists — while CFT-14-004's definition still
returns a value there. This exercise is the compiled anchor for that boundary
discussion; the divergence example itself is not compiled.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter14.exercise_05_solution`.
Formal mode: `proved-here`.
Provider boundary: the definition of `unitDisk` as `Metric.ball 0 1`, unfolded by `simp`; no Crouzeix provider and no divergence claim.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L57).
Type SHA-256: `ed0bf87c2ded5b16942f7307615901fe9ebab61e50bfa6f3ea40cd963b9771f7`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:ed0bf87c2ded5b16942f7307615901fe9ebab61e50bfa6f3ea40cd963b9771f7`.

### CFT-14-E06 — the bound at a constant sequence {#exercise-cft-14-e06}

Inspect why a coefficient bound yields radius at least one, by instantiating it
where the bound is immediate.

#### Complete written solution

Take the constant sequence `a_m = B`. Then `‖a_m‖ = ‖B‖` for every `m`, so the
uniform bound holds with `C = ‖B‖` and each instance is `le_rfl` — the bound is
not estimated, it is an equality.

CFT-14-003 then gives radius at least one. Reading the two conjuncts together
shows what the card's hypothesis actually asks for: not a bound that decays, and
not a bound related to `z`, but a single constant dominating every coefficient
norm. For the constant sequence that constant is the coefficient's own norm, which
is why this is the smallest instance that exhibits the mechanism.

The corresponding series is the matrix geometric series `\sum z^{m}B`, whose sum
is analytic on the open unit disk by CFT-14-005 — a conclusion this exercise does
not state.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter14.exercise_06_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.one_le_matrixPowerSeries_radius` (CFT-14-003's provider) at the constant sequence, with both bound instances discharged by `le_rfl`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter14.lean#L64).
Type SHA-256: `02e273182f4d94f929bd17c63517f2ba981c6526b1178290d92340e1d8a43410`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:02e273182f4d94f929bd17c63517f2ba981c6526b1178290d92340e1d8a43410`.

## Synthesis and forward dependencies

The chapter contributes a matrix-valued power series, a sufficient condition for
it to converge on the open unit disk, and the analyticity of its sum there.

Chapter 15 turns to complex differentiability proper, where analyticity stops
being imported from a series and becomes the object of study. The cards here are
consumed in Part VI, where both constant-two routes build a matrix-valued function
on the disk from a bounded coefficient sequence and need exactly this package:
radius, analyticity, and the identification of the sum with the limit of its
monomials.
