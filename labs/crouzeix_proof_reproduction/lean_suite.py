from __future__ import annotations

import json
import os
import re
import shutil
import stat
import subprocess
import tempfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Mapping

import protocol


LOCK_FIELDS = frozenset(
    {
        "schema_version",
        "runtime_id",
        "platform",
        "lean",
        "lake",
        "packages",
        "command_profiles",
        "allowed_env",
        "created_at_utc",
    }
)
TOOL_FIELDS = frozenset({"path", "version", "bytes", "sha256"})
COMMAND_PROFILE_FIELDS = frozenset({"profile_id", "argv", "timeout_seconds", "env"})
MANIFEST_FIELDS = frozenset(
    {"schema_version", "suite_id", "runtime_id", "modules", "created_at_utc"}
)
MANIFEST_OPTIONAL_FIELDS = frozenset({"source_files"})
MODULE_FIELDS = frozenset(
    {
        "module_id",
        "tier",
        "path",
        "module_name",
        "command_profile",
        "expected_declarations",
        "allowed_axioms",
        "source_sha256",
        "source_bytes",
    }
)
SOURCE_FILE_FIELDS = frozenset({"path", "role", "sha256", "bytes"})
RECEIPT_FIELDS = frozenset(
    {
        "schema_version",
        "run_id",
        "runtime_id",
        "suite_id",
        "runtime_lock_sha256",
        "suite_manifest_sha256",
        "source_inventory_sha256",
        "runtime_inventory_sha256",
        "artifact_inventory_sha256",
        "execution_root_sha256",
        "command_receipts",
        "module_outcomes",
        "outcome",
        "reason",
        "lean_suite_receipt_sha256",
    }
)
COMMAND_RECEIPT_FIELDS = frozenset(
    {
        "module_id",
        "argv",
        "cwd",
        "env",
        "timeout_seconds",
        "exit_code",
        "stdout_path",
        "stdout_sha256",
        "stderr_path",
        "stderr_sha256",
    }
)
MODULE_OUTCOME_FIELDS = frozenset({"module_id", "outcome", "reason"})
TIERS = frozenset({"tiny_smoke", "local_lake", "reference_suite", "target_route"})
SOURCE_ROLES = frozenset({"lean_module", "lake_config", "toolchain_lock", "adapter"})
OUTCOMES = frozenset({"passed", "failed", "blocked"})
SAFE_ID = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
SOURCE_ROLE = re.compile(r"^[a-z][a-z0-9]*(?:_[a-z0-9]+)*$")
ENV_NAME = re.compile(r"^[A-Z_][A-Z0-9_]*$")
LEAN_MODULE = re.compile(r"^[A-Z][A-Za-z0-9_]*(?:\.[A-Z][A-Za-z0-9_]*)*$")
MAX_JSON_BYTES = 1024 * 1024
DISALLOWED_ENV = frozenset(
    {
        "PATH",
        "HOME",
        "PWD",
        "SHELL",
        "LEAN_PATH",
        "LAKE_HOME",
        "ELAN_HOME",
        "XDG_CACHE_HOME",
    }
)
DISALLOWED_ENV_PREFIXES = ("DYLD_", "LD_", "PYTHON")


@dataclass(frozen=True)
class ToolIdentity:
    path: str
    version: str
    bytes: int
    sha256: str
    absolute_path: Path


@dataclass(frozen=True)
class CommandProfile:
    profile_id: str
    argv: tuple[str, ...]
    timeout_seconds: int
    env: dict[str, str]


@dataclass(frozen=True)
class RuntimeLock:
    schema_version: str
    runtime_id: str
    platform: str
    lean: ToolIdentity
    lake: ToolIdentity | None
    packages: tuple[object, ...]
    command_profiles: dict[str, CommandProfile]
    allowed_env: tuple[str, ...]
    created_at_utc: str


@dataclass(frozen=True)
class SuiteModule:
    module_id: str
    tier: str
    path: str
    module_name: str
    command_profile: str
    expected_declarations: tuple[str, ...]
    allowed_axioms: tuple[str, ...]
    source_sha256: str
    source_bytes: int


@dataclass(frozen=True)
class SourceFile:
    path: str
    role: str
    sha256: str
    bytes: int


@dataclass(frozen=True)
class SuiteManifest:
    schema_version: str
    suite_id: str
    runtime_id: str
    modules: tuple[SuiteModule, ...]
    created_at_utc: str
    source_files: tuple[SourceFile, ...] = ()


def loads_json_object(data: bytes, label: str) -> dict[str, object]:
    if len(data) > MAX_JSON_BYTES:
        raise protocol.ValidationError(f"{label} exceeds size limit")

    def reject_duplicates(pairs: list[tuple[str, object]]) -> dict[str, object]:
        result: dict[str, object] = {}
        for key, value in pairs:
            if key in result:
                raise protocol.ValidationError(f"{label} has duplicate key {key}")
            result[key] = value
        return result

    try:
        value = json.loads(data.decode("utf-8"), object_pairs_hook=reject_duplicates)
    except UnicodeDecodeError as exc:
        raise protocol.ValidationError(f"{label} must be UTF-8 JSON") from exc
    except json.JSONDecodeError as exc:
        raise protocol.ValidationError(f"{label} must be valid JSON") from exc
    if not isinstance(value, dict):
        raise protocol.ValidationError(f"{label} must be an object")
    return value


