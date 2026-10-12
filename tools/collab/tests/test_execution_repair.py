"""Execution fences, timeout quiescence, and tracker claim release through the public CLI."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import unittest

import test_collab as fixtures


class ExecutionRepairTests(unittest.TestCase):
    setUp = fixtures.CollabTests.setUp
    run_process = fixtures.CollabTests.run_process
    git = fixtures.CollabTests.git
    commit = fixtures.CollabTests.commit
    cli = fixtures.CollabTests.cli
    init = fixtures.CollabTests.init
    ticket = fixtures.CollabTests.ticket
    claim = fixtures.CollabTests.claim
    contribute = fixtures.CollabTests.contribute
    prepared = fixtures.CollabTests.prepared
    effort_ticket = fixtures.CollabTests.effort_ticket
    open_corenius_attempt = fixtures.CollabTests.open_corenius_attempt

    def wait_file(self, file):
        deadline = time.monotonic() + 5
        while not file.exists() and time.monotonic() < deadline:
            time.sleep(0.02)
        self.assertTrue(file.exists(), str(file))

    def test_abandon_and_gc_cannot_change_a_running_check(self):
        started = self.root / "started"
        release = self.root / "release"
        code = ("import time; from pathlib import Path; "
                f"Path({str(started)!r}).write_text('running'); "
                f"exec(\"while not Path({str(release)!r}).exists(): time.sleep(.02)\")")
        attempt = self.prepared([sys.executable, "-c", code])
        process = subprocess.Popen([sys.executable, str(fixtures.CLI), "--repo", str(self.repo),
                                    "check", attempt["id"]], env=self.env, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, text=True)
        def stop_check():
            release.touch()
            process.communicate(timeout=5)

        self.addCleanup(stop_check)
        self.wait_file(started)
        checking = self.cli("show", attempt["id"])
        before = self.git("worktree", "list", "--porcelain")
        refused = self.cli("abandon", attempt["id"], "--confirm-stopped", ok=False)
        self.assertIn("busy", refused.stderr)
        refused = self.cli("gc", attempt["id"], "--confirm-stopped", ok=False)
        self.assertIn("busy", refused.stderr)
        self.assertEqual(self.cli("show", attempt["id"])["state"], "checking")
        self.assertEqual(self.git("worktree", "list", "--porcelain"), before)
        self.assertTrue(Path(checking["check_path"]).exists())
        release.touch()
        out, err = process.communicate(timeout=5)
        self.assertEqual(process.returncode, 0, err)
        self.assertEqual(json.loads(out)["state"], "verified")
        self.assertEqual(self.cli("abandon", attempt["id"], "--confirm-stopped")["state"], "abandoned")

    def test_timeout_kills_term_ignoring_descendant_after_leader_exits(self):
        child_pid = self.root / "child-pid"
        child_code = ("import os, signal, time; from pathlib import Path; "
                      "signal.signal(signal.SIGTERM, signal.SIG_IGN); "
                      f"Path({str(child_pid)!r}).write_text(str(os.getpid())); time.sleep(120)")
        leader_code = ("import subprocess, sys, time; from pathlib import Path; "
                       f"subprocess.Popen([sys.executable, '-c', {child_code!r}]); "
                       f"exec(\"while not Path({str(child_pid)!r}).exists(): time.sleep(.01)\"); time.sleep(120)")
        attempt = self.prepared([sys.executable, "-c", leader_code], timeout_seconds=0.5)
        def kill_child():
            if child_pid.exists():
                try:
                    os.kill(int(child_pid.read_text()), signal.SIGKILL)
                except ProcessLookupError:
                    pass

        self.addCleanup(kill_child)
        self.cli("check", attempt["id"], ok=False)
        self.wait_file(child_pid)
        pid = int(child_pid.read_text())
        proc = Path("/proc") / str(pid) / "stat"
        live = proc.exists() and proc.read_text().rsplit(")", 1)[1].strip().split()[0] != "Z"
        self.assertFalse(live, "descendant still runs after timed-out check returns")
        checked = self.cli("show", attempt["id"])
        self.assertEqual(checked["check_receipt"]["outcome"], "timeout")
        self.assertEqual(self.cli("abandon", attempt["id"], "--confirm-stopped")["state"], "abandoned")

    def corenius_attempt(self):
        self.init()
        ticket = self.effort_ticket()
        self.commit("tracker ticket")
        attempt = self.open_corenius_attempt(owner="worker")
        return attempt, ticket

    def test_corenius_abandon_commits_owned_claim_release_before_fresh_admission(self):
        attempt, ticket = self.corenius_attempt()
        abandoned = self.cli("abandon", attempt["id"], "--confirm-stopped")
        self.assertEqual(abandoned["state"], "abandoned")
        self.assertIn("Status: open", ticket.read_text())
        self.assertIn("Owner:\n", ticket.read_text())
        self.assertIn("Claimed:\n", ticket.read_text())
        self.assertIn(attempt["id"], ticket.read_text())
        self.assertEqual(self.git("status", "--porcelain"), "")
        self.assertIn("Status: open", self.git("show", "HEAD:" + str(ticket.relative_to(self.repo))))
        fresh = self.open_corenius_attempt(owner="new-worker")
        self.assertNotEqual(fresh["id"], attempt["id"])
        self.assertTrue(Path(attempt["path"]).exists())

    def test_abandon_refuses_a_changed_claim_without_releasing_another_owner(self):
        attempt, ticket = self.corenius_attempt()
        ticket.write_text(ticket.read_text().replace("Owner: worker", "Owner: other-worker"))
        self.commit("claim transferred externally")
        before = ticket.read_bytes()
        result = self.cli("abandon", attempt["id"], "--confirm-stopped", ok=False)
        self.assertIn("claim", result.stderr.lower())
        self.assertEqual(ticket.read_bytes(), before)
        self.assertEqual(self.cli("show", attempt["id"])["state"], "working")

    def exercise_release_crash(self, failpoint):
        attempt, ticket = self.corenius_attempt()
        crashed = self.cli("abandon", attempt["id"], "--confirm-stopped", failpoint=failpoint, ok=False)
        self.assertEqual(crashed.returncode, 86)
        self.assertEqual(self.cli("show", attempt["id"])["state"], "abandoning")
        self.open_corenius_attempt(owner="new-worker", ok=False)
        self.cli("gc", attempt["id"], "--confirm-stopped", ok=False)
        self.cli("refresh", attempt["id"], "--confirm-stopped", ok=False)
        swept = self.cli("sweep", "--apply", "--archive-dirty", "--retention-days", "0")
        self.assertTrue(Path(attempt["path"]).exists())
        self.assertIn("abandoning", str(swept["skipped"]))
        recovered = self.cli("recover", attempt["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "abandoned")
        self.assertEqual(ticket.read_text().count("released claim for abandoned Attempt " + attempt["id"]), 1)
        self.assertEqual(self.git("status", "--porcelain"), "")
        fresh = self.open_corenius_attempt(owner="new-worker")
        self.assertEqual(fresh["owner"], "new-worker")
        self.assertNotEqual(fresh["ticket_hash"], attempt["ticket_hash"])
        self.assertEqual(self.cli("show", attempt["id"])["ticket_hash"], attempt["ticket_hash"])

    def test_release_intent_crash_recovers_before_fresh_admission(self):
        self.exercise_release_crash("after_claim_release_intent")

    def test_uncommitted_tracker_release_crash_recovers_only_expected_ticket_edit(self):
        self.exercise_release_crash("after_tracker_release")

    def test_committed_tracker_release_crash_recovers_before_fresh_admission(self):
        self.exercise_release_crash("after_claim_release_commit")

    def test_release_recovery_refuses_newer_claim_and_preserves_original_binding(self):
        attempt, ticket = self.corenius_attempt()
        self.cli("abandon", attempt["id"], "--confirm-stopped",
                 failpoint="after_claim_release_commit", ok=False)
        result = self.run_process([sys.executable, str(fixtures.CLI.parents[1] / "agents" / "tracker.py"),
                                   "--repo", str(self.repo), "claim", "sample-effort", "01",
                                   "--owner", "other-worker"])
        self.assertEqual(result.returncode, 0, result.stderr)
        self.commit("newer independently owned claim")
        before = ticket.read_bytes()
        result = self.cli("recover", attempt["id"], "--confirm-stopped", ok=False)
        self.assertIn("claim changed", result.stderr.lower())
        self.assertEqual(ticket.read_bytes(), before)
        observed = self.cli("show", attempt["id"])
        self.assertEqual(observed["state"], "abandoning")
        self.assertEqual(observed["ticket_hash"], attempt["ticket_hash"])
        self.assertEqual(observed["owner"], "worker")

    def test_old_refreshed_attempt_cannot_release_transferred_claim(self):
        old, ticket = self.corenius_attempt()
        self.contribute(old)
        (self.repo / "unrelated").write_text("trunk movement\n")
        self.commit("direct trunk movement")
        fresh = self.cli("refresh", old["id"], "--confirm-stopped")
        before = ticket.read_bytes()
        self.cli("abandon", old["id"], "--confirm-stopped", ok=False)
        self.assertEqual(ticket.read_bytes(), before)
        self.assertEqual(self.cli("show", fresh["id"])["state"], "working")
        self.assertEqual(fresh["ticket_hash"], old["ticket_hash"])
        self.assertIn("Status: claimed", ticket.read_text())

    def test_successful_leader_with_live_descendant_cannot_publish_passed_receipt(self):
        child_pid = self.root / "surviving-child"
        child_code = ("import os, signal, time; from pathlib import Path; "
                      "signal.signal(signal.SIGTERM, signal.SIG_IGN); "
                      f"Path({str(child_pid)!r}).write_text(str(os.getpid())); time.sleep(120)")
        code = ("import subprocess, sys, time; from pathlib import Path; "
                f"subprocess.Popen([sys.executable, '-c', {child_code!r}]); "
                f"exec(\"while not Path({str(child_pid)!r}).exists(): time.sleep(.01)\")")
        attempt = self.prepared([sys.executable, "-c", code])
        def kill_child():
            if child_pid.exists():
                try:
                    os.kill(int(child_pid.read_text()), signal.SIGKILL)
                except ProcessLookupError:
                    pass

        self.addCleanup(kill_child)
        result = self.cli("check", attempt["id"], ok=False)
        self.assertIn("error", result.stderr)
        proc = Path("/proc") / child_pid.read_text() / "stat"
        self.assertFalse(proc.exists() and proc.read_text().rsplit(")", 1)[1].strip().split()[0] != "Z")
        receipt = self.cli("show", attempt["id"])["check_receipt"]
        self.assertEqual(receipt["exit_code"], 0)
        self.assertFalse(receipt["passed"])
        self.cli("land", attempt["id"], "--no-push", ok=False)


if __name__ == "__main__":
    unittest.main()
