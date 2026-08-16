from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import expert_runner
import frontier
import prepare_frontier
import protocol
import tickets


EXCLUDED_SOURCES = [
    "search",
    "network",
    "mcp",
    "shell",
    "read",
    "delegation",
    "public proof manuscripts",
]
FORBIDDEN_TOOLS = [
    "Read",
    "Glob",
    "Grep",
    "Bash",
    "Edit",
    "spawn_agent",
    "WebSearch",
    "MCP",
    "Shell",
]
ROLES = prepare_frontier.EXPERT_ROLES
THEOREM_TEXT = "For every square matrix A, prove the Crouzeix bound."


def digest_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def digest_json(value: object) -> str:
    return digest_bytes(
        json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
    )


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def make_run(root: Path) -> Path:
    run_dir = root / "expert-frontier-001"
    for relative in (
        "inputs",
        "prompts",
        "schemas",
        "contexts",
        "tickets",
        "attempts",
        "calls",
        "admissions",
        "mathematical_nodes",
        "node_evaluations",
        "archive_entries",
        "candidate_projections",
        "review",
    ):
        (run_dir / relative).mkdir(parents=True, exist_ok=True)

    (run_dir / "inputs/theorem.txt").write_text(THEOREM_TEXT, encoding="utf-8")
    expert_prompt = "Return a strict expert result."
    evaluator_prompt = "Return a strict evaluator result."
    (run_dir / "prompts/expert.md").write_text(expert_prompt, encoding="utf-8")
    (run_dir / "prompts/proof_progress_evaluator.md").write_text(
        evaluator_prompt, encoding="utf-8"
    )
    (run_dir / "schemas/expert_result.schema.json").write_text("{}\n", encoding="utf-8")
    (run_dir / "schemas/node_evaluation.schema.json").write_text("{}\n", encoding="utf-8")

    frontier_config = {
        "selection_seed": 20260814,
        "expert_roles": list(ROLES),
        "proof_progress_probe_ids": list(prepare_frontier.PROOF_PROGRESS_PROBE_IDS),
        "root_expert_count": 5,
        "child_generations": 3,
        "draws_per_generation": 2,
        "admitted_mathematical_node_budget": 11,
        "total_call_budget": 33,
        "per_call_preflight": {"memory_mib": 4096},
    }
    spec = {
        "schema_version": "crouzeix-run-spec/v1",
        "run_id": "expert-frontier-001",
        "arm": "expert_frontier",
        "leakage": "L1",
        "model": "gpt-5.6-sol",
        "cli": {
            "path": str(run_dir / "fake-traecli"),
            "version": "traecli fake",
            "sha256": "1" * 64,
        },
        "historical_prompt": {
            "source_bytes": 4106,
            "source_sha256": protocol.HISTORICAL_PROMPT_SHA256,
            "execution_bytes": 4000,
            "execution_sha256": "2" * 64,
            "normalization": "test fixture",
        },
        "sandbox": "workspace-write",
        "approval_policy": "never",
        "allowed_tools": ["Write"],
        "network_access": False,
        "timeout_seconds": 3600,
        "max_calls": 33,
        "token_accounting": {"boundary": "all fake frontier calls"},
        "generation_visible_files": ["inputs/theorem.txt"],
        "generation_excluded_classes": ["public proof manuscripts"],
        "frontier": frontier_config,
        "digests": {"config/frontier": digest_json(frontier_config)},
        "created_at_utc": "2026-08-15T12:00:00Z",
    }
    protocol.validate_run_spec(spec)
    write_json(run_dir / "run_spec.json", spec)

    for role in ROLES:
        ticket_id = prepare_frontier.root_ticket_id(role)
        node_id = prepare_frontier.root_node_id(role)
        direction = prepare_frontier.root_direction(role)
        context = expert_runner.build_expert_runtime_context(
            run_dir=run_dir,
            attempt_id=ticket_id,
            ticket_id=ticket_id,
            proposed_node_id=node_id,
            parent_node=None,
            generation=0,
            expert_role=role,
            selected_direction=direction,
        )
        write_json(run_dir / "contexts" / f"{ticket_id}.json", context)
        ticket = expert_runner.runtime_ticket(
            run_dir=run_dir,
            ticket_id=ticket_id,
            task_kind="expert",
            node_id=node_id,
            parent_node_id=None,
            generation=0,
            direction_id=str(direction["direction_id"]),
            role=role,
            owner_type="expert",
            context=context,
            prompt_name="expert.md",
            schema_name="expert_result.schema.json",
            parent_artifact_sha256=None,
            dependency_ticket_ids=[],
            objective=f"Run root {role}.",
            expected_deliverable="Strict expert result or terminal failure.",
        )
        tickets.publish_ticket(run_dir / "tickets", ticket)
    return run_dir


