from __future__ import annotations

import hashlib
import json
import os
import stat
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import protocol
import runner


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_fake_cli(path: Path, mode: str = "success") -> None:
    script = f"""\
#!/usr/bin/env python3
import json
import pathlib
import sys
import time

MODE = {mode!r}

if "--version" in sys.argv:
    print("traecli fake-1.0")
    raise SystemExit(0)

def option(name):
    return sys.argv[sys.argv.index(name) + 1]

prompt = sys.stdin.read()
workspace = pathlib.Path(option("--cd"))
final_path = pathlib.Path(option("--output-last-message"))

if MODE == "timeout":
    time.sleep(30)
if MODE == "failed":
    print(json.dumps({{"type": "error", "message": "fake failure"}}))
    raise SystemExit(7)
if MODE == "malformed":
    print("not-json")
    final_path.write_text("not-json")
    raise SystemExit(0)

role = "historical_root"
for line in prompt.splitlines():
    if line.startswith("CALL_ROLE: "):
        role = line.split(": ", 1)[1]
        break

def payload():
    if role == "historical_root":
        if MODE != "missing_candidate":
            (workspace / "candidate.tex").write_text(
                "\\\\documentclass{{article}}\\n\\\\begin{{document}}Candidate\\\\end{{document}}\\n"
            )
        return {{
            "schema_version": "crouzeix-historical-final/v1",
            "status": "candidate_written",
            "candidate_path": "candidate.tex"
        }}
    if role == "route_worker":
        route_id = next(
            line.split(": ", 1)[1]
            for line in prompt.splitlines()
            if line.startswith("ROUTE_ID: ")
        )
        viable = route_id == "route-1"
        return {{
            "schema_version": "crouzeix-route/v1",
            "route_id": route_id,
            "family": "family-" + route_id,
            "mechanism": "derive a concrete chain for " + route_id,
            "proved_statements": ["bounded intermediate statement"],
            "unproved_obligations": [] if viable else [
                {{"statement": "close the main estimate", "strength": "theorem_strength"}}
            ],
            "circularity_risks": [],
            "candidate_proof": "Candidate from " + route_id if viable else None,
            "blocker": None if viable else "main estimate remains open",
            "confidence_basis": "explicit equations were checked"
        }}
    if role == "controller":
        return {{
            "schema_version": "crouzeix-controller/v1",
            "decisions": [
                {{"route_id": "route-1", "family": "family-route-1", "state": "viable", "reason": "concrete"}},
                {{"route_id": "route-2", "family": "family-route-2", "state": "blocked", "reason": "theorem strength"}},
                {{"route_id": "route-3", "family": "family-route-3", "state": "blocked", "reason": "theorem strength"}}
            ],
            "selected_route_ids": ["route-1"],
            "family_collapse": False,
            "redirect_brief": None
        }}
    if role == "synthesizer":
        return {{
            "schema_version": "crouzeix-synthesis/v1",
            "parent_route_ids": ["route-1"],
            "candidate_proof": "\\\\documentclass{{article}}\\n\\\\begin{{document}}Synthesized candidate\\\\end{{document}}\\n",
            "obligations": []
        }}
    if role in ("logical_critic", "operator_critic"):
        return {{
            "schema_version": "crouzeix-critic/v1",
            "critic_role": "logical" if role == "logical_critic" else "operator_theory",
            "candidate_sha256": next(
                line.split(": ", 1)[1]
                for line in prompt.splitlines()
                if line.startswith("CANDIDATE_SHA256: ")
            ),
            "findings": []
        }}
    if role == "repair":
        parent = next(
            line.split(": ", 1)[1]
            for line in prompt.splitlines()
            if line.startswith("CANDIDATE_SHA256: ")
        )
        return {{
            "schema_version": "crouzeix-repair/v1",
            "parent_candidate_sha256": parent,
            "candidate_proof": "\\\\documentclass{{article}}\\n\\\\begin{{document}}Synthesized candidate\\\\end{{document}}\\n",
            "dispositions": [],
            "unproved_obligations": []
        }}
    raise RuntimeError(role)

final_path.write_text(json.dumps(payload()))
print(json.dumps({{"type": "thread.started", "thread_id": "fake-thread"}}))
if MODE == "incomplete_events":
    raise SystemExit(0)
print(json.dumps({{"type": "turn.started"}}))
print(json.dumps({{"type": "item.completed", "item": {{"type": "agent_message", "text": final_path.read_text()}}}}))
completed = {{"type": "turn.completed"}}
if MODE != "missing_usage":
    completed["usage"] = {{
        "input_tokens": 100,
        "cache_creation_input_tokens": 0,
        "cached_input_tokens": 25,
        "output_tokens": 20,
        "reasoning_output_tokens": 5
    }}
print(json.dumps(completed))
"""
    path.write_text(textwrap.dedent(script))
    path.chmod(0o755)


