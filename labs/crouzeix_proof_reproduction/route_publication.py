from __future__ import annotations

import ctypes
import errno
import hashlib
import json
import os
import platform
import signal
import stat
import subprocess
import tempfile
import threading
import time
import uuid
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Mapping

try:
    from . import execution_ledger, goal_validation, provider_independence, route_validation
except ImportError:  # pragma: no cover - direct script execution path
    import execution_ledger
    import goal_validation
    import provider_independence
    import route_validation


MAX_OUTPUT_BYTES = 1024 * 1024
TIMEOUT_SECONDS = 3600
PIPE_DRAIN_GRACE_SECONDS = 1.0
STAGING_PARENT = route_validation.ROUTE_STAGING_PARENT
REVIEW_CANDIDATE_PARENT = route_validation.ROUTE_REVIEW_CANDIDATE_PARENT
ROUTE_FINAL_ROOT = route_validation.ROUTE_FINAL_ROOT
REVIEW_FINAL_ROOT = route_validation.ROUTE_REVIEW_FINAL_ROOT
RECEIPT_MEMBERS = route_validation.ROUTE_RECEIPT_MEMBERS
DARWIN_RENAME_EXCL = 0x00000004
LINUX_RENAME_NOREPLACE = 0x00000001
LEDGER_PATH = Path("docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv")
ROUTE_PHASES = {
    "jin": "cpfr-085",
    "lorist-schwenninger": "cpfr-086",
    "harp": "cpfr-087",
}
_STAGE_AUTHORITIES: dict[str, "_StageAuthority"] = {}
_LOCK_PARENT_DESCRIPTORS: dict[int, int] = {}


@dataclass(frozen=True)
class RoutePublication:
    route_id: str
    artifact_root: Path
    receipt_path: Path
    receipt_sha256: str


@dataclass(frozen=True)
class ReviewPublication:
    route_id: str
    review_path: Path
    review_sha256: str


@dataclass(frozen=True)
class CommandResult:
    exit_code: int | None
    stdout: bytes
    stderr: bytes
    blocked_reason: str | None = None
    stdout_truncated: bool = False
    stderr_truncated: bool = False


@dataclass(frozen=True)
class _StageAuthority:
    path: Path
    parent_fd: int
    descriptor: int
    device: int
    inode: int


class RoutePublicationError(RuntimeError):
    """A route receipt/review candidate cannot be safely published."""


class CommittedPublicationError(RoutePublicationError):
    """Final bytes are visible but post-commit validation or durability failed."""

    def __init__(self, path: Path, digest: str, reason: BaseException) -> None:
        self.path = path
        self.digest = digest
        self.reason = reason
        super().__init__(
            f"route publication committed at {path} sha256={digest}, "
            f"but post-commit validation or durability failed: {reason}"
        )


Executor = Callable[[list[str], Path, dict[str, str], int, int], CommandResult]


def publish_route_receipt(
    repository_root: Path, route_id: str, *, executor: Executor = None
) -> RoutePublication:
    root = Path(repository_root).resolve(strict=True)
    manifest_path = _manifest_path(route_id)
    manifest_raw = route_validation._read_json(root, manifest_path, "route manifest")
    manifest = route_validation.parse_route_manifest(manifest_raw)
    if manifest.route_id != route_id:
        raise RoutePublicationError("route manifest route mismatch")
    _require_unpublished(root, manifest)
    _preflight_ready(root, route_id)
    route_validation.validate_route_bundle(root, manifest_path, allow_unpublished=True)

    lock_fd = _acquire_lock(root, f"receipt-{route_id}")
    stage: Path | None = None
    try:
        _require_unpublished(root, manifest)
        _preflight_ready(root, route_id)
        route_validation.validate_route_bundle(root, manifest_path, allow_unpublished=True)
        stage = _create_stage(root, route_id)
        before = _input_snapshot(root, manifest)
        candidate_commit, candidate_tree = _candidate_identity(root, manifest)
        run = executor or _subprocess_executor
        environment = _shared_environment()
        result = _run_checked(
            run,
            ["scripts/check_lean_library.sh", manifest.build_target],
            root,
            environment,
            "route build",
        )
        if _input_snapshot(root, manifest) != before:
            raise RoutePublicationError("route input/cache mutation detected after build")
        _write_receipt_candidate(
            root,
            manifest_path,
            manifest_raw,
            manifest,
            stage,
            result,
            run,
            candidate_commit,
            candidate_tree,
        )
        route_validation.validate_route_receipt_candidate(root, manifest_path, stage)
        _fsync_tree(root, stage.relative_to(root))
        if _input_snapshot(root, manifest) != before:
            raise RoutePublicationError("route input/cache mutation detected before publication")
        final_root = root / ROUTE_FINAL_ROOT / route_id
        _rename_no_replace(root, stage, final_root)
        stage = None
        digest = _sha256(
            route_validation._read_bytes(
                root, manifest.receipt_path, "published route receipt"
            )
        )
        try:
            _fsync_tree(root, final_root.relative_to(root))
            _fsync_directory(final_root.parent)
            _validate_published_receipt(root, manifest_path, manifest, digest)
        except BaseException as error:
            raise CommittedPublicationError(final_root / "receipt.json", digest, error) from error
        return RoutePublication(
            route_id=route_id,
            artifact_root=ROUTE_FINAL_ROOT / route_id,
            receipt_path=ROUTE_FINAL_ROOT / route_id / "receipt.json",
            receipt_sha256=digest,
        )
    finally:
        if stage is not None:
            _remove_owned_stage(root, stage)
        _release_lock(lock_fd)


