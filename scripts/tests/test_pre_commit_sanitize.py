"""Verify the optional hook checks index bytes without mutating or building."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

HOOK = Path(__file__).resolve().parents[1] / "pre-commit-sanitize.sh"


class StagedSanitizerTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.git("init", "-q")
        self.scanner = self.root / "scanner"
        self.scanner.write_text("""#!/usr/bin/env python3
import pathlib, sys
assert sys.argv[1:3] == ['sanitize', 'check']
raise SystemExit(b'rejected-index-marker' in pathlib.Path(sys.argv[3]).read_bytes())
""")
        self.scanner.chmod(0o700)
        self.env = dict(os.environ, HARP_BIN=str(self.scanner))

    def git(self, *args):
        subprocess.run(["git", *args], cwd=self.root, check=True, capture_output=True)

    def hook(self):
        return subprocess.run(["sh", str(HOOK)], cwd=self.root, env=self.env, capture_output=True)

    def test_staged_violation_is_not_hidden_by_clean_worktree(self):
        path = self.root / "note.txt"
        path.write_text("rejected-index-marker")
        self.git("add", "note.txt")
        path.write_text("clean worktree")
        result = self.hook()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"staged file", result.stderr)
        self.assertEqual(path.read_text(), "clean worktree")

    def test_unstaged_violation_does_not_change_clean_index_result(self):
        path = self.root / "note.txt"
        path.write_text("clean index")
        self.git("add", "note.txt")
        path.write_text("rejected-index-marker")
        self.assertEqual(self.hook().returncode, 0)
        self.assertEqual(path.read_text(), "rejected-index-marker")

    def test_spaces_newlines_and_unicode_are_checked_as_whole_paths(self):
        name = "space 雪\nand newline.txt"
        (self.root / name).write_text("rejected-index-marker")
        self.git("add", "--", name)
        self.assertNotEqual(self.hook().returncode, 0)

    def test_missing_binary_fails_without_an_implicit_build(self):
        (self.root / "note.txt").write_text("clean index")
        self.git("add", "note.txt")
        self.env["HARP_BIN"] = str(self.root / "missing")
        result = self.hook()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn(b"Build Harp explicitly", result.stderr)
        self.assertFalse((self.root / "target").exists())


if __name__ == "__main__":
    unittest.main()
