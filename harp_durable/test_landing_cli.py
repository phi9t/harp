"""Human approval and physical Git landing through the public Linux CLI."""
import json
import os
from pathlib import Path
import subprocess
import sys
import unittest

from . import test_admission as admission_tests

ROOT = admission_tests.ROOT


@unittest.skipUnless(sys.platform == "linux" and all(os.environ.get(name) for name in
                     ("HARP_TEST_CGROUP_ROOT", "HARP_TEST_ROOTFS", "HARP_DURABLE_BIN")),
                     "requires public Harp binary, admitted native rootfs and delegated Linux cgroup")
class LandingLifecycle(unittest.TestCase):
    setUp = admission_tests.CodingLifecycle.setUp
    git = admission_tests.GitInputs.git
    call = admission_tests.CodingLifecycle.call
    admit = admission_tests.CodingLifecycle.admit

    def verified(self):
        self.admit("coding")
        self.call("start", "--until-idle", "--timeout", "30")
        proposal = self.call("capture", "coding")
        requested = self.call("verify", "coding")
        self.call("start", "--until-idle", "--timeout", "30")
        proof = self.call("status", requested["run_id"])
        self.assertEqual(proof["status"], "SUCCEEDED", proof)
        self.assertTrue(proof["result"]["passed"], proof)
        self.capability = self.state.parent / "owner" / "capability"
        self.capability.parent.mkdir(mode=0o700)
        self.call("owner-init", "--capability", str(self.capability))
        return proposal, requested

    def approve(self, requested):
        return self.call("approve", "coding", "--verification", requested["run_id"],
                         "--destination", str(self.repo), "--capability", str(self.capability))

    def landed(self, approval, proposal):
        result = self.call("land", approval["approval_id"], "--capability", str(self.capability))
        self.assertEqual(result["status"], "LANDED", result)
        self.assertEqual(result["commit"], proposal["subject"]["commit"])
        self.assertEqual(self.git("rev-parse", "HEAD").strip(), proposal["subject"]["commit"])
        self.assertEqual(self.git("write-tree").strip(), proposal["subject"]["tree"])
        self.assertEqual(self.git("status", "--porcelain=v1"), "")
        self.assertEqual((self.repo / "answer.txt").read_text(), "42\n")
        return result

    def snapshot(self):
        return {"head": self.git("rev-parse", "HEAD"), "index": self.git("write-tree"),
                "status": self.git("status", "--porcelain=v1"),
                "files": {str(p.relative_to(self.repo)): p.read_bytes() for p in self.repo.rglob("*")
                          if p.is_file() and ".git" not in p.relative_to(self.repo).parts}}

    def test_owner_approval_is_required_and_success_physically_synchronizes_checkout(self):
        proposal, requested = self.verified()
        wrong = self.capability.parent / "worker-capability"
        wrong.write_text("worker does not possess the human capability")
        wrong.chmod(0o600)
        before = self.snapshot()
        self.call("approve", "coding", "--verification", requested["run_id"],
                  "--destination", str(self.repo), "--capability", str(wrong), ok=False)
        self.call("land", "invented-approval", "--capability", str(self.capability), ok=False)
        self.assertEqual(self.snapshot(), before)
        # A later input selection cannot expose an already registered capability.
        owner_repo = self.capability.parent
        subprocess.run(["git", "-C", str(owner_repo), "init", "-q"], check=True)
        for name in ("task.md", "acceptance.json"):
            (owner_repo / name).write_bytes((self.repo / name).read_bytes())
        subprocess.run(["git", "-C", str(owner_repo), "add", "task.md", "acceptance.json"], check=True)
        subprocess.run(["git", "-C", str(owner_repo), "-c", "user.name=Owner fixture",
                        "-c", "user.email=fixture@harp.local", "commit", "-qm", "Pin separate inputs"], check=True)
        error = self.call("admit", "expose-owner", "--repository", str(owner_repo),
                          "--task", "task.md", "--acceptance", "acceptance.json",
                          "--provider", "deterministic", ok=False)
        self.assertIn("owner capability", error["message"])
        self.call("status", "expose-owner", ok=False)
        alias = self.state.parent / "owner-alias"
        alias.symlink_to(self.capability.parent, target_is_directory=True)
        self.call("approve", "coding", "--verification", requested["run_id"],
                  "--destination", str(self.repo), "--capability", str(alias / self.capability.name), ok=False)
        destination_alias = self.state.parent / "repository-alias"
        destination_alias.symlink_to(self.repo, target_is_directory=True)
        self.call("approve", "coding", "--verification", requested["run_id"],
                  "--destination", str(destination_alias), "--capability", str(self.capability), ok=False)
        approval = self.approve(requested)
        self.assertEqual(self.snapshot(), before, "approval itself must not land")
        first = self.landed(approval, proposal)
        second = self.landed(approval, proposal)
        self.assertEqual(first["approval_id"], second["approval_id"])
        # A valid owner outside all projections does not block later execution.
        self.admit("followup")
        self.call("start", "--until-idle", "--timeout", "30")
        self.assertEqual(self.call("status", "followup")["status"], "SUCCEEDED")
        self.assertEqual(self.git("rev-parse", "HEAD").strip(), proposal["subject"]["commit"])

    def test_dirty_destination_is_preserved_then_explicit_retry_lands(self):
        proposal, requested = self.verified()
        approval = self.approve(requested)
        self.git("config", "core.autocrlf", "true")
        before = self.snapshot()
        self.call("land", approval["approval_id"], "--capability", str(self.capability), ok=False)
        self.assertEqual(self.snapshot(), before, "conversion refusal must precede Git mutation")
        self.git("config", "--unset", "core.autocrlf")
        (self.repo / "task.md").write_text("operator work must survive\n")
        (self.repo / "local-only.txt").write_text("untracked operator work\n")
        before = self.snapshot()
        error = self.call("land", approval["approval_id"], "--capability", str(self.capability), ok=False)
        self.assertIn("dirty", error["message"].lower())
        self.assertEqual(self.snapshot(), before)
        # Remove only this test's explicitly created fixture changes.
        (self.repo / "task.md").write_text("Produce answer.txt containing 42.\n")
        (self.repo / "local-only.txt").unlink()
        self.landed(approval, proposal)

    def test_advanced_destination_invalidates_old_candidate_and_approval(self):
        _proposal, requested = self.verified()
        approval = self.approve(requested)
        (self.repo / "other.txt").write_text("independent accepted work\n")
        self.git("add", "other.txt")
        self.git("commit", "-qm", "Advance destination independently")
        before = self.snapshot()
        self.call("land", approval["approval_id"], "--capability", str(self.capability), ok=False)
        self.call("approve", "coding", "--verification", requested["run_id"],
                  "--destination", str(self.repo), "--capability", str(self.capability), ok=False)
        self.assertEqual(self.snapshot(), before)
        self.assertFalse((self.repo / "answer.txt").exists())

    def test_lost_acknowledgment_reconciles_exact_git_effect(self):
        proposal, requested = self.verified()
        approval = self.approve(requested)
        for point in ("after_intent", "after_git"):
            process = subprocess.run([*self.command, "land", approval["approval_id"],
                                      "--capability", str(self.capability), "--test-crash-at", point],
                                     cwd=ROOT, env=self.env, text=True, capture_output=True, timeout=60)
            self.assertEqual(process.returncode, 86, {"out": process.stdout, "err": process.stderr})
            marker = json.loads((self.state / "test-landing-crash.json").read_text())
            self.assertEqual(marker["point"], point)
            expected = self.base if point == "after_intent" else proposal["subject"]["commit"]
            self.assertEqual(self.git("rev-parse", "HEAD").strip(), expected)
        reconciled = self.landed(approval, proposal)
        self.assertTrue(reconciled["reconciled"], reconciled)
