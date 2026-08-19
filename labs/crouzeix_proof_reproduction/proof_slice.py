from __future__ import annotations

import argparse
import json
import os
import signal
import re
import shutil
import stat
import subprocess
import threading
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Callable, Mapping

import formal_target
import jin_validation
import protocol


SCHEMA_VERSION = "crouzeix-jin-proof-slice-descriptor/v1"
ROUTE_ID = "jin"
TERMINAL_ROW_ID = "jin-terminal-crouzeix"
TERMINAL_REFERENCE_IMPORTS = frozenset({"CrouzeixConjecture.FinalTheorems"})
BUILD_TARGET = "module/Slice.lean"
MAX_ALLOWED_IMPORTS = 32
MAX_DEPENDENCY_RECEIPTS = 64
MAX_ALLOWED_AXIOMS = 32
MAX_REJECT_TOKENS = 8
MAX_TIMEOUT_SECONDS = 86_400
MAX_OUTPUT_BYTES = 16 * 1024 * 1024
PIPE_DRAIN_GRACE_SECONDS = 1.0
DEFAULT_SOURCE_MAP = (
    Path(__file__).resolve().parent
    / "formal_targets"
    / "jin-565b6a3"
    / "source-map.json"
)
REPO_ROOT = Path(__file__).resolve().parents[2]
SHARED_LEAN_CWD = "formalization/lean"
SHARED_LEAN_ROOT = REPO_ROOT / SHARED_LEAN_CWD
SHARED_ELAN_HOME = Path("/private/tmp/harp-mathematical-foundations-elan")
SHARED_LEAN_TOOLCHAIN = "leanprover/lean4:v4.32.1"
SHARED_TOOLCHAIN_BIN = (
    SHARED_ELAN_HOME
    / "toolchains"
    / "leanprover--lean4---v4.32.1"
    / "bin"
)

DESCRIPTOR_FIELDS = frozenset(
    {
        "schema_version",
        "route_id",
        "source_map_row_id",
        "pinned_source_locator",
        "informal_statement_sha256",
        "expected_lean_declaration",
        "allowed_imports",
        "dependency_receipts",
        "output_declaration_name",
        "build_target",
        "proof_hole_policy",
        "axiom_policy",
        "attempt_budget",
    }
)
DEPENDENCY_RECEIPT_FIELDS = frozenset({"row_id", "receipt_sha256"})
PROOF_HOLE_POLICY_FIELDS = frozenset({"reject_tokens"})
AXIOM_POLICY_FIELDS = frozenset({"allowed_axioms"})
ATTEMPT_BUDGET_FIELDS = frozenset({"timeout_seconds", "max_output_bytes"})
TASK_FIELDS = frozenset(
    {
        "schema_version",
        "descriptor",
        "route_id",
        "source_map_row_id",
        "expected_lean_declaration",
        "build_target",
        "timeout_seconds",
        "max_output_bytes",
    }
)
COMMAND_FIELDS = frozenset(
    {"schema_version", "argv", "cwd", "env", "timeout_seconds", "max_output_bytes"}
)


@dataclass(frozen=True)
class CommandResult:
    exit_code: int | None
    stdout: bytes
    stderr: bytes
    blocked_reason: str | None = None
    stdout_truncated: bool = False
    stderr_truncated: bool = False
    axiom_audit_output: bytes | None = None


Executor = Callable[[list[str], Path, int, int], CommandResult]


@dataclass(frozen=True)
class DependencyReceipt:
    row_id: str
    receipt_sha256: str

    @classmethod
    def from_mapping(cls, value: Mapping[str, Any]) -> "DependencyReceipt":
        _require_fields(value, DEPENDENCY_RECEIPT_FIELDS, "dependency receipt")
        return cls(
            row_id=_runtime_id(value["row_id"], "dependency receipt row_id"),
            receipt_sha256=_digest(
                value["receipt_sha256"], "dependency receipt receipt_sha256"
            ),
        )

    def to_json(self) -> dict[str, object]:
        return {
            "row_id": self.row_id,
            "receipt_sha256": self.receipt_sha256,
        }


@dataclass(frozen=True)
class ProofSliceDescriptor:
    schema_version: str
    route_id: str
    source_map_row_id: str
    pinned_source_locator: str
    informal_statement_sha256: str
    expected_lean_declaration: str
    allowed_imports: tuple[str, ...]
    dependency_receipts: tuple[DependencyReceipt, ...]
    output_declaration_name: str
    build_target: str
    reject_tokens: tuple[str, ...]
    allowed_axioms: tuple[str, ...]
    timeout_seconds: int
    max_output_bytes: int

    def to_json(self) -> dict[str, object]:
        return {
            "schema_version": self.schema_version,
            "route_id": self.route_id,
            "source_map_row_id": self.source_map_row_id,
            "pinned_source_locator": self.pinned_source_locator,
            "informal_statement_sha256": self.informal_statement_sha256,
            "expected_lean_declaration": self.expected_lean_declaration,
            "allowed_imports": list(self.allowed_imports),
            "dependency_receipts": [
                receipt.to_json() for receipt in self.dependency_receipts
            ],
            "output_declaration_name": self.output_declaration_name,
            "build_target": self.build_target,
            "proof_hole_policy": {"reject_tokens": list(self.reject_tokens)},
            "axiom_policy": {"allowed_axioms": list(self.allowed_axioms)},
            "attempt_budget": {
                "timeout_seconds": self.timeout_seconds,
                "max_output_bytes": self.max_output_bytes,
            },
        }


def load_descriptor(
    path: Path,
    target: formal_target.FormalTargetLock,
    rows: tuple[jin_validation.SourceMapRow, ...],
) -> ProofSliceDescriptor:
    return validate_descriptor(
        formal_target._read_strict_json_object(path, "proof-slice descriptor"),
        target,
        rows,
    )


