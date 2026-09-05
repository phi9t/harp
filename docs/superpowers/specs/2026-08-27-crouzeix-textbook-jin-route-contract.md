# Jin teaching-route completion contract

This is the executable development contract for Chapters 30--32. It does not
claim that the chapter work already exists. Canonical mathematics remains in
`knowledge/crouzeix_textbook/`; canonical correspondence metadata remains in
`content/crouzeix_textbook/`.

- Active phase: `wave-2-complete`.
- Target phase: `wave-2-complete`.
- Phase order: `baseline-frozen`, `chapter-30-complete`,
  `chapter-31-complete`, `wave-2-complete`.

## Phase model and frozen baseline

`B` means the exact pending state: `prose_proof_status=summary`, the immutable
baseline kind, formal mode, correspondence, declaration, provider, public and
provider hashes below, `lean_solution=null`, and all ten card fields absent.
`C` means `prose_proof_status=reconstructible`, the row-specific completed
kind/mode, `lean_correspondence_status=exact`, a fresh-receipt-bound public and
provider row, six distinct solutions for the chapter, and all ten fields
present. Tests construct `B` directly from the immutable specification rather
than cloning live JSON, then derive every later phase from it.

| Item | Baseline kind/mode/correspondence | Completed kind/mode | baseline | ch30 | ch31 | wave2 |
|---|---|---|---|---|---|---|
| CFT-30-001 | definition / checkpoint / checkpoint | definition / definition | B | C | C | C |
| CFT-30-002 | theorem / checkpoint / checkpoint | theorem / reexported-proof | B | C | C | C |
| CFT-30-003 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | C | C | C |
| CFT-30-004 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | C | C | C |
| CFT-30-005 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | C | C | C |
| CFT-30-006 | theorem / reexported-proof / unmapped | theorem / proved-here | B | C | C | C |
| CFT-31-001 | definition / checkpoint / checkpoint | definition / definition | B | B | C | C |
| CFT-31-002 | theorem / reexported-proof / unmapped | theorem / proved-here | B | B | C | C |
| CFT-31-003 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | B | C | C |
| CFT-31-004 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | B | C | C |
| CFT-31-005 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | B | C | C |
| CFT-31-006 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | B | C | C |
| CFT-32-001 | definition / reexported-proof / unmapped | theorem / proved-here | B | B | B | C |
| CFT-32-002 | theorem / checkpoint / checkpoint | theorem / reexported-proof | B | B | B | C |
| CFT-32-003 | theorem / checkpoint / checkpoint | theorem / reexported-proof | B | B | B | C |
| CFT-32-004 | theorem / checkpoint / checkpoint | theorem / reexported-proof | B | B | B | C |
| CFT-32-005 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | B | B | C |
| CFT-32-006 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | B | B | C |

The baseline at `b85b3db7` has exactly these eighteen rows: fifteen theorems,
three definitions, six checkpoints, twelve unmapped provider reexports, and
eighteen null exercise solutions. Tests construct and mutate all four phases;
aggregate counts cannot substitute for this per-row table.

For a pending card, every field is absent: `Purpose`, `Statement`,
`Hypothesis ledger`, `Proof roadmap`, `Proof`, `Boundary case`,
`Pedagogical prerequisites`, `Lean correspondence`, `Historical context`, and
`ML analogy`. Complete cards have exactly these headings in that order.
`Purpose` labels `Motivation.`. `ML analogy` labels `Mathematical object / ML
counterpart.`, `Exact transfer.`, `Non-transfer.`, and `Diagnostic.`

## Mathematical completion obligations

Within a cell, order is load-bearing. Only visible rendered text counts.

