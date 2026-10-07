"""Real tmux (isolated server) + real capsules + reference/cmd agents. Skipped without tmux or libtmux."""

from __future__ import annotations

import os
import shutil
import sys
import tempfile
import threading
import time
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
try:
    import libtmux  # noqa: F401
    HAVE = shutil.which("tmux") is not None
except ImportError:
    HAVE = False

ROOT = Path(__file__).resolve().parents[2]
if HAVE:
    import scheduler  # first: it puts the harness and capsule directories on sys.path
    import run
    import state
    import tasklib

_n = [0]


@unittest.skipUnless(HAVE, "needs tmux and libtmux (run with bench/tmuxh/.venv/bin/python)")
class SchedulerTest(unittest.TestCase):
    def setUp(self):
        (ROOT / ".scratch").mkdir(exist_ok=True)
        self.tmp = tempfile.TemporaryDirectory(dir=str(ROOT / ".scratch"))
        self.root = Path(self.tmp.name) / "run"
        self.out = Path(self.tmp.name) / "results"
        _n[0] += 1
        self.socket = f"bench-test-{os.getpid()}-{_n[0]}"
        self.h = None

    def harness(self, slots=4, dashboard=True):
        self.h = scheduler.Harness(self.root, socket=self.socket, slots=slots, dashboard=dashboard).start()
        return self.h

    def tearDown(self):
        if self.h:
            self.h.stop()
        self.tmp.cleanup()

    def task(self, tid):
        return tasklib.load_tasks(only=[tid])[0]

    def windows(self):
        return {w.window_name: w for w in self.h.session.windows}

    def pane_text(self, pane):
        return "\n".join(pane.capture_pane() or [])

    def wait_text(self, pane, needle, timeout=6.0):
        """Pane output lags the state files (tail -F may poll once a second); wait for it."""
        end = time.time() + timeout
        text = self.pane_text(pane)
        while needle not in text and time.time() < end:
            time.sleep(0.3)
            text = self.pane_text(pane)
        return text

    def test_solo_run_gets_one_window_with_one_agent_pane(self):
        h = self.harness()
        rec = h.run_one(self.task("t01-log-window"), "solo", "reference", 0, self.out)
        self.assertTrue(rec["outcome"]["passed"])
        names = self.windows()
        self.assertIn("dashboard", names)
        run_windows = [n for n in names if "t01-log-window-solo-0" in n]
        self.assertEqual(len(run_windows), 1)
        self.assertTrue(run_windows[0].startswith("✓"), run_windows)
        panes = names[run_windows[0]].panes
        self.assertEqual(len(panes), 1)
        self.assertIn("finished rc=0", self.wait_text(panes[0], "finished rc=0"))
        # files: result record, run state, unit state
        self.assertTrue((self.out / rec["run_id"] / "result.json").exists())
        self.assertEqual(state.read(state.run_state_path(self.root, rec["run_id"]))["passed"], True)
        unit = state.read(state.unit_state_path(self.root, rec["run_id"] + "--agent"))
        self.assertEqual((unit["phase"], unit["rc"]), ("done", 0))
        self.assertTrue(unit["pane_id"].startswith("%"))

    def test_fleet_run_has_a_lead_pane_and_a_pane_per_ticket_and_matches_the_direct_runner(self):
        h = self.harness()
        task = self.task("m03-same-file-append")
        rec = h.run_one(task, "fleet:2", "reference", 0, self.out)
        direct = run.run_fleet_condition(task, run.Agent("reference"), Path(self.tmp.name) / "direct", 0, 2)
        for key in ("passed",):
            self.assertEqual(rec["outcome"][key], direct["outcome"][key])
        self.assertEqual((rec["coordination"]["conflicts"], rec["coordination"]["redos"]),
                         (direct["coordination"]["conflicts"], direct["coordination"]["redos"]))
        self.assertEqual((rec["coordination"]["conflicts"], rec["coordination"]["redos"]), (1, 1))
        wins = [w for n, w in self.windows().items() if "m03-same-file-append-fleet2-0" in n]
        self.assertEqual(len(wins), 1)
        panes = wins[0].panes
        self.assertEqual(len(panes), 4)  # lead, t1, t2, and the redo of the conflicting ticket
        lead = self.wait_text(panes[0], "verifier: PASS")
        self.assertIn("CONFLICT landing", lead)
        self.assertIn("verifier: PASS", lead)
        units = [u for u in state.collect(self.root)["units"] if u.get("run_id") == rec["run_id"]]
        self.assertEqual(sorted(u["ticket"] for u in units), ["t1", "t2", "t2-redo1"] if any(u["ticket"] == "t2-redo1" for u in units)
                         else ["t1", "t1-redo1", "t2"])

    def test_slots_bound_concurrent_agents_and_queued_units_are_visible(self):
        h = self.harness(slots=1, dashboard=False)
        task = self.task("t01-log-window")
        seen_queued = []

        def watch():
            end = time.time() + 30
            while time.time() < end:
                units = state.collect(self.root)["units"]
                running = [u for u in units if u.get("phase") == "agent"]
                queued = [u for u in units if u.get("phase") == "queued"]
                self.assertLessEqual(len(running), 1)
                if queued and running:
                    seen_queued.append(1)
                    return
                time.sleep(0.1)

        t = threading.Thread(target=watch)
        t.start()
        recs = h.run_batch([(task, "solo", "cmd:sleep 2", 0), (task, "solo", "cmd:sleep 2", 1)], self.out)
        t.join()
        self.assertEqual(len(recs), 2)
        self.assertTrue(seen_queued, "never saw one agent running while the other was queued")
        self.assertGreater(max(r["timing"]["wall_seconds"] for r in recs), 3.5)  # the second waited for the slot

    def test_a_killed_pane_is_reported_as_crashed_not_hung(self):
        h = self.harness(dashboard=False)
        task = self.task("t01-log-window")
        holder = {}

        def go():
            holder["rec"] = h.run_one(task, "solo", "cmd:sleep 60", 0, self.out)

        t = threading.Thread(target=go)
        t.start()
        pane = None
        for _ in range(100):
            for w in h.session.windows:
                if "t01-log-window-solo-0" in w.window_name and w.panes:
                    pane = w.panes[0]
            if pane is not None and (state.collect(self.root)["units"] or [{}])[0].get("phase") == "agent":
                break
            time.sleep(0.2)
        self.assertIsNotNone(pane)
        pane.cmd("kill-pane")
        t.join(timeout=60)
        self.assertFalse(t.is_alive(), "scheduler hung after the pane was killed")
        rec = holder["rec"]
        self.assertFalse(rec["outcome"]["passed"])
        self.assertEqual(state.collect(self.root)["units"][0]["phase"], "crashed")

    def test_stop_kills_only_the_dedicated_server(self):
        h = self.harness(dashboard=False)
        self.assertTrue(h.server.is_alive())
        self.assertEqual(h.server.socket_name, self.socket)
        self.assertNotEqual(self.socket, "default")
        h.stop()
        self.assertFalse(h.server.is_alive())
        self.assertFalse((Path(f"/run/corenius-tmp/tmux-{os.getuid()}") / self.socket).exists(), "stale socket file left behind")

    def test_every_agent_kind_runs_through_a_pane(self):
        h = self.harness(dashboard=False)
        task = self.task("t01-log-window")
        rec_noop = h.run_one(task, "solo", "noop", 0, self.out)
        rec_cmd = h.run_one(task, "solo", "cmd:test -s /run/corenius-tmp/task/instruction.md", 0, self.out)
        self.assertFalse(rec_noop["outcome"]["passed"])
        self.assertEqual(rec_cmd["outcome"]["agent_rc"], 0)
        with self.assertRaises(SystemExit):
            scheduler.PaneAgent("traecli", h, scheduler.RunCtx("r", "w", "t", "solo", "traecli", False))


if __name__ == "__main__":
    unittest.main()
