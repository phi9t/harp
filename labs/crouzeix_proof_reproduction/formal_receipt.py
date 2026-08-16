from __future__ import annotations

import argparse
import json
import os
import stat
from pathlib import Path, PurePosixPath
from typing import Any, Mapping

import protocol
import tickets


FORMAL_RECEIPT_FIELDS = frozenset(
    {
        "schema_version",
        "attempt_id",
        "run_id",
        "ticket_id",
        "ticket_sha256",
        "candidate",
        "toolchain",
        "source",
        "command",
        "logs",
        "resource_receipt",
        "axioms",
        "status",
        "reason",
        "started_at_utc",
        "completed_at_utc",
        "formal_attempt_sha256",
    }
)
CANDIDATE_FIELDS = frozenset(
    {"outcome", "candidate_id", "candidate_sha256", "candidate_bytes"}
)
TOOLCHAIN_FIELDS = frozenset({"name", "version", "platform", "toolchain_sha256"})
SOURCE_FIELDS = frozenset({"path", "sha256", "bytes"})
COMMAND_FIELDS = frozenset({"argv", "cwd", "env_sha256", "exit_code"})
LOG_FIELDS = frozenset(
    {
        "stdout_path",
        "stdout_sha256",
        "stderr_path",
        "stderr_sha256",
        "complete",
    }
)
RESOURCE_REF_FIELDS = frozenset({"path", "sha256"})
RESOURCE_RECEIPT_FIELDS = frozenset(
    {
        "schema_version",
        "host_space_bytes",
        "memory_limit_bytes",
        "cpu_limit",
        "preflight_status",
        "checked_at_utc",
    }
)
AXIOM_FIELDS = frozenset(
    {
        "scan_performed",
        "scanner",
        "allowed_axioms",
        "observed_axioms",
        "scan_log_path",
        "scan_log_sha256",
    }
)
FORMAL_STATUSES = frozenset({"not_attempted", "blocked", "failed", "passed"})
CANDIDATE_OUTCOMES = frozenset({"candidate", "no_candidate"})
RESOURCE_STATUSES = frozenset({"ok", "blocked"})
MAX_JSON_BYTES = 1024 * 1024
MAX_TEXT_BYTES = 4 * 1024 * 1024


def prepare_attempt(
    attempt_dir: Path,
    *,
    ticket: Mapping[str, Any],
    request: Mapping[str, Any],
    source_bytes: bytes,
    stdout_bytes: bytes,
    stderr_bytes: bytes,
    axiom_log_bytes: bytes,
    resource_receipt: Mapping[str, Any],
) -> Path:
    destination = attempt_dir.expanduser().absolute()
    _reject_symlink(destination, "formal attempt directory")
    if destination.exists():
        raise protocol.ValidationError(
            f"formal attempt directory already exists: {destination}"
        )
    _ensure_directory(destination.parent, "formal attempt parent", create=True)

    ticket_value = tickets.validate_runtime_ticket(ticket)
    if ticket_value["task_kind"] != "formal_attempt":
        raise protocol.ValidationError("runtime ticket task_kind must be formal_attempt")
    receipt = _build_receipt(
        ticket=ticket_value,
        request=request,
        source_bytes=source_bytes,
        stdout_bytes=stdout_bytes,
        stderr_bytes=stderr_bytes,
        axiom_log_bytes=axiom_log_bytes,
        resource_receipt=resource_receipt,
    )

    destination.mkdir(mode=0o700)
    try:
        _write_json_create_only(destination / "attempt.json", ticket_value, "formal attempt ticket")
        _write_json_create_only(
            destination / "resource_receipt.json",
            validate_resource_receipt(resource_receipt),
            "formal resource receipt",
        )
        _write_bytes_create_only(
            destination / str(receipt["source"]["path"]),
            source_bytes,
            "formal source",
        )
        _write_bytes_create_only(
            destination / str(receipt["logs"]["stdout_path"]),
            stdout_bytes,
            "formal stdout log",
        )
        _write_bytes_create_only(
            destination / str(receipt["logs"]["stderr_path"]),
            stderr_bytes,
            "formal stderr log",
        )
        _write_bytes_create_only(
            destination / str(receipt["axioms"]["scan_log_path"]),
            axiom_log_bytes,
            "formal axiom log",
        )
        _write_json_create_only(destination / "receipt.json", receipt, "formal receipt")
    except BaseException:
        _remove_new_tree(destination)
        raise
    return destination.resolve()


