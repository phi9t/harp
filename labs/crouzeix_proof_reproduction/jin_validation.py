from __future__ import annotations

import json
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping

import formal_receipt
import formal_target
import protocol
import tickets


MAX_JSON_BYTES = 1024 * 1024
SOURCE_MAP_FIELDS = frozenset({"schema_version", "source_commit", "rows"})
SOURCE_MAP_ROW_FIELDS = frozenset(
    {
        "row_id",
        "source_locator",
        "statement_sha256",
        "lean_name",
        "dependency_ids",
        "status",
    }
)
SOURCE_MAP_STATUSES = frozenset({"mapped", "blocked"})


@dataclass(frozen=True)
class SourceMapRow:
    row_id: str
    source_locator: str
    statement_sha256: str
    lean_name: str
    dependency_ids: tuple[str, ...]
    status: str


@dataclass(frozen=True)
class AlignmentTask:
    task_id: str
    expected_declaration: str
    target_type_sha256: str
    imports: tuple[str, ...]
    source_map_row_ids: tuple[str, ...]


@dataclass(frozen=True)
class FakeLeanResult:
    exit_code: int | None
    stdout: bytes
    stderr: bytes
    axioms: list[str]
    blocked_reason: str | None = None


def load_source_map(
    path: Path, target: formal_target.FormalTargetLock
) -> tuple[SourceMapRow, ...]:
    value = _read_json_object(path, "Jin source map")
    _require_fields(value, SOURCE_MAP_FIELDS, "Jin source map")
    _require_equal(
        value["schema_version"], "crouzeix-jin-source-map/v1", "schema_version"
    )
    if value["source_commit"] != target.source.commit:
        raise protocol.ValidationError("Jin source map source_commit must match pinned target")
    rows = _row_list(value["rows"], target)
    row_ids = [row.row_id for row in rows]
    if len(set(row_ids)) != len(row_ids):
        raise protocol.ValidationError("duplicate Jin source-map row_id")
    dependencies = set(row_ids)
    for row in rows:
        for dependency_id in row.dependency_ids:
            if dependency_id not in dependencies:
                raise protocol.ValidationError("Jin source-map dependency is missing")
    terminal = [
        row
        for row in rows
        if row.lean_name == target.target.declaration_name
        and row.statement_sha256 == target.target.statement_sha256
    ]
    if len(terminal) != 1:
        raise protocol.ValidationError("Jin source map must contain terminal target theorem")
    return tuple(rows)


def make_alignment_task(
    target: formal_target.FormalTargetLock,
    rows: tuple[SourceMapRow, ...],
    target_lean: Path,
) -> AlignmentTask:
    imports = _read_imports(target_lean)
    unexpected = sorted(set(imports) - set(target.import_allowlist))
    if unexpected:
        raise protocol.ValidationError(
            f"Target.lean imports outside allowlist: {', '.join(unexpected)}"
        )
    if target.target.declaration_name not in {row.lean_name for row in rows}:
        raise protocol.ValidationError("alignment task requires terminal source-map row")
    return AlignmentTask(
        task_id="jin-target-alignment",
        expected_declaration=target.target.declaration_name,
        target_type_sha256=target.target.statement_sha256,
        imports=tuple(imports),
        source_map_row_ids=tuple(row.row_id for row in rows),
    )


def run_jin_validation(
    target: formal_target.ResolvedTarget,
    task: AlignmentTask,
    attempt_root: Path,
    *,
    fake_result: FakeLeanResult,
) -> Path:
    resolved = formal_target.resolve(_lock_from_resolved(target, task), target.root)
    if resolved.inventory_sha256 != target.inventory_sha256:
        raise protocol.ValidationError("runtime inventory mismatch")
    if fake_result.exit_code == 0 and fake_result.axioms:
        raise protocol.ValidationError("passed formal attempt has disallowed axioms")
    status = _status_from_fake_result(fake_result)
    reason = _reason_from_fake_result(fake_result, status)
    return _write_attempt(
        attempt_root,
        task=task,
        source_bytes=_target_source(task),
        stdout_bytes=fake_result.stdout,
        stderr_bytes=fake_result.stderr,
        axiom_log_bytes=_axiom_log(fake_result.axioms),
        status=status,
        reason=reason,
        command_exit_code=fake_result.exit_code,
        runtime_inventory_sha256=target.inventory_sha256,
        resource_preflight_status="blocked" if status == "blocked" else "ok",
    )


