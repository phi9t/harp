from __future__ import annotations

import hashlib
import json
import os
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
    (run_dir / "schemas/node_evaluation_payload.schema.json").write_text(
        "{}\n", encoding="utf-8"
    )

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
        score_complete_roles: set[str] | None = None,
        schema_valid_evaluator_findings: bool = False,
    ) -> None:
        self.root_status_by_role = root_status_by_role or {}
        self.evaluator_failures = evaluator_failures or set()
        self.score_complete_roles = score_complete_roles or {"function_theory"}
        self.schema_valid_evaluator_findings = schema_valid_evaluator_findings
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
        score_complete = role in self.score_complete_roles and int(context["generation"]) == 0
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
        del run_dir
        self.evaluator_contexts.append(context)
        ticket_id = str(ticket["ticket_id"])
        evaluator_index = 1 if ticket_id.endswith("-e1") else 2
        key = (str(ticket["node_id"]), evaluator_index)
        if key in self.evaluator_failures:
            return expert_runner.ProviderAttempt(
                terminal_status="timed_out",
                provider_call={"kind": "fake-evaluator"},
                receipt={"status": "timed_out", "ticket_id": ticket_id},
                validated_output=None,
                reason="fake evaluator timeout",
            )
        node_id = str(ticket["node_id"])
        role = str(node_id).removeprefix("node-g0-").replace("-", "_")
        pass_count = 10 if role in self.score_complete_roles else 6
        findings = [
            {
                "finding_id": f"note-e{evaluator_index}",
                "severity": "minor" if pass_count == 10 else "major",
                "statement": f"Evaluator {evaluator_index} note in {node_id}.",
                "locator": "candidate.tex#L1",
                "recommended_role": "approximation_audit",
            }
        ]
        if self.schema_valid_evaluator_findings:
            findings = [
                {
                    "finding_id": f"note-e{evaluator_index}",
                    "probe_id": prepare_frontier.PROOF_PROGRESS_PROBE_IDS[
                        min(pass_count, len(prepare_frontier.PROOF_PROGRESS_PROBE_IDS)) - 1
                    ],
                    "severity": "minor" if pass_count == 10 else "major",
                    "locator": "candidate.tex#L1",
                    "statement": f"Evaluator {evaluator_index} note in {node_id}.",
                    "test": f"Check probe evidence for evaluator {evaluator_index}.",
                }
            ]
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
            "findings": findings,
        }
        return expert_runner.ProviderAttempt(
            terminal_status="completed",
            provider_call={"kind": "fake-evaluator"},
            receipt={"status": "completed", "ticket_id": ticket_id},
            validated_output=payload,
            reason=None,
        )


class DrawBarrierProvider(FakeProvider):
    def __init__(self, run_dir: Path) -> None:
        super().__init__()
        self.run_dir = run_dir

    def run_expert(
        self,
        *,
        run_dir: Path,
        ticket: dict[str, object],
        context: dict[str, object],
    ) -> expert_runner.ProviderAttempt:
        if int(context["generation"]) > 0:
            generation = int(context["generation"])
            draw_one = self.run_dir / "tickets" / f"select-g{generation}-d1" / "ticket_events.jsonl"
            states = [
                json.loads(line)["to_state"]
                for line in draw_one.read_text(encoding="utf-8").splitlines()
            ] if draw_one.exists() else []
            if states[-2:] != ["completed", "accepted"]:
                raise AssertionError("child expert ran before draw 1 terminalized")
        return super().run_expert(run_dir=run_dir, ticket=ticket, context=context)


class FailOnceAfterFirstRootProvider(FakeProvider):
    def __init__(self) -> None:
        super().__init__()
        self.failed = False

    def run_expert(
        self,
        *,
        run_dir: Path,
        ticket: dict[str, object],
        context: dict[str, object],
    ) -> expert_runner.ProviderAttempt:
        if (
            int(context["generation"]) == 0
            and context["expert_role"] == "operator_dilation"
            and not self.failed
        ):
            self.failed = True
            raise RuntimeError("simulated provider interruption")
        return super().run_expert(run_dir=run_dir, ticket=ticket, context=context)