def validate_runtime_lock(value: Mapping[str, Any], root: Path) -> RuntimeLock:
    _require_fields(value, LOCK_FIELDS, "runtime lock")
    _require_equal(
        value["schema_version"], "crouzeix-lean-runtime-lock/v1", "schema_version"
    )
    schema_version = str(value["schema_version"])
    runtime_id = _safe_id(value["runtime_id"], "runtime_id")
    platform = _bounded_string(value["platform"], "platform", 1, 128)
    lean = _validate_tool(value["lean"], root, "lean")
    lake_value = value["lake"]
    lake = None if lake_value is None else _validate_tool(lake_value, root, "lake")
    allowed_env = _validate_allowed_env(value["allowed_env"])
    profiles = _validate_command_profiles(
        value["command_profiles"],
        allowed_env,
        lake_available=lake is not None,
    )
    packages = value["packages"]
    if not isinstance(packages, list):
        raise protocol.ValidationError("packages must be a list")
    if packages:
        raise protocol.ValidationError("packages must be empty until package locks exist")
    created_at_utc = _timestamp(value["created_at_utc"], "created_at_utc")
    return RuntimeLock(
        schema_version=schema_version,
        runtime_id=runtime_id,
        platform=platform,
        lean=lean,
        lake=lake,
        packages=tuple(packages),
        command_profiles=profiles,
        allowed_env=tuple(sorted(allowed_env)),
        created_at_utc=created_at_utc,
    )


def validate_suite_manifest(
    value: Mapping[str, Any], command_profiles: set[str]
) -> SuiteManifest:
    _require_fields(
        value,
        MANIFEST_FIELDS,
        "suite manifest",
        optional=MANIFEST_OPTIONAL_FIELDS,
    )
    _require_equal(
        value["schema_version"], "crouzeix-lean-suite-manifest/v1", "schema_version"
    )
    schema_version = str(value["schema_version"])
    suite_id = _safe_id(value["suite_id"], "suite_id")
    runtime_id = _safe_id(value["runtime_id"], "runtime_id")
    created_at_utc = _timestamp(value["created_at_utc"], "created_at_utc")
    if not all(isinstance(profile, str) for profile in command_profiles):
        raise protocol.ValidationError("command_profiles must contain strings")

    raw_modules = value["modules"]
    if not isinstance(raw_modules, list) or not raw_modules:
        raise protocol.ValidationError("modules must be a non-empty list")
    modules: list[SuiteModule] = []
    seen_ids: set[str] = set()
    seen_paths: set[str] = set()
    for raw in raw_modules:
        module = _validate_module(raw, command_profiles)
        if module.module_id in seen_ids:
            raise protocol.ValidationError(f"duplicate module_id {module.module_id}")
        if module.path in seen_paths:
            raise protocol.ValidationError(f"duplicate module path {module.path}")
        seen_ids.add(module.module_id)
        seen_paths.add(module.path)
        modules.append(module)
    source_files = tuple(
        _validate_source_files(value.get("source_files", []), seen_paths)
    )
    return SuiteManifest(
        schema_version=schema_version,
        suite_id=suite_id,
        runtime_id=runtime_id,
        modules=tuple(modules),
        created_at_utc=created_at_utc,
        source_files=source_files,
    )


def canonical_sha256(value: Mapping[str, Any]) -> str:
    data = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return protocol.sha256_bytes(data)


def compute_source_inventory(suite_root: Path, manifest: SuiteManifest) -> dict[str, object]:
    root = suite_root.absolute()
    _reject_symlink_ancestors(root, "suite root")
    if root.is_symlink():
        raise protocol.ValidationError("suite root contains symlink component")
    if not root.is_dir():
        raise protocol.ValidationError("suite root must be a directory")

    expected: dict[str, SourceFile] = {}
    for module in manifest.modules:
        expected[module.path] = SourceFile(
            path=module.path,
            role="lean_module",
            sha256=module.source_sha256,
            bytes=module.source_bytes,
        )
    for source_file in manifest.source_files:
        if source_file.path in expected:
            raise protocol.ValidationError(f"duplicate source path {source_file.path}")
        expected[source_file.path] = source_file

    observed: dict[str, dict[str, object]] = {}
    for path in sorted(
        root.rglob("*"), key=lambda candidate: candidate.relative_to(root).as_posix()
    ):
        relative = path.relative_to(root).as_posix()
        if path.is_symlink():
            raise protocol.ValidationError(f"source contains symlink: {relative}")
        if path.is_dir():
            continue
        if not path.is_file():
            raise protocol.ValidationError(f"source must be a regular file: {relative}")
        if relative not in expected:
            raise protocol.ValidationError(f"unknown source file: {relative}")
        declared = expected[relative]
        data = path.read_bytes()
        if len(data) != declared.bytes:
            raise protocol.ValidationError(f"source_bytes mismatch: {relative}")
        sha256 = protocol.sha256_bytes(data)
        if sha256 != declared.sha256:
            raise protocol.ValidationError(f"source_sha256 mismatch: {relative}")
        observed[relative] = {
            "path": relative,
            "role": declared.role,
            "bytes": len(data),
            "sha256": sha256,
        }

    missing = sorted(set(expected) - set(observed))
    if missing:
        raise protocol.ValidationError(f"missing source file: {missing[0]}")
    return {
        "schema_version": "crouzeix-lean-source-inventory/v1",
        "suite_id": manifest.suite_id,
        "file_count": len(observed),
        "files": [observed[key] for key in sorted(observed)],
    }


