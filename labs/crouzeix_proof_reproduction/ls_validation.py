from __future__ import annotations

import json
import os
import re
import stat
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Mapping, Sequence

import formal_target
import ls_contract
import provider_independence
import protocol


MAX_JSON_BYTES = 1024 * 1024
MAX_MEMBER_BYTES = 4 * 1024 * 1024
MAX_ATTEMPTS = 256
MAX_ATTEMPT_RECEIPT_BYTES = 4 * MAX_JSON_BYTES
MAX_IMPORT_MODULES = 4096
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
NODE_OPTIONAL_FIELDS = frozenset({"receipt_sha256", "blocked_reason", "failed_reason"})
INVENTORY_FIELDS = frozenset({"schema_version", "source_identity", "facts"})
FACT_FIELDS = frozenset({"fact_id", "statement_sha256", "source_locator", "resolution"})
NODE_ROLES = frozenset(
    {"definition", "adapter", "library_fact", "intermediate", "terminal"}
)
NODE_STATUSES = frozenset({"mapped", "blocked", "passed", "failed"})
FACT_RESOLUTIONS = frozenset(
    {"mathlib_available", "local_task", "local_compiled", "blocked"}
)
LS_SOURCE_IDENTITY = ls_contract.SOURCE_IDENTITY
LS_ROUTE_ID = ls_contract.ROUTE_ID
LS_ALLOWED_AXIOMS = ls_contract.ALLOWED_AXIOMS
LS_AGGREGATE_MODULE = ls_contract.AGGREGATE_MODULE
LS_NODE_BUILD_TARGETS = {node.node_id: node.build_target for node in ls_contract.NODES}
RECEIPT_FIELDS = frozenset(
    {
        "schema_version",
        "route_id",
        "source_node_id",
        "status",
        "reason",
        "build_target",
        "expected_lean_declaration",
        "allowed_axioms",
        "observed_axioms",
        "task_sha256",
        "source_slice_sha256",
        "command_sha256",
        "stdout_sha256",
        "stderr_sha256",
        "axiom_audit_sha256",
        "result_sha256",
        "module_sha256",
    }
)
TASK_FIELDS = frozenset(
    {
        "schema_version",
        "route_id",
        "source_node_id",
        "build_target",
        "expected_lean_declaration",
        "max_output_bytes",
        "timeout_seconds",
    }
)
SOURCE_SLICE_FIELDS = frozenset(
    {
        "schema_version",
        "node_id",
        "source_locator",
        "statement_sha256",
        "lean_name",
        "dependency_ids",
    }
)
RESULT_FIELDS = frozenset({"schema_version", "status", "reason"})
COMMAND_V1_FIELDS = frozenset(
    {"schema_version", "argv", "cwd", "exit_code", "cache_policy", "elan_home"}
)
COMMAND_V2_FIELDS = (COMMAND_V1_FIELDS - {"elan_home"}) | frozenset(
    {
        "elan_toolchain",
        "env",
        "output_cap_bytes",
        "timeout_seconds",
        "wrapper_sha256",
        "lake_manifest_sha256",
        "dependency_cache_metadata_sha256",
        "required_mathlib_artifacts_sha256",
        "local_source_closure_sha256",
    }
)
MEMBER_PATHS = {
    "task_sha256": "task.json",
    "source_slice_sha256": "source-slice.json",
    "result_sha256": "result.json",
    "command_sha256": "build/command.json",
    "stdout_sha256": "build/stdout.log",
    "stderr_sha256": "build/stderr.log",
    "axiom_audit_sha256": "build/axioms.txt",
}
ATTEMPT_DIRECTORIES = frozenset({"build"})
ATTEMPT_FILES = frozenset({"receipt.json", *MEMBER_PATHS.values()})
ATTEMPT_NAME = re.compile(r"attempt-[0-9]+\Z")
AXIOM_AUDIT = re.compile(
    r"\A'(?P<declaration>[^'\r\n]+)' depends on axioms: \[(?P<axioms>.*?)\]\s*\Z",
    re.DOTALL,
)
AXIOM_AUDIT_NONE = re.compile(
    r"\A'(?P<declaration>[^'\r\n]+)' does not depend on any axioms\s*\Z"
)
LEAN_MODULE_NAME = re.compile(
    r"[A-Za-z_][A-Za-z0-9_']*(?:\.[A-Za-z_][A-Za-z0-9_']*)*\Z"
)
LS_AUDIT_CALL_TOKEN = re.compile(
    r"(?<![\w'])[_A-Za-z][A-Za-z0-9_']*(?:\.[_A-Za-z][A-Za-z0-9_']*)*(?![\w'!?])"
)
LS_AUDIT_TOPLEVEL_COMMAND = re.compile(r"^[A-Za-z_]")


@dataclass(frozen=True)
class LSGraphRow:
    node_id: str
    source_locator: str
    statement_sha256: str
    lean_name: str
    dependencies: tuple[str, ...]
    role: str
    status: str
    receipt_sha256: str | None = None
    blocked_reason: str | None = None
    failed_reason: str | None = None


@dataclass(frozen=True)
class _PinnedDirectory:
    path: Path
    descriptor: int
    device: int
    inode: int
    parent: "_PinnedDirectory | None" = None
    entry_name: str | None = None


@dataclass(frozen=True)
class _FileSnapshot:
    name: str
    data: bytes
    device: int
    inode: int
    mode: int
    size: int
    modified_ns: int
    changed_ns: int


@dataclass(frozen=True)
class _LocalSourceClosure:
    modules: frozenset[str]
    sources: tuple[tuple[str, bytes], ...]
    sha256: str


def _metadata_tuple(metadata: os.stat_result) -> tuple[int, int, int, int, int, int]:
    return (
        metadata.st_dev,
        metadata.st_ino,
        stat.S_IFMT(metadata.st_mode),
        metadata.st_size,
        metadata.st_mtime_ns,
        metadata.st_ctime_ns,
    )


def _require_validation_primitives() -> None:
    if (
        not hasattr(os, "O_DIRECTORY")
        or not hasattr(os, "O_NOFOLLOW")
        or os.open not in os.supports_dir_fd
        or os.stat not in os.supports_dir_fd
        or os.stat not in os.supports_follow_symlinks
    ):
        raise protocol.ValidationError(
            "host runtime lacks descriptor-relative LS validation primitives"
        )


def _directory_flags() -> int:
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    return flags


def _pin_absolute_chain(path: Path, label: str) -> tuple[_PinnedDirectory, ...]:
    """Pin every component of an absolute path without following symlinks."""

    _require_validation_primitives()
    absolute = Path(os.path.abspath(os.fspath(path)))
    if not absolute.is_absolute():
        raise protocol.ValidationError(f"{label} must be absolute")
    pins: list[_PinnedDirectory] = []
    try:
        descriptor = os.open(absolute.anchor, _directory_flags())
        metadata = os.fstat(descriptor)
        root = _PinnedDirectory(
            path=Path(absolute.anchor),
            descriptor=descriptor,
            device=metadata.st_dev,
            inode=metadata.st_ino,
        )
        pins.append(root)
        current = root
        current_path = root.path
        for part in absolute.parts[1:]:
            current_path /= part
            current = _pin_directory_at(current, part, current_path, label)
            pins.append(current)
        _require_pinned_directory(current, label)
        return tuple(pins)
    except BaseException:
        _close_pinned_directories(pins)
        raise


def _pin_directory_at(
    parent: _PinnedDirectory, name: str, path: Path, label: str
) -> _PinnedDirectory:
    _safe_entry_name(name, label)
    _require_pinned_directory(parent, f"{label} parent")
    try:
        descriptor = os.open(name, _directory_flags(), dir_fd=parent.descriptor)
    except OSError as error:
        try:
            metadata = os.stat(name, dir_fd=parent.descriptor, follow_symlinks=False)
        except OSError:
            metadata = None
        if metadata is not None and stat.S_ISLNK(metadata.st_mode):
            raise protocol.ValidationError(f"{label} cannot be a symlink") from error
        raise protocol.ValidationError(f"cannot pin {label}: {error}") from error
    try:
        metadata = os.fstat(descriptor)
        pinned = _PinnedDirectory(
            path=path,
            descriptor=descriptor,
            device=metadata.st_dev,
            inode=metadata.st_ino,
            parent=parent,
            entry_name=name,
        )
        _require_pinned_directory(pinned, label)
        return pinned
    except BaseException:
        os.close(descriptor)
        raise


