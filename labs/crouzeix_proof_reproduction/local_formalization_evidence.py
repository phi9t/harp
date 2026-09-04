"""Create the hash-bound local Crouzeix formalization evidence atomically."""

from __future__ import annotations

import ctypes
import errno
import json
import os
import platform
import re
import stat
import tempfile
from contextvars import ContextVar
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Callable, Mapping

if __package__:
    from . import (
        ls_contract,
        ls_receipts,
        ls_validation,
        protocol,
        provider_independence,
        route_validation,
    )
else:  # pragma: no cover - direct script execution path
    import ls_contract
    import ls_receipts
    import ls_validation
    import protocol
    import provider_independence
    import route_validation


HEADER = (
    "schema_version\tformalization_id\troute_id\tsource_node_id\t"
    "declaration_name\tmodule_path\tmodule_sha256\troute_manifest_path\t"
    "route_manifest_sha256\troute_receipt_path\troute_receipt_sha256\t"
    "route_review_path\troute_review_sha256\tbuild_command_path\t"
    "build_command_sha256\t"
    "build_stdout_path\tbuild_stdout_sha256\tbuild_stderr_path\t"
    "build_stderr_sha256\taxiom_audit_path\taxiom_audit_sha256\t"
    "allowed_axioms\tobserved_axioms\tprovider_independence_path\t"
    "provider_independence_sha256\tlean_toolchain\t"
    "lean_toolchain_sha256\tlake_manifest_sha256\tstatus"
)
SCHEMA_VERSION = "crouzeix-local-formalization/v2"
BUILD_SCHEMA_VERSION = "crouzeix-local-build-command/v1"
PROVIDER_SCHEMA_VERSION = "crouzeix-provider-independence/v1"
ALLOWED_AXIOMS = ("Classical.choice", "Quot.sound", "propext")
AGGREGATE_ARGV = ("scripts/check_lean_library.sh", "Crouzeix")
AGGREGATE_SUCCESS_LINES = (
    "[lean] target=Crouzeix",
    "[lean] root=formalization/lean",
    "[lean] outcome=passed",
)
MAX_OUTPUT_BYTES = 1024 * 1024
MAX_MEMBER_BYTES = 4 * 1024 * 1024
MAX_BUNDLE_ENTRIES = 1024
MAX_ACTIVE_SOURCE_FILES = 4096
MAX_SOURCE_SCAN_ENTRIES = 8192
MAX_ATTEMPTS = 256
TIMEOUT_SECONDS = 3600
STAGE_PREFIX = ".local-formalization-stage-"
DESTINATION_NAME = "local_formalization"
MANIFEST_NAME = "manifest.tsv"
EVIDENCE_ROOT_NAME = "crouzeix_conjecture"
DARWIN_RENAME_EXCL = 0x00000004
LINUX_RENAME_NOREPLACE = 0x00000001
ATTEMPT_NAME = re.compile(r"attempt-[0-9]+\Z")
LS_TARGET = Path("labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger")
LS_TERMINAL_NODE = "ls-terminal-crouzeix"
TERMINAL_MEMBER_PATHS = (
    "task.json",
    "source-slice.json",
    "result.json",
    "receipt.json",
    "build/command.json",
    "build/stdout.log",
    "build/stderr.log",
    "build/axioms.txt",
)


@dataclass(frozen=True)
class FormalizationSpec:
    formalization_id: str
    route_id: str
    source_node_id: str
    declaration_name: str
    module_path: str


FORMALIZATIONS = (
    FormalizationSpec(
        "harp-closed-numerical-range",
        "harp",
        "harp-closed-range-consequence",
        "CrouzeixConjecture."
        "harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet",
        "formalization/lean/Crouzeix/Harp/Consequences.lean",
    ),
    FormalizationSpec(
        "harp-main-theorem",
        "harp",
        "harp-terminal-theorem",
        "CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem",
        "formalization/lean/Crouzeix/Harp/MainTheorem.lean",
    ),
    FormalizationSpec(
        "jin-closed-numerical-range",
        "jin",
        "jin-hilbert-spectral-set-consequence",
        "CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet",
        "formalization/lean/CrouzeixConjecture/HilbertSpectralSet.lean",
    ),
    FormalizationSpec(
        "jin-main-theorem",
        "jin",
        "jin-terminal-crouzeix",
        "CrouzeixConjecture.crouzeixConjecture",
        "formalization/lean/Crouzeix/Jin/Terminal.lean",
    ),
    FormalizationSpec(
        "ls-closed-numerical-range",
        "lorist-schwenninger",
        "-",
        "CrouzeixConjecture."
        "loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet",
        "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean",
    ),
    FormalizationSpec(
        "ls-main-theorem",
        "lorist-schwenninger",
        LS_TERMINAL_NODE,
        "CrouzeixConjecture.loristSchwenningerMainTheorem",
        "formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean",
    ),
)

JIN_ROOTS = ("CrouzeixJin",)
HARP_ROOTS = (
    "Crouzeix.Harp.Consequences",
    "Crouzeix.Harp.FiniteAtomicL2Dilation",
    "Crouzeix.Harp.FiniteHorizonPerturbation",
    "Crouzeix.Harp.MainTheorem",
)
LS_ROOTS = (ls_contract.AGGREGATE_MODULE,)
LEGACY_FORBIDDEN = (
    "Crouzeix.Jin.Terminal",
    "CrouzeixConjecture.FinalTheorems",
    "CrouzeixConjecture.HilbertSpace",
    "CrouzeixConjecture.HilbertSpectralSet",
    "CrouzeixConjecture.RadialOuterReduction",
)


@dataclass(frozen=True)
class Publication:
    artifact_root: Path
    manifest_path: Path


class PublicationCommittedError(RuntimeError):
    """Report a committed bundle whose durability or cleanup step failed."""

    def __init__(
        self,
        publication: Publication,
        recovery_path: Path,
        errors: tuple[BaseException, ...],
    ) -> None:
        self.publication = publication
        self.recovery_path = recovery_path
        self.errors = errors
        details = "; ".join(str(error) for error in errors)
        super().__init__(
            "local formalization evidence was committed at "
            f"{recovery_path}, but post-commit durability or cleanup failed: {details}"
        )


class PublicationCleanupError(RuntimeError):
    """Preserve a pre-commit failure together with cleanup failures."""

    def __init__(
        self,
        primary_error: BaseException,
        cleanup_errors: tuple[BaseException, ...],
    ) -> None:
        self.primary_error = primary_error
        self.cleanup_errors = cleanup_errors
        details = "; ".join(str(error) for error in cleanup_errors)
        super().__init__(
            f"local formalization publication failed ({primary_error}); "
            f"cleanup also failed: {details}"
        )


@dataclass(frozen=True)
class _BuildState:
    wrapper: ls_receipts._FileSnapshot
    lake_manifest: ls_receipts._FileSnapshot
    dependency_cache_metadata_sha256: str
    toolchain: ls_receipts._FileSnapshot
    lakefile: ls_receipts._FileSnapshot
    lean_sources: tuple[tuple[str, ls_receipts._FileSnapshot], ...]
    required_mathlib_artifacts: tuple[tuple[str, ls_receipts._FileSnapshot], ...]


@dataclass(frozen=True)
class _TerminalEvidence:
    graph: ls_receipts._FileSnapshot
    receipt_path: str
    receipt_sha256: str
    members: tuple[tuple[str, ls_receipts._FileSnapshot], ...]


@dataclass(frozen=True)
class _RouteEvidence:
    manifest_path: str
    manifest: ls_receipts._FileSnapshot
    receipt_path: str
    receipt: ls_receipts._FileSnapshot
    review_path: str
    review: ls_receipts._FileSnapshot


