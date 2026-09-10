---
id: cft-chapter-11-differentiation-as-linear-approximation
title: Differentiation as linear approximation
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-09-10
tags: [crouzeix-textbook, geometry-and-calculus, mathematics, lean]
confidence: high
canonical: 11_differentiation_as_linear_approximation.md
chapter: 11
part: 2
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 11: Differentiation as linear approximation

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-ii-geometry-and-calculus|Part II -- Geometry and calculus]]
Previous: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/10_multilinear_maps_and_tensors|Chapter 10 — Multilinear maps and tensors]]
Next: [[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/12_differential_forms_and_stokes|Chapter 12 — Differential forms and Stokes]]

## Opening problem

Chapters 5 through 10 built linear algebra and then measured it. Nothing so far
bends. A derivative is the device that lets a curved map borrow the linear
theory: at each point it hands back a single linear map, and every question
about first-order behaviour becomes a question about that map.

The device is worth stating carefully because two different things are usually
called "the derivative". One is the linear map itself, an object with a domain
and a codomain. The other is a rectangle of numbers, its matrix in coordinates.
The chapter keeps them apart, because the two automatic-differentiation modes
that Part II hands to the rest of the book differ precisely in which one they
touch: forward mode applies the linear map, reverse mode applies its adjoint,
and neither forms the rectangle.

## Conceptual model

Fix normed spaces `E` and `F`. A map `f : E → F` is differentiable at `x` when
there is a *bounded* linear `Df(x) : E →L F` with
$$
f(x+h)=f(x)+Df(x)h+r(h),\qquad \frac{\lVert r(h)\rVert}{\lVert h\rVert}\to0 .
$$
Three consequences organise the chapter. The map is unique, so "the" derivative
is well defined. A bounded linear map is its own derivative, which is the base
case of every computation. And derivatives compose, which is the chain rule.

In coordinates on `\mathbb R^n` the derivative is a matrix `A`, and the two
modes are the two ways to contract it. Forward mode computes `A v` for a tangent
`v`; reverse mode computes `A^{\mathsf T} y` for a cotangent `y`. Chapter 10
proved these are adjoint. What this chapter adds is that the transpose is the
*only* matrix with that property, which is the reason reverse mode computes the
adjoint rather than an accident that resembles it.

## Running example: the shear as its own derivative

The running family
$$
A_{\lambda,\alpha}=\begin{bmatrix}\lambda&\alpha\\0&\lambda\end{bmatrix}
$$
is linear, so it is its own derivative at every point: the first-order model of a
linear map is exact and the remainder is identically zero. That makes it the
cleanest possible test of the coordinate story. Forward mode through
`A_{\lambda,\alpha}` sends the tangent `v` to `A_{\lambda,\alpha}v`; reverse
mode sends the cotangent `y` to
$$
A_{\lambda,\alpha}^{\mathsf T}y=\begin{bmatrix}\lambda&0\\\alpha&\lambda\end{bmatrix}y ,
$$
and the shear entry `\alpha` moves from the upper right to the lower left. The
two modes see the same matrix from opposite sides, which is exactly the content
of the duality card below.

## A note on the previous formalization

Before this chapter was written the six items were re-exports of declarations in
the `AutodiffGeometry` namespace. That namespace is not one the receipt exporter
treats as maintained, so none of the six could carry a checked provider. The
deeper problem was mathematical: four of those five re-exported statements were
proved by `rfl`. `AutodiffGeometry` *defines* `jvp` to be `matVec`, `vjp` to be
`matVec` of the transpose, and `hvp` to be `matVec`, so the theorems asserting
those equalities were unfoldings of definitions, and that module's own docstring
disclaims analytic differentiability. They recorded a naming convention.

Every card below is therefore proved locally, and the analytic content is stated
where it actually lives: in the Fréchet layer, where uniqueness, the linear base
case and the chain rule are genuine theorems.

## Formal development

### CFT-11-001 — gradient component contract {#cft-11-001}

#### Purpose

A gradient is a vector, but what a derivative produces is a covector. This card
pins the identification the rest of the book uses: pairing the gradient with the
`i`th coordinate direction returns its `i`th entry, and that number is the `i`th
partial derivative. Nothing here is a definition unfolding; the second clause is
a derivative computation.

#### Definitions and notation

Work in `\mathbb R^n` written as `Fin n → ℝ` with the coordinate pairing
`x ⬝ᵥ y = \sum_i x_iy_i` of Chapter 7. Write `e_i` for `Pi.single i 1`, the
vector with `1` in slot `i` and `0` elsewhere.

#### Statement

For `g : Fin n → ℝ` and an index `i`, the pairing `g ⬝ᵥ e_i` equals `g i`, and
the function `t ↦ g ⬝ᵥ (t e_i)` has derivative `g i` at `t = 0`.

#### Hypothesis ledger

Only finiteness of the index type is used. There is no differentiability
hypothesis: the function being differentiated is linear in `t` by construction,
which is what makes the directional derivative computable in closed form. The
card says nothing about a gradient of a nonlinear `f`; it fixes the dictionary
between a covector's coordinates and directional derivatives.

#### Proof roadmap

Expand the pairing, collapse the sum against the indicator, then observe that the
resulting function of `t` is `t ↦ t·g i` and differentiate it.

#### Proof

The pairing is `\sum_j g_j (e_i)_j`. Every term with `j ≠ i` vanishes and the
term at `j = i` is `g_i·1`, so the sum is `g_i`; in Lean this is `simp` with
`Pi.single_apply` and `Finset.sum_ite_eq'`.

For the second clause, homogeneity of the pairing in its right argument gives
$$
g\cdot(t\,e_i) = t\,(g\cdot e_i) = t\,g_i ,
$$
using `dotProduct_smul` and then the first clause. So the function of `t` is
literally `t ↦ t·g_i`, whose derivative at `0` is `g_i` by the product rule for a
constant factor — `(hasDerivAt_id 0).mul_const (g i)` in the checked proof. The
two clauses together are the statement that the `i`th entry of a gradient and the
derivative along `e_i` are the same real number.

#### Worked instance