class ExpertRunnerTests(unittest.TestCase):
    def test_evaluator_provider_receives_full_ticket_bound_context(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FakeProvider()
            expert_runner.run_phase(run_dir, "roots", provider=provider)

            expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)

            self.assertGreater(len(provider.evaluator_contexts), 0)
            for context in provider.evaluator_contexts:
                self.assertEqual(
                    set(context),
                    {
                        "schema_version",
                        "run_id",
                        "evaluation_id",
                        "evaluator_index",
                        "ticket_id",
                        "node_id",
                        "node_artifact_sha256",
                        "mathematical_payload_sha256",
                        "theorem_text",
                        "theorem_sha256",
                        "mathematical_payload",
                        "probe_ids",
                        "forbidden_sources",
                        "forbidden_tools",
                        "allowed_tools",
                        "delegation_allowed",
                        "completion_criteria",
                        "result_schema",
                        "schema_sha256",
                        "prompt_sha256",
                        "parent_artifact_sha256",
                        "allowed_parent_artifacts",
                    },
                )
                ticket = json.loads(
                    (
                        run_dir
                        / "tickets"
                        / str(context["ticket_id"])
                        / "ticket.json"
                    ).read_text(encoding="utf-8")
                )
                self.assertEqual(ticket["context_sha256"], digest_json(context))

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
            for full_context in provider.evaluator_contexts:
                call_dir = run_dir / "evaluator_calls" / str(full_context["ticket_id"])
                context = json.loads(
                    (call_dir / "provider_context.json").read_text(encoding="utf-8")
                )
                for hidden in (
                    "parent",
                    "parent_node_id",
                    "generation",
                    "expert_role",
                    "score",
                    "run_id",
                    "ticket_id",
                    "node_id",
                    "node_artifact_sha256",
                    "mathematical_payload_sha256",
                    "schema_sha256",
                    "prompt_sha256",
                    "parent_artifact_sha256",
                    "allowed_parent_artifacts",
                ):
                    self.assertNotIn(hidden, context)
                self.assertEqual(
                    set(context),
                    {"schema_version", "theorem_text", "mathematical_payload", "probe_ids"},
                )

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

    def test_node_evaluation_binds_actual_persisted_terminal_ticket_event_digest(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FakeProvider()
            expert_runner.run_phase(run_dir, "roots", provider=provider)
            expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)

            evaluation_path = (
                run_dir
                / "node_evaluations"
                / "node-g0-function-theory"
                / "evaluator-1"
                / "evaluation.json"
            )
            evaluation = json.loads(evaluation_path.read_text())
            event_path = (
                run_dir
                / "tickets"
                / evaluation["ticket_id"]
                / "ticket_events.jsonl"
            )
            completed_events = [
                json.loads(line)
                for line in event_path.read_text().splitlines()
                if json.loads(line)["to_state"] == "completed"
            ]
            self.assertEqual(len(completed_events), 1)
            self.assertEqual(
                evaluation["terminal_ticket_event_sha256"],
                digest_json(completed_events[0]),
            )

    def test_schema_valid_evaluator_finding_is_terminalized_and_namespaced(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FakeProvider(schema_valid_evaluator_findings=True)
            expert_runner.run_phase(run_dir, "roots", provider=provider)

            expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)

            event_path = (
                run_dir
                / "tickets"
                / "evaluate-node-g0-operator-dilation-e1"
                / "ticket_events.jsonl"
            )
            states = [
                json.loads(line)["to_state"]
                for line in event_path.read_text(encoding="utf-8").splitlines()
            ]
            self.assertEqual(states[-2:], ["completed", "accepted"])
            reconciliation = json.loads(
                (
                    run_dir
                    / "node_evaluations"
                    / "node-g0-operator-dilation"
                    / "reconciliation.json"
                ).read_text(encoding="utf-8")
            )
            finding = reconciliation["evaluator_findings"][0]
            self.assertEqual(finding["finding_id"], "e1:note-e1")
            self.assertEqual(finding["source_finding_id"], "note-e1")
            self.assertEqual(finding["source_evaluator_index"], 1)
            self.assertEqual(finding["recommended_role"], "approximation_audit")
            self.assertIn("Check probe evidence", finding["statement"])

    def test_generation_barrier_terminalizes_both_draws_before_child_execution(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = DrawBarrierProvider(run_dir)
            expert_runner.run_phase(run_dir, "roots", provider=provider)
            expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)

            result = expert_runner.run_phase(run_dir, "generations", provider=provider)

            self.assertEqual(result["selection_count"], 6)

    def test_two_score_complete_candidates_are_retained_without_stopping_search(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FakeProvider(score_complete_roles={"function_theory", "operator_dilation"})
            expert_runner.run_phase(run_dir, "roots", provider=provider)

            evaluated = expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)

            self.assertEqual(evaluated["candidate_projection_count"], 2)
            index = json.loads((run_dir / "candidate_projections" / "index.json").read_text())
            self.assertEqual(
                sorted(item["source_node_id"] for item in index["projections"]),
                ["node-g0-function-theory", "node-g0-operator-dilation"],
            )
            expert_runner.run_phase(run_dir, "generations", provider=provider)
            selections = [
                json.loads(line)
                for line in (run_dir / "selection_events.jsonl").read_text().splitlines()
            ]
            self.assertGreater(len(selections), 0)
            self.assertNotIn(
                "node-g0-function-theory",
                {item["selected_node_id"] for item in selections},
            )
            self.assertNotIn(
                "node-g0-operator-dilation",
                {item["selected_node_id"] for item in selections},
            )

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

    def test_phase_boundaries_reject_marker_only_prior_phase_and_partial_later_artifacts(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            write_json(
                run_dir / "frontier_phase_state.json",
                {
                    "roots": {
                        "completed_at_utc": "2026-08-15T12:00:00Z",
                        "result_sha256": "a" * 64,
                    }
                },
            )

            with self.assertRaisesRegex(protocol.ValidationError, "roots evidence"):
                expert_runner.run_phase(run_dir, "evaluate-roots", provider=FakeProvider())

        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FakeProvider()
            expert_runner.run_phase(run_dir, "roots", provider=provider)
            write_json(
                run_dir / "candidate_projections" / "finalization.json",
                {
                    "schema_version": "partial-later-phase/v1",
                    "run_id": run_dir.name,
                },
            )

            with self.assertRaisesRegex(protocol.ValidationError, "later phase"):
                expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)

    def test_reconciliation_rejects_runtime_ticket_without_terminal_evidence(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FakeProvider()
            expert_runner.run_phase(run_dir, "roots", provider=provider)
            context = {
                "schema_version": "crouzeix-operator-context/v1",
                "run_id": run_dir.name,
                "reason": "test dangling ticket",
            }
            ticket = expert_runner.runtime_ticket(
                run_dir=run_dir,
                ticket_id="operator-intervention-test",
                task_kind="operator_intervention",
                node_id=None,
                parent_node_id=None,
                generation=0,
                direction_id=None,
                role="operator",
                owner_type="operator",
                context=context,
                prompt_name="expert.md",
                schema_name="expert_result.schema.json",
                parent_artifact_sha256=None,
                dependency_ticket_ids=[],
                objective="Record a manual intervention.",
                expected_deliverable="Terminal intervention record.",
            )
            tickets.publish_ticket(run_dir / "tickets", ticket)

            with self.assertRaisesRegex(protocol.ValidationError, "terminal"):
                expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)

    def test_roots_phase_resumes_after_partial_failure_without_duplicate_evidence(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FailOnceAfterFirstRootProvider()
            with self.assertRaisesRegex(RuntimeError, "simulated provider interruption"):
                expert_runner.run_phase(run_dir, "roots", provider=provider)
            first_attempts = [
                json.loads(line)
                for line in (run_dir / "attempt_ledger.jsonl").read_text().splitlines()
            ]
            self.assertEqual(
                [item["attempt_id"] for item in first_attempts],
                ["expert-g0-function-theory"],
            )

            result = expert_runner.run_phase(run_dir, "roots", provider=provider)

            self.assertEqual(result["root_attempt_count"], 5)
            attempts = [
                json.loads(line)
                for line in (run_dir / "attempt_ledger.jsonl").read_text().splitlines()
            ]
            self.assertEqual(len(attempts), 5)
            self.assertEqual(
                len({item["attempt_id"] for item in attempts}),
                5,
            )

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

    def test_phase_state_rejects_dangling_symlink_target(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            run_dir = make_run(root)
            escaped = root / "escaped-phase-state.json"
            os.symlink(escaped, run_dir / "frontier_phase_state.json")

            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                expert_runner.run_phase(run_dir, "roots", provider=FakeProvider())

            self.assertFalse(escaped.exists())

    def test_run_receipt_rejects_dangling_symlink_target(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            run_dir = make_run(root)
            escaped = root / "escaped-run-receipt.json"
            os.symlink(escaped, run_dir / "run_receipt.json")

            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                expert_runner.run_phase(run_dir, "roots", provider=FakeProvider())

            self.assertFalse(escaped.exists())

    def test_candidate_projection_rejects_dangling_symlink_target(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            root = Path(directory)
            run_dir = make_run(root)
            candidate_bytes = b"Candidate proof text for expert-g0-function-theory."
            candidate_sha256 = digest_bytes(candidate_bytes)
            escaped = root / "escaped-candidate.tex"
            os.symlink(
                escaped,
                run_dir / "candidate_projections" / f"{candidate_sha256}.tex",
            )
            provider = FakeProvider()

            expert_runner.run_phase(run_dir, "roots", provider=provider)
            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)

            self.assertFalse(escaped.exists())

    def test_reconciliation_rejects_orphan_evaluator_evidence(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FakeProvider()
            expert_runner.run_phase(run_dir, "roots", provider=provider)
            expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)
            expert_runner.run_phase(run_dir, "generations", provider=provider)
            expert_runner.run_phase(run_dir, "finalize", provider=provider)
            ticket_dir = run_dir / "tickets" / "evaluate-node-g0-function-theory-e1"
            for path in sorted(ticket_dir.rglob("*"), reverse=True):
                path.unlink()
            ticket_dir.rmdir()

            with self.assertRaisesRegex(protocol.ValidationError, "orphan evaluator"):
                expert_runner.run_phase(run_dir, "check", provider=provider)

    def test_reconciliation_rejects_orphan_candidate_freeze_evidence(self) -> None:
        with tempfile.TemporaryDirectory(dir="/private/tmp") as directory:
            run_dir = make_run(Path(directory))
            provider = FakeProvider()
            expert_runner.run_phase(run_dir, "roots", provider=provider)
            expert_runner.run_phase(run_dir, "evaluate-roots", provider=provider)
            expert_runner.run_phase(run_dir, "generations", provider=provider)
            expert_runner.run_phase(run_dir, "finalize", provider=provider)
            ticket_dir = run_dir / "tickets" / "candidate-freeze-node-g0-function-theory"
            for path in sorted(ticket_dir.rglob("*"), reverse=True):
                path.unlink()
            ticket_dir.rmdir()

            with self.assertRaisesRegex(protocol.ValidationError, "orphan candidate"):
                expert_runner.run_phase(run_dir, "check", provider=provider)


if __name__ == "__main__":
    unittest.main()
