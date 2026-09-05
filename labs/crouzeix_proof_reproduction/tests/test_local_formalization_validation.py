from __future__ import annotations

import inspect
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import local_formalization_evidence  # noqa: E402
import protocol  # noqa: E402
import route_validation  # noqa: E402
from labs.crouzeix_proof_reproduction.tests.test_local_formalization_evidence import (  # noqa: E402
    FakeExecutor,
    FORMALIZATIONS,
    make_workspace,
)

import local_formalization_validation  # noqa: E402


class LocalFormalizationValidationTests(unittest.TestCase):
    HEADER_FIELDS = local_formalization_evidence.HEADER.split("\t")

    def setUp(self) -> None:
        for name, side_effect in (
            (
                "inspect_route",
                lambda _root, route_id: route_validation.RouteValidationResult(
                    route_id,
                    "complete-local",
                    "complete",
                    route_validation.ROUTE_MANIFEST_PATHS[route_id].as_posix(),
                ),
            ),
            (
                "_read_json",
                lambda root, path, _label: __import__("json").loads(
                    (Path(root) / path).read_text(encoding="utf-8")
                ),
            ),
        ):
            patcher = mock.patch.object(
                local_formalization_evidence.route_validation,
                name,
                side_effect=side_effect,
            )
            patcher.start()
            self.addCleanup(patcher.stop)

    def publish_bundle(self, directory: str) -> tuple[Path, Path]:
        repository, _, _ = make_workspace(Path(directory).resolve())
        publication = local_formalization_evidence.publish_local_formalization_evidence(
            repository,
            executor=FakeExecutor(),
        )
        return repository, publication.artifact_root

    def manifest_table(self, repository: Path) -> tuple[list[str], list[list[str]]]:
        manifest = (
            repository / "evidence/crouzeix_conjecture/local_formalization/manifest.tsv"
        )
        lines = manifest.read_text(encoding="utf-8").splitlines()
        return (
            lines[0].split("\t"),
            [line.split("\t") for line in lines[1:]],
        )

    def write_manifest_table(
        self, repository: Path, header: list[str], rows: list[list[str]]
    ) -> None:
        manifest = (
            repository / "evidence/crouzeix_conjecture/local_formalization/manifest.tsv"
        )
        manifest.write_text(
            "\n".join(["\t".join(header), *("\t".join(row) for row in rows)]) + "\n",
            encoding="utf-8",
        )

    def field_index(self, name: str) -> int:
        return self.HEADER_FIELDS.index(name)

    def row_for(
        self, rows: list[list[str]], formalization_id: str = "harp-main-theorem"
    ) -> list[str]:
        id_index = self.field_index("formalization_id")
        return next(row for row in rows if row[id_index] == formalization_id)

    def assert_invalid(self, repository: Path, pattern: str) -> None:
        with self.assertRaisesRegex(
            (ValueError, protocol.ValidationError), pattern
        ):
            local_formalization_validation.validate_local_formalization_bundle(repository)

    def test_public_bundle_validator_is_read_only_by_construction(self) -> None:
        signature = inspect.signature(
            local_formalization_validation.validate_local_formalization_bundle
        )
        self.assertEqual(tuple(signature.parameters), ("repository_root",))
        self.assertEqual(
            tuple(
                local_formalization_validation.LocalBundleValidationResult.__dataclass_fields__
            ),
            ("status", "formalization_ids", "route_ids", "manifest_sha256", "manifest_path"),
        )

    def test_validator_accepts_exact_published_six_row_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, _, _ = make_workspace(Path(directory).resolve())
            local_formalization_evidence.publish_local_formalization_evidence(
                repository,
                executor=FakeExecutor(),
            )

            result = local_formalization_validation.validate_local_formalization_bundle(
                repository
            )

            self.assertEqual(result.status, "passed")
            self.assertEqual(result.formalization_ids, tuple(item[0] for item in FORMALIZATIONS))
            self.assertEqual(
                result.route_ids,
                ("harp", "jin", "lorist-schwenninger"),
            )
            manifest = (
                repository
                / "evidence/crouzeix_conjecture/local_formalization/manifest.tsv"
            )
            self.assertEqual(
                result.manifest_sha256,
                protocol.sha256_bytes(manifest.read_bytes()),
            )

    def test_validator_rejects_symlinked_bundle_member(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, _, _ = make_workspace(Path(directory).resolve())
            publication = local_formalization_evidence.publish_local_formalization_evidence(
                repository,
                executor=FakeExecutor(),
            )
            target = publication.artifact_root / "providers/jin.json"
            moved = publication.artifact_root / "providers/jin.real.json"
            target.rename(moved)
            target.symlink_to(moved.name)

            with self.assertRaisesRegex(ValueError, "symlink|unsafe|cannot inspect"):
                local_formalization_validation.validate_local_formalization_bundle(
                    repository
                )

    def test_validator_rejects_hard_linked_bundle_member(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, _, _ = make_workspace(Path(directory).resolve())
            publication = local_formalization_evidence.publish_local_formalization_evidence(
                repository,
                executor=FakeExecutor(),
            )
            target = publication.artifact_root / "providers/jin.json"
            backup = publication.artifact_root / "providers/jin.backup.json"
            target.rename(backup)
            os.link(backup, target)

            with self.assertRaisesRegex(ValueError, "hard link|exactly one hard link"):
                local_formalization_validation.validate_local_formalization_bundle(
                    repository
                )

    def test_validator_rejects_symlinked_or_hardlinked_formalization_module(self) -> None:
        for kind in ("symlink", "hardlink"):
            with self.subTest(kind=kind), tempfile.TemporaryDirectory() as directory:
                repository, _ = self.publish_bundle(directory)
                module = repository / FORMALIZATIONS[0][4]
                replacement = repository / f"{kind}-module.lean"
                module.rename(replacement)
                if kind == "symlink":
                    module.symlink_to(replacement)
                else:
                    os.link(replacement, module)

                self.assert_invalid(repository, "symlink|hard link|exactly one hard link")

    def test_validator_rejects_route_snapshot_drift_and_cross_route_substitution(self) -> None:
        for kind in ("manifest", "receipt", "review"):
            for case in ("drift", "substitution"):
                with self.subTest(kind=kind, case=case), tempfile.TemporaryDirectory() as directory:
                    repository, publication_root = self.publish_bundle(directory)
                    target = publication_root / f"routes/harp.{kind}.json"
                    if case == "drift":
                        target.write_bytes(target.read_bytes() + b" ")
                    else:
                        target.write_bytes(
                            (publication_root / f"routes/jin.{kind}.json").read_bytes()
                        )
                    self.assert_invalid(repository, "digest|mismatch|differs")

    def test_validator_rejects_manifest_field_mutation_matrix(self) -> None:
        cases = [
            ("schema_version", "schema_version", "wrong/v1", "spec row mismatch"),
            ("formalization_id", "formalization_id", "wrong-id", "spec row mismatch"),
            ("route_id", "route_id", "jin", "spec row mismatch"),
            ("source_node_id", "source_node_id", "wrong-node", "spec row mismatch"),
            ("declaration_name", "declaration_name", "Wrong.name", "spec row mismatch"),
            (
                "module_path",
                "module_path",
                "formalization/lean/Crouzeix/Harp/Consequences.lean",
                "spec row mismatch",
            ),
            ("module_sha256", "module_sha256", "0" * 64, "module digest mismatch"),
            (
                "route_manifest_path",
                "route_manifest_path",
                "evidence/crouzeix_conjecture/local_formalization/routes/harp.receipt.json",
                "route artifact path mismatch",
            ),
            (
                "route_manifest_sha256",
                "route_manifest_sha256",
                "0" * 64,
                "route artifact digest mismatch",
            ),
            (
                "route_receipt_path",
                "route_receipt_path",
                "evidence/crouzeix_conjecture/local_formalization/routes/harp.review.json",
                "route artifact path mismatch",
            ),
            (
                "route_receipt_sha256",
                "route_receipt_sha256",
                "0" * 64,
                "route artifact digest mismatch",
            ),
            (
                "route_review_path",
                "route_review_path",
                "evidence/crouzeix_conjecture/local_formalization/routes/harp.manifest.json",
                "route artifact path mismatch",
            ),
            (
                "route_review_sha256",
                "route_review_sha256",
                "0" * 64,
                "route artifact digest mismatch",
            ),
            (
                "build_command_path",
                "build_command_path",
                "evidence/crouzeix_conjecture/local_formalization/providers/harp.json",
                "schema mismatch|aliases unrelated",
            ),
            ("build_command_sha256", "build_command_sha256", "0" * 64, "digest mismatch"),
            (
                "build_stdout_path",
                "build_stdout_path",
                "evidence/crouzeix_conjecture/local_formalization/build/stderr.log",
                "aggregate build artifact fields changed|aliases unrelated",
            ),
            ("build_stdout_sha256", "build_stdout_sha256", "0" * 64, "digest mismatch"),
            (
                "build_stderr_path",
                "build_stderr_path",
                "evidence/crouzeix_conjecture/local_formalization/build/stdout.log",
                "aggregate build artifact fields changed|aliases unrelated",
            ),
            ("build_stderr_sha256", "build_stderr_sha256", "0" * 64, "digest mismatch"),
            (
                "axiom_audit_path",
                "axiom_audit_path",
                "evidence/crouzeix_conjecture/local_formalization/axioms/harp-closed-numerical-range.txt",
                "aliases unrelated|axiom audit",
            ),
            ("axiom_audit_sha256", "axiom_audit_sha256", "0" * 64, "digest mismatch"),
            (
                "allowed_axioms",
                "allowed_axioms",
                "Classical.choice,Quot.sound",
                "allowed_axioms policy mismatch",
            ),
            (
                "observed_axioms",
                "observed_axioms",
                "Classical.choice,unsafeAxiom",
                "observed_axioms policy mismatch",
            ),
            (
                "provider_independence_path",
                "provider_independence_path",
                "evidence/crouzeix_conjecture/local_formalization/providers/jin.json",
                "aliases unrelated|provider report mismatch",
            ),
            (
                "provider_independence_sha256",
                "provider_independence_sha256",
                "0" * 64,
                "digest mismatch",
            ),
            (
                "lean_toolchain",
                "lean_toolchain",
                "leanprover/lean4:v4.31.0",
                "Lean toolchain identity mismatch",
            ),
            (
                "lean_toolchain_sha256",
                "lean_toolchain_sha256",
                "0" * 64,
                "Lean toolchain digest mismatch",
            ),
            (
                "lake_manifest_sha256",
                "lake_manifest_sha256",
                "0" * 64,
                "Lake manifest digest mismatch",
            ),
            ("status", "status", "blocked", "status must be passed"),
        ]
        aggregate_fields = {
            "build_command_path",
            "build_command_sha256",
            "build_stdout_path",
            "build_stdout_sha256",
            "build_stderr_path",
            "build_stderr_sha256",
        }
        route_shared_fields = {
            "provider_independence_sha256",
        }
        with tempfile.TemporaryDirectory() as directory:
            base_repository, _ = self.publish_bundle(directory)
            _, base_rows = self.manifest_table(base_repository)
            base_map = {
                row[self.field_index("formalization_id")]: list(row) for row in base_rows
            }

        for case_name, field, value, pattern in cases:
            with self.subTest(field=field, case=case_name), tempfile.TemporaryDirectory() as directory:
                repository, _ = self.publish_bundle(directory)
                header, rows = self.manifest_table(repository)
                index = self.field_index(field)
                if field in aggregate_fields:
                    for row in rows:
                        row[index] = value
                elif field in route_shared_fields:
                    route_index = self.field_index("route_id")
                    for row in rows:
                        if row[route_index] == "harp":
                            row[index] = value
                else:
                    self.row_for(rows)[index] = value
                if field == "build_stdout_path":
                    digest_index = self.field_index("build_stdout_sha256")
                    for row in rows:
                        row[digest_index] = base_map[row[self.field_index("formalization_id")]][
                            self.field_index("build_stderr_sha256")
                        ]
                if field == "build_stderr_path":
                    digest_index = self.field_index("build_stderr_sha256")
                    for row in rows:
                        row[digest_index] = base_map[row[self.field_index("formalization_id")]][
                            self.field_index("build_stdout_sha256")
                        ]
                if field == "axiom_audit_path":
                    row = self.row_for(rows)
                    row[self.field_index("axiom_audit_sha256")] = base_map[
                        "harp-closed-numerical-range"
                    ][self.field_index("axiom_audit_sha256")]
                if field == "provider_independence_path":
                    row = self.row_for(rows)
                    row[self.field_index("provider_independence_sha256")] = base_map[
                        "jin-main-theorem"
                    ][self.field_index("provider_independence_sha256")]
                self.write_manifest_table(repository, header, rows)
                self.assert_invalid(repository, pattern)

    def test_validator_rejects_missing_row_member_and_alias_surfaces(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, publication_root = self.publish_bundle(directory)
            header, rows = self.manifest_table(repository)
            rows.pop()
            self.write_manifest_table(repository, header, rows)
            self.assert_invalid(repository, "row count mismatch")

        for relative in (
            "routes/harp.manifest.json",
            "routes/jin.receipt.json",
            "routes/lorist-schwenninger.review.json",
            "providers/harp.json",
            "axioms/harp-main-theorem.txt",
        ):
            with self.subTest(missing_member=relative), tempfile.TemporaryDirectory() as directory:
                repository, publication_root = self.publish_bundle(directory)
                (publication_root / relative).unlink()
                self.assert_invalid(repository, "roster differs|escapes local bundle")

        with tempfile.TemporaryDirectory() as directory:
            repository, publication_root = self.publish_bundle(directory)
            header, rows = self.manifest_table(repository)
            for row in rows:
                row[self.field_index("build_stdout_path")] = row[
                    self.field_index("build_stderr_path")
                ]
                row[self.field_index("build_stdout_sha256")] = row[
                    self.field_index("build_stderr_sha256")
                ]
            self.write_manifest_table(repository, header, rows)
            self.assert_invalid(repository, "aliases unrelated")

        with tempfile.TemporaryDirectory() as directory:
            repository, publication_root = self.publish_bundle(directory)
            header, rows = self.manifest_table(repository)
            jin_bytes = (
                publication_root / "routes/jin.review.json"
            ).read_bytes()
            (publication_root / "routes/harp.review.json").write_bytes(jin_bytes)
            (
                repository / "evidence/crouzeix_conjecture/reviews/harp.json"
            ).write_bytes(jin_bytes)
            route_manifest = (
                repository
                / "labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json"
            )
            manifest_payload = json.loads(route_manifest.read_text(encoding="utf-8"))
            digest = protocol.sha256_bytes(jin_bytes)
            manifest_payload["review_sha256"] = digest
            route_manifest.write_text(
                json.dumps(
                    manifest_payload,
                    sort_keys=True,
                    separators=(",", ":"),
                    ensure_ascii=False,
                    allow_nan=False,
                )
                + "\n",
                encoding="utf-8",
            )
            updated_manifest_bytes = route_manifest.read_bytes()
            (publication_root / "routes/harp.manifest.json").write_bytes(updated_manifest_bytes)
            route_index = self.field_index("route_id")
            for row in rows:
                if row[route_index] != "harp":
                    continue
                row[self.field_index("route_manifest_sha256")] = protocol.sha256_bytes(
                    updated_manifest_bytes
                )
                row[self.field_index("route_review_sha256")] = digest
            self.write_manifest_table(repository, header, rows)
            self.assert_invalid(
                repository,
                "artifact digest aliases unrelated|route artifact digest mismatch",
            )
