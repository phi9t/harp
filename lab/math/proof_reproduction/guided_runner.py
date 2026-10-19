from __future__ import annotations

import argparse
import json
import shutil
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Mapping, Protocol

import runner
import tickets
from protocol import ValidationError, read_run_spec, read_strict_json_object, sha256_bytes


GUIDED_TICKET_ID = "guided-reconstruction"
GUIDED_PROMPT = "guided_reconstruction.md"
GUIDED_SCHEMA = "guided_result.schema.json"
GUIDED_ALLOWED_TOOLS = ["Write"]
GUIDED_FORBIDDEN_SOURCES = [
    "search",
    "network",
    "mcp",
    "shell",
    "read",
    "delegation",
    "manuscript bytes",
    "formalization bytes",
    "source proof text",
    "public proof manuscripts",
]
CORRECTNESS_OUTCOMES = frozenset(
    {"complete", "incomplete", "invalid", "indeterminate", "no_candidate"}
)
NONCOMPLETE_OUTCOMES = CORRECTNESS_OUTCOMES - {"complete"}
MECHANISM_CARD_FIELDS = frozenset(
    {
        "schema_version",
        "card_id",
        "source_family",
        "attribution",
        "mechanism_summary",
        "allowed_guidance",
        "excluded_materials",
        "mechanism_card_sha256",
    }
)
ATTRIBUTION_FIELDS = frozenset({"source_id", "locator", "claim"})
GUIDED_RESULT_FIELDS = frozenset(
    {
        "schema_version",
        "run_id",
        "ticket_id",
        "mechanism_card_sha256",
        "theorem_sha256",
        "endpoint",
        "used_guidance",
        "unproved_obligations",
        "confidence_basis",
    }
)
ENDPOINT_FIELDS = frozenset({"kind", "text"})
OBLIGATION_FIELDS = frozenset({"statement", "locator"})
CALL_RECEIPT_FIELDS = frozenset(
    {
        "schema_version",
        "call_id",
        "role",
        "status",
        "cli",
        "model",
        "sandbox",
        "approval_policy",
        "network_access",
        "allowed_tools",
        "blocked_reason",
        "started_at_utc",
        "completed_at_utc",
        "returncode",
        "timed_out",
        "prompt_bytes",
        "prompt_sha256",
        "schema_bytes",
        "schema_sha256",
        "events_bytes",
        "events_sha256",
        "stderr_bytes",
        "stderr_sha256",
        "final_bytes",
        "final_sha256",
        "session_id",
        "usage",
        "event_types",
        "parse_error",
        "parent_digests",
        "ticket_id",
        "ticket_sha256",
        "ticket_context_sha256",
        "ticket_schema_sha256",
        "ticket_prompt_sha256",
        "ticket_parent_artifact_sha256",
    }
)
GUIDED_OUTPUT_RECEIPT_FIELDS = frozenset(
    {
        "schema_version",
        "run_id",
        "ticket_id",
        "ticket_sha256",
        "call_receipt_sha256",
        "terminal_status",
        "endpoint",
        "endpoint_sha256",
        "candidate_sha256",
        "blocker_sha256",
        "guided_result_sha256",
        "materialized_path",
        "content_addressed_path",
        "created_at_utc",
        "guided_receipt_sha256",
    }
)
FORBIDDEN_CARD_TEXT_MARKERS = (
    "manuscript bytes",
    "manuscript excerpt",
    "formalization bytes",
    "source proof text",
    "lean file",
    "proof bytes from",
)


@dataclass(frozen=True)
class ProviderAttempt:
    terminal_status: str
    provider_call: Mapping[str, Any]
    receipt: Mapping[str, Any]
    validated_output: Mapping[str, Any] | None
    reason: str | None = None


class GuidedProvider(Protocol):
    def run_guided(
        self,
        *,
        run_dir: Path,
        ticket: dict[str, object],
        context: dict[str, object],
    ) -> ProviderAttempt:
        ...


class TraeCliGuidedProvider:
    def run_guided(
        self,
        *,
        run_dir: Path,
        ticket: dict[str, object],
        context: dict[str, object],
    ) -> ProviderAttempt:
        prompt = _read_text(run_dir / "prompts" / GUIDED_PROMPT, "guided prompt")
        schema_path = run_dir / "schemas" / GUIDED_SCHEMA
        result = _run_guided_call(
            run_dir,
            ticket=ticket,
            context=context,
            prompt=prompt,
            schema_path=schema_path,
        )
        return ProviderAttempt(
            terminal_status=str(result["receipt"]["status"]),
            provider_call={"kind": "traecli", "call_id": GUIDED_TICKET_ID},
            receipt=result["receipt"],
            validated_output=result.get("validated_output"),
            reason=result["receipt"].get("blocked_reason") or result["receipt"].get("parse_error"),
        )


def should_run_guided(
    *,
    h_correctness_outcome: str,
    e_correctness_outcome: str,
    resource_preflight_passed: bool,
) -> bool:
    h = _correctness_outcome(h_correctness_outcome, "H correctness outcome")
    e = _correctness_outcome(e_correctness_outcome, "E correctness outcome")
    if not isinstance(resource_preflight_passed, bool):
        raise ValidationError("resource preflight result must be boolean")
    return h in NONCOMPLETE_OUTCOMES and e in NONCOMPLETE_OUTCOMES and resource_preflight_passed


