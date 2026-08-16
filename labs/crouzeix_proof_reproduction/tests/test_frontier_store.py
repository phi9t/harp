from __future__ import annotations

import hashlib
import json
import os
import sys
import tempfile
import unittest
from unittest import mock
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import expert_contracts
import frontier
import frontier_store
import protocol


DIGEST_A = "a" * 64
DIGEST_B = "b" * 64
DIGEST_C = "c" * 64
DIGEST_D = "d" * 64
DIGEST_E = "e" * 64
DIGEST_1 = "1" * 64
DIGEST_2 = "2" * 64
DIGEST_3 = "3" * 64
DIGEST_4 = "4" * 64
DIGEST_5 = "5" * 64
DIGEST_6 = "6" * 64


def digest_json(value: object) -> str:
    data = json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")
    return hashlib.sha256(data).hexdigest()


def selected_direction(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "direction_id": "root-function-theory",
        "kind": "root_task",
        "statement": "Develop an independent route to the theorem.",
        "strength": "root",
        "recommended_role": "function_theory",
        "source_parent_node_id": None,
        "source_node_artifact_sha256": None,
        "source_reconciliation_sha256": None,
    }
    value.update(overrides)
    return value


def payload(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "proof_family": "functional calculus route",
        "mechanism": "Reduce the target to a local estimate.",
        "proved_statements": [
            {
                "statement_id": "stmt-local-bound",
                "statement": "The local bound follows from normalization.",
                "justification": "Direct estimate in the proposed normalization.",
                "depends_on_statement_ids": [],
            }
        ],
        "unproved_obligations": [
            {
                "obligation_id": "obl-global-extension",
                "statement": "Extend the local estimate globally.",
                "strength": "major",
            }
        ],
        "circularity_risks": [],
        "proposed_directions": [
            {
                "direction_id": "dir-check-extension",
                "statement": "Audit the global extension step.",
                "strength": "major",
                "recommended_role": "approximation_audit",
            }
        ],
        "endpoint": {
            "kind": "candidate_proof",
            "text": "Candidate proof text with explicit remaining checks.",
        },
        "confidence_basis": "One proved statement and one explicit obligation.",
    }
    value.update(overrides)
    return value


def expert_result(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
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
        "mathematical_payload": payload(),
    }
    value.update(overrides)
    return expert_contracts.validate_expert_result(value)


def attempt(status: str = "completed", **overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "run_id": "expert-frontier-001",
        "attempt_id": "attempt-g0-function-theory",
        "ticket_id": "expert-g0-function-theory",
        "proposed_node_id": "node-g0-function-theory",
        "terminal_status": status,
    }
    value.update(overrides)
    return value


def provenance(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "theorem_sha256": DIGEST_A,
        "leakage_tier": "L1",
        "context_sha256": DIGEST_B,
        "call_receipt_sha256": DIGEST_C,
        "leakage_audit_sha256": DIGEST_D,
        "selected_direction": selected_direction(),
    }
    value.update(overrides)
    return value


def accepted_records() -> tuple[dict[str, object], dict[str, object], dict[str, object]]:
    result = expert_result()
    decision = expert_contracts.decide_admission(
        attempt(),
        result,
        provenance(),
        outcome="accepted",
    )
    node = expert_contracts.build_mathematical_node(decision, result, provenance())
    return result, decision, node


def node_evaluation(
    node: dict[str, object],
    *,
    evaluator_index: int,
    status: str = "pass",
) -> dict[str, object]:
    evaluation_payload = {
        "probes": [
            {"probe_id": probe_id, "status": status} for probe_id in frontier.PROBE_IDS
        ],
        "findings": []
        if evaluator_index == 1
        else [
            {
                "finding_id": "independent-note",
                "severity": "minor",
                "statement": "Second evaluator records an independent note.",
                "locator": "candidate.tex#L1",
                "recommended_role": "approximation_audit",
            }
        ],
    }
    evaluation = {
        "schema_version": "crouzeix-node-evaluation/v1",
        "evaluation_id": f"eval-{node['node_id']}-{evaluator_index}",
        "evaluator_index": evaluator_index,
        "ticket_id": f"evaluate-{node['node_id']}-e{evaluator_index}",
        "node_artifact_sha256": node["node_artifact_sha256"],
        "mathematical_payload_sha256": node["mathematical_payload_sha256"],
        "context_sha256": DIGEST_1 if evaluator_index == 1 else DIGEST_2,
        "source_kind": "provider_output",
        "call_receipt_sha256": DIGEST_3 if evaluator_index == 1 else DIGEST_4,
        "terminal_ticket_event_sha256": DIGEST_5 if evaluator_index == 1 else DIGEST_6,
        "evaluation_payload": evaluation_payload,
        "evaluation_payload_sha256": digest_json(evaluation_payload),
    }
    evaluation["node_evaluation_sha256"] = digest_json(evaluation)
    return evaluation


