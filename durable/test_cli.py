"""Observable lifecycle checks using real Linux processes and SQLite state."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

ROOT = next(p for p in Path(__file__).resolve().parents if (p / "Cargo.toml").is_file() and (p / "durable").is_dir())


@unittest.skipUnless(sys.platform == "linux", "execution contract is Linux; run qualification on a qualified Linux host")
class Lifecycle(unittest.TestCase):
    def setUp(self):
        scratch = ROOT / ".scratch" / "durable-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        self.temp = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(self.temp.cleanup)
        self.state = Path(self.temp.name) / "state"
        self.env = {**os.environ, "PYTHONPATH": str(ROOT),
                    "PYTHONDONTWRITEBYTECODE": "1", "PYTHONNOUSERSITE": "1",
                    "TMPDIR": self.temp.name}
        entry = ([os.environ["HARP_DURABLE_BIN"], "--format", "json", "durable"]
                 if "HARP_DURABLE_BIN" in os.environ else [sys.executable, "-m", "durable"])
        self.command = [*entry, "--state", str(self.state)]

    def call(self, *args, ok=True):
        result = subprocess.run([*self.command, *args], cwd=ROOT, env=self.env,
                                text=True, capture_output=True, timeout=20)
        self.assertEqual(result.returncode == 0, ok, result.stderr)
        return json.loads(result.stdout if ok else result.stderr)

    def wait_running(self, process):
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if process.poll() is not None:
                self.fail(process.communicate()[1])
            if (self.state / "instance.json").exists() and self.call("status")["data"]["locked"]:
                return
            time.sleep(0.03)
        self.fail("runtime never acquired state")

    def launch(self):
        process = subprocess.Popen([*self.command, "start", "--timeout", "15"],
                                   cwd=ROOT, env=self.env, text=True,
                                   stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        def cleanup():
            if process.poll() is None:
                process.terminate()
            process.communicate(timeout=10)
        self.addCleanup(cleanup)
        self.wait_running(process)
        return process

    def test_completed_activity_replays_without_reexecution(self):
        self.call("submit", "square", "--value", "7")
        self.assertEqual(self.call("status", "square")["data"]["phase"], "pending")
        self.call("start", "--until-idle", "--timeout", "10")
        before = self.call("status", "square")["data"]
        self.assertEqual(before["status"], "SUCCEEDED")
        self.assertEqual(before["result"]["square"], 49)
        self.assertEqual(len(before["attempts"]), 1)
        self.assertTrue(Path(before["attempts"][0]["logs"]["stderr"]).is_file())
        self.call("start", "--until-idle", "--timeout", "10")
        after = self.call("status", "square")["data"]
        self.assertEqual(after["attempts"], before["attempts"])
        self.assertEqual(self.call("check")["data"]["integrity_check"], "ok")

    def test_competing_runtime_refused_and_explicit_stop_retains_state(self):
        process = self.launch()
        error = self.call("start", "--until-idle", ok=False)
        self.assertEqual(error["error"]["code"], "ControllerBusy")
        self.assertTrue(self.call("stop")["data"]["stop_requested"])
        out, err = process.communicate(timeout=10)
        self.assertEqual(process.returncode, 0, err)
        self.assertEqual(json.loads(out)["data"]["instance"]["status"], "stopped")
        self.assertFalse(self.call("status")["data"]["locked"])
        replacement = self.launch()
        self.assertTrue(self.call("status")["data"]["locked"])
        self.call("stop")
        replacement.communicate(timeout=10)
        self.assertEqual(replacement.returncode, 0)

    def test_rejects_nonprivate_state_before_dispatch(self):
        self.state.mkdir(mode=0o755)
        error = self.call("check", ok=False)
        self.assertIn("private", error["error"]["message"])
        self.assertFalse((self.state / "runtime.sqlite").exists())

    def test_stop_during_activity_cleans_up_and_reopens(self):
        self.call("submit", "slow", "--sleep", "0.5")
        process = self.launch()
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline:
            if self.call("status", "slow")["data"]["attempts"]:
                break
            time.sleep(0.03)
        else:
            self.fail("activity was never dispatched")
        self.call("stop")
        out, err = process.communicate(timeout=10)
        self.assertEqual(process.returncode, 0, err)
        self.assertFalse(self.call("status")["data"]["locked"])
        self.call("start", "--until-idle", "--timeout", "10")
        self.assertEqual(self.call("status", "slow")["data"]["status"], "SUCCEEDED")

    def test_symlinked_database_and_lock_never_touch_external_targets(self):
        self.state.mkdir(mode=0o700)
        target = Path(self.temp.name) / "external"
        target.write_bytes(b"original external bytes")
        for name in ("runtime.sqlite", "runtime.sqlite-wal", "controller.lock", "attempts"):
            with self.subTest(name=name):
                link = self.state / name
                link.symlink_to(target)
                error = self.call("check", ok=False)
                self.assertIn("symlinks", error["error"]["message"])
                self.assertEqual(target.read_bytes(), b"original external bytes")
                link.unlink()


if __name__ == "__main__":
    unittest.main()
