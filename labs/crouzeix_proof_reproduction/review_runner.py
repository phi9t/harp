from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Mapping, Sequence

import review_contracts
import runner
import tickets
from protocol import ValidationError, read_run_spec, read_strict_json_object, sha256_bytes


REVIEW_ALLOWED_TOOLS = ["Write"]
RECONCILIATION_PROMPT_SHA256 = "0" * 64
RECONCILIATION_SCHEMA_SHA256 = "0" * 64
REVIEW_PROVIDER_ROLES = frozenset(
    {"correctness_review", "repair", "mechanism_classification"}
)
TASK_BY_ROLE = {
    "correctness_review": "correctness_review",
    "repair": "correctness_repair",
    "mechanism_classification": "mechanism_classification",
    "finding_reconciliation": "finding_reconciliation",
}
OWNER_BY_ROLE = {
    "correctness_review": "reviewer",
    "repair": "reviewer",
    "mechanism_classification": "reviewer",
    "finding_reconciliation": "orchestrator",
}


def run_correctness_review(
    review_dir: Path,
    *,
    ticket_id: str,
    reviewer_index: int,
    theorem_text: str,
    candidate_text: str,
    candidate_projection: Mapping[str, Any],
) -> dict[str, Any]:
    root = Path(review_dir).resolve()
    context = review_contracts.build_correctness_context(
        theorem_text=theorem_text,
        candidate_text=candidate_text,
        candidate_projection=candidate_projection,
        reviewer_index=reviewer_index,
        ticket_id=ticket_id,
    )
    _require_ticket_file(root, ticket_id)
    prompt = _read_prompt(root, "correctness_review.md")
    schema = root / "schemas/correctness_review.schema.json"
    ticket = _load_and_validate_ticket(
        root,
        ticket_id=ticket_id,
        role="correctness_review",
        context=context,
        prompt=prompt,
        schema_path=schema,
        parent_artifact_sha256=str(context["candidate_sha256"]),
    )
    result = _run_review_call(
        root,
        ticket=ticket,
        call_id=ticket_id,
        provider_role="correctness_review",
        prompt=prompt,
        context=context,
        schema_path=schema,
        validator=review_contracts.validate_correctness_review_payload,
    )
    review = None
    if result["validated_output"] is not None:
        receipt_sha256 = _canonical_sha256(result["receipt"])
        review = review_contracts.build_correctness_review_envelope(
            review_id=ticket_id,
            ticket_id=ticket_id,
            ticket_sha256=tickets.canonical_sha256(ticket),
            context_sha256=str(ticket["context_sha256"]),
            prompt_sha256=str(ticket["prompt_sha256"]),
            schema_sha256=str(ticket["schema_sha256"]),
            call_receipt_sha256=receipt_sha256,
            terminal_ticket_event_sha256=str(result["terminal_ticket_event_sha256"]),
            review_payload=result["validated_output"],
        )
    result["review"] = review
    return result


