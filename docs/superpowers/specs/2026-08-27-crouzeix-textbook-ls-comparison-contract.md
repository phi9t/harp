# LS realization and three-route comparison contract

This document is the executable development contract for Chapters 34 and 35.
It freezes the state before the chapter mathematics is rewritten. Canonical
mathematics remains under `knowledge/crouzeix_textbook/`; canonical
correspondence metadata remains under `content/crouzeix_textbook/`.

- Active phase: `three-route-comparison-complete`.
- Target phase: `three-route-comparison-complete`.
- Phase order: `baseline-frozen`, `chapter-34-boundary-data`,
  `chapter-34-moments`, `chapter-34-realization-complete`,
  `chapter-35-consequences`, `three-route-comparison-complete`.

## Phase model and baseline gaps

The phase transitions follow Wave 3 Tasks 2 through 6. The model is monotone,
but exercise readiness is explicit. Task 2 completes CFT-34-001 and Exercises
34-E01 and 34-E02. Task 3 completes CFT-34-002 and CFT-34-003 plus Exercises
34-E03 and 34-E04. Task 4 completes the remaining Chapter 34 rows and
exercises. Task 5 completes CFT-35-001 through CFT-35-005 and their five
exercises. Task 6 alone completes CFT-35-006 and Exercise 35-E06.

`B` is the frozen baseline. `C` is complete. `P` is still pending.

| Item | Baseline kind / mode / correspondence | Completed kind / mode | baseline | boundary | moments | realization | consequences | comparison |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CFT-34-001 | definition / checkpoint / checkpoint | definition / definition | B | C | C | C | C | C |
| CFT-34-002 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | P | C | C | C | C |
| CFT-34-003 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | P | C | C | C | C |
| CFT-34-004 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | P | P | C | C | C |
| CFT-34-005 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | P | P | C | C | C |
| CFT-34-006 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | P | P | C | C | C |
| CFT-35-001 | definition / reexported-proof / unmapped | theorem / reexported-proof | B | P | P | P | C | C |
| CFT-35-002 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | P | P | P | C | C |
| CFT-35-003 | theorem / reexported-proof / unmapped | theorem / reexported-proof | B | P | P | P | C | C |
| CFT-35-004 | theorem / checkpoint / checkpoint | theorem / reexported-proof | B | P | P | P | C | C |
| CFT-35-005 | theorem / checkpoint / checkpoint | theorem / reexported-proof | B | P | P | P | C | C |
| CFT-35-006 | theorem / checkpoint / checkpoint | theorem / proved-here | B | P | P | P | P | C |

The baseline has exactly 12 summary rows, four checkpoints, eight unmapped
rows, and 12 null exercise solutions. Chapter 34 currently gives a compressed
overview rather than the field-by-field construction. Chapter 35 currently
compares only Jin and Lorist--Schwenninger in prose. It does not import Harp or
compile a three-provider proof term. The LS source graph also has dated
`blocked` fields. Those fields are historical status, not the current compiler
result. This contract does not rewrite them.

The declaration `CrouzeixTextbook.Part06.three_route_terminal_bundle` is
forbidden in the first five phases and is required exactly once in the final
phase. Its final compiler row must classify it as a theorem. The six existing
Chapter 35 compatibility declarations remain compiler-classified direct
aliases in every phase, even when the associated coverage row advances from a
checkpoint to a proved theorem. Declaration kind is taken from the compiler
receipt, never from matching source text such as `def` or `theorem`.

Every pending card has none of the ten future fields. A completed card has
these headings, once and in this order: `Purpose`, `Statement`, `Hypothesis
ledger`, `Proof roadmap`, `Proof`, `Boundary case`, `Pedagogical
prerequisites`, `Lean correspondence`, `Historical context`, and `ML analogy`.
The purpose begins with `Motivation.`. The ML field contains `Mathematical
object / ML counterpart.`, `Exact transfer.`, `Non-transfer.`, and
`Diagnostic.`. Keyword-only filler does not satisfy the contract because each
row below also freezes its equations and proof order.

## Mathematical obligations

