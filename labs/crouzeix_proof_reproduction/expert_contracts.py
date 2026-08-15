from __future__ import annotations

import hashlib
import json
from typing import Any, Mapping

from protocol import ValidationError


SCHEMA_VERSION_EXPERT_CONTEXT = "crouzeix-expert-context/v1"
SCHEMA_VERSION_EVALUATOR_CONTEXT = "crouzeix-evaluator-context/v1"
SCHEMA_VERSION_EXPERT_RESULT = "crouzeix-expert-result/v1"
SCHEMA_VERSION_ADMISSION_DECISION = "crouzeix-admission-decision/v1"
SCHEMA_VERSION_MATHEMATICAL_NODE = "crouzeix-mathematical-node/v1"
SCHEMA_VERSION_NODE_EVALUATION_PAYLOAD = "crouzeix-node-evaluation-payload/v1"
SCHEMA_VERSION_NODE_EVALUATION = "crouzeix-node-evaluation/v1"

EXPERT_ROLES = frozenset(
    {
        "function_theory",
        "operator_dilation",
        "matrix_extremal",
        "completion_positivity",
        "approximation_audit",
    }
)
LEAKAGE_TIERS = frozenset({"L0", "L1", "L2", "L3", "L4"})
DIRECTION_KINDS = frozenset(
    {"root_task", "obligation", "evaluator_finding", "proposed_direction"}
)
DIRECTION_STRENGTHS = frozenset(
    {"root", "theorem_strength", "critical", "major", "local", "minor", "proposed"}
)
ENDPOINT_KINDS = frozenset({"candidate_proof", "blocker"})
ADMISSION_OUTCOMES = frozenset(
    {
        "accepted",
        "rejected_leakage",
        "rejected_provenance",
        "rejected_nonfunctioning",
    }
)
ADMISSION_REASON_BY_OUTCOME = {
    "accepted": "admission_criteria_satisfied",
    "rejected_leakage": "leakage_boundary_failed",
    "rejected_provenance": "provenance_check_failed",
    "rejected_nonfunctioning": "nonfunctioning_result",
}
PROBE_STATUSES = frozenset({"pass", "fail", "insufficient_evidence"})
SOURCE_KINDS = frozenset({"provider_output", "conservative_fallback"})
FINDING_SEVERITIES = frozenset({"critical", "major", "minor"})
OBLIGATION_STRENGTHS = frozenset({"local", "major", "theorem_strength"})
PROPOSED_DIRECTION_STRENGTHS = frozenset(
    {"local", "major", "theorem_strength", "critical", "minor", "proposed"}
)
PROOF_PROGRESS_PROBE_IDS = (
    "p01_theorem_statement_preserved",
    "p02_no_theorem_strength_reduction",
    "p03_core_mechanism_explicit",
    "p04_local_claims_justified",
    "p05_dependencies_closed",
    "p06_circularity_addressed",
    "p07_obligations_listed",
    "p08_candidate_text_coherent",
    "p09_blockers_falsifiable",
    "p10_candidate_proof_present",
)
DEFAULT_EXPERT_LIMITS = {
    "timeout_seconds": 3600,
    "max_output_bytes": 1048576,
}
FUNCTIONING_CRITERIA = (
    "candidate_proof requires a concrete mechanism and at least one proved statement; "
    "blocker requires a precise falsifiable obstruction and a materially new direction"
)
COMPLETION_CRITERIA = (
    "return one strict expert_result with one endpoint variant and stable local "
    "IDs for statements, obligations, risks, and directions"
)

_PORTABLE_ID = __import__("re").compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
_SHA256 = __import__("re").compile(r"^[0-9a-f]{64}$")


def build_expert_context(
    *,
    run_id: str,
    attempt_id: str,
    ticket_id: str,
    proposed_node_id: str,
    parent_node: Mapping[str, Any] | None,
    generation: int,
    expert_role: str,
    selected_direction: Mapping[str, Any],
    theorem_text: str,
    forbidden_sources: list[str],
) -> dict[str, object]:
    direction = _validate_selected_direction(selected_direction)
    role = _enum(expert_role, EXPERT_ROLES, "expert_role")
    if direction["recommended_role"] != role:
        raise ValidationError("expert_role must match selected direction recommended_role")
    generation_value = _bounded_int(generation, "generation", 0, 1000)
    parent = _project_parent(parent_node, generation_value)
    if generation_value == 0 and parent is not None:
        raise ValidationError("root expert context cannot include a parent")
    if generation_value > 0 and parent is None:
        raise ValidationError("child expert context requires exactly one parent")
    if parent is None and (
        direction["kind"] != "root_task"
        or direction["source_node_artifact_sha256"] is not None
        or direction["source_reconciliation_sha256"] is not None
    ):
        raise ValidationError("root direction must not reference parent artifacts")
    if parent is not None and direction["source_node_artifact_sha256"] != parent["node_artifact_sha256"]:
        raise ValidationError("child direction must reference the selected parent artifact")

    theorem = _bounded_str(theorem_text, "theorem_text", 1, 200_000)
    sources = _unique_strings(forbidden_sources, "forbidden_sources", 1, 64)
    return {
        "schema_version": SCHEMA_VERSION_EXPERT_CONTEXT,
        "run_id": _portable_id(run_id, "run_id"),
        "attempt_id": _portable_id(attempt_id, "attempt_id"),
        "ticket_id": _portable_id(ticket_id, "ticket_id"),
        "proposed_node_id": _portable_id(proposed_node_id, "proposed_node_id"),
        "parent": parent,
        "generation": generation_value,
        "expert_role": role,
        "selected_direction": direction,
        "theorem_text": theorem,
        "theorem_sha256": _sha256_text(theorem),
        "forbidden_sources": sources,
        "allowed_tools": ["Write"],
        "delegation_allowed": False,
        "limits": dict(DEFAULT_EXPERT_LIMITS),
        "functioning_criteria": FUNCTIONING_CRITERIA,
        "completion_criteria": COMPLETION_CRITERIA,
        "result_schema": "expert_result",
    }


