from __future__ import annotations

import json
import stat
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping

import formal_target
import protocol


MAX_JSON_BYTES = 1024 * 1024
GRAPH_FIELDS = frozenset({"schema_version", "source_id", "source_identity", "nodes"})
NODE_FIELDS = frozenset(
    {
        "node_id",
        "source_locator",
        "statement_sha256",
        "lean_name",
        "dependencies",
        "role",
        "status",
    }
)
INVENTORY_FIELDS = frozenset({"schema_version", "source_identity", "facts"})
FACT_FIELDS = frozenset({"fact_id", "statement_sha256", "source_locator", "resolution"})
NODE_ROLES = frozenset({"definition", "adapter", "library_fact", "intermediate", "terminal"})
NODE_STATUSES = frozenset({"mapped", "blocked"})
FACT_RESOLUTIONS = frozenset({"mathlib_available", "local_task", "blocked"})
LS_SOURCE_IDENTITY = "arxiv:2608.03841v1"


@dataclass(frozen=True)
class LSGraphRow:
    node_id: str
    source_locator: str
    statement_sha256: str
    lean_name: str
    dependencies: tuple[str, ...]
    role: str
    status: str


def load_source_graph(path: Path) -> tuple[LSGraphRow, ...]:
    value = _read_json_object(path, "LS source graph")
    _require_fields(value, GRAPH_FIELDS, "LS source graph")
    _require_equal(value["schema_version"], "crouzeix-ls-source-graph/v1", "schema_version")
    _require_equal(value["source_id"], "LS-ARXIV-V1", "source_id")
    _require_equal(value["source_identity"], LS_SOURCE_IDENTITY, "source_identity")
    rows = _node_list(value["nodes"])
    ids = [row.node_id for row in rows]
    if len(set(ids)) != len(ids):
        raise protocol.ValidationError("duplicate LS graph node_id")
    id_set = set(ids)
    for row in rows:
        for dependency in row.dependencies:
            if dependency not in id_set:
                raise protocol.ValidationError("LS graph dependency is missing")
    _reject_cycles(rows)
    terminal = [row for row in rows if row.role == "terminal"]
    if len(terminal) != 1:
        raise protocol.ValidationError("LS graph must contain exactly one terminal node")
    return tuple(rows)


def load_library_inventory(path: Path) -> dict[str, object]:
    value = _read_json_object(path, "LS library inventory")
    _require_fields(value, INVENTORY_FIELDS, "LS library inventory")
    _require_equal(
        value["schema_version"], "crouzeix-ls-library-inventory/v1", "schema_version"
    )
    _require_equal(value["source_identity"], LS_SOURCE_IDENTITY, "source_identity")
    facts = value["facts"]
    if not isinstance(facts, list) or not facts:
        raise protocol.ValidationError("LS library facts must be a nonempty list")
    normalized = [_fact(_mapping(fact, "LS library fact")) for fact in facts]
    ids = [fact["fact_id"] for fact in normalized]
    if len(set(ids)) != len(ids):
        raise protocol.ValidationError("duplicate LS library fact_id")
    return {
        "schema_version": "crouzeix-ls-library-inventory/v1",
        "source_identity": LS_SOURCE_IDENTITY,
        "facts": normalized,
    }


def _node_list(value: Any) -> list[LSGraphRow]:
    if not isinstance(value, list) or not value:
        raise protocol.ValidationError("LS graph nodes must be a nonempty list")
    if len(value) > 1024:
        raise protocol.ValidationError("LS graph nodes exceed cap")
    return [_node(_mapping(item, "LS graph node")) for item in value]


def _node(value: Mapping[str, Any]) -> LSGraphRow:
    _require_fields(value, NODE_FIELDS, "LS graph node")
    locator = _source_locator(value["source_locator"], "source_locator")
    return LSGraphRow(
        node_id=formal_target._runtime_id(value["node_id"], "node_id"),
        source_locator=locator,
        statement_sha256=formal_target._digest(value["statement_sha256"], "statement_sha256"),
        lean_name=_bounded_string(value["lean_name"], "lean_name", 1, 256),
        dependencies=tuple(_runtime_id_list(value["dependencies"], "dependencies")),
        role=_enum(value["role"], NODE_ROLES, "role"),
        status=_enum(value["status"], NODE_STATUSES, "status"),
    )


def _fact(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, FACT_FIELDS, "LS library fact")
    return {
        "fact_id": formal_target._runtime_id(value["fact_id"], "fact_id"),
        "statement_sha256": formal_target._digest(value["statement_sha256"], "statement_sha256"),
        "source_locator": _source_locator(value["source_locator"], "source_locator"),
        "resolution": _enum(value["resolution"], FACT_RESOLUTIONS, "resolution"),
    }


def _reject_cycles(rows: list[LSGraphRow]) -> None:
    by_id = {row.node_id: row for row in rows}
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(node_id: str) -> None:
        if node_id in visiting:
            raise protocol.ValidationError("LS graph dependency cycle")
        if node_id in visited:
            return
        visiting.add(node_id)
        for dependency in by_id[node_id].dependencies:
            visit(dependency)
        visiting.remove(node_id)
        visited.add(node_id)

    for row in rows:
        visit(row.node_id)


def _source_locator(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 4096)
    if "565b6a3" in text or "JIN" in text or "CrouzeixConjecture/blob/" in text:
        raise protocol.ValidationError("LS route cannot reference Jin private work files")
    if LS_SOURCE_IDENTITY not in text:
        raise protocol.ValidationError(f"{label} must reference LS arXiv v1")
    return text


def _read_json_object(path: Path, label: str) -> dict[str, Any]:
    _ensure_safe_file(path, label)
    if path.stat().st_size > MAX_JSON_BYTES:
        raise protocol.ValidationError(f"{label} exceeds byte cap")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise protocol.ValidationError(f"cannot parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise protocol.ValidationError(f"{label} must contain one JSON object")
    return value


def _ensure_safe_file(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(f"{label} cannot be a symlink")
    if not stat.S_ISREG(metadata.st_mode):
        raise protocol.ValidationError(f"{label} must be a regular file")


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


def _runtime_id_list(value: Any, label: str) -> list[str]:
    if not isinstance(value, list) or len(value) > 256:
        raise protocol.ValidationError(f"{label} must be a bounded list")
    return [formal_target._runtime_id(item, label) for item in value]