def validate_attempt_dir(attempt_dir: Path) -> dict[str, object]:
    root = attempt_dir.expanduser().absolute()
    _ensure_directory(root, "formal attempt directory", create=False)
    ticket_path = root / "attempt.json"
    receipt_path = root / "receipt.json"
    resource_path = root / "resource_receipt.json"
    ticket_value = tickets.validate_runtime_ticket(
        _read_json_object(ticket_path, "formal attempt ticket")
    )
    receipt = validate_receipt(_read_json_object(receipt_path, "formal receipt"))
    resource = validate_resource_receipt(
        _read_json_object(resource_path, "formal resource receipt")
    )
    if ticket_value["task_kind"] != "formal_attempt":
        raise protocol.ValidationError("formal attempt ticket has wrong task_kind")
    if receipt["ticket_id"] != ticket_value["ticket_id"]:
        raise protocol.ValidationError("formal receipt ticket_id mismatch")
    if receipt["ticket_sha256"] != tickets.canonical_sha256(ticket_value):
        raise protocol.ValidationError("formal receipt ticket_sha256 mismatch")
    _assert_file_digest(
        root / str(receipt["source"]["path"]),
        int(receipt["source"]["bytes"]),
        str(receipt["source"]["sha256"]),
        "source.sha256",
    )
    _assert_file_digest(
        root / str(receipt["logs"]["stdout_path"]),
        None,
        str(receipt["logs"]["stdout_sha256"]),
        "logs.stdout_sha256",
    )
    _assert_file_digest(
        root / str(receipt["logs"]["stderr_path"]),
        None,
        str(receipt["logs"]["stderr_sha256"]),
        "logs.stderr_sha256",
    )
    _assert_file_digest(
        root / str(receipt["axioms"]["scan_log_path"]),
        None,
        str(receipt["axioms"]["scan_log_sha256"]),
        "axioms.scan_log_sha256",
    )
    if receipt["resource_receipt"]["sha256"] != canonical_sha256(resource):
        raise protocol.ValidationError("resource_receipt.sha256 mismatch")
    return receipt


def validate_receipt(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, FORMAL_RECEIPT_FIELDS, "formal receipt")
    _require_equal(
        value["schema_version"], "crouzeix-formal-attempt-receipt/v1", "schema_version"
    )
    result: dict[str, object] = {
        "schema_version": value["schema_version"],
        "attempt_id": _runtime_id(value["attempt_id"], "attempt_id"),
        "run_id": _runtime_id(value["run_id"], "run_id"),
        "ticket_id": _runtime_id(value["ticket_id"], "ticket_id"),
        "ticket_sha256": _digest(value["ticket_sha256"], "ticket_sha256"),
        "candidate": _validate_candidate(value["candidate"]),
        "toolchain": _validate_toolchain(value["toolchain"]),
        "source": _validate_source(value["source"]),
        "command": _validate_command(value["command"]),
        "logs": _validate_logs(value["logs"]),
        "resource_receipt": _validate_resource_ref(value["resource_receipt"]),
        "axioms": _validate_axioms(value["axioms"]),
        "status": _enum(value["status"], FORMAL_STATUSES, "status"),
        "reason": _bounded_string(value["reason"], "reason", 1, 4096),
        "started_at_utc": _timestamp(value["started_at_utc"], "started_at_utc"),
        "completed_at_utc": _timestamp(value["completed_at_utc"], "completed_at_utc"),
        "formal_attempt_sha256": _digest(
            value["formal_attempt_sha256"], "formal_attempt_sha256"
        ),
    }
    if result["formal_attempt_sha256"] != canonical_sha256_without_self(result):
        raise protocol.ValidationError("formal_attempt_sha256 mismatch")
    _validate_terminal_invariants(result)
    return result


def validate_resource_receipt(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, RESOURCE_RECEIPT_FIELDS, "formal resource receipt")
    _require_equal(
        value["schema_version"],
        "crouzeix-formal-resource-receipt/v1",
        "schema_version",
    )
    return {
        "schema_version": value["schema_version"],
        "host_space_bytes": _integer(
            value["host_space_bytes"], "host_space_bytes", 0, 1 << 60
        ),
        "memory_limit_bytes": _integer(
            value["memory_limit_bytes"], "memory_limit_bytes", 0, 1 << 60
        ),
        "cpu_limit": _integer(value["cpu_limit"], "cpu_limit", 0, 1024),
        "preflight_status": _enum(
            value["preflight_status"], RESOURCE_STATUSES, "preflight_status"
        ),
        "checked_at_utc": _timestamp(value["checked_at_utc"], "checked_at_utc"),
    }