def prepare_guided_run(
    run_dir: Path,
    *,
    theorem_path: Path,
    mechanism_card_path: Path,
    cli_identity: Mapping[str, Any],
    model: str,
    h_correctness_outcome: str,
    e_correctness_outcome: str,
    resource_preflight_passed: bool,
    timeout_seconds: int = 3600,
    leakage: str = "L3",
    run_id: str | None = None,
) -> dict[str, object]:
    root = Path(run_dir).resolve()
    if root.exists() or root.is_symlink():
        raise ValidationError(f"refusing existing guided run directory: {root}")
    trigger = _trigger_receipt(
        run_id=run_id or root.name,
        h_correctness_outcome=h_correctness_outcome,
        e_correctness_outcome=e_correctness_outcome,
        resource_preflight_passed=resource_preflight_passed,
    )
    root.mkdir(mode=0o700, parents=True)
    _write_create_only_json(root / "guided_trigger.json", trigger)
    if trigger["status"] == "not_applicable":
        return trigger

    if leakage != "L3":
        raise ValidationError("guided preparation requires leakage L3")
    theorem_text = _read_regular_text(Path(theorem_path), "theorem")
    theorem_sha256 = sha256_bytes(theorem_text.encode("utf-8"))
    card = validate_mechanism_card(
        read_strict_json_object(Path(mechanism_card_path), "mechanism card")
    )

    for relative in ("inputs", "prompts", "schemas", "contexts", "tickets", "calls"):
        (root / relative).mkdir(mode=0o700)
    _write_create_only_text(root / "inputs/theorem.txt", theorem_text)
    _write_create_only_json(root / "inputs/mechanism_card.json", card)
    shutil.copyfile(_lab_root() / "prompts" / GUIDED_PROMPT, root / "prompts" / GUIDED_PROMPT)
    shutil.copyfile(_lab_root() / "schemas" / GUIDED_SCHEMA, root / "schemas" / GUIDED_SCHEMA)

    prompt_sha256 = _file_sha256(root / "prompts" / GUIDED_PROMPT)
    schema_sha256 = _file_sha256(root / "schemas" / GUIDED_SCHEMA)
    context = build_guided_context(
        run_id=run_id or root.name,
        theorem_text=theorem_text,
        mechanism_card=card,
        prompt_sha256=prompt_sha256,
        schema_sha256=schema_sha256,
    )
    _write_create_only_json(root / "contexts" / f"{GUIDED_TICKET_ID}.json", context)
    ticket = runtime_ticket(
        run_dir=root,
        context=context,
        prompt_sha256=prompt_sha256,
        schema_sha256=schema_sha256,
        timeout_seconds=timeout_seconds,
    )
    tickets.publish_ticket(root / "tickets", ticket)

    spec = {
        "schema_version": "crouzeix-run-spec/v1",
        "run_id": run_id or root.name,
        "arm": "guided",
        "leakage": "L3",
        "model": _bounded_string(model, "model", 1, 128),
        "cli": _validate_cli_identity(cli_identity),
        "historical_prompt": {
            "source_bytes": 4106,
            "source_sha256": "0a0c3000b81efc4d9edc65ec3cd1d53df0d4e69b24bfee9fe0860301d853d6fc",
            "execution_bytes": 4106,
            "execution_sha256": "0a0c3000b81efc4d9edc65ec3cd1d53df0d4e69b24bfee9fe0860301d853d6fc",
            "normalization": "guided arm uses theorem and L3 mechanism card, not historical prompt bytes",
        },
        "sandbox": "workspace-write",
        "approval_policy": "never",
        "allowed_tools": list(GUIDED_ALLOWED_TOOLS),
        "network_access": False,
        "timeout_seconds": _bounded_integer(timeout_seconds, "timeout_seconds", 30, 14_400),
        "max_calls": 1,
        "token_accounting": {"boundary": "one guided reconstruction provider call"},
        "generation_visible_files": [
            "inputs/theorem.txt",
            "inputs/mechanism_card.json",
        ],
        "generation_excluded_classes": [
            "public proof manuscripts",
            "manuscript bytes",
            "formalization bytes",
            "source proof text",
        ],
        "guided": {
            "mechanism_card_sha256": card["mechanism_card_sha256"],
            "theorem_sha256": theorem_sha256,
            "prompt_sha256": prompt_sha256,
            "schema_sha256": schema_sha256,
            "resource_preflight_passed": True,
            "max_calls": 1,
        },
        "digests": {
            "inputs/theorem.txt": theorem_sha256,
            "inputs/mechanism_card.json": _canonical_sha256(card),
            "prompts/guided_reconstruction.md": prompt_sha256,
            "schemas/guided_result.schema.json": schema_sha256,
            "contexts/guided-reconstruction.json": _canonical_sha256(context),
            "tickets/guided-reconstruction/ticket.json": tickets.canonical_sha256(ticket),
        },
        "created_at_utc": _now(),
    }
    spec = read_run_spec(_write_then_read_spec(root / "run_spec.json", spec))
    receipt = {
        "schema_version": "crouzeix-guided-preparation-receipt/v1",
        "run_id": spec["run_id"],
        "status": "prepared",
        "trigger_sha256": _canonical_sha256(trigger),
        "run_spec_sha256": _canonical_sha256(spec),
        "ticket_id": GUIDED_TICKET_ID,
        "prepared_at_utc": _now(),
    }
    _write_create_only_json(root / "guided_preparation_receipt.json", receipt)
    return receipt


def build_guided_context(
    *,
    run_id: str,
    theorem_text: str,
    mechanism_card: Mapping[str, Any],
    prompt_sha256: str,
    schema_sha256: str,
) -> dict[str, object]:
    card = validate_mechanism_card(mechanism_card)
    theorem_sha256 = sha256_bytes(theorem_text.encode("utf-8"))
    return {
        "schema_version": "crouzeix-guided-context/v1",
        "run_id": _runtime_id(run_id, "run_id"),
        "ticket_id": GUIDED_TICKET_ID,
        "leakage": "L3",
        "theorem_text": _bounded_string(theorem_text, "theorem_text", 1, 64_000),
        "theorem_sha256": theorem_sha256,
        "mechanism_card": card,
        "mechanism_card_sha256": card["mechanism_card_sha256"],
        "allowed_tools": list(GUIDED_ALLOWED_TOOLS),
        "forbidden_sources": list(GUIDED_FORBIDDEN_SOURCES),
        "delegation_allowed": False,
        "result_schema": "guided_result",
        "prompt_sha256": prompt_sha256,
        "schema_sha256": schema_sha256,
        "max_calls": 1,
        "diagnostic_claim_ceiling": (
            "Mechanism-guided reconstruction under supplied context; not independent rediscovery."
        ),
    }