def publish_route_review(repository_root: Path, route_id: str) -> ReviewPublication:
    root = Path(repository_root).resolve(strict=True)
    manifest_path = _manifest_path(route_id)
    manifest_raw = route_validation._read_json(root, manifest_path, "route manifest")
    manifest = route_validation.parse_route_manifest(manifest_raw)
    if manifest.route_id != route_id:
        raise RoutePublicationError("route manifest route mismatch")
    receipt_path = root / manifest.receipt_path
    if not receipt_path.exists() or manifest.receipt_sha256 is None:
        raise RoutePublicationError("route receipt must be published before review")
    receipt_sha = _sha256(route_validation._read_bytes(root, manifest.receipt_path, "route receipt"))
    if receipt_sha != manifest.receipt_sha256:
        raise RoutePublicationError("published route receipt digest mismatch")
    receipt = route_validation._validate_receipt(root, manifest_path, manifest_raw, manifest)
    candidate_path = _review_candidate_path(root, route_id)
    review = _read_review_candidate(root, candidate_path, route_id)
    _validate_review_candidate(manifest_path, manifest_raw, manifest, receipt, review)

    lock_fd = _acquire_lock(root, f"review-{route_id}")
    try:
        _require_absent(root / manifest.review_path, "route review")
        final = root / REVIEW_FINAL_ROOT / f"{route_id}.json"
        if final.relative_to(root).as_posix() != manifest.review_path:
            raise RoutePublicationError("route review final path mismatch")
        review_bytes = route_validation.canonical_json_bytes(review)
        parent_fd = _open_or_create_rooted_directory(root, REVIEW_FINAL_ROOT)
        hidden_name = f".{route_id}-review-{uuid.uuid4().hex}.tmp"
        digest: str | None = None
        committed = False
        try:
            digest = _write_bytes_at(
                parent_fd, hidden_name, review_bytes, "staged route review"
            )
            os.fsync(parent_fd)
            _rename_file_no_replace_at(parent_fd, hidden_name, final.name)
            committed = True
            os.fsync(parent_fd)
            observed = route_validation._read_json(root, manifest.review_path, "proof review")
            _validate_review_candidate(
                manifest_path, manifest_raw, manifest, receipt, observed
            )
            observed_bytes = route_validation._read_bytes(
                root, manifest.review_path, "published proof review"
            )
            if observed != review or _sha256(observed_bytes) != digest:
                raise RoutePublicationError("route review changed after publication")
        except BaseException as error:
            if committed and digest is not None:
                raise CommittedPublicationError(final, digest, error) from error
            try:
                os.unlink(hidden_name, dir_fd=parent_fd)
            except FileNotFoundError:
                pass
            raise
        finally:
            os.close(parent_fd)
        return ReviewPublication(
            route_id=route_id,
            review_path=REVIEW_FINAL_ROOT / f"{route_id}.json",
            review_sha256=digest,
        )
    finally:
        _release_lock(lock_fd)


def _manifest_path(route_id: str) -> Path:
    if route_id not in route_validation.ROUTE_IDS:
        raise RoutePublicationError("invalid route id")
    return route_validation.ROUTE_MANIFEST_PATHS[route_id]


def _require_unpublished(root: Path, manifest: route_validation.RouteManifest) -> None:
    if manifest.receipt_sha256 is not None or manifest.review_sha256 is not None:
        raise RoutePublicationError("route manifest is already published")
    _require_absent(root / manifest.receipt_path, "route receipt")
    _require_absent(root / manifest.review_path, "route review")


def _require_absent(path: Path, label: str) -> None:
    try:
        path.lstat()
    except FileNotFoundError:
        return
    raise RoutePublicationError(f"{label} already exists: {path}")


def _preflight_ready(root: Path, route_id: str) -> None:
    try:
        route_validation.cache_contract_identity(root)
        result = route_validation.inspect_route(root, route_id, allow_unpublished=True)
    except Exception as error:
        raise RoutePublicationError(f"route preflight blocked: {error}") from error
    if result.route_id != route_id or result.status not in {"incomplete", "complete"}:
        raise RoutePublicationError(f"route preflight blocked: {result.reason or result.status}")


def _shared_environment() -> dict[str, str]:
    elan_home = Path("/private/tmp/harp-mathematical-foundations-elan")
    toolchain_bin = elan_home / "toolchains/leanprover--lean4---v4.32.1/bin"
    return {
        "ELAN_HOME": elan_home.as_posix(),
        "ELAN_TOOLCHAIN": route_validation.PINNED_TOOLCHAIN,
        "PATH": os.pathsep.join((toolchain_bin.as_posix(), "/usr/bin", "/bin")),
    }


def _run_checked(
    executor: Executor,
    argv: list[str],
    cwd: Path,
    environment: dict[str, str],
    label: str,
) -> CommandResult:
    result = executor(argv, cwd, dict(environment), TIMEOUT_SECONDS, MAX_OUTPUT_BYTES)
    if not isinstance(result, CommandResult):
        if all(hasattr(result, attr) for attr in ("exit_code", "stdout", "stderr")):
            result = CommandResult(
                result.exit_code,
                result.stdout,
                result.stderr,
                getattr(result, "blocked_reason", None),
                getattr(result, "stdout_truncated", False),
                getattr(result, "stderr_truncated", False),
            )
        else:
            raise RoutePublicationError("route executor must return CommandResult")
    if not isinstance(result.stdout, bytes) or not isinstance(result.stderr, bytes):
        raise RoutePublicationError("route executor output must be bytes")
    if (
        result.stdout_truncated
        or result.stderr_truncated
        or len(result.stdout) > MAX_OUTPUT_BYTES
        or len(result.stderr) > MAX_OUTPUT_BYTES
    ):
        raise RoutePublicationError(f"{label} exceeded output cap")
    if result.blocked_reason is not None or result.exit_code is None:
        raise RoutePublicationError(f"{label} was blocked: {result.blocked_reason or 'no exit code'}")
    if result.exit_code != 0:
        raise RoutePublicationError(f"{label} failed with exit code {result.exit_code}")
    return result


