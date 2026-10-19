from __future__ import annotations

import hashlib
import json
import re
from decimal import Context, Decimal, ROUND_HALF_EVEN, localcontext
from typing import Any, Iterable, Mapping, Sequence

import protocol
import expert_contracts


DECIMAL_CONTEXT = Context(
    prec=80,
    rounding=ROUND_HALF_EVEN,
    Emin=-999999,
    Emax=999999,
    capitals=1,
    clamp=0,
)

SELECTOR_DOMAIN = b"crouzeix-frontier/v1\0"
TWO_TO_256 = 1 << 256
PROBE_IDS = expert_contracts.PROOF_PROGRESS_PROBE_IDS

SHA256 = re.compile(r"^[0-9a-f]{64}$")
PORTABLE_ID = re.compile(r"^[a-z][a-z0-9]*(?:-[a-z0-9]+)*$")

NODE_FIELDS = frozenset(
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
        "source_attempt_id",
        "source_ticket_id",
        "source_expert_result_sha256",
        "source_context_sha256",
        "source_call_receipt_sha256",
        "admission_decision_sha256",
        "mathematical_payload",
    }
)

PAYLOAD_FIELDS = frozenset(
    {
        "proof_family",
        "mechanism",
        "proved_statements",
        "unproved_obligations",
        "circularity_risks",
        "proposed_directions",
        "endpoint",
        "confidence_basis",
    }
)

DIRECTION_FIELDS = frozenset(
    {
        "direction_id",
        "kind",
        "statement",
        "strength",
        "recommended_role",
        "source_node_artifact_sha256",
        "source_reconciliation_sha256",
    }
)

EVALUATION_FIELDS = frozenset(
    {
        "schema_version",
        "evaluation_id",
        "evaluator_index",
        "ticket_id",
        "node_artifact_sha256",
        "mathematical_payload_sha256",
        "context_sha256",
        "source_kind",
        "call_receipt_sha256",
        "terminal_ticket_event_sha256",
        "evaluation_payload",
        "evaluation_payload_sha256",
        "node_evaluation_sha256",
    }
)

EVALUATION_PAYLOAD_FIELDS = frozenset({"probes", "findings"})
PROBE_FIELDS = frozenset({"probe_id", "status"})
RECONCILED_PROBE_FIELDS = frozenset(
    {"probe_id", "evaluator_1_status", "evaluator_2_status", "unanimous_pass"}
)
FINDING_FIELDS = frozenset(
    {"finding_id", "severity", "statement", "locator", "recommended_role"}
)
RECONCILIATION_FIELDS = frozenset(
    {
        "schema_version",
        "reconciliation_id",
        "node_artifact_sha256",
        "mathematical_payload_sha256",
        "node_evaluation_sha256s",
        "probes",
        "unanimous_pass_count",
        "disagreement_count",
        "evaluator_findings",
        "reconciliation_sha256",
    }
)
ARCHIVE_ENTRY_FIELDS = frozenset(
    {
        "schema_version",
        "node_id",
        "node_artifact_sha256",
        "mathematical_payload_sha256",
        "reconciliation_sha256",
        "unanimous_pass_count",
        "candidate_proof_sha256",
        "archive_entry_sha256",
    }
)

