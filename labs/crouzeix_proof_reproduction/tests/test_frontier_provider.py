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


def canonical_json_bytes(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")


def digest_json(value: object) -> str:
    return digest(canonical_json_bytes(value))


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
if MODE == "evaluator":
    assert set(context) == {{"schema_version", "theorem_text", "mathematical_payload", "probe_ids"}}
    assert context["schema_version"] == "crouzeix-evaluator-provider-context/v1"
else:
    assert context["delegation_allowed"] is False
    assert context["allowed_tools"] == ["Write"]
    for forbidden in ["Read", "Bash", "Glob", "Grep", "Edit", "spawn_agent", "WebSearch", "MCP"]:
        assert forbidden not in context["allowed_tools"]
    expected_ticket = "expert-g1-d0" if MODE == "child_expert" else "expert-g0-function-theory"
    assert context["ticket_id"] == expected_ticket
    assert context["schema_version"] == "crouzeix-expert-context/v1"
    assert isinstance(context["schema_sha256"], str) and len(context["schema_sha256"]) == 64
    assert isinstance(context["prompt_sha256"], str) and len(context["prompt_sha256"]) == 64
    assert context["parent_artifact_sha256"] is None or len(context["parent_artifact_sha256"]) == 64

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
if MODE == "evaluator":
    payload = {{
        "schema_version": "crouzeix-node-evaluation-payload/v1",
        "probes": [
            {{"probe_id": probe_id, "status": "insufficient_evidence", "rationale": "bounded fake evaluation"}}
            for probe_id in [
                "p01_theorem_statement_preserved",
                "p02_no_theorem_strength_reduction",
                "p03_core_mechanism_explicit",
                "p04_local_claims_justified",
                "p05_dependencies_closed",
                "p06_circularity_addressed",
                "p07_obligations_listed",
                "p08_candidate_text_coherent",
                "p09_blockers_falsifiable",
                "p10_candidate_proof_present",
            ]
        ],
        "findings": [],
    }}
    final_path.write_text(json.dumps(payload))
    print(json.dumps({{"type": "thread.started", "thread_id": "frontier-thread"}}))
    print(json.dumps({{"type": "turn.started"}}))
    print(json.dumps({{"type": "item.completed", "item": {{"type": "agent_message", "text": final_path.read_text()}}}}))
    print(json.dumps({{"type": "turn.completed", "usage": {{"input_tokens": 11, "output_tokens": 7}}}}))
    raise SystemExit(0)

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
if MODE == "wrong_ticket":
    payload["ticket_id"] = "expert-g0-other"
if MODE == "child_expert":
    payload["attempt_id"] = "attempt-g1-d0"
    payload["ticket_id"] = "expert-g1-d0"
    payload["proposed_node_id"] = "node-g1-d0"
    payload["parent_node_id"] = "node-g0-function-theory"
    payload["parent_node_artifact_sha256"] = "1" * 64
    payload["generation"] = 1
    payload["selected_direction_id"] = "dir-child-obligation"
without_digest = dict(payload)
payload["expert_result_sha256"] = __import__("hashlib").sha256(
    json.dumps(without_digest, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")
).hexdigest()
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
    evaluator_schema = LAB / "schemas/node_evaluation_payload.schema.json"
    (run_dir / "schemas/node_evaluation_payload.schema.json").write_bytes(
        evaluator_schema.read_bytes()
    )
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


def root_direction(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "direction_id": "root-function-theory",
        "kind": "root_task",
        "statement": "Develop an independent function-theory route.",
        "strength": "root",
        "recommended_role": "function_theory",
        "source_parent_node_id": None,
        "source_node_artifact_sha256": None,
        "source_reconciliation_sha256": None,
    }
    value.update(overrides)
    return value


def strict_expert_context(prompt: str, schema_path: Path, **overrides: object) -> dict[str, object]:
    theorem = "For every square matrix A, prove the required norm bound."
    value: dict[str, object] = {
        "schema_version": "crouzeix-expert-context/v1",
        "run_id": "expert-frontier-001",
        "attempt_id": "attempt-g0-function-theory",
        "ticket_id": "expert-g0-function-theory",
        "proposed_node_id": "node-g0-function-theory",
        "parent": None,
        "generation": 0,
        "expert_role": "function_theory",
        "selected_direction": root_direction(),
        "theorem_text": theorem,
        "theorem_sha256": digest(theorem.encode("utf-8")),
        "forbidden_sources": [
            "search",
            "network",
            "mcp",
            "shell",
            "read",
            "delegation",
            "public proof manuscripts",
        ],
        "forbidden_tools": [
            "Read",
            "Glob",
            "Grep",
            "Bash",
            "Edit",
            "spawn_agent",
            "WebSearch",
            "MCP",
            "Shell",
        ],
        "allowed_tools": ["Write"],
        "delegation_allowed": False,
        "limits": {
            "timeout_seconds": 3600,
            "max_output_bytes": 1048576,
        },
        "functioning_criteria": "candidate or falsifiable blocker with concrete progress",
        "completion_criteria": "return one strict expert_result",
        "result_schema": "expert_result",
        "schema_sha256": digest(schema_path.read_bytes()),
        "prompt_sha256": digest(prompt.encode("utf-8")),
        "parent_artifact_sha256": None,
        "allowed_parent_artifacts": [],
    }
    value.update(overrides)
    return value


def projected_parent_node() -> dict[str, object]:
    return {
        "node_id": "node-g0-function-theory",
        "node_artifact_sha256": "1" * 64,
        "mathematical_payload": {
            "proof_family": "functional calculus",
            "mechanism": "derive a bounded numerical range estimate",
            "proved_statements": [
                {
                    "statement_id": "s1",
                    "statement": "A bounded intermediate estimate follows.",
                    "justification": "The construction is explicit.",
                    "depends_on_statement_ids": [],
                }
            ],
            "unproved_obligations": [],
            "circularity_risks": [],
            "proposed_directions": [],
            "endpoint": {"kind": "blocker", "text": "The final constant remains open."},
            "confidence_basis": "checked algebraic steps",
        },
    }


def child_expert_context(prompt: str, schema_path: Path) -> dict[str, object]:
    parent = projected_parent_node()
    return strict_expert_context(
        prompt,
        schema_path,
        attempt_id="attempt-g1-d0",
        ticket_id="expert-g1-d0",
        proposed_node_id="node-g1-d0",
        parent=parent,
        generation=1,
        selected_direction=root_direction(
            direction_id="dir-child-obligation",
            kind="obligation",
            statement="Close the parent blocker.",
            strength="major",
            source_parent_node_id="node-g0-function-theory",
            source_node_artifact_sha256="1" * 64,
        ),
        parent_artifact_sha256="1" * 64,
        allowed_parent_artifacts=["1" * 64],
    )


def strict_evaluator_context(prompt: str, schema_path: Path, **overrides: object) -> dict[str, object]:
    theorem = "For every square matrix A, prove the required norm bound."
    value: dict[str, object] = {
        "schema_version": "crouzeix-evaluator-context/v1",
        "run_id": "expert-frontier-001",
        "evaluation_id": "eval-node-g0-function-theory-e1",
        "evaluator_index": 1,
        "ticket_id": "evaluate-node-g0-function-theory-e1",
        "node_artifact_sha256": "1" * 64,
        "mathematical_payload_sha256": "2" * 64,
        "theorem_text": theorem,
        "theorem_sha256": digest(theorem.encode("utf-8")),
        "mathematical_payload": {
            "proof_family": "functional calculus",
            "mechanism": "derive a bounded numerical range estimate",
            "proved_statements": [
                {
                    "statement_id": "s1",
                    "statement": "A bounded intermediate estimate follows.",
                    "justification": "The construction is explicit.",
                    "depends_on_statement_ids": [],
                }
            ],
            "unproved_obligations": [],
            "circularity_risks": [],
            "proposed_directions": [],
            "endpoint": {"kind": "blocker", "text": "The final constant remains open."},
            "confidence_basis": "checked algebraic steps",
        },
        "probe_ids": list(expert_contracts.PROOF_PROGRESS_PROBE_IDS),
        "forbidden_sources": [
            "search",
            "network",
            "mcp",
            "shell",
            "read",
            "delegation",
            "public proof manuscripts",
        ],
        "forbidden_tools": [
            "Read",
            "Glob",
            "Grep",
            "Bash",
            "Edit",
            "spawn_agent",
            "WebSearch",
            "MCP",
            "Shell",
        ],
        "allowed_tools": ["Write"],
        "delegation_allowed": False,
        "completion_criteria": "return one strict node evaluation payload",
        "result_schema": "node_evaluation_payload",
        "schema_sha256": digest(schema_path.read_bytes()),
        "prompt_sha256": digest(prompt.encode("utf-8")),
        "parent_artifact_sha256": "1" * 64,
        "allowed_parent_artifacts": ["1" * 64],
    }
    value.update(overrides)
    return value


def frontier_ticket(
    prompt: str,
    schema_path: Path,
    *,
    context: dict[str, object] | None = None,
    **overrides: object,
) -> dict[str, object]:
    context_value = context if context is not None else strict_expert_context(prompt, schema_path)
    value = runtime_ticket(
        context_sha256=digest_json(context_value),
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
            context = strict_expert_context(prompt, schema)
            ticket = frontier_ticket(prompt, schema, context=context)
            accounting = run_dir / "attempt_ledger.jsonl"

            result = frontier_provider.run_frontier_call(
                run_dir,
                ticket=ticket,
                call_id="expert-g0-function-theory",
                role="expert",
                prompt=prompt,
                context=context,
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
            self.assertFalse(request["network_access"])
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

    def test_evaluator_context_and_payload_are_bound_to_ticket_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_frontier_cli.py"
            write_fake_frontier_cli(cli, "evaluator")
            run_dir = write_frontier_run(root, cli)
            prompt = "Evaluate the sealed node."
            schema = run_dir / "schemas/expert_result.schema.json"
            context = strict_evaluator_context(prompt, schema)
            ticket = frontier_ticket(
                prompt,
                schema,
                context=context,
                ticket_id="evaluate-node-g0-function-theory-e1",
                task_kind="evaluator",
                node_id="node-g0-function-theory",
                generation=0,
                direction_id=None,
                role="proof_progress_evaluator",
                parent_artifact_sha256="1" * 64,
                owner_type="evaluator",
            )

            result = frontier_provider.run_frontier_call(
                run_dir,
                ticket=ticket,
                call_id="evaluate-node-g0-function-theory-e1",
                role="proof_progress_evaluator",
                prompt=prompt,
                context=context,
                schema_path=schema,
                accounting_path=run_dir / "attempt_ledger.jsonl",
            )

            self.assertEqual(result["receipt"]["status"], "completed")
            self.assertEqual(
                result["validated_output"]["schema_version"],
                "crouzeix-node-evaluation-payload/v1",
            )
            self.assertEqual(result["receipt"]["ticket_id"], ticket["ticket_id"])
            self.assertEqual(
                result["receipt"]["ticket_context_sha256"], ticket["context_sha256"]
            )
            self.assertEqual(
                result["receipt"]["ticket_context_sha256"], digest_json(context)
            )
            request = json.loads((result["call_dir"] / "request.json").read_text())
            self.assertEqual(request["ticket_context_sha256"], digest_json(context))
            self.assertEqual(request["ticket_schema_sha256"], ticket["schema_sha256"])
            prompt_text = (result["call_dir"] / "prompt.md").read_text()
            prefix = "FRONTIER_CONTEXT_JSON:"
            context_line = next(
                line for line in prompt_text.splitlines() if line.startswith(prefix)
            )
            provider_context = json.loads(context_line.removeprefix(prefix).strip())
            self.assertEqual(
                set(provider_context),
                {"schema_version", "theorem_text", "mathematical_payload", "probe_ids"},
            )
            for hidden in (
                "ticket_id",
                "run_id",
                "evaluation_id",
                "evaluator_index",
                "node_id",
                "node_artifact_sha256",
                "mathematical_payload_sha256",
                "theorem_sha256",
                "schema_sha256",
                "prompt_sha256",
                "parent_artifact_sha256",
                "allowed_parent_artifacts",
            ):
                self.assertNotIn(hidden, provider_context)

    def test_child_expert_context_with_one_parent_artifact_passes_adapter(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_frontier_cli.py"
            write_fake_frontier_cli(cli, "child_expert")
            run_dir = write_frontier_run(root, cli)
            prompt = "Extend the selected parent direction."
            schema = run_dir / "schemas/expert_result.schema.json"
            context = child_expert_context(prompt, schema)
            ticket = frontier_ticket(
                prompt,
                schema,
                context=context,
                ticket_id="expert-g1-d0",
                node_id="node-g1-d0",
                parent_node_id="node-g0-function-theory",
                generation=1,
                direction_id="dir-child-obligation",
                parent_artifact_sha256="1" * 64,
            )

            result = frontier_provider.run_frontier_call(
                run_dir,
                ticket=ticket,
                call_id="expert-g1-d0",
                role="expert",
                prompt=prompt,
                context=context,
                schema_path=schema,
                accounting_path=run_dir / "attempt_ledger.jsonl",
            )

            self.assertEqual(result["receipt"]["status"], "completed")
            self.assertEqual(
                result["validated_output"]["parent_node_artifact_sha256"], "1" * 64
            )
            self.assertEqual(result["validated_output"]["generation"], 1)

    def test_context_digest_drift_fails_before_call_directory_creation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_frontier_cli.py"
            write_fake_frontier_cli(cli)
            run_dir = write_frontier_run(root, cli)
            prompt = "Attempt the function-theory route."
            schema = run_dir / "schemas/expert_result.schema.json"
            context = strict_expert_context(prompt, schema)
            ticket = frontier_ticket(prompt, schema, context=context)
            drifted = dict(context)
            drifted["extra_context"] = "leakier material"

            with self.assertRaisesRegex(protocol.ValidationError, "context_sha256"):
                frontier_provider.run_frontier_call(
                    run_dir,
                    ticket=ticket,
                    call_id="expert-g0-function-theory",
                    role="expert",
                    prompt=prompt,
                    context=drifted,
                    schema_path=schema,
                    accounting_path=run_dir / "attempt_ledger.jsonl",
                )

            self.assertFalse((run_dir / "calls/expert-g0-function-theory").exists())

    def test_two_field_context_is_rejected_before_call_directory_creation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_frontier_cli.py"
            write_fake_frontier_cli(cli)
            run_dir = write_frontier_run(root, cli)
            prompt = "Attempt the function-theory route."
            schema = run_dir / "schemas/expert_result.schema.json"
            context = strict_expert_context(prompt, schema)
            two_field_context = {
                "ticket_id": "expert-g0-function-theory",
                "delegation_allowed": False,
            }
            ticket = frontier_ticket(prompt, schema, context=two_field_context)

            with self.assertRaisesRegex(protocol.ValidationError, "expert context"):
                frontier_provider.run_frontier_call(
                    run_dir,
                    ticket=ticket,
                    call_id="expert-g0-function-theory",
                    role="expert",
                    prompt=prompt,
                    context=two_field_context,
                    schema_path=schema,
                    accounting_path=run_dir / "attempt_ledger.jsonl",
                )

            self.assertFalse((run_dir / "calls/expert-g0-function-theory").exists())

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
                    context = strict_expert_context(prompt, schema)
                    ticket = frontier_ticket(prompt, schema, context=context)
                    resource_probe = run_dir / "resource_probe.txt"

                    result = frontier_provider.run_frontier_call(
                        run_dir,
                        ticket=ticket,
                        call_id="expert-g0-function-theory",
                        role="expert",
                        prompt=prompt,
                        context=context,
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

    def test_valid_shaped_output_for_another_ticket_is_malformed_and_not_completed_on_disk(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            cli = root / "fake_frontier_cli.py"
            write_fake_frontier_cli(cli, "wrong_ticket")
            run_dir = write_frontier_run(root, cli)
            prompt = "Attempt the function-theory route."
            schema = run_dir / "schemas/expert_result.schema.json"
            context = strict_expert_context(prompt, schema)
            ticket = frontier_ticket(prompt, schema, context=context)

            result = frontier_provider.run_frontier_call(
                run_dir,
                ticket=ticket,
                call_id="expert-g0-function-theory",
                role="expert",
                prompt=prompt,
                context=context,
                schema_path=schema,
                accounting_path=run_dir / "attempt_ledger.jsonl",
            )

            self.assertEqual(result["receipt"]["status"], "malformed")
            receipt = json.loads((result["call_dir"] / "receipt.json").read_text())
            self.assertEqual(receipt["status"], "malformed")
            self.assertNotEqual(receipt["status"], "completed")
            ledger = [
                json.loads(line)
                for line in (run_dir / "attempt_ledger.jsonl").read_text().splitlines()
            ]
            self.assertEqual(ledger[0]["status"], "malformed")

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
