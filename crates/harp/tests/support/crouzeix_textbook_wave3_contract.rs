use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};
use std::sync::OnceLock;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::{
    array, contracts_root, packet_root, read_json, receipt_type_hash, string, workspace_root,
    CHAPTERS,
};

const WAVE3_CONTRACT: &str =
    "docs/superpowers/specs/2026-08-27-crouzeix-textbook-ls-comparison-contract.md";
const CARD_FIELDS: [&str; 10] = [
    "Purpose",
    "Statement",
    "Hypothesis ledger",
    "Proof roadmap",
    "Proof",
    "Boundary case",
    "Pedagogical prerequisites",
    "Lean correspondence",
    "Historical context",
    "ML analogy",
];
const AXIOMS: [&str; 3] = ["Classical.choice", "Quot.sound", "propext"];
const JIN_TERMINAL: &str = "CrouzeixConjecture.jinFinalCrouzeixConjecture";
const LS_TERMINAL: &str = "CrouzeixConjecture.loristSchwenningerMainTheorem";
const HARP_TERMINAL: &str = "CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem";
const THREE_ROUTE_BUNDLE: &str = "CrouzeixTextbook.Part06.three_route_terminal_bundle";
const CHAPTER34_E06_CONSTRUCTOR: &str =
    "CrouzeixConjecture.LoristSchwenninger.dilationDataOfParametricPolynomial";
const CHAPTER34_E06_ENDPOINT: &str =
    "CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two";
const AUDITED_EXTERNAL_PROOF_DEPENDENCIES: [&str; 2] = [
    "circleIntegral.integral_sub_inv_of_mem_ball",
    "circleIntegral.integral_sub_zpow_of_ne",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Wave3Phase {
    BaselineFrozen,
    Chapter34BoundaryData,
    Chapter34Moments,
    Chapter34RealizationComplete,
    Chapter35Consequences,
    ThreeRouteComparisonComplete,
}

impl Wave3Phase {
    const ALL: [Self; 6] = [
        Self::BaselineFrozen,
        Self::Chapter34BoundaryData,
        Self::Chapter34Moments,
        Self::Chapter34RealizationComplete,
        Self::Chapter35Consequences,
        Self::ThreeRouteComparisonComplete,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::BaselineFrozen => "baseline-frozen",
            Self::Chapter34BoundaryData => "chapter-34-boundary-data",
            Self::Chapter34Moments => "chapter-34-moments",
            Self::Chapter34RealizationComplete => "chapter-34-realization-complete",
            Self::Chapter35Consequences => "chapter-35-consequences",
            Self::ThreeRouteComparisonComplete => "three-route-comparison-complete",
        }
    }

    fn rank(self) -> u8 {
        match self {
            Self::BaselineFrozen => 0,
            Self::Chapter34BoundaryData => 1,
            Self::Chapter34Moments => 2,
            Self::Chapter34RealizationComplete => 3,
            Self::Chapter35Consequences => 4,
            Self::ThreeRouteComparisonComplete => 5,
        }
    }

    fn parse(value: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|phase| phase.label() == value)
    }
}

#[derive(Clone, Copy)]
struct CftSpec {
    item_id: &'static str,
    chapter: u64,
    baseline_kind: &'static str,
    baseline_mode: &'static str,
    baseline_correspondence: &'static str,
    public_declaration: &'static str,
    public_file: &'static str,
    provider_declaration: &'static str,
    provider_file: &'static str,
    baseline_type_sha256: &'static str,
    compiler_public_type_sha256: &'static str,
    completed_kind: &'static str,
    completed_mode: &'static str,
    completion_rank: u8,
    hypotheses: &'static [&'static str],
    statement_steps: &'static [&'static str],
    proof_steps: &'static [&'static str],
}

const CFT_SPECS: [CftSpec; 12] = [
    CftSpec {
        item_id: "CFT-34-001",
        chapter: 34,
        baseline_kind: "definition",
        baseline_mode: "checkpoint",
        baseline_correspondence: "checkpoint",
        public_declaration: "CrouzeixTextbook.Part06.boundary_dilation_data",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean",
        provider_declaration: "CrouzeixConjecture.LoristSchwenninger.dilationDataOfParametricPolynomial",
        provider_file: "formalization/lean/Crouzeix/LoristSchwenninger/ConcreteDilation.lean",
        baseline_type_sha256: "6a6fc2824a8faaa4dc9cd85fb9fe476145818054e33772c18188d7a5d253e612",
        compiler_public_type_sha256: "b8dabd71a831f9daaa373f3d7938aa9bc00087592d81975e00a97061af17cb7e",
        completed_kind: "definition",
        completed_mode: "definition",
        completion_rank: 1,
        hypotheses: &[
            "i n : Type*",
            "TopologicalSpace i",
            "CompactSpace i",
            "MeasurableSpace i",
            "BorelSpace i",
            "OpensMeasurableSpace i",
            "SecondCountableTopologyEither i ℂ",
            "Fintype n",
            "DecidableEq n",
            "Nonempty n",
            "μ : Measure i",
            "IsFiniteMeasure μ",
            "Ω : Set ℂ",
            "EuclideanVector n finite-dimensional complete",
            "L2(μ; EuclideanVector n) complete",
            "Γ : ParametricConvexBoundary Ω",
            "B : SquareMatrix n",
            "hWB : numericalRange B ⊆ Ω",
            "q : Polynomial ℂ",
            "hq : ∀ z ∈ closure Ω, ‖Polynomial.eval z q‖ ≤ 1",
            "hCauchy : HasParametricPolynomialCauchyFormula Γ μ B",
        ],
        statement_steps: &[
            "D := parametricPositiveBoundaryDensity",
            "V := (boundaryEmbedding D).toContinuousLinearMap",
            "Q := bcfMulL h",
            "C_k := ∫ σ, star ((h^k) σ) • F(σ) ∂μ",
            "E_k := euclideanOperator C_k",
            "DilationData",
        ],
        proof_steps: &[
            "V_isometry",
            "Q_norm_le_one",
            "perturbation_eq",
            "bound_nonneg",
            "perturbation_norm_le",
            "commutes_with_target",
        ],
    },
    CftSpec {
        item_id: "CFT-34-002",
        chapter: 34,
        baseline_kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.boundary_compression_first_moment",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean",
        provider_declaration: "CrouzeixConjecture.LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_comp_boundaryEmbedding",
        provider_file: "formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean",
        baseline_type_sha256: "c239874c55c1aed80e22529c2a780b3a0728e561c65e677a28505ac45511965e",
        compiler_public_type_sha256: "2a414f51546bd96dd1afcd5d0bbe93fea39684dff0c35843da4b85b9282c1e14",
        completed_kind: "theorem",
        completed_mode: "reexported-proof",
        completion_rank: 2,
        hypotheses: &["positive boundary density D", "bounded continuous h"],
        statement_steps: &["V* M_h V", "boundaryPhiCLM D h", "euclideanOperator"],
        proof_steps: &[
            "expand the L2 inner product",
            "insert the square root field",
            "use D^(1/2)D^(1/2)=D",
            "move the integral through the continuous linear map",
        ],
    },
    CftSpec {
        item_id: "CFT-34-003",
        chapter: 34,
        baseline_kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.boundary_compression_power_moments",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean",
        provider_declaration: "CrouzeixConjecture.LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding",
        provider_file: "formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean",
        baseline_type_sha256: "fd451caedfa1808e5ce7a824522d7fb5a613eb3add9e988b47d1d851b5ceee55",
        compiler_public_type_sha256: "4fb3020149c49a8b8a0a126673e03400b99e295b8bab79a60ea8adfcffafcc73",
        completed_kind: "theorem",
        completed_mode: "reexported-proof",
        completion_rank: 2,
        hypotheses: &["positive boundary density D", "bounded continuous h", "k : Nat"],
        statement_steps: &["V* (M_h)^k V", "boundaryPhiCLM D (h^k)"],
        proof_steps: &[
            "rewrite bcfMulL_pow",
            "apply the first-moment compression identity to h^k",
            "explain why k=1 alone is insufficient",
        ],
    },
    CftSpec {
        item_id: "CFT-34-004",
        chapter: 34,
        baseline_kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.boundary_multiplier_powers",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean",
        provider_declaration: "CrouzeixConjecture.LoristSchwenninger.bcfMulL_pow",
        provider_file: "formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean",
        baseline_type_sha256: "f723bad3e2f1b0cfed8531b208a8de1492201fdf88439726d83fcb8166a738c3",
        compiler_public_type_sha256: "735335a245a1a4b3bac89546d203b2b856f97029284167c36259e5f6fbc805e3",
        completed_kind: "theorem",
        completed_mode: "reexported-proof",
        completion_rank: 3,
        hypotheses: &["bounded continuous h", "k : Nat"],
        statement_steps: &["M_(h^k)", "(M_h)^k"],
        proof_steps: &["base k=0", "pow_succ", "bcfMulL_mul", "continuous-linear-map extensionality"],
    },
    CftSpec {
        item_id: "CFT-34-005",
        chapter: 34,
        baseline_kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.boundary_multiplier_contractive",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean",
        provider_declaration: "CrouzeixConjecture.LoristSchwenninger.bcfMulL_norm_le_one",
        provider_file: "formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean",
        baseline_type_sha256: "20864fd4143c209116e93458a00e0f812db607dd0c84a5bea02cbe9334155c37",
        compiler_public_type_sha256: "1ae87992f1e7a9b66a79358fcafa7247061979381845fc4132e07b5768ce67f1",
        completed_kind: "theorem",
        completed_mode: "reexported-proof",
        completion_rank: 3,
        hypotheses: &["bounded continuous h", "norm h <= 1"],
        statement_steps: &["norm (M_h) <= norm h", "norm (M_h) <= 1"],
        proof_steps: &["pointwise norm bound", "Lp norm bound", "transitivity"],
    },
    CftSpec {
        item_id: "CFT-34-006",
        chapter: 34,
        baseline_kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.realization_norm_two",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean",
        provider_declaration: "CrouzeixConjecture.LoristSchwenninger.norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary",
        provider_file: "formalization/lean/Crouzeix/LoristSchwenninger/ConcreteDilation.lean",
        baseline_type_sha256: "5726b7d2562ac08d0a520d8ccc5440f9a72b6f9aa978ac2da5c723a552ef4596",
        compiler_public_type_sha256: "84ab420a345d504769f8df1067adfc0424dad4b18d2e58e4ef3d4c71ebc3654f",
        completed_kind: "theorem",
        completed_mode: "reexported-proof",
        completion_rank: 3,
        hypotheses: &["W(B) subset Omega", "norm q <= 1 on closure Omega", "power Cauchy formula"],
        statement_steps: &["norm (euclideanOperator (polynomialEval q B)) <= 2"],
        proof_steps: &["construct dilationDataOfParametricPolynomial", "apply Chapter 33 perturbation endpoint", "identify the normalized polynomial evaluation"],
    },
    CftSpec {
        item_id: "CFT-35-001",
        chapter: 35,
        baseline_kind: "definition",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.lorist_schwenninger_main",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean",
        provider_declaration: "CrouzeixConjecture.loristSchwenningerMainTheorem",
        provider_file: "formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean",
        baseline_type_sha256: "eaa8ca24d59d173f34972e80bdb20acf20d1c78178f2171ac28bbc6688a1960b",
        compiler_public_type_sha256: "eaa8ca24d59d173f34972e80bdb20acf20d1c78178f2171ac28bbc6688a1960b",
        completed_kind: "theorem",
        completed_mode: "reexported-proof",
        completion_rank: 4,
        hypotheses: &["finite nonempty index type", "A", "polynomial p"],
        statement_steps: &["norm (polynomialEval p A)", "2 * maxPolynomialModulusOnNumericalRange A p"],
        proof_steps: &["fix an outer domain", "normalize the polynomial", "apply CFT-34-006", "pass simple-spectrum limit", "pass outer-domain limit"],
    },
    CftSpec {
        item_id: "CFT-35-002",
        chapter: 35,
        baseline_kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.lorist_schwenninger_finite_matrix",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean",
        provider_declaration: "CrouzeixConjecture.loristSchwenningerFiniteMatrixMainTheorem",
        provider_file: "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean",
        baseline_type_sha256: "606baa75394270e977e5ddb6e843205dd71e60ae005c408ea5a71884c1907776",
        compiler_public_type_sha256: "606baa75394270e977e5ddb6e843205dd71e60ae005c408ea5a71884c1907776",
        completed_kind: "theorem",
        completed_mode: "reexported-proof",
        completion_rank: 4,
        hypotheses: &["d : Nat", "Nonempty (Fin d)"],
        statement_steps: &["FiniteMatrixMainTheoremStatement", "Fin d"],
        proof_steps: &["specialize CFT-35-001", "identify SquareMatrix (Fin d)"],
    },
    CftSpec {
        item_id: "CFT-35-003",
        chapter: 35,
        baseline_kind: "theorem",
        baseline_mode: "reexported-proof",
        baseline_correspondence: "unmapped",
        public_declaration: "CrouzeixTextbook.Part06.lorist_schwenninger_rational",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean",
        provider_declaration: "CrouzeixConjecture.loristSchwenningerRationalSpectralSetCorollary",
        provider_file: "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean",
        baseline_type_sha256: "a68683384ff3984804bccbfaf7f5dea8ca6128d26a0618984bc5d293837f0144",
        compiler_public_type_sha256: "a68683384ff3984804bccbfaf7f5dea8ca6128d26a0618984bc5d293837f0144",
        completed_kind: "theorem",
        completed_mode: "reexported-proof",
        completion_rank: 4,
        hypotheses: &["rational r", "poles avoid W(A)"],
        statement_steps: &["norm (rationalMatrixEval r A)", "2 * maxRationalModulusOnNumericalRange A r"],
        proof_steps: &[
            "polynomial approximants",
            "uniform scalar convergence",
            "matrix evaluation convergence",
            "maximum convergence",
            "limit inequality",
        ],
    },
    CftSpec {
        item_id: "CFT-35-004",
        chapter: 35,
        baseline_kind: "theorem",
        baseline_mode: "checkpoint",
        baseline_correspondence: "checkpoint",
        public_declaration: "CrouzeixTextbook.Part06.lorist_schwenninger_hilbert",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean",
        provider_declaration: "CrouzeixConjecture.loristSchwenningerHilbertSpacePolynomialCrouzeix",
        provider_file: "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean",
        baseline_type_sha256: "d93d63c731ea1723a81d71d06845c000a240f960e67c6f3114079e64fff4627a",
        compiler_public_type_sha256: "ff3d2b377c9cc61c89686022d9e9146cf351ae90293d4036a9e12c4a2b2cc17d",
        completed_kind: "theorem",
        completed_mode: "reexported-proof",
        completion_rank: 4,
        hypotheses: &["complete nontrivial complex Hilbert space H", "bounded operator A", "polynomial p"],
        statement_steps: &["operatorPolynomialEval p A", "2 * supPolynomialModulusOnOperatorNumericalRange A p"],
        proof_steps: &["finite-dimensional range model", "Fin m coordinate matrix", "transport back"],
    },
    CftSpec {
        item_id: "CFT-35-005",
        chapter: 35,
        baseline_kind: "theorem",
        baseline_mode: "checkpoint",
        baseline_correspondence: "checkpoint",
        public_declaration: "CrouzeixTextbook.Part06.lorist_schwenninger_two_spectral_set",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean",
        provider_declaration: "CrouzeixConjecture.loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet",
        provider_file: "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean",
        baseline_type_sha256: "d12d0a2f2deb6ce66f803a1ca394b125f2c16d3756570d5b63ab701e933dad77",
        compiler_public_type_sha256: "391ee68292160554f5ba1ad54719f4bdab6266357b767a99ff690a17ad111904",
        completed_kind: "theorem",
        completed_mode: "reexported-proof",
        completion_rank: 4,
        hypotheses: &["complete nontrivial complex Hilbert space H", "bounded operator A"],
        statement_steps: &[
            "IsCompact (closedOperatorNumericalRange A)",
            "spectrum ℂ A ⊆ closedOperatorNumericalRange A",
            "rational constant-two inequality",
        ],
        proof_steps: &[
            "closedOperatorNumericalRange_isCompact",
            "spectrum_subset_closedOperatorNumericalRange_of_mainTheorem",
            "hilbertSpaceRationalCrouzeix_of_mainTheorem",
        ],
    },
    CftSpec {
        item_id: "CFT-35-006",
        chapter: 35,
        baseline_kind: "theorem",
        baseline_mode: "checkpoint",
        baseline_correspondence: "checkpoint",
        public_declaration: "CrouzeixTextbook.Part06.jin_final_comparator",
        public_file: "formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean",
        provider_declaration: "CrouzeixConjecture.jinFinalCrouzeixConjecture",
        provider_file: "formalization/lean/CrouzeixConjecture/FinalTheorems.lean",
        baseline_type_sha256: "0c2cdfce0642e8d4ab88dc860b6a60e85be855897e5f63fd51babbf67c87fc5d",
        compiler_public_type_sha256: "5a08b37106dc806e9cb5f69cb03f5e13bd850055d67cb410ad615924faaab9c2",
        completed_kind: "theorem",
        completed_mode: "proved-here",
        completion_rank: 5,
        hypotheses: &["finite nonempty index type"],
        statement_steps: &["Jin MainTheoremStatement", "Lorist-Schwenninger MainTheoremStatement", "Harp MainTheoremStatement"],
        proof_steps: &["jinFinalCrouzeixConjecture", "loristSchwenningerMainTheorem", "harpFiniteHorizonMainTheorem"],
    },
];

#[derive(Clone, Copy)]
struct ExerciseSpec {
    exercise_id: &'static str,
    chapter: u64,
    parent: &'static str,
    probe_declaration: &'static str,
    solution: &'static str,
    completion_rank: u8,
    allowed_cfts: &'static [&'static str],
    /// Constants that must occur in the compiled proof-body closure. Constants
    /// used only by the theorem type are checked separately against the
    /// compiler-normalized statement.
    required_dependencies: &'static [&'static str],
    canonical_type_sha256: Option<&'static str>,
}