def run_repair(
    review_dir: Path,
    *,
    ticket_id: str,
    repair_id: str,
    parent_candidate_text: str,
    finding_ledger: Mapping[str, Any],
    theorem_text: str = "Theorem.",
) -> dict[str, Any]:
    root = Path(review_dir).resolve()
    request = review_contracts.build_repair_request(
        repair_id=repair_id,
        parent_candidate_text=parent_candidate_text,
        finding_ledger=finding_ledger,
        ticket_id=ticket_id,
    )
    _require_ticket_file(root, ticket_id)
    required = [
        f"review-{repair_id}-r1",
        f"review-{repair_id}-r2",
    ]
    prompt = _read_prompt(root, "review_repair.md")
    schema = root / "schemas/repair_result.schema.json"
    ticket = _load_and_validate_ticket(
        root,
        ticket_id=ticket_id,
        role="repair",
        context=request,
        prompt=prompt,
        schema_path=schema,
        parent_artifact_sha256=str(request["parent_candidate_sha256"]),
    )
    result = _load_existing_repair_call(root, ticket_id)
    if result is None:
        result = _run_review_call(
            root,
            ticket=ticket,
            call_id=ticket_id,
            provider_role="repair",
            prompt=prompt,
            context=request,
            schema_path=schema,
            validator=review_contracts.validate_repair_payload,
        )
    repair_result = None
    if result["validated_output"] is not None:
        repair_result = review_contracts.build_repair_result(
            repair_id=repair_id,
            parent_candidate_sha256=str(request["parent_candidate_sha256"]),
            finding_ledger_sha256=str(request["finding_ledger_sha256"]),
            repair_payload=result["validated_output"],
        )
        if repair_result["changed_candidate_bytes"]:
            marker_digests = _materialize_post_repair_context_markers(
                root,
                repair_id=repair_id,
                repaired_candidate_text=str(repair_result["repair_payload"]["candidate_text"]),
                theorem_text=theorem_text,
            )
            _require_re_review_tickets(
                root,
                required_ticket_ids=required,
                repaired_candidate_sha256=str(repair_result["repaired_candidate_sha256"]),
                repaired_candidate_text=str(repair_result["repair_payload"]["candidate_text"]),
                theorem_text=theorem_text,
                required_context_marker_sha256s=marker_digests,
            )
    result["repair_result"] = repair_result
    result["required_re_review_ticket_ids"] = required
    return result


def run_mechanism_classification(
    review_dir: Path,
    *,
    ticket_id: str,
    classification_id: str,
    theorem_text: str,
    candidate_text: str,
    frozen_correctness: Mapping[str, Any],
    reference_cards: Sequence[Mapping[str, Any]],
) -> dict[str, Any]:
    root = Path(review_dir).resolve()
    context = review_contracts.build_mechanism_classification_context(
        classification_id=classification_id,
        theorem_text=theorem_text,
        candidate_text=candidate_text,
        frozen_correctness=frozen_correctness,
        reference_cards=reference_cards,
        ticket_id=ticket_id,
    )
    _require_ticket_file(root, ticket_id)
    prompt = _read_prompt(root, "mechanism_classification.md")
    schema = root / "schemas/mechanism_classification.schema.json"
    ticket = _load_and_validate_ticket(
        root,
        ticket_id=ticket_id,
        role="mechanism_classification",
        context=context,
        prompt=prompt,
        schema_path=schema,
        parent_artifact_sha256=str(context["candidate_sha256"]),
    )
    result = _run_review_call(
        root,
        ticket=ticket,
        call_id=ticket_id,
        provider_role="mechanism_classification",
        prompt=prompt,
        context=context,
        schema_path=schema,
        validator=review_contracts.validate_mechanism_classification_payload,
    )
    result["classification"] = result["validated_output"]
    return result


def build_reconciliation_context(
    *,
    reconciliation_id: str,
    finding_ledgers: Sequence[Mapping[str, Any]],
) -> dict[str, object]:
    ledgers = [review_contracts.validate_finding_ledger(item) for item in finding_ledgers]
    return {
        "schema_version": "crouzeix-finding-reconciliation-context/v1",
        "reconciliation_id": reconciliation_id,
        "candidate_sha256s": [ledger["candidate_sha256"] for ledger in ledgers],
        "finding_ledger_sha256s": [ledger["finding_ledger_sha256"] for ledger in ledgers],
        "finding_ledgers": ledgers,
    }


