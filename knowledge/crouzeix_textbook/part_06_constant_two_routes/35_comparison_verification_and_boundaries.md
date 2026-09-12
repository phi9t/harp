---
id: cft-chapter-35-comparison-verification-and-boundaries
title: Comparison, verification, and boundaries
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-28
tags: [crouzeix-textbook, constant-two-routes, mathematics, lean]
confidence: high
canonical: 35_comparison_verification_and_boundaries.md
chapter: 35
part: 6
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 35: Comparison, verification, and boundaries

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-vi-constant-two-routes|Part VI — Constant-two routes]]
Previous: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/34_lorist_schwenninger_realization|Chapter 34 — The Lorist–Schwenninger realization]]
Next: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/36_harp_finite_horizon_proof|Chapter 36 — The Harp finite-horizon proof]].

## Contract snapshot

| Metric | Count |
| --- | ---: |
| theorem rows | 216 |
| summary prose rows | 53 |
| reconstructible prose rows | 161 |
| exact correspondence rows | 168 |
| unmapped correspondence rows | 30 |
| solved exercises | 168 |
| unresolved exercises | 48 |

## Opening problem

Three checked proofs reach the same finite-matrix bound. Which statements are
genuinely common, which mechanisms belong to one route, and what exactly has
Lean verified?

## Conceptual model

All three routes normalize a function, use smooth outer numerical-range
domains, prove a fixed-domain estimate for simple-spectrum matrices, and pass
through the simple-spectrum and outer-domain limits. Jin converts the complete
boundary power family into positive-real kernel completion and Gramians.
Lorist–Schwenninger realizes the whole family as a boundary dilation and uses
a bounded recurrence. Harp replaces the exact infinite realization by finite
atomic data at every horizon and then proves a uniform finite-horizon bound.
The final inequality agrees; the proof objects do not.

[[knowledge/crouzeix_textbook/part_06_constant_two_routes/36_harp_finite_horizon_proof|Chapter 36]]
develops Harp's construction and recurrence in full. This chapter's comparison
is a reading map, not a substitute for that proof.

## Formal development

### The six-item spine

### CFT-35-001 — lorist schwenninger main {#cft-35-001}

#### Purpose

Motivation. We remove the normalization and approximation scaffolding from
Chapter 34. The realization theorem there controls a
normalized polynomial on one smooth outer domain and one simple-spectrum
matrix. The result wanted by a reader, or by an ML stability calculation, is
instead a bound for an arbitrary polynomial and the original matrix, measured
on its actual numerical range.

#### Statement

For every finite nonempty index type `n`, matrix `A : SquareMatrix n`, and
polynomial `p : Polynomial ℂ`,

```text
norm (polynomialEval p A)
  ≤ 2 * maxPolynomialModulusOnNumericalRange A p.
```

Thus the two quantities in the contract are, in order,
`norm (polynomialEval p A)` and
`2 * maxPolynomialModulusOnNumericalRange A p`. In mathematical notation,

```text
‖p(A)‖ ≤ 2 max_{z∈W(A)} |p(z)|.
```

#### Hypothesis ledger

For a finite nonempty index type, the complete Lean parameter list is:
`n : Type*`; `Fintype n`;
`DecidableEq n`; `Nonempty n`; `A : SquareMatrix n`; and
`p : Polynomial ℂ`. In short: matrix A; polynomial p. The phrase finite nonempty index type abbreviates exactly
these typeclass assumptions. There is no normality, diagonalizability, or
simple-spectrum assumption on `A`.

#### Proof roadmap

Fix an outer domain around `W(A)`. Approximate `A` by simple-spectrum
matrices, normalize `p` on the closure of that domain, and apply CFT-34-006.
First pass to the original matrix. Then shrink the outer domains to `W(A)`.
The two limits occur in that order because the fixed-domain estimate needs the
approximating numerical ranges to stay inside one open set.

#### Proof

Let

```text
K := numericalRange A,
Ω_k := parallelOuterDomain K k,
M_k := maxPolynomialModulusOnSet (closure Ω_k) p.
```

Compactness and nonemptiness of `K`, together with convexity of the numerical
range, make each `Ω_k` an admissible smooth open outer domain. In the radial
picture one may write the same final shrinkage as `r ↓ 1`; the formal provider
uses the canonical parallel outer approximation indexed by `k`.

Fix `k`. The simple-spectrum approximation
`B_j := simpleSpectrumApproximation A j` satisfies `B_j → A`, and eventually
`W(B_j) ⊆ Ω_k`. We now separate the normalization into its two real cases.

If `M_k > 0`, define

```text
q_k := M_k⁻¹ • p.
```

The maximum property gives `|q_k(z)|≤1` on `closure Ω_k`. The boundary data
constructed in Chapter 34 therefore satisfy every hypothesis of CFT-34-006,
so that theorem first gives the Euclidean operator-norm estimate

```text
hnormalizedOperator :
  ‖euclideanOperator (polynomialEval q_k B_j)‖ ≤ 2.
```

The matrix norm in the main theorem is the induced Euclidean operator norm.
The compiled proof crosses that bridge explicitly:

```text
hnormalized : ‖polynomialEval q_k B_j‖ ≤ 2
  := by simpa only [matrix_norm_eq_euclidean_operator_norm]
       using hnormalizedOperator.
```

Scalar compatibility of polynomial evaluation gives

```text
polynomialEval q_k B_j = M_k⁻¹ • polynomialEval p B_j,
```

and hence `‖polynomialEval p B_j‖≤2*M_k`.

Compiled Lean zero branch. If `M_k = 0`, division by `M_k` is unavailable.
Instead, use the distinct spectrum of `B_j`:

```text
hDiag := simpleDiagonalization_of_hasDistinctEigenvalues B_j
hDiag.eigenvalues i ∈ matrixSpectrum B_j
                      ⊆ numericalRange B_j
                      ⊆ closure Ω_k.
```