The phrases within each cell are ordered. Future prose must display them in
this order. They are deliberately concrete so a reviewer can distinguish a
proof from an account of a proof.

| Item | Exact hypotheses | Ordered statement and equations | Ordered proof transitions |
| --- | --- | --- | --- |
| CFT-34-001 | i n : Type*; TopologicalSpace i; CompactSpace i; MeasurableSpace i; BorelSpace i; OpensMeasurableSpace i; SecondCountableTopologyEither i ℂ; Fintype n; DecidableEq n; Nonempty n; μ : Measure i; IsFiniteMeasure μ; Ω : Set ℂ; EuclideanVector n finite-dimensional complete; L2(μ; EuclideanVector n) complete; Γ : ParametricConvexBoundary Ω; B : SquareMatrix n; hWB : numericalRange B ⊆ Ω; q : Polynomial ℂ; hq : ∀ z ∈ closure Ω, ‖Polynomial.eval z q‖ ≤ 1; hCauchy : HasParametricPolynomialCauchyFormula Γ μ B | D := parametricPositiveBoundaryDensity; V := (boundaryEmbedding D).toContinuousLinearMap; Q := bcfMulL h; C_k := ∫ σ, star ((h^k) σ) • F(σ) ∂μ; E_k := euclideanOperator C_k; DilationData | V_isometry; Q_norm_le_one; perturbation_eq; bound_nonneg; perturbation_norm_le; commutes_with_target |
| CFT-34-002 | positive boundary density D; bounded continuous h | V* M_h V; boundaryPhiCLM D h; euclideanOperator | expand the L2 inner product; insert the square root field; use D^(1/2)D^(1/2)=D; move the integral through the continuous linear map |
| CFT-34-003 | positive boundary density D; bounded continuous h; k : Nat | V* (M_h)^k V; boundaryPhiCLM D (h^k) | rewrite bcfMulL_pow; apply the first-moment compression identity to h^k; explain why k=1 alone is insufficient |
| CFT-34-004 | bounded continuous h; k : Nat | M_(h^k); (M_h)^k | base k=0; pow_succ; bcfMulL_mul; continuous-linear-map extensionality |
| CFT-34-005 | bounded continuous h; norm h <= 1 | norm (M_h) <= norm h; norm (M_h) <= 1 | pointwise norm bound; Lp norm bound; transitivity |
| CFT-34-006 | W(B) subset Omega; norm q <= 1 on closure Omega; power Cauchy formula | norm (euclideanOperator (polynomialEval q B)) <= 2 | construct dilationDataOfParametricPolynomial; apply Chapter 33 perturbation endpoint; identify the normalized polynomial evaluation |
| CFT-35-001 | finite nonempty index type; A; polynomial p | norm (polynomialEval p A); 2 * maxPolynomialModulusOnNumericalRange A p | fix an outer domain; normalize the polynomial; apply CFT-34-006; pass simple-spectrum limit; pass outer-domain limit |
| CFT-35-002 | d : Nat; Nonempty (Fin d) | FiniteMatrixMainTheoremStatement; Fin d | specialize CFT-35-001; identify SquareMatrix (Fin d) |
| CFT-35-003 | rational r; poles avoid W(A) | norm (rationalMatrixEval r A); 2 * maxRationalModulusOnNumericalRange A r | polynomial approximants; uniform scalar convergence; matrix evaluation convergence; maximum convergence; limit inequality |
| CFT-35-004 | complete nontrivial complex Hilbert space H; bounded operator A; polynomial p | operatorPolynomialEval p A; 2 * supPolynomialModulusOnOperatorNumericalRange A p | finite-dimensional range model; Fin m coordinate matrix; transport back |
| CFT-35-005 | complete nontrivial complex Hilbert space H; bounded operator A | IsCompact (closedOperatorNumericalRange A); spectrum ℂ A ⊆ closedOperatorNumericalRange A; rational constant-two inequality | closedOperatorNumericalRange_isCompact; spectrum_subset_closedOperatorNumericalRange_of_mainTheorem; hilbertSpaceRationalCrouzeix_of_mainTheorem |
| CFT-35-006 | finite nonempty index type | Jin MainTheoremStatement; Lorist-Schwenninger MainTheoremStatement; Harp MainTheoremStatement | jinFinalCrouzeixConjecture; loristSchwenningerMainTheorem; harpFiniteHorizonMainTheorem |

