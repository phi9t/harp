---
id: cft-chapter-30-jin-positive-real-completion
title: Jin’s positive-real completion
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-27
tags: [crouzeix-textbook, constant-two-routes, mathematics, lean]
confidence: high
canonical: 30_jin_positive_real_completion.md
chapter: 30
part: 6
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 30: Jin’s positive-real completion

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-vi-constant-two-routes|Part VI — Constant-two routes]]
Previous: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/29_crouzeix_problem_and_sharpness|Chapter 29 — The Crouzeix problem and sharpness]]
Next: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/31_jin_correction_cancellation|Chapter 31 — Jin’s correction cancellation]]

## Opening problem

A diagonalizable matrix can have every eigenvalue in the unit disk and still
have norm much larger than one because its eigenbasis is badly conditioned.
What extra certificate forces the norm below two? Jin's route uses a
matrix-valued analytic function with positive real part. The function is a
certificate, not the final object. Its four defining conditions eventually
produce a positive matrix inequality. This chapter states the certificate
exactly and changes that inequality into weighted Gramians.

## Conceptual model

There are three representations of the same problem. In the eigenbasis, the
target is the diagonal matrix Λ, but the geometry is distorted by the Gram
matrix G. Taking G=K² and conjugating by K produces the balanced matrix
C=KΛK⁻¹. The completion certificate first yields positivity for matrices
P, R, and X=R-P in the distorted coordinates. Congruence by K⁻¹ then turns
those matrices into weighted Gramians of C, where the eventual norm estimate
can be read from positive-semidefinite inequalities. The point of the chapter
is to make every change of representation explicit.

## Notation

All matrices lie in M_n(ℂ), and Aᴴ denotes conjugate transpose. The
matrices B and T share an eigenbasis S∈GL_n(ℂ). Only B must have simple
spectrum. Write λ:n→ℂ, Λ=diag(λ), G=SᴴS for the Gram matrix of the
basis, K²=G, and K⁻¹ for the two-sided inverse of K. The balanced target is
C=KΛK⁻¹. For q∈ℝ with q>1, define

    Gramian_q(C)=Σ_{k≥0}q⁻ᵏ(Cᵏ)ᴴCᵏ.

The analytic completion remains H(z). The square-root matrix is K throughout
the prose, so the two mathematical objects cannot be confused.

## Source and review boundary

The pinned source locators separate the completion construction
(`the_numerical_range_is_a_2_spectral_set_v4.tex#L319-L386`), the sampled
source inequality (`#L438-L460`), and the balancing and Gramian identities
(`#L464-L498`). All three ranges belong to commit
`565b6a3e0659b6e0785f783b016c3f6d9f171fa5`.
Harp's provider files are a maintained Lean port derived from that revision.
The upstream bytes remain remote-only, and the captured provenance notes that
the revision had no license file. The proofs and examples below are Harp
exposition, not quoted source prose.

### Cumulative historical record

