---
id: cft-chapter-29-crouzeix-problem-and-sharpness
title: The Crouzeix problem and sharpness
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-29
tags: [crouzeix-textbook, constant-two-routes, mathematics, lean]
confidence: high
canonical: 29_crouzeix_problem_and_sharpness.md
chapter: 29
part: 6
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 29: The Crouzeix problem and sharpness

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-vi-constant-two-routes|Part VI — Constant-two routes]]
Previous: [[knowledge/crouzeix_textbook/part_05_crouzeix_machinery/28_complete_power_family|Chapter 28 — The complete power family]]
Next: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/30_jin_positive_real_completion|Chapter 30 — Jin's positive-real completion]]

## Opening problem

For a complex matrix `A` and a polynomial `p`, compare the scalar quantity
`M(A,p)=max_{z∈W(A)}|p(z)|` with the operator norm `‖p(A)‖`. The Crouzeix
bound says

$$\|p(A)\|\le 2M(A,p).$$

The number two has two logically separate jobs. An upper-bound proof must work
for every finite complex matrix and polynomial. A sharpness proof must exhibit
one pair `(A,p)` whose ratio is two. This chapter states the common target,
performs scalar normalization without dividing by zero, and proves sharpness
by an exact two-dimensional calculation.

The running witness is

$$A_{0,2}=\begin{pmatrix}0&2\\0&0\end{pmatrix}.$$

For `p(z)=z`, its numerical range is the closed unit disk and its norm is two.
The ratio `2/1=2` is exactly attained. This is not a limiting sharpness claim.

## Conceptual model

There are three scales to keep apart. Affine changes of the matrix transport
the geometry. Multiplying the polynomial transports both sides of the norm
inequality. Scaling the Jordan witness changes its numerator and denominator
but not their ratio. The proof below writes each transport before using it.

### Affine normalization before the theorem cards

Translation and nonzero scaling do not change the essential ratio. Directly
from the quadratic-form definition,

$$W(A+βI)=W(A)+β,\qquad W(αA)=αW(A).$$

If `B=α^{-1}(A-βI)` and `p_{α,β}(z)=p(αz+β)`, then

$$p(A)=p_{α,β}(B),\qquad
\max_{z\in W(B)}|p_{α,β}(z)|=\max_{w\in W(A)}|p(w)|.$$

The convention `α≠0` matters. Scalar normalization by `M(A,p)` has the same
split: use `q=M^{-1}p` only when `M>0`, and prove `p(A)=0` directly when `M=0`.

## Formal development

### CFT-29-001 — the maximum is an attained value {#cft-29-001}

#### Purpose

The later normalization uses a genuine maximum, not a formal supremum. This
card defines the closed numerical-range maximum and proves existence by compactness,
so choosing a maximizing point does not hide an extra assumption.

#### Definitions and notation

For a finite nonempty index type, `W(A)` is the set of quadratic forms
`⟨x,Ax⟩` over unit vectors. Put `M(A,p)=sSup {|p(z)|:z∈W(A)}`. Lean calls
this `maxPolynomialModulusOnNumericalRange A p`.

#### Statement

For every positive-dimensional finite complex square matrix `A` and complex
polynomial `p`, there is `z_*∈W(A)` such that

$$|p(z_*)|=M(A,p)=\max_{z\in W(A)}|p(z)|.$$

#### Hypothesis ledger

`Fintype` gives a finite-dimensional compact unit sphere. `Nonempty` rules out
the zero-dimensional sphere. `DecidableEq` supports matrices. Polynomial
evaluation and the complex norm are continuous.

#### Proof roadmap

Represent `W(A)` as the continuous image of the unit sphere, infer compactness,
maximize `z↦|p(z)|`, then identify the attained value with the defining supremum.

#### Proof

We define the closed numerical-range maximum and prove existence by compactness.
The unit sphere `S={x:‖x‖=1}` is compact, and `x↦⟨x,Ax⟩` is continuous. Hence

$$W(A)=\{\langle x,Ax\rangle:x\in S\}$$

is compact and nonempty. The continuous map `z↦|p(z)|` has a maximizer `z_*`.
Its value belongs to the image set and is an upper bound for it. Therefore

$$\operatorname{sSup}\{|p(z)|:z\in W(A)\}=|p(z_*)|.$$

#### Worked instance

For `[λ]`, the only unit-vector quadratic form is `λ`. Thus `W([λ])={λ}`
and `M([λ],p)=|p(λ)|`; the maximizing point is forced.

#### Boundary case

For an infinite-dimensional bounded operator, the numerical range need not be
closed. One uses its closure or a supremum formulation. This card makes only
the finite-matrix compactness claim.

#### Historical context

Source boundary: `MATHLIB-4.32.1` supplies compactness machinery and `CROUZEIX-PACKET` fixes the finite-matrix numerical-range convention.

Review status: the compact image, nonemptiness, continuous maximization, and supremum identification are compiler checked; no priority or peer-review claim is made.

#### ML analogy

Mathematical object: a continuous worst-case loss on a compact structured state set.

ML counterpart: adversarial evaluation over a compact, exactly specified perturbation set.

Exact transfer: compactness plus continuity guarantees an attained worst case in either problem.

Non-transfer: a finite attack search or sampled validation set need not contain the true maximizer.

Diagnostic: compare the largest sampled value with an independently certified upper bound and report the unresolved gap.

#### Pedagogical prerequisites

