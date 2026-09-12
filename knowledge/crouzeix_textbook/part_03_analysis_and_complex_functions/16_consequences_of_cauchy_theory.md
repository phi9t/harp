---
id: cft-chapter-16-consequences-of-cauchy-theory
title: Consequences of Cauchy theory
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-12
tags: [crouzeix-textbook, analysis-and-complex-functions, mathematics, lean]
confidence: high
canonical: 16_consequences_of_cauchy_theory.md
chapter: 16
part: 3
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 16: Consequences of Cauchy theory

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iii-analysis-and-complex-functions|Part III — Analysis and complex functions]]
Previous: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/15_complex_differentiability|Chapter 15 — Complex differentiability]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/17_functions_of_matrices_and_operators|Chapter 17 — Functions of matrices and operators]]

## Opening problem

Chapter 15 built a contour integral and proved it computes `f(B)` — diagonalize, apply
`f` to each eigenvalue, conjugate back — whenever `B` has distinct eigenvalues, `f` is
holomorphic on an open convex neighborhood of the closed region, and `W(B)` lies inside
that region. That recipe covers almost every matrix, in the precise sense that matrices
with distinct eigenvalues are dense. It does not cover all of them. The single
Jordan block `\begin{bmatrix}0&1\\0&0\end{bmatrix}` has one eigenvalue, no basis
of eigenvectors, and no diagonalization to run the recipe through.

Crouzeix's problem is a statement about `f(A)` for *every* `A`, so a calculus
that stops at diagonalizable matrices is not enough. The question of this chapter
is how to extend the recipe, and the answer is the one that density arguments
always suggest: define `f(A)` as a limit of values at nearby matrices that do
diagonalize, then prove the limit is well behaved.

The work is entirely in the second half. A limit is easy to write down and worth
nothing until something guarantees it exists, is independent of the approximating
sequence, and respects the algebra. That guarantee is what Cauchy theory supplies,
and it is the sense in which this chapter is a consequence of it.

## Conceptual model

Three descriptions of `f(A)` are in play, and keeping them apart is most of the
chapter.

*The eigenvalue recipe.* For `B` with distinct eigenvalues, `f(B) = X f(\Lambda) X^{-1}`
where `B = X \Lambda X^{-1}`. Concrete, but undefined at defective matrices.

*The contour integral.* `\frac{1}{2\pi i}\oint_\Gamma f(z)(z - A)^{-1}\,dz`, for a
contour `\Gamma` surrounding the numerical range. Defined for every `A`, but it
depends on a contour, and nothing about the formula says the value does not.

*The approximation limit.* `\lim_k f(A_k)` where `A_k \to A` and each `A_k` has
distinct eigenvalues. Defined for every `A` and every `f`, and depending on the
chosen sequence.

Each has a defect the other two repair. The formalization takes the third as the
**definition** and proves the other two agree with it. That ordering is worth
stating plainly, because the chapter title invites the opposite guess: the
Lean development does not define `f(A)` by a Cauchy integral. It defines `f(A)`
as an approximation limit, and Cauchy theory enters as the theorem that the
integral computes that limit — which is simultaneously what proves the limit
exists and what proves it does not depend on the contour.

The dependency runs: Chapter 15's contour machinery gives convergence; convergence
gives the algebraic laws; the laws make `f \mapsto f(A)` a homomorphism on a
neighborhood of the numerical range.

Where that interface is consumed is worth stating precisely, because the obvious guess
is wrong. Chapter 17 does not take it as given: it never names this chapter, does not
import its module, defines `f(A)` by the contour integral rather than the limit, and
re-derives additivity and multiplicativity from nested contours and the resolvent
identity. The two chapters present the same calculus through different architectures,
and reconciling them is outstanding work rather than something this chapter can claim.

## What this chapter compiles, and what it does not

The chapter is titled for the consequences of Cauchy theory, and a reader who
expects the classical list — maximum modulus, the identity theorem, the open
mapping theorem, the residue theorem — will not find it here. None of those is
compiled, and none is stated as a claim of this chapter.

What is compiled is the one consequence the book needs: a holomorphic functional
calculus on matrices, together with the four properties that make it usable. The
Cauchy theory it consumes is Chapter 15's contour development, which enters
through maintained declarations this chapter names rather than reproves.

The six cards are CFT-16-001 through CFT-16-006: the definition, the contour
agreement, the polynomial compatibility, the locality statement, and the additive
and multiplicative laws.

## Formal development

### CFT-16-001 — holomorphic matrix evaluation {#cft-16-001}

#### Purpose

The object the rest of the chapter is about. Every later card is a statement that
this definition behaves like a functional calculus.

#### Statement

For a square matrix `A` and any function `f : \mathbb{C} \to \mathbb{C}`,

    holomorphicMatrixEval A f = limUnder atTop (simpleSpectrumHolomorphicEval A f)

where `simpleSpectrumHolomorphicEval A f k` is the eigenvalue recipe applied to
the `k`-th simple-spectrum approximation of `A`.

#### Hypothesis ledger

None beyond the ambient typeclasses. `f` is an arbitrary function of a complex
variable, not assumed holomorphic, not assumed continuous, not assumed measurable, and
`A` is an arbitrary square matrix.

The ambient typeclasses are not quite empty, and one of them is doing work. `[Nonempty n]`
appears in the normalized type below, and it is needed because the approximating sequence
comes from an existence theorem that requires it. The zero-by-zero case is outside the
definition, not a degenerate instance of it.

This is deliberate and it is the fact that shapes every other card. `limUnder` is
Mathlib's totalized limit: it returns the limit when the filter converges and an
unspecified element of the type when it does not. So `holomorphicMatrixEval` is a
total function, and the hypotheses that appear on CFT-16-002, CFT-16-005 and
CFT-16-006 do not make the expression *defined*. They make it *mean* something.

The distinction matters when reading those cards. A hypothesis discharged there is
never protecting against a missing value; it is establishing convergence, which is
what licenses the identification of the value with something independently
described.

#### Proof roadmap

A definition has no proof. What needs unfolding is the sequence being limited,
which is three definitions deep.

#### Proof

Reading inward:

`SimpleDiagonalization.functionEval hB f = innerConjugation hB.changeBasis (diagonal (fun i => f (hB.eigenvalues i)))`.
This is the eigenvalue recipe in symbols: build the diagonal matrix of `f` applied
to each eigenvalue, conjugate by the eigenvector basis. Note what it reads of `f`:
**only the finitely many values `f(\lambda_i)` at eigenvalues of `B`**. Nothing
else about `f` is consulted. This single observation is the whole proof of
CFT-16-004 and is worth carrying forward.

`simpleSpectrumHolomorphicEval A f k` applies that recipe to
`simpleSpectrumApproximation A k`, using the diagonalization that exists because
that matrix has distinct eigenvalues.

`simpleSpectrumApproximation A k` is `Classical.choose` applied to the existence
statement "there is a matrix with distinct eigenvalues within `1/(k+1)` of `A`".
It is *some* such matrix. The existence theorem behind it is in fact constructive — its
witness is `A + \eta \cdot (D - A)` for an explicit diagonal `D` and a small scalar
`\eta` — but `Classical.choose` does not expose that witness, so the definition is opaque
even though the construction is not. As a Lean function it is deterministic, so the
definition is well formed; mathematically it is a non-canonical choice, and nothing in the
definition explains why a different admissible choice would give the same answer. The later cards are what supply that, by pinning the
limit to descriptions — a contour integral, a polynomial — in which no choice
appears.

#### Boundary case