Chapter 34 must also include the finite atomic boundary worked model, state
the strict-contraction boundary case, and calculate an empirical moment
residual for the running nonnormal matrix. Chapter 35 must compare objects,
hypotheses, shared machinery, decisive mechanism, approximation order,
conclusion, provenance, and formal provider. A finite PSD or moment-residual
check is a diagnostic. It does not prove the infinite exact family.

## Row-specific semantic obligations

Generic prose cannot complete a card. The visible body of each card (HTML
comments excluded) must contain its theorem-specific motivation, boundary
case, history, ML analogy, and worked calculation below. CFT-35-006 must also
spell out all eight comparison axes in the final column.

| Item | Motivation | Boundary | History | ML analogy | Worked content |
| --- | --- | --- | --- | --- | --- |
| CFT-34-001 | build the concrete dilation record | zero polynomial q = 0 | Lorist--Schwenninger boundary realization | lifted feature-space realization | finite atomic boundary model: for i = Fin m with atom weights w_a, the boundary integral becomes ∑ a, w_a • D_a |
| CFT-34-002 | identify the compressed first boundary moment | constant multiplier h = 0 | first compression moment | feature covariance compression | two-atom compression calculation: V* M_h V = w₀ V₀* h₀ V₀ + w₁ V₁* h₁ V₁ |
| CFT-34-003 | upgrade the first moment to every power | power k = 0 | power-moment compression | multi-step rollout moments | empirical moment residual R_k := V* M_h^k V - Φ(h^k), calculated as ‖R_k‖ for each sampled k |
| CFT-34-004 | turn multiplier products into powers | base case k = 0 | boundary multiplier algebra | tied linear-layer composition | worked k = 2 multiplication: M_h^2 f = h • (h • f) = h^2 • f |
| CFT-34-005 | certify contraction before dilation | strict contraction ‖h‖ < 1 gives ‖M_h‖ < 1 | contractive boundary multiplier | Lipschitz certificate | worked bound ‖h‖ = ρ < 1 gives ‖M_h f‖ ≤ ρ ‖f‖ |
| CFT-34-006 | feed the realization into the Chapter 33 endpoint | q = 0 gives operator norm zero | factor-two realization endpoint | robust operator-norm certificate | nonnormal 2 × 2 worked matrix: evaluate q(B), then compare its norm with the sampled boundary maximum |
| CFT-35-001 | remove the normalization and approximation scaffolding | zero polynomial makes both sides zero | main polynomial consequence | nonnormal layer functional bound | worked polynomial p(z) = z reduces the conclusion to ‖A‖ ≤ 2 max_{z∈W(A)} ‖z‖ |
| CFT-35-002 | specialize the type-parametric theorem to Fin d | d = 0 is excluded by Nonempty (Fin d) | finite-matrix consequence adapter | width-indexed matrix theorem | worked dimension d = 2 instantiates SquareMatrix (Fin 2) without changing the constant |
| CFT-35-003 | extend polynomial control to pole-free rational functions | an embedded polynomial is a rational boundary case | rational approximation consequence | resolvent-filter certificate | one-pole rational approximant sequence q_N converges uniformly before q_N(A) and the maxima pass to their limits |
| CFT-35-004 | transport finite matrices to bounded Hilbert-space operators | finite-dimensional H recovers the matrix theorem | Hilbert-space transport consequence | infinite-width operator limit | finite-rank compression calculation compares P_d A P_d with its Fin m matrix model |
| CFT-35-005 | package compactness, spectral containment, and rational control | a scalar operator makes all three components explicit | closed numerical-range spectral-set consequence | three-part stability certificate | worked compactness-spectrum-bound triple checks the compact set, spectrum inclusion, and rational inequality separately |
| CFT-35-006 | compare three independently checked terminal routes | propositional agreement is not proof-term identity | three-route comparison, not a fourth proof | proof-architecture ablation | worked three-column comparison row; Objects.; Hypotheses.; Shared trunk.; Decisive mechanism.; Approximation order.; Conclusion.; Provenance.; Formal provider. |