def payload_for(label: str, *, candidate: bool = False, score_complete: bool = False) -> dict[str, object]:
    endpoint = {
        "kind": "candidate_proof" if candidate else "blocker",
        "text": f"Candidate proof text for {label}." if candidate else f"Blocker for {label}.",
    }
    obligations = []
    if not score_complete:
        obligations.append(
            {
                "obligation_id": "global-extension",
                "statement": f"Close the global extension for {label}.",
                "strength": "major",
            }
        )
    return {
        "proof_family": f"family {label}",
        "mechanism": f"mechanism {label}",
        "proved_statements": [
            {
                "statement_id": "stmt-main",
                "statement": f"Intermediate statement for {label}.",
                "justification": "Fake provider fixture.",
                "depends_on_statement_ids": [],
            }
        ],
        "unproved_obligations": obligations,
        "circularity_risks": [],
        "proposed_directions": [
            {
                "direction_id": "audit-next",
                "statement": f"Audit next step for {label}.",
                "strength": "major",
                "recommended_role": "approximation_audit",
            }
        ],
        "endpoint": endpoint,
        "confidence_basis": "fake deterministic fixture",
    }


class FakeProvider:
    def __init__(
        self,
        *,
        root_status_by_role: dict[str, str] | None = None,
        evaluator_failures: set[tuple[str, int]] | None = None,
    ) -> None:
        self.root_status_by_role = root_status_by_role or {}
        self.evaluator_failures = evaluator_failures or set()
        self.expert_calls: list[dict[str, object]] = []
        self.evaluator_contexts: list[dict[str, object]] = []

    def run_expert(
        self,
        *,
        run_dir: Path,
        ticket: dict[str, object],
        context: dict[str, object],
    ) -> expert_runner.ProviderAttempt:
        del run_dir, ticket
        self.expert_calls.append(context)
        role = str(context["expert_role"])
        status = self.root_status_by_role.get(role, "completed")
        if int(context["generation"]) > 0:
            status = "completed"
        if status != "completed":
            return expert_runner.ProviderAttempt(
                terminal_status=status,
                provider_call={"kind": "fake", "role": role},
                receipt={"status": status, "ticket_id": context["ticket_id"]},
                validated_output=None,
                reason=f"fake {status}",
            )
        label = str(context["attempt_id"])
        score_complete = role == "function_theory" and int(context["generation"]) == 0
        output = {
            "schema_version": "crouzeix-expert-result/v1",
            "run_id": context["run_id"],
            "attempt_id": context["attempt_id"],
            "ticket_id": context["ticket_id"],
            "proposed_node_id": context["proposed_node_id"],
            "parent_node_id": None
            if context["parent"] is None
            else context["parent"]["node_id"],
            "parent_node_artifact_sha256": None
            if context["parent"] is None
            else context["parent"]["node_artifact_sha256"],
            "generation": context["generation"],
            "expert_role": role,
            "selected_direction_id": context["selected_direction"]["direction_id"],
            "mathematical_payload": payload_for(
                label,
                candidate=score_complete or int(context["generation"]) > 0,
                score_complete=score_complete,
            ),
        }
        return expert_runner.ProviderAttempt(
            terminal_status="completed",
            provider_call={"kind": "fake", "role": role},
            receipt={"status": "completed", "ticket_id": context["ticket_id"]},
            validated_output=output,
            reason=None,
        )

    def run_evaluator(
        self,
        *,
        run_dir: Path,
        ticket: dict[str, object],
        context: dict[str, object],
    ) -> expert_runner.ProviderAttempt:
        del run_dir, ticket
        self.evaluator_contexts.append(context)
        key = (str(context["node_id"]), int(context["evaluator_index"]))
        if key in self.evaluator_failures:
            return expert_runner.ProviderAttempt(
                terminal_status="timed_out",
                provider_call={"kind": "fake-evaluator"},
                receipt={"status": "timed_out", "ticket_id": context["ticket_id"]},
                validated_output=None,
                reason="fake evaluator timeout",
            )
        node_id = str(context["node_id"])
        pass_count = 10 if node_id == "node-g0-function-theory" else 6
        payload = {
            "schema_version": "crouzeix-node-evaluation-payload/v1",
            "probes": [
                {
                    "probe_id": probe_id,
                    "status": "pass" if index < pass_count else "fail",
                    "rationale": "fake evaluation",
                }
                for index, probe_id in enumerate(prepare_frontier.PROOF_PROGRESS_PROBE_IDS)
            ],
            "findings": [
                {
                    "finding_id": f"note-e{context['evaluator_index']}",
                    "severity": "minor" if pass_count == 10 else "major",
                    "statement": f"Evaluator {context['evaluator_index']} note in {node_id}.",
                    "locator": "candidate.tex#L1",
                    "recommended_role": "approximation_audit",
                }
            ],
        }
        return expert_runner.ProviderAttempt(
            terminal_status="completed",
            provider_call={"kind": "fake-evaluator"},
            receipt={"status": "completed", "ticket_id": context["ticket_id"]},
            validated_output=payload,
            reason=None,
        )