def record_blocked_preflight_attempt(
    target: formal_target.FormalTargetLock,
    task: AlignmentTask,
    attempt_root: Path,
    preflight: Mapping[str, object],
) -> Path:
    status = preflight.get("status")
    if status != "blocked":
        raise protocol.ValidationError("preflight receipt is not blocked")
    reason = str(preflight["reason"])
    target_sha256 = formal_target.protocol.sha256_bytes(
        formal_target._canonical_json_bytes(target.to_json())
    )
    return _write_attempt(
        attempt_root,
        task=task,
        source_bytes=_target_source(task),
        stdout_bytes=b"",
        stderr_bytes=reason.encode("utf-8") + b"\n",
        axiom_log_bytes=b"",
        status="blocked",
        reason=f"preflight blocked: {reason}",
        command_exit_code=None,
        runtime_inventory_sha256=target_sha256,
        resource_preflight_status="blocked",
    )


def _row_list(value: Any, target: formal_target.FormalTargetLock) -> list[SourceMapRow]:
    if not isinstance(value, list) or not value:
        raise protocol.ValidationError("Jin source-map rows must be a nonempty list")
    if len(value) > 1024:
        raise protocol.ValidationError("Jin source-map rows exceed cap")
    return [_source_map_row(_mapping(row, "Jin source-map row"), target) for row in value]


def _source_map_row(
    value: Mapping[str, Any], target: formal_target.FormalTargetLock
) -> SourceMapRow:
    _require_fields(value, SOURCE_MAP_ROW_FIELDS, "Jin source-map row")
    source_locator = _bounded_string(value["source_locator"], "source_locator", 1, 4096)
    if f"git:{target.source.commit}:" not in source_locator:
        raise protocol.ValidationError("source_locator must reference pinned Jin revision")
    dependencies = _string_list(value["dependency_ids"], "dependency_ids", maximum=256)
    return SourceMapRow(
        row_id=formal_target._runtime_id(value["row_id"], "row_id"),
        source_locator=source_locator,
        statement_sha256=formal_target._digest(value["statement_sha256"], "statement_sha256"),
        lean_name=_bounded_string(value["lean_name"], "lean_name", 1, 256),
        dependency_ids=tuple(dependencies),
        status=_enum(value["status"], SOURCE_MAP_STATUSES, "status"),
    )


def _read_imports(path: Path) -> tuple[str, ...]:
    _ensure_safe_file(path, "Target.lean")
    imports: list[str] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        stripped = raw.strip()
        if stripped.startswith("import "):
            imports.append(stripped.removeprefix("import ").strip())
    if not imports:
        raise protocol.ValidationError("Target.lean must contain imports")
    if len(set(imports)) != len(imports):
        raise protocol.ValidationError("Target.lean imports must be unique")
    return tuple(imports)


def _lock_from_resolved(
    target: formal_target.ResolvedTarget, task: AlignmentTask
) -> formal_target.FormalTargetLock:
    inventory = _read_json_object(target.root / "runtime-inventory.json", "runtime inventory")
    lock_value = {
        "schema_version": "crouzeix-formal-target-lock/v1",
        "source": inventory["source"],
        "toolchain": inventory["toolchain"],
        "command": inventory["command"],
        "target": {
            "target_id": "crouzeix-main",
            "declaration_name": task.expected_declaration,
            "statement_sha256": task.target_type_sha256,
            "source_locator": "Harp-authored",
            "dependency_ids": [],
        },
        "artifacts": inventory["artifacts"],
        "ledger_path": "ledger",
        "import_allowlist": list(task.imports),
    }
    return formal_target.FormalTargetLock.from_mapping(lock_value)