def _require_pinned_directory(directory: _PinnedDirectory, label: str) -> None:
    try:
        pinned = os.fstat(directory.descriptor)
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot inspect pinned {label}: {error}"
        ) from error
    if not stat.S_ISDIR(pinned.st_mode) or (pinned.st_dev, pinned.st_ino) != (
        directory.device,
        directory.inode,
    ):
        raise protocol.ValidationError(f"pinned {label} identity changed")
    if directory.parent is None:
        return
    assert directory.entry_name is not None
    try:
        current = os.stat(
            directory.entry_name,
            dir_fd=directory.parent.descriptor,
            follow_symlinks=False,
        )
    except OSError as error:
        raise protocol.ValidationError(
            f"pinned {label} identity changed: {error}"
        ) from error
    if stat.S_ISLNK(current.st_mode):
        raise protocol.ValidationError(f"pinned {label} changed to a symlink")
    if not stat.S_ISDIR(current.st_mode) or (current.st_dev, current.st_ino) != (
        directory.device,
        directory.inode,
    ):
        raise protocol.ValidationError(f"pinned {label} identity changed")


def _close_pinned_directories(directories: Sequence[_PinnedDirectory]) -> None:
    first_error: OSError | None = None
    for directory in reversed(tuple(directories)):
        try:
            os.close(directory.descriptor)
        except OSError as error:
            if first_error is None:
                first_error = error
    if first_error is not None:
        raise protocol.ValidationError(
            f"cannot close pinned LS validation directory: {first_error}"
        ) from first_error


def _safe_entry_name(name: str, label: str) -> None:
    if not isinstance(name, str) or name in {"", ".", ".."} or "/" in name:
        raise protocol.ValidationError(f"{label} has an unsafe entry name")


def _read_file_at(
    directory: _PinnedDirectory, name: str, label: str, maximum: int
) -> bytes:
    _safe_entry_name(name, label)
    _require_pinned_directory(directory, f"{label} parent")
    flags = os.O_RDONLY | os.O_NOFOLLOW
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    try:
        descriptor = os.open(name, flags, dir_fd=directory.descriptor)
    except FileNotFoundError as error:
        raise protocol.ValidationError(f"{label} does not exist") from error
    except OSError as error:
        try:
            metadata = os.stat(name, dir_fd=directory.descriptor, follow_symlinks=False)
        except OSError:
            metadata = None
        if metadata is not None and stat.S_ISLNK(metadata.st_mode):
            raise protocol.ValidationError(f"{label} cannot be a symlink") from error
        if getattr(error, "errno", None) == getattr(os, "ELOOP", 62):
            raise protocol.ValidationError(f"{label} cannot be a symlink") from error
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    try:
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode):
            raise protocol.ValidationError(f"{label} must be a regular file")
        if before.st_nlink != 1:
            raise protocol.ValidationError(f"{label} must have exactly one hard link")
        if before.st_size > maximum:
            raise protocol.ValidationError(f"{label} exceeds byte cap")
        chunks: list[bytes] = []
        size = 0
        while True:
            chunk = os.read(descriptor, min(64 * 1024, maximum + 1 - size))
            if not chunk:
                break
            chunks.append(chunk)
            size += len(chunk)
            if size > maximum:
                raise protocol.ValidationError(f"{label} exceeds byte cap")
        after = os.fstat(descriptor)
        current = os.stat(name, dir_fd=directory.descriptor, follow_symlinks=False)
        if after.st_nlink != 1 or current.st_nlink != 1:
            raise protocol.ValidationError(f"{label} must have exactly one hard link")
        if (
            stat.S_ISLNK(current.st_mode)
            or _metadata_tuple(before) != _metadata_tuple(after)
            or _metadata_tuple(after) != _metadata_tuple(current)
        ):
            raise protocol.ValidationError(f"{label} changed while being read")
        _require_pinned_directory(directory, f"{label} parent")
        return b"".join(chunks)
    except protocol.ValidationError:
        raise
    except OSError as error:
        raise protocol.ValidationError(f"cannot read {label}: {error}") from error
    finally:
        os.close(descriptor)


def _read_relative_file(
    root: _PinnedDirectory, relative_path: str, label: str, maximum: int
) -> bytes:
    path = PurePosixPath(relative_path)
    if path.is_absolute() or any(part in {"", ".", ".."} for part in path.parts):
        raise protocol.ValidationError(f"{label} has an unsafe relative path")
    current = root
    opened: list[_PinnedDirectory] = []
    try:
        for part in path.parts[:-1]:
            current = _pin_directory_at(
                current, part, current.path / part, f"{label} directory"
            )
            opened.append(current)
        return _read_file_at(current, path.name, label, maximum)
    finally:
        _close_pinned_directories(opened)


def load_route_graph(path: Path) -> tuple[LSGraphRow, ...]:
    value = _read_json_object(path, "LS source graph")
    _require_fields(value, GRAPH_FIELDS, "LS source graph")
    _require_equal(
        value["schema_version"], ls_contract.SCHEMA_VERSION, "schema_version"
    )
    _require_equal(value["source_id"], ls_contract.SOURCE_ID, "source_id")
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
    _validate_graph_contract(rows, allow_legacy=True)
    _reject_cycles(rows)
    terminal = [row for row in rows if row.role == "terminal"]
    if len(terminal) != 1:
        raise protocol.ValidationError(
            "LS graph must contain exactly one terminal node"
        )
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


def _is_legacy_contract_row(
    row: LSGraphRow, expected: ls_contract.LSNodeContract
) -> bool:
    return (
        row.dependencies == expected.legacy_dependencies
        and expected.legacy_status is not None
        and row.role == expected.role
        and row.source_locator == expected.source_locator
        and row.statement_sha256 == expected.statement_sha256
        and row.lean_name == (expected.legacy_graph_lean_name or expected.declaration)
        and row.status == expected.legacy_status
        and row.receipt_sha256 == expected.legacy_receipt_sha256
        and row.blocked_reason == expected.legacy_blocked_reason
        and row.failed_reason is None
    )


def _validate_graph_contract(
    rows: Sequence[LSGraphRow], *, allow_legacy: bool, require_complete: bool = True
) -> None:
    if require_complete and len(rows) != len(ls_contract.NODES):
        raise protocol.ValidationError("canonical LS graph requires exactly six nodes")
    expected_order = tuple(
        node_id
        for node_id in ls_contract.NODE_ORDER
        if node_id in {row.node_id for row in rows}
    )
    if tuple(row.node_id for row in rows) != expected_order:
        raise protocol.ValidationError("canonical LS graph node order is invalid")
    for row in rows:
        expected = ls_contract.BY_ID.get(row.node_id)
        if expected is None:
            raise protocol.ValidationError(
                f"canonical LS graph node is unknown: {row.node_id}"
            )
        canonical_identity = (
            row.dependencies,
            row.role,
            row.source_locator,
            row.statement_sha256,
            row.lean_name,
        )
        expected_canonical_identity = (
            expected.dependencies,
            expected.role,
            expected.source_locator,
            expected.statement_sha256,
            expected.declaration,
        )
        if canonical_identity == expected_canonical_identity:
            continue
        if allow_legacy and _is_legacy_contract_row(row, expected):
            continue
        raise protocol.ValidationError(
            f"canonical LS graph identity is invalid: {row.node_id}"
        )


