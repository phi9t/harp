---
id: cft-chapter-34-lorist-schwenninger-realization
title: The Lorist–Schwenninger realization
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-27
tags: [crouzeix-textbook, constant-two-routes, mathematics, lean]
confidence: high
canonical: 34_lorist_schwenninger_realization.md
chapter: 34
part: 6
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 34: The Lorist–Schwenninger realization

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-vi-constant-two-routes|Part VI — Constant-two routes]]
Previous: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/33_lorist_schwenninger_perturbation_lemma|Chapter 33 — The Lorist–Schwenninger perturbation lemma]]
Next: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/35_comparison_verification_and_boundaries|Chapter 35 — Comparison, verification, and boundaries]]

## Opening problem

The perturbation lemma is abstract. Why does the double-layer power family actually supply its isometry, contraction, uniform perturbations, moment identity, and commutation hypotheses?

## Conceptual model

Build the dilation space as an L² space of boundary vector fields weighted by the PSD double-layer density. A square root of the density embeds the original Euclidean space isometrically. Multiplication by the normalized boundary polynomial is a contraction. Its compressed powers reproduce the positive-map moments; the companion terms become the perturbations.

## Formal development

### The six-item spine

### CFT-34-001 — boundary dilation data {#cft-34-001}

#### Purpose

Motivation. Chapter 33 proved that any `DilationData` record forces the target
operator norm to be at most two. We now build the concrete dilation record
carried by the boundary double-layer density. This is the point where an
abstract recurrence becomes a theorem about a polynomial of a matrix.

#### Statement

Let `E := EuclideanVector n`, `K := L2(μ; EuclideanVector n)`, and let `I`
denote the identity matrix in `SquareMatrix n`. The double-layer construction
begins with the first part of the density,

```text
F(σ) := ((2 * π)^-1 * Γ.speed σ) •
  (Γ.normal σ • (Γ.point σ • I - B)^-1).
D.density σ := F(σ) + F(σ)ᴴ.
```

Thus `F` is exactly `parametricBoundaryFirstPart Γ B`; the second summand is
its pointwise conjugate transpose. Define the remaining objects from these
formulas:

```text
D := parametricPositiveBoundaryDensity Γ B hWB hCauchy
P := polynomialEval q B
h := parametricPolynomialBoundaryFunction Γ q
h(σ) := Polynomial.eval (Γ.point σ) q
V := (boundaryEmbedding D).toContinuousLinearMap
(Vx)(σ) := (Real.sqrt 2)^-1 •
  euclideanOperator (D.density σ)^(1/2) x
Q := bcfMulL h
(Qf)(σ) := h(σ) f(σ)
Φ_D(g) := (1 / 2) ∫ σ, g(σ) • D.density σ ∂μ
C_k := ∫ σ, star ((h^k) σ) • F(σ) ∂μ
E_k := euclideanOperator C_k
DilationData :=
  { T := euclideanOperator P,
    V := V,
    Q := Q,
    perturbation k := E_k,
    bound := ∫ σ, ‖F(σ)‖ ∂μ, ... }
```

Here `D : PositiveBoundaryDensity μ`; its field
`D.density : i → SquareMatrix n` is positive semidefinite almost everywhere,
Bochner integrable, and normalized by

```text
∫ σ, D.density σ ∂μ = 2I.
```

The boundary coordinate is `Γ.point : i → ℂ`. The target
`T := euclideanOperator P : E →L[ℂ] E`, dilation map
`V : E →L[ℂ] K`, multiplier `Q : K →L[ℂ] K`, matrix moment
`Φ_D(h^k) : SquareMatrix n`, companion `C_k : SquareMatrix n`, and
perturbation `E_k : E →L[ℂ] E` now have explicit types. In the Lean
provider, `Q` names the multiplier, not a projection. Writing operator
composition by juxtaposition, the derived operator
`Q_VV* := V V* := V.comp V* : K →L[ℂ] K` is the orthogonal projection onto
`range V`.

#### Hypothesis ledger

The full ambient ledger is long because the construction mixes topology,
measure theory, finite-dimensional linear algebra, and Hilbert adjoints. No
item below is implicit:

1. `i n : Type*`.
2. `TopologicalSpace i`.
3. `CompactSpace i`.
4. `MeasurableSpace i`.
5. `BorelSpace i`.
6. `OpensMeasurableSpace i`.
7. `SecondCountableTopologyEither i ℂ`.
8. `Fintype n`.
9. `DecidableEq n`.
10. `Nonempty n`.
11. `μ : Measure i`.
12. `IsFiniteMeasure μ`.
13. `Ω : Set ℂ`.
14. `EuclideanVector n finite-dimensional complete` with its standard
    complex inner product and a nontrivial vector supplied by `Nonempty n`.
15. `L2(μ; EuclideanVector n) complete` with its standard complex Hilbert
    structure.
16. `Γ : ParametricConvexBoundary Ω`; its continuous fields are
    `Γ.point`, `Γ.normal`, and nonnegative `Γ.speed`, and each boundary point
    has the stated outward support property.
17. `B : SquareMatrix n`.
18. `hWB : numericalRange B ⊆ Ω`.
19. `q : Polynomial ℂ`.
20. `hq : ∀ z ∈ closure Ω, ‖Polynomial.eval z q‖ ≤ 1`.
21. `hCauchy : HasParametricPolynomialCauchyFormula Γ μ B`.

The provider proves
`AEStronglyMeasurable (boundarySquareRoot D) μ`: the boundary square root is
almost-everywhere strongly measurable. This is the right statement because
L2 is a space of equivalence classes modulo almost-everywhere equality; a
representative may be changed on a null set without changing its `L2` class.
It also supplies integrability of `D`, bounded continuity of `h`, and the
completeness needed for `ContinuousLinearMap.adjoint`. These facts are proved
upstream from the displayed assumptions. They are not extra axioms. The last
hypothesis is
the whole Cauchy moment family

```text
∫ σ, r(Γ.point σ) • parametricBoundaryFirstPart Γ B σ ∂μ
  = polynomialEval r B
```

for every polynomial `r`. Its constant case constructs the mass identity;
its power cases construct the perturbation identity.

#### Proof roadmap

First use outward support geometry together with hWB to prove positivity of
the double-layer density. Separately, use the constant Cauchy moment to prove
its mass is `2I`. Take the almost-everywhere measurable positive square root
to define an isometric map into boundary `L²`. Restrict the normalized
polynomial to the boundary and let it act by multiplication. Compress powers
of that multiplier through the embedding. The double-layer identity splits
each compression into an adjoint target power and a companion matrix. Finally
turn that matrix into an operator, bound the operators by one integral, and
prove that they commute with the target.

#### Proof

Write

```text
K := L2(μ; EuclideanVector n)
h := parametricPolynomialBoundaryFunction Γ q
P := polynomialEval q B
V := (boundaryEmbedding D).toContinuousLinearMap
V* := ContinuousLinearMap.adjoint V
Q := bcfMulL h
C_k := ∫ σ, star ((h^k) σ) • F(σ) ∂μ
E_k := euclideanOperator C_k
M := ∫ σ, ‖parametricBoundaryFirstPart Γ B σ‖ ∂μ
T := euclideanOperator P.
```

Before checking the record fields, separate two facts that play different
logical roles. For each `σ`, outward support geometry together with hWB proves
D.density σ is positive semidefinite: the full assumption is
`hWB : numericalRange B ⊆ Ω`. Specifically,
the support inequality controls the real part of the resolvent quadratic
form, while `Γ.speed σ ≥ 0` preserves its sign. The constant Cauchy moment
does not prove positivity. It proves instead

```text
∫ σ, F(σ) ∂μ = I,
∫ σ, F(σ)ᴴ ∂μ = I,
∫ σ, D.density σ ∂μ = 2I.
```

The provider also proves that the square-root field is a.e. strongly
measurable. This is sufficient to form its `L2` equivalence class and avoids
choosing behavior on null sets.

We check the fields of
`LoristSchwenninger.dilationDataOfParametricPolynomial` in the same order as
the `DilationData` structure.

1. `V_isometry`. Almost everywhere,

   ```text
   ‖(Vx)(σ)‖² = (1/2) Re⟪D(σ)x,x⟫.
   ```

   Integrate, move the continuous quadratic-form functional through the
   Bochner integral, and use `∫D = 2I`. This gives
   `‖Vx‖² = ‖x‖²`, hence `‖Vx‖ = ‖x‖`. Polarization upgrades norm
   preservation to inner-product preservation:
   `⟪Vx,Vy⟫ = ⟪x,y⟫`. By the defining identity of the adjoint this is
   `⟪x,V*Vy⟫ = ⟪x,y⟫` for all `x,y`, hence `V*V = I`.

   Put `R := VV*`, the operator denoted `Q_VV*` above. Then

   ```text
   (VV*)* = VV*,
   (VV*)² = V(V*V)V* = VV*,
   range(VV*) = range(V).
   ```

   In particular, `(VV*)² = VV*`.

   The inclusion `range(VV*) ⊆ range(V)` follows from the leading factor
   `V`; the reverse inclusion follows from `VV*(Vx)=V(V*V)x=Vx`. Therefore
   `R` is the orthogonal projection onto `range(V)`; equivalently,
   V V* is the orthogonal projection onto range V. The projection is useful
   intuition, although `DilationData` stores `V` and not this derived field.

