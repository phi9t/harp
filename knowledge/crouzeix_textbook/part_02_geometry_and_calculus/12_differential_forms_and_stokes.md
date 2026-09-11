---
id: cft-chapter-12-differential-forms-and-stokes
title: Differential forms and Stokes
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-10
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 12_differential_forms_and_stokes.md
chapter: 12
part: 2
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 12: Differential forms and Stokes

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/11_differentiation_as_linear_approximation|Chapter 11 — Differentiation as linear approximation]]
Next: [[knowledge/crouzeix_textbook/part_03_analysis_and_complex_functions/13_metric_and_normed_spaces|Chapter 13 — Metric and normed spaces]]

## Opening problem

Chapter 11 attached a linear map to each point. Integration needs one more
ingredient: a rule for turning a pair of tangent directions into a signed number,
so that "area" has a sign and "boundary" has a direction. That rule is the area
form, and getting its sign conventions right is the whole content of the
orientation bookkeeping in Stokes' theorem.

This chapter is where the Crouzeix argument's contour work is grounded. Part V
integrates over the boundary of a convex region and needs three things to be
true at once: the boundary tangent points the correct way around, the tangent has
the arclength normalization, and the parametrization is differentiable enough to
change variables. Those three are exactly what is compiled here.

## What this chapter compiles, and what it does not

**Stokes' theorem is not compiled in this book.** Neither is the exterior
derivative, the pullback of a differential form, the wedge product as a general
construction, or the cancellation of interior edges in a triangulation. The
chapter title names the destination, not the contents.

What is compiled is the planar orientation infrastructure the theorem rests on,
in the concrete model where Part V needs it — the complex plane, with the area
form
$$
\omega(z,w)=\operatorname{Im}(\bar z w)
$$
that a coordinate treatment would write `dx \wedge dy`. Six cards give the
dictionary between that form and the real inner product, the positivity and
normalization of a radial boundary tangent, and the differentiability of the
polar parametrization. The earlier sketch of this chapter promised Stokes on a
rectangle and a triangulation argument; those promises are withdrawn here rather
than left standing, and the exercises have been restated on the compiled
material.

Five of the six cards are `reexported-proof` rows: the proof lives in
`CrouzeixConjecture`, the maintained namespace, and the card names it as its
underlying declaration. None of those five claims a second proof. The exception is
CFT-12-001, which is `proved-here`, for a reason its own card gives: the
maintained provider for it is an eta-alias and cannot serve as an underlying
declaration.

## Conceptual model

On `\mathbb C` regarded as the oriented plane, two bilinear forms carry all the
geometry. The real inner product
$$
\langle z,w\rangle_{\mathbb R}=\operatorname{Re}(w\bar z)
$$
measures projection; the area form `\omega(z,w)` measures signed area. They are
the same object seen through a quarter turn: multiplying one argument by `i`
exchanges them, up to sign. That single fact is the chapter's engine.

A unit vector `n` and its quarter turn `i n` form an oriented orthonormal frame.
Any vector orthogonal to `n` is therefore a real multiple of `i n`, and the
multiple is positive exactly when the vector points in the positively oriented
direction. Applied to the boundary of a convex region, with `n` the outward
normal, this is the statement that the boundary is traversed counterclockwise.

## Running example: the circle of radius `r` about `c`

Take the polar parametrization `(t,s) \mapsto c + s e^{it}` of the plane about a
centre `c`. Fixing `s = r` traces the circle of radius `r`; fixing `t` and moving
`s` runs outward along a ray. At the boundary point `c + r e^{it}` the outward
unit normal is `n = e^{it}` and the counterclockwise tangent is `i e^{it}`,
which is `i n`. The area form of the two is
$$
\omega(n, i n)=\langle n,n\rangle_{\mathbb R}=1>0 ,
$$
the first equality by the quarter-turn card below and the second by `‖n‖ = 1`,
which no card in this chapter compiles. The sign is what says the frame is
positively oriented. The chapter's radial cards are this computation without the circle:
they assume only that the tangent is orthogonal to a unit outward normal and has
a positive radial component, and they recover both the sign and the arclength
normalization.

## Formal development

### CFT-12-001 — the real inner product in complex coordinates {#cft-12-001}

#### Purpose

Fix the coordinate reading of the real pairing, so that later expansions have a
definite form to expand into. The real inner product is what the area form is
measured against, and every radial computation below reduces to it.

#### Statement

For complex `z` and `w`, the real inner product in real coordinates is
`⟪z,w⟫_ℝ = z.re · w.re + z.im · w.im`.

#### Hypothesis ledger

None. The identity holds for all complex `z` and `w`.

#### Proof roadmap

It is the definition of the real inner product on `ℂ`, exposed.

#### Proof

Mathlib defines the real inner product on `ℂ` as `⟪z,w⟫_ℝ = (w\,\bar z).re`.
Expanding that real part, `(w\bar z).re = w.re\,z.re - w.im\,(-z.im)`, since
conjugation negates the imaginary part; collecting terms gives
`z.re\,w.re + z.im\,w.im`, the Euclidean pairing of the coordinate pairs.

**Why this is proved here rather than re-exported.** The maintained provider
`complexRealInner_eq_re_mul_conj` states the conjugate form and its proof is the
single application `Complex.inner z w` — an exact eta-alias, which the receipt
classifies as `direct-alias`. A `reexported-proof` row needs a theorem-kind
underlying declaration, so that provider cannot serve as one. The card is
therefore proved locally, in coordinates, which is also the form its title names.
The checked proof rewrites with the provider and with `Complex.mul_re`, simplifies
the conjugate components, and closes by `ring`.

#### Worked instance