def validate_committed_receipts(
    rows: tuple[LSGraphRow, ...],
    formal_target_root: Path,
    lean_root: Path,
) -> dict[str, dict[str, object]]:
    """Validate every graph-published LS receipt against committed bytes.

    ``lean_root`` is the repository checkout root. Receipt ``build_target``
    paths remain repository-relative so the validator binds the receipt to the
    current Lean module instead of to a copied path named by the caller.
    """

    target_root = Path(os.path.abspath(os.fspath(formal_target_root)))
    repository_root = Path(os.path.abspath(os.fspath(lean_root)))
    by_id: dict[str, LSGraphRow] = {}
    for row in rows:
        if not isinstance(row, LSGraphRow):
            raise protocol.ValidationError("LS committed receipt row is invalid")
        node_id = formal_target._runtime_id(row.node_id, "node_id")
        if node_id in by_id:
            raise protocol.ValidationError("duplicate LS graph node_id")
        if len(set(row.dependencies)) != len(row.dependencies):
            raise protocol.ValidationError(
                "LS graph node dependencies cannot contain duplicates"
            )
        if row.status in {"mapped", "blocked"} and row.receipt_sha256 is not None:
            raise protocol.ValidationError(
                f"{row.status} LS graph node rejects receipt_sha256"
            )
        if row.status in {"passed", "failed"} and row.receipt_sha256 is None:
            raise protocol.ValidationError(
                f"{row.status} LS graph node requires receipt_sha256"
            )
        by_id[node_id] = row

    if len(rows) == len(ls_contract.NODES) and all(
        row.status == "passed" for row in rows
    ):
        _validate_graph_contract(rows, allow_legacy=False)

    # Receipt schema selection below determines whether the exact migration
    # legacy declaration or the canonical v2 declaration is admissible. Keep
    # this entrypoint's earlier semantic diagnostics intact for constructed
    # rows while load_route_graph remains the strict six-node boundary.

    target_chain = _pin_absolute_chain(target_root, "LS formal target root")
    repository_chain = _pin_absolute_chain(repository_root, "LS repository root")
    target_directory = target_chain[-1]
    repository_directory = repository_chain[-1]
    proof_slices: _PinnedDirectory | None = None
    try:
        proof_slices = _pin_directory_at(
            target_directory,
            "proof-slices",
            target_root / "proof-slices",
            "LS proof-slices root",
        )
        validated: dict[str, dict[str, object]] = {}
        visiting: set[str] = set()

        def validate_node(row: LSGraphRow) -> None:
            if row.node_id in validated:
                return
            if row.node_id in visiting:
                raise protocol.ValidationError("LS graph dependency cycle")
            visiting.add(row.node_id)
            try:
                for dependency_id in row.dependencies:
                    dependency = by_id.get(dependency_id)
                    if dependency is None:
                        raise protocol.ValidationError(
                            f"passed LS graph node {row.node_id} predecessor is missing: "
                            f"{dependency_id}"
                        )
                    if dependency.status != "passed":
                        raise protocol.ValidationError(
                            f"passed LS graph node {row.node_id} predecessor "
                            f"{dependency_id} is not passed"
                        )
                    try:
                        validate_node(dependency)
                    except protocol.ValidationError as error:
                        raise protocol.ValidationError(
                            f"passed LS graph node {row.node_id} predecessor "
                            f"{dependency_id} is not mechanically validated: {error}"
                        ) from error
                validated[row.node_id] = _validate_committed_receipt(
                    row,
                    proof_slices,
                    repository_directory,
                    None,
                    None,
                )
            finally:
                visiting.remove(row.node_id)

        for row in rows:
            if row.status == "passed":
                validate_node(row)
            elif row.status == "failed":
                validated[row.node_id] = _validate_committed_receipt(
                    row,
                    proof_slices,
                    repository_directory,
                    None,
                    None,
                )

        result = {
            row.node_id: validated[row.node_id]
            for row in rows
            if row.status in {"passed", "failed"}
        }
        canonical_passed_v2 = (
            len(rows) == len(ls_contract.NODES)
            and tuple(row.node_id for row in rows) == ls_contract.NODE_ORDER
            and all(row.status == "passed" for row in rows)
            and all(
                receipt.get("command_schema_version") == "crouzeix-ls-lean-command/v2"
                for receipt in result.values()
            )
        )
        if canonical_passed_v2:
            identities = {
                (
                    receipt["command_sha256"],
                    receipt["stdout_sha256"],
                    receipt["stderr_sha256"],
                    receipt["local_source_closure_sha256"],
                )
                for receipt in result.values()
            }
            if len(identities) != 1:
                raise protocol.ValidationError(
                    "canonical six LS receipts must share identical aggregate build evidence"
                )
        for receipt in result.values():
            receipt.pop("command_schema_version", None)
        return result
    finally:
        if proof_slices is not None:
            _close_pinned_directories((proof_slices,))
        _close_pinned_directories(repository_chain)
        _close_pinned_directories(target_chain)


def _validate_committed_receipt(
    row: LSGraphRow,
    proof_slices: _PinnedDirectory,
    repository_root: _PinnedDirectory,
    legacy_reachable_modules: frozenset[str] | None = None,
    current_source_closure: _LocalSourceClosure | None = None,
) -> dict[str, object]:
    attempt, receipt_bytes = _resolve_committed_attempt(row, proof_slices)
    try:
        build = _validate_attempt_members(attempt, row.node_id)
        receipt = _validate_receipt_object(
            _json_object_from_bytes(
                receipt_bytes, f"LS committed receipt for node {row.node_id}"
            ),
            row,
        )

        members: dict[str, bytes] = {}
        for digest_field, relative_path in MEMBER_PATHS.items():
            label = f"{digest_field} member for node {row.node_id}"
            member_path = PurePosixPath(relative_path)
            data = (
                _read_file_at(build, member_path.name, label, MAX_MEMBER_BYTES)
                if member_path.parent.as_posix() == "build"
                else _read_file_at(attempt, member_path.name, label, MAX_MEMBER_BYTES)
            )
            _require_bytes_digest(data, str(receipt[digest_field]), digest_field)
            members[digest_field] = data

        task = _validate_task_object(
            _json_object_from_bytes(
                members["task_sha256"], f"LS committed task for node {row.node_id}"
            ),
            row,
            receipt,
        )
        for digest_field, relative_path in (
            ("stdout_sha256", "build/stdout.log"),
            ("stderr_sha256", "build/stderr.log"),
        ):
            if len(members[digest_field]) > task["max_output_bytes"]:
                raise protocol.ValidationError(
                    f"{relative_path} exceeds task max_output_bytes"
                )
        source_slice = _validate_source_slice_object(
            _json_object_from_bytes(
                members["source_slice_sha256"],
                f"LS committed source slice for node {row.node_id}",
            ),
            row,
        )
        result = _validate_result_object(
            _json_object_from_bytes(
                members["result_sha256"],
                f"LS committed result for node {row.node_id}",
            )
        )
        command = _json_object_from_bytes(
            members["command_sha256"],
            f"LS committed command for node {row.node_id}",
        )
        command_exit_code = _validate_command_object(
            command, repository_root, current_source_closure
        )
        _validate_execution_evidence(
            command_exit_code,
            members["stdout_sha256"],
            members["axiom_audit_sha256"],
            receipt,
            row.node_id,
            str(command["schema_version"]),
        )

        if task["build_target"] != receipt["build_target"]:
            raise protocol.ValidationError(
                "LS committed task build_target does not match receipt build_target"
            )
        if source_slice["dependency_ids"] != list(row.dependencies):
            raise protocol.ValidationError(
                "LS committed source-slice dependencies do not match graph node"
            )
        if result["status"] != receipt["status"]:
            raise protocol.ValidationError(
                "LS committed result status does not match receipt status"
            )

        expected_contract = ls_contract.BY_ID[row.node_id]
        if command["schema_version"] == "crouzeix-ls-lean-command/v1":
            expected_build_target = (
                expected_contract.legacy_build_target or expected_contract.build_target
            )
            if legacy_reachable_modules is None:
                legacy_reachable_modules = _active_import_closure_at(
                    repository_root, ("Crouzeix",)
                ).modules
            reachable_modules = legacy_reachable_modules
        else:
            expected_build_target = expected_contract.build_target
            if row.lean_name != expected_contract.declaration:
                raise protocol.ValidationError(
                    "canonical LS graph declaration is required for v2 receipt: "
                    f"{row.node_id}"
                )
            _validate_provider_independence(repository_root.path)
            if current_source_closure is None:
                current_source_closure = _active_import_closure_at(
                    repository_root, (LS_AGGREGATE_MODULE,)
                )
            reachable_modules = current_source_closure.modules
        if receipt["build_target"] != expected_build_target:
            raise protocol.ValidationError(
                f"LS committed build_target does not match LS node {row.node_id}"
            )
        module_name = _build_target_module(str(receipt["build_target"]))
        if module_name not in reachable_modules:
            raise protocol.ValidationError(
                "LS committed build_target is not reachable from "
                + (
                    "Crouzeix"
                    if command["schema_version"] == "crouzeix-ls-lean-command/v1"
                    else LS_AGGREGATE_MODULE
                )
            )

        module_bytes = _read_relative_file(
            repository_root,
            str(receipt["build_target"]),
            f"module_sha256 member for node {row.node_id}",
            MAX_MEMBER_BYTES,
        )
        _require_bytes_digest(
            module_bytes, str(receipt["module_sha256"]), "module_sha256"
        )
        receipt["command_schema_version"] = command["schema_version"]
        if command["schema_version"] == "crouzeix-ls-lean-command/v2":
            receipt["local_source_closure_sha256"] = command[
                "local_source_closure_sha256"
            ]
        return receipt
    finally:
        if "build" in locals():
            _close_pinned_directories((build,))
        _close_pinned_directories((attempt,))
        if attempt.parent is not None:
            _close_pinned_directories((attempt.parent,))


def _validate_provider_independence(repository_root: Path) -> None:
    try:
        provider_independence.audit_provider_independence(
            repository_root / "formalization/lean",
            (LS_AGGREGATE_MODULE,),
            ls_contract.FORBIDDEN_MODULES,
            forbidden_prefixes=ls_contract.FORBIDDEN_PREFIXES,
        )
    except provider_independence.ProviderIndependenceError as error:
        raise protocol.ValidationError(
            f"LS receipt target is not provider independent: {error}"
        ) from error


