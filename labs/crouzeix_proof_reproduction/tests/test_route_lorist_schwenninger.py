from __future__ import annotations

import copy
import json
import re
import tempfile
import unittest
from pathlib import Path
from unittest import mock

from labs.crouzeix_proof_reproduction import ls_contract, route_validation

LAB = Path(__file__).resolve().parents[1]
import sys
if str(LAB) not in sys.path:
    sys.path.insert(0, str(LAB))

import ls_validation  # noqa: E402


REPO = Path(__file__).resolve().parents[3]
SCHEMA_PATH = (
    REPO
    / "labs/crouzeix_proof_reproduction/schemas/route_manifest.schema.json"
)
SOURCE_MANIFEST_PATH = (
    REPO / "evidence/crouzeix_conjecture/source_manifest.tsv"
)
ARTIFACT_MANIFEST_PATH = (
    REPO
    / "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/artifact-manifest.json"
)
ROUTE_MANIFEST_PATH = (
    REPO
    / "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/route-manifest.json"
)
SOURCE_GRAPH_PATH = ROUTE_MANIFEST_PATH.parent / "source-graph.json"
DECLARATION_TYPES_ROOT = ROUTE_MANIFEST_PATH.parent / "declaration-types"

SOURCE_ID = "LS-ARXIV-V1"
SOURCE_IDENTITY = "arxiv:2608.03841v1"
SOURCE_ARCHIVE_SHA256 = (
    "b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9"
)
SOURCE_FILE_SHA256 = (
    "20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a"
)
SOURCE_PATH = "CrouzeixConjecturev2.tex"
SOURCE_BYTES = 18_783
SOURCE_LINES = 281
SOURCE_LOCATOR = "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99"
SOURCE_IDENTITIES = sorted(
    [
        SOURCE_IDENTITY,
        f"sha256:{SOURCE_ARCHIVE_SHA256}",
        f"sha256:{SOURCE_FILE_SHA256}",
    ]
)
NODE_CORRESPONDENCE_KINDS = (
    ("ls-equation-one-terminal-bound", "direct-source"),
    ("ls-power-recurrence", "structural-refactor"),
    ("ls-scalar-contradiction", "direct-source"),
    ("ls-perturbation-lemma", "structural-refactor"),
    ("ls-double-layer-realization", "structural-refactor"),
    ("ls-terminal-crouzeix", "structural-refactor"),
)
DECLARATION_TYPE_FILES = {
    node_id: DECLARATION_TYPES_ROOT / f"{node_id}.txt"
    for node_id in ls_contract.NODE_ORDER
}


def load_schema() -> dict[str, object]:
    return json.loads(SCHEMA_PATH.read_text(encoding="utf-8"))


def schema_source_identity_pattern() -> re.Pattern[str]:
    schema = load_schema()
    pattern = schema["properties"]["source_identities"]["items"]["pattern"]
    return re.compile(pattern)


def schema_locator_patterns() -> tuple[re.Pattern[str], ...]:
    schema = load_schema()
    return tuple(
        re.compile(option["pattern"])
        for option in schema["$defs"]["locator"]["oneOf"]
    )


def schema_accepts_source_identity(value: str) -> bool:
    return schema_source_identity_pattern().fullmatch(value) is not None


def schema_accepts_locator(value: str) -> bool:
    return any(pattern.fullmatch(value) is not None for pattern in schema_locator_patterns())


