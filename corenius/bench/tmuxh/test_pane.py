"""pane.py runs real capsules (no model): state, events, result and exit code."""

from __future__ import annotations

import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import pane  # noqa: E402
import state  # noqa: E402

ROOT = Path(__file__).resolve().parents[2]
REQ = str(ROOT / "bench" / "insula" / "requests" / "toolchains.json")


class PaneTest(unittest.TestCase):
    def setUp(self):
        (ROOT / ".scratch").mkdir(exist_ok=True)
        self.tmp = tempfile.TemporaryDirectory(dir=str(ROOT / ".scratch"))
        self.root = Path(self.tmp.name)
        self.ws = self.root / "ws"
        self.ws.mkdir()

    def tearDown(self):
        self.tmp.cleanup()

    def job(self, command, timeout=60, env=None, unit="u1"):
        return {"unit": unit, "run_id": "r1", "unit_dir": str(self.root / "units" / unit), "workspace": str(self.ws),
                "meta": {"task": "demo", "condition": "solo", "agent": "cmd", "ticket": ""},
                "plan": {"command": command, "env": env or {}, "net": "none", "gpu": False, "ro": [], "timeout": timeout,
                         "capsule": REQ}}

    def run_job(self, job):
        buf = io.StringIO()
        code = pane.run_job(job, out=buf, color=False)
        return code, buf.getvalue(), state.read(Path(job["unit_dir"]) / "state.json")

    def test_successful_unit_records_result_state_and_stream(self):
        code, shown, st = self.run_job(self.job("echo hello; echo edit > f.txt; exit 0"))
        self.assertEqual(code, 0)
        self.assertEqual((st["phase"], st["rc"]), ("done", 0))
        self.assertIn("hello", shown)
        self.assertIn("finished rc=0", shown)
        self.assertEqual((self.ws / "f.txt").read_text(), "edit\n")
        unit = self.root / "units" / "u1"
        self.assertEqual(json.loads((unit / "result.json").read_text())["rc"], 0)
        self.assertIn("hello", (unit / "events.jsonl").read_text())
        self.assertIn("finished", (unit / "pane.log").read_text())

    def test_nonzero_exit_is_reported_not_swallowed(self):
        code, shown, st = self.run_job(self.job("echo boom >&2; exit 7"))
        self.assertEqual((code, st["phase"], st["rc"]), (7, "done", 7))
        self.assertIn("boom", shown)

    def test_traecli_style_events_update_tokens_and_counters(self):
        events = [
            {"type": "item.started", "item": {"type": "command_execution", "command": "ls"}},
            {"type": "item.completed", "item": {"type": "command_execution", "command": "ls", "aggregated_output": "a\n", "exit_code": 0}},
            {"type": "item.completed", "item": {"type": "file_change", "changes": [{"path": "/workspace/a", "kind": "update"}]}},
            {"type": "turn.completed", "usage": {"input_tokens": 100, "output_tokens": 7}},
        ]
        script = "; ".join("printf '%s\\n' '" + json.dumps(e) + "'" for e in events)
        code, shown, st = self.run_job(self.job(script))
        self.assertEqual(code, 0)
        self.assertEqual((st["commands"], st["files"], st["tokens_in"], st["tokens_out"]), (1, 1, 100, 7))
        self.assertIn("$ ls", shown)

    def test_timeout_marks_the_unit_and_returns_124(self):
        code, shown, st = self.run_job(self.job("sleep 30", timeout=1.5))
        self.assertEqual((code, st["phase"], st["timed_out"]), (124, "done", True))
        self.assertIn("TIMED OUT", shown)

    def test_crash_before_the_capsule_leaves_a_crashed_state(self):
        job = self.job("true")
        job["plan"]["capsule"] = str(self.root / "no-such-request.json")
        code, shown, st = self.run_job(job)
        self.assertEqual((code, st["phase"]), (255, "crashed"))
        self.assertIn("pane crashed", shown)
        self.assertTrue(json.loads((self.root / "units" / "u1" / "result.json").read_text())["crashed"])

    def test_capsule_env_is_the_allowlist_inside_the_pane_job(self):
        code, shown, st = self.run_job(self.job('echo "IN=$CORENIUS_IN_INSULA X=$X"', env={"X": "1"}))
        self.assertIn("IN=1 X=1", shown)

    def test_saved_log_has_no_ansi_codes_even_when_the_pane_is_colored(self):
        buf = io.StringIO()
        job = self.job("echo colored")
        pane.run_job(job, out=buf, color=True)
        self.assertIn("\033[", buf.getvalue())
        self.assertNotIn("\033", (self.root / "units" / "u1" / "pane.log").read_text())

    def test_main_reads_the_job_file(self):
        job = self.job("echo via-main")
        path = self.root / "job.json"
        path.write_text(json.dumps(job))
        self.assertEqual(pane.main(["--job", str(path), "--no-color"]), 0)
        self.assertEqual(state.read(self.root / "units" / "u1" / "state.json")["phase"], "done")


if __name__ == "__main__":
    unittest.main()