**SOURCE CLAIM.** The pinned manuscript presents the completion construction at
`git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L319-L386`,
the sampled source inequality at the same file's `#L438-L460`, and the
balancing and Gramian identities at `#L464-L498`.
**INSPECTED EVIDENCE.** The native receipt
[[evidence/crouzeix_conjecture/source_manifest.tsv|JIN-565-V4-TEX]]
([source_manifest.tsv#L11](../../../evidence/crouzeix_conjecture/source_manifest.tsv#L11))
records the remote path, byte count, and SHA-256 digest. Harp inspected that
receipt and the commit-pinned semantic locator. The manuscript bytes remain
remote-only in this repository.
**PEER-REVIEW / PUBLICATION STATE.** Harp has no peer-review, acceptance, or
journal-publication receipt for this revision, and makes no priority claim.
The dated Preprints.org metadata receipt does not establish byte identity with
this Git revision.
**HARP REPRODUCTION / FORMALIZATION STATE.** Harp's maintained Lean 4.32 port
compiles the local Chapter 30 declarations. The older upstream clean-build
receipt was blocked by its recorded disk precondition, so it is not evidence
of an upstream clean build.

## Formal development

### CFT-30-001 — positive-real completion {#cft-30-001}

#### Purpose

**Motivation.** Replace “choose a completion” by four obligations that later
proofs can consume one at a time.

#### Statement

This is a definition, not a theorem. It introduces the proposition
`IsPositiveRealCompletion(B,T,H)` by the following four fields; this card does
not assert that a completion exists.

For B,T∈M_n(ℂ), H:ℂ→M_n(ℂ), and 𝔻={z:|z|<1}, define
IsPositiveRealCompletion(B,T,H) by

    H analytic on 𝔻
    ∧ H(0)=I
    ∧ IsPositiveMatrix(Re H(z)) for z∈𝔻
    ∧ H(z)-(I-zT)⁻¹∈alg(Bᴴ) for z∈𝔻.

Here Re H(z)=(H(z)+H(z)ᴴ)/2. IsPositiveMatrix means positive
semidefinite: vᴴ Re H(z)v≥0 for every v. It does not mean strictly positive
definite. The notation alg(Bᴴ) is the unital algebra generated by Bᴴ.

#### Hypothesis ledger

The types are `B,T∈M_n(ℂ)`, `H:ℂ→M_n(ℂ)`, and `𝔻={z:|z|<1}`. This definition
assumes no diagonalization, invertibility, or norm bound.

#### Proof roadmap

Unfold the definition. Read the analytic, normalization, positive-real, and
algebra-defect fields in order; conversely, assemble those four witnesses.

#### Proof

The definitional expansion is

    IsPositiveRealCompletion(B,T,H)
    ↔ analytic ∧ normalized ∧ positive-real ∧ algebra-defect.

The first projection is analyticity on 𝔻. The second is H(0)=I. The third is
pointwise PSD of the real part. The fourth places the resolvent defect in the
generated adjoint algebra. The reverse implication is construction of the
fourfold conjunction. Exercise E01 performs exactly this destructuring and
reconstruction in Lean; it does not prove that a completion exists.

#### Boundary case

At z=0, normalization and the resolvent identity give defect I-I=0. Also
Re I=I⪰0. This checks compatibility at the origin; it does not establish a
completion elsewhere in the disk.

#### Pedagogical prerequisites

Use analytic functions from Chapter 18, PSD matrices from Chapter 8, and
resolvents from Chapter 28.

#### Lean correspondence

Public code: [[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|Chapter30.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/CompletionStatement.lean|CompletionStatement.lean]].
This is an exact local definition alias of
`CrouzeixConjecture.IsPositiveRealCompletion`. Its compiler provider is a
definition, and no proof or existence theorem is claimed here.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.positive_real_completion; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean; provider-declaration=CrouzeixConjecture.IsPositiveRealCompletion; provider-file=formalization/lean/CrouzeixConjecture/CompletionStatement.lean; type-sha256=d72a6d691c4c807c0b20b620c2e030345c8d923a1c9b31d1a2a2e9fb07732523; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The interface is reconstructed from the pinned manuscript,
`the_numerical_range_is_a_2_spectral_set_v4.tex#L319-L386`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt and the
compiler type of both declarations.
**PEER-REVIEW / PUBLICATION STATE.** Those checks are not peer review or a
publication judgment, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The provider definition and exact
local definition alias compile in the isolated Jin target.

#### ML analogy

**Mathematical object / ML counterpart.** H is a structured certificate
function and the defect algebra is its allowed model class.
**Exact transfer.** A deterministic certificate can be checked constraint by
constraint on its stated domain.
**Non-transfer.** Noisy empirical kernels and a floating-point PSD check do
not establish analyticity or PSD on the whole disk.
**Diagnostic.** Compute a four-clause completion residual: analytic residual,
origin residual, sampled smallest eigenvalue of Re H(z), and algebra-defect
residual. A negative sampled eigenvalue refutes a candidate, while
nonnegative samples alone do not prove it.

### CFT-30-002 — the shared-basis theorem {#cft-30-002}

#### Purpose

**Motivation.** Make every quantifier visible and separate the simple
auxiliary spectrum from the possibly repeated target spectrum.

#### Statement

Let n be a nonempty finite type with decidable equality. The theorem is the
following exact quantified implication:

    ∀ (B T : M_n(ℂ)) (H : ℂ→M_n(ℂ))
      (hB : SimpleDiagonalization B) (λ : n→ℂ),
      T=innerConjugation(hB.changeBasis,diag(λ))
      → (∀ i, |λ_i|≤1)
      → IsPositiveRealCompletion(B,T,H)
      → ‖T‖≤2.

Here `hB` packages B=S diag(μ)S⁻¹ with invertible S and pairwise distinct
μ_i; `hB.changeBasis` is S. Thus B and T share S. The matrix B has simple
spectrum. The conclusion is ‖T‖≤2. The simple spectrum belongs to B, not T;
λ need not be injective.

#### Hypothesis ledger

Besides finite decidable n, the proof consumes `[Nonempty n]` in the norm
endpoint. Its interface assumes `hB : SimpleDiagonalization B`, which carries
the fields `eigenvalues`, `changeBasis`, `eq_conjugate`, and
`eigenvalues_injective`; thus it presents B=S diag(μ)S⁻¹ with μ_i distinct.
The compiled algebra-to-diagonal-correction route uses `eigenvalues`,
`changeBasis`, and `eq_conjugate`, but it does not use
`eigenvalues_injective`. Injectivity is therefore carried by the current
interface but unused in this proof term, suggesting that this interface may
admit a future weakening. The remaining hypotheses are
T=S diag(λ)S⁻¹, |λ_i|≤1, and IsPositiveRealCompletion(B,T,H).

#### Proof roadmap

Use the diagonalization data carried by `hB` to express generated-algebra
elements as diagonal corrections in the shared basis, apply completion
positivity, and extract the norm bound.

#### Proof

First, simple spectrum belongs to B, not T. The compiled provider proves the
endpoint through the following explicit chain. Put S=`hB.changeBasis` and
define

    completionPullbackFunction(S,H)(z)=SᴴH(z)S,
    G=completionGramMatrix(S)=SᴴS,
    completionKernelModel(G,λ,d)(z)
      =G diag(j↦(1-zλ_j)⁻¹)+diag(d(z))G.

Here d:ℂ→(n→ℂ) is a diagonal correction. The generated-algebra clause in
CFT-30-001 and the diagonalization data in `hB` let
`exists_completionKernelModel_of_isPositiveRealCompletion` produce d with
d(0)=0 and, for every z in the unit disk,

    completionPullbackFunction(S,H)(z)
      =completionKernelModel(G,λ,d)(z).

More precisely, the local algebra-to-diagonal-correction proof evaluates a
polynomial at `conj(hB.eigenvalues i)`, conjugates through `hB.changeBasis`,
and rewrites B using `hB.eq_conjugate`. No call to
`hB.eigenvalues_injective` occurs. Distinctness remains an assumption of the
`SimpleDiagonalization B` interface, but it is not a dependency of this proof
term. No distinctness condition is placed on λ.

For a matrix function F, define its Herglotz kernel by

    L_F(z,w)=(1-z conj(w))⁻¹(F(z)+F(w)ᴴ).

`matrixHerglotzKernel_isPositiveMatrixKernelOn` says that analyticity of F and
Re F(z)⪰0 make every finite sampled block matrix of L_F positive semidefinite.
Apply it to F(z)=SᴴH(z)S: constant left and right multiplication preserves
analyticity, and Re(SᴴH(z)S)=Sᴴ(Re H(z))S⪰0 by congruence. The displayed model
equality then lets `matrixHerglotzKernel_positive_congr_on` replace F by
`completionKernelModel(G,λ,d)` in every sampled block matrix.

This is the shared-basis completion.

The remaining exact lemma,
`norm_completionDiagonalizableMatrix_le_two_of_positiveKernelModel`, has the
following interface: for invertible S, |λ_i|≤1, d(0)=0, and positivity of this
model kernel on the disk, it proves

    ‖completionDiagonalizableMatrix(S,λ)‖≤2.

Its proof constructs the positive square root of G, obtains the source
inequality for X and P from a finite kernel sample, converts it to the Gramian
inequality by congruence, applies the first-term norm estimate, and transports
back through the polar unitary. Chapters 31 and 32 expand those two internal
workshops; their statements and hypotheses are already explicit here, so the
application chain has no unnamed premise. Finally `hT` identifies
`completionDiagonalizableMatrix(S,λ)` with T. Rewriting yields ‖T‖≤2. The
public Chapter 30 declaration is a direct reexport of this compiled proof.

One local consequence is worth proving now. Diagonal matrices commute:

    diag(μ)diag(λ)=diag(λ)diag(μ).

Mapping this equality through A↦SAS⁻¹ proves BT=TB. Exercise E02 formalizes
this calculation without using the completion endpoint.

#### Boundary case

If every λ_i=0, then T=0 although B still has distinct μ_i. The example shows
why T must be allowed repeated eigenvalues.

#### Pedagogical prerequisites

Use diagonalization and simple spectrum from Chapter 4, nonunitary similarity
from Chapter 5, and CFT-30-001.

#### Lean correspondence

Public code: [[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|Chapter30.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/CompletionDiagonalization.lean|CompletionDiagonalization.lean]].
This is an exact reexport of the theorem
`CrouzeixConjecture.positiveRealCompletionStatement`, whose conclusion is the
proposition `PositiveRealCompletionStatement` displayed above.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.positive_real_completion_statement; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean; provider-declaration=CrouzeixConjecture.positiveRealCompletionStatement; provider-file=formalization/lean/CrouzeixConjecture/CompletionDiagonalization.lean; type-sha256=720010d2fd3b19be340951f1fda1fd0e02a095476a0bdd4a60e8af086352deb2; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The quantified theorem follows the pinned manuscript at
`the_numerical_range_is_a_2_spectral_set_v4.tex#L319-L386`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, binder
structure, and B-versus-T simplicity distinction.
**PEER-REVIEW / PUBLICATION STATE.** This is no external review or publication
claim, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The proved theorem reexport and
independent commutation exercise compile without an LS import.

#### ML analogy

**Mathematical object / ML counterpart.** B is an auxiliary probe and T is a
target representation sharing its feature basis. **Source intuition only, not
a compiled dependency:** distinct probe coordinates may identify basis
components even when target outputs collide.
**Exact transfer.** The exact compiled transfer is shared-basis
diagonalization: `hB.eigenvalues`, `hB.changeBasis`, and `hB.eq_conjugate`
turn generated-algebra elements into diagonal corrections in that basis.
**Non-transfer.** Approximate joint diagonalization, noisy empirical kernels,
and a floating-point PSD check do not satisfy the exact shared-basis theorem.
**Diagnostic.** Measure two diagonalization residuals, one for B and one for
T, together with max_i|λ_i| and the smallest sampled eigenvalue of the
completion real part. Reject the analogy if the fitted bases differ beyond
tolerance.

### CFT-30-003 — denominator-four Gramian {#cft-30-003}

#### Purpose

**Motivation.** Turn an entrywise geometric-kernel matrix into a sum of squared
operator powers in balanced coordinates.

#### Statement

Assume G=K², K⁻¹K=KK⁻¹=I, |λ_i|≤1, and P∈M_n(ℂ), where

    P_ij=G_ij/(1-conj(λ_i)λ_j/4).

Set C=K diag(λ)K⁻¹. Then

    K⁻¹PK⁻¹=Gramian_4(C).

#### Hypothesis ledger

The data are G=K², K⁻¹K=KK⁻¹=I, |λ_i|≤1, and P∈M_n(ℂ). The Lean
square-root record also supplies K=Kᴴ and K⁻¹=(K⁻¹)ᴴ.

#### Proof roadmap

Expand the scalar denominators geometrically, identify each diagonal-basis
term, conjugate termwise, then pass through the norm-convergent sum.

#### Proof

With Λ=diag(λ), the ratio has norm at most 1/4. Therefore

    P_ij=Σ_{k≥0}(conj(λ_i)λ_j/4)^kG_ij,

and the kth matrix is `4⁻ᵏ(Λᵏ)ᴴGΛᵏ`. Since Cᵏ=KΛᵏK⁻¹,

`K⁻¹[4⁻ᵏ(Λᵏ)ᴴGΛᵏ]K⁻¹=4⁻ᵏ(Cᵏ)ᴴCᵏ`.

The powers of C are bounded by ‖K‖‖K⁻¹‖, so the weighted series converges in
norm and multiplication by K⁻¹ passes through its sum. Hence

`Σ_k 4⁻ᵏ(Cᵏ)ᴴCᵏ=Gramian_4(C)`,

which proves the claimed congruence.

**2×2 worked instance.** For λ=(a,b) and
G=[g11 g12; conj(g12) g22],

    P_12=g12/(1-conj(a)b/4)
        =g12+(conj(a)b/4)g12+(conj(a)b/4)²g12+⋯.

The other three entries follow by the same scalar geometric sum.

#### Boundary case

If λ=0, then P=G, C=0, Gramian_4(0)=I, and the identity becomes
K⁻¹GK⁻¹=I.

#### Pedagogical prerequisites

Use PSD congruence from Chapter 8, geometric series from Chapter 24, and
CFT-30-001.

#### Lean correspondence

Public code: [[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|Chapter30.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean|CompletionGramianBridge.lean]].

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.completion_gramian_four; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean; provider-declaration=CrouzeixConjecture.completionP_congruence_eq_gramian_four; provider-file=formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean; type-sha256=d999ab712880d5363dbc792dfeaa9c208ec4eae8435b470a1c25b4344c83fbdc; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The balancing identity for P and its denominator-four
Gramian are traced to
`the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L498`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, that exact
balancing range, the series calculation, and the compiler type.
**PEER-REVIEW / PUBLICATION STATE.** This is not an external publication
judgment, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The provider proves convergence,
termwise congruence, and equality of sums.

#### ML analogy

**Mathematical object / ML counterpart.** G is feature geometry, K⁻¹ is the
exact whitening map, and Gramian_4 accumulates discounted feature propagation.
**Exact transfer.** Under G=K² and K⁻¹K=KK⁻¹=I, exact whitening means

    K⁻¹ G K⁻¹ = K⁻¹ K² K⁻¹ = I.

Indeed, associativity rewrites the middle expression as
(K⁻¹K)(KK⁻¹)=I. Only associativity and the two inverse identities are used;
no commutation hypothesis with an unrelated matrix is needed. The same
invertible coordinate change transports each quadratic contribution by
congruence.
**Non-transfer.** Regularized empirical covariance changes G, truncation adds
a tail error, and floating-point PSD does not prove the infinite identity.
**Diagnostic.** Report the denominator-four congruence residual by comparing
K⁻¹PK⁻¹ with partial sums in operator or Frobenius norm. Use the smallest
eigenvalue only for PSD claims, not for equality residuals.

### CFT-30-004 — denominator-two Gramian {#cft-30-004}

#### Purpose

**Motivation.** Build the slower-discounted Gramian in the same coordinates so
that its difference from the denominator-four Gramian is exact.

#### Statement

Under G=K², K⁻¹K=KK⁻¹=I, |λ_i|≤1, and R∈M_n(ℂ), define

    R_ij=G_ij/(1-conj(λ_i)λ_j/2).

For C=K diag(λ)K⁻¹,

    K⁻¹RK⁻¹=Gramian_2(C).

#### Hypothesis ledger

The hypotheses are G=K², K⁻¹K=KK⁻¹=I, |λ_i|≤1, and R∈M_n(ℂ), together
with the stored self-adjointness of K and K⁻¹.

#### Proof roadmap

Use the ratio conj(λ_i)λ_j/2, transport each diagonal-basis term, and sum.

#### Proof

The disk bound gives ratio norm at most 1/2, so

    R_ij=Σ_{k≥0}(conj(λ_i)λ_j/2)^kG_ij.

The kth matrix is `2⁻ᵏ(Λᵏ)ᴴGΛᵏ`, and the square-root identities give

`K⁻¹[2⁻ᵏ(Λᵏ)ᴴGΛᵏ]K⁻¹=2⁻ᵏ(Cᵏ)ᴴCᵏ`.

Summability then yields

`Σ_k 2⁻ᵏ(Cᵏ)ᴴCᵏ=Gramian_2(C)`

and the claimed congruence. In the 2×2 instance above,
R_12=g12/(1-conj(a)b/2). Its first correction is conj(a)g12b/2, not the
denominator-four correction.

#### Boundary case

Even when |λ_i|=1, the ratio has norm at most 1/2<1. Closed-disk eigenvalues
still give a convergent series.

#### Pedagogical prerequisites

Use CFT-30-003 and the same geometric-series and congruence facts.

#### Lean correspondence

Public code: [[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|Chapter30.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean|CompletionGramianBridge.lean]].

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.completion_gramian_two; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean; provider-declaration=CrouzeixConjecture.completionR_congruence_eq_gramian_two; provider-file=formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean; type-sha256=e9a78b5db14824f47398ed1a8a9cdbfcc1bbfae558930039ddbbb5d431dd3f84; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The balancing identity for R and its denominator-two
Gramian are tied to
`the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L498`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, that exact
balancing range, and the denominator-two case independently of P.
**PEER-REVIEW / PUBLICATION STATE.** No external review or publication claim
follows, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** Its convergence and congruence
compile.

#### ML analogy

**Mathematical object / ML counterpart.** Gramian_2 is a longer-horizon
discounted feature accumulator than Gramian_4.
**Exact transfer.** Changing a fixed discount changes every time-step weight
while preserving the coordinate map.
**Non-transfer.** Learned discounts, noisy empirical kernels, and
floating-point finite-horizon estimators are different objects.
**Diagnostic.** Report the denominator-two tail residual together with the
partial-sum equality residual in operator or Frobenius norm. Stalled
convergence signals conditioning or a failed power bound. An eigenvalue
diagnostic is appropriate only after a matrix is known Hermitian and the
claim being tested is PSD.

### CFT-30-005 — the Gramian difference {#cft-30-005}

#### Purpose

**Motivation.** Isolate the positive weighted tail used in norm extraction.

#### Statement

Define X:=R-P. Assume G=K², K⁻¹K=KK⁻¹=I, and |λ_i|≤1. The
conclusion is the single congruence

    K⁻¹XK⁻¹=Gramian_2(C)-Gramian_4(C).

#### Hypothesis ledger

Define X:=R-P. The remaining data are G=K², K⁻¹K=KK⁻¹=I, and
|λ_i|≤1. The matrices P and R are those of the preceding cards.

#### Proof roadmap

Distribute the congruence over subtraction and substitute the two established
identities.

#### Proof

Keep the noncommutative order:

    K⁻¹(R-P)K⁻¹
    =K⁻¹RK⁻¹-K⁻¹PK⁻¹
    =Gramian_2(C)-Gramian_4(C).

Termwise,

    Gramian_2(C)-Gramian_4(C)
    =Σ_k(2⁻ᵏ-4⁻ᵏ)(Cᵏ)ᴴCᵏ.

The k=0 coefficient is zero and all later coefficients are nonnegative. In
the 2×2 example, X_12 is the difference of the two displayed fractions. It
may be complex; PSD is not entrywise nonnegativity.

#### Boundary case

For C=0 both Gramians equal I and X becomes zero. The first nonzero tail term
for general C is (1/4)CᴴC.

#### Pedagogical prerequisites

Use CFT-30-003, CFT-30-004, and matrix distributivity.

#### Lean correspondence

Public code: [[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|Chapter30.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean|CompletionGramianBridge.lean]].

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.completion_gramian_difference; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean; provider-declaration=CrouzeixConjecture.completionX_congruence_eq_gramian_difference; provider-file=formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean; type-sha256=2e7c01edefeb5dc2a3212ae85afc54e9caea46aaacf62c7996ad3de1ab1c1b67; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The balanced Gramian difference X=R-P follows the identities
in `the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L498`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, that exact
Gramian range, the subtraction order, and both prior congruences.
**PEER-REVIEW / PUBLICATION STATE.** This local check is not publication
review, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The provider and distinct
exercise proof compile without the Jin endpoint.

#### ML analogy

**Mathematical object / ML counterpart.** X compares two discounted feature
Gramians on identical trajectories.
**Exact transfer.** Slower discount adds termwise nonnegative coefficients.
**Non-transfer.** Differences of noisy empirical covariances need not be PSD,
and a floating-point PSD result is only a numerical observation.
**Diagnostic.** Compare the Gramian-difference eigenvalue, namely the smallest
eigenvalue of the Hermitian difference, with truncation and rounding bounds.
A larger negative value refutes the calculation.

### CFT-30-006 — source positivity to Gramian positivity {#cft-30-006}

#### Purpose

**Motivation.** Expose every order-preserving step between the sampled-kernel
source matrix and the Gramian inequality used in Chapter 32.

#### Statement

Let G,K,K⁻¹∈M_n(ℂ), let λ:n→ℂ satisfy |λ_i|≤1, and set
C=K diag(λ)K⁻¹. Let P, R, and X be the matrices defined in CFT-30-003
through CFT-30-005. Assume G=K², K=Kᴴ, K⁻¹=(K⁻¹)ᴴ, the two inverse
identities, and

    Y=4X-XG⁻¹P-PG⁻¹X⪰0.

Then

    K⁻¹YK⁻¹⪰0

and, writing G₂=Gramian_2(C), G₄=Gramian_4(C),

    4(G₂-G₄)-(G₂-G₄)G₄-G₄(G₂-G₄)⪰0.

#### Hypothesis ledger

The hypotheses are λ:n→ℂ, `|λ_i|≤1`, `G=K²`, `K=Kᴴ`,
`K⁻¹=(K⁻¹)ᴴ`, K⁻¹K=KK⁻¹=I, and
`Y=4X-XG⁻¹P-PG⁻¹X⪰0`. The spectral bound supplies the Gramian
congruences.

#### Proof roadmap

Start with Y⪰0, congruence by K⁻¹, replace G⁻¹, regroup, replace X, replace P,
and read the final Gramian expression.

#### Proof

From Y⪰0, for every v,

    vᴴ(K⁻¹YK⁻¹)v=(K⁻¹v)ᴴY(K⁻¹v)≥0.

The equality consumes K⁻¹=(K⁻¹)ᴴ at this line, so K⁻¹YK⁻¹⪰0. Next G=K²
and the inverse identities give G⁻¹=K⁻¹K⁻¹. Invertibility is consumed here:

    K⁻¹YK⁻¹
    =4(K⁻¹XK⁻¹)
     -(K⁻¹XK⁻¹)(K⁻¹PK⁻¹)
     -(K⁻¹PK⁻¹)(K⁻¹XK⁻¹).

Now replace X by K⁻¹XK⁻¹=G₂-G₄, then replace P by
K⁻¹PK⁻¹=G₄. This gives

    4(G₂-G₄)-(G₂-G₄)G₄-G₄(G₂-G₄)⪰0.

We never subtract arbitrary PSD matrices and assume the result is PSD. The
order comes from Y and survives a genuine congruence.

#### Boundary case

For C=0, G₂=G₄=I and the final matrix is zero. Without self-adjoint K⁻¹, the
valid congruence would be (K⁻¹)ᴴYK⁻¹, so dropping the adjoint would be an error.

#### Pedagogical prerequisites

Use PSD congruence from Chapter 8 and CFT-30-003 through CFT-30-005.

#### Lean correspondence

Public code: [[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|Chapter30.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean|CompletionGramianBridge.lean]].
The public declaration is a local theorem proved here. It exposes the
congruenced PSD intermediate and then invokes the provider for the final
Gramian PSD conclusion.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.completion_gramian_source_positive; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean; provider-declaration=CrouzeixConjecture.completion_gramian_expression_posSemidef_of_source; provider-file=formalization/lean/CrouzeixConjecture/CompletionGramianBridge.lean; type-sha256=2c90dd2e81251f789c757fc791515b9d4889f0c5c8f0e74fc1b7ce75e8f17356; provider-type-sha256=54e0b92ee477409a1b741eb87362322c7a801bb9c9ecdb4b61a1320140728baa; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The sampled source inequality and source-PSD passage are tied
to `the_numerical_range_is_a_2_spectral_set_v4.tex#L438-L460`; its congruence
to the Gramian inequality, especially the terminal identity, is tied to
`the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L498`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, both exact
ranges, and the Hermitian and inverse hypotheses at their use sites.
**PEER-REVIEW / PUBLICATION STATE.** This is not independent journal review,
and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The provider and E06 compile the
congruence and exact expression. Chapter 31 still owes the source PSD.

#### ML analogy

**Mathematical object / ML counterpart.** Congruence is exact feature
reparameterization of a quadratic certificate.
**Exact transfer.** Applying the same linear feature map to both slots
preserves PSD.
**Non-transfer.** Unrelated left-right maps, noisy empirical geometry,
floating-point subtraction, or a nonsymmetric whitening estimate do not
preserve order.
**Diagnostic.** Report the source-to-Gramian congruence residual together with
Hermitian error, inverse error, and the smallest eigenvalue. Reject the
analogy if these exceed a stated budget.

## Worked examples

### Running nonnormal family

Fix

$$
A_{\lambda,\alpha}=\begin{pmatrix}\lambda&\alpha\\0&-\lambda\end{pmatrix}.
$$

In compact text notation this is

    A_{λ,α}=\begin{pmatrix}λ&α\\0&-λ\end{pmatrix}.

When $\lambda\ne0$, put

$$
S_{\lambda,\alpha}=\begin{pmatrix}1&-\alpha/(2\lambda)\\0&1\end{pmatrix}.
$$

Equivalently,

    S_{λ,α}=\begin{pmatrix}1&-α/(2λ)\\0&1\end{pmatrix}.

Direct multiplication, including
$S_{\lambda,\alpha}^{-1}=\left(\begin{smallmatrix}1&\alpha/(2\lambda)\\0&1\end{smallmatrix}\right)$,
gives

$$
A_{\lambda,\alpha}=S_{\lambda,\alpha}
  \operatorname{diag}(\lambda,-\lambda)S_{\lambda,\alpha}^{-1}.
$$

That identity is

    A_{λ,α}=S_{λ,α}\operatorname{diag}(λ,-λ)S_{λ,α}^{-1}.

This is the shared-basis form used by the completion route. An auxiliary
matrix $B=S_{\lambda,\alpha}\operatorname{diag}(\mu_1,\mu_2)
S_{\lambda,\alpha}^{-1}$ has simple spectrum whenever $\mu_1\ne\mu_2$.
The analytic function $H$ must still satisfy all four clauses of
`IsPositiveRealCompletion(B,A_{\lambda,\alpha},H)`. Thus $S$,
$G=S^{\mathrm H}S$, and the matrices $P,R,X$ below are illustrative
completion data. Their
calculation does not prove that the analytic completion H exists.

For the specified member $(\lambda,\alpha)=(1/2,1)$,

$$
A_{1/2,1}=\begin{pmatrix}1/2&1\\0&-1/2\end{pmatrix},\qquad
S=\begin{pmatrix}1&-1\\0&1\end{pmatrix},\qquad
G=S^{\mathrm H}S=\begin{pmatrix}1&-1\\-1&2\end{pmatrix}.
$$

For copyable comparison with Chapter 31:

    A_{1/2,1}=\begin{pmatrix}1/2&1\\0&-1/2\end{pmatrix}
    G=S^{\mathrm H}S=\begin{pmatrix}1&-1\\-1&2\end{pmatrix}

Its eigenvalue list is $(1/2,-1/2)$. Every denominator in $P$ and $R$ is
therefore nonzero, so all entries can be computed exactly. Chapter 31 carries
out that computation and the compensating-vector cancellation. Chapter 32
then studies the nilpotent limit $A_{0,2}$, where the general theorem reaches
equality even though this direct shared-basis parametrization has degenerated.

### A balanced Gramian calculation

Take the nonunitary positive change of coordinates

$$
K=\begin{pmatrix}2&1\\1&2\end{pmatrix},\qquad
G=K^2=\begin{pmatrix}5&4\\4&5\end{pmatrix},\qquad
K^{-1}=\frac13\begin{pmatrix}2&-1\\-1&2\end{pmatrix}.
$$
With $\Lambda=\operatorname{diag}(1,0)$,
$$
C=K\Lambda K^{-1}
 =\frac13\begin{pmatrix}4&-2\\2&-1\end{pmatrix}.
$$

Thus λ=(1,0). Applying the entry definitions separately to the (1,1),
(1,2), (2,1), and (2,2) positions gives

$$
P=\begin{pmatrix}20/3&4\\4&5\end{pmatrix},\qquad
R=\begin{pmatrix}10&4\\4&5\end{pmatrix},\qquad
X=\begin{pmatrix}10/3&0\\0&0\end{pmatrix}.
$$

This example is nontrivial: C is idempotent but is not self-adjoint. Direct
multiplication gives

$$
C^{\mathrm H}C=\frac19\begin{pmatrix}20&-10\\-10&5\end{pmatrix},
$$
$$
\operatorname{Gramian}_4(C)=I+\frac13C^{\mathrm H}C
 =\begin{pmatrix}47/27&-10/27\\-10/27&32/27\end{pmatrix},
$$
$$
\operatorname{Gramian}_2(C)=I+C^{\mathrm H}C
 =\begin{pmatrix}29/9&-10/9\\-10/9&14/9\end{pmatrix}.
$$

Now multiply on both sides, in order:

$$
K^{-1}PK^{-1}
 =\begin{pmatrix}47/27&-10/27\\-10/27&32/27\end{pmatrix}
 =\operatorname{Gramian}_4(C),
$$
$$
K^{-1}RK^{-1}
 =\begin{pmatrix}29/9&-10/9\\-10/9&14/9\end{pmatrix}
 =\operatorname{Gramian}_2(C),
$$
and
$$
K^{-1}XK^{-1}
 =\begin{pmatrix}40/27&-20/27\\-20/27&10/27\end{pmatrix}
 =\operatorname{Gramian}_2(C)-\operatorname{Gramian}_4(C).
$$

The last line also checks subtraction after congruence: the right-hand matrix
is the exact difference of the preceding two Gramians. If λ=(0,0), then
P=R=G and X=0; that degenerate case remains a useful normalization check.

## ML bridge

Weighted Gramians also occur in linear state-space models, where they measure
accumulated energy along repeated applications of a transition matrix. That
is an exact algebraic analogy for the sums Gramian_4(C) and Gramian_2(C).
Whitening by K⁻¹ is likewise an exact change of coordinates for quadratic
forms. The analogy stops short of saying that an empirical covariance or a
learned representation satisfies Jin's analytic completion hypothesis. In a
numerical experiment, report the Hermitian residuals, inverse residuals,
congruence residuals, and smallest eigenvalues before interpreting the
matrices as positive certificates.

## Lean translation

Lean represents M_n(ℂ) by `SquareMatrix n`, conjugate transpose by `Bᴴ`, and
positive semidefiniteness either by `IsPositiveMatrix` for the real-part
interface or `Matrix.PosSemidef` for the Gramian interface. The completion
predicate is `IsPositiveRealCompletion`; `CompletionSquareRootData` packages
G=K², the two inverse identities, and the required self-adjointness facts.
The definition card and five theorem cards link to their public declarations and
underlying
providers. The six exercise solutions in `Exercises.Chapter30` are separate
theorems whose types are checked definitionally against the exercise
contracts.

## Exercises

### CFT-30-E01 -- unpack the completion {#exercise-cft-30-e01}

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|exercise_01_solution]]
is `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_01_solution`.
Return the four completion conjuncts in order. Copyable Lean starter:

    theorem exercise_01_solution
        (B T : SquareMatrix n) (H : ℂ → SquareMatrix n)
        (hcompletion : IsPositiveRealCompletion B T H) :
        AnalyticOnNhd ℂ H unitDisk ∧ H 0 = 1 ∧
          (∀ z ∈ unitDisk, IsPositiveMatrix (rePart (H z))) ∧
          ∀ z ∈ unitDisk, H z - (1 - z • T)⁻¹ ∈ generatedAlgebra Bᴴ := by
      rcases hcompletion with ⟨hanalytic, hzero, hpositive, hdefect⟩
      -- assemble the fields

