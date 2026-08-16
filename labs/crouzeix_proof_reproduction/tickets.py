from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import stat
import sys
from dataclasses import dataclass, replace
from pathlib import Path
from typing import Any, Mapping

from protocol import ValidationError, sha256_bytes


TRACKER_REQUIRED_PROPERTIES = frozenset(
    {
        "ID",
        "DESIGN_SECTION",
        "DEPENDS_ON",
        "BRANCH",
        "WORKTREE",
        "OWNER",
        "AGENT_RUN",
        "MODEL",
    }
)
TRACKER_REQUIRED_SECTIONS = (
    "Scope",
    "Non-goals",
    "Owned Files",
    "Acceptance Criteria",
    "Implementation Steps",
    "Verification Plan",
    "Verification Evidence",
)
TERMINAL_TRACKER_STATES = frozenset({"DONE", "CANCELED"})
TRACKER_STATES = frozenset(
    {"BACKLOG", "TODO", "IMPLEMENTING", "REVIEW", "BLOCKED", "DONE", "CANCELED"}
)

RUNTIME_TICKET_FIELDS = frozenset(
    {
        "schema_version",
        "ticket_id",
        "run_id",
        "task_kind",
        "node_id",
        "parent_node_id",
        "generation",
        "direction_id",
        "role",
        "objective",
        "expected_deliverable",
        "dependency_ticket_ids",
        "context_sha256",
        "schema_sha256",
        "prompt_sha256",
        "parent_artifact_sha256",
        "allowed_tools",
        "forbidden_sources",
        "timeout_seconds",
        "max_output_bytes",
        "owner_type",
        "created_at_utc",
        "state",
    }
)
TASK_KINDS = frozenset(
    {
        "expert",
        "evaluator",
        "selection",
        "candidate_freeze",
        "operator_intervention",
        "correctness_review",
        "correctness_repair",
        "finding_reconciliation",
        "mechanism_classification",
        "formal_attempt",
    }
)
OWNER_TYPES = frozenset({"expert", "evaluator", "reviewer", "orchestrator", "operator"})
TICKET_STATES = frozenset(
    {
        "created",
        "admitted",
        "running",
        "completed",
        "failed",
        "timed_out",
        "blocked_resource",
        "accepted",
        "rejected",
        "superseded",
        "canceled",
    }
)
TICKET_TRANSITIONS = frozenset(
    {
        ("created", "admitted"),
        ("created", "canceled"),
        ("admitted", "running"),
        ("admitted", "canceled"),
        ("running", "completed"),
        ("running", "failed"),
        ("running", "timed_out"),
        ("running", "blocked_resource"),
        ("running", "canceled"),
        ("completed", "accepted"),
        ("completed", "rejected"),
        ("completed", "superseded"),
    }
)
TERMINAL_TICKET_STATES = frozenset(
    {
        "failed",
        "timed_out",
        "blocked_resource",
        "accepted",
        "rejected",
        "superseded",
        "canceled",
    }
)
RUNTIME_EVENT_FIELDS = frozenset(
    {
        "schema_version",
        "sequence",
        "ticket_id",
        "from_state",
        "to_state",
        "reason",
        "occurred_at_utc",
        "artifact_sha256",
    }
)
LEGACY_EXCEPTION_FIELDS = frozenset(
    {
        "schema_version",
        "classification",
        "run_id",
        "arm",
        "original_run_spec_sha256",
        "original_run_receipt_sha256",
        "original_operator_intervention_sha256",
        "original_call_manifest_sha256",
        "original_call_count",
        "original_completed_at_utc",
        "original_intervention_at_utc",
        "migration_ticket_id",
        "migration_ticket_sha256",
        "migration_created_at_utc",
        "claim_boundary",
    }
)
LEGACY_CLASSIFICATIONS = frozenset({"legacy_pre_ticket_contract"})
LEGACY_RUN_SPEC_FIELDS = frozenset(
    {
        "schema_version",
        "run_id",
        "arm",
        "leakage",
        "model",
        "cli",
        "historical_prompt",
        "sandbox",
        "approval_policy",
        "allowed_tools",
        "network_access",
        "timeout_seconds",
        "max_calls",
        "token_accounting",
        "generation_visible_files",
        "generation_excluded_classes",
        "created_at_utc",
    }
)
LEGACY_RUN_RECEIPT_FIELDS = frozenset(
    {
        "schema_version",
        "run_id",
        "arm",
        "leakage",
        "model",
        "execution_status",
        "failure",
        "promotion",
        "candidate_sha256",
        "call_count",
        "call_status_counts",
        "usage",
        "completed_at_utc",
    }
)
LEGACY_OPERATOR_INTERVENTION_FIELDS = frozenset(
    {
        "schema_version",
        "run_id",
        "call_id",
        "occurred_at_utc",
        "action",
        "reason",
        "observed_available_kib",
        "completed_calls_before_intervention",
        "configured_timeout_seconds",
        "elapsed_seconds_approximate",
        "claim_boundary",
    }
)

