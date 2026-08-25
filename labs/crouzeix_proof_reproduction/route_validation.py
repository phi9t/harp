"""Strict, read-only contracts for Crouzeix route proof evidence."""

from __future__ import annotations

import hashlib
import csv
import errno
import io
import json
import os
import re
import stat
import subprocess
from dataclasses import asdict, dataclass
from pathlib import Path, PurePosixPath
from typing import Any, Mapping, Sequence

try:
    from .provider_independence import (
        ProviderIndependenceError,
        audit_provider_independence,
        parse_active_imports,
    )
except ImportError:  # pragma: no cover - direct script import path
    from provider_independence import (
        ProviderIndependenceError,
        audit_provider_independence,
        parse_active_imports,
    )

MANIFEST_SCHEMA_VERSION = "crouzeix-route-proof-manifest/v1"
RECEIPT_SCHEMA_VERSION = "crouzeix-route-proof-receipt/v1"
REVIEW_SCHEMA_VERSION = "crouzeix-proof-review/v1"
ROUTE_IDS = ("jin", "lorist-schwenninger", "harp")
CLAIM_KINDS = ("source-faithful", "derived")
PROVENANCE_KINDS = ("source", "shared-foundation", "reused-route", "derived")
CORRESPONDENCE_KINDS = (
    "direct-source",
    "compatibility-port",
    "structural-refactor",
    "derived-extraction",
    "shared-foundation",
    "reused-route",
)
ALLOWED_CORRESPONDENCE_KINDS = {
    "source": frozenset({"direct-source", "compatibility-port", "structural-refactor"}),
    "derived": frozenset({"derived-extraction"}),
    "shared-foundation": frozenset({"shared-foundation"}),
    "reused-route": frozenset({"reused-route"}),
}
FINDING_SEVERITIES = ("Critical", "Important", "Minor")
ALLOWED_AXIOMS = ("Classical.choice", "Quot.sound", "propext")
MAX_JSON_BYTES = 2 * 1024 * 1024
MAX_CACHE_ARTIFACT_BYTES = 8 * 1024 * 1024
MAX_STRING_BYTES = 16 * 1024
MAX_ARRAY_ITEMS = 4096
MAX_JSON_DEPTH = 32
PINNED_TOOLCHAIN = "leanprover/lean4:v4.32.1"
COMMAND_SCHEMA_VERSION = "crouzeix-route-command/v1"
AXIOM_SCHEMA_VERSION = "crouzeix-route-axiom-audit/v1"
SHA256_RE = re.compile(r"[0-9a-f]{64}\Z")
GIT_ID_RE = re.compile(r"[0-9a-f]{40,64}\Z")
ARXIV_IDENTITY_RE = re.compile(r"arxiv:[0-9]{4}\.[0-9]{4,5}v[1-9][0-9]*\Z")
IDENTITY_RE = re.compile(
    r"(?:sha256:[0-9a-f]{64}|arxiv:[0-9]{4}\.[0-9]{4,5}v[1-9][0-9]*)\Z"
)
MODULE_RE = re.compile(r"[A-Za-z_][A-Za-z0-9_']*(?:\.[A-Za-z_][A-Za-z0-9_']*)*\Z")
NODE_ID_RE = re.compile(r"[a-z0-9][a-z0-9-]{0,127}\Z")
LOCAL_SOURCE_LOCATOR_RE = re.compile(
    r"(?P<path>[^#]+)#L(?P<start>[1-9][0-9]*)-L(?P<end>[1-9][0-9]*)\Z"
)
GIT_SOURCE_LOCATOR_RE = re.compile(
    r"git:(?P<commit>[0-9a-f]{40}):(?P<path>[^#]+)"
    r"#L(?P<start>[1-9][0-9]*)-L(?P<end>[1-9][0-9]*)\Z"
)
ARXIV_SOURCE_LOCATOR_RE = re.compile(
    r"arxiv:(?P<version>[0-9]{4}\.[0-9]{4,5}v[1-9][0-9]*):"
    r"(?P<path>[A-Za-z0-9._/-]+)"
    r"#L(?P<start>[1-9][0-9]*)-L(?P<end>[1-9][0-9]*)\Z"
)
NODE_ROLES = ("load-bearing", "terminal", "consequence")

ROUTE_AGGREGATES = {
    "jin": "CrouzeixJin",
    "lorist-schwenninger": "CrouzeixLoristSchwenninger",
    "harp": "CrouzeixHarp",
}
ROUTE_MANIFEST_PATHS = {
    "jin": Path("labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/route-manifest.json"),
    "lorist-schwenninger": Path("labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/route-manifest.json"),
    "harp": Path("labs/crouzeix_proof_reproduction/formal_targets/harp/route-manifest.json"),
}
ROUTE_STAGING_PARENT = Path(".build/crouzeix-route-publication-staging")
ROUTE_FINAL_ROOT = Path("evidence/crouzeix_conjecture/routes")
ROUTE_REVIEW_CANDIDATE_PARENT = Path(".build/crouzeix-route-review-candidates")
ROUTE_REVIEW_FINAL_ROOT = Path("evidence/crouzeix_conjecture/reviews")
ROUTE_RECEIPT_MEMBERS = frozenset({
    "receipt.json",
    "build/command.json",
    "build/stdout.log",
    "build/stderr.log",
    "audit/axioms.json",
    "audit/provider.json",
})
HARP_ALLOWED_LS_SUPPORT = frozenset({
    "Crouzeix.LoristSchwenninger.BoundaryEmbedding",
    "Crouzeix.LoristSchwenninger.BoundaryMultiplier",
    "Crouzeix.LoristSchwenninger.BoundarySquareRoot",
    "Crouzeix.LoristSchwenninger.CompanionAlgebra",
    "Crouzeix.LoristSchwenninger.CompletedSquare",
    "Crouzeix.LoristSchwenninger.CompressionMoments",
    "Crouzeix.LoristSchwenninger.Dilation",
    "Crouzeix.LoristSchwenninger.NormAttainment",
    "Crouzeix.LoristSchwenninger.PolynomialPowerCauchy",
    "Crouzeix.LoristSchwenninger.Recurrence",
    "Crouzeix.LoristSchwenninger.Scalar",
})
LS_ARTIFACT_MANIFEST_PATH = Path(
    "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger/artifact-manifest.json"
)
SOURCE_MANIFEST_PATH = Path("evidence/crouzeix_conjecture/source_manifest.tsv")
LS_SOURCE_ID = "LS-ARXIV-V1"
LS_SOURCE_IDENTITY = "arxiv:2608.03841v1"
LS_SOURCE_ARCHIVE_SHA256 = "b4b6ddcdd726897826db500743e06649eddee5de5f32ded53e0d1adaa5f512c9"
LS_SOURCE_FILE_SHA256 = "20aad7aedd831e32e8a8b452fc51542185251a052830620c2201ff9863709f0a"
LS_SOURCE_PATH = "CrouzeixConjecturev2.tex"
LS_SOURCE_BYTES = 18_783
LS_SOURCE_LINES = 281
LS_SOURCE_IDENTITIES = tuple(sorted((
    LS_SOURCE_IDENTITY,
    f"sha256:{LS_SOURCE_ARCHIVE_SHA256}",
    f"sha256:{LS_SOURCE_FILE_SHA256}",
)))
LS_ARTIFACT_MANIFEST_FIELDS = frozenset({
    "archive_sha256",
    "manuscript_bytes",
    "manuscript_path",
    "manuscript_sha256",
    "line_count",
    "schema_version",
    "source_id",
    "source_identity",
})
SOURCE_MANIFEST_FIELDS = (
    "schema_version",
    "receipt_id",
    "source_id",
    "source_class",
    "role",
    "immutable_identity",
    "source_url",
    "upstream_path",
    "bytes",
    "sha256",
    "local_path",
    "observed",
    "license_status",
    "redistribution_status",
)


class RouteValidationError(ValueError):
    """A route artifact is malformed, inconsistent, or unsafe."""


@dataclass(frozen=True)
class RouteNode:
    node_id: str
    role: str
    declaration: str
    module_path: str
    dependency_ids: tuple[str, ...]
    provenance_kind: str
    correspondence_kind: str
    source_locator: str | None
    source_archive_sha256: str | None
    source_file_sha256: str | None
    source_excerpt_sha256: str | None
    source_line_count: int | None
    reused_from_route: str | None
    reused_node_id: str | None
    declaration_type_path: str
    statement_sha256: str


@dataclass(frozen=True)
class RouteManifest:
    schema_version: str
    route_id: str
    claim_kind: str
    aggregate_module: str
    build_target: str
    terminal_declaration: str
    terminal_type_sha256: str
    consequence_declarations: tuple[str, ...]
    source_identities: tuple[str, ...]
    shared_foundation_modules: tuple[str, ...]
    module_closure: tuple[str, ...]
    module_closure_sha256: str
    allowed_axioms: tuple[str, ...]
    review_path: str
    review_sha256: str | None
    receipt_path: str
    receipt_sha256: str | None
    nodes: tuple[RouteNode, ...]


@dataclass(frozen=True)
class RouteValidationResult:
    route_id: str
    claim_level: str
    status: str
    manifest_path: str
    reason: str | None = None


MANIFEST_FIELDS = frozenset({
    "schema_version", "route_id", "claim_kind", "aggregate_module",
    "build_target", "terminal_declaration", "terminal_type_sha256",
    "consequence_declarations", "source_identities", "shared_foundation_modules",
    "module_closure", "module_closure_sha256", "allowed_axioms",
    "review_path", "review_sha256", "receipt_path", "receipt_sha256", "nodes",
})
NODE_FIELDS = frozenset({
    "node_id", "role", "declaration", "module_path", "dependency_ids",
    "provenance_kind", "correspondence_kind", "source_locator", "source_archive_sha256",
    "source_file_sha256", "source_excerpt_sha256", "source_line_count",
    "reused_from_route", "reused_node_id",
    "declaration_type_path", "statement_sha256",
})
RECEIPT_FIELDS = frozenset({
    "schema_version", "route_id", "aggregate_module", "build_target",
    "manifest_path", "manifest_sha256", "candidate_commit", "candidate_tree",
    "command_artifact_path", "command_artifact_sha256", "argv",
    "working_directory", "cache_identity", "toolchain", "local_closure_modules",
    "local_closure_sha256", "mathlib_artifacts", "mathlib_artifacts_sha256",
    "declaration_types", "allowed_axioms", "axiom_audit_path",
    "axiom_audit_sha256", "axiom_results", "provider_report_path",
    "provider_report_sha256", "stdout_path", "stdout_sha256", "stderr_path",
    "stderr_sha256", "exit_code", "status", "receipt_sha256",
})
REVIEW_FIELDS = frozenset({
    "schema_version", "route_id", "reviewed_commit", "reviewed_tree",
    "manifest_path", "manifest_sha256", "terminal_type_sha256", "review_id",
    "reviewer_identity", "reviewer_model", "reviewer_run_id", "verdict",
    "outcome", "source_fidelity_check", "derivation_reuse_check",
    "findings", "review_sha256",
})


