"""Public legacy CLI regressions from independent review round zero."""
from __future__ import annotations

import json
import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "tools/agents/land.py"


class LegacyRepairTest(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(dir=os.environ["TMPDIR"])
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repository"
        self.repo.mkdir()
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.name", "owner")
        self.git("config", "user.email", "owner@example.invalid")
        self.git("config", "collab.overrideOwners", "owner")
        (self.repo / "data").write_text("base\n")
        self.git("add", "data")
        self.git("commit", "-qm", "base")
        self.base = self.git("rev-parse", "HEAD").strip()
        self.worker = self.root / "worker"
        self.git("worktree", "add", "-q", "-b", "topic", str(self.worker))
        (self.worker / "data").write_text("candidate\n")
        self.git("add", "data", cwd=self.worker)
        self.git("commit", "-qm", "candidate", cwd=self.worker)
        self.tip = self.git("rev-parse", "topic").strip()

    def tearDown(self):
        self.temporary.cleanup()

    def git(self, *args, cwd=None):
        result = subprocess.run(["git", "-C", str(cwd or self.repo), *args], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        return result.stdout

    def land(self, gate="true", *args, env=None):
        environment = {**os.environ, "TMPDIR": str(self.root), **(env or {})}
        return subprocess.run(
            [sys.executable, str(SCRIPT), "topic", "--repo", str(self.repo), "--no-push",
             "--owner-mainline-override", "owner", "--expected-tip", self.tip, "--gate", gate, *args],
            capture_output=True, text=True, env=environment,
        )

    def journal(self):
        paths = list((self.repo / ".git/legacy-land").glob("*.json"))
        self.assertEqual(len(paths), 1)
        return json.loads(paths[0].read_text())

    def assert_hidden_gate_refused(self, flag):
        gate = f'git update-index --{flag} data && printf wrong > data && test "$(cat data)" = wrong'
        result = self.land(gate)
        self.assertEqual(result.returncode, 5, result.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)
        self.assertEqual((self.repo / "data").read_text(), "base\n")
        record = self.journal()
        receipt = next(event for event in record["events"] if event["kind"] == "gate_receipt")
        self.assertEqual(receipt["outcome"], "invalid_subject")
        self.assertFalse(receipt["clean"])
        self.assertEqual(record["state"], "gate_failed")
        checkout = Path(receipt["check_path"])
        self.assertTrue(checkout.is_dir())
        self.assertEqual((checkout / "data").read_text(), "wrong")
        self.assertIn(str(checkout), self.git("worktree", "list", "--porcelain"))
        replay = self.land("true", "--recover-landing", record["id"])
        self.assertEqual(replay.returncode, 64, replay.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)

    def test_assume_unchanged_cannot_hide_the_gate_subject_bytes(self):
        self.assert_hidden_gate_refused("assume-unchanged")

    def test_skip_worktree_cannot_hide_the_gate_subject_bytes(self):
        self.assert_hidden_gate_refused("skip-worktree")

    def test_changed_gate_flags_cannot_be_replayed_during_recovery(self):
        result = self.land(env={"COLLAB_FAILPOINT": "before_legacy_local"})
        self.assertEqual(result.returncode, 86, result.stderr)
        record = self.journal()
        receipt = next(event for event in record["events"] if event["kind"] == "gate_receipt")
        checkout = Path(receipt["check_path"])
        self.git("update-index", "--assume-unchanged", "data", cwd=checkout)
        (checkout / "data").write_text("wrong\n")
        recovered = self.land("true", "--recover-landing", record["id"])
        self.assertEqual(recovered.returncode, 64, recovered.stderr)
        self.assertIn("gate workspace changed", recovered.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)
        self.assertEqual((checkout / "data").read_text(), "wrong\n")

    def test_replaced_gate_workspace_cannot_be_replayed_during_recovery(self):
        result = self.land(env={"COLLAB_FAILPOINT": "before_legacy_local"})
        self.assertEqual(result.returncode, 86, result.stderr)
        record = self.journal()
        receipt = next(event for event in record["events"] if event["kind"] == "gate_receipt")
        checkout = Path(receipt["check_path"])
        retained = checkout.with_name(checkout.name + "-original")
        checkout.rename(retained)
        self.git("clone", "-q", "--no-checkout", str(self.repo), str(checkout))
        self.git("checkout", "--detach", "-q", self.tip, cwd=checkout)
        recovered = self.land("true", "--recover-landing", record["id"])
        self.assertEqual(recovered.returncode, 64, recovered.stderr)
        self.assertIn("gate workspace changed", recovered.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)
        self.assertTrue(retained.is_dir())
        self.assertTrue(checkout.is_dir())

    def test_external_gate_worktree_lock_is_preserved(self):
        result = self.land('git worktree lock --reason external "$PWD"')
        self.assertEqual(result.returncode, 5, result.stderr)
        record = self.journal()
        receipt = next(event for event in record["events"] if event["kind"] == "gate_receipt")
        self.assertFalse(receipt["identity"])
        self.assertTrue(Path(receipt["check_path"]).is_dir())
        self.assertIn("locked external", self.git("worktree", "list", "--porcelain"))
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)

    def test_live_gate_descendant_preserves_workspace_and_refuses_acceptance(self):
        pidfile = self.root / "gate-descendant"
        gate = f'sleep 10 >/dev/null 2>&1 & printf %s "$!" > {shlex.quote(str(pidfile))}'
        try:
            result = self.land(gate)
            self.assertEqual(result.returncode, 5, result.stderr)
            record = self.journal()
            receipt = next(event for event in record["events"] if event["kind"] == "gate_receipt")
            self.assertFalse(receipt["quiescent"])
            self.assertTrue(Path(receipt["check_path"]).is_dir())
            self.assertEqual(self.git("rev-parse", "main").strip(), self.base)
        finally:
            if pidfile.exists():
                try:
                    os.kill(int(pidfile.read_text()), signal.SIGKILL)
                except ProcessLookupError:
                    pass


if __name__ == "__main__":
    unittest.main()