def select_first_jin_slice(
    rows: tuple[jin_validation.SourceMapRow, ...],
    target: formal_target.FormalTargetLock,
) -> jin_validation.SourceMapRow:
    for row in rows:
        if _is_terminal_row(row, target):
            continue
        if not row.dependency_ids:
            return row
    raise protocol.ValidationError("no nonterminal Jin row is selectable")


def default_descriptor_for_row(
    row: jin_validation.SourceMapRow,
) -> ProofSliceDescriptor:
    value = {
        "schema_version": SCHEMA_VERSION,
        "route_id": ROUTE_ID,
        "source_map_row_id": row.row_id,
        "pinned_source_locator": row.source_locator,
        "informal_statement_sha256": row.statement_sha256,
        "expected_lean_declaration": row.lean_name,
        "allowed_imports": [],
        "dependency_receipts": [],
        "output_declaration_name": row.lean_name,
        "build_target": BUILD_TARGET,
        "proof_hole_policy": {"reject_tokens": ["sorry", "admit"]},
        "axiom_policy": {"allowed_axioms": []},
        "attempt_budget": {
            "timeout_seconds": 3600,
            "max_output_bytes": 1048576,
        },
    }
    return validate_descriptor(value, formal_target.load_lock(), (row,))


def main_json(argv: list[str]) -> dict[str, object]:
    parser = argparse.ArgumentParser(description="Manage Jin proof-slice tasks")
    subcommands = parser.add_subparsers(dest="command", required=True)

    select_parser = subcommands.add_parser("select")
    select_parser.add_argument("--source-map", type=Path, required=True)

    materialize_parser = subcommands.add_parser("materialize")
    materialize_parser.add_argument("--task-dir", type=Path, required=True)
    materialize_parser.add_argument("--source-map", type=Path, default=DEFAULT_SOURCE_MAP)

    run_parser = subcommands.add_parser("run")
    run_parser.add_argument("--task-dir", type=Path, required=True)

    args = parser.parse_args(argv)
    target = formal_target.load_lock()

    if args.command == "select":
        rows = jin_validation.load_source_map(args.source_map, target)
        row = select_first_jin_slice(rows, target)
        return {
            "row_id": row.row_id,
            "lean_name": row.lean_name,
            "source_locator": row.source_locator,
        }

    if args.command == "materialize":
        rows = jin_validation.load_source_map(args.source_map, target)
        row = select_first_jin_slice(rows, target)
        descriptor = validate_descriptor(
            default_descriptor_for_row(row).to_json(),
            target,
            rows,
        )
        task_dir = materialize_task(args.task_dir, descriptor, rows)
        return {
            "task_dir": str(task_dir),
            "row_id": row.row_id,
            "descriptor": descriptor.to_json(),
        }

    if args.command == "run":
        return run_task(args.task_dir)

    raise AssertionError(f"unhandled command: {args.command}")


def main() -> None:
    import sys

    print(json.dumps(main_json(sys.argv[1:]), sort_keys=True))


def validate_descriptor(
    value: Mapping[str, Any],
    target: formal_target.FormalTargetLock,
    rows: tuple[jin_validation.SourceMapRow, ...],
) -> ProofSliceDescriptor:
    _require_fields(value, DESCRIPTOR_FIELDS, "proof-slice descriptor")
    _require_equal(value["schema_version"], SCHEMA_VERSION, "schema_version")

    route_id = _runtime_id(value["route_id"], "route_id")
    if route_id != ROUTE_ID:
        raise protocol.ValidationError("route_id must be jin")

    row_id = _runtime_id(value["source_map_row_id"], "source_map_row_id")
    row_by_id = {row.row_id: row for row in rows}
    row = row_by_id.get(row_id)
    if row is None:
        raise protocol.ValidationError("source_map_row_id does not exist in source map")
    if _is_terminal_row(row, target):
        raise protocol.ValidationError("terminal Jin row is out of scope for the first slice")

    pinned_source_locator = _bounded_string(
        value["pinned_source_locator"], "pinned_source_locator", 1, 4096
    )
    if pinned_source_locator != row.source_locator:
        raise protocol.ValidationError("pinned_source_locator must match source-map row")

    informal_statement_sha256 = _digest(
        value["informal_statement_sha256"], "informal_statement_sha256"
    )
    if informal_statement_sha256 != row.statement_sha256:
        raise protocol.ValidationError(
            "informal_statement_sha256 must match source-map row"
        )

    expected_declaration = _lean_declaration(
        value["expected_lean_declaration"], "expected_lean_declaration"
    )
    output_declaration = _lean_declaration(
        value["output_declaration_name"], "output_declaration_name"
    )
    if expected_declaration != row.lean_name:
        raise protocol.ValidationError(
            "expected Lean declaration must match source-map row declaration"
        )
    if output_declaration != row.lean_name:
        raise protocol.ValidationError(
            "output declaration must match source-map row declaration"
        )

    allowed_imports = _unique_string_tuple(
        value["allowed_imports"],
        "allowed_imports",
        minimum=0,
        maximum=MAX_ALLOWED_IMPORTS,
    )
    _validate_allowed_imports(allowed_imports, target)

    dependency_receipts = _dependency_receipts(
        value["dependency_receipts"], row.dependency_ids, row_by_id
    )

    build_target = _safe_relative_path(value["build_target"], "build_target")
    if build_target != BUILD_TARGET:
        raise protocol.ValidationError("build_target must be module/Slice.lean")

    proof_hole_policy = _mapping(value["proof_hole_policy"], "proof_hole_policy")
    _require_fields(
        proof_hole_policy, PROOF_HOLE_POLICY_FIELDS, "proof_hole_policy"
    )
    reject_tokens = _unique_string_tuple(
        proof_hole_policy["reject_tokens"],
        "proof_hole_policy.reject_tokens",
        minimum=1,
        maximum=MAX_REJECT_TOKENS,
    )
    rejected = set(reject_tokens)
    if "sorry" not in rejected:
        raise protocol.ValidationError("proof_hole_policy must reject sorry")
    if "admit" not in rejected:
        raise protocol.ValidationError("proof_hole_policy must reject admit")

    axiom_policy = _mapping(value["axiom_policy"], "axiom_policy")
    _require_fields(axiom_policy, AXIOM_POLICY_FIELDS, "axiom_policy")
    allowed_axioms = _unique_string_tuple(
        axiom_policy["allowed_axioms"],
        "axiom_policy.allowed_axioms",
        minimum=0,
        maximum=MAX_ALLOWED_AXIOMS,
    )

    attempt_budget = _mapping(value["attempt_budget"], "attempt_budget")
    _require_fields(attempt_budget, ATTEMPT_BUDGET_FIELDS, "attempt_budget")
    timeout_seconds = _integer(
        attempt_budget["timeout_seconds"],
        "attempt_budget.timeout_seconds",
        1,
        MAX_TIMEOUT_SECONDS,
    )
    max_output_bytes = _integer(
        attempt_budget["max_output_bytes"],
        "attempt_budget.max_output_bytes",
        1,
        MAX_OUTPUT_BYTES,
    )

    return ProofSliceDescriptor(
        schema_version=SCHEMA_VERSION,
        route_id=route_id,
        source_map_row_id=row.row_id,
        pinned_source_locator=pinned_source_locator,
        informal_statement_sha256=informal_statement_sha256,
        expected_lean_declaration=expected_declaration,
        allowed_imports=allowed_imports,
        dependency_receipts=dependency_receipts,
        output_declaration_name=output_declaration,
        build_target=build_target,
        reject_tokens=reject_tokens,
        allowed_axioms=allowed_axioms,
        timeout_seconds=timeout_seconds,
        max_output_bytes=max_output_bytes,
    )