### CFT-30-E02 -- shared-basis commutation {#exercise-cft-30-e02}

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|exercise_02_solution]]
is `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_02_solution`.
Prove this exact signature; do not use the norm theorem:

    theorem exercise_02_solution
        {n : Type*} [Fintype n] [DecidableEq n]
        (B T : SquareMatrix n) (H : ℂ → SquareMatrix n)
        (hB : SimpleDiagonalization B) (lambda : n → ℂ)
        (hT : T = innerConjugation hB.changeBasis (Matrix.diagonal lambda))
        (hlambda : ∀ i, ‖lambda i‖ ≤ 1)
        (hcompletion : IsPositiveRealCompletion B T H) : Commute B T := by
      -- prove the diagonal commutation, then transport it through the shared basis

### CFT-30-E03 -- P and Gramian_4 {#exercise-cft-30-e03}

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|exercise_03_solution]]
is `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_03_solution`.
Prove the entry formula and its series congruence with every binder shown:

    theorem exercise_03_solution
        {n : Type*} [Fintype n] [DecidableEq n]
        {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
        (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1) (i j : n) :
        completionP G lambda i j =
            G i j / (1 - conj (lambda i) * lambda j / 4) ∧
          Hinv * completionP G lambda * Hinv =
            gramian 4 (completionSimilarity H Hinv lambda) := by
      -- prove both conjuncts

### CFT-30-E04 -- R and Gramian_2 {#exercise-cft-30-e04}

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|exercise_04_solution]]
is `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_04_solution`.
Repeat E03 with the denominator and weight changed to two:

    theorem exercise_04_solution
        {n : Type*} [Fintype n] [DecidableEq n]
        {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
        (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1) (i j : n) :
        completionR G lambda i j =
            G i j / (1 - conj (lambda i) * lambda j / 2) ∧
          Hinv * completionR G lambda * Hinv =
            gramian 2 (completionSimilarity H Hinv lambda) := by
      -- prove both conjuncts independently of E03's theorem

### CFT-30-E05 -- X as a difference {#exercise-cft-30-e05}

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|exercise_05_solution]]
is `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_05_solution`.
Unfold X, distribute the congruence, and prove the exact conjunction:

    theorem exercise_05_solution
        {n : Type*} [Fintype n] [DecidableEq n]
        {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
        (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1) :
        completionX G lambda = completionR G lambda - completionP G lambda ∧
          Hinv * completionX G lambda * Hinv =
            gramian 2 (completionSimilarity H Hinv lambda) -
              gramian 4 (completionSimilarity H Hinv lambda) := by
      -- establish the definition field and the congruence field

### CFT-30-E06 -- positivity congruence {#exercise-cft-30-e06}

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|exercise_06_solution]]
is `CrouzeixTextbook.Part06.Exercises.Chapter30.exercise_06_solution`.
From source PSD, prove the congruenced PSD and the exact ordered-matrix
identity that turns it into the final Gramian expression:

    theorem exercise_06_solution
        {n : Type*} [Fintype n] [DecidableEq n]
        {G H Hinv : SquareMatrix n} (hsqrt : CompletionSquareRootData G H Hinv)
        (lambda : n → ℂ) (hlambda : ∀ i, ‖lambda i‖ ≤ 1)
        (hsource : (4 * completionX G lambda -
          completionX G lambda * G⁻¹ * completionP G lambda -
          completionP G lambda * G⁻¹ * completionX G lambda).PosSemidef) :
        (Hinv * (4 * completionX G lambda -
          completionX G lambda * G⁻¹ * completionP G lambda -
          completionP G lambda * G⁻¹ * completionX G lambda) * Hinv).PosSemidef ∧
        Hinv * (4 * completionX G lambda -
          completionX G lambda * G⁻¹ * completionP G lambda -
          completionP G lambda * G⁻¹ * completionX G lambda) * Hinv =
          4 * (gramian 2 (completionSimilarity H Hinv lambda) -
            gramian 4 (completionSimilarity H Hinv lambda)) -
          (gramian 2 (completionSimilarity H Hinv lambda) -
            gramian 4 (completionSimilarity H Hinv lambda)) *
              gramian 4 (completionSimilarity H Hinv lambda) -
          gramian 4 (completionSimilarity H Hinv lambda) *
            (gramian 2 (completionSimilarity H Hinv lambda) -
              gramian 4 (completionSimilarity H Hinv lambda)) := by
      -- preserve PSD by congruence, then prove the ordered identity

