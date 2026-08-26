from __future__ import annotations

import copy
import importlib
import importlib.util
import inspect
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

from labs.crouzeix_proof_reproduction import provider_independence, route_validation


REPO = Path(__file__).resolve().parents[3]
SCRIPT = REPO / "labs/crouzeix_proof_reproduction/proof_evidence.py"
MANIFEST_PATH = Path(
    "labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json"
)
TYPE_ROOT = MANIFEST_PATH.parent / "declaration-types"
EXPECTED_CLOSURE_SHA256 = (
    "d0901748dd51b8850603918f528b2da2feb7b3fe653c7f5be77423f40b5ab137"
)
LS_MANIFEST_PATH = (
    "labs/crouzeix_proof_reproduction/formal_targets/"
    "lorist-schwenninger/route-manifest.json"
)
LS_MANIFEST_SHA256 = (
    "c8c4aa731781015a356b0cc43f080df36a9d489a0b60700fea8e331c5993fbaa"
)
LS_RECEIPT_PATH = (
    "evidence/crouzeix_conjecture/routes/lorist-schwenninger/receipt.json"
)
LS_RECEIPT_SHA256 = (
    "f672bb002c9d7ebc516683d61ba87b2c1ff2bed891daac0e6781aeb50cc7a01b"
)
HARP_REVIEW_SHA256 = (
    "60e9bbc25979c8b2e4f38e0d03fa3fc68c8ff077949f94e81a024a56dd1957d6"
)
HARP_RECEIPT_SHA256 = (
    "5bc448e9a27968d6b73c7448fe72e41b52e2c1ec82d7b7ddf898dcf1aaa79883"
)

EXPECTED_CONCEPTUAL_NODES = {
    "harp-positive-cubature": (
        "CrouzeixConjecture.Harp.exists_positive_cubature_moments_through_succ",
        "Crouzeix/Harp/PositiveCubature.lean",
        "load-bearing",
        "derived",
        "derived-extraction",
    ),
    "harp-finite-measure-cubature": (
        "CrouzeixConjecture.Harp.exists_positive_cubature_finite_measure",
        "Crouzeix/Harp/FiniteMeasureCubature.lean",
        "load-bearing",
        "derived",
        "derived-extraction",
    ),
    "harp-finite-atomic-matrix-moments": (
        "CrouzeixConjecture.Harp.exists_positive_matrix_moment_cubature",
        "Crouzeix/Harp/FiniteAtomicDilation.lean",
        "load-bearing",
        "derived",
        "derived-extraction",
    ),
    "harp-finite-horizon-dilation-interface": (
        "CrouzeixConjecture.Harp.FiniteHorizonDilationData",
        "Crouzeix/Harp/FiniteHorizonDilation.lean",
        "load-bearing",
        "derived",
        "derived-extraction",
    ),
    "harp-counting-l2-witness-dimension-bound": (
        "CrouzeixConjecture.Harp.finiteAtomicL2DilationWitness",
        "Crouzeix/Harp/FiniteAtomicL2Dilation.lean",
        "load-bearing",
        "derived",
        "derived-extraction",
    ),
    "harp-finite-horizon-recurrence": (
        "CrouzeixConjecture.Harp.finite_weighted_inequalities_to_limit_lower_bound",
        "Crouzeix/Harp/FiniteHorizonRecurrence.lean",
        "load-bearing",
        "derived",
        "derived-extraction",
    ),
    "harp-operator-recurrence": (
        "CrouzeixConjecture.Harp.FiniteHorizonDilationData.equation_three_finite_lower_bound",
        "Crouzeix/Harp/FiniteHorizonOperatorRecurrence.lean",
        "load-bearing",
        "derived",
        "derived-extraction",
    ),
    "harp-perturbation-endpoint": (
        "CrouzeixConjecture.Harp.norm_target_le_two_of_finiteHorizonDilationData",
        "Crouzeix/Harp/FiniteHorizonPerturbation.lean",
        "load-bearing",
        "derived",
        "derived-extraction",
    ),
    "harp-double-layer-application": (
        "CrouzeixConjecture.two_smul_boundaryPhi_parametric_eq",
        "CrouzeixConjecture/ParametricDoubleLayerIdentity.lean",
        "load-bearing",
        "shared-foundation",
        "shared-foundation",
    ),
    "harp-inner-limit": (
        "CrouzeixConjecture.norm_polynomialEval_le_of_tendsto",
        "CrouzeixConjecture/Limiting.lean",
        "load-bearing",
        "shared-foundation",
        "shared-foundation",
    ),
    "harp-outer-limit": (
        "CrouzeixConjecture.tendsto_maxPolynomialModulusOnSet_of_outerApproximation",
        "CrouzeixConjecture/OuterApproximationLimit.lean",
        "load-bearing",
        "shared-foundation",
        "shared-foundation",
    ),
    "harp-terminal-theorem": (
        "CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem",
        "Crouzeix/Harp/MainTheorem.lean",
        "terminal",
        "derived",
        "derived-extraction",
    ),
    "harp-closed-range-consequence": (
        "CrouzeixConjecture.harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet",
        "Crouzeix/Harp/Consequences.lean",
        "consequence",
        "derived",
        "derived-extraction",
    ),
}

