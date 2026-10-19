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
    def dispatch(self, *arguments, **options):
        with tempfile.TemporaryDirectory(prefix="gate-dispatch-") as directory:
            fixture = Path(directory)
            trace = fixture / "trace.jsonl"
            shim = "#!" + sys.executable + "\n" + r'''import json, os, sys
if len(sys.argv) > 1 and sys.argv[1] in ("tools/temp_root.py", "tools/agents/check.py"):
    os.execv(sys.executable, [sys.executable, *sys.argv[1:]])
with open(os.environ["GATE_TRACE"], "a") as stream:
    stream.write(json.dumps([os.path.basename(sys.argv[0]), *sys.argv[1:]]) + "\n")
raise SystemExit(7 if os.environ.get("GATE_FAIL_MATCH") in sys.argv[1:] else 0)
'''
            for name in ("python3", "cargo", "bash"):
                executable = fixture / name
                executable.write_text(shim)
                executable.chmod(0o700)
            git = "#!" + sys.executable + "\n" + r'''import json, os, sys
with open(os.environ["GATE_TRACE"], "a") as stream:
    stream.write(json.dumps([os.path.basename(sys.argv[0]), *sys.argv[1:]]) + "\n")
if os.environ.get("GATE_GIT_MODE") == "fail":
    raise SystemExit(1)
args = sys.argv[1:]
if args == ["rev-parse", "--verify", "--quiet", "main"]:
    raise SystemExit(0)
if args == ["merge-base", "HEAD", "main"]:
    print("base")
    raise SystemExit(0)
if args == ["diff", "--name-only", "base", "HEAD"]:
    sys.stdout.write(os.environ.get("GATE_CHANGED_FILES", ""))
    if os.environ.get("GATE_CHANGED_FILES"):
        sys.stdout.write("\n")
    raise SystemExit(0)
if args == ["diff", "--name-only"]:
    sys.stdout.write(os.environ.get("GATE_WORKTREE_CHANGED_FILES", ""))
    if os.environ.get("GATE_WORKTREE_CHANGED_FILES"):
        sys.stdout.write("\n")
    raise SystemExit(0)
if args == ["diff", "--name-only", "--cached"]:
    sys.stdout.write(os.environ.get("GATE_INDEX_CHANGED_FILES", ""))
    if os.environ.get("GATE_INDEX_CHANGED_FILES"):
        sys.stdout.write("\n")
    raise SystemExit(0)
raise SystemExit(1)
'''
            git_executable = fixture / "git"
            git_executable.write_text(git)
            git_executable.chmod(0o700)
            environment = dict(os.environ)
            for variable in ("CHECK_BENCH", "CHECK_SYNC", "CHECK_CARGO", "CHECK_TEAM_CONTROL_PLANE", "CHECK_CONTROLLER_THREADS",
                             "GATE_CHANGED_FILES", "GATE_GIT_MODE", "GATE_INDEX_CHANGED_FILES",
                             "GATE_WORKTREE_CHANGED_FILES"):
                environment.pop(variable, None)
            environment.update(PATH=str(fixture) + os.pathsep + environment["PATH"],
                               GATE_TRACE=str(trace), CHECK_CARGO="0")
            environment.update(options)
            result = subprocess.run(["/bin/sh", "tools/agents/check.sh", *arguments], cwd=ROOT,
                                    env=environment, text=True, capture_output=True)
            calls = [json.loads(line) for line in (trace.read_text().splitlines() if trace.exists() else [])]
            return result, calls

    def test_selected_suite_runs_without_default_suites(self):
        result, calls = self.dispatch('--suite', '10-wt')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(calls, [['python3', 'tools/agents/test_wt.py']])

    def test_defaults_keep_selected_runtime_and_pure_evidence(self):
        result, calls = self.dispatch()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn(['python3', 'tools/agents/test_wt.py'], calls)
        self.assertIn(['python3', 'tools/agents/test_evidence.py'], calls)
        self.assertTrue(any('controller.durable.runtime_test' in call for call in calls))
        self.assertTrue(any('controller.durable.qualify_test' in call for call in calls))
        self.assertEqual(sum('tools/agents/test_wt.py' in call for call in calls), 1)
        self.assertFalse(any('controller/Cargo.toml' in call for call in calls))
        self.assertFalse(any('tools/agents/tracker.py' in call for call in calls))

    def test_retired_flags_cannot_restore_old_authority(self):
        result, calls = self.dispatch(CHECK_TEAM_CONTROL_PLANE='1', CHECK_BENCH='1',
                                      CHECK_SYNC='1', CHECK_CARGO='1', CHECK_CONTROLLER_THREADS='8')
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn(['cargo', 'test', '--offline', '--', '--test-threads=1'], calls)
        self.assertIn(['python3', 'tools/insula/test_capsule.py'], calls)
        for retired in ('controller/Cargo.toml', 'tools/agents/test_worker.py', 'tools/agents/test_tracker.py',
                        'bench/harness/test_lead_driven.py', 'tools/sync_agentic_workflow.py'):
            self.assertFalse(any(retired in call for call in calls), retired)
        for suite in ('15-worker','20-land','30-tracker','65-collab','70-sqlite-controller',
                      '75-controller-python','80-monitor-wake','70-team-control-plane','sync','policy'):
            with self.subTest(suite=suite):
                result, calls = self.dispatch('--suite', suite, CHECK_SYNC='1', CHECK_TEAM_CONTROL_PLANE='1')
                self.assertEqual(result.returncode, 2)
                self.assertIn('unknown suite', result.stderr)
                self.assertEqual(calls, [])

    def test_failure_is_reported_and_later_checks_still_run(self):
        result, calls = self.dispatch(GATE_FAIL_MATCH='tools/agents/test_wt.py')
        self.assertEqual(result.returncode, 1)
        self.assertIn('FAILED:', result.stdout)
        self.assertIn('check: FAILED', result.stdout)
        self.assertTrue(any('controller.durable.runtime_test' in call for call in calls))


if __name__ == '__main__':
    unittest.main()