`⟪1, i⟫_ℝ = (i · 1).re = 0`, so the standard frame is orthogonal. Exercise E02
checks this together with the matching area-form value.

#### Boundary case

The *complex* inner product on `ℂ` is conjugate-linear in the first argument, and
the opposite convention conjugates the other one. That choice does not matter
here: taking the real part destroys the difference, since
`\operatorname{Re}(w\bar z)=\operatorname{Re}(z\bar w)`, and the coordinate form
above is visibly symmetric in `z` and `w`. Nothing in this chapter's orientation
conclusions depends on it. What would reverse those conclusions is reversing the
*orientation* — the area form is supplied by `Complex.orientation` and is not
derived from the inner-product convention at all.

#### Historical context

The two conventions for a complex inner product have coexisted since the 1930s,
physics generally conjugating the second argument and mathematics the first. The
ambiguity is real for the complex pairing and vanishes for its real part, which is
the quantity this chapter uses throughout.
Source boundary: Mathlib 4.32.1 complex inner-product declarations.
Review status: convention and provider reviewed together.

#### ML analogy

Mathematical object: the real inner product underlying a complex vector space.
ML counterpart: the real dot product used when a complex-valued layer is stored as interleaved real pairs.
Exact transfer: the real part of `w · conj z` is the dot product of the real coordinate vectors.
Non-transfer: nothing here says a complex parameterization and its real reading have the same gradients; that requires the Wirtinger bookkeeping this book does not develop.
Diagnostic: a real-part pairing that disagrees between a complex and an interleaved-real implementation is an indexing error, not a conjugation convention — the real inner product is the same under either convention.

#### Pedagogical prerequisites

Chapter 7's real inner product and complex conjugation.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.real_inner_complex_coordinates`.
Formal mode: `proved-here`.

Substantive provider: `none` as an underlying declaration; the maintained `complexRealInner_eq_re_mul_conj` is an eta-alias of `Complex.inner` and is used as a rewrite step, not as a provider.
Readable type map: `z` and `w` are the two complex vectors, read through their real and imaginary parts.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L14).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ (z w : Complex), Eq.{1} (Inner.inner.{0, 0} Real z w) (HAdd.hAdd.{0, 0, 0} (HMul.hMul.{0, 0, 0} z.re w.re) (HMul.hMul.{0, 0, 0} z.im w.im))`.
Type SHA-256: `bcd9dc783e3441cb9fd4bb908dc62253885b9ab13d3cb5729b2a9ff75fa29be0`.
Direct maintained dependencies: `CrouzeixConjecture.complexRealInner_eq_re_mul_conj`, used as a rewrite.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: none.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:bcd9dc783e3441cb9fd4bb908dc62253885b9ab13d3cb5729b2a9ff75fa29be0`.

#### Exercises and solutions

CFT-12-E02 evaluates this pairing and the area form on the standard frame.

### CFT-12-002 — the quarter turn exchanges the two forms {#cft-12-002}

#### Purpose

This is the engine. Everything oriented in the chapter is the inner product seen
through a rotation by a right angle.

#### Statement

For complex `z` and `w`, `\omega(z, i w) = ⟪z,w⟫_ℝ`.

#### Hypothesis ledger

None; no unit-norm assumption is needed here. The unit hypotheses appear only in
the radial cards, where they are used for the Pythagorean identity.

#### Proof roadmap

Multiplication by `i` is the right-angle rotation of the oriented plane, so the
statement is Mathlib's area-form identity transported along that identification.

#### Proof

On the oriented plane, the rotation `J` by a positive right angle satisfies
`\omega(z, Jw) = ⟪z,w⟫`. In `\mathbb C` that rotation is multiplication by `i`.

**What is checked.** The provider rewrites `Complex.rightAngleRotation` into
multiplication by `i` and then applies
`Complex.orientation.areaForm_rightAngleRotation_right`. It is one library
application after one unfolding; the geometric statement that multiplication by
`i` *is* the right-angle rotation is Mathlib's, not this chapter's.

#### Worked instance

`\omega(1, i) = ⟪1,1⟫_ℝ = 1`. The standard frame has unit positive area, which
is the normalization every later sign is measured against.

#### Boundary case

The companion identity puts the rotation on the first argument of an inner
product instead: `⟪i z, w⟫_ℝ = \omega(z,w)`, with no sign change. A third
member of the dictionary puts it on the second argument and does change sign:
`⟪z, i w⟫_ℝ = -\omega(z,w)`. The three are different statements. E01 records the
first two; the third is the one CFT-12-003's orthogonality step actually consumes,
and it is named there rather than carried by a card.

#### Historical context

Identifying the plane's area form with its inner product through a right-angle
rotation is the two-dimensional shadow of the Hodge star; in one complex
dimension the star on one-forms is exactly multiplication by `i`.
Source boundary: Mathlib 4.32.1 two-dimensional orientation declarations.
Review status: both sidedness variants reviewed; only the right-hand one is this card.

#### ML analogy

Mathematical object: a rotation that converts a symmetric pairing into an antisymmetric one.
ML counterpart: the `[[0,-1],[1,0]]` block used to rotate two-dimensional features.
Exact transfer: the same matrix converts a dot product into a signed area.
Non-transfer: in dimensions above two there is no single rotation with this property, and the analogy stops.
Diagnostic: an area that comes out negative where it should be positive indicates the rotation was applied to the wrong argument.

#### Pedagogical prerequisites

CFT-12-001 and the oriented area form.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.area_form_quarter_turn`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.complexAreaForm_I_mul_right`.

