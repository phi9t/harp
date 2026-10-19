"""Public CLI retirement and retained offline solo replay regressions."""

import json
import os
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[3]
CLI = Path(__file__).with_name("run.py")


class RetirementCLI(unittest.TestCase):
    def setUp(self):
        # Keep all fixtures and the capsule cache owned and inside this checkout.
        self.temporary = tempfile.TemporaryDirectory(prefix="retirement-", dir=ROOT / ".scratch")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.out = self.root / "output"
        self.env = dict(os.environ, INSULA_CAPSULES=str(self.root / "capsules"),
                        TMPDIR=str(self.root), TMP=str(self.root), TEMP=str(self.root),
                        PYTHONDONTWRITEBYTECODE="1")

    def cli(self, *arguments):
        command = [sys.executable, str(CLI), *arguments]
        # Base capsules expect merged-/usr. On split-/usr hosts, expose the
        # existing loader libraries in a private mount namespace for the CLI
        # and its real nested capsules. No host files or capsule layers change.
        if not Path("/usr/lib64").exists() and Path("/lib64").is_dir():
            mounts = ["bwrap", "--die-with-parent", "--ro-bind", "/", "/",
                      "--bind", str(ROOT / ".scratch"), str(ROOT / ".scratch"),
                      "--dev-bind", "/dev", "/dev", "--proc", "/proc",
                      "--tmpfs", "/usr"]
            for child in sorted(Path("/usr").iterdir()):
                mounts.extend(["--ro-bind", str(child), str(child)])
            mounts.extend(["--ro-bind", "/lib64", "/usr/lib64"])
            library_dir = Path("/lib") / (os.uname().machine + "-linux-gnu")
            if library_dir.is_dir():
                mounts.extend(["--ro-bind", str(library_dir),
                               str(Path("/usr/lib") / library_dir.name)])
            command = [*mounts, "--", *command]
        return subprocess.run(
            command, cwd=ROOT, env=self.env,
            capture_output=True, text=True, timeout=90,
        )

    def assert_refused(self, *arguments):
        # An absent task root also proves refusal precedes task discovery.
        proc = self.cli("--tasks-root", str(self.root / "missing-tasks"),
                        "--out", str(self.out), *arguments)
        self.assertEqual(proc.returncode, 2, proc.stdout + proc.stderr)
        self.assertIn("Retired", proc.stderr)
        self.assertIn("harp durable", proc.stderr)
        self.assertFalse(self.out.exists(), "retired execution allocated an output directory")
        return proc

    def test_controller_refuses_before_output_creation(self):
        self.assert_refused("--condition", "controller:t1")

    def test_fleet_refuses_before_output_creation(self):
        self.assert_refused("--condition", "fleet:2")

    def test_cmd_refuses_without_executing_command(self):
        marker = self.root / "command-executed"
        command = "printf executed > " + shlex.quote(str(marker))
        self.assert_refused("--agent", "cmd:" + command)
        self.assertFalse(marker.exists(), "retired cmd agent executed its payload")

    def test_reference_solo_help_remains_available(self):
        self.assert_help("reference")

    def test_noop_solo_help_remains_available(self):
        self.assert_help("noop")

    def assert_help(self, agent):
        proc = self.cli("--condition", "solo", "--agent", agent, "--help",
                        "--out", str(self.out))
        self.assertEqual(proc.returncode, 0, proc.stderr)
        self.assertIn("--tasks-root", proc.stdout)
        self.assertIn("--agent", proc.stdout)
        self.assertIn("solo reference/noop", proc.stdout)
        self.assertFalse(self.out.exists())

    def replay(self, agent):
        task = self.root / "tasks" / "tiny"
        for directory in ("environment", "tests", "solution"):
            (task / directory).mkdir(parents=True)
        (task / "instruction.md").write_text("Replace answer.txt with the word fixed.\n")
        (task / "environment" / "answer.txt").write_text("broken\n")
        (task / "solution" / "solve.sh").write_text("set -eu\nprintf 'fixed\\n' > answer.txt\n")
        (task / "tests" / "grade.sh").write_text(
            'set -eu\nif [ "$(cat answer.txt)" = fixed ]; then\n'
            '  echo graded-fixed\nelse\n  echo graded-broken\n  exit 1\nfi\n'
        )
        (task / "capsule.json").write_text(json.dumps({
            "layers": [{"kind": "base", "source": "host-usr"}],
            "policy": {"net": "none", "gpu": False},
        }))
        (task / "task.toml").write_text(
            '[bench]\ncapsule_request = "capsule.json"\nnet = "none"\n'
            '[verifier]\ncommand = "sh tests/grade.sh"\ntimeout_sec = 10\n'
        )
        proc = self.cli("tiny", "--tasks-root", str(task.parent), "--condition", "solo",
                        "--agent", agent, "--out", str(self.out))
        self.assertEqual(proc.returncode, 0, proc.stdout + proc.stderr)
        paths = list(self.out.glob("*/result.json"))
        self.assertEqual(len(paths), 1, proc.stdout + proc.stderr)
        record = json.loads(paths[0].read_text())
        self.assertEqual(record["task_id"], "tiny")
        self.assertEqual(record["condition"], "solo")
        self.assertEqual(record["agent"]["spec"], agent)
        self.assertEqual(record["capsule"]["net"], "none")
        self.assertTrue(record["capsule"]["digest"])
        self.assertEqual(record["outcome"]["agent_rc"], 0,
                         (paths[0].parent / "agent-tail.txt").read_text())
        self.assertFalse(record["outcome"]["timed_out"])
        self.assertIsNone(record["cost"]["usd"])
        return record, paths[0].parent, proc

    def test_reference_solves_and_passes_offline(self):
        record, directory, proc = self.replay("reference")
        self.assertIs(record["outcome"]["passed"], True)
        self.assertEqual(record["outcome"]["reward"], 1.0)
        self.assertEqual(record["outcome"]["verifier_rc"], 0)
        self.assertIn("graded-fixed", (directory / "verify-tail.txt").read_text())
        self.assertIn("PASS", proc.stdout)

    def test_noop_runs_but_fails_grading_offline(self):
        record, directory, proc = self.replay("noop")
        self.assertIs(record["outcome"]["passed"], False)
        self.assertEqual(record["outcome"]["reward"], 0.0)
        self.assertNotEqual(record["outcome"]["verifier_rc"], 0)
        self.assertIn("graded-broken", (directory / "verify-tail.txt").read_text())
        self.assertIn("FAIL", proc.stdout)


if __name__ == "__main__":
    unittest.main()
