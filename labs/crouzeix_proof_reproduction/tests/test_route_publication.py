from __future__ import annotations

import json
import inspect
import os
import time
import shutil
import subprocess
import sys
import tempfile
import unittest
from dataclasses import FrozenInstanceError, is_dataclass
from pathlib import Path
from unittest import mock

from labs.crouzeix_proof_reproduction import (
    execution_ledger,
    proof_evidence,
    route_publication,
    route_validation,
)
from labs.crouzeix_proof_reproduction.tests.test_route_validation import (
    RouteFixture,
    write_json,
)


REPO = Path(__file__).resolve().parents[3]


def make_unpublished_fixture(test_case: unittest.TestCase) -> RouteFixture:
    fixture = RouteFixture("jin")
    test_case.addCleanup(fixture.cleanup)
    for node in fixture.manifest["nodes"]:
        old = fixture.repo / node["declaration_type_path"]
        new = fixture.repo / "evidence/crouzeix_conjecture/declaration_types/jin" / old.name
        new.parent.mkdir(parents=True, exist_ok=True)
        shutil.move(old, new)
        node["declaration_type_path"] = new.relative_to(fixture.repo).as_posix()
    shutil.rmtree(fixture.repo / "evidence/crouzeix_conjecture/routes/jin")
    (fixture.repo / fixture.review_path).unlink()
    fixture.manifest["receipt_sha256"] = None
    fixture.manifest["review_sha256"] = None
    fixture.rewrite_manifest()
    script = fixture.repo / "scripts/check_lean_library.sh"
    script.parent.mkdir(parents=True, exist_ok=True)
    script.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
    script.chmod(0o755)
    (fixture.repo / "formalization/lean/lakefile.toml").write_text(
        "name = \"fixture\"\n", encoding="utf-8"
    )
    artifact_manifest = (
        fixture.repo
        / "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/artifact-manifest.json"
    )
    write_json(artifact_manifest, {"schema_version": "fixture/v1"})
    subprocess.run(["git", "add", "-A"], cwd=fixture.repo, check=True)
    subprocess.run(
        ["git", "commit", "-q", "-m", "add wrapper"],
        cwd=fixture.repo,
        env={
            **os.environ,
            "GIT_AUTHOR_NAME": "Fixture",
            "GIT_AUTHOR_EMAIL": "fixture@example.invalid",
            "GIT_COMMITTER_NAME": "Fixture",
            "GIT_COMMITTER_EMAIL": "fixture@example.invalid",
        },
        check=True,
    )
    candidate_commit = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=fixture.repo,
        text=True,
        capture_output=True,
        check=True,
    ).stdout.strip()
    candidate_tree = subprocess.run(
        ["git", "rev-parse", "HEAD^{tree}"],
        cwd=fixture.repo,
        text=True,
        capture_output=True,
        check=True,
    ).stdout.strip()
    ledger = fixture.repo / route_publication.LEDGER_PATH
    ledger.parent.mkdir(parents=True, exist_ok=True)
    ledger.write_text(
        execution_ledger.HEADER
        + "\n"
        + (
            "cpfr-085\tCPFR-085\tlocally-verified\tfeat/fixture\t"
            f".worktrees/fixture\t{fixture.commit}\t{candidate_commit}\t"
            f"fixture-verifier\t0\tsha256:{'1' * 64}\t\ttree={candidate_tree}\n"
        ),
        encoding="utf-8",
    )
    return fixture


class RecordingExecutor:
    def __init__(self, *, axiom_output: bytes | None = None) -> None:
        self.calls: list[tuple[list[str], Path, dict[str, str], int, int]] = []
        self.axiom_output = axiom_output

    def __call__(
        self,
        argv: list[str],
        cwd: Path,
        env: dict[str, str],
        timeout: int,
        cap: int,
    ) -> route_publication.CommandResult:
        self.calls.append((argv, cwd, env, timeout, cap))
        if argv[:2] == ["scripts/check_lean_library.sh", "CrouzeixJin"]:
            return route_publication.CommandResult(0, b"route build ok\n", b"")
        if argv[:3] == ["lake", "env", "lean"]:
            output = self.axiom_output
            if output is None:
                audit_source = (cwd / argv[-1]).read_text(encoding="utf-8")
                declaration = audit_source.split("#print axioms ", 1)[1].strip()
                output = f"'{declaration}' depends on axioms: [propext]\n".encode()
            return route_publication.CommandResult(0, output, b"")
        return route_publication.CommandResult(2, b"", b"unexpected command")


