---
id: cft-chapter-32-jin-constant-two-endpoint
title: Jin’s constant-two endpoint
type: textbook-chapter
status: active
created: 2026-08-23
updated: 2026-08-27
tags: [crouzeix-textbook, constant-two-routes, mathematics, lean]
confidence: high
canonical: 32_jin_constant_two_endpoint.md
chapter: 32
part: 6
lean_exercise_solution_declarations: 6
lean_exact_correspondences: 6
---

# Chapter 32: Jin’s constant-two endpoint

Book: [[knowledge/crouzeix_textbook/crouzeix_textbook_index|Crouzeix foundations textbook]]
Part: [[knowledge/crouzeix_textbook/crouzeix_textbook_index#part-vi-constant-two-routes|Part VI: Constant-two routes]]
Previous: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/31_jin_correction_cancellation|Chapter 31: Jin’s correction cancellation]]
Next: [[knowledge/crouzeix_textbook/part_06_constant_two_routes/33_lorist_schwenninger_perturbation_lemma|Chapter 33: The Lorist-Schwenninger perturbation lemma]]

## Opening problem

Chapters 30 and 31 have done the hard analytic work. We now have one positive
semidefinite matrix inequality, but the desired conclusion is a norm bound.
The number two is hidden in the weights of two Gramians. Extracting it takes
an eigenvector contradiction, not a casual instruction to "drop a positive
tail." After that finite-dimensional step, two ordered limits are still
needed. We first remove the simple-spectrum hypothesis while an outer domain
is fixed, and only then shrink that domain to the numerical range.

## Conceptual model

Think of the proof as an extraction pipeline. The completion theorem produces
an ordered matrix certificate. An extremal eigenvector turns that certificate
into the scalar obstruction `p≤2`; the positive Gramian tail then converts the
order bound into an operator-norm bound. Normalization transports that finite
matrix estimate to polynomial and pole-free rational functions. Finally, two
separate continuity arguments remove the auxiliary outer domain and recover
the original polynomial statement. Keeping these stages separate makes clear
where positivity, spectral geometry, and functional-calculus continuity enter.

## Data carried into the endpoint

Let S∈GL_n(ℂ), write G=SᴴS, and let Λ=diag(λ), with |λ_i|≤1. Positivity of G
gives its positive square root and inverse

    K=G^{1/2},
    K⁻¹=G^{-1/2},
    K⁻¹K=KK⁻¹=I.

The balanced matrix is C=KΛK⁻¹. Its weighted Gramians are

    G₂=Gramian_2(C)=Σ_{k≥0}2⁻ᵏ(Cᵏ)*Cᵏ,
    G₄=Gramian_4(C)=Σ_{k≥0}4⁻ᵏ(Cᵏ)*Cᵏ,
    X=G₂-G₄.

Here `*` in `C*C` denotes conjugate transpose in the compiler-facing notation;
typographically it is CᴴC. The Chapter 30 congruences transport the source
matrix from Chapter 31 to

    4X-XG₄-G₄X⪰0.

Every series converges because Cᵏ=KΛᵏK⁻¹ and
`‖Cᵏ‖≤‖K‖‖K⁻¹‖`. This is also where the invertibility and square-root data
enter. They are not decorative assumptions.

## Source and review boundary

The finite norm extraction follows the pinned Jin source at
git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L530.
The radial Cayley completion and auxiliary sharp bound follow the same file at
git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L652-L774.
The approximation and consequence passage follows
git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910.
The prose below reconstructs the argument. The Lean links state which parts
are checked as public theorems, which are provider proofs, and which prose
sentences explain the interfaces between those theorems.

### Cumulative historical record