The maximum property and `M_k=0` imply

```text
Polynomial.eval (hDiag.eigenvalues i) p = 0
```

for every diagonal entry. The diagonalization formula for polynomial
evaluation now gives

```text
polynomialEval p B_j = 0.
```

Thus `‖polynomialEval p B_j‖≤2*M_k` also holds in the zero branch. The
compiled proof only needs `p` to vanish on the eigenvalues of this `B_j`; it
does not infer that `p` is the zero polynomial.

Now apply continuity of polynomial evaluation. As `j → ∞`,

```text
polynomialEval p (simpleSpectrumApproximation A j)
  → polynomialEval p A,
```

and the fixed `k` estimate becomes

```text
‖polynomialEval p A‖ ≤ 2 * M_k.
```

Finally, as `k → ∞`, the compact sets `closure Ω_k` decrease toward `K`, and
continuity of `|p|` gives

```text
M_k → maxPolynomialModulusOnNumericalRange A p.
```

Passing the inequality through this outer approximation proves

```text
‖polynomialEval p A‖ ≤ 2 * maxPolynomialModulusOnNumericalRange A p.
```

This is the provider's actual order: fix an outer domain; normalize the
polynomial; apply CFT-34-006; pass simple-spectrum limit; pass outer-domain
limit.

The worked polynomial p(z) = z reduces the conclusion to ‖A‖ ≤ 2 max_{z∈W(A)} ‖z‖.
For the running nonnormal matrix
`A_{λ,α}=[[λ,α],[0,-λ]]`, the right side responds to both eigenvalue location
and nonnormal off-diagonal growth; replacing `W(A)` by the spectrum would lose
that information.

#### Boundary case

The zero polynomial makes both sides zero. More substantively, the `M_k=0`
branch above proves the result without an illegal inverse. It is part of the
argument, not an omitted convention.

#### Pedagogical prerequisites

Use CFT-29-003 for the simple-spectrum and outer-domain limit interface, and
CFT-34-006 for the normalized fixed-domain estimate. Chapter 34 is the proof
provider for the sharp local bound.

#### Lean correspondence

The coverage row is a theorem in `reexported-proof` mode. The public
declaration is
`CrouzeixTextbook.Part06.lorist_schwenninger_main` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].
It reexports `CrouzeixConjecture.loristSchwenningerMainTheorem` from
[[formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean|MainTheorem.lean]].
The compiler receipt binds the exact files
`formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean` and
`formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean`, the
normalized declaration type, and its standard axioms.

#### Historical context

This is the main polynomial consequence of the Lorist--Schwenninger boundary
realization. Classification: LS source-derived terminal. Exact source locator:
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128`. The local formal proof
also makes the two limits and the zero-normalization branch explicit; the
classification does not claim that every local supporting lemma appears in
those four source lines.

#### ML analogy

Mathematical object / ML counterpart. The polynomial functional calculus is a
nonnormal layer functional bound for a tied linear operator.

Exact transfer. If an ML model applies the same finite matrix `A` repeatedly
and a post-processing map is exactly a polynomial `p`, the displayed operator
norm inequality applies without assuming that `A` is normal.

Non-transfer. A nonlinear network, stochastic layer, time-varying recurrence,
or approximate polynomial evaluator is not covered until its additional error
is bounded. Empirical spectral plots do not replace the numerical-range
maximum.

Diagnostic. For `A_{λ,α}`, compute `p(A_{λ,α})`, estimate its largest singular
value, and sample `max_{z∈W(A_{λ,α})}|p(z)|`. Report the ratio. A value above
two rejects the numerical implementation or the sampling resolution; a value
below two is a check, not a proof that the sampled maximum is exact.

### CFT-35-002 — lorist schwenninger finite matrix {#cft-35-002}

#### Purpose

Motivation. Its purpose is to specialize the type-parametric theorem to Fin d, the
index type used by conventional finite matrices and by the later Hilbert-space
transport. It makes the finite-index equivalence visible instead of treating
`SquareMatrix (Fin d)` as opaque notation.

#### Statement

The conclusion is `FiniteMatrixMainTheoremStatement`, quantified over
`Fin d`. Unfolded,

```text
FiniteMatrixMainTheoremStatement :=
  ∀ (d : ℕ) [Nonempty (Fin d)], MainTheoremStatement (n := Fin d).