def build_evaluator_context(
    *,
    theorem_text: str,
    node: Mapping[str, Any],
) -> dict[str, object]:
    validated_node = validate_mathematical_node(node)
    return {
        "schema_version": SCHEMA_VERSION_EVALUATOR_CONTEXT,
        "theorem_text": _bounded_str(theorem_text, "theorem_text", 1, 200_000),
        "mathematical_payload": _copy_mapping(
            validated_node["mathematical_payload"], "mathematical_payload"
        ),
        "probe_ids": list(PROOF_PROGRESS_PROBE_IDS),
    }


def validate_expert_result(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "expert result")
    expected = {
        "schema_version",
        "run_id",
        "attempt_id",
        "ticket_id",
        "proposed_node_id",
        "parent_node_id",
        "parent_node_artifact_sha256",
        "generation",
        "expert_role",
        "selected_direction_id",
        "mathematical_payload",
    }
    if "expert_result_sha256" in item:
        expected.add("expert_result_sha256")
    _require_fields(item, expected, "expert result")
    result: dict[str, object] = {
        "schema_version": _const(
            item["schema_version"], SCHEMA_VERSION_EXPERT_RESULT, "schema_version"
        ),
        "run_id": _portable_id(item["run_id"], "run_id"),
        "attempt_id": _portable_id(item["attempt_id"], "attempt_id"),
        "ticket_id": _portable_id(item["ticket_id"], "ticket_id"),
        "proposed_node_id": _portable_id(item["proposed_node_id"], "proposed_node_id"),
        "parent_node_id": _optional_portable_id(item["parent_node_id"], "parent_node_id"),
        "parent_node_artifact_sha256": _optional_digest(
            item["parent_node_artifact_sha256"], "parent_node_artifact_sha256"
        ),
        "generation": _bounded_int(item["generation"], "generation", 0, 1000),
        "expert_role": _enum(item["expert_role"], EXPERT_ROLES, "expert_role"),
        "selected_direction_id": _portable_id(
            item["selected_direction_id"], "selected_direction_id"
        ),
        "mathematical_payload": _validate_mathematical_payload(
            item["mathematical_payload"]
        ),
    }
    _validate_parent_pair(
        result["parent_node_id"], result["parent_node_artifact_sha256"], "expert result"
    )
    digest = _canonical_sha256(result)
    if "expert_result_sha256" in item and item["expert_result_sha256"] != digest:
        raise ValidationError("expert_result_sha256 does not match canonical expert result")
    result["expert_result_sha256"] = digest
    return result


def decide_admission(
    attempt: Mapping[str, Any],
    result: Mapping[str, Any],
    provenance: Mapping[str, Any],
    *,
    outcome: str,
    diagnostic_detail: str | None = None,
) -> dict[str, object]:
    attempt_value = _validate_attempt(attempt)
    result_value = validate_expert_result(result)
    provenance_value = _validate_provenance(provenance)
    closed_outcome = _enum(outcome, ADMISSION_OUTCOMES, "outcome")
    if attempt_value["terminal_status"] != "completed":
        raise ValidationError("admission requires a completed terminal attempt")
    for field in ("run_id", "attempt_id", "ticket_id", "proposed_node_id"):
        if attempt_value[field] != result_value[field]:
            raise ValidationError(f"attempt {field} must match expert result")
    if provenance_value["selected_direction"]["direction_id"] != result_value["selected_direction_id"]:
        raise ValidationError("selected direction must match expert result")
    _validate_parent_pair(
        result_value["parent_node_id"],
        result_value["parent_node_artifact_sha256"],
        "admission",
    )
    if result_value["generation"] == 0 and result_value["parent_node_id"] is not None:
        raise ValidationError("root result cannot have parent identity")
    if result_value["generation"] > 0 and result_value["parent_node_id"] is None:
        raise ValidationError("child result requires parent identity")
    detail = None
    if diagnostic_detail is not None:
        detail = _bounded_str(diagnostic_detail, "diagnostic_detail", 1, 4096)

    direction = provenance_value["selected_direction"]
    if result_value["generation"] > 0 and (
        result_value["parent_node_artifact_sha256"] != direction["source_node_artifact_sha256"]
    ):
        raise ValidationError(
            "child parent artifact must match selected direction source node artifact"
        )
    if closed_outcome == "accepted" and not _is_functioning_payload(
        result_value["mathematical_payload"]
    ):
        closed_outcome = "rejected_nonfunctioning"
        if detail is None:
            detail = "Strict expert result did not meet functioning criteria."
    decision: dict[str, object] = {
        "schema_version": SCHEMA_VERSION_ADMISSION_DECISION,
        "run_id": result_value["run_id"],
        "attempt_id": result_value["attempt_id"],
        "ticket_id": result_value["ticket_id"],
        "proposed_node_id": result_value["proposed_node_id"],
        "expert_result_sha256": result_value["expert_result_sha256"],
        "context_sha256": provenance_value["context_sha256"],
        "call_receipt_sha256": provenance_value["call_receipt_sha256"],
        "leakage_audit_sha256": provenance_value["leakage_audit_sha256"],
        "outcome": closed_outcome,
        "reason_code": ADMISSION_REASON_BY_OUTCOME[closed_outcome],
        "diagnostic_detail": detail,
    }
    decision["admission_decision_sha256"] = _canonical_sha256(decision)
    return decision