For `g = (3, 4)` and `i = 1`, the pairing with `e_1 = (0,1)` is `4`, and
`t ↦ 3·0 + 4·t` has derivative `4` at the origin.

#### Boundary case

The card is about a covector already given in coordinates. It does not assert
that a nonlinear `f` has a gradient, nor that partial derivatives assembled into
a vector are a derivative — CFT-11-005's counterexample below shows that
direction-by-direction data need not assemble into a linear map at all.

#### Historical context

The identification of a covector with a vector requires an inner product and is
Riesz representation in finite dimensions. Writing gradients as column vectors is
a convention that suppresses this choice, which is harmless in `\mathbb R^n` with
the standard pairing and misleading as soon as the pairing changes.
Source boundary: Mathlib 4.32.1 coordinate and derivative declarations.
Review status: statement, both clauses, and the compiler locator reviewed together.

#### ML analogy

Mathematical object: the `i`th coordinate of a covector, read as a directional derivative.
ML counterpart: the `i`th entry of a parameter gradient buffer.
Exact transfer: the entry equals the derivative of the loss along the `i`th coordinate direction.
Non-transfer: it does not say the buffer was produced by a differentiable computation, nor that coordinatewise finite differences approximate it at any given step size.
Diagnostic: an entry that disagrees with a directional probe indicates a non-differentiable path, not a bookkeeping error.

#### Pedagogical prerequisites

Chapter 7's coordinate pairing and the derivative of a linear function of one
real variable. No operator theory.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.gradient_component_contract`.
Formal mode: `proved-here`.
Substantive provider: `none` among maintained Crouzeix declarations; the pairing collapse is `simp` over `Pi.single`, and the derivative clause is Mathlib's `hasDerivAt_id` with `mul_const`.
Readable type map: `g` is the gradient in coordinates, `i` the index, and `Pi.single i 1` the direction `e_i`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L55).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Nat} (g : Fin n → Real) (i : Fin n), And (Eq.{1} (dotProduct.{0, 0} g (Pi.single.{0, 0} i (OfNat.ofNat.{0} 1))) (g i)) (HasDerivAt.{0, 0} (fun t => dotProduct.{0, 0} g (HSMul.hSMul.{0, 0, 0} t (Pi.single.{0, 0} i (OfNat.ofNat.{0} 1)))) (g i) (OfNat.ofNat.{0} 0))`.
Type SHA-256: `20ac241c2f63421a13d2427b679ebd777a14742cf991360c84cc76ea4423f65e`.
Direct maintained dependencies: `none`; the proof descends directly to Mathlib.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: a finite index type; no differentiability hypothesis.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:20ac241c2f63421a13d2427b679ebd777a14742cf991360c84cc76ea4423f65e`.

#### Exercises and solutions

CFT-11-E01 records the two facts this dictionary rests on — that a bounded linear
map is its own derivative and that the identity differentiates to the identity —
without calling this card.

### CFT-11-002 — the derivative action is the JVP {#cft-11-002}

#### Purpose

Forward mode never builds a Jacobian. This card is the identity that licenses
that: the derivative of a matrix action *is* the action, so applying the
derivative to a tangent is one matrix-vector product.

#### Definitions and notation

`jacobianAction A` is the matrix `A` read as a bounded linear map
`(Fin n → ℝ) →L[ℝ] (Fin m → ℝ)`. It is `A.mulVecLin` passed through
`LinearMap.toContinuousLinearMap`, which is available because the domain is
finite-dimensional; boundedness is a theorem there, not a hypothesis.

#### Statement

For every matrix `A`, every base point `x` and every tangent `v`, the map
`w ↦ A w` has Fréchet derivative `jacobianAction A` at `x`, and
`jacobianAction A v = A v`.

#### Hypothesis ledger

None beyond finite index types. In particular no invertibility, no rank
condition, and no hypothesis on `x`: the derivative of a linear map does not
depend on where it is taken.

#### Proof roadmap

Both clauses are immediate once the matrix is packaged as a bounded linear map:
the first is the linear base case, the second is definitional unfolding.

#### Proof

A bounded linear `L` satisfies `L(x+h) = L x + L h` exactly, so the remainder is
identically zero and `L` is its own derivative; this is Mathlib's
`ContinuousLinearMap.hasFDerivAt`, applied here to `jacobianAction A`. The second
clause holds by `rfl`, because `jacobianAction A` was *defined* by transporting
`A.mulVecLin`, whose application is `A v`.

The honest reading is that this card carries one theorem and one convention. The
theorem is the linear base case. The convention is that the packaging preserves
the action, and the checked proof records it as `rfl` rather than pretending
otherwise. What makes the pair worth stating is that the previous formalization
recorded *only* the convention, in a namespace the receipt could not check.

#### Worked instance

For `A_{\lambda,\alpha}` and tangent `v = (v_0,v_1)` the derivative action is
`(\lambda v_0 + \alpha v_1,\ \lambda v_1)`, at every base point.

#### Boundary case

The card is about a map that is already linear. For a nonlinear `f` the
derivative is a different matrix at each point, and no clause here supplies it.

#### Historical context

That a bounded linear map is its own derivative is the first lemma in every
treatment of the Fréchet derivative; the finite-dimensional convenience that
*every* linear map is bounded is what removes the continuity hypothesis here.
Source boundary: Mathlib 4.32.1 derivative and finite-dimension declarations.
Review status: derivative clause and unfolding clause reviewed separately.

#### ML analogy

Mathematical object: the derivative of a linear layer, applied to a tangent.
ML counterpart: one forward-mode JVP through a dense layer.
Exact transfer: the tangent propagates by the same matrix action the layer applies.
Non-transfer: nothing here covers a nonlinear activation, a fused kernel, or the cost model that makes forward mode attractive only when inputs outnumber outputs.
Diagnostic: a JVP that varies with the base point signals that the layer was not linear.

#### Pedagogical prerequisites

The definition of the Fréchet derivative and Chapter 10's matrix-vector action.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.derivative_action_is_jvp`.
Formal mode: `proved-here`.
Substantive provider: `none` among maintained Crouzeix declarations; the derivative clause is Mathlib's `ContinuousLinearMap.hasFDerivAt` and the action clause is `rfl`.
Readable type map: `A` is the Jacobian, `x` the base point, `v` the tangent, and `jacobianAction A` the derivative as a bounded map.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L70).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {m n : Nat} (A : Matrix.{0, 0, 0} (Fin m) (Fin n) Real) (x v : Fin n → Real), And (HasFDerivAt.{0, 0, 0} (fun w => Matrix.mulVec.{0, 0, 0} A w) (CrouzeixTextbook.Part02.jacobianAction A) x) (Eq.{1} (DFunLike.coe.{1, 1, 1} (CrouzeixTextbook.Part02.jacobianAction A) v) (Matrix.mulVec.{0, 0, 0} A v))`.
Type SHA-256: `f20c0b639553eef4c9670e3e49f7ccaca9c0a0f8545b9c27738b139a2a8f8582`.
Direct maintained dependencies: `CrouzeixTextbook.Part02.jacobianAction`, the packaging definition.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: finite index types; no condition on `A` or `x`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:f20c0b639553eef4c9670e3e49f7ccaca9c0a0f8545b9c27738b139a2a8f8582`.

