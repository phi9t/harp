"""Read-only validation for the local Crouzeix formalization evidence bundle."""

from __future__ import annotations

import json
from dataclasses import dataclass
from pathlib import Path, PurePosixPath

if __package__:
    from . import local_formalization_evidence, ls_receipts, protocol, route_validation
else:  # pragma: no cover - direct script execution path
    import local_formalization_evidence  # type: ignore
    import ls_receipts  # type: ignore
    import protocol  # type: ignore
    import route_validation  # type: ignore


@dataclass(frozen=True)
class LocalBundleValidationResult:
    status: str
    formalization_ids: tuple[str, ...]
    route_ids: tuple[str, ...]
    manifest_sha256: str


@dataclass(frozen=True)
class _ArtifactPathBinding:
    role: str
    digest: str
    owner: str


@dataclass(frozen=True)
class _ArtifactDigestBinding:
    role: str
    owner: str


def validate_local_formalization_bundle(
    repository_root: Path,
) -> LocalBundleValidationResult:
    repository = Path(repository_root).resolve()
    repository_directory = ls_receipts._pin_directory(
        repository, "local formalization validator repository root"
    )
    try:
        evidence_root = ls_receipts._pin_directory_at(
            repository_directory,
            "evidence",
            repository / "evidence",
            "local formalization validator evidence root",
        )
        try:
            crouzeix_root = ls_receipts._pin_directory_at(
                evidence_root,
                local_formalization_evidence.EVIDENCE_ROOT_NAME,
                repository / "evidence" / local_formalization_evidence.EVIDENCE_ROOT_NAME,
                "local formalization validator crouzeix root",
            )
            try:
                bundle_root = ls_receipts._pin_directory_at(
                    crouzeix_root,
                    local_formalization_evidence.DESTINATION_NAME,
                    repository
                    / "evidence"
                    / local_formalization_evidence.EVIDENCE_ROOT_NAME
                    / local_formalization_evidence.DESTINATION_NAME,
                    "local formalization validator bundle root",
                )
                try:
                    observed_members = local_formalization_evidence._bundle_member_snapshots(
                        bundle_root.descriptor
                    )
                    manifest_bytes = observed_members[
                        local_formalization_evidence.MANIFEST_NAME
                    ]
                    manifest_sha256 = protocol.sha256_bytes(manifest_bytes)
                    header, rows = _parse_manifest(
                        repository
                        / "evidence"
                        / local_formalization_evidence.EVIDENCE_ROOT_NAME
                        / local_formalization_evidence.DESTINATION_NAME
                        / local_formalization_evidence.MANIFEST_NAME,
                        manifest_bytes,
                    )
                    if header != local_formalization_evidence.HEADER:
                        raise ValueError("local formalization manifest header mismatch")
                    if len(rows) != len(local_formalization_evidence.FORMALIZATIONS):
                        raise ValueError("local formalization manifest row count mismatch")

                    toolchain_bytes = _read_repository_file(
                        repository_directory,
                        ("formalization", "lean", "lean-toolchain"),
                        "Lean toolchain identity",
                    )
                    lake_manifest_bytes = _read_repository_file(
                        repository_directory,
                        ("formalization", "lean", "lake-manifest.json"),
                        "Lake dependency manifest",
                    )
                    expected_toolchain = toolchain_bytes.decode("utf-8")
                    if not expected_toolchain.endswith("\n"):
                        raise ValueError("Lean toolchain identity must be LF-terminated")
                    expected_toolchain = expected_toolchain[:-1]
                    expected_toolchain_sha256 = protocol.sha256_bytes(toolchain_bytes)
                    expected_lake_manifest_sha256 = protocol.sha256_bytes(
                        lake_manifest_bytes
                    )

                    provider_reports = local_formalization_evidence._provider_reports(
                        repository / "formalization/lean"
                    )
                    provider_bytes = {
                        route_id: _canonical_json_bytes(payload)
                        for route_id, payload in provider_reports.items()
                    }
                    routes = local_formalization_evidence._snapshot_route_evidence(
                        repository
                    )

                    formalization_ids: list[str] = []
                    route_ids: list[str] = []
                    aggregate_build: tuple[str, ...] | None = None
                    path_bindings: dict[str, _ArtifactPathBinding] = {}
                    digest_bindings: dict[str, _ArtifactDigestBinding] = {}
                    expected_specs = local_formalization_evidence.FORMALIZATIONS
                    for spec, row in zip(expected_specs, rows):
                        if len(row) != 29:
                            raise ValueError(
                                "local formalization manifest column count mismatch"
                            )
                        if tuple(row[:6]) != (
                            local_formalization_evidence.SCHEMA_VERSION,
                            spec.formalization_id,
                            spec.route_id,
                            spec.source_node_id,
                            spec.declaration_name,
                            spec.module_path,
                        ):
                            raise ValueError(
                                "local formalization manifest spec row mismatch"
                            )
                        if row[6] != _module_digest(
                            repository_directory, spec.module_path
                        ):
                            raise ValueError(
                                "local formalization module digest mismatch"
                            )
                        _register_artifact_binding(
                            path_bindings,
                            digest_bindings,
                            spec.module_path,
                            "module",
                            row[6],
                            f"route:{spec.route_id}:module:{spec.module_path}",
                        )
                        _validate_route_binding(
                            spec, row, routes[spec.route_id], observed_members
                        )
                        for path_index, digest_index, role in (
                            (7, 8, "route_manifest"),
                            (9, 10, "route_receipt"),
                            (11, 12, "route_review"),
                        ):
                            _register_artifact_binding(
                                path_bindings,
                                digest_bindings,
                                row[path_index],
                                role,
                                row[digest_index],
                                f"route:{spec.route_id}",
                            )
                        aggregate_fields = tuple(row[13:19])
                        if aggregate_build is None:
                            aggregate_build = aggregate_fields
                        elif aggregate_build != aggregate_fields:
                            raise ValueError(
                                "aggregate build artifact fields changed across rows"
                            )
                        for path_index, digest_index, role, owner in (
                            (13, 14, "build_command", "aggregate-build"),
                            (15, 16, "build_stdout", "aggregate-build"),
                            (17, 18, "build_stderr", "aggregate-build"),
                            (
                                19,
                                20,
                                "axiom_audit",
                                f"formalization:{spec.formalization_id}",
                            ),
                            (
                                23,
                                24,
                                "provider_independence",
                                f"route:{spec.route_id}",
                            ),
                        ):
                            _register_artifact_binding(
                                path_bindings,
                                digest_bindings,
                                row[path_index],
                                role,
                                row[digest_index],
                                owner,
                            )
                        _validate_build_artifacts(repository, row, observed_members)
                        _validate_axiom_artifact(repository, spec, row, observed_members)
                        _validate_provider_artifact(
                            repository,
                            spec,
                            row,
                            provider_bytes[spec.route_id],
                            observed_members,
                        )
                        if row[21] != "Classical.choice,Quot.sound,propext":
                            raise ValueError("allowed_axioms policy mismatch")
                        if row[22] != "Classical.choice,Quot.sound,propext":
                            raise ValueError("observed_axioms policy mismatch")
                        if row[25] != expected_toolchain:
                            raise ValueError("Lean toolchain identity mismatch")
                        if row[26] != expected_toolchain_sha256:
                            raise ValueError("Lean toolchain digest mismatch")
                        if row[27] != expected_lake_manifest_sha256:
                            raise ValueError("Lake manifest digest mismatch")
                        if row[28] != "passed":
                            raise ValueError(
                                "local formalization row status must be passed"
                            )
                        formalization_ids.append(spec.formalization_id)
                        if spec.route_id not in route_ids:
                            route_ids.append(spec.route_id)

                    return LocalBundleValidationResult(
                        status="passed",
                        formalization_ids=tuple(formalization_ids),
                        route_ids=tuple(route_ids),
                        manifest_sha256=manifest_sha256,
                    )
                finally:
                    ls_receipts._close_pinned_directory(bundle_root)
            finally:
                ls_receipts._close_pinned_directory(crouzeix_root)
        finally:
            ls_receipts._close_pinned_directory(evidence_root)
    finally:
        ls_receipts._close_pinned_directory(repository_directory)