def runtime_ticket(
    *,
    run_dir: Path,
    context: Mapping[str, Any],
    prompt_sha256: str,
    schema_sha256: str,
    timeout_seconds: int,
) -> dict[str, object]:
    ticket = {
        "schema_version": "crouzeix-runtime-ticket/v1",
        "ticket_id": GUIDED_TICKET_ID,
        "run_id": Path(run_dir).name,
        "task_kind": "expert",
        "node_id": None,
        "parent_node_id": None,
        "generation": 0,
        "direction_id": None,
        "role": "guided_reconstruction",
        "objective": "Run one mechanism-guided diagnostic reconstruction.",
        "expected_deliverable": "Strict guided candidate or blocker result.",
        "dependency_ticket_ids": [],
        "context_sha256": _canonical_sha256(context),
        "schema_sha256": schema_sha256,
        "prompt_sha256": prompt_sha256,
        "parent_artifact_sha256": context["mechanism_card_sha256"],
        "allowed_tools": list(GUIDED_ALLOWED_TOOLS),
        "forbidden_sources": list(GUIDED_FORBIDDEN_SOURCES),
        "timeout_seconds": timeout_seconds,
        "max_output_bytes": 2 * 1024 * 1024,
        "owner_type": "expert",
        "created_at_utc": _now(),
        "state": "created",
    }
    return tickets.validate_runtime_ticket(ticket)


def run_guided_reconstruction(
    run_dir: Path,
    *,
    provider: GuidedProvider | None = None,
) -> dict[str, object]:
    root = Path(run_dir).resolve()
    spec = read_run_spec(root / "run_spec.json")
    if spec.get("arm") != "guided" or "guided" not in spec:
        raise ValidationError("guided reconstruction requires prepared guided run")
    if (root / "guided_run_receipt.json").exists():
        raise ValidationError("refusing to rerun guided reconstruction")
    if not (root / "calls").is_dir():
        raise ValidationError("guided calls directory is missing")
    ticket = _read_ticket(root)
    context = read_strict_json_object(
        root / "contexts" / f"{GUIDED_TICKET_ID}.json",
        "guided context",
    )
    _validate_context_against_ticket(context, ticket)
    engine = provider or TraeCliGuidedProvider()
    _append_ticket_progress(root, "admitted", str(ticket["context_sha256"]))
    _append_ticket_progress(root, "running", tickets.canonical_sha256(ticket))
    attempt = engine.run_guided(run_dir=root, ticket=ticket, context=context)
    try:
        call_receipt = _load_typed_call_receipt(root, ticket)
        if _canonical_sha256(call_receipt) != _canonical_sha256(attempt.receipt):
            raise ValidationError("provider call receipt does not match persisted call receipt")
        if call_receipt["status"] != attempt.terminal_status:
            raise ValidationError("provider call receipt status does not match attempt")
    except ValidationError as error:
        failure = {"provider_receipt": dict(attempt.receipt), "error": str(error)}
        terminal_artifact = _canonical_sha256(failure)
        _append_ticket_progress(root, "failed", terminal_artifact)
        return _write_guided_run_receipt(
            root,
            spec=spec,
            terminal_status="failed",
            provider_status=attempt.terminal_status,
            provider_call=attempt.provider_call,
            provider_receipt=attempt.receipt,
            provider_reason=str(error),
            guided_receipt_sha256=None,
        )
    terminal_status = _terminal_status(str(call_receipt["status"]))
    guided_receipt: dict[str, object] | None = None
    terminal_artifact = _canonical_sha256(call_receipt)
    if terminal_status == "completed" and attempt.validated_output is not None:
        try:
            output = validate_guided_result(
                attempt.validated_output,
                run_id=str(spec["run_id"]),
                theorem_sha256=str(spec["guided"]["theorem_sha256"]),
                mechanism_card_sha256=str(spec["guided"]["mechanism_card_sha256"]),
            )
        except ValidationError as error:
            failure = {"provider_receipt": dict(attempt.receipt), "error": str(error)}
            terminal_artifact = _canonical_sha256(failure)
            _append_ticket_progress(root, "failed", terminal_artifact)
            final_status = "failed"
            guided_receipt = None
        else:
            guided_receipt = _materialize_guided_output(
                root,
                output=output,
                ticket=ticket,
                provider_receipt=call_receipt,
            )
            terminal_artifact = str(guided_receipt["guided_receipt_sha256"])
            _append_ticket_progress(root, "completed", terminal_artifact)
            _append_ticket_progress(root, "accepted", terminal_artifact)
            final_status = str(guided_receipt["terminal_status"])
    else:
        ticket_status = "failed" if terminal_status == "malformed" else terminal_status
        _append_ticket_progress(root, ticket_status, terminal_artifact)
        final_status = ticket_status
    return _write_guided_run_receipt(
        root,
        spec=spec,
        terminal_status=final_status,
        provider_status=attempt.terminal_status,
        provider_call=attempt.provider_call,
        provider_receipt=call_receipt,
        provider_reason=attempt.reason,
        guided_receipt_sha256=None
        if guided_receipt is None
        else guided_receipt["guided_receipt_sha256"],
    )


def _write_guided_run_receipt(
    root: Path,
    *,
    spec: Mapping[str, Any],
    terminal_status: str,
    provider_status: str,
    provider_call: Mapping[str, Any],
    provider_receipt: Mapping[str, Any],
    provider_reason: str | None,
    guided_receipt_sha256: object | None,
) -> dict[str, object]:
    run_receipt = {
        "schema_version": "crouzeix-guided-run-receipt/v1",
        "run_id": spec["run_id"],
        "arm": "guided",
        "leakage": "L3",
        "call_count": 1,
        "terminal_status": terminal_status,
        "provider_status": provider_status,
        "provider_call": dict(provider_call),
        "provider_receipt": dict(provider_receipt),
        "provider_reason": provider_reason,
        "guided_receipt_sha256": guided_receipt_sha256,
        "completed_at_utc": _now(),
    }
    run_receipt["guided_run_receipt_sha256"] = _canonical_sha256(run_receipt)
    _write_create_only_json(root / "guided_run_receipt.json", run_receipt)
    return run_receipt


