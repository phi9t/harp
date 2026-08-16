from __future__ import annotations

import json
from pathlib import Path
from typing import Any, Callable, Mapping

import expert_contracts
import runner
import tickets
from protocol import ValidationError, sha256_bytes


FRONTIER_ALLOWED_TOOLS = ["Write"]
FRONTIER_ROLES = frozenset({"expert", "proof_progress_evaluator"})
FORBIDDEN_CONTEXT_TOOLS = frozenset(
    {
        "Read",
        "Glob",
        "Grep",
        "Bash",
        "Edit",
        "spawn_agent",
        "WebSearch",
        "MCP",
        "Shell",
    }
)


def run_frontier_call(
    run_dir: Path,
    *,
    ticket: Mapping[str, Any],
    call_id: str,
    role: str,
    prompt: str,
    context: Mapping[str, Any],
    schema_path: Path,
    accounting_path: Path,
    after_receipt: Callable[[], None] | None = None,
) -> dict[str, Any]:
    if role not in FRONTIER_ROLES:
        raise ValidationError(f"unknown frontier provider role {role}")
    ticket_value = tickets.validate_runtime_ticket(ticket)
    _validate_ticket_scope(ticket_value, role)
    schema_bytes = schema_path.read_bytes()
    base_binding = {
        "ticket_id": str(ticket_value["ticket_id"]),
        "ticket_sha256": tickets.canonical_sha256(ticket_value),
    }
    tickets.validate_ticket_binding(base_binding, ticket_value)
    binding = {
        **base_binding,
        "ticket_context_sha256": str(ticket_value["context_sha256"]),
        "ticket_schema_sha256": str(ticket_value["schema_sha256"]),
        "ticket_prompt_sha256": str(ticket_value["prompt_sha256"]),
        "ticket_parent_artifact_sha256": ticket_value["parent_artifact_sha256"],
    }
    _validate_pinned_digest(ticket_value, "schema_sha256", sha256_bytes(schema_bytes))
    _validate_pinned_digest(
        ticket_value, "prompt_sha256", sha256_bytes(prompt.encode("utf-8"))
    )

    context_value = _build_context_envelope(
        context,
        ticket=ticket_value,
        binding=binding,
        schema_sha256=sha256_bytes(schema_bytes),
        prompt_sha256=sha256_bytes(prompt.encode("utf-8")),
    )
    wrapped_prompt = (
        "FRONTIER_CONTEXT_JSON: "
        + json.dumps(context_value, sort_keys=True, separators=(",", ":"))
        + "\n\n"
        + prompt
    )
    result = runner.run_call(
        run_dir,
        _frontier_spec(runner.read_run_spec(run_dir / "run_spec.json")),
        call_id=call_id,
        role=_runner_role(role),
        prompt=wrapped_prompt,
        schema_path=schema_path,
        parent_digests=_parent_digests(ticket_value),
        ticket_binding=binding,
        allowed_tools=list(FRONTIER_ALLOWED_TOOLS),
    )
    if _is_blocked_resource(result):
        _rewrite_status(
            result,
            "blocked_resource",
            "resource block reported by provider",
        )
    result["validated_output"] = _validate_output(role, result)
    runner._append_accounting_receipt(accounting_path, result["receipt"])
    if result["receipt"]["status"] == "completed" and after_receipt is not None:
        after_receipt()
    return result


def _frontier_spec(spec: Mapping[str, Any]) -> dict[str, Any]:
    value = dict(spec)
    if value.get("network_access") is not False:
        raise ValidationError("frontier provider requires network_access=false")
    if value.get("approval_policy") != "never":
        raise ValidationError("frontier provider requires approval_policy=never")
    value["allowed_tools"] = list(FRONTIER_ALLOWED_TOOLS)
    return value


def _runner_role(role: str) -> str:
    if role == "expert":
        return "expert"
    if role == "proof_progress_evaluator":
        return "proof_progress_evaluator"
    raise ValidationError(f"unknown frontier provider role {role}")


