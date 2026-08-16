from __future__ import annotations

import hashlib
import json
import re
from typing import Any, Mapping, Sequence

from protocol import ValidationError


SCHEMA_VERSION_CORRECTNESS_CONTEXT = "crouzeix-correctness-context/v1"
SCHEMA_VERSION_CORRECTNESS_REVIEW_PAYLOAD = (
    "crouzeix-correctness-review-payload/v1"
)
SCHEMA_VERSION_CORRECTNESS_REVIEW = "crouzeix-correctness-review/v1"
SCHEMA_VERSION_RECONCILIATION = "crouzeix-correctness-reconciliation/v1"
SCHEMA_VERSION_REPAIR_REQUEST = "crouzeix-repair-request/v1"
SCHEMA_VERSION_REPAIR_PAYLOAD = "crouzeix-repair-payload/v1"
SCHEMA_VERSION_REPAIR_RESULT = "crouzeix-repair-result/v1"
SCHEMA_VERSION_CLASSIFICATION_CONTEXT = (
    "crouzeix-mechanism-classification-context/v1"
)
SCHEMA_VERSION_CLASSIFICATION_PAYLOAD = (
    "crouzeix-mechanism-classification-payload/v1"
)
SCHEMA_VERSION_CLASSIFICATION_RESULT = "crouzeix-mechanism-classification/v1"

CORRECTNESS_OUTCOMES = frozenset(
    {"complete", "incomplete", "invalid", "indeterminate"}
)
FINDING_SEVERITIES = frozenset({"critical", "major", "minor"})
FINDING_DISPOSITIONS = frozenset(
    {"fixed", "rejected_with_reason", "unresolved"}
)
MECHANISM_SIMILARITIES = frozenset(
    {"same", "related", "distinct", "insufficient_evidence"}
)

CORRECTNESS_CONTEXT_FIELDS = frozenset(
    {
        "schema_version",
        "anonymous_candidate_id",
        "reviewer_index",
        "ticket_id",
        "theorem_text",
        "theorem_sha256",
        "candidate_text",
        "candidate_sha256",
        "allowed_tools",
        "forbidden_sources",
        "delegation_allowed",
        "completion_criteria",
        "result_schema",
    }
)
FINDING_FIELDS = frozenset(
    {
        "finding_id",
        "severity",
        "locator",
        "statement",
        "falsifying_test_or_gap",
    }
)
OBLIGATION_FIELDS = frozenset({"obligation_id", "locator", "statement"})
CORRECTNESS_PAYLOAD_FIELDS = frozenset(
    {
        "schema_version",
        "anonymous_candidate_id",
        "candidate_sha256",
        "reviewer_index",
        "outcome",
        "findings",
        "theorem_strength_obligations",
    }
)
CORRECTNESS_PAYLOAD_WITH_DIGEST_FIELDS = CORRECTNESS_PAYLOAD_FIELDS | frozenset(
    {"correctness_review_payload_sha256"}
)
REVIEW_ENVELOPE_FIELDS = frozenset(
    {
        "schema_version",
        "review_id",
        "ticket_id",
        "ticket_sha256",
        "context_sha256",
        "prompt_sha256",
        "schema_sha256",
        "call_receipt_sha256",
        "terminal_ticket_event_sha256",
        "review_payload",
        "review_payload_sha256",
    }
)
REVIEW_ENVELOPE_WITH_DIGEST_FIELDS = REVIEW_ENVELOPE_FIELDS | frozenset(
    {"correctness_review_sha256"}
)
DISPOSITION_FIELDS = frozenset(
    {
        "namespaced_finding_id",
        "disposition",
        "rationale",
    }
)
LEDGER_DISPOSITION_FIELDS = frozenset(
    {
        "namespaced_finding_id",
        "disposition",
        "severity",
        "locator",
        "statement",
        "falsifying_test_or_gap",
        "rationale",
    }
)
RECONCILIATION_FIELDS = frozenset(
    {
        "schema_version",
        "reconciliation_id",
        "candidate_sha256",
        "outcome",
        "finding_dispositions",
    }
)
RECONCILIATION_WITH_DIGEST_FIELDS = RECONCILIATION_FIELDS | frozenset(
    {"finding_ledger_sha256"}
)
REPAIR_REQUEST_FIELDS = frozenset(
    {
        "schema_version",
        "repair_id",
        "ticket_id",
        "parent_candidate_sha256",
        "parent_candidate_byte_count",
        "parent_candidate_text",
        "finding_ledger_sha256",
        "unresolved_findings",
        "max_provider_calls",
        "allowed_tools",
        "delegation_allowed",
        "result_schema",
    }
)
REPAIR_PAYLOAD_FIELDS = frozenset(
    {
        "schema_version",
        "repair_id",
        "parent_candidate_sha256",
        "finding_ledger_sha256",
        "candidate_text",
        "dispositions",
        "unproved_obligations",
    }
)
REPAIR_PAYLOAD_WITH_DIGEST_FIELDS = REPAIR_PAYLOAD_FIELDS | frozenset(
    {"repair_payload_sha256"}
)
CLASSIFICATION_CONTEXT_FIELDS = frozenset(
    {
        "schema_version",
        "classification_id",
        "ticket_id",
        "theorem_text",
        "theorem_sha256",
        "candidate_text",
        "candidate_sha256",
        "frozen_correctness_sha256",
        "correctness_bytes_sha256",
        "reference_cards",
        "allowed_tools",
        "delegation_allowed",
        "result_schema",
    }
)
REFERENCE_CARD_FIELDS = frozenset(
    {"reference_id", "mechanism_summary", "reference_sha256"}
)
SIMILARITY_FIELDS = frozenset(
    {"reference_id", "similarity", "locator", "rationale"}
)
CLASSIFICATION_PAYLOAD_FIELDS = frozenset(
    {
        "schema_version",
        "classification_id",
        "candidate_sha256",
        "frozen_correctness_sha256",
        "similarities",
        "candidate_rewrite",
        "correctness_outcome",
    }
)
CLASSIFICATION_PAYLOAD_WITH_DIGEST_FIELDS = CLASSIFICATION_PAYLOAD_FIELDS | frozenset(
    {"mechanism_classification_payload_sha256"}
)