def run_finding_reconciliation(
    review_dir: Path,
    *,
    ticket_id: str,
    reconciliation_id: str,
    finding_ledgers: Sequence[Mapping[str, Any]],
) -> dict[str, object]:
    root = Path(review_dir).resolve()
    context = build_reconciliation_context(
        reconciliation_id=reconciliation_id,
        finding_ledgers=finding_ledgers,
    )
    _require_ticket_file(root, ticket_id)
    ticket = _load_and_validate_ticket(
        root,
        ticket_id=ticket_id,
        role="finding_reconciliation",
        context=context,
        prompt=RECONCILIATION_PROMPT_SHA256,
        schema_path=None,
        parent_artifact_sha256=None,
    )
    _append_ticket_progress(root, ticket_id, "admitted", review_contracts.canonical_sha256(context))
    _append_ticket_progress(root, ticket_id, "running", review_contracts.canonical_sha256(ticket))
    try:
        decision = _build_repair_decision(reconciliation_id, context["finding_ledgers"])
        _write_repair_decision(root, decision)
    except ValidationError as error:
        _append_ticket_progress(
            root,
            ticket_id,
            "failed",
            review_contracts.canonical_sha256({"error": str(error)}),
        )
        raise
    _append_ticket_progress(root, ticket_id, "completed", decision["repair_decision_sha256"])
    _append_ticket_progress(root, ticket_id, "accepted", decision["repair_decision_sha256"])
    return decision


def _run_review_call(
    root: Path,
    *,
    ticket: Mapping[str, Any],
    call_id: str,
    provider_role: str,
    prompt: str,
    context: Mapping[str, Any],
    schema_path: Path,
    validator: Any,
) -> dict[str, Any]:
    if provider_role not in REVIEW_PROVIDER_ROLES:
        raise ValidationError(f"unknown review provider role {provider_role}")
    spec = _review_spec(read_run_spec(root / "run_spec.json"))
    context_value = dict(context)
    provider_prompt = (
        "REVIEW_CONTEXT_JSON: "
        + json.dumps(
            _provider_visible_context(provider_role, context_value),
            sort_keys=True,
            separators=(",", ":"),
        )
        + "\n\n"
        + prompt
    )
    binding = {
        "ticket_id": str(ticket["ticket_id"]),
        "ticket_sha256": tickets.canonical_sha256(ticket),
        "ticket_context_sha256": str(ticket["context_sha256"]),
        "ticket_schema_sha256": str(ticket["schema_sha256"]),
        "ticket_prompt_sha256": str(ticket["prompt_sha256"]),
        "ticket_parent_artifact_sha256": ticket["parent_artifact_sha256"],
    }
    validated_output = None
    terminal_ticket_event_sha256 = None
    _append_ticket_progress(
        root,
        str(ticket["ticket_id"]),
        "admitted",
        str(ticket["context_sha256"]),
    )
    _append_ticket_progress(
        root,
        str(ticket["ticket_id"]),
        "running",
        tickets.canonical_sha256(ticket),
    )

    def finalize_receipt(
        receipt: dict[str, Any], final_value: dict[str, Any] | None
    ) -> None:
        nonlocal validated_output
        if receipt["status"] != "completed":
            return
        try:
            if final_value is None:
                raise ValidationError("review provider produced no final value")
            validated_output = validator(final_value)
            _validate_output_binding(provider_role, context_value, validated_output)
        except ValidationError as error:
            receipt["status"] = "malformed"
            receipt["parse_error"] = str(error)
            validated_output = None

    result = runner.run_call(
        root,
        spec,
        call_id=call_id,
        role="repair" if provider_role == "repair" else "proof_progress_evaluator",
        prompt=provider_prompt,
        schema_path=schema_path,
        parent_digests=_parent_digests(ticket),
        ticket_binding=binding,
        allowed_tools=list(REVIEW_ALLOWED_TOOLS),
        finalize_receipt=finalize_receipt,
    )
    status = str(result["receipt"]["status"])
    if status == "completed":
        terminal_ticket_event_sha256 = _append_ticket_progress(
            root,
            str(ticket["ticket_id"]),
            "completed",
            review_contracts.canonical_sha256(result["receipt"]),
        )
        terminal_ticket_event_sha256 = _append_ticket_progress(
            root,
            str(ticket["ticket_id"]),
            "accepted",
            terminal_ticket_event_sha256,
        )
    else:
        terminal_ticket_event_sha256 = _append_ticket_progress(
            root,
            str(ticket["ticket_id"]),
            "failed",
            review_contracts.canonical_sha256(result["receipt"]),
        )
    result["validated_output"] = validated_output
    result["terminal_ticket_event_sha256"] = terminal_ticket_event_sha256
    return result


