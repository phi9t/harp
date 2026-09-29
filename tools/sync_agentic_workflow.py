#!/usr/bin/env python3
"""Synchronize Corenius's agentic-engineering policy across associated repos."""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import TextIO


START_MARKER = "<!-- ultron-agentic-workflow:start -->"
END_MARKER = "<!-- ultron-agentic-workflow:end -->"
VCS_METADATA = frozenset({".git", ".hg", ".svn"})
MANAGED_BLOCK = f"""{START_MARKER}
## Agentic engineering workflow

**Mandatory:** Read and follow `CONSTITUTION.md` before acting. Before planning,
building, fixing, or changing code, read and follow
`docs/agents/agentic-engineering.md`. Direct user instructions and more specific
repository guidance take precedence.
{END_MARKER}"""


class SyncError(Exception):
    """An unsafe or invalid synchronization request."""


@dataclass(frozen=True)
class Operation:
    label: str
    destination: Path
    expected: bytes
    managed_mode: int | None = None
    default_mode: int = 0o644


@dataclass(frozen=True)
class BranchOperation:
    label: str
    repository: Path
    branch: str
    base_branch: str


def _branch_name(value: object, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise SyncError(f"{label} must be a non-empty string")
    name = value.strip()
    if name != value:
        raise SyncError(f"{label} must not have surrounding whitespace: {value!r}")
    if name.startswith("-") or name.startswith("/") or name.endswith("/"):
        raise SyncError(f"{label} is not a valid branch name: {value!r}")
    if ".." in name or "\\" in name or " " in name:
        raise SyncError(f"{label} is not a valid branch name: {value!r}")
    for char in name:
        if ord(char) < 0x20 or char in "~^:?*[":
            raise SyncError(f"{label} is not a valid branch name: {value!r}")
    return name


def _relative_path(value: object, label: str) -> PurePosixPath:
    if not isinstance(value, str) or not value.strip():
        raise SyncError(f"{label} must be a non-empty relative path")
    path = PurePosixPath(value)
    if path.is_absolute() or ".." in path.parts:
        raise SyncError(f"{label} points outside the workspace: {value!r}")
    if VCS_METADATA.intersection(path.parts):
        raise SyncError(f"{label} targets VCS metadata: {value!r}")
    return path


def _within(root: Path, relative: PurePosixPath, label: str) -> Path:
    candidate = root.joinpath(*relative.parts)
    resolved_root = root.resolve()
    resolved_candidate = candidate.resolve(strict=False)
    if not resolved_candidate.is_relative_to(resolved_root):
        raise SyncError(f"{label} points outside the workspace: {relative}")
    return candidate


def _reject_symlinked_path(
    root: Path, destination: Path, label: str, *, include_leaf: bool = True
) -> None:
    relative = destination.relative_to(root)
    parts = relative.parts if include_leaf else relative.parts[:-1]
    current = root
    for part in parts:
        current /= part
        if current.is_symlink():
            raise SyncError(f"{label} contains a symlink: {current}")


def _load_manifest(path: Path) -> dict:
    try:
        manifest = json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as error:
        raise SyncError(f"cannot read manifest {path}: {error}") from error
    if not isinstance(manifest, dict):
        raise SyncError("manifest root must be an object")
    return manifest


def _validate_manifest(
    manifest: dict,
) -> tuple[str, list[str], list[str], list[dict], list[dict], list[dict], str | None]:
    source = manifest.get("source_repository")
    repositories = manifest.get("repositories")
    policy_files = manifest.get("managed_policy_files")
    tracker_files = manifest.get("local_tracker_files")
    required_symlinks = manifest.get("required_symlinks", [])
    required_content = manifest.get("required_content", [])
    mainline_branch = manifest.get("mainline_branch")
    if not isinstance(source, str) or not source:
        raise SyncError("source_repository must be a non-empty string")
    if not isinstance(repositories, list) or not repositories:
        raise SyncError("repositories must be a non-empty list")
    if not isinstance(policy_files, list) or not policy_files:
        raise SyncError("managed_policy_files must be a non-empty list")
    if not isinstance(tracker_files, list):
        raise SyncError("local_tracker_files must be a list")
    if not isinstance(required_symlinks, list):
        raise SyncError("required_symlinks must be a list")
    if not isinstance(required_content, list):
        raise SyncError("required_content must be a list")
    if mainline_branch is not None:
        mainline_branch = _branch_name(mainline_branch, "mainline_branch")

    names: list[str] = []
    for entry in repositories:
        if not isinstance(entry, dict):
            raise SyncError("every repository entry must be an object")
        name_path = _relative_path(entry.get("name"), "repository name")
        if len(name_path.parts) != 1:
            raise SyncError(
                f"repository name points outside the workspace: {entry.get('name')!r}"
            )
        name = name_path.parts[0]
        if name in names:
            raise SyncError(f"duplicate repository entry: {name}")
        names.append(name)
        instruction_files = entry.get("instruction_files")
        if not isinstance(instruction_files, list) or not instruction_files:
            raise SyncError(f"{name}: instruction_files must be a non-empty list")
        for target in instruction_files:
            _relative_path(target, f"{name} instruction file")
        if not isinstance(entry.get("install_local_tracker"), bool):
            raise SyncError(f"{name}: install_local_tracker must be boolean")
        if "base_branch" in entry:
            _branch_name(entry.get("base_branch"), f"{name} base_branch")

    if source not in names:
        raise SyncError(f"source repository {source!r} is not in repositories")
    for value in policy_files:
        _relative_path(value, "managed policy file")
    for value in tracker_files:
        _relative_path(value, "local tracker file")
    for requirement in required_symlinks:
        if not isinstance(requirement, dict):
            raise SyncError("every required symlink must be an object")
        path = _relative_path(requirement.get("path"), "required symlink path")
        target = requirement.get("target")
        if len(path.parts) < 2 or path.parts[0] not in names:
            raise SyncError(f"required symlink is outside a managed repository: {path}")
        if not isinstance(target, str) or not target:
            raise SyncError(f"required symlink target must be non-empty: {path}")
    for requirement in required_content:
        if not isinstance(requirement, dict):
            raise SyncError("every required content entry must be an object")
        path = _relative_path(requirement.get("path"), "required content path")
        snippets = requirement.get("snippets")
        if len(path.parts) < 2 or path.parts[0] not in names:
            raise SyncError(f"required content is outside a managed repository: {path}")
        if (
            not isinstance(snippets, list)
            or not snippets
            or not all(isinstance(snippet, str) and snippet for snippet in snippets)
        ):
            raise SyncError(f"required content snippets must be non-empty: {path}")
    return (
        source,
        policy_files,
        tracker_files,
        repositories,
        required_symlinks,
        required_content,
        mainline_branch,
    )


def _render_instructions(current: str) -> str:
    start_count = current.count(START_MARKER)
    end_count = current.count(END_MARKER)
    if start_count == 0 and end_count == 0:
        if not current:
            return f"{MANAGED_BLOCK}\n"
        separator = "\n" if current.endswith("\n") else "\n\n"
        if current.endswith("\n\n"):
            separator = ""
        return f"{current}{separator}{MANAGED_BLOCK}\n"
    if start_count != 1 or end_count != 1:
        raise SyncError("malformed managed block: duplicate or unbalanced markers")
    start = current.index(START_MARKER)
    end = current.index(END_MARKER)
    if end < start:
        raise SyncError("malformed managed block: end marker precedes start marker")
    end += len(END_MARKER)
    return f"{current[:start]}{MANAGED_BLOCK}{current[end:]}"


def _build_operations(
    workspace_root: Path, manifest_path: Path
) -> tuple[list[Operation], list[BranchOperation]]:
    workspace_root = workspace_root.resolve()
    if not workspace_root.is_dir():
        raise SyncError(f"workspace root is missing: {workspace_root}")
    manifest = _load_manifest(manifest_path)
    (
        source_name,
        policy_files,
        tracker_files,
        repositories,
        required_symlinks,
        required_content,
        mainline_branch,
    ) = _validate_manifest(manifest)
    for requirement in required_symlinks:
        relative = _relative_path(requirement["path"], "required symlink path")
        symlink = _within(workspace_root, relative, "required symlink")
        _reject_symlinked_path(
            workspace_root,
            symlink,
            "required symlink path",
            include_leaf=False,
        )
        if not symlink.is_symlink():
            raise SyncError(f"required symlink is missing: {relative}")
        actual_target = os.readlink(symlink)
        if actual_target != requirement["target"]:
            raise SyncError(
                "unexpected symlink target for "
                f"{relative}: expected {requirement['target']!r}, got {actual_target!r}"
            )
    for requirement in required_content:
        relative = _relative_path(requirement["path"], "required content path")
        content_file = _within(workspace_root, relative, "required content")
        _reject_symlinked_path(
            workspace_root, content_file, "required content path"
        )
        if not content_file.is_file() or content_file.is_symlink():
            raise SyncError(f"required content file is missing or unsafe: {relative}")
        content = content_file.read_text()
        missing = [
            snippet for snippet in requirement["snippets"] if snippet not in content
        ]
        if missing:
            raise SyncError(
                f"required content is missing from {relative}: {missing!r}"
            )
    source_repo = _within(
        workspace_root, PurePosixPath(source_name), "source repository"
    )
    if source_repo.is_symlink():
        raise SyncError(f"source repository root must not be a symlink: {source_repo}")
    if not source_repo.is_dir():
        raise SyncError(f"source repository is missing: {source_repo}")

    canonical: dict[str, tuple[bytes, int]] = {}
    for relative_value in [*policy_files, *tracker_files]:
        relative = _relative_path(relative_value, "canonical file")
        source_file = _within(source_repo, relative, "canonical file")
        _reject_symlinked_path(source_repo, source_file, "canonical path")
        if not source_file.is_file() or source_file.is_symlink():
            raise SyncError(f"canonical file is missing or unsafe: {source_file}")
        canonical[relative_value] = (
            source_file.read_bytes(),
            0o644,
        )

    operations: list[Operation] = []
    branch_operations: list[BranchOperation] = []
    for entry in repositories:
        name = entry["name"]
        repository = _within(
            workspace_root, PurePosixPath(name), f"repository {name}"
        )
        if repository.is_symlink():
            raise SyncError(f"{name}: repository root must not be a symlink")
        if not repository.is_dir():
            raise SyncError(f"{name}: repository is missing: {repository}")

        if mainline_branch is not None:
            base_branch = entry.get("base_branch")
            if base_branch is None:
                base_branch = _detect_default_branch(repository, name)
            else:
                base_branch = _branch_name(base_branch, f"{name} base_branch")
            branch_operations.append(
                BranchOperation(
                    label=f"{name}/{mainline_branch}",
                    repository=repository,
                    branch=mainline_branch,
                    base_branch=base_branch,
                )
            )

        managed_files = list(policy_files)
        if entry["install_local_tracker"]:
            managed_files.extend(tracker_files)
        for relative_value in managed_files:
            relative = _relative_path(relative_value, f"{name} managed file")
            destination = _within(repository, relative, f"{name} managed file")
            _reject_symlinked_path(repository, destination, f"{name} managed path")
            if destination.is_symlink() or (
                destination.exists() and not destination.is_file()
            ):
                raise SyncError(f"{name}: managed destination is unsafe: {destination}")
            content, managed_mode = canonical[relative_value]
            operations.append(
                Operation(
                    label=f"{name}/{relative_value}",
                    destination=destination,
                    expected=content,
                    managed_mode=managed_mode,
                )
            )

        for instruction_value in entry["instruction_files"]:
            relative = _relative_path(instruction_value, f"{name} instruction file")
            destination = _within(repository, relative, f"{name} instruction file")
            _reject_symlinked_path(
                repository, destination, f"{name} instruction path"
            )
            if destination.is_symlink() or (
                destination.exists() and not destination.is_file()
            ):
                raise SyncError(
                    f"{name}: instruction destination is unsafe: {destination}"
                )
            current = destination.read_text() if destination.exists() else ""
            try:
                expected = _render_instructions(current).encode()
            except SyncError as error:
                raise SyncError(f"{name}/{instruction_value}: {error}") from error
            operations.append(
                Operation(
                    label=f"{name}/{instruction_value}",
                    destination=destination,
                    expected=expected,
                )
            )
    return operations, branch_operations


def _git(repository: Path, *args: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["git", *args],
        cwd=repository,
        capture_output=True,
        text=True,
        check=False,
    )


def _git_dir(repository: Path, name: str) -> None:
    result = _git(repository, "rev-parse", "--git-dir")
    if result.returncode != 0:
        raise SyncError(f"{name}: not a git repository: {repository}")


def _detect_default_branch(repository: Path, name: str) -> str:
    for candidate in ("main", "master"):
        if _branch_exists(repository, candidate):
            return candidate
    raise SyncError(
        f"{name}: cannot detect default branch (no local main or master)"
    )


def _branch_exists(repository: Path, branch: str) -> bool:
    result = _git(
        repository, "show-ref", "--verify", "--quiet", f"refs/heads/{branch}"
    )
    return result.returncode == 0


def _atomic_write(operation: Operation) -> None:
    destination = operation.destination
    destination.parent.mkdir(parents=True, exist_ok=True)
    file_descriptor, temporary_name = tempfile.mkstemp(
        prefix=f".{destination.name}.", dir=destination.parent
    )
    try:
        with os.fdopen(file_descriptor, "wb") as temporary:
            temporary.write(operation.expected)
            temporary.flush()
            os.fsync(temporary.fileno())
        if operation.managed_mode is not None:
            os.chmod(temporary_name, operation.managed_mode)
        elif destination.exists():
            shutil.copymode(destination, temporary_name, follow_symlinks=False)
        else:
            os.chmod(temporary_name, operation.default_mode)
        os.replace(temporary_name, destination)
    except BaseException:
        try:
            os.unlink(temporary_name)
        except FileNotFoundError:
            pass
        raise


def run(
    *, mode: str, workspace_root: Path, manifest_path: Path, output: TextIO
) -> int:
    if mode not in {"check", "apply"}:
        raise ValueError(f"unsupported mode: {mode}")
    try:
        operations, branch_operations = _build_operations(
            workspace_root, manifest_path
        )
    except SyncError as error:
        print(f"ERROR {error}", file=output)
        return 2

    drift: list[Operation] = []
    for operation in operations:
        destination_exists = operation.destination.exists()
        current = (
            operation.destination.read_bytes() if destination_exists else None
        )
        mode_differs = (
            destination_exists
            and operation.managed_mode is not None
            and operation.destination.stat().st_mode & 0o777 != operation.managed_mode
        )
        if current != operation.expected or mode_differs:
            drift.append(operation)
            state = "DRIFT" if current is not None else "MISSING"
            print(f"{state} {operation.label}", file=output)

    branch_drift: list[BranchOperation] = []
    try:
        for branch_operation in branch_operations:
            _git_dir(branch_operation.repository, branch_operation.label)
            if not _branch_exists(
                branch_operation.repository, branch_operation.branch
            ):
                branch_drift.append(branch_operation)
                print(f"MISSING-BRANCH {branch_operation.label}", file=output)
    except SyncError as error:
        print(f"ERROR {error}", file=output)
        return 2

    total = len(operations) + len(branch_operations)
    if mode == "check":
        drift_count = len(drift) + len(branch_drift)
        if drift_count:
            print(f"CHECK FAILED: {drift_count} managed path(s) differ", file=output)
            return 1
        print(f"CHECK OK: {total} managed path(s)", file=output)
        return 0

    for operation in drift:
        _atomic_write(operation)
        print(f"UPDATED {operation.label}", file=output)
    try:
        for branch_operation in branch_drift:
            if not _branch_exists(
                branch_operation.repository, branch_operation.base_branch
            ):
                raise SyncError(
                    f"{branch_operation.label}: base branch "
                    f"{branch_operation.base_branch!r} is missing"
                )
            result = _git(
                branch_operation.repository,
                "branch",
                branch_operation.branch,
                branch_operation.base_branch,
            )
            if result.returncode != 0:
                raise SyncError(
                    f"{branch_operation.label}: cannot create branch: "
                    f"{result.stderr.strip()}"
                )
            print(f"CREATED-BRANCH {branch_operation.label}", file=output)
    except SyncError as error:
        print(f"ERROR {error}", file=output)
        return 2
    updated = len(drift) + len(branch_drift)
    print(f"APPLY OK: {updated} path(s) updated", file=output)
    return 0


def main(argv: list[str] | None = None) -> int:
    script = Path(__file__).resolve()
    default_source = script.parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="report drift")
    mode.add_argument("--apply", action="store_true", help="repair drift")
    parser.add_argument(
        "--workspace-root",
        type=Path,
        default=default_source.parent,
        help="parent directory containing the managed repositories",
    )
    parser.add_argument(
        "--manifest",
        type=Path,
        default=default_source / "docs/agents/agentic-workflow-repos.json",
        help="repository manifest",
    )
    arguments = parser.parse_args(argv)
    return run(
        mode="apply" if arguments.apply else "check",
        workspace_root=arguments.workspace_root,
        manifest_path=arguments.manifest,
        output=sys.stdout,
    )


if __name__ == "__main__":
    raise SystemExit(main())