#### Exercises and solutions

CFT-11-E01 proves the linear base case in the abstract setting this card
instantiates.

### CFT-11-003 — the derivative pullback is the VJP {#cft-11-003}

#### Purpose

Reverse mode moves a cotangent backwards through a layer. This card says the
backward move is the transpose action, in the same coordinates.

#### Definitions and notation

`y ᵥ* A` is the row-vector action of Chapter 10: the covector `y` pushed through
`A` on the left.

#### Statement

For every `A` and cotangent `y`, `y ᵥ* A = A^{\mathsf T} y`, and
`jacobianAction A^{\mathsf T} y = A^{\mathsf T} y`.

#### Hypothesis ledger

None beyond finite index types.

#### Proof roadmap

The first clause is Chapter 10's card, re-used; the second is unfolding.

#### Proof

The mathematical content of the first clause is not new here. It is
[[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/10_multilinear_maps_and_tensors#cft-10-002|CFT-10-002]],
`transpose_coordinate_action`, whose checked proof rewrites with
`Matrix.vecMul_transpose` and cancels a double transpose. This card cites it
rather than reproving it, and the second clause is `rfl` for the same packaging
reason as CFT-11-002.

What this card adds is the placement: it names the transpose action as the
*derivative pullback*, which is the role Chapter 10 did not assign it. The
reader should not read a second proof here. There is one proof, in Chapter 10.

#### Worked instance

For `A_{\lambda,\alpha}` and cotangent `y=(y_0,y_1)` the pullback is
`(\lambda y_0,\ \alpha y_0 + \lambda y_1)`: the shear entry now mixes the first
cotangent slot into the second.

#### Boundary case

The identification is with the *transpose*, not the conjugate transpose. Over
`\mathbb C` these differ, as Chapter 7 showed with `diag(i,0)`, and reverse-mode
conventions for complex parameters have to choose one.

#### Historical context

Reverse-mode differentiation was described as a general technique in the 1970s
and popularised in machine learning as backpropagation; the identification of the
backward pass with the transpose is the whole of its linear-algebraic content.
Source boundary: Chapter 10's coordinate duality plus Mathlib 4.32.1.
Review status: reviewed together with CFT-10-002, whose proof it re-uses.

#### ML analogy

Mathematical object: the transpose action on a covector.
ML counterpart: the backward pass of a dense layer.
Exact transfer: the cotangent propagates by the transposed matrix.
Non-transfer: nothing here covers checkpointing, accumulation order, or the numerical effect of doing the backward pass in reduced precision.
Diagnostic: a backward pass disagreeing with the transpose indicates a wrong convention, most often a conjugation.

#### Pedagogical prerequisites

Chapter 10's row-vector action and transpose.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.derivative_pullback_is_vjp`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixTextbook.Part02.transpose_coordinate_action` (CFT-10-002) supplies the first clause; the second is `rfl`. No second proof of the transpose identity is offered.
Readable type map: `A` is the Jacobian and `y` the cotangent; `y ᵥ* A` is the covector pushed through `A`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L77).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {m n : Nat} (A : Matrix.{0, 0, 0} (Fin m) (Fin n) Real) (y : Fin m → Real), And (Eq.{1} (Matrix.vecMul.{0, 0, 0} y A) (Matrix.mulVec.{0, 0, 0} (Matrix.transpose.{0, 0, 0} A) y)) (Eq.{1} (DFunLike.coe.{1, 1, 1} (CrouzeixTextbook.Part02.jacobianAction (Matrix.transpose.{0, 0, 0} A)) y) (Matrix.mulVec.{0, 0, 0} (Matrix.transpose.{0, 0, 0} A) y))`.
Type SHA-256: `f53fc7922b6aad49ef87d9a3ccb4b3f4a99f42775d53f2163273301f532632f2`.
Direct maintained dependencies: `CrouzeixTextbook.Part02.transpose_coordinate_action` and `CrouzeixTextbook.Part02.jacobianAction`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: finite index types.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:f53fc7922b6aad49ef87d9a3ccb4b3f4a99f42775d53f2163273301f532632f2`.

#### Exercises and solutions

CFT-11-E06 states the duality with its uniqueness clause, which is the part this
card does not carry.

### CFT-11-004 — the Hessian-vector action {#cft-11-004}

#### Purpose

Second-order methods need `Hv`, not `H`. This card derives the Hessian-vector
product for a quadratic form and shows it is again a matrix action, so it costs
one product rather than `n` of them.

#### Definitions and notation

For symmetric `H` write the quadratic form `q(x) = (Hx) ⬝ᵥ x`. Symmetry means
`H^{\mathsf T} = H`.

#### Statement

For symmetric `H`, the first-order expansion of `q` at `x` has cross terms
summing to `(2Hx) ⬝ᵥ w` for every `w`; the gradient map `z ↦ 2Hz` has Fréchet
derivative `jacobianAction (2H)` at `x`; and that derivative sends `v` to `2Hv`.

#### Hypothesis ledger

Symmetry is used, and only in the first clause. Without it the two cross terms
`(Hx) ⬝ᵥ w` and `(Hw) ⬝ᵥ x` are different numbers and the gradient is
`(H + H^{\mathsf T})x`, not `2Hx`. The second and third clauses hold for any `H`;
they are stated at `2H` because that is the gradient the first clause produced.

#### Proof roadmap

Collapse the cross terms with Chapter 10's duality and symmetry, then apply the
linear base case to the gradient map.

#### Proof

Expanding `q(x+w) - q(x)` produces the two cross terms `(Hx) ⬝ᵥ w` and
`(Hw) ⬝ᵥ x` together with the second-order term `(Hw) ⬝ᵥ w`. The supporting
lemma `symmetric_cross_terms` shows the two cross terms are equal: by
[[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/10_multilinear_maps_and_tensors#cft-10-001|CFT-10-001]],
`(Hw) ⬝ᵥ x = w ⬝ᵥ (H^{\mathsf T}x)`, which symmetry turns into `w ⬝ᵥ (Hx)` and
commutativity of the pairing into `(Hx) ⬝ᵥ w`. Their sum is therefore
`2\,(Hx) ⬝ᵥ w`, and pulling the scalar into the left argument writes it as
`(2Hx) ⬝ᵥ w`. So the gradient of `q` at `x` is `2Hx`.

The gradient map `z ↦ 2Hz` is linear, so it is its own derivative by the same
base case as CFT-11-002 — and `2Hz = (2H)z` by `Matrix.smul_mulVec`, which is the
rewriting the checked proof performs before applying it. The Hessian is therefore
the matrix `2H`, and the Hessian-vector product is `2Hv`, one matrix action.

The chapter compiles this for a quadratic form only. That a general twice
differentiable `f` has a symmetric second derivative is Clairaut's theorem and is
not compiled here.

#### Worked instance

For `H = I` the form is `q(x) = x ⬝ᵥ x`, the gradient is `2x`, and the
Hessian-vector product is `2v` at every point. Exercise E02 checks the cross-term
collapse in this instance.

#### Boundary case

Drop symmetry and the first clause fails: for
`H = \begin{bmatrix}0&1\\0&0\end{bmatrix}`, `x = e_0` and `w = e_1`, the cross
terms are `1` and `0`, whose sum is not `2·1`. The quadratic form of a
non-symmetric `H` sees only its symmetric part.

#### Historical context

The observation that curvature can be probed one direction at a time without
forming the Hessian underlies the conjugate-gradient and Newton-Krylov families;
in automatic differentiation the same product is obtained by differentiating the
gradient map, which is the route this card takes.
Source boundary: Chapter 10's duality plus Mathlib 4.32.1.
Review status: symmetry usage isolated to the first clause and reviewed as such.

#### ML analogy

Mathematical object: the derivative of the gradient map of a quadratic form.
ML counterpart: a Hessian-vector product obtained by differentiating a gradient.
Exact transfer: for an exactly quadratic objective, the product is the matrix action and is linear in the probe direction.
Non-transfer: a real loss is not quadratic, so the Hessian varies with the point; nothing here bounds that variation or licenses a fixed-curvature step.
Diagnostic: a curvature probe that is not linear in the direction indicates the local quadratic model has been left behind.

#### Pedagogical prerequisites

Chapter 10's coordinate duality, matrix symmetry, and the linear base case above.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.hessian_vector_action`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixTextbook.Part02.symmetric_cross_terms`, which itself rests on CFT-10-001; the derivative clauses use Mathlib's `ContinuousLinearMap.hasFDerivAt` after `Matrix.smul_mulVec`.
Readable type map: `H` is the symmetric matrix, `x` the base point, `v` the probe direction, and `w` the test direction of the cross-term clause.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L142).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {n : Nat} (H : Matrix.{0, 0, 0} (Fin n) (Fin n) Real), Eq.{1} (Matrix.transpose.{0, 0, 0} H) H → ∀ (x v : Fin n → Real), And (∀ (w : Fin n → Real), Eq.{1} (HAdd.hAdd.{0, 0, 0} (dotProduct.{0, 0} (Matrix.mulVec.{0, 0, 0} H x) w) (dotProduct.{0, 0} (Matrix.mulVec.{0, 0, 0} H w) x)) (dotProduct.{0, 0} (HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) (Matrix.mulVec.{0, 0, 0} H x)) w)) (And (HasFDerivAt.{0, 0, 0} (fun z => HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) (Matrix.mulVec.{0, 0, 0} H z)) (CrouzeixTextbook.Part02.jacobianAction (HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) H)) x) (Eq.{1} (DFunLike.coe.{1, 1, 1} (CrouzeixTextbook.Part02.jacobianAction (HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) H)) v) (HSMul.hSMul.{0, 0, 0} (OfNat.ofNat.{0} 2) (Matrix.mulVec.{0, 0, 0} H v))))`.
Type SHA-256: `9aabb42cdfaa07edac465d7824a08c6b075fe9e3f5a570a557dc6d1dcaab297d`.
Direct maintained dependencies: `CrouzeixTextbook.Part02.symmetric_cross_terms`, `CrouzeixTextbook.Part02.jacobianAction`, `CrouzeixTextbook.Part02.jacobianAction_apply`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: symmetric `H` for the first clause; arbitrary `H` for the other two.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:9aabb42cdfaa07edac465d7824a08c6b075fe9e3f5a570a557dc6d1dcaab297d`.

