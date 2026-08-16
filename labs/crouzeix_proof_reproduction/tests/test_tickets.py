from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
REPO = LAB.parents[1]
sys.path.insert(0, str(LAB))

import protocol
import tickets


def runtime_ticket(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-runtime-ticket/v1",
        "ticket_id": "expert-g0-function-theory",
        "run_id": "expert-frontier-001",
        "task_kind": "expert",
        "node_id": "node-g0-function-theory",
        "parent_node_id": None,
        "generation": 0,
        "direction_id": "root-function-theory",
        "role": "function_theory",
        "objective": "Attempt an independent function-theory route.",
        "expected_deliverable": "Strict expert result or typed failure.",
        "dependency_ticket_ids": [],
        "context_sha256": "a" * 64,
        "schema_sha256": "b" * 64,
        "prompt_sha256": "c" * 64,
        "parent_artifact_sha256": None,
        "allowed_tools": ["Write"],
        "forbidden_sources": ["public proof manuscripts"],
        "timeout_seconds": 3600,
        "max_output_bytes": 1048576,
        "owner_type": "expert",
        "created_at_utc": "2026-08-15T12:00:00Z",
        "state": "created",
    }
    value.update(overrides)
    return value


def runtime_event(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-runtime-ticket-event/v1",
        "sequence": 1,
        "ticket_id": "expert-g0-function-theory",
        "from_state": "created",
        "to_state": "admitted",
        "reason": "ticket admitted before expert call",
        "occurred_at_utc": "2026-08-15T12:00:01Z",
        "artifact_sha256": "d" * 64,
    }
    value.update(overrides)
    return value


def frontier_event(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-frontier-event/v1",
        "sequence": 1,
        "run_id": "expert-frontier-001",
        "event_kind": "admission_accepted",
        "ticket_id": "expert-g0-function-theory",
        "artifact_sha256": "e" * 64,
        "occurred_at_utc": "2026-08-15T12:00:02Z",
    }
    value.update(overrides)
    return value


def write_legacy_run(root: Path, *, run_id: str = "historical-001") -> dict[str, str]:
    root.mkdir(parents=True)
    call_dir = root / "calls" / "historical-root"
    call_dir.mkdir(parents=True)
    run_spec = {
        "schema_version": "crouzeix-run-spec/v1",
        "run_id": run_id,
        "arm": "historical",
        "created_at_utc": "2026-08-15T05:21:29Z",
    }
    run_receipt = {
        "schema_version": "crouzeix-run-receipt/v1",
        "run_id": run_id,
        "arm": "historical",
        "completed_at_utc": "2026-08-15T05:59:49Z",
    }
    intervention = {
        "schema_version": "crouzeix-operator-intervention/v1",
        "run_id": run_id,
        "call_id": "historical-root",
        "occurred_at_utc": "2026-08-15T05:59:49Z",
        "action": "terminate_process_group",
        "reason": "host_free_space_below_safety_floor",
        "claim_boundary": "resource intervention only",
    }
    (root / "run_spec.json").write_text(json.dumps(run_spec, sort_keys=True), encoding="utf-8")
    (root / "run_receipt.json").write_text(json.dumps(run_receipt, sort_keys=True), encoding="utf-8")
    (root / "operator_intervention.json").write_text(
        json.dumps(intervention, sort_keys=True),
        encoding="utf-8",
    )
    (call_dir / "receipt.json").write_text(
        json.dumps({"schema_version": "call/v1", "call_id": "historical-root"}, sort_keys=True),
        encoding="utf-8",
    )
    call_manifest = tickets.legacy_call_manifest(root)
    migration_root = root.parent / "legacy-ticket-migration-001"
    ticket_root = migration_root / "tickets"
    migration_ticket = runtime_ticket(
        ticket_id="legacy-ticket-migration-001",
        run_id="legacy-ticket-migration-001",
        task_kind="operator_intervention",
        node_id=None,
        direction_id=None,
        role="legacy_ticket_migration",
        context_sha256="1" * 64,
        schema_sha256="2" * 64,
        prompt_sha256="3" * 64,
        owner_type="operator",
        created_at_utc="2026-08-16T06:00:00Z",
    )
    if not (ticket_root / "legacy-ticket-migration-001" / "ticket.json").exists():
        tickets.publish_ticket(ticket_root, migration_ticket)
    return {
        "run_spec_sha256": protocol.sha256_bytes((root / "run_spec.json").read_bytes()),
        "run_receipt_sha256": protocol.sha256_bytes((root / "run_receipt.json").read_bytes()),
        "operator_intervention_sha256": protocol.sha256_bytes(
            (root / "operator_intervention.json").read_bytes()
        ),
        "call_manifest_sha256": tickets.canonical_sha256(call_manifest),
        "call_count": str(call_manifest["call_count"]),
        "migration_ticket_sha256": tickets.canonical_sha256(migration_ticket),
    }