const EXERCISE_SPECS: [ExerciseSpec; 12] = [
    ExerciseSpec { exercise_id: "CFT-34-E01", chapter: 34, parent: "CFT-34-001", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_34_e01", solution: "CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_01_solution", completion_rank: 1, allowed_cfts: &[], required_dependencies: &["CrouzeixConjecture.LoristSchwenninger.norm_boundaryEmbeddingToLp"], canonical_type_sha256: Some("753cce1e54cadc7157f3f66fa425d0db3c53bbb2b7122291659f72ee9b46a4e2") },
    ExerciseSpec { exercise_id: "CFT-34-E02", chapter: 34, parent: "CFT-34-002", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_34_e02", solution: "CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_02_solution", completion_rank: 1, allowed_cfts: &["CFT-34-001"], required_dependencies: &["CrouzeixConjecture.LoristSchwenninger.boundaryEmbeddingField_memLp", "CrouzeixConjecture.LoristSchwenninger.bcfMulL_apply_ae", "CrouzeixConjecture.LoristSchwenninger.boundarySquareRoot_mul_self_ae", "CrouzeixConjecture.boundaryPhiCLM_apply"], canonical_type_sha256: Some("7bb43df815f450a28993129b412f57183d1e79d87e8e2dfef2ff7e7551fabcbe") },
    ExerciseSpec { exercise_id: "CFT-34-E03", chapter: 34, parent: "CFT-34-002", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_34_e03", solution: "CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_03_solution", completion_rank: 2, allowed_cfts: &[], required_dependencies: &["circleIntegral.integral_sub_inv_of_mem_ball", "circleIntegral.integral_sub_zpow_of_ne"], canonical_type_sha256: Some("76f4a4e504f0fcf2ef49c9dc4fd5b407904d698e407e01d9b103849adaf72c8f") },
    ExerciseSpec { exercise_id: "CFT-34-E04", chapter: 34, parent: "CFT-34-003", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_34_e04", solution: "CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_04_solution", completion_rank: 2, allowed_cfts: &["CFT-34-002", "CFT-34-004"], required_dependencies: &["CrouzeixConjecture.LoristSchwenninger.bcfMulL_pow", "CrouzeixConjecture.LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_comp_boundaryEmbedding"], canonical_type_sha256: Some("51681236d59868126e8c021ca57f255e2f3fc5fc36d6c48810d332a657a9308c") },
    ExerciseSpec { exercise_id: "CFT-34-E05", chapter: 34, parent: "CFT-34-005", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_34_e05", solution: "CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_05_solution", completion_rank: 3, allowed_cfts: &["CFT-34-004"], required_dependencies: &["CrouzeixConjecture.LoristSchwenninger.bcfMulL_norm_le"], canonical_type_sha256: Some("3a48557c92ad6fbc3143c184346c90ab47a189e0f62207a0ae31d06ab0d64f30") },
    ExerciseSpec { exercise_id: "CFT-34-E06", chapter: 34, parent: "CFT-34-006", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_34_e06", solution: "CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_06_solution", completion_rank: 3, allowed_cfts: &["CFT-33-006", "CFT-34-001"], required_dependencies: &["CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two"], canonical_type_sha256: Some("20ab40086adab31213e3adb0ad72e2912e1a9301373f3c2a7e1fe9c996a6b992") },
    ExerciseSpec { exercise_id: "CFT-35-E01", chapter: 35, parent: "CFT-35-001", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_35_e01", solution: "CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_01_solution", completion_rank: 4, allowed_cfts: &["CFT-34-006"], required_dependencies: &[], canonical_type_sha256: Some("fa6192e91ba075de59d54b56939088015448d45ffca20538750d74eeec09acc0") },
    ExerciseSpec { exercise_id: "CFT-35-E02", chapter: 35, parent: "CFT-35-002", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_35_e02", solution: "CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_02_solution", completion_rank: 4, allowed_cfts: &["CFT-35-001"], required_dependencies: &[], canonical_type_sha256: Some("d4f53d2f56895a7d7ea5fd383b2a2beb56d87a9bc32833ff8be272b015e58ca3") },
    ExerciseSpec { exercise_id: "CFT-35-E03", chapter: 35, parent: "CFT-35-003", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_35_e03", solution: "CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_03_solution", completion_rank: 4, allowed_cfts: &["CFT-35-001"], required_dependencies: &["CrouzeixConjecture.rationalSpectralSetCorollary_of_mainTheorem"], canonical_type_sha256: Some("b2d2f068fe54808f15b51e1fb893a73d548b00c58cda6338350afb3262bcfe96") },
    ExerciseSpec { exercise_id: "CFT-35-E04", chapter: 35, parent: "CFT-35-004", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_35_e04", solution: "CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_04_solution", completion_rank: 4, allowed_cfts: &["CFT-35-002"], required_dependencies: &["CrouzeixConjecture.hilbertSpacePolynomialCrouzeix_of_mainTheorem"], canonical_type_sha256: Some("14a4eb4cca259753f3f8d025e69fc3c176b5639f305ae361a00630482525a847") },
    ExerciseSpec { exercise_id: "CFT-35-E05", chapter: 35, parent: "CFT-35-005", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_35_e05", solution: "CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_05_solution", completion_rank: 4, allowed_cfts: &["CFT-35-003", "CFT-35-004"], required_dependencies: &["CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem"], canonical_type_sha256: Some("bc28757a0755dbb1c831f00eac266609ccf270b59cd7baed981b1962f1fff01d") },
    ExerciseSpec { exercise_id: "CFT-35-E06", chapter: 35, parent: "CFT-35-006", probe_declaration: "CrouzeixTextbook.Wave3ContractProbe.cft_35_e06", solution: "CrouzeixTextbook.Part06.Exercises.Chapter35.exercise_06_solution", completion_rank: 5, allowed_cfts: &[], required_dependencies: &[], canonical_type_sha256: Some("a6b5302087094c1fd6c8378f297d6ff79a995904d3e4a6be961a7fe5f3d8cdb7") },
];

fn wave3_contract() -> String {
    fs::read_to_string(workspace_root().join(WAVE3_CONTRACT)).expect("Wave 3 contract document")
}

fn active_phase(contract: &str) -> Wave3Phase {
    let label = contract
        .lines()
        .find_map(|line| line.strip_prefix("- Active phase: `")?.strip_suffix("`."))
        .expect("Wave 3 active phase");
    Wave3Phase::parse(label).unwrap_or_else(|| panic!("unknown Wave 3 phase {label}"))
}

fn cft_spec(item_id: &str) -> &'static CftSpec {
    CFT_SPECS
        .iter()
        .find(|spec| spec.item_id == item_id)
        .unwrap_or_else(|| panic!("unknown Wave 3 item {item_id}"))
}

fn completed_public(spec: &CftSpec) -> &'static str {
    if spec.item_id == "CFT-35-006" {
        THREE_ROUTE_BUNDLE
    } else {
        spec.public_declaration
    }
}

fn chapter_markdown(chapter: u64) -> String {
    fs::read_to_string(packet_root().join(CHAPTERS[(chapter - 1) as usize]))
        .unwrap_or_else(|error| panic!("Chapter {chapter} prose: {error}"))
}

fn card<'a>(markdown: &'a str, item_id: &str, chapter: u64) -> Option<&'a str> {
    let marker = format!("### {item_id} ");
    let start = markdown.find(&marker)?;
    let tail = &markdown[start..];
    let next = format!("\n### CFT-{chapter:02}-");
    let end = tail[marker.len()..]
        .find(&next)
        .map(|offset| marker.len() + offset)
        .or_else(|| tail.find("\n## Worked examples"))
        .unwrap_or(tail.len());
    Some(&tail[..end])
}

fn card_field<'a>(card: &'a str, field: &str) -> Option<&'a str> {
    let marker = format!("\n#### {field}\n");
    let start = card.find(&marker)?;
    let body = &card[start + marker.len()..];
    let end = body.find("\n#### ").unwrap_or(body.len());
    Some(body[..end].trim())
}

fn exercise<'a>(markdown: &'a str, exercise_id: &str) -> Option<&'a str> {
    let marker = format!("### {exercise_id} ");
    let start = markdown.find(&marker)?;
    let tail = &markdown[start..];
    let end = tail[marker.len()..]
        .find("\n### CFT-")
        .map(|offset| marker.len() + offset)
        .or_else(|| tail.find("\n### Solution sketches"))
        .unwrap_or(tail.len());
    Some(&tail[..end])
}

fn visible_markdown(input: &str) -> String {
    let mut visible = String::new();
    let mut rest = input;
    while let Some(start) = rest.find("<!--") {
        visible.push_str(&rest[..start]);
        let comment = &rest[start + 4..];
        let Some(end) = comment.find("-->") else {
            return visible;
        };
        rest = &comment[end + 3..];
    }
    visible.push_str(rest);
    visible
}

fn normalized_visible_markdown(input: &str) -> String {
    visible_markdown(input)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[derive(Clone, Copy)]
struct CardSemantics {
    purpose: &'static str,
    boundary: &'static str,
    history: &'static str,
    ml: &'static str,
    worked: &'static str,
}

fn card_semantics(item_id: &str) -> CardSemantics {
    match item_id {
        "CFT-34-001" => CardSemantics {
            purpose: "build the concrete dilation record",
            boundary: "zero polynomial q = 0",
            history: "Lorist--Schwenninger boundary realization",
            ml: "lifted feature-space realization",
            worked: "finite atomic boundary model: for i = Fin m with atom weights w_a, the boundary integral becomes ∑ a, w_a • D_a",
        },
        "CFT-34-002" => CardSemantics {
            purpose: "identify the compressed first boundary moment",
            boundary: "constant multiplier h = 0",
            history: "first compression moment",
            ml: "feature covariance compression",
            worked: "two-atom compression calculation: V* M_h V = w₀ V₀* h₀ V₀ + w₁ V₁* h₁ V₁",
        },
        "CFT-34-003" => CardSemantics {
            purpose: "upgrade the first moment to every power",
            boundary: "power k = 0",
            history: "power-moment compression",
            ml: "multi-step rollout moments",
            worked: "empirical moment residual R_k := V* M_h^k V - Φ(h^k)",
        },
        "CFT-34-004" => CardSemantics {
            purpose: "turn multiplier products into powers",
            boundary: "base case k = 0",
            history: "boundary multiplier algebra",
            ml: "tied linear-layer composition",
            worked: "worked k = 2 multiplication: M_h^2 f = h • (h • f) = h^2 • f",
        },
        "CFT-34-005" => CardSemantics {
            purpose: "certify contraction before dilation",
            boundary: "strict contraction ‖h‖ < 1 gives ‖M_h‖ < 1",
            history: "contractive boundary multiplier",
            ml: "Lipschitz certificate",
            worked: "worked bound ‖h‖ = ρ < 1 gives ‖M_h f‖ ≤ ρ ‖f‖",
        },
        "CFT-34-006" => CardSemantics {
            purpose: "feed the realization into the Chapter 33 endpoint",
            boundary: "q = 0 gives operator norm zero",
            history: "factor-two realization endpoint",
            ml: "robust operator-norm certificate",
            worked: "nonnormal 2 × 2 worked matrix: evaluate q(B), then compare its norm with the sampled boundary maximum",
        },
        "CFT-35-001" => CardSemantics {
            purpose: "remove the normalization and approximation scaffolding",
            boundary: "zero polynomial makes both sides zero",
            history: "main polynomial consequence",
            ml: "nonnormal layer functional bound",
            worked: "worked polynomial p(z) = z reduces the conclusion to ‖A‖ ≤ 2 max_{z∈W(A)} ‖z‖",
        },
        "CFT-35-002" => CardSemantics {
            purpose: "specialize the type-parametric theorem to Fin d",
            boundary: "d = 0 is excluded by Nonempty (Fin d)",
            history: "finite-matrix consequence adapter",
            ml: "width-indexed matrix theorem",
            worked: "worked dimension d = 2 instantiates SquareMatrix (Fin 2) without changing the constant",
        },
        "CFT-35-003" => CardSemantics {
            purpose: "extend polynomial control to pole-free rational functions",
            boundary: "an embedded polynomial is a rational boundary case",
            history: "rational approximation consequence",
            ml: "resolvent-filter certificate",
            worked: "one-pole rational approximant sequence q_N converges uniformly before q_N(A) and the maxima pass to their limits",
        },
        "CFT-35-004" => CardSemantics {
            purpose: "transport finite matrices to bounded Hilbert-space operators",
            boundary: "finite-dimensional H recovers the matrix theorem",
            history: "Hilbert-space transport consequence",
            ml: "infinite-width operator limit",
            worked: "finite-rank compression calculation compares P_d A P_d with its Fin m matrix model",
        },
        "CFT-35-005" => CardSemantics {
            purpose: "package compactness, spectral containment, and rational control",
            boundary: "a scalar operator makes all three components explicit",
            history: "closed numerical-range spectral-set consequence",
            ml: "three-part stability certificate",
            worked: "worked compactness-spectrum-bound triple checks the compact set, spectrum inclusion, and rational inequality separately",
        },
        "CFT-35-006" => CardSemantics {
            purpose: "compare three independently checked terminal routes",
            boundary: "propositional agreement is not proof-term identity",
            history: "three-route comparison, not a fourth proof",
            ml: "proof-architecture ablation",
            worked: "worked three-column comparison row records the common conclusion while preserving route-specific evidence",
        },
        _ => panic!("missing card semantics for {item_id}"),
    }
}

const COMPARISON_AXES: [&str; 8] = [
    "Objects.",
    "Hypotheses.",
    "Shared trunk.",
    "Decisive mechanism.",
    "Approximation order.",
    "Conclusion.",
    "Provenance.",
    "Formal provider.",
];

fn row_provenance(item_id: &str) -> (&'static str, &'static str) {
    match item_id {
        "CFT-34-001" | "CFT-34-002" | "CFT-34-003" | "CFT-34-004" | "CFT-34-005" | "CFT-34-006" => {
            (
                "LS source-derived",
                "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124",
            )
        }
        "CFT-35-001" => (
            "LS source-derived terminal",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128",
        ),
        "CFT-35-002" => (
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L20-L24",
        ),
        "CFT-35-003" => (
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L28-L33",
        ),
        "CFT-35-004" => (
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L37-L45",
        ),
        "CFT-35-005" => (
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L55-L61",
        ),
        "CFT-35-006" => (
            "comparison-only; Harp-derived component",
            "formalization/lean/Crouzeix/Harp/MainTheorem.lean#L25-L172",
        ),
        _ => panic!("missing row provenance for {item_id}"),
    }
}

fn future_card_fixture(spec: &CftSpec) -> String {
    let semantics = card_semantics(spec.item_id);
    let (provenance_class, provenance_locator) = row_provenance(spec.item_id);
    let hypotheses = spec.hypotheses.join("; ");
    let statement = spec.statement_steps.join("; ");
    let mut proof = format!(
        "{}; Worked calculation. {}",
        spec.proof_steps.join("; "),
        semantics.worked
    );
    if spec.item_id == "CFT-35-006" {
        proof.push_str("; ");
        proof.push_str(&COMPARISON_AXES.join(" "));
    }
    let comparison = if spec.item_id == "CFT-35-006" {
        format!("; {JIN_TERMINAL}; {LS_TERMINAL}; {HARP_TERMINAL}")
    } else {
        String::new()
    };
    format!(
        "### {} — completed contract fixture {{#cft-{}}}\n\n\
#### Purpose\n\nMotivation. This row is needed to {}.\n\n\
#### Statement\n\n{statement}\n\n\
#### Hypothesis ledger\n\n{hypotheses}\n\n\
#### Proof roadmap\n\nFollow the displayed obligations without invoking the endpoint.\n\n\
#### Proof\n\n{proof}\n\n\
#### Boundary case\n\n{}.\n\n\
#### Pedagogical prerequisites\n\nUse only the frozen pedagogical graph.\n\n\
#### Lean correspondence\n\nkind={}; mode={}; public={}; provider={}; public_file={}; provider_file={}{}\n\n\
#### Historical context\n\n{}; {}; {}; the row retains its own provenance boundary.\n\n\
#### ML analogy\n\nMathematical object / ML counterpart. {}. Exact transfer. The algebraic implication transfers exactly. Non-transfer. No empirical training claim follows. Diagnostic. The worked finite residual is diagnostic only.\n",
        spec.item_id,
        spec.item_id.strip_prefix("CFT-").expect("CFT item").to_ascii_lowercase(),
        semantics.purpose,
        semantics.boundary,
        spec.completed_kind,
        spec.completed_mode,
        completed_public(spec),
        spec.provider_declaration,
        spec.public_file,
        spec.provider_file,
        comparison,
        semantics.history,
        provenance_class,
        provenance_locator,
        semantics.ml,
    )
}

fn complete_card_errors(markdown: &str, spec: &CftSpec) -> Vec<String> {
    let mut errors = Vec::new();
    let Some(section) = card(markdown, spec.item_id, spec.chapter) else {
        return vec![format!("{} is missing its unique card", spec.item_id)];
    };
    let mut cursor = 0;
    for field in CARD_FIELDS {
        let marker = format!("\n#### {field}\n");
        if section.matches(&marker).count() != 1 {
            errors.push(format!(
                "{} must contain `{field}` exactly once",
                spec.item_id
            ));
            continue;
        }
        let position = section.find(&marker).expect("unique card field");
        if position < cursor {
            errors.push(format!("{} has `{field}` out of order", spec.item_id));
        }
        cursor = position;
        if card_field(section, field).is_none_or(str::is_empty) {
            errors.push(format!("{} has an empty `{field}`", spec.item_id));
        }
    }
    for (field, steps) in [
        ("Hypothesis ledger", spec.hypotheses),
        ("Statement", spec.statement_steps),
        ("Proof", spec.proof_steps),
    ] {
        let body = normalized_visible_markdown(card_field(section, field).unwrap_or_default());
        if !steps_in_order(&body, steps) {
            errors.push(format!(
                "{} `{field}` omits or misorders scoped obligations",
                spec.item_id
            ));
        }
    }
    if !card_field(section, "Purpose")
        .unwrap_or_default()
        .starts_with("Motivation.")
    {
        errors.push(format!(
            "{} Purpose must begin with `Motivation.`",
            spec.item_id
        ));
    }
    let ml = normalized_visible_markdown(card_field(section, "ML analogy").unwrap_or_default());
    if !steps_in_order(
        &ml,
        &[
            "Mathematical object / ML counterpart.",
            "Exact transfer.",
            "Non-transfer.",
            "Diagnostic.",
        ],
    ) {
        errors.push(format!(
            "{} has an incomplete ML transfer interface",
            spec.item_id
        ));
    }
    let lean = card_field(section, "Lean correspondence").unwrap_or_default();
    for exact in [
        spec.completed_kind,
        spec.completed_mode,
        completed_public(spec),
        spec.provider_declaration,
        spec.public_file,
        spec.provider_file,
    ] {
        if !lean.contains(exact) {
            errors.push(format!("{} Lean field omits `{exact}`", spec.item_id));
        }
    }
    if spec.item_id == "CFT-35-006" {
        for terminal in [JIN_TERMINAL, LS_TERMINAL, HARP_TERMINAL] {
            if !lean.contains(terminal) {
                errors.push(format!("{} Lean field omits `{terminal}`", spec.item_id));
            }
        }
    }
    let semantics = card_semantics(spec.item_id);
    for (field, token) in [
        ("Purpose", semantics.purpose),
        ("Boundary case", semantics.boundary),
        ("Historical context", semantics.history),
        ("ML analogy", semantics.ml),
        ("Proof", semantics.worked),
    ] {
        let visible = normalized_visible_markdown(card_field(section, field).unwrap_or_default());
        if !visible.contains(token) {
            errors.push(format!(
                "{} `{field}` omits visible row-specific semantic `{token}`",
                spec.item_id
            ));
        }
    }
    let history =
        normalized_visible_markdown(card_field(section, "Historical context").unwrap_or_default());
    let (provenance_class, provenance_locator) = row_provenance(spec.item_id);
    for token in [provenance_class, provenance_locator] {
        if !history.contains(token) {
            errors.push(format!(
                "{} Historical context omits row-specific provenance `{token}`",
                spec.item_id
            ));
        }
    }
    if spec.item_id == "CFT-35-006" {
        let proof = normalized_visible_markdown(card_field(section, "Proof").unwrap_or_default());
        for axis in COMPARISON_AXES {
            if !proof.contains(axis) {
                errors.push(format!("{} comparison omits `{axis}`", spec.item_id));
            }
        }
    }
    errors
}

fn steps_in_order(body: &str, steps: &[&str]) -> bool {
    let mut cursor = 0;
    for step in steps {
        let Some(offset) = body[cursor..].find(step) else {
            return false;
        };
        cursor += offset + step.len();
    }
    true
}

const WAVE3_PROBE_DECLARATIONS: &str = r#"
namespace CrouzeixTextbook.Wave3ContractProbe

open CrouzeixConjecture MeasureTheory Set
open CrouzeixConjecture.LoristSchwenninger
open scoped BoundedContinuousFunction InnerProductSpace Matrix Matrix.Norms.L2Operator

axiom cft_34_e01 {i n : Type*} [MeasurableSpace i]
    [Fintype n] [DecidableEq n] {mu : Measure i}
    (D : PositiveBoundaryDensity (n := n) mu) (x : EuclideanVector n) :
    ‖boundaryEmbeddingToLp D x‖ = ‖x‖

axiom cft_34_e02 {i n : Type*} [MeasurableSpace i] [TopologicalSpace i]
    [BorelSpace i] [SecondCountableTopologyEither i ℂ]
    [Fintype n] [DecidableEq n] {mu : Measure i}
    (D : PositiveBoundaryDensity (n := n) mu) (h : i →ᵇ ℂ)
    (x : EuclideanVector n) :
    ((ContinuousLinearMap.adjoint (boundaryEmbedding D).toContinuousLinearMap).comp
        ((bcfMulL (mu := mu) (n := n) h).comp
          (boundaryEmbedding D).toContinuousLinearMap)) x =
      euclideanOperator (boundaryPhiCLM D h) x

axiom cft_34_e03 :
    ((∮ z in C((0 : ℂ), 1), z⁻¹) = 2 * Real.pi * Complex.I) ∧
      (∀ n : ℤ, n ≠ -1 → (∮ z in C((0 : ℂ), 1), z ^ n) = 0) ∧
      ∀ (N : ℕ) (a : ℕ → ℂ),
        (∮ z in C((0 : ℂ), 1),
            ∑ m ∈ Finset.range (N + 1), a m * z ^ ((m : ℤ) - 1)) =
          a 0 * (2 * Real.pi * Complex.I)

axiom cft_34_e04 {i n : Type*} [MeasurableSpace i] [TopologicalSpace i]
    [BorelSpace i] [SecondCountableTopologyEither i ℂ]
    [Fintype n] [DecidableEq n] {mu : Measure i}
    (D : PositiveBoundaryDensity (n := n) mu) (h : i →ᵇ ℂ) (k : ℕ)
    (x : EuclideanVector n) :
    ((ContinuousLinearMap.adjoint (boundaryEmbedding D).toContinuousLinearMap).comp
        (((bcfMulL (mu := mu) (n := n) h) ^ k).comp
          (boundaryEmbedding D).toContinuousLinearMap)) x =
      euclideanOperator (boundaryPhiCLM D (h ^ k)) x

axiom cft_34_e05 {i n : Type*} [MeasurableSpace i] [TopologicalSpace i]
    [BorelSpace i] [SecondCountableTopologyEither i ℂ]
    [Fintype n] {mu : Measure i} (h : i →ᵇ ℂ) (hh : ‖h‖ ≤ 1)
    (f : i →₂[mu] EuclideanVector n) :
    ‖bcfMulL (mu := mu) (n := n) h f‖ ≤ ‖f‖

axiom cft_34_e06 {i n : Type*} [TopologicalSpace i] [CompactSpace i]
    [MeasurableSpace i] [BorelSpace i] [OpensMeasurableSpace i]
    [SecondCountableTopologyEither i ℂ]
    [Fintype n] [DecidableEq n] [Nonempty n]
    {mu : Measure i} [IsFiniteMeasure mu] {Omega : Set ℂ}
    (Gamma : ParametricConvexBoundary (i := i) Omega)
    (B : SquareMatrix n) (hWB : numericalRange B ⊆ Omega)
    (q : Polynomial ℂ)
    (hq : ∀ z ∈ closure Omega, ‖Polynomial.eval z q‖ ≤ 1)
    (hCauchy : HasParametricPolynomialCauchyFormula Gamma mu B) :
    (dilationDataOfParametricPolynomial Gamma B hWB q hq hCauchy).T =
        euclideanOperator (polynomialEval q B) ∧
      ‖(dilationDataOfParametricPolynomial Gamma B hWB q hq hCauchy).T‖ ≤ 2

axiom cft_35_e01 {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (hmain : MainTheoremStatement (n := n)) (A : SquareMatrix n)
    (p : Polynomial ℂ) : PolynomialCrouzeixBound A p

axiom cft_35_e02
    (hmain : ∀ (d : ℕ) [Nonempty (Fin d)], MainTheoremStatement (n := Fin d)) :
    FiniteMatrixMainTheoremStatement

axiom cft_35_e03 {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (hmain : MainTheoremStatement (n := n)) :
    RationalSpectralSetCorollaryStatement (n := n)

axiom cft_35_e04 {H : Type*} [NormedAddCommGroup H] [InnerProductSpace ℂ H]
    [CompleteSpace H] [Nontrivial H]
    (hfinite : FiniteMatrixMainTheoremStatement)
    (A : H →L[ℂ] H) (p : Polynomial ℂ) :
    ‖operatorPolynomialEval p A‖ ≤
      2 * supPolynomialModulusOnOperatorNumericalRange A p

axiom cft_35_e05 {H : Type*} [NormedAddCommGroup H] [InnerProductSpace ℂ H]
    [CompleteSpace H] [Nontrivial H]
    (hfinite : FiniteMatrixMainTheoremStatement) (A : H →L[ℂ] H) :
    ClosedOperatorNumericalRangeIsTwoSpectralSet A

theorem cft_35_e06 {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) (p : Polynomial ℂ)
    (jinConclusion lsConclusion harpConclusion : Prop)
    (hJinNormalized : jinConclusion = PolynomialCrouzeixBound A p)
    (hLsNormalized : lsConclusion = PolynomialCrouzeixBound A p)
    (hHarpNormalized : harpConclusion = PolynomialCrouzeixBound A p)
    (hJin : jinConclusion) (hLs : lsConclusion) (hHarp : harpConclusion) :
    PolynomialCrouzeixBound A p ∧ PolynomialCrouzeixBound A p ∧
      PolynomialCrouzeixBound A p := by
  exact ⟨hJinNormalized ▸ hJin, hLsNormalized ▸ hLs, hHarpNormalized ▸ hHarp⟩

axiom cft_35_006_bundle {n : Type} [Fintype n] [DecidableEq n] [Nonempty n] :
    MainTheoremStatement (n := n) ∧ MainTheoremStatement (n := n) ∧
      MainTheoremStatement (n := n)

axiom mutation_cft_35_e06_terminal {n : Type*}
    [Fintype n] [DecidableEq n] [Nonempty n] : MainTheoremStatement (n := n)

axiom mutation_cft_35_e06_reflexive {n : Type*}
    [Fintype n] [DecidableEq n] [Nonempty n]
    (jinRoute lsRoute harpRoute : MainTheoremStatement (n := n)) :
    jinRoute = jinRoute ∧ lsRoute = lsRoute ∧ harpRoute = harpRoute

axiom mutation_cft_35_e06_trivial {n : Type*}
    [Fintype n] [DecidableEq n] [Nonempty n]
    (jinRoute lsRoute harpRoute : MainTheoremStatement (n := n)) : True

end CrouzeixTextbook.Wave3ContractProbe
"#;

const BUNDLE_PROBE: &str = "CrouzeixTextbook.Wave3ContractProbe.cft_35_006_bundle";
const TERMINAL_EXERCISE_MUTATION: &str =
    "CrouzeixTextbook.Wave3ContractProbe.mutation_cft_35_e06_terminal";
const REFLEXIVE_EXERCISE_MUTATION: &str =
    "CrouzeixTextbook.Wave3ContractProbe.mutation_cft_35_e06_reflexive";
const TRIVIAL_EXERCISE_MUTATION: &str =
    "CrouzeixTextbook.Wave3ContractProbe.mutation_cft_35_e06_trivial";

fn probe_names() -> Vec<&'static str> {
    let mut names = CFT_SPECS
        .iter()
        .flat_map(|spec| [spec.public_declaration, spec.provider_declaration])
        .chain(EXERCISE_SPECS.iter().map(|spec| spec.probe_declaration))
        .chain([
            BUNDLE_PROBE,
            TERMINAL_EXERCISE_MUTATION,
            REFLEXIVE_EXERCISE_MUTATION,
            TRIVIAL_EXERCISE_MUTATION,
        ])
        .collect::<Vec<_>>();
    names.sort_unstable();
    names.dedup();
    names
}

fn compile_wave3_probe(declarations: &str) -> Result<Value, String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    let source_path = temp.path().join("Wave3ContractProbe.lean");
    let names = probe_names()
        .into_iter()
        .map(|name| format!("`{name}"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        r#"import CrouzeixTextbook.ExportReceipt
import CrouzeixHarp
{declarations}
open Lean
run_cmd do
  let env ← getEnv
  let names : Array Name := #[{names}]
  let rows ← Lean.Elab.Command.liftCoreM <|
    CrouzeixTextbook.ExportReceipt.receiptRows env names
  IO.println s!"WAVE3_CONTRACT_PROBE:{{(Json.arr rows).compress}}"
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
            "Wave 3 contract probe failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("WAVE3_CONTRACT_PROBE:"))
        .ok_or_else(|| "Wave 3 contract probe omitted its JSON marker".to_owned())?;
    serde_json::from_str(payload).map_err(|error| error.to_string())
}

fn compile_wave3_lean_fixture(source: &str) -> Output {
    let temp = tempfile::tempdir().expect("Wave 3 Lean fixture directory");
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .expect("private Wave 3 Lean fixture directory");
    let source_path = temp.path().join("Wave3NegativeFixture.lean");
    fs::write(&source_path, source).expect("write Wave 3 Lean fixture");
    super::textbook_lean_command("lake")
        .arg("env")
        .arg("lean")
        .arg(&source_path)
        .current_dir(workspace_root().join("formalization/lean"))
        .output()
        .expect("compile Wave 3 Lean fixture")
}

fn probe_receipt() -> &'static Value {
    static RECEIPT: OnceLock<Value> = OnceLock::new();
    RECEIPT.get_or_init(|| {
        compile_wave3_probe(WAVE3_PROBE_DECLARATIONS)
            .unwrap_or_else(|error| panic!("Wave 3 signatures must elaborate: {error}"))
    })
}

fn probe_row(name: &str) -> &'static Value {
    probe_receipt()
        .as_array()
        .expect("Wave 3 probe rows")
        .iter()
        .find(|row| row["name"].as_str() == Some(name))
        .unwrap_or_else(|| panic!("Wave 3 probe omitted {name}"))
}

fn compile_active_dependency_receipt(phase: Wave3Phase) -> Result<Value, String> {
    let temp = tempfile::tempdir().map_err(|error| error.to_string())?;
    fs::set_permissions(temp.path(), fs::Permissions::from_mode(0o700))
        .map_err(|error| error.to_string())?;
    let source_path = temp.path().join("Wave3ActiveDependencyProbe.lean");
    let roots = EXERCISE_SPECS
        .iter()
        .filter(|spec| phase.rank() >= spec.completion_rank)
        .map(|spec| spec.solution)
        .chain((phase == Wave3Phase::ThreeRouteComparisonComplete).then_some(THREE_ROUTE_BUNDLE))
        .map(|name| format!("`{name}"))
        .collect::<Vec<_>>()
        .join(", ");
    let source = format!(
        r#"import CrouzeixTextbook.ExportReceipt
import CrouzeixHarp
open Lean Meta
namespace CrouzeixTextbook.Wave3ActiveDependencyProbe
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
/-- This audit starts at a theorem's proof body, not its type. It follows
transitive proof-body references through maintained declarations, but treats
the exact external allowlist as terminal leaves. Thus comments and type-only
mentions are invisible, while a maintained helper cannot hide an external
proof dependency. -/
def externalProofAllowlist : NameHashSet :=
  #[`circleIntegral.integral_sub_inv_of_mem_ball,
    `circleIntegral.integral_sub_zpow_of_ne].foldl
    (init := ({{}} : NameHashSet)) fun selected name => selected.insert name
partial def externalProofClosure (env : Environment) : List Name → NameHashSet → NameHashSet
  | [], seen => seen
  | name :: pending, seen =>
      if seen.contains name then externalProofClosure env pending seen
      else
        let seen := seen.insert name
        if externalProofAllowlist.contains name ||
            !CrouzeixTextbook.ExportReceipt.isMaintainedName name then
          externalProofClosure env pending seen
        else
          match env.find? name with
          | none => externalProofClosure env pending seen
          | some info =>
              let dependencies := info.value? (allowOpaque := true)
                |>.map CrouzeixTextbook.ExportReceipt.collectConstants |>.getD {{}}
              externalProofClosure env (dependencies.toList ++ pending) seen
def externalProofDependencies (env : Environment) (name : Name) : Array Name :=
  match env.find? name with
  | none => #[]
  | some info =>
      let roots := info.value? (allowOpaque := true)
        |>.map CrouzeixTextbook.ExportReceipt.collectConstants |>.getD {{}}
      (externalProofClosure env roots.toList {{}}).toArray
        |>.filter externalProofAllowlist.contains
        |>.qsort (fun left right => left.toString < right.toString)
def isTrackedExerciseDeclaration (env : Environment) (name : Name) : Bool :=
  match CrouzeixTextbook.ExportReceipt.sourceLocation env name with
  | none => false
  | some (sourcePath, _, _) =>
      let rendered := name.toString
      (sourcePath.endsWith "Part06/Chapter34.lean" &&
          rendered.contains "Exercises.Chapter34.") ||
        (sourcePath.endsWith "Part06/Chapter35.lean" &&
          rendered.contains "Exercises.Chapter35.")
run_cmd do
  let env ← getEnv
  let roots : Array Name := #[{roots}]
  let maintainedClosure := (dependencyClosure env roots.toList {{}}).toArray
    |>.filter CrouzeixTextbook.ExportReceipt.isMaintainedName
    |>.foldl (init := ({{}} : NameHashSet)) fun selected name => selected.insert name
  let declarations := env.constants.toList.foldl (init := maintainedClosure) fun selected (name, _) =>
    if isTrackedExerciseDeclaration env name then selected.insert name else selected
  let names := (declarations.toArray.filter fun name =>
      (CrouzeixTextbook.ExportReceipt.sourceLocation env name).isSome)
    |>.qsort (fun left right => left.toString < right.toString)
  let rows ← Lean.Elab.Command.liftCoreM <|
    CrouzeixTextbook.ExportReceipt.receiptRows env names
  IO.println s!"WAVE3_ACTIVE_DEPENDENCY_RECEIPT:{{(Json.arr rows).compress}}"
  let externalRows := roots.map fun name =>
    let dependencies := (externalProofDependencies env name).map (Json.str ·.toString)
    json% {{
      name: $(name.toString),
      audited_external_proof_dependencies: $(dependencies)
    }}
  IO.println s!"WAVE3_EXTERNAL_PROOF_DEPENDENCY_RECEIPT:{{(Json.arr externalRows).compress}}"
end CrouzeixTextbook.Wave3ActiveDependencyProbe
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
            "Wave 3 active dependency probe failed:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let stdout = String::from_utf8(output.stdout).map_err(|error| error.to_string())?;
    let payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("WAVE3_ACTIVE_DEPENDENCY_RECEIPT:"))
        .ok_or_else(|| "Wave 3 active dependency probe omitted its JSON marker".to_owned())?;
    let external_payload = stdout
        .lines()
        .find_map(|line| line.strip_prefix("WAVE3_EXTERNAL_PROOF_DEPENDENCY_RECEIPT:"))
        .ok_or_else(|| {
            "Wave 3 active dependency probe omitted its external-proof JSON marker".to_owned()
        })?;
    let mut receipt: Value = serde_json::from_str(payload).map_err(|error| error.to_string())?;
    let external: Value =
        serde_json::from_str(external_payload).map_err(|error| error.to_string())?;
    let external_by_name = external
        .as_array()
        .ok_or_else(|| "external-proof receipt must be an array".to_owned())?
        .iter()
        .filter_map(|row| Some((row["name"].as_str()?, row)))
        .collect::<BTreeMap<_, _>>();
    for row in receipt
        .as_array_mut()
        .ok_or_else(|| "active dependency receipt must be an array".to_owned())?
    {
        let name = row["name"]
            .as_str()
            .ok_or_else(|| "active dependency row omitted its name".to_owned())?;
        row["audited_external_proof_dependencies"] = external_by_name
            .get(name)
            .map(|external_row| external_row["audited_external_proof_dependencies"].clone())
            .unwrap_or_else(|| json!([]));
    }
    Ok(receipt)
}

fn active_dependency_receipt() -> &'static Value {
    static RECEIPT: OnceLock<Value> = OnceLock::new();
    RECEIPT.get_or_init(|| {
        let phase = active_phase(&wave3_contract());
        compile_active_dependency_receipt(phase)
            .unwrap_or_else(|error| panic!("active Wave 3 receipt must compile: {error}"))
    })
}

fn provider_hash(spec: &CftSpec) -> &'static str {
    probe_row(spec.provider_declaration)["type_sha256"]
        .as_str()
        .expect("provider type hash")
}

fn completed_type_hash(spec: &CftSpec) -> &'static str {
    if spec.item_id == "CFT-35-006" {
        "5ce10ccd8b2e5f6a0a93932256d6df60556b450817f4324eb4bac53cd444962b"
    } else {
        spec.baseline_type_sha256
    }
}

fn exercise_metadata_hash(spec: &ExerciseSpec, phase: Wave3Phase) -> &'static str {
    let live_phase = active_phase(&wave3_contract());
    if phase.rank() <= live_phase.rank() {
        spec.canonical_type_sha256
            .expect("completed live exercise must record its canonical receipt hash")
    } else {
        spec.canonical_type_sha256.unwrap_or_else(|| {
            probe_row(spec.probe_declaration)["type_sha256"]
                .as_str()
                .expect("future exercise target hash")
        })
    }
}

fn public_receipt_fixture(phase: Wave3Phase) -> Value {
    let mut declarations = CFT_SPECS
        .iter()
        .map(|spec| {
            let complete = phase.rank() >= spec.completion_rank;
            let target = if complete && spec.item_id == "CFT-35-006" {
                probe_row(BUNDLE_PROBE)
            } else {
                probe_row(spec.public_declaration)
            };
            let formal_mode = if complete {
                spec.completed_mode
            } else {
                spec.baseline_mode
            };
            let provider_dependency = if complete {
                (formal_mode != "proved-here").then_some(spec.provider_declaration)
            } else {
                (spec.baseline_correspondence == "unmapped").then_some(spec.provider_declaration)
            };
            let direct_dependencies = if complete {
                if spec.item_id == "CFT-35-006" {
                    json!([JIN_TERMINAL, LS_TERMINAL, HARP_TERMINAL])
                } else {
                    json!([spec.provider_declaration])
                }
            } else {
                target["direct_dependencies"].clone()
            };
            json!({
                "name": if complete { completed_public(spec) } else { spec.public_declaration },
                "source_path": if complete && spec.item_id == "CFT-35-006" {
                    json!(spec.public_file)
                } else {
                    target["source_path"].clone()
                },
                "kind": if complete && spec.item_id == "CFT-35-006" {
                    json!("theorem")
                } else {
                    target["kind"].clone()
                },
                "normalized_type": target["normalized_type"],
                "type_sha256": if complete && spec.item_id == "CFT-35-006" {
                    json!(completed_type_hash(spec))
                } else {
                    target["type_sha256"].clone()
                },
                "axioms": if complete && spec.item_id == "CFT-35-006" {
                    json!(AXIOMS)
                } else {
                    target["axioms"].clone()
                },
                "formal_mode": formal_mode,
                "provider_dependency": provider_dependency,
                "direct_dependencies": direct_dependencies,
            })
        })
        .collect::<Vec<_>>();
    if phase == Wave3Phase::ThreeRouteComparisonComplete {
        let compatibility = cft_spec("CFT-35-006");
        let compiler = probe_row(compatibility.public_declaration);
        declarations.push(json!({
            "name": compatibility.public_declaration,
            "source_path": compiler["source_path"],
            "kind": compiler["kind"],
            "normalized_type": compiler["normalized_type"],
            "type_sha256": compiler["type_sha256"],
            "axioms": compiler["axioms"],
            "formal_mode": "compatibility",
            "provider_dependency": compatibility.provider_declaration,
            "direct_dependencies": compiler["direct_dependencies"],
        }));
    }
    json!({"declarations": declarations})
}

fn public_receipt_errors(receipt: &Value, phase: Wave3Phase) -> Vec<String> {
    let rows = receipt["declarations"]
        .as_array()
        .expect("public receipt rows");
    let mut errors = Vec::new();
    for spec in CFT_SPECS {
        let complete = phase.rank() >= spec.completion_rank;
        let expected_name = if complete {
            completed_public(&spec)
        } else {
            spec.public_declaration
        };
        let matches = rows
            .iter()
            .filter(|row| row["name"].as_str() == Some(expected_name))
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            errors.push(format!(
                "{} must have exactly one compiler public row `{expected_name}`",
                spec.item_id
            ));
            continue;
        }
        let row = matches[0];
        let compiler = if complete && spec.item_id == "CFT-35-006" {
            probe_row(BUNDLE_PROBE)
        } else {
            probe_row(spec.public_declaration)
        };
        let expected_kind = if complete && spec.item_id == "CFT-35-006" {
            "theorem"
        } else {
            "direct-alias"
        };
        let expected_mode = if complete {
            spec.completed_mode
        } else {
            spec.baseline_mode
        };
        let expected_provider = if complete {
            (expected_mode != "proved-here").then_some(spec.provider_declaration)
        } else {
            (spec.baseline_correspondence == "unmapped").then_some(spec.provider_declaration)
        };
        let expected_hash = if complete {
            if spec.item_id == "CFT-35-006" {
                completed_type_hash(&spec)
            } else {
                spec.compiler_public_type_sha256
            }
        } else {
            spec.compiler_public_type_sha256
        };
        for (field, matches_expected) in [
            (
                "source_path",
                row["source_path"].as_str() == Some(spec.public_file),
            ),
            (
                "normalized_type",
                row["normalized_type"] == compiler["normalized_type"],
            ),
            (
                "type_sha256",
                row["type_sha256"].as_str() == Some(expected_hash),
            ),
            ("kind", row["kind"].as_str() == Some(expected_kind)),
            ("axioms", row["axioms"] == json!(AXIOMS)),
            (
                "formal_mode",
                row["formal_mode"].as_str() == Some(expected_mode),
            ),
            (
                "provider_dependency",
                row["provider_dependency"].as_str() == expected_provider,
            ),
        ] {
            if !matches_expected {
                errors.push(format!(
                    "{expected_name} {field} differs from compiler contract: observed {}",
                    row[field]
                ));
            }
        }
        let dependencies = row["direct_dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        if complete && spec.item_id == "CFT-35-006" {
            if dependencies != BTreeSet::from([JIN_TERMINAL, LS_TERMINAL, HARP_TERMINAL]) {
                errors.push(format!(
                    "{expected_name} direct_dependencies differ from the three-provider contract"
                ));
            }
        } else {
            let compiler_dependencies = compiler["direct_dependencies"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .collect::<BTreeSet<_>>();
            if dependencies != compiler_dependencies {
                errors.push(format!(
                    "{expected_name} direct_dependencies differ from compiler evidence"
                ));
            }
            if let Some(provider) = expected_provider {
                if !dependencies.contains(provider) {
                    errors.push(format!(
                        "{expected_name} direct_dependencies omit active provider `{provider}`"
                    ));
                }
            }
        }
    }

    for stable in CFT_SPECS.iter().filter(|spec| spec.chapter == 35) {
        let matches = rows
            .iter()
            .filter(|row| row["name"].as_str() == Some(stable.public_declaration))
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            errors.push(format!(
                "stable compatibility name `{}` must have exactly one compiler row",
                stable.public_declaration
            ));
            continue;
        }
        let expected_kind = "direct-alias";
        let row = matches[0];
        let compiler = probe_row(stable.public_declaration);
        let complete = phase.rank() >= stable.completion_rank;
        let expected_mode = if complete && stable.item_id == "CFT-35-006" {
            "compatibility"
        } else if complete {
            stable.completed_mode
        } else {
            stable.baseline_mode
        };
        let expected_provider = if complete && stable.item_id == "CFT-35-006" {
            Some(stable.provider_declaration)
        } else if complete {
            (stable.completed_mode != "proved-here").then_some(stable.provider_declaration)
        } else {
            (stable.baseline_correspondence == "unmapped").then_some(stable.provider_declaration)
        };
        for (field, matches_expected) in [
            (
                "source_path",
                row["source_path"].as_str() == Some(stable.public_file),
            ),
            (
                "normalized_type",
                row["normalized_type"] == compiler["normalized_type"],
            ),
            (
                "type_sha256",
                row["type_sha256"].as_str() == Some(stable.compiler_public_type_sha256),
            ),
            ("kind", row["kind"].as_str() == Some(expected_kind)),
            ("axioms", row["axioms"] == json!(AXIOMS)),
            (
                "formal_mode",
                row["formal_mode"].as_str() == Some(expected_mode),
            ),
            (
                "provider_dependency",
                row["provider_dependency"].as_str() == expected_provider,
            ),
            (
                "direct_dependencies",
                row["direct_dependencies"] == compiler["direct_dependencies"],
            ),
        ] {
            if !matches_expected {
                errors.push(format!(
                    "stable compatibility name `{}` {field} differs from compiler evidence",
                    stable.public_declaration
                ));
            }
        }
    }
    errors
}

fn baseline_fixture() -> (Value, Value) {
    let items = CFT_SPECS
        .iter()
        .map(|spec| {
            let underlying = if spec.baseline_correspondence == "unmapped" {
                json!(spec.provider_declaration)
            } else {
                Value::Null
            };
            json!({
                "item_id": spec.item_id,
                "chapter": spec.chapter,
                "kind": spec.baseline_kind,
                "formal_mode": spec.baseline_mode,
                "lean_correspondence_status": spec.baseline_correspondence,
                "prose_proof_status": "summary",
                "anchor": spec.item_id.to_ascii_lowercase(),
                "lean_declaration": {
                    "name": spec.public_declaration,
                    "underlying_declaration": underlying,
                    "source_path": spec.public_file,
                    "verification_target": "CrouzeixTextbook",
                    "type_sha256": spec.baseline_type_sha256,
                    "assumptions": [],
                    "axioms": AXIOMS,
                }
            })
        })
        .collect::<Vec<_>>();
    let exercises = EXERCISE_SPECS
        .iter()
        .map(|spec| {
            json!({
                "exercise_id": spec.exercise_id,
                "chapter": spec.chapter,
                "skills": [spec.parent],
                "starter": null,
                "lean_solution": null,
            })
        })
        .collect::<Vec<_>>();
    (json!({"items": items}), json!({"exercises": exercises}))
}

fn promote_fixture(coverage: &mut Value, exercises: &mut Value, phase: Wave3Phase) {
    for spec in CFT_SPECS {
        if phase.rank() < spec.completion_rank {
            continue;
        }
        let row = coverage["items"]
            .as_array_mut()
            .expect("fixture coverage rows")
            .iter_mut()
            .find(|row| row["item_id"] == spec.item_id)
            .expect("fixture CFT row");
        row["kind"] = json!(spec.completed_kind);
        row["formal_mode"] = json!(spec.completed_mode);
        row["lean_correspondence_status"] = json!("exact");
        row["prose_proof_status"] = json!("reconstructible");
        row["lean_declaration"]["name"] = json!(completed_public(&spec));
        row["lean_declaration"]["underlying_declaration"] = if spec.completed_mode == "proved-here"
        {
            Value::Null
        } else {
            json!(spec.provider_declaration)
        };
        row["lean_declaration"]["type_sha256"] = json!(completed_type_hash(&spec));
    }
    for spec in EXERCISE_SPECS {
        if phase.rank() < spec.completion_rank {
            continue;
        }
        let row = exercises["exercises"]
            .as_array_mut()
            .expect("fixture exercise rows")
            .iter_mut()
            .find(|row| row["exercise_id"] == spec.exercise_id)
            .expect("fixture exercise row");
        row["lean_solution"] = json!({
            "declaration": spec.solution,
            "source_path": format!(
                "formalization/lean/CrouzeixTextbook/Part06/Chapter{}.lean",
                spec.chapter
            ),
            "type_sha256": exercise_metadata_hash(&spec, phase),
            "verification_target": "CrouzeixTextbook",
            "axioms": AXIOMS,
        });
    }
}

fn phase_metadata_errors(coverage: &Value, exercises: &Value, phase: Wave3Phase) -> Vec<String> {
    let mut errors = Vec::new();
    let rows = coverage["items"].as_array().expect("coverage fixture rows");
    let exercise_rows = exercises["exercises"]
        .as_array()
        .expect("exercise fixture rows");
    for spec in CFT_SPECS {
        let matching = rows
            .iter()
            .filter(|row| row["item_id"].as_str() == Some(spec.item_id))
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            errors.push(format!(
                "{} must have exactly one row, observed {}",
                spec.item_id,
                matching.len()
            ));
            continue;
        }
        let complete = phase.rank() >= spec.completion_rank;
        let row = matching[0];
        for (field, expected) in [
            (
                "kind",
                if complete {
                    spec.completed_kind
                } else {
                    spec.baseline_kind
                },
            ),
            (
                "formal_mode",
                if complete {
                    spec.completed_mode
                } else {
                    spec.baseline_mode
                },
            ),
            (
                "lean_correspondence_status",
                if complete {
                    "exact"
                } else {
                    spec.baseline_correspondence
                },
            ),
            (
                "prose_proof_status",
                if complete {
                    "reconstructible"
                } else {
                    "summary"
                },
            ),
        ] {
            if row[field].as_str() != Some(expected) {
                errors.push(format!(
                    "{} {field} must be `{expected}` in phase `{}`",
                    spec.item_id,
                    phase.label()
                ));
            }
        }
        let declaration = &row["lean_declaration"];
        let expected_name = if complete {
            completed_public(&spec)
        } else {
            spec.public_declaration
        };
        let expected_underlying = if complete {
            if spec.completed_mode == "proved-here" {
                None
            } else {
                Some(spec.provider_declaration)
            }
        } else if spec.baseline_correspondence == "unmapped" {
            Some(spec.provider_declaration)
        } else {
            None
        };
        let expected_hash = if complete {
            completed_type_hash(&spec)
        } else {
            spec.baseline_type_sha256
        };
        if declaration["name"].as_str() != Some(expected_name)
            || declaration["underlying_declaration"].as_str() != expected_underlying
            || declaration["source_path"].as_str() != Some(spec.public_file)
            || declaration["verification_target"].as_str() != Some("CrouzeixTextbook")
            || declaration["type_sha256"].as_str() != Some(expected_hash)
            || declaration["assumptions"] != json!([])
            || declaration["axioms"] != json!(AXIOMS)
        {
            errors.push(format!(
                "{} declaration metadata differs from phase `{}`",
                spec.item_id,
                phase.label()
            ));
        }
    }
    for spec in EXERCISE_SPECS {
        let matching = exercise_rows
            .iter()
            .filter(|row| row["exercise_id"].as_str() == Some(spec.exercise_id))
            .collect::<Vec<_>>();
        if matching.len() != 1 {
            errors.push(format!(
                "{} must have exactly one row, observed {}",
                spec.exercise_id,
                matching.len()
            ));
            continue;
        }
        let row = matching[0];
        if row["skills"] != json!([spec.parent]) || !row["starter"].is_null() {
            errors.push(format!(
                "{} has the wrong parent or starter",
                spec.exercise_id
            ));
        }
        if phase.rank() >= spec.completion_rank {
            let solution = &row["lean_solution"];
            let expected_hash = exercise_metadata_hash(&spec, phase);
            if solution["declaration"].as_str() != Some(spec.solution)
                || solution["source_path"].as_str()
                    != Some(
                        format!(
                            "formalization/lean/CrouzeixTextbook/Part06/Chapter{}.lean",
                            spec.chapter
                        )
                        .as_str(),
                    )
                || solution["type_sha256"].as_str() != Some(expected_hash)
                || solution["verification_target"].as_str() != Some("CrouzeixTextbook")
            {
                errors.push(format!(
                    "{} lacks its distinct exact solution in phase `{}`",
                    spec.exercise_id,
                    phase.label()
                ));
            }
            if solution["axioms"] != json!(AXIOMS) {
                errors.push(format!(
                    "{} solution must have exactly the standard axiom set",
                    spec.exercise_id
                ));
            }
        } else if !row["lean_solution"].is_null() {
            errors.push(format!(
                "{} must remain unsolved in phase `{}`",
                spec.exercise_id,
                phase.label()
            ));
        }
    }
    errors
}

fn card_presence_errors(item_id: &str, phase: Wave3Phase, observed: &[bool; 10]) -> Vec<String> {
    let spec = cft_spec(item_id);
    let expected = phase.rank() >= spec.completion_rank;
    CARD_FIELDS
        .iter()
        .zip(observed)
        .filter(|(_, present)| **present != expected)
        .map(|(field, _)| {
            format!(
                "{item_id} `{field}` presence must be {expected} in phase `{}`",
                phase.label()
            )
        })
        .collect()
}

#[test]
fn ls_route_contract_freezes_the_six_phase_model() {
    let contract = wave3_contract();
    let active = active_phase(&contract);
    assert!(Wave3Phase::ALL.contains(&active));
    for phase in Wave3Phase::ALL {
        assert!(
            contract.contains(phase.label()),
            "missing phase `{}`",
            phase.label()
        );
    }
    assert!(contract.contains("- Target phase: `three-route-comparison-complete`."));
}

#[test]
fn ls_route_contract_phase_fixtures_accept_every_phase_and_reject_fake_promotions() {
    for phase in Wave3Phase::ALL {
        let (mut coverage, mut exercises) = baseline_fixture();
        promote_fixture(&mut coverage, &mut exercises, phase);
        let errors = phase_metadata_errors(&coverage, &exercises, phase);
        assert!(
            errors.is_empty(),
            "valid {} fixture: {errors:#?}",
            phase.label()
        );
        for spec in CFT_SPECS {
            let observed = [phase.rank() >= spec.completion_rank; 10];
            assert!(card_presence_errors(spec.item_id, phase, &observed).is_empty());
        }
    }

    let (mut coverage, exercises) = baseline_fixture();
    let row = coverage["items"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["item_id"] == "CFT-34-003")
        .unwrap();
    row["prose_proof_status"] = json!("reconstructible");
    assert!(
        phase_metadata_errors(&coverage, &exercises, Wave3Phase::Chapter34BoundaryData)
            .iter()
            .any(|error| error.contains("CFT-34-003 prose_proof_status"))
    );

    let (coverage, mut exercises) = baseline_fixture();
    exercises["exercises"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["exercise_id"] == "CFT-34-E06")
        .unwrap()["lean_solution"] = json!({"declaration": EXERCISE_SPECS[5].solution});
    assert!(
        phase_metadata_errors(&coverage, &exercises, Wave3Phase::Chapter34BoundaryData)
            .iter()
            .any(|error| error.contains("CFT-34-E06 must remain unsolved"))
    );

    let (mut coverage, mut exercises) = baseline_fixture();
    promote_fixture(
        &mut coverage,
        &mut exercises,
        Wave3Phase::ThreeRouteComparisonComplete,
    );
    let bundle = coverage["items"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["item_id"] == "CFT-35-006")
        .unwrap();
    bundle["lean_declaration"]["name"] = json!("CrouzeixTextbook.Part06.jin_final_comparator");
    bundle["lean_declaration"]["underlying_declaration"] = json!(JIN_TERMINAL);
    assert!(phase_metadata_errors(
        &coverage,
        &exercises,
        Wave3Phase::ThreeRouteComparisonComplete,
    )
    .iter()
    .any(|error| error.contains("CFT-35-006 declaration metadata")));

    let mut missing_field = [true; 10];
    missing_field[7] = false;
    assert!(card_presence_errors(
        "CFT-35-006",
        Wave3Phase::ThreeRouteComparisonComplete,
        &missing_field,
    )
    .iter()
    .any(|error| error.contains("Lean correspondence")));

    for phase in Wave3Phase::ALL
        .into_iter()
        .filter(|phase| *phase != Wave3Phase::ThreeRouteComparisonComplete)
    {
        let premature_bundle = json!({"declarations": [{
            "name": THREE_ROUTE_BUNDLE,
            "direct_dependencies": [JIN_TERMINAL, LS_TERMINAL, HARP_TERMINAL],
        }]});
        assert!(
            bundle_dependency_errors(&premature_bundle, phase)
                .iter()
                .any(|error| error.contains("must be absent")),
            "{} accepted the final bundle before its phase",
            phase.label()
        );
    }

    let (mut coverage, mut exercises) = baseline_fixture();
    promote_fixture(
        &mut coverage,
        &mut exercises,
        Wave3Phase::ThreeRouteComparisonComplete,
    );
    exercises["exercises"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["exercise_id"] == "CFT-34-E01")
        .unwrap()["lean_solution"]["axioms"] = json!([
        "Classical.choice",
        "CrouzeixTextbook.ProjectAxiom",
        "Quot.sound",
        "propext"
    ]);
    assert!(phase_metadata_errors(
        &coverage,
        &exercises,
        Wave3Phase::ThreeRouteComparisonComplete,
    )
    .iter()
    .any(|error| error.contains("standard axiom set")));
}

#[test]
fn ls_route_contract_future_cards_are_full_and_reject_keyword_templates() {
    for spec in CFT_SPECS {
        let fixture = future_card_fixture(&spec);
        let errors = complete_card_errors(&fixture, &spec);
        assert!(
            errors.is_empty(),
            "valid {} future card: {errors:#?}",
            spec.item_id
        );

        let keyword_template = CARD_FIELDS
            .iter()
            .map(|field| format!("#### {field}\n\nplaceholder\n"))
            .collect::<Vec<_>>()
            .join("\n");
        let thin = format!(
            "### {} — keyword template {{#cft-{}}}\n\n{keyword_template}",
            spec.item_id,
            spec.item_id
                .strip_prefix("CFT-")
                .expect("CFT item")
                .to_ascii_lowercase(),
        );
        let errors = complete_card_errors(&thin, &spec);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("omits or misorders scoped obligations")),
            "{} accepted a keyword-only template: {errors:#?}",
            spec.item_id
        );
    }

    let spec = cft_spec("CFT-35-006");
    let missing_harp = future_card_fixture(spec).replace(HARP_TERMINAL, "missing-harp-provider");
    assert!(
        complete_card_errors(&missing_harp, spec)
            .iter()
            .any(|error| error.contains(HARP_TERMINAL)),
        "comparison card accepted a two-provider Lean field"
    );
}

#[test]
fn ls_route_contract_cards_require_row_specific_semantics_and_visible_worked_content() {
    let expected = [
        (
            "CFT-34-001",
            [
                "build the concrete dilation record",
                "zero polynomial q = 0",
                "Lorist--Schwenninger boundary realization",
                "lifted feature-space realization",
                "finite atomic boundary model",
            ],
        ),
        (
            "CFT-34-002",
            [
                "identify the compressed first boundary moment",
                "constant multiplier h = 0",
                "first compression moment",
                "feature covariance compression",
                "two-atom compression calculation",
            ],
        ),
        (
            "CFT-34-003",
            [
                "upgrade the first moment to every power",
                "power k = 0",
                "power-moment compression",
                "multi-step rollout moments",
                "empirical moment residual R_k",
            ],
        ),
        (
            "CFT-34-004",
            [
                "turn multiplier products into powers",
                "base case k = 0",
                "boundary multiplier algebra",
                "tied linear-layer composition",
                "worked k = 2 multiplication",
            ],
        ),
        (
            "CFT-34-005",
            [
                "certify contraction before dilation",
                "strict contraction ‖h‖ < 1 gives ‖M_h‖ < 1",
                "contractive boundary multiplier",
                "Lipschitz certificate",
                "worked bound ‖h‖ = ρ < 1",
            ],
        ),
        (
            "CFT-34-006",
            [
                "feed the realization into the Chapter 33 endpoint",
                "q = 0 gives operator norm zero",
                "factor-two realization endpoint",
                "robust operator-norm certificate",
                "nonnormal 2 × 2 worked matrix",
            ],
        ),
        (
            "CFT-35-001",
            [
                "remove the normalization and approximation scaffolding",
                "zero polynomial makes both sides zero",
                "main polynomial consequence",
                "nonnormal layer functional bound",
                "worked polynomial p(z) = z",
            ],
        ),
        (
            "CFT-35-002",
            [
                "specialize the type-parametric theorem to Fin d",
                "d = 0 is excluded by Nonempty (Fin d)",
                "finite-matrix consequence adapter",
                "width-indexed matrix theorem",
                "worked dimension d = 2",
            ],
        ),
        (
            "CFT-35-003",
            [
                "extend polynomial control to pole-free rational functions",
                "an embedded polynomial is a rational boundary case",
                "rational approximation consequence",
                "resolvent-filter certificate",
                "one-pole rational approximant sequence",
            ],
        ),
        (
            "CFT-35-004",
            [
                "transport finite matrices to bounded Hilbert-space operators",
                "finite-dimensional H recovers the matrix theorem",
                "Hilbert-space transport consequence",
                "infinite-width operator limit",
                "finite-rank compression calculation",
            ],
        ),
        (
            "CFT-35-005",
            [
                "package compactness, spectral containment, and rational control",
                "a scalar operator makes all three components explicit",
                "closed numerical-range spectral-set consequence",
                "three-part stability certificate",
                "worked compactness-spectrum-bound triple",
            ],
        ),
        (
            "CFT-35-006",
            [
                "compare three independently checked terminal routes",
                "propositional agreement is not proof-term identity",
                "three-route comparison, not a fourth proof",
                "proof-architecture ablation",
                "worked three-column comparison row",
            ],
        ),
    ];
    for (item_id, tokens) in expected {
        let spec = cft_spec(item_id);
        let fixture = future_card_fixture(spec);
        for token in tokens {
            assert!(
                fixture.contains(token),
                "{item_id} generic fixture omits row-specific semantic `{token}`"
            );
            let deleted = fixture.replacen(token, "", 1);
            assert!(
                !complete_card_errors(&deleted, spec).is_empty(),
                "{item_id} accepted deletion of semantic obligation `{token}`"
            );
        }
    }

    let comparison = future_card_fixture(cft_spec("CFT-35-006"));
    for axis in [
        "Objects.",
        "Hypotheses.",
        "Shared trunk.",
        "Decisive mechanism.",
        "Approximation order.",
        "Conclusion.",
        "Provenance.",
        "Formal provider.",
    ] {
        assert!(comparison.contains(axis), "comparison omits `{axis}`");
        let deleted = comparison.replacen(axis, "", 1);
        assert!(
            !complete_card_errors(&deleted, cft_spec("CFT-35-006")).is_empty(),
            "comparison accepted deletion of `{axis}`"
        );
    }

    let spec = cft_spec("CFT-34-003");
    let fixture = future_card_fixture(spec);
    let token = card_semantics(spec.item_id).worked;
    let comment_only = fixture.replacen(token, &format!("<!-- {token} -->"), 1);
    assert!(
        complete_card_errors(&comment_only, spec)
            .iter()
            .any(|error| error.contains("visible row-specific semantic")),
        "HTML-comment-only worked content was accepted"
    );
}

#[test]
fn ls_route_contract_active_state_matches_all_twelve_rows_anchors_and_exercises() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let phase = active_phase(&wave3_contract());
    let errors = phase_metadata_errors(&coverage, &exercises, phase);
    assert!(
        errors.is_empty(),
        "active {} metadata differs from the phase contract: {errors:#?}",
        phase.label()
    );
    let live_rows = array(&coverage, "items", "coverage")
        .iter()
        .filter(|row| matches!(row["chapter"].as_u64(), Some(34 | 35)))
        .collect::<Vec<_>>();
    assert_eq!(
        live_rows.len(),
        12,
        "Wave 3 must have exactly twelve CFT rows"
    );
    for spec in CFT_SPECS {
        let item_id = spec.item_id;
        let row = live_rows
            .iter()
            .find(|row| row["item_id"] == item_id)
            .unwrap_or_else(|| panic!("missing live row {item_id}"));
        assert_eq!(row["anchor"], spec.item_id.to_ascii_lowercase());
        let expected_sources = if item_id == "CFT-35-006" {
            json!([
                "MATHLIB-4.32.1",
                "CROUZEIX-PACKET",
                "LS-ARXIV-V1",
                "JIN-V4-AUDITED"
            ])
        } else {
            json!(["MATHLIB-4.32.1", "CROUZEIX-PACKET", "LS-ARXIV-V1"])
        };
        assert_eq!(row["source_ids"], expected_sources);
    }

    let live_exercises = array(&exercises, "exercises", "exercises")
        .iter()
        .filter(|row| matches!(row["chapter"].as_u64(), Some(34 | 35)))
        .collect::<Vec<_>>();
    assert_eq!(live_exercises.len(), 12);
    let expected_solution_count = EXERCISE_SPECS
        .iter()
        .filter(|spec| phase.rank() >= spec.completion_rank)
        .count();
    let solution_names = live_exercises
        .iter()
        .filter_map(|row| row["lean_solution"]["declaration"].as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(solution_names.len(), expected_solution_count);
    let compiled = json!({"declarations": active_dependency_receipt()});
    let errors = exercise_receipt_errors(&compiled, phase);
    assert!(
        errors.is_empty(),
        "active {} compiled exercise closure differs from the contract: {errors:#?}",
        phase.label()
    );
    let errors = bundle_dependency_errors(&compiled, phase);
    assert!(
        errors.is_empty(),
        "active {} bundle closure differs from the contract: {errors:#?}",
        phase.label()
    );
    let exporter = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/ExportReceipt.lean"),
    )
    .expect("textbook receipt exporter");
    let exported =
        super::jin_checked_exercise_names(&exporter).expect("checked exercise theorem-name array");
    let exported_wave3 = exported
        .iter()
        .filter(|name| {
            name.contains("Exercises.Chapter34.") || name.contains("Exercises.Chapter35.")
        })
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    assert_eq!(exported_wave3, solution_names);

    for chapter in [34_u64, 35] {
        let markdown = chapter_markdown(chapter);
        for index in 1..=6 {
            let item_id = format!("CFT-{chapter:02}-{index:03}");
            let section = card(&markdown, &item_id, chapter)
                .unwrap_or_else(|| panic!("missing unique card anchor {item_id}"));
            assert_eq!(
                markdown
                    .matches(&format!("{{#cft-{chapter:02}-{index:03}}}"))
                    .count(),
                1
            );
            let spec = cft_spec(&item_id);
            if phase.rank() >= spec.completion_rank {
                let errors = complete_card_errors(&markdown, spec);
                assert!(
                    errors.is_empty(),
                    "completed {item_id} card differs from the contract: {errors:#?}"
                );
            } else {
                let observed = CARD_FIELDS.map(|field| card_field(section, field).is_some());
                assert!(card_presence_errors(&item_id, phase, &observed).is_empty());
            }
            let exercise_anchor = format!("{{#exercise-cft-{chapter:02}-e{index:02}}}");
            assert_eq!(
                markdown.matches(&exercise_anchor).count(),
                1,
                "{exercise_anchor}"
            );
        }
    }
}

#[test]
fn ls_route_contract_records_every_mathematical_obligation_in_order() {
    let contract = wave3_contract();
    let obligations = contract
        .split_once("## Mathematical obligations")
        .expect("mathematical-obligation section")
        .1;
    for spec in CFT_SPECS {
        let marker = format!("| {} |", spec.item_id);
        let row = obligations
            .split_once(&marker)
            .unwrap_or_else(|| panic!("contract omits {}", spec.item_id))
            .1
            .lines()
            .next()
            .expect("obligation row");
        for token in spec
            .hypotheses
            .iter()
            .chain(spec.statement_steps)
            .chain(spec.proof_steps)
        {
            assert!(
                row.contains(token),
                "{} obligation row omits `{token}`",
                spec.item_id
            );
        }
        assert!(
            steps_in_order(row, spec.hypotheses),
            "{} hypotheses are out of order",
            spec.item_id
        );
        assert!(
            steps_in_order(row, spec.statement_steps),
            "{} statement steps are out of order",
            spec.item_id
        );
        assert!(
            steps_in_order(row, spec.proof_steps),
            "{} proof steps are out of order",
            spec.item_id
        );
    }
    let normalized = contract.split_whitespace().collect::<Vec<_>>().join(" ");
    for field in CARD_FIELDS {
        assert!(
            normalized.contains(field),
            "contract omits card field `{field}`"
        );
    }
    for label in [
        "Motivation.",
        "Mathematical object / ML counterpart.",
        "Exact transfer.",
        "Non-transfer.",
        "Diagnostic.",
    ] {
        assert!(
            normalized.contains(label),
            "contract omits future card label `{label}`"
        );
    }
}

#[test]
fn ls_route_contract_freezes_corrected_consequence_routes_and_full_ambient_ledger() {
    assert_eq!(
        cft_spec("CFT-35-003").proof_steps,
        [
            "polynomial approximants",
            "uniform scalar convergence",
            "matrix evaluation convergence",
            "maximum convergence",
            "limit inequality",
        ]
    );
    assert_eq!(
        cft_spec("CFT-35-005").statement_steps,
        [
            "IsCompact (closedOperatorNumericalRange A)",
            "spectrum ℂ A ⊆ closedOperatorNumericalRange A",
            "rational constant-two inequality",
        ]
    );
    let ambient = [
        "i n : Type*",
        "TopologicalSpace i",
        "CompactSpace i",
        "MeasurableSpace i",
        "BorelSpace i",
        "OpensMeasurableSpace i",
        "SecondCountableTopologyEither i ℂ",
        "Fintype n",
        "DecidableEq n",
        "Nonempty n",
        "μ : Measure i",
        "IsFiniteMeasure μ",
        "Ω : Set ℂ",
        "EuclideanVector n finite-dimensional complete",
        "L2(μ; EuclideanVector n) complete",
        "Γ : ParametricConvexBoundary Ω",
        "B : SquareMatrix n",
        "hWB : numericalRange B ⊆ Ω",
        "q : Polynomial ℂ",
        "hq : ∀ z ∈ closure Ω, ‖Polynomial.eval z q‖ ≤ 1",
        "hCauchy : HasParametricPolynomialCauchyFormula Γ μ B",
    ];
    assert_eq!(cft_spec("CFT-34-001").hypotheses, ambient);
    for exact in ambient {
        assert!(
            cft_spec("CFT-34-001").hypotheses.contains(&exact),
            "CFT-34-001 omits ambient/provider requirement `{exact}`"
        );
        let deleted = future_card_fixture(cft_spec("CFT-34-001")).replacen(exact, "", 1);
        assert!(
            complete_card_errors(&deleted, cft_spec("CFT-34-001"))
                .iter()
                .any(|error| error.contains("Hypothesis ledger")),
            "CFT-34-001 accepted omission of `{exact}`"
        );
    }
    for (item_id, field, tokens) in [
        ("CFT-35-003", "Proof", cft_spec("CFT-35-003").proof_steps),
        (
            "CFT-35-005",
            "Statement",
            cft_spec("CFT-35-005").statement_steps,
        ),
    ] {
        let fixture = future_card_fixture(cft_spec(item_id));
        for token in tokens {
            let deleted = fixture.replacen(token, "", 1);
            assert!(
                complete_card_errors(&deleted, cft_spec(item_id))
                    .iter()
                    .any(|error| error.contains(field)),
                "{item_id} accepted omission of `{token}`"
            );
        }
        let swapped = fixture
            .replacen(tokens[0], "__WAVE3_SWAP__", 1)
            .replacen(tokens[1], tokens[0], 1)
            .replacen("__WAVE3_SWAP__", tokens[1], 1);
        assert!(
            complete_card_errors(&swapped, cft_spec(item_id))
                .iter()
                .any(|error| error.contains(field)),
            "{item_id} accepted reordered {field} obligations"
        );
    }
}

#[test]
fn chapter_34_boundary_data_card_exposes_the_concrete_package_field_by_field() {
    let contract = wave3_contract();
    assert!(
        active_phase(&contract).rank() >= Wave3Phase::Chapter34BoundaryData.rank(),
        "Chapter 34 may not retreat behind the completed boundary-data phase"
    );

    let markdown = chapter_markdown(34);
    let section = card(&markdown, "CFT-34-001", 34).expect("CFT-34-001 card");
    let errors = complete_card_errors(section, cft_spec("CFT-34-001"));
    assert!(errors.is_empty(), "CFT-34-001 card contract: {errors:#?}");

    let statement = normalized_visible_markdown(
        card_field(section, "Statement").expect("CFT-34-001 statement"),
    );
    for exact in [
        "F(σ) := ((2 * π)^-1 * Γ.speed σ) • (Γ.normal σ • (Γ.point σ • I - B)^-1)",
        "D.density σ := F(σ) + F(σ)ᴴ",
        "(Vx)(σ) := (Real.sqrt 2)^-1 • euclideanOperator (D.density σ)^(1/2) x",
        "(Qf)(σ) := h(σ) f(σ)",
        "Φ_D(g) := (1 / 2) ∫ σ, g(σ) • D.density σ ∂μ",
        "C_k := ∫ σ, star ((h^k) σ) • F(σ) ∂μ",
        "E_k := euclideanOperator C_k",
    ] {
        assert!(
            statement.contains(exact),
            "CFT-34-001 statement omits `{exact}`"
        );
    }

    let hypotheses = visible_markdown(
        card_field(section, "Hypothesis ledger").expect("CFT-34-001 hypothesis ledger"),
    );
    for exact in cft_spec("CFT-34-001").hypotheses {
        assert!(
            hypotheses.contains(exact),
            "CFT-34-001 hypothesis ledger omits `{exact}`"
        );
    }
    for exact in [
        "K := L2(μ; EuclideanVector n)",
        "F(σ) := ((2 * π)^-1 * Γ.speed σ) •",
        "Γ.normal σ • (Γ.point σ • I - B)^-1",
        "D.density σ := F(σ) + F(σ)ᴴ",
        "h := parametricPolynomialBoundaryFunction Γ q",
        "P := polynomialEval q B",
        "V := (boundaryEmbedding D).toContinuousLinearMap",
        "V* := ContinuousLinearMap.adjoint V",
        "Q := bcfMulL h",
        "C_k := ∫ σ, star ((h^k) σ) • F(σ) ∂μ",
        "E_k := euclideanOperator C_k",
        "M := ∫ σ, ‖parametricBoundaryFirstPart Γ B σ‖ ∂μ",
    ] {
        assert!(section.contains(exact), "CFT-34-001 omits `{exact}`");
    }

    let proof = visible_markdown(card_field(section, "Proof").expect("CFT-34-001 proof"));
    for exact in [
        "T := euclideanOperator P",
        "V_isometry",
        "Q_norm_le_one",
        "perturbation_eq",
        "bound_nonneg",
        "perturbation_norm_le",
        "commutes_with_target",
        "Q_VV*",
        "V V* is the orthogonal projection onto range V",
        "LoristSchwenninger.dilationDataOfParametricPolynomial",
    ] {
        assert!(proof.contains(exact), "CFT-34-001 proof omits `{exact}`");
    }
    assert!(
        steps_in_order(&proof, cft_spec("CFT-34-001").proof_steps),
        "CFT-34-001 does not discharge DilationData fields in contract order"
    );

    if active_phase(&contract).rank() < Wave3Phase::Chapter34RealizationComplete.rank() {
        for pending in ["CFT-34-004", "CFT-34-005", "CFT-34-006"] {
            let pending_card = card(&markdown, pending, 34).expect("pending Chapter 34 card");
            for field in CARD_FIELDS {
                assert!(
                    card_field(pending_card, field).is_none(),
                    "{pending} advanced before the realization phase"
                );
            }
        }
    }
}

#[test]
fn chapter_34_boundary_data_separates_density_positivity_mass_and_l2_measurability() {
    let markdown = chapter_markdown(34);
    let card = card(&markdown, "CFT-34-001", 34).expect("CFT-34-001 card");
    let section = normalized_visible_markdown(card);
    for exact in [
        "F(σ) := ((2 * π)^-1 * Γ.speed σ) • (Γ.normal σ • (Γ.point σ • I - B)^-1)",
        "D.density σ := F(σ) + F(σ)ᴴ",
        "outward support geometry together with hWB",
        "proves D.density σ is positive semidefinite",
        "does not prove positivity",
        "constant Cauchy moment",
        "∫ σ, D.density σ ∂μ = 2I",
        "AEStronglyMeasurable",
        "almost-everywhere strongly measurable",
        "L2 is a space of equivalence classes modulo almost-everywhere equality",
    ] {
        assert!(
            section.contains(exact),
            "CFT-34-001 density/measurability derivation omits `{exact}`"
        );
    }
    let proof = normalized_visible_markdown(
        card_field(card, "Proof").expect("CFT-34-001 positivity and mass proof"),
    );
    assert!(
        steps_in_order(
            &proof,
            &[
                "outward support geometry together with hWB",
                "proves D.density σ is positive semidefinite",
                "constant Cauchy moment does not prove positivity",
                "∫ σ, D.density σ ∂μ = 2I",
            ],
        ),
        "CFT-34-001 must derive positivity before separately deriving mass"
    );
}

#[test]
fn chapter_34_boundary_data_types_projection_and_perturbation_algebra_are_explicit() {
    let markdown = chapter_markdown(34);
    let section =
        normalized_visible_markdown(card(&markdown, "CFT-34-001", 34).expect("CFT-34-001 card"));
    for exact in [
        "C_k : SquareMatrix n",
        "E_k : E →L[ℂ] E",
        "Φ_D(h^k) : SquareMatrix n",
        "2 Φ_D(h^k) = P^k + C_kᴴ",
        "2 Φ_D(h^k)ᴴ = (Pᴴ)^k + C_k",
        "V* (Q*)^k V = euclideanOperator (Φ_D(h^k)ᴴ)",
        "E_k = 2 V* (Q*)^k V - (T*)^k",
        "inner-product preservation",
        "V*V = I",
        "(VV*)* = VV*",
        "(VV*)² = VV*",
        "range(VV*) = range(V)",
        "V V* is the orthogonal projection onto range V",
        "‖V*Q^kV - euclideanOperator (Φ_D(h^k))‖",
    ] {
        assert!(
            section.contains(exact),
            "CFT-34-001 typed dilation derivation omits `{exact}`"
        );
    }
}

#[test]
fn chapter_34_boundary_data_atomic_model_is_computable_and_names_its_failure_mode() {
    let markdown = chapter_markdown(34);
    let section = visible_markdown(card(&markdown, "CFT-34-001", 34).expect("CFT-34-001 card"))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for exact in [
        "finite atomic boundary model: for i = Fin m with atom weights w_a, the boundary integral becomes ∑ a, w_a • D_a",
        "∑ a, w_a • D_a = 2I",
        "(Vx)_a := (w_a / 2)^(1/2) D_a^(1/2) x",
        "‖Vx‖² = ∑ a, (w_a / 2) ⟪D_a x, x⟫ = ‖x‖²",
        "(Qf)_a := h_a f_a",
        "V*(f) = ∑ a, (w_a / 2)^(1/2) D_a^(1/2) f_a",
        "V* Q V = euclideanOperator ((1 / 2) ∑ a, w_a h_a D_a)",
        "if ∑ a, w_a • D_a ≠ 2I, then V is not certified isometric",
        "m = 1, w_0 = 1, and D_0 = I",
        "‖Vx‖² = (1 / 2) ‖x‖²",
    ] {
        assert!(section.contains(exact), "atomic model omits `{exact}`");
    }
}

#[test]
fn chapter_34_boundary_data_card_states_history_ml_transfer_and_formal_boundary() {
    let markdown = chapter_markdown(34);
    let section = card(&markdown, "CFT-34-001", 34).expect("CFT-34-001 card");
    let history =
        visible_markdown(card_field(section, "Historical context").expect("CFT-34-001 history"))
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
    for exact in [
        "LS source-derived",
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124",
        "Lorist--Schwenninger boundary realization",
        "no priority claim",
        "no peer-review, acceptance, or journal-publication receipt",
    ] {
        assert!(
            history.contains(exact),
            "CFT-34-001 history omits `{exact}`"
        );
    }

    let ml = visible_markdown(card_field(section, "ML analogy").expect("CFT-34-001 ML"))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for exact in [
        "Mathematical object / ML counterpart.",
        "lifted feature-space realization",
        "Exact transfer.",
        "V* Q^k V",
        "Non-transfer.",
        "Learned features do not imply the exact mass or Cauchy moment identities",
        "Diagnostic.",
        "‖V*V - I‖",
        "‖V*Q^kV - euclideanOperator (Φ_D(h^k))‖",
    ] {
        assert!(ml.contains(exact), "CFT-34-001 ML field omits `{exact}`");
    }

    let lean =
        visible_markdown(card_field(section, "Lean correspondence").expect("CFT-34-001 Lean"))
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
    for exact in [
        "CrouzeixTextbook.Part06.boundary_dilation_data",
        "CrouzeixConjecture.LoristSchwenninger.dilationDataOfParametricPolynomial",
        "formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean",
        "formalization/lean/Crouzeix/LoristSchwenninger/ConcreteDilation.lean",
        "formal_mode `definition`",
        "standard axioms `Classical.choice`, `Quot.sound`, and `propext`",
        "does not formalize the finite atomic example as a separate theorem",
    ] {
        assert!(
            lean.contains(exact),
            "CFT-34-001 Lean field omits `{exact}`"
        );
    }
}

#[test]
fn chapter_34_boundary_data_atlas_render_preserves_the_typed_derivation() {
    let corpus = read_json(&workspace_root().join("atlas/src/content/generated/corpus.json"));
    let chapter = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| document["title"] == "Chapter 34: The Lorist–Schwenninger realization")
        .expect("Chapter 34 Atlas document");
    let html = chapter["html"].as_str().expect("Chapter 34 Atlas HTML");
    for rendered in [
        "F(σ) := ((2 * π)^-1 * Γ.speed σ) •",
        "Γ.normal σ • (Γ.point σ • I - B)^-1",
        "D.density σ := F(σ) + F(σ)ᴴ",
        "(Vx)(σ) := (Real.sqrt 2)^-1 •",
        "euclideanOperator (D.density σ)^(1/2) x",
        "(Qf)(σ) := h(σ) f(σ)",
        "Φ_D(g) := (1 / 2) ∫ σ, g(σ) • D.density σ ∂μ",
        "AEStronglyMeasurable",
        "C_k : SquareMatrix n",
        "E_k : E →L[ℂ] E",
        "(VV*)* = VV*",
        "(VV*)² = V(V*V)V* = VV*",
        "range(VV*) = range(V)",
        "2 Φ_D(h^k) = P^k + C_kᴴ",
        "2 Φ_D(h^k)ᴴ = (Pᴴ)^k + C_k",
        "V* (Q*)^k V = euclideanOperator (Φ_D(h^k)ᴴ)",
        "E_k = 2 V* (Q*)^k V - (T*)^k",
        "V* Q V = euclideanOperator",
        "‖Vx‖² = (1 / 2) ‖x‖²",
    ] {
        assert!(
            html.contains(rendered),
            "Chapter 34 Atlas HTML omits `{rendered}`"
        );
    }
}

#[test]
fn chapter_34_boundary_data_exercises_have_full_prompts_solutions_and_exact_lean_links() {
    let markdown = chapter_markdown(34);
    let visible = visible_markdown(&markdown);
    for exact in [
        "Given D : PositiveBoundaryDensity μ and x : EuclideanVector n, prove ‖boundaryEmbeddingToLp D x‖ = ‖x‖.",
        "Solution to CFT-34-E01",
        "boundaryEmbeddingField_sq_norm_ae",
        "integral_normalized_boundaryQuadraticForm",
        "CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_01_solution",
        "Given D, h, and x, prove (V* M_h V)x = euclideanOperator (boundaryPhiCLM D h)x by expanding the L2 inner product.",
        "Solution to CFT-34-E02",
        "boundaryEmbeddingField_memLp",
        "bcfMulL_apply_ae",
        "boundarySquareRoot_mul_self_ae",
        "boundaryPhiCLM_apply",
        "CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_02_solution",
    ] {
        assert!(visible.contains(exact), "Chapter 34 exercises omit `{exact}`");
    }
    for file in [
        "[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]]",
        "[[formalization/lean/Crouzeix/LoristSchwenninger/BoundaryEmbedding.lean|BoundaryEmbedding.lean]]",
        "[[formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean|CompressionMoments.lean]]",
    ] {
        assert!(markdown.contains(file), "Chapter 34 exercises omit code link `{file}`");
    }
}

#[test]
fn chapter_34_moments_phase_advances_only_the_two_moment_cards() {
    let contract = wave3_contract();
    assert!(
        active_phase(&contract).rank() >= Wave3Phase::Chapter34Moments.rank(),
        "Chapter 34 may not retreat behind the completed moments phase"
    );

    let markdown = chapter_markdown(34);
    for completed in ["CFT-34-001", "CFT-34-002", "CFT-34-003"] {
        let errors = complete_card_errors(&markdown, cft_spec(completed));
        assert!(
            errors.is_empty(),
            "completed {completed} card differs from the moments contract: {errors:#?}"
        );
    }
    if active_phase(&contract) == Wave3Phase::Chapter34Moments {
        for pending in ["CFT-34-004", "CFT-34-005", "CFT-34-006"] {
            let section = card(&markdown, pending, 34).expect("pending Chapter 34 card");
            for field in CARD_FIELDS {
                assert!(
                    card_field(section, field).is_none(),
                    "{pending} advanced during the moments-only phase"
                );
            }
        }
    }
}

#[test]
fn chapter_34_moments_first_compression_displays_the_full_typed_integral_calculation() {
    let markdown = chapter_markdown(34);
    let section = card(&markdown, "CFT-34-002", 34).expect("CFT-34-002 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-34-002 proof"));
    for exact in [
        "V : EuclideanVector n →L[ℂ] L2(μ; EuclideanVector n)",
        "M_h : L2(μ; EuclideanVector n) →L[ℂ] L2(μ; EuclideanVector n)",
        "V* : L2(μ; EuclideanVector n) →L[ℂ] EuclideanVector n",
        "⟪y, (V* M_h V)x⟫",
        "⟪Vy, M_h(Vx)⟫_{L2}",
        "∫ σ, ⟪(Vy)(σ), h(σ)(Vx)(σ)⟫ ∂μ",
        "(Vx)(σ) = (Real.sqrt 2)^-1 • euclideanOperator (D.density σ)^(1/2) x",
        "D(σ)^(1/2) D(σ)^(1/2) = D(σ)",
        "(1 / 2) ∫ σ, ⟪y, euclideanOperator (h(σ) • D.density σ) x⟫ ∂μ",
        "⟪y, euclideanOperator ((1 / 2) ∫ σ, h(σ) • D.density σ ∂μ) x⟫",
        "⟪y, euclideanOperator (boundaryPhiCLM D h) x⟫",
        "V* M_h V = euclideanOperator (boundaryPhiCLM D h)",
    ] {
        assert!(proof.contains(exact), "CFT-34-002 proof omits `{exact}`");
    }
    assert!(
        steps_in_order(
            &proof,
            &[
                "⟪y, (V* M_h V)x⟫",
                "⟪Vy, M_h(Vx)⟫_{L2}",
                "∫ σ, ⟪(Vy)(σ), h(σ)(Vx)(σ)⟫ ∂μ",
                "D(σ)^(1/2) D(σ)^(1/2) = D(σ)",
                "(1 / 2) ∫ σ, ⟪y, euclideanOperator (h(σ) • D.density σ) x⟫ ∂μ",
                "⟪y, euclideanOperator (boundaryPhiCLM D h) x⟫",
                "V* M_h V = euclideanOperator (boundaryPhiCLM D h)",
            ],
        ),
        "CFT-34-002 hides or reorders the compression algebra"
    );

    let hypotheses = normalized_visible_markdown(
        card_field(section, "Hypothesis ledger").expect("CFT-34-002 hypotheses"),
    );
    for exact in [
        "MeasurableSpace i",
        "TopologicalSpace i",
        "BorelSpace i",
        "SecondCountableTopologyEither i ℂ",
        "Fintype n",
        "DecidableEq n",
        "μ : Measure i",
        "D : PositiveBoundaryDensity (n := n) μ",
        "h : i →ᵇ ℂ",
    ] {
        assert!(hypotheses.contains(exact), "CFT-34-002 omits `{exact}`");
    }
}

#[test]
fn chapter_34_moments_scalar_fourier_example_identifies_the_surviving_mode() {
    let markdown = chapter_markdown(34);
    let section =
        normalized_visible_markdown(card(&markdown, "CFT-34-002", 34).expect("CFT-34-002 card"));
    for exact in [
        "p(z) = ∑_{m=0}^N a_m z^m",
        "(1 / (2πi)) ∮_{|z|=1} p(z) / z dz",
        "∮_{|z|=1} z^-1 dz = 2πi",
        "∮_{|z|=1} z^(m-1) dz = 0 for m ≥ 1",
        "only the exponent -1 mode survives",
        "p(0) = a_0",
        "all integer powers `z^r`, with `r ∈ ℤ` and `r ≠ -1`, have the single-valued primitive",
        "`z^(r+1)/(r+1)` on the punctured plane `ℂ \\ {0}`",
        "an open neighborhood of the unit circle",
        "first compression moment",
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124",
    ] {
        assert!(
            section.contains(exact),
            "scalar Fourier calculation omits `{exact}`"
        );
    }
    assert!(
        !section.contains("all all"),
        "scalar Fourier prose retains duplicated word"
    );
}

#[test]
fn chapter_34_moments_power_compression_exposes_induction_and_all_power_use() {
    let markdown = chapter_markdown(34);
    let section = card(&markdown, "CFT-34-003", 34).expect("CFT-34-003 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-34-003 proof"));
    for exact in [
        "k = 0",
        "M_h^0 = I = M_(h^0)",
        "M_h^(k+1) = M_h^k M_h",
        "M_(h^k) M_h = M_(h^k h)",
        "M_(h^(k+1))",
        "bcfMulL_mul",
        "bcfMulL_pow",
        "V* M_h^k V = V* M_(h^k) V",
        "V* M_(h^k) V = euclideanOperator (boundaryPhiCLM D (h^k))",
        "V* M_h^k V = euclideanOperator (boundaryPhiCLM D (h^k))",
        "matrix boundaryPhiCLM D (h^k)",
        "continuous linear map euclideanOperator (boundaryPhiCLM D (h^k))",
        "for every k : Nat",
    ] {
        assert!(proof.contains(exact), "CFT-34-003 proof omits `{exact}`");
    }
    assert!(
        steps_in_order(&proof, cft_spec("CFT-34-003").proof_steps),
        "CFT-34-003 does not follow the compiler-backed proof bridge"
    );

    for exact in [
        "k = n",
        "k = n + 1",
        "(Q*)^n",
        "(T*)^n",
        "the identity at k = 1 cannot instantiate this",
        "quantified field when",
        "all positive powers",
        "telescoping sum starts at n = 1",
        "k = 0 is only the multiplier-induction base and unitality check",
        "Chapter 33 recurrence",
    ] {
        assert!(
            normalized_visible_markdown(section).contains(exact),
            "CFT-34-003 all-power explanation omits `{exact}`"
        );
    }
}

#[test]
fn chapter_34_moments_cards_state_ml_transfer_nontransfer_diagnostics_and_formal_boundaries() {
    let markdown = chapter_markdown(34);
    for (item_id, tokens) in [
        (
            "CFT-34-002",
            vec![
                "feature covariance compression",
                "Exact transfer.",
                "the exact square-root and mass identities",
                "Non-transfer.",
                "a minibatch covariance is not a Bochner integral identity",
                "Diagnostic.",
                "R_1 := V* M_h V - euclideanOperator (boundaryPhiCLM D h)",
                "‖R_1‖",
                "Formal boundary.",
                "does not turn a sampled residual into an exact theorem",
            ],
        ),
        (
            "CFT-34-003",
            vec![
                "multi-step rollout moments",
                "Exact transfer.",
                "tied linear transition",
                "exact first-moment density identity",
                "Non-transfer.",
                "untied, nonlinear, stochastic, or time-varying rollouts",
                "Diagnostic.",
                "R_k := V* M_h^k V - euclideanOperator (boundaryPhiCLM D (h^k))",
                "calculate ‖R_k‖ for each sampled k",
                "Formal boundary.",
                "no finite residual table proves the infinite exact family",
            ],
        ),
    ] {
        let section = card(&markdown, item_id, 34).expect("moment card");
        let ml = normalized_visible_markdown(
            card_field(section, "ML analogy").expect("moment ML field"),
        );
        for exact in tokens {
            assert!(ml.contains(exact), "{item_id} ML field omits `{exact}`");
        }
    }
}

#[test]
fn chapter_34_moments_exercises_match_scalar_cauchy_and_induction_compiler_targets() {
    let markdown = chapter_markdown(34);
    let visible = normalized_visible_markdown(&markdown);
    for exact in [
        "CFT-34-E03",
        "Prove the scalar Cauchy/Fourier mode calculation",
        "∮ z in C(0, 1), z⁻¹ = 2 * π * I",
        "n ≠ -1 → ∮ z in C(0, 1), z ^ n = 0",
        "∑ m ∈ Finset.range (N + 1), a m * z ^ ((m : ℤ) - 1)",
        "a 0 * (2 * π * I)",
        "finite-sum linearity",
        "Solution to CFT-34-E03",
        "circleIntegral.integral_sub_inv_of_mem_ball",
        "circleIntegral.integral_sub_zpow_of_ne",
        "CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_03_solution",
        "CFT-34-E04",
        "derive the pointwise power-compression identity",
        "rewrite bcfMulL_pow",
        "apply the first-moment compression theorem to h^k",
        "Solution to CFT-34-E04",
        "CrouzeixTextbook.Part06.Exercises.Chapter34.exercise_04_solution",
    ] {
        assert!(
            visible.contains(exact),
            "Chapter 34 moment exercises omit `{exact}`"
        );
    }
    for file in [
        "[[formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean|Chapter34.lean]]",
        "[[formalization/lean/Crouzeix/LoristSchwenninger/CompressionMoments.lean|CompressionMoments.lean]]",
        "[[formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean|BoundaryMultiplier.lean]]",
        "[Mathlib CircleIntegral.lean](https://github.com/leanprover-community/mathlib4/blob/v4.32.1/Mathlib/MeasureTheory/Integral/CircleIntegral.lean)",
    ] {
        assert!(markdown.contains(file), "Chapter 34 moment exercises omit `{file}`");
    }

    let lean = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean"),
    )
    .expect("Chapter 34 Lean module");
    let e03 = lean
        .split_once("theorem exercise_03_solution")
        .expect("Chapter 34 E03 solution")
        .1
        .split_once("theorem exercise_04_solution")
        .expect("Chapter 34 E04 boundary")
        .0;
    for exact in ["circleIntegral.integral_fun_sum", "Finset.sum_eq_single"] {
        assert!(e03.contains(exact), "CFT-34-E03 Lean proof omits `{exact}`");
    }
    let e04 = lean
        .split_once("theorem exercise_04_solution")
        .expect("Chapter 34 E04 solution")
        .1;
    for exact in [
        "bcfMulL_pow",
        "boundaryEmbedding_adjoint_comp_bcfMulL_comp_boundaryEmbedding",
    ] {
        assert!(e04.contains(exact), "CFT-34-E04 Lean proof omits `{exact}`");
    }
    assert!(
        !e04.contains("boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding"),
        "CFT-34-E04 may not solve itself through the parent power-moment theorem"
    );
}

#[test]
fn chapter_34_moments_e03_compiler_audits_exact_external_proof_dependencies() {
    let row = active_dependency_receipt()
        .as_array()
        .expect("active dependency rows")
        .iter()
        .find(|row| row["name"] == EXERCISE_SPECS[2].solution)
        .expect("compiled CFT-34-E03 row");
    assert_eq!(
        row["audited_external_proof_dependencies"],
        json!(AUDITED_EXTERNAL_PROOF_DEPENDENCIES),
        "CFT-34-E03 must reach both exact Mathlib lemmas through its compiled proof body"
    );
    let mut missing_dependency = exercise_receipt_fixture(Wave3Phase::Chapter34Moments);
    missing_dependency["declarations"]
        .as_array_mut()
        .expect("fixture declarations")
        .iter_mut()
        .find(|row| row["name"] == EXERCISE_SPECS[2].solution)
        .expect("fixture CFT-34-E03 row")["audited_external_proof_dependencies"] =
        json!([AUDITED_EXTERNAL_PROOF_DEPENDENCIES[0]]);
    assert!(
        exercise_receipt_errors(&missing_dependency, Wave3Phase::Chapter34Moments)
            .iter()
            .any(|error| error.contains(AUDITED_EXTERNAL_PROOF_DEPENDENCIES[1])),
        "the receipt validator accepted a missing allowlisted external dependency"
    );

    let negative = compile_wave3_lean_fixture(
        r#"import CrouzeixTextbook.ExportReceipt
import CrouzeixHarp
open Lean Meta
namespace CrouzeixTextbook.Wave3ExternalNegativeProbe
open scoped Interval

theorem inverseModeHelper :
    (∮ z in C((0 : ℂ), 1), z⁻¹) = 2 * Real.pi * Complex.I := by
  simpa using circleIntegral.integral_sub_inv_of_mem_ball
    (c := (0 : ℂ)) (w := (0 : ℂ)) (R := 1) (by simp)

/- A comment naming circleIntegral.integral_sub_zpow_of_ne is not a proof dependency. -/
theorem candidate :
    (∮ z in C((0 : ℂ), 1), z⁻¹) = 2 * Real.pi * Complex.I :=
  inverseModeHelper

def allowlist : NameHashSet :=
  #[`circleIntegral.integral_sub_inv_of_mem_ball,
    `circleIntegral.integral_sub_zpow_of_ne].foldl
    (init := ({} : NameHashSet)) fun selected name => selected.insert name

partial def proofClosure (env : Environment) : List Name → NameHashSet → NameHashSet
  | [], seen => seen
  | name :: pending, seen =>
      if seen.contains name then proofClosure env pending seen
      else
        let seen := seen.insert name
        if allowlist.contains name ||
            !CrouzeixTextbook.ExportReceipt.isMaintainedName name then
          proofClosure env pending seen
        else
          match env.find? name with
          | none => proofClosure env pending seen
          | some info =>
              let dependencies := info.value? (allowOpaque := true)
                |>.map CrouzeixTextbook.ExportReceipt.collectConstants |>.getD {}
              proofClosure env (dependencies.toList ++ pending) seen

run_cmd do
  let env ← getEnv
  let some info := env.find? `CrouzeixTextbook.Wave3ExternalNegativeProbe.candidate
    | throwError "candidate missing"
  let roots := info.value? (allowOpaque := true)
    |>.map CrouzeixTextbook.ExportReceipt.collectConstants |>.getD {}
  let found := proofClosure env roots.toList {}
  unless found.contains `circleIntegral.integral_sub_inv_of_mem_ball do
    throwError "helper indirection hid integral_sub_inv_of_mem_ball"
  unless found.contains `circleIntegral.integral_sub_zpow_of_ne do
    throwError "missing audited dependency circleIntegral.integral_sub_zpow_of_ne"
end CrouzeixTextbook.Wave3ExternalNegativeProbe
"#,
    );
    assert!(
        !negative.status.success(),
        "a comment was incorrectly accepted as the missing external proof dependency"
    );
    let diagnostics = format!(
        "{}\n{}",
        String::from_utf8_lossy(&negative.stdout),
        String::from_utf8_lossy(&negative.stderr)
    );
    assert!(
        diagnostics.contains("missing audited dependency circleIntegral.integral_sub_zpow_of_ne"),
        "negative external-dependency audit failed for the wrong reason:\n{diagnostics}"
    );
    assert!(
        !diagnostics.contains("helper indirection hid"),
        "transitive proof-body audit failed to follow the maintained helper:\n{diagnostics}"
    );
}

#[test]
fn chapter_34_moments_publish_only_exact_moment_rows_and_exercise_receipts() {
    let phase = active_phase(&wave3_contract());
    let coverage = read_json(&contracts_root().join("coverage.json"));
    for item_id in ["CFT-34-002", "CFT-34-003"] {
        let spec = cft_spec(item_id);
        let row = array(&coverage, "items", "coverage")
            .iter()
            .find(|row| row["item_id"] == item_id)
            .unwrap_or_else(|| panic!("missing {item_id}"));
        assert_eq!(row["prose_proof_status"], "reconstructible");
        assert_eq!(row["lean_correspondence_status"], "exact");
        assert_eq!(row["formal_mode"], spec.completed_mode);
        assert_eq!(
            row["lean_declaration"]["underlying_declaration"],
            spec.provider_declaration
        );
    }
    if phase == Wave3Phase::Chapter34Moments {
        for item_id in ["CFT-34-004", "CFT-34-005", "CFT-34-006"] {
            let spec = cft_spec(item_id);
            let row = array(&coverage, "items", "coverage")
                .iter()
                .find(|row| row["item_id"] == item_id)
                .unwrap_or_else(|| panic!("missing {item_id}"));
            assert_eq!(row["prose_proof_status"], "summary");
            assert_eq!(
                row["lean_correspondence_status"],
                spec.baseline_correspondence
            );
        }
    }

    let exercises = read_json(&contracts_root().join("exercises.json"));
    for exercise_id in ["CFT-34-E03", "CFT-34-E04"] {
        let spec = EXERCISE_SPECS
            .iter()
            .find(|spec| spec.exercise_id == exercise_id)
            .expect("moment exercise spec");
        let row = array(&exercises, "exercises", "exercises")
            .iter()
            .find(|row| row["exercise_id"] == exercise_id)
            .unwrap_or_else(|| panic!("missing {exercise_id}"));
        assert_eq!(row["lean_solution"]["declaration"], spec.solution);
        assert_eq!(
            row["lean_solution"]["type_sha256"],
            spec.canonical_type_sha256.expect("moment exercise hash")
        );
        assert_eq!(row["lean_solution"]["axioms"], json!(AXIOMS));
    }
    if phase == Wave3Phase::Chapter34Moments {
        for exercise_id in ["CFT-34-E05", "CFT-34-E06"] {
            let row = array(&exercises, "exercises", "exercises")
                .iter()
                .find(|row| row["exercise_id"] == exercise_id)
                .unwrap_or_else(|| panic!("missing {exercise_id}"));
            assert!(
                row["lean_solution"].is_null(),
                "{exercise_id} advanced early"
            );
        }
    }
}

#[test]
fn chapter_34_moments_atlas_render_preserves_integral_and_power_algebra() {
    let corpus = read_json(&workspace_root().join("atlas/src/content/generated/corpus.json"));
    let chapter = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| document["title"] == "Chapter 34: The Lorist–Schwenninger realization")
        .expect("Chapter 34 Atlas document");
    let html = chapter["html"].as_str().expect("Chapter 34 Atlas HTML");
    for rendered in [
        "⟪y, (V* M_h V)x⟫",
        "∫ σ, ⟪(Vy)(σ), h(σ)(Vx)(σ)⟫ ∂μ",
        "D(σ)^(1/2) D(σ)^(1/2) = D(σ)",
        "(1 / (2πi)) ∮_{|z|=1} p(z) / z dz",
        "only the exponent -1 mode survives",
        "M_h^(k+1) = M_h^k M_h",
        "V* M_h^k V = euclideanOperator",
        "the identity at k = 1 cannot instantiate this",
        "quantified field when",
        "<pre><code class=\"language-text\">R_k := V* M_h^k V - Φ(h^k),",
        "where this displayed <code>Φ(h^k)</code> abbreviates the operator",
    ] {
        assert!(
            html.contains(rendered),
            "Chapter 34 Atlas HTML omits `{rendered}`"
        );
    }
    assert!(
        !html.contains("<dl>") && !html.contains("<dt>") && !html.contains("<dd>"),
        "Chapter 34 Atlas HTML renders the empirical residual as a definition list"
    );
}

#[test]
fn chapter_34_realization_phase_completes_exactly_chapter_34() {
    let contract = wave3_contract();
    assert!(
        active_phase(&contract).rank() >= Wave3Phase::Chapter34RealizationComplete.rank(),
        "the active phase regressed before the completed Chapter 34 realization"
    );

    let markdown = chapter_markdown(34);
    for completed in [
        "CFT-34-001",
        "CFT-34-002",
        "CFT-34-003",
        "CFT-34-004",
        "CFT-34-005",
        "CFT-34-006",
    ] {
        let errors = complete_card_errors(&markdown, cft_spec(completed));
        assert!(
            errors.is_empty(),
            "completed {completed} card differs from the realization contract: {errors:#?}"
        );
    }
}

#[test]
fn chapter_34_realization_multiplier_power_proof_tracks_every_equality_layer() {
    let markdown = chapter_markdown(34);
    let section = card(&markdown, "CFT-34-004", 34).expect("CFT-34-004 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-34-004 proof"));
    for exact in [
        "bcfMulL (h ^ 0) = 1",
        "(bcfMulL h) ^ 0 = 1",
        "bcfMulL (h ^ (k + 1))",
        "bcfMulL (h ^ k * h)",
        "bcfMulL (h ^ k) * bcfMulL h",
        "(bcfMulL h) ^ k * bcfMulL h",
        "(bcfMulL h) ^ (k + 1)",
        "M_h^2 f = h • (h • f) = h^2 • f",
        "bounded continuous functions",
        "almost everywhere",
        "ContinuousLinearMap.ext",
        "Lp.ext",
    ] {
        assert!(proof.contains(exact), "CFT-34-004 proof omits `{exact}`");
    }
    assert!(
        steps_in_order(&proof, cft_spec("CFT-34-004").proof_steps),
        "CFT-34-004 does not follow the compiler-backed induction order"
    );
}

#[test]
fn chapter_34_realization_contractive_bound_includes_strict_case_without_claiming_equality() {
    let markdown = chapter_markdown(34);
    let section = card(&markdown, "CFT-34-005", 34).expect("CFT-34-005 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-34-005 proof"));
    for exact in [
        "‖h(σ)f(σ)‖ = ‖h(σ)‖ ‖f(σ)‖",
        "‖h(σ)‖ ≤ ‖h‖∞",
        "‖M_h f‖₂ ≤ ‖h‖∞ ‖f‖₂",
        "‖M_h‖ ≤ ‖h‖∞",
        "‖h‖∞ ≤ 1",
        "‖M_h‖ ≤ 1",
        "equality is unnecessary",
        "ρ < 1",
        "‖M_h‖ ≤ ρ < 1",
    ] {
        assert!(proof.contains(exact), "CFT-34-005 proof omits `{exact}`");
    }
    assert!(
        steps_in_order(&proof, cft_spec("CFT-34-005").proof_steps),
        "CFT-34-005 hides the pointwise-to-operator norm chain"
    );
}

#[test]
fn chapter_34_realization_final_assembly_maps_every_hypothesis_and_undoes_normalization() {
    let markdown = chapter_markdown(34);
    let section = card(&markdown, "CFT-34-006", 34).expect("CFT-34-006 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-34-006 proof"));
    for exact in [
        "data := dilationDataOfParametricPolynomial Γ B hWB q hq hCauchy",
        "data.T = euclideanOperator (polynomialEval q B)",
        "data.V_isometry",
        "data.Q_norm_le_one",
        "data.perturbation_eq",
        "data.bound_nonneg",
        "data.perturbation_norm_le",
        "data.commutes_with_target",
        "data.norm_target_le_two",
        "‖euclideanOperator (polynomialEval q B)‖ ≤ 2",
        "m := sSup",
        "q := m⁻¹ • p",
        "polynomialEval q B = m⁻¹ • polynomialEval p B",
        "‖euclideanOperator (polynomialEval p B)‖ ≤ 2 * m",
        "m = 0",
        "m > 0",
    ] {
        assert!(proof.contains(exact), "CFT-34-006 assembly omits `{exact}`");
    }
    assert!(
        steps_in_order(&proof, cft_spec("CFT-34-006").proof_steps),
        "CFT-34-006 does not instantiate the concrete data before applying Chapter 33"
    );
}