FRONTIER_EVENT_FIELDS = frozenset(
    {
        "schema_version",
        "sequence",
        "run_id",
        "event_kind",
        "ticket_id",
        "artifact_sha256",
        "occurred_at_utc",
    }
)
FRONTIER_EVENT_KINDS = frozenset(
    {
        "attempt_terminal",
        "admission_accepted",
        "admission_rejected",
        "archive_entry_created",
        "selection_recorded",
        "candidate_projected",
    }
)

PORTABLE_RUNTIME_ID_CHARS = frozenset(
    "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-"
)


@dataclass(frozen=True)
class TrackerItem:
    state: str
    title: str
    properties: dict[str, str]
    sections: dict[str, str]

    @property
    def item_id(self) -> str:
        return self.properties["ID"]

    def with_property(self, key: str, value: str) -> "TrackerItem":
        properties = dict(self.properties)
        properties[key] = value
        return replace(self, properties=properties)

    def without_section(self, name: str) -> "TrackerItem":
        sections = dict(self.sections)
        sections.pop(name, None)
        return replace(self, sections=sections)


@dataclass(frozen=True)
class Tracker:
    items: dict[str, TrackerItem]

    def replace(self, item: TrackerItem) -> "Tracker":
        items = dict(self.items)
        items[item.item_id] = item
        return Tracker(items)


def parse_tracker(path: Path) -> Tracker:
    text = _read_bounded_text_file(path, "tracker", max_bytes=2 * 1024 * 1024)
    items: dict[str, TrackerItem] = {}
    current: dict[str, Any] | None = None
    in_properties = False
    current_section: str | None = None
    section_lines: list[str] = []

    def flush_section() -> None:
        nonlocal current_section, section_lines
        if current is not None and current_section is not None:
            current["sections"][current_section] = "\n".join(section_lines).strip()
        current_section = None
        section_lines = []

    def flush_item() -> None:
        if current is None:
            return
        flush_section()
        properties = dict(current["properties"])
        if "ID" not in properties:
            raise ValidationError(f"tracker item {current['title']} missing ID")
        item = TrackerItem(
            state=str(current["state"]),
            title=str(current["title"]),
            properties=properties,
            sections=dict(current["sections"]),
        )
        if item.item_id in items:
            raise ValidationError(f"duplicate tracker ID {item.item_id}")
        items[item.item_id] = item

    for raw in text.splitlines():
        if raw.startswith("** "):
            flush_item()
            in_properties = False
            current_section = None
            section_lines = []
            parts = raw[3:].split(None, 1)
            state = parts[0]
            title = parts[1] if len(parts) > 1 else ""
            current = {
                "state": state,
                "title": title,
                "properties": {},
                "sections": {},
            }
            continue
        if current is None:
            continue
        if raw.strip() == ":PROPERTIES:":
            flush_section()
            in_properties = True
            continue
        if raw.strip() == ":END:" and in_properties:
            in_properties = False
            continue
        if in_properties:
            stripped = raw.strip()
            if stripped.startswith(":") and ":" in stripped[1:]:
                key, value = stripped[1:].split(":", 1)
                current["properties"][key] = value.strip()
            continue
        if raw.startswith("*** "):
            flush_section()
            current_section = raw[4:].strip()
            continue
        if current_section is not None:
            section_lines.append(raw)
    flush_item()
    return Tracker(items)


def validate_tracker(tracker: Tracker) -> None:
    for item in tracker.items.values():
        if item.state not in TRACKER_STATES:
            raise ValidationError(f"{item.item_id} has invalid state {item.state}")
        missing_properties = TRACKER_REQUIRED_PROPERTIES - set(item.properties)
        if missing_properties:
            raise ValidationError(
                f"{item.item_id} missing properties: {', '.join(sorted(missing_properties))}"
            )
        for key in TRACKER_REQUIRED_PROPERTIES:
            value = item.properties.get(key, "")
            if value == "" and key != "DEPENDS_ON":
                raise ValidationError(f"{item.item_id} {key} must be nonempty")
        for key in ("OWNER", "AGENT_RUN", "MODEL"):
            value = item.properties[key]
            if item.state in {"TODO", "BACKLOG"} and value != "unassigned":
                raise ValidationError(f"{item.item_id} {key} must be unassigned")
            if item.state not in {"TODO", "BACKLOG"} and value == "unassigned":
                raise ValidationError(f"{item.item_id} {key} must be claimed")
        missing_sections = [
            name
            for name in TRACKER_REQUIRED_SECTIONS
            if not item.sections.get(name, "").strip()
        ]
        if missing_sections:
            raise ValidationError(
                f"{item.item_id} missing section {missing_sections[0]}"
            )
    _validate_dependencies(tracker)