EXPECTED_REUSE_NODES = {
    "harp-reuse-boundary-embedding": (
        "CrouzeixConjecture.LoristSchwenninger.boundaryEmbedding",
        "Crouzeix/LoristSchwenninger/BoundaryEmbedding.lean",
    ),
    "harp-reuse-boundary-multiplier": (
        "CrouzeixConjecture.LoristSchwenninger.bcfMulL",
        "Crouzeix/LoristSchwenninger/BoundaryMultiplier.lean",
    ),
    "harp-reuse-boundary-square-root": (
        "CrouzeixConjecture.LoristSchwenninger.boundarySquareRoot",
        "Crouzeix/LoristSchwenninger/BoundarySquareRoot.lean",
    ),
    "harp-reuse-companion-algebra": (
        "CrouzeixConjecture.LoristSchwenninger.parametricBoundaryCompanion_commute_of_mem_generatedAlgebra",
        "Crouzeix/LoristSchwenninger/CompanionAlgebra.lean",
    ),
    "harp-reuse-completed-square": (
        "CrouzeixConjecture.LoristSchwenninger.pre_square_lower_bound",
        "Crouzeix/LoristSchwenninger/CompletedSquare.lean",
    ),
    "harp-reuse-compression-moments": (
        "CrouzeixConjecture.LoristSchwenninger.boundaryEmbedding_adjoint_comp_bcfMulL_pow_comp_boundaryEmbedding",
        "Crouzeix/LoristSchwenninger/CompressionMoments.lean",
    ),
    "harp-reuse-dilation": (
        "CrouzeixConjecture.LoristSchwenninger.compressedAdjointPower",
        "Crouzeix/LoristSchwenninger/Dilation.lean",
    ),
    "harp-reuse-norm-attainment": (
        "CrouzeixConjecture.LoristSchwenninger.exists_unit_norm_attaining_and_adjoint_apply",
        "Crouzeix/LoristSchwenninger/NormAttainment.lean",
    ),
    "harp-reuse-polynomial-power-cauchy": (
        "CrouzeixConjecture.LoristSchwenninger.parametricPolynomialPowerCauchy",
        "Crouzeix/LoristSchwenninger/PolynomialPowerCauchy.lean",
    ),
    "harp-reuse-recurrence": (
        "CrouzeixConjecture.LoristSchwenninger.recurrence_finite_iteration_of_constant_error",
        "Crouzeix/LoristSchwenninger/Recurrence.lean",
    ),
    "harp-reuse-scalar": (
        "CrouzeixConjecture.LoristSchwenninger.scalar_endpoint_le_two",
        "Crouzeix/LoristSchwenninger/Scalar.lean",
    ),
}

EXPECTED_ORDER = (
    "harp-reuse-boundary-square-root",
    "harp-reuse-boundary-embedding",
    "harp-reuse-boundary-multiplier",
    "harp-reuse-compression-moments",
    "harp-reuse-dilation",
    "harp-reuse-norm-attainment",
    "harp-reuse-completed-square",
    "harp-reuse-companion-algebra",
    "harp-reuse-polynomial-power-cauchy",
    "harp-reuse-recurrence",
    "harp-reuse-scalar",
    "harp-positive-cubature",
    "harp-finite-measure-cubature",
    "harp-finite-horizon-dilation-interface",
    "harp-finite-atomic-matrix-moments",
    "harp-double-layer-application",
    "harp-counting-l2-witness-dimension-bound",
    "harp-finite-horizon-recurrence",
    "harp-operator-recurrence",
    "harp-perturbation-endpoint",
    "harp-inner-limit",
    "harp-outer-limit",
    "harp-terminal-theorem",
    "harp-closed-range-consequence",
)

