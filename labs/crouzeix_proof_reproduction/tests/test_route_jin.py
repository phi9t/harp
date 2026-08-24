from __future__ import annotations

import copy
import hashlib
import json
import os
import re
import subprocess
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path, PurePosixPath

from labs.crouzeix_proof_reproduction import route_validation


REPO = Path(__file__).resolve().parents[3]
SCRIPT = REPO / "labs/crouzeix_proof_reproduction/proof_evidence.py"
MANIFEST_PATH = Path(
    "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json"
)
TYPE_ROOT = MANIFEST_PATH.parent / "declaration-types"
SOURCE_COMMIT = "565b6a3e0659b6e0785f783b016c3f6d9f171fa5"
SOURCE_ARCHIVE_SHA256 = (
    "33ee5b75c1037866c4d2bda8eff872cc0640fd42e7c31c58a8e6047405f69542"
)
SOURCE_ROOT = Path(
    "/private/tmp/harp-cpfr085-jin.S8a1v1/"
    "CrouzeixConjecture-565b6a3e0659b6e0785f783b016c3f6d9f171fa5"
)
EXPECTED_CLOSURE_SHA256 = (
    "e0498425be2f30edc876eb5e802628169f80dedd54204b1861dfe62715eb5b7f"
)
GIT_LOCATOR_RE = re.compile(
    r"git:(?P<commit>[0-9a-f]{40}):(?P<path>[^#]+)"
    r"#L(?P<start>[1-9][0-9]*)-L(?P<end>[1-9][0-9]*)\Z"
)