#[test]
fn chapter_34_realization_uses_an_admissible_open_domain_for_the_sharp_matrix() {
    let markdown = chapter_markdown(34);
    let section = card(&markdown, "CFT-34-006", 34).expect("CFT-34-006 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-34-006 proof"));
    for exact in [
        "Ω_r := {z : ℂ | ‖z‖ < r}",
        "r > 1",
        "W(A_{0,2}) = {z : ℂ | ‖z‖ ≤ 1} ⊂ Ω_r",
        "q_r(z) := z / r",
        "Γ_r : ParametricConvexBoundary Ω_r",
        "μ_r",
        "hCauchy_r : HasParametricPolynomialCauchyFormula Γ_r μ_r A_{0,2}",
        "hW_r : numericalRange A_{0,2} ⊆ Ω_r",
        "not sufficient by itself",
        "‖q_r(A_{0,2})‖ = 2 / r ≤ 2",
        "‖A_{0,2}‖ ≤ 2 * r",
        "r ↓ 1",
        "Chapter 35",
    ] {
        assert!(
            proof.contains(exact),
            "CFT-34-006 admissible-domain example omits `{exact}`"
        );
    }
    assert!(!proof.contains("Its numerical range is the closed unit disk, so m=1"));
    assert!(!proof.contains("The estimate reads 2≤2·1, with equality"));
}