| Item | Exact hypotheses | Ordered statement/equations | Ordered proof transitions |
|---|---|---|---|
| CFT-30-001 | `B,T∈M_n(ℂ)`; `H:ℂ→M_n(ℂ)`; unit disk | completion predicate; analytic; `H(0)=I`; `IsPositiveMatrix(Re H(z))`, hence PSD/nonnegative and not strict `≻0`; algebra defect in `alg(Bᴴ)` | analytic `∧` normalized `∧` positive-real-PSD `∧` algebra-defect |
| CFT-30-002 | `[Nonempty n]`; `B=Sdiag(μ)S⁻¹`, distinct `μ_i`; `T=Sdiag(λ)S⁻¹`; `|λ_i|≤1`; completion | shared basis; simple spectrum belongs to `B`; `‖T‖≤2` | use simplicity of `B`, not `T`; shared-basis completion; norm bound |
| CFT-30-003 | `G=K²`; two-sided inverse; `|λ_i|≤1`; `P` | `P_ij=G_ij/(1-conj(λ_i)λ_j/4)`; `C=Kdiag(λ)K⁻¹`; `K⁻¹PK⁻¹=Gramian_4(C)` | expand `4⁻ᵏ(Λᵏ)ᴴGΛᵏ`; conjugate termwise; sum |
| CFT-30-004 | same; `R` | `R_ij=G_ij/(1-conj(λ_i)λ_j/2)`; same `C`; `K⁻¹RK⁻¹=Gramian_2(C)` | expand `2⁻ᵏ(Λᵏ)ᴴGΛᵏ`; conjugate termwise; sum |
| CFT-30-005 | define `X:=R-P`; square-root/inverse/spectral hypotheses | `K⁻¹XK⁻¹=Gramian_2(C)-Gramian_4(C)` | distribute congruence; substitute both preceding congruences |
| CFT-30-006 | `G=K²`; self-adjoint `K,K⁻¹`; `Y=4X-XG⁻¹P-PG⁻¹X⪰0` | source PSD; `K⁻¹YK⁻¹⪰0`; `4(G₂-G₄)-(G₂-G₄)G₄-G₄(G₂-G₄)⪰0` | source; congruence; replace `X`; replace `P`; Gramian expression |
| CFT-31-001 | `G`; `Λ=diag(λ)`; `D(z)=diag(d_i(z))`; `Q(z)=diag((1-zλ_j)⁻¹)` | `K(z)=GQ(z)+D(z)G`; exact entry formula; under `1-zλ_j≠0` for every `j`, `Q(z)=(I-zΛ)⁻¹` | diagonal entry; matrix products in their stated order; add; identify the matrix inverse only under the pole-free condition |
| CFT-31-002 | `K(0)=G`; `G` invertible; `D(0)` diagonal | `K(0)=G+D(0)G`; therefore `D(0)G=0`; then `D(0)=0` | derive `D(0)G=0` from the normalization; cancel the invertible `G`; conclude `D(0)=0`. This is the reverse implication, not `d(0)=0 ⇒ K(0)=G`. |
| CFT-31-003 | `z_0=0`; `z_i=conj(λ_i)/2`; `x_0=v=-G⁻¹Pu`; `x_i=u_i e_i`; dimensions of every matrix/vector | sample/sample `4R-2P`; sample/origin `G+R`; origin/sample `G+R`; origin/origin `2G` | expand `L_0` at all four ordered sample pairs; prove the scalar partial-fraction identity for `4R-2P`; apply sparse selection; sum the surviving coordinates without commuting factors |
| CFT-31-004 | coordinatewise sampled diagonal `Δ`; `G=Gᴴ`; `P=Pᴴ`; `d(0)=0`; `v=-G⁻¹Pu` | selected correction entries `(ΔP+PΔᴴ)_ij`, `(ΔG)_ij`, `(GΔᴴ)_ij`, and the zero origin block; raw ordered sum; `Θ=u*Δ(Gv+Pu)`; correction sum `=Θ+conj(Θ)` | derive the three selected entries and the zero origin block without asserting false full-matrix equalities; expand the raw sum; write the coordinate sum for `Θ`; conjugate it using both Hermiticity hypotheses; add |
| CFT-31-005 | `G` invertible; `v=-G⁻¹Pu`; `d(0)=0` | `Gv+Pu=0`; `Θ=0`; `Θ+conj(Θ)=0` | substitute `v`; cancel `GG⁻¹`; zero every coordinate; conclude |
| CFT-31-006 | positive two-variable kernel `L_K`; self-adjoint invertible `G`; spectral bound; `d(0)=0` | sampled-kernel PSD; block-matrix PSD; universal scalar quadratic nonnegativity; matrix PSD | state the separate source bridge from analyticity plus pointwise positive-real behavior to positivity of `L_K`; sample finitely; test the block matrix; cancel the correction; identify the quadratic form; prove its coefficient Hermitian; apply the PSD criterion |
| CFT-32-001 | invertible `S`; `G=S*S`; balanced `C`; spectral bound; positive kernel | Gramian inequality; `G₄≤2I`; positive Gramian tail; `C*C≤4I`; `‖C‖≤2`; transfer to `SΛS⁻¹` | choose `e` with `G₄e=pe`; assume `p>2`; show `e*(G₂-G₄)e=0`; use `G₂-G₄⪰₀` to get `(G₂-G₄)e=0`; put `Y=(1/4)C*C`; combine `G₂-G₄-Y⪰₀` and `Y⪰₀` to obtain `e*Ye=0`; hence `Ce=0`; then `G₄e=e`, contradicting `G₄e=pe`; conclude `G₄≤2I`; use `G₄=I+Σ_{k≥1}4⁻ᵏ(Cᵏ)*Cᵏ≥I`; obtain `C*C≤4(G₄-I)≤4I`; extract `‖C‖≤2`; transfer the norm |
| CFT-32-002 | `A,p`; `M=max_{W(A)}|p|`; fixed outer domains `Ω_m`; `M_m=max_{closure(Ω_m)}|p|`; simple-spectrum `A_j→A` with `W(A_j)⊆Ω_m` eventually; supported boundary Cauchy identities | `q_m=p/M_m`; `|q_m(z)|≤1` on `closure(Ω_m)`; `μ_{j,i}=q_m(λ_{j,i})`; `q_m(A_j)=S_j diag(μ_j)S_j⁻¹`; `‖q_m(A_j)‖≤2`; `‖p(A)‖≤2M` | handle `M=0`; for `M>0`, note a bound on `W(A)` does not control perturbation eigenvalues; fix `m`; normalize by `M_m`; establish the same-basis functional-calculus identity; use the outer-boundary Cauchy/Cayley identities to construct the positive-real completion; pull it back to obtain `d_j(0)=0` and the positive Herglotz kernel; only then map all hypotheses into CFT-32-001; pass `j→∞` with `m` fixed; only then pass `m→∞` using `M_m→M` |
| CFT-32-003 | poles avoid `W(A)`; rational calculus | pole-free neighborhood; rational constant-two bound | obtain neighborhood holomorphy; apply holomorphic bound |
| CFT-32-004 | rational maximum `M`; pole avoidance | `s=r/M`; `max|s|≤1`; `‖r(A)‖≤2M` | `M=0`; normalize; spectral-set bound; rescale |
| CFT-32-005 | `U⊇W(A)`; `f` holomorphic | fixed-domain simple-spectrum limit; outer-domain limit; holomorphic bound | approximate `A` first; pass evaluations/maxima; only then shrink domains |
| CFT-32-006 | holomorphic hypotheses; polynomial `p` | `f=p`; identify evaluations; identify maxima; polynomial bound | complete outer-domain limit first; specialize and rewrite both sides |