_PORTABLE_ID = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")
_RUNTIME_ID = re.compile(r"^[A-Za-z0-9._-]{1,128}$")
_SHA256 = re.compile(r"^[0-9a-f]{64}$")


def build_correctness_context(
    *,
    theorem_text: str,
    candidate_text: str,
    candidate_projection: Mapping[str, Any],
    reviewer_index: int,
    ticket_id: str,
) -> dict[str, object]:
    projection = _mapping(candidate_projection, "candidate projection")
    candidate = _bounded_str(candidate_text, "candidate_text", 1, 2 * 1024 * 1024)
    candidate_sha256 = _sha256_text(candidate)
    if projection.get("candidate_sha256") != candidate_sha256:
        raise ValidationError("candidate projection digest does not match candidate text")
    byte_count = projection.get("candidate_byte_count")
    if byte_count != len(candidate.encode("utf-8")):
        raise ValidationError("candidate projection byte count does not match candidate text")
    reviewer = _reviewer_index(reviewer_index)
    return {
        "schema_version": SCHEMA_VERSION_CORRECTNESS_CONTEXT,
        "anonymous_candidate_id": f"anon-{_portable_id(projection.get('projection_id'), 'projection_id')}",
        "reviewer_index": reviewer,
        "ticket_id": _runtime_id(ticket_id, "ticket_id"),
        "theorem_text": _bounded_str(theorem_text, "theorem_text", 1, 200_000),
        "theorem_sha256": _sha256_text(theorem_text),
        "candidate_text": candidate,
        "candidate_sha256": candidate_sha256,
        "allowed_tools": ["Write"],
        "forbidden_sources": [
            "arm metadata",
            "runtime identity",
            "budget metadata",
            "ancestor metadata",
            "similarity labels",
            "earlier defects",
            "peer-review output",
        ],
        "delegation_allowed": False,
        "completion_criteria": (
            "Return one strict correctness review payload with locator-based "
            "findings using only the theorem and anonymous candidate text."
        ),
        "result_schema": "correctness_review_payload",
    }


def validate_correctness_context(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "correctness context")
    _require_fields(item, CORRECTNESS_CONTEXT_FIELDS, "correctness context")
    theorem = _bounded_str(item["theorem_text"], "theorem_text", 1, 200_000)
    candidate = _bounded_str(item["candidate_text"], "candidate_text", 1, 2 * 1024 * 1024)
    context = {
        "schema_version": _const(
            item["schema_version"],
            SCHEMA_VERSION_CORRECTNESS_CONTEXT,
            "schema_version",
        ),
        "anonymous_candidate_id": _portable_id(
            item["anonymous_candidate_id"], "anonymous_candidate_id"
        ),
        "reviewer_index": _reviewer_index(item["reviewer_index"]),
        "ticket_id": _runtime_id(item["ticket_id"], "ticket_id"),
        "theorem_text": theorem,
        "theorem_sha256": _digest(item["theorem_sha256"], "theorem_sha256"),
        "candidate_text": candidate,
        "candidate_sha256": _digest(item["candidate_sha256"], "candidate_sha256"),
        "allowed_tools": _string_list(item["allowed_tools"], "allowed_tools", 1, 8),
        "forbidden_sources": _string_list(
            item["forbidden_sources"], "forbidden_sources", 1, 16
        ),
        "delegation_allowed": item["delegation_allowed"],
        "completion_criteria": _bounded_str(
            item["completion_criteria"], "completion_criteria", 1, 4096
        ),
        "result_schema": _const(
            item["result_schema"], "correctness_review_payload", "result_schema"
        ),
    }
    if context["theorem_sha256"] != _sha256_text(theorem):
        raise ValidationError("theorem_sha256 does not match theorem_text")
    if context["candidate_sha256"] != _sha256_text(candidate):
        raise ValidationError("candidate_sha256 does not match candidate_text")
    if context["allowed_tools"] != ["Write"]:
        raise ValidationError("correctness context allowed_tools must be exactly Write")
    if context["delegation_allowed"] is not False:
        raise ValidationError("correctness context delegation_allowed must be false")
    return context


