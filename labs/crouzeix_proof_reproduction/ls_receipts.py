from __future__ import annotations

import ctypes
import errno
import fcntl
import hashlib
import json
import os
import platform
import re
import signal
import stat
import subprocess
import threading
import uuid
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Callable, Mapping, Sequence

import ls_validation
import ls_contract
import provider_independence
import protocol


MAX_OUTPUT_BYTES = 1024 * 1024
MAX_MEMBER_BYTES = 4 * 1024 * 1024
MAX_ATTEMPTS = 256
MAX_CACHE_ENTRIES = 500_000
TIMEOUT_SECONDS = 3600
PIPE_DRAIN_GRACE_SECONDS = 1.0
SHARED_ELAN_HOME = Path("/private/tmp/harp-mathematical-foundations-elan")
SHARED_LEAN_TOOLCHAIN = "leanprover/lean4:v4.32.1"
SHARED_TOOLCHAIN_BIN = (
    SHARED_ELAN_HOME / "toolchains" / "leanprover--lean4---v4.32.1" / "bin"
)
SHARED_SYSTEM_PATH = (Path("/usr/bin"), Path("/bin"))
AGGREGATE_ARGV = ("scripts/check_lean_library.sh", ls_contract.AGGREGATE_MODULE)
CACHE_POLICY = (
    "requested cached wrapper command; wrapper policy is repository-controlled"
)
ALLOWED_AXIOMS = ls_contract.ALLOWED_AXIOMS
STAGE_PREFIX = ".ls-receipts-stage-"
AUDIT_PREFIX = ".harp-ls-axiom-audit-"
HIDDEN_ATTEMPT_PREFIX = ".attempt-"
ATTEMPT_NAME = re.compile(r"attempt-(?P<number>[0-9]+)\Z")
LEAN_NAME = re.compile(r"[A-Za-z_][A-Za-z0-9_']*(?:\.[A-Za-z_][A-Za-z0-9_']*)*\Z")
AXIOM_AUDIT = re.compile(
    r"\A'(?P<declaration>[^'\r\n]+)' depends on axioms: \[(?P<axioms>.*?)\]\s*\Z",
    re.DOTALL,
)
AXIOM_AUDIT_NONE = re.compile(
    r"\A'(?P<declaration>[^'\r\n]+)' does not depend on any axioms\s*\Z"
)
WRAPPER_FORBIDDEN_COMMANDS = ("lake update", "cache get")
PUBLICATION_LOCK_ROOT = Path("/private/tmp/harp-ls-receipts-locks")
DARWIN_AT_FDCWD = -2
DARWIN_RENAME_EXCL = 0x00000004
LINUX_AT_FDCWD = -100
LINUX_RENAME_NOREPLACE = 0x00000001
BUNDLE_ROOT_FILES = ("task.json", "source-slice.json", "result.json", "receipt.json")
BUNDLE_BUILD_FILES = ("command.json", "stdout.log", "stderr.log", "axioms.txt")


LS_RECEIPT_BINDINGS = {
    node.node_id: (node.declaration, node.build_target) for node in ls_contract.NODES
}
LS_RECEIPT_DEPENDENCIES = {
    node.node_id: node.dependencies for node in ls_contract.NODES
}


@dataclass(frozen=True)
class CommandResult:
    exit_code: int | None
    stdout: bytes
    stderr: bytes
    blocked_reason: str | None = None
    stdout_truncated: bool = False
    stderr_truncated: bool = False


class PartialPublicationError(protocol.ValidationError):
    """A no-replace rename committed one or more unreferenced candidates."""

    def __init__(self, message: str, candidates: tuple[dict[str, str], ...]) -> None:
        super().__init__(message)
        self.candidates = candidates


Executor = Callable[[list[str], Path, dict[str, str], int, int], CommandResult]


@dataclass(frozen=True)
class _AttemptPlan:
    row: ls_validation.LSGraphRow
    declaration: str
    build_target: str
    attempt_name: str
    node_root: Path


@dataclass(frozen=True)
class _FileSnapshot:
    data: bytes
    device: int
    inode: int
    mode: int
    size: int
    mtime_ns: int
    ctime_ns: int
    nlink: int


@dataclass(frozen=True)
class _PublicationLock:
    path: Path
    descriptor: int
    root_descriptor: int
    root_ancestor_descriptors: tuple[int, ...] = ()


@dataclass(frozen=True)
class _PinnedDirectory:
    path: Path
    descriptor: int
    device: int
    inode: int
    parent_descriptor: int | None = None
    entry_name: str | None = None
    owned_ancestor_descriptors: tuple[int, ...] = ()


@dataclass(frozen=True)
class _ExecutionSnapshot:
    wrapper: _FileSnapshot
    lake_manifest: _FileSnapshot
    dependency_cache_metadata_sha256: str
    required_mathlib_artifacts: tuple[tuple[str, _FileSnapshot], ...]
    required_mathlib_artifacts_sha256: str
    local_modules: tuple[tuple[str, _FileSnapshot], ...]
    local_source_closure_sha256: str


@dataclass
class _PublicationCandidate:
    plan: _AttemptPlan
    node_root: _PinnedDirectory
    hidden_attempt: _PinnedDirectory
    metadata: dict[str, str]
    rename_attempted: bool = False
    visible: bool = False
    published_attempt: _PinnedDirectory | None = None


def publish_ls_receipts(
    rows: Sequence[ls_validation.LSGraphRow],
    repository_root: Path,
    formal_target_root: Path,
    *,
    executor: Executor | None = None,
) -> dict[str, dict[str, str]]:
    """Build, audit, and create one new receipt attempt for every LS node.

    The aggregate build and all six declaration audits must succeed before any
    attempt directory is created beneath ``proof-slices``. Existing attempts
    are immutable history and are never reused or overwritten. The returned
    receipt digests are candidates: this function never mutates the graph, so
    every new complete attempt remains unreferenced until its caller updates
    the authoritative graph ``receipt_sha256``.
    """
    publication_candidates: list[_PublicationCandidate] = []
    try:
        return _publish_ls_receipts(
            rows,
            repository_root,
            formal_target_root,
            executor=executor,
            publication_candidates=publication_candidates,
        )
    except BaseException as error:
        candidates = _visible_candidate_metadata(publication_candidates)
        if not candidates:
            raise
        if (
            isinstance(error, PartialPublicationError)
            and error.candidates == candidates
        ):
            cleanup_error = error.__context__
            if cleanup_error is None or str(cleanup_error) in str(error):
                raise
        details = ", ".join(
            f"{item['attempt_path']} receipt_sha256={item['receipt_sha256']}"
            for item in candidates
        )
        raise PartialPublicationError(
            "LS committed partial publication; visible candidates remain "
            f"unreferenced: {details}; cause: {error}",
            candidates,
        ) from error


def _publish_ls_receipts(
    rows: Sequence[ls_validation.LSGraphRow],
    repository_root: Path,
    formal_target_root: Path,
    *,
    executor: Executor | None,
    publication_candidates: list[_PublicationCandidate],
) -> dict[str, dict[str, str]]:
    repository = _absolute_normalized(repository_root)
    target = _absolute_normalized(formal_target_root)
    _require_descendant(target, repository, "LS formal target root")
    ordered_rows = _validate_rows(rows)
    repository_directory = _pin_absolute_directory(repository, "repository root")
    target_directory: _PinnedDirectory | None = None
    proof_slices: _PinnedDirectory | None = None
    lean_directory: _PinnedDirectory | None = None
    publication_lock: _PublicationLock | None = None
    stage: _PinnedDirectory | None = None
    try:
        relative_target = target.relative_to(repository)
        target_directory = _pin_directory_chain_at(
            repository_directory, relative_target.parts, target, "LS formal target root"
        )
        proof_slices = _pin_directory_at(
            target_directory,
            "proof-slices",
            target / "proof-slices",
            "LS proof-slices root",
        )
        lean_root = repository / "formalization/lean"
        lean_directory = _pin_directory_chain_at(
            repository_directory,
            ("formalization", "lean"),
            lean_root,
            "Lean root",
        )
        publication_lock = _acquire_publication_lock(proof_slices)
        plans = _plan_attempts(ordered_rows, proof_slices)
        execution_roots = (ls_contract.AGGREGATE_MODULE,)
        execution_snapshot = _read_execution_snapshot(repository, execution_roots)
        _validate_wrapper_cache_policy(execution_snapshot.wrapper.data)

        module_snapshots = _read_module_snapshots(repository, plans)
        environment = _shared_environment()
        run = executor or _subprocess_executor

        stage = _create_unique_directory_at(
            target_directory, STAGE_PREFIX, "LS receipt staging directory"
        )
        aggregate = _run_checked(
            run, list(AGGREGATE_ARGV), repository, environment, "aggregate build"
        )
        _validate_aggregate_stdout(aggregate.stdout)
        audits: dict[str, tuple[bytes, tuple[str, ...]]] = {}
        for plan in plans:
            audit_bytes = _run_axiom_audit(
                run, plan, lean_root, lean_directory, environment
            )
            audits[plan.row.node_id] = (
                audit_bytes,
                _parse_axiom_audit(audit_bytes, plan.declaration),
            )

        if _read_module_snapshots(repository, plans) != module_snapshots:
            raise protocol.ValidationError(
                "LS Lean module bytes changed during receipt execution"
            )
        _require_execution_snapshot(repository, execution_roots, execution_snapshot)

        for plan in plans:
            attempt = _create_stage_attempt(stage, plan)
            try:
                audit_bytes, observed_axioms = audits[plan.row.node_id]
                _stage_bundle(
                    attempt,
                    plan,
                    aggregate,
                    audit_bytes,
                    observed_axioms,
                    module_snapshots[plan.row.node_id].data,
                    execution_snapshot,
                )
            finally:
                _close_pinned_directory(attempt)

        current_plans = _plan_attempts(ordered_rows, proof_slices)
        if [plan.attempt_name for plan in current_plans] != [
            plan.attempt_name for plan in plans
        ]:
            raise protocol.ValidationError(
                "LS attempt history changed during receipt execution"
            )
        return _publish_staged_bundles(
            stage,
            proof_slices,
            plans,
            repository,
            module_snapshots,
            execution_snapshot,
            publication_candidates,
        )
    finally:
        try:
            if stage is not None:
                _remove_stage(stage, target_directory)
        finally:
            try:
                if stage is not None:
                    _close_pinned_directory(stage)
            finally:
                try:
                    if lean_directory is not None:
                        _close_pinned_directory(lean_directory)
                finally:
                    try:
                        if proof_slices is not None:
                            _close_pinned_directory(proof_slices)
                    finally:
                        try:
                            if target_directory is not None:
                                _close_pinned_directory(target_directory)
                        finally:
                            try:
                                _close_pinned_directory(repository_directory)
                            finally:
                                if publication_lock is not None:
                                    _release_publication_lock(publication_lock)