def write_run(root: Path, arm: str, cli: Path, timeout_seconds: int = 30) -> Path:
    run_dir = root / f"{arm}-001"
    for relative in ("inputs", "schemas", "calls", "candidate", "review"):
        (run_dir / relative).mkdir(parents=True, exist_ok=True)
    if arm == "orchestrated":
        (run_dir / "prompts").mkdir()
    (run_dir / "inputs/theorem.txt").write_text("Prove the stated theorem.")
    (run_dir / "inputs/execution_prompt.txt").write_text(
        "Write a complete proof to candidate.tex."
    )
    schema_names = (
        ["historical_final.schema.json"]
        if arm == "historical"
        else [
            "controller.schema.json",
            "critic.schema.json",
            "repair.schema.json",
            "route.schema.json",
            "synthesis.schema.json",
        ]
    )
    for name in schema_names:
        (run_dir / "schemas" / name).write_bytes(
            (LAB / "schemas" / name).read_bytes()
        )
    if arm == "orchestrated":
        for path in (LAB / "prompts").iterdir():
            (run_dir / "prompts" / path.name).write_bytes(path.read_bytes())
    spec = {
        "schema_version": "crouzeix-run-spec/v1",
        "run_id": run_dir.name,
        "arm": arm,
        "leakage": "L1",
        "model": "gpt-5.6-sol",
        "cli": {
            "path": str(cli.resolve()),
            "version": "traecli fake-1.0",
            "sha256": digest(cli.read_bytes()),
        },
        "historical_prompt": {
            "source_bytes": 4106,
            "source_sha256": protocol.HISTORICAL_PROMPT_SHA256,
            "execution_bytes": 4060,
            "execution_sha256": "e" * 64,
            "normalization": "replace path",
        },
        "sandbox": "workspace-write",
        "approval_policy": "never",
        "allowed_tools": ["Read", "Write"],
        "network_access": False,
        "timeout_seconds": timeout_seconds,
        "max_calls": 1 if arm == "historical" else 9,
        "token_accounting": {
            "boundary": "all provider calls launched by this run",
        },
        "generation_visible_files": ["inputs/execution_prompt.txt"],
        "generation_excluded_classes": ["public proof manuscripts"],
        "created_at_utc": "2026-08-15T04:00:00Z",
    }
    protocol.validate_run_spec(spec)
    (run_dir / "run_spec.json").write_text(
        json.dumps(spec, indent=2, sort_keys=True) + "\n"
    )
    return run_dir


