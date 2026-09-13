---
id: cft-chapter-17-functions-of-matrices-and-operators
title: Functions of matrices and operators
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-12
tags: [crouzeix-textbook, analysis-and-complex-functions, mathematics, lean]
confidence: high
canonical: 17_functions_of_matrices_and_operators.md
chapter: 17
part: 3
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 17: Functions of matrices and operators

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iii-analysis-and-complex-functions|Part III — Analysis and complex functions]]
Previous: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/16_consequences_of_cauchy_theory|Chapter 16 — Consequences of Cauchy theory]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/18_positive_real_analytic_functions|Chapter 18 — Positive-real analytic functions]]

## Opening problem

Chapter 16 defined `f(A)` for every matrix `A` and every function `f`, and proved the
value meaningful when `f` is holomorphic near the numerical range. The price of that
generality is that nothing about `f(A)` can be *computed* from the definition: it is a
limit of eigenvalue-recipe values along a sequence the development never writes down.

For a rational function `r = p/q` there is an obvious algebraic candidate that needs no
limit at all: `p(A)\,q(A)^{-1}`. This is an exact finite-dimensional algebraic formula,
not a prescription to form an explicit inverse numerically. Two questions stand between it and the
name `r(A)`. When is `q(A)` invertible? And does the algebraic value agree with the
analytic one from Chapter 16 wherever both make sense?

This chapter answers the first with a predicate — `r` has no pole on `W(A)` — and a
theorem that the predicate makes the denominator a unit. The second it answers by
narrating a maintained theorem, not indexed as a card, that identifies the two calculi
on pole-free rational functions and is proved from Chapter 16's laws. That theorem is
where Chapter 16's polynomial compatibility, locality and multiplicativity are actually
consumed; the additive law is spent elsewhere in the packet, not here.

## What this chapter compiles, and what it does not

The title promises functions of matrices and operators. A reader expecting the Jordan
block calculus — `f(J) = f(\lambda)I + f'(\lambda)N` — the resolvent identity, the
spectral mapping theorem, or anything about operators on infinite-dimensional spaces
will find none of it. Nothing here concerns derivatives at defective eigenvalues, and
every declaration is about finite square matrices.

What is compiled is the rational functional calculus, in six pieces: the pole set of a
rational function, its finiteness, the openness of its complement, the pole-freeness
predicate, scalar evaluation, and matrix evaluation. The theorem that makes the matrix
evaluation meaningful, and the theorem that identifies it with Chapter 16's calculus,
are maintained declarations the chapter narrates rather than indexes.

## Conceptual model

Three objects are in play and the chapter's care is in keeping them distinct.

*The rational function.* A `RatFunc \mathbb{C}` in Mathlib's sense: a formal fraction
carrying a canonical reduced presentation, `r.num / r.denom`, with the denominator monic
and the pair coprime (`RatFunc.monic_denom`, `RatFunc.isCoprime_num_denom`,
`RatFunc.num_div_denom`). Every rational function has exactly one such presentation. The
consequence that matters is that a factor common to numerator and denominator in some
*other* presentation has already been cancelled: the reduced denominator does not see it.

*Its poles.* The zeros of the reduced denominator. Because the denominator is reduced, a
removable singularity is not a pole; because the denominator is a nonzero polynomial,
there are finitely many; because finite sets are closed, the complement is open. That
open complement is the domain on which `r` is holomorphic and, as CFT-17-003's card
explains, it is exactly the open neighborhood Chapter 16's cards ask for.

*Evaluation.* Scalar evaluation is `\mathrm{num}(z) / \mathrm{denom}(z)`; matrix
evaluation is `\mathrm{num}(A) \cdot \mathrm{denom}(A)^{-1}`. Both are **total**, for
the same reason Chapter 16's calculus was total: Mathlib's division returns `0` at a
zero divisor and Mathlib's matrix inverse returns `0` at a singular matrix. So both
evaluations are defined everywhere and meaningful only somewhere, and the predicate
"pole-free on `s`" is the hypothesis that turns the one into the other.

The parallel with Chapter 16 is worth holding onto. There, the definition was total and
meaning came from *convergence*, supplied by holomorphy on a neighborhood. Here, the
definition is total and meaning comes from *invertibility*, supplied by pole-freeness on
the numerical range. In both chapters the hypotheses live on theorems, never on the
definitions. Here the value is the zero matrix when the denominator matrix is singular.
Failure of pole-freeness on the numerical range alone does not imply singularity:
the denominator can remain nonzero on the spectrum.

## Running example: the resolvent

Take `r(z) = 1/(z - \lambda)`. Its reduced presentation has numerator `1` and
denominator `z - \lambda`: the denominator is monic, the pair is coprime, and the
reduced presentation is unique, so this is Mathlib's `r.num` and `r.denom`. (The
uniqueness is elementary — two coprime presentations with monic denominators have
denominators dividing each other — but it is not compiled here; the example is read
off the characterization, not checked.)

The pole set is `\{\lambda\}`. Pole-freeness on `W(A)` says `\lambda \notin W(A)`. The
matrix evaluation is `1 \cdot (A - \lambda I)^{-1}`, which is the resolvent. And the
engine theorem behind CFT-17-006 says: if `\lambda` is outside the numerical range, then
`A - \lambda I` is invertible — true because the spectrum lies inside the numerical
range. So the rational calculus at this one function is the resolvent, and the
pole-free condition on the numerical range is sufficient for the resolvent to exist.
The exact condition is that `\lambda` lies outside the spectrum.

## Formal development

### CFT-17-001 — the pole set of a rational function {#cft-17-001}

#### Purpose

The set on which everything else is conditioned. Every hypothesis in this chapter is a
statement about where this set is not.

#### Statement

For `r : RatFunc \mathbb{C}`,

    rationalPoleSet r = {z | Polynomial.eval z r.denom = 0},

the zero set of the canonical reduced denominator.

#### Hypothesis ledger

None. A definition, taking any rational function.

The content is in the word *reduced*. `r.denom` is Mathlib's canonical denominator —
monic, coprime to the numerator, and uniquely determined by `r` — so the pole set
depends only on the rational function and not on how it happened to be written. The
provider's docstring makes the consequence explicit: removable singularities from an
unreduced presentation are not counted.

#### Proof roadmap