def _parse_manifest(path: Path, data: bytes) -> tuple[str, list[list[str]]]:
    try:
        lines = data.decode("utf-8").splitlines()
    except UnicodeDecodeError as error:
        raise ValueError(f"local formalization manifest is not UTF-8: {path}") from error
    if not lines:
        raise ValueError("local formalization manifest is empty")
    return lines[0], [line.split("\t") for line in lines[1:]]


def _module_digest(
    repository_directory: ls_receipts._PinnedDirectory, relative_path: str
) -> str:
    path = PurePosixPath(relative_path)
    if path.is_absolute() or not path.parts or ".." in path.parts:
        raise ValueError("formalization module path is unsafe")
    snapshot = ls_receipts._read_file_snapshot_relative_at(
        repository_directory,
        path.parts,
        f"formalization module {relative_path}",
        local_formalization_evidence.MAX_MEMBER_BYTES,
    )
    return protocol.sha256_bytes(snapshot.data)


def _validate_route_binding(
    spec: local_formalization_evidence.FormalizationSpec,
    row: list[str],
    route: local_formalization_evidence._RouteEvidence,
    observed_members: dict[str, bytes],
) -> None:
    expected = (
        (f"routes/{spec.route_id}.manifest.json", route.manifest.data),
        (f"routes/{spec.route_id}.receipt.json", route.receipt.data),
        (f"routes/{spec.route_id}.review.json", route.review.data),
    )
    for (member, data), path_index in zip(expected, (7, 9, 11)):
        expected_path = local_formalization_evidence._published_path(member)
        if row[path_index] != expected_path:
            raise ValueError(f"route artifact path mismatch for {spec.route_id}")
        if (
            _member_bytes(row[path_index], observed_members) != data
            or row[path_index + 1] != protocol.sha256_bytes(data)
        ):
            raise ValueError(f"route artifact digest mismatch for {spec.route_id}")
    manifest = json.loads(route.manifest.data.decode("utf-8"))
    if (
        manifest.get("route_id") != spec.route_id
        or manifest.get("receipt_path") != route.receipt_path
        or manifest.get("receipt_sha256")
        != protocol.sha256_bytes(route.receipt.data)
        or manifest.get("review_path") != route.review_path
        or manifest.get("review_sha256") != protocol.sha256_bytes(route.review.data)
    ):
        raise ValueError(f"route publication binding mismatch for {spec.route_id}")