Generic word counts cannot satisfy these scoped equation and ordering checks.

Chapter 31 fixes its matrix data before the first argument:
`P_ij=G_ij/(1-conj(λ_i)λ_j/4)`,
`R_ij=G_ij/(1-conj(λ_i)λ_j/2)`, and `X=R-P`. It reserves `K(z)` for the
one-variable matrix function and writes its two-variable Herglotz kernel as
`L_K(z,w)=(1-zconj(w))⁻¹(K(z)+K(w)ᴴ)`. The chapter must not overload `K`
with two arguments or introduce an undefined `K_H` synonym.

## Exact Lean map

In `baseline-frozen`, every public row and every provider row is required
exactly once from fresh compiler introspection. The public row is a
`direct-alias` with exactly the provider as its body dependency, the exact
public file/hash below, axioms `Classical.choice,Quot.sound,propext`, and target
`CrouzeixTextbook`. The provider has its own exact file and hash; pretty
printing a provider in its defining module is not assumed to produce the same
hash as pretty printing the public alias. A test-only compiler probe supplies
provider rows that are intentionally outside the maintained correspondence
marker roster; absence or duplication is an error, never an optional lookup.

At completion, metadata is compared to the active fresh receipt rather than
to the old alias hash. A `reexported-proof` still has exactly its named
provider. A `proved-here` theorem has no `underlying_declaration`, elaborates
as theorem, and must directly depend on its named proof bridge. Completed cards
visibly record the active receipt values in `Receipt audit: {...}`.