def ls_manifest_payload() -> dict[str, object]:
    return {
        "schema_version": route_validation.MANIFEST_SCHEMA_VERSION,
        "route_id": "lorist-schwenninger",
        "claim_kind": "source-faithful",
        "aggregate_module": "CrouzeixLoristSchwenninger",
        "build_target": "CrouzeixLoristSchwenninger",
        "terminal_declaration": "CrouzeixConjecture.loristSchwenningerMainTheorem",
        "terminal_type_sha256": "c" * 64,
        "consequence_declarations": [
            "CrouzeixConjecture.loristSchwenningerMainTheorem"
        ],
        "source_identities": [
            SOURCE_IDENTITY,
            f"sha256:{SOURCE_ARCHIVE_SHA256}",
            f"sha256:{SOURCE_FILE_SHA256}",
        ],
        "shared_foundation_modules": ["CrouzeixLoristSchwenninger"],
        "module_closure": ["CrouzeixLoristSchwenninger"],
        "module_closure_sha256": "d" * 64,
        "allowed_axioms": list(route_validation.ALLOWED_AXIOMS),
        "review_path": "evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json",
        "review_sha256": None,
        "receipt_path": "evidence/crouzeix_conjecture/routes/lorist-schwenninger/receipt.json",
        "receipt_sha256": None,
        "nodes": [
            {
                "node_id": "ls-terminal-crouzeix",
                "role": "terminal",
                "declaration": "CrouzeixConjecture.loristSchwenningerMainTheorem",
                "module_path": "CrouzeixLoristSchwenninger.lean",
                "dependency_ids": [],
                "provenance_kind": "source",
                "correspondence_kind": "direct-source",
                "source_locator": SOURCE_LOCATOR,
                "source_archive_sha256": SOURCE_ARCHIVE_SHA256,
                "source_file_sha256": SOURCE_FILE_SHA256,
                "source_excerpt_sha256": "e" * 64,
                "source_line_count": SOURCE_LINES,
                "reused_from_route": None,
                "reused_node_id": None,
                "declaration_type_path": "evidence/crouzeix_conjecture/routes/lorist-schwenninger/types/main.txt",
                "statement_sha256": "c" * 64,
            }
        ],
    }


def parsed_ls_manifest() -> route_validation.RouteManifest:
    return route_validation.parse_route_manifest(ls_manifest_payload())


class LoristSchwenningerSourceLocatorTests(unittest.TestCase):
    def test_source_locator_parses_exact_arxiv_identity(self) -> None:
        kind, identity, path, start, end = route_validation._source_locator(
            SOURCE_LOCATOR,
            "source locator",
        )
        self.assertEqual(
            (kind, identity, path, start, end),
            ("arxiv", SOURCE_IDENTITY, SOURCE_PATH, 67, 99),
        )

    def test_source_locator_accepts_generic_arxiv_syntax(self) -> None:
        kind, identity, path, start, end = route_validation._source_locator(
            "arxiv:2608.03841v2:appendix.tex#L1-L2",
            "source locator",
        )
        self.assertEqual(
            (kind, identity, path, start, end),
            ("arxiv", "arxiv:2608.03841v2", "appendix.tex", 1, 2),
        )

    def test_source_locator_rejects_malformed_arxiv_locator_shapes(self) -> None:
        cases = (
            "arxiv:2608.03841:CrouzeixConjecturev2.tex#L67-L99",
            "arxiv:2608.03841v0:CrouzeixConjecturev2.tex#L67-L99",
            "arxiv:2608.03841v1:/CrouzeixConjecturev2.tex#L67-L99",
            "arxiv:2608.03841v1:../CrouzeixConjecturev2.tex#L67-L99",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L0-L99",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L99-L67",
            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99#L100-L101",
        )
        for locator in cases:
            with self.subTest(locator=locator):
                with self.assertRaises(route_validation.RouteValidationError):
                    route_validation._source_locator(locator, "source locator")

    def test_source_locator_rejects_aliasing_path_forms_for_all_locator_kinds(self) -> None:
        cases = (
            "dir//file.tex#L1-L2",
            "./dir/file.tex#L1-L2",
            "git:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:dir//file.tex#L1-L2",
            "git:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:./dir/file.tex#L1-L2",
            "arxiv:2608.03841v1:dir//file.tex#L1-L2",
            "arxiv:2608.03841v1:./dir/file.tex#L1-L2",
        )
        for locator in cases:
            with self.subTest(locator=locator):
                with self.assertRaises(route_validation.RouteValidationError):
                    route_validation._source_locator(locator, "source locator")