def check_guided_run(run_dir: Path) -> dict[str, object]:
    root = Path(run_dir).resolve()
    trigger = read_strict_json_object(root / "guided_trigger.json", "guided trigger")
    if trigger.get("status") == "not_applicable":
        return {
            "schema_version": "crouzeix-guided-check/v1",
            "run_id": trigger["run_id"],
            "status": "ok",
            "terminal_status": "not_applicable",
            "call_count": 0,
        }
    spec = read_run_spec(root / "run_spec.json")
    preparation = read_strict_json_object(
        root / "guided_preparation_receipt.json",
        "guided preparation receipt",
    )
    if preparation["run_spec_sha256"] != _canonical_sha256(spec):
        raise ValidationError("guided preparation receipt does not match run_spec")
    run_receipt = read_strict_json_object(root / "guided_run_receipt.json", "guided run receipt")
    if run_receipt["guided_run_receipt_sha256"] != _canonical_sha256(
        {key: value for key, value in run_receipt.items() if key != "guided_run_receipt_sha256"}
    ):
        raise ValidationError("guided run receipt digest mismatch")
    ticket = _read_ticket(root)
    _validate_terminal_ticket(root, ticket)
    call_receipts = _load_guided_call_receipts(root, ticket)
    call_count = len(call_receipts)
    if run_receipt["call_count"] != call_count or call_count != 1:
        raise ValidationError("guided run must account for exactly one call")
    call_receipt = call_receipts[0]
    if _canonical_sha256(run_receipt["provider_receipt"]) != _canonical_sha256(call_receipt):
        raise ValidationError("guided run receipt provider call receipt mismatch")
    terminal_status = str(run_receipt["terminal_status"])
    if terminal_status in {"candidate", "blocker"}:
        _validate_materialized_guided_output(root, ticket, call_receipt, terminal_status)
    elif (root / "guided_candidate").exists() or (root / "guided_blocker").exists():
        raise ValidationError("failed guided run cannot retain materialized output")
    return {
        "schema_version": "crouzeix-guided-check/v1",
        "run_id": spec["run_id"],
        "status": "ok",
        "terminal_status": terminal_status,
        "call_count": call_count,
    }


def validate_mechanism_card(value: Mapping[str, Any]) -> dict[str, object]:
    _require_fields(value, MECHANISM_CARD_FIELDS, "mechanism card")
    _require_equal(
        value["schema_version"],
        "crouzeix-guided-mechanism-card/v1",
        "mechanism card schema_version",
    )
    _runtime_id(value["card_id"], "card_id")
    _bounded_string(value["source_family"], "source_family", 1, 256)
    attributions = value["attribution"]
    if not isinstance(attributions, list) or not 1 <= len(attributions) <= 32:
        raise ValidationError("mechanism card attribution must be a nonempty bounded list")
    normalized_attributions = []
    for attribution in attributions:
        item = _mapping(attribution, "mechanism card attribution")
        _require_fields(item, ATTRIBUTION_FIELDS, "mechanism card attribution")
        normalized_attributions.append(
            {
                "source_id": _bounded_string(item["source_id"], "source_id", 1, 256),
                "locator": _bounded_string(item["locator"], "locator", 1, 512),
                "claim": _reject_forbidden_card_text(
                    _bounded_string(item["claim"], "claim", 1, 4096),
                    "attribution.claim",
                ),
            }
        )
    summary = _reject_forbidden_card_text(
        _bounded_string(value["mechanism_summary"], "mechanism_summary", 1, 16_000),
        "mechanism_summary",
    )
    guidance = [
        _reject_forbidden_card_text(item, "allowed_guidance")
        for item in _string_list(value["allowed_guidance"], "allowed_guidance", 1, 32)
    ]
    excluded = _string_list(value["excluded_materials"], "excluded_materials", 3, 16)
    required_exclusions = {"manuscript bytes", "formalization bytes", "source proof text"}
    if not required_exclusions.issubset({item.lower() for item in excluded}):
        raise ValidationError("mechanism card must exclude manuscript, formalization, and source proof text")
    result = dict(value)
    result["attribution"] = normalized_attributions
    result["mechanism_summary"] = summary
    result["allowed_guidance"] = guidance
    result["excluded_materials"] = excluded
    digest = _digest(value["mechanism_card_sha256"], "mechanism_card_sha256")
    without_digest = dict(result)
    without_digest.pop("mechanism_card_sha256")
    if digest != _canonical_sha256(without_digest):
        raise ValidationError("mechanism card digest mismatch")
    return result


def validate_guided_result(
    value: Mapping[str, Any],
    *,
    run_id: str,
    theorem_sha256: str,
    mechanism_card_sha256: str,
) -> dict[str, object]:
    _require_fields(value, GUIDED_RESULT_FIELDS, "guided result")
    _require_equal(value["schema_version"], "crouzeix-guided-result/v1", "schema_version")
    if value["run_id"] != run_id:
        raise ValidationError("guided result run_id mismatch")
    if value["ticket_id"] != GUIDED_TICKET_ID:
        raise ValidationError("guided result ticket_id mismatch")
    if value["theorem_sha256"] != theorem_sha256:
        raise ValidationError("guided result theorem_sha256 mismatch")
    if value["mechanism_card_sha256"] != mechanism_card_sha256:
        raise ValidationError("guided result mechanism_card_sha256 mismatch")
    endpoint = _mapping(value["endpoint"], "guided endpoint")
    _require_fields(endpoint, ENDPOINT_FIELDS, "guided endpoint")
    kind = _enum(endpoint["kind"], frozenset({"candidate_proof", "blocker"}), "endpoint.kind")
    text = _bounded_string(endpoint["text"], "endpoint.text", 1, 1_000_000)
    guidance = _string_list(value["used_guidance"], "used_guidance", 1, 32)
    obligations = value["unproved_obligations"]
    if not isinstance(obligations, list) or len(obligations) > 64:
        raise ValidationError("unproved_obligations must be a bounded list")
    normalized_obligations = []
    for obligation in obligations:
        item = _mapping(obligation, "unproved obligation")
        _require_fields(item, OBLIGATION_FIELDS, "unproved obligation")
        normalized_obligations.append(
            {
                "statement": _bounded_string(item["statement"], "obligation.statement", 1, 4096),
                "locator": _bounded_string(item["locator"], "obligation.locator", 1, 512),
            }
        )
    return {
        "schema_version": value["schema_version"],
        "run_id": value["run_id"],
        "ticket_id": value["ticket_id"],
        "mechanism_card_sha256": value["mechanism_card_sha256"],
        "theorem_sha256": value["theorem_sha256"],
        "endpoint": {"kind": kind, "text": text},
        "used_guidance": guidance,
        "unproved_obligations": normalized_obligations,
        "confidence_basis": _bounded_string(value["confidence_basis"], "confidence_basis", 1, 4096),
    }