EXPECTED_NODES = {
    "jin-polynomial-bound": (
        "CrouzeixConjecture.PolynomialCrouzeixBound",
        "CrouzeixConjecture/Statements.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-numerical-range-convexity": (
        "CrouzeixConjecture.numericalRange_convex",
        "CrouzeixConjecture/NumericalRangeConvexity.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-simple-spectrum-approximation": (
        "CrouzeixConjecture.tendsto_simpleSpectrumApproximation",
        "CrouzeixConjecture/SimpleSpectrumDensity.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-completion-algebra-cancellation": (
        "CrouzeixConjecture.completion_mulVec_add_eq_zero",
        "CrouzeixConjecture/CompletionAlgebra.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-completion-sampling": (
        "CrouzeixConjecture.completion_X_inequality_of_sampling",
        "CrouzeixConjecture/CompletionSampling.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-completion-kernel-cancellation": (
        "CrouzeixConjecture.completion_X_inequality_of_positiveKernel",
        "CrouzeixConjecture/CompletionKernelModel.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-completion-gramian-bridge": (
        "CrouzeixConjecture.completion_gramian_expression_posSemidef_of_source",
        "CrouzeixConjecture/CompletionGramianBridge.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-completion-eigenvector-endpoint": (
        "CrouzeixConjecture.norm_le_two_of_gramian_inequality",
        "CrouzeixConjecture/CompletionEigenvector.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-positive-real-completion": (
        "CrouzeixConjecture.norm_completionDiagonalizableMatrix_le_two_of_positiveKernelModel",
        "CrouzeixConjecture/PositiveRealCompletion.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-positive-real-completion-statement": (
        "CrouzeixConjecture.positiveRealCompletionStatement",
        "CrouzeixConjecture/CompletionDiagonalization.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-double-layer-realization": (
        "CrouzeixConjecture.PositivePeriodicRadialData.OrientedRadialConvexBoundary.norm_functionEval_le_two_of_holomorphic_of_simpleDiagonalization",
        "CrouzeixConjecture/HolomorphicDoubleLayer.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-function-maximum-convergence": (
        "CrouzeixConjecture.tendsto_maxFunctionModulusOnSet_of_outerApproximation",
        "CrouzeixConjecture/FunctionMaximum.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-fixed-outer-domain-convergence": (
        "CrouzeixConjecture.holomorphicCrouzeixBound",
        "CrouzeixConjecture/HolomorphicOuterLimit.lean",
        "load-bearing",
        "source",
        "direct-source",
    ),
    "jin-terminal-polynomial-specialization": (
        "CrouzeixConjecture.polynomialCrouzeixBound_of_holomorphicCrouzeixBound",
        "CrouzeixConjecture/HolomorphicConsequences.lean",
        "load-bearing",
        "derived",
        "derived-extraction",
    ),
    "jin-terminal-crouzeix": (
        "CrouzeixConjecture.crouzeixConjecture",
        "Crouzeix/Jin/Terminal.lean",
        "terminal",
        "source",
        "compatibility-port",
    ),
    "jin-finite-rational-consequence": (
        "CrouzeixConjecture.crouzeixRationalBound",
        "CrouzeixConjecture/FinalTheorems.lean",
        "consequence",
        "source",
        "compatibility-port",
    ),
    "jin-holomorphic-rational-bridge": (
        "CrouzeixConjecture.jinHolomorphicCrouzeixRationalBound",
        "Crouzeix/Jin/RationalBridge.lean",
        "load-bearing",
        "source",
        "structural-refactor",
    ),
    "jin-hilbert-polynomial-core": (
        "CrouzeixConjecture.hilbertSpacePolynomialCrouzeix_of_mainTheorem",
        "CrouzeixConjecture/HilbertSpaceCore.lean",
        "load-bearing",
        "source",
        "structural-refactor",
    ),
    "jin-hilbert-polynomial-consequence": (
        "CrouzeixConjecture.hilbertSpacePolynomialCrouzeix",
        "CrouzeixConjecture/HilbertSpace.lean",
        "consequence",
        "source",
        "compatibility-port",
    ),
    "jin-hilbert-spectral-set-core": (
        "CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem",
        "CrouzeixConjecture/HilbertSpectralSetCore.lean",
        "load-bearing",
        "source",
        "structural-refactor",
    ),
    "jin-hilbert-spectral-set-consequence": (
        "CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet",
        "CrouzeixConjecture/HilbertSpectralSet.lean",
        "consequence",
        "source",
        "compatibility-port",
    ),
}

ALLOWED_CORRESPONDENCE = {
    "source": {
        "direct-source",
        "compatibility-port",
        "structural-refactor",
    },
    "derived": {"derived-extraction"},
    "shared-foundation": {"shared-foundation"},
    "reused-route": {"reused-route"},
}

EXPECTED_DEPENDENCIES = {
    "jin-polynomial-bound": [],
    "jin-numerical-range-convexity": [],
    "jin-simple-spectrum-approximation": [],
    "jin-completion-algebra-cancellation": [],
    "jin-completion-sampling": [],
    "jin-completion-kernel-cancellation": ["jin-completion-sampling"],
    "jin-completion-gramian-bridge": [],
    "jin-completion-eigenvector-endpoint": [],
    "jin-positive-real-completion": [
        "jin-completion-kernel-cancellation",
        "jin-completion-gramian-bridge",
        "jin-completion-eigenvector-endpoint",
    ],
    "jin-positive-real-completion-statement": ["jin-positive-real-completion"],
    "jin-double-layer-realization": ["jin-positive-real-completion-statement"],
    "jin-function-maximum-convergence": [],
    "jin-fixed-outer-domain-convergence": [
        "jin-numerical-range-convexity",
        "jin-simple-spectrum-approximation",
        "jin-double-layer-realization",
        "jin-function-maximum-convergence",
    ],
    "jin-terminal-polynomial-specialization": ["jin-fixed-outer-domain-convergence"],
    "jin-terminal-crouzeix": ["jin-terminal-polynomial-specialization"],
    "jin-holomorphic-rational-bridge": ["jin-fixed-outer-domain-convergence"],
    "jin-finite-rational-consequence": ["jin-holomorphic-rational-bridge"],
    "jin-hilbert-polynomial-core": [],
    "jin-hilbert-polynomial-consequence": [
        "jin-terminal-crouzeix",
        "jin-hilbert-polynomial-core",
    ],
    "jin-hilbert-spectral-set-core": [
        "jin-hilbert-polynomial-core",
    ],
    "jin-hilbert-spectral-set-consequence": [
        "jin-terminal-crouzeix",
        "jin-hilbert-spectral-set-core",
    ],
}

AUDITED_DEPENDENCY_EVIDENCE = {
    "jin-simple-spectrum-approximation": {
        "reason": (
            "The body closes by squeezing the quantitative simple-spectrum "
            "error bound; numerical-range convexity is not in the provider chain."
        ),
        "checks": [(
            "CrouzeixConjecture/SimpleSpectrumDensity.lean",
            "tendsto_simpleSpectrumApproximation",
            ["norm_simpleSpectrumApproximation_sub_lt", "squeeze_zero"],
            ["numericalRange_convex"],
        )],
    },
    "jin-completion-sampling": {
        "reason": (
            "The exported sampling theorem uses local sampling/coefficient lemmas; "
            "shared algebra only appears below unmapped helper declarations."
        ),
        "checks": [(
            "CrouzeixConjecture/CompletionSampling.lean",
            "completion_X_inequality_of_sampling",
            ["completionSampleCoefficient_posSemidef_of_quadratic_nonneg"],
            ["completion_mulVec_add_eq_zero"],
        )],
    },
    "jin-completion-kernel-cancellation": {
        "reason": (
            "The mapped theorem calls the mapped sampling theorem after an "
            "unmapped positive-kernel calculation, so sampling is the only "
            "contracted mapped provider."
        ),
        "checks": [(
            "CrouzeixConjecture/CompletionKernelModel.lean",
            "completion_X_inequality_of_positiveKernel",
            [
                "completion_X_inequality_of_sampling",
                "completionSampleCoefficient_quadratic_nonneg_of_positiveKernel",
            ],
            ["completion_mulVec_add_eq_zero"],
        )],
    },
    "jin-completion-gramian-bridge": {
        "reason": (
            "The bridge consumes the Gramian-source inequality as an argument "
            "and rewrites through local congruence lemmas; it does not call the "
            "mapped completion algebra cancellation theorem."
        ),
        "checks": [(
            "CrouzeixConjecture/CompletionGramianBridge.lean",
            "completion_gramian_expression_posSemidef_of_source",
            [
                "hsource.conjTranspose_mul_mul_same",
                "completion_PRX_congruence_eq_gramian_expression",
            ],
            ["completion_mulVec_add_eq_zero"],
        )],
    },
    "jin-completion-eigenvector-endpoint": {
        "reason": (
            "The endpoint assumes the Gramian inequality as `hineq` and derives "
            "the norm endpoint, so there is no mapped Gramian provider edge."
        ),
        "checks": [(
            "CrouzeixConjecture/CompletionEigenvector.lean",
            "norm_le_two_of_gramian_inequality",
            ["four_sub_conjTranspose_mul_self_posSemidef_of_gramian_inequality"],
            ["completion_gramian_expression_posSemidef_of_source"],
        )],
    },
    "jin-positive-real-completion": {
        "reason": (
            "The completion theorem directly calls kernel cancellation, the "
            "Gramian bridge, and the eigenvector endpoint."
        ),
        "checks": [(
            "CrouzeixConjecture/PositiveRealCompletion.lean",
            "norm_completionDiagonalizableMatrix_le_two_of_positiveKernelModel",
            [
                "completion_X_inequality_of_positiveKernel",
                "completion_gramian_expression_posSemidef_of_source",
                "norm_le_two_of_gramian_inequality",
            ],
            [],
        )],
    },
    "jin-terminal-polynomial-specialization": {
        "reason": (
            "The derived specialization calls the holomorphic theorem directly; "
            "the polynomial-bound predicate is statement shape only."
        ),
        "checks": [(
            "CrouzeixConjecture/HolomorphicConsequences.lean",
            "polynomialCrouzeixBound_of_holomorphicCrouzeixBound",
            ["holomorphicCrouzeixBound"],
            ["PolynomialCrouzeixBound"],
        )],
    },
    "jin-terminal-crouzeix": {
        "reason": (
            "The route terminal delegates to the derived polynomial "
            "specialization, not to the polynomial-bound predicate."
        ),
        "checks": [(
            "Crouzeix/Jin/Terminal.lean",
            "crouzeixConjecture",
            ["polynomialCrouzeixBound_of_holomorphicCrouzeixBound"],
            ["PolynomialCrouzeixBound"],
        )],
    },
    "jin-holomorphic-rational-bridge": {
        "reason": (
            "The route-local bridge calls an unmapped rational specialization; "
            "that theorem's body calls `holomorphicCrouzeixBound`, so contraction "
            "keeps the fixed-outer provider."
        ),
        "checks": [
            (
                "Crouzeix/Jin/RationalBridge.lean",
                "jinHolomorphicCrouzeixRationalBound",
                ["holomorphicCrouzeixRationalBound"],
                ["PolynomialCrouzeixBound"],
            ),
            (
                "CrouzeixConjecture/HolomorphicConsequences.lean",
                "holomorphicCrouzeixRationalBound",
                ["holomorphicCrouzeixBound"],
                ["PolynomialCrouzeixBound"],
            ),
        ],
    },
    "jin-hilbert-polynomial-core": {
        "reason": (
            "The core theorem accepts `hMain` and routes through local Hilbert "
            "helpers; it does not call numerical-range convexity as a mapped "
            "provider."
        ),
        "checks": [(
            "CrouzeixConjecture/HilbertSpaceCore.lean",
            "hilbertSpacePolynomialCrouzeix_of_mainTheorem",
            ["operatorPolynomialEval_apply_le_of_norm_one_of_mainTheorem hMain"],
            ["numericalRange_convex", "crouzeixConjecture"],
        )],
    },
    "jin-hilbert-spectral-set-core": {
        "reason": (
            "The spectral core accepts `hMain` and calls unmapped rational and "
            "spectral helpers; that helper chain contracts through the mapped "
            "Hilbert polynomial core."
        ),
        "checks": [
            (
                "CrouzeixConjecture/HilbertSpectralSetCore.lean",
                "norm_operatorPolynomialEval_le_of_forall_closedNumericalRange_of_mainTheorem",
                ["hilbertSpacePolynomialCrouzeix_of_mainTheorem hMain"],
                ["crouzeixConjecture"],
            ),
            (
                "CrouzeixConjecture/HilbertSpectralSetCore.lean",
                "closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem",
                [
                    "spectrum_subset_closedOperatorNumericalRange_of_mainTheorem hMain",
                    "operatorRationalEval_eq_num_mul_inverse_denom_of_mainTheorem hMain",
                    "hilbertSpaceRationalCrouzeix_of_mainTheorem hMain",
                ],
                ["crouzeixConjecture"],
            ),
        ],
    },
    "jin-hilbert-polynomial-consequence": {
        "reason": (
            "The public polynomial consequence supplies the terminal "
            "`crouzeixConjecture` provider to the Hilbert polynomial core."
        ),
        "checks": [(
            "CrouzeixConjecture/HilbertSpace.lean",
            "hilbertSpacePolynomialCrouzeix",
            [
                "hilbertSpacePolynomialCrouzeix_of_mainTheorem",
                "fun d => crouzeixConjecture (n := Fin d)",
            ],
            [],
        )],
    },
    "jin-hilbert-spectral-set-consequence": {
        "reason": (
            "The public spectral consequence calls the spectral core and supplies "
            "the terminal `crouzeixConjecture` provider through "
            "`jinFiniteMatrixMainTheorem`."
        ),
        "checks": [
            (
                "CrouzeixConjecture/HilbertSpectralSet.lean",
                "jinFiniteMatrixMainTheorem",
                ["fun d => crouzeixConjecture (n := Fin d)"],
                [],
            ),
            (
                "CrouzeixConjecture/HilbertSpectralSet.lean",
                "closedOperatorNumericalRange_isTwoSpectralSet",
                ["closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem"],
                ["hilbertSpacePolynomialCrouzeix_of_mainTheorem"],
            ),
        ],
    },
}

HISTORICAL_RECEIPTS = {
    "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-max-polynomial-modulus/attempt-004/receipt.json": "9e0cf882a8f866a4897a46548c7e387fb0d07b7dee159b3c2260ef9ece1fb563",
    "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-polynomial-bound/attempt-001/receipt.json": "b25ca01a109ad93708d474feb60389e6544f5fb54fe64f557e24571cd7de58c8",
    "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/proof-slices/jin-terminal-crouzeix/attempt-001/receipt.json": "a8b6fdc904ed35de4a533b924bf8847c923f3a50e3cc19785ced4bee5f7da0f9",
}


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def load_manifest() -> dict[str, object]:
    return json.loads((REPO / MANIFEST_PATH).read_text(encoding="utf-8"))


def git_locator_parts(locator: str) -> tuple[str, PurePosixPath, int, int]:
    match = GIT_LOCATOR_RE.fullmatch(locator)
    if match is None:
        raise AssertionError(f"invalid Git source locator: {locator}")
    return (
        match.group("commit"),
        PurePosixPath(match.group("path")),
        int(match.group("start")),
        int(match.group("end")),
    )


def excerpt_bytes(source: bytes, start: int, end: int) -> bytes:
    return b"".join(source.splitlines(keepends=True)[start - 1 : end])


def strip_lean_block_comments(source: str) -> str:
    result: list[str] = []
    index = 0
    depth = 0
    while index < len(source):
        if source.startswith("/-", index):
            result.extend("  ")
            index += 2
            depth += 1
            continue
        if depth and source.startswith("-/", index):
            result.extend("  ")
            index += 2
            depth -= 1
            continue
        char = source[index]
        if depth:
            result.append("\n" if char == "\n" else " ")
        else:
            result.append(char)
        index += 1
    return "".join(result)


def lean_declaration_body(relative_path: str, declaration: str) -> str:
    source = (REPO / "formalization/lean" / relative_path).read_text(encoding="utf-8")
    searchable = strip_lean_block_comments(source)
    declaration_re = re.compile(
        rf"(?m)^(?:private\s+)?(?:theorem|lemma|def)\s+{re.escape(declaration)}(?:\s|$)"
    )
    match = declaration_re.search(searchable)
    if match is None:
        raise AssertionError(f"missing Lean declaration {declaration} in {relative_path}")
    next_re = re.compile(r"(?m)^(?:private\s+)?(?:theorem|lemma|def)\s+[A-Za-z_][A-Za-z0-9_']*")
    next_match = next_re.search(searchable, match.end())
    return source[match.start() : next_match.start() if next_match else len(source)]


class JinRouteManifestTests(unittest.TestCase):
    maxDiff = None

    def test_manifest_exists_for_mapped_claim(self) -> None:
        self.assertTrue((REPO / MANIFEST_PATH).is_file())

    def test_exact_closure_digest_and_nonoverlapping_coverage(self) -> None:
        manifest = load_manifest()
        closure = route_validation.active_local_closure(
            REPO / "formalization/lean", "CrouzeixJin"
        )
        node_modules = {
            str(node["module_path"])[:-5].replace("/", ".")
            for node in manifest["nodes"]
        }
        shared = set(manifest["shared_foundation_modules"])

        self.assertEqual(len(closure), 66)
        self.assertEqual(route_validation.string_roster_sha256(closure), EXPECTED_CLOSURE_SHA256)
        self.assertEqual(manifest["module_closure"], list(closure))
        self.assertEqual(manifest["module_closure_sha256"], EXPECTED_CLOSURE_SHA256)
        self.assertEqual(len(node_modules), 21)
        self.assertEqual(len(shared), 45)
        self.assertFalse(node_modules & shared)
        self.assertEqual(node_modules | shared, set(closure))
        self.assertIn("CrouzeixJin", shared)
        self.assertIn("Crouzeix.Jin.MaxPolynomialModulus", shared)

    def test_proof_spine_nodes_dependencies_roles_and_provenance_are_exact(self) -> None:
        manifest = load_manifest()
        nodes = {node["node_id"]: node for node in manifest["nodes"]}
        self.assertEqual(set(nodes), set(EXPECTED_NODES))
        for node_id, (declaration, module_path, role, provenance, correspondence) in EXPECTED_NODES.items():
            with self.subTest(node_id=node_id):
                node = nodes[node_id]
                self.assertEqual(node["declaration"], declaration)
                self.assertEqual(node["module_path"], module_path)
                self.assertEqual(node["role"], role)
                self.assertEqual(node["provenance_kind"], provenance)
                self.assertEqual(node["correspondence_kind"], correspondence)
                self.assertEqual(node["dependency_ids"], EXPECTED_DEPENDENCIES[node_id])
                self.assertIn(
                    node["correspondence_kind"],
                    ALLOWED_CORRESPONDENCE[node["provenance_kind"]],
                )
        derived = nodes["jin-terminal-polynomial-specialization"]
        self.assertIsNone(derived["source_locator"])
        self.assertIsNone(derived["source_archive_sha256"])
        self.assertIsNone(derived["source_file_sha256"])
        self.assertIsNone(derived["source_excerpt_sha256"])
        self.assertIsNone(derived["source_line_count"])
        self.assertFalse(any(node["provenance_kind"] == "reused-route" for node in nodes.values()))

    def test_audited_dependency_edges_match_active_provider_calls(self) -> None:
        manifest = load_manifest()
        nodes = {node["node_id"]: node for node in manifest["nodes"]}

        for node_id, audit in AUDITED_DEPENDENCY_EVIDENCE.items():
            with self.subTest(node_id=node_id, field="dependencies"):
                self.assertEqual(nodes[node_id]["dependency_ids"], EXPECTED_DEPENDENCIES[node_id])
            for relative_path, declaration, required, forbidden in audit["checks"]:
                body = lean_declaration_body(relative_path, declaration)
                for token in required:
                    with self.subTest(node_id=node_id, declaration=declaration, required=token):
                        self.assertIn(token, body)
                for token in forbidden:
                    with self.subTest(node_id=node_id, declaration=declaration, forbidden=token):
                        self.assertNotIn(token, body)

    def test_consequence_declarations_match_exact_consequence_nodes(self) -> None:
        manifest = load_manifest()
        consequence_nodes = [
            node["declaration"] for node in manifest["nodes"]
            if node["role"] == "consequence"
        ]
        self.assertEqual(manifest["consequence_declarations"], consequence_nodes)
        self.assertEqual(len(consequence_nodes), 3)

    def test_every_git_locator_has_complete_remote_metadata(self) -> None:
        manifest = load_manifest()
        self.assertIn(f"sha256:{SOURCE_ARCHIVE_SHA256}", manifest["source_identities"])
        for node in manifest["nodes"]:
            if node["provenance_kind"] != "source":
                continue
            with self.subTest(node_id=node["node_id"]):
                commit, path, start, end = git_locator_parts(node["source_locator"])
                self.assertEqual(commit, SOURCE_COMMIT)
                self.assertFalse(path.is_absolute())
                self.assertNotIn("..", path.parts)
                self.assertLessEqual(start, end)
                self.assertEqual(node["source_archive_sha256"], SOURCE_ARCHIVE_SHA256)
                self.assertRegex(node["source_file_sha256"], r"^[0-9a-f]{64}$")
                self.assertRegex(node["source_excerpt_sha256"], r"^[0-9a-f]{64}$")
                self.assertIsInstance(node["source_line_count"], int)
                self.assertLessEqual(end, node["source_line_count"])

    def test_artifact_manifest_exactly_registers_all_route_source_paths(self) -> None:
        manifest = load_manifest()
        artifact_manifest = json.loads(
            (MANIFEST_PATH.parent / "artifact-manifest.json").read_text(encoding="utf-8")
        )
        route_metadata: dict[str, tuple[str, int]] = {}
        for node in manifest["nodes"]:
            if node["provenance_kind"] != "source":
                continue
            _, path, _, _ = git_locator_parts(node["source_locator"])
            metadata = (node["source_file_sha256"], node["source_line_count"])
            self.assertEqual(route_metadata.setdefault(path.as_posix(), metadata), metadata)
        artifact_records = [
            record for record in artifact_manifest["artifacts"].values()
            if str(record["path"]).endswith(".lean")
        ]
        self.assertEqual(
            {record["path"] for record in artifact_records}, set(route_metadata)
        )
        self.assertEqual(len(artifact_records), len(route_metadata))
        for record in artifact_records:
            path = record["path"]
            file_sha, _ = route_metadata[path]
            self.assertEqual(record["sha256"], file_sha)
            self.assertEqual(record["license_status"], "not-present-at-revision")
            self.assertEqual(
                record["source_locator"], f"git:{SOURCE_COMMIT}:{path}"
            )
            if SOURCE_ROOT.is_dir():
                source = (SOURCE_ROOT / path).read_bytes()
                self.assertEqual(record["bytes"], len(source))
                self.assertEqual(record["sha256"], sha256_bytes(source))

    def test_git_locator_fixture_binds_archive_and_registered_file_hash(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            artifact_path = (
                root
                / "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/artifact-manifest.json"
            )
            artifact_path.parent.mkdir(parents=True)
            source = b"alpha\nbeta\ngamma\n"
            file_sha = sha256_bytes(source)
            excerpt_sha = sha256_bytes(b"beta\ngamma\n")
            archive_sha = "a" * 64
            commit = "b" * 40
            artifact_path.write_text(json.dumps({
                "schema_version": "crouzeix-formal-artifact-manifest/v1",
                "source_commit": commit,
                "archive": {"sha256": archive_sha},
                "artifacts": {
                    "sample": {
                        "path": "Lean/Sample.lean",
                        "sha256": file_sha,
                    }
                },
            }))
            payload = {
                "schema_version": route_validation.MANIFEST_SCHEMA_VERSION,
                "route_id": "jin",
                "claim_kind": "source-faithful",
                "aggregate_module": "CrouzeixJin",
                "build_target": "CrouzeixJin",
                "terminal_declaration": "CrouzeixConjecture.main",
                "terminal_type_sha256": "c" * 64,
                "consequence_declarations": ["CrouzeixConjecture.main"],
                "source_identities": [f"sha256:{archive_sha}"],
                "shared_foundation_modules": [],
                "module_closure": ["CrouzeixJin"],
                "module_closure_sha256": "d" * 64,
                "allowed_axioms": [],
                "review_path": "review.json",
                "review_sha256": None,
                "receipt_path": "receipt.json",
                "receipt_sha256": None,
                "nodes": [{
                    "node_id": "main",
                    "role": "consequence",
                    "declaration": "CrouzeixConjecture.main",
                    "module_path": "CrouzeixJin.lean",
                    "dependency_ids": [],
                    "provenance_kind": "source",
                    "correspondence_kind": "direct-source",
                    "source_locator": f"git:{commit}:Lean/Sample.lean#L2-L3",
                    "source_archive_sha256": archive_sha,
                    "source_file_sha256": file_sha,
                    "source_excerpt_sha256": excerpt_sha,
                    "source_line_count": 3,
                    "reused_from_route": None,
                    "reused_node_id": None,
                    "declaration_type_path": "type.txt",
                    "statement_sha256": "c" * 64,
                }],
            }
            manifest = route_validation.parse_route_manifest(payload)
            route_validation.validate_source_provenance_metadata(
                root, manifest, manifest.nodes[0]
            )
            for field, value, pattern in (
                ("source_archive_sha256", "e" * 64, "archive"),
                ("source_file_sha256", "e" * 64, "file"),
                ("source_line_count", 2, "line count"),
            ):
                candidate = copy.deepcopy(payload)
                candidate["nodes"][0][field] = value
                parsed = route_validation.parse_route_manifest(candidate)
                with self.assertRaisesRegex(route_validation.RouteValidationError, pattern):
                    route_validation.validate_source_provenance_metadata(
                        root, parsed, parsed.nodes[0]
                    )

    def test_jin_git_locator_requires_exactly_one_registered_artifact_path(self) -> None:
        manifest = route_validation.load_route_manifest(REPO, MANIFEST_PATH)
        node = replace(
            manifest.nodes[0],
            source_locator=(
                f"git:{SOURCE_COMMIT}:"
                "Lean/CrouzeixConjecture/Unregistered.lean#L1-L1"
            ),
            source_file_sha256="a" * 64,
            source_excerpt_sha256="b" * 64,
            source_line_count=1,
        )

        with self.assertRaisesRegex(
            route_validation.RouteValidationError,
            "exactly one.*artifact.*path",
        ):
            route_validation.validate_source_provenance_metadata(REPO, manifest, node)

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            relative = MANIFEST_PATH.with_name("artifact-manifest.json")
            destination = root / relative
            destination.parent.mkdir(parents=True)
            artifact_manifest = json.loads((REPO / relative).read_text())
            registered = copy.deepcopy(artifact_manifest["artifacts"]["statements"])
            artifact_manifest["artifacts"]["duplicate-statements"] = registered
            destination.write_text(json.dumps(artifact_manifest))
            statements = next(
                item for item in manifest.nodes
                if item.node_id == "jin-polynomial-bound"
            )
            with self.assertRaisesRegex(
                route_validation.RouteValidationError,
                "exactly one.*artifact.*path",
            ):
                route_validation.validate_source_provenance_metadata(
                    root, manifest, statements
                )

    @unittest.skipUnless(SOURCE_ROOT.is_dir(), "verified temporary Jin source is absent")
    def test_git_metadata_matches_verified_upstream_bytes(self) -> None:
        manifest = load_manifest()
        for node in manifest["nodes"]:
            if node["provenance_kind"] != "source":
                continue
            with self.subTest(node_id=node["node_id"]):
                _, relative, start, end = git_locator_parts(node["source_locator"])
                source = (SOURCE_ROOT / relative).read_bytes()
                self.assertEqual(node["source_file_sha256"], sha256_bytes(source))
                self.assertEqual(node["source_line_count"], len(source.splitlines()))
                self.assertEqual(
                    node["source_excerpt_sha256"],
                    sha256_bytes(excerpt_bytes(source, start, end)),
                )

    def test_declaration_type_artifacts_exist_and_match_normalized_digests(self) -> None:
        manifest = load_manifest()
        expected_paths = set()
        for node in manifest["nodes"]:
            path = Path(node["declaration_type_path"])
            expected_paths.add(path)
            self.assertEqual(path.parent, TYPE_ROOT)
            source = (REPO / path).read_text(encoding="utf-8")
            self.assertEqual(
                route_validation.normalized_type_sha256(source),
                node["statement_sha256"],
            )
        self.assertEqual(len(expected_paths), len(manifest["nodes"]))
        self.assertEqual(
            {path.relative_to(REPO) for path in (REPO / TYPE_ROOT).glob("*.txt")},
            expected_paths,
        )

    def test_historical_source_map_and_receipts_are_unchanged(self) -> None:
        source_map = json.loads((REPO / MANIFEST_PATH.with_name("source-map.json")).read_text())
        rows = source_map["rows"]
        self.assertEqual([row["row_id"] for row in rows], [
            "jin-max-polynomial-modulus",
            "jin-polynomial-bound",
            "jin-terminal-crouzeix",
        ])
        self.assertEqual([row["receipt_sha256"] for row in rows], list(HISTORICAL_RECEIPTS.values()))
        for relative, expected in HISTORICAL_RECEIPTS.items():
            self.assertEqual(sha256_bytes((REPO / relative).read_bytes()), expected)

    def test_route_has_no_ls_or_harp_imports(self) -> None:
        closure = route_validation.active_local_closure(
            REPO / "formalization/lean", "CrouzeixJin"
        )
        forbidden = [
            module for module in closure
            if module.startswith(("Crouzeix.LoristSchwenninger", "Crouzeix.Harp"))
        ]
        self.assertEqual(forbidden, [])

    def test_active_wrappers_use_route_terminal_and_rational_bridge(self) -> None:
        final = (
            REPO / "formalization/lean/CrouzeixConjecture/FinalTheorems.lean"
        ).read_text()
        hilbert = (
            REPO / "formalization/lean/CrouzeixConjecture/HilbertSpace.lean"
        ).read_text()
        spectral = (
            REPO / "formalization/lean/CrouzeixConjecture/HilbertSpectralSet.lean"
        ).read_text()
        bridge = REPO / "formalization/lean/Crouzeix/Jin/RationalBridge.lean"
        terminal = (
            REPO / "formalization/lean/Crouzeix/Jin/Terminal.lean"
        ).read_text()
        max_polynomial = (
            REPO / "formalization/lean/Crouzeix/Jin/MaxPolynomialModulus.lean"
        ).read_text()

        self.assertTrue(bridge.is_file())
        self.assertTrue(terminal.startswith("module\n\npublic import"))
        self.assertIn("@[expose] public section", terminal)
        self.assertTrue(max_polynomial.startswith("module\n\npublic import"))
        self.assertIn("@[expose] public section", max_polynomial)
        self.assertIn("import Crouzeix.Jin.Terminal", final)
        self.assertIn("import Crouzeix.Jin.RationalBridge", final)
        self.assertRegex(
            final,
            r"jinFinalCrouzeixConjecture[\s\S]+?:=\s*crouzeixConjecture A p",
        )
        self.assertNotIn("holomorphicCrouzeixRationalBound A r hfree", final)
        self.assertEqual(final.count("jinHolomorphicCrouzeixRationalBound A r hfree"), 2)
        self.assertNotIn("jinFinalCrouzeixConjecture", hilbert)
        self.assertNotIn("jinFinalCrouzeixConjecture", spectral)
        self.assertIn("fun d => crouzeixConjecture (n := Fin d)", hilbert)
        self.assertIn("fun d => crouzeixConjecture (n := Fin d)", spectral)

    def test_python_and_schema_reject_opaque_git_path_fragment(self) -> None:
        malformed = f"git:{SOURCE_COMMIT}:Lean/Foo.lean#opaque#L1-L1"
        with self.assertRaisesRegex(route_validation.RouteValidationError, "Git path|locator"):
            route_validation._source_locator(malformed, "source locator")
        schema = json.loads(
            (REPO / "labs/crouzeix_proof_reproduction/schemas/route_manifest.schema.json")
            .read_text(encoding="utf-8")
        )
        git_pattern = schema["$defs"]["locator"]["oneOf"][1]["pattern"]
        self.assertIsNone(re.fullmatch(git_pattern, malformed))

    def test_validate_reports_mapped_incomplete_without_publishing(self) -> None:
        before = subprocess.run(
            ["git", "status", "--short"],
            cwd=REPO, text=True, capture_output=True, check=True,
        ).stdout
        allowed = subprocess.run(
            ["python3", str(SCRIPT), "validate", "--route", "jin", "--allow-unpublished"],
            cwd=REPO, text=True, capture_output=True, check=False,
        )
        default = subprocess.run(
            ["python3", str(SCRIPT), "validate", "--route", "jin"],
            cwd=REPO, text=True, capture_output=True, check=False,
        )
        after = subprocess.run(
            ["git", "status", "--short"],
            cwd=REPO, text=True, capture_output=True, check=True,
        ).stdout
        payload = json.loads(allowed.stdout)

        self.assertEqual(allowed.returncode, 1)
        self.assertEqual(payload["claim_level"], "mapped")
        self.assertEqual(payload["status"], "incomplete")
        self.assertNotEqual(default.returncode, 0)
        self.assertEqual(before, after)
        self.assertFalse((REPO / "evidence/crouzeix_conjecture/routes/jin").exists())
        self.assertFalse((REPO / "evidence/crouzeix_conjecture/reviews/jin.json").exists())

    def test_manifest_schema_requires_remote_source_metadata(self) -> None:
        schema = json.loads(
            (REPO / "labs/crouzeix_proof_reproduction/schemas/route_manifest.schema.json")
            .read_text(encoding="utf-8")
        )
        node = schema["$defs"]["node"]
        self.assertIn("correspondence_kind", node["required"])
        self.assertIn("correspondence_kind", node["properties"])
        self.assertEqual(
            node["properties"]["correspondence_kind"]["enum"],
            [
                "direct-source",
                "compatibility-port",
                "structural-refactor",
                "derived-extraction",
                "shared-foundation",
                "reused-route",
            ],
        )
        for field in (
            "source_archive_sha256",
            "source_file_sha256",
            "source_excerpt_sha256",
            "source_line_count",
        ):
            self.assertIn(field, node["required"])
            self.assertIn(field, node["properties"])
        locator = schema["$defs"]["locator"]
        self.assertIn("git:", json.dumps(locator))


if __name__ == "__main__":
    unittest.main()