2. `Q_norm_le_one`. Pointwise, `(Qf)(σ)=h(σ)f(σ)`, hence
   `‖Qf‖₂ ≤ ‖h‖∞‖f‖₂`. The support field in `Γ` places
   `Γ.point σ` on the boundary of `Ω`, hence in `closure Ω`; `hq` gives
   `‖h‖∞≤1`. Thus `‖Q‖≤1`.

3. `perturbation_eq`. The compression theorem first gives a matrix identity
   interpreted as a continuous operator:

   ```text
   V* Q^k V = euclideanOperator (Φ_D(h^k)).
   ```

   Apply `hCauchy` to the polynomial `q^k` and use
   `q(Γ.point σ)^k = (q^k)(Γ.point σ)`. Splitting
   `D.density=F+Fᴴ` gives the exact matrix equation

   ```text
   2 Φ_D(h^k) = P^k + C_kᴴ.
   ```

   Taking conjugate transposes yields

   ```text
   2 Φ_D(h^k)ᴴ = (Pᴴ)^k + C_k.
   ```

   Taking adjoints of the compression identity and using that
   `euclideanOperator` respects conjugate transpose gives

   ```text
   V* (Q*)^k V = euclideanOperator (Φ_D(h^k)ᴴ).
   ```

   Now apply `euclideanOperator` to the preceding matrix equation and
   rearrange. Because `E_k := euclideanOperator C_k` and
   `T := euclideanOperator P`, the result is the operator identity

   ```text
   E_k = 2 V* (Q*)^k V - (T*)^k,
   ```

   exactly the `perturbation_eq` field required by Chapter 33. At no point is
   the matrix `Φ_D(h^k)` or `C_k` silently identified with an operator; every
   type change is carried by `euclideanOperator`.

4. `bound_nonneg`. Every integrand in `M` is a norm, so
   `0 ≤ M` by `integral_nonneg`.

5. `perturbation_norm_le`. The companion estimate and contractivity of powers
   give

   ```text
   ‖E_k‖
     ≤ ‖h^k‖∞ M
     ≤ 1 · M
     = M.
   ```

   The same `M` works for every `k`; this uniformity is what the Chapter 33
   recurrence needs.

6. `commutes_with_target`. Each boundary companion belongs to the matrix
   algebra generated by `B`. The matrix `P=q(B)` belongs to the same
   commutative one-generator algebra. Therefore `C_k P = P C_k`; applying
   `euclideanOperator` gives `Commute E_k T`.

Every provider hypothesis has now been consumed: compactness and finite
measure give integrability and bounded boundary functions; the Borel and
second-countability instances support measurable `L²` representatives;
finite-dimensional completeness gives matrix adjoints and square roots;
`hWB` supplies resolvents and density positivity; `hq` supplies contraction;
and `hCauchy` supplies mass and every polynomial moment.

**Worked calculation, finite atoms.** The finite atomic boundary model: for i = Fin m with atom weights w_a, the boundary integral becomes ∑ a, w_a • D_a. Assume `w_a≥0`, each `D_a` is positive semidefinite, and

```text
∑ a, w_a • D_a = 2I.
```

Identify the weighted `L²` space with an ordinary direct sum by absorbing the
weights into its coordinates. Then

```text
(Vx)_a := (w_a / 2)^(1/2) D_a^(1/2) x,
(Qf)_a := h_a f_a,
V*(f) = ∑ a, (w_a / 2)^(1/2) D_a^(1/2) f_a.
```

To derive the displayed adjoint, pair `Vx` with `f`, move each self-adjoint
positive square root from the first slot to the second, and collect the sum as
`⟪x,V*f⟫`. This model is fully computable with eigendecompositions of the
positive matrices `D_a`. Substituting `QVx` into the adjoint formula gives

```text
‖Vx‖² = ∑ a, (w_a / 2) ⟪D_a x, x⟫ = ‖x‖²,
V* Q V = euclideanOperator ((1 / 2) ∑ a, w_a h_a D_a).
```

The second formula is the discrete positive-map moment. It makes clear where
the factor `1/2` and the boundary weights go.

#### Boundary case

For the zero polynomial `q = 0`, the boundary function `h`, target `T`, and
every positive power moment vanish; `Q` is the zero multiplier. The package
still exists and the isometry remains the same because it depends on `D`, not
on `q`. This zero polynomial q = 0 case separates the geometry of the boundary
embedding from the dynamics placed on it.

There is also a genuine failure boundary: if ∑ a, w_a • D_a ≠ 2I, then V is
not certified isometric. For an explicit computation, take one atom with
m = 1, w_0 = 1, and D_0 = I. Then the mass is `I`, not `2I`, and the definition
computes

```text
‖Vx‖² = (1 / 2) ‖x‖².
```

Thus `V` shrinks every nonzero vector by `1/√2`; it is not merely missing a
certificate of isometry. Likewise, an empirically positive density without
Bochner integrability and a.e. strong measurability, or an approximate Cauchy
formula, does not inhabit the exact Lean structure.

#### Pedagogical prerequisites

The construction uses the positive density and Cauchy machinery from Chapters
22, 23, and 28, then instantiates the abstract Chapter 33 endpoint. It does not
depend on Chapters 30 through 32 or on the Jin completion route. Within this
chapter, CFT-34-001 is a package whose fields are proved using the provider
lemmas later unpacked as CFT-34-002 through CFT-34-005. That logical packaging
order is why those same-chapter prerequisites point toward field proofs rather
than suggesting a temporal reading order.

#### Lean correspondence

The public definition
`CrouzeixTextbook.Part06.boundary_dilation_data` is in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]]. It
is an exact eta reexport of
`CrouzeixConjecture.LoristSchwenninger.dilationDataOfParametricPolynomial` in
[[formalization/lean/Crouzeix/LoristSchwenninger/ConcreteDilation.lean|ConcreteDilation.lean]].
The coverage contract records kind `definition`, formal_mode `definition`, the
provider declaration, its normalized type, and the standard axioms
`Classical.choice`, `Quot.sound`, and `propext` under target `CrouzeixTextbook`.

The source file formalizes the boundary density, multiplier, compression,
perturbation bound, and commutation proof. It does not formalize the finite
atomic example as a separate theorem; that calculation is a finite model of
the general Bochner-integral construction, not an additional claimed Lean
result.

#### Historical context

This is an LS source-derived row for the Lorist--Schwenninger boundary
realization, the name used in the contract. The inspected mathematical locator is
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124`, registered locally as
`LS-ARXIV-V1`. Harp reproduces its construction in Lean and records the
compiler dependencies. The evidence set contains no peer-review, acceptance, or journal-publication receipt, and this chapter makes no priority claim.

#### ML analogy

Mathematical object / ML counterpart. The closest ML picture is a lifted
feature-space realization. The density square root is a feature map, `V` is
exact whitening, `Q` is a diagonal action in the lifted coordinates, and
`V* Q^k V` is the visible `k`-step moment.

Exact transfer. Once exact matrices `D_a`, weights, and feature values satisfy
the mass and moment equations, the same finite-dimensional linear algebra
checks `V*V=I` and each compression `V* Q^k V`. This transfer is useful for
auditing a proposed operator certificate.

Non-transfer. Learned features do not imply the exact mass or Cauchy moment
identities. Small training loss for finitely many powers does not produce the
infinite exact family, positivity almost everywhere, measurability, or the
commutation theorem.

Diagnostic. On a discretized boundary, report both `‖V*V - I‖` and
`‖V*Q^kV - euclideanOperator (Φ_D(h^k))‖` for every tested `k`, together with
the smallest eigenvalue of each sampled `D_a`. These numbers can reject a
candidate lifted model. They cannot prove the analytic hypotheses when they
are merely small.

### CFT-34-002 — boundary compression first moment {#cft-34-002}

#### Purpose

Motivation. The abstract record from Chapter 33 asks for compressed powers,
but its symbols do not yet say what compression computes. The first boundary
moment is the calculation that makes the dilation visible: multiplication by
`h` in the large `L2` space, followed by compression through `V`, is exactly
the matrix-valued boundary integral `Φ_D(h)`. In ML language, it identifies
the compressed latent action with a feature covariance compression. Its task
is to identify the compressed first boundary moment. We prove the equality
rather than treating it as a diagrammatic slogan.

#### Statement

Let

```text
V : EuclideanVector n →L[ℂ] L2(μ; EuclideanVector n)
  := (boundaryEmbedding D).toContinuousLinearMap,
M_h : L2(μ; EuclideanVector n) →L[ℂ] L2(μ; EuclideanVector n)
  := bcfMulL h,
V* : L2(μ; EuclideanVector n) →L[ℂ] EuclideanVector n
  := ContinuousLinearMap.adjoint V.
