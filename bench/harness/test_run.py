"""run.py and results.py: agents, conditions, result records, aggregation (real capsules, no model calls)."""

from __future__ import annotations

import json
import math
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import results  # noqa: E402
import run  # noqa: E402
import tasklib  # noqa: E402

FIX = """cat > slugger.py <<'PY'
import re


def slugify(value: str) -> str:
    value = re.sub(r"[^a-z0-9]+", "-", value.strip().lower()).strip("-")
    return value or "untitled"
PY"""


class RunTest(unittest.TestCase):
    def setUp(self):
        (tasklib.ROOT / ".scratch").mkdir(exist_ok=True)
        self.tmp = tempfile.TemporaryDirectory(dir=str(tasklib.ROOT / ".scratch"))
        self.out = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def solo(self, task_id, spec, index=0):
        task = tasklib.load_tasks(only=[task_id])[0]
        rec = run.run_solo(task, run.Agent(spec), self.out, index)
        return results.validate(rec)

    def test_reference_passes_and_noop_fails_with_valid_records(self):
        good = self.solo("t01-log-window", "reference")
        bad = self.solo("t01-log-window", "noop")
        self.assertTrue(good["outcome"]["passed"])
        self.assertFalse(bad["outcome"]["passed"])
        self.assertEqual(good["provenance"]["cost"], "unmeasured")
        self.assertIsNone(good["cost"]["usd"])
        self.assertTrue(good["capsule"]["digest"])

    def test_cmd_agent_runs_in_the_capsule_offline_with_the_instruction(self):
        rec = self.solo("p01-slugger", "cmd:" + FIX)
        self.assertTrue(rec["outcome"]["passed"], rec)
        probe = self.solo("p01-slugger", "cmd:test -s /tmp/task/instruction.md && test \"$CORENIUS_IN_INSULA\" = 1 "
                          "&& [ $(tail -n +3 /proc/net/dev | cut -d: -f1 | tr -d ' ' | grep -vc '^lo$') -eq 0 ]")
        self.assertEqual(probe["outcome"]["agent_rc"], 0)

    def test_traecli_requires_explicit_spend_authorization(self):
        with self.assertRaises(SystemExit):
            run.Agent("traecli")
        run.Agent("traecli:GPT-5.5", authorized=True)  # constructing is fine once authorized; nothing is launched

    def test_unknown_agent_is_rejected(self):
        with self.assertRaises(SystemExit):
            run.Agent("magic")

    def test_fleet_reference_records_conflicts_and_redos(self):
        task = tasklib.load_tasks(only=["m03-same-file-append"])[0]
        rec = results.validate(run.run_fleet_condition(task, run.Agent("reference"), self.out, 0, 2))
        self.assertTrue(rec["outcome"]["passed"])
        self.assertEqual((rec["coordination"]["conflicts"], rec["coordination"]["redos"]), (1, 1))
        self.assertEqual(rec["coordination"]["tickets"], 2)

    def test_fleet_needs_a_multi_task(self):
        with self.assertRaises(SystemExit):
            run.run_fleet_condition(tasklib.load_tasks(only=["t01-log-window"])[0], run.Agent("reference"), self.out, 0, 2)

    def test_validate_rejects_bad_records(self):
        rec = self.solo("t01-log-window", "reference")
        for mutate in (lambda r: r.pop("outcome"), lambda r: r["timing"].update(wall_seconds=math.nan),
                       lambda r: r.update(condition="sideways"), lambda r: r["outcome"].update(passed="yes"),
                       lambda r: r.update(schema_version="v0")):
            broken = json.loads(json.dumps(rec, allow_nan=True))
            mutate(broken)
            with self.assertRaises((ValueError, KeyError)):
                results.validate(broken)

    def test_summary_groups_by_task_condition_agent_and_reports_an_interval(self):
        records = [self.solo("t01-log-window", "reference", i) for i in range(2)] + [self.solo("t01-log-window", "noop")]
        rows = {r["agent"]: r for r in results.summarize(records)}
        self.assertEqual((rows["reference"]["n"], rows["reference"]["passed"]), (2, 2))
        self.assertEqual((rows["noop"]["n"], rows["noop"]["pass_rate"]), (1, 0.0))
        lo, hi = rows["reference"]["ci95"]
        self.assertTrue(0 < lo < 1 and hi == 1.0)
        self.assertIn("t01-log-window", results.table(list(rows.values())))

    def test_parse_usage_sums_turns_and_ignores_noise(self):
        text = "\n".join(['{"type":"item.completed"}', "garbage",
                           '{"type":"turn.completed","usage":{"input_tokens":10,"output_tokens":3}}',
                           '{"type":"turn.completed","usage":{"input_tokens":5,"output_tokens":2}}'])
        self.assertEqual(run.parse_usage(text), {"input_tokens": 15, "output_tokens": 5})
        self.assertIsNone(run.parse_usage("no events here"))

    def test_plan_is_json_serializable_and_executing_it_equals_run(self):
        task = tasklib.load_tasks(only=["t01-log-window"])[0]
        agent = run.Agent("reference")
        plan = agent.plan("", task, None, self.out / "w")
        self.assertEqual(json.loads(json.dumps(plan.to_json()))["command"], "sh /tmp/solution/solve.sh")
        record = agent.capsule_record(task)
        ws = self.out / "ws"
        tasklib.materialize(task, ws)
        self.assertEqual(run.Agent.execute(plan, record, ws)["rc"], 0)
        self.assertTrue(tasklib.verify(task, ws, record)["passed"])

    def test_plan_for_cmd_and_traecli_writes_files_into_the_workdir(self):
        task = tasklib.load_tasks(only=["p01-slugger"])[0]
        cmd = run.Agent("cmd:true").plan("do the thing", task, None, self.out / "c")
        self.assertEqual((self.out / "c" / "instruction.md").read_text(), "do the thing")
        self.assertEqual((cmd.net, cmd.gpu), ("none", False))
        tc = run.Agent("traecli:GPT-5.5", authorized=True).plan("do the thing", task, None, self.out / "t")
        self.assertEqual(tc.net, "host")
        self.assertTrue(tc.capsule.endswith("agent.json"))
        self.assertIn("do the thing", (self.out / "t" / "prompt.md").read_text())
        self.assertIn("--skip-git-repo-check", tc.command)

    def test_observer_sees_the_run_phases_in_order(self):
        events = []
        task = tasklib.load_tasks(only=["t01-log-window"])[0]
        rec = run.run_solo(task, run.Agent("reference"), self.out, 0, rid="fixed-run-id",
                           observer=lambda event, **f: events.append((event, f.get("passed"))))
        self.assertEqual(rec["run_id"], "fixed-run-id")
        self.assertEqual(events, [("agents", None), ("verify", None), ("done", True)])

    def test_fleet_log_and_observer_hooks(self):
        logs, events = [], []
        task = tasklib.load_tasks(only=["m03-same-file-append"])[0]
        rec = run.run_fleet_condition(task, run.Agent("reference"), self.out, 0, 2, rid="fleet-id", log=logs.append,
                                      observer=lambda event, **f: events.append(event))
        self.assertTrue(rec["outcome"]["passed"])
        self.assertTrue(any("CONFLICT landing" in m for m in logs))
        self.assertTrue(any("verifier: PASS" in m for m in logs))
        self.assertEqual(events[0], "agents")
        self.assertEqual(events[-1], "done")
        self.assertIn("landing", events)

    def test_fleet_run_saves_the_verifier_tail(self):
        task = tasklib.load_tasks(only=["m01-disjoint-utils"])[0]
        rec = run.run_fleet_condition(task, run.Agent("noop"), self.out, 0, 2)  # nobody fixes anything: the verifier fails
        self.assertFalse(rec["outcome"]["passed"])
        tail = (self.out / rec["run_id"] / "verify-tail.txt").read_text()
        self.assertIn("FAIL", tail)

    def test_persist_writes_a_validated_record(self):
        rec = self.solo("t01-log-window", "reference")
        path = run.persist(rec, self.out / "p")
        self.assertEqual(json.loads(path.read_text())["run_id"], rec["run_id"])
        rec2 = dict(rec)
        rec2.pop("outcome")
        with self.assertRaises(ValueError):
            run.persist(rec2, self.out / "p2")

    def test_wilson_edge_cases(self):
        self.assertEqual(results.wilson(0, 0), (0.0, 1.0))
        lo, hi = results.wilson(0, 5)
        self.assertEqual(lo, 0.0)
        self.assertGreater(hi, 0.3)

    def test_main_writes_result_files_and_report_reads_them(self):
        code = run.main(["t01-log-window", "t02-quoting-archive", "--agent", "reference", "--out", str(self.out / "m")])
        self.assertEqual(code, 0)
        self.assertEqual(len(list((self.out / "m").rglob("result.json"))), 2)
        self.assertEqual(run.main(["--report", str(self.out / "m")]), 0)


if __name__ == "__main__":
    unittest.main()