#### Exercises and solutions

CFT-11-E02 computes the cross-term collapse for the squared-length form directly,
without symmetry as a hypothesis, since the identity matrix is its own transpose.

### CFT-11-005 — JVP and VJP are adjoint, and only the transpose is {#cft-11-005}

#### Purpose

The duality itself is Chapter 10's. What this card adds is uniqueness: the
transpose is the *only* matrix that pairs correctly with the forward action. That
is the difference between "reverse mode computes something adjoint-like" and
"reverse mode computes the adjoint".

#### Definitions and notation

As above; `B` ranges over candidate backward matrices of the transposed shape.

#### Statement

For every `A`: first, `(Av) ⬝ᵥ y = v ⬝ᵥ (A^{\mathsf T}y)` for all `v` and `y`;
second, if some `B` satisfies `(Av) ⬝ᵥ y = v ⬝ᵥ (By)` for all `v` and `y`, then
`B = A^{\mathsf T}`.

#### Hypothesis ledger

None beyond finite index types. Uniqueness needs the identity for *all* `v` and
`y`; a `B` agreeing on a proper subset of directions need not be the transpose,
which is why the quantifier order matters.

#### Proof roadmap

Cite Chapter 10 for existence; for uniqueness, test the hypothesis on pairs of
standard basis vectors and read off entries.