**SOURCE CLAIM.** The pinned manuscript gives the finite norm extraction at
`git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L530`,
the radial Cayley completion and auxiliary sharp bound at the same file's
`#L652-L774`, and the approximation and rational-consequence passage at
`#L854-L910`.
**INSPECTED EVIDENCE.** The native receipt
[[evidence/crouzeix_conjecture/source_manifest.tsv|JIN-565-V4-TEX]]
([source_manifest.tsv#L11](../../../evidence/crouzeix_conjecture/source_manifest.tsv#L11))
records the remote path, byte count, and digest. Harp inspected this receipt,
all three commit-pinned line ranges, and the local theorem dependency receipts.
**PEER-REVIEW / PUBLICATION STATE.** Harp has no peer-review, acceptance, or
journal-publication receipt for this revision, and makes no priority claim.
Neither the metadata-only Preprints.org observation nor an Annals-formatted
artifact establishes submission, acceptance, or publication.
**HARP REPRODUCTION / FORMALIZATION STATE.** Harp's Lean 4.32 port compiles the
finite endpoint and both ordered limit passages. The older upstream clean-build
receipt was blocked by its recorded disk precondition and is not being
silently promoted to a successful build.

## Formal development

### CFT-32-001 - completion implies norm two {#cft-32-001}

#### Purpose

**Motivation.** Turn the positive completion certificate into the numerical
constant two, with the eigenvector, square-root, tail, and unitary transfers
all visible.

#### Statement

The transported Chapter 31 inequality is

    4(G₂-G₄)-(G₂-G₄)G₄-G₄(G₂-G₄)⪰0.

It implies G₄≤2I. The positive Gramian tail then gives
`4I-C*C⪰0`, hence ‖C‖≤2. Finally polar decomposition gives

    ‖SΛS⁻¹‖=‖C‖≤2.

#### Hypothesis ledger

The compiler-facing hypotheses are S invertible, G=S*S, and
C=G¹ᐟ²ΛG⁻¹ᐟ². In the chapter notation G=SᴴS and C=KΛK⁻¹. We also assume
|λ_i|≤1 and a positive completion kernel. The positive kernel supplies the
source PSD inequality through CFT-31-006. The identities K⁻¹K=KK⁻¹=I and
the self-adjointness of K and K⁻¹ come from the positive square root of G.

    S invertible; G=S*S; C=G¹ᐟ²ΛG⁻¹ᐟ²; |λ_i|≤1; positive completion kernel.

#### Proof roadmap

First bound every eigenvalue of G₄ by two. Then combine the resulting upper
bound with the first positive term of G₄-I. Finally use polar decomposition
to return from the balanced matrix C to SΛS⁻¹.

#### Proof

Put X=G₂-G₄. Both G₄ and X are PSD. The difference series also gives

    Y=(1/4)C*C,
    X-Y⪰0 and Y⪰0.

Because G₄ is Hermitian, choose e with G₄e=pe. We now assume p>2 for
contradiction. Test the transported inequality on e. The anticommutator collapses:

    0≤e*[4X-XG₄-G₄X]e=(4-2p)e*Xe.

Since X⪰0, the scalar e*Xe is nonnegative. Since 4-2p<0, both facts force

    e*(G₂-G₄)e=0,
    e*Xe=0.

For a PSD matrix, a zero quadratic form puts the vector in its kernel, so

    (G₂-G₄)e=0,
    Xe=0.

Recall the first positive tail explicitly: `Y=(1/4)C*C`. Now decompose
X=(X-Y)+Y. Both summands are PSD, while e*Xe=0. Therefore

    G₂-G₄-Y⪰₀ and Y⪰₀.

    e*Ye=0.

Because Y=(1/4)C*C, this scalar is `(1/4)‖Ce‖²`. Hence Ce=0. Every
positive-degree term in G₄e now vanishes, while the k=0 term is the identity:

    G₄e=e.

But the chosen eigenvector also satisfies G₄e=pe. The eigenvector is nonzero,
so G₄e=pe gives p=1, contradiction. We conclude

    G₄≤2I.

The second half is an ordered tail argument. The series identity is

    G₄=I+Σ_{k≥1}4⁻ᵏ(Cᵏ)*Cᵏ≥I.

More precisely,

    G₄-I-(1/4)C*C⪰0.

Adding this lower estimate to `2I-G₄⪰0` gives

    I-(1/4)C*C⪰0,
    C*C≤4(G₄-I)≤4I.

Testing on an arbitrary vector x yields
`‖Cx‖²≤4‖x‖²`; taking the operator-norm supremum gives ‖C‖≤2.

To return to the original similarity, polar decomposition writes S=UK with
U unitary. Thus `SΛS⁻¹=UCU⁻¹`, and unitary invariance gives

    ‖SΛS⁻¹‖=‖UCU⁻¹‖=‖C‖≤2.

In particular, ‖SΛS⁻¹‖=‖C‖≤2.

#### Boundary case

If p=2, the scalar coefficient 4-2p vanishes and no contradiction follows.
That is fine: the argument only excludes eigenvalues strictly above two.
The Loewner bound G₄≤2I includes equality.

#### Pedagogical prerequisites

Use PSD quadratic forms, the spectral theorem for Hermitian matrices,
positive square roots, Gramian series, and polar decomposition from Chapters
8, 9, 24, 30, and 31.

#### Lean correspondence

Public theorem: [[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|Chapter32.lean]].
Main provider: [[formalization/lean/CrouzeixConjecture/PositiveRealCompletion.lean|PositiveRealCompletion.lean]].
The eigenvector and tail steps are in
[[formalization/lean/CrouzeixConjecture/CompletionEigenvector.lean|CompletionEigenvector.lean]];
the norm extraction from `4I-CᴴC⪰0` is in
[[formalization/lean/CrouzeixConjecture/Positivity.lean|Positivity.lean]].

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.completion_implies_norm_two; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean; provider-declaration=CrouzeixConjecture.norm_completionDiagonalizableMatrix_le_two_of_positiveKernelModel; provider-file=formalization/lean/CrouzeixConjecture/PositiveRealCompletion.lean; type-sha256=1c24687b99a83a92aa728551ff33b388ec0e9f1551a6a36145d826b25311ccd1; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The pinned manuscript uses the eigenvalue contradiction as
the last finite step at `the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L530`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, PSD zero-kernel
implication, positive first term, and polar transfer separately.
**PEER-REVIEW / PUBLICATION STATE.** This local check is not peer review or
publication evidence, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The theorem and named supporting
lemmas compile without an LS-route import.

#### ML analogy

**Mathematical object / ML counterpart.** G is feature geometry, while G₄ is
an infinite-horizon observability Gramian for a stable linear feature update.
**Exact transfer.** The PSD inequalities form a finite certificate once the
analytic provider has been proved. **Non-transfer.** noisy empirical kernels
and floating-point PSD checks do not justify the zero-kernel step or the
infinite series identity. **Diagnostic.** Report the largest eigenvalue of G₄,
the smallest eigenvalue of `X-Y`, and the smallest eigenvalue of 4I-C*C as
three endpoint eigenvalue margins. The last quantity directly tests the norm
certificate.

### CFT-32-002 - Jin polynomial constant two {#cft-32-002}

#### Purpose

**Motivation.** Apply the completion endpoint to a polynomial after making
its scalar maximum one, then restore its original scale.

#### Statement

For A∈M_n(ℂ), p∈ℂ[z], set

    M=max_{z∈W(A)}|p(z)|.

When M>0 define q=p/M. Then max_{W(A)}|q|≤1. Proving the unit bound for
q(A) still requires the ordered outer-domain argument below; the completion
cannot be applied directly to perturbation eigenvalues using only a bound on
W(A). On a fixed outer domain the statement uses

    q=p/M when M>0,
    max_{W(A)}|q|≤1,
    |q_m(z)|≤1 for every z∈closure(Ω_m),
    μ_{j,i}=q_m(λ_{j,i}),
    q_m(A_j)=S_j diag(μ_j) S_j⁻¹.

After the completion construction and the ordered limits,

    ‖p(A)‖≤2M.

Equivalently, the statement follows the exact order

    q=p/M when M>0; max_{W(A)}|q|≤1; ‖p(A)‖≤2M.

#### Hypothesis ledger

The data are A∈M_n(ℂ), p∈ℂ[z], and
M=max_{z∈W(A)}|p(z)|. Compactness and nonemptiness of W(A) make M an attained
nonnegative maximum. The earlier construction supplies the completion for
the normalized polynomial on each fixed outer domain.

    A∈M_n(ℂ); p∈ℂ[z]; M=max_{z∈W(A)}|p(z)|.

#### Proof roadmap

Split at M=0. In the positive branch, normalize separately on each fixed outer
domain, map every hypothesis to the completion theorem, pass the two limits in
order, and only then identify the limiting maximum with M.

#### Proof

If M=0, every z∈W(A) satisfies |p(z)|=0. This zero branch does not need the
completed Crouzeix theorem. If p is the zero polynomial, the conclusion is
immediate. Otherwise its root set is finite, so W(A), being contained in that
root set, is finite. The numerical range is convex and therefore connected;
a finite connected subset of ℂ is a singleton, say W(A)={z₀}. Polarization
of the identity

    ⟨x,Ax⟩=z₀‖x‖²

forces A=z₀I. Since p(z₀)=0, the polynomial functional calculus gives

    p(A)=p(z₀)I=0.

Now assume M>0. For the final matrix, q=p/M satisfies

    |q(z)|=M⁻¹|p(z)|≤1 on W(A),
    max_{W(A)}|q|≤1.

This scalar bound alone does not control q at eigenvalues of a perturbed
matrix, because those eigenvalues lie in W(A_j), not necessarily W(A). Fix a
sufficiently small outer domain Ω_m and instead set

    M_m=max_{z∈closure(Ω_m)}|p(z)|,
    q_m=p/M_m.

Since W(A)⊆closure(Ω_m), we have M≤M_m; hence M_m>0. The definition of
M_m first gives the uniform scalar estimate

    |q_m(z)|≤1 for every z∈closure(Ω_m).

With m fixed, choose simple-spectrum A_j→A and, eventually, W(A_j)⊆Ω_m.
Diagonalize

    A_j=S_jΛ_jS_j⁻¹.

Every diagonal entry λ_{j,i} of Λ_j lies in
σ(A_j)⊆W(A_j)⊆Ω_m. Define

    μ_{j,i}=q_m(λ_{j,i}).

Polynomial functional calculus in the same diagonalizing basis gives the
matrix identity

    q_m(A_j)=S_j diag(μ_j) S_j⁻¹.

Thus |μ_{j,i}|≤1, which is the `hlambda` input of CFT-32-001. This
eigenvalue bound is necessary but does not itself supply the completion. The
completion comes from a separate analytic construction on the fixed outer
domain.

In short: the eigenvalue bound is necessary but does not itself supply the completion.

Choose the supported boundary parametrization Γ_m and its boundary measure
for Ω_m. Its polynomial Cauchy formula is an identity for every polynomial,
not merely an estimate at the eigenvalues. The unit bound on closure(Ω_m)
makes the boundary function of q_m contractive. The outer-boundary Cauchy
identities then identify every power in the Cayley expansion of q_m(A_j).
Adding the adjoint companion produces the direct Cayley identity

    2H_j(z)=(I+zq_m(A_j))(I-zq_m(A_j))⁻¹+g_j(z)*.

The double-layer density is positive, so its Cayley series H_j is analytic,
has H_j(0)=I, and has positive semidefinite real part. These facts construct
the positive-real completion of the pair (A_j,q_m(A_j)); they are precisely
the hypotheses packaged by
`hasDoubleLayerCompletionProvider_of_parametricBoundary`.

Now pull this completion through S_j. The generated-algebra companion becomes
a diagonal correction d_j, and the normalization at the origin gives

    d_j(0)=0.

Analyticity and positive real part give a positive Herglotz kernel for the
pulled-back function. Its equality with
`completionKernelModel(S_j*S_j,μ_j,d_j)` transports that positivity to the
exact kernel required by CFT-32-001. Only at this point have all five inputs
been supplied: S_j is invertible, |μ_{j,i}|≤1, the target has the displayed
diagonalization, d_j(0)=0, and the model has a positive Herglotz kernel.
CFT-32-001 therefore gives

    ‖q_m(A_j)‖≤2,
    ‖p(A_j)‖=M_m‖q_m(A_j)‖≤2M_m.

First let j→∞ with m fixed. Polynomial functional-calculus continuity gives
`p(A_j)→p(A)`, hence `‖p(A)‖≤2M_m`. Only then let m→∞. Uniform continuity on
one compact buffer gives M_m→M, so

    ‖p(A)‖≤2M.

Finally, scalar homogeneity for q=p/M yields ‖q(A)‖≤2 and

    p(A)=M q(A),
    ‖p(A)‖=M‖q(A)‖≤2M.

The zero branch and the ordered positive branch together prove the statement.

The branch ledger is therefore

    M=0 ⇒ p(A)=0;
    M>0 ⇒ M_m=max_{closure(Ω_m)}|p| ⇒ q_m=p/M_m
        ⇒ ‖q_m(A_j)‖≤2 ⇒ j→∞ ⇒ m→∞ ⇒ ‖p(A)‖≤2M.

The Lean boundary is exact but distributed. The checked declarations
`parametricBoundaryFirstPartIntegral_cayley_eq`,
`parametric_direct_cayley_identity`, and
`hasDoubleLayerCompletionProvider_of_parametricBoundary` formalize the
outer-boundary Cauchy and positive-real construction. The declarations
`exists_completionKernelModel_of_isPositiveRealCompletion` and
`matrixHerglotzKernel_positive_congr_on` formalize d_j(0)=0 and transport of
kernel positivity. Finally `positiveRealCompletionStatement` applies the
same norm-extraction provider exposed as CFT-32-001. The public
`jin_polynomial_constant_two` declaration is a checked reexport of the
already assembled polynomial endpoint; Chapter32.lean does not contain a
second theorem that re-proves this entire distributed bridge.

#### Boundary case

Division by M occurs only after M>0 has been established. Treating M=0 by
the same formula would hide an invalid inverse even though Lean's inverse is
total.

#### Pedagogical prerequisites

Use maximum modulus on compact sets, scalar normalization, simple-spectrum
approximation, polynomial functional-calculus continuity, and CFT-32-001.

#### Lean correspondence

Public alias: [[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|Chapter32.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean|HolomorphicConsequences.lean]].
The alias is the checked polynomial theorem. Exercise E02 separately exposes
the two normalization branches. The completion construction displayed above
is supported by checked declarations in
[[formalization/lean/CrouzeixConjecture/ParametricDoubleLayerIdentity.lean|ParametricDoubleLayerIdentity.lean]],
[[formalization/lean/CrouzeixConjecture/DoubleLayerBoundary.lean|DoubleLayerBoundary.lean]],
and
[[formalization/lean/CrouzeixConjecture/CompletionDiagonalization.lean|CompletionDiagonalization.lean]];
it is not inferred from the eigenvalue inequality alone.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.jin_polynomial_constant_two; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean; provider-declaration=CrouzeixConjecture.polynomialCrouzeixBound_of_holomorphicCrouzeixBound; provider-file=formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean; type-sha256=0c2cdfce0642e8d4ab88dc860b6a60e85be855897e5f63fd51babbf67c87fc5d; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The finite norm extraction is at
`the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L530`; the radial Cayley
completion and auxiliary sharp bound used for the polynomial consequence are
at `the_numerical_range_is_a_2_spectral_set_v4.tex#L652-L774`; the
simple-spectrum and outer-domain passage is at
`the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, all three exact
ranges, and normalization separately from norm extraction.
**PEER-REVIEW / PUBLICATION STATE.** This is no external review or publication
claim, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The public theorem is a checked
provider reexport; the exercise has its own proposition and proof.

#### ML analogy

**Mathematical object / ML counterpart.** Polynomial evaluation is a linear
filter applied to a nonnormal feature-transition matrix. **Exact transfer.**
Normalizing by a known scalar sup norm reduces a scale-dependent claim to a
unit finite certificate. **Non-transfer.** noisy empirical kernels can
underestimate M, and floating-point PSD checks cannot repair that error.
**Diagnostic.** The polynomial normalization ratio is `‖p(A)‖/(2M)` when
M>0. Sample the boundary densely and compare the sampled maximum with a
certified enclosure before reporting that ratio.

### CFT-32-003 - rational spectral set {#cft-32-003}

#### Purpose

**Motivation.** State the rational consequence with its pole condition and
functional calculus explicit, rather than calling it another polynomial case.

#### Statement

Let r be rational and assume poles(r)∩W(A)=∅. Define the pole-free domain

    U_r=ℂ\poles(r).

Then r is holomorphic on a neighborhood of W(A), its matrix value
`r(A)=num(r)(A)den(r)(A)⁻¹` is defined, and

    ‖r(A)‖≤2 max_{z∈W(A)}|r(z)|.

#### Hypothesis ledger

The two inputs are poles(r)∩W(A)=∅ and r(A) defined by rational functional
calculus. Pole avoidance implies W(A)⊆U_r and makes the denominator matrix
invertible because the spectrum of A lies in W(A).

#### Proof roadmap

Convert disjointness into containment in the open complement of the finite
pole set, prove holomorphy there, identify holomorphic and rational matrix
evaluation, and apply the holomorphic bound.

#### Proof

The reduced denominator has finitely many roots, so U_r is open. The
hypothesis gives

    W(A)⊆ℂ\poles(r).

On this set the denominator is nonzero, hence

    r holomorphic on a neighborhood of W(A).

The spectrum containment `σ(A)⊆W(A)` makes `den(r)(A)` invertible. The
holomorphic functional calculus then computes

    r(A)=num(r)(A)den(r)(A)⁻¹.

Applying the holomorphic bound to r and using this identification gives

    holomorphic bound ⇒ rational spectral-set bound.

This proves the rational statement without identifying rational
normalization with polynomial normalization.

#### Boundary case

A pole outside W(A) is allowed. A pole on W(A) invalidates both the scalar
maximum and the neighborhood needed by the holomorphic calculus.

#### Pedagogical prerequisites

Use rational functions, reduced denominators, spectrum containment, and the
holomorphic functional calculus from Chapters 17 and 21.

#### Lean correspondence

Public alias: [[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|Chapter32.lean]].
Provider: Chapter32.lean, declaration
`CrouzeixTextbook.Part06.jin_rational_spectral_set_provider`; see
[[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|the checked source]].
`CrouzeixConjecture.holomorphicCrouzeixRationalBound` is its proof dependency, not the direct provider:
the dependency and its underlying evaluation bridge live in
[[formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean|HolomorphicConsequences.lean]].

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.jin_rational_spectral_set; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean; provider-declaration=CrouzeixTextbook.Part06.jin_rational_spectral_set_provider; provider-file=formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean; type-sha256=a68683384ff3984804bccbfaf7f5dea8ca6128d26a0618984bc5d293837f0144; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The pinned source gives the rational 2-spectral-set
conclusion in `the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, that exact
rational-consequence range, pole avoidance, denominator invertibility, and the
evaluation identity separately.
**PEER-REVIEW / PUBLICATION STATE.** This supplies no peer-review or
publication receipt, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The rational corollary compiles as
a distinct declaration from the polynomial theorem.

#### ML analogy

**Mathematical object / ML counterpart.** A rational filter models a linear
system with poles, such as a resolvent feature map. **Exact transfer.** A
certified pole-free region and a finite certificate control the matrix filter.
**Non-transfer.** Noisy empirical spectral plots and floating-point PSD do not
prove that every pole avoids W(A). **Diagnostic.** Report the pole-distance
and denominator-inverse residual: the minimum certified distance from W(A) to
the pole set and `‖den(r)(A)·den(r)(A)⁻¹-I‖`.

### CFT-32-004 - rational constant two by normalization {#cft-32-004}

#### Purpose

**Motivation.** Separate rational normalization from the pole-free spectral
set theorem, including the zero maximum branch.

#### Statement

Let M\_r=max_{z∈W(A)}|r(z)| and assume poles(r)∩W(A)=∅. When M\_r>0, set
s=r/M\_r. This is rational normalization, and it must be kept separate from the
polynomial normalization q=p/M above.

In plain compiler-facing notation, `M_r=max_{z∈W(A)}|r(z)|` and `s=r/M_r`.
The key warning is exact: polynomial normalization and rational normalization are different operations.
Then max_{W(A)}|s|≤1 and

    ‖r(A)‖≤2M.

In the notation used by the theorem contract,

    s=r/M when M>0; max_{W(A)}|s|≤1; ‖r(A)‖≤2M.

#### Hypothesis ledger

The exact inputs are M=max_{z∈W(A)}|r(z)| and poles(r)∩W(A)=∅. The pole set
does not change under multiplication by a nonzero scalar. The rational
functional calculus, rather than polynomial evaluation, defines r(A).

    M=max_{z∈W(A)}|r(z)|; poles(r)∩W(A)=∅.

#### Proof roadmap

Split at M=0. In the positive case normalize the rational function, apply the
spectral-set bound, and use homogeneity of rational evaluation.

#### Proof

If M=0, every z∈W(A) satisfies r(z)=0. Write the reduced fraction as
r=num(r)/denom(r). Pole avoidance says denom(r)(z)≠0 throughout W(A), so
num(r)(z)=0 there. Apply the elementary zero-branch argument from
CFT-32-002 to the numerator: `num(r)(A)=0`. The denominator matrix is
invertible by pole avoidance, and therefore

    r(A)=num(r)(A) denom(r)(A)⁻¹=0.

This argument deliberately does not appeal to the rational spectral-set
endpoint it is helping normalize.

If M>0, define

    s=r/M,
    max_{W(A)}|s|≤1.

The pole-free set is unchanged because M is nonzero. CFT-32-003 gives
‖s(A)‖≤2. Rational functional calculus respects scalar multiplication, so

    r(A)=M s(A),
    ‖r(A)‖=M‖s(A)‖≤2M.

Thus the ordered proof is `M=0 ⇒ r(A)=0`; otherwise `M>0 ⇒ s=r/M ⇒
‖s(A)‖≤2 ⇒ ‖r(A)‖≤2M`. Polynomial normalization and rational normalization
are different operations: the latter must preserve pole avoidance and the
denominator inverse.

The exact rational branch order is

    M=0 ⇒ r(A)=0; M>0 ⇒ s=r/M; ‖s(A)‖≤2; r(A)=M s(A); ‖r(A)‖≤2M.

#### Boundary case

When r is identically zero, its reduced denominator is one and M=0. No
division by M is needed.

#### Pedagogical prerequisites

Use CFT-32-003, scalar homogeneity, and the zero/positive case split from
CFT-32-002.

#### Lean correspondence

Public alias: [[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|Chapter32.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean|HolomorphicConsequences.lean]].
Exercise E04 has a distinct normalization proposition.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.jin_rational_constant_two; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean; provider-declaration=CrouzeixConjecture.holomorphicCrouzeixRationalBound; provider-file=formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean; type-sha256=45eabf35921b773b4ec7c3aacb0af28c045bd1f008d6f6ced30c63724d5a91c8; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The pointwise rational form and its conclusion are traced to
`the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, that exact
rational-consequence range, and kept the pole condition visible through
normalization.
**PEER-REVIEW / PUBLICATION STATE.** This is not publication review, and Harp
makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The compiled rational theorem and
exercise are separate from the polynomial declarations.

#### ML analogy

**Mathematical object / ML counterpart.** Rational normalization rescales a
resolvent filter without moving its poles. **Exact transfer.** With certified
pole avoidance, scalar rescaling preserves the finite certificate.
**Non-transfer.** noisy empirical kernels may misestimate both M and distance
to a pole; floating-point PSD checks address neither issue. **Diagnostic.**
Track the normalized sup bound and pole distance. The rational normalization
ratio is `‖r(A)‖/(2M)` when M>0.

### CFT-32-005 - holomorphic constant two and ordered limits {#cft-32-005}

#### Purpose

**Motivation.** Remove simple spectrum and the auxiliary outer domain without
interchanging two logically different limits.

#### Statement

Let U open, W(A)⊆U, and f holomorphic on U. For fixed outer domain Ω_m,
simple-spectrum matrices A_j→A satisfy f(A_j)→f(A). After that matrix limit,
the closed outer domains shrink to W(A), giving

    ‖f(A)‖≤2 max_{z∈W(A)}|f(z)|.

#### Hypothesis ledger

The exact inputs are U open, W(A)⊆U, and f holomorphic on U. Compactness of
W(A) provides a positive buffer inside U. The outer domains must remain
inside this buffer so functional calculus is applied on one common
neighborhood.

#### Proof roadmap

Fix m. Approximate A by simple-spectrum matrices, establish eventual spectrum
and numerical-range containment, pass their matrix evaluations to the limit,
and only then let m tend to infinity in the scalar maxima.

#### Proof

Choose parallel outer domains Ω_m with

    W(A)⊆Ω_m,
    closure(Ω_m)⊆U

for all sufficiently large m. Now fix m. There are simple-spectrum matrices

    A_j→A.

Continuity of the numerical range on a fixed open neighborhood gives

    W(A_j)⊆Ω_m eventually.

Spectrum containment then gives `σ(A_j)⊆Ω_m` eventually. Holomorphy on the
common neighborhood and continuity of the finite-dimensional functional
calculus give

    f(A_j)→f(A).

For those large j, the simple-spectrum completion argument gives

    ‖f(A_j)‖≤2 max_{z∈closure(Ω_m)}|f(z)|.

Norm continuity passes this bound to f(A). This is the first limit:

    first j→∞ with m fixed.

The matrix A and the left side are now fixed. Only after this passage do we
shrink the outer domain:

    then m→∞,
    closure(Ω_m)↓W(A),
    max_{closure(Ω_m)}|f|→max_{W(A)}|f|.

The last convergence uses uniform continuity of |f| on one compact buffer
and the fact that every point of closure(Ω_m) is within the outer radius of
W(A). Taking the scalar limit yields the claimed holomorphic bound.

The provider records the same proof with k as the perturbation index:

    A_k→A with A_k simple-spectrum;
    f(A_k)→f(A) on one fixed outer domain Ω_m;
    ‖f(A)‖≤2 max_{z∈closure(Ω_m)}|f(z)|;
    closure(Ω_m)↓W(A);
    max_{closure(Ω_m)}|f|→max_{W(A)}|f|;
    ‖f(A)‖≤2 max_{W(A)}|f|.

#### Boundary case

If one lets m depend on j without a uniform neighborhood, continuity of the
functional calculus has no fixed domain on which to act. The proof therefore
requires the displayed order.

#### Pedagogical prerequisites

Use simple-spectrum perturbation, continuity of numerical ranges, compact
outer approximation, and continuity of holomorphic functional calculus.

#### Lean correspondence

Public alias: [[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|Chapter32.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/HolomorphicOuterLimit.lean|HolomorphicOuterLimit.lean]].
The provider fully formalizes simple-spectrum evaluation continuity, eventual
containment, outer maxima convergence, and the two norm-limit passages.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.holomorphic_constant_two; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean; provider-declaration=CrouzeixConjecture.holomorphicCrouzeixBound; provider-file=formalization/lean/CrouzeixConjecture/HolomorphicOuterLimit.lean; type-sha256=46a9ed9e7bfeb40a079ae46d8cb6d2bbb0d0d330b9eaf2646c0f15ddae2552b8; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The pinned source separates perturbation from the
outer-domain passage at `the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt, both `Tendsto`
statements, and the eventual containment event.
**PEER-REVIEW / PUBLICATION STATE.** This local trace supplies no publication
judgment, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** These limit theorems are
formalized in the provider; this chapter reconstructs them for readers.

#### ML analogy

**Mathematical object / ML counterpart.** A_j is a regularized model with a
simple spectrum, while Ω_m is a certified robustness region. **Exact
transfer.** One may remove regularization under a fixed certificate region,
then tighten that region. **Non-transfer.** noisy empirical kernels and
floating-point PSD do not provide uniform convergence or eventual
containment. **Diagnostic.** Measure an ordered-limit residual: for fixed m,
track `‖f(A_j)-f(A)‖` and the containment margin first; only after convergence
track the outer-maximum gap.

### CFT-32-006 - polynomial recovery after the holomorphic limit {#cft-32-006}

#### Purpose

**Motivation.** Recover the polynomial theorem by specialization after, not
before, the analytic limit machinery has finished.

#### Statement

For p∈ℂ[z], p holomorphic on ℂ and W(A) compact. Set f=p. Then

    holomorphicMatrixEval(A,p)=polynomialEval(p,A),
    ‖p(A)‖≤2 max_{z∈W(A)}|p(z)|.

Using the reader-facing abbreviation p(A), this is

    holomorphicMatrixEval(A,p)=p(A); ‖p(A)‖≤2 max_{z∈W(A)}|p(z)|.

#### Hypothesis ledger

The inputs are p∈ℂ[z], p holomorphic on ℂ, and W(A) compact. No pole condition
is needed. Compactness supplies the maximum on W(A).

    p∈ℂ[z]; p holomorphic on ℂ; W(A) compact.

#### Proof roadmap

Complete the outer-domain limit first. Then specialize the scalar function to
a polynomial and rewrite both the matrix evaluation and scalar maximum.

#### Proof

The outer-domain limit completed before polynomial specialization in
CFT-32-005. Now set

    f=p.

Polynomial functions are entire. The holomorphic calculus agrees with the
polynomial calculus:

    holomorphicMatrixEval(A,p)=polynomialEval(p,A).

The two scalar maximum definitions also agree:

    maxFunctionModulusOnSet(W(A),p)=maxPolynomialModulusOnNumericalRange(A,p).

Substituting both identities into the holomorphic estimate gives

    ‖p(A)‖≤2 max_{W(A)}|p|.

This order matters. The polynomial identity is a specialization of the
already completed holomorphic theorem, not a replacement for either limit.

#### Boundary case

For a constant polynomial, both functional calculi return the same scalar
multiple of I. The proof still uses the same identification theorem.

#### Pedagogical prerequisites

Use CFT-32-005 and the compatibility of polynomial and holomorphic functional
calculus from Chapter 16.

#### Lean correspondence

Public alias: [[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|Chapter32.lean]].
Provider: [[formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean|HolomorphicConsequences.lean]].
Exercise E06 checks the two rewrite identities directly.

Duplication disclosure. This card and
[[knowledge/crouzeix_textbook/part_06_constant_two_routes/32_jin_constant_two_endpoint#cft-32-002|CFT-32-002]]
rest on the same provider,
`CrouzeixConjecture.polynomialCrouzeixBound_of_holomorphicCrouzeixBound`, and
therefore index one theorem under two identities. No second proof is claimed.
The two are indexed separately because they are consumed at different points of
the route: CFT-32-002 applies the completion endpoint to a normalized
polynomial, while this card recovers the polynomial statement after the
holomorphic limit has already run. Card identities are frozen, so the roster is
not adjusted. The whole-roster figure is on the
[[knowledge/crouzeix_textbook/status_and_scope|status and scope]] page, and
`mise run textbook-counts` with `--audit` lists every such pair.

Receipt audit: {public-declaration=CrouzeixTextbook.Part06.polynomial_from_holomorphic; public-file=formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean; provider-declaration=CrouzeixConjecture.polynomialCrouzeixBound_of_holomorphicCrouzeixBound; provider-file=formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean; type-sha256=0c2cdfce0642e8d4ab88dc860b6a60e85be855897e5f63fd51babbf67c87fc5d; axioms=Classical.choice,Quot.sound,propext; verification-target=CrouzeixTextbook}

#### Historical context

**SOURCE CLAIM.** The polynomial result is traced to the pinned manuscript at
`the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910`.
**INSPECTED EVIDENCE.** Harp checked the JIN-565-V4-TEX receipt and both rewrite
identities rather than treating them as prose conventions.
**PEER-REVIEW / PUBLICATION STATE.** This supplies no peer-review or
publication receipt, and Harp makes no priority claim.
**HARP REPRODUCTION / FORMALIZATION STATE.** The specialization and its
exercise compile with standard axioms.

#### ML analogy

**Mathematical object / ML counterpart.** A polynomial filter is an entire
finite-depth filter inside the larger class of holomorphic filters. **Exact
transfer.** A theorem for the larger function class specializes after the
evaluation interfaces are proved equal, giving a finite certificate.
**Non-transfer.** noisy empirical kernels and floating-point PSD checks do not
establish the preceding uniform limits. **Diagnostic.** Compare polynomial
and holomorphic evaluation numerically as a calculus-interface residual, and
report the outer-limit residual before interpreting their small difference as
evidence.

## Worked examples

Continue the family

    A_{λ,α}=\begin{pmatrix}λ&α\\0&-λ\end{pmatrix}.

Chapter 30 wrote the shared-basis and Gramian data for nonzero λ, and Chapter
31 computed the samples for $A_{1/2,1}$. Those calculations were explicitly
conditional on a positive-real completion. The family now supplies a separate
sharpness test at its degenerate member

$$
A_{0,2}=\begin{pmatrix}0&2\\0&0\end{pmatrix}.
$$

In compact text notation,

    A_{0,2}=\begin{pmatrix}0&2\\0&0\end{pmatrix}.

Direct multiplication gives

$$
A_{0,2}^{\mathrm H}A_{0,2}=\operatorname{diag}(0,4),
\qquad \lVert A_{0,2}\rVert=2.
$$

Thus

    A_{0,2}^{\mathrm H}A_{0,2}=\operatorname{diag}(0,4)
    \lVert A_{0,2}\rVert=2

For a unit vector $x=(x_1,x_2)^{\mathsf T}$,

$$
x^{\mathrm H}A_{0,2}x=2\overline{x_1}x_2,
\qquad |2\overline{x_1}x_2|\le |x_1|^2+|x_2|^2=1.
$$

Every point of the unit disk occurs. Given $re^{i\theta}$ with $0\le r\le1$,
choose $t\in[0,\pi/2]$ with $\sin(2t)=r$ and take
$x=(\cos t,e^{i\theta}\sin t)$. Then
$2\overline{x_1}x_2=re^{i\theta}$. Hence

$$
W(A_{0,2})=\{z\in\mathbb C:|z|\le1\}.
$$

Equivalently,

    W(A_{0,2})=\{z\in\mathbb C:|z|\le1\}.

For $p(z)=z$,

$$
\sup_{z\in W(A_{0,2})}|z|=1,
\qquad \lVert p(A_{0,2})\rVert=2.
$$

The scalar side of the equality is

    \sup_{z\in W(A_{0,2})}|z|=1.

This is equality in the factor-two estimate. The matrix $A_{0,2}$ is nonzero
and nilpotent, so it is not diagonalizable and does not itself satisfy the
shared-basis completion hypothesis from Chapter 30. The final theorem removes
the auxiliary simple-spectrum hypothesis.
Concretely, $A_{\varepsilon,2}\to A_{0,2}$ as nonzero $\varepsilon\to0$, and
the ordered approximation argument transfers the already established bound
by continuity. The nilpotent member certifies sharpness of the final theorem,
not existence of a completion for that member.

Lean proves the normalized version with
$J=\left(\begin{smallmatrix}0&1\\0&0\end{smallmatrix}\right)$: the public
declaration
[[formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean|`CrouzeixTextbook.Part06.jordan_two_attains_two`]]
points to
[[formalization/lean/CrouzeixConjecture/Sharpness.lean|`CrouzeixConjecture.jordanNilpotentTwo_attains_two`]].
Our matrix is $A_{0,2}=2J$, so the displayed norm and numerical-range
calculation is the scalar-rescaled form of that checked sharpness witness.

For a diagonalizable member with a proved completion, one may also form the
balanced matrix C and truncated Gramians

    G₄^{(N)}=Σ_{k=0}^N4⁻ᵏ(Cᵏ)ᴴCᵏ.

The useful numerical checks are the top eigenvalue of G₄^{(N)}, the
`smallest eigenvalue of 4I-C*C`, and the tail estimate from the power bound. They
explain where the exact proof can fail in finite precision. They do not
replace the infinite-series or kernel-positivity arguments.

## ML bridge

For ML researchers, the endpoint separates three concerns that are often
conflated in stability arguments: a certificate is constructed in a convenient
coordinate system, an extremal direction extracts the sharp scalar constant,
and continuity transports the estimate back to the deployed operator. The
running nonnormal family above is a diagnostic for this separation: eigenvalue
control alone misses transient amplification, while the numerical range and
Gramian certificate retain it.

## Lean translation

The public theorem `completion_implies_norm_two` is theorem-kind and points to
the positive-real completion provider. The other five public declarations
are truthful reexports. Six exercise theorems have distinct compiler types.
E01 exposes the Gramian hypotheses and proves the norm bridge without calling
the completion endpoint. E03 proves the pole-free neighborhood. E05 returns
the two ordered `Tendsto` facts. E06 proves the two specialization identities.
The file is [[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|Chapter32.lean]].

## Exercises

Each prompt gives the exact checked signature. Replace `sorry` in a scratch
copy; the linked source contains the proof.

### CFT-32-E01 -- Gramian norm extraction {#exercise-cft-32-e01}

Derive `4I-CᴴC⪰0` from the visible Gramian inequality, then extract ‖C‖≤2.
Do not invoke the completion endpoint.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|`CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_01_solution`]].

```lean
theorem exercise_01_solution
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    {M : ℝ} (hM : 0 ≤ M) (C : SquareMatrix n)
    (hbound : ∀ k : ℕ, ‖C ^ k‖ ≤ M)
    (hineq : (4 • (gramian 2 C - gramian 4 C) -
      (gramian 2 C - gramian 4 C) * gramian 4 C -
      gramian 4 C * (gramian 2 C - gramian 4 C)).PosSemidef) : ‖C‖ ≤ 2 := by
  sorry
```

### CFT-32-E02 -- polynomial normalization {#exercise-cft-32-e02}

Prove the zero branch and the positive rescaling implication as one checked
conjunction.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|`CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_02_solution`]].