Substantive provider: `CrouzeixConjecture.complexAreaForm_I_mul_right`, itself one application of Mathlib's `areaForm_rightAngleRotation_right` after unfolding `Complex.rightAngleRotation`.
Readable type map: `z` and `w` are the two vectors; `i w` is the quarter turn of the second.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L20).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ (z w : Complex), Eq.{1} (DFunLike.coe.{1, 1, 1} (DFunLike.coe.{1, 1, 1} (Orientation.areaForm.{0} Complex.orientation) z) (HMul.hMul.{0, 0, 0} Complex.I w)) (Inner.inner.{0, 0} Real z w)`.
Type SHA-256: `d352bcd37e0f0cd0d277be348b019272c42954c4f73c766861824e6876d27fa0`.
Direct maintained dependencies: `CrouzeixConjecture.complexAreaForm_I_mul_right`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: none.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:d352bcd37e0f0cd0d277be348b019272c42954c4f73c766861824e6876d27fa0`.

#### Exercises and solutions

CFT-12-E01 records this identity together with its left-hand companion, which
this card does not carry.

### CFT-12-003 — a radial tangent is positively oriented {#cft-12-003}

#### Purpose

This is the sign that makes a contour integral come out with the right sign. It
says a boundary tangent that is orthogonal to the outward normal and has a
positive radial component has positive oriented area against that normal.

#### Definitions and notation

`u` is a unit direction, `n` a unit outward normal, and `v` a tangent written in
the radial frame as `v = (a + i\rho)u` with `\rho > 0`. The hypothesis
`⟪n,u⟫_ℝ > 0` says the ray direction points outward.

#### Statement

Under those hypotheses, `\omega(n,v) > 0`.

#### Hypothesis ledger

All six hypotheses are used. `‖u‖ = 1` and `‖n‖ = 1` feed the Pythagorean
identity; `\rho > 0` and `⟪n,u⟫_ℝ > 0` are the two positivity inputs;
orthogonality `⟪n,v⟫_ℝ = 0` is what eliminates the unknown `a`; and the radial
form of `v` is what makes the expansion available. Drop `⟪n,u⟫_ℝ > 0` and the
conclusion reverses for an inward-pointing ray.

#### Proof roadmap

Expand `v` in the frame `(u, iu)`, turn orthogonality into a linear relation
between `a` and `\rho`, expand the area form in the same frame, and combine the
two with the unit-vector Pythagorean identity.

#### Proof

Write `v = a\,u + \rho\,(iu)`, which is the radial form rearranged.

Orthogonality gives the first relation. Expanding `⟪n,v⟫_ℝ = 0` by additivity and
real homogeneity leaves a term `⟪n, iu⟫_ℝ`, with the rotation on the *second*
argument. Converting that needs a third member of the quarter-turn dictionary,
$$
\langle z, i w\rangle_{\mathbb R}=-\,\omega(z,w),
$$
which is `CrouzeixConjecture.complexRealInner_I_mul_right` and is where the minus
sign below comes from. Neither CFT-12-002 nor its companion in E01 is the identity
used here: CFT-12-002 puts the rotation inside an area form, and the companion
puts it on the *first* argument of an inner product. Applying the right one yields
$$
a\,\langle n,u\rangle_{\mathbb R}-\rho\,\omega(n,u)=0 .
$$

The area form expands in the same frame, this time using the right-hand
quarter-turn identity of CFT-12-002:
$$
\omega(n,v)=a\,\omega(n,u)+\rho\,\langle n,u\rangle_{\mathbb R}.
$$

Because `n` and `u` are unit vectors, Mathlib's identity
`⟪n,u⟫^2 + \omega(n,u)^2 = ‖n‖^2‖u‖^2` collapses to
`⟪n,u⟫_{\mathbb R}^2+\omega(n,u)^2=1`.

Now multiply the area expansion by `⟪n,u⟫_ℝ`. The checked proof does this as a
single `ring` rearrangement into
$$
\rho\bigl(\langle n,u\rangle^2+\omega(n,u)^2\bigr)
+\omega(n,u)\bigl(a\langle n,u\rangle-\rho\,\omega(n,u)\bigr),
$$
whose second bracket is zero by orthogonality and whose first is `\rho` by the
Pythagorean identity. So `\omega(n,v)\,⟪n,u⟫_ℝ = \rho`, hence
`\omega(n,v) = \rho/⟪n,u⟫_ℝ`, positive as a quotient of positives.

The elimination of `a` is the point: the unknown radial coefficient never needs
to be computed, only cancelled.

#### Worked instance

On the circle of radius `r` about `c`, take `u = n = e^{it}` and `v = i r e^{it}`,
so `a = 0` and `\rho = r`. Then `⟪n,u⟫_ℝ = 1` and the formula gives
`\omega(n,v) = r`, positive and equal to the speed.

#### Boundary case

With `\rho < 0` the tangent runs clockwise and the area form is negative; with
`⟪n,u⟫_ℝ < 0` the ray points inward and the same reversal happens. Both are why
the two positivity hypotheses cannot be dropped.

#### Historical context

The convention that a positively oriented boundary keeps the region on the left
is Stokes' orientation rule; stating it as a strict inequality on an area form is
what makes it checkable rather than pictorial.
Source boundary: Mathlib 4.32.1 orientation declarations and the registered Crouzeix geometry packet.
Review status: all six hypotheses traced to their uses in the checked proof.

#### ML analogy

Mathematical object: a sign condition certifying that a boundary traversal is counterclockwise.
ML counterpart: the winding or orientation test in a geometric routine over polygonal regions.
Exact transfer: the sign of the area form of normal against tangent decides the traversal direction exactly.
Non-transfer: nothing here handles a self-intersecting boundary or a numerically computed normal that is only approximately unit.
Diagnostic: an integral that comes out with the wrong overall sign usually has the normal and tangent in the wrong order, which this card's argument order fixes.