def _resolve_committed_attempt(
    row: LSGraphRow, proof_slices: _PinnedDirectory
) -> tuple[_PinnedDirectory, bytes]:
    if row.receipt_sha256 is None:
        raise protocol.ValidationError(
            f"{row.status} LS graph node {row.node_id} requires receipt_sha256"
        )
    expected = formal_target._digest(row.receipt_sha256, "receipt_sha256")
    node_root = _pin_directory_at(
        proof_slices,
        row.node_id,
        proof_slices.path / row.node_id,
        f"LS proof-slice directory for node {row.node_id}",
    )
    attempts: list[str] = []
    try:
        entry_count = 0
        with os.scandir(node_root.descriptor) as entries:
            for entry in entries:
                entry_count += 1
                if entry_count > MAX_ATTEMPTS:
                    raise protocol.ValidationError(
                        f"LS attempt count exceeds cap for node {row.node_id}"
                    )
                if not entry.name.startswith("attempt-"):
                    continue
                metadata = entry.stat(follow_symlinks=False)
                if stat.S_ISLNK(metadata.st_mode):
                    raise protocol.ValidationError(
                        f"LS attempt cannot be a symlink: {entry.name}"
                    )
                if stat.S_ISDIR(metadata.st_mode):
                    attempts.append(entry.name)
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot inspect LS proof-slice directory for node {row.node_id}: {error}"
        ) from error

    matches: list[tuple[_PinnedDirectory, bytes]] = []
    aggregate_receipt_bytes = 0
    for attempt_name in sorted(attempts):
        attempt: _PinnedDirectory | None = None
        try:
            attempt = _pin_directory_at(
                node_root,
                attempt_name,
                node_root.path / attempt_name,
                f"LS attempt directory for node {row.node_id}",
            )
            candidate_bytes = _read_candidate_receipt(attempt)
            if candidate_bytes is not None:
                aggregate_receipt_bytes += len(candidate_bytes)
                if aggregate_receipt_bytes > MAX_ATTEMPT_RECEIPT_BYTES:
                    raise protocol.ValidationError(
                        "LS attempt aggregate receipt bytes exceed cap for node "
                        f"{row.node_id}"
                    )
            if (
                candidate_bytes is None
                or protocol.sha256_bytes(candidate_bytes) != expected
            ):
                continue
            if ATTEMPT_NAME.fullmatch(attempt_name) is None:
                raise protocol.ValidationError(
                    f"matching LS attempt directory name is unsafe: {attempt_name}"
                )
            receipt_bytes = _read_file_at(
                attempt,
                "receipt.json",
                f"LS committed receipt for node {row.node_id}",
                MAX_JSON_BYTES,
            )
            _require_bytes_digest(receipt_bytes, expected, "receipt_sha256")
            matches.append((attempt, receipt_bytes))
            attempt = None
        finally:
            if attempt is not None:
                _close_pinned_directories((attempt,))

    if len(matches) != 1:
        _close_pinned_directories(tuple(attempt for attempt, _ in matches))
        _close_pinned_directories((node_root,))
        raise protocol.ValidationError(
            "exactly one LS committed receipt_sha256 must match an attempt for "
            f"node {row.node_id}; found {len(matches)}"
        )
    _require_pinned_directory(node_root, f"LS proof-slice directory for {row.node_id}")
    return matches[0]


def _validate_receipt_object(
    value: Mapping[str, Any], row: LSGraphRow
) -> dict[str, object]:
    _require_fields(value, RECEIPT_FIELDS, "LS committed receipt")
    _require_equal(
        value["schema_version"],
        "crouzeix-ls-proof-slice-receipt/v1",
        "schema_version",
    )
    _require_equal(value["route_id"], LS_ROUTE_ID, "route_id")
    source_node_id = formal_target._runtime_id(
        value["source_node_id"], "source_node_id"
    )
    if source_node_id != row.node_id:
        raise protocol.ValidationError(
            "LS committed receipt source_node_id does not match graph node"
        )
    status = _enum(value["status"], NODE_STATUSES, "receipt status")
    if status != row.status:
        raise protocol.ValidationError(
            "LS committed receipt status does not match graph node status"
        )
    if status not in {"passed", "failed"}:
        raise protocol.ValidationError(
            "LS committed receipt must have passed or failed status"
        )
    declaration = _bounded_string(
        value["expected_lean_declaration"], "expected_lean_declaration", 1, 256
    )
    if declaration != row.lean_name:
        raise protocol.ValidationError(
            "LS committed receipt declaration does not match graph node declaration"
        )
    allowed_axioms = _unique_string_list(
        value["allowed_axioms"], "allowed_axioms", maximum=128
    )
    observed_axioms = _unique_string_list(
        value["observed_axioms"], "observed_axioms", maximum=128
    )
    if allowed_axioms != list(LS_ALLOWED_AXIOMS):
        raise protocol.ValidationError(
            "LS committed receipt allowed axioms do not match LS policy"
        )
    unexpected = sorted(set(observed_axioms) - set(allowed_axioms))
    if status == "passed" and unexpected:
        raise protocol.ValidationError(
            "LS committed passed receipt has disallowed observed axioms: "
            + ", ".join(unexpected)
        )

    receipt: dict[str, object] = {
        "schema_version": value["schema_version"],
        "route_id": value["route_id"],
        "source_node_id": source_node_id,
        "status": status,
        "reason": _bounded_string(value["reason"], "reason", 1, 4096),
        "build_target": _safe_build_target(value["build_target"]),
        "expected_lean_declaration": declaration,
        "allowed_axioms": allowed_axioms,
        "observed_axioms": observed_axioms,
    }
    for digest_field in (*MEMBER_PATHS, "module_sha256"):
        receipt[digest_field] = formal_target._digest(value[digest_field], digest_field)
    return receipt


def _validate_task_object(
    value: Mapping[str, Any], row: LSGraphRow, receipt: Mapping[str, object]
) -> dict[str, object]:
    _require_fields(value, TASK_FIELDS, "LS committed task")
    _require_equal(
        value["schema_version"], "crouzeix-ls-proof-slice-task/v1", "schema_version"
    )
    _require_equal(value["route_id"], LS_ROUTE_ID, "route_id")
    source_node_id = formal_target._runtime_id(
        value["source_node_id"], "source_node_id"
    )
    if source_node_id != row.node_id:
        raise protocol.ValidationError(
            "LS committed task source_node_id does not match graph node"
        )
    declaration = _bounded_string(
        value["expected_lean_declaration"], "expected_lean_declaration", 1, 256
    )
    if declaration != receipt["expected_lean_declaration"]:
        raise protocol.ValidationError(
            "LS committed task declaration does not match receipt declaration"
        )
    max_output_bytes = _integer(
        value["max_output_bytes"], "max_output_bytes", 1, MAX_MEMBER_BYTES
    )
    if max_output_bytes != MAX_JSON_BYTES:
        raise protocol.ValidationError("LS committed task max_output_bytes is invalid")
    timeout_seconds = _integer(value["timeout_seconds"], "timeout_seconds", 1, 86400)
    if timeout_seconds != 3600:
        raise protocol.ValidationError("LS committed task timeout_seconds is invalid")
    return {
        "schema_version": value["schema_version"],
        "route_id": value["route_id"],
        "source_node_id": source_node_id,
        "build_target": _safe_build_target(value["build_target"]),
        "expected_lean_declaration": declaration,
        "max_output_bytes": max_output_bytes,
        "timeout_seconds": timeout_seconds,
    }


def _validate_source_slice_object(
    value: Mapping[str, Any], row: LSGraphRow
) -> dict[str, object]:
    _require_fields(value, SOURCE_SLICE_FIELDS, "LS committed source slice")
    _require_equal(
        value["schema_version"], "crouzeix-ls-source-slice/v1", "schema_version"
    )
    node_id = formal_target._runtime_id(value["node_id"], "node_id")
    if node_id != row.node_id:
        raise protocol.ValidationError(
            "LS committed source-slice node_id does not match graph node"
        )
    lean_name = _bounded_string(value["lean_name"], "lean_name", 1, 256)
    if lean_name != row.lean_name:
        raise protocol.ValidationError(
            "LS committed source-slice declaration does not match graph node"
        )
    source_locator = _source_locator(value["source_locator"], "source_locator")
    if source_locator != row.source_locator:
        raise protocol.ValidationError(
            "LS committed source-slice locator does not match graph node"
        )
    statement_sha256 = formal_target._digest(
        value["statement_sha256"], "statement_sha256"
    )
    if statement_sha256 != row.statement_sha256:
        raise protocol.ValidationError(
            "LS committed source-slice statement digest does not match graph node"
        )
    return {
        "schema_version": value["schema_version"],
        "node_id": node_id,
        "lean_name": lean_name,
        "source_locator": source_locator,
        "statement_sha256": statement_sha256,
        "dependency_ids": _runtime_id_list(value["dependency_ids"], "dependency_ids"),
    }


def _validate_result_object(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, RESULT_FIELDS, "LS committed result")
    _require_equal(
        value["schema_version"], "crouzeix-ls-proof-slice-result/v1", "schema_version"
    )
    return {
        "schema_version": value["schema_version"],
        "status": _enum(value["status"], NODE_STATUSES, "result status"),
        "reason": _bounded_string(value["reason"], "reason", 1, 4096),
    }