def _validate_rows(
    rows: Sequence[ls_validation.LSGraphRow],
) -> tuple[ls_validation.LSGraphRow, ...]:
    if isinstance(rows, (str, bytes)):
        raise protocol.ValidationError("LS receipt rows must be a sequence")
    materialized = tuple(rows)
    by_id: dict[str, ls_validation.LSGraphRow] = {}
    for row in materialized:
        if not isinstance(row, ls_validation.LSGraphRow):
            raise protocol.ValidationError("LS receipt publisher row is invalid")
        node_id = ls_validation.formal_target._runtime_id(row.node_id, "node_id")
        if node_id in by_id:
            raise protocol.ValidationError("LS receipt publisher has duplicate node_id")
        ls_validation._source_locator(row.source_locator, "source_locator")
        ls_validation.formal_target._digest(row.statement_sha256, "statement_sha256")
        if (
            not isinstance(row.lean_name, str)
            or not 1 <= len(row.lean_name) <= 256
            or "\0" in row.lean_name
        ):
            raise protocol.ValidationError(
                "lean_name must be a non-NUL string of length 1..256"
            )
        if not isinstance(row.dependencies, tuple) or len(row.dependencies) > 256:
            raise protocol.ValidationError(
                "LS graph node dependencies must be a bounded tuple"
            )
        for dependency in row.dependencies:
            ls_validation.formal_target._runtime_id(dependency, "dependencies")
        if len(set(row.dependencies)) != len(row.dependencies):
            raise protocol.ValidationError(
                f"LS receipt row has duplicate dependencies: {row.node_id}"
            )
        if row.role not in ls_validation.NODE_ROLES:
            raise protocol.ValidationError(
                f"LS receipt row role is invalid: {row.node_id}"
            )
        ls_validation._validate_node_outcome_fields(
            row.status,
            row.receipt_sha256,
            row.blocked_reason,
            row.failed_reason,
            has_receipt_sha256=row.receipt_sha256 is not None,
            has_blocked_reason=row.blocked_reason is not None,
            has_failed_reason=row.failed_reason is not None,
        )
        by_id[node_id] = row

    ls_validation._validate_graph_contract(materialized, allow_legacy=True)
    if set(by_id) != set(LS_RECEIPT_BINDINGS):
        raise protocol.ValidationError("LS receipt publisher node set is invalid")
    for index, (node_id, (declaration, _)) in enumerate(LS_RECEIPT_BINDINGS.items()):
        row = by_id[node_id]
        if row.lean_name not in ls_contract.BY_ID[node_id].accepted_graph_lean_names:
            raise protocol.ValidationError(
                f"LS receipt declaration does not match fixed mapping: {node_id}"
            )
        unknown_dependencies = set(row.dependencies) - set(by_id)
        if unknown_dependencies:
            raise protocol.ValidationError(
                f"LS receipt row has unknown dependency: {node_id}"
            )
        expected_contract = ls_contract.BY_ID[node_id]
        accepted_dependencies = {
            expected_contract.dependencies,
            expected_contract.legacy_dependencies,
        }
        if row.dependencies not in accepted_dependencies:
            raise protocol.ValidationError(
                f"LS receipt dependencies do not match fixed graph: {node_id}"
            )
        expected_role = (
            "terminal" if index == len(LS_RECEIPT_BINDINGS) - 1 else "intermediate"
        )
        if row.role != expected_role:
            raise protocol.ValidationError(
                f"LS receipt role does not match fixed graph: {node_id}"
            )
    ls_validation._reject_cycles(list(materialized))
    return tuple(_canonicalize_row(by_id[node_id]) for node_id in LS_RECEIPT_BINDINGS)


def _canonicalize_row(row: ls_validation.LSGraphRow) -> ls_validation.LSGraphRow:
    expected = ls_contract.BY_ID[row.node_id]
    return ls_validation.LSGraphRow(
        node_id=row.node_id,
        source_locator=expected.source_locator,
        statement_sha256=expected.statement_sha256,
        lean_name=expected.declaration,
        dependencies=expected.dependencies,
        role=expected.role,
        status=row.status,
        receipt_sha256=row.receipt_sha256,
        blocked_reason=row.blocked_reason,
        failed_reason=row.failed_reason,
    )


def _plan_attempts(
    rows: tuple[ls_validation.LSGraphRow, ...], proof_slices: _PinnedDirectory
) -> tuple[_AttemptPlan, ...]:
    plans = []
    for row in rows:
        _, build_target = LS_RECEIPT_BINDINGS[row.node_id]
        node_root = proof_slices.path / row.node_id
        next_number = _next_attempt_number_at(proof_slices, row.node_id)
        plans.append(
            _AttemptPlan(
                row=row,
                declaration=LS_RECEIPT_BINDINGS[row.node_id][0],
                build_target=build_target,
                attempt_name=f"attempt-{next_number:03d}",
                node_root=node_root,
            )
        )
    return tuple(plans)


def _next_attempt_number_at(proof_slices: _PinnedDirectory, node_id: str) -> int:
    _safe_entry_name(node_id, "LS node id")
    _require_pinned_directory(proof_slices, "LS proof-slices root")
    try:
        node_root = _pin_directory_at(
            proof_slices,
            node_id,
            proof_slices.path / node_id,
            f"LS node root for {node_id}",
        )
    except FileNotFoundError:
        return 1
    try:
        numbers: list[int] = []
        entry_count = 0
        try:
            with os.scandir(node_root.descriptor) as entries:
                for entry in entries:
                    entry_count += 1
                    if entry_count >= MAX_ATTEMPTS:
                        raise protocol.ValidationError(
                            f"LS attempt count leaves no publication capacity: {node_id}"
                        )
                    match = ATTEMPT_NAME.fullmatch(entry.name)
                    if match is None:
                        if entry.name.startswith("attempt-"):
                            raise protocol.ValidationError(
                                f"unsafe LS attempt name for node {node_id}: "
                                f"{entry.name}"
                            )
                        continue
                    metadata = entry.stat(follow_symlinks=False)
                    if stat.S_ISLNK(metadata.st_mode):
                        raise protocol.ValidationError(
                            f"LS attempt cannot be a symlink: {entry.name}"
                        )
                    if not stat.S_ISDIR(metadata.st_mode):
                        raise protocol.ValidationError(
                            f"LS attempt must be a directory: {entry.name}"
                        )
                    number_text = match.group("number")
                    if len(number_text) > 12:
                        raise protocol.ValidationError(
                            f"LS attempt number is too large: {entry.name}"
                        )
                    numbers.append(int(number_text))
        except protocol.ValidationError:
            raise
        except OSError as error:
            raise protocol.ValidationError(
                f"cannot inspect LS attempts for node {node_id}: {error}"
            ) from error
        return max(numbers, default=0) + 1
    finally:
        _close_pinned_directory(node_root)


def _next_attempt_number(node_root: Path, node_id: str) -> int:
    if not node_root.exists() and not node_root.is_symlink():
        _ensure_directory(node_root.parent, f"LS node parent for {node_id}")
        return 1
    _ensure_directory(node_root, f"LS node root for {node_id}")
    numbers: list[int] = []
    entry_count = 0
    try:
        with os.scandir(node_root) as entries:
            for entry in entries:
                entry_count += 1
                if entry_count >= MAX_ATTEMPTS:
                    raise protocol.ValidationError(
                        f"LS attempt count leaves no publication capacity: {node_id}"
                    )
                match = ATTEMPT_NAME.fullmatch(entry.name)
                if match is None:
                    if entry.name.startswith("attempt-"):
                        raise protocol.ValidationError(
                            f"unsafe LS attempt name for node {node_id}: {entry.name}"
                        )
                    continue
                metadata = entry.stat(follow_symlinks=False)
                if stat.S_ISLNK(metadata.st_mode):
                    raise protocol.ValidationError(
                        f"LS attempt cannot be a symlink: {entry.name}"
                    )
                if not stat.S_ISDIR(metadata.st_mode):
                    raise protocol.ValidationError(
                        f"LS attempt must be a directory: {entry.name}"
                    )
                number_text = match.group("number")
                if len(number_text) > 12:
                    raise protocol.ValidationError(
                        f"LS attempt number is too large: {entry.name}"
                    )
                numbers.append(int(number_text))
    except protocol.ValidationError:
        raise
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot inspect LS attempts for node {node_id}: {error}"
        ) from error
    return max(numbers, default=0) + 1


def _read_module_snapshots(
    repository_root: Path, plans: tuple[_AttemptPlan, ...]
) -> dict[str, _FileSnapshot]:
    snapshots = {}
    for plan in plans:
        module_path = repository_root.joinpath(*PurePosixPath(plan.build_target).parts)
        snapshots[plan.row.node_id] = _read_file_snapshot(
            module_path,
            f"LS Lean module for {plan.row.node_id}",
            MAX_MEMBER_BYTES,
        )
    return snapshots


def _local_module_snapshots(
    repository_root: Path, roots: Sequence[str]
) -> tuple[tuple[str, _FileSnapshot], ...]:
    lean_root = repository_root / "formalization/lean"
    try:
        report = provider_independence.audit_provider_independence(
            lean_root,
            roots,
            ls_contract.FORBIDDEN_MODULES,
            forbidden_prefixes=ls_contract.FORBIDDEN_PREFIXES,
        )
    except provider_independence.ProviderIndependenceError as error:
        raise protocol.ValidationError(
            f"LS receipt target is not provider independent: {error}"
        ) from error
    return tuple(
        (
            module,
            _read_file_snapshot(
                lean_root.joinpath(*module.split(".")).with_suffix(".lean"),
                f"LS local source module {module}",
                MAX_MEMBER_BYTES,
            ),
        )
        for module in report.modules
    )


def _read_execution_snapshot(
    repository_root: Path, roots: Sequence[str]
) -> _ExecutionSnapshot:
    wrapper = _read_file_snapshot(
        repository_root / AGGREGATE_ARGV[0],
        "aggregate Lean wrapper",
        MAX_MEMBER_BYTES,
    )
    lean_root = repository_root / "formalization/lean"
    lake_manifest = _read_file_snapshot(
        lean_root / "lake-manifest.json",
        "Lean lake manifest",
        MAX_MEMBER_BYTES,
    )
    cache_metadata = _dependency_cache_metadata_snapshot(lean_root)
    local_modules = _local_module_snapshots(repository_root, roots)
    required_artifacts = _required_mathlib_artifact_snapshots(
        lean_root, _required_mathlib_modules(local_modules)
    )
    return _ExecutionSnapshot(
        wrapper=wrapper,
        lake_manifest=lake_manifest,
        dependency_cache_metadata_sha256=_snapshot_digest(cache_metadata),
        required_mathlib_artifacts=required_artifacts,
        required_mathlib_artifacts_sha256=_artifact_snapshot_digest(required_artifacts),
        local_modules=local_modules,
        local_source_closure_sha256=_source_snapshot_digest(local_modules),
    )


def _require_execution_snapshot(
    repository_root: Path, roots: Sequence[str], expected: _ExecutionSnapshot
) -> None:
    observed = _read_execution_snapshot(repository_root, roots)
    if observed.wrapper != expected.wrapper:
        raise protocol.ValidationError(
            "aggregate Lean wrapper changed during receipt execution"
        )
    if observed.lake_manifest != expected.lake_manifest:
        raise protocol.ValidationError(
            "Lean lake manifest changed during receipt execution"
        )
    if observed.required_mathlib_artifacts != expected.required_mathlib_artifacts:
        raise protocol.ValidationError(
            "required Mathlib artifacts changed during receipt execution"
        )
    if (
        observed.dependency_cache_metadata_sha256
        != expected.dependency_cache_metadata_sha256
    ):
        raise protocol.ValidationError(
            "Lean dependency cache metadata changed during receipt execution"
        )
    if observed.local_modules != expected.local_modules:
        raise protocol.ValidationError(
            "LS local source module closure changed during receipt execution"
        )