def materialize_task(
    task_dir: Path,
    descriptor: ProofSliceDescriptor,
    rows: tuple[jin_validation.SourceMapRow, ...],
) -> Path:
    destination = task_dir.expanduser().absolute()
    _reject_symlink_ancestors(destination, "proof-slice task directory")
    if destination.exists() or destination.is_symlink():
        raise protocol.ValidationError(
            f"proof-slice task directory already exists: {destination}"
        )

    row_by_id = {row.row_id: row for row in rows}
    row = row_by_id.get(descriptor.source_map_row_id)
    if row is None:
        raise protocol.ValidationError("source_map_row_id does not exist in source map")
    _validate_materialization_row_binding(row, descriptor, row_by_id)

    created_destination = False
    try:
        destination.mkdir(parents=True, mode=0o700)
        created_destination = True
        _write_json_create_only(destination / "task.json", _task_json(descriptor), "task")
        _write_json_create_only(
            destination / "source-slice.json", _source_slice_json(row), "source slice"
        )
        _write_text_create_only(
            destination / descriptor.build_target,
            _lean_slice_source(descriptor),
            "Lean slice",
        )
        _write_json_create_only(
            destination / "build/command.json",
            _command_json(descriptor, destination),
            "command",
        )
        _write_text_create_only(destination / "build/stdout.log", "", "stdout log")
        _write_text_create_only(destination / "build/stderr.log", "", "stderr log")
        _write_json_create_only(
            destination / "build/axioms.json",
            _empty_axiom_audit(descriptor),
            "axiom audit",
        )
        _write_json_create_only(
            destination / "result.json",
            _result_json("not_attempted", "task materialized but not run"),
            "result",
        )
        _write_json_create_only(
            destination / "receipt.json",
            _receipt_json(
                descriptor,
                "not_attempted",
                "task materialized but not run",
            ),
            "receipt",
        )
    except BaseException:
        if created_destination:
            shutil.rmtree(destination, ignore_errors=True)
        raise
    return destination