def _materialize_source_inventory(
    source_root: Path, execution_root: Path, source_inventory: Mapping[str, object]
) -> None:
    files = source_inventory["files"]
    if not isinstance(files, list):
        raise protocol.ValidationError("source inventory files must be a list")
    for raw in files:
        entry = _mapping(raw, "source inventory file")
        relative = _safe_relative_path(entry["path"], "source inventory path")
        source = source_root / relative
        target = execution_root / relative
        _reject_symlink_ancestors(source, "source")
        _reject_symlink_ancestors(target, "execution root")
        target.parent.mkdir(parents=True, exist_ok=True)
        if source.is_symlink() or not source.is_file():
            raise protocol.ValidationError(f"source must be a regular file: {relative}")
        if target.exists():
            raise protocol.ValidationError(f"duplicate materialized source: {relative}")
        data = source.read_bytes()
        if len(data) != _integer(entry["bytes"], "source inventory bytes", 0, 1 << 30):
            raise protocol.ValidationError(f"source_bytes mismatch: {relative}")
        if protocol.sha256_bytes(data) != _digest(
            entry["sha256"], "source inventory sha256"
        ):
            raise protocol.ValidationError(f"source_sha256 mismatch: {relative}")
        _write_bytes_create_only(target, data)


def _compute_execution_root_inventory(
    execution_root: Path, manifest: SuiteManifest
) -> dict[str, object]:
    declared: dict[str, str] = {
        module.path: "lean_module" for module in manifest.modules
    }
    declared.update({source.path: source.role for source in manifest.source_files})
    files: list[dict[str, object]] = []
    root = execution_root.absolute()
    for path in sorted(
        root.rglob("*"), key=lambda candidate: candidate.relative_to(root).as_posix()
    ):
        relative = path.relative_to(root).as_posix()
        if path.is_dir() and not path.is_symlink():
            continue
        if path.is_symlink():
            files.append(
                {
                    "path": relative,
                    "role": declared.get(relative, "undeclared"),
                    "kind": "symlink",
                }
            )
            continue
        if not path.is_file():
            files.append(
                {
                    "path": relative,
                    "role": declared.get(relative, "undeclared"),
                    "kind": "non_regular",
                }
            )
            continue
        data = path.read_bytes()
        files.append(
            {
                "path": relative,
                "role": declared.get(relative, "undeclared"),
                "kind": "file",
                "bytes": len(data),
                "sha256": protocol.sha256_bytes(data),
            }
        )
    return {
        "schema_version": "crouzeix-lean-execution-root-inventory/v1",
        "suite_id": manifest.suite_id,
        "file_count": len(files),
        "files": files,
    }


def _compute_runtime_inventory(runtime_lock: RuntimeLock) -> dict[str, object]:
    tools = [_runtime_tool_inventory(runtime_lock.lean, "lean")]
    if runtime_lock.lake is not None:
        tools.append(_runtime_tool_inventory(runtime_lock.lake, "lake"))
    return {
        "schema_version": "crouzeix-lean-runtime-inventory/v1",
        "runtime_id": runtime_lock.runtime_id,
        "tool_count": len(tools),
        "tools": sorted(tools, key=lambda tool: str(tool["role"])),
    }


def _runtime_tool_inventory(tool: ToolIdentity, role: str) -> dict[str, object]:
    path = tool.absolute_path
    entry: dict[str, object] = {
        "role": role,
        "path": tool.path,
        "expected_bytes": tool.bytes,
        "expected_sha256": tool.sha256,
        "version": tool.version,
    }
    try:
        _reject_symlink_ancestors(path, f"{role} tool")
    except protocol.ValidationError:
        entry["status"] = "symlinked"
        return entry
    if path.is_symlink():
        entry["status"] = "symlinked"
        return entry
    if not path.exists():
        entry["status"] = "missing"
        return entry
    if not path.is_file():
        entry["status"] = "non_regular"
        return entry
    data = path.read_bytes()
    sha256 = protocol.sha256_bytes(data)
    if len(data) == tool.bytes and sha256 == tool.sha256:
        status = "matched"
    else:
        status = "drifted"
    entry.update(
        {
            "status": status,
            "bytes": len(data),
            "sha256": sha256,
        }
    )
    return entry


def _runtime_inventory_block_reason(runtime_inventory: Mapping[str, object]) -> str | None:
    tools = runtime_inventory.get("tools")
    if not isinstance(tools, list):
        return "runtime inventory is malformed before command execution"
    for raw_tool in tools:
        tool = _mapping(raw_tool, "runtime inventory tool")
        role = _bounded_string(tool.get("role"), "runtime inventory tool role", 1, 64)
        status = _bounded_string(
            tool.get("status"), "runtime inventory tool status", 1, 64
        )
        if status != "matched":
            return f"runtime inventory blocked {role} tool with status {status}"
    return None


def _compute_artifact_inventory(runtime_lock: RuntimeLock) -> dict[str, object]:
    return {
        "schema_version": "crouzeix-lean-artifact-inventory/v1",
        "runtime_id": runtime_lock.runtime_id,
        "artifact_count": 0,
        "artifacts": [],
    }


def _post_execution_inventory_failure(
    *,
    execution_root: Path,
    manifest: SuiteManifest,
    source_inventory: Mapping[str, object],
    runtime_lock: RuntimeLock,
    runtime_inventory: Mapping[str, object],
) -> str | None:
    try:
        after_execution = compute_source_inventory(execution_root, manifest)
        if canonical_sha256(after_execution) != canonical_sha256(source_inventory):
            return "execution root inventory changed after command execution"
        after_runtime = _compute_runtime_inventory(runtime_lock)
        if canonical_sha256(after_runtime) != canonical_sha256(runtime_inventory):
            return "runtime inventory changed after command execution"
    except protocol.ValidationError as exc:
        return f"execution root inventory validation failed: {exc}"
    return None


