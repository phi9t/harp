from __future__ import annotations

import json
import re
import stat
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
TIERS = frozenset({"tiny_smoke", "local_lake", "reference_suite", "target_route"})
SOURCE_ROLES = frozenset({"lean_module", "lake_config", "toolchain_lock", "adapter"})
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
    runtime_id: str
    platform: str
    lean: ToolIdentity
    lake: ToolIdentity | None
    command_profiles: dict[str, CommandProfile]


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
    suite_id: str
    runtime_id: str
    modules: tuple[SuiteModule, ...]
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
    _timestamp(value["created_at_utc"], "created_at_utc")
    return RuntimeLock(
        runtime_id=runtime_id,
        platform=platform,
        lean=lean,
        lake=lake,
        command_profiles=profiles,
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
    suite_id = _safe_id(value["suite_id"], "suite_id")
    runtime_id = _safe_id(value["runtime_id"], "runtime_id")
    _timestamp(value["created_at_utc"], "created_at_utc")
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
        suite_id=suite_id,
        runtime_id=runtime_id,
        modules=tuple(modules),
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