def _load_existing_repair_call(root: Path, call_id: str) -> dict[str, Any] | None:
    call_dir = root / "calls" / call_id
    if not call_dir.exists():
        return None
    receipt = read_strict_json_object(call_dir / "receipt.json", "repair call receipt")
    final = read_strict_json_object(call_dir / "final.json", "repair final response")
    validated_output = None
    if receipt["status"] == "completed":
        validated_output = review_contracts.validate_repair_payload(final)
    return {
        "call_dir": call_dir,
        "receipt": receipt,
        "final": final,
        "validated_output": validated_output,
        "terminal_ticket_event_sha256": _last_ticket_event_sha256(root, call_id),
    }


def _load_and_validate_ticket(
    root: Path,
    *,
    ticket_id: str,
    role: str,
    context: Mapping[str, Any],
    prompt: str,
    schema_path: Path | None,
    parent_artifact_sha256: str | None,
) -> dict[str, object]:
    ticket_path = root / "tickets" / ticket_id / "ticket.json"
    if not ticket_path.is_file() or ticket_path.is_symlink():
        raise ValidationError(f"runtime ticket is missing: {ticket_id}")
    ticket = tickets.validate_runtime_ticket(
        read_strict_json_object(ticket_path, f"{ticket_id} runtime ticket")
    )
    if ticket["ticket_id"] != ticket_id:
        raise ValidationError("runtime ticket ID mismatch")
    if ticket["task_kind"] != TASK_BY_ROLE[role]:
        raise ValidationError("runtime ticket task_kind does not match review action")
    if ticket["owner_type"] != OWNER_BY_ROLE[role]:
        raise ValidationError("runtime ticket owner_type does not match review action")
    if ticket["allowed_tools"] != REVIEW_ALLOWED_TOOLS:
        raise ValidationError("review ticket allowed_tools must be exactly Write")
    if ticket["context_sha256"] != review_contracts.canonical_sha256(context):
        raise ValidationError("ticket context_sha256 does not match review context")
    observed_prompt_sha256 = (
        prompt if _is_sha256(prompt) else sha256_bytes(prompt.encode("utf-8"))
    )
    if ticket["prompt_sha256"] != observed_prompt_sha256:
        raise ValidationError("ticket prompt_sha256 does not match review prompt")
    observed_schema_sha256 = (
        RECONCILIATION_SCHEMA_SHA256
        if schema_path is None
        else sha256_bytes(schema_path.read_bytes())
    )
    if ticket["schema_sha256"] != observed_schema_sha256:
        raise ValidationError("ticket schema_sha256 does not match review schema")
    if ticket["parent_artifact_sha256"] != parent_artifact_sha256:
        raise ValidationError("ticket parent_artifact_sha256 does not match review input")
    return ticket


def _require_ticket_file(root: Path, ticket_id: str) -> None:
    ticket_path = root / "tickets" / ticket_id / "ticket.json"
    if not ticket_path.is_file() or ticket_path.is_symlink():
        raise ValidationError(f"runtime ticket is missing: {ticket_id}")