Compactness in finite-dimensional normed spaces, continuous images, polynomial
continuity, supremum, and the numerical-range definition.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part06.max_polynomial_modulus`. Formal mode: `proved-here`. Substantive provider: `CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.exists_maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.norm_polynomial_eval_le_maxOnNumericalRange, CrouzeixConjecture.numericalRange`. Readable type map: for every finite nonempty matrix index, matrix `A`, and polynomial `p`, a point of `W(A)` attains the maximum and bounds every other point. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L150). Compiler receipt: fresh canonical run. Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n) (p : Polynomial.{0} Complex), Exists.{1} fun z => And (Membership.mem.{0, 0} (CrouzeixConjecture.numericalRange.{u_1} A) z) (And (Eq.{1} (Norm.norm.{0} (Polynomial.eval.{0} z p)) (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{u_1} A p)) (∀ (w : Complex), Membership.mem.{0, 0} (CrouzeixConjecture.numericalRange.{u_1} A) w → LE.le.{0} (Norm.norm.{0} (Polynomial.eval.{0} w p)) (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{u_1} A p)))`. Type SHA-256: `3799044d1d7cf216e1d5a16d6f176195a40d2f60105cb22cc17066c56df6e3f4`. Direct maintained dependencies: `CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.exists_maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.norm_polynomial_eval_le_maxOnNumericalRange, CrouzeixConjecture.numericalRange`. Axioms: `Classical.choice, Quot.sound, propext`. Assumptions: finite, decidable, nonempty index type, matrix, and polynomial. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:3799044d1d7cf216e1d5a16d6f176195a40d2f60105cb22cc17066c56df6e3f4`.

#### Exercises and solutions

CFT-29-E01 reconstructs the special branch in which the scalar maximum is zero.
It substitutes the zero maximum into the displayed norm inequality and uses
norm definiteness, so the exercise checks the one branch where normalization
by an inverse would be illegal.

### CFT-29-002 — the polynomial bound and its zero branch {#cft-29-002}

#### Purpose

Every route produces a normalized estimate with maximum one. This card states
the quantified polynomial bound and its zero-maximum edge case, preventing the
invalid move of dividing by `M` before proving `M>0`.

#### Definitions and notation

`PolynomialCrouzeixBound A p` abbreviates
`‖polynomialEval p A‖≤2*M(A,p)`. Polynomial evaluation is the algebraic matrix
functional calculus, not entrywise evaluation.

#### Statement

Given `PolynomialCrouzeixBound A p`, both

$$\|p(A)\|\le2M(A,p)$$

and `M(A,p)=0→p(A)=0` hold. This is quantified over every positive-dimensional
finite complex matrix and polynomial.

#### Hypothesis ledger

Positive dimension gives the numerical-range maximum. No spectrum or
diagonalizability hypothesis occurs. The zero branch uses only nonnegativity
and definiteness of the matrix norm.

#### Proof roadmap

Unfold the bound, retain its inequality, substitute `M=0`, squeeze `‖p(A)‖`
between zero and zero, and use `‖X‖=0↔X=0`.

#### Proof

We state the quantified polynomial bound and its zero-maximum edge case. If
`M(A,p)=0`, the inequality becomes

$$0\le\|p(A)\|\le2\cdot0=0.$$

Thus `‖p(A)‖=0`, so `p(A)=0`. Vanishing at eigenvalues alone would not suffice:
Jordan derivatives can remain. The conclusion uses the full matrix bound.

#### Worked instance

If `p=0`, then `M(A,p)=0` and `p(A)=0`. If `A=[λ]`, the same branch says
`|p(λ)|=0`, hence `p(λ)=0`.

#### Boundary case

For a nontrivial nilpotent Jordan block and `p(z)=z`, the polynomial vanishes
on the spectrum `{0}` but `p(A)=A≠0`. Spectrum-only checking cannot replace
the numerical-range bound.

#### Historical context

Source boundary: `CC-001` in `CROUZEIX-PACKET` records the finite polynomial source claim; this card independently audits the elementary zero branch.

Review status: the predicate expansion and norm-definiteness step are checked in Lean; the universal upper bound remains separately proved.

#### ML analogy

Mathematical object: a homogeneous operator guarantee with a separately handled zero scale.

ML counterpart: normalizing a feature map by its certified maximum magnitude.

Exact transfer: division by a scale is valid only after proving the scale is positive.

Non-transfer: a near-zero empirical maximum does not prove exact zero output.

Diagnostic: branch on the certified scale and log whether normalization used `M>0` or the exact-zero proof.

#### Pedagogical prerequisites

Matrix polynomial evaluation, operator norms, norm definiteness, numerical
ranges, and CFT-29-001.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part06.polynomial_crouzeix_bound`. Formal mode: `proved-here`. Substantive provider: `CrouzeixConjecture.PolynomialCrouzeixBound, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.polynomialEval`. Readable type map: a supplied Crouzeix bound gives both the displayed inequality and the zero-maximum consequence. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L162). Compiler receipt: fresh canonical run. Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n) (p : Polynomial.{0} Complex), CrouzeixConjecture.PolynomialCrouzeixBound.{u_1} A p → And (LE.le.{0} (Norm.norm.{u_1} (CrouzeixConjecture.polynomialEval.{u_1} p A)) (HMul.hMul.{0, 0, 0} (OfNat.ofNat.{0} 2) (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{u_1} A p))) (Eq.{1} (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{u_1} A p) (OfNat.ofNat.{0} 0) → Eq.{u_1 + 1} (CrouzeixConjecture.polynomialEval.{u_1} p A) (OfNat.ofNat.{u_1} 0))`. Type SHA-256: `4d98441cfca7d869d873c43a1e6794f5127c362be399d6291ef85f13f0fb6707`. Direct maintained dependencies: `CrouzeixConjecture.PolynomialCrouzeixBound, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.polynomialEval`. Axioms: `Classical.choice, Quot.sound, propext`. Assumptions: finite nonempty matrix index, polynomial, and the bound hypothesis. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:4d98441cfca7d869d873c43a1e6794f5127c362be399d6291ef85f13f0fb6707`.

#### Exercises and solutions

CFT-29-E02 computes the numerical range of the unscaled Jordan block. It asks
for both inclusions: the coordinate-product estimate gives the upper disk,
while explicit unit vectors realize its boundary and convexity fills its
interior.

### CFT-29-003 — three theorem surfaces, one finite core {#cft-29-003}

#### Purpose

The phrase "Crouzeix theorem" appears at several scopes. This card separates
the finite-matrix polynomial, finite-matrix rational, and Hilbert-space rational
surfaces so that one is not cited as if it directly proved all three.

#### Definitions and notation

`FiniteMatrixMainTheoremStatement` is the all-dimensional finite-matrix
polynomial statement, quantified over the standard index types `Fin d`.
`MainTheoremStatement (n:=n)` is the corresponding fixed-index statement.
`RationalSpectralSetCorollaryStatement` adds rational functions pole-free on
`W(A)`. `HilbertRationalSpectralSetStatement` concerns bounded Hilbert-space
operators and the closed numerical range.

#### Statement

In other words, the public card is all-dimensional. Its exact Lean surface is

$$
\operatorname{FiniteMatrixMainTheoremStatement}
\iff
\bigl[\forall d∈ℕ,\ [\operatorname{Nonempty}(\operatorname{Fin}d)]\quad
\forall A:\operatorname{SquareMatrix}(\operatorname{Fin}d),\ \forall p,
\operatorname{PolynomialCrouzeixBound}(A,p)\bigr].
$$

