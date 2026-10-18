"""Cleanup proof at the collab and compatibility CLI seams, using real Git repositories."""
import json
import os
from pathlib import Path
import subprocess
import shutil
import sys
import unittest
import test_collab as tc

CLI = tc.CLI

WT = CLI.parents[1] / "agents" / "wt.py"


class CleanupTests(unittest.TestCase):
    setUp = tc.CollabTests.setUp
    run_process = tc.CollabTests.run_process
    git = tc.CollabTests.git
    commit = tc.CollabTests.commit
    cli = tc.CollabTests.cli
    init = tc.CollabTests.init
    ticket = tc.CollabTests.ticket

    def terminal(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        attempt = self.cli("open-attempt", "T-one", "--owner", "worker")
        self.cli("abandon", attempt["id"], "--confirm-stopped")
        self.git("worktree", "unlock", attempt["path"])
        return attempt, Path(attempt["path"])

    def wt(self, *args):
        return self.run_process([sys.executable, str(WT), *args, "--repo", str(self.repo)])

    def test_dirty_archive_restores_index_files_modes_and_symlinks(self):
        attempt, path = self.terminal()
        (path / "code.txt").write_text("staged\n")
        self.git("add", "code.txt", cwd=path)
        (path / "code.txt").write_text("unstaged\n")
        (path / "new.txt").write_text("untracked\n")
        (path / "build").mkdir()
        (path / "build" / "ignored").write_bytes(b"ignored\x00bytes")
        (path / "executable").write_text("executable\n")
        (path / "executable").chmod(0o755)
        (path / "link").symlink_to(self.repo / "code.txt")
        expected_status = self.git("status", "--porcelain", "--ignored", cwd=path)
        expected_index = self.git("show", ":code.txt", cwd=path)
        skipped = self.cli("sweep", "--apply")
        self.assertTrue(path.exists())
        self.assertIn("not clean", str(skipped["skipped"]))
        dry = self.cli("sweep", "--archive-dirty")
        self.assertTrue(path.exists())
        self.assertEqual(len(dry["candidates"]), 1)
        result = self.cli("sweep", "--apply", "--archive-dirty")
        self.assertFalse(path.exists())
        archive = result["archived"][0]["archive"]
        restored = self.root / "restored"
        self.cli("restore-archive", archive, str(restored))
        self.assertEqual(self.git("status", "--porcelain", "--ignored", cwd=restored), expected_status)
        self.assertEqual(self.git("show", ":code.txt", cwd=restored), expected_index)
        self.assertEqual((restored / "code.txt").read_text(), "unstaged\n")
        self.assertEqual((restored / "new.txt").read_text(), "untracked\n")
        self.assertEqual((restored / "build" / "ignored").read_bytes(), b"ignored\x00bytes")
        self.assertTrue((restored / "executable").stat().st_mode & 0o111)
        self.assertTrue((restored / "link").is_symlink())
        self.assertEqual(os.readlink(restored / "link"), str(self.repo / "code.txt"))
        self.assertEqual((self.repo / "code.txt").read_text(), "base\n")

    def test_foreign_unmerged_branch_requires_flag_and_is_archived(self):
        self.init()
        self.git("checkout", "-b", "human/valuable")
        (self.repo / "valuable").write_text("keep me\n")
        head = self.commit("valuable")
        self.git("checkout", "main")
        result = self.cli("sweep", "--apply")
        self.assertIn("human", str(result["skipped"]))
        self.assertEqual(self.git("rev-parse", "human/valuable"), head)
        result = self.cli("sweep", "--apply", "--include-foreign")
        self.assertEqual(len(result["archived"]), 1)
        ref = result["archived"][0]["ref"]
        self.assertEqual(self.git("show", ref + ":valuable"), "keep me")
        self.assertEqual(self.git("rev-parse", "--verify", "human/valuable", ok=False), "")

    def test_retention_prunes_old_terminal_artifacts_and_preserves_active_and_young(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        attempt = self.cli("open-attempt", "T-one", "--owner", "worker")
        path = Path(attempt["path"])
        (path / "code.txt").write_text("change\n")
        self.commit("change", cwd=path)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        first_log = Path(checked["log"])
        first_path = Path(checked["check_path"])
        self.git("worktree", "unlock", str(first_path))
        second = self.cli("check", attempt["id"])
        home = self.repo / ".git" / "collab"
        old_time = 946684800
        os.utime(first_log, (old_time, old_time))
        os.utime(first_path, (old_time, old_time))
        tag = first_path.name.rsplit("-", 1)[1]
        first_ref = "refs/collab/checks/" + attempt["id"] + "/" + tag
        os.utime(self.repo / ".git" / first_ref, (old_time, old_time))
        self.cli("sweep", "--apply", "--retention-days", "30")
        self.assertTrue(first_log.exists())
        self.assertTrue(first_path.exists())
        self.cli("abandon", attempt["id"], "--confirm-stopped")
        dry = self.cli("sweep", "--retention-days", "30")
        self.assertTrue(first_log.exists())
        self.assertTrue(first_path.exists())
        self.assertIn(str(first_log), str(dry["candidates"]))
        self.cli("sweep", "--apply", "--retention-days", "30")
        self.assertFalse(first_log.exists())
        self.assertFalse(first_path.exists())
        self.assertTrue(Path(second["log"]).exists())
        self.assertTrue(Path(second["check_path"]).exists())
        self.assertEqual(self.git("rev-parse", "--verify", first_ref, ok=False), "")

    def test_sweep_and_retention_retain_terminal_attempt_with_unknown_worker_evidence(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        run = self.root / "run"
        run.mkdir()
        (run / "run.json").write_text(json.dumps({"pid": 2147483647}))
        attempt = self.cli("open-attempt", "T-one", "--owner", "worker", "--worker-run-dir", str(run))
        path = Path(attempt["path"])
        (path / "code.txt").write_text("change\n")
        self.commit("change", cwd=path)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        log = Path(checked["log"])
        check_path = Path(checked["check_path"])
        tag = check_path.name.rsplit("-", 1)[1]
        ref = "refs/collab/checks/" + attempt["id"] + "/" + tag
        old_time = 946684800
        os.utime(log, (old_time, old_time))
        os.utime(check_path, (old_time, old_time))
        os.utime(self.repo / ".git" / ref, (old_time, old_time))
        self.cli("land", attempt["id"], "--no-push")
        (run / "run.json").write_text("{}")

        result = self.cli("sweep", "--apply", "--retention-days", "0")
        self.assertIn("state unknown", str(result["skipped"]))
        self.assertIn("unknown Attempt still needs this evidence", str(result["skipped"]))
        self.assertTrue(path.exists())
        self.assertTrue(log.exists())
        self.assertTrue(check_path.exists())
        self.assertEqual(self.git("rev-parse", "--verify", ref), checked["integration"])

    def test_shim_fresh_gc_metadata_corruption_and_deprecation(self):
        result = self.wt("new", "--agent", "worker", "--topic", "fresh")
        self.assertEqual(result.returncode, 0, result.stderr)
        path = Path(json.loads(result.stdout)["path"])
        result = self.wt("gc", "--apply")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(path.exists())
        gitdir = Path(self.git("rev-parse", "--path-format=absolute", "--git-dir", cwd=path))
        (gitdir / "agent-worktree.json").write_text("{broken")
        result = self.wt("status", "--json")
        self.assertEqual(result.returncode, 1)
        self.assertIn("metadata", result.stderr)
        help_result = self.run_process([sys.executable, str(WT), "--help"])
        self.assertIn("deprecated", help_result.stdout.lower())

    def test_shim_creation_serializes_and_status_uses_configured_remote_mainline(self):
        argv = [sys.executable, str(WT), "new", "--agent", "worker", "--topic", "same", "--repo", str(self.repo)]
        first = subprocess.Popen(argv, cwd=self.repo, env=self.env, stdout=subprocess.PIPE,
                                 stderr=subprocess.PIPE, text=True)
        second = subprocess.Popen(argv, cwd=self.repo, env=self.env, stdout=subprocess.PIPE,
                                  stderr=subprocess.PIPE, text=True)
        one = first.communicate(timeout=10)
        two = second.communicate(timeout=10)
        self.assertEqual(sorted([first.returncode, second.returncode]), [0, 1], str((one, two)))
        self.git("branch", "configured")
        self.git("config", "collab.mainline", "configured")
        remote = self.root / "remote.git"
        self.git("init", "--bare", str(remote))
        self.git("remote", "add", "origin", str(remote))
        self.git("push", "origin", "configured")
        self.git("checkout", "configured")
        (self.repo / "remote-only").write_text("remote\n")
        ahead = self.commit("remote ahead")
        self.git("push", "origin", "configured")
        self.git("checkout", "main")
        # Point local mainline back without rewriting any shared history: test-owned branch only.
        self.git("update-ref", "refs/heads/configured", "main", ahead)
        result = self.wt("status", "--json")
        self.assertEqual(result.returncode, 0, result.stderr)
        row = next(row for row in json.loads(result.stdout) if row["branch"] == "worker/same")
        self.assertEqual(row["behind"], 0)
        self.assertEqual(row["remote_behind"], 1)
        self.assertEqual(row["remote_mainline"], "refs/remotes/origin/configured")
        explicit = self.wt("status", "--json", "--mainline", "origin/configured")
        self.assertEqual(explicit.returncode, 0, explicit.stderr)
        self.assertEqual(next(row for row in json.loads(explicit.stdout)
                              if row["branch"] == "worker/same")["behind"], 1)

    def test_locked_dirty_and_foreign_checkouts_are_retained(self):
        attempt, path = self.terminal()
        (path / "new").write_text("dirty\n")
        self.git("worktree", "lock", "--reason", "external owner", str(path))
        foreign = self.root / "foreign"
        self.git("worktree", "add", "-b", "human/active", str(foreign))
        result = self.cli("sweep", "--apply", "--archive-dirty", "--include-foreign", "--retention-days", "0")
        self.assertTrue(path.exists())
        self.assertTrue(foreign.exists())
        self.assertIn("external owner", str(result["skipped"]))
        self.assertIn("unknown worker identity", str(result["skipped"]))
        self.assertEqual(result["archived"], [])

    def test_archive_tampering_refuses_restore_and_nested_git_refuses_cleanup(self):
        attempt, path = self.terminal()
        (path / "new").write_text("dirty\n")
        nested = path / "private"
        nested.mkdir()
        self.git("init", cwd=nested)
        failed = self.cli("sweep", "--apply", "--archive-dirty", ok=False)
        self.assertIn("Nested Git", failed.stderr)
        self.assertTrue(path.exists())
        shutil.rmtree(nested)
        result = self.cli("sweep", "--apply", "--archive-dirty")
        archive = Path(result["archived"][0]["archive"])
        with (archive / "files.tar").open("ab") as stream:
            stream.write(b"tampered")
        result = self.cli("restore-archive", str(archive), str(self.root / "restore-tampered"), ok=False)
        self.assertIn("verification", result.stderr)
        self.assertFalse((self.root / "restore-tampered").exists())

    def test_retention_removes_only_known_old_missing_registration(self):
        self.init()
        self.ticket()
        self.commit("ticket")
        attempt = self.cli("open-attempt", "T-one", "--owner", "worker")
        path = Path(attempt["path"])
        (path / "code.txt").write_text("change\n")
        self.commit("change", cwd=path)
        self.cli("submit", attempt["id"])
        checked = self.cli("check", attempt["id"])
        check_path = Path(checked["check_path"])
        registration = Path(self.git("rev-parse", "--path-format=absolute", "--git-dir", cwd=check_path))
        self.cli("abandon", attempt["id"], "--confirm-stopped")
        self.git("worktree", "unlock", str(check_path))
        shutil.rmtree(check_path)
        os.utime(registration, (946684800, 946684800))
        result = self.cli("sweep", "--apply")
        self.assertIn("prune_registration", str(result["removed"]))
        self.assertNotIn(str(check_path), self.git("worktree", "list", "--porcelain"))

    def test_archive_storage_symlink_does_not_write_an_unrelated_tree(self):
        attempt, path = self.terminal()
        (path / "new").write_text("dirty\n")
        unrelated = self.root / "unrelated"
        unrelated.mkdir()
        (self.repo / ".git" / "collab" / "archives").symlink_to(unrelated, target_is_directory=True)
        result = self.cli("sweep", "--apply", "--archive-dirty", ok=False)
        self.assertIn("archive", result.stderr.lower())
        self.assertTrue(path.exists())
        self.assertEqual(list(unrelated.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
