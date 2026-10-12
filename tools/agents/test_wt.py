"""wt.py against throwaway git repos only."""

from __future__ import annotations

import argparse
import io
import json
import subprocess
import sys
import tempfile
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))
import wt  # noqa: E402

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


def run(cwd, *args, check=True):
    return subprocess.run(
        ["git", "-C", str(cwd), *args],
        check=check,
        capture_output=True,
        text=True,
    )


class WtTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.repo = root / "proj"
        self.repo.mkdir()
        run(self.repo, "init", "-q", "-b", "main")
        run(self.repo, "config", "user.email", "a@b")
        run(self.repo, "config", "user.name", "a")
        (self.repo / "README").write_text("hi\n")
        run(self.repo, "add", "README")
        run(self.repo, "commit", "-qm", "init")
        self.initial = run(self.repo, "rev-parse", "HEAD").stdout.strip()

    def tearDown(self):
        self.tmp.cleanup()

    def call(self, argv):
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            self.code = wt.main(argv)
        self.err = err.getvalue()
        return out.getvalue()

    def new_worktree(self, topic: str, agent: str = "worker") -> tuple[Path, str]:
        out = self.call(["new", "--agent", agent, "--topic", topic, "--repo", str(self.repo)])
        self.assertEqual(self.code, wt.OK, self.err)
        payload = json.loads(out)
        return Path(payload["path"]), payload["branch"]

    def commit_file(self, repo: Path, name: str, text: str, message: str) -> str:
        (repo / name).write_text(text)
        run(repo, "add", name)
        run(repo, "-c", "user.email=a@b", "-c", "user.name=a", "commit", "-qm", message)
        return run(repo, "rev-parse", "HEAD").stdout.strip()

    def set_created(self, worktree: Path, created: str) -> None:
        path = wt.metadata_path(worktree)
        data = json.loads(path.read_text())
        data["created"] = created
        path.write_text(json.dumps(data, indent=2, sort_keys=True) + "\n")

    def branch_exists(self, branch: str) -> bool:
        return bool(run(self.repo, "rev-parse", "--verify", "-q", f"refs/heads/{branch}", check=False).stdout.strip())

    def assert_one_line_refusal(self):
        self.assertEqual(self.code, wt.REFUSED)
        self.assertEqual(len(self.err.strip().splitlines()), 1)

    def test_new_creates_branch_directory_and_metadata(self):
        out = self.call(["new", "--agent", "worker", "--topic", "demo", "--repo", str(self.repo)])
        self.assertEqual(self.code, wt.OK, self.err)
        payload = json.loads(out)
        created = Path(payload["path"])
        self.assertEqual(payload["branch"], "worker/demo")
        self.assertEqual(payload["base_sha"], self.initial)
        self.assertTrue(created.is_dir())
        self.assertEqual(run(created, "rev-parse", "--abbrev-ref", "HEAD").stdout.strip(), "worker/demo")
        metadata = json.loads(wt.metadata_path(created).read_text())
        self.assertEqual(metadata["agent"], "worker")
        self.assertEqual(metadata["topic"], "demo")
        self.assertEqual(metadata["base_sha"], self.initial)
        self.assertRegex(metadata["created"], r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$")

    def test_new_refuses_existing_branch(self):
        run(self.repo, "branch", "worker/taken")
        self.call(["new", "--agent", "worker", "--topic", "taken", "--repo", str(self.repo)])
        self.assert_one_line_refusal()
        self.assertIn("already exists", self.err)

    def test_new_refuses_existing_directory(self):
        target = self.repo.parent / "proj-worker-taken"
        target.mkdir()
        self.call(["new", "--agent", "worker", "--topic", "taken", "--repo", str(self.repo)])
        self.assert_one_line_refusal()
        self.assertIn(str(target), self.err)

    def test_status_reports_divergence_dirty_owner_age_and_json_shape(self):
        worktree, branch = self.new_worktree("status")
        self.commit_file(worktree, "feature.txt", "feature\n", "feature")
        self.commit_file(self.repo, "main.txt", "main\n", "main")
        (worktree / "dirty.txt").write_text("dirty\n")

        manual = self.repo.parent / "proj-manual"
        run(self.repo, "worktree", "add", "-q", "-b", "manual/topic", str(manual), "main")

        rows = json.loads(self.call(["status", "--json", "--repo", str(self.repo)]))
        self.assertEqual(self.code, wt.OK, self.err)
        row = next(r for r in rows if r["branch"] == branch)
        self.assertTrue({"path", "branch", "ahead", "behind", "dirty", "owner", "age", "created"} <= set(row))
        self.assertEqual(row["ahead"], 1)
        self.assertEqual(row["behind"], 1)
        self.assertEqual(row["dirty"], 1)
        self.assertEqual(row["owner"], "worker")
        self.assertIsInstance(row["age"], int)
        fallback = next(r for r in rows if r["branch"] == "manual/topic")
        self.assertEqual(fallback["owner"], "manual")

    def test_finish_removes_worktree_when_run_from_inside_it(self):
        worktree, branch = self.new_worktree("done")
        result = subprocess.run(
            [sys.executable, str(Path(__file__).with_name("wt.py")), "finish", branch],
            cwd=worktree,
            capture_output=True,
            text=True,
        )
        self.assertEqual(result.returncode, wt.OK, result.stderr)
        self.assertFalse(worktree.exists())
        self.assertFalse(self.branch_exists(branch))

    def test_finish_refuses_unmerged_branch(self):
        worktree, branch = self.new_worktree("unmerged")
        self.commit_file(worktree, "feature.txt", "feature\n", "feature")
        self.call(["finish", branch, "--repo", str(self.repo)])
        self.assert_one_line_refusal()
        self.assertIn("not merged", self.err)
        self.assertTrue(worktree.exists())
        self.assertTrue(self.branch_exists(branch))

    def test_finish_refuses_dirty_worktree(self):
        worktree, branch = self.new_worktree("dirty")
        (worktree / "dirty.txt").write_text("dirty\n")
        self.call(["finish", branch, "--repo", str(self.repo)])
        self.assert_one_line_refusal()
        self.assertIn("dirty", self.err)
        self.assertTrue(worktree.exists())
        self.assertTrue(self.branch_exists(branch))

    def test_gc_is_dry_by_default_and_apply_removes_only_safe_trees(self):
        merged, merged_branch = self.new_worktree("merged")
        old, old_branch = self.new_worktree("old")
        dirty, dirty_branch = self.new_worktree("dirty-gc")
        ahead, ahead_branch = self.new_worktree("ahead")
        self.set_created(old, "2000-01-01T00:00:00Z")
        (dirty / "dirty.txt").write_text("dirty\n")
        self.commit_file(ahead, "ahead.txt", "ahead\n", "ahead")

        dry_rows = [json.loads(line) for line in self.call(["gc", "--repo", str(self.repo)]).splitlines()]
        self.assertEqual(self.code, wt.OK, self.err)
        dry_branches = {row["branch"] for row in dry_rows}
        self.assertEqual(dry_branches, set())
        self.assertTrue(all(row["removed"] is False for row in dry_rows))
        self.assertTrue(merged.exists())
        self.assertTrue(old.exists())

        apply_rows = [json.loads(line) for line in self.call(["gc", "--apply", "--repo", str(self.repo)]).splitlines()]
        self.assertEqual(self.code, wt.OK, self.err)
        self.assertEqual({row["branch"] for row in apply_rows}, set())
        self.assertTrue(all(row["removed"] is True for row in apply_rows))
        self.assertTrue(merged.exists())
        self.assertTrue(old.exists())
        self.assertTrue(self.branch_exists(merged_branch))
        self.assertTrue(self.branch_exists(old_branch))
        self.assertTrue(dirty.exists())
        self.assertTrue(ahead.exists())
        self.assertTrue(self.branch_exists(dirty_branch))
        self.assertTrue(self.branch_exists(ahead_branch))

    def test_usage_errors_exit_64(self):
        self.call(["new", "--agent", "worker", "--repo", str(self.repo)])
        self.assertEqual(self.code, wt.USAGE)

    def test_load_plugins_ignores_missing_land_module(self):
        parser = argparse.ArgumentParser()
        sub = parser.add_subparsers()
        error = ModuleNotFoundError("No module named 'land'", name="land")
        with mock.patch("wt.importlib.import_module", side_effect=error):
            wt._load_plugins(sub)

    def test_load_plugins_surfaces_broken_land_imports(self):
        parser = argparse.ArgumentParser()
        sub = parser.add_subparsers()
        error = ModuleNotFoundError("No module named 'missing_dep'", name="missing_dep")
        with mock.patch("wt.importlib.import_module", side_effect=error):
            with self.assertRaises(ModuleNotFoundError):
                wt._load_plugins(sub)

    def test_load_plugins_registers_fake_land_subcommand(self):
        parser = argparse.ArgumentParser()
        sub = parser.add_subparsers(dest="cmd")

        class FakeLand:
            @staticmethod
            def run(args):
                return 17

            @staticmethod
            def add_parser(subparsers):
                p = subparsers.add_parser("land")
                p.set_defaults(func=FakeLand.run)

        with mock.patch("wt.importlib.import_module", return_value=FakeLand):
            wt._load_plugins(sub)
        args = parser.parse_args(["land"])
        self.assertEqual(args.func(args), 17)


if __name__ == "__main__":
    unittest.main()