def validate_correctness_review_payload(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "correctness review payload")
    expected = (
        CORRECTNESS_PAYLOAD_WITH_DIGEST_FIELDS
        if "correctness_review_payload_sha256" in item
        else CORRECTNESS_PAYLOAD_FIELDS
    )
    _require_fields(item, expected, "correctness review payload")
    findings = _validate_findings(item["findings"])
    payload: dict[str, object] = {
        "schema_version": _const(
            item["schema_version"],
            SCHEMA_VERSION_CORRECTNESS_REVIEW_PAYLOAD,
            "schema_version",
        ),
        "anonymous_candidate_id": _portable_id(
            item["anonymous_candidate_id"], "anonymous_candidate_id"
        ),
        "candidate_sha256": _digest(item["candidate_sha256"], "candidate_sha256"),
        "reviewer_index": _reviewer_index(item["reviewer_index"]),
        "outcome": _enum(item["outcome"], CORRECTNESS_OUTCOMES, "outcome"),
        "findings": findings,
        "theorem_strength_obligations": _validate_obligations(
            item["theorem_strength_obligations"]
        ),
    }
    if payload["outcome"] == "complete" and findings:
        raise ValidationError("complete correctness outcome cannot include findings")
    digest = _canonical_sha256(payload)
    if (
        "correctness_review_payload_sha256" in item
        and item["correctness_review_payload_sha256"] != digest
    ):
        raise ValidationError("correctness_review_payload_sha256 does not match payload")
    payload["correctness_review_payload_sha256"] = digest
    return payload


def build_correctness_review_envelope(
    *,
    review_id: str,
    ticket_id: str,
    ticket_sha256: str,
    context_sha256: str,
    prompt_sha256: str,
    schema_sha256: str,
    call_receipt_sha256: str,
    terminal_ticket_event_sha256: str,
    review_payload: Mapping[str, Any],
) -> dict[str, object]:
    payload = validate_correctness_review_payload(review_payload)
    envelope: dict[str, object] = {
        "schema_version": SCHEMA_VERSION_CORRECTNESS_REVIEW,
        "review_id": _runtime_id(review_id, "review_id"),
        "ticket_id": _runtime_id(ticket_id, "ticket_id"),
        "ticket_sha256": _digest(ticket_sha256, "ticket_sha256"),
        "context_sha256": _digest(context_sha256, "context_sha256"),
        "prompt_sha256": _digest(prompt_sha256, "prompt_sha256"),
        "schema_sha256": _digest(schema_sha256, "schema_sha256"),
        "call_receipt_sha256": _digest(call_receipt_sha256, "call_receipt_sha256"),
        "terminal_ticket_event_sha256": _digest(
            terminal_ticket_event_sha256, "terminal_ticket_event_sha256"
        ),
        "review_payload": payload,
        "review_payload_sha256": payload["correctness_review_payload_sha256"],
    }
    envelope["correctness_review_sha256"] = _canonical_sha256(envelope)
    return envelope


def validate_correctness_review_envelope(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "correctness review")
    _require_fields(
        item, REVIEW_ENVELOPE_WITH_DIGEST_FIELDS, "correctness review"
    )
    payload = validate_correctness_review_payload(item["review_payload"])
    envelope = {
        "schema_version": _const(
            item["schema_version"], SCHEMA_VERSION_CORRECTNESS_REVIEW, "schema_version"
        ),
        "review_id": _runtime_id(item["review_id"], "review_id"),
        "ticket_id": _runtime_id(item["ticket_id"], "ticket_id"),
        "ticket_sha256": _digest(item["ticket_sha256"], "ticket_sha256"),
        "context_sha256": _digest(item["context_sha256"], "context_sha256"),
        "prompt_sha256": _digest(item["prompt_sha256"], "prompt_sha256"),
        "schema_sha256": _digest(item["schema_sha256"], "schema_sha256"),
        "call_receipt_sha256": _digest(
            item["call_receipt_sha256"], "call_receipt_sha256"
        ),
        "terminal_ticket_event_sha256": _digest(
            item["terminal_ticket_event_sha256"], "terminal_ticket_event_sha256"
        ),
        "review_payload": payload,
        "review_payload_sha256": _digest(
            item["review_payload_sha256"], "review_payload_sha256"
        ),
        "correctness_review_sha256": _digest(
            item["correctness_review_sha256"], "correctness_review_sha256"
        ),
    }
    if envelope["review_payload_sha256"] != payload["correctness_review_payload_sha256"]:
        raise ValidationError("review_payload_sha256 does not match payload")
    without_digest = dict(envelope)
    without_digest.pop("correctness_review_sha256")
    if envelope["correctness_review_sha256"] != _canonical_sha256(without_digest):
        raise ValidationError("correctness_review_sha256 does not match review")
    return envelope