def _dependency_cache_metadata_snapshot(
    lean_root: Path,
) -> tuple[tuple[object, ...], ...]:
    lake_root = _resolve_approved_lake_root(lean_root)
    lake = _pin_absolute_directory(lake_root, "approved Lean .lake root")
    packages: _PinnedDirectory | None = None
    try:
        packages = _pin_directory_at(
            lake, "packages", lake.path / "packages", "Lean dependency packages"
        )
        records: list[tuple[object, ...]] = []
        with os.scandir(packages.descriptor) as package_entries:
            for package_entry in package_entries:
                if len(records) >= MAX_CACHE_ENTRIES:
                    raise protocol.ValidationError(
                        "Lean dependency cache entry count exceeds safety cap"
                    )
                metadata = package_entry.stat(follow_symlinks=False)
                if stat.S_ISLNK(metadata.st_mode):
                    raise protocol.ValidationError(
                        "Lean dependency cache package cannot be a symlink: "
                        + package_entry.name
                    )
                if not stat.S_ISDIR(metadata.st_mode):
                    raise protocol.ValidationError(
                        "Lean dependency package must be a directory: "
                        + package_entry.name
                    )
                package = _pin_directory_at(
                    packages,
                    package_entry.name,
                    packages.path / package_entry.name,
                    f"Lean dependency package {package_entry.name}",
                )
                package_lake: _PinnedDirectory | None = None
                build: _PinnedDirectory | None = None
                try:
                    try:
                        package_lake = _pin_directory_at(
                            package,
                            ".lake",
                            package.path / ".lake",
                            f"Lean dependency package cache {package_entry.name}",
                        )
                    except FileNotFoundError:
                        continue
                    try:
                        build = _pin_directory_at(
                            package_lake,
                            "build",
                            package_lake.path / "build",
                            f"Lean dependency build root for {package_entry.name}",
                        )
                    except FileNotFoundError:
                        continue
                    build_metadata = os.fstat(build.descriptor)
                    records.append(
                        (
                            f"{package_entry.name}/.lake/build",
                            "directory",
                            build_metadata.st_dev,
                            build_metadata.st_ino,
                            build_metadata.st_mode,
                            build_metadata.st_size,
                            build_metadata.st_mtime_ns,
                            build_metadata.st_ctime_ns,
                        )
                    )
                    _walk_cache_tree_at(
                        build, (package_entry.name, ".lake", "build"), records
                    )
                finally:
                    if build is not None:
                        _close_pinned_directory(build)
                    if package_lake is not None:
                        _close_pinned_directory(package_lake)
                    _close_pinned_directory(package)
        return tuple(sorted(records))
    finally:
        if packages is not None:
            _close_pinned_directory(packages)
        _close_pinned_directory(lake)


def _walk_cache_tree_at(
    root: _PinnedDirectory,
    prefix: tuple[str, ...],
    records: list[tuple[object, ...]],
) -> None:
    pending: list[tuple[_PinnedDirectory, tuple[str, ...]]] = [(root, prefix)]
    opened: list[_PinnedDirectory] = []
    try:
        while pending:
            directory, parts = pending.pop()
            with os.scandir(directory.descriptor) as entries:
                for entry in entries:
                    metadata = entry.stat(follow_symlinks=False)
                    relative_parts = (*parts, entry.name)
                    relative = "/".join(relative_parts)
                    if stat.S_ISLNK(metadata.st_mode):
                        raise protocol.ValidationError(
                            f"Lean dependency cache cannot contain symlinks: {relative}"
                        )
                    if stat.S_ISDIR(metadata.st_mode):
                        entry_type = "directory"
                        child = _pin_directory_at(
                            directory,
                            entry.name,
                            directory.path / entry.name,
                            "Lean dependency cache directory",
                        )
                        opened.append(child)
                        pending.append((child, relative_parts))
                    elif stat.S_ISREG(metadata.st_mode):
                        entry_type = "file"
                    else:
                        raise protocol.ValidationError(
                            f"Lean dependency cache contains unsupported entry: {relative}"
                        )
                    if len(records) >= MAX_CACHE_ENTRIES:
                        raise protocol.ValidationError(
                            "Lean dependency cache entry count exceeds safety cap"
                        )
                    records.append(
                        (
                            relative,
                            entry_type,
                            metadata.st_dev,
                            metadata.st_ino,
                            metadata.st_mode,
                            metadata.st_size,
                            metadata.st_mtime_ns,
                            metadata.st_ctime_ns,
                        )
                    )
    finally:
        for directory in reversed(opened):
            _close_pinned_directory(directory)


def _bounded_sorted_scandir(
    directory: Path, maximum: int, label: str
) -> list[os.DirEntry[str]]:
    entries: list[os.DirEntry[str]] = []
    try:
        with os.scandir(directory) as iterator:
            for entry in iterator:
                if len(entries) >= maximum:
                    raise protocol.ValidationError(
                        f"{label} entry count exceeds safety cap"
                    )
                entries.append(entry)
    except protocol.ValidationError:
        raise
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    entries.sort(key=lambda item: item.name)
    return entries


def _required_mathlib_artifact_snapshots(
    lean_root: Path,
    modules: Sequence[str],
) -> tuple[tuple[str, _FileSnapshot], ...]:
    lake = _pin_absolute_directory(
        _resolve_approved_lake_root(lean_root), "approved Lean .lake root"
    )
    try:
        return tuple(
            (
                module,
                _read_file_snapshot_relative_at(
                    lake,
                    (
                        "packages",
                        "mathlib",
                        ".lake",
                        "build",
                        "lib",
                        "lean",
                        *module.split(".")[:-1],
                        module.split(".")[-1] + ".olean",
                    ),
                    f"required Mathlib artifact for {module}",
                    MAX_MEMBER_BYTES,
                ),
            )
            for module in modules
        )
    finally:
        _close_pinned_directory(lake)


def _required_mathlib_modules(
    local_modules: Sequence[tuple[str, _FileSnapshot]],
) -> tuple[str, ...]:
    modules = set()
    for module, snapshot in local_modules:
        try:
            source = snapshot.data.decode("utf-8", errors="strict")
        except UnicodeDecodeError as error:
            raise protocol.ValidationError(
                f"cannot decode LS local source module {module}: {error}"
            ) from error
        try:
            imports = provider_independence.parse_active_imports(source, module)
        except provider_independence.ProviderIndependenceError as error:
            raise protocol.ValidationError(
                f"cannot parse active Crouzeix imports: {error}"
            ) from error
        for token in imports:
            if not token.startswith("Mathlib"):
                continue
            if LEAN_NAME.fullmatch(token) is None:
                raise protocol.ValidationError(
                    f"LS local source has invalid Mathlib import: {token}"
                )
            modules.add(token)
    return tuple(sorted(modules))


def _snapshot_digest(records: tuple[tuple[object, ...], ...]) -> str:
    return protocol.sha256_bytes(_canonical_json_bytes({"entries": records}))


def _artifact_snapshot_digest(
    snapshots: tuple[tuple[str, _FileSnapshot], ...],
) -> str:
    records = [
        {
            "module": module,
            "sha256": protocol.sha256_bytes(snapshot.data),
        }
        for module, snapshot in snapshots
    ]
    return protocol.sha256_bytes(_canonical_json_bytes({"artifacts": records}))


def _source_snapshot_digest(
    snapshots: Sequence[tuple[str, _FileSnapshot]],
) -> str:
    records = [
        {"module": module, "sha256": protocol.sha256_bytes(snapshot.data)}
        for module, snapshot in snapshots
    ]
    return protocol.sha256_bytes(_canonical_json_bytes({"sources": records}))


def _validate_aggregate_stdout(data: bytes) -> None:
    try:
        lines = data.decode("utf-8").splitlines()
    except UnicodeDecodeError as error:
        raise protocol.ValidationError(
            f"aggregate stdout is not UTF-8: {error}"
        ) from error
    required = (
        f"[lean] target={ls_contract.AGGREGATE_MODULE}",
        "[lean] root=formalization/lean",
        "[lean] outcome=passed",
    )
    if any(lines.count(marker) != 1 for marker in required) or any(
        line.startswith("[lean] outcome=") and line != required[-1] for line in lines
    ):
        raise protocol.ValidationError(
            "aggregate stdout does not contain exact success markers"
        )


def _validate_wrapper_cache_policy(data: bytes) -> None:
    try:
        source = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise protocol.ValidationError(
            "aggregate Lean wrapper cache policy is not UTF-8"
        ) from error
    source = source.replace("\\\r\n", "").replace("\\\n", "")
    active_tokens = []
    for line in source.splitlines():
        active_tokens.extend(_strip_shell_comment(line).split())
    normalized = " ".join(active_tokens)
    forbidden_patterns = []
    for command in WRAPPER_FORBIDDEN_COMMANDS:
        command_pattern = re.escape(command).replace("\\ ", r"\s+")
        forbidden_patterns.append(
            rf"(?<![A-Za-z0-9_]){command_pattern}(?![A-Za-z0-9_])"
        )
    if any(
        re.search(pattern, normalized)
        for pattern in forbidden_patterns
    ):
        raise protocol.ValidationError(
            "aggregate Lean wrapper cache policy contains a forbidden hydration command"
        )


def _strip_shell_comment(line: str) -> str:
    quote: str | None = None
    escaped = False
    for index, character in enumerate(line):
        if escaped:
            escaped = False
            continue
        if character == "\\" and quote != "'":
            escaped = True
            continue
        if character in {"'", '"'}:
            if quote is None:
                quote = character
            elif quote == character:
                quote = None
            continue
        if (
            character == "#"
            and quote is None
            and (index == 0 or line[index - 1].isspace())
        ):
            return line[:index]
    return line


def _run_checked(
    executor: Executor,
    argv: list[str],
    cwd: Path,
    environment: dict[str, str],
    label: str,
) -> CommandResult:
    result = executor(argv, cwd, dict(environment), TIMEOUT_SECONDS, MAX_OUTPUT_BYTES)
    if not isinstance(result, CommandResult):
        raise protocol.ValidationError("LS executor must return CommandResult")
    if not isinstance(result.stdout, bytes) or not isinstance(result.stderr, bytes):
        raise protocol.ValidationError("LS executor output must be bytes")
    if result.exit_code is not None and (
        not isinstance(result.exit_code, int) or isinstance(result.exit_code, bool)
    ):
        raise protocol.ValidationError("LS executor exit_code is invalid")
    if (
        result.stdout_truncated
        or result.stderr_truncated
        or len(result.stdout) > MAX_OUTPUT_BYTES
        or len(result.stderr) > MAX_OUTPUT_BYTES
    ):
        raise protocol.ValidationError(f"LS {label} exceeded output cap")
    if result.blocked_reason is not None or result.exit_code is None:
        reason = result.blocked_reason or "command did not produce an exit code"
        raise protocol.ValidationError(f"LS {label} was blocked: {reason}")
    if result.exit_code != 0:
        raise protocol.ValidationError(
            f"LS {label} failed with exit code {result.exit_code}"
        )
    return result


def _run_axiom_audit(
    executor: Executor,
    plan: _AttemptPlan,
    lean_root: Path,
    lean_directory: _PinnedDirectory,
    environment: dict[str, str],
) -> bytes:
    module = (
        PurePosixPath(plan.build_target)
        .relative_to("formalization/lean")
        .with_suffix("")
    )
    module_name = ".".join(module.parts)
    source = (f"import {module_name}\n#print axioms {plan.declaration}\n").encode(
        "utf-8"
    )
    audit_directory = _create_unique_directory_at(
        lean_directory, AUDIT_PREFIX, "LS axiom audit directory"
    )
    try:
        _write_bytes_create_only_at(
            audit_directory.descriptor,
            "Audit.lean",
            source,
            "LS axiom audit source",
        )
        relative_source = f"{audit_directory.entry_name}/Audit.lean"
        result = _run_checked(
            executor,
            ["lake", "env", "lean", relative_source],
            lean_root,
            environment,
            f"axiom audit for {plan.row.node_id}",
        )
        return result.stdout
    finally:
        try:
            _unlink_if_present(audit_directory.descriptor, "Audit.lean")
            _rmdir_pinned_entry(lean_directory, audit_directory)
        finally:
            _close_pinned_directory(audit_directory)