In compact source notation this is `∀d∈ℕ` and
`A:SquareMatrix(Fin d)`. E06 is the fixed-index equivalence
`MainTheoremStatement (n:=n) ↔ ∀ A p, PolynomialCrouzeixBound A p`.

Polynomial approximation is an additional step to the finite rational surface.
Compression and finite-dimensional limits are additional steps to the Hilbert surface.

#### Hypothesis ledger

`Fintype`, `DecidableEq`, and `Nonempty` describe positive finite dimension.
Polynomials need no pole condition. Rational functions must be pole-free on
the numerical range. Hilbert-space transport needs closure and approximation.

#### Proof roadmap

Unfold `FiniteMatrixMainTheoremStatement`, and then unfold its fixed-index
`MainTheoremStatement` body. Keep that two-level quantifier structure visible.
Then display the rational and Hilbert implication chains and the added theorem
at each arrow, rather than folding them into a definition.

#### Proof

We separate finite-matrix, rational spectral-set, and Hilbert-space theorem surfaces.
The public theorem first says, definitionally,

$$
\operatorname{FiniteMatrixMainTheoremStatement}
=\bigl[\forall d,\ [\operatorname{Nonempty}(\operatorname{Fin}d)],\
\operatorname{Main}(\operatorname{Fin}d)\bigr].
$$

At one fixed finite index type, E06 records the different statement

$$\operatorname{Main}(n)
=\bigl[\forall A,p,\ \|p(A)\|\le2M(A,p)\bigr].$$

For rational `r`, one must prove both

$$q_k(A)\to r(A),\qquad M(A,q_k)\to M(A,r)$$

for pole-safe polynomial approximants. Hilbert-space transport additionally
compresses the operator and passes finite-dimensional estimates to a limit.

#### Worked instance

For `[λ]`, the polynomial statement is `|p(λ)|≤2|p(λ)|`. A rational function
with a pole at `λ` is excluded, which shows why pole-freeness is load-bearing.

#### Boundary case

A polynomial has no finite poles, so its rational embedding is pole-free. The
reverse is false: a general rational function is not a polynomial and needs a
unit denominator in functional calculus.

#### Historical context

Source boundary: `JIN-V4-AUDITED` and `LS-ARXIV-V1` state source-derived finite routes; `HARP-LOCAL-VERIFY` records local compilation, not peer review.

Review status: this card's Lean theorem checks only the finite polynomial surface; rational and Hilbert-space consequences retain separate hypotheses and code links.

#### ML analogy

Mathematical object: related theorem interfaces connected by explicit transport lemmas.

ML counterpart: a core finite-model guarantee, an extended function class, and a deployment-space limit theorem.

Exact transfer: a guarantee changes scope only through a proved adapter with visible assumptions.

Non-transfer: similar names do not erase approximation error, poles, or limits.

Diagnostic: record each evaluated claim's domain, hypotheses, and exact adapter theorem rather than one benchmark label.

#### Pedagogical prerequisites

Quantified propositions, polynomial and rational functional calculus,
pole-freeness, compression, and closed numerical ranges.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part06.main_theorem_statement`. Formal mode: `proved-here`. Substantive provider: `CrouzeixConjecture.FiniteMatrixMainTheoremStatement, CrouzeixConjecture.PolynomialCrouzeixBound, CrouzeixConjecture.SquareMatrix`. Readable type map: the finite-matrix theorem is the family of polynomial bounds over all finite dimensions. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L180). Compiler receipt: fresh canonical run. Normalized type: `Iff CrouzeixConjecture.FiniteMatrixMainTheoremStatement (∀ (d : Nat) [Nonempty.{1} (Fin d)] (A : CrouzeixConjecture.SquareMatrix.{0} (Fin d)) (p : Polynomial.{0} Complex), CrouzeixConjecture.PolynomialCrouzeixBound.{0} A p)`. Type SHA-256: `c500bddcbe4f0b4927ef40b3effb463212129a1744caca97cd171c2e30a863bd`. Direct maintained dependencies: `CrouzeixConjecture.FiniteMatrixMainTheoremStatement, CrouzeixConjecture.PolynomialCrouzeixBound, CrouzeixConjecture.SquareMatrix`. Axioms: `Classical.choice, Quot.sound, propext`. Assumptions: dimension, nonempty finite index, matrix, and polynomial. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:c500bddcbe4f0b4927ef40b3effb463212129a1744caca97cd171c2e30a863bd`.