def build_mathematical_node(
    admission: Mapping[str, Any],
    result: Mapping[str, Any],
    provenance: Mapping[str, Any],
) -> dict[str, object]:
    decision = _validate_admission_decision(admission)
    result_value = validate_expert_result(result)
    provenance_value = _validate_provenance(provenance)
    if decision["outcome"] != "accepted":
        raise ValidationError("mathematical node requires an accepted admission decision")
    if decision["expert_result_sha256"] != result_value["expert_result_sha256"]:
        raise ValidationError("admission and result digest mismatch")
    for field in ("run_id", "attempt_id", "ticket_id"):
        if decision[field] != result_value[field]:
            raise ValidationError(f"admission {field} must match expert result")
    if decision["proposed_node_id"] != result_value["proposed_node_id"]:
        raise ValidationError("admission proposed node must match expert result")
    if decision["context_sha256"] != provenance_value["context_sha256"]:
        raise ValidationError("admission context digest must match provenance")
    if decision["call_receipt_sha256"] != provenance_value["call_receipt_sha256"]:
        raise ValidationError("admission call receipt digest must match provenance")

    direction = provenance_value["selected_direction"]
    if direction["direction_id"] != result_value["selected_direction_id"]:
        raise ValidationError("selected direction must match expert result")
    direction_digest = _canonical_sha256(direction)
    payload = _copy_mapping(result_value["mathematical_payload"], "mathematical_payload")
    node: dict[str, object] = {
        "schema_version": SCHEMA_VERSION_MATHEMATICAL_NODE,
        "run_id": result_value["run_id"],
        "node_id": result_value["proposed_node_id"],
        "theorem_sha256": provenance_value["theorem_sha256"],
        "leakage_tier": provenance_value["leakage_tier"],
        "parent_node_id": result_value["parent_node_id"],
        "parent_node_artifact_sha256": result_value["parent_node_artifact_sha256"],
        "generation": result_value["generation"],
        "expert_role": result_value["expert_role"],
        "selected_direction": direction,
        "selected_direction_sha256": direction_digest,
        "source_attempt_id": result_value["attempt_id"],
        "source_ticket_id": result_value["ticket_id"],
        "source_expert_result_sha256": result_value["expert_result_sha256"],
        "source_context_sha256": provenance_value["context_sha256"],
        "source_call_receipt_sha256": provenance_value["call_receipt_sha256"],
        "admission_decision_sha256": decision["admission_decision_sha256"],
        "mathematical_payload": payload,
        "mathematical_payload_sha256": _canonical_sha256(payload),
    }
    node["node_artifact_sha256"] = _canonical_sha256(node)
    return validate_mathematical_node(node)