#[test]
fn chapter_34_realization_normalization_proves_both_supremum_branches() {
    let markdown = chapter_markdown(34);
    let section = card(&markdown, "CFT-34-006", 34).expect("CFT-34-006 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-34-006 proof"));
    for exact in [
        "K := closure Ω",
        "K is compact and nonempty",
        "S_p := {t : ℝ | ∃ z ∈ K, t = ‖Polynomial.eval z p‖}",
        "m := sSup S_p",
        "S_p is nonempty and bounded above",
        "m is finite, nonnegative, and attained",
        "m > 0",
        "q := m⁻¹ • p",
        "m = 0",
        "p vanishes on K",
        "Ω is nonempty and open",
        "polynomial identity theorem",
        "p = 0",
        "polynomialEval p B = 0",
        "‖euclideanOperator (polynomialEval p B)‖ ≤ 2 * m",
        "does not package this unnormalization argument",
    ] {
        assert!(
            proof.contains(exact),
            "CFT-34-006 normalization omits `{exact}`"
        );
    }
    assert!(!proof.contains("Chapter 35 owns that step"));
    let chapter = normalized_visible_markdown(&markdown);
    assert!(!chapter.contains("remaining zero-normalization branch is left"));
    assert!(!chapter.contains("must discharge the zero-normalization"));
}