def _write_receipt_candidate(
    root: Path,
    manifest_path: Path,
    manifest_raw: Mapping[str, object],
    manifest: route_validation.RouteManifest,
    stage: Path,
    result: CommandResult,
    executor: Executor,
    candidate_commit: str,
    candidate_tree: str,
) -> None:
    authority = _stage_authority(stage)
    _mkdir_at(authority.descriptor, "build", "route build directory")
    _mkdir_at(authority.descriptor, "audit", "route audit directory")
    final_root = ROUTE_FINAL_ROOT / manifest.route_id
    command = {
        "schema_version": route_validation.COMMAND_SCHEMA_VERSION,
        "argv": ["scripts/check_lean_library.sh", manifest.build_target],
        "working_directory": ".",
        "aggregate_module": manifest.aggregate_module,
        "build_target": manifest.build_target,
        "cache_identity": route_validation.cache_contract_identity(root),
        "toolchain": route_validation.PINNED_TOOLCHAIN,
    }
    command_sha = _write_stage_json(stage, Path("build/command.json"), command)
    stdout_sha = _write_stage_bytes(stage, Path("build/stdout.log"), result.stdout)
    stderr_sha = _write_stage_bytes(stage, Path("build/stderr.log"), result.stderr)
    axiom_results = _run_axiom_audits(root, manifest, executor, _shared_environment())
    axiom = {
        "schema_version": route_validation.AXIOM_SCHEMA_VERSION,
        "route_id": manifest.route_id,
        "allowed_axioms": list(manifest.allowed_axioms),
        "results": axiom_results,
        "status": "passed",
    }
    axiom_sha = _write_stage_json(stage, Path("audit/axioms.json"), axiom)
    provider = _compute_provider_report(root, manifest)
    provider_sha = _write_stage_json(stage, Path("audit/provider.json"), provider)
    mathlib_artifacts = _mathlib_artifacts(root, manifest)
    declaration_types = [
        {
            "declaration": node.declaration,
            "type_artifact_path": node.declaration_type_path,
            "type_artifact_sha256": _sha256(route_validation._read_bytes(root, node.declaration_type_path, "declaration type artifact")),
            "statement_sha256": node.statement_sha256,
        }
        for node in manifest.nodes
    ]
    receipt: dict[str, object] = {
        "schema_version": route_validation.RECEIPT_SCHEMA_VERSION,
        "route_id": manifest.route_id,
        "aggregate_module": manifest.aggregate_module,
        "build_target": manifest.build_target,
        "manifest_path": manifest_path.as_posix(),
        "manifest_sha256": route_validation.manifest_contract_sha256(manifest_raw),
        "candidate_commit": candidate_commit,
        "candidate_tree": candidate_tree,
        "command_artifact_path": (final_root / "build/command.json").as_posix(),
        "command_artifact_sha256": command_sha,
        "argv": ["scripts/check_lean_library.sh", manifest.build_target],
        "working_directory": ".",
        "cache_identity": command["cache_identity"],
        "toolchain": route_validation.PINNED_TOOLCHAIN,
        "local_closure_modules": list(manifest.module_closure),
        "local_closure_sha256": manifest.module_closure_sha256,
        "mathlib_artifacts": mathlib_artifacts,
        "mathlib_artifacts_sha256": route_validation.record_roster_sha256(mathlib_artifacts),
        "declaration_types": declaration_types,
        "allowed_axioms": list(manifest.allowed_axioms),
        "axiom_audit_path": (final_root / "audit/axioms.json").as_posix(),
        "axiom_audit_sha256": axiom_sha,
        "axiom_results": axiom_results,
        "provider_report_path": (final_root / "audit/provider.json").as_posix(),
        "provider_report_sha256": provider_sha,
        "stdout_path": (final_root / "build/stdout.log").as_posix(),
        "stdout_sha256": stdout_sha,
        "stderr_path": (final_root / "build/stderr.log").as_posix(),
        "stderr_sha256": stderr_sha,
        "exit_code": 0,
        "status": "passed",
        "receipt_sha256": "0" * 64,
    }
    receipt["receipt_sha256"] = route_validation.self_digest(receipt, "receipt_sha256")
    _write_stage_json(stage, Path("receipt.json"), receipt)


def _run_axiom_audits(
    root: Path,
    manifest: route_validation.RouteManifest,
    executor: Executor,
    environment: dict[str, str],
) -> list[dict[str, object]]:
    results: list[dict[str, object]] = []
    for node in sorted(manifest.nodes, key=lambda item: item.declaration):
        module = Path(node.module_path).with_suffix("").as_posix().replace("/", ".")
        source = f"import {module}\n#print axioms {node.declaration}\n".encode("utf-8")
        lean_root = root / "formalization/lean"
        with tempfile.TemporaryDirectory(
            prefix=".route-axiom-audit-", dir=lean_root
        ) as directory:
            audit_path = Path(directory) / "Audit.lean"
            _write_bytes(audit_path, source)
            relative = audit_path.relative_to(lean_root).as_posix()
            audit = _run_checked(
                executor,
                ["lake", "env", "lean", relative],
                lean_root,
                environment,
                "axiom audit",
            )
        observed = _parse_axiom_output(audit.stdout, node.declaration)
        forbidden = sorted(set(observed) - set(manifest.allowed_axioms))
        if forbidden:
            raise RoutePublicationError(f"forbidden axiom: {forbidden[0]}")
        results.append({"declaration": node.declaration, "axioms": list(observed)})
    return results