def _apply_inventory_failure(
    module_outcomes: list[dict[str, object]], reason: str
) -> None:
    if module_outcomes:
        module_outcomes[0] = {
            "module_id": module_outcomes[0]["module_id"],
            "outcome": "failed",
            "reason": reason,
        }


def run_suite(
    runtime_lock: RuntimeLock,
    manifest: SuiteManifest,
    suite_root: Path,
    receipt_root: Path,
    run_id: str,
    *,
    allowed_write_roots: tuple[Path, ...] | None = None,
) -> Path:
    safe_run_id = _safe_id(run_id, "run_id")
    root = suite_root.absolute()
    _reject_symlink_ancestors(root, "suite root")
    _reject_local_path_escape(receipt_root, "receipt root")
    receipt_base = receipt_root.absolute()
    _reject_symlink_ancestors(receipt_base, "receipt root")
    destination = receipt_base / safe_run_id
    _reject_symlink_ancestors(destination, "receipt destination")
    if destination.exists():
        raise protocol.ValidationError(
            f"receipt destination already exists: {destination}"
        )
    destination.parent.mkdir(parents=True, exist_ok=True)
    source_inventory = compute_source_inventory(root, manifest)
    destination.mkdir(mode=0o700)
    try:
        with tempfile.TemporaryDirectory(
            prefix="execution-", dir=destination
        ) as execution_directory:
            execution_root = Path(execution_directory)
            _materialize_source_inventory(root, execution_root, source_inventory)
            pre_execution_inventory = compute_source_inventory(execution_root, manifest)
            if canonical_sha256(pre_execution_inventory) != canonical_sha256(
                source_inventory
            ):
                raise protocol.ValidationError(
                    "execution root inventory does not match source inventory"
                )
            runtime_inventory = _compute_runtime_inventory(runtime_lock)
            artifact_inventory = _compute_artifact_inventory(runtime_lock)
            runtime_block_reason = _runtime_inventory_block_reason(runtime_inventory)
            write_roots = _effective_allowed_write_roots(
                execution_root=execution_root,
                receipt_root=destination,
                extra_roots=allowed_write_roots,
            )
            command_receipts: list[dict[str, object]] = []
            module_outcomes: list[dict[str, object]] = []
            for module in manifest.modules:
                profile = runtime_lock.command_profiles[module.command_profile]
                command, outcome = _run_module(
                    runtime_lock=runtime_lock,
                    profile=profile,
                    suite_root=execution_root,
                    receipt_root=destination,
                    module=module,
                    runtime_block_reason=runtime_block_reason,
                    allowed_write_roots=write_roots,
                )
                command_receipts.append(command)
                module_outcomes.append(outcome)
            final_execution_inventory = _compute_execution_root_inventory(
                execution_root, manifest
            )
            inventory_failure = _post_execution_inventory_failure(
                execution_root=execution_root,
                manifest=manifest,
                source_inventory=source_inventory,
                runtime_lock=runtime_lock,
                runtime_inventory=runtime_inventory,
            )
            if inventory_failure is not None:
                _apply_inventory_failure(module_outcomes, inventory_failure)
        final = _final_outcome(module_outcomes)
        receipt: dict[str, object] = {
            "schema_version": "crouzeix-lean-suite-receipt/v1",
            "run_id": safe_run_id,
            "runtime_id": runtime_lock.runtime_id,
            "suite_id": manifest.suite_id,
            "runtime_lock_sha256": canonical_sha256(_runtime_lock_wire(runtime_lock)),
            "suite_manifest_sha256": canonical_sha256(_suite_manifest_wire(manifest)),
            "source_inventory_sha256": canonical_sha256(source_inventory),
            "runtime_inventory_sha256": canonical_sha256(runtime_inventory),
            "artifact_inventory_sha256": canonical_sha256(artifact_inventory),
            "execution_root_sha256": canonical_sha256(final_execution_inventory),
            "command_receipts": command_receipts,
            "module_outcomes": module_outcomes,
            "outcome": final["outcome"],
            "reason": final["reason"],
            "lean_suite_receipt_sha256": "",
        }
        receipt["lean_suite_receipt_sha256"] = canonical_sha256_without_receipt_self(
            receipt
        )
        _write_json_create_only(destination / "receipt.json", receipt)
    except BaseException:
        _remove_new_tree(destination)
        raise
    return destination / "receipt.json"