def reconcile_correctness_reviews(
    *,
    reconciliation_id: str,
    candidate_sha256: str,
    reviews: Sequence[Mapping[str, Any]],
    dispositions: Sequence[Mapping[str, Any]],
) -> dict[str, object]:
    digest = _digest(candidate_sha256, "candidate_sha256")
    if len(reviews) != 2:
        raise ValidationError("correctness reconciliation requires exactly two reviews")
    payloads = [_extract_review_payload(review) for review in reviews]
    reviewer_indices = [payload["reviewer_index"] for payload in payloads]
    if sorted(reviewer_indices) != [1, 2]:
        raise ValidationError("correctness reconciliation requires reviewer indices 1 and 2")
    for payload in payloads:
        if payload["candidate_sha256"] != digest:
            raise ValidationError("review candidate_sha256 does not match reconciliation")
    namespaced: dict[str, dict[str, object]] = {}
    for payload in sorted(payloads, key=lambda item: int(item["reviewer_index"])):
        prefix = f"r{payload['reviewer_index']}:"
        for raw_finding in payload["findings"]:
            finding = dict(raw_finding)
            namespaced_id = prefix + str(finding["finding_id"])
            if namespaced_id in namespaced:
                raise ValidationError("duplicate namespaced finding ID")
            namespaced[namespaced_id] = finding

    disposition_rows = [_validate_disposition(item) for item in dispositions]
    observed_ids = [str(item["namespaced_finding_id"]) for item in disposition_rows]
    if len(set(observed_ids)) != len(observed_ids):
        raise ValidationError("each finding must receive exactly one disposition")
    if set(observed_ids) != set(namespaced):
        raise ValidationError("each finding must receive exactly one disposition")

    normalized_rows = []
    for row in sorted(disposition_rows, key=lambda item: str(item["namespaced_finding_id"])):
        source = namespaced[str(row["namespaced_finding_id"])]
        normalized_rows.append(
            {
                "namespaced_finding_id": row["namespaced_finding_id"],
                "disposition": row["disposition"],
                "severity": source["severity"],
                "locator": source["locator"],
                "statement": source["statement"],
                "falsifying_test_or_gap": source["falsifying_test_or_gap"],
                "rationale": row["rationale"],
            }
        )

    outcomes = {str(payload["outcome"]) for payload in payloads}
    if "invalid" in outcomes:
        outcome = "invalid"
    elif "incomplete" in outcomes or any(
        row["disposition"] == "unresolved" for row in normalized_rows
    ):
        outcome = "incomplete"
    elif outcomes == {"complete"}:
        outcome = "complete"
    else:
        outcome = "indeterminate"
    ledger: dict[str, object] = {
        "schema_version": SCHEMA_VERSION_RECONCILIATION,
        "reconciliation_id": _runtime_id(reconciliation_id, "reconciliation_id"),
        "candidate_sha256": digest,
        "outcome": outcome,
        "finding_dispositions": normalized_rows,
    }
    ledger["finding_ledger_sha256"] = _canonical_sha256(ledger)
    return ledger


def validate_finding_ledger(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "finding ledger")
    _require_fields(item, RECONCILIATION_WITH_DIGEST_FIELDS, "finding ledger")
    rows = _validate_ledger_dispositions(item["finding_dispositions"])
    ledger = {
        "schema_version": _const(
            item["schema_version"], SCHEMA_VERSION_RECONCILIATION, "schema_version"
        ),
        "reconciliation_id": _runtime_id(
            item["reconciliation_id"], "reconciliation_id"
        ),
        "candidate_sha256": _digest(item["candidate_sha256"], "candidate_sha256"),
        "outcome": _enum(item["outcome"], CORRECTNESS_OUTCOMES, "outcome"),
        "finding_dispositions": rows,
        "finding_ledger_sha256": _digest(
            item["finding_ledger_sha256"], "finding_ledger_sha256"
        ),
    }
    without_digest = dict(ledger)
    without_digest.pop("finding_ledger_sha256")
    if ledger["finding_ledger_sha256"] != _canonical_sha256(without_digest):
        raise ValidationError("finding_ledger_sha256 does not match ledger")
    return ledger


