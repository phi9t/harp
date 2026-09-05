---
id: cft-chapter-31-jin-correction-cancellation
title: Jin’s correction cancellation
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-27
tags: [crouzeix-textbook, constant-two-routes, mathematics, lean]
confidence: high
canonical: 31_jin_correction_cancellation.md
chapter: 31
part: 6
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 31: Jin’s correction cancellation

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-vi-constant-two-routes|Part VI — Constant-two routes]]
Previous: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/30_jin_positive_real_completion|Chapter 30 — Jin’s positive-real completion]]
Next: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/32_jin_constant_two_endpoint|Chapter 32 — Jin’s constant-two endpoint]]

## Notation fixed before the argument

Chapter 30 defined three Hermitian matrices. We restate their entries here so
the sampling calculation can be read on its own:

    P_ij=G_ij/(1-conj(λ_i)λ_j/4),
    R_ij=G_ij/(1-conj(λ_i)λ_j/2),
    X=R-P.

For any one-variable matrix function K, its two-variable Herglotz kernel is

    L_K(z,w)=(1-zconj(w))⁻¹(K(z)+K(w)ᴴ).

Thus K(z) is an n×n matrix at one point, while L_K(z,w) is an n×n kernel
value at an ordered pair. The proof below never writes K with two arguments.

## Opening problem

Chapter 30 produced a positive-real completion, but its analytic diagonal
correction is unknown. Estimating that correction would lose the sharp
constant. Jin's move is better: choose a finite family of sample points and
test vectors so that the correction contributes exactly zero. What remains is
a quadratic form built only from the known matrices P, R, and X.

The calculation is delicate because five kinds of positivity or
nonnegativity appear nearby. We will keep them separate: pointwise
positive-real behavior of a matrix function, sampled-kernel PSD, block-matrix
PSD, a scalar quadratic inequality, and final X PSD. None of these phrases is
a synonym for another.

## Conceptual model

Think of the proof as an exact elimination argument. The completion theorem
introduces an unknown diagonal function D because that freedom makes analytic
positivity possible. We then augment the eigenvalue samples with the origin
and choose the origin vector v by solving one linear system. That choice puts
the residual Gv+Pu in the nullspace of every diagonal coefficient at once.
The nuisance term disappears from the finite quadratic form, leaving an
explicit Hermitian matrix whose positivity can be tested for every u.

This order is essential: define the samples and their vector dimensions,
expand all four known blocks, isolate the unknown half-contribution, cancel
it, and only then invoke the quadratic-form characterization of PSD.

## Data and dimensions

Let n be a finite index type. We regard vectors as columns in ℂⁿ and matrices
as elements of M_n(ℂ). Fix G∈M_n(ℂ), a function λ:n→ℂ, and
Λ=diag(λ). Let d:ℂ→(n→ℂ), set D(z)=diag(d_i(z)), and define

    Q(z)=diag((1-zλ_j)⁻¹),
    K(z)=GQ(z)+D(z)G.

This is the definition used by Lean at every z. If 1-zλ_j≠0 for every j,
then I-zΛ is invertible and

    Q(z)=(I-zΛ)⁻¹.

At a pole, Lean's scalar inverse is still a total operation, but Q(z) must not
be identified with a matrix inverse. We split the model into K_0(z)=GQ(z) and
K_D(z)=D(z)G, and write their Herglotz kernels as L_0 and L_D. Therefore
L_K=L_0+L_D. Every one of these values is in M_n(ℂ). For u∈ℂⁿ, write e_i
for the ith coordinate vector and define

    z_0=0,                  x_0=v=-G⁻¹Pu∈ℂⁿ,
    z_i=conj(λ_i)/2,        x_i=u_i e_i∈ℂⁿ.

The sparse vector x_i has only one nonzero coordinate. The origin vector v is
chosen before any kernel sum is expanded. Its sole job is to enforce
Gv+Pu=0. The finite sample index is Option n: `none` denotes the origin and
`some i` denotes the eigenvalue sample.

## Source and review boundary

The pinned Jin source separates the kernel decomposition and origin
normalization
(`git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L346-L373`)
from the sampling, cancellation, and source-positivity calculation in the same
file (`#L387-L463`).
The provider module is Harp's maintained Lean reconstruction. The chapter
derives the displayed formulas rather than quoting source prose. Publication,
review, and local reproduction are reported separately in every theorem card.

### Cumulative historical record