def _parse_axiom_output(data: bytes, declaration: str) -> tuple[str, ...]:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise RoutePublicationError("axiom audit output is not UTF-8") from error
    prefix = f"'{declaration}' depends on axioms: ["
    if text.startswith(prefix) and text.rstrip().endswith("]"):
        raw = text[len(prefix): text.rfind("]")]
        axioms = tuple(sorted(item.strip() for item in raw.split(",") if item.strip()))
    elif text.strip() == f"'{declaration}' does not depend on any axioms":
        axioms = ()
    else:
        raise RoutePublicationError("axiom audit output is malformed")
    if len(axioms) != len(set(axioms)):
        raise RoutePublicationError("duplicate axiom audit entry")
    return axioms


def _compute_provider_report(
    root: Path, manifest: route_validation.RouteManifest
) -> dict[str, object]:
    forbidden, prefixes = route_validation._provider_policy(manifest)
    try:
        report = provider_independence.audit_provider_independence(
            root / "formalization/lean",
            (manifest.aggregate_module,),
            forbidden,
            forbidden_prefixes=prefixes,
        )
    except provider_independence.ProviderIndependenceError as error:
        raise RoutePublicationError(f"provider policy violation: {error}") from error
    if report.modules != manifest.module_closure:
        raise RoutePublicationError("observed provider closure does not match manifest")
    return {
        "schema_version": "crouzeix-route-provider-report/v1",
        "route_id": manifest.route_id,
        "aggregate_module": manifest.aggregate_module,
        "modules": list(report.modules),
        "module_closure_sha256": route_validation.string_roster_sha256(
            report.modules
        ),
        "status": "passed",
    }


def _mathlib_artifacts(root: Path, manifest: route_validation.RouteManifest) -> list[dict[str, object]]:
    modules = route_validation.active_mathlib_closure(root, manifest.module_closure)
    cache_root = route_validation.resolve_approved_cache_root(root)
    artifacts: list[dict[str, object]] = []
    for module in modules:
        relative = route_validation.mathlib_artifact_path(module)
        data = route_validation._read_cache_bytes(
            cache_root,
            Path("packages/mathlib/.lake/build/lib/lean") / Path(*module.split(".")).with_suffix(".olean"),
            "Mathlib artifact",
        )
        artifacts.append({"module": module, "path": relative.as_posix(), "sha256": _sha256(data)})
    return artifacts


def _validate_published_receipt(
    root: Path,
    manifest_path: Path,
    manifest: route_validation.RouteManifest,
    digest: str,
) -> None:
    final_root = root / ROUTE_FINAL_ROOT / manifest.route_id
    route_validation._validate_published_route_receipt(
        root, manifest_path, final_root
    )
    observed = route_validation._read_bytes(
        root, manifest.receipt_path, "published route receipt"
    )
    if _sha256(observed) != digest:
        raise RoutePublicationError("published receipt bytes changed")


def _review_candidate_path(root: Path, route_id: str) -> Path:
    if route_id not in route_validation.ROUTE_IDS:
        raise RoutePublicationError("invalid route id")
    return root / REVIEW_CANDIDATE_PARENT / f"{route_id}.json"


def _read_review_candidate(root: Path, path: Path, route_id: str) -> dict[str, object]:
    expected = REVIEW_CANDIDATE_PARENT / f"{route_id}.json"
    try:
        relative = path.relative_to(root)
    except ValueError as error:
        raise RoutePublicationError("route review candidate is outside fixed parent") from error
    if relative != expected:
        raise RoutePublicationError("route review candidate path mismatch")
    try:
        data = route_validation._read_rooted_bytes(
            root, expected.as_posix(), "route review candidate"
        )
        value = json.loads(data.decode("utf-8"), object_pairs_hook=route_validation._pairs_no_duplicates)
    except route_validation.RouteValidationError as error:
        message = str(error)
        if "duplicate JSON key" in message:
            raise RoutePublicationError("route review candidate is invalid JSON") from error
        if "hardlink" in message:
            raise RoutePublicationError("route review candidate cannot be a hardlink alias") from error
        if "byte bound" in message:
            raise RoutePublicationError("route review candidate exceeds byte bound") from error
        if "symlink" in message or "without following links" in message:
            raise RoutePublicationError("route review candidate path contains a symlink") from error
        raise RoutePublicationError(f"route review candidate is invalid: {error}") from error
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        raise RoutePublicationError("route review candidate is invalid JSON") from error
    if not isinstance(value, dict):
        raise RoutePublicationError("route review candidate must be an object")
    return value


def _validate_review_candidate(
    manifest_path: Path,
    manifest_raw: Mapping[str, object],
    manifest: route_validation.RouteManifest,
    receipt: Mapping[str, object],
    review: dict[str, object],
) -> None:
    try:
        route_validation._validate_review_payload(
            manifest_path, manifest_raw, manifest, receipt, review
        )
    except route_validation.RouteValidationError as error:
        message = str(error)
        aliases = {
            "review route mismatch": "review route_id mismatch",
            "review commit mismatch": "review reviewed_commit mismatch",
            "review manifest digest mismatch": "review manifest_sha256 mismatch",
            "review terminal type mismatch": "review terminal_type_sha256 mismatch",
            "complete review has unresolved Critical/Important finding":
                "review has unresolved Critical/Important finding",
        }
        raise RoutePublicationError(aliases.get(message, message)) from error


def _create_stage(root: Path, route_id: str) -> Path:
    parent = root / STAGING_PARENT
    parent_fd = _open_or_create_rooted_directory(root, STAGING_PARENT)
    name = f".{route_id}-{uuid.uuid4().hex}"
    path = parent / name
    try:
        os.mkdir(name, mode=0o700, dir_fd=parent_fd)
        descriptor = route_validation._open_directory_component(
            parent_fd, name, "route stage"
        )
    except BaseException:
        os.close(parent_fd)
        raise
    metadata = os.fstat(descriptor)
    _STAGE_AUTHORITIES[path.as_posix()] = _StageAuthority(
        path, parent_fd, descriptor, metadata.st_dev, metadata.st_ino
    )
    return path