def _run_guided_call(
    run_dir: Path,
    *,
    ticket: Mapping[str, Any],
    context: Mapping[str, Any],
    prompt: str,
    schema_path: Path,
) -> dict[str, Any]:
    spec = read_run_spec(run_dir / "run_spec.json")
    provider_prompt = (
        "GUIDED_CONTEXT_JSON: "
        + json.dumps(_provider_visible_context(context), sort_keys=True, separators=(",", ":"))
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

    def finalize_receipt(receipt: dict[str, Any], final_value: dict[str, Any] | None) -> None:
        nonlocal validated_output
        if receipt["status"] != "completed":
            return
        if final_value is None:
            receipt["status"] = "malformed"
            receipt["parse_error"] = "guided provider produced no final value"
            return
        try:
            validated_output = validate_guided_result(
                final_value,
                run_id=str(spec["run_id"]),
                theorem_sha256=str(spec["guided"]["theorem_sha256"]),
                mechanism_card_sha256=str(spec["guided"]["mechanism_card_sha256"]),
            )
        except ValidationError as error:
            receipt["status"] = "malformed"
            receipt["parse_error"] = str(error)

    result = runner.run_call(
        run_dir,
        spec,
        call_id=GUIDED_TICKET_ID,
        role="expert",
        prompt=provider_prompt,
        schema_path=schema_path,
        parent_digests={
            "theorem": str(spec["guided"]["theorem_sha256"]),
            "mechanism_card": str(spec["guided"]["mechanism_card_sha256"]),
        },
        ticket_binding=binding,
        allowed_tools=list(GUIDED_ALLOWED_TOOLS),
        finalize_receipt=finalize_receipt,
    )
    result["validated_output"] = validated_output
    return result


def _provider_visible_context(context: Mapping[str, Any]) -> dict[str, object]:
    return {
        "schema_version": context["schema_version"],
        "run_id": context["run_id"],
        "ticket_id": context["ticket_id"],
        "leakage": context["leakage"],
        "theorem_text": context["theorem_text"],
        "theorem_sha256": context["theorem_sha256"],
        "mechanism_card": context["mechanism_card"],
        "mechanism_card_sha256": context["mechanism_card_sha256"],
        "allowed_tools": context["allowed_tools"],
        "forbidden_sources": context["forbidden_sources"],
        "delegation_allowed": context["delegation_allowed"],
        "result_schema": context["result_schema"],
        "max_calls": context["max_calls"],
        "diagnostic_claim_ceiling": context["diagnostic_claim_ceiling"],
    }


def _materialize_guided_output(
    root: Path,
    *,
    output: Mapping[str, Any],
    ticket: Mapping[str, Any],
    provider_receipt: Mapping[str, Any],
) -> dict[str, object]:
    endpoint = output["endpoint"]
    endpoint_text = str(endpoint["text"])
    endpoint_sha256 = sha256_bytes(endpoint_text.encode("utf-8"))
    if endpoint["kind"] == "candidate_proof":
        output_dir = root / "guided_candidate"
        output_path = output_dir / "candidate.tex"
        candidate_sha256 = endpoint_sha256
        blocker_sha256 = None
        terminal_status = "candidate"
    else:
        output_dir = root / "guided_blocker"
        output_path = output_dir / "blocker.txt"
        candidate_sha256 = None
        blocker_sha256 = endpoint_sha256
        terminal_status = "blocker"
    if output_dir.exists() or output_dir.is_symlink():
        raise ValidationError("refusing existing guided output directory")
    output_dir.mkdir(mode=0o700)
    _write_create_only_text(output_path, endpoint_text)
    content_addressed_path = output_dir / (
        f"{endpoint_sha256}.tex" if endpoint["kind"] == "candidate_proof" else f"{endpoint_sha256}.txt"
    )
    _write_create_only_text(content_addressed_path, endpoint_text)
    receipt = {
        "schema_version": "crouzeix-guided-output-receipt/v1",
        "run_id": output["run_id"],
        "ticket_id": output["ticket_id"],
        "ticket_sha256": tickets.canonical_sha256(ticket),
        "call_receipt_sha256": _canonical_sha256(provider_receipt),
        "terminal_status": terminal_status,
        "endpoint": dict(endpoint),
        "endpoint_sha256": endpoint_sha256,
        "candidate_sha256": candidate_sha256,
        "blocker_sha256": blocker_sha256,
        "guided_result_sha256": _canonical_sha256(output),
        "materialized_path": output_path.relative_to(root).as_posix(),
        "content_addressed_path": content_addressed_path.relative_to(root).as_posix(),
        "created_at_utc": _now(),
    }
    receipt["guided_receipt_sha256"] = _canonical_sha256(receipt)
    _write_create_only_json(output_dir / "receipt.json", receipt)
    return receipt


def _trigger_receipt(
    *,
    run_id: str,
    h_correctness_outcome: str,
    e_correctness_outcome: str,
    resource_preflight_passed: bool,
) -> dict[str, object]:
    should_run = should_run_guided(
        h_correctness_outcome=h_correctness_outcome,
        e_correctness_outcome=e_correctness_outcome,
        resource_preflight_passed=resource_preflight_passed,
    )
    receipt = {
        "schema_version": "crouzeix-guided-trigger/v1",
        "run_id": _runtime_id(run_id, "run_id"),
        "h_correctness_outcome": h_correctness_outcome,
        "e_correctness_outcome": e_correctness_outcome,
        "resource_preflight_passed": resource_preflight_passed,
        "decision": "run" if should_run else "skip",
        "status": "applicable" if should_run else "not_applicable",
        "decided_at_utc": _now(),
    }
    receipt["guided_trigger_sha256"] = _canonical_sha256(receipt)
    return receipt


def _append_ticket_progress(
    root: Path,
    to_state: str,
    artifact_sha256: str | None,
) -> str:
    event_path = root / "tickets" / GUIDED_TICKET_ID / "ticket_events.jsonl"
    state = "created"
    sequence = 1
    if event_path.exists():
        lines = event_path.read_text(encoding="utf-8").splitlines()
        if lines:
            last = json.loads(lines[-1])
            state = str(last["to_state"])
            sequence = int(last["sequence"]) + 1
    event = {
        "schema_version": "crouzeix-runtime-ticket-event/v1",
        "sequence": sequence,
        "ticket_id": GUIDED_TICKET_ID,
        "from_state": state,
        "to_state": to_state,
        "reason": f"guided reconstruction {to_state}",
        "occurred_at_utc": _now(),
        "artifact_sha256": artifact_sha256,
    }
    tickets.append_ticket_event(root / "tickets", GUIDED_TICKET_ID, event)
    return _canonical_sha256(event)


def _validate_context_against_ticket(context: Mapping[str, Any], ticket: Mapping[str, Any]) -> None:
    if _canonical_sha256(context) != ticket["context_sha256"]:
        raise ValidationError("guided context does not match ticket")
    if context.get("delegation_allowed") is not False:
        raise ValidationError("guided context must set delegation_allowed=false")
    if context.get("allowed_tools") != GUIDED_ALLOWED_TOOLS:
        raise ValidationError("guided context allowed_tools must be exactly Write")
    if context.get("ticket_id") != ticket["ticket_id"]:
        raise ValidationError("guided context ticket_id must match ticket")
    if context.get("mechanism_card_sha256") != ticket["parent_artifact_sha256"]:
        raise ValidationError("guided context mechanism card digest must match ticket")


def _load_typed_call_receipt(root: Path, ticket: Mapping[str, Any]) -> dict[str, object]:
    receipt_path = root / "calls" / GUIDED_TICKET_ID / "receipt.json"
    receipt = validate_guided_call_receipt(
        read_strict_json_object(receipt_path, "guided call receipt"),
        ticket=ticket,
    )
    return receipt


def _load_guided_call_receipts(root: Path, ticket: Mapping[str, Any]) -> list[dict[str, object]]:
    calls_root = root / "calls"
    if not calls_root.is_dir() or calls_root.is_symlink():
        raise ValidationError("guided calls directory is missing")
    call_dirs = [path for path in calls_root.iterdir() if path.is_dir() and not path.is_symlink()]
    if len(call_dirs) != 1 or call_dirs[0].name != GUIDED_TICKET_ID:
        raise ValidationError("guided call receipt count must be exactly one")
    return [_load_typed_call_receipt(root, ticket)]


def validate_guided_call_receipt(
    value: Mapping[str, Any],
    *,
    ticket: Mapping[str, Any],
) -> dict[str, object]:
    _require_fields(value, CALL_RECEIPT_FIELDS, "guided call receipt")
    _require_equal(value["schema_version"], "crouzeix-call-receipt/v1", "call receipt schema_version")
    _require_equal(value["call_id"], GUIDED_TICKET_ID, "call_id")
    _require_equal(value["role"], "expert", "call role")
    _enum(value["status"], frozenset({"completed", "failed", "timed_out", "malformed", "blocked_resource"}), "call status")
    _validate_cli_identity(value["cli"])
    _bounded_string(value["model"], "model", 1, 128)
    _require_equal(value["sandbox"], "workspace-write", "sandbox")
    _require_equal(value["approval_policy"], "never", "approval_policy")
    if value["network_access"] is not False:
        raise ValidationError("guided call receipt network_access must be false")
    if value["allowed_tools"] != GUIDED_ALLOWED_TOOLS:
        raise ValidationError("guided call receipt allowed_tools must be exactly Write")
    if value["blocked_reason"] is not None:
        _bounded_string(value["blocked_reason"], "blocked_reason", 1, 4096)
    _bounded_string(value["started_at_utc"], "started_at_utc", 1, 64)
    _bounded_string(value["completed_at_utc"], "completed_at_utc", 1, 64)
    if value["returncode"] is not None:
        _bounded_integer(value["returncode"], "returncode", -255, 255)
    if not isinstance(value["timed_out"], bool):
        raise ValidationError("guided call receipt timed_out must be boolean")
    _bounded_integer(value["prompt_bytes"], "prompt_bytes", 1, 1024 * 1024)
    _digest(value["prompt_sha256"], "prompt_sha256")
    _bounded_integer(value["schema_bytes"], "schema_bytes", 1, 1024 * 1024)
    _digest(value["schema_sha256"], "schema_sha256")
    _bounded_integer(value["events_bytes"], "events_bytes", 0, 16 * 1024 * 1024)
    if value["events_sha256"] is not None:
        _digest(value["events_sha256"], "events_sha256")
    _bounded_integer(value["stderr_bytes"], "stderr_bytes", 0, 4 * 1024 * 1024)
    if value["stderr_sha256"] is not None:
        _digest(value["stderr_sha256"], "stderr_sha256")
    if value["final_bytes"] is not None:
        _bounded_integer(value["final_bytes"], "final_bytes", 1, 2 * 1024 * 1024)
    if value["final_sha256"] is not None:
        _digest(value["final_sha256"], "final_sha256")
    if value["session_id"] is not None:
        _bounded_string(value["session_id"], "session_id", 1, 256)
    if value["usage"] is not None and not isinstance(value["usage"], dict):
        raise ValidationError("guided call receipt usage must be object or null")
    if not isinstance(value["event_types"], list) or not all(isinstance(item, str) for item in value["event_types"]):
        raise ValidationError("guided call receipt event_types must be strings")
    if value["parse_error"] is not None:
        _bounded_string(value["parse_error"], "parse_error", 1, 4096)
    parent_digests = _mapping(value["parent_digests"], "parent_digests")
    if set(parent_digests) != {"theorem", "mechanism_card"}:
        raise ValidationError("guided call receipt parent_digests must bind theorem and mechanism_card")
    _digest(parent_digests["theorem"], "parent_digests.theorem")
    _digest(parent_digests["mechanism_card"], "parent_digests.mechanism_card")
    if value["ticket_id"] != ticket["ticket_id"]:
        raise ValidationError("guided call receipt ticket_id does not match ticket")
    if value["ticket_sha256"] != tickets.canonical_sha256(ticket):
        raise ValidationError("guided call receipt ticket_sha256 does not match ticket")
    for receipt_field, ticket_field in (
        ("ticket_context_sha256", "context_sha256"),
        ("ticket_schema_sha256", "schema_sha256"),
        ("ticket_prompt_sha256", "prompt_sha256"),
        ("ticket_parent_artifact_sha256", "parent_artifact_sha256"),
    ):
        if value[receipt_field] != ticket[ticket_field]:
            raise ValidationError(f"guided call receipt {receipt_field} does not match ticket")
    return dict(value)


def _validate_materialized_guided_output(
    root: Path,
    ticket: Mapping[str, Any],
    call_receipt: Mapping[str, Any],
    terminal_status: str,
) -> dict[str, object]:
    output_dir = root / ("guided_candidate" if terminal_status == "candidate" else "guided_blocker")
    receipt = read_strict_json_object(output_dir / "receipt.json", "guided output receipt")
    _require_fields(receipt, GUIDED_OUTPUT_RECEIPT_FIELDS, "guided output receipt")
    if receipt["guided_receipt_sha256"] != _canonical_sha256(
        {key: value for key, value in receipt.items() if key != "guided_receipt_sha256"}
    ):
        raise ValidationError("guided output receipt digest mismatch")
    if receipt["schema_version"] != "crouzeix-guided-output-receipt/v1":
        raise ValidationError("guided output receipt schema_version is invalid")
    if receipt["ticket_id"] != ticket["ticket_id"] or receipt["ticket_sha256"] != tickets.canonical_sha256(ticket):
        raise ValidationError("guided output receipt ticket binding mismatch")
    if receipt["call_receipt_sha256"] != _canonical_sha256(call_receipt):
        raise ValidationError("guided output receipt call receipt digest mismatch")
    if receipt["terminal_status"] != terminal_status:
        raise ValidationError("guided output receipt terminal status mismatch")
    endpoint = _mapping(receipt["endpoint"], "guided output endpoint")
    _require_fields(endpoint, ENDPOINT_FIELDS, "guided output endpoint")
    expected_kind = "candidate_proof" if terminal_status == "candidate" else "blocker"
    if endpoint["kind"] != expected_kind:
        raise ValidationError("guided output endpoint kind mismatch")
    endpoint_text = _bounded_string(endpoint["text"], "guided output endpoint.text", 1, 1_000_000)
    endpoint_sha256 = sha256_bytes(endpoint_text.encode("utf-8"))
    if receipt["endpoint_sha256"] != endpoint_sha256:
        raise ValidationError("guided output endpoint digest mismatch")
    _digest(receipt["guided_result_sha256"], "guided_result_sha256")
    materialized_path = _safe_relative_runtime_path(str(receipt["materialized_path"]), "materialized_path")
    content_path = _safe_relative_runtime_path(str(receipt["content_addressed_path"]), "content_addressed_path")
    if terminal_status == "candidate":
        if receipt["candidate_sha256"] != endpoint_sha256 or receipt["blocker_sha256"] is not None:
            raise ValidationError("guided output candidate digest mismatch")
        expected_materialized = "guided_candidate/candidate.tex"
        expected_content = f"guided_candidate/{endpoint_sha256}.tex"
        label = "candidate"
    else:
        if receipt["blocker_sha256"] != endpoint_sha256 or receipt["candidate_sha256"] is not None:
            raise ValidationError("guided output blocker digest mismatch")
        expected_materialized = "guided_blocker/blocker.txt"
        expected_content = f"guided_blocker/{endpoint_sha256}.txt"
        label = "blocker"
    if materialized_path != expected_materialized or content_path != expected_content:
        raise ValidationError(f"guided output {label} content-addressed path mismatch")
    for relative_path in (materialized_path, content_path):
        path = root / relative_path
        if path.is_symlink() or not path.is_file():
            raise ValidationError(f"guided output {label} file is missing")
        if path.read_text(encoding="utf-8") != endpoint_text:
            raise ValidationError(f"guided output {label} content mismatch")
    return dict(receipt)


def _validate_terminal_ticket(root: Path, ticket: Mapping[str, Any]) -> None:
    path = root / "tickets" / str(ticket["ticket_id"]) / "ticket_events.jsonl"
    if not path.is_file():
        raise ValidationError("guided ticket has no event log")
    lines = [json.loads(line) for line in path.read_text(encoding="utf-8").splitlines()]
    if not lines or lines[-1]["to_state"] not in tickets.TERMINAL_TICKET_STATES:
        raise ValidationError("guided ticket is not terminal")


def _read_ticket(root: Path) -> dict[str, object]:
    return tickets.validate_runtime_ticket(
        read_strict_json_object(
            root / "tickets" / GUIDED_TICKET_ID / "ticket.json",
            "guided runtime ticket",
        )
    )


def _write_then_read_spec(path: Path, value: Mapping[str, Any]) -> Path:
    _write_create_only_json(path, value)
    return path


def _read_regular_text(path: Path, label: str) -> str:
    if path.is_symlink() or not path.is_file():
        raise ValidationError(f"{label} must be a regular non-symlinked file")
    return path.read_text(encoding="utf-8")


def _read_text(path: Path, label: str) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as error:
        raise ValidationError(f"cannot read {label}: {error}") from error


def _write_create_only_text(path: Path, text: str) -> None:
    try:
        with path.open("x", encoding="utf-8") as handle:
            handle.write(text)
    except FileExistsError as error:
        raise ValidationError(f"refusing existing file: {path}") from error


def _write_create_only_json(path: Path, value: Mapping[str, Any]) -> None:
    try:
        with path.open("x", encoding="utf-8") as handle:
            json.dump(value, handle, indent=2, sort_keys=True)
            handle.write("\n")
    except FileExistsError as error:
        raise ValidationError(f"refusing existing file: {path}") from error


def _file_sha256(path: Path) -> str:
    return sha256_bytes(path.read_bytes())


def _canonical_sha256(value: object) -> str:
    return sha256_bytes(
        json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
    )


def _lab_root() -> Path:
    return Path(__file__).resolve().parent


def _now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def _correctness_outcome(value: str, label: str) -> str:
    return _enum(value, CORRECTNESS_OUTCOMES, label)


def _terminal_status(value: str) -> str:
    return _enum(
        value,
        frozenset({"completed", "failed", "timed_out", "blocked_resource", "malformed"}),
        "terminal status",
    )


def _validate_cli_identity(value: Mapping[str, Any]) -> dict[str, object]:
    item = _mapping(value, "cli")
    _require_fields(item, frozenset({"path", "version", "sha256"}), "cli")
    path = _bounded_string(item["path"], "cli.path", 1, 4096)
    if not Path(path).is_absolute() or "\0" in path:
        raise ValidationError("cli.path must be an absolute non-NUL path")
    return {
        "path": path,
        "version": _bounded_string(item["version"], "cli.version", 1, 256),
        "sha256": _digest(item["sha256"], "cli.sha256"),
    }


def _reject_forbidden_card_text(value: str, label: str) -> str:
    lowered = value.lower()
    for marker in FORBIDDEN_CARD_TEXT_MARKERS:
        if marker in lowered:
            raise ValidationError(f"{label} contains forbidden source or formalization bytes")
    return value


def _safe_relative_runtime_path(value: str, label: str) -> str:
    text = _bounded_string(value, label, 1, 4096)
    path = Path(text)
    if (
        path.is_absolute()
        or text in {".", ".."}
        or any(part in {"", ".", ".."} for part in path.parts)
    ):
        raise ValidationError(f"{label} must be a normalized relative path")
    return text


def _require_fields(value: Mapping[str, Any], expected: frozenset[str], label: str) -> None:
    if not isinstance(value, Mapping):
        raise ValidationError(f"{label} must be an object")
    if set(value) != set(expected):
        raise ValidationError(
            f"{label} fields must be {sorted(expected)}, got {sorted(value)}"
        )


def _mapping(value: Any, label: str) -> Mapping[str, Any]:
    if not isinstance(value, dict):
        raise ValidationError(f"{label} must be an object")
    return value


def _require_equal(value: Any, expected: Any, label: str) -> None:
    if value != expected:
        raise ValidationError(f"{label} must be {expected!r}")


def _enum(value: Any, allowed: frozenset[str], label: str) -> str:
    if not isinstance(value, str) or value not in allowed:
        raise ValidationError(f"{label} must be one of {sorted(allowed)}")
    return value


def _bounded_string(value: Any, label: str, minimum: int, maximum: int) -> str:
    if (
        not isinstance(value, str)
        or "\0" in value
        or len(value) < minimum
        or len(value) > maximum
    ):
        raise ValidationError(f"{label} must be a non-NUL string of length {minimum}..{maximum}")
    return value


def _runtime_id(value: Any, label: str) -> str:
    text = _bounded_string(value, label, 1, 128)
    allowed = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789._-"
    if text in {".", ".."} or text.startswith("-") or any(char not in allowed for char in text):
        raise ValidationError(f"{label} is not a safe runtime identifier")
    return text


def _digest(value: Any, label: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(char not in "0123456789abcdef" for char in value)
    ):
        raise ValidationError(f"{label} must be a SHA-256 hex digest")
    return value


def _bounded_integer(value: Any, label: str, minimum: int, maximum: int) -> int:
    if not isinstance(value, int) or isinstance(value, bool) or not minimum <= value <= maximum:
        raise ValidationError(f"{label} must be an integer in {minimum}..{maximum}")
    return value


def _string_list(value: Any, label: str, minimum: int, maximum: int) -> list[str]:
    if not isinstance(value, list) or not minimum <= len(value) <= maximum:
        raise ValidationError(f"{label} must be a list with {minimum}..{maximum} entries")
    return [_bounded_string(item, f"{label} item", 1, 4096) for item in value]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Prepare, run, and check Arm G guided diagnostics.")
    subcommands = parser.add_subparsers(dest="command", required=True)

    prepare = subcommands.add_parser("prepare")
    prepare.add_argument("--run-dir", required=True, type=Path)
    prepare.add_argument("--theorem", required=True, type=Path)
    prepare.add_argument("--mechanism-card", required=True, type=Path)
    prepare.add_argument("--cli", required=True, type=Path)
    prepare.add_argument("--cli-version", required=True)
    prepare.add_argument("--cli-sha256", required=True)
    prepare.add_argument("--model", required=True)
    prepare.add_argument("--h-outcome", required=True)
    prepare.add_argument("--e-outcome", required=True)
    prepare.add_argument("--resource-preflight", choices=("pass", "fail"), required=True)

    run = subcommands.add_parser("run")
    run.add_argument("--run-dir", required=True, type=Path)

    check = subcommands.add_parser("check")
    check.add_argument("--run-dir", required=True, type=Path)

    args = parser.parse_args(argv)
    if args.command == "prepare":
        result = prepare_guided_run(
            args.run_dir,
            theorem_path=args.theorem,
            mechanism_card_path=args.mechanism_card,
            cli_identity={
                "path": str(args.cli.resolve()),
                "version": args.cli_version,
                "sha256": args.cli_sha256,
            },
            model=args.model,
            h_correctness_outcome=args.h_outcome,
            e_correctness_outcome=args.e_outcome,
            resource_preflight_passed=args.resource_preflight == "pass",
        )
    elif args.command == "run":
        result = run_guided_reconstruction(args.run_dir)
    else:
        result = check_guided_run(args.run_dir)
    print(json.dumps(result, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