def canonical_sha256(value: Mapping[str, Any]) -> str:
    return protocol.sha256_bytes(_canonical_json_bytes(value))


def canonical_sha256_without_self(value: Mapping[str, Any]) -> str:
    payload = dict(value)
    payload.pop("formal_attempt_sha256", None)
    return canonical_sha256(payload)


def main() -> None:
    parser = argparse.ArgumentParser()
    subcommands = parser.add_subparsers(dest="command", required=True)
    validate = subcommands.add_parser("validate")
    validate.add_argument("--attempt-dir", required=True, type=Path)
    args = parser.parse_args()
    if args.command == "validate":
        receipt = validate_attempt_dir(args.attempt_dir)
        print(json.dumps(receipt, sort_keys=True, separators=(",", ":")))


def _build_receipt(
    *,
    ticket: Mapping[str, Any],
    request: Mapping[str, Any],
    source_bytes: bytes,
    stdout_bytes: bytes,
    stderr_bytes: bytes,
    axiom_log_bytes: bytes,
    resource_receipt: Mapping[str, Any],
) -> dict[str, object]:
    if request.get("ticket_id") not in {None, ticket["ticket_id"]}:
        raise protocol.ValidationError("request ticket_id does not match ticket")
    draft = dict(request)
    draft["schema_version"] = "crouzeix-formal-attempt-receipt/v1"
    draft["ticket_id"] = ticket["ticket_id"]
    draft["ticket_sha256"] = tickets.canonical_sha256(ticket)
    resource_value = validate_resource_receipt(resource_receipt)
    source = _validate_source(draft["source"])
    logs = _validate_logs(draft["logs"])
    axioms = _validate_axioms(draft["axioms"])
    resource_ref = _validate_resource_ref(draft["resource_receipt"])
    _check_bytes(source_bytes, int(source["bytes"]), str(source["sha256"]), "source.sha256")
    _check_bytes(stdout_bytes, None, str(logs["stdout_sha256"]), "logs.stdout_sha256")
    _check_bytes(stderr_bytes, None, str(logs["stderr_sha256"]), "logs.stderr_sha256")
    _check_bytes(
        axiom_log_bytes,
        None,
        str(axioms["scan_log_sha256"]),
        "axioms.scan_log_sha256",
    )
    if resource_ref["sha256"] != canonical_sha256(resource_value):
        raise protocol.ValidationError("resource_receipt.sha256 mismatch")
    draft["formal_attempt_sha256"] = "0" * 64
    validated = validate_receipt(
        draft | {"formal_attempt_sha256": canonical_sha256_without_self(draft)}
    )
    return validated


def _validate_candidate(value: Any) -> dict[str, object]:
    item = _mapping(value, "candidate")
    _require_fields(item, CANDIDATE_FIELDS, "candidate")
    outcome = _enum(item["outcome"], CANDIDATE_OUTCOMES, "candidate.outcome")
    candidate_id = item["candidate_id"]
    candidate_sha256 = item["candidate_sha256"]
    candidate_bytes = item["candidate_bytes"]
    if outcome == "candidate":
        candidate_id = _runtime_id(candidate_id, "candidate.candidate_id")
        candidate_sha256 = _digest(candidate_sha256, "candidate.candidate_sha256")
        candidate_bytes = _integer(
            candidate_bytes, "candidate.candidate_bytes", 1, MAX_TEXT_BYTES
        )
    else:
        if candidate_id is not None or candidate_sha256 is not None or candidate_bytes != 0:
            raise protocol.ValidationError("no_candidate outcome must not bind candidate bytes")
        candidate_id = None
        candidate_sha256 = None
        candidate_bytes = 0
    return {
        "outcome": outcome,
        "candidate_id": candidate_id,
        "candidate_sha256": candidate_sha256,
        "candidate_bytes": candidate_bytes,
    }


def _validate_toolchain(value: Any) -> dict[str, object]:
    item = _mapping(value, "toolchain")
    _require_fields(item, TOOLCHAIN_FIELDS, "toolchain")
    return {
        "name": _bounded_string(item["name"], "toolchain.name", 1, 128),
        "version": _bounded_string(item["version"], "toolchain.version", 1, 256),
        "platform": _bounded_string(item["platform"], "toolchain.platform", 1, 256),
        "toolchain_sha256": _digest(
            item["toolchain_sha256"], "toolchain.toolchain_sha256"
        ),
    }


