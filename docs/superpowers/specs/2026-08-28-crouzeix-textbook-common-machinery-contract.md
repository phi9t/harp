# Crouzeix textbook common-machinery contract

**Status:** Wave 4 common trunk independently reviewed and published

**Date:** 2026-08-29

**Scope:** Chapters 25--29 and their two independent pedagogical handoffs

## Phase

- Active phase: `wave-4-complete`.
- Target phase: `wave-4-complete`.
- Current exact correspondence rows: `30`.
- Current distinct checked exercise solutions: `30`.
- Current coverage truth: Chapters 25--29 have six compiler-validated exact
  correspondences and six distinct checked exercise solutions each. CFT-28-006
  remains an explicitly deferred Jin-route summary even though its Lean
  correspondence is exact. Chapter 29 separates its proved universal upper
  provider from the independently reconstructed scaled sharpness witness.
- Chapters 25--29 are promoted by this phase.

The allowed phase order is:

1. `contract-frozen`
2. `chapter-25-complete`
3. `chapter-26-complete`
4. `chapter-27-complete`
5. `chapter-28-complete`
6. `chapter-29-complete`
7. `wave-4-complete`

A chapter phase means that all six theorem cards and all six exercises in the
canonical chapter Markdown satisfy this document. Synthetic card strings are
parser tests only and are never promotion evidence. A phase label alone proves
nothing. Promotion requires a fresh compiler receipt, complete prose, exact
contract rows, and the focused tests. In particular, changing the phase while
leaving the current summary chapter untouched fails on canonical-card content,
not merely on status labels.
`wave-4-complete` adds independent mathematical and formal review plus the
aggregate publication gate.

## Independent Wave 4 review record

The release review reconstructed the load-bearing arguments in Chapters
25--29, compared all 30 public declarations and 30 exercise solutions with
fresh compiler receipts, checked the common import closure, and inspected the
rendered publication. The reader graph remains acyclic. Its Jin and
Lorist--Schwenninger seeds depend on the common trunk rather than on each
other. Harp's finite-horizon route may reuse lower-level
Lorist--Schwenninger modules, so this review does not claim three-way kernel
independence.

Review result: no blocking mathematical, formal-correspondence, or publication findings.
This release interpretation does not change CFT-28-006 from its truthful
`summary` status. That card has exact Lean correspondence for its displayed
conditional statement, but the explanation of why the completion mechanism
forces the constant two remains an explicit Jin-route preview.

## Canonical chapter roster

- `part_05_crouzeix_machinery/25_convex_boundaries_and_cauchy_layers.md`
- `part_05_crouzeix_machinery/26_double_layer_map.md`
- `part_05_crouzeix_machinery/27_one_plus_sqrt_two_barrier.md`
- `part_05_crouzeix_machinery/28_complete_power_family.md`
- `part_06_constant_two_routes/29_crouzeix_problem_and_sharpness.md`

The repository-root versions of these paths live under
`knowledge/crouzeix_textbook/`. Their 30 CFT identifiers, anchors, public Lean
names, and 30 exercise identifiers are stable.

## Complete theorem-card shape

At a completed chapter phase, each CFT card has exactly one stable anchor and
substantive sections named:

1. `Purpose`
2. `Definitions and notation`
3. `Statement`
4. `Hypothesis ledger`
5. `Proof roadmap`
6. `Proof`
7. `Worked instance`
8. `Boundary case`
9. `Historical context`
10. `ML analogy`
11. `Pedagogical prerequisites`
12. `Lean correspondence`
13. `Exercises and solutions`

The 13 headings occur exactly once, in the order above, and are bounded by that
card's heading and the next CFT or chapter heading. The proof contains the
row-specific obligation and displays every load-bearing equality, inequality,
limit, and case split. Reflexive placeholders such as `x=x` are not proof
equations. A heading followed by generic filler does not satisfy the field.
The worked instance and boundary case must test the card's mechanism rather
than merely repeat its statement. Historical context carries a source and
review-status boundary. The ML analogy states the mathematical object and ML
counterpart, exact transfer, non-transfer, and a concrete diagnostic.

The Lean correspondence section records the public name, substantive provider
when applicable, readable type map, repository-root code link,
compiler-reported source line, normalized type, type fingerprint, direct
maintained dependencies, axioms, assumptions, target, and receipt identity.
Those values are compared literally with the fresh public compiler receipt;
labels or invented values do not pass. Existing
formal modes remain stable unless the implementation proves a reason to change
them; `checkpoint` does not become exact merely because its declaration
compiles.

## Frozen theorem map

The locator and hash columns below describe the compiler-validated state at
`contract-frozen`. Later chapter tasks update a row only from a fresh receipt.