### Written solutions

#### CFT-30-E01 solution

The hypothesis already has the four-field conjunction as its definitional
shape. Destructure it as
`⟨hanalytic,hzero,hpositive,hdefect⟩`. These terms have, in order,
analyticity on the unit disk, `H 0=1`, pointwise positivity of `rePart (H z)`,
and membership of the resolvent defect in `generatedAlgebra Bᴴ`. Rebuild the
goal as `⟨hanalytic,hzero,hpositive,hdefect⟩`. This checks the exact field
order; it neither constructs H nor assumes the existence of a completion.

#### CFT-30-E02 solution

The diagonal representatives commute entrywise:

    diag(μ)diag(λ)=diag(λ)diag(μ).

In Lean this is `Matrix.commute_diagonal hB.eigenvalues lambda`. The equation
`hB.eq_conjugate` rewrites B as the image of `diag(μ)` under the algebra
equivalence `innerConjugation hB.changeBasis`; `hT` gives the same statement
for T and `diag(λ)`. Mapping the diagonal commutation through this one algebra
equivalence preserves both multiplication orders, hence BT=TB. The disk bound
and completion hypothesis occur in the exercise's exact interface but are not
needed for this algebraic consequence; the proof does not smuggle in the norm
endpoint.

#### CFT-30-E03 solution

