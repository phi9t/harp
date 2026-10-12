"""Refresh reservation atomicity proved by CLI process death in real repositories."""
from pathlib import Path
import unittest

import test_collab as fixtures


class AtomicRefreshTests(unittest.TestCase):
    setUp = fixtures.CollabTests.setUp
    run_process = fixtures.CollabTests.run_process
    git = fixtures.CollabTests.git
    commit = fixtures.CollabTests.commit
    cli = fixtures.CollabTests.cli
    init = fixtures.CollabTests.init
    ticket = fixtures.CollabTests.ticket
    effort_ticket = fixtures.CollabTests.effort_ticket
    contribute = fixtures.CollabTests.contribute

    def assert_uncommitted_replacement_crash_preserves_old_attempt(self, corenius):
        self.init()
        ticket = self.effort_ticket() if corenius else self.ticket()
        self.commit("ticket")
        identity = "sample-effort:01" if corenius else "T-one"
        old = self.cli("open-attempt", identity, "--owner", "worker")
        claimed = ticket.read_bytes()
        contribution = self.contribute(old)
        (self.repo / "trunk.txt").write_text("trunk movement\n")
        trunk = self.commit("direct trunk movement")
        stopped = self.cli("refresh", old["id"], "--confirm-stopped",
                           failpoint="after_refresh_abandon", ok=False)
        self.assertEqual(stopped.returncode, 86)
        self.assertEqual(self.cli("show", old["id"])["state"], "working")
        attempts = self.cli("status")["attempts"]
        self.assertEqual([row["id"] for row in attempts], [old["id"]])
        self.assertEqual(ticket.read_bytes(), claimed)
        self.assertEqual(self.git("rev-parse", old["branch"]), contribution)
        self.assertEqual(self.git("rev-parse", "HEAD", cwd=old["path"]), contribution)
        recovered = self.cli("recover", old["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "working")
        fresh = self.cli("refresh", old["id"], "--confirm-stopped")
        self.assertEqual(fresh["base"], trunk)
        self.assertEqual(fresh["state"], "working")
        self.assertEqual(self.cli("show", old["id"])["state"], "abandoned")
        self.assertEqual(ticket.read_bytes(), claimed)
        self.assertEqual((Path(fresh["path"]) / "code.txt").read_text(), "contribution\n")
        self.assertEqual((Path(fresh["path"]) / "trunk.txt").read_text(), "trunk movement\n")
        self.assertEqual(self.git("rev-parse", old["branch"]), contribution)
        self.assertEqual(sum(row["state"] not in ("landed", "abandoned")
                             for row in self.cli("status")["attempts"]), 1)

    def test_corenius_crash_between_abandonment_and_reservation_preserves_recoverable_claim(self):
        self.assert_uncommitted_replacement_crash_preserves_old_attempt(corenius=True)

    def test_standalone_crash_between_abandonment_and_reservation_preserves_active_attempt(self):
        self.assert_uncommitted_replacement_crash_preserves_old_attempt(corenius=False)

    def test_committed_corenius_replacement_is_reserved_and_recoverable_after_process_death(self):
        self.init()
        ticket = self.effort_ticket()
        self.commit("ticket")
        old = self.cli("open-attempt", "sample-effort:01", "--owner", "worker")
        claimed = ticket.read_bytes()
        contribution = self.contribute(old)
        (self.repo / "trunk.txt").write_text("direct trunk movement\n")
        self.commit("direct trunk movement")
        stopped = self.cli("refresh", old["id"], "--confirm-stopped",
                           failpoint="after_claim_record", ok=False)
        self.assertEqual(stopped.returncode, 86)
        self.assertEqual(self.cli("show", old["id"])["state"], "abandoned")
        active = [row for row in self.cli("status")["attempts"] if row["state"] != "abandoned"]
        self.assertEqual(len(active), 1)
        fresh = active[0]
        self.assertEqual(fresh["state"], "allocating")
        journal = next(event["details"] for event in self.cli("show", fresh["id"])["events"]
                       if event["kind"] == "refresh_started")
        self.assertEqual(journal["previous"], old["id"])
        self.assertEqual(journal["commits"], [contribution])
        recovered = self.cli("recover", fresh["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "working")
        self.assertEqual(ticket.read_bytes(), claimed)
        self.assertEqual(self.git("rev-parse", old["branch"]), contribution)
        self.assertEqual((Path(old["path"]) / "code.txt").read_text(), "contribution\n")


if __name__ == "__main__":
    unittest.main()