#### Pedagogical prerequisites

CFT-12-001, CFT-12-002, and the Pythagorean identity for a unit pair.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.oriented_radial_tangent_positive`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.complexAreaForm_radialTangent_pos`.

Substantive provider: `CrouzeixConjecture.complexAreaForm_radialTangent_pos`, a genuine multi-step proof: frame expansion, orthogonality relation, area expansion, the unit Pythagorean identity, one `ring` rearrangement, then a positive quotient.
Readable type map: `u` is the ray direction, `n` the unit outward normal, `v` the tangent, `a` and `rho` its radial coordinates.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L22).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {a rho : Real} {u n v : Complex}, Eq.{1} v (HMul.hMul.{0, 0, 0} (HAdd.hAdd.{0, 0, 0} (Complex.ofReal a) (HMul.hMul.{0, 0, 0} Complex.I (Complex.ofReal rho))) u) → Eq.{1} (Norm.norm.{0} u) (OfNat.ofNat.{0} 1) → LT.lt.{0} (OfNat.ofNat.{0} 0) rho → Eq.{1} (Norm.norm.{0} n) (OfNat.ofNat.{0} 1) → Eq.{1} (Inner.inner.{0, 0} Real n v) (OfNat.ofNat.{0} 0) → LT.lt.{0} (OfNat.ofNat.{0} 0) (Inner.inner.{0, 0} Real n u) → LT.lt.{0} (OfNat.ofNat.{0} 0) (DFunLike.coe.{1, 1, 1} (DFunLike.coe.{1, 1, 1} (Orientation.areaForm.{0} Complex.orientation) n) v)`.
Type SHA-256: `73f2f5bbf40269700259b1e09984c373263f89264e1f45f3f8885a2a1e4cf0fc`.
Direct maintained dependencies: `CrouzeixConjecture.complexAreaForm_radialTangent_pos`, which itself uses `complexRealInner_I_mul_right` and `complexAreaForm_I_mul_right`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: unit `u` and `n`, positive `rho`, positive radial component, orthogonality, and the radial form of `v`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:73f2f5bbf40269700259b1e09984c373263f89264e1f45f3f8885a2a1e4cf0fc`.

#### Exercises and solutions

CFT-12-E06 states this conclusion together with CFT-12-004's under the same
hypotheses, which is the pair Part V consumes.

### CFT-12-004 — the tangent is the normal turned and scaled {#cft-12-004}

#### Purpose

Upgrade the sign of CFT-12-003 to an identity. The boundary tangent is not merely
positively oriented; it is exactly `i n` scaled by its own length, which is the
arclength normalization every change of variables needs.

#### Statement

Under the hypotheses of CFT-12-003, `v = i\,n\,‖v‖`.

#### Hypothesis ledger

The same six. Unit `n` is what makes `(n, in)` an orthonormal frame; the
positivity hypotheses are what fix the sign of the scaling.

#### Proof roadmap

Decompose `v` in the frame `(n, in)`, identify the coefficient as positive by
CFT-12-003, then recognize that coefficient as `‖v‖`.

#### Proof

Because `‖n‖ = 1` and `⟪n,v⟫_ℝ = 0`, the frame decomposition gives
$$
v = i\,n\,\operatorname{Im}(v/n),
$$
the coefficient being deliberately written as the imaginary part of the quotient
rather than as an area form, so that it matches what a radial parametrization
computes. The provider's proof of that decomposition runs through
`normSq n = 1`, rewrites `v` as `v\,\overline n\,n`, and observes that
`v\overline n` is purely imaginary because its real part is `⟪n,v⟫_ℝ = 0`.

That coefficient is positive: `\operatorname{Im}(v/n)` equals `\omega(n,v)`
when `‖n‖ = 1`, and CFT-12-003 makes the latter positive.

Finally the coefficient is `‖v‖`. Taking norms in the decomposition and using
`‖i‖ = ‖n‖ = 1` gives `‖v‖ = |\operatorname{Im}(v/n)|`, and positivity removes
the absolute value. Substituting back gives `v = i\,n\,‖v‖`.

#### Worked instance

On the circle, `v = i r e^{it}` and `n = e^{it}` give `‖v‖ = r` and the identity
reads `i r e^{it} = i e^{it} r`.

#### Boundary case

The identity pins both direction and length. Without the positivity hypotheses
only `v = \pm i n ‖v‖` survives, and the sign is exactly what CFT-12-003 supplies.

#### Historical context

Writing a boundary tangent as `i n` times speed is the planar case of the general
rule that an oriented boundary's tangent is the interior normal rotated; the
complex notation makes the rotation a multiplication.
Source boundary: Mathlib 4.32.1 orientation declarations and the registered Crouzeix geometry packet.
Review status: decomposition, positivity and normalization reviewed as three separate steps.

#### ML analogy

Mathematical object: the exact arclength-normalized boundary tangent.
ML counterpart: a unit tangent computed for a parametrized curve before a line integral.
Exact transfer: the tangent equals the rotated normal times the exact speed.
Non-transfer: a numerically differentiated tangent is neither exactly unit nor exactly orthogonal, and nothing here bounds the discrepancy.
Diagnostic: a tangent whose norm drifts from the parametric speed indicates the normal was not normalized.

#### Pedagogical prerequisites

CFT-12-003 and the orthonormal frame `(n, in)`.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.oriented_tangent_formula`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.radialTangent_eq_I_mul_normal_mul_norm`.