def attempt_ledger_record(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-attempt-ledger/v1",
        "sequence": 1,
        "run_id": "expert-frontier-001",
        "attempt_id": "attempt-g0-function-theory",
        "ticket_id": "expert-g0-function-theory",
        "proposed_node_id": "node-g0-function-theory",
        "terminal_status": "completed",
        "attempt_dir_sha256": DIGEST_A,
        "occurred_at_utc": "2026-08-15T00:00:00Z",
    }
    value.update(overrides)
    return value


def frontier_event(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-frontier-event/v1",
        "sequence": 1,
        "run_id": "expert-frontier-001",
        "event_kind": "attempt_terminal",
        "ticket_id": "expert-g0-function-theory",
        "artifact_sha256": DIGEST_A,
        "occurred_at_utc": "2026-08-15T00:00:01Z",
    }
    value.update(overrides)
    return value


def selection_event(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-selection-event/v1",
        "sequence": 1,
        "run_id": "expert-frontier-001",
        "event_kind": "selection_recorded",
        "ticket_id": "select-g1-d0",
        "artifact_sha256": DIGEST_B,
        "occurred_at_utc": "2026-08-15T00:01:00Z",
        "generation": 1,
        "draw_index": 0,
        "selected_node_id": "node-g0-function-theory",
        "archive_snapshot_sha256": DIGEST_C,
    }
    value.update(overrides)
    return value