The baseline provider hash differs from the public hash only in these four
compiler-printer cases:

- CFT-30-001 provider: `484e9737cde90a145a1db689cb649f22f3ee2f1cf3e299bd52d3a2374b9686a5`.
- CFT-31-001 provider: `36e752a3852f89e9a1bc7b1e567b974b702759d6ec39097a448fb277a00fb7ce`.
- CFT-32-002 provider: `5a08b37106dc806e9cb5f69cb03f5e13bd850055d67cb410ad615924faaab9c2`.
- CFT-32-004 provider: `65ba1f3e8ad1e9fd48dede033d58509648d2c01d5e7cb5d49b09d967d2affe34`.

| Item | Public declaration | Provider declaration / provider file | Type SHA-256 |
|---|---|---|---|
| CFT-30-001 | `CrouzeixTextbook.Part06.positive_real_completion` | `CrouzeixConjecture.IsPositiveRealCompletion` / `CompletionStatement.lean` | `d72a6d691c4c807c0b20b620c2e030345c8d923a1c9b31d1a2a2e9fb07732523` |
| CFT-30-002 | `CrouzeixTextbook.Part06.positive_real_completion_statement` | `CrouzeixConjecture.PositiveRealCompletionStatement` / `CompletionStatement.lean` | `23602f64481feee6c2fd6a0f7e743a74f4a1ee8625f11e06048a5c82ff99aeab` |
| CFT-30-003 | `CrouzeixTextbook.Part06.completion_gramian_four` | `CrouzeixConjecture.completionP_congruence_eq_gramian_four` / `CompletionGramianBridge.lean` | `d999ab712880d5363dbc792dfeaa9c208ec4eae8435b470a1c25b4344c83fbdc` |
| CFT-30-004 | `CrouzeixTextbook.Part06.completion_gramian_two` | `CrouzeixConjecture.completionR_congruence_eq_gramian_two` / `CompletionGramianBridge.lean` | `e9a78b5db14824f47398ed1a8a9cdbfcc1bbfae558930039ddbbb5d431dd3f84` |
| CFT-30-005 | `CrouzeixTextbook.Part06.completion_gramian_difference` | `CrouzeixConjecture.completionX_congruence_eq_gramian_difference` / `CompletionGramianBridge.lean` | `2e7c01edefeb5dc2a3212ae85afc54e9caea46aaacf62c7996ad3de1ab1c1b67` |
| CFT-30-006 | `CrouzeixTextbook.Part06.completion_gramian_source_positive` | `CrouzeixConjecture.completion_gramian_expression_posSemidef_of_source` / `CompletionGramianBridge.lean` | `f9075e863bc01786d9635fb4461c6901054f999834318bfaf265db564791c01d` |
| CFT-31-001 | `CrouzeixTextbook.Part06.completion_kernel_model` | `CrouzeixConjecture.completionKernelModel` / `CompletionKernelModel.lean` | `b6f13ba7834ee232d6361bbf95f939ad00760f82d267ffde7b3c22ed1b5a48b1` |
| CFT-31-002 | `CrouzeixTextbook.Part06.completion_kernel_at_zero` | `CrouzeixConjecture.completionKernelModel_zero` / `CompletionKernelModel.lean` | `4749a4faa985491913c85c0e2c8b4194e3338905cc25dd01a2bca6426686e927` |
| CFT-31-003 | `CrouzeixTextbook.Part06.sample_origin_quadratic_identity` | `CrouzeixConjecture.completionResolventKernel_sampling_quadratic_eq` / `CompletionKernelModel.lean` | `a19830b823dec21c68e5ab7b248b9d78c9b03518b8605d869ee888192c2db7db` |
| CFT-31-004 | `CrouzeixTextbook.Part06.correction_sampling_identity` | `CrouzeixConjecture.completionCorrectionKernel_sampling_quadratic_eq` / `CompletionKernelModel.lean` | `3fbcbf6aba9f45dc919a68fa512cb886899febda58461c932c11d93054a74053` |
| CFT-31-005 | `CrouzeixTextbook.Part06.correction_sampling_cancels` | `CrouzeixConjecture.completionCorrectionKernel_sampling_eq_zero` / `CompletionKernelModel.lean` | `3883ecf653a417da1c0aceb9c8ee7dd5f2c9444bf7fa7936660051df5e071234` |
| CFT-31-006 | `CrouzeixTextbook.Part06.kernel_positivity_implies_X` | `CrouzeixConjecture.completion_X_inequality_of_positiveKernel` / `CompletionKernelModel.lean` | `8ea6a00ebbb6a7326410c88a8d764396afc51794cde409a6c5f3b44855ce7e04` |
| CFT-32-001 | `CrouzeixTextbook.Part06.completion_implies_norm_two` | `CrouzeixConjecture.norm_completionDiagonalizableMatrix_le_two_of_positiveKernelModel` / `PositiveRealCompletion.lean` | `1c24687b99a83a92aa728551ff33b388ec0e9f1551a6a36145d826b25311ccd1` |
| CFT-32-002 | `CrouzeixTextbook.Part06.jin_polynomial_constant_two` | `CrouzeixConjecture.jinFinalCrouzeixConjecture` / `FinalTheorems.lean` | `0c2cdfce0642e8d4ab88dc860b6a60e85be855897e5f63fd51babbf67c87fc5d` |
| CFT-32-003 | `CrouzeixTextbook.Part06.jin_rational_spectral_set` | `CrouzeixConjecture.crouzeixRationalSpectralSetCorollary` / `FinalTheorems.lean` | `a68683384ff3984804bccbfaf7f5dea8ca6128d26a0618984bc5d293837f0144` |
| CFT-32-004 | `CrouzeixTextbook.Part06.jin_rational_constant_two` | `CrouzeixConjecture.crouzeixRationalBound` / `FinalTheorems.lean` | `45eabf35921b773b4ec7c3aacb0af28c045bd1f008d6f6ced30c63724d5a91c8` |
| CFT-32-005 | `CrouzeixTextbook.Part06.holomorphic_constant_two` | `CrouzeixConjecture.holomorphicCrouzeixBound` / `HolomorphicOuterLimit.lean` | `46a9ed9e7bfeb40a079ae46d8cb6d2bbb0d0d330b9eaf2646c0f15ddae2552b8` |
| CFT-32-006 | `CrouzeixTextbook.Part06.polynomial_from_holomorphic` | `CrouzeixConjecture.polynomialCrouzeixBound_of_holomorphicCrouzeixBound` / `HolomorphicConsequences.lean` | `0c2cdfce0642e8d4ab88dc860b6a60e85be855897e5f63fd51babbf67c87fc5d` |