**SOURCE CLAIM.** The pinned manuscript gives the kernel decomposition and
origin normalization at
`git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L346-L373`
and the sampled-kernel, correction-cancellation, and source-positivity stage at
the same file's `#L387-L463`.
**INSPECTED EVIDENCE.** The native receipt
[[evidence/crouzeix_conjecture/source_manifest.tsv|JIN-565-V4-TEX]]
([source_manifest.tsv#L11](../../../evidence/crouzeix_conjecture/source_manifest.tsv#L11))
records the remote path, byte count, and digest. Harp inspected that receipt,
the commit-pinned locator, and the local provider declarations.
**PEER-REVIEW / PUBLICATION STATE.** Harp has no peer-review, acceptance, or
journal-publication receipt for this revision, and makes no priority claim.
The metadata-only Preprints.org observation cannot establish review status or
byte identity.
**HARP REPRODUCTION / FORMALIZATION STATE.** Harp's Lean 4.32 port compiles the
sample blocks, correction pairing, and cancellation theorem. This local result
is separate from the older upstream clean-build receipt, which was blocked by
its recorded disk precondition.

## Formal development

### CFT-31-001 — completion kernel model {#cft-31-001}

#### Purpose

**Motivation.** Split the completion into a resolvent term whose sampled
blocks can be computed and a diagonal correction whose contribution will be
canceled.

#### Statement

For G∈M_n(ℂ), Λ=diag(λ), D(z)=diag(d_i(z)), and

    Q(z)=diag((1-zλ_j)⁻¹),

define

    K(z)=GQ(z)+D(z)G.

The exact entry formula is

    K(z)_ij=G_ij(1-zλ_j)⁻¹+d_i(z)G_ij.

On the pole-free domain 1-zλ_j≠0 for every j, diagonal inversion gives

    Q(z)=(I-zΛ)⁻¹.

#### Hypothesis ledger

The data are G∈M_n(ℂ), Λ=diag(λ), D(z)=diag(d_i(z)), and
Q(z)=diag((1-zλ_j)⁻¹). No positivity, Hermiticity, or invertibility is needed
to define K. The optional identification Q(z)=(I-zΛ)⁻¹ needs the stated
pole-free condition.

#### Proof roadmap

Compute the two matrix products entrywise. The diagonal matrices collapse
each finite sum to one term, and then the two surviving entries add.

#### Proof

The right diagonal factor selects column j:

    [GQ(z)]_ij=G_ij(1-zλ_j)⁻¹.

The left diagonal factor selects row i:

    [D(z)G]_ij=d_i(z)G_ij.

Adding the two formulas gives

    K(z)_ij=G_ij(1-zλ_j)⁻¹+d_i(z)G_ij.

The first Lean exercise also checks sparse-vector column selection:
`A(completionSparseVector(1,j))` has ith coordinate `A_ij`.

#### Boundary case

At z=0 the resolvent part is G. The correction has not yet vanished, so the
formula is K(0)=G+D(0)G. That distinction drives the reverse theorem in the
next card.

#### Pedagogical prerequisites

Use diagonal matrices and matrix multiplication from Chapters 3 and 6, and
resolvents from Chapter 28.

#### Lean correspondence

Public code: [[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|Chapter31.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean|CompletionKernelModel.lean]].
The public definition reexports `CrouzeixConjecture.completionKernelModel`.
The entry calculation is proved by `completionKernelModel_apply`.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.completion_kernel_model; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean; provider-declaration=CrouzeixConjecture.completionKernelModel; provider-file=formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean; type-sha256=b6f13ba7834ee232d6361bbf95f939ad00760f82d267ffde7b3c22ed1b5a48b1; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The kernel decomposition follows the pinned manuscript at
`the_numerical_range_is_a_2_spectral_set_v4.tex#L346-L373`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, that exact
decomposition range, the matrix multiplication, and the compiled provider.
**PEER-REVIEW / PUBLICATION STATE.** This check supplies no peer-review or
publication receipt, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The definition and entry theorem
compile in the Jin and textbook targets.

#### ML analogy

**Mathematical object / ML counterpart.** G records feature geometry, while
D is a nuisance correction to a structured feature map. **Exact transfer.**
The finite sample matrix is a finite certificate once all entries are exact.
**Non-transfer.** noisy empirical kernels and floating-point PSD checks do not
prove the analytic identity. **Diagnostic.** Compute the entrywise model residual
`max_ij |K(z)_ij-G_ij(1-zλ_j)⁻¹-d_i(z)G_ij|` at pole-free test points. This
checks this card's algebra without pretending to certify kernel positivity.

### CFT-31-002 — normalization forces the correction to vanish {#cft-31-002}

#### Purpose

**Motivation.** Prove the direction the cancellation argument actually uses:
normalization of K at the origin forces D(0)=0 when G is invertible.

#### Statement

Assume K(0)=G, G invertible, and D(0) diagonal. Setting z=0 in the model gives

    K(0)=G+D(0)G.

Hence D(0)G=0, and right cancellation by G yields

    D(0)=0.

The order matters: this card proves normalization implies the zero
correction. The older forward theorem starts from d(0)=0 and is not this
result.

#### Hypothesis ledger

The hypotheses are K(0)=G, G invertible, and D(0) diagonal. In Lean,
invertibility is `IsUnit G`; the diagonal matrix is
`completionDiagonalCorrection d 0`.

#### Proof roadmap

Evaluate the kernel model at zero, subtract G, use right-invertible
cancellation by the two-sided matrix inverse, and read the diagonal entries.

#### Proof

From the model and the hypothesis K(0)=G,

    G+D(0)G=G,
    D(0)G=0.

Because G is right-invertible, GG⁻¹=I. Associativity now gives the complete
cancellation in one line:

    D(0)=D(0)(GG⁻¹)=(D(0)G)G⁻¹=0.

In particular,

    D(0)=0.

Taking the ith diagonal entry gives d_i(0)=0 for every i, hence d(0)=0 as a
function. This finishes the reverse implication without appealing to the
forward normalization theorem.

#### Boundary case

Invertibility is essential. If G=0, then K(0)=G+D(0)G=0 for every D(0), so
normalization gives no information about the correction.

#### Pedagogical prerequisites

Use two-sided matrix inverses, cancellation, and extensional equality of
diagonal matrices.

#### Lean correspondence

Public code and provider bridge:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|Chapter31.lean]].
`completion_kernel_normalization_forces_correction_zero` is proved here and
depends on the local theorem
`completion_kernel_normalization_forces_correction_zero_bridge`. The older
provider theorem `completionKernelModel_zero` proves the opposite direction
and is not used as this card's proof.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.completion_kernel_normalization_forces_correction_zero; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean; provider-declaration=CrouzeixTextbook.Part06.completion_kernel_normalization_forces_correction_zero_bridge; provider-file=formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean; type-sha256=c366aa91bab218699594028032e0feca8e1d84f06804e10dd48e71a7625ccb49; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook; provider-type-sha256=22fb9474d934ab848aa65584ab7f1a7122b901afc9aae3553554bd415ac1eefe}

#### Historical context

**SOURCE CLAIM.** Origin normalization, including the evaluation at the origin
near line 364, belongs to
`the_numerical_range_is_a_2_spectral_set_v4.tex#L346-L373`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, that exact
origin-normalization range, and the reverse implication separately because the
preexisting provider exposes only the forward theorem.
**PEER-REVIEW / PUBLICATION STATE.** This is no external review or publication
claim, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** Harp proves the reverse theorem
locally with no new axiom.