def _validate_command_object(
    value: Mapping[str, Any],
    repository_root: Path | _PinnedDirectory | None = None,
    source_closure: _LocalSourceClosure | None = None,
) -> int:
    schema_version = value.get("schema_version")
    if schema_version == "crouzeix-ls-lean-command/v1":
        _require_fields(value, COMMAND_V1_FIELDS, "LS committed command")
    elif schema_version == "crouzeix-ls-lean-command/v2":
        _require_fields(value, COMMAND_V2_FIELDS, "LS committed command")
        if value["elan_toolchain"] != "leanprover/lean4:v4.32.1":
            raise protocol.ValidationError(
                "LS committed command elan_toolchain is invalid"
            )
        if _integer(value["timeout_seconds"], "timeout_seconds", 1, 86400) != 3600:
            raise protocol.ValidationError(
                "LS committed command timeout_seconds is invalid"
            )
        if (
            _integer(value["output_cap_bytes"], "output_cap_bytes", 1, MAX_MEMBER_BYTES)
            != 1048576
        ):
            raise protocol.ValidationError(
                "LS committed command output_cap_bytes is invalid"
            )
        if value["env"] != {
            "ELAN_HOME": "/private/tmp/harp-mathematical-foundations-elan",
            "ELAN_TOOLCHAIN": "leanprover/lean4:v4.32.1",
            "PATH": (
                "/private/tmp/harp-mathematical-foundations-elan/toolchains/"
                "leanprover--lean4---v4.32.1/bin:/usr/bin:/bin"
            ),
        }:
            raise protocol.ValidationError("LS committed command env is invalid")
        # Cache metadata and artifact digests describe the publisher's
        # machine-local execution snapshot. They remain digest-validated receipt
        # claims; portable validation intentionally does not recreate that cache.
        for field in (
            "wrapper_sha256",
            "lake_manifest_sha256",
            "dependency_cache_metadata_sha256",
            "required_mathlib_artifacts_sha256",
            "local_source_closure_sha256",
        ):
            formal_target._digest(value[field], field)
        if repository_root is None:
            raise protocol.ValidationError(
                "LS committed command v2 requires repository root validation"
            )
        owned_chain: tuple[_PinnedDirectory, ...] = ()
        if isinstance(repository_root, _PinnedDirectory):
            repository_directory = repository_root
        else:
            owned_chain = _pin_absolute_chain(
                Path(repository_root), "LS committed command repository root"
            )
            repository_directory = owned_chain[-1]
        try:
            for field, relative_path in (
                ("wrapper_sha256", "scripts/check_lean_library.sh"),
                ("lake_manifest_sha256", "formalization/lean/lake-manifest.json"),
            ):
                data = _read_relative_file(
                    repository_directory, relative_path, field, MAX_MEMBER_BYTES
                )
                _require_bytes_digest(data, str(value[field]), field)
        finally:
            _close_pinned_directories(owned_chain)
        if source_closure is None:
            _validate_provider_independence(repository_directory.path)
            source_closure = _active_import_closure_at(repository_directory)
        _require_equal(
            formal_target._digest(
                value["local_source_closure_sha256"],
                "local_source_closure_sha256",
            ),
            source_closure.sha256,
            "local_source_closure_sha256",
        )
        _require_equal(
            formal_target._digest(
                value["required_mathlib_artifacts_sha256"],
                "required_mathlib_artifacts_sha256",
            ),
            _required_mathlib_artifacts_digest(
                repository_directory.path, source_closure
            ),
            "required_mathlib_artifacts_sha256",
        )
    else:
        raise protocol.ValidationError("LS committed command schema_version is invalid")
    expected_target = (
        "Crouzeix"
        if schema_version == "crouzeix-ls-lean-command/v1"
        else LS_AGGREGATE_MODULE
    )
    if value["argv"] != ["scripts/check_lean_library.sh", expected_target]:
        raise protocol.ValidationError("LS committed command argv is invalid")
    if value["cwd"] != ".":
        raise protocol.ValidationError("LS committed command cwd is invalid")
    exit_code = _integer(value["exit_code"], "exit_code", -255, 255)
    _bounded_string(value["cache_policy"], "cache_policy", 1, 4096)
    if (
        schema_version == "crouzeix-ls-lean-command/v1"
        and value["elan_home"] != "/private/tmp/harp-mathematical-foundations-elan"
    ):
        raise protocol.ValidationError("LS committed command elan_home is invalid")
    return exit_code


def _validate_execution_evidence(
    command_exit_code: int,
    stdout_data: bytes,
    axiom_data: bytes,
    receipt: Mapping[str, object],
    node_id: str,
    command_schema_version: str,
) -> None:
    if receipt["status"] == "failed":
        if (
            command_exit_code == 0
            or axiom_data != b""
            or receipt["observed_axioms"] != []
        ):
            raise protocol.ValidationError(
                "LS committed failed receipt lacks coherent failure evidence"
            )
        return
    if command_exit_code != 0:
        raise protocol.ValidationError(
            "LS committed passed receipt requires zero command exit_code"
        )
    _validate_aggregate_stdout(stdout_data, node_id, command_schema_version)
    _validate_axiom_audit(axiom_data, receipt, node_id)


def _validate_aggregate_stdout(
    data: bytes, node_id: str, command_schema_version: str
) -> None:
    try:
        lines = data.decode("utf-8").splitlines()
    except UnicodeDecodeError as error:
        raise protocol.ValidationError(
            f"cannot decode LS aggregate stdout for node {node_id}: {error}"
        ) from error
    target = (
        "Crouzeix"
        if command_schema_version == "crouzeix-ls-lean-command/v1"
        else LS_AGGREGATE_MODULE
    )
    required = (
        f"[lean] target={target}",
        "[lean] root=formalization/lean",
        "[lean] outcome=passed",
    )
    if any(lines.count(marker) != 1 for marker in required) or any(
        line.startswith("[lean] outcome=") and line != required[-1] for line in lines
    ):
        raise protocol.ValidationError(
            "LS committed aggregate stdout lacks exact success markers"
        )


def _validate_axiom_audit(
    data: bytes, receipt: Mapping[str, object], node_id: str
) -> None:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise protocol.ValidationError(
            f"cannot decode LS axiom audit for node {node_id}: {error}"
        ) from error
    match = AXIOM_AUDIT.fullmatch(text)
    if match is not None:
        declaration = match.group("declaration")
        raw_axioms = match.group("axioms").strip()
        observed = (
            [] if not raw_axioms else [item.strip() for item in raw_axioms.split(",")]
        )
    else:
        no_axioms_match = AXIOM_AUDIT_NONE.fullmatch(text)
        if no_axioms_match is None:
            raise protocol.ValidationError("LS committed axiom audit format is invalid")
        declaration = no_axioms_match.group("declaration")
        observed = []
    if declaration != receipt["expected_lean_declaration"]:
        raise protocol.ValidationError(
            "LS committed axiom audit declaration does not match expected declaration"
        )
    if any(not item for item in observed):
        raise protocol.ValidationError(
            "LS committed axiom audit contains an empty axiom"
        )
    if len(set(observed)) != len(observed):
        raise protocol.ValidationError(
            "LS committed axiom audit contains duplicate axioms"
        )
    if set(observed) != set(receipt["observed_axioms"]):
        raise protocol.ValidationError(
            "LS committed axiom audit does not match receipt observed_axioms"
        )


def _module_path(repository_root: Path, build_target: str) -> Path:
    normalized = _safe_build_target(build_target)
    return repository_root.joinpath(*PurePosixPath(normalized).parts)


def _build_target_module(build_target: str) -> str:
    path = PurePosixPath(_safe_build_target(build_target))
    relative = path.relative_to("formalization/lean").with_suffix("")
    return ".".join(relative.parts)


def _active_import_closure(repository_root: Path) -> frozenset[str]:
    chain = _pin_absolute_chain(repository_root, "LS repository root")
    try:
        return _active_import_closure_at(chain[-1]).modules
    finally:
        _close_pinned_directories(chain)


def _active_import_closure_at(
    repository_root: _PinnedDirectory,
    roots: Sequence[str] = (LS_AGGREGATE_MODULE,),
) -> _LocalSourceClosure:
    formalization = _pin_directory_at(
        repository_root,
        "formalization",
        repository_root.path / "formalization",
        "LS formalization root",
    )
    lean_directory: _PinnedDirectory | None = None
    try:
        lean_directory = _pin_directory_at(
            formalization,
            "lean",
            formalization.path / "lean",
            "LS Lean source root",
        )
        modules = _bounded_tree_modules(lean_directory, MAX_IMPORT_MODULES)

        missing_roots = [root for root in roots if root not in modules]
        if missing_roots:
            raise protocol.ValidationError(
                f"LS source closure root does not exist: {missing_roots[0]}"
            )
        visited: set[str] = set()
        snapshots: dict[str, bytes] = {}
        pending = list(reversed(tuple(roots)))
        while pending:
            module = pending.pop()
            if module in visited:
                continue
            visited.add(module)
            source = _read_relative_file(
                lean_directory,
                modules[module],
                f"LS Lean module {module}",
                MAX_MEMBER_BYTES,
            )
            snapshots[module] = source
            for imported in _active_lean_imports(source, module):
                if imported in modules and imported not in visited:
                    pending.append(imported)
        records = [
            {
                "module": module,
                "sha256": protocol.sha256_bytes(snapshots[module]),
            }
            for module in sorted(snapshots)
        ]
        return _LocalSourceClosure(
            modules=frozenset(visited),
            sources=tuple((module, snapshots[module]) for module in sorted(snapshots)),
            sha256=protocol.sha256_bytes(
                (
                    json.dumps(
                        {"sources": records},
                        sort_keys=True,
                        separators=(",", ":"),
                        ensure_ascii=False,
                        allow_nan=False,
                    ).encode("utf-8")
                    + b"\n"
                )
            ),
        )
    finally:
        if lean_directory is not None:
            _close_pinned_directories((lean_directory,))
        _close_pinned_directories((formalization,))