def assert_ticket_executable(tracker: Tracker, item_id: str) -> None:
    validate_tracker(tracker)
    item = tracker.items[item_id]
    for dependency in _dependencies(item):
        dependency_item = tracker.items[dependency]
        if dependency_item.state not in TERMINAL_TRACKER_STATES:
            raise ValidationError(
                f"{item_id} depends on nonterminal ticket {dependency}"
            )


def validate_runtime_ticket(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, RUNTIME_TICKET_FIELDS, "runtime ticket")
    _require_equal(value["schema_version"], "crouzeix-runtime-ticket/v1", "schema_version")
    _runtime_id(value["ticket_id"], "ticket_id")
    _runtime_id(value["run_id"], "run_id")
    _enum(value["task_kind"], TASK_KINDS, "task_kind")
    if value["node_id"] is not None:
        _runtime_id(value["node_id"], "node_id")
    if value["parent_node_id"] is not None:
        _runtime_id(value["parent_node_id"], "parent_node_id")
    _bounded_integer(value["generation"], "generation", 0, 1_000)
    if value["direction_id"] is not None:
        _runtime_id(value["direction_id"], "direction_id")
    _bounded_string(value["role"], "role", 1, 128)
    _bounded_string(value["objective"], "objective", 1, 4096)
    _bounded_string(value["expected_deliverable"], "expected_deliverable", 1, 4096)
    dependencies = _string_list(
        value["dependency_ticket_ids"],
        "dependency_ticket_ids",
        minimum=0,
        maximum=128,
    )
    if len(set(dependencies)) != len(dependencies):
        raise ValidationError("dependency_ticket_ids must be unique")
    for dependency in dependencies:
        _runtime_id(dependency, "dependency_ticket_ids")
    _digest(value["context_sha256"], "context_sha256")
    _digest(value["schema_sha256"], "schema_sha256")
    _digest(value["prompt_sha256"], "prompt_sha256")
    if value["parent_artifact_sha256"] is not None:
        _digest(value["parent_artifact_sha256"], "parent_artifact_sha256")
    _string_list(value["allowed_tools"], "allowed_tools", minimum=0, maximum=16)
    _string_list(value["forbidden_sources"], "forbidden_sources", minimum=0, maximum=64)
    _bounded_integer(value["timeout_seconds"], "timeout_seconds", 1, 86_400)
    _bounded_integer(value["max_output_bytes"], "max_output_bytes", 1, 128 * 1024 * 1024)
    _enum(value["owner_type"], OWNER_TYPES, "owner_type")
    _timestamp(value["created_at_utc"], "created_at_utc")
    _enum(value["state"], TICKET_STATES, "state")
    if value["state"] != "created":
        raise ValidationError("runtime ticket initial state must be created")
    return dict(value)


def validate_runtime_ticket_event(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, RUNTIME_EVENT_FIELDS, "runtime ticket event")
    _require_equal(
        value["schema_version"],
        "crouzeix-runtime-ticket-event/v1",
        "schema_version",
    )
    _bounded_integer(value["sequence"], "sequence", 1, 1_000_000)
    _runtime_id(value["ticket_id"], "ticket_id")
    _enum(value["from_state"], TICKET_STATES, "from_state")
    _enum(value["to_state"], TICKET_STATES, "to_state")
    transition = (str(value["from_state"]), str(value["to_state"]))
    if transition not in TICKET_TRANSITIONS:
        raise ValidationError(f"invalid runtime ticket transition {transition}")
    _bounded_string(value["reason"], "reason", 1, 4096)
    _timestamp(value["occurred_at_utc"], "occurred_at_utc")
    if value["artifact_sha256"] is not None:
        _digest(value["artifact_sha256"], "artifact_sha256")
    return dict(value)


class RuntimeTicketEventLog:
    def __init__(self, ticket_id: str) -> None:
        self.ticket_id = _runtime_id(ticket_id, "ticket_id")
        self.state = "created"
        self.events: list[dict[str, object]] = []

    def apply(self, raw_event: Mapping[str, Any]) -> None:
        if self.state in TERMINAL_TICKET_STATES:
            raise ValidationError(f"terminal ticket {self.ticket_id} cannot transition")
        event = validate_runtime_ticket_event(raw_event)
        if event["ticket_id"] != self.ticket_id:
            raise ValidationError("runtime ticket event ticket_id mismatch")
        expected_sequence = len(self.events) + 1
        if event["sequence"] != expected_sequence:
            raise ValidationError(
                f"runtime ticket event sequence must be {expected_sequence}"
            )
        if event["from_state"] != self.state:
            raise ValidationError("runtime ticket event from_state mismatch")
        self.state = str(event["to_state"])
        self.events.append(event)


