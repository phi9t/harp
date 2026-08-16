from __future__ import annotations

import json
import shutil
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
PINNED_JIN_COMMIT = "565b6a3e0659b6e0785f783b016c3f6d9f171fa5"
PINNED_JIN_TREE = "40aafa503bd32762dbf6d1a67ddef3e2b067f0e1"
PINNED_JIN_ARCHIVE_SHA256 = (
    "33ee5b75c1037866c4d2bda8eff872cc0640fd42e7c31c58a8e6047405f69542"
)
PINNED_LEAN = "leanprover/lean4:v4.28.0"
PINNED_MATHLIB_REVISION = "8f9d9cff6bd728b17a24e163c9402775d9e6a365"


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
class SourceIdentity:
    source_id: str
    commit: str
    tree: str
    archive_sha256: str

    @classmethod
    def from_mapping(cls, value: Mapping[str, Any]) -> "SourceIdentity":
        _require_fields(
            value,
            frozenset({"source_id", "commit", "tree", "archive_sha256"}),
            "source",
        )
        source_id = _runtime_id(value["source_id"], "source.source_id")
        commit = _git_sha(value["commit"], "source.commit")
        tree = _git_sha(value["tree"], "source.tree")
        archive_sha256 = _digest(value["archive_sha256"], "source.archive_sha256")
        if commit != PINNED_JIN_COMMIT:
            raise protocol.ValidationError("source.commit must match pinned Jin commit")
        if tree != PINNED_JIN_TREE:
            raise protocol.ValidationError("source.tree must match pinned Jin tree")
        if archive_sha256 != PINNED_JIN_ARCHIVE_SHA256:
            raise protocol.ValidationError("source.archive_sha256 must match pinned Jin archive")
        return cls(
            source_id=source_id,
            commit=commit,
            tree=tree,
            archive_sha256=archive_sha256,
        )

    def to_json(self) -> dict[str, object]:
        return {
            "source_id": self.source_id,
            "commit": self.commit,
            "tree": self.tree,
            "archive_sha256": self.archive_sha256,
        }


@dataclass(frozen=True)
class ToolchainIdentity:
    lean: str
    mathlib_revision: str

    @classmethod
    def from_mapping(cls, value: Mapping[str, Any]) -> "ToolchainIdentity":
        _require_fields(value, frozenset({"lean", "mathlib_revision"}), "toolchain")
        lean = _bounded_string(value["lean"], "toolchain.lean", 1, 256)
        mathlib_revision = _git_sha(value["mathlib_revision"], "toolchain.mathlib_revision")
        if lean != PINNED_LEAN:
            raise protocol.ValidationError("toolchain.lean must match pinned Lean")
        if mathlib_revision != PINNED_MATHLIB_REVISION:
            raise protocol.ValidationError(
                "toolchain.mathlib_revision must match pinned Mathlib revision"
            )
        return cls(lean=lean, mathlib_revision=mathlib_revision)

    def to_json(self) -> dict[str, object]:
        return {"lean": self.lean, "mathlib_revision": self.mathlib_revision}