_PINNED_STAGE_CLEANUP: ContextVar[
    tuple[ls_receipts._PinnedDirectory, ls_receipts._PinnedDirectory] | None
] = ContextVar("local_formalization_pinned_stage_cleanup", default=None)


Executor = Callable[
    [list[str], Path, dict[str, str], int, int], ls_receipts.CommandResult
]


def publish_local_formalization_evidence(
    repository_root: Path,
    *,
    executor: Executor | None = None,
) -> Publication:
    """Build and create the exact six-row local evidence bundle once."""

    repository = _absolute_normalized(repository_root)
    _ensure_directory(repository, "repository root")
    evidence_root = repository / "evidence" / EVIDENCE_ROOT_NAME
    _ensure_directory(evidence_root, "Crouzeix evidence root")
    destination = evidence_root / DESTINATION_NAME
    _require_absent(destination, "local formalization evidence bundle")

    publication_lock = ls_receipts._acquire_publication_lock(evidence_root)
    stage_root = repository / ".build"
    stage: Path | None = None
    repository_directory: ls_receipts._PinnedDirectory | None = None
    source_parent: ls_receipts._PinnedDirectory | None = None
    destination_parent: ls_receipts._PinnedDirectory | None = None
    staged_bundle: ls_receipts._PinnedDirectory | None = None
    publication: Publication | None = None
    committed = False
    primary_error: BaseException | None = None

    try:
        destination_parent = ls_receipts._pin_directory(
            evidence_root, "local formalization publication parent"
        )
        repository_directory = ls_receipts._pin_directory(
            repository, "local formalization repository root"
        )
        _require_absent(destination, "local formalization evidence bundle")
        lean_root = repository / "formalization/lean"
        _ensure_directory(lean_root, "Lean root")

        build_state = _snapshot_build_state(repository)
        ls_receipts._validate_wrapper_cache_policy(build_state.wrapper.data)
        lean_toolchain = _parse_toolchain(build_state.toolchain.data)
        toolchain_sha256 = protocol.sha256_bytes(build_state.toolchain.data)
        lake_manifest_sha256 = protocol.sha256_bytes(build_state.lake_manifest.data)
        modules = _snapshot_modules(repository)
        terminal = _snapshot_terminal_evidence(repository, modules["ls-main-theorem"])
        routes = _snapshot_route_evidence(repository)

        run = executor or ls_receipts._subprocess_executor
        environment = _shared_xdg_environment()
        build = _require_build_report(
            _require_success(
                run(
                    list(AGGREGATE_ARGV),
                    repository,
                    dict(environment),
                    TIMEOUT_SECONDS,
                    MAX_OUTPUT_BYTES,
                ),
                "aggregate build",
            )
        )
        audits = _run_axiom_audits(lean_root, run, environment)
        reports = _provider_reports(lean_root)

        try:
            os.mkdir(".build", mode=0o700, dir_fd=repository_directory.descriptor)
        except FileExistsError:
            pass
        except OSError as error:
            raise protocol.ValidationError(
                f"cannot create local formalization staging root: {error}"
            ) from error
        source_parent = ls_receipts._pin_directory_at(
            repository_directory,
            ".build",
            stage_root,
            "local formalization staging parent",
        )
        if source_parent.device != destination_parent.device:
            raise protocol.ValidationError(
                "local formalization stage and destination must share a filesystem"
            )
        staged_bundle = ls_receipts._create_unique_directory_at(
            source_parent, STAGE_PREFIX, "staged local formalization bundle"
        )
        stage = staged_bundle.path
        build_directory = _create_directory_at(
            staged_bundle, "build", "staged local formalization build directory"
        )
        axiom_directory = _create_directory_at(
            staged_bundle, "axioms", "staged local formalization axiom directory"
        )
        provider_directory = _create_directory_at(
            staged_bundle,
            "providers",
            "staged local formalization provider directory",
        )
        route_directory = _create_directory_at(
            staged_bundle, "routes", "staged local formalization route directory"
        )

        stdout_path = _published_path("build/stdout.log")
        stderr_path = _published_path("build/stderr.log")
        command = {
            "schema_version": BUILD_SCHEMA_VERSION,
            "argv": list(AGGREGATE_ARGV),
            "cwd": ".",
            "lean_toolchain": lean_toolchain,
            "lean_toolchain_sha256": toolchain_sha256,
            "lake_manifest_sha256": lake_manifest_sha256,
            "wrapper_sha256": protocol.sha256_bytes(build_state.wrapper.data),
            "lakefile_sha256": protocol.sha256_bytes(build_state.lakefile.data),
            "active_source_closure_sha256": _source_snapshot_digest(
                build_state.lean_sources
            ),
            "dependency_cache_metadata_sha256": build_state.dependency_cache_metadata_sha256,
            "required_mathlib_artifacts_sha256": _artifact_snapshot_digest(
                build_state.required_mathlib_artifacts
            ),
            "ls_graph_sha256": protocol.sha256_bytes(terminal.graph.data),
            "exit_code": 0,
            "status": "passed",
            "stdout_path": stdout_path,
            "stdout_sha256": protocol.sha256_bytes(build.stdout),
            "stderr_path": stderr_path,
            "stderr_sha256": protocol.sha256_bytes(build.stderr),
        }
        command_bytes = _canonical_json_bytes(command)
        members: dict[str, bytes] = {
            "build/command.json": command_bytes,
            "build/stdout.log": build.stdout,
            "build/stderr.log": build.stderr,
        }
        for spec in FORMALIZATIONS:
            members[f"axioms/{spec.formalization_id}.txt"] = audits[
                spec.formalization_id
            ][0]
        for route_id, report in reports.items():
            members[f"providers/{route_id}.json"] = _canonical_json_bytes(report)
        for route_id, route in routes.items():
            members[f"routes/{route_id}.manifest.json"] = route.manifest.data
            members[f"routes/{route_id}.receipt.json"] = route.receipt.data
            members[f"routes/{route_id}.review.json"] = route.review.data
        members[MANIFEST_NAME] = _manifest_bytes(
            members=members,
            modules=modules,
            routes=routes,
            audits=audits,
            lean_toolchain=lean_toolchain,
            toolchain_sha256=toolchain_sha256,
            lake_manifest_sha256=lake_manifest_sha256,
        )
        try:
            for relative_path, data in members.items():
                parent, name = _member_destination(
                    staged_bundle,
                    build_directory,
                    axiom_directory,
                    provider_directory,
                    route_directory,
                    relative_path,
                )
                _write_bytes_create_only_at(
                    parent.descriptor, name, data, f"local evidence {relative_path}"
                )
        finally:
            _close_pinned_directories(
                (build_directory, axiom_directory, provider_directory, route_directory)
            )

        _require_build_state(repository, build_state)
        if _snapshot_modules(repository) != modules:
            raise protocol.ValidationError(
                "local formalization modules changed during evidence generation"
            )
        _require_terminal_evidence(repository, terminal, modules["ls-main-theorem"])
        ls_receipts._require_descriptor_identity(
            destination_parent, "local formalization evidence bundle parent"
        )
        _require_route_evidence(repository, routes)
        _require_staged_bundle(staged_bundle, members)
        _require_entry_absent(
            destination_parent, DESTINATION_NAME, "local formalization evidence bundle"
        )

        _fsync_tree(staged_bundle)
        _require_build_state(repository, build_state)
        if _snapshot_modules(repository) != modules:
            raise protocol.ValidationError(
                "local formalization modules changed during evidence generation"
            )
        _require_terminal_evidence(repository, terminal, modules["ls-main-theorem"])
        ls_receipts._require_descriptor_identity(
            destination_parent, "local formalization evidence bundle parent"
        )
        _require_route_evidence(repository, routes)
        _require_staged_bundle(staged_bundle, members)
        _require_entry_absent(
            destination_parent, DESTINATION_NAME, "local formalization evidence bundle"
        )
        published = _rename_directory_no_replace(
            source_parent, staged_bundle, destination_parent, DESTINATION_NAME
        )
        committed = True
        publication = Publication(
            evidence_root / DESTINATION_NAME,
            evidence_root / DESTINATION_NAME / MANIFEST_NAME,
        )

        post_commit_errors: list[BaseException] = []
        for validate in (
            lambda: ls_receipts._require_descriptor_identity(
                source_parent, "local formalization staging parent"
            ),
            lambda: ls_receipts._require_descriptor_identity(
                destination_parent, "local formalization publication parent"
            ),
            lambda: _require_staged_bundle(published, members),
        ):
            try:
                validate()
            except BaseException as error:
                post_commit_errors.append(error)
        for parent, label in (
            (source_parent, "local formalization staging parent"),
            (destination_parent, "local formalization publication parent"),
        ):
            try:
                _fsync_pinned_directory(parent, label)
            except BaseException as error:
                post_commit_errors.append(error)
        if post_commit_errors:
            raise PublicationCommittedError(
                publication, publication.artifact_root, tuple(post_commit_errors)
            )
        return publication
    except BaseException as error:
        primary_error = error
        raise
    finally:
        cleanup_errors: list[BaseException] = []
        if not committed:
            try:
                if staged_bundle is not None and source_parent is not None:
                    cleanup_token = _PINNED_STAGE_CLEANUP.set(
                        (source_parent, staged_bundle)
                    )
                    try:
                        _remove_stage(stage, stage_root)
                    finally:
                        _PINNED_STAGE_CLEANUP.reset(cleanup_token)
                elif stage is not None:
                    _remove_stage(stage, stage_root)
            except BaseException as error:
                cleanup_errors.append(error)
        descriptors: set[int] = set()
        for pinned in (
            staged_bundle,
            source_parent,
            destination_parent,
            repository_directory,
        ):
            if pinned is None or pinned.descriptor in descriptors:
                continue
            descriptors.update((pinned.descriptor, *pinned.owned_ancestor_descriptors))
            try:
                ls_receipts._close_pinned_directory(pinned)
            except BaseException as error:
                cleanup_errors.append(error)
        try:
            ls_receipts._release_publication_lock(publication_lock)
        except BaseException as error:
            cleanup_errors.append(error)
        if cleanup_errors:
            if committed and publication is not None:
                committed_error = PublicationCommittedError(
                    publication, publication.artifact_root, tuple(cleanup_errors)
                )
                if primary_error is not None:
                    raise committed_error from primary_error
                raise committed_error
            if primary_error is not None:
                raise PublicationCleanupError(
                    primary_error, tuple(cleanup_errors)
                ) from primary_error
            if len(cleanup_errors) == 1:
                raise cleanup_errors[0]
            first, *remaining = cleanup_errors
            raise PublicationCleanupError(first, tuple(remaining)) from first


