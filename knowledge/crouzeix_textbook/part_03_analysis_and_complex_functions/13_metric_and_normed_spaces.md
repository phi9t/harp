---
id: cft-chapter-13-metric-and-normed-spaces
title: Metric and normed spaces
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-10
tags: [crouzeix-textbook, analysis, mathematics, lean]
confidence: high
canonical: 13_metric_and_normed_spaces.md
chapter: 13
part: 3
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 13: Metric and normed spaces

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-iii-analysis-and-complex-functions|Part III -- Analysis and complex functions]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/12_differential_forms_and_stokes|Chapter 12 — Differential forms and Stokes]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/14_sequences_and_series_of_operators|Chapter 14 — Sequences and series of operators]]

## Opening problem

Part II measured vectors and matrices. Part III needs one more thing before it
can estimate anything: a guarantee that a supremum is *attained*, so that a bound
stated as "the largest value of `‖f‖` on this set" names an actual number
belonging to an actual point rather than an unreachable limit.

That guarantee is the quiet engine of every contour estimate in Part V. Bounding
`‖p(A)‖` by the maximum of `|p|` over a boundary is worthless unless the maximum
exists, dominates the function pointwise, has a known sign, does not decrease when
the contour is enlarged, and is stable when the contour is approximated from
outside. Those five properties are what this chapter compiles. Finiteness is not
among them: no declaration here establishes that the image is bounded above, and
the compactness hypotheses are what stand in for it.

## What this chapter compiles, and what it does not

The chapter is titled for metric and normed spaces, and it does not develop them.
**Not compiled here:** Cauchy sequences, completeness, the equivalence of norms on
a finite-dimensional space, continuity of a finite-dimensional linear map,
compactness of the unit sphere, and the failure of compactness for the closed unit
ball in infinite dimensions. None of them is compiled *in this chapter*; some, such
as sphere compactness, are used elsewhere in the maintained development. The
earlier sketch set five of these six topics as exercises, and those five are
withdrawn; its sixth exercise was a reading of monotonicity and survives below as
E06.

What *is* compiled is one functional and five of its properties:
$$
M(s,f)=\sup_{z\in s}\lVert f(z)\rVert ,
$$
attained on a nonempty compact set, dominating `f` pointwise, nonnegative,
monotone in the set, and continuous under outer approximation of the set. Five of
the six cards are `reexported-proof` rows over genuine multi-step
`CrouzeixConjecture` proofs; the sixth, CFT-13-001, is the definition itself.

The compactness theorems the chapter's title suggests are used, not proved: they
enter through Mathlib, as `IsCompact.exists_isMaxOn` and the Heine–Cantor theorem
`IsCompact.uniformContinuousOn_of_continuous`.

## Conceptual model

A metric turns "near" into an inequality; a norm additionally respects addition
and scalars. The property this chapter needs from them is not any particular
inequality but a two-step pattern that recurs throughout Part V.

First, *existence by compactness*: a continuous real function on a nonempty
compact set attains its supremum, so `M(s,f)` is a value and not merely a bound.
Second, *stability under approximation*: if compact sets `s k` each contain a
compact `K`, each lie within `radius k` of it with `radius k → 0`, and all sit
inside one compact set on which the function is continuous, then the maxima
converge. The family is not assumed nested. The second is where uniform
continuity enters, and it is the only place in the chapter where an `ε`-`δ`
argument appears.

Everything else is bookkeeping that makes those two usable: a pointwise bound, a
sign, and monotonicity in the set.

## Running example: a polynomial on nested disks

Take `f(z)=z^2` and the closed disks `D_r` of radius `r` about the origin. On
`D_r` the maximum modulus is `r^2`, attained at every boundary point. The
functional is monotone — `D_1 \subseteq D_2` gives `1 \le 4` — and nonnegative,
and the approximation statement says that maxima over disks of radius
`1+1/k` converge to the maximum over `D_1` as `k \to \infty`, which here is the
elementary limit `(1+1/k)^2 \to 1`.

The chapter compiles none of those particular values; the disk is an
illustration, and `f(z) = z^2` appears in no compiled statement. What it compiles
is the general shape the illustration instantiates.

## Formal development

### CFT-13-001 — the maximum-modulus functional {#cft-13-001}

#### Purpose

Name the quantity the rest of Part III and Part V estimate, once, so that later
cards can state properties of it rather than re-deriving a supremum each time.

#### Statement

This is a definition, not a theorem. For a set `s ⊆ ℂ` and a function
`f : ℂ → ℂ`,
$$
M(s,f)=\sup\,\{\lVert f(z)\rVert : z\in s\},
$$
the supremum of the image of `s` under `z ↦ ‖f z‖`. The card asserts nothing
about that supremum: not that it is attained, not that it is finite, not that `s`
is nonempty or compact.

#### Hypothesis ledger

None. The definition is total: it accepts any set and any function, including
sets on which the supremum is meaningless. That totality is deliberate and is
what forces every later card to carry its own compactness and nonemptiness
hypotheses.

#### Proof roadmap

Unfold the definition: read `M(s,f)` as `sSup` applied to the image of `s`.

#### Proof

The definitional expansion is
$$
M(s,f)=\operatorname{sSup}\bigl((z\mapsto\lVert f(z)\rVert)\,''\,s\bigr),
$$
where `''` is image. Exercise E01 performs exactly this unfolding in Lean and
adds the one immediate consequence, that the image of a nonempty set is nonempty.

#### Boundary case

On the empty set the definition returns a junk value. Mathlib's real supremum of
the empty set is `0`, so `M(∅,f)=0` for every `f`, even though no point attains
it and every `‖f z‖` could be large elsewhere. Exercise E05 compiles that value.
It is the reason `s.Nonempty` appears as a hypothesis in all five theorem cards
below rather than being folded into the definition.