Supporting declarations, not the primary card theorem: the finite rational
surface is
[`CrouzeixConjecture.crouzeixRationalSpectralSetCorollary`](../../../formalization/lean/CrouzeixConjecture/FinalTheorems.lean#L26),
whose pointwise form is
[`CrouzeixConjecture.crouzeixRationalBound`](../../../formalization/lean/CrouzeixConjecture/FinalTheorems.lean#L31).
The Hilbert-space rational surface is
[`CrouzeixConjecture.hilbertSpaceRationalSpectralSet`](../../../formalization/lean/CrouzeixConjecture/HilbertSpectralSet.lean#L115).
These links support the two later theorem interfaces; they do not change the
public declaration, normalized type, or receipt identity of CFT-29-003.

#### Exercises and solutions

CFT-29-E03 proves the sharp ratio for the unscaled Jordan block. It combines
the exact operator norm with the exact numerical-range maximum, then performs
the nonzero scalar division rather than appealing to the scaled theorem card.

### CFT-29-004 — the scaled Jordan block has unit-disk numerical range {#cft-29-004}

#### Purpose

Sharpness should be visible at maximum one. This card computes the unit-disk numerical range of the scaled two-by-two Jordan block
instead of citing the radius-one-half computation for a different matrix.

#### Definitions and notation

Let `J_1=(\begin{smallmatrix}0&1\\0&0\end{smallmatrix})` and
`A_{0,2}=2J_1`. For a unit vector
`x=(x_1,x_2)`, the quadratic form is `2\bar x_1x_2`, up to the fixed inner-product convention.

#### Statement

The exact numerical ranges are

$$W(J_1)={z∈ℂ:|z|≤1/2},\qquad W(A_{0,2})={z∈ℂ:|z|≤1}.$$

Consequently `max_{z∈W(A_{0,2})}|z|=1` for `p(z)=z`.

#### Hypothesis ledger

The upper inclusion uses only `‖x‖=1` and `2ab≤a²+b²`. For the reverse
inclusion, checked unit vectors realize the entire boundary circle; the
Toeplitz--Hausdorff convexity theorem then fills its convex hull. Scaling
transports the radius-one-half disk to the unit disk.

#### Proof roadmap

Compute the quadratic form, prove its radius bound, realize every point of the
boundary circle, invoke `numericalRange_convex` and
`convexHull_sphere_eq_closedBall`, then scale the unscaled result and evaluate
the identity maximum.

#### Proof

We compute the unit-disk numerical range of the scaled two-by-two Jordan block.
For `‖x‖=1`,

$$⟨x,A_{0,2}x⟩=2\bar x_1x_2,
\qquad |⟨x,A_{0,2}x⟩|
\le |x_1|^2+|x_2|^2=1.$$

For the reverse inclusion, first work with `J_1`. If `|z|=1/2`, put
`s=√2` and

$$x=(s^{-1},sz).$$

Then

$$\|x\|^2=s^{-2}+s^2|z|^2=\frac12+2\cdot\frac14=1,
\qquad \langle x,J_1x\rangle=\overline{s^{-1}}(sz)=z.$$

Thus the radius-one-half boundary circle lies in `W(J_1)`. The numerical
range is convex by `numericalRange_convex`; since the convex hull of that
circle is its closed disk by `convexHull_sphere_eq_closedBall`, the entire
closed disk lies in `W(J_1)`. Together with the upper inclusion this proves
`W(J_1)=closedBall(0,1/2)`. Finally, `W(2J_1)=2W(J_1)` gives the unit disk
for `A_{0,2}`. This is the same boundary-plus-convexity construction checked
in Lean; no unproved choice of interior amplitudes is hidden.

#### Worked instance

At `x=(1/√2,e^{iθ}/√2)`, the value is `e^{iθ}`. Every boundary phase is
attained. At `x=(1,0)`, the value is zero.

#### Boundary case

Both Jordan blocks have spectrum `{0}`. Their numerical ranges have nonzero
area, so replacing numerical range by spectrum destroys the sharpness calculation.

#### Historical context

Source boundary: `CC-002` in `CROUZEIX-PACKET` records the scaled witness; `JIN-V4-AUDITED` supplies pinned source lineage.

Review status: the scaled and unscaled matrices, both disk radii, phase realization, and identity maximum are checked separately; no priority claim is made.

#### ML analogy

Mathematical object: a nilpotent operator whose quadratic-form image is a disk.

ML counterpart: a two-state feed-forward coupling with zero eigenvalue but nonzero one-step amplification.

Exact transfer: the displayed two-state linear map has the same matrix calculation.

Non-transfer: the witness says nothing about data, optimizers, or typical trained-model trajectories.

Diagnostic: compare spectral radius with a numerical-range boundary approximation to isolate nonnormal geometry.

#### Pedagogical prerequisites

Complex inner products, arithmetic-geometric mean, polar form, two-dimensional
unit vectors, and numerical-range scaling.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part06.jordan_two_maximum`. Formal mode: `proved-here`. Substantive provider: `CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.jordanNilpotentTwo, CrouzeixConjecture.maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.numericalRange`. Readable type map: the scaled block `2J` has unit-disk numerical range and identity-polynomial maximum one. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L188). Compiler receipt: fresh canonical run. Normalized type: `And (Eq.{1} (CrouzeixConjecture.numericalRange.{0} (HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) CrouzeixConjecture.jordanNilpotentTwo)) (Metric.closedBall.{0} (OfNat.ofNat.{0} 0) (OfNat.ofNat.{0} 1))) (Eq.{1} (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{0} (HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) CrouzeixConjecture.jordanNilpotentTwo) Polynomial.X.{0}) (OfNat.ofNat.{0} 1))`. Type SHA-256: `4fcca18fce47dfeafad4133214ea02a50d523e160c2cca7be86a383beef98641`. Direct maintained dependencies: `CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.jordanNilpotentTwo, CrouzeixConjecture.maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.numericalRange`. Axioms: `Classical.choice, Quot.sound, propext`. Assumptions: none beyond the fixed two-dimensional scaled witness. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:4fcca18fce47dfeafad4133214ea02a50d523e160c2cca7be86a383beef98641`.

#### Exercises and solutions

CFT-29-E04 proves the contrasting constant-one result for normal matrices. It
uses unitary diagonalization to identify the norm of the polynomial action and
then places every eigenvalue in the numerical range, isolating precisely what
fails for the nonnormal Jordan witness.

### CFT-29-005 — exact norm-two attainment {#cft-29-005}

#### Purpose

The disk supplies the denominator. This card evaluates the identity polynomial and calculates norm two against maximum one,
completing sharpness without approximation or an epsilon argument.

#### Definitions and notation

For `p(z)=z`, polynomial calculus gives `p(A)=A`. The induced Euclidean norm
is `sup_{‖x‖=1}‖Ax‖`. The ratio is `R(A,p)=‖p(A)‖/M(A,p)` when `M>0`.

#### Statement

For `A_{0,2}` and `p(z)=z`,

$$\|p(A_{0,2})\|=2,\qquad M(A_{0,2},p)=1,
\qquad R(A_{0,2},p)=2.$$

The unscaled witness has `‖J_1‖=1`, `M(J_1,p)=1/2`, and the same ratio.

#### Hypothesis ledger

No universal upper bound enters. Evaluation of `X` gives the matrix. A
coordinate estimate gives the norm upper bound, and `e_2` attains it.
CFT-29-004 supplies the exact maxima.

#### Proof roadmap

Evaluate `X(A)`, calculate its action, prove and attain the norm bound, then
divide by the positive maximum.

#### Proof

We evaluate the identity polynomial and calculate norm two against maximum one.
For `x=(x_1,x_2)`,

$$A_{0,2}x=(2x_2,0),\qquad
\|A_{0,2}x\|=2|x_2|\le2\|x\|.$$

Taking `x=e_2` gives equality, so `‖A_{0,2}‖=2`. Since `X(A)=A`,

$$\frac{\|X(A_{0,2})\|}
{\max_{z\in W(A_{0,2})}|X(z)|}=\frac21=2.$$

This endpoint is exactly attained. It is not a limiting sharpness claim.

#### Worked instance

Scaling down gives `J_1`: numerator and denominator become `1` and `1/2`.
Their ratio remains two.

#### Boundary case

Outer-domain proofs use epsilon thickenings `Ω_ε` with maxima `M_ε→M`.
That limit belongs to the upper-bound proof. The lower-bound witness attains
its denominator on `W(A)` itself.

#### Historical context

Source boundary: `CC-002` identifies the elementary witness; the compiler checks the scaled equality for `A_{0,2}` and the supporting calculation records its relation to the unscaled block.

Review status: matrix action, attaining vector, maximum, positive denominator, and both ratios are locally checked; upper-bound publication review is separate.

#### ML analogy

Mathematical object: an exact worst-case amplification ratio for a fixed nonnormal map.

ML counterpart: exact adversarial gain of a two-state linear layer under a fixed norm.

Exact transfer: the matrix-vector norm calculation and maximizing input are identical.

Non-transfer: one exact witness does not predict average loss or trained nonlinear networks. No trained-network claim is made.

Diagnostic: solve the top singular-vector problem and compare its gain with the maximum scalar filter magnitude on a numerical-range approximation.

#### Pedagogical prerequisites

Induced norms, finite-dimensional norm attainment, polynomial evaluation of
`X`, and CFT-29-004.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part06.jordan_two_attains_two`. Formal mode: `proved-here`. Substantive provider: `CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.jordanNilpotentTwo, CrouzeixConjecture.maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.polynomialEval`. Readable type map: on `2J`, the identity evaluation has norm two, maximum one, ratio two, and equality in the factor-two bound. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L197). Compiler receipt: fresh canonical run. Normalized type: `And (Eq.{1} (Norm.norm.{0} (CrouzeixConjecture.polynomialEval.{0} Polynomial.X.{0} (HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) CrouzeixConjecture.jordanNilpotentTwo))) (OfNat.ofNat.{0} 2)) (And (Eq.{1} (HDiv.hDiv.{0, 0, 0} (Norm.norm.{0} (CrouzeixConjecture.polynomialEval.{0} Polynomial.X.{0} (HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) CrouzeixConjecture.jordanNilpotentTwo))) (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{0} (HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) CrouzeixConjecture.jordanNilpotentTwo) Polynomial.X.{0})) (OfNat.ofNat.{0} 2)) (Eq.{1} (Norm.norm.{0} (CrouzeixConjecture.polynomialEval.{0} Polynomial.X.{0} (HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) CrouzeixConjecture.jordanNilpotentTwo))) (HMul.hMul.{0, 0, 0} (OfNat.ofNat.{0} 2) (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{0} (HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) CrouzeixConjecture.jordanNilpotentTwo) Polynomial.X.{0}))))`. Type SHA-256: `4cf9d3a9c2a267d26959ef5ae89a54abaf2ab714d3fabc2a5ea4e930c55cb7cd`. Direct maintained dependencies: `CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.jordanNilpotentTwo, CrouzeixConjecture.maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.polynomialEval`. Axioms: `Classical.choice, Quot.sound, propext`. Assumptions: none beyond the fixed scaled block and identity polynomial. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:4cf9d3a9c2a267d26959ef5ae89a54abaf2ab714d3fabc2a5ea4e930c55cb7cd`.