A definition has no proof. What needs unfolding is what `r.denom` is, which is Mathlib's
business, characterized by `RatFunc.num_div_denom`, `RatFunc.monic_denom` and
`RatFunc.isCoprime_num_denom`.

#### Proof

There is nothing to prove; CFT-17-E01 records that the definition unfolds to the
displayed set by `rfl`.

The choice worth remarking on is that poles are defined through the denominator's
zeros rather than through the function's behaviour — through algebra, not through
`|r(z)| \to \infty`. The two agree for reduced fractions, but only the algebraic version
is a decidable-looking predicate on a polynomial, and only it supports the finiteness
argument of CFT-17-002 in one line.

#### Boundary case

The presentation `z / z` names the rational function `1`. Its reduced denominator is
`1`, whose zero set is empty: the pole set of `z/z` is `\varnothing`, and `0` is not a
pole. A reader who computes poles from a presentation rather than from the reduced
form will disagree with every card in this chapter at such points.

The zero rational function has numerator `0` and denominator `1`, hence no poles at
all; polynomials likewise, since `RatFunc.denom_algebraMap` gives denominator `1`.

#### Pedagogical prerequisites

None within the book; the definition rests on Mathlib's `RatFunc`.

#### Historical context

Defining the poles of a rational function as the zeros of its denominator in lowest
terms is classical and needs no history; what is worth noting is that the formal
development gets "lowest terms" for free from Mathlib's `RatFunc`, whose canonical
`num`/`denom` are exactly the reduced pair. A development on raw fractions would have
to reduce by hand before any of the later cards could be stated.
Source boundary: Mathlib 4.32.1 `RatFunc` declarations, and the registered Crouzeix packet, which owns the definition.
Review status: registered for content-wave review.

#### ML analogy

Mathematical object: the singular set of a rational map, read off a canonical form.
ML counterpart: the set of inputs on which a rational activation or filter has no finite value, determined from its reduced parametrization rather than from the parametrization used in code.
Exact transfer: two implementations of the same rational function have the same singular set, even if one cancels a common factor and the other does not.
Non-transfer: floating-point evaluation of an unreduced form near a cancelled root is numerically unstable even though the mathematical object has no pole there.
Diagnostic: a reported "pole" at a point where the reduced denominator does not vanish is an artifact of the presentation, not a property of the function.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.rational_pole_set`.
Formal mode: `definition`.
Underlying declaration: `CrouzeixConjecture.rationalPoleSet`.
Substantive provider: none — this is a definition, so there is no proof to narrate; `CrouzeixConjecture` is a maintained prefix, so the alias names its target and the definition carries a receipt entry of its own.
Readable type map: `r` is the rational function; the value is the zero set of its reduced denominator.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L9).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `RatFunc.{0} Complex → Set.{0} Complex`.
Type SHA-256: `df698834284278b6673d92defe7d321fca0fcc2d07036d8d14f241e1f415eb30`.
Direct maintained dependencies: `CrouzeixConjecture.rationalPoleSet`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:df698834284278b6673d92defe7d321fca0fcc2d07036d8d14f241e1f415eb30`.

### CFT-17-002 — the pole set is finite {#cft-17-002}

#### Purpose

The fact that makes the pole set small enough to avoid. Without it, "pole-free on a
neighborhood" would not be a usable hypothesis.

#### Statement

For every `r : RatFunc \mathbb{C}`, `rationalPoleSet r` is finite.

#### Hypothesis ledger

None beyond `r`. The one fact used is that the reduced denominator is nonzero,
`RatFunc.denom_ne_zero`, and that holds for every rational function — the reduced
denominator is monic, hence never the zero polynomial.

#### Proof roadmap

A nonzero polynomial has finitely many roots.

#### Proof

The provider is one line:

    simpa [rationalPoleSet, Polynomial.IsRoot] using
      (Polynomial.finite_setOf_isRoot (RatFunc.denom_ne_zero r))

`Polynomial.finite_setOf_isRoot` is Mathlib's statement that the root set of a nonzero
polynomial is finite; `RatFunc.denom_ne_zero` supplies the nonvanishing; the `simpa`
matches `IsRoot` against the definition of the pole set.

**Provider-proof disclosure.** The mathematical content — finitely many roots — is
Mathlib's, and the card contributes only the identification of the pole set with a
root set and the observation that reduced denominators are nonzero. This is a genuine
theorem with a one-line proof, not an alias.

#### Boundary case

Finite includes empty. A polynomial has an empty pole set, and the card says nothing to
distinguish that from a rational function with several poles: both are "finite".

A consequence the chapter uses but does not compile: since `\mathbb{C}` is infinite and
the pole set is finite, the pole set is never all of `\mathbb{C}`, so every rational
function is pole-free on some nonempty set. Mathlib has this as
`Set.Finite.infinite_compl`; it is not a card here.

#### Pedagogical prerequisites

CFT-17-001.

#### Historical context

That a nonzero polynomial of degree `d` has at most `d` roots is older than complex
analysis; here it enters through Mathlib's `finite_setOf_isRoot`, which gives finiteness
without the degree bound. Nothing in the chapter needs the bound.
Source boundary: Mathlib 4.32.1 polynomial declarations, and the registered Crouzeix packet.
Review status: registered for content-wave review; the disclosure that the content is Mathlib's reviewed.

#### ML analogy

Mathematical object: the singular set of a rational map is finite.
ML counterpart: a rational activation has finitely many poles; numerical sensitivity near those poles is a separate question.
Exact transfer: any compact set disjoint from finitely many points has a positive distance to them.
Non-transfer: "finite" gives no bound on how close a pole can sit to the working region, so this is not a conditioning statement.
Diagnostic: inspect distances to poles and numerical conditioning separately; finitely many poles can affect evaluation throughout nearby neighborhoods.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.rational_poles_finite`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.rationalPoleSet_finite`.
Substantive provider: `CrouzeixConjecture.rationalPoleSet_finite`, one line: `Polynomial.finite_setOf_isRoot` applied to `RatFunc.denom_ne_zero`, matched to the pole set by `simpa`. The content is Mathlib's finiteness of roots; the card contributes the identification.
Readable type map: `r` is the rational function; the conclusion is `Set.Finite` of its pole set.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L11).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ (r : RatFunc.{0} Complex), Set.Finite.{0} (CrouzeixConjecture.rationalPoleSet r)`.
Type SHA-256: `195c96f457562510229e4480cd2d03272c0b82e2a89787efa3ae280ecfbde8bf`.
Direct maintained dependencies: `CrouzeixConjecture.rationalPoleSet_finite`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:195c96f457562510229e4480cd2d03272c0b82e2a89787efa3ae280ecfbde8bf`.