## Exact Lean map

The baseline hash is the maintained coverage hash. The compiler-public hash
is independently computed from the normalized elaborated public type; the
provider hash comes from the same private compiler probe. Direct aliases can
therefore have a compiler-public hash equal to the provider hash while still
retaining a distinct baseline coverage hash. Every compiler public row is
unique and binds the exact source path, normalized type and hash, declaration
kind, standard axiom set (`Classical.choice`, `Quot.sound`, `propext`), active
mode, provider dependency, and direct dependency set. A provider is not
inferred from a matching name. Completed reexports retain their provider.
CFT-35-006 changes to the proved-here
`CrouzeixTextbook.Part06.three_route_terminal_bundle` and has no
`underlying_declaration`.

| Item | Public declaration / public file | Provider declaration / provider file | Baseline coverage SHA-256 | Compiler public kind / SHA-256 | Provider SHA-256 |
| --- | --- | --- | --- | --- | --- |
| CFT-34-001 | `CrouzeixTextbook.Part06.boundary_dilation_data` / `formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean` | `CrouzeixConjecture.LoristSchwenninger.dilationDataOfParametricPolynomial` / `formalization/lean/Crouzeix/LoristSchwenninger/ConcreteDilation.lean` | `6a6fc2824a8faaa4dc9cd85fb9fe476145818054e33772c18188d7a5d253e612` | `direct-alias` / `b8dabd71a831f9daaa373f3d7938aa9bc00087592d81975e00a97061af17cb7e` | `b8dabd71a831f9daaa373f3d7938aa9bc00087592d81975e00a97061af17cb7e` |
| CFT-34-002 | `CrouzeixTextbook.Part06.boundary_compression_first_moment` / `formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean` | `CrouzeixConjecture.LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_comp_boundaryEmbedding` / `formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean` | `c239874c55c1aed80e22529c2a780b3a0728e561c65e677a28505ac45511965e` | `direct-alias` / `2a414f51546bd96dd1afcd5d0bbe93fea39684dff0c35843da4b85b9282c1e14` | `2a414f51546bd96dd1afcd5d0bbe93fea39684dff0c35843da4b85b9282c1e14` |
| CFT-34-003 | `CrouzeixTextbook.Part06.boundary_compression_power_moments` / `formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean` | `CrouzeixConjecture.LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding` / `formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean` | `fd451caedfa1808e5ce7a824522d7fb5a613eb3add9e988b47d1d851b5ceee55` | `direct-alias` / `4fb3020149c49a8b8a0a126673e03400b99e295b8bab79a60ea8adfcffafcc73` | `4fb3020149c49a8b8a0a126673e03400b99e295b8bab79a60ea8adfcffafcc73` |
| CFT-34-004 | `CrouzeixTextbook.Part06.boundary_multiplier_powers` / `formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean` | `CrouzeixConjecture.LoristSchwenninger.bcfMulL_pow` / `formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean` | `f723bad3e2f1b0cfed8531b208a8de1492201fdf88439726d83fcb8166a738c3` | `direct-alias` / `735335a245a1a4b3bac89546d203b2b856f97029284167c36259e5f6fbc805e3` | `735335a245a1a4b3bac89546d203b2b856f97029284167c36259e5f6fbc805e3` |
| CFT-34-005 | `CrouzeixTextbook.Part06.boundary_multiplier_contractive` / `formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean` | `CrouzeixConjecture.LoristSchwenninger.bcfMulL_norm_le_one` / `formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean` | `20864fd4143c209116e93458a00e0f812db607dd0c84a5bea02cbe9334155c37` | `direct-alias` / `1ae87992f1e7a9b66a79358fcafa7247061979381845fc4132e07b5768ce67f1` | `1ae87992f1e7a9b66a79358fcafa7247061979381845fc4132e07b5768ce67f1` |
| CFT-34-006 | `CrouzeixTextbook.Part06.realization_norm_two` / `formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean` | `CrouzeixConjecture.LoristSchwenninger.norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary` / `formalization/lean/Crouzeix/LoristSchwenninger/ConcreteDilation.lean` | `5726b7d2562ac08d0a520d8ccc5440f9a72b6f9aa978ac2da5c723a552ef4596` | `direct-alias` / `84ab420a345d504769f8df1067adfc0424dad4b18d2e58e4ef3d4c71ebc3654f` | `84ab420a345d504769f8df1067adfc0424dad4b18d2e58e4ef3d4c71ebc3654f` |
| CFT-35-001 | `CrouzeixTextbook.Part06.lorist_schwenninger_main` / `formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean` | `CrouzeixConjecture.loristSchwenningerMainTheorem` / `formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean` | `eaa8ca24d59d173f34972e80bdb20acf20d1c78178f2171ac28bbc6688a1960b` | `direct-alias` / `eaa8ca24d59d173f34972e80bdb20acf20d1c78178f2171ac28bbc6688a1960b` | `eaa8ca24d59d173f34972e80bdb20acf20d1c78178f2171ac28bbc6688a1960b` |
| CFT-35-002 | `CrouzeixTextbook.Part06.lorist_schwenninger_finite_matrix` / `formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean` | `CrouzeixConjecture.loristSchwenningerFiniteMatrixMainTheorem` / `formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean` | `606baa75394270e977e5ddb6e843205dd71e60ae005c408ea5a71884c1907776` | `direct-alias` / `606baa75394270e977e5ddb6e843205dd71e60ae005c408ea5a71884c1907776` | `606baa75394270e977e5ddb6e843205dd71e60ae005c408ea5a71884c1907776` |
| CFT-35-003 | `CrouzeixTextbook.Part06.lorist_schwenninger_rational` / `formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean` | `CrouzeixConjecture.loristSchwenningerRationalSpectralSetCorollary` / `formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean` | `a68683384ff3984804bccbfaf7f5dea8ca6128d26a0618984bc5d293837f0144` | `direct-alias` / `a68683384ff3984804bccbfaf7f5dea8ca6128d26a0618984bc5d293837f0144` | `a68683384ff3984804bccbfaf7f5dea8ca6128d26a0618984bc5d293837f0144` |
| CFT-35-004 | `CrouzeixTextbook.Part06.lorist_schwenninger_hilbert` / `formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean` | `CrouzeixConjecture.loristSchwenningerHilbertSpacePolynomialCrouzeix` / `formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean` | `d93d63c731ea1723a81d71d06845c000a240f960e67c6f3114079e64fff4627a` | `direct-alias` / `ff3d2b377c9cc61c89686022d9e9146cf351ae90293d4036a9e12c4a2b2cc17d` | `b2990956880277aaea8314c1874db2c9f863c1304c088cd5f62577235355e1d6` |
| CFT-35-005 | `CrouzeixTextbook.Part06.lorist_schwenninger_two_spectral_set` / `formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean` | `CrouzeixConjecture.loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet` / `formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean` | `d12d0a2f2deb6ce66f803a1ca394b125f2c16d3756570d5b63ab701e933dad77` | `direct-alias` / `391ee68292160554f5ba1ad54719f4bdab6266357b767a99ff690a17ad111904` | `13586fecd9adb5d6e1b01ef8e051521822eb1900c7a8c035d2cfa8cb7ab5957e` |
| CFT-35-006 | `CrouzeixTextbook.Part06.jin_final_comparator` / `formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean` | `CrouzeixConjecture.jinFinalCrouzeixConjecture` / `formalization/lean/CrouzeixConjecture/FinalTheorems.lean` | `0c2cdfce0642e8d4ab88dc860b6a60e85be855897e5f63fd51babbf67c87fc5d` | `direct-alias` / `5a08b37106dc806e9cb5f69cb03f5e13bd850055d67cb410ad615924faaab9c2` | `5a08b37106dc806e9cb5f69cb03f5e13bd850055d67cb410ad615924faaab9c2` |