```

#### Hypothesis ledger

Choose `d : Nat`, written `d : ℕ` in Lean, together with
`Nonempty (Fin d)`. The latter is the positive-dimension convention. No
additional matrix hypothesis is introduced.

#### Proof roadmap

Specialize CFT-35-001 at `n := Fin d`. Then unfold the coordinate meanings of
the matrix and vector types so the standard `d × d` statement is apparent.

#### Proof

By definition,

```text
FiniteMatrixMainTheoremStatement := ∀ (d : ℕ) [Nonempty (Fin d)], MainTheoremStatement (n := Fin d)
SquareMatrix (Fin d) = Matrix (Fin d) (Fin d) ℂ
EuclideanVector (Fin d) = Fin d → ℂ.
```

The first type is a `d × d complex matrix`; the second is its standard
coordinate Hilbert space with the Euclidean norm. The proof term is simply

```text
fun d => loristSchwenningerMainTheorem (n := Fin d).
```

In words: specialize CFT-35-001; identify SquareMatrix (Fin d). This proves
every field of `FiniteMatrixMainTheoremStatement` without changing the
constant or adding a choice of basis. The case `d = 0` cannot supply
`Nonempty (Fin d)`.

The worked dimension d = 2 instantiates SquareMatrix (Fin 2) without changing the constant.
A function `A : Fin 2 → Fin 2 → ℂ` is the familiar four-entry
array, while `polynomialEval p A` is ordinary matrix polynomial evaluation.

#### Boundary case

d = 0 is excluded by Nonempty (Fin d). The exclusion is mathematical:
the numerical range is defined using unit vectors and would be empty in the
zero-dimensional space.

#### Pedagogical prerequisites

CFT-20-006 supplies the finite-coordinate interpretation. CFT-35-001 supplies
the theorem for an arbitrary finite nonempty index type.

#### Lean correspondence

The theorem is an exact `reexported-proof`. Public declaration:
`CrouzeixTextbook.Part06.lorist_schwenninger_finite_matrix` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].
Provider: `CrouzeixConjecture.loristSchwenningerFiniteMatrixMainTheorem` in
[[formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean|Consequences.lean]].
The receipt records
`formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean` and
`formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean`.

#### Historical context

This finite-matrix consequence adapter is a provider-clean consequence, not a
second LS proof. Exact locator:
`formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L20-L24`.
Its small proof is valuable because it fixes the positive-dimension and
standard-index conventions used by downstream adapters.

#### ML analogy

Mathematical object / ML counterpart. This is a width-indexed matrix theorem:
the width `d` is explicit in the coordinate type.

Exact transfer. A finite linear layer with complex `d × d` weights is exactly
a `SquareMatrix (Fin d)` after choosing the displayed coordinate convention.

Non-transfer. Changing width during inference, using rectangular maps, or
moving between learned bases requires a separate transport argument. The
adapter itself says nothing about numerical conditioning of that basis.

Diagnostic. For `d=2,4,8`, check that the implementation uses the same
Euclidean operator norm and polynomial coefficient convention as Lean. Record
the matrix shape, norm routine, and the ratio from CFT-35-001. A shape-only
test does not certify the inequality.

### CFT-35-003 — lorist schwenninger rational {#cft-35-003}

#### Purpose

Motivation. Resolvents and rational filters occur more often than raw
polynomials in stability analysis. The purpose is to extend polynomial control to pole-free rational functions
by approximation. The limiting argument matters:
a rational function is not a polynomial that can be normalized and passed
directly to CFT-35-001.

#### Statement

For a rational `r`, assuming its poles avoid `W(A)`,

```text
norm (rationalMatrixEval r A)
  ≤ 2 * maxRationalModulusOnNumericalRange A r.
```

Equivalently, the ordered quantities are
`norm (rationalMatrixEval r A)` and
`2 * maxRationalModulusOnNumericalRange A r`.

#### Hypothesis ledger

Work over a finite nonempty index type with `A : SquareMatrix n`. Let
`r : RatFunc ℂ` be a rational function, abbreviated rational r. The poles avoid W(A): the pole-free assumption is
`RationalPoleFreeOn r (numericalRange A)`; informally,
`poles r ∩ numericalRange A = ∅`. The condition is on the whole numerical
range, not only the spectrum.

#### Proof roadmap

Construct polynomial approximants on the compact convex numerical range.
Prove uniform scalar convergence. Use the main polynomial theorem to obtain
matrix-evaluation convergence. Prove convergence of the maxima. Then pass the
polynomial inequalities to the limit.

#### Proof

Write `K:=W(A)`. It is compact, nonempty, and convex. The assumption may be
displayed as `poles r ∩ numericalRange A = ∅`. Factor a reduced
rational representation of `r` into a polynomial numerator and finitely many
reciprocal linear factors. Because every pole `a` lies outside `K`, complex
separation supplies `c≠0` and `0≤ρ<1` such that

```text
sup_{z∈K} |1 - c(a-z)| ≤ ρ.
```

The finite geometric polynomial

```text
-c * Σ_{j=0}^{N-1} (1-c(a-z))^j
```

approximates `(z-a)⁻¹` with uniform error bounded by a constant times `ρ^N`.
Multiplying these approximants, including repeated factors for pole
multiplicity, and then multiplying by the numerator gives polynomial
approximants `q_N` satisfying the uniform scalar convergence

```text
sup_{z∈W(A)} ‖q_N(z) - r(z)‖ → 0.
```

This step uses pole-freeness and convexity; it is the concrete replacement for
an unexplained invocation of Runge's theorem.

The polynomial bound now controls differences of evaluations. Applied to
`q_N-q_M`, it shows that `q_N(A)` is Cauchy and identifies its limit with the
rational functional calculus:

```text
‖polynomialEval q_N A - rationalMatrixEval r A‖ → 0.
```

Uniform scalar convergence also yields maximum convergence:

```text
maxPolynomialModulusOnNumericalRange A q_N
  → maxRationalModulusOnNumericalRange A r.
```

For every `N`, CFT-35-001 gives

```text
‖polynomialEval q_N A‖
  ≤ 2 * maxPolynomialModulusOnNumericalRange A q_N.
```

Taking limits on both sides proves the limit inequality

```text
‖rationalMatrixEval r A‖
  ≤ 2 * maxRationalModulusOnNumericalRange A r.