An unbounded set is the other degenerate case: `sSup` of an unbounded real set is
again a junk value, and the compactness hypothesis of the later cards is what
excludes it.

#### Historical context

Defining a maximum-modulus quantity before knowing it is attained is the modern
convention, and it is what lets the attainment theorem be stated as a property of
an already-named object. The classical literature usually writes `\max` and
carries the attainment implicitly, which is exactly the conflation this card's
totality prevents.
Source boundary: Mathlib 4.32.1 order and norm declarations, and the registered Crouzeix geometry packet.
Review status: definition and its two degenerate cases reviewed together.

#### ML analogy

Mathematical object: the supremum of a continuous nonnegative function over a set.
ML counterpart: the worst-case loss over a constraint region, as written in a robust or adversarial objective.
Exact transfer: the definition is the same supremum, and it is a well-defined real number whenever the region is nonempty and the objective is bounded on it.
Non-transfer: the definition alone gives no attainment, so nothing here says an adversarial inner maximization has a solution to find; that needs the compactness of the next card.
Diagnostic: an inner maximization that returns a value no input achieves points at a hypothesis of the next card rather than at the search — either the region is not compact or the objective is not continuous on it.

#### Pedagogical prerequisites

Chapter 7's norm and the notion of a supremum of a set of reals.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.max_modulus_on_compact_set`.
Formal mode: `definition`.
Underlying declaration: `CrouzeixConjecture.maxFunctionModulusOnSet`.
Substantive provider: `none`; a definition has no proof obligation. The named declaration is an alias of the definition, which the underlying-declaration line names.
Readable type map: `s` is the set, `f` the function, and the value is the supremum of `‖f‖` over `s`.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L9).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `Set.{0} Complex → (Complex → Complex) → Real`.
Type SHA-256: `08e98f1a059154fe41a72cad9d2cedf64b73e7612e856619a4273a4c84d006aa`.
Direct maintained dependencies: `CrouzeixConjecture.maxFunctionModulusOnSet`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: none; the definition is total.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:08e98f1a059154fe41a72cad9d2cedf64b73e7612e856619a4273a4c84d006aa`.

#### Exercises and solutions

CFT-13-E01 unfolds the definition; CFT-13-E02 evaluates it on a constant function,
needing nonemptiness but no compactness; CFT-13-E05 computes its value on the empty
set, which is what makes the nonemptiness hypotheses of the other cards visible.

### CFT-13-002 — the maximum is attained {#cft-13-002}

#### Purpose

Turn the supremum into a value at a point. Every later estimate that says "pick
the worst point on the contour" depends on this card.

#### Statement

If `s` is compact and nonempty and `f` is continuous on `s`, then there is a
`z ∈ s` with `‖f z‖ = M(s,f)`.

#### Hypothesis ledger

All three are load-bearing. Compactness supplies the maximizer; nonemptiness
excludes the empty case where the definition returns junk; continuity is needed
on `s` only, not on any neighbourhood, which is what lets later cards apply the
result to a closed contour without extending `f` past it.

#### Proof roadmap

Apply Mathlib's compact-maximum theorem to `‖f‖`, then identify the resulting
greatest element of the image with the supremum.

#### Proof

The composite `z ↦ ‖f z‖` is continuous on `s`, being the norm of a continuous
function. Mathlib's `IsCompact.exists_isMaxOn` applied to it yields a point `z ∈ s`
at which it is maximal over `s`.

That maximality is then upgraded to a statement about the supremum. The provider
builds `IsGreatest ((w ↦ ‖f w‖) '' s) ‖f z‖` — membership from `z ∈ s`, and the
upper bound from maximality applied to an arbitrary image element — and concludes
with `IsGreatest.csSup_eq`, which says a greatest element of a set *is* its
supremum. Unfolding `M(s,f)` as the supremum of that image finishes.

The two halves are worth keeping apart: compactness gives a maximizer, and
`IsGreatest.csSup_eq` is what connects a maximizer to the supremum the definition
names.

#### Worked instance

For `f(z)=z^2` on the closed unit disk, every boundary point attains the value
`1`. Attainment does not assert uniqueness, and here the maximizer is a whole
circle.

#### Boundary case

Drop compactness and attainment fails: on the *open* unit disk the supremum of
`|z^2|` is `1` and no point reaches it. Drop nonemptiness and the conclusion is
false for a different reason — there is no `z` at all, while the definition still
returns `0`. Neither degenerate case is compiled; the empty-set value is, as
exercise E05.

#### Historical context

The extreme value theorem is Weierstrass's, and the compactness formulation is
the one that survives into infinite dimensions unchanged — where it stops being
implied by closed and bounded, which is exactly the distinction the chapter's
title promises and does not compile.
Source boundary: Mathlib 4.32.1 compactness declarations and the registered Crouzeix geometry packet.
Review status: compactness step and supremum-identification step reviewed separately.

#### ML analogy

Mathematical object: attainment of a supremum by a continuous function on a compact set.
ML counterpart: the inner maximization of a robust objective over a compact constraint set.
Exact transfer: on a compact region a continuous loss has a genuine worst case, so the inner problem has a solution.
Non-transfer: nothing here says that solution is unique, that it is findable, or that gradient ascent reaches it.
Diagnostic: an inner maximization whose value keeps improving without converging suggests a non-compact region rather than a tuning problem.

#### Pedagogical prerequisites

CFT-13-001, continuity on a set, and compactness.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.compact_maximum_exists`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.exists_maxFunctionModulusOnSet`.
Substantive provider: `CrouzeixConjecture.exists_maxFunctionModulusOnSet`, a genuine multi-step proof: `IsCompact.exists_isMaxOn` on `‖f‖`, then `IsGreatest` assembly, then `IsGreatest.csSup_eq`.
Readable type map: `s` is the compact set, `f` the function, and `z` the attaining point.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L11).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {s : Set.{0} Complex} {f : Complex → Complex}, IsCompact.{0} s → Set.Nonempty.{0} s → ContinuousOn.{0, 0} f s → Exists.{1} fun z => And (Membership.mem.{0, 0} s z) (Eq.{1} (Norm.norm.{0} (f z)) (CrouzeixConjecture.maxFunctionModulusOnSet s f))`.
Type SHA-256: `477120b54163fa7c24b36142f0e0cdca3d9c36244b12566149b5ea20cc15c1ea`.
Direct maintained dependencies: `CrouzeixConjecture.exists_maxFunctionModulusOnSet`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: compact nonempty `s` and `f` continuous on `s`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:477120b54163fa7c24b36142f0e0cdca3d9c36244b12566149b5ea20cc15c1ea`.

#### Exercises and solutions

CFT-13-E03 states the attainment and the pointwise bound together as an
`IsGreatest`, which is the form the two cards jointly deliver.

### CFT-13-003 — the maximum dominates pointwise {#cft-13-003}

#### Purpose

The half of "maximum" that estimates use most: every point is bounded by it.

#### Statement

If `s` is compact and nonempty, `f` is continuous on `s`, and `z ∈ s`, then
`‖f z‖ ≤ M(s,f)`.

#### Hypothesis ledger

The same three as CFT-13-002, plus membership `z ∈ s`. Compactness is genuinely
load-bearing, and not only for the proof. Mathlib's real supremum returns the junk
value `0` on a set unbounded above, so without a boundedness hypothesis the
statement is false: take `s = ℝ` inside `ℂ`, `f` the identity and `z = 1`. The
image of `‖f‖` is `[0,∞)`, so `M(s,f) = 0`, and `1 ≤ 0` fails. Nonemptiness is
implied by `z ∈ s` and is carried only to match the other cards.

#### Proof roadmap

Rebuild the greatest element, then read off the upper-bound half.

#### Proof

A supremum bounds its set whenever that set is bounded above — and only then, in
Mathlib's convention, since `Real.sSup_of_not_bddAbove` returns `0` otherwise. The
provider establishes boundedness the same way CFT-13-002 does, by re-running the
maximizer construction: it takes a maximizer `w` from `IsCompact.exists_isMaxOn`, assembles
`IsGreatest ((y ↦ ‖f y‖) '' s) ‖f w‖`, converts it to the supremum with
`IsGreatest.csSup_eq`, and then closes by applying the `IsMaxOn` witness directly
at `z ∈ s` — not the `IsGreatest` upper-bound field, though the two agree
definitionally.

That is why compactness appears: the supremum is an upper bound only once the
image is known to be bounded above, and this proof supplies that by exhibiting a
maximum rather than by a separate `BddAbove` argument. A version assuming only
`BddAbove ((y ↦ ‖f y‖) '' s)` would be strictly more general, and is not compiled
here.

#### Worked instance

For `f(z)=z^2` on the closed unit disk and `z = 1/2`, the bound reads
`1/4 \le 1`.

#### Boundary case

The inequality is not strict and cannot be: at a maximizer it is an equality,
which is precisely CFT-13-002.

#### Historical context

That a supremum is an upper bound for a set bounded above is definitional; that it
is *attained* is the theorem. Keeping the two as separate cards is what lets a
later proof cite only the half it needs, and Part V mostly needs this one.
Source boundary: Mathlib 4.32.1 compactness and order declarations, and the registered Crouzeix geometry packet, which owns the provider.
Review status: reviewed against CFT-13-002, with the unbounded-set counterexample checked.

#### ML analogy

Mathematical object: a pointwise bound by a supremum over a region.
ML counterpart: certifying that no input in a constraint region exceeds a stated loss bound.
Exact transfer: every point of the region satisfies the bound, with equality somewhere.
Non-transfer: the bound says nothing about inputs outside the region, which is where a deployed model usually fails.
Diagnostic: a point violating the bound is outside the region, not a counterexample to the inequality.

#### Pedagogical prerequisites

CFT-13-001 and CFT-13-002.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.pointwise_norm_le_maximum`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.norm_function_le_maxFunctionModulusOnSet`.
Substantive provider: `CrouzeixConjecture.norm_function_le_maxFunctionModulusOnSet`, which re-runs the maximizer construction rather than citing CFT-13-002, then closes with the `IsMaxOn` witness at `z`.
Readable type map: `s` is the compact set, `f` the function, `z` the point being bounded.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L13).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {s : Set.{0} Complex} {f : Complex → Complex}, IsCompact.{0} s → Set.Nonempty.{0} s → ContinuousOn.{0, 0} f s → ∀ {z : Complex}, Membership.mem.{0, 0} s z → LE.le.{0} (Norm.norm.{0} (f z)) (CrouzeixConjecture.maxFunctionModulusOnSet s f)`.
Type SHA-256: `c77f5a3302ab3a64e6b993aeffdecce5675dfbebaa9ec2b23b9f7216b45999ab`.
Direct maintained dependencies: `CrouzeixConjecture.norm_function_le_maxFunctionModulusOnSet`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: compact nonempty `s`, `f` continuous on `s`, and `z ∈ s`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:c77f5a3302ab3a64e6b993aeffdecce5675dfbebaa9ec2b23b9f7216b45999ab`.

#### Exercises and solutions

CFT-13-E03 packages this bound with CFT-13-002's attainment as a single
`IsGreatest` statement.

### CFT-13-004 — the maximum is nonnegative {#cft-13-004}

#### Purpose

A sign, needed wherever the maximum is multiplied by another bound or appears
under a square root.

#### Statement

If `s` is compact and nonempty and `f` is continuous on `s`, then `0 ≤ M(s,f)`.

#### Hypothesis ledger

The same three, and none of them is necessary — see the Proof. They are present
because the provider routes through attainment; the sign itself follows from the
definition alone.

#### Proof roadmap

Attain the maximum, then use that a norm is nonnegative.

#### Proof

By CFT-13-002 there is a `z ∈ s` with `‖f z‖ = M(s,f)`. Rewriting the goal
backwards along that equality turns `0 ≤ M(s,f)` into `0 ≤ ‖f z‖`, which is
`norm_nonneg`. The checked proof is exactly those three steps: `obtain`, `rw [← hmax]`,
`exact norm_nonneg _`.

Note which direction the dependency runs *in this provider*: the sign is derived
from attainment. That is a fact about the proof, not about the statement. The
conclusion holds with all three hypotheses deleted: Mathlib's `Real.sSup_nonneg`
gives `0 ≤ sSup S` for any set `S` of nonnegative reals, covering the empty and
unbounded cases through the same junk value `0`, and `‖f z‖` is always
nonnegative. So a strictly more general card is available and is not compiled
here; the hypotheses are inherited from the route through attainment.

#### Worked instance

For any `f` on the closed unit disk the maximum is at least `0`. It equals `0`
exactly when `f` vanishes identically on the disk — true, and not compiled: no
declaration in this chapter states the equality case in either direction.

#### Boundary case

The bound `0 ≤ M(∅,f)` also holds, since `M(∅,f)=0` by exercise E05. It does not
follow from this card, whose hypotheses exclude the empty set, but it does follow
from the more general statement named in the Proof — the two are the same fact
reached by different routes.

#### Historical context

Explicit nonnegativity cards are an artefact of formalization: on paper the sign
of a maximum of moduli is never mentioned. In a checked development it has to be
available as a term, because a later `nlinarith` or square-root step will ask for
it by name.
Source boundary: Mathlib 4.32.1 norm declarations, and the registered Crouzeix geometry packet, which owns the provider.
Review status: the derivation from attainment, rather than from the definition, reviewed explicitly.

#### ML analogy

Mathematical object: nonnegativity of a supremum of norms.
ML counterpart: the guarantee that a worst-case loss built from magnitudes is never negative.
Exact transfer: the worst-case value is never negative, for any objective built from magnitudes over any region.
Non-transfer: a loss that can go negative is not of this form, and nothing here applies to it.
Diagnostic: a negative worst-case value indicates the objective is not built from magnitudes; an empty region yields zero, not a negative number.

#### Pedagogical prerequisites

CFT-13-002 and the nonnegativity of a norm, which is Chapter 7's CFT-07-006.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.compact_maximum_nonnegative`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.maxFunctionModulusOnSet_nonneg`.
Substantive provider: `CrouzeixConjecture.maxFunctionModulusOnSet_nonneg`, three steps: attain by CFT-13-002's provider, rewrite backwards, apply `norm_nonneg`.
Readable type map: `s` is the compact set and `f` the function; the value bounded below is the maximum.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L15).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {s : Set.{0} Complex} {f : Complex → Complex}, IsCompact.{0} s → Set.Nonempty.{0} s → ContinuousOn.{0, 0} f s → LE.le.{0} (OfNat.ofNat.{0} 0) (CrouzeixConjecture.maxFunctionModulusOnSet s f)`.
Type SHA-256: `495bc7be9d3a28d8fb02c8c329c0e3a46a2a6d915bc1b7ac6b59736e680d8007`.
Direct maintained dependencies: `CrouzeixConjecture.maxFunctionModulusOnSet_nonneg`, which calls `exists_maxFunctionModulusOnSet`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: compact nonempty `s` and `f` continuous on `s`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:495bc7be9d3a28d8fb02c8c329c0e3a46a2a6d915bc1b7ac6b59736e680d8007`.

#### Exercises and solutions

CFT-13-E04 runs this derivation itself, stating the attainment it uses as an
explicit conjunct rather than citing this card.

### CFT-13-005 — enlarging the set cannot decrease the maximum {#cft-13-005}

#### Purpose

Monotonicity in the set. Part V enlarges a contour and needs to know the estimate
degrades in the safe direction.

#### Statement

If `s ⊆ t`, both compact and nonempty, and `f` is continuous on `t`, then
`M(s,f) ≤ M(t,f)`.

#### Hypothesis ledger

Six hypotheses, of which three are removable. Compactness of `t` together with
continuity on `t` is what bounds the image and makes the right-hand supremum
meaningful; the inclusion is what compares them. Compactness of `s`, nonemptiness
of `s`, and nonemptiness of `t` are all artefacts of the route through
attainment — with `s` empty the claim reads `0 ≤ M(t,f)`, which holds by the
general nonnegativity named in CFT-13-004, and `t.Nonempty` follows from
`s.Nonempty` with the inclusion in any case. Continuity is assumed on `t` only and
restricted to `s` where needed, which is the right direction: assuming it on `s`
would not support the right-hand side.

#### Proof roadmap

Attain the smaller maximum at a point, then bound that point by the larger
maximum.

#### Proof

By CFT-13-002 applied to `s`, with continuity restricted along the inclusion,
there is a `z ∈ s` with `‖f z‖ = M(s,f)`. Since `s ⊆ t`, that same `z` lies in
`t`, so CFT-13-003 applied to `t` gives `‖f z‖ ≤ M(t,f)`. Rewriting backwards
along the attainment equality turns this into the claim. The checked proof is
those three steps, using `ContinuousOn.mono` for the restriction and `hst hz` to
transport membership.

The argument is a clean illustration of why both previous cards are separately
useful: attainment is used on the small set and the pointwise bound on the large
one.

#### Worked instance

For `f(z)=z^2` and the disks `D_1 \subseteq D_2`, the maxima are `1` and `4`.

#### Boundary case

Monotone, not strictly monotone: enlarging a set need not increase the maximum,
and does not when the new points carry no larger value. Taking `s = t` gives
equality.

#### Historical context

Monotonicity of a supremum in its index set is *not* immediate from the
definition in a conditionally complete order: `csSup_le_csSup` needs the larger
set bounded above and the smaller nonempty. What supplies boundedness here is
compactness of `t` with continuity on it. Compactness of `s` is an artefact of
this provider's route through attainment, not a requirement of the statement.
Source boundary: Mathlib 4.32.1 compactness declarations, and the registered Crouzeix geometry packet, which owns the provider.
Review status: the asymmetry in the continuity hypothesis reviewed explicitly.

#### ML analogy

Mathematical object: monotonicity of a supremum under set inclusion.
ML counterpart: widening a constraint region never lowers the worst-case loss.
Exact transfer: the inequality is exact and requires no relationship between the two regions beyond inclusion.
Non-transfer: nothing here quantifies how much the worst case grows, so it gives no budget for relaxing a constraint.
Diagnostic: a wider region reporting a strictly smaller worst case indicates the inner maximization did not converge on the wider one.

#### Pedagogical prerequisites

CFT-13-002 and CFT-13-003.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.compact_maximum_monotone`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.maxFunctionModulusOnSet_mono`.
Substantive provider: `CrouzeixConjecture.maxFunctionModulusOnSet_mono`, which attains on `s` via CFT-13-002's provider and bounds on `t` via CFT-13-003's.
Readable type map: `s` and `t` are the smaller and larger compact sets and `f` the function.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L17).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {s t : Set.{0} Complex} {f : Complex → Complex}, IsCompact.{0} s → Set.Nonempty.{0} s → IsCompact.{0} t → Set.Nonempty.{0} t → ContinuousOn.{0, 0} f t → LE.le.{0} s t → LE.le.{0} (CrouzeixConjecture.maxFunctionModulusOnSet s f) (CrouzeixConjecture.maxFunctionModulusOnSet t f)`.
Type SHA-256: `922c2f0e2aa205a9fb7bfc96cdd44d88c67419b4763dab024c6091f42f4b0109`.
Direct maintained dependencies: `CrouzeixConjecture.maxFunctionModulusOnSet_mono`, which calls `exists_maxFunctionModulusOnSet` and `norm_function_le_maxFunctionModulusOnSet`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: compact nonempty `s` and `t`, `f` continuous on `t`, and `s ⊆ t`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:922c2f0e2aa205a9fb7bfc96cdd44d88c67419b4763dab024c6091f42f4b0109`.

#### Exercises and solutions

CFT-13-E06 pairs this inequality with the nonnegativity of the smaller maximum,
which is the two-sided form a later estimate consumes.

### CFT-13-006 — maxima converge under outer approximation {#cft-13-006}

#### Purpose

The chapter's one genuine analysis theorem. It says the maximum-modulus
functional is continuous along a shrinking family of compact sets, which is what
licenses replacing an awkward contour by a sequence of convenient ones.

#### Definitions and notation

`K` is the target compact set, `C` an ambient compact set containing everything,
and `s k` a sequence of compact sets with `K ⊆ s k ⊆ C`. The approximation is
quantitative: `radius k → 0`, and every point of `s k` is within `radius k` of
some point of `K`.

#### Statement

Under those hypotheses, `M(s k, f) → M(K, f)` as `k → ∞`.

#### Hypothesis ledger

The ambient `C` supplies a single set on which `f` is assumed continuous; a
per-`k` hypothesis `ContinuousOn f (s k)` would not glue into the uniform
continuity this proof needs. Its *compactness* is what this proof feeds to
Heine-Cantor, but it is not necessary for the statement: the union of the `s k` is
already compact under the remaining hypotheses — bounded because each `s k` lies
within `radius k` of `K` and `radius` converges, closed because a limit of points
with `d(·,K) → 0` lands in `K` — so one could always take that union as the
ambient set. The hypothesis buys a shorter proof, not a stronger theorem.
`K ⊆ s k` gives the lower bound by monotonicity, and the `radius` condition gives
the upper bound. Nonemptiness of `K` propagates to each `s k` through the
inclusion.

#### Proof roadmap

Uniform continuity of `‖f‖` on `C` converts closeness of points into closeness of
values; monotonicity gives one inequality and the approximation gives the other.

#### Proof

Write `g = ‖f·‖`. By Heine–Cantor on the compact `C`, `g` is *uniformly*
continuous there — this is the step the ambient set exists for.

Fix `η > 0` and take the `δ` uniform continuity provides. Since `radius k → 0`,
there is an `N` beyond which `radius k < δ`.

Fix `k ≥ N`. The lower bound is monotonicity: `K ⊆ s k` gives
`M(K,f) ≤ M(s k,f)` by CFT-13-005.

For the upper bound, attain the larger maximum: by CFT-13-002 there is
`z ∈ s k` with `g(z) = M(s k,f)`. The approximation hypothesis supplies a
`w ∈ K` with `‖z - w‖ ≤ radius k < δ`, so uniform continuity gives
`|g(z) - g(w)| < η`. And `g(w) ≤ M(K,f)` by CFT-13-003, since `w ∈ K`. Chaining,
$$
M(s\,k,f)=g(z)<g(w)+\eta\le M(K,f)+\eta .
$$
The two bounds together give `|M(s k,f) - M(K,f)| < η`, and the checked proof
closes both sides with `linarith`.

Note where each earlier card is consumed: CFT-13-005 for the lower bound,
CFT-13-002 to produce `z`, CFT-13-003 to bound `g(w)`. The only ingredient not
from this chapter is uniform continuity.

#### Worked instance

For `f(z)=z^2`, `K` the closed unit disk and `s k` the closed disk of radius
`1+1/k` for `k \ge 1`, the maxima are `(1+1/k)^2 → 1`. The `radius k = 1/k`
hypothesis holds because a point `z` with `1 < |z| \le 1+1/k` has `z/|z|` in the
unit disk at distance `|z|-1 \le 1/k`, and a point inside is at distance `0`. The
ambient `C` can be taken to be the disk of radius `2`.

#### Boundary case

Convergence *to `M(K,f)`* needs the quantitative approximation, not merely
`K ⊆ s k`. A decreasing family that contains `K` but always retains a far-away
point still has convergent maxima — they are non-increasing and bounded below — but
the limit is wrong: with `K` the unit disk and `s k = K ∪ [2, 2+1/k]`, the maxima
converge to `max(M(K,f), |f(2)|)`. Dropping monotonicity of the family as well
gives genuine non-convergence, by alternating a far-away point between two values.
The `radius` hypothesis forbids both.

#### Historical context

The argument is the standard one for continuity of a supremum under Hausdorff
convergence of compact sets, specialized to one-sided approximation from outside,
which is the only case the contour construction needs.
Source boundary: Mathlib 4.32.1 Heine-Cantor and filter declarations, and the registered Crouzeix geometry packet.
Review status: the role of the ambient set `C` reviewed explicitly.

#### ML analogy

Mathematical object: continuity of a supremum under outer approximation of the region.
ML counterpart: replacing a constraint region by a slightly enlarged, easier one and expecting the worst-case loss to change little.
Exact transfer: with a quantitative shrinking radius and a common compact ambient region, the worst-case values converge.
Non-transfer: no rate is provided; the convergence depends on the modulus of continuity, which the statement does not expose.
Diagnostic: worst-case values that settle on the wrong limit indicate the relaxation was not shrinking quantitatively onto the region; values that do not settle at all indicate a discontinuous objective.

#### Pedagogical prerequisites

CFT-13-002, CFT-13-003, CFT-13-005, and uniform continuity on a compact set.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.outer_maxima_converge`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.tendsto_maxFunctionModulusOnSet_of_outerApproximation`.
Substantive provider: `CrouzeixConjecture.tendsto_maxFunctionModulusOnSet_of_outerApproximation`, the chapter's longest provider: Heine-Cantor, an epsilon-delta extraction, then the three earlier cards' providers for the two bounds.
Readable type map: `K` is the target, `s k` the approximating family, `C` the ambient compact set, and `radius k` the approximation scale.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L19).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {f : Complex → Complex} {K C : Set.{0} Complex}, IsCompact.{0} K → Set.Nonempty.{0} K → IsCompact.{0} C → ContinuousOn.{0, 0} f C → ∀ {s : Nat → Set.{0} Complex}, (∀ (k : Nat), IsCompact.{0} (s k)) → (∀ (k : Nat), LE.le.{0} K (s k)) → (∀ (k : Nat), LE.le.{0} (s k) C) → ∀ {radius : Nat → Real}, Filter.Tendsto.{0, 0} radius Filter.atTop.{0} (nhds.{0} (OfNat.ofNat.{0} 0)) → (∀ (k : Nat) (z : Complex), Membership.mem.{0, 0} (s k) z → Exists.{1} fun w => And (Membership.mem.{0, 0} K w) (LE.le.{0} (Norm.norm.{0} (HSub.hSub.{0, 0, 0} z w)) (radius k))) → Filter.Tendsto.{0, 0} (fun k => CrouzeixConjecture.maxFunctionModulusOnSet (s k) f) Filter.atTop.{0} (nhds.{0} (CrouzeixConjecture.maxFunctionModulusOnSet K f))`.
Type SHA-256: `7b798048701c82c441890f7a4351d452b93a459e868f17c3b1c04efcbcefacf9`.
Direct maintained dependencies: `CrouzeixConjecture.tendsto_maxFunctionModulusOnSet_of_outerApproximation`, which calls `maxFunctionModulusOnSet_mono`, `exists_maxFunctionModulusOnSet` and `norm_function_le_maxFunctionModulusOnSet`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: compact nonempty `K`, compact `C` with `f` continuous on it, compact `s k` between them, and a vanishing approximation radius.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:7b798048701c82c441890f7a4351d452b93a459e868f17c3b1c04efcbcefacf9`.

#### Exercises and solutions

No exercise restates this card. Its hypotheses are the chapter's heaviest, and
the exercises below work on the four simpler cards.

## Worked examples

**A maximum attained on a whole circle.** For `f(z)=z^2` on the closed unit disk,
`M = 1` and every boundary point attains it. Attainment does not mean a unique
maximizer, and no card claims one.

**Where the supremum is not attained.** On the *open* unit disk the supremum of
`|z^2|` is still `1`, and no point of the set reaches it. Nothing in the chapter
covers this case: every theorem card assumes compactness, and this example is the
reason. It is not compiled.

**The empty set.** `M(∅,f)=0` for every `f`. That value is real and compiled, as
exercise E05, but it is a convention of Mathlib's `sSup`, not a maximum of
anything. A reader who forgets the nonemptiness hypothesis will get a bound of
`0` that means nothing.

**Nested disks.** With `K = D_1` and `s k = D_{1+1/k}`, the approximation
hypotheses of CFT-13-006 hold with `radius k = 1/k`, and the maxima of `f(z)=z^2`
are `(1+1/k)^2`, converging to `1`. The chapter compiles the general statement,
not this instance.

## ML bridge

**Mathematical object.** A worst-case value over a region, together with its
existence, sign, monotonicity, and stability under enlarging the region slightly.

**ML counterpart.** The inner maximization of a robust or adversarial objective
over a constraint set, and the practice of relaxing that set to make the inner
problem tractable.

**Exact transfer.** On a compact region a continuous objective has a genuine worst
case; widening the region cannot lower it; and shrinking a relaxation back onto
the region makes the values converge, provided everything stays inside one
compact ambient set.

**Non-transfer.** No rate of convergence is available, so this gives no schedule
for tightening a relaxation. Nothing here says a worst case is *findable*, and in
practice the inner maximization is the hard part. Nothing addresses non-compact
parameter regions, which is the usual situation for unconstrained training.

**Diagnostic.** An inner maximization whose reported value keeps drifting upward
without settling is better explained by a non-compact region than by optimizer
tuning. Compactness is the load-bearing hypothesis in CFT-13-002 and CFT-13-003
and in the convergence card; it is not needed for the definition, for the sign, or
for the constant-function calculation of E02.

## Lean translation

`CrouzeixTextbook.Part03.Chapter13` exposes six cards and six checked exercise
solutions. CFT-13-001 is a `definition` row naming
`CrouzeixConjecture.maxFunctionModulusOnSet`; the other five are
`reexported-proof` rows over `CrouzeixConjecture.FunctionMaximum`, and all five
providers are genuine multi-step proofs — none is `rfl` and none is a single
library application.

The providers form a chain: attainment is proved from Mathlib's
`IsCompact.exists_isMaxOn`, the pointwise bound re-runs that construction rather
than citing attainment, nonnegativity and monotonicity both call attainment, and
the convergence theorem calls three of the four. That structure is narrated in
each card's Proof rather than left for the reader to reconstruct.

The compactness results the chapter's title suggests come from Mathlib and are
used, not proved here: `IsCompact.exists_isMaxOn` and
`IsCompact.uniformContinuousOn_of_continuous`.

Not compiled, and not claimed: Cauchy sequences, completeness, equivalence of
norms in finite dimension, continuity of a finite-dimensional linear map,
compactness of the unit sphere, non-compactness of the infinite-dimensional unit
ball, any rate for the convergence of CFT-13-006, any statement about a supremum
that is not attained, boundedness of the image itself, and the characterization of
when the maximum equals zero. The earlier sketch of this chapter set six
exercises, on Cauchy sequences, norm equivalence on `ℂⁿ`, continuity of linear
maps, compactness of the sphere, the `ℓ²` counterexample, and a reading of
monotonicity; the first five are withdrawn and the sixth survives as E06. The
exercises below are stated on the compiled material.

## Exercises with complete solutions

### CFT-13-E01 — unfolding the definition {#exercise-cft-13-e01}

Unfold the maximum-modulus functional, and record the one immediate consequence
of nonemptiness.

#### Complete written solution

By definition `M(s,f)` is the supremum of the image of `s` under `z ↦ ‖f z‖`, so
the first conjunct holds by `rfl` — it is a definitional unfolding and nothing
more. The second conjunct is that this image is nonempty when `s` is: the image
of a nonempty set under any function is nonempty, which is `Set.Nonempty.image`.

The second half is what makes the first useful. A supremum over an empty set is
the junk value of exercise E05; the nonemptiness hypothesis carried by every
theorem card is exactly the condition that rules it out, and this is where it
first bites.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter13.exercise_01_solution`.
Formal mode: `proved-here`.
Provider boundary: `rfl` for the unfolding and Mathlib's `Set.Nonempty.image` for the second conjunct; no maintained Crouzeix provider and no card cited.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L24).
Type SHA-256: `fa7e532c0399011dbd8da1b1a586d35dd6aff087052d1c0130e75f547003b0df`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:fa7e532c0399011dbd8da1b1a586d35dd6aff087052d1c0130e75f547003b0df`.

### CFT-13-E02 — the maximum of a constant {#exercise-cft-13-e02}

Compute the maximum modulus of a constant function on a nonempty set.

#### Complete written solution

For `f ≡ c` the image of `s` under `z ↦ ‖f z‖` is the single point `{‖c‖}`,
provided `s` is nonempty — this is Mathlib's `Set.Nonempty.image_const`. The
supremum of a singleton is its element, by `csSup_singleton`. So
`M(s, fun _ => c) = ‖c‖`.

No compactness is needed: a singleton is bounded and its supremum is attained
trivially, so the calculation goes through for any nonempty `s`, compact or not.
That is worth noticing, because it shows compactness in the other cards is doing
work about *varying* functions, not about suprema as such.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter13.exercise_02_solution`.
Formal mode: `proved-here`.
Provider boundary: Mathlib's `Set.Nonempty.image_const` and `csSup_singleton`; no compactness hypothesis and no card cited.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L30).
Type SHA-256: `61ba71e5d3cdb5ab8ae68f581d51df50b1503e7c4c239c1d8e7448938d5ad2b2`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:61ba71e5d3cdb5ab8ae68f581d51df50b1503e7c4c239c1d8e7448938d5ad2b2`.

### CFT-13-E03 — the maximum is greatest {#exercise-cft-13-e03}

Prove that the maximum modulus is the greatest element of the image, combining
attainment with the pointwise bound.

#### Complete written solution

`IsGreatest S m` means `m ∈ S` and `m` bounds `S`. Take `S` to be the image of
`s` under `z ↦ ‖f z‖` and `m = M(s,f)`.

Membership is attainment: by CFT-13-002 there is a `z ∈ s` with
`‖f z‖ = M(s,f)`, and that exhibits `M(s,f)` as an element of the image.

The bound is CFT-13-003: an arbitrary element of the image is `‖f w‖` for some
`w ∈ s`, and that card bounds it by `M(s,f)`.

The checked solution runs exactly this, destructuring the attainment witness and
then `rintro`-ing the image element to expose its preimage point. It is the
converse direction to the providers' own internal use of `IsGreatest`: they build
one to *define* the supremum, and this exercise rebuilds it from the two public
cards.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter13.exercise_03_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.exists_maxFunctionModulusOnSet` and `norm_function_le_maxFunctionModulusOnSet`, the providers of CFT-13-002 and CFT-13-003.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L37).
Type SHA-256: `9726762bd3e1170f6077edc6c0da361f7295e293b59e92ad256d150f879ccad7`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:9726762bd3e1170f6077edc6c0da361f7295e293b59e92ad256d150f879ccad7`.

### CFT-13-E04 — nonnegativity from attainment {#exercise-cft-13-e04}

Derive the sign of the maximum from the fact that it is attained, stating the
attainment explicitly rather than citing the nonnegativity card.

#### Complete written solution

By CFT-13-002 there is a `z ∈ s` with `‖f z‖ = M(s,f)`. Rewriting the goal
backwards along that equality reduces `0 ≤ M(s,f)` to `0 ≤ ‖f z‖`, which is the
nonnegativity of a norm.

The exercise states both conclusions — the attainment witness and the sign — as
conjuncts, so that the dependency is visible in the type rather than buried in a
proof. Be clear about what that is and is not: each conjunct is verbatim a public
card's conclusion, CFT-13-002's and CFT-13-004's, under those cards' own
hypotheses, so the exercise adds the pairing and the derivation, not a new
statement. What it does not do is call CFT-13-004: it rebuilds the sign from
attainment and `norm_nonneg` directly, which is the point of the exercise.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter13.exercise_04_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.exists_maxFunctionModulusOnSet` and Mathlib's `norm_nonneg`; it does not call CFT-13-004's provider.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L46).
Type SHA-256: `37ba6b2f9a3ce0a791d9b4cf34f679dc8794323db67ac1a239d4f9ae694368db`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:37ba6b2f9a3ce0a791d9b4cf34f679dc8794323db67ac1a239d4f9ae694368db`.

### CFT-13-E05 — the empty set {#exercise-cft-13-e05}

Compute the functional on the empty set and explain what the value does and does
not mean.

#### Complete written solution

The image of the empty set is empty, and Mathlib defines the real supremum of the
empty set to be `0`. So `M(∅,f)=0` for every `f`, by `Set.image_empty` followed by
`Real.sSup_empty`.

The value is a convention, not a maximum. No point attains it, and `‖f‖` may be
arbitrarily large elsewhere. This is why `s.Nonempty` is a hypothesis of all five
theorem cards rather than being folded into CFT-13-001: the definition is total
and returns something on the empty set, so the theorems have to exclude it
explicitly. A reader who drops the hypothesis gets the bound `0`, which is true of
the definition and says nothing about `f`.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter13.exercise_05_solution`.
Formal mode: `proved-here`.
Provider boundary: Mathlib's `Set.image_empty` and `Real.sSup_empty`; a two-rewrite computation with no Crouzeix provider.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L56).
Type SHA-256: `3793c7327336f5eb4415269140fed79f458d3c7708275358bdb4cad2e4af152e`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:3793c7327336f5eb4415269140fed79f458d3c7708275358bdb4cad2e4af152e`.

### CFT-13-E06 — enlarging the domain {#exercise-cft-13-e06}

Formalize why enlarging a compact domain cannot decrease the maximum modulus, and
record the sign of the smaller maximum alongside it.

#### Complete written solution

Monotonicity is CFT-13-005: attaining the smaller maximum at a point of `s` and
bounding that point within `t` gives `M(s,f) ≤ M(t,f)`. The sign of the smaller
maximum is CFT-13-004, applied to `s` with continuity restricted along the
inclusion.

Stated together, the two give the two-sided information a later estimate wants:
the smaller maximum is a nonnegative real bounded above by the larger one, so it
can be chained into a product or a square root without a separate sign argument.
The checked solution is the pair of card applications, with `ContinuousOn.mono`
supplying the restricted continuity for the second.

This is the one exercise the earlier sketch of this chapter set that survives
unchanged in intent.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part03.Exercises.Chapter13.exercise_06_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.maxFunctionModulusOnSet_mono` and `maxFunctionModulusOnSet_nonneg`, the providers of CFT-13-005 and CFT-13-004; the exercise adds the pairing, not a proof.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part03/Chapter13.lean#L61).
Type SHA-256: `4181055ad0dc4d82f5a4bd5e3e54fa042ee37a4182a10e03f1c65592010c4438`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:4181055ad0dc4d82f5a4bd5e3e54fa042ee37a4182a10e03f1c65592010c4438`.

## Synthesis and forward dependencies

The chapter contributes one functional and five properties: the maximum modulus
over a set exists when the set is compact and nonempty, dominates the function
pointwise, is nonnegative, grows with the set, and is stable under quantitative
outer approximation.

Chapter 14 turns to sequences and series of operators, where the same
compactness-and-supremum pattern recurs for operator norms. The convergence card
here is the one consumed furthest away: Part V uses it to replace an awkward
convex boundary by a family of smooth outer approximations without changing the
estimate in the limit.