Substantive provider: `CrouzeixConjecture.radialTangent_eq_I_mul_normal_mul_norm`, which composes the frame decomposition, CFT-12-003's positivity through `im_div_radialTangent_pos`, and a norm computation.
Readable type map: `v` is the tangent, `n` the unit outward normal, and `‖v‖` the parametric speed.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L24).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {a rho : Real} {u n v : Complex}, Eq.{1} v (HMul.hMul.{0, 0, 0} (HAdd.hAdd.{0, 0, 0} (Complex.ofReal a) (HMul.hMul.{0, 0, 0} Complex.I (Complex.ofReal rho))) u) → Eq.{1} (Norm.norm.{0} u) (OfNat.ofNat.{0} 1) → LT.lt.{0} (OfNat.ofNat.{0} 0) rho → Eq.{1} (Norm.norm.{0} n) (OfNat.ofNat.{0} 1) → Eq.{1} (Inner.inner.{0, 0} Real n v) (OfNat.ofNat.{0} 0) → LT.lt.{0} (OfNat.ofNat.{0} 0) (Inner.inner.{0, 0} Real n u) → Eq.{1} v (HMul.hMul.{0, 0, 0} (HMul.hMul.{0, 0, 0} Complex.I n) (Complex.ofReal (Norm.norm.{0} v)))`.
Type SHA-256: `71497b4822c51a78398ff46b4e6c77996da483dd69639fb9aa7d14dde5e60464`.
Direct maintained dependencies: `CrouzeixConjecture.radialTangent_eq_I_mul_normal_mul_norm`, which uses `eq_I_mul_mul_im_div_of_norm_eq_one_of_realInner_eq_zero` and `im_div_radialTangent_pos`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: the same six hypotheses as CFT-12-003.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:71497b4822c51a78398ff46b4e6c77996da483dd69639fb9aa7d14dde5e60464`.

#### Exercises and solutions

CFT-12-E06 pairs this identity with the positivity it rests on.

### CFT-12-005 — the polar parametrization is continuously differentiable {#cft-12-005}

#### Purpose

A change of variables needs the parametrization to be `C^1`. This card supplies
exactly that much regularity for the polar map, and no more.

#### Definitions and notation

`parallelRadialDirection t = e^{it}` is the unit direction at angle `t`, and
`parallelRadialParameterPoint c (t,s) = c + s\,e^{it}` is the polar point map
about the centre `c`.

#### Statement

For every centre `c`, the map `(t,s) \mapsto c + s\,e^{it}` is `C^1` on
`\mathbb R^2`.

#### Hypothesis ledger

None beyond the centre being a fixed complex number. In particular no boundary
equation has been solved yet: this is the parametrization *before* the radius is
determined, which is why it is unconditionally smooth.

#### Proof roadmap

Build the map from coordinate projections by operations that preserve `C^1`.

#### Proof

The two coordinate inclusions `(t,s) \mapsto t` and `(t,s) \mapsto s`, composed
with the real-to-complex embedding, are `C^1`: the provider writes each as
`Complex.ofRealCLM.contDiff.comp` applied to `contDiff_fst` and `contDiff_snd`,
a continuous linear map composed with a projection.

The direction `(t,s) \mapsto e^{it}` is then `C^1` as the complex exponential of
a `C^1` function — the provider's `(ht.mul contDiff_const).cexp`, multiplying the
embedded `t` by the constant `i` before exponentiating.

Finally the point map is a constant plus a product of two `C^1` functions, which
is `contDiff_const.add (hs.mul hdirection)` after unfolding the definition.

`C^1` is all that is claimed. The map is in fact smooth, and the card does not
say so, because `C^1` is what the change of variables consumes.

#### Worked instance

At `c = 0` the map is the usual polar chart, and its derivative at `(t,s)` sends
`(\delta t, \delta s)` to `i s e^{it}\delta t + e^{it}\delta s` — the angular
term tangent to the circle and the radial term along the ray.

#### Boundary case

The map is not injective: `s = 0` collapses every angle to the centre, and `t` is
only determined modulo `2\pi`. Smoothness is a statement about the map, not about
it being a chart.

#### Historical context

That polar coordinates are smooth away from nothing — but only a chart away from
the origin — is the standard caution; the compiled statement here is the smooth
half, which is the half a boundary integral uses.
Source boundary: Mathlib 4.32.1 exponential and `ContDiff` declarations.
Review status: the `C^1`-only claim reviewed against the stronger smoothness available.

#### ML analogy

Mathematical object: continuous differentiability of a polar reparametrization.
ML counterpart: the smoothness requirement on a coordinate transform before gradients are pushed through it.
Exact transfer: the transform is exactly `C^1`, so the chain rule of Chapter 11 applies to it.
Non-transfer: nothing here says the transform is invertible or numerically well conditioned near the origin, and it is neither.
Diagnostic: gradients that blow up near the centre reflect the chart degeneracy at `s = 0`, not a differentiability failure.

#### Pedagogical prerequisites

Chapter 11's derivative and the complex exponential.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.boundary_parameter_is_smooth`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.contDiff_one_parallelRadialParameterPoint`.

Substantive provider: `CrouzeixConjecture.contDiff_one_parallelRadialParameterPoint`, built from `contDiff_fst`, `contDiff_snd`, `Complex.ofRealCLM.contDiff`, `.cexp`, and the sum and product closure of `ContDiff`.
Readable type map: `c` is the centre; the pair `(t,s)` is angle and radius.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L26).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ (c : Complex), ContDiff.{0, 0, 0} Real (OfNat.ofNat.{0} 1) (CrouzeixConjecture.parallelRadialParameterPoint c)`.
Type SHA-256: `9e899523fc6feffedc4882e54856a4773afe294163cce87075ca13315a7f1c6b`.
Direct maintained dependencies: `CrouzeixConjecture.contDiff_one_parallelRadialParameterPoint`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: a fixed complex centre; no boundary equation solved.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:9e899523fc6feffedc4882e54856a4773afe294163cce87075ca13315a7f1c6b`.

