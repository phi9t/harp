"""Strict, dependency-free records for hermetic Lean/TRAE verifier artifacts.

This module deliberately contains no runner, compiler, or policy behavior.  It
defines the byte and JSON boundaries shared by those later layers.
"""

from __future__ import annotations

import hashlib
import json
import re
import stat
from dataclasses import dataclass
from pathlib import Path, PureWindowsPath
from typing import Any, Mapping, Sequence
from types import MappingProxyType


_SHA256_RE = re.compile(r"[0-9a-f]{64}\Z")
_IDENTIFIER_RE = re.compile(r"[A-Za-z][A-Za-z0-9_.-]{0,127}\Z")
_LEAN_IDENTIFIER_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_'.]*(?:\.[A-Za-z_][A-Za-z0-9_'.]*)*\Z")
_URL_RE = re.compile(r"https://[^\s]+\Z")
_RUNTIME_SCHEMA = "harp-lean-runtime-lock/v1"
_TASK_SCHEMA = "harp-lean-task-seal/v1"
_LEDGER_SCHEMA = "harp-lean-ledger-event/v1"
_CANDIDATE_SCHEMA = "harp-lean-candidate/v1"
_COMPILER_SCHEMA = "harp-lean-compiler-report/v1"
_RESULT_SCHEMA = "harp-lean-run-result/v1"
_RECEIPT_SCHEMA = "harp-lean-receipt/v1"


class ValidationError(ValueError):
    """Raised when untrusted artifacts do not meet the sealed data contract."""


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValidationError(f"duplicate key {key!r}")
        result[key] = value
    return result


def _reject_json_constant(value: str) -> None:
    raise ValidationError(f"non-finite JSON number {value!r} is not allowed")


def _validate_json_tree(value: Any, label: str) -> None:
    if value is None or isinstance(value, (bool, str, int)):
        return
    if isinstance(value, float):
        raise ValidationError(f"{label} must not contain floating-point values")
    if isinstance(value, (list, tuple)):
        for index, item in enumerate(value):
            _validate_json_tree(item, f"{label}[{index}]")
        return
    if isinstance(value, Mapping):
        for key, item in value.items():
            if not isinstance(key, str):
                raise ValidationError(f"{label} contains a non-string object key")
            _validate_json_tree(item, f"{label}.{key}")
        return
    raise ValidationError(f"{label} contains a non-JSON value")


def _freeze_json(value: Any) -> Any:
    """Make a recursively immutable copy of already-validated JSON data."""
    if value is None or isinstance(value, (bool, str, int)):
        return value
    if isinstance(value, list):
        return tuple(_freeze_json(item) for item in value)
    if isinstance(value, dict):
        return MappingProxyType({key: _freeze_json(item) for key, item in value.items()})
    raise ValidationError("cannot freeze a non-JSON value")


def _thaw_json(value: Any) -> Any:
    """Convert frozen JSON data back to ordinary JSON containers for encoding."""
    if value is None or isinstance(value, (bool, str, int)):
        return value
    if isinstance(value, tuple):
        return [_thaw_json(item) for item in value]
    if isinstance(value, Mapping):
        return {key: _thaw_json(item) for key, item in value.items()}
    raise ValidationError("cannot encode a non-JSON value")


def parse_json_object(raw: bytes | str, label: str) -> dict[str, Any]:
    """Parse one UTF-8 JSON object without accepting duplicate object keys."""
    try:
        if isinstance(raw, bytes):
            raw = raw.decode("utf-8", "strict")
        elif not isinstance(raw, str):
            raise ValidationError(f"{label} must be UTF-8 JSON bytes or text")
        value = json.loads(
            raw,
            object_pairs_hook=_reject_duplicate_keys,
            parse_constant=_reject_json_constant,
        )
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValidationError(f"cannot parse {label} as strict JSON: {error}") from error
    if not isinstance(value, dict):
        raise ValidationError(f"{label} must be a JSON object")
    _validate_json_tree(value, label)
    return value


