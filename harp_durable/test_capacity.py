"""Public deterministic capacity boundaries; live provider qualification is separate."""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import unittest

from . import test_admission as admission_tests
from . import test_recovery as recovery_tests

ROOT = admission_tests.ROOT


@unittest.skipUnless(sys.platform == "linux" and os.environ.get("HARP_TEST_CGROUP_ROOT")
                     and os.environ.get("HARP_TEST_ROOTFS") and os.environ.get("HARP_DURABLE_BIN"),
                     "requires public Harp binary, admitted rootfs and explicit Linux cgroup delegation")
class CapacityLifecycle(unittest.TestCase):
    # Reuse fixtures and observation helpers, never inherited test cases.
    setUp = admission_tests.CodingLifecycle.setUp
    git = admission_tests.CodingLifecycle.git
    call = admission_tests.CodingLifecycle.call
    admit = admission_tests.CodingLifecycle.admit
    wait_for = recovery_tests.RecoveryLifecycle.wait_for
    group = recovery_tests.RecoveryLifecycle.group
    guardian = recovery_tests.RecoveryLifecycle.guardian
    assert_empty = recovery_tests.RecoveryLifecycle.assert_empty

    def evidence(self, name, value):
        directory = os.environ.get("HARP_CAPACITY_EVIDENCE_DIR")
        if directory:
            path = Path(directory)
            path.mkdir(mode=0o700, parents=True, exist_ok=True)
            (path / (name + ".json")).write_text(json.dumps(value, sort_keys=True, indent=2) + "\n")

    def launch(self, capacity):
        process = subprocess.Popen([*self.command, "start", "--capacity", str(capacity),
                                    "--until-idle", "--timeout", "90"], cwd=ROOT, env=self.env,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        def cleanup():
            if process.poll() is None:
                process.terminate()
            try:
                process.communicate(timeout=15)
            except subprocess.TimeoutExpired:
                process.kill()
                process.communicate(timeout=5)
                self.fail("owned capacity fixture runtime did not cleanly stop")
        self.addCleanup(cleanup)
        return process

    def populated(self, attempt):
        return dict(line.split() for line in
                    (self.group(attempt).path / "cgroup.events").read_text().splitlines())["populated"]

    def live_attempt(self, run_id):
        def observed():
            status = self.call("status", run_id)
            if not status["attempts"]:
                return None
            attempt = status["attempts"][-1]
            process = Path(attempt["directory"]) / "process.json"
            if process.is_file() and self.populated(attempt) == "1":
                pid = json.loads(process.read_text())["pid"]
                try:
                    membership = Path(f"/proc/{pid}/cgroup").read_text().strip()
                except FileNotFoundError:
                    return None
                self.assertEqual(membership, "0::" + str(self.group(attempt).path).removeprefix("/sys/fs/cgroup"))
                self.assertEqual(attempt["released"], 1)
                return attempt
        return self.wait_for(observed, f"{run_id} never acquired a released populated cgroup")

    def exercise_capacity(self, limit):
        run_ids = [f"worker-{number}" for number in range(limit)]
        for run_id in run_ids:
            self.admit(run_id, "--fixture-sleep", "20")
        self.admit("queued", "--fixture-sleep", "1")
        process = self.launch(limit)
        attempts = {run_id: self.live_attempt(run_id) for run_id in run_ids}
        self.assertEqual(len({attempt["id"] for attempt in attempts.values()}), limit)
        self.assertEqual(len({str(self.group(attempt).path) for attempt in attempts.values()}), limit)
        for attempt in attempts.values():
            self.assertEqual(self.populated(attempt), "1")
        self.assertEqual(self.call("status", "queued")["attempts"], [])
        self.evidence(f"limit-{limit}-overlap", {
            "limit": limit, "runs": {key: value["id"] for key, value in attempts.items()},
            "groups": {key: str(self.group(value).path) for key, value in attempts.items()},
            "populated": {key: self.populated(value) for key, value in attempts.items()},
            "queued_attempts": []})
        cancelled = run_ids[0]
        self.assertTrue(self.call("cancel", cancelled)["cancel_requested"])
        queued = self.live_attempt("queued")
        old = attempts[cancelled]
        self.assert_empty(old)
        receipt = json.loads((Path(old["directory"]) / "receipt.json").read_text())
        self.assertTrue(receipt["cleanup"])
        self.assertEqual(receipt["cleanup_proof"]["backend"], "cgroup-v2")
        self.assertEqual(receipt["cleanup_proof"]["populated"], 0)
        self.assertLessEqual(receipt["cleanup_proof"]["observed_at"], queued["created"])
        for run_id in run_ids[1:]:
            self.assertEqual([value["id"] for value in self.call("status", run_id)["attempts"]],
                             [attempts[run_id]["id"]])
        self.evidence(f"limit-{limit}-reuse", {
            "cancelled_attempt": old["id"], "cleanup_proof": receipt["cleanup_proof"],
            "replacement_attempt": queued["id"], "replacement_created": queued["created"],
            "other_attempts": {key: attempts[key]["id"] for key in run_ids[1:]}})
        _out, err = process.communicate(timeout=60)
        self.assertEqual(process.returncode, 0, err)
        self.assertEqual(self.call("status", "queued")["status"], "SUCCEEDED")
        for run_id in run_ids[1:]:
            status = self.call("status", run_id)
            self.assertEqual(status["status"], "SUCCEEDED", status)
            self.assertEqual([value["id"] for value in status["attempts"]], [attempts[run_id]["id"]])
        for attempt in [*attempts.values(), queued]:
            self.assert_empty(attempt)
        self.assertEqual(self.call("status", cancelled)["status"], "BLOCKED")

    def test_one_worker_capacity_reuses_only_cleaned_slot(self):
        self.exercise_capacity(1)

    def test_two_worker_capacity_reuses_only_cleaned_slot(self):
        self.exercise_capacity(2)

    def test_four_worker_capacity_reuses_only_cleaned_slot(self):
        self.exercise_capacity(4)

    def test_unknown_cleanup_blocks_dispatch_even_after_other_worker_completed(self):
        self.admit("lost", "--fixture-sleep", "20")
        self.admit("completed", "--fixture-sleep", "6")
        process = self.launch(2)
        lost = self.live_attempt("lost")
        completed = self.live_attempt("completed")
        guardian = self.guardian(lost)
        group = self.group(lost)
        signal.pidfd_send_signal(guardian, signal.SIGSTOP)
        killed = False
        try:
            self.wait_for(lambda: self.call("status", "completed")["status"] == "SUCCEEDED",
                          "other worker never completed while the guardian was stopped")
            self.assert_empty(completed)
            self.assertEqual(self.populated(lost), "1")
            process.kill()
            process.communicate(timeout=10)
            self.admit("queued")
            error = self.call("start", "--capacity", "2", "--until-idle", "--timeout", "10", ok=False)
            self.assertIn("lock", error["message"])
            self.assertEqual(self.call("status", "queued")["attempts"], [])
            signal.pidfd_send_signal(guardian, signal.SIGKILL)
            killed = True
            (Path(lost["directory"]) / "membership.json").unlink()
            self.wait_for(lambda: not self.call("status")["locked"], "lost guardian retained controller ownership")
            error = self.call("start", "--capacity", "2", "--until-idle", "--timeout", "10", ok=False)
            self.assertIn("cleanup", error["message"].lower())
            self.assertEqual(self.call("status", "queued")["attempts"], [])
            self.assertEqual(self.call("status", "lost")["operations"][0]["status"], "UNCERTAIN")
            proof = group.kill_and_wait()
            self.assertEqual(proof["populated"], 0)
            self.assert_empty(lost)
            acknowledged = self.call("ack-cleanup", lost["id"], "--evidence", json.dumps(proof))
            self.assertFalse(acknowledged["work_launched"])
            resolved = self.call("resolve", "lost", "coding", "fail", "--evidence",
                                 "Exact attempt cgroup empty; retain interrupted workspace and reject its effect")
            self.assertFalse(resolved["work_launched"])
            self.assertEqual(self.call("status", "queued")["attempts"], [])
            self.call("start", "--capacity", "2", "--until-idle", "--timeout", "30")
            self.assertEqual(self.call("status", "lost")["status"], "FAILED")
            self.assertEqual(self.call("status", "queued")["status"], "SUCCEEDED")
            after = self.call("status", "completed")
            self.assertEqual(after["status"], "SUCCEEDED")
            self.assertEqual([value["id"] for value in after["attempts"]], [completed["id"]])
            self.evidence("limit-2-uncertain-reconciliation", {
                "lost_attempt": lost["id"], "completed_attempt": completed["id"],
                "cleanup_proof": proof, "ack_work_launched": acknowledged["work_launched"],
                "resolve_work_launched": resolved["work_launched"],
                "queued_status": self.call("status", "queued")["status"]})
        finally:
            if not killed:
                signal.pidfd_send_signal(guardian, signal.SIGCONT)
            group.kill_and_wait()


if __name__ == "__main__":
    unittest.main()