def canonical_json_bytes(value: object) -> bytes:
    text = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False)
    return (text + "\n").encode("utf-8")


def _sha256(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def self_digest(value: Mapping[str, object], field: str) -> str:
    payload = dict(value)
    payload[field] = None
    return _sha256(canonical_json_bytes(payload))


def manifest_contract_sha256(value: Mapping[str, object] | RouteManifest) -> str:
    payload = route_manifest_to_dict(value) if isinstance(value, RouteManifest) else dict(value)
    payload["receipt_sha256"] = None
    payload["review_sha256"] = None
    return _sha256(canonical_json_bytes(payload))


def normalized_type_sha256(source: str) -> str:
    normalized = " ".join(source.split())
    if not normalized:
        raise RouteValidationError("declaration type cannot be empty")
    return _sha256(normalized.encode("utf-8"))


def string_roster_sha256(values: Sequence[str]) -> str:
    return _sha256(canonical_json_bytes(list(values)))


def record_roster_sha256(values: Sequence[Mapping[str, object]]) -> str:
    return _sha256(canonical_json_bytes(list(values)))


def module_relative_path(module: str) -> Path:
    return Path(*module.split(".")).with_suffix(".lean")


def mathlib_source_path(module: str) -> Path:
    return Path("formalization/lean/.lake/packages/mathlib") / module_relative_path(module)


def mathlib_artifact_path(module: str) -> Path:
    return (
        Path("formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean")
        / Path(*module.split("."))
    ).with_suffix(".olean")


def _mathlib_source_cache_relative(module: str) -> Path:
    if MODULE_RE.fullmatch(module) is None:
        raise RouteValidationError(f"invalid Mathlib module: {module!r}")
    return Path("packages/mathlib") / (
        Path("Mathlib.lean") if module == "Mathlib" else module_relative_path(module)
    )


def _mathlib_artifact_cache_relative(module: str) -> Path:
    if MODULE_RE.fullmatch(module) is None:
        raise RouteValidationError(f"invalid Mathlib module: {module!r}")
    return (
        Path("packages/mathlib/.lake/build/lib/lean") / Path(*module.split("."))
    ).with_suffix(".olean")


def active_local_closure(lean_root: Path, aggregate_module: str) -> tuple[str, ...]:
    repository_root = lean_root.resolve().parents[1]
    pending = [aggregate_module]
    visited: set[str] = set()
    while pending:
        module = pending.pop()
        if module in visited:
            continue
        path = lean_root / module_relative_path(module)
        if not path.exists():
            if _managed_local_module(module):
                raise RouteValidationError(f"missing managed local import: {module}")
            continue
        source = _read_bytes(
            repository_root,
            (Path("formalization/lean") / module_relative_path(module)).as_posix(),
            f"Lean module {module}",
        ).decode("utf-8")
        visited.add(module)
        try:
            imports = parse_active_imports(source, module)
        except ProviderIndependenceError as error:
            raise RouteValidationError(str(error)) from error
        for imported in reversed(imports):
            if _managed_local_module(imported):
                pending.append(imported)
    return tuple(sorted(visited))


def active_mathlib_closure(repo_root: Path, local_modules: Sequence[str]) -> tuple[str, ...]:
    lean_root = repo_root / "formalization/lean"
    cache_root = resolve_approved_cache_root(repo_root)
    pending: list[str] = []
    for module in local_modules:
        source = _read_bytes(
            repo_root,
            (Path("formalization/lean") / module_relative_path(module)).as_posix(),
            f"Lean module {module}",
        ).decode("utf-8")
        try:
            pending.extend(
                imported for imported in parse_active_imports(source, module)
                if imported == "Mathlib" or imported.startswith("Mathlib.")
            )
        except ProviderIndependenceError as error:
            raise RouteValidationError(str(error)) from error
    visited: set[str] = set()
    while pending:
        module = pending.pop()
        if module in visited:
            continue
        source = _read_mathlib_source_bytes(cache_root, module)
        try:
            text = source.decode("utf-8")
        except UnicodeDecodeError as error:
            raise RouteValidationError(f"Mathlib source is not UTF-8: {module}") from error
        visited.add(module)
        try:
            imports = parse_active_imports(text, module)
        except ProviderIndependenceError as error:
            raise RouteValidationError(str(error)) from error
        pending.extend(
            imported for imported in imports
            if imported == "Mathlib" or imported.startswith("Mathlib.")
        )
    return tuple(sorted(visited))


def _managed_local_module(module: str) -> bool:
    return (
        module in {"Crouzeix", *ROUTE_AGGREGATES.values()}
        or module.startswith(("Crouzeix.", "CrouzeixConjecture."))
    )


def resolve_approved_cache_root(repository_root: Path) -> Path:
    root = Path(repository_root).resolve()
    try:
        result = subprocess.run(
            ["git", "-C", str(root), "rev-parse", "--path-format=absolute", "--git-common-dir"],
            capture_output=True, text=True, check=False,
        )
    except OSError as error:
        raise RouteValidationError(f"approved primary cache lookup unavailable: {error}") from error
    if result.returncode != 0 or not result.stdout.strip():
        raise RouteValidationError("approved primary cache lookup failed")
    common = Path(result.stdout.strip()).resolve()
    if common.name != ".git":
        raise RouteValidationError("Git common-dir is not a primary .git directory")
    approved = common.parent / "formalization/lean/.lake"
    link = root / "formalization/lean/.lake"
    if root == common.parent.resolve():
        if link.is_symlink() or not link.is_dir():
            raise RouteValidationError("primary cache must be a real directory")
    elif not link.is_symlink() or link.resolve() != approved.resolve():
        raise RouteValidationError("worktree .lake does not target approved primary cache")
    approved = approved.resolve(strict=True)
    for relative in (Path("packages"), Path("packages/mathlib")):
        probe = approved / relative
        if probe.is_symlink():
            raise RouteValidationError(f"approved cache contains nested symlink: {relative}")
    return approved


def _read_cache_bytes(cache_root: Path, relative: Path | str, label: str) -> bytes:
    relative_text = relative.as_posix() if isinstance(relative, Path) else relative
    parts = relative_text.split("/")
    prefix = ["packages", "mathlib", ".lake", "build", "lib", "lean"]
    if (
        relative_text.startswith("/")
        or not parts
        or any(part in {"", ".", ".."} for part in parts)
        or len(parts) <= len(prefix)
        or parts[:len(prefix)] != prefix
        or not relative_text.endswith(".olean")
    ):
        raise RouteValidationError(f"{label} must use a validated Mathlib artifact path")
    return _read_rooted_bytes(
        cache_root.resolve(strict=True),
        relative_text,
        label,
        max_bytes=MAX_CACHE_ARTIFACT_BYTES,
    )


def _read_mathlib_source_bytes(cache_root: Path, module: str) -> bytes:
    return _read_rooted_bytes(
        cache_root.resolve(strict=True),
        _mathlib_source_cache_relative(module).as_posix(),
        f"Mathlib source {module}",
    )


def _read_mathlib_artifact_bytes(cache_root: Path, module: str) -> bytes:
    return _read_cache_bytes(
        cache_root,
        _mathlib_artifact_cache_relative(module),
        "Mathlib artifact",
    )


def cache_contract_identity(repository_root: Path) -> str:
    lean_root = Path(repository_root) / "formalization/lean"
    toolchain = _read_bytes(Path(repository_root), "formalization/lean/lean-toolchain", "lean-toolchain").decode("utf-8").strip()
    if toolchain != PINNED_TOOLCHAIN:
        raise RouteValidationError("toolchain does not match pinned route toolchain")
    manifest = _read_json(Path(repository_root), Path("formalization/lean/lake-manifest.json"), "lake manifest")
    payload = {
        "schema_version": "crouzeix-route-cache-identity/v1",
        "toolchain": toolchain,
        "lake_manifest_sha256": _sha256(canonical_json_bytes(manifest)),
    }
    return f"sha256:{_sha256(canonical_json_bytes(payload))}"


def _pairs_no_duplicates(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise RouteValidationError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _check_json_shape(value: object, depth: int = 0) -> None:
    if depth > MAX_JSON_DEPTH:
        raise RouteValidationError("JSON nesting exceeds depth bound")
    if isinstance(value, str):
        if len(value.encode("utf-8")) > MAX_STRING_BYTES:
            raise RouteValidationError("JSON string exceeds size bound")
    elif isinstance(value, list):
        if len(value) > MAX_ARRAY_ITEMS:
            raise RouteValidationError("JSON array exceeds item bound")
        for item in value:
            _check_json_shape(item, depth + 1)
    elif isinstance(value, dict):
        for key, item in value.items():
            _check_json_shape(key, depth + 1)
            _check_json_shape(item, depth + 1)


def _safe_relative(value: object, label: str, *, locator: bool = False) -> str:
    if not isinstance(value, str) or not value or len(value.encode()) > MAX_STRING_BYTES:
        raise RouteValidationError(f"{label} must be a safe repository-relative path")
    path_text = value.split("#", 1)[0] if locator else value
    if path_text.startswith(("/", "\\")) or ":" in path_text.split("/", 1)[0]:
        raise RouteValidationError(f"{label} must be a safe repository-relative path")
    if chr(92) in path_text:
        raise RouteValidationError(f"{label} must be a safe repository-relative path")
    parts = path_text.split("/")
    if any(part in {"", ".", ".."} for part in parts):
        raise RouteValidationError(f"{label} must be a safe repository-relative path")
    if locator:
        fragment = value.split("#", 1)[1] if "#" in value else ""
        if re.fullmatch(r"L[1-9][0-9]*-L[1-9][0-9]*", fragment) is None:
            raise RouteValidationError(f"{label} requires a pinned #Lx-Ly source span")
    return path_text


def _source_locator(
    value: object, label: str
) -> tuple[str, str | None, str, int, int]:
    if not isinstance(value, str) or not value or len(value.encode()) > MAX_STRING_BYTES:
        raise RouteValidationError(f"{label} must be a pinned local or immutable Git locator")
    arxiv_match = ARXIV_SOURCE_LOCATOR_RE.fullmatch(value)
    if arxiv_match is not None:
        path = _safe_relative(arxiv_match.group("path"), f"{label} arXiv path")
        start = int(arxiv_match.group("start"))
        end = int(arxiv_match.group("end"))
        if start > end:
            raise RouteValidationError("reversed source locator span")
        identity = f"arxiv:{arxiv_match.group('version')}"
        return "arxiv", identity, path, start, end
    git_match = GIT_SOURCE_LOCATOR_RE.fullmatch(value)
    if git_match is not None:
        path = _safe_relative(git_match.group("path"), f"{label} Git path")
        start = int(git_match.group("start"))
        end = int(git_match.group("end"))
        if start > end:
            raise RouteValidationError("reversed source locator span")
        return "git", git_match.group("commit"), path, start, end
    local_match = LOCAL_SOURCE_LOCATOR_RE.fullmatch(value)
    if local_match is None:
        raise RouteValidationError(
            f"{label} requires path#Lx-Ly, git:<40hex>:path#Lx-Ly, "
            "or arxiv:<idvN>:path#Lx-Ly"
        )
    path = _safe_relative(local_match.group("path"), f"{label} local path")
    start = int(local_match.group("start"))
    end = int(local_match.group("end"))
    if start > end:
        raise RouteValidationError("reversed source locator span")
    return (
        "local",
        None,
        path,
        start,
        end,
    )


def _nullable_positive_int(value: object, label: str) -> int | None:
    if value is None:
        return None
    if not isinstance(value, int) or isinstance(value, bool) or value < 1:
        raise RouteValidationError(f"{label} must be null or a positive integer")
    return value


def _read_rooted_bytes(
    root_path: Path,
    relative: str,
    label: str,
    *,
    max_bytes: int = MAX_JSON_BYTES,
) -> bytes:
    root_fd = os.open(root_path, os.O_RDONLY | os.O_DIRECTORY | getattr(os, "O_CLOEXEC", 0))
    current_fd = root_fd
    try:
        parts = PurePosixPath(relative).parts
        for part in parts[:-1]:
            next_fd = os.open(
                part,
                os.O_RDONLY | os.O_DIRECTORY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0),
                dir_fd=current_fd,
            )
            if current_fd != root_fd:
                os.close(current_fd)
            current_fd = next_fd
        leaf_fd = os.open(
            parts[-1],
            os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0),
            dir_fd=current_fd,
        )
        try:
            metadata = os.fstat(leaf_fd)
            if not stat.S_ISREG(metadata.st_mode):
                raise RouteValidationError(f"{label} is not a regular file: {relative}")
            if metadata.st_nlink != 1:
                raise RouteValidationError(f"{label} cannot be a hardlink alias: {relative}")
            if metadata.st_size > max_bytes:
                raise RouteValidationError(f"{label} exceeds byte bound")
            with os.fdopen(leaf_fd, "rb", closefd=False) as handle:
                data = handle.read(max_bytes + 1)
            if len(data) > max_bytes:
                raise RouteValidationError(f"{label} exceeds byte bound")
            return data
        finally:
            os.close(leaf_fd)
    except OSError as error:
        if error.errno == errno.ELOOP:
            raise RouteValidationError(f"{label} path contains symlink: {relative}") from error
        raise RouteValidationError(f"cannot open {label} without following links: {relative}: {error}") from error
    finally:
        if current_fd != root_fd:
            os.close(current_fd)
        os.close(root_fd)