EXPECTED_DEPENDENCIES = {
    "harp-reuse-boundary-square-root": [],
    "harp-reuse-boundary-embedding": ["harp-reuse-boundary-square-root"],
    "harp-reuse-boundary-multiplier": [],
    "harp-reuse-compression-moments": [
        "harp-reuse-boundary-embedding",
        "harp-reuse-boundary-multiplier",
    ],
    "harp-reuse-dilation": [],
    "harp-reuse-norm-attainment": [],
    "harp-reuse-completed-square": [
        "harp-reuse-dilation",
        "harp-reuse-norm-attainment",
    ],
    "harp-reuse-companion-algebra": [],
    "harp-reuse-polynomial-power-cauchy": [],
    "harp-reuse-recurrence": [],
    "harp-reuse-scalar": [],
    "harp-positive-cubature": [],
    "harp-finite-measure-cubature": ["harp-positive-cubature"],
    "harp-finite-horizon-dilation-interface": ["harp-reuse-dilation"],
    "harp-finite-atomic-matrix-moments": [
        "harp-finite-measure-cubature",
        "harp-reuse-companion-algebra",
        "harp-reuse-polynomial-power-cauchy",
    ],
    "harp-counting-l2-witness-dimension-bound": [
        "harp-finite-atomic-matrix-moments",
        "harp-finite-horizon-dilation-interface",
        "harp-reuse-boundary-embedding",
        "harp-reuse-boundary-multiplier",
        "harp-reuse-boundary-square-root",
        "harp-reuse-compression-moments",
        "harp-reuse-polynomial-power-cauchy",
        "harp-double-layer-application",
    ],
    "harp-finite-horizon-recurrence": ["harp-reuse-recurrence"],
    "harp-operator-recurrence": [
        "harp-finite-horizon-dilation-interface",
        "harp-finite-horizon-recurrence",
        "harp-reuse-completed-square",
        "harp-reuse-dilation",
    ],
    "harp-perturbation-endpoint": [
        "harp-operator-recurrence",
        "harp-reuse-norm-attainment",
        "harp-reuse-scalar",
    ],
    "harp-double-layer-application": [],
    "harp-inner-limit": [],
    "harp-outer-limit": [],
    "harp-terminal-theorem": [
        "harp-counting-l2-witness-dimension-bound",
        "harp-perturbation-endpoint",
        "harp-inner-limit",
        "harp-outer-limit",
    ],
    "harp-closed-range-consequence": ["harp-terminal-theorem"],
}


def load_manifest() -> dict[str, object]:
    return json.loads((REPO / MANIFEST_PATH).read_text(encoding="utf-8"))


def assert_manifest_rejected(
    testcase: unittest.TestCase,
    payload: dict[str, object],
    pattern: str,
) -> None:
    manifest = route_validation.parse_route_manifest(payload)
    with testcase.assertRaisesRegex(route_validation.RouteValidationError, pattern):
        route_validation._validate_manifest_semantics(
            REPO, REPO / "formalization/lean", manifest
        )