```

For positive boundary density D and bounded continuous h, the compressed
operator `V* M_h V` is obtained by first forming the matrix
`boundaryPhiCLM D h`, then applying the matrix-to-operator conversion
`euclideanOperator`:

```text
V* M_h V = euclideanOperator (boundaryPhiCLM D h),
boundaryPhiCLM D h = (1 / 2) ∫ σ, h(σ) • D.density σ ∂μ.
```

The left side is a continuous linear endomorphism of `EuclideanVector n`.
The integral on the right is a `SquareMatrix n`. The theorem does not identify
those types silently; `euclideanOperator` performs the conversion.

#### Hypothesis ledger

The exact ambient assumptions are `i n : Type*`, `MeasurableSpace i`,
`TopologicalSpace i`, `BorelSpace i`, `SecondCountableTopologyEither i ℂ`,
`Fintype n`, `DecidableEq n`, and `μ : Measure i`. The mathematical inputs
are a positive boundary density D, written in Lean as
`D : PositiveBoundaryDensity (n := n) μ`, and a bounded continuous h, written
`h : i →ᵇ ℂ`.

The density record supplies three facts consumed below: `D.density` is
Bochner integrable, is positive semidefinite almost everywhere, and has mass
`∫ σ, D.density σ ∂μ = 2I`. Positivity constructs the measurable square-root
field. Integrability makes the matrix integral and every exchange with a
continuous linear map valid. The mass identity normalizes `V`, although the
compression calculation itself only needs the already-constructed embedding
and the definition of `boundaryPhiCLM`. Bounded continuity of `h` makes
`h(σ)D(σ)` integrable and makes pointwise multiplication a bounded operator
on `L2`.

#### Proof roadmap

Test the two operators on `x` and pair against an arbitrary `y`. Use the
adjoint identity to move `V*` to the first inner-product slot. Expand the `L2`
inner product as an integral, replace both `L2` equivalence classes by their
almost-everywhere representatives, and insert the square root field. Move the
self-adjoint square root across the finite-dimensional inner product and
multiply the two square roots. Finally commute the matrix-to-vector map and
the inner product through the Bochner integral. Equality against every `y`
gives equality of vectors, then continuous-linear-map extensionality gives
the operator identity.

#### Proof

Keep the operator types in view:

```text
V : EuclideanVector n →L[ℂ] L2(μ; EuclideanVector n),
M_h : L2(μ; EuclideanVector n) →L[ℂ] L2(μ; EuclideanVector n),
V* : L2(μ; EuclideanVector n) →L[ℂ] EuclideanVector n.
```

Fix `x y : EuclideanVector n`. Our inner product is conjugate-linear in its
first entry and linear in its second. This convention fixes both the adjoint
order and the location of the scalar `h(σ)`. Begin with

```text
⟪y, (V* M_h V)x⟫
  = ⟪Vy, M_h(Vx)⟫_{L2}
  = ∫ σ, ⟪(Vy)(σ), h(σ)(Vx)(σ)⟫ ∂μ.
```

The first equality is the defining property of `V*`. The second is the `L2`
inner product together with `bcfMulL_apply_ae`. Both are statements about
equivalence classes. To justify the displayed pointwise integrand, use
`boundaryEmbeddingField_memLp` for `x` and `y`; it says that their chosen
representatives agree almost everywhere with

```text
(Vx)(σ) = (Real.sqrt 2)^-1 •
  euclideanOperator (D.density σ)^(1/2) x.
```

On the common full-measure set, the square root is positive semidefinite and
therefore self-adjoint. The provider lemma
`boundarySquareRoot_mul_self_ae` gives

```text
D(σ)^(1/2) D(σ)^(1/2) = D(σ).
```

Move the first square root from the first inner-product entry to the second
by the adjoint identity. Since it is self-adjoint, its conjugate transpose is
itself. The two normalization factors multiply to `1/2`. Linearity in the
second entry places `h(σ)` beside the density. Thus the scalar integrand is

```text
(1 / 2) ⟪y, euclideanOperator (h(σ) • D.density σ) x⟫,
```

and hence

```text
⟪y, (V* M_h V)x⟫
  = (1 / 2) ∫ σ,
      ⟪y, euclideanOperator (h(σ) • D.density σ) x⟫ ∂μ
  = ⟪y, euclideanOperator
      ((1 / 2) ∫ σ, h(σ) • D.density σ ∂μ) x⟫
  = ⟪y, euclideanOperator (boundaryPhiCLM D h) x⟫.
```

The middle step is not an entrywise sleight of hand. For fixed `x`, the map

```text
SquareMatrix n →L[ℂ] EuclideanVector n,
A ↦ euclideanOperator A x
```

is continuous linear, so it commutes with the Bochner integral. The map
`v ↦ ⟪y,v⟫` is continuous linear as well. The last step unfolds
`boundaryPhiCLM_apply` and `boundaryPhi`. Since the equality holds for every
`y`, nondegeneracy gives equality of the two vectors for each `x`.
Extensionality now proves

```text
V* M_h V = euclideanOperator (boundaryPhiCLM D h).
```

This is the promised sequence: expand the L2 inner product, insert the square
root field, use D^(1/2)D^(1/2)=D, then move the integral through the continuous
linear map.

**Worked two-atom model.** Let the boundary have atoms `0,1`, keep their
weights `w₀,w₁` explicit, and write `V₀,V₁` for the unweighted component
maps. Since the multiplier acts diagonally by scalars `h₀,h₁`, direct-sum
multiplication and the adjoint formula give the two-atom compression
calculation: V* M_h V = w₀ V₀* h₀ V₀ + w₁ V₁* h₁ V₁.
Each summand is an endomorphism of the original space. This is the finite-sum
version of moving the matrix-valued Bochner integral through
`euclideanOperator`.

**Worked scalar Fourier/Cauchy moment.** On the positively oriented unit
circle, write a scalar polynomial as

```text
p(z) = ∑_{m=0}^N a_m z^m.
```

Then

```text
(1 / (2πi)) ∮_{|z|=1} p(z) / z dz
  = (1 / (2πi)) ∑_{m=0}^N a_m ∮_{|z|=1} z^(m-1) dz.
```

Here `∮_{|z|=1} z^-1 dz = 2πi`, whereas
`∮_{|z|=1} z^(m-1) dz = 0 for m ≥ 1`. The reason is elementary: all
integer powers `z^r`, with `r ∈ ℤ` and `r ≠ -1`, have the single-valued primitive
`z^(r+1)/(r+1)` on the punctured plane `ℂ \ {0}`, which is an open
neighborhood of the unit circle,
while `z^-1` has no such primitive around a loop enclosing zero. Therefore
only the exponent -1 mode survives, and the normalized integral is
`p(0) = a_0`. This is the scalar Fourier version of Cauchy coefficient
selection. In the matrix-valued boundary proof, the Cauchy hypothesis plays
the same coefficient-selection role after the density is split into its
holomorphic and adjoint parts. The compression identity above is more basic:
it is valid for every `PositiveBoundaryDensity D` and does not assume that
`D` came from that concrete split.

#### Boundary case

For the constant multiplier h = 0, `M_h=0`, the matrix integral is zero, and
both sides of the compression identity are the zero operator. This checks the
scalar placement and the `1/2` normalization without using cancellation.
At the opposite boundary, if `h` is merely a sampled function with no bounded
continuous representative, pointwise multiplication has not been shown to
define the continuous operator `M_h` used by the theorem.

#### Pedagogical prerequisites

The proof uses CFT-22-004 for matrix-valued positive integration and
CFT-23-001 for compression and adjoints. CFT-34-001 supplies the concrete
embedding. It does not use the Jin route in Chapters 30 through 32, the
Chapter 33 terminal norm bound, or any later realization result.

#### Lean correspondence

Coverage classifies this card as a theorem with formal_mode
`reexported-proof`. The public declaration is
`CrouzeixTextbook.Part06.boundary_compression_first_moment` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean]].
Its exact provider is
`CrouzeixConjecture.LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_comp_boundaryEmbedding`
in
[[formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean|formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean]].
The compiler receipt records no project axiom and exactly the standard axioms
`Classical.choice`, `Quot.sound`, and `propext`. The Lean theorem formalizes
the operator identity and all type conversions. It does not formalize the
polynomial Fourier paragraph as part of that theorem; Exercise 34-E03 checks
both the scalar circle-integral kernel and its finite-sum coefficient-selection
corollary separately.

#### Historical context

This is an LS source-derived first compression moment. Its source locator is
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124`. The captured-byte
identity and digest are in
[[evidence/crouzeix_conjecture/source_manifest.tsv|the Crouzeix source manifest]],
and [[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json|the LS source graph]]
records the locator chain.
Harp reconstructs the measure-theoretic calculation in Lean. This description
makes no priority claim and no peer-review, acceptance, or journal-publication
claim.

#### ML analogy

Mathematical object / ML counterpart. The map `V` is an exact feature lift,
`M_h` is a diagonal latent action, and `V* M_h V` is the corresponding feature
covariance compression.

Exact transfer. If a model supplies the exact square-root and mass identities,
the compressed linear action can be evaluated either in feature space or by
the smaller matrix moment. The equality is algebraic, not asymptotic.

Non-transfer. A learned feature map need not be isometric, and a minibatch
covariance is not a Bochner integral identity. Approximate quadrature,
estimated density, or finite training data does not imply exact compression.

Diagnostic. Compute
`R_1 := V* M_h V - euclideanOperator (boundaryPhiCLM D h)` and report
`‖R_1‖`. A large value rejects the proposed moment model. A small value only
measures the chosen discretization and sample.

Formal boundary. Lean proves `R_1=0` from the analytic hypotheses. It does not
turn a sampled residual into an exact theorem.

### CFT-34-003 — boundary compression power moments {#cft-34-003}

#### Purpose

