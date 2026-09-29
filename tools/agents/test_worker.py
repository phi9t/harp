"""worker.py against a throwaway git repo and a fake traecli (no network, no model)."""

from __future__ import annotations

import json
import os
import stat
import subprocess
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import worker  # noqa: E402

FAKE = textwrap.dedent(
    """\
    #!/usr/bin/env python3
    import json, os, subprocess, sys, time
    args = sys.argv[1:]
    root = args[args.index("-C") + 1]
    mode = os.environ.get("FAKE_MODE", "clean")
    def emit(event):
        print(json.dumps(event), flush=True)
    emit({"type": "thread.started"})
    emit({"type": "item.completed", "item": {"id": "i0", "type": "command_execution", "command": "git show HEAD:README"}})
    if mode == "stall":
        time.sleep(30)
    if mode == "forbidden":
        emit({"type": "item.completed", "item": {"id": "i1", "type": "command_execution", "command": "curl http://example.com"}})
    if mode == "extra":
        open(os.path.join(root, "stray.txt"), "w").write("x")
    if mode == "commit":
        open(os.path.join(root, "out.md"), "w").write("x")
        subprocess.run(["git", "-C", root, "add", "out.md"], check=True)
        subprocess.run(["git", "-C", root, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "x"], check=True)
    if mode == "clean":
        open(os.path.join(root, "out.md"), "w").write("result\\n")
    if mode == "fail":
        sys.exit(3)
    if mode in ("extra",):
        open(os.path.join(root, "out.md"), "w").write("result\\n")
    time.sleep(0.5)
    emit({"type": "turn.completed"})
    """
)


def run(cwd, *args):
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True, text=True).stdout


class WorkerTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.repo = root / "corenius"
        self.repo.mkdir()
        run(self.repo, "init", "-q", "-b", "main")
        run(self.repo, "config", "user.email", "a@b")
        run(self.repo, "config", "user.name", "a")
        (self.repo / "README").write_text("hi\n")
        run(self.repo, "add", "README")
        run(self.repo, "commit", "-qm", "init")
        self.fake = root / "traecli"
        self.fake.write_text(FAKE)
        self.fake.chmod(self.fake.stat().st_mode | stat.S_IXUSR)
        self.task = root / "task.md"
        self.task.write_text("Write out.md.\n")
        os.environ["CORENIUS_TRAECLI"] = str(self.fake)
        os.environ["CORENIUS_RUNS"] = str(root / "runs")
        self.name = 0

    def tearDown(self):
        os.environ.pop("CORENIUS_TRAECLI", None)
        os.environ.pop("CORENIUS_RUNS", None)
        os.environ.pop("FAKE_MODE", None)
        self.tmp.cleanup()

    def launch(self, mode, *extra):
        os.environ["FAKE_MODE"] = mode
        self.name += 1
        out = self._main(
            ["launch", "--task", str(self.task), "--name", f"t{self.name}", "--allow", "out.md",
             "--repo", str(self.repo), "--stall", "1", *extra]
        )
        return json.loads(out)

    def _main(self, argv):
        import io
        from contextlib import redirect_stdout

        buffer = io.StringIO()
        with redirect_stdout(buffer):
            code = worker.main(argv)
        self.last_code = code
        return buffer.getvalue()

    def supervise(self, launched, **kw):
        out = self._main(["supervise", launched["run_dir"], "--interval", "0.2",
                          "--max-wait", str(kw.get("max_wait", 20))])
        return json.loads(out.strip().splitlines()[-1])

    def test_clean_run(self):
        launched = self.launch("clean")
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertEqual(self.last_code, worker.CLEAN)
        self.assertEqual(verdict["exit_code"], 0)
        self.assertTrue((Path(launched["run_dir"]) / "verdict.json").exists())
        self.assertIn("out.md", (Path(launched["worktree"]) / "out.md").name)
        prompt = (Path(launched["run_dir"]) / "prompt.md").read_text()
        self.assertIn("Write out.md.", prompt)
        self.assertIn("out.md", prompt.split("TASK")[0])

    def test_unexpected_file_is_a_violation(self):
        verdict = self.supervise(self.launch("extra"))
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertEqual(self.last_code, worker.VIOLATED)
        self.assertIn("stray.txt", json.dumps(verdict["findings"]))

    def test_commit_is_a_violation(self):
        verdict = self.supervise(self.launch("commit"))
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertIn("worker-committed", json.dumps(verdict["findings"]))

    def test_forbidden_command_is_a_violation(self):
        verdict = self.supervise(self.launch("forbidden"))
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertIn("curl", json.dumps(verdict["findings"]))

    def test_read_only_git_is_allowed(self):
        verdict = self.supervise(self.launch("clean"))
        self.assertNotIn("forbidden-command", json.dumps(verdict["findings"]))

    def test_protected_repo_change_is_a_violation(self):
        other = Path(self.tmp.name) / "other"
        other.mkdir()
        run(other, "init", "-q", "-b", "main")
        run(other, "config", "user.email", "a@b")
        run(other, "config", "user.name", "a")
        (other / "f").write_text("1\n")
        run(other, "add", "f")
        run(other, "commit", "-qm", "1")
        launched = self.launch("clean", "--protect", str(other))
        (other / "f").write_text("2\n")
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertIn("protected-dirty-changed", json.dumps(verdict["findings"]))

    def test_nonzero_exit_is_degraded(self):
        verdict = self.supervise(self.launch("fail"))
        self.assertEqual(verdict["verdict"], "DEGRADED")
        self.assertEqual(verdict["exit_code"], 3)
        self.assertEqual(verdict["missing_allowed_outputs"], ["out.md"])

    def test_stall_is_reported_and_running_returns_3(self):
        launched = self.launch("stall")
        result = self.supervise(launched, max_wait=3)
        self.assertEqual(result["verdict"], "RUNNING")
        self.assertEqual(self.last_code, worker.RUNNING)
        self.assertIn("stall", json.dumps(result["findings"]))
        self.assertFalse((Path(launched["run_dir"]) / "verdict.json").exists())
        pid = json.loads((Path(launched["run_dir"]) / "run.json").read_text())["pid"]
        os.killpg(os.getpgid(pid), 15)

    def test_supervise_waits_for_process_not_for_output(self):
        # Output file appears immediately; the process runs 0.5s longer.
        launched = self.launch("clean")
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN")
        self.assertGreaterEqual(verdict["events"], 3)

    def test_existing_worktree_is_refused(self):
        self.launch("clean")
        os.environ["FAKE_MODE"] = "clean"
        self._main(["launch", "--task", str(self.task), "--name", "t1", "--repo", str(self.repo)])
        self.assertEqual(self.last_code, 64)

    def test_missing_task_file_is_a_clean_error(self):
        self._main(["launch", "--task", str(self.task) + ".nope", "--name", "x", "--repo", str(self.repo)])
        self.assertEqual(self.last_code, 64)
        self.assertFalse((self.repo.parent / "corenius-worker-x").exists())


if __name__ == "__main__":
    unittest.main()