ROLES = frozenset(
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
PROBE_STATUSES = frozenset({"pass", "fail", "insufficient_evidence"})
FINDING_SEVERITIES = frozenset({"critical", "major", "minor"})
SOURCE_KINDS = frozenset({"provider_output", "conservative_fallback"})

OBLIGATION_PRIORITY = {
    "theorem_strength": 0,
    "major": 2,
    "local": 4,
}
FINDING_PRIORITY = {
    "critical": 1,
    "major": 3,
}
PROPOSED_PRIORITY = 5


def build_mathematical_node(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, NODE_FIELDS, "mathematical node")
    _require_equal(value["schema_version"], "crouzeix-mathematical-node/v1", "schema_version")
    _portable_id(value["run_id"], "run_id")
    _portable_id(value["node_id"], "node_id")
    _digest(value["theorem_sha256"], "theorem_sha256")
    _enum(value["leakage_tier"], LEAKAGE_TIERS, "leakage_tier")
    generation = _integer(value["generation"], "generation", minimum=0, maximum=1_000_000)
    _enum(value["expert_role"], ROLES, "expert_role")
    _bounded_string(value["source_attempt_id"], "source_attempt_id", 1, 128)
    _bounded_string(value["source_ticket_id"], "source_ticket_id", 1, 128)
    _digest(value["source_expert_result_sha256"], "source_expert_result_sha256")
    _digest(value["source_context_sha256"], "source_context_sha256")
    _digest(value["source_call_receipt_sha256"], "source_call_receipt_sha256")
    _digest(value["admission_decision_sha256"], "admission_decision_sha256")

    parent_node_id = _optional_portable_id(value["parent_node_id"], "parent_node_id")
    parent_digest = _optional_digest(
        value["parent_node_artifact_sha256"], "parent_node_artifact_sha256"
    )
    if generation == 0:
        if parent_node_id is not None or parent_digest is not None:
            raise protocol.ValidationError("root node must not bind parent identity fields")
    elif parent_node_id is None or parent_digest is None:
        raise protocol.ValidationError("child node must bind both parent identity fields")

    selected_direction = _validate_direction(
        _mapping(value["selected_direction"], "selected_direction"),
        generation=generation,
        parent_digest=parent_digest,
    )
    payload = _validate_payload(_mapping(value["mathematical_payload"], "mathematical_payload"))

    node = dict(value)
    node["selected_direction"] = selected_direction
    node["selected_direction_sha256"] = canonical_sha256(selected_direction)
    node["mathematical_payload"] = payload
    node["mathematical_payload_sha256"] = canonical_sha256(payload)
    node["node_artifact_sha256"] = canonical_sha256(node)
    return node


def reconcile_node_evaluations(
    reconciliation_id: str,
    first: Mapping[str, Any],
    second: Mapping[str, Any],
) -> dict[str, object]:
    _bounded_string(reconciliation_id, "reconciliation_id", 1, 128)
    evaluations = [_validate_evaluation(first), _validate_evaluation(second)]
    evaluations.sort(key=lambda item: int(item["evaluator_index"]))
    if [evaluation["evaluator_index"] for evaluation in evaluations] != [1, 2]:
        raise protocol.ValidationError("evaluations must have evaluator indices one and two")

    node_artifact_sha256 = evaluations[0]["node_artifact_sha256"]
    payload_sha256 = evaluations[0]["mathematical_payload_sha256"]
    if evaluations[1]["node_artifact_sha256"] != node_artifact_sha256:
        raise protocol.ValidationError("evaluations must bind the same node_artifact_sha256")
    if evaluations[1]["mathematical_payload_sha256"] != payload_sha256:
        raise protocol.ValidationError("evaluations must bind the same mathematical_payload_sha256")

    distinct_fields = (
        "ticket_id",
        "context_sha256",
        "call_receipt_sha256",
        "evaluation_payload_sha256",
        "node_evaluation_sha256",
    )
    for field in distinct_fields:
        if evaluations[0][field] == evaluations[1][field]:
            raise protocol.ValidationError(f"evaluation {field} values must be distinct")

    probes = []
    unanimous_pass_count = 0
    disagreement_count = 0
    for probe_id in PROBE_IDS:
        left = _probe_status(evaluations[0], probe_id)
        right = _probe_status(evaluations[1], probe_id)
        unanimous_pass = left == "pass" and right == "pass"
        if unanimous_pass:
            unanimous_pass_count += 1
        if left != right:
            disagreement_count += 1
        probes.append(
            {
                "probe_id": probe_id,
                "evaluator_1_status": left,
                "evaluator_2_status": right,
                "unanimous_pass": unanimous_pass,
            }
        )

    findings = []
    for evaluation in evaluations:
        evaluator_index = int(evaluation["evaluator_index"])
        for finding in evaluation["evaluation_payload"]["findings"]:
            source_id = str(finding["finding_id"])
            namespaced = dict(finding)
            namespaced["source_finding_id"] = source_id
            namespaced["source_evaluator_index"] = evaluator_index
            namespaced["finding_id"] = f"e{evaluator_index}:{source_id}"
            findings.append(namespaced)
    findings.sort(key=lambda item: (item["source_evaluator_index"], item["source_finding_id"]))

    reconciliation = {
        "schema_version": "crouzeix-reconciliation/v1",
        "reconciliation_id": reconciliation_id,
        "node_artifact_sha256": node_artifact_sha256,
        "mathematical_payload_sha256": payload_sha256,
        "node_evaluation_sha256s": [
            evaluation["node_evaluation_sha256"] for evaluation in evaluations
        ],
        "probes": probes,
        "unanimous_pass_count": unanimous_pass_count,
        "disagreement_count": disagreement_count,
        "evaluator_findings": findings,
    }
    reconciliation["reconciliation_sha256"] = canonical_sha256(reconciliation)
    return reconciliation


def build_archive_entry(
    mathematical_node: Mapping[str, Any],
    reconciliation: Mapping[str, Any],
) -> dict[str, object]:
    reconciliation = _validate_reconciliation(reconciliation)
    node_artifact_sha256 = _digest(
        mathematical_node.get("node_artifact_sha256"), "node.node_artifact_sha256"
    )
    payload_sha256 = _digest(
        mathematical_node.get("mathematical_payload_sha256"),
        "node.mathematical_payload_sha256",
    )
    if reconciliation.get("node_artifact_sha256") != node_artifact_sha256:
        raise protocol.ValidationError("reconciliation node_artifact_sha256 must match node")
    if reconciliation.get("mathematical_payload_sha256") != payload_sha256:
        raise protocol.ValidationError("reconciliation mathematical_payload_sha256 must match node")
    _digest(reconciliation.get("reconciliation_sha256"), "reconciliation_sha256")
    pass_count = _pass_count(reconciliation.get("unanimous_pass_count"))

    endpoint = _mapping(
        _mapping(mathematical_node.get("mathematical_payload"), "mathematical_payload").get(
            "endpoint"
        ),
        "mathematical_payload.endpoint",
    )
    kind = _enum(endpoint.get("kind"), ENDPOINT_KINDS, "endpoint.kind")
    text = _bounded_string(endpoint.get("text"), "endpoint.text", 1, 1_000_000)
    candidate_sha256 = hashlib.sha256(text.encode("utf-8")).hexdigest() if kind == "candidate_proof" else None

    entry = {
        "schema_version": "crouzeix-archive-entry/v1",
        "node_id": _portable_id(mathematical_node.get("node_id"), "node_id"),
        "node_artifact_sha256": node_artifact_sha256,
        "mathematical_payload_sha256": payload_sha256,
        "reconciliation_sha256": reconciliation["reconciliation_sha256"],
        "unanimous_pass_count": pass_count,
        "candidate_proof_sha256": candidate_sha256,
    }
    entry["archive_entry_sha256"] = canonical_sha256(entry)
    return entry


def keep_all_archive(entries: Iterable[Mapping[str, Any]]) -> list[dict[str, object]]:
    archive = [_validate_archive_entry(entry) for entry in entries]
    _reject_duplicate_values(archive, "archive_entry_sha256", "archive entries")
    return sorted(archive, key=lambda entry: str(entry["node_id"]))


def select_generation(
    archive_entries: Sequence[Mapping[str, Any]],
    nodes: Sequence[Mapping[str, Any]],
    *,
    seed: int,
    generation: int,
) -> dict[str, object]:
    seed = _integer(seed, "seed", minimum=0, maximum=10**30)
    generation = _integer(generation, "generation", minimum=1, maximum=1_000_000)
    entries = keep_all_archive(archive_entries)
    node_by_id = {_portable_id(node.get("node_id"), "node.node_id"): dict(node) for node in nodes}
    if len(node_by_id) != len(nodes):
        raise protocol.ValidationError("node IDs must be distinct")

    for entry in entries:
        _pass_count(entry["unanimous_pass_count"])
        node = node_by_id.get(entry["node_id"])
        if node is None:
            raise protocol.ValidationError("archive entry must have matching node")
        if node.get("node_artifact_sha256") != entry["node_artifact_sha256"]:
            raise protocol.ValidationError("archive entry node_artifact_sha256 must match node")

    child_counts = _child_counts(entries, node_by_id)
    eligible_entries = [
        entry for entry in entries if int(entry["unanimous_pass_count"]) < 10
    ]
    if not eligible_entries:
        raise protocol.ValidationError("eligible archive is empty")

    terms = _selection_terms(eligible_entries, child_counts)
    snapshot = {
        "schema_version": "crouzeix-frontier-snapshot/v1",
        "generation": generation,
        "eligible_node_ids": [entry["node_id"] for entry in eligible_entries],
        "eligible_archive_entry_sha256s": [
            entry["archive_entry_sha256"] for entry in eligible_entries
        ],
        "unanimous_pass_counts_by_node_id": {
            entry["node_id"]: entry["unanimous_pass_count"] for entry in eligible_entries
        },
        "child_counts_by_node_id": {
            entry["node_id"]: child_counts[entry["node_id"]] for entry in eligible_entries
        },
        "terms": terms,
    }
    snapshot["archive_snapshot_sha256"] = canonical_sha256(snapshot)

    selection_events = [
        _selection_event(seed, generation, draw_index, snapshot, terms)
        for draw_index in (0, 1)
    ]
    return {"snapshot": snapshot, "selection_events": selection_events}


def select_frontier_direction(
    mathematical_node: Mapping[str, Any],
    reconciliation: Mapping[str, Any],
    *,
    used_direction_ids: Iterable[str] = (),
) -> dict[str, object]:
    candidates = _direction_candidates(mathematical_node, reconciliation)
    used = set(used_direction_ids)
    for _, direction in candidates:
        if direction["direction_id"] not in used:
            return direction
    if not candidates:
        raise protocol.ValidationError("node has no frontier directions")
    return candidates[0][1]


def assign_directions_to_selection_events(
    selection_events: Sequence[Mapping[str, Any]],
    nodes_by_id: Mapping[str, Mapping[str, Any]],
    reconciliations_by_node_artifact_sha256: Mapping[str, Mapping[str, Any]],
) -> list[dict[str, object]]:
    used_by_node: dict[str, set[str]] = {}
    assignments = []
    for event in selection_events:
        event_copy = dict(event)
        node_id = _portable_id(event_copy.get("selected_node_id"), "selected_node_id")
        node = nodes_by_id.get(node_id)
        if node is None:
            raise protocol.ValidationError("selection event references unknown node")
        node_digest = _digest(node.get("node_artifact_sha256"), "node_artifact_sha256")
        reconciliation = reconciliations_by_node_artifact_sha256.get(node_digest)
        if reconciliation is None:
            raise protocol.ValidationError("selected node is missing reconciliation")
        used = used_by_node.setdefault(node_id, set())
        direction = select_frontier_direction(node, reconciliation, used_direction_ids=used)
        used.add(str(direction["direction_id"]))
        event_copy["selected_direction"] = direction
        assignments.append(event_copy)
    return assignments


def canonical_sha256(value: Mapping[str, Any]) -> str:
    return hashlib.sha256(_canonical_json_bytes(value)).hexdigest()


def _canonical_json_bytes(value: Any) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")


def _validate_direction(
    value: Mapping[str, Any],
    *,
    generation: int,
    parent_digest: str | None,
) -> dict[str, object]:
    _require_fields(value, DIRECTION_FIELDS, "selected_direction")
    direction = dict(value)
    _portable_id(direction["direction_id"], "selected_direction.direction_id")
    kind = _enum(direction["kind"], DIRECTION_KINDS, "selected_direction.kind")
    _bounded_string(direction["statement"], "selected_direction.statement", 1, 20_000)
    _enum(direction["strength"], DIRECTION_STRENGTHS, "selected_direction.strength")
    direction["recommended_role"] = _role(direction["recommended_role"], "selected_direction.recommended_role")
    source_node_digest = _optional_digest(
        direction["source_node_artifact_sha256"],
        "selected_direction.source_node_artifact_sha256",
    )
    source_reconciliation_digest = _optional_digest(
        direction["source_reconciliation_sha256"],
        "selected_direction.source_reconciliation_sha256",
    )
    if generation == 0:
        if kind != "root_task" or source_node_digest is not None or source_reconciliation_digest is not None:
            raise protocol.ValidationError("root selected_direction must be a root_task without source digests")
    else:
        if source_node_digest != parent_digest:
            raise protocol.ValidationError("child selected_direction must bind parent node artifact")
        if kind == "evaluator_finding" and source_reconciliation_digest is None:
            raise protocol.ValidationError("evaluator_finding direction must bind reconciliation digest")
    return direction


def _validate_payload(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, PAYLOAD_FIELDS, "mathematical_payload")
    payload = dict(value)
    for field in ("proof_family", "mechanism", "confidence_basis"):
        _bounded_string(payload[field], f"mathematical_payload.{field}", 1, 20_000)
    for field in ("proved_statements", "unproved_obligations", "circularity_risks", "proposed_directions"):
        if not isinstance(payload[field], list):
            raise protocol.ValidationError(f"mathematical_payload.{field} must be a list")
    endpoint = _mapping(payload["endpoint"], "mathematical_payload.endpoint")
    if set(endpoint) != {"kind", "text"}:
        raise protocol.ValidationError("mathematical_payload.endpoint fields are invalid")
    _enum(endpoint["kind"], ENDPOINT_KINDS, "mathematical_payload.endpoint.kind")
    _bounded_string(endpoint["text"], "mathematical_payload.endpoint.text", 1, 1_000_000)
    payload["endpoint"] = dict(endpoint)
    return payload


def _validate_evaluation(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, EVALUATION_FIELDS, "node evaluation")
    result = dict(value)
    _require_equal(result["schema_version"], "crouzeix-node-evaluation/v1", "schema_version")
    _bounded_string(result["evaluation_id"], "evaluation_id", 1, 128)
    _integer(result["evaluator_index"], "evaluator_index", minimum=1, maximum=2)
    _bounded_string(result["ticket_id"], "ticket_id", 1, 128)
    _digest(result["node_artifact_sha256"], "node_artifact_sha256")
    _digest(result["mathematical_payload_sha256"], "mathematical_payload_sha256")
    _digest(result["context_sha256"], "context_sha256")
    source_kind = _enum(result["source_kind"], SOURCE_KINDS, "source_kind")
    if source_kind == "provider_output":
        _digest(result["call_receipt_sha256"], "call_receipt_sha256")
    elif result["call_receipt_sha256"] is not None:
        _digest(result["call_receipt_sha256"], "call_receipt_sha256")
    _digest(result["terminal_ticket_event_sha256"], "terminal_ticket_event_sha256")
    payload = _mapping(result["evaluation_payload"], "evaluation_payload")
    _require_fields(payload, EVALUATION_PAYLOAD_FIELDS, "evaluation_payload")
    probes = payload["probes"]
    if not isinstance(probes, list) or len(probes) != 10:
        raise protocol.ValidationError("node evaluation must contain exactly ten probe IDs")
    seen = set()
    for probe in probes:
        probe_map = _mapping(probe, "probe")
        _require_fields(probe_map, PROBE_FIELDS, "probe")
        probe_id = _bounded_string(probe_map["probe_id"], "probe_id", 1, 128)
        if probe_id not in PROBE_IDS or probe_id in seen:
            raise protocol.ValidationError("node evaluation must contain exactly ten probe IDs")
        seen.add(probe_id)
        _enum(probe_map["status"], PROBE_STATUSES, "probe.status")
    if tuple(probe["probe_id"] for probe in probes) != PROBE_IDS:
        raise protocol.ValidationError("node evaluation probe order must match closed probe IDs")
    if set(seen) != set(PROBE_IDS):
        raise protocol.ValidationError("node evaluation must contain exactly ten probe IDs")

    findings = payload["findings"]
    if not isinstance(findings, list):
        raise protocol.ValidationError("evaluation_payload.findings must be a list")
    normalized_findings = []
    finding_ids = set()
    for finding in findings:
        finding_map = _mapping(finding, "finding")
        _require_fields(finding_map, FINDING_FIELDS, "finding")
        finding_id = _portable_id(finding_map["finding_id"], "finding_id")
        if finding_id in finding_ids:
            raise protocol.ValidationError("finding IDs must be distinct per evaluator")
        finding_ids.add(finding_id)
        normalized_findings.append(
            {
                "finding_id": finding_id,
                "severity": _enum(finding_map["severity"], FINDING_SEVERITIES, "severity"),
                "statement": _bounded_string(finding_map["statement"], "statement", 1, 20_000),
                "locator": _bounded_string(finding_map["locator"], "locator", 1, 1_000),
                "recommended_role": _role(finding_map["recommended_role"], "finding.recommended_role"),
            }
        )

    normalized_payload = {
        "probes": [
            {"probe_id": probe["probe_id"], "status": probe["status"]} for probe in probes
        ],
        "findings": normalized_findings,
    }
    if result["evaluation_payload_sha256"] != canonical_sha256(normalized_payload):
        raise protocol.ValidationError("evaluation_payload_sha256 does not match payload")
    without_artifact = dict(result)
    without_artifact["evaluation_payload"] = normalized_payload
    without_artifact.pop("node_evaluation_sha256")
    if result["node_evaluation_sha256"] != canonical_sha256(without_artifact):
        raise protocol.ValidationError("node_evaluation_sha256 does not match evaluation")
    result["evaluation_payload"] = normalized_payload
    return result


def _probe_status(evaluation: Mapping[str, Any], probe_id: str) -> str:
    for probe in evaluation["evaluation_payload"]["probes"]:
        if probe["probe_id"] == probe_id:
            return str(probe["status"])
    raise protocol.ValidationError("node evaluation must contain exactly ten probe IDs")


def _validate_archive_entry(value: Mapping[str, Any]) -> dict[str, object]:
    entry = dict(value)
    _require_fields(entry, ARCHIVE_ENTRY_FIELDS, "archive entry")
    _require_equal(entry.get("schema_version"), "crouzeix-archive-entry/v1", "schema_version")
    _portable_id(entry.get("node_id"), "node_id")
    _digest(entry.get("node_artifact_sha256"), "node_artifact_sha256")
    _digest(entry.get("mathematical_payload_sha256"), "mathematical_payload_sha256")
    _digest(entry.get("reconciliation_sha256"), "reconciliation_sha256")
    _pass_count(entry.get("unanimous_pass_count"))
    if entry.get("candidate_proof_sha256") is not None:
        _digest(entry.get("candidate_proof_sha256"), "candidate_proof_sha256")
    _digest(entry.get("archive_entry_sha256"), "archive_entry_sha256")
    expected = canonical_sha256(
        {key: value for key, value in entry.items() if key != "archive_entry_sha256"}
    )
    if entry["archive_entry_sha256"] != expected:
        raise protocol.ValidationError("archive_entry_sha256 does not match archive entry")
    return entry


def _validate_reconciliation(value: Mapping[str, Any]) -> dict[str, object]:
    reconciliation = dict(value)
    _require_fields(reconciliation, RECONCILIATION_FIELDS, "reconciliation")
    _require_equal(
        reconciliation.get("schema_version"),
        "crouzeix-reconciliation/v1",
        "schema_version",
    )
    _bounded_string(reconciliation.get("reconciliation_id"), "reconciliation_id", 1, 128)
    _digest(reconciliation.get("node_artifact_sha256"), "node_artifact_sha256")
    _digest(reconciliation.get("mathematical_payload_sha256"), "mathematical_payload_sha256")
    evaluation_digests = reconciliation.get("node_evaluation_sha256s")
    if not isinstance(evaluation_digests, list) or len(evaluation_digests) != 2:
        raise protocol.ValidationError("reconciliation must bind two node evaluation digests")
    for digest in evaluation_digests:
        _digest(digest, "node_evaluation_sha256s")
    _pass_count(reconciliation.get("unanimous_pass_count"))
    _integer(
        reconciliation.get("disagreement_count"),
        "disagreement_count",
        minimum=0,
        maximum=10,
    )
    probes = reconciliation.get("probes")
    if not isinstance(probes, list) or len(probes) != 10:
        raise protocol.ValidationError("reconciliation must contain ten probes")
    expected_pass_count = 0
    expected_disagreement_count = 0
    seen_probe_ids = set()
    for probe in probes:
        probe_map = _mapping(probe, "reconciliation.probe")
        _require_fields(probe_map, RECONCILED_PROBE_FIELDS, "reconciliation probe")
        probe_id = _bounded_string(probe_map["probe_id"], "probe_id", 1, 128)
        if probe_id not in PROBE_IDS or probe_id in seen_probe_ids:
            raise protocol.ValidationError("reconciliation must contain ten probe IDs")
        seen_probe_ids.add(probe_id)
        evaluator_1_status = _enum(
            probe_map["evaluator_1_status"],
            PROBE_STATUSES,
            "probe.evaluator_1_status",
        )
        evaluator_2_status = _enum(
            probe_map["evaluator_2_status"],
            PROBE_STATUSES,
            "probe.evaluator_2_status",
        )
        expected_unanimous_pass = (
            evaluator_1_status == "pass" and evaluator_2_status == "pass"
        )
        if not isinstance(probe_map["unanimous_pass"], bool):
            raise protocol.ValidationError("probe.unanimous_pass must be boolean")
        if probe_map["unanimous_pass"] != expected_unanimous_pass:
            raise protocol.ValidationError("probe unanimous_pass does not match evaluator statuses")
        if expected_unanimous_pass:
            expected_pass_count += 1
        if evaluator_1_status != evaluator_2_status:
            expected_disagreement_count += 1
    if tuple(probe["probe_id"] for probe in probes) != PROBE_IDS:
        raise protocol.ValidationError("reconciliation probe order must match closed probe IDs")
    if set(seen_probe_ids) != set(PROBE_IDS):
        raise protocol.ValidationError("reconciliation must contain ten probe IDs")
    if reconciliation["unanimous_pass_count"] != expected_pass_count:
        raise protocol.ValidationError("unanimous_pass_count does not match probe rows")
    if reconciliation["disagreement_count"] != expected_disagreement_count:
        raise protocol.ValidationError("disagreement_count does not match probe rows")
    if not isinstance(reconciliation.get("evaluator_findings"), list):
        raise protocol.ValidationError("reconciliation evaluator_findings must be a list")
    _digest(reconciliation.get("reconciliation_sha256"), "reconciliation_sha256")
    expected = canonical_sha256(
        {
            key: item
            for key, item in reconciliation.items()
            if key != "reconciliation_sha256"
        }
    )
    if reconciliation["reconciliation_sha256"] != expected:
        raise protocol.ValidationError("reconciliation_sha256 does not match reconciliation")
    return reconciliation


def _child_counts(
    archive_entries: Sequence[Mapping[str, Any]],
    node_by_id: Mapping[str, Mapping[str, Any]],
) -> dict[str, int]:
    artifact_to_node_id = {
        str(entry["node_artifact_sha256"]): str(entry["node_id"]) for entry in archive_entries
    }
    counts = {str(entry["node_id"]): 0 for entry in archive_entries}
    archived_node_ids = set(counts)
    for node_id in archived_node_ids:
        node = node_by_id[node_id]
        parent_digest = node.get("parent_node_artifact_sha256")
        if parent_digest is None:
            continue
        parent_node_id = artifact_to_node_id.get(str(parent_digest))
        if parent_node_id is not None:
            counts[parent_node_id] += 1
    return counts


def _selection_terms(
    eligible_entries: Sequence[Mapping[str, Any]],
    child_counts: Mapping[str, int],
) -> list[dict[str, object]]:
    with localcontext(DECIMAL_CONTEXT) as context:
        raw_terms = []
        total = Decimal(0)
        for entry in eligible_entries:
            node_id = str(entry["node_id"])
            pass_count = _pass_count(entry["unanimous_pass_count"])
            alpha = context.divide(Decimal(pass_count), Decimal(10))
            exponent = context.multiply(Decimal(-10), context.subtract(alpha, Decimal("0.5")))
            score = context.divide(Decimal(1), context.add(Decimal(1), context.exp(exponent)))
            underexploration = context.divide(
                Decimal(1), context.add(Decimal(1), Decimal(child_counts[node_id]))
            )
            weight = context.multiply(score, underexploration)
            if not weight.is_finite() or weight <= 0:
                raise protocol.ValidationError("DGM weight must be finite and positive")
            raw_terms.append(
                {
                    "node_id": node_id,
                    "archive_entry_sha256": entry["archive_entry_sha256"],
                    "unanimous_pass_count": pass_count,
                    "functioning_child_count": child_counts[node_id],
                    "alpha": alpha,
                    "score": score,
                    "underexploration": underexploration,
                    "weight": weight,
                }
            )
            total = context.add(total, weight)
        if not total.is_finite() or total <= 0:
            raise protocol.ValidationError("DGM total weight must be finite and positive")

        cumulative = Decimal(0)
        terms = []
        for index, term in enumerate(raw_terms):
            probability = context.divide(term["weight"], total)
            interval_start = cumulative
            interval_end = Decimal(1) if index == len(raw_terms) - 1 else context.add(cumulative, probability)
            cumulative = interval_end
            terms.append(
                {
                    "node_id": term["node_id"],
                    "archive_entry_sha256": term["archive_entry_sha256"],
                    "unanimous_pass_count": term["unanimous_pass_count"],
                    "functioning_child_count": term["functioning_child_count"],
                    "alpha_decimal": _decimal_string(term["alpha"]),
                    "score_decimal": _decimal_string(term["score"]),
                    "underexploration_decimal": _decimal_string(term["underexploration"]),
                    "weight_decimal": _decimal_string(term["weight"]),
                    "probability_decimal": _decimal_string(probability),
                    "interval_start_decimal": _decimal_string(interval_start),
                    "interval_end_decimal": _decimal_string(interval_end),
                }
            )
    return terms


def _selection_event(
    seed: int,
    generation: int,
    draw_index: int,
    snapshot: Mapping[str, Any],
    terms: Sequence[Mapping[str, Any]],
) -> dict[str, object]:
    preimage = _draw_preimage(seed, generation, draw_index)
    digest = hashlib.sha256(preimage).hexdigest()
    unsigned = int.from_bytes(bytes.fromhex(digest), "big")
    with localcontext(DECIMAL_CONTEXT) as context:
        uniform = context.divide(Decimal(unsigned), Decimal(TWO_TO_256))
    selected = terms[-1]
    for term in terms:
        start = Decimal(str(term["interval_start_decimal"]))
        end = Decimal(str(term["interval_end_decimal"]))
        if uniform >= start and uniform < end:
            selected = term
            break
    event = {
        "schema_version": "crouzeix-selection-event/v1",
        "generation": generation,
        "draw_index": draw_index,
        "seed": seed,
        "archive_snapshot_sha256": snapshot["archive_snapshot_sha256"],
        "eligible_node_ids": list(snapshot["eligible_node_ids"]),
        "eligible_archive_entry_sha256s": list(snapshot["eligible_archive_entry_sha256s"]),
        "child_counts_by_node_id": dict(snapshot["child_counts_by_node_id"]),
        "probabilities_by_node_id": {
            term["node_id"]: term["probability_decimal"] for term in terms
        },
        "draw_preimage": preimage.decode("ascii"),
        "draw_preimage_hex": preimage.hex(),
        "draw_sha256": digest,
        "draw_unsigned_integer": unsigned,
        "uniform_decimal": _decimal_string(uniform),
        "terms": [dict(term) for term in terms],
        "selected_node_id": selected["node_id"],
        "selected_archive_entry_sha256": selected["archive_entry_sha256"],
        "selected_interval_start_decimal": selected["interval_start_decimal"],
        "selected_interval_end_decimal": selected["interval_end_decimal"],
    }
    event["selection_event_sha256"] = canonical_sha256(event)
    return event


def _draw_preimage(seed: int, generation: int, draw_index: int) -> bytes:
    if draw_index not in (0, 1):
        raise protocol.ValidationError("draw_index must be zero or one")
    parts = (
        _ascii_decimal(seed, "seed"),
        _ascii_decimal(generation, "generation"),
        _ascii_decimal(draw_index, "draw_index"),
    )
    return SELECTOR_DOMAIN + b"\0".join(parts)


def _ascii_decimal(value: int, field: str) -> bytes:
    value = _integer(value, field, minimum=0, maximum=10**30)
    return str(value).encode("ascii")


def _direction_candidates(
    mathematical_node: Mapping[str, Any],
    reconciliation: Mapping[str, Any],
) -> list[tuple[tuple[int, str], dict[str, object]]]:
    node_digest = _digest(mathematical_node.get("node_artifact_sha256"), "node_artifact_sha256")
    reconciliation_digest = _digest(
        reconciliation.get("reconciliation_sha256"), "reconciliation_sha256"
    )
    if reconciliation.get("node_artifact_sha256") != node_digest:
        raise protocol.ValidationError("reconciliation node_artifact_sha256 must match node")
    payload = _mapping(mathematical_node.get("mathematical_payload"), "mathematical_payload")
    candidates = []

    for obligation in payload.get("unproved_obligations", []):
        obligation_map = _mapping(obligation, "obligation")
        obligation_id = _bounded_string(
            obligation_map.get("obligation_id", obligation_map.get("statement_id", "")),
            "obligation_id",
            1,
            128,
        )
        strength = _bounded_string(obligation_map.get("strength"), "obligation.strength", 1, 128)
        priority = OBLIGATION_PRIORITY.get(strength)
        if priority is None:
            continue
        direction = {
            "direction_id": f"obligation-{obligation_id}",
            "kind": "obligation",
            "statement": _bounded_string(obligation_map.get("statement"), "obligation.statement", 1, 20_000),
            "strength": strength,
            "recommended_role": _role(
                obligation_map.get("recommended_role"), "obligation.recommended_role"
            ),
            "source_node_artifact_sha256": node_digest,
            "source_reconciliation_sha256": None,
        }
        candidates.append(((priority, direction["direction_id"]), direction))

    for finding in reconciliation.get("evaluator_findings", []):
        finding_map = _mapping(finding, "finding")
        severity = str(finding_map.get("severity"))
        priority = FINDING_PRIORITY.get(severity)
        if priority is None:
            continue
        finding_id = _bounded_string(finding_map.get("finding_id"), "finding_id", 1, 128)
        direction = {
            "direction_id": f"finding-{finding_id}",
            "kind": "evaluator_finding",
            "statement": _bounded_string(finding_map.get("statement"), "finding.statement", 1, 20_000),
            "strength": severity,
            "recommended_role": _role(
                finding_map.get("recommended_role"), "finding.recommended_role"
            ),
            "source_node_artifact_sha256": node_digest,
            "source_reconciliation_sha256": reconciliation_digest,
        }
        candidates.append(((priority, direction["direction_id"]), direction))

    for proposed in payload.get("proposed_directions", []):
        proposed_map = _mapping(proposed, "proposed_direction")
        proposed_id = _bounded_string(proposed_map.get("direction_id"), "direction_id", 1, 128)
        direction = {
            "direction_id": f"proposed-{proposed_id}",
            "kind": "proposed_direction",
            "statement": _bounded_string(proposed_map.get("statement"), "proposed.statement", 1, 20_000),
            "strength": "proposed",
            "recommended_role": _role(
                proposed_map.get("recommended_role"), "proposed_direction.recommended_role"
            ),
            "source_node_artifact_sha256": node_digest,
            "source_reconciliation_sha256": None,
        }
        candidates.append(((PROPOSED_PRIORITY, direction["direction_id"]), direction))

    candidates.sort(key=lambda item: item[0])
    return candidates


def _role(value: Any, field: str) -> str:
    return _enum(value, ROLES, field)


def _decimal_string(value: Decimal) -> str:
    with localcontext(DECIMAL_CONTEXT):
        normalized = value.normalize()
    if normalized == normalized.to_integral():
        return format(normalized, "f")
    return format(normalized, "f")


def _require_fields(value: Mapping[str, Any], fields: frozenset[str], label: str) -> None:
    observed = set(value.keys())
    if observed != fields:
        missing = sorted(fields - observed)
        extra = sorted(observed - fields)
        raise protocol.ValidationError(f"{label} fields invalid; missing={missing} extra={extra}")


def _require_equal(value: Any, expected: str, field: str) -> str:
    if value != expected:
        raise protocol.ValidationError(f"{field} must be {expected}")
    return expected


def _mapping(value: Any, field: str) -> Mapping[str, Any]:
    if not isinstance(value, Mapping):
        raise protocol.ValidationError(f"{field} must be an object")
    return value


def _bounded_string(value: Any, field: str, minimum: int, maximum: int) -> str:
    if not isinstance(value, str) or "\0" in value or not (minimum <= len(value) <= maximum):
        raise protocol.ValidationError(f"{field} must be a bounded non-NUL string")
    return value


def _integer(value: Any, field: str, *, minimum: int, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not (minimum <= value <= maximum):
        raise protocol.ValidationError(f"{field} must be an integer in [{minimum}, {maximum}]")
    return value


def _pass_count(value: Any) -> int:
    return _integer(value, "unanimous_pass_count", minimum=0, maximum=10)


def _digest(value: Any, field: str) -> str:
    if not isinstance(value, str) or SHA256.fullmatch(value) is None:
        raise protocol.ValidationError(f"{field} must be a SHA-256 digest")
    return value


def _optional_digest(value: Any, field: str) -> str | None:
    if value is None:
        return None
    return _digest(value, field)


def _portable_id(value: Any, field: str) -> str:
    if not isinstance(value, str) or PORTABLE_ID.fullmatch(value) is None:
        raise protocol.ValidationError(f"{field} must be a portable identifier")
    return value


def _optional_portable_id(value: Any, field: str) -> str | None:
    if value is None:
        return None
    return _portable_id(value, field)


def _enum(value: Any, allowed: Iterable[str], field: str) -> str:
    if not isinstance(value, str) or value not in allowed:
        raise protocol.ValidationError(f"{field} must be one of {sorted(allowed)}")
    return value


def _reject_duplicate_values(values: Sequence[Mapping[str, Any]], field: str, label: str) -> None:
    seen = set()
    for value in values:
        item = value[field]
        if item in seen:
            raise protocol.ValidationError(f"{label} must have distinct {field}")
        seen.add(item)