Motivation. Chapter 33 does not study one application of the latent
multiplier. Its recurrence compares arbitrary powers of the target and the
dilation. We therefore upgrade the first moment to every power by upgrading
the first compression identity to every power.
This is power-moment compression, and its ML counterpart is an exact family of
multi-step rollout moments.

#### Statement

Under the same boundary assumptions, add `k : Nat`. For positive boundary
density D and bounded continuous h, every natural power satisfies

```text
V* (M_h)^k V = euclideanOperator (boundaryPhiCLM D (h^k)).
```

The notation distinguishes three operations. `(M_h)^k` is a power in the
algebra of continuous linear endomorphisms of boundary `L2`. The function
`h^k : i →ᵇ ℂ` is a pointwise power in the bounded-continuous-function
algebra. The expression `boundaryPhiCLM D (h^k)` is a matrix, and
`euclideanOperator` converts it to a continuous linear map. The assertion is
for every k : Nat, including the base case `k=0`.

#### Hypothesis ledger

The exact hypotheses are a positive boundary density D, a bounded continuous
h, and k : Nat. Expanded to their Lean types, these are the same ambient
instances as CFT-34-002, together with
`D : PositiveBoundaryDensity (n := n) μ`, `h : i →ᵇ ℂ`, and `k : ℕ`.
No norm bound on `h` is needed for the algebraic identity. Positivity,
integrability, and mass normalization enter through the first-moment theorem.
Bounded continuity is closed under powers, so `h^k` is again a legal input to
that theorem.

#### Proof roadmap

First prove the multiplier power law by induction on `k`. The base is the
constant-one multiplier. In the successor step use pointwise multiplication
of bounded continuous functions and composition of their multipliers. This is
the provider theorem `bcfMulL_pow`. Rewrite the operator power using that
identity, then apply the first-moment compression theorem with `h^k` in place
of `h`. Keep the matrix and continuous-linear-map types visible throughout.

#### Proof

The algebraic bridge is

```text
bcfMulL_pow : M_(h^k) = M_h^k.
```

It is proved by induction. For k = 0,

```text
M_h^0 = I = M_(h^0),
```

because `h^0` is the constant-one bounded continuous function and
`bcfMulL_one` identifies its multiplier with the identity. Suppose
`M_h^k=M_(h^k)`. Then

```text
M_h^(k+1) = M_h^k M_h
           = M_(h^k) M_h
           = M_(h^k h)
           = M_(h^(k+1)).
```

The third equality is `bcfMulL_mul`, proved by continuous-linear-map
extensionality from pointwise scalar multiplication. The first and last are
`pow_succ`; this fixes the product order. Scalars commute here, but the proof
does not rely on silently reversing operator composition. This completes
`bcfMulL_pow` for every natural `k`, hence for every k : Nat.

Now rewrite bcfMulL_pow in the compression:

```text
V* M_h^k V = V* M_(h^k) V.
```

Next, apply the first-moment compression identity to h^k:

```text
V* M_(h^k) V = euclideanOperator (boundaryPhiCLM D (h^k)).
```

Transitivity gives

```text
V* M_h^k V = euclideanOperator (boundaryPhiCLM D (h^k)).
```

The left side is a continuous linear map. On the right, the matrix
boundaryPhiCLM D (h^k) becomes the continuous linear map
euclideanOperator (boundaryPhiCLM D (h^k)). This explicit conversion is why
the theorem elaborates without conflating matrices with operators.

We must now explain why k=1 alone is insufficient. The concrete
`DilationData.perturbation_eq` field is
quantified over all `n`. At recurrence index `n`, Chapter 33 substitutes the
compression formulas with k = n and k = n + 1, then forms terms involving
`(Q*)^n` and `(T*)^n`; the identity at k = 1 cannot instantiate this
quantified field when `n>1`. The Chapter 33 recurrence therefore consumes all
positive powers, not a one-step covariance. More precisely, the telescoping
sum starts at n = 1, so it never consumes the zero-th compression moment.
The statement at k = 0 is only the multiplier-induction base and unitality
check: it verifies that the multiplier representation and compression preserve
the unit. It is not a base term of Chapter 33's telescoping argument.

**Worked empirical moment residual R_k := V* M_h^k V - Φ(h^k).** Given a
finite quadrature model, define

```text
R_k := V* M_h^k V - Φ(h^k),
```

where this displayed `Φ(h^k)` abbreviates the operator
`euclideanOperator (boundaryPhiCLM D (h^k))`. Calculate `‖R_k‖` for each
sampled k. This residual tests the chosen horizon, not
the infinite family. For example, if `V=I` and `M_h=diag(α,β)`, the exact moments are
`diag(α^k,β^k)`, so any proposed compressed moment matrix has an entrywise
check. This finite computation is useful for debugging. It cannot establish
the theorem for untested powers or replace the analytic boundary hypotheses.

#### Boundary case

At power k = 0, the formula reads

```text
V* I V = euclideanOperator (boundaryPhiCLM D 1).
```

The left side is `V*V=I` because the embedding is isometric. The right side is
`(1/2)∫D=I` by the mass normalization. Thus the zero-th moment is not a
vacuous special case; it checks that the lift and positive map share the same
unit.

#### Pedagogical prerequisites

The card uses CFT-28-001 for the full power family, CFT-34-002 for first
compression, and the multiplier algebra exposed here and registered as
CFT-34-004. This pedagogical edge does not mean CFT-34-004 is complete in the
current phase. The compiled provider imports its already-proved
`bcfMulL_pow` lemma. No Jin terminal theorem and no Chapter 33 norm endpoint
appears in the proof term.

#### Lean correspondence

Coverage classifies this card as a theorem with formal_mode
`reexported-proof`. The public declaration
`CrouzeixTextbook.Part06.boundary_compression_power_moments` is in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean]].
Its exact provider is
`CrouzeixConjecture.LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding`
in
[[formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean|formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean]].
The induction bridge `bcfMulL_pow` is in
[[formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean|formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean]].
The compiler records exactly `Classical.choice`, `Quot.sound`, and `propext`
and no project axiom. Exercise 34-E04 proves the applied identity from the two
smaller provider lemmas; it does not invoke this parent power-moment theorem.

#### Historical context

This LS source-derived power-moment compression reconstructs the full moment
family used in the Lorist--Schwenninger boundary realization. The captured
source locator is
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124`. Its captured-byte
digest is registered in
[[evidence/crouzeix_conjecture/source_manifest.tsv|source_manifest.tsv]], and
[[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json|the LS source graph]]
records the exact source interval.
The local term power-moment compression describes the formal factorization;
it is not a historical priority claim. There is no local receipt for
peer-review, acceptance, or journal publication.

#### ML analogy

Mathematical object / ML counterpart. `M_h^k` is the k-step action of one tied
linear latent transition, and compression gives exact multi-step rollout
moments in the observed space.

Exact transfer. For a tied linear transition, an exact isometric feature map,
and the exact first-moment density identity
`V* M_g V = euclideanOperator (boundaryPhiCLM D g)` for every bounded
continuous `g`, the same induction that proves `M_h^k=M_(h^k)` validates every
compressed rollout moment. Tied powers and isometry alone do not imply this
compression identity.

Non-transfer. The untied, nonlinear, stochastic, or time-varying rollouts do not
obey this power law without additional state augmentation or probabilistic
hypotheses. Matching one-step behavior does not force multi-step moments.

Diagnostic. Form
`R_k := V* M_h^k V - euclideanOperator (boundaryPhiCLM D (h^k))` and
calculate ‖R_k‖ for each sampled k. Plotting or tabulating these norms can
expose error growth with horizon.

Formal boundary. Lean proves the equality for every natural `k` from exact
hypotheses; no finite residual table proves the infinite exact family.

### CFT-34-004 — boundary multiplier powers {#cft-34-004}

#### Purpose

Motivation. The compression theorem of CFT-34-003 needs a bridge between two
different powers. One is the pointwise power `h^k` of a scalar boundary
function. The other is the composition power `(M_h)^k` of an operator on
boundary `L2`. We now prove that these constructions agree. The purpose is to
turn multiplier products into powers and make the recurrence in Chapter 33
applicable without a hidden change of type.

#### Statement

Let `h : i →ᵇ ℂ` be a bounded continuous h and let `k : Nat`. On
`L2(μ; EuclideanVector n)`, write

```text
M_h := bcfMulL h.
```

Then

```text
M_(h^k) = (M_h)^k.
```

In Lean's concrete names, the two sides are
`bcfMulL (h ^ k)` and `(bcfMulL h) ^ k`. The left power lives in the
commutative algebra of bounded continuous functions. The right power lives
in the generally noncommutative algebra of continuous linear endomorphisms.
The theorem says that pointwise multiplication represents the former algebra
inside the latter.

#### Hypothesis ledger

The exact mathematical inputs are bounded continuous h and k : Nat. Expanded
to the public Lean declaration, they are:

1. `i n : Type*`;
2. `MeasurableSpace i`, `TopologicalSpace i`, and `BorelSpace i`;
3. `SecondCountableTopologyEither i ℂ`;
4. `Fintype n`;
5. `μ : Measure i`;
6. `h : i →ᵇ ℂ`;
7. `k : ℕ`.

There is no positivity assumption and no bound `‖h‖≤1`. Those belong to the
contractivity theorem, not to this algebraic identity. The measure need not
be finite. Bounded continuity supplies measurability and a uniform scalar
bound, which is enough for multiplication to preserve `L2`.

#### Proof roadmap

Start with base k=0 and identify multiplication by the constant-one function
with the identity operator. For a successor, expand both function and
operator powers with `pow_succ`. Convert multiplication of functions to
composition of multipliers using `bcfMulL_mul`, and insert the induction
hypothesis. Finally explain how `bcfMulL_mul` itself descends from the
pointwise identity to equality of `L2` classes and then to
continuous-linear-map extensionality.

#### Proof

The base k=0 has two separately typed equalities:

```text
bcfMulL (h ^ 0) = bcfMulL 1 = 1,
(bcfMulL h) ^ 0 = 1.
```

Thus `bcfMulL (h ^ 0) = 1` and `(bcfMulL h) ^ 0 = 1`. The lemma
`bcfMulL_one` proves the only non-formal step: multiplication by the constant
function one acts as the identity on every `L2` equivalence class.

Assume the induction hypothesis

```text
bcfMulL (h ^ k) = (bcfMulL h) ^ k.
```

Use `pow_succ` first in the function algebra:

```text
bcfMulL (h ^ (k + 1))
  = bcfMulL (h ^ k * h).