### CFT-17-003 — the complement of the pole set is open {#cft-17-003}

#### Purpose

The bridge to Chapter 16. That chapter's algebraic cards need an *open* set containing
the numerical range on which the function is holomorphic; for a rational function, this
card supplies it.

#### Statement

For every `r : RatFunc \mathbb{C}`, the set `(rationalPoleSet r)^c` is open.

#### Hypothesis ledger

None beyond `r`. Finiteness (CFT-17-002) is what is used, and it is unconditional.

#### Proof roadmap

Finite sets are closed; complements of closed sets are open.

#### Proof

    (rationalPoleSet_finite r).isClosed.isOpen_compl

`Set.Finite.isClosed` is Mathlib's statement that a finite subset of a `T_1` space is
closed, and `IsClosed.isOpen_compl` is the complement. One term, two lemma applications.

**Provider-proof disclosure.** The card's content is the composition of two Mathlib
facts with CFT-17-002. It is indexed separately because its *use* is separate: it is
the declaration the bridge theorem `holomorphicMatrixEval_rational` invokes first, to
manufacture the open `U` that Chapter 16's cards require.

Why openness is worth a card is best seen from what would fail without it. Chapter 16's
locality and algebraic laws are stated on an open `U \supseteq W(A)`, and their proofs
need openness essentially — the approximating matrices' numerical ranges must eventually
sit inside `U`. A rational function pole-free on `W(A)` is holomorphic on the pole
complement, and this card says that complement is a legitimate `U`. CFT-17-E03 is this
step in isolation.

#### Boundary case

Openness of the complement says nothing about connectedness. The complement of a finite
set in `\mathbb{C}` is in fact connected — path-connected, even — but no declaration here
states that and nothing in the chapter uses it. Nor does
the card say anything about the pole set itself being open; it is finite and, unless
empty, is not open.

#### Pedagogical prerequisites

CFT-17-001 and CFT-17-002.

#### Historical context

The domain of holomorphy of a rational function is the complement of its poles, and
that it is open is the reason rational functions were the first class on which the
holomorphic calculus was checked. In this development the step is one line, and its
significance is entirely in being the first line of the bridge to Chapter 16.
Source boundary: Mathlib 4.32.1 topology declarations, and the registered Crouzeix packet.
Review status: registered for content-wave review.

#### ML analogy