#### ML analogy

**Mathematical object / ML counterpart.** Invertible G is nondegenerate
feature geometry; D(0) is an unobserved featurewise correction. **Exact
transfer.** In exact arithmetic, a zero correction action on an invertible
feature metric forces the correction itself to vanish, giving a finite
certificate. **Non-transfer.** noisy empirical kernels and floating-point PSD
checks replace equality by a conditioning-sensitive residual. **Diagnostic.**
Use the actual right-cancellation estimate, rather than a generic condition
number warning,

    ‖D(0)‖≤‖K(0)-G‖‖G⁻¹‖.

Thus the normalization residual and the inverse norm, rather than a raw
residual multiplied by κ(G), control the inferred correction. Equivalently,
after dividing by ‖G‖, the relative residual is amplified by κ(G). No
eigenvalue test is needed for this theorem.

### CFT-31-003 — the four resolvent blocks {#cft-31-003}

#### Purpose

**Motivation.** Expose the whole block quadratic form before simplification,
so every factor of two and every adjoint has a visible origin.

#### Statement

Use z_0=0, z_i=conj(λ_i)/2, x_0=v=-G⁻¹Pu, and x_i=u_i e_i∈ℂⁿ. The known
resolvent kernel has four blocks:

    sample/sample=4R-2P,
    sample/origin=G+R,
    origin/sample=G+R,
    origin/origin=2G.

Thus the finite quadratic sum over Option n is

    vᴴ(2G)v+vᴴ(G+R)u+uᴴ(G+R)v+uᴴ(4R-2P)u.

Every displayed matrix is n×n and every term is a scalar.

#### Hypothesis ledger

The samples are z_0=0 and z_i=conj(λ_i)/2. The vectors are
x_0=v=-G⁻¹Pu and x_i=u_i e_i∈ℂⁿ. We use G=Gᴴ and |λ_i|≤1; the latter keeps
the scalar denominators nonzero and the samples inside the disk. For the
entry calculation abbreviate

    a_ij=conj(λ_i)λ_j.

#### Proof roadmap

Evaluate the Herglotz kernel at each ordered pair of sample types. Then use
sparse-vector column selection and sum the surviving coordinates.

#### Proof

Set

    a_ij=conj(λ_i)λ_j.

Write K_0(z)=GQ(z), so L_0=L_{K_0}. At two nonzero samples, the outer
Herglotz denominator is `1-a_ij/4`. Hermiticity of G turns the adjoint entry
into a second copy with denominator `1-a_ij/2`. Therefore

    L_0(z_i,z_j)_ij
      =(1-a_ij/4)⁻¹(2G_ij(1-a_ij/2)⁻¹).

The only rational simplification in the main block is

    2/((1-a_ij/4)(1-a_ij/2))=4/(1-a_ij/2)-2/(1-a_ij/4).

The right side is exactly `4R_ij-2P_ij`, so

    L_0(z_i,z_j)_ij=(4R-2P)_ij.

The ordered cross blocks must be computed separately. Since K_0(0)=G,

    L_0(z_i,0)_ij
      =K_0(z_i)_ij+K_0(0)ᴴ_ij
      =R_ij+G_ij
      =(G+R)_ij,

and

    L_0(0,z_j)_ij
      =K_0(0)_ij+K_0(z_j)ᴴ_ij
      =G_ij+R_ij
      =(G+R)_ij.

At the origin twice, Hermiticity gives

    L_0(0,0)_ij=(G+Gᴴ)_ij=(2G)_ij.

Thus the four ordered entry identities, now recorded without intermediate
line breaks, are

    L_0(z_i,z_j)_ij=(4R-2P)_ij,
    L_0(z_i,0)_ij=(G+R)_ij,
    L_0(0,z_j)_ij=(G+R)_ij,
    L_0(0,0)_ij=(2G)_ij.

Now sparse selection does actual work. Each x_i=u_i e_i is in ℂⁿ, each
L_0(z_i,z_j) is n×n, and each pairing below is a scalar:

    conj(u_i)L_0(z_i,z_j)_ij u_j=conj(u_i)(4R-2P)_ij u_j.

Summing the surviving coordinates yields

    Σ_i Σ_j conj(u_i)(4R-2P)_ij u_j=uᴴ(4R-2P)u.

The same selection in the two cross blocks gives `uᴴ(G+R)v` and
`vᴴ(G+R)u`; the origin/origin pairing gives `vᴴ(2G)v`. Adding the four
Option n regions proves the displayed finite sum without commuting any
matrix factors.

#### Boundary case

If u=0 then v=0 and all four blocks contribute zero. This checks homogeneity,
but it does not test any coefficient.

#### Pedagogical prerequisites

Use Herglotz kernels, sparse basis vectors, conjugate transpose, and quadratic
forms from Chapters 8, 18, and 24.

#### Lean correspondence

Public code: [[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|Chapter31.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean|CompletionKernelModel.lean]].
The provider proves each selected block and then the complete finite sum.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.sample_origin_quadratic_identity; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean; provider-declaration=CrouzeixConjecture.completionResolventKernel_sampling_quadratic_eq; provider-file=formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean; type-sha256=a19830b823dec21c68e5ab7b248b9d78c9b03518b8605d869ee888192c2db7db; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The four-block calculation reconstructs the pinned argument
at `the_numerical_range_is_a_2_spectral_set_v4.tex#L387-L463`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, every block,
and its dimensions before simplifying the sum.
**PEER-REVIEW / PUBLICATION STATE.** This local algebra check is not publication
review, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The entrywise and summed
identities compile as separate provider theorems.

#### ML analogy

**Mathematical object / ML counterpart.** G is feature geometry and the four
blocks are a structured augmented Gram matrix. **Exact transfer.** Testing a
finite family of features gives a finite certificate for that sample family.
**Non-transfer.** noisy empirical kernels and floating-point PSD checks can
obscure which ordered identity failed. **Diagnostic.** Compute four ordered
block residuals, one each for sample/sample, sample/origin, origin/sample,
and origin/origin. Report them separately before summing the scalar pairings.