Wave 2 replaces the baseline alias-of-alias providers for three endpoint
reexports with theorem-kind providers, making the receipt boundary exact:

| Item | Wave-2 theorem provider | Provider file |
|---|---|---|
| CFT-32-002 | `CrouzeixConjecture.polynomialCrouzeixBound_of_holomorphicCrouzeixBound` | `formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean` |
| CFT-32-003 | `CrouzeixTextbook.Part06.jin_rational_spectral_set_provider` | `formalization/lean/CrouzeixTextbook/Part06/Chapter32.lean` |
| CFT-32-004 | `CrouzeixConjecture.holomorphicCrouzeixRationalBound` | `formalization/lean/CrouzeixConjecture/HolomorphicConsequences.lean` |

For completed CFT-32-003, `CrouzeixConjecture.holomorphicCrouzeixRationalBound`
is a proof dependency of the local theorem-kind provider, not the direct
provider recorded by the correspondence row.

Chapter 30 completion changes three active receipt relationships without
rewriting that immutable baseline. CFT-30-001 is a definition alias whose
completed provider/type identity is
`d72a6d691c4c807c0b20b620c2e030345c8d923a1c9b31d1a2a2e9fb07732523`.
CFT-30-002 reexports theorem
`CrouzeixConjecture.positiveRealCompletionStatement` from
`CompletionDiagonalization.lean`, including `[Nonempty n]`, with active hash
`720010d2fd3b19be340951f1fda1fd0e02a095476a0bdd4a60e8af086352deb2`.
CFT-30-006 is a local `proved-here` theorem with active hash
`2c90dd2e81251f789c757fc791515b9d4889f0c5c8f0e74fc1b7ce75e8f17356`;
it exposes congruenced PSD before using
`CrouzeixConjecture.completion_gramian_expression_posSemidef_of_source` for
the final Gramian PSD. That provider retains its narrower final-PSD statement
and therefore has the distinct active provider hash
`54e0b92ee477409a1b741eb87362322c7a801bb9c9ecdb4b61a1320140728baa`.