def build_repair_request(
    *,
    repair_id: str,
    parent_candidate_text: str,
    finding_ledger: Mapping[str, Any],
    ticket_id: str,
) -> dict[str, object]:
    ledger = validate_finding_ledger(finding_ledger)
    parent_text = _bounded_str(
        parent_candidate_text, "parent_candidate_text", 1, 2 * 1024 * 1024
    )
    parent_sha256 = _sha256_text(parent_text)
    if parent_sha256 != ledger["candidate_sha256"]:
        raise ValidationError("parent candidate digest does not match finding ledger")
    unresolved = [
        row
        for row in ledger["finding_dispositions"]
        if row["disposition"] == "unresolved"
    ]
    if not unresolved:
        raise ValidationError("repair requires at least one unresolved finding")
    return {
        "schema_version": SCHEMA_VERSION_REPAIR_REQUEST,
        "repair_id": _runtime_id(repair_id, "repair_id"),
        "ticket_id": _runtime_id(ticket_id, "ticket_id"),
        "parent_candidate_sha256": parent_sha256,
        "parent_candidate_byte_count": len(parent_text.encode("utf-8")),
        "parent_candidate_text": parent_text,
        "finding_ledger_sha256": ledger["finding_ledger_sha256"],
        "unresolved_findings": unresolved,
        "max_provider_calls": 1,
        "allowed_tools": ["Write"],
        "delegation_allowed": False,
        "result_schema": "repair_payload",
    }


def validate_repair_request(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "repair request")
    _require_fields(item, REPAIR_REQUEST_FIELDS, "repair request")
    parent_text = _bounded_str(
        item["parent_candidate_text"], "parent_candidate_text", 1, 2 * 1024 * 1024
    )
    request = {
        "schema_version": _const(
            item["schema_version"], SCHEMA_VERSION_REPAIR_REQUEST, "schema_version"
        ),
        "repair_id": _runtime_id(item["repair_id"], "repair_id"),
        "ticket_id": _runtime_id(item["ticket_id"], "ticket_id"),
        "parent_candidate_sha256": _digest(
            item["parent_candidate_sha256"], "parent_candidate_sha256"
        ),
        "parent_candidate_byte_count": _bounded_int(
            item["parent_candidate_byte_count"], "parent_candidate_byte_count", 1, 2 * 1024 * 1024
        ),
        "parent_candidate_text": parent_text,
        "finding_ledger_sha256": _digest(
            item["finding_ledger_sha256"], "finding_ledger_sha256"
        ),
        "unresolved_findings": _validate_ledger_dispositions(
            item["unresolved_findings"]
        ),
        "max_provider_calls": _bounded_int(
            item["max_provider_calls"], "max_provider_calls", 1, 1
        ),
        "allowed_tools": _string_list(item["allowed_tools"], "allowed_tools", 1, 8),
        "delegation_allowed": item["delegation_allowed"],
        "result_schema": _const(item["result_schema"], "repair_payload", "result_schema"),
    }
    if request["parent_candidate_sha256"] != _sha256_text(parent_text):
        raise ValidationError("parent_candidate_sha256 does not match parent_candidate_text")
    if request["parent_candidate_byte_count"] != len(parent_text.encode("utf-8")):
        raise ValidationError("parent_candidate_byte_count does not match parent_candidate_text")
    if request["allowed_tools"] != ["Write"]:
        raise ValidationError("repair request allowed_tools must be exactly Write")
    if request["delegation_allowed"] is not False:
        raise ValidationError("repair request delegation_allowed must be false")
    if any(row["disposition"] != "unresolved" for row in request["unresolved_findings"]):
        raise ValidationError("repair request unresolved_findings must be unresolved")
    return request


def validate_repair_payload(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "repair payload")
    expected = (
        REPAIR_PAYLOAD_WITH_DIGEST_FIELDS
        if "repair_payload_sha256" in item
        else REPAIR_PAYLOAD_FIELDS
    )
    _require_fields(item, expected, "repair payload")
    dispositions = []
    for raw in _list(item["dispositions"], "repair dispositions", 0, 128):
        row = _mapping(raw, "repair disposition")
        _require_fields(row, DISPOSITION_FIELDS, "repair disposition")
        dispositions.append(_validate_disposition(row))
    payload: dict[str, object] = {
        "schema_version": _const(
            item["schema_version"], SCHEMA_VERSION_REPAIR_PAYLOAD, "schema_version"
        ),
        "repair_id": _runtime_id(item["repair_id"], "repair_id"),
        "parent_candidate_sha256": _digest(
            item["parent_candidate_sha256"], "parent_candidate_sha256"
        ),
        "finding_ledger_sha256": _digest(
            item["finding_ledger_sha256"], "finding_ledger_sha256"
        ),
        "candidate_text": _bounded_str(
            item["candidate_text"], "candidate_text", 1, 2 * 1024 * 1024
        ),
        "dispositions": dispositions,
        "unproved_obligations": _validate_obligations(item["unproved_obligations"]),
    }
    digest = _canonical_sha256(payload)
    if "repair_payload_sha256" in item and item["repair_payload_sha256"] != digest:
        raise ValidationError("repair_payload_sha256 does not match payload")
    payload["repair_payload_sha256"] = digest
    return payload