Take `f` the indicator of the Gaussian rationals `\mathbb{Q} + i\mathbb{Q}` — dense
in `\mathbb{C}`, unlike `\mathbb{Q}` itself, which lies in `\mathbb{R}` and whose
indicator is therefore identically zero, hence holomorphic, off the real line — and let
`A` be any matrix. `holomorphicMatrixEval A f` is a perfectly good matrix, and nothing
in this chapter says anything about it.

Note what cannot be said. The defining sequence has no *reason* to converge, but that
is not the same as knowing it diverges, and the chapter cannot know: the approximating
matrices come from a `Classical.choose`, so whether any of their eigenvalues is a
Gaussian rational is not determined by anything here. If none of them is, every term of
the sequence is the conjugate of the zero diagonal matrix, the sequence is constantly
zero, and the limit exists and equals zero. The honest statement is not that the value
is junk but that this chapter does not determine it.

Nothing in the type flags this, and no card in this chapter distinguishes the
junk from the good values: there is no compiled predicate here saying "this input
was in range". The chapter's honest position is that `holomorphicMatrixEval` is
meaningful exactly where some card supplies convergence, and the cards that do so
are CFT-16-002, CFT-16-003, CFT-16-005 and CFT-16-006. CFT-16-004 is the curious
exception discussed there.

#### Pedagogical prerequisites

CFT-06-006, that matrices with simple spectrum are dense — which is what makes
the approximating sequence exist at all — and CFT-15-004, the eigenvalue recipe
being limited. Not CFT-15-001: the contour integral plays no part in the
definition, and that is the chapter's main structural point.

#### Historical context

Defining `f(A)` by density of the diagonalizable matrices is the elementary route,
and for polynomials it is how the agreement with ordinary matrix algebra is usually
checked. The Riesz–Dunford integral is the route that generalizes to operators on
a Banach space, where no density argument is available. In finite dimensions both
are available and the formalization uses the elementary one as the definition,
which keeps the definition free of measure theory and pushes the analysis into
the theorems.
Source boundary: Mathlib 4.32.1, and the registered Crouzeix geometry packet, which owns the provider.
Review status: registered for content-wave review.

#### ML analogy

Mathematical object: a function of a matrix defined as a limit over perturbed matrices that are easier to compute with.
ML counterpart: defining the behaviour of a model at a degenerate input as the limit of its behaviour at nearby non-degenerate ones.
Exact transfer: the value is pinned only where the limit exists, and the construction's arbitrary choices wash out exactly there.
Non-transfer: the definition is total, so an implementation returns a number on inputs where the limit does not exist, with no error raised. Totality is not validity.
Diagnostic: if a computed `f(A)` changes when the perturbation scheme changes, the input was outside the range where any of this chapter's cards apply.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.holomorphic_matrix_eval`.
Formal mode: `checkpoint`.
Underlying declaration: `CrouzeixConjecture.holomorphicMatrixEval`.
Substantive provider: none — this is a definition, so there is no proof to narrate. `CrouzeixConjecture` is a maintained prefix, so the alias names its target and the definition carries a receipt entry of its own.
Readable type map: `A` is the matrix, `f` the scalar function, and the value a matrix of the same size. No hypotheses appear.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L9).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `{n : Type u_1} → [Fintype.{u_1} n] → [DecidableEq.{u_1 + 1} n] → [Nonempty.{u_1 + 1} n] → CrouzeixConjecture.SquareMatrix.{u_1} n → (Complex → Complex) → CrouzeixConjecture.SquareMatrix.{u_1} n`.
Type SHA-256: `f1e22464e5b075a93017529bf5d7b9acaf1e15a022a5956c8ba242f925f0b260`.
Direct maintained dependencies: `CrouzeixConjecture.holomorphicMatrixEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:f1e22464e5b075a93017529bf5d7b9acaf1e15a022a5956c8ba242f925f0b260`.

### CFT-16-002 — an admissible contour computes the limit {#cft-16-002}

#### Purpose

The bridge from Chapter 15. It is what makes the definition above deserve the name
"holomorphic functional calculus" rather than "limit of a chosen perturbation".

#### Statement

Let `\Gamma` be an oriented radial convex boundary for `\Omega`, let `V` be open and
convex with `\overline{\Omega} \subseteq V`, let `f` be differentiable on `V`, and
let `A` satisfy `W(A) \subseteq \Omega`. Then the parametric boundary integral of
`f` against `A` over `\Gamma` equals `holomorphicMatrixEval A f`.

#### Hypothesis ledger

Four hypotheses, and they do different jobs.

`hf : DifferentiableOn \mathbb{C} f V` on an open convex `V` containing the closure of
`\Omega` is the holomorphy assumption, stated on a neighborhood of the closure rather
than on `\Omega` itself so that Cauchy's theorem applies on a convex set.

`hWA : W(A) \subseteq \Omega` keeps the resolvent invertible along the whole contour,
as in Chapter 15. It is sufficient, not necessary: the spectrum sits inside the
numerical range and the containment is strict in general, so a matrix can fail this
and still have an invertible resolvent on `\Gamma`.

The admissibility of `\Gamma` — that it is an oriented radial convex boundary — is
what lets the provider quote the convergence result rather than reprove it.

Convexity of `V` is used by the underlying Cauchy theory, not by anything visible in
this card's own proof.

#### Proof roadmap

Quote the convergence theorem for this contour; observe that a convergent sequence's
limit is its `limUnder`.

#### Proof

The provider's proof is four lines, and reading it honestly is the point of this
subsection:

    have hlimit := G.tendsto_simpleSpectrumHolomorphicEval
      R c hVopen hVconvex hclosure hf A hWA
    symm
    simpa only [holomorphicMatrixEval] using hlimit.limUnder_eq

`G.tendsto_simpleSpectrumHolomorphicEval` states that along this contour the
eigenvalue-recipe sequence converges to the boundary integral. It is an indexed card in
its own right — it is CFT-15-006's underlying declaration — so this card's direct
dependency is on Chapter 15's last item, not only on CFT-15-003. Given it, the argument
here is: a sequence that converges to `L` has `limUnder = L`, and `holomorphicMatrixEval`
is by definition that `limUnder`; `symm` orients the equation.

**Provider-proof disclosure.** The displayed statement is a Cauchy-type identity, but
no contour manipulation happens in this proof. The analytic content lives in
`tendsto_simpleSpectrumHolomorphicEval` and the modules beneath it —
`HolomorphicFunctionalCalculus` and `HolomorphicRadialContour` — which this card quotes
and does not restate. What this card contributes is the identification of that limit with
the definition of CFT-16-001.

Worth naming what that content is and is not. The chain rests on an exact Cauchy identity
for a fixed diagonalization, `integral_holomorphic_cauchy_resolvent_of_simpleDiagonalization`,
together with a continuity-and-limit argument. It contains no estimate: the double-layer
bound that gives the constant `2` lives in a different module and feeds a different result,
and is not used here. A reader who wants the Cauchy theory itself should read Chapter 15
and `HolomorphicRadialContour`, not this proof.

That said, the identification is not bookkeeping. It is what makes the value
contour-independent: the right-hand side mentions no contour, so any two admissible
contours give the same integral. The card is usually read for the left-to-right
direction and is most useful read right-to-left.

#### Boundary case

Drop `hWA` and the integrand can fail to exist at a point of `\Gamma` where the
resolvent is singular — though, as CFT-15-002's ledger records, Mathlib's total
matrix inverse returns `0` there, so what is lost is again meaning rather than
definedness. Drop holomorphy and the sequence need not converge, at which point
the right-hand side is the totalization artifact of CFT-16-001 and the equation
is comparing an integral against junk.

#### Pedagogical prerequisites

CFT-15-003, CFT-15-006 — the convergence result the proof quotes directly — and
CFT-16-001.

#### Historical context