def _parse_axiom_audit(data: bytes, expected_declaration: str) -> tuple[str, ...]:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise protocol.ValidationError("LS axiom audit is not UTF-8") from error
    match = AXIOM_AUDIT.fullmatch(text)
    if match is None:
        match = AXIOM_AUDIT_NONE.fullmatch(text)
        if match is None:
            raise protocol.ValidationError("LS axiom audit output is malformed")
        observed: list[str] = []
    else:
        raw_axioms = match.group("axioms").strip()
        observed = (
            [] if not raw_axioms else [item.strip() for item in raw_axioms.split(",")]
        )
    if match.group("declaration") != expected_declaration:
        raise protocol.ValidationError(
            "LS axiom audit declaration does not match expected declaration"
        )
    if any(LEAN_NAME.fullmatch(item) is None for item in observed):
        raise protocol.ValidationError("LS axiom audit contains an invalid axiom")
    if len(set(observed)) != len(observed):
        raise protocol.ValidationError("LS axiom audit contains duplicate axioms")
    unexpected = sorted(set(observed) - set(ALLOWED_AXIOMS))
    if unexpected:
        raise protocol.ValidationError(
            "LS axiom audit contains disallowed axioms: " + ", ".join(unexpected)
        )
    return tuple(sorted(observed))


def _stage_bundle(
    attempt: _PinnedDirectory,
    plan: _AttemptPlan,
    aggregate: CommandResult,
    audit_bytes: bytes,
    observed_axioms: tuple[str, ...],
    module_bytes: bytes,
    execution_snapshot: _ExecutionSnapshot,
) -> None:
    row = plan.row
    try:
        os.mkdir("build", mode=0o700, dir_fd=attempt.descriptor)
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot create staged LS build directory: {error}"
        ) from error
    task = {
        "schema_version": "crouzeix-ls-proof-slice-task/v1",
        "route_id": "lorist-schwenninger",
        "source_node_id": row.node_id,
        "build_target": plan.build_target,
        "expected_lean_declaration": plan.declaration,
        "max_output_bytes": MAX_OUTPUT_BYTES,
        "timeout_seconds": TIMEOUT_SECONDS,
    }
    source_slice = {
        "schema_version": "crouzeix-ls-source-slice/v1",
        "node_id": row.node_id,
        "source_locator": row.source_locator,
        "statement_sha256": row.statement_sha256,
        "lean_name": plan.declaration,
        "dependency_ids": list(row.dependencies),
    }
    reason = (
        "Lean command succeeded; Crouzeix aggregate build passed; axiom audit "
        f"reported only allowed axioms; published as {plan.attempt_name}."
    )
    result = {
        "schema_version": "crouzeix-ls-proof-slice-result/v1",
        "status": "passed",
        "reason": reason,
    }
    command = {
        "schema_version": "crouzeix-ls-lean-command/v2",
        "argv": list(AGGREGATE_ARGV),
        "cache_policy": CACHE_POLICY,
        "cwd": ".",
        "env": _shared_environment(),
        "elan_toolchain": SHARED_LEAN_TOOLCHAIN,
        "exit_code": aggregate.exit_code,
        "output_cap_bytes": MAX_OUTPUT_BYTES,
        "timeout_seconds": TIMEOUT_SECONDS,
        "wrapper_sha256": protocol.sha256_bytes(execution_snapshot.wrapper.data),
        "lake_manifest_sha256": protocol.sha256_bytes(
            execution_snapshot.lake_manifest.data
        ),
        "dependency_cache_metadata_sha256": (
            execution_snapshot.dependency_cache_metadata_sha256
        ),
        "required_mathlib_artifacts_sha256": (
            execution_snapshot.required_mathlib_artifacts_sha256
        ),
        "local_source_closure_sha256": (execution_snapshot.local_source_closure_sha256),
    }
    members = {
        "task_sha256": ("task.json", _canonical_json_bytes(task)),
        "source_slice_sha256": (
            "source-slice.json",
            _canonical_json_bytes(source_slice),
        ),
        "result_sha256": ("result.json", _canonical_json_bytes(result)),
        "command_sha256": (
            "build/command.json",
            _canonical_json_bytes(command),
        ),
        "stdout_sha256": ("build/stdout.log", aggregate.stdout),
        "stderr_sha256": ("build/stderr.log", aggregate.stderr),
        "axiom_audit_sha256": ("build/axioms.txt", audit_bytes),
    }
    for digest_field, (relative_path, data) in members.items():
        limit = (
            MAX_OUTPUT_BYTES
            if digest_field
            in {
                "stdout_sha256",
                "stderr_sha256",
            }
            else MAX_MEMBER_BYTES
        )
        if len(data) > limit:
            raise protocol.ValidationError(
                f"LS staged member exceeds byte cap: {relative_path}"
            )
        parent = attempt
        name = relative_path
        build: _PinnedDirectory | None = None
        if relative_path.startswith("build/"):
            build = _pin_directory_at(
                attempt, "build", attempt.path / "build", "LS staged build"
            )
            parent = build
            name = relative_path.removeprefix("build/")
        try:
            _write_bytes_create_only_at(
                parent.descriptor, name, data, f"LS staged {relative_path}"
            )
        finally:
            if build is not None:
                os.close(build.descriptor)

    receipt: dict[str, object] = {
        "schema_version": "crouzeix-ls-proof-slice-receipt/v1",
        "route_id": "lorist-schwenninger",
        "source_node_id": row.node_id,
        "status": "passed",
        "reason": reason,
        "build_target": plan.build_target,
        "expected_lean_declaration": plan.declaration,
        "allowed_axioms": list(ALLOWED_AXIOMS),
        "observed_axioms": list(observed_axioms),
        "module_sha256": protocol.sha256_bytes(module_bytes),
    }
    for digest_field, (_, data) in members.items():
        receipt[digest_field] = protocol.sha256_bytes(data)
    _write_bytes_create_only_at(
        attempt.descriptor,
        "receipt.json",
        _canonical_json_bytes(receipt),
        "LS staged receipt",
    )


def _publish_staged_bundles(
    stage: _PinnedDirectory,
    proof_slices: _PinnedDirectory,
    plans: tuple[_AttemptPlan, ...],
    repository_root: Path,
    module_snapshots: Mapping[str, _FileSnapshot],
    execution_snapshot: _ExecutionSnapshot,
    publication_candidates: list[_PublicationCandidate],
) -> dict[str, dict[str, str]]:
    pinned_node_roots: dict[str, _PinnedDirectory] = {}
    created_node_roots: list[_PinnedDirectory] = []
    staged_attempts: list[_PinnedDirectory] = []
    hidden_attempts: list[tuple[_PinnedDirectory, _PinnedDirectory]] = []
    published_attempts: list[
        tuple[_AttemptPlan, _PinnedDirectory, _PinnedDirectory]
    ] = []
    candidate_receipt_digests: dict[str, str] = {}
    expected_bundle_bytes: dict[str, dict[str, bytes]] = {}
    published: dict[str, dict[str, str]] = {}
    try:
        for plan in plans:
            node_root, created = _pin_node_root(proof_slices, plan)
            pinned_node_roots[plan.row.node_id] = node_root
            if created:
                created_node_roots.append(node_root)
        for plan in plans:
            node_root = pinned_node_roots[plan.row.node_id]
            staged_attempt = _pin_stage_attempt(stage, plan)
            staged_attempts.append(staged_attempt)
            expected_bundle_bytes[plan.row.node_id] = _read_bundle_bytes_at(
                staged_attempt, "staged LS bundle"
            )
            hidden = _copy_bundle_create_only(
                staged_attempt,
                node_root,
                f".{plan.attempt_name}.{uuid.uuid4().hex}.staging",
            )
            hidden_attempts.append((node_root, hidden))
            candidate_receipt_digests[plan.row.node_id] = protocol.sha256_bytes(
                expected_bundle_bytes[plan.row.node_id]["receipt.json"]
            )
            publication_candidates.append(
                _PublicationCandidate(
                    plan=plan,
                    node_root=node_root,
                    hidden_attempt=hidden,
                    metadata={
                        "attempt_path": (node_root.path / plan.attempt_name)
                        .relative_to(repository_root)
                        .as_posix(),
                        "receipt_sha256": candidate_receipt_digests[plan.row.node_id],
                    },
                )
            )
            _require_bundle_bytes(
                hidden,
                expected_bundle_bytes[plan.row.node_id],
                candidate_receipt_digests[plan.row.node_id],
                "hidden candidate",
            )
        _require_module_snapshots(repository_root, plans, module_snapshots)
        _require_execution_snapshot(
            repository_root,
            (ls_contract.AGGREGATE_MODULE,),
            execution_snapshot,
        )
        for candidate in publication_candidates:
            plan = candidate.plan
            node_root = candidate.node_root
            hidden = candidate.hidden_attempt
            candidate.rename_attempted = True
            published_attempt = _rename_directory_no_replace(
                node_root, hidden, plan.attempt_name, candidate=candidate
            )
            hidden_attempts.remove((node_root, hidden))
            published_attempts.append((plan, node_root, published_attempt))
            _fsync_descriptor(node_root.descriptor, "LS node root after publication")
            _require_pinned_directory(node_root, "LS node root")
            _require_pinned_directory(published_attempt, "published LS attempt")
            for published_plan, _, published_bundle in published_attempts:
                _require_bundle_bytes(
                    published_bundle,
                    expected_bundle_bytes[published_plan.row.node_id],
                    candidate_receipt_digests[published_plan.row.node_id],
                    f"published bundle for {published_plan.row.node_id}",
                )
            published[plan.row.node_id] = {
                "attempt_path": candidate.metadata["attempt_path"],
                "receipt_sha256": candidate_receipt_digests[plan.row.node_id],
                "lean_name": plan.declaration,
                "status": "passed",
            }
        _require_module_snapshots(repository_root, plans, module_snapshots)
        _require_execution_snapshot(
            repository_root,
            (ls_contract.AGGREGATE_MODULE,),
            execution_snapshot,
        )
        for node_root in pinned_node_roots.values():
            _require_pinned_directory(node_root, "LS node root")
        _require_pinned_directory(proof_slices, "LS proof-slices root")
        for plan, _, attempt in published_attempts:
            _require_bundle_bytes(
                attempt,
                expected_bundle_bytes[plan.row.node_id],
                candidate_receipt_digests[plan.row.node_id],
                f"published receipt for {plan.row.node_id}",
            )
    except BaseException as error:
        _reconcile_publication_visibility(publication_candidates)
        for candidate in publication_candidates:
            if not candidate.visible:
                continue
            try:
                if candidate.published_attempt is None:
                    raise protocol.ValidationError(
                        "published LS attempt identity cannot be revalidated"
                    )
                _require_bundle_bytes(
                    candidate.published_attempt,
                    expected_bundle_bytes[candidate.plan.row.node_id],
                    candidate.metadata["receipt_sha256"],
                    f"published receipt for {candidate.plan.row.node_id}",
                )
            except BaseException as validation_error:
                error = protocol.ValidationError(
                    "LS publication and post-rename validation both failed: "
                    f"{error}; validation: {validation_error}"
                )
        cleanup_errors: list[BaseException] = []
        candidates = _visible_candidate_metadata(publication_candidates)
        if candidates:
            for candidate in publication_candidates:
                if candidate.visible:
                    continue
                try:
                    _detach_hidden_attempt(candidate)
                except BaseException as cleanup_error:
                    cleanup_errors.append(cleanup_error)
            _reconcile_publication_visibility(publication_candidates)
            candidates = _visible_candidate_metadata(publication_candidates)
            for candidate in publication_candidates:
                if not candidate.visible:
                    continue
                try:
                    if candidate.published_attempt is None:
                        raise protocol.ValidationError(
                            "published LS attempt identity cannot be revalidated"
                        )
                    _require_bundle_bytes(
                        candidate.published_attempt,
                        expected_bundle_bytes[candidate.plan.row.node_id],
                        candidate.metadata["receipt_sha256"],
                        f"published receipt for {candidate.plan.row.node_id}",
                    )
                except BaseException as validation_error:
                    cleanup_errors.append(validation_error)
        else:
            for node_root, hidden in reversed(hidden_attempts):
                try:
                    _remove_bundle_at(node_root, hidden, hidden=True)
                except BaseException as cleanup_error:
                    cleanup_errors.append(cleanup_error)
        published_node_descriptors = {
            candidate.node_root.descriptor
            for candidate in publication_candidates
            if candidate.visible
        }
        for node_root in reversed(created_node_roots):
            if node_root.descriptor in published_node_descriptors:
                continue
            try:
                _remove_created_node_root(proof_slices, node_root)
            except BaseException as cleanup_error:
                cleanup_errors.append(cleanup_error)
        if candidates:
            details = ", ".join(
                f"{item['attempt_path']} receipt_sha256={item['receipt_sha256']}"
                for item in candidates
            )
            if cleanup_errors:
                details += "; hidden cleanup failures: " + "; ".join(
                    str(item) for item in cleanup_errors
                )
            raise PartialPublicationError(
                "LS committed partial publication; visible candidates remain "
                f"unreferenced: {details}; cause: {error}",
                candidates,
            ) from error
        if cleanup_errors:
            raise protocol.ValidationError(
                "LS publication rollback encountered cleanup failures: "
                + "; ".join(str(item) for item in cleanup_errors)
            ) from error
        raise
    finally:
        close_errors: list[OSError] = []
        attempt_descriptors = {
            attempt.descriptor
            for attempt in (
                *staged_attempts,
                *(attempt for _, attempt in hidden_attempts),
                *(attempt for _, _, attempt in published_attempts),
            )
        }
        for descriptor in attempt_descriptors:
            try:
                os.close(descriptor)
            except OSError as error:
                close_errors.append(error)
        for node_root in pinned_node_roots.values():
            try:
                os.close(node_root.descriptor)
            except OSError as error:
                close_errors.append(error)
        if close_errors:
            message = "cannot close pinned LS publication directories: " + "; ".join(
                str(item) for item in close_errors
            )
            candidates = _visible_candidate_metadata(publication_candidates)
            if candidates:
                details = ", ".join(
                    f"{item['attempt_path']} receipt_sha256={item['receipt_sha256']}"
                    for item in candidates
                )
                raise PartialPublicationError(
                    "LS committed partial publication; visible candidates remain "
                    f"unreferenced: {details}; cause: {message}",
                    candidates,
                )
            raise protocol.ValidationError(message)
    return published