#### Proof

The first clause is
[[knowledge/crouzeix_textbook/part_02_geometry_and_calculus/10_multilinear_maps_and_tensors#cft-10-001|CFT-10-001]],
`bilinear_pairing_duality`, cited and not reproved.

For uniqueness, suppose `B` satisfies the same identity. Fix indices `i` and `j`
and instantiate at `v = e_i`, `y = e_j`. The left side is `(Ae_i) ⬝ᵥ e_j`, which
selects the `(j,i)` entry of `A`. Rewriting the left side by the first clause
turns the hypothesis into
$$
e_i\cdot(A^{\mathsf T}e_j) = e_i\cdot(Be_j),
$$
and pairing with `e_i` selects the `i`th slot of each side: `A^{\mathsf T}_{ij}`
on the left and `B_{ij}` on the right. Since `i` and `j` were arbitrary,
`B = A^{\mathsf T}` by extensionality. The checked proof is exactly this: `ext i j`,
instantiate the hypothesis at the two `Pi.single` vectors, rewrite with the
duality, and finish by `simp` over the indicator sums.

#### Worked instance

For `A_{\lambda,\alpha}` the only backward matrix consistent with the forward
action is the lower-triangular `A_{\lambda,\alpha}^{\mathsf T}`. Replacing it by
`A_{\lambda,\alpha}` itself fails already at `v=e_1`, `y=e_0`, where the two
sides read `\alpha` and `0`.

#### Boundary case

Uniqueness is a statement about matrices over the real coordinate pairing.
Change the pairing to a nondegenerate `G` and the adjoint becomes
`G^{-1}A^{\mathsf T}G`; the transpose is distinguished by the choice of pairing,
not by the matrix.

#### Historical context

That an adjoint is unique once the pairing is fixed is the finite-dimensional
shadow of the same statement for Hilbert-space operators, where it follows from
Riesz representation.
Source boundary: Chapter 10's coordinate duality plus Mathlib 4.32.1.
Review status: uniqueness clause reviewed as the card's new content; existence attributed to CFT-10-001.

#### ML analogy

Mathematical object: uniqueness of the adjoint with respect to a fixed pairing.
ML counterpart: the assertion that a hand-written backward pass is correct if and only if it matches the transpose on all inputs.
Exact transfer: agreement on every tangent-cotangent pair forces equality with the transpose, so a full pairing test is a complete check.
Non-transfer: agreement on sampled directions does not force equality, so a passing spot-check of a custom gradient is not a proof.
Diagnostic: a backward pass that matches on random probes but fails a basis-vector sweep has an entry-level error.

#### Pedagogical prerequisites

Chapter 10's duality and matrix extensionality.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.jvp_vjp_duality`.
Formal mode: `proved-here`.
Substantive provider: `CrouzeixTextbook.Part02.bilinear_pairing_duality` (CFT-10-001) supplies the duality clause; the uniqueness clause is proved locally by basis-vector instantiation.
Readable type map: `A` is the Jacobian, `B` a candidate backward matrix, `v` a tangent and `y` a cotangent.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L84).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {m n : Nat} (A : Matrix.{0, 0, 0} (Fin m) (Fin n) Real), And (∀ (v : Fin n → Real) (y : Fin m → Real), Eq.{1} (dotProduct.{0, 0} (Matrix.mulVec.{0, 0, 0} A v) y) (dotProduct.{0, 0} v (Matrix.mulVec.{0, 0, 0} (Matrix.transpose.{0, 0, 0} A) y))) (∀ (B : Matrix.{0, 0, 0} (Fin n) (Fin m) Real), (∀ (v : Fin n → Real) (y : Fin m → Real), Eq.{1} (dotProduct.{0, 0} (Matrix.mulVec.{0, 0, 0} A v) y) (dotProduct.{0, 0} v (Matrix.mulVec.{0, 0, 0} B y))) → Eq.{1} B (Matrix.transpose.{0, 0, 0} A))`.
Type SHA-256: `d2008dc98e51c53e78654d3f872153da3a6baf9d240d818ff62ad8ccbeab6306`.
Direct maintained dependencies: `CrouzeixTextbook.Part02.bilinear_pairing_duality`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: finite index types; the uniqueness clause quantifies over all `v` and `y`.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:d2008dc98e51c53e78654d3f872153da3a6baf9d240d818ff62ad8ccbeab6306`.

#### Exercises and solutions

CFT-11-E06 restates both clauses at a fixed `A`, which is the form the later
adjoint arguments consume.

### CFT-11-006 — the chain kernel {#cft-11-006}

#### Purpose

The chain rule composes derivatives as linear maps. In coordinates that
composition is matrix multiplication, and this card is the identity that says so.

#### Definitions and notation

`A` maps `n` inputs to `m`, `B` maps `m` to `p`, and `BA` is their product.

#### Statement

For any semiring `R` and any `A`, `B`, `v` of matching shapes,
`B(Av) = (BA)v`.

#### Hypothesis ledger

A semiring suffices; commutativity is not used, nor is any analytic hypothesis.
The card is pure algebra, which is the point: the analytic content sits in the
Fréchet chain rule, and this identity is what turns it into a matrix product.

#### Proof roadmap

Mathlib's associativity of the matrix-vector action.

#### Proof

The checked proof is `Matrix.mulVec_mulVec v B A`, a single library application.
Its own content is the interchange of two finite sums: the `i`th slot of
`B(Av)` is `\sum_k B_{ik}\sum_j A_{kj}v_j`, and the `i`th slot of `(BA)v` is
`\sum_j(\sum_k B_{ik}A_{kj})v_j`; the two agree by distributing and swapping the
order of summation. This card does not run that argument — it names the library
result that does.

The companion `chain_rule_in_coordinates_is_matrix_product` is where the two
halves meet. It composes the two derivative statements of CFT-11-002 through the
Fréchet chain rule and rewrites the resulting composite bounded map by
`jacobianAction_comp`, the lemma that
`jacobianAction B ∘L jacobianAction A = jacobianAction (BA)`. The conclusion is
that the composite of the two matrix actions has derivative the action of `BA`.
That statement — not this card alone — is what the chapter exists to prove.

#### Worked instance

Taking `B = A = A_{\lambda,\alpha}` gives
`A_{\lambda,\alpha}^2 = A_{\lambda^2,\,2\lambda\alpha}`, so composing the
layer with itself doubles the shear and squares the eigenvalue, exactly as
Chapter 6's polynomial calculus predicted.

#### Boundary case

Matrix multiplication does not commute, and neither does composition: `BA` and
`AB` are different derivatives of different composites, and for non-square shapes
only one of them typechecks. The type discipline is doing real work here.

#### Historical context

The reduction of the chain rule to matrix multiplication is what makes
reverse-mode accumulation a question of association order; the observation that
choosing that order optimally is itself a hard problem is the origin of the
optimal-Jacobian-accumulation literature.
Source boundary: Mathlib 4.32.1 matrix declarations.
Review status: reviewed together with the companion chain-rule theorem.

#### ML analogy

Mathematical object: associativity of composed linear actions.
ML counterpart: the freedom to accumulate a chain of Jacobians in any association order.
Exact transfer: every association gives the same exact result.
Non-transfer: the orders differ enormously in cost and in floating-point result; nothing here is a claim about either.
Diagnostic: two accumulation orders disagreeing beyond rounding indicates a shape or transpose error, not an ordering effect.

#### Pedagogical prerequisites

Chapter 5's matrix product and the definition of the Fréchet derivative.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.linear_approximation_chain_kernel`.
Formal mode: `proved-here`.
Substantive provider: `none` among maintained Crouzeix declarations; the checked proof is the single library application `Matrix.mulVec_mulVec`.
Readable type map: `A` and `B` are the two Jacobians and `v` the tangent entering the composite.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L99).
Compiler receipt: fresh canonical compiler output, not copied source metadata.
Normalized type: `∀ {R : Type u_1} [inst : Semiring.{u_1} R] {m n p : Nat} (A : Matrix.{0, 0, u_1} (Fin m) (Fin n) R) (B : Matrix.{0, 0, u_1} (Fin p) (Fin m) R) (v : Fin n → R), Eq.{u_1 + 1} (Matrix.mulVec.{u_1, 0, 0} B (Matrix.mulVec.{u_1, 0, 0} A v)) (Matrix.mulVec.{u_1, 0, 0} (HMul.hMul.{u_1, u_1, u_1} B A) v)`.
Type SHA-256: `e8eacf71deee4ea839cea592352011dd5f1b14a2dc680e8af935e814e416fa02`.
Direct maintained dependencies: `none`; the proof descends directly to Mathlib.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Assumptions: a semiring of scalars and matching finite shapes.
Verification target: `CrouzeixTextbook`.
Receipt identity: `CrouzeixTextbook:e8eacf71deee4ea839cea592352011dd5f1b14a2dc680e8af935e814e416fa02`.