#### Exercises and solutions

CFT-29-E05 reconstructs positive-maximum normalization and rescales the actual matrix evaluation.

### CFT-29-006 — two is the least universal constant {#cft-29-006}

#### Purpose

Sharpness gives a lower bound, not the matching upper bound. This card combines
the universal lower bound only with a separately identified upper bound and
records which proof supplies each half.

#### Definitions and notation

`RationalCrouzeixBoundOnFinTwo K` means that every `2×2` matrix and rational
function pole-free on its numerical range satisfy the bound with `K`.
`IsLeast S 2` means `2∈S` and `2≤K` for every `K∈S`.

#### Statement

Two is the least admissible rational constant already in dimension two:

$$\operatorname{IsLeast}
\{K:\operatorname{RationalCrouzeixBoundOnFinTwo}(K)\}\,2.$$

Membership is the universal upper bound; minimality is forced by the Jordan witness.

#### Hypothesis ledger

The upper half calls a separately named rational provider. The lower half
assumes an arbitrary candidate `K`, instantiates it on the scaled witness
`A=A_{0,2}=2J_1` and rational `X`, then uses the exact values `2` and `1`.

#### Proof roadmap

Prove membership of two from the maintained rational theorem. For minimality,
specialize any candidate bound to pole-free `X` on `A=A_{0,2}=2J_1`, rewrite
the exact norm and maximum, and solve `2≤K`.

#### Proof

We combine the universal lower bound only with a separately identified upper bound.
The upper provider proves

$$\|r(A)\|\le2\max_{z\in W(A)}|r(z)|$$

for every admissible pair. If `K` is another candidate, use
`A=A_{0,2}=2J_1`, `r=X`:

$$2=\|X(A_{0,2})\|\le K\cdot1.$$

In the compact notation mirrored by the checked Lean calculation,
`2=‖X(A_{0,2})‖≤K·1`; CFT-29-004 supplies
`max_{z∈W(A_{0,2})}|z|=1`.

Thus `2≤K`. The first line is the upper theorem; the second is sharpness.
The conclusion is deliberately one-way: sharpness does not prove the universal upper bound.