def _checked_file(repo_root: Path, value: object, label: str, *, locator: bool = False) -> Path:
    relative = _safe_relative(value, label, locator=locator)
    root = repo_root.resolve(strict=True)
    candidate = root / relative
    probe = root
    for part in PurePosixPath(relative).parts:
        probe = probe / part
        try:
            probe.lstat()
        except FileNotFoundError as error:
            raise RouteValidationError(f"{label} is missing: {relative}") from error
        if probe.is_symlink():
            raise RouteValidationError(f"{label} cannot contain a symlink: {relative}")
    metadata = candidate.stat()
    if not candidate.is_file():
        raise RouteValidationError(f"{label} is not a regular file: {relative}")
    if metadata.st_nlink != 1:
        raise RouteValidationError(f"{label} cannot be a hardlink alias: {relative}")
    try:
        candidate.resolve(strict=True).relative_to(root)
    except ValueError as error:
        raise RouteValidationError(f"{label} escapes repository root: {relative}") from error
    return candidate


def _read_bytes(repo_root: Path, value: object, label: str, *, locator: bool = False) -> bytes:
    relative = _safe_relative(value, label, locator=locator)
    return _read_rooted_bytes(repo_root.resolve(strict=True), relative, label)


def _read_json(repo_root: Path, path: Path | str, label: str) -> dict[str, object]:
    relative = path.as_posix() if isinstance(path, Path) else path
    data = _read_bytes(repo_root, relative, label)
    try:
        value = json.loads(data.decode("utf-8"), object_pairs_hook=_pairs_no_duplicates)
    except UnicodeDecodeError as error:
        raise RouteValidationError(f"{label} is not UTF-8") from error
    except json.JSONDecodeError as error:
        raise RouteValidationError(f"{label} is invalid JSON: {error.msg}") from error
    if not isinstance(value, dict):
        raise RouteValidationError(f"{label} must be a JSON object")
    _check_json_shape(value)
    return value


def _exact_fields(value: Mapping[str, object], fields: frozenset[str], label: str) -> None:
    unknown = sorted(set(value) - fields)
    missing = sorted(fields - set(value))
    if unknown:
        raise RouteValidationError(f"{label} has unknown field: {unknown[0]}")
    if missing:
        raise RouteValidationError(f"{label} is missing field: {missing[0]}")


def _text(value: object, label: str, *, pattern: re.Pattern[str] | None = None) -> str:
    if not isinstance(value, str) or not value or len(value.encode()) > MAX_STRING_BYTES:
        raise RouteValidationError(f"{label} must be a bounded nonempty string")
    if pattern is not None and pattern.fullmatch(value) is None:
        raise RouteValidationError(f"invalid {label}: {value!r}")
    return value


def _optional_digest(value: object, label: str) -> str | None:
    if value is None:
        return None
    return _text(value, label, pattern=SHA256_RE)


def _string_tuple(value: object, label: str, *, pattern: re.Pattern[str] | None = None, allow_empty: bool = False) -> tuple[str, ...]:
    if not isinstance(value, list) or len(value) > MAX_ARRAY_ITEMS or (not value and not allow_empty):
        raise RouteValidationError(f"{label} must be a bounded array")
    result = tuple(_text(item, label, pattern=pattern) for item in value)
    if len(result) != len(set(result)):
        raise RouteValidationError(f"duplicate {label}")
    return result


def parse_route_manifest(value: Mapping[str, object]) -> RouteManifest:
    _exact_fields(value, MANIFEST_FIELDS, "route manifest")
    if value["schema_version"] != MANIFEST_SCHEMA_VERSION:
        raise RouteValidationError("invalid route manifest schema version")
    route_id = _text(value["route_id"], "route id")
    if route_id not in ROUTE_IDS:
        raise RouteValidationError("invalid route id")
    claim_kind = _text(value["claim_kind"], "claim kind")
    if claim_kind not in CLAIM_KINDS:
        raise RouteValidationError("invalid claim kind")
    aggregate = _text(value["aggregate_module"], "aggregate module", pattern=MODULE_RE)
    if aggregate != ROUTE_AGGREGATES[route_id] or value["build_target"] != aggregate:
        raise RouteValidationError("route aggregate/build target mismatch")
    raw_nodes = value["nodes"]
    if not isinstance(raw_nodes, list) or not raw_nodes or len(raw_nodes) > MAX_ARRAY_ITEMS:
        raise RouteValidationError("nodes must be a bounded nonempty array")
    nodes: list[RouteNode] = []
    for raw in raw_nodes:
        if not isinstance(raw, dict):
            raise RouteValidationError("route node must be an object")
        _exact_fields(raw, NODE_FIELDS, "route node")
        provenance = _text(raw["provenance_kind"], "provenance kind")
        if provenance not in PROVENANCE_KINDS:
            raise RouteValidationError("invalid provenance kind")
        correspondence = _text(raw["correspondence_kind"], "correspondence kind")
        if correspondence not in CORRESPONDENCE_KINDS:
            raise RouteValidationError("invalid correspondence kind")
        if correspondence not in ALLOWED_CORRESPONDENCE_KINDS[provenance]:
            raise RouteValidationError(
                f"correspondence kind {correspondence!r} is incompatible with "
                f"provenance kind {provenance!r}"
            )
        source = raw["source_locator"]
        reused_route = raw["reused_from_route"]
        reused_node = raw["reused_node_id"]
        if source is not None:
            _source_locator(source, "source locator")
        source_archive_sha256 = _optional_digest(
            raw["source_archive_sha256"], "source archive sha256"
        )
        source_file_sha256 = _optional_digest(
            raw["source_file_sha256"], "source file sha256"
        )
        source_excerpt_sha256 = _optional_digest(
            raw["source_excerpt_sha256"], "source excerpt sha256"
        )
        source_line_count = _nullable_positive_int(
            raw["source_line_count"], "source line count"
        )
        if provenance == "source":
            if source is None:
                raise RouteValidationError("source node requires a pinned source locator")
            if reused_route is not None or reused_node is not None:
                raise RouteValidationError("source node cannot declare route reuse")
        elif provenance == "reused-route":
            valid_reuse = (
                reused_route in ROUTE_IDS
                and isinstance(reused_node, str)
                and NODE_ID_RE.fullmatch(reused_node) is not None
            )
            if not valid_reuse:
                raise RouteValidationError("reused-route node requires exact route and node identities")
            if source is not None:
                raise RouteValidationError("reused-route node cannot have a source locator")
        elif source is not None or reused_route is not None or reused_node is not None:
            raise RouteValidationError(f"{provenance} node cannot claim source or reuse provenance")
        if raw["role"] not in NODE_ROLES:
            raise RouteValidationError(f"invalid node role: {raw['role']!r}")
        nodes.append(RouteNode(
            node_id=_text(raw["node_id"], "node id", pattern=NODE_ID_RE),
            role=_text(raw["role"], "node role"),
            declaration=_text(raw["declaration"], "declaration", pattern=MODULE_RE),
            module_path=_safe_relative(raw["module_path"], "node module path"),
            dependency_ids=_string_tuple(raw["dependency_ids"], "dependency id", pattern=NODE_ID_RE, allow_empty=True),
            provenance_kind=provenance,
            correspondence_kind=correspondence,
            source_locator=source if isinstance(source, str) else None,
            source_archive_sha256=source_archive_sha256,
            source_file_sha256=source_file_sha256,
            source_excerpt_sha256=source_excerpt_sha256,
            source_line_count=source_line_count,
            reused_from_route=reused_route if isinstance(reused_route, str) else None,
            reused_node_id=reused_node if isinstance(reused_node, str) else None,
            declaration_type_path=_safe_relative(raw["declaration_type_path"], "declaration type path"),
            statement_sha256=_text(raw["statement_sha256"], "statement sha256", pattern=SHA256_RE),
        ))
    return RouteManifest(
        schema_version=MANIFEST_SCHEMA_VERSION,
        route_id=route_id,
        claim_kind=claim_kind,
        aggregate_module=aggregate,
        build_target=_text(value["build_target"], "build target", pattern=MODULE_RE),
        terminal_declaration=_text(value["terminal_declaration"], "terminal declaration", pattern=MODULE_RE),
        terminal_type_sha256=_text(value["terminal_type_sha256"], "terminal type sha256", pattern=SHA256_RE),
        consequence_declarations=_string_tuple(value["consequence_declarations"], "consequence declaration", pattern=MODULE_RE, allow_empty=True),
        source_identities=_string_tuple(value["source_identities"], "source identity", pattern=IDENTITY_RE, allow_empty=True),
        shared_foundation_modules=_string_tuple(value["shared_foundation_modules"], "shared foundation module", pattern=MODULE_RE, allow_empty=True),
        module_closure=_string_tuple(value["module_closure"], "module closure entry", pattern=MODULE_RE),
        module_closure_sha256=_text(value["module_closure_sha256"], "module closure sha256", pattern=SHA256_RE),
        allowed_axioms=_string_tuple(value["allowed_axioms"], "allowed axiom", pattern=MODULE_RE, allow_empty=True),
        review_path=_safe_relative(value["review_path"], "review path"),
        review_sha256=_optional_digest(value["review_sha256"], "review sha256"),
        receipt_path=_safe_relative(value["receipt_path"], "receipt path"),
        receipt_sha256=_optional_digest(value["receipt_sha256"], "receipt sha256"),
        nodes=tuple(nodes),
    )