@dataclass(frozen=True)
class BuildCommand:
    argv: tuple[str, ...]
    cwd: str
    env: Mapping[str, str]

    @classmethod
    def from_mapping(cls, value: Mapping[str, Any]) -> "BuildCommand":
        _require_fields(value, frozenset({"argv", "cwd", "env"}), "command")
        argv = _string_tuple(value["argv"], "command.argv", minimum=1, maximum=32)
        if argv not in {("lake", "build"), ("lake", "env", "lean", "AxiomAudit.lean")}:
            raise protocol.ValidationError("command.argv is not an approved pinned command")
        env = _string_mapping(value["env"], "command.env", maximum=16)
        return cls(
            argv=argv,
            cwd=_safe_relative_path(value["cwd"], "command.cwd"),
            env=MappingProxyType(dict(env)),
        )

    def to_json(self) -> dict[str, object]:
        return {"argv": list(self.argv), "cwd": self.cwd, "env": dict(self.env)}


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
    source: SourceIdentity
    toolchain: ToolchainIdentity
    command: BuildCommand
    target: TargetSpec
    artifacts: tuple[Artifact, ...]
    ledger_path: str
    import_allowlist: tuple[str, ...]

    @classmethod
    def from_mapping(cls, value: Mapping[str, Any]) -> "FormalTargetLock":
        _require_fields(
            value,
            frozenset(
                {
                    "schema_version",
                    "source",
                    "toolchain",
                    "command",
                    "target",
                    "artifacts",
                    "ledger_path",
                    "import_allowlist",
                }
            ),
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
        import_allowlist = _string_tuple(
            value["import_allowlist"], "import_allowlist", minimum=1, maximum=256
        )
        if len(set(import_allowlist)) != len(import_allowlist):
            raise protocol.ValidationError("import_allowlist must be unique")
        return cls(
            schema_version="crouzeix-formal-target-lock/v1",
            source=SourceIdentity.from_mapping(_mapping(value["source"], "source")),
            toolchain=ToolchainIdentity.from_mapping(
                _mapping(value["toolchain"], "toolchain")
            ),
            command=BuildCommand.from_mapping(_mapping(value["command"], "command")),
            target=TargetSpec.from_mapping(_mapping(value["target"], "target")),
            artifacts=tuple(artifacts),
            ledger_path=_safe_relative_path(value["ledger_path"], "ledger_path"),
            import_allowlist=tuple(import_allowlist),
        )

    def to_json(self) -> dict[str, object]:
        return {
            "schema_version": self.schema_version,
            "source": self.source.to_json(),
            "toolchain": self.toolchain.to_json(),
            "command": self.command.to_json(),
            "target": self.target.to_json(),
            "artifacts": [artifact.to_json() for artifact in self.artifacts],
            "ledger_path": self.ledger_path,
            "import_allowlist": list(self.import_allowlist),
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


@dataclass(frozen=True)
class ResolvedTarget:
    root: Path
    inventory_sha256: str
    command: BuildCommand
    artifacts: Mapping[str, Path]


def load_lock() -> FormalTargetLock:
    return _load_lock_path(PRODUCTION_LOCK_PATH)


def load_lock_for_test(path: Path) -> FormalTargetLock:
    return _load_lock_path(path)


def load_artifact_manifest(path: Path) -> dict[str, object]:
    value = _read_strict_json_object(path, "formal target artifact manifest")
    _require_fields(
        value,
        frozenset({"schema_version", "source_commit", "archive", "artifacts"}),
        "formal target artifact manifest",
    )
    _require_equal(
        value["schema_version"],
        "crouzeix-formal-artifact-manifest/v1",
        "schema_version",
    )
    if _git_sha(value["source_commit"], "source_commit") != PINNED_JIN_COMMIT:
        raise protocol.ValidationError("source_commit must match pinned Jin commit")
    archive = _mapping(value["archive"], "archive")
    _require_fields(archive, frozenset({"source_url", "bytes", "sha256"}), "archive")
    archive_value = {
        "source_url": _bounded_string(archive["source_url"], "archive.source_url", 1, 4096),
        "bytes": _integer(archive["bytes"], "archive.bytes", 1, MAX_ARTIFACT_BYTES),
        "sha256": _digest(archive["sha256"], "archive.sha256"),
    }
    if archive_value["sha256"] != PINNED_JIN_ARCHIVE_SHA256:
        raise protocol.ValidationError("archive.sha256 must match pinned Jin archive")
    artifacts = _manifest_artifacts(value["artifacts"])
    return {
        "schema_version": "crouzeix-formal-artifact-manifest/v1",
        "source_commit": PINNED_JIN_COMMIT,
        "archive": archive_value,
        "artifacts": artifacts,
    }


def load_preflight_receipt(path: Path) -> dict[str, object]:
    value = _read_strict_json_object(path, "formal target preflight")
    _require_fields(
        value,
        frozenset(
            {
                "schema_version",
                "source_commit",
                "toolchain",
                "mathlib_revision",
                "status",
                "reason",
                "available_kib",
                "required_kib",
                "build_log",
                "scan_log",
            }
        ),
        "formal target preflight",
    )
    _require_equal(
        value["schema_version"],
        "crouzeix-formal-preflight/v1",
        "schema_version",
    )
    if _git_sha(value["source_commit"], "source_commit") != PINNED_JIN_COMMIT:
        raise protocol.ValidationError("source_commit must match pinned Jin commit")
    if value["toolchain"] != PINNED_LEAN:
        raise protocol.ValidationError("toolchain must match pinned Lean")
    if _git_sha(value["mathlib_revision"], "mathlib_revision") != PINNED_MATHLIB_REVISION:
        raise protocol.ValidationError("mathlib_revision must match pinned Mathlib")
    status = _enum(value["status"], frozenset({"passed", "blocked"}), "status")
    reason = _bounded_string(value["reason"], "reason", 1, 512)
    if status == "passed" and reason != "preflight-passed":
        raise protocol.ValidationError("passed preflight must use preflight-passed reason")
    if status == "blocked" and reason == "preflight-passed":
        raise protocol.ValidationError("blocked preflight must record a blocker reason")
    build_log = _manifest_artifact(_mapping(value["build_log"], "build_log"), "build_log")
    scan_log = _manifest_artifact(_mapping(value["scan_log"], "scan_log"), "scan_log")
    return {
        "schema_version": "crouzeix-formal-preflight/v1",
        "source_commit": PINNED_JIN_COMMIT,
        "toolchain": PINNED_LEAN,
        "mathlib_revision": PINNED_MATHLIB_REVISION,
        "status": status,
        "reason": reason,
        "available_kib": _integer(value["available_kib"], "available_kib", 0, 1 << 60),
        "required_kib": _integer(value["required_kib"], "required_kib", 0, 1 << 60),
        "build_log": build_log,
        "scan_log": scan_log,
    }


def ensure_preflight_allows_provision(
    preflight: Mapping[str, object], destination: Path
) -> None:
    status = _enum(preflight.get("status"), frozenset({"passed", "blocked"}), "status")
    if status != "passed":
        reason = _bounded_string(preflight.get("reason"), "reason", 1, 512)
        raise protocol.ValidationError(f"preflight blocks provision: {reason}")
    _reject_symlink(destination, "runtime destination")


def provision(
    lock: FormalTargetLock, artifacts: Mapping[str, Path], base: Path
) -> ResolvedTarget:
    expected_ids = {artifact.artifact_id for artifact in lock.artifacts}
    observed_ids = set(artifacts)
    missing = sorted(expected_ids - observed_ids)
    extra = sorted(observed_ids - expected_ids)
    if missing:
        raise protocol.ValidationError(f"missing artifacts: {', '.join(missing)}")
    if extra:
        raise protocol.ValidationError(f"extra artifacts: {', '.join(extra)}")
    _reject_symlink(base, "runtime destination")
    if base.exists():
        raise protocol.ValidationError(f"runtime destination already exists: {base}")
    _ensure_safe_directory(base.parent, "runtime parent", create=True)
    staging = base.parent / f".{base.name}.staging"
    _reject_symlink(staging, "runtime staging")
    if staging.exists():
        raise protocol.ValidationError(f"runtime staging already exists: {staging}")
    staging.mkdir(mode=0o700)
    try:
        copied: dict[str, Path] = {}
        for artifact in lock.artifacts:
            source = artifacts[artifact.artifact_id]
            _assert_regular_file_digest(source, artifact.bytes, artifact.sha256)
            destination = staging / artifact.path
            _ensure_safe_directory(destination.parent, "runtime artifact parent", create=True)
            try:
                with source.open("rb") as reader, destination.open("xb") as writer:
                    shutil.copyfileobj(reader, writer)
            except FileExistsError as error:
                raise protocol.ValidationError(
                    f"runtime artifact already exists: {destination}"
                ) from error
            _assert_regular_file_digest(destination, artifact.bytes, artifact.sha256)
            copied[artifact.artifact_id] = base / artifact.path
        inventory = _runtime_inventory(lock)
        inventory_path = staging / "runtime-inventory.json"
        _write_json_create_only(inventory_path, inventory, "runtime inventory")
        inventory_sha256 = protocol.sha256_bytes(inventory_path.read_bytes())
        try:
            staging.rename(base)
        except FileExistsError as error:
            raise protocol.ValidationError(
                f"runtime destination already exists: {base}"
            ) from error
        return ResolvedTarget(
            root=base,
            inventory_sha256=inventory_sha256,
            command=lock.command,
            artifacts=MappingProxyType(copied),
        )
    except BaseException:
        if staging.exists() and not base.exists():
            _remove_tree(staging)
        raise


def resolve(lock: FormalTargetLock, base: Path) -> ResolvedTarget:
    _ensure_safe_directory(base, "runtime root", create=False)
    copied: dict[str, Path] = {}
    for artifact in lock.artifacts:
        path = base / artifact.path
        _assert_regular_file_digest(path, artifact.bytes, artifact.sha256)
        copied[artifact.artifact_id] = path
    inventory_path = base / "runtime-inventory.json"
    _assert_runtime_inventory(lock, inventory_path)
    return ResolvedTarget(
        root=base,
        inventory_sha256=protocol.sha256_bytes(inventory_path.read_bytes()),
        command=lock.command,
        artifacts=MappingProxyType(copied),
    )


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


def _write_json_create_only(path: Path, value: Mapping[str, object], label: str) -> None:
    _reject_symlink(path, label)
    _ensure_safe_directory(path.parent, f"{label} parent", create=True)
    payload = _canonical_json_bytes(value) + b"\n"
    try:
        with path.open("xb") as handle:
            handle.write(payload)
    except FileExistsError as error:
        raise protocol.ValidationError(f"{label} already exists: {path}") from error


def _runtime_inventory(lock: FormalTargetLock) -> dict[str, object]:
    return {
        "schema_version": "crouzeix-formal-runtime-inventory/v1",
        "source": lock.source.to_json(),
        "toolchain": lock.toolchain.to_json(),
        "command": lock.command.to_json(),
        "artifacts": [artifact.to_json() for artifact in lock.artifacts],
    }


def _assert_runtime_inventory(lock: FormalTargetLock, path: Path) -> None:
    value = _read_strict_json_object(path, "runtime inventory")
    if value != _runtime_inventory(lock):
        raise protocol.ValidationError("runtime inventory mismatch")


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


def _remove_tree(root: Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        if path.is_symlink() or path.is_file():
            path.unlink()
        elif path.is_dir():
            path.rmdir()
    root.rmdir()


def _artifact_list(value: Any) -> list[Artifact]:
    if not isinstance(value, list) or len(value) > 256:
        raise protocol.ValidationError("artifacts must be a bounded list")
    return [
        Artifact.from_mapping(_mapping(item, "artifact"))
        for item in value
    ]


def _manifest_artifacts(value: Any) -> dict[str, object]:
    item = _mapping(value, "artifacts")
    if len(item) > 256:
        raise protocol.ValidationError("artifacts must be bounded")
    return {
        _runtime_id(key, "artifact id"): _manifest_artifact(
            _mapping(artifact, "artifact"), f"artifacts.{key}"
        )
        for key, artifact in item.items()
    }


def _manifest_artifact(value: Mapping[str, Any], label: str) -> dict[str, object]:
    _require_fields(
        value,
        frozenset({"path", "bytes", "sha256", "source_locator", "license_status"}),
        label,
    )
    return {
        "path": _safe_relative_path(value["path"], f"{label}.path"),
        "bytes": _integer(value["bytes"], f"{label}.bytes", 0, MAX_ARTIFACT_BYTES),
        "sha256": _digest(value["sha256"], f"{label}.sha256"),
        "source_locator": _source_locator(value["source_locator"], f"{label}.source_locator"),
        "license_status": _bounded_string(
            value["license_status"], f"{label}.license_status", 1, 128
        ),
    }


def _runtime_id_list(value: Any, label: str, *, maximum: int) -> list[str]:
    if not isinstance(value, list) or len(value) > maximum:
        raise protocol.ValidationError(f"{label} must be a bounded list")
    return [_runtime_id(item, label) for item in value]


def _string_tuple(value: Any, label: str, *, minimum: int, maximum: int) -> tuple[str, ...]:
    if not isinstance(value, list) or len(value) < minimum or len(value) > maximum:
        raise protocol.ValidationError(f"{label} must be a list with {minimum}..{maximum} entries")
    return tuple(_bounded_string(item, f"{label} item", 1, 4096) for item in value)


def _string_mapping(value: Any, label: str, *, maximum: int) -> dict[str, str]:
    item = _mapping(value, label)
    if len(item) > maximum:
        raise protocol.ValidationError(f"{label} must be bounded")
    result: dict[str, str] = {}
    for key, text in item.items():
        result[_bounded_string(key, f"{label} key", 1, 128)] = _bounded_string(
            text, f"{label}.{key}", 0, 4096
        )
    return result


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


def _git_sha(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 40, 40)
    if any(char not in "0123456789abcdef" for char in text):
        raise protocol.ValidationError(f"{label} must be a lowercase Git SHA-1")
    return text


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
