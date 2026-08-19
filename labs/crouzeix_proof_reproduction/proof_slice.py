from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path
from typing import Any, Mapping

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
        value["dependency_receipts"], row.dependency_ids
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


def _dependency_receipts(
    value: Any, dependency_ids: tuple[str, ...]
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
    return receipts


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
