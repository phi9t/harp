from __future__ import annotations

import json
import stat
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from types import MappingProxyType
from typing import Any, Mapping

import protocol
import tickets


PRODUCTION_LOCK_PATH = Path(__file__).resolve().with_name("formal_target.lock.json")
MAX_JSON_BYTES = 1024 * 1024
MAX_ARTIFACT_BYTES = 16 * 1024 * 1024
LEDGER_STATUSES = frozenset(
    {"mathlib_available", "locally_proved", "blocked", "conjectural"}
)
PASSING_DEPENDENCY_STATUSES = frozenset({"mathlib_available", "locally_proved"})


@dataclass(frozen=True)
class Artifact:
    artifact_id: str
    path: str
    bytes: int
    sha256: str

    @classmethod
    def from_mapping(cls, value: Mapping[str, Any]) -> "Artifact":
        _require_fields(
            value,
            frozenset({"artifact_id", "path", "bytes", "sha256"}),
            "artifact",
        )
        return cls(
            artifact_id=_runtime_id(value["artifact_id"], "artifact.artifact_id"),
            path=_safe_relative_path(value["path"], "artifact.path"),
            bytes=_integer(value["bytes"], "artifact.bytes", 0, MAX_ARTIFACT_BYTES),
            sha256=_digest(value["sha256"], "artifact.sha256"),
        )

    def to_json(self) -> dict[str, object]:
        return {
            "artifact_id": self.artifact_id,
            "path": self.path,
            "bytes": self.bytes,
            "sha256": self.sha256,
        }


@dataclass(frozen=True)
class TargetSpec:
    target_id: str
    declaration_name: str
    statement_sha256: str
    source_locator: str
    dependency_ids: tuple[str, ...]

    @classmethod
    def from_mapping(cls, value: Mapping[str, Any]) -> "TargetSpec":
        _require_fields(
            value,
            frozenset(
                {
                    "target_id",
                    "declaration_name",
                    "statement_sha256",
                    "source_locator",
                    "dependency_ids",
                }
            ),
            "target",
        )
        dependency_ids = _runtime_id_list(
            value["dependency_ids"], "target.dependency_ids", maximum=256
        )
        if len(set(dependency_ids)) != len(dependency_ids):
            raise protocol.ValidationError("target.dependency_ids must be unique")
        return cls(
            target_id=_runtime_id(value["target_id"], "target.target_id"),
            declaration_name=_bounded_string(
                value["declaration_name"], "target.declaration_name", 1, 256
            ),
            statement_sha256=_digest(
                value["statement_sha256"], "target.statement_sha256"
            ),
            source_locator=_source_locator(value["source_locator"], "target.source_locator"),
            dependency_ids=tuple(dependency_ids),
        )

    def to_json(self) -> dict[str, object]:
        return {
            "target_id": self.target_id,
            "declaration_name": self.declaration_name,
            "statement_sha256": self.statement_sha256,
            "source_locator": self.source_locator,
            "dependency_ids": list(self.dependency_ids),
        }


@dataclass(frozen=True)
class FormalTargetLock:
    schema_version: str
    target: TargetSpec
    artifacts: tuple[Artifact, ...]
    ledger_path: str

    @classmethod
    def from_mapping(cls, value: Mapping[str, Any]) -> "FormalTargetLock":
        _require_fields(
            value,
            frozenset({"schema_version", "target", "artifacts", "ledger_path"}),
            "formal target lock",
        )
        _require_equal(
            value["schema_version"],
            "crouzeix-formal-target-lock/v1",
            "schema_version",
        )
        artifacts = _artifact_list(value["artifacts"])
        artifact_ids = [artifact.artifact_id for artifact in artifacts]
        artifact_paths = [artifact.path for artifact in artifacts]
        if len(set(artifact_ids)) != len(artifact_ids):
            raise protocol.ValidationError("artifact_id values must be unique")
        if len(set(artifact_paths)) != len(artifact_paths):
            raise protocol.ValidationError("artifact paths must be unique")
        return cls(
            schema_version="crouzeix-formal-target-lock/v1",
            target=TargetSpec.from_mapping(_mapping(value["target"], "target")),
            artifacts=tuple(artifacts),
            ledger_path=_safe_relative_path(value["ledger_path"], "ledger_path"),
        )

    def to_json(self) -> dict[str, object]:
        return {
            "schema_version": self.schema_version,
            "target": self.target.to_json(),
            "artifacts": [artifact.to_json() for artifact in self.artifacts],
            "ledger_path": self.ledger_path,
        }


