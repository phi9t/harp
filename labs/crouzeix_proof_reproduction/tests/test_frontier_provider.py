from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import textwrap
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
TESTS = Path(__file__).resolve().parent
sys.path.insert(0, str(LAB))
sys.path.insert(0, str(TESTS))

import expert_contracts
import frontier_provider
import protocol
import tickets
from test_tickets import runtime_ticket


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def write_fake_frontier_cli(path: Path, mode: str = "expert") -> None:
    script = f"""\
#!/usr/bin/env python3
import json
import pathlib
import sys
import time

MODE = {mode!r}

if "--version" in sys.argv:
    print("traecli frontier-fake-1.0")
    raise SystemExit(0)

def option(name):
    return sys.argv[sys.argv.index(name) + 1]

def options(name):
    return [
        sys.argv[index + 1]
        for index, item in enumerate(sys.argv)
        if item == name
    ]

workspace = pathlib.Path(option("--cd"))
final_path = pathlib.Path(option("--output-last-message"))
prompt = sys.stdin.read()

assert "--ignore-user-config" in sys.argv
assert "--ignore-rules" in sys.argv
assert "--ephemeral" in sys.argv
assert "--json" in sys.argv
assert "--skip-git-repo-check" in sys.argv
assert "--search" not in sys.argv
assert "--mcp" not in sys.argv
assert options("--allowed-tool") == ["Write"], options("--allowed-tool")
assert sorted(item.name for item in workspace.iterdir()) == []

prefix = "FRONTIER_CONTEXT_JSON:"
context_line = next(line for line in prompt.splitlines() if line.startswith(prefix))
context = json.loads(context_line.removeprefix(prefix).strip())
assert context["delegation_allowed"] is False
assert context["allowed_tools"] == ["Write"]
for forbidden in ["Read", "Bash", "Glob", "Grep", "Edit", "spawn_agent", "WebSearch", "MCP"]:
    assert forbidden not in context["allowed_tools"]
assert context["ticket_id"] == "expert-g0-function-theory"
assert isinstance(context["ticket_sha256"], str) and len(context["ticket_sha256"]) == 64
assert isinstance(context["schema_sha256"], str) and len(context["schema_sha256"]) == 64
assert isinstance(context["prompt_sha256"], str) and len(context["prompt_sha256"]) == 64
assert context["parent_artifact_sha256"] is None

if MODE == "timeout":
    time.sleep(30)
if MODE == "failed":
    print(json.dumps({{"type": "thread.started", "thread_id": "frontier-thread"}}))
    print(json.dumps({{"type": "turn.completed"}}))
    raise SystemExit(9)
if MODE == "malformed":
    print(json.dumps({{"type": "thread.started", "thread_id": "frontier-thread"}}))
    print(json.dumps({{"type": "turn.completed"}}))
    final_path.write_text("not-json")
    raise SystemExit(0)
if MODE == "blocked":
    print(json.dumps({{"type": "thread.started", "thread_id": "frontier-thread"}}))
    print(json.dumps({{"type": "turn.completed"}}))
    final_path.write_text(json.dumps({{"schema_version": "frontier-resource-block/v1", "resource": "disk"}}))
    raise SystemExit(75)

payload = {{
    "schema_version": "crouzeix-expert-result/v1",
    "run_id": "expert-frontier-001",
    "attempt_id": "attempt-g0-function-theory",
    "ticket_id": "expert-g0-function-theory",
    "proposed_node_id": "node-g0-function-theory",
    "parent_node_id": None,
    "parent_node_artifact_sha256": None,
    "generation": 0,
    "expert_role": "function_theory",
    "selected_direction_id": "root-function-theory",
    "mathematical_payload": {{
        "proof_family": "functional calculus",
        "mechanism": "derive a bounded numerical range estimate",
        "proved_statements": [
            {{
                "statement_id": "s1",
                "statement": "A bounded intermediate estimate follows.",
                "justification": "The construction is explicit.",
                "depends_on_statement_ids": []
            }}
        ],
        "unproved_obligations": [],
        "circularity_risks": [],
        "proposed_directions": [],
        "endpoint": {{"kind": "blocker", "text": "The final constant remains open."}},
        "confidence_basis": "checked algebraic steps"
    }}
}}
final_path.write_text(json.dumps(payload))
print(json.dumps({{"type": "thread.started", "thread_id": "frontier-thread"}}))
print(json.dumps({{"type": "turn.started"}}))
print(json.dumps({{"type": "item.completed", "item": {{"type": "agent_message", "text": final_path.read_text()}}}}))
print(json.dumps({{
    "type": "turn.completed",
    "usage": {{
        "input_tokens": 11,
        "cache_creation_input_tokens": 0,
        "cached_input_tokens": 0,
        "output_tokens": 7,
        "reasoning_output_tokens": 3
    }}
}}))
"""
    path.write_text(textwrap.dedent(script))
    path.chmod(0o755)