Mathematical object: the good region of a rational map is open.
ML counterpart: inputs at which a rational filter is well-defined form an open set, so a small perturbation of a good input stays good.
Exact transfer: a compact working region disjoint from the poles has a uniform margin around it that is still pole-free.
Non-transfer: the size of that margin is not given; openness is qualitative.
Diagnostic: if a slightly perturbed good input becomes bad, the perturbation crossed a pole, which is a finite set — check whether the working region was actually disjoint from it.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.pole_complement_open`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.isOpen_compl_rationalPoleSet`.
Substantive provider: `CrouzeixConjecture.isOpen_compl_rationalPoleSet`, one term: `(rationalPoleSet_finite r).isClosed.isOpen_compl`. Finite sets are closed; complements of closed sets are open.
Readable type map: `r` is the rational function; the conclusion is openness of the pole set's complement.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L13).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ (r : RatFunc.{0} Complex), IsOpen.{0} (Compl.compl.{0} (CrouzeixConjecture.rationalPoleSet r))`.
Type SHA-256: `6546c6b2a84d65b93dde09deef9b30d30d997442b3eb25d3162fd69a00550f9e`.
Direct maintained dependencies: `CrouzeixConjecture.isOpen_compl_rationalPoleSet`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:6546c6b2a84d65b93dde09deef9b30d30d997442b3eb25d3162fd69a00550f9e`.

### CFT-17-004 — pole-freeness on a set {#cft-17-004}

#### Purpose

The hypothesis every later theorem carries. It is what upgrades the total evaluations
of CFT-17-005 and CFT-17-006 from defined to meaningful.

#### Statement

`RationalPoleFreeOn r s` is `Disjoint (rationalPoleSet r) s`: no pole of `r` lies in `s`.

#### Hypothesis ledger

None; a definition. Two equivalent forms are proved beside it and both are used later.
The pointwise form `rationalPoleFreeOn_iff`: pole-free on `s` iff the reduced
denominator vanishes at no point of `s`. The containment form
`rationalPoleFreeOn_iff_subset_compl`: pole-free on `s` iff `s \subseteq (\text{poles})^c`.
The bridge theorem uses both: the second turns "pole-free on `W(A)`" into "`W(A)` is
inside the open set of CFT-17-003", and the first supplies `q \neq 0` on that set, which
is what makes `1/q` differentiable there.

Pole-freeness is monotone in the set — a subset of a pole-free set is pole-free — which
is `Disjoint.mono_right` and is CFT-17-E04. And the pole complement is pole-free,
`rationalPoleFreeOn_compl_rationalPoleSet`, which is the containment form applied to
the identity inclusion.

#### Proof roadmap

A definition. What deserves a roadmap is the pair of consequences that are not cards:
on a pole-free set, `rationalScalarEval r` is continuous
(`continuousOn_rationalScalarEval`) and differentiable
(`differentiableOn_rationalScalarEval`), each by the quotient rule with the pointwise
form supplying the nonvanishing denominator.

#### Proof

Nothing to prove. The two analytic consequences are worth reading because they are
what makes a pole-free rational function *admissible* for Chapter 16: differentiable on
the open pole complement is exactly the hypothesis CFT-16-005 and CFT-16-006 take, with
`U` the complement. The provider proves them by `ContinuousOn.div` and
`DifferentiableOn.div`, with numerator and denominator continuous and differentiable as
polynomials, and the pointwise form of pole-freeness as the third obligation.

#### Boundary case

Pole-free on the empty set holds for every `r`. Pole-free on all of `\mathbb{C}` holds
exactly when the pole set is empty; whether that characterizes polynomials is a
question about `RatFunc.denom` that this chapter does not settle and does not need.

The predicate says nothing about the *numerator*. A rational function pole-free on `s`
may vanish identically on `s` or have zeros there; "pole-free" is a one-sided condition
and no card here bounds `r` from below.

#### Pedagogical prerequisites

CFT-17-001.

#### Historical context

Formulating the admissibility of a rational function as disjointness of its pole set
from a region is the natural predicate for the spectral-set literature, where the
region is the numerical range. The finite-dimensional theorems this book is about are
stated with exactly this hypothesis: `crouzeixRationalBound` in the maintained packet
assumes `RationalPoleFreeOn r (numericalRange A)` and nothing more about `r`.
Source boundary: Mathlib 4.32.1 order and set declarations, and the registered Crouzeix packet.
Review status: registered for content-wave review.

#### ML analogy

Mathematical object: a predicate saying a rational map is regular on a region.
ML counterpart: a validity check that a rational filter's parameters keep its poles outside the operating range before the filter is applied.
Exact transfer: the check is monotone — passing on a larger region implies passing on any subregion.
Non-transfer: passing the check gives no bound on the filter's gain; poles just outside the region can make values enormous.
Diagnostic: check gain and pole separation separately. A large constant function already has large outputs with no poles.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.rational_pole_free_on`.
Formal mode: `definition`.
Underlying declaration: `CrouzeixConjecture.RationalPoleFreeOn`.
Substantive provider: none — this is a definition, so there is no proof to narrate; `CrouzeixConjecture` is a maintained prefix, so the alias names its target and the definition carries a receipt entry of its own.
Readable type map: `r` is the rational function and `s` the set; the value is the proposition that the pole set and `s` are disjoint.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L15).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `RatFunc.{0} Complex → Set.{0} Complex → Prop`.
Type SHA-256: `1e640019cefe8a185b38ea1896cb2b895c1e42e4e8788383ec7e4867497002c6`.
Direct maintained dependencies: `CrouzeixConjecture.RationalPoleFreeOn`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:1e640019cefe8a185b38ea1896cb2b895c1e42e4e8788383ec7e4867497002c6`.

### CFT-17-005 — scalar evaluation {#cft-17-005}

#### Purpose

The scalar function that the matrix calculus is the matrix version of, and the function
Chapter 16's calculus is applied to when the two are compared.

#### Statement

`rationalScalarEval r z = Polynomial.eval z r.num / Polynomial.eval z r.denom`.

#### Hypothesis ledger

None; a definition, total in `z` because division by zero returns `0` in Mathlib. It
agrees with Mathlib's own `RatFunc.eval (RingHom.id \mathbb{C})`, by
`rationalScalarEval_eq_ratFuncEval`, whose proof is `simp` — both are the same reduced
quotient.

#### Proof roadmap

A definition. The one fact to establish is agreement with Mathlib's evaluation, which
is definitional after unfolding.

#### Proof

Nothing to prove. The reason the packet has its own scalar evaluation rather than using
`RatFunc.eval` directly is that the matrix evaluation of CFT-17-006 is built from the
*same* `r.num` and `r.denom`, and the bridge theorem needs the scalar function to be
literally `fun z => num(z) * (denom(z))^{-1}` so that Chapter 16's multiplicativity can
split it. The first step of the closing calculation in `holomorphicMatrixEval_rational` is `rfl`
on exactly this unfolding, and its last step is `rfl` on the definition of matrix
evaluation.

#### Boundary case

At a pole the value is `0`. Not infinity, not undefined: the specific complex number
`0`, because `x / 0 = 0` in Mathlib. CFT-17-E05 records this.

This is a different kind of artifact from Chapter 16's. There, the value off the good
region was an *unspecified* element that the development could not name; here it is a
*specific* value that looks like data. Neither convention marks invalid inputs in the
result type. Both values can be propagated by a consumer without the validity
hypotheses needed for the intended calculus laws. Here `0` simplifies cleanly,
but its presence alone does not distinguish a pole from an ordinary zero.

#### Pedagogical prerequisites

CFT-17-003 and CFT-17-004.

#### Historical context

Evaluating a rational function at a point of its domain is not a topic with a history;
the point of the card is the totalization convention, which is a proof-assistant
artifact and not a mathematical choice. Mathlib's decision that `x / 0 = 0` is what makes
this definition total, and the chapter inherits it rather than choosing it.
Source boundary: Mathlib 4.32.1 field and `RatFunc` declarations, and the registered Crouzeix packet.
Review status: registered for content-wave review.

#### ML analogy

Mathematical object: pointwise evaluation of a rational map, totalized with `0` at poles.
ML counterpart: a division layer whose implementation returns `0` on a zero denominator rather than raising.
Exact transfer: outputs at legitimate inputs are exact; there is no approximation in the definition.
Non-transfer: the `0` at a pole is not a limit, not a regularization, and not an average of nearby values; it is a convention with no analytic content.
Diagnostic: an exactly-zero output from a rational layer at an input where the function should be large is a pole being silently totalized.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.rational_scalar_eval`.
Formal mode: `definition`.
Underlying declaration: `CrouzeixConjecture.rationalScalarEval`.
Substantive provider: none — this is a definition, so there is no proof to narrate; `CrouzeixConjecture` is a maintained prefix, so the alias names its target and the definition carries a receipt entry of its own.
Readable type map: `r` is the rational function and `z` the point; the value is the reduced quotient at `z`, totalized by division by zero.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L17).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `RatFunc.{0} Complex → Complex → Complex`.
Type SHA-256: `b4be13ee7d5fca850f3f53f9cd7bbacb91727556700fd7b485bd561a2ed52550`.
Direct maintained dependencies: `CrouzeixConjecture.rationalScalarEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:b4be13ee7d5fca850f3f53f9cd7bbacb91727556700fd7b485bd561a2ed52550`.

### CFT-17-006 — matrix evaluation {#cft-17-006}

#### Purpose

The object this chapter exists to define, and the object the constant-two theorems of
Parts V and VI are about when the function is rational.

#### Statement

`rationalMatrixEval r A = polynomialEval r.num A * (polynomialEval r.denom A)^{-1}`,

with `polynomialEval` the polynomial calculus `Polynomial.aeval A`.

#### Hypothesis ledger

None on the definition. In particular no `[Nonempty n]`: the normalized type below
carries only `Fintype` and `DecidableEq`, in contrast to Chapter 16's definition. The
definition is total because Mathlib's matrix inverse is: `(\cdot)^{-1}` returns the
zero matrix on a singular argument, so `rationalMatrixEval r A` is a matrix for every
`r` and every `A`, and equals `\mathrm{num}(A) \cdot 0 = 0` whenever the denominator
matrix is singular.