def write_review_candidate(fixture: RouteFixture, *, mutate: dict[str, object] | None = None) -> Path:
    receipt = json.loads((fixture.repo / fixture.receipt_path).read_text(encoding="utf-8"))
    review = dict(fixture.review)
    review["reviewed_commit"] = receipt["candidate_commit"]
    review["reviewed_tree"] = receipt["candidate_tree"]
    review["review_sha256"] = "0" * 64
    if mutate:
        review.update(mutate)
    review["review_sha256"] = route_validation.self_digest(review, "review_sha256")
    path = fixture.repo / route_validation.ROUTE_REVIEW_CANDIDATE_PARENT / f"{fixture.route_id}.json"
    write_json(path, review)
    return path


class RoutePublicationApiTests(unittest.TestCase):
    def test_route_publication_is_frozen_public_receipt_dataclass(self) -> None:
        self.assertTrue(is_dataclass(route_publication.RoutePublication))
        publication = route_publication.RoutePublication(
            route_id="jin",
            artifact_root=Path("evidence/crouzeix_conjecture/routes/jin"),
            receipt_path=Path("evidence/crouzeix_conjecture/routes/jin/receipt.json"),
            receipt_sha256="a" * 64,
        )

        self.assertEqual(publication.route_id, "jin")
        self.assertEqual(
            publication.artifact_root,
            Path("evidence/crouzeix_conjecture/routes/jin"),
        )
        self.assertEqual(
            publication.receipt_path,
            Path("evidence/crouzeix_conjecture/routes/jin/receipt.json"),
        )
        self.assertEqual(publication.receipt_sha256, "a" * 64)
        with self.assertRaises(FrozenInstanceError):
            publication.route_id = "harp"  # type: ignore[misc]

    def test_review_publication_is_frozen_public_review_dataclass(self) -> None:
        self.assertTrue(is_dataclass(route_publication.ReviewPublication))
        publication = route_publication.ReviewPublication(
            route_id="jin",
            review_path=Path("evidence/crouzeix_conjecture/reviews/jin.json"),
            review_sha256="b" * 64,
        )

        self.assertEqual(publication.route_id, "jin")
        self.assertEqual(
            publication.review_path,
            Path("evidence/crouzeix_conjecture/reviews/jin.json"),
        )
        self.assertEqual(publication.review_sha256, "b" * 64)
        with self.assertRaises(FrozenInstanceError):
            publication.route_id = "harp"  # type: ignore[misc]

    def test_publish_route_receipt_signature_is_public_and_create_only(self) -> None:
        signature = inspect.signature(route_publication.publish_route_receipt)

        self.assertEqual(
            list(signature.parameters),
            ["repository_root", "route_id", "executor"],
        )
        self.assertEqual(
            signature.parameters["repository_root"].annotation,
            Path,
        )
        self.assertEqual(
            signature.parameters["route_id"].annotation,
            str,
        )
        self.assertEqual(
            signature.parameters["executor"].kind,
            inspect.Parameter.KEYWORD_ONLY,
        )
        self.assertIsNot(
            signature.parameters["executor"].default,
            inspect.Parameter.empty,
        )
        self.assertEqual(
            signature.return_annotation,
            route_publication.RoutePublication,
        )

    def test_publish_route_review_signature_has_no_evidence_injection(self) -> None:
        signature = inspect.signature(route_publication.publish_route_review)

        self.assertEqual(list(signature.parameters), ["repository_root", "route_id"])
        self.assertEqual(
            signature.parameters["repository_root"].annotation,
            Path,
        )
        self.assertEqual(
            signature.parameters["route_id"].annotation,
            str,
        )
        self.assertEqual(signature.return_annotation, route_publication.ReviewPublication)


