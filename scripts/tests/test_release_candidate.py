"""Exercise candidate publication in disposable repositories with fixture tools."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
import release_candidate as release


class CandidateTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name).resolve()
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.bin = self.root / "tools"
        self.bin.mkdir()
        self.env = os.environ.copy()
        for key in list(self.env):
            if key.startswith("GIT_"):
                self.env.pop(key)
        self.env["PATH"] = str(self.bin) + os.pathsep + self.env["PATH"]
        self.env["HARP_TARGET_DIR"] = str(self.repo / ".build")
        self.environment = patch.dict(os.environ, self.env, clear=True)
        self.environment.start()
        self.addCleanup(self.environment.stop)
        self.git("init", "-q")
        self.git("config", "user.name", "Release Test")
        self.git("config", "user.email", "release@example.invalid")
        self.git("config", "core.hooksPath", "/dev/null")
        (self.repo / ".gitignore").write_text("/dist/\n/.build/\n")
        (self.repo / "input").write_text("tracked input\n")
        self.git("add", ".gitignore", "input")
        self.git("commit", "-qm", "fixture")
        self.tool("mise", "exit 0")
        self.tool("rustc", "printf 'rustc fixture\\nhost: aarch64-apple-darwin\\n'")
        self.tool("cargo", '''if [ "$1" = metadata ]; then
printf '%s\n' '{"packages":[{"name":"harp","version":"0.1.0"}]}'
else
mkdir -p "$HARP_TARGET_DIR/aarch64-apple-darwin/size"
printf '#!/bin/sh\nprintf "harp 0.1.0\\\\n"\n' > "$HARP_TARGET_DIR/aarch64-apple-darwin/size/harp"
chmod +x "$HARP_TARGET_DIR/aarch64-apple-darwin/size/harp"
fi''')

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.repo, text=True).strip()

    def tool(self, name, body):
        executable = self.bin / name
        executable.write_text("#!/bin/sh\nset -eu\n" + body + "\n")
        executable.chmod(0o755)

    def candidate(self):
        return release.create_candidate(self.repo)

    def test_archive_manifest_checksums_and_extracted_binary(self):
        output = self.candidate()
        manifest = json.loads((output / "manifest.json").read_text())
        self.assertEqual(manifest["commit"], self.git("rev-parse", "HEAD"))
        self.assertEqual(manifest["version"], "0.1.0")
        self.assertEqual(manifest["target"], "aarch64-apple-darwin")
        archive = output / manifest["archive"]["name"]
        self.assertEqual(hashlib.sha256(archive.read_bytes()).hexdigest(), manifest["archive"]["sha256"])
        with tarfile.open(archive) as bundle:
            self.assertEqual(set(bundle.getnames()), {"harp", "RELEASE_NOTES.md"})
            self.assertEqual(bundle.getmember("harp").mode, 0o755)
            self.assertEqual(bundle.getmember("harp").uname, "")
            executable = bundle.extractfile("harp").read()
        self.assertEqual(hashlib.sha256(executable).hexdigest(), manifest["binary"]["sha256"])
        for line in (output / "SHA256SUMS").read_text().splitlines():
            digest, name = line.split("  ")
            self.assertEqual(hashlib.sha256((output / name).read_bytes()).hexdigest(), digest)
        self.assertEqual(self.git("status", "--porcelain"), "")

    def test_dirty_tracked_input_rejected_before_gate(self):
        (self.repo / "input").write_text("changed")
        with self.assertRaisesRegex(release.ReleaseError, "clean"):
            self.candidate()
        self.assertFalse((self.repo / "dist").exists())

    def test_untracked_input_rejected(self):
        (self.repo / "new-input").write_text("uncommitted")
        with self.assertRaisesRegex(release.ReleaseError, "clean"):
            self.candidate()

    def test_failed_gate_publishes_nothing(self):
        self.tool("mise", "exit 7")
        with self.assertRaises(subprocess.CalledProcessError):
            self.candidate()
        self.assertFalse((self.repo / "dist").exists())

    def test_gate_mutating_tracked_input_rejected(self):
        self.tool("mise", "printf changed > input")
        with self.assertRaisesRegex(release.ReleaseError, "clean"):
            self.candidate()

    def test_commit_change_during_gate_rejected(self):
        self.tool("mise", "git -c core.hooksPath=/dev/null commit --allow-empty -qm changed")
        with self.assertRaisesRegex(release.ReleaseError, "commit changed"):
            self.candidate()

    def test_existing_candidate_preserved(self):
        output = self.candidate()
        previous = (output / "manifest.json").read_bytes()
        with self.assertRaisesRegex(release.ReleaseError, "exists"):
            self.candidate()
        self.assertEqual((output / "manifest.json").read_bytes(), previous)

    def test_symlinked_dist_rejected(self):
        external = self.root / "external"
        external.mkdir()
        (self.repo / "dist").symlink_to(external, target_is_directory=True)
        with self.assertRaisesRegex(release.ReleaseError, "symlink"):
            self.candidate()
        self.assertEqual(list(external.iterdir()), [])

    def test_wrong_binary_version_publishes_nothing(self):
        cargo = self.bin / "cargo"
        cargo.write_text(cargo.read_text().replace('harp 0.1.0', 'harp 9.9.9'))
        with self.assertRaisesRegex(release.ReleaseError, "version"):
            self.candidate()
        self.assertEqual(list((self.repo / "dist").iterdir()), [])

    def test_archive_normalizes_executable_permissions(self):
        cargo = self.bin / "cargo"
        cargo.write_text(cargo.read_text().replace('chmod +x', 'chmod -x'))
        output = self.candidate()
        archive = next(output.glob("*.tar.gz"))
        with tarfile.open(archive) as bundle:
            self.assertEqual(bundle.getmember("harp").mode, 0o755)


if __name__ == "__main__":
    unittest.main()