```

Now use `bcfMulL_mul` and the induction hypothesis:

```text
bcfMulL (h ^ k * h)
  = bcfMulL (h ^ k) * bcfMulL h
  = (bcfMulL h) ^ k * bcfMulL h
  = (bcfMulL h) ^ (k + 1).
```

The last equality is `pow_succ` in the operator algebra. This completes the
induction.

It remains worth opening `bcfMulL_mul`, since this is where a paper proof can
hide two quotient steps. For `f : L2(μ; EuclideanVector n)`, both sides have
representatives satisfying, almost everywhere,

```text
M_(h g) f (σ) = (h(σ)g(σ)) • f(σ),
(M_h M_g f)(σ) = h(σ) • (g(σ) • f(σ)).
```

Associativity of scalar multiplication makes these representatives equal
almost everywhere. `Lp.ext` turns that almost-everywhere equality into
equality in `L2`. `ContinuousLinearMap.ext` then proves equality of the
bundled operators. In short, the equality passes through bounded continuous
functions, pointwise representatives, `L2` quotient classes, and finally
continuous-linear-map extensionality. None of those levels may be silently
identified with another.

For k=2 the whole mechanism is visible on one vector:

```text
M_h^2 f = h • (h • f) = h^2 • f.
```

This is the worked k = 2 multiplication: M_h^2 f = h • (h • f) = h^2 • f.
It is the finite-dimensional intuition, but
the Lean proof works for arbitrary boundary measure spaces and every natural
power.

#### Boundary case

The base case k = 0 reads `M_1=I`. It checks that the multiplier map is
unital. This matters at the type level: `h^0` is the constant-one bounded
continuous function, whereas `(M_h)^0` is the identity continuous linear map.

#### Pedagogical prerequisites

CFT-08-003 supplies powers in an algebra, CFT-28-003 supplies continuous
linear maps on `L2`, and CFT-34-003 shows where the power bridge enters the
compression theorem. The proof does not use positivity of the boundary
density, the perturbation lemma, Jin's route, or any terminal constant-two
theorem.

#### Lean correspondence

Coverage classifies this theorem as an exact `reexported-proof`. The stable
public declaration
`CrouzeixTextbook.Part06.boundary_multiplier_powers` is in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]].
Its exact provider is
`CrouzeixConjecture.LoristSchwenninger.bcfMulL_pow` in
[[formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean|BoundaryMultiplier.lean]].
That provider proves the result from `bcfMulL_one` and `bcfMulL_mul`, whose
body contains both `Lp.ext` and `ContinuousLinearMap.ext`. The compiled
receipt records the standard axioms `Classical.choice`, `Quot.sound`, and
`propext`, with no project axiom.

#### Historical context

This is LS source-derived boundary multiplier algebra. The exact captured
locator is `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124`. The byte
identity is recorded in
[[evidence/crouzeix_conjecture/source_manifest.tsv|the source manifest]], and
[[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json|the LS source graph]]
records its route position. Harp expands the representation argument because
the source can use standard multiplier notation without discussing Lean's
`L2` quotient. This is a reconstruction claim, not a priority or publication
claim.

#### ML analogy

Mathematical object / ML counterpart. Boundary multiplication is a diagonal
lifted feature action. Reusing `M_h` at each step is tied linear-layer
composition, while `h^k` multiplies each lifted coordinate by its k-step
scalar response.

Exact transfer. A fixed diagonal linear layer satisfies the same identity:
applying it k times equals a diagonal layer whose entries are the k-th powers
of the original entries. The transfer is exact when the feature coordinates
and the layer are fixed across steps.

Non-transfer. Untied layers, nonlinear activations, state-dependent gates,
and stochastic updates do not satisfy `M^k=M_(h^k)` without further
hypotheses. A product `M_{h_k}⋯M_{h_1}` is not a power unless the same
multiplier repeats.

Diagnostic. For a proposed tied diagonal lift, compare an explicitly
composed k-step matrix with the diagonal matrix formed from coordinatewise
k-th powers. A nonzero difference rejects the tied-multiplier model. A zero
finite computation checks only the sampled representation, not the analytic
boundary construction.

### CFT-34-005 — boundary multiplier contractive {#cft-34-005}

#### Purpose

Motivation. Chapter 33 requires the dilation operator `Q` to be a contraction.
In the concrete realization, `Q=M_h`. The proof should use only the boundary
supremum bound for `h`; none of the matrix geometry is needed. This card will
certify contraction before dilation packages the other fields.

#### Statement

For bounded continuous h on the boundary, the multiplier first satisfies the
sharper estimate

```text
norm (M_h) <= norm h,
```

or, in mathematical notation,

```text
‖M_h‖ ≤ ‖h‖∞.
```

Under the added hypothesis norm h <= 1, transitivity gives

```text
norm (M_h) <= 1.
```

No equality claim is needed. The exact theorem used by the concrete
`DilationData` constructor is `bcfMulL_norm_le_one`.

#### Hypothesis ledger

The exact inputs are bounded continuous h and norm h <= 1. The elaborated
ambient assumptions are `i n : Type*`, `MeasurableSpace i`,
`TopologicalSpace i`, `BorelSpace i`, `SecondCountableTopologyEither i ℂ`,
`Fintype n`, `μ : Measure i`, `h : i →ᵇ ℂ`, and `hh : ‖h‖ ≤ 1`.
Neither a positive boundary density nor finite measure is used. The scalar
function is bounded because it belongs to `i →ᵇ ℂ`; `hh` supplies the unit
bound needed by Chapter 33.

#### Proof roadmap

Begin with the pointwise norm bound for scalar multiplication. Integrate its
square through the `L2` norm to obtain the Lp norm bound on every input.
Pass from that vector estimate to the operator norm. Finish by transitivity
with the hypothesis `‖h‖∞≤1`. Then isolate the strict case `‖h‖∞≤ρ<1`.

#### Proof

For almost every boundary point `σ`,

```text
‖h(σ)f(σ)‖ = ‖h(σ)‖ ‖f(σ)‖
             ≤ ‖h‖∞ ‖f(σ)‖.