### CFT-31-004 — the unknown diagonal contribution {#cft-31-004}

#### Purpose

**Motivation.** Gather every occurrence of the unknown correction into one
scalar Θ and its conjugate before using the compensating vector.

#### Statement

Assume G=G*, let P=Pᴴ, let Δ=diag(d_i(conj(λ_i)/2)), assume d(0)=0, and
use v=-G⁻¹Pu. The exact compiler-facing identity is

    L_D(z_i,z_j)_ij=(ΔP+PΔᴴ)_ij,
    L_D(z_i,0)_ij=(ΔG)_ij,
    L_D(0,z_j)_ij=(GΔᴴ)_ij,
    L_D(0,0)=0.

Consequently, the exact compiler-facing scalar identity is

    correction sampling sum=v*(GΔ*)u+u*(ΔG)v+u*(ΔP+PΔ*)u.

With adjoints displayed typographically, this is

    correction sampling sum=vᴴ(GΔᴴ)u+uᴴ(ΔG)v+uᴴ(ΔP+PΔᴴ)u.

Define

    Θ=u*Δ(Gv+Pu).

Regrouping these three terms shows that the complete correction sampling sum is

    correction sampling sum=Θ+conj(Θ).

#### Hypothesis ledger

The exact hypothesis ledger is

    G=G*
    P=Pᴴ
    Δ=diag(d_i(conj(λ_i)/2))
    d(0)=0
    v=-G⁻¹Pu

Hermiticity of P follows from its entry formula and Hermiticity of G.
Hermiticity of G and P identifies the reverse terms with the conjugate half.

#### Proof roadmap

Compute all four ordered correction blocks. Apply sparse selection, collect
the terms with Δ on the left into Θ, and use G=Gᴴ and P=Pᴴ when conjugating
to obtain the remaining half.

#### Proof

Write K_D(z)=D(z)G. At two eigenvalue samples, sparse selection asks only for
entry `(i,j)`. At that entry the Herglotz denominator converts G into P while
the sampled diagonal factors stay on their original sides:

    L_D(z_i,z_j)_ij=(ΔP+PΔᴴ)_ij.

This is a selected-entry identity, not a full matrix equality: away from row
`i` and column `j`, `D(z_i)` and `D(z_j)` evaluate at fixed sample points and
need not agree with the coordinatewise sampled diagonal Δ.

At sample then origin, d(0)=0 removes the adjoint correction:

    L_D(z_i,0)_ij=(ΔG)_ij.

At origin then sample, the first correction vanishes and G=Gᴴ gives

    L_D(0,z_j)_ij=(GΔᴴ)_ij.

At the origin twice, both correction values vanish:

    L_D(0,0)=0.

Sparse selection and summation now give the raw ordered expression

    correction sampling sum=vᴴ(GΔᴴ)u+uᴴ(ΔG)v+uᴴ(ΔP+PΔᴴ)u.

Matrix multiplication by Δ scales coordinate i. Therefore

    Θ=Σ_i conj(u_i)d_i(conj(λ_i)/2)(Gv+Pu)_i.

Expanding the matrix pairing gives

    Θ=uᴴΔGv+uᴴΔPu.

Conjugating reverses the scalar products. The identities G=Gᴴ and P=Pᴴ are
used here, not silently:

    conj(Θ)=Σ_i conj((Gv+Pu)_i)conj(d_i(conj(λ_i)/2))u_i.

Equivalently,

    conj(Θ)=vᴴGΔᴴu+uᴴPΔᴴu.

The origin/origin correction block is zero because d(0)=0. The other three
blocks are exactly the terms in these two coordinate sums, so

    correction sampling sum=Θ+conj(Θ).

#### Boundary case

If one sampled value d_i(conj(λ_i)/2) is zero, coordinate i makes no
contribution to Θ. Cancellation below is stronger: it works for arbitrary
sampled correction values.

#### Pedagogical prerequisites

Use diagonal multiplication, complex conjugation of inner products, and
finite sums.

#### Lean correspondence

Public code: [[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|Chapter31.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean|CompletionKernelModel.lean]].
The provider's selected-entry scope is explicit in
`completionCorrectionKernel_sample_sample_apply`,
`completionCorrectionKernel_sample_zero_apply`, and
`completionCorrectionKernel_zero_sample_apply`; its origin/origin theorem is
`completionCorrectionKernel_zero_zero`. The raw expanded sum is the public
checkpoint. The regrouping into Θ plus its conjugate is proved by
`completionCorrectionKernel_sampling_eq_unknownContribution` and is also
checked independently by the textbook theorem
`CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_04_solution`.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.correction_sampling_identity; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean; provider-declaration=CrouzeixConjecture.completionCorrectionKernel_sampling_quadratic_eq; provider-file=formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean; type-sha256=3fbcbf6aba9f45dc919a68fa512cb886899febda58461c932c11d93054a74053; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The Θ packaging is the cancellation step at
`the_numerical_range_is_a_2_spectral_set_v4.tex#L387-L463`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, expanded both
halves coordinate by coordinate, and checked the adjoint order.
**PEER-REVIEW / PUBLICATION STATE.** This is no external review or publication
claim, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** Lean proves both the raw
three-block sum and the Θ-plus-conjugate identity.

#### ML analogy

**Mathematical object / ML counterpart.** Δ is a featurewise nuisance term in
the feature geometry G. **Exact transfer.** The nuisance contribution is a
finite certificate scalar Θ+conj(Θ), not an expectation. **Non-transfer.**
noisy empirical kernels and floating-point PSD checks do not preserve exact
conjugate cancellation. **Diagnostic.** Compare the raw correction sum against
2Re(Θ), where `2Re(Θ)=Θ+conj(Θ)`. This isolates block-regrouping error before
the compensating vector is substituted.

### CFT-31-005 — exact cancellation {#cft-31-005}

#### Purpose