def write_frontier_run(root: Path, cli: Path, timeout_seconds: int = 30) -> Path:
    run_dir = root / "expert-frontier-001"
    for relative in ("calls", "schemas"):
        (run_dir / relative).mkdir(parents=True)
    schema = LAB / "schemas/expert_result.schema.json"
    (run_dir / "schemas/expert_result.schema.json").write_bytes(schema.read_bytes())
    spec = {
        "schema_version": "crouzeix-run-spec/v1",
        "run_id": run_dir.name,
        "arm": "guided",
        "leakage": "L1",
        "model": "gpt-5.6-sol",
        "cli": {
            "path": str(cli.resolve()),
            "version": "traecli frontier-fake-1.0",
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
        "max_calls": 12,
        "token_accounting": {
            "boundary": "all provider calls launched by this run",
        },
        "generation_visible_files": ["inputs/theorem.txt"],
        "generation_excluded_classes": ["public proof manuscripts"],
        "created_at_utc": "2026-08-15T04:00:00Z",
    }
    protocol.validate_run_spec(spec)
    (run_dir / "run_spec.json").write_text(
        json.dumps(spec, indent=2, sort_keys=True) + "\n"
    )
    return run_dir


def frontier_ticket(prompt: str, schema_path: Path, **overrides: object) -> dict[str, object]:
    value = runtime_ticket(
        prompt_sha256=digest(prompt.encode("utf-8")),
        schema_sha256=digest(schema_path.read_bytes()),
        forbidden_sources=[
            "search",
            "network",
            "mcp",
            "shell",
            "read",
            "delegation",
            "public proof manuscripts",
        ],
        **overrides,
    )
    tickets.validate_runtime_ticket(value)
    return value


class FrontierProviderTests(unittest.TestCase):
    def test_expert_call_binds_ticket_and_runs_in_write_only_empty_workspace(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_frontier_cli.py"
            write_fake_frontier_cli(cli)
            run_dir = write_frontier_run(root, cli)
            prompt = "Attempt the function-theory route."
            schema = run_dir / "schemas/expert_result.schema.json"
            ticket = frontier_ticket(prompt, schema)
            accounting = run_dir / "attempt_ledger.jsonl"

            result = frontier_provider.run_frontier_call(
                run_dir,
                ticket=ticket,
                call_id="expert-g0-function-theory",
                role="expert",
                prompt=prompt,
                context={
                    "ticket_id": ticket["ticket_id"],
                    "delegation_allowed": False,
                },
                schema_path=schema,
                accounting_path=accounting,
            )

            self.assertEqual(result["receipt"]["status"], "completed")
            self.assertEqual(result["receipt"]["ticket_id"], ticket["ticket_id"])
            self.assertEqual(
                result["receipt"]["ticket_sha256"], tickets.canonical_sha256(ticket)
            )
            self.assertEqual(result["receipt"]["allowed_tools"], ["Write"])
            self.assertEqual(result["receipt"]["model"], "gpt-5.6-sol")
            self.assertEqual(result["receipt"]["sandbox"], "workspace-write")
            self.assertEqual(result["receipt"]["approval_policy"], "never")
            self.assertEqual(
                result["receipt"]["ticket_context_sha256"], ticket["context_sha256"]
            )
            self.assertEqual(
                result["receipt"]["ticket_schema_sha256"], ticket["schema_sha256"]
            )
            self.assertEqual(
                result["receipt"]["ticket_prompt_sha256"], ticket["prompt_sha256"]
            )
            self.assertIsNone(result["receipt"]["ticket_parent_artifact_sha256"])
            self.assertEqual(result["receipt"]["blocked_reason"], None)
            self.assertEqual(
                result["validated_output"]["expert_result_sha256"],
                expert_contracts.validate_expert_result(result["final"])[
                    "expert_result_sha256"
                ],
            )
            request = json.loads(
                (result["call_dir"] / "request.json").read_text()
            )
            self.assertEqual(request["ticket_id"], ticket["ticket_id"])
            self.assertEqual(request["ticket_sha256"], tickets.canonical_sha256(ticket))
            self.assertEqual(request["allowed_tools"], ["Write"])
            self.assertEqual(request["model"], "gpt-5.6-sol")
            self.assertEqual(request["sandbox"], "workspace-write")
            self.assertEqual(request["approval_policy"], "never")
            self.assertEqual(
                sorted(path.name for path in (result["call_dir"] / "workspace").iterdir()),
                [],
            )
            ledger = [
                json.loads(line)
                for line in accounting.read_text().splitlines()
            ]
            self.assertEqual(len(ledger), 1)
            self.assertEqual(ledger[0]["status"], "completed")
            self.assertEqual(ledger[0]["ticket_id"], ticket["ticket_id"])

    def test_invalid_ticket_binding_fails_before_call_directory_creation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_frontier_cli.py"
            write_fake_frontier_cli(cli)
            run_dir = write_frontier_run(root, cli)
            prompt = "Attempt the function-theory route."
            ticket = runtime_ticket(prompt_sha256="0" * 64)

            with self.assertRaisesRegex(protocol.ValidationError, "ticket"):
                frontier_provider.run_frontier_call(
                    run_dir,
                    ticket=ticket,
                    call_id="expert-g0-function-theory",
                    role="expert",
                    prompt=prompt,
                    context={"ticket_id": ticket["ticket_id"]},
                    schema_path=run_dir / "schemas/expert_result.schema.json",
                    accounting_path=run_dir / "attempt_ledger.jsonl",
                )

            self.assertFalse((run_dir / "calls/expert-g0-function-theory").exists())

    def test_terminal_failure_receipts_are_appended_before_resource_checks(self) -> None:
        for mode, expected in [
            ("malformed", "malformed"),
            ("failed", "failed"),
            ("blocked", "blocked_resource"),
        ]:
            with self.subTest(mode=mode):
                with tempfile.TemporaryDirectory() as directory:
                    root = Path(directory)
                    cli = root / "fake_frontier_cli.py"
                    write_fake_frontier_cli(cli, mode)
                    run_dir = write_frontier_run(root, cli)
                    prompt = "Attempt the function-theory route."
                    schema = run_dir / "schemas/expert_result.schema.json"
                    ticket = frontier_ticket(prompt, schema)
                    resource_probe = run_dir / "resource_probe.txt"

                    result = frontier_provider.run_frontier_call(
                        run_dir,
                        ticket=ticket,
                        call_id="expert-g0-function-theory",
                        role="expert",
                        prompt=prompt,
                        context={
                            "ticket_id": ticket["ticket_id"],
                            "delegation_allowed": False,
                        },
                        schema_path=run_dir / "schemas/expert_result.schema.json",
                        accounting_path=run_dir / "attempt_ledger.jsonl",
                        after_receipt=lambda: resource_probe.write_text("after"),
                    )

                    self.assertEqual(result["receipt"]["status"], expected)
                    ledger = [
                        json.loads(line)
                        for line in (run_dir / "attempt_ledger.jsonl").read_text().splitlines()
                    ]
                    self.assertEqual(ledger[0]["status"], expected)
                    self.assertFalse(resource_probe.exists())

    def test_frontier_access_policy_rejects_read_network_and_delegation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_frontier_cli.py"
            write_fake_frontier_cli(cli)
            run_dir = write_frontier_run(root, cli)
            prompt = "Attempt the function-theory route."
            schema = run_dir / "schemas/expert_result.schema.json"
            ticket = frontier_ticket(prompt, schema, allowed_tools=["Read", "Write"])

            with self.assertRaisesRegex(protocol.ValidationError, "Write"):
                frontier_provider.run_frontier_call(
                    run_dir,
                    ticket=ticket,
                    call_id="expert-g0-function-theory",
                    role="expert",
                    prompt="Attempt the function-theory route.",
                    context={
                        "ticket_id": ticket["ticket_id"],
                        "delegation_allowed": False,
                    },
                    schema_path=run_dir / "schemas/expert_result.schema.json",
                    accounting_path=run_dir / "attempt_ledger.jsonl",
                )


if __name__ == "__main__":
    unittest.main()