class ExpertRunnerTests(unittest.TestCase):
    def test_fake_end_to_end_run_reconciles_tickets_ledgers_archive_and_candidate(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FakeProvider(
                root_status_by_role={"matrix_extremal": "malformed"},
                evaluator_failures={("node-g0-operator-dilation", 2)},
            )

            roots = expert_runner.run_phase(run_dir, "roots", provider=provider)
            self.assertEqual(roots["root_attempt_count"], 5)
            self.assertEqual(roots["root_completed"], 4)
            self.assertEqual(
                [call["expert_role"] for call in provider.expert_calls[:5]],
                list(ROLES),
            )

            evaluated = expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)
            self.assertEqual(evaluated["archive_entry_count"], 4)
            self.assertEqual(evaluated["candidate_projection_count"], 1)
            self.assertEqual(len(provider.evaluator_contexts), 8)
            for context in provider.evaluator_contexts:
                for hidden in ("parent", "parent_node_id", "generation", "expert_role", "score"):
                    self.assertNotIn(hidden, context)
                self.assertEqual(context["allowed_tools"], ["Write"])
                self.assertFalse(context["delegation_allowed"])

            generations = expert_runner.run_phase(run_dir, "generations", provider=provider)
            self.assertEqual(generations["selection_count"], 6)
            self.assertGreaterEqual(generations["archive_entry_count"], 10)
            selections = [
                json.loads(line)
                for line in (run_dir / "selection_events.jsonl").read_text().splitlines()
            ]
            self.assertEqual(
                [(item["generation"], item["draw_index"]) for item in selections],
                [(1, 0), (1, 1), (2, 0), (2, 1), (3, 0), (3, 1)],
            )

            finalized = expert_runner.run_phase(run_dir, "finalize", provider=provider)
            self.assertEqual(finalized["candidate_projection_count"], 1)
            self.assertIsNotNone(finalized["strongest_archive_entry_sha256"])
            receipt = json.loads((run_dir / "run_receipt.json").read_text())
            self.assertEqual(receipt["selection_count"], 6)
            self.assertEqual(receipt["terminal_attempt_states"]["malformed"], 1)
            self.assertLessEqual(receipt["provider_call_count"], 33)
            self.assertEqual(receipt["ticket_count"], len(list((run_dir / "tickets").iterdir())))

            check = expert_runner.run_phase(run_dir, "check", provider=provider)
            self.assertEqual(check["phase"], "check")
            self.assertEqual(check["run_receipt_sha256"], receipt["run_receipt_sha256"])

            ticket_ids = sorted(path.name for path in (run_dir / "tickets").iterdir())
            self.assertIn("select-g1-d0", ticket_ids)
            self.assertIn("candidate-freeze-node-g0-function-theory", ticket_ids)
            for ticket_id in ticket_ids:
                event_path = run_dir / "tickets" / ticket_id / "ticket_events.jsonl"
                self.assertTrue(event_path.exists(), ticket_id)
                lines = event_path.read_text().splitlines()
                self.assertGreaterEqual(len(lines), 1, ticket_id)

    def test_phase_boundaries_reject_skips_and_repeats(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FakeProvider()
            with self.assertRaisesRegex(protocol.ValidationError, "roots"):
                expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)
            expert_runner.run_phase(run_dir, "roots", provider=provider)
            with self.assertRaisesRegex(protocol.ValidationError, "repeat"):
                expert_runner.run_phase(run_dir, "roots", provider=provider)
            with self.assertRaisesRegex(protocol.ValidationError, "evaluate-roots"):
                expert_runner.run_phase(run_dir, "generations", provider=provider)

    def test_budget_failure_closes_with_receipt(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FakeProvider()
            limits = expert_runner.RunnerLimits(max_provider_calls=5)
            expert_runner.run_phase(run_dir, "roots", provider=provider, limits=limits)
            result = expert_runner.run_phase(
                run_dir,
                "evaluate-roots",
                provider=provider,
                limits=limits,
            )

            self.assertTrue(result["budget_exhausted"])
            receipt = json.loads((run_dir / "run_receipt.json").read_text())
            self.assertEqual(receipt["schema_version"], "crouzeix-run-receipt/v1")
            self.assertLessEqual(receipt["provider_call_count"], 5)


if __name__ == "__main__":
    unittest.main()
