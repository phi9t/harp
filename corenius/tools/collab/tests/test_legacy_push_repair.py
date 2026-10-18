"""Public CLI proofs for legacy push intent and surviving Git children."""
from __future__ import annotations

import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys
import time
import unittest

import test_legacy_repair as fixtures

ROOT = Path(__file__).resolve().parents[3]
SCRIPT = ROOT / "tools/agents/land.py"


class LegacyPushRepairTest(unittest.TestCase):
    setUp = fixtures.LegacyRepairTest.setUp
    tearDown = fixtures.LegacyRepairTest.tearDown
    git = fixtures.LegacyRepairTest.git
    journal = fixtures.LegacyRepairTest.journal

    def remote(self):
        remote = self.root / "remote.git"
        self.git("init", "-q", "--bare", "-b", "main", str(remote))
        self.git("remote", "add", "origin", str(remote))
        self.git("push", "-q", "origin", "main")
        return remote

    def argv(self, *args):
        return [sys.executable, str(SCRIPT), "topic", "--repo", str(self.repo),
                "--owner-mainline-override", "owner", "--expected-tip", self.tip, "--gate", "true", *args]

    def call(self, *args, env=None):
        return subprocess.run(
            self.argv(*args), capture_output=True, text=True,
            env={**os.environ, "TMPDIR": str(self.root), **(env or {})},
        )

    def receive_pack(self, body):
        script = self.root / "receive-pack"
        script.write_text("#!/bin/sh\n" + body)
        script.chmod(0o755)
        self.git("config", "remote.origin.receivepack", str(script))

    def initialize_collab(self):
        result = subprocess.run(
            [sys.executable, str(ROOT / "tools/collab/collab.py"), "--repo", str(self.repo), "init",
             "--trunk", "main", "--check-json", '["true"]', "--override-owners-json", '["owner"]'],
            capture_output=True, text=True,
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def assert_parent_death_keeps_fence(self, initialized=False):
        if initialized:
            self.initialize_collab()
        remote = self.remote()
        entered, release = self.root / "entered", self.root / "release"
        self.receive_pack(
            f"touch {shlex.quote(str(entered))}\n"
            f"while [ ! -f {shlex.quote(str(release))} ]; do sleep 0.02; done\n"
            'exec git-receive-pack "$@"\n'
        )
        process = subprocess.Popen(
            self.argv(), stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True,
            env={**os.environ, "TMPDIR": str(self.root)},
        )
        recovery = None
        try:
            deadline = time.monotonic() + 5
            while not entered.exists() and time.monotonic() < deadline:
                if process.poll() is not None:
                    self.fail("landing exited before the delayed receive-pack was entered")
                time.sleep(0.02)
            self.assertTrue(entered.exists(), "delayed receive-pack never started")
            record = self.journal()
            self.assertEqual(record["state"], "push_started")
            self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.base)
            process.kill()
            process.wait(timeout=5)
            if initialized:
                backup = subprocess.run(
                    [sys.executable, str(ROOT / "tools/collab/collab.py"), "--repo", str(self.repo),
                     "backup", str(self.root / "backup.sqlite3")], capture_output=True, text=True,
                )
                self.assertEqual(backup.returncode, 2, backup.stderr)
                self.assertIn("lane operation busy", backup.stderr)
                self.assertFalse((self.root / "backup.sqlite3").exists())
            recovery = subprocess.Popen(
                self.argv("--recover-landing", record["id"]), stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                text=True, env={**os.environ, "TMPDIR": str(self.root)},
            )
            with self.assertRaises(subprocess.TimeoutExpired):
                recovery.communicate(timeout=0.3)
            self.assertEqual(self.journal()["state"], "push_started")
            release.touch()
            out, err = recovery.communicate(timeout=5)
            self.assertEqual(recovery.returncode, 0, err)
            self.assertEqual(out.strip(), self.tip)
            self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.tip)
            self.assertEqual(self.git("rev-parse", "main").strip(), self.tip)
            self.assertEqual(self.journal()["state"], "landed")
        finally:
            release.touch()
            if process.poll() is None:
                process.kill()
            try:
                process.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.communicate(timeout=5)
            if recovery is not None and recovery.poll() is None:
                recovery.kill()
                recovery.communicate(timeout=5)

    def test_parent_death_cannot_release_a_delayed_push_fence(self):
        self.assert_parent_death_keeps_fence()

    def test_initialized_lane_stays_fenced_while_push_survives_parent(self):
        self.assert_parent_death_keeps_fence(initialized=True)

    def test_unchanged_remote_after_transport_failure_remains_uncertain(self):
        remote = self.remote()
        self.receive_pack("echo transport-disconnected >&2\nexit 1\n")
        result = self.call()
        self.assertEqual(result.returncode, 6, result.stderr)
        self.assertIn("transport-disconnected", result.stderr)
        record = self.journal()
        self.assertEqual(record["state"], "push_started")
        recovered = self.call("--recover-landing", record["id"])
        self.assertEqual(recovered.returncode, 64, recovered.stderr)
        self.assertIn("unknown", recovered.stderr)
        self.assertEqual(self.journal()["state"], "push_started")
        blocked = self.call()
        self.assertEqual(blocked.returncode, 64, blocked.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.base)

    def test_recorded_push_start_without_an_outcome_cannot_be_rolled_back(self):
        remote = self.remote()
        result = self.call(env={"COLLAB_FAILPOINT": "after_legacy_push_started"})
        self.assertEqual(result.returncode, 86, result.stderr)
        record = self.journal()
        self.assertEqual(record["state"], "push_started")
        recovered = self.call("--recover-landing", record["id"])
        self.assertEqual(recovered.returncode, 64, recovered.stderr)
        self.assertIn("unknown", recovered.stderr)
        self.assertEqual(self.journal()["state"], "push_started")
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.base)

    def assert_explicit_target_rejection_recoverable(self):
        remote = self.remote()
        hook = remote / "hooks/pre-receive"
        hook.write_text("#!/bin/sh\necho explicit-policy-rejected >&2\nexit 1\n")
        hook.chmod(0o755)
        result = self.call()
        self.assertEqual(result.returncode, 6, result.stderr)
        self.assertIn("explicit-policy-rejected", result.stderr)
        record = self.journal()
        self.assertEqual(record["state"], "push_failed")
        proof = next(event for event in record["events"] if event["kind"] == "push_rejection")
        self.assertEqual(proof["target_ref"], "refs/heads/main")
        self.assertEqual(proof["source_ref"], self.tip)
        recovered = self.call("--recover-landing", record["id"])
        self.assertEqual(recovered.returncode, 0, recovered.stderr)
        self.assertEqual(self.journal()["state"], "rolled_back")
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.base)
        hook.unlink()
        retry = self.call()
        self.assertEqual(retry.returncode, 0, retry.stderr)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.tip)

    def test_explicit_target_rejection_can_be_recovered_without_moving_refs(self):
        self.assert_explicit_target_rejection_recoverable()

    def test_initialized_target_rejection_is_recoverable_and_recorded(self):
        self.initialize_collab()
        self.assert_explicit_target_rejection_recoverable()

    def test_failure_after_remote_update_recovers_the_exact_tip(self):
        remote = self.remote()
        self.receive_pack(
            'git-receive-pack "$@"\n'
            'status=$?\n'
            '[ "$status" -eq 0 ] || exit "$status"\n'
            'echo transport-failed-after-update >&2\n'
            'exit 1\n'
        )
        result = self.call()
        self.assertEqual(result.returncode, 6, result.stderr)
        self.assertIn("transport-failed-after-update", result.stderr)
        self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.tip)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.base)
        record = self.journal()
        self.assertEqual(record["state"], "pushed")
        self.assertFalse(any(event["kind"] == "push_rejection" for event in record["events"]))
        recovered = self.call("--recover-landing", record["id"])
        self.assertEqual(recovered.returncode, 0, recovered.stderr)
        self.assertEqual(recovered.stdout.strip(), self.tip)
        self.assertEqual(self.git("rev-parse", "main").strip(), self.tip)
        self.assertEqual(self.journal()["state"], "landed")


if __name__ == "__main__":
    unittest.main()