def write_legacy_exception(root: Path, digests: dict[str, str], **overrides: object) -> None:
    value: dict[str, object] = {
        "schema_version": "crouzeix-legacy-ticket-exception/v1",
        "classification": "legacy_pre_ticket_contract",
        "run_id": "historical-001",
        "arm": "historical",
        "original_run_spec_sha256": digests["run_spec_sha256"],
        "original_run_receipt_sha256": digests["run_receipt_sha256"],
        "original_operator_intervention_sha256": digests[
            "operator_intervention_sha256"
        ],
        "original_call_manifest_sha256": digests["call_manifest_sha256"],
        "original_call_count": int(digests["call_count"]),
        "original_completed_at_utc": "2026-08-15T05:59:49Z",
        "original_intervention_at_utc": "2026-08-15T05:59:49Z",
        "migration_ticket_id": "legacy-ticket-migration-001",
        "migration_ticket_sha256": digests["migration_ticket_sha256"],
        "migration_created_at_utc": "2026-08-16T06:00:00Z",
        "claim_boundary": "audit-only exception; not a retroactive runtime ticket",
    }
    value.update(overrides)
    (root / "legacy_ticket_exception.json").write_text(
        json.dumps(value, sort_keys=True),
        encoding="utf-8",
    )


class TrackerValidationTests(unittest.TestCase):
    def test_current_tracker_is_complete_and_dependency_graph_allows_cpfr010(self) -> None:
        tracker = tickets.parse_tracker(REPO / "docs/workstream/crouzeix-proof-reproduction/tracker.org")

        tickets.validate_tracker(tracker)
        tickets.assert_ticket_executable(tracker, "CPFR-010")

    def test_tracker_rejects_missing_sections_blank_assignments_and_bad_dependencies(self) -> None:
        tracker = tickets.parse_tracker(REPO / "docs/workstream/crouzeix-proof-reproduction/tracker.org")
        item = tracker.items["CPFR-010"]

        without_scope = item.without_section("Scope")
        broken = tracker.replace(without_scope)
        with self.assertRaisesRegex(protocol.ValidationError, "Scope"):
            tickets.validate_tracker(broken)

        blank_owner = item.with_property("OWNER", "")
        broken = tracker.replace(blank_owner)
        with self.assertRaisesRegex(protocol.ValidationError, "OWNER"):
            tickets.validate_tracker(broken)

        unknown_dependency = item.with_property("DEPENDS_ON", "CPFR-999")
        broken = tracker.replace(unknown_dependency)
        with self.assertRaisesRegex(protocol.ValidationError, "unknown dependency"):
            tickets.validate_tracker(broken)

        self_dependency = item.with_property("DEPENDS_ON", "CPFR-010")
        broken = tracker.replace(self_dependency)
        with self.assertRaisesRegex(protocol.ValidationError, "self dependency"):
            tickets.validate_tracker(broken)


class RuntimeTicketTests(unittest.TestCase):
    def test_runtime_ticket_rejects_unknown_fields_unsafe_ids_and_invalid_limits(self) -> None:
        self.assertEqual(tickets.validate_runtime_ticket(runtime_ticket())["ticket_id"], "expert-g0-function-theory")

        with self.assertRaisesRegex(protocol.ValidationError, "fields"):
            tickets.validate_runtime_ticket(runtime_ticket(extra=True))
        with self.assertRaisesRegex(protocol.ValidationError, "ticket_id"):
            tickets.validate_runtime_ticket(runtime_ticket(ticket_id="../escape"))
        with self.assertRaisesRegex(protocol.ValidationError, "context_sha256"):
            tickets.validate_runtime_ticket(runtime_ticket(context_sha256="bad"))
        with self.assertRaisesRegex(protocol.ValidationError, "owner_type"):
            tickets.validate_runtime_ticket(runtime_ticket(owner_type="scheduler"))
        with self.assertRaisesRegex(protocol.ValidationError, "timeout_seconds"):
            tickets.validate_runtime_ticket(runtime_ticket(timeout_seconds=0))

    def test_runtime_event_state_machine_is_closed(self) -> None:
        registry = tickets.RuntimeTicketEventLog("expert-g0-function-theory")
        registry.apply(runtime_event(sequence=1, from_state="created", to_state="admitted"))
        registry.apply(runtime_event(sequence=2, from_state="admitted", to_state="running"))
        registry.apply(runtime_event(sequence=3, from_state="running", to_state="completed"))
        registry.apply(runtime_event(sequence=4, from_state="completed", to_state="accepted"))

        with self.assertRaisesRegex(protocol.ValidationError, "terminal"):
            registry.apply(runtime_event(sequence=5, from_state="accepted", to_state="running"))

        other = tickets.RuntimeTicketEventLog("expert-g0-function-theory")
        with self.assertRaisesRegex(protocol.ValidationError, "transition"):
            other.apply(runtime_event(sequence=1, from_state="created", to_state="completed"))


