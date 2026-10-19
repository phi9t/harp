"""Public CLI verification against real nested namespaces and delegated cgroups."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import sys
import unittest

import durable.jobs.coding.test_admission as admission_tests


@unittest.skipUnless(sys.platform == "linux" and all(os.environ.get(name) for name in
                     ("HARP_TEST_CGROUP_ROOT", "HARP_TEST_ROOTFS", "HARP_DURABLE_BIN")),
                     "requires public Harp binary, admitted native rootfs and delegated Linux cgroup")
class VerificationLifecycle(unittest.TestCase):
    # Reuse process/Git setup without inheriting unrelated admission tests.
    setUp = admission_tests.CodingLifecycle.setUp
    git = admission_tests.GitInputs.git
    call = admission_tests.CodingLifecycle.call
    admit = admission_tests.CodingLifecycle.admit

    def capture_with_checks(self, checks):
        (self.repo / "acceptance.json").write_text(json.dumps({"allowed_paths": ["**"], "checks": checks}))
        self.git("add", "acceptance.json")
        self.git("commit", "-qm", "Pin verification commands")
        self.base = self.git("rev-parse", "HEAD").strip()
        self.admit("coding")
        self.call("start", "--until-idle", "--timeout", "30")
        status = self.call("status", "coding")
        self.assertEqual(status["status"], "SUCCEEDED", self.diagnostics(status))
        return self.call("capture", "coding"), status

    def diagnostics(self, status):
        return {"status": status, "stderr": [Path(a["logs"]["stderr"]).read_text()
                for a in status["attempts"] if Path(a["logs"]["stderr"]).exists()]}

    def finish_verification(self):
        requested = self.call("verify", "coding")
        self.call("start", "--until-idle", "--timeout", "30")
        status = self.call("status", requested["run_id"])
        self.assertEqual(status["status"], "SUCCEEDED", self.diagnostics(status))
        return status

    def payload(self, status):
        self.assertEqual(len(status["attempts"]), 1)
        return Path(status["attempts"][0]["directory"]) / "payload"

    def assert_kernel_cleanup(self, status):
        from insula.cgroup import Cgroup
        attempt = status["attempts"][0]
        receipt = json.loads((Path(attempt["directory"]) / "receipt.json").read_text())
        self.assertTrue(receipt["cleanup"])
        proof = receipt["cleanup_proof"]
        self.assertEqual(proof["backend"], "cgroup-v2")
        self.assertEqual(proof["populated"], 0)
        group = Cgroup.resume(proof["identity"], attempt_id=attempt["id"])
        self.assertIn("populated 0", (group.path / "cgroup.events").read_text().splitlines())
        self.assertEqual((group.path / "cgroup.procs").read_text().strip(), "")

    def test_exact_capture_verifies_after_worker_workspace_is_changed_and_removed(self):
        checks = [["/usr/bin/python3", "-I", "-c",
                   "from pathlib import Path; assert Path('answer.txt').read_text() == '42\\n'"]]
        captured, coding = self.capture_with_checks(checks)
        worker_payload = self.payload(coding)
        (worker_payload / "workspace/answer.txt").write_text("worker changed after capture\n")
        shutil.rmtree(worker_payload)
        (self.repo / "task.md").write_text("operator working edit remains here\n")
        status = self.finish_verification()
        proof = status["result"]
        self.assertTrue(proof["passed"], proof)
        self.assertEqual(proof["subject"], captured["subject"])
        self.assertEqual(proof["subject_digest"], captured["subject_digest"])
        self.assertEqual(proof["checks"][0]["argv"], checks[0])
        self.assertEqual((self.payload(status) / "workspace/answer.txt").read_text(), "42\n")
        self.assertEqual(self.git("rev-parse", "HEAD").strip(), self.base)
        self.assertEqual((self.repo / "task.md").read_text(), "operator working edit remains here\n")
        self.assert_kernel_cleanup(status)

    def test_nonzero_configured_check_is_inspectable_and_never_passes(self):
        self.capture_with_checks([["/usr/bin/python3", "-I", "-c",
                                   "import sys; print('configured failure',flush=True); sys.exit(23)"]])
        status = self.finish_verification()
        proof = status["result"]
        self.assertFalse(proof["passed"])
        check = proof["checks"][0]
        self.assertEqual(check["returncode"], 23)
        self.assertTrue(check["tracked_source_unchanged"])
        log = self.payload(status) / check["logs"]["stdout"]["path"]
        self.assertIn("configured failure", log.read_text())
        self.assertEqual(hashlib.sha256(log.read_bytes()).hexdigest(), check["logs"]["stdout"]["sha256"])
        self.assert_kernel_cleanup(status)

    def test_hostile_check_cannot_forge_parent_result_or_logs(self):
        attack = ("from pathlib import Path; import json,sys; root=Path('/home/bench/run'); "
                  "(root/'checks').mkdir(); "
                  "(root/'checks/000.stdout.log').write_text('forged-parent-log'); "
                  "(root/'result.json').write_text(json.dumps({'passed':True})); "
                  "(root/'progress.json').write_text(json.dumps({'passed':True})); "
                  "print('hostile child output',flush=True); sys.exit(17)")
        captured, _ = self.capture_with_checks([["/usr/bin/python3", "-I", "-c", attack]])
        status = self.finish_verification()
        proof = status["result"]
        self.assertFalse(proof["passed"])
        self.assertEqual(proof["checks"][0]["returncode"], 17)
        self.assertEqual(proof["subject_digest"], captured["subject_digest"])
        payload = self.payload(status)
        self.assertEqual(json.loads((payload / "result.json").read_text()), proof)
        progress = json.loads((payload / "progress.json").read_text())
        self.assertEqual(progress["subject_digest"], captured["subject_digest"])
        self.assertFalse(progress["passed"])
        log = payload / proof["checks"][0]["logs"]["stdout"]["path"]
        self.assertEqual(log.read_text(), "hostile child output\n")
        self.assert_kernel_cleanup(status)

    def test_zero_exit_cannot_pass_after_mutating_tracked_candidate_bytes(self):
        captured, _ = self.capture_with_checks([["/usr/bin/python3", "-I", "-c",
            "from pathlib import Path; Path('answer.txt').write_text('substituted source\\n')"]])
        status = self.finish_verification()
        proof = status["result"]
        self.assertEqual(proof["checks"][0]["returncode"], 0)
        self.assertFalse(proof["checks"][0]["tracked_source_unchanged"])
        self.assertFalse(proof["passed"])
        self.assertEqual(proof["subject"], captured["subject"])
        bundle = self.state / "inputs" / captured["inputs"] / "candidate.bundle"
        self.assertEqual(hashlib.sha256(bundle.read_bytes()).hexdigest(), captured["subject"]["bundle_sha256"])
        self.assert_kernel_cleanup(status)

    def test_outer_deadline_retains_partial_logs_and_proves_kernel_cleanup(self):
        # Both a sleeping check and its forked descendant must disappear under
        # the owning guardian. No persisted bare PID is used as cleanup proof.
        hanging = ("import os,time; print('verification-hung',flush=True); "
                   "os.fork(); time.sleep(120)")
        self.capture_with_checks([["/usr/bin/python3", "-I", "-c", hanging]])
        requested = self.call("verify", "coding")
        error = self.call("start", "--until-idle", "--timeout", "10", ok=False)
        self.assertEqual(error["code"], "TimeoutError")
        status = self.call("status", requested["run_id"])
        self.assertEqual(status["status"], "BLOCKED", self.diagnostics(status))
        self.assertIsNone(status["result"])
        payload = self.payload(status)
        progress = json.loads((payload / "progress.json").read_text())
        self.assertEqual(progress["running_check"], 0)
        self.assertFalse(progress["passed"])
        self.assertIn("verification-hung", (payload / "checks/000.stdout.log").read_text())
        self.assertFalse((payload / "result.json").exists())
        self.assert_kernel_cleanup(status)
        self.assertFalse(self.call("status")["locked"])


if __name__ == "__main__":
    unittest.main()