#[test]
fn chapter_34_realization_maps_exactly_six_proof_fields_and_definitional_target() {
    let markdown = chapter_markdown(34);
    let section = card(&markdown, "CFT-34-006", 34).expect("CFT-34-006 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-34-006 proof"));
    for exact in [
        "six proof fields",
        "Target identification (definitional data)",
        "data.T = euclideanOperator (polynomialEval q B)",
        "data.V_isometry",
        "data.Q_norm_le_one",
        "data.perturbation_eq",
        "data.bound_nonneg",
        "data.perturbation_norm_le",
        "data.commutes_with_target",
    ] {
        assert!(
            proof.contains(exact),
            "CFT-34-006 field ledger omits `{exact}`"
        );
    }
    assert!(!proof.contains("seven proof obligations"));
}

#[test]
fn chapter_34_realization_ml_contract_has_exact_transfer_nontransfer_and_nonnormal_diagnostic() {
    let markdown = chapter_markdown(34);
    for item_id in ["CFT-34-004", "CFT-34-005", "CFT-34-006"] {
        let section = card(&markdown, item_id, 34).expect("realization card");
        let ml = normalized_visible_markdown(
            card_field(section, "ML analogy").expect("realization ML field"),
        );
        for label in [
            "Mathematical object / ML counterpart.",
            "Exact transfer.",
            "Non-transfer.",
            "Diagnostic.",
        ] {
            assert!(ml.contains(label), "{item_id} ML field omits `{label}`");
        }
    }

    let ml = normalized_visible_markdown(
        card_field(
            card(&markdown, "CFT-34-006", 34).expect("CFT-34-006 card"),
            "ML analogy",
        )
        .expect("CFT-34-006 ML field"),
    );
    for exact in [
        "boundary multiplication",
        "diagonal lifted feature action",
        "exact compression identity",
        "learned or approximate features",
        "do not supply exact moments",
        "A_{λ,α}",
        "V̂* M̂_h^k V̂",
        "R̂_k(λ, α)",
        "‖R̂_k(λ, α)‖",
        "finite residual table does not prove",
    ] {
        assert!(
            ml.contains(exact),
            "CFT-34-006 ML diagnostic omits `{exact}`"
        );
    }
}