def _validate_ticket_scope(ticket: Mapping[str, Any], role: str) -> None:
    if list(ticket["allowed_tools"]) != FRONTIER_ALLOWED_TOOLS:
        raise ValidationError("frontier tickets must allow exactly Write")
    if role == "expert" and ticket["owner_type"] != "expert":
        raise ValidationError("expert call requires an expert ticket")
    if role == "proof_progress_evaluator" and ticket["owner_type"] != "evaluator":
        raise ValidationError("evaluator call requires an evaluator ticket")
    forbidden_sources = set(str(item) for item in ticket["forbidden_sources"])
    required_sources = {
        "search",
        "network",
        "mcp",
        "shell",
        "read",
        "delegation",
    }
    missing = required_sources - {item.lower() for item in forbidden_sources}
    if missing:
        raise ValidationError(
            "frontier ticket forbidden_sources must include "
            + ", ".join(sorted(missing))
        )


def _validate_pinned_digest(
    ticket: Mapping[str, Any], field: str, observed_sha256: str
) -> None:
    if ticket[field] != observed_sha256:
        raise ValidationError(f"ticket {field} does not match call input")


def _build_context_envelope(
    context: Mapping[str, Any],
    *,
    ticket: Mapping[str, Any],
    binding: Mapping[str, str],
    schema_sha256: str,
    prompt_sha256: str,
) -> dict[str, Any]:
    value = dict(context)
    if value.get("delegation_allowed") is not False:
        raise ValidationError("frontier context must set delegation_allowed=false")
    if value.get("ticket_id") != ticket["ticket_id"]:
        raise ValidationError("frontier context ticket_id must match ticket")
    supplied_tools = value.get("allowed_tools")
    if supplied_tools is not None and supplied_tools != FRONTIER_ALLOWED_TOOLS:
        raise ValidationError("frontier context allowed_tools must be exactly Write")
    for tool in supplied_tools or []:
        if str(tool) in FORBIDDEN_CONTEXT_TOOLS:
            raise ValidationError(f"forbidden frontier tool exposed: {tool}")
    value.update(
        {
            "allowed_tools": list(FRONTIER_ALLOWED_TOOLS),
            "ticket_sha256": binding["ticket_sha256"],
            "schema_sha256": schema_sha256,
            "prompt_sha256": prompt_sha256,
            "parent_artifact_sha256": ticket["parent_artifact_sha256"],
        }
    )
    return value


def _parent_digests(ticket: Mapping[str, Any]) -> dict[str, str]:
    parent = ticket["parent_artifact_sha256"]
    if parent is None:
        return {}
    return {"parent_artifact_sha256": str(parent)}


def _is_blocked_resource(result: Mapping[str, Any]) -> bool:
    final = result.get("final")
    receipt = result.get("receipt")
    return (
        isinstance(final, dict)
        and final.get("schema_version") == "frontier-resource-block/v1"
        and isinstance(receipt, dict)
        and receipt.get("status") == "failed"
    )


def _rewrite_status(
    result: Mapping[str, Any],
    status: str,
    blocked_reason: str,
) -> None:
    receipt = result["receipt"]
    receipt["status"] = status
    receipt["blocked_reason"] = blocked_reason
    receipt_path = Path(result["call_dir"]) / "receipt.json"
    receipt_path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")


def _validate_output(role: str, result: Mapping[str, Any]) -> dict[str, object] | None:
    receipt = result["receipt"]
    receipt.setdefault("blocked_reason", None)
    if receipt["status"] != "completed":
        return None
    final = result["final"]
    if final is None:
        return None
    try:
        if role == "expert":
            return expert_contracts.validate_expert_result(final)
        if role == "proof_progress_evaluator":
            return expert_contracts.validate_evaluator_result(final)
    except ValidationError as error:
        result["receipt"]["parse_error"] = str(error)
        _rewrite_status(
            result,
            "malformed",
            str(error),
        )
        return None
    raise ValidationError(f"unknown frontier provider role {role}")