def _remove_owned_stage(root: Path, stage: Path) -> None:
    authority = _STAGE_AUTHORITIES.pop(stage.as_posix(), None)
    if authority is None or stage.parent != root / STAGING_PARENT or not stage.name.startswith("."):
        raise RoutePublicationError("refusing to remove an unbound route stage")
    try:
        metadata = os.stat(
            stage.name, dir_fd=authority.parent_fd, follow_symlinks=False
        )
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISDIR(metadata.st_mode) or (metadata.st_dev, metadata.st_ino) != (authority.device, authority.inode):
            raise RoutePublicationError("route stage identity changed before cleanup")
        _remove_fixed_receipt_contents(authority.descriptor)
        current = os.stat(
            stage.name, dir_fd=authority.parent_fd, follow_symlinks=False
        )
        if (current.st_dev, current.st_ino) != (authority.device, authority.inode):
            raise RoutePublicationError("route stage identity changed before removal")
        os.rmdir(stage.name, dir_fd=authority.parent_fd)
    except FileNotFoundError:
        return
    finally:
        os.close(authority.descriptor)
        os.close(authority.parent_fd)


def _stage_authority(stage: Path) -> _StageAuthority:
    authority = _STAGE_AUTHORITIES.get(stage.as_posix())
    if authority is None:
        raise RoutePublicationError("route stage identity is unbound")
    metadata = os.fstat(authority.descriptor)
    if (metadata.st_dev, metadata.st_ino) != (authority.device, authority.inode):
        raise RoutePublicationError("route stage descriptor identity changed")
    return authority


def _mkdir_at(parent_fd: int, name: str, label: str) -> None:
    try:
        os.mkdir(name, mode=0o700, dir_fd=parent_fd)
    except FileExistsError as error:
        raise RoutePublicationError(f"{label} already exists") from error


def _write_stage_bytes(stage: Path, relative: Path, data: bytes) -> str:
    authority = _stage_authority(stage)
    if len(relative.parts) == 1:
        return _write_bytes_at(
            authority.descriptor, relative.name, data, "route receipt member"
        )
    if len(relative.parts) != 2 or relative.parts[0] not in {"build", "audit"}:
        raise RoutePublicationError("unsafe route stage member path")
    child_fd = route_validation._open_directory_component(
        authority.descriptor, relative.parts[0], "route stage directory"
    )
    try:
        return _write_bytes_at(
            child_fd, relative.name, data, "route receipt member"
        )
    finally:
        os.close(child_fd)


def _write_stage_json(stage: Path, relative: Path, value: object) -> str:
    return _write_stage_bytes(
        stage, relative, route_validation.canonical_json_bytes(value)
    )


def _remove_fixed_receipt_contents(stage_fd: int) -> None:
    names = set(os.listdir(stage_fd))
    if not names <= {"receipt.json", "build", "audit"}:
        raise RoutePublicationError("route stage contains an unknown cleanup entry")
    if "receipt.json" in names:
        os.unlink("receipt.json", dir_fd=stage_fd)
    for directory, allowed in (
        ("build", {"command.json", "stdout.log", "stderr.log"}),
        ("audit", {"axioms.json", "provider.json"}),
    ):
        if directory not in names:
            continue
        child_fd = route_validation._open_directory_component(stage_fd, directory, f"route stage {directory}")
        try:
            child_names = set(os.listdir(child_fd))
            if not child_names <= allowed:
                raise RoutePublicationError("route stage contains an unknown cleanup entry")
            for name in child_names:
                os.unlink(name, dir_fd=child_fd)
        finally:
            os.close(child_fd)
        os.rmdir(directory, dir_fd=stage_fd)


def _input_snapshot(root: Path, manifest: route_validation.RouteManifest) -> tuple[tuple[str, str], ...]:
    paths: set[Path] = {
        Path("scripts/check_lean_library.sh"),
        Path("formalization/lean/lean-toolchain"),
        Path("formalization/lean/lake-manifest.json"),
        Path("formalization/lean/lakefile.toml"),
        LEDGER_PATH,
    }
    if manifest.route_id == "jin":
        paths.add(Path("labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/artifact-manifest.json"))
    for module in manifest.module_closure:
        paths.add(Path("formalization/lean") / route_validation.module_relative_path(module))
    for node in manifest.nodes:
        paths.add(Path(node.declaration_type_path))
    cache_root = route_validation.resolve_approved_cache_root(root)
    records: list[tuple[str, str]] = []
    for relative in sorted(paths):
        records.append((relative.as_posix(), _sha256(route_validation._read_bytes(root, relative.as_posix(), "route input"))))
    for module in route_validation.active_mathlib_closure(root, manifest.module_closure):
        artifact = Path("packages/mathlib/.lake/build/lib/lean") / Path(*module.split(".")).with_suffix(".olean")
        records.append((f"cache/{artifact.as_posix()}", _sha256(route_validation._read_cache_bytes(cache_root, artifact, "route cache input"))))
    return tuple(records)


