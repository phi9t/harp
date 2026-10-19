from __future__ import annotations

import hashlib
import json
import os
import shutil
import stat
from pathlib import Path
from typing import Any, Callable, Mapping

import expert_contracts
import frontier
import protocol


JSONL_BYTE_CAP = 16 * 1024 * 1024
JSON_RECORD_BYTE_CAP = 1024 * 1024
ATTEMPT_LEDGER_FIELDS = frozenset(
    {
        "schema_version",
        "sequence",
        "run_id",
        "attempt_id",
        "ticket_id",
        "proposed_node_id",
        "terminal_status",
        "attempt_dir_sha256",
        "occurred_at_utc",
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
SELECTION_EVENT_FIELDS = frozenset(
    {
        "schema_version",
        "sequence",
        "run_id",
        "event_kind",
        "ticket_id",
        "artifact_sha256",
        "occurred_at_utc",
        "generation",
        "draw_index",
        "selected_node_id",
        "archive_snapshot_sha256",
    }
)
ATTEMPT_TERMINAL_STATES = frozenset(
    {"completed", "failed", "malformed", "timed_out", "blocked_resource"}
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


def open_frontier_store(run_dir: Path) -> "FrontierStore":
    return FrontierStore(run_dir)


class FrontierStore:
    def __init__(self, run_dir: Path) -> None:
        _ensure_directory(Path(run_dir), "frontier run directory", create=True)
        self.run_dir = Path(run_dir).resolve()
        _ensure_directory(self.run_dir, "frontier run directory", create=True)

    def append_attempt(self, event: Mapping[str, Any]) -> dict[str, object]:
        return self._append_jsonl(
            self.run_dir / "attempt_ledger.jsonl",
            _validate_attempt_ledger_record(event),
            validator=_validate_attempt_ledger_record,
            unique_field="attempt_id",
        )

    def append_frontier_event(self, event: Mapping[str, Any]) -> dict[str, object]:
        return self._append_jsonl(
            self.run_dir / "frontier_events.jsonl",
            _validate_frontier_event(event),
            validator=_validate_frontier_event,
        )

    def append_selection_event(self, event: Mapping[str, Any]) -> dict[str, object]:
        value = _validate_selection_event(event)
        without_digest = dict(value)
        without_digest.pop("selection_event_sha256", None)
        value["selection_event_sha256"] = _canonical_sha256(without_digest)
        return self._append_jsonl(
            self.run_dir / "selection_events.jsonl",
            value,
            validator=_validate_selection_event,
            unique_key=lambda item: f"{item['generation']}:{item['draw_index']}",
        )

    def materialize_attempt(
        self,
        *,
        attempt_id: str,
        context: Mapping[str, Any],
        provider_call: Mapping[str, Any],
        receipt: Mapping[str, Any],
        expert_result: Mapping[str, Any] | None,
        terminal_failure: Mapping[str, Any] | None,
    ) -> Path:
        attempt_id = _portable_id(attempt_id, "attempt_id")
        if (expert_result is None) == (terminal_failure is None):
            raise protocol.ValidationError(
                "attempt must contain exactly one expert result or terminal failure"
            )
        attempts_root = self.run_dir / "attempts"
        _ensure_directory(attempts_root, "attempts directory", create=True)
        attempt_dir = attempts_root / attempt_id
        _create_directory_once(attempt_dir, "attempt directory")
        try:
            _write_json_create_only(
                attempt_dir / "attempt.json",
                {
                    "schema_version": "crouzeix-attempt-directory/v1",
                    "attempt_id": attempt_id,
                    "terminal_status": "completed"
                    if expert_result is not None
                    else _terminal_failure_status(terminal_failure),
                },
                "attempt record",
            )
            _write_json_create_only(attempt_dir / "context.json", context, "attempt context")
            provider_dir = attempt_dir / "provider_call"
            _create_directory_once(provider_dir, "provider call directory")
            _write_json_create_only(
                provider_dir / "call.json", provider_call, "provider call record"
            )
            _write_json_create_only(attempt_dir / "receipt.json", receipt, "attempt receipt")
            if expert_result is not None:
                _write_json_create_only(
                    attempt_dir / "expert_result.json",
                    expert_contracts.validate_expert_result(expert_result),
                    "expert result",
                )
            else:
                _write_json_create_only(
                    attempt_dir / "terminal_failure.json",
                    _validate_terminal_failure(terminal_failure),
                    "terminal failure",
                )
        except Exception:
            shutil.rmtree(attempt_dir, ignore_errors=True)
            raise
        return attempt_dir

    def record_admission(self, decision: Mapping[str, Any]) -> Path:
        value = _validate_admission_decision(decision)
        if "node_artifact_sha256" in value:
            raise protocol.ValidationError("admission must not reference node_artifact_sha256")
        admissions_root = self.run_dir / "admissions"
        _ensure_directory(admissions_root, "admissions directory", create=True)
        path = admissions_root / f"{value['attempt_id']}.json"
        _write_json_create_only(path, value, "admission decision")
        return path

    def materialize_mathematical_node(self, node: Mapping[str, Any]) -> Path:
        value = expert_contracts.validate_mathematical_node(node)
        self._require_single_accepted_admission(value)
        root = self.run_dir / "mathematical_nodes"
        _ensure_directory(root, "mathematical nodes directory", create=True)
        node_dir = root / str(value["node_id"])
        _create_directory_once(node_dir, "mathematical node directory")
        try:
            payload = value["mathematical_payload"]
            _write_json_create_only(node_dir / "node.json", value, "mathematical node")
            _write_json_create_only(
                node_dir / "mathematical_payload.json",
                payload,
                "mathematical payload",
            )
            inventory = {
                "schema_version": "crouzeix-mathematical-node-inventory/v1",
                "node_id": value["node_id"],
                "node_artifact_sha256": value["node_artifact_sha256"],
                "mathematical_payload_sha256": value["mathematical_payload_sha256"],
                "admission_decision_sha256": value["admission_decision_sha256"],
                "node_json_sha256": _file_sha256(node_dir / "node.json"),
                "mathematical_payload_json_sha256": _file_sha256(
                    node_dir / "mathematical_payload.json"
                ),
            }
            inventory["inventory_sha256"] = _canonical_sha256(inventory)
            _write_json_create_only(node_dir / "inventory.json", inventory, "node inventory")
        except Exception:
            shutil.rmtree(node_dir, ignore_errors=True)
            raise
        return node_dir

    def materialize_node_evaluation(self, evaluation: Mapping[str, Any]) -> Path:
        value = _validate_node_evaluation(evaluation)
        node_id = self._node_id_for_artifact(str(value["node_artifact_sha256"]))
        root = self.run_dir / "node_evaluations" / node_id
        _ensure_directory(root, "node evaluations directory", create=True)
        eval_dir = root / f"evaluator-{value['evaluator_index']}"
        _create_directory_once(eval_dir, "node evaluation directory")
        try:
            _write_json_create_only(eval_dir / "evaluation.json", value, "node evaluation")
            inventory = {
                "schema_version": "crouzeix-node-evaluation-inventory/v1",
                "node_id": node_id,
                "evaluator_index": value["evaluator_index"],
                "node_evaluation_sha256": value["node_evaluation_sha256"],
                "node_artifact_sha256": value["node_artifact_sha256"],
                "evaluation_json_sha256": _file_sha256(eval_dir / "evaluation.json"),
            }
            inventory["inventory_sha256"] = _canonical_sha256(inventory)
            _write_json_create_only(
                eval_dir / "inventory.json", inventory, "node evaluation inventory"
            )
        except Exception:
            shutil.rmtree(eval_dir, ignore_errors=True)
            raise
        return eval_dir

    def materialize_archive_entry(
        self,
        archive_entry: Mapping[str, Any],
        reconciliation: Mapping[str, Any],
    ) -> Path:
        entry = _validate_archive_entry(archive_entry)
        reconciliation_value = _validate_reconciliation(reconciliation)
        node = self._read_node(str(entry["node_id"]))
        rebuilt = frontier.build_archive_entry(node, reconciliation_value)
        if rebuilt != entry:
            raise protocol.ValidationError("archive entry does not match node and reconciliation")
        self._require_two_persisted_evaluations(reconciliation_value)
        eval_root = self.run_dir / "node_evaluations" / str(entry["node_id"])
        _write_json_create_only(
            eval_root / "reconciliation.json",
            reconciliation_value,
            "reconciliation",
        )
        archive_root = self.run_dir / "archive_entries"
        _ensure_directory(archive_root, "archive entries directory", create=True)
        path = archive_root / f"{entry['node_id']}.json"
        _write_json_create_only(path, entry, "archive entry")
        return path

    def project_snapshot(self) -> dict[str, object]:
        snapshot = self._build_snapshot()
        path = self.run_dir / "frontier_snapshot.json"
        if path.exists():
            existing = _read_json_object(path, "frontier snapshot")
            if existing != snapshot:
                raise protocol.ValidationError("frontier snapshot is stale")
        else:
            _write_json_create_only(path, snapshot, "frontier snapshot")
        return snapshot

    def freeze_candidate(
        self,
        *,
        projection_id: str,
        node: Mapping[str, Any],
        reconciliation: Mapping[str, Any],
        candidate_bytes: bytes,
    ) -> dict[str, object]:
        projection_id = _portable_id(projection_id, "projection_id")
        node_value = expert_contracts.validate_mathematical_node(node)
        reconciliation_value = _validate_reconciliation(reconciliation)
        if reconciliation_value["node_artifact_sha256"] != node_value["node_artifact_sha256"]:
            raise protocol.ValidationError("candidate reconciliation digest does not match node")
        entry = self._read_archive_entry(str(node_value["node_id"]))
        if entry["node_artifact_sha256"] != node_value["node_artifact_sha256"]:
            raise protocol.ValidationError("candidate node artifact digest does not match archive")
        if entry["reconciliation_sha256"] != reconciliation_value["reconciliation_sha256"]:
            raise protocol.ValidationError("candidate reconciliation digest does not match archive")
        candidate_sha256 = hashlib.sha256(candidate_bytes).hexdigest()
        if entry["candidate_proof_sha256"] != candidate_sha256:
            raise protocol.ValidationError("candidate proof digest does not match archive entry")

        root = self.run_dir / "candidate_projections"
        _ensure_directory(root, "candidate projections directory", create=True)
        candidate_path = root / f"{candidate_sha256}.tex"
        index_path = root / "index.json"
        if candidate_path.exists() or index_path.exists():
            raise protocol.ValidationError("candidate projection already exists")
        projection: dict[str, object] = {
            "schema_version": "crouzeix-candidate-projection/v1",
            "projection_id": projection_id,
            "source_node_id": node_value["node_id"],
            "source_node_artifact_sha256": node_value["node_artifact_sha256"],
            "source_reconciliation_sha256": reconciliation_value["reconciliation_sha256"],
            "candidate_sha256": candidate_sha256,
            "candidate_byte_count": len(candidate_bytes),
        }
        projection["candidate_projection_sha256"] = _canonical_sha256(projection)
        tmp_candidate = root / f"{projection_id}.tmp"
        tmp_index = root / f"{projection_id}.index.tmp"
        try:
            _write_bytes_create_only(tmp_candidate, candidate_bytes, "candidate bytes")
            _write_json_create_only(tmp_index, projection, "candidate projection index")
            _publish_create_only(tmp_candidate, candidate_path, "candidate bytes")
            try:
                _publish_create_only(tmp_index, index_path, "candidate projection index")
            except Exception:
                try:
                    candidate_path.unlink()
                except FileNotFoundError:
                    pass
                raise
        except Exception:
            for path in (tmp_candidate, tmp_index):
                try:
                    path.unlink()
                except FileNotFoundError:
                    pass
            raise
        return projection

    def reconcile_run(self, *, ticket_count: int) -> dict[str, object]:
        if not isinstance(ticket_count, int) or isinstance(ticket_count, bool) or ticket_count < 0:
            raise protocol.ValidationError("ticket_count must be a nonnegative integer")
        attempts = _read_jsonl(
            self.run_dir / "attempt_ledger.jsonl",
            "attempt ledger",
            _validate_attempt_ledger_record,
        )
        selections = _read_jsonl(
            self.run_dir / "selection_events.jsonl",
            "selection ledger",
            _validate_selection_event,
        )
        self._validate_run_consistency(attempts, ticket_count)
        terminal_states: dict[str, int] = {}
        for event in attempts:
            state = str(event["terminal_status"])
            terminal_states[state] = terminal_states.get(state, 0) + 1
        receipt = {
            "schema_version": "crouzeix-run-receipt/v1",
            "ticket_count": ticket_count,
            "attempt_count": len(attempts),
            "provider_call_count": _count_provider_calls(self.run_dir / "attempts"),
            "selection_count": len(selections),
            "mathematical_node_count": len(self._read_nodes()),
            "archive_entry_count": len(self._read_archive_entries()),
            "terminal_attempt_states": dict(sorted(terminal_states.items())),
        }
        receipt["run_receipt_sha256"] = _canonical_sha256(receipt)
        _write_json_replace(self.run_dir / "run_receipt.json", receipt, "run receipt")
        return receipt

    def _append_jsonl(
        self,
        path: Path,
        value: dict[str, object],
        *,
        validator: Callable[[Mapping[str, Any]], dict[str, object]],
        unique_field: str | None = None,
        unique_key: Callable[[Mapping[str, object]], str] | None = None,
    ) -> dict[str, object]:
        _reject_symlink(path, path.name)
        _validate_existing_jsonl(path, path.name)
        previous = _read_jsonl(path, path.name, validator)
        expected_sequence = len(previous) + 1
        if value["sequence"] != expected_sequence:
            raise protocol.ValidationError(
                f"{path.name} sequence must be {expected_sequence}"
            )
        if unique_field is not None:
            seen = {str(item[unique_field]) for item in previous}
            if str(value[unique_field]) in seen:
                raise protocol.ValidationError(f"duplicate {unique_field}")
        if unique_key is not None:
            seen_keys = {unique_key(item) for item in previous}
            if unique_key(value) in seen_keys:
                raise protocol.ValidationError("duplicate ledger key")
        encoded = _canonical_json_bytes(value)
        if len(encoded) > JSON_RECORD_BYTE_CAP:
            raise protocol.ValidationError("JSONL record exceeds byte cap")
        _ensure_directory(path.parent, f"{path.name} parent", create=True)
        fd = os.open(path, os.O_APPEND | os.O_CREAT | os.O_WRONLY, 0o600)
        try:
            os.write(fd, encoded + b"\n")
        finally:
            os.close(fd)
        return value

    def _validate_run_consistency(
        self,
        attempts: list[dict[str, object]],
        ticket_count: int,
    ) -> None:
        attempt_by_id = {str(item["attempt_id"]): item for item in attempts}
        if len(attempt_by_id) != len(attempts):
            raise protocol.ValidationError("attempt ledger contains duplicate attempt IDs")
        ticket_ids = {str(item["ticket_id"]) for item in attempts}
        selection_ticket_ids = {
            str(item["ticket_id"])
            for item in _read_jsonl(
                self.run_dir / "selection_events.jsonl",
                "selection ledger",
                _validate_selection_event,
            )
        }
        if ticket_count != len(ticket_ids | selection_ticket_ids):
            raise protocol.ValidationError("ticket_count is inconsistent with run evidence")

        attempt_dirs = self._read_attempt_directories()
        for attempt_id, ledger in attempt_by_id.items():
            directory = attempt_dirs.get(attempt_id)
            if directory is None:
                raise protocol.ValidationError(
                    f"attempt directory missing for ledger attempt {attempt_id}"
                )
            if not directory["has_provider_call"]:
                raise protocol.ValidationError(
                    f"provider call missing for ledger attempt {attempt_id}"
                )
            if not directory["has_provider_call_receipt"]:
                raise protocol.ValidationError(
                    f"provider call receipt missing for ledger attempt {attempt_id}"
                )
            if directory["terminal_status"] != ledger["terminal_status"]:
                raise protocol.ValidationError(
                    f"terminal state mismatch for attempt {attempt_id}"
                )
        for attempt_id, directory in attempt_dirs.items():
            has_provider_call = bool(directory["has_provider_call"])
            if has_provider_call and attempt_id not in attempt_by_id:
                raise protocol.ValidationError(
                    f"provider call for {attempt_id} has no ledger attempt"
                )

        self._validate_admission_node_links()

    def _require_single_accepted_admission(self, node: Mapping[str, Any]) -> None:
        admissions = _read_admissions(self.run_dir / "admissions")
        matching = [
            item
            for item in admissions
            if item["admission_decision_sha256"] == node["admission_decision_sha256"]
        ]
        if len(matching) != 1 or matching[0]["outcome"] != "accepted":
            raise protocol.ValidationError(
                "mathematical node must reference exactly one accepted admission"
            )
        existing_nodes = self._read_nodes()
        for existing in existing_nodes:
            if existing["admission_decision_sha256"] == node["admission_decision_sha256"]:
                raise protocol.ValidationError(
                    "accepted admission is already referenced by exactly one mathematical node"
                )

    def _node_id_for_artifact(self, node_artifact_sha256: str) -> str:
        for node in self._read_nodes():
            if node["node_artifact_sha256"] == node_artifact_sha256:
                return str(node["node_id"])
        raise protocol.ValidationError("node evaluation references unknown node artifact")

    def _read_node(self, node_id: str) -> dict[str, object]:
        path = self.run_dir / "mathematical_nodes" / node_id / "node.json"
        return expert_contracts.validate_mathematical_node(
            _read_json_object(path, "mathematical node")
        )

    def _read_nodes(self) -> list[dict[str, object]]:
        root = self.run_dir / "mathematical_nodes"
        if not root.exists():
            return []
        _ensure_directory(root, "mathematical nodes directory", create=False)
        nodes = [
            expert_contracts.validate_mathematical_node(
                _read_json_object(path / "node.json", "mathematical node")
            )
            for path in sorted(root.iterdir())
            if path.is_dir()
        ]
        _reject_duplicate(nodes, "node_id", "mathematical nodes")
        _reject_duplicate(nodes, "node_artifact_sha256", "mathematical nodes")
        return nodes

    def _read_attempt_directories(self) -> dict[str, dict[str, object]]:
        root = self.run_dir / "attempts"
        if not root.exists():
            return {}
        _ensure_directory(root, "attempts directory", create=False)
        attempts: dict[str, dict[str, object]] = {}
        for path in sorted(root.iterdir()):
            if not path.is_dir():
                continue
            record = _validate_attempt_directory_record(
                _read_json_object(path / "attempt.json", "attempt record")
            )
            attempt_id = str(record["attempt_id"])
            if attempt_id in attempts:
                raise protocol.ValidationError("duplicate attempt directory")
            if path.name != attempt_id:
                raise protocol.ValidationError("attempt directory name does not match record")
            attempts[attempt_id] = {
                "attempt_id": attempt_id,
                "terminal_status": record["terminal_status"],
                "has_provider_call": (path / "provider_call" / "call.json").is_file(),
                "has_provider_call_receipt": (
                    path / "provider_call" / "receipt.json"
                ).is_file()
                or (path / "receipt.json").is_file(),
            }
        return attempts

    def _validate_admission_node_links(self) -> None:
        admissions = _read_admissions(self.run_dir / "admissions")
        nodes = self._read_nodes()
        accepted_admission_digests = {
            str(item["admission_decision_sha256"])
            for item in admissions
            if item["outcome"] == "accepted"
        }
        by_admission: dict[str, int] = {}
        for node in nodes:
            digest = str(node["admission_decision_sha256"])
            if digest not in accepted_admission_digests:
                raise protocol.ValidationError(
                    "mathematical node must reference exactly one accepted admission"
                )
            by_admission[digest] = by_admission.get(digest, 0) + 1
        for admission in admissions:
            if admission["outcome"] != "accepted":
                continue
            count = by_admission.get(str(admission["admission_decision_sha256"]), 0)
            if count != 1:
                raise protocol.ValidationError(
                    "accepted admission must be referenced by exactly one mathematical node"
                )

    def _require_two_persisted_evaluations(self, reconciliation: Mapping[str, Any]) -> None:
        node_id = self._node_id_for_artifact(str(reconciliation["node_artifact_sha256"]))
        root = self.run_dir / "node_evaluations" / node_id
        observed = []
        for index in (1, 2):
            path = root / f"evaluator-{index}" / "evaluation.json"
            evaluation = _validate_node_evaluation(_read_json_object(path, "node evaluation"))
            observed.append(evaluation["node_evaluation_sha256"])
        if observed != list(reconciliation["node_evaluation_sha256s"]):
            raise protocol.ValidationError("reconciliation does not match persisted evaluations")

    def _read_archive_entry(self, node_id: str) -> dict[str, object]:
        return _validate_archive_entry(
            _read_json_object(
                self.run_dir / "archive_entries" / f"{node_id}.json",
                "archive entry",
            )
        )

    def _read_archive_entries(self) -> list[dict[str, object]]:
        root = self.run_dir / "archive_entries"
        if not root.exists():
            return []
        _ensure_directory(root, "archive entries directory", create=False)
        entries = [
            _validate_archive_entry(_read_json_object(path, "archive entry"))
            for path in sorted(root.glob("*.json"))
        ]
        _reject_duplicate(entries, "node_id", "archive entries")
        _reject_duplicate(entries, "archive_entry_sha256", "archive entries")
        return entries

    def _build_snapshot(self) -> dict[str, object]:
        nodes = self._read_nodes()
        entries = self._read_archive_entries()
        node_by_id = {str(node["node_id"]): node for node in nodes}
        for entry in entries:
            node = node_by_id.get(str(entry["node_id"]))
            if node is None:
                raise protocol.ValidationError("archive entry references missing node")
            if node["node_artifact_sha256"] != entry["node_artifact_sha256"]:
                raise protocol.ValidationError("archive entry node digest is stale")
        child_counts = {str(entry["node_id"]): 0 for entry in entries}
        artifact_to_node_id = {
            str(entry["node_artifact_sha256"]): str(entry["node_id"]) for entry in entries
        }
        for entry in entries:
            parent_digest = node_by_id[str(entry["node_id"])]["parent_node_artifact_sha256"]
            if parent_digest in artifact_to_node_id:
                child_counts[artifact_to_node_id[str(parent_digest)]] += 1
        selection_events = _read_jsonl(
            self.run_dir / "selection_events.jsonl",
            "selection ledger",
            _validate_selection_event,
        )
        frontier_events = _read_jsonl(
            self.run_dir / "frontier_events.jsonl",
            "frontier event ledger",
            _validate_frontier_event,
        )
        snapshot: dict[str, object] = {
            "schema_version": "crouzeix-frontier-store-snapshot/v1",
            "archive_entry_count": len(entries),
            "nodes_by_id": {
                str(node["node_id"]): {
                    "node_artifact_sha256": node["node_artifact_sha256"],
                    "parent_node_id": node["parent_node_id"],
                    "parent_node_artifact_sha256": node["parent_node_artifact_sha256"],
                    "generation": node["generation"],
                }
                for node in sorted(nodes, key=lambda item: str(item["node_id"]))
            },
            "archive_entries_by_node_id": {
                str(entry["node_id"]): {
                    "archive_entry_sha256": entry["archive_entry_sha256"],
                    "reconciliation_sha256": entry["reconciliation_sha256"],
                    "unanimous_pass_count": entry["unanimous_pass_count"],
                    "candidate_proof_sha256": entry["candidate_proof_sha256"],
                }
                for entry in sorted(entries, key=lambda item: str(item["node_id"]))
            },
            "functioning_child_counts_by_node_id": dict(sorted(child_counts.items())),
            "frontier_events": frontier_events,
            "selection_events": selection_events,
        }
        snapshot["frontier_snapshot_sha256"] = _canonical_sha256(snapshot)
        return snapshot


def _validate_attempt_ledger_record(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, ATTEMPT_LEDGER_FIELDS, "attempt ledger record")
    _require_equal(value["schema_version"], "crouzeix-attempt-ledger/v1", "schema_version")
    return {
        "schema_version": value["schema_version"],
        "sequence": _integer(value["sequence"], "sequence", 1, 1_000_000),
        "run_id": _portable_id(value["run_id"], "run_id"),
        "attempt_id": _portable_id(value["attempt_id"], "attempt_id"),
        "ticket_id": _runtime_id(value["ticket_id"], "ticket_id"),
        "proposed_node_id": _portable_id(value["proposed_node_id"], "proposed_node_id"),
        "terminal_status": _enum(
            value["terminal_status"], ATTEMPT_TERMINAL_STATES, "terminal_status"
        ),
        "attempt_dir_sha256": _digest(value["attempt_dir_sha256"], "attempt_dir_sha256"),
        "occurred_at_utc": _timestamp(value["occurred_at_utc"], "occurred_at_utc"),
    }


def _validate_frontier_event(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, FRONTIER_EVENT_FIELDS, "frontier event")
    _require_equal(value["schema_version"], "crouzeix-frontier-event/v1", "schema_version")
    return {
        "schema_version": value["schema_version"],
        "sequence": _integer(value["sequence"], "sequence", 1, 1_000_000),
        "run_id": _portable_id(value["run_id"], "run_id"),
        "event_kind": _enum(value["event_kind"], FRONTIER_EVENT_KINDS, "event_kind"),
        "ticket_id": _runtime_id(value["ticket_id"], "ticket_id"),
        "artifact_sha256": _digest(value["artifact_sha256"], "artifact_sha256"),
        "occurred_at_utc": _timestamp(value["occurred_at_utc"], "occurred_at_utc"),
    }


def _validate_selection_event(value: Mapping[str, Any]) -> dict[str, object]:
    fields = set(SELECTION_EVENT_FIELDS)
    if "selection_event_sha256" in value:
        fields.add("selection_event_sha256")
    _require_fields(value, frozenset(fields), "selection event")
    _require_equal(
        value["schema_version"], "crouzeix-selection-event/v1", "schema_version"
    )
    result = {
        "schema_version": value["schema_version"],
        "sequence": _integer(value["sequence"], "sequence", 1, 1_000_000),
        "run_id": _portable_id(value["run_id"], "run_id"),
        "event_kind": _enum(value["event_kind"], frozenset({"selection_recorded"}), "event_kind"),
        "ticket_id": _runtime_id(value["ticket_id"], "ticket_id"),
        "artifact_sha256": _digest(value["artifact_sha256"], "artifact_sha256"),
        "occurred_at_utc": _timestamp(value["occurred_at_utc"], "occurred_at_utc"),
        "generation": _integer(value["generation"], "generation", 1, 1_000_000),
        "draw_index": _integer(value["draw_index"], "draw_index", 0, 1),
        "selected_node_id": _portable_id(value["selected_node_id"], "selected_node_id"),
        "archive_snapshot_sha256": _digest(
            value["archive_snapshot_sha256"], "archive_snapshot_sha256"
        ),
    }
    if "selection_event_sha256" in value:
        result["selection_event_sha256"] = _digest(
            value["selection_event_sha256"], "selection_event_sha256"
        )
    return result


def _validate_admission_decision(value: Mapping[str, Any]) -> dict[str, object]:
    return expert_contracts._validate_admission_decision(value)


def _validate_node_evaluation(value: Mapping[str, Any]) -> dict[str, object]:
    return frontier._validate_evaluation(value)


def _validate_reconciliation(value: Mapping[str, Any]) -> dict[str, object]:
    return frontier._validate_reconciliation(value)


def _validate_archive_entry(value: Mapping[str, Any]) -> dict[str, object]:
    return frontier._validate_archive_entry(value)


def _validate_terminal_failure(value: Mapping[str, Any] | None) -> dict[str, object]:
    if value is None:
        raise protocol.ValidationError("terminal failure is required")
    _require_fields(value, frozenset({"terminal_status", "reason"}), "terminal failure")
    return {
        "terminal_status": _enum(
            value["terminal_status"], ATTEMPT_TERMINAL_STATES - {"completed"}, "terminal_status"
        ),
        "reason": _bounded_string(value["reason"], "reason", 1, 4096),
    }


def _terminal_failure_status(value: Mapping[str, Any] | None) -> str:
    return str(_validate_terminal_failure(value)["terminal_status"])


def _validate_attempt_directory_record(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(
        value,
        frozenset({"schema_version", "attempt_id", "terminal_status"}),
        "attempt record",
    )
    _require_equal(
        value["schema_version"], "crouzeix-attempt-directory/v1", "schema_version"
    )
    return {
        "schema_version": value["schema_version"],
        "attempt_id": _portable_id(value["attempt_id"], "attempt_id"),
        "terminal_status": _enum(
            value["terminal_status"], ATTEMPT_TERMINAL_STATES, "terminal_status"
        ),
    }


def _read_admissions(root: Path) -> list[dict[str, object]]:
    if not root.exists():
        return []
    _ensure_directory(root, "admissions directory", create=False)
    return [
        _validate_admission_decision(_read_json_object(path, "admission decision"))
        for path in sorted(root.glob("*.json"))
    ]


def _count_provider_calls(root: Path) -> int:
    if not root.exists():
        return 0
    _ensure_directory(root, "attempts directory", create=False)
    count = 0
    for attempt_dir in root.iterdir():
        if (attempt_dir / "provider_call" / "call.json").is_file():
            count += 1
    return count


def _write_json_create_only(path: Path, value: Mapping[str, Any], label: str) -> None:
    _write_bytes_create_only(path, _canonical_json_bytes(value) + b"\n", label)


def _write_json_replace(path: Path, value: Mapping[str, Any], label: str) -> None:
    _reject_symlink(path, label)
    _ensure_directory(path.parent, f"{label} parent", create=True)
    tmp = path.with_name(f".{path.name}.tmp")
    _reject_symlink(tmp, label)
    tmp.write_bytes(_canonical_json_bytes(value) + b"\n")
    os.replace(tmp, path)


def _publish_create_only(source: Path, destination: Path, label: str) -> None:
    _reject_symlink(source, label)
    _reject_symlink(destination, label)
    try:
        os.link(source, destination)
    except FileExistsError as error:
        raise protocol.ValidationError(f"{label} already exists: {destination}") from error
    except OSError as error:
        raise protocol.ValidationError(f"cannot publish {label}: {error}") from error
    try:
        source.unlink()
    except FileNotFoundError:
        pass


def _write_bytes_create_only(path: Path, data: bytes, label: str) -> None:
    if len(data) > JSON_RECORD_BYTE_CAP:
        raise protocol.ValidationError(f"{label} exceeds byte cap")
    _reject_symlink(path, label)
    _ensure_directory(path.parent, f"{label} parent", create=True)
    try:
        with path.open("xb") as handle:
            handle.write(data)
    except FileExistsError as error:
        raise protocol.ValidationError(f"{label} already exists: {path}") from error


def _read_json_object(path: Path, label: str) -> dict[str, Any]:
    _reject_symlink(path, label)
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if not stat.S_ISREG(metadata.st_mode):
        raise protocol.ValidationError(f"{label} must be a regular file")
    if metadata.st_size > JSON_RECORD_BYTE_CAP:
        raise protocol.ValidationError(f"{label} exceeds byte cap")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise protocol.ValidationError(f"cannot parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise protocol.ValidationError(f"{label} must contain one JSON object")
    return value


def _read_jsonl(
    path: Path,
    label: str,
    validator: Callable[[Mapping[str, Any]], dict[str, object]],
) -> list[dict[str, object]]:
    _reject_symlink(path, label)
    if not path.exists():
        return []
    _validate_existing_jsonl(path, label)
    items = []
    for index, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        try:
            value = json.loads(line)
        except json.JSONDecodeError as error:
            raise protocol.ValidationError(f"cannot parse {label}: {error}") from error
        if not isinstance(value, dict):
            raise protocol.ValidationError(f"{label} line {index} must be an object")
        item = validator(value)
        if item.get("sequence") != index:
            raise protocol.ValidationError(f"{label} sequence must be contiguous")
        items.append(item)
    return items


def _validate_existing_jsonl(path: Path, label: str) -> None:
    if not path.exists():
        return
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(f"{label} cannot be a symlink")
    if not stat.S_ISREG(metadata.st_mode):
        raise protocol.ValidationError(f"{label} must be a regular file")
    if metadata.st_size > JSONL_BYTE_CAP:
        raise protocol.ValidationError(f"{label} exceeds byte cap")


def _ensure_directory(path: Path, label: str, *, create: bool) -> None:
    current = Path(path.anchor) if path.is_absolute() else Path(".")
    parts = path.parts[1:] if path.is_absolute() else path.parts
    for part in parts:
        current = current / part
        try:
            metadata = current.lstat()
        except FileNotFoundError:
            if not create:
                raise protocol.ValidationError(f"{label} does not exist: {current}")
            current.mkdir(mode=0o700)
            metadata = current.lstat()
        if stat.S_ISLNK(metadata.st_mode):
            raise protocol.ValidationError(f"{label} contains symlink: {current}")
        if not stat.S_ISDIR(metadata.st_mode):
            raise protocol.ValidationError(f"{label} must be a directory: {current}")


def _create_directory_once(path: Path, label: str) -> None:
    _reject_symlink(path, label)
    try:
        path.mkdir(mode=0o700)
    except FileExistsError as error:
        raise protocol.ValidationError(f"{label} already exists: {path}") from error


def _reject_symlink(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        return
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(f"{label} cannot be a symlink")


def _file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _canonical_sha256(value: object) -> str:
    return hashlib.sha256(_canonical_json_bytes(value)).hexdigest()


def _canonical_json_bytes(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")


def _require_fields(value: Mapping[str, Any], expected: frozenset[str], label: str) -> None:
    if not isinstance(value, Mapping):
        raise protocol.ValidationError(f"{label} must be an object")
    actual = set(value)
    if actual != set(expected):
        raise protocol.ValidationError(
            f"{label} fields invalid; missing={sorted(set(expected) - actual)} "
            f"extra={sorted(actual - set(expected))}"
        )


def _require_equal(value: Any, expected: str, label: str) -> str:
    if value != expected:
        raise protocol.ValidationError(f"{label} must be {expected}")
    return expected


def _portable_id(value: Any, label: str) -> str:
    if not isinstance(value, str) or frontier.PORTABLE_ID.fullmatch(value) is None:
        raise protocol.ValidationError(f"{label} must be a portable identifier")
    return value


def _runtime_id(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value or len(value) > 128:
        raise protocol.ValidationError(f"{label} must be a runtime identifier")
    if value in {".", ".."} or "/" in value or "\\" in value or "\0" in value:
        raise protocol.ValidationError(f"{label} must be a runtime identifier")
    return value


def _digest(value: Any, label: str) -> str:
    if not isinstance(value, str) or frontier.SHA256.fullmatch(value) is None:
        raise protocol.ValidationError(f"{label} must be a SHA-256 digest")
    return value


def _integer(value: Any, label: str, minimum: int, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not minimum <= value <= maximum:
        raise protocol.ValidationError(f"{label} must be an integer in range")
    return value


def _enum(value: Any, allowed: frozenset[str], label: str) -> str:
    if not isinstance(value, str) or value not in allowed:
        raise protocol.ValidationError(f"{label} must be one of {sorted(allowed)}")
    return value


def _bounded_string(value: Any, label: str, minimum: int, maximum: int) -> str:
    if not isinstance(value, str) or "\0" in value or not minimum <= len(value) <= maximum:
        raise protocol.ValidationError(f"{label} must be a bounded non-NUL string")
    return value


def _timestamp(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 20, 64)
    if not text.endswith("Z") or "T" not in text:
        raise protocol.ValidationError(f"{label} must be a UTC timestamp")
    return text


def _reject_duplicate(values: list[Mapping[str, object]], field: str, label: str) -> None:
    seen = set()
    for value in values:
        item = value[field]
        if item in seen:
            raise protocol.ValidationError(f"duplicate {field} in {label}")
        seen.add(item)