The independence of the Riesz–Dunford integral from the contour is normally proved
by a homotopy argument between contours. Routing it through an approximation limit
instead is what lets this development quote a single convergence theorem and get
independence as a corollary rather than as separate work.
Source boundary: Mathlib 4.32.1, and the registered Crouzeix geometry packet.
Review status: registered for content-wave review; the provider-proof disclosure above reviewed against the displayed statement.

#### ML analogy

Mathematical object: two constructions of the same object, one depending on an arbitrary contour and one on an arbitrary perturbation sequence, proved equal.
ML counterpart: two estimators of the same quantity, each with its own nuisance parameter, shown to agree.
Exact transfer: agreement proves both are free of their respective nuisance parameter.
Non-transfer: agreement holds only on the common domain of validity; outside it both are defined and neither is meaningful.
Diagnostic: disagreement between the two routes means a hypothesis failed, not that one implementation is buggy.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.contour_eval_agrees`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_holomorphicMatrixEval`.
Substantive provider: `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_holomorphicMatrixEval`, a four-line proof that quotes `tendsto_simpleSpectrumHolomorphicEval` for this contour and identifies its limit with the definition by `limUnder_eq`. Every analytic step lives in the quoted theorem and its supporting modules, not here.
Readable type map: `R`, `c` and `G` present the contour, `V` is the open convex set carrying holomorphy, `Omega` the region whose boundary is integrated, and `A` the matrix whose numerical range lies in `Omega`.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L11).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [inst_2 : Nonempty.{u_1 + 1} n] (R : CrouzeixConjecture.PositivePeriodicRadialData) (c : Complex) {Omega V : Set.{0} Complex} (G : R.OrientedRadialConvexBoundary c Omega), IsOpen.{0} V → Convex.{0, 0} Real V → LE.le.{0} (closure.{0} Omega) V → ∀ {f : Complex → Complex}, DifferentiableOn.{0, 0, 0} Complex f V → ∀ (A : CrouzeixConjecture.SquareMatrix.{u_1} n), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} A) Omega → Eq.{u_1 + 1} (CrouzeixConjecture.parametricBoundaryIntegral.{0, u_1} (CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundary R c G) CrouzeixConjecture.contourParameterMeasure f A) (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A f)`.
Type SHA-256: `37b0f8603441665ec04b58f40567ce1dbfb89d5960d3be2d374aa456f3910750`.
Direct maintained dependencies: `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_holomorphicMatrixEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:37b0f8603441665ec04b58f40567ce1dbfb89d5960d3be2d374aa456f3910750`.

### CFT-16-003 — the calculus extends polynomial evaluation {#cft-16-003}

#### Purpose

The consistency check without which the notation `f(A)` would be an abuse. If `f`
is the polynomial `z \mapsto z^2`, then `f(A)` had better be `A^2`.

#### Statement

For every square matrix `A` and every polynomial `p`,

    holomorphicMatrixEval A (fun z => Polynomial.eval z p) = polynomialEval p A

where `polynomialEval p A` is `Polynomial.aeval A p`, ordinary matrix polynomial
evaluation.

#### Hypothesis ledger

**None beyond the ambient typeclasses.** No open set, no numerical-range condition,
no differentiability. This is the only card in the chapter's algebraic group that
is unconditional, and the asymmetry against CFT-16-005 and CFT-16-006 is not an
oversight.

The reason is that the hypotheses on those cards exist to secure convergence on a
region, and here convergence is available everywhere for free: a polynomial is
entire, and more to the point `p.aeval` is a continuous function of the matrix on
the whole space. The approximating matrices converge to `A`, so their polynomial
values converge to `A`'s, with no region to restrict to.

A reader tempted to conclude that the hypotheses elsewhere are therefore removable
should note what is being used: not that `p` is holomorphic, but that `B \mapsto p(B)`
is continuous *as a function of the matrix*. Continuity in the matrix argument is not by
itself special to polynomials — CFT-15-002 supplies it for any `f` holomorphic on the
relevant region. What is special is that `Polynomial.continuous_aeval` holds on the whole
matrix space, with no region to stay inside, so there is no neighborhood for a hypothesis
to be about.

#### Proof roadmap

Identify the approximating sequence as polynomial evaluation at the approximating
matrices; note that polynomial evaluation is continuous in the matrix; conclude.

#### Proof

Two steps, both concrete.

*The sequence is what you would guess.* `SimpleDiagonalization.functionEval_polynomial`
says that running the eigenvalue recipe with a polynomial gives ordinary polynomial
evaluation: `hB.functionEval (fun z => p.eval z) = polynomialEval p B`. Its own proof
rewrites with `polynomialEval_eq_innerConjugation_diagonal` — which says polynomial
evaluation of a diagonalizable matrix conjugates the diagonal of polynomial values —
and closes by `rfl`. Applying this at each `k` gives

    simpleSpectrumHolomorphicEval A (fun z => p.eval z) = fun k => polynomialEval p (simpleSpectrumApproximation A k).

*The sequence converges to the right thing.* `p.continuous_aeval` is continuity of
`B \mapsto p(B)`, and `tendsto_simpleSpectrumApproximation` says the approximations
converge to `A`. Composing gives convergence to `polynomialEval p A`, and
`limUnder_eq` turns that into the required identification with `holomorphicMatrixEval`.

The pleasant feature of this proof is that it needs none of the contour machinery.
It is the one place in the chapter where convergence comes from continuity of an
algebraic operation rather than from Cauchy theory.

#### Worked instance

For `p = X^2` the card reads `holomorphicMatrixEval A (fun z => z^2) = A^2`,
including at defective `A`. Take `A = \begin{bmatrix}0&1\\0&0\end{bmatrix}`, which
has no diagonalization at all: the card still gives `A^2 = 0`. Every approximating
matrix `A_k` is diagonalizable with `A_k^2 \to 0`, and the limit is the value the
recipe could not compute directly. This instance is not separately compiled; it is
the general card read at one polynomial.

#### Boundary case

The card says nothing about rational functions. `z \mapsto 1/z` is not a polynomial,
and at a singular `A` the natural candidate `A^{-1}` does not exist while
`holomorphicMatrixEval A (fun z => 1/z)` is still a defined matrix. Nothing here
identifies it, and nothing here should be read as suggesting it is an inverse.

#### Pedagogical prerequisites

CFT-06-002 and CFT-16-001.

#### Historical context

Agreement with the polynomial calculus is the first axiom one writes down for any
functional calculus, and in the operator-theoretic development it is proved from the
integral by a residue computation. Here it is proved from the definition by continuity,
which is shorter and is available without any holomorphy hypothesis — an advantage of
taking the approximation limit as primitive.
Source boundary: Mathlib 4.32.1 polynomial and continuity declarations, and the registered Crouzeix geometry packet.
Review status: registered for content-wave review; the absence of a holomorphy hypothesis reviewed against the providers of CFT-16-005 and CFT-16-006.

#### ML analogy

Mathematical object: a general construction agreeing with an elementary one wherever both apply.
ML counterpart: a learned operator reproducing the closed-form answer on the inputs where a closed form exists.
Exact transfer: agreement on the elementary class is evidence the general construction is the right extension.
Non-transfer: agreement on polynomials says nothing about behaviour on functions with poles, which is a genuinely different regime.
Diagnostic: if a general implementation disagrees with direct matrix powers on a polynomial, the implementation is wrong — this card leaves it no freedom.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.polynomial_compatibility`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.holomorphicMatrixEval_polynomial`.
Substantive provider: `CrouzeixConjecture.holomorphicMatrixEval_polynomial`, a genuine two-step proof: the approximating sequence is identified with polynomial evaluation at the approximating matrices via `SimpleDiagonalization.functionEval_polynomial`, then `Polynomial.continuous_aeval` composed with `tendsto_simpleSpectrumApproximation` supplies convergence.
Readable type map: `p` is the polynomial, `A` the matrix; `polynomialEval p A` is `Polynomial.aeval A p`. No set and no analytic hypothesis appear.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L13).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [inst_2 : Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n) (p : Polynomial.{0} Complex), Eq.{u_1 + 1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A fun z => Polynomial.eval.{0} z p) (CrouzeixConjecture.polynomialEval.{u_1} p A)`.
Type SHA-256: `40b1b85968c33231bba9d1ab05835841a984f09d53e943e881fcae2dd598b8b1`.
Direct maintained dependencies: `CrouzeixConjecture.holomorphicMatrixEval_polynomial`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:40b1b85968c33231bba9d1ab05835841a984f09d53e943e881fcae2dd598b8b1`.

### CFT-16-004 — the calculus is local near the numerical range {#cft-16-004}

#### Purpose

The card that makes `f(A)` depend on `f` only where `A` lives. It is the technical
statement behind every later claim that a function may be modified far away without
consequence.

#### Statement

Let `U` be open with `W(A) \subseteq U`, and let `f` and `g` agree at every point of
`U`. Then `holomorphicMatrixEval A f = holomorphicMatrixEval A g`.

#### Hypothesis ledger

`hUopen` and `hWU` place an open set around the numerical range. `hfg` requires
agreement only on `U`; off `U` the two functions are unrelated.

**No holomorphy, on either function.** `f` and `g` are arbitrary. This is stronger
than the corresponding statement in the classical theory, where locality is usually
derived from the integral representation and therefore inherits its hypotheses.

Openness is used, and used essentially: the proof needs the approximating matrices'
numerical ranges to be *eventually* inside `U`, which is a statement about a
neighborhood. With `U` merely containing `W(A)` — say `U = W(A)` exactly — the
approximations, which are near `A` but not equal to it, need not have their
numerical ranges inside, and the argument has nothing to stand on.

#### Proof roadmap

The recipe reads `f` only at eigenvalues; eigenvalues of the approximations
eventually lie in `U`; so the two sequences are eventually equal, and equal
sequences have equal limits.

#### Proof

The first step is `eventually_simpleSpectrumApproximation_numericalRange_subset_open`:
for large `k`, `W(A_k) \subseteq U`. This is where openness is spent.

On that tail the two sequences are not merely close, they are **equal**. Recall from
CFT-16-001 that `functionEval` builds `diagonal (fun i => f(\lambda_i))` over the
eigenvalues of `A_k`. The provider compares the two diagonal matrices entry by entry:
off the diagonal both are zero; on the diagonal the entries are `f(\lambda_i)` and
`g(\lambda_i)`, and `\lambda_i` lies in `U` by the chain

    eigenvalue \in matrixSpectrum \subseteq numericalRange(A_k) \subseteq U,

the containment being `matrixSpectrum_subset_numericalRange`. So `hfg` applies at
each `\lambda_i` and the diagonals agree. Conjugating equal matrices by the same
basis gives equal matrices.

The last step is the one worth pausing on. Having established
`simpleSpectrumHolomorphicEval A f =^f[atTop] simpleSpectrumHolomorphicEval A g`, the
provider does **not** invoke uniqueness of limits. It unfolds `limUnder` and rewrites
with `Filter.map_congr`: two functions that are eventually equal push the filter
`atTop` forward to the *same filter*, so their `limUnder` values are equal by
construction.

The consequence is that **this card holds even when neither side converges.** If `f`
and `g` agree near `W(A)` and both produce divergent sequences, both sides are
totalization junk — and it is the *same* junk, because the junk is a function of the
pushforward filter, which is the same for both. Locality survives totalization. CFT-16-005 and CFT-16-006 do not, and the reason is
exactly that they close with `tendsto_nhds_unique` and this card does not. CFT-16-003
needs no hypotheses either, but for a different reason again: its sequence always
converges.

#### Boundary case

This proof route needs an open `U`, for the reason in the ledger; whether the statement
survives with `U = W(A)` is not settled here, and no counterexample is available from this
chapter's data, since `simpleSpectrumApproximation` is opaque. Nor can the conclusion be strengthened to say that `f` near `W(A)`
determines `f(A)` in any quantitative sense: the card gives equality under exact
agreement and says nothing about what a small perturbation of `f` on `U` does to
`f(A)`. That is a different statement and is not proved here.

#### Pedagogical prerequisites

CFT-16-001.

#### Historical context

Locality is the property that lets the holomorphic calculus be defined for functions
that are holomorphic only near the spectrum, which is the form the theory takes in
operator theory. In this development it arrives earlier and cheaper than usual, because
the definition reads `f` at finitely many points at each stage rather than integrating
it along a contour.
Source boundary: Mathlib 4.32.1 filter declarations, and the registered Crouzeix geometry packet.
Review status: registered for content-wave review; the filter-equality route and its consequence for divergent inputs reviewed against the provider.

#### ML analogy

Mathematical object: the value depends on the function only through its restriction to a neighborhood of a compact set.
ML counterpart: a model's prediction depending only on the input distribution's support, not on behaviour defined far off-support.
Exact transfer: two specifications agreeing on the support are interchangeable.
Non-transfer: exact agreement is required. This is not a robustness statement and gives no bound for approximate agreement.
Diagnostic: if changing a function far from the numerical range changes `f(A)`, the neighborhood was not actually a neighborhood of `W(A)`.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.locality_on_neighborhood`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.holomorphicMatrixEval_congr_on_neighborhood`.
Substantive provider: `CrouzeixConjecture.holomorphicMatrixEval_congr_on_neighborhood`, a genuine multi-step proof: the approximants' numerical ranges are eventually inside `U`, the two eigenvalue-recipe sequences are then equal entrywise, and `Filter.map_congr` concludes by equality of pushforward filters rather than by uniqueness of limits.
Readable type map: `U` is the open neighborhood of the numerical range, and `f`, `g` the two functions required to agree on it and nowhere else.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L15).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [inst_2 : Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n) {U : Set.{0} Complex}, IsOpen.{0} U → LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} A) U → ∀ {f g : Complex → Complex}, (∀ (z : Complex), Membership.mem.{0, 0} U z → Eq.{1} (f z) (g z)) → Eq.{u_1 + 1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A f) (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A g)`.
Type SHA-256: `1adea35fde8d82277038fa66e3033cc5fc973b77a854fe6edb36b99c4601e5f5`.
Direct maintained dependencies: `CrouzeixConjecture.holomorphicMatrixEval_congr_on_neighborhood`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:1adea35fde8d82277038fa66e3033cc5fc973b77a854fe6edb36b99c4601e5f5`.

### CFT-16-005 — the calculus is additive {#cft-16-005}

#### Purpose

The first half of the homomorphism property. Together with CFT-16-006 it is what
lets later chapters manipulate `f(A)` algebraically instead of unfolding the limit.

#### Statement

Let `U` be open with `W(A) \subseteq U`, and let `f` and `g` be differentiable on `U`.
Then

    holomorphicMatrixEval A (fun z => f z + g z) = holomorphicMatrixEval A f + holomorphicMatrixEval A g.

#### Hypothesis ledger

`hUopen`, `hWU`, and differentiability of **both** `f` and `g` on `U`.

Here is the question the chapter has been building toward: CFT-16-003 and CFT-16-004
needed no holomorphy, so why does additivity?

The answer is about the proof route, and it is worth being careful rather than
emphatic. The argument identifies three limits and applies uniqueness of limits.
Uniqueness of limits requires the sequences to **converge**, and convergence is
exactly what differentiability on a neighborhood of the numerical range buys, through
the contour machinery of Chapter 15. CFT-16-004 escaped because filter equality needs
no convergence; CFT-16-003 escaped because continuity of `B \mapsto p(B)` supplies
convergence directly. Neither escape is available for a general pair `f`, `g`.

What is **not** claimed is that the statement is false without holomorphy. Nothing in
this chapter exhibits a non-holomorphic `f` and `g` for which additivity fails. What
can be said is weaker and sharper: without a convergence hypothesis all three
expressions may be totalization artifacts of CFT-16-001, and no card here asserts
anything about how those artifacts relate. The hypothesis is what this proof uses;
its necessity is not settled here.

Note also that differentiability is required of `f` and `g` separately, on a *common*
`U`. A `U` for `f` and a different `V` for `g` would need to be intersected first, and
the intersection must still be an open neighborhood of `W(A)`. That is automatic when
each of `U` and `V` was already a neighborhood of `W(A)`, and it is a genuine assumption
when they are merely the regions on which the two functions happen to be differentiable —
which is the situation CFT-16-E05 works in.

#### Proof roadmap

Get convergence for `f`, `g` and `f + g` from the neighborhood theorem; observe the
sequence for `f + g` is the sum of the sequences; apply uniqueness of limits.

#### Proof

The engine is `tendsto_simpleSpectrumHolomorphicEval_of_differentiableOn_neighborhood`,
invoked three times — at `f`, at `g`, and at `hf.add hg`. It states that when a function
is differentiable on an open neighborhood of `W(A)`, the eigenvalue-recipe sequence
converges to `holomorphicMatrixEval A` of it. No card's public declaration *is* this theorem — CFT-16-005 and CFT-16-006 both name it
as their substantive provider — and it is where the chapter's real weight sits: its own proof takes the
abstract `U`, thickens the compact convex `W(A)` by an `\epsilon` small enough to stay
inside `U`, chooses `N` with the outer approximation radius below `\epsilon`, and
invokes `canonicalParallelOrientedRadialBoundaryStatement` to produce a concrete
admissible contour — at which point CFT-16-002's convergence result applies. The
passage from "some open neighborhood" to "a specific contour" is that construction.

The pointwise step is `SimpleDiagonalization.functionEval_add`: the eigenvalue recipe
is additive at each fixed `k`, because it builds `diagonal (fun i => f(\lambda_i) + g(\lambda_i))`,
and the diagonal of a sum is the sum of the diagonals, conjugation being linear. So

    simpleSpectrumHolomorphicEval A (f + g) = fun k => simpleSpectrumHolomorphicEval A f k + simpleSpectrumHolomorphicEval A g k.

Now two facts about the same sequence: it converges to `holomorphicMatrixEval A (f+g)`
by the engine, and it converges to the sum of the two limits because limits add. Both
limits therefore agree, by `tendsto_nhds_unique`. That final appeal is what the
hypotheses were for.

#### Boundary case

The common `U` cannot be dropped to "each is differentiable somewhere", and the
neighborhood cannot shrink to `W(A)` itself, for the same reason as in CFT-16-004:
the approximating matrices are near `A`, not equal to it, and the engine needs their
numerical ranges to fit inside the region eventually.

#### Pedagogical prerequisites

CFT-15-002 and CFT-16-001.

#### Historical context

Linearity of the holomorphic calculus is immediate from the integral representation,
where it is inherited from linearity of the integral. Proving it from an approximation
definition costs a convergence argument instead, which is the trade this development
accepts in exchange for a definition that needs no hypotheses to state.
Source boundary: Mathlib 4.32.1 topology declarations, and the registered Crouzeix geometry packet.
Review status: registered for content-wave review; the necessity-versus-use distinction in the ledger reviewed against the provider.

#### ML analogy

Mathematical object: a map that is additive on a restricted domain.
ML counterpart: a transform that is linear on the region where its defining approximation converges.
Exact transfer: on the good region, decomposing an input and recombining outputs is safe.
Non-transfer: the additivity is not asserted off the region; an implementation that is total will still return values there.
Diagnostic: additivity failing numerically is evidence the numerical range left the neighborhood, not that the theorem is wrong.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.functional_calculus_additive`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.holomorphicMatrixEval_add`.
Substantive provider: `CrouzeixConjecture.holomorphicMatrixEval_add`, a genuine multi-step proof: three invocations of `tendsto_simpleSpectrumHolomorphicEval_of_differentiableOn_neighborhood`, the pointwise additivity `SimpleDiagonalization.functionEval_add`, and `tendsto_nhds_unique`.
Readable type map: `U` is the common open neighborhood, `f` and `g` the two functions differentiable on it.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L17).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [inst_2 : Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n) {U : Set.{0} Complex}, IsOpen.{0} U → LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} A) U → ∀ {f g : Complex → Complex}, DifferentiableOn.{0, 0, 0} Complex f U → DifferentiableOn.{0, 0, 0} Complex g U → Eq.{u_1 + 1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A fun z => HAdd.hAdd.{0, 0, 0} (f z) (g z)) (HAdd.hAdd.{u_1, u_1, u_1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A f) (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A g))`.
Type SHA-256: `ef16b5d5c7fdc111416f8e8b37f0a93a985e463a53d649af716188935f2dc938`.
Direct maintained dependencies: `CrouzeixConjecture.holomorphicMatrixEval_add`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:ef16b5d5c7fdc111416f8e8b37f0a93a985e463a53d649af716188935f2dc938`.

### CFT-16-006 — the calculus is multiplicative {#cft-16-006}

#### Purpose

The second half of the homomorphism property, and the one Crouzeix's problem needs:
it is what lets a bound on a product of functions be reduced to the factors.

#### Statement

Under the same hypotheses as CFT-16-005,

    holomorphicMatrixEval A (fun z => f z * g z) = holomorphicMatrixEval A f * holomorphicMatrixEval A g,

with the factor order on the right as displayed.

#### Hypothesis ledger

Identical to CFT-16-005, and for the identical reason. The ledger there applies
verbatim, including the caution about necessity.

One addition specific to this card. The right-hand side is a product of **matrices**,
which do not commute in general, so the statement has to fix an order, and the
provider's docstring flags that it does.

The fixed order is a feature of the formal statement, not a restriction on the
mathematics. Multiplication of complex numbers is commutative, so `fun z => f z * g z`
and `fun z => g z * f z` are the *same function*; applying this card to each orientation
gives `f(A)g(A) = g(A)f(A)` under exactly these hypotheses. So the card does license
reordering wherever it applies. What is true is the narrower statement that no
*declaration* in this chapter is the commutation identity — it is a one-step corollary,
not a compiled card.

#### Proof roadmap

Identical in shape to CFT-16-005, with multiplication in place of addition.

#### Proof

The same three invocations of
`tendsto_simpleSpectrumHolomorphicEval_of_differentiableOn_neighborhood`, at `f`, `g`
and `hf.mul hg`, and the same closing `tendsto_nhds_unique`.

The pointwise step is the one that differs. `SimpleDiagonalization.functionEval_mul`
must show

    innerConjugation X (diagonal (f \cdot g)) = innerConjugation X (diagonal f) * innerConjugation X (diagonal g),

and the reason this holds is that the two conjugations share the basis `X`: the inner
`X^{-1}X` cancels, leaving the product of two diagonal matrices, which is the diagonal
of the pointwise product. The provider does this by `map_mul` for the conjugation
followed by an entrywise check splitting on whether the indices agree. The shared basis
is essential — it is why the recipe is multiplicative at each fixed `k` even though
matrix multiplication is not commutative, and it is available only because both
functions are evaluated through the *same* diagonalization of the *same* approximating
matrix.

#### Boundary case

As in CFT-16-005, and with the additional caution that nothing here extends to
functions with poles. The product `f \cdot g` must itself be differentiable on `U`,
which it is, but the card gives no route to `1/f` even where `f` is nonvanishing on
`U`; that is the rational calculus, and it is Chapter 17's subject — developed there from the
contour integral rather than from this chapter's laws.

#### Pedagogical prerequisites

CFT-16-001 and CFT-16-005.

#### Historical context

Multiplicativity is the deepest of the calculus axioms and in the integral development
it requires the resolvent identity and a two-contour argument. The approximation route
replaces that with the observation that a single diagonalization handles both factors
at once, which is a genuine simplification available only in finite dimensions.
Source boundary: Mathlib 4.32.1 algebra declarations, and the registered Crouzeix geometry packet.
Review status: registered for content-wave review; the factor-order caution reviewed against the provider docstring.

#### ML analogy

Mathematical object: a homomorphism from an algebra of scalar functions to an algebra of matrices.
ML counterpart: a representation under which composition of scalar filters corresponds to composition of operators.
Exact transfer: on the good region, a product specification may be implemented as a product of implementations.
Non-transfer: the target algebra is noncommutative in general, so an order must be fixed; here the two orders agree, but that is a corollary of this card rather than a property of the target algebra.
Diagnostic: an implementation that silently reorders factors will agree on these cards' instances only when the factors happen to commute.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.functional_calculus_multiplicative`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.holomorphicMatrixEval_mul`.
Substantive provider: `CrouzeixConjecture.holomorphicMatrixEval_mul`, a genuine multi-step proof of the same shape as CFT-16-005, with `SimpleDiagonalization.functionEval_mul` — whose content is that a single shared diagonalization handles both factors — in place of the additive step.
Readable type map: `U` is the common open neighborhood, `f` and `g` the two functions differentiable on it; the conclusion's factor order is the one shown.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L19).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [inst_2 : Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n) {U : Set.{0} Complex}, IsOpen.{0} U → LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} A) U → ∀ {f g : Complex → Complex}, DifferentiableOn.{0, 0, 0} Complex f U → DifferentiableOn.{0, 0, 0} Complex g U → Eq.{u_1 + 1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A fun z => HMul.hMul.{0, 0, 0} (f z) (g z)) (HMul.hMul.{u_1, u_1, u_1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A f) (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A g))`.
Type SHA-256: `25e1b1b73070577dda1993394216345d2c368966717bf75147dd6676b609ee70`.
Direct maintained dependencies: `CrouzeixConjecture.holomorphicMatrixEval_mul`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:25e1b1b73070577dda1993394216345d2c368966717bf75147dd6676b609ee70`.

## Worked examples

**Example 1 — the calculus at a matrix with no diagonalization.** Let
`A = \begin{bmatrix}0&1\\0&0\end{bmatrix}` and `f(z) = z^2`. The eigenvalue recipe
cannot be run: `A` has the single eigenvalue `0` with a one-dimensional eigenspace,
so there is no basis of eigenvectors. CFT-16-003 nevertheless gives
`holomorphicMatrixEval A f = A^2 = 0`. Every approximating `A_k` is diagonalizable,
each `A_k^2` is computed by the recipe, and the limit fills in the value the recipe
could not reach. This is the whole point of the construction in one line.

**Example 2 — locality is not continuity.** Fix `A` with `W(A)` inside the unit disk
`U`. Let `f(z) = z` and let `g` agree with `f` on `U` and equal the indicator of the
Gaussian rationals `\mathbb{Q} + i\mathbb{Q}` outside it. CFT-16-004 gives
`g(A) = f(A) = A` exactly, even though `g` is nowhere continuous off `U`. No hypothesis
of CFT-16-004 is violated, because it asks nothing of either function off `U`.

The Gaussian rationals rather than the rationals, because `\mathbb{Q}` sits inside
`\mathbb{R}` and is not dense in `\mathbb{C}`: its indicator vanishes on a neighborhood
of every non-real point and is therefore holomorphic off the real line, which would make
it a poor witness for "badly behaved".

Contrast this with CFT-16-005: to add `g` to something we would first have to know
`g` is differentiable on a neighborhood of `W(A)`, and it is — on `U` it agrees with
`z`. So `g` is usable in the algebraic cards too, provided every hypothesis is read
as being about `U` and not about `\mathbb{C}`.

**Example 3 — where the chapter says nothing.** Take `f` the indicator of the Gaussian
rationals on all of `\mathbb{C}`, and any `A`. Then `holomorphicMatrixEval A f` is a
defined matrix and no card in this chapter says anything about it.

It is tempting to add that the value is junk, and that would be overreaching in the same
breath as warning against overreach. Whether the defining sequence converges depends on
whether the eigenvalues of the chosen approximants happen to be Gaussian rationals, and
`simpleSpectrumApproximation` is a `Classical.choose` the development never opens. If no
such eigenvalue occurs, every term is zero and the limit exists. The correct statement is
the modest one: writing the value down is legal, and this chapter determines nothing about
it either way.

## ML bridge

The chapter is an extended case study in the difference between a function being
*defined* and a function being *specified*.

`holomorphicMatrixEval` is total. It type-checks on every input, returns a matrix on
every input, and raises nothing on the inputs where its defining limit does not exist.
An implementation with this signature would pass every smoke test and would be wrong
on a set of inputs that no test distinguishes. The specification lives entirely in the
theorems, and the theorems carry hypotheses: differentiability on an open neighborhood
of the numerical range, for the algebraic laws; no analytic hypothesis for locality,
though it still needs `U` open with `W(A) \subseteq U`; and nothing at all beyond the
ambient typeclasses for the polynomial case.

The pattern recurs whenever a construction is totalized for convenience — a division
that returns zero at zero, a normalization that returns the input when the norm
vanishes, an argmax on an empty set. The convenience is real: totality keeps the type
simple and keeps hypotheses out of the definition, where they would have to be carried
by every downstream statement. The cost is that the type no longer records the domain
of validity, and the only place that domain survives is in the hypotheses of the
theorems.

The transferable discipline is the one this chapter's cards follow: for a totalized
construction, track which results carry a convergence hypothesis and which do not, and
never read a result proved on the good region as a description of the value off it.
CFT-16-004 is the instructive case, because it is proved by a route that does not need
convergence and therefore says something genuine on the bad region too — that two
specifications agreeing near the numerical range produce the same output, valid or not.
That is a rarer and stronger guarantee than the algebraic cards give, and it comes from
the proof technique rather than from the statement.

## Lean translation

`CrouzeixTextbook.Part03.Chapter16` re-exports six declarations from
`CrouzeixConjecture.HolomorphicOuterLimit`: the definition
`holomorphic_matrix_eval`, the contour agreement `contour_eval_agrees`, the polynomial
compatibility `polynomial_compatibility`, the locality statement
`locality_on_neighborhood`, and the two algebraic laws
`functional_calculus_additive` and `functional_calculus_multiplicative`.

The structural fact a reader should take from the Lean is the ordering of definition
and theorem. `holomorphicMatrixEval` is defined as `limUnder atTop` of a sequence built
from diagonalizations of perturbed matrices — so the formal development *does* define
`f(A)` through eigenvectors, on the approximants, and the contour integral is a theorem
about the result rather than its definition. Reading the chapter title as announcing an
integral definition gets the architecture backwards.

The chapter's analytic weight is carried by a declaration that is no card's public
declaration, though CFT-16-005 and CFT-16-006 name it as their provider,
`tendsto_simpleSpectrumHolomorphicEval_of_differentiableOn_neighborhood`, which converts
an abstract open neighborhood of the numerical range into a concrete admissible contour
and is what CFT-16-005 and CFT-16-006 actually stand on.

## Exercises with complete solutions

### CFT-16-E01 — the definition, unfolded {#exercise-cft-16-e01}

State what `holomorphicMatrixEval A f` is, by definition, for an arbitrary `f`,
and say what hypotheses the definition requires.

#### Complete written solution

It is `limUnder atTop (simpleSpectrumHolomorphicEval A f)`: the totalized limit,
along the simple-spectrum approximations of `A`, of the eigenvalue recipe applied
to `f`.

The definition requires nothing beyond the ambient typeclasses. `f` is an arbitrary
function of a complex variable and `A` an arbitrary square matrix: no holomorphy, no
continuity, no condition relating `f` to the numerical range. The one ambient assumption
that is not bookkeeping is `[Nonempty n]`, which the approximating sequence's existence
theorem needs.

The exercise is worth stating because the answer is the opposite of what the
chapter title suggests. The formal development does not define `f(A)` by a Cauchy
integral; it defines `f(A)` by diagonalizing perturbed matrices, and the integral
is a theorem about the result. The statement holds by `rfl` — it is a definitional
unfolding and nothing more — and that is precisely the content being checked: that
the definition really is the limit and not something else.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter16.exercise_01_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L29).
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [inst_2 : Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n) (f : Complex → Complex), Eq.{u_1 + 1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A f) (Filter.limUnder.{u_1, 0} Filter.atTop.{0} (CrouzeixConjecture.simpleSpectrumHolomorphicEval.{u_1} A f))`.
Type SHA-256: `948547b6d9e71d06745aef600230ced14166651197532babcec5c1609d1ab9a4`.
Direct maintained dependencies: `CrouzeixConjecture.SquareMatrix`, `CrouzeixConjecture.holomorphicMatrixEval`, `CrouzeixConjecture.simpleSpectrumHolomorphicEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:948547b6d9e71d06745aef600230ced14166651197532babcec5c1609d1ab9a4`.

