"""acceptance.py records review decisions for exact Candidates."""

from __future__ import annotations

import io
import json
import os
import subprocess
import sys
import tempfile
import textwrap
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import acceptance  # noqa: E402
import claims  # noqa: E402


def git(cwd, *args):
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True, text=True).stdout


class AcceptanceTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Path(self.tmp.name) / "repo"
        self.repo.mkdir()
        git(self.repo, "init", "-q", "-b", "main")
        git(self.repo, "config", "user.email", "a@b")
        git(self.repo, "config", "user.name", "a")
        effort = self.repo / "docs" / "efforts" / "sample"
        (effort / "tickets").mkdir(parents=True)
        (effort / "evidence").mkdir()
        (effort / "EFFORT.md").write_text(
            "# Effort: sample\n\n"
            "Status: active\n"
            "Base: main\n"
            "Integration branch: effort/sample\n"
        )
        (effort / "tickets" / "01-one.md").write_text(
            "# 01 one: One\n\n"
            "Type: feature\n"
            "Status: claimed\n"
            "Owner: worker-01\n"
            "Blocked by:\n"
            "Claimed: 2026-10-05T10:00:00Z expires 2026-10-06T10:00:00Z\n\n"
            "## Acceptance\n"
            "- [ ] works\n\n"
            "## Comments\n"
        )
        (self.repo / "source.txt").write_text("alpha\n")
        git(self.repo, "add", ".")
        git(self.repo, "commit", "-qm", "init")
        self.candidate = git(self.repo, "rev-parse", "HEAD").strip()

    def tearDown(self):
        self.tmp.cleanup()

    def tree_listing(self, root: Path) -> list[str]:
        return sorted(str(path.relative_to(root)) for path in root.rglob("*"))

    def call(self, argv):
        return self.call_from(self.repo, argv)

    def call_from(self, repo: Path, argv):
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            code = acceptance.main(["--repo", str(repo), *argv])
        return code, out.getvalue(), err.getvalue()

    def move_effort_to_integration_worktree(self) -> Path:
        integration = Path(self.tmp.name) / "integration"
        git(self.repo, "worktree", "add", "-q", "-b", "effort/sample", str(integration), "main")
        git(self.repo, "rm", "-qr", "docs/efforts")
        git(self.repo, "commit", "-qm", "main without effort record")
        return integration

    def move_records_to_branch(self) -> Path:
        effort = self.repo / "docs/efforts/sample"
        (effort / "EFFORT.md").write_text(
            "# Effort: sample\n\n"
            "Status: active\n"
            "Base: main\n"
            "Integration branch: effort/sample\n"
            "Record branch: effort/sample-records\n"
            "Lead: lead\n"
        )
        git(self.repo, "add", "docs/efforts/sample/EFFORT.md")
        git(self.repo, "commit", "-qm", "add effort record")
        records = self.repo.parent / "records"
        git(self.repo, "worktree", "add", "-q", "-b", "effort/sample-records", str(records), "main")
        git(self.repo, "rm", "-qr", "docs/efforts/sample")
        git(self.repo, "commit", "-qm", "remove effort record from code")
        return records

    def commit_worker_report(self, text: str) -> tuple[Path, Path, str]:
        worker = self.repo.parent / "worker"
        git(self.repo, "worktree", "add", "-q", "-b", "worker/01", str(worker), "HEAD")
        report = worker / "docs" / "efforts" / "sample" / "evidence" / "01-report.md"
        report.parent.mkdir(parents=True, exist_ok=True)
        report.write_text(text)
        git(worker, "add", "docs/efforts/sample/evidence/01-report.md")
        git(worker, "commit", "-qm", "ticket 01: ready for review")
        candidate = git(worker, "rev-parse", "HEAD").strip()
        return worker, report, candidate

    def test_record_requires_both_axes_and_all_required_checks(self):
        code, out, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=passed",
            ]
        )
        self.assertEqual(code, acceptance.ERROR)
        self.assertEqual(out, "")
        self.assertIn("missing review axis: spec", err)

        code, out, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
            ]
        )
        self.assertEqual(code, acceptance.ERROR)
        self.assertEqual(out, "")
        self.assertIn("missing required check: unit", err)

    def test_record_writes_acceptance_and_summary_counts_non_lead_review(self):
        code, out, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "reviewer-1",
                "--reviewer-context",
                "other",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=passed",
            ]
        )
        self.assertEqual(code, acceptance.OK, err)
        printed = json.loads(out)
        record = json.loads(Path(printed["recorded"]).read_text().splitlines()[-1])
        self.assertEqual(record["candidate"], self.candidate)
        self.assertEqual(record["reviewer"], "reviewer-1")
        self.assertEqual(record["reviewer_context"], "other")
        self.assertTrue(record["accepted"])
        self.assertEqual(record["axes"], {"standards": "passed", "spec": "passed"})
        self.assertEqual(record["required_checks"], [{"name": "unit", "result": "passed"}])

        code, out, err = self.call(["summary", "sample"])
        self.assertEqual(code, acceptance.OK, err)
        summary = json.loads(out)
        self.assertEqual(summary["tickets_reviewed_by_other_context"], 1)
        self.assertEqual(summary["tickets_accepted"], 1)

    def test_record_with_tmpdir_unset_does_not_use_default_temp_root(self):
        forbidden = Path(self.tmp.name) / "forbidden-host-tmp"
        forbidden.mkdir()
        before = self.tree_listing(forbidden)
        old_tempdir = tempfile.tempdir
        old_tmpdir = os.environ.pop("TMPDIR", None)
        try:
            tempfile.tempdir = str(forbidden)
            code, _, err = self.call(
                [
                    "record",
                    "sample",
                    "01",
                    "--candidate",
                    self.candidate,
                    "--reviewer",
                    "reviewer-1",
                    "--reviewer-context",
                    "other",
                    "--standards",
                    "passed",
                    "--spec",
                    "passed",
                    "--required-check",
                    "unit",
                    "--check",
                    "unit=passed",
                ]
            )
        finally:
            tempfile.tempdir = old_tempdir
            if old_tmpdir is not None:
                os.environ["TMPDIR"] = old_tmpdir

        self.assertEqual(code, acceptance.OK, err)
        self.assertEqual(self.tree_listing(forbidden), before)

    def test_commands_from_main_checkout_resolve_effort_worktree(self):
        integration = self.move_effort_to_integration_worktree()
        findings = integration / "docs" / "efforts" / "sample" / "evidence" / "review-main.md"
        findings.parent.mkdir(parents=True, exist_ok=True)
        findings.write_text("Spec finding.\n")

        code, out, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "reviewer-1",
                "--reviewer-context",
                "other",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=passed",
            ]
        )
        self.assertEqual(code, acceptance.OK, err)
        accepted_path = Path(json.loads(out)["recorded"])
        self.assertTrue(accepted_path.is_relative_to(integration))

        code, out, err = self.call(
            [
                "review",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "failed",
                "--findings-file",
                str(findings),
            ]
        )
        self.assertEqual(code, acceptance.OK, err)
        review_path = Path(json.loads(out)["recorded"])
        self.assertTrue(review_path.is_relative_to(integration))

        code, out, err = self.call(
            [
                "request-rework",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--kind",
                "review",
                "--findings-file",
                str(findings),
                "--actor",
                "lead",
            ]
        )
        self.assertEqual(code, acceptance.OK, err)
        rework_path = Path(json.loads(out)["recorded"])
        self.assertTrue(rework_path.is_relative_to(integration))

        code, out, err = self.call(["summary", "sample"])
        self.assertEqual(code, acceptance.OK, err)
        self.assertEqual(json.loads(out)["tickets_accepted"], 1)
        self.assertFalse((self.repo / "docs" / "efforts" / "sample" / "EFFORT.md").exists())
        self.assertEqual(git(self.repo, "status", "--porcelain", "--untracked-files=all"), "")

    def test_record_writes_to_effort_record_branch_when_code_checkout_has_no_effort(self):
        records = self.move_records_to_branch()

        code, out, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=passed",
            ]
        )

        self.assertEqual(code, acceptance.OK, err)
        printed = json.loads(out)
        self.assertTrue(str(printed["recorded"]).startswith(str(records)))
        self.assertTrue((records / "docs/efforts/sample/evidence/acceptance-01.jsonl").exists())
        self.assertFalse((self.repo / "docs/efforts/sample/EFFORT.md").exists())

    def test_summary_uses_record_branch_from_every_checkout(self):
        records = self.move_records_to_branch()
        stale = self.repo.parent / "stale-copy"
        git(self.repo, "worktree", "add", "-q", "-b", "agent/stale-copy", str(stale), "effort/sample-records")

        code, out, err = self.call_from(
            records,
            [
                "record",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "reviewer-1",
                "--reviewer-context",
                "other",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=passed",
            ],
        )
        self.assertEqual(code, acceptance.OK, err)

        summaries = []
        for checkout in (self.repo, records, stale):
            code, out, err = self.call_from(checkout, ["summary", "sample"])
            self.assertEqual(code, acceptance.OK, (checkout, err))
            summaries.append(json.loads(out))

        self.assertEqual(summaries[0], summaries[1])
        self.assertEqual(summaries[1], summaries[2])
        self.assertEqual(summaries[0]["tickets_accepted"], 1)

    def test_record_imports_worker_report_from_exact_candidate_to_record_branch(self):
        records = self.move_records_to_branch()
        _, report, candidate = self.commit_worker_report(
            "# Ticket 01 report\n\n"
            "## Built\n"
            "candidate report\n\n"
            "## For later tickets\n"
            "- to 02: Carry candidate finding.\n"
        )
        report.write_text("uncommitted local drift\n")

        code, out, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=passed",
            ]
        )

        self.assertEqual(code, acceptance.OK, err)
        imported = records / "docs/efforts/sample/evidence/01-report-round-1.md"
        self.assertEqual(imported.read_text().splitlines()[3], "candidate report")
        self.assertNotIn("uncommitted local drift", imported.read_text())
        record = json.loads(Path(json.loads(out)["recorded"]).read_text().splitlines()[-1])
        self.assertEqual(record["worker_report"]["path"], "docs/efforts/sample/evidence/01-report-round-1.md")
        self.assertEqual(record["worker_report"]["source_path"], "docs/efforts/sample/evidence/01-report.md")
        self.assertEqual(record["worker_report"]["source_commit"], candidate)

    def test_record_claim_sampling_uses_imported_worker_report_on_record_branch(self):
        records = self.move_records_to_branch()
        _, report, candidate = self.commit_worker_report(
            "# Ticket 01 report\n\n"
            "- [R] source exists `source.txt:1`\n"
        )
        claim = claims.parse_claims(report)[0]
        out_file = self.repo / "claim.out"
        out_file.write_text("alpha\n")
        with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            self.assertEqual(
                claims.main(["record", str(report), "--claim", claim.claim_id, "--passed", "--output", str(out_file)]),
                claims.OK,
            )

        code, out, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=passed",
                "--claims-report",
                str(report),
            ]
        )

        self.assertEqual(code, acceptance.OK, err)
        record = json.loads(Path(json.loads(out)["recorded"]).read_text().splitlines()[-1])
        imported = records / "docs/efforts/sample/evidence/01-report-round-1.md"
        expected_outcomes = claims.outcome_path(imported).resolve()
        self.assertEqual(record["claim_sampling"]["report"], "docs/efforts/sample/evidence/01-report-round-1.md")
        self.assertEqual(record["claim_sampling"]["outcomes"], str(expected_outcomes))
        self.assertTrue(expected_outcomes.exists())
        self.assertEqual(record["claim_sampling"]["summary"]["recorded"], 1)
        self.assertEqual(record["claim_sampling"]["summary"]["passed"], 1)

    def test_review_and_rework_records_use_moved_effort_record_branch(self):
        records = self.move_records_to_branch()
        findings = records / "docs" / "efforts" / "sample" / "evidence" / "review-main.md"
        findings.parent.mkdir(parents=True, exist_ok=True)
        findings.write_text("Spec finding.\n")

        code, out, err = self.call(
            [
                "review",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "failed",
                "--findings-file",
                str(findings),
            ]
        )
        self.assertEqual(code, acceptance.OK, err)
        review_path = Path(json.loads(out)["recorded"])
        self.assertTrue(review_path.is_relative_to(records))
        self.assertEqual(
            json.loads(review_path.read_text().splitlines()[-1])["findings"]["path"],
            "docs/efforts/sample/evidence/review-main.md",
        )

        code, out, err = self.call(
            [
                "request-rework",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--kind",
                "review",
                "--findings-file",
                str(findings),
                "--actor",
                "lead",
            ]
        )
        self.assertEqual(code, acceptance.OK, err)
        rework_path = Path(json.loads(out)["recorded"])
        self.assertTrue(rework_path.is_relative_to(records))
        self.assertFalse((self.repo / "docs/efforts/sample/EFFORT.md").exists())

    def test_summary_counts_non_lead_review_even_when_not_accepted(self):
        code, _, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "reviewer-1",
                "--reviewer-context",
                "other",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=failed",
            ]
        )
        self.assertEqual(code, acceptance.OK, err)

        code, out, err = self.call(["summary", "sample"])

        self.assertEqual(code, acceptance.OK, err)
        summary = json.loads(out)
        self.assertEqual(summary["tickets_reviewed_by_other_context"], 1)
        self.assertEqual(summary["tickets_accepted"], 0)

    def test_record_help_states_landing_order(self):
        result = subprocess.run(
            [sys.executable, str(Path(acceptance.__file__)), "record", "--help"],
            capture_output=True,
            text=True,
        )

        self.assertEqual(result.returncode, 0, result.stderr)
        help_text = " ".join(result.stdout.split()).lower()
        self.assertIn("record branch", result.stdout)
        self.assertIn("ticket landings need effort records on a record branch", help_text)
        self.assertIn("effort.py move-records <effort>", help_text)
        self.assertNotIn("may be committed before landing", result.stdout)
        self.assertNotIn("commit the record after landing", result.stdout)

    def test_record_with_owner_waiver_accepts_failing_required_check(self):
        code, out, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=failed",
                "--waiver-owner",
                "owner",
                "--waiver-reason",
                "known flaky fixture",
            ]
        )
        self.assertEqual(code, acceptance.OK, err)
        record = json.loads(Path(json.loads(out)["recorded"]).read_text().splitlines()[-1])
        self.assertTrue(record["accepted"])
        self.assertEqual(record["owner_waiver"]["owner"], "owner")
        self.assertEqual(record["owner_waiver"]["reason"], "known flaky fixture")

    def test_recorded_failing_required_check_without_waiver_is_not_accepted(self):
        code, out, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=failed",
            ]
        )
        self.assertEqual(code, acceptance.OK, err)
        record = json.loads(Path(json.loads(out)["recorded"]).read_text().splitlines()[-1])
        self.assertFalse(record["accepted"])
        self.assertIsNone(record["owner_waiver"])

    def test_recorded_failing_nonrequired_check_without_waiver_is_not_accepted(self):
        code, out, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=passed",
                "--check",
                "integration=failed",
            ]
        )
        self.assertEqual(code, acceptance.OK, err)
        record = json.loads(Path(json.loads(out)["recorded"]).read_text().splitlines()[-1])
        self.assertFalse(record["accepted"])
        self.assertEqual(
            record["required_checks"],
            [{"name": "integration", "result": "failed"}, {"name": "unit", "result": "passed"}],
        )
        ok, reason = acceptance.validate_acceptance(record, "sample", "01", self.candidate)
        self.assertFalse(ok)
        self.assertEqual(reason, "failing required check without owner waiver: integration")

    def test_record_links_claim_sampling_outcomes(self):
        report = self.repo / "report.md"
        report.write_text(
            textwrap.dedent(
                """\
                # Worker report

                - [R] source exists `source.txt:1`
                """
            )
        )
        claim = claims.parse_claims(report)[0]
        out_file = self.repo / "claim.out"
        out_file.write_text("alpha\n")
        with redirect_stdout(io.StringIO()), redirect_stderr(io.StringIO()):
            self.assertEqual(
                claims.main(["record", str(report), "--claim", claim.claim_id, "--passed", "--output", str(out_file)]),
                claims.OK,
            )

        code, out, err = self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=passed",
                "--claims-report",
                str(report),
            ]
        )
        self.assertEqual(code, acceptance.OK, err)
        record = json.loads(Path(json.loads(out)["recorded"]).read_text().splitlines()[-1])
        expected_outcomes = claims.outcome_path(report).resolve().relative_to(self.repo.resolve())
        self.assertEqual(record["claim_sampling"]["report"], "report.md")
        self.assertEqual(record["claim_sampling"]["outcomes"], str(expected_outcomes))
        self.assertTrue((self.repo / record["claim_sampling"]["outcomes"]).exists())
        self.assertEqual(record["claim_sampling"]["summary"]["recorded"], 1)
        self.assertEqual(record["claim_sampling"]["summary"]["passed"], 1)

    def test_rejection_record_does_not_accept_candidate_and_names_findings_file(self):
        findings = self.repo / "docs" / "efforts" / "sample" / "evidence" / "review-1.md"
        findings.write_text("Spec finding.\n")

        code, out, err = self.call(
            [
                "review",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "failed",
                "--findings-file",
                str(findings),
            ]
        )

        self.assertEqual(code, acceptance.OK, err)
        printed = json.loads(out)
        self.assertEqual(printed["accepted"], False)
        self.assertEqual(printed["round"], 1)
        record = json.loads(Path(printed["recorded"]).read_text().splitlines()[-1])
        self.assertEqual(record["schema"], "corenius.acceptance.v1")
        self.assertEqual(record["candidate"], self.candidate)
        self.assertEqual(record["axes"], {"standards": "passed", "spec": "failed"})
        self.assertFalse(record["accepted"])
        self.assertEqual(record["findings"]["path"], "docs/efforts/sample/evidence/review-1.md")
        self.assertEqual(record["findings"]["sha256"], acceptance.digest_file(findings))
        self.assertEqual(record["round"], 1)

        _, reason = acceptance.acceptance_for_candidate(self.repo, "sample", "01", self.candidate)
        self.assertEqual(reason, "review axis spec did not pass")

    def test_successor_candidate_needs_its_own_acceptance_record(self):
        first = self.candidate
        (self.repo / "source.txt").write_text("beta\n")
        git(self.repo, "add", "source.txt")
        git(self.repo, "commit", "-qm", "successor")
        second = git(self.repo, "rev-parse", "HEAD").strip()
        self.call(
            [
                "record",
                "sample",
                "01",
                "--candidate",
                first,
                "--reviewer",
                "lead",
                "--reviewer-context",
                "lead",
                "--standards",
                "passed",
                "--spec",
                "passed",
                "--required-check",
                "unit",
                "--check",
                "unit=passed",
            ]
        )

        self.assertIsNotNone(acceptance.acceptance_for_candidate(self.repo, "sample", "01", first)[0])
        self.assertEqual(acceptance.acceptance_for_candidate(self.repo, "sample", "01", second)[1], "record names another commit")

    def test_fourth_review_round_moves_ticket_to_ready_for_human_but_stale_redo_does_not_count(self):
        findings = self.repo / "docs" / "efforts" / "sample" / "evidence" / "review.md"
        findings.write_text("Needs another round.\n")
        for kind in ("review", "stale-base", "review", "review"):
            code, _, err = self.call(
                [
                    "request-rework",
                    "sample",
                    "01",
                    "--candidate",
                    self.candidate,
                    "--kind",
                    kind,
                    "--findings-file",
                    str(findings),
                    "--actor",
                    "lead",
                ]
            )
            self.assertEqual(code, acceptance.OK, (kind, err))

        code, out, err = self.call(
            [
                "request-rework",
                "sample",
                "01",
                "--candidate",
                self.candidate,
                "--kind",
                "review",
                "--findings-file",
                str(findings),
                "--actor",
                "lead",
            ]
        )

        self.assertEqual(code, acceptance.ERROR)
        self.assertEqual(out, "")
        self.assertIn("review-driven rework cap exhausted", err)
        ticket = self.repo / "docs" / "efforts" / "sample" / "tickets" / "01-one.md"
        text = ticket.read_text()
        self.assertIn("Triage: ready-for-human", text)
        self.assertIn("review-driven rework cap exhausted after 3 round(s)", text)
        records = [
            json.loads(line)
            for line in (self.repo / "docs" / "efforts" / "sample" / "evidence" / "rework-01.jsonl").read_text().splitlines()
        ]
        self.assertEqual([record["round"] for record in records], [1, 1, 2, 3])
        self.assertEqual([record["counts_toward_cap"] for record in records], [True, False, True, True])


if __name__ == "__main__":
    unittest.main()