The hypothesis that makes the value meaningful lives on a theorem the chapter does not
index but cannot do without, `polynomialEval_denom_isUnit_of_rationalPoleFreeOn_numericalRange`:
if `r` is pole-free on `W(A)`, then `\mathrm{denom}(A)` is a unit. That theorem does
need `[Nonempty n]`, and its proof is where the analysis enters this otherwise
algebraic chapter.

**Necessity and use.** Pole-freeness on the *numerical range* is what the engine uses;
nonvanishing of the denominator on the *spectrum* is what its proof actually needs, and
the packet has that version too — `polynomialEval_isUnit_of_forall_ne_zero_on_spectrum`,
stated for an arbitrary polynomial nonvanishing on `matrixSpectrum A` rather than through
the pole-freeness predicate. The numerical range is the stronger, more convenient
hypothesis because it is the one the Crouzeix theorems are stated with. Nothing here
claims a rational function with a pole in `W(A) \setminus \sigma(A)` has a singular
denominator matrix; it need not: `q(A)` is singular exactly when `q` vanishes somewhere
on `\sigma(A)`, by `spectrum.zero_notMem_iff` with `spectrum.map_polynomial_aeval`.

#### Proof roadmap

For the definition, none. For the engine: a matrix is a unit iff `0` is not in its
spectrum; the spectrum of `q(A)` is `q` applied to the spectrum of `A` (spectral
mapping for polynomials); so `q(A)` is singular iff `q` vanishes somewhere on the
spectrum, which lies inside the numerical range, where `q` does not vanish.

#### Proof

The engine's proof, read in order:

    rw [← spectrum.zero_notMem_iff ℂ]
    rw [polynomialEval, spectrum.map_polynomial_aeval]
    rintro ⟨z, hz, hzero⟩
    have hzW : z ∈ numericalRange A := matrixSpectrum_subset_numericalRange A hz
    exact (rationalPoleFreeOn_iff r (numericalRange A)).mp hfree z hzW hzero

`spectrum.zero_notMem_iff` turns "unit" into "zero not in the spectrum";
`spectrum.map_polynomial_aeval` is Mathlib's polynomial spectral mapping theorem, after
which a zero in the spectrum of `q(A)` is a point `z` of the spectrum of `A` with
`q(z) = 0`; `matrixSpectrum_subset_numericalRange` places `z` in `W(A)`; and the
pointwise form of pole-freeness says `q(z) \neq 0` there. Contradiction.

**Forward reference.** `matrixSpectrum_subset_numericalRange` is indexed as CFT-20-006,
three chapters ahead. The engine is therefore not narratable from this chapter's
prerequisites alone, and the card does not pretend otherwise: the containment of the
spectrum in the numerical range is used here and proved later. The theorem is compiled
now; the card's correspondence is not yet accepted.

**The bridge to Chapter 16.** The theorem `holomorphicMatrixEval_rational` states that
for `r` pole-free on `W(A)`,

    holomorphicMatrixEval A (rationalScalarEval r) = rationalMatrixEval r A.

Its proof is the place Chapter 16's laws are spent, and it runs as follows. Let `U` be
the pole complement, open by CFT-17-003, containing `W(A)` by the containment form of
CFT-17-004. On `U` the denominator `q` is differentiable as a polynomial and `1/q` is
differentiable because `q` does not vanish there. Chapter 16's multiplicativity
(CFT-16-006) gives `H(q) \cdot H(1/q) = H(q \cdot 1/q)`; Chapter 16's locality
(CFT-16-004) replaces `q \cdot 1/q` by the constant `1` because the two agree on `U`;
Chapter 16's polynomial compatibility (CFT-16-003) identifies `H(q)` with `q(A)` and
`H(1)` with the identity. So `q(A) \cdot H(1/q) = I`, and `Matrix.inv_eq_right_inv`
identifies `H(1/q)` with `q(A)^{-1}`. One more application of multiplicativity splits
`H(p \cdot 1/q)` into `H(p) \cdot H(1/q) = p(A) \cdot q(A)^{-1}`, and the closing calculation opens with `rfl` on the definition of scalar evaluation and
ends with `rfl` on the definition of matrix evaluation.

Two things are notable about this proof. It never uses the engine theorem: the
invertibility of `q(A)` comes out of Chapter 16's calculus as a right inverse, not from
spectral mapping. And it is the only place in the `CrouzeixConjecture` namespace where CFT-16-004's
locality is invoked for its intended purpose — replacing a function by another that
agrees with it near the numerical range; the textbook's own CFT-16-E04 is the other use.

#### Boundary case

The factor order is fixed by the definition: numerator first, then the inverse.
CFT-17-E06 shows the cancellation this order supports directly,
`r(A) \cdot q(A) = p(A)` under pole-freeness. Mathematically the two factors commute —
both are polynomials in `A`, the inverse of a polynomial in `A` being again one by
Cayley–Hamilton — but no declaration in this chapter states that, and the left-hand
cancellation `q(A) \cdot r(A) = p(A)` is not compiled here.

When the denominator matrix is singular, `rationalMatrixEval r A = 0` exactly. As with
CFT-17-005 the artifact is a specific value, not an unspecified one, and it is the zero
matrix regardless of the numerator.

#### Pedagogical prerequisites

CFT-06-004, that polynomial evaluation lands in the algebra generated by `A` — the
reason the numerator and the denominator matrix are functions of `A` alone; that the
inverse factor is too rests on the uncompiled Cayley–Hamilton remark in the boundary
case — and CFT-17-005. Not CFT-16-001: the
definition is algebraic and never unfolds the holomorphic limit. The two calculi meet in
the bridge theorem above, not in this definition.

#### Historical context

The definition `r(A) = p(A) q(A)^{-1}` is algebraic; its invertibility condition is
that `q` have no zero on the spectrum.
Stating the condition on the numerical range instead is the spectral-set viewpoint of
Crouzeix's problem, where `W(A)` replaces `\sigma(A)` as the set that controls norms.
The maintained packet's `crouzeixRationalBound` carries exactly this card's hypothesis.
Source boundary: Mathlib 4.32.1 matrix-inverse and spectrum declarations, and the registered Crouzeix packet.
Review status: registered for content-wave review; the forward reference to CFT-20-006 and the bridge narration reviewed against the providers.