```

The first equality is the norm rule for scalar multiplication. The inequality
uses `‖h(σ)‖ ≤ ‖h‖∞`. This is the pointwise norm bound.

The `Lp` comparison theorem integrates the pointwise estimate and gives the
Lp norm bound

```text
‖M_h f‖₂ ≤ ‖h‖∞ ‖f‖₂.
```

Because this holds for every `f`, the constructor
`LinearMap.mkContinuous` and its norm theorem yield

```text
‖M_h‖ ≤ ‖h‖∞.
```

Finally, use `‖h‖∞ ≤ 1` and transitivity:

```text
‖M_h‖ ≤ ‖h‖∞ ≤ 1,
```

so `‖M_h‖ ≤ 1`. For Chapter 33, equality is unnecessary. The perturbation lemma consumes
only this upper bound and never asks for a vector at which the multiplier
norm is attained.

The strict case is equally direct. If `‖h‖∞ ≤ ρ` for a real number `ρ < 1`,
then

```text
‖M_h f‖₂ ≤ ρ ‖f‖₂,
‖M_h‖ ≤ ρ < 1.
```

The worked bound ‖h‖ = ρ < 1 gives ‖M_h f‖ ≤ ρ ‖f‖, a genuine
strict-contraction certificate.
It does not improve the universal factor two in Chapter 33, because that
abstract endpoint was designed to use the weaker information `‖Q‖≤1`.

#### Boundary case

The strict contraction ‖h‖ < 1 gives ‖M_h‖ < 1 by choosing, for example,
`ρ=‖h‖∞` in the displayed estimate. At the other boundary, `‖h‖∞=1` gives
only `‖M_h‖≤1`. Equality can fail when the essential support of the measure
does not see points where the continuous sup norm is attained.

#### Pedagogical prerequisites

CFT-12-002 supplies operator norms, CFT-13-002 supplies `L2` norm comparison,
and CFT-34-004 identifies powers of the multiplier. The estimate itself does
not use CFT-34-004, but the concrete dilation uses both statements. No
compression identity and no Chapter 33 terminal theorem enters this proof.

#### Lean correspondence

Coverage classifies the card as an exact theorem in `reexported-proof` mode.
The public declaration
`CrouzeixTextbook.Part06.boundary_multiplier_contractive` is in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]].
Its provider
`CrouzeixConjecture.LoristSchwenninger.bcfMulL_norm_le_one` is in
[[formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean|BoundaryMultiplier.lean]].
The provider applies `bcfMulL_norm_le` and transitivity. Exercise 34-E05 has a
different type: it proves the bound after applying the multiplier to one
`L2` vector and calls `bcfMulL_norm_le`, not this parent theorem. The receipt
records only `Classical.choice`, `Quot.sound`, and `propext`.

#### Historical context

This contractive boundary multiplier is LS source-derived. The precise
captured span is `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124`.
The source manifest and
[[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json|the LS source graph]]
bind that span to the local provider. The pointwise-to-`L2` norm chain is
expanded here for teaching. No claim is made about priority, peer review, or
journal status.

#### ML analogy

Mathematical object / ML counterpart. `M_h` is a diagonal feature-space
layer, and `‖h‖∞` is its coordinatewise Lipschitz certificate.

Exact transfer. For an exactly diagonal complex linear layer, the largest
coordinate modulus bounds the layer's `L2` operator norm. If every coordinate
has modulus at most `ρ<1`, repeated applications contract by at most `ρ^k`.

Non-transfer. A learned layer that is only approximately diagonal, a
nonlinear block, or an empirical spectral normalization estimate does not
inherit this exact operator bound. Off-diagonal coupling can increase the
norm even when all displayed diagonal entries are small.

Diagnostic. Apply the implemented lifted layer to adversarial or singular
vectors and compare `‖M_h f‖/‖f‖` with the declared `ρ`. A violation refutes
the Lipschitz certificate. Passing finitely many vectors does not prove the
operator-norm inequality.

### CFT-34-006 — realization norm two {#cft-34-006}

#### Purpose

Motivation. The preceding cards have built every field required by Chapter
33. We now feed the realization into the Chapter 33 endpoint and identify its
abstract target with the concrete polynomial evaluation. This is the
factor-two realization endpoint for one parametrized convex boundary.

#### Statement

Let `Γ : ParametricConvexBoundary Ω`, `B : SquareMatrix n`, and
`q : Polynomial ℂ`. Assume W(B) subset Omega, written
`hWB : numericalRange B ⊆ Ω`; assume norm q <= 1 on closure Omega, written

```text
hq : ∀ z ∈ closure Ω, ‖Polynomial.eval z q‖ ≤ 1;
```

and assume the power Cauchy formula
`hCauchy : HasParametricPolynomialCauchyFormula Γ μ B`. Then

```text
norm (euclideanOperator (polynomialEval q B)) <= 2.
```

Equivalently,

```text
‖euclideanOperator (polynomialEval q B)‖ ≤ 2.
```

This is a normalized fixed-domain result. It is the exact target of the public
Lean theorem. The final unnormalized and outer-domain statement belongs to
Chapter 35.

#### Hypothesis ledger

The full ambient assumptions are the same as CFT-34-001: `i n : Type*`,
compact measurable boundary index `i` with its Borel and countability
instances, finite nonempty decidable index `n`, finite measure `μ`, set
`Ω : Set ℂ`, parametrized convex boundary `Γ`, matrix `B`, polynomial `q`,
and the three mathematical hypotheses below.

1. W(B) subset Omega: `hWB : numericalRange B ⊆ Ω`.
2. norm q <= 1 on closure Omega:
   `hq : ∀ z ∈ closure Ω, ‖Polynomial.eval z q‖ ≤ 1`.
3. power Cauchy formula:
   `hCauchy : HasParametricPolynomialCauchyFormula Γ μ B`.

The topological and measure instances make the boundary integrals and `L2`
space legal. `Nonempty n` makes the Euclidean space nontrivial, which Chapter
33 needs for norm attainment. No simple-spectrum assumption appears in this
fixed-domain realization.

#### Proof roadmap

First construct dilationDataOfParametricPolynomial from the three concrete
hypotheses. Separate its target identification, which is definitional data,
from its six proof fields, and map every proof field to CFT-34-001 through
CFT-34-005.
Then apply Chapter 33 perturbation endpoint
`DilationData.norm_target_le_two`. Finally identify the normalized polynomial
evaluation stored in `data.T`. Afterward, keep separate the algebra that
undoes normalization for a polynomial with positive closed-domain maximum and
the polynomial-identity argument for the zero-maximum branch.

#### Proof

We first construct dilationDataOfParametricPolynomial from the concrete
boundary inputs. Set

```text
data := dilationDataOfParametricPolynomial Γ B hWB q hq hCauchy.
```

The constructor does not merely remember the hypotheses. Its target
identification is definitional data, while the remaining assertions are six
proof fields.

```text
Target identification (definitional data):
  data.T = euclideanOperator (polynomialEval q B).

Six proof fields:
data.V_isometry,
data.Q_norm_le_one,
data.perturbation_eq,
data.bound_nonneg,
data.perturbation_norm_le,
data.commutes_with_target.
```

Here is the exact hypothesis map.

1. `data.V_isometry` is the isometry theorem for the boundary embedding from
   CFT-34-001. Its proof uses positivity to construct the square-root field
   and the normalized mass identity to show `V*V=I`.
2. `data.Q_norm_le_one` is CFT-34-005 applied to the boundary polynomial
   multiplier. Its bound comes from `hq` restricted along the
   boundary coordinate.
3. `data.perturbation_eq` follows from CFT-34-002, CFT-34-003, and CFT-34-004
   together with the power Cauchy identity, for every natural power.
4. `data.bound_nonneg` is nonnegativity of the integral of the pointwise
   norm of the first boundary part.
5. `data.perturbation_norm_le` follows from the companion integral estimate
   and `‖h^k‖∞≤1`, with the same bound for every `k`.
6. `data.commutes_with_target` follows because every companion belongs to the
   one-generator algebra of `B`; applying `euclideanOperator` preserves the
   required multiplication identity.

The constructor also defines `V`, `Q`, the perturbation sequence, and its
common bound. Those are data fields, not additional proof obligations. This
is why the ledger has exactly six proof fields rather than seven.

Now apply Chapter 33 perturbation endpoint:

```text
data.norm_target_le_two : ‖data.T‖ ≤ 2.
```

That endpoint has two internal branches. If `‖data.T‖≤1`, the desired result
is immediate. If `1<‖data.T‖`, finite-dimensional norm attainment selects a
unit singular vector, Equations (3) and (4) squeeze its displacement, and the
scalar contradiction forces `‖data.T‖≤2`. Thus both the small-norm branch and
the nontrivial recurrence branch are covered.

It remains to identify the normalized polynomial evaluation. By the
definition of the constructor,

```text
data.T = euclideanOperator (polynomialEval q B).
```

Changing the target in `data.norm_target_le_two` gives exactly

```text
‖euclideanOperator (polynomialEval q B)‖ ≤ 2.
```

No Jin completion theorem is imported. The proof term ends at the LS
perturbation endpoint from Chapter 33.

For later use, here is the complete normalization algebra under the standard
bounded-domain hypotheses. Assume now that `Ω` is nonempty and open and that

```text
K := closure Ω
```

is compact (and therefore, since `Ω` is nonempty, `K` is compact and
nonempty). Thus K is compact and nonempty. For `p : Polynomial ℂ`, define the
real value set and its supremum by

```text
S_p := {t : ℝ | ∃ z ∈ K, t = ‖Polynomial.eval z p‖},
m := sSup S_p.
```

The map `z ↦ ‖Polynomial.eval z p‖` is continuous. Its image on compact `K`
is compact; because `K` is nonempty, S_p is nonempty and bounded above.
Consequently m is finite, nonnegative, and attained, and
`‖Polynomial.eval z p‖≤m` for every `z∈K`. These are exactly the
compactness, nonemptiness, boundedness, and attainment facts used below.

In the branch `m > 0`, put `q := m⁻¹ • p`. Then `‖q(z)‖≤1` on `K`, and
linearity of polynomial evaluation gives

```text
polynomialEval q B = m⁻¹ • polynomialEval p B.
```

Applying the normalized theorem and multiplying by `m` yields

```text
‖euclideanOperator (polynomialEval p B)‖ ≤ 2 * m.
```

This is how one undo normalization step produces the factor-two inequality.
The branch `m = 0` cannot divide by `m`, but it is not left to a later
chapter. Since every value norm on `K` lies between zero and `m`, p vanishes
on K. In particular it vanishes on `Ω⊆K`. Because Ω is nonempty and open,
the polynomial identity theorem gives p = 0. Therefore

```text
m = 0 ⇒ p vanishes on K ⇒ p = 0,
polynomialEval p B = 0,
‖euclideanOperator (polynomialEval p B)‖ = 0 ≤ 2 * m.
```

Thus both `m > 0` and `m = 0` give

```text
‖euclideanOperator (polynomialEval p B)‖ ≤ 2 * m.
```

The Lean provider `realization_norm_two` proves only the normalized theorem
for `q`; it does not package this unnormalization argument. In particular, the
zero branch here is a transparent prose corollary using the polynomial
identity theorem, not a hidden call to the provider or a delegated branch.
Chapter 35 will formalize the passage among admissible outer domains.

**Nonnormal 2 × 2 worked matrix.** This is the nonnormal 2 × 2 worked matrix:
evaluate q(B), then compare its norm with the sampled boundary maximum, while
keeping the outer domain admissibly open. Take

```text
A_{0,2} = [[0, 2], [0, 0]],    p(z)=z.
```

Its numerical range is the closed unit disk. A closed disk is not an
admissible open outer domain, so fix `r > 1` and set

```text
Ω_r := {z : ℂ | ‖z‖ < r},
W(A_{0,2}) = {z : ℂ | ‖z‖ ≤ 1} ⊂ Ω_r,
q_r(z) := z / r.
```

To invoke the fixed-domain theorem, the boundary data must also be present.
Fix the usual positively oriented circle parametrization and its compatible
finite boundary measure, and record the two provider inputs

```text
Γ_r : ParametricConvexBoundary Ω_r,
μ_r : Measure (parameter space of Γ_r),
hW_r : numericalRange A_{0,2} ⊆ Ω_r,
hCauchy_r : HasParametricPolynomialCauchyFormula Γ_r μ_r A_{0,2}.
```

Here `hW_r` is the displayed strict inclusion, while `hCauchy_r` is the full
polynomial Cauchy-moment family for the circle. The check that `q_r` has
supremum norm one on `closure Ω_r` is necessary but not sufficient by itself:
the application also consumes `Γ_r`, `μ_r`, `hW_r`, and `hCauchy_r`. With
those hypotheses explicit, the fixed-domain theorem gives

```text
‖q_r(A_{0,2})‖ = 2 / r ≤ 2.
```

Undoing this normalization gives the legitimate outer-domain estimate

```text
‖A_{0,2}‖ ≤ 2 * r.
```

For every fixed `r>1` this bound is not the equality case: direct calculation
gives `‖A_{0,2}‖=2`, while the right side is `2r>2`. The sharp boundary value
at `r=1` is obtained only after the Chapter 35 outer-domain limit `r ↓ 1`.
CFT-34-006 itself neither substitutes the inadmissible closed unit disk for
`Ω` nor claims equality at `r=1`. The full verification of the numerical
range appears in Chapter 32.

#### Boundary case

For q = 0, `polynomialEval q B=0`, so the operator norm is zero and the bound
is immediate. This is the card's boundary case q = 0 gives operator norm zero.
For a nonzero normalized polynomial whose induced operator happens to satisfy
`‖q(B)‖≤1`, Chapter 33 exits through its small-norm branch without using the
scalar contradiction.

#### Pedagogical prerequisites

The proof depends on the complete Chapter 33 perturbation lemma, especially
CFT-33-006, and on CFT-34-001 through CFT-34-005. CFT-29-001 supplies the
normalization target, and Chapter 32 supplies the sharp nilpotent example.
These are pedagogical dependencies. The compiled LS route does not depend on
the Jin providers from Chapters 30 through 32.

#### Lean correspondence

Coverage classifies this theorem as exact and `reexported-proof`. The public
declaration `CrouzeixTextbook.Part06.realization_norm_two` is in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]].
Its exact provider is
`CrouzeixConjecture.LoristSchwenninger.norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary`
in
[[formalization/lean/Crouzeix/LoristSchwenninger/ConcreteDilation.lean|ConcreteDilation.lean]].
That provider constructs `dilationDataOfParametricPolynomial` and invokes
`DilationData.norm_target_le_two` from
[[formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean|PerturbationLemma.lean]].
Exercise 34-E06 instantiates the concrete boundary constructor, identifies its
stored target definitionally, and then applies the abstract endpoint to that
specific record. It does not call this parent theorem or either terminal
Crouzeix theorem. The receipt records `Classical.choice`, `Quot.sound`, and
`propext` and no project axiom.

#### Historical context

This is the LS source-derived factor-two realization endpoint. Its exact
captured locator is
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124`. The source manifest
records the captured bytes and digest, while
[[labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json|the LS source graph]]
binds the local concrete-dilation theorem to that span. Harp's Lean theorem
reconstructs the fixed-domain endpoint. It does not assert peer-review status,
acceptance, journal publication, or historical priority.