def build_repair_result(
    *,
    repair_id: str,
    parent_candidate_sha256: str,
    finding_ledger_sha256: str,
    repair_payload: Mapping[str, Any],
) -> dict[str, object]:
    payload = validate_repair_payload(repair_payload)
    parent_digest = _digest(parent_candidate_sha256, "parent_candidate_sha256")
    ledger_digest = _digest(finding_ledger_sha256, "finding_ledger_sha256")
    if payload["parent_candidate_sha256"] != parent_digest:
        raise ValidationError("repair payload parent digest mismatch")
    if payload["finding_ledger_sha256"] != ledger_digest:
        raise ValidationError("repair payload ledger digest mismatch")
    candidate_text = str(payload["candidate_text"])
    result: dict[str, object] = {
        "schema_version": SCHEMA_VERSION_REPAIR_RESULT,
        "repair_id": _runtime_id(repair_id, "repair_id"),
        "parent_candidate_sha256": parent_digest,
        "finding_ledger_sha256": ledger_digest,
        "repaired_candidate_sha256": _sha256_text(candidate_text),
        "repaired_candidate_byte_count": len(candidate_text.encode("utf-8")),
        "changed_candidate_bytes": _sha256_text(candidate_text) != parent_digest,
        "repair_payload": payload,
        "repair_payload_sha256": payload["repair_payload_sha256"],
    }
    result["repair_result_sha256"] = _canonical_sha256(result)
    return result


def build_mechanism_classification_context(
    *,
    classification_id: str,
    theorem_text: str,
    candidate_text: str,
    frozen_correctness: Mapping[str, Any],
    reference_cards: Sequence[Mapping[str, Any]],
    ticket_id: str,
) -> dict[str, object]:
    frozen = _mapping(frozen_correctness, "frozen correctness")
    candidate = _bounded_str(candidate_text, "candidate_text", 1, 2 * 1024 * 1024)
    candidate_sha256 = _sha256_text(candidate)
    if frozen.get("candidate_sha256") != candidate_sha256:
        raise ValidationError("frozen correctness candidate digest mismatch")
    references = [_validate_reference_card(card) for card in reference_cards]
    if not references:
        raise ValidationError("classification requires at least one reference card")
    return {
        "schema_version": SCHEMA_VERSION_CLASSIFICATION_CONTEXT,
        "classification_id": _runtime_id(classification_id, "classification_id"),
        "ticket_id": _runtime_id(ticket_id, "ticket_id"),
        "theorem_text": _bounded_str(theorem_text, "theorem_text", 1, 200_000),
        "theorem_sha256": _sha256_text(theorem_text),
        "candidate_text": candidate,
        "candidate_sha256": candidate_sha256,
        "frozen_correctness_sha256": _digest(
            frozen.get("correctness_reconciliation_sha256"),
            "correctness_reconciliation_sha256",
        ),
        "correctness_bytes_sha256": _digest(
            frozen.get("correctness_bytes_sha256"), "correctness_bytes_sha256"
        ),
        "reference_cards": references,
        "allowed_tools": ["Write"],
        "delegation_allowed": False,
        "result_schema": "mechanism_classification_payload",
    }


def validate_mechanism_classification_context(
    value: Mapping[str, Any]
) -> dict[str, object]:
    item = _mapping(value, "mechanism classification context")
    _require_fields(
        item, CLASSIFICATION_CONTEXT_FIELDS, "mechanism classification context"
    )
    theorem = _bounded_str(item["theorem_text"], "theorem_text", 1, 200_000)
    candidate = _bounded_str(item["candidate_text"], "candidate_text", 1, 2 * 1024 * 1024)
    context = {
        "schema_version": _const(
            item["schema_version"],
            SCHEMA_VERSION_CLASSIFICATION_CONTEXT,
            "schema_version",
        ),
        "classification_id": _runtime_id(item["classification_id"], "classification_id"),
        "ticket_id": _runtime_id(item["ticket_id"], "ticket_id"),
        "theorem_text": theorem,
        "theorem_sha256": _digest(item["theorem_sha256"], "theorem_sha256"),
        "candidate_text": candidate,
        "candidate_sha256": _digest(item["candidate_sha256"], "candidate_sha256"),
        "frozen_correctness_sha256": _digest(
            item["frozen_correctness_sha256"], "frozen_correctness_sha256"
        ),
        "correctness_bytes_sha256": _digest(
            item["correctness_bytes_sha256"], "correctness_bytes_sha256"
        ),
        "reference_cards": [
            _validate_reference_card(card)
            for card in _list(item["reference_cards"], "reference_cards", 1, 16)
        ],
        "allowed_tools": _string_list(item["allowed_tools"], "allowed_tools", 1, 8),
        "delegation_allowed": item["delegation_allowed"],
        "result_schema": _const(
            item["result_schema"],
            "mechanism_classification_payload",
            "result_schema",
        ),
    }
    if context["theorem_sha256"] != _sha256_text(theorem):
        raise ValidationError("theorem_sha256 does not match theorem_text")
    if context["candidate_sha256"] != _sha256_text(candidate):
        raise ValidationError("candidate_sha256 does not match candidate_text")
    if context["allowed_tools"] != ["Write"]:
        raise ValidationError("classification context allowed_tools must be exactly Write")
    if context["delegation_allowed"] is not False:
        raise ValidationError("classification context delegation_allowed must be false")
    return context