class FrontierStoreTests(unittest.TestCase):
    def setUp(self) -> None:
        self.tempdir = tempfile.TemporaryDirectory()
        self.run_dir = Path(self.tempdir.name).resolve() / "run"
        self.store = frontier_store.open_frontier_store(self.run_dir)

    def tearDown(self) -> None:
        self.tempdir.cleanup()

    def test_open_rejects_symlinked_run_root_and_symlinked_ancestor(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            real = root / "real"
            real.mkdir()
            run_link = root / "run-link"
            os.symlink(real, run_link)

            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                frontier_store.open_frontier_store(run_link)

            real_parent = root / "real-parent"
            real_parent.mkdir()
            parent_link = root / "parent-link"
            os.symlink(real_parent, parent_link)
            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                frontier_store.open_frontier_store(parent_link / "run")

    def test_jsonl_ledgers_are_strict_bounded_append_only_and_reject_symlinks(self) -> None:
        first = self.store.append_attempt(
            attempt_ledger_record()
        )
        self.assertEqual(first["sequence"], 1)

        with self.assertRaisesRegex(protocol.ValidationError, "sequence"):
            self.store.append_attempt(
                {
                    **first,
                    "attempt_id": "attempt-g0-operator-dilation",
                    "sequence": 3,
                }
            )
        with self.assertRaisesRegex(protocol.ValidationError, "duplicate"):
            self.store.append_attempt({**first, "sequence": 2})
        with self.assertRaisesRegex(protocol.ValidationError, "fields"):
            self.store.append_frontier_event({"schema_version": "bad", "sequence": 1})

        selection_path = self.run_dir / "selection_events.jsonl"
        os.symlink(self.run_dir / "elsewhere.jsonl", selection_path)
        with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
            self.store.append_selection_event(selection_event())

    def test_existing_jsonl_rows_are_replayed_through_strict_ledger_validators(self) -> None:
        attempt_path = self.store.run_dir / "attempt_ledger.jsonl"
        attempt_path.write_text(json.dumps({**attempt_ledger_record(), "unknown": True}) + "\n")
        with self.assertRaisesRegex(protocol.ValidationError, "fields"):
            self.store.append_attempt(
                attempt_ledger_record(sequence=2, attempt_id="attempt-g0-operator-dilation")
            )
        self.assertEqual(len(attempt_path.read_text().splitlines()), 1)

        with tempfile.TemporaryDirectory() as directory:
            store = frontier_store.open_frontier_store(Path(directory).resolve() / "run")
            event_path = store.run_dir / "frontier_events.jsonl"
            event_path.write_text(json.dumps({**frontier_event(), "unknown": True}) + "\n")
            with self.assertRaisesRegex(protocol.ValidationError, "fields"):
                store.append_frontier_event(frontier_event(sequence=2, artifact_sha256=DIGEST_B))
            self.assertEqual(len(event_path.read_text().splitlines()), 1)

        with tempfile.TemporaryDirectory() as directory:
            store = frontier_store.open_frontier_store(Path(directory).resolve() / "run")
            selection_path = store.run_dir / "selection_events.jsonl"
            selection_path.write_text(json.dumps({**selection_event(), "unknown": True}) + "\n")
            with self.assertRaisesRegex(protocol.ValidationError, "fields"):
                store.append_selection_event(selection_event(sequence=2, draw_index=1))
            self.assertEqual(len(selection_path.read_text().splitlines()), 1)

    def test_attempt_directory_preserves_result_failure_call_and_receipts(self) -> None:
        attempt_dir = self.store.materialize_attempt(
            attempt_id="attempt-g0-function-theory",
            context={"context": "bytes"},
            provider_call={"argv": ["traecli", "exec"], "exit_code": 0},
            receipt={"ticket_id": "expert-g0-function-theory"},
            expert_result=expert_result(),
            terminal_failure=None,
        )

        self.assertEqual(
            sorted(path.name for path in attempt_dir.iterdir()),
            ["attempt.json", "context.json", "expert_result.json", "provider_call", "receipt.json"],
        )
        self.assertTrue((attempt_dir / "provider_call" / "call.json").is_file())
        with self.assertRaisesRegex(protocol.ValidationError, "already exists"):
            self.store.materialize_attempt(
                attempt_id="attempt-g0-function-theory",
                context={"context": "bytes"},
                provider_call={"argv": ["again"]},
                receipt={"ticket_id": "expert-g0-function-theory"},
                expert_result=expert_result(),
                terminal_failure=None,
            )

        blocked = self.store.materialize_attempt(
            attempt_id="attempt-g0-resource-block",
            context={"context": "bytes"},
            provider_call={"argv": ["traecli", "exec"], "exit_code": 2},
            receipt={"ticket_id": "expert-g0-resource-block"},
            expert_result=None,
            terminal_failure={
                "terminal_status": "blocked_resource",
                "reason": "preflight failed after call receipt",
            },
        )
        self.assertTrue((blocked / "terminal_failure.json").is_file())
        self.assertTrue((blocked / "provider_call" / "call.json").is_file())
        self.assertTrue((blocked / "receipt.json").is_file())

    def test_admission_node_evaluation_and_archive_records_are_create_only_and_separate(self) -> None:
        _, decision, node = accepted_records()
        self.store.record_admission(decision)
        admission_path = self.run_dir / "admissions" / f"{decision['attempt_id']}.json"
        self.assertNotIn("node_artifact_sha256", json.loads(admission_path.read_text()))
        with self.assertRaisesRegex(protocol.ValidationError, "already exists"):
            self.store.record_admission(decision)

        node_dir = self.store.materialize_mathematical_node(node)
        self.assertEqual(
            sorted(path.name for path in node_dir.iterdir()),
            ["inventory.json", "mathematical_payload.json", "node.json"],
        )
        self.assertEqual(
            json.loads((node_dir / "inventory.json").read_text())["admission_decision_sha256"],
            decision["admission_decision_sha256"],
        )
        duplicate_node = {**node, "node_id": "node-g0-copy"}
        duplicate_node.pop("node_artifact_sha256")
        duplicate_node["node_artifact_sha256"] = digest_json(duplicate_node)
        with self.assertRaisesRegex(protocol.ValidationError, "exactly one"):
            self.store.materialize_mathematical_node(duplicate_node)

        first = node_evaluation(node, evaluator_index=1)
        second = node_evaluation(node, evaluator_index=2)
        self.store.materialize_node_evaluation(first)
        self.store.materialize_node_evaluation(second)
        self.assertTrue(
            (
                self.run_dir
                / "node_evaluations"
                / str(node["node_id"])
                / "evaluator-1"
                / "evaluation.json"
            ).is_file()
        )

        reconciliation = frontier.reconcile_node_evaluations(
            f"reconcile-{node['node_id']}",
            first,
            second,
        )
        entry = frontier.build_archive_entry(node, reconciliation)
        archive_path = self.store.materialize_archive_entry(entry, reconciliation)
        self.assertEqual(
            archive_path,
            (self.run_dir / "archive_entries" / f"{node['node_id']}.json").resolve(),
        )
        with self.assertRaisesRegex(protocol.ValidationError, "already exists"):
            self.store.materialize_archive_entry(entry, reconciliation)

    def test_accepted_admission_requires_exactly_one_later_node_during_receipt_reconciliation(self) -> None:
        _, decision, node = accepted_records()
        self.store.record_admission(decision)

        with self.assertRaisesRegex(protocol.ValidationError, "accepted admission"):
            self.store.reconcile_run(ticket_count=0)

        self.store.materialize_mathematical_node(node)
        duplicate = self.store.run_dir / "mathematical_nodes" / "node-g0-duplicate"
        duplicate.mkdir()
        source = self.store.run_dir / "mathematical_nodes" / str(node["node_id"])
        duplicate_node = {**node, "node_id": "node-g0-duplicate"}
        duplicate_node.pop("node_artifact_sha256")
        duplicate_node["node_artifact_sha256"] = digest_json(duplicate_node)
        (duplicate / "node.json").write_text(json.dumps(duplicate_node))
        for name in ("mathematical_payload.json", "inventory.json"):
            (duplicate / name).write_bytes((source / name).read_bytes())

        with self.assertRaisesRegex(protocol.ValidationError, "accepted admission"):
            self.store.reconcile_run(ticket_count=0)

    def test_snapshot_is_reconstructed_deterministically_and_stale_snapshot_is_rejected(self) -> None:
        _, decision, node = accepted_records()
        self.store.record_admission(decision)
        self.store.materialize_mathematical_node(node)
        first = node_evaluation(node, evaluator_index=1)
        second = node_evaluation(node, evaluator_index=2, status="fail")
        self.store.materialize_node_evaluation(first)
        self.store.materialize_node_evaluation(second)
        reconciliation = frontier.reconcile_node_evaluations(
            f"reconcile-{node['node_id']}",
            first,
            second,
        )
        entry = frontier.build_archive_entry(node, reconciliation)
        self.store.materialize_archive_entry(entry, reconciliation)
        event = self.store.append_selection_event(
            {
                "schema_version": "crouzeix-selection-event/v1",
                "sequence": 1,
                "run_id": "expert-frontier-001",
                "event_kind": "selection_recorded",
                "ticket_id": "select-g1-d0",
                "artifact_sha256": DIGEST_B,
                "occurred_at_utc": "2026-08-15T00:01:00Z",
                "generation": 1,
                "draw_index": 0,
                "selected_node_id": str(node["node_id"]),
                "archive_snapshot_sha256": DIGEST_C,
            }
        )

        snapshot = self.store.project_snapshot()
        self.assertEqual(snapshot["archive_entry_count"], 1)
        self.assertEqual(snapshot["nodes_by_id"][node["node_id"]]["generation"], 0)
        self.assertEqual(
            snapshot["selection_events"][0]["selection_event_sha256"],
            event["selection_event_sha256"],
        )

        first_digest = snapshot["frontier_snapshot_sha256"]
        second_snapshot = self.store.project_snapshot()
        self.assertEqual(second_snapshot["frontier_snapshot_sha256"], first_digest)

        tampered = json.loads((self.run_dir / "frontier_snapshot.json").read_text())
        tampered["archive_entry_count"] = 2
        (self.run_dir / "frontier_snapshot.json").write_text(json.dumps(tampered))
        with self.assertRaisesRegex(protocol.ValidationError, "stale"):
            self.store.project_snapshot()

    def test_snapshot_digest_includes_frontier_events(self) -> None:
        snapshot = self.store.project_snapshot()
        self.assertEqual(snapshot["frontier_events"], [])

        self.store.append_frontier_event(frontier_event())
        with self.assertRaisesRegex(protocol.ValidationError, "stale"):
            self.store.project_snapshot()

    def test_candidate_projection_verifies_digests_and_cleans_partial_writes(self) -> None:
        _, decision, node = accepted_records()
        self.store.record_admission(decision)
        self.store.materialize_mathematical_node(node)
        first = node_evaluation(node, evaluator_index=1)
        second = node_evaluation(node, evaluator_index=2)
        self.store.materialize_node_evaluation(first)
        self.store.materialize_node_evaluation(second)
        reconciliation = frontier.reconcile_node_evaluations(
            f"reconcile-{node['node_id']}",
            first,
            second,
        )
        entry = frontier.build_archive_entry(node, reconciliation)
        self.store.materialize_archive_entry(entry, reconciliation)

        candidate_bytes = str(node["mathematical_payload"]["endpoint"]["text"]).encode("utf-8")
        projection = self.store.freeze_candidate(
            projection_id="candidate-node-g0-function-theory",
            node=node,
            reconciliation=reconciliation,
            candidate_bytes=candidate_bytes,
        )
        self.assertEqual(projection["candidate_sha256"], entry["candidate_proof_sha256"])
        self.assertTrue(
            (self.run_dir / "candidate_projections" / f"{projection['candidate_sha256']}.tex").is_file()
        )
        self.assertTrue((self.run_dir / "candidate_projections" / "index.json").is_file())

        with self.assertRaisesRegex(protocol.ValidationError, "candidate proof digest"):
            self.store.freeze_candidate(
                projection_id="candidate-tampered",
                node=node,
                reconciliation=reconciliation,
                candidate_bytes=b"tampered",
            )
        self.assertFalse((self.run_dir / "candidate_projections" / "candidate-tampered.tmp").exists())

    def test_candidate_projection_fails_if_final_path_appears_before_publish(self) -> None:
        _, decision, node = accepted_records()
        self.store.record_admission(decision)
        self.store.materialize_mathematical_node(node)
        first = node_evaluation(node, evaluator_index=1)
        second = node_evaluation(node, evaluator_index=2)
        self.store.materialize_node_evaluation(first)
        self.store.materialize_node_evaluation(second)
        reconciliation = frontier.reconcile_node_evaluations(
            f"reconcile-{node['node_id']}",
            first,
            second,
        )
        entry = frontier.build_archive_entry(node, reconciliation)
        self.store.materialize_archive_entry(entry, reconciliation)
        candidate_bytes = str(node["mathematical_payload"]["endpoint"]["text"]).encode("utf-8")
        candidate_sha256 = hashlib.sha256(candidate_bytes).hexdigest()
        projection_root = self.store.run_dir / "candidate_projections"
        final_candidate = projection_root / f"{candidate_sha256}.tex"
        original_write = frontier_store._write_bytes_create_only

        def write_with_intervening_final(path: Path, data: bytes, label: str) -> None:
            if label == "candidate bytes":
                projection_root.mkdir(exist_ok=True)
                final_candidate.write_text("intervening file")
            original_write(path, data, label)

        with mock.patch.object(
            frontier_store,
            "_write_bytes_create_only",
            side_effect=write_with_intervening_final,
        ), self.assertRaisesRegex(protocol.ValidationError, "already exists"):
            self.store.freeze_candidate(
                projection_id="candidate-node-g0-function-theory",
                node=node,
                reconciliation=reconciliation,
                candidate_bytes=candidate_bytes,
            )
        self.assertEqual(
            final_candidate.read_text(),
            "intervening file",
        )
        self.assertFalse((projection_root / "index.json").exists())

    def test_run_receipt_reconciles_tickets_attempts_calls_selections_nodes_and_terminals(self) -> None:
        self.store.append_attempt(
            attempt_ledger_record()
        )
        self.store.append_attempt(
            attempt_ledger_record(
                sequence=2,
                attempt_id="attempt-g0-resource-block",
                ticket_id="expert-g0-resource-block",
                proposed_node_id="node-g0-resource-block",
                terminal_status="blocked_resource",
                attempt_dir_sha256=DIGEST_B,
                occurred_at_utc="2026-08-15T00:02:00Z",
            )
        )
        self.store.materialize_attempt(
            attempt_id="attempt-g0-function-theory",
            context={"context": "bytes"},
            provider_call={"argv": ["traecli"], "exit_code": 0},
            receipt={"ticket_id": "expert-g0-function-theory"},
            expert_result=expert_result(),
            terminal_failure=None,
        )
        self.store.materialize_attempt(
            attempt_id="attempt-g0-resource-block",
            context={"context": "bytes"},
            provider_call={"argv": ["traecli"], "exit_code": 2},
            receipt={"ticket_id": "expert-g0-resource-block"},
            expert_result=None,
            terminal_failure={"terminal_status": "blocked_resource", "reason": "quota"},
        )
        _, decision, node = accepted_records()
        self.store.record_admission(decision)
        self.store.materialize_mathematical_node(node)
        first = node_evaluation(node, evaluator_index=1)
        second = node_evaluation(node, evaluator_index=2)
        self.store.materialize_node_evaluation(first)
        self.store.materialize_node_evaluation(second)
        reconciliation = frontier.reconcile_node_evaluations(
            f"reconcile-{node['node_id']}",
            first,
            second,
        )
        entry = frontier.build_archive_entry(node, reconciliation)
        self.store.materialize_archive_entry(entry, reconciliation)
        self.store.append_selection_event(
            {
                "schema_version": "crouzeix-selection-event/v1",
                "sequence": 1,
                "run_id": "expert-frontier-001",
                "event_kind": "selection_recorded",
                "ticket_id": "select-g1-d0",
                "artifact_sha256": DIGEST_C,
                "occurred_at_utc": "2026-08-15T00:01:00Z",
                "generation": 1,
                "draw_index": 0,
                "selected_node_id": str(node["node_id"]),
                "archive_snapshot_sha256": DIGEST_D,
            }
        )

        receipt = self.store.reconcile_run(ticket_count=3)

        self.assertEqual(receipt["ticket_count"], 3)
        self.assertEqual(receipt["attempt_count"], 2)
        self.assertEqual(receipt["provider_call_count"], 2)
        self.assertEqual(receipt["selection_count"], 1)
        self.assertEqual(receipt["mathematical_node_count"], 1)
        self.assertEqual(receipt["archive_entry_count"], 1)
        self.assertEqual(
            receipt["terminal_attempt_states"],
            {"blocked_resource": 1, "completed": 1},
        )

    def test_run_receipt_rejects_inconsistent_attempt_call_ticket_and_terminal_state(self) -> None:
        self.store.append_attempt(attempt_ledger_record())
        with self.assertRaisesRegex(protocol.ValidationError, "attempt directory"):
            self.store.reconcile_run(ticket_count=1)

        self.store.materialize_attempt(
            attempt_id="attempt-g0-function-theory",
            context={"context": "bytes"},
            provider_call={"argv": ["traecli"], "exit_code": 0},
            receipt={"ticket_id": "expert-g0-function-theory"},
            expert_result=expert_result(),
            terminal_failure=None,
        )
        extra = self.store.run_dir / "attempts" / "attempt-g0-extra"
        extra.mkdir()
        (extra / "attempt.json").write_text(
            json.dumps(
                {
                    "schema_version": "crouzeix-attempt-directory/v1",
                    "attempt_id": "attempt-g0-extra",
                    "terminal_status": "completed",
                }
            )
        )
        provider = extra / "provider_call"
        provider.mkdir()
        (provider / "call.json").write_text("{}")
        with self.assertRaisesRegex(protocol.ValidationError, "ledger"):
            self.store.reconcile_run(ticket_count=1)
        (provider / "call.json").unlink()
        provider.rmdir()
        (extra / "attempt.json").unlink()
        extra.rmdir()

        attempt_record = self.store.run_dir / "attempts" / "attempt-g0-function-theory" / "attempt.json"
        attempt_record.write_text(
            json.dumps(
                {
                    "schema_version": "crouzeix-attempt-directory/v1",
                    "attempt_id": "attempt-g0-function-theory",
                    "terminal_status": "failed",
                }
            )
        )
        with self.assertRaisesRegex(protocol.ValidationError, "terminal"):
            self.store.reconcile_run(ticket_count=1)
        attempt_record.write_text(
            json.dumps(
                {
                    "schema_version": "crouzeix-attempt-directory/v1",
                    "attempt_id": "attempt-g0-function-theory",
                    "terminal_status": "completed",
                }
            )
        )
        with self.assertRaisesRegex(protocol.ValidationError, "ticket_count"):
            self.store.reconcile_run(ticket_count=2)


if __name__ == "__main__":
    unittest.main()