def run_task(task_dir: Path, *, executor: Executor | None = None) -> dict[str, object]:
    root = task_dir.expanduser().absolute()
    _ensure_existing_directory(root, "proof-slice task directory")
    _ensure_rewritable_outputs(root)

    target = formal_target.load_lock()
    source_map = (
        Path(__file__).resolve().parent
        / "formal_targets"
        / "jin-565b6a3"
        / "source-map.json"
    )
    rows = jin_validation.load_source_map(source_map, target)
    task = _task_object(root / "task.json")
    descriptor = validate_descriptor(
        _mapping(task["descriptor"], "task.descriptor"),
        target,
        rows,
    )
    _validate_task_binding(task, descriptor)
    _validate_command_json(root / "build/command.json", descriptor, root)

    source_path = root / descriptor.build_target
    _ensure_existing_file(source_path, "Lean slice")
    source = source_path.read_text(encoding="utf-8")
    if not _has_active_expected_declaration_check(
        source, descriptor.expected_lean_declaration
    ):
        reason = (
            "Lean slice missing active expected declaration check: "
            f"#check {descriptor.expected_lean_declaration}"
        )
        result = CommandResult(None, b"", b"")
        axiom_audit = _axiom_audit(
            descriptor,
            (),
            status="not_applicable",
            reason="axiom scan not applicable because slice validation failed",
            scan_performed=False,
        )
        return _publish_run_result(root, descriptor, result, "failed", reason, axiom_audit)

    proof_hole = _first_proof_hole_token(source, descriptor.reject_tokens)
    if proof_hole is not None:
        reason = f"proof-hole token rejected before execution: {proof_hole}"
        result = CommandResult(None, b"", b"")
        axiom_audit = _axiom_audit(
            descriptor,
            (),
            status="not_applicable",
            reason="axiom scan not applicable because proof-hole policy failed",
            scan_performed=False,
        )
        return _publish_run_result(root, descriptor, result, "failed", reason, axiom_audit)

    _ensure_existing_directory(SHARED_LEAN_ROOT, "shared Lean root")
    command_result = (executor or _subprocess_executor)(
        _command_argv(descriptor, root),
        SHARED_LEAN_ROOT,
        descriptor.timeout_seconds,
        descriptor.max_output_bytes,
    )
    if not isinstance(command_result, CommandResult):
        raise protocol.ValidationError("executor must return CommandResult")
    command_result = _normalize_command_result(
        command_result, descriptor.max_output_bytes
    )

    if command_result.blocked_reason is not None or command_result.exit_code is None:
        reason = command_result.blocked_reason or "command blocked before exit"
        axiom_audit = _axiom_audit(
            descriptor,
            (),
            status="not_applicable",
            reason="axiom scan not applicable because command was blocked",
            scan_performed=False,
        )
        return _publish_run_result(
            root, descriptor, command_result, "blocked", reason, axiom_audit
        )

    if command_result.exit_code != 0:
        reason = f"Lean command failed with exit code {command_result.exit_code}"
        axiom_audit = _axiom_audit(
            descriptor,
            (),
            status="not_applicable",
            reason="axiom scan not applicable because command failed",
            scan_performed=False,
        )
        return _publish_run_result(
            root, descriptor, command_result, "failed", reason, axiom_audit
        )

    axiom_scan = _parse_axiom_audit(command_result.axiom_audit_output)
    if axiom_scan["status"] == "failed":
        observed_axioms = tuple(axiom_scan["observed_axioms"])
        axiom_audit = _axiom_audit(
            descriptor,
            observed_axioms,
            status="failed",
            reason=str(axiom_scan["reason"]),
            scan_performed=bool(axiom_scan["scan_performed"]),
        )
        output_cap_reason = _output_cap_reason(
            command_result, descriptor.max_output_bytes
        )
        reason = str(axiom_scan["reason"])
        if output_cap_reason is not None:
            reason = f"{reason}; {output_cap_reason}"
        return _publish_run_result(
            root, descriptor, command_result, "failed", reason, axiom_audit
        )

    observed_axioms = tuple(axiom_scan["observed_axioms"])
    forbidden = sorted(set(observed_axioms) - set(descriptor.allowed_axioms))
    output_cap_reason = _output_cap_reason(command_result, descriptor.max_output_bytes)
    axiom_status = "failed" if forbidden else "passed"
    axiom_reason = (
        f"forbidden axiom observed: {forbidden[0]}"
        if forbidden
        else "Lean command succeeded and axiom audit passed"
    )
    axiom_audit = _axiom_audit(
        descriptor,
        observed_axioms,
        status=axiom_status,
        reason=axiom_reason,
        scan_performed=True,
    )
    if forbidden or output_cap_reason is not None:
        reason = axiom_reason if output_cap_reason is None else output_cap_reason
        if forbidden and output_cap_reason is not None:
            reason = f"{axiom_reason}; {output_cap_reason}"
        return _publish_run_result(
            root, descriptor, command_result, "failed", reason, axiom_audit
        )

    reason = axiom_reason
    return _publish_run_result(
        root, descriptor, command_result, "passed", reason, axiom_audit
    )


def _validate_materialization_row_binding(
    row: jin_validation.SourceMapRow,
    descriptor: ProofSliceDescriptor,
    row_by_id: Mapping[str, jin_validation.SourceMapRow],
) -> None:
    if row.source_locator != descriptor.pinned_source_locator:
        raise protocol.ValidationError(
            "source-map row source_locator no longer matches descriptor"
        )
    if row.statement_sha256 != descriptor.informal_statement_sha256:
        raise protocol.ValidationError(
            "source-map row statement_sha256 no longer matches descriptor"
        )
    if row.lean_name != descriptor.expected_lean_declaration:
        raise protocol.ValidationError(
            "source-map row Lean declaration no longer matches descriptor"
        )

    expected = set(row.dependency_ids)
    observed = {receipt.row_id for receipt in descriptor.dependency_receipts}
    missing = sorted(expected - observed)
    extra = sorted(observed - expected)
    if missing:
        raise protocol.ValidationError(
            f"source-map row dependency missing descriptor receipt: {missing[0]}"
        )
    if extra:
        raise protocol.ValidationError(
            f"source-map row dependency receipts contain extra row: {extra[0]}"
        )
    _validate_dependency_receipt_bindings(
        descriptor.dependency_receipts, row.dependency_ids, row_by_id
    )


def _task_json(descriptor: ProofSliceDescriptor) -> dict[str, object]:
    return {
        "schema_version": "crouzeix-jin-proof-slice-task/v1",
        "descriptor": descriptor.to_json(),
        "route_id": descriptor.route_id,
        "source_map_row_id": descriptor.source_map_row_id,
        "expected_lean_declaration": descriptor.expected_lean_declaration,
        "build_target": descriptor.build_target,
        "timeout_seconds": descriptor.timeout_seconds,
        "max_output_bytes": descriptor.max_output_bytes,
    }


def _source_slice_json(row: jin_validation.SourceMapRow) -> dict[str, object]:
    return {
        "schema_version": "crouzeix-jin-source-slice/v1",
        "row_id": row.row_id,
        "source_locator": row.source_locator,
        "statement_sha256": row.statement_sha256,
        "lean_name": row.lean_name,
        "dependency_ids": list(row.dependency_ids),
    }