```lean
theorem exercise_02_solution
    {n : Type*} [Fintype n] [DecidableEq n]
    (A : SquareMatrix n) (p : Polynomial ℂ) (M : ℝ)
    (hM : M = maxPolynomialModulusOnNumericalRange A p) :
    (M = 0 → polynomialEval p A = 0) ∧
      (0 < M →
        ‖polynomialEval (((M : ℂ)⁻¹) • p) A‖ ≤ 2 →
        ‖polynomialEval p A‖ ≤ 2 * M) := by
  sorry
```

### CFT-32-E03 -- pole-free holomorphy {#exercise-cft-32-e03}

Turn pole avoidance into an open domain, set containment, and differentiability.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|`CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_03_solution`]].

```lean
theorem exercise_03_solution (r : RatFunc ℂ) (s : Set ℂ)
    (hfree : RationalPoleFreeOn r s) :
    IsOpen (rationalPoleSet r)ᶜ ∧ s ⊆ (rationalPoleSet r)ᶜ ∧
      DifferentiableOn ℂ (rationalScalarEval r) (rationalPoleSet r)ᶜ := by
  sorry
```

### CFT-32-E04 -- rational normalization {#exercise-cft-32-e04}

Prove the rational zero branch and positive branch without confusing rational
evaluation with polynomial evaluation.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|`CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_04_solution`]].