def validate_mechanism_classification_payload(
    value: Mapping[str, Any]
) -> dict[str, object]:
    item = _mapping(value, "mechanism classification payload")
    expected = (
        CLASSIFICATION_PAYLOAD_WITH_DIGEST_FIELDS
        if "mechanism_classification_payload_sha256" in item
        else CLASSIFICATION_PAYLOAD_FIELDS
    )
    _require_fields(item, expected, "mechanism classification payload")
    if item["candidate_rewrite"] is not None:
        raise ValidationError("mechanism classification cannot rewrite candidate bytes")
    if item["correctness_outcome"] is not None:
        raise ValidationError("mechanism classification cannot modify correctness outcome")
    similarities = []
    for raw in _list(item["similarities"], "similarities", 0, 16):
        row = _mapping(raw, "similarity")
        _require_fields(row, SIMILARITY_FIELDS, "similarity")
        similarities.append(
            {
                "reference_id": _portable_id(row["reference_id"], "reference_id"),
                "similarity": _enum(
                    row["similarity"], MECHANISM_SIMILARITIES, "similarity"
                ),
                "locator": _bounded_str(row["locator"], "locator", 1, 4096),
                "rationale": _bounded_str(row["rationale"], "rationale", 1, 4096),
            }
        )
    payload: dict[str, object] = {
        "schema_version": _const(
            item["schema_version"],
            SCHEMA_VERSION_CLASSIFICATION_PAYLOAD,
            "schema_version",
        ),
        "classification_id": _runtime_id(item["classification_id"], "classification_id"),
        "candidate_sha256": _digest(item["candidate_sha256"], "candidate_sha256"),
        "frozen_correctness_sha256": _digest(
            item["frozen_correctness_sha256"], "frozen_correctness_sha256"
        ),
        "similarities": similarities,
        "candidate_rewrite": None,
        "correctness_outcome": None,
    }
    digest = _canonical_sha256(payload)
    if (
        "mechanism_classification_payload_sha256" in item
        and item["mechanism_classification_payload_sha256"] != digest
    ):
        raise ValidationError(
            "mechanism_classification_payload_sha256 does not match payload"
        )
    payload["mechanism_classification_payload_sha256"] = digest
    return payload


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