#[test]
fn chapter_34_realization_exercises_have_exact_prompts_distinct_compiler_proofs_and_links() {
    let markdown = chapter_markdown(34);
    let e05 = visible_markdown(exercise(&markdown, "CFT-34-E05").expect("CFT-34-E05 exercise"));
    for exact in [
        "(h : i →ᵇ ℂ)",
        "hh : ‖h‖ ≤ 1",
        "f : i →₂[μ] EuclideanVector n",
        "‖bcfMulL h f‖ ≤ ‖f‖",
        "exercise_05_solution",
        "bcfMulL_norm_le",
    ] {
        assert!(
            e05.contains(exact),
            "CFT-34-E05 prompt/solution omits `{exact}`"
        );
    }
    let e06 = visible_markdown(exercise(&markdown, "CFT-34-E06").expect("CFT-34-E06 exercise"));
    for exact in [
        "dilationDataOfParametricPolynomial",
        "data.T = euclideanOperator (polynomialEval q B)",
        "‖data.T‖ ≤ 2",
        "exercise_06_solution",
        "DilationData.norm_target_le_two",
    ] {
        assert!(
            e06.contains(exact),
            "CFT-34-E06 prompt/solution omits `{exact}`"
        );
    }
    assert!(markdown.contains(
        "[[formalization/lean/Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean|BoundaryMultiplier.lean]]"
    ));
    assert!(markdown.contains(
        "[[formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean|PerturbationLemma.lean]]"
    ));

    let lean = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter34.lean"),
    )
    .expect("Chapter 34 Lean module");
    let e05_lean = lean
        .split_once("theorem exercise_05_solution")
        .expect("CFT-34-E05 Lean theorem")
        .1
        .split_once("theorem exercise_06_solution")
        .expect("CFT-34-E06 follows E05")
        .0;
    assert!(e05_lean.contains("bcfMulL_norm_le"));
    assert!(!e05_lean.contains("bcfMulL_norm_le_one"));
    let e06_lean = lean
        .split_once("theorem exercise_06_solution")
        .expect("CFT-34-E06 Lean theorem")
        .1
        .split_once("end Exercises.Chapter34")
        .expect("Chapter 34 exercise namespace end")
        .0;
    assert!(e06_lean.contains("dilationDataOfParametricPolynomial"));
    assert!(e06_lean.contains("norm_target_le_two"));
    assert!(!e06_lean.contains("realization_norm_two"));
    assert!(
        !e06_lean.contains("norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary")
    );
    let chapter_33 = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter33.lean"),
    )
    .expect("Chapter 33 Lean module");
    let e33_lean = chapter_33
        .split_once("theorem exercise_06_solution")
        .expect("CFT-33-E06 Lean theorem")
        .1
        .split_once("end Exercises.Chapter33")
        .expect("Chapter 33 exercise namespace end")
        .0;
    assert_ne!(
        e06_lean, e33_lean,
        "Chapter 34 must not copy Chapter 33 E06"
    );
}

#[test]
fn chapter_34_handoff_records_completed_chapter_35_and_renders_it_in_atlas() {
    let markdown = chapter_markdown(34);
    let normalized_markdown = markdown.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(normalized_markdown.contains("Chapter 35 is complete as the next handoff"));
    assert!(markdown.contains("loristSchwenningerMainTheorem"));
    assert!(markdown.contains("harpFiniteHorizonMainTheorem"));
    assert!(!markdown.contains("Chapter 35 remains pending"));

    let corpus = read_json(&workspace_root().join("atlas/src/content/generated/corpus.json"));
    let chapter = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| document["title"] == "Chapter 34: The Lorist–Schwenninger realization")
        .expect("Chapter 34 Atlas document");
    let html = chapter["html"].as_str().expect("Chapter 34 Atlas HTML");
    let normalized_html = html.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(normalized_html.contains("Chapter 35 is complete as the next handoff"));
    assert!(html.contains("loristSchwenningerMainTheorem"));
    assert!(html.contains("harpFiniteHorizonMainTheorem"));
    assert!(!html.contains("Chapter 35 remains pending"));
}

#[test]
fn chapter_34_realization_publishes_exact_rows_exercise_receipts_and_rendered_algebra() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    for item_id in ["CFT-34-004", "CFT-34-005", "CFT-34-006"] {
        let spec = cft_spec(item_id);
        let row = array(&coverage, "items", "coverage")
            .iter()
            .find(|row| row["item_id"] == item_id)
            .unwrap_or_else(|| panic!("missing {item_id}"));
        assert_eq!(row["prose_proof_status"], "reconstructible");
        assert_eq!(row["lean_correspondence_status"], "exact");
        assert_eq!(row["formal_mode"], spec.completed_mode);
        assert_eq!(
            row["lean_declaration"]["underlying_declaration"],
            spec.provider_declaration
        );
    }

    let exercises = read_json(&contracts_root().join("exercises.json"));
    for exercise_id in ["CFT-34-E05", "CFT-34-E06"] {
        let spec = EXERCISE_SPECS
            .iter()
            .find(|spec| spec.exercise_id == exercise_id)
            .expect("realization exercise spec");
        let row = array(&exercises, "exercises", "exercises")
            .iter()
            .find(|row| row["exercise_id"] == exercise_id)
            .unwrap_or_else(|| panic!("missing {exercise_id}"));
        assert_eq!(row["lean_solution"]["declaration"], spec.solution);
        assert!(row["lean_solution"]["type_sha256"].as_str().is_some());
        assert_eq!(row["lean_solution"]["axioms"], json!(AXIOMS));
    }

    let corpus = read_json(&workspace_root().join("atlas/src/content/generated/corpus.json"));
    let chapter = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| document["title"] == "Chapter 34: The Lorist–Schwenninger realization")
        .expect("Chapter 34 Atlas document");
    let html = chapter["html"].as_str().expect("Chapter 34 Atlas HTML");
    for rendered in [
        "M_h^2 f = h • (h • f) = h^2 • f",
        "‖M_h‖ ≤ ρ &lt; 1",
        "data.norm_target_le_two",
        "polynomialEval q B = m⁻¹ • polynomialEval p B",
        "Ω_r := {z : ℂ | ‖z‖ &lt; r}",
        "r &gt; 1",
        "W(A_{0,2}) = {z : ℂ | ‖z‖ ≤ 1} ⊂ Ω_r",
        "Γ_r : ParametricConvexBoundary Ω_r",
        "hCauchy_r : HasParametricPolynomialCauchyFormula Γ_r μ_r A_{0,2}",
        "‖q_r(A_{0,2})‖ = 2 / r ≤ 2",
        "‖A_{0,2}‖ ≤ 2 * r",
        "r ↓ 1",
        "K is compact and nonempty",
        "S_p is nonempty and bounded above",
        "m is finite, nonnegative, and attained",
        "m := sSup S_p",
        "m &gt; 0",
        "m = 0",
        "p vanishes on K",
        "polynomial identity theorem",
        "six proof fields",
        "Target identification (definitional data)",
        "data.V_isometry",
        "data.Q_norm_le_one",
        "data.perturbation_eq",
        "data.bound_nonneg",
        "data.perturbation_norm_le",
        "data.commutes_with_target",
        "R̂_k(λ, α)",
        "exercise_05_solution",
        "exercise_06_solution",
    ] {
        assert!(
            html.contains(rendered),
            "Chapter 34 Atlas HTML omits `{rendered}`"
        );
    }
}

#[test]
fn chapter_35_comparison_phase_preserves_all_six_complete_cards() {
    let contract = wave3_contract();
    assert_eq!(
        active_phase(&contract),
        Wave3Phase::ThreeRouteComparisonComplete
    );

    let markdown = chapter_markdown(35);
    for index in 1..=6 {
        let completed = format!("CFT-35-{index:03}");
        let errors = complete_card_errors(&markdown, cft_spec(&completed));
        assert!(
            errors.is_empty(),
            "completed {completed} card differs from the consequences contract: {errors:#?}"
        );
    }

    let coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let errors = phase_metadata_errors(
        &coverage,
        &exercises,
        Wave3Phase::ThreeRouteComparisonComplete,
    );
    assert!(
        errors.is_empty(),
        "published metadata differs from the comparison phase: {errors:#?}"
    );
}

#[test]
fn chapter_35_consequences_main_theorem_exposes_normalization_and_both_limits() {
    let markdown = chapter_markdown(35);
    let section = card(&markdown, "CFT-35-001", 35).expect("CFT-35-001 card");
    let hypotheses = normalized_visible_markdown(
        card_field(section, "Hypothesis ledger").expect("CFT-35-001 hypotheses"),
    );
    for exact in [
        "n : Type*",
        "Fintype n",
        "DecidableEq n",
        "Nonempty n",
        "A : SquareMatrix n",
        "p : Polynomial ℂ",
    ] {
        assert!(hypotheses.contains(exact), "CFT-35-001 omits `{exact}`");
    }

    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-35-001 proof"));
    for exact in [
        "K := numericalRange A",
        "Ω_k := parallelOuterDomain K k",
        "M_k := maxPolynomialModulusOnSet (closure Ω_k) p",
        "q_k := M_k⁻¹ • p",
        "CFT-34-006",
        "simpleSpectrumApproximation A j",
        "j → ∞",
        "k → ∞",
        "maxPolynomialModulusOnNumericalRange A p",
        "‖polynomialEval p A‖ ≤ 2 * maxPolynomialModulusOnNumericalRange A p",
        "M_k = 0",
        "M_k > 0",
        "r ↓ 1",
        "outer approximation",
    ] {
        assert!(proof.contains(exact), "CFT-35-001 proof omits `{exact}`");
    }
    assert!(
        steps_in_order(&proof, cft_spec("CFT-35-001").proof_steps),
        "CFT-35-001 does not follow the provider proof order"
    );
}

#[test]
fn chapter_35_consequences_main_theorem_matches_the_compiled_norm_and_zero_branches() {
    let markdown = chapter_markdown(35);
    let section = card(&markdown, "CFT-35-001", 35).expect("CFT-35-001 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-35-001 proof"));
    for exact in [
        "hnormalizedOperator : ‖euclideanOperator (polynomialEval q_k B_j)‖ ≤ 2",
        "matrix_norm_eq_euclidean_operator_norm",
        "hnormalized : ‖polynomialEval q_k B_j‖ ≤ 2",
        "simpleDiagonalization_of_hasDistinctEigenvalues B_j",
        "hDiag.eigenvalues i ∈ matrixSpectrum B_j",
        "Polynomial.eval (hDiag.eigenvalues i) p = 0",
        "polynomialEval p B_j = 0",
        "Compiled Lean zero branch.",
    ] {
        assert!(
            proof.contains(exact),
            "CFT-35-001 compiled branch account omits `{exact}`"
        );
    }
    for false_bridge in [
        "The polynomial identity theorem gives p=0",
        "the operator-norm result is already the matrix-norm result",
    ] {
        assert!(
            !proof.contains(false_bridge),
            "CFT-35-001 retains the false bridge `{false_bridge}`"
        );
    }
}

#[test]
fn chapter_35_consequences_finite_matrix_adapter_states_the_index_equivalence() {
    let markdown = chapter_markdown(35);
    let section = card(&markdown, "CFT-35-002", 35).expect("CFT-35-002 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-35-002 proof"));
    for exact in [
        "FiniteMatrixMainTheoremStatement := ∀ (d : ℕ) [Nonempty (Fin d)], MainTheoremStatement (n := Fin d)",
        "SquareMatrix (Fin d) = Matrix (Fin d) (Fin d) ℂ",
        "EuclideanVector (Fin d) = Fin d → ℂ",
        "d × d complex matrix",
        "loristSchwenningerMainTheorem (n := Fin d)",
        "d = 0",
        "Nonempty (Fin d)",
    ] {
        assert!(proof.contains(exact), "CFT-35-002 proof omits `{exact}`");
    }
}

#[test]
fn chapter_35_consequences_rational_route_proves_each_limit_transition() {
    let markdown = chapter_markdown(35);
    let section = card(&markdown, "CFT-35-003", 35).expect("CFT-35-003 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-35-003 proof"));
    for exact in [
        "poles r ∩ numericalRange A = ∅",
        "q_N",
        "sup_{z∈W(A)} ‖q_N(z) - r(z)‖ → 0",
        "‖polynomialEval q_N A - rationalMatrixEval r A‖ → 0",
        "maxPolynomialModulusOnNumericalRange A q_N → maxRationalModulusOnNumericalRange A r",
        "‖rationalMatrixEval r A‖ ≤ 2 * maxRationalModulusOnNumericalRange A r",
        "polynomial approximants",
        "uniform scalar convergence",
        "matrix evaluation convergence",
        "maximum convergence",
        "limit inequality",
        "embedded polynomial",
    ] {
        assert!(proof.contains(exact), "CFT-35-003 proof omits `{exact}`");
    }
    assert!(
        steps_in_order(&proof, cft_spec("CFT-35-003").proof_steps),
        "CFT-35-003 does not expose the rational approximation order"
    );
    for forbidden in [
        "normalize the rational function and rescale",
        "the polynomial theorem applies directly to r",
    ] {
        assert!(
            !proof.contains(forbidden),
            "CFT-35-003 uses forbidden shortcut `{forbidden}`"
        );
    }
}

#[test]
fn chapter_35_consequences_hilbert_transport_keeps_the_finite_dimension_boundary() {
    let markdown = chapter_markdown(35);
    let section = card(&markdown, "CFT-35-004", 35).expect("CFT-35-004 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-35-004 proof"));
    for exact in [
        "H : Type*",
        "CompleteSpace H",
        "Nontrivial H",
        "A : H →L[ℂ] H",
        "p : Polynomial ℂ",
        "H_d := span ℂ",
        "P_d A P_d",
        "orthonormal basis",
        "Fin m",
        "Module.finrank ℂ H_d",
        "finite-dimensional range model",
        "transport back",
        "arbitrary complete nontrivial Hilbert spaces",
    ] {
        assert!(proof.contains(exact), "CFT-35-004 proof omits `{exact}`");
    }
}

#[test]
fn chapter_35_consequences_hilbert_transport_tracks_krylov_and_coordinate_dimensions() {
    let markdown = chapter_markdown(35);
    let section = card(&markdown, "CFT-35-004", 35).expect("CFT-35-004 card");
    let statement = normalized_visible_markdown(
        card_field(section, "Statement").expect("CFT-35-004 statement"),
    );
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-35-004 proof"));
    for exact in [
        "every complete nontrivial complex Hilbert space H",
        "d := p.natDegree",
        "H_d := span ℂ {A^k x | 0 ≤ k ≤ d}",
        "m := Module.finrank ℂ H_d",
        "m ≤ d + 1",
        "Fin m",
        "m := Module.finrank ℂ H",
        "bypass the Krylov compression",
        "arbitrary complete nontrivial Hilbert spaces",
    ] {
        assert!(
            statement.contains(exact) || proof.contains(exact),
            "CFT-35-004 dimension account omits `{exact}`"
        );
    }
    for false_dimension in [
        "taking d=dim H",
        "taking `d=dim H`",
        "Fin d matrix",
        "does not prove an infinite-dimensional extension",
    ] {
        assert!(
            !proof.contains(false_dimension),
            "CFT-35-004 retains the false dimension claim `{false_dimension}`"
        );
    }
}

#[test]
fn chapter_35_consequences_two_spectral_set_builds_all_three_components() {
    let markdown = chapter_markdown(35);
    let section = card(&markdown, "CFT-35-005", 35).expect("CFT-35-005 card");
    let statement = normalized_visible_markdown(
        card_field(section, "Statement").expect("CFT-35-005 statement"),
    );
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-35-005 proof"));
    for exact in [
        "IsCompact (closedOperatorNumericalRange A)",
        "spectrum ℂ A ⊆ closedOperatorNumericalRange A",
        "rational constant-two inequality",
        "ClosedOperatorNumericalRangeIsTwoSpectralSet A",
    ] {
        assert!(
            statement.contains(exact),
            "CFT-35-005 statement omits `{exact}`"
        );
    }
    for exact in [
        "closedOperatorNumericalRange_isCompact",
        "spectrum_subset_closedOperatorNumericalRange_of_mainTheorem",
        "hilbertSpaceRationalCrouzeix_of_mainTheorem",
        "compactness",
        "spectrum containment",
        "rational inequality",
        "if and only if",
    ] {
        assert!(proof.contains(exact), "CFT-35-005 proof omits `{exact}`");
    }
    assert!(
        steps_in_order(&proof, cft_spec("CFT-35-005").proof_steps),
        "CFT-35-005 does not assemble the predicate from its providers"
    );
}

#[test]
fn chapter_35_consequences_two_spectral_set_displays_the_exact_predicate_equivalence() {
    let markdown = chapter_markdown(35);
    let section = card(&markdown, "CFT-35-005", 35).expect("CFT-35-005 card");
    let statement = normalized_visible_markdown(
        card_field(section, "Statement").expect("CFT-35-005 statement"),
    );
    for exact in [
        "ClosedOperatorNumericalRangeIsTwoSpectralSet A ↔",
        "IsCompact (closedOperatorNumericalRange A) ∧",
        "spectrum ℂ A ⊆ closedOperatorNumericalRange A ∧",
        "∀ (r : RatFunc ℂ),",
        "RationalPoleFreeOn r (closedOperatorNumericalRange A) →",
        "‖operatorPolynomialEval r.num A * Ring.inverse (operatorPolynomialEval r.denom A)‖ ≤",
        "2 * supRationalModulusOnClosedOperatorNumericalRange A r",
        "The rational norm bound alone is not equivalent to the three-part predicate.",
    ] {
        assert!(
            statement.contains(exact),
            "CFT-35-005 exact predicate omits `{exact}`"
        );
    }
}

