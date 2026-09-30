"""Every file of every task must be visible to git: an ignored task file passes locally and is missing after a clone."""

from __future__ import annotations

import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class TasksTrackedTest(unittest.TestCase):
    def test_no_task_file_is_ignored_by_gitignore(self):
        out = subprocess.run(["git", "-C", str(ROOT), "ls-files", "--others", "--ignored", "--exclude-standard",
                              "bench/tasks", "bench/insula"], capture_output=True, text=True)
        self.assertEqual(out.returncode, 0, out.stderr)
        ignored = [p for p in out.stdout.split("\n") if p]
        self.assertEqual(ignored, [], "these task files are ignored by .gitignore and would be lost on clone: %s" % ignored)

    def test_every_task_has_the_required_files(self):
        for task in sorted((ROOT / "bench" / "tasks").iterdir()):
            for rel in ("task.toml", "instruction.md", "environment", "tests", "solution/solve.sh"):
                self.assertTrue((task / rel).exists(), f"{task.name}: missing {rel}")


if __name__ == "__main__":
    unittest.main()
