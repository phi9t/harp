"""Candidate-controlled record paths cannot redirect host writes."""
import json
from pathlib import Path
import unittest

import test_collab as fixtures
import test_lifecycle_ops as lifecycle


class RecordPathTests(unittest.TestCase):
    setUp = fixtures.CollabTests.setUp
    run_process = fixtures.CollabTests.run_process
    git = fixtures.CollabTests.git
    commit = fixtures.CollabTests.commit
    cli = lifecycle.LifecycleOpsTests.cli
    init = fixtures.CollabTests.init
    ticket = fixtures.CollabTests.ticket
    claim = fixtures.CollabTests.claim
    contribute = fixtures.CollabTests.contribute
    effort_ticket = fixtures.CollabTests.effort_ticket
    open_corenius_attempt = fixtures.CollabTests.open_corenius_attempt

    def refuse_record_file(self, name):
        self.init()
        self.effort_ticket()
        self.commit("effort ticket")
        attempt = self.open_corenius_attempt()
        victim = self.root / "host-record.jsonl"
        victim.write_text("{}\n")
        evidence = Path(attempt["path"]) / "docs/efforts/sample-effort/evidence"
        evidence.mkdir(parents=True)
        (evidence / name).symlink_to(victim)
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        review = json.dumps({"reviewer": "operator", "standards": "passed", "spec": "passed", "criteria": [1]})
        self.cli("land", attempt["id"], "--review-json", review, ok=False)
        self.assertEqual(victim.read_text(), "{}\n")
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])
        self.assertEqual(self.cli("show", attempt["id"])["state"], "verified")

    def test_symlinked_acceptance_file_is_refused_before_publication(self):
        self.refuse_record_file("acceptance-01.jsonl")

    def test_symlinked_landing_file_is_refused_before_publication(self):
        self.refuse_record_file("landing-01.jsonl")

    def test_symlinked_evidence_directory_is_refused_before_publication(self):
        self.init()
        self.effort_ticket()
        self.commit("effort ticket")
        attempt = self.open_corenius_attempt()
        victim = self.root / "host-records"
        victim.mkdir()
        (victim / "sentinel").write_text("preserve\n")
        evidence = Path(attempt["path"]) / "docs/efforts/sample-effort/evidence"
        evidence.symlink_to(victim, target_is_directory=True)
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        review = json.dumps({"reviewer": "operator", "standards": "passed", "spec": "passed", "criteria": [1]})
        self.cli("land", attempt["id"], "--review-json", review, ok=False)
        self.assertEqual(sorted(item.name for item in victim.iterdir()), ["sentinel"])
        self.assertEqual((victim / "sentinel").read_text(), "preserve\n")
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])

    def test_symlinked_override_export_is_refused_before_publication(self):
        self.cli("init", "--trunk", "main", "--check-json", '["true"]',
                 "--override-owners-json", '["operator"]')
        self.effort_ticket()
        self.ticket()
        self.commit("tickets")
        attempt = self.claim()
        victim = self.root / "host-overrides.jsonl"
        victim.write_text("{}\n")
        evidence = Path(attempt["path"]) / "docs/efforts/sample-effort/evidence"
        evidence.mkdir(parents=True)
        (evidence / "mainline-landing-overrides.jsonl").symlink_to(victim)
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        self.cli("land", attempt["id"], "--owner-mainline-override", "operator",
                 "--effort", "sample-effort", ok=False)
        self.assertEqual(victim.read_text(), "{}\n")
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])


if __name__ == "__main__":
    unittest.main()