```

The provider order is therefore: polynomial approximants; uniform scalar
convergence; matrix evaluation convergence; maximum convergence; limit
inequality.

For `r(z)=(z-a)⁻¹`, the one-pole rational approximant sequence q_N converges uniformly before q_N(A) and the maxima pass to their limits.
An embedded polynomial is a rational boundary case: its denominator
is one and the approximation sequence may be chosen constant.

#### Boundary case

Here an embedded polynomial is a rational boundary case. If a pole touches
`W(A)`, the scalar maximum and rational matrix evaluation no longer meet this
theorem's pole-free contract; the proof does not assign them a finite bound by
continuity across the pole.

#### Pedagogical prerequisites

CFT-17-006 supplies the rational functional calculus and polynomial
approximation interface. CFT-35-001 supplies the polynomial inequalities that
control the operator-evaluation sequence.

#### Lean correspondence

The exact theorem is in `reexported-proof` mode. Public declaration:
`CrouzeixTextbook.Part06.lorist_schwenninger_rational` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].
Provider: `CrouzeixConjecture.loristSchwenningerRationalSpectralSetCorollary`
in
[[formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean|Consequences.lean]].
The provider delegates to the neutral proof
`CrouzeixConjecture.rationalSpectralSetCorollary_of_mainTheorem` in
[[formalization/lean/CrouzeixConjecture/RationalApproximation.lean|RationalApproximation.lean]].
The receipt binds `formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean`
and `formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean`.

#### Historical context

This rational approximation consequence is a provider-clean consequence.
Exact locator:
`formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L28-L33`.
The local neutral adapter expands the approximation route so the rational
statement does not masquerade as a source theorem proved by a rescaling trick.

#### ML analogy

Mathematical object / ML counterpart. A pole-free rational functional is a
resolvent-filter certificate for a linear model.

Exact transfer. Finite compositions and sums of resolvents of the exact matrix
`A` satisfy the rational bound when every pole avoids `W(A)`.

Non-transfer. Truncated iterative solvers, estimated poles, pseudospectral
surrogates, and nonlinear attention kernels add approximation errors. A pole
outside the spectrum but inside `W(A)` does not satisfy this theorem.

Diagnostic. Compute
`min_{z∈W(A)} distance(z, poles(r))`, the uniform scalar approximation error,
the matrix error `‖q_N(A)-r(A)‖`, and the maximum-modulus error. Reject the
certificate when the sampled pole margin is nonpositive or the two convergence
curves fail to decay. Finite samples cannot prove pole-freeness on all of
`W(A)`.

### CFT-35-004 — lorist schwenninger hilbert {#cft-35-004}

#### Purpose

Motivation. Its purpose is to transport finite matrices to bounded Hilbert-space operators.
The key is local finite-dimensionality: a polynomial applied to one
vector only visits a finite Krylov subspace. The argument does not assume that
the whole Hilbert space is finite-dimensional.

#### Statement

For every complete nontrivial complex Hilbert space H, bounded operator `A`,
and polynomial `p`,

```text
norm (operatorPolynomialEval p A)
  ≤ 2 * supPolynomialModulusOnOperatorNumericalRange A p.
```

The two ordered expressions are `operatorPolynomialEval p A` and
`2 * supPolynomialModulusOnOperatorNumericalRange A p`.

#### Hypothesis ledger

For a complete nontrivial complex Hilbert space H, the exact assumptions are
`H : Type*`; `NormedAddCommGroup H`;
`InnerProductSpace ℂ H`; `CompleteSpace H`; `Nontrivial H`;
`A : H →L[ℂ] H`; and `p : Polynomial ℂ`: bounded operator A; polynomial p.

#### Proof roadmap

First prove the result for a finite-dimensional Hilbert space by choosing an
orthonormal basis and applying the `Fin d` matrix theorem. For general `H`,
compress to the finite Krylov space generated by one unit vector and the
powers needed by `p`. Transport the finite estimate back, then take the
operator-norm supremum.

#### Proof

In the finite-dimensional case put

```text
H : Type*, CompleteSpace H, Nontrivial H,
A : H →L[ℂ] H, p : Polynomial ℂ,
m := Module.finrank ℂ H
b : OrthonormalBasis (Fin m) ℂ H.
```

The matrix `C` of `A` in this orthonormal basis is indexed by `Fin m`. The basis
representation is an isometry, polynomial evaluation commutes with the matrix
representation, and

```text
numericalRange C = operatorNumericalRange A.
```

Thus CFT-35-002 gives the same factor-two estimate for `A`. This direct
finite-dimensional argument uses `m := Module.finrank ℂ H` and can bypass the
Krylov compression.

For arbitrary `H`, fix a unit vector `x` and put `d := p.natDegree`. Define the
finite-dimensional range model

```text
H_d := span ℂ {A^k x | 0 ≤ k ≤ d}.
```

It is spanned by `d+1` vectors. Its coordinate dimension is a different
number:

```text
m := Module.finrank ℂ H_d,
m ≤ d + 1.
```

Let `P_d` be the orthogonal projection onto `H_d` and let the compressed
operator be `T := P_d A P_d`, interpreted as an operator on `H_d`. Every power
needed by `p` stays in the Krylov span, so

```text
p(T)x = p(A)x.
```

Compression does not enlarge the numerical range:

```text
W(T) ⊆ W(A).
```

Choose an orthonormal basis of `H_d` indexed by `Fin m` and identify `T` with
its Fin m coordinate matrix. CFT-35-002 at dimension `m` then applies. This
gives

```text
‖p(A)x‖ = ‖p(T)x‖
  ≤ 2 * sup_{z∈W(T)} |p(z)|
  ≤ 2 * sup_{z∈W(A)} |p(z)|.