The entry formula is the definition of P. The disk bound makes
`|conj(λ_i)λ_j/4|≤1/4<1`, so the scalar geometric series yields

    P_ij=Σ_{k≥0}(conj(λ_i)λ_j/4)^k G_ij.

The matrix form of the kth summand is `4⁻ᵏ(Λᵏ)ᴴGΛᵏ`. The square-root data give
`Cᵏ=KΛᵏK⁻¹`, whence

    K⁻¹[4⁻ᵏ(Λᵏ)ᴴGΛᵏ]K⁻¹=4⁻ᵏ(Cᵏ)ᴴCᵏ.

Summability permits multiplication by K⁻¹ to pass through the total sum.
Termwise congruence followed by equality of the sums therefore gives
K⁻¹PK⁻¹=Gramian_4(C). The Lean proof follows these three equalities rather
than invoking the public Chapter 30 checkpoint.

#### CFT-30-E04 solution

Now `|conj(λ_i)λ_j/2|≤1/2<1`, and the geometric expansion is

    R_ij=Σ_{k≥0}(conj(λ_i)λ_j/2)^k G_ij.

Its kth matrix term is `2⁻ᵏ(Λᵏ)ᴴGΛᵏ`. Congruence by K⁻¹ changes this term to
`2⁻ᵏ(Cᵏ)ᴴCᵏ`, using the same two inverse identities but a different weight.
After moving both matrix multiplications through the summable series, equality
of corresponding terms yields K⁻¹RK⁻¹=Gramian_2(C). Keeping this calculation
separate from E03 makes the denominator-two convergence bound and every
discount coefficient explicit and independently checkable.

