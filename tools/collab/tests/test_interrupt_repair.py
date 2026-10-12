"""Controller-only SIGINT proof using public CLIs, real gates, and real receive-pack."""
import json
import os
from pathlib import Path
import shlex
import signal
import subprocess
import sys
import time
import unittest

import test_collab as fixtures
import test_legacy_push_repair as legacy


def wait_file(test, path, controller=None):
    deadline = time.monotonic() + 5
    while not path.exists() and time.monotonic() < deadline:
        if controller is not None and controller.poll() is not None:
            test.fail("controller exited before its child reached the gate")
        time.sleep(0.02)
    test.assertTrue(path.exists(), str(path))


class InterruptCheckTests(unittest.TestCase):
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

    def test_sigint_only_to_controller_drains_gate_before_recovery(self):
        pidfile = self.root / "gate-pid"
        code = ("import os, signal, time; from pathlib import Path; "
                "signal.signal(signal.SIGTERM, signal.SIG_IGN); "
                f"Path({str(pidfile)!r}).write_text(str(os.getpid())); time.sleep(120)")
        attempt = self.prepared([sys.executable, "-c", code])
        controller = subprocess.Popen([sys.executable, str(fixtures.CLI), "--repo", str(self.repo),
                                       "check", attempt["id"]], env=self.env,
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        def stop_children():
            if controller.poll() is None:
                controller.kill()
            controller.communicate(timeout=5)
            if pidfile.exists():
                try:
                    os.kill(int(pidfile.read_text()), signal.SIGKILL)
                except ProcessLookupError:
                    pass

        self.addCleanup(stop_children)
        wait_file(self, pidfile, controller)
        check_path = Path(self.cli("show", attempt["id"])["check_path"])
        os.kill(controller.pid, signal.SIGINT)
        out, err = controller.communicate(timeout=5)
        self.assertNotEqual(controller.returncode, 0, str((out, err)))
        stat = Path("/proc") / pidfile.read_text() / "stat"
        self.assertFalse(stat.exists() and stat.read_text().rsplit(")", 1)[1].strip().split()[0] != "Z",
                         "interrupted controller returned while its gate remained alive")
        self.assertTrue(check_path.exists())
        current = self.cli("show", attempt["id"])
        self.assertEqual(current["state"], "checking")
        self.assertIsNone(current["check_receipt"])
        self.assertEqual(self.cli("recover", attempt["id"], "--confirm-stopped")["state"], "check_failed")


class InterruptLegacyTests(unittest.TestCase):
    setUp = legacy.LegacyPushRepairTest.setUp
    tearDown = legacy.LegacyPushRepairTest.tearDown
    git = legacy.LegacyPushRepairTest.git
    journal = legacy.LegacyPushRepairTest.journal
    remote = legacy.LegacyPushRepairTest.remote
    argv = legacy.LegacyPushRepairTest.argv
    call = legacy.LegacyPushRepairTest.call
    initialize_collab = legacy.LegacyPushRepairTest.initialize_collab

    def exercise_sigint_push(self, initialized):
        if initialized:
            self.initialize_collab()
        remote = self.remote()
        entered, release = self.root / "receive-entered", self.root / "receive-release"
        hook = remote / "hooks/pre-receive"
        hook.write_text("#!/bin/sh\n"
                        f"touch {shlex.quote(str(entered))}\n"
                        f"while [ ! -f {shlex.quote(str(release))} ]; do sleep 0.02; done\n"
                        "exit 0\n")
        hook.chmod(0o755)
        controller = subprocess.Popen(self.argv(), stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                      text=True, start_new_session=True,
                                      env={**os.environ, "TMPDIR": str(self.root)})
        recovery = None
        try:
            wait_file(self, entered, controller)
            record = self.journal()
            self.assertEqual(record["state"], "push_started")
            os.kill(controller.pid, signal.SIGINT)
            controller.wait(timeout=5)
            self.assertNotEqual(controller.returncode, 0)
            if initialized:
                result = subprocess.run([sys.executable, str(legacy.ROOT / "tools/collab/collab.py"),
                                         "--repo", str(self.repo), "backup", str(self.root / "backup.sqlite3")],
                                        text=True, capture_output=True)
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertIn("lane operation busy", result.stderr)
                self.assertFalse((self.root / "backup.sqlite3").exists())
            recovery = subprocess.Popen(self.argv("--recover-landing", record["id"]),
                                        stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
            with self.assertRaises(subprocess.TimeoutExpired):
                recovery.communicate(timeout=0.3)
            self.assertEqual(self.journal()["state"], "push_started")
            release.touch()
            out, err = recovery.communicate(timeout=5)
            self.assertEqual(recovery.returncode, 0, err)
            self.assertEqual(self.git("rev-parse", "main", cwd=remote).strip(), self.tip)
            self.assertEqual(self.git("rev-parse", "main").strip(), self.tip)
            self.assertEqual(self.journal()["state"], "landed")
        finally:
            release.touch()
            if controller.poll() is None:
                controller.kill()
            try:
                controller.communicate(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(controller.pid, signal.SIGKILL)
                controller.communicate(timeout=5)
            if recovery is not None and recovery.poll() is None:
                recovery.kill()
                recovery.communicate(timeout=5)

    def test_sigint_does_not_release_surviving_legacy_receive_pack_fence(self):
        self.exercise_sigint_push(initialized=False)

    def test_sigint_does_not_release_surviving_initialized_lane_fence(self):
        self.exercise_sigint_push(initialized=True)


if __name__ == "__main__":
    unittest.main()