The six existing Chapter 35 names remain compatibility declarations:
`lorist_schwenninger_main`, `lorist_schwenninger_finite_matrix`,
`lorist_schwenninger_rational`, `lorist_schwenninger_hilbert`,
`lorist_schwenninger_two_spectral_set`, and `jin_final_comparator`. The checked
bundle body must depend on all three terminal declarations. In full names they
are `CrouzeixConjecture.jinFinalCrouzeixConjecture`,
`CrouzeixConjecture.loristSchwenningerMainTheorem`, and
`CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem`. A conjunction filled
from assumptions or a wrapper around only one provider fails the receipt
audit.

In the completed phase, the compiler-normalized type hash of
`CrouzeixTextbook.Part06.three_route_terminal_bundle` is
`5ce10ccd8b2e5f6a0a93932256d6df60556b450817f4324eb4bac53cd444962b`.
The bundle uses `n : Type`, matching the certified Harp provider. Every finite
matrix size is represented by `Fin d`; the exercise below only combines given
hypotheses and retains its generalized universe.
The compiler-normalized type hash of
`CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_06_solution` is
`a6b5302087094c1fd6c8378f297d6ff79a995904d3e4a6be961a7fe5f3d8cdb7`.

## Exercise signatures and proof boundaries

The raw block `WAVE3_PROBE_DECLARATIONS` in
`crates/harp/tests/support/crouzeix_textbook_wave3_contract.rs` is the durable
signature source. The test writes it to a private temporary Lean file. The
file imports `CrouzeixTextbook.ExportReceipt` and `CrouzeixHarp`, but no project
module imports the temporary file. Its axioms therefore do not enter the
textbook library. The compiler supplies normalized types and hashes. Every
target type is distinct from its parent card's public and provider type; an
exercise cannot satisfy the contract by renaming the parent theorem.
For CFT-34-E06, the compiler-normalized target type must contain
`CrouzeixConjecture.LoristSchwenninger.dilationDataOfParametricPolynomial`.
That constructor is a signature dependency, not a proof-body dependency; the
proof-only closure is computed after subtracting the signature dependencies.