class LoristSchwenningerSchemaParityTests(unittest.TestCase):
    def test_schema_accepts_exact_ls_arxiv_identities_and_locator(self) -> None:
        payload = ls_manifest_payload()
        self.assertTrue(schema_accepts_source_identity(payload["source_identities"][0]))
        self.assertTrue(schema_accepts_source_identity(payload["source_identities"][1]))
        self.assertTrue(schema_accepts_source_identity(payload["source_identities"][2]))
        self.assertTrue(schema_accepts_locator(payload["nodes"][0]["source_locator"]))
        self.assertTrue(schema_accepts_locator("arxiv:2608.03841v2:appendix.tex#L1-L2"))

    def test_schema_rejects_each_malformed_arxiv_shape(self) -> None:
        cases = (
            ("source_identity", "arxiv:2608.03841"),
            ("source_identity", "arxiv:2608.03841v0"),
            ("nodes.0.source_locator", "arxiv:2608.03841v1:/CrouzeixConjecturev2.tex#L67-L99"),
            ("nodes.0.source_locator", "arxiv:2608.03841v1:../CrouzeixConjecturev2.tex#L67-L99"),
            ("nodes.0.source_locator", "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L0-L99"),
            ("nodes.0.source_locator", "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L67-L99#L100-L101"),
        )
        for field, value in cases:
            with self.subTest(field=field, value=value):
                if field == "source_identity":
                    self.assertFalse(schema_accepts_source_identity(value))
                else:
                    self.assertFalse(schema_accepts_locator(value))

    def test_schema_rejects_aliasing_path_forms_for_all_locator_kinds(self) -> None:
        cases = (
            "dir//file.tex#L1-L2",
            "./dir/file.tex#L1-L2",
            "git:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:dir//file.tex#L1-L2",
            "git:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa:./dir/file.tex#L1-L2",
            "arxiv:2608.03841v1:dir//file.tex#L1-L2",
            "arxiv:2608.03841v1:./dir/file.tex#L1-L2",
        )
        for value in cases:
            with self.subTest(value=value):
                self.assertFalse(schema_accepts_locator(value))


class LoristSchwenningerArtifactManifestTests(unittest.TestCase):
    def _write_isolated_repo(self, directory: str) -> Path:
        repo_root = Path(directory)
        source_manifest_target = repo_root / SOURCE_MANIFEST_PATH
        source_manifest_target.parent.mkdir(parents=True, exist_ok=True)
        source_manifest_target.write_text(
            SOURCE_MANIFEST_PATH.read_text(encoding="utf-8"),
            encoding="utf-8",
        )
        artifact_manifest_target = repo_root / ARTIFACT_MANIFEST_PATH.relative_to(REPO)
        artifact_manifest_target.parent.mkdir(parents=True, exist_ok=True)
        artifact_manifest_target.write_text(
            ARTIFACT_MANIFEST_PATH.read_text(encoding="utf-8"),
            encoding="utf-8",
        )
        return repo_root

    def test_artifact_manifest_matches_exact_closed_ls_payload(self) -> None:
        manifest = json.loads(ARTIFACT_MANIFEST_PATH.read_text(encoding="utf-8"))
        self.assertEqual(
            manifest,
            {
                "archive_sha256": SOURCE_ARCHIVE_SHA256,
                "manuscript_bytes": SOURCE_BYTES,
                "manuscript_path": SOURCE_PATH,
                "manuscript_sha256": SOURCE_FILE_SHA256,
                "line_count": SOURCE_LINES,
                "schema_version": "crouzeix-arxiv-artifact-manifest/v1",
                "source_id": SOURCE_ID,
                "source_identity": SOURCE_IDENTITY,
            },
        )

    def test_artifact_manifest_cross_checks_source_manifest_rows(self) -> None:
        rows = SOURCE_MANIFEST_PATH.read_text(encoding="utf-8").splitlines()
        self.assertIn(
            "\t".join(
                [
                    "crouzeix-source-receipt/v1",
                    "LS-ARXIV-V1-SOURCE-ARCHIVE",
                    SOURCE_ID,
                    "arxiv-artifact",
                    "source",
                    SOURCE_IDENTITY,
                    "https://export.arxiv.org/e-print/2608.03841v1",
                    "source.tar.gz",
                    "7330",
                    SOURCE_ARCHIVE_SHA256,
                    "-",
                    "2026-08-14",
                    "arxiv-nonexclusive",
                    "quotation-only",
                ]
            ),
            rows,
        )
        self.assertIn(
            "\t".join(
                [
                    "crouzeix-source-receipt/v1",
                    "LS-ARXIV-V1-TEX",
                    SOURCE_ID,
                    "arxiv-artifact",
                    "manuscript",
                    SOURCE_IDENTITY,
                    "https://export.arxiv.org/e-print/2608.03841v1",
                    SOURCE_PATH,
                    str(SOURCE_BYTES),
                    SOURCE_FILE_SHA256,
                    "-",
                    "2026-08-14",
                    "arxiv-nonexclusive",
                    "quotation-only",
                ]
            ),
            rows,
        )

    def test_artifact_manifest_mutations_fail_closed(self) -> None:
        self.assertTrue(ARTIFACT_MANIFEST_PATH.is_file())
        manifest = json.loads(ARTIFACT_MANIFEST_PATH.read_text(encoding="utf-8"))
        parsed = parsed_ls_manifest()
        node = parsed.nodes[0]
        expectations = (
            (
                "source_identity",
                "arxiv:2608.03841v2",
                "source_identity mismatch|source manifest .* immutable_identity mismatch",
            ),
            ("source_id", "LS-ARXIV-V2", "source_id mismatch"),
            ("manuscript_path", "Wrong.tex", "manuscript path mismatch"),
            ("manuscript_bytes", 0, "byte count mismatch"),
            ("line_count", 0, "line count mismatch"),
            ("archive_sha256", "0" * 64, "archive digest mismatch"),
            ("manuscript_sha256", "0" * 64, "manuscript digest mismatch"),
        )
        for field, mutated, pattern in expectations:
            with self.subTest(field=field), tempfile.TemporaryDirectory() as directory:
                repo_root = self._write_isolated_repo(directory)
                artifact_manifest_path = (
                    repo_root / ARTIFACT_MANIFEST_PATH.relative_to(REPO)
                )
                mutated_manifest = copy.deepcopy(manifest)
                mutated_manifest[field] = mutated
                artifact_manifest_path.write_text(
                    json.dumps(mutated_manifest, indent=2) + "\n",
                    encoding="utf-8",
                )
                self.assertEqual(
                    ARTIFACT_MANIFEST_PATH.read_text(encoding="utf-8"),
                    json.dumps(manifest, indent=2) + "\n",
                )
                with self.assertRaisesRegex(
                    route_validation.RouteValidationError, pattern
                ):
                    route_validation.validate_source_provenance_metadata(
                        repo_root, parsed, node
                    )

    def test_ls_provenance_validation_rejects_wrong_identity_and_wrong_path_after_generic_parse(self) -> None:
        payload = ls_manifest_payload()
        payload["source_identities"][0] = "arxiv:2608.03841v2"
        payload["source_identities"] = sorted(payload["source_identities"])
        payload["nodes"][0]["source_locator"] = "arxiv:2608.03841v2:appendix.tex#L1-L2"
        payload["nodes"][0]["source_line_count"] = 2
        manifest = route_validation.parse_route_manifest(payload)
        with self.assertRaisesRegex(
            route_validation.RouteValidationError,
            "exact sorted arXiv identity set|source identity|source path",
        ):
            route_validation.validate_source_provenance_metadata(
                REPO, manifest, manifest.nodes[0]
            )