#### ML analogy

Mathematical object: the rational calculus as exact matrix arithmetic, valid under a spectral condition and totalized to zero outside it.
ML counterpart: a rational matrix function implemented as a solve, `p(A)` against `q(A)`, which is what a Padé-type exponential or a rational graph filter actually computes.
Exact transfer: on the valid region the algebraic value agrees with the analytic one — that is the bridge theorem — so the solve computes the same object the contour or limit definitions describe.
Non-transfer: the exact definition returns zero for a singular denominator matrix. It supplies no floating-point error bound for a nearly singular one, and does not specify a numerical solver's failure behavior.
Diagnostic: check denominator invertibility independently of the output. A zero output may be a valid answer, for example for the zero polynomial with denominator `1`.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.rational_matrix_eval`.
Formal mode: `definition`.
Underlying declaration: `CrouzeixConjecture.rationalMatrixEval`.
Substantive provider: none — this is a definition, so there is no proof to narrate; `CrouzeixConjecture` is a maintained prefix, so the alias names its target and the definition carries a receipt entry of its own.
Readable type map: `r` is the rational function and `A` the matrix; the value is `num(A) * (denom(A))⁻¹`, totalized by Mathlib's matrix inverse. No `[Nonempty n]` appears.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L19).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `{n : Type u_1} → [Fintype.{u_1} n] → [DecidableEq.{u_1 + 1} n] → RatFunc.{0} Complex → CrouzeixConjecture.SquareMatrix.{u_1} n → CrouzeixConjecture.SquareMatrix.{u_1} n`.
Type SHA-256: `e0b1f943618f688b0db4dbef311558d41b621aa960c8f7229d8962342abea64d`.
Direct maintained dependencies: `CrouzeixConjecture.rationalMatrixEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:e0b1f943618f688b0db4dbef311558d41b621aa960c8f7229d8962342abea64d`.

## Worked examples

**Example 1 — the resolvent, again.** With `r(z) = 1/(z - \lambda)` as in the running
example and `\lambda \notin W(A)`, the engine theorem makes `A - \lambda I` a unit and
`rationalMatrixEval r A = (A - \lambda I)^{-1}`. The bridge theorem then says Chapter
16's `holomorphicMatrixEval A (fun z => 1/(z - \lambda))` is the same matrix. This is the
simplest non-polynomial instance in which Chapter 16's limit definition has a closed
form; CFT-16-003 supplies one for every polynomial, and the bridge theorem for every
pole-free rational function.

**Example 2 — a polynomial is a rational function with no poles.** For a polynomial
`p`, `RatFunc.num_algebraMap` and `RatFunc.denom_algebraMap` give reduced presentation
`p / 1`. The pole set is empty, pole-freeness holds on every set, and
`rationalMatrixEval` of it is `p(A) \cdot 1^{-1} = p(A)`. CFT-17-E02 compiles this; it
is the rational analogue of CFT-16-003, and like that card it needs no hypothesis.

**Example 3 — where the value is zero and should not be.** Take `r(z) = 1/z` and `A` any
singular matrix. Then `0 \in \sigma(A) \subseteq W(A)`, `r` is not pole-free on `W(A)`,
and `rationalMatrixEval r A = I \cdot A^{-1}`, since the reduced denominator of `1/z` is
`z` itself; Mathlib's inverse of the singular matrix `A` is `0`, so the value is the zero
matrix. No card describes this value, and a consumer that expected "the inverse of
`A`" and received `0` has been given the totalization artifact with no error raised.

## ML bridge

Chapter 16 and this chapter are two implementations of the same interface, and the
contrast between them is the lesson.

The analytic implementation covers holomorphic functions through a noncomputable
limit construction; this does not rule out formulas for particular functions.
The algebraic implementation covers rational functions through exact matrix arithmetic.
That an exact, narrow method and
a general, non-constructive definition agree on their common domain is what licenses
using the narrow one as an implementation of the general one; the bridge theorem is
precisely that license, and its hypothesis — pole-free on `W(A)` — is the interface
contract.

Both implementations are totalized, and totalized differently. The analytic one returns
an unspecified value off the good region; the algebraic one returns zero when its
denominator matrix is singular. Failure of the numerical-range condition alone does
not imply this singularity. Neither definition reports invalidity through its result
type. A consumer must retain the hypotheses needed by the relevant theorem rather
than infer validity from the returned value.

The transferable discipline is the one both chapters' cards follow. Validity is a
hypothesis on theorems, never a property of the definition; the definition is total so
that it composes; and every consumer states which predicate it assumes. In this chapter
that predicate is a disjointness condition between a finite set and a compact one, which
is as checkable as such conditions get.

## Lean translation

`CrouzeixTextbook.Part03.Chapter17` re-exports six declarations from
`CrouzeixConjecture.RationalFunctionalCalculus`: four definitions — the pole set, the
pole-freeness predicate, scalar evaluation and matrix evaluation — and two theorems,
that the pole set is finite and that its complement is open.

The two theorems this chapter cannot be understood without are not among the six.
`polynomialEval_denom_isUnit_of_rationalPoleFreeOn_numericalRange`, in the same module,
is what makes the matrix evaluation meaningful, and it reaches forward to CFT-20-006.
`holomorphicMatrixEval_rational`, in `CrouzeixConjecture.HolomorphicConsequences`, is
what identifies this chapter's calculus with Chapter 16's, and it is proved from
CFT-16-003, CFT-16-004 and CFT-16-006 together with CFT-17-003 and CFT-17-004.

The structural fact to take from the Lean is that the rational calculus is defined
without reference to Chapter 16 — `rationalMatrixEval` imports `polynomialEval` and the
matrix inverse, nothing analytic — and that the two chapters are connected by a theorem
rather than by one being defined in terms of the other.

## Exercises with complete solutions

### CFT-17-E01 — the pole set, unfolded {#exercise-cft-17-e01}

State what `rationalPoleSet r` is by definition, and say which denominator it refers to.

#### Complete written solution

It is `\{z \mid \mathrm{eval}\, z\, r.\mathrm{denom} = 0\}`, the zero set of the
*reduced* denominator — Mathlib's canonical `r.denom`, monic and coprime to the
numerator. The statement holds by `rfl`.

The exercise is worth stating for the word "reduced". A pole is a zero of the
denominator in lowest terms, so a factor cancelled in reduction contributes no pole, and
the presentation `z/z` has none. Every hypothesis in the chapter is about this set, and
a reader who computes it from an unreduced presentation will disagree with every card.

**Provider-proof disclosure.** The checked solution is `rfl`; the content is the
statement.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter17.exercise_01_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L28).
Normalized type: `∀ (r : RatFunc.{0} Complex), Eq.{1} (CrouzeixConjecture.rationalPoleSet r) (setOf.{0} fun z => Eq.{1} (Polynomial.eval.{0} z (RatFunc.denom.{0} r)) (OfNat.ofNat.{0} 0))`.
Type SHA-256: `7befd8d17694051579a853dabd3f34282c7ca9102fb4d984ec664924cc1858e8`.
Direct maintained dependencies: `CrouzeixConjecture.rationalPoleSet`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:7befd8d17694051579a853dabd3f34282c7ca9102fb4d984ec664924cc1858e8`.