#### CFT-30-E05 solution

The first conjunct is definitional: `completionX G λ` reduces to
`completionR G λ-completionP G λ`. For the second, independently reconstruct
the q=2 and q=4 total-sum congruences as in E03 and E04. Matrix distributivity,
with multiplication order unchanged, gives

    K⁻¹(R-P)K⁻¹
    =K⁻¹RK⁻¹-K⁻¹PK⁻¹
    =Gramian_2(C)-Gramian_4(C).

No order argument is used here; this is an equality in a noncommutative matrix
algebra. In Lean, `mul_sub` distributes the left multiplication and `sub_mul`
distributes the right multiplication before the two reconstructed congruences
are substituted.

#### CFT-30-E06 solution

Let `Y=4X-XG⁻¹P-PG⁻¹X`. From `Y.PosSemidef`, the standard congruence lemma gives
`(Hinvᴴ*Y*Hinv).PosSemidef`. The square-root record supplies
`Hinvᴴ=Hinv`; rewriting at this exact point proves K⁻¹YK⁻¹⪰0, the first
conjunct. Next use `G⁻¹=K⁻¹K⁻¹` and regroup without commuting factors:

    K⁻¹YK⁻¹
    =4(K⁻¹XK⁻¹)-(K⁻¹XK⁻¹)(K⁻¹PK⁻¹)
      -(K⁻¹PK⁻¹)(K⁻¹XK⁻¹).