def _required_mathlib_artifacts_digest(
    repository_root: Path, source_closure: _LocalSourceClosure
) -> str:
    modules: set[str] = set()
    for source_module, data in source_closure.sources:
        for imported in _active_lean_imports(data, source_module):
            if imported.startswith("Mathlib"):
                if LEAN_MODULE_NAME.fullmatch(imported) is None:
                    raise protocol.ValidationError(
                        f"LS local source has invalid Mathlib import: {imported}"
                    )
                modules.add(imported)
    if not modules:
        return protocol.sha256_bytes(b'{"artifacts":[]}\n')
    lake_root = _resolve_approved_lake_root(repository_root / "formalization/lean")
    lake_chain = _pin_absolute_chain(lake_root, "approved Lean .lake root")
    records = []
    try:
        for module in sorted(modules):
            relative = (
                "packages/mathlib/.lake/build/lib/lean/"
                + "/".join(module.split("."))
                + ".olean"
            )
            data = _read_relative_file(
                lake_chain[-1],
                relative,
                f"required Mathlib artifact for {module}",
                MAX_MEMBER_BYTES,
            )
            records.append({"module": module, "sha256": protocol.sha256_bytes(data)})
    finally:
        _close_pinned_directories(lake_chain)
    return protocol.sha256_bytes(
        json.dumps(
            {"artifacts": records},
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
        + b"\n"
    )


def _resolve_approved_lake_root(lean_root: Path) -> Path:
    lake_path = lean_root / ".lake"
    try:
        metadata = lake_path.lstat()
        resolved = lake_path.resolve(strict=True)
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot resolve approved Lean .lake boundary: {error}"
        ) from error
    if not (stat.S_ISLNK(metadata.st_mode) or stat.S_ISDIR(metadata.st_mode)):
        raise protocol.ValidationError(
            "approved Lean .lake boundary must be a directory or symlink"
        )
    return resolved


def _bounded_tree_modules(
    root: _PinnedDirectory, maximum_entries: int
) -> dict[str, str]:
    modules: dict[str, str] = {}
    pending: list[tuple[_PinnedDirectory, tuple[str, ...]]] = [(root, ())]
    opened: list[_PinnedDirectory] = []
    entry_count = 0
    try:
        while pending:
            directory, prefix = pending.pop()
            with os.scandir(directory.descriptor) as entries:
                for entry in entries:
                    entry_count += 1
                    if entry_count > maximum_entries:
                        raise protocol.ValidationError(
                            "LS Lean source entry count exceeds cap"
                        )
                    metadata = entry.stat(follow_symlinks=False)
                    if not prefix and entry.name == ".lake":
                        continue
                    if stat.S_ISLNK(metadata.st_mode):
                        raise protocol.ValidationError(
                            "LS Lean source contains symlink: "
                            + "/".join((*prefix, entry.name))
                        )
                    parts = (*prefix, entry.name)
                    if stat.S_ISDIR(metadata.st_mode):
                        child = _pin_directory_at(
                            directory,
                            entry.name,
                            directory.path / entry.name,
                            "LS Lean source directory",
                        )
                        opened.append(child)
                        pending.append((child, parts))
                    elif stat.S_ISREG(metadata.st_mode) and entry.name.endswith(
                        ".lean"
                    ):
                        module_parts = (*prefix, entry.name[:-5])
                        module = ".".join(module_parts)
                        if LEAN_MODULE_NAME.fullmatch(module) is None:
                            continue
                        if module in modules:
                            raise protocol.ValidationError(
                                f"duplicate LS Lean module: {module}"
                            )
                        modules[module] = "/".join(parts)
            _require_pinned_directory(directory, "LS Lean source directory")
        return modules
    finally:
        _close_pinned_directories(opened)


def _active_lean_imports(data: bytes, module: str) -> tuple[str, ...]:
    try:
        source = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise protocol.ValidationError(
            f"cannot decode LS Lean module {module}: {error}"
        ) from error
    try:
        return provider_independence.parse_active_imports(source, module)
    except provider_independence.ProviderIndependenceError as error:
        raise protocol.ValidationError(
            f"cannot parse imports for LS Lean module {module}: {error}"
        ) from error


def audit_required_theorem_provider_calls(
    repository_root: Path,
) -> dict[str, tuple[str, ...]]:
    """Audit required provider calls in exact active theorem bodies."""

    chain = _pin_absolute_chain(repository_root, "LS repository root")
    root = chain[-1]
    try:
        audited: dict[str, tuple[str, ...]] = {}
        for node_id, spec in ls_contract.THEOREM_BODY_AUDITS.items():
            path = str(spec["path"])
            module = _build_target_module(path)
            data = _read_relative_file(
                root,
                path,
                f"LS theorem audit source for {node_id}",
                provider_independence.MAX_SOURCE_BYTES,
            )
            try:
                source = data.decode("utf-8")
            except UnicodeDecodeError as error:
                raise protocol.ValidationError(
                    f"cannot decode LS theorem audit source for {node_id}: {error}"
                ) from error
            try:
                masked = provider_independence._mask_inactive_source(source, module)
            except provider_independence.ProviderIndependenceError as error:
                raise protocol.ValidationError(
                    f"cannot mask LS theorem audit source for {node_id}: {error}"
                ) from error
            body = _extract_top_level_theorem_body(
                masked,
                str(spec["theorem"]),
                f"LS theorem audit source for {node_id}",
            )
            calls = _active_identifier_leaves(body)
            required = tuple(str(name) for name in spec["required_calls"])
            forbidden = tuple(str(name) for name in spec["forbidden_calls"])
            missing = [name for name in required if name not in calls]
            if missing:
                raise protocol.ValidationError(
                    f"{node_id} theorem body is missing required provider calls: "
                    + ", ".join(missing)
                )
            present_forbidden = [name for name in forbidden if name in calls]
            if present_forbidden:
                raise protocol.ValidationError(
                    f"{node_id} theorem body uses forbidden provider calls: "
                    + ", ".join(present_forbidden)
                )
            extra_main_theorem = {
                name
                for name in calls
                if name.endswith("MainTheorem")
                and name not in set(required)
                and name != str(spec["theorem"])
            }
            if extra_main_theorem:
                raise protocol.ValidationError(
                    f"{node_id} theorem body uses unexpected terminal provider calls: "
                    + ", ".join(sorted(extra_main_theorem))
                )
            audited[node_id] = required
        return audited
    finally:
        _close_pinned_directories(chain)


def _extract_top_level_theorem_body(
    masked_source: str, theorem_name: str, label: str
) -> str:
    matches: list[tuple[int, int]] = []
    lines = masked_source.splitlines(keepends=True)
    offset = 0
    for index, line in enumerate(lines):
        stripped = line.rstrip("\r\n")
        if stripped.startswith(f"theorem {theorem_name}"):
            signature_lines = [stripped]
            cursor = index + 1
            while ":= by" not in "\n".join(signature_lines) and cursor < len(lines):
                next_line = lines[cursor].rstrip("\r\n")
                if next_line and next_line[0] not in " \t":
                    break
                signature_lines.append(next_line)
                cursor += 1
            signature = "\n".join(signature_lines)
            if ":= by" not in signature:
                if ":=" in signature:
                    raise protocol.ValidationError(
                        f"non-by theorem declaration in {label}: {theorem_name}"
                    )
                if signature.strip() == f"theorem {theorem_name}":
                    raise protocol.ValidationError(
                        f"malformed theorem declaration in {label}: {theorem_name}"
                    )
                continue
            if not re.fullmatch(
                rf"theorem\s+{re.escape(theorem_name)}\b[\s\S]*:=\s*by\s*",
                signature,
            ):
                raise protocol.ValidationError(
                    f"malformed theorem declaration in {label}: {theorem_name}"
                )
            start = offset + sum(len(lines[i]) for i in range(index, cursor))
            if cursor < len(lines):
                start += 0
            matches.append((start, cursor))
        offset += len(line)
    if not matches:
        raise protocol.ValidationError(
            f"missing theorem declaration in {label}: {theorem_name}"
        )
    if len(matches) != 1:
        raise protocol.ValidationError(
            f"duplicate theorem declaration in {label}: {theorem_name}"
        )
    start, start_line_index = matches[0]
    lines = masked_source.splitlines(keepends=True)
    end = len(masked_source)
    running = sum(len(lines[i]) for i in range(start_line_index))
    for line in lines[start_line_index:]:
        if line and line[0] not in " \t\r\n" and LS_AUDIT_TOPLEVEL_COMMAND.match(line):
            end = running
            break
        running += len(line)
    body = masked_source[start:end]
    if not body.strip():
        raise protocol.ValidationError(f"empty theorem body in {label}: {theorem_name}")
    return body


