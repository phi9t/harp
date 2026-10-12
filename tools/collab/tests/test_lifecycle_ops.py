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
    cli = fixtures.CollabTests.cli
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


if __name__ == "__main__":
    unittest.main()
