from __future__ import annotations

import json
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


if __name__ == "__main__":
    unittest.main()