class CommandTests(unittest.TestCase):
    def test_build_command_pins_identity_access_and_output_contract(self) -> None:
        spec = {
            "cli": {"path": "/opt/traecli"},
            "sandbox": "workspace-write",
            "approval_policy": "never",
            "model": "gpt-5.6-sol",
            "allowed_tools": ["Read", "Write"],
        }

        command = runner.build_command(
            spec,
            workspace=Path("/tmp/workspace"),
            schema=Path("/tmp/schema.json"),
            final=Path("/tmp/final.json"),
        )

        self.assertEqual(command[:2], ["/opt/traecli", "exec"])
        for required in [
            "--ignore-user-config",
            "--ignore-rules",
            "--ephemeral",
            "--sandbox",
            "workspace-write",
            "--config",
            'approval_policy="never"',
            "--model",
            "gpt-5.6-sol",
            "--skip-git-repo-check",
            "--output-schema",
            "/tmp/schema.json",
            "--output-last-message",
            "/tmp/final.json",
            "--json",
        ]:
            self.assertIn(required, command)
        self.assertNotIn("--search", command)
        self.assertEqual(command[-1], "-")
        allowed = [
            command[index + 1]
            for index, item in enumerate(command)
            if item == "--allowed-tool"
        ]
        self.assertEqual(allowed, ["Read", "Write"])