def _copy_bundle_create_only(
    source: Path | _PinnedDirectory,
    node_root: _PinnedDirectory,
    destination_name: str,
) -> _PinnedDirectory:
    _require_pinned_directory(node_root, "LS node root")
    _validate_attempt_entry_name(destination_name, hidden=True)
    try:
        os.mkdir(destination_name, mode=0o700, dir_fd=node_root.descriptor)
    except FileExistsError as error:
        raise protocol.ValidationError(
            f"LS attempt already exists: {node_root.path / destination_name}"
        ) from error
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot create hidden LS attempt: {error}"
        ) from error
    destination = _pin_directory_at(
        node_root,
        destination_name,
        node_root.path / destination_name,
        "hidden LS attempt",
    )
    source_directory: _PinnedDirectory
    close_source = False
    if isinstance(source, _PinnedDirectory):
        source_directory = source
    else:
        source_directory = _pin_absolute_directory(
            _absolute_normalized(source), "staged LS attempt"
        )
        close_source = True
    try:
        _require_pinned_directory(source_directory, "staged LS attempt")
        os.mkdir("build", mode=0o700, dir_fd=destination.descriptor)
        source_build = _pin_directory_at(
            source_directory,
            "build",
            source_directory.path / "build",
            "staged LS build directory",
        )
        build = _pin_directory_at(
            destination,
            "build",
            destination.path / "build",
            "hidden LS build directory",
        )
        try:
            for relative_path in BUNDLE_ROOT_FILES:
                data = _read_safe_file_at(
                    source_directory.descriptor,
                    relative_path,
                    f"staged LS member {relative_path}",
                    MAX_MEMBER_BYTES,
                )
                _write_bytes_create_only_at(
                    destination.descriptor,
                    relative_path,
                    data,
                    f"published LS member {relative_path}",
                )
            for name in BUNDLE_BUILD_FILES:
                relative_path = f"build/{name}"
                data = _read_safe_file_at(
                    source_build.descriptor,
                    name,
                    f"staged LS member {relative_path}",
                    MAX_MEMBER_BYTES,
                )
                _write_bytes_create_only_at(
                    build.descriptor,
                    name,
                    data,
                    f"published LS member {relative_path}",
                )
        finally:
            _close_pinned_directory(source_build)
            _fsync_descriptor(build.descriptor, "hidden LS build directory")
            os.close(build.descriptor)
        _fsync_descriptor(destination.descriptor, "hidden LS attempt")
        _require_pinned_directory(node_root, "LS node root")
        _require_pinned_directory(destination, "hidden LS attempt")
    except BaseException as error:
        try:
            _remove_bundle_at(node_root, destination, hidden=True)
        except BaseException as cleanup_error:
            os.close(destination.descriptor)
            raise protocol.ValidationError(
                f"cannot clean incomplete hidden LS attempt: {cleanup_error}"
            ) from error
        os.close(destination.descriptor)
        raise
    finally:
        if close_source:
            _close_pinned_directory(source_directory)
    return destination


def _read_bundle_file_at(
    attempt: _PinnedDirectory,
    member_name: str,
    label: str,
    maximum: int,
) -> bytes:
    _require_pinned_directory(attempt, "published LS attempt")
    data = _read_safe_file_at(attempt.descriptor, member_name, label, maximum)
    _require_pinned_directory(attempt, "published LS attempt")
    return data


def _entry_matches_pinned_directory(
    parent: _PinnedDirectory, name: str | None, directory: _PinnedDirectory
) -> bool:
    if name is None:
        return False
    try:
        metadata = os.stat(name, dir_fd=parent.descriptor, follow_symlinks=False)
    except OSError:
        return False
    return (
        stat.S_ISDIR(metadata.st_mode)
        and not stat.S_ISLNK(metadata.st_mode)
        and (metadata.st_dev, metadata.st_ino) == (directory.device, directory.inode)
    )


def _reconcile_publication_visibility(
    candidates: Sequence[_PublicationCandidate],
) -> None:
    for candidate in candidates:
        if not candidate.rename_attempted:
            continue
        if candidate.visible:
            if candidate.published_attempt is None and _entry_matches_pinned_directory(
                candidate.node_root,
                candidate.plan.attempt_name,
                candidate.hidden_attempt,
            ):
                candidate.published_attempt = _published_attempt_from_hidden(candidate)
            continue
        hidden = _entry_matches_pinned_directory(
            candidate.node_root,
            candidate.hidden_attempt.entry_name,
            candidate.hidden_attempt,
        )
        if hidden:
            continue
        candidate.visible = True
        if _entry_matches_pinned_directory(
            candidate.node_root,
            candidate.plan.attempt_name,
            candidate.hidden_attempt,
        ):
            candidate.published_attempt = _published_attempt_from_hidden(candidate)


def _published_attempt_from_hidden(
    candidate: _PublicationCandidate,
) -> _PinnedDirectory:
    return _PinnedDirectory(
        path=candidate.node_root.path / candidate.plan.attempt_name,
        descriptor=candidate.hidden_attempt.descriptor,
        device=candidate.hidden_attempt.device,
        inode=candidate.hidden_attempt.inode,
        parent_descriptor=candidate.node_root.descriptor,
        entry_name=candidate.plan.attempt_name,
    )


def _detach_hidden_attempt(candidate: _PublicationCandidate) -> None:
    hidden_name = candidate.hidden_attempt.entry_name
    if hidden_name is None:
        raise protocol.ValidationError(
            "cannot prove hidden LS attempt is safe to detach"
        )
    if not _entry_matches_pinned_directory(
        candidate.node_root, hidden_name, candidate.hidden_attempt
    ):
        raise protocol.ValidationError(
            "cannot prove hidden LS attempt is safe to detach"
        )
    detached_name = f".abandoned-{uuid.uuid4().hex}"
    os.rename(
        hidden_name,
        detached_name,
        src_dir_fd=candidate.node_root.descriptor,
        dst_dir_fd=candidate.node_root.descriptor,
    )
    candidate.hidden_attempt = _PinnedDirectory(
        path=candidate.node_root.path / detached_name,
        descriptor=candidate.hidden_attempt.descriptor,
        device=candidate.hidden_attempt.device,
        inode=candidate.hidden_attempt.inode,
        parent_descriptor=candidate.node_root.descriptor,
        entry_name=detached_name,
    )


def _visible_candidate_metadata(
    candidates: Sequence[_PublicationCandidate],
) -> tuple[dict[str, str], ...]:
    return tuple(candidate.metadata for candidate in candidates if candidate.visible)


def _read_bundle_bytes_at(attempt: _PinnedDirectory, label: str) -> dict[str, bytes]:
    _require_bundle_entries(
        attempt, (*BUNDLE_ROOT_FILES, "build"), f"{label} root directory"
    )
    build = _pin_directory_at(
        attempt,
        "build",
        attempt.path / "build",
        f"{label} build directory",
        require_parent_path=False,
    )
    try:
        _require_bundle_entries(build, BUNDLE_BUILD_FILES, f"{label} build directory")
        members = {
            name: _read_bundle_file_at(
                attempt, name, f"{label} member {name}", MAX_MEMBER_BYTES
            )
            for name in BUNDLE_ROOT_FILES
        }
        members.update(
            {
                f"build/{name}": _read_bundle_file_at(
                    build,
                    name,
                    f"{label} member build/{name}",
                    MAX_OUTPUT_BYTES
                    if name in {"stdout.log", "stderr.log"}
                    else MAX_MEMBER_BYTES,
                )
                for name in BUNDLE_BUILD_FILES
            }
        )
        _require_bundle_entries(build, BUNDLE_BUILD_FILES, f"{label} build directory")
        _require_bundle_entries(
            attempt, (*BUNDLE_ROOT_FILES, "build"), f"{label} root directory"
        )
        return members
    finally:
        _close_pinned_directory(build)


def _require_bundle_entries(
    directory: _PinnedDirectory, expected: Sequence[str], label: str
) -> None:
    _require_pinned_directory(directory, label)
    expected_names = set(expected)
    observed: set[str] = set()
    try:
        with os.scandir(directory.descriptor) as entries:
            for entry in entries:
                _safe_entry_name(entry.name, label)
                observed.add(entry.name)
                if len(observed) > len(expected_names):
                    raise protocol.ValidationError(f"{label} members changed")
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if observed != expected_names:
        raise protocol.ValidationError(f"{label} members changed")
    _require_pinned_directory(directory, label)