### CFT-17-E02 — the rational calculus extends the polynomial one {#exercise-cft-17-e02}

Show that for a polynomial `p`, regarded as a rational function, the matrix evaluation
of CFT-17-006 is ordinary polynomial evaluation `p(A)`.

#### Complete written solution

A polynomial `p` becomes a rational function through `algebraMap`. Its reduced
presentation is `p / 1`: `RatFunc.num_algebraMap` gives numerator `p` and
`RatFunc.denom_algebraMap` gives denominator `1`. Unfolding CFT-17-006,

    rationalMatrixEval (algebraMap p) A = p(A) \cdot (1(A))^{-1} = p(A) \cdot 1^{-1} = p(A),

since polynomial evaluation of the constant `1` is the identity matrix, the inverse of
the identity is the identity, and `p(A) \cdot I = p(A)`.

No hypothesis is needed: not pole-freeness, not `[Nonempty n]`, nothing about `A`. This
is the rational analogue of CFT-16-003, and for the same underlying reason — a
polynomial has no poles, so there is no region to condition on.

**Provider-proof disclosure.** The checked solution is an unfolding, two `RatFunc`
rewrites, a `simp` establishing `1(A) = I`, and three closing rewrites; it invokes no
theorem of this chapter and is a calculation on the definition.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter17.exercise_02_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L32).
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] (p : Polynomial.{0} Complex) (A : CrouzeixConjecture.SquareMatrix.{u_1} n), Eq.{u_1 + 1} (CrouzeixConjecture.rationalMatrixEval.{u_1} (DFunLike.coe.{1, 1, 1} (Algebra.algebraMap.{0, 0} (Polynomial.{0} Complex) (RatFunc.{0} Complex)) p) A) (CrouzeixConjecture.polynomialEval.{u_1} p A)`.
Type SHA-256: `3610ab65f2e5ad5c79a3db467f1112b82dc6d52a331cbda54ad8e4b1721bfc24`.
Direct maintained dependencies: `CrouzeixConjecture.SquareMatrix`, `CrouzeixConjecture.polynomialEval`, `CrouzeixConjecture.rationalMatrixEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:3610ab65f2e5ad5c79a3db467f1112b82dc6d52a331cbda54ad8e4b1721bfc24`.

### CFT-17-E03 — the pole-free open neighborhood {#exercise-cft-17-e03}

Prove that if `r` is pole-free on a set `s`, then there is an open set `U \supseteq s`
on which `r` is pole-free.

#### Complete written solution

Take `U` to be the complement of the pole set. It is open by CFT-17-003. It contains `s`
by the containment form of CFT-17-004: pole-free on `s` is exactly
`s \subseteq (\text{poles})^c`. And `r` is pole-free on it by
`rationalPoleFreeOn_compl_rationalPoleSet`, which is the containment form applied to
the identity inclusion.

This is not an idle exercise; it is the first three lines of the bridge theorem
`holomorphicMatrixEval_rational`, which needs an *open* `U \supseteq W(A)` to invoke
Chapter 16's cards and manufactures it exactly this way. The exercise isolates the step
so that its dependence on CFT-17-003 — and hence on finiteness of the pole set — is
visible.

The choice of `U` is canonical but not forced. Any open set between `s` and the pole
complement would do, and for the bridge theorem's purposes the complement is simply the
largest available.

**Provider-proof disclosure.** The checked solution is a triple of three maintained
lemmas — CFT-17-003, the containment form of CFT-17-004, and
`rationalPoleFreeOn_compl_rationalPoleSet` — with no step of its own. Its content is the
assembly, which is the bridge theorem's first three lines.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter17.exercise_03_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L41).
Normalized type: `∀ (r : RatFunc.{0} Complex) (s : Set.{0} Complex), CrouzeixConjecture.RationalPoleFreeOn r s → Exists.{1} fun U => And (IsOpen.{0} U) (And (LE.le.{0} s U) (CrouzeixConjecture.RationalPoleFreeOn r U))`.
Type SHA-256: `e7411bb8f44ed848a6ca9d064458ab5bc15863080fabd204d39c3e8a96beddc4`.
Direct maintained dependencies: `CrouzeixConjecture.RationalPoleFreeOn`, `CrouzeixConjecture.isOpen_compl_rationalPoleSet`, `CrouzeixConjecture.rationalPoleFreeOn_compl_rationalPoleSet`, `CrouzeixConjecture.rationalPoleFreeOn_iff_subset_compl`, `CrouzeixConjecture.rationalPoleSet`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:e7411bb8f44ed848a6ca9d064458ab5bc15863080fabd204d39c3e8a96beddc4`.

### CFT-17-E04 — pole-freeness is monotone {#exercise-cft-17-e04}

Prove that if `r` is pole-free on `t` and `s \subseteq t`, then `r` is pole-free on `s`.

#### Complete written solution

Pole-freeness is disjointness from the pole set, and disjointness from a larger set
implies disjointness from a smaller one. This is `Disjoint.mono_right`.