def _require_re_review_tickets(
    root: Path,
    *,
    required_ticket_ids: Sequence[str],
    repaired_candidate_sha256: str | None,
    repaired_candidate_text: str | None = None,
    theorem_text: str = "Theorem.",
    required_context_marker_sha256s: Mapping[str, str] | None = None,
) -> None:
    missing = []
    for index, ticket_id in enumerate(required_ticket_ids, 1):
        ticket_path = root / "tickets" / ticket_id / "ticket.json"
        if not ticket_path.is_file() or ticket_path.is_symlink():
            missing.append(ticket_id)
            continue
        ticket = tickets.validate_runtime_ticket(
            read_strict_json_object(ticket_path, f"{ticket_id} runtime ticket")
        )
        if ticket["task_kind"] != "correctness_review":
            missing.append(ticket_id)
        elif (
            repaired_candidate_sha256 is not None
            and ticket["parent_artifact_sha256"] != repaired_candidate_sha256
        ):
            missing.append(ticket_id)
        elif repaired_candidate_text is not None and not _ticket_matches_repair_review_context(
            root,
            ticket,
            ticket_id=ticket_id,
            reviewer_index=index,
            repaired_candidate_text=repaired_candidate_text,
            theorem_text=theorem_text,
            required_context_marker_sha256=None
            if required_context_marker_sha256s is None
            else required_context_marker_sha256s[ticket_id],
        ):
            missing.append(ticket_id)
    if missing:
        reason = (
            "post-repair re-review tickets/context markers"
            if required_context_marker_sha256s is not None
            else "two fresh re-review tickets"
        )
        raise ValidationError(
            f"changed repair bytes require {reason}: " + ", ".join(missing)
        )


def _review_spec(spec: Mapping[str, Any]) -> dict[str, Any]:
    value = dict(spec)
    if value.get("network_access") is not False:
        raise ValidationError("review provider requires network_access=false")
    if value.get("approval_policy") != "never":
        raise ValidationError("review provider requires approval_policy=never")
    value["allowed_tools"] = list(REVIEW_ALLOWED_TOOLS)
    return value


def _provider_visible_context(role: str, context: Mapping[str, Any]) -> dict[str, Any]:
    if role == "correctness_review":
        value = review_contracts.validate_correctness_context(context)
        return {
            "schema_version": "crouzeix-correctness-provider-context/v1",
            "anonymous_candidate_id": value["anonymous_candidate_id"],
            "reviewer_index": value["reviewer_index"],
            "ticket_id": value["ticket_id"],
            "theorem_text": value["theorem_text"],
            "candidate_text": value["candidate_text"],
            "candidate_sha256": value["candidate_sha256"],
            "allowed_tools": value["allowed_tools"],
            "delegation_allowed": value["delegation_allowed"],
            "completion_criteria": value["completion_criteria"],
            "result_schema": value["result_schema"],
        }
    if role == "repair":
        value = review_contracts.validate_repair_request(context)
        return dict(value)
    if role == "mechanism_classification":
        value = review_contracts.validate_mechanism_classification_context(context)
        return dict(value)
    raise ValidationError(f"unknown review provider role {role}")


def _validate_output_binding(
    role: str,
    context: Mapping[str, Any],
    output: Mapping[str, Any],
) -> None:
    if role == "correctness_review":
        expected = context["candidate_sha256"]
        if output["candidate_sha256"] != expected:
            raise ValidationError("correctness review candidate digest mismatch")
        if output["reviewer_index"] != context["reviewer_index"]:
            raise ValidationError("correctness review reviewer index mismatch")
    elif role == "repair":
        for field in ("parent_candidate_sha256", "finding_ledger_sha256"):
            if output[field] != context[field]:
                raise ValidationError(f"repair output {field} mismatch")
    elif role == "mechanism_classification":
        if output["candidate_sha256"] != context["candidate_sha256"]:
            raise ValidationError("classification candidate digest mismatch")
        if output["frozen_correctness_sha256"] != context["frozen_correctness_sha256"]:
            raise ValidationError("classification frozen correctness digest mismatch")
    else:
        raise ValidationError(f"unknown review provider role {role}")


def _parent_digests(ticket: Mapping[str, Any]) -> dict[str, str]:
    parent = ticket["parent_artifact_sha256"]
    return {} if parent is None else {"parent": str(parent)}


def _read_prompt(root: Path, name: str) -> str:
    path = root / "prompts" / name
    if path.is_symlink() or not path.is_file():
        raise ValidationError(f"review prompt is missing: {name}")
    return path.read_text(encoding="utf-8")