| Exercise | Parent | Probe declaration | Future solution | Allowed CFT dependencies | Required proof-body dependencies |
| --- | --- | --- | --- | --- | --- |
| CFT-34-E01 | CFT-34-001 | `CrouzeixTextbook.Wave3ContractProbe.cft_34_e01` | `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_01_solution` | none | `CrouzeixConjecture.LoristSchwenninger.norm_boundaryEmbeddingToLp` |
| CFT-34-E02 | CFT-34-002 | `CrouzeixTextbook.Wave3ContractProbe.cft_34_e02` | `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_02_solution` | CFT-34-001 | `CrouzeixConjecture.LoristSchwenninger.boundaryEmbeddingField_memLp`; `CrouzeixConjecture.LoristSchwenninger.bcfMulL_apply_ae`; `CrouzeixConjecture.LoristSchwenninger.boundarySquareRoot_mul_self_ae`; `CrouzeixConjecture.boundaryPhiCLM_apply` |
| CFT-34-E03 | CFT-34-002 | `CrouzeixTextbook.Wave3ContractProbe.cft_34_e03` | `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_03_solution` | none | `circleIntegral.integral_sub_inv_of_mem_ball`; `circleIntegral.integral_sub_zpow_of_ne` |
| CFT-34-E04 | CFT-34-003 | `CrouzeixTextbook.Wave3ContractProbe.cft_34_e04` | `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_04_solution` | CFT-34-002, CFT-34-004 | `CrouzeixConjecture.LoristSchwenninger.bcfMulL_pow`; `CrouzeixConjecture.LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_comp_boundaryEmbedding` |
| CFT-34-E05 | CFT-34-005 | `CrouzeixTextbook.Wave3ContractProbe.cft_34_e05` | `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_05_solution` | CFT-34-004 | `CrouzeixConjecture.LoristSchwenninger.bcfMulL_norm_le` |
| CFT-34-E06 | CFT-34-006 | `CrouzeixTextbook.Wave3ContractProbe.cft_34_e06` | `CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_06_solution` | CFT-33-006, CFT-34-001 | `CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two` |
| CFT-35-E01 | CFT-35-001 | `CrouzeixTextbook.Wave3ContractProbe.cft_35_e01` | `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_01_solution` | CFT-34-006 | none beyond the hypothesis |
| CFT-35-E02 | CFT-35-002 | `CrouzeixTextbook.Wave3ContractProbe.cft_35_e02` | `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_02_solution` | CFT-35-001 | none beyond the hypothesis |
| CFT-35-E03 | CFT-35-003 | `CrouzeixTextbook.Wave3ContractProbe.cft_35_e03` | `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_03_solution` | CFT-35-001 | `CrouzeixConjecture.rationalSpectralSetCorollary_of_mainTheorem` |
| CFT-35-E04 | CFT-35-004 | `CrouzeixTextbook.Wave3ContractProbe.cft_35_e04` | `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_04_solution` | CFT-35-002 | `CrouzeixConjecture.hilbertSpacePolynomialCrouzeix_of_mainTheorem` |
| CFT-35-E05 | CFT-35-005 | `CrouzeixTextbook.Wave3ContractProbe.cft_35_e05` | `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_05_solution` | CFT-35-003, CFT-35-004 | `CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem` |
| CFT-35-E06 | CFT-35-006 | `CrouzeixTextbook.Wave3ContractProbe.cft_35_e06` | `CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_06_solution` | none | from three supplied route conclusions and three supplied normalization equalities, transport every route proof to derive three witnesses of `PolynomialCrouzeixBound A p`, combined as `PolynomialCrouzeixBound A p ∧ PolynomialCrouzeixBound A p ∧ PolynomialCrouzeixBound A p` |

