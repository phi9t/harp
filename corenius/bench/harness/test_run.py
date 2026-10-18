"""run.py and results.py: agents, conditions, result records, aggregation (real capsules, no model calls)."""

from __future__ import annotations

import json
import math
import io
import os
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest import mock
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import controller_mode  # noqa: E402
import fleet  # noqa: E402
import results  # noqa: E402
import run  # noqa: E402
import tasklib  # noqa: E402

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)

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
        probe = self.solo("p01-slugger", "cmd:test -s /run/corenius-tmp/task/instruction.md && test \"$CORENIUS_IN_INSULA\" = 1 "
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

    def test_fleet_git_commands_are_noninteractive_and_bounded(self):
        with mock.patch.object(fleet.subprocess, "run") as fake_run:
            fake_run.side_effect = subprocess.TimeoutExpired(["git", "status"], timeout=0.001)
            proc = fleet.git(self.out, "status", check=False)

        self.assertEqual(proc.returncode, 124)
        self.assertIn("git status timed out", proc.stderr)
        self.assertEqual(fake_run.call_args.kwargs["timeout"], fleet.GIT_TIMEOUT_SECONDS)
        self.assertEqual(fake_run.call_args.kwargs["env"]["GIT_TERMINAL_PROMPT"], "0")
        self.assertEqual(fake_run.call_args.kwargs["env"]["GIT_EDITOR"], "true")

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
        self.assertEqual(json.loads(json.dumps(plan.to_json()))["command"], tasklib.SOLUTION_COMMAND)
        record = agent.capsule_record(task)
        ws = self.out / "ws"
        tasklib.materialize(task, ws)
        self.assertEqual(run.Agent.execute(plan, record, ws)["rc"], 0)
        self.assertTrue(tasklib.verify(task, ws, record)["passed"])

    def test_plan_for_cmd_and_traecli_writes_files_into_the_workdir(self):
        task = tasklib.load_tasks(only=["p01-slugger"])[0]
        task.agent["env"] = {"EXAMPLE": "1"}
        task.agent["net"] = "host"
        cmd = run.Agent("cmd:true").plan("do the thing", task, None, self.out / "c")
        self.assertEqual((self.out / "c" / "instruction.md").read_text(), "do the thing")
        self.assertEqual((cmd.net, cmd.gpu), ("host", False))
        self.assertEqual(cmd.env["EXAMPLE"], "1")
        self.assertEqual(cmd.env["TASK_INSTRUCTION"], "/run/corenius-tmp/task/instruction.md")
        reference = run.Agent("reference").plan("", task, None, self.out / "r")
        self.assertEqual(reference.net, "host")
        tc = run.Agent("traecli:GPT-5.5", authorized=True).plan("do the thing", task, None, self.out / "t")
        self.assertEqual(tc.net, "host")
        self.assertTrue(tc.capsule.endswith("agent.json"))
        self.assertEqual(tc.env["EXAMPLE"], "1")
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

    def test_controller_reference_runs_one_m01_ticket_through_controller(self):
        out = self.out / "controller"
        code = run.main(["m01-disjoint-utils", "--condition", "controller:t1", "--agent", "reference", "--out", str(out)])
        self.assertEqual(code, 0)
        records = list(out.rglob("result.json"))
        self.assertEqual(len(records), 1)
        rec = results.validate(json.loads(records[0].read_text()))
        self.assertTrue(rec["outcome"]["passed"], rec)
        self.assertEqual(rec["condition"], "controller:t1")
        self.assertEqual(rec["coordination"]["tickets"], 1)
        self.assertEqual(rec["coordination"]["tickets_landed"], 1)
        self.assertEqual(rec["coordination"]["condition_mode"], "controller")
        controller = rec["controller"]
        self.assertEqual(controller["schema_version"], "corenius.bench.controller.v2")
        self.assertEqual(controller["mode"], "deterministic")
        self.assertEqual(controller["task"], "m01-disjoint-utils")
        self.assertEqual(controller["ticket"], "t1")
        self.assertEqual(controller["execution_window"]["deadline_seconds"], 1800.0)
        self.assertEqual(
            controller["execution_window"]["dispatch_cutoff_unix"],
            controller["execution_window"]["durable_run_admission_unix"] + 1800.0,
        )
        self.assertFalse(controller["execution_window"]["automatic_rerun_after_timeout_or_provider_failure"])
        self.assertEqual(controller["partial_contract"]["suite_id"], "m01-disjoint-utils:t1:partial")
        self.assertEqual(controller["partial_contract"]["selected_tests"], ["T.test_title_keeps_small_words_lower_and_collapses_spaces"])
        self.assertEqual(controller["partial_contract"]["noop_verdict"], "fail")
        self.assertEqual(controller["partial_contract"]["fixed_verdict"], "pass")
        self.assertTrue(controller["partial_contract"]["complete_original_test_file_restored"])
        self.assertEqual(controller["landing"]["state"], "landed")
        self.assertRegex(controller["landing"]["commit"], r"^[0-9a-f]{40}$")
        self.assertEqual(controller["process_disposition"]["owned_attempts"], "stopped")
        self.assertEqual(len(controller["process_disposition"]["attempts"]), 2)
        self.assertGreater(rec["timing"]["agent_seconds"], 0)
        self.assertGreater(rec["timing"]["verify_seconds"], 0)
        self.assertGreater(rec["timing"]["build_seconds"], 0)
        self.assertGreater(rec["timing"]["setup_seconds"], rec["timing"]["build_seconds"])
        self.assertGreater(rec["timing"]["wall_seconds"], rec["timing"]["setup_seconds"])
        self.assertEqual(rec["provenance"]["timing"], "measured by harness phase timers")
        manifest_paths = {entry["path"] for entry in controller["source"]["content_manifest"]["entries"]}
        self.assertIn("bench/harness/controller_mode.py", manifest_paths)
        self.assertIn("bench/harness/run.py", manifest_paths)
        self.assertTrue(controller["run_root"].startswith(os.environ.get("TMPDIR", "/run/corenius-tmp")))
        self.assertEqual(controller["runtime_closure"]["status"], "pinned")
        self.assertIn("verifier_runtime", controller["runtime_closure"]["pinned"])
        runtime = controller["runtime_closure"]["pinned"]["verifier_runtime"]
        self.assertRegex(runtime["manifest"]["sha256"], r"^[0-9a-f]{64}$")
        self.assertEqual(controller["verification"]["document"]["runtime"]["manifest"], runtime["manifest"]["sha256"])
        self.assertNotIn("limitation", controller["runtime_closure"])

    def test_controller_reference_runs_numeric_m01_ticket_through_controller(self):
        out = self.out / "controller-t2"
        code = run.main(["m01-disjoint-utils", "--condition", "controller:t2", "--agent", "reference", "--out", str(out)])
        self.assertEqual(code, 0)
        records = list(out.rglob("result.json"))
        self.assertEqual(len(records), 1)
        rec = results.validate(json.loads(records[0].read_text()))
        self.assertTrue(rec["outcome"]["passed"], rec)
        self.assertEqual(rec["condition"], "controller:t2")
        self.assertEqual(rec["coordination"]["condition_mode"], "controller")
        controller = rec["controller"]
        self.assertEqual(controller["ticket"], "t2")
        self.assertEqual(controller["partial_contract"]["suite_id"], "m01-disjoint-utils:t2:partial")
        self.assertEqual(controller["partial_contract"]["selected_tests"], ["T.test_clamp_both_ends"])
        self.assertEqual(controller["partial_contract"]["noop_verdict"], "fail")
        self.assertEqual(controller["partial_contract"]["fixed_verdict"], "pass")
        self.assertEqual(controller["comparison"]["changed_paths"], ["numbers_util.py"])
        self.assertEqual(controller["landing"]["state"], "landed")

    def test_controller_complete_m01_records_overlap_stale_refusal_and_fresh_integration(self):
        out = self.out / "controller-complete"
        code = run.main(["m01-disjoint-utils", "--condition", "controller:complete", "--agent", "reference", "--out", str(out)])
        self.assertEqual(code, 0)
        records = list(out.rglob("result.json"))
        self.assertEqual(len(records), 1)
        rec = results.validate(json.loads(records[0].read_text()))
        self.assertTrue(rec["outcome"]["passed"], rec)
        self.assertEqual(rec["condition"], "controller:complete")
        self.assertEqual(rec["outcome"]["task_verdict"], "pass")
        self.assertEqual(rec["outcome"]["orchestration_verdict"], "pass")
        self.assertEqual(rec["coordination"]["condition_mode"], "controller")
        self.assertEqual(rec["coordination"]["tickets"], 2)
        self.assertEqual(rec["coordination"]["tickets_landed"], 2)
        self.assertEqual(rec["coordination"]["stale_refusals"], 1)
        self.assertEqual(rec["coordination"]["redos"], 1)
        self.assertEqual(rec["coordination"]["attempts"], 3)

        controller = rec["controller"]
        self.assertEqual(controller["ticket"], "complete")
        self.assertEqual(controller["complete_contract"]["suite_id"], "m01-disjoint-utils:complete")
        self.assertEqual(
            controller["complete_contract"]["selected_tests"],
            ["T.test_title_keeps_small_words_lower_and_collapses_spaces", "T.test_clamp_both_ends"],
        )
        self.assertEqual(controller["complete_contract"]["final_verdict"], "pass")
        self.assertEqual(controller["complete_contract"]["omitted_controls"], {"t1_only": "fail", "t2_only": "fail"})
        self.assertEqual(controller["complete_contract"]["tampered_tests_control"], "fail")
        self.assertTrue(controller["complete_contract"]["complete_original_test_file_restored"])
        runtime_digest = controller["runtime_closure"]["pinned"]["verifier_runtime"]["manifest"]["sha256"]
        script_digest = controller["runtime_closure"]["pinned"]["complete_verification_adapter"]["sha256"]
        for proof in [
            controller["complete_contract"]["final_external"],
            controller["complete_contract"]["omitted_control_evidence"]["t1_only"],
            controller["complete_contract"]["omitted_control_evidence"]["t2_only"],
            controller["complete_contract"]["tampered_tests_evidence"],
        ]:
            self.assertRegex(proof["source_commit"], r"^[0-9a-f]{40}$")
            self.assertRegex(proof["source_tree"], r"^[0-9a-f]{40}$")
            self.assertRegex(proof["source_view"]["digest"], r"^[0-9a-f]{64}$")
            self.assertRegex(proof["verifier_view"]["digest"], r"^[0-9a-f]{64}$")
            self.assertEqual(proof["suite"]["id"], "m01-disjoint-utils:complete")
            self.assertRegex(proof["suite"]["digest"], r"^[0-9a-f]{64}$")
            self.assertEqual(proof["script"]["sha256"], script_digest)
            self.assertEqual(proof["runtime"]["manifest"]["sha256"], runtime_digest)
            self.assertEqual(proof["runtime"]["status"], "pinned")
            log = json.loads(Path(proof["log"]["path"]).read_text())
            self.assertEqual(log["argv"][0], proof["tools"]["python"]["path"])
            self.assertEqual(log["tools"]["python"]["path"], proof["tools"]["python"]["path"])
        self.assertNotEqual(
            controller["complete_contract"]["tampered_tests_evidence"]["source_view"]["digest"],
            controller["complete_contract"]["tampered_tests_evidence"]["verifier_view"]["digest"],
        )

        overlap = controller["useful_execution_overlap"]
        self.assertGreater(overlap["seconds"], 0)
        self.assertEqual({entry["ticket"] for entry in overlap["intervals"]}, {"t1", "t2"})
        self.assertEqual({entry["base"] for entry in overlap["intervals"]}, {controller["git"]["base"]})
        self.assertTrue(all(entry["candidate"] and entry["commit"] for entry in overlap["intervals"]))
        self.assertEqual(controller["landing_sequence"][0]["state"], "landed")
        self.assertEqual(controller["stale_landing"]["state"], "stale_base")
        self.assertEqual(controller["fresh_integration"]["attempt_count"], 1)
        self.assertEqual(controller["fresh_integration"]["base"], controller["landing_sequence"][0]["commit"])
        self.assertEqual(controller["fresh_integration"]["old_proof_for_fresh_candidate"], "rejected")
        self.assertEqual(controller["fresh_integration"]["verification"]["verdict"], "pass")
        self.assertEqual(controller["fresh_integration"]["acceptance"]["candidate"], controller["fresh_integration"]["candidate"]["candidate"])
        self.assertEqual(controller["git"]["final"], controller["fresh_integration"]["candidate"]["commit"])
        self.assertEqual(controller["process_disposition"]["owned_attempts"], "stopped")
        self.assertEqual(len(controller["process_disposition"]["attempts"]), 3)
        self.assertNotIn("stale acceptance", " ".join(controller["qualification_limits"]).lower())

    def minimal_controller_record(self, run_id="controller-v2", passed=True):
        verdict = "pass" if passed else "fail"
        return {
            "schema_version": results.SCHEMA,
            "run_id": run_id,
            "task_id": "m01-disjoint-utils",
            "condition": "controller:complete",
            "agent": {"spec": "reference"},
            "capsule": {"digest": "capsule", "net": "none"},
            "outcome": {
                "passed": passed,
                "reward": 1.0 if passed else 0.0,
                "verifier_rc": 0 if passed else 1,
                "timed_out": False,
                "agent_rc": 0,
                "task_verdict": verdict,
                "orchestration_verdict": verdict,
                "effect_disposition": "resolved",
                "failure_stage": None,
            },
            "timing": {
                "wall_seconds": 10.0,
                "agent_seconds": 3.0,
                "verify_seconds": 2.0,
                "source_capture_seconds": 0.1,
                "build_seconds": 1.0,
                "setup_seconds": 2.0,
                "execution_seconds": 6.0,
                "controller_flow_seconds": 6.0,
                "cleanup_seconds": 1.0,
            },
            "coordination": {
                "conflicts": 0,
                "stale_refusals": 1,
                "redos": 1,
                "contamination": 0,
                "tickets": 2,
                "tickets_landed": 2,
                "condition_mode": "controller",
                "attempts": 3,
                "manual_interventions": 0,
            },
            "cost": {"usd": None, "input_tokens": None, "output_tokens": None},
            "provenance": {
                "outcome": "controller measured",
                "timing": "measured by harness phase timers",
                "coordination": "controller measured",
                "cost": "not_applicable; deterministic no-model mode",
            },
            "controller": {
                "schema_version": "corenius.bench.controller.v2",
                "mode": "deterministic",
                "task": "m01-disjoint-utils",
                "ticket": "complete",
                "run_root": str(self.out / "retained-run"),
                "execution_window": {
                    "clock": "durable_admission_to_final_verification",
                    "deadline_seconds": 1800.0,
                    "setup_timeout_seconds": 300.0,
                    "cleanup_timeout_seconds": 60.0,
                    "pinned_verifier_timeout_seconds": 120.0,
                    "durable_run_admission_unix": 1000.0,
                    "first_mission_submission_unix": 1000.1,
                    "dispatch_cutoff_unix": 2800.0,
                    "failed_preflight_consumed_model_calls": False,
                    "automatic_rerun_after_timeout_or_provider_failure": False,
                    "live_host_requirement": "m01",
                    "platform": "linux",
                },
                "outcomes": {
                    "task": {"verdict": verdict},
                    "orchestration": {"verdict": verdict},
                    "effects": {"disposition": "resolved"},
                },
                "failure": {"stage": None, "kind": None, "message": None},
                "interventions": [],
                "attempts": {
                    "total": 3,
                    "initial": 2,
                    "fresh_integration": 1,
                    "max_fresh_integration_attempts": 1,
                    "automatic_retries_after_failure": 0,
                    "max_concurrent_worker_executions": 2,
                },
                "model_usage": {
                    "status": "not_applicable",
                    "input_tokens": None,
                    "output_tokens": None,
                    "usd": None,
                    "complete": False,
                },
                "retained_evidence": {
                    "run_root": str(self.out / "retained-run"),
                    "repositories": [{"kind": "canonical", "path": str(self.out / "canonical"), "head": "0" * 40}],
                    "manifests": [{"kind": "source", "path": str(self.out / "source-manifest.json"), "sha256": "a" * 64}],
                    "events": [{"kind": "controller", "first_seq": 1, "last_seq": 10}],
                    "proofs": [{"kind": "final", "sha256": "b" * 64}],
                    "logs": [{"kind": "summary", "path": str(self.out / "summary.log"), "sha256": "c" * 64}],
                },
                "partial_contract": {
                    "suite_id": "multiple",
                    "suite_digest": "multiple",
                    "selected_tests": ["T.test_title_keeps_small_words_lower_and_collapses_spaces", "T.test_clamp_both_ends"],
                    "noop_verdict": "not_run",
                    "fixed_verdict": verdict,
                },
                "landing": {"state": "landed" if passed else "not_landed"},
                "process_disposition": {"owned_attempts": "stopped", "services": "stopped", "effect_disposition": "resolved"},
                "source": {
                    "content_manifest": {
                        "schema_version": "corenius.bench.source-manifest.v1",
                        "digest": "d" * 64,
                        "entries": [],
                    },
                },
                "runtime_closure": {
                    "status": "pinned",
                    "pinned": {"verifier_runtime": {"manifest": {"sha256": "e" * 64}}},
                },
                "qualification_limits": ["no one-run speed, cost or recovery superiority claim"],
                "orchestration_passed": passed,
            },
        }

    def test_controller_v2_result_records_independent_outcomes_deadlines_usage_and_evidence(self):
        rec = results.validate(self.minimal_controller_record())
        self.assertTrue(rec["outcome"]["passed"])
        controller = rec["controller"]
        self.assertEqual(controller["schema_version"], "corenius.bench.controller.v2")
        self.assertEqual(controller["execution_window"]["deadline_seconds"], 1800.0)
        self.assertEqual(
            controller["execution_window"]["dispatch_cutoff_unix"],
            controller["execution_window"]["durable_run_admission_unix"] + 1800.0,
        )
        self.assertEqual(rec["timing"]["execution_seconds"], rec["timing"]["controller_flow_seconds"])
        self.assertEqual(controller["outcomes"]["task"]["verdict"], "pass")
        self.assertEqual(controller["outcomes"]["orchestration"]["verdict"], "pass")
        self.assertEqual(controller["outcomes"]["effects"]["disposition"], "resolved")
        self.assertEqual(controller["attempts"]["fresh_integration"], 1)
        self.assertEqual(controller["model_usage"]["status"], "not_applicable")
        self.assertIsNone(controller["model_usage"]["input_tokens"])
        self.assertTrue(controller["retained_evidence"]["manifests"][0]["sha256"])

    def test_controller_v2_rejects_autonomous_pass_that_contradicts_failure_state(self):
        rec = self.minimal_controller_record(passed=False)
        rec["outcome"]["passed"] = True
        rec["outcome"]["reward"] = 1.0
        rec["outcome"]["failure_stage"] = "timeout"
        rec["outcome"]["timed_out"] = True
        rec["controller"]["failure"] = {"stage": "timeout", "kind": "timeout", "message": "elapsed"}
        rec["controller"]["outcomes"]["task"]["verdict"] = "unknown"
        rec["controller"]["outcomes"]["effects"]["disposition"] = "unresolved"
        rec["outcome"]["task_verdict"] = "unknown"
        rec["outcome"]["effect_disposition"] = "unresolved"
        with self.assertRaisesRegex(ValueError, "autonomous PASS"):
            results.validate(rec)

    def test_controller_v2_permits_dispatch_failure_resolved_after_proven_cleanup(self):
        rec = self.minimal_controller_record(passed=False)
        rec["outcome"].update(
            timed_out=True,
            task_verdict="unknown",
            orchestration_verdict="fail",
            effect_disposition="resolved",
            failure_stage="timeout",
        )
        rec["controller"]["failure"] = {"stage": "timeout", "kind": "timeout", "message": "elapsed"}
        rec["controller"]["outcomes"] = {
            "task": {"verdict": "unknown"},
            "orchestration": {"verdict": "fail"},
            "effects": {"disposition": "resolved"},
        }
        rec["controller"]["process_disposition"] = {
            "owned_attempts": "stopped",
            "services": "stopped",
            "effect_disposition": "resolved",
            "remaining_uncertain": [],
            "reconciliation": {
                "status": "resolved",
                "authority": "controller.cancel_attempt",
                "bounded": True,
                "identity_bound": True,
                "cancel_attempts": [{"attempt": "attempt-1", "status": "cancelled"}],
            },
        }
        self.assertIs(results.validate(rec), rec)

    def timeout_failure_record(self, index: int = 0, stage: str = "timeout") -> dict:
        rec = self.minimal_controller_record(run_id=f"controller-timeout-repeat-{index}", passed=False)
        rec["condition"] = "controller:t1"
        rec["controller"]["ticket"] = "t1"
        rec["coordination"].update(stale_refusals=0, redos=0, tickets=1, tickets_landed=0, attempts=1)
        rec["controller"]["attempts"].update(total=1, initial=1, fresh_integration=0)
        rec["outcome"].update(
            timed_out=stage == "timeout",
            task_verdict="unknown",
            orchestration_verdict="fail",
            effect_disposition="unresolved",
            failure_stage=stage,
        )
        rec["controller"]["failure"] = {"stage": stage, "kind": stage, "message": f"{stage} failure"}
        rec["controller"]["outcomes"] = {
            "task": {"verdict": "unknown"},
            "orchestration": {"verdict": "fail"},
            "effects": {"disposition": "unresolved"},
        }
        rec["controller"]["partial_contract"]["suite_id"] = "not_run"
        rec["controller"]["partial_contract"]["suite_digest"] = "not_run"
        rec["controller"]["partial_contract"]["selected_tests"] = []
        rec["controller"]["partial_contract"]["noop_verdict"] = "not_run"
        rec["controller"]["partial_contract"]["fixed_verdict"] = "unknown"
        rec["controller"]["landing"] = {"state": "not_landed"}
        rec["controller"]["process_disposition"] = {
            "owned_attempts": "unresolved",
            "services": "stopped",
            "effect_disposition": "unresolved",
            "remaining_uncertain": [{"kind": "attempt", "id": "attempt-1"}],
        }
        return rec

    def test_controller_repeat_stops_after_timeout_failure(self):
        calls = []
        original = run.controller_mode.run_controller_condition

        def fake_controller(task, agent, out, index, ticket_id, **kwargs):
            calls.append(index)
            return self.timeout_failure_record(index=index, stage="timeout")

        try:
            run.controller_mode.run_controller_condition = fake_controller
            code = run.main([
                "m01-disjoint-utils",
                "--condition",
                "controller:t1",
                "--agent",
                "reference",
                "--repeat",
                "3",
                "--out",
                str(self.out / "repeat-timeout"),
            ])
        finally:
            run.controller_mode.run_controller_condition = original
        self.assertEqual(code, 0)
        self.assertEqual(calls, [0])

    def test_controller_repeat_stops_after_provider_failure(self):
        calls = []
        original = run.controller_mode.run_controller_condition

        def fake_controller(task, agent, out, index, ticket_id, **kwargs):
            calls.append(index)
            return self.timeout_failure_record(index=index, stage="provider_failure")

        try:
            run.controller_mode.run_controller_condition = fake_controller
            code = run.main([
                "m01-disjoint-utils",
                "--condition",
                "controller:t1",
                "--agent",
                "reference",
                "--repeat",
                "3",
                "--out",
                str(self.out / "repeat-provider-failure"),
            ])
        finally:
            run.controller_mode.run_controller_condition = original
        self.assertEqual(code, 0)
        self.assertEqual(calls, [0])

    def assert_controller_fault_is_loadable(self, fault: str, stage: str):
        out = self.out / f"controller-{fault}"
        code = run.main([
            "m01-disjoint-utils",
            "--condition",
            "controller:t1",
            "--agent",
            "reference",
            "--controller-lifecycle-fault",
            fault,
            "--out",
            str(out),
        ])
        self.assertEqual(code, 0)
        loaded = results.load_all(out)
        self.assertEqual(len(loaded), 1)
        rec = loaded[0]
        self.assertFalse(rec["outcome"]["passed"], rec)
        self.assertEqual(rec["outcome"]["failure_stage"], stage)
        self.assertEqual(rec["controller"]["failure"]["stage"], stage)
        self.assertTrue(rec["controller"]["retained_evidence"]["manifests"])
        self.assertTrue(rec["controller"]["retained_evidence"]["logs"])
        closure = rec["controller"]["runtime_closure"]
        self.assertIn(closure["status"], {"limited", "pinned"})
        if closure["status"] == "pinned":
            self.assertIn("verifier_runtime", closure["pinned"])
        return rec

    def assert_retained_controller_events_are_actual(self, rec: dict, required_kinds: set[str]):
        events = rec["controller"]["retained_evidence"]["events"]
        controller_events = [
            item for item in events
            if item.get("kind") == "controller" and item.get("path") and item.get("status") != "unknown"
        ]
        self.assertTrue(controller_events, events)
        retained = controller_events[0]
        self.assertGreater(retained["count"], 1)
        self.assertEqual(retained["first_seq"], 1)
        self.assertGreaterEqual(retained["last_seq"], retained["first_seq"])
        path = Path(retained["path"])
        self.assertTrue(path.is_file(), retained)
        self.assertEqual(retained["sha256"], controller_mode.sha256_bytes(path.read_bytes()))
        document = json.loads(path.read_text())
        actual = document["events"]
        self.assertEqual(len(actual), retained["count"])
        self.assertEqual(actual[0]["seq"], retained["first_seq"])
        self.assertEqual(actual[-1]["seq"], retained["last_seq"])
        kinds = {event["kind"] for event in actual}
        self.assertTrue(required_kinds.issubset(kinds), kinds)

    def test_controller_missing_prerequisite_fault_persists_loadable_preflight_result(self):
        rec = self.assert_controller_fault_is_loadable("missing-prerequisite", "missing_prerequisite")
        self.assertEqual(rec["outcome"]["task_verdict"], "unknown")
        self.assertFalse(rec["controller"]["execution_window"]["failed_preflight_consumed_model_calls"])
        self.assertTrue(rec["controller"]["failure_details"]["missing_paths"])
        self.assertEqual(rec["controller"]["runtime_closure"]["status"], "limited")

    def test_controller_provider_failure_fault_persists_loadable_result_with_reconciliation(self):
        rec = self.assert_controller_fault_is_loadable("provider-failure", "provider_failure")
        self.assertEqual(rec["outcome"]["task_verdict"], "unknown")
        self.assertIn(rec["outcome"]["effect_disposition"], {"resolved", "unresolved"})
        self.assertGreaterEqual(rec["coordination"]["attempts"], 1)
        self.assertIn("reconciliation", rec["controller"]["process_disposition"])
        self.assertEqual(rec["controller"]["failure_details"]["boundary"], "worker useful checkpoint")
        self.assertTrue(rec["controller"]["failure_details"]["attempt_records"])
        self.assert_retained_controller_events_are_actual(
            rec,
            {"set_mission", "admit", "observe_attempt"},
        )

    def test_controller_delivery_failure_fault_persists_loadable_result(self):
        rec = self.assert_controller_fault_is_loadable("delivery-failure", "delivery_failure")
        self.assertEqual(rec["outcome"]["task_verdict"], "unknown")
        self.assertEqual(rec["controller"]["failure_details"]["boundary"], "kata issue delivery")
        self.assertIn("transport_error", rec["controller"]["failure_details"])

    def test_controller_generic_failure_fault_persists_loadable_result(self):
        rec = self.assert_controller_fault_is_loadable("generic-failure", "generic_failure")
        self.assertEqual(rec["outcome"]["task_verdict"], "unknown")
        self.assertEqual(rec["controller"]["failure_details"]["exception_type"], "RuntimeError")

    def test_controller_manual_intervention_fault_records_actual_observation(self):
        rec = self.assert_controller_fault_is_loadable("manual-intervention", "manual_intervention")
        self.assertEqual(rec["coordination"]["manual_interventions"], 1)
        self.assertFalse(rec["controller"]["autonomous_pass_eligible"])
        intervention = rec["controller"]["interventions"][0]
        self.assertEqual(intervention["kind"], "manual_pause_dispatch")
        self.assertEqual(intervention["observation"]["paused"], True)
        self.assertTrue(any(item.get("kind") == "controller" for item in rec["controller"]["retained_evidence"]["events"]))

    def test_controller_source_capture_timeout_persists_preflight_result(self):
        original = controller_mode.source_identity

        def fail_source_identity(*args, **kwargs):
            raise subprocess.TimeoutExpired(["git", "status"], timeout=0.001)

        try:
            controller_mode.source_identity = fail_source_identity
            task = tasklib.load_tasks(only=["m01-disjoint-utils"])[0]
            rec = controller_mode.run_controller_condition(
                task,
                run.Agent("reference"),
                self.out / "source-timeout",
                0,
                "t1",
            )
        finally:
            controller_mode.source_identity = original
        rec = results.validate(rec)
        self.assertEqual(rec["outcome"]["failure_stage"], "preflight")
        self.assertEqual(rec["controller"]["failure_details"]["exception_type"], "TimeoutExpired")
        self.assertEqual(rec["controller"]["runtime_closure"]["status"], "limited")

    def test_controller_failure_fixtures_never_autonomous_pass_and_keep_unknowns_explicit(self):
        task = tasklib.load_tasks(only=["m01-disjoint-utils"])[0]
        stages = [
            "missing_prerequisite",
            "unsupported_task",
            "delivery_failure",
            "failed_verifier",
            "timeout",
            "provider_failure",
            "missing_evidence",
            "manual_intervention",
        ]
        for stage in stages:
            with self.subTest(stage=stage):
                rec = controller_mode.failure_fixture_record(
                    task,
                    run.Agent("reference"),
                    self.out / stage,
                    0,
                    ticket_id="complete",
                    stage=stage,
                    message=f"fixture {stage}",
                )
                rec = results.validate(rec)
                self.assertFalse(rec["outcome"]["passed"], rec)
                self.assertEqual(rec["outcome"]["orchestration_verdict"], "fail")
                self.assertEqual(rec["controller"]["failure"]["stage"], stage)
                self.assertEqual(rec["controller"]["attempts"]["automatic_retries_after_failure"], 0)
                self.assertFalse(rec["controller"]["execution_window"]["automatic_rerun_after_timeout_or_provider_failure"])
                self.assertIn(rec["controller"]["model_usage"]["status"], {"not_applicable", "unknown"})
                if stage == "failed_verifier":
                    self.assertEqual(rec["outcome"]["task_verdict"], "fail")
                    self.assertEqual(rec["outcome"]["effect_disposition"], "unresolved")
                if stage == "manual_intervention":
                    self.assertEqual(rec["coordination"]["manual_interventions"], 1)
                    self.assertFalse(rec["controller"]["autonomous_pass_eligible"])
                if stage in {"missing_prerequisite", "unsupported_task"}:
                    self.assertEqual(rec["timing"]["execution_seconds"], 0.0)
                    self.assertFalse(rec["controller"]["execution_window"]["failed_preflight_consumed_model_calls"])

    def test_controller_timeout_fault_persists_valid_failure_result_through_run_lifecycle(self):
        out = self.out / "controller-timeout"
        code = run.main([
            "m01-disjoint-utils",
            "--condition",
            "controller:t1",
            "--agent",
            "reference",
            "--controller-lifecycle-fault",
            "elapsed-deadline",
            "--out",
            str(out),
        ])
        self.assertEqual(code, 0)
        loaded = results.load_all(out)
        self.assertEqual(len(loaded), 1)
        rec = loaded[0]
        self.assertFalse(rec["outcome"]["passed"], rec)
        self.assertTrue(rec["outcome"]["timed_out"])
        self.assertEqual(rec["outcome"]["failure_stage"], "timeout")
        self.assertEqual(rec["outcome"]["effect_disposition"], "resolved")
        self.assertGreater(rec["timing"]["setup_seconds"], 0.0)
        self.assertGreaterEqual(rec["timing"]["cleanup_seconds"], 0.0)
        self.assertLessEqual(
            rec["timing"]["cleanup_seconds"],
            rec["controller"]["execution_window"]["cleanup_timeout_seconds"],
        )
        controller = rec["controller"]
        self.assertEqual(controller["failure"]["stage"], "timeout")
        self.assertEqual(controller["failure_details"]["command"], "set_mission")
        admission = json.loads((Path(controller["run_root"]) / "durable-run-admission.json").read_text())
        self.assertEqual(
            controller["execution_window"]["durable_run_admission_unix"],
            admission["admitted_unix"],
        )
        self.assertEqual(
            controller["execution_window"]["dispatch_cutoff_unix"],
            admission["dispatch_cutoff_unix"],
        )
        self.assertGreaterEqual(
            controller["failure_details"]["observed_unix"],
            controller["failure_details"]["dispatch_cutoff_unix"],
        )
        self.assertEqual(controller["attempts"]["automatic_retries_after_failure"], 0)
        self.assertFalse(controller["execution_window"]["automatic_rerun_after_timeout_or_provider_failure"])
        self.assertEqual(
            controller["execution_window"]["dispatch_cutoff_unix"],
            controller["execution_window"]["durable_run_admission_unix"] + 1800.0,
        )
        self.assertTrue(controller["retained_evidence"]["logs"])
        self.assertTrue(controller["retained_evidence"]["manifests"])
        self.assertEqual(controller["process_disposition"]["reconciliation"]["status"], "resolved")
        self.assertTrue(controller["process_disposition"]["reconciliation"]["bounded"])
        self.assertTrue(controller["process_disposition"]["reconciliation"]["identity_bound"])
        self.assertFalse(controller["process_disposition"]["remaining_uncertain"])

    def test_controller_failed_verifier_fault_observes_real_verifier_failure(self):
        out = self.out / "controller-failed-verifier"
        code = run.main([
            "m01-disjoint-utils",
            "--condition",
            "controller:t1",
            "--agent",
            "reference",
            "--controller-lifecycle-fault",
            "failed-verifier",
            "--out",
            str(out),
        ])
        self.assertEqual(code, 0)
        rec = results.load_all(out)[0]
        self.assertFalse(rec["outcome"]["passed"], rec)
        self.assertEqual(rec["outcome"]["failure_stage"], "failed_verifier")
        self.assertEqual(rec["outcome"]["task_verdict"], "fail")
        self.assertEqual(rec["outcome"]["effect_disposition"], "resolved")
        verification = rec["controller"]["failure_details"]["verification"]
        self.assertEqual(verification["verdict"], "fail")
        self.assertIn("verification", verification)
        self.assertGreater(rec["timing"]["verify_seconds"], 0.0)
        self.assertFalse(rec["controller"]["process_disposition"]["remaining_uncertain"])

    def test_controller_missing_evidence_fault_observes_missing_retained_artifact_after_cleanup(self):
        out = self.out / "controller-missing-evidence"
        code = run.main([
            "m01-disjoint-utils",
            "--condition",
            "controller:t1",
            "--agent",
            "reference",
            "--controller-lifecycle-fault",
            "missing-evidence",
            "--out",
            str(out),
        ])
        self.assertEqual(code, 0)
        rec = results.load_all(out)[0]
        self.assertFalse(rec["outcome"]["passed"], rec)
        self.assertEqual(rec["outcome"]["failure_stage"], "missing_evidence")
        missing = rec["controller"]["failure_details"]["missing_paths"]
        self.assertTrue(missing)
        self.assertFalse(Path(missing[0]).exists())
        self.assertGreaterEqual(rec["timing"]["cleanup_seconds"], 0.0)
        self.assertGreaterEqual(rec["controller"]["cleanup"]["observed_owned_processes"], 2)
        self.assertEqual(rec["controller"]["process_disposition"]["owned_attempts"], "stopped")

    def test_controller_final_verifier_uses_shared_deadline(self):
        out = self.out / "controller-final-deadline"
        code = run.main([
            "m01-disjoint-utils",
            "--condition",
            "controller:complete",
            "--agent",
            "reference",
            "--controller-lifecycle-fault",
            "final-deadline",
            "--out",
            str(out),
        ])
        self.assertEqual(code, 0)
        rec = results.load_all(out)[0]
        self.assertFalse(rec["outcome"]["passed"], rec)
        self.assertTrue(rec["outcome"]["timed_out"])
        self.assertEqual(rec["outcome"]["failure_stage"], "timeout")
        self.assertEqual(rec["controller"]["failure_details"]["command"], "final_independent_verifier")
        self.assertEqual(rec["coordination"]["attempts"], 3)
        self.assertEqual(rec["controller"]["attempts"]["fresh_integration"], 1)
        self.assertLessEqual(
            rec["timing"]["cleanup_seconds"],
            rec["controller"]["execution_window"]["cleanup_timeout_seconds"],
        )

    def test_controller_final_verifier_inflight_timeout_classified_as_timeout(self):
        out = self.out / "controller-final-inflight-timeout"
        code = run.main([
            "m01-disjoint-utils",
            "--condition",
            "controller:complete",
            "--agent",
            "reference",
            "--controller-lifecycle-fault",
            "final-inflight-timeout",
            "--repeat",
            "2",
            "--out",
            str(out),
        ])
        self.assertEqual(code, 0)
        loaded = results.load_all(out)
        self.assertEqual(len(loaded), 1)
        rec = loaded[0]
        self.assertEqual(rec["outcome"]["failure_stage"], "timeout")
        self.assertTrue(rec["outcome"]["timed_out"])
        self.assertEqual(rec["controller"]["failure_details"]["exception_type"], "TimeoutExpired")
        self.assertIn("independent verifier", rec["controller"]["failure_details"]["boundary"])

    def test_controller_cleanup_keeps_missing_attempt_identity_uncertain(self):
        controller = object.__new__(controller_mode.ControllerRun)
        controller.root = self.out / "cleanup-identity"
        attempts = controller.root / "attempts"
        attempts.mkdir(parents=True)
        controller.processes = []
        controller.admitted_attempt_ids = ["missing-attempt", "unreadable-attempt", "exited-attempt"]
        (attempts / "unreadable-operation.json").write_text("{not-json")
        controller_mode.write_json(
            attempts / "exited-operation.json",
            {
                "binding": {"attempt": "exited-attempt", "operation": "exited-operation"},
                "state": "exited",
                "outcome": "identity-bound supervisor confirmed exit",
            },
        )
        cancelled = []

        def fake_query(*args, **kwargs):
            return {"attempts": []}

        def fake_command(kind, args, **kwargs):
            cancelled.append(args["attempt"])
            return {"ok": {"attempt": args["attempt"]}}

        controller.query = fake_query
        controller.command = fake_command

        records = controller.reconcile_owned_processes()
        reconciliation = next(item["record"] for item in records if item.get("name") == "reconciliation")
        remaining = {item["id"] for item in reconciliation["remaining_uncertain"]}
        self.assertEqual(remaining, {"attempt:missing-attempt", "attempt:unreadable-attempt", "attempt:unreadable-operation"})
        self.assertEqual(reconciliation["status"], "unresolved")
        self.assertEqual(set(cancelled), {"missing-attempt", "unreadable-attempt", "unreadable-operation"})
        observed = {item["attempt"]: item for item in reconciliation["final_attempt_records"]}
        self.assertEqual(observed["missing-attempt"]["identity_evidence"], "missing")
        self.assertEqual(observed["unreadable-attempt"]["identity_evidence"], "missing")
        self.assertEqual(observed["unreadable-operation"]["identity_evidence"], "unreadable")
        self.assertEqual(observed["exited-attempt"]["identity_evidence"], "read")

    def test_controller_cleanup_keeps_unwaited_service_uncertain(self):
        class HangingProcess:
            pid = 424242

            def poll(self):
                return None

            def terminate(self):
                pass

            def kill(self):
                pass

            def wait(self, timeout=None):
                raise subprocess.TimeoutExpired(["service"], timeout)

        service_log = self.out / "hanging-service.log"
        service_log.write_text("still running\n")
        controller = object.__new__(controller_mode.ControllerRun)
        controller.root = self.out / "cleanup-service"
        (controller.root / "attempts").mkdir(parents=True)
        controller.processes = [
            controller_mode.ManagedProcess("controller", HangingProcess(), service_log, "service-start")
        ]
        controller.admitted_attempt_ids = []

        def fake_query(*args, **kwargs):
            return {"attempts": []}

        controller.query = fake_query
        controller.command = lambda *args, **kwargs: {"ok": {}}

        records = controller.reconcile_owned_processes(timeout_seconds=0.0)
        reconciliation = next(item["record"] for item in records if item.get("name") == "reconciliation")
        self.assertEqual(reconciliation["status"], "unresolved")
        self.assertIn("service:controller", {item["id"] for item in reconciliation["remaining_uncertain"]})

    def test_controller_success_requires_observed_service_exit(self):
        original_stop = controller_mode.stopped_services

        def uncertain_stop(*args, **kwargs):
            records = original_stop(*args, **kwargs)
            # Physical closure of one owned service was not observed after kill.
            records[0]["exit"] = None
            return records

        task = tasklib.load_tasks(only=["m01-disjoint-utils"])[0]
        for ticket in ("t1", "complete"):
            with self.subTest(ticket=ticket), mock.patch.object(
                controller_mode, "stopped_services", side_effect=uncertain_stop
            ):
                rec = results.validate(controller_mode.run_controller_condition(
                    task, run.Agent("reference"), self.out, 0, ticket
                ))
                self.assertIn(rec["outcome"]["task_verdict"], ("pass", "partial_pass"))
                self.assertFalse(rec["outcome"]["passed"])
                self.assertEqual(rec["outcome"]["effect_disposition"], "unresolved")
                disposition = rec["controller"]["process_disposition"]
                self.assertEqual(disposition["services"], "unresolved")
                self.assertTrue(disposition["remaining_uncertain"])

    def test_controller_manual_pause_before_verification_has_unknown_task_truth(self):
        task = tasklib.load_tasks(only=["m01-disjoint-utils"])[0]
        rec = results.validate(controller_mode.run_controller_condition(
            task, run.Agent("reference"), self.out, 0, "t1", fault="manual-intervention"
        ))
        self.assertEqual(rec["outcome"]["failure_stage"], "manual_intervention")
        self.assertEqual(rec["outcome"]["task_verdict"], "unknown")
        self.assertEqual(rec["outcome"]["orchestration_verdict"], "fail")
        self.assertIsNone(rec["outcome"]["verifier_rc"])
        self.assertFalse(rec["outcome"]["passed"])
        self.assertEqual(rec["controller"]["retained_evidence"]["proofs"][0]["status"], "unknown")

    def test_controller_failure_disposition_follows_actual_cleanup_not_failure_stage(self):
        task = tasklib.load_tasks(only=["m01-disjoint-utils"])[0]
        controller = object.__new__(controller_mode.ControllerRun)
        controller.root = self.out / "resolved-failure"
        (controller.root / "attempts").mkdir(parents=True)
        controller.processes = []
        controller.admitted_attempt_ids = []
        controller.query = lambda *args, **kwargs: {"attempts": []}
        controller.command = lambda *args, **kwargs: {"ok": {}}
        records = controller.reconcile_owned_processes()
        for stage in ("failed_verifier", "generic_failure", "delivery_failure"):
            with self.subTest(stage=stage):
                rec = results.validate(controller_mode.controller_failure_record(
                    task=task, agent=run.Agent("reference"), rid=stage, ticket_id="t1",
                    root=controller.root, source=controller_mode.unknown_source_identity(),
                    build={"seconds": 0.0, "exit": None},
                    failure=controller_mode.ControllerFailure(stage, "controlled failure", {"attempts_total": 0}),
                    control=None, process_records=records, setup_seconds=0.0,
                    execution_seconds=0.0, cleanup_seconds=0.0, total_wall_seconds=0.0,
                    source_capture_seconds=0.0,
                ))
                self.assertFalse(rec["outcome"]["passed"])
                self.assertEqual(rec["outcome"]["effect_disposition"], "resolved")
                self.assertFalse(rec["controller"]["process_disposition"]["remaining_uncertain"])

    def test_controller_report_separates_stale_refusals_from_fleet_conflicts(self):
        controller = results.validate(self.minimal_controller_record(run_id="controller-stale"))
        solo = self.solo("t01-log-window", "reference")
        rows = {row["condition"]: row for row in results.summarize([controller, solo])}
        self.assertEqual(rows["controller:complete"]["conflicts"], 0)
        self.assertEqual(rows["controller:complete"]["stale_refusals"], 1)
        self.assertEqual(rows["solo"]["stale_refusals"], 0)
        table = results.table(list(rows.values()))
        self.assertIn("stale", table)
        self.assertRegex(table, r"controller:complete\s+reference\s+1\s+1\.00.*\s0\s+1\s+1\s+0")

    def test_controller_export_rejects_unsafe_archive_members_before_extraction(self):
        archive = self.out / "malicious.tar"
        outside = self.out / "escaped"
        with tarfile.open(archive, "w") as stream:
            info = tarfile.TarInfo("../escaped")
            payload = b"not a workspace member"
            info.size = len(payload)
            stream.addfile(info, io.BytesIO(payload))
        with self.assertRaisesRegex(RuntimeError, "unsafe archive member"):
            controller_mode.extract_git_archive_safely(archive.read_bytes(), self.out / "workspace")
        self.assertFalse(outside.exists())
        link_archive = self.out / "link.tar"
        with tarfile.open(link_archive, "w") as stream:
            info = tarfile.TarInfo("tests/test_utils.py")
            info.type = tarfile.SYMTYPE
            info.linkname = "safe-looking-target"
            stream.addfile(info)
        with self.assertRaisesRegex(RuntimeError, "unsupported archive member type"):
            controller_mode.extract_git_archive_safely(link_archive.read_bytes(), self.out / "workspace-link")

    def test_controller_mode_rejects_unsupported_tasks_before_execution(self):
        out = self.out / "unsupported-controller"
        with self.assertRaisesRegex(SystemExit, "controller mode currently supports only m01-disjoint-utils"):
            run.main(["m02-shared-interface", "--condition", "controller:t1", "--agent", "reference", "--out", str(out)])
        self.assertFalse(out.exists())

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