def _register_artifact_binding(
    path_bindings: dict[str, _ArtifactPathBinding],
    digest_bindings: dict[str, _ArtifactDigestBinding],
    path: str,
    role: str,
    digest: str,
    owner: str,
) -> None:
    path_binding = _ArtifactPathBinding(role=role, digest=digest, owner=owner)
    existing_path = path_bindings.get(path)
    if existing_path is not None and existing_path != path_binding:
        raise ValueError("artifact path aliases unrelated role or owner")
    path_bindings.setdefault(path, path_binding)

    digest_binding = _ArtifactDigestBinding(role=role, owner=owner)
    existing_digest = digest_bindings.get(digest)
    same_aggregate = (
        existing_digest is not None
        and existing_digest.owner == "aggregate-build"
        and owner == "aggregate-build"
    )
    same_route_identity = (
        existing_digest is not None
        and existing_digest.owner == owner
        and role in {"route_manifest", "route_receipt", "route_review", "provider_independence"}
        and existing_digest.role in {"route_manifest", "route_receipt", "route_review", "provider_independence"}
        and owner.startswith("route:")
    )
    if (
        existing_digest is not None
        and existing_digest != digest_binding
        and not same_aggregate
        and not same_route_identity
    ):
        raise ValueError("artifact digest aliases unrelated role or owner")
    digest_bindings.setdefault(digest, digest_binding)