#### ML analogy

Mathematical object / ML counterpart. Here boundary multiplication ↔ diagonal
lifted feature action. The isometry `V` lifts the visible state,
and `V* M_h^k V` compresses a tied k-step feature evolution back to the
original coordinates. The factor-two theorem is a robust operator-norm
certificate for a nonnormal linear layer filtered by a polynomial.

Exact transfer. When the lift, multiplier, every compression moment, the
uniform companion bound, and commutation identities are exact, the concrete
record satisfies Chapter 33 and its norm certificate transfers unchanged.
The exact compression identity is the key bridge from latent dynamics to the
visible operator.

Non-transfer. In particular, learned or approximate features do not supply exact moments.
Small training loss, a nearly isometric encoder, or matching finitely many
rollout moments does not establish the infinite power identity, the common
perturbation bound, or exact commutation.

Diagnostic. For the running family

```text
A_{λ,α} = [[λ, α], [0, -λ]],
```

choose quadrature nodes `σ_j`, weights `w_j`, density samples `D_j`, and
`h_j=q(Γ.point σ_j)`. Form the block lift `V̂`, the diagonal multiplier
`M̂_h`, and the sampled companion `Ĉ_k`. The exact continuum identity predicts

```text
V̂* M̂_h^k V̂ ≈ (1/2) (q(A_{λ,α})^k + Ĉ_k*).
```

Calculate the empirical moment residuals

```text
R̂_k(λ, α) := V̂* M̂_h^k V̂
              - (1/2) (q(A_{λ,α})^k + Ĉ_k*),
```

and tabulate `‖R̂_k(λ, α)‖` over chosen powers and parameters. Also report the
mass residual `‖Σ_j w_j D_j - 2I‖`. A large residual rejects the sampled
certificate. A finite residual table does not prove the infinite exact
family, even when every reported number is small.

Together, CFT-34-001--006 form a typed assembly line: normalized boundary
mass builds the isometry; first-moment compression establishes the base
identity; the pointwise multiplier law propagates it to every power;
contractivity supplies a uniform bound; and the resulting `DilationData`
record is exactly the input expected by Chapter 33. No step imports Jin's
completion theorem. The normalization corollary handles both `m>0` and `m=0`;
only the outer-domain limiting passage is reserved for Chapter 35.

## Worked examples

**Example 1.** For constant density `D=2I` on a probability boundary, `V`
maps `x` to the constant field `x`; the mass calculation reduces immediately
to `‖Vx‖₂=‖x‖`.

**Example 2.** If `|h(σ)|≤ρ<1` almost everywhere, multiplication by `h` is a
strict contraction on scalar or vector-valued `L²`: `‖M_h f‖₂≤ρ‖f‖₂`.
Nothing in the final argument needs the stronger—and generally irrelevant—
claim that the operator norm equals the boundary supremum.

## ML bridge

Read `D^{1/2}` as exact whitening, `V` as a feature lift, and `M_h` as tied
diagonal latent dynamics. This analogy explains the architecture, but not the
proof obligation: learned features and finitely many small empirical residuals
do not imply the exact infinite moment family. CFT-34-006 gives the residual
diagnostic for `A_{λ,α}` and marks that boundary precisely.

## Lean translation

`#check CrouzeixTextbook.Part06.boundary_dilation_data` displays every analytic
and measure-theoretic assumption; `#check
CrouzeixTextbook.Part06.realization_norm_two` shows the final Chapter 33
application. The exact declarations and source lines are linked from the six
cards above and from
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]].
The import graph contains no Jin endpoint, so route independence is a checked
formal boundary rather than a prose convention.

## Exercises

### CFT-34-E01 -- retrieval {#exercise-cft-34-e01}

Given D : PositiveBoundaryDensity μ and x : EuclideanVector n, prove ‖boundaryEmbeddingToLp D x‖ = ‖x‖.

The corresponding checked statement is
`CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_01_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]].
Do not invoke `boundary_dilation_data`; calculate the isometry field itself.

### CFT-34-E02 -- calculation {#exercise-cft-34-e02}

Given D, h, and x, prove (V* M_h V)x = euclideanOperator (boundaryPhiCLM D h)x by expanding the L2 inner product.

Here `V=boundaryEmbedding D` and `M_h=bcfMulL h`. The corresponding checked
statement is
`CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_02_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]].
Do not invoke the completed compression theorem; expose the pointwise fields
and integral calculation.

### CFT-34-E03 -- written-proof {#exercise-cft-34-e03}

Prove the scalar Cauchy/Fourier mode calculation

```text
∮ z in C(0, 1), z⁻¹ = 2 * π * I,
n ≠ -1 → ∮ z in C(0, 1), z ^ n = 0.
```

Conclude that division by `z` shifts the constant coefficient of a polynomial
to the unique surviving exponent `-1`. In the checked finite-sum form, prove

```text
∮ z in C(0, 1),
  ∑ m ∈ Finset.range (N + 1), a m * z ^ ((m : ℤ) - 1)
  = a 0 * (2 * π * I).
```

The corresponding checked statement is
`CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_03_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]].
Use the circle-integral kernel itself, not either compression theorem.

### CFT-34-E04 -- written-proof {#exercise-cft-34-e04}

Given `D`, `h`, `k`, and `x`, derive the pointwise power-compression identity

```text
((V*).comp ((M_h^k).comp V)) x
  = euclideanOperator (boundaryPhiCLM D (h^k)) x.
