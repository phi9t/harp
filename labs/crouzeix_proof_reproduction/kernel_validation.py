from __future__ import annotations

import json
import stat
from pathlib import Path
from typing import Any, Mapping

import formal_target
import protocol


FIELDS = frozenset({"schema_version", "status", "admissions"})
ADMISSION_FIELDS = frozenset(
    {
        "admission_id",
        "statement_sha256",
        "consumer_ids",
        "allowed_imports",
        "prohibited_dependencies",
        "status",
    }
)


def load_admissions(path: Path) -> dict[str, object]:
    return validate_admissions(_read_json_object(path, "shared kernel admissions"))


def validate_admissions(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, FIELDS, "shared kernel admissions")
    _require_equal(
        value["schema_version"],
        "crouzeix-shared-kernel-admissions/v1",
        "schema_version",
    )
    status = _enum(
        value["status"],
        frozenset({"no-admitted-kernel-lemmas", "admissions-present"}),
        "status",
    )
    admissions = value["admissions"]
    if not isinstance(admissions, list):
        raise protocol.ValidationError("admissions must be a list")
    normalized = [_admission(_mapping(item, "kernel admission")) for item in admissions]
    if status == "no-admitted-kernel-lemmas" and normalized:
        raise protocol.ValidationError("no-admitted-kernel-lemmas requires empty admissions")
    if status == "admissions-present" and not normalized:
        raise protocol.ValidationError("admissions-present requires admissions")
    ids = [item["admission_id"] for item in normalized]
    if len(set(ids)) != len(ids):
        raise protocol.ValidationError("duplicate kernel admission_id")
    return {
        "schema_version": "crouzeix-shared-kernel-admissions/v1",
        "status": status,
        "admissions": normalized,
    }


def _admission(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, ADMISSION_FIELDS, "kernel admission")
    consumers = _runtime_id_list(value["consumer_ids"], "consumer_ids")
    if len(set(consumers)) != len(consumers):
        raise protocol.ValidationError("consumer_ids must be unique")
    if len(consumers) < 2:
        raise protocol.ValidationError("kernel admission requires two consumers")
    prohibited = _string_list(value["prohibited_dependencies"], "prohibited_dependencies")
    if any("terminal" in dependency or "crouzeix" in dependency.lower() for dependency in prohibited):
        raise protocol.ValidationError("kernel admission cannot depend on terminal Crouzeix theorem")
    return {
        "admission_id": formal_target._runtime_id(value["admission_id"], "admission_id"),
        "statement_sha256": formal_target._digest(value["statement_sha256"], "statement_sha256"),
        "consumer_ids": consumers,
        "allowed_imports": _string_list(value["allowed_imports"], "allowed_imports"),
        "prohibited_dependencies": prohibited,
        "status": _enum(value["status"], frozenset({"admitted", "blocked"}), "status"),
    }


def _read_json_object(path: Path, label: str) -> dict[str, Any]:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(f"{label} cannot be a symlink")
    if not stat.S_ISREG(metadata.st_mode):
        raise protocol.ValidationError(f"{label} must be a regular file")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise protocol.ValidationError(f"cannot parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise protocol.ValidationError(f"{label} must contain one JSON object")
    return value


def _require_fields(value: Mapping[str, Any], allowed: frozenset[str], label: str) -> None:
    fields = set(value)
    if fields != set(allowed):
        raise protocol.ValidationError(f"{label} fields are invalid")


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


def _runtime_id_list(value: Any, label: str) -> list[str]:
    if not isinstance(value, list) or len(value) > 256:
        raise protocol.ValidationError(f"{label} must be a bounded list")
    return [formal_target._runtime_id(item, label) for item in value]


def _string_list(value: Any, label: str) -> list[str]:
    if not isinstance(value, list) or len(value) > 256:
        raise protocol.ValidationError(f"{label} must be a bounded list")
    result: list[str] = []
    for item in value:
        if not isinstance(item, str) or not item or "\0" in item:
            raise protocol.ValidationError(f"{label} entries must be nonempty strings")
        result.append(item)
    return result