#### Exercises and solutions

CFT-11-E04 states the coordinate identity together with the Fréchet chain rule
conclusion, which is the pairing the rest of Part II uses.

## Worked examples

**The squared length.** For `q(x) = x ⬝ᵥ x` on `\mathbb R^n`, expanding
`q(x+w)` gives `x ⬝ᵥ x + 2(x ⬝ᵥ w) + w ⬝ᵥ w`. The linear term is `2(x ⬝ᵥ w)`, so
the gradient is `2x`, and the quadratic remainder `w ⬝ᵥ w` is `O(\lVert w\rVert^2)`
and therefore `o(\lVert w\rVert)`. This is CFT-11-004 at `H = I`, and the
cross-term collapse is exercise E02.

**A composite through the running family.** Let `f(w) = A_{\lambda,\alpha}w` and
`g(w) = A_{\mu,\beta}w`. The composite has derivative
`A_{\mu,\beta}A_{\lambda,\alpha} = A_{\mu\lambda,\ \mu\alpha+\beta\lambda}`.
Forward mode evaluates this on a tangent right-to-left, one product at a time;
reverse mode evaluates the transpose left-to-right. Both avoid ever forming the
`2×2` product — which matters only when the dimensions are large, and is the
entire cost argument for both modes.

**Directions without a derivative.** Let
$$
c(x,y)=\frac{x^{3}}{x^{2}+y^{2}},\qquad c(0,0)=0 .
$$
Then `c(tv) = t\,c(v)` for every real `t` and every `v`, so along every line
through the origin `c` is exactly linear and every directional derivative exists,
with value `c(v)`. But `c(1,1) = 1/2` while `c(1,0) + c(0,1) = 1 + 0 = 1`. The
directional derivative is not additive, so it is not a linear map, so `c` has no
Fréchet derivative at the origin. Exercise E05 compiles all three steps.

## ML bridge

Forward mode transports tangents through `A`; reverse mode pulls cotangents back
through `A^{\mathsf T}`. The compiled layer of this chapter is the finite-coordinate
algebra of those two operations together with the Fréchet facts they rest on. It
is not a model of tracing, pytrees, fused kernels, or floating-point execution,
and it makes no cost claim: the reason reverse mode dominates for scalar losses
is a count of passes, and no count is compiled here.

The one transferable sharpening is CFT-11-005's uniqueness clause. A custom
backward pass is correct exactly when it agrees with the transpose on every
tangent-cotangent pair. Agreement on sampled directions is not enough, and the
compiled proof shows why a basis sweep is: instantiating at `e_i` and `e_j`
recovers the `(i,j)` entry.