```lean
theorem exercise_04_solution
    {n : Type*} [Fintype n] [DecidableEq n]
    (A : SquareMatrix n) (r : RatFunc ℂ)
    (hfree : RationalPoleFreeOn r (numericalRange A)) (M : ℝ)
    (hM : M = maxRationalModulusOnNumericalRange A r) :
    (M = 0 → rationalMatrixEval r A = 0) ∧
      (0 < M →
        ‖rationalMatrixEval (((M : ℂ)⁻¹) • r) A‖ ≤ 2 →
        ‖rationalMatrixEval r A‖ ≤ 2 * M) := by
  sorry
```

### CFT-32-E05 -- the two limits {#exercise-cft-32-e05}

Return both the simple-spectrum evaluation limit and a shifted outer-domain
maximum limit. The order in the conjunction matches the proof.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|`CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_05_solution`]].

```lean
theorem exercise_05_solution
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) {U : Set ℂ} (hUopen : IsOpen U)
    (hWU : numericalRange A ⊆ U) {f : ℂ → ℂ}
    (hf : DifferentiableOn ℂ f U) :
    Tendsto (simpleSpectrumHolomorphicEval A f) atTop
      (nhds (holomorphicMatrixEval A f)) ∧
    ∃ N : ℕ, Tendsto
      (fun k ↦ maxFunctionModulusOnSet
        (closure (parallelOuterDomain (numericalRange A) (k + N))) f)
      atTop (nhds (maxFunctionModulusOnSet (numericalRange A) f)) := by
  sorry
```

