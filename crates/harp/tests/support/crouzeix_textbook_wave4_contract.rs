use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{array, contracts_root, packet_root, read_json, string, workspace_root};

const WAVE4_CONTRACT: &str =
    "docs/superpowers/specs/2026-08-28-crouzeix-textbook-common-machinery-contract.md";
const AXIOMS: [&str; 3] = ["Classical.choice", "Quot.sound", "propext"];
const COMMON_ROOTS: [&str; 5] = [
    "CrouzeixTextbook.Part05.Chapter25",
    "CrouzeixTextbook.Part05.Chapter26",
    "CrouzeixTextbook.Part05.Chapter27",
    "CrouzeixTextbook.Part05.Chapter28",
    "CrouzeixTextbook.Part06.Chapter29",
];
const COMMON_CLOSURE_COUNT: usize = 66;
const COMMON_CLOSURE_SHA256: &str =
    "f3b82cd7a98454f1ff6d388914e5dde343593fda5f2f71c87b3a8ed4886f344b";
const CARD_FIELDS: [&str; 13] = [
    "Purpose",
    "Definitions and notation",
    "Statement",
    "Hypothesis ledger",
    "Proof roadmap",
    "Proof",
    "Worked instance",
    "Boundary case",
    "Historical context",
    "ML analogy",
    "Pedagogical prerequisites",
    "Lean correspondence",
    "Exercises and solutions",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Wave4Phase {
    ContractFrozen,
    Chapter25Complete,
    Chapter26Complete,
    Chapter27Complete,
    Chapter28Complete,
    Chapter29Complete,
    Wave4Complete,
}

impl Wave4Phase {
    const ALL: [Self; 7] = [
        Self::ContractFrozen,
        Self::Chapter25Complete,
        Self::Chapter26Complete,
        Self::Chapter27Complete,
        Self::Chapter28Complete,
        Self::Chapter29Complete,
        Self::Wave4Complete,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::ContractFrozen => "contract-frozen",
            Self::Chapter25Complete => "chapter-25-complete",
            Self::Chapter26Complete => "chapter-26-complete",
            Self::Chapter27Complete => "chapter-27-complete",
            Self::Chapter28Complete => "chapter-28-complete",
            Self::Chapter29Complete => "chapter-29-complete",
            Self::Wave4Complete => "wave-4-complete",
        }
    }

    fn parse(label: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|phase| phase.label() == label)
    }

    fn rank(self) -> usize {
        Self::ALL
            .iter()
            .position(|candidate| *candidate == self)
            .expect("phase rank")
    }

    fn completes(self, chapter: u64) -> bool {
        (25..=29).contains(&chapter) && self.rank() >= (chapter - 24) as usize
    }
}

#[derive(Clone, Copy)]
struct CommonSpec {
    item_id: &'static str,
    chapter: u64,
    anchor: &'static str,
    public_declaration: &'static str,
    source_path: &'static str,
    line: u64,
    type_sha256: &'static str,
    baseline_proof: &'static str,
    baseline_correspondence: &'static str,
    obligation: &'static str,
}

#[derive(Clone, Copy)]
struct ExerciseSpec {
    exercise_id: &'static str,
    parent: &'static str,
    prompt: &'static str,
    reconstruction: &'static str,
    statement_shape: &'static str,
    reconstruction_target_providers: &'static [&'static str],
    semantic_fragments: &'static [&'static str],
    permits_definition_unfolding: bool,
}

