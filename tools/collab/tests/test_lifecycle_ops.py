"""Public CLI proof of configured overrides and Corenius ticket closure."""
import json
from pathlib import Path
import unittest

import test_collab as fixtures


class LifecycleOpsTests(unittest.TestCase):
    setUp = fixtures.CollabTests.setUp
    run_process = fixtures.CollabTests.run_process
    git = fixtures.CollabTests.git
    commit = fixtures.CollabTests.commit
    def cli(self, *args, **kwargs):
        if args and args[0] == "land" and "--no-push" not in args:
            args = (*args, "--no-push")
        return fixtures.CollabTests.cli(self, *args, **kwargs)
    init = fixtures.CollabTests.init
    ticket = fixtures.CollabTests.ticket
    claim = fixtures.CollabTests.claim
    contribute = fixtures.CollabTests.contribute
    effort_ticket = fixtures.CollabTests.effort_ticket
    open_corenius_attempt = fixtures.CollabTests.open_corenius_attempt

    def configured_attempt(self):
        config = self.cli("init", "--trunk", "main", "--check-json", '["true"]',
                          "--override-owners-json", '["operator"]',
                          "--workspace-root", str(self.root / "workspaces"))
        self.assertEqual(config["mainline_override_owners"], ["operator"])
        self.ticket()
        self.commit("ticket")
        attempt = self.claim()
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        return self.cli("check", attempt["id"])

    def test_configured_owner_override_is_recorded_and_unknown_owner_is_refused(self):
        checked = self.configured_attempt()
        result = self.cli("land", checked["id"], "--owner-mainline-override", "unknown", ok=False)
        self.assertIn("configured", result.stderr)
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])
        landed = self.cli("land", checked["id"], "--owner-mainline-override", "operator")
        self.assertEqual(landed["state"], "landed")
        events = self.cli("show", checked["id"])["events"]
        overrides = [item for item in events if item["kind"] == "mainline_override"]
        self.assertEqual(len(overrides), 1)
        self.assertEqual(overrides[0]["details"]["owner"], "operator")
        self.assertEqual(overrides[0]["details"]["subject"], checked["integration"])

    def test_corenius_check_preserves_ticket_and_landing_commits_tracker_resolution(self):
        self.init()
        ticket = self.effort_ticket()
        self.commit("effort ticket")
        attempt = self.open_corenius_attempt()
        claimed_bytes = ticket.read_bytes()
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        checked_ticket = Path(checked["check_path"]) / ticket.relative_to(self.repo)
        self.assertEqual(checked_ticket.read_bytes(), claimed_bytes)
        review = json.dumps({"reviewer": "operator", "standards": "passed", "spec": "passed", "criteria": [1]})
        landed = self.cli("land", attempt["id"], "--review-json", review)
        self.assertEqual(landed["state"], "landed")
        self.assertIn("Status: resolved", ticket.read_text())
        self.assertIn("- [x] observable result", ticket.read_text())
        self.assertEqual(self.git("status", "--porcelain"), "")
        record_commit = self.git("rev-parse", "HEAD")
        self.assertNotEqual(record_commit, checked["integration"])
        self.assertEqual(self.git("rev-parse", "HEAD^"), checked["integration"])
        acceptance = self.repo / "docs/efforts/sample-effort/evidence/acceptance-01.jsonl"
        self.assertEqual(json.loads(acceptance.read_text())["candidate"], checked["integration"])
        again = self.cli("recover", attempt["id"], "--confirm-stopped")
        self.assertEqual(again["state"], "landed")
        self.assertEqual(self.git("rev-parse", "HEAD"), record_commit)


    def test_corenius_landing_requires_complete_explicit_review(self):
        self.init()
        self.effort_ticket()
        self.commit("effort ticket")
        attempt = self.open_corenius_attempt()
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        result = self.cli("land", attempt["id"], ok=False)
        self.assertIn("review-json", result.stderr)
        review = json.dumps({"reviewer": "operator", "standards": "passed", "spec": "passed", "criteria": []})
        self.cli("land", attempt["id"], "--review-json", review, ok=False)
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])

    def test_corenius_records_use_repository_local_commit_identity(self):
        for key in ("GIT_AUTHOR_NAME", "GIT_AUTHOR_EMAIL", "GIT_COMMITTER_NAME", "GIT_COMMITTER_EMAIL"):
            self.env.pop(key, None)
        self.init()
        ticket = self.effort_ticket()
        self.commit("effort ticket")
        attempt = self.open_corenius_attempt()
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        review = json.dumps({"reviewer": "operator", "standards": "passed", "spec": "passed", "criteria": [1]})
        landed = self.cli("land", attempt["id"], "--review-json", review)
        self.assertEqual(landed["state"], "landed")
        self.assertIn("Status: resolved", ticket.read_text())
        self.assertEqual(self.git("log", "-1", "--format=%an <%ae>|%cn <%ce>"),
                         "Collab Test <collab@example.invalid>|Collab Test <collab@example.invalid>")
        self.assertEqual(self.git("rev-parse", "HEAD^"), checked["integration"])

    def test_missing_bookkeeping_identity_refuses_before_code_publication(self):
        self.init()
        self.effort_ticket()
        self.commit("effort ticket")
        attempt = self.open_corenius_attempt()
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        self.git("config", "user.name", "")
        self.git("config", "user.email", "")
        env = dict(self.env)
        for key in ("GIT_AUTHOR_NAME", "GIT_AUTHOR_EMAIL", "GIT_COMMITTER_NAME", "GIT_COMMITTER_EMAIL"):
            env.pop(key, None)
        review = json.dumps({"reviewer": "operator", "standards": "passed", "spec": "passed", "criteria": [1]})
        self.cli("land", attempt["id"], "--review-json", review, env=env, ok=False)
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])
        self.assertEqual(self.cli("show", attempt["id"])["state"], "verified")

    def test_corenius_record_crash_recovers_without_repeating_landing(self):
        self.init()
        ticket = self.effort_ticket()
        self.commit("effort ticket")
        attempt = self.open_corenius_attempt()
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        review = json.dumps({"reviewer": "operator", "standards": "passed", "spec": "passed", "criteria": [1]})
        self.cli("land", attempt["id"], "--review-json", review, failpoint="before_records_land", ok=False)
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["integration"])
        recovered = self.cli("recover", attempt["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "landed")
        self.assertIn("Status: resolved", ticket.read_text())
        self.assertEqual(self.git("rev-parse", "HEAD^"), checked["integration"])
        record_head = self.git("rev-parse", "HEAD")
        self.cli("recover", attempt["id"], "--confirm-stopped")
        self.assertEqual(self.git("rev-parse", "HEAD"), record_head)

    def test_override_with_effort_exports_attributed_committed_record(self):
        self.cli("init", "--trunk", "main", "--check-json", '["true"]',
                 "--override-owners-json", '["operator"]')
        self.effort_ticket()
        self.ticket()
        self.commit("tickets")
        attempt = self.claim()
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        self.cli("land", attempt["id"], "--owner-mainline-override", "operator", "--effort", "sample-effort")
        export = self.repo / "docs/efforts/sample-effort/evidence/mainline-landing-overrides.jsonl"
        record = json.loads(export.read_text())
        self.assertEqual(record["owner"], "operator")
        self.assertEqual(record["candidate"], checked["integration"])
        self.assertEqual(self.git("status", "--porcelain"), "")


    def test_unknown_recorded_worker_run_cannot_be_abandoned(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        run = self.root / "unknown-run"
        run.mkdir()
        attempt = self.cli("open-attempt", "T-one", "--owner", "test-worker", "--worker-run-dir", str(run))
        refused = self.cli("abandon", attempt["id"], "--confirm-stopped", ok=False)
        self.assertIn("unknown", refused.stderr)
        self.assertEqual(self.cli("show", attempt["id"])["state"], "working")


    def test_transferred_corenius_claim_recovers_interrupted_fresh_allocation(self):
        self.init()
        self.effort_ticket()
        self.commit("effort ticket")
        old = self.open_corenius_attempt()
        self.contribute(old)
        (self.repo / "other.txt").write_text("new trunk\n")
        self.commit("trunk moves")
        self.cli("refresh", old["id"], "--confirm-stopped", failpoint="after_claim_record", ok=False)
        fresh = next(row for row in self.cli("status")["attempts"] if row["state"] == "allocating")
        recovered = self.cli("recover", fresh["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "working")
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=recovered["path"]), recovered["base"])
        self.assertTrue(Path(old["path"]).is_dir())


    def test_tracker_failure_reports_landed_records_incomplete_and_recovers(self):
        self.init()
        ticket = self.effort_ticket()
        self.commit("effort ticket")
        attempt = self.open_corenius_attempt()
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        review = json.dumps({"reviewer": "operator", "standards": "passed", "spec": "passed", "criteria": [1]})
        env = dict(self.env, CORENIUS_TRACKER_NOW="invalid")
        partial = self.cli("land", attempt["id"], "--review-json", review, env=env)
        self.assertEqual(partial["state"], "landed_records_incomplete")
        self.assertTrue(partial["landed"])
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["integration"])
        self.assertIn("Status: claimed", ticket.read_text())
        recovered = self.cli("recover", attempt["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "landed")
        self.assertIn("Status: resolved", ticket.read_text())
        self.assertEqual(self.git("rev-parse", "HEAD^"), checked["integration"])



if __name__ == "__main__":
    unittest.main()