def _active_identifier_leaves(body: str) -> frozenset[str]:
    leaves = set()
    for match in LS_AUDIT_CALL_TOKEN.finditer(body):
        token = match.group(0)
        leaves.add(token.rsplit(".", 1)[-1])
    return frozenset(leaves)


def _safe_build_target(value: Any) -> str:
    text = _bounded_string(value, "build_target", 1, 4096)
    path = PurePosixPath(text)
    if (
        "\\" in text
        or path.is_absolute()
        or text != path.as_posix()
        or any(part in {"", ".", ".."} for part in path.parts)
        or path.parts[:2] != ("formalization", "lean")
        or len(path.parts) < 3
        or path.suffix != ".lean"
    ):
        raise protocol.ValidationError(
            "build_target must be normalized beneath formalization/lean"
        )
    return text


def _read_candidate_receipt(attempt: _PinnedDirectory) -> bytes | None:
    """Read bytes for digest-only attempt discovery.

    Missing, non-regular, oversized, and unreadable regular historical receipts
    cannot match the graph digest, so discovery ignores them. Symlinks are
    always rejected, including in non-selected history.
    """

    try:
        metadata = os.stat(
            "receipt.json", dir_fd=attempt.descriptor, follow_symlinks=False
        )
    except FileNotFoundError:
        return None
    except OSError:
        return None
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError("LS attempt receipt cannot be a symlink")
    if not stat.S_ISREG(metadata.st_mode) or metadata.st_size > MAX_JSON_BYTES:
        return None
    try:
        return _read_file_at(
            attempt, "receipt.json", "LS historical attempt receipt", MAX_JSON_BYTES
        )
    except protocol.ValidationError as error:
        if "symlink" in str(error):
            raise
        return None


def _validate_attempt_members(
    attempt: _PinnedDirectory, node_id: str
) -> _PinnedDirectory:
    observed_files: set[str] = set()
    observed_directories: set[str] = set()
    build: _PinnedDirectory | None = None
    try:
        pending: list[tuple[_PinnedDirectory, str]] = [(attempt, "")]
        while pending:
            directory, prefix = pending.pop()
            _require_pinned_directory(directory, "LS attempt member directory")
            with os.scandir(directory.descriptor) as entries:
                for entry in entries:
                    relative = f"{prefix}/{entry.name}" if prefix else entry.name
                    metadata = entry.stat(follow_symlinks=False)
                    if stat.S_ISLNK(metadata.st_mode):
                        raise protocol.ValidationError(
                            f"LS attempt member {relative} cannot be a symlink"
                        )
                    if stat.S_ISDIR(metadata.st_mode):
                        if relative not in ATTEMPT_DIRECTORIES:
                            raise protocol.ValidationError(
                                "unexpected LS attempt member directory: " + relative
                            )
                        observed_directories.add(relative)
                        child = _pin_directory_at(
                            directory,
                            entry.name,
                            directory.path / entry.name,
                            f"LS attempt member directory {relative}",
                        )
                        build = child
                        pending.append((child, relative))
                        continue
                    if not stat.S_ISREG(metadata.st_mode):
                        raise protocol.ValidationError(
                            f"LS attempt member must be a regular file: {relative}"
                        )
                    if relative not in ATTEMPT_FILES:
                        raise protocol.ValidationError(
                            f"unexpected LS attempt member: {relative}"
                        )
                    observed_files.add(relative)
            _require_pinned_directory(directory, "LS attempt member directory")
    except protocol.ValidationError:
        if build is not None:
            _close_pinned_directories((build,))
        raise
    except OSError as error:
        if build is not None:
            _close_pinned_directories((build,))
        raise protocol.ValidationError(
            f"cannot inspect LS attempt members for node {node_id}: {error}"
        ) from error

    missing_directories = sorted(ATTEMPT_DIRECTORIES - observed_directories)
    if missing_directories:
        raise protocol.ValidationError(
            f"missing LS attempt member directory: {missing_directories[0]}"
        )
    extra_directories = sorted(observed_directories - ATTEMPT_DIRECTORIES)
    if extra_directories:
        raise protocol.ValidationError(
            f"unexpected LS attempt member directory: {extra_directories[0]}"
        )
    missing_files = sorted(ATTEMPT_FILES - observed_files)
    if missing_files:
        raise protocol.ValidationError(f"missing LS attempt member: {missing_files[0]}")
    extra_files = sorted(observed_files - ATTEMPT_FILES)
    if extra_files:
        raise protocol.ValidationError(
            f"unexpected LS attempt member: {extra_files[0]}"
        )
    assert build is not None
    return build


def _read_safe_bytes(path: Path, label: str, maximum: int) -> bytes:
    absolute = Path(os.path.abspath(os.fspath(path)))
    try:
        parent = absolute.parent.resolve(strict=True)
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot resolve {label} parent: {error}"
        ) from error
    chain = _pin_absolute_chain(parent, f"{label} parent")
    try:
        return _read_file_at(chain[-1], absolute.name, label, maximum)
    finally:
        _close_pinned_directories(chain)


def _json_object_from_bytes(data: bytes, label: str) -> dict[str, Any]:
    try:
        value = json.loads(data, object_pairs_hook=_reject_duplicate_json_pairs)
    except (UnicodeDecodeError, ValueError) as error:
        raise protocol.ValidationError(f"cannot parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise protocol.ValidationError(f"{label} must contain one JSON object")
    return value


def _reject_duplicate_json_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f"duplicate JSON key: {key}")
        value[key] = item
    return value


def _require_bytes_digest(data: bytes, expected: str, label: str) -> None:
    if protocol.sha256_bytes(data) != expected:
        raise protocol.ValidationError(f"{label} mismatch")


def materialize_tasks(rows: tuple[LSGraphRow, ...], root: Path) -> dict[str, object]:
    _ensure_output_root(root)
    published: set[str] = set()
    for row in rows:
        if "JIN" in row.lean_name or "Jin" in row.lean_name:
            raise protocol.ValidationError("LS task cannot import Jin private work")
        for dependency in row.dependencies:
            if dependency not in published:
                raise protocol.ValidationError(
                    f"LS task predecessor is not published: {dependency}"
                )
        task_dir = root / row.node_id
        _ensure_new_directory(task_dir, "LS task directory")
        _write_json_create_only(
            task_dir / "task.json",
            {
                "schema_version": "crouzeix-ls-task/v1",
                "node_id": row.node_id,
                "source_locator": row.source_locator,
                "statement_sha256": row.statement_sha256,
                "lean_name": row.lean_name,
                "dependencies": list(row.dependencies),
                "status": row.status,
            },
        )
        _write_json_create_only(
            task_dir / "result.json",
            {
                "schema_version": "crouzeix-ls-result/v1",
                "node_id": row.node_id,
                "status": row.status,
                "reason": _node_result_reason(row),
            },
        )
        published.add(row.node_id)
    terminal = [row for row in rows if row.role == "terminal"][0]
    assembly = root / "assembly"
    _ensure_new_directory(assembly, "LS assembly directory")
    blocked_by = list(terminal.dependencies)
    _write_json_create_only(
        assembly / "result.json",
        {
            "schema_version": "crouzeix-ls-assembly-result/v1",
            "terminal_node_id": terminal.node_id,
            "status": "blocked",
            "blocked_by": blocked_by,
            "reason": "terminal LS theorem awaits formal slices and clean rebuild",
        },
    )
    return {
        "schema_version": "crouzeix-ls-materialization/v1",
        "status": "blocked",
        "terminal_node_id": terminal.node_id,
        "blocked_by": blocked_by,
    }


def _node_result_reason(row: LSGraphRow) -> str:
    if row.status == "passed":
        return "LS node has a compiled Lean receipt"
    if row.status == "blocked":
        assert row.blocked_reason is not None
        return row.blocked_reason
    if row.status == "failed":
        assert row.failed_reason is not None
        return row.failed_reason
    return "mapped LS node requires later formal proof slice"


def _node_list(value: Any) -> list[LSGraphRow]:
    if not isinstance(value, list) or not value:
        raise protocol.ValidationError("LS graph nodes must be a nonempty list")
    if len(value) > 1024:
        raise protocol.ValidationError("LS graph nodes exceed cap")
    return [_node(_mapping(item, "LS graph node")) for item in value]