| Item | Anchor | Public Lean declaration | Current code locator | Current type SHA-256 | Required mathematical work |
| --- | --- | --- | --- | --- | --- |
| CFT-25-001 | cft-25-001 | `CrouzeixTextbook.Part05.convex_projection` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L14) | `4ffef37c950b39279846aeb028511a0d8f18a945e1d8888379d849d8096a6100` | prove closed-convex nearest-point existence and separate existence from uniqueness |
| CFT-25-002 | cft-25-002 | `CrouzeixTextbook.Part05.convex_projection_variational` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L47) | `e5ff9225c14b24a0bc391ce2d438e1f6d80e2f0e1fe4075becb75d5ac2fe160a` | differentiate squared distance along a convex segment and handle the endpoint |
| CFT-25-003 | cft-25-003 | `CrouzeixTextbook.Part05.convex_projection_nonexpansive` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L56) | `6491d411e634da59fa43f86b95c536274c4f53474e95382ada98b49cc76f6fc5` | add both variational inequalities and apply Cauchy--Schwarz |
| CFT-25-004 | cft-25-004 | `CrouzeixTextbook.Part05.outer_approximation_radius` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L65) | `b2294ee6be90f9259c5d55fcb89f02182b9ba7feeeb8124179e51fabf7056ce8` | define the parallel outer radius and its geometric containment data |
| CFT-25-005 | cft-25-005 | `CrouzeixTextbook.Part05.outer_radius_tends_to_zero` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L71) | `1f2d7a4b929e101df48496466ae11483b7aa7f7328ffe49b5c0d95c34335b000` | prove outer-radius decay with compactness and uniformity explicit |
| CFT-25-006 | cft-25-006 | `CrouzeixTextbook.Part05.parallel_outer_domain_data` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean#L114) | `e0c090a879dce8bda6dbcd5297ac2597904ac6646e0fff4ebdae2e33e4f846b9` | construct the positively oriented regular radial C¹ boundary package consumed by Cauchy integration |
| CFT-26-001 | cft-26-001 | `CrouzeixTextbook.Part05.support_point_outside_numerical_range` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L15) | `c81aa6c33f155d60b3ea289c9683b19f669b9b37f5b0fa0e1afbe8be12f42086` | prove boundary exclusion together with unit-vector scalar half-plane separation from numerical-range containment |
| CFT-26-002 | cft-26-002 | `CrouzeixTextbook.Part05.support_resolvent_invertible` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L29) | `ba925bdd2b8af209941855dc71402f2fc0d602598e38bb205a1282955c8e5866` | contradict a kernel vector using strict separation; isolate the boundary limit |
| CFT-26-003 | cft-26-003 | `CrouzeixTextbook.Part05.double_layer_support_positive` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L33) | `669c3561a5e2fa2329dbac180de86d6b8ac4f22a91c3d0ab2b827e9570dbd577` | prove support-resolvent positivity by an explicit quadratic-form congruence |
| CFT-26-004 | cft-26-004 | `CrouzeixTextbook.Part05.double_layer_resolvent` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L37) | `2576ccb833b38fce237c0923d56df69609bf656489d5f61e0ab3427e4486ec09` | define the oriented normalized double-layer resolvent from scalar Cauchy data |
| CFT-26-005 | cft-26-005 | `CrouzeixTextbook.Part05.double_layer_congruence` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L41) | `3ca440cf9415bc7043af96c8f7c9292238e56809c47f535ee2f896ccbb9006b1` | write the matrix congruence in full, including adjoints and scalar factors |
| CFT-26-006 | cft-26-006 | `CrouzeixTextbook.Part05.double_layer_density_positive` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L46) | `acfa9886104dc38842086a82b8fe579c01f2b5ebf959a35159b37f1b76cea4b4` | integrate pointwise positivity with integrability and finite-sum interchange justified |
| CFT-27-001 | cft-27-001 | `CrouzeixTextbook.Part05.sqrt_two_nonnegative` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L12) | `0671f6cc626c2d1ec60ba1496a2a0bcfcea215f114a3712a2e6ad0333943c82b` | establish the nonnegative real square-root branch |
| CFT-27-002 | cft-27-002 | `CrouzeixTextbook.Part05.sqrt_two_squared` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L16) | `807b03a281f82bdc472d967959ea405a451a428770ceb42484d1cd5bd3c5c85c` | prove the square-root specification at two |
| CFT-27-003 | cft-27-003 | `CrouzeixTextbook.Part05.one_plus_sqrt_two_positive` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L20) | `b5d6ab35facd3ff402611c5653d77b68002e38c6a2303ed396fd26262b35ec94` | exclude the negative root with the sign branch visible |
| CFT-27-004 | cft-27-004 | `CrouzeixTextbook.Part05.one_plus_sqrt_two_barrier` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L31) | `b34b7fab7e6f0711708e126020eace13d85ec177642c7a3d3c3172f0dc9bc8f5` | factor the scalar quadratic and audit both factor signs |
| CFT-27-005 | cft-27-005 | `CrouzeixTextbook.Part05.triangle_barrier_kernel` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L46) | `4ee8f501079eaa15f32b005c52c5275ec658b9f32af686f89de532b09a890aeb` | derive the independent-term triangle estimate and expose its equality case |
| CFT-27-006 | cft-27-006 | `CrouzeixTextbook.Part05.positive_map_norm_kernel` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean#L57) | `ff5adddf717c380cb4f3b3123f60e9e8569e51e49c18c3d4d3c8de299b91935e` | derive the positive-map quadratic barrier and identify the discarded cross term |
| CFT-28-001 | cft-28-001 | `CrouzeixTextbook.Part05.power_cauchy_formula` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L11) | `258dcd40effc3e1c87075f12ed8e7b33594385efb646067ebd90517af2e3238f` | state the power-indexed Cauchy formula with domain, orientation, and analyticity |
| CFT-28-002 | cft-28-002 | `CrouzeixTextbook.Part05.power_cauchy_mass_one` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L35) | `af7ffea07579ffc73817d257712c98a60596ed2a7df38403d75cee67224d96dd` | apply scalar Cauchy to the constant function and track normalization |
| CFT-28-003 | cft-28-003 | `CrouzeixTextbook.Part05.contractive_boundary_function` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L50) | `89cd7ac55f8eef42107167ebd2bc6310d339d9a3a319ed58ca3ceba31f31e907` | define normalized boundary data and prove its supremum norm is at most one |
| CFT-28-004 | cft-28-004 | `CrouzeixTextbook.Part05.power_cayley_companion` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L68) | `b8f10ac0cc5db54f484a749ec7b303f346a9d9803d1a9bab62062bbefc2099dc` | construct every Cayley companion power and prove the positivity identity |
| CFT-28-005 | cft-28-005 | `CrouzeixTextbook.Part05.positive_completion_from_power_family` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L102) | `655425bab338ed249ed27b699808957c6fddbfb80f2a31fdfa72e65852a6ccb0` | assemble pointwise positivity into a positive-real completion for all powers |
| CFT-28-006 | cft-28-006 | `CrouzeixTextbook.Part05.power_family_norm_two` | [code](../../../formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean#L129) | `a505f972d0a99bda91d63ea153feeadbea85c3112034679470c5e48b4da9f7bc` | assemble the common completion and preview the deferred Jin completion-to-two consequence |
| CFT-29-001 | cft-29-001 | `CrouzeixTextbook.Part06.max_polynomial_modulus` | [code](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L150) | `3799044d1d7cf216e1d5a16d6f176195a40d2f60105cb22cc17066c56df6e3f4` | define the closed numerical-range maximum and prove existence by compactness |
| CFT-29-002 | cft-29-002 | `CrouzeixTextbook.Part06.polynomial_crouzeix_bound` | [code](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L162) | `4d98441cfca7d869d873c43a1e6794f5127c362be399d6291ef85f13f0fb6707` | state the quantified polynomial bound and its zero-maximum edge case |
| CFT-29-003 | cft-29-003 | `CrouzeixTextbook.Part06.main_theorem_statement` | [code](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L180) | `c500bddcbe4f0b4927ef40b3effb463212129a1744caca97cd171c2e30a863bd` | separate finite-matrix, rational spectral-set, and Hilbert-space theorem surfaces |
| CFT-29-004 | cft-29-004 | `CrouzeixTextbook.Part06.jordan_two_maximum` | [code](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L188) | `4fcca18fce47dfeafad4133214ea02a50d523e160c2cca7be86a383beef98641` | compute the unit-disk numerical range of the scaled two-by-two Jordan block |
| CFT-29-005 | cft-29-005 | `CrouzeixTextbook.Part06.jordan_two_attains_two` | [code](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L197) | `4cf9d3a9c2a267d26959ef5ae89a54abaf2ab714d3fabc2a5ea4e930c55cb7cd` | evaluate the identity polynomial and calculate norm two against maximum one |
| CFT-29-006 | cft-29-006 | `CrouzeixTextbook.Part06.constant_two_is_least` | [code](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L213) | `e4e8201aa2ea2f299dac20b63ca70ac72b1e0dfb1ffae78c75485eb6d88f4eed` | combine the universal lower bound only with a separately identified upper bound |

Frozen kind correction: `CFT-27-001` is a theorem. Its public Lean declaration
proves a proposition and must not be published as a definition.

### CFT-25-006 target boundary

The frozen public checkpoint `parallel_outer_domain_data` currently exposes
`ConvexOuterApproximationData`: exact containment and distance data for a
parallel body. It does **not** prove that a raw parallel body has a C∞ boundary,
and the chapter must not say that it does. The completion object is the
positively oriented radial C¹ boundary package proved by
`CrouzeixConjecture.orientedRadialConvexBoundary_thickening` (and assembled by
`CrouzeixConjecture.canonicalParallelOrientedRadialBoundaryStatement` with type
`CanonicalParallelOrientedRadialBoundaryStatement`). A completed CFT-25-006
keeps its stable textbook public name, but that name must compile as a theorem
with the same normalized-type fingerprint as the canonical statement and must
have the canonical theorem among its direct maintained dependencies. The raw
`parallelOuterDomain_data` definition cannot be promoted. This is the regularity,
orientation, containment, and limiting package consumed by the later Cauchy
reduction. A future separate smoothing construction would require its own CFT
row and statement; it is not silently folded into CFT-25-006.

## Exercise target map

Each completed chapter keeps complete prose solutions for all six exercises.
Every formal solution compiles under its stable name, answers the actual
prompt, has a statement fingerprint different from every public theorem card
and every other exercise globally, and must compile as a theorem, never as a
direct or eta alias. The canonical exercise metadata is joined to a fresh
compiler receipt and must exactly match its declaration, source path, line,
column, type SHA-256, axioms, and `CrouzeixTextbook` target. The receipt also
supplies the normalized type, direct maintained dependencies, and
compiler-derived proof-body dependencies. Its source
coordinate must resolve to the named declaration in the repository file.
The receipt's normalized-type fingerprint must equal the independently
Lean-elaborated planned proposition below; a novel but unrelated theorem is not
evidence. The compiler-reported axioms must equal the approved set
`Classical.choice`, `Quot.sound`, and `propext` independently of copied metadata.

Each exercise block contains a `Complete written solution` section. Its proof
may use Mathlib and strictly earlier common CFT results. Each row has an
explicit set of reconstruction-target providers independent of its indexed
parent: an earlier theorem is still forbidden when it is the result that the
exercise reconstructs. A solution may not directly or transitively invoke its
parent public theorem, any reconstruction target, a same-or-later common CFT
result, the substantive provider behind any same-or-later common CFT row, or any Jin,
Lorist--Schwenninger, or Harp terminal provider. Provider checks recursively
close compiler-reported proof-body constants; constants occurring only in
mathematical statement types are allowlisted, so shared objects such as
`SquareMatrix` are not mistaken for proof shortcuts. All 30 rows require distinct
checked declarations; no interpretive-row exception is implicit.

Every planned type also carries row-specific semantic fragments. The contract
rejects missing named objects, hypotheses, or conclusions, as well as `True`,
identity implications, synthetic witness surrogates, and definition-unfolding
equivalences when the prompt promises a theorem. CFT-28-E01 and the explicit
`MainTheoremStatement` restatement in CFT-29-E06 are definition-surface
exercises and are explicitly allowed to expose their defining equivalences.

| Exercise / parent | Prompt/task | Expected reconstruction | Expository statement shape | Stable solution and reconstruction targets |
| --- | --- | --- | --- | --- |
| CFT-25-E01 / CFT-25-001 | Prove existence of a nearest point in a nonempty closed convex subset of the complex plane. | use closedness to obtain completeness, construct a minimizer, and prove its minimizing inequality | `∃ p∈K, ∀ w∈K, ‖z-p‖≤‖z-w‖` | `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_01_solution`; no reconstruction shortcut or maintained target provider |
| CFT-25-E02 / CFT-25-002 | Compute metric projection onto a closed disk. | split inside/outside cases and derive the exact radial formula from the variational inequality | `P_closedBall(r,x)=x if ‖x‖≤r, and P_closedBall(r,x)=(r/‖x‖)x otherwise` | `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_02_solution`; target `CrouzeixConjecture.convexProjection_variational` |
| CFT-25-E03 / CFT-25-003 | Prove the projection variational inequalities imply uniqueness. | test each candidate against the other, add the real-inner-product inequalities, and force zero distance | `VI(x,p,q) ∧ VI(x,q,p) → p=q` | `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_03_solution`; target `CrouzeixConjecture.eq_convexProjection_of_mem_of_variational` |
| CFT-25-E04 / CFT-25-004 | Derive nonexpansiveness from two variational inequalities. | add the two projection inequalities and close with Cauchy--Schwarz, treating the zero-distance case | `VI(x,Px,Py) ∧ VI(y,Py,Px) → ‖Px-Py‖ ≤ ‖x-y‖` | `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_04_solution`; targets `CrouzeixConjecture.convexProjection_firm_nonexpansive`, `CrouzeixConjecture.norm_convexProjection_sub_le` |
| CFT-25-E05 / CFT-25-005 | Show why the natural constant-speed polygonal parametrization is incompatible with the positive-speed regular radial C¹ package. | exhibit unequal one-sided complex tangents, prove nondifferentiability of the constant-speed corner model, and distinguish degenerate C¹ parametrizations that pause | `v≠w → ¬ DifferentiableAt ℝ (piecewiseLinearVertex v w) 0` | `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_05_solution`; no maintained target provider |
| CFT-25-E06 / CFT-25-006 | Formalize and prove the oriented radial C¹ package required for every canonical parallel outer domain. | quantify over the numerical range and parallel-body index and produce the oriented radial boundary package consumed by Cauchy integration | `∀ A k, HasOrientedRadialConvexBoundary (parallelOuterDomain (numericalRange A) k)` | `CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_06_solution`; targets `CrouzeixConjecture.canonicalParallelOrientedRadialBoundaryStatement`, `CrouzeixConjecture.exists_orientedRadialConvexBoundary_thickening` |
| CFT-26-E01 / CFT-26-001 | Write the outward support inequality for W(B). | transport numerical-range containment into the scalar real-part inequality at every unit vector | `OutwardBoundarySupport Ω σ ν → W(B)⊆Ω → ‖x‖=1 → 0≤re(ν̄(σ-⟪x,Bx⟫))` | `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_01_solution`; no maintained target provider |
| CFT-26-E02 / CFT-26-002 | Prove unit-vector nonnegativity of the support quadratic form. | expand the support quadratic form, insert the geometric support inequality, and obtain nonnegativity on every unit vector; Hermitianity and zero/nonzero rescaling are the separate upgrade to PSD | `support geometry → W(B)⊆Ω → ‖x‖=1 → 0≤re⟪S(B,σ,ν)x,x⟫` | `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_02_solution`; targets `CrouzeixConjecture.doubleLayerSupportMatrix_reApplyInnerSelf_eq_two`, `CrouzeixConjecture.doubleLayerSupportMatrix_posSemidef_of_outwardBoundarySupport` |
| CFT-26-E03 / CFT-26-003 | Prove matrix congruence preserves positive semidefiniteness. | establish Hermitianity and rewrite the quadratic form of BᴴMB as the quadratic form of M at Bx | `M.PosSemidef → (Bᴴ*M*B).PosSemidef` | `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_03_solution`; targets `CrouzeixConjecture.posSemidef_congruence`, `Matrix.PosSemidef.conjTranspose_mul_mul_same` |
| CFT-26-E04 / CFT-26-004 | Expand the double-layer resolvent congruence algebraically. | use the right-inverse identity, its adjoint, and all scalar factors to identify the double-layer density | `(σI-B)R=I → Rᴴ*supportMatrix(B,σ,ν)*R=doubleLayerDensity(R,ν)` | `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_04_solution`; targets `CrouzeixConjecture.resolvent_conjTranspose_leftInverse`, `CrouzeixConjecture.doubleLayer_congruence_density_identity` |
| CFT-26-E05 / CFT-26-005 | Explain why a supported boundary point gives resolvent invertibility. | turn a hypothetical spectral/kernel witness into a numerical-range witness and contradict strict containment | `support geometry → W(B)⊆Ω → IsUnit(σI-B)` | `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_05_solution`; targets `CrouzeixConjecture.OutwardBoundarySupport.sigma_not_mem_numericalRange`, `CrouzeixConjecture.scalar_sub_matrix_isUnit_of_outwardBoundarySupport` |
| CFT-26-E06 / CFT-26-006 | Reconstruct positivity of the double-layer resolvent density. | compose support positivity, resolvent inversion, the exact congruence, and congruence preservation without calling the terminal density theorem | `support geometry → W(B)⊆Ω → PosSemidef(doubleLayerDensity(resolvent B σ,ν))` | `CrouzeixTextbook.Part05.Exercises.Chapter26.exercise_06_solution`; targets `CrouzeixConjecture.doubleLayerDensity_posSemidef`, `CrouzeixConjecture.doubleLayerResolvent_density_posSemidef` |
| CFT-27-E01 / CFT-27-001 | Solve κ²≤2κ+1 under the operator-norm hypothesis κ≥0. | factor at 1±√2, determine the factor signs, and derive the upper bound | `0≤κ → κ²≤2κ+1 → κ≤1+√2` | `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_01_solution`; target `CrouzeixTextbook.Part05.one_plus_sqrt_two_barrier` |
| CFT-27-E02 / CFT-27-002 | Numerically compare 1+√2 with 2 and 5/2. | square positive comparisons and obtain 2<1+√2<5/2 | `2<1+√2 ∧ 1+√2<5/2` | `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_02_solution`; target `CrouzeixTextbook.Part05.sqrt_two_squared` |
| CFT-27-E03 / CFT-27-003 | Prove the quadratic barrier by completing the square. | rewrite as (κ-1)²≤2 and use the nonnegative square-root branch to derive the upper bound | `κ²≤2κ+1 → (κ-1)²≤2 ∧ κ≤1+√2` | `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_03_solution`; target `CrouzeixTextbook.Part05.one_plus_sqrt_two_barrier` |
| CFT-27-E04 / CFT-27-004 | Show the triangle inequality can attain 1+√2. | construct aligned complex summands with norms 1 and √2 and verify equality in the triangle inequality | `∃ a b:ℂ, ‖a‖=1 ∧ ‖b‖=√2 ∧ ‖a+b‖=1+√2` | `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_04_solution`; no maintained target provider |
| CFT-27-E05 / CFT-27-005 | Give a structural condition improving the triangle estimate for vectors in a complex inner-product space. | assume inner-product orthogonality and replace the triangle bound by the Pythagorean norm identity | `⟪x,y⟫=0 → ‖x+y‖²=‖x‖²+‖y‖²` | `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_05_solution`; no maintained target provider |
| CFT-27-E06 / CFT-27-006 | Show the scalar upper barrier does not require κ≥0, and locate where nonnegativity enters the operator application. | factor the quadratic at 1±√2 and derive κ≤1+√2 without a sign hypothesis; prove separately that κ≥0 when κ is instantiated as an operator norm | `κ²≤2κ+1 → factored inequality ∧ κ≤1+√2 ∧ (κ=‖T‖→0≤κ)` | `CrouzeixTextbook.Part05.Exercises.Chapter27.exercise_06_solution`; target `CrouzeixTextbook.Part05.one_plus_sqrt_two_barrier` |
| CFT-28-E01 / CFT-28-001 | Define a common-contour power Cauchy family. | bind one contour, measure, boundary function, matrix, and target to a formula valid for every natural power | `HasParametricPowerCauchyFormula Γ μ B f T ↔ ∀m, ∫ f(x)^m·firstPart(Γ,B,x)dμ=T^m` | `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_01_solution`; definition row, no theorem target |
| CFT-28-E02 / CFT-28-002 | Prove pointwise contractivity of every power of a contractive boundary function. | use norm multiplicativity and monotonicity from ‖f(x)‖≤1 to every natural power | `ContractiveBoundaryFunction f → ∀m x, ‖f(x)^m‖≤1` | `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_02_solution`; no maintained target provider |
| CFT-28-E03 / CFT-28-003 | Derive the scalar Cayley power series. | expand the named Cayley transform geometrically and justify convergence when ‖zw‖<1 | `‖zw‖<1 → cayleyTransform z w=1+2∑'(zw)^(m+1)` | `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_03_solution`; no maintained target provider |
| CFT-28-E04 / CFT-28-004 | Derive mass one from the zeroth power Cauchy identity. | specialize the simultaneous common-contour formula at m=0 and simplify both zeroth powers | `HasParametricPowerCauchyFormula Γ μ B f T → ∫firstPart=I` | `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_04_solution`; target `CrouzeixConjecture.HasParametricPowerCauchyFormula.mass_eq_one` |
| CFT-28-E05 / CFT-28-005 | Compare the common-contour package with its indexed identities and mass normalization. | extract every indexed identity and the zeroth-power mass identity while keeping the shared contour and measure explicit | `HasParametricPowerCauchyFormula → (∀m, power identity) ∧ mass=I` | `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_05_solution`; target `CrouzeixConjecture.HasParametricPowerCauchyFormula.mass_eq_one` |
| CFT-28-E06 / CFT-28-006 | Assemble the deferred Jin completion-to-two consequence with every hypothesis explicit. | derive spectral containment, construct the common positive-real completion, and call the named Jin completion-to-two provider | `power Cauchy + W(B)⊆Ω + diagonalization + ‖λj‖≤1 → ‖T‖≤2` | `CrouzeixTextbook.Part05.Exercises.Chapter28.exercise_06_solution`; calls `CrouzeixConjecture.positiveRealCompletionStatement` as a deferred Jin input and still forbids the wrapper shortcuts `CrouzeixConjecture.exists_positiveRealCompletion_of_parametricPowerCauchy`, `CrouzeixConjecture.norm_le_two_of_parametricPowerCauchy` |
| CFT-29-E01 / CFT-29-001 | Prove the zero-maximum edge case of the polynomial Crouzeix bound. | unpack the matrix polynomial bound and use norm definiteness to show p(A)=0 when the numerical-range maximum is zero | `PolynomialCrouzeixBound A p → maxOnW(A,p)=0 → polynomialEval p A=0` | `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_01_solution`; no maintained target provider |
| CFT-29-E02 / CFT-29-002 | Compute W(J) for the 2×2 nilpotent Jordan block. | parametrize a unit vector, bound the coordinate product, and realize every phase to prove exact disk equality | `numericalRange jordanNilpotentTwo=closedBall(0,1/2)` | `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_02_solution`; no maintained target provider |
| CFT-29-E03 / CFT-29-003 | Prove the sharp ratio two for p(z)=z on the Jordan block. | combine ‖p(J)‖=1 with the exact numerical-range maximum 1/2 and perform the division | `‖polynomialEval X J‖/maxOnW(J,X)=2` | `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_03_solution`; target `CrouzeixConjecture.jordanNilpotentTwo_attains_two` |
| CFT-29-E04 / CFT-29-004 | Prove the polynomial numerical-range constant is one for a normal matrix. | use the normality equation and unitary diagonalization to bound ‖p(A)‖ by the numerical-range maximum | `AAᴴ=AᴴA → ‖p(A)‖≤maxOnW(A,p)` | `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_04_solution`; no maintained target provider |
| CFT-29-E05 / CFT-29-005 | Carry out the M>0 polynomial normalization. | set q=M⁻¹p, record maxOnW(q)=1, apply the normalized bound, and rescale the actual matrix polynomial evaluation | `M>0 → maxOnW(p)=M → maxOnW(M⁻¹p)=1 → ‖(M⁻¹p)(A)‖≤2 → ‖p(A)‖≤2M` | `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_05_solution`; no maintained target provider |
| CFT-29-E06 / CFT-29-006 | Restate MainTheoremStatement with every finite-index assumption and its full conclusion explicit. | quantify the index type, Fintype, DecidableEq, Nonempty, matrix, and polynomial and prove that this expanded surface is exactly MainTheoremStatement | `∀n [Fintype n] [DecidableEq n] [Nonempty n], MainTheoremStatement ↔ ∀A p, PolynomialCrouzeixBound A p` | `CrouzeixTextbook.Part06.Exercises.Chapter29.exercise_06_solution`; no maintained target provider |

### Compiler-comparable planned propositions

The expository shapes above communicate the intended full textbook exercise.
Promotion is mechanically bound to the following Lean-elaborated proposition
for each solution. The contract test elaborates all 30 types in the pinned
toolchain and compares their normalized-type SHA-256 values with the fresh
solution receipt, avoiding dependence on pretty-printer context.

| Exercise | Exact Lean proposition elaborated for the fingerprint contract |
| --- | --- |
| CFT-25-E01 | `∀ (K : Set ℂ) (hKne : K.Nonempty) (hKclosed : IsClosed K) (hKconvex : Convex ℝ K) (z : ℂ), ∃ p : ℂ, p ∈ K ∧ ∀ w ∈ K, ‖z - p‖ ≤ ‖z - w‖` |
| CFT-25-E02 | `∀ (r : ℝ) (hr : 0 < r) (x : ℂ) (hne : (Metric.closedBall (0 : ℂ) r).Nonempty) (hcompact : IsCompact (Metric.closedBall (0 : ℂ) r)) (hconvex : Convex ℝ (Metric.closedBall (0 : ℂ) r)), CrouzeixConjecture.convexProjection (Metric.closedBall (0 : ℂ) r) hne hcompact hconvex x = if ‖x‖ ≤ r then x else ((r / ‖x‖ : ℝ) : ℂ) * x` |
| CFT-25-E03 | `∀ (x p q : ℂ), ⟪x - p, q - p⟫_ℝ ≤ 0 → ⟪x - q, p - q⟫_ℝ ≤ 0 → p = q` |
| CFT-25-E04 | `∀ (x y px py : ℂ), ⟪x - px, py - px⟫_ℝ ≤ 0 → ⟪y - py, px - py⟫_ℝ ≤ 0 → ‖px - py‖ ≤ ‖x - y‖` |
| CFT-25-E05 | `∀ (v w : ℂ), v ≠ w → ¬ DifferentiableAt ℝ (fun t : ℝ => if t ≤ 0 then t • v else t • w) 0` |
| CFT-25-E06 | `∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n], CrouzeixConjecture.CanonicalParallelOrientedRadialBoundaryStatement (n := n)` |
| CFT-26-E01 | `∀ (n : Type) [Fintype n] [DecidableEq n] {Omega : Set ℂ} {sigma nu : ℂ} (hgeom : CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu) (B : CrouzeixConjecture.SquareMatrix n) (hWB : CrouzeixConjecture.numericalRange B ⊆ Omega) (x : CrouzeixConjecture.EuclideanVector n), ‖x‖ = 1 → 0 ≤ RCLike.re (star nu * (sigma - ⟪x, CrouzeixConjecture.euclideanOperator B x⟫_ℂ))` |
| CFT-26-E02 | `∀ (n : Type) [Fintype n] [DecidableEq n] {Omega : Set ℂ} {sigma nu : ℂ} (hgeom : CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu) (B : CrouzeixConjecture.SquareMatrix n) (hWB : CrouzeixConjecture.numericalRange B ⊆ Omega) (x : CrouzeixConjecture.EuclideanVector n), ‖x‖ = 1 → 0 ≤ (CrouzeixConjecture.euclideanOperator (CrouzeixConjecture.doubleLayerSupportMatrix B sigma nu)).reApplyInnerSelf x` |
| CFT-26-E03 | `∀ (n : Type) [Fintype n] [DecidableEq n] (M B : CrouzeixConjecture.SquareMatrix n), M.PosSemidef → (Bᴴ * M * B).PosSemidef` |
| CFT-26-E04 | `∀ (n : Type) [Fintype n] [DecidableEq n] (B R : CrouzeixConjecture.SquareMatrix n) (sigma nu : ℂ), (sigma • (1 : CrouzeixConjecture.SquareMatrix n) - B) * R = 1 → Rᴴ * CrouzeixConjecture.doubleLayerSupportMatrix B sigma nu * R = CrouzeixConjecture.doubleLayerDensity R nu` |
| CFT-26-E05 | `∀ (n : Type) [Fintype n] [DecidableEq n] {Omega : Set ℂ} {sigma nu : ℂ} (hgeom : CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu) (B : CrouzeixConjecture.SquareMatrix n), CrouzeixConjecture.numericalRange B ⊆ Omega → IsUnit (sigma • (1 : CrouzeixConjecture.SquareMatrix n) - B)` |
| CFT-26-E06 | `∀ (n : Type) [Fintype n] [DecidableEq n] {Omega : Set ℂ} {sigma nu : ℂ} (hgeom : CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu) (B : CrouzeixConjecture.SquareMatrix n), CrouzeixConjecture.numericalRange B ⊆ Omega → (CrouzeixConjecture.doubleLayerDensity (CrouzeixConjecture.doubleLayerResolvent B sigma) nu).PosSemidef` |
| CFT-27-E01 | `∀ κ : ℝ, 0 ≤ κ → κ ^ 2 ≤ 2 * κ + 1 → κ ≤ 1 + Real.sqrt 2` |
| CFT-27-E02 | `2 < (1 : ℝ) + Real.sqrt 2 ∧ (1 : ℝ) + Real.sqrt 2 < 5 / 2` |
| CFT-27-E03 | `∀ κ : ℝ, κ ^ 2 ≤ 2 * κ + 1 → (κ - 1) ^ 2 ≤ 2 ∧ κ ≤ 1 + Real.sqrt 2` |
| CFT-27-E04 | `∃ a b : ℂ, ‖a‖ = 1 ∧ ‖b‖ = Real.sqrt 2 ∧ ‖a + b‖ = 1 + Real.sqrt 2` |
| CFT-27-E05 | `∀ (E : Type) [NormedAddCommGroup E] [InnerProductSpace ℂ E] (x y : E), ⟪x, y⟫_ℂ = 0 → ‖x + y‖ ^ 2 = ‖x‖ ^ 2 + ‖y‖ ^ 2` |
| CFT-27-E06 | `∀ κ : ℝ, κ ^ 2 ≤ 2 * κ + 1 → (κ - (1 + Real.sqrt 2)) * (κ - (1 - Real.sqrt 2)) ≤ 0 ∧ κ ≤ 1 + Real.sqrt 2 ∧ ∀ (E : Type) [NormedAddCommGroup E] [InnerProductSpace ℂ E] (T : E →L[ℂ] E), κ = ‖T‖ → 0 ≤ κ` |
| CFT-28-E01 | `∀ (i n : Type) [TopologicalSpace i] [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Fintype n] [DecidableEq n] [Nonempty n] (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ) (Gamma : CrouzeixConjecture.ParametricConvexBoundary (i := i) Omega) (B : CrouzeixConjecture.SquareMatrix n) (f : CrouzeixConjecture.ContractiveBoundaryFunction i) (T : CrouzeixConjecture.SquareMatrix n), CrouzeixConjecture.HasParametricPowerCauchyFormula Gamma mu B f T ↔ ∀ m : ℕ, ∫ x, (f.function x) ^ m • CrouzeixConjecture.parametricBoundaryFirstPart Gamma B x ∂mu = T ^ m` |
| CFT-28-E02 | `∀ (i : Type) [TopologicalSpace i] (f : CrouzeixConjecture.ContractiveBoundaryFunction i), ∀ m : ℕ, ∀ x : i, ‖f.function x ^ m‖ ≤ 1` |
| CFT-28-E03 | `∀ z w : ℂ, ‖z * w‖ < 1 → CrouzeixConjecture.cayleyTransform z w = 1 + 2 * ∑' m : ℕ, (z * w) ^ (m + 1)` |
| CFT-28-E04 | `∀ (i n : Type) [TopologicalSpace i] [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Fintype n] [DecidableEq n] [Nonempty n] (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ) (Gamma : CrouzeixConjecture.ParametricConvexBoundary (i := i) Omega) (B : CrouzeixConjecture.SquareMatrix n) (f : CrouzeixConjecture.ContractiveBoundaryFunction i) (T : CrouzeixConjecture.SquareMatrix n), CrouzeixConjecture.HasParametricPowerCauchyFormula Gamma mu B f T → ∫ x, CrouzeixConjecture.parametricBoundaryFirstPart Gamma B x ∂mu = (1 : CrouzeixConjecture.SquareMatrix n)` |
| CFT-28-E05 | `∀ (i n : Type) [TopologicalSpace i] [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Fintype n] [DecidableEq n] [Nonempty n] (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ) (Gamma : CrouzeixConjecture.ParametricConvexBoundary (i := i) Omega) (B : CrouzeixConjecture.SquareMatrix n) (f : CrouzeixConjecture.ContractiveBoundaryFunction i) (T : CrouzeixConjecture.SquareMatrix n), CrouzeixConjecture.HasParametricPowerCauchyFormula Gamma mu B f T → (∀ m : ℕ, ∫ x, (f.function x) ^ m • CrouzeixConjecture.parametricBoundaryFirstPart Gamma B x ∂mu = T ^ m) ∧ ∫ x, CrouzeixConjecture.parametricBoundaryFirstPart Gamma B x ∂mu = (1 : CrouzeixConjecture.SquareMatrix n)` |
| CFT-28-E06 | `∀ (i n : Type) [TopologicalSpace i] [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Fintype n] [DecidableEq n] [Nonempty n] (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ) (Gamma : CrouzeixConjecture.ParametricConvexBoundary (i := i) Omega) (B T : CrouzeixConjecture.SquareMatrix n) (hWB : CrouzeixConjecture.numericalRange B ⊆ Omega) (f : CrouzeixConjecture.ContractiveBoundaryFunction i), CrouzeixConjecture.HasParametricPowerCauchyFormula Gamma mu B f T → ∀ (hB : CrouzeixConjecture.SimpleDiagonalization B) (lambda : n → ℂ), T = CrouzeixConjecture.innerConjugation hB.changeBasis (Matrix.diagonal lambda) → (∀ j, ‖lambda j‖ ≤ 1) → ‖T‖ ≤ 2` |
| CFT-29-E01 | `∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n] (A : CrouzeixConjecture.SquareMatrix n) (p : Polynomial ℂ), CrouzeixConjecture.PolynomialCrouzeixBound A p → CrouzeixConjecture.maxPolynomialModulusOnNumericalRange A p = 0 → CrouzeixConjecture.polynomialEval p A = 0` |
| CFT-29-E02 | `CrouzeixConjecture.numericalRange CrouzeixConjecture.jordanNilpotentTwo = Metric.closedBall 0 ((1 : ℝ) / 2)` |
| CFT-29-E03 | `‖CrouzeixConjecture.polynomialEval Polynomial.X CrouzeixConjecture.jordanNilpotentTwo‖ / CrouzeixConjecture.maxPolynomialModulusOnNumericalRange CrouzeixConjecture.jordanNilpotentTwo Polynomial.X = 2` |
| CFT-29-E04 | `∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n] (A : CrouzeixConjecture.SquareMatrix n) (p : Polynomial ℂ), A * Aᴴ = Aᴴ * A → ‖CrouzeixConjecture.polynomialEval p A‖ ≤ CrouzeixConjecture.maxPolynomialModulusOnNumericalRange A p` |
| CFT-29-E05 | `∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n] (A : CrouzeixConjecture.SquareMatrix n) (p : Polynomial ℂ) (M : ℝ), 0 < M → CrouzeixConjecture.maxPolynomialModulusOnNumericalRange A p = M → CrouzeixConjecture.maxPolynomialModulusOnNumericalRange A (((M⁻¹ : ℝ) : ℂ) • p) = 1 → ‖CrouzeixConjecture.polynomialEval (((M⁻¹ : ℝ) : ℂ) • p) A‖ ≤ 2 → ‖CrouzeixConjecture.polynomialEval p A‖ ≤ 2 * M` |
| CFT-29-E06 | `∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n], CrouzeixConjecture.MainTheoremStatement (n := n) ↔ ∀ (A : CrouzeixConjecture.SquareMatrix n) (p : Polynomial ℂ), CrouzeixConjecture.PolynomialCrouzeixBound A p` |

## Pedagogical graph and branch boundary

The entire 210-node pedagogical graph must remain acyclic. A reconstructible
row may depend only on its own or an earlier chapter. A summary preview may
carry an explicit forward reader link to the later proof it defers. The sole
Wave 4 exception is CFT-28-006, which points to CFT-30-002 and CFT-32-001;
these are pedagogical links, not Lean import or kernel edges. The terminal
routes otherwise begin from common nodes, not from each other:

| Branch seed | Exact direct pedagogical prerequisites |
| --- | --- |
| CFT-30-001 | CFT-28-005, CFT-29-002 |
| CFT-33-001 | CFT-23-004, CFT-23-005, CFT-29-002 |

The Jin branch is CFT-30 through CFT-32. The Lorist--Schwenninger branch is
CFT-33 through CFT-34. Before Chapter 35 compares conclusions, neither seed
may depend on the other branch, directly or transitively. Comparison prose is
not a pedagogical or kernel edge.
The kernel graph remains compiler-derived and is never inferred from this
reader graph.

## Maintained common Lean closure

The common closure has five roots:

- `CrouzeixTextbook.Part05.Chapter25`
- `CrouzeixTextbook.Part05.Chapter26`
- `CrouzeixTextbook.Part05.Chapter27`
- `CrouzeixTextbook.Part05.Chapter28`
- `CrouzeixTextbook.Part06.Chapter29`

It may import maintained `CrouzeixConjecture` common modules. It must import
neither `Crouzeix.Jin`, `Crouzeix.LoristSchwenninger`, nor `Crouzeix.Harp`,
and it must not import the corresponding aggregate provider modules.

| Closure | Modules | Sorted-module SHA-256 |
| --- | ---: | --- |
| common-machinery | 66 | `f3b82cd7a98454f1ff6d388914e5dde343593fda5f2f71c87b3a8ed4886f344b` |

This is a structural source receipt, not a theorem receipt. The tests use the
canonical comment-aware import parser, reject direct and transitive provider
mutations, and separately compile the 30 public declarations to recover their
source positions, type hashes, and axioms.
The mutation matrix covers both direct and transitive imports of every dotted
namespace (`Crouzeix.Jin`, `Crouzeix.LoristSchwenninger`, `Crouzeix.Harp`) and
every aggregate module (`CrouzeixJin`, `CrouzeixLoristSchwenninger`,
`CrouzeixHarp`). Comment-only mentions of the same names remain accepted.

## Promotion and mutation rules

A chapter may advance only when all six real cards and exercise rows satisfy
the corresponding phase. The test suite rejects:

- changing only the phase label while prose remains summary or solutions are
  absent;
- missing or duplicated theorem anchors;
- a missing required theorem-card section;
- duplicated exercise solution names, a theorem-card hash reused as an
  exercise hash, or an absent compiler receipt;
- an edge from one source-derived route into the other or any pedagogical
  cycle;
- a direct or transitive terminal-provider import in the common Lean closure;
- stale source paths, line numbers, declaration names, type hashes, or axioms.

Publication artifacts are regenerated only by the Wave 4 integration task.
This contract-freeze task changes no Chapter 25--29 prose, Lean theorem, or
publication status.