def route_manifest_to_dict(manifest: RouteManifest | Mapping[str, object]) -> dict[str, object]:
    if not isinstance(manifest, RouteManifest):
        return dict(manifest)
    value = asdict(manifest)
    value["nodes"] = [asdict(node) for node in manifest.nodes]
    return value


def load_route_manifest(repo_root: Path, relative_path: Path) -> RouteManifest:
    return parse_route_manifest(_read_json(Path(repo_root), relative_path, "route manifest"))


def _module_from_path(path: str) -> str:
    if not path.endswith(".lean"):
        raise RouteValidationError(f"node module path must end in .lean: {path}")
    module = path[:-5].replace("/", ".")
    if MODULE_RE.fullmatch(module) is None:
        raise RouteValidationError(f"invalid node module path: {path}")
    return module


def _provider_policy(manifest: RouteManifest) -> tuple[tuple[str, ...], tuple[str, ...]]:
    if manifest.route_id == "jin":
        return ("Crouzeix", "CrouzeixLoristSchwenninger", "CrouzeixHarp"), ("Crouzeix.LoristSchwenninger", "Crouzeix.Harp")
    if manifest.route_id == "lorist-schwenninger":
        return ("Crouzeix", "CrouzeixJin", "CrouzeixHarp"), ("Crouzeix.Jin", "Crouzeix.Harp")
    return (
        "Crouzeix",
        "CrouzeixJin",
        "CrouzeixLoristSchwenninger",
        "Crouzeix.LoristSchwenninger.MainTheorem",
        "Crouzeix.LoristSchwenninger.Consequences",
    ), ("Crouzeix.Jin",)


def _source_excerpt(source: bytes, start: int, end: int) -> bytes:
    return b"".join(source.splitlines(keepends=True)[start - 1:end])


def _jin_artifact_manifest(repo_root: Path) -> dict[str, object]:
    return _read_json(
        repo_root,
        "labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/artifact-manifest.json",
        "Jin artifact manifest",
    )


def _read_source_manifest_rows(repo_root: Path) -> list[dict[str, str]]:
    data = _read_bytes(repo_root, SOURCE_MANIFEST_PATH.as_posix(), "source manifest")
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise RouteValidationError("source manifest is not UTF-8") from error
    reader = csv.DictReader(io.StringIO(text), delimiter="\t")
    if reader.fieldnames != list(SOURCE_MANIFEST_FIELDS):
        raise RouteValidationError("source manifest header mismatch")
    rows = list(reader)
    for row in rows:
        if set(row) != set(SOURCE_MANIFEST_FIELDS):
            raise RouteValidationError("source manifest row shape mismatch")
    return rows


def _source_manifest_row(
    repo_root: Path,
    receipt_id: str,
    *,
    source_id: str,
    source_class: str,
    role: str,
    immutable_identity: str,
    source_url: str,
    upstream_path: str,
    bytes_value: str,
    sha256: str,
    local_path: str,
    observed: str,
    license_status: str,
    redistribution_status: str,
) -> dict[str, str]:
    rows = _read_source_manifest_rows(repo_root)
    matching = [row for row in rows if row["receipt_id"] == receipt_id]
    if len(matching) != 1:
        raise RouteValidationError(f"expected exactly one source manifest row for {receipt_id}")
    row = matching[0]
    expected = {
        "schema_version": "crouzeix-source-receipt/v1",
        "receipt_id": receipt_id,
        "source_id": source_id,
        "source_class": source_class,
        "role": role,
        "immutable_identity": immutable_identity,
        "source_url": source_url,
        "upstream_path": upstream_path,
        "bytes": bytes_value,
        "sha256": sha256,
        "local_path": local_path,
        "observed": observed,
        "license_status": license_status,
        "redistribution_status": redistribution_status,
    }
    for field, expected_value in expected.items():
        if row[field] != expected_value:
            raise RouteValidationError(
                f"source manifest {receipt_id} field {field} mismatch"
            )
    return row


def _ls_artifact_manifest(repo_root: Path) -> dict[str, object]:
    artifact = _read_json(repo_root, LS_ARTIFACT_MANIFEST_PATH, "LS artifact manifest")
    _exact_fields(artifact, LS_ARTIFACT_MANIFEST_FIELDS, "LS artifact manifest")
    if artifact["schema_version"] != "crouzeix-arxiv-artifact-manifest/v1":
        raise RouteValidationError("invalid LS artifact manifest schema version")
    if artifact["source_id"] != LS_SOURCE_ID:
        raise RouteValidationError("LS artifact manifest source_id mismatch")
    if artifact["source_identity"] != LS_SOURCE_IDENTITY:
        raise RouteValidationError("LS artifact manifest source_identity mismatch")
    if artifact["archive_sha256"] != LS_SOURCE_ARCHIVE_SHA256:
        raise RouteValidationError("LS artifact manifest archive digest mismatch")
    if artifact["manuscript_sha256"] != LS_SOURCE_FILE_SHA256:
        raise RouteValidationError("LS artifact manifest manuscript digest mismatch")
    if artifact["manuscript_path"] != LS_SOURCE_PATH:
        raise RouteValidationError("LS artifact manifest manuscript path mismatch")
    if artifact["manuscript_bytes"] != LS_SOURCE_BYTES:
        raise RouteValidationError("LS artifact manifest manuscript byte count mismatch")
    if artifact["line_count"] != LS_SOURCE_LINES:
        raise RouteValidationError("LS artifact manifest line count mismatch")
    _source_manifest_row(
        repo_root,
        "LS-ARXIV-V1-SOURCE-ARCHIVE",
        source_id=LS_SOURCE_ID,
        source_class="arxiv-artifact",
        role="source",
        immutable_identity=LS_SOURCE_IDENTITY,
        source_url="https://export.arxiv.org/e-print/2608.03841v1",
        upstream_path="source.tar.gz",
        bytes_value="7330",
        sha256=LS_SOURCE_ARCHIVE_SHA256,
        local_path="-",
        observed="2026-08-14",
        license_status="arxiv-nonexclusive",
        redistribution_status="quotation-only",
    )
    _source_manifest_row(
        repo_root,
        "LS-ARXIV-V1-TEX",
        source_id=LS_SOURCE_ID,
        source_class="arxiv-artifact",
        role="manuscript",
        immutable_identity=LS_SOURCE_IDENTITY,
        source_url="https://export.arxiv.org/e-print/2608.03841v1",
        upstream_path=LS_SOURCE_PATH,
        bytes_value=str(LS_SOURCE_BYTES),
        sha256=LS_SOURCE_FILE_SHA256,
        local_path="-",
        observed="2026-08-14",
        license_status="arxiv-nonexclusive",
        redistribution_status="quotation-only",
    )
    return artifact


def validate_source_provenance_metadata(
    repo_root: Path, manifest: RouteManifest, node: RouteNode
) -> None:
    metadata = (
        node.source_archive_sha256,
        node.source_file_sha256,
        node.source_excerpt_sha256,
        node.source_line_count,
    )
    if node.provenance_kind != "source":
        if any(value is not None for value in metadata):
            raise RouteValidationError(
                f"{node.provenance_kind} node cannot claim source or reuse provenance"
            )
        return
    assert node.source_locator is not None
    kind, commit, path, start, end = _source_locator(node.source_locator, "source locator")
    if node.source_file_sha256 is None or node.source_excerpt_sha256 is None:
        raise RouteValidationError("source node requires file and excerpt digests")
    if node.source_line_count is None:
        raise RouteValidationError("source node requires a declared source line count")
    if end > node.source_line_count:
        raise RouteValidationError("source locator span exceeds file line count")
    if kind == "arxiv":
        if node.source_archive_sha256 is None:
            raise RouteValidationError("arXiv source locator requires an archive digest")
        if commit is None or ARXIV_IDENTITY_RE.fullmatch(commit) is None:
            raise RouteValidationError("invalid arXiv source identity")
        if f"sha256:{node.source_archive_sha256}" not in manifest.source_identities:
            raise RouteValidationError("source archive digest is not declared by a source identity")
        if f"sha256:{node.source_file_sha256}" not in manifest.source_identities:
            raise RouteValidationError("source file digest is not declared by a source identity")
        if commit not in manifest.source_identities:
            raise RouteValidationError("arXiv source identity is not declared by the route manifest")
        if manifest.route_id == "lorist-schwenninger":
            artifact = _ls_artifact_manifest(repo_root)
            if manifest.source_identities != LS_SOURCE_IDENTITIES:
                raise RouteValidationError("LS route manifest source identities must be the exact sorted arXiv identity set")
            if commit != artifact["source_identity"]:
                raise RouteValidationError("LS arXiv source identity does not match artifact manifest")
            if path != artifact["manuscript_path"]:
                raise RouteValidationError("LS arXiv source path does not match artifact manifest")
            if node.source_archive_sha256 != artifact["archive_sha256"]:
                raise RouteValidationError("LS source archive digest does not match artifact manifest")
            if node.source_file_sha256 != artifact["manuscript_sha256"]:
                raise RouteValidationError("LS source file digest does not match artifact manifest")
            if node.source_line_count != artifact["line_count"]:
                raise RouteValidationError("LS source line count does not match artifact manifest")
        return
    if kind == "local":
        if node.source_archive_sha256 is not None:
            raise RouteValidationError("local source locator cannot declare an archive digest")
        source_bytes = _read_bytes(repo_root, path, "source locator")
        try:
            observed_line_count = len(source_bytes.decode("utf-8").splitlines())
        except UnicodeDecodeError as error:
            raise RouteValidationError("source locator file is not UTF-8") from error
        if observed_line_count != node.source_line_count:
            raise RouteValidationError("source line count mismatch")
        if _sha256(source_bytes) != node.source_file_sha256:
            raise RouteValidationError("source file digest mismatch")
        if _sha256(_source_excerpt(source_bytes, start, end)) != node.source_excerpt_sha256:
            raise RouteValidationError("source excerpt digest mismatch")
        source_identity = f"sha256:{node.source_file_sha256}"
        if source_identity not in manifest.source_identities:
            raise RouteValidationError("source locator digest is not declared by a source identity")
        return

    if node.source_archive_sha256 is None:
        raise RouteValidationError("Git source locator requires an archive digest")
    if f"sha256:{node.source_archive_sha256}" not in manifest.source_identities:
        raise RouteValidationError("source archive digest is not declared by a source identity")
    if manifest.route_id != "jin":
        return
    artifact = _jin_artifact_manifest(repo_root)
    if artifact.get("source_commit") != commit:
        raise RouteValidationError("Jin Git source commit does not match artifact manifest")
    archive = artifact.get("archive")
    if not isinstance(archive, dict) or archive.get("sha256") != node.source_archive_sha256:
        raise RouteValidationError("Jin source archive digest does not match artifact manifest")
    artifacts = artifact.get("artifacts")
    if not isinstance(artifacts, dict):
        raise RouteValidationError("Jin artifact manifest artifacts must be an object")
    matches = [
        record for record in artifacts.values()
        if isinstance(record, dict) and record.get("path") == path
    ]
    if len(matches) != 1:
        raise RouteValidationError(
            "exactly one Jin artifact record must match Git source path"
        )
    if matches[0].get("sha256") != node.source_file_sha256:
        raise RouteValidationError("Jin source file digest does not match artifact manifest")