def _node(value: Mapping[str, Any]) -> LSGraphRow:
    _require_node_fields(value)
    locator = _source_locator(value["source_locator"], "source_locator")
    status = _enum(value["status"], NODE_STATUSES, "status")
    receipt_sha256 = _optional_digest(value.get("receipt_sha256"), "receipt_sha256")
    blocked_reason = _optional_bounded_string(
        value.get("blocked_reason"), "blocked_reason", 1, 4096
    )
    failed_reason = _optional_bounded_string(
        value.get("failed_reason"), "failed_reason", 1, 4096
    )
    _validate_node_outcome_fields(
        status,
        receipt_sha256,
        blocked_reason,
        failed_reason,
        has_receipt_sha256="receipt_sha256" in value,
        has_blocked_reason="blocked_reason" in value,
        has_failed_reason="failed_reason" in value,
    )
    return LSGraphRow(
        node_id=formal_target._runtime_id(value["node_id"], "node_id"),
        source_locator=locator,
        statement_sha256=formal_target._digest(
            value["statement_sha256"], "statement_sha256"
        ),
        lean_name=_bounded_string(value["lean_name"], "lean_name", 1, 256),
        dependencies=tuple(_runtime_id_list(value["dependencies"], "dependencies")),
        role=_enum(value["role"], NODE_ROLES, "role"),
        status=status,
        receipt_sha256=receipt_sha256,
        blocked_reason=blocked_reason,
        failed_reason=failed_reason,
    )


def _fact(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, FACT_FIELDS, "LS library fact")
    return {
        "fact_id": formal_target._runtime_id(value["fact_id"], "fact_id"),
        "statement_sha256": formal_target._digest(
            value["statement_sha256"], "statement_sha256"
        ),
        "source_locator": _source_locator(value["source_locator"], "source_locator"),
        "resolution": _enum(value["resolution"], FACT_RESOLUTIONS, "resolution"),
    }


def _require_node_fields(value: Mapping[str, Any]) -> None:
    fields = set(value)
    required = set(NODE_FIELDS)
    allowed = required | set(NODE_OPTIONAL_FIELDS)
    missing = sorted(required - fields)
    extra = sorted(fields - allowed)
    if missing or extra:
        detail = []
        if missing:
            detail.append(f"missing {', '.join(missing)}")
        if extra:
            detail.append(f"unknown {', '.join(extra)}")
        raise protocol.ValidationError(
            f"LS graph node fields are invalid: {'; '.join(detail)}"
        )


def _validate_node_outcome_fields(
    status: str,
    receipt_sha256: str | None,
    blocked_reason: str | None,
    failed_reason: str | None,
    *,
    has_receipt_sha256: bool,
    has_blocked_reason: bool,
    has_failed_reason: bool,
) -> None:
    if status == "mapped":
        if has_receipt_sha256:
            raise protocol.ValidationError(
                "mapped LS graph node rejects receipt_sha256"
            )
        if has_blocked_reason:
            raise protocol.ValidationError(
                "mapped LS graph node rejects blocked_reason"
            )
        if has_failed_reason:
            raise protocol.ValidationError("mapped LS graph node rejects failed_reason")
        return

    if status == "blocked":
        if not has_blocked_reason or blocked_reason is None:
            raise protocol.ValidationError(
                "blocked LS graph node requires blocked_reason"
            )
        if has_receipt_sha256:
            raise protocol.ValidationError(
                "blocked LS graph node rejects receipt_sha256"
            )
        if has_failed_reason:
            raise protocol.ValidationError(
                "blocked LS graph node rejects failed_reason"
            )
        return

    if status == "failed":
        if not has_receipt_sha256 or receipt_sha256 is None:
            raise protocol.ValidationError(
                "failed LS graph node requires receipt_sha256"
            )
        if not has_failed_reason or failed_reason is None:
            raise protocol.ValidationError(
                "failed LS graph node requires failed_reason"
            )
        if has_blocked_reason:
            raise protocol.ValidationError(
                "failed LS graph node rejects blocked_reason"
            )
        return

    if status == "passed":
        if not has_receipt_sha256 or receipt_sha256 is None:
            raise protocol.ValidationError(
                "passed LS graph node requires receipt_sha256"
            )
        if has_blocked_reason:
            raise protocol.ValidationError(
                "passed LS graph node rejects blocked_reason"
            )
        if has_failed_reason:
            raise protocol.ValidationError("passed LS graph node rejects failed_reason")
        return

    raise AssertionError(f"unhandled LS graph status: {status}")


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
        raise protocol.ValidationError(
            "LS route cannot reference Jin private work files"
        )
    if LS_SOURCE_IDENTITY not in text:
        raise protocol.ValidationError(f"{label} must reference LS arXiv v1")
    return text


def _read_json_object(path: Path, label: str) -> dict[str, Any]:
    absolute = Path(os.path.abspath(os.fspath(path)))
    try:
        parent = absolute.parent.resolve(strict=True)
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot resolve {label} parent: {error}"
        ) from error
    chain = _pin_absolute_chain(parent, f"{label} parent")
    try:
        data = _read_file_at(chain[-1], absolute.name, label, MAX_JSON_BYTES)
    finally:
        _close_pinned_directories(chain)
    return _json_object_from_bytes(data, label)


def _ensure_safe_file(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(f"{label} cannot be a symlink")
    if not stat.S_ISREG(metadata.st_mode):
        raise protocol.ValidationError(f"{label} must be a regular file")


def _ensure_output_root(path: Path) -> None:
    if path.exists():
        _ensure_directory(path, "LS task root")
        if any(path.iterdir()):
            raise protocol.ValidationError("LS task root must be empty")
        return
    _ensure_directory(path.parent, "LS task root parent")
    path.mkdir(mode=0o700)


def _ensure_new_directory(path: Path, label: str) -> None:
    if path.exists():
        raise protocol.ValidationError(f"{label} already exists")
    _ensure_directory(path.parent, f"{label} parent")
    path.mkdir(mode=0o700)


def _ensure_directory(path: Path, label: str) -> None:
    current = Path(path.anchor) if path.is_absolute() else Path(".")
    parts = path.parts[1:] if path.is_absolute() else path.parts
    for part in parts:
        current = current / part
        try:
            metadata = current.lstat()
        except OSError as error:
            raise protocol.ValidationError(
                f"cannot inspect {label}: {error}"
            ) from error
        if stat.S_ISLNK(metadata.st_mode):
            raise protocol.ValidationError(f"{label} contains symlink: {current}")
        if not stat.S_ISDIR(metadata.st_mode):
            raise protocol.ValidationError(f"{label} must be a directory: {current}")


def _write_json_create_only(path: Path, value: Mapping[str, object]) -> None:
    payload = (
        json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
        + b"\n"
    )
    try:
        fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError as error:
        raise protocol.ValidationError(f"output already exists: {path}") from error
    try:
        os.write(fd, payload)
    finally:
        os.close(fd)


def _require_fields(
    value: Mapping[str, Any], allowed: frozenset[str], label: str
) -> None:
    fields = set(value)
    if fields != set(allowed):
        missing = sorted(set(allowed) - fields)
        extra = sorted(fields - set(allowed))
        detail = []
        if missing:
            detail.append(f"missing {', '.join(missing)}")
        if extra:
            detail.append(f"unknown {', '.join(extra)}")
        raise protocol.ValidationError(
            f"{label} fields are invalid: {'; '.join(detail)}"
        )


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


def _optional_bounded_string(
    value: Any, label: str, minimum: int, maximum: int
) -> str | None:
    if value is None:
        return None
    return _bounded_string(value, label, minimum, maximum)


def _optional_digest(value: Any, label: str) -> str | None:
    if value is None:
        return None
    return formal_target._digest(value, label)


def _integer(value: Any, label: str, minimum: int, maximum: int) -> int:
    if (
        not isinstance(value, int)
        or isinstance(value, bool)
        or not minimum <= value <= maximum
    ):
        raise protocol.ValidationError(
            f"{label} must be an integer between {minimum} and {maximum}"
        )
    return value


def _unique_string_list(value: Any, label: str, *, maximum: int) -> list[str]:
    if not isinstance(value, list) or len(value) > maximum:
        raise protocol.ValidationError(f"{label} must be a bounded list")
    items = [_bounded_string(item, label, 1, 256) for item in value]
    if len(set(items)) != len(items):
        raise protocol.ValidationError(f"{label} cannot contain duplicates")
    return items


def _runtime_id_list(value: Any, label: str) -> list[str]:
    if not isinstance(value, list) or len(value) > 256:
        raise protocol.ValidationError(f"{label} must be a bounded list")
    items = [formal_target._runtime_id(item, label) for item in value]
    if len(set(items)) != len(items):
        raise protocol.ValidationError(f"{label} cannot contain duplicates")
    return items