def _snapshot_build_state(repository: Path) -> _BuildState:
    lean_root = repository / "formalization/lean"
    sources = _active_source_snapshots(repository)
    return _BuildState(
        wrapper=ls_receipts._read_file_snapshot(
            repository / AGGREGATE_ARGV[0],
            "aggregate Lean wrapper",
            MAX_MEMBER_BYTES,
        ),
        lake_manifest=ls_receipts._read_file_snapshot(
            lean_root / "lake-manifest.json",
            "Lean lake manifest",
            MAX_MEMBER_BYTES,
        ),
        dependency_cache_metadata_sha256=ls_receipts._snapshot_digest(
            ls_receipts._dependency_cache_metadata_snapshot(lean_root)
        ),
        toolchain=ls_receipts._read_file_snapshot(
            lean_root / "lean-toolchain",
            "Lean toolchain identity",
            MAX_MEMBER_BYTES,
        ),
        lakefile=ls_receipts._read_file_snapshot(
            lean_root / "lakefile.toml", "Lean Lake configuration", MAX_MEMBER_BYTES
        ),
        lean_sources=sources,
        required_mathlib_artifacts=_required_mathlib_artifact_snapshots(
            lean_root, sources
        ),
    )


def _require_build_state(repository: Path, expected: _BuildState) -> None:
    if _snapshot_build_state(repository) != expected:
        raise protocol.ValidationError(
            "local formalization build state changed during evidence generation"
        )


def _active_source_snapshots(
    repository: Path,
) -> tuple[tuple[str, ls_receipts._FileSnapshot], ...]:
    lean_root = repository / "formalization/lean"
    source_paths: list[Path] = []
    aggregate = lean_root / "Crouzeix.lean"
    if aggregate.exists() or aggregate.is_symlink():
        source_paths.append(aggregate)
    for source_root in (lean_root / "Crouzeix", lean_root / "CrouzeixConjecture"):
        if not source_root.exists() and not source_root.is_symlink():
            continue
        _ensure_directory(source_root, "Crouzeix Lean source root")
        source_paths.extend(_bounded_lean_source_paths(source_root))
    source_paths.sort()
    if len(source_paths) > MAX_ACTIVE_SOURCE_FILES:
        raise protocol.ValidationError(
            "active Crouzeix source count exceeds safety cap"
        )
    return tuple(
        (
            source_path.relative_to(repository).as_posix(),
            ls_receipts._read_file_snapshot(
                source_path, "active Crouzeix source", MAX_MEMBER_BYTES
            ),
        )
        for source_path in source_paths
    )