### CFT-16-E02 — the identity function {#exercise-cft-16-e02}

Compute `holomorphicMatrixEval A (fun z => z)` for an arbitrary square matrix `A`.

#### Complete written solution

The answer is `A`.

Apply CFT-16-003 at the polynomial `p = X`. That card gives
`holomorphicMatrixEval A (fun z => Polynomial.eval z X) = polynomialEval X A`, and
`polynomialEval` is `Polynomial.aeval`, so the right-hand side is `A`; the
left-hand function is `fun z => z`.

What deserves attention is the hypothesis list: there is none. No open set, no
numerical-range condition, no differentiability. The identity function is a polynomial,
and CFT-16-003 is unconditional, so this holds for every matrix including those with no
diagonalization at all.

**Provider-proof disclosure.** The checked solution is a single instantiation of
CFT-16-003 at `p = X`, closed by simplification; it contributes no argument of its own.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter16.exercise_02_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L34).
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [inst_2 : Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n), Eq.{u_1 + 1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A fun z => z) A`.
Type SHA-256: `0282d0805d73680dea5da3fa6738a51693ec184a61224008c1daa9a9de812ffe`.
Direct maintained dependencies: `CrouzeixConjecture.SquareMatrix`, `CrouzeixConjecture.holomorphicMatrixEval`, `CrouzeixConjecture.holomorphicMatrixEval_polynomial`, `CrouzeixConjecture.polynomialEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:0282d0805d73680dea5da3fa6738a51693ec184a61224008c1daa9a9de812ffe`.

### CFT-16-E03 — contour independence {#exercise-cft-16-e03}

Prove that any two admissible oriented radial contours compute the same value.

#### Complete written solution

This is CFT-16-002 read from right to left, which is how that card is most useful.

Let `\Gamma_1` and `\Gamma_2` be oriented radial convex boundaries for regions
`\Omega_1` and `\Omega_2`, each with its own open convex `V_j` containing
`\overline{\Omega_j}`, with `f` differentiable on both and `W(A)` inside both
regions. CFT-16-002 applied to each gives

    integral over \Gamma_1 = holomorphicMatrixEval A f = integral over \Gamma_2,

and the two integrals are therefore equal.

The proof is two rewrites and no analysis, but the conclusion is not trivial: it
is the statement that the Riesz–Dunford integral does not depend on the contour.
It comes out this cheaply only because the right-hand side of CFT-16-002 mentions
no contour at all. In the classical development the same fact costs a homotopy
argument between the two contours.

Note that the two contours need not be comparable in any geometric sense — no
containment, no homotopy, no shared region is assumed. All that is required is that each
is admissible for the same `A` and the same `f`.

**Provider-proof disclosure.** The packet already carries this statement, hypothesis for
hypothesis, as
`CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_of_two_orientedRadialBoundaries`,
proved directly by `tendsto_nhds_unique`. The exercise is a re-derivation rather than new
content, and it is set because the route differs: two rewrites through CFT-16-002 exhibit
the right-to-left reading of that card, which is the point being taught.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter16.exercise_03_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L40).
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [Nonempty.{u_1 + 1} n] (R₁ R₂ : CrouzeixConjecture.PositivePeriodicRadialData) (c₁ c₂ : Complex) {Ω₁ Ω₂ V₁ V₂ : Set.{0} Complex} (G₁ : R₁.OrientedRadialConvexBoundary c₁ Ω₁) (G₂ : R₂.OrientedRadialConvexBoundary c₂ Ω₂), IsOpen.{0} V₁ → Convex.{0, 0} Real V₁ → LE.le.{0} (closure.{0} Ω₁) V₁ → IsOpen.{0} V₂ → Convex.{0, 0} Real V₂ → LE.le.{0} (closure.{0} Ω₂) V₂ → ∀ {f : Complex → Complex}, DifferentiableOn.{0, 0, 0} Complex f V₁ → DifferentiableOn.{0, 0, 0} Complex f V₂ → ∀ (A : CrouzeixConjecture.SquareMatrix.{u_1} n), LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} A) Ω₁ → LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} A) Ω₂ → Eq.{u_1 + 1} (CrouzeixConjecture.parametricBoundaryIntegral.{0, u_1} (CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundary R₁ c₁ G₁) CrouzeixConjecture.contourParameterMeasure f A) (CrouzeixConjecture.parametricBoundaryIntegral.{0, u_1} (CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundary R₂ c₂ G₂) CrouzeixConjecture.contourParameterMeasure f A)`.
Type SHA-256: `fb3142adbf592bfce648a49b04de3760e47c602e62ee787fe508fed14bfca4e4`.
Direct maintained dependencies: `CrouzeixConjecture.ContourParameter`, `CrouzeixConjecture.PositivePeriodicRadialData`, `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary`, `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundary`, `CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.parametricBoundaryIntegral_eq_holomorphicMatrixEval`, `CrouzeixConjecture.SquareMatrix`, `CrouzeixConjecture.contourParameterMeasure`, `CrouzeixConjecture.holomorphicMatrixEval`, `CrouzeixConjecture.numericalRange`, `CrouzeixConjecture.parametricBoundaryIntegral`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:fb3142adbf592bfce648a49b04de3760e47c602e62ee787fe508fed14bfca4e4`.

### CFT-16-E04 — replacement outside the neighborhood {#exercise-cft-16-e04}

Show that `f` may be replaced, outside an open neighborhood of `W(A)`, by an
arbitrary constant without changing `f(A)`, and note which hypotheses of the
chapter this does *not* require.

#### Complete written solution

Let `U` be open with `W(A) \subseteq U`, and define `g(z) = f(z)` for `z \in U`
and `g(z) = c` otherwise. Then `f` and `g` agree at every point of `U`, so
CFT-16-004 gives `f(A) = g(A)`.

The hypotheses this does not require are the point. Neither `f` nor `g` is assumed
holomorphic, continuous, or measurable — anywhere, including on `U`. The modified
function is in general wildly discontinuous across the boundary of `U`, and
CFT-16-004 does not care, because it asks nothing of either function off `U` and
reads each of them only at finitely many eigenvalues on `U`.

A second thing this does not require is convergence. CFT-16-004 is proved by
filter equality rather than by uniqueness of limits, so the conclusion holds even
when both sides are totalization artifacts. If `f` is chosen so that the defining
sequence diverges, `f(A)` and `g(A)` are both meaningless — and still equal.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter16.exercise_04_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L57).
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [inst_2 : Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n) {U : Set.{0} Complex}, IsOpen.{0} U → LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} A) U → ∀ (f : Complex → Complex) (c : Complex), Eq.{u_1 + 1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A f) (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A fun z => ite.{1} (Membership.mem.{0, 0} U z) (f z) c)`.
Type SHA-256: `270bbdba76559035aeb765f2a372eb8804874031cd1cb11af00359e17e3dbeb8`.
Direct maintained dependencies: `CrouzeixConjecture.SquareMatrix`, `CrouzeixConjecture.holomorphicMatrixEval`, `CrouzeixConjecture.holomorphicMatrixEval_congr_on_neighborhood`, `CrouzeixConjecture.numericalRange`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:270bbdba76559035aeb765f2a372eb8804874031cd1cb11af00359e17e3dbeb8`.

### CFT-16-E05 — two neighborhoods, intersected {#exercise-cft-16-e05}

Additivity requires `f` and `g` holomorphic on a *common* open neighborhood of
`W(A)`. Suppose instead `f` is differentiable on an open `U` and `g` on an open
`V`. State and prove the additivity law available in that situation.

#### Complete written solution

The law holds provided `W(A) \subseteq U \cap V`.

The intersection of two open sets is open, so `U \cap V` is an open set, and the
hypothesis places `W(A)` inside it. Restricting differentiability along the
inclusions `U \cap V \subseteq U` and `U \cap V \subseteq V` makes `f` and `g`
both differentiable on `U \cap V`, and CFT-16-005 applies there.

The hypothesis doing the work is `W(A) \subseteq U \cap V`, and it is a real
condition rather than bookkeeping. Two regions can each be a perfectly good
neighborhood of part of the numerical range while their intersection misses it
entirely, or fails to contain it; in that case no version of the law is available
by this route, because there is no common region on which to run the argument.

This is the boundary the card's phrase "a common open neighborhood" is pointing at, and
the exercise is the reason to state it that way rather than as "each is holomorphic
somewhere".

**Provider-proof disclosure.** The checked solution is a single term-mode application of
CFT-16-005, supplying `IsOpen.inter` and two restrictions along the inclusions. The
mathematical step — that intersecting the two regions is legitimate exactly when the
intersection still surrounds the numerical range — is in the statement rather than in the
proof.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter16.exercise_05_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L66).
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [inst_2 : Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n) {U V : Set.{0} Complex}, IsOpen.{0} U → IsOpen.{0} V → LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} A) (Inter.inter.{0} U V) → ∀ {f g : Complex → Complex}, DifferentiableOn.{0, 0, 0} Complex f U → DifferentiableOn.{0, 0, 0} Complex g V → Eq.{u_1 + 1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A fun z => HAdd.hAdd.{0, 0, 0} (f z) (g z)) (HAdd.hAdd.{u_1, u_1, u_1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A f) (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A g))`.
Type SHA-256: `f04b6e686d38cd3abf759b72bd1825cc1a0f852515c04d88e91a33f08bbe9be1`.
Direct maintained dependencies: `CrouzeixConjecture.SquareMatrix`, `CrouzeixConjecture.holomorphicMatrixEval`, `CrouzeixConjecture.holomorphicMatrixEval_add`, `CrouzeixConjecture.numericalRange`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:f04b6e686d38cd3abf759b72bd1825cc1a0f852515c04d88e91a33f08bbe9be1`.

### CFT-16-E06 — the square law {#exercise-cft-16-e06}

Derive `(f^2)(A) = f(A)^2` from the multiplicative card.

#### Complete written solution

Instantiate CFT-16-006 at `g := f`. Both differentiability hypotheses are then the
same hypothesis, and the conclusion reads
`holomorphicMatrixEval A (fun z => f z * f z) = holomorphicMatrixEval A f * holomorphicMatrixEval A f`.

**Provider-proof disclosure.** The checked solution is a single instantiation of
CFT-16-006 and contributes no argument of its own.

It is still worth doing, for what it shows about the factor order. The right-hand
side is a product of two matrices, and in general matrix multiplication does not
commute, so a statement of this shape has to fix an order. Here the two factors
happen to be equal, so the order is invisible — which is exactly why this instance
is a poor guide to the general card. Nothing in this chapter establishes that
`f(A)` and `g(A)` commute for distinct `f` and `g`, and the square law should not
be read as evidence that they do.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter16.exercise_06_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter16.lean#L76).
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [inst_2 : Nonempty.{u_1 + 1} n] (A : CrouzeixConjecture.SquareMatrix.{u_1} n) {U : Set.{0} Complex}, IsOpen.{0} U → LE.le.{0} (CrouzeixConjecture.numericalRange.{u_1} A) U → ∀ {f : Complex → Complex}, DifferentiableOn.{0, 0, 0} Complex f U → Eq.{u_1 + 1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A fun z => HMul.hMul.{0, 0, 0} (f z) (f z)) (HMul.hMul.{u_1, u_1, u_1} (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A f) (CrouzeixConjecture.holomorphicMatrixEval.{u_1} A f))`.
Type SHA-256: `ff5f460028cae4619402e6d56bcfe25585979bf6f4d029150c043f751c4f299d`.
Direct maintained dependencies: `CrouzeixConjecture.SquareMatrix`, `CrouzeixConjecture.holomorphicMatrixEval`, `CrouzeixConjecture.holomorphicMatrixEval_mul`, `CrouzeixConjecture.numericalRange`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:ff5f460028cae4619402e6d56bcfe25585979bf6f4d029150c043f751c4f299d`.

## Synthesis and forward dependencies

Chapter 16 delivers a functional calculus defined for every matrix, agreeing with
polynomial evaluation unconditionally, local near the numerical range unconditionally,
and additive and multiplicative on any open neighborhood of the numerical range where
the functions are holomorphic.

The dependency to keep in view is the one that gives the chapter its title. The two
algebraic cards are proved by uniqueness of limits, and uniqueness of limits is worth
nothing without convergence. Convergence comes from Chapter 15's contour theory by way
of a neighborhood-to-contour construction. Remove Cauchy theory and the definition survives, as do the polynomial compatibility
and the locality statement, neither of which touches the contour machinery; what becomes
unprovable by this route is the additive and multiplicative pair.

The forward edges are narrower than the construction suggests. Of the six cards, two
are cited outside this chapter: CFT-16-001 by CFT-17-006 and CFT-32-005, and CFT-16-003
by CFT-28-001, CFT-28-003, CFT-29-002 and CFT-32-006. The locality statement and the two
algebraic laws are not cited by any later card — they are here because they are what
makes the definition a calculus, not because a later chapter names them.

CFT-16-003 is the load-bearing one. CFT-32-006 is `polynomial_from_holomorphic`, the step
taking a holomorphic Crouzeix bound to the polynomial statement, and polynomial
compatibility is what licenses it. Chapter 17 builds the rational calculus on CFT-16-001,
though by its own contour-first route rather than by consuming this chapter's laws.