class LoristSchwenningerContractTests(unittest.TestCase):
    def test_unpromoted_ls_manifest_is_authored_and_incomplete(self) -> None:
        result = route_validation.inspect_route(
            REPO, "lorist-schwenninger", allow_unpublished=True
        )
        self.assertEqual(result.claim_level, "authored")
        self.assertEqual(result.status, "incomplete")
        self.assertEqual(result.reason, "LS graph is not promoted")

    def test_route_manifest_matches_exact_task7_contract(self) -> None:
        payload = json.loads(ROUTE_MANIFEST_PATH.read_text(encoding="utf-8"))
        manifest = route_validation.parse_route_manifest(payload)
        closure = route_validation.active_local_closure(
            REPO / "formalization/lean",
            "CrouzeixLoristSchwenninger",
        )
        self.assertEqual(manifest.route_id, "lorist-schwenninger")
        self.assertEqual(manifest.claim_kind, "source-faithful")
        self.assertEqual(manifest.aggregate_module, "CrouzeixLoristSchwenninger")
        self.assertEqual(manifest.build_target, "CrouzeixLoristSchwenninger")
        self.assertEqual(
            manifest.terminal_declaration,
            "CrouzeixConjecture.loristSchwenningerMainTheorem",
        )
        self.assertEqual(manifest.consequence_declarations, ())
        self.assertEqual(list(manifest.source_identities), SOURCE_IDENTITIES)
        self.assertEqual(manifest.allowed_axioms, route_validation.ALLOWED_AXIOMS)
        self.assertEqual(
            manifest.review_path,
            "evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json",
        )
        self.assertIsNone(manifest.review_sha256)
        self.assertEqual(
            manifest.receipt_path,
            "evidence/crouzeix_conjecture/routes/lorist-schwenninger/receipt.json",
        )
        self.assertIsNone(manifest.receipt_sha256)
        self.assertEqual(tuple(node.node_id for node in manifest.nodes), ls_contract.NODE_ORDER)
        self.assertEqual(
            tuple(node.role for node in manifest.nodes),
            ("load-bearing", "load-bearing", "load-bearing", "load-bearing", "load-bearing", "terminal"),
        )
        self.assertEqual(
            tuple((node.node_id, node.correspondence_kind) for node in manifest.nodes),
            NODE_CORRESPONDENCE_KINDS,
        )
        self.assertEqual(
            tuple(node.declaration for node in manifest.nodes),
            tuple(node.declaration for node in ls_contract.NODES),
        )
        self.assertEqual(
            tuple(node.dependency_ids for node in manifest.nodes),
            tuple(node.dependencies for node in ls_contract.NODES),
        )
        self.assertEqual(
            tuple(node.source_locator for node in manifest.nodes),
            tuple(node.source_locator for node in ls_contract.NODES),
        )
        self.assertEqual(
            tuple(node.source_archive_sha256 for node in manifest.nodes),
            (SOURCE_ARCHIVE_SHA256,) * len(ls_contract.NODES),
        )
        self.assertEqual(
            tuple(node.source_file_sha256 for node in manifest.nodes),
            (SOURCE_FILE_SHA256,) * len(ls_contract.NODES),
        )
        self.assertEqual(
            tuple(node.source_line_count for node in manifest.nodes),
            (SOURCE_LINES,) * len(ls_contract.NODES),
        )
        self.assertEqual(manifest.module_closure, closure)
        self.assertEqual(
            manifest.module_closure_sha256,
            route_validation.string_roster_sha256(closure),
        )
        node_module_names = {
            route_validation._module_from_path(node.module_path) for node in manifest.nodes
        }
        self.assertEqual(
            manifest.shared_foundation_modules,
            tuple(module for module in closure if module not in node_module_names),
        )
        self.assertEqual(
            manifest.terminal_type_sha256,
            manifest.nodes[-1].statement_sha256,
        )

    def test_exactly_six_declaration_type_artifacts_exist_and_match_statement_hashes(self) -> None:
        self.assertEqual(
            sorted(path.name for path in DECLARATION_TYPES_ROOT.glob("*.txt")),
            sorted(f"{node_id}.txt" for node_id in ls_contract.NODE_ORDER),
        )
        manifest = route_validation.load_route_manifest(REPO, ROUTE_MANIFEST_PATH.relative_to(REPO))
        self.assertEqual(len(manifest.nodes), 6)
        for node in manifest.nodes:
            with self.subTest(node_id=node.node_id):
                path = DECLARATION_TYPE_FILES[node.node_id]
                text = path.read_text(encoding="utf-8")
                self.assertEqual(node.declaration_type_path, path.relative_to(REPO).as_posix())
                self.assertEqual(
                    node.statement_sha256,
                    route_validation.normalized_type_sha256(text),
                )
                self.assertTrue(text.endswith("\n"))
                self.assertEqual(text, text.strip() + "\n")

    def test_promoted_ls_authority_uses_shared_state_and_receipt_validation(self) -> None:
        promoted_rows = tuple(
            row
            for row in ls_validation.load_route_graph(SOURCE_GRAPH_PATH)
        )
        state = {"graph": promoted_rows, "promotion": {"schema_version": "fixture"}}
        selected = {
            row.node_id: {
                "attempt_path": (
                    "labs/crouzeix_proof_reproduction/formal_targets/"
                    f"lorist-schwenninger/proof-slices/{row.node_id}/attempt-999"
                )
            }
            for row in promoted_rows
        }
        with mock.patch.object(ls_validation, "load_route_state", return_value=state) as load, mock.patch.object(
            ls_validation, "validate_committed_receipts", return_value=selected
        ) as validate:
            roster = ls_validation.validated_route_authority_paths(REPO)

        load.assert_called_once()
        validate.assert_called_once()
        self.assertIn(
            Path("labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/artifact-manifest.json"),
            roster,
        )
        self.assertEqual(len(roster), 4 + 6 * len(ls_validation.ATTEMPT_FILES))

    def test_exact_node_authority_uses_canonical_terminal_dependency_only(self) -> None:
        by_id = ls_contract.BY_ID
        self.assertEqual(
            tuple(node.node_id for node in ls_contract.NODES),
            (
                "ls-equation-one-terminal-bound",
                "ls-power-recurrence",
                "ls-scalar-contradiction",
                "ls-perturbation-lemma",
                "ls-double-layer-realization",
                "ls-terminal-crouzeix",
            ),
        )
        self.assertEqual(
            by_id["ls-terminal-crouzeix"].dependencies,
            ("ls-double-layer-realization",),
        )
        self.assertEqual(
            by_id["ls-terminal-crouzeix"].legacy_dependencies,
            ("ls-perturbation-lemma", "ls-double-layer-realization"),
        )

    def test_source_claim_hashes_stay_separate_from_route_type_hashes(self) -> None:
        payload = ls_manifest_payload()
        payload["terminal_type_sha256"] = "d" * 64
        payload["nodes"][0]["statement_sha256"] = "c" * 64
        parsed = route_validation.parse_route_manifest(payload)
        self.assertNotEqual(
            ls_contract.BY_ID["ls-terminal-crouzeix"].statement_sha256,
            parsed.terminal_type_sha256,
        )
        self.assertEqual(parsed.nodes[0].statement_sha256, "c" * 64)
        self.assertEqual(parsed.terminal_type_sha256, "d" * 64)

    def test_reviewed_source_locators_cover_the_actual_claims(self) -> None:
        manifest = route_validation.load_route_manifest(
            REPO, ROUTE_MANIFEST_PATH.relative_to(REPO)
        )
        by_id = {node.node_id: node for node in manifest.nodes}
        expected = {
            "ls-double-layer-realization": (
                "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L116-L128",
                "067ea553be5b993a7695aa33e666d6ef05a7831e430aa8325716c25769b62a85",
            ),
            "ls-terminal-crouzeix": (
                "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L107-L129",
                "35e1efc2b96f9c5297fa841aac08c34f92c3a1a53a78ea91c73481ff6469becd",
            ),
        }
        for node_id, (locator, excerpt_sha256) in expected.items():
            with self.subTest(node_id=node_id):
                self.assertEqual(ls_contract.BY_ID[node_id].source_locator, locator)
                self.assertEqual(by_id[node_id].source_locator, locator)
                self.assertEqual(by_id[node_id].source_excerpt_sha256, excerpt_sha256)

        payload = json.loads(ROUTE_MANIFEST_PATH.read_text(encoding="utf-8"))
        for node in payload["nodes"]:
            if node["node_id"] not in expected:
                continue
            old_locator = {
                "ls-double-layer-realization": (
                    "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L100-L124"
                ),
                "ls-terminal-crouzeix": (
                    "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128"
                ),
            }[node["node_id"]]
            node["source_locator"] = old_locator
        stale = route_validation.parse_route_manifest(payload)
        with self.assertRaisesRegex(
            route_validation.RouteValidationError, "reviewed source locator"
        ):
            route_validation._validate_manifest_semantics(
                REPO, REPO / "formalization/lean", stale
            )

    def test_theorem_body_audits_require_exact_provider_calls(self) -> None:
        audits = ls_validation.audit_required_theorem_provider_calls(REPO)
        self.assertEqual(
            audits["ls-power-recurrence"],
            (
                "recurrence_lower_bound",
                "recurrence_difference_lower_bound",
            ),
        )
        self.assertEqual(
            audits["ls-perturbation-lemma"],
            (
                "equation_three_lower_bound",
                "displacementSq_le",
                "scalar_endpoint_le_two",
            ),
        )
        self.assertEqual(
            audits["ls-double-layer-realization"],
            (
                "dilationDataOfParametricPolynomial",
                "norm_target_le_two",
            ),
        )
        self.assertEqual(
            audits["ls-terminal-crouzeix"],
            (
                "norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary",
                "norm_polynomialEval_le_of_tendsto",
                "tendsto_maxPolynomialModulusOnSet_of_outerApproximation",
            ),
        )