### CFT-32-E06 -- polynomial specialization {#exercise-cft-32-e06}

Prove both the evaluation identity and the maximum-modulus identity.

Lean declaration:
[[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|`CrouzeixTextbook.Part06.Exercises.Chapter32.exercise_06_solution`]].

```lean
theorem exercise_06_solution
    {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) (p : Polynomial ℂ) :
    holomorphicMatrixEval A (fun z ↦ Polynomial.eval z p) = polynomialEval p A ∧
      maxFunctionModulusOnSet (numericalRange A) (fun z ↦ Polynomial.eval z p) =
        maxPolynomialModulusOnNumericalRange A p := by
  sorry
```

### Written solutions

#### CFT-32-E01 solution

First convert the natural-number scalar `4 • X` in the prompt to the complex
scalar `(4:ℂ) • X`. Apply
`four_sub_conjTranspose_mul_self_posSemidef_of_gramian_inequality` to the
power bound and anticommutator inequality. Its proof performs the eigenvector
contradiction and combines the upper bound on G₄ with the positive first
Gramian term, producing `4I-CᴴC⪰0`. Then apply
`matrix_norm_le_two_of_four_sub_conjTranspose_mul_self_posSemidef` and test
the PSD matrix on arbitrary vectors. This route exposes the hypotheses and
does not call `completion_implies_norm_two` or its provider. Notice why the
power bound is present: it licenses convergence of both infinite Gramians,
so the anticommutator theorem is not manipulating merely formal series. The
final norm lemma uses the operator-norm characterization, not an entrywise
estimate; this is exactly the conversion needed by the matrix theorem.