def _lean_slice_source(descriptor: ProofSliceDescriptor) -> str:
    lines = [f"import {name}" for name in descriptor.allowed_imports]
    lines.extend(
        [
            "",
            (
                "/- Harp-owned proof-slice adapter. This file checks only the "
                "selected nonterminal declaration. -/"
            ),
            f"#check {descriptor.expected_lean_declaration}",
        ]
    )
    return "\n".join(lines).lstrip("\n") + "\n"


def _command_json(descriptor: ProofSliceDescriptor, task_dir: Path) -> dict[str, object]:
    return {
        "schema_version": "crouzeix-jin-proof-slice-command/v1",
        "argv": _command_argv(descriptor, task_dir),
        "cwd": SHARED_LEAN_CWD,
        "env": {},
        "timeout_seconds": descriptor.timeout_seconds,
        "max_output_bytes": descriptor.max_output_bytes,
    }


def _command_argv(descriptor: ProofSliceDescriptor, task_dir: Path) -> list[str]:
    return [
        "lake",
        "env",
        "lean",
        os.path.relpath(task_dir / descriptor.build_target, SHARED_LEAN_ROOT),
    ]


def _shared_lean_environment() -> dict[str, str]:
    path_entries = [SHARED_TOOLCHAIN_BIN.as_posix()]
    ambient_path = os.environ.get("PATH")
    if ambient_path:
        path_entries.append(ambient_path)
    return {
        "ELAN_HOME": SHARED_ELAN_HOME.as_posix(),
        "ELAN_TOOLCHAIN": SHARED_LEAN_TOOLCHAIN,
        "PATH": os.pathsep.join(path_entries),
    }


def _empty_axiom_audit(descriptor: ProofSliceDescriptor) -> dict[str, object]:
    return {
        "schema_version": "crouzeix-jin-proof-slice-axioms/v1",
        "scan_performed": False,
        "expected_declaration": descriptor.expected_lean_declaration,
        "allowed_axioms": list(descriptor.allowed_axioms),
        "observed_axioms": [],
        "status": "not_attempted",
        "reason": "task materialized but not run",
    }


def _result_json(status: str, reason: str) -> dict[str, object]:
    return {
        "schema_version": "crouzeix-jin-proof-slice-result/v1",
        "status": status,
        "reason": reason,
    }


def _receipt_json(
    descriptor: ProofSliceDescriptor,
    status: str,
    reason: str,
    *,
    command: Mapping[str, object] | None = None,
    command_result: CommandResult | None = None,
    axiom_audit: Mapping[str, object] | None = None,
    result: Mapping[str, object] | None = None,
) -> dict[str, object]:
    receipt = {
        "schema_version": "crouzeix-jin-proof-slice-receipt/v1",
        "route_id": descriptor.route_id,
        "source_map_row_id": descriptor.source_map_row_id,
        "expected_lean_declaration": descriptor.expected_lean_declaration,
        "status": status,
        "reason": reason,
        "task_sha256": _canonical_sha256(_task_json(descriptor)),
    }
    if command is not None:
        receipt["command_sha256"] = _canonical_sha256(command)
    if command_result is not None:
        receipt["command_exit_code"] = command_result.exit_code
        receipt["stdout_sha256"] = protocol.sha256_bytes(command_result.stdout)
        receipt["stderr_sha256"] = protocol.sha256_bytes(command_result.stderr)
        receipt["stdout_truncated"] = command_result.stdout_truncated
        receipt["stderr_truncated"] = command_result.stderr_truncated
        receipt["max_output_bytes"] = descriptor.max_output_bytes
    if axiom_audit is not None:
        receipt["axioms_sha256"] = _canonical_sha256(axiom_audit)
    if result is not None:
        receipt["result_sha256"] = _canonical_sha256(result)
    return receipt


def _is_terminal_row(
    row: jin_validation.SourceMapRow, target: formal_target.FormalTargetLock
) -> bool:
    return row.row_id == TERMINAL_ROW_ID or row.lean_name == target.target.declaration_name


def _validate_allowed_imports(
    allowed_imports: tuple[str, ...], target: formal_target.FormalTargetLock
) -> None:
    allowlist = set(target.import_allowlist)
    for module in allowed_imports:
        _lean_module(module, "allowed_imports item")
        if module in TERMINAL_REFERENCE_IMPORTS:
            raise protocol.ValidationError(
                f"allowed_imports contains terminal reference import: {module}"
            )
        if module not in allowlist:
            raise protocol.ValidationError(
                f"allowed_imports contains import outside FormalTarget allowlist: {module}"
            )


def _task_object(path: Path) -> dict[str, Any]:
    task = formal_target._read_strict_json_object(path, "proof-slice task")
    _require_fields(task, TASK_FIELDS, "proof-slice task")
    _require_equal(
        task["schema_version"],
        "crouzeix-jin-proof-slice-task/v1",
        "task.schema_version",
    )
    return task


def _validate_task_binding(
    task: Mapping[str, Any], descriptor: ProofSliceDescriptor
) -> None:
    if task["descriptor"] != descriptor.to_json():
        raise protocol.ValidationError("task descriptor is not canonical")
    if task["route_id"] != descriptor.route_id:
        raise protocol.ValidationError("task route_id does not match descriptor")
    if task["source_map_row_id"] != descriptor.source_map_row_id:
        raise protocol.ValidationError(
            "task source_map_row_id does not match descriptor"
        )
    if task["expected_lean_declaration"] != descriptor.expected_lean_declaration:
        raise protocol.ValidationError(
            "task expected_lean_declaration does not match descriptor"
        )
    if task["build_target"] != descriptor.build_target:
        raise protocol.ValidationError("task build_target does not match descriptor")
    if task["timeout_seconds"] != descriptor.timeout_seconds:
        raise protocol.ValidationError("task timeout_seconds does not match descriptor")
    if task["max_output_bytes"] != descriptor.max_output_bytes:
        raise protocol.ValidationError("task max_output_bytes does not match descriptor")