def _write_attempt(
    attempt_root: Path,
    *,
    task: AlignmentTask,
    source_bytes: bytes,
    stdout_bytes: bytes,
    stderr_bytes: bytes,
    axiom_log_bytes: bytes,
    status: str,
    reason: str,
    command_exit_code: int | None,
    runtime_inventory_sha256: str,
    resource_preflight_status: str,
) -> Path:
    resource = {
        "schema_version": "crouzeix-formal-resource-receipt/v1",
        "host_space_bytes": 8 * 1024 * 1024 * 1024,
        "memory_limit_bytes": 4 * 1024 * 1024 * 1024,
        "cpu_limit": 4,
        "preflight_status": resource_preflight_status,
        "checked_at_utc": "2026-08-15T12:00:00Z",
    }
    ticket = _formal_ticket()
    request = {
        "schema_version": "crouzeix-formal-attempt-receipt/v2",
        "attempt_id": "jin-target-alignment-attempt",
        "run_id": "jin-validation",
        "ticket_id": ticket["ticket_id"],
        "ticket_sha256": tickets.canonical_sha256(ticket),
        "candidate": {
            "outcome": "no_candidate",
            "candidate_id": None,
            "candidate_sha256": None,
            "candidate_bytes": 0,
        },
        "toolchain": {
            "name": "lean",
            "version": "4.28.0",
            "platform": "fixture",
            "toolchain_sha256": "1" * 64,
        },
        "source": {
            "path": "source/Target.lean",
            "sha256": protocol.sha256_bytes(source_bytes),
            "bytes": len(source_bytes),
        },
        "command": {
            "argv": ["lake", "build"],
            "cwd": ".",
            "env_sha256": "2" * 64,
            "exit_code": command_exit_code,
        },
        "logs": {
            "stdout_path": "logs/stdout.txt",
            "stdout_sha256": protocol.sha256_bytes(stdout_bytes),
            "stderr_path": "logs/stderr.txt",
            "stderr_sha256": protocol.sha256_bytes(stderr_bytes),
            "complete": True,
        },
        "resource_receipt": {
            "path": "resource_receipt.json",
            "sha256": formal_receipt.canonical_sha256(resource),
        },
        "axioms": {
            "scan_performed": status == "passed",
            "scanner": "jin-validation-fixture",
            "allowed_axioms": [],
            "observed_axioms": [],
            "scan_log_path": "logs/axioms.txt",
            "scan_log_sha256": protocol.sha256_bytes(axiom_log_bytes),
        },
        "status": status,
        "reason": reason,
        "started_at_utc": "2026-08-15T12:00:01Z",
        "completed_at_utc": "2026-08-15T12:00:02Z",
        "formal_target": {
            "path": "formal_target.lock.json",
            "sha256": "4" * 64,
        },
        "runtime_inventory_sha256": runtime_inventory_sha256,
        "target_type_sha256": task.target_type_sha256,
    }
    request["formal_attempt_sha256"] = formal_receipt.canonical_sha256_without_self(request)
    return formal_receipt.prepare_attempt(
        attempt_root,
        ticket=ticket,
        request=request,
        source_bytes=source_bytes,
        stdout_bytes=stdout_bytes,
        stderr_bytes=stderr_bytes,
        axiom_log_bytes=axiom_log_bytes,
        resource_receipt=resource,
    )


def _formal_ticket() -> dict[str, object]:
    return {
        "schema_version": "crouzeix-runtime-ticket/v1",
        "ticket_id": "jin-target-alignment",
        "run_id": "jin-validation",
        "task_kind": "formal_attempt",
        "node_id": "jin-target-alignment",
        "parent_node_id": None,
        "generation": 0,
        "direction_id": "jin-target-alignment",
        "role": "formal_verifier",
        "objective": "Validate the pinned Jin terminal theorem against FormalTarget.",
        "expected_deliverable": "v2 formal attempt receipt.",
        "dependency_ticket_ids": ["CPFR-074"],
        "context_sha256": "a" * 64,
        "schema_sha256": "b" * 64,
        "prompt_sha256": "c" * 64,
        "parent_artifact_sha256": None,
        "allowed_tools": ["Read"],
        "forbidden_sources": ["blind frontier private contexts"],
        "timeout_seconds": 3600,
        "max_output_bytes": 1048576,
        "owner_type": "orchestrator",
        "created_at_utc": "2026-08-15T12:00:00Z",
        "state": "created",
    }


