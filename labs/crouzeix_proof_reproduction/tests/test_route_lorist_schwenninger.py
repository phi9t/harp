from __future__ import annotations

import copy
import json
import re
import unittest
from pathlib import Path

from labs.crouzeix_proof_reproduction import route_validation


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


class LoristSchwenningerArtifactManifestTests(unittest.TestCase):
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
            with self.subTest(field=field):
                original = manifest[field]
                patched = ARTIFACT_MANIFEST_PATH.read_text(encoding="utf-8")
                old_text = json.dumps(original)
                new_text = json.dumps(mutated)
                if old_text not in patched:
                    self.fail(f"could not locate serialized field value for {field}")
                ARTIFACT_MANIFEST_PATH.write_text(
                    patched.replace(old_text, new_text, 1),
                    encoding="utf-8",
                )
                try:
                    with self.assertRaisesRegex(
                        route_validation.RouteValidationError, pattern
                    ):
                        route_validation.validate_source_provenance_metadata(
                            REPO, parsed, node
                        )
                finally:
                    ARTIFACT_MANIFEST_PATH.write_text(
                        json.dumps(manifest, indent=2) + "\n",
                        encoding="utf-8",
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
