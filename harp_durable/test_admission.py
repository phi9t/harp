"""Git intent boundaries and the complete admitted coding CLI path."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]


class GitInputs(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.repo = Path(self.temp.name)
        self.git("init", "-q")
        self.git("config", "user.name", "Intent fixture")
        self.git("config", "user.email", "fixture@harp.local")
        self.git("config", "core.hooksPath", "/dev/null")
        (self.repo / "task.md").write_text("Produce answer.txt containing 42.\n")
        (self.repo / "acceptance.json").write_text(json.dumps({"checks": [
            ["/usr/bin/python3", "-c", "from pathlib import Path; assert Path('answer.txt').read_text() == '42\\n'"]]}))
        self.git("add", "task.md", "acceptance.json")
        self.git("commit", "-qm", "Pin task and acceptance")
        self.base = self.git("rev-parse", "HEAD").strip()

    def git(self, *args):
        return subprocess.check_output(["git", "-C", str(self.repo), *args], text=True)

    def test_git_bytes_are_pinned_without_reading_working_edits(self):
        from .admission import blob, commit
        (self.repo / "task.md").write_text("later uncommitted task\n")
        self.assertEqual(commit(self.repo, "HEAD"), self.base)
        self.assertEqual(blob(self.repo, self.base, "task.md"), b"Produce answer.txt containing 42.\n")
        (self.repo / "link.md").symlink_to("task.md")
        self.git("add", "link.md")
        self.git("commit", "-qm", "Link")
        with self.assertRaisesRegex(ValueError, "regular files"):
            blob(self.repo, commit(self.repo, "HEAD"), "link.md")

    def test_rejects_ambiguous_acceptance_and_escaping_paths(self):
        from .admission import acceptance, relative_path
        for data in (b'{"checks":[],"checks":[["true"]]}', b'{"checks":"true"}', b'{"checks":[[]]}'):
            with self.subTest(data=data), self.assertRaises(ValueError):
                acceptance(data)
        for path in ("../escape", "/absolute", ".git/config", "dir/../escape", "dir//file"):
            with self.subTest(path=path), self.assertRaises(ValueError):
                relative_path(path)

    def test_export_bundle_pins_a_commit_without_creating_source_refs(self):
        from .admission import export_bundle
        before = self.git("show-ref")
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.bundle"
            export_bundle(self.repo, self.base, path, directory)
            heads = subprocess.check_output(["git", "bundle", "list-heads", str(path)], text=True)
            self.assertEqual(heads.strip(), self.base + " refs/heads/source")
        self.assertEqual(self.git("show-ref"), before)


@unittest.skipUnless(sys.platform == "linux" and os.environ.get("HARP_TEST_CGROUP_ROOT"),
                     "requires explicit delegated cgroup and admitted native rootfs on Linux")
class CodingLifecycle(GitInputs):
    def setUp(self):
        # Owned scratch outside /tmp is also used for guardian temporaries.
        scratch = ROOT / ".scratch" / "coding-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        self.project = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(self.project.cleanup)
        self.root = Path(self.project.name)
        self.repo = self.root / "repository"
        self.repo.mkdir()
        self.git("init", "-q")
        self.git("config", "user.name", "Intent fixture")
        self.git("config", "user.email", "fixture@harp.local")
        self.git("config", "core.hooksPath", "/dev/null")
        (self.repo / "task.md").write_text("Produce answer.txt containing 42.\n")
        (self.repo / "acceptance.json").write_text(json.dumps({"checks": [["/usr/bin/true"]]}))
        self.git("add", "task.md", "acceptance.json")
        self.git("commit", "-qm", "Pin task and acceptance")
        self.base = self.git("rev-parse", "HEAD").strip()
        self.state = self.root / "state"
        self.execution = self.root / "execution.json"
        self.env = {**os.environ, "PYTHONPATH": f"{ROOT / 'corenius'}:{ROOT}",
                    "PYTHONDONTWRITEBYTECODE": "1", "TMPDIR": str(self.root)}
        entry = ([os.environ["HARP_DURABLE_BIN"], "--format", "json", "durable"]
                 if "HARP_DURABLE_BIN" in os.environ else [sys.executable, "-m", "harp_durable"])
        self.command = [*entry, "--state", str(self.state), "--execution", str(self.execution)]
        self.call("prepare", "--rootfs", os.environ["HARP_TEST_ROOTFS"],
                  "--cgroup-root", os.environ["HARP_TEST_CGROUP_ROOT"])

    def call(self, *args, ok=True):
        out = subprocess.run([*self.command, *args], cwd=ROOT, env=self.env,
                             text=True, capture_output=True, timeout=60)
        self.assertEqual(out.returncode == 0, ok, out.stderr)
        return json.loads(out.stdout if ok else out.stderr)["data" if ok else "error"]

    def admit(self, run_id, *extra, ok=True):
        return self.call("admit", run_id, "--repository", str(self.repo),
                         "--task", "task.md", "--acceptance", "acceptance.json",
                         "--provider", "deterministic", "--fixture-path", "answer.txt",
                         "--fixture-content", "42\n", *extra, ok=ok)

    def test_pinned_duplicate_admission_and_private_candidate(self):
        first = self.admit("one")
        self.assertTrue(self.admit("one")["reused"])
        (self.repo / "task.md").write_text("later working edit\n")
        self.assertEqual(self.admit("one")["inputs"], first["inputs"])
        self.admit("one", "--model", "conflicting-model", ok=False)
        self.call("start", "--until-idle", "--timeout", "30")
        status = self.call("status", "one")
        diagnostics = [Path(a["logs"]["stderr"]).read_text() for a in status["attempts"]
                       if Path(a["logs"]["stderr"]).exists()]
        self.assertEqual(status["status"], "SUCCEEDED", {"status": status, "logs": diagnostics})
        self.assertEqual(status["result"]["base"], self.base)
        self.assertEqual(status["result"]["attempt_id"], status["attempts"][0]["id"])
        self.assertFalse((self.repo / "answer.txt").exists())
        self.assertEqual((self.repo / "task.md").read_text(), "later working edit\n")
        self.assertEqual(self.git("rev-parse", "HEAD").strip(), self.base)
        self.assertTrue((Path(status["attempts"][0]["directory"]) / "payload/candidate.bundle").is_file())
        receipt = json.loads((Path(status["attempts"][0]["directory"]) / "receipt.json").read_text())
        self.assertTrue(receipt["cleanup"])
        self.assertEqual(receipt["cleanup_proof"]["backend"], "cgroup-v2")

    def test_changed_input_bundle_refuses_before_attempt_claim(self):
        admitted = self.admit("tampered")
        task = self.state / "inputs" / admitted["inputs"] / "task.md"
        task.chmod(0o600)
        task.write_text("substituted task\n")
        self.call("start", "--until-idle", "--timeout", "10", ok=False)
        self.assertEqual(self.call("status", "tampered")["attempts"], [])

    def test_capacity_one_queues_another_workspace(self):
        self.admit("first", "--fixture-sleep", "10")
        self.admit("second")
        process = subprocess.Popen([*self.command, "start", "--until-idle", "--timeout", "30"],
                                   cwd=ROOT, env=self.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        def cleanup():
            if process.poll() is None:
                process.terminate()
            process.communicate(timeout=15)
        self.addCleanup(cleanup)
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline:
            status = self.call("status", "first")
            if status["attempts"]:
                self.assertEqual(self.call("status", "second")["attempts"], [])
                break
            time.sleep(0.02)
        else:
            self.fail("first worker was not dispatched")
        _out, err = process.communicate(timeout=40)
        self.assertEqual(process.returncode, 0, err)
        first, second = self.call("status", "first"), self.call("status", "second")
        self.assertEqual((first["status"], second["status"]), ("SUCCEEDED", "SUCCEEDED"))
        self.assertNotEqual(first["attempts"][0]["directory"], second["attempts"][0]["directory"])


if __name__ == "__main__":
    unittest.main()