Each completed chapter namespace has exactly six theorem declarations,
including the private-name audit. Helpers inside the namespace, including
compiler-generated private names that point to a shortcut, fail. Every
solution must match its probe type and may use only the listed CFT rows plus
foundations. The receipt validator subtracts dependencies contributed by the
signature before auditing proof-body dependencies. Direct and helper-hidden
parent providers are forbidden. Jin, LS,
and Harp terminal declarations are forbidden in exercises. CFT-35-E06 accepts
three separately supplied propositions, three equations normalizing them to
the same polynomial Crouzeix bound, and one proof of each proposition. It
must transport each route proof through its corresponding equality and return
three normalized witnesses of `PolynomialCrouzeixBound A p`. Thus the direct
pairing `⟨hJin, hLs, hHarp⟩` is ill typed; an ephemeral negative compiler
fixture freezes that rejection while the positive target is itself elaborated
as a theorem. Neither target imports a terminal theorem. A reflexive `P = P`,
a tautological `True`, or a parent theorem is a rejected substitute.

Every completed solution has exactly the standard compiler axiom set
`Classical.choice`, `Quot.sound`, and `propext`, in both metadata and the
compiled receipt. Probe axioms are confined to the unimported temporary test
module and are never accepted as project solution axioms.

The two Mathlib names required by CFT-34-E03 are audited separately from the
project dependency graph. The ephemeral compiler probe starts from the
solution's elaborated proof body, follows proof-body references transitively
through maintained helpers, and intersects the result with the exact external
allowlist in the table. It does not scan source text: comments and type-only
mentions cannot satisfy the requirement. Allowlisted external lemmas are
terminal leaves, so the audit does not recursively import Mathlib's internal
dependency graph. A compiled negative fixture freezes both properties: the
probe follows a maintained helper to the inverse-mode lemma while rejecting a
comment that merely names the missing non-`-1`-mode lemma.