#[test]
fn chapter_35_consequences_synthesis_names_the_actual_theorem_strength() {
    let markdown = normalized_visible_markdown(&chapter_markdown(35));
    let synthesis = markdown
        .split("Synthesis and forward dependencies")
        .nth(1)
        .expect("Chapter 35 synthesis section");
    for exact in [
        "unnormalized arbitrary-polynomial theorem",
        "arbitrary complete nontrivial Hilbert spaces",
        "finite Krylov compression",
    ] {
        assert!(
            synthesis.contains(exact),
            "Chapter 35 synthesis omits `{exact}`"
        );
    }
    for false_summary in ["normalized matrix theorem", "finite-dimensional transport"] {
        assert!(
            !synthesis.contains(false_summary),
            "Chapter 35 synthesis retains `{false_summary}`"
        );
    }
}

#[test]
fn chapter_35_consequences_have_row_provenance_and_four_field_ml_diagnostics() {
    let markdown = chapter_markdown(35);
    let expected = [
        (
            "CFT-35-001",
            "LS source-derived terminal",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128",
        ),
        (
            "CFT-35-002",
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L20-L24",
        ),
        (
            "CFT-35-003",
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L28-L33",
        ),
        (
            "CFT-35-004",
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L37-L45",
        ),
        (
            "CFT-35-005",
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L55-L61",
        ),
    ];
    for (item_id, classification, locator) in expected {
        let section = card(&markdown, item_id, 35).expect("Chapter 35 consequence card");
        let history = normalized_visible_markdown(
            card_field(section, "Historical context").expect("history field"),
        );
        assert!(
            history.contains(classification),
            "{item_id} classification drift"
        );
        assert!(history.contains(locator), "{item_id} locator drift");

        let ml = normalized_visible_markdown(
            card_field(section, "ML analogy").expect("ML analogy field"),
        );
        for label in [
            "Mathematical object / ML counterpart.",
            "Exact transfer.",
            "Non-transfer.",
            "Diagnostic.",
        ] {
            assert!(ml.contains(label), "{item_id} ML field omits `{label}`");
        }
    }

    for exact in [
        "resolvent-filter certificate",
        "min_{z∈W(A)} distance(z, poles(r))",
        "finite-rank compression residual",
        "‖P_d A - A P_d‖",
        "compactness-spectrum-bound triple",
    ] {
        assert!(
            markdown.contains(exact),
            "Chapter 35 ML diagnostics omit `{exact}`"
        );
    }
}

#[test]
fn chapter_35_exercises_match_final_compiler_targets_without_terminal_shortcuts() {
    let markdown = chapter_markdown(35);
    for index in 1..=6 {
        let exercise_id = format!("CFT-35-E{index:02}");
        let section = visible_markdown(
            exercise(&markdown, &exercise_id).unwrap_or_else(|| panic!("missing {exercise_id}")),
        );
        assert!(
            section.contains(&format!("exercise_{index:02}_solution")),
            "{exercise_id} omits its Lean solution link"
        );
    }
    let lean = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean"),
    )
    .expect("Chapter 35 Lean module");
    for index in 1..=6 {
        assert!(
            lean.contains(&format!("theorem exercise_{index:02}_solution")),
            "Chapter 35 Lean omits exercise {index:02}"
        );
    }
    assert!(lean.contains("three_route_terminal_bundle"));

    let compiled = compile_active_dependency_receipt(Wave3Phase::ThreeRouteComparisonComplete)
        .expect("compile final dependency receipt");
    let receipt = json!({"declarations": compiled});
    let errors = exercise_receipt_errors(&receipt, Wave3Phase::ThreeRouteComparisonComplete);
    assert!(
        errors.is_empty(),
        "Chapter 35 exercise receipt drift: {errors:#?}"
    );
}

#[test]
fn chapter_35_atlas_renders_consequences_and_comparison() {
    let corpus = read_json(&workspace_root().join("atlas/src/content/generated/corpus.json"));
    let chapter = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| {
            document["title"] == "Chapter 35: Comparison, verification, and boundaries"
        })
        .expect("Chapter 35 Atlas document");
    let html = chapter["html"].as_str().expect("Chapter 35 Atlas HTML");
    for rendered in [
        "q_k := M_k⁻¹ • p",
        "matrix_norm_eq_euclidean_operator_norm",
        "simpleDiagonalization_of_hasDistinctEigenvalues B_j",
        "polynomialEval p B_j = 0",
        "simpleSpectrumApproximation A j",
        "SquareMatrix (Fin d) = Matrix (Fin d) (Fin d) ℂ",
        "matrix evaluation convergence",
        "m := Module.finrank ℂ H_d",
        "m ≤ d + 1",
        "ClosedOperatorNumericalRangeIsTwoSpectralSet A ↔",
        "RationalPoleFreeOn r (closedOperatorNumericalRange A)",
        "Ring.inverse (operatorPolynomialEval r.denom A)",
        "supRationalModulusOnClosedOperatorNumericalRange A r",
        "unnormalized arbitrary-polynomial theorem",
        "spectrum ℂ A ⊆ closedOperatorNumericalRange A",
        "exercise_05_solution",
    ] {
        assert!(
            html.contains(rendered),
            "Chapter 35 Atlas HTML omits `{rendered}`"
        );
    }
    assert!(html.contains("three_route_terminal_bundle"));
    assert!(html.contains("exercise_06_solution"));
}

#[test]
fn chapter_35_comparison_phase_completes_only_the_terminal_bundle_and_e06() {
    // The synthetic probe checks the provider-compatible Type target shape. The
    // publication receipt separately freezes the real declarations' hashes,
    // whose normalized instance-binder names are declaration-derived.
    assert_eq!(
        probe_row(BUNDLE_PROBE)["type_sha256"],
        "9e46ae67e3d0f760eee325ea6080a70211eb84dcdbb9209488302fa129e9c4c2"
    );
    assert_eq!(
        probe_row(EXERCISE_SPECS[11].probe_declaration)["type_sha256"],
        "33d5e65f524a71fed49c471239627a1abf75138de7b6a193041f77a47f8c4246"
    );
    let contract = wave3_contract();
    assert_eq!(
        active_phase(&contract),
        Wave3Phase::ThreeRouteComparisonComplete
    );

    let markdown = chapter_markdown(35);
    let comparison = cft_spec("CFT-35-006");
    let errors = complete_card_errors(&markdown, comparison);
    assert!(
        errors.is_empty(),
        "completed CFT-35-006 card differs from the comparison contract: {errors:#?}"
    );

    let coverage = read_json(&contracts_root().join("coverage.json"));
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let errors = phase_metadata_errors(
        &coverage,
        &exercises,
        Wave3Phase::ThreeRouteComparisonComplete,
    );
    assert!(
        errors.is_empty(),
        "published metadata differs from the comparison phase: {errors:#?}"
    );

    let compiled = compile_active_dependency_receipt(Wave3Phase::ThreeRouteComparisonComplete)
        .expect("compile final Wave 3 dependency receipt");
    let receipt = json!({"declarations": compiled});
    let errors = exercise_receipt_errors(&receipt, Wave3Phase::ThreeRouteComparisonComplete);
    assert!(
        errors.is_empty(),
        "final exercise receipt drift: {errors:#?}"
    );
    let errors = bundle_dependency_errors(&receipt, Wave3Phase::ThreeRouteComparisonComplete);
    assert!(errors.is_empty(), "final bundle receipt drift: {errors:#?}");

    let rows = receipt["declarations"]
        .as_array()
        .expect("final receipt rows");
    let bundle = rows
        .iter()
        .find(|row| row["name"] == THREE_ROUTE_BUNDLE)
        .expect("compiled three-route bundle");
    assert_eq!(bundle["kind"], "theorem");
    assert_eq!(bundle["axioms"], json!(AXIOMS));
    let direct = bundle["direct_dependencies"]
        .as_array()
        .expect("bundle direct dependencies")
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    for terminal in [JIN_TERMINAL, LS_TERMINAL, HARP_TERMINAL] {
        assert!(
            direct.contains(terminal),
            "bundle direct dependencies omit `{terminal}`"
        );
    }

    let mut single_provider = receipt.clone();
    single_provider["declarations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["name"] == THREE_ROUTE_BUNDLE)
        .unwrap()["direct_dependencies"] = json!([JIN_TERMINAL]);
    let errors =
        bundle_dependency_errors(&single_provider, Wave3Phase::ThreeRouteComparisonComplete);
    assert!(errors.iter().any(|error| error.contains(LS_TERMINAL)));
    assert!(errors.iter().any(|error| error.contains(HARP_TERMINAL)));
}

#[test]
fn chapter_35_comparison_exposes_eight_axes_history_and_ml_diagnostics() {
    let markdown = chapter_markdown(35);
    let section = card(&markdown, "CFT-35-006", 35).expect("CFT-35-006 card");
    let proof =
        normalized_visible_markdown(card_field(section, "Proof").expect("CFT-35-006 proof"));
    for axis in COMPARISON_AXES {
        assert!(proof.contains(axis), "comparison proof omits `{axis}`");
    }
    for exact in [
        "positive-real completion",
        "boundary dilation",
        "finite-horizon atomic certificates",
        "Jin",
        "Lorist–Schwenninger",
        "Harp",
        "comparison is not a proof dependency",
    ] {
        assert!(proof.contains(exact), "comparison proof omits `{exact}`");
    }

    let history = normalized_visible_markdown(
        card_field(section, "Historical context").expect("CFT-35-006 history"),
    );
    for exact in [
        "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910",
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128",
        "Jin source identity",
        "LS source identity",
        "review status",
        "reproduction status",
        "Harp-derived finite-horizon route",
        "does not assert priority or acceptance",
    ] {
        assert!(history.contains(exact), "comparison history omits `{exact}`");
    }

    let ml = normalized_visible_markdown(
        card_field(section, "ML analogy").expect("CFT-35-006 ML analogy"),
    );
    for exact in [
        "proof-architecture ablation",
        "positive-completion certificate",
        "boundary-dilation certificate",
        "finite-horizon atomic certificate",
        "smallest sampled eigenvalue",
        "moment residual",
        "A finite PSD or moment-residual check is only a diagnostic",
        "does not prove the exact infinite family",
    ] {
        assert!(ml.contains(exact), "comparison ML field omits `{exact}`");
    }
}

#[test]
fn chapter_35_comparison_e06_transports_all_three_route_witnesses() {
    let markdown = chapter_markdown(35);
    let section = normalized_visible_markdown(
        exercise(&markdown, "CFT-35-E06").expect("CFT-35-E06 exercise"),
    );
    for exact in [
        "jinConclusion lsConclusion harpConclusion : Prop",
        "hJinNormalized : jinConclusion = PolynomialCrouzeixBound A p",
        "hLsNormalized : lsConclusion = PolynomialCrouzeixBound A p",
        "hHarpNormalized : harpConclusion = PolynomialCrouzeixBound A p",
        "hJin : jinConclusion",
        "hLs : lsConclusion",
        "hHarp : harpConclusion",
        "PolynomialCrouzeixBound A p ∧ PolynomialCrouzeixBound A p ∧ PolynomialCrouzeixBound A p",
        "hJinNormalized ▸ hJin",
        "hLsNormalized ▸ hLs",
        "hHarpNormalized ▸ hHarp",
        "exercise_06_solution",
    ] {
        assert!(section.contains(exact), "CFT-35-E06 omits `{exact}`");
    }

    let lean = fs::read_to_string(
        workspace_root().join("formalization/lean/CrouzeixTextbook/Part06/Chapter35.lean"),
    )
    .expect("Chapter 35 Lean module");
    assert!(lean.contains("theorem exercise_06_solution"));
    for forbidden in [JIN_TERMINAL, LS_TERMINAL, HARP_TERMINAL, THREE_ROUTE_BUNDLE] {
        let exercise_source = lean
            .split("theorem exercise_06_solution")
            .nth(1)
            .expect("exercise 06 source");
        assert!(
            !exercise_source.contains(forbidden),
            "CFT-35-E06 source imports forbidden terminal `{forbidden}`"
        );
    }
}

#[test]
fn chapter_35_comparison_atlas_renders_terminal_evidence_and_code_links() {
    let corpus = read_json(&workspace_root().join("atlas/src/content/generated/corpus.json"));
    let chapter = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| {
            document["title"] == "Chapter 35: Comparison, verification, and boundaries"
        })
        .expect("Chapter 35 Atlas document");
    let html = chapter["html"].as_str().expect("Chapter 35 Atlas HTML");
    for rendered in [
        "Objects.",
        "Hypotheses.",
        "Shared trunk.",
        "Decisive mechanism.",
        "Approximation order.",
        "Conclusion.",
        "Provenance.",
        "Formal provider.",
        "three_route_terminal_bundle",
        "exercise_06_solution",
        "positive-completion certificate",
        "boundary-dilation certificate",
        "finite-horizon atomic certificate",
    ] {
        assert!(
            html.contains(rendered),
            "Chapter 35 Atlas HTML omits `{rendered}`"
        );
    }
}