def _validate_manifest_semantics(repo_root: Path, lean_root: Path, manifest: RouteManifest) -> None:
    if manifest.route_id in {"jin", "lorist-schwenninger"}:
        if manifest.claim_kind != "source-faithful":
            raise RouteValidationError("Jin and Lorist-Schwenninger must be source-faithful")
        if not manifest.source_identities:
            raise RouteValidationError("source-faithful route requires source identities")
        if not any(node.provenance_kind == "source" for node in manifest.nodes):
            raise RouteValidationError("source-faithful route requires at least one source node")
        if any(
            node.provenance_kind not in {"source", "derived"}
            for node in manifest.nodes
        ):
            raise RouteValidationError("source-faithful route cannot reuse another route")
    else:
        if manifest.claim_kind != "derived":
            raise RouteValidationError("Harp route must be derived")
        if manifest.source_identities:
            raise RouteValidationError("Harp route source identities must be empty")
        reused = [node for node in manifest.nodes if node.provenance_kind == "reused-route"]
        if not reused:
            raise RouteValidationError("Harp route must explicitly record LS reuse")
        if any(node.reused_from_route != "lorist-schwenninger" for node in reused):
            raise RouteValidationError("Harp may reuse only lorist-schwenninger nodes")
    if manifest.allowed_axioms != tuple(sorted(manifest.allowed_axioms)):
        raise RouteValidationError("allowed axioms must be sorted")
    if any(item not in ALLOWED_AXIOMS for item in manifest.allowed_axioms):
        raise RouteValidationError("manifest contains forbidden axiom")
    ids = [node.node_id for node in manifest.nodes]
    if len(ids) != len(set(ids)):
        raise RouteValidationError("duplicate node_id")
    modules = [_module_from_path(node.module_path) for node in manifest.nodes]
    if len(modules) != len(set(modules)):
        raise RouteValidationError("duplicate node module")
    seen: set[str] = set()
    all_ids = set(ids)
    for node in manifest.nodes:
        canonical_module_path = module_relative_path(_module_from_path(node.module_path)).as_posix()
        if node.module_path != canonical_module_path:
            raise RouteValidationError(f"node module path is not canonical: {node.module_path}")
        for dependency in node.dependency_ids:
            if dependency == node.node_id:
                raise RouteValidationError("self dependency")
            if dependency not in all_ids:
                raise RouteValidationError(f"unknown dependency: {dependency}")
            if dependency not in seen:
                raise RouteValidationError("dependency cycle or unstable topological order")
        seen.add(node.node_id)
        validate_source_provenance_metadata(repo_root, manifest, node)
        type_bytes = _read_bytes(repo_root, node.declaration_type_path, "declaration type artifact")
        try:
            observed = normalized_type_sha256(type_bytes.decode("utf-8"))
        except UnicodeDecodeError as error:
            raise RouteValidationError("declaration type artifact is not UTF-8") from error
        if observed != node.statement_sha256:
            raise RouteValidationError(f"declaration type digest mismatch for {node.declaration}")
    terminals = [node for node in manifest.nodes if node.role == "terminal"]
    if len(terminals) != 1:
        raise RouteValidationError("exactly one terminal node is required")
    if terminals[0].declaration != manifest.terminal_declaration:
        raise RouteValidationError("terminal declaration mismatch")
    if terminals[0].statement_sha256 != manifest.terminal_type_sha256:
        raise RouteValidationError("terminal type digest mismatch")
    represented = {node.declaration for node in manifest.nodes}
    if any(item not in represented for item in manifest.consequence_declarations):
        raise RouteValidationError("consequence declaration is not represented")
    consequence_set = set(manifest.consequence_declarations)
    consequence_nodes = {node.declaration for node in manifest.nodes if node.role == "consequence"}
    if consequence_set - consequence_nodes:
        raise RouteValidationError("consequence role mismatch")
    if consequence_nodes - consequence_set:
        raise RouteValidationError("undeclared consequence-role node")
    forbidden, prefixes = _provider_policy(manifest)
    try:
        report = audit_provider_independence(
            lean_root,
            (manifest.aggregate_module,),
            forbidden,
            forbidden_prefixes=prefixes,
        )
    except ProviderIndependenceError as error:
        raise RouteValidationError(f"provider policy violation: {error}") from error
    if manifest.route_id == "harp":
        drift = sorted(
            module for module in report.modules
            if module.startswith("Crouzeix.LoristSchwenninger.")
            and module not in HARP_ALLOWED_LS_SUPPORT
        )
        if drift:
            raise RouteValidationError(f"provider policy drift: {drift[0]}")
    closure = active_local_closure(lean_root, manifest.aggregate_module)
    if closure != manifest.module_closure:
        missing = sorted(set(closure) - set(manifest.module_closure))
        if missing:
            raise RouteValidationError(f"unmapped active closure module: {missing[0]}")
        raise RouteValidationError("module closure roster mismatch")
    if string_roster_sha256(closure) != manifest.module_closure_sha256:
        raise RouteValidationError("module closure digest mismatch")
    node_modules = set(modules)
    shared = set(manifest.shared_foundation_modules)
    overlap = sorted(node_modules & shared)
    if overlap:
        raise RouteValidationError(f"module covered by both a node and shared foundation: {overlap[0]}")
    declared_coverage = node_modules | shared
    extra = sorted(declared_coverage - set(closure))
    if extra:
        raise RouteValidationError(f"inactive node/shared module in closure coverage: {extra[0]}")
    for module in closure:
        coverage = int(module in node_modules) + int(module in shared)
        if coverage == 0:
            raise RouteValidationError(f"unmapped active closure module: {module}")
        if coverage > 1:
            raise RouteValidationError(f"duplicate closure coverage: {module}")
    if manifest.route_id == "harp":
        active_ls = set(closure) & HARP_ALLOWED_LS_SUPPORT
        reused_nodes = [node for node in manifest.nodes if node.provenance_kind == "reused-route"]
        reused_modules = {_module_from_path(node.module_path) for node in reused_nodes}
        if active_ls != reused_modules:
            raise RouteValidationError("Harp LS support reuse coverage mismatch")
        if shared & HARP_ALLOWED_LS_SUPPORT:
            raise RouteValidationError("Harp LS support module cannot be shared foundation")
        ls_path = ROUTE_MANIFEST_PATHS["lorist-schwenninger"]
        try:
            ls_manifest = load_route_manifest(repo_root, ls_path)
        except RouteValidationError as error:
            raise RouteValidationError(f"LS route manifest is missing or invalid: {error}") from error
        ls_nodes = {node.node_id: node for node in ls_manifest.nodes}
        for node in reused_nodes:
            referenced = ls_nodes.get(node.reused_node_id or "")
            if referenced is None:
                raise RouteValidationError(f"LS reuse has dangling node: {node.reused_node_id}")
            if (
                referenced.module_path != node.module_path
                or referenced.declaration != node.declaration
                or referenced.statement_sha256 != node.statement_sha256
            ):
                raise RouteValidationError(f"LS reuse mismatch for node: {node.node_id}")


def _artifact_digest(repo_root: Path, path: object, expected: object, label: str) -> bytes:
    data = _read_bytes(repo_root, path, label)
    digest = _text(expected, f"{label} sha256", pattern=SHA256_RE)
    if _sha256(data) != digest:
        raise RouteValidationError(f"{label} digest mismatch")
    return data


def _validate_axiom_results(
    results: object, declarations: set[str], allowed: tuple[str, ...]
) -> tuple[dict[str, object], ...]:
    if not isinstance(results, list) or not results or len(results) > MAX_ARRAY_ITEMS:
        raise RouteValidationError("axiom results must be a bounded nonempty array")
    output: list[dict[str, object]] = []
    seen: set[str] = set()
    for raw in results:
        if not isinstance(raw, dict):
            raise RouteValidationError("axiom result must be an object")
        _exact_fields(raw, frozenset({"declaration", "axioms"}), "axiom result")
        declaration = _text(raw["declaration"], "axiom declaration", pattern=MODULE_RE)
        axioms = _string_tuple(raw["axioms"], "observed axiom", pattern=MODULE_RE, allow_empty=True)
        if axioms != tuple(sorted(axioms)):
            raise RouteValidationError("observed axioms must be sorted")
        if declaration in seen:
            raise RouteValidationError("duplicate axiom declaration")
        if declaration not in declarations:
            raise RouteValidationError("axiom result names unknown declaration")
        forbidden = sorted(set(axioms) - set(allowed))
        if forbidden:
            raise RouteValidationError(f"forbidden axiom: {forbidden[0]}")
        seen.add(declaration)
        output.append(dict(raw))
    if seen != declarations:
        raise RouteValidationError("axiom audit does not cover every route declaration")
    return tuple(output)