#### Exercises and solutions

CFT-12-E03 computes the exact increment of the radial slice, which is the
statement the `C^1` regularity makes meaningful.

### CFT-12-006 — the radial slice has constant derivative {#cft-12-006}

#### Purpose

Fix the angle and move outward: the resulting curve is a straight ray traversed
at unit speed. This is the derivative that a radial integration uses.

#### Statement

For every centre `c`, angle `t` and radius `s`, the slice
`y \mapsto c + y\,e^{it}` has derivative `e^{it}` at `s`.

#### Hypothesis ledger

None. The derivative is independent of `s`, which is the content: the slice is
affine in `y`.

#### Proof roadmap

Differentiate the affine map directly.

#### Proof

The embedding `y \mapsto (y : \mathbb C)` has derivative `1`, being a continuous
linear map — `Complex.ofRealCLM.hasDerivAt` in the provider. Multiplying by the
constant `e^{it}` gives derivative `e^{it}` by `mul_const`, and adding the
constant `c` leaves it unchanged by `const_add`. The provider closes with
`simpa only [one_mul]`, absorbing the `1` from the embedding's derivative.

The derivative does not depend on `s`, so the slice is a ray of constant speed
`‖e^{it}‖ = 1`. That is exactly what makes `s` an arclength parameter along the
ray, matching the `‖v‖` normalization of CFT-12-004.

#### Worked instance

At `t = 0` the slice is `y \mapsto c + y`, with derivative `1` everywhere.

#### Boundary case

Constancy is special to the radial direction. The angular slice
`t \mapsto c + s e^{it}` has derivative `i s e^{it}`, which depends on both `s`
and `t`; this chapter does not compile that one.

#### Historical context

That a polar chart's radial curves are unit-speed rays is why polar integration
carries the Jacobian factor in the angular variable alone.
Source boundary: Mathlib 4.32.1 derivative declarations.
Review status: reviewed together with CFT-12-005, whose regularity it refines in one variable.

#### ML analogy

Mathematical object: the derivative of an affine slice of a parametrization.
ML counterpart: the exactly constant Jacobian of a residual branch that adds a fixed direction.
Exact transfer: the derivative is the direction itself, independent of the base point.
Non-transfer: nothing here covers the angular slice, whose derivative does vary, nor any composite of the two.
Diagnostic: a radial derivative that varies with the radius indicates the angle was not held fixed.

#### Pedagogical prerequisites

Chapter 11's linear base case and the complex exponential.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.boundary_slice_derivative`.
Formal mode: `reexported-proof`.
Underlying declaration: `CrouzeixConjecture.parallelRadialParameterPoint_slice_hasDerivAt`.

Substantive provider: `CrouzeixConjecture.parallelRadialParameterPoint_slice_hasDerivAt`, which is `Complex.ofRealCLM.hasDerivAt` followed by `mul_const` and `const_add`.
Readable type map: `c` is the centre, `t` the fixed angle, `s` the point of differentiation, and `e^{it}` the derivative.
Code: [Lean re-export](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L28).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ (c : Complex) (t s : Real), HasDerivAt.{0, 0} (fun y => CrouzeixConjecture.parallelRadialParameterPoint c (Prod.mk.{0, 0} t y)) (CrouzeixConjecture.parallelRadialDirection t) s`.
Type SHA-256: `b8d15a11ffebc20213434306ff59b3beb264f4cec85b501a95d6fb796936b71c`.
Direct maintained dependencies: `CrouzeixConjecture.parallelRadialParameterPoint_slice_hasDerivAt`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: none.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:b8d15a11ffebc20213434306ff59b3beb264f4cec85b501a95d6fb796936b71c`.

#### Exercises and solutions

CFT-12-E03 integrates this constant derivative into an exact increment over a
radius interval.

## Worked examples

**The unit frame.** `\omega(1,i) = ⟪1,1⟫_ℝ = 1` and `⟪1,i⟫_ℝ = 0`. The standard
frame is orthonormal and positively oriented, and every sign in the chapter is
measured against this normalization. Exercise E02 checks both values.

**A tangent that is not a tangent.** Take `n = 1` and `v = 1`. Then `v` is not
orthogonal to `n`, the hypotheses of CFT-12-003 fail, and indeed
`\omega(1,1) = 0` — neither positive nor of the form `i n ‖v‖`. The orthogonality
hypothesis is not decoration.

**Antisymmetry as edge cancellation.** `\omega(z,w) = -\omega(w,z)`, so a shared
edge traversed once in each direction contributes opposite signed areas. That is
the algebraic content behind the cancellation of interior edges in a
triangulation. The cancellation *argument* — that a triangulated region's
interior edges pair up — is not compiled in this book; only the antisymmetry it
relies on is, as Exercise E04.

**The radial increment.** With the angle fixed at `t`, moving the radius from
`s_0` to `s_1` moves the point by exactly `(s_1-s_0)e^{it}`. The derivative is
constant, so the increment is exact rather than approximate; no remainder term
appears. Exercise E03 compiles this, and it is the one place in the chapter where
a fundamental-theorem-style statement is available in closed form.

## ML bridge

**Mathematical object.** A signed area form on the oriented plane, together with
the orientation convention for the boundary of a region.

**ML counterpart.** The orientation and winding conventions in geometric
processing: signed polygon area, consistent triangle winding, and the outward
normal of a mesh face.

**Exact transfer.** The sign of `\omega(n,v)` decides traversal direction
exactly, and antisymmetry is exactly why consistently wound shared edges cancel.