class CallTests(unittest.TestCase):
    def test_call_roles_include_distinct_review_provider_roles(self) -> None:
        self.assertIn("proof_progress_evaluator", runner.CALL_ROLES)
        self.assertIn("correctness_review", runner.CALL_ROLES)
        self.assertIn("mechanism_classification", runner.CALL_ROLES)

    def test_completed_call_records_exact_prompt_events_usage_and_empty_workspace(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_cli.py"
            write_fake_cli(cli)
            run_dir = write_run(root, "historical", cli)
            spec = protocol.read_run_spec(run_dir / "run_spec.json")

            result = runner.run_call(
                run_dir,
                spec,
                call_id="historical-root",
                role="historical_root",
                prompt="CALL_ROLE: historical_root\nDo the work.",
                schema_path=run_dir / "schemas/historical_final.schema.json",
                parent_digests={},
            )

            self.assertEqual(result["receipt"]["status"], "completed")
            self.assertEqual(result["receipt"]["session_id"], "fake-thread")
            self.assertEqual(result["receipt"]["usage"]["input_tokens"], 100)
            self.assertEqual(
                result["receipt"]["prompt_sha256"],
                digest((result["call_dir"] / "prompt.md").read_bytes()),
            )
            self.assertEqual(
                sorted(path.name for path in (result["call_dir"] / "workspace").iterdir()),
                ["candidate.tex"],
            )
            self.assertEqual(
                [json.loads(line)["type"] for line in (result["call_dir"] / "events.jsonl").read_text().splitlines()],
                ["thread.started", "turn.started", "item.completed", "turn.completed"],
            )

            with self.assertRaisesRegex(protocol.ValidationError, "existing"):
                runner.run_call(
                    run_dir,
                    spec,
                    call_id="historical-root",
                    role="historical_root",
                    prompt="again",
                    schema_path=run_dir / "schemas/historical_final.schema.json",
                    parent_digests={},
                )

    def test_cli_identity_drift_fails_before_call_directory_creation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_cli.py"
            write_fake_cli(cli)
            run_dir = write_run(root, "historical", cli)
            spec = protocol.read_run_spec(run_dir / "run_spec.json")
            cli.write_text("#!/bin/sh\nexit 0\n")

            with self.assertRaisesRegex(protocol.ValidationError, "identity"):
                runner.run_call(
                    run_dir,
                    spec,
                    call_id="historical-root",
                    role="historical_root",
                    prompt="prompt",
                    schema_path=run_dir / "schemas/historical_final.schema.json",
                    parent_digests={},
                )

            self.assertFalse((run_dir / "calls/historical-root").exists())

    def test_failed_timeout_malformed_and_missing_usage_are_typed(self) -> None:
        for mode, expected in [
            ("failed", "failed"),
            ("timeout", "timed_out"),
            ("malformed", "malformed"),
            ("incomplete_events", "malformed"),
            ("missing_usage", "completed"),
        ]:
            with self.subTest(mode=mode):
                with tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    cli = root / "fake_cli.py"
                    write_fake_cli(cli, mode)
                    run_dir = write_run(root, "historical", cli)
                    spec = protocol.read_run_spec(run_dir / "run_spec.json")
                    spec["timeout_seconds"] = 1

                    result = runner.run_call(
                        run_dir,
                        spec,
                        call_id="historical-root",
                        role="historical_root",
                        prompt="CALL_ROLE: historical_root\nprompt",
                        schema_path=run_dir / "schemas/historical_final.schema.json",
                        parent_digests={},
                    )

                    self.assertEqual(result["receipt"]["status"], expected)
                    if mode == "missing_usage":
                        self.assertIsNone(result["receipt"]["usage"])
                    self.assertTrue((result["call_dir"] / "receipt.json").is_file())

    def test_run_call_binds_ticket_and_appends_receipt_before_post_call_hook(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_cli.py"
            write_fake_cli(cli)
            run_dir = write_run(root, "historical", cli)
            spec = protocol.read_run_spec(run_dir / "run_spec.json")
            accounting = run_dir / "attempt_ledger.jsonl"
            marker = run_dir / "post-call-marker"
            ticket_binding = {
                "ticket_id": "expert-g0-function-theory",
                "ticket_sha256": "a" * 64,
            }

            with self.assertRaisesRegex(RuntimeError, "post-call failed"):
                runner.run_call(
                    run_dir,
                    spec,
                    call_id="historical-root",
                    role="historical_root",
                    prompt="CALL_ROLE: historical_root\nprompt",
                    schema_path=run_dir / "schemas/historical_final.schema.json",
                    parent_digests={},
                    ticket_binding=ticket_binding,
                    allowed_tools=["Write"],
                    accounting_path=accounting,
                    after_receipt=lambda: (
                        marker.write_text("after"),
                        (_ for _ in ()).throw(RuntimeError("post-call failed")),
                    ),
                )

            receipt = json.loads((run_dir / "calls/historical-root/receipt.json").read_text())
            self.assertEqual(receipt["ticket_id"], ticket_binding["ticket_id"])
            self.assertEqual(receipt["ticket_sha256"], ticket_binding["ticket_sha256"])
            self.assertEqual(receipt["allowed_tools"], ["Write"])
            ledger = [json.loads(line) for line in accounting.read_text().splitlines()]
            self.assertEqual(len(ledger), 1)
            self.assertEqual(ledger[0]["call_id"], "historical-root")
            self.assertEqual(ledger[0]["ticket_id"], ticket_binding["ticket_id"])
            self.assertTrue(marker.exists())

    def test_role_output_validation_rejects_unknown_or_inconsistent_fields(self) -> None:
        valid = {
            "schema_version": "crouzeix-historical-final/v1",
            "status": "candidate_written",
            "candidate_path": "candidate.tex",
        }
        self.assertEqual(
            runner.validate_role_output("historical_root", valid),
            valid,
        )
        with self.assertRaisesRegex(protocol.ValidationError, "fields"):
            runner.validate_role_output(
                "historical_root",
                {**valid, "claim": "complete"},
            )
        with self.assertRaisesRegex(protocol.ValidationError, "inconsistent"):
            runner.validate_role_output(
                "historical_root",
                {
                    "schema_version": "crouzeix-historical-final/v1",
                    "status": "no_candidate",
                    "candidate_path": "candidate.tex",
                },
            )

        route = {
            "schema_version": "crouzeix-route/v1",
            "route_id": "route-1",
            "family": "operator route",
            "mechanism": "concrete mechanism",
            "proved_statements": ["lemma"],
            "unproved_obligations": [],
            "circularity_risks": [],
            "candidate_proof": None,
            "blocker": "endpoint remains open",
            "confidence_basis": "bounded derivation",
        }
        self.assertEqual(runner.validate_role_output("route_worker", route), route)
        with self.assertRaisesRegex(protocol.ValidationError, "candidate or blocker"):
            runner.validate_role_output(
                "route_worker",
                {**route, "blocker": None},
            )


class ExperimentTests(unittest.TestCase):
    def test_historical_arm_makes_one_call_and_freezes_candidate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_cli.py"
            write_fake_cli(cli)
            run_dir = write_run(root, "historical", cli)

            receipt = runner.run_experiment(run_dir)

            self.assertEqual(receipt["arm"], "historical")
            self.assertEqual(receipt["call_count"], 1)
            self.assertEqual(receipt["call_status_counts"], {"completed": 1})
            self.assertEqual(receipt["usage"]["input_tokens"], 100)
            self.assertIsNotNone(receipt["candidate_sha256"])
            self.assertTrue((run_dir / "candidate/candidate.tex").is_file())
            self.assertEqual(len(list((run_dir / "calls").iterdir())), 1)

    def test_historical_missing_candidate_is_retained_as_not_promoted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_cli.py"
            write_fake_cli(cli, "missing_candidate")
            run_dir = write_run(root, "historical", cli)

            receipt = runner.run_experiment(run_dir)

            self.assertEqual(receipt["promotion"], "not_promoted")
            self.assertIsNone(receipt["candidate_sha256"])
            self.assertEqual(receipt["call_status_counts"], {"malformed": 1})
            call_receipt = json.loads(
                (run_dir / "calls/historical-root/receipt.json").read_text()
            )
            self.assertIn("candidate", call_receipt["parse_error"])
            self.assertTrue((run_dir / "run_receipt.json").is_file())

    def test_orchestrated_arm_preserves_initial_independence_and_promotes_deterministically(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_cli.py"
            write_fake_cli(cli)
            run_dir = write_run(root, "orchestrated", cli)

            receipt = runner.run_experiment(run_dir)

            self.assertEqual(receipt["arm"], "orchestrated")
            self.assertEqual(receipt["call_count"], 8)
            self.assertEqual(receipt["promotion"], "promoted")
            self.assertEqual(receipt["usage"]["input_tokens"], 800)
            self.assertTrue((run_dir / "candidate/candidate.tex").is_file())
            events = [
                json.loads(line)
                for line in (run_dir / "route_events.jsonl").read_text().splitlines()
            ]
            self.assertEqual(events[-1]["to_state"], "promoted")
            worker_prompts = [
                (run_dir / f"calls/route-worker-{index}/prompt.md").read_text()
                for index in range(1, 4)
            ]
            for index, prompt in enumerate(worker_prompts, 1):
                self.assertIn(f"ROUTE_ID: route-{index}", prompt)
                for peer in range(1, 4):
                    if peer != index:
                        self.assertNotIn(f"Candidate from route-{peer}", prompt)
            controller_prompt = (
                run_dir / "calls/controller/prompt.md"
            ).read_text()
            self.assertIn("route-1", controller_prompt)
            self.assertIn("route-2", controller_prompt)
            self.assertIn("route-3", controller_prompt)
            self.assertFalse((run_dir / "calls/redirect").exists())

    def test_orchestrated_phase_failure_seals_partial_run_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_cli.py"
            write_fake_cli(cli, "malformed")
            run_dir = write_run(root, "orchestrated", cli)

            receipt = runner.run_experiment(run_dir)

            self.assertEqual(receipt["execution_status"], "failed")
            self.assertEqual(receipt["promotion"], "not_promoted")
            self.assertEqual(receipt["call_count"], 1)
            self.assertEqual(receipt["call_status_counts"], {"malformed": 1})
            self.assertIn("route-worker-1", receipt["failure"])
            self.assertTrue((run_dir / "run_receipt.json").is_file())

    def test_aggregate_usage_preserves_unknown_components(self) -> None:
        receipts = [
            {"status": "completed", "usage": {"input_tokens": 10, "output_tokens": 2}},
            {"status": "completed", "usage": None},
        ]

        result = runner.aggregate_usage(receipts)

        self.assertEqual(result["input_tokens"], 10)
        self.assertEqual(result["output_tokens"], 2)
        self.assertIsNone(result["cached_input_tokens"])
        self.assertEqual(result["calls_without_usage"], 1)


if __name__ == "__main__":
    unittest.main()