def _candidate_identity(root: Path, manifest: route_validation.RouteManifest) -> tuple[str, str]:
    phase = ROUTE_PHASES[manifest.route_id]
    ledger = goal_validation._load_execution_ledger_checked(root / LEDGER_PATH, repository_root=root)
    candidate_row: execution_ledger.ExecutionLedgerRow | None = None
    for row in reversed(ledger.rows):
        if row.phase != phase:
            continue
        if execution_ledger.STATE_INDEX[row.state] < execution_ledger.STATE_INDEX["locally-verified"]:
            continue
        candidate_row = row
        break
    if candidate_row is None:
        raise RoutePublicationError(f"route candidate identity is missing for {phase}")
    candidate_commit = candidate_row.candidate_commit
    with goal_validation.git_session() as session:
        candidate_tree = _git_stdout(
            root, "rev-parse", f"{candidate_commit}^{{tree}}", session=session
        ).strip()
        _require_git_ancestor(root, candidate_commit, "HEAD", session=session)
        _require_route_inputs_at_commit(
            root, manifest, candidate_commit, session=session
        )
    recorded_tree = _tree_from_ledger_note(candidate_row.note)
    if recorded_tree is not None and recorded_tree != candidate_tree:
        raise RoutePublicationError("execution ledger candidate tree mismatch")
    return candidate_commit, candidate_tree


def _tree_from_ledger_note(note: str) -> str | None:
    for item in note.split(";"):
        if item.startswith("tree="):
            value = item.removeprefix("tree=")
            if route_validation.GIT_ID_RE.fullmatch(value) is None:
                raise RoutePublicationError("execution ledger candidate tree is invalid")
            return value
    return None


def _git_stdout(
    root: Path, *args: str, session: goal_validation.GitSession | None = None
) -> str:
    if session is None:
        with goal_validation.git_session() as owned_session:
            return _git_stdout(root, *args, session=owned_session)
    result = goal_validation._run_git_bounded(root, session, *args)
    if result.returncode != 0:
        stderr = result.stderr.strip() or "unknown git error"
        raise RoutePublicationError(f"git command failed: {stderr}")
    return result.stdout


def _require_git_ancestor(
    root: Path,
    candidate_commit: str,
    head_ref: str,
    *,
    session: goal_validation.GitSession,
) -> None:
    result = goal_validation._run_git_bounded(
        root, session, "merge-base", "--is-ancestor", candidate_commit, head_ref
    )
    if result.returncode != 0:
        raise RoutePublicationError(f"candidate commit is not an ancestor of {head_ref}")


def _require_route_inputs_at_commit(
    root: Path,
    manifest: route_validation.RouteManifest,
    candidate_commit: str,
    *,
    session: goal_validation.GitSession,
) -> None:
    required: list[Path] = [_manifest_path(manifest.route_id), Path("scripts/check_lean_library.sh"), Path("formalization/lean/lake-manifest.json"), Path("formalization/lean/lakefile.toml"), Path("formalization/lean/lean-toolchain")]
    if manifest.route_id == "jin":
        required.append(Path("labs/crouzeix_proof_reproduction/formal_targets/jin-565b6a3/artifact-manifest.json"))
    required.extend(Path(node.declaration_type_path) for node in manifest.nodes)
    required.extend(Path("formalization/lean") / route_validation.module_relative_path(module) for module in manifest.module_closure)
    _require_paths_at_commit(root, candidate_commit, required, session=session)


def _require_paths_at_commit(
    root: Path,
    commit: str,
    paths: list[Path],
    *,
    session: goal_validation.GitSession,
) -> None:
    for path in paths:
        relative = path.as_posix()
        current = route_validation._read_bytes(
            root, relative, "route authority input"
        )
        expected_blob = _git_blob_oid(current)
        observed_blob = _git_stdout(
            root, "rev-parse", f"{commit}:{relative}", session=session
        ).strip()
        if observed_blob != expected_blob:
            raise RoutePublicationError(
                f"route authority input drifted since candidate commit: {relative}"
            )


def _git_blob_oid(data: bytes) -> str:
    header = f"blob {len(data)}\0".encode("ascii")
    return hashlib.sha1(header + data).hexdigest()


def _acquire_lock(root: Path, name: str) -> int:
    lock_parent = Path(".build/crouzeix-route-publication-locks")
    parent_fd = _open_or_create_rooted_directory(root, lock_parent)
    lock_name = f"{name}.lock"
    flags = (
        os.O_RDWR
        | os.O_CREAT
        | getattr(os, "O_NOFOLLOW", 0)
        | getattr(os, "O_CLOEXEC", 0)
    )
    try:
        fd = os.open(lock_name, flags, 0o600, dir_fd=parent_fd)
    except BaseException:
        os.close(parent_fd)
        raise
    try:
        import fcntl

        metadata = os.fstat(fd)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1 or metadata.st_uid != os.geteuid():
            raise RoutePublicationError("route publication lock is unsafe")
        fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError as error:
        os.close(fd)
        os.close(parent_fd)
        raise RoutePublicationError("route publication is already locked") from error
    except OSError as error:
        os.close(fd)
        os.close(parent_fd)
        if error.errno in {errno.EACCES, errno.EAGAIN}:
            raise RoutePublicationError("route publication is already locked") from error
        raise
    except BaseException:
        os.close(fd)
        os.close(parent_fd)
        raise
    _LOCK_PARENT_DESCRIPTORS[fd] = parent_fd
    return fd


def _release_lock(descriptor: int) -> None:
    parent_fd = _LOCK_PARENT_DESCRIPTORS.pop(descriptor, None)
    try:
        os.close(descriptor)
    finally:
        if parent_fd is not None:
            os.close(parent_fd)


def _rename_no_replace(root: Path, source: Path, destination: Path) -> None:
    source_parent_fd = _open_rooted_directory(root, source.parent.relative_to(root))
    destination_parent_fd = _open_rooted_directory(root, destination.parent.relative_to(root))
    authority = _STAGE_AUTHORITIES.get(source.as_posix())
    if authority is None:
        os.close(source_parent_fd)
        os.close(destination_parent_fd)
        raise RoutePublicationError("route stage identity is unbound")
    try:
        metadata = os.stat(
            source.name, dir_fd=source_parent_fd, follow_symlinks=False
        )
        if (
            stat.S_ISLNK(metadata.st_mode)
            or not stat.S_ISDIR(metadata.st_mode)
            or (metadata.st_dev, metadata.st_ino)
            != (authority.device, authority.inode)
        ):
            raise RoutePublicationError(
                "route stage identity changed before publication"
            )
        _rename_at_no_replace(
            source_parent_fd,
            source.name,
            destination_parent_fd,
            destination.name,
            "route publication",
        )
        _STAGE_AUTHORITIES.pop(source.as_posix(), None)
        os.close(authority.descriptor)
        os.close(authority.parent_fd)
    finally:
        os.close(source_parent_fd)
        os.close(destination_parent_fd)