def validate_frontier_event(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, FRONTIER_EVENT_FIELDS, "frontier event")
    _require_equal(value["schema_version"], "crouzeix-frontier-event/v1", "schema_version")
    _bounded_integer(value["sequence"], "sequence", 1, 1_000_000)
    _runtime_id(value["run_id"], "run_id")
    _enum(value["event_kind"], FRONTIER_EVENT_KINDS, "event_kind")
    _runtime_id(value["ticket_id"], "ticket_id")
    _digest(value["artifact_sha256"], "artifact_sha256")
    _timestamp(value["occurred_at_utc"], "occurred_at_utc")
    return dict(value)


def derive_score_complete(archive_entry: Mapping[str, Any]) -> bool:
    count = _bounded_integer(
        archive_entry.get("unanimous_pass_count"),
        "unanimous_pass_count",
        0,
        10,
    )
    candidate = archive_entry.get("candidate_proof_sha256")
    if count == 10:
        _digest(candidate, "candidate_proof_sha256")
        return True
    if candidate is not None:
        _digest(candidate, "candidate_proof_sha256")
    return False


def publish_ticket(ticket_root: Path, ticket: Mapping[str, Any]) -> Path:
    value = validate_runtime_ticket(ticket)
    _ensure_safe_directory(ticket_root, "runtime ticket root", create=True)
    ticket_dir = ticket_root / str(value["ticket_id"])
    _ensure_safe_directory(ticket_dir, "runtime ticket directory", create=True)
    path = ticket_dir / "ticket.json"
    _reject_symlink(path, "runtime ticket")
    try:
        with path.open("x", encoding="utf-8") as handle:
            json.dump(value, handle, sort_keys=True, separators=(",", ":"))
            handle.write("\n")
    except FileExistsError as error:
        raise ValidationError(f"runtime ticket already exists: {path}") from error
    return ticket_dir