def _require_bundle_bytes(
    attempt: _PinnedDirectory,
    expected: Mapping[str, bytes],
    expected_receipt_digest: str,
    label: str,
) -> None:
    observed = _read_bundle_bytes_at(attempt, label)
    for relative_path, expected_bytes in expected.items():
        if observed[relative_path] != expected_bytes:
            raise protocol.ValidationError(
                f"{label} member bytes changed: {relative_path}"
            )
    if protocol.sha256_bytes(observed["receipt.json"]) != expected_receipt_digest:
        raise protocol.ValidationError(f"{label} receipt digest changed")


def _remove_bundle_at(
    node_root: _PinnedDirectory, attempt: _PinnedDirectory, *, hidden: bool
) -> None:
    if attempt.parent_descriptor != node_root.descriptor or attempt.entry_name is None:
        raise protocol.ValidationError("LS rollback attempt has invalid parent")
    _validate_attempt_entry_name(attempt.entry_name, hidden=hidden)
    if not _entry_matches_pinned_directory(node_root, attempt.entry_name, attempt):
        raise protocol.ValidationError(
            "cannot prove LS rollback attempt is safe to remove"
        )
    _require_descriptor_identity(attempt, "LS attempt rollback directory")
    try:
        build = _pin_directory_at(
            attempt,
            "build",
            attempt.path / "build",
            "LS build rollback directory",
            require_parent_path=False,
        )
    except FileNotFoundError:
        build = None
    if build is not None:
        try:
            for name in BUNDLE_BUILD_FILES:
                _unlink_if_present(build.descriptor, name)
        finally:
            os.close(build.descriptor)
        _rmdir_if_present(attempt.descriptor, "build")
    for name in BUNDLE_ROOT_FILES:
        _unlink_if_present(attempt.descriptor, name)
    _rmdir_pinned_entry(node_root, attempt)


def _remove_created_node_root(
    proof_slices: _PinnedDirectory, node_root: _PinnedDirectory
) -> None:
    _require_descriptor_identity(node_root, "LS node root")
    metadata = os.stat(
        node_root.path.name, dir_fd=proof_slices.descriptor, follow_symlinks=False
    )
    if (metadata.st_dev, metadata.st_ino) != (node_root.device, node_root.inode):
        raise protocol.ValidationError(
            f"LS node root identity changed during rollback: {node_root.path}"
        )
    try:
        os.rmdir(node_root.path.name, dir_fd=proof_slices.descriptor)
    except OSError as error:
        if error.errno in (errno.ENOTEMPTY, errno.EEXIST):
            return
        raise


def _pin_node_root(
    proof_slices: _PinnedDirectory, plan: _AttemptPlan
) -> tuple[_PinnedDirectory, bool]:
    if (
        plan.node_root.parent != proof_slices.path
        or plan.node_root.name != plan.row.node_id
    ):
        raise protocol.ValidationError("LS node root does not match publication plan")
    _require_pinned_directory(proof_slices, "LS proof-slices root")
    created = False
    try:
        os.mkdir(plan.row.node_id, mode=0o700, dir_fd=proof_slices.descriptor)
        created = True
    except FileExistsError:
        pass
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot create LS node root for {plan.row.node_id}: {error}"
        ) from error
    try:
        metadata = os.stat(
            plan.row.node_id,
            dir_fd=proof_slices.descriptor,
            follow_symlinks=False,
        )
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot inspect LS node root for {plan.row.node_id}: {error}"
        ) from error
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(
            f"LS node root cannot be a symlink: {plan.row.node_id}"
        )
    if not stat.S_ISDIR(metadata.st_mode):
        raise protocol.ValidationError(
            f"LS node root must be a directory: {plan.row.node_id}"
        )
    node_root = _pin_directory_at(
        proof_slices,
        plan.row.node_id,
        plan.node_root,
        f"LS node root for {plan.row.node_id}",
    )
    if created:
        try:
            _fsync_descriptor(
                proof_slices.descriptor, "LS proof-slices root after node creation"
            )
        except BaseException:
            try:
                os.close(node_root.descriptor)
            finally:
                _rmdir_if_present(proof_slices.descriptor, plan.row.node_id)
            raise
    return node_root, created


def _pin_absolute_directory(path: Path, label: str) -> _PinnedDirectory:
    absolute = _absolute_normalized(path)
    if not absolute.is_absolute():
        raise protocol.ValidationError(f"{label} must be absolute")
    _require_dirfd_primitives()
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    root_path = Path(absolute.anchor)
    try:
        descriptor = os.open(root_path, flags)
    except OSError as error:
        raise protocol.ValidationError(f"cannot pin {label}: {error}") from error
    owned: list[int] = []
    try:
        current_path = root_path
        for part in absolute.parts[1:]:
            parent_descriptor = descriptor
            descriptor = os.open(part, flags, dir_fd=parent_descriptor)
            owned.append(parent_descriptor)
            current_path /= part
        metadata = os.fstat(descriptor)
        pinned = _PinnedDirectory(
            path=absolute,
            descriptor=descriptor,
            device=metadata.st_dev,
            inode=metadata.st_ino,
            owned_ancestor_descriptors=tuple(owned),
        )
        _require_pinned_directory(pinned, label)
        return pinned
    except OSError as error:
        for owned_descriptor in reversed((*owned, descriptor)):
            try:
                os.close(owned_descriptor)
            except OSError:
                pass
        raise protocol.ValidationError(f"cannot pin {label}: {error}") from error
    except BaseException:
        for owned_descriptor in reversed((*owned, descriptor)):
            try:
                os.close(owned_descriptor)
            except OSError:
                pass
        raise


def _pin_directory(path: Path, label: str) -> _PinnedDirectory:
    return _pin_absolute_directory(path, label)


def _pin_directory_chain_at(
    parent: _PinnedDirectory,
    parts: Sequence[str],
    path: Path,
    label: str,
) -> _PinnedDirectory:
    _require_pinned_directory(parent, f"{label} ancestor")
    if not parts:
        raise protocol.ValidationError(f"{label} must be below its pinned ancestor")
    current = parent
    owned: list[int] = []
    current_path = parent.path
    try:
        for index, part in enumerate(parts):
            current_path /= part
            child = _pin_directory_at(
                current,
                part,
                current_path,
                label,
                require_parent_path=index == 0,
            )
            if current is not parent:
                owned.append(current.descriptor)
            current = child
        if current.path != path:
            raise protocol.ValidationError(f"{label} path does not match pinned chain")
        return _PinnedDirectory(
            path=current.path,
            descriptor=current.descriptor,
            device=current.device,
            inode=current.inode,
            parent_descriptor=current.parent_descriptor,
            entry_name=current.entry_name,
            owned_ancestor_descriptors=tuple(owned),
        )
    except BaseException:
        if current is not parent:
            try:
                os.close(current.descriptor)
            except OSError:
                pass
        for descriptor in reversed(owned):
            try:
                os.close(descriptor)
            except OSError:
                pass
        raise


def _create_unique_directory_at(
    parent: _PinnedDirectory, prefix: str, label: str
) -> _PinnedDirectory:
    _require_pinned_directory(parent, f"{label} parent")
    if not prefix.startswith(".") or "/" in prefix:
        raise protocol.ValidationError(f"{label} has an unsafe prefix")
    for _ in range(128):
        name = prefix + uuid.uuid4().hex
        try:
            os.mkdir(name, mode=0o700, dir_fd=parent.descriptor)
        except FileExistsError:
            continue
        except OSError as error:
            raise protocol.ValidationError(f"cannot create {label}: {error}") from error
        try:
            return _pin_directory_at(parent, name, parent.path / name, label)
        except BaseException:
            _rmdir_if_present(parent.descriptor, name)
            raise
    raise protocol.ValidationError(f"cannot allocate unique {label}")


def _create_stage_attempt(
    stage: _PinnedDirectory, plan: _AttemptPlan
) -> _PinnedDirectory:
    _validate_attempt_entry_name(plan.attempt_name, hidden=False)
    _require_pinned_directory(stage, "LS receipt staging directory")
    try:
        os.mkdir(plan.row.node_id, mode=0o700, dir_fd=stage.descriptor)
    except FileExistsError as error:
        raise protocol.ValidationError(
            f"staged LS node already exists: {plan.row.node_id}"
        ) from error
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot create staged LS node {plan.row.node_id}: {error}"
        ) from error
    return _pin_directory_at(
        stage,
        plan.row.node_id,
        stage.path / plan.row.node_id,
        f"staged LS attempt for {plan.row.node_id}",
    )


def _pin_stage_attempt(stage: _PinnedDirectory, plan: _AttemptPlan) -> _PinnedDirectory:
    return _pin_directory_at(
        stage,
        plan.row.node_id,
        stage.path / plan.row.node_id,
        f"staged LS attempt for {plan.row.node_id}",
    )


def _close_pinned_directory(directory: _PinnedDirectory) -> None:
    errors: list[OSError] = []
    for descriptor in (
        directory.descriptor,
        *reversed(directory.owned_ancestor_descriptors),
    ):
        try:
            os.close(descriptor)
        except OSError as error:
            errors.append(error)
    if errors:
        raise protocol.ValidationError(
            "cannot close pinned directory: "
            + "; ".join(str(error) for error in errors)
        )


def _pin_directory_at(
    parent: _PinnedDirectory,
    name: str,
    path: Path,
    label: str,
    *,
    require_parent_path: bool = True,
) -> _PinnedDirectory:
    _safe_entry_name(name, label)
    if require_parent_path:
        _require_pinned_directory(parent, f"{label} parent")
    else:
        _require_descriptor_identity(parent, f"{label} parent")
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    try:
        descriptor = os.open(name, flags, dir_fd=parent.descriptor)
    except FileNotFoundError:
        raise
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
            parent_descriptor=parent.descriptor,
            entry_name=name,
        )
        _require_pinned_directory(pinned, label)
        return pinned
    except BaseException:
        os.close(descriptor)
        raise


def _require_dirfd_primitives() -> None:
    required = (os.open, os.mkdir, os.stat, os.unlink, os.rmdir)
    if (
        not hasattr(os, "O_DIRECTORY")
        or not hasattr(os, "O_NOFOLLOW")
        or any(function not in os.supports_dir_fd for function in required)
        or os.stat not in os.supports_follow_symlinks
    ):
        raise protocol.ValidationError(
            "host runtime lacks descriptor-relative LS publication primitives"
        )


def _require_descriptor_identity(directory: _PinnedDirectory, label: str) -> None:
    try:
        metadata = os.fstat(directory.descriptor)
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot inspect pinned {label}: {error}"
        ) from error
    if not stat.S_ISDIR(metadata.st_mode) or (metadata.st_dev, metadata.st_ino) != (
        directory.device,
        directory.inode,
    ):
        raise protocol.ValidationError(f"pinned {label} identity changed")


def _require_pinned_directory(directory: _PinnedDirectory, label: str) -> None:
    _require_descriptor_identity(directory, label)
    try:
        if directory.parent_descriptor is None:
            metadata = directory.path.lstat()
        elif directory.entry_name is not None:
            metadata = os.stat(
                directory.entry_name,
                dir_fd=directory.parent_descriptor,
                follow_symlinks=False,
            )
        else:
            raise protocol.ValidationError(f"pinned {label} has no entry name")
    except OSError as error:
        raise protocol.ValidationError(
            f"pinned {label} identity changed: {error}"
        ) from error
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(f"pinned {label} changed to a symlink")
    if not stat.S_ISDIR(metadata.st_mode) or (metadata.st_dev, metadata.st_ino) != (
        directory.device,
        directory.inode,
    ):
        raise protocol.ValidationError(f"pinned {label} identity changed")