class FrontierEventTests(unittest.TestCase):
    def test_frontier_events_are_closed_and_reject_node_lifecycle_states(self) -> None:
        for kind in (
            "attempt_terminal",
            "admission_accepted",
            "admission_rejected",
            "archive_entry_created",
            "selection_recorded",
            "candidate_projected",
        ):
            self.assertEqual(
                tickets.validate_frontier_event(frontier_event(event_kind=kind))[
                    "event_kind"
                ],
                kind,
            )

        with self.assertRaisesRegex(protocol.ValidationError, "event_kind"):
            tickets.validate_frontier_event(frontier_event(event_kind="selected"))
        with self.assertRaisesRegex(protocol.ValidationError, "event_kind"):
            tickets.validate_frontier_event(frontier_event(event_kind="complete"))
        with self.assertRaisesRegex(protocol.ValidationError, "fields"):
            tickets.validate_frontier_event(frontier_event(node_state="superseded"))

    def test_score_complete_is_derived_from_archive_entry_not_node_state(self) -> None:
        self.assertTrue(
            tickets.derive_score_complete(
                {
                    "unanimous_pass_count": 10,
                    "candidate_proof_sha256": "f" * 64,
                }
            )
        )
        self.assertFalse(
            tickets.derive_score_complete(
                {
                    "unanimous_pass_count": 9,
                    "candidate_proof_sha256": "f" * 64,
                }
            )
        )
        with self.assertRaisesRegex(protocol.ValidationError, "candidate"):
            tickets.derive_score_complete(
                {
                    "unanimous_pass_count": 10,
                    "candidate_proof_sha256": None,
                }
            )


