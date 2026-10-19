"""Public-CLI crash/reconciliation cases with real delegated kernel ownership.

These cases deliberately destroy owned fixture processes or their recovery
metadata. They require a disposable delegated scope and run no model providers.
"""
import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import time
import unittest

import durable.agents.test_admission as admission_tests

ROOT = admission_tests.ROOT


@unittest.skipUnless(sys.platform == "linux" and os.environ.get("HARP_TEST_CGROUP_ROOT")
                     and os.environ.get("HARP_TEST_ROOTFS") and os.environ.get("HARP_DURABLE_BIN"),
                     "requires public Harp binary, admitted rootfs and explicit Linux cgroup delegation")
class RecoveryLifecycle(unittest.TestCase):
    # Reuse the fixture, not its already-covered admission test cases.
    setUp = admission_tests.CodingLifecycle.setUp
    git = admission_tests.CodingLifecycle.git
    call = admission_tests.CodingLifecycle.call
    admit = admission_tests.CodingLifecycle.admit

    def wait_for(self, operation, reason):
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            result = operation()
            if result:
                return result
            time.sleep(0.05)
        self.fail(reason)

    def start_process(self, *extra):
        process = subprocess.Popen([*self.command, "start", "--until-idle", "--timeout", "30", *extra],
                                   cwd=ROOT, env=self.env, stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE, text=True)
        def cleanup():
            if process.poll() is None:
                process.terminate()
            try:
                process.communicate(timeout=10)
            except subprocess.TimeoutExpired:
                process.kill()
                process.communicate(timeout=5)
                self.fail("owned fixture runtime did not stop after TERM")
        self.addCleanup(cleanup)
        return process

    def started_attempt(self, run_id):
        def started():
            status = self.call("status", run_id)
            if status["attempts"] and (Path(status["attempts"][-1]["directory"]) / "process.json").is_file():
                return status["attempts"][-1]
        return self.wait_for(started, "owned payload never reached the released-process boundary")

    def crash(self, point):
        output = subprocess.run([*self.command, "start", "--until-idle", "--timeout", "30",
                                 "--test-crash-at", point], cwd=ROOT, env=self.env,
                                text=True, capture_output=True, timeout=60)
        self.assertEqual(output.returncode, 86, {"stdout": output.stdout, "stderr": output.stderr})
        self.assertEqual(json.loads((self.state / "test-crash.json").read_text())["point"], point)
        self.wait_for(lambda: not self.call("status")["locked"],
                      "guardian did not finish cleanup and release ownership after controller death")

    def group(self, attempt):
        from insula.cgroup import Cgroup
        identity = json.loads((Path(attempt["directory"]) / "cgroup.json").read_text())
        binding = json.loads(self.execution.read_text())["cgroup_identity"]
        return Cgroup.resume(identity, attempt_id=attempt["id"], root_identity=binding)

    def assert_empty(self, attempt):
        events = dict(line.split() for line in (self.group(attempt).path / "cgroup.events").read_text().splitlines())
        self.assertEqual(events["populated"], "0", "kernel still observes an owned descendant")

    def guardian(self, attempt):
        expected = os.fsencode(str(Path(attempt["directory"]) / "request.json"))
        for entry in Path("/proc").glob("[0-9]*/cmdline"):
            try:
                argv = entry.read_bytes().split(b"\0")
                if b"durable.runtime.guard" in argv and expected in argv:
                    descriptor = os.pidfd_open(int(entry.parent.name))
                    # Recheck identity after opening the stable kernel handle.
                    if entry.read_bytes().split(b"\0") == argv:
                        self.addCleanup(os.close, descriptor)
                        return descriptor
                    os.close(descriptor)
            except (OSError, ProcessLookupError):
                continue
        self.fail("cannot find this fixture attempt's guardian")

    def test_before_release_crashes_retry_only_after_old_ownership_ends(self):
        for point in ("after_claim", "before_release"):
            with self.subTest(point=point):
                self.admit(point)
                self.crash(point)
                before = self.call("status", point)
                self.assertEqual(len(before["attempts"]), 1)
                first = before["attempts"][0]
                self.assertFalse((Path(first["directory"]) / "process.json").exists())
                if point == "before_release":
                    receipt = json.loads((Path(first["directory"]) / "receipt.json").read_text())
                    self.assertFalse(receipt["started"])
                    self.assertTrue(receipt["cleanup"])
                    self.assert_empty(first)
                self.call("start", "--until-idle", "--timeout", "30")
                after = self.call("status", point)
                self.assertEqual(after["status"], "SUCCEEDED", after)
                self.assertEqual(len(after["attempts"]), 2)
                self.assertEqual(after["attempts"][0]["status"], "FAILED")
                self.assertNotEqual(after["attempts"][0]["directory"], after["attempts"][1]["directory"])
                self.assert_empty(after["attempts"][1])

    def test_completed_receipt_is_recovered_without_another_attempt(self):
        for point in ("after_receipt", "after_commit"):
            with self.subTest(point=point):
                self.admit(point)
                self.crash(point)
                before = self.call("status", point)
                self.assertEqual(len(before["attempts"]), 1)
                attempt = before["attempts"][0]
                receipt_path = Path(attempt["directory"]) / "receipt.json"
                receipt_bytes = receipt_path.read_bytes()
                receipt = json.loads(receipt_bytes)
                self.assertEqual(receipt["status"], "SUCCEEDED")
                self.assertTrue(receipt["cleanup"])
                bundle = Path(attempt["directory"]) / "payload/candidate.bundle"
                bundle_bytes = bundle.read_bytes()
                self.assert_empty(attempt)
                self.call("start", "--until-idle", "--timeout", "30")
                self.call("start", "--until-idle", "--timeout", "30")
                after = self.call("status", point)
                self.assertEqual(after["status"], "SUCCEEDED", after)
                self.assertEqual(after["result"], receipt["result"])
                self.assertEqual(len(after["attempts"]), 1)
                self.assertEqual(receipt_path.read_bytes(), receipt_bytes)
                self.assertEqual(bundle.read_bytes(), bundle_bytes)

    def test_started_owner_death_stays_uncertain_until_explicit_retry(self):
        self.admit("interrupted", "--fixture-sleep", "2")
        self.crash("after_release_started")
        self.call("start", "--until-idle", "--timeout", "30")
        blocked = self.call("status", "interrupted")
        self.assertEqual(blocked["status"], "BLOCKED", blocked)
        self.assertEqual(blocked["operations"][0]["status"], "UNCERTAIN")
        self.assertEqual(len(blocked["attempts"]), 1)
        self.assert_empty(blocked["attempts"][0])
        self.call("start", "--until-idle", "--timeout", "30")
        self.assertEqual(len(self.call("status", "interrupted")["attempts"]), 1)
        decision = self.call("resolve", "interrupted", "coding", "retry", "--evidence",
                             "Kernel subtree empty; isolated interrupted workspace retained; authorize a new private attempt")
        self.assertFalse(decision["work_launched"])
        self.assertEqual(len(self.call("status", "interrupted")["attempts"]), 1)
        self.call("start", "--until-idle", "--timeout", "30")
        after = self.call("status", "interrupted")
        self.assertEqual(after["status"], "SUCCEEDED", after)
        self.assertEqual(len(after["attempts"]), 2)
        self.assertNotEqual(after["attempts"][0]["id"], after["attempts"][1]["id"])
        self.assert_empty(after["attempts"][0])
        self.assert_empty(after["attempts"][1])

    def test_cancel_keeps_capacity_until_guardian_proves_cleanup(self):
        self.admit("cancelled", "--fixture-sleep", "20")
        self.admit("queued")
        process = self.start_process()
        attempt = self.started_attempt("cancelled")
        guardian = self.guardian(attempt)
        signal.pidfd_send_signal(guardian, signal.SIGSTOP)
        try:
            self.assertTrue(self.call("cancel", "cancelled")["cancel_requested"])
            self.assertEqual(self.call("status", "queued")["attempts"], [])
            events = dict(line.split() for line in (self.group(attempt).path / "cgroup.events").read_text().splitlines())
            self.assertEqual(events["populated"], "1", "fixture must still own live processes while guardian is stopped")
        finally:
            signal.pidfd_send_signal(guardian, signal.SIGCONT)
        _output, error = process.communicate(timeout=40)
        self.assertEqual(process.returncode, 0, error)
        status = self.call("status", "cancelled")
        self.assertEqual(status["status"], "BLOCKED", status)
        self.assertEqual(status["operations"][0]["status"], "UNCERTAIN")
        self.assertTrue(status["cancel_requested"])
        self.assert_empty(attempt)
        self.assertEqual(self.call("status", "queued")["status"], "SUCCEEDED")
        receipt = json.loads((Path(attempt["directory"]) / "receipt.json").read_text())
        self.assertEqual(receipt["cleanup_proof"]["populated"], 0)
        self.assertFalse(self.call("resolve", "cancelled", "coding", "retry", "--evidence",
                                   "Cleaned private fixture workspace inspected; cancellation remains requested")["work_launched"])
        self.call("start", "--until-idle", "--timeout", "30")
        cancelled = self.call("status", "cancelled")
        self.assertEqual(cancelled["status"], "CANCELLED")
        self.assertEqual(len(cancelled["attempts"]), 1, "effect resolution must not revoke cancellation")

    def test_stopped_guardian_and_lost_cleanup_evidence_block_dispatch(self):
        self.admit("lost", "--fixture-sleep", "20")
        self.admit("queued")
        process = self.start_process()
        attempt = self.started_attempt("lost")
        guardian = self.guardian(attempt)
        group = self.group(attempt)
        signal.pidfd_send_signal(guardian, signal.SIGSTOP)
        killed = False
        try:
            process.kill()
            process.communicate(timeout=10)
            error = self.call("start", "--until-idle", "--timeout", "10", ok=False)
            self.assertIn("lock", error["message"])
            self.assertEqual(self.call("status", "queued")["attempts"], [])
            # Lose the guardian and its membership confirmation: the runtime
            # cannot infer cleanup from a missing process or an expired owner.
            signal.pidfd_send_signal(guardian, signal.SIGKILL)
            killed = True
            (Path(attempt["directory"]) / "membership.json").unlink()
            self.wait_for(lambda: not self.call("status")["locked"], "lost guardian retained ownership")
            error = self.call("start", "--until-idle", "--timeout", "10", ok=False)
            self.assertIn("cleanup", error["message"].lower())
            status = self.call("status", "lost")
            self.assertEqual(status["status"], "BLOCKED")
            self.assertEqual(status["operations"][0]["status"], "UNCERTAIN")
            self.assertEqual(self.call("status", "queued")["attempts"], [])
            proof = group.kill_and_wait()
            self.assertEqual(proof["populated"], 0)
            self.assert_empty(attempt)
            acknowledged = self.call("ack-cleanup", attempt["id"], "--evidence", json.dumps(proof))
            self.assertFalse(acknowledged["work_launched"])
            self.assertEqual(self.call("status", "lost")["status"], "BLOCKED")
            self.assertEqual(self.call("status", "queued")["attempts"], [])
            resolved = self.call("resolve", "lost", "coding", "fail", "--evidence",
                                 "Kernel cleanup independently observed; interrupted workspace retained without accepted proposal")
            self.assertFalse(resolved["work_launched"])
            self.assertEqual(self.call("status", "queued")["attempts"], [])
            self.call("start", "--until-idle", "--timeout", "30")
            self.assertEqual(self.call("status", "lost")["status"], "FAILED")
            self.assertEqual(self.call("status", "queued")["status"], "SUCCEEDED")
        finally:
            if not killed:
                signal.pidfd_send_signal(guardian, signal.SIGCONT)
            # Native ownership is always cleaned even if an assertion failed.
            group.kill_and_wait()


if __name__ == "__main__":
    unittest.main()