def _canonical_sha256(value: object) -> str:
    return review_contracts.canonical_sha256(value)


def _append_ticket_progress(
    root: Path,
    ticket_id: str,
    to_state: str,
    artifact_sha256: str | None,
) -> str:
    event_path = root / "tickets" / ticket_id / "ticket_events.jsonl"
    if event_path.exists():
        lines = event_path.read_text(encoding="utf-8").splitlines()
        sequence = len(lines) + 1
        if lines:
            from_state = str(json.loads(lines[-1])["to_state"])
        else:
            from_state = "created"
    else:
        sequence = 1
        from_state = "created"
    event = {
        "schema_version": "crouzeix-runtime-ticket-event/v1",
        "sequence": sequence,
        "ticket_id": ticket_id,
        "from_state": from_state,
        "to_state": to_state,
        "reason": f"review action {to_state}",
        "occurred_at_utc": "2026-08-15T04:00:00Z",
        "artifact_sha256": artifact_sha256,
    }
    tickets.append_ticket_event(root / "tickets", ticket_id, event)
    return review_contracts.canonical_sha256(event)


def _last_ticket_event_sha256(root: Path, ticket_id: str) -> str:
    path = root / "tickets" / ticket_id / "ticket_events.jsonl"
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines:
        raise ValidationError("runtime ticket event log is empty")
    return review_contracts.canonical_sha256(json.loads(lines[-1]))


def _ticket_matches_repair_review_context(
    root: Path,
    ticket: Mapping[str, object],
    *,
    ticket_id: str,
    reviewer_index: int,
    repaired_candidate_text: str,
    theorem_text: str,
    required_context_marker_sha256: str | None,
) -> bool:
    context = review_contracts.build_correctness_context(
        theorem_text=theorem_text,
        candidate_text=repaired_candidate_text,
        candidate_projection={
            "schema_version": "crouzeix-candidate-projection/v1",
            "projection_id": "candidate-alpha",
            "source_node_id": "node-g2-d1",
            "source_node_artifact_sha256": "a" * 64,
            "source_reconciliation_sha256": "b" * 64,
            "candidate_sha256": sha256_bytes(repaired_candidate_text.encode("utf-8")),
            "candidate_byte_count": len(repaired_candidate_text.encode("utf-8")),
            "candidate_projection_sha256": "c" * 64,
        },
        reviewer_index=reviewer_index,
        ticket_id=ticket_id,
    )
    prompt = _read_prompt(root, "correctness_review.md")
    schema_path = root / "schemas/correctness_review.schema.json"
    return (
        ticket["context_sha256"]
        == (
            review_contracts.canonical_sha256(context)
            if required_context_marker_sha256 is None
            else required_context_marker_sha256
        )
        and ticket["prompt_sha256"] == sha256_bytes(prompt.encode("utf-8"))
        and ticket["schema_sha256"] == sha256_bytes(schema_path.read_bytes())
        and ticket["parent_artifact_sha256"] == context["candidate_sha256"]
    )