Public files are exactly `CrouzeixTextbook/Part06/Chapter30.lean`,
`Chapter31.lean`, and `Chapter32.lean` for their respective chapters. The
visible audit uses repository-qualified paths.

CFT-31-002 is deliberately phase-split. Its baseline public alias remains
`CrouzeixTextbook.Part06.completion_kernel_at_zero` over
`CrouzeixConjecture.completionKernelModel_zero`; that theorem proves the
opposite direction `d(0)=0 ⇒ K(0)=G` and cannot discharge the teaching card.
Chapter 31 completion must instead publish the proved-here theorem
`CrouzeixTextbook.Part06.completion_kernel_normalization_forces_correction_zero`
with the local provider bridge
`CrouzeixTextbook.Part06.completion_kernel_normalization_forces_correction_zero_bridge`.
The human-readable target is: invertible `G` and
`completionKernelModel G lambda d 0 = G` imply `d 0 = 0`. The complete Lean
signature, including type, instance, matrix, function, invertibility, and
normalization binders, is
`CrouzeixTextbook.JinContractProbe.cft_31_002_public` in the ephemeral contract
probe described below. Its compiler row, not this display sentence, supplies
the completed public and provider type hash.

The maintained public receipt renders the complex scalar type as `Complex`,
while the provider probe renders the definitionally equal notation `ℂ` for
the local bridge. The completed public hash is therefore
`c366aa91bab218699594028032e0feca8e1d84f06804e10dd48e71a7625ccb49`,
while the exact provider-probe hash is
`22fb9474d934ab848aa65584ab7f1a7122b901afc9aae3553554bd415ac1eefe`.
Lean checks the bridge application in the public proof; the signature probe
still checks the full proposition and the negative omitted-invertibility
mutation.

CFT-32-001 likewise changes from a baseline definition alias to a completed
`proved-here` theorem at the same public name. Its fresh receipt must name
`CrouzeixConjecture.norm_completionDiagonalizableMatrix_le_two_of_positiveKernelModel`
as the required provider and expose the ordered eigenvector argument above.

## Exercise proof obligations

Each completed chapter has exactly six theorem declarations named
`CrouzeixTextbook.Part06.Exercises.ChapterNN.exercise_01_solution` through
`exercise_06_solution`, exactly once in active
`checkedExerciseTheoremNames`. Its compiler type must expose the shape below.