**Provider-proof disclosure.** The checked solution is a single application of a Mathlib
lemma to the definition. It is set because the direction matters and is easy to get
backwards: pole-freeness passes *down* to subsets, so establishing it on a large
convenient set — the pole complement, or an open neighborhood — establishes it on the
numerical range, which is where the theorems want it. Neither compiled proof in this chapter invokes it: the engine applies the pointwise form
on `W(A)` directly, and the bridge takes pole-freeness on the complement unconditionally.
The direction matters for the reader, not for the Lean.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter17.exercise_04_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L48).
Normalized type: `∀ (r : RatFunc.{0} Complex) {s t : Set.{0} Complex}, LE.le.{0} s t → CrouzeixConjecture.RationalPoleFreeOn r t → CrouzeixConjecture.RationalPoleFreeOn r s`.
Type SHA-256: `969e9c6a45f955c231a70a65ce698e504da555bbcc38e8e05ff905941ef9e7ca`.
Direct maintained dependencies: `CrouzeixConjecture.RationalPoleFreeOn`, `CrouzeixConjecture.rationalPoleSet`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:969e9c6a45f955c231a70a65ce698e504da555bbcc38e8e05ff905941ef9e7ca`.

### CFT-17-E05 — the value at a pole {#exercise-cft-17-e05}

Compute `rationalScalarEval r z` when `z` is a pole of `r`.

#### Complete written solution

It is `0`. At a pole the reduced denominator vanishes, so the definition reads
`\mathrm{num}(z) / 0`, and Mathlib's division returns `0` on a zero divisor
(`div_zero`).

This is the boundary the whole chapter is organized around. The value is not undefined,
not infinite, and not an error: it is the specific complex number `0`, and it will
propagate through any downstream computation that does not check pole-freeness first.
Compare Chapter 16, where the value off the good region was unspecified. Here it is
worse in one respect — it looks like data — and better in another — it is reproducible
and can be reasoned about, as this exercise does.

**Provider-proof disclosure.** The checked solution unfolds the definition, rewrites the
denominator to zero, and applies `div_zero`.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter17.exercise_05_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L53).
Normalized type: `∀ (r : RatFunc.{0} Complex) {z : Complex}, Membership.mem.{0, 0} (CrouzeixConjecture.rationalPoleSet r) z → Eq.{1} (CrouzeixConjecture.rationalScalarEval r z) (OfNat.ofNat.{0} 0)`.
Type SHA-256: `0f4b89996bf255b839d82200f369dd54cb42999928ca8d304ec98dab02515b16`.
Direct maintained dependencies: `CrouzeixConjecture.rationalPoleSet`, `CrouzeixConjecture.rationalScalarEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:0f4b89996bf255b839d82200f369dd54cb42999928ca8d304ec98dab02515b16`.

### CFT-17-E06 — cancelling the denominator {#exercise-cft-17-e06}

Under pole-freeness on `W(A)`, prove `r(A) \cdot q(A) = p(A)`, where `p` and `q` are the
reduced numerator and denominator.

#### Complete written solution

The engine theorem `polynomialEval_denom_isUnit_of_rationalPoleFreeOn_numericalRange`
makes `q(A)` a unit, and `Matrix.isUnit_iff_isUnit_det` converts that to a unit
determinant, which is the form Mathlib's cancellation lemmas take. Unfolding the
definition, the left side is `p(A) \cdot q(A)^{-1} \cdot q(A)`, and
`Matrix.nonsing_inv_mul_cancel_right` cancels the inverse against the matrix on the
right, leaving `p(A)`.

Two remarks. First, this is the direction the definition's factor order supports: the
inverse sits to the right of `p(A)`, so `q(A)` cancels on the right. The left-hand
identity `q(A) \cdot r(A) = p(A)` is also true, since the factors commute, but it is not
what this exercise proves and no declaration in the chapter proves it. Second, this is
the one exercise that needs `[Nonempty n]`, inherited from the engine theorem; the
definition itself does not.

**Provider-proof disclosure.** The checked solution is the engine theorem followed by a
Mathlib cancellation lemma; the mathematical content is the engine's.

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter17.exercise_06_solution`.
Formal mode: `proved-here`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter17.lean#L60).
Normalized type: `∀ {n : Type u_1} [inst : Fintype.{u_1} n] [inst_1 : DecidableEq.{u_1 + 1} n] [Nonempty.{u_1 + 1} n] (r : RatFunc.{0} Complex) (A : CrouzeixConjecture.SquareMatrix.{u_1} n), CrouzeixConjecture.RationalPoleFreeOn r (CrouzeixConjecture.numericalRange.{u_1} A) → Eq.{u_1 + 1} (HMul.hMul.{u_1, u_1, u_1} (CrouzeixConjecture.rationalMatrixEval.{u_1} r A) (CrouzeixConjecture.polynomialEval.{u_1} (RatFunc.denom.{0} r) A)) (CrouzeixConjecture.polynomialEval.{u_1} (RatFunc.num.{0} r) A)`.
Type SHA-256: `d9af88a7d89f2b97ac27c9125fd68354da5fe19a120199778e6ffdcd4ef33203`.
Direct maintained dependencies: `CrouzeixConjecture.RationalPoleFreeOn`, `CrouzeixConjecture.SquareMatrix`, `CrouzeixConjecture.numericalRange`, `CrouzeixConjecture.polynomialEval`, `CrouzeixConjecture.polynomialEval_denom_isUnit_of_rationalPoleFreeOn_numericalRange`, `CrouzeixConjecture.rationalMatrixEval`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:d9af88a7d89f2b97ac27c9125fd68354da5fe19a120199778e6ffdcd4ef33203`.

## Synthesis and forward dependencies

The chapter delivers a rational functional calculus that is exact matrix arithmetic,
total by Mathlib's inverse convention, and meaningful under one hypothesis: no pole on
the numerical range. That hypothesis is the interface contract of the constant-two
theorems for rational functions, and it is carried verbatim by the maintained
`crouzeixRationalBound`.

The forward edges are to the rational Crouzeix results. CFT-17-006 is cited by
CFT-21-005, the Hilbert-space rational spectral-set statement of Part IV, and by
CFT-32-003 and CFT-35-003 — the rational form of the bound on the Jin and
Lorist–Schwenninger routes. The five other cards are not cited by any later card; they
are here because they are what makes the definition a calculus.

The connection to Chapter 16 runs through a theorem, not a card, and not through the
definition. `holomorphicMatrixEval_rational` proves the two calculi agree on pole-free
rational functions, and it is proved from Chapter 16's polynomial compatibility,
locality and multiplicativity together with this chapter's openness and containment
lemmas. Chapter 16's remark that its laws are consumed nowhere as cards is accurate;
this is where they are consumed as theorems.