def _target_source(task: AlignmentTask) -> bytes:
    return (
        f"import {task.imports[0]}\n\n#check {task.expected_declaration}\n"
    ).encode("utf-8")


def _status_from_fake_result(result: FakeLeanResult) -> str:
    if result.blocked_reason is not None or result.exit_code is None:
        return "blocked"
    if result.exit_code == 0:
        return "passed"
    return "failed"


def _reason_from_fake_result(result: FakeLeanResult, status: str) -> str:
    if status == "passed":
        return "fake Lean command succeeded and axiom scan reported no axioms"
    if status == "blocked":
        return result.blocked_reason or "fake Lean command blocked"
    return "fake Lean command failed"


def _axiom_log(axioms: list[str]) -> bytes:
    if not axioms:
        return b""
    return ("\n".join(axioms) + "\n").encode("utf-8")


def _read_json_object(path: Path, label: str) -> dict[str, Any]:
    _ensure_safe_file(path, label)
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if metadata.st_size > MAX_JSON_BYTES:
        raise protocol.ValidationError(f"{label} exceeds byte cap")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise protocol.ValidationError(f"cannot parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise protocol.ValidationError(f"{label} must contain one JSON object")
    return value


def _ensure_safe_file(path: Path, label: str) -> None:
    _ensure_directory(path.parent, f"{label} parent")
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(f"{label} cannot be a symlink")
    if not stat.S_ISREG(metadata.st_mode):
        raise protocol.ValidationError(f"{label} must be a regular file")


def _ensure_directory(path: Path, label: str) -> None:
    current = Path(path.anchor) if path.is_absolute() else Path(".")
    parts = path.parts[1:] if path.is_absolute() else path.parts
    for part in parts:
        current = current / part
        try:
            metadata = current.lstat()
        except OSError as error:
            raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
        if stat.S_ISLNK(metadata.st_mode):
            raise protocol.ValidationError(f"{label} contains symlink: {current}")
        if not stat.S_ISDIR(metadata.st_mode):
            raise protocol.ValidationError(f"{label} must be a directory: {current}")


def _require_fields(value: Mapping[str, Any], allowed: frozenset[str], label: str) -> None:
    fields = set(value)
    if fields != set(allowed):
        missing = sorted(set(allowed) - fields)
        extra = sorted(fields - set(allowed))
        detail = []
        if missing:
            detail.append(f"missing {', '.join(missing)}")
        if extra:
            detail.append(f"unknown {', '.join(extra)}")
        raise protocol.ValidationError(f"{label} fields are invalid: {'; '.join(detail)}")


def _mapping(value: Any, label: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise protocol.ValidationError(f"{label} must be an object")
    return value


def _require_equal(value: Any, expected: str, label: str) -> None:
    if value != expected:
        raise protocol.ValidationError(f"{label} must be {expected}")


def _enum(value: Any, allowed: frozenset[str], label: str) -> str:
    if not isinstance(value, str) or value not in allowed:
        raise protocol.ValidationError(f"{label} is invalid")
    return value


def _bounded_string(value: Any, label: str, minimum: int, maximum: int) -> str:
    if (
        not isinstance(value, str)
        or len(value) < minimum
        or len(value) > maximum
        or "\0" in value
    ):
        raise protocol.ValidationError(
            f"{label} must be a non-NUL string of length {minimum}..{maximum}"
        )
    return value


def _string_list(value: Any, label: str, *, maximum: int) -> list[str]:
    if not isinstance(value, list) or len(value) > maximum:
        raise protocol.ValidationError(f"{label} must be a bounded list")
    return [formal_target._runtime_id(item, label) for item in value]