def _validate_command_json(
    path: Path, descriptor: ProofSliceDescriptor, task_dir: Path
) -> None:
    command = formal_target._read_strict_json_object(path, "proof-slice command")
    _require_fields(command, COMMAND_FIELDS, "proof-slice command")
    _require_equal(
        command["schema_version"],
        "crouzeix-jin-proof-slice-command/v1",
        "command.schema_version",
    )
    expected_argv = _command_argv(descriptor, task_dir)
    if command["argv"] != expected_argv:
        raise protocol.ValidationError("command argv does not match descriptor")
    if command["cwd"] != SHARED_LEAN_CWD:
        raise protocol.ValidationError("command cwd must be the shared Lean root")
    if command["env"] != {}:
        raise protocol.ValidationError("command env must be empty")
    if command["timeout_seconds"] != descriptor.timeout_seconds:
        raise protocol.ValidationError("command timeout_seconds does not match descriptor")
    if command["max_output_bytes"] != descriptor.max_output_bytes:
        raise protocol.ValidationError("command max_output_bytes does not match descriptor")


def _first_proof_hole_token(source: str, reject_tokens: tuple[str, ...]) -> str | None:
    for token in reject_tokens:
        if token in source:
            return token
    return None


def _has_active_expected_declaration_check(source: str, declaration: str) -> bool:
    expected = f"#check {declaration}"
    block_comment_depth = 0
    for line in source.splitlines():
        code, block_comment_depth = _lean_code_prefix(line, block_comment_depth)
        if code.strip() == expected:
            return True
    return False


def _lean_code_prefix(line: str, block_comment_depth: int) -> tuple[str, int]:
    index = 0
    output = []
    while index < len(line):
        if block_comment_depth:
            nested = line.find("/-", index)
            end = line.find("-/", index)
            if end == -1 and nested == -1:
                return "".join(output), block_comment_depth
            if nested != -1 and (end == -1 or nested < end):
                block_comment_depth += 1
                index = nested + 2
                continue
            if end == -1:
                return "".join(output), block_comment_depth
            index = end + 2
            block_comment_depth -= 1
            continue
        line_comment = line.find("--", index)
        block_comment = line.find("/-", index)
        if line_comment != -1 and (
            block_comment == -1 or line_comment < block_comment
        ):
            output.append(line[index:line_comment])
            return "".join(output), False
        if block_comment != -1:
            output.append(line[index:block_comment])
            index = block_comment + 2
            block_comment_depth = 1
            continue
        output.append(line[index:])
        return "".join(output), 0
    return "".join(output), block_comment_depth


def _parse_axiom_audit(output: bytes | None) -> dict[str, object]:
    if output is None:
        return _failed_axiom_scan("axiom audit output missing")
    text = output.decode("utf-8", errors="replace")
    observed: tuple[str, ...] | None = None
    for line in text.splitlines():
        stripped = line.strip()
        if not stripped.startswith("axioms:"):
            if stripped:
                return _failed_axiom_scan("axiom audit output malformed")
            continue
        if stripped == "axioms: none":
            parsed: tuple[str, ...] = ()
        else:
            payload = stripped.removeprefix("axioms:").strip()
            if not payload:
                return _failed_axiom_scan("axiom audit output malformed")
            pieces = tuple(part.strip() for part in payload.split(","))
            if any(not _is_lean_name(part) for part in pieces):
                return _failed_axiom_scan("axiom audit output malformed")
            if len(set(pieces)) != len(pieces):
                return _failed_axiom_scan("axiom audit output malformed")
            parsed = pieces
        if observed is not None:
            return _failed_axiom_scan("axiom audit output malformed")
        observed = parsed
    if observed is None:
        return _failed_axiom_scan("axiom audit output missing")
    return {
        "scan_performed": True,
        "observed_axioms": observed,
        "status": "passed",
        "reason": "axiom audit output parsed",
    }


def _failed_axiom_scan(reason: str) -> dict[str, object]:
    return {
        "scan_performed": False,
        "observed_axioms": (),
        "status": "failed",
        "reason": reason,
    }


def _is_lean_name(value: str) -> bool:
    pattern = r"[A-Za-z_][A-Za-z0-9_']*(\.[A-Za-z_][A-Za-z0-9_']*)*"
    return re.fullmatch(pattern, value) is not None


def _output_cap_reason(
    command_result: CommandResult, max_output_bytes: int
) -> str | None:
    if command_result.stdout_truncated or len(command_result.stdout) > max_output_bytes:
        return "output cap policy violation: stdout exceeds max_output_bytes"
    if command_result.stderr_truncated or len(command_result.stderr) > max_output_bytes:
        return "output cap policy violation: stderr exceeds max_output_bytes"
    return None


def _normalize_command_result(
    command_result: CommandResult, max_output_bytes: int
) -> CommandResult:
    stdout_truncated = (
        command_result.stdout_truncated
        or len(command_result.stdout) > max_output_bytes
    )
    stderr_truncated = (
        command_result.stderr_truncated
        or len(command_result.stderr) > max_output_bytes
    )
    if (
        len(command_result.stdout) <= max_output_bytes
        and len(command_result.stderr) <= max_output_bytes
        and stdout_truncated == command_result.stdout_truncated
        and stderr_truncated == command_result.stderr_truncated
    ):
        return command_result
    return CommandResult(
        command_result.exit_code,
        command_result.stdout[:max_output_bytes],
        command_result.stderr[:max_output_bytes],
        command_result.blocked_reason,
        stdout_truncated=stdout_truncated,
        stderr_truncated=stderr_truncated,
        axiom_audit_output=command_result.axiom_audit_output,
    )