#### CFT-32-E02 solution

Split on whether the finite index type is empty. In the empty case all square
matrices are equal. In the nonempty zero branch, `M=0` makes p vanish on
W(A). If p≠0, its finite root set contains the convex, connected numerical
range, so W(A) is a singleton. Polarization then forces A to be the
corresponding scalar matrix, and p(A)=0 follows from scalar polynomial
evaluation. Thus this branch has no dependency on the terminal Crouzeix
theorem. For M>0, polynomial evaluation commutes
with scalar multiplication:
`polynomialEval(M⁻¹•p,A)=M⁻¹•polynomialEval(p,A)`. Taking norms turns the
assumed normalized inequality into `M⁻¹‖p(A)‖≤2`. Multiply by the positive M,
use `M·M⁻¹=1`, and obtain `‖p(A)‖≤2M`. The checked proposition keeps the
normalization algebra separate from the public endpoint declaration. The
case split is essential because the expression `M⁻¹•p` has no useful
normalizing meaning at M=0. In the positive case every inequality direction
is preserved because M>0, while the complex scalar norm reduces to the same
positive real number M.

#### CFT-32-E03 solution

The reduced pole set is finite, hence closed, so its complement is open. The
definition of `RationalPoleFreeOn` is disjointness. The theorem
`rationalPoleFreeOn_iff_subset_compl` converts that disjointness into
`s⊆(rationalPoleSet r)ᶜ`. On the full complement the reduced denominator is
nonzero. Polynomial numerator and denominator functions are holomorphic, and
the quotient rule gives differentiability of `rationalScalarEval r`. These
three facts are returned in exactly the order stated by the exercise.