def _materialize_post_repair_context_markers(
    root: Path,
    *,
    repair_id: str,
    repaired_candidate_text: str,
    theorem_text: str,
) -> dict[str, str]:
    marker_root = root / "post_repair_review_contexts" / repair_id
    marker_root.mkdir(mode=0o700, parents=True, exist_ok=True)
    marker_digests = {}
    repaired_candidate_sha256 = sha256_bytes(repaired_candidate_text.encode("utf-8"))
    repaired_candidate_byte_count = len(repaired_candidate_text.encode("utf-8"))
    for reviewer_index in (1, 2):
        ticket_id = f"review-{repair_id}-r{reviewer_index}"
        context = review_contracts.build_correctness_context(
            theorem_text=theorem_text,
            candidate_text=repaired_candidate_text,
            candidate_projection={
                "schema_version": "crouzeix-candidate-projection/v1",
                "projection_id": "candidate-alpha",
                "source_node_id": "node-g2-d1",
                "source_node_artifact_sha256": "a" * 64,
                "source_reconciliation_sha256": "b" * 64,
                "candidate_sha256": repaired_candidate_sha256,
                "candidate_byte_count": repaired_candidate_byte_count,
                "candidate_projection_sha256": "c" * 64,
            },
            reviewer_index=reviewer_index,
            ticket_id=ticket_id,
        )
        marker = {
            "schema_version": "crouzeix-post-repair-review-context/v1",
            "repair_id": repair_id,
            "ticket_id": ticket_id,
            "reviewer_index": reviewer_index,
            "repaired_candidate_sha256": context["candidate_sha256"],
            "correctness_context": context,
            "correctness_context_sha256": review_contracts.canonical_sha256(context),
        }
        marker["post_repair_context_sha256"] = review_contracts.canonical_sha256(marker)
        path = marker_root / f"{ticket_id}.json"
        if not path.exists():
            with path.open("x", encoding="utf-8") as handle:
                json.dump(marker, handle, indent=2, sort_keys=True)
                handle.write("\n")
        else:
            existing = read_strict_json_object(path, "post-repair review context marker")
            if existing != marker:
                raise ValidationError(
                    "post-repair review context marker does not match repair output"
                )
        marker_digests[ticket_id] = str(marker["post_repair_context_sha256"])
    return marker_digests


def _build_repair_decision(
    reconciliation_id: str,
    finding_ledgers: Sequence[Mapping[str, object]],
) -> dict[str, object]:
    candidate_rows = []
    for ledger in finding_ledgers:
        validated = review_contracts.validate_finding_ledger(ledger)
        dispositions = list(validated["finding_dispositions"])
        unresolved = [
            row for row in dispositions if row["disposition"] == "unresolved"
        ]
        severity_counts = {
            "critical": sum(1 for row in unresolved if row["severity"] == "critical"),
            "major": sum(1 for row in unresolved if row["severity"] == "major"),
            "minor": sum(1 for row in unresolved if row["severity"] == "minor"),
        }
        candidate_rows.append(
            {
                "candidate_sha256": validated["candidate_sha256"],
                "finding_ledger_sha256": validated["finding_ledger_sha256"],
                "decision": "repair_required" if unresolved else "no_repair",
                "unresolved_count": len(unresolved),
                "unresolved_severity_counts": severity_counts,
                "finding_dispositions": dispositions,
            }
        )
    candidate_rows.sort(
        key=lambda row: (
            row["decision"] != "repair_required",
            row["unresolved_severity_counts"]["critical"],
            row["unresolved_severity_counts"]["major"],
            row["unresolved_severity_counts"]["minor"],
            row["candidate_sha256"],
        )
    )
    selected = next(
        (row for row in candidate_rows if row["decision"] == "repair_required"),
        None,
    )
    decision: dict[str, object] = {
        "schema_version": "crouzeix-repair-decision/v1",
        "reconciliation_id": reconciliation_id,
        "decision": "repair_required" if selected is not None else "no_repair",
        "selected_candidate_sha256": None
        if selected is None
        else selected["candidate_sha256"],
        "candidate_decisions": candidate_rows,
    }
    decision["repair_decision_sha256"] = review_contracts.canonical_sha256(decision)
    return decision


def _write_repair_decision(root: Path, decision: Mapping[str, object]) -> None:
    decision_root = root / "reconciliation_decisions"
    decision_root.mkdir(mode=0o700, exist_ok=True)
    path = decision_root / f"{decision['reconciliation_id']}.json"
    with path.open("x", encoding="utf-8") as handle:
        json.dump(decision, handle, indent=2, sort_keys=True)
        handle.write("\n")


def _is_sha256(value: str) -> bool:
    return (
        len(value) == 64
        and all(char in "0123456789abcdef" for char in value)
    )