| Exercise | Parent | Proposition shape | Allowed CFT prerequisites |
|---|---|---|---|
| CFT-30-E01 | CFT-30-001 | four analytic/normalization/positivity/algebra-defect conjuncts | CFT-29-002 |
| CFT-30-E02 | CFT-30-002 | distinct `μ`, shared basis, identify simple matrix | CFT-30-001 |
| CFT-30-E03 | CFT-30-003 | `completionP`, `gramianFour`, congruence | CFT-30-001 |
| CFT-30-E04 | CFT-30-004 | `completionR`, `gramianTwo`, congruence | CFT-30-001 |
| CFT-30-E05 | CFT-30-005 | `X=R-P`, difference of Gramians | CFT-30-003, CFT-30-004 |
| CFT-30-E06 | CFT-30-006 | source PSD gives congruenced PSD plus exact ordered equality to the Gramian expression | CFT-30-003, CFT-30-004, CFT-30-005 |
| CFT-31-E01 | CFT-31-001 | conjunction of sparse-vector column selection and the completion-kernel entry formula | CFT-30-006 |
| CFT-31-E02 | CFT-31-002 | normalization `K(0)=G` and invertibility give both matrix `D(0)=0` and coordinate function `d(0)=0` | CFT-31-001 |
| CFT-31-E03 | CFT-31-003 | four sample/origin blocks, `4R-2P`, `2G` | CFT-31-001, CFT-31-002 |
| CFT-31-E04 | CFT-31-004 | complete correction sum equals `Θ+conj(Θ)`, using Hermiticity and `d(0)=0` | CFT-31-003 |
| CFT-31-E05 | CFT-31-005 | `v=-G⁻¹Pu`, `Gv+Pu=0`, `Θ=0`, and `Θ+conj(Θ)=0`; it becomes the complete correction sum only after E04's hypotheses | CFT-31-004 |
| CFT-31-E06 | CFT-31-006 | universal quadratic nonnegativity implies PSD | CFT-31-003, CFT-31-004, CFT-31-005 |
| CFT-32-E01 | CFT-32-001 | kernel/eigenvector cases and `‖C‖≤2` | CFT-30-006, CFT-31-006 |
| CFT-32-E02 | CFT-32-002 | polynomial `M=0/M>0` normalization | CFT-32-001 |
| CFT-32-E03 | CFT-32-003 | pole avoidance gives neighborhood holomorphy | CFT-32-001 |
| CFT-32-E04 | CFT-32-004 | rational `M=0/M>0` normalization | CFT-32-003 |
| CFT-32-E05 | CFT-32-005 | simple-spectrum limit before outer-domain limit | CFT-32-001, CFT-32-003, CFT-32-004 |
| CFT-32-E06 | CFT-32-006 | polynomial specialization after outer limit | CFT-32-005 |

The short descriptions above are mathematical reading aids, not type-hash
inputs. The durable executable signature source is the raw Lean declaration
block `JIN_CONTRACT_PROBE_DECLARATIONS` in
`crates/harp/tests/crouzeix_textbook.rs`. During the test, that block is written
to a private temporary `JinContractProbe.lean`, imports only
`CrouzeixTextbook.ExportReceipt`, and declares nineteen canonical targets plus
one negative mutation axiom (twenty total): one target for each exercise, the
reverse-direction CFT-31-002 target, and its omitted-invertibility mutation.
Lean must elaborate the whole block. `ExportReceipt.receiptRows` then returns each
declaration's compiler `kind`, normalized `ppExpr` type, and type SHA-256.
Those compiler values drive all four phase fixtures and completed-phase
validation. The temporary module is never imported by a project module and
adds no axiom to the textbook library.

The exact exercise probe roster is:

- CFT-30-E01 through E06:
  `CrouzeixTextbook.JinContractProbe.cft_30_e01`,
  `CrouzeixTextbook.JinContractProbe.cft_30_e02`,
  `CrouzeixTextbook.JinContractProbe.cft_30_e03`,
  `CrouzeixTextbook.JinContractProbe.cft_30_e04`,
  `CrouzeixTextbook.JinContractProbe.cft_30_e05`, and
  `CrouzeixTextbook.JinContractProbe.cft_30_e06`.