def _validate_source(value: Any) -> dict[str, object]:
    item = _mapping(value, "source")
    _require_fields(item, SOURCE_FIELDS, "source")
    return {
        "path": _safe_relative_path(item["path"], "source.path"),
        "sha256": _digest(item["sha256"], "source.sha256"),
        "bytes": _integer(item["bytes"], "source.bytes", 0, MAX_TEXT_BYTES),
    }


def _validate_command(value: Any) -> dict[str, object]:
    item = _mapping(value, "command")
    _require_fields(item, COMMAND_FIELDS, "command")
    argv = _string_list(item["argv"], "command.argv", minimum=1, maximum=64)
    exit_code = item["exit_code"]
    if exit_code is not None:
        exit_code = _integer(exit_code, "command.exit_code", 0, 255)
    return {
        "argv": argv,
        "cwd": _safe_relative_path(item["cwd"], "command.cwd", allow_dot=True),
        "env_sha256": _digest(item["env_sha256"], "command.env_sha256"),
        "exit_code": exit_code,
    }


def _validate_logs(value: Any) -> dict[str, object]:
    item = _mapping(value, "logs")
    _require_fields(item, LOG_FIELDS, "logs")
    if not isinstance(item["complete"], bool):
        raise protocol.ValidationError("logs.complete must be boolean")
    return {
        "stdout_path": _safe_relative_path(item["stdout_path"], "logs.stdout_path"),
        "stdout_sha256": _digest(item["stdout_sha256"], "logs.stdout_sha256"),
        "stderr_path": _safe_relative_path(item["stderr_path"], "logs.stderr_path"),
        "stderr_sha256": _digest(item["stderr_sha256"], "logs.stderr_sha256"),
        "complete": item["complete"],
    }


def _validate_resource_ref(value: Any) -> dict[str, object]:
    item = _mapping(value, "resource_receipt")
    _require_fields(item, RESOURCE_REF_FIELDS, "resource_receipt")
    return {
        "path": _safe_relative_path(item["path"], "resource_receipt.path"),
        "sha256": _digest(item["sha256"], "resource_receipt.sha256"),
    }


def _validate_axioms(value: Any) -> dict[str, object]:
    item = _mapping(value, "axioms")
    _require_fields(item, AXIOM_FIELDS, "axioms")
    if not isinstance(item["scan_performed"], bool):
        raise protocol.ValidationError("axioms.scan_performed must be boolean")
    allowed = _string_list(
        item["allowed_axioms"], "axioms.allowed_axioms", minimum=0, maximum=128
    )
    observed = _string_list(
        item["observed_axioms"], "axioms.observed_axioms", minimum=0, maximum=128
    )
    return {
        "scan_performed": item["scan_performed"],
        "scanner": _bounded_string(item["scanner"], "axioms.scanner", 1, 256),
        "allowed_axioms": allowed,
        "observed_axioms": observed,
        "scan_log_path": _safe_relative_path(
            item["scan_log_path"], "axioms.scan_log_path"
        ),
        "scan_log_sha256": _digest(
            item["scan_log_sha256"], "axioms.scan_log_sha256"
        ),
    }


def _validate_terminal_invariants(receipt: Mapping[str, object]) -> None:
    status = str(receipt["status"])
    command = _mapping(receipt["command"], "command")
    logs = _mapping(receipt["logs"], "logs")
    axioms = _mapping(receipt["axioms"], "axioms")
    resource = _mapping(receipt["resource_receipt"], "resource_receipt")
    exit_code = command["exit_code"]
    if status == "passed":
        if exit_code != 0:
            raise protocol.ValidationError("passed formal attempt requires zero exit code")
        if logs["complete"] is not True:
            raise protocol.ValidationError("passed formal attempt requires complete logs")
        if axioms["scan_performed"] is not True:
            raise protocol.ValidationError("passed formal attempt requires explicit axiom scan")
        unexpected = sorted(set(axioms["observed_axioms"]) - set(axioms["allowed_axioms"]))
        if unexpected:
            raise protocol.ValidationError("passed formal attempt has disallowed axioms")
    elif status == "failed":
        if exit_code is None or exit_code == 0:
            raise protocol.ValidationError("failed formal attempt requires nonzero command exit")
    elif status == "blocked":
        if exit_code == 0:
            raise protocol.ValidationError("blocked formal attempt cannot have zero command exit")
        _digest(resource["sha256"], "resource_receipt.sha256")
    elif status == "not_attempted":
        if exit_code is not None:
            raise protocol.ValidationError("not_attempted formal attempt cannot bind a command exit")