def canonical_json_bytes(value: Any) -> bytes:
    """Return canonical JSON bytes used for stable digests and receipts."""
    _validate_json_tree(value, "canonical JSON")
    try:
        return json.dumps(
            _thaw_json(value),
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
    except (TypeError, ValueError) as error:
        raise ValidationError(f"cannot encode canonical JSON: {error}") from error


def require_exact_fields(value: Mapping[str, Any], fields: Sequence[str], label: str) -> None:
    """Require a closed record with exactly the declared keys."""
    if set(value) != set(fields):
        missing = sorted(set(fields) - set(value))
        unknown = sorted(set(value) - set(fields))
        detail = []
        if missing:
            detail.append(f"missing={missing!r}")
        if unknown:
            detail.append(f"unknown={unknown!r}")
        suffix = f" ({', '.join(detail)})" if detail else ""
        raise ValidationError(f"{label} fields are not exact{suffix}")


def validate_relative_artifact_path(value: object, label: str) -> str:
    """Validate portable, slash-separated path syntax before any filesystem use."""
    if not isinstance(value, str) or not value:
        raise ValidationError(f"{label} must be a non-empty relative path")
    if "\x00" in value or "\\" in value or value.startswith("/") or PureWindowsPath(value).is_absolute():
        raise ValidationError(f"{label} must be a real relative artifact path")
    parts = value.split("/")
    if any(part in ("", ".", "..") for part in parts):
        raise ValidationError(f"{label} must be a real relative artifact path")
    return value


def _expect_object(value: object, label: str) -> Mapping[str, Any]:
    if not isinstance(value, dict):
        raise ValidationError(f"{label} must be an object")
    return value


def _expect_text(value: object, label: str, *, pattern: re.Pattern[str] | None = None) -> str:
    if not isinstance(value, str) or not value or "\x00" in value:
        raise ValidationError(f"{label} must be non-empty text")
    if pattern is not None and pattern.fullmatch(value) is None:
        raise ValidationError(f"{label} has an invalid value")
    return value


def _expect_nonnegative_int(value: object, label: str) -> int:
    if isinstance(value, bool) or not isinstance(value, int) or value < 0:
        raise ValidationError(f"{label} must be a non-negative integer")
    return value


def _expect_digest(value: object, label: str) -> "Digest":
    return Digest.from_value(value, label)


@dataclass(frozen=True)
class Digest:
    algorithm: str
    hex: str

    @classmethod
    def from_value(cls, value: object, label: str = "digest") -> "Digest":
        source = _expect_object(value, label)
        require_exact_fields(source, ("algorithm", "hex"), label)
        if source["algorithm"] != "sha256":
            raise ValidationError(f"{label} algorithm must be sha256")
        hex_value = _expect_text(source["hex"], f"{label} hex")
        if _SHA256_RE.fullmatch(hex_value) is None:
            raise ValidationError(f"{label} hex must be 64 lower-case SHA-256 characters")
        return cls(algorithm="sha256", hex=hex_value)

    def to_value(self) -> dict[str, str]:
        return {"algorithm": self.algorithm, "hex": self.hex}


def sha256_bytes(data: bytes) -> Digest:
    if not isinstance(data, bytes):
        raise TypeError("sha256_bytes requires bytes")
    return Digest("sha256", hashlib.sha256(data).hexdigest())


def _require_real_directory(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISDIR(metadata.st_mode):
        raise ValidationError(f"{label} must be a real non-symlink directory")


def _safe_artifact_target(root: Path, relative_path: str, label: str) -> Path:
    _require_real_directory(root, "artifact root")
    current = root
    for part in relative_path.split("/"):
        current = current / part
        try:
            metadata = current.lstat()
        except OSError as error:
            raise ValidationError(f"cannot inspect {label}: {error}") from error
        if stat.S_ISLNK(metadata.st_mode):
            raise ValidationError(f"{label} must not traverse a symlink")
    return current


@dataclass(frozen=True)
class ArtifactRef:
    path: str
    bytes: int
    digest: Digest

    @classmethod
    def from_value(cls, value: object, label: str = "artifact") -> "ArtifactRef":
        source = _expect_object(value, label)
        require_exact_fields(source, ("path", "bytes", "digest"), label)
        return cls(
            path=validate_relative_artifact_path(source["path"], f"{label} path"),
            bytes=_expect_nonnegative_int(source["bytes"], f"{label} bytes"),
            digest=_expect_digest(source["digest"], f"{label} digest"),
        )

    @classmethod
    def from_file(cls, root: Path, path: str) -> "ArtifactRef":
        relative_path = validate_relative_artifact_path(path, "artifact path")
        target = _safe_artifact_target(root, relative_path, "artifact")
        metadata = target.lstat()
        if not stat.S_ISREG(metadata.st_mode):
            raise ValidationError("artifact must be a regular non-symlink file")
        data = target.read_bytes()
        return cls(path=relative_path, bytes=len(data), digest=sha256_bytes(data))

    def verify(self, root: Path) -> Path:
        target = _safe_artifact_target(root, self.path, "artifact")
        metadata = target.lstat()
        if not stat.S_ISREG(metadata.st_mode):
            raise ValidationError("artifact must be a regular non-symlink file")
        if metadata.st_size != self.bytes:
            raise ValidationError("artifact byte size does not match its reference")
        if sha256_bytes(target.read_bytes()) != self.digest:
            raise ValidationError("artifact digest does not match its reference")
        return target

    def to_value(self) -> dict[str, object]:
        return {"path": self.path, "bytes": self.bytes, "digest": self.digest.to_value()}


@dataclass(frozen=True)
class RuntimeArtifact:
    source_url: str
    digest: Digest
    bytes: int

    @classmethod
    def from_value(cls, value: object, label: str) -> "RuntimeArtifact":
        source = _expect_object(value, label)
        require_exact_fields(source, ("source_url", "digest", "bytes"), label)
        source_url = _expect_text(source["source_url"], f"{label} source_url", pattern=_URL_RE)
        return cls(source_url, _expect_digest(source["digest"], f"{label} digest"), _expect_nonnegative_int(source["bytes"], f"{label} bytes"))


@dataclass(frozen=True)
class TraeRuntime:
    version: str
    artifact: RuntimeArtifact
    published_path: str
    version_stdout: str

    @classmethod
    def from_value(cls, value: object, label: str) -> "TraeRuntime":
        source = _expect_object(value, label)
        require_exact_fields(source, ("version", "artifact", "published_path", "version_stdout"), label)
        return cls(
            _expect_text(source["version"], f"{label} version", pattern=re.compile(r"\d+\.\d+\.\d+\Z")),
            RuntimeArtifact.from_value(source["artifact"], f"{label} artifact"),
            validate_relative_artifact_path(source["published_path"], f"{label} published_path"),
            _expect_text(source["version_stdout"], f"{label} version_stdout"),
        )


@dataclass(frozen=True)
class LeanRuntime:
    version: str
    toolchain_id: str
    artifact_digest: Digest
    elan_layout_digest: Digest

    @classmethod
    def from_value(cls, value: object, label: str) -> "LeanRuntime":
        source = _expect_object(value, label)
        require_exact_fields(source, ("version", "toolchain_id", "artifact_digest", "elan_layout_digest"), label)
        return cls(
            _expect_text(source["version"], f"{label} version", pattern=re.compile(r"\d+\.\d+\.\d+\Z")),
            _expect_text(source["toolchain_id"], f"{label} toolchain_id"),
            _expect_digest(source["artifact_digest"], f"{label} artifact_digest"),
            _expect_digest(source["elan_layout_digest"], f"{label} elan_layout_digest"),
        )


@dataclass(frozen=True)
class MathlibLock:
    repository: str
    revision: str
    project_tree_digest: Digest
    lake_manifest_digest: Digest
    package_cache_digest: Digest

    @classmethod
    def from_value(cls, value: object, label: str) -> "MathlibLock":
        source = _expect_object(value, label)
        require_exact_fields(source, ("repository", "revision", "project_tree_digest", "lake_manifest_digest", "package_cache_digest"), label)
        return cls(
            _expect_text(source["repository"], f"{label} repository", pattern=_URL_RE),
            _expect_text(source["revision"], f"{label} revision", pattern=re.compile(r"[0-9a-f]{40}\Z")),
            _expect_digest(source["project_tree_digest"], f"{label} project_tree_digest"),
            _expect_digest(source["lake_manifest_digest"], f"{label} lake_manifest_digest"),
            _expect_digest(source["package_cache_digest"], f"{label} package_cache_digest"),
        )


@dataclass(frozen=True)
class RuntimeLock:
    runtime_id: str
    os: str
    arch: str
    trae: TraeRuntime
    lean: LeanRuntime
    mathlib: MathlibLock | None

    @classmethod
    def from_value(cls, value: object, label: str = "runtime lock") -> "RuntimeLock":
        source = _expect_object(value, label)
        require_exact_fields(source, ("schema_version", "runtime_id", "platform", "trae", "lean", "mathlib"), label)
        if source["schema_version"] != _RUNTIME_SCHEMA:
            raise ValidationError(f"{label} schema version is not recognized")
        platform = _expect_object(source["platform"], f"{label} platform")
        require_exact_fields(platform, ("os", "arch"), f"{label} platform")
        os_name = _expect_text(platform["os"], f"{label} platform os")
        arch = _expect_text(platform["arch"], f"{label} platform arch")
        if os_name not in {"darwin", "linux"}:
            raise ValidationError(f"{label} platform os is not supported")
        if arch not in {"arm64", "x86_64"}:
            raise ValidationError(f"{label} platform arch is not supported")
        mathlib_value = source["mathlib"]
        if mathlib_value is not None and not isinstance(mathlib_value, dict):
            raise ValidationError(f"{label} mathlib must be an object or null")
        return cls(
            _expect_text(source["runtime_id"], f"{label} runtime_id", pattern=_IDENTIFIER_RE),
            os_name,
            arch,
            TraeRuntime.from_value(source["trae"], f"{label} trae"),
            LeanRuntime.from_value(source["lean"], f"{label} lean"),
            None if mathlib_value is None else MathlibLock.from_value(mathlib_value, f"{label} mathlib"),
        )


@dataclass(frozen=True)
class SlotSeal:
    slot_id: str
    source_path: str
    declaration_name: str
    declaration_type: str
    prefix: ArtifactRef
    suffix: ArtifactRef
    mode: str
    forbidden_tokens: tuple[str, ...]

    @classmethod
    def from_value(cls, value: object, label: str) -> "SlotSeal":
        source = _expect_object(value, label)
        require_exact_fields(source, ("slot_id", "source_path", "declaration_name", "declaration_type", "prefix", "suffix", "mode", "forbidden_tokens"), label)
        tokens = source["forbidden_tokens"]
        if not isinstance(tokens, list) or any(not isinstance(item, str) or not item for item in tokens):
            raise ValidationError(f"{label} forbidden_tokens must be non-empty text entries")
        if len(tokens) != len(set(tokens)):
            raise ValidationError(f"{label} forbidden_tokens must be unique")
        if source["mode"] != "proof_body":
            raise ValidationError(f"{label} mode is not recognized")
        return cls(
            _expect_text(source["slot_id"], f"{label} slot_id", pattern=_IDENTIFIER_RE),
            validate_relative_artifact_path(source["source_path"], f"{label} source_path"),
            _expect_text(source["declaration_name"], f"{label} declaration_name", pattern=_LEAN_IDENTIFIER_RE),
            _expect_text(source["declaration_type"], f"{label} declaration_type"),
            ArtifactRef.from_value(source["prefix"], f"{label} prefix"),
            ArtifactRef.from_value(source["suffix"], f"{label} suffix"),
            "proof_body",
            tuple(tokens),
        )


@dataclass(frozen=True)
class CommandRule:
    argv: tuple[str, ...]
    cwd: str

    @classmethod
    def from_value(cls, value: object, label: str) -> "CommandRule":
        source = _expect_object(value, label)
        require_exact_fields(source, ("argv", "cwd"), label)
        argv = source["argv"]
        if not isinstance(argv, list) or not argv or any(not isinstance(item, str) or not item or "\x00" in item for item in argv):
            raise ValidationError(f"{label} argv must be a non-empty text array")
        cwd = source["cwd"]
        if cwd == ".":
            return cls(tuple(argv), cwd)
        return cls(tuple(argv), validate_relative_artifact_path(cwd, f"{label} cwd"))


@dataclass(frozen=True)
class CommandPolicy:
    commands: tuple[CommandRule, ...]
    writable_paths: tuple[str, ...]
    network: bool

    @classmethod
    def from_value(cls, value: object, label: str) -> "CommandPolicy":
        source = _expect_object(value, label)
        require_exact_fields(source, ("commands", "writable_paths", "network"), label)
        commands = source["commands"]
        paths = source["writable_paths"]
        if not isinstance(commands, list) or not commands:
            raise ValidationError(f"{label} commands must be a non-empty array")
        if not isinstance(paths, list) or any(not isinstance(item, str) for item in paths):
            raise ValidationError(f"{label} writable_paths must be an array of paths")
        if not isinstance(source["network"], bool):
            raise ValidationError(f"{label} network must be boolean")
        validated_paths = tuple(validate_relative_artifact_path(item, f"{label} writable_paths") for item in paths)
        if len(validated_paths) != len(set(validated_paths)):
            raise ValidationError(f"{label} writable_paths must be unique")
        return cls(tuple(CommandRule.from_value(item, f"{label} commands[{index}]") for index, item in enumerate(commands)), validated_paths, source["network"])


@dataclass(frozen=True)
class TaskSeal:
    task_id: str
    profile: str
    runtime_id: str
    generation_budget: int
    repair_budget: int
    candidate_encoding: str
    slots: tuple[SlotSeal, ...]
    slot_order: tuple[str, ...] | None
    project_inventory: tuple[ArtifactRef, ...]
    command_policy: CommandPolicy
    prompt: ArtifactRef

    @classmethod
    def from_value(cls, value: object, label: str = "task seal") -> "TaskSeal":
        source = _expect_object(value, label)
        base_fields = ("schema_version", "task_id", "profile", "runtime_id", "generation_budget", "repair_budget", "candidate_encoding", "slots", "project_inventory", "command_policy", "prompt")
        allowed = set(base_fields) | {"slot_order"}
        if set(source) not in (set(base_fields), allowed):
            raise ValidationError(f"{label} fields are not exact")
        if source["schema_version"] != _TASK_SCHEMA:
            raise ValidationError(f"{label} schema version is not recognized")
        profile = _expect_text(source["profile"], f"{label} profile")
        expected_argv_by_profile = {
            "core-proposition": ("lean", "Candidate.lean"),
            "core-structural": ("lean", "Candidate.lean"),
            "mathlib-add-comm": ("lake", "env", "lean", "Candidate.lean"),
        }
        if profile not in expected_argv_by_profile:
            raise ValidationError(f"{label} profile is not recognized")
        if source["candidate_encoding"] != "utf-8-proof-body":
            raise ValidationError(f"{label} candidate_encoding is not recognized")
        slots_value = source["slots"]
        inventory_value = source["project_inventory"]
        if not isinstance(slots_value, list) or not slots_value:
            raise ValidationError(f"{label} slots must be a non-empty array")
        if not isinstance(inventory_value, list):
            raise ValidationError(f"{label} project_inventory must be an array")
        slots = tuple(SlotSeal.from_value(item, f"{label} slots[{index}]") for index, item in enumerate(slots_value))
        if len({slot.slot_id for slot in slots}) != len(slots):
            raise ValidationError(f"{label} slot ids must be unique")
        slot_order_value = source.get("slot_order")
        if slot_order_value is None:
            slot_order = None
        else:
            if not isinstance(slot_order_value, list) or any(not isinstance(item, str) for item in slot_order_value):
                raise ValidationError(f"{label} slot_order must be an array of slot ids")
            slot_order = tuple(slot_order_value)
            if tuple(slot.slot_id for slot in slots) != slot_order or len(set(slot_order)) != len(slot_order):
                raise ValidationError(f"{label} slot_order must exactly match declared slot order")
        inventory = tuple(ArtifactRef.from_value(item, f"{label} project_inventory[{index}]") for index, item in enumerate(inventory_value))
        if len({item.path for item in inventory}) != len(inventory):
            raise ValidationError(f"{label} project_inventory paths must be unique")
        generation_budget = _expect_nonnegative_int(source["generation_budget"], f"{label} generation_budget")
        if generation_budget != 1:
            raise ValidationError(f"{label} generation_budget must be exactly 1")
        repair_budget = _expect_nonnegative_int(source["repair_budget"], f"{label} repair_budget")
        if repair_budget > 2:
            raise ValidationError(f"{label} repair_budget must be between 0 and 2")
        command_policy = CommandPolicy.from_value(source["command_policy"], f"{label} command_policy")
        expected_argv = expected_argv_by_profile[profile]
        expected_writable_paths = ("Scratch.lean", "Candidate.lean")
        if command_policy.network:
            raise ValidationError(f"{label} command_policy network must be false")
        if (
            len(command_policy.commands) != 1
            or command_policy.commands[0].argv != expected_argv
            or command_policy.commands[0].cwd != "."
            or command_policy.writable_paths != expected_writable_paths
        ):
            raise ValidationError(f"{label} command_policy does not match the profile policy")
        return cls(
            _expect_text(source["task_id"], f"{label} task_id", pattern=_IDENTIFIER_RE),
            profile,
            _expect_text(source["runtime_id"], f"{label} runtime_id", pattern=_IDENTIFIER_RE),
            generation_budget,
            repair_budget,
            "utf-8-proof-body",
            slots,
            slot_order,
            inventory,
            command_policy,
            ArtifactRef.from_value(source["prompt"], f"{label} prompt"),
        )


_LEDGER_KINDS = frozenset({"runtime_verified", "task_verified", "attempt_started", "raw_trace_closed", "policy_decision", "candidate_materialized", "compiler_started", "compiler_finished", "terminal"})
_TERMINAL_STATES = frozenset({"accepted", "agent_failed", "contract_failed", "policy_violated", "compiler_failed", "environment_blocked"})
_COMPILER_RESULTS = frozenset({"passed", "compiler_failed", "timed_out", "environment_blocked", "contract_failed"})


@dataclass(frozen=True)
class LedgerEvent:
    sequence: int
    run_id: str
    attempt: int
    kind: str
    previous_event_digest: Digest | None
    payload: Mapping[str, Any]
    event_digest: Digest

    @classmethod
    def from_value(cls, value: object, label: str = "ledger event") -> "LedgerEvent":
        source = _expect_object(value, label)
        require_exact_fields(source, ("schema_version", "sequence", "run_id", "attempt", "kind", "previous_event_digest", "payload", "event_digest"), label)
        if source["schema_version"] != _LEDGER_SCHEMA:
            raise ValidationError(f"{label} schema version is not recognized")
        kind = _expect_text(source["kind"], f"{label} kind")
        if kind not in _LEDGER_KINDS:
            raise ValidationError(f"{label} kind is not recognized")
        payload = _expect_object(source["payload"], f"{label} payload")
        _validate_json_tree(payload, f"{label} payload")
        previous = source["previous_event_digest"]
        if previous is not None and not isinstance(previous, dict):
            raise ValidationError(f"{label} previous_event_digest must be an object or null")
        return cls(_expect_nonnegative_int(source["sequence"], f"{label} sequence"), _expect_text(source["run_id"], f"{label} run_id", pattern=_IDENTIFIER_RE), _expect_nonnegative_int(source["attempt"], f"{label} attempt"), kind, None if previous is None else Digest.from_value(previous, f"{label} previous_event_digest"), _freeze_json(payload), Digest.from_value(source["event_digest"], f"{label} event_digest"))


@dataclass(frozen=True)
class CandidateSlotRecord:
    slot_id: str
    proof_body: ArtifactRef
    materialized_source: ArtifactRef
    seal_check: str

    @classmethod
    def from_value(cls, value: object, label: str) -> "CandidateSlotRecord":
        source = _expect_object(value, label)
        require_exact_fields(source, ("slot_id", "proof_body", "materialized_source", "seal_check"), label)
        if source["seal_check"] != "passed":
            raise ValidationError(f"{label} seal_check is not recognized")
        return cls(_expect_text(source["slot_id"], f"{label} slot_id", pattern=_IDENTIFIER_RE), ArtifactRef.from_value(source["proof_body"], f"{label} proof_body"), ArtifactRef.from_value(source["materialized_source"], f"{label} materialized_source"), "passed")


@dataclass(frozen=True)
class CandidateRecord:
    run_id: str
    attempt: int
    slots: tuple[CandidateSlotRecord, ...]

    @classmethod
    def from_value(cls, value: object, label: str = "candidate record") -> "CandidateRecord":
        source = _expect_object(value, label)
        require_exact_fields(source, ("schema_version", "run_id", "attempt", "slots"), label)
        if source["schema_version"] != _CANDIDATE_SCHEMA:
            raise ValidationError(f"{label} schema version is not recognized")
        slots = source["slots"]
        if not isinstance(slots, list) or not slots:
            raise ValidationError(f"{label} slots must be a non-empty array")
        records = tuple(CandidateSlotRecord.from_value(item, f"{label} slots[{index}]") for index, item in enumerate(slots))
        if len({record.slot_id for record in records}) != len(records):
            raise ValidationError(f"{label} slot ids must be unique")
        return cls(_expect_text(source["run_id"], f"{label} run_id", pattern=_IDENTIFIER_RE), _expect_nonnegative_int(source["attempt"], f"{label} attempt"), records)


@dataclass(frozen=True)
class DeclarationAudit:
    required: tuple[str, ...]
    observed: tuple[str, ...]
    forbidden_axioms: tuple[str, ...]
    result: str

    @classmethod
    def from_value(cls, value: object, label: str) -> "DeclarationAudit":
        source = _expect_object(value, label)
        require_exact_fields(source, ("required", "observed", "forbidden_axioms", "result"), label)
        if source["result"] not in {"passed", "failed"}:
            raise ValidationError(f"{label} result is not recognized")
        parsed: list[tuple[str, ...]] = []
        for field in ("required", "observed", "forbidden_axioms"):
            value = source[field]
            if not isinstance(value, list) or any(not isinstance(item, str) or _LEAN_IDENTIFIER_RE.fullmatch(item) is None for item in value):
                raise ValidationError(f"{label} {field} must be Lean declaration names")
            parsed.append(tuple(value))
        return cls(parsed[0], parsed[1], parsed[2], source["result"])


@dataclass(frozen=True)
class CompilerReport:
    run_id: str
    attempt: int
    runtime_id: str
    task_id: str
    argv: tuple[str, ...]
    cwd: str
    network: str
    started_at_utc: str
    completed_at_utc: str
    exit_code: int | None
    stdout: ArtifactRef
    stderr: ArtifactRef
    declaration_audit: DeclarationAudit
    result: str

    @classmethod
    def from_value(cls, value: object, label: str = "compiler report") -> "CompilerReport":
        source = _expect_object(value, label)
        fields = ("schema_version", "run_id", "attempt", "runtime_id", "task_id", "argv", "cwd", "network", "started_at_utc", "completed_at_utc", "exit_code", "stdout", "stderr", "declaration_audit", "result")
        require_exact_fields(source, fields, label)
        if source["schema_version"] != _COMPILER_SCHEMA:
            raise ValidationError(f"{label} schema version is not recognized")
        argv = source["argv"]
        if not isinstance(argv, list) or not argv or any(not isinstance(item, str) or not item for item in argv):
            raise ValidationError(f"{label} argv must be a non-empty text array")
        if source["network"] != "disabled":
            raise ValidationError(f"{label} network must be disabled")
        exit_code = source["exit_code"]
        if exit_code is not None and (isinstance(exit_code, bool) or not isinstance(exit_code, int)):
            raise ValidationError(f"{label} exit_code must be an integer or null")
        result = source["result"]
        if result not in _COMPILER_RESULTS:
            raise ValidationError(f"{label} result is not recognized")
        return cls(_expect_text(source["run_id"], f"{label} run_id", pattern=_IDENTIFIER_RE), _expect_nonnegative_int(source["attempt"], f"{label} attempt"), _expect_text(source["runtime_id"], f"{label} runtime_id", pattern=_IDENTIFIER_RE), _expect_text(source["task_id"], f"{label} task_id", pattern=_IDENTIFIER_RE), tuple(argv), validate_relative_artifact_path(source["cwd"], f"{label} cwd"), "disabled", _expect_text(source["started_at_utc"], f"{label} started_at_utc"), _expect_text(source["completed_at_utc"], f"{label} completed_at_utc"), exit_code, ArtifactRef.from_value(source["stdout"], f"{label} stdout"), ArtifactRef.from_value(source["stderr"], f"{label} stderr"), DeclarationAudit.from_value(source["declaration_audit"], f"{label} declaration_audit"), result)


@dataclass(frozen=True)
class RunResult:
    run_id: str
    terminal_state: str
    accepted_attempt: int | None
    request_digest: Digest
    ledger_digest: Digest
    raw_trace_digest: Digest
    candidate_digest: Digest
    compiler_report_digest: Digest

    @classmethod
    def from_value(cls, value: object, label: str = "run result") -> "RunResult":
        source = _expect_object(value, label)
        fields = ("schema_version", "run_id", "terminal_state", "accepted_attempt", "request_digest", "ledger_digest", "raw_trace_digest", "candidate_digest", "compiler_report_digest")
        require_exact_fields(source, fields, label)
        if source["schema_version"] != _RESULT_SCHEMA:
            raise ValidationError(f"{label} schema version is not recognized")
        state = source["terminal_state"]
        if state not in _TERMINAL_STATES:
            raise ValidationError(f"{label} terminal_state is not recognized")
        attempt = source["accepted_attempt"]
        if state == "accepted":
            attempt = _expect_nonnegative_int(attempt, f"{label} accepted_attempt")
        elif attempt is not None:
            raise ValidationError(f"{label} accepted_attempt must be null for non-accepted runs")
        return cls(_expect_text(source["run_id"], f"{label} run_id", pattern=_IDENTIFIER_RE), state, attempt, _expect_digest(source["request_digest"], f"{label} request_digest"), _expect_digest(source["ledger_digest"], f"{label} ledger_digest"), _expect_digest(source["raw_trace_digest"], f"{label} raw_trace_digest"), _expect_digest(source["candidate_digest"], f"{label} candidate_digest"), _expect_digest(source["compiler_report_digest"], f"{label} compiler_report_digest"))


@dataclass(frozen=True)
class Receipt:
    run_id: str
    task_id: str
    runtime_id: str
    terminal_state: str
    accepted_attempt: int | None
    task_seal_digest: Digest
    runtime_lock_digest: Digest
    ledger_digest: Digest
    candidate_digest: Digest
    compiler_report_digest: Digest
    receipt_digest: Digest

    @classmethod
    def from_value(cls, value: object, label: str = "receipt") -> "Receipt":
        source = _expect_object(value, label)
        fields = ("schema_version", "run_id", "task_id", "runtime_id", "terminal_state", "accepted_attempt", "task_seal_digest", "runtime_lock_digest", "ledger_digest", "candidate_digest", "compiler_report_digest", "receipt_digest")
        require_exact_fields(source, fields, label)
        if source["schema_version"] != _RECEIPT_SCHEMA:
            raise ValidationError(f"{label} schema version is not recognized")
        state = source["terminal_state"]
        if state not in _TERMINAL_STATES:
            raise ValidationError(f"{label} terminal_state is not recognized")
        attempt = source["accepted_attempt"]
        if state == "accepted":
            attempt = _expect_nonnegative_int(attempt, f"{label} accepted_attempt")
        elif attempt is not None:
            raise ValidationError(f"{label} accepted_attempt must be null for non-accepted receipts")
        return cls(_expect_text(source["run_id"], f"{label} run_id", pattern=_IDENTIFIER_RE), _expect_text(source["task_id"], f"{label} task_id", pattern=_IDENTIFIER_RE), _expect_text(source["runtime_id"], f"{label} runtime_id", pattern=_IDENTIFIER_RE), state, attempt, _expect_digest(source["task_seal_digest"], f"{label} task_seal_digest"), _expect_digest(source["runtime_lock_digest"], f"{label} runtime_lock_digest"), _expect_digest(source["ledger_digest"], f"{label} ledger_digest"), _expect_digest(source["candidate_digest"], f"{label} candidate_digest"), _expect_digest(source["compiler_report_digest"], f"{label} compiler_report_digest"), _expect_digest(source["receipt_digest"], f"{label} receipt_digest"))