def _safe_entry_name(name: str, label: str) -> None:
    if not isinstance(name, str) or name in {"", ".", ".."} or "/" in name:
        raise protocol.ValidationError(f"{label} has an unsafe entry name")


def _validate_attempt_entry_name(name: str, *, hidden: bool) -> None:
    if hidden:
        match = re.fullmatch(
            r"\.(?P<attempt>attempt-[0-9]+)\.[0-9a-f]{32}\.staging", name
        )
        valid = (
            match is not None
            and ATTEMPT_NAME.fullmatch(match.group("attempt")) is not None
        )
    else:
        valid = ATTEMPT_NAME.fullmatch(name) is not None
    if not valid:
        raise protocol.ValidationError("unsafe LS attempt entry name")


def _unlink_if_present(directory_descriptor: int, name: str) -> None:
    try:
        os.unlink(name, dir_fd=directory_descriptor)
    except FileNotFoundError:
        return


def _rmdir_if_present(directory_descriptor: int, name: str) -> None:
    try:
        os.rmdir(name, dir_fd=directory_descriptor)
    except FileNotFoundError:
        return


def _rmdir_pinned_entry(parent: _PinnedDirectory, directory: _PinnedDirectory) -> None:
    if directory.parent_descriptor != parent.descriptor or directory.entry_name is None:
        raise protocol.ValidationError("pinned LS directory has invalid parent")
    try:
        metadata = os.stat(
            directory.entry_name,
            dir_fd=parent.descriptor,
            follow_symlinks=False,
        )
    except FileNotFoundError:
        return
    if (
        stat.S_ISLNK(metadata.st_mode)
        or not stat.S_ISDIR(metadata.st_mode)
        or (metadata.st_dev, metadata.st_ino) != (directory.device, directory.inode)
    ):
        raise protocol.ValidationError("LS directory identity changed before removal")
    os.rmdir(directory.entry_name, dir_fd=parent.descriptor)


def _read_safe_file_at(
    directory_descriptor: int, name: str, label: str, maximum: int
) -> bytes:
    _safe_entry_name(name, label)
    flags = os.O_RDONLY | os.O_NOFOLLOW
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    try:
        descriptor = os.open(name, flags, dir_fd=directory_descriptor)
    except OSError as error:
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
        current = os.stat(name, dir_fd=directory_descriptor, follow_symlinks=False)
        if after.st_nlink != 1 or current.st_nlink != 1:
            raise protocol.ValidationError(f"{label} must have exactly one hard link")
        if _metadata_tuple(before) != _metadata_tuple(after) or _metadata_tuple(
            after
        ) != _metadata_tuple(current):
            raise protocol.ValidationError(f"{label} changed while being read")
        return b"".join(chunks)
    except OSError as error:
        raise protocol.ValidationError(f"cannot read {label}: {error}") from error
    finally:
        os.close(descriptor)


def _write_bytes_create_only_at(
    directory_descriptor: int, name: str, data: bytes, label: str
) -> None:
    if "/" in name or name in {"", ".", ".."}:
        raise protocol.ValidationError(f"{label} has an unsafe name")
    if len(data) > MAX_MEMBER_BYTES:
        raise protocol.ValidationError(f"{label} exceeds byte cap")
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    if not hasattr(os, "O_NOFOLLOW"):
        raise protocol.ValidationError(
            "host runtime lacks no-follow descriptor-relative publication"
        )
    flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(name, flags, 0o600, dir_fd=directory_descriptor)
    except FileExistsError as error:
        raise protocol.ValidationError(f"{label} already exists") from error
    except OSError as error:
        raise protocol.ValidationError(f"cannot create {label}: {error}") from error
    try:
        view = memoryview(data)
        while view:
            written = os.write(descriptor, view)
            if written <= 0:
                raise OSError("short write")
            view = view[written:]
        metadata = os.fstat(descriptor)
        if metadata.st_nlink != 1:
            raise protocol.ValidationError(f"{label} must have exactly one hard link")
        _fsync_descriptor(descriptor, label)
    except OSError as error:
        raise protocol.ValidationError(f"cannot write {label}: {error}") from error
    finally:
        os.close(descriptor)


def _rename_directory_no_replace(
    node_root: _PinnedDirectory,
    source: _PinnedDirectory,
    destination_name: str,
    *,
    candidate: _PublicationCandidate | None = None,
) -> _PinnedDirectory:
    """Atomically publish ``source`` only when ``destination`` is absent."""

    if source.parent_descriptor != node_root.descriptor or source.entry_name is None:
        raise protocol.ValidationError("hidden LS attempt has invalid parent")
    source_name = source.entry_name
    _validate_attempt_entry_name(source_name, hidden=True)
    _validate_attempt_entry_name(destination_name, hidden=False)
    _require_pinned_directory(node_root, "LS node root")
    _require_pinned_directory(source, "hidden LS attempt")
    _fsync_descriptor(source.descriptor, "hidden LS attempt")
    _fsync_descriptor(node_root.descriptor, "LS node root before publication")
    system = platform.system()
    source_bytes = os.fsencode(source_name)
    destination_bytes = os.fsencode(destination_name)
    if b"\0" in source_bytes or b"\0" in destination_bytes:
        raise protocol.ValidationError(
            "LS publication paths must not contain NUL bytes"
        )
    library = ctypes.CDLL(None, use_errno=True)
    if system == "Darwin":
        try:
            rename = library.renameatx_np
        except AttributeError as error:
            raise protocol.ValidationError(
                "host Darwin runtime lacks atomic no-replace publication"
            ) from error
        rename.argtypes = (
            ctypes.c_int,
            ctypes.c_char_p,
            ctypes.c_int,
            ctypes.c_char_p,
            ctypes.c_uint,
        )
        rename.restype = ctypes.c_int
        result = rename(
            node_root.descriptor,
            source_bytes,
            node_root.descriptor,
            destination_bytes,
            DARWIN_RENAME_EXCL,
        )
    elif system == "Linux":
        try:
            rename = library.renameat2
        except AttributeError as error:
            raise protocol.ValidationError(
                "host Linux runtime lacks atomic no-replace publication"
            ) from error
        rename.argtypes = (
            ctypes.c_int,
            ctypes.c_char_p,
            ctypes.c_int,
            ctypes.c_char_p,
            ctypes.c_uint,
        )
        rename.restype = ctypes.c_int
        result = rename(
            node_root.descriptor,
            source_bytes,
            node_root.descriptor,
            destination_bytes,
            LINUX_RENAME_NOREPLACE,
        )
    else:
        raise protocol.ValidationError(
            f"atomic no-replace LS publication is unsupported on {system}"
        )
    if result == 0:
        published = _PinnedDirectory(
            path=node_root.path / destination_name,
            descriptor=source.descriptor,
            device=source.device,
            inode=source.inode,
            parent_descriptor=node_root.descriptor,
            entry_name=destination_name,
        )
        if candidate is not None:
            candidate.visible = True
            candidate.published_attempt = published
        return published
    error_number = ctypes.get_errno()
    if error_number in (errno.EEXIST, errno.ENOTEMPTY):
        raise protocol.ValidationError(
            "LS attempt appeared during publication and was not overwritten: "
            f"{node_root.path / destination_name}"
        )
    raise protocol.ValidationError(
        "cannot atomically publish LS attempt with no-replace rename: "
        + os.strerror(error_number)
    )


def _build_target_module(build_target: str) -> str:
    path = PurePosixPath(build_target)
    try:
        relative = path.relative_to("formalization/lean")
    except ValueError as error:
        raise protocol.ValidationError(
            "LS build_target must be beneath formalization/lean"
        ) from error
    if (
        path.is_absolute()
        or path.suffix != ".lean"
        or any(part in {"", ".", ".."} for part in path.parts)
    ):
        raise protocol.ValidationError(
            "LS build_target must be a normalized Lean source path"
        )
    module = ".".join(relative.with_suffix("").parts)
    if LEAN_NAME.fullmatch(module) is None:
        raise protocol.ValidationError("LS build_target has an invalid module name")
    return module


def _require_module_snapshot(
    repository_root: Path, plan: _AttemptPlan, expected: _FileSnapshot
) -> None:
    module_path = repository_root.joinpath(*PurePosixPath(plan.build_target).parts)
    observed = _read_file_snapshot(
        module_path,
        f"LS Lean module for {plan.row.node_id}",
        MAX_MEMBER_BYTES,
    )
    if observed != expected:
        raise protocol.ValidationError(
            f"LS Lean module changed during receipt publication: {plan.row.node_id}"
        )


def _require_module_snapshots(
    repository_root: Path,
    plans: tuple[_AttemptPlan, ...],
    expected: Mapping[str, _FileSnapshot],
) -> None:
    for plan in plans:
        _require_module_snapshot(repository_root, plan, expected[plan.row.node_id])


def _shared_environment() -> dict[str, str]:
    return {
        "ELAN_HOME": SHARED_ELAN_HOME.as_posix(),
        "ELAN_TOOLCHAIN": SHARED_LEAN_TOOLCHAIN,
        "PATH": os.pathsep.join(
            path.as_posix() for path in (SHARED_TOOLCHAIN_BIN, *SHARED_SYSTEM_PATH)
        ),
    }