The completed moment-phase targets are intentionally different. CFT-34-E03
proves the scalar unit-circle identities `∮ z⁻¹ dz = 2πi` and
`∮ z^n dz = 0` for `n ≠ -1`, then uses finite-sum linearity to prove the
coefficient-selection corollary
`∮ ∑_{m=0}^N a_m z^(m-1) dz = a_0(2πi)`; it does not import a compression
theorem.
CFT-34-E04 proves the power-compression identity after evaluation at one
vector. Its proof body must use `bcfMulL_pow` and the first-moment compression
provider and must not use the parent
`boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding`.

## Row-specific provenance

Each future card carries its own classification and exact locator. These
labels are scoped per row; in particular, `Harp-derived` belongs only to the
CFT-35-006 comparison context.

| Item | Classification | Exact locator |
| --- | --- | --- |
| CFT-34-001 | LS source-derived | arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124 |
| CFT-34-002 | LS source-derived | arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124 |
| CFT-34-003 | LS source-derived | arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124 |
| CFT-34-004 | LS source-derived | arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124 |
| CFT-34-005 | LS source-derived | arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124 |
| CFT-34-006 | LS source-derived | arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124 |
| CFT-35-001 | LS source-derived terminal | arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128 |
| CFT-35-002 | provider-clean consequence | formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L20-L24 |
| CFT-35-003 | provider-clean consequence | formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L28-L33 |
| CFT-35-004 | provider-clean consequence | formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L37-L45 |
| CFT-35-005 | provider-clean consequence | formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L55-L61 |
| CFT-35-006 | comparison-only; Harp-derived component | formalization/lean/Crouzeix/Harp/MainTheorem.lean#L25-L172 |

## Sources and provenance

The LS source identity is `arxiv:2608.03841v1`. Exact source spans are:

- `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99`
- `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L74-L90`
- `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98`
- `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124`
- `arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128`

The authorities are
`evidence/crouzeix_conjecture/source_manifest.tsv#L25-L28` and
`labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json#L1`.
The captured TeX digest is
`20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a`.

The Jin comparison provider is pinned to
`git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910`.
Within the CFT-35-006 comparison only, the Harp component is derived rather
than source-faithful. Its local provenance locators are
`formalization/lean/Crouzeix/Harp/MainTheorem.lean#L25` and
`formalization/lean/Crouzeix/Harp/Consequences.lean#L22-L61`. These locators
do not assert priority, publication, or independence from reused lower-level
LS modules.

## Pedagogical and provider graphs

Chapter 34 reaches CFT-33-006 and common machinery. Chapter 34 must never
reach CFT-30-001 through CFT-32-006. CFT-35-006 is the first and only
three-route comparison row. CFT-35-001 through CFT-35-005 remain LS
consequences. Terminal theorem references in the comparison table are labels,
not pedagogical prerequisites among the proof routes.

The canonical nested-comment-aware import parser reports the following active
local closures. The digest hashes the sorted module names joined by newlines,
without a trailing newline.

| Route | Modules | Roster SHA-256 |
| --- | ---: | --- |
| `jin` | 66 | `35d570c92d791bc0162b8f9b19b6450e52c4e80226d13fb885efc1874b91dca5` |
| `lorist-schwenninger` | 56 | `63c7b93484759a68c41bb30e86ac7c2957cfccbef37933bc059dee3baaa46f53` |
| `harp` | 59 | `db41d5e011d149397a5fb6bb6b33c2a7e70d3ec07a43e5193a7159605808f903` |

Tests obtain these rosters from `proof_evidence.py preflight --route all` and
reject direct or transitive undocumented imports. Names in prose and comments
do not count as imports. Compilation remains a separate gate:
`CrouzeixJin`, `CrouzeixLoristSchwenninger`, `CrouzeixHarp`, and
`CrouzeixTextbook` must all build against the pinned cache.

## Exit rule

Advance a phase only when its exact row metadata, ten-field cards, equations,
exercise signatures, exercise dependency closure, source locators, and
compiler paths pass together. The final phase additionally requires the
three-provider bundle receipt to show all three terminal dependencies. No
phase change can be inferred from prose counts, declaration names, or an
aggregate build alone.