```

First rewrite bcfMulL_pow, then apply the first-moment compression theorem to
h^k. The corresponding checked statement is
`CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_04_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]].
Do not call the completed power-compression theorem.

### CFT-34-E05 -- boundary {#exercise-cft-34-e05}

Let `h : i →ᵇ ℂ`, assume `hh : ‖h‖ ≤ 1`, and let
`f : i →₂[μ] EuclideanVector n`. Prove the applied contraction estimate

```text
(h : i →ᵇ ℂ) (hh : ‖h‖ ≤ 1)
(f : i →₂[μ] EuclideanVector n) :
```

```text
‖bcfMulL h f‖ ≤ ‖f‖.
```

First use the operator estimate `bcfMulL_norm_le`, then the hypothesis `hh`,
and finally the operator-norm inequality on `f`. Do not call
`bcfMulL_norm_le_one`, which is the provider of the parent card. The checked
statement is
`CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_05_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]].

### CFT-34-E06 -- lean-proof {#exercise-cft-34-e06}

Under the compact parametrized-boundary assumptions of this chapter, let
`Γ : ParametricConvexBoundary Ω`, assume `hWB : numericalRange B ⊆ Ω`, let
`q : Polynomial ℂ` satisfy
`hq : ∀ z ∈ closure Ω, ‖Polynomial.eval z q‖ ≤ 1`, and assume
`hCauchy : HasParametricPolynomialCauchyFormula Γ μ B`. Define

```text
data := dilationDataOfParametricPolynomial Γ B hWB q hq hCauchy.
```

Prove the concrete target identification and factor-two bound together:

```text
data.T = euclideanOperator (polynomialEval q B) ∧ ‖data.T‖ ≤ 2.
```

The first component is definitional after instantiating the Chapter 34
constructor. For the second, use `DilationData.norm_target_le_two` on that
specific record. Do not call `realization_norm_two` or
`norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary`. The
exercise therefore checks the final Chapter 34 assembly boundary rather than
repeating the abstract Chapter 33 endpoint. Its checked statement is
`CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_06_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]].

### Solution sketches

#### Solution to CFT-34-E01

The representative `boundaryEmbeddingField D x` is almost-everywhere strongly
measurable and belongs to `L²` by `boundaryEmbeddingField_memLp`. The theorem
`boundaryEmbeddingField_sq_norm_ae` gives, almost everywhere,

```text
‖boundaryEmbeddingField D x σ‖²
  = (1/2) Re⟪D(σ)x,x⟫.
```

Integrating and applying `integral_normalized_boundaryQuadraticForm` yields
`‖boundaryEmbeddingToLp D x‖²=‖x‖²`. Both norms are nonnegative, so equality of
squares gives equality of norms. The Lean solution uses the compiled provider
`norm_boundaryEmbeddingToLp`; its proof in
[[formalization/lean/Crouzeix/LoristSchwenninger/BoundaryEmbedding.lean|BoundaryEmbedding.lean]]
contains these steps.

#### Solution to CFT-34-E02

Test both sides against an arbitrary `y`. Move `V*` across the inner product,
then unfold the `L²` inner product. The lemmas `boundaryEmbeddingField_memLp`
and `bcfMulL_apply_ae` replace the quotient-space representatives by

```text
(Vx)(σ)=2^(-1/2)D(σ)^(1/2)x,
(M_hVx)(σ)=h(σ)(Vx)(σ)
```

almost everywhere. Now `boundarySquareRoot_mul_self_ae` turns the two square
roots into `D(σ)`. The remaining integral is

```text
(1/2) ∫ σ, ⟪y, euclideanOperator (h(σ)D(σ)) x⟫ ∂μ.
```

Move the continuous linear map and inner product through the Bochner integral.
Finally `boundaryPhiCLM_apply` identifies the result with
`⟪y,euclideanOperator (boundaryPhiCLM D h)x⟫`. Equality of inner products
against every `y` proves the vector equality. This proof is written directly
in the exercise theorem; it does not call the parent compression theorem in
[[formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean|CompressionMoments.lean]].

#### Solution to CFT-34-E03

Mathlib's theorem `circleIntegral.integral_sub_inv_of_mem_ball` states that
the integral of `(z-w)⁻¹` around a circle containing `w` is `2πi`. Set the
center and `w` to zero and the radius to one. The point `0` lies in the open
unit ball, so simplification gives

```text
∮ z in C(0, 1), z⁻¹ = 2 * π * I.
```

For an integer `n ≠ -1`, the theorem
`circleIntegral.integral_sub_zpow_of_ne` gives zero. Again set both center and
offset to zero. This formal result packages the elementary primitive argument
for every other integer mode, including negative modes whose singularities
need the circle-integrability case split. The Lean proof is distinct from both
compression declarations. For coefficients `a : ℕ → ℂ`, use
`circleIntegral.integral_fun_sum` to move the circle integral through the
finite sum. Every summand is circle-integrable because zero is not on the unit
circle. Then `Finset.sum_eq_single` separates `m=0`: its exponent is `-1` and
contributes `a 0 * (2 * π * I)`, while `m>0` makes
`(m : ℤ) - 1 ≠ -1`, so every remaining integral is zero. Thus the polynomial
coefficient-selection corollary follows by finite-sum linearity in the same
compiled theorem; it is not prose-only.

The ephemeral compiler receipt audits proof-body dependencies against the
exact external allowlist. Its semantics are transitive through maintained
helpers, but it excludes type-only names and comments; external allowlisted
lemmas are terminal leaves rather than recursively expanding all of Mathlib.
For this theorem the compiled audit records exactly
`circleIntegral.integral_sub_inv_of_mem_ball` and
`circleIntegral.integral_sub_zpow_of_ne`. The Lean proof uses
[Mathlib CircleIntegral.lean](https://github.com/leanprover-community/mathlib4/blob/v4.32.1/Mathlib/MeasureTheory/Integral/CircleIntegral.lean)
directly.

#### Solution to CFT-34-E04

The goal contains the operator power `(bcfMulL h)^k`. Rewrite it in the reverse
direction with

```text
bcfMulL_pow : bcfMulL (h^k) = (bcfMulL h)^k.
```

After the rewrite, the left side is first-moment compression with the bounded
continuous function `h^k`. Apply
`boundaryEmbedding_adjoint_comp_bcfMulL_comp_boundaryEmbedding D (h^k)`, then
evaluate the resulting equality of continuous linear maps at `x` using
`congrArg`. The proof depends on the smaller multiplier and first-compression
lemmas in
[[formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean|BoundaryMultiplier.lean]]
and
[[formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean|CompressionMoments.lean]].
It does not use
`boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding`, which is
the parent theorem this exercise is meant to reconstruct.

#### Solution to CFT-34-E05

The continuous linear map norm inequality gives

```text
‖bcfMulL h f‖ ≤ ‖bcfMulL h‖ ‖f‖.
```

The smaller provider `bcfMulL_norm_le` bounds the first factor by `‖h‖`.
Multiply its inequality and `hh : ‖h‖≤1` by the nonnegative number `‖f‖`:

```text
‖bcfMulL h‖ ‖f‖ ≤ 1 * ‖f‖ = ‖f‖.
```

Transitivity proves the goal. The solution is distinct from CFT-34-005: its
conclusion is an inequality for one input vector, and its proof body depends
on `bcfMulL_norm_le`, not `bcfMulL_norm_le_one`. The provider is in
[[formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean|BoundaryMultiplier.lean]].

#### Solution to CFT-34-E06

Instantiate the exact boundary record

```text
data := dilationDataOfParametricPolynomial Γ B hWB q hq hCauchy.
```

The constructor stores
`T := euclideanOperator (polynomialEval q B)`, so the first conjunct is `rfl`.
Its other fields contain the boundary isometry, multiplier contraction,
compressed-power perturbation identity, nonnegative common bound, uniform
perturbation estimate, and commutation family proved in CFT-34-001 through
CFT-34-005. Apply `DilationData.norm_target_le_two data` for the second
conjunct. Thus the proof term is

```text
⟨rfl, data.norm_target_le_two⟩.
```

The solution uses both the concrete Chapter 34 constructor and the abstract
Chapter 33 endpoint, but it is not a copy of CFT-33-E06: its hypotheses are the
boundary realization data and its conclusion also identifies the constructed
target. It invokes neither `realization_norm_two` nor
`norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary`. The
constructor is in
[[formalization/lean/Crouzeix/LoristSchwenninger/ConcreteDilation.lean|ConcreteDilation.lean]],
and the endpoint proof is in
[[formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean|PerturbationLemma.lean]].

## Synthesis and forward dependencies

All six Chapter 34 cards now have reconstructible prose and exact compiled
correspondence. The fixed-domain realization is complete. Chapter 35 is
complete as the next handoff: it passes from this normalized fixed-domain
result to the checked LS terminal
`CrouzeixConjecture.loristSchwenningerMainTheorem`, derives the maintained
consequences, and compares that route with Jin and the Harp-derived
finite-horizon terminal
`CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem`. Those declarations are
compiler-checked boundaries, not claims of publication status, priority, or
independent peer review.