def _read_json_object(path: Path, label: str) -> dict[str, Any]:
    _reject_symlink(path, label)
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if not stat.S_ISREG(metadata.st_mode):
        raise protocol.ValidationError(f"{label} must be a regular file")
    if metadata.st_size > MAX_JSON_BYTES:
        raise protocol.ValidationError(f"{label} exceeds byte cap")
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise protocol.ValidationError(f"cannot parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise protocol.ValidationError(f"{label} must contain one JSON object")
    return value


def _write_json_create_only(path: Path, value: Mapping[str, Any], label: str) -> None:
    _write_bytes_create_only(path, _canonical_json_bytes(value) + b"\n", label)


def _write_bytes_create_only(path: Path, data: bytes, label: str) -> None:
    if len(data) > MAX_TEXT_BYTES:
        raise protocol.ValidationError(f"{label} exceeds byte cap")
    _reject_symlink(path, label)
    _ensure_directory(path.parent, f"{label} parent", create=True)
    try:
        with path.open("xb") as handle:
            handle.write(data)
    except FileExistsError as error:
        raise protocol.ValidationError(f"{label} already exists: {path}") from error


def _assert_file_digest(
    path: Path, expected_bytes: int | None, expected_sha256: str, label: str
) -> None:
    _reject_symlink(path, label)
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if not stat.S_ISREG(metadata.st_mode):
        raise protocol.ValidationError(f"{label} must be a regular file")
    if expected_bytes is not None and metadata.st_size != expected_bytes:
        raise protocol.ValidationError(f"{label} byte count mismatch")
    if metadata.st_size > MAX_TEXT_BYTES:
        raise protocol.ValidationError(f"{label} exceeds byte cap")
    data = path.read_bytes()
    _check_bytes(data, expected_bytes, expected_sha256, label)


def _check_bytes(
    data: bytes, expected_bytes: int | None, expected_sha256: str, label: str
) -> None:
    if expected_bytes is not None and len(data) != expected_bytes:
        raise protocol.ValidationError(f"{label} byte count mismatch")
    observed = protocol.sha256_bytes(data)
    if observed != expected_sha256:
        raise protocol.ValidationError(f"{label} mismatch")


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


def _reject_symlink(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        return
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(f"{label} cannot be a symlink")


def _remove_new_tree(root: Path) -> None:
    if not root.exists():
        return
    for path in sorted(root.rglob("*"), reverse=True):
        if path.is_symlink() or path.is_file():
            path.unlink()
        elif path.is_dir():
            path.rmdir()
    root.rmdir()


def _canonical_json_bytes(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")


def _require_fields(value: Mapping[str, Any], allowed: frozenset[str], label: str) -> None:
    if not isinstance(value, Mapping):
        raise protocol.ValidationError(f"{label} must be an object")
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


def _runtime_id(value: Any, label: str) -> str:
    return tickets._runtime_id(value, label)


def _digest(value: Any, label: str) -> str:
    return tickets._digest(value, label)


def _timestamp(value: Any, label: str) -> str:
    return tickets._timestamp(value, label)


def _integer(value: Any, label: str, minimum: int, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        raise protocol.ValidationError(f"{label} must be an integer")
    if not minimum <= value <= maximum:
        raise protocol.ValidationError(f"{label} must be between {minimum} and {maximum}")
    return value


def _bounded_string(value: Any, label: str, minimum: int, maximum: int) -> str:
    if not isinstance(value, str):
        raise protocol.ValidationError(f"{label} must be a string")
    if "\0" in value or not minimum <= len(value) <= maximum:
        raise protocol.ValidationError(f"{label} length is invalid")
    return value


def _string_list(
    value: Any, label: str, *, minimum: int, maximum: int
) -> list[str]:
    if not isinstance(value, list) or not minimum <= len(value) <= maximum:
        raise protocol.ValidationError(f"{label} must be a bounded list")
    return [_bounded_string(item, label, 1, 4096) for item in value]


def _safe_relative_path(value: Any, label: str, *, allow_dot: bool = False) -> str:
    text = _bounded_string(value, label, 1, 4096)
    if allow_dot and text == ".":
        return text
    path = PurePosixPath(text)
    if (
        path.is_absolute()
        or text in {"", "."}
        or any(part in {"", ".", ".."} for part in path.parts)
    ):
        raise protocol.ValidationError(f"{label} must be a normalized relative path")
    return text


if __name__ == "__main__":
    main()
