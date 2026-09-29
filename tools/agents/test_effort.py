"""effort.py against a throwaway git repo: the dehydrate/rehydrate round trip."""

from __future__ import annotations

import io
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import effort  # noqa: E402


def run(cwd, *args):
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True, text=True).stdout


class EffortTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.repo = root / "proj"
        self.repo.mkdir()
        run(self.repo, "init", "-q", "-b", "main")
        run(self.repo, "config", "user.email", "a@b")
        run(self.repo, "config", "user.name", "a")
        (self.repo / ".gitignore").write_text(".scratch/\n")
        (self.repo / "README").write_text("hi\n")
        run(self.repo, "add", ".")
        run(self.repo, "commit", "-qm", "init")
        launched = self.call(["--repo", str(self.repo), "new", "demo"])
        self.wt = Path(json.loads(launched)["worktree"])
        run(self.wt, "config", "user.email", "a@b")
        run(self.wt, "config", "user.name", "a")

    def tearDown(self):
        self.tmp.cleanup()

    def call(self, argv):
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            self.code = effort.main(argv)
        self.err = err.getvalue()
        return out.getvalue()

    def fill_state(self):
        state = self.wt / "docs/efforts/demo/STATE.md"
        text = state.read_text()
        text = text.replace("## Where we are\n\n_TODO_", "## Where we are\n\nStep one done.")
        text = text.replace("## Next actions\n\n_TODO_", "## Next actions\n\n1. Do step two.")
        state.write_text(text)

    def second_worktree(self):
        other = self.repo.parent / "proj-second"
        run(self.repo, "worktree", "add", "-q", "-b", "work/second", str(other), "main")
        (other / "f.txt").write_text("x\n")
        run(other, "add", "f.txt")
        run(other, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "work")
        return other

    def test_new_creates_durable_and_scratch(self):
        self.assertTrue((self.wt / "docs/efforts/demo/EFFORT.md").exists())
        self.assertTrue((self.repo / ".scratch/demo").is_dir())

    def test_dehydrate_requires_human_part(self):
        self.call(["--repo", str(self.wt), "dehydrate", "demo"])
        self.assertEqual(self.code, effort.ERROR)
        self.assertIn("Where we are", self.err)

    def test_dirty_worktree_blocks_dehydrate(self):
        self.fill_state()
        other = self.second_worktree()
        (other / "f.txt").write_text("dirty\n")
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--worktree", str(other)])
        self.assertEqual(self.code, effort.ERROR)
        self.assertIn("uncommitted", self.err)

    def test_round_trip_restores_worktree_and_briefs(self):
        self.fill_state()
        other = self.second_worktree()
        head = run(other, "rev-parse", "HEAD").strip()
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--worktree", str(other)])
        self.assertEqual(self.code, effort.OK, self.err)
        self.assertEqual(run(self.wt, "status", "--porcelain").strip(), "")
        # Lose the worktree and the scratch.
        run(self.repo, "worktree", "remove", "--force", str(other))
        shutil.rmtree(self.repo / ".scratch")
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertEqual(self.code, effort.OK, out)
        self.assertEqual(run(other, "rev-parse", "HEAD").strip(), head)
        self.assertIn("Do step two", out)
        self.assertIn("restored worktree", out)
        self.assertTrue((self.repo / ".scratch/demo").is_dir())

    def test_missing_branch_is_recreated_from_sha(self):
        self.fill_state()
        other = self.second_worktree()
        head = run(other, "rev-parse", "HEAD").strip()
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--worktree", str(other)])
        run(self.repo, "worktree", "remove", "--force", str(other))
        run(self.repo, "branch", "-D", "work/second")
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertIn("recreated branch", out)
        self.assertEqual(run(other, "rev-parse", "HEAD").strip(), head)

    def test_diverged_branch_is_an_error(self):
        self.fill_state()
        other = self.second_worktree()
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--worktree", str(other)])
        run(self.repo, "worktree", "remove", "--force", str(other))
        run(self.repo, "branch", "-f", "work/second", "main")
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertEqual(self.code, effort.ERROR)
        self.assertIn("diverged", out)

    def test_advanced_branch_is_a_warning(self):
        self.fill_state()
        other = self.second_worktree()
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--worktree", str(other)])
        (other / "g.txt").write_text("y\n")
        run(other, "add", "g.txt")
        run(other, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", "more")
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertEqual(self.code, effort.WARN, out)

    def test_rewritten_base_is_an_error(self):
        self.fill_state()
        self.call(["--repo", str(self.wt), "dehydrate", "demo"])
        run(self.repo, "commit", "-q", "--amend", "-m", "rewritten")
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertEqual(self.code, effort.ERROR)
        self.assertIn("base commit", out)

    def test_lost_run_logs_keep_verdict(self):
        self.fill_state()
        run_dir = Path(self.tmp.name) / "runs" / "r1"
        run_dir.mkdir(parents=True)
        (run_dir / "verdict.json").write_text(json.dumps({"verdict": "CLEAN", "exit_code": 0}))
        self.call(["--repo", str(self.wt), "dehydrate", "demo", "--run", str(run_dir)])
        shutil.rmtree(run_dir)
        out = self.call(["--repo", str(self.wt), "rehydrate", "demo"])
        self.assertEqual(self.code, effort.WARN)
        self.assertIn("logs lost, recorded verdict CLEAN", out)

    def test_decisions_are_append_only(self):
        self.call(["--repo", str(self.wt), "decide", "demo", "Use tracked efforts"])
        self.assertEqual(self.code, effort.OK)
        self.call(["--repo", str(self.wt), "decide", "demo", "Use tracked efforts"])
        self.assertEqual(self.code, 64)

    def test_status_lists_effort(self):
        out = self.call(["--repo", str(self.wt), "status"])
        self.assertIn('"effort": "demo"', out)
        self.assertIn("never dehydrated", out)


if __name__ == "__main__":
    unittest.main()