```

Scale an arbitrary nonzero vector to unit norm and take the operator norm.
This is the transport back step.

The provider therefore proves the consequence for arbitrary complete
nontrivial Hilbert spaces. It never chooses an infinite matrix. The neutral
Krylov-compression theorem reduces each unit-vector query to a finite `Fin m`
matrix, and its only terminal input is CFT-35-002.

A finite-rank compression calculation compares P_d A P_d with its Fin m
matrix model. In finite-dimensional `H`, the direct basis argument with
`m := Module.finrank ℂ H` recovers the matrix theorem without identifying the
polynomial degree with the dimension of `H`.

#### Boundary case

The boundary case says finite-dimensional H recovers the matrix theorem. It
uses `m := Module.finrank ℂ H` and bypasses the Krylov compression. The
nontriviality assumption excludes the zero Hilbert space so the operator
numerical range has unit-vector witnesses.

#### Pedagogical prerequisites

CFT-21-001 supplies orthonormal coordinates, compression, and operator norms.
CFT-35-002 supplies the positive-dimensional finite matrix theorem.

#### Lean correspondence

The theorem is an exact `reexported-proof`. Public declaration:
`CrouzeixTextbook.Part06.lorist_schwenninger_hilbert` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].
Provider:
`CrouzeixConjecture.loristSchwenningerHilbertSpacePolynomialCrouzeix` in
[[formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean|Consequences.lean]].
The neutral transport is
`CrouzeixConjecture.hilbertSpacePolynomialCrouzeix_of_mainTheorem` in
[[formalization/lean/CrouzeixConjecture/HilbertSpaceCore.lean|HilbertSpaceCore.lean]].
The receipt records `formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean`
and `formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean`.

#### Historical context

This Hilbert-space transport consequence is a provider-clean consequence.
Exact locator:
`formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L37-L45`.
The local consequence is broader in ambient dimension than the finite-matrix
source endpoint, but its proof boundary is explicit and independently checked.

#### ML analogy

Mathematical object / ML counterpart. The finite Krylov model resembles an
infinite-width operator limit inspected through the finite features reached by
one polynomial query.

Exact transfer. For a bounded linear operator on a complete complex Hilbert
space, a degree-`d` polynomial and one input vector lie in a Krylov space of
dimension at most `d+1`; the compression equality is exact.

Non-transfer. A nonlinear or data-dependent operator may leave the Krylov
space, and a numerical finite-rank truncation need not preserve `p(A)x`
exactly. Compactness of `A` is neither assumed nor proved here.

Diagnostic. For a chosen numerical projection, report the finite-rank compression residual
`‖P_d A - A P_d‖` and the query residual
`‖P_d p(A)x-p(P_dAP_d)P_dx‖`. The proof uses a specially constructed Krylov
space with zero query residual; an arbitrary learned subspace does not inherit
that property.

### CFT-35-005 — lorist schwenninger two spectral set {#cft-35-005}

#### Purpose

Motivation. Its purpose is to package compactness, spectral containment, and rational control
into the standard statement that the closed numerical range is a two-spectral set. The package
has three separate obligations: the set is compact, it contains the spectrum,
and it controls every pole-free rational functional calculus by the constant
two.

#### Statement

For a complete nontrivial complex Hilbert space `H` and bounded operator `A`,
the statement `ClosedOperatorNumericalRangeIsTwoSpectralSet A` unfolds to

```text
ClosedOperatorNumericalRangeIsTwoSpectralSet A ↔
  IsCompact (closedOperatorNumericalRange A) ∧
  spectrum ℂ A ⊆ closedOperatorNumericalRange A ∧
  ∀ (r : RatFunc ℂ),
    RationalPoleFreeOn r (closedOperatorNumericalRange A) →
      ‖operatorPolynomialEval r.num A *
          Ring.inverse (operatorPolynomialEval r.denom A)‖ ≤
        2 * supRationalModulusOnClosedOperatorNumericalRange A r.