def _rename_file_no_replace_at(
    directory_fd: int, source_name: str, destination_name: str
) -> None:
    _rename_at_no_replace(
        directory_fd, source_name, directory_fd, destination_name, "route review"
    )


def _rename_at_no_replace(
    source_fd: int,
    source_name: str,
    destination_fd: int,
    destination_name: str,
    label: str,
) -> None:
    system = platform.system()
    library = ctypes.CDLL(None, use_errno=True)
    if system == "Darwin":
        rename = getattr(library, "renameatx_np", None)
        if rename is None:
            raise RoutePublicationError(
                "host Darwin runtime lacks atomic no-replace publication"
            )
        rename.argtypes = (
            ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p,
            ctypes.c_uint,
        )
        rename.restype = ctypes.c_int
        result = rename(
            source_fd, os.fsencode(source_name),
            destination_fd, os.fsencode(destination_name), DARWIN_RENAME_EXCL,
        )
    elif system == "Linux":
        rename = getattr(library, "renameat2", None)
        if rename is None:
            raise RoutePublicationError(
                "host Linux runtime lacks atomic no-replace publication"
            )
        rename.argtypes = (
            ctypes.c_int, ctypes.c_char_p, ctypes.c_int, ctypes.c_char_p,
            ctypes.c_uint,
        )
        rename.restype = ctypes.c_int
        result = rename(
            source_fd, os.fsencode(source_name),
            destination_fd, os.fsencode(destination_name), LINUX_RENAME_NOREPLACE,
        )
    else:
        raise RoutePublicationError(
            f"unsupported host for no-replace publication: {system}"
        )
    if result != 0:
        error = ctypes.get_errno()
        if error == errno.EEXIST:
            raise RoutePublicationError(f"{label} already exists")
        raise RoutePublicationError(
            f"{label} rename failed: {os.strerror(error)}"
        )


def _reject_unsafe_ancestors(root: Path, path: Path) -> None:
    root = root.resolve(strict=True)
    probe = root
    for part in path.resolve(strict=False).relative_to(root).parts:
        probe = probe / part
        try:
            metadata = probe.lstat()
        except FileNotFoundError:
            continue
        if stat.S_ISLNK(metadata.st_mode):
            raise RoutePublicationError(f"publication path contains symlink ancestor: {probe}")


def _write_bytes(path: Path, data: bytes) -> str:
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_CLOEXEC", 0)
    fd = os.open(path, flags, 0o600)
    try:
        os.write(fd, data)
        os.fsync(fd)
    finally:
        os.close(fd)
    return _sha256(data)


def _write_bytes_at(
    directory_fd: int, name: str, data: bytes, label: str
) -> str:
    if not name or name in {".", ".."} or "/" in name or "\\" in name:
        raise RoutePublicationError(f"{label} has an unsafe file name")
    flags = (
        os.O_WRONLY
        | os.O_CREAT
        | os.O_EXCL
        | getattr(os, "O_NOFOLLOW", 0)
        | getattr(os, "O_CLOEXEC", 0)
    )
    try:
        descriptor = os.open(name, flags, 0o600, dir_fd=directory_fd)
    except FileExistsError as error:
        raise RoutePublicationError(f"{label} already exists") from error
    except OSError as error:
        raise RoutePublicationError(f"cannot create {label}: {error}") from error
    try:
        view = memoryview(data)
        while view:
            written = os.write(descriptor, view)
            if written <= 0:
                raise OSError("short write")
            view = view[written:]
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
            raise RoutePublicationError(f"{label} is not a safe regular file")
        os.fsync(descriptor)
    finally:
        os.close(descriptor)
    return _sha256(data)


def _write_json(path: Path, value: object) -> str:
    return _write_bytes(path, route_validation.canonical_json_bytes(value))


def _write_json_create_only(path: Path, value: object) -> str:
    _require_absent(path, "route review")
    return _write_json(path, value)


def _fsync_file(path: Path) -> None:
    fd = os.open(path, os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0))
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def _fsync_directory(path: Path) -> None:
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | getattr(os, "O_CLOEXEC", 0))
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def _open_rooted_directory(root: Path, relative: Path) -> int:
    flags = os.O_RDONLY | os.O_DIRECTORY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0)
    descriptor = os.open(root, flags)
    try:
        for part in relative.parts:
            next_fd = route_validation._open_directory_component(descriptor, part, "route publication directory")
            os.close(descriptor)
            descriptor = next_fd
        return descriptor
    except BaseException:
        os.close(descriptor)
        raise


def _open_or_create_rooted_directory(root: Path, relative: Path) -> int:
    flags = os.O_RDONLY | os.O_DIRECTORY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0)
    descriptor = os.open(root, flags)
    try:
        for part in relative.parts:
            try:
                next_fd = route_validation._open_directory_component(
                    descriptor, part, "route publication directory"
                )
            except route_validation.RouteValidationError:
                try:
                    metadata = os.stat(part, dir_fd=descriptor, follow_symlinks=False)
                except FileNotFoundError:
                    os.mkdir(part, mode=0o700, dir_fd=descriptor)
                    next_fd = route_validation._open_directory_component(
                        descriptor, part, "route publication directory"
                    )
                else:
                    if stat.S_ISLNK(metadata.st_mode):
                        raise RoutePublicationError(
                            "route publication directory contains a symlink"
                        )
                    raise
            os.close(descriptor)
            descriptor = next_fd
        return descriptor
    except BaseException:
        os.close(descriptor)
        raise