**Non-transfer.** Nothing here covers meshes, self-intersecting boundaries,
floating-point normals that are only approximately unit, or any integral at all.
The chapter compiles no integration theorem, so no claim about numerical
quadrature follows from it.

**Diagnostic.** A contour integral with a flipped overall sign, or a mesh with
inconsistent face orientation, is usually a normal-tangent ordering error of the
kind CFT-12-003 pins down — the area form is antisymmetric, so swapping the two
arguments negates it. Changing the inner-product conjugation convention does not
produce such a flip.

## Lean translation

`CrouzeixTextbook.Part02.Chapter12` exposes six cards — five `reexported-proof`
rows over `CrouzeixConjecture` providers and one, CFT-12-001, proved here — and
six checked exercise solutions proved in this chapter.

The providers divide into two groups. `RadialOrientation` supplies the
orientation algebra: the inner-product convention, the quarter-turn exchange, and
the two radial-tangent results, of which CFT-12-003 is the only genuinely long
proof in the chapter. `ParallelRadialDifferentiability` supplies the regularity
of the polar parametrization and the derivative of its radial slice.

CFT-12-002 rests on a single library application and says so: one use of
`areaForm_rightAngleRotation_right` after an unfolding. CFT-12-001's *provider*
is likewise a single application — it is `Complex.inner` exposed under a stable
name — but the card itself is not that alias: it states the coordinate form and
derives it in three tactics. The other four cards have substantive provider
proofs, of which CFT-12-003's is much the longest.

The chapter needs `Complex.finrank_real_complex_fact` available as an instance to
mention `Complex.orientation` at all. The declaration is Mathlib's and is a plain
theorem, not an instance; what makes it usable is an `attribute [local instance]`,
which Mathlib and the provider module each apply locally. A local attribute does
not travel with an import, so this chapter applies it again.

Not compiled, and not claimed: Stokes' theorem in any form, the exterior
derivative, pullback of differential forms, the wedge product as a general
construction, interior-edge cancellation in a triangulation, the angular slice
derivative, and any statement about numerical integration. The earlier sketch of
this chapter promised a pullback definition, an exterior-derivative computation,
Stokes on a rectangle, a triangulation argument, and an orientation-reversal
statement about an integral; all five are withdrawn, and the exercises below are
stated on the material that is actually checked.

## Exercises with complete solutions

### CFT-12-E01 — both quarter-turn identities {#exercise-cft-12-e01}

Record both directions of the quarter-turn dictionary: the rotation applied to
the right argument, and to the left.

#### Complete written solution

The right-hand identity is CFT-12-002: `\omega(z, iw) = ⟪z,w⟫_ℝ`. The left-hand
companion is `⟪iz, w⟫_ℝ = \omega(z,w)`. They are different statements: one converts an area form into an inner product,
the other an inner product into an area form. Of the two, only the first is used
by CFT-12-003, to expand the area form. The orthogonality step of that card uses
neither — it needs the rotation on the *second* argument of an inner product,
`⟪z, i w⟫_ℝ = -\omega(z,w)`, which is a third identity carrying a sign change and
is not among this exercise's conjuncts. The checked solution states the two
recorded here as conjuncts, citing `complexAreaForm_I_mul_right` and
`complexRealInner_I_mul_left`; the second is not the subject of any card, and
`complexRealInner_I_mul_left` is used by no proof in this chapter.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter12.exercise_01_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.complexAreaForm_I_mul_right` and `CrouzeixConjecture.complexRealInner_I_mul_left`; both are maintained providers and neither is reproved here.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L33).
Type SHA-256: `33b530e5fbc458ac294e0f4c71181ce037fe3751bfcb3f8e17aa812f468176e1`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:33b530e5fbc458ac294e0f4c71181ce037fe3751bfcb3f8e17aa812f468176e1`.

### CFT-12-E02 — the standard frame {#exercise-cft-12-e02}

Evaluate the area form and the real inner product on the frame `(1, i)`.

#### Complete written solution

By Mathlib's concrete formula `\omega(w,z) = \operatorname{Im}(\bar w z)`, we
get `\omega(1,i) = \operatorname{Im}(i) = 1`. For the pairing, the checked solution rewrites with the conjugate form
`complexRealInner_eq_re_mul_conj` rather than with CFT-12-001's coordinate form:
`⟪1,i⟫_ℝ = \operatorname{Re}(i \cdot \overline 1) = \operatorname{Re}(i) = 0`.
CFT-12-001's own statement gives the same value as
`1\cdot 0 + 0\cdot 1 = 0`.
So the frame is orthonormal and carries unit positive area, which is the
normalization the rest of the chapter measures against. The checked solution
rewrites with `Complex.areaForm` and with CFT-12-001's provider, closing each
branch by `simp`.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter12.exercise_02_solution`.
Formal mode: `proved-here`.
Provider boundary: Mathlib's `Complex.areaForm` and `CrouzeixConjecture.complexRealInner_eq_re_mul_conj`; no card is cited for the arithmetic.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L39).
Type SHA-256: `8243fdaa203d38f69967282df6f2d3943bf2d99a5ce6e57e2df7e981259ebf5f`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:8243fdaa203d38f69967282df6f2d3943bf2d99a5ce6e57e2df7e981259ebf5f`.

### CFT-12-E03 — the exact radial increment {#exercise-cft-12-e03}

Show that moving the radius from `s_0` to `s_1` at a fixed angle displaces the
point by exactly `(s_1-s_0)e^{it}`.

#### Complete written solution