#### CFT-32-E04 solution

Again separate the empty matrix type. For a nonempty type and M=0, scalar
rational evaluation vanishes on W(A). Pole avoidance makes the reduced
denominator nonzero there, so the reduced numerator vanishes there. The
elementary polynomial zero lemma from E02 gives `num(r)(A)=0`; multiplying by
the inverse denominator gives r(A)=0. No terminal polynomial or rational
Crouzeix estimate occurs in this dependency chain. In the positive branch,
scalar homogeneity is established from a displayed reduced fraction: the old
denominator remains invertible, pole-freeness supplies the new reduced
denominator, and `rationalMatrixEval_fraction` identifies both
representations. The normalized premise becomes
`M⁻¹‖r(A)‖≤2`; multiplying by the positive M proves `‖r(A)‖≤2M`.
Unlike polynomial normalization, this branch must retain the pole-free
hypothesis and denominator inverse. Thus the normalized premise is actually
consumed rather than bypassed by a terminal rational estimate.

#### CFT-32-E05 solution

Set K=W(A). Compactness gives ε>0 whose closed ε-buffer lies in U. Choose N
so every later outer radius is below ε. The theorem
`tendsto_simpleSpectrumHolomorphicEval_of_differentiableOn_neighborhood`
proves the first conjunct. For the second, each shifted outer closure is
compact, contains K, stays in the fixed closed buffer, and every one of its
points is within the corresponding radius of K. Uniform continuity of f on
the buffer and `tendsto_outerApproximationRadius` satisfy every hypothesis of
`tendsto_maxFunctionModulusOnSet_of_outerApproximation`. This produces the
requested shifted maximum limit without interchanging it with the matrix
limit. The shift by N records the compact-buffer threshold explicitly; no
claim is made that the early outer domains lie in U. The first conjunct has
matrix-valued convergence, whereas the second is a real-valued maximum
convergence. Returning them separately prevents a silent diagonal limit in
which the outer domain changes while the matrices are still converging.