def validate_receipt_json(path: Path) -> dict[str, object]:
    receipt_path = path.absolute()
    _reject_symlink_ancestors(receipt_path, "receipt")
    if not receipt_path.is_file():
        raise protocol.ValidationError("receipt must be a regular file")
    receipt = loads_json_object(receipt_path.read_bytes(), "lean suite receipt")
    _require_fields(receipt, RECEIPT_FIELDS, "lean suite receipt")
    result: dict[str, object] = dict(receipt)
    _require_equal(
        result["schema_version"],
        "crouzeix-lean-suite-receipt/v1",
        "schema_version",
    )
    result["run_id"] = _safe_id(result["run_id"], "run_id")
    result["runtime_id"] = _safe_id(result["runtime_id"], "runtime_id")
    result["suite_id"] = _safe_id(result["suite_id"], "suite_id")
    result["runtime_lock_sha256"] = _digest(
        result["runtime_lock_sha256"], "runtime_lock_sha256"
    )
    result["suite_manifest_sha256"] = _digest(
        result["suite_manifest_sha256"], "suite_manifest_sha256"
    )
    result["source_inventory_sha256"] = _digest(
        result["source_inventory_sha256"], "source_inventory_sha256"
    )
    result["runtime_inventory_sha256"] = _digest(
        result["runtime_inventory_sha256"], "runtime_inventory_sha256"
    )
    result["artifact_inventory_sha256"] = _digest(
        result["artifact_inventory_sha256"], "artifact_inventory_sha256"
    )
    result["execution_root_sha256"] = _digest(
        result["execution_root_sha256"], "execution_root_sha256"
    )
    result["outcome"] = _enum(result["outcome"], OUTCOMES, "outcome")
    result["reason"] = _bounded_string(result["reason"], "reason", 1, 4096)
    result["lean_suite_receipt_sha256"] = _digest(
        result["lean_suite_receipt_sha256"], "lean_suite_receipt_sha256"
    )
    _validate_receipt_lists(result, receipt_path.parent)
    if result["lean_suite_receipt_sha256"] != canonical_sha256_without_receipt_self(
        result
    ):
        raise protocol.ValidationError("lean_suite_receipt_sha256 mismatch")
    final = _final_outcome(result["module_outcomes"])  # type: ignore[arg-type]
    if result["outcome"] != final["outcome"] or result["reason"] != final["reason"]:
        raise protocol.ValidationError("receipt outcome does not match module outcomes")
    return result


def _validate_tool(value: Any, root: Path, label: str) -> ToolIdentity:
    mapping = _mapping(value, label)
    _require_fields(mapping, TOOL_FIELDS, label)
    path = _safe_relative_path(mapping["path"], f"{label}.path")
    absolute = root / path
    _reject_symlink_ancestors(absolute, f"{label}.path")
    if not absolute.is_file():
        raise protocol.ValidationError(f"{label}.path must be a regular file")
    mode = absolute.stat().st_mode
    if not (mode & stat.S_IXUSR):
        raise protocol.ValidationError(f"{label}.path must be executable")
    expected_bytes = _integer(mapping["bytes"], f"{label}.bytes", 1, 1 << 40)
    data = absolute.read_bytes()
    if len(data) != expected_bytes:
        raise protocol.ValidationError(f"{label}.bytes mismatch")
    expected_sha = _digest(mapping["sha256"], f"{label}.sha256")
    if protocol.sha256_bytes(data) != expected_sha:
        raise protocol.ValidationError(f"{label}.sha256 mismatch")
    return ToolIdentity(
        path=path,
        version=_bounded_string(mapping["version"], f"{label}.version", 1, 256),
        bytes=expected_bytes,
        sha256=expected_sha,
        absolute_path=absolute,
    )


def _run_module(
    *,
    runtime_lock: RuntimeLock,
    profile: CommandProfile,
    suite_root: Path,
    receipt_root: Path,
    module: SuiteModule,
    runtime_block_reason: str | None,
    allowed_write_roots: tuple[Path, ...],
) -> tuple[dict[str, object], dict[str, object]]:
    module_dir = receipt_root / "modules" / module.module_id
    module_dir.mkdir(parents=True, mode=0o700)
    stdout_path = module_dir / "stdout.txt"
    stderr_path = module_dir / "stderr.txt"
    argv = [
        _expand_arg(arg, runtime_lock=runtime_lock, module=module, suite_root=suite_root)
        for arg in profile.argv
    ]
    env = dict(sorted(profile.env.items()))
    exit_code: int | None = None
    stdout = b""
    stderr = b""
    outcome = "blocked"
    reason = "locked tool unavailable before command execution"
    policy_block_reason = _command_write_policy_block_reason(
        argv=argv,
        env=env,
        allowed_write_roots=allowed_write_roots,
    )
    if runtime_block_reason is not None:
        reason = runtime_block_reason
        stderr = reason.encode("utf-8")
    elif policy_block_reason is not None:
        reason = policy_block_reason
        stderr = reason.encode("utf-8")
    elif not Path(argv[0]).is_file():
        stderr = reason.encode("utf-8")
    else:
        try:
            completed = subprocess.run(
                argv,
                cwd=suite_root,
                env=env,
                text=False,
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                timeout=profile.timeout_seconds,
                check=False,
            )
            exit_code = completed.returncode
            stdout = completed.stdout
            stderr = completed.stderr
            if completed.returncode == 0:
                outcome = "passed"
                reason = "module compiled under locked command"
            else:
                outcome = "failed"
                reason = "lean command returned nonzero exit"
        except FileNotFoundError:
            stderr = reason.encode("utf-8")
        except PermissionError:
            reason = "locked tool unavailable due to permission error"
            stderr = reason.encode("utf-8")
        except subprocess.TimeoutExpired as exc:
            outcome = "blocked"
            reason = "lean command timed out before judging proof correctness"
            stdout = exc.stdout or b""
            stderr = exc.stderr or reason.encode("utf-8")

    _write_bytes_create_only(stdout_path, stdout)
    _write_bytes_create_only(stderr_path, stderr)
    command_receipt = {
        "module_id": module.module_id,
        "argv": argv,
        "cwd": suite_root.as_posix(),
        "env": env,
        "timeout_seconds": profile.timeout_seconds,
        "exit_code": exit_code,
        "stdout_path": stdout_path.relative_to(receipt_root).as_posix(),
        "stdout_sha256": protocol.sha256_bytes(stdout),
        "stderr_path": stderr_path.relative_to(receipt_root).as_posix(),
        "stderr_sha256": protocol.sha256_bytes(stderr),
    }
    module_outcome = {
        "module_id": module.module_id,
        "outcome": outcome,
        "reason": reason,
    }
    return command_receipt, module_outcome