def _validate_receipt(
    repo_root: Path,
    manifest_path: Path,
    manifest_raw: Mapping[str, object],
    manifest: RouteManifest,
) -> dict[str, object]:
    raw = _read_json(repo_root, manifest.receipt_path, "route receipt")
    _exact_fields(raw, RECEIPT_FIELDS, "route receipt")
    if raw["schema_version"] != RECEIPT_SCHEMA_VERSION:
        raise RouteValidationError("invalid route receipt schema version")
    if raw["route_id"] != manifest.route_id:
        raise RouteValidationError("receipt route mismatch")
    if raw["aggregate_module"] != manifest.aggregate_module or raw["build_target"] != manifest.build_target:
        raise RouteValidationError("receipt aggregate mismatch")
    if raw["manifest_path"] != manifest_path.as_posix():
        raise RouteValidationError("receipt manifest path mismatch")
    if raw["manifest_sha256"] != manifest_contract_sha256(manifest_raw):
        raise RouteValidationError("receipt manifest digest mismatch")
    candidate_commit = _text(raw["candidate_commit"], "candidate commit", pattern=GIT_ID_RE)
    candidate_tree = _text(raw["candidate_tree"], "candidate tree", pattern=GIT_ID_RE)
    _validate_git_identity(
        repo_root, candidate_commit, candidate_tree, manifest.module_closure,
        authority_paths=_ls_authority_paths(repo_root, manifest),
    )
    if raw["receipt_sha256"] != self_digest(raw, "receipt_sha256"):
        raise RouteValidationError("receipt identity digest mismatch")
    command_data = _artifact_digest(
        repo_root, raw["command_artifact_path"], raw["command_artifact_sha256"], "command artifact"
    )
    try:
        command = json.loads(command_data.decode(), object_pairs_hook=_pairs_no_duplicates)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RouteValidationError("command artifact is invalid JSON") from error
    command_fields = frozenset({
        "schema_version", "argv", "working_directory", "aggregate_module",
        "build_target", "cache_identity", "toolchain",
    })
    if not isinstance(command, dict):
        raise RouteValidationError("command artifact must be an object")
    _exact_fields(command, command_fields, "command artifact")
    if command["schema_version"] != COMMAND_SCHEMA_VERSION:
        raise RouteValidationError("command artifact schema version mismatch")
    for field in ("argv", "working_directory", "aggregate_module", "build_target", "cache_identity", "toolchain"):
        if command[field] != raw[field]:
            raise RouteValidationError(f"receipt command mismatch: {field}")
    if raw["argv"] != ["scripts/check_lean_library.sh", manifest.build_target]:
        raise RouteValidationError("receipt command mismatch: argv")
    if raw["working_directory"] != ".":
        raise RouteValidationError("working directory must be repository root '.'")
    if raw["toolchain"] != PINNED_TOOLCHAIN:
        raise RouteValidationError("toolchain mismatch")
    if raw["cache_identity"] != cache_contract_identity(repo_root):
        raise RouteValidationError("cache identity mismatch")
    if raw["local_closure_modules"] != list(manifest.module_closure):
        raise RouteValidationError("receipt local closure mismatch")
    if raw["local_closure_sha256"] != manifest.module_closure_sha256:
        raise RouteValidationError("receipt local closure mismatch")
    artifacts = raw["mathlib_artifacts"]
    if not isinstance(artifacts, list):
        raise RouteValidationError("Mathlib artifacts must be an array")
    expected_mathlib = active_mathlib_closure(repo_root, manifest.module_closure)
    expected_pairs = [
        (module, mathlib_artifact_path(module).as_posix()) for module in expected_mathlib
    ]
    actual_pairs = [
        (item.get("module"), item.get("path"))
        for item in artifacts
        if isinstance(item, dict)
    ]
    if len(actual_pairs) != len(artifacts) or actual_pairs != expected_pairs:
        raise RouteValidationError("Mathlib artifact roster mismatch")
    cache_root = resolve_approved_cache_root(repo_root)
    for item in artifacts:
        if not isinstance(item, dict):
            raise RouteValidationError("Mathlib artifact entry must be an object")
        _exact_fields(item, frozenset({"module", "path", "sha256"}), "Mathlib artifact")
        data = _read_mathlib_artifact_bytes(cache_root, str(item["module"]))
        if _sha256(data) != item["sha256"]:
            raise RouteValidationError("Mathlib artifact digest mismatch")
    if raw["mathlib_artifacts_sha256"] != record_roster_sha256(artifacts):
        raise RouteValidationError("Mathlib roster digest mismatch")
    types = raw["declaration_types"]
    if not isinstance(types, list) or len(types) != len(manifest.nodes):
        raise RouteValidationError("receipt declaration type roster mismatch")
    expected_nodes = {node.declaration: node for node in manifest.nodes}
    actual_declarations = [item.get("declaration") for item in types if isinstance(item, dict)]
    if len(actual_declarations) != len(set(actual_declarations)) or set(actual_declarations) != set(expected_nodes):
        raise RouteValidationError("receipt declaration type roster mismatch")
    for item in types:
        if not isinstance(item, dict):
            raise RouteValidationError("declaration type entry must be an object")
        _exact_fields(
            item,
            frozenset({"declaration", "type_artifact_path", "type_artifact_sha256", "statement_sha256"}),
            "declaration type entry",
        )
        declaration = item["declaration"]
        if declaration not in expected_nodes:
            raise RouteValidationError("receipt declaration type names unknown declaration")
        node = expected_nodes[str(declaration)]
        if item["type_artifact_path"] != node.declaration_type_path or item["statement_sha256"] != node.statement_sha256:
            raise RouteValidationError("receipt declaration type mismatch")
        data = _artifact_digest(
            repo_root, item["type_artifact_path"], item["type_artifact_sha256"], "declaration type artifact"
        )
        if normalized_type_sha256(data.decode()) != node.statement_sha256:
            raise RouteValidationError("receipt declaration type digest mismatch")
    if raw["allowed_axioms"] != list(manifest.allowed_axioms):
        raise RouteValidationError("receipt allowed axioms mismatch")
    declarations = set(expected_nodes)
    receipt_results = _validate_axiom_results(raw["axiom_results"], declarations, manifest.allowed_axioms)
    audit_data = _artifact_digest(repo_root, raw["axiom_audit_path"], raw["axiom_audit_sha256"], "axiom audit")
    try:
        audit = json.loads(audit_data.decode(), object_pairs_hook=_pairs_no_duplicates)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RouteValidationError("axiom audit is invalid JSON") from error
    audit_fields = frozenset({"schema_version", "route_id", "allowed_axioms", "results", "status"})
    if not isinstance(audit, dict):
        raise RouteValidationError("axiom audit must be an object")
    _exact_fields(audit, audit_fields, "axiom audit")
    audit_matches = (
        audit["schema_version"] == AXIOM_SCHEMA_VERSION
        and audit["route_id"] == manifest.route_id
        and audit["allowed_axioms"] == list(manifest.allowed_axioms)
        and audit["results"] == list(receipt_results)
        and audit["status"] == "passed"
    )
    if not audit_matches:
        raise RouteValidationError("axiom audit mismatch")
    provider_data = _artifact_digest(
        repo_root, raw["provider_report_path"], raw["provider_report_sha256"], "provider report"
    )
    try:
        provider = json.loads(provider_data.decode(), object_pairs_hook=_pairs_no_duplicates)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RouteValidationError("provider report is invalid JSON") from error
    expected_provider = {
        "schema_version": "crouzeix-route-provider-report/v1",
        "route_id": manifest.route_id,
        "aggregate_module": manifest.aggregate_module,
        "modules": list(manifest.module_closure),
        "module_closure_sha256": manifest.module_closure_sha256,
        "status": "passed",
    }
    if provider != expected_provider:
        raise RouteValidationError("provider report content mismatch")
    _artifact_digest(repo_root, raw["stdout_path"], raw["stdout_sha256"], "stdout")
    _artifact_digest(repo_root, raw["stderr_path"], raw["stderr_sha256"], "stderr")
    if (raw["status"] == "passed") != (raw["exit_code"] == 0):
        raise RouteValidationError("receipt status/exit mismatch")
    if raw["status"] != "passed" or raw["exit_code"] != 0:
        raise RouteValidationError("route receipt is not successful")
    return raw