def _axiom_audit(
    descriptor: ProofSliceDescriptor,
    observed_axioms: tuple[str, ...],
    *,
    status: str,
    reason: str,
    scan_performed: bool,
) -> dict[str, object]:
    return {
        "schema_version": "crouzeix-jin-proof-slice-axioms/v1",
        "scan_performed": scan_performed,
        "expected_declaration": descriptor.expected_lean_declaration,
        "allowed_axioms": list(descriptor.allowed_axioms),
        "observed_axioms": list(observed_axioms),
        "status": status,
        "reason": reason,
    }


def _publish_run_result(
    root: Path,
    descriptor: ProofSliceDescriptor,
    command_result: CommandResult,
    status: str,
    reason: str,
    axiom_audit: Mapping[str, object],
) -> dict[str, object]:
    result = _result_json(status, reason)
    command = _command_json(descriptor, root)
    receipt = _receipt_json(
        descriptor,
        status,
        reason,
        command=command,
        command_result=command_result,
        axiom_audit=axiom_audit,
        result=result,
    )
    _write_bytes_existing(root / "build/stdout.log", command_result.stdout, "stdout log")
    _write_bytes_existing(root / "build/stderr.log", command_result.stderr, "stderr log")
    _write_json_existing(root / "build/axioms.json", axiom_audit, "axiom audit")
    _write_json_existing(root / "result.json", result, "result")
    _write_json_existing(root / "receipt.json", receipt, "receipt")
    return result


def _subprocess_executor(
    argv: list[str],
    cwd: Path,
    timeout_seconds: int,
    max_output_bytes: int,
) -> CommandResult:
    environment = _shared_lean_environment()
    executable = argv[0]
    resolved_executable = executable
    if not Path(executable).is_absolute():
        found = shutil.which(executable, path=environment["PATH"])
        if found is None:
            reason = f"missing executable: {executable}"
            stderr = reason.encode("utf-8")
            return CommandResult(
                None,
                b"",
                stderr[:max_output_bytes],
                reason,
                stderr_truncated=len(stderr) > max_output_bytes,
            )
        resolved_executable = found
    resolved_argv = [resolved_executable, *argv[1:]]
    stdout = _BoundedPipe(max_output_bytes)
    stderr = _BoundedPipe(max_output_bytes)
    try:
        process = subprocess.Popen(
            resolved_argv,
            cwd=cwd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            env=environment,
            start_new_session=True,
        )
        stdout.start(process.stdout)
        stderr.start(process.stderr)
        try:
            exit_code = process.wait(timeout=timeout_seconds)
        except subprocess.TimeoutExpired:
            _terminate_process_tree(process)
            exit_code = None
            blocked_reason = f"command timed out after {timeout_seconds} seconds"
        else:
            blocked_reason = None
        stdout.join(PIPE_DRAIN_GRACE_SECONDS)
        stderr.join(PIPE_DRAIN_GRACE_SECONDS)
    except FileNotFoundError as error:
        stderr = str(error).encode("utf-8")
        return CommandResult(
            None,
            b"",
            stderr[:max_output_bytes],
            str(error),
            stderr_truncated=len(stderr) > max_output_bytes,
        )
    if blocked_reason is not None:
        return CommandResult(
            None,
            stdout.data,
            stderr.data,
            blocked_reason,
            stdout_truncated=stdout.truncated,
            stderr_truncated=stderr.truncated,
        )
    return CommandResult(
        exit_code,
        stdout.data,
        stderr.data,
        stdout_truncated=stdout.truncated,
        stderr_truncated=stderr.truncated,
    )


def _terminate_process_tree(process: subprocess.Popen[bytes]) -> None:
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except (AttributeError, ProcessLookupError):
        return
    except OSError:
        process.kill()


class _BoundedPipe:
    def __init__(self, max_bytes: int) -> None:
        self._max_bytes = max_bytes
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

    def join(self, timeout: float | None = None) -> None:
        if self._thread is not None:
            self._thread.join(timeout)

    def _read(self, stream: Any) -> None:
        with stream:
            while True:
                chunk = stream.read(8192)
                if not chunk:
                    return
                remaining = self._max_bytes - self._size
                if remaining > 0:
                    self._chunks.append(chunk[:remaining])
                    self._size += min(len(chunk), remaining)
                if len(chunk) > remaining:
                    self.truncated = True


def _dependency_receipts(
    value: Any,
    dependency_ids: tuple[str, ...],
    row_by_id: Mapping[str, jin_validation.SourceMapRow],
) -> tuple[DependencyReceipt, ...]:
    if not isinstance(value, list) or len(value) > MAX_DEPENDENCY_RECEIPTS:
        raise protocol.ValidationError("dependency_receipts must be a bounded list")
    receipts = tuple(
        DependencyReceipt.from_mapping(_mapping(item, "dependency receipt"))
        for item in value
    )
    row_ids = tuple(receipt.row_id for receipt in receipts)
    if len(set(row_ids)) != len(row_ids):
        raise protocol.ValidationError(
            "dependency_receipts must not contain duplicate row_id values"
        )
    receipt_sha256_values = tuple(receipt.receipt_sha256 for receipt in receipts)
    if len(set(receipt_sha256_values)) != len(receipt_sha256_values):
        raise protocol.ValidationError(
            "dependency_receipts must not contain duplicate receipt_sha256 values"
        )
    expected = set(dependency_ids)
    observed = set(row_ids)
    missing = sorted(expected - observed)
    extra = sorted(observed - expected)
    if missing:
        raise protocol.ValidationError(
            f"dependency_receipts missing dependency row: {missing[0]}"
        )
    if extra:
        raise protocol.ValidationError(
            f"dependency_receipts contains extra dependency row: {extra[0]}"
        )
    _validate_dependency_receipt_bindings(receipts, dependency_ids, row_by_id)
    return receipts