**Motivation.** Use the one engineered degree of freedom, the origin vector,
to annihilate every unknown correction coordinate at once.

#### Statement

Assume G=G*, G invertible, v=-G⁻¹Pu, and d(0)=0. Then

    Gv+Pu=0,
    Θ=0,
    Θ+conj(Θ)=0.

Combining this with the exact identity of CFT-31-004 gives

    correction sampling sum=0.

The result holds for every function d; no size estimate for the correction is
needed.

#### Hypothesis ledger

The exact hypothesis ledger is

    G=G*
    G invertible
    v=-G⁻¹Pu
    d(0)=0

Invertibility supplies GG⁻¹=I. Hermiticity identifies the reverse correction
block as the adjoint half, and the zero-at-origin condition removes the
origin/origin correction.

#### Proof roadmap

Substitute v in the vector residual. Then place the zero residual into the
coordinate formula for Θ and conjugate the scalar equality.

#### Proof

First substitute v=-G⁻¹Pu:

    Gv+Pu=-GG⁻¹Pu+Pu=0.

Thus every coordinate of the residual vanishes, and

    Θ=Σ_i conj(u_i)d_i(conj(λ_i)/2)·0=0.

Complex conjugation preserves zero, so

    Θ+conj(Θ)=0.

The raw three-term correction identity from CFT-31-004 is Θ+conj(Θ), hence

    correction sampling sum=0.

This is the key logical point of the chapter. We do not show that D is small.
We choose a test vector on which D is invisible.

#### Boundary case

If G is singular, `Gv=-Pu` need not have a solution for every u. The chosen
formula for v also lacks a two-sided inverse, so the proof stops at its first
line.

#### Pedagogical prerequisites

Use matrix-vector multiplication, inverse cancellation, and the coordinate
formula for Θ from CFT-31-004.

#### Lean correspondence

Public code: [[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|Chapter31.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean|CompletionKernelModel.lean]].
The provider first proves `completionUnknownHalfContribution_eq_zero`, then
uses it in `completionCorrectionKernel_sampling_eq_zero`.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.correction_sampling_cancels; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean; provider-declaration=CrouzeixConjecture.completionCorrectionKernel_sampling_eq_zero; provider-file=formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean; type-sha256=3883ecf653a417da1c0aceb9c8ee7dd5f2c9444bf7fa7936660051df5e071234; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** Exact correction cancellation is the algebraic device in the
pinned route at `the_numerical_range_is_a_2_spectral_set_v4.tex#L387-L463`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, vector
equation, scalar half, and conjugate half as separate claims.
**PEER-REVIEW / PUBLICATION STATE.** This supplies no publication judgment,
and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** Lean proves the cancellation
without assuming the final X inequality.

#### ML analogy

**Mathematical object / ML counterpart.** The compensating vector is an exact
control variate chosen using feature geometry. **Exact transfer.** Its linear
constraint removes a nuisance component from a finite certificate.
**Non-transfer.** noisy empirical kernels and floating-point PSD checks leave
a residual whose size is amplified by κ(G). **Diagnostic.** Report ‖Gv+Pu‖
and the cancellation residual. The first is the compensating-equation
residual; the second is the absolute value of the full correction sampling
sum. Together they measure the engineered annihilation directly.

### CFT-31-006 — positivity becomes the X inequality {#cft-31-006}

#### Purpose

**Motivation.** Convert analytic positivity into the precise matrix inequality
that Chapter 30's Gramian bridge consumes, without skipping a positivity
notion.

#### Statement

Assume L_K positive as a matrix kernel on 𝔻, G=G* and G invertible,
|λ_i|≤1, and d(0)=0. This is where the compiled CFT-31-006 begins with kernel
positivity. Finite sampling gives sampled-kernel PSD. Testing its block matrix
on the selected vector yields, for every u,

    u*(4X-XG⁻¹P-PG⁻¹X)u≥0 for every u.

Because the coefficient is Hermitian, this universal scalar inequality is
equivalent to

    4X-XG⁻¹P-PG⁻¹X⪰0.

#### Hypothesis ledger

The exact inputs, in order, are

    L_K positive as a matrix kernel on 𝔻
    G=G* and G invertible
    |λ_i|≤1
    d(0)=0.

Hermiticity is needed for the final PSD criterion, not for the mere scalar
expansion.

#### Proof roadmap

Move through five named levels: analytic pointwise positivity, finite kernel
positivity, block positivity, a selected scalar test, and the PSD matrix
criterion.

#### Proof

The transitions are deliberately explicit. There are two formal boundaries.
At the source/bridge boundary, analyticity of K on 𝔻 together with pointwise
positive-real behavior implies L_K positive as a matrix kernel on 𝔻.
`matrixHerglotzKernel_isPositiveMatrixKernelOn` proves that implication. The
compiled CFT-31-006 begins with kernel positivity and does not reprove the
analytic bridge.

    analyticity + pointwise positive-real ⇒ L_K kernel-positive
    L_K kernel-positive ⇒ sampled-kernel PSD
    sampled-kernel PSD ⇒ block-matrix PSD
    block-matrix PSD ⇒ finite quadratic sum≥0
    correction sum=0
    finite quadratic sum=u*(4X-XG⁻¹P-PG⁻¹X)u
    Hermitian coefficient + universal quadratic nonnegativity ⇒ matrix PSD

1. Analyticity plus pointwise positive-real behavior makes `L_K`
   kernel-positive. This is the Chapter 30 bridge that produces `hKernel`;
   pointwise positivity alone would not suffice.
2. `L_K` kernel-positive implies sampled-kernel PSD for the finite Option n
   sample family.
3. Sampled-kernel PSD implies block-matrix PSD, because the matrix indexed by
   the selected sample points is one of the kernel's finite Gram matrices.
4. Block-matrix PSD implies the finite quadratic sum is nonnegative when
   tested on the block vector `(x_a)_a`.
