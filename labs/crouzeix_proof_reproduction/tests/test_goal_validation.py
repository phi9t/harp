from __future__ import annotations

import gc
import json
import os
import hashlib
import subprocess
import sys
import tempfile
import unittest
import stat
import warnings
from pathlib import Path
from unittest import mock


LAB = Path(__file__).resolve().parents[1]
REPO = LAB.parents[1]
SCRIPT = LAB / "proof_evidence.py"
sys.path.insert(0, str(LAB))

import proof_evidence  # noqa: E402


GOAL_PLAN = (
    REPO / "docs/superpowers/plans/2026-08-23-crouzeix-end-to-end-proof-goal.md"
)
LEDGER_PATH = (
    REPO / "docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv"
)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def run_cli(*args: str) -> subprocess.CompletedProcess[str]:
    env = dict(os.environ)
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    return subprocess.run(
        [sys.executable, str(SCRIPT), *args],
        cwd=REPO,
        env=env,
        text=True,
        capture_output=True,
        check=False,
    )


class GoalValidationTests(unittest.TestCase):
    maxDiff = None
    LANDED_VERIFIER_PATHS = {
        "cpfr-085": "evidence/crouzeix_conjecture/routes/jin/receipt.json",
        "cpfr-086": "evidence/crouzeix_conjecture/routes/lorist-schwenninger/receipt.json",
        "cpfr-087": "evidence/crouzeix_conjecture/routes/harp/receipt.json",
        "cpfr-088": "evidence/crouzeix_conjecture/local_formalization/manifest.tsv",
        "cpfr-089": "atlas/dist/harp-atlas.receipt.json",
        "cpfr-090": "docs/workstream/crouzeix-proof-reproduction/program-verification-003.json",
        "cpfr-091": "docs/workstream/crouzeix-proof-reproduction/worktree-inventory-002.md",
        "phase-8-plan-review": "docs/workstream/crouzeix-proof-reproduction/retrospective-003-review.json",
    }

    def make_in_progress_goal(self, root: Path) -> Path:
        goal = root / "goal.md"
        goal.write_text(
            "## Completion footer\n\n```yaml\n"
            "goal_status: in-progress\n"
            "jin: incomplete\n"
            "lorist_schwenninger: incomplete\n"
            "harp: incomplete\n"
            "local_bundle: absent-or-optional\n"
            "reader_surfaces: unreconciled\n"
            "program_verifier: pending\n"
            "master_landing: null\n"
            "worktree_audit: pending\n"
            "post_execution_review: pending\n"
            "plan_revision: null\n"
            "```\n",
            encoding="utf-8",
        )
        return goal

    def make_complete_goal(self, root: Path) -> Path:
        goal = root / "goal.md"
        goal.write_text(
            "## Completion footer\n\n```yaml\n"
            "goal_status: complete\n"
            "jin: complete-local\n"
            "lorist_schwenninger: complete-local\n"
            "harp: complete-local\n"
            "local_bundle: required-and-valid\n"
            "reader_surfaces: reconciled\n"
            "program_verifier: passed\n"
            "master_landing: " + ("a" * 40) + "\n"
            "worktree_audit: passed\n"
            "post_execution_review: passed\n"
            "plan_revision: " + ("b" * 40) + "\n"
            "```\n",
            encoding="utf-8",
        )
        return goal

    def write_ledger(self, root: Path, *rows: str) -> Path:
        path = root / "execution-ledger.tsv"
        path.write_text(
            "phase\tticket\tstate\tbranch\tworktree\tbase_commit\tcandidate_commit\tverifier\texit_code\tevidence_digest\tlanding_commit\tnote\n"
            + "".join(rows),
            encoding="utf-8",
        )
        return path

    def write_text(self, root: Path, relative: str, content: str) -> Path:
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8")
        return path

    def write_bytes(self, root: Path, relative: str, content: bytes) -> Path:
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)
        return path

    def init_git_repo_with_commit(self, root: Path) -> tuple[str, str]:
        subprocess.run(
            ["/usr/bin/git", "init", "-b", "master"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
            env={
                "HOME": str(root),
                "LC_ALL": "C",
                "PATH": "/usr/bin:/bin",
            },
        )
        subprocess.run(
            ["/usr/bin/git", "config", "user.name", "Test User"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
            env={
                "HOME": str(root),
                "LC_ALL": "C",
                "PATH": "/usr/bin:/bin",
            },
        )
        subprocess.run(
            ["/usr/bin/git", "config", "user.email", "test@example.com"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
            env={
                "HOME": str(root),
                "LC_ALL": "C",
                "PATH": "/usr/bin:/bin",
            },
        )
        self.write_text(root, "tracked.txt", "root\n")
        subprocess.run(
            ["/usr/bin/git", "add", "tracked.txt"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
            env={
                "HOME": str(root),
                "LC_ALL": "C",
                "PATH": "/usr/bin:/bin",
            },
        )
        subprocess.run(
            ["/usr/bin/git", "commit", "-m", "initial"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
            env={
                "HOME": str(root),
                "LC_ALL": "C",
                "PATH": "/usr/bin:/bin",
            },
        )
        head = subprocess.run(
            ["/usr/bin/git", "rev-parse", "HEAD"],
            cwd=root,
            check=True,
            capture_output=True,
            text=True,
            env={
                "HOME": str(root),
                "LC_ALL": "C",
                "PATH": "/usr/bin:/bin",
            },
        ).stdout.strip()
        return head, head

    def write_phase8_review(
        self,
        root: Path,
        *,
        spec_status: str = "passed",
        standards_status: str = "passed",
    ) -> None:
        retrospective_rel = "docs/workstream/crouzeix-proof-reproduction/retrospective-003.json"
        plan_rel = "docs/superpowers/plans/2026-08-23-crouzeix-end-to-end-proof-goal.md"
        retrospective = self.write_text(root, retrospective_rel, "{\"retrospective\":true}\n")
        plan = self.write_text(root, plan_rel, "# plan\n")
        review_payload = {
            "schema_version": "crouzeix-plan-review/v1",
            "spec_status": spec_status,
            "standards_status": standards_status,
            "retrospective_path": retrospective_rel,
            "retrospective_sha256": sha256_bytes(retrospective.read_bytes()),
            "plan_path": plan_rel,
            "plan_sha256": sha256_bytes(plan.read_bytes()),
        }
        self.write_text(
            root,
            self.LANDED_VERIFIER_PATHS["phase-8-plan-review"],
            json.dumps(review_payload) + "\n",
        )

    def materialize_landed_evidence(self, root: Path) -> dict[str, str]:
        digests: dict[str, str] = {}
        for phase, relative in self.LANDED_VERIFIER_PATHS.items():
            if phase == "phase-8-plan-review":
                self.write_phase8_review(root)
                digests[phase] = "sha256:" + sha256_bytes(
                    (root / relative).read_bytes()
                )
                continue
            path = self.write_text(root, relative, f"{phase}\n")
            digests[phase] = "sha256:" + sha256_bytes(path.read_bytes())
        return digests

    def landed_row(
        self,
        phase: str,
        ticket: str,
        *,
        verifier: str | None = None,
        exit_code: str = "0",
        evidence_digest: str | None = None,
        landing_commit: str | None = None,
    ) -> str:
        digest = evidence_digest if evidence_digest is not None else "sha256:" + ("3" * 64)
        commit = landing_commit if landing_commit is not None else ("4" * 40)
        verifier_value = verifier if verifier is not None else self.LANDED_VERIFIER_PATHS[phase]
        return (
            f"{phase}\t{ticket}\tlanded\tfeat/{phase}\t.worktrees/{phase}\t"
            + ("1" * 40)
            + "\t"
            + ("2" * 40)
            + f"\t{verifier_value}\t{exit_code}\t{digest}\t{commit}\t\n"
        )

    def implementing_row(self, phase: str, ticket: str) -> str:
        return (
            f"{phase}\t{ticket}\timplementing\tfeat/{phase}\t.worktrees/{phase}\t"
            + ("1" * 40)
            + "\t"
            + ("2" * 40)
            + "\t\t\t\t\t\n"
        )

    def test_validate_goal_cli_reports_next_incomplete_phase(self) -> None:
        result = run_cli("validate-goal")

        self.assertEqual(result.returncode, 1)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["status"], "incomplete")
        self.assertEqual(payload["goal_status"], "in-progress")
        self.assertEqual(payload["earliest_incomplete_phase"], "cpfr-090")
        self.assertEqual(payload["verified_commits"], [])
        self.assertIn(
            "docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv",
            payload["evidence_paths"],
        )

    def test_public_result_dataclass_contract(self) -> None:
        from labs.crouzeix_proof_reproduction.goal_validation import GoalValidationResult

        result = GoalValidationResult(
            goal_status="in-progress",
            earliest_incomplete_phase="cpfr-085",
            verified_commits=(),
            evidence_paths=(LEDGER_PATH.as_posix(),),
        )

        self.assertEqual(result.goal_status, "in-progress")
        self.assertEqual(result.earliest_incomplete_phase, "cpfr-085")
        self.assertEqual(result.verified_commits, ())
        self.assertEqual(result.evidence_paths, (LEDGER_PATH.as_posix(),))

    def test_validator_uses_fixed_goal_plan_path(self) -> None:
        with mock.patch.object(proof_evidence, "canonical_repository_root", return_value=REPO):
            stdout = []
            with mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock:
                stdout_mock.write.side_effect = stdout.append
                exit_code = proof_evidence.main(
                    [
                        "validate-goal",
                        "--goal-plan",
                        "docs/superpowers/plans/other.md",
                    ]
                )

        self.assertEqual(exit_code, 2)
        self.assertEqual("".join(stdout), "")

    def test_validate_goal_is_read_only_and_does_not_invoke_lean_or_lake(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            fake_bin = Path(directory)
            marker = fake_bin / "lean-or-lake-marker"
            for name in ("lean", "lake"):
                tool = fake_bin / name
                tool.write_text("#!/bin/sh\n: > \"$GOAL_VALIDATION_MARKER\"\nexit 99\n", encoding="utf-8")
                tool.chmod(0o755)
            before = {
                path.relative_to(REPO).as_posix(): path.stat().st_mtime_ns
                for path in (GOAL_PLAN, LEDGER_PATH)
            }
            env = dict(os.environ)
            env["PATH"] = f"{fake_bin}{os.pathsep}{env['PATH']}"
            env["GOAL_VALIDATION_MARKER"] = str(marker)
            env["PYTHONDONTWRITEBYTECODE"] = "1"

            result = subprocess.run(
                [sys.executable, str(SCRIPT), "validate-goal"],
                cwd=REPO,
                env=env,
                text=True,
                capture_output=True,
                check=False,
            )

            after = {
                path.relative_to(REPO).as_posix(): path.stat().st_mtime_ns
                for path in (GOAL_PLAN, LEDGER_PATH)
            }

        self.assertEqual(result.returncode, 1)
        self.assertFalse(marker.exists(), "validate-goal invoked lean or lake")
        self.assertEqual(before, after, "validate-goal wrote to tracked inputs")

    def test_rejects_missing_duplicate_unknown_reordered_and_malformed_footer_keys(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = root / "goal.md"
            footer = self.make_in_progress_goal(root).read_text(encoding="utf-8")
            goal.write_text(footer, encoding="utf-8")

            missing = goal.with_name("missing.md")
            missing.write_text(footer.replace("jin: incomplete\n", ""), encoding="utf-8")
            duplicate = goal.with_name("duplicate.md")
            duplicate.write_text(footer.replace("jin: incomplete\n", "jin: incomplete\njin: incomplete\n"), encoding="utf-8")
            unknown = goal.with_name("unknown.md")
            unknown.write_text(footer.replace("plan_revision: null\n", "surprise: nope\nplan_revision: null\n"), encoding="utf-8")
            reordered = goal.with_name("reordered.md")
            reordered.write_text(footer.replace("harp: incomplete\nlocal_bundle: absent-or-optional\n", "local_bundle: absent-or-optional\nharp: incomplete\n"), encoding="utf-8")
            malformed = goal.with_name("malformed.md")
            malformed.write_text(footer.replace("master_landing: null\n", "master_landing null\n"), encoding="utf-8")

            for path, pattern in (
                (missing, "missing footer key"),
                (duplicate, "duplicate footer key"),
                (unknown, "unknown footer key"),
                (reordered, "footer keys are out of order"),
                (malformed, "malformed footer line"),
            ):
                with self.subTest(path=path.name):
                    with self.assertRaisesRegex(ValueError, pattern):
                        goal_validation.parse_goal_footer(path, repository_root=root)

    def test_footer_fences_and_trailing_content_are_exact(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            base = self.make_in_progress_goal(root).read_text(encoding="utf-8")

            bad_header = root / "bad_header.md"
            bad_header.write_text(base.replace("## Completion footer", "## completion footer", 1), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "completion footer is missing"):
                goal_validation.parse_goal_footer(bad_header, repository_root=root)

            bad_open = root / "bad_open.md"
            bad_open.write_text(base.replace("```yaml", "``` yaml", 1), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "completion footer fence is missing"):
                goal_validation.parse_goal_footer(bad_open, repository_root=root)

            bad_close = root / "bad_close.md"
            bad_close.write_text(base.replace("\n```\n", "\n``` \n", 1), encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "completion footer fence is unterminated"):
                goal_validation.parse_goal_footer(bad_close, repository_root=root)

            trailing = root / "trailing.md"
            trailing.write_text(base + "extra\n", encoding="utf-8")
            with self.assertRaisesRegex(ValueError, "completion footer has trailing content"):
                goal_validation.parse_goal_footer(trailing, repository_root=root)

    def test_rejects_complete_without_full_landed_execution_ledger(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            ledger = self.write_ledger(root, self.landed_row("cpfr-085", "CPFR-085"))
            goal = self.make_complete_goal(root)

            with self.assertRaisesRegex(ValueError, "requires landed rows for CPFR-085 through CPFR-091"):
                goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

    def test_complete_empty_ledger_requires_full_landed_rows_before_commit_or_bundle_checks(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_complete_goal(root)
            ledger = self.write_ledger(root)

            with self.assertRaisesRegex(ValueError, "requires landed rows for CPFR-085 through CPFR-091"):
                goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

    def test_complete_requires_current_master_commit_ancestry_checks(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_complete_goal(root)
            digests = self.materialize_landed_evidence(root)
            ledger = self.write_ledger(
                root,
                self.landed_row("cpfr-085", "CPFR-085", evidence_digest=digests["cpfr-085"]),
                self.landed_row("cpfr-086", "CPFR-086", evidence_digest=digests["cpfr-086"]),
                self.landed_row("cpfr-087", "CPFR-087", evidence_digest=digests["cpfr-087"]),
                self.landed_row("cpfr-088", "CPFR-088", evidence_digest=digests["cpfr-088"]),
                self.landed_row("cpfr-089", "CPFR-089", evidence_digest=digests["cpfr-089"]),
                self.landed_row("cpfr-090", "CPFR-090", evidence_digest=digests["cpfr-090"]),
                self.landed_row("cpfr-091", "CPFR-091", evidence_digest=digests["cpfr-091"]),
                self.landed_row(
                    "phase-8-plan-review",
                    "PHASE-8-PLAN-REVIEW",
                    evidence_digest=digests["phase-8-plan-review"],
                ),
            )

            with mock.patch.object(
                goal_validation,
                "_git_commit_exists",
                side_effect=lambda repository_root, commit_id, **kwargs: commit_id != ("a" * 40),
            ), mock.patch.object(
                goal_validation,
                "_git_is_ancestor",
                return_value=True,
            ):
                with self.assertRaisesRegex(ValueError, "master_landing does not exist"):
                    goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

    def test_complete_uses_route_validators_and_bundle_reader_checks(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_complete_goal(root)
            digests = self.materialize_landed_evidence(root)
            landed_rows = []
            for phase, ticket in (
                ("cpfr-085", "CPFR-085"),
                ("cpfr-086", "CPFR-086"),
                ("cpfr-087", "CPFR-087"),
                ("cpfr-088", "CPFR-088"),
                ("cpfr-089", "CPFR-089"),
                ("cpfr-090", "CPFR-090"),
                ("cpfr-091", "CPFR-091"),
                ("phase-8-plan-review", "PHASE-8-PLAN-REVIEW"),
            ):
                landed_rows.append(
                    self.landed_row(
                        phase,
                        ticket,
                        evidence_digest=digests[phase],
                    )
                )
            ledger = self.write_ledger(root, *landed_rows)

            route_result = mock.Mock(status="complete", claim_level="complete-local")
            with mock.patch.object(goal_validation.route_validation, "inspect_route", return_value=route_result) as inspect_mock, mock.patch.object(
                goal_validation,
                "_git_commit_exists",
                return_value=True,
            ), mock.patch.object(
                goal_validation,
                "_git_is_ancestor",
                return_value=True,
            ):
                with self.assertRaisesRegex(ValueError, "local formalization bundle is required"):
                    goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

        self.assertEqual(inspect_mock.call_count, 3)
        self.assertEqual(
            [call.args[1] for call in inspect_mock.call_args_list],
            ["jin", "lorist-schwenninger", "harp"],
        )

    def test_in_progress_earliest_phase_is_derived_from_ordered_ledger(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_in_progress_goal(root)
            ledger = self.write_ledger(
                root,
                self.landed_row("cpfr-085", "CPFR-085"),
                self.implementing_row("cpfr-086", "CPFR-086"),
            )

            result = goal_validation.validate_goal(
                goal,
                execution_ledger_path=ledger,
                repository_root=root,
            )

        self.assertEqual(result.earliest_incomplete_phase, "cpfr-086")

    def test_rejects_later_phase_before_earlier_phase_is_landed(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_in_progress_goal(root)
            ledger = self.write_ledger(root, self.implementing_row("cpfr-086", "CPFR-086"))

            with self.assertRaisesRegex(ValueError, "later phase appears before earlier landed phase"):
                goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

    def test_footer_value_enums_are_validated(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_in_progress_goal(root)
            bad_in_progress = goal.with_name("bad-in-progress.md")
            bad_in_progress.write_text(
                goal.read_text(encoding="utf-8").replace("program_verifier: pending\n", "program_verifier: done\n"),
                encoding="utf-8",
            )
            ledger = self.write_ledger(root)
            with self.assertRaisesRegex(ValueError, "program_verifier value is invalid"):
                goal_validation.validate_goal(
                    bad_in_progress,
                    execution_ledger_path=ledger,
                    repository_root=root,
                )

            complete = self.make_complete_goal(root)
            bad_complete = complete.with_name("bad-complete.md")
            bad_complete.write_text(
                complete.read_text(encoding="utf-8").replace("jin: complete-local\n", "jin: complete\n"),
                encoding="utf-8",
            )
            with self.assertRaisesRegex(ValueError, "jin value is invalid"):
                goal_validation.validate_goal(
                    bad_complete,
                    execution_ledger_path=ledger,
                    repository_root=root,
                )

    def test_landed_rows_require_bound_evidence_fields(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_complete_goal(root)
            digests = self.materialize_landed_evidence(root)
            ledger = self.write_ledger(
                root,
                self.landed_row("cpfr-085", "CPFR-085", verifier="", exit_code="0"),
                self.landed_row("cpfr-086", "CPFR-086", evidence_digest=digests["cpfr-086"]),
                self.landed_row("cpfr-087", "CPFR-087", evidence_digest=digests["cpfr-087"]),
                self.landed_row("cpfr-088", "CPFR-088", evidence_digest=digests["cpfr-088"]),
                self.landed_row("cpfr-089", "CPFR-089", evidence_digest=digests["cpfr-089"]),
                self.landed_row("cpfr-090", "CPFR-090", evidence_digest=digests["cpfr-090"]),
                self.landed_row("cpfr-091", "CPFR-091", evidence_digest=digests["cpfr-091"]),
                self.landed_row(
                    "phase-8-plan-review",
                    "PHASE-8-PLAN-REVIEW",
                    evidence_digest=digests["phase-8-plan-review"],
                ),
            )

            with self.assertRaisesRegex(ValueError, "landed row requires verifier"):
                goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

    def test_phase8_plan_review_row_requires_exact_ticket_and_evidence(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_complete_goal(root)
            digests = self.materialize_landed_evidence(root)
            ledger = self.write_ledger(
                root,
                self.landed_row("cpfr-085", "CPFR-085", evidence_digest=digests["cpfr-085"]),
                self.landed_row("cpfr-086", "CPFR-086", evidence_digest=digests["cpfr-086"]),
                self.landed_row("cpfr-087", "CPFR-087", evidence_digest=digests["cpfr-087"]),
                self.landed_row("cpfr-088", "CPFR-088", evidence_digest=digests["cpfr-088"]),
                self.landed_row("cpfr-089", "CPFR-089", evidence_digest=digests["cpfr-089"]),
                self.landed_row("cpfr-090", "CPFR-090", evidence_digest=digests["cpfr-090"]),
                self.landed_row("cpfr-091", "CPFR-091", evidence_digest=digests["cpfr-091"]),
                self.landed_row(
                    "phase-8-plan-review",
                    "CPFR-091",
                    evidence_digest=digests["phase-8-plan-review"],
                ),
            )

            with self.assertRaisesRegex(ValueError, "Phase 8 plan-review row is invalid"):
                goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

    def test_local_bundle_validation_fails_closed_without_optional_validator(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_bytes(
                root,
                "evidence/crouzeix_conjecture/local_formalization/manifest.tsv",
                b"x",
            )

            with mock.patch.object(goal_validation, "local_formalization_validation", None):
                with self.assertRaisesRegex(ValueError, "local formalization bundle is required"):
                    goal_validation._validate_local_bundle(root)

    def test_local_bundle_validation_requires_passed_read_only_bundle_verdict(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)

            class Result:
                status = "blocked"

            validator = mock.Mock()
            validator.validate_local_formalization_bundle.return_value = Result()

            with mock.patch.object(goal_validation, "local_formalization_validation", validator):
                with self.assertRaisesRegex(ValueError, "local formalization bundle is required"):
                    goal_validation._validate_local_bundle(root)

    def test_local_bundle_validation_preserves_validator_reason(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        validator = mock.Mock()
        validator.validate_local_formalization_bundle.side_effect = ValueError(
            "route review digest mismatch"
        )
        with tempfile.TemporaryDirectory() as directory, mock.patch.object(
            goal_validation, "local_formalization_validation", validator
        ):
            with self.assertRaisesRegex(
                ValueError,
                "local formalization bundle is required: route review digest mismatch",
            ):
                goal_validation._validate_local_bundle(Path(directory))

    def test_landed_rows_require_exact_evidence_paths_and_real_digests(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_complete_goal(root)
            digests = self.materialize_landed_evidence(root)
            ledger = self.write_ledger(
                root,
                self.landed_row(
                    "cpfr-085",
                    "CPFR-085",
                    verifier="evidence/crouzeix_conjecture/routes/jin/wrong.json",
                    evidence_digest=digests["cpfr-085"],
                ),
                self.landed_row("cpfr-086", "CPFR-086", evidence_digest=digests["cpfr-086"]),
                self.landed_row("cpfr-087", "CPFR-087", evidence_digest=digests["cpfr-087"]),
                self.landed_row("cpfr-088", "CPFR-088", evidence_digest=digests["cpfr-088"]),
                self.landed_row("cpfr-089", "CPFR-089", evidence_digest=digests["cpfr-089"]),
                self.landed_row("cpfr-090", "CPFR-090", evidence_digest=digests["cpfr-090"]),
                self.landed_row("cpfr-091", "CPFR-091", evidence_digest=digests["cpfr-091"]),
                self.landed_row(
                    "phase-8-plan-review",
                    "PHASE-8-PLAN-REVIEW",
                    evidence_digest=digests["phase-8-plan-review"],
                ),
            )

            with self.assertRaisesRegex(ValueError, "landed row verifier path is invalid"):
                goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

    def test_landed_rows_require_existing_ancestor_landing_commit(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_complete_goal(root)
            digests = self.materialize_landed_evidence(root)
            ledger = self.write_ledger(
                root,
                self.landed_row("cpfr-085", "CPFR-085", evidence_digest=digests["cpfr-085"]),
                self.landed_row("cpfr-086", "CPFR-086", evidence_digest=digests["cpfr-086"]),
                self.landed_row("cpfr-087", "CPFR-087", evidence_digest=digests["cpfr-087"]),
                self.landed_row("cpfr-088", "CPFR-088", evidence_digest=digests["cpfr-088"]),
                self.landed_row("cpfr-089", "CPFR-089", evidence_digest=digests["cpfr-089"]),
                self.landed_row("cpfr-090", "CPFR-090", evidence_digest=digests["cpfr-090"]),
                self.landed_row("cpfr-091", "CPFR-091", evidence_digest=digests["cpfr-091"]),
                self.landed_row(
                    "phase-8-plan-review",
                    "PHASE-8-PLAN-REVIEW",
                    evidence_digest=digests["phase-8-plan-review"],
                    landing_commit="c" * 40,
                ),
            )

            with mock.patch.object(goal_validation, "_git_commit_exists", return_value=False):
                with self.assertRaisesRegex(ValueError, "landed row landing_commit does not exist"):
                    goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

            with mock.patch.object(goal_validation, "_git_commit_exists", return_value=True), mock.patch.object(
                goal_validation,
                "_git_is_ancestor",
                side_effect=lambda repository_root, older, newer_ref, **kwargs: False if older == ("c" * 40) else True,
            ):
                with self.assertRaisesRegex(ValueError, "landed row landing_commit is not an ancestor of master"):
                    goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

    def test_landed_rows_require_existing_evidence_and_valid_phase8_review(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_complete_goal(root)
            digests = self.materialize_landed_evidence(root)
            missing = root / self.LANDED_VERIFIER_PATHS["cpfr-090"]
            missing.unlink()
            ledger = self.write_ledger(
                root,
                self.landed_row("cpfr-085", "CPFR-085", evidence_digest=digests["cpfr-085"]),
                self.landed_row("cpfr-086", "CPFR-086", evidence_digest=digests["cpfr-086"]),
                self.landed_row("cpfr-087", "CPFR-087", evidence_digest=digests["cpfr-087"]),
                self.landed_row("cpfr-088", "CPFR-088", evidence_digest=digests["cpfr-088"]),
                self.landed_row("cpfr-089", "CPFR-089", evidence_digest=digests["cpfr-089"]),
                self.landed_row("cpfr-090", "CPFR-090", evidence_digest=digests["cpfr-090"]),
                self.landed_row("cpfr-091", "CPFR-091", evidence_digest=digests["cpfr-091"]),
                self.landed_row(
                    "phase-8-plan-review",
                    "PHASE-8-PLAN-REVIEW",
                    evidence_digest=digests["phase-8-plan-review"],
                ),
            )

            with self.assertRaisesRegex(ValueError, "landed row evidence is missing"):
                goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_complete_goal(root)
            digests = self.materialize_landed_evidence(root)
            self.write_text(
                root,
                self.LANDED_VERIFIER_PATHS["cpfr-091"],
                "mutated inventory\n",
            )
            ledger = self.write_ledger(
                root,
                self.landed_row("cpfr-085", "CPFR-085", evidence_digest=digests["cpfr-085"]),
                self.landed_row("cpfr-086", "CPFR-086", evidence_digest=digests["cpfr-086"]),
                self.landed_row("cpfr-087", "CPFR-087", evidence_digest=digests["cpfr-087"]),
                self.landed_row("cpfr-088", "CPFR-088", evidence_digest=digests["cpfr-088"]),
                self.landed_row("cpfr-089", "CPFR-089", evidence_digest=digests["cpfr-089"]),
                self.landed_row("cpfr-090", "CPFR-090", evidence_digest=digests["cpfr-090"]),
                self.landed_row("cpfr-091", "CPFR-091", evidence_digest=digests["cpfr-091"]),
                self.landed_row(
                    "phase-8-plan-review",
                    "PHASE-8-PLAN-REVIEW",
                    evidence_digest=digests["phase-8-plan-review"],
                ),
            )

            with self.assertRaisesRegex(ValueError, "landed row evidence digest mismatch"):
                goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_complete_goal(root)
            digests = self.materialize_landed_evidence(root)
            self.write_text(
                root,
                self.LANDED_VERIFIER_PATHS["phase-8-plan-review"],
                json.dumps(
                    {
                        "schema_version": "crouzeix-plan-review/v1",
                        "spec_status": "failed",
                        "standards_status": "passed",
                        "retrospective_path": "docs/workstream/crouzeix-proof-reproduction/retrospective-003.json",
                        "retrospective_sha256": "0" * 64,
                        "plan_path": "docs/superpowers/plans/2026-08-23-crouzeix-end-to-end-proof-goal.md",
                        "plan_sha256": "0" * 64,
                    }
                ) + "\n",
            )
            ledger = self.write_ledger(
                root,
                self.landed_row("cpfr-085", "CPFR-085", evidence_digest=digests["cpfr-085"]),
                self.landed_row("cpfr-086", "CPFR-086", evidence_digest=digests["cpfr-086"]),
                self.landed_row("cpfr-087", "CPFR-087", evidence_digest=digests["cpfr-087"]),
                self.landed_row("cpfr-088", "CPFR-088", evidence_digest=digests["cpfr-088"]),
                self.landed_row("cpfr-089", "CPFR-089", evidence_digest=digests["cpfr-089"]),
                self.landed_row("cpfr-090", "CPFR-090", evidence_digest=digests["cpfr-090"]),
                self.landed_row("cpfr-091", "CPFR-091", evidence_digest=digests["cpfr-091"]),
                self.landed_row(
                    "phase-8-plan-review",
                    "PHASE-8-PLAN-REVIEW",
                    evidence_digest="sha256:" + sha256_bytes(
                        (root / self.LANDED_VERIFIER_PATHS["phase-8-plan-review"]).read_bytes()
                    ),
                ),
            )

            with self.assertRaisesRegex(ValueError, "Phase 8 plan-review row is invalid"):
                goal_validation.validate_goal(goal, execution_ledger_path=ledger, repository_root=root)

    def test_reader_receipt_validation_rejects_corrupt_receipt_and_member_bytes(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "atlas/src/content/generated").mkdir(parents=True)
            (root / "atlas/dist").mkdir(parents=True)
            corpus = root / "atlas/src/content/generated/corpus.json"
            html = root / "atlas/dist/harp-atlas.html"
            receipt = root / "atlas/dist/harp-atlas.receipt.json"
            corpus.write_text("{\"ok\":true}\n", encoding="utf-8")
            html.write_text("<html></html>\n", encoding="utf-8")
            receipt.write_text(
                json.dumps(
                    {
                        "schema_version": "harp-atlas-export/v1",
                        "corpus_sha256": sha256_bytes(corpus.read_bytes()),
                        "app_inputs_sha256": "0" * 64,
                        "html_sha256": "0" * 64,
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(ValueError, "atlas receipt html_sha256 mismatch"):
                goal_validation._validate_reader_surfaces(root)

            receipt.write_text(
                json.dumps(
                    {
                        "schema_version": "harp-atlas-export/v1",
                        "corpus_sha256": sha256_bytes(corpus.read_bytes()),
                        "app_inputs_sha256": "0" * 64,
                        "html_sha256": sha256_bytes(html.read_bytes()),
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            corpus.write_text("{\"ok\":false}\n", encoding="utf-8")

            with self.assertRaisesRegex(ValueError, "atlas receipt corpus_sha256 mismatch"):
                goal_validation._validate_reader_surfaces(root)

    def test_git_adapter_rejects_timeout_overflow_and_malformed_output(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        class FakeRunner:
            def __init__(self, result: subprocess.CompletedProcess[str] | None = None, *, timeout: bool = False):
                self.result = result
                self.timeout = timeout
                self.calls: list[list[str]] = []

            def __call__(self, argv: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
                self.calls.append(argv)
                if self.timeout:
                    raise subprocess.TimeoutExpired(argv, kwargs.get("timeout"))
                assert self.result is not None
                return self.result

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with goal_validation.git_session() as session:
                timeout_runner = FakeRunner(timeout=True)
                with self.assertRaisesRegex(ValueError, "git command timed out"):
                    goal_validation._run_git_checked(root, session, timeout_runner, "rev-parse", "HEAD")

                overflow_runner = FakeRunner(
                    subprocess.CompletedProcess(["/usr/bin/git"], 0, stdout=("a" * 9001), stderr="")
                )
                with self.assertRaisesRegex(ValueError, "git stdout exceeds cap"):
                    goal_validation._run_git_checked(root, session, overflow_runner, "rev-parse", "HEAD")

                bad_runner = FakeRunner(
                    subprocess.CompletedProcess(["/usr/bin/git"], 0, stdout="not-a-commit\n", stderr="")
                )
                with self.assertRaisesRegex(ValueError, "git output is malformed"):
                    goal_validation._git_commit_exists(root, "a" * 40, session=session, runner=bad_runner)

            self.assertFalse(Path(session.directory.name).exists(), "git tempdir leaked")

    def test_git_session_uses_minimal_allowlist_env(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        original = {key: os.environ.get(key) for key in (
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_COMMON_DIR",
            "GIT_OBJECT_DIRECTORY",
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
            "GIT_CONFIG_COUNT",
            "GIT_CONFIG_KEY_0",
            "GIT_CONFIG_VALUE_0",
            "GIT_TRACE",
            "GIT_SSH_COMMAND",
        )}
        try:
            os.environ.update({
                "GIT_DIR": "/tmp/evil.git",
                "GIT_WORK_TREE": "/tmp/evil-worktree",
                "GIT_COMMON_DIR": "/tmp/common",
                "GIT_OBJECT_DIRECTORY": "/tmp/objects",
                "GIT_ALTERNATE_OBJECT_DIRECTORIES": "/tmp/alt",
                "GIT_CONFIG_COUNT": "1",
                "GIT_CONFIG_KEY_0": "evil.key",
                "GIT_CONFIG_VALUE_0": "evil",
                "GIT_TRACE": "1",
                "GIT_SSH_COMMAND": "evil-ssh",
            })
            with tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                self.init_git_repo_with_commit(root)
                captured: dict[str, object] = {}

                def fake_runner(argv: list[str], **kwargs: object) -> subprocess.CompletedProcess[str]:
                    captured["argv"] = argv
                    captured["env"] = kwargs["env"]
                    return subprocess.CompletedProcess(argv, 0, stdout=("a" * 40) + "\n", stderr="")

                with goal_validation.git_session() as session:
                    goal_validation._git_commit_exists(root, "a" * 40, session=session, runner=fake_runner)

                env = captured["env"]
                assert isinstance(env, dict)
                self.assertEqual(captured["argv"][0], "/usr/bin/git")
                self.assertEqual(captured["argv"][2], str(root.resolve()))
                self.assertEqual(env["PATH"], "/usr/bin:/bin")
                self.assertEqual(env["LC_ALL"], "C")
                self.assertEqual(env["GIT_CONFIG_NOSYSTEM"], "1")
                self.assertTrue(env["GIT_CONFIG_GLOBAL"].endswith("/gitconfig"))
                self.assertEqual(env["HOME"], str(Path(env["GIT_CONFIG_GLOBAL"]).parent))
                for hostile in original:
                    self.assertNotIn(hostile, env)
        finally:
            for key, value in original.items():
                if value is None:
                    os.environ.pop(key, None)
                else:
                    os.environ[key] = value

    def test_git_production_runner_bounds_stdout_stderr_and_timeout(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            script = root / "fake-git.sh"
            with warnings.catch_warnings(record=True) as caught:
                warnings.simplefilter("always")

                script.write_text(
                    "#!/bin/sh\n"
                    "python3 - <<'PY'\n"
                    "import sys\n"
                    "import time\n"
                    "sys.stdout.write('a' * 9001)\n"
                    "sys.stdout.flush()\n"
                    "time.sleep(30)\n"
                    "PY\n",
                    encoding="utf-8",
                )
                script.chmod(0o755)
                with goal_validation.git_session() as session:
                    with self.assertRaisesRegex(ValueError, "git stdout exceeds cap"):
                        goal_validation._run_git_checked(root, session, None, "rev-parse", "HEAD", executable=script)

                script.write_text(
                    "#!/bin/sh\n"
                    "python3 - <<'PY'\n"
                    "import sys\n"
                    "import time\n"
                    "sys.stderr.write('b' * 9001)\n"
                    "sys.stderr.flush()\n"
                    "time.sleep(30)\n"
                    "PY\n",
                    encoding="utf-8",
                )
                script.chmod(0o755)
                with goal_validation.git_session() as session:
                    with self.assertRaisesRegex(ValueError, "git stderr exceeds cap"):
                        goal_validation._run_git_checked(root, session, None, "rev-parse", "HEAD", executable=script)

                marker = root / "child-marker"
                script.write_text(
                    "#!/bin/sh\n"
                    "(sleep 30) &\n"
                    "echo $! > " + str(marker) + "\n"
                    "sleep 30\n",
                    encoding="utf-8",
                )
                script.chmod(0o755)
                with goal_validation.git_session() as session:
                    with self.assertRaisesRegex(ValueError, "git command timed out"):
                        goal_validation._run_git_checked(root, session, None, "rev-parse", "HEAD", executable=script)
                gc.collect()

            resource_warnings = [
                warning
                for warning in caught
                if issubclass(warning.category, ResourceWarning)
                and "subprocess" in str(warning.message)
            ]
            self.assertEqual(resource_warnings, [])
            self.assertTrue(marker.exists())
            child_pid = int(marker.read_text(encoding="utf-8").strip())
            with self.assertRaises(ProcessLookupError):
                os.kill(child_pid, 0)

    def test_descriptor_relative_readers_reject_symlink_hardlink_oversize_and_parent_swap(self) -> None:
        from labs.crouzeix_proof_reproduction import goal_validation

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal_target = self.write_text(root, "real-goal.md", self.make_in_progress_goal(root).read_text(encoding="utf-8"))
            goal_link = root / "goal-link.md"
            goal_link.symlink_to(goal_target)
            with self.assertRaisesRegex(ValueError, "goal plan path is invalid"):
                goal_validation.parse_goal_footer(goal_link, repository_root=root)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            ledger = self.write_ledger(root)
            hard = root / "hard-ledger.tsv"
            os.link(ledger, hard)
            with self.assertRaisesRegex(ValueError, "execution ledger path is invalid"):
                goal_validation.validate_goal(self.make_in_progress_goal(root), execution_ledger_path=hard, repository_root=root)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            real_goal_dir = root / "real-goals"
            real_goal_dir.mkdir()
            self.write_text(
                real_goal_dir,
                "goal.md",
                self.make_in_progress_goal(root).read_text(encoding="utf-8"),
            )
            goal_link_dir = root / "goal-link-dir"
            goal_link_dir.symlink_to(real_goal_dir, target_is_directory=True)
            ledger = self.write_ledger(root)
            with self.assertRaisesRegex(ValueError, "goal plan path is invalid"):
                goal_validation.validate_goal(
                    goal_link_dir / "goal.md",
                    execution_ledger_path=ledger,
                    repository_root=root,
                )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            goal = self.make_in_progress_goal(root)
            real_ledger_dir = root / "real-ledgers"
            real_ledger_dir.mkdir()
            real_ledger = real_ledger_dir / "execution-ledger.tsv"
            real_ledger.write_text(
                "phase\tticket\tstate\tbranch\tworktree\tbase_commit\tcandidate_commit\tverifier\texit_code\tevidence_digest\tlanding_commit\tnote\n",
                encoding="utf-8",
            )
            ledger_link_dir = root / "ledger-link-dir"
            ledger_link_dir.symlink_to(real_ledger_dir, target_is_directory=True)
            with self.assertRaisesRegex(ValueError, "execution ledger path is invalid"):
                goal_validation.validate_goal(
                    goal,
                    execution_ledger_path=ledger_link_dir / "execution-ledger.tsv",
                    repository_root=root,
                )

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.write_bytes(root, "atlas/src/content/generated/corpus.json", b"a" * (1024 * 1024 + 1))
            self.write_text(root, "atlas/dist/harp-atlas.html", "<html></html>\n")
            self.write_text(
                root,
                "atlas/dist/harp-atlas.receipt.json",
                json.dumps(
                    {
                        "schema_version": "harp-atlas-export/v1",
                        "corpus_sha256": "0" * 64,
                        "app_inputs_sha256": "0" * 64,
                        "html_sha256": "0" * 64,
                    }
                ) + "\n",
            )
            with self.assertRaisesRegex(ValueError, "atlas artifact exceeds cap"):
                goal_validation._validate_reader_surfaces(root)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            current = root / "docs/workstream"
            current.mkdir(parents=True)
            swapped = root / "swapped"
            swapped.mkdir()
            target = current / "crouzeix-proof-reproduction"
            target.mkdir()
            artifact = target / "program-verification-003.json"
            artifact.write_text("ok\n", encoding="utf-8")
            original = os.open(current, os.O_RDONLY)
            try:
                os.rename(target, swapped / "moved")
                os.symlink(swapped / "moved", target)
                with self.assertRaisesRegex(ValueError, "landed row verifier path is invalid"):
                    goal_validation._read_bounded_artifact(root, "docs/workstream/crouzeix-proof-reproduction/program-verification-003.json")
            finally:
                os.close(original)
