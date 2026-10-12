"""Regression proof for terminal cleanup and ambiguous push outcomes at the public CLI."""
import json
from pathlib import Path
import subprocess
import sys
import unittest

import test_collab as fixtures


class CoreRepairTests(unittest.TestCase):
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
    add_bare_remote = fixtures.CollabTests.add_bare_remote
    remote_head = fixtures.CollabTests.remote_head

    def terminal(self, *binding):
        self.init()
        self.ticket()
        self.commit("ticket")
        attempt = self.cli("open-attempt", "T-one", "--owner", "worker", *binding)
        self.contribute(attempt)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        self.cli("land", attempt["id"], "--no-push")
        return checked

    def test_gc_checks_live_worker_and_all_external_locks_before_any_cleanup(self):
        worker = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(120)"], env=self.env)
        def stop_worker():
            if worker.poll() is None:
                worker.kill()
            worker.wait(timeout=5)

        self.addCleanup(stop_worker)
        attempt = self.terminal("--worker-pid", str(worker.pid))
        result = self.cli("gc", attempt["id"], "--confirm-stopped", ok=False)
        self.assertIn("still running", result.stderr)
        worker.terminate()
        worker.wait(timeout=5)
        for name, owned_prefix in (("path", "collab:"), ("check_path", "collab-check:")):
            self.git("worktree", "unlock", attempt[name])
            self.git("worktree", "lock", "--reason", "external owner", attempt[name])
            before = self.git("worktree", "list", "--porcelain")
            result = self.cli("gc", attempt["id"], "--confirm-stopped", ok=False)
            self.assertIn("external owner", result.stderr)
            self.assertEqual(self.git("worktree", "list", "--porcelain"), before)
            self.assertTrue(Path(attempt["path"]).exists())
            self.assertTrue(Path(attempt["check_path"]).exists())
            self.git("worktree", "unlock", attempt[name])
            self.git("worktree", "lock", "--reason", owned_prefix + attempt["id"], attempt[name])
        removed = self.cli("gc", attempt["id"], "--confirm-stopped")
        self.assertEqual(set(removed["removed"]), {attempt["path"], attempt["check_path"]})

    def test_gc_retains_all_worktrees_when_worker_binding_becomes_unknown(self):
        run = self.root / "run"
        run.mkdir()
        (run / "run.json").write_text(json.dumps({"pid": 2147483647}))
        attempt = self.terminal("--worker-run-dir", str(run))
        (run / "run.json").write_text("{unreadable")
        before = self.git("worktree", "list", "--porcelain")
        result = self.cli("gc", attempt["id"], "--confirm-stopped", ok=False)
        self.assertIn("unknown", result.stderr)
        self.assertEqual(self.git("worktree", "list", "--porcelain"), before)
        self.assertTrue(Path(attempt["path"]).exists())
        self.assertTrue(Path(attempt["check_path"]).exists())

    def fail_after_real_push(self):
        script = self.root / "receive-then-fail"
        calls = self.root / "push-calls"
        script.write_text(
            "#!/usr/bin/env python3\nimport subprocess, sys\nfrom pathlib import Path\n"
            f"with Path({str(calls)!r}).open('a') as stream: stream.write('push\\n')\n"
            "subprocess.call(['git-receive-pack', *sys.argv[1:]])\nsys.exit(1)\n"
        )
        script.chmod(0o700)
        self.git("config", "remote.origin.receivepack", str(script))
        return calls

    def test_nonzero_after_real_push_recovers_acknowledged_tip_without_retry(self):
        attempt = self.prepared()
        remote = self.add_bare_remote()
        checked = self.cli("check", attempt["id"])
        calls = self.fail_after_real_push()
        failed = self.cli("land", attempt["id"], ok=False)
        self.assertIn("Push failed", failed.stderr)
        self.assertEqual(self.remote_head(remote), checked["integration"])
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])
        self.assertEqual(self.cli("show", attempt["id"])["state"], "push_intended")
        recovered = self.cli("recover", attempt["id"], "--confirm-stopped")
        self.assertEqual(recovered["state"], "landed")
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["integration"])
        self.assertEqual(calls.read_text().splitlines(), ["push"])

    def test_uninspectable_remote_keeps_push_outcome_uncertain(self):
        attempt = self.prepared()
        remote = self.add_bare_remote()
        checked = self.cli("check", attempt["id"])
        self.fail_after_real_push()
        self.cli("land", attempt["id"], ok=False)
        inaccessible = remote.with_name("inaccessible.git")
        remote.rename(inaccessible)
        result = self.cli("recover", attempt["id"], "--confirm-stopped", ok=False)
        self.assertIn("Cannot inspect", result.stderr)
        self.assertEqual(self.cli("show", attempt["id"])["state"], "push_intended")
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])
        inaccessible.rename(remote)
        self.assertEqual(self.cli("recover", attempt["id"], "--confirm-stopped")["state"], "landed")

    def test_remote_base_or_absence_cannot_prove_push_never_executed(self):
        attempt = self.prepared()
        remote = self.add_bare_remote()
        checked = self.cli("check", attempt["id"])
        crashed = self.cli("land", attempt["id"], failpoint="after_push_intent", ok=False)
        self.assertEqual(crashed.returncode, 86)
        for absent in (False, True):
            if absent:
                self.git("--git-dir", str(remote), "update-ref", "-d", "refs/heads/main")
            result = self.cli("recover", attempt["id"], "--confirm-stopped", ok=False)
            self.assertIn("inspect", result.stderr.lower())
            self.assertEqual(self.cli("show", attempt["id"])["state"], "push_intended")
            self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])

    def test_recovered_push_checks_ignored_overlap_and_receipt_before_local_ff(self):
        attempt = self.prepared()
        worker = Path(attempt["path"])
        (worker / "build").mkdir()
        (worker / "build" / "ship").write_text("checked file\n")
        self.git("add", "--force", "build/ship", cwd=worker)
        self.commit("ignored path contribution", cwd=worker)
        self.cli("submit", attempt["id"])
        remote = self.add_bare_remote()
        checked = self.cli("check", attempt["id"])
        self.fail_after_real_push()
        self.cli("land", attempt["id"], ok=False)
        (self.repo / "build").mkdir()
        (self.repo / "build" / "ship").write_text("operator file\n")
        blocked = self.cli("recover", attempt["id"], "--confirm-stopped", ok=False)
        self.assertIn("overwritten", blocked.stderr)
        self.assertEqual((self.repo / "build" / "ship").read_text(), "operator file\n")
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])
        (self.repo / "build" / "ship").unlink()
        original_log = Path(checked["log"]).read_bytes()
        Path(checked["log"]).write_bytes(original_log + b"tampered")
        blocked = self.cli("recover", attempt["id"], "--confirm-stopped", ok=False)
        self.assertIn("receipt", blocked.stderr)
        self.assertEqual(self.git("rev-parse", "HEAD"), checked["base"])
        self.assertEqual(self.remote_head(remote), checked["integration"])
        Path(checked["log"]).write_bytes(original_log)
        self.assertEqual(self.cli("recover", attempt["id"], "--confirm-stopped")["state"], "landed")


if __name__ == "__main__":
    unittest.main()