The Lean proof takes the upper bound directly from
[`holomorphicCrouzeixRationalBound`](../../../formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean#L83).
This keeps the common sharpness argument independent of the terminal Jin, LS,
and Harp modules. The upper bound is reused here, not derived from the witness.

#### Worked instance

`K=1.99` would imply `2≤1.99`, a contradiction. `K=3` survives this lower
test but is not least once the separate upper theorem supplies two.

#### Boundary case

This is an ordinary, not completely bounded, statement. Matrix-valued
amplifications and removal of rational pole hypotheses need separate theorems.

#### Historical context

Source boundary: the registered publisher and revision receipts below
[[knowledge/crouzeix_conjecture/source_registry#crouzeix-2007-earlier-universal-numerical-range-bound|CROUZEIX-2007]]
and
[[knowledge/crouzeix_conjecture/source_registry#crouzeix-palencia-2017-one-plus-square-root-two-result|CROUZEIX-PALENCIA-2017]]
identify the earlier universal bound and the 2017 `1+√2` result. The Jin route
is pinned to commit `565b6a3e0659b6e0785f783b016c3f6d9f171fa5`, dated
`2026-08-05T16:32:05+08:00` in the
[[knowledge/crouzeix_conjecture/reproduction_research#public-artifact-sequence|public artifact sequence]],
with the revision identity registered through
[[knowledge/crouzeix_conjecture/source_registry#jin-v4-audited-formalization-matched-git-manuscript|JIN-V4-AUDITED]].
The Lorist--Schwenninger route is pinned to `arxiv:2608.03841v1`, with its
source receipts observed on 2026-08-14, through
[[knowledge/crouzeix_conjecture/source_registry#ls-arxiv-v1-lorist-schwenninger-arxiv-v1|LS-ARXIV-V1]].

The `2026-08-29 Harp review boundary` records a third maintained terminal,
[`CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem`](../../../formalization/lean/Crouzeix/Harp/MainTheorem.lean#L25).
It is linked to the local verification record
[[knowledge/crouzeix_textbook/claim_evidence_ledger#cft-cl-003|CFT-CL-003]]
and the
[[knowledge/crouzeix_textbook/status_and_scope#harp-finite-horizon-verification-boundary|Harp finite-horizon verification boundary]].
This route is Harp-derived, may reuse lower-level Lorist--Schwenninger modules,
and is not a third historical source or an independent third kernel branch.

Review status: the source-derived declarations are locally compiler verified under pinned sources; this is not a peer-review, priority, acceptance, or journal-status claim. The Harp-derived finite-horizon route is local derived work, not a third historical source.

#### ML analogy

Mathematical object: matching universal upper and existential lower bounds for one constant.

ML counterpart: a certified robustness bound paired with an explicit saturating adversarial instance.

Exact transfer: an attained witness refutes every smaller universal constant.

Non-transfer: an adversarial example alone cannot certify every model, dimension, or amplification level.

Diagnostic: track upper certificates and lower witnesses separately and report their gap before calling a constant optimal.

#### Pedagogical prerequisites

Least elements, rational functional calculus, pole-freeness, sharpness, and the
logical difference between universal bounds and existential witnesses.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part06.constant_two_is_least`. Formal mode: `proved-here`. Substantive provider: `CrouzeixConjecture.RationalCrouzeixBoundOnFinTwo, CrouzeixConjecture.RationalPoleFreeOn, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.holomorphicCrouzeixRationalBound, CrouzeixConjecture.jordanNilpotentTwo, CrouzeixConjecture.maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.maxRationalModulusOnNumericalRange, CrouzeixConjecture.maxRationalModulusOnNumericalRange_algebraMap_polynomial, CrouzeixConjecture.numericalRange, CrouzeixConjecture.polynomialEval, CrouzeixConjecture.rationalMatrixEval, CrouzeixConjecture.rationalMatrixEval_algebraMap_polynomial, CrouzeixConjecture.rationalPoleFreeOn_algebraMap_polynomial`. Readable type map: two belongs to the finite-two rational bound set, and every member is at least two. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L213). Compiler receipt: fresh canonical run. Normalized type: `IsLeast.{0} (setOf.{0} fun K => CrouzeixConjecture.RationalCrouzeixBoundOnFinTwo K) (OfNat.ofNat.{0} 2)`. Type SHA-256: `e4e8201aa2ea2f299dac20b63ca70ac72b1e0dfb1ffae78c75485eb6d88f4eed`. Direct maintained dependencies: `CrouzeixConjecture.RationalCrouzeixBoundOnFinTwo, CrouzeixConjecture.RationalPoleFreeOn, CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.holomorphicCrouzeixRationalBound, CrouzeixConjecture.jordanNilpotentTwo, CrouzeixConjecture.maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.maxRationalModulusOnNumericalRange, CrouzeixConjecture.maxRationalModulusOnNumericalRange_algebraMap_polynomial, CrouzeixConjecture.numericalRange, CrouzeixConjecture.polynomialEval, CrouzeixConjecture.rationalMatrixEval, CrouzeixConjecture.rationalMatrixEval_algebraMap_polynomial, CrouzeixConjecture.rationalPoleFreeOn_algebraMap_polynomial`. Axioms: `Classical.choice, Quot.sound, propext`. Assumptions: the upper half uses the maintained rational bound; the lower half tests any proposed constant on the scaled Jordan witness. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:e4e8201aa2ea2f299dac20b63ca70ac72b1e0dfb1ffae78c75485eb6d88f4eed`.

#### Exercises and solutions

CFT-29-E06 expands the fixed-dimension main statement without using this least-constant theorem.

## Worked examples

For a normal matrix, unitary diagonalization gives constant one because the
operator norm of `p(A)` is the largest `|p(λ)|` over eigenvalues, and every
eigenvalue lies in `W(A)`. For `A_{0,2}`, both eigenvalues vanish but the
ratio is two. These examples isolate nonnormality rather than dimension.

For an epsilon-thickened outer domain `Ω_ε`, define
`M_ε=max_{z∈Ω̄_ε}|p(z)|`. Compact convergence of the domains gives `M_ε→M`.
This limit belongs to upper-bound transport; the exact Jordan witness needs no limit.

## ML bridge

For a fixed finite matrix and polynomial filter, define

$$ρ_p(A)=\frac{\|p(A)\|}{\max_{z∈W(A)}|p(z)|}$$

when the denominator is positive. The exact finite-dimensional transfer to a
linear ML model uses the true matrix polynomial and induced Euclidean norm.
For `A_{0,2},X`, `ρ_X(A_{0,2})=2`. A discretized numerical range gives only
an estimate. No nonlinear, time-varying, data-dependent, or trained-network
tightness claim follows.

The affine-consistency residual

$$r_{\mathrm{aff}}
=\frac{d_H(α\widehat W(B)+β,\widehat W(A))}
{1+\operatorname{diam}(\widehat W(A))}$$

checks an implementation of `B=α^{-1}(A-βI)`. A small residual is a diagnostic,
not a proof of exact numerical-range geometry.

## Lean translation

The six public declarations are local theorem proofs in
`CrouzeixTextbook.Part06.Chapter29`, not definition aliases. The scaled
`A_{0,2}` cards expose unit-disk and norm-two facts. Exercises E02 and E03
intentionally use the unscaled `J_1`; their ratio is scale equivalent. CFT-29-006
reconstructs the least-constant proof while naming the upper provider separately.

## Exercises

### CFT-29-E01 — zero maximum {#exercise-cft-29-e01}

Prove the zero-maximum edge case of the polynomial Crouzeix bound.

#### Hint

Unfold the bound, substitute `M=0`, and use definiteness of the matrix norm.

#### Complete written solution

Unpack the matrix polynomial bound and use norm definiteness to show p(A)=0 when the numerical-range maximum is zero. Assume the bound and
`M(A,p)=0`. Then `‖p(A)‖≤2M=0`. Norm nonnegativity
gives equality, and norm definiteness yields `p(A)=0`. No division occurs.

#### Lean solution

Declaration: `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_01_solution`. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L248). Normalized type: `∀ (n : Type) [inst : Fintype.{0} n] [inst_1 : DecidableEq.{1} n] [Nonempty.{1} n] (A : CrouzeixConjecture.SquareMatrix.{0} n) (p : Polynomial.{0} Complex), CrouzeixConjecture.PolynomialCrouzeixBound.{0} A p → Eq.{1} (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{0} A p) (OfNat.ofNat.{0} 0) → Eq.{1} (CrouzeixConjecture.polynomialEval.{0} p A) (OfNat.ofNat.{0} 0)`. Type SHA-256: `59abc989d682ad792f815cc3f84b92a875570b7d614a033fccc0d24c802b8243`. Axioms: `Classical.choice, Quot.sound, propext`. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:59abc989d682ad792f815cc3f84b92a875570b7d614a033fccc0d24c802b8243`.

### CFT-29-E02 — unscaled Jordan disk {#exercise-cft-29-e02}

Compute W(J) for the 2×2 nilpotent Jordan block.

#### Hint

Bound `|\bar x_1x_2|` by `1/2`, then realize every radius and phase.

#### Complete written solution

Parametrize a unit vector, bound the coordinate product, and realize every
phase to prove exact disk equality. For unit `x`,
`|⟨x,J_1x⟩|≤(|x_1|²+|x_2|²)/2=1/2`, proving the upper inclusion. For the
reverse inclusion, let `|z|=1/2`, set `s=√2`, and take
`x=(s^{-1},sz)`. The calculation
`‖x‖²=1/2+2|z|²=1` and
`⟨x,J_1x⟩=conj(s^{-1})sz=z` realizes every point of the boundary circle.
The theorem `numericalRange_convex` says `W(J_1)` is convex, while
`convexHull_sphere_eq_closedBall` identifies the convex hull of that boundary
circle with the radius-one-half closed disk. Therefore the disk lies in the
numerical range, and `W(J_1)={z∈ℂ:|z|≤1/2}`. This follows the checked Lean
proof rather than assuming the existence of unspecified interior amplitudes.

#### Lean solution

Declaration: `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_02_solution`. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L261). Normalized type: `Eq.{1} (CrouzeixConjecture.numericalRange.{0} CrouzeixConjecture.jordanNilpotentTwo) (Metric.closedBall.{0} (OfNat.ofNat.{0} 0) (HDiv.hDiv.{0, 0, 0} (OfNat.ofNat.{0} 1) (OfNat.ofNat.{0} 2)))`. Type SHA-256: `64a8b2319f6c1e152076bd4f21e3b0167a9dbf76750f2d7f0c2c21d73e6263a2`. Axioms: `Classical.choice, Quot.sound, propext`. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:64a8b2319f6c1e152076bd4f21e3b0167a9dbf76750f2d7f0c2c21d73e6263a2`.

### CFT-29-E03 — the unscaled sharp ratio {#exercise-cft-29-e03}

Prove the sharp ratio two for p(z)=z on the Jordan block.

#### Hint

Use `‖J_1‖=1`, maximum `1/2`, and calculate the division.

#### Complete written solution

Combine ‖p(J)‖=1 with the exact numerical-range maximum 1/2 and perform the division. Indeed, `X(J_1)=J_1`, whose norm is one, attained at `e_2`. Exercise E02 gives
`M(J_1,X)=1/2`. Hence `1/(1/2)=2`, the same ratio as the scaled witness.

#### Lean solution

Declaration: `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_03_solution`. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L269). Normalized type: `Eq.{1} (HDiv.hDiv.{0, 0, 0} (Norm.norm.{0} (CrouzeixConjecture.polynomialEval.{0} Polynomial.X.{0} CrouzeixConjecture.jordanNilpotentTwo)) (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{0} CrouzeixConjecture.jordanNilpotentTwo Polynomial.X.{0})) (OfNat.ofNat.{0} 2)`. Type SHA-256: `7ca04c82d187719bf4d59e6f362243bbd28dd2834029784f084e7badfce7c7d7`. Axioms: `Classical.choice, Quot.sound, propext`. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:7ca04c82d187719bf4d59e6f362243bbd28dd2834029784f084e7badfce7c7d7`.

### CFT-29-E04 — normal matrices need only constant one {#exercise-cft-29-e04}

Prove the polynomial numerical-range constant is one for a normal matrix.

#### Hint

Use unitary diagonalization and note that every eigenvalue lies in the numerical range.

#### Complete written solution

Use the normality equation and unitary diagonalization to bound ‖p(A)‖ by the numerical-range maximum. Write `A=U diag(λ_j)U*`. Then
`p(A)=U diag(p(λ_j))U*`, so
`‖p(A)‖=max_j|p(λ_j)|`. Each normalized eigenvector shows `λ_j∈W(A)`. Therefore

$$\|p(A)\|\le\max_{z\in W(A)}|p(z)|.$$

The Jordan witness is not normal.

#### Lean solution

Declaration: `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_04_solution`. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L277). Normalized type: `∀ (n : Type) [inst : Fintype.{0} n] [inst_1 : DecidableEq.{1} n] [Nonempty.{1} n] (A : CrouzeixConjecture.SquareMatrix.{0} n) (p : Polynomial.{0} Complex), Eq.{1} (HMul.hMul.{0, 0, 0} A (Matrix.conjTranspose.{0, 0, 0} A)) (HMul.hMul.{0, 0, 0} (Matrix.conjTranspose.{0, 0, 0} A) A) → LE.le.{0} (Norm.norm.{0} (CrouzeixConjecture.polynomialEval.{0} p A)) (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{0} A p)`. Type SHA-256: `bf32bd17a48dc481b4abd5b148867806166d84f562a7ff39a886cdaa6a1198c6`. Axioms: `Classical.choice, Quot.sound, propext`. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:bf32bd17a48dc481b4abd5b148867806166d84f562a7ff39a886cdaa6a1198c6`.

### CFT-29-E05 — positive-maximum normalization {#exercise-cft-29-e05}

Carry out the M>0 polynomial normalization.

#### Hint

Set `q=M^{-1}p`, apply the normalized bound, and rescale `q(A)` itself.

#### Complete written solution

Set q=M⁻¹p, record maxOnW(q)=1, apply the normalized bound, and rescale the actual
matrix polynomial evaluation. Here `q`, `maxOnW`, and the matrix evaluation
refer to the exact formal objects below. The supporting theorem
`CrouzeixTextbook.Part06.normalized_max_polynomial_modulus` proves that
`M(A,M^{-1}p)=1` is derived from `M(A,p)=M` and `M>0`: maximum attainment
gives both inequalities after the pointwise identity
`|q(z)|=M^{-1}|p(z)|`. Apply the normalized bound and rescale the actual
matrix polynomial evaluation. Since
`q(A)=M^{-1}p(A)`, norm homogeneity gives

$$\|p(A)\|=M\|q(A)\|\le2M.$$

The `M=0` branch is E01. E05 keeps the frozen normalized-maximum hypothesis
so its exercise signature remains stable; `exercise_05_solution` checks that
supplied equality against the independently derived one before rescaling.

Supporting Lean declaration:
`CrouzeixTextbook.Part06.normalized_max_polynomial_modulus`. Formal mode:
`proved-here`. Compiler locator: `Chapter29.lean:120:9`. Code:
[Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L120).
Compiler receipt: fresh compiler-environment supporting-declaration run.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n) (p : Polynomial.{0} ℂ) (M : ℝ), LT.lt.{0} (OfNat.ofNat.{0} 0) M → Eq.{1} (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{u_1} A p) M → Eq.{1} (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{u_1} A (HSMul.hSMul.{0, 0, 0} (Complex.ofReal (Inv.inv.{0} M)) p)) (OfNat.ofNat.{0} 1)`.
Type SHA-256: `907f1242b2662c78c6878597633b91cb7ac60ffc0b11b47b416f011ffabe18ab`.
Direct maintained dependencies: `CrouzeixConjecture.SquareMatrix, CrouzeixConjecture.exists_maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.maxPolynomialModulusOnNumericalRange, CrouzeixConjecture.norm_polynomial_eval_le_maxOnNumericalRange, CrouzeixConjecture.numericalRange`.
Axioms: `Classical.choice, Quot.sound, propext`. Verification target:
`CrouzeixTextbook`. Receipt identity:
`CrouzeixTextbook:907f1242b2662c78c6878597633b91cb7ac60ffc0b11b47b416f011ffabe18ab`.

#### Lean solution

Declaration: `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_05_solution`. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L301). Normalized type: `∀ (n : Type) [inst : Fintype.{0} n] [inst_1 : DecidableEq.{1} n] [Nonempty.{1} n] (A : CrouzeixConjecture.SquareMatrix.{0} n) (p : Polynomial.{0} Complex) (M : Real), LT.lt.{0} (OfNat.ofNat.{0} 0) M → Eq.{1} (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{0} A p) M → Eq.{1} (CrouzeixConjecture.maxPolynomialModulusOnNumericalRange.{0} A (HSMul.hSMul.{0, 0, 0} (Complex.ofReal (Inv.inv.{0} M)) p)) (OfNat.ofNat.{0} 1) → LE.le.{0} (Norm.norm.{0} (CrouzeixConjecture.polynomialEval.{0} (HSMul.hSMul.{0, 0, 0} (Complex.ofReal (Inv.inv.{0} M)) p) A)) (OfNat.ofNat.{0} 2) → LE.le.{0} (Norm.norm.{0} (CrouzeixConjecture.polynomialEval.{0} p A)) (HMul.hMul.{0, 0, 0} (OfNat.ofNat.{0} 2) M)`. Type SHA-256: `a36d04bd9b6b78a573e387ebaff761eaf2150bf9897ba440cd132c4c47ceeb84`. Axioms: `Classical.choice, Quot.sound, propext`. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:a36d04bd9b6b78a573e387ebaff761eaf2150bf9897ba440cd132c4c47ceeb84`.

### CFT-29-E06 — expanded finite theorem surface {#exercise-cft-29-e06}

Restate MainTheoremStatement with every finite-index assumption and its full conclusion explicit.

#### Hint

Unfold the definition in both directions while retaining the same quantifiers.

#### Complete written solution

Quantify the index type, Fintype, DecidableEq, Nonempty, matrix, and polynomial and prove that this expanded surface is exactly MainTheoremStatement. For `n` with these three typeclass assumptions, the
definition is

$$\forall A:\operatorname{SquareMatrix}(n),
  \forall p:\mathbb C[z],
  \operatorname{PolynomialCrouzeixBound}(A,p).$$

Thus MainTheoremStatement = the displayed universally quantified predicate at
this fixed index type. Each implication introduces `A,p` and applies the
supplied statement. Therefore the two propositions are equivalent, and no
terminal theorem is called.

#### Lean solution

Declaration: `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_06_solution`. Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L330). Normalized type: `∀ (n : Type) [inst : Fintype.{0} n] [inst_1 : DecidableEq.{1} n] [inst_2 : Nonempty.{1} n], Iff CrouzeixConjecture.MainTheoremStatement.{0} (∀ (A : CrouzeixConjecture.SquareMatrix.{0} n) (p : Polynomial.{0} Complex), CrouzeixConjecture.PolynomialCrouzeixBound.{0} A p)`. Type SHA-256: `36aa075421a8b7e36ef9b0d1c88dab50dbfc73bdf9dec8d1372278ef2001f6fd`. Axioms: `Classical.choice, Quot.sound, propext`. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:36aa075421a8b7e36ef9b0d1c88dab50dbfc73bdf9dec8d1372278ef2001f6fd`.

## ML lab: affine consistency without a theorem claim

For an estimated numerical-range boundary `\widehat W(A)`, compare it with
the transformed estimate for `B=α^{-1}(A-βI)`. An affine-consistency residual is

$$r_{\mathrm{aff}}
=\frac{d_H(α\widehat W(B)+β,\widehat W(A))}
{1+\operatorname{diam}(\widehat W(A))}.$$

Also compare `p(A)` with `p_{α,β}(B)`. These are diagnostics. No trained-network claim
follows from a small residual, and a sampled boundary does not certify exact geometry.

## Synthesis and forward dependencies

Compactness turns the scalar supremum into a maximum. The polynomial target
has a positive-scale normalization and a separate zero branch. The nilpotent
block attains ratio two, so every universal constant is at least two. A
separate upper theorem supplies the reverse inequality. Chapter 30 begins the
Jin route; Chapter 33 begins the Lorist--Schwenninger route. Both consume the
same common target without depending on each other.
