"""Behavioral checks for source-preserving WIP review artifacts."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from tools.agents import review_snapshot


SCRIPT = Path(__file__).with_name("review_snapshot.py")


class ReviewSnapshotTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.repo = self.root / "source"
        self.repo.mkdir()
        self.git("init", "-q")
        self.git("config", "user.name", "Test")
        self.git("config", "user.email", "test@example.invalid")
        (self.repo / "text").write_bytes(b"base\n")
        (self.repo / "deleted").write_bytes(b"gone\n")
        self.git("add", ".")
        self.git("commit", "-qm", "base")
        self.base = self.git("rev-parse", "HEAD").strip().decode()
        self.output = self.root / "artifact"

    def git(self, *args, repo=None, input=None):
        return subprocess.check_output(["git", "-C", str(repo or self.repo), *args], input=input)

    def run_snapshot(self, success=True):
        before_index = (self.repo / ".git/index").read_bytes()
        before_refs = self.git("show-ref", "--head")
        before_objects = sorted(p.relative_to(self.repo / ".git/objects") for p in (self.repo / ".git/objects").rglob("*"))
        result = subprocess.run(["python3", str(SCRIPT), "--repo", str(self.repo), "--base", self.base,
                                 "--output", str(self.output)], capture_output=True, timeout=20)
        self.assertEqual((self.repo / ".git/index").read_bytes(), before_index)
        self.assertEqual(self.git("show-ref", "--head"), before_refs)
        self.assertEqual(sorted(p.relative_to(self.repo / ".git/objects") for p in (self.repo / ".git/objects").rglob("*")), before_objects)
        self.assertEqual(result.returncode == 0, success, result.stderr.decode())
        if success:
            return json.loads((self.output / "manifest.json").read_text())
        self.assertFalse(self.output.exists())
        return result.stderr

    def blob(self, manifest, state, path):
        return self.git("show", manifest["trees"][state] + ":" + path, repo=self.output / "repository")

    def test_all_layers_bytes_modes_and_odd_names(self):
        (self.repo / "text").write_bytes(b"committed\n")
        self.git("commit", "-qam", "committed")
        (self.repo / "text").write_bytes(b"staged\n")
        self.git("add", "text")
        (self.repo / "text").write_bytes(b"unstaged\n")
        (self.repo / "deleted").unlink()
        odd = "untracked\t\n-\udcff.bin"
        (self.repo / odd).write_bytes(b"\x00\xff\x01binary\r\n")
        (self.repo / "executable").write_bytes(b"echo hello\n")
        (self.repo / "executable").chmod(0o755)
        (self.repo / "link").symlink_to("/not/a/readable/target")
        manifest = self.run_snapshot()
        for state, expected in [("base", b"base\n"), ("head", b"committed\n"), ("index", b"staged\n"), ("worktree", b"unstaged\n")]:
            self.assertEqual(self.blob(manifest, state, "text"), expected)
        self.assertEqual(self.blob(manifest, "worktree", odd), b"\x00\xff\x01binary\r\n")
        self.assertEqual(self.blob(manifest, "worktree", "link"), b"/not/a/readable/target")
        tree = self.git("ls-tree", "-rz", manifest["trees"]["worktree"], repo=self.output / "repository")
        self.assertIn(b"100755 blob", tree)
        self.assertIn(b"120000 blob", tree)
        self.assertNotIn(b"\tdeleted\0", tree)
        self.assertIn(b"GIT binary patch", (self.output / "unstaged.patch").read_bytes())
        self.assertFalse(manifest["landing_acceptance"])

    def test_fifo_refused_without_blocking(self):
        os.mkfifo(self.repo / "fifo")
        self.assertIn(b"unsupported", self.run_snapshot(False))

    def test_symlink_parent_refused_without_reading_target(self):
        (self.repo / "dir").mkdir()
        (self.repo / "dir/file").write_text("tracked")
        self.git("add", "dir/file")
        (self.repo / "dir/file").unlink()
        (self.repo / "dir").rmdir()
        (self.repo / "dir").symlink_to(self.root)
        (self.root / "file").write_text("outside secret")
        self.run_snapshot(False)

    def test_filters_and_hooks_are_not_executed(self):
        marker = self.root / "executed"
        self.git("config", "filter.evil.clean", f"touch {marker}")
        self.git("config", "diff.external", f"touch {marker}")
        (self.repo / ".gitattributes").write_text("* filter=evil diff=evil\n")
        (self.repo / "raw").write_bytes(b"raw\r\nbytes\x00")
        manifest = self.run_snapshot()
        self.assertFalse(marker.exists())
        self.assertEqual(self.blob(manifest, "worktree", "raw"), b"raw\r\nbytes\x00")

    def test_conflicted_index_refused(self):
        oid = self.git("rev-parse", "HEAD:text").strip()
        self.git("update-index", "--index-info", input=b"0 " + b"0" * 40 + b"\ttext\n100644 " + oid + b" 1\ttext\n")
        self.assertIn(b"unmerged", self.run_snapshot(False))

    def test_ignored_content_is_excluded_but_ignored_tracked_files_remain(self):
        (self.repo / ".gitignore").write_text("ignored/\ntext\n")
        (self.repo / "ignored").mkdir()
        os.mkfifo(self.repo / "ignored/fifo")
        (self.repo / "text").write_bytes(b"still tracked\n")
        manifest = self.run_snapshot()
        self.assertEqual(self.blob(manifest, "worktree", "text"), b"still tracked\n")
        self.assertNotIn("ignored/fifo", [entry["path"] for entry in manifest["files"]["worktree"]])

    def test_input_changed_during_publication_is_refused(self):
        original = review_snapshot.write_tree

        def mutate_then_write(*args, **kwargs):
            (self.repo / "text").write_bytes(b"changed during capture\n")
            return original(*args, **kwargs)

        before_index = (self.repo / ".git/index").read_bytes()
        with patch.object(review_snapshot, "write_tree", side_effect=mutate_then_write):
            with self.assertRaisesRegex(review_snapshot.SnapshotError, "input changed"):
                review_snapshot.snapshot(self.repo, self.base, self.output)
        self.assertFalse(self.output.exists())
        self.assertEqual(before_index, (self.repo / ".git/index").read_bytes())
        self.assertEqual([], list(self.root.glob(".review-snapshot-*")))

    def test_unsafe_paths_are_refused(self):
        for path in (b"../escape", b"/absolute", b"a/../../escape", b"a/.git/config", b"a//b"):
            with self.subTest(path=path), self.assertRaisesRegex(review_snapshot.SnapshotError, "unsafe"):
                review_snapshot.safe_path(path)

    def test_existing_output_is_preserved(self):
        self.output.mkdir()
        sentinel = self.output / "sentinel"
        sentinel.write_text("keep")
        with self.assertRaisesRegex(review_snapshot.SnapshotError, "must not exist"):
            review_snapshot.snapshot(self.repo, self.base, self.output)
        self.assertEqual("keep", sentinel.read_text())

    def test_source_output_is_refused(self):
        with self.assertRaisesRegex(review_snapshot.SnapshotError, "outside"):
            review_snapshot.snapshot(self.repo, self.base, self.repo / "artifact")
        self.assertFalse((self.repo / "artifact").exists())

    def test_file_directory_transitions(self):
        (self.repo / "dir").mkdir()
        (self.repo / "dir/file").write_bytes(b"old child")
        self.git("add", "dir/file")
        (self.repo / "dir/file").unlink()
        (self.repo / "dir").rmdir()
        (self.repo / "dir").write_bytes(b"now a file")
        (self.repo / "text").unlink()
        (self.repo / "text").mkdir()
        (self.repo / "text/child").write_bytes(b"new child")
        manifest = self.run_snapshot()
        self.assertEqual(self.blob(manifest, "worktree", "dir"), b"now a file")
        self.assertEqual(self.blob(manifest, "worktree", "text/child"), b"new child")

    def test_git_submodule_refused(self):
        self.git("update-index", "--add", "--cacheinfo", "160000," + self.base + ",submodule")
        self.assertIn(b"unsupported Git mode", self.run_snapshot(False))


if __name__ == "__main__":
    unittest.main()
