"""Mixed adapter slot identity and restart faults through the public CLI.

Controlled executables exercise actual Codex/Trae adapters, private Git workspaces,
SQLite, namespaces and kernel cleanup. These are not paid-provider qualifications.
"""
import json
import os
from pathlib import Path
import signal
import sys
import types
import unittest

from . import test_admission as admission_tests
from .test_provider_recovery import provider_fixture


def releasable_fixture(provider):
    # The only controls are ordinary files in this attempt's private workspace.
    return provider_fixture(provider).replace(
        b"if child:\n    time.sleep(120)",
        b"if child:\n    while not (workspace / '.fixture-release').exists():\n"
        b"        time.sleep(0.05)\n    child.terminate()\n    child.wait()")


@unittest.skipUnless(sys.platform == "linux" and all(os.environ.get(name) for name in
                     ("HARP_TEST_CGROUP_ROOT", "HARP_TEST_ROOTFS", "HARP_DURABLE_BIN")),
                     "requires public Harp binary, native rootfs and disposable delegated Linux cgroup")
class MixedRecovery(unittest.TestCase):
    git = admission_tests.GitInputs.git
    call = admission_tests.CodingLifecycle.call

    def setUp(self):
        admission_tests.CodingLifecycle.setUp(self)
        from .test_recovery import RecoveryLifecycle
        from .test_execution_policy import metadata_fixture
        from .test_provider_recovery import ProviderRecovery
        for name in ("wait_for", "start_process", "started_attempt", "group", "assert_empty", "guardian"):
            setattr(self, name, types.MethodType(getattr(RecoveryLifecycle, name), self))
        for name in ("admit_provider", "launched"):
            setattr(self, name, types.MethodType(getattr(ProviderRecovery, name), self))
        old_state, old_execution = self.state, self.execution
        self.state, self.execution = self.root / "mixed-state", self.root / "mixed-execution.json"
        self.command = [str(self.state) if item == str(old_state) else
                        str(self.execution) if item == str(old_execution) else item for item in self.command]
        providers, auth = self.root / "providers", self.root / "credentials"
        providers.mkdir(mode=0o700)
        auth.mkdir(mode=0o700)
        for name in ("codex", "trae", "codex-code-mode-host"):
            path = providers / name
            path.write_bytes(metadata_fixture(name) if name == "codex-code-mode-host" else releasable_fixture(name))
            path.chmod(0o755)
        for name in ("codex", "trae"):
            path = auth / name
            path.write_text(json.dumps({"fixture": name}))
            path.chmod(0o600)
        self.call("prepare", "--rootfs", os.environ["HARP_TEST_ROOTFS"],
                  "--cgroup-root", os.environ["HARP_TEST_CGROUP_ROOT"], "--network", "host",
                  "--providers-bin", str(providers), "--codex-auth", str(auth / "codex"),
                  "--trae-auth", str(auth / "trae"))

    def populated(self, attempt):
        return dict(line.split() for line in (self.group(attempt).path / "cgroup.events").read_text().splitlines())["populated"]

    def release(self, attempt):
        (Path(attempt["directory"]) / "payload/workspace/.fixture-release").write_text("operator fixture release\n")

    def succeeded(self, run_id):
        status = self.call("status", run_id)
        return status if status["status"] == "SUCCEEDED" else None

    def test_mixed_slots_reuse_only_the_cancelled_workers_cleaned_slot(self):
        for capacity in (2, 4):
            with self.subTest(capacity=capacity):
                active = [(f"cap{capacity}-{index}", ("codex", "trae")[index % 2])
                          for index in range(capacity)]
                for run_id, provider in active:
                    self.admit_provider(run_id, provider, hold=True)
                queued = f"cap{capacity}-queued"
                self.admit_provider(queued, "trae")
                process = self.start_process("--capacity", str(capacity), "--timeout", "90")
                attempts = [self.launched(run_id, provider) for run_id, provider in active]
                self.assertEqual(len({attempt["id"] for attempt in attempts}), capacity)
                self.assertEqual(len({str(self.group(attempt).path) for attempt in attempts}), capacity)
                self.assertTrue(all(self.populated(attempt) == "1" for attempt in attempts))
                self.assertEqual(self.call("status", queued)["attempts"], [])
                guardian = self.guardian(attempts[0])
                signal.pidfd_send_signal(guardian, signal.SIGSTOP)
                try:
                    self.call("cancel", active[0][0])
                    self.assertEqual(self.populated(attempts[0]), "1")
                    self.assertEqual(self.call("status", queued)["attempts"], [])
                    for (run_id, _), attempt in zip(active[1:], attempts[1:]):
                        self.assertEqual(self.populated(attempt), "1")
                        self.assertEqual([item["id"] for item in self.call("status", run_id)["attempts"]], [attempt["id"]])
                finally:
                    signal.pidfd_send_signal(guardian, signal.SIGCONT)
                done = self.wait_for(lambda: self.succeeded(queued), "cleaned slot did not admit queued adapter")
                self.assert_empty(attempts[0])
                receipt = json.loads((Path(attempts[0]["directory"]) / "receipt.json").read_text())
                self.assertTrue(receipt["cleanup"])
                self.assertEqual(receipt["cleanup_proof"]["populated"], 0)
                launch = json.loads((Path(done["attempts"][0]["directory"]) /
                                     "payload/workspace/.provider-launch.json").read_text())
                self.assertGreaterEqual(launch["launched_at"], receipt["finished_at"])
                for (run_id, _), attempt in zip(active[1:], attempts[1:]):
                    self.assertEqual(self.populated(attempt), "1", "another worker's ownership was lost")
                    self.assertEqual([item["id"] for item in self.call("status", run_id)["attempts"]], [attempt["id"]])
                    self.release(attempt)
                _, error = process.communicate(timeout=40)
                self.assertEqual(process.returncode, 0, error)
                for (run_id, provider), attempt in zip(active[1:], attempts[1:]):
                    result = self.call("status", run_id)
                    self.assertEqual(result["status"], "SUCCEEDED", result)
                    self.assertEqual(result["result"]["provider_report"]["provider"], provider)
                    self.assertEqual(result["result"]["attempt_id"], attempt["id"])
                    self.assertEqual(len(result["attempts"]), 1)
                    self.assert_empty(attempt)
                self.assertEqual(self.call("status", queued)["result"], done["result"])

    def test_restart_preserves_completed_peer_and_fences_only_ambiguous_adapter(self):
        self.admit_provider("completed-codex", "codex", hold=True)
        self.admit_provider("interrupted-trae", "trae", hold=True)
        process = self.start_process("--capacity", "2")
        completed = self.launched("completed-codex", "codex")
        interrupted = self.launched("interrupted-trae", "trae")
        self.assertEqual(self.populated(completed), "1")
        self.assertEqual(self.populated(interrupted), "1")
        self.release(completed)
        accepted = self.wait_for(lambda: self.succeeded("completed-codex"), "completed peer was not accepted")
        receipt_path = Path(completed["directory"]) / "receipt.json"
        receipt = receipt_path.read_bytes()
        bundle_path = Path(completed["directory"]) / "payload/candidate.bundle"
        bundle = bundle_path.read_bytes()
        self.assertEqual(self.populated(interrupted), "1")
        process.kill()
        process.communicate(timeout=10)
        self.wait_for(lambda: not self.call("status")["locked"], "guardian retained ownership after runtime death")
        for _ in range(2):
            self.call("start", "--capacity", "2", "--until-idle", "--timeout", "30")
        blocked = self.call("status", "interrupted-trae")
        self.assertEqual(blocked["status"], "BLOCKED", blocked)
        self.assertEqual(blocked["operations"][0]["status"], "UNCERTAIN")
        self.assertEqual(len(blocked["attempts"]), 1)
        self.assert_empty(interrupted)
        decision = self.call("resolve", "interrupted-trae", "coding", "fail", "--evidence",
                             "Mixed fixture interruption inspected; kernel subtree empty; reject ambiguous proposal")
        self.assertFalse(decision["work_launched"])
        self.call("start", "--capacity", "2", "--until-idle", "--timeout", "30")
        self.assertEqual(self.call("status", "interrupted-trae")["status"], "FAILED")
        preserved = self.call("status", "completed-codex")
        self.assertEqual(preserved["status"], "SUCCEEDED", preserved)
        self.assertEqual(preserved["result"], accepted["result"])
        self.assertEqual(len(preserved["attempts"]), 1)
        self.assertEqual(receipt_path.read_bytes(), receipt)
        self.assertEqual(bundle_path.read_bytes(), bundle)
        self.assert_empty(completed)


if __name__ == "__main__":
    unittest.main()
