#!/usr/bin/env python3
"""Exercise the real shell gate dispatcher without starting suite services."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class GateDispatcherTests(unittest.TestCase):
    def dispatch(self, **options):
        with tempfile.TemporaryDirectory(prefix="gate-dispatch-") as directory:
            fixture = Path(directory)
            trace = fixture / "trace.jsonl"
            shim = "#!" + sys.executable + "\n" + r'''import json, os, sys
if len(sys.argv) > 1 and sys.argv[1] == "tools/temp_root.py":
    os.execv(sys.executable, [sys.executable, *sys.argv[1:]])
with open(os.environ["GATE_TRACE"], "a") as stream:
    stream.write(json.dumps([os.path.basename(sys.argv[0]), *sys.argv[1:]]) + "\n")
raise SystemExit(7 if os.environ.get("GATE_FAIL_MATCH") in sys.argv[1:] else 0)
'''
            for name in ("python3", "cargo", "bash"):
                executable = fixture / name
                executable.write_text(shim)
                executable.chmod(0o700)
            environment = dict(os.environ)
            for variable in ("CHECK_BENCH", "CHECK_SYNC", "CHECK_CARGO", "CHECK_TEAM_CONTROL_PLANE"):
                environment.pop(variable, None)
            environment.update(PATH=str(fixture) + os.pathsep + environment["PATH"],
                               GATE_TRACE=str(trace), CHECK_CARGO="0")
            environment.update(options)
            result = subprocess.run(["/bin/sh", "tools/agents/check.sh"], cwd=ROOT,
                                    env=environment, text=True, capture_output=True)
            calls = [json.loads(line) for line in trace.read_text().splitlines()]
            return result, calls

    def test_each_existing_suite_runs_once(self):
        result, calls = self.dispatch()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        for suite in ("tools/agents/test_worker.py", "tools/agents/test_effort.py"):
            self.assertEqual(sum(suite in call for call in calls), 1, suite)
        self.assertEqual(sum(any(Path(argument).name == "check_temp_paths.py"
                                 for argument in call[1:]) for call in calls), 1)

    def test_tracker_lint_runs_once(self):
        result, calls = self.dispatch()
        self.assertEqual(result.returncode, 0)
        self.assertEqual(sum("tools/agents/tracker.py" in call and "lint" in call for call in calls), 1)

    def test_default_retains_sqlite_but_excludes_parked_services(self):
        result, calls = self.dispatch()
        self.assertEqual(result.returncode, 0)
        self.assertTrue(any("crates/sqlite-controller/Cargo.toml" in call for call in calls))
        self.assertFalse(any("crates/team-control-plane/tests/run-temporal.sh" in call for call in calls))

    def test_all_options_include_parked_integration_and_optional_suites(self):
        result, calls = self.dispatch(CHECK_TEAM_CONTROL_PLANE="1", CHECK_BENCH="1",
                                      CHECK_SYNC="1", CHECK_CARGO="1")
        self.assertEqual(result.returncode, 0)
        self.assertEqual(sum("crates/team-control-plane/tests/run-temporal.sh" in call for call in calls), 1)
        self.assertEqual(sum("bench/harness/test_lead_driven.py" in call for call in calls), 1)
        self.assertIn(["cargo", "test", "--offline", "--", "--test-threads=1"], calls)
        self.assertIn(["python3", "tools/sync_agentic_workflow.py", "--check"], calls)
        self.assertIn(["python3", "tools/insula/test_capsule.py"], calls)

    def test_failure_is_reported_and_later_checks_still_run(self):
        result, calls = self.dispatch(GATE_FAIL_MATCH="tools/agents/test_worker.py")
        self.assertEqual(result.returncode, 1)
        self.assertIn("FAILED:", result.stdout)
        self.assertIn("check: FAILED", result.stdout)
        self.assertTrue(any("crates/sqlite-controller/Cargo.toml" in call for call in calls))


if __name__ == "__main__":
    unittest.main()