```

This is the exact rational constant-two inequality in the third conjunct. The
equivalence is definitional: unfolding the predicate proves the displayed
`↔`. The rational norm bound alone is not equivalent to the three-part
predicate. Compactness and spectrum containment remain separate conjuncts.

#### Hypothesis ledger

Assume a complete nontrivial complex Hilbert space H, concretely
`H : Type*`, `NormedAddCommGroup H`, `InnerProductSpace ℂ H`,
`CompleteSpace H`, and `Nontrivial H`. The bounded operator A is
`A : H →L[ℂ] H`.

#### Proof roadmap

Prove compactness from closedness and boundedness. Prove spectrum containment
by approximating the reciprocal of `z-a` outside the numerical range and
constructing an inverse. Prove the rational inequality by the same polynomial
approximation mechanism as CFT-35-003, now on the closed operator numerical
range. Assemble the three terms and unfold the predicate equivalence.

#### Proof

First, `operatorNumericalRange A` lies in the closed disk of radius `‖A‖`.
Its closure is therefore closed and bounded in `ℂ`, hence compact. This is the
provider `closedOperatorNumericalRange_isCompact`.

Second, suppose `a` lies outside the closed numerical range. Polynomial
approximants to `(z-a)⁻¹` converge uniformly on that compact convex set. The
CFT-35-004 polynomial bound makes the corresponding operator polynomials
Cauchy. Their limit is a two-sided inverse of `A-aI`, so `a` is outside the
spectrum. Contraposition gives spectrum containment through
`spectrum_subset_closedOperatorNumericalRange_of_mainTheorem`.

Third, for rational `r` pole-free on the closed numerical range, uniform
polynomial approximation passes to operator evaluations and scalar suprema.
The limiting bound is the rational inequality provided by
`hilbertSpaceRationalCrouzeix_of_mainTheorem`.

The constructor
`closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem` combines these
in exactly this order:

```text
closedOperatorNumericalRange_isCompact;
spectrum_subset_closedOperatorNumericalRange_of_mainTheorem;
hilbertSpaceRationalCrouzeix_of_mainTheorem.
```

Unfolding `ClosedOperatorNumericalRangeIsTwoSpectralSet A` shows the explicit
if and only if displayed in the statement. It equates the packaged predicate
with the full compactness, spectrum containment, and rational inequality
triple. No component follows merely from the other two.

For a scalar operator `A=λI`, the closed numerical range is `{λ}`, the spectrum
is `{λ}`, and rational evaluation is scalar multiplication by `r(λ)`. This
worked compactness-spectrum-bound triple checks the compact set, spectrum inclusion, and rational inequality separately.

#### Boundary case

The boundary case is that a scalar operator makes all three components explicit. If a rational pole is
at `λ`, the pole-free premise fails; the spectral-set predicate does not claim
a bound for that function.

#### Pedagogical prerequisites

CFT-21-005 supplies the spectral-set vocabulary and closure properties.
CFT-35-004 supplies the Hilbert polynomial estimate used by the neutral
rational and spectrum adapters.

#### Lean correspondence

The theorem is an exact `reexported-proof`. Public declaration:
`CrouzeixTextbook.Part06.lorist_schwenninger_two_spectral_set` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].
Provider:
`CrouzeixConjecture.loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet`
in
[[formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean|Consequences.lean]].
The neutral package is checked in
[[formalization/lean/CrouzeixConjecture/HilbertSpectralSetCore.lean|HilbertSpectralSetCore.lean]].
The receipt records `formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean`
and `formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean`.

#### Historical context

This closed numerical-range spectral-set consequence is a provider-clean
consequence. Exact locator:
`formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L55-L61`.
The local theorem keeps the compactness and spectrum claims visible rather
than using "spectral set" as an unexplained synonym for one norm inequality.

#### ML analogy

Mathematical object / ML counterpart. The predicate is a three-part stability certificate
for rational filters of a bounded linear layer.

Exact transfer. Compactness, spectrum containment, and the rational bound are
separate exact facts for the closed numerical range of the operator in the
theorem.

Non-transfer. A finite eigenvalue plot does not prove spectrum containment in
an infinite-dimensional approximation. A grid maximum does not prove a
uniform rational bound, and compactness can fail for a different candidate
set.

Diagnostic. Build a compactness-spectrum-bound triple: verify that the chosen
set representation is closed and numerically bounded, estimate the distance
of computed spectral values to the set, and track rational-bound ratios for a
specified pole family. Any failed component rejects the numerical
certificate. Passing finite checks does not prove the quantified predicate.

### CFT-35-006 — three route terminal comparison {#cft-35-006}

#### Purpose

Motivation. We compare three independently checked terminal routes without
turning the comparison into a fourth proof. A reader should be able to see
which mathematical choice changes the proof and which steps are shared. This
is also the point where an ML researcher can ask a useful ablation question:
what certificate would an implementation have to construct for each route?

#### Statement

For a finite nonempty index type `n`, Lean checks a conjunction with one copy
of the finite-matrix theorem from each route:

```text
Jin MainTheoremStatement ∧
Lorist-Schwenninger MainTheoremStatement ∧
Harp MainTheoremStatement.
```

All three conjuncts are `MainTheoremStatement (n := n)`. The labels preserve
their provenance. They do not create three different mathematical
propositions.

#### Hypothesis ledger

The shared hypothesis is a finite nonempty index type:
`n : Type`, `Fintype n`, `DecidableEq n`, and `Nonempty n`. The universe here
matches the certified Harp provider; every finite matrix size is represented
by `Fin d` in this universe. Each conjunct then
quantifies over `A : SquareMatrix n` and `p : Polynomial ℂ`. No normality,
simple-spectrum, or diagonalizability assumption survives into the terminal
statement.

#### Proof roadmap

Take the checked Jin terminal proof, the checked Lorist–Schwenninger terminal
proof, and the checked Harp terminal proof. Place them in the three conjuncts
without rewriting one route through another. Then inspect the compiler
receipt: the proof body must name all three providers directly.

#### Proof

The Lean proof uses the providers in this order:
`jinFinalCrouzeixConjecture`; `loristSchwenningerMainTheorem`;
`harpFiniteHorizonMainTheorem`. The first provider is eta-expanded from its
pointwise theorem to `MainTheoremStatement`; the other two already have that
quantified type.

The following eight-axis table is a comparison of proof architecture. The
comparison is not a proof dependency between routes.

| Axis | Jin | Lorist–Schwenninger | Harp |
| --- | --- | --- | --- |
| Objects. | A matrix-valued positive-real completion `H(z)`, sampled kernels, and Gram matrices. | A boundary `L²` space, isometry `V`, contraction `Q`, and commuting perturbations `E_k`. | Positive finite cubature nodes and weights, finite atomic `L²` dilations, and one certificate for every horizon `N`. |
| Hypotheses. | The normalized polynomial, simple-spectrum fixed-domain geometry, and the complete boundary power Cauchy family. | The same normalized fixed-domain data, realized as exact compression moments for every power. | The same normalized fixed-domain data, with exact moments only through the requested finite horizon; a certificate exists for every `N`. |
| Shared trunk. | Chapters 1–29: Euclidean matrix norms, numerical-range geometry, boundary positivity, polynomial Cauchy formulas, normalization, and the two terminal limits. | The same Chapters 1–29 trunk. | The same Chapters 1–29 trunk, plus finite positive cubature used only by this route. |
| Decisive mechanism. | Positive-real completion, cancellation at the sampled points, Gramian congruence, and a positive-semidefinite endpoint. | The boundary dilation, completed-square recurrence, uniform perturbation bounds, and a scalar contradiction. | The finite-horizon atomic certificates feed a horizon-uniform recurrence estimate; no single infinite atomic model is asserted. |
| Approximation order. | Complete the fixed-domain kernel, prove the simple-spectrum estimate, pass `j → ∞`, then shrink the outer domain. | Build all compression moments, prove the recurrence endpoint, pass `j → ∞`, then shrink the outer domain. | For each `N`, build exact finite atomic moments through `N+1`; use the uniform finite-horizon endpoint, then take the same simple-spectrum and outer-domain limits. |
| Conclusion. | `MainTheoremStatement (n := n)`, hence `‖p(A)‖ ≤ 2 max_{z∈W(A)} \|p(z)\|`. | The identical proposition and constant. | The identical proposition and constant. |
| Provenance. | Source-derived from the pinned Jin manuscript span, with local review and reproduction records. | Source-derived from the pinned LS preprint span, with local review and reproduction records. | Harp-derived finite-horizon route. It reuses checked common and LS lower-level modules and is not presented as a source reproduction. |
| Formal provider. | `CrouzeixConjecture.jinFinalCrouzeixConjecture`. | `CrouzeixConjecture.loristSchwenningerMainTheorem`. | `CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem`. |

This worked three-column comparison row records the common conclusion while
preserving route-specific evidence. It is an architectural map, not a theorem
that one provider follows from another.

#### Boundary case

The boundary lesson is that propositional agreement is not proof-term identity. All three terms inhabit
the same proposition, but their compiler dependency closures retain different
terminal declarations. Replacing the bundle by three copies of the Jin proof
would preserve the proposition and fail this card's evidence contract.

#### Pedagogical prerequisites

CFT-29-006 ends the common trunk. CFT-32-006 hands off the Jin endpoint.
CFT-35-001 hands off the Lorist–Schwenninger endpoint through Chapter 34. The
Harp handoff is its finite-horizon terminal module. These edges meet here for
comparison; CFT-30–32 never become prerequisites of the LS or Harp proofs.

#### Lean correspondence

The exact `proved-here` public theorem is
`CrouzeixTextbook.Part06.three_route_terminal_bundle` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].
The stable Jin compatibility declaration
`CrouzeixTextbook.Part06.jin_final_comparator` remains a direct alias of
`CrouzeixConjecture.jinFinalCrouzeixConjecture` in
[[formalization/lean/CrouzeixConjecture/FinalTheorems.lean|FinalTheorems.lean]].
The bundle also links the LS provider
`CrouzeixConjecture.loristSchwenningerMainTheorem` in
[[formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean|MainTheorem.lean]]
and the Harp provider
`CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem` in
[[formalization/lean/Crouzeix/Harp/MainTheorem.lean|Harp MainTheorem.lean]].

The compiler target is `CrouzeixTextbook`. Its receipt classifies the bundle
as a theorem, records no underlying declaration, and records direct proof-body
dependencies on all three terminal names. The accepted axiom set is exactly
`Classical.choice`, `Quot.sound`, and `propext`. The maintained coverage row
and generated correspondence ledger carry compiler-normalized type hash
`5ce10ccd8b2e5f6a0a93932256d6df60556b450817f4324eb4bac53cd444962b`
and the exact source location. Those values are regenerated from the same
receipt as the links above.

#### Historical context

This is a three-route comparison, not a fourth proof. The Jin source identity
is
`git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910`.
The LS source identity is
`arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128`. For both routes,
review status means that the local correspondence and proof exposition were
checked against those pinned bytes. The reproduction status means that the named
Lean terminal compiles under the pinned toolchain and its receipt has the
maintained dependency and axiom checks.

The third component is the Harp-derived finite-horizon route at
`formalization/lean/Crouzeix/Harp/MainTheorem.lean#L25`. It is a local derived
formalization, not another source-faithful manuscript route. This account does
not assert priority or acceptance for Jin, LS, or Harp. It states source
identity and local verification status only.