def _subprocess_executor(
    argv: list[str],
    cwd: Path,
    environment: dict[str, str],
    timeout_seconds: int,
    max_output_bytes: int,
) -> CommandResult:
    stdout = _BoundedPipe(max_output_bytes)
    stderr = _BoundedPipe(max_output_bytes)
    try:
        process = subprocess.Popen(
            argv,
            cwd=cwd,
            env=environment,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
    except (FileNotFoundError, OSError) as error:
        data = str(error).encode("utf-8")
        return CommandResult(
            None,
            b"",
            data[:max_output_bytes],
            blocked_reason=str(error),
            stderr_truncated=len(data) > max_output_bytes,
        )
    stdout.start(process.stdout)
    stderr.start(process.stderr)
    stdout_drained = False
    stderr_drained = False
    try:
        try:
            exit_code = process.wait(timeout=timeout_seconds)
        except subprocess.TimeoutExpired:
            exit_code = None
            blocked_reason = f"command timed out after {timeout_seconds} seconds"
        else:
            blocked_reason = None
    finally:
        termination_error = _terminate_process_tree(process)
        if process.poll() is None:
            try:
                process.wait(timeout=PIPE_DRAIN_GRACE_SECONDS)
            except subprocess.TimeoutExpired:
                termination_error = (
                    termination_error
                    or "process group did not terminate after direct-child wait"
                )
        if termination_error is not None:
            exit_code = None
            blocked_reason = termination_error
    try:
        stdout_drained = stdout.join(PIPE_DRAIN_GRACE_SECONDS)
        stderr_drained = stderr.join(PIPE_DRAIN_GRACE_SECONDS)
        if not stdout_drained or not stderr_drained:
            stdout.join(PIPE_DRAIN_GRACE_SECONDS)
            stderr.join(PIPE_DRAIN_GRACE_SECONDS)
            exit_code = None
            blocked_reason = "command pipes did not drain after parent exit"
    finally:
        if process.stdout is not None:
            process.stdout.close()
        if process.stderr is not None:
            process.stderr.close()
    return CommandResult(
        exit_code,
        stdout.data,
        stderr.data,
        blocked_reason=blocked_reason,
        stdout_truncated=stdout.truncated or not stdout_drained,
        stderr_truncated=stderr.truncated or not stderr_drained,
    )


def _terminate_process_tree(process: subprocess.Popen[bytes]) -> str | None:
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        return None
    except (AttributeError, OSError) as error:
        try:
            process.kill()
        except ProcessLookupError:
            return None
        except OSError as fallback_error:
            return f"cannot terminate command process group: {fallback_error}"
        return f"cannot terminate command process group: {error}"
    return None


class _BoundedPipe:
    def __init__(self, maximum: int) -> None:
        self._maximum = maximum
        self._chunks: list[bytes] = []
        self._size = 0
        self.truncated = False
        self._thread: threading.Thread | None = None

    @property
    def data(self) -> bytes:
        return b"".join(self._chunks)

    def start(self, stream: Any) -> None:
        if stream is None:
            return
        self._thread = threading.Thread(target=self._read, args=(stream,), daemon=True)
        self._thread.start()

    def join(self, timeout: float) -> bool:
        if self._thread is not None:
            self._thread.join(timeout)
            return not self._thread.is_alive()
        return True

    def _read(self, stream: Any) -> None:
        with stream:
            while True:
                chunk = stream.read(8192)
                if not chunk:
                    return
                remaining = self._maximum - self._size
                if remaining > 0:
                    retained = chunk[:remaining]
                    self._chunks.append(retained)
                    self._size += len(retained)
                if len(chunk) > remaining:
                    self.truncated = True


def _canonical_json_bytes(value: Mapping[str, object]) -> bytes:
    return (
        json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
        + b"\n"
    )


def _write_bytes_create_only(path: Path, data: bytes, label: str) -> None:
    if len(data) > MAX_MEMBER_BYTES:
        raise protocol.ValidationError(f"{label} exceeds byte cap")
    try:
        descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    except FileExistsError as error:
        raise protocol.ValidationError(f"{label} already exists") from error
    except OSError as error:
        raise protocol.ValidationError(f"cannot create {label}: {error}") from error
    try:
        view = memoryview(data)
        while view:
            written = os.write(descriptor, view)
            if written <= 0:
                raise OSError("short write")
            view = view[written:]
    except OSError as error:
        raise protocol.ValidationError(f"cannot write {label}: {error}") from error
    finally:
        os.close(descriptor)


def _read_safe_file(path: Path, label: str, maximum: int) -> bytes:
    return _read_file_snapshot(path, label, maximum).data


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


def _read_file_snapshot_relative_at(
    root: _PinnedDirectory,
    parts: Sequence[str],
    label: str,
    maximum: int,
) -> _FileSnapshot:
    if len(parts) < 1:
        raise protocol.ValidationError(f"{label} path is empty")
    current = root
    opened: list[_PinnedDirectory] = []
    try:
        for part in parts[:-1]:
            child = _pin_directory_at(
                current, part, current.path / part, f"{label} directory"
            )
            opened.append(child)
            current = child
        return _read_file_snapshot_at(current.descriptor, parts[-1], label, maximum)
    finally:
        for directory in reversed(opened):
            _close_pinned_directory(directory)


def _read_file_snapshot_at(
    directory_descriptor: int, name: str, label: str, maximum: int
) -> _FileSnapshot:
    _safe_entry_name(name, label)
    flags = os.O_RDONLY | os.O_NOFOLLOW
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    try:
        descriptor = os.open(name, flags, dir_fd=directory_descriptor)
    except OSError as error:
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
        current = os.stat(name, dir_fd=directory_descriptor, follow_symlinks=False)
        if after.st_nlink != 1 or current.st_nlink != 1:
            raise protocol.ValidationError(f"{label} must have exactly one hard link")
        if _metadata_tuple(before) != _metadata_tuple(after) or _metadata_tuple(
            after
        ) != _metadata_tuple(current):
            raise protocol.ValidationError(f"{label} changed while being read")
        return _FileSnapshot(
            data=b"".join(chunks),
            device=after.st_dev,
            inode=after.st_ino,
            mode=after.st_mode,
            size=after.st_size,
            mtime_ns=after.st_mtime_ns,
            ctime_ns=after.st_ctime_ns,
            nlink=after.st_nlink,
        )
    finally:
        os.close(descriptor)


def _read_file_snapshot(path: Path, label: str, maximum: int) -> _FileSnapshot:
    _ensure_directory(path.parent, f"{label} parent")
    flags = os.O_RDONLY
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags)
    except OSError as error:
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
        if after.st_nlink != 1:
            raise protocol.ValidationError(f"{label} must have exactly one hard link")
        if _metadata_tuple(before) != _metadata_tuple(after):
            raise protocol.ValidationError(f"{label} changed while being read")
    except OSError as error:
        raise protocol.ValidationError(f"cannot read {label}: {error}") from error
    finally:
        os.close(descriptor)
    return _FileSnapshot(
        data=b"".join(chunks),
        device=after.st_dev,
        inode=after.st_ino,
        mode=after.st_mode,
        size=after.st_size,
        mtime_ns=after.st_mtime_ns,
        ctime_ns=after.st_ctime_ns,
        nlink=after.st_nlink,
    )


def _metadata_tuple(
    metadata: os.stat_result,
) -> tuple[int, int, int, int, int, int, int]:
    return (
        metadata.st_dev,
        metadata.st_ino,
        metadata.st_mode,
        metadata.st_size,
        metadata.st_mtime_ns,
        metadata.st_ctime_ns,
        metadata.st_nlink,
    )


def _fsync_descriptor(descriptor: int, label: str) -> None:
    try:
        os.fsync(descriptor)
    except OSError as error:
        raise protocol.ValidationError(f"cannot fsync {label}: {error}") from error


def _ensure_safe_file(path: Path, label: str, maximum: int) -> None:
    _read_safe_file(path, label, maximum)


def _ensure_directory(path: Path, label: str) -> None:
    current = Path(path.anchor)
    for part in path.parts[1:]:
        current /= part
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


def _absolute_normalized(path: Path) -> Path:
    return Path(os.path.abspath(os.fspath(path.expanduser())))


def _acquire_publication_lock(
    proof_slices: Path | _PinnedDirectory,
) -> _PublicationLock:
    proof_slices_path = (
        proof_slices.path
        if isinstance(proof_slices, _PinnedDirectory)
        else _absolute_normalized(proof_slices)
    )
    try:
        canonical = proof_slices_path.resolve(strict=True)
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot canonicalize LS proof-slices root: {error}"
        ) from error
    lock_name = hashlib.sha256(os.fsencode(canonical)).hexdigest() + ".lock"
    try:
        PUBLICATION_LOCK_ROOT.mkdir(mode=0o700, parents=True, exist_ok=True)
        root_metadata = PUBLICATION_LOCK_ROOT.lstat()
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot prepare LS publication lock root: {error}"
        ) from error
    if (
        stat.S_ISLNK(root_metadata.st_mode)
        or not stat.S_ISDIR(root_metadata.st_mode)
        or root_metadata.st_uid != os.geteuid()
    ):
        raise protocol.ValidationError(
            "LS publication lock root must be a caller-owned directory"
        )
    try:
        lock_root_path = PUBLICATION_LOCK_ROOT.resolve(strict=True)
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot resolve LS publication lock root: {error}"
        ) from error
    lock_root = _pin_absolute_directory(lock_root_path, "LS publication lock root")
    lock = lock_root.path / lock_name
    flags = os.O_RDWR | os.O_CREAT
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(lock_name, flags, 0o600, dir_fd=lock_root.descriptor)
    except OSError as error:
        _close_pinned_directory(lock_root)
        raise protocol.ValidationError(
            f"cannot acquire LS publication lock: {error}"
        ) from error
    try:
        metadata = os.fstat(descriptor)
        if (
            not stat.S_ISREG(metadata.st_mode)
            or metadata.st_uid != os.geteuid()
            or metadata.st_nlink != 1
        ):
            raise protocol.ValidationError(
                "LS publication lock must be a caller-owned regular file"
            )
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except OSError as error:
            if error.errno in (errno.EACCES, errno.EAGAIN):
                raise protocol.ValidationError(
                    "another LS receipt publisher is active"
                ) from error
            raise protocol.ValidationError(
                f"cannot acquire LS publication lock: {error}"
            ) from error
    except BaseException:
        os.close(descriptor)
        _close_pinned_directory(lock_root)
        raise
    return _PublicationLock(
        path=lock,
        descriptor=descriptor,
        root_descriptor=lock_root.descriptor,
        root_ancestor_descriptors=lock_root.owned_ancestor_descriptors,
    )


def _release_publication_lock(lock: _PublicationLock) -> None:
    errors: list[OSError] = []
    try:
        os.close(lock.descriptor)
    except OSError as error:
        errors.append(error)
    for descriptor in (lock.root_descriptor, *reversed(lock.root_ancestor_descriptors)):
        try:
            os.close(descriptor)
        except OSError as error:
            errors.append(error)
    if errors:
        raise protocol.ValidationError(
            "cannot release LS publication lock: "
            + "; ".join(str(error) for error in errors)
        )


def _require_descendant(path: Path, root: Path, label: str) -> None:
    try:
        path.relative_to(root)
    except ValueError as error:
        raise protocol.ValidationError(
            f"{label} must be beneath repository root"
        ) from error


def _remove_stage(
    stage: Path | _PinnedDirectory, target: Path | _PinnedDirectory | None
) -> None:
    if target is None:
        raise protocol.ValidationError("refusing to remove unbound LS stage")
    owned_stage = False
    owned_target = False
    if isinstance(target, _PinnedDirectory):
        target_directory = target
    else:
        target_directory = _pin_absolute_directory(
            _absolute_normalized(target), "LS stage parent"
        )
        owned_target = True
    try:
        if isinstance(stage, _PinnedDirectory):
            stage_directory = stage
        else:
            stage_path = _absolute_normalized(stage)
            if stage_path.parent != target_directory.path:
                raise protocol.ValidationError(
                    "refusing to remove unsafe LS stage path"
                )
            try:
                stage_directory = _pin_directory_at(
                    target_directory,
                    stage_path.name,
                    stage_path,
                    "LS receipt staging directory",
                )
            except FileNotFoundError:
                return
            owned_stage = True
        if (
            stage_directory.parent_descriptor != target_directory.descriptor
            or stage_directory.entry_name is None
            or not stage_directory.entry_name.startswith(STAGE_PREFIX)
        ):
            raise protocol.ValidationError("refusing to remove unsafe LS stage path")
        _remove_tree_contents_at(stage_directory)
        _rmdir_pinned_entry(target_directory, stage_directory)
    finally:
        if owned_stage and "stage_directory" in locals():
            _close_pinned_directory(stage_directory)
        if owned_target:
            _close_pinned_directory(target_directory)


def _remove_tree_contents_at(directory: _PinnedDirectory) -> None:
    _require_descriptor_identity(directory, "LS cleanup directory")
    try:
        names = os.listdir(directory.descriptor)
    except OSError as error:
        raise protocol.ValidationError(
            f"cannot inspect LS cleanup tree: {error}"
        ) from error
    if len(names) > MAX_CACHE_ENTRIES:
        raise protocol.ValidationError("LS cleanup tree exceeds entry cap")
    for name in names:
        _safe_entry_name(name, "LS cleanup entry")
        try:
            metadata = os.stat(name, dir_fd=directory.descriptor, follow_symlinks=False)
        except OSError as error:
            raise protocol.ValidationError(
                f"cannot inspect LS cleanup entry: {error}"
            ) from error
        if stat.S_ISDIR(metadata.st_mode) and not stat.S_ISLNK(metadata.st_mode):
            child = _pin_directory_at(
                directory,
                name,
                directory.path / name,
                "LS cleanup directory",
                require_parent_path=False,
            )
            try:
                _remove_tree_contents_at(child)
            finally:
                _close_pinned_directory(child)
            _rmdir_if_present(directory.descriptor, name)
        else:
            _unlink_if_present(directory.descriptor, name)