class RoutePublicationBehaviorTests(unittest.TestCase):
    maxDiff = None

    def test_cli_exposes_closed_publish_commands_with_canonical_success_payload(self) -> None:
        fixture = make_unpublished_fixture(self)
        calls: list[tuple[Path, str]] = []

        def fake_publish(root: Path, route_id: str, **kwargs: object) -> route_publication.RoutePublication:
            calls.append((root, route_id))
            return route_publication.RoutePublication(
                route_id=route_id,
                artifact_root=Path("evidence/crouzeix_conjecture/routes/jin"),
                receipt_path=Path("evidence/crouzeix_conjecture/routes/jin/receipt.json"),
                receipt_sha256="a" * 64,
            )

        with (
            mock.patch.object(proof_evidence, "canonical_repository_root", return_value=fixture.repo),
            mock.patch.object(proof_evidence, "route_publication") as publication_module,
            mock.patch("sys.stdout", new_callable=lambda: __import__("io").StringIO()) as stdout,
        ):
            publication_module.publish_route_receipt.side_effect = fake_publish
            exit_code = proof_evidence.main(["publish-route", "--route", "jin"])

        self.assertEqual(exit_code, 0)
        self.assertEqual(calls, [(fixture.repo, "jin")])
        self.assertEqual(
            json.loads(stdout.getvalue()),
            {
                "schema_version": "crouzeix-route-publication/v1",
                "route_id": "jin",
                "status": "published-unreferenced",
                "artifact_root": "evidence/crouzeix_conjecture/routes/jin",
                "receipt_path": "evidence/crouzeix_conjecture/routes/jin/receipt.json",
                "receipt_sha256": "a" * 64,
            },
        )

    def test_cli_rejects_publish_override_arguments(self) -> None:
        with self.assertRaises(SystemExit):
            proof_evidence.main(["publish-route", "--route", "jin", "--path", "x"])
        with self.assertRaises(SystemExit):
            proof_evidence.main(["publish-review", "--route", "jin", "--env", "x=y"])

    def test_publish_review_cli_normalizes_validation_failure_to_blocked_json(self) -> None:
        fixture = make_unpublished_fixture(self)
        with (
            mock.patch.object(
                proof_evidence, "canonical_repository_root", return_value=fixture.repo
            ),
            mock.patch.object(
                proof_evidence.route_publication,
                "publish_route_review",
                side_effect=route_validation.RouteValidationError(
                    "review parent contains symlink"
                ),
            ),
            mock.patch(
                "sys.stdout", new_callable=lambda: __import__("io").StringIO()
            ) as stdout,
        ):
            exit_code = proof_evidence.main(["publish-review", "--route", "jin"])

        self.assertEqual(exit_code, 1)
        self.assertEqual(
            json.loads(stdout.getvalue()),
            {
                "schema_version": "crouzeix-review-publication/v1",
                "route_id": "jin",
                "status": "blocked",
                "reason": "review parent contains symlink",
            },
        )

    def test_blocked_preflight_prevents_executor_stage_lock_and_final_creation(self) -> None:
        fixture = make_unpublished_fixture(self)
        (fixture.repo / "formalization/lean/lean-toolchain").write_text(
            "leanprover/lean4:wrong\n", encoding="utf-8"
        )
        executor = RecordingExecutor()

        with self.assertRaisesRegex(route_publication.RoutePublicationError, "preflight|toolchain"):
            route_publication.publish_route_receipt(fixture.repo, "jin", executor=executor)

        self.assertEqual(executor.calls, [])
        self.assertFalse((fixture.repo / route_validation.ROUTE_STAGING_PARENT).exists())
        self.assertFalse((fixture.repo / fixture.receipt_path).exists())
        self.assertFalse((fixture.repo / ".build/crouzeix-route-publication-locks").exists())

    def test_receipt_publication_runs_route_build_once_and_audits_every_declaration(self) -> None:
        fixture = make_unpublished_fixture(self)
        executor = RecordingExecutor()

        publication = route_publication.publish_route_receipt(fixture.repo, "jin", executor=executor)

        build_calls = [call for call in executor.calls if call[0][:1] == ["scripts/check_lean_library.sh"]]
        audit_calls = [call for call in executor.calls if call[0][:3] == ["lake", "env", "lean"]]
        expected_declarations = sorted(node["declaration"] for node in fixture.manifest["nodes"])
        self.assertEqual(len(build_calls), 1)
        self.assertEqual(build_calls[0][0], ["scripts/check_lean_library.sh", "CrouzeixJin"])
        self.assertEqual(len(audit_calls), len(expected_declarations))
        receipt = json.loads((fixture.repo / publication.receipt_path).read_text(encoding="utf-8"))
        self.assertEqual(
            sorted(item["declaration"] for item in receipt["axiom_results"]),
            expected_declarations,
        )
        self.assertEqual(
            {tuple(item["axioms"]) for item in receipt["axiom_results"]},
            {("propext",)},
        )
        for call in audit_calls:
            audit_source = call[1] / call[0][-1]
            self.assertFalse(audit_source.exists())

    def test_axiom_audit_executes_real_print_axioms_source(self) -> None:
        fixture = make_unpublished_fixture(self)
        observed_sources: list[str] = []
        executor = RecordingExecutor()
        original_call = executor.__call__

        def inspect_audit(
            argv: list[str], cwd: Path, env: dict[str, str], timeout: int, cap: int
        ) -> route_publication.CommandResult:
            if argv[:3] == ["lake", "env", "lean"]:
                observed_sources.append((cwd / argv[-1]).read_text(encoding="utf-8"))
            return original_call(argv, cwd, env, timeout, cap)

        route_publication.publish_route_receipt(
            fixture.repo, "jin", executor=inspect_audit
        )
        expected = {
            f"import {Path(node['module_path']).with_suffix('').as_posix().replace('/', '.')}\n"
            f"#print axioms {node['declaration']}\n"
            for node in fixture.manifest["nodes"]
        }
        self.assertEqual(set(observed_sources), expected)

    def test_provider_report_binds_observed_import_closure(self) -> None:
        fixture = make_unpublished_fixture(self)
        manifest = route_validation.parse_route_manifest(fixture.manifest)
        incomplete = route_publication.provider_independence.ProviderIndependenceReport(
            roots=(manifest.aggregate_module,),
            modules=(manifest.aggregate_module,),
            set_options=(),
        )
        with mock.patch.object(
            route_publication.provider_independence,
            "audit_provider_independence",
            return_value=incomplete,
        ):
            with self.assertRaisesRegex(
                route_publication.RoutePublicationError, "observed provider closure"
            ):
                route_publication._compute_provider_report(fixture.repo, manifest)

    def test_forbidden_or_missing_axiom_evidence_prevents_publication(self) -> None:
        for output, pattern in (
            (b"'CrouzeixConjecture.jinBase' depends on axioms: [False.elim]\n", "forbidden axiom"),
            (b"not an axiom report\n", "axiom audit"),
        ):
            with self.subTest(output=output):
                fixture = make_unpublished_fixture(self)
                executor = RecordingExecutor(axiom_output=output)
                with self.assertRaisesRegex(route_publication.RoutePublicationError, pattern):
                    route_publication.publish_route_receipt(fixture.repo, "jin", executor=executor)
                self.assertFalse((fixture.repo / fixture.receipt_path).exists())

    def test_candidate_validator_rejects_unknown_hardlink_and_symlink_members(self) -> None:
        fixture = make_unpublished_fixture(self)
        stage = fixture.repo / route_validation.ROUTE_STAGING_PARENT / ".candidate"
        (stage / "build").mkdir(parents=True)
        (stage / "audit").mkdir()
        for member in route_validation.ROUTE_RECEIPT_MEMBERS:
            path = stage / member
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(b"{}\n" if member.endswith(".json") else b"")

        (stage / "extra.txt").write_text("x", encoding="utf-8")
        with self.assertRaisesRegex(route_validation.RouteValidationError, "unknown member"):
            route_validation.validate_route_receipt_candidate(fixture.repo, fixture.manifest_path, stage)
        (stage / "extra.txt").unlink()

        (stage / "build/stdout.log").unlink()
        (stage / "build/stdout.log").symlink_to("stderr.log")
        with self.assertRaisesRegex(route_validation.RouteValidationError, "symlink"):
            route_validation.validate_route_receipt_candidate(fixture.repo, fixture.manifest_path, stage)

        (stage / "build/stdout.log").unlink()
        os.link(stage / "build/stderr.log", stage / "build/stdout.log")
        with self.assertRaisesRegex(route_validation.RouteValidationError, "hardlink"):
            route_validation.validate_route_receipt_candidate(fixture.repo, fixture.manifest_path, stage)

    def test_post_rename_validation_failure_is_committed_publication_error(self) -> None:
        fixture = make_unpublished_fixture(self)
        executor = RecordingExecutor()

        with mock.patch.object(
            route_publication,
            "_validate_published_receipt",
            side_effect=route_publication.RoutePublicationError("mutated after rename"),
        ):
            with self.assertRaises(route_publication.CommittedPublicationError) as context:
                route_publication.publish_route_receipt(fixture.repo, "jin", executor=executor)

        self.assertTrue((fixture.repo / fixture.receipt_path).exists())
        self.assertEqual(
            context.exception.path,
            (fixture.repo / fixture.receipt_path).resolve(),
        )

    def test_full_post_rename_validator_rejects_mutated_member_and_preserves_final(self) -> None:
        fixture = make_unpublished_fixture(self)
        executor = RecordingExecutor()
        original = route_publication._fsync_tree
        calls = 0

        def mutate_after_final_fsync(root: Path, relative: Path) -> None:
            nonlocal calls
            calls += 1
            original(root, relative)
            if calls == 2:
                (root / relative / "build/command.json").write_text(
                    '{"mutated":true}\n', encoding="utf-8"
                )

        with mock.patch.object(
            route_publication, "_fsync_tree", side_effect=mutate_after_final_fsync
        ):
            with self.assertRaises(route_publication.CommittedPublicationError):
                route_publication.publish_route_receipt(
                    fixture.repo, "jin", executor=executor
                )

        self.assertTrue((fixture.repo / fixture.receipt_path).exists())
        self.assertEqual(
            (fixture.repo / fixture.receipt_path).parent.joinpath("build/command.json").read_text(),
            '{"mutated":true}\n',
        )

    def test_input_mutation_during_build_prevents_publication(self) -> None:
        fixture = make_unpublished_fixture(self)
        executor = RecordingExecutor()
        original_call = executor.__call__

        def mutating_call(
            argv: list[str], cwd: Path, env: dict[str, str], timeout: int, cap: int
        ) -> route_publication.CommandResult:
            result = original_call(argv, cwd, env, timeout, cap)
            if argv[:1] == ["scripts/check_lean_library.sh"]:
                wrapper = fixture.repo / "scripts/check_lean_library.sh"
                wrapper.write_text(wrapper.read_text() + "# drift\n", encoding="utf-8")
            return result

        with self.assertRaisesRegex(
            route_publication.RoutePublicationError, "input/cache mutation"
        ):
            route_publication.publish_route_receipt(
                fixture.repo, "jin", executor=mutating_call
            )
        self.assertFalse((fixture.repo / fixture.receipt_path).exists())

    def test_stage_cleanup_refuses_replacement_directory(self) -> None:
        fixture = make_unpublished_fixture(self)
        stage = route_publication._create_stage(fixture.repo, "jin")
        moved = stage.with_name(stage.name + "-original")
        stage.rename(moved)
        stage.mkdir()
        marker = stage / "preserve.txt"
        marker.write_text("replacement", encoding="utf-8")

        with self.assertRaisesRegex(
            route_publication.RoutePublicationError, "identity changed"
        ):
            route_publication._remove_owned_stage(fixture.repo, stage)

        self.assertEqual(marker.read_text(encoding="utf-8"), "replacement")
        shutil.rmtree(stage)
        shutil.rmtree(moved)

    def test_stage_and_lock_reject_symlinked_build_without_external_write(self) -> None:
        fixture = make_unpublished_fixture(self)
        build = fixture.repo / ".build"
        external = fixture.repo / "external-build"
        external.mkdir()
        build.symlink_to(external, target_is_directory=True)

        with self.assertRaisesRegex(
            (route_publication.RoutePublicationError, route_validation.RouteValidationError),
            "symlink|without following links",
        ):
            route_publication._create_stage(fixture.repo, "jin")
        with self.assertRaisesRegex(
            (route_publication.RoutePublicationError, route_validation.RouteValidationError),
            "symlink|without following links",
        ):
            route_publication._acquire_lock(fixture.repo, "receipt-jin")

        self.assertEqual(list(external.iterdir()), [])

    def test_stage_writes_remain_bound_after_stage_path_swap(self) -> None:
        fixture = make_unpublished_fixture(self)
        stage = route_publication._create_stage(fixture.repo, "jin")
        moved = stage.with_name(stage.name + "-moved")
        stage.rename(moved)
        replacement = stage
        replacement.mkdir()
        manifest_raw = route_validation._read_json(
            fixture.repo, fixture.manifest_path, "manifest"
        )
        manifest = route_validation.parse_route_manifest(manifest_raw)

        route_publication._write_receipt_candidate(
            fixture.repo,
            fixture.manifest_path,
            manifest_raw,
            manifest,
            stage,
            route_publication.CommandResult(0, b"build ok\n", b""),
            RecordingExecutor(),
            fixture.commit,
            fixture.tree,
        )

        self.assertEqual(list(replacement.iterdir()), [])
        self.assertTrue((moved / "receipt.json").is_file())
        replacement.rmdir()
        moved.rename(stage)
        route_publication._remove_owned_stage(fixture.repo, stage)

    def test_review_precreate_failure_is_not_reported_as_committed(self) -> None:
        fixture = RouteFixture("jin")
        self.addCleanup(fixture.cleanup)
        write_review_candidate(fixture)
        (fixture.repo / fixture.review_path).unlink()
        fixture.manifest["review_sha256"] = None
        fixture.rewrite_manifest()

        with (
            mock.patch.object(
                route_publication,
                "_write_bytes_at",
                side_effect=route_publication.RoutePublicationError("precreate failure"),
            ),
            mock.patch.object(route_validation, "_validate_receipt", return_value=fixture.receipt),
        ):
            with self.assertRaisesRegex(
                route_publication.RoutePublicationError, "precreate failure"
            ) as context:
                route_publication.publish_route_review(fixture.repo, "jin")

        self.assertNotIsInstance(
            context.exception, route_publication.CommittedPublicationError
        )
        self.assertFalse((fixture.repo / fixture.review_path).exists())

    def test_review_publication_rejects_unresolved_blocking_findings_and_overwrite(self) -> None:
        fixture = RouteFixture("jin")
        self.addCleanup(fixture.cleanup)
        write_review_candidate(
            fixture,
            mutate={
                "findings": [
                    {
                        "finding_id": "F-1",
                        "severity": "Critical",
                        "resolved": False,
                        "locator": fixture.manifest_path.as_posix(),
                        "statement": "blocking",
                        "falsifying_test_or_gap": "gap",
                    }
                ]
            },
        )
        (fixture.repo / fixture.review_path).unlink()
        fixture.manifest["review_sha256"] = None
        fixture.rewrite_manifest()
        bound_receipt = json.loads(
            (fixture.repo / fixture.receipt_path).read_text(encoding="utf-8")
        )

        with mock.patch.object(
            route_validation,
            "_validate_receipt",
            return_value=bound_receipt,
        ):
            with self.assertRaisesRegex(route_publication.RoutePublicationError, "Critical/Important"):
                route_publication.publish_route_review(fixture.repo, "jin")

        write_review_candidate(fixture)
        (fixture.repo / fixture.review_path).write_text("existing\n", encoding="utf-8")
        with mock.patch.object(
            route_validation,
            "_validate_receipt",
            return_value=bound_receipt,
        ):
            with self.assertRaisesRegex(route_publication.RoutePublicationError, "already exists"):
                route_publication.publish_route_review(fixture.repo, "jin")

    def test_review_publication_rejects_symlinked_final_parent_without_external_write(self) -> None:
        fixture = RouteFixture("jin")
        self.addCleanup(fixture.cleanup)
        write_review_candidate(fixture)
        (fixture.repo / fixture.review_path).unlink()
        fixture.manifest["review_sha256"] = None
        fixture.rewrite_manifest()
        parent = (fixture.repo / fixture.review_path).parent
        parent.rmdir()
        external = fixture.repo / "external-reviews"
        external.mkdir()
        parent.symlink_to(external, target_is_directory=True)

        with self.assertRaisesRegex(
            (route_publication.RoutePublicationError, route_validation.RouteValidationError),
            "symlink|without following links",
        ):
            route_publication.publish_route_review(fixture.repo, "jin")

        self.assertFalse((external / "jin.json").exists())

    def test_candidate_identity_comes_from_route_phase_ledger_and_is_not_jin_constant_for_other_routes(self) -> None:
        phase_map = {
            "jin": ("cpfr-085", "8c3ca032707da2f87627b944c3f336e6c4062653"),
            "lorist-schwenninger": ("cpfr-086", "b" * 40),
            "harp": ("cpfr-087", "c" * 40),
        }
        for route_id, (phase, commit) in phase_map.items():
            with self.subTest(route_id=route_id):
                root = Path(tempfile.mkdtemp())
                self.addCleanup(shutil.rmtree, root, ignore_errors=True)
                ledger = root / route_publication.LEDGER_PATH
                ledger.parent.mkdir(parents=True, exist_ok=True)
                ledger.write_text(
                    execution_ledger.HEADER
                    + "\n"
                    + (
                        f"{phase}\t{phase.upper()}\tlocally-verified\t"
                        f"feat/{phase}\t.worktrees/{phase}\t{'2' * 40}\t"
                        f"{commit}\tverifier\t0\tsha256:{'2' * 64}\t\t"
                        f"tree={'d' * 40}\n"
                    ),
                    encoding="utf-8",
                )
                with mock.patch.object(route_publication, "_git_stdout", side_effect=[("d" * 40) + "\n"]), mock.patch.object(
                    route_publication, "_require_git_ancestor"
                ), mock.patch.object(route_publication, "_require_route_inputs_at_commit"):
                    candidate_commit, candidate_tree = route_publication._candidate_identity(
                        root,
                        mock.Mock(route_id=route_id),
                    )
                self.assertEqual(candidate_commit, commit)
                self.assertEqual(candidate_tree, "d" * 40)

    def test_candidate_identity_rejects_missing_or_preverified_rows(self) -> None:
        root = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, root, ignore_errors=True)
        ledger = root / route_publication.LEDGER_PATH
        ledger.parent.mkdir(parents=True, exist_ok=True)
        ledger.write_text(
            execution_ledger.HEADER
            + "\n"
            + "cpfr-085\tCPFR-085\timplementing\tfeat/a\t.worktrees/a\t"
            + "1" * 40
            + "\t"
            + "2" * 40
            + "\t\t\t\t\t\n",
            encoding="utf-8",
        )
        with self.assertRaisesRegex(route_publication.RoutePublicationError, "candidate identity is missing"):
            route_publication._candidate_identity(root, mock.Mock(route_id="jin"))

    def test_candidate_input_binding_uses_bounded_blob_ids_for_large_sources(self) -> None:
        fixture = make_unpublished_fixture(self)
        large_module = fixture.repo / "formalization/lean/Crouzeix/Jin/Large.lean"
        large_module.parent.mkdir(parents=True, exist_ok=True)
        large_module.write_bytes(b"-- large source\n" + (b"x" * 20000))
        subprocess.run(["git", "add", "-A"], cwd=fixture.repo, check=True)
        subprocess.run(
            ["git", "commit", "-q", "-m", "bind large source"],
            cwd=fixture.repo,
            env={
                **os.environ,
                "GIT_AUTHOR_NAME": "Fixture",
                "GIT_AUTHOR_EMAIL": "fixture@example.invalid",
                "GIT_COMMITTER_NAME": "Fixture",
                "GIT_COMMITTER_EMAIL": "fixture@example.invalid",
            },
            check=True,
        )
        commit = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=fixture.repo,
            text=True, capture_output=True, check=True,
        ).stdout.strip()
        with route_publication.goal_validation.git_session() as session:
            route_publication._require_paths_at_commit(
                fixture.repo, commit, [Path("formalization/lean/Crouzeix/Jin/Large.lean")],
                session=session,
            )
        large_module.write_bytes(large_module.read_bytes() + b"drift")
        with route_publication.goal_validation.git_session() as session:
            with self.assertRaisesRegex(
                route_publication.RoutePublicationError, "drifted since candidate commit"
            ):
                route_publication._require_paths_at_commit(
                    fixture.repo, commit,
                    [Path("formalization/lean/Crouzeix/Jin/Large.lean")],
                    session=session,
                )

    def test_nonblocking_lock_rejects_second_publisher(self) -> None:
        fixture = make_unpublished_fixture(self)
        first = route_publication._acquire_lock(fixture.repo, "receipt-jin")
        self.addCleanup(os.close, first)
        with self.assertRaisesRegex(route_publication.RoutePublicationError, "already locked"):
            second = route_publication._acquire_lock(fixture.repo, "receipt-jin")
            os.close(second)

    def test_review_candidate_reader_rejects_parent_symlink_hardlink_oversize_and_duplicate_keys(self) -> None:
        fixture = RouteFixture("jin")
        self.addCleanup(fixture.cleanup)
        parent = fixture.repo / route_validation.ROUTE_REVIEW_CANDIDATE_PARENT
        parent.mkdir(parents=True, exist_ok=True)
        candidate = parent / "jin.json"
        candidate.write_text("{}", encoding="utf-8")

        alt_parent = fixture.repo / ".build/alt-review-parent"
        alt_parent.mkdir(parents=True)
        parent.unlink(missing_ok=True) if parent.is_symlink() else None
        shutil.rmtree(parent)
        parent.symlink_to(alt_parent, target_is_directory=True)
        with self.assertRaisesRegex(route_publication.RoutePublicationError, "outside fixed parent|symlink"):
            route_publication._read_review_candidate(fixture.repo, candidate, "jin")

        parent.unlink()
        parent.mkdir(parents=True)
        candidate = parent / "jin.json"
        other = parent / "other.json"
        other.write_text("{}", encoding="utf-8")
        os.link(other, candidate)
        with self.assertRaisesRegex(route_publication.RoutePublicationError, "hardlink"):
            route_publication._read_review_candidate(fixture.repo, candidate, "jin")
        candidate.unlink()
        other.unlink()

        candidate.write_bytes(b"x" * (route_validation.MAX_JSON_BYTES + 1))
        with self.assertRaisesRegex(route_publication.RoutePublicationError, "byte bound|exceeds"):
            route_publication._read_review_candidate(fixture.repo, candidate, "jin")
        candidate.write_text('{"a":1,"a":2}\n', encoding="utf-8")
        with self.assertRaisesRegex(route_publication.RoutePublicationError, "invalid JSON"):
            route_publication._read_review_candidate(fixture.repo, candidate, "jin")

    def test_subprocess_executor_hard_caps_output_and_reaps_process(self) -> None:
        result = route_publication._subprocess_executor(
            [
                sys.executable,
                "-c",
                "import sys,time; sys.stdout.buffer.write(b'x'*2000000); sys.stdout.flush(); time.sleep(5)",
            ],
            REPO,
            dict(os.environ),
            10,
            1024,
        )

        self.assertIsNone(result.exit_code)
        self.assertTrue(result.stdout_truncated)
        self.assertLessEqual(len(result.stdout), 1024)

    def test_subprocess_executor_kills_descendant_that_retains_pipes(self) -> None:
        started = time.monotonic()
        result = route_publication._subprocess_executor(
            [
                sys.executable,
                "-c",
                (
                    "import subprocess,sys; "
                    "subprocess.Popen([sys.executable,'-c','import time; time.sleep(30)']); "
                    "print('parent done')"
                ),
            ],
            REPO,
            dict(os.environ),
            5,
            1024,
        )
        elapsed = time.monotonic() - started

        self.assertLess(elapsed, 5)
        self.assertIsNotNone(result.blocked_reason)
        self.assertIn("pipes", result.blocked_reason)

    def test_review_binding_matrix_rejects_every_identity_drift(self) -> None:
        fixture = RouteFixture("jin")
        self.addCleanup(fixture.cleanup)
        receipt = dict(fixture.receipt)
        manifest_raw = dict(fixture.manifest)
        manifest = route_validation.parse_route_manifest(manifest_raw)
        base = dict(fixture.review)
        cases = {
            "route_id": ("harp", "review route_id mismatch"),
            "reviewed_commit": ("a" * 40, "review reviewed_commit mismatch"),
            "reviewed_tree": ("b" * 40, "review reviewed_commit mismatch"),
            "manifest_sha256": ("c" * 64, "review manifest_sha256 mismatch"),
            "terminal_type_sha256": ("d" * 64, "review terminal_type_sha256 mismatch"),
        }
        for field, (value, pattern) in cases.items():
            with self.subTest(field=field):
                review = dict(base)
                review[field] = value
                review["review_sha256"] = None
                review["review_sha256"] = route_validation.self_digest(
                    review, "review_sha256"
                )
                with self.assertRaisesRegex(
                    route_publication.RoutePublicationError, pattern
                ):
                    route_publication._validate_review_candidate(
                        fixture.manifest_path, manifest_raw, manifest, receipt, review
                    )
        bad_digest = dict(base)
        bad_digest["review_sha256"] = "0" * 64
        with self.assertRaisesRegex(
            route_publication.RoutePublicationError, "review identity digest mismatch"
        ):
            route_publication._validate_review_candidate(
                fixture.manifest_path, manifest_raw, manifest, receipt, bad_digest
            )

    def test_source_guard_forbids_walk_rmtree_rglob_and_blocking_locks(self) -> None:
        for module in (route_publication, route_validation):
            source = inspect.getsource(module)
            self.assertNotIn(".rglob(", source)
            self.assertNotIn("os.walk(", source)
            self.assertNotIn("shutil.rmtree", source)
            self.assertNotIn("LOCK_EX)", source)


if __name__ == "__main__":
    unittest.main()
