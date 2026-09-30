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
    if mode in ("rm_scratch", "rm_home", "rm_parent", "rm_tracked"):
        cmd = {
            "rm_scratch": 'bash -lc "mkdir -p scratch && rm -rf scratch"',
            "rm_home": "rm -rf ~/somewhere",
            "rm_parent": "rm -rf ../other",
            "rm_tracked": "rm -f README",
        }[mode]
        emit({"type": "item.completed", "item": {"id": "i2", "type": "command_execution", "command": cmd}})
        if mode == "rm_tracked":
            os.remove(os.path.join(root, "README"))
    if mode == "extra":
        open(os.path.join(root, "stray.txt"), "w").write("x")
    if mode == "commit":
        open(os.path.join(root, "out.md"), "w").write("x")
        subprocess.run(["git", "-C", root, "add", "out.md"], check=True)
        subprocess.run(["git", "-C", root, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "x"], check=True)
    if mode in ("clean", "rm_scratch", "rm_home", "rm_parent", "rm_tracked"):
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
        os.environ.pop("CORENIUS_CAPSULES", None)
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

    def test_rm_inside_worktree_is_a_note_not_a_violation(self):
        verdict = self.supervise(self.launch("rm_scratch"))
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertEqual([f["kind"] for f in verdict["findings"]], ["rm-recursive"])
        self.assertEqual(verdict["findings"][0]["severity"], "NOTE")

    def test_rm_of_home_or_parent_is_a_violation(self):
        for mode in ("rm_home", "rm_parent"):
            verdict = self.supervise(self.launch(mode))
            self.assertEqual(verdict["verdict"], "VIOLATED", (mode, verdict))
            self.assertIn("rm-outside-worktree", json.dumps(verdict["findings"]))

    def test_rm_of_tracked_file_is_caught_by_the_diff(self):
        verdict = self.supervise(self.launch("rm_tracked"))
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertIn("unexpected-change", json.dumps(verdict["findings"]))

    def test_old_run_with_recorded_rm_regex_is_rejudged(self):
        launched = self.launch("rm_scratch")
        run_json = Path(launched["run_dir"]) / "run.json"
        run = json.loads(run_json.read_text())
        run["forbidden"].append(worker.LEGACY_RM)
        run_json.write_text(json.dumps(run))
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)

    def test_rerun_keeps_the_previous_verdict(self):
        launched = self.launch("clean")
        self.supervise(launched)
        self.supervise(launched)
        self.assertTrue((Path(launched["run_dir"]) / "verdict.prev.json").exists())

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

    def make_other(self):
        other = Path(self.tmp.name) / "other"
        other.mkdir()
        run(other, "init", "-q", "-b", "main")
        run(other, "config", "user.email", "a@b")
        run(other, "config", "user.name", "a")
        (other / "f").write_text("1\n")
        run(other, "add", "f")
        run(other, "commit", "-qm", "1")
        return other

    def test_protected_head_fast_forward_is_a_note(self):
        other = self.make_other()
        launched = self.launch("clean", "--protect", str(other))
        (other / "g").write_text("2\n")
        run(other, "add", "g")
        run(other, "commit", "-qm", "2")  # the lead landing while the worker runs
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        self.assertIn("protected-head-advanced", json.dumps(verdict["findings"]))

    def test_protected_head_rewrite_is_a_violation(self):
        other = self.make_other()
        launched = self.launch("clean", "--protect", str(other))
        run(other, "commit", "-q", "--amend", "-m", "rewritten")
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertIn("protected-head-moved", json.dumps(verdict["findings"]))

    def capsule_request(self, mode="clean"):
        # The capsule clears the environment, so the fake's mode is baked into the script.
        tool = Path(self.tmp.name) / "fake-tool"
        (tool / "bin").mkdir(parents=True, exist_ok=True)
        (tool / "bin" / "traecli").write_text(self.fake.read_text().replace(
            'os.environ.get("FAKE_MODE", "clean")', repr(mode)))
        (tool / "bin" / "traecli").chmod(0o755)
        req = Path(self.tmp.name) / "capsule.json"
        req.write_text(json.dumps({"name": "w", "layers": [{"kind": "base"},
                                   {"kind": "tree", "name": "fake", "source": str(tool), "path": ["bin"]}],
                                   "policy": {"net": "host"}}))
        os.environ["CORENIUS_CAPSULES"] = str(Path(self.tmp.name) / "capsules")
        os.environ["CORENIUS_TRAECLI"] = "traecli"
        return str(req)

    def test_worker_runs_inside_a_capsule(self):
        launched = self.launch("clean", "--capsule", self.capsule_request())
        verdict = self.supervise(launched)
        self.assertEqual(verdict["verdict"], "CLEAN", verdict)
        run_dir = Path(launched["run_dir"])
        self.assertEqual((Path(launched["worktree"]) / "out.md").read_text(), "result\n")
        prompt = (run_dir / "prompt.md").read_text()
        self.assertIn("/workspace", prompt)
        self.assertNotIn(launched["worktree"], prompt)
        record = json.loads((run_dir / "capsule-run.json").read_text())
        self.assertEqual((record["net"], record["exit_code"]), ("host", 0))
        self.assertEqual(json.loads((run_dir / "run.json").read_text())["sandbox"], "danger-full-access (capsule)")

    def test_capsule_worker_boundary_violation_is_still_detected(self):
        verdict = self.supervise(self.launch("extra", "--capsule", self.capsule_request("extra")))
        self.assertEqual(verdict["verdict"], "VIOLATED")
        self.assertIn("stray.txt", json.dumps(verdict["findings"]))

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

    # --- F12: --sandbox insula (read-only worktree, allowed paths bound writable) ---

    def test_sandbox_insula_needs_a_capsule(self):
        os.environ["FAKE_MODE"] = "clean"
        self._main(["launch", "--task", str(self.task), "--name", "s1", "--allow", "out.md",
                    "--repo", str(self.repo), "--sandbox", "insula"])
        self.assertEqual(self.last_code, 64)
        self.assertFalse((self.repo.parent / "corenius-worker-s1").exists())

    def test_sandbox_binds_precreate_missing_allowed_paths(self):
        tree = Path(self.tmp.name) / "tree"
        (tree / "tools").mkdir(parents=True)
        (tree / "README").write_text("x\n")
        binds, created = worker.sandbox_binds(tree, ["README", "tools/agents/new.py", "scratch/"])
        self.assertEqual([dest for _, dest in binds],
                         ["/workspace/README", "/workspace/tools/agents/new.py", "/workspace/scratch"])
        self.assertEqual(created, [tree / "tools" / "agents", tree / "tools" / "agents" / "new.py", tree / "scratch"])
        self.assertTrue((tree / "tools" / "agents" / "new.py").is_file())
        self.assertTrue((tree / "scratch").is_dir())
        self.assertEqual((tree / "README").read_text(), "x\n")
        with self.assertRaises(worker.WorkerError):
            worker.sandbox_binds(tree, ["../escape"])
        with self.assertRaises(worker.WorkerError):
            worker.sandbox_binds(tree, ["/abs"])

    def test_remove_untouched_keeps_what_the_worker_wrote(self):
        tree = Path(self.tmp.name) / "tree2"
        tree.mkdir()
        _, created = worker.sandbox_binds(tree, ["a/kept.py", "b/empty.py", "c/"])
        (tree / "a" / "kept.py").write_text("print(1)\n")
        removed = worker.remove_untouched(created)
        self.assertEqual(sorted(removed), sorted([str(tree / "b" / "empty.py"), str(tree / "b"), str(tree / "c")]))
        self.assertTrue((tree / "a" / "kept.py").exists())
        self.assertFalse((tree / "b").exists())
        self.assertFalse((tree / "c").exists())


if __name__ == "__main__":
    unittest.main()
