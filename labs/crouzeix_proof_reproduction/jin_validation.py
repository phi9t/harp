from __future__ import annotations

import json
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping

import formal_target
import protocol


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