The row classification is `comparison-only; Harp-derived component`. The
contract locator is
`formalization/lean/Crouzeix/Harp/MainTheorem.lean#L25-L172`; the narrower
line-27 link above resolves the terminal declaration itself.

#### ML analogy

Mathematical object / ML counterpart. This card is a proof-architecture
ablation for a nonnormal linear layer. The Jin route asks for a
positive-completion certificate, the LS route asks for a boundary-dilation certificate,
and Harp asks for a finite-horizon atomic certificate at each requested depth.

Exact transfer. For an exact finite complex matrix and polynomial, any one of
the three terminal proofs certifies the same factor-two inequality. The bundle
retains three witnesses, so an audit can distinguish which construction
supplied the bound.

Non-transfer. A learned approximate kernel, sampled boundary, floating-point
cubature rule, or truncated rollout is not one of the exact Lean witnesses.
Agreement of three numerical estimates also gives no statistical guarantee
for nonlinear, stochastic, or time-varying models.

Diagnostic. For positive completion, record the smallest sampled eigenvalue
of each Hermitian kernel or Gram matrix and reject a materially negative
value. For boundary dilation, record the moment residual
`R_k := V*Q^kV - Φ(h^k)` over the inspected powers. For the finite-horizon
route, record the cubature moment residual through `N+1` and the node count.
A finite PSD or moment-residual check is only a diagnostic; it does not prove
the exact infinite family required by the Jin or LS route, nor the quantified
existence of exact Harp certificates for every horizon.

## Worked examples

**Example 1.** To audit Jin, begin at jin_polynomial_constant_two and walk dependencies backward through completion_implies_norm_two and correction_sampling_cancels. To audit Lorist–Schwenninger, begin at lorist_schwenninger_main and walk through realization_norm_two and perturbation_lemma.

**Example 2.** A failed matrix amplification would not refute the scalar constant-two theorem. It would show that an additional complete-boundedness claim lacks a route. Statement boundaries must be preserved even when constants and notation look similar.

## ML bridge

For ML researchers, the main methodological lesson is to distinguish a certificate representation from a recurrence contradiction. Both can certify the same stability bound while exposing different generalization paths. Mechanism comparison should follow dependency interfaces, not superficial similarity of final inequalities.

## Lean translation

The top-level aggregate `CrouzeixTextbook` imports the contract-generated
`CrouzeixTextbook.Correspondence` module. That module imports all 35 chapters
and checks exactly the public declarations selected by the coverage and
exercise contracts; the compiled exporter then records their types and kernel
dependencies. The route-specific terminal modules remain independently
buildable, and the full Harp gate checks the corpus, Atlas export, source
contracts, and Lean libraries together.

## Exercises

### CFT-35-E01 -- retrieval {#exercise-cft-35-e01}

Let `n` be finite and nonempty. Given
`hmain : MainTheoremStatement (n := n)`, `A : SquareMatrix n`, and
`p : Polynomial ℂ`, prove

```text
PolynomialCrouzeixBound A p.
```

Apply the quantified hypothesis to its two arguments. Do not call the LS
terminal theorem. The checked solution is
`CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_01_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].

### CFT-35-E02 -- calculation {#exercise-cft-35-e02}

Given

```text
hmain : ∀ (d : ℕ) [Nonempty (Fin d)],
  MainTheoremStatement (n := Fin d),
```

prove `FiniteMatrixMainTheoremStatement` by unfolding the definition. Explain
why `Nonempty (Fin d)` carries the positive-dimension convention. The checked
solution is
`CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_02_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].

