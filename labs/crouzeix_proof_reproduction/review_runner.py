from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Mapping, Sequence

import review_contracts
import runner
import tickets
from protocol import ValidationError, read_run_spec, read_strict_json_object, sha256_bytes


REVIEW_ALLOWED_TOOLS = ["Write"]
REVIEW_PROVIDER_ROLES = frozenset(
    {"correctness_review", "repair", "mechanism_classification"}
)
TASK_BY_ROLE = {
    "correctness_review": "correctness_review",
    "repair": "correctness_review",
    "mechanism_classification": "mechanism_classification",
}
OWNER_BY_ROLE = {
    "correctness_review": "reviewer",
    "repair": "reviewer",
    "mechanism_classification": "reviewer",
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
            terminal_ticket_event_sha256=receipt_sha256,
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
    _require_re_review_tickets(
        root,
        required_ticket_ids=required,
        repaired_candidate_sha256=None,
    )
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
            _require_re_review_tickets(
                root,
                required_ticket_ids=required,
                repaired_candidate_sha256=str(repair_result["repaired_candidate_sha256"]),
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
    result["validated_output"] = validated_output
    return result


def _load_and_validate_ticket(
    root: Path,
    *,
    ticket_id: str,
    role: str,
    context: Mapping[str, Any],
    prompt: str,
    schema_path: Path,
    parent_artifact_sha256: str,
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
    if ticket["prompt_sha256"] != sha256_bytes(prompt.encode("utf-8")):
        raise ValidationError("ticket prompt_sha256 does not match review prompt")
    if ticket["schema_sha256"] != sha256_bytes(schema_path.read_bytes()):
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
) -> None:
    missing = []
    for ticket_id in required_ticket_ids:
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
    if missing:
        raise ValidationError(
            "changed repair bytes require two fresh re-review tickets: "
            + ", ".join(missing)
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