class StorageAndBindingTests(unittest.TestCase):
    def test_publish_ticket_and_append_events_use_contract_layout(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            ticket_root = Path(directory).resolve() / "tickets"

            ticket_dir = tickets.publish_ticket(ticket_root, runtime_ticket())
            ticket_path = ticket_dir / "ticket.json"
            event_path = ticket_dir / "ticket_events.jsonl"
            self.assertEqual(ticket_dir, ticket_root / "expert-g0-function-theory")
            self.assertTrue(ticket_path.is_file())
            with self.assertRaisesRegex(protocol.ValidationError, "already exists"):
                tickets.publish_ticket(ticket_root, runtime_ticket())

            tickets.append_ticket_event(ticket_root, "expert-g0-function-theory", runtime_event())
            tickets.append_ticket_event(
                ticket_root,
                "expert-g0-function-theory",
                runtime_event(sequence=2, from_state="admitted", to_state="running"),
            )
            self.assertEqual(len(event_path.read_text().splitlines()), 2)

    def test_append_replays_existing_events_and_rejects_invalid_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            ticket_root = Path(directory).resolve() / "tickets"
            tickets.publish_ticket(ticket_root, runtime_ticket())
            tickets.append_ticket_event(ticket_root, "expert-g0-function-theory", runtime_event())

            with self.assertRaisesRegex(protocol.ValidationError, "sequence"):
                tickets.append_ticket_event(
                    ticket_root,
                    "expert-g0-function-theory",
                    runtime_event(sequence=1, from_state="created", to_state="canceled"),
                )
            with self.assertRaisesRegex(protocol.ValidationError, "from_state"):
                tickets.append_ticket_event(
                    ticket_root,
                    "expert-g0-function-theory",
                    runtime_event(sequence=2, from_state="created", to_state="canceled"),
                )
            self.assertEqual(
                len((ticket_root / "expert-g0-function-theory" / "ticket_events.jsonl").read_text().splitlines()),
                1,
            )

    def test_append_requires_published_ticket_json(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            ticket_root = Path(directory).resolve() / "tickets"
            (ticket_root / "expert-g0-function-theory").mkdir(parents=True)

            with self.assertRaisesRegex(protocol.ValidationError, "ticket.json"):
                tickets.append_ticket_event(
                    ticket_root,
                    "expert-g0-function-theory",
                    runtime_event(),
                )

    def test_ticket_publication_rejects_symlinked_ancestors(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            target = root / "target"
            target.mkdir()
            ticket_root = root / "tickets"
            ticket_root.symlink_to(target)

            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                tickets.publish_ticket(ticket_root, runtime_ticket())

            safe_root = root / "safe-tickets"
            safe_root.mkdir()
            ticket_dir = safe_root / "expert-g0-function-theory"
            ticket_dir.symlink_to(target)
            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                tickets.append_ticket_event(
                    safe_root,
                    "expert-g0-function-theory",
                    runtime_event(),
                )

    def test_ticket_binding_requires_matching_ticket_id_and_digest(self) -> None:
        ticket = runtime_ticket()
        ticket_sha256 = tickets.canonical_sha256(ticket)

        tickets.validate_ticket_binding(
            {
                "ticket_id": ticket["ticket_id"],
                "ticket_sha256": ticket_sha256,
            },
            ticket,
        )

        with self.assertRaisesRegex(protocol.ValidationError, "ticket_id"):
            tickets.validate_ticket_binding({"ticket_sha256": ticket_sha256}, ticket)
        with self.assertRaisesRegex(protocol.ValidationError, "ticket_sha256"):
            tickets.validate_ticket_binding(
                {"ticket_id": ticket["ticket_id"], "ticket_sha256": "0" * 64},
                ticket,
            )


class TicketsCliTests(unittest.TestCase):
    def run_tickets(self, *args: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, str(LAB / "tickets.py"), *args],
            cwd=REPO,
            text=True,
            capture_output=True,
            check=False,
        )

    def test_validate_tracker_cli_checks_the_real_tracker(self) -> None:
        result = self.run_tickets(
            "validate-tracker",
            "docs/workstream/crouzeix-proof-reproduction/tracker.org",
        )

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_validate_tracker_cli_reports_missing_files_without_traceback(self) -> None:
        result = self.run_tickets("validate-tracker", "/no/such/tracker.org")

        self.assertEqual(result.returncode, 2)
        self.assertIn("tickets:", result.stderr)
        self.assertNotIn("Traceback", result.stderr)

    def test_validate_runtime_cli_replays_each_published_ticket(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            run_root = Path(directory).resolve()
            ticket_root = run_root / "tickets"
            tickets.publish_ticket(ticket_root, runtime_ticket())
            tickets.append_ticket_event(ticket_root, "expert-g0-function-theory", runtime_event())
            tickets.append_ticket_event(
                ticket_root,
                "expert-g0-function-theory",
                runtime_event(sequence=2, from_state="admitted", to_state="running"),
            )
            tickets.append_ticket_event(
                ticket_root,
                "expert-g0-function-theory",
                runtime_event(sequence=3, from_state="running", to_state="completed"),
            )
            tickets.append_ticket_event(
                ticket_root,
                "expert-g0-function-theory",
                runtime_event(sequence=4, from_state="completed", to_state="accepted"),
            )

            result = self.run_tickets("validate-runtime", str(run_root))

        self.assertEqual(result.returncode, 0, result.stderr)

    def test_validate_runtime_cli_fails_on_nonterminal_ticket(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            run_root = Path(directory).resolve()
            ticket_root = run_root / "tickets"
            tickets.publish_ticket(ticket_root, runtime_ticket())
            tickets.append_ticket_event(ticket_root, "expert-g0-function-theory", runtime_event())

            result = self.run_tickets("validate-runtime", str(run_root))

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("not terminal", result.stderr)

    def test_validate_runtime_cli_requires_event_log_for_generation_zero_expert_ticket(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            run_root = Path(directory).resolve()
            ticket_root = run_root / "tickets"
            tickets.publish_ticket(ticket_root, runtime_ticket())

            result = self.run_tickets("validate-runtime", str(run_root))

        self.assertEqual(result.returncode, 2)
        self.assertIn("event log", result.stderr)

    def test_validate_runtime_cli_rejects_oversized_ticket_before_reading(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            run_root = Path(directory).resolve()
            ticket_dir = run_root / "tickets" / "expert-g0-function-theory"
            ticket_dir.mkdir(parents=True)
            (ticket_dir / "ticket.json").write_bytes(b"{" + b" " * (1024 * 1024 + 1))
            (ticket_dir / "ticket_events.jsonl").write_text("", encoding="utf-8")

            result = self.run_tickets("validate-runtime", str(run_root))

        self.assertEqual(result.returncode, 2)
        self.assertIn("exceeds byte cap", result.stderr)

    def test_validate_legacy_cli_requires_digest_bound_exception_receipts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            run_root = Path(directory).resolve() / "historical-001"
            digests = write_legacy_run(run_root)

            missing = self.run_tickets("validate-legacy", str(run_root))
            write_legacy_exception(run_root, digests)
            valid = self.run_tickets("validate-legacy", str(run_root))

        self.assertEqual(missing.returncode, 2)
        self.assertIn("legacy exception", missing.stderr)
        self.assertEqual(valid.returncode, 0, valid.stderr)

    def test_validate_legacy_cli_rejects_digest_drift_and_backdated_migrations(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            run_root = Path(directory).resolve() / "historical-001"
            digests = write_legacy_run(run_root)
            write_legacy_exception(run_root, digests, original_run_receipt_sha256="2" * 64)

            drift = self.run_tickets("validate-legacy", str(run_root))

            write_legacy_exception(
                run_root,
                digests,
                migration_created_at_utc="2026-08-15T05:00:00Z",
            )
            backdated = self.run_tickets("validate-legacy", str(run_root))

        self.assertEqual(drift.returncode, 2)
        self.assertIn("run_receipt", drift.stderr)
        self.assertEqual(backdated.returncode, 2)
        self.assertIn("migration", backdated.stderr)

    def test_validate_legacy_cli_binds_call_manifest_and_migration_ticket(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            run_root = Path(directory).resolve() / "historical-001"
            digests = write_legacy_run(run_root)
            write_legacy_exception(run_root, digests, original_call_count=2)

            call_count = self.run_tickets("validate-legacy", str(run_root))

            write_legacy_exception(run_root, digests, migration_ticket_sha256="4" * 64)
            migration_digest = self.run_tickets("validate-legacy", str(run_root))

            write_legacy_exception(
                run_root,
                digests,
                claim_boundary="retroactive runtime ticket governing the original action",
            )
            retroactive = self.run_tickets("validate-legacy", str(run_root))

        self.assertEqual(call_count.returncode, 2)
        self.assertIn("call", call_count.stderr)
        self.assertEqual(migration_digest.returncode, 2)
        self.assertIn("migration_ticket_sha256", migration_digest.stderr)
        self.assertEqual(retroactive.returncode, 2)
        self.assertIn("retroactive", retroactive.stderr)

    def test_validate_legacy_cli_rejects_symlinked_migration_ticket_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            run_root = root / "historical-001"
            digests = write_legacy_run(run_root)
            real_migration = root / "legacy-ticket-migration-real"
            (root / "legacy-ticket-migration-001").rename(real_migration)
            (root / "legacy-ticket-migration-001").symlink_to(real_migration)
            write_legacy_exception(run_root, digests)

            result = self.run_tickets("validate-legacy", str(run_root))

        self.assertEqual(result.returncode, 2)
        self.assertIn("symlink", result.stderr)

    def test_validate_legacy_cli_rejects_loose_original_json_and_bad_timestamp(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            run_root = root / "historical-001"
            digests = write_legacy_run(run_root)
            write_legacy_exception(run_root, digests)

            run_spec = json.loads((run_root / "run_spec.json").read_text(encoding="utf-8"))
            run_spec["unknown"] = True
            (run_root / "run_spec.json").write_text(
                json.dumps(run_spec, sort_keys=True),
                encoding="utf-8",
            )
            loose = self.run_tickets("validate-legacy", str(run_root))

            timestamp_root = root / "historical-002"
            digests = write_legacy_run(timestamp_root, run_id="historical-002")
            write_legacy_exception(
                timestamp_root,
                digests,
                run_id="historical-002",
                migration_created_at_utc="not-a-dateZ",
            )
            timestamp = self.run_tickets("validate-legacy", str(timestamp_root))

        self.assertEqual(loose.returncode, 2)
        self.assertIn("run_spec", loose.stderr)
        self.assertEqual(timestamp.returncode, 2)
        self.assertIn("timestamp", timestamp.stderr)


if __name__ == "__main__":
    unittest.main()