def validate_mathematical_node(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "mathematical node")
    _require_fields(
        item,
        {
            "schema_version",
            "run_id",
            "node_id",
            "theorem_sha256",
            "leakage_tier",
            "parent_node_id",
            "parent_node_artifact_sha256",
            "generation",
            "expert_role",
            "selected_direction",
            "selected_direction_sha256",
            "source_attempt_id",
            "source_ticket_id",
            "source_expert_result_sha256",
            "source_context_sha256",
            "source_call_receipt_sha256",
            "admission_decision_sha256",
            "mathematical_payload",
            "mathematical_payload_sha256",
            "node_artifact_sha256",
        },
        "mathematical node",
    )
    node: dict[str, object] = {
        "schema_version": _const(
            item["schema_version"], SCHEMA_VERSION_MATHEMATICAL_NODE, "schema_version"
        ),
        "run_id": _portable_id(item["run_id"], "run_id"),
        "node_id": _portable_id(item["node_id"], "node_id"),
        "theorem_sha256": _digest(item["theorem_sha256"], "theorem_sha256"),
        "leakage_tier": _enum(item["leakage_tier"], LEAKAGE_TIERS, "leakage_tier"),
        "parent_node_id": _optional_portable_id(item["parent_node_id"], "parent_node_id"),
        "parent_node_artifact_sha256": _optional_digest(
            item["parent_node_artifact_sha256"], "parent_node_artifact_sha256"
        ),
        "generation": _bounded_int(item["generation"], "generation", 0, 1000),
        "expert_role": _enum(item["expert_role"], EXPERT_ROLES, "expert_role"),
        "selected_direction": _validate_selected_direction(item["selected_direction"]),
        "selected_direction_sha256": _digest(
            item["selected_direction_sha256"], "selected_direction_sha256"
        ),
        "source_attempt_id": _portable_id(item["source_attempt_id"], "source_attempt_id"),
        "source_ticket_id": _portable_id(item["source_ticket_id"], "source_ticket_id"),
        "source_expert_result_sha256": _digest(
            item["source_expert_result_sha256"], "source_expert_result_sha256"
        ),
        "source_context_sha256": _digest(
            item["source_context_sha256"], "source_context_sha256"
        ),
        "source_call_receipt_sha256": _digest(
            item["source_call_receipt_sha256"], "source_call_receipt_sha256"
        ),
        "admission_decision_sha256": _digest(
            item["admission_decision_sha256"], "admission_decision_sha256"
        ),
        "mathematical_payload": _validate_mathematical_payload(
            item["mathematical_payload"]
        ),
        "mathematical_payload_sha256": _digest(
            item["mathematical_payload_sha256"], "mathematical_payload_sha256"
        ),
        "node_artifact_sha256": _digest(
            item["node_artifact_sha256"], "node_artifact_sha256"
        ),
    }
    if node["selected_direction_sha256"] != _canonical_sha256(node["selected_direction"]):
        raise ValidationError("selected_direction_sha256 does not match selected_direction")
    if node["mathematical_payload_sha256"] != _canonical_sha256(node["mathematical_payload"]):
        raise ValidationError("mathematical_payload_sha256 does not match payload")
    _validate_parent_pair(
        node["parent_node_id"], node["parent_node_artifact_sha256"], "mathematical node"
    )
    without_digest = dict(node)
    without_digest.pop("node_artifact_sha256")
    if node["node_artifact_sha256"] != _canonical_sha256(without_digest):
        raise ValidationError("node_artifact_sha256 does not match mathematical node")
    return node


def validate_evaluator_result(value: Mapping[str, Any]) -> dict[str, object]:
    return _validate_evaluation_payload(value)


def build_node_evaluation(
    *,
    evaluation_id: str,
    evaluator_index: int,
    ticket_id: str,
    node: Mapping[str, Any],
    context_sha256: str,
    source_kind: str,
    call_receipt_sha256: str | None,
    terminal_ticket_event_sha256: str,
    evaluation_payload: Mapping[str, Any],
) -> dict[str, object]:
    validated_node = validate_mathematical_node(node)
    payload = _validate_evaluation_payload(evaluation_payload)
    kind = _enum(source_kind, SOURCE_KINDS, "source_kind")
    if kind == "provider_output" and call_receipt_sha256 is None:
        raise ValidationError("provider output evaluation requires a call receipt digest")
    if kind == "conservative_fallback":
        _require_all_probe_status(payload, "insufficient_evidence")
    _enforce_structural_p10(validated_node, payload)
    envelope: dict[str, object] = {
        "schema_version": SCHEMA_VERSION_NODE_EVALUATION,
        "evaluation_id": _portable_id(evaluation_id, "evaluation_id"),
        "evaluator_index": _evaluator_index(evaluator_index),
        "ticket_id": _portable_id(ticket_id, "ticket_id"),
        "node_artifact_sha256": validated_node["node_artifact_sha256"],
        "mathematical_payload_sha256": validated_node["mathematical_payload_sha256"],
        "context_sha256": _digest(context_sha256, "context_sha256"),
        "source_kind": kind,
        "call_receipt_sha256": _optional_digest(
            call_receipt_sha256, "call_receipt_sha256"
        ),
        "terminal_ticket_event_sha256": _digest(
            terminal_ticket_event_sha256, "terminal_ticket_event_sha256"
        ),
        "evaluation_payload": payload,
        "evaluation_payload_sha256": _canonical_sha256(payload),
    }
    envelope["node_evaluation_sha256"] = _canonical_sha256(envelope)
    return envelope


def build_conservative_fallback_evaluation(
    *,
    evaluation_id: str,
    evaluator_index: int,
    ticket_id: str,
    node: Mapping[str, Any],
    context_sha256: str,
    terminal_ticket_event_sha256: str,
    call_receipt_sha256: str | None,
) -> dict[str, object]:
    payload = {
        "schema_version": SCHEMA_VERSION_NODE_EVALUATION_PAYLOAD,
        "probes": [
            {
                "probe_id": probe_id,
                "status": "insufficient_evidence",
                "rationale": "Harness conservative fallback for non-completed evaluator.",
            }
            for probe_id in PROOF_PROGRESS_PROBE_IDS
        ],
        "findings": [],
    }
    return build_node_evaluation(
        evaluation_id=evaluation_id,
        evaluator_index=evaluator_index,
        ticket_id=ticket_id,
        node=node,
        context_sha256=context_sha256,
        source_kind="conservative_fallback",
        call_receipt_sha256=call_receipt_sha256,
        terminal_ticket_event_sha256=terminal_ticket_event_sha256,
        evaluation_payload=payload,
    )