def _effective_allowed_write_roots(
    *,
    execution_root: Path,
    receipt_root: Path,
    extra_roots: tuple[Path, ...] | None,
) -> tuple[Path, ...]:
    roots = [execution_root.absolute(), receipt_root.absolute()]
    for root in extra_roots or ():
        absolute = root.absolute()
        _reject_symlink_ancestors(absolute, "allowed write root")
        roots.append(absolute)
    return tuple(roots)


def _command_write_policy_block_reason(
    *,
    argv: list[str],
    env: Mapping[str, str],
    allowed_write_roots: tuple[Path, ...],
) -> str | None:
    for index, arg in enumerate(argv[1:], start=1):
        path = _absolute_path_value(arg)
        if path is not None and not _is_under_allowed_root(path, allowed_write_roots):
            return (
                "local_process_no_os_sandbox: write policy blocked absolute "
                f"argv[{index}] outside allowed roots"
            )
    for key, value in sorted(env.items()):
        path = _absolute_path_value(value)
        if path is not None and not _is_under_allowed_root(path, allowed_write_roots):
            return (
                "local_process_no_os_sandbox: write policy blocked "
                f"env.{key} absolute path outside allowed roots"
            )
    return None


def _absolute_path_value(value: str) -> Path | None:
    if "\0" in value:
        return None
    path = Path(value)
    if not path.is_absolute():
        return None
    return path.absolute()


def _is_under_allowed_root(path: Path, allowed_write_roots: tuple[Path, ...]) -> bool:
    for root in allowed_write_roots:
        try:
            path.relative_to(root)
            return True
        except ValueError:
            continue
    return False


def _expand_arg(
    arg: str, *, runtime_lock: RuntimeLock, module: SuiteModule, suite_root: Path
) -> str:
    if arg == "{lean}":
        return runtime_lock.lean.absolute_path.as_posix()
    if arg == "{lake}":
        if runtime_lock.lake is None:
            raise protocol.ValidationError(
                "command profile references lake but lake is null"
            )
        return runtime_lock.lake.absolute_path.as_posix()
    if arg == "{module_path}":
        return (suite_root / module.path).as_posix()
    if arg == "{module_name}":
        return module.module_name
    if "{" in arg or "}" in arg:
        raise protocol.ValidationError("unknown command template variable")
    return arg


def _final_outcome(module_outcomes: list[dict[str, object]]) -> dict[str, str]:
    for outcome in module_outcomes:
        if outcome["outcome"] == "blocked":
            return {"outcome": "blocked", "reason": str(outcome["reason"])}
    for outcome in module_outcomes:
        if outcome["outcome"] == "failed":
            return {"outcome": "failed", "reason": str(outcome["reason"])}
    return {"outcome": "passed", "reason": "all modules compiled under locked commands"}


def canonical_sha256_without_receipt_self(value: Mapping[str, Any]) -> str:
    payload = dict(value)
    payload.pop("lean_suite_receipt_sha256", None)
    return canonical_sha256(payload)


def _validate_receipt_lists(receipt: dict[str, object], receipt_root: Path) -> None:
    commands = receipt["command_receipts"]
    outcomes = receipt["module_outcomes"]
    if not isinstance(commands, list) or not commands:
        raise protocol.ValidationError("command_receipts must be a non-empty list")
    if not isinstance(outcomes, list) or not outcomes:
        raise protocol.ValidationError("module_outcomes must be a non-empty list")
    for command in commands:
        mapping = _mapping(command, "command receipt")
        _require_fields(mapping, COMMAND_RECEIPT_FIELDS, "command receipt")
        _safe_id(mapping["module_id"], "module_id")
        _string_list(mapping["argv"], "argv", minimum=1, maximum=64)
        _bounded_string(mapping["cwd"], "cwd", 1, 4096)
        env = _mapping(mapping["env"], "env")
        for key, value in env.items():
            if not isinstance(key, str) or ENV_NAME.fullmatch(key) is None:
                raise protocol.ValidationError("env key is invalid")
            _bounded_string(value, f"env.{key}", 0, 4096)
        exit_code = mapping["exit_code"]
        if exit_code is not None:
            _integer(exit_code, "exit_code", -255, 255)
        stdout_path = _safe_relative_path(mapping["stdout_path"], "stdout_path")
        stdout_sha256 = _digest(mapping["stdout_sha256"], "stdout_sha256")
        stderr_path = _safe_relative_path(mapping["stderr_path"], "stderr_path")
        stderr_sha256 = _digest(mapping["stderr_sha256"], "stderr_sha256")
        _validate_output_digest(receipt_root, stdout_path, stdout_sha256, "stdout")
        _validate_output_digest(receipt_root, stderr_path, stderr_sha256, "stderr")
    for outcome in outcomes:
        mapping = _mapping(outcome, "module outcome")
        _require_fields(mapping, MODULE_OUTCOME_FIELDS, "module outcome")
        _safe_id(mapping["module_id"], "module_id")
        _enum(mapping["outcome"], OUTCOMES, "module outcome")
        _bounded_string(mapping["reason"], "reason", 1, 4096)


