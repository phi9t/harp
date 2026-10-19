"""Lead-driven readiness scenarios: scripted participants, real git, independent hand-off verifier."""

from __future__ import annotations

import contextlib
import importlib.util
import io
import json
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))
import lead_driven  # noqa: E402
import results  # noqa: E402
import tasklib  # noqa: E402


class LeadDrivenTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.out = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()

    def test_interactive_tmux_rejects_complete_overlong_socket_before_allocation(self):
        root = self.out / ('long' * 25)
        with patch.dict(os.environ, TMPDIR=str(root)):
            with self.assertRaisesRegex(lead_driven.LeadDrivenError, 'socket path'):
                lead_driven.interactive_tmux_env(dict(os.environ), self.out)
        self.assertEqual(list(root.iterdir()), [])

    def run_command(self, *args):
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code = lead_driven.main([*args, "--out", str(self.out)])
        return code, buffer.getvalue()

    def run_scenario(self, scenario):
        out = self.out / scenario
        buffer = io.StringIO()
        with contextlib.redirect_stdout(buffer):
            code = lead_driven.main([scenario, "--out", str(out)])
        records = {
            json.loads(path.read_text())["participant"]: json.loads(path.read_text())
            for path in sorted(out.rglob("result.json"))
        }
        return code, buffer.getvalue(), records

    def run_process(self, *args):
        return subprocess.run(
            [sys.executable, str(Path(lead_driven.__file__)), *args],
            capture_output=True,
            text=True,
        )

    def run_process_env(self, env, *args):
        return subprocess.run(
            [sys.executable, str(Path(lead_driven.__file__)), *args],
            capture_output=True,
            text=True,
            env={**os.environ, **env},
        )

    def records(self):
        return {
            json.loads(path.read_text())["participant"]: json.loads(path.read_text())
            for path in sorted(self.out.rglob("result.json"))
        }

    def case_records(self):
        rows = {}
        for path in sorted(self.out.rglob("result.json")):
            row = json.loads(path.read_text())
            rows[row.get("case") or row["participant"]] = row
        return rows

    def write_readiness_row(self, **overrides):
        row = {
            "schema_version": "corenius.bench.readiness_result.v1",
            "recorded_at": "2026-10-05T10:00:00Z",
            "scenario": "handoff",
            "instantiation": "lead-driven",
            "runtime": "scripted",
            "proof": "deterministic",
            "outcome": "passes",
            "reason": None,
            "interventions": [],
            "elapsed_seconds": 1.25,
            "model_usage": "unknown",
            "participants": ["commit-ready"],
            "stories": [1, 2, 44],
        }
        row.update(overrides)
        path = self.out / "readiness-results.jsonl"
        with path.open("a") as fh:
            fh.write(json.dumps(row, sort_keys=True) + "\n")

    def test_handoff_scenario_runs_offline_and_prints_one_result_per_participant(self):
        code, text = self.run_command("handoff")
        self.assertEqual(code, 0, text)
        lines = [line for line in text.splitlines() if line.strip()]
        self.assertEqual(len(lines), 2, text)
        self.assertTrue(all(line.startswith("passes ") for line in lines), text)
        self.assertIn("commit-ready", text)
        self.assertIn("dirty-final-message", text)

    def test_scenario_with_tmpdir_unset_does_not_use_default_temp_root(self):
        forbidden = self.out / "forbidden-host-tmp"
        forbidden.mkdir()
        old_tempdir = tempfile.tempdir
        old_tmpdir = os.environ.pop("TMPDIR", None)
        try:
            tempfile.tempdir = str(forbidden)
            code, text = self.run_command("handoff", "--participant", "commit-ready")
        finally:
            tempfile.tempdir = old_tempdir
            if old_tmpdir is not None:
                os.environ["TMPDIR"] = old_tmpdir

        self.assertEqual(code, 0, text)
        self.assertEqual(list(forbidden.rglob("*")), [])

    def test_launch_by_edges_and_triage_scenario_runs_offline(self):
        code, text = self.run_command("launch-by-edges-and-triage")
        self.assertEqual(code, 0, text)
        for name in (
            "launchable-set",
            "unresolved-blocker-refused",
            "bad-blocker-refused",
            "human-triage-refused",
            "missing-triage-refused",
            "parallel-cap-refused",
            "integration-base-and-pinned-brief",
            "two-worktree-integration-checkout",
            "worker-question",
            "handoffs-into-briefs",
        ):
            self.assertIn(f"passes launch-by-edges-and-triage {name}", text)
        records = self.case_records()
        self.assertEqual(records["launchable-set"]["observed"]["launchable_ids"], ["02", "08", "09", "10"])
        self.assertEqual(records["integration-base-and-pinned-brief"]["observed"]["base"], "integration")
        self.assertTrue(records["integration-base-and-pinned-brief"]["observed"]["record_sources_pinned"])
        two_worktree = records["two-worktree-integration-checkout"]["observed"]
        self.assertEqual(two_worktree["base"], "integration")
        self.assertTrue(two_worktree["claim_commit_on_integration"])
        self.assertTrue(two_worktree["main_checkout_untouched"])
        self.assertTrue(two_worktree["mainline_untouched"])
        self.assertEqual(records["worker-question"]["observed"]["triage"], "needs-info")
        self.assertTrue(records["worker-question"]["observed"]["lint_ok"])
        handoff = records["handoffs-into-briefs"]["observed"]
        self.assertEqual(handoff["finding_sources"], ["ticket 02-ready", "report 05", "lead"])
        self.assertTrue(handoff["prompt_contains_findings"])

    def test_lead_launch_runtime_option_reaches_worker_and_launch_record(self):
        code, text = self.run_command(
            "launch-by-edges-and-triage",
            "--participant",
            "integration-base-and-pinned-brief",
            "--runtime",
            "codex",
        )
        self.assertEqual(code, 0, text)
        record = self.case_records()["integration-base-and-pinned-brief"]
        self.assertEqual(record["observed"]["worker_runtime"], "codex")
        self.assertEqual(record["observed"]["lead_launch_runtime"], "codex")
        rows = [json.loads(line) for line in (self.out / "readiness-results.jsonl").read_text().splitlines()]
        self.assertEqual(rows[-1]["runtime"], "scripted")
        self.assertEqual(rows[-1]["requested_runtime"], "codex")

    def test_good_participant_records_candidate_equal_to_worker_branch_tip(self):
        code, _ = self.run_command("handoff")
        self.assertEqual(code, 0)
        record = self.records()["commit-ready"]
        candidate = record["toolbox_verdict"]["candidate"]
        repo = Path(record["repo"])
        branch = record["run"]["branch"]
        tip = subprocess.check_output(["git", "-C", str(repo), "rev-parse", branch], text=True).strip()
        tree = subprocess.check_output(["git", "-C", str(repo), "rev-parse", f"{tip}^{{tree}}"], text=True).strip()
        self.assertTrue(record["toolbox_verdict"]["ready_for_review"], record)
        self.assertEqual(candidate["commit"], tip)
        self.assertEqual(candidate["tree"], tree)
        self.assertEqual(candidate["base"], record["run"]["base_sha"])
        self.assertEqual(candidate["brief_digest"], record["run"]["brief_digest"])

    def test_dirty_final_message_is_not_used_as_readiness_evidence(self):
        code, _ = self.run_command("handoff")
        self.assertEqual(code, 0)
        record = self.records()["dirty-final-message"]
        self.assertEqual(Path(record["run"]["last_message"]).read_text(), "ready for review\n")
        self.assertFalse(record["verification"]["ready_for_review"], record)
        self.assertIsNone(record["verification"]["candidate"])
        self.assertIn("dirty-tree", record["verification"]["not_ready"])

    def test_no_commit_participant_is_not_ready(self):
        code, _ = self.run_command("handoff", "--participant", "no-commit")
        self.assertEqual(code, 0)
        record = self.records()["no-commit"]
        self.assertFalse(record["verification"]["ready_for_review"], record)
        self.assertIn("no-commit-after-base", record["verification"]["not_ready"])

    def test_branch_movement_participants_are_violations(self):
        code, _ = self.run_command(
            "handoff",
            "--participant",
            "move-foreign-branch",
            "--participant",
            "create-foreign-branch",
            "--participant",
            "rewrite-base",
        )
        self.assertEqual(code, 0)
        records = self.records()
        self.assertIn("foreign-branch-moved", records["move-foreign-branch"]["verification"]["violations"])
        self.assertIn("foreign-branch-moved", records["create-foreign-branch"]["verification"]["violations"])
        self.assertIn("base-not-ancestor", records["rewrite-base"]["verification"]["violations"])

    def test_doctored_toolbox_verdict_disagreements_fail_the_scenario(self):
        verification = {"ready_for_review": False, "candidate": None, "violations": [], "not_ready": ["dirty-tree"]}
        ready_on_dirty = {"ready_for_review": True, "candidate": {"commit": "abc"}}
        outcome, details = lead_driven.judge_result(False, ready_on_dirty, verification)
        self.assertEqual(outcome, "fails")
        self.assertIn("expected.ready_for_review", details)
        self.assertIn("ready_for_review", details)

        verification = {
            "ready_for_review": True,
            "candidate": {"commit": "good", "tree": "tree", "base": "base", "brief_digest": "brief"},
            "violations": [],
            "not_ready": [],
        }
        wrong_candidate = {
            "ready_for_review": True,
            "candidate": {"commit": "bad", "tree": "tree", "base": "base", "brief_digest": "brief"},
        }
        outcome, details = lead_driven.judge_result(True, wrong_candidate, verification)
        self.assertEqual(outcome, "fails")
        self.assertIn("candidate.commit", details)

    def test_marker_participants_pin_candidate_and_report_later_movement(self):
        code, _ = self.run_command("handoff", "--participant", "marker-ready", "--participant", "marker-rewrite")
        self.assertEqual(code, 0)
        records = self.records()
        self.assertTrue(records["marker-ready"]["toolbox_verdict"]["ready_for_review"])
        self.assertEqual(records["marker-ready"]["run"]["candidate"], records["marker-ready"]["toolbox_verdict"]["candidate"])
        self.assertTrue(records["marker-rewrite"]["toolbox_verdict"]["ready_for_review"])
        self.assertEqual(records["marker-rewrite"]["run"]["candidate"], records["marker-rewrite"]["toolbox_verdict"]["candidate"])
        self.assertIn("candidate-moved", json.dumps(records["marker-rewrite"]["post_ready"]["findings"]))
        self.assertEqual(
            records["marker-rewrite"]["run"]["candidate"]["commit"],
            records["marker-rewrite"]["post_ready"]["candidate"]["commit"],
        )

    def test_marker_dirty_participant_is_not_ready(self):
        code, _ = self.run_command("handoff", "--participant", "marker-dirty")
        self.assertEqual(code, 0)
        record = self.records()["marker-dirty"]
        self.assertFalse(record["toolbox_verdict"]["ready_for_review"], record)
        self.assertIn("dirty-tree", record["verification"]["not_ready"])

    def test_scenario_materializes_a_benchmark_task_with_tasklib(self):
        code, _ = self.run_command("handoff", "--participant", "commit-ready")
        self.assertEqual(code, 0)
        record = self.records()["commit-ready"]
        self.assertEqual(record["materialized_task"], "t01-log-window")
        self.assertTrue((Path(record["repo"]) / "metrics.sh").exists())
        self.assertTrue((Path(record["repo"]) / "events.log").exists())

    def test_scenario_appends_one_readiness_row_with_intervention_details(self):
        proc = self.run_process(
            "handoff",
            "--participant",
            "commit-ready",
            "--intervention",
            "lead:waited for idle",
            "--out",
            str(self.out),
        )
        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)
        rows = [json.loads(line) for line in (self.out / "readiness-results.jsonl").read_text().splitlines()]
        self.assertEqual(len(rows), 1)
        row = rows[0]
        self.assertEqual(row["scenario"], "handoff")
        self.assertEqual(row["instantiation"], "lead-driven")
        self.assertEqual(row["runtime"], "scripted")
        self.assertEqual(row["proof"], "deterministic")
        self.assertEqual(row["outcome"], "passes with intervention")
        self.assertEqual(row["participants"], ["commit-ready"])
        self.assertEqual(row["stories"], [1, 2, 44])
        self.assertEqual(row["interventions"], [{"who": "lead", "what": "waited for idle"}])
        self.assertEqual(row["model_usage"], "unknown")
        self.assertGreater(row["elapsed_seconds"], 0)

    def test_codex_runtime_option_reaches_worker_and_results_row(self):
        proc = self.run_process(
            "handoff",
            "--participant",
            "commit-ready",
            "--runtime",
            "codex",
            "--proof",
            "live",
            "--out",
            str(self.out),
        )
        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)
        record = self.records()["commit-ready"]
        self.assertEqual(record["run"]["runtime"], "codex")
        self.assertEqual(record["toolbox_verdict"]["verdict"], "CLEAN", record)
        rows = [json.loads(line) for line in (self.out / "readiness-results.jsonl").read_text().splitlines()]
        self.assertEqual(rows[-1]["runtime"], "scripted")
        self.assertEqual(rows[-1]["requested_runtime"], "codex")
        self.assertEqual(rows[-1]["proof"], "deterministic")

    def test_erroring_scenario_writes_a_failing_readiness_row(self):
        proc = self.run_process_env(
            {"INSULA_READINESS_WORKER": str(self.out / "missing-worker")},
            "handoff",
            "--participant",
            "commit-ready",
            "--out",
            str(self.out),
        )

        self.assertEqual(proc.returncode, 1)
        rows = [json.loads(line) for line in (self.out / "readiness-results.jsonl").read_text().splitlines()]
        self.assertEqual(rows[-1]["scenario"], "handoff")
        self.assertEqual(rows[-1]["outcome"], "fails")
        self.assertIn("worker", rows[-1]["reason"].lower())

    def test_empty_scenario_record_list_is_a_failure(self):
        self.assertEqual(lead_driven.scenario_outcome([], []), "fails")

    def test_mutation_check_fails_when_judged_tool_is_stubbed(self):
        stub = self.out / "noop-tool"
        stub.write_text("#!/bin/sh\nexit 0\n")
        stub.chmod(0o755)
        cases = [
            ("handoff", "INSULA_READINESS_WORKER", ["--participant", "commit-ready"]),
            ("acceptance-failing-check", "INSULA_READINESS_ACCEPTANCE", []),
            ("land-exact-candidate", "INSULA_READINESS_LAND", []),
            ("launch-by-edges-and-triage", "INSULA_READINESS_TRACKER", ["--participant", "launchable-set"]),
            ("launch-by-edges-and-triage", "INSULA_READINESS_LEAD_LAUNCH", ["--participant", "integration-base-and-pinned-brief"]),
            ("bookkeeping-off-code-branch", "INSULA_READINESS_WORKER", []),
            ("interactive-launch-and-monitors", "INSULA_READINESS_LEAD_LAUNCH", []),
            ("interactive-launch-and-monitors", "INSULA_READINESS_LAND", []),
        ]
        for scenario, variable, extra in cases:
            with self.subTest(scenario=scenario, variable=variable):
                out = self.out / f"mutated-{scenario}-{variable}"
                proc = self.run_process_env({variable: str(stub)}, scenario, *extra, "--out", str(out))
                self.assertEqual(proc.returncode, 1, proc.stderr + proc.stdout)
                rows = [json.loads(line) for line in (out / "readiness-results.jsonl").read_text().splitlines()]
                self.assertEqual(rows[-1]["outcome"], "fails")

    def test_all_scripted_participants_are_executable(self):
        scripts = sorted((Path(lead_driven.READINESS)).glob("*/scripted_participant.py"))
        self.assertTrue(scripts)
        not_executable = [str(path.relative_to(lead_driven.ROOT)) for path in scripts if not os.access(path, os.X_OK)]
        self.assertEqual(not_executable, [])

    def test_unexpected_worker_stderr_fails_participant(self):
        with patch.object(lead_driven, "run_cmd") as run_cmd:
            def fake_run(args, env=None, cwd=None):
                text_args = [str(arg) for arg in args]
                if "supervise" in text_args:
                    run_dir = Path(text_args[text_args.index("supervise") + 1])
                    run = json.loads((run_dir / "run.json").read_text())
                    repo = Path(run["worktree"])
                    (repo / "result.txt").write_text("ready\n")
                    subprocess.run(["git", "-C", str(repo), "add", "result.txt"], check=True)
                    subprocess.run(
                        [
                            "git",
                            "-C",
                            str(repo),
                            "-c",
                            "user.email=bench@corenius",
                            "-c",
                            "user.name=bench",
                            "commit",
                            "-q",
                            "-m",
                            "result",
                        ],
                        check=True,
                    )
                    (run_dir / "exit_code").write_text("0\n")
                    (run_dir / "stderr.log").write_text("unexpected runtime error\n")
                    candidate = lead_driven.verify_handoff(run_dir)["candidate"]
                    return subprocess.CompletedProcess(args, 0, json.dumps({
                        "verdict": "CLEAN",
                        "ready_for_review": True,
                        "candidate": candidate,
                        "not_ready": [],
                        "findings": [],
                    }) + "\n", "")
                return subprocess.run(args, capture_output=True, text=True, env=env, cwd=cwd)

            run_cmd.side_effect = fake_run
            record = lead_driven.launch_and_supervise(
                lead_driven.load_scenario("handoff"),
                lead_driven.load_scenario("handoff").participants[0],
                self.out / "stderr-case",
            )

        self.assertEqual(record["scenario_result"], "fails")
        self.assertIn("worker-stderr", record["failure_details"])

    def test_non_json_worker_event_line_fails_participant(self):
        run_dir = self.out / "bad-events"
        run_dir.mkdir()
        (run_dir / "events.jsonl").write_text("plain runtime output\n")

        details = lead_driven.worker_execution_failures(run_dir)

        self.assertEqual(details, ["worker-unexpected-output:plain runtime output"])

    def test_report_prints_all_instantiations_and_core_summary(self):
        self.write_readiness_row(outcome="passes")
        self.write_readiness_row(
            instantiation="local-controller",
            runtime="git",
            outcome="not applicable by design",
            reason="controller qualification pending",
        )
        proc = self.run_process("--report", str(self.out))
        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)
        text = proc.stdout
        self.assertIn("Scenario: handoff", text)
        self.assertIn("lead-driven", text)
        self.assertIn("local-controller", text)
        self.assertIn("workflow-controller", text)
        self.assertIn("not yet run", text)
        self.assertIn("not applicable by design", text)
        self.assertIn("controller qualification pending", text)
        self.assertIn(
            "Core set (deterministic, lead-driven): passes=1 passes with intervention=0 fails=0 not yet run=11 not applicable by design=0",
            text,
        )
        self.assertIn("participants", text)
        self.assertIn(
            "Core set (deterministic, local-controller): passes=0 passes with intervention=0 fails=0 not yet run=11 not applicable by design=1",
            text,
        )
        self.assertIn(
            "Core set (deterministic, workflow-controller): passes=0 passes with intervention=0 fails=0 not yet run=12 not applicable by design=0",
            text,
        )

    def test_report_keeps_deterministic_and_live_rows_separate(self):
        self.write_readiness_row(proof="deterministic", outcome="passes")
        self.write_readiness_row(proof="live", outcome="fails")
        proc = self.run_process("--report", str(self.out))
        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)
        text = proc.stdout
        self.assertIn("Proof: deterministic", text)
        self.assertIn("Proof: live", text)
        self.assertIn(
            "Core set (deterministic, lead-driven): passes=1 passes with intervention=0 fails=0 not yet run=11 not applicable by design=0",
            text,
        )
        self.assertIn(
            "Core set (live, lead-driven): passes=0 passes with intervention=0 fails=1 not yet run=11 not applicable by design=0",
            text,
        )

    def test_report_calls_out_scenarios_without_declared_stories(self):
        readiness = self.out / "readiness"
        scenario = readiness / "unstoried"
        scenario.mkdir(parents=True)
        (scenario / "scenario.toml").write_text(
            'id = "unstoried"\n'
            'task = "task.md"\n'
            'environment = "environment"\n'
            '\n'
            '[worker]\n'
            'allow = []\n'
        )
        text = results.readiness_report(self.out, readiness)
        self.assertIn("Scenario: unstoried", text)
        self.assertIn("Stories: not declared", text)

    def test_notes_command_appends_known_kinds_and_refuses_unknown_kind(self):
        notes = self.out / "notes.md"
        proc = self.run_process("--note", "obstacle", "disk nearly full", "--notes-file", str(notes))
        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)
        text = notes.read_text()
        self.assertIn("# Running notes: getting the lead-driven instantiation working", text)
        self.assertRegex(text, r"- 20\d\d-\d\d-\d\dT\d\d:\d\dZ obstacle: disk nearly full")

        bad = self.run_process("--note", "surprise", "not a known kind", "--notes-file", str(notes))
        self.assertEqual(bad.returncode, 2)
        self.assertIn("unknown note kind", bad.stderr)
        self.assertEqual(notes.read_text(), text)

    def test_exact_candidate_landing_scenario_lands_candidate_and_binds_gate(self):
        code, text = self.run_command("land-exact-candidate")
        self.assertEqual(code, 0, text)
        self.assertIn("passes land-exact-candidate commit-ready expected=landed observed=landed", text)
        record = self.records()["commit-ready"]
        candidate = record["toolbox_verdict"]["candidate"]["commit"]
        landing = record["landing_verification"]
        self.assertEqual(landing["return_code"], 0, landing)
        self.assertEqual(landing["stdout"], candidate)
        self.assertEqual(landing["target_after"], candidate)
        self.assertEqual(landing["gate_seen"], candidate)
        self.assertEqual(landing["target_status_after"], "")
        self.assertEqual(landing["landed_result"], "ready\n")

    def test_stale_candidate_landing_scenario_refuses_and_preserves_worker(self):
        code, text = self.run_command("land-stale-candidate")
        self.assertEqual(code, 0, text)
        self.assertIn("passes land-stale-candidate commit-ready expected=stale observed=stale", text)
        record = self.records()["commit-ready"]
        landing = record["landing_verification"]
        self.assertEqual(landing["return_code"], 6, landing)
        self.assertEqual(landing["target_after"], landing["target_before"])
        self.assertEqual(landing["worker_branch_after"], landing["worker_branch_before"])
        self.assertEqual(landing["worker_status_after"], landing["worker_status_before"])
        self.assertIsNone(landing["gate_seen"])
        self.assertIn("new Candidate on the new base", landing["stderr"])

    def test_report_lists_landing_scenarios_after_runs_with_stories(self):
        code, text = self.run_command("land-exact-candidate")
        self.assertEqual(code, 0, text)
        code, text = self.run_command("land-stale-candidate")
        self.assertEqual(code, 0, text)

        proc = self.run_process("--report", str(self.out))

        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)
        self.assertIn("Scenario: land-exact-candidate", proc.stdout)
        self.assertIn("Stories: 5, 27", proc.stdout)
        self.assertIn("Scenario: land-stale-candidate", proc.stdout)
        self.assertIn("Stories: 28, 30", proc.stdout)
        self.assertIn("| lead-driven | scripted | scripted | passes |", proc.stdout)

    def test_access_profile_scenario_runs_through_lead_driven_mode(self):
        code, text = self.run_command("access-profile")
        self.assertEqual(code, 0, text)
        lines = [line for line in text.splitlines() if line.strip()]
        self.assertEqual(len(lines), 4, text)
        self.assertTrue(all(line.startswith("passes ") for line in lines), text)
        records = self.records()

        default = records["default-network"]
        self.assertEqual(default["run"]["access_profile"]["network"], "on")
        self.assertTrue(default["toolbox_verdict"]["ready_for_review"], default)
        self.assertNotIn("network-command", json.dumps(default["toolbox_verdict"]["findings"]))
        self.assertEqual(default["observed_actions"]["loopback_fetch"], "ok")

        offline = records["network-off-download"]
        self.assertEqual(offline["run"]["access_profile"]["network"], "off")
        self.assertFalse(offline["toolbox_verdict"]["ready_for_review"], offline)
        self.assertIn("network-command", json.dumps(offline["toolbox_verdict"]["findings"]))

        timeout = records["long-command"]
        self.assertEqual(timeout["run"]["access_profile"]["command_timeout_seconds"], 125)
        self.assertTrue(timeout["toolbox_verdict"]["ready_for_review"], timeout)
        self.assertGreater(timeout["observed_actions"]["long_command_seconds"], 120.0)

        external = records["external-add-path"]
        profile = external["run"]["access_profile"]
        self.assertEqual(profile["external_add_paths"], [str((Path(external["case_dir"]) / "external-adds").resolve())])
        self.assertFalse(profile["container_builds"])
        self.assertFalse(profile["shared_cache_writes"])
        self.assertTrue(Path(external["observed_actions"]["external_add_file"]).is_file())

        proc = self.run_process("--report", str(self.out))
        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)
        self.assertIn("Scenario: access-profile", proc.stdout)
        self.assertIn("Stories: 31, 32, 33, 36", proc.stdout)

    def test_external_state_report_scenario_runs_through_lead_driven_mode(self):
        code, text = self.run_command("external-state-report")
        self.assertEqual(code, 0, text)
        lines = [line for line in text.splitlines() if line.strip()]
        self.assertEqual(len(lines), 2, text)
        self.assertTrue(all(line.startswith("passes ") for line in lines), text)
        records = self.records()

        unchanged = records["unchanged-external-state"]
        self.assertTrue(unchanged["toolbox_verdict"]["ready_for_review"], unchanged)
        baseline = Path(unchanged["run"]["external_state"]["baseline_file"])
        self.assertLess(baseline.stat().st_mtime, unchanged["run"]["started_epoch"])

        changed = records["modify-remove-add"]
        self.assertFalse(changed["toolbox_verdict"]["ready_for_review"], changed)
        self.assertIn("external-state-modified", changed["verification"]["violations"])
        self.assertIn("external-state-removed", changed["verification"]["violations"])
        self.assertIn("external-state-added", changed["verification"]["violations"])
        findings = changed["toolbox_verdict"]["findings"]
        detail = " ".join(f["detail"] for f in findings if f["kind"].startswith("external-state-"))
        self.assertIn("changed during the run", detail)
        self.assertNotIn("worker", detail.lower())
        self.assertNotIn("allowed-output.txt", json.dumps(findings))

        proc = self.run_process("--report", str(self.out))
        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)
        self.assertIn("Scenario: external-state-report", proc.stdout)
        self.assertIn("Stories: 34, 35", proc.stdout)

    def test_watching_limits_accounting_scenario_runs_through_lead_driven_mode(self):
        code, text = self.run_command("watching-limits-accounting")
        self.assertEqual(code, 0, text)
        for case in ("tmux-missing-launch", "limit-notices-and-accounting"):
            self.assertIn(f"passes watching-limits-accounting {case} expected=checked observed=checked", text)
        records = self.case_records()
        tmux = records["tmux-missing-launch"]["observed"]
        self.assertFalse(tmux["tmux_window"]["available"])
        self.assertEqual(tmux["tmux_window"]["session"], "readiness-watch")
        self.assertEqual(tmux["action_record_commands"], ["printf watching-limits-accounting"])
        limit = records["limit-notices-and-accounting"]["observed"]
        self.assertIn("wall-clock-limit", limit["status_notice_kinds"])
        self.assertIn("quiet-time", limit["status_notice_kinds"])
        self.assertGreaterEqual(limit["elapsed_before_notice"], limit["cap_seconds"])
        self.assertEqual(limit["accounting_process_state"], "stopped")
        self.assertGreater(limit["disk_usage"]["worktree_bytes"], 0)
        self.assertGreater(limit["disk_usage"]["git_dir_bytes"], 0)
        self.assertGreater(limit["disk_usage"]["worker_repo_bytes"], limit["disk_usage"]["worktree_bytes"])
        self.assertGreater(limit["disk_usage"]["run_dir_bytes"], 0)

        proc = self.run_process("--report", str(self.out))
        self.assertEqual(proc.returncode, 0, proc.stderr + proc.stdout)
        self.assertIn("Scenario: watching-limits-accounting", proc.stdout)
        self.assertIn("Stories: 38, 39, 40, 41", proc.stdout)

    def test_acceptance_landing_scenarios_refuse_without_valid_accepted_record(self):
        cases = {
            "acceptance-missing-record": "acceptance record missing",
            "acceptance-other-candidate": "record names another commit",
            "acceptance-failing-check": "failing required check without owner waiver: unit",
            "acceptance-ticket-tamper": "acceptance record missing",
        }
        for scenario, reason in cases.items():
            with self.subTest(scenario=scenario):
                code, text, records = self.run_scenario(scenario)
                self.assertEqual(code, 0, text)
                self.assertIn(f"passes {scenario} commit-ready expected=acceptance-refused observed=acceptance-refused", text)
                landing = records["commit-ready"]["landing_verification"]
                self.assertEqual(landing["return_code"], 7, landing)
                self.assertIn(reason, landing["stderr"])
                self.assertIsNone(landing["gate_seen"])
                self.assertEqual(landing["target_after"], landing["target_before"])

    def test_acceptance_waiver_scenario_lands_and_records_waiver(self):
        code, text, records = self.run_scenario("acceptance-waived-check")

        self.assertEqual(code, 0, text)
        self.assertIn("passes acceptance-waived-check commit-ready expected=landed observed=landed", text)
        landing = records["commit-ready"]["landing_verification"]
        candidate = records["commit-ready"]["toolbox_verdict"]["candidate"]["commit"]
        self.assertEqual(landing["return_code"], 0, landing)
        self.assertEqual(landing["stdout"], candidate)
        self.assertEqual(landing["target_after"], candidate)
        self.assertEqual(landing["acceptance_record"]["owner_waiver"]["owner"], "owner")
        self.assertEqual(landing["acceptance_record"]["required_checks"], [{"name": "unit", "result": "failed"}])

    def test_integration_branch_scenario_runs_through_lead_driven_mode(self):
        code, text = self.run_command("integration-branch")
        records = self.case_records()

        self.assertEqual(code, 0, text)
        for case in (
            "land-default-integration",
            "mainline-refused",
            "mainline-owner-override",
            "moved-mainline-status",
            "ticket-without-record-branch-refused",
            "launch-from-effort-record",
            "main-checkout-effort-worktree",
        ):
            self.assertIn(f"passes integration-branch {case} expected=checked observed=checked", text)
        default = records["land-default-integration"]["observed"]
        self.assertTrue(default["land_command_uses_effort_record"])
        self.assertEqual(default["integration_after"], default["candidate"])
        self.assertEqual(default["main_after"], default["main_before"])
        refused = records["mainline-refused"]["observed"]
        self.assertEqual(refused["returncode"], 8)
        self.assertEqual(refused["main_after"], refused["main_before"])
        override = records["mainline-owner-override"]["observed"]
        self.assertEqual(override["main_after"], override["candidate"])
        self.assertEqual(override["override_record"]["owner"], "owner")
        status = records["moved-mainline-status"]["observed"]
        self.assertTrue(status["integration_behind_mainline"])
        self.assertEqual(
            [item["ticket"] for item in status["landed_candidates"]],
            ["01", "02"],
        )
        missing_records = records["ticket-without-record-branch-refused"]["observed"]
        self.assertEqual(missing_records["returncode"], 11)
        self.assertEqual(missing_records["integration_after"], missing_records["integration_before"])
        self.assertFalse(missing_records["gate_ran"])
        self.assertIn("effort.py move-records demo", missing_records["stderr"])
        launch = records["launch-from-effort-record"]["observed"]
        self.assertFalse(launch["command_passed_integration"])
        self.assertEqual(launch["recorded_integration"], "effort/demo")
        main_checkout = records["main-checkout-effort-worktree"]["observed"]
        self.assertEqual(main_checkout["integration_after"], main_checkout["candidate"])
        self.assertEqual(main_checkout["main_after"], main_checkout["main_before"])
        self.assertTrue(main_checkout["status_found_effort"])
        self.assertEqual(main_checkout["status_effort_worktree"], main_checkout["record_worktree"])
        self.assertEqual(main_checkout["status_landed_candidates"], [{"ticket": "01", "candidate": main_checkout["candidate"]}])

    def test_rework_round_scenario_runs_through_lead_driven_mode(self):
        code, text = self.run_command("rework-round")

        self.assertEqual(code, 0, text)
        self.assertIn("passes rework-round one-round-accepted expected=checked observed=checked", text)
        self.assertIn("passes rework-round second-review-required expected=checked observed=checked", text)
        self.assertIn("passes rework-round round-cap-and-stale-redo expected=checked observed=checked", text)
        records = self.case_records()
        accepted = records["one-round-accepted"]["observed"]
        self.assertEqual(len(accepted["candidates"]), 2)
        self.assertEqual(accepted["candidates"][0]["review"]["accepted"], False)
        self.assertEqual(accepted["run_candidates"][0]["review"]["delivery_id"], "review-1")
        self.assertEqual(
            accepted["run_candidates"][0]["review"]["findings_sha256"],
            accepted["delivery"]["findings_sha256"],
        )
        self.assertEqual(accepted["layout"], "main-checkout-effort-worktree")
        self.assertFalse(accepted["effort_record_exists_in_command_repo"])
        self.assertNotEqual(accepted["command_repo"], accepted["effort_worktree"])
        self.assertTrue(Path(accepted["candidates"][0]["review_record_path"]).is_relative_to(accepted["effort_worktree"]))
        self.assertTrue(Path(accepted["candidates"][1]["review_record_path"]).is_relative_to(accepted["effort_worktree"]))
        self.assertEqual(accepted["landing"]["target_branch"], "effort/readiness")
        self.assertEqual(accepted["landing"]["observed"], "landed")
        self.assertEqual(accepted["delivery"]["receipt"]["transport"], "resume")
        refused = records["second-review-required"]["observed"]
        self.assertEqual(refused["landing"]["observed"], "acceptance-refused")
        self.assertIn("record names another commit", refused["landing"]["stderr"])
        cap = records["round-cap-and-stale-redo"]["observed"]
        self.assertEqual(cap["rounds"], [1, 1, 2, 3])
        self.assertEqual(cap["counts_toward_cap"], [True, False, True, True])
        self.assertEqual(cap["triage"], "ready-for-human")

    def test_bookkeeping_off_code_branch_scenario_runs_through_lead_driven_mode(self):
        code, text = self.run_command("bookkeeping-off-code-branch")
        records = self.case_records()

        self.assertEqual(code, 0, text)
        self.assertIn(
            "passes bookkeeping-off-code-branch bookkeeping-while-worker-runs expected=checked observed=checked",
            text,
        )
        observed = records["bookkeeping-while-worker-runs"]["observed"]
        self.assertEqual(observed["worker_base"], observed["base"])
        self.assertEqual(observed["candidate_parent"], observed["base"])
        self.assertEqual(observed["integration_after"], observed["land_stdout"])
        self.assertEqual(observed["gate_seen"], observed["land_stdout"])
        self.assertNotEqual(observed["record_branch_after"], observed["integration_after"])
        self.assertTrue(observed["claim_committed"])
        self.assertTrue(observed["resolve_committed"])
        self.assertTrue(observed["resolve_forced_logged"])
        self.assertTrue(observed["acceptance_committed"])
        self.assertEqual(observed["landing_status"], [{"ticket": "01", "candidate": observed["land_stdout"]}])
        self.assertIn("Status: claimed", observed["fresh_history"]["ticket_02"])
        self.assertIn("Status: resolved", observed["fresh_history"]["ticket_03"])
        self.assertIn(observed["accepted_candidate"], observed["fresh_history"]["acceptance_01"])
        self.assertIn(observed["land_stdout"], observed["fresh_history"]["landing_01"])

    def test_parallel_workers_and_overlap_scenario_runs_through_lead_driven_mode(self):
        code, text = self.run_command("parallel-workers-and-overlap")
        records = self.case_records()

        self.assertEqual(code, 0, text)
        self.assertIn("passes parallel-workers-and-overlap two-workers expected=checked observed=checked", text)
        record = records["two-workers"]
        observed = record["observed"]
        self.assertTrue(observed["intervals_overlap"], observed)
        self.assertEqual(observed["landed_tickets"], ["01", "02"])
        self.assertNotEqual(observed["first_02_candidate"], observed["second_02_candidate"])
        self.assertEqual(observed["second_02_base"], observed["after_first_landing"])
        self.assertIn("declared-path-overlap", json.dumps(observed["launch_overlap_findings"]))
        self.assertIn("changed-outside-declared", json.dumps(observed["running_overlap_findings"]))
        self.assertIn("changed-path-declared-by-other", json.dumps(observed["running_overlap_findings"]))
        self.assertIn("common-undeclared-changed-path", json.dumps(observed["running_overlap_findings"]))
        self.assertIn("unaccounted-worktree-change", json.dumps(observed["workspace_status_findings"]))

    def test_shared_repository_safe_scenario_allows_unrelated_branch_movement(self):
        code, text = self.run_command("shared-repository-safe")
        records = self.case_records()

        self.assertEqual(code, 0, text)
        self.assertIn(
            "passes shared-repository-safe unrelated-branches-move expected=ready observed=ready",
            text,
        )
        record = records["unrelated-branches-move"]
        verdict = record["toolbox_verdict"]
        self.assertTrue(verdict["ready_for_review"])
        self.assertNotIn("boundary-violation", verdict["not_ready"])
        branch_findings = [f for f in verdict["findings"] if f["kind"] == "shared-branch-moved"]
        self.assertTrue(branch_findings)
        self.assertTrue(all(f["severity"] == "NOTE" for f in branch_findings))
        self.assertTrue(record["observed"]["decoy_untouched"])

    def test_interactive_launch_and_monitors_scenario_runs_through_lead_driven_mode(self):
        code, text = self.run_command("interactive-launch-and-monitors")
        records = self.case_records()

        self.assertEqual(code, 0, text)
        self.assertIn(
            "passes interactive-launch-and-monitors full-flow expected=checked observed=checked",
            text,
        )
        observed = records["full-flow"]["observed"]
        self.assertEqual(observed["layout"], "main-integration-record-workers")
        self.assertEqual(observed["worker_workspace_count"], 4)
        self.assertEqual(observed["state_sequence"], [
            "running",
            "finishing",
            "rework",
            "waiting for re-integration",
            "ready",
            "landed",
            "needs-info",
            "stopped",
        ])
        self.assertTrue(observed["interactive_window_available"])
        self.assertEqual(observed["trust_override"]["state"], "ready")
        self.assertIsNone(observed["trust_override"]["prompt"])
        self.assertEqual(observed["trust_override"]["wait_trigger"]["state"], "ready")
        self.assertEqual(observed["open_session_windows"], ["tickets", "workers"])
        self.assertEqual(observed["review_delivery"]["receipt"]["transport"], "terminal")
        self.assertTrue(observed["review_delivery"]["receipt"]["acknowledged"])
        self.assertEqual(observed["monitor_private_clone"]["worktree"], observed["ticket_01_worktree"])
        self.assertEqual(observed["landing"]["observed"], "landed")
        self.assertFalse(observed["effort_record_exists_in_main_checkout"])
        self.assertFalse(observed["effort_record_exists_in_integration_worktree"])
        self.assertTrue(observed["effort_record_exists_in_record_worktree"])
        rows = [json.loads(line) for line in (self.out / "readiness-results.jsonl").read_text().splitlines()]
        self.assertEqual(rows[-1]["runtime"], "scripted")
        self.assertEqual(rows[-1]["proof"], "deterministic")
        self.assertEqual(rows[-1]["participants"], ["full-flow"])
        self.assertEqual(rows[-1]["stories"], [1, 2, 4, 14, 15, 27, 28, 29, 32, 34, 37, 38, 44])

    def test_scripted_interactive_participant_trust_check_rejects_dotted_project_key(self):
        script = lead_driven.READINESS / "interactive-launch-and-monitors" / "scripted_participant.py"
        spec = importlib.util.spec_from_file_location("scripted_participant", script)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        workspace = "/data02/home/philip.yang/devx/tmp/trustprobe.with.dot"

        self.assertFalse(
            module.trust_override_applies(
                ["-c", f'projects."{workspace}".trust_level="trusted"', "--yolo"],
                workspace,
            )
        )
        self.assertTrue(
            module.trust_override_applies(
                ["-c", f'projects={{"{workspace}"={{trust_level="trusted"}}}}', "--yolo"],
                workspace,
            )
        )

    def test_scripted_interactive_participant_prompts_when_trust_override_would_not_apply(self):
        script = lead_driven.READINESS / "interactive-launch-and-monitors" / "scripted_participant.py"
        workspace = self.out / "trustprobe.with.dot"
        signals = self.out / "signals"
        stdout = self.out / "trust-prompt.txt"
        workspace.mkdir()
        signals.mkdir()
        env = {**os.environ, "INSULA_INTERACTIVE_SIGNAL_DIR": str(signals)}
        stdout_handle = stdout.open("w")
        proc = subprocess.Popen(
            [
                sys.executable,
                str(script),
                "-c",
                f'projects."{workspace}".trust_level="trusted"',
                "--yolo",
                "Ticket: 04-prompt\n",
            ],
            cwd=workspace,
            env=env,
            stdin=subprocess.PIPE,
            stdout=stdout_handle,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            deadline = time.monotonic() + 3.0
            prompt = ""
            while time.monotonic() < deadline:
                prompt = stdout.read_text(errors="replace") if stdout.exists() else ""
                if "Do you trust the contents of this directory" in prompt:
                    break
                time.sleep(0.05)

            self.assertIn("Do you trust the contents of this directory", prompt)
            self.assertIsNone(proc.poll())
        finally:
            if proc.stdin:
                proc.stdin.close()
            proc.terminate()
            try:
                proc.wait(timeout=3)
            except subprocess.TimeoutExpired:
                proc.kill()
                proc.wait(timeout=3)
            if proc.stderr:
                proc.stderr.close()
            stdout_handle.close()


if __name__ == "__main__":
    unittest.main()