def canonical_json_bytes(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")


def canonical_sha256(value: object) -> str:
    return _canonical_sha256(value)


def _canonical_sha256(value: object) -> str:
    return hashlib.sha256(canonical_json_bytes(value)).hexdigest()


def _sha256_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def _validate_attempt(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "attempt")
    _require_fields(
        item,
        {"run_id", "attempt_id", "ticket_id", "proposed_node_id", "terminal_status"},
        "attempt",
    )
    return {
        "run_id": _portable_id(item["run_id"], "attempt.run_id"),
        "attempt_id": _portable_id(item["attempt_id"], "attempt.attempt_id"),
        "ticket_id": _portable_id(item["ticket_id"], "attempt.ticket_id"),
        "proposed_node_id": _portable_id(
            item["proposed_node_id"], "attempt.proposed_node_id"
        ),
        "terminal_status": _enum(
            item["terminal_status"],
            frozenset({"completed", "failed", "malformed", "timed_out", "blocked_resource"}),
            "terminal_status",
        ),
    }


def _validate_provenance(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "provenance")
    _require_fields(
        item,
        {
            "theorem_sha256",
            "leakage_tier",
            "context_sha256",
            "call_receipt_sha256",
            "leakage_audit_sha256",
            "selected_direction",
        },
        "provenance",
    )
    return {
        "theorem_sha256": _digest(item["theorem_sha256"], "theorem_sha256"),
        "leakage_tier": _enum(item["leakage_tier"], LEAKAGE_TIERS, "leakage_tier"),
        "context_sha256": _digest(item["context_sha256"], "context_sha256"),
        "call_receipt_sha256": _digest(
            item["call_receipt_sha256"], "call_receipt_sha256"
        ),
        "leakage_audit_sha256": _digest(
            item["leakage_audit_sha256"], "leakage_audit_sha256"
        ),
        "selected_direction": _validate_selected_direction(item["selected_direction"]),
    }


def _validate_admission_decision(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "admission decision")
    _require_fields(
        item,
        {
            "schema_version",
            "run_id",
            "attempt_id",
            "ticket_id",
            "proposed_node_id",
            "expert_result_sha256",
            "context_sha256",
            "call_receipt_sha256",
            "leakage_audit_sha256",
            "outcome",
            "reason_code",
            "diagnostic_detail",
            "admission_decision_sha256",
        },
        "admission decision",
    )
    decision: dict[str, object] = {
        "schema_version": _const(
            item["schema_version"], SCHEMA_VERSION_ADMISSION_DECISION, "schema_version"
        ),
        "run_id": _portable_id(item["run_id"], "run_id"),
        "attempt_id": _portable_id(item["attempt_id"], "attempt_id"),
        "ticket_id": _portable_id(item["ticket_id"], "ticket_id"),
        "proposed_node_id": _portable_id(item["proposed_node_id"], "proposed_node_id"),
        "expert_result_sha256": _digest(
            item["expert_result_sha256"], "expert_result_sha256"
        ),
        "context_sha256": _digest(item["context_sha256"], "context_sha256"),
        "call_receipt_sha256": _digest(
            item["call_receipt_sha256"], "call_receipt_sha256"
        ),
        "leakage_audit_sha256": _digest(
            item["leakage_audit_sha256"], "leakage_audit_sha256"
        ),
        "outcome": _enum(item["outcome"], ADMISSION_OUTCOMES, "outcome"),
        "reason_code": _bounded_str(item["reason_code"], "reason_code", 1, 128),
        "diagnostic_detail": None
        if item["diagnostic_detail"] is None
        else _bounded_str(item["diagnostic_detail"], "diagnostic_detail", 1, 4096),
        "admission_decision_sha256": _digest(
            item["admission_decision_sha256"], "admission_decision_sha256"
        ),
    }
    if decision["reason_code"] != ADMISSION_REASON_BY_OUTCOME[decision["outcome"]]:
        raise ValidationError("admission reason_code does not match outcome")
    without_digest = dict(decision)
    without_digest.pop("admission_decision_sha256")
    if decision["admission_decision_sha256"] != _canonical_sha256(without_digest):
        raise ValidationError("admission_decision_sha256 does not match decision")
    return decision


def _validate_selected_direction(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "selected_direction")
    _require_fields(
        item,
        {
            "direction_id",
            "kind",
            "statement",
            "strength",
            "recommended_role",
            "source_node_artifact_sha256",
            "source_reconciliation_sha256",
        },
        "selected_direction",
    )
    direction = {
        "direction_id": _portable_id(item["direction_id"], "direction_id"),
        "kind": _enum(item["kind"], DIRECTION_KINDS, "direction.kind"),
        "statement": _bounded_str(item["statement"], "direction.statement", 1, 12_000),
        "strength": _enum(item["strength"], DIRECTION_STRENGTHS, "direction.strength"),
        "recommended_role": _enum(
            item["recommended_role"], EXPERT_ROLES, "direction.recommended_role"
        ),
        "source_node_artifact_sha256": _optional_digest(
            item["source_node_artifact_sha256"], "source_node_artifact_sha256"
        ),
        "source_reconciliation_sha256": _optional_digest(
            item["source_reconciliation_sha256"], "source_reconciliation_sha256"
        ),
    }
    if direction["kind"] == "root_task":
        if direction["strength"] != "root":
            raise ValidationError("root direction strength must be root")
        if direction["source_node_artifact_sha256"] is not None:
            raise ValidationError("root direction cannot reference a node artifact")
        if direction["source_reconciliation_sha256"] is not None:
            raise ValidationError("root direction cannot reference reconciliation")
    else:
        if direction["source_node_artifact_sha256"] is None:
            raise ValidationError("child direction requires source node artifact")
        if (
            direction["kind"] == "evaluator_finding"
            and direction["source_reconciliation_sha256"] is None
        ):
            raise ValidationError("evaluator finding direction requires reconciliation")
    return direction


def _validate_mathematical_payload(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "mathematical_payload")
    _require_fields(
        item,
        {
            "proof_family",
            "mechanism",
            "proved_statements",
            "unproved_obligations",
            "circularity_risks",
            "proposed_directions",
            "endpoint",
            "confidence_basis",
        },
        "mathematical_payload",
    )
    statements = _validate_statements(item["proved_statements"])
    payload = {
        "proof_family": _bounded_str(item["proof_family"], "proof_family", 1, 512),
        "mechanism": _bounded_str(item["mechanism"], "mechanism", 1, 20_000),
        "proved_statements": statements,
        "unproved_obligations": _validate_obligations(item["unproved_obligations"]),
        "circularity_risks": _validate_risks(item["circularity_risks"]),
        "proposed_directions": _validate_proposed_directions(
            item["proposed_directions"]
        ),
        "endpoint": _validate_endpoint(item["endpoint"]),
        "confidence_basis": _bounded_str(
            item["confidence_basis"], "confidence_basis", 1, 8000
        ),
    }
    _validate_statement_graph(statements)
    return payload


def _is_functioning_payload(payload: Mapping[str, object]) -> bool:
    endpoint = _mapping(payload["endpoint"], "endpoint")
    if endpoint["kind"] == "candidate_proof":
        return bool(payload["mechanism"].strip()) and bool(payload["proved_statements"])
    if endpoint["kind"] == "blocker":
        return (
            bool(str(endpoint["text"]).strip())
            and bool(payload["mechanism"].strip())
            and bool(payload["proposed_directions"])
        )
    return False


def _validate_statements(value: Any) -> list[dict[str, object]]:
    if not isinstance(value, list) or len(value) > 64:
        raise ValidationError("proved_statements must be a bounded list")
    seen: set[str] = set()
    statements = []
    for raw in value:
        item = _mapping(raw, "proved statement")
        _require_fields(
            item,
            {"statement_id", "statement", "justification", "depends_on_statement_ids"},
            "proved statement",
        )
        statement_id = _portable_id(item["statement_id"], "statement_id")
        if statement_id in seen:
            raise ValidationError("duplicate proved statement ID")
        seen.add(statement_id)
        dependencies = _portable_id_list(
            item["depends_on_statement_ids"],
            "depends_on_statement_ids",
            0,
            64,
        )
        statements.append(
            {
                "statement_id": statement_id,
                "statement": _bounded_str(item["statement"], "statement", 1, 12_000),
                "justification": _bounded_str(
                    item["justification"], "justification", 1, 20_000
                ),
                "depends_on_statement_ids": dependencies,
            }
        )
    return statements


def _validate_statement_graph(statements: list[dict[str, object]]) -> None:
    ids = {str(statement["statement_id"]) for statement in statements}
    for statement in statements:
        for dependency in statement["depends_on_statement_ids"]:
            if dependency not in ids:
                raise ValidationError("proved statement dependency must be closed")

    visiting: set[str] = set()
    visited: set[str] = set()
    edges = {
        str(statement["statement_id"]): list(statement["depends_on_statement_ids"])
        for statement in statements
    }

    def visit(node_id: str) -> None:
        if node_id in visiting:
            raise ValidationError("proved statement dependency graph must be acyclic")
        if node_id in visited:
            return
        visiting.add(node_id)
        for dependency in edges[node_id]:
            visit(str(dependency))
        visiting.remove(node_id)
        visited.add(node_id)

    for node_id in edges:
        visit(node_id)


def _validate_obligations(value: Any) -> list[dict[str, object]]:
    if not isinstance(value, list) or len(value) > 64:
        raise ValidationError("unproved_obligations must be a bounded list")
    seen: set[str] = set()
    obligations = []
    for raw in value:
        item = _mapping(raw, "unproved obligation")
        _require_fields(
            item,
            {"obligation_id", "statement", "strength"},
            "unproved obligation",
        )
        obligation_id = _portable_id(item["obligation_id"], "obligation_id")
        if obligation_id in seen:
            raise ValidationError("duplicate obligation ID")
        seen.add(obligation_id)
        obligations.append(
            {
                "obligation_id": obligation_id,
                "statement": _bounded_str(item["statement"], "obligation.statement", 1, 12_000),
                "strength": _enum(
                    item["strength"], OBLIGATION_STRENGTHS, "obligation.strength"
                ),
            }
        )
    return obligations


def _validate_risks(value: Any) -> list[dict[str, object]]:
    if not isinstance(value, list) or len(value) > 64:
        raise ValidationError("circularity_risks must be a bounded list")
    seen: set[str] = set()
    risks = []
    for raw in value:
        item = _mapping(raw, "circularity risk")
        _require_fields(item, {"risk_id", "statement", "locator"}, "circularity risk")
        risk_id = _portable_id(item["risk_id"], "risk_id")
        if risk_id in seen:
            raise ValidationError("duplicate circularity risk ID")
        seen.add(risk_id)
        risks.append(
            {
                "risk_id": risk_id,
                "statement": _bounded_str(item["statement"], "risk.statement", 1, 12_000),
                "locator": _bounded_str(item["locator"], "risk.locator", 1, 1024),
            }
        )
    return risks


def _validate_proposed_directions(value: Any) -> list[dict[str, object]]:
    if not isinstance(value, list) or len(value) > 64:
        raise ValidationError("proposed_directions must be a bounded list")
    seen: set[str] = set()
    directions = []
    for raw in value:
        item = _mapping(raw, "proposed direction")
        _require_fields(
            item,
            {"direction_id", "statement", "strength", "recommended_role"},
            "proposed direction",
        )
        direction_id = _portable_id(item["direction_id"], "direction_id")
        if direction_id in seen:
            raise ValidationError("duplicate proposed direction ID")
        seen.add(direction_id)
        directions.append(
            {
                "direction_id": direction_id,
                "statement": _bounded_str(item["statement"], "direction.statement", 1, 12_000),
                "strength": _enum(
                    item["strength"],
                    PROPOSED_DIRECTION_STRENGTHS,
                    "direction.strength",
                ),
                "recommended_role": _enum(
                    item["recommended_role"], EXPERT_ROLES, "direction.recommended_role"
                ),
            }
        )
    return directions


def _validate_endpoint(value: Any) -> dict[str, object]:
    item = _mapping(value, "endpoint")
    _require_fields(item, {"kind", "text"}, "endpoint")
    return {
        "kind": _enum(item["kind"], ENDPOINT_KINDS, "endpoint.kind"),
        "text": _bounded_str(item["text"], "endpoint.text", 1, 250_000),
    }


def _validate_evaluation_payload(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "evaluation payload")
    _require_fields(item, {"schema_version", "probes", "findings"}, "evaluation payload")
    payload = {
        "schema_version": _const(
            item["schema_version"],
            SCHEMA_VERSION_NODE_EVALUATION_PAYLOAD,
            "schema_version",
        ),
        "probes": _validate_probes(item["probes"]),
        "findings": _validate_findings(item["findings"]),
    }
    return payload


def _validate_probes(value: Any) -> list[dict[str, object]]:
    if not isinstance(value, list) or len(value) != len(PROOF_PROGRESS_PROBE_IDS):
        raise ValidationError("evaluation payload must contain exactly ten probes")
    seen: set[str] = set()
    probes = []
    for raw in value:
        item = _mapping(raw, "probe")
        expected = {"probe_id", "status", "rationale"}
        if "candidate_proof_sha256" in item:
            expected.add("candidate_proof_sha256")
        _require_fields(item, expected, "probe")
        probe_id = _enum(item["probe_id"], frozenset(PROOF_PROGRESS_PROBE_IDS), "probe_id")
        if probe_id in seen:
            raise ValidationError("duplicate probe_id")
        seen.add(probe_id)
        probe = {
            "probe_id": probe_id,
            "status": _enum(item["status"], PROBE_STATUSES, "probe.status"),
            "rationale": _bounded_str(item["rationale"], "probe.rationale", 1, 8000),
        }
        if "candidate_proof_sha256" in item:
            probe["candidate_proof_sha256"] = _digest(
                item["candidate_proof_sha256"], "candidate_proof_sha256"
            )
        probes.append(probe)
    if tuple(probe["probe_id"] for probe in probes) != PROOF_PROGRESS_PROBE_IDS:
        raise ValidationError("probe IDs must match the closed probe order")
    return probes


def _validate_findings(value: Any) -> list[dict[str, object]]:
    if not isinstance(value, list) or len(value) > 128:
        raise ValidationError("findings must be a bounded list")
    seen: set[str] = set()
    findings = []
    for raw in value:
        item = _mapping(raw, "finding")
        _require_fields(
            item,
            {"finding_id", "probe_id", "severity", "locator", "statement", "test"},
            "finding",
        )
        finding_id = _portable_id(item["finding_id"], "finding_id")
        if finding_id in seen:
            raise ValidationError("duplicate finding ID")
        seen.add(finding_id)
        findings.append(
            {
                "finding_id": finding_id,
                "probe_id": _enum(
                    item["probe_id"], frozenset(PROOF_PROGRESS_PROBE_IDS), "probe_id"
                ),
                "severity": _enum(item["severity"], FINDING_SEVERITIES, "severity"),
                "locator": _bounded_str(item["locator"], "locator", 1, 1024),
                "statement": _bounded_str(item["statement"], "finding.statement", 1, 12_000),
                "test": _bounded_str(item["test"], "finding.test", 1, 12_000),
            }
        )
    return findings


def _enforce_structural_p10(
    node: Mapping[str, object], evaluation_payload: Mapping[str, object]
) -> None:
    probes = list(evaluation_payload["probes"])
    all_pass = all(probe["status"] == "pass" for probe in probes)
    if not all_pass:
        return
    endpoint = _mapping(node["mathematical_payload"], "mathematical_payload")["endpoint"]
    if _mapping(endpoint, "endpoint")["kind"] != "candidate_proof":
        raise ValidationError("structural P10 pass requires a candidate proof endpoint")


def _require_all_probe_status(payload: Mapping[str, object], status: str) -> None:
    if any(probe["status"] != status for probe in payload["probes"]):
        raise ValidationError(f"fallback probes must all be {status}")


def _project_parent(
    parent_node: Mapping[str, Any] | None, generation: int
) -> dict[str, object] | None:
    if parent_node is None:
        return None
    if generation <= 0:
        raise ValidationError("parent requires child generation")
    node = validate_mathematical_node(parent_node)
    return {
        "node_id": node["node_id"],
        "node_artifact_sha256": node["node_artifact_sha256"],
        "mathematical_payload": _copy_mapping(
            node["mathematical_payload"], "mathematical_payload"
        ),
    }


def _validate_parent_pair(
    parent_node_id: object, parent_node_artifact_sha256: object, label: str
) -> None:
    if (parent_node_id is None) != (parent_node_artifact_sha256 is None):
        raise ValidationError(f"{label} parent identity must be pairwise null or present")


def _mapping(value: Any, label: str) -> Mapping[str, Any]:
    if not isinstance(value, dict):
        raise ValidationError(f"{label} must be an object")
    return value


def _copy_mapping(value: Any, label: str) -> dict[str, object]:
    return json.loads(canonical_json_bytes(_mapping(value, label)))


def _require_fields(value: Mapping[str, Any], expected: set[str], label: str) -> None:
    actual = set(value)
    if actual != expected:
        raise ValidationError(
            f"{label} fields must be {sorted(expected)}, got {sorted(actual)}"
        )


def _const(value: Any, expected: str, label: str) -> str:
    if value != expected:
        raise ValidationError(f"{label} must be {expected!r}")
    return expected


def _enum(value: Any, allowed: frozenset[str], label: str) -> str:
    if not isinstance(value, str) or value not in allowed:
        raise ValidationError(f"{label} must be one of {sorted(allowed)}")
    return value


def _bounded_str(value: Any, label: str, minimum: int, maximum: int) -> str:
    if (
        not isinstance(value, str)
        or len(value) < minimum
        or len(value) > maximum
        or "\0" in value
    ):
        raise ValidationError(
            f"{label} must be a non-NUL string of length {minimum}..{maximum}"
        )
    return value


def _bounded_int(value: Any, label: str, minimum: int, maximum: int) -> int:
    if (
        not isinstance(value, int)
        or isinstance(value, bool)
        or value < minimum
        or value > maximum
    ):
        raise ValidationError(f"{label} must be an integer in {minimum}..{maximum}")
    return value


def _evaluator_index(value: Any) -> int:
    return _bounded_int(value, "evaluator_index", 1, 2)


def _portable_id(value: Any, label: str) -> str:
    text = _bounded_str(value, label, 1, 128)
    if _PORTABLE_ID.fullmatch(text) is None:
        raise ValidationError(f"{label} must be a portable lowercase ID")
    return text


def _optional_portable_id(value: Any, label: str) -> str | None:
    if value is None:
        return None
    return _portable_id(value, label)


def _portable_id_list(value: Any, label: str, minimum: int, maximum: int) -> list[str]:
    if not isinstance(value, list) or len(value) < minimum or len(value) > maximum:
        raise ValidationError(f"{label} must be a bounded list")
    items = [_portable_id(item, label) for item in value]
    if len(set(items)) != len(items):
        raise ValidationError(f"{label} contains duplicate IDs")
    return items


def _digest(value: Any, label: str) -> str:
    if not isinstance(value, str) or _SHA256.fullmatch(value) is None:
        raise ValidationError(f"{label} must be a lowercase SHA-256")
    return value


def _optional_digest(value: Any, label: str) -> str | None:
    if value is None:
        return None
    return _digest(value, label)


def _unique_strings(
    value: Any, label: str, minimum: int, maximum: int
) -> list[str]:
    if not isinstance(value, list) or len(value) < minimum or len(value) > maximum:
        raise ValidationError(f"{label} must be a list with {minimum}..{maximum} entries")
    items = [_bounded_str(item, f"{label} item", 1, 4096) for item in value]
    if len(set(items)) != len(items):
        raise ValidationError(f"{label} must be unique")
    return items
