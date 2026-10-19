from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import dashboard  # noqa: E402
import state  # noqa: E402

NOW = 1_000_000.0


def snap(runs=(), units=()):
    return {"runs": list(runs), "units": list(units), "taken": NOW}


RUN_DONE_PASS = {"run_id": "r1", "task": "t01-log-window", "condition": "solo", "phase": "done", "passed": True, "wall": 71.4,
                 "created": NOW - 100, "tokens_out": 2500}
RUN_FLEET = {"run_id": "r2", "task": "m03-same-file-append", "condition": "fleet:2", "phase": "landing", "passed": None,
             "created": NOW - 50, "conflicts": 1, "redos": 1}
RUN_FAIL = {"run_id": "r3", "task": "p01-slugger", "condition": "solo", "phase": "done", "passed": False, "wall": 9.0, "created": NOW - 80}
U1 = {"unit": "r1--agent", "run_id": "r1", "phase": "done", "rc": 0, "elapsed": 70.0, "tokens_out": 2500, "commands": 9,
      "last": "finished rc=0 in 70s", "created": NOW - 99}
U2 = {"unit": "r2--t1", "run_id": "r2", "ticket": "t1", "phase": "agent", "started": NOW - 42, "tokens_out": 800, "commands": 3,
      "last": "$ rg -n pricing.py", "created": NOW - 49}
U3 = {"unit": "r2--t2", "run_id": "r2", "ticket": "t2", "phase": "queued", "tokens_out": 0, "commands": 0, "last": "waiting for a free slot",
      "created": NOW - 49}
U4 = {"unit": "r3--agent", "run_id": "r3", "phase": "crashed", "rc": 255, "elapsed": 3.0, "last": "pane died without a result", "created": NOW - 79}


class RenderTest(unittest.TestCase):
    def text(self, s, **kw):
        return "\n".join(t for _, t in dashboard.render(s, 150, now=NOW, **kw))

    def test_empty_state_says_so(self):
        out = self.text(snap())
        self.assertIn("(no runs yet)", out)
        self.assertIn("runs 0", out)

    def test_runs_and_units_with_verdicts_and_columns(self):
        out = self.text(snap([RUN_DONE_PASS, RUN_FLEET, RUN_FAIL], [U1, U2, U3, U4]), slots=4)
        self.assertIn("t01-log-window solo", out)
        self.assertRegex(out, r"t01-log-window solo\s+PASS\s+1:11")
        self.assertRegex(out, r"p01-slugger solo\s+FAIL")
        self.assertRegex(out, r"m03-same-file-append fleet:2\s+landing\s+0:50\s+-\s+1/1")
        self.assertRegex(out, r"└ t1\s+agent\s+0:42\s+800\s+3\s+\$ rg -n pricing.py")
        self.assertRegex(out, r"└ t2\s+queued")
        self.assertIn("pane died without a result", out)
        self.assertIn("runs 3", out)
        self.assertIn("pass 1  fail 1", out)
        self.assertIn("1 running / 4 slots, 1 waiting", out)
        self.assertIn("tokens in 0 out 3,300", out)

    def test_units_are_grouped_under_their_run_and_orphans_still_show(self):
        orphan = {"unit": "x--agent", "run_id": "gone", "phase": "agent", "started": NOW - 1, "last": "?"}
        flat = dashboard.rows(snap([RUN_FLEET], [U2, U3, orphan]))
        self.assertEqual([(r["kind"], r["obj"].get("ticket") or r["obj"].get("task") or r["obj"]["unit"]) for r in flat],
                         [("run", "m03-same-file-append"), ("unit", "t1"), ("unit", "t2"), ("unit", "x--agent")])

    def test_selection_and_log_tail(self):
        lines = dashboard.render(snap([RUN_FLEET], [U2]), 150, selected=1, now=NOW, log_tail=["line a", "line b"])
        styles = {t.strip()[:6]: s for s, t in lines}
        self.assertEqual([s for s, t in lines if "└ t1" in t], ["sel"])
        self.assertTrue(any("log tail" in t for _, t in lines))
        self.assertTrue(any("line b" in t for _, t in lines))

    def test_width_is_respected_and_long_text_is_clipped(self):
        u = dict(U2, last="x" * 500)
        for width in (80, 120):
            for _, t in dashboard.render(snap([RUN_FLEET], [u]), width, now=NOW):
                self.assertLessEqual(len(t), width)

    def test_fmt_secs(self):
        self.assertEqual((dashboard.fmt_secs(None), dashboard.fmt_secs(5), dashboard.fmt_secs(125), dashboard.fmt_secs(3725)),
                         ("-", "0:05", "2:05", "1:02:05"))

    def test_once_mode_reads_files_and_prints_plain_text(self):
        import io
        from contextlib import redirect_stdout
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            state.update(state.run_state_path(root, "r1"), **RUN_DONE_PASS)
            state.update(state.unit_state_path(root, "r1--agent"), **U1)
            buf = io.StringIO()
            with redirect_stdout(buf):
                self.assertEqual(dashboard.main(["--root", d, "--once"]), 0)
            self.assertIn("t01-log-window solo", buf.getvalue())
            self.assertNotIn("\033", buf.getvalue())


if __name__ == "__main__":
    unittest.main()
