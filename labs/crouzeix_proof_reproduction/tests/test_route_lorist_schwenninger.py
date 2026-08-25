from __future__ import annotations

import copy
import json
import re
import tempfile
import unittest
from pathlib import Path

from labs.crouzeix_proof_reproduction import ls_contract, route_validation


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

    def test_theorem_body_audits_require_exact_provider_calls(self) -> None:
        audits = ls_contract.audit_required_theorem_provider_calls(REPO)
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