5. correction sum=0. CFT-31-005 removes the unknown diagonal term exactly.
6. finite quadratic sum=u*(4X-XG⁻¹P-PG⁻¹X)u. Substituting v=-G⁻¹Pu into
   the four resolvent blocks gives the coefficient

       2PG⁻¹P-P-PG⁻¹R-P-RG⁻¹P+4R-2P.

   Regrouping without commuting any factors,

       2PG⁻¹P-P-PG⁻¹R-P-RG⁻¹P+4R-2P
       =4(R-P)+2PG⁻¹P-PG⁻¹R-RG⁻¹P
       =4X-XG⁻¹P-PG⁻¹X,

   because X=R-P. Thus the selected scalar is exactly
   `u*(4X-XG⁻¹P-PG⁻¹X)u`.
7. Hermitian coefficient and universal nonnegativity imply matrix PSD. First
   verify the hypothesis of the
   quadratic-form criterion. The entry definitions and G=Gᴴ give

       P=Pᴴ,
       R=Rᴴ,
       X=Xᴴ,
       (G⁻¹)ᴴ=G⁻¹.

   Conjugate transpose reverses multiplication order, so

       (XG⁻¹P)ᴴ=PG⁻¹X,
       (PG⁻¹X)ᴴ=XG⁻¹P.

   Therefore

       (4X-XG⁻¹P-PG⁻¹X)ᴴ=4X-XG⁻¹P-PG⁻¹X.

   This Hermiticity identity, followed by the scalar inequality for every u,
   gives the final transition `quadratic nonnegativity ⇒ matrix PSD` and
   yields 4X-XG⁻¹P-PG⁻¹X⪰0.

In the vocabulary fixed at the chapter opening, the chain is pointwise
positive-real, sampled-kernel PSD, block-matrix PSD, scalar quadratic
inequality, final X PSD. The conclusion is the input required by CFT-30-006.

#### Boundary case

Checking one u gives one scalar inequality, not matrix PSD. The quantifier
“for every u” is essential at the last transition.

#### Pedagogical prerequisites

Use positive kernels from Chapter 18, PSD block matrices from Chapter 24, and
the P, R, X definitions from Chapter 30.

#### Lean correspondence

Public code: [[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|Chapter31.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean|CompletionKernelModel.lean]].
Analytic bridge: [[formalization/lean/CrouzeixConjecture/MatrixHerglotz.lean|MatrixHerglotz.lean]].
`completionSampleCoefficient_quadratic_nonneg_of_positiveKernel` proves the
universal scalar inequality. `completion_X_inequality_of_positiveKernel`
applies the Hermitian quadratic-form criterion.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.kernel_positivity_implies_X; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean; provider-declaration=CrouzeixConjecture.completion_X_inequality_of_positiveKernel; provider-file=formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean; type-sha256=8ea6a00ebbb6a7326410c88a8d764396afc51794cde409a6c5f3b44855ce7e04; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** This is the finite positivity extraction at
`the_numerical_range_is_a_2_spectral_set_v4.tex#L387-L463`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, separated the
five positivity notions, and checked each implication against the provider.
**PEER-REVIEW / PUBLICATION STATE.** This local inspection is not peer review
or publication evidence, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The complete implication to X PSD
compiles, with no import from the Lorist-Schwenninger route.

#### ML analogy

**Mathematical object / ML counterpart.** G is feature geometry and sampled
kernel positivity is a finite certificate on selected features. **Exact
transfer.** A universal exact quadratic test certifies PSD of the Hermitian
coefficient. **Non-transfer.** noisy empirical kernels and floating-point PSD
checks neither establish analytic positivity nor exact universal
nonnegativity. **Diagnostic.** Make two smallest-eigenvalue comparisons: one
for the full sampled block matrix and one for `4X-XG⁻¹P-PG⁻¹X`. The first
tests finite kernel sampling; the second tests the compressed coefficient.

## Worked examples

### The specified member of the running family

Continue Chapter 30's specified diagonalizable member
$A_{1/2,1}=S\operatorname{diag}(1/2,-1/2)S^{-1}$, with

$$
G=\begin{pmatrix}1&-1\\-1&2\end{pmatrix}.
$$

The eigenvalue samples are $z_1=1/4, z_2=-1/4$. Substitution into the entry
definitions gives

$$
P=\begin{pmatrix}16/15&-16/17\\-16/17&32/15\end{pmatrix},\qquad
R=\begin{pmatrix}8/7&-8/9\\-8/9&16/7\end{pmatrix},
$$

and therefore

$$
X=R-P=\begin{pmatrix}8/105&8/153\\8/153&16/105\end{pmatrix}.
$$

The exact calculation ledger is

    z_1=1/4, z_2=-1/4
    P=\begin{pmatrix}16/15&-16/17\\-16/17&32/15\end{pmatrix}
    R=\begin{pmatrix}8/7&-8/9\\-8/9&16/7\end{pmatrix}
    X=\begin{pmatrix}8/105&8/153\\8/153&16/105\end{pmatrix}

Since $G^{-1}=\left(\begin{smallmatrix}2&1\\1&1\end{smallmatrix}\right)$,
the compensating vector for $u=(u_1,u_2)^{\mathsf T}$ is

$$
v=-G^{-1}Pu=-\frac1{255}
\begin{pmatrix}304&64\\32&304\end{pmatrix}u.
$$

In one line,

    v=-\frac1{255}\begin{pmatrix}304&64\\32&304\end{pmatrix}u.

These are exact fractions, not numerical fits. For the concrete test vector
$u=e_1$,

$$
Gv=\begin{pmatrix}-16/15\\16/17\end{pmatrix},\qquad
Pu=\begin{pmatrix}16/15\\-16/17\end{pmatrix},\qquad Gv+Pu=0.
$$

Hence, for every diagonal sampled correction $\Delta$,
$\Theta=u^{\mathrm H}\Delta(Gv+Pu)=0$, and its conjugate also vanishes. The
known sampled blocks are all computable:

$$
2G=\begin{pmatrix}2&-2\\-2&4\end{pmatrix},\quad
G+R=\begin{pmatrix}15/7&-17/9\\-17/9&30/7\end{pmatrix},
$$

$$
4R-2P=\begin{pmatrix}256/105&-256/153\\-256/153&512/105\end{pmatrix}.
$$

This calculation illustrates the cancellation once a positive completion is
available. It does not construct $H$ or prove sampled-kernel positivity for
this matrix. The nilpotent member $A_{0,2}$ used for sharpness in Chapter 32
is not this specified diagonalizable member and does not enter the
shared-basis sampling calculation directly.

### Scalar two-sample calculation.

Take n=1, choose λ with |λ|≤1, and set

    p=g/(1-|λ|²/4),
    r=g/(1-|λ|²/2),
    z_0=0,
    z_1=conj(λ)/2.

Write G=g>0, take x_0=v=-g⁻¹pu and x_1=u, and put
δ=d(conj(λ)/2). The known resolvent contribution is

    2g|v|²+conj(v)(g+r)u+conj(u)(g+r)v+(4r-2p)|u|².

The correction contribution is

    conj(v)gconj(δ)u+conj(u)δgv+|u|²(δp+pconj(δ)).

Thus the full scalar sum is

    2g|v|²+conj(v)(g+r)u+conj(u)(g+r)v+(4r-2p)|u|²
      +conj(v)gconj(δ)u+conj(u)δgv+|u|²(δp+pconj(δ)).

The unknown half is

    Θ=conj(u)δ(gv+pu).

But gv+pu=g(-g⁻¹pu)+pu=0. Hence Θ=0, its adjoint
conj(Θ)=0, and the entire correction contribution vanishes. This one-coordinate
example contains the full cancellation mechanism.

For general n, two eigenvalue samples i and j plus the origin produce a 3×3
matrix of scalar pairings, not a 3×3 matrix of n×n blocks:

    [ vᴴ(2G)v                    vᴴ(G+R)e_i u_i                    vᴴ(G+R)e_j u_j                 ]
    [ conj(u_i)e_iᴴ(G+R)v       conj(u_i)(4R-2P)_ii u_i          conj(u_i)(4R-2P)_ij u_j       ]
    [ conj(u_j)e_jᴴ(G+R)v       conj(u_j)(4R-2P)_ji u_i          conj(u_j)(4R-2P)_jj u_j       ].

In general the lower-right entry indexed by sample labels a,b is

    conj(u_a)(4R-2P)_ab u_b.

Summing this scalar matrix gives exactly the corresponding terms in the
Option n quadratic form.

## ML bridge

The closest ML picture is nuisance-feature elimination in an augmented Gram
matrix. G is the feature geometry, D is an unknown featurewise correction,
and v is an exact control variate. The analogy is useful because it explains
why conditioning of G matters and why a smallest-eigenvalue diagnostic is
natural. Here sampled-kernel PSD is a finite certificate for the chosen
points and vectors, not a certificate on the whole disk. An
eigenvalue-sampling diagnostic reports the smallest eigenvalue as points are
added, together with the residuals of the exact block identities. The analogy
stops at the exactness boundary: empirical kernel estimates, finite precision,
and a positive sampled matrix do not prove analytic positive-real behavior on
the whole disk.

## Lean translation

The Lean file follows the proof boundary rather than compressing the chapter
into one terminal theorem. It defines the public kernel-model alias, proves
the reverse normalization theorem locally, reexports the four provider
checkpoints, and gives six distinct exercise theorems. In particular,
Exercise E02 returns the matrix equality D(0)=0 before its coordinate
consequence d(0)=0, while E05 explicitly uses
`completionUnknownHalfContribution_eq_zero`. The checked source is
[[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|Chapter31.lean]].

## Exercises

Each exercise has a distinct checked Lean theorem. The prompt states the full
signature so the result can be copied into a scratch file.

### CFT-31-E01 -- kernel entry and sparse column {#exercise-cft-31-e01}

Prove both the sparse-column identity for `completionKernelModel` and its
entry formula. Then expand the two diagonal products in the selected entry.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|`CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_01_solution`]].

```lean
theorem exercise_01_solution
    {n : Type*} [Fintype n] [DecidableEq n]
    (G : SquareMatrix n) (lambda : n → ℂ) (d : ℂ → n → ℂ)
    (z : ℂ) (i j : n) :
    (completionKernelModel G lambda d z *ᵥ
        completionSparseVector (fun _ ↦ (1 : ℂ)) j) i =
      completionKernelModel G lambda d z i j ∧
    completionKernelModel G lambda d z i j =
      G i j * (1 - z * lambda j)⁻¹ + d z i * G i j := by
  sorry
```

### CFT-31-E02 -- reverse normalization {#exercise-cft-31-e02}

Starting only from invertibility and `completionKernelModel ... 0 = G`, prove
both the matrix equality `completionDiagonalCorrection d 0 = 0` and its
coordinate consequence `d 0 = 0`. Do not invoke the forward theorem
`completionKernelModel_zero`.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|`CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_02_solution`]].

```lean
theorem exercise_02_solution
    {n : Type*} [Fintype n] [DecidableEq n]
    (G : SquareMatrix n) (hGunit : IsUnit G) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (hnormalized : completionKernelModel G lambda d 0 = G) :
    completionDiagonalCorrection d 0 = 0 ∧ d 0 = 0 := by
  sorry
```

### CFT-31-E03 -- four sampled blocks {#exercise-cft-31-e03}

Prove the sample/sample, sample/origin, origin/sample, and origin/origin
entries as one fourfold conjunction.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|`CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_03_solution`]].