- CFT-31-E01 through E06:
  `CrouzeixTextbook.JinContractProbe.cft_31_e01`,
  `CrouzeixTextbook.JinContractProbe.cft_31_e02`,
  `CrouzeixTextbook.JinContractProbe.cft_31_e03`,
  `CrouzeixTextbook.JinContractProbe.cft_31_e04`,
  `CrouzeixTextbook.JinContractProbe.cft_31_e05`, and
  `CrouzeixTextbook.JinContractProbe.cft_31_e06`.
- CFT-32-E01 through E06:
  `CrouzeixTextbook.JinContractProbe.cft_32_e01`,
  `CrouzeixTextbook.JinContractProbe.cft_32_e02`,
  `CrouzeixTextbook.JinContractProbe.cft_32_e03`,
  `CrouzeixTextbook.JinContractProbe.cft_32_e04`,
  `CrouzeixTextbook.JinContractProbe.cft_32_e05`, and
  `CrouzeixTextbook.JinContractProbe.cft_32_e06`.

The negative compiler row
`CrouzeixTextbook.JinContractProbe.mutation_cft_31_002_without_invertibility`
elaborates a deliberately weaker signature. Substituting it for the canonical
CFT-31-002 row must fail, as must a hand-injected display pseudotype even when
its SHA-256 is internally consistent.

Solutions may use only listed Jin prerequisites plus foundations. They may not
depend on their phase-active parent public/provider declaration, an unlisted
Jin CFT, or a terminal polynomial, rational, or holomorphic endpoint at any
depth in their transitive dependency closure. The parent guard therefore uses
the completed Chapter 31 reverse theorem and its local bridge after the phase
advances; mutations exercise both direct reuse and reuse hidden behind a
helper. Each Chapter30--32 exercise namespace contains exactly its six
expected theorem names and no helpers or extra declarations. Required E01,
E04, and E05 proof dependencies are read only from the compiler receipt and
its dependency closure; names appearing only in Lean comments cannot satisfy
the requirement. Negative fixtures reject `True := by trivial`,
assumed-conclusion plumbing, direct aliases, eta aliases, direct and
helper-hidden parent or terminal shortcuts, extra namespace declarations,
extra proposition conclusions, and name-only exporter comments.

## Pinned sources

Source identity: commit `565b6a3e0659b6e0785f783b016c3f6d9f171fa5`,
1,106 lines, SHA-256
`5713de029c4a7486e25e86d16e6413d04929bdf5f92439c3237d4a930b1c9242`.

- `git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L319-L386`
- `git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L387-L463`
- `git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L464-L530`
- `git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910`
- `git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/CrouzeixConjecture/Statements.lean#L13-L18`
- `git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/CrouzeixConjecture/Statements.lean#L21-L22`
- `git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/CrouzeixConjecture/FinalTheorems.lean#L15-L23`

Authorities: `evidence/crouzeix_conjecture/source_manifest.tsv#L11` and
`labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/source-map.json#L1`.

## Dependency and route independence

The load-bearing chain is
`CFT-32-006 →* CFT-31-006 →* CFT-30-006 →* CFT-30-001 →* CFT-29-002`.
No CFT-30--32 node reaches CFT-33--34, and no CFT-33--34 node reaches
CFT-30--32. The first node allowed to reach both branches is in Chapter 35.

`CrouzeixJin` is a dedicated Lake library. The canonical nested-comment-aware
parser follows its transitive closure and rejects
`Crouzeix.LoristSchwenninger.*`, `CrouzeixLoristSchwenninger`,
`Crouzeix.Harp.*`, `CrouzeixHarp`, and the all-provider `Crouzeix` aggregate.
`mise run lean-crouzeix-jin` must compile against the pinned warm cache.

## Exit rule

Advance a phase only with all six scoped cards, six structurally distinct
exercise theorems, exact metadata, locators, fresh receipt comparison, and
anti-shortcut tests. Wave 2 completes only when all eighteen rows are `C`, all
exercise proofs pass receipt validation, and both independence checks hold.