## Lean translation

`CrouzeixTextbook.Part02.Chapter11` exposes six checked cards, six checked
exercise solutions, and the supporting declarations they use. The Fréchet layer —
`frechet_derivative_unique`, `continuous_linear_map_hasFDerivAt` and
`frechet_chain_rule` — is stated over arbitrary real normed spaces and delegates
to Mathlib's `HasFDerivAt.unique`, `ContinuousLinearMap.hasFDerivAt` and
`HasFDerivAt.comp`. The coordinate layer works in `Fin n → ℝ` with Chapter 7's
pairing, and `jacobianAction` is the bridge, defined through
`LinearMap.toContinuousLinearMap`, which is where finite-dimensionality is used.

Two cards re-use Chapter 10 rather than reproving it: CFT-11-003 cites CFT-10-002
and CFT-11-005 cites CFT-10-001. Their boundary paragraphs say so. The genuinely
new coordinate content of the chapter is the uniqueness clause of CFT-11-005, the
symmetric cross-term collapse behind CFT-11-004, the composite theorem
`chain_rule_in_coordinates_is_matrix_product`, and the `coneCusp` boundary.

Not compiled here, and not claimed: implicit differentiation, Clairaut symmetry
of second derivatives, derivatives of nonlinear maps other than the quadratic
form and `coneCusp`, and any statement about numerical differentiation. The
earlier sketch of this chapter promised implicit differentiation; it is not in
the compiled layer and the promise is withdrawn rather than left standing.

## Exercises with complete solutions

### CFT-11-E01 — the linear base case {#exercise-cft-11-e01}

State the two facts the definition of the Fréchet derivative immediately yields:
a bounded linear map is its own derivative, and the identity is its own
derivative.

#### Complete written solution

Let `L : E →L F` be bounded linear. Then `L(x+h) = Lx + Lh` exactly, so the
remainder `r(h) = L(x+h) - Lx - Lh` is identically zero and `\lVert r(h)\rVert/\lVert h\rVert \to 0`
trivially. Hence `L` has derivative `L` at every `x`. The identity map is a
bounded linear map, so the same argument gives it derivative `id`. The checked
solution states both conjuncts over arbitrary real normed spaces and discharges
each by Mathlib's `ContinuousLinearMap.hasFDerivAt`; it does not call CFT-11-002,
which is the coordinate instance of the same base case.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter11.exercise_01_solution`.
Formal mode: `proved-here`.
Provider boundary: Mathlib `ContinuousLinearMap.hasFDerivAt` applied twice; no maintained Crouzeix provider and no call to CFT-11-002.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L240).
Type SHA-256: `eee304bebe0283cfc107ba8d4215e91dd315029e444be050a4593d169322f656`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:eee304bebe0283cfc107ba8d4215e91dd315029e444be050a4593d169322f656`.

### CFT-11-E02 — the squared-length cross terms {#exercise-cft-11-e02}

Compute the first-order term of `x ↦ x ⬝ᵥ x`, by collapsing the two cross terms
its expansion produces.

#### Complete written solution

Expanding `(x+w) ⬝ᵥ (x+w)` gives `x ⬝ᵥ x + x ⬝ᵥ w + w ⬝ᵥ x + w ⬝ᵥ w`. Writing
the form through the identity matrix, the two cross terms are `(Ix) ⬝ᵥ w` and
`(Iw) ⬝ᵥ x`. The identity action is trivial, so these are `x ⬝ᵥ w` and `w ⬝ᵥ x`,
and the pairing is symmetric, so both equal `x ⬝ᵥ w` and their sum is
`2(x ⬝ᵥ w)`. The gradient is therefore `2x`. The checked solution rewrites twice
with `Matrix.one_mulVec`, applies `dotProduct_comm`, and closes with `ring`. No
symmetry hypothesis is needed here because the identity is its own transpose;
CFT-11-004 carries the hypothesis because a general `H` needs it.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter11.exercise_02_solution`.
Formal mode: `proved-here`.
Provider boundary: Mathlib `Matrix.one_mulVec` and `dotProduct_comm`; it does not call CFT-11-004 or the `symmetric_cross_terms` lemma.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L248).
Type SHA-256: `919a0d7a2b5ce7dcd805f82f98fabbf4bb07d535a0614c86f4101d78b02f10ca`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:919a0d7a2b5ce7dcd805f82f98fabbf4bb07d535a0614c86f4101d78b02f10ca`.

### CFT-11-E03 — uniqueness of the derivative {#exercise-cft-11-e03}

Prove that a map has at most one Fréchet derivative at a point.

#### Complete written solution

Suppose `f` has derivatives `f'` and `g'` at `x`. Subtracting the two expansions,
`(f'-g')h = r_g(h) - r_f(h)`, whose norm is `o(\lVert h\rVert)`. Evaluate at
`h = tv` for fixed `v` and `t \to 0`: the left side is `t\,(f'-g')v` by
linearity, so `\lVert (f'-g')v\rVert = \lVert r_g(tv)-r_f(tv)\rVert/|t| \to 0`.
The left side does not depend on `t`, so it is zero, and `v` was arbitrary, so
`f' = g'`. The checked solution states both the equality of the bounded maps and
their pointwise agreement, and obtains the first from Mathlib's
`HasFDerivAt.unique` — which runs exactly the argument above — and the second by
rewriting with it.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter11.exercise_03_solution`.
Formal mode: `proved-here`.
Provider boundary: Mathlib `HasFDerivAt.unique`; the pointwise clause is obtained by rewriting. The displayed limit argument is the one that lemma runs, not a separate compiled proof.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L256).
Type SHA-256: `abfa8ccd819e73f74405486d8c103fbb118b86e48caf16903bb393f07825f2ac`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:abfa8ccd819e73f74405486d8c103fbb118b86e48caf16903bb393f07825f2ac`.

### CFT-11-E04 — the chain rule in coordinates {#exercise-cft-11-e04}

Show that composing two matrix actions differentiates to the action of the
product, and record the underlying algebraic identity.

#### Complete written solution

