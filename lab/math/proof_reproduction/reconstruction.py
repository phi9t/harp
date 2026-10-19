from __future__ import annotations

import hashlib
from dataclasses import dataclass, field
from typing import Any, Mapping

import formal_target
import formal_receipt
import protocol


OBLIGATION_FIELDS = frozenset(
    {
        "schema_version",
        "obligation_id",
        "statement_sha256",
        "source_locator",
        "imports",
        "local_context",
        "proof_slots",
        "attempt_budget",
        "decomposition_budget",
    }
)
CLASSIFICATIONS = frozenset(
    {
        "compiled",
        "syntax",
        "missing_fact",
        "type_mismatch",
        "false_subgoal",
        "timeout",
        "policy_violation",
    }
)


@dataclass(frozen=True)
class SealedObligation:
    obligation_id: str
    statement_sha256: str
    source_locator: str
    imports: tuple[str, ...]
    local_context: tuple[str, ...]
    proof_slots: tuple[str, ...]
    attempt_budget: int
    decomposition_budget: int

    @classmethod
    def from_mapping(cls, value: Mapping[str, Any]) -> "SealedObligation":
        _require_fields(value, OBLIGATION_FIELDS, "sealed obligation")
        _require_equal(
            value["schema_version"],
            "crouzeix-reconstruction-obligation/v1",
            "schema_version",
        )
        proof_slots = _runtime_id_tuple(value["proof_slots"], "proof_slots")
        if len(set(proof_slots)) != len(proof_slots):
            raise protocol.ValidationError("proof_slots must be unique")
        return cls(
            obligation_id=formal_target._runtime_id(
                value["obligation_id"], "obligation_id"
            ),
            statement_sha256=formal_target._digest(
                value["statement_sha256"], "statement_sha256"
            ),
            source_locator=_bounded_string(value["source_locator"], "source_locator"),
            imports=_string_tuple(value["imports"], "imports"),
            local_context=_string_tuple(value["local_context"], "local_context"),
            proof_slots=proof_slots,
            attempt_budget=_integer(value["attempt_budget"], "attempt_budget", 1, 128),
            decomposition_budget=_integer(
                value["decomposition_budget"], "decomposition_budget", 0, 128
            ),
        )


@dataclass
class ReconstructionHistory:
    obligation: SealedObligation
    events: list[dict[str, object]] = field(default_factory=list)

    def record(self, event: Mapping[str, Any]) -> None:
        if len(self.events) >= self.obligation.attempt_budget:
            raise protocol.ValidationError("reconstruction attempt budget exhausted")
        classification = _classification(event)
        candidate_sha256 = formal_target._digest(
            event.get("candidate_sha256"), "candidate_sha256"
        )
        self.events.append(
            {
                "classification": classification,
                "candidate_sha256": candidate_sha256,
            }
        )

    def publish_if_compiled(self, event: Mapping[str, Any]) -> dict[str, object]:
        classification = _classification(event)
        if classification != "compiled":
            raise protocol.ValidationError("only compiled candidates can be published")
        candidate_sha256 = formal_target._digest(
            event.get("candidate_sha256"), "candidate_sha256"
        )
        axiom_audit_sha256 = formal_target._digest(
            event.get("axiom_audit_sha256"), "axiom_audit_sha256"
        )
        receipt_raw = event.get("formal_receipt")
        if not isinstance(receipt_raw, Mapping):
            raise protocol.ValidationError("compiled publication requires formal receipt")
        receipt = formal_receipt.validate_receipt(receipt_raw)
        if receipt["schema_version"] != "crouzeix-formal-attempt-receipt/v2":
            raise protocol.ValidationError("compiled publication requires v2 formal receipt")
        if receipt["status"] != "passed":
            raise protocol.ValidationError("compiled publication requires passed formal receipt")
        candidate = receipt["candidate"]
        if not isinstance(candidate, Mapping) or candidate.get("candidate_sha256") != candidate_sha256:
            raise protocol.ValidationError("compiled publication candidate digest mismatch")
        axioms = receipt["axioms"]
        if not isinstance(axioms, Mapping) or axioms.get("scan_log_sha256") != axiom_audit_sha256:
            raise protocol.ValidationError("compiled publication axiom audit digest mismatch")
        return {
            "schema_version": "crouzeix-formal-ledger-row/v1",
            "row_id": self.obligation.obligation_id,
            "statement_sha256": self.obligation.statement_sha256,
            "source_locator": self.obligation.source_locator,
            "dependency_ids": [],
            "owner_route": "reconstruction",
            "status": "locally_proved",
            "candidate_sha256": candidate_sha256,
            "axiom_audit_sha256": axiom_audit_sha256,
        }


def materialize_candidate(
    obligation: SealedObligation, slots: Mapping[str, str]
) -> str:
    unexpected = sorted(set(slots) - set(obligation.proof_slots))
    if unexpected:
        raise protocol.ValidationError(f"candidate writes undeclared slot: {unexpected[0]}")
    missing = sorted(set(obligation.proof_slots) - set(slots))
    if missing:
        raise protocol.ValidationError(f"candidate missing proof slot: {missing[0]}")
    rendered = []
    for item in obligation.imports:
        rendered.append(f"import {item}")
    rendered.append("")
    for item in obligation.local_context:
        rendered.append(f"-- context: {item}")
    for slot in obligation.proof_slots:
        body = _bounded_string(slots[slot], f"slot {slot}")
        if "\nimport " in f"\n{body}" or "\nopen " in f"\n{body}":
            raise protocol.ValidationError("candidate cannot change imports or namespace")
        rendered.append(f"-- slot: {slot}")
        rendered.append(body)
    return "\n".join(rendered) + "\n"


def candidate_sha256(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def _classification(event: Mapping[str, Any]) -> str:
    return _enum(event.get("classification"), CLASSIFICATIONS, "classification")


def _require_fields(value: Mapping[str, Any], allowed: frozenset[str], label: str) -> None:
    fields = set(value)
    if fields != set(allowed):
        raise protocol.ValidationError(f"{label} fields are invalid")


def _require_equal(value: Any, expected: str, label: str) -> None:
    if value != expected:
        raise protocol.ValidationError(f"{label} must be {expected}")


def _enum(value: Any, allowed: frozenset[str], label: str) -> str:
    if not isinstance(value, str) or value not in allowed:
        raise protocol.ValidationError(f"{label} is invalid")
    return value


def _bounded_string(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value or len(value) > 4096 or "\0" in value:
        raise protocol.ValidationError(f"{label} must be a bounded non-NUL string")
    return value


def _string_tuple(value: Any, label: str) -> tuple[str, ...]:
    if not isinstance(value, list) or len(value) > 256:
        raise protocol.ValidationError(f"{label} must be a bounded list")
    return tuple(_bounded_string(item, label) for item in value)


def _runtime_id_tuple(value: Any, label: str) -> tuple[str, ...]:
    if not isinstance(value, list) or not value or len(value) > 256:
        raise protocol.ValidationError(f"{label} must be a nonempty bounded list")
    return tuple(formal_target._runtime_id(item, label) for item in value)


def _integer(value: Any, label: str, minimum: int, maximum: int) -> int:
    if (
        not isinstance(value, int)
        or isinstance(value, bool)
        or value < minimum
        or value > maximum
    ):
        raise protocol.ValidationError(f"{label} must be an integer in range")
    return value