Substitute `K⁻¹XK⁻¹=G₂-G₄` and `K⁻¹PK⁻¹=G₄`. This proves the
exercise's second conjunct, the exact identity

    K⁻¹YK⁻¹=4(G₂-G₄)-(G₂-G₄)G₄-G₄(G₂-G₄).

Together with the first conjunct, equality transport recovers the final PSD
claim of CFT-30-006. E06 is therefore a distinct internal bridge rather than a
second declaration of the public theorem's proposition; it never infers that
a difference of arbitrary PSD matrices is PSD.

The Lean proof deliberately performs this chain from the smaller lemmas
`completionGram_inverse_eq`, `completion_congruence_identity`,
`completionX_congruence_eq_gramian_difference`, and
`completionP_congruence_eq_gramian_four`. It does not invoke
`completion_PRX_congruence_eq_gramian_expression`, which would package the
entire equality being exercised.

## Lean exercise audit

The six distinct solutions are in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|Chapter30.lean]]
under CrouzeixTextbook.Part06.Exercises.Chapter30. Their compiler types are
matched to the six contract probes. The exporter rejects aliases, extra
namespace declarations, parent-provider shortcuts, and terminal shortcuts.

## Synthesis and forward dependencies

The completion predicate supplies analytic and order data. The shared basis
separates the simple auxiliary B from the possibly repeated spectrum of T.
The P, R, and X calculations convert entrywise kernels to balanced Gramians.
The last card says exactly how a sampled source inequality will enter the norm
argument. Chapter 31 must now prove that source inequality.