def _validate_output_digest(
    receipt_root: Path, relative_path: str, expected_sha256: str, label: str
) -> None:
    path = receipt_root / relative_path
    _reject_symlink_ancestors(path, label)
    if not path.is_file():
        raise protocol.ValidationError(f"{label} output must be a regular file")
    data = path.read_bytes()
    if protocol.sha256_bytes(data) != expected_sha256:
        raise protocol.ValidationError(f"{label} output digest mismatch")


def _write_json_create_only(path: Path, value: Mapping[str, object]) -> None:
    data = (json.dumps(value, indent=2, sort_keys=True) + "\n").encode("utf-8")
    _write_bytes_create_only(path, data)


def _write_bytes_create_only(path: Path, data: bytes) -> None:
    _reject_symlink_ancestors(path, path.name)
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    fd = os.open(path, flags, 0o600)
    with os.fdopen(fd, "wb") as handle:
        handle.write(data)


def _remove_new_tree(path: Path) -> None:
    if path.exists() and not path.is_symlink():
        shutil.rmtree(path)


def _runtime_lock_wire(lock: RuntimeLock) -> dict[str, object]:
    return {
        "schema_version": lock.schema_version,
        "runtime_id": lock.runtime_id,
        "platform": lock.platform,
        "lean": _tool_wire(lock.lean),
        "lake": None if lock.lake is None else _tool_wire(lock.lake),
        "packages": list(lock.packages),
        "command_profiles": {
            key: {
                "profile_id": value.profile_id,
                "argv": list(value.argv),
                "timeout_seconds": value.timeout_seconds,
                "env": dict(sorted(value.env.items())),
            }
            for key, value in sorted(lock.command_profiles.items())
        },
        "allowed_env": list(lock.allowed_env),
        "created_at_utc": lock.created_at_utc,
    }


def _tool_wire(tool: ToolIdentity) -> dict[str, object]:
    return {
        "path": tool.path,
        "version": tool.version,
        "bytes": tool.bytes,
        "sha256": tool.sha256,
    }


def _suite_manifest_wire(manifest: SuiteManifest) -> dict[str, object]:
    return {
        "schema_version": manifest.schema_version,
        "suite_id": manifest.suite_id,
        "runtime_id": manifest.runtime_id,
        "modules": [
            {
                "module_id": module.module_id,
                "tier": module.tier,
                "path": module.path,
                "module_name": module.module_name,
                "command_profile": module.command_profile,
                "expected_declarations": list(module.expected_declarations),
                "allowed_axioms": list(module.allowed_axioms),
                "source_sha256": module.source_sha256,
                "source_bytes": module.source_bytes,
            }
            for module in manifest.modules
        ],
        "created_at_utc": manifest.created_at_utc,
        "source_files": [
            {
                "path": source.path,
                "role": source.role,
                "sha256": source.sha256,
                "bytes": source.bytes,
            }
            for source in manifest.source_files
        ],
    }


def _validate_allowed_env(value: Any) -> frozenset[str]:
    names = _string_list(value, "allowed_env", maximum=64)
    if len(set(names)) != len(names):
        raise protocol.ValidationError("allowed_env must be unique")
    for name in names:
        if (
            ENV_NAME.fullmatch(name) is None
            or name in DISALLOWED_ENV
            or name.startswith(DISALLOWED_ENV_PREFIXES)
        ):
            raise protocol.ValidationError("allowed_env contains invalid name")
    return frozenset(names)


def _validate_command_profiles(
    value: Any, allowed_env: frozenset[str], *, lake_available: bool
) -> dict[str, CommandProfile]:
    if not isinstance(value, list) or not value:
        raise protocol.ValidationError("command_profiles must be a non-empty list")
    profiles: dict[str, CommandProfile] = {}
    for raw in value:
        mapping = _mapping(raw, "command profile")
        _require_fields(mapping, COMMAND_PROFILE_FIELDS, "command profile")
        profile_id = _safe_id(mapping["profile_id"], "profile_id")
        if profile_id in profiles:
            raise protocol.ValidationError(f"duplicate profile_id {profile_id}")
        argv = tuple(_string_list(mapping["argv"], "argv", maximum=32, minimum=1))
        if argv[0] not in {"{lean}", "{lake}"}:
            raise protocol.ValidationError("argv must start from a locked tool token")
        if argv[0] == "{lake}" and not lake_available:
            raise protocol.ValidationError("argv references lake without locked lake tool")
        env = _validate_profile_env(mapping["env"], allowed_env)
        profiles[profile_id] = CommandProfile(
            profile_id=profile_id,
            argv=argv,
            timeout_seconds=_integer(mapping["timeout_seconds"], "timeout_seconds", 1, 3600),
            env=env,
        )
    return profiles


def _validate_profile_env(value: Any, allowed_env: frozenset[str]) -> dict[str, str]:
    if not isinstance(value, dict):
        raise protocol.ValidationError("env must be an object")
    result: dict[str, str] = {}
    for key, raw_value in value.items():
        if not isinstance(key, str) or key not in allowed_env:
            raise protocol.ValidationError("env uses name outside allowed_env")
        result[key] = _bounded_string(raw_value, f"env.{key}", 0, 4096)
    return result