@dataclass(frozen=True)
class LedgerRow:
    schema_version: str
    row_id: str
    statement_sha256: str
    source_locator: str
    dependency_ids: tuple[str, ...]
    owner_route: str
    status: str

    @classmethod
    def from_mapping(cls, value: Mapping[str, Any]) -> "LedgerRow":
        _require_fields(
            value,
            frozenset(
                {
                    "schema_version",
                    "row_id",
                    "statement_sha256",
                    "source_locator",
                    "dependency_ids",
                    "owner_route",
                    "status",
                }
            ),
            "ledger row",
        )
        _require_equal(
            value["schema_version"],
            "crouzeix-formal-ledger-row/v1",
            "schema_version",
        )
        dependency_ids = _runtime_id_list(
            value["dependency_ids"], "ledger row dependency_ids", maximum=256
        )
        if len(set(dependency_ids)) != len(dependency_ids):
            raise protocol.ValidationError("ledger row dependency_ids must be unique")
        row_id = _runtime_id(value["row_id"], "ledger row row_id")
        if row_id in dependency_ids:
            raise protocol.ValidationError("ledger row cannot depend on itself")
        return cls(
            schema_version="crouzeix-formal-ledger-row/v1",
            row_id=row_id,
            statement_sha256=_digest(
                value["statement_sha256"], "ledger row statement_sha256"
            ),
            source_locator=_source_locator(
                value["source_locator"], "ledger row source_locator"
            ),
            dependency_ids=tuple(dependency_ids),
            owner_route=_runtime_id(value["owner_route"], "ledger row owner_route"),
            status=_enum(value["status"], LEDGER_STATUSES, "ledger row status"),
        )

    def to_json(self) -> dict[str, object]:
        return {
            "schema_version": self.schema_version,
            "row_id": self.row_id,
            "statement_sha256": self.statement_sha256,
            "source_locator": self.source_locator,
            "dependency_ids": list(self.dependency_ids),
            "owner_route": self.owner_route,
            "status": self.status,
        }


@dataclass(frozen=True)
class FormalTarget:
    target_id: str
    declaration_name: str
    statement_sha256: str
    source_locator: str
    artifacts: tuple[Artifact, ...]
    ledger_rows: Mapping[str, LedgerRow]


def load_lock() -> FormalTargetLock:
    return _load_lock_path(PRODUCTION_LOCK_PATH)


def load_lock_for_test(path: Path) -> FormalTargetLock:
    return _load_lock_path(path)


def validate_target(root: Path, lock: FormalTargetLock) -> FormalTarget:
    _ensure_safe_directory(root, "formal target root", create=False)
    root_absolute = root.absolute()
    ledger_root = _safe_child(root_absolute, lock.ledger_path, "ledger_path")
    _ensure_safe_directory(ledger_root, "formal target ledger", create=False)
    ledger_rows = _read_ledger_rows(ledger_root)

    for artifact in lock.artifacts:
        artifact_path = _safe_child(root_absolute, artifact.path, "artifact.path")
        _assert_regular_file_digest(artifact_path, artifact.bytes, artifact.sha256)

    for dependency_id in lock.target.dependency_ids:
        row = ledger_rows.get(dependency_id)
        if row is None:
            raise protocol.ValidationError(
                f"target dependency {dependency_id} is missing from ledger"
            )
        if row.status not in PASSING_DEPENDENCY_STATUSES:
            raise protocol.ValidationError(
                f"target dependency {dependency_id} has non-passing status {row.status}"
            )
    return FormalTarget(
        target_id=lock.target.target_id,
        declaration_name=lock.target.declaration_name,
        statement_sha256=lock.target.statement_sha256,
        source_locator=lock.target.source_locator,
        artifacts=lock.artifacts,
        ledger_rows=MappingProxyType(dict(ledger_rows)),
    )


def append_ledger_row(ledger: Path, row: LedgerRow) -> Path:
    _ensure_safe_directory(ledger, "formal target ledger", create=True)
    row_sha256 = protocol.sha256_bytes(_canonical_json_bytes(row.to_json()))
    existing = sorted(ledger.glob(f"{row.row_id}.*.json"))
    if existing:
        raise protocol.ValidationError(f"ledger row already exists: {existing[0]}")
    destination = ledger / f"{row.row_id}.{row_sha256}.json"
    _reject_symlink(destination, "ledger row")
    payload = _canonical_json_bytes(row.to_json()) + b"\n"
    try:
        with destination.open("xb") as handle:
            handle.write(payload)
    except FileExistsError as error:
        raise protocol.ValidationError(
            f"ledger row already exists: {destination}"
        ) from error
    return destination