```lean
theorem exercise_03_solution
    {n : Type*} [Fintype n] [DecidableEq n]
    {G : SquareMatrix n} (hG : G.IsHermitian) (lambda : n → ℂ)
    (hlambda : ∀ i, ‖lambda i‖ ≤ 1) :
    (∀ i j, matrixHerglotzKernel (completionResolventModel G lambda)
      (conj (lambda i) / 2) (conj (lambda j) / 2) i j =
        (4 • completionR G lambda - 2 • completionP G lambda) i j) ∧
    (∀ i j, matrixHerglotzKernel (completionResolventModel G lambda)
      (conj (lambda i) / 2) 0 i j = (G + completionR G lambda) i j) ∧
    (∀ i j, matrixHerglotzKernel (completionResolventModel G lambda)
      0 (conj (lambda j) / 2) i j = (G + completionR G lambda) i j) ∧
    ∀ i j, matrixHerglotzKernel (completionResolventModel G lambda) 0 0 i j =
      (2 • G) i j := by
  sorry
```

### CFT-31-E04 -- correction and conjugate half {#exercise-cft-31-e04}

Show that the full correction sample is Θ plus its conjugate.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|`CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_04_solution`]].

```lean
theorem exercise_04_solution
    {n : Type*} [Fintype n] [DecidableEq n]
    {G : SquareMatrix n} (hG : G.IsHermitian) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (hd0 : d 0 = 0) (u : n → ℂ) :
    let theta := completionUnknownHalfContribution G lambda d u
    let correctionSum := ∑ a, ∑ b,
      star (completionSampleVector G lambda u a) ⬝ᵥ
        (matrixHerglotzKernel (completionCorrectionModel G d)
          (completionSamplePoint lambda a) (completionSamplePoint lambda b) *ᵥ
            completionSampleVector G lambda u b)
    correctionSum = theta + conj theta := by
  sorry
```

### CFT-31-E05 -- cancellation vector {#exercise-cft-31-e05}

Prove the vector residual, the unknown half-contribution, and its conjugate
sum all vanish. With E04's Hermiticity hypothesis and d(0)=0, this becomes the
complete correction contribution.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|`CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_05_solution`]].

```lean
theorem exercise_05_solution
    {n : Type*} [Fintype n] [DecidableEq n]
    (G : SquareMatrix n) (hGunit : IsUnit G) (lambda : n → ℂ)
    (d : ℂ → n → ℂ) (u : n → ℂ) :
    G *ᵥ completionV G (completionP G lambda) u + completionP G lambda *ᵥ u = 0 ∧
      completionUnknownHalfContribution G lambda d u = 0 ∧
      completionUnknownHalfContribution G lambda d u +
        conj (completionUnknownHalfContribution G lambda d u) = 0 := by
  sorry
```

### CFT-31-E06 -- quadratic criterion for PSD {#exercise-cft-31-e06}

Prove that a Hermitian matrix whose quadratic form is nonnegative on every
vector is positive semidefinite. Do not invoke the final Jin endpoint.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|`CrouzeixTextbook.Part06.Exercises.Chapter31.exercise_06_solution`]].

```lean
theorem exercise_06_solution
    {n : Type*} [Fintype n] [DecidableEq n]
    (Y : SquareMatrix n) (hY : Y.IsHermitian)
    (hquadratic : ∀ u : n → ℂ, 0 ≤ star u ⬝ᵥ (Y *ᵥ u)) : Y.PosSemidef := by
  sorry
```

### Written solutions

#### CFT-31-E01 solution

Use `mulVec_completionSparseVector` with the all-ones coefficient vector. It
proves the first conjunct `(K(z)e_j)_i=K(z)_ij`. For the second conjunct,
expand K(z) as the sum of the resolvent and correction terms. The right
diagonal matrix contributes `G_ij(1-zλ_j)⁻¹`; the left diagonal matrix
contributes `d_i(z)G_ij`.

#### CFT-31-E02 solution

At z=0, simplify the model equality to `G+D(0)G=G`, hence `D(0)G=0`.
Multiplying on the right by G⁻¹ gives the first conclusion, `D(0)=0`. Matrix
extensionality at entry `(i,i)` turns this into `d_i(0)=0`; function
extensionality gives the second conclusion, `d(0)=0`. The compiled proof
returns both facts and does not use
`completionKernelModel_zero`.

#### CFT-31-E03 solution

Prove the four entries independently. The sample/sample denominator produces
`4R-2P`. Setting only the second sample to zero gives `G+R`; setting only the
first to zero gives its reverse `G+R`; setting both to zero gives `2G`.
Sparse vector selection then turns entrywise identities into the four
quadratic blocks.

#### CFT-31-E04 solution

The sample diagonal Δ scales the ith coordinate. Collecting the terms with Δ
on the left gives `Θ=uᴴΔ(Gv+Pu)`. Conjugating this scalar reverses the pairing
and supplies the terms with Δᴴ on the right; this step uses both G=Gᴴ and
P=Pᴴ. Since d(0)=0, there is no origin/origin correction. Their sum is the
full correction sample.

#### CFT-31-E05 solution

The lemma `completion_mulVec_add_eq_zero` proves `Gv+Pu=0` from
v=-G⁻¹Pu. Substitution into the finite sum proves
`completionUnknownHalfContribution_eq_zero`. Rewriting Θ by zero also
rewrites conj(Θ) by zero. This theorem proves that scalar sum is zero. E04,
with Hermiticity and d(0)=0, identifies it with the complete correction
sampling contribution.

#### CFT-31-E06 solution

Use the Hermitian quadratic-form characterization of PSD:
`Matrix.PosSemidef.of_dotProduct_mulVec_nonneg hY hquadratic`. This exercise
ends at the general criterion. It never invokes
`completion_X_inequality_of_positiveKernel` or the constant-two endpoint.

## Synthesis and forward dependencies

The chapter has converted positive completion data into the exact inequality

    4X-XG⁻¹P-PG⁻¹X⪰0.

The conversion uses no bound on the unknown correction: normalization removes
its origin value, the compensating vector removes every sampled value, and
kernel positivity supplies the remaining quadratic inequality. Chapter 32
will combine this matrix inequality with Chapter 30's Gramian congruences to
reach the norm-two endpoint. It may use the result of this chapter, but it
must not bypass the cancellation calculation established here.