### CFT-35-E03 -- written-proof {#exercise-cft-35-e03}

For finite nonempty `n`, assume
`hmain : MainTheoremStatement (n := n)`. Prove

```text
RationalSpectralSetCorollaryStatement (n := n).
```

Use `rationalSpectralSetCorollary_of_mainTheorem hmain`. In prose, list the
five limiting steps from polynomial approximants through the final
inequality. The checked solution is
`CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_03_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].

### CFT-35-E04 -- written-proof {#exercise-cft-35-e04}

Let `H` be a complete nontrivial complex Hilbert space. Given
`hfinite : FiniteMatrixMainTheoremStatement`, `A : H →L[ℂ] H`, and
`p : Polynomial ℂ`, prove

```text
‖operatorPolynomialEval p A‖ ≤
  2 * supPolynomialModulusOnOperatorNumericalRange A p.
```

Use `hilbertSpacePolynomialCrouzeix_of_mainTheorem hfinite A p`. Identify the
finite Krylov subspace used by the provider. The checked solution is
`CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_04_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].

### CFT-35-E05 -- boundary {#exercise-cft-35-e05}

Let `H` be a complete nontrivial complex Hilbert space. Given
`hfinite : FiniteMatrixMainTheoremStatement` and `A : H →L[ℂ] H`, prove

```text
ClosedOperatorNumericalRangeIsTwoSpectralSet A.
```

Use `closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem hfinite A`.
Then unfold the result and name its compactness, spectrum-containment, and
rational-bound components. The checked solution is
`CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_05_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].

### CFT-35-E06 -- lean-proof {#exercise-cft-35-e06}

Let `n` be a finite nonempty index type, with `A : SquareMatrix n` and
`p : Polynomial ℂ`. Suppose three routes have supplied propositions and
proofs

```text
jinConclusion lsConclusion harpConclusion : Prop
hJin : jinConclusion
hLs : lsConclusion
hHarp : harpConclusion
```

together with the normalization equations

```text
hJinNormalized : jinConclusion = PolynomialCrouzeixBound A p
hLsNormalized : lsConclusion = PolynomialCrouzeixBound A p
hHarpNormalized : harpConclusion = PolynomialCrouzeixBound A p.
```

Prove

```text
PolynomialCrouzeixBound A p ∧ PolynomialCrouzeixBound A p ∧ PolynomialCrouzeixBound A p.
```

Transport each route proof through its own equality before constructing the
conjunction. The direct tuple `⟨hJin, hLs, hHarp⟩` is ill typed because its
entries still have the three route-specific proposition types. A written
solution is

```text
exact ⟨hJinNormalized ▸ hJin,
  hLsNormalized ▸ hLs,
  hHarpNormalized ▸ hHarp⟩
```

The checked theorem is
`CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_06_solution` in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean|Chapter35.lean]].
Its signature and proof use only the supplied propositions, equalities, and
witnesses. No terminal theorem or comparison bundle is a proof dependency.
The `CrouzeixTextbook` compiler receipt records normalized type hash
`a6b5302087094c1fd6c8378f297d6ff79a995904d3e4a6be961a7fe5f3d8cdb7`
and the standard axiom set `Classical.choice`, `Quot.sound`, and `propext`.

### Solution sketches

#### Solution to CFT-35-E01

`MainTheoremStatement` is the function type

```text
∀ A p, PolynomialCrouzeixBound A p.
```

So `hmain A p` has exactly the requested type. The exercise conclusion is one
instance, not a renamed copy of the quantified parent theorem.

#### Solution to CFT-35-E02

After unfolding `FiniteMatrixMainTheoremStatement`, the goal is identical to
the type of `hmain`. The proof is `exact hmain`. The `[Nonempty (Fin d)]`
argument excludes `d=0`, matching the nonempty numerical-range convention.

#### Solution to CFT-35-E03

Apply the provider-neutral adapter
`rationalSpectralSetCorollary_of_mainTheorem`. Its proof constructs polynomial
approximants, proves uniform scalar convergence, derives matrix-evaluation and
maximum convergence, and passes the polynomial inequalities to their limits.
The solution depends on that adapter, not on
`loristSchwenningerRationalSpectralSetCorollary`.

#### Solution to CFT-35-E04

Apply `hilbertSpacePolynomialCrouzeix_of_mainTheorem`. For a unit vector `x`
and `d=p.natDegree`, its proof uses the span of
`x,Ax,...,A^d x`. The compression to this finite-dimensional space agrees with
`p(A)` on `x`, so the finite theorem supplies the required pointwise bound.
Taking the operator norm finishes the proof.

#### Solution to CFT-35-E05

Apply `closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem`. The
result is a conjunction. Its first field is compactness of the closed numerical
range. Its second is spectrum containment. Its third quantifies the rational
constant-two estimate over every rational function pole-free on that set.
The exercise proof uses the neutral constructor rather than the LS parent
theorem.

#### Solution to CFT-35-E06

Equality elimination changes each supplied witness to the common normalized
type. Concretely, `hJinNormalized ▸ hJin` has type
`PolynomialCrouzeixBound A p`, and the LS and Harp terms work the same way.
Only after those three transports do we form the nested conjunction. This
exercise proves a general proposition-transport pattern; it does not call any
of the terminal Crouzeix theorems.

## Synthesis and forward dependencies

The first five declarations give the unnormalized arbitrary-polynomial theorem,
its finite-index form, the rational limit, the theorem for arbitrary
complete nontrivial Hilbert spaces via finite Krylov compression, and the
closed-numerical-range spectral-set predicate. The sixth declaration now
bundles three separately checked witnesses of the finite-matrix theorem.
Their propositions agree, while their proof terms and terminal dependencies
remain distinct. Exercise E06 isolates the logical operation used to compare
such conclusions: normalize each proposition, transport each witness, and
only then assemble the result.