def _validate_module(value: Any, command_profiles: set[str]) -> SuiteModule:
    mapping = _mapping(value, "module")
    _require_fields(mapping, MODULE_FIELDS, "module")
    module_id = _safe_id(mapping["module_id"], "module_id")
    tier = _enum(mapping["tier"], TIERS, "tier")
    path = _safe_relative_path(mapping["path"], "module path")
    if not path.endswith(".lean"):
        raise protocol.ValidationError("module path must end with .lean")
    module_name = _bounded_string(mapping["module_name"], "module_name", 1, 256)
    if LEAN_MODULE.fullmatch(module_name) is None:
        raise protocol.ValidationError("module_name is invalid")
    command_profile = _safe_id(mapping["command_profile"], "command_profile")
    if command_profile not in command_profiles:
        raise protocol.ValidationError("command_profile is not declared")
    return SuiteModule(
        module_id=module_id,
        tier=tier,
        path=path,
        module_name=module_name,
        command_profile=command_profile,
        expected_declarations=tuple(
            _string_list(
                mapping["expected_declarations"],
                "expected_declarations",
                maximum=64,
            )
        ),
        allowed_axioms=tuple(
            _string_list(mapping["allowed_axioms"], "allowed_axioms", maximum=64)
        ),
        source_sha256=_digest(mapping["source_sha256"], "source_sha256"),
        source_bytes=_integer(mapping["source_bytes"], "source_bytes", 0, 1 << 30),
    )


def _validate_source_files(value: Any, module_paths: set[str]) -> list[SourceFile]:
    if not isinstance(value, list):
        raise protocol.ValidationError("source_files must be a list")
    if len(value) > 128:
        raise protocol.ValidationError("source_files must have at most 128 entries")
    source_files: list[SourceFile] = []
    seen_paths = set(module_paths)
    for raw in value:
        mapping = _mapping(raw, "source file")
        _require_fields(mapping, SOURCE_FILE_FIELDS, "source file")
        path = _safe_relative_path(mapping["path"], "source file path")
        if path in seen_paths:
            raise protocol.ValidationError(f"duplicate source path {path}")
        seen_paths.add(path)
        role = _source_role(mapping["role"], "source role")
        source_files.append(
            SourceFile(
                path=path,
                role=role,
                sha256=_digest(mapping["sha256"], "source file sha256"),
                bytes=_integer(mapping["bytes"], "source file bytes", 0, 1 << 30),
            )
        )
    return source_files


def _require_fields(
    value: Mapping[str, Any],
    fields: frozenset[str],
    label: str,
    *,
    optional: frozenset[str] = frozenset(),
) -> None:
    actual = set(value)
    if not fields <= actual or not actual <= fields | optional:
        raise protocol.ValidationError(
            f"{label} fields mismatch: expected {sorted(fields)}, got {sorted(actual)}"
        )


def _require_equal(value: Any, expected: str, label: str) -> None:
    if value != expected:
        raise protocol.ValidationError(f"{label} must be {expected}")


def _mapping(value: Any, label: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise protocol.ValidationError(f"{label} must be an object")
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


def _safe_id(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 128)
    if SAFE_ID.fullmatch(text) is None:
        raise protocol.ValidationError(f"{label} is invalid")
    return text


def _safe_relative_path(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 512)
    raw_parts = text.split("/")
    pure = PurePosixPath(text)
    if (
        pure.is_absolute()
        or "\\" in text
        or any(part in {"", ".", ".."} for part in raw_parts)
        or pure.as_posix() != text
    ):
        raise protocol.ValidationError(f"{label} must be a safe relative path")
    return text


def _reject_local_path_escape(path: Path, label: str) -> None:
    if any(part in {"..", ""} for part in path.parts):
        raise protocol.ValidationError(f"{label} must not contain path traversal")


def _reject_symlink_ancestors(path: Path, label: str) -> None:
    current = Path(path.anchor) if path.is_absolute() else Path(".")
    parts = path.parts[1:] if path.is_absolute() else path.parts
    for part in parts:
        current = current / part
        if current.exists() and current.is_symlink():
            raise protocol.ValidationError(f"{label} contains symlink component")


def _digest(value: Any, label: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(char not in "0123456789abcdef" for char in value)
    ):
        raise protocol.ValidationError(f"{label} must be a SHA-256 hex digest")
    return value


def _integer(value: Any, label: str, minimum: int, maximum: int) -> int:
    if (
        not isinstance(value, int)
        or isinstance(value, bool)
        or value < minimum
        or value > maximum
    ):
        raise protocol.ValidationError(f"{label} is out of range")
    return value


def _enum(value: Any, allowed: frozenset[str], label: str) -> str:
    text = _bounded_string(value, label, 1, 128)
    if text not in allowed:
        raise protocol.ValidationError(f"{label} is invalid")
    return text


def _source_role(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 128)
    if SOURCE_ROLE.fullmatch(text) is None or text not in SOURCE_ROLES:
        raise protocol.ValidationError(f"{label} is invalid")
    return text


def _string_list(
    value: Any, label: str, *, maximum: int, minimum: int = 0
) -> list[str]:
    if (
        not isinstance(value, list)
        or len(value) < minimum
        or len(value) > maximum
    ):
        raise protocol.ValidationError(
            f"{label} must be a list with {minimum}..{maximum} entries"
        )
    return [
        _bounded_string(item, f"{label} item", 0 if minimum == 0 else 1, 4096)
        for item in value
    ]


def _timestamp(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 20, 40)
    if not text.endswith("Z") or "T" not in text:
        raise protocol.ValidationError(f"{label} must be an UTC timestamp")
    return text
