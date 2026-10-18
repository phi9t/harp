"""Sweep ownership races exercised through the public CLI and real Git processes."""
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import unittest

import test_collab as tc
import test_cleanup as cleanup


class SweepRepairTests(unittest.TestCase):
    setUp = tc.CollabTests.setUp
    run_process = tc.CollabTests.run_process
    git = tc.CollabTests.git
    commit = tc.CollabTests.commit
    cli = tc.CollabTests.cli
    init = tc.CollabTests.init
    ticket = tc.CollabTests.ticket
    terminal = cleanup.CleanupTests.terminal

    def test_destructive_sweep_refuses_while_a_check_process_holds_the_execution_lock(self):
        ready, release = self.root / "check-ready", self.root / "check-release"
        (self.repo / "gate.py").write_text(
            "from pathlib import Path\nimport time\n"
            f"Path({str(ready)!r}).touch()\n"
            "deadline = time.monotonic() + 20\n"
            f"while not Path({str(release)!r}).exists() and time.monotonic() < deadline: time.sleep(.01)\n"
        )
        self.commit("gate")
        self.init([sys.executable, "gate.py"])
        self.ticket("T-one")
        self.ticket("T-two")
        self.commit("tickets")
        checking = self.cli("open-attempt", "T-one", "--owner", "checker")
        retired = self.cli("open-attempt", "T-two", "--owner", "retired-worker")
        retired_path = Path(retired["path"])
        (retired_path / "new.txt").write_text("retained during check\n")
        self.cli("abandon", retired["id"], "--confirm-stopped")
        self.git("worktree", "unlock", str(retired_path))
        checking_path = Path(checking["path"])
        (checking_path / "code.txt").write_text("check contribution\n")
        self.commit("check contribution", cwd=checking_path)
        self.cli("submit", checking["id"])
        process = subprocess.Popen(
            [sys.executable, str(tc.CLI), "--repo", str(self.repo), "check", checking["id"]],
            cwd=self.repo, env=self.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
        )
        try:
            deadline = time.monotonic() + 10
            while not ready.exists() and process.poll() is None and time.monotonic() < deadline:
                time.sleep(.01)
            self.assertTrue(ready.exists(), "check did not reach its real gate command")
            result = self.cli("sweep", "--apply", "--archive-dirty", ok=False)
            self.assertIn("check operation busy", result.stderr)
            self.assertTrue(retired_path.is_dir())
            self.assertEqual((retired_path / "new.txt").read_text(), "retained during check\n")
            self.assertFalse((self.repo / ".git/collab/archives").exists())
        finally:
            release.touch()
            stdout, stderr = process.communicate(timeout=20)
            self.assertEqual(process.returncode, 0, str((stdout, stderr)))

    def test_lock_acquired_during_archive_preserves_dirty_worktree_and_staging(self):
        attempt, path = self.terminal()
        (path / "code.txt").write_text("staged owner bytes\n")
        self.git("add", "code.txt", cwd=path)
        (path / "code.txt").write_text("unsaved owner bytes\n")
        (path / "new.txt").write_bytes(b"untracked\x00owner bytes")
        (path / "build").mkdir()
        (path / "build" / "ignored").write_bytes(b"ignored\x00owner bytes")
        head = self.git("rev-parse", "HEAD", cwd=path)
        before_status = self.git("status", "--porcelain", "--ignored", cwd=path)
        ready = self.root / "archive-ready"
        release = self.root / "archive-release"
        shim_root = self.root / "shim"
        shim_root.mkdir()
        real_git = shutil.which("git", path=self.env.get("PATH"))
        self.assertIsNotNone(real_git)
        shim = shim_root / "git"
        shim.write_text(
            f"#!{sys.executable}\n"
            "import os, sys, time\nfrom pathlib import Path\n"
            "if 'bundle' in sys.argv and 'verify' in sys.argv:\n"
            f"    Path({str(ready)!r}).touch()\n"
            "    deadline = time.monotonic() + 20\n"
            f"    while not Path({str(release)!r}).exists():\n"
            "        if time.monotonic() >= deadline: sys.exit(90)\n"
            "        time.sleep(.01)\n"
            f"os.execv({real_git!r}, [{real_git!r}, *sys.argv[1:]])\n"
        )
        shim.chmod(0o755)
        env = dict(self.env, PATH=str(shim_root) + os.pathsep + self.env["PATH"])
        process = subprocess.Popen(
            [sys.executable, str(tc.CLI), "--repo", str(self.repo), "sweep", "--apply", "--archive-dirty"],
            cwd=self.repo, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
        )
        try:
            deadline = time.monotonic() + 10
            while not ready.exists() and process.poll() is None and time.monotonic() < deadline:
                time.sleep(.01)
            self.assertTrue(ready.exists(), "archive creation did not reach its Git verification barrier")
            self.git("worktree", "lock", "--reason", "owner resumed during archive", str(path))
            release.touch()
            stdout, stderr = process.communicate(timeout=20)
            self.assertEqual(process.returncode, 2, str((stdout, stderr)))
            self.assertIn("locked", stderr.lower())
            self.assertTrue(path.is_dir())
            self.assertEqual((path / "code.txt").read_text(), "unsaved owner bytes\n")
            self.assertEqual(self.git("show", ":code.txt", cwd=path), "staged owner bytes")
            self.assertEqual((path / "new.txt").read_bytes(), b"untracked\x00owner bytes")
            self.assertEqual((path / "build" / "ignored").read_bytes(), b"ignored\x00owner bytes")
            self.assertEqual(self.git("status", "--porcelain", "--ignored", cwd=path), before_status)
            self.assertEqual(self.git("rev-parse", attempt["branch"]), head)
            self.assertIn("locked owner resumed during archive", self.git("worktree", "list", "--porcelain"))
        finally:
            release.touch()
            if process.poll() is None:
                process.kill()
            process.communicate(timeout=10)

    def test_archive_changed_after_verification_retains_original_worktree_bytes(self):
        attempt, path = self.terminal()
        (path / "code.txt").write_text("owner bytes before archive corruption\n")
        (path / "new.txt").write_bytes(b"retained\x00untracked")
        ready, release = self.root / "archive-ready", self.root / "archive-release"
        archive_root = self.repo / ".git/collab/archives"
        shim_root = self.root / "shim"
        shim_root.mkdir()
        real_git = shutil.which("git", path=self.env.get("PATH"))
        shim = shim_root / "git"
        shim.write_text(
            f"#!{sys.executable}\n"
            "import os, sys, time\nfrom pathlib import Path\n"
            f"archives = list(Path({str(archive_root)!r}).glob('*/manifest.json'))\n"
            "if 'worktree' in sys.argv and 'list' in sys.argv and archives:\n"
            "    archive = archives[0].parent\n"
            "    if not list(archive.glob('verify-*')):\n"
            f"        Path({str(ready)!r}).write_text(str(archive))\n"
            "        deadline = time.monotonic() + 20\n"
            f"        while not Path({str(release)!r}).exists():\n"
            "            if time.monotonic() >= deadline: sys.exit(90)\n"
            "            time.sleep(.01)\n"
            f"os.execv({real_git!r}, [{real_git!r}, *sys.argv[1:]])\n"
        )
        shim.chmod(0o755)
        env = dict(self.env, PATH=str(shim_root) + os.pathsep + self.env["PATH"])
        process = subprocess.Popen(
            [sys.executable, str(tc.CLI), "--repo", str(self.repo), "sweep", "--apply", "--archive-dirty"],
            cwd=self.repo, env=env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
        )
        try:
            deadline = time.monotonic() + 10
            while not ready.exists() and process.poll() is None and time.monotonic() < deadline:
                time.sleep(.01)
            self.assertTrue(ready.exists(), "archive verification did not complete before removal")
            archive = Path(ready.read_text())
            with (archive / "files.tar").open("ab") as stream:
                stream.write(b"archive changed before destructive cleanup")
            release.touch()
            stdout, stderr = process.communicate(timeout=20)
            self.assertEqual(process.returncode, 2, str((stdout, stderr)))
            self.assertIn("Archive artifact changed before cleanup", stderr)
            self.assertTrue(path.is_dir())
            self.assertEqual((path / "code.txt").read_text(), "owner bytes before archive corruption\n")
            self.assertEqual((path / "new.txt").read_bytes(), b"retained\x00untracked")
            self.assertEqual(self.git("rev-parse", attempt["branch"]), attempt["base"])
        finally:
            release.touch()
            if process.poll() is None:
                process.kill()
            process.communicate(timeout=10)


if __name__ == "__main__":
    unittest.main()