def append_ticket_event(
    ticket_root: Path,
    ticket_id: str,
    event: Mapping[str, Any],
) -> None:
    ticket_id = _runtime_id(ticket_id, "ticket_id")
    _ensure_safe_directory(ticket_root, "runtime ticket root", create=False)
    ticket_dir = ticket_root / ticket_id
    _ensure_safe_directory(ticket_dir, "runtime ticket directory", create=False)
    ticket_path = ticket_dir / "ticket.json"
    _reject_symlink(ticket_path, "runtime ticket")
    if not ticket_path.is_file():
        raise ValidationError(f"runtime ticket ticket.json is missing: {ticket_path}")
    try:
        published_ticket = json.loads(ticket_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise ValidationError(f"cannot parse runtime ticket ticket.json: {error}") from error
    published = validate_runtime_ticket(published_ticket)
    if published["ticket_id"] != ticket_id:
        raise ValidationError("runtime ticket ticket.json does not match ticket_id")
    path = ticket_dir / "ticket_events.jsonl"
    value = validate_runtime_ticket_event(event)
    _reject_symlink(path, "runtime ticket event log")
    _validate_existing_event_log(path, ticket_id)
    replay = RuntimeTicketEventLog(ticket_id)
    if path.exists():
        for line in path.read_text(encoding="utf-8").splitlines():
            try:
                previous = json.loads(line)
            except json.JSONDecodeError as error:
                raise ValidationError(f"cannot parse runtime ticket event log: {error}") from error
            replay.apply(previous)
    replay.apply(value)
    flags = os.O_APPEND | os.O_CREAT | os.O_WRONLY
    fd = os.open(path, flags, 0o600)
    try:
        payload = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
        if len(payload) > 64 * 1024:
            raise ValidationError("runtime ticket event exceeds byte cap")
        os.write(fd, payload + b"\n")
    finally:
        os.close(fd)


def validate_ticket_binding(
    binding: Mapping[str, Any],
    ticket: Mapping[str, Any],
) -> None:
    value = validate_runtime_ticket(ticket)
    _require_fields(binding, frozenset({"ticket_id", "ticket_sha256"}), "ticket binding")
    if binding["ticket_id"] != value["ticket_id"]:
        raise ValidationError("ticket_id does not match ticket")
    if binding["ticket_sha256"] != canonical_sha256(value):
        raise ValidationError("ticket_sha256 does not match ticket")


def canonical_sha256(value: Mapping[str, Any]) -> str:
    data = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    return sha256_bytes(data.encode("utf-8"))


def validate_runtime_root(run_root: Path) -> None:
    _ensure_safe_directory(run_root, "runtime root", create=False)
    ticket_root = run_root / "tickets"
    _ensure_safe_directory(ticket_root, "runtime ticket root", create=False)
    ticket_dirs = sorted(path for path in ticket_root.iterdir() if path.is_dir())
    if not ticket_dirs:
        raise ValidationError("runtime root has no published tickets")
    for ticket_dir in ticket_dirs:
        _reject_symlink(ticket_dir, "runtime ticket directory")
        ticket_path = ticket_dir / "ticket.json"
        ticket_text = _read_bounded_text_file(
            ticket_path,
            "runtime ticket",
            max_bytes=1024 * 1024,
        )
        try:
            ticket = validate_runtime_ticket(json.loads(ticket_text))
        except json.JSONDecodeError as error:
            raise ValidationError(f"cannot parse runtime ticket: {error}") from error
        ticket_id = str(ticket["ticket_id"])
        if ticket_dir.name != ticket_id:
            raise ValidationError("runtime ticket directory name does not match ticket_id")
        event_path = ticket_dir / "ticket_events.jsonl"
        if not event_path.exists():
            raise ValidationError(f"runtime ticket {ticket_id} has no event log")
        _validate_existing_event_log(event_path, ticket_id)
        replay = RuntimeTicketEventLog(ticket_id)
        for line_number, line in enumerate(event_path.read_text(encoding="utf-8").splitlines(), 1):
            try:
                replay.apply(json.loads(line))
            except json.JSONDecodeError as error:
                raise ValidationError(
                    f"cannot parse runtime ticket {ticket_id} event {line_number}: {error}"
                ) from error
        if replay.state not in TERMINAL_TICKET_STATES:
            raise ValidationError(
                f"runtime ticket {ticket_id} is not terminal: {replay.state}"
            )


def validate_legacy_run(root: Path) -> None:
    _ensure_safe_directory(root, "legacy run root", create=False)
    run_spec_path = root / "run_spec.json"
    run_receipt_path = root / "run_receipt.json"
    intervention_path = root / "operator_intervention.json"
    exception_path = root / "legacy_ticket_exception.json"
    try:
        run_spec_bytes = _read_bounded_bytes_file(
            run_spec_path, "legacy run_spec", max_bytes=1024 * 1024
        )
        run_receipt_bytes = _read_bounded_bytes_file(
            run_receipt_path,
            "legacy run_receipt",
            max_bytes=1024 * 1024,
        )
        intervention_bytes = _read_bounded_bytes_file(
            intervention_path,
            "legacy operator_intervention",
            max_bytes=1024 * 1024,
        )
        exception = json.loads(
            _read_bounded_text_file(
                exception_path,
                "legacy exception",
                max_bytes=1024 * 1024,
            )
        )
        run_spec = json.loads(run_spec_bytes.decode("utf-8"))
        run_receipt = json.loads(run_receipt_bytes.decode("utf-8"))
        intervention = json.loads(intervention_bytes.decode("utf-8"))
    except json.JSONDecodeError as error:
        raise ValidationError(f"cannot parse legacy JSON: {error}") from error
    _validate_legacy_original_record(
        run_spec,
        LEGACY_RUN_SPEC_FIELDS,
        "legacy run_spec",
    )
    _validate_legacy_original_record(
        run_receipt,
        LEGACY_RUN_RECEIPT_FIELDS,
        "legacy run_receipt",
    )
    _validate_legacy_original_record(
        intervention,
        LEGACY_OPERATOR_INTERVENTION_FIELDS,
        "legacy operator_intervention",
    )
    _require_fields(exception, LEGACY_EXCEPTION_FIELDS, "legacy exception")
    _require_equal(
        exception["schema_version"],
        "crouzeix-legacy-ticket-exception/v1",
        "schema_version",
    )
    _enum(exception["classification"], LEGACY_CLASSIFICATIONS, "classification")
    run_id = _runtime_id(exception["run_id"], "run_id")
    _bounded_string(exception["arm"], "arm", 1, 128)
    if run_spec.get("run_id") != run_id:
        raise ValidationError("legacy exception run_id does not match run_spec")
    if run_receipt.get("run_id") != run_id:
        raise ValidationError("legacy exception run_id does not match run_receipt")
    if intervention.get("run_id") != run_id:
        raise ValidationError(
            "legacy exception run_id does not match operator_intervention"
        )
    if run_spec.get("arm") != exception["arm"] or run_receipt.get("arm") != exception["arm"]:
        raise ValidationError("legacy exception arm does not match original run")
    _require_digest_match(
        exception["original_run_spec_sha256"],
        run_spec_bytes,
        "original_run_spec_sha256",
    )
    _require_digest_match(
        exception["original_run_receipt_sha256"],
        run_receipt_bytes,
        "original_run_receipt_sha256",
    )
    _require_digest_match(
        exception["original_operator_intervention_sha256"],
        intervention_bytes,
        "original_operator_intervention_sha256",
    )
    call_manifest = legacy_call_manifest(root)
    _require_digest_match(
        exception["original_call_manifest_sha256"],
        _canonical_json_bytes(call_manifest),
        "original_call_manifest_sha256",
    )
    original_call_count = _bounded_integer(
        exception["original_call_count"],
        "original_call_count",
        1,
        128,
    )
    if call_manifest["call_count"] != original_call_count:
        raise ValidationError("legacy exception original_call_count mismatch")
    call_ids = {call["call_id"] for call in call_manifest["calls"]}
    if intervention["call_id"] not in call_ids:
        raise ValidationError("legacy operator_intervention call_id is not preserved")
    original_completed = _strict_timestamp(
        exception["original_completed_at_utc"],
        "original_completed_at_utc",
    )
    original_intervention = _strict_timestamp(
        exception["original_intervention_at_utc"],
        "original_intervention_at_utc",
    )
    migration_created = _strict_timestamp(
        exception["migration_created_at_utc"],
        "migration_created_at_utc",
    )
    if run_receipt.get("completed_at_utc") != original_completed:
        raise ValidationError("legacy exception completion timestamp mismatch")
    if intervention.get("occurred_at_utc") != original_intervention:
        raise ValidationError("legacy exception intervention timestamp mismatch")
    if _parse_utc_timestamp(migration_created) <= max(
        _parse_utc_timestamp(original_completed),
        _parse_utc_timestamp(original_intervention),
    ):
        raise ValidationError("legacy exception migration timestamp is backdated")
    migration_ticket_id = _runtime_id(
        exception["migration_ticket_id"],
        "migration_ticket_id",
    )
    migration_ticket = _read_migration_ticket(root.parent, migration_ticket_id)
    if migration_ticket["ticket_id"] != migration_ticket_id:
        raise ValidationError("legacy exception migration ticket_id mismatch")
    if migration_ticket["created_at_utc"] != migration_created:
        raise ValidationError("legacy exception migration timestamp mismatch")
    _require_digest_match(
        exception["migration_ticket_sha256"],
        _canonical_json_bytes(migration_ticket),
        "migration_ticket_sha256",
    )
    claim_boundary = _bounded_string(exception["claim_boundary"], "claim_boundary", 1, 512)
    if "not a retroactive runtime ticket" not in claim_boundary:
        raise ValidationError("legacy exception claim_boundary must reject retroactive tickets")


def validate_legacy_roots(roots: list[Path]) -> None:
    if not roots:
        raise ValidationError("validate-legacy requires at least one run root")
    for root in roots:
        validate_legacy_run(root)


def legacy_call_manifest(root: Path) -> dict[str, object]:
    calls_root = root / "calls"
    _ensure_safe_directory(calls_root, "legacy calls root", create=False)
    calls: list[dict[str, object]] = []
    for call_dir in sorted(path for path in calls_root.iterdir() if path.is_dir()):
        _reject_symlink(call_dir, "legacy call directory")
        files = []
        for path in sorted(call_dir.rglob("*")):
            _reject_symlink(path, "legacy call artifact")
            if path.is_dir():
                continue
            relative_path = path.relative_to(call_dir).as_posix()
            data = _read_bounded_bytes_file(
                path,
                "legacy call artifact",
                max_bytes=16 * 1024 * 1024,
            )
            files.append(
                {
                    "path": relative_path,
                    "sha256": sha256_bytes(data),
                }
            )
        if not files:
            raise ValidationError("legacy call directory is empty")
        calls.append({"call_id": call_dir.name, "files": files})
    if not calls:
        raise ValidationError("legacy calls root has no calls")
    return {
        "schema_version": "crouzeix-legacy-call-manifest/v1",
        "call_count": len(calls),
        "calls": calls,
    }


def _validate_dependencies(tracker: Tracker) -> None:
    for item in tracker.items.values():
        for dependency in _dependencies(item):
            if dependency not in tracker.items:
                raise ValidationError(f"{item.item_id} has unknown dependency {dependency}")
            if dependency == item.item_id:
                raise ValidationError(f"{item.item_id} has self dependency")
    visited: set[str] = set()
    active: set[str] = set()

    def visit(item_id: str) -> None:
        if item_id in active:
            raise ValidationError(f"dependency cycle involving {item_id}")
        if item_id in visited:
            return
        active.add(item_id)
        for dependency in _dependencies(tracker.items[item_id]):
            visit(dependency)
        active.remove(item_id)
        visited.add(item_id)

    for item_id in tracker.items:
        visit(item_id)


def _dependencies(item: TrackerItem) -> list[str]:
    raw = item.properties.get("DEPENDS_ON", "")
    if not raw:
        return []
    return [part for part in raw.split() if part]


def _require_fields(value: Mapping[str, Any], allowed: frozenset[str], label: str) -> None:
    if not isinstance(value, Mapping):
        raise ValidationError(f"{label} must be an object")
    fields = set(value)
    if fields != set(allowed):
        missing = sorted(set(allowed) - fields)
        extra = sorted(fields - set(allowed))
        detail = []
        if missing:
            detail.append(f"missing {', '.join(missing)}")
        if extra:
            detail.append(f"unknown {', '.join(extra)}")
        raise ValidationError(f"{label} fields are invalid: {'; '.join(detail)}")


def _validate_legacy_original_record(
    value: Any,
    fields: frozenset[str],
    label: str,
) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ValidationError(f"{label} must be an object")
    keys = set(value)
    if not keys.issubset(fields):
        extra = ", ".join(sorted(keys - fields))
        raise ValidationError(f"{label} fields are invalid: unknown {extra}")
    if "schema_version" not in value or "run_id" not in value:
        raise ValidationError(f"{label} fields are invalid: missing required fields")
    if label.endswith("run_spec"):
        _require_equal(value["schema_version"], "crouzeix-run-spec/v1", "schema_version")
    elif label.endswith("run_receipt"):
        _require_equal(value["schema_version"], "crouzeix-run-receipt/v1", "schema_version")
    elif label.endswith("operator_intervention"):
        _require_equal(
            value["schema_version"],
            "crouzeix-operator-intervention/v1",
            "schema_version",
        )
    _runtime_id(value["run_id"], "run_id")
    _bounded_string(value["arm"], "arm", 1, 128) if "arm" in value else None
    _strict_timestamp(value["created_at_utc"], "created_at_utc") if "created_at_utc" in value else None
    _strict_timestamp(value["completed_at_utc"], "completed_at_utc") if "completed_at_utc" in value else None
    _strict_timestamp(value["occurred_at_utc"], "occurred_at_utc") if "occurred_at_utc" in value else None
    _bounded_string(value["call_id"], "call_id", 1, 128) if "call_id" in value else None
    _bounded_string(value["action"], "action", 1, 128) if "action" in value else None
    _bounded_string(value["reason"], "reason", 1, 512) if "reason" in value else None
    _bounded_string(value["claim_boundary"], "claim_boundary", 1, 512) if "claim_boundary" in value else None
    return value


def _require_equal(value: Any, expected: str, label: str) -> None:
    if value != expected:
        raise ValidationError(f"{label} must be {expected}")


def _enum(value: Any, allowed: frozenset[str], label: str) -> str:
    if not isinstance(value, str) or value not in allowed:
        raise ValidationError(f"{label} is invalid")
    return value


def _runtime_id(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 128)
    if (
        text in {".", ".."}
        or text.startswith("-")
        or "/" in text
        or "\\" in text
        or any(char not in PORTABLE_RUNTIME_ID_CHARS for char in text)
    ):
        raise ValidationError(f"{label} is not a safe runtime identifier")
    return text


def _digest(value: Any, label: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(char not in "0123456789abcdef" for char in value)
    ):
        raise ValidationError(f"{label} must be a SHA-256 hex digest")
    return value


def _timestamp(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 64)
    if not text.endswith("Z") or "T" not in text:
        raise ValidationError(f"{label} must be an UTC timestamp")
    return text


def _bounded_integer(value: Any, label: str, minimum: int, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValidationError(f"{label} must be an integer")
    if not minimum <= value <= maximum:
        raise ValidationError(f"{label} must be between {minimum} and {maximum}")
    return value


def _bounded_string(value: Any, label: str, minimum: int, maximum: int) -> str:
    if not isinstance(value, str):
        raise ValidationError(f"{label} must be a string")
    if "\0" in value or not minimum <= len(value) <= maximum:
        raise ValidationError(f"{label} length is invalid")
    return value


def _string_list(
    value: Any,
    label: str,
    *,
    minimum: int,
    maximum: int,
) -> list[str]:
    if not isinstance(value, list) or not minimum <= len(value) <= maximum:
        raise ValidationError(f"{label} must be a bounded list")
    result = []
    for item in value:
        result.append(_bounded_string(item, label, 1, 4096))
    return result


def _reject_symlink(path: Path, label: str) -> None:
    try:
        mode = path.lstat().st_mode
    except FileNotFoundError:
        return
    if stat.S_ISLNK(mode):
        raise ValidationError(f"{label} cannot be a symlink")


def _ensure_safe_directory(path: Path, label: str, *, create: bool) -> None:
    if path.is_absolute():
        current = Path(path.anchor)
        parts = path.parts[1:]
    else:
        current = Path(".")
        parts = path.parts
    for part in parts:
        current = current / part
        try:
            mode = current.lstat().st_mode
        except FileNotFoundError:
            if create:
                current.mkdir(mode=0o700)
                mode = current.lstat().st_mode
            else:
                raise ValidationError(f"{label} does not exist: {current}")
        if stat.S_ISLNK(mode):
            raise ValidationError(f"{label} contains symlink: {current}")
        if not stat.S_ISDIR(mode):
            raise ValidationError(f"{label} must be a directory: {current}")


def _validate_existing_event_log(path: Path, ticket_id: str) -> None:
    if not path.exists():
        return
    try:
        mode = path.lstat().st_mode
    except OSError as error:
        raise ValidationError(f"cannot inspect runtime ticket event log: {error}") from error
    if stat.S_ISLNK(mode):
        raise ValidationError("runtime ticket event log cannot be a symlink")
    if not stat.S_ISREG(mode):
        raise ValidationError("runtime ticket event log must be a regular file")
    if path.stat().st_size > 16 * 1024 * 1024:
        raise ValidationError("runtime ticket event log exceeds byte cap")


def _read_bounded_text_file(path: Path, label: str, *, max_bytes: int) -> str:
    try:
        return _read_bounded_bytes_file(path, label, max_bytes=max_bytes).decode("utf-8")
    except UnicodeDecodeError as error:
        raise ValidationError(f"{label} must be UTF-8 text: {error}") from error


def _read_bounded_bytes_file(path: Path, label: str, *, max_bytes: int) -> bytes:
    _reject_symlink(path, label)
    try:
        mode = path.lstat().st_mode
    except FileNotFoundError as error:
        raise ValidationError(f"{label} does not exist: {path}") from error
    if not stat.S_ISREG(mode):
        raise ValidationError(f"{label} must be a regular file: {path}")
    size = path.stat().st_size
    if size > max_bytes:
        raise ValidationError(f"{label} exceeds byte cap: {path}")
    try:
        return path.read_bytes()
    except OSError as error:
        raise ValidationError(f"cannot read {label}: {error}") from error


def _require_digest_match(value: Any, data: bytes, label: str) -> None:
    expected = _digest(value, label)
    observed = sha256_bytes(data)
    if observed != expected:
        raise ValidationError(f"{label} does not match observed bytes")


def _canonical_json_bytes(value: Mapping[str, Any]) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")


def _read_migration_ticket(base_root: Path, ticket_id: str) -> dict[str, object]:
    migration_root = base_root / ticket_id
    tickets_root = migration_root / "tickets"
    ticket_dir = tickets_root / ticket_id
    _ensure_safe_directory(migration_root, "legacy migration root", create=False)
    _ensure_safe_directory(tickets_root, "legacy migration ticket root", create=False)
    _ensure_safe_directory(ticket_dir, "legacy migration ticket directory", create=False)
    ticket_path = ticket_dir / "ticket.json"
    try:
        raw = json.loads(
            _read_bounded_text_file(
                ticket_path,
                "legacy migration ticket",
                max_bytes=1024 * 1024,
            )
        )
    except json.JSONDecodeError as error:
        raise ValidationError(f"cannot parse legacy migration ticket: {error}") from error
    return validate_runtime_ticket(raw)


def _strict_timestamp(value: Any, label: str) -> str:
    text = _timestamp(value, label)
    _parse_utc_timestamp(text)
    return text


def _parse_utc_timestamp(value: str) -> dt.datetime:
    try:
        parsed = dt.datetime.fromisoformat(value.removesuffix("Z") + "+00:00")
    except ValueError as error:
        raise ValidationError(f"{value} is not a valid UTC timestamp") from error
    if parsed.tzinfo != dt.timezone.utc:
        raise ValidationError(f"{value} is not a UTC timestamp")
    return parsed


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Validate Crouzeix ticket records.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    validate_tracker_parser = subparsers.add_parser("validate-tracker")
    validate_tracker_parser.add_argument("tracker", type=Path)

    validate_runtime_parser = subparsers.add_parser("validate-runtime")
    validate_runtime_parser.add_argument("run_root", type=Path)

    validate_legacy_parser = subparsers.add_parser("validate-legacy")
    validate_legacy_parser.add_argument("run_roots", nargs="*", type=Path)

    args = parser.parse_args(argv)
    try:
        if args.command == "validate-tracker":
            validate_tracker(parse_tracker(args.tracker))
            return 0
        if args.command == "validate-runtime":
            validate_runtime_root(args.run_root)
            return 0
        if args.command == "validate-legacy":
            validate_legacy_roots(args.run_roots)
            return 0
    except ValidationError as error:
        print(f"tickets: {error}", file=sys.stderr)
        return 2
    raise AssertionError(f"unhandled command {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