class HarpRouteManifestTests(unittest.TestCase):
    maxDiff = None

    def test_manifest_exists_for_mapped_claim(self) -> None:
        self.assertTrue((REPO / MANIFEST_PATH).is_file())

    def test_published_manifest_validates_as_complete_local_without_mutation(self) -> None:
        before = subprocess.run(
            ["git", "status", "--short"],
            cwd=REPO,
            text=True,
            capture_output=True,
            check=True,
        ).stdout
        allowed = subprocess.run(
            [
                "python3",
                str(SCRIPT),
                "validate",
                "--route",
                "harp",
                "--allow-unpublished",
            ],
            cwd=REPO,
            text=True,
            capture_output=True,
            check=False,
        )
        default = subprocess.run(
            ["python3", str(SCRIPT), "validate", "--route", "harp"],
            cwd=REPO,
            text=True,
            capture_output=True,
            check=False,
        )
        after = subprocess.run(
            ["git", "status", "--short"],
            cwd=REPO,
            text=True,
            capture_output=True,
            check=True,
        ).stdout
        expected = {
            "schema_version": "crouzeix-route-validation/v1",
            "route_id": "harp",
            "status": "complete",
            "claim_level": "complete-local",
            "manifest_path": MANIFEST_PATH.as_posix(),
        }

        self.assertEqual(allowed.returncode, 0, allowed.stderr)
        self.assertEqual(default.returncode, 0, default.stderr)
        self.assertEqual(json.loads(allowed.stdout), expected)
        self.assertEqual(json.loads(default.stdout), expected)
        self.assertEqual(before, after)

    def test_ls_receipts_supports_package_and_direct_script_imports(self) -> None:
        module = importlib.import_module(
            "labs.crouzeix_proof_reproduction.ls_receipts"
        )
        package = "labs.crouzeix_proof_reproduction"
        siblings = {
            name: getattr(module, name)
            for name in (
                "ls_validation",
                "ls_contract",
                "provider_independence",
                "protocol",
            )
        }
        result = subprocess.run(
            [
                "python3",
                str(REPO / "labs/crouzeix_proof_reproduction/ls_receipts.py"),
            ],
            cwd=REPO,
            text=True,
            capture_output=True,
            check=False,
        )

        self.assertEqual(
            module.__package__, package
        )
        self.assertEqual(
            {name: sibling.__name__ for name, sibling in siblings.items()},
            {name: f"{package}.{name}" for name in siblings},
        )
        for imported_module in (
            module,
            siblings["ls_validation"],
            siblings["ls_validation"].formal_target,
            siblings["ls_validation"].formal_target.tickets,
        ):
            with self.subTest(module=imported_module.__name__):
                self.assertNotIn(
                    "except ImportError", inspect.getsource(imported_module)
                )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_manifest_binds_exact_route_and_ls_evidence_contract(self) -> None:
        manifest = load_manifest()
        self.assertEqual(manifest["route_id"], "harp")
        self.assertEqual(manifest["claim_kind"], "derived")
        self.assertEqual(manifest["aggregate_module"], "CrouzeixHarp")
        self.assertEqual(manifest["build_target"], "CrouzeixHarp")
        self.assertEqual(
            manifest["terminal_declaration"],
            "CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem",
        )
        self.assertEqual(
            manifest["consequence_declarations"],
            [
                "CrouzeixConjecture."
                "harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet"
            ],
        )
        self.assertEqual(manifest["source_identities"], [])
        self.assertEqual(
            manifest["allowed_axioms"],
            ["Classical.choice", "Quot.sound", "propext"],
        )
        self.assertEqual(
            manifest["review_path"],
            "evidence/crouzeix_conjecture/reviews/harp.json",
        )
        self.assertEqual(manifest["review_sha256"], HARP_REVIEW_SHA256)
        self.assertEqual(
            manifest["receipt_path"],
            "evidence/crouzeix_conjecture/routes/harp/receipt.json",
        )
        self.assertEqual(manifest["receipt_sha256"], HARP_RECEIPT_SHA256)
        self.assertEqual(manifest["route_dependencies"], [{
            "route_id": "lorist-schwenninger",
            "manifest_path": LS_MANIFEST_PATH,
            "manifest_sha256": LS_MANIFEST_SHA256,
            "receipt_path": LS_RECEIPT_PATH,
            "receipt_sha256": LS_RECEIPT_SHA256,
        }])

    def test_exact_nodes_classifications_and_topological_dependencies(self) -> None:
        manifest = load_manifest()
        nodes = manifest["nodes"]
        self.assertEqual(tuple(node["node_id"] for node in nodes), EXPECTED_ORDER)
        by_id = {node["node_id"]: node for node in nodes}
        self.assertEqual(len(by_id), 24)
        self.assertEqual(
            set(by_id), set(EXPECTED_CONCEPTUAL_NODES) | set(EXPECTED_REUSE_NODES)
        )
        for node_id, expected in EXPECTED_CONCEPTUAL_NODES.items():
            declaration, module_path, role, provenance, correspondence = expected
            with self.subTest(node_id=node_id):
                node = by_id[node_id]
                self.assertEqual(node["declaration"], declaration)
                self.assertEqual(node["module_path"], module_path)
                self.assertEqual(node["role"], role)
                self.assertEqual(node["provenance_kind"], provenance)
                self.assertEqual(node["correspondence_kind"], correspondence)
                self.assertEqual(node["dependency_ids"], EXPECTED_DEPENDENCIES[node_id])
                self.assertIsNone(node["source_locator"])
                self.assertIsNone(node["reused_from_route"])
                self.assertIsNone(node["reused_node_id"])
        for node_id, (declaration, module_path) in EXPECTED_REUSE_NODES.items():
            with self.subTest(node_id=node_id):
                node = by_id[node_id]
                self.assertEqual(node["declaration"], declaration)
                self.assertEqual(node["module_path"], module_path)
                self.assertEqual(node["role"], "load-bearing")
                self.assertEqual(node["provenance_kind"], "reused-route")
                self.assertEqual(node["correspondence_kind"], "reused-route")
                self.assertEqual(node["dependency_ids"], EXPECTED_DEPENDENCIES[node_id])
                self.assertEqual(node["reused_from_route"], "lorist-schwenninger")
                self.assertIsNone(node["reused_node_id"])
                self.assertIsNone(node["source_locator"])

    def test_closure_digest_coverage_and_reuse_modules_are_exact(self) -> None:
        manifest = load_manifest()
        closure = route_validation.active_local_closure(
            REPO / "formalization/lean", "CrouzeixHarp"
        )
        node_modules = {
            route_validation._module_from_path(node["module_path"])
            for node in manifest["nodes"]
        }
        reused_modules = {
            route_validation._module_from_path(node["module_path"])
            for node in manifest["nodes"]
            if node["provenance_kind"] == "reused-route"
        }
        shared = set(manifest["shared_foundation_modules"])
        self.assertEqual(len(closure), 59)
        self.assertEqual(route_validation.string_roster_sha256(closure), EXPECTED_CLOSURE_SHA256)
        self.assertEqual(manifest["module_closure"], list(closure))
        self.assertEqual(manifest["module_closure_sha256"], EXPECTED_CLOSURE_SHA256)
        self.assertEqual(len(node_modules), 24)
        self.assertEqual(reused_modules, set(route_validation.HARP_ALLOWED_LS_SUPPORT))
        self.assertEqual(manifest["shared_foundation_modules"], sorted(shared))
        self.assertFalse(node_modules & shared)
        self.assertEqual(node_modules | shared, set(closure))
        self.assertIn("CrouzeixHarp", shared)

    def test_exactly_24_type_artifacts_match_normalized_digests(self) -> None:
        manifest = load_manifest()
        expected_paths = set()
        for node in manifest["nodes"]:
            path = Path(node["declaration_type_path"])
            expected_paths.add(path)
            self.assertEqual(path.parent, TYPE_ROOT)
            source = (REPO / path).read_text(encoding="utf-8")
            self.assertEqual(source, source.strip() + "\n")
            self.assertEqual(
                route_validation.normalized_type_sha256(source),
                node["statement_sha256"],
            )
        self.assertEqual(len(expected_paths), 24)
        self.assertEqual(
            {path.relative_to(REPO) for path in (REPO / TYPE_ROOT).glob("*.txt")},
            expected_paths,
        )

    def test_terminal_type_digest_matches_terminal_node(self) -> None:
        manifest = load_manifest()
        terminal = next(node for node in manifest["nodes"] if node["role"] == "terminal")
        self.assertEqual(manifest["terminal_type_sha256"], terminal["statement_sha256"])

    @unittest.skipUnless(importlib.util.find_spec("jsonschema"), "jsonschema is unavailable")
    def test_real_manifest_validates_against_json_schema(self) -> None:
        import jsonschema

        schema = json.loads(
            (REPO / "labs/crouzeix_proof_reproduction/schemas/route_manifest.schema.json")
            .read_text(encoding="utf-8")
        )
        jsonschema.Draft202012Validator(schema).validate(load_manifest())

    def test_contract_mutations_are_rejected(self) -> None:
        original = load_manifest()
        cases: list[tuple[str, dict[str, object], str]] = []

        extra_reuse = copy.deepcopy(original)
        extra = copy.deepcopy(extra_reuse["nodes"][0])
        extra["node_id"] = "harp-reuse-ls-terminal-provider"
        extra["module_path"] = "Crouzeix/LoristSchwenninger/MainTheorem.lean"
        extra_reuse["nodes"].insert(11, extra)
        cases.append(("twelfth LS module", extra_reuse, "inactive|reuse|coverage"))

        omitted_reuse = copy.deepcopy(original)
        omitted_reuse["nodes"] = [
            node for node in omitted_reuse["nodes"]
            if node["node_id"] != "harp-reuse-boundary-multiplier"
        ]
        cases.append(
            ("omitted reuse", omitted_reuse, "unknown dependency|unmapped|reuse coverage")
        )

        source_faithful = copy.deepcopy(original)
        source_faithful["claim_kind"] = "source-faithful"
        cases.append(("source-faithful claim", source_faithful, "must be derived"))

        independence_claim = copy.deepcopy(original)
        independence_claim["source_identities"] = ["sha256:" + "a" * 64]
        cases.append(("wholly independent claim", independence_claim, "source identities must be empty"))

        hidden_reuse = copy.deepcopy(original)
        hidden_reuse["route_dependencies"] = []
        for node in hidden_reuse["nodes"]:
            if node["provenance_kind"] == "reused-route":
                node["provenance_kind"] = "shared-foundation"
                node["correspondence_kind"] = "shared-foundation"
                node["reused_from_route"] = None
        cases.append(("hidden LS reuse", hidden_reuse, "explicitly record LS reuse"))

        for field in ("route_id", "manifest_sha256", "receipt_sha256"):
            wrong_dependency = copy.deepcopy(original)
            wrong_dependency["route_dependencies"][0][field] = (
                "jin" if field == "route_id" else "0" * 64
            )
            cases.append((f"wrong LS dependency {field}", wrong_dependency, "dependency identity"))

        missing_concept = copy.deepcopy(original)
        missing_concept["nodes"] = [
            node for node in missing_concept["nodes"]
            if node["node_id"] != "harp-finite-measure-cubature"
        ]
        cases.append(("missing conceptual node", missing_concept, "unknown dependency|unmapped"))

        broken_dependency = copy.deepcopy(original)
        next(
            node for node in broken_dependency["nodes"]
            if node["node_id"] == "harp-terminal-theorem"
        )["dependency_ids"] = ["harp-missing-provider"]
        cases.append(("broken dependency", broken_dependency, "unknown dependency"))

        swapped_types = copy.deepcopy(original)
        first = swapped_types["nodes"][0]
        second = swapped_types["nodes"][1]
        first["statement_sha256"], second["statement_sha256"] = (
            second["statement_sha256"], first["statement_sha256"]
        )
        cases.append(("swapped declaration type digest", swapped_types, "type digest mismatch"))

        for label, payload, pattern in cases:
            with self.subTest(label=label):
                assert_manifest_rejected(self, payload, pattern)

    def test_provider_import_mutations_are_rejected(self) -> None:
        manifest = route_validation.parse_route_manifest(load_manifest())
        forbidden, prefixes = route_validation._provider_policy(manifest)
        for imported in (
            "Crouzeix.Jin.Terminal",
            "CrouzeixLoristSchwenninger",
            "Crouzeix.LoristSchwenninger.MainTheorem",
        ):
            with self.subTest(imported=imported), tempfile.TemporaryDirectory() as temporary:
                lean_root = Path(temporary)
                (lean_root / "CrouzeixHarp.lean").write_text(
                    f"import {imported}\n", encoding="utf-8"
                )
                with self.assertRaisesRegex(
                    provider_independence.ProviderIndependenceError,
                    "forbidden",
                ):
                    provider_independence.audit_provider_independence(
                        lean_root,
                        ("CrouzeixHarp",),
                        forbidden,
                        forbidden_prefixes=prefixes,
                    )

    def test_active_route_sources_have_no_proof_escape_tokens(self) -> None:
        closure = route_validation.active_local_closure(
            REPO / "formalization/lean", "CrouzeixHarp"
        )
        for module in closure:
            path = REPO / "formalization/lean" / route_validation.module_relative_path(module)
            source = path.read_text(encoding="utf-8")
            with self.subTest(module=module):
                self.assertEqual(
                    provider_independence.scan_active_source(source, module).dangers,
                    (),
                )
        inert = '-- sorry axiom unsafe\n#check "admit opaque native_decide implemented_by"\n'
        self.assertEqual(
            provider_independence.scan_active_source(inert, "CommentAndStringFixture").dangers,
            (),
        )
        self.assertEqual(
            tuple(
                finding.token
                for finding in provider_independence.scan_active_source(
                    "axiom escape : Prop\n", "ActiveAxiomFixture"
                ).dangers
            ),
            ("axiom",),
        )


if __name__ == "__main__":
    unittest.main()