def _extract_review_payload(review: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(review, "correctness review")
    if item.get("schema_version") == SCHEMA_VERSION_CORRECTNESS_REVIEW:
        return validate_correctness_review_envelope(item)["review_payload"]  # type: ignore[return-value]
    return validate_correctness_review_payload(item)


def _validate_findings(value: Any) -> list[dict[str, object]]:
    findings = []
    seen: set[str] = set()
    for raw in _list(value, "findings", 0, 128):
        item = _mapping(raw, "finding")
        _require_fields(item, FINDING_FIELDS, "finding")
        finding_id = _portable_id(item["finding_id"], "finding_id")
        if finding_id in seen:
            raise ValidationError("duplicate finding_id")
        seen.add(finding_id)
        findings.append(
            {
                "finding_id": finding_id,
                "severity": _enum(item["severity"], FINDING_SEVERITIES, "severity"),
                "locator": _bounded_str(item["locator"], "locator", 1, 4096),
                "statement": _bounded_str(item["statement"], "statement", 1, 12000),
                "falsifying_test_or_gap": _bounded_str(
                    item["falsifying_test_or_gap"],
                    "falsifying_test_or_gap",
                    1,
                    12000,
                ),
            }
        )
    return findings


def _validate_obligations(value: Any) -> list[dict[str, object]]:
    obligations = []
    seen: set[str] = set()
    for raw in _list(value, "theorem_strength_obligations", 0, 128):
        item = _mapping(raw, "theorem strength obligation")
        _require_fields(item, OBLIGATION_FIELDS, "theorem strength obligation")
        obligation_id = _portable_id(item["obligation_id"], "obligation_id")
        if obligation_id in seen:
            raise ValidationError("duplicate obligation_id")
        seen.add(obligation_id)
        obligations.append(
            {
                "obligation_id": obligation_id,
                "locator": _bounded_str(item["locator"], "locator", 1, 4096),
                "statement": _bounded_str(item["statement"], "statement", 1, 12000),
            }
        )
    return obligations


def _validate_disposition(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "finding disposition")
    _require_fields(item, DISPOSITION_FIELDS, "finding disposition")
    return {
        "namespaced_finding_id": _namespaced_finding_id(
            item["namespaced_finding_id"], "namespaced_finding_id"
        ),
        "disposition": _enum(
            item["disposition"], FINDING_DISPOSITIONS, "disposition"
        ),
        "rationale": _bounded_str(item["rationale"], "rationale", 1, 4096),
    }


def _validate_ledger_dispositions(value: Any) -> list[dict[str, object]]:
    rows = []
    seen: set[str] = set()
    for raw in _list(value, "finding_dispositions", 0, 128):
        item = _mapping(raw, "ledger finding disposition")
        _require_fields(item, LEDGER_DISPOSITION_FIELDS, "ledger finding disposition")
        namespaced_id = _namespaced_finding_id(
            item["namespaced_finding_id"], "namespaced_finding_id"
        )
        if namespaced_id in seen:
            raise ValidationError("duplicate namespaced_finding_id")
        seen.add(namespaced_id)
        rows.append(
            {
                "namespaced_finding_id": namespaced_id,
                "disposition": _enum(
                    item["disposition"], FINDING_DISPOSITIONS, "disposition"
                ),
                "severity": _enum(item["severity"], FINDING_SEVERITIES, "severity"),
                "locator": _bounded_str(item["locator"], "locator", 1, 4096),
                "statement": _bounded_str(item["statement"], "statement", 1, 12000),
                "falsifying_test_or_gap": _bounded_str(
                    item["falsifying_test_or_gap"],
                    "falsifying_test_or_gap",
                    1,
                    12000,
                ),
                "rationale": _bounded_str(item["rationale"], "rationale", 1, 4096),
            }
        )
    return rows


def _validate_reference_card(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "reference card")
    _require_fields(item, REFERENCE_CARD_FIELDS, "reference card")
    return {
        "reference_id": _portable_id(item["reference_id"], "reference_id"),
        "mechanism_summary": _bounded_str(
            item["mechanism_summary"], "mechanism_summary", 1, 12000
        ),
        "reference_sha256": _digest(item["reference_sha256"], "reference_sha256"),
    }


def _mapping(value: Any, label: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise ValidationError(f"{label} must be an object")
    return value


def _require_fields(value: Mapping[str, Any], expected: frozenset[str], label: str) -> None:
    fields = frozenset(value)
    if fields != expected:
        missing = sorted(expected - fields)
        extra = sorted(fields - expected)
        detail = []
        if missing:
            detail.append("missing " + ", ".join(missing))
        if extra:
            detail.append("unknown " + ", ".join(extra))
        raise ValidationError(f"{label} fields are invalid: {'; '.join(detail)}")


def _const(value: Any, expected: str, label: str) -> str:
    if value != expected:
        raise ValidationError(f"{label} must be {expected}")
    return expected


def _enum(value: Any, allowed: frozenset[str], label: str) -> str:
    if not isinstance(value, str) or value not in allowed:
        raise ValidationError(f"{label} must be one of {sorted(allowed)}")
    return value


def _portable_id(value: Any, label: str) -> str:
    text = _bounded_str(value, label, 1, 128)
    if _PORTABLE_ID.fullmatch(text) is None:
        raise ValidationError(f"{label} must be a portable lowercase ID")
    return text


def _runtime_id(value: Any, label: str) -> str:
    text = _bounded_str(value, label, 1, 128)
    if (
        _RUNTIME_ID.fullmatch(text) is None
        or text in {".", ".."}
        or text.startswith("-")
        or "/" in text
        or "\\" in text
    ):
        raise ValidationError(f"{label} must be a safe runtime ID")
    return text


def _namespaced_finding_id(value: Any, label: str) -> str:
    text = _bounded_str(value, label, 3, 260)
    prefix, sep, rest = text.partition(":")
    if sep != ":" or prefix not in {"r1", "r2"}:
        raise ValidationError(f"{label} must use r1: or r2: namespace")
    _portable_id(rest, label)
    return text


def _digest(value: Any, label: str) -> str:
    if not isinstance(value, str) or _SHA256.fullmatch(value) is None:
        raise ValidationError(f"{label} must be a lowercase SHA-256")
    return value


def _bounded_str(value: Any, label: str, minimum: int, maximum: int) -> str:
    if (
        not isinstance(value, str)
        or "\0" in value
        or len(value) < minimum
        or len(value) > maximum
    ):
        raise ValidationError(
            f"{label} must be a non-NUL string of length {minimum}..{maximum}"
        )
    return value


def _bounded_int(value: Any, label: str, minimum: int, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool):
        raise ValidationError(f"{label} must be an integer")
    if value < minimum or value > maximum:
        raise ValidationError(f"{label} must be in {minimum}..{maximum}")
    return value


def _reviewer_index(value: Any) -> int:
    return _bounded_int(value, "reviewer_index", 1, 2)


def _string_list(value: Any, label: str, minimum: int, maximum: int) -> list[str]:
    return [
        _bounded_str(item, f"{label} item", 1, 4096)
        for item in _list(value, label, minimum, maximum)
    ]


def _list(value: Any, label: str, minimum: int, maximum: int) -> list[Any]:
    if not isinstance(value, list) or len(value) < minimum or len(value) > maximum:
        raise ValidationError(f"{label} must be a list with {minimum}..{maximum} entries")
    return list(value)


def _sha256_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def _canonical_sha256(value: object) -> str:
    return hashlib.sha256(canonical_json_bytes(value)).hexdigest()