def _required_mathlib_artifact_snapshots(
    lean_root: Path,
    sources: tuple[tuple[str, ls_receipts._FileSnapshot], ...],
) -> tuple[tuple[str, ls_receipts._FileSnapshot], ...]:
    modules: set[str] = set()
    for relative_path, snapshot in sources:
        try:
            source = snapshot.data.decode("utf-8")
        except UnicodeDecodeError as error:
            raise protocol.ValidationError(
                f"active Crouzeix source is not UTF-8: {relative_path}"
            ) from error
        module = (
            PurePosixPath(relative_path)
            .relative_to("formalization/lean")
            .with_suffix("")
        )
        module_name = ".".join(module.parts)
        try:
            imports = provider_independence.parse_active_imports(source, module_name)
        except provider_independence.ProviderIndependenceError as error:
            raise protocol.ValidationError(str(error)) from error
        modules.update(item for item in imports if item.startswith("Mathlib"))
    lake = ls_receipts._pin_absolute_directory(
        ls_receipts._resolve_approved_lake_root(lean_root),
        "approved Lean .lake root",
    )
    packages: ls_receipts._PinnedDirectory | None = None
    try:
        packages = ls_receipts._pin_approved_packages_root_at(lake)
        return tuple(
            (
                module,
                ls_receipts._read_file_snapshot_relative_at(
                    packages,
                    (
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
            for module in sorted(modules)
        )
    finally:
        if packages is not None:
            ls_receipts._close_pinned_directory(packages)
        ls_receipts._close_pinned_directory(lake)


def _snapshot_terminal_evidence(
    repository: Path, module_digest: str
) -> _TerminalEvidence:
    target = repository / LS_TARGET
    graph_path = target / "source-graph.json"
    graph = ls_receipts._read_file_snapshot(
        graph_path, "LS source graph", MAX_MEMBER_BYTES
    )
    rows = ls_validation.load_route_graph(graph_path)
    if tuple(row.node_id for row in rows) != ls_contract.NODE_ORDER or any(
        row.status != "passed" or row.receipt_sha256 is None for row in rows
    ):
        raise protocol.ValidationError(
            "all canonical LS graph rows must be passed and receipt-bound"
        )
    terminal = next((row for row in rows if row.node_id == LS_TERMINAL_NODE), None)
    if (
        terminal is None
        or terminal.status != "passed"
        or terminal.receipt_sha256 is None
    ):
        raise protocol.ValidationError("LS terminal source node is not passed")
    receipts = ls_validation.validate_committed_receipts(rows, target, repository)
    receipt = receipts.get(LS_TERMINAL_NODE)
    if receipt is None:
        raise protocol.ValidationError("LS terminal receipt was not validated")
    if receipt["module_sha256"] != module_digest:
        raise protocol.ValidationError("LS terminal receipt module_sha256 mismatch")
    receipt_path = _resolve_receipt_path(target, terminal.receipt_sha256)
    attempt = receipt_path.parent
    if ATTEMPT_NAME.fullmatch(attempt.name) is None:
        raise protocol.ValidationError(
            "LS terminal receipt attempt must use attempt- followed by digits"
        )
    members = tuple(
        (
            relative_path,
            ls_receipts._read_file_snapshot(
                attempt.joinpath(*PurePosixPath(relative_path).parts),
                f"LS terminal receipt member {relative_path}",
                MAX_MEMBER_BYTES,
            ),
        )
        for relative_path in TERMINAL_MEMBER_PATHS
    )
    receipt_snapshot = dict(members)["receipt.json"]
    if protocol.sha256_bytes(receipt_snapshot.data) != terminal.receipt_sha256:
        raise protocol.ValidationError(
            "LS terminal receipt no longer matches the source graph"
        )
    if (
        ls_receipts._read_file_snapshot(graph_path, "LS source graph", MAX_MEMBER_BYTES)
        != graph
    ):
        raise protocol.ValidationError(
            "LS terminal source graph changed during validation"
        )
    return _TerminalEvidence(
        graph=graph,
        receipt_path=receipt_path.relative_to(repository).as_posix(),
        receipt_sha256=terminal.receipt_sha256,
        members=members,
    )


def _require_terminal_evidence(
    repository: Path, expected: _TerminalEvidence, module_digest: str
) -> None:
    graph_path = repository / LS_TARGET / "source-graph.json"
    if (
        ls_receipts._read_file_snapshot(graph_path, "LS source graph", MAX_MEMBER_BYTES)
        != expected.graph
    ):
        raise protocol.ValidationError("LS terminal source graph changed")
    attempt = repository.joinpath(*PurePosixPath(expected.receipt_path).parts).parent
    for relative_path, snapshot in expected.members:
        if (
            ls_receipts._read_file_snapshot(
                attempt.joinpath(*PurePosixPath(relative_path).parts),
                f"LS terminal receipt member {relative_path}",
                MAX_MEMBER_BYTES,
            )
            != snapshot
        ):
            raise protocol.ValidationError(
                f"LS terminal receipt member changed: {relative_path}"
            )
    observed = _snapshot_terminal_evidence(repository, module_digest)
    if observed != expected:
        raise protocol.ValidationError("LS terminal receipt binding changed")


def _resolve_receipt_path(target: Path, digest: str) -> Path:
    root = target / "proof-slices" / LS_TERMINAL_NODE
    _ensure_directory(root, "LS terminal proof-slice root")
    matches: list[Path] = []
    for entry in _bounded_sorted_scandir(root, MAX_ATTEMPTS, "LS terminal attempts"):
        if ATTEMPT_NAME.fullmatch(entry.name) is None:
            continue
        metadata = entry.stat(follow_symlinks=False)
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISDIR(metadata.st_mode):
            raise protocol.ValidationError(
                f"LS terminal attempt must be a directory: {entry.name}"
            )
        candidate = Path(entry.path) / "receipt.json"
        data = _read_safe_file(candidate, "LS terminal receipt", MAX_MEMBER_BYTES)
        if protocol.sha256_bytes(data) == digest:
            matches.append(candidate)
    if len(matches) != 1:
        raise protocol.ValidationError(
            f"exactly one LS terminal receipt must match; found {len(matches)}"
        )
    return matches[0]


def _run_axiom_audits(
    lean_root: Path,
    executor: Executor,
    environment: dict[str, str],
) -> dict[str, tuple[bytes, tuple[str, ...]]]:
    audits: dict[str, tuple[bytes, tuple[str, ...]]] = {}
    for spec in FORMALIZATIONS:
        module = PurePosixPath(spec.module_path).relative_to("formalization/lean")
        module_name = ".".join(module.with_suffix("").parts)
        source = (
            f"import {module_name}\n#print axioms {spec.declaration_name}\n"
        ).encode("utf-8")
        with tempfile.TemporaryDirectory(
            prefix=".local-axiom-audit-", dir=lean_root
        ) as directory:
            audit_path = Path(directory) / "Audit.lean"
            _write_bytes_create_only(audit_path, source, "axiom audit source")
            relative = audit_path.relative_to(lean_root).as_posix()
            result = _require_success(
                executor(
                    ["lake", "env", "lean", relative],
                    lean_root,
                    dict(environment),
                    TIMEOUT_SECONDS,
                    MAX_OUTPUT_BYTES,
                ),
                f"axiom audit for {spec.formalization_id}",
            )
        observed = ls_receipts._parse_axiom_audit(result.stdout, spec.declaration_name)
        audits[spec.formalization_id] = (result.stdout, observed)
    return audits


def _shared_xdg_environment() -> dict[str, str]:
    home = os.environ.get("HOME")
    if not home:
        raise protocol.ValidationError("HOME must be set for cached Lean execution")
    cache_home = Path(os.environ.get("XDG_CACHE_HOME", Path(home) / ".cache"))
    if not cache_home.is_absolute():
        raise protocol.ValidationError("XDG_CACHE_HOME must be absolute")
    elan_home = cache_home / "harp/lean/elan"
    toolchain_bin = (
        elan_home / "toolchains/leanprover--lean4---v4.32.1/bin"
    )
    _ensure_directory(elan_home, "shared ELAN_HOME")
    _ensure_directory(toolchain_bin, "shared Lean toolchain bin")
    for executable in ("lake", "lean"):
        path = toolchain_bin / executable
        try:
            metadata = path.lstat()
        except OSError as error:
            raise protocol.ValidationError(
                f"shared Lean executable is unavailable: {path}: {error}"
            ) from error
        if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
            raise protocol.ValidationError(
                f"shared Lean executable must be a regular file: {path}"
            )
        if not os.access(path, os.X_OK):
            raise protocol.ValidationError(
                f"shared Lean executable is not executable: {path}"
            )
    return {
        "ELAN_HOME": elan_home.as_posix(),
        "ELAN_TOOLCHAIN": "leanprover/lean4:v4.32.1",
        "HARP_ELAN_HOME": elan_home.as_posix(),
        "HARP_LEAN_CACHE_ROOT": (
            cache_home / "harp/lean/lean-4.32.1"
        ).as_posix(),
        "PATH": os.pathsep.join(
            path.as_posix()
            for path in (toolchain_bin, Path("/usr/bin"), Path("/bin"))
        ),
    }


def _provider_reports(lean_root: Path) -> dict[str, dict[str, object]]:
    reports: dict[str, dict[str, object]] = {}
    for route_id, roots, other_roots in (
        ("jin", JIN_ROOTS, (*LS_ROOTS, *HARP_ROOTS)),
        ("harp", HARP_ROOTS, LS_ROOTS),
        ("lorist-schwenninger", LS_ROOTS, (*HARP_ROOTS, *JIN_ROOTS)),
    ):
        if route_id == "jin":
            forbidden = tuple(sorted(("Crouzeix", "CrouzeixLoristSchwenninger", "CrouzeixHarp")))
            forbidden_prefixes = ("Crouzeix.LoristSchwenninger", "Crouzeix.Harp")
        else:
            forbidden, forbidden_prefixes = ls_contract.provider_policy(
                route_id,
                tuple(sorted((*LEGACY_FORBIDDEN, *other_roots))),
            )
        try:
            report = provider_independence.audit_provider_independence(
                lean_root,
                roots,
                forbidden,
                forbidden_prefixes=forbidden_prefixes,
            )
        except provider_independence.ProviderIndependenceError as error:
            raise protocol.ValidationError(str(error)) from error
        modules = []
        for module_name in report.modules:
            path = lean_root.joinpath(*module_name.split(".")).with_suffix(".lean")
            data = _read_safe_file(
                path, f"provider module {module_name}", MAX_MEMBER_BYTES
            )
            modules.append(
                {
                    "module_name": module_name,
                    "module_path": path.relative_to(lean_root.parent.parent).as_posix(),
                    "module_sha256": protocol.sha256_bytes(data),
                }
            )
        reports[route_id] = {
            "schema_version": PROVIDER_SCHEMA_VERSION,
            "route_id": route_id,
            "status": "passed",
            "roots": list(roots),
            "forbidden_modules": list(forbidden),
            "forbidden_prefixes": list(forbidden_prefixes),
            "modules": modules,
            "set_options": [
                {
                    "module_name": finding.module,
                    "line": finding.line,
                    "token": finding.token,
                    "source_line": finding.source_line,
                }
                for finding in report.set_options
            ],
        }
    return reports


def _snapshot_modules(repository: Path) -> dict[str, str]:
    snapshots = {}
    for spec in FORMALIZATIONS:
        path = repository.joinpath(*PurePosixPath(spec.module_path).parts)
        data = _read_safe_file(
            path, f"formalization module {spec.formalization_id}", MAX_MEMBER_BYTES
        )
        snapshots[spec.formalization_id] = protocol.sha256_bytes(data)
    return snapshots


def _snapshot_route_evidence(repository: Path) -> dict[str, _RouteEvidence]:
    snapshots: dict[str, _RouteEvidence] = {}
    for route_id in route_validation.ROUTE_IDS:
        result = route_validation.inspect_route(repository, route_id)
        if result.status != "complete" or result.claim_level != "complete-local":
            raise protocol.ValidationError(f"{route_id} route is not complete-local")
        manifest_path = route_validation.ROUTE_MANIFEST_PATHS[route_id]
        manifest_raw = route_validation._read_json(
            repository, manifest_path, f"{route_id} route manifest"
        )
        if manifest_raw.get("route_id") != route_id:
            raise protocol.ValidationError(f"{route_id} route manifest identity mismatch")
        receipt_path = manifest_raw.get("receipt_path")
        receipt_sha256 = manifest_raw.get("receipt_sha256")
        review_path = manifest_raw.get("review_path")
        review_sha256 = manifest_raw.get("review_sha256")
        if not all(
            isinstance(value, str) and value
            for value in (receipt_path, receipt_sha256, review_path, review_sha256)
        ):
            raise protocol.ValidationError(
                f"{route_id} route manifest lacks receipt or review binding"
            )
        try:
            receipt_path = route_validation._safe_relative(
                receipt_path, f"{route_id} route receipt path"
            )
            review_path = route_validation._safe_relative(
                review_path, f"{route_id} route review path"
            )
        except route_validation.RouteValidationError as error:
            raise protocol.ValidationError(
                f"{route_id} route manifest has an unsafe publication path: {error}"
            ) from error
        snapshots[route_id] = _RouteEvidence(
            manifest_path=manifest_path.as_posix(),
            manifest=ls_receipts._read_file_snapshot(
                repository / manifest_path, f"{route_id} route manifest", MAX_MEMBER_BYTES
            ),
            receipt_path=receipt_path,
            receipt=ls_receipts._read_file_snapshot(
                repository.joinpath(*PurePosixPath(receipt_path).parts),
                f"{route_id} route receipt",
                MAX_MEMBER_BYTES,
            ),
            review_path=review_path,
            review=ls_receipts._read_file_snapshot(
                repository.joinpath(*PurePosixPath(review_path).parts),
                f"{route_id} route review",
                MAX_MEMBER_BYTES,
            ),
        )
        confirmed = route_validation.inspect_route(repository, route_id)
        if confirmed.status != "complete" or confirmed.claim_level != "complete-local":
            raise protocol.ValidationError(f"{route_id} route is not complete-local")
        current = snapshots[route_id]
        if (
            ls_receipts._read_file_snapshot(
                repository / manifest_path,
                f"{route_id} route manifest",
                MAX_MEMBER_BYTES,
            )
            != current.manifest
            or ls_receipts._read_file_snapshot(
                repository.joinpath(*PurePosixPath(receipt_path).parts),
                f"{route_id} route receipt",
                MAX_MEMBER_BYTES,
            )
            != current.receipt
            or ls_receipts._read_file_snapshot(
                repository.joinpath(*PurePosixPath(review_path).parts),
                f"{route_id} route review",
                MAX_MEMBER_BYTES,
            )
            != current.review
        ):
            raise protocol.ValidationError(
                f"{route_id} route evidence changed during validation"
            )
    return snapshots


def _require_route_evidence(
    repository: Path, expected: Mapping[str, _RouteEvidence]
) -> None:
    try:
        observed = _snapshot_route_evidence(repository)
    except protocol.ValidationError as error:
        if "evidence/crouzeix_conjecture" in str(error):
            raise protocol.ValidationError(
                "local formalization evidence bundle parent identity changed"
            ) from error
        raise
    if observed != dict(expected):
        raise protocol.ValidationError(
            "published route evidence changed during local bundle generation"
        )


def _manifest_bytes(
    *,
    members: Mapping[str, bytes],
    modules: Mapping[str, str],
    routes: Mapping[str, _RouteEvidence],
    audits: Mapping[str, tuple[bytes, tuple[str, ...]]],
    lean_toolchain: str,
    toolchain_sha256: str,
    lake_manifest_sha256: str,
) -> bytes:
    command = "build/command.json"
    stdout = "build/stdout.log"
    stderr = "build/stderr.log"
    rows = []
    for spec in FORMALIZATIONS:
        audit = f"axioms/{spec.formalization_id}.txt"
        provider = f"providers/{spec.route_id}.json"
        route = routes[spec.route_id]
        route_manifest = f"routes/{spec.route_id}.manifest.json"
        route_receipt = f"routes/{spec.route_id}.receipt.json"
        route_review = f"routes/{spec.route_id}.review.json"
        observed = audits[spec.formalization_id][1]
        rows.append(
            (
                SCHEMA_VERSION,
                spec.formalization_id,
                spec.route_id,
                spec.source_node_id,
                spec.declaration_name,
                spec.module_path,
                modules[spec.formalization_id],
                _published_path(route_manifest),
                protocol.sha256_bytes(members[route_manifest]),
                _published_path(route_receipt),
                protocol.sha256_bytes(members[route_receipt]),
                _published_path(route_review),
                protocol.sha256_bytes(members[route_review]),
                _published_path(command),
                protocol.sha256_bytes(members[command]),
                _published_path(stdout),
                protocol.sha256_bytes(members[stdout]),
                _published_path(stderr),
                protocol.sha256_bytes(members[stderr]),
                _published_path(audit),
                protocol.sha256_bytes(members[audit]),
                ",".join(ALLOWED_AXIOMS),
                ",".join(observed) if observed else "-",
                _published_path(provider),
                protocol.sha256_bytes(members[provider]),
                lean_toolchain,
                toolchain_sha256,
                lake_manifest_sha256,
                "passed",
            )
        )
    text = HEADER + "\n" + "\n".join("\t".join(row) for row in rows) + "\n"
    return text.encode("utf-8")


def _published_path(relative_path: str) -> str:
    path = PurePosixPath(relative_path)
    if path.is_absolute() or ".." in path.parts or not path.parts:
        raise protocol.ValidationError("local evidence artifact path is unsafe")
    return (
        PurePosixPath("evidence") / EVIDENCE_ROOT_NAME / DESTINATION_NAME / path
    ).as_posix()


def _require_build_report(
    result: ls_receipts.CommandResult,
) -> ls_receipts.CommandResult:
    try:
        lines = result.stdout.decode("utf-8").splitlines()
    except UnicodeDecodeError as error:
        raise protocol.ValidationError(
            "aggregate build stdout must be UTF-8"
        ) from error
    outcomes = [line for line in lines if line.startswith("[lean] outcome=")]
    if any(
        lines.count(marker) != 1 for marker in AGGREGATE_SUCCESS_LINES
    ) or outcomes != ["[lean] outcome=passed"]:
        raise protocol.ValidationError(
            "aggregate build stdout does not contain a successful Crouzeix build report"
        )
    return result


def _require_success(
    result: ls_receipts.CommandResult, label: str
) -> ls_receipts.CommandResult:
    if not isinstance(result, ls_receipts.CommandResult):
        raise protocol.ValidationError("executor must return CommandResult")
    if not isinstance(result.stdout, bytes) or not isinstance(result.stderr, bytes):
        raise protocol.ValidationError("executor output must be bytes")
    if (
        result.stdout_truncated
        or result.stderr_truncated
        or len(result.stdout) > MAX_OUTPUT_BYTES
        or len(result.stderr) > MAX_OUTPUT_BYTES
    ):
        raise protocol.ValidationError(f"{label} exceeded output cap")
    if result.blocked_reason is not None or result.exit_code is None:
        raise protocol.ValidationError(
            f"{label} was blocked: {result.blocked_reason or 'missing exit code'}"
        )
    if type(result.exit_code) is not int:
        raise protocol.ValidationError(f"{label} returned an invalid exit code")
    if result.exit_code != 0:
        raise protocol.ValidationError(
            f"{label} failed with exit code {result.exit_code}"
        )
    return result


def _parse_toolchain(data: bytes) -> str:
    try:
        text = data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise protocol.ValidationError(
            "Lean toolchain identity must be UTF-8"
        ) from error
    if not text.endswith("\n") or "\n" in text[:-1] or "\r" in text or "\0" in text:
        raise protocol.ValidationError(
            "Lean toolchain identity must be one LF-terminated line"
        )
    value = text[:-1]
    if not value or value.strip() != value:
        raise protocol.ValidationError("Lean toolchain identity is invalid")
    return value


def _read_safe_file(path: Path, label: str, maximum: int) -> bytes:
    return ls_receipts._read_safe_file(path, label, maximum)


def _write_json_create_only(
    path: Path, value: Mapping[str, object], label: str
) -> None:
    _write_bytes_create_only(path, _canonical_json_bytes(value), label)


def _canonical_json_bytes(value: object) -> bytes:
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
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    if hasattr(os, "O_NOFOLLOW"):
        flags |= os.O_NOFOLLOW
    try:
        descriptor = os.open(path, flags, 0o600)
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
        os.fsync(descriptor)
    except OSError as error:
        raise protocol.ValidationError(f"cannot write {label}: {error}") from error
    finally:
        os.close(descriptor)


def _write_bytes_create_only_at(
    directory_descriptor: int, name: str, data: bytes, label: str
) -> None:
    ls_receipts._write_bytes_create_only_at(directory_descriptor, name, data, label)


def _create_directory_at(
    parent: ls_receipts._PinnedDirectory, name: str, label: str
) -> ls_receipts._PinnedDirectory:
    ls_receipts._safe_entry_name(name, label)
    ls_receipts._require_pinned_directory(parent, f"{label} parent")
    try:
        os.mkdir(name, mode=0o700, dir_fd=parent.descriptor)
    except OSError as error:
        raise protocol.ValidationError(f"cannot create {label}: {error}") from error
    try:
        return ls_receipts._pin_directory_at(parent, name, parent.path / name, label)
    except BaseException:
        try:
            os.rmdir(name, dir_fd=parent.descriptor)
        except OSError:
            pass
        raise


def _member_destination(
    root: ls_receipts._PinnedDirectory,
    build: ls_receipts._PinnedDirectory,
    axioms: ls_receipts._PinnedDirectory,
    providers: ls_receipts._PinnedDirectory,
    routes: ls_receipts._PinnedDirectory,
    relative_path: str,
) -> tuple[ls_receipts._PinnedDirectory, str]:
    path = PurePosixPath(relative_path)
    if len(path.parts) == 1:
        return root, path.name
    directories = {
        "build": build,
        "axioms": axioms,
        "providers": providers,
        "routes": routes,
    }
    if len(path.parts) != 2 or path.parts[0] not in directories:
        raise protocol.ValidationError("local evidence member path is unsafe")
    return directories[path.parts[0]], path.name


def _close_pinned_directories(
    directories: tuple[ls_receipts._PinnedDirectory, ...],
) -> None:
    errors: list[BaseException] = []
    for directory in directories:
        try:
            ls_receipts._close_pinned_directory(directory)
        except BaseException as error:
            errors.append(error)
    if errors:
        raise protocol.ValidationError(
            "cannot close staged local formalization directories: "
            + "; ".join(str(error) for error in errors)
        )


def _source_snapshot_digest(
    snapshots: tuple[tuple[str, ls_receipts._FileSnapshot], ...],
) -> str:
    records = [
        {"path": path, "sha256": protocol.sha256_bytes(snapshot.data)}
        for path, snapshot in snapshots
    ]
    return protocol.sha256_bytes(_canonical_json_bytes({"sources": records}))


def _artifact_snapshot_digest(
    snapshots: tuple[tuple[str, ls_receipts._FileSnapshot], ...],
) -> str:
    records = [
        {"module": module, "sha256": protocol.sha256_bytes(snapshot.data)}
        for module, snapshot in snapshots
    ]
    return protocol.sha256_bytes(_canonical_json_bytes({"artifacts": records}))


def _require_staged_bundle(
    stage: ls_receipts._PinnedDirectory, expected_members: Mapping[str, bytes]
) -> None:
    ls_receipts._require_pinned_directory(stage, "local formalization bundle")
    observed = _bundle_member_snapshots(stage.descriptor)
    if set(observed) != set(expected_members):
        raise protocol.ValidationError("local formalization bundle roster differs")
    for relative_path, expected in expected_members.items():
        if observed[relative_path] != expected:
            raise protocol.ValidationError(
                f"local formalization bundle member changed: {relative_path}"
            )


def _bundle_member_snapshots(directory_descriptor: int) -> dict[str, bytes]:
    provider_paths = {
        f"providers/{route_id}.json" for route_id in {spec.route_id for spec in FORMALIZATIONS}
    }
    route_paths = {
        f"routes/{route_id}.{kind}.json"
        for route_id in {spec.route_id for spec in FORMALIZATIONS}
        for kind in ("manifest", "receipt", "review")
    }
    expected_files = {
        MANIFEST_NAME,
        "build/command.json",
        "build/stdout.log",
        "build/stderr.log",
        *provider_paths,
        *route_paths,
        *(f"axioms/{spec.formalization_id}.txt" for spec in FORMALIZATIONS),
    }
    expected_directories = {"build", "axioms", "providers", "routes"}
    files: dict[str, bytes] = {}
    directories: set[str] = set()
    entries_seen = [0]
    _collect_bundle_members_at(
        directory_descriptor, "", files, directories, entries_seen
    )
    if set(files) != expected_files or directories != expected_directories:
        raise protocol.ValidationError("local formalization bundle roster differs")
    return files


def _collect_bundle_members_at(
    directory_descriptor: int,
    prefix: str,
    files: dict[str, bytes],
    directories: set[str],
    entries_seen: list[int],
) -> None:
    for entry in _bounded_sorted_scandir_at(
        directory_descriptor, MAX_BUNDLE_ENTRIES - entries_seen[0], "bundle"
    ):
        entries_seen[0] += 1
        relative = f"{prefix}/{entry.name}" if prefix else entry.name
        metadata = entry.stat(follow_symlinks=False)
        if stat.S_ISLNK(metadata.st_mode):
            raise protocol.ValidationError(
                f"local formalization bundle contains a symlink: {relative}"
            )
        if stat.S_ISREG(metadata.st_mode):
            if metadata.st_nlink != 1:
                raise protocol.ValidationError(
                    f"local formalization bundle file must have exactly one hard link: {relative}"
                )
            files[relative] = ls_receipts._read_safe_file_at(
                directory_descriptor,
                entry.name,
                f"local formalization bundle member {relative}",
                MAX_MEMBER_BYTES,
            )
        elif stat.S_ISDIR(metadata.st_mode):
            directories.add(relative)
            child = _open_directory_at(
                directory_descriptor, entry.name, metadata, relative
            )
            try:
                _collect_bundle_members_at(
                    child, relative, files, directories, entries_seen
                )
            finally:
                os.close(child)
        else:
            raise protocol.ValidationError(
                f"local formalization bundle contains an unsafe entry: {relative}"
            )


def _fsync_tree(root: ls_receipts._PinnedDirectory) -> None:
    """Fsync every regular file and directory in a pinned staged bundle."""

    ls_receipts._require_pinned_directory(root, "staged local formalization bundle")
    entries_seen = [0]
    _fsync_tree_at(root.descriptor, "staged local formalization bundle", entries_seen)
    ls_receipts._require_pinned_directory(root, "staged local formalization bundle")


def _fsync_tree_at(
    directory_descriptor: int, label: str, entries_seen: list[int]
) -> None:
    try:
        entries = _bounded_sorted_scandir_at(
            directory_descriptor,
            MAX_BUNDLE_ENTRIES - entries_seen[0],
            "local formalization bundle",
        )
    except OSError as error:
        raise protocol.ValidationError(f"cannot scan {label}: {error}") from error
    for entry in entries:
        entries_seen[0] += 1
        if entries_seen[0] > MAX_BUNDLE_ENTRIES:
            raise protocol.ValidationError(
                "local formalization bundle exceeds entry cap"
            )
        try:
            metadata = entry.stat(follow_symlinks=False)
        except OSError as error:
            raise protocol.ValidationError(
                f"cannot inspect {label} member {entry.name}: {error}"
            ) from error
        if stat.S_ISLNK(metadata.st_mode):
            raise protocol.ValidationError(f"{label} contains a symlink: {entry.name}")
        flags = os.O_RDONLY | os.O_NOFOLLOW
        if hasattr(os, "O_CLOEXEC"):
            flags |= os.O_CLOEXEC
        if stat.S_ISDIR(metadata.st_mode):
            flags |= os.O_DIRECTORY
            try:
                child = os.open(entry.name, flags, dir_fd=directory_descriptor)
            except OSError as error:
                raise protocol.ValidationError(
                    f"cannot open {label} directory {entry.name}: {error}"
                ) from error
            try:
                opened = os.fstat(child)
                if (opened.st_dev, opened.st_ino) != (metadata.st_dev, metadata.st_ino):
                    raise protocol.ValidationError(
                        f"{label} directory changed before fsync: {entry.name}"
                    )
                _fsync_tree_at(child, f"{label}/{entry.name}", entries_seen)
                current = os.stat(
                    entry.name,
                    dir_fd=directory_descriptor,
                    follow_symlinks=False,
                )
                if (current.st_dev, current.st_ino) != (opened.st_dev, opened.st_ino):
                    raise protocol.ValidationError(
                        f"{label} directory changed during fsync: {entry.name}"
                    )
            finally:
                os.close(child)
        elif stat.S_ISREG(metadata.st_mode):
            if metadata.st_nlink != 1:
                raise protocol.ValidationError(
                    f"{label} file must have exactly one hard link: {entry.name}"
                )
            try:
                member = os.open(entry.name, flags, dir_fd=directory_descriptor)
            except OSError as error:
                raise protocol.ValidationError(
                    f"cannot open {label} file {entry.name}: {error}"
                ) from error
            try:
                opened = os.fstat(member)
                if opened.st_nlink != 1:
                    raise protocol.ValidationError(
                        f"{label} file must have exactly one hard link: {entry.name}"
                    )
                if (opened.st_dev, opened.st_ino) != (metadata.st_dev, metadata.st_ino):
                    raise protocol.ValidationError(
                        f"{label} file changed before fsync: {entry.name}"
                    )
                os.fsync(member)
                current = os.stat(
                    entry.name,
                    dir_fd=directory_descriptor,
                    follow_symlinks=False,
                )
                if current.st_nlink != 1:
                    raise protocol.ValidationError(
                        f"{label} file must have exactly one hard link: {entry.name}"
                    )
                if ls_receipts._metadata_tuple(opened) != ls_receipts._metadata_tuple(
                    current
                ):
                    raise protocol.ValidationError(
                        f"{label} file changed during fsync: {entry.name}"
                    )
            except OSError as error:
                raise protocol.ValidationError(
                    f"cannot fsync {label} file {entry.name}: {error}"
                ) from error
            finally:
                os.close(member)
        else:
            raise protocol.ValidationError(
                f"{label} contains an unsafe entry: {entry.name}"
            )
    try:
        os.fsync(directory_descriptor)
    except OSError as error:
        raise protocol.ValidationError(f"cannot fsync {label}: {error}") from error


def _fsync_pinned_directory(
    directory: ls_receipts._PinnedDirectory, label: str
) -> None:
    validation_error: BaseException | None = None
    try:
        ls_receipts._require_pinned_directory(directory, label)
    except BaseException as error:
        validation_error = error
    try:
        os.fsync(directory.descriptor)
    except OSError as error:
        if validation_error is not None:
            raise PublicationCleanupError(
                validation_error, (error,)
            ) from validation_error
        raise protocol.ValidationError(f"cannot fsync {label}: {error}") from error
    if validation_error is not None:
        raise validation_error


def _rename_directory_no_replace(
    source_parent: ls_receipts._PinnedDirectory,
    source: ls_receipts._PinnedDirectory,
    destination_parent: ls_receipts._PinnedDirectory,
    destination_name: str,
) -> ls_receipts._PinnedDirectory:
    """Atomically move a pinned stage to an absent destination name."""

    if (
        source.parent_descriptor != source_parent.descriptor
        or source.entry_name is None
    ):
        raise protocol.ValidationError(
            "staged local formalization bundle has an invalid parent"
        )
    ls_receipts._safe_entry_name(source.entry_name, "local evidence stage")
    ls_receipts._safe_entry_name(destination_name, "local evidence destination")
    if not source.entry_name.startswith(STAGE_PREFIX):
        raise protocol.ValidationError("local evidence stage name is unsafe")
    if destination_name != DESTINATION_NAME:
        raise protocol.ValidationError("local evidence destination name is unsafe")
    ls_receipts._require_pinned_directory(
        source_parent, "local formalization staging parent"
    )
    ls_receipts._require_pinned_directory(source, "staged local formalization bundle")
    ls_receipts._require_pinned_directory(
        destination_parent, "local formalization publication parent"
    )
    if source_parent.device != destination_parent.device:
        raise protocol.ValidationError(
            "local formalization stage and destination must share a filesystem"
        )

    source_bytes = os.fsencode(source.entry_name)
    destination_bytes = os.fsencode(destination_name)
    if b"\0" in source_bytes or b"\0" in destination_bytes:
        raise protocol.ValidationError(
            "local formalization publication names must not contain NUL bytes"
        )
    library = ctypes.CDLL(None, use_errno=True)
    system = platform.system()
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
            source_parent.descriptor,
            source_bytes,
            destination_parent.descriptor,
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
            source_parent.descriptor,
            source_bytes,
            destination_parent.descriptor,
            destination_bytes,
            LINUX_RENAME_NOREPLACE,
        )
    else:
        raise protocol.ValidationError(
            f"atomic local formalization publication is unsupported on {system}"
        )
    if result == 0:
        return ls_receipts._PinnedDirectory(
            path=destination_parent.path / destination_name,
            descriptor=source.descriptor,
            device=source.device,
            inode=source.inode,
            parent_descriptor=destination_parent.descriptor,
            entry_name=destination_name,
        )
    error_number = ctypes.get_errno()
    if error_number in (errno.EEXIST, errno.ENOTEMPTY):
        raise protocol.ValidationError(
            "local formalization destination appeared during publication and was not "
            f"overwritten: {destination_parent.path / destination_name}"
        )
    raise protocol.ValidationError(
        "cannot atomically publish local formalization bundle: "
        + os.strerror(error_number)
    )


def _require_entry_absent(
    parent: ls_receipts._PinnedDirectory, name: str, label: str
) -> None:
    ls_receipts._safe_entry_name(name, label)
    ls_receipts._require_pinned_directory(parent, f"{label} parent")
    try:
        os.stat(name, dir_fd=parent.descriptor, follow_symlinks=False)
    except FileNotFoundError:
        return
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    raise protocol.ValidationError(f"{label} already exists: {parent.path / name}")


def _require_absent(path: Path, label: str) -> None:
    try:
        path.lstat()
    except FileNotFoundError:
        return
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    raise protocol.ValidationError(f"{label} already exists: {path}")


def _ensure_directory(path: Path, label: str) -> None:
    ls_receipts._ensure_directory(path, label)


def _absolute_normalized(path: Path) -> Path:
    return Path(os.path.abspath(os.fspath(path.expanduser())))


def _remove_stage(stage: Path | None, parent: Path) -> None:
    if stage is None:
        return
    if stage.parent != parent or not stage.name.startswith(STAGE_PREFIX):
        raise protocol.ValidationError("refusing to remove unsafe local evidence stage")
    pinned = _PINNED_STAGE_CLEANUP.get()
    if pinned is not None:
        pinned_parent, pinned_stage = pinned
        if pinned_parent.path != parent or pinned_stage.path != stage:
            raise protocol.ValidationError(
                "pinned local evidence stage does not match cleanup path"
            )
        _remove_stage_at(pinned_parent, pinned_stage)
        return
    raise protocol.ValidationError(
        "refusing to remove local evidence stage without pinned descriptors"
    )


def _remove_stage_at(
    parent: ls_receipts._PinnedDirectory, stage: ls_receipts._PinnedDirectory
) -> None:
    if (
        stage.parent_descriptor != parent.descriptor
        or stage.entry_name is None
        or not stage.entry_name.startswith(STAGE_PREFIX)
    ):
        raise protocol.ValidationError(
            "refusing to remove unsafe pinned local evidence stage"
        )
    ls_receipts._require_descriptor_identity(
        parent, "local formalization staging parent"
    )
    ls_receipts._require_descriptor_identity(stage, "staged local formalization bundle")
    _remove_tree_contents_at(stage.descriptor, [0])
    try:
        metadata = os.stat(
            stage.entry_name,
            dir_fd=parent.descriptor,
            follow_symlinks=False,
        )
    except FileNotFoundError:
        return
    if (
        stat.S_ISLNK(metadata.st_mode)
        or not stat.S_ISDIR(metadata.st_mode)
        or (metadata.st_dev, metadata.st_ino) != (stage.device, stage.inode)
    ):
        raise protocol.ValidationError(
            "staged local formalization bundle identity changed before removal"
        )
    os.rmdir(stage.entry_name, dir_fd=parent.descriptor)


def _remove_tree_contents_at(
    directory_descriptor: int, entries_seen: list[int] | None = None
) -> None:
    if entries_seen is None:
        entries_seen = [0]
    entries = _bounded_sorted_scandir_at(
        directory_descriptor,
        MAX_BUNDLE_ENTRIES - entries_seen[0],
        "local evidence cleanup",
    )
    for entry in entries:
        entries_seen[0] += 1
        metadata = entry.stat(follow_symlinks=False)
        if stat.S_ISLNK(metadata.st_mode) or stat.S_ISREG(metadata.st_mode):
            os.unlink(entry.name, dir_fd=directory_descriptor)
        elif stat.S_ISDIR(metadata.st_mode):
            flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
            if hasattr(os, "O_CLOEXEC"):
                flags |= os.O_CLOEXEC
            child = os.open(entry.name, flags, dir_fd=directory_descriptor)
            try:
                opened = os.fstat(child)
                if (opened.st_dev, opened.st_ino) != (metadata.st_dev, metadata.st_ino):
                    raise protocol.ValidationError(
                        "local evidence stage changed during cleanup"
                    )
                _remove_tree_contents_at(child, entries_seen)
                current = os.stat(
                    entry.name,
                    dir_fd=directory_descriptor,
                    follow_symlinks=False,
                )
                if (current.st_dev, current.st_ino) != (opened.st_dev, opened.st_ino):
                    raise protocol.ValidationError(
                        "local evidence stage changed during cleanup"
                    )
            finally:
                os.close(child)
            os.rmdir(entry.name, dir_fd=directory_descriptor)
        else:
            raise protocol.ValidationError(
                "local evidence stage contains an unsafe cleanup entry"
            )


def _bounded_sorted_scandir_at(
    directory_descriptor: int, maximum: int, label: str
) -> list[os.DirEntry[str]]:
    entries: list[os.DirEntry[str]] = []
    try:
        with os.scandir(directory_descriptor) as iterator:
            for entry in iterator:
                if len(entries) >= maximum:
                    raise protocol.ValidationError(f"{label} exceeds entry cap")
                entries.append(entry)
    except protocol.ValidationError:
        raise
    except OSError as error:
        raise protocol.ValidationError(f"cannot scan {label}: {error}") from error
    entries.sort(key=lambda item: item.name)
    return entries


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


def _bounded_lean_source_paths(root: Path) -> list[Path]:
    paths: list[Path] = []
    pending = [root]
    entries_seen = 0
    while pending:
        directory = pending.pop()
        child_directories: list[Path] = []
        for entry in _bounded_sorted_scandir(
            directory,
            MAX_SOURCE_SCAN_ENTRIES - entries_seen,
            "active Crouzeix source tree",
        ):
            entries_seen += 1
            metadata = entry.stat(follow_symlinks=False)
            if stat.S_ISLNK(metadata.st_mode):
                raise protocol.ValidationError(
                    f"active Crouzeix source tree contains a symlink: {entry.path}"
                )
            if stat.S_ISDIR(metadata.st_mode):
                child_directories.append(Path(entry.path))
            elif stat.S_ISREG(metadata.st_mode) and entry.name.endswith(".lean"):
                if len(paths) >= MAX_ACTIVE_SOURCE_FILES:
                    raise protocol.ValidationError(
                        "active Crouzeix source count exceeds safety cap"
                    )
                paths.append(Path(entry.path))
            elif not stat.S_ISREG(metadata.st_mode):
                raise protocol.ValidationError(
                    f"active Crouzeix source tree contains an unsafe entry: {entry.path}"
                )
        pending.extend(reversed(child_directories))
    return paths


def _open_directory_at(
    parent_descriptor: int, name: str, expected: os.stat_result, label: str
) -> int:
    flags = os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW
    if hasattr(os, "O_CLOEXEC"):
        flags |= os.O_CLOEXEC
    try:
        descriptor = os.open(name, flags, dir_fd=parent_descriptor)
    except OSError as error:
        raise protocol.ValidationError(f"cannot open {label}: {error}") from error
    observed = os.fstat(descriptor)
    if (observed.st_dev, observed.st_ino) != (expected.st_dev, expected.st_ino):
        os.close(descriptor)
        raise protocol.ValidationError(f"{label} changed while being opened")
    return descriptor
