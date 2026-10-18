"""Protect source preservation at the Corenius import boundary."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "verify_corenius_import.py"


class ImportTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.env = {key: value for key, value in os.environ.items() if not key.startswith("GIT_")}
        self.git("init", "-q")
        self.git("config", "user.name", "Import fixture")
        self.git("config", "user.email", "fixture@example.invalid")
        self.git("config", "core.hooksPath", "/dev/null")
        (self.root / "runtime.py").write_text("print('retained source')\n")
        self.git("add", "runtime.py")
        self.git("commit", "-qm", "source")
        source = self.git("rev-parse", "HEAD")
        content = (self.root / "runtime.py").read_bytes()
        self.git("rm", "runtime.py")
        component = self.root / "corenius"
        component.mkdir()
        (component / "runtime.py").write_bytes(content)
        self.manifest = {
            "schema": "harp-corenius-import/v1",
            "source_commit": source,
            "branches": [{"commit": source}],
            "worktrees": [{"head": source}],
            "adaptations": {},
        }
        self.write_manifest()
        self.git("add", "corenius/runtime.py", "corenius/HARP_IMPORT.json")
        self.git("commit", "-qm", "absorb")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, env=self.env).decode().strip()

    def write_manifest(self):
        (self.root / "corenius/HARP_IMPORT.json").write_text(json.dumps(self.manifest))

    def check(self):
        return subprocess.run([sys.executable, str(SCRIPT)], cwd=self.root, env=self.env,
                              capture_output=True, text=True)

    def test_exact_source_with_retained_history_passes(self):
        result = self.check()
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_changed_source_bytes_fail(self):
        (self.root / "corenius/runtime.py").write_text("print('lost fix')\n")
        self.assertIn("bytes changed", self.check().stderr)

    def test_missing_source_file_fails(self):
        (self.root / "corenius/runtime.py").unlink()
        self.assertNotEqual(self.check().returncode, 0)

    def test_symlinked_source_is_rejected(self):
        path = self.root / "corenius/runtime.py"
        path.unlink()
        path.symlink_to(SCRIPT)
        self.assertIn("symlinked", self.check().stderr)

    def test_source_objects_without_integrated_history_fail(self):
        tree = self.git("rev-parse", self.manifest["source_commit"] + "^{tree}")
        unrelated = subprocess.check_output(["git", "commit-tree", tree, "-m", "unrelated"],
                                            cwd=self.root, env=self.env).decode().strip()
        self.manifest["source_commit"] = unrelated
        self.manifest["branches"] = [{"commit": unrelated}]
        self.manifest["worktrees"] = [{"head": unrelated}]
        self.write_manifest()
        self.assertIn("history is not included", self.check().stderr)


if __name__ == "__main__":
    unittest.main()