def _validate_dependency_receipt_bindings(
    receipts: tuple[DependencyReceipt, ...],
    dependency_ids: tuple[str, ...],
    row_by_id: Mapping[str, jin_validation.SourceMapRow],
) -> None:
    receipt_by_row = {receipt.row_id: receipt for receipt in receipts}
    for dependency_id in dependency_ids:
        row = row_by_id.get(dependency_id)
        if row is None:
            raise protocol.ValidationError(
                f"dependency_receipts reference missing source-map row: {dependency_id}"
            )
        if row.status != "passed":
            raise protocol.ValidationError(
                f"dependency row is not passed: {dependency_id}"
            )
        if row.receipt_sha256 is None:
            raise protocol.ValidationError(
                f"passed dependency row is missing receipt_sha256: {dependency_id}"
            )
        receipt = receipt_by_row[dependency_id]
        if receipt.receipt_sha256 != row.receipt_sha256:
            raise protocol.ValidationError(
                f"dependency receipt digest mismatch for row: {dependency_id}"
            )


def _unique_string_tuple(
    value: Any,
    label: str,
    *,
    minimum: int,
    maximum: int,
) -> tuple[str, ...]:
    values = _string_tuple(value, label, minimum=minimum, maximum=maximum)
    if len(set(values)) != len(values):
        raise protocol.ValidationError(f"{label} must not contain duplicate entries")
    return values


def _lean_declaration(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 256)
    if "." not in text:
        raise protocol.ValidationError(f"{label} must be a Lean declaration name")
    return text


def _lean_module(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 256)
    if "." not in text:
        raise protocol.ValidationError(f"{label} must be a Lean module name")
    return text


def _require_fields(value: Mapping[str, Any], allowed: frozenset[str], label: str) -> None:
    return formal_target._require_fields(value, allowed, label)


def _mapping(value: Any, label: str) -> Mapping[str, Any]:
    return formal_target._mapping(value, label)


def _require_equal(value: Any, expected: str, label: str) -> None:
    return formal_target._require_equal(value, expected, label)


def _integer(value: Any, label: str, minimum: int, maximum: int) -> int:
    return formal_target._integer(value, label, minimum, maximum)


def _bounded_string(value: Any, label: str, minimum: int, maximum: int) -> str:
    return formal_target._bounded_string(value, label, minimum, maximum)


def _runtime_id(value: Any, label: str) -> str:
    return formal_target._runtime_id(value, label)


def _digest(value: Any, label: str) -> str:
    return formal_target._digest(value, label)


def _safe_relative_path(value: Any, label: str) -> str:
    return formal_target._safe_relative_path(value, label)


def _string_tuple(
    value: Any, label: str, *, minimum: int, maximum: int
) -> tuple[str, ...]:
    return formal_target._string_tuple(value, label, minimum=minimum, maximum=maximum)


def _reject_symlink_ancestors(path: Path, label: str) -> None:
    current = Path(path.anchor) if path.is_absolute() else Path(".")
    parts = path.parts[1:] if path.is_absolute() else path.parts
    for part in parts:
        current = current / part
        if current.exists() or current.is_symlink():
            metadata = current.lstat()
            if stat.S_ISLNK(metadata.st_mode):
                raise protocol.ValidationError(f"{label} contains symlink: {current}")


def _write_json_create_only(
    path: Path, value: Mapping[str, object], label: str
) -> None:
    _write_text_create_only(path, _stable_json(value), label)


def _write_text_create_only(path: Path, value: str, label: str) -> None:
    _reject_symlink_ancestors(path, label)
    path.parent.mkdir(parents=True, exist_ok=True)
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    try:
        fd = os.open(path, flags, 0o600)
    except FileExistsError as error:
        raise protocol.ValidationError(f"{label} already exists: {path}") from error
    with os.fdopen(fd, "w", encoding="utf-8") as handle:
        handle.write(value)


def _write_json_existing(
    path: Path, value: Mapping[str, object], label: str
) -> None:
    _write_bytes_existing(path, _stable_json(value).encode("utf-8"), label)


def _write_bytes_existing(path: Path, value: bytes, label: str) -> None:
    _ensure_existing_file(path, label)
    with path.open("wb") as handle:
        handle.write(value)


def _ensure_rewritable_outputs(root: Path) -> None:
    for relative, label in (
        ("build/stdout.log", "stdout log"),
        ("build/stderr.log", "stderr log"),
        ("build/axioms.json", "axiom audit"),
        ("result.json", "result"),
        ("receipt.json", "receipt"),
    ):
        _ensure_existing_file(root / relative, label)


def _ensure_existing_file(path: Path, label: str) -> None:
    _reject_symlink_ancestors(path, label)
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(f"{label} cannot be a symlink")
    if not stat.S_ISREG(metadata.st_mode):
        raise protocol.ValidationError(f"{label} must be a regular file")


def _ensure_existing_directory(path: Path, label: str) -> None:
    _reject_symlink_ancestors(path, label)
    try:
        metadata = path.lstat()
    except OSError as error:
        raise protocol.ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(f"{label} cannot be a symlink")
    if not stat.S_ISDIR(metadata.st_mode):
        raise protocol.ValidationError(f"{label} must be a directory")


def _stable_json(value: Mapping[str, object]) -> str:
    return json.dumps(value, indent=2, sort_keys=True) + "\n"


def _canonical_sha256(value: Mapping[str, object]) -> str:
    data = json.dumps(value, sort_keys=True, separators=(",", ":")).encode("utf-8")
    return protocol.sha256_bytes(data)


if __name__ == "__main__":
    main()