const COMMON_SPECS: [CommonSpec; 30] = [
    CommonSpec { item_id: "CFT-25-001", chapter: 25, anchor: "cft-25-001", public_declaration: "CrouzeixTextbook.Part05.convex_projection", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean", line: 14, type_sha256: "4ffef37c950b39279846aeb028511a0d8f18a945e1d8888379d849d8096a6100", baseline_proof: "summary", baseline_correspondence: "checkpoint", obligation: "prove closed-convex nearest-point existence and separate existence from uniqueness" },
    CommonSpec { item_id: "CFT-25-002", chapter: 25, anchor: "cft-25-002", public_declaration: "CrouzeixTextbook.Part05.convex_projection_variational", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean", line: 47, type_sha256: "e5ff9225c14b24a0bc391ce2d438e1f6d80e2f0e1fe4075becb75d5ac2fe160a", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "differentiate squared distance along a convex segment and handle the endpoint" },
    CommonSpec { item_id: "CFT-25-003", chapter: 25, anchor: "cft-25-003", public_declaration: "CrouzeixTextbook.Part05.convex_projection_nonexpansive", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean", line: 56, type_sha256: "6491d411e634da59fa43f86b95c536274c4f53474e95382ada98b49cc76f6fc5", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "add both variational inequalities and apply Cauchy--Schwarz" },
    CommonSpec { item_id: "CFT-25-004", chapter: 25, anchor: "cft-25-004", public_declaration: "CrouzeixTextbook.Part05.outer_approximation_radius", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean", line: 65, type_sha256: "b2294ee6be90f9259c5d55fcb89f02182b9ba7feeeb8124179e51fabf7056ce8", baseline_proof: "summary", baseline_correspondence: "checkpoint", obligation: "define the parallel outer radius and its geometric containment data" },
    CommonSpec { item_id: "CFT-25-005", chapter: 25, anchor: "cft-25-005", public_declaration: "CrouzeixTextbook.Part05.outer_radius_tends_to_zero", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean", line: 71, type_sha256: "1f2d7a4b929e101df48496466ae11483b7aa7f7328ffe49b5c0d95c34335b000", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "prove outer-radius decay with compactness and uniformity explicit" },
    CommonSpec { item_id: "CFT-25-006", chapter: 25, anchor: "cft-25-006", public_declaration: "CrouzeixTextbook.Part05.parallel_outer_domain_data", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter25.lean", line: 114, type_sha256: "e0c090a879dce8bda6dbcd5297ac2597904ac6646e0fff4ebdae2e33e4f846b9", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "construct the positively oriented regular radial C¹ boundary package consumed by Cauchy integration" },
    CommonSpec { item_id: "CFT-26-001", chapter: 26, anchor: "cft-26-001", public_declaration: "CrouzeixTextbook.Part05.support_point_outside_numerical_range", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean", line: 15, type_sha256: "c81aa6c33f155d60b3ea289c9683b19f669b9b37f5b0fa0e1afbe8be12f42086", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "prove boundary exclusion together with unit-vector scalar half-plane separation from numerical-range containment" },
    CommonSpec { item_id: "CFT-26-002", chapter: 26, anchor: "cft-26-002", public_declaration: "CrouzeixTextbook.Part05.support_resolvent_invertible", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean", line: 29, type_sha256: "ba925bdd2b8af209941855dc71402f2fc0d602598e38bb205a1282955c8e5866", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "contradict a kernel vector using strict separation; isolate the boundary limit" },
    CommonSpec { item_id: "CFT-26-003", chapter: 26, anchor: "cft-26-003", public_declaration: "CrouzeixTextbook.Part05.double_layer_support_positive", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean", line: 33, type_sha256: "669c3561a5e2fa2329dbac180de86d6b8ac4f22a91c3d0ab2b827e9570dbd577", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "prove support-resolvent positivity by an explicit quadratic-form congruence" },
    CommonSpec { item_id: "CFT-26-004", chapter: 26, anchor: "cft-26-004", public_declaration: "CrouzeixTextbook.Part05.double_layer_resolvent", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean", line: 37, type_sha256: "2576ccb833b38fce237c0923d56df69609bf656489d5f61e0ab3427e4486ec09", baseline_proof: "summary", baseline_correspondence: "checkpoint", obligation: "define the oriented normalized double-layer resolvent from scalar Cauchy data" },
    CommonSpec { item_id: "CFT-26-005", chapter: 26, anchor: "cft-26-005", public_declaration: "CrouzeixTextbook.Part05.double_layer_congruence", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean", line: 41, type_sha256: "3ca440cf9415bc7043af96c8f7c9292238e56809c47f535ee2f896ccbb9006b1", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "write the matrix congruence in full, including adjoints and scalar factors" },
    CommonSpec { item_id: "CFT-26-006", chapter: 26, anchor: "cft-26-006", public_declaration: "CrouzeixTextbook.Part05.double_layer_density_positive", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean", line: 46, type_sha256: "acfa9886104dc38842086a82b8fe579c01f2b5ebf959a35159b37f1b76cea4b4", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "integrate pointwise positivity with integrability and finite-sum interchange justified" },
    CommonSpec { item_id: "CFT-27-001", chapter: 27, anchor: "cft-27-001", public_declaration: "CrouzeixTextbook.Part05.sqrt_two_nonnegative", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean", line: 12, type_sha256: "0671f6cc626c2d1ec60ba1496a2a0bcfcea215f114a3712a2e6ad0333943c82b", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "establish the nonnegative real square-root branch" },
    CommonSpec { item_id: "CFT-27-002", chapter: 27, anchor: "cft-27-002", public_declaration: "CrouzeixTextbook.Part05.sqrt_two_squared", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean", line: 16, type_sha256: "807b03a281f82bdc472d967959ea405a451a428770ceb42484d1cd5bd3c5c85c", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "prove the square-root specification at two" },
    CommonSpec { item_id: "CFT-27-003", chapter: 27, anchor: "cft-27-003", public_declaration: "CrouzeixTextbook.Part05.one_plus_sqrt_two_positive", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean", line: 20, type_sha256: "b5d6ab35facd3ff402611c5653d77b68002e38c6a2303ed396fd26262b35ec94", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "exclude the negative root with the sign branch visible" },
    CommonSpec { item_id: "CFT-27-004", chapter: 27, anchor: "cft-27-004", public_declaration: "CrouzeixTextbook.Part05.one_plus_sqrt_two_barrier", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean", line: 31, type_sha256: "b34b7fab7e6f0711708e126020eace13d85ec177642c7a3d3c3172f0dc9bc8f5", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "factor the scalar quadratic and audit both factor signs" },
    CommonSpec { item_id: "CFT-27-005", chapter: 27, anchor: "cft-27-005", public_declaration: "CrouzeixTextbook.Part05.triangle_barrier_kernel", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean", line: 46, type_sha256: "4ee8f501079eaa15f32b005c52c5275ec658b9f32af686f89de532b09a890aeb", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "derive the independent-term triangle estimate and expose its equality case" },
    CommonSpec { item_id: "CFT-27-006", chapter: 27, anchor: "cft-27-006", public_declaration: "CrouzeixTextbook.Part05.positive_map_norm_kernel", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter27.lean", line: 57, type_sha256: "ff5adddf717c380cb4f3b3123f60e9e8569e51e49c18c3d4d3c8de299b91935e", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "derive the positive-map quadratic barrier and identify the discarded cross term" },
    CommonSpec { item_id: "CFT-28-001", chapter: 28, anchor: "cft-28-001", public_declaration: "CrouzeixTextbook.Part05.power_cauchy_formula", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean", line: 11, type_sha256: "258dcd40effc3e1c87075f12ed8e7b33594385efb646067ebd90517af2e3238f", baseline_proof: "summary", baseline_correspondence: "checkpoint", obligation: "state the power-indexed Cauchy formula with domain, orientation, and analyticity" },
    CommonSpec { item_id: "CFT-28-002", chapter: 28, anchor: "cft-28-002", public_declaration: "CrouzeixTextbook.Part05.power_cauchy_mass_one", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean", line: 35, type_sha256: "af7ffea07579ffc73817d257712c98a60596ed2a7df38403d75cee67224d96dd", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "apply scalar Cauchy to the constant function and track normalization" },
    CommonSpec { item_id: "CFT-28-003", chapter: 28, anchor: "cft-28-003", public_declaration: "CrouzeixTextbook.Part05.contractive_boundary_function", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean", line: 50, type_sha256: "89cd7ac55f8eef42107167ebd2bc6310d339d9a3a319ed58ca3ceba31f31e907", baseline_proof: "summary", baseline_correspondence: "checkpoint", obligation: "define normalized boundary data and prove its supremum norm is at most one" },
    CommonSpec { item_id: "CFT-28-004", chapter: 28, anchor: "cft-28-004", public_declaration: "CrouzeixTextbook.Part05.power_cayley_companion", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean", line: 68, type_sha256: "b8f10ac0cc5db54f484a749ec7b303f346a9d9803d1a9bab62062bbefc2099dc", baseline_proof: "summary", baseline_correspondence: "checkpoint", obligation: "construct every Cayley companion power and prove the positivity identity" },
    CommonSpec { item_id: "CFT-28-005", chapter: 28, anchor: "cft-28-005", public_declaration: "CrouzeixTextbook.Part05.positive_completion_from_power_family", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean", line: 102, type_sha256: "655425bab338ed249ed27b699808957c6fddbfb80f2a31fdfa72e65852a6ccb0", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "assemble pointwise positivity into a positive-real completion for all powers" },
    CommonSpec { item_id: "CFT-28-006", chapter: 28, anchor: "cft-28-006", public_declaration: "CrouzeixTextbook.Part05.power_family_norm_two", source_path: "formalization/lean/CrouzeixTextbook/Part05/Chapter28.lean", line: 129, type_sha256: "a505f972d0a99bda91d63ea153feeadbea85c3112034679470c5e48b4da9f7bc", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "assemble the common completion and preview the deferred Jin completion-to-two consequence" },
    CommonSpec { item_id: "CFT-29-001", chapter: 29, anchor: "cft-29-001", public_declaration: "CrouzeixTextbook.Part06.max_polynomial_modulus", source_path: "formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean", line: 150, type_sha256: "3799044d1d7cf216e1d5a16d6f176195a40d2f60105cb22cc17066c56df6e3f4", baseline_proof: "summary", baseline_correspondence: "checkpoint", obligation: "define the closed numerical-range maximum and prove existence by compactness" },
    CommonSpec { item_id: "CFT-29-002", chapter: 29, anchor: "cft-29-002", public_declaration: "CrouzeixTextbook.Part06.polynomial_crouzeix_bound", source_path: "formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean", line: 162, type_sha256: "4d98441cfca7d869d873c43a1e6794f5127c362be399d6291ef85f13f0fb6707", baseline_proof: "summary", baseline_correspondence: "checkpoint", obligation: "state the quantified polynomial bound and its zero-maximum edge case" },
    CommonSpec { item_id: "CFT-29-003", chapter: 29, anchor: "cft-29-003", public_declaration: "CrouzeixTextbook.Part06.main_theorem_statement", source_path: "formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean", line: 180, type_sha256: "c500bddcbe4f0b4927ef40b3effb463212129a1744caca97cd171c2e30a863bd", baseline_proof: "summary", baseline_correspondence: "checkpoint", obligation: "separate finite-matrix, rational spectral-set, and Hilbert-space theorem surfaces" },
    CommonSpec { item_id: "CFT-29-004", chapter: 29, anchor: "cft-29-004", public_declaration: "CrouzeixTextbook.Part06.jordan_two_maximum", source_path: "formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean", line: 188, type_sha256: "4fcca18fce47dfeafad4133214ea02a50d523e160c2cca7be86a383beef98641", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "compute the unit-disk numerical range of the scaled two-by-two Jordan block" },
    CommonSpec { item_id: "CFT-29-005", chapter: 29, anchor: "cft-29-005", public_declaration: "CrouzeixTextbook.Part06.jordan_two_attains_two", source_path: "formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean", line: 197, type_sha256: "4cf9d3a9c2a267d26959ef5ae89a54abaf2ab714d3fabc2a5ea4e930c55cb7cd", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "evaluate the identity polynomial and calculate norm two against maximum one" },
    CommonSpec { item_id: "CFT-29-006", chapter: 29, anchor: "cft-29-006", public_declaration: "CrouzeixTextbook.Part06.constant_two_is_least", source_path: "formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean", line: 213, type_sha256: "e4e8201aa2ea2f299dac20b63ca70ac72b1e0dfb1ffae78c75485eb6d88f4eed", baseline_proof: "summary", baseline_correspondence: "unmapped", obligation: "combine the universal lower bound only with a separately identified upper bound" },
];

const EXERCISE_SPECS: [ExerciseSpec; 30] = [
    ExerciseSpec { exercise_id: "CFT-25-E01", parent: "CFT-25-001", prompt: "Prove existence of a nearest point in a nonempty closed convex subset of the complex plane.", reconstruction: "use closedness to obtain completeness, construct a minimizer, and prove its minimizing inequality", statement_shape: "∃ p∈K, ∀ w∈K, ‖z-p‖≤‖z-w‖", reconstruction_target_providers: &[], semantic_fragments: &["(hKclosed : IsClosed K)", "∃ p : ℂ, p ∈ K ∧", "∀ w ∈ K, ‖z - p‖ ≤ ‖z - w‖"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-25-E02", parent: "CFT-25-002", prompt: "Compute metric projection onto a closed disk.", reconstruction: "split inside/outside cases and derive the exact radial formula from the variational inequality", statement_shape: "P_closedBall(r,x)=x if ‖x‖≤r, and P_closedBall(r,x)=(r/‖x‖)x otherwise", reconstruction_target_providers: &["CrouzeixConjecture.convexProjection_variational"], semantic_fragments: &["convexProjection", "Metric.closedBall", "if ‖x‖ ≤ r", "((r / ‖x‖ : ℝ) : ℂ) * x"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-25-E03", parent: "CFT-25-003", prompt: "Prove the projection variational inequalities imply uniqueness.", reconstruction: "test each candidate against the other, add the real-inner-product inequalities, and force zero distance", statement_shape: "VI(x,p,q) ∧ VI(x,q,p) → p=q", reconstruction_target_providers: &["CrouzeixConjecture.eq_convexProjection_of_mem_of_variational"], semantic_fragments: &["⟪x - p, q - p⟫_ℝ ≤ 0", "⟪x - q, p - q⟫_ℝ ≤ 0", "p = q"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-25-E04", parent: "CFT-25-004", prompt: "Derive nonexpansiveness from two variational inequalities.", reconstruction: "add the two projection inequalities and close with Cauchy--Schwarz, treating the zero-distance case", statement_shape: "VI(x,Px,Py) ∧ VI(y,Py,Px) → ‖Px-Py‖ ≤ ‖x-y‖", reconstruction_target_providers: &["CrouzeixConjecture.convexProjection_firm_nonexpansive", "CrouzeixConjecture.norm_convexProjection_sub_le"], semantic_fragments: &["⟪x - px, py - px⟫_ℝ ≤ 0", "⟪y - py, px - py⟫_ℝ ≤ 0", "‖px - py‖ ≤ ‖x - y‖"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-25-E05", parent: "CFT-25-005", prompt: "Show why the natural constant-speed polygonal parametrization is incompatible with the positive-speed regular radial C¹ package.", reconstruction: "exhibit unequal one-sided complex tangents, prove nondifferentiability of the constant-speed corner model, and distinguish degenerate C¹ parametrizations that pause", statement_shape: "v≠w → ¬ DifferentiableAt ℝ (piecewiseLinearVertex v w) 0", reconstruction_target_providers: &[], semantic_fragments: &["v w : ℂ", "v ≠ w", "DifferentiableAt ℝ", "if t ≤ 0"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-25-E06", parent: "CFT-25-006", prompt: "Formalize and prove the oriented radial C¹ package required for every canonical parallel outer domain.", reconstruction: "quantify over the numerical range and parallel-body index and produce the oriented radial boundary package consumed by Cauchy integration", statement_shape: "∀ A k, HasOrientedRadialConvexBoundary (parallelOuterDomain (numericalRange A) k)", reconstruction_target_providers: &["CrouzeixConjecture.canonicalParallelOrientedRadialBoundaryStatement", "CrouzeixConjecture.exists_orientedRadialConvexBoundary_thickening"], semantic_fragments: &["CanonicalParallelOrientedRadialBoundaryStatement", "[Nonempty n]"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-26-E01", parent: "CFT-26-001", prompt: "Write the outward support inequality for W(B).", reconstruction: "transport numerical-range containment into the scalar real-part inequality at every unit vector", statement_shape: "OutwardBoundarySupport Ω σ ν → W(B)⊆Ω → ‖x‖=1 → 0≤re(ν̄(σ-⟪x,Bx⟫))", reconstruction_target_providers: &[], semantic_fragments: &["OutwardBoundarySupport", "numericalRange B ⊆ Omega", "‖x‖ = 1", "RCLike.re (star nu * (sigma - ⟪x, CrouzeixConjecture.euclideanOperator B x⟫_ℂ))"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-26-E02", parent: "CFT-26-002", prompt: "Prove unit-vector nonnegativity of the support quadratic form.", reconstruction: "expand the support quadratic form, insert the geometric support inequality, and obtain nonnegativity on every unit vector; Hermitianity and zero/nonzero rescaling are the separate upgrade to PSD", statement_shape: "support geometry → W(B)⊆Ω → ‖x‖=1 → 0≤re⟪S(B,σ,ν)x,x⟫", reconstruction_target_providers: &["CrouzeixConjecture.doubleLayerSupportMatrix_reApplyInnerSelf_eq_two", "CrouzeixConjecture.doubleLayerSupportMatrix_posSemidef_of_outwardBoundarySupport"], semantic_fragments: &["doubleLayerSupportMatrix", "reApplyInnerSelf x", "OutwardBoundarySupport", "numericalRange B ⊆ Omega", "0 ≤"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-26-E03", parent: "CFT-26-003", prompt: "Prove matrix congruence preserves positive semidefiniteness.", reconstruction: "establish Hermitianity and rewrite the quadratic form of BᴴMB as the quadratic form of M at Bx", statement_shape: "M.PosSemidef → (Bᴴ*M*B).PosSemidef", reconstruction_target_providers: &["CrouzeixConjecture.posSemidef_congruence", "Matrix.PosSemidef.conjTranspose_mul_mul_same"], semantic_fragments: &["SquareMatrix", "M.PosSemidef", "(Bᴴ * M * B).PosSemidef"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-26-E04", parent: "CFT-26-004", prompt: "Expand the double-layer resolvent congruence algebraically.", reconstruction: "use the right-inverse identity, its adjoint, and all scalar factors to identify the double-layer density", statement_shape: "(σI-B)R=I → Rᴴ*supportMatrix(B,σ,ν)*R=doubleLayerDensity(R,ν)", reconstruction_target_providers: &["CrouzeixConjecture.resolvent_conjTranspose_leftInverse", "CrouzeixConjecture.doubleLayer_congruence_density_identity"], semantic_fragments: &["SquareMatrix", "(sigma • (1 : CrouzeixConjecture.SquareMatrix n) - B) * R = 1", "Rᴴ * CrouzeixConjecture.doubleLayerSupportMatrix", "CrouzeixConjecture.doubleLayerDensity R nu"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-26-E05", parent: "CFT-26-005", prompt: "Explain why a supported boundary point gives resolvent invertibility.", reconstruction: "turn a hypothetical spectral/kernel witness into a numerical-range witness and contradict strict containment", statement_shape: "support geometry → W(B)⊆Ω → IsUnit(σI-B)", reconstruction_target_providers: &["CrouzeixConjecture.OutwardBoundarySupport.sigma_not_mem_numericalRange", "CrouzeixConjecture.scalar_sub_matrix_isUnit_of_outwardBoundarySupport"], semantic_fragments: &["OutwardBoundarySupport", "numericalRange B ⊆ Omega", "IsUnit (sigma • (1 : CrouzeixConjecture.SquareMatrix n) - B)"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-26-E06", parent: "CFT-26-006", prompt: "Reconstruct positivity of the double-layer resolvent density.", reconstruction: "compose support positivity, resolvent inversion, the exact congruence, and congruence preservation without calling the terminal density theorem", statement_shape: "support geometry → W(B)⊆Ω → PosSemidef(doubleLayerDensity(resolvent B σ,ν))", reconstruction_target_providers: &["CrouzeixConjecture.doubleLayerDensity_posSemidef", "CrouzeixConjecture.doubleLayerResolvent_density_posSemidef"], semantic_fragments: &["OutwardBoundarySupport", "numericalRange B ⊆ Omega", "doubleLayerDensity (CrouzeixConjecture.doubleLayerResolvent B sigma) nu", ".PosSemidef"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-27-E01", parent: "CFT-27-001", prompt: "Solve κ²≤2κ+1 under the operator-norm hypothesis κ≥0.", reconstruction: "factor at 1±√2, determine the factor signs, and derive the upper bound", statement_shape: "0≤κ → κ²≤2κ+1 → κ≤1+√2", reconstruction_target_providers: &["CrouzeixTextbook.Part05.one_plus_sqrt_two_barrier"], semantic_fragments: &["0 ≤ κ", "κ ^ 2 ≤ 2 * κ + 1", "κ ≤ 1 + Real.sqrt 2"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-27-E02", parent: "CFT-27-002", prompt: "Numerically compare 1+√2 with 2 and 5/2.", reconstruction: "square positive comparisons and obtain 2<1+√2<5/2", statement_shape: "2<1+√2 ∧ 1+√2<5/2", reconstruction_target_providers: &["CrouzeixTextbook.Part05.sqrt_two_squared"], semantic_fragments: &["2 < (1 : ℝ) + Real.sqrt 2", "(1 : ℝ) + Real.sqrt 2 < 5 / 2"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-27-E03", parent: "CFT-27-003", prompt: "Prove the quadratic barrier by completing the square.", reconstruction: "rewrite as (κ-1)²≤2 and use the nonnegative square-root branch to derive the upper bound", statement_shape: "κ²≤2κ+1 → (κ-1)²≤2 ∧ κ≤1+√2", reconstruction_target_providers: &["CrouzeixTextbook.Part05.one_plus_sqrt_two_barrier"], semantic_fragments: &["κ ^ 2 ≤ 2 * κ + 1", "(κ - 1) ^ 2 ≤ 2", "κ ≤ 1 + Real.sqrt 2"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-27-E04", parent: "CFT-27-004", prompt: "Show the triangle inequality can attain 1+√2.", reconstruction: "construct aligned complex summands with norms 1 and √2 and verify equality in the triangle inequality", statement_shape: "∃ a b:ℂ, ‖a‖=1 ∧ ‖b‖=√2 ∧ ‖a+b‖=1+√2", reconstruction_target_providers: &[], semantic_fragments: &["∃ a b : ℂ", "‖a‖ = 1", "‖b‖ = Real.sqrt 2", "‖a + b‖ = 1 + Real.sqrt 2"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-27-E05", parent: "CFT-27-005", prompt: "Give a structural condition improving the triangle estimate for vectors in a complex inner-product space.", reconstruction: "assume inner-product orthogonality and replace the triangle bound by the Pythagorean norm identity", statement_shape: "⟪x,y⟫=0 → ‖x+y‖²=‖x‖²+‖y‖²", reconstruction_target_providers: &[], semantic_fragments: &["[NormedAddCommGroup E]", "[InnerProductSpace ℂ E]", "x y : E", "⟪x, y⟫_ℂ = 0", "‖x + y‖ ^ 2 = ‖x‖ ^ 2 + ‖y‖ ^ 2"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-27-E06", parent: "CFT-27-006", prompt: "Show the scalar upper barrier does not require κ≥0, and locate where nonnegativity enters the operator application.", reconstruction: "factor the quadratic at 1±√2 and derive κ≤1+√2 without a sign hypothesis; prove separately that κ≥0 when κ is instantiated as an operator norm", statement_shape: "κ²≤2κ+1 → factored inequality ∧ κ≤1+√2 ∧ (κ=‖T‖→0≤κ)", reconstruction_target_providers: &["CrouzeixTextbook.Part05.one_plus_sqrt_two_barrier"], semantic_fragments: &["κ ^ 2 ≤ 2 * κ + 1", "(κ - (1 + Real.sqrt 2)) * (κ - (1 - Real.sqrt 2)) ≤ 0", "κ ≤ 1 + Real.sqrt 2", "[InnerProductSpace ℂ E]", "T : E →L[ℂ] E", "κ = ‖T‖ → 0 ≤ κ"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-28-E01", parent: "CFT-28-001", prompt: "Define a common-contour power Cauchy family.", reconstruction: "bind one contour, measure, boundary function, matrix, and target to a formula valid for every natural power", statement_shape: "HasParametricPowerCauchyFormula Γ μ B f T ↔ ∀m, ∫ f(x)^m·firstPart(Γ,B,x)dμ=T^m", reconstruction_target_providers: &[], semantic_fragments: &["HasParametricPowerCauchyFormula", "ParametricConvexBoundary", "Measure i", "∀ m : ℕ", "parametricBoundaryFirstPart", "= T ^ m"], permits_definition_unfolding: true },
    ExerciseSpec { exercise_id: "CFT-28-E02", parent: "CFT-28-002", prompt: "Prove pointwise contractivity of every power of a contractive boundary function.", reconstruction: "use norm multiplicativity and monotonicity from ‖f(x)‖≤1 to every natural power", statement_shape: "ContractiveBoundaryFunction f → ∀m x, ‖f(x)^m‖≤1", reconstruction_target_providers: &[], semantic_fragments: &["ContractiveBoundaryFunction", "∀ m : ℕ", "∀ x : i", "‖f.function x ^ m‖ ≤ 1"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-28-E03", parent: "CFT-28-003", prompt: "Derive the scalar Cayley power series.", reconstruction: "expand the named Cayley transform geometrically and justify convergence when ‖zw‖<1", statement_shape: "‖zw‖<1 → cayleyTransform z w=1+2∑'(zw)^(m+1)", reconstruction_target_providers: &[], semantic_fragments: &["cayleyTransform z w", "‖z * w‖ < 1", "∑' m : ℕ", "(z * w) ^ (m + 1)"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-28-E04", parent: "CFT-28-004", prompt: "Derive mass one from the zeroth power Cauchy identity.", reconstruction: "specialize the simultaneous common-contour formula at m=0 and simplify both zeroth powers", statement_shape: "HasParametricPowerCauchyFormula Γ μ B f T → ∫firstPart=I", reconstruction_target_providers: &["CrouzeixConjecture.HasParametricPowerCauchyFormula.mass_eq_one"], semantic_fragments: &["HasParametricPowerCauchyFormula", "∫ x, CrouzeixConjecture.parametricBoundaryFirstPart", "= (1 : CrouzeixConjecture.SquareMatrix n)"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-28-E05", parent: "CFT-28-005", prompt: "Compare the common-contour package with its indexed identities and mass normalization.", reconstruction: "extract every indexed identity and the zeroth-power mass identity while keeping the shared contour and measure explicit", statement_shape: "HasParametricPowerCauchyFormula → (∀m, power identity) ∧ mass=I", reconstruction_target_providers: &["CrouzeixConjecture.HasParametricPowerCauchyFormula.mass_eq_one"], semantic_fragments: &["HasParametricPowerCauchyFormula", "∀ m : ℕ", "parametricBoundaryFirstPart", "= T ^ m", "= (1 : CrouzeixConjecture.SquareMatrix n)"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-28-E06", parent: "CFT-28-006", prompt: "Assemble the deferred Jin completion-to-two consequence with every hypothesis explicit.", reconstruction: "derive spectral containment, construct the common positive-real completion, and call the named Jin completion-to-two provider", statement_shape: "power Cauchy + W(B)⊆Ω + diagonalization + ‖λj‖≤1 → ‖T‖≤2", reconstruction_target_providers: &["CrouzeixConjecture.exists_positiveRealCompletion_of_parametricPowerCauchy", "CrouzeixConjecture.norm_le_two_of_parametricPowerCauchy"], semantic_fragments: &["HasParametricPowerCauchyFormula", "numericalRange B ⊆ Omega", "SimpleDiagonalization B", "T = CrouzeixConjecture.innerConjugation", "∀ j, ‖lambda j‖ ≤ 1", "‖T‖ ≤ 2"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-29-E01", parent: "CFT-29-001", prompt: "Prove the zero-maximum edge case of the polynomial Crouzeix bound.", reconstruction: "unpack the matrix polynomial bound and use norm definiteness to show p(A)=0 when the numerical-range maximum is zero", statement_shape: "PolynomialCrouzeixBound A p → maxOnW(A,p)=0 → polynomialEval p A=0", reconstruction_target_providers: &[], semantic_fragments: &["PolynomialCrouzeixBound A p", "maxPolynomialModulusOnNumericalRange A p = 0", "polynomialEval p A = 0"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-29-E02", parent: "CFT-29-002", prompt: "Compute W(J) for the 2×2 nilpotent Jordan block.", reconstruction: "parametrize a unit vector, bound the coordinate product, and realize every phase to prove exact disk equality", statement_shape: "numericalRange jordanNilpotentTwo=closedBall(0,1/2)", reconstruction_target_providers: &[], semantic_fragments: &["numericalRange CrouzeixConjecture.jordanNilpotentTwo", "Metric.closedBall 0 ((1 : ℝ) / 2)"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-29-E03", parent: "CFT-29-003", prompt: "Prove the sharp ratio two for p(z)=z on the Jordan block.", reconstruction: "combine ‖p(J)‖=1 with the exact numerical-range maximum 1/2 and perform the division", statement_shape: "‖polynomialEval X J‖/maxOnW(J,X)=2", reconstruction_target_providers: &["CrouzeixConjecture.jordanNilpotentTwo_attains_two"], semantic_fragments: &["polynomialEval Polynomial.X CrouzeixConjecture.jordanNilpotentTwo", "maxPolynomialModulusOnNumericalRange CrouzeixConjecture.jordanNilpotentTwo Polynomial.X", "= 2"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-29-E04", parent: "CFT-29-004", prompt: "Prove the polynomial numerical-range constant is one for a normal matrix.", reconstruction: "use the normality equation and unitary diagonalization to bound ‖p(A)‖ by the numerical-range maximum", statement_shape: "AAᴴ=AᴴA → ‖p(A)‖≤maxOnW(A,p)", reconstruction_target_providers: &[], semantic_fragments: &["SquareMatrix", "A * Aᴴ = Aᴴ * A", "‖CrouzeixConjecture.polynomialEval p A‖ ≤ CrouzeixConjecture.maxPolynomialModulusOnNumericalRange A p"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-29-E05", parent: "CFT-29-005", prompt: "Carry out the M>0 polynomial normalization.", reconstruction: "set q=M⁻¹p, record maxOnW(q)=1, apply the normalized bound, and rescale the actual matrix polynomial evaluation", statement_shape: "M>0 → maxOnW(p)=M → maxOnW(M⁻¹p)=1 → ‖(M⁻¹p)(A)‖≤2 → ‖p(A)‖≤2M", reconstruction_target_providers: &[], semantic_fragments: &["0 < M", "maxPolynomialModulusOnNumericalRange A p = M", "maxPolynomialModulusOnNumericalRange A (((M⁻¹ : ℝ) : ℂ) • p) = 1", "‖CrouzeixConjecture.polynomialEval p A‖ ≤ 2 * M"], permits_definition_unfolding: false },
    ExerciseSpec { exercise_id: "CFT-29-E06", parent: "CFT-29-006", prompt: "Restate MainTheoremStatement with every finite-index assumption and its full conclusion explicit.", reconstruction: "quantify the index type, Fintype, DecidableEq, Nonempty, matrix, and polynomial and prove that this expanded surface is exactly MainTheoremStatement", statement_shape: "∀n [Fintype n] [DecidableEq n] [Nonempty n], MainTheoremStatement ↔ ∀A p, PolynomialCrouzeixBound A p", reconstruction_target_providers: &[], semantic_fragments: &["[Fintype n]", "[DecidableEq n]", "[Nonempty n]", "MainTheoremStatement", "SquareMatrix n", "Polynomial ℂ", "PolynomialCrouzeixBound A p"], permits_definition_unfolding: true },
];

// These are elaborated by Lean in `compile_planned_exercise_receipt`.  The
// resulting normalized-type fingerprints, rather than pretty-printed Rust
// strings, are the promotion authority for the future solution theorems.
const EXERCISE_LEAN_TYPES: [(&str, &str); 30] = [
    ("CFT-25-E01", "∀ (K : Set ℂ) (hKne : K.Nonempty) (hKclosed : IsClosed K) (hKconvex : Convex ℝ K) (z : ℂ), ∃ p : ℂ, p ∈ K ∧ ∀ w ∈ K, ‖z - p‖ ≤ ‖z - w‖"),
    ("CFT-25-E02", "∀ (r : ℝ) (hr : 0 < r) (x : ℂ) (hne : (Metric.closedBall (0 : ℂ) r).Nonempty) (hcompact : IsCompact (Metric.closedBall (0 : ℂ) r)) (hconvex : Convex ℝ (Metric.closedBall (0 : ℂ) r)), CrouzeixConjecture.convexProjection (Metric.closedBall (0 : ℂ) r) hne hcompact hconvex x = if ‖x‖ ≤ r then x else ((r / ‖x‖ : ℝ) : ℂ) * x"),
    ("CFT-25-E03", "∀ (x p q : ℂ), ⟪x - p, q - p⟫_ℝ ≤ 0 → ⟪x - q, p - q⟫_ℝ ≤ 0 → p = q"),
    ("CFT-25-E04", "∀ (x y px py : ℂ), ⟪x - px, py - px⟫_ℝ ≤ 0 → ⟪y - py, px - py⟫_ℝ ≤ 0 → ‖px - py‖ ≤ ‖x - y‖"),
    ("CFT-25-E05", "∀ (v w : ℂ), v ≠ w → ¬ DifferentiableAt ℝ (fun t : ℝ => if t ≤ 0 then t • v else t • w) 0"),
    ("CFT-25-E06", "∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n], CrouzeixConjecture.CanonicalParallelOrientedRadialBoundaryStatement (n := n)"),
    ("CFT-26-E01", "∀ (n : Type) [Fintype n] [DecidableEq n] {Omega : Set ℂ} {sigma nu : ℂ} (hgeom : CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu) (B : CrouzeixConjecture.SquareMatrix n) (hWB : CrouzeixConjecture.numericalRange B ⊆ Omega) (x : CrouzeixConjecture.EuclideanVector n), ‖x‖ = 1 → 0 ≤ RCLike.re (star nu * (sigma - ⟪x, CrouzeixConjecture.euclideanOperator B x⟫_ℂ))"),
    ("CFT-26-E02", "∀ (n : Type) [Fintype n] [DecidableEq n] {Omega : Set ℂ} {sigma nu : ℂ} (hgeom : CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu) (B : CrouzeixConjecture.SquareMatrix n) (hWB : CrouzeixConjecture.numericalRange B ⊆ Omega) (x : CrouzeixConjecture.EuclideanVector n), ‖x‖ = 1 → 0 ≤ (CrouzeixConjecture.euclideanOperator (CrouzeixConjecture.doubleLayerSupportMatrix B sigma nu)).reApplyInnerSelf x"),
    ("CFT-26-E03", "∀ (n : Type) [Fintype n] [DecidableEq n] (M B : CrouzeixConjecture.SquareMatrix n), M.PosSemidef → (Bᴴ * M * B).PosSemidef"),
    ("CFT-26-E04", "∀ (n : Type) [Fintype n] [DecidableEq n] (B R : CrouzeixConjecture.SquareMatrix n) (sigma nu : ℂ), (sigma • (1 : CrouzeixConjecture.SquareMatrix n) - B) * R = 1 → Rᴴ * CrouzeixConjecture.doubleLayerSupportMatrix B sigma nu * R = CrouzeixConjecture.doubleLayerDensity R nu"),
    ("CFT-26-E05", "∀ (n : Type) [Fintype n] [DecidableEq n] {Omega : Set ℂ} {sigma nu : ℂ} (hgeom : CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu) (B : CrouzeixConjecture.SquareMatrix n), CrouzeixConjecture.numericalRange B ⊆ Omega → IsUnit (sigma • (1 : CrouzeixConjecture.SquareMatrix n) - B)"),
    ("CFT-26-E06", "∀ (n : Type) [Fintype n] [DecidableEq n] {Omega : Set ℂ} {sigma nu : ℂ} (hgeom : CrouzeixConjecture.OutwardBoundarySupport Omega sigma nu) (B : CrouzeixConjecture.SquareMatrix n), CrouzeixConjecture.numericalRange B ⊆ Omega → (CrouzeixConjecture.doubleLayerDensity (CrouzeixConjecture.doubleLayerResolvent B sigma) nu).PosSemidef"),
    ("CFT-27-E01", "∀ κ : ℝ, 0 ≤ κ → κ ^ 2 ≤ 2 * κ + 1 → κ ≤ 1 + Real.sqrt 2"),
    ("CFT-27-E02", "2 < (1 : ℝ) + Real.sqrt 2 ∧ (1 : ℝ) + Real.sqrt 2 < 5 / 2"),
    ("CFT-27-E03", "∀ κ : ℝ, κ ^ 2 ≤ 2 * κ + 1 → (κ - 1) ^ 2 ≤ 2 ∧ κ ≤ 1 + Real.sqrt 2"),
    ("CFT-27-E04", "∃ a b : ℂ, ‖a‖ = 1 ∧ ‖b‖ = Real.sqrt 2 ∧ ‖a + b‖ = 1 + Real.sqrt 2"),
    ("CFT-27-E05", "∀ (E : Type) [NormedAddCommGroup E] [InnerProductSpace ℂ E] (x y : E), ⟪x, y⟫_ℂ = 0 → ‖x + y‖ ^ 2 = ‖x‖ ^ 2 + ‖y‖ ^ 2"),
    ("CFT-27-E06", "∀ κ : ℝ, κ ^ 2 ≤ 2 * κ + 1 → (κ - (1 + Real.sqrt 2)) * (κ - (1 - Real.sqrt 2)) ≤ 0 ∧ κ ≤ 1 + Real.sqrt 2 ∧ ∀ (E : Type) [NormedAddCommGroup E] [InnerProductSpace ℂ E] (T : E →L[ℂ] E), κ = ‖T‖ → 0 ≤ κ"),
    ("CFT-28-E01", "∀ (i n : Type) [TopologicalSpace i] [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Fintype n] [DecidableEq n] [Nonempty n] (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ) (Gamma : CrouzeixConjecture.ParametricConvexBoundary (i := i) Omega) (B : CrouzeixConjecture.SquareMatrix n) (f : CrouzeixConjecture.ContractiveBoundaryFunction i) (T : CrouzeixConjecture.SquareMatrix n), CrouzeixConjecture.HasParametricPowerCauchyFormula Gamma mu B f T ↔ ∀ m : ℕ, ∫ x, (f.function x) ^ m • CrouzeixConjecture.parametricBoundaryFirstPart Gamma B x ∂mu = T ^ m"),
    ("CFT-28-E02", "∀ (i : Type) [TopologicalSpace i] (f : CrouzeixConjecture.ContractiveBoundaryFunction i), ∀ m : ℕ, ∀ x : i, ‖f.function x ^ m‖ ≤ 1"),
    ("CFT-28-E03", "∀ z w : ℂ, ‖z * w‖ < 1 → CrouzeixConjecture.cayleyTransform z w = 1 + 2 * ∑' m : ℕ, (z * w) ^ (m + 1)"),
    ("CFT-28-E04", "∀ (i n : Type) [TopologicalSpace i] [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Fintype n] [DecidableEq n] [Nonempty n] (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ) (Gamma : CrouzeixConjecture.ParametricConvexBoundary (i := i) Omega) (B : CrouzeixConjecture.SquareMatrix n) (f : CrouzeixConjecture.ContractiveBoundaryFunction i) (T : CrouzeixConjecture.SquareMatrix n), CrouzeixConjecture.HasParametricPowerCauchyFormula Gamma mu B f T → ∫ x, CrouzeixConjecture.parametricBoundaryFirstPart Gamma B x ∂mu = (1 : CrouzeixConjecture.SquareMatrix n)"),
    ("CFT-28-E05", "∀ (i n : Type) [TopologicalSpace i] [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Fintype n] [DecidableEq n] [Nonempty n] (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ) (Gamma : CrouzeixConjecture.ParametricConvexBoundary (i := i) Omega) (B : CrouzeixConjecture.SquareMatrix n) (f : CrouzeixConjecture.ContractiveBoundaryFunction i) (T : CrouzeixConjecture.SquareMatrix n), CrouzeixConjecture.HasParametricPowerCauchyFormula Gamma mu B f T → (∀ m : ℕ, ∫ x, (f.function x) ^ m • CrouzeixConjecture.parametricBoundaryFirstPart Gamma B x ∂mu = T ^ m) ∧ ∫ x, CrouzeixConjecture.parametricBoundaryFirstPart Gamma B x ∂mu = (1 : CrouzeixConjecture.SquareMatrix n)"),
    ("CFT-28-E06", "∀ (i n : Type) [TopologicalSpace i] [CompactSpace i] [MeasurableSpace i] [OpensMeasurableSpace i] [Fintype n] [DecidableEq n] [Nonempty n] (mu : Measure i) [IsFiniteMeasure mu] (Omega : Set ℂ) (Gamma : CrouzeixConjecture.ParametricConvexBoundary (i := i) Omega) (B T : CrouzeixConjecture.SquareMatrix n) (hWB : CrouzeixConjecture.numericalRange B ⊆ Omega) (f : CrouzeixConjecture.ContractiveBoundaryFunction i), CrouzeixConjecture.HasParametricPowerCauchyFormula Gamma mu B f T → ∀ (hB : CrouzeixConjecture.SimpleDiagonalization B) (lambda : n → ℂ), T = CrouzeixConjecture.innerConjugation hB.changeBasis (Matrix.diagonal lambda) → (∀ j, ‖lambda j‖ ≤ 1) → ‖T‖ ≤ 2"),
    ("CFT-29-E01", "∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n] (A : CrouzeixConjecture.SquareMatrix n) (p : Polynomial ℂ), CrouzeixConjecture.PolynomialCrouzeixBound A p → CrouzeixConjecture.maxPolynomialModulusOnNumericalRange A p = 0 → CrouzeixConjecture.polynomialEval p A = 0"),
    ("CFT-29-E02", "CrouzeixConjecture.numericalRange CrouzeixConjecture.jordanNilpotentTwo = Metric.closedBall 0 ((1 : ℝ) / 2)"),
    ("CFT-29-E03", "‖CrouzeixConjecture.polynomialEval Polynomial.X CrouzeixConjecture.jordanNilpotentTwo‖ / CrouzeixConjecture.maxPolynomialModulusOnNumericalRange CrouzeixConjecture.jordanNilpotentTwo Polynomial.X = 2"),
    ("CFT-29-E04", "∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n] (A : CrouzeixConjecture.SquareMatrix n) (p : Polynomial ℂ), A * Aᴴ = Aᴴ * A → ‖CrouzeixConjecture.polynomialEval p A‖ ≤ CrouzeixConjecture.maxPolynomialModulusOnNumericalRange A p"),
    ("CFT-29-E05", "∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n] (A : CrouzeixConjecture.SquareMatrix n) (p : Polynomial ℂ) (M : ℝ), 0 < M → CrouzeixConjecture.maxPolynomialModulusOnNumericalRange A p = M → CrouzeixConjecture.maxPolynomialModulusOnNumericalRange A (((M⁻¹ : ℝ) : ℂ) • p) = 1 → ‖CrouzeixConjecture.polynomialEval (((M⁻¹ : ℝ) : ℂ) • p) A‖ ≤ 2 → ‖CrouzeixConjecture.polynomialEval p A‖ ≤ 2 * M"),
    ("CFT-29-E06", "∀ (n : Type) [Fintype n] [DecidableEq n] [Nonempty n], CrouzeixConjecture.MainTheoremStatement (n := n) ↔ ∀ (A : CrouzeixConjecture.SquareMatrix n) (p : Polynomial ℂ), CrouzeixConjecture.PolynomialCrouzeixBound A p"),
];

fn wave4_contract() -> String {
    fs::read_to_string(workspace_root().join(WAVE4_CONTRACT))
        .expect("frozen Wave 4 common-machinery contract")
}

fn active_phase(contract: &str) -> Wave4Phase {
    let label = contract
        .split_once("- Active phase: `")
        .expect("Wave 4 active phase")
        .1
        .split('`')
        .next()
        .expect("Wave 4 active phase label");
    Wave4Phase::parse(label).unwrap_or_else(|| panic!("unknown Wave 4 phase `{label}`"))
}

fn normalized_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn exercise_id(spec: CommonSpec) -> String {
    let index = spec.item_id[7..].parse::<u64>().expect("CFT index");
    format!("CFT-{:02}-E{index:02}", spec.chapter)
}

fn exercise_declaration(spec: CommonSpec) -> String {
    let part = if spec.chapter <= 28 {
        "Part05"
    } else {
        "Part06"
    };
    let index = spec.item_id[7..].parse::<u64>().expect("CFT index");
    format!(
        "CrouzeixTextbook.{part}.Exercises.Chapter{}.exercise_{index:02}_solution",
        spec.chapter
    )
}

fn common_spec(item_id: &str) -> CommonSpec {
    COMMON_SPECS
        .into_iter()
        .find(|spec| spec.item_id == item_id)
        .unwrap_or_else(|| panic!("unknown common item {item_id}"))
}

fn exercise_declaration_for(spec: ExerciseSpec) -> String {
    exercise_declaration(common_spec(spec.parent))
}

fn exercise_region<'a>(markdown: &'a str, exercise_id: &str) -> Option<&'a str> {
    let marker = format!("### {exercise_id} ");
    let mut matches = markdown.match_indices(&marker);
    let (start, _) = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    let suffix = &markdown[start + marker.len()..];
    let end = [suffix.find("\n### CFT-"), suffix.find("\n## ")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(suffix.len());
    Some(&markdown[start..start + marker.len() + end])
}

fn exercise_prose_errors(markdown: &str, spec: ExerciseSpec) -> Vec<String> {
    let mut errors = Vec::new();
    let anchor = format!("{{#exercise-{}}}", spec.exercise_id.to_lowercase());
    if markdown.matches(&anchor).count() != 1 {
        errors.push(format!(
            "{} must have exactly one canonical exercise anchor",
            spec.exercise_id
        ));
    }
    let Some(block) = exercise_region(markdown, spec.exercise_id) else {
        errors.push(format!(
            "{} must have exactly one canonical exercise block",
            spec.exercise_id
        ));
        return errors;
    };
    if !block.contains(spec.prompt) {
        errors.push(format!(
            "{} canonical prompt no longer matches its frozen mathematical task",
            spec.exercise_id
        ));
    }
    let solution = section_content(block, "Complete written solution").unwrap_or_default();
    if !substantive(solution)
        || !["=", "≤", "≥", "\\le", "\\ge", "therefore", "hence"]
            .iter()
            .any(|marker| solution.contains(marker))
    {
        errors.push(format!(
            "{} lacks a substantive complete written solution with its reconstruction algebra",
            spec.exercise_id
        ));
    }
    if !normalized_text(solution)
        .to_lowercase()
        .contains(&normalized_text(spec.reconstruction).to_lowercase())
    {
        errors.push(format!(
            "{} written solution does not discharge its frozen expected reconstruction `{}`",
            spec.exercise_id, spec.reconstruction
        ));
    }
    errors
}

fn receipt_type_hash(normalized_type: &str) -> String {
    format!("{:x}", Sha256::digest(normalized_type.as_bytes()))
}

fn planned_exercise_name(exercise_id: &str) -> String {
    format!(
        "CrouzeixTextbook.PlannedExercise.{}",
        exercise_id.replace('-', "_")
    )
}

fn exercise_lean_type(exercise_id: &str) -> &'static str {
    EXERCISE_LEAN_TYPES
        .iter()
        .find_map(|(candidate, lean_type)| (*candidate == exercise_id).then_some(*lean_type))
        .unwrap_or_else(|| panic!("missing planned Lean type for {exercise_id}"))
}

fn exercise_semantic_errors(spec: ExerciseSpec, lean_type: &str) -> Vec<String> {
    let mut errors = Vec::new();
    for fragment in spec.semantic_fragments {
        if !lean_type.contains(fragment) {
            errors.push(format!(
                "{} planned type lost semantic fragment `{fragment}`",
                spec.exercise_id
            ));
        }
    }
    let normalized = lean_type.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized == "True" || normalized.contains("→ True") {
        errors.push(format!(
            "{} planned type is a vacuous truth",
            spec.exercise_id
        ));
    }
    if normalized.contains("Nonempty n → Nonempty n")
        || normalized.contains("∃ F : ℕ → ℂ, ∀ k : ℕ, F k =")
    {
        errors.push(format!(
            "{} planned type is a synthetic witness or identity surrogate",
            spec.exercise_id
        ));
    }
    if !spec.permits_definition_unfolding && normalized.contains(" ↔ ") {
        errors.push(format!(
            "{} promises a theorem but is only a definition-unfolding equivalence",
            spec.exercise_id
        ));
    }
    let implication_parts = normalized.split(" → ").collect::<Vec<_>>();
    if implication_parts
        .windows(2)
        .any(|parts| parts[0].trim_matches(['(', ')']) == parts[1].trim_matches(['(', ')']))
    {
        errors.push(format!(
            "{} planned type contains an identity implication",
            spec.exercise_id
        ));
    }
    errors
}

fn compile_planned_exercise_receipt() -> Value {
    assert_common_lean_artifacts_fresh();
    let temp = tempfile::tempdir().expect("planned exercise compiler probe root");
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .expect("private planned exercise compiler probe root");
    let source_path = temp.path().join("PlannedCommonExercises.lean");
    let declarations = EXERCISE_LEAN_TYPES
        .iter()
        .map(|(exercise_id, lean_type)| {
            format!("axiom {} : {lean_type}", exercise_id.replace('-', "_"))
        })
        .collect::<Vec<_>>()
        .join("\n");
    let names = EXERCISE_LEAN_TYPES
        .iter()
        .map(|(exercise_id, _)| format!("`{}", planned_exercise_name(exercise_id)))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "import CrouzeixTextbook.ExportReceipt\n\
         open Lean MeasureTheory\n\
         open scoped ComplexConjugate ComplexOrder InnerProductSpace Matrix Matrix.Norms.L2Operator\n\
         namespace CrouzeixTextbook.PlannedExercise\n\
         {declarations}\n\
         end CrouzeixTextbook.PlannedExercise\n\
         run_cmd do\n\
           let env ← getEnv\n\
           let names : Array Name := #[{names}]\n\
           let rows ← Lean.Elab.Command.liftCoreM <|\n\
             CrouzeixTextbook.ExportReceipt.receiptRows env names\n\
           IO.println s!\"PLANNED_EXERCISE_RECEIPT:{{(Json.arr rows).compress}}\"\n"
    );
    fs::write(&source_path, source).expect("write planned exercise compiler probe");
    let output = super::textbook_lean_command("lake")
        .arg("env")
        .arg("lean")
        .arg(&source_path)
        .current_dir(workspace_root().join("formalization/lean"))
        .output()
        .expect("run planned exercise compiler probe");
    assert!(
        output.status.success(),
        "planned exercise compiler probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("planned exercise probe UTF-8");
    let payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("PLANNED_EXERCISE_RECEIPT:"))
        .expect("planned exercise compiler probe marker");
    serde_json::from_str(payload).expect("planned exercise compiler probe JSON")
}

fn planned_exercise_receipt() -> &'static Value {
    static RECEIPT: OnceLock<Value> = OnceLock::new();
    RECEIPT.get_or_init(compile_planned_exercise_receipt)
}

fn receipt_dependencies(rows: &[Value], root: &str) -> BTreeSet<String> {
    fn visit(rows: &[Value], name: &str, seen: &mut BTreeSet<String>) {
        let Some(row) = rows.iter().find(|row| row["name"].as_str() == Some(name)) else {
            return;
        };
        for dependency in row["direct_dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if seen.insert(dependency.to_owned()) {
                visit(rows, dependency, seen);
            }
        }
    }
    let mut result = BTreeSet::new();
    visit(rows, root, &mut result);
    result.remove(root);
    result
}

fn receipt_body_dependencies(rows: &[Value], root: &str) -> BTreeSet<String> {
    fn visit(rows: &[Value], name: &str, seen: &mut BTreeSet<String>) {
        let Some(row) = rows.iter().find(|row| row["name"].as_str() == Some(name)) else {
            return;
        };
        for dependency in row["body_dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            if seen.insert(dependency.to_owned()) {
                visit(rows, dependency, seen);
            }
        }
    }
    let mut result = BTreeSet::new();
    visit(rows, root, &mut result);
    result.remove(root);
    result
}

fn attach_body_dependencies(mut receipt: Value) -> Value {
    let body_rows = receipt["body_dependency_rows"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for declaration in receipt["declarations"].as_array_mut().into_iter().flatten() {
        let name = declaration["name"].as_str().unwrap_or_default();
        declaration["body_dependencies"] = body_rows
            .iter()
            .find(|row| row["name"].as_str() == Some(name))
            .map(|row| row["body_dependencies"].clone())
            .unwrap_or_else(|| serde_json::json!([]));
    }
    let known = receipt["declarations"]
        .as_array()
        .expect("compiler receipt declarations")
        .iter()
        .filter_map(|row| row["name"].as_str().map(str::to_owned))
        .collect::<BTreeSet<_>>();
    let dependency_only_rows = body_rows.into_iter().filter(|row| {
        row["name"]
            .as_str()
            .is_some_and(|name| !known.contains(name))
    });
    receipt["declarations"]
        .as_array_mut()
        .expect("compiler receipt declarations")
        .extend(dependency_only_rows);
    receipt
        .as_object_mut()
        .expect("compiler receipt object")
        .remove("body_dependency_rows");
    receipt
}

fn reconstruction_target_errors(
    spec: ExerciseSpec,
    dependencies: &BTreeSet<String>,
) -> Vec<String> {
    spec.reconstruction_target_providers
        .iter()
        .filter(|provider| dependencies.contains(**provider))
        .map(|provider| {
            format!(
                "{} invokes reconstruction target provider `{provider}`",
                spec.exercise_id
            )
        })
        .collect()
}

fn common_statement_constants(public_rows: &[Value]) -> BTreeSet<String> {
    let mut constants = planned_exercise_receipt()
        .as_array()
        .expect("planned exercise compiler receipt rows")
        .iter()
        .flat_map(|row| {
            row["direct_dependencies"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_owned)
        })
        .collect::<BTreeSet<_>>();
    for spec in COMMON_SPECS {
        let row = public_rows
            .iter()
            .find(|row| row["name"].as_str() == Some(spec.public_declaration))
            .expect("public provider compiler receipt row");
        let body = row["body_dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        constants.extend(
            row["direct_dependencies"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .filter(|dependency| !body.contains(dependency))
                .map(str::to_owned),
        );
    }
    constants
}

fn same_or_later_provider_errors(
    spec: ExerciseSpec,
    exercise_body_dependencies: &BTreeSet<String>,
    public_rows: &[Value],
) -> Vec<String> {
    let parent_position = COMMON_SPECS
        .iter()
        .position(|candidate| candidate.item_id == spec.parent)
        .expect("parent position");
    let mut allowed = common_statement_constants(public_rows);
    for earlier in &COMMON_SPECS[..parent_position] {
        allowed.insert(earlier.public_declaration.to_owned());
        allowed.extend(receipt_body_dependencies(
            public_rows,
            earlier.public_declaration,
        ));
    }
    if spec.exercise_id == "CFT-25-E02" {
        // E02 derives the disk formula from the selected compact projection's
        // defining membership and infimum properties.  CFT-25-001 is now the
        // stronger closed-set existence theorem, so these lower compact
        // projection primitives must be classified explicitly rather than
        // inherited accidentally from its old proof body.
        allowed.extend(
            [
                "CrouzeixConjecture.convexProjection",
                "CrouzeixConjecture.convexProjection_mem",
                "CrouzeixConjecture.norm_sub_convexProjection_eq_iInf",
            ]
            .into_iter()
            .map(str::to_owned),
        );
    }
    if spec.exercise_id == "CFT-25-E06" {
        // E06 is intentionally the direct construction exercise for the
        // canonical radial package.  These are its lower-level geometric
        // ingredients, not a preassembled same-or-later theorem shortcut.
        allowed.extend(
            [
                "CrouzeixConjecture.HasOrientedRadialConvexBoundary",
                "CrouzeixConjecture.PositivePeriodicRadialData",
                "CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary",
                "CrouzeixConjecture.isCompact_numericalRange",
                "CrouzeixConjecture.numericalRange_convex",
                "CrouzeixConjecture.numericalRange_nonempty",
                "CrouzeixConjecture.orientedRadialConvexBoundary_thickening",
                "CrouzeixConjecture.outerApproximationRadius_pos",
                "CrouzeixConjecture.parallelPositivePeriodicRadialData",
            ]
            .into_iter()
            .map(str::to_owned),
        );
    }
    if spec.exercise_id == "CFT-26-E01" {
        // E01 is the direct transport exercise for the scalar support field of
        // OutwardBoundarySupport.  The field is lower-level geometry, not a
        // preassembled support-matrix or density theorem.
        allowed.insert("CrouzeixConjecture.OutwardBoundarySupport.support_inequality".to_owned());
    }
    if spec.exercise_id == "CFT-26-E02" {
        // E02 expands the quadratic form itself.  These are the scalar support
        // field and the operator/adjoint bridge needed for that expansion,
        // rather than either forbidden positivity theorem.
        allowed.extend(
            [
                "CrouzeixConjecture.OutwardBoundarySupport.support_inequality",
                "CrouzeixConjecture.euclideanOperator_conjTranspose",
            ]
            .into_iter()
            .map(str::to_owned),
        );
    }
    if spec.exercise_id == "CFT-28-E02" {
        // E02 proves contractivity of powers from the boundary function's
        // defining pointwise bound.  That field is reconstruction data, not
        // a later complete-power or completion theorem.
        allowed.insert("CrouzeixConjecture.ContractiveBoundaryFunction.norm_le_one".to_owned());
    }
    if spec.exercise_id == "CFT-28-E03" {
        // E03 expands the scalar Cayley definition and the geometric series.
        // The generated equation helper belongs to that definition's body;
        // it is not the matrix-valued Cayley-companion theorem.
        allowed.extend(
            [
                "CrouzeixConjecture.cayleyTransform",
                "CrouzeixConjecture.cayleyTransform.eq_1",
            ]
            .into_iter()
            .map(str::to_owned),
        );
    }
    if spec.exercise_id == "CFT-28-E06" {
        // E06 is an explicitly deferred assembly exercise: it must pass from
        // the earlier public completion to the named Jin completion-to-two
        // provider.  The two wrapper shortcuts remain rejected by
        // reconstruction_target_errors.
        allowed.extend(
            [
                "CrouzeixConjecture.matrixSpectrum",
                "CrouzeixConjecture.matrixSpectrum.eq_1",
                "CrouzeixConjecture.positiveRealCompletionStatement",
            ]
            .into_iter()
            .map(str::to_owned),
        );
    }
    // If a statement constant or an explicitly approved reconstruction
    // primitive is allowed, its implementation closure is allowed as well.
    // Otherwise generated helpers such as `convexProjection._proof_1` are
    // misclassified as independent theorem shortcuts.
    let allowed_roots = allowed.iter().cloned().collect::<Vec<_>>();
    for root in allowed_roots {
        allowed.extend(receipt_body_dependencies(public_rows, &root));
    }
    let mut errors = Vec::new();
    for candidate in &COMMON_SPECS[parent_position..] {
        for dependency in receipt_body_dependencies(public_rows, candidate.public_declaration)
            .intersection(exercise_body_dependencies)
            .filter(|dependency| !allowed.contains(*dependency))
        {
            errors.push(format!(
                "{} uses same-or-later common proof provider `{dependency}` from {}",
                spec.exercise_id, candidate.item_id
            ));
        }
    }
    errors
}

fn coordinate_resolves(row: &Value, declaration: &str) -> bool {
    let Some(source_path) = row["source_path"].as_str() else {
        return false;
    };
    let Some(line) = row["line"].as_u64().and_then(|line| line.checked_sub(1)) else {
        return false;
    };
    let Some(column) = row["column"]
        .as_u64()
        .and_then(|column| column.checked_sub(1))
    else {
        return false;
    };
    let Ok(source) = fs::read_to_string(workspace_root().join(source_path)) else {
        return false;
    };
    let Some(source_line) = source.lines().nth(line as usize) else {
        return false;
    };
    let Some(suffix) = source_line.get(column as usize..) else {
        return false;
    };
    declaration
        .rsplit('.')
        .next()
        .is_some_and(|name| suffix.starts_with(name))
}

fn exercise_receipt_errors(
    coverage: &Value,
    exercises: &Value,
    receipt: Option<&Value>,
    phase: Wave4Phase,
) -> Vec<String> {
    let mut errors = Vec::new();
    let exercise_rows = rows_for_chapters(exercises, "exercises");
    let receipt_rows = receipt
        .and_then(|receipt| receipt["declarations"].as_array())
        .cloned()
        .unwrap_or_default();
    if receipt.is_some_and(|receipt| receipt["target"].as_str() != Some("CrouzeixTextbook")) {
        errors.push("exercise compiler receipt has the wrong target".to_owned());
    }
    let theorem_hashes = rows_for_chapters(coverage, "items")
        .iter()
        .filter_map(|row| row["lean_declaration"]["type_sha256"].as_str())
        .collect::<BTreeSet<_>>();
    let planned_rows = planned_exercise_receipt()
        .as_array()
        .expect("planned exercise compiler receipt rows");
    let public_provider_rows = public_provider_receipt()["declarations"]
        .as_array()
        .expect("public common provider compiler receipt rows");
    let mut fingerprints = BTreeMap::<String, String>::new();

    for spec in EXERCISE_SPECS {
        let parent = common_spec(spec.parent);
        if !phase.completes(parent.chapter) {
            continue;
        }
        let declaration = exercise_declaration_for(spec);
        let metadata = exercise_rows
            .iter()
            .find(|row| row["exercise_id"].as_str() == Some(spec.exercise_id))
            .map(|row| &row["lean_solution"]);
        let matches = receipt_rows
            .iter()
            .filter(|row| row["name"].as_str() == Some(declaration.as_str()))
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            errors.push(format!(
                "{} must join to exactly one fresh canonical compiler receipt row, observed {}",
                spec.exercise_id,
                matches.len()
            ));
            continue;
        }
        let row = matches[0];
        if row["kind"].as_str() != Some("theorem") {
            errors.push(format!(
                "{} must compile as a theorem, never a direct/eta alias",
                spec.exercise_id
            ));
        }
        let normalized_type = row["normalized_type"].as_str().unwrap_or_default();
        let type_hash = row["type_sha256"].as_str().unwrap_or_default();
        let planned = planned_rows
            .iter()
            .find(|planned| {
                planned["name"].as_str() == Some(planned_exercise_name(spec.exercise_id).as_str())
            })
            .expect("compiler-validated planned exercise type");
        if normalized_type.trim().is_empty()
            || normalized_type.trim() == "True"
            || receipt_type_hash(normalized_type) != type_hash
        {
            errors.push(format!(
                "{} has a fabricated or vacuous normalized type/hash",
                spec.exercise_id
            ));
        }
        if row["planned_type_sha256"] != planned["type_sha256"] {
            errors.push(format!(
                "{} compiler fingerprint does not match its planned mathematical proposition `{}`",
                spec.exercise_id,
                exercise_lean_type(spec.exercise_id)
            ));
        }
        if row["axioms"] != serde_json::json!(AXIOMS) {
            errors.push(format!(
                "{} compiler axioms are not exactly the approved common-proof set",
                spec.exercise_id
            ));
        }
        if theorem_hashes.contains(type_hash) {
            errors.push(format!(
                "{} reuses a theorem-card statement fingerprint",
                spec.exercise_id
            ));
        }
        if let Some(previous) =
            fingerprints.insert(type_hash.to_owned(), spec.exercise_id.to_owned())
        {
            errors.push(format!(
                "{} and {previous} reuse the same exercise statement fingerprint",
                spec.exercise_id
            ));
        }
        let expected_path = parent.source_path;
        let exact_metadata = metadata.is_some_and(|metadata| {
            metadata["declaration"].as_str() == Some(declaration.as_str())
                && metadata["source_path"] == row["source_path"]
                && metadata["line"] == row["line"]
                && metadata["column"] == row["column"]
                && metadata["type_sha256"] == row["type_sha256"]
                && metadata["axioms"] == row["axioms"]
                && metadata["verification_target"].as_str() == Some("CrouzeixTextbook")
                && row["source_path"].as_str() == Some(expected_path)
        });
        if !exact_metadata || !coordinate_resolves(row, &declaration) {
            errors.push(format!(
                "{} metadata does not exactly match its compiler-valid code locator, hash, axioms, and target",
                spec.exercise_id
            ));
        }
        if !row["body_dependencies"].is_array() {
            errors.push(format!(
                "{} lacks compiler-reported proof-body dependencies",
                spec.exercise_id
            ));
        }
        let dependencies = receipt_dependencies(&receipt_rows, &declaration);
        let body_dependencies = receipt_body_dependencies(&receipt_rows, &declaration);
        if body_dependencies.contains(parent.public_declaration) {
            errors.push(format!(
                "{} invokes its forbidden parent theorem shortcut",
                spec.exercise_id
            ));
        }
        errors.extend(reconstruction_target_errors(spec, &body_dependencies));
        errors.extend(same_or_later_provider_errors(
            spec,
            &body_dependencies,
            public_provider_rows,
        ));
        let parent_position = COMMON_SPECS
            .iter()
            .position(|candidate| candidate.item_id == parent.item_id)
            .expect("parent position");
        for dependency in &dependencies {
            if dependency.starts_with("Crouzeix.Jin")
                || dependency.starts_with("Crouzeix.LoristSchwenninger")
                || dependency.starts_with("Crouzeix.Harp")
                || matches!(
                    dependency.as_str(),
                    "CrouzeixJin" | "CrouzeixLoristSchwenninger" | "CrouzeixHarp"
                )
            {
                errors.push(format!(
                    "{} reaches forbidden terminal provider `{dependency}`",
                    spec.exercise_id
                ));
            }
            if let Some(candidate) = COMMON_SPECS
                .iter()
                .find(|candidate| candidate.public_declaration == dependency)
            {
                let candidate_position = COMMON_SPECS
                    .iter()
                    .position(|row| row.item_id == candidate.item_id)
                    .expect("dependency position");
                if candidate_position >= parent_position {
                    errors.push(format!(
                        "{} uses non-prerequisite common declaration `{dependency}`",
                        spec.exercise_id
                    ));
                }
            }
        }
    }
    errors
}

fn rows_for_chapters<'a>(value: &'a Value, key: &str) -> Vec<&'a Value> {
    array(value, key, key)
        .iter()
        .filter(|row| {
            row["chapter"]
                .as_u64()
                .is_some_and(|chapter| (25..=29).contains(&chapter))
        })
        .collect()
}

fn chapter_path(chapter: u64) -> &'static str {
    match chapter {
        25 => "knowledge/crouzeix_textbook/part_05_crouzeix_machinery/25_convex_boundaries_and_cauchy_layers.md",
        26 => "knowledge/crouzeix_textbook/part_05_crouzeix_machinery/26_double_layer_map.md",
        27 => "knowledge/crouzeix_textbook/part_05_crouzeix_machinery/27_one_plus_sqrt_two_barrier.md",
        28 => "knowledge/crouzeix_textbook/part_05_crouzeix_machinery/28_complete_power_family.md",
        29 => "knowledge/crouzeix_textbook/part_06_constant_two_routes/29_crouzeix_problem_and_sharpness.md",
        _ => panic!("Chapter {chapter} is outside the common-machinery contract"),
    }
}

fn chapter_markdown(chapter: u64) -> String {
    fs::read_to_string(workspace_root().join(chapter_path(chapter)))
        .unwrap_or_else(|error| panic!("read canonical Chapter {chapter}: {error}"))
}

fn card_region(markdown: &str, spec: CommonSpec) -> Option<&str> {
    let marker = format!("### {} ", spec.item_id);
    let mut matches = markdown.match_indices(&marker);
    let (start, _) = matches.next()?;
    if matches.next().is_some() {
        return None;
    }
    let suffix = &markdown[start + marker.len()..];
    let end = [suffix.find("\n### CFT-"), suffix.find("\n## ")]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(suffix.len());
    Some(&markdown[start..start + marker.len() + end])
}

fn section_content<'a>(card: &'a str, field: &str) -> Option<&'a str> {
    let marker = format!("#### {field}\n");
    let start = card.find(&marker)? + marker.len();
    let suffix = &card[start..];
    let end = suffix.find("\n#### ").unwrap_or(suffix.len());
    Some(suffix[..end].trim())
}

fn substantive(text: &str) -> bool {
    let normalized = normalized_text(text);
    normalized.len() >= 80
        && ![
            "todo",
            "tbd",
            "placeholder",
            "generic filler",
            "material for",
        ]
        .iter()
        .any(|needle| normalized.to_lowercase().contains(needle))
}

fn labeled_value_is_substantive(section: &str, label: &str) -> bool {
    section.split_once(label).is_some_and(|(_, suffix)| {
        let value = suffix.split(['\n', ';']).next().unwrap_or_default().trim();
        value.len() >= 24 && !value.eq_ignore_ascii_case("recorded verbatim")
    })
}

fn public_receipt_row(spec: CommonSpec) -> &'static Value {
    public_receipt()
        .as_array()
        .expect("public common compiler receipt rows")
        .iter()
        .find(|row| row["name"].as_str() == Some(spec.public_declaration))
        .unwrap_or_else(|| panic!("missing public compiler receipt for {}", spec.item_id))
}

fn completed_formalization(spec: CommonSpec) -> (&'static str, Option<&'static str>) {
    match spec.item_id {
        "CFT-25-002" => (
            "reexported-proof",
            Some("CrouzeixConjecture.convexProjection_variational"),
        ),
        "CFT-25-003" => (
            "reexported-proof",
            Some("CrouzeixConjecture.norm_convexProjection_sub_le"),
        ),
        "CFT-25-006" => (
            "reexported-proof",
            Some("CrouzeixConjecture.canonicalParallelOrientedRadialBoundaryStatement"),
        ),
        "CFT-26-001" => ("proved-here", None),
        "CFT-26-002" => (
            "reexported-proof",
            Some("CrouzeixConjecture.scalar_sub_matrix_isUnit_of_outwardBoundarySupport"),
        ),
        "CFT-26-003" => (
            "reexported-proof",
            Some(
                "CrouzeixConjecture.doubleLayerSupportMatrix_posSemidef_of_outwardBoundarySupport",
            ),
        ),
        "CFT-26-004" => (
            "definition",
            Some("CrouzeixConjecture.doubleLayerResolvent"),
        ),
        "CFT-26-005" => (
            "reexported-proof",
            Some("CrouzeixConjecture.doubleLayerResolvent_congruence_density"),
        ),
        _ => ("proved-here", None),
    }
}

fn canonical_card_errors(markdown: &str, spec: CommonSpec) -> Vec<String> {
    let mut errors = Vec::new();
    let anchor = format!("{{#{}}}", spec.anchor);
    if markdown.matches(&anchor).count() != 1 {
        errors.push(format!(
            "{} canonical theorem card must have exactly one `{anchor}`",
            spec.item_id
        ));
    }
    let Some(card) = card_region(markdown, spec) else {
        errors.push(format!(
            "{} canonical theorem card must occur exactly once",
            spec.item_id
        ));
        return errors;
    };
    if !card.lines().next().is_some_and(|heading| {
        heading.starts_with(&format!("### {} ", spec.item_id)) && heading.ends_with(&anchor)
    }) {
        errors.push(format!(
            "{} canonical theorem card heading does not bind its stable anchor",
            spec.item_id
        ));
    }

    let mut previous = 0;
    for field in CARD_FIELDS {
        let marker = format!("#### {field}\n");
        let positions = card.match_indices(&marker).collect::<Vec<_>>();
        if positions.len() != 1 {
            errors.push(format!(
                "{} canonical theorem card must contain exactly one `{field}` section",
                spec.item_id
            ));
            continue;
        }
        if positions[0].0 < previous {
            errors.push(format!(
                "{} canonical theorem card section `{field}` is out of order",
                spec.item_id
            ));
        }
        previous = positions[0].0;
        if !section_content(card, field).is_some_and(substantive) {
            errors.push(format!(
                "{} canonical theorem card section `{field}` is filler or not substantive",
                spec.item_id
            ));
        }
    }

    let proof = section_content(card, "Proof").unwrap_or_default();
    if !proof
        .to_lowercase()
        .contains(&spec.obligation.to_lowercase())
    {
        errors.push(format!(
            "{} proof does not discharge its row-specific obligation `{}`",
            spec.item_id, spec.obligation
        ));
    }
    if !(proof.contains("$$") || proof.contains("\\["))
        || !["=", "≤", "≥", "\\le", "\\ge", "\\to", "→"]
            .iter()
            .any(|symbol| proof.contains(symbol))
        || ["$$x=x$$", "$$0=0$$", "\\[x=x\\]", "\\[0=0\\]"]
            .iter()
            .any(|placeholder| proof.replace(' ', "").contains(placeholder))
    {
        errors.push(format!(
            "{} canonical theorem card proof omits a displayed load-bearing equation",
            spec.item_id
        ));
    }
    let history = section_content(card, "Historical context").unwrap_or_default();
    if !labeled_value_is_substantive(history, "Source boundary:")
        || !labeled_value_is_substantive(history, "Review status:")
    {
        errors.push(format!(
            "{} historical context must label `Source boundary:` and `Review status:`",
            spec.item_id
        ));
    }
    let ml = section_content(card, "ML analogy").unwrap_or_default();
    for label in [
        "Mathematical object:",
        "ML counterpart:",
        "Exact transfer:",
        "Non-transfer:",
        "Diagnostic:",
    ] {
        if !labeled_value_is_substantive(ml, label) {
            errors.push(format!(
                "{} ML analogy omits a substantive `{label}` value",
                spec.item_id
            ));
        }
    }
    let lean = section_content(card, "Lean correspondence").unwrap_or_default();
    let (mode, underlying) = completed_formalization(spec);
    if !lean.contains(&format!("Formal mode: `{mode}`.")) {
        errors.push(format!(
            "{} Lean correspondence omits truthful formal mode `{mode}`",
            spec.item_id
        ));
    }
    if let Some(underlying) = underlying {
        if !lean.contains(&format!("Underlying declaration: `{underlying}`.")) {
            errors.push(format!(
                "{} Lean correspondence omits underlying provider `{underlying}`",
                spec.item_id
            ));
        }
    }
    let lean = section_content(card, "Lean correspondence").unwrap_or_default();
    for required in [
        "Public declaration:",
        "Substantive provider:",
        "Readable type map:",
        "Code:",
        "Compiler receipt:",
        "Normalized type:",
        "Type SHA-256:",
        "Direct maintained dependencies:",
        "Axioms:",
        "Assumptions:",
        "Verification target:",
        "Receipt identity:",
    ] {
        if !lean.contains(required) {
            errors.push(format!(
                "{} Lean correspondence omits `{required}`",
                spec.item_id
            ));
        }
    }
    let receipt = public_receipt_row(spec);
    let normalized_type = receipt["normalized_type"].as_str().unwrap_or_default();
    let dependencies = receipt["direct_dependencies"]
        .as_array()
        .expect("public receipt dependencies")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    let dependencies_display = if dependencies.is_empty() {
        "none"
    } else {
        dependencies.as_str()
    };
    let axioms = receipt["axioms"]
        .as_array()
        .expect("public receipt axioms")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    let receipt_line = receipt["line"].as_u64().expect("public receipt line");
    let type_sha256 = receipt["type_sha256"]
        .as_str()
        .expect("public receipt type hash");
    for exact in [
        format!("Public declaration: `{}`", spec.public_declaration),
        format!("Substantive provider: `{dependencies_display}`"),
        format!("{}#L{receipt_line}", spec.source_path),
        format!("Normalized type: `{normalized_type}`"),
        format!("Type SHA-256: `{type_sha256}`"),
        format!("Direct maintained dependencies: `{dependencies_display}`"),
        format!("Axioms: `{axioms}`"),
        "Verification target: `CrouzeixTextbook`".to_owned(),
        format!("Receipt identity: `CrouzeixTextbook:{type_sha256}`"),
    ] {
        if !lean.contains(&exact) {
            errors.push(format!(
                "{} Lean correspondence does not match compiler field `{exact}`",
                spec.item_id
            ));
        }
    }
    let exercise = exercise_id(spec);
    if !section_content(card, "Exercises and solutions")
        .is_some_and(|section| section.contains(&exercise))
    {
        errors.push(format!(
            "{} theorem card does not bind exercise {exercise}",
            spec.item_id
        ));
    }
    errors
}

fn canonical_chapter_errors(chapter: u64) -> Vec<String> {
    let markdown = chapter_markdown(chapter);
    let card_count = markdown
        .lines()
        .filter(|line| line.starts_with(&format!("### CFT-{chapter:02}-")) && !line.contains("-E"))
        .count();
    let mut errors = Vec::new();
    if card_count != 6 {
        errors.push(format!(
            "Chapter {chapter} must contain exactly six canonical theorem cards, observed {card_count}"
        ));
    }
    for spec in COMMON_SPECS
        .into_iter()
        .filter(|spec| spec.chapter == chapter)
    {
        errors.extend(canonical_card_errors(&markdown, spec));
    }
    errors
}

fn phase_state_errors(
    coverage: &Value,
    exercises: &Value,
    receipt: Option<&Value>,
    phase: Wave4Phase,
) -> Vec<String> {
    let coverage_rows = rows_for_chapters(coverage, "items");
    let exercise_rows = rows_for_chapters(exercises, "exercises");
    let mut errors = Vec::new();
    for chapter in 25..=29 {
        if phase.completes(chapter) {
            errors.extend(canonical_chapter_errors(chapter));
            let markdown = chapter_markdown(chapter);
            for spec in EXERCISE_SPECS
                .into_iter()
                .filter(|spec| common_spec(spec.parent).chapter == chapter)
            {
                errors.extend(exercise_prose_errors(&markdown, spec));
            }
        }
    }
    for spec in COMMON_SPECS {
        let matches = coverage_rows
            .iter()
            .filter(|row| row["item_id"].as_str() == Some(spec.item_id))
            .copied()
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            errors.push(format!(
                "{} does not have exactly one coverage row",
                spec.item_id
            ));
            continue;
        }
        let row = matches[0];
        if phase == Wave4Phase::Wave4Complete
            && row["review_status"]["status"].as_str() != Some("content-and-formal-review-complete")
        {
            errors.push(format!(
                "{} lacks completed independent content and formal review evidence",
                spec.item_id
            ));
        }
        if row["anchor"].as_str() != Some(spec.anchor)
            || row["lean_declaration"]["name"].as_str() != Some(spec.public_declaration)
            || row["lean_declaration"]["source_path"].as_str() != Some(spec.source_path)
        {
            errors.push(format!(
                "{} changed its stable public mapping",
                spec.item_id
            ));
        }
        let completed = phase.completes(spec.chapter);
        let expected_proof = if completed && spec.item_id == "CFT-28-006" {
            "summary"
        } else if completed {
            "reconstructible"
        } else {
            spec.baseline_proof
        };
        let expected_lean = if completed {
            "exact"
        } else {
            spec.baseline_correspondence
        };
        if row["prose_proof_status"].as_str() != Some(expected_proof) {
            errors.push(format!(
                "{} prose status is not {expected_proof}",
                spec.item_id
            ));
        }
        if row["lean_correspondence_status"].as_str() != Some(expected_lean) {
            errors.push(format!(
                "{} correspondence status is not {expected_lean}",
                spec.item_id
            ));
        }
        if completed {
            let compiler_row = public_receipt_row(spec);
            let declaration = &row["lean_declaration"];
            let (formal_mode, underlying) = completed_formalization(spec);
            if row["formal_mode"].as_str() != Some(formal_mode)
                || declaration["underlying_declaration"].as_str() != underlying
            {
                errors.push(format!(
                    "{} does not record its truthful `{formal_mode}` mode and underlying provider",
                    spec.item_id
                ));
            }
            let expected_kind = if underlying.is_some() {
                "direct-alias"
            } else {
                "theorem"
            };
            if compiler_row["kind"].as_str() != Some(expected_kind) {
                errors.push(format!(
                    "{} compiler kind does not match formal mode `{formal_mode}`",
                    spec.item_id
                ));
            }
            if let Some(underlying) = underlying {
                let direct_dependencies = compiler_row["direct_dependencies"]
                    .as_array()
                    .expect("completed public direct dependencies");
                if direct_dependencies.as_slice() != [serde_json::json!(underlying)] {
                    errors.push(format!(
                        "{} compiler alias does not target exact underlying provider `{underlying}`",
                        spec.item_id
                    ));
                }
            }
            if declaration["source_path"] != compiler_row["source_path"]
                || declaration["line"] != compiler_row["line"]
                || declaration["column"] != compiler_row["column"]
                || declaration["type_sha256"] != compiler_row["type_sha256"]
                || declaration["axioms"] != compiler_row["axioms"]
                || compiler_row["axioms"] != serde_json::json!(AXIOMS)
            {
                errors.push(format!(
                    "{} completed coverage row is not joined to its exact compiler locator, fingerprint, and approved axioms",
                    spec.item_id
                ));
            }
            if spec.item_id == "CFT-25-006" {
                let expected = radial_completion_receipt()
                    .as_array()
                    .expect("radial completion receipt rows")
                    .first()
                    .expect("radial completion receipt row");
                let provider =
                    "CrouzeixConjecture.canonicalParallelOrientedRadialBoundaryStatement";
                let dependencies = compiler_row["direct_dependencies"]
                    .as_array()
                    .expect("CFT-25-006 compiler dependencies");
                if compiler_row["type_sha256"] != expected["type_sha256"]
                    || !compiler_row["normalized_type"]
                        .as_str()
                        .is_some_and(|lean_type| {
                            lean_type.contains("CanonicalParallelOrientedRadialBoundaryStatement")
                        })
                    || !dependencies
                        .iter()
                        .any(|dependency| dependency.as_str() == Some(provider))
                {
                    errors.push(
                        "CFT-25-006 completion must be the canonical positively oriented radial C¹ boundary package and depend on its exact provider"
                            .to_owned(),
                    );
                }
            }
            if spec.item_id == "CFT-26-006" {
                let dependencies = compiler_row["direct_dependencies"]
                    .as_array()
                    .expect("CFT-26-006 compiler dependencies");
                let required = [
                    "CrouzeixConjecture.integrable_parametricDoubleLayerDensity",
                    "CrouzeixConjecture.integral_posSemidef",
                    "CrouzeixConjecture.parametricDoubleLayerDensity_posSemidef",
                ];
                let normalized = compiler_row["normalized_type"].as_str().unwrap_or_default();
                if !normalized.contains("MeasureTheory.Integrable")
                    || !normalized.contains("Matrix.PosSemidef")
                    || !normalized.contains("∀ (a b : n)")
                    || required.iter().any(|provider| {
                        !dependencies
                            .iter()
                            .any(|dependency| dependency.as_str() == Some(provider))
                    })
                {
                    errors.push(
                        "CFT-26-006 completion must combine integrability, integrated positive semidefiniteness, and entrywise finite-dimensional integral interchange"
                            .to_owned(),
                    );
                }
            }
            if spec.item_id == "CFT-26-001" {
                let dependencies = compiler_row["direct_dependencies"]
                    .as_array()
                    .expect("CFT-26-001 compiler dependencies");
                let normalized = compiler_row["normalized_type"].as_str().unwrap_or_default();
                let required = [
                    "CrouzeixConjecture.OutwardBoundarySupport.sigma_not_mem",
                    "CrouzeixConjecture.OutwardBoundarySupport.support_inequality",
                ];
                if !normalized.contains("And")
                    || !normalized.contains("Not (Membership.mem")
                    || !normalized.contains("∀ (x : CrouzeixConjecture.EuclideanVector")
                    || !normalized.contains("Norm.norm")
                    || !normalized.contains("RCLike.re")
                    || required.iter().any(|provider| {
                        !dependencies
                            .iter()
                            .any(|dependency| dependency.as_str() == Some(provider))
                    })
                    || dependencies.iter().any(|dependency| {
                        dependency.as_str()
                            == Some("CrouzeixConjecture.OutwardBoundarySupport.sigma_not_mem_numericalRange")
                    })
                {
                    errors.push(
                        "CFT-26-001 completion must prove both boundary exclusion and the unit-vector scalar support inequality directly from the support datum"
                            .to_owned(),
                    );
                }
            }
        }

        let expected_exercise = exercise_id(spec);
        let matches = exercise_rows
            .iter()
            .filter(|row| row["exercise_id"].as_str() == Some(expected_exercise.as_str()))
            .copied()
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            errors.push(format!(
                "{expected_exercise} does not have exactly one exercise row"
            ));
            continue;
        }
        let exercise_row = matches[0];
        let expected_anchor = format!("exercise-{}", expected_exercise.to_lowercase());
        if exercise_row["chapter"].as_u64() != Some(spec.chapter)
            || exercise_row["anchor"].as_str() != Some(expected_anchor.as_str())
            || exercise_row["skills"] != serde_json::json!([spec.item_id])
            || !exercise_row["starter"].is_null()
        {
            errors.push(format!(
                "{expected_exercise} changed its canonical chapter, anchor, parent skill, or starter metadata"
            ));
        }
        let solution = &exercise_row["lean_solution"];
        if completed && solution.is_null() {
            errors.push(format!(
                "{expected_exercise} lacks its distinct compiled solution"
            ));
        } else if !completed && !solution.is_null() {
            errors.push(format!(
                "{expected_exercise} claims a solution before its chapter phase"
            ));
        }
    }
    errors.extend(exercise_receipt_errors(coverage, exercises, receipt, phase));
    errors
}

fn common_build_once(
    result: &OnceLock<Result<(), String>>,
    command: &mut Command,
) -> Result<(), String> {
    result
        .get_or_init(|| {
            let output = command
                .output()
                .map_err(|error| format!("run canonical textbook build: {error}"))?;
            if !output.status.success() {
                return Err(format!(
                    "canonical textbook build failed ({}):\n{}\n{}",
                    output.status,
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
            Ok(())
        })
        .clone()
}

fn assert_common_lean_artifacts_fresh() {
    static BUILD: OnceLock<Result<(), String>> = OnceLock::new();
    let mut command =
        super::textbook_lean_command(workspace_root().join("scripts/check_lean_library.sh"));
    command
        .arg("CrouzeixTextbook")
        .current_dir(workspace_root());
    common_build_once(&BUILD, &mut command).unwrap_or_else(|error| panic!("{error}"));
}

fn compile_canonical_receipt() -> Value {
    assert_common_lean_artifacts_fresh();
    let temp = tempfile::tempdir().expect("canonical compiler receipt root");
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .expect("private canonical compiler receipt root");
    let receipt_path = temp.path().join("receipt.json");
    let output = super::textbook_lean_command("lake")
        .args(["env", "lean", "--run", "CrouzeixTextbook.lean"])
        .arg(&receipt_path)
        .current_dir(workspace_root().join("formalization/lean"))
        .output()
        .expect("run canonical textbook compiler receipt");
    assert!(
        output.status.success(),
        "canonical compiler receipt failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    read_json(&receipt_path)
}

fn canonical_receipt() -> &'static Value {
    static RECEIPT: OnceLock<Value> = OnceLock::new();
    RECEIPT.get_or_init(compile_canonical_receipt)
}

fn canonicalize_receipt_rows(mut value: Value) -> Value {
    let canonical = canonical_receipt()["declarations"]
        .as_array()
        .expect("canonical receipt declarations");
    let rows = value.as_array_mut().expect("compiler receipt rows");
    for row in rows {
        let Some(name) = row["name"].as_str() else {
            continue;
        };
        let is_completed_chapter = COMMON_SPECS
            .iter()
            .any(|spec| spec.chapter <= 29 && spec.public_declaration == name);
        if is_completed_chapter {
            if let Some(exact) = canonical
                .iter()
                .find(|candidate| candidate["name"].as_str() == Some(name))
            {
                *row = exact.clone();
            }
        }
    }
    value
}

fn compile_public_receipt() -> Value {
    assert_common_lean_artifacts_fresh();
    let temp = tempfile::tempdir().expect("common compiler probe root");
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .expect("private common compiler probe root");
    let source_path = temp.path().join("CommonMachineryContractProbe.lean");
    let names = COMMON_SPECS
        .iter()
        .map(|spec| format!("`{}", spec.public_declaration))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        "import CrouzeixTextbook.ExportReceipt\n\
         open Lean\n\
         run_cmd do\n\
           let env ← getEnv\n\
           let names : Array Name := #[{names}]\n\
           let rows ← Lean.Elab.Command.liftCoreM <|\n\
             CrouzeixTextbook.ExportReceipt.receiptRows env names\n\
           IO.println s!\"COMMON_MACHINERY_RECEIPT:{{(Json.arr rows).compress}}\"\n"
    );
    fs::write(&source_path, source).expect("write common compiler probe");
    let output = super::textbook_lean_command("lake")
        .arg("env")
        .arg("lean")
        .arg(&source_path)
        .current_dir(workspace_root().join("formalization/lean"))
        .output()
        .expect("run common compiler probe");
    assert!(
        output.status.success(),
        "common compiler probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("common compiler probe UTF-8");
    let payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("COMMON_MACHINERY_RECEIPT:"))
        .expect("common compiler probe marker");
    canonicalize_receipt_rows(serde_json::from_str(payload).expect("common compiler probe JSON"))
}

fn compile_supporting_receipt(name: &str) -> Value {
    assert_common_lean_artifacts_fresh();
    let temp = tempfile::tempdir().expect("supporting theorem compiler probe root");
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .expect("private supporting theorem compiler probe root");
    let source_path = temp.path().join("SupportingTheoremContractProbe.lean");
    let source = format!(
        "import CrouzeixTextbook.ExportReceipt\n\
         open Lean\n\
         run_cmd do\n\
           let env ← getEnv\n\
           let rows ← Lean.Elab.Command.liftCoreM <|\n\
             CrouzeixTextbook.ExportReceipt.receiptRows env #[`{name}]\n\
           IO.println s!\"SUPPORTING_THEOREM_RECEIPT:{{(Json.arr rows).compress}}\"\n"
    );
    fs::write(&source_path, source).expect("write supporting theorem compiler probe");
    let output = super::textbook_lean_command("lake")
        .arg("env")
        .arg("lean")
        .arg(&source_path)
        .current_dir(workspace_root().join("formalization/lean"))
        .output()
        .expect("run supporting theorem compiler probe");
    assert!(
        output.status.success(),
        "supporting theorem compiler probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("supporting theorem probe UTF-8");
    let payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("SUPPORTING_THEOREM_RECEIPT:"))
        .expect("supporting theorem compiler probe marker");
    serde_json::from_str(payload).expect("supporting theorem compiler probe JSON")
}

fn compile_public_provider_receipt() -> Value {
    assert_common_lean_artifacts_fresh();
    let temp = tempfile::tempdir().expect("common provider compiler probe root");
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .expect("private common provider compiler probe root");
    let source_path = temp.path().join("CommonProviderContractProbe.lean");
    let roots = COMMON_SPECS
        .iter()
        .map(|spec| format!("`{}", spec.public_declaration))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        r#"import CrouzeixTextbook.ExportReceipt
open Lean
partial def dependencyClosure (env : Environment) : List Name → NameHashSet → NameHashSet
  | [], seen => seen
  | name :: pending, seen =>
      if seen.contains name then dependencyClosure env pending seen
      else
        let seen := seen.insert name
        match env.find? name with
        | none => dependencyClosure env pending seen
        | some info =>
            let dependencies := CrouzeixTextbook.ExportReceipt.directConstants info
            dependencyClosure env (dependencies.toList ++ pending) seen
def dependencyRow (env : Environment) (name : Name) : Json :=
  match env.find? name with
  | none => json% {{ name: $(name.toString), direct_dependencies: [], body_dependencies: [] }}
  | some info =>
      let directDependencies := CrouzeixTextbook.ExportReceipt.directConstants info
        |>.filter (· != name)
      let bodyDependencies := CrouzeixTextbook.ExportReceipt.bodyConstants info
        |>.filter (· != name)
      let directJson := directDependencies.map (Json.str ·.toString)
      let bodyJson := bodyDependencies.map (Json.str ·.toString)
      json% {{
        name: $(name.toString),
        direct_dependencies: $(directJson),
        body_dependencies: $(bodyJson)
      }}
run_cmd do
  let env ← getEnv
  let roots : Array Name := #[{roots}]
  let names := (dependencyClosure env roots.toList {{}}).toArray
    |>.filter CrouzeixTextbook.ExportReceipt.isMaintainedName
    |>.qsort (fun left right => left.toString < right.toString)
  let rows := names.map (dependencyRow env)
  let receipt := json% {{
    target: "CrouzeixTextbook",
    declarations: $(rows)
  }}
  IO.println s!"COMMON_PROVIDER_RECEIPT:{{receipt.compress}}"
"#
    );
    fs::write(&source_path, source).expect("write common provider compiler probe");
    let output = super::textbook_lean_command("lake")
        .arg("env")
        .arg("lean")
        .arg(&source_path)
        .current_dir(workspace_root().join("formalization/lean"))
        .output()
        .expect("run common provider compiler probe");
    assert!(
        output.status.success(),
        "common provider compiler probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("common provider probe UTF-8");
    let payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("COMMON_PROVIDER_RECEIPT:"))
        .expect("common provider compiler probe marker");
    serde_json::from_str(payload).expect("common provider compiler probe JSON")
}

fn compile_radial_completion_receipt() -> Value {
    assert_common_lean_artifacts_fresh();
    let temp = tempfile::tempdir().expect("radial completion compiler probe root");
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .expect("private radial completion compiler probe root");
    let source_path = temp.path().join("RadialCompletionContractProbe.lean");
    let source = r#"import CrouzeixTextbook.ExportReceipt
open Lean
run_cmd do
  let env ← getEnv
  let names : Array Name := #[`CrouzeixConjecture.canonicalParallelOrientedRadialBoundaryStatement]
  let rows ← Lean.Elab.Command.liftCoreM <|
    CrouzeixTextbook.ExportReceipt.receiptRows env names
  IO.println s!"RADIAL_COMPLETION_RECEIPT:{(Json.arr rows).compress}"
"#;
    fs::write(&source_path, source).expect("write radial completion compiler probe");
    let output = super::textbook_lean_command("lake")
        .arg("env")
        .arg("lean")
        .arg(&source_path)
        .current_dir(workspace_root().join("formalization/lean"))
        .output()
        .expect("run radial completion compiler probe");
    assert!(
        output.status.success(),
        "radial completion compiler probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("radial completion probe UTF-8");
    let payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("RADIAL_COMPLETION_RECEIPT:"))
        .expect("radial completion compiler probe marker");
    serde_json::from_str(payload).expect("radial completion compiler probe JSON")
}

fn radial_completion_receipt() -> &'static Value {
    static RECEIPT: OnceLock<Value> = OnceLock::new();
    RECEIPT.get_or_init(compile_radial_completion_receipt)
}

fn compile_exercise_receipt(phase: Wave4Phase) -> Result<Value, String> {
    assert_common_lean_artifacts_fresh();
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    let source_path = temp.path().join("CommonExerciseReceipt.lean");
    let roots = EXERCISE_SPECS
        .iter()
        .copied()
        .filter(|spec| phase.completes(common_spec(spec.parent).chapter))
        .map(|spec| format!("`{}", exercise_declaration_for(spec)))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        r#"import CrouzeixTextbook.ExportReceipt
open Lean MeasureTheory
open scoped ComplexConjugate ComplexOrder InnerProductSpace Matrix Matrix.Norms.L2Operator
partial def dependencyClosure (env : Environment) : List Name → NameHashSet → NameHashSet
  | [], seen => seen
  | name :: pending, seen =>
      if seen.contains name then dependencyClosure env pending seen
      else
        let seen := seen.insert name
        match env.find? name with
        | none => dependencyClosure env pending seen
        | some info =>
            let dependencies := CrouzeixTextbook.ExportReceipt.directConstants info
            dependencyClosure env (dependencies.toList ++ pending) seen
def bodyDependencyRow (env : Environment) (name : Name) : Json :=
  match env.find? name with
  | none => json% {{ name: $(name.toString), body_dependencies: [] }}
  | some info =>
      let dependencies := CrouzeixTextbook.ExportReceipt.bodyConstants info
        |>.filter (· != name)
      let dependencyJson := dependencies.map (Json.str ·.toString)
      json% {{ name: $(name.toString), body_dependencies: $(dependencyJson) }}
run_cmd do
  let env ← getEnv
  let roots : Array Name := #[{roots}]
  for name in roots do
    unless CrouzeixTextbook.ExportReceipt.checkedExerciseTheoremNames.contains name do
      throwError "completed common exercise is absent from checkedExerciseTheoremNames: {{name}}"
  let allNames := (dependencyClosure env roots.toList {{}}).toArray
    |>.filter CrouzeixTextbook.ExportReceipt.isMaintainedName
    |>.qsort (fun left right => left.toString < right.toString)
  let names := allNames
    |>.filter (fun name => CrouzeixTextbook.ExportReceipt.sourceLocation env name |>.isSome)
  let rows ← Lean.Elab.Command.liftCoreM <|
    CrouzeixTextbook.ExportReceipt.receiptRows env names
  let bodyRows := allNames.map (bodyDependencyRow env)
  let receipt := json% {{
    target: "CrouzeixTextbook",
    declarations: $(rows),
    body_dependency_rows: $(bodyRows)
  }}
  IO.println s!"COMMON_EXERCISE_RECEIPT:{{receipt.compress}}"
"#
    );
    fs::write(&source_path, source).map_err(|error| error.to_string())?;
    let output = super::textbook_lean_command("lake")
        .arg("env")
        .arg("lean")
        .arg(&source_path)
        .current_dir(workspace_root().join("formalization/lean"))
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(format!(
            "fresh common exercise compiler receipt failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("COMMON_EXERCISE_RECEIPT:"))
        .ok_or_else(|| "common exercise compiler receipt omitted its marker".to_owned())?;
    serde_json::from_str(payload)
        .map(attach_body_dependencies)
        .map(|mut receipt| {
            let rows = receipt["declarations"]
                .as_array_mut()
                .expect("exercise receipt declarations");
            let canonical = canonical_receipt()["declarations"]
                .as_array()
                .expect("canonical receipt declarations");
            for row in rows {
                let body = row["body_dependencies"].clone();
                let planned_type_sha256 = row["type_sha256"].clone();
                let Some(name) = row["name"].as_str() else {
                    continue;
                };
                if let Some(exact) = canonical
                    .iter()
                    .find(|candidate| candidate["name"].as_str() == Some(name))
                {
                    *row = exact.clone();
                    row["body_dependencies"] = body;
                    row["planned_type_sha256"] = planned_type_sha256;
                }
            }
            receipt
        })
        .map_err(|error| error.to_string())
}

fn public_receipt() -> &'static Value {
    static RECEIPT: OnceLock<Value> = OnceLock::new();
    RECEIPT.get_or_init(compile_public_receipt)
}

fn public_provider_receipt() -> &'static Value {
    static RECEIPT: OnceLock<Value> = OnceLock::new();
    RECEIPT.get_or_init(compile_public_provider_receipt)
}

fn graph_from_coverage(coverage: &Value) -> BTreeMap<String, BTreeSet<String>> {
    array(coverage, "items", "coverage")
        .iter()
        .map(|row| {
            (
                string(row, "item_id", "coverage row").to_owned(),
                array(row, "pedagogical_prerequisites", "coverage row")
                    .iter()
                    .map(|value| value.as_str().expect("prerequisite string").to_owned())
                    .collect(),
            )
        })
        .collect()
}

fn reaches(
    graph: &BTreeMap<String, BTreeSet<String>>,
    start: &str,
    target: &str,
    visited: &mut BTreeSet<String>,
) -> bool {
    if !visited.insert(start.to_owned()) {
        return false;
    }
    start == target
        || graph.get(start).is_some_and(|next| {
            next.iter()
                .any(|node| reaches(graph, node, target, visited))
        })
}

fn graph_is_acyclic(graph: &BTreeMap<String, BTreeSet<String>>) -> bool {
    graph.keys().all(|node| {
        graph.get(node).is_none_or(|next| {
            next.iter()
                .all(|dependency| !reaches(graph, dependency, node, &mut BTreeSet::new()))
        })
    })
}

fn branches_are_independent(graph: &BTreeMap<String, BTreeSet<String>>) -> bool {
    let jin = (30..=32)
        .flat_map(|chapter| (1..=6).map(move |index| format!("CFT-{chapter:02}-{index:03}")))
        .collect::<Vec<_>>();
    let ls = (33..=34)
        .flat_map(|chapter| (1..=6).map(move |index| format!("CFT-{chapter:02}-{index:03}")))
        .collect::<Vec<_>>();
    jin.iter().all(|jin_node| {
        ls.iter().all(|ls_node| {
            !reaches(graph, jin_node, ls_node, &mut BTreeSet::new())
                && !reaches(graph, ls_node, jin_node, &mut BTreeSet::new())
        })
    })
}

fn run_common_closure(lean_root: &Path, roots: &[&str]) -> Output {
    let script = r#"
import json, sys
from pathlib import Path
repo = Path(sys.argv[1])
sys.path.insert(0, str(repo / 'labs' / 'crouzeix_proof_reproduction'))
import proof_evidence
proof_evidence.MANAGED_LOCAL_PREFIXES = (*proof_evidence.MANAGED_LOCAL_PREFIXES, 'CrouzeixTextbook')
modules = set()
try:
    for root in sys.argv[3].split(','):
        policy = proof_evidence.RoutePolicy(
            route_id='common-machinery', aggregate_module=root,
            allowed_exact=(root,),
            allowed_prefixes=('CrouzeixTextbook.Part05', 'CrouzeixTextbook.Part06.Chapter29', 'CrouzeixConjecture'),
            rejected_prefixes=('Crouzeix.Jin', 'Crouzeix.LoristSchwenninger', 'Crouzeix.Harp'),
            rejected_exact=('CrouzeixJin', 'CrouzeixLoristSchwenninger', 'CrouzeixHarp'))
        modules.update(proof_evidence.gather_route_closure(
            Path(sys.argv[2]), policy, cache_identity='wave4-contract-test',
            toolchain=proof_evidence.PINNED_TOOLCHAIN))
except proof_evidence.PreflightError as error:
    print(error.reason, file=sys.stderr)
    raise SystemExit(7)
print(json.dumps(sorted(modules)))
"#;
    Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(workspace_root())
        .arg(lean_root)
        .arg(roots.join(","))
        .output()
        .expect("run common structural closure")
}

fn write_module(root: &Path, module: &str, source: &str) {
    let path = root.join(format!("{}.lean", module.replace('.', "/")));
    fs::create_dir_all(path.parent().expect("module parent")).expect("module directory");
    fs::write(path, source).expect("test module source");
}

fn future_card(spec: CommonSpec) -> String {
    let receipt = public_receipt_row(spec);
    let dependencies = receipt["direct_dependencies"]
        .as_array()
        .expect("public receipt dependencies")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    let dependencies_display = if dependencies.is_empty() {
        "none"
    } else {
        dependencies.as_str()
    };
    let axioms = receipt["axioms"]
        .as_array()
        .expect("public receipt axioms")
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    let normalized_type = receipt["normalized_type"].as_str().expect("public type");
    let type_sha256 = receipt["type_sha256"].as_str().expect("public hash");
    let line = receipt["line"].as_u64().expect("public line");
    let (mode, underlying) = completed_formalization(spec);
    let underlying = underlying
        .map(|name| format!(" Underlying declaration: `{name}`."))
        .unwrap_or_default();
    let mut card = format!("### {} {{#{}}}\n", spec.item_id, spec.anchor);
    for field in CARD_FIELDS {
        let content = match field {
            "Proof" => format!(
                "We carry out the row-specific step: {}. The load-bearing calculation is $$\\operatorname{{commonStep}}_{{{}}}(input) \\le output,$$ and every inference needed to reach the stated conclusion is recorded here.",
                spec.obligation, spec.item_id
            ),
            "Historical context" => format!(
                "Source boundary: the cited historical result supplies only the named antecedent and no textbook reconstruction.\n\nReview status: this reconstruction of {} is independently checked against the primary source and compiler receipt.",
                spec.obligation
            ),
            "ML analogy" => format!(
                "Mathematical object: the exact mathematical object used to {}.\n\nML counterpart: a controlled operator component in a nonnormal learned dynamics model.\n\nExact transfer: the displayed norm inequality transfers without changing its hypotheses.\n\nNon-transfer: the geometric regularity has no automatic statistical-learning interpretation.\n\nDiagnostic: test the displayed residual numerically before treating the analogy as predictive.",
                spec.obligation
            ),
            "Lean correspondence" => format!(
                "Public declaration: `{}`. Formal mode: `{mode}`.{underlying} Substantive provider: `{dependencies_display}`. Readable type map: every binder is matched to the mathematical statement. Code: [{}]({}#L{line}). Compiler receipt: fresh canonical run. Normalized type: `{normalized_type}`. Type SHA-256: `{type_sha256}`. Direct maintained dependencies: `{dependencies_display}`. Axioms: `{axioms}`. Assumptions: every explicit and typeclass assumption is audited. Verification target: `CrouzeixTextbook`. Receipt identity: `CrouzeixTextbook:{type_sha256}`.",
                spec.public_declaration,
                spec.source_path,
                spec.source_path
            ),
            "Exercises and solutions" => format!(
                "{} reconstructs the card without calling its public theorem or substantive provider. Its complete written derivation and distinct compiler-checked Lean theorem are both linked here.",
                exercise_id(spec)
            ),
            _ => format!(
                "This section develops the card-specific obligation to {}. It states the relevant objects and explains why this exact step is necessary for the common Crouzeix machinery.",
                spec.obligation
            ),
        };
        card.push_str(&format!("\n#### {field}\n\n{content}\n"));
    }
    card
}

fn future_card_errors(card: &str, spec: CommonSpec) -> Vec<String> {
    canonical_card_errors(card, spec)
}

#[test]
fn common_machinery_contract_exists_and_freezes_the_truthful_phase() {
    let contract = wave4_contract();
    let normalized = normalized_text(&contract);
    let phase = active_phase(&contract);
    assert_eq!(phase, Wave4Phase::Wave4Complete);
    assert!(contract.contains("- Target phase: `wave-4-complete`."));
    for candidate in Wave4Phase::ALL {
        assert!(contract.contains(&format!("`{}`", candidate.label())));
    }
    for required in [
        "Wave 4 common trunk independently reviewed and published",
        "Active phase: `wave-4-complete`.",
        "Current exact correspondence rows: `30`.",
        "Current distinct checked exercise solutions: `30`.",
        "Chapters 25--29 are promoted by this phase.",
        "Promotion requires a fresh compiler receipt",
        "CFT-27-001` is a theorem",
        "Independent Wave 4 review record",
        "Review result: no blocking mathematical, formal-correspondence, or publication findings.",
        "does not change CFT-28-006 from its truthful `summary` status.",
        "Harp's finite-horizon route may reuse lower-level Lorist--Schwenninger modules",
    ] {
        assert!(normalized.contains(required), "contract omits `{required}`");
    }

    let coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let compiler_receipt = (phase != Wave4Phase::ContractFrozen).then(|| {
        compile_exercise_receipt(phase)
            .unwrap_or_else(|error| panic!("completed phase lacks fresh exercise receipt: {error}"))
    });
    let errors = phase_state_errors(&coverage, &exercises, compiler_receipt.as_ref(), phase);
    assert!(
        errors.is_empty(),
        "active phase is not truthful: {errors:#?}"
    );

    let premature = phase_state_errors(&coverage, &exercises, None, Wave4Phase::Chapter26Complete);
    assert!(
        premature
            .iter()
            .any(|error| error.contains("fresh canonical compiler receipt row")),
        "completed metadata without a fresh compiler receipt was not rejected: {premature:#?}"
    );

    assert!(contract.contains("positively oriented radial C¹ boundary package"));
    assert!(contract.contains("Expected reconstruction"));
    assert!(contract.contains("Compiler-comparable planned propositions"));
    assert!(contract.contains("reconstruction targets"));
}

#[test]
fn wave_four_promotion_requires_review_evidence_for_every_common_row() {
    let mut coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let receipt = compile_exercise_receipt(Wave4Phase::Wave4Complete)
        .expect("fresh Wave 4 exercise compiler receipt");

    let row = coverage["items"]
        .as_array_mut()
        .expect("coverage rows")
        .iter_mut()
        .find(|row| row["item_id"] == "CFT-29-006")
        .expect("CFT-29-006 coverage row");
    row["review_status"]["status"] = serde_json::json!("registered-for-content-wave-review");

    let errors = phase_state_errors(
        &coverage,
        &exercises,
        Some(&receipt),
        Wave4Phase::Wave4Complete,
    );
    assert!(
        errors.iter().any(|error| {
            error.contains("CFT-29-006")
                && error.contains("independent content and formal review evidence")
        }),
        "Wave 4 promotion accepted a row without completed review evidence: {errors:#?}"
    );
}

#[test]
fn chapter_27_barrier_workshop_exposes_the_exact_relaxation() {
    let markdown = chapter_markdown(27);
    for exact in [
        "(κ-1-√2)(κ-1+√2)",
        "κ²≤2κ+1",
        "-2\\operatorname{Re}\\langle u,v\\rangle",
        "A_{0,2}",
        "No trained-network claim",
        "CROUZEIX-PALENCIA-2017",
        "metadata-only",
        "the scalar upper bound does not require `κ≥0`",
    ] {
        assert!(markdown.contains(exact), "Chapter 27 omits `{exact}`");
    }
    assert!(
        markdown.contains("\\|S^*T-H\\|≤\\|S\\|\\,\\|T\\|+\\|H\\|≤2κ+1"),
        "Chapter 27 must display the operator inequality producing the quadratic"
    );
    assert!(
        markdown.contains("measure the normalized cross-term residual"),
        "Chapter 27 must give ML researchers a concrete cross-term diagnostic"
    );
    for truthful_bridge in [
        "finite-dimensional norm-attaining unit vector",
        "\\|U+V\\|^2=\\|Ux\\|^2+\\|Vx\\|^2+2\\operatorname{Re}\\langle Ux,Vx\\rangle",
        "vector-level analogue, not an operator-norm identity",
        "the Lean helper formalizes only this vector identity",
    ] {
        assert!(
            markdown.contains(truthful_bridge),
            "Chapter 27 omits the vector-to-operator bridge `{truthful_bridge}`"
        );
    }
    assert!(
        markdown.contains("complex witnesses `a=1`") && markdown.contains("`b=√2`"),
        "Chapter 27 must use the compiled complex equality witness"
    );
    assert!(
        !markdown.contains("choose `u=2κe` and `v=-e`"),
        "Chapter 27 must not claim a general scalar-multiplication witness under only a normed additive-group hypothesis"
    );

    let coverage = read_json(&contracts_root().join("coverage.json"));
    let row = coverage["items"]
        .as_array()
        .expect("coverage rows")
        .iter()
        .find(|row| row["item_id"] == "CFT-27-001")
        .expect("CFT-27-001 coverage row");
    assert_eq!(
        row["kind"], "theorem",
        "CFT-27-001 is a proved proposition, not a definition"
    );
}

#[test]
fn chapter_28_power_family_keeps_every_power_and_the_sharp_extraction_visible() {
    let markdown = chapter_markdown(28);
    for exact in [
        "for every `m : ℕ` on one fixed contour and one fixed measure",
        "the `m=0` identity",
        "(1+zw)/(1-zw)=1+2∑_{m≥0}(zw)^{m+1}",
        "(1-|zw|²)/|1-zw|²",
        "2Φ_D(c_z)=C_z(T)+g(z)^*",
        "preserves the complete power family",
        "The power-family construction through CFT-28-005 does not invoke a Jin",
        "A_{0,2}^0=I",
        "A_{0,2}^m=0",
        "No trained-network claim",
        "multiplicative-coherence residual",
    ] {
        assert!(markdown.contains(exact), "Chapter 28 omits `{exact}`");
    }
    assert!(
        markdown.contains("‖T‖≤2"),
        "Chapter 28 must display the sharp scalar norm extraction"
    );
    assert!(
        markdown.contains("one function at `m=1` is insufficient"),
        "Chapter 28 must state why a single Cauchy identity cannot justify the Cayley sum"
    );
    assert!(
        markdown.contains(r"r_m=‖\hat T_m-\hat T_1^m‖/(1+‖\hat T_1‖^m)"),
        "Chapter 28 must give ML researchers a normalized coherence diagnostic"
    );

    let corpus: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(workspace_root().join("atlas/src/content/generated/corpus.json"))
            .expect("generated Atlas corpus"),
    )
    .expect("valid generated Atlas corpus");
    let rendered = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| {
            document["canonical_markdown_path"].as_str()
                == Some(
                    "knowledge/crouzeix_textbook/part_05_crouzeix_machinery/28_complete_power_family.md",
                )
        })
        .and_then(|document| document["html"].as_str())
        .expect("rendered Chapter 28 HTML");
    for exact in [
        "for every <code>m : ℕ</code> on one fixed contour and one fixed measure",
        "the <code>m=0</code> identity",
        "multiplicative-coherence residual",
        "No trained-network claim",
    ] {
        assert!(
            rendered.contains(exact),
            "rendered Chapter 28 omits `{exact}`"
        );
    }
}

#[test]
fn chapter_29_normalization_and_sharpness_separate_exact_attainment_from_limits() {
    let markdown = chapter_markdown(29);
    for exact in [
        "A_{0,2}=\\begin{pmatrix}0&2\\\\0&0\\end{pmatrix}",
        "W(A+βI)=W(A)+β",
        "W(αA)=αW(A)",
        "p_{α,β}(z)=p(αz+β)",
        "W(A_{0,2})={z∈ℂ:|z|≤1}",
        "⟨x,A_{0,2}x⟩=2\\bar x_1x_2",
        "‖A_{0,2}‖=2",
        "max_{z∈W(A_{0,2})}|z|=1",
        "2/1=2",
        "exactly attained",
        "not a limiting sharpness claim",
        "M_ε→M",
        "J_1=(\\begin{smallmatrix}0&1\\\\0&0\\end{smallmatrix})",
        "W(J_1)={z∈ℂ:|z|≤1/2}",
        "CROUZEIX-PACKET",
        "JIN-V4-AUDITED",
        "LS-ARXIV-V1",
        "Harp-derived finite-horizon route",
        "No trained-network claim",
        "affine-consistency residual",
    ] {
        assert!(markdown.contains(exact), "Chapter 29 omits `{exact}`");
    }
    assert!(
        markdown.contains("M=0") && markdown.contains("p(A)=0"),
        "Chapter 29 must prove the zero-maximum branch instead of dividing by zero"
    );
    assert!(
        markdown.contains("finite-matrix polynomial")
            && markdown.contains("finite-matrix rational")
            && markdown.contains("Hilbert-space rational"),
        "Chapter 29 must separate the three theorem surfaces"
    );
    assert!(
        markdown.contains("sharpness does not prove the universal upper bound"),
        "Chapter 29 must keep the lower and upper bounds logically separate"
    );

    let corpus: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(workspace_root().join("atlas/src/content/generated/corpus.json"))
            .expect("generated Atlas corpus"),
    )
    .expect("valid generated Atlas corpus");
    let rendered = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| {
            document["canonical_markdown_path"].as_str()
                == Some(
                    "knowledge/crouzeix_textbook/part_06_constant_two_routes/29_crouzeix_problem_and_sharpness.md",
                )
        })
        .and_then(|document| document["html"].as_str())
        .expect("rendered Chapter 29 HTML");
    for exact in [
        "A_{0,2}",
        "exactly attained",
        "not a limiting sharpness claim",
        "affine-consistency residual",
        "No trained-network claim",
    ] {
        assert!(
            rendered.contains(exact),
            "rendered Chapter 29 omits `{exact}`"
        );
    }
}

#[test]
fn chapter_29_positive_normalization_derives_the_maximum_in_lean() {
    const NAME: &str = "CrouzeixTextbook.Part06.normalized_max_polynomial_modulus";
    const SOURCE: &str = "formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean";
    const TYPE_SHA256: &str = "907f1242b2662c78c6878597633b91cb7ac60ffc0b11b47b416f011ffabe18ab";
    let markdown = chapter_markdown(29);
    assert!(
        markdown.contains(NAME),
        "Chapter 29 omits the checked maximum-scaling theorem"
    );
    for exact in [
        "[Lean proof](../../../formalization/lean/CrouzeixTextbook/Part06/Chapter29.lean#L120)",
        "M(A,M^{-1}p)=1",
        "derived from `M(A,p)=M` and `M>0`",
        "E05 keeps the frozen normalized-maximum hypothesis",
        "Formal mode:\n`proved-here`",
        "Compiler locator: `Chapter29.lean:120:9`",
    ] {
        assert!(
            markdown.contains(exact),
            "Chapter 29 normalization support omits `{exact}`"
        );
    }

    let receipt = compile_supporting_receipt(NAME);
    let rows = receipt
        .as_array()
        .expect("normalization support receipt rows");
    assert_eq!(rows.len(), 1, "normalization receipt must be exact");
    let row = &rows[0];
    assert_eq!(row["name"], NAME);
    assert_eq!(row["kind"], "theorem");
    assert_eq!(row["source_path"], SOURCE);
    assert_eq!(row["line"], 120);
    assert_eq!(row["column"], 9);
    assert_eq!(row["type_sha256"], TYPE_SHA256);
    assert_eq!(row["axioms"], serde_json::json!(AXIOMS));
    let dependencies = [
        "CrouzeixConjecture.SquareMatrix",
        "CrouzeixConjecture.exists_maxPolynomialModulusOnNumericalRange",
        "CrouzeixConjecture.maxPolynomialModulusOnNumericalRange",
        "CrouzeixConjecture.norm_polynomial_eval_le_maxOnNumericalRange",
        "CrouzeixConjecture.numericalRange",
    ];
    assert_eq!(row["direct_dependencies"], serde_json::json!(dependencies));
    let normalized_type = row["normalized_type"]
        .as_str()
        .expect("normalization support type");
    for exact in [
        "maxPolynomialModulusOnNumericalRange",
        "LT.lt",
        "Inv.inv",
        "OfNat.ofNat.{0} 1",
    ] {
        assert!(
            normalized_type.contains(exact),
            "normalization support type omits `{exact}`: {normalized_type}"
        );
    }
    for exact in [
        format!("Normalized type: `{normalized_type}`."),
        format!("Type SHA-256: `{TYPE_SHA256}`."),
        format!(
            "Direct maintained dependencies: `{}`.",
            dependencies.join(", ")
        ),
        "Axioms: `Classical.choice, Quot.sound, propext`.".to_owned(),
        "Verification target:\n`CrouzeixTextbook`.".to_owned(),
        format!("Receipt identity:\n`CrouzeixTextbook:{TYPE_SHA256}`."),
    ] {
        assert!(
            markdown.contains(&exact),
            "Chapter 29 supporting receipt omits or drifts from `{exact}`"
        );
    }
}

#[test]
fn chapter_29_main_theorem_card_separates_all_dimension_and_fixed_index_scopes() {
    let markdown = chapter_markdown(29);
    let spec = COMMON_SPECS
        .iter()
        .copied()
        .find(|candidate| candidate.item_id == "CFT-29-003")
        .expect("CFT-29-003 spec");
    let card = card_region(&markdown, spec).expect("CFT-29-003 card");
    for exact in [
        "FiniteMatrixMainTheoremStatement",
        "∀d∈ℕ",
        "A:SquareMatrix(Fin d)",
        "the public card is all-dimensional",
        "E06 is the fixed-index equivalence",
        "MainTheoremStatement (n:=n)",
    ] {
        assert!(card.contains(exact), "CFT-29-003 omits `{exact}`");
    }
}

#[test]
fn chapter_29_least_constant_card_matches_the_scaled_lean_witness() {
    let markdown = chapter_markdown(29);
    let spec = COMMON_SPECS
        .iter()
        .copied()
        .find(|candidate| candidate.item_id == "CFT-29-006")
        .expect("CFT-29-006 spec");
    let card = card_region(&markdown, spec).expect("CFT-29-006 card");
    for exact in [
        "A=A_{0,2}=2J_1",
        "2=‖X(A_{0,2})‖≤K·1",
        "max_{z∈W(A_{0,2})}|z|=1",
    ] {
        assert!(card.contains(exact), "CFT-29-006 omits `{exact}`");
    }
    for mismatched in ["use `A=J_1`", "1≤K·\\frac12", "1≤K/2"] {
        assert!(
            !card.contains(mismatched),
            "CFT-29-006 retains the unscaled calculation `{mismatched}`"
        );
    }
}

#[test]
fn chapter_29_reverse_disk_inclusion_follows_boundary_and_convexity() {
    let markdown = chapter_markdown(29);
    let card_spec = COMMON_SPECS
        .iter()
        .copied()
        .find(|candidate| candidate.item_id == "CFT-29-004")
        .expect("CFT-29-004 spec");
    let card = card_region(&markdown, card_spec).expect("CFT-29-004 card");
    let exercise = markdown
        .split_once("### CFT-29-E02")
        .expect("E02 start")
        .1
        .split_once("### CFT-29-E03")
        .expect("E02 end")
        .0;
    for (label, region) in [("CFT-29-004", card), ("CFT-29-E02", exercise)] {
        for exact in [
            "boundary circle",
            "numericalRange_convex",
            "convexHull_sphere_eq_closedBall",
        ] {
            assert!(region.contains(exact), "{label} omits `{exact}`");
        }
    }
}

#[test]
fn chapter_28_norm_two_card_is_an_explicit_deferred_jin_preview() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let row = coverage["items"]
        .as_array()
        .expect("coverage rows")
        .iter()
        .find(|row| row["item_id"] == "CFT-28-006")
        .expect("CFT-28-006 coverage row");
    assert_eq!(row["prose_proof_status"], "summary");
    assert!(
        row["source_ids"]
            .as_array()
            .expect("source ids")
            .iter()
            .any(|source| source == "JIN-V4-AUDITED"),
        "CFT-28-006 must name the Jin source boundary"
    );
    for prerequisite in ["CFT-30-002", "CFT-32-001"] {
        assert!(
            row["pedagogical_prerequisites"]
                .as_array()
                .expect("pedagogical prerequisites")
                .iter()
                .any(|candidate| candidate == prerequisite),
            "CFT-28-006 must defer its reader proof to {prerequisite}"
        );
    }

    let markdown = chapter_markdown(28);
    let card = markdown
        .split("### CFT-28-006")
        .nth(1)
        .expect("CFT-28-006 card");
    for exact in [
        "DEFERRED JIN COMPLETION-TO-TWO CONSEQUENCE",
        "CrouzeixConjecture.positiveRealCompletionStatement",
        "CFT-30-002",
        "CFT-32-001",
    ] {
        assert!(card.contains(exact), "CFT-28-006 omits `{exact}`");
    }
    for false_claim in [
        "Applying the common completion statement to a norm-attaining vector",
        "route-specific historical attribution begins with later provider chapters",
        "No terminal route is an assumption",
    ] {
        assert!(
            !card.contains(false_claim),
            "CFT-28-006 retains false route-neutral claim `{false_claim}`"
        );
    }
}

#[test]
fn chapter_28_cayley_series_uses_spectral_radius_for_nonnormal_matrices() {
    let markdown = chapter_markdown(28);
    let card = card_region(&markdown, common_spec("CFT-28-004")).expect("CFT-28-004 card");
    for exact in [
        "σ(zT)=zσ(T)",
        "r(zT)≤|z|<1",
        "(I-zT)^{-1}=\\sum_{m≥0}(zT)^m",
        "converges in operator norm",
        "bounded linear integration map",
        "cannot use `‖zT‖<1`",
    ] {
        assert!(card.contains(exact), "CFT-28-004 omits `{exact}`");
    }
}

#[test]
fn chapter_25_variational_card_states_the_formal_compactness_scope() {
    let markdown = chapter_markdown(25);
    let card = normalized_text(
        card_region(&markdown, common_spec("CFT-25-002")).expect("CFT-25-002 card"),
    );
    for exact in [
        "exact Lean theorem below assumes that `K` is nonempty, compact, and real-convex",
        "segment argument itself needs only convexity and an attained minimizer",
        "CFT-25-001 supplies the broader closed-convex existence theorem",
    ] {
        assert!(card.contains(exact), "CFT-25-002 omits `{exact}`");
    }
}

#[test]
fn chapter_29_theorem_surfaces_link_their_supporting_lean_declarations() {
    let markdown = chapter_markdown(29);
    let card = card_region(&markdown, common_spec("CFT-29-003")).expect("CFT-29-003 card");
    for exact in [
        "CrouzeixConjecture.crouzeixRationalSpectralSetCorollary",
        "../../../formalization/lean/CrouzeixConjecture/FinalTheorems.lean#L26",
        "CrouzeixConjecture.hilbertSpaceRationalSpectralSet",
        "../../../formalization/lean/CrouzeixConjecture/HilbertSpectralSet.lean#L115",
        "Supporting declarations, not the primary card theorem",
    ] {
        assert!(card.contains(exact), "CFT-29-003 omits `{exact}`");
    }
}

#[test]
fn chapter_29_history_records_dated_three_route_provenance() {
    let markdown = chapter_markdown(29);
    let card = card_region(&markdown, common_spec("CFT-29-006")).expect("CFT-29-006 card");
    for exact in [
        "2026-08-05T16:32:05+08:00",
        "565b6a3e0659b6e0785f783b016c3f6d9f171fa5",
        "reproduction_research#public-artifact-sequence",
        "arxiv:2608.03841v1",
        "observed on 2026-08-14",
        "2026-08-29 Harp review boundary",
        "../../../formalization/lean/Crouzeix/Harp/MainTheorem.lean#L25",
        "claim_evidence_ledger#cft-cl-003",
    ] {
        assert!(card.contains(exact), "CFT-29-006 omits `{exact}`");
    }
}

#[test]
fn chapter_28_a_zero_two_power_truncation_has_a_compiler_receipt() {
    const NAME: &str = "CrouzeixTextbook.Part05.a_zero_two_power_truncation";
    let markdown = chapter_markdown(28);
    assert!(
        markdown.contains(NAME),
        "Chapter 28 omits the supporting Lean theorem"
    );
    assert!(
        markdown.contains("Chapter28.lean#L"),
        "Chapter 28 omits the supporting theorem code link"
    );

    let receipt = compile_supporting_receipt(NAME);
    let rows = receipt.as_array().expect("supporting theorem receipt rows");
    assert_eq!(rows.len(), 1, "supporting receipt must be exact");
    assert_eq!(rows[0]["name"], NAME);
    assert_eq!(rows[0]["kind"], "theorem");
    assert_eq!(rows[0]["axioms"], serde_json::json!(AXIOMS));
    let normalized_type = rows[0]["normalized_type"]
        .as_str()
        .expect("supporting normalized type");
    for exact in [
        "CrouzeixTextbook.Part05.aZeroTwo (OfNat.ofNat.{0} 0)",
        "CrouzeixTextbook.Part05.aZeroTwo (OfNat.ofNat.{0} 1)",
        "∀ (m : ℕ), LE.le.{0} (OfNat.ofNat.{0} 2) m",
    ] {
        assert!(
            normalized_type.contains(exact),
            "supporting theorem type omits `{exact}`: {normalized_type}"
        );
    }
}

#[test]
fn common_build_guard_propagates_failure_and_does_not_retry_probes() {
    let result = OnceLock::new();
    let mut rejected = Command::new("sh");
    rejected.args([
        "-c",
        "printf '%s\\n' 'cache precondition failed' >&2; exit 7",
    ]);
    let error = common_build_once(&result, &mut rejected).expect_err("reject failed wrapper");
    assert!(error.contains("cache precondition failed"), "{error}");
    let mut would_succeed = Command::new("sh");
    would_succeed.args(["-c", "exit 0"]);
    assert_eq!(common_build_once(&result, &mut would_succeed), Err(error));
}

#[test]
fn common_build_guard_caches_success_and_reports_missing_wrapper() {
    let result = OnceLock::new();
    let mut success = Command::new("sh");
    success.args(["-c", "exit 0"]);
    assert_eq!(common_build_once(&result, &mut success), Ok(()));
    let mut would_fail = Command::new("sh");
    would_fail.args(["-c", "exit 9"]);
    assert_eq!(common_build_once(&result, &mut would_fail), Ok(()));

    let mut missing_wrapper = Command::new("");
    let error = common_build_once(&OnceLock::new(), &mut missing_wrapper)
        .expect_err("missing wrapper cannot certify freshness");
    assert!(error.contains("run canonical textbook build"), "{error}");
}

#[test]
fn common_machinery_contract_rejects_fake_completed_exercise_receipts() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let mut exercises = read_json(&contracts_root().join("exercises.json"));
    let empty_compiler_receipt = compile_exercise_receipt(Wave4Phase::ContractFrozen)
        .expect("the canonical exercise receipt probe itself must compile");
    assert_eq!(
        empty_compiler_receipt,
        serde_json::json!({"target": "CrouzeixTextbook", "declarations": []})
    );
    let mut declarations = Vec::new();
    for exercise in EXERCISE_SPECS
        .into_iter()
        .filter(|spec| common_spec(spec.parent).chapter == 25)
    {
        let parent = common_spec(exercise.parent);
        let declaration = exercise_declaration_for(exercise);
        let normalized_type = format!("FabricatedExerciseProposition {}", exercise.exercise_id);
        let type_sha256 = receipt_type_hash(&normalized_type);
        let row = serde_json::json!({
            "name": declaration,
            "kind": "theorem",
            "source_path": parent.source_path,
            "line": 1,
            "column": 1,
            "normalized_type": normalized_type,
            "type_sha256": type_sha256,
            "direct_dependencies": [],
            "body_dependencies": [],
            "axioms": AXIOMS,
        });
        let exercise_row = exercises["exercises"]
            .as_array_mut()
            .expect("exercise rows")
            .iter_mut()
            .find(|row| row["exercise_id"].as_str() == Some(exercise.exercise_id))
            .expect("Chapter 25 exercise row");
        exercise_row["lean_solution"] = serde_json::json!({
            "declaration": row["name"],
            "source_path": row["source_path"],
            "line": row["line"],
            "column": row["column"],
            "type_sha256": row["type_sha256"],
            "verification_target": "CrouzeixTextbook",
            "axioms": AXIOMS,
        });
        declarations.push(row);
    }
    let fabricated = serde_json::json!({
        "target": "CrouzeixTextbook",
        "declarations": declarations,
    });
    let fabricated_errors = exercise_receipt_errors(
        &coverage,
        &exercises,
        Some(&fabricated),
        Wave4Phase::Chapter25Complete,
    );
    assert!(
        fabricated_errors
            .iter()
            .filter(|error| error.contains("compiler-valid code locator"))
            .count()
            == 6,
        "fabricated receipt coordinates were accepted: {fabricated_errors:#?}"
    );
    assert_eq!(
        fabricated_errors
            .iter()
            .filter(|error| error.contains("planned mathematical proposition"))
            .count(),
        6,
        "unrelated theorem fingerprints were accepted: {fabricated_errors:#?}"
    );

    let mut unapproved_axiom = fabricated.clone();
    unapproved_axiom["declarations"][0]["axioms"] = serde_json::json!(["sorryAx"]);
    exercises["exercises"]
        .as_array_mut()
        .expect("exercise rows")
        .iter_mut()
        .find(|row| row["exercise_id"].as_str() == Some("CFT-25-E01"))
        .expect("CFT-25-E01 metadata")["lean_solution"]["axioms"] = serde_json::json!(["sorryAx"]);
    assert!(exercise_receipt_errors(
        &coverage,
        &exercises,
        Some(&unapproved_axiom),
        Wave4Phase::Chapter25Complete,
    )
    .iter()
    .any(|error| error.contains("approved common-proof set")));
    exercises["exercises"]
        .as_array_mut()
        .expect("exercise rows")
        .iter_mut()
        .find(|row| row["exercise_id"].as_str() == Some("CFT-25-E01"))
        .expect("CFT-25-E01 metadata")["lean_solution"]["axioms"] = serde_json::json!(AXIOMS);

    let mut duplicate = fabricated.clone();
    let first_type = duplicate["declarations"][0]["normalized_type"].clone();
    let first_hash = duplicate["declarations"][0]["type_sha256"].clone();
    duplicate["declarations"][1]["normalized_type"] = first_type;
    duplicate["declarations"][1]["type_sha256"] = first_hash;
    assert!(
        exercise_receipt_errors(
            &coverage,
            &exercises,
            Some(&duplicate),
            Wave4Phase::Chapter25Complete,
        )
        .iter()
        .any(|error| error.contains("reuse the same exercise statement fingerprint")),
        "cross-card duplicate statement survived the receipt contract"
    );

    let mut shortcut = fabricated.clone();
    shortcut["declarations"][2]["direct_dependencies"] =
        serde_json::json!([COMMON_SPECS[2].public_declaration]);
    shortcut["declarations"][2]["body_dependencies"] =
        serde_json::json!([COMMON_SPECS[2].public_declaration]);
    assert!(
        exercise_receipt_errors(
            &coverage,
            &exercises,
            Some(&shortcut),
            Wave4Phase::Chapter25Complete,
        )
        .iter()
        .any(|error| error.contains("forbidden parent theorem shortcut")),
        "direct parent shortcut survived the receipt contract"
    );

    let public_provider_rows = public_provider_receipt()["declarations"]
        .as_array()
        .expect("public provider receipt rows");
    let statement_constants = common_statement_constants(public_provider_rows);
    let later_provider =
        receipt_body_dependencies(public_provider_rows, COMMON_SPECS[4].public_declaration)
            .into_iter()
            .find(|provider| !statement_constants.contains(provider))
            .expect("later substantive provider name");
    let mut later_shortcut = fabricated.clone();
    later_shortcut["declarations"][0]["direct_dependencies"] = serde_json::json!([later_provider]);
    later_shortcut["declarations"][0]["body_dependencies"] = serde_json::json!([later_provider]);
    assert!(exercise_receipt_errors(
        &coverage,
        &exercises,
        Some(&later_shortcut),
        Wave4Phase::Chapter25Complete,
    )
    .iter()
    .any(|error| error.contains("same-or-later common proof provider")));

    let mut earlier_reconstruction_shortcut = fabricated.clone();
    earlier_reconstruction_shortcut["declarations"][3]["direct_dependencies"] =
        serde_json::json!(["CrouzeixConjecture.norm_convexProjection_sub_le"]);
    earlier_reconstruction_shortcut["declarations"][3]["body_dependencies"] =
        serde_json::json!(["CrouzeixConjecture.norm_convexProjection_sub_le"]);
    assert!(
        exercise_receipt_errors(
            &coverage,
            &exercises,
            Some(&earlier_reconstruction_shortcut),
            Wave4Phase::Chapter25Complete,
        )
        .iter()
        .any(|error| error.contains("reconstruction target provider")),
        "an earlier theorem reconstructed by CFT-25-E04 survived the shortcut contract"
    );

    let mut alias = fabricated.clone();
    alias["declarations"][0]["kind"] = serde_json::json!("direct-alias");
    assert!(
        exercise_receipt_errors(
            &coverage,
            &exercises,
            Some(&alias),
            Wave4Phase::Chapter25Complete,
        )
        .iter()
        .any(|error| error.contains("must compile as a theorem")),
        "direct alias survived the receipt contract"
    );

    let mut transitive = fabricated;
    transitive["declarations"][3]["direct_dependencies"] =
        serde_json::json!(["CrouzeixTextbook.Part05.Exercises.HiddenShortcut"]);
    transitive["declarations"][3]["body_dependencies"] =
        serde_json::json!(["CrouzeixTextbook.Part05.Exercises.HiddenShortcut"]);
    transitive["declarations"]
        .as_array_mut()
        .expect("receipt rows")
        .push(serde_json::json!({
            "name": "CrouzeixTextbook.Part05.Exercises.HiddenShortcut",
            "direct_dependencies": [EXERCISE_SPECS[3].reconstruction_target_providers[0]],
            "body_dependencies": [EXERCISE_SPECS[3].reconstruction_target_providers[0]],
        }));
    assert!(
        exercise_receipt_errors(
            &coverage,
            &exercises,
            Some(&transitive),
            Wave4Phase::Chapter25Complete,
        )
        .iter()
        .any(|error| error.contains("reconstruction target provider")),
        "transitive provider shortcut survived the receipt contract"
    );
}

#[test]
fn chapter_26_congruence_exercise_rejects_the_mathlib_shortcut_transitively() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let source = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean"),
    )
    .expect("Chapter 26 Lean source");
    assert!(
        !source.contains(".conjTranspose_mul_mul_same"),
        "CFT-26-E03 still calls the former one-line congruence shortcut"
    );
    let receipt = compile_exercise_receipt(Wave4Phase::Chapter26Complete)
        .expect("fresh Chapter 26 exercise receipt");
    let rows = receipt["declarations"]
        .as_array()
        .expect("exercise receipt declarations");
    let shortcut = "Matrix.PosSemidef.conjTranspose_mul_mul_same";
    for exercise_id in ["CFT-26-E03", "CFT-26-E06"] {
        let declaration = exercise_declaration_for(
            EXERCISE_SPECS
                .into_iter()
                .find(|spec| spec.exercise_id == exercise_id)
                .expect("named Chapter 26 exercise"),
        );
        assert!(
            !receipt_body_dependencies(rows, &declaration).contains(shortcut),
            "{exercise_id} proof closure still contains the forbidden congruence shortcut"
        );
    }

    let mut former_proof = receipt;
    let declaration = exercise_declaration_for(
        EXERCISE_SPECS
            .into_iter()
            .find(|spec| spec.exercise_id == "CFT-26-E03")
            .expect("CFT-26-E03 spec"),
    );
    let row = former_proof["declarations"]
        .as_array_mut()
        .expect("exercise receipt rows")
        .iter_mut()
        .find(|row| row["name"].as_str() == Some(declaration.as_str()))
        .expect("CFT-26-E03 receipt row");
    row["direct_dependencies"] = serde_json::json!([shortcut]);
    row["body_dependencies"] = serde_json::json!([shortcut]);
    let errors = exercise_receipt_errors(
        &coverage,
        &exercises,
        Some(&former_proof),
        Wave4Phase::Chapter26Complete,
    );
    assert!(
        errors.iter().any(|error| error.contains(shortcut)),
        "the former one-line Mathlib congruence proof survived the negative mutation: {errors:#?}"
    );
}

#[test]
fn chapter_26_nilpotent_example_tracks_cauchy_and_measure_normalization() {
    let markdown = chapter_markdown(26);
    let corpus: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(workspace_root().join("atlas/src/content/generated/corpus.json"))
            .expect("generated Atlas corpus"),
    )
    .expect("valid generated Atlas corpus");
    let rendered = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| {
            document["canonical_markdown_path"].as_str()
                == Some(
                    "knowledge/crouzeix_textbook/part_05_crouzeix_machinery/26_double_layer_map.md",
                )
        })
        .and_then(|document| document["html"].as_str())
        .expect("rendered Chapter 26 HTML");
    for exact in [
        "parameter domain is $[0,2π]$",
        "unnormalized measure $dμ(t)=dt$",
        "normalized angular\nmeasure $d\\widehat{μ}(t)=dt/(2π)$",
        "\\int_0^{2π}F(t)\\,dt=I",
        "\\int_0^{2π}K(t)\\,dt=2I",
        "\\int_0^{2π}A(t)\\,d\\widehat{μ}(t)=I",
        "The analytic first part has identity mass; the adjoint-symmetrized density has mass $2I$.",
    ] {
        assert!(markdown.contains(exact), "Chapter 26 omits `{exact}`");
    }
    for exact in [
        "parameter domain is <span class=\"math math-inline\" data-tex=\"[0,2π]\">[0,2π]</span>",
        "unnormalized measure <span class=\"math math-inline\" data-tex=\"dμ(t)=dt\">dμ(t)=dt</span>",
        "normalized angular\nmeasure <span class=\"math math-inline\" data-tex=\"d\\widehat{μ}(t)=dt/(2π)\">",
        "\\int_0^{2π}F(t)\\,dt=I,\\qquad\n\\int_0^{2π}K(t)\\,dt=2I.",
        "data-tex=\"\\int_0^{2π}A(t)\\,d\\widehat{μ}(t)=I\"",
        "The analytic first part has identity mass; the adjoint-symmetrized density has mass",
    ] {
        assert!(
            rendered.contains(exact),
            "rendered Chapter 26 HTML omits `{exact}`"
        );
    }
}

#[test]
fn common_machinery_planned_types_retain_each_exercises_mathematical_objects() {
    for spec in EXERCISE_SPECS {
        let lean_type = exercise_lean_type(spec.exercise_id);
        let errors = exercise_semantic_errors(spec, lean_type);
        assert!(
            errors.is_empty(),
            "{} failed its row-specific semantic audit: {errors:#?}",
            spec.exercise_id
        );
        for fragment in spec.semantic_fragments {
            let mutated = lean_type.replace(fragment, "");
            assert!(
                exercise_semantic_errors(spec, &mutated)
                    .iter()
                    .any(|error| error.contains(fragment)),
                "{} accepted removal of semantic fragment `{fragment}`",
                spec.exercise_id
            );
        }
    }
    let scalar_barrier = exercise_lean_type("CFT-27-E06");
    assert!(scalar_barrier.contains("κ ^ 2 ≤ 2 * κ + 1"));
    assert!(scalar_barrier.contains("κ ≤ 1 + Real.sqrt 2"));
    assert!(
        !scalar_barrier.starts_with("∀ κ : ℝ, 0 ≤ κ →")
            && scalar_barrier.contains("κ = ‖T‖ → 0 ≤ κ"),
        "CFT-27-E06 must formalize the upper bound without a nonnegativity hypothesis"
    );

    let theorem_spec = EXERCISE_SPECS[0];
    for bad_type in [
        "True",
        "P → True",
        "Nonempty n → Nonempty n",
        "∃ F : ℕ → ℂ, ∀ k : ℕ, F k = A ^ k",
        "P → P",
        "namedObject = namedObject ↔ namedObject = namedObject",
    ] {
        assert!(
            !exercise_semantic_errors(theorem_spec, bad_type).is_empty(),
            "generic semantic audit accepted `{bad_type}`"
        );
    }
}

#[test]
fn common_machinery_reconstruction_targets_are_independent_of_indexed_parent() {
    for (exercise_id, provider) in [
        (
            "CFT-25-E04",
            "CrouzeixConjecture.norm_convexProjection_sub_le",
        ),
        (
            "CFT-26-E05",
            "CrouzeixConjecture.scalar_sub_matrix_isUnit_of_outwardBoundarySupport",
        ),
        (
            "CFT-28-E04",
            "CrouzeixConjecture.HasParametricPowerCauchyFormula.mass_eq_one",
        ),
    ] {
        let spec = EXERCISE_SPECS
            .into_iter()
            .find(|spec| spec.exercise_id == exercise_id)
            .expect("named reconstruction exercise");
        assert_ne!(common_spec(spec.parent).public_declaration, provider);
        let dependencies = BTreeSet::from([provider.to_owned()]);
        assert!(
            reconstruction_target_errors(spec, &dependencies)
                .iter()
                .any(|error| error.contains(provider)),
            "{exercise_id} accepted earlier reconstruction target `{provider}`"
        );
    }

    for spec in EXERCISE_SPECS {
        for provider in spec.reconstruction_target_providers {
            let dependencies = BTreeSet::from([(*provider).to_owned()]);
            assert_eq!(
                reconstruction_target_errors(spec, &dependencies).len(),
                1,
                "{} did not reject reconstruction target `{provider}`",
                spec.exercise_id
            );
        }
    }
}

#[test]
fn common_machinery_provider_gate_uses_transitive_proof_bodies_not_statement_constants() {
    let public_rows = public_provider_receipt()["declarations"]
        .as_array()
        .expect("public provider receipt rows");
    let spec = EXERCISE_SPECS[0];

    let statement_constants = common_statement_constants(public_rows);
    let statement_body_constant = COMMON_SPECS
        .iter()
        .flat_map(|candidate| {
            receipt_body_dependencies(public_rows, candidate.public_declaration).into_iter()
        })
        .find(|provider| statement_constants.contains(provider))
        .expect("statement-level constant that also occurs in a proof body");
    let statement_body = BTreeSet::from([statement_body_constant.clone()]);
    assert!(
        same_or_later_provider_errors(spec, &statement_body, public_rows).is_empty(),
        "proof-body constant `{statement_body_constant}` was not accepted by the compiler-derived statement allowlist"
    );

    let (later_candidate, later_provider) = COMMON_SPECS
        .iter()
        .find_map(|candidate| {
            let direct = public_rows
                .iter()
                .find(|row| row["name"].as_str() == Some(candidate.public_declaration))
                .expect("candidate public provider row")["body_dependencies"]
                .as_array()
                .expect("candidate direct body dependencies")
                .iter()
                .filter_map(Value::as_str)
                .collect::<BTreeSet<_>>();
            receipt_body_dependencies(public_rows, candidate.public_declaration)
                .into_iter()
                .find(|provider| {
                    !direct.contains(provider.as_str())
                        && !statement_constants.contains(provider)
                        && !same_or_later_provider_errors(
                            spec,
                            &BTreeSet::from([provider.clone()]),
                            public_rows,
                        )
                        .is_empty()
                })
                .map(|provider| (*candidate, provider))
        })
        .expect("genuinely transitive same-or-later provider");
    let transitive = vec![
        serde_json::json!({
            "name": "CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_01_solution",
            "direct_dependencies": ["CrouzeixTextbook.Part05.Exercises.HiddenProvider"],
            "body_dependencies": ["CrouzeixTextbook.Part05.Exercises.HiddenProvider"],
        }),
        serde_json::json!({
            "name": "CrouzeixTextbook.Part05.Exercises.HiddenProvider",
            "direct_dependencies": [later_provider],
            "body_dependencies": [later_provider],
        }),
    ];
    let transitive_body = receipt_body_dependencies(
        &transitive,
        "CrouzeixTextbook.Part05.Exercises.Chapter25.exercise_01_solution",
    );
    assert!(
        same_or_later_provider_errors(spec, &transitive_body, public_rows)
            .iter()
            .any(|error| {
                error.contains(&later_provider) && error.contains(later_candidate.item_id)
            }),
        "provider `{later_provider}` reached only through the public theorem and exercise helper closures survived"
    );

    let later_exercise = EXERCISE_SPECS[11];
    let parent_position = COMMON_SPECS
        .iter()
        .position(|candidate| candidate.item_id == later_exercise.parent)
        .expect("later exercise parent");
    let earlier_provider = COMMON_SPECS[..parent_position]
        .iter()
        .flat_map(|earlier| {
            receipt_body_dependencies(public_rows, earlier.public_declaration).into_iter()
        })
        .find(|provider| {
            !later_exercise
                .reconstruction_target_providers
                .contains(&provider.as_str())
        })
        .expect("legitimate earlier common provider");
    assert!(
        same_or_later_provider_errors(
            later_exercise,
            &BTreeSet::from([earlier_provider.clone()]),
            public_rows,
        )
        .is_empty(),
        "legitimate earlier provider `{earlier_provider}` was rejected"
    );
}

#[test]
fn common_machinery_contract_binds_all_public_names_to_fresh_compiler_locators() {
    let contract = wave4_contract();
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let coverage_rows = rows_for_chapters(&coverage, "items");
    assert_eq!(coverage_rows.len(), 30);
    assert_eq!(
        coverage_rows
            .iter()
            .filter_map(|row| row["anchor"].as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        30
    );
    assert_eq!(
        coverage_rows
            .iter()
            .filter_map(|row| row["lean_declaration"]["name"].as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        30
    );

    let rows = public_receipt().as_array().expect("compiler receipt rows");
    assert_eq!(rows.len(), 30);
    for spec in COMMON_SPECS {
        let row = rows
            .iter()
            .find(|row| row["name"].as_str() == Some(spec.public_declaration))
            .unwrap_or_else(|| panic!("compiler omitted {}", spec.public_declaration));
        assert_eq!(
            row["source_path"].as_str(),
            Some(spec.source_path),
            "{} path",
            spec.item_id
        );
        assert_eq!(
            row["line"].as_u64(),
            Some(spec.line),
            "{} line",
            spec.item_id
        );
        assert_eq!(
            row["type_sha256"].as_str(),
            Some(spec.type_sha256),
            "{} type",
            spec.item_id
        );
        assert_eq!(
            row["axioms"],
            serde_json::json!(AXIOMS),
            "{} axioms",
            spec.item_id
        );
        for exact in [
            spec.item_id.to_owned(),
            spec.anchor.to_owned(),
            spec.public_declaration.to_owned(),
            format!("{}#L{}", spec.source_path, spec.line),
            spec.type_sha256.to_owned(),
            spec.obligation.to_owned(),
        ] {
            assert!(contract.contains(&exact), "contract omits `{exact}`");
        }
    }
}

#[test]
fn common_machinery_contract_freezes_complete_cards_and_distinct_exercise_targets() {
    let contract = wave4_contract();
    let normalized = normalized_text(&contract);
    for field in CARD_FIELDS {
        assert!(
            contract.contains(&format!("`{field}`")),
            "contract omits card field `{field}`"
        );
    }
    let solutions = COMMON_SPECS
        .iter()
        .copied()
        .map(exercise_declaration)
        .collect::<Vec<_>>();
    assert_eq!(solutions.iter().collect::<BTreeSet<_>>().len(), 30);
    for (spec, solution) in COMMON_SPECS.iter().copied().zip(&solutions) {
        assert!(contract.contains(&exercise_id(spec)));
        assert!(contract.contains(solution));
        let valid = future_card(spec);
        let valid_errors = future_card_errors(&valid, spec);
        assert!(
            valid_errors.is_empty(),
            "{} valid parser fixture failed: {valid_errors:#?}",
            spec.item_id
        );
        for field in CARD_FIELDS {
            let mutated = valid.replacen(&format!("#### {field}\n"), "", 1);
            assert!(
                future_card_errors(&mutated, spec)
                    .iter()
                    .any(|error| error.contains(field)),
                "{} accepted a card missing {field}",
                spec.item_id
            );
        }
        let duplicate_anchor = format!("{valid}\n{{#{}}}\n", spec.anchor);
        assert!(future_card_errors(&duplicate_anchor, spec)
            .iter()
            .any(|error| error.contains(spec.anchor)));
    }

    let valid = future_card(COMMON_SPECS[0]);
    let reordered = valid
        .replacen("#### Purpose", "#### TEMP", 1)
        .replacen("#### Statement", "#### Purpose", 1)
        .replacen("#### TEMP", "#### Statement", 1);
    assert!(!future_card_errors(&reordered, COMMON_SPECS[0]).is_empty());
    for (needle, replacement, expected) in [
        (
            "$$\\operatorname{commonStep}_{CFT-25-001}(input) \\le output,$$",
            "words only",
            "equation",
        ),
        ("Source boundary:", "Source:", "Source boundary:"),
        ("Exact transfer:", "Transfer:", "Exact transfer:"),
        ("Compiler receipt:", "Receipt:", "Compiler receipt:"),
    ] {
        let mutated = valid.replacen(needle, replacement, 1);
        assert!(
            future_card_errors(&mutated, COMMON_SPECS[0])
                .iter()
                .any(|error| error.contains(expected)),
            "canonical card parser accepted mutation removing {needle}"
        );
    }
    let generic_equation = valid.replacen(
        "$$\\operatorname{commonStep}_{CFT-25-001}(input) \\le output,$$",
        "$$x=x$$",
        1,
    );
    assert!(future_card_errors(&generic_equation, COMMON_SPECS[0])
        .iter()
        .any(|error| error.contains("equation")));
    let fabricated_hash = valid.replacen(COMMON_SPECS[0].type_sha256, "fabricated", 1);
    assert!(future_card_errors(&fabricated_hash, COMMON_SPECS[0])
        .iter()
        .any(|error| error.contains("compiler field")));

    for exercise in EXERCISE_SPECS {
        let declaration = exercise_declaration_for(exercise);
        for exact in [
            exercise.exercise_id,
            exercise.prompt,
            exercise.reconstruction,
            exercise.statement_shape,
            declaration.as_str(),
        ] {
            assert!(contract.contains(exact), "contract omits `{exact}`");
        }
        for provider in exercise.reconstruction_target_providers {
            assert!(contract.contains(provider), "contract omits `{provider}`");
        }
        assert!(
            contract
                .replace("\\|", "|")
                .contains(exercise_lean_type(exercise.exercise_id)),
            "contract omits compiler-comparable proposition for {}",
            exercise.exercise_id
        );
    }

    let mut duplicate = solutions.clone();
    duplicate[29] = duplicate[0].clone();
    assert_ne!(duplicate.iter().collect::<BTreeSet<_>>().len(), 30);
    for required in [
        "statement fingerprint different from every public theorem card",
        "must compile as a theorem, never as a direct or eta alias",
        "Complete written solution",
        "compiler-reported source line, normalized type, type fingerprint",
    ] {
        assert!(normalized.contains(required), "contract omits `{required}`");
    }

    let exercises = read_json(&contracts_root().join("exercises.json"));
    let chapter_26 = chapter_markdown(26);
    for row in array(&exercises, "exercises", "exercise rows")
        .iter()
        .filter(|row| row["chapter"].as_u64() == Some(26))
    {
        let exercise_id = row["exercise_id"].as_str().expect("exercise id");
        let line = row["lean_solution"]["line"]
            .as_u64()
            .expect("compiler-reported exercise solution line");
        let locator =
            format!("../../../formalization/lean/CrouzeixTextbook/Part05/Chapter26.lean#L{line}");
        assert!(
            chapter_26.contains(&locator),
            "{exercise_id} prose omits exact compiler-bound locator `{locator}`"
        );
        let region = exercise_region(&chapter_26, exercise_id)
            .unwrap_or_else(|| panic!("missing unique exercise region for {exercise_id}"));
        let solution = &row["lean_solution"];
        let declaration = solution["declaration"].as_str().expect("declaration");
        let type_sha256 = solution["type_sha256"].as_str().expect("type hash");
        let verification_target = solution["verification_target"]
            .as_str()
            .expect("verification target");
        for exact in [
            declaration.to_owned(),
            locator,
            type_sha256.to_owned(),
            "Axioms: `Classical.choice, Quot.sound, propext`.".to_owned(),
            format!("Verification target: `{verification_target}`."),
            format!("Receipt identity: `{verification_target}:{type_sha256}`."),
        ] {
            assert!(
                region.contains(&exact),
                "{exercise_id} prose omits compiler receipt field `{exact}`"
            );
        }
    }
}

#[test]
fn common_machinery_contract_preserves_the_dag_and_independent_branch_seeds() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let graph = graph_from_coverage(&coverage);
    assert!(
        graph_is_acyclic(&graph),
        "canonical pedagogical graph has a cycle"
    );
    assert!(
        branches_are_independent(&graph),
        "Jin and LS pedagogical branches cross"
    );

    for spec in COMMON_SPECS {
        if spec.item_id == "CFT-28-006" {
            assert_eq!(
                graph[spec.item_id],
                BTreeSet::from([
                    "CFT-09-005".to_owned(),
                    "CFT-28-005".to_owned(),
                    "CFT-30-002".to_owned(),
                    "CFT-32-001".to_owned(),
                ]),
                "CFT-28-006 must expose its exact deferred Jin reader links"
            );
            continue;
        }
        assert!(
            graph[spec.item_id].iter().all(|dependency| {
                dependency
                    .strip_prefix("CFT-")
                    .and_then(|rest| rest.get(..2))
                    .and_then(|chapter| chapter.parse::<u64>().ok())
                    .is_some_and(|chapter| chapter < 30)
            }),
            "{} depends on a terminal-route chapter",
            spec.item_id
        );
    }
    assert_eq!(
        graph["CFT-30-001"],
        BTreeSet::from(["CFT-28-005".to_owned(), "CFT-29-002".to_owned()])
    );
    assert_eq!(
        graph["CFT-33-001"],
        BTreeSet::from([
            "CFT-23-004".to_owned(),
            "CFT-23-005".to_owned(),
            "CFT-29-002".to_owned(),
        ])
    );

    let mut crossed = graph.clone();
    crossed
        .get_mut("CFT-30-001")
        .expect("Jin seed")
        .insert("CFT-33-001".to_owned());
    assert!(!branches_are_independent(&crossed));

    let mut cyclic = graph.clone();
    cyclic
        .get_mut("CFT-29-001")
        .expect("CFT-29-001")
        .insert("CFT-29-002".to_owned());
    assert!(!graph_is_acyclic(&cyclic));

    let contract = wave4_contract();
    let normalized = normalized_text(&contract);
    for required in [
        "CFT-30-001 | CFT-28-005, CFT-29-002",
        "CFT-33-001 | CFT-23-004, CFT-23-005, CFT-29-002",
        "neither seed may depend on the other branch",
        "Comparison prose is not a pedagogical or kernel edge.",
    ] {
        assert!(normalized.contains(required), "contract omits `{required}`");
    }
}

#[test]
fn common_machinery_contract_closure_excludes_terminal_provider_namespaces() {
    let output = run_common_closure(&workspace_root().join("formalization/lean"), &COMMON_ROOTS);
    assert!(
        output.status.success(),
        "common closure failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let modules: Vec<String> = serde_json::from_slice(&output.stdout).expect("common closure JSON");
    assert_eq!(modules.len(), COMMON_CLOSURE_COUNT);
    let digest = format!("{:x}", Sha256::digest(modules.join("\n").as_bytes()));
    assert_eq!(digest, COMMON_CLOSURE_SHA256);
    assert!(modules.iter().all(|module| {
        !module.starts_with("Crouzeix.Jin")
            && !module.starts_with("Crouzeix.LoristSchwenninger")
            && !module.starts_with("Crouzeix.Harp")
            && !matches!(
                module.as_str(),
                "CrouzeixJin" | "CrouzeixLoristSchwenninger" | "CrouzeixHarp"
            )
    }));
    let contract = wave4_contract();
    assert!(contract.contains(&format!(
        "| common-machinery | {COMMON_CLOSURE_COUNT} | `{COMMON_CLOSURE_SHA256}` |"
    )));

    for forbidden in [
        "Crouzeix.Jin.Terminal",
        "Crouzeix.LoristSchwenninger.MainTheorem",
        "Crouzeix.Harp.MainTheorem",
        "CrouzeixJin",
        "CrouzeixLoristSchwenninger",
        "CrouzeixHarp",
    ] {
        let direct = tempfile::tempdir().expect("direct provider mutation root");
        write_module(
            direct.path(),
            "CommonRoot",
            &format!("import {forbidden}\n"),
        );
        let rejected = run_common_closure(direct.path(), &["CommonRoot"]);
        assert_eq!(
            rejected.status.code(),
            Some(7),
            "direct import of {forbidden} was accepted"
        );
        assert!(String::from_utf8_lossy(&rejected.stderr)
            .contains(&format!("rejected provider import {forbidden}")));

        let transitive = tempfile::tempdir().expect("transitive provider mutation root");
        write_module(
            transitive.path(),
            "CommonRoot",
            "import CrouzeixTextbook.Part05.Hidden\n",
        );
        write_module(
            transitive.path(),
            "CrouzeixTextbook.Part05.Hidden",
            &format!("import {forbidden}\n"),
        );
        let rejected = run_common_closure(transitive.path(), &["CommonRoot"]);
        assert_eq!(
            rejected.status.code(),
            Some(7),
            "transitive import of {forbidden} was accepted"
        );
        assert!(String::from_utf8_lossy(&rejected.stderr)
            .contains(&format!("rejected provider import {forbidden}")));
    }

    let comments = tempfile::tempdir().expect("comment import mutation root");
    write_module(
        comments.path(),
        "CommonRoot",
        "/- import Crouzeix.Harp.MainTheorem\nimport CrouzeixLoristSchwenninger -/\n-- import Crouzeix.Jin.Terminal\n-- import CrouzeixHarp\n",
    );
    let accepted = run_common_closure(comments.path(), &["CommonRoot"]);
    assert!(accepted.status.success(), "comments were parsed as imports");
}

#[test]
fn common_machinery_contract_names_the_canonical_chapter_and_exercise_roster() {
    let contract = wave4_contract();
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let exercise_rows = rows_for_chapters(&exercises, "exercises");
    assert_eq!(exercise_rows.len(), 30);
    for spec in COMMON_SPECS {
        let chapter_path = if spec.chapter <= 28 {
            format!(
                "part_05_crouzeix_machinery/{:02}_{}",
                spec.chapter,
                match spec.chapter {
                    25 => "convex_boundaries_and_cauchy_layers.md",
                    26 => "double_layer_map.md",
                    27 => "one_plus_sqrt_two_barrier.md",
                    28 => "complete_power_family.md",
                    _ => unreachable!(),
                }
            )
        } else {
            "part_06_constant_two_routes/29_crouzeix_problem_and_sharpness.md".to_owned()
        };
        assert!(
            packet_root().join(&chapter_path).is_file(),
            "missing {chapter_path}"
        );
        assert!(contract.contains(&chapter_path));
        let id = exercise_id(spec);
        assert_eq!(
            exercise_rows
                .iter()
                .filter(|row| row["exercise_id"].as_str() == Some(id.as_str()))
                .count(),
            1,
            "{id} roster count"
        );
    }
}
