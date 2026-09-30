"""fleet.py: landing, conflicts, contamination, and multi-task validation (real bwrap and git)."""

from __future__ import annotations

import json
import os
import shutil
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fleet  # noqa: E402
import tasklib  # noqa: E402
import validate  # noqa: E402


def load(tid, root=None):
    return tasklib.load_tasks(root or tasklib.TASKS, only=[tid])[0]


class FleetTest(unittest.TestCase):
    def setUp(self):
        (tasklib.ROOT / ".scratch").mkdir(exist_ok=True)
        self.tmp = tempfile.TemporaryDirectory(dir=str(tasklib.ROOT / ".scratch"))
        self.root = Path(self.tmp.name)
        os.environ["CORENIUS_CAPSULES"] = str(self.root / "store")
        req = self.root / "req.json"
        req.write_text(json.dumps({"name": "t", "layers": [{"kind": "base"}], "policy": {"net": "none"}}))
        self.record = tasklib.capsule.build(str(req))

    def tearDown(self):
        os.environ.pop("CORENIUS_CAPSULES", None)
        self.tmp.cleanup()

    def test_disjoint_tickets_land_cleanly_in_any_order(self):
        for order in (["t1", "t2"], ["t2", "t1"]):
            run = fleet.run_fleet(load("m01-disjoint-utils"), fleet.reference_agent, self.record, order=order)
            self.assertTrue(run["passed"])
            self.assertEqual((run["conflicts"], run["redos"], run["contamination"]), (0, 0, []))

    def test_same_place_append_conflicts_once_and_redo_recovers(self):
        run = fleet.run_fleet(load("m03-same-file-append"), fleet.reference_agent, self.record, order=["t1", "t2"])
        self.assertEqual((run["conflicts"], run["redos"]), (1, 1))
        self.assertTrue(run["passed"])
        second = run["landings"][1]
        self.assertEqual(second["first_conflict_files"], ["pricing.py"])
        self.assertTrue(second["landed"])

    def test_without_redo_the_conflicting_ticket_is_lost_and_the_task_fails(self):
        run = fleet.run_fleet(load("m03-same-file-append"), fleet.reference_agent, self.record, order=["t1", "t2"], redo=False)
        self.assertEqual(run["conflicts"], 1)
        self.assertFalse(run["landings"][1]["landed"])
        self.assertFalse(run["passed"])

    def test_workers_run_tickets_concurrently(self):
        import time

        def slow(ticket, ws, record):
            time.sleep(1.5)
            return fleet.reference_agent(ticket, ws, record)

        task = load("m01-disjoint-utils")
        seq = fleet.run_fleet(task, slow, self.record, workers=1)
        par = fleet.run_fleet(task, slow, self.record, workers=2)
        self.assertTrue(seq["passed"] and par["passed"])
        self.assertGreater(seq["wall_seconds"], 2.9)
        self.assertLess(par["wall_seconds"], seq["wall_seconds"] - 1.0)

    def test_noop_agent_fails_the_task(self):
        run = fleet.run_fleet(load("m01-disjoint-utils"), lambda t, ws, rec: {"rc": 0}, self.record)
        self.assertFalse(run["passed"])

    def test_contamination_is_reported_and_not_landed(self):
        def sloppy(ticket, ws, record):
            out = fleet.reference_agent(ticket, ws, record)
            (ws / "stray.txt").write_text("x\n")
            (ws / "numbers_util.py").write_text((ws / "numbers_util.py").read_text() + "# touched\n")
            return out

        run = fleet.run_fleet(load("m01-disjoint-utils"), sloppy, self.record, order=["t1", "t2"])
        self.assertIn("stray.txt", run["contamination"])
        self.assertIn("numbers_util.py", run["contamination"])  # t1 may only change strings_util.py
        self.assertTrue(run["passed"])  # allowed changes still landed
        self.assertEqual(run["conflicts"], 0)
        self.assertTrue(all("error" not in l for l in run["landings"]), run["landings"])

    def test_fleet_never_runs_the_agent_outside_the_capsule_marker(self):
        seen = {}

        def probe(ticket, ws, record):
            seen.update(tasklib.run_in_capsule(record, ws, "test \"$CORENIUS_IN_INSULA\" = 1", {}, "none", 30))
            return seen

        fleet.run_fleet(load("m01-disjoint-utils"), probe, self.record)
        self.assertEqual(seen["rc"], 0)

    def test_check_multi_flags_a_wrong_conflict_declaration(self):
        copy = self.root / "tasks"
        shutil.copytree(tasklib.TASKS / "m03-same-file-append", copy / "m03-same-file-append")
        toml = copy / "m03-same-file-append" / "task.toml"
        toml.write_text(toml.read_text().replace("expected_conflicts = 1", "expected_conflicts = 0"))
        result = validate.check_multi(load("m03-same-file-append", copy), self.record)
        self.assertFalse(result["orders-land"]["ok"])
        self.assertTrue(result["tickets-necessary"]["ok"])

    def test_check_multi_flags_an_unnecessary_ticket(self):
        copy = self.root / "tasks"
        shutil.copytree(tasklib.TASKS / "m01-disjoint-utils", copy / "m01-disjoint-utils")
        # make t2 a no-op: the final task then passes without it, so the ticket is not necessary
        (copy / "m01-disjoint-utils/environment/numbers_util.py").write_text((copy / "m01-disjoint-utils/solution/overlay/numbers_util.py").read_text())
        result = validate.check_multi(load("m01-disjoint-utils", copy), self.record)
        self.assertFalse(result["tickets-necessary"]["ok"])


if __name__ == "__main__":
    unittest.main()