By CFT-11-002 each of `w ↦ Aw` and `w ↦ Bw` is its own derivative. The Fréchet
chain rule composes them, giving the composite derivative
`jacobianAction B ∘L jacobianAction A`. That composite bounded map is
`jacobianAction (BA)`: applying both sides to `v` gives `B(Av)` and `(BA)v`,
equal by the chain kernel, and two bounded maps agreeing everywhere are equal.
The checked solution states the derivative conclusion and the algebraic identity
as two conjuncts, citing `chain_rule_in_coordinates_is_matrix_product` and
CFT-11-006 respectively. The remainder-estimate argument for the chain rule
itself is Mathlib's `HasFDerivAt.comp` and is not re-run here.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter11.exercise_04_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixTextbook.Part02.chain_rule_in_coordinates_is_matrix_product` and `CrouzeixTextbook.Part02.linear_approximation_chain_kernel`; the analytic chain rule descends to Mathlib `HasFDerivAt.comp`.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L265).
Type SHA-256: `18b2bf2941d47ecfdae750b95917aa176758727215a41dfc69a16c8836890653`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:18b2bf2941d47ecfdae750b95917aa176758727215a41dfc69a16c8836890653`.

### CFT-11-E05 — directions without a derivative {#exercise-cft-11-e05}

Give a function on the plane whose directional derivative exists in every
direction at the origin but which is not Fréchet differentiable there.

#### Complete written solution

Take `c(x,y) = x^3/(x^2+y^2)` off the origin and `c(0,0)=0`. First, `c` is
homogeneous of degree one: for `t \neq 0` and `p \neq 0`,
$$
c(tp)=\frac{t^{3}p_0^{3}}{t^{2}(p_0^{2}+p_1^{2})}=t\,c(p),
$$
and the degenerate cases `t = 0` and `p = 0` both give `0 = 0`. Consequently
`t ↦ c(tv)` is the linear function `t ↦ t\,c(v)`, whose derivative at `0` is
`c(v)`: every directional derivative exists and equals `c(v)`.

Second, `c` is not additive on directions: `c(1,1) = 1/(1+1) = 1/2`, while
`c(1,0) = 1` and `c(0,1) = 0`, so `c(1,1) \neq c(1,0)+c(0,1)`.

Third, suppose `c` had a Fréchet derivative `L` at the origin. Composing `L` with
the line `t ↦ tv` shows `t ↦ c(tv)` has derivative `Lv` at `0`; by the second
step it also has derivative `c(v)`; uniqueness of the derivative gives
`Lv = c(v)` for every `v`. But `L` is linear and `c` is not additive, and
evaluating `L` at `(1,1) = (1,0)+(0,1)` contradicts the previous paragraph. So no
such `L` exists.

The checked solution states all three conclusions as conjuncts. The third is
compiled: `coneCusp_not_frechet_differentiable` runs exactly the argument above,
composing through `ContinuousLinearMap.smulRight` for the line, using
`HasFDerivAt.unique` for the uniqueness step, and finishing with `map_add`.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter11.exercise_05_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixTextbook.Part02.coneCusp_homogeneous`, `coneCusp_not_additive` and `coneCusp_not_frechet_differentiable`, all proved in this chapter; Mathlib supplies only `HasFDerivAt.unique` and the line map.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L274).
Type SHA-256: `b32e71ce826ffe4c6fff0733b4eaa5705cc6ca1b94b39fff514e40425e632e01`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:b32e71ce826ffe4c6fff0733b4eaa5705cc6ca1b94b39fff514e40425e632e01`.

### CFT-11-E06 — the adjoint is the transpose, and nothing else {#exercise-cft-11-e06}

Use the duality to explain why forward and reverse mode compute adjoint
contractions, and record the uniqueness that makes "the adjoint" well posed.

#### Complete written solution

Forward mode produces `Av` from a tangent `v`; reverse mode produces `A^{\mathsf T}y`
from a cotangent `y`. Pairing either result against the other argument gives the
same number:
$$
(Av)\cdot y = v\cdot(A^{\mathsf T}y).
$$
So the two modes are two ways to evaluate one bilinear quantity, and which mode is
cheaper depends only on which side is contracted first — the whole cost argument
for reverse mode on scalar losses.

Uniqueness is what makes this an identification rather than a coincidence. If a
backward pass `B` satisfies the same pairing identity for all `v` and `y`, then
testing at `v = e_i`, `y = e_j` reads off `B_{ij} = A^{\mathsf T}_{ij}`, so
`B = A^{\mathsf T}`. The checked solution states both clauses at a fixed `A`,
citing CFT-11-005 for each; the uniqueness proof lives there.

#### Lean correspondence

Public declaration: `CrouzeixTextbook.Part02.Exercises.Chapter11.exercise_06_solution`.
Formal mode: `proved-here`.
Provider boundary: `CrouzeixTextbook.Part02.jvp_vjp_duality`, both clauses; the duality itself descends to CFT-10-001.
Code: [Lean proof](../../../formalization/lean/CrouzeixTextbook/Part02/Chapter11.lean#L281).
Type SHA-256: `d64cf657ca3dc2c206d1bc6bf6b31cba50277007169c9cc35562ec868868ac67`.
Axioms: `Classical.choice`, `Quot.sound`, `propext`.
Compiler receipt: fresh `CrouzeixTextbook` compiler receipt at the exact source locator.
Receipt identity: `CrouzeixTextbook:d64cf657ca3dc2c206d1bc6bf6b31cba50277007169c9cc35562ec868868ac67`.

## Synthesis and forward dependencies

The chapter contributes a small compiled interface: the derivative is unique, a
bounded linear map is its own derivative, derivatives compose, and in coordinates
that composition is matrix multiplication. On top of that sit the two contraction
modes and the fact that the transpose is the unique adjoint.

Chapter 12 keeps the linear-approximation viewpoint and adds orientation, moving
from a single linear map at a point to forms integrated over an oriented
boundary. The adjoint uniqueness proved here is what later lets Part V move a
matrix across a pairing without tracking which side it started on, and the
`coneCusp` boundary is the reminder that directionwise data is weaker than
differentiability — a distinction Part III's analysis chapters depend on.