def _validate_git_identity(
    repo_root: Path,
    commit: str,
    expected_tree: str,
    local_modules: Sequence[str],
    *,
    authority_paths: Sequence[Path] = (),
) -> None:
    try:
        exists = subprocess.run(
            ["git", "-C", str(repo_root), "cat-file", "-e", f"{commit}^{{commit}}"],
            capture_output=True,
            text=True,
            check=False,
        )
        if exists.returncode != 0:
            detail = exists.stderr.strip() or "object is unreachable"
            raise RouteValidationError(f"candidate commit is not a reachable Git commit: {detail}")
        ancestor = subprocess.run(
            ["git", "-C", str(repo_root), "merge-base", "--is-ancestor", commit, "HEAD"],
            capture_output=True,
            text=True,
            check=False,
        )
        if ancestor.returncode != 0:
            raise RouteValidationError("candidate commit is not an ancestor of HEAD")
        tree = subprocess.run(
            ["git", "-C", str(repo_root), "rev-parse", f"{commit}^{{tree}}"],
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as error:
        raise RouteValidationError(f"Git identity validation unavailable: {error}") from error
    if tree.returncode != 0:
        raise RouteValidationError(f"cannot resolve candidate Git tree: {tree.stderr.strip()}")
    if tree.stdout.strip() != expected_tree:
        raise RouteValidationError("candidate tree mismatch")
    bound_paths = [
        (Path("formalization/lean") / module_relative_path(module), f"route source {module}", "source")
        for module in local_modules
    ]
    bound_paths.extend((path, "route authority input", "authority") for path in authority_paths)
    for relative, label, kind in bound_paths:
        current = _read_bytes(repo_root, relative.as_posix(), label)
        try:
            candidate = subprocess.run(
                ["git", "-C", str(repo_root), "show", f"{commit}:{relative.as_posix()}"],
                capture_output=True,
                check=False,
            )
        except OSError as error:
            raise RouteValidationError(f"Git source binding unavailable: {error}") from error
        if candidate.returncode != 0 or candidate.stdout != current:
            raise RouteValidationError(
                f"candidate route {kind} blob mismatch: {relative.as_posix()}"
            )


def _ls_authority_paths(repo_root: Path, manifest: RouteManifest) -> tuple[Path, ...]:
    if manifest.route_id != "lorist-schwenninger":
        return ()
    try:
        from . import ls_validation
    except ImportError:  # pragma: no cover - direct script execution path
        import ls_validation
    try:
        return ls_validation.validated_route_authority_paths(repo_root)
    except Exception as error:
        raise RouteValidationError(f"LS graph state is invalid: {error}") from error


def _validate_review(
    repo_root: Path,
    manifest_path: Path,
    manifest_raw: Mapping[str, object],
    manifest: RouteManifest,
    receipt: Mapping[str, object],
) -> None:
    raw = _read_json(repo_root, manifest.review_path, "proof review")
    _validate_review_payload(manifest_path, manifest_raw, manifest, receipt, raw)


def _validate_review_payload(
    manifest_path: Path,
    manifest_raw: Mapping[str, object],
    manifest: RouteManifest,
    receipt: Mapping[str, object],
    raw: Mapping[str, object],
) -> None:
    _exact_fields(raw, REVIEW_FIELDS, "proof review")
    if raw["schema_version"] != REVIEW_SCHEMA_VERSION:
        raise RouteValidationError("invalid proof review schema version")
    if raw["review_sha256"] != self_digest(raw, "review_sha256"):
        raise RouteValidationError("review identity digest mismatch")
    if raw["route_id"] != manifest.route_id:
        raise RouteValidationError("review route mismatch")
    if raw["reviewed_commit"] != receipt["candidate_commit"] or raw["reviewed_tree"] != receipt["candidate_tree"]:
        raise RouteValidationError("review commit mismatch")
    if raw["manifest_path"] != manifest_path.as_posix():
        raise RouteValidationError("review manifest digest mismatch")
    if raw["manifest_sha256"] != manifest_contract_sha256(manifest_raw):
        raise RouteValidationError("review manifest digest mismatch")
    if raw["terminal_type_sha256"] != manifest.terminal_type_sha256:
        raise RouteValidationError("review terminal type mismatch")
    for field in ("review_id", "reviewer_identity", "reviewer_model", "reviewer_run_id"):
        _text(raw[field], field.replace("_", " "))
    if manifest.claim_kind == "source-faithful":
        checks_match = (
            raw["source_fidelity_check"] == "passed"
            and raw["derivation_reuse_check"] == "not-applicable"
        )
        if not checks_match:
            raise RouteValidationError("source-fidelity review checks mismatch")
    else:
        checks_match = (
            raw["source_fidelity_check"] == "not-applicable"
            and raw["derivation_reuse_check"] == "passed"
        )
        if not checks_match:
            raise RouteValidationError("derivation-reuse review checks mismatch")
    findings = raw["findings"]
    if not isinstance(findings, list) or len(findings) > MAX_ARRAY_ITEMS:
        raise RouteValidationError("review findings must be a bounded array")
    finding_fields = frozenset({
        "finding_id", "severity", "resolved", "locator", "statement",
        "falsifying_test_or_gap",
    })
    unresolved_blocking = False
    seen: set[str] = set()
    for finding in findings:
        if not isinstance(finding, dict):
            raise RouteValidationError("review finding must be an object")
        _exact_fields(finding, finding_fields, "review finding")
        finding_id = _text(finding["finding_id"], "finding id")
        if finding_id in seen:
            raise RouteValidationError("duplicate review finding id")
        seen.add(finding_id)
        if finding["severity"] not in FINDING_SEVERITIES:
            raise RouteValidationError("invalid finding severity")
        if not isinstance(finding["resolved"], bool):
            raise RouteValidationError("finding resolved must be boolean")
        _safe_relative(finding["locator"], "finding locator")
        _text(finding["statement"], "finding statement")
        _text(finding["falsifying_test_or_gap"], "falsifying test or gap")
        unresolved_blocking |= (
            finding["severity"] in {"Critical", "Important"}
            and not finding["resolved"]
        )
    if raw["verdict"] == "complete" and unresolved_blocking:
        raise RouteValidationError("complete review has unresolved Critical/Important finding")
    if raw["verdict"] != "complete" or raw["outcome"] != "approved":
        raise RouteValidationError("proof review is not complete and approved")


def validate_route_bundle(
    repo_root: Path,
    manifest_path: Path,
    *,
    lean_root: Path = Path("formalization/lean"),
    allow_unpublished: bool = False,
) -> RouteValidationResult:
    root = Path(repo_root).resolve(strict=True)
    manifest_raw = _read_json(root, manifest_path, "route manifest")
    manifest = parse_route_manifest(manifest_raw)
    _validate_manifest_semantics(root, root / lean_root, manifest)
    if manifest.route_id == "lorist-schwenninger":
        try:
            from . import ls_validation
        except ImportError:  # pragma: no cover - direct script execution path
            import ls_validation
        try:
            state = ls_validation.load_route_state(
                root / "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger"
            )
        except Exception as error:
            raise RouteValidationError(f"LS graph state is invalid: {error}") from error
        if state["promotion"] is None:
            if not allow_unpublished:
                raise RouteValidationError("LS graph is not promoted")
            return RouteValidationResult(
                manifest.route_id,
                "authored",
                "incomplete",
                manifest_path.as_posix(),
                "LS graph is not promoted",
            )
        _ls_authority_paths(root, manifest)
    receipt_candidate = root / manifest.receipt_path
    review_candidate = root / manifest.review_path
    receipt_present = receipt_candidate.exists() or receipt_candidate.is_symlink()
    review_present = review_candidate.exists() or review_candidate.is_symlink()
    if review_present and not receipt_present:
        raise RouteValidationError("review publication requires a receipt")
    if receipt_present != (manifest.receipt_sha256 is not None):
        raise RouteValidationError("receipt publication presence/digest mismatch")
    if review_present != (manifest.review_sha256 is not None):
        raise RouteValidationError("review publication presence/digest mismatch")
    receipt: dict[str, object] | None = None
    if receipt_present:
        receipt_bytes = _read_bytes(root, manifest.receipt_path, "route receipt")
        if manifest.receipt_sha256 is not None and _sha256(receipt_bytes) != manifest.receipt_sha256:
            raise RouteValidationError("manifest receipt digest mismatch")
        receipt = _validate_receipt(root, manifest_path, manifest_raw, manifest)
    if review_present:
        if receipt is None:
            raise RouteValidationError("proof review cannot exist without a route receipt")
        review_bytes = _read_bytes(root, manifest.review_path, "proof review")
        if manifest.review_sha256 is not None and _sha256(review_bytes) != manifest.review_sha256:
            raise RouteValidationError("manifest review digest mismatch")
        _validate_review(root, manifest_path, manifest_raw, manifest, receipt)
    if manifest.receipt_sha256 is not None and manifest.review_sha256 is not None:
        return RouteValidationResult(
            manifest.route_id, "complete-local", "complete", manifest_path.as_posix()
        )
    if not allow_unpublished:
        raise RouteValidationError("route has partial publication")
    claim_level = "receipt-backed" if manifest.receipt_sha256 is not None else "mapped"
    return RouteValidationResult(
        manifest.route_id,
        claim_level,
        "incomplete",
        manifest_path.as_posix(),
        "receipt or review is unpublished",
    )


def validate_route_receipt_candidate(
    repo_root: Path, manifest_path: Path, candidate_root: Path
) -> RouteValidationResult:
    """Validate a staged route receipt bundle without requiring final publication.

    The staged directory must live directly under the pinned publication staging
    parent. Candidate receipts still bind the final public paths; this seam
    remaps those final path fields to the staged bytes for validation only.
    """

    input_root = Path(os.path.abspath(os.fspath(repo_root)))
    root = input_root.resolve(strict=True)
    candidate = Path(os.path.abspath(os.fspath(candidate_root)))
    try:
        relative_candidate = candidate.relative_to(input_root)
    except ValueError as error:
        raise RouteValidationError("route receipt candidate is outside pinned staging parent") from error
    expected_parent = ROUTE_STAGING_PARENT.parts
    if (
        relative_candidate.parts[:-1] != expected_parent
        or not relative_candidate.name.startswith(".")
        or len(relative_candidate.parts) != len(expected_parent) + 1
    ):
        raise RouteValidationError("route receipt candidate must be a direct staging child")
    return _validate_route_receipt_tree(
        root, manifest_path, relative_candidate, require_final_absent=True
    )


def _validate_published_route_receipt(
    repo_root: Path, manifest_path: Path, published_root: Path
) -> RouteValidationResult:
    input_root = Path(os.path.abspath(os.fspath(repo_root)))
    root = input_root.resolve(strict=True)
    published = Path(os.path.abspath(os.fspath(published_root)))
    try:
        relative = published.relative_to(input_root)
    except ValueError as error:
        raise RouteValidationError("published route receipt escapes repository") from error
    return _validate_route_receipt_tree(
        root, manifest_path, relative, require_final_absent=False
    )


def _validate_route_receipt_tree(
    root: Path,
    manifest_path: Path,
    relative_root: Path,
    *,
    require_final_absent: bool,
) -> RouteValidationResult:
    manifest_raw = _read_json(root, manifest_path, "route manifest")
    manifest = parse_route_manifest(manifest_raw)
    final_root = Path(manifest.receipt_path).parent
    if require_final_absent and (
        manifest.receipt_sha256 is not None or manifest.review_sha256 is not None
    ):
        raise RouteValidationError("route manifest must be unpublished before receipt publication")
    if not require_final_absent and relative_root != final_root:
        raise RouteValidationError("published route receipt root mismatch")
    members = _read_exact_receipt_tree(root, relative_root)
    if require_final_absent and (
        _rooted_entry_exists(root, final_root)
        or _rooted_entry_exists(root, Path(manifest.review_path))
    ):
        raise RouteValidationError("route final publication path already exists")
    _validate_manifest_semantics(root, root / "formalization/lean", manifest)
    if (
        not require_final_absent
        and manifest.receipt_sha256 is not None
        and _sha256(members["receipt.json"]) != manifest.receipt_sha256
    ):
        raise RouteValidationError("manifest receipt digest mismatch")
    receipt_raw = _json_from_bytes(members["receipt.json"], "route receipt candidate")
    _validate_staged_receipt(
        root, manifest_path, manifest_raw, manifest, members, receipt_raw
    )
    return RouteValidationResult(
        manifest.route_id,
        "receipt-candidate",
        "valid",
        manifest_path.as_posix(),
    )

def _rooted_entry_exists(root: Path, relative: Path) -> bool:
    flags = os.O_RDONLY | os.O_DIRECTORY | getattr(os, "O_NOFOLLOW", 0)
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    descriptor = os.open(root, flags)
    try:
        for index, part in enumerate(relative.parts):
            try:
                metadata = os.stat(part, dir_fd=descriptor, follow_symlinks=False)
            except FileNotFoundError:
                return False
            if stat.S_ISLNK(metadata.st_mode):
                raise RouteValidationError("route publication path contains a symlink")
            if index == len(relative.parts) - 1:
                return True
            if not stat.S_ISDIR(metadata.st_mode):
                raise RouteValidationError("route publication ancestor is not a directory")
            next_fd = _open_directory_component(
                descriptor, part, "route publication ancestor"
            )
            os.close(descriptor)
            descriptor = next_fd
        return True
    finally:
        os.close(descriptor)


def _open_directory_component(parent_fd: int, name: str, label: str) -> int:
    flags = os.O_RDONLY | os.O_DIRECTORY | getattr(os, "O_NOFOLLOW", 0)
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    try:
        descriptor = os.open(name, flags, dir_fd=parent_fd)
    except OSError as error:
        raise RouteValidationError(f"cannot open {label} without following links: {error}") from error
    metadata = os.fstat(descriptor)
    if not stat.S_ISDIR(metadata.st_mode):
        os.close(descriptor)
        raise RouteValidationError(f"{label} is not a directory")
    return descriptor


def _read_file_at(directory_fd: int, name: str, label: str) -> bytes:
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    try:
        descriptor = os.open(name, flags, dir_fd=directory_fd)
    except OSError as error:
        if error.errno == errno.ELOOP:
            raise RouteValidationError(f"{label} is a symlink") from error
        raise RouteValidationError(f"cannot open {label} without following links: {error}") from error
    try:
        before = os.fstat(descriptor)
        if not stat.S_ISREG(before.st_mode):
            raise RouteValidationError(f"{label} is not a regular file")
        if before.st_nlink != 1:
            raise RouteValidationError(f"{label} cannot be a hardlink alias")
        if before.st_size > MAX_JSON_BYTES:
            raise RouteValidationError(f"{label} exceeds byte bound")
        chunks: list[bytes] = []
        size = 0
        while True:
            chunk = os.read(descriptor, min(64 * 1024, MAX_JSON_BYTES + 1 - size))
            if not chunk:
                break
            chunks.append(chunk)
            size += len(chunk)
            if size > MAX_JSON_BYTES:
                raise RouteValidationError(f"{label} exceeds byte bound")
        after = os.fstat(descriptor)
        current = os.stat(name, dir_fd=directory_fd, follow_symlinks=False)
        identity = lambda item: (
            item.st_dev, item.st_ino, item.st_mode, item.st_size,
            item.st_mtime_ns, item.st_ctime_ns, item.st_nlink,
        )
        if identity(before) != identity(after) or identity(after) != identity(current):
            raise RouteValidationError(f"{label} changed while being read")
        return b"".join(chunks)
    finally:
        os.close(descriptor)


def _exact_directory_names(directory_fd: int, expected: set[str], label: str) -> None:
    try:
        names = set(os.listdir(directory_fd))
    except OSError as error:
        raise RouteValidationError(f"cannot inspect {label}: {error}") from error
    if names != expected:
        missing = sorted(expected - names)
        unknown = sorted(names - expected)
        if missing:
            raise RouteValidationError(f"route receipt candidate missing member: {missing[0]}")
        raise RouteValidationError(f"route receipt candidate has unknown member: {unknown[0]}")


def _read_exact_receipt_tree(root: Path, relative_root: Path) -> dict[str, bytes]:
    flags = os.O_RDONLY | os.O_DIRECTORY | getattr(os, "O_NOFOLLOW", 0)
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    root_fd = os.open(root, flags)
    opened = [root_fd]
    try:
        current = root_fd
        for part in relative_root.parts:
            current = _open_directory_component(current, part, "route receipt directory")
            opened.append(current)
        _exact_directory_names(current, {"receipt.json", "build", "audit"}, "route receipt root")
        build_fd = _open_directory_component(current, "build", "route receipt build directory")
        opened.append(build_fd)
        audit_fd = _open_directory_component(current, "audit", "route receipt audit directory")
        opened.append(audit_fd)
        _exact_directory_names(build_fd, {"command.json", "stdout.log", "stderr.log"}, "route receipt build directory")
        _exact_directory_names(audit_fd, {"axioms.json", "provider.json"}, "route receipt audit directory")
        return {
            "receipt.json": _read_file_at(current, "receipt.json", "route receipt candidate receipt"),
            "build/command.json": _read_file_at(build_fd, "command.json", "command artifact"),
            "build/stdout.log": _read_file_at(build_fd, "stdout.log", "stdout"),
            "build/stderr.log": _read_file_at(build_fd, "stderr.log", "stderr"),
            "audit/axioms.json": _read_file_at(audit_fd, "axioms.json", "axiom audit"),
            "audit/provider.json": _read_file_at(audit_fd, "provider.json", "provider report"),
        }
    finally:
        for descriptor in reversed(opened):
            os.close(descriptor)


def _json_from_bytes(data: bytes, label: str) -> dict[str, object]:
    try:
        value = json.loads(data.decode("utf-8"), object_pairs_hook=_pairs_no_duplicates)
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RouteValidationError(f"{label} is invalid JSON") from error
    if not isinstance(value, dict):
        raise RouteValidationError(f"{label} must be an object")
    return value


def _validate_staged_receipt(
    repo_root: Path,
    manifest_path: Path,
    manifest_raw: Mapping[str, object],
    manifest: RouteManifest,
    members: Mapping[str, bytes],
    raw: Mapping[str, object],
) -> None:
    _exact_fields(raw, RECEIPT_FIELDS, "route receipt")
    if raw["schema_version"] != RECEIPT_SCHEMA_VERSION:
        raise RouteValidationError("invalid route receipt schema version")
    if raw["route_id"] != manifest.route_id:
        raise RouteValidationError("receipt route mismatch")
    if raw["aggregate_module"] != manifest.aggregate_module or raw["build_target"] != manifest.build_target:
        raise RouteValidationError("receipt aggregate mismatch")
    if raw["manifest_path"] != manifest_path.as_posix():
        raise RouteValidationError("receipt manifest path mismatch")
    if raw["manifest_sha256"] != manifest_contract_sha256(manifest_raw):
        raise RouteValidationError("receipt manifest digest mismatch")
    candidate_commit = _text(raw["candidate_commit"], "candidate commit", pattern=GIT_ID_RE)
    candidate_tree = _text(raw["candidate_tree"], "candidate tree", pattern=GIT_ID_RE)
    _validate_git_identity(
        repo_root, candidate_commit, candidate_tree, manifest.module_closure,
        authority_paths=_ls_authority_paths(repo_root, manifest),
    )
    if raw["receipt_sha256"] != self_digest(raw, "receipt_sha256"):
        raise RouteValidationError("receipt identity digest mismatch")
    final_root = ROUTE_FINAL_ROOT / manifest.route_id
    expected_paths = {
        "command_artifact_path": final_root / "build/command.json",
        "stdout_path": final_root / "build/stdout.log",
        "stderr_path": final_root / "build/stderr.log",
        "axiom_audit_path": final_root / "audit/axioms.json",
        "provider_report_path": final_root / "audit/provider.json",
    }
    for field, path in expected_paths.items():
        if raw[field] != path.as_posix():
            raise RouteValidationError(f"receipt path mismatch: {field}")
    if raw["argv"] != ["scripts/check_lean_library.sh", manifest.build_target]:
        raise RouteValidationError("receipt command mismatch: argv")
    if raw["working_directory"] != ".":
        raise RouteValidationError("working directory must be repository root '.'")
    if raw["toolchain"] != PINNED_TOOLCHAIN:
        raise RouteValidationError("toolchain mismatch")
    if raw["cache_identity"] != cache_contract_identity(repo_root):
        raise RouteValidationError("cache identity mismatch")
    if raw["local_closure_modules"] != list(manifest.module_closure):
        raise RouteValidationError("receipt local closure mismatch")
    if raw["local_closure_sha256"] != manifest.module_closure_sha256:
        raise RouteValidationError("receipt local closure mismatch")
    command_data = _validate_staged_file(members["build/command.json"], raw["command_artifact_sha256"], "command artifact")
    command = _json_from_bytes(command_data, "command artifact")
    if command != {
        "schema_version": COMMAND_SCHEMA_VERSION,
        "argv": raw["argv"],
        "working_directory": ".",
        "aggregate_module": manifest.aggregate_module,
        "build_target": manifest.build_target,
        "cache_identity": raw["cache_identity"],
        "toolchain": PINNED_TOOLCHAIN,
    }:
        raise RouteValidationError("command artifact content mismatch")
    expected_nodes = {node.declaration: node for node in manifest.nodes}
    declarations = set(expected_nodes)
    if raw["allowed_axioms"] != list(manifest.allowed_axioms):
        raise RouteValidationError("receipt allowed axioms mismatch")
    receipt_results = _validate_axiom_results(raw["axiom_results"], declarations, manifest.allowed_axioms)
    audit_data = _validate_staged_file(members["audit/axioms.json"], raw["axiom_audit_sha256"], "axiom audit")
    audit = _json_from_bytes(audit_data, "axiom audit")
    if audit != {
        "schema_version": AXIOM_SCHEMA_VERSION,
        "route_id": manifest.route_id,
        "allowed_axioms": list(manifest.allowed_axioms),
        "results": list(receipt_results),
        "status": "passed",
    }:
        raise RouteValidationError("axiom audit mismatch")
    provider_data = _validate_staged_file(members["audit/provider.json"], raw["provider_report_sha256"], "provider report")
    provider = _json_from_bytes(provider_data, "provider report")
    if provider != {
        "schema_version": "crouzeix-route-provider-report/v1",
        "route_id": manifest.route_id,
        "aggregate_module": manifest.aggregate_module,
        "modules": list(manifest.module_closure),
        "module_closure_sha256": manifest.module_closure_sha256,
        "status": "passed",
    }:
        raise RouteValidationError("provider report content mismatch")
    _validate_staged_file(members["build/stdout.log"], raw["stdout_sha256"], "stdout")
    _validate_staged_file(members["build/stderr.log"], raw["stderr_sha256"], "stderr")
    if raw["status"] != "passed" or raw["exit_code"] != 0:
        raise RouteValidationError("route receipt is not successful")
    if raw["mathlib_artifacts_sha256"] != record_roster_sha256(raw["mathlib_artifacts"]):
        raise RouteValidationError("Mathlib roster digest mismatch")
    artifacts = raw["mathlib_artifacts"]
    if not isinstance(artifacts, list):
        raise RouteValidationError("Mathlib artifacts must be an array")
    expected_mathlib = active_mathlib_closure(repo_root, manifest.module_closure)
    expected_pairs = [
        (module, mathlib_artifact_path(module).as_posix()) for module in expected_mathlib
    ]
    actual_pairs = [
        (item.get("module"), item.get("path"))
        for item in artifacts if isinstance(item, dict)
    ]
    if len(actual_pairs) != len(artifacts) or actual_pairs != expected_pairs:
        raise RouteValidationError("Mathlib artifact roster mismatch")
    cache_root = resolve_approved_cache_root(repo_root)
    for item in artifacts:
        assert isinstance(item, dict)
        _exact_fields(item, frozenset({"module", "path", "sha256"}), "Mathlib artifact")
        data = _read_mathlib_artifact_bytes(cache_root, str(item["module"]))
        if _sha256(data) != item["sha256"]:
            raise RouteValidationError("Mathlib artifact digest mismatch")
    if raw["declaration_types"] != [
        {
            "declaration": node.declaration,
            "type_artifact_path": node.declaration_type_path,
            "type_artifact_sha256": _sha256(_read_bytes(repo_root, node.declaration_type_path, "declaration type artifact")),
            "statement_sha256": node.statement_sha256,
        }
        for node in manifest.nodes
    ]:
        raise RouteValidationError("receipt declaration type roster mismatch")


def _validate_staged_file(data: bytes, expected_digest: object, label: str) -> bytes:
    if _sha256(data) != _text(expected_digest, f"{label} sha256", pattern=SHA256_RE):
        raise RouteValidationError(f"{label} digest mismatch")
    return data


def inspect_route(
    repo_root: Path, route_id: str, *, allow_unpublished: bool = False
) -> RouteValidationResult:
    if route_id not in ROUTE_IDS:
        raise RouteValidationError("invalid route id")
    path = ROUTE_MANIFEST_PATHS[route_id]
    root = Path(repo_root)
    if not (root / path).exists():
        if not allow_unpublished:
            raise RouteValidationError("route manifest is unpublished")
        return RouteValidationResult(
            route_id, "authored", "incomplete", path.as_posix(), "route manifest is unpublished"
        )
    return validate_route_bundle(root, path, allow_unpublished=allow_unpublished)