The polar point map is `c + s\,e^{it}`, affine in `s` with the constant `c` and
the fixed direction `e^{it}`. Subtracting the two values, the centres cancel and
the direction factors out:
$$
\bigl(c+s_1e^{it}\bigr)-\bigl(c+s_0e^{it}\bigr)=(s_1-s_0)e^{it}.
$$
This is the integrated form of CFT-12-006: the derivative along the slice is the
constant `e^{it}`, so the increment over an interval is the length of the
interval times that constant, with no remainder. The checked solution unfolds the
definition and distributes the scalar with `sub_smul`, closing by `ring`; it does
not invoke CFT-12-006, because the identity is available directly from the affine
form. A statement relating this increment to an integral of the derivative would
be the fundamental theorem of calculus, which this chapter does not compile.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter12.exercise_03_solution`.
Formal mode: `proved-here`.
Provider boundary: the definition of `parallelRadialParameterPoint` together with `sub_smul` and `ring`; it does not call CFT-12-006, and no integral appears.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L47).
Type SHA-256: `78b3aef3508f9e70399ffc67b3127dd1db31d702af4d9863d0e3e143df9f5488`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:78b3aef3508f9e70399ffc67b3127dd1db31d702af4d9863d0e3e143df9f5488`.

### CFT-12-E04 — antisymmetry {#exercise-cft-12-e04}

Prove the area form is antisymmetric and vanishes on a repeated argument — the
algebra behind the cancellation of interior edges.

#### Complete written solution

`\omega(z,w) = -\omega(w,z)` and `\omega(z,z) = 0`. The second follows from the
first by setting `w = z`, though Mathlib carries both as separate lemmas and the
checked solution cites each. Read geometrically, the first says reversing the
order of two edge vectors reverses the signed area, so an edge shared by two
regions and traversed once in each direction contributes cancelling amounts; the
second says a degenerate edge contributes nothing. The triangulation argument
that turns this into a global cancellation is not compiled here — only the
two-vector algebra it rests on.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter12.exercise_04_solution`.
Formal mode: `proved-here`.
Provider boundary: Mathlib's `Orientation.areaForm_swap` and `Orientation.areaForm_apply_self`; no maintained Crouzeix provider and no triangulation argument.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L54).
Type SHA-256: `2c8fce124ad489938bc316f6ec2034d8f1ebfafa2415fea7f262b99c1917a2a8`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:2c8fce124ad489938bc316f6ec2034d8f1ebfafa2415fea7f262b99c1917a2a8`.

### CFT-12-E05 — reversing orientation {#exercise-cft-12-e05}

Show that negating either argument negates the area form.

#### Complete written solution

The area form is bilinear, so `\omega(z,-w) = -\omega(z,w)` and
`\omega(-z,w) = -\omega(z,w)`. Traversing a boundary backwards negates the
tangent, so by the first identity it negates the oriented area against a fixed
normal — the compiled shadow of "reversing the boundary orientation negates the
integral". The integral statement itself is not available here, since no integral
is defined in this chapter; what is checked is the sign behaviour of the
integrand's orientation factor. The checked solution discharges both conjuncts by
`simp`, which applies the form's additive-map structure.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter12.exercise_05_solution`.
Formal mode: `proved-here`.
Provider boundary: bilinearity of `Orientation.areaForm` through `simp`; no integral and no maintained Crouzeix provider.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L61).
Type SHA-256: `015c0aef452189bda8269965f9d36dde2a8709d8732df73cfe122ef6289a4163`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:015c0aef452189bda8269965f9d36dde2a8709d8732df73cfe122ef6289a4163`.

### CFT-12-E06 — the radial tangent package {#exercise-cft-12-e06}

Under the radial hypotheses, state both the orientation sign and the arclength
identity, and identify which hypothesis does what.

#### Complete written solution

Assume `v = (a+i\rho)u` with `\rho > 0`, `‖u‖ = ‖n‖ = 1`, `⟪n,v⟫_ℝ = 0` and
`⟪n,u⟫_ℝ > 0`. Then `\omega(n,v) > 0` and `v = i\,n\,‖v‖`.

The hypotheses divide cleanly. The two unit-norm assumptions supply the
Pythagorean identity `⟪n,u⟫^2+\omega(n,u)^2=1` and make `(n,in)` an orthonormal
frame. Orthogonality eliminates the unknown radial coefficient `a`. The two
positivity assumptions, `\rho>0` and `⟪n,u⟫_ℝ>0`, are what make the quotient
`\rho/⟪n,u⟫_ℝ` positive, and that positivity is what upgrades the frame
decomposition's `\pm` into a definite sign. The radial form of `v` is what makes
the expansion available at all.

The checked solution states the two conclusions as conjuncts under exactly these
hypotheses, citing CFT-12-003 and CFT-12-004; both proofs live in the providers,
and the exercise restates them together because Part V consumes them as a pair.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter12.exercise_06_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixConjecture.complexAreaForm_radialTangent_pos` and `CrouzeixConjecture.radialTangent_eq_I_mul_normal_mul_norm`, the two card providers; the exercise adds the pairing, not a proof.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter12.lean#L69).
Type SHA-256: `d5d1528110dc44d2d2b2a679fc62b2da737fdbc2660c9c745f85e62674dc2228`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:d5d1528110dc44d2d2b2a679fc62b2da737fdbc2660c9c745f85e62674dc2228`.

## Synthesis and forward dependencies

The chapter contributes the orientation layer: a convention for the real inner
product, the quarter turn that exchanges it with the area form, a positively
oriented and arclength-normalized boundary tangent, and a `C^1` polar
parametrization whose radial slices are unit-speed rays.

Part III turns to analysis, starting with metric and normed spaces, and does not
use this material. It reappears in Part V, where the boundary of a convex
region is integrated over and all four orientation facts are consumed at once:
CFT-12-004 supplies the tangent in the form the change of variables wants, and
CFT-12-005 supplies the regularity that licenses the change of variables at all.