#[test]
fn chapter_35_comparison_atlas_table_has_four_columns_and_eight_complete_axes() {
    let corpus = read_json(&workspace_root().join("atlas/src/content/generated/corpus.json"));
    let chapter = corpus["documents"]
        .as_array()
        .expect("Atlas documents")
        .iter()
        .find(|document| {
            document["title"] == "Chapter 35: Comparison, verification, and boundaries"
        })
        .expect("Chapter 35 Atlas document");
    let html = chapter["html"].as_str().expect("Chapter 35 Atlas HTML");
    let header = "<tr><th>Axis</th><th>Jin</th><th>Lorist–Schwenninger</th><th>Harp</th></tr>";
    let table_tail = html
        .split_once(header)
        .expect("rendered three-route table header")
        .1;
    let table_body = table_tail
        .split_once("</table>")
        .expect("rendered three-route table end")
        .0;
    let rows = table_body
        .split("<tr>")
        .skip(1)
        .map(|row| {
            let row = row
                .split_once("</tr>")
                .expect("complete rendered table row")
                .0;
            row.split("<td>")
                .skip(1)
                .map(|cell| {
                    cell.split_once("</td>")
                        .expect("complete rendered table cell")
                        .0
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    assert_eq!(
        rows.len(),
        8,
        "comparison table must render exactly eight axes"
    );
    assert!(rows.iter().all(|row| row.len() == 4));
    assert_eq!(
        rows.iter().map(|row| row[0]).collect::<Vec<_>>(),
        COMPARISON_AXES
    );
    let conclusion = rows
        .iter()
        .find(|row| row[0] == "Conclusion.")
        .expect("rendered conclusion row");
    assert!(conclusion[1].contains("MainTheoremStatement (n := n)"));
    assert!(conclusion[1].contains("‖p(A)‖ ≤ 2 max_{z∈W(A)} |p(z)|"));
    assert_eq!(conclusion[2], "The identical proposition and constant.");
    assert_eq!(conclusion[3], "The identical proposition and constant.");
}

fn dependency_closure(
    rows: &[Value],
    root: &str,
    visited: &mut BTreeSet<String>,
) -> BTreeSet<String> {
    if !visited.insert(root.to_owned()) {
        return BTreeSet::new();
    }
    let Some(row) = rows.iter().find(|row| row["name"].as_str() == Some(root)) else {
        return BTreeSet::new();
    };
    let mut closure = BTreeSet::new();
    for dependency in row["direct_dependencies"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        closure.insert(dependency.to_owned());
        closure.extend(dependency_closure(rows, dependency, visited));
    }
    closure
}

fn exercise_receipt_fixture(phase: Wave3Phase) -> Value {
    let declarations = EXERCISE_SPECS
        .iter()
        .filter(|spec| phase.rank() >= spec.completion_rank)
        .map(|spec| {
            let target = probe_row(spec.probe_declaration);
            let mut dependencies = target["direct_dependencies"]
                .as_array()
                .expect("exercise signature dependencies")
                .clone();
            dependencies.extend(
                spec.required_dependencies
                    .iter()
                    .filter(|name| !AUDITED_EXTERNAL_PROOF_DEPENDENCIES.contains(name))
                    .map(|name| json!(name)),
            );
            let external_dependencies = spec
                .required_dependencies
                .iter()
                .filter(|name| AUDITED_EXTERNAL_PROOF_DEPENDENCIES.contains(name))
                .collect::<Vec<_>>();
            json!({
                "name": spec.solution,
                "kind": "theorem",
                "normalized_type": target["normalized_type"],
                "type_sha256": target["type_sha256"],
                "axioms": AXIOMS,
                "direct_dependencies": dependencies,
                "audited_external_proof_dependencies": external_dependencies,
            })
        })
        .collect::<Vec<_>>();
    json!({"declarations": declarations})
}

fn exercise_receipt_errors(receipt: &Value, phase: Wave3Phase) -> Vec<String> {
    let mut errors = Vec::new();
    let rows = receipt["declarations"]
        .as_array()
        .expect("exercise receipt rows");
    for chapter in [34_u64, 35] {
        let namespace = format!("CrouzeixTextbook.Part06.Exercises.Chapter{chapter}.");
        let actual = rows
            .iter()
            .filter_map(|row| row["name"].as_str())
            .filter(|name| name.contains(&namespace))
            .collect::<BTreeSet<_>>();
        let expected = EXERCISE_SPECS
            .iter()
            .filter(|spec| spec.chapter == chapter && phase.rank() >= spec.completion_rank)
            .map(|spec| spec.solution)
            .collect::<BTreeSet<_>>();
        if actual != expected {
            errors.push(format!("Chapter{chapter} exercise namespace differs: expected {expected:?}, got {actual:?}"));
        }
        if phase.rank() >= if chapter == 34 { 3 } else { 5 } && actual.len() != 6 {
            errors.push(format!(
                "Chapter{chapter} must contain exactly six declarations"
            ));
        }
    }
    for spec in EXERCISE_SPECS {
        let matches = rows
            .iter()
            .filter(|row| row["name"].as_str() == Some(spec.solution))
            .collect::<Vec<_>>();
        if phase.rank() < spec.completion_rank {
            if !matches.is_empty() {
                errors.push(format!("{} appears before its phase", spec.exercise_id));
            }
            continue;
        }
        if matches.len() != 1 {
            errors.push(format!(
                "{} must have exactly one receipt row",
                spec.exercise_id
            ));
            continue;
        }
        let row = matches[0];
        let target = probe_row(spec.probe_declaration);
        if row["kind"] != "theorem" {
            errors.push(format!(
                "{} must elaborate as theorem, never alias",
                spec.exercise_id
            ));
        }
        if row["normalized_type"] != target["normalized_type"]
            || row["type_sha256"] != target["type_sha256"]
        {
            errors.push(format!(
                "{} differs from its compiler-elaborated target",
                spec.exercise_id
            ));
        }
        if row["axioms"] != json!(AXIOMS) {
            errors.push(format!(
                "{} compiled solution must have exactly the standard axiom set",
                spec.exercise_id
            ));
        }
        let signature_dependencies = target["direct_dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        let mut closure = BTreeSet::new();
        let mut visited = BTreeSet::from([spec.solution.to_owned()]);
        for dependency in row["direct_dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .filter(|dependency| !signature_dependencies.contains(dependency))
        {
            closure.insert(dependency.to_owned());
            closure.extend(dependency_closure(rows, dependency, &mut visited));
        }
        let external_dependencies = row["audited_external_proof_dependencies"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect::<BTreeSet<_>>();
        let expected_external_dependencies = spec
            .required_dependencies
            .iter()
            .copied()
            .filter(|dependency| AUDITED_EXTERNAL_PROOF_DEPENDENCIES.contains(dependency))
            .collect::<BTreeSet<_>>();
        if external_dependencies != expected_external_dependencies {
            errors.push(format!(
                "{} external proof dependencies differ: expected {expected_external_dependencies:?}, got {external_dependencies:?}",
                spec.exercise_id
            ));
        }
        for dependency in &external_dependencies {
            if !AUDITED_EXTERNAL_PROOF_DEPENDENCIES.contains(dependency) {
                errors.push(format!(
                    "{} reports non-allowlisted external proof dependency `{dependency}`",
                    spec.exercise_id
                ));
            }
        }
        for required in spec.required_dependencies {
            let found = if AUDITED_EXTERNAL_PROOF_DEPENDENCIES.contains(required) {
                external_dependencies.contains(required)
            } else {
                closure.contains(*required)
            };
            if !found {
                errors.push(format!(
                    "{} omits required dependency `{required}`",
                    spec.exercise_id
                ));
            }
        }
        let parent = cft_spec(spec.parent);
        for forbidden in [
            parent.public_declaration,
            parent.provider_declaration,
            completed_public(parent),
            JIN_TERMINAL,
            LS_TERMINAL,
            HARP_TERMINAL,
            THREE_ROUTE_BUNDLE,
        ] {
            if closure.contains(forbidden) {
                errors.push(format!(
                    "{} uses forbidden parent/terminal `{forbidden}`",
                    spec.exercise_id
                ));
            }
        }
        for dependency in &closure {
            if let Some(cft) = CFT_SPECS.iter().find(|candidate| {
                candidate.public_declaration == dependency
                    || candidate.provider_declaration == dependency
                    || completed_public(candidate) == dependency
            }) {
                if !spec.allowed_cfts.contains(&cft.item_id) {
                    errors.push(format!(
                        "{} uses unlisted CFT dependency `{dependency}`",
                        spec.exercise_id
                    ));
                }
            }
        }
    }
    errors
}

fn bundle_dependency_errors(receipt: &Value, phase: Wave3Phase) -> Vec<String> {
    let rows = receipt["declarations"]
        .as_array()
        .expect("bundle receipt rows");
    let matches = rows
        .iter()
        .filter(|row| row["name"].as_str() == Some(THREE_ROUTE_BUNDLE))
        .collect::<Vec<_>>();
    if phase != Wave3Phase::ThreeRouteComparisonComplete {
        return (!matches.is_empty())
            .then(|| "three-provider bundle must be absent before the final phase".to_owned())
            .into_iter()
            .collect();
    }
    if matches.len() != 1 {
        return vec!["three-provider bundle must have exactly one receipt row".to_owned()];
    }
    let closure = dependency_closure(rows, THREE_ROUTE_BUNDLE, &mut BTreeSet::new());
    [JIN_TERMINAL, LS_TERMINAL, HARP_TERMINAL]
        .into_iter()
        .filter(|dependency| !closure.contains(*dependency))
        .map(|dependency| format!("three-provider bundle omits `{dependency}`"))
        .collect()
}

#[test]
fn ls_route_contract_exercise_targets_elaborate_and_shortcut_mutations_fail() {
    let receipt = probe_receipt().as_array().expect("probe rows");
    for spec in EXERCISE_SPECS {
        let row = probe_row(spec.probe_declaration);
        let normalized = row["normalized_type"]
            .as_str()
            .expect("normalized exercise type");
        let expected_kind = if spec.exercise_id == "CFT-35-E06" {
            "theorem"
        } else {
            "axiom"
        };
        assert_eq!(row["kind"], expected_kind);
        assert_eq!(row["type_sha256"], receipt_type_hash(normalized));
        let parent = cft_spec(spec.parent);
        assert_ne!(
            row["type_sha256"],
            probe_row(parent.public_declaration)["type_sha256"],
            "{} target duplicates its parent public type",
            spec.exercise_id
        );
        assert_ne!(
            row["type_sha256"],
            probe_row(parent.provider_declaration)["type_sha256"],
            "{} target duplicates its parent provider type",
            spec.exercise_id
        );
        assert!(
            row["axioms"]
                .as_array()
                .expect("probe target axioms")
                .iter()
                .filter_map(Value::as_str)
                .all(|axiom| {
                    axiom.starts_with("CrouzeixTextbook.Wave3ContractProbe.")
                        || AXIOMS.contains(&axiom)
                }),
            "{} probe target depends on a project axiom",
            spec.exercise_id
        );
    }
    assert_eq!(
        EXERCISE_SPECS
            .iter()
            .map(|spec| spec.solution)
            .collect::<BTreeSet<_>>()
            .len(),
        12
    );

    for phase in Wave3Phase::ALL {
        let fixture = exercise_receipt_fixture(phase);
        let errors = exercise_receipt_errors(&fixture, phase);
        assert!(
            errors.is_empty(),
            "valid {} exercise fixture: {errors:#?}",
            phase.label()
        );
    }

    let phase = Wave3Phase::ThreeRouteComparisonComplete;
    let mut alias = exercise_receipt_fixture(phase);
    alias["declarations"].as_array_mut().unwrap()[0]["kind"] = json!("direct-alias");
    assert!(exercise_receipt_errors(&alias, phase)
        .iter()
        .any(|error| error.contains("never alias")));

    let mut project_axiom = exercise_receipt_fixture(phase);
    project_axiom["declarations"].as_array_mut().unwrap()[0]["axioms"] = json!([
        "Classical.choice",
        "CrouzeixTextbook.ProjectAxiom",
        "Quot.sound",
        "propext"
    ]);
    assert!(exercise_receipt_errors(&project_axiom, phase)
        .iter()
        .any(|error| error.contains("standard axiom set")));

    let mut wrong_shape = exercise_receipt_fixture(phase);
    let e06 = wrong_shape["declarations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["name"] == EXERCISE_SPECS[11].solution)
        .unwrap();
    *e06 = probe_row(TERMINAL_EXERCISE_MUTATION).clone();
    e06["name"] = json!(EXERCISE_SPECS[11].solution);
    e06["direct_dependencies"] = json!([]);
    assert!(exercise_receipt_errors(&wrong_shape, phase)
        .iter()
        .any(|error| error.contains("compiler-elaborated target")));

    let comparison_target = probe_row(EXERCISE_SPECS[11].probe_declaration);
    let comparison_type = comparison_target["normalized_type"]
        .as_str()
        .expect("comparison target type");
    assert_eq!(
        comparison_type.matches("PolynomialCrouzeixBound").count(),
        6,
        "CFT-35-E06 must expose three normalization equations and three normalized witnesses"
    );
    assert!(
        comparison_type.contains("And") || comparison_type.contains('∧'),
        "CFT-35-E06 must derive a non-reflexive combined conclusion"
    );
    let comparison_dependencies = comparison_target["direct_dependencies"]
        .as_array()
        .expect("comparison dependencies")
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    for terminal in [JIN_TERMINAL, LS_TERMINAL, HARP_TERMINAL] {
        assert!(
            !comparison_dependencies.contains(terminal),
            "CFT-35-E06 target imports forbidden terminal `{terminal}`"
        );
    }
    let old_pairing = compile_wave3_lean_fixture(
        r#"import CrouzeixTextbook.ExportReceipt
import CrouzeixHarp
namespace CrouzeixTextbook.Wave3NegativeProbe
open CrouzeixConjecture
example {n : Type*} [Fintype n] [DecidableEq n] [Nonempty n]
    (A : SquareMatrix n) (p : Polynomial ℂ)
    (jinConclusion lsConclusion harpConclusion : Prop)
    (hJinNormalized : jinConclusion = PolynomialCrouzeixBound A p)
    (hLsNormalized : lsConclusion = PolynomialCrouzeixBound A p)
    (hHarpNormalized : harpConclusion = PolynomialCrouzeixBound A p)
    (hJin : jinConclusion) (hLs : lsConclusion) (hHarp : harpConclusion) :
    PolynomialCrouzeixBound A p ∧ PolynomialCrouzeixBound A p ∧
      PolynomialCrouzeixBound A p := by
  exact ⟨hJin, hLs, hHarp⟩
end CrouzeixTextbook.Wave3NegativeProbe
"#,
    );
    assert!(
        !old_pairing.status.success(),
        "the old direct pairing unexpectedly inhabited the normalized comparison target"
    );
    let old_pairing_diagnostics = format!(
        "{}\n{}",
        String::from_utf8_lossy(&old_pairing.stdout),
        String::from_utf8_lossy(&old_pairing.stderr)
    );
    assert!(
        old_pairing_diagnostics.contains("type mismatch"),
        "old-pairing failure was not a type mismatch:\n{old_pairing_diagnostics}"
    );
    for mutation in [
        "CrouzeixTextbook.Wave3ContractProbe.mutation_cft_35_e06_reflexive",
        "CrouzeixTextbook.Wave3ContractProbe.mutation_cft_35_e06_trivial",
    ] {
        let mutation_row = probe_row(mutation);
        assert_ne!(
            comparison_target["type_sha256"], mutation_row["type_sha256"],
            "CFT-35-E06 collapsed to `{mutation}`"
        );
        let mut receipt = exercise_receipt_fixture(phase);
        let row = receipt["declarations"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|row| row["name"] == EXERCISE_SPECS[11].solution)
            .unwrap();
        row["normalized_type"] = mutation_row["normalized_type"].clone();
        row["type_sha256"] = mutation_row["type_sha256"].clone();
        assert!(exercise_receipt_errors(&receipt, phase)
            .iter()
            .any(|error| error.contains("compiler-elaborated target")));
    }

    let mut direct_parent = exercise_receipt_fixture(phase);
    direct_parent["declarations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["name"] == EXERCISE_SPECS[5].solution)
        .unwrap()["direct_dependencies"] = json!([cft_spec("CFT-34-006").provider_declaration]);
    assert!(exercise_receipt_errors(&direct_parent, phase)
        .iter()
        .any(|error| error.contains("forbidden parent/terminal")));

    let mut hidden_terminal = exercise_receipt_fixture(phase);
    hidden_terminal["declarations"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "name": "CrouzeixTextbook.Wave3TestHelper.hiddenTerminal",
            "kind": "theorem",
            "normalized_type": "helper",
            "type_sha256": "helper",
            "direct_dependencies": [HARP_TERMINAL],
        }));
    hidden_terminal["declarations"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["name"] == EXERCISE_SPECS[11].solution)
        .unwrap()["direct_dependencies"] =
        json!(["CrouzeixTextbook.Wave3TestHelper.hiddenTerminal"]);
    assert!(exercise_receipt_errors(&hidden_terminal, phase)
        .iter()
        .any(|error| error.contains(HARP_TERMINAL)));

    let mut private_extra = exercise_receipt_fixture(phase);
    private_extra["declarations"]
        .as_array_mut()
        .unwrap()
        .push(json!({
            "name": "_private.1.CrouzeixTextbook.Part06.Exercises.Chapter34.auditShortcut",
            "kind": "theorem",
            "normalized_type": "True",
            "type_sha256": "fake",
            "direct_dependencies": [],
        }));
    assert!(exercise_receipt_errors(&private_extra, phase)
        .iter()
        .any(|error| error.contains("namespace differs")));

    let mut bundle = json!({"declarations": [{
        "name": THREE_ROUTE_BUNDLE,
        "direct_dependencies": [JIN_TERMINAL, LS_TERMINAL, HARP_TERMINAL],
    }]});
    assert!(bundle_dependency_errors(&bundle, phase).is_empty());
    bundle["declarations"][0]["direct_dependencies"] = json!([JIN_TERMINAL, LS_TERMINAL]);
    assert!(bundle_dependency_errors(&bundle, phase)
        .iter()
        .any(|error| error.contains(HARP_TERMINAL)));
    assert!(
        receipt.len() >= 26,
        "probe must include targets and providers"
    );
}

#[test]
fn ls_route_contract_e06_distinguishes_type_construction_from_proof_dependencies() {
    let spec = EXERCISE_SPECS
        .iter()
        .find(|spec| spec.exercise_id == "CFT-34-E06")
        .expect("CFT-34-E06 contract");
    let target = probe_row(spec.probe_declaration);
    assert!(
        target["normalized_type"]
            .as_str()
            .expect("CFT-34-E06 normalized type")
            .contains(CHAPTER34_E06_CONSTRUCTOR),
        "CFT-34-E06 must instantiate the concrete dilation provider in its compiled statement"
    );
    let exercises = read_json(&contracts_root().join("exercises.json"));
    let canonical = array(&exercises, "exercises", "exercises")
        .iter()
        .find(|row| row["exercise_id"] == spec.exercise_id)
        .expect("canonical CFT-34-E06 exercise row");
    assert_eq!(
        canonical["lean_solution"]["type_sha256"],
        json!(spec.canonical_type_sha256.expect("CFT-34-E06 type hash"))
    );
    let signature_dependencies = target["direct_dependencies"]
        .as_array()
        .expect("CFT-34-E06 signature dependencies")
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    assert!(signature_dependencies.contains(CHAPTER34_E06_CONSTRUCTOR));

    let compiled = active_dependency_receipt()
        .as_array()
        .expect("active dependency receipt");
    let solution = compiled
        .iter()
        .find(|row| row["name"].as_str() == Some(spec.solution))
        .expect("compiled CFT-34-E06 solution");
    let direct_dependencies = solution["direct_dependencies"]
        .as_array()
        .expect("CFT-34-E06 direct dependencies")
        .iter()
        .filter_map(Value::as_str)
        .collect::<BTreeSet<_>>();
    assert!(direct_dependencies.contains(CHAPTER34_E06_CONSTRUCTOR));
    let proof_dependencies = direct_dependencies
        .difference(&signature_dependencies)
        .copied()
        .collect::<BTreeSet<_>>();
    assert!(proof_dependencies.contains(CHAPTER34_E06_ENDPOINT));
    assert!(
        !proof_dependencies.contains(CHAPTER34_E06_CONSTRUCTOR),
        "a constructor used only in the theorem type must not be invented as a proof-body dependency"
    );
}

fn graph_reaches(
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
                .any(|node| graph_reaches(graph, node, target, visited))
        })
}

#[test]
fn ls_route_contract_pedagogical_graph_keeps_ls_independent_until_cft_35_006() {
    let coverage = read_json(&contracts_root().join("coverage.json"));
    let graph = array(&coverage, "items", "coverage")
        .iter()
        .map(|row| {
            let item_id = string(row, "item_id", "coverage row").to_owned();
            let prerequisites = array(row, "pedagogical_prerequisites", &item_id)
                .iter()
                .map(|value| value.as_str().expect("prerequisite string").to_owned())
                .collect::<BTreeSet<_>>();
            (item_id, prerequisites)
        })
        .collect::<BTreeMap<_, _>>();

    assert!(graph_reaches(
        &graph,
        "CFT-34-006",
        "CFT-33-006",
        &mut BTreeSet::new()
    ));
    for index in 1..=6 {
        let ls_node = format!("CFT-34-{index:03}");
        for chapter in 30..=32 {
            for terminal_index in 1..=6 {
                let jin_node = format!("CFT-{chapter:02}-{terminal_index:03}");
                assert!(
                    !graph_reaches(&graph, &ls_node, &jin_node, &mut BTreeSet::new()),
                    "{ls_node} reaches Jin node {jin_node}"
                );
            }
        }
    }
    for index in 1..=5 {
        let node = format!("CFT-35-{index:03}");
        for chapter in 30..=32 {
            for jin_index in 1..=6 {
                let jin = format!("CFT-{chapter:02}-{jin_index:03}");
                assert!(
                    !graph_reaches(&graph, &node, &jin, &mut BTreeSet::new()),
                    "comparison leaked into {node} through {jin}"
                );
            }
        }
    }
    assert!(graph_reaches(
        &graph,
        "CFT-35-006",
        "CFT-32-006",
        &mut BTreeSet::new()
    ));

    let contract = wave3_contract()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(contract.contains("CFT-35-006 is the first and only three-route comparison row"));
    assert!(contract.contains("Chapter 34 must never reach CFT-30-001 through CFT-32-006"));
}

fn run_preflight_all() -> Value {
    let output = Command::new("python3")
        .arg("labs/crouzeix_proof_reproduction/proof_evidence.py")
        .arg("preflight")
        .arg("--route")
        .arg("all")
        .current_dir(workspace_root())
        .output()
        .expect("run three-route preflight");
    assert!(
        output.status.success(),
        "preflight failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("three-route preflight JSON")
}

fn module_roster_sha256(modules: &[Value]) -> String {
    let joined = modules
        .iter()
        .map(|module| module.as_str().expect("module string"))
        .collect::<Vec<_>>()
        .join("\n");
    format!("{:x}", Sha256::digest(joined.as_bytes()))
}

fn run_structural_closure(lean_root: &Path, route: &str) -> Output {
    let script = r#"
import sys
from pathlib import Path
repo = Path(sys.argv[1])
sys.path.insert(0, str(repo / 'labs' / 'crouzeix_proof_reproduction'))
import proof_evidence
try:
    closure = proof_evidence.gather_route_closure(
        Path(sys.argv[2]), proof_evidence.ROUTE_POLICIES[sys.argv[3]],
        cache_identity='wave3-contract-test',
        toolchain=proof_evidence.PINNED_TOOLCHAIN)
except proof_evidence.PreflightError as error:
    print(error.reason, file=sys.stderr)
    raise SystemExit(7)
print('\n'.join(closure))
"#;
    Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(workspace_root())
        .arg(lean_root)
        .arg(route)
        .output()
        .expect("run canonical route structural parser")
}

fn write_test_module(root: &Path, module: &str, source: &str) {
    let path = root.join(format!("{}.lean", module.replace('.', "/")));
    fs::create_dir_all(path.parent().expect("module parent")).expect("module directory");
    fs::write(path, source).expect("test Lean module");
}

#[test]
fn ls_route_contract_route_closures_match_the_documented_structural_receipts() {
    let preflight = run_preflight_all();
    let expected = BTreeMap::from([
        (
            "jin",
            (
                66_u64,
                "35d570c92d791bc0162b8f9b19b6450e52c4e80226d13fb885efc1874b91dca5",
            ),
        ),
        (
            "lorist-schwenninger",
            (
                56_u64,
                "63c7b93484759a68c41bb30e86ac7c2957cfccbef37933bc059dee3baaa46f53",
            ),
        ),
        (
            "harp",
            (
                59_u64,
                "db41d5e011d149397a5fb6bb6b33c2a7e70d3ec07a43e5193a7159605808f903",
            ),
        ),
    ]);
    let routes = preflight["routes"].as_array().expect("preflight routes");
    assert_eq!(routes.len(), 3);
    let contract = wave3_contract();
    for route in routes {
        let route_id = route["route_id"].as_str().expect("route id");
        let modules = route["local_modules"].as_array().expect("route modules");
        let (count, digest) = expected[route_id];
        assert_eq!(modules.len() as u64, count, "{route_id} closure count");
        assert_eq!(
            module_roster_sha256(modules),
            digest,
            "{route_id} closure digest"
        );
        assert_eq!(route["status"], "ready");
        assert!(
            contract.contains(&format!("`{route_id}` | {count} | `{digest}`")),
            "contract omits {route_id} structural receipt"
        );
    }

    let temp = tempfile::tempdir().expect("provider mutation root");
    write_test_module(
        temp.path(),
        "CrouzeixJin",
        "import Crouzeix.LoristSchwenninger.Consequences\n",
    );
    let direct = run_structural_closure(temp.path(), "jin");
    assert_eq!(direct.status.code(), Some(7));
    assert!(String::from_utf8_lossy(&direct.stderr)
        .contains("rejected provider import Crouzeix.LoristSchwenninger.Consequences"));

    let temp = tempfile::tempdir().expect("transitive provider mutation root");
    write_test_module(temp.path(), "CrouzeixJin", "import Crouzeix.Jin.Terminal\n");
    write_test_module(
        temp.path(),
        "Crouzeix.Jin.Terminal",
        "/- import Crouzeix.Harp.Consequences -/\nimport Crouzeix.Jin.Hidden\n",
    );
    write_test_module(
        temp.path(),
        "Crouzeix.Jin.Hidden",
        "import Crouzeix.LoristSchwenninger.Consequences\n",
    );
    let transitive = run_structural_closure(temp.path(), "jin");
    assert_eq!(transitive.status.code(), Some(7));
    assert!(String::from_utf8_lossy(&transitive.stderr)
        .contains("rejected provider import Crouzeix.LoristSchwenninger.Consequences"));

    let temp = tempfile::tempdir().expect("comment parser fixture root");
    write_test_module(temp.path(), "CrouzeixJin",
        "/- outer /- import Crouzeix.Harp.Consequences -/ comment -/\nimport Crouzeix.Jin.Terminal\n");
    write_test_module(
        temp.path(),
        "Crouzeix.Jin.Terminal",
        "-- import Crouzeix.Harp.Consequences\n",
    );
    let comment_only = run_structural_closure(temp.path(), "jin");
    assert!(
        comment_only.status.success(),
        "comments were mistaken for imports: {}",
        String::from_utf8_lossy(&comment_only.stderr)
    );
}

#[test]
fn ls_route_contract_binds_exact_source_provenance_and_compiler_valid_code_links() {
    let contract = wave3_contract();
    for locator in [
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99",
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L74-L90",
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L90-L98",
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124",
        "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128",
        "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:preprint/the_numerical_range_is_a_2_spectral_set_v4.tex#L854-L910",
        "formalization/lean/Crouzeix/Harp/MainTheorem.lean#L25",
        "formalization/lean/Crouzeix/Harp/Consequences.lean#L22-L61",
        "evidence/crouzeix_conjecture/source_manifest.tsv#L25-L28",
        "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json#L1",
    ] {
        assert!(contract.contains(locator), "missing exact provenance locator `{locator}`");
    }
    for spec in CFT_SPECS {
        assert!(
            workspace_root().join(spec.public_file).is_file(),
            "missing public file {}",
            spec.public_file
        );
        assert!(
            workspace_root().join(spec.provider_file).is_file(),
            "missing provider file {}",
            spec.provider_file
        );
        let public = probe_row(spec.public_declaration);
        let provider = probe_row(spec.provider_declaration);
        assert_eq!(
            public["source_path"], spec.public_file,
            "{} public compiler path",
            spec.item_id
        );
        assert_eq!(
            provider["source_path"], spec.provider_file,
            "{} provider compiler path",
            spec.item_id
        );
        for exact in [
            spec.item_id,
            spec.public_declaration,
            spec.provider_declaration,
            spec.public_file,
            spec.provider_file,
            spec.baseline_type_sha256,
            spec.compiler_public_type_sha256,
            provider_hash(&spec),
        ] {
            assert!(
                contract.contains(exact),
                "contract does not bind {} to `{exact}`",
                spec.item_id
            );
        }
    }
    let exercise_section = contract
        .split_once("## Exercise signatures and proof boundaries")
        .expect("exercise-contract section")
        .1;
    for exercise in EXERCISE_SPECS {
        let marker = format!("| {} |", exercise.exercise_id);
        let row = exercise_section
            .split_once(&marker)
            .unwrap_or_else(|| panic!("contract omits {} exercise row", exercise.exercise_id))
            .1
            .lines()
            .next()
            .expect("exercise contract row");
        for exact in [
            exercise.parent,
            exercise.probe_declaration,
            exercise.solution,
        ] {
            assert!(
                row.contains(exact),
                "contract omits {} binding `{exact}`",
                exercise.exercise_id
            );
        }
        for exact in exercise
            .allowed_cfts
            .iter()
            .chain(exercise.required_dependencies)
        {
            assert!(
                row.contains(exact),
                "{} row omits allowed/required dependency `{exact}`",
                exercise.exercise_id
            );
        }
    }

    let provenance_graph = read_json(&workspace_root().join(
        "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/source-graph.json",
    ));
    assert_eq!(provenance_graph["source_identity"], "arxiv:2608.03841v1");
    assert_eq!(
        provenance_graph["nodes"]
            .as_array()
            .expect("LS graph nodes")
            .len(),
        6
    );
    let source_manifest = fs::read_to_string(
        workspace_root().join("evidence/crouzeix_conjecture/source_manifest.tsv"),
    )
    .expect("source manifest");
    assert!(source_manifest
        .contains("20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a"));
}

#[test]
fn ls_route_contract_binds_row_specific_provenance_without_global_harp_history() {
    let expected = [
        (
            "CFT-34-001",
            "LS source-derived",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124",
        ),
        (
            "CFT-34-002",
            "LS source-derived",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124",
        ),
        (
            "CFT-34-003",
            "LS source-derived",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124",
        ),
        (
            "CFT-34-004",
            "LS source-derived",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124",
        ),
        (
            "CFT-34-005",
            "LS source-derived",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124",
        ),
        (
            "CFT-34-006",
            "LS source-derived",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124",
        ),
        (
            "CFT-35-001",
            "LS source-derived terminal",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128",
        ),
        (
            "CFT-35-002",
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L20-L24",
        ),
        (
            "CFT-35-003",
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L28-L33",
        ),
        (
            "CFT-35-004",
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L37-L45",
        ),
        (
            "CFT-35-005",
            "provider-clean consequence",
            "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean#L55-L61",
        ),
        (
            "CFT-35-006",
            "comparison-only; Harp-derived component",
            "formalization/lean/Crouzeix/Harp/MainTheorem.lean#L25-L172",
        ),
    ];
    let terminal_source_rows = expected
        .iter()
        .filter_map(|(item_id, _, locator)| {
            (*locator == "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128")
                .then_some(*item_id)
        })
        .collect::<Vec<_>>();
    assert_eq!(
        terminal_source_rows,
        ["CFT-35-001"],
        "the LS terminal theorem source span belongs exclusively to CFT-35-001"
    );
    let contract = wave3_contract();
    let section = contract
        .split_once("## Row-specific provenance")
        .expect("row-specific provenance section")
        .1;
    for (item_id, classification, locator) in expected {
        let marker = format!("| {item_id} |");
        let row = section
            .split_once(&marker)
            .unwrap_or_else(|| panic!("provenance table omits {item_id}"))
            .1
            .lines()
            .next()
            .expect("provenance row");
        assert!(
            row.contains(classification),
            "{item_id} classification drift"
        );
        assert!(row.contains(locator), "{item_id} locator drift");
        let fixture = future_card_fixture(cft_spec(item_id));
        let history = card_field(&fixture, "Historical context").expect("future history field");
        assert!(history.contains(classification));
        assert!(history.contains(locator));
        for token in [classification, locator] {
            let deleted = fixture.replacen(token, "", 1);
            assert!(
                complete_card_errors(&deleted, cft_spec(item_id))
                    .iter()
                    .any(|error| error.contains("row-specific provenance")),
                "{item_id} accepted deleted provenance `{token}`"
            );
        }
        if item_id != "CFT-35-006" {
            assert!(
                !history.contains("Harp-derived"),
                "{item_id} received a global Harp-derived history label"
            );
        }
    }
}

#[test]
fn ls_route_contract_public_map_is_exact_compiler_evidence_in_every_phase() {
    for phase in Wave3Phase::ALL {
        let fixture = public_receipt_fixture(phase);
        let errors = public_receipt_errors(&fixture, phase);
        assert!(
            errors.is_empty(),
            "valid {} public receipt: {errors:#?}",
            phase.label()
        );
    }

    let phase = Wave3Phase::ThreeRouteComparisonComplete;
    let pristine = public_receipt_fixture(phase);
    let first_name = pristine["declarations"][0]["name"]
        .as_str()
        .expect("public fixture name")
        .to_owned();
    for (field, replacement) in [
        ("source_path", json!("formalization/lean/Wrong.lean")),
        ("normalized_type", json!("False")),
        ("type_sha256", json!("00")),
        ("kind", json!("axiom")),
        ("axioms", json!(["CrouzeixTextbook.ProjectAxiom"])),
        ("formal_mode", json!("checkpoint")),
        ("provider_dependency", Value::Null),
        ("direct_dependencies", json!([])),
    ] {
        let mut drift = pristine.clone();
        drift["declarations"].as_array_mut().unwrap()[0][field] = replacement;
        assert!(
            public_receipt_errors(&drift, phase)
                .iter()
                .any(|error| error.contains(&first_name) && error.contains(field)),
            "public-map drift in {field} was accepted"
        );
    }

    let mut duplicate = pristine;
    let duplicate_row = duplicate["declarations"][0].clone();
    duplicate["declarations"]
        .as_array_mut()
        .unwrap()
        .push(duplicate_row);
    assert!(public_receipt_errors(&duplicate, phase)
        .iter()
        .any(|error| error.contains("exactly one compiler public row")));

    let mut extra_dependency = public_receipt_fixture(phase);
    extra_dependency["declarations"].as_array_mut().unwrap()[0]["direct_dependencies"] =
        json!([CFT_SPECS[0].provider_declaration, JIN_TERMINAL]);
    assert!(public_receipt_errors(&extra_dependency, phase)
        .iter()
        .any(|error| error.contains("direct_dependencies")));
}

#[test]
fn ls_route_contract_preserves_chapter35_compatibility_names_and_reserves_comparison() {
    for phase in Wave3Phase::ALL {
        let receipt = public_receipt_fixture(phase);
        assert!(public_receipt_errors(&receipt, phase).is_empty());
        let rows = receipt["declarations"].as_array().unwrap();
        for spec in CFT_SPECS.iter().filter(|spec| spec.chapter == 35) {
            let matches = rows
                .iter()
                .filter(|row| row["name"] == spec.public_declaration)
                .collect::<Vec<_>>();
            assert_eq!(matches.len(), 1, "{} compatibility row", spec.item_id);
            assert_eq!(matches[0]["kind"], "direct-alias");
            assert!(wave3_contract().contains(spec.public_declaration));
        }
        let bundles = rows
            .iter()
            .filter(|row| row["name"] == THREE_ROUTE_BUNDLE)
            .collect::<Vec<_>>();
        if phase == Wave3Phase::ThreeRouteComparisonComplete {
            assert_eq!(bundles.len(), 1);
            assert_eq!(bundles[0]["kind"], "theorem");
        } else {
            assert!(bundles.is_empty(), "{} admitted bundle", phase.label());
        }
    }
    let contract = wave3_contract()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        contract.contains("The checked bundle body must depend on all three terminal declarations")
    );
}