def _fsync_file_at(directory_fd: int, name: str) -> None:
    flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0) | getattr(os, "O_CLOEXEC", 0)
    descriptor = os.open(name, flags, dir_fd=directory_fd)
    try:
        metadata = os.fstat(descriptor)
        if not stat.S_ISREG(metadata.st_mode) or metadata.st_nlink != 1:
            raise RoutePublicationError("route publication member is unsafe")
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def _fsync_tree(root: Path, relative: Path) -> None:
    route_validation._read_exact_receipt_tree(root, relative)
    stage_fd = _open_rooted_directory(root, relative)
    build_fd = audit_fd = None
    try:
        build_fd = route_validation._open_directory_component(stage_fd, "build", "route build directory")
        audit_fd = route_validation._open_directory_component(stage_fd, "audit", "route audit directory")
        _fsync_file_at(stage_fd, "receipt.json")
        for name in ("command.json", "stdout.log", "stderr.log"):
            _fsync_file_at(build_fd, name)
        for name in ("axioms.json", "provider.json"):
            _fsync_file_at(audit_fd, name)
        os.fsync(build_fd)
        os.fsync(audit_fd)
        os.fsync(stage_fd)
    finally:
        if audit_fd is not None:
            os.close(audit_fd)
        if build_fd is not None:
            os.close(build_fd)
        os.close(stage_fd)


class _BoundedPipe:
    def __init__(self, maximum: int) -> None:
        self.maximum = maximum
        self.data = b""
        self.truncated = False
        self.overflow = threading.Event()
        self._thread: threading.Thread | None = None

    def start(self, pipe: object) -> None:
        def drain() -> None:
            while True:
                chunk = pipe.read(65536)
                if not chunk:
                    return
                remaining = self.maximum - len(self.data)
                if remaining > 0:
                    self.data += chunk[:remaining]
                if len(chunk) > remaining:
                    self.truncated = True
                    self.overflow.set()

        self._thread = threading.Thread(target=drain, daemon=True)
        self._thread.start()

    def join(self, timeout: float) -> bool:
        if self._thread is None:
            return True
        self._thread.join(timeout)
        return not self._thread.is_alive()


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
    except OSError as error:
        data = str(error).encode("utf-8")
        return CommandResult(None, b"", data[:max_output_bytes], str(error), False, len(data) > max_output_bytes)
    assert process.stdout is not None and process.stderr is not None
    stdout.start(process.stdout)
    stderr.start(process.stderr)
    blocked_reason = None
    try:
        deadline = time.monotonic() + timeout_seconds
        exit_code = None
        while True:
            exit_code = process.poll()
            if exit_code is not None:
                if _process_group_exists(process.pid):
                    blocked_reason = "command descendants retained pipes after parent exit"
                    _terminate_process_group(process.pid)
                    exit_code = None
                break
            if stdout.overflow.is_set() or stderr.overflow.is_set():
                blocked_reason = "command exceeded output cap"
                _terminate_process_tree(process)
                exit_code = None
                break
            if time.monotonic() >= deadline:
                blocked_reason = f"command timed out after {timeout_seconds} seconds"
                _terminate_process_tree(process)
                exit_code = None
                break
            time.sleep(0.01)
    finally:
        if process.poll() is None:
            _terminate_process_tree(process)
    if blocked_reason is not None:
        try:
            process.wait(timeout=PIPE_DRAIN_GRACE_SECONDS)
        except subprocess.TimeoutExpired:
            _terminate_process_tree(process)
    stdout_drained = stdout.join(PIPE_DRAIN_GRACE_SECONDS)
    stderr_drained = stderr.join(PIPE_DRAIN_GRACE_SECONDS)
    drained = stdout_drained and stderr_drained
    if not drained:
        blocked_reason = "command pipes did not drain after parent exit"
        exit_code = None
        _terminate_process_group(process.pid)
        stdout_drained = stdout.join(PIPE_DRAIN_GRACE_SECONDS)
        stderr_drained = stderr.join(PIPE_DRAIN_GRACE_SECONDS)
        drained = stdout_drained and stderr_drained
    if stdout_drained:
        process.stdout.close()
    if stderr_drained:
        process.stderr.close()
    return CommandResult(
        exit_code,
        stdout.data,
        stderr.data,
        blocked_reason,
        stdout.truncated or not drained,
        stderr.truncated or not drained,
    )


def _terminate_process_tree(process: subprocess.Popen[bytes]) -> None:
    _terminate_process_group(process.pid)
    try:
        process.wait(timeout=PIPE_DRAIN_GRACE_SECONDS)
    except subprocess.TimeoutExpired:
        try:
            process.kill()
        except ProcessLookupError:
            pass
        process.wait(timeout=PIPE_DRAIN_GRACE_SECONDS)


def _process_group_exists(process_group: int) -> bool:
    try:
        os.killpg(process_group, 0)
        return True
    except ProcessLookupError:
        return False
    except PermissionError:
        return True


def _terminate_process_group(process_group: int) -> None:
    try:
        os.killpg(process_group, signal.SIGTERM)
    except ProcessLookupError:
        return
    deadline = time.monotonic() + PIPE_DRAIN_GRACE_SECONDS
    while _process_group_exists(process_group) and time.monotonic() < deadline:
        time.sleep(0.01)
    if _process_group_exists(process_group):
        try:
            os.killpg(process_group, signal.SIGKILL)
        except (ProcessLookupError, PermissionError):
            pass


def _sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()