def _validate_build_artifacts(
    root: Path, row: list[str], observed_members: dict[str, bytes]
) -> None:
    for path_index, digest_index in ((13, 14), (15, 16), (17, 18)):
        _validate_digest_bound_path(root, row[path_index], row[digest_index], observed_members)
    command = json.loads(_member_bytes(row[13], observed_members).decode("utf-8"))
    expected_fields = {
        "schema_version",
        "argv",
        "cwd",
        "lean_toolchain",
        "lean_toolchain_sha256",
        "lake_manifest_sha256",
        "wrapper_sha256",
        "lakefile_sha256",
        "active_source_closure_sha256",
        "dependency_cache_metadata_sha256",
        "required_mathlib_artifacts_sha256",
        "ls_graph_sha256",
        "exit_code",
        "status",
        "stdout_path",
        "stdout_sha256",
        "stderr_path",
        "stderr_sha256",
    }
    if set(command) != expected_fields:
        raise ValueError("local build command schema mismatch")
    if command["schema_version"] != local_formalization_evidence.BUILD_SCHEMA_VERSION:
        raise ValueError("local build command schema version mismatch")


def _validate_axiom_artifact(
    root: Path,
    spec: local_formalization_evidence.FormalizationSpec,
    row: list[str],
    observed_members: dict[str, bytes],
) -> None:
    _validate_digest_bound_path(root, row[19], row[20], observed_members)
    parsed = ls_receipts._parse_axiom_audit(
        _member_bytes(row[19], observed_members),
        spec.declaration_name,
    )
    if parsed != local_formalization_evidence.ALLOWED_AXIOMS:
        raise ValueError("axiom audit content mismatch")


def _validate_provider_artifact(
    root: Path,
    spec: local_formalization_evidence.FormalizationSpec,
    row: list[str],
    expected_bytes: bytes,
    observed_members: dict[str, bytes],
) -> None:
    _validate_digest_bound_path(root, row[23], row[24], observed_members)
    actual = _member_bytes(row[23], observed_members)
    if actual != expected_bytes:
        raise ValueError(f"provider report mismatch for {spec.route_id}")


def _validate_digest_bound_path(
    root: Path,
    relative_path: str,
    expected_sha256: str,
    observed_members: dict[str, bytes],
) -> None:
    path = PurePosixPath(relative_path)
    if path.is_absolute() or ".." in path.parts or not path.parts:
        raise ValueError("artifact path is unsafe")
    actual = protocol.sha256_bytes(_member_bytes(relative_path, observed_members))
    if actual != expected_sha256:
        raise ValueError(f"digest mismatch for {relative_path}")


def _member_bytes(relative_path: str, observed_members: dict[str, bytes]) -> bytes:
    try:
        return observed_members[
            str(
                PurePosixPath(relative_path).relative_to(
                    PurePosixPath(
                        "evidence",
                        local_formalization_evidence.EVIDENCE_ROOT_NAME,
                        local_formalization_evidence.DESTINATION_NAME,
                    )
                )
            )
        ]
    except (KeyError, ValueError) as error:
        raise ValueError(f"artifact path escapes local bundle: {relative_path}") from error


def _read_repository_file(
    repository_directory: ls_receipts._PinnedDirectory,
    parts: tuple[str, ...],
    label: str,
) -> bytes:
    snapshot = ls_receipts._read_file_snapshot_relative_at(
        repository_directory,
        parts,
        label,
        local_formalization_evidence.MAX_MEMBER_BYTES,
    )
    return snapshot.data


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