#### CFT-32-E06 solution

Use `holomorphicMatrixEval_polynomial A p` for the first conjunct. It proves
that the limit-defined holomorphic calculus agrees with the algebraic
polynomial calculus. The second conjunct is definitional: both sides are the
supremum of `‖Polynomial.eval z p‖` over W(A), with only the public notation
changed. Returning both equalities makes the final specialization auditable.
No exercise theorem calls `jinFinalCrouzeixConjecture`, the public polynomial
alias, or the holomorphic endpoint.

## Synthesis and forward dependencies

The Jin branch is now complete. Chapters 30 and 31 construct and compress the
positive kernel. CFT-32-001 extracts the sharp norm bound, CFT-32-005 removes
the auxiliary hypotheses in the correct order, and CFT-32-006 identifies the
polynomial statement. Rational consequences remain distinct because pole
avoidance and denominator inversion are real mathematical obligations.

## Jin route review

The reader-facing dependency map is

    Chapter 29 → Chapter 30 → Chapter 31 → Chapter 32.

This branch is independent of the Lorist-Schwenninger route. The two branches
meet only in the comparison chapter after each has reached its own endpoint.
No theorem in Chapters 30-32 imports a Lorist-Schwenninger module.

**Notation continuity.** Chapter 29 fixes the numerical range and the target
constant. Chapters 30-32 use one meaning for each of $S$, $G=S^{\mathrm H}S$,
$K=G^{1/2}$, $C=K\Lambda K^{-1}$, $P$, $R$, and $X=R-P$. The definitions
agree with the book's
[[knowledge/crouzeix_textbook/notation_and_glossary|notation and glossary]].
The letter $K(z)$ in Chapter 31 is a matrix-valued completion model and is
always written with its argument; the square root $K$ in Chapters 30 and 32
has no argument.

**Chapter 30 handoff.** The completion predicate and the Gramian congruences
turn source matrices $P,R,X$ into weighted Gramians. The handoff supplies
identities, not the source PSD inequality.

**Chapter 31 handoff.** Exact eigenvalue samples, the origin sample, and
$v=-G^{-1}Pu$ cancel the unknown diagonal correction. Sampled-kernel
positivity then yields $4X-XG^{-1}P-PG^{-1}X\succeq0$.

**Chapter 32 endpoint.** Congruence transports that inequality to the two
weighted Gramians. The extremal-eigenvector argument gives the constant two;
normalization and two ordered limits produce the polynomial, rational, and
holomorphic consequences. The nilpotent member $A_{0,2}$ shows that two is
sharp.

**Formalization boundary.** The route has 18 theorem cards and 18 exercise
solutions. Their public declarations live in
[[formalization/lean/CrouzeixTextbook/Part06/Chapter30.lean|Chapter30.lean]],
[[formalization/lean/CrouzeixTextbook/Part06/Chapter31.lean|Chapter31.lean]],
and
[[formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean|Chapter32.lean]].
The principal provider files are
[[formalization/lean/CrouzeixConjecture/CompletionStatement.lean|CompletionStatement.lean]],
[[formalization/lean/CrouzeixConjecture/CompletionKernelModel.lean|CompletionKernelModel.lean]],
and
[[formalization/lean/CrouzeixConjecture/PositiveRealCompletion.lean|PositiveRealCompletion.lean]].
The `CrouzeixJin` target checks the Jin provider closure; the
`CrouzeixTextbook` target checks public declarations and exercises. Each card's
receipt audit identifies its exact public declaration, provider declaration,
source paths, type digest, axioms, and target. Lean verifies those theorem
statements and proof terms. The prose derivations, historical labels, and ML
analogies remain editorial claims checked by publication tests rather than
propositions in Lean.