def _load_lock_path(path: Path) -> FormalTargetLock:
    return FormalTargetLock.from_mapping(_read_strict_json_object(path, "formal target lock"))


def _read_strict_json_object(path: Path, label: str) -> dict[str, Any]:
    _ensure_safe_file(path, label)
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if metadata.st_size > MAX_JSON_BYTES:
        raise protocol.ValidationError(f"{label} exceeds byte cap")
    try:
        data = path.read_bytes()
        value = json.loads(data, object_pairs_hook=_reject_duplicate_pairs)
    except (OSError, UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise protocol.ValidationError(f"cannot parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise protocol.ValidationError(f"{label} must contain one JSON object")
    return value


def _reject_duplicate_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _read_ledger_rows(ledger: Path) -> dict[str, LedgerRow]:
    rows: dict[str, LedgerRow] = {}
    for path in sorted(ledger.glob("*.json")):
        _reject_symlink(path, "ledger row")
        row = LedgerRow.from_mapping(_read_strict_json_object(path, "ledger row"))
        row_sha256 = protocol.sha256_bytes(_canonical_json_bytes(row.to_json()))
        if path.name != f"{row.row_id}.{row_sha256}.json":
            raise protocol.ValidationError("ledger row filename must match row_id and sha256")
        if row.row_id in rows:
            raise protocol.ValidationError(f"duplicate ledger row {row.row_id}")
        rows[row.row_id] = row
    for row in rows.values():
        for dependency_id in row.dependency_ids:
            dependency = rows.get(dependency_id)
            if dependency is None:
                raise protocol.ValidationError(
                    f"ledger dependency {dependency_id} is missing"
                )
            if dependency.status not in PASSING_DEPENDENCY_STATUSES:
                raise protocol.ValidationError(
                    f"ledger dependency {dependency_id} has non-passing status {dependency.status}"
                )
    return rows


def _assert_regular_file_digest(
    path: Path, expected_bytes: int, expected_sha256: str
) -> None:
    _ensure_safe_file(path, "formal target artifact")
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect artifact: {error}") from error
    if metadata.st_size != expected_bytes:
        raise protocol.ValidationError("artifact byte count mismatch")
    data = path.read_bytes()
    observed = protocol.sha256_bytes(data)
    if observed != expected_sha256:
        raise protocol.ValidationError("artifact sha256 mismatch")


def _safe_child(root: Path, relative: str, label: str) -> Path:
    child = root / relative
    try:
        child.relative_to(root)
    except ValueError as error:
        raise protocol.ValidationError(f"{label} escapes formal target root") from error
    return child


def _ensure_safe_file(path: Path, label: str) -> None:
    _ensure_safe_directory(path.parent, f"{label} parent", create=False)
    _reject_symlink(path, label)
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if not stat.S_ISREG(metadata.st_mode):
        raise protocol.ValidationError(f"{label} must be a regular file")


def _ensure_safe_directory(path: Path, label: str, *, create: bool) -> None:
    if path == Path(""):
        raise protocol.ValidationError(f"{label} must be a directory")
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


def _artifact_list(value: Any) -> list[Artifact]:
    if not isinstance(value, list) or len(value) > 256:
        raise protocol.ValidationError("artifacts must be a bounded list")
    return [
        Artifact.from_mapping(_mapping(item, "artifact"))
        for item in value
    ]


def _runtime_id_list(value: Any, label: str, *, maximum: int) -> list[str]:
    if not isinstance(value, list) or len(value) > maximum:
        raise protocol.ValidationError(f"{label} must be a bounded list")
    return [_runtime_id(item, label) for item in value]


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


def _integer(value: Any, label: str, minimum: int, maximum: int) -> int:
    if (
        not isinstance(value, int)
        or isinstance(value, bool)
        or value < minimum
        or value > maximum
    ):
        raise protocol.ValidationError(f"{label} must be an integer in {minimum}..{maximum}")
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


def _runtime_id(value: Any, label: str) -> str:
    return tickets._runtime_id(value, label)


def _digest(value: Any, label: str) -> str:
    return tickets._digest(value, label)


def _source_locator(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 4096)
    if text == "Harp-authored":
        return text
    if text.strip() != text:
        raise protocol.ValidationError(f"{label} must be normalized")
    return text


def _safe_relative_path(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 4096)
    path = PurePosixPath(text)
    if (
        path.is_absolute()
        or text in {".", ".."}
        or any(part in {"", ".", ".."} for part in path.parts)
    ):
        raise protocol.ValidationError(f"{label} must contain normalized relative paths")
    return text


def _canonical_json_bytes(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")
