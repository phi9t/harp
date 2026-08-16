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

    context_value = dict(context)
    if ticket_value["context_sha256"] != _canonical_sha256(context_value):
        raise ValidationError("ticket context_sha256 does not match provider context")
    context_value = _validate_context(
        context,
        ticket=ticket_value,
        role=role,
        schema_sha256=sha256_bytes(schema_bytes),
        prompt_sha256=sha256_bytes(prompt.encode("utf-8")),
    )
    provider_context = _provider_visible_context(role, context_value)
    wrapped_prompt = (
        "FRONTIER_CONTEXT_JSON: "
        + json.dumps(provider_context, sort_keys=True, separators=(",", ":"))
        + "\n\n"
        + prompt
    )
    validated_output: dict[str, object] | None = None

    def finalize_receipt(
        receipt: dict[str, Any], final_value: dict[str, Any] | None
    ) -> None:
        nonlocal validated_output
        if _is_blocked_resource_value(receipt, final_value):
            receipt["status"] = "blocked_resource"
            receipt["blocked_reason"] = "resource block reported by provider"
            return
        validated_output = _validate_output(role, receipt, final_value, context_value)

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
        finalize_receipt=finalize_receipt,
    )
    result["validated_output"] = validated_output
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


def _validate_context(
    context: Mapping[str, Any],
    *,
    ticket: Mapping[str, Any],
    role: str,
    schema_sha256: str,
    prompt_sha256: str,
) -> dict[str, Any]:
    value = dict(context)
    if role == "expert":
        _validate_expert_context(value, ticket)
    elif role == "proof_progress_evaluator":
        _validate_evaluator_context(value)
    else:
        raise ValidationError(f"unknown frontier provider role {role}")
    _validate_common_context(value, ticket, schema_sha256, prompt_sha256)
    return value


def _validate_common_context(
    value: Mapping[str, Any],
    ticket: Mapping[str, Any],
    schema_sha256: str,
    prompt_sha256: str,
) -> None:
    if value.get("delegation_allowed") is not False:
        raise ValidationError("frontier context must set delegation_allowed=false")
    if value.get("ticket_id") != ticket["ticket_id"]:
        raise ValidationError("frontier context ticket_id must match ticket")
    if value.get("allowed_tools") != FRONTIER_ALLOWED_TOOLS:
        raise ValidationError("frontier context allowed_tools must be exactly Write")
    for tool in value.get("allowed_tools", []):
        if str(tool) in FORBIDDEN_CONTEXT_TOOLS:
            raise ValidationError(f"forbidden frontier tool exposed: {tool}")
    for tool in value.get("forbidden_tools", []):
        if str(tool) not in FORBIDDEN_CONTEXT_TOOLS:
            raise ValidationError(f"unknown forbidden frontier tool: {tool}")
    required_sources = {"search", "network", "mcp", "shell", "read", "delegation"}
    observed_sources = {str(item).lower() for item in value.get("forbidden_sources", [])}
    missing_sources = required_sources - observed_sources
    if missing_sources:
        raise ValidationError(
            "frontier context forbidden_sources must include "
            + ", ".join(sorted(missing_sources))
        )
    if value.get("schema_sha256") != schema_sha256:
        raise ValidationError("frontier context schema_sha256 does not match schema")
    if value.get("prompt_sha256") != prompt_sha256:
        raise ValidationError("frontier context prompt_sha256 does not match prompt")
    if value.get("parent_artifact_sha256") != ticket["parent_artifact_sha256"]:
        raise ValidationError("frontier context parent_artifact_sha256 must match ticket")


def _validate_expert_context(
    value: Mapping[str, Any], ticket: Mapping[str, Any]
) -> None:
    expected = {
        "schema_version",
        "run_id",
        "attempt_id",
        "ticket_id",
        "proposed_node_id",
        "parent",
        "generation",
        "expert_role",
        "selected_direction",
        "theorem_text",
        "theorem_sha256",
        "forbidden_sources",
        "forbidden_tools",
        "allowed_tools",
        "delegation_allowed",
        "limits",
        "functioning_criteria",
        "completion_criteria",
        "result_schema",
        "schema_sha256",
        "prompt_sha256",
        "parent_artifact_sha256",
        "allowed_parent_artifacts",
    }
    _require_fields(value, expected, "expert context")
    if value["schema_version"] != expert_contracts.SCHEMA_VERSION_EXPERT_CONTEXT:
        raise ValidationError("expert context schema_version is invalid")
    if value["result_schema"] != "expert_result":
        raise ValidationError("expert context result_schema must be expert_result")
    theorem = value["theorem_text"]
    if not isinstance(theorem, str):
        raise ValidationError("expert context theorem_text must be a string")
    if sha256_bytes(theorem.encode("utf-8")) != value["theorem_sha256"]:
        raise ValidationError("expert context theorem_sha256 does not match theorem_text")
    _validate_expert_lineage(value, ticket)


def _validate_evaluator_context(value: Mapping[str, Any]) -> None:
    expected = {
        "schema_version",
        "run_id",
        "evaluation_id",
        "evaluator_index",
        "ticket_id",
        "node_artifact_sha256",
        "mathematical_payload_sha256",
        "theorem_text",
        "theorem_sha256",
        "mathematical_payload",
        "probe_ids",
        "forbidden_sources",
        "forbidden_tools",
        "allowed_tools",
        "delegation_allowed",
        "completion_criteria",
        "result_schema",
        "schema_sha256",
        "prompt_sha256",
        "parent_artifact_sha256",
        "allowed_parent_artifacts",
    }
    _require_fields(value, expected, "evaluator context")
    if value["schema_version"] != expert_contracts.SCHEMA_VERSION_EVALUATOR_CONTEXT:
        raise ValidationError("evaluator context schema_version is invalid")
    if value["result_schema"] != "node_evaluation_payload":
        raise ValidationError(
            "evaluator context result_schema must be node_evaluation_payload"
        )
    theorem = value["theorem_text"]
    if not isinstance(theorem, str):
        raise ValidationError("evaluator context theorem_text must be a string")
    if sha256_bytes(theorem.encode("utf-8")) != value["theorem_sha256"]:
        raise ValidationError(
            "evaluator context theorem_sha256 does not match theorem_text"
        )
    if tuple(value["probe_ids"]) != expert_contracts.PROOF_PROGRESS_PROBE_IDS:
        raise ValidationError("evaluator context probe_ids must match closed probe set")
    if value["node_artifact_sha256"] != value["parent_artifact_sha256"]:
        raise ValidationError("evaluator context node artifact must match parent artifact")


def _validate_expert_lineage(value: Mapping[str, Any], ticket: Mapping[str, Any]) -> None:
    generation = value["generation"]
    if not isinstance(generation, int) or generation < 0:
        raise ValidationError("expert context generation must be a nonnegative integer")
    direction = _validate_selected_direction_record(value["selected_direction"])
    if direction["recommended_role"] != value["expert_role"]:
        raise ValidationError("expert context role must match selected direction")
    parent = value["parent"]
    if generation == 0:
        expert_contracts.build_expert_context(
            run_id=str(value["run_id"]),
            attempt_id=str(value["attempt_id"]),
            ticket_id=str(value["ticket_id"]),
            proposed_node_id=str(value["proposed_node_id"]),
            parent_node=None,
            generation=0,
            expert_role=str(value["expert_role"]),
            selected_direction=direction,
            theorem_text=str(value["theorem_text"]),
            forbidden_sources=list(value["forbidden_sources"]),
        )
        if parent is not None:
            raise ValidationError("root expert context cannot include a parent")
        if value["parent_artifact_sha256"] is not None:
            raise ValidationError("root expert context cannot bind parent artifact")
        if value["allowed_parent_artifacts"] != []:
            raise ValidationError("root expert context cannot allow parent artifacts")
        return
    if not isinstance(parent, dict):
        raise ValidationError("child expert context requires exactly one parent")
    _require_fields(
        parent,
        {"node_id", "node_artifact_sha256", "mathematical_payload"},
        "expert parent projection",
    )
    if parent["node_id"] != ticket["parent_node_id"]:
        raise ValidationError("child expert parent node_id must match ticket")
    if parent["node_artifact_sha256"] != ticket["parent_artifact_sha256"]:
        raise ValidationError("child expert parent artifact must match ticket")
    if value["parent_artifact_sha256"] != parent["node_artifact_sha256"]:
        raise ValidationError("child expert context parent artifact mismatch")
    if value["allowed_parent_artifacts"] != [parent["node_artifact_sha256"]]:
        raise ValidationError("child expert context must allow exactly one parent artifact")
    if direction["source_parent_node_id"] != parent["node_id"]:
        raise ValidationError("child direction parent node must match context parent")
    if direction["source_node_artifact_sha256"] != parent["node_artifact_sha256"]:
        raise ValidationError("child direction parent artifact must match context parent")


def _validate_selected_direction_record(value: Any) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValidationError("selected direction must be an object")
    expected = {
        "direction_id",
        "kind",
        "statement",
        "strength",
        "recommended_role",
        "source_parent_node_id",
        "source_node_artifact_sha256",
        "source_reconciliation_sha256",
    }
    _require_fields(value, expected, "selected direction")
    if value["recommended_role"] not in expert_contracts.EXPERT_ROLES:
        raise ValidationError("selected direction recommended_role is invalid")
    if value["kind"] == "root_task":
        if value["strength"] != "root":
            raise ValidationError("root direction strength must be root")
        if value["source_parent_node_id"] is not None:
            raise ValidationError("root direction cannot reference a parent node")
        if value["source_node_artifact_sha256"] is not None:
            raise ValidationError("root direction cannot reference a node artifact")
        if value["source_reconciliation_sha256"] is not None:
            raise ValidationError("root direction cannot reference reconciliation")
    elif value["kind"] in {"obligation", "evaluator_finding", "proposed_direction"}:
        if value["source_parent_node_id"] is None:
            raise ValidationError("child direction requires source parent node")
        if value["source_node_artifact_sha256"] is None:
            raise ValidationError("child direction requires source node artifact")
        if value["kind"] == "evaluator_finding" and value["source_reconciliation_sha256"] is None:
            raise ValidationError("evaluator finding direction requires reconciliation")
    else:
        raise ValidationError("selected direction kind is invalid")
    return dict(value)


def _provider_visible_context(role: str, context: Mapping[str, Any]) -> dict[str, Any]:
    if role == "proof_progress_evaluator":
        return {
            "schema_version": "crouzeix-evaluator-provider-context/v1",
            "theorem_text": context["theorem_text"],
            "mathematical_payload": context["mathematical_payload"],
            "probe_ids": context["probe_ids"],
        }
    return dict(context)


def _parent_digests(ticket: Mapping[str, Any]) -> dict[str, str]:
    parent = ticket["parent_artifact_sha256"]
    if parent is None:
        return {}
    return {"parent_artifact_sha256": str(parent)}


def _is_blocked_resource_value(
    receipt: Mapping[str, Any], final: Mapping[str, Any] | None
) -> bool:
    return (
        isinstance(final, dict)
        and final.get("schema_version") == "frontier-resource-block/v1"
        and receipt.get("status") == "failed"
    )


def _validate_output(
    role: str,
    receipt: dict[str, Any],
    final: Mapping[str, Any] | None,
    context: Mapping[str, Any],
) -> dict[str, object] | None:
    receipt.setdefault("blocked_reason", None)
    if receipt["status"] != "completed":
        return None
    if final is None:
        return None
    try:
        if role == "expert":
            value = expert_contracts.validate_expert_result(final)
            _validate_expert_output_binding(value, context)
            return value
        if role == "proof_progress_evaluator":
            value = expert_contracts.validate_evaluator_result(final)
            _validate_evaluator_output_binding(value, context)
            return value
    except ValidationError as error:
        receipt["status"] = "malformed"
        receipt["parse_error"] = str(error)
        receipt["blocked_reason"] = str(error)
        return None
    raise ValidationError(f"unknown frontier provider role {role}")


def _validate_expert_output_binding(
    value: Mapping[str, object], context: Mapping[str, Any]
) -> None:
    for field in (
        "run_id",
        "attempt_id",
        "ticket_id",
        "proposed_node_id",
        "generation",
        "expert_role",
    ):
        if value[field] != context[field]:
            raise ValidationError(f"expert output {field} does not match context")
    direction = context["selected_direction"]
    if value["selected_direction_id"] != direction["direction_id"]:
        raise ValidationError("expert output selected_direction_id does not match context")
    parent = context["parent"]
    expected_parent_id = None if parent is None else parent["node_id"]
    expected_parent_artifact = None if parent is None else parent["node_artifact_sha256"]
    if value["parent_node_id"] != expected_parent_id:
        raise ValidationError("expert output parent_node_id does not match context")
    if value["parent_node_artifact_sha256"] != expected_parent_artifact:
        raise ValidationError(
            "expert output parent_node_artifact_sha256 does not match context"
        )


def _validate_evaluator_output_binding(
    value: Mapping[str, object], context: Mapping[str, Any]
) -> None:
    if tuple(probe["probe_id"] for probe in value["probes"]) != tuple(
        context["probe_ids"]
    ):
        raise ValidationError("evaluator output probe IDs do not match context")


def _canonical_sha256(value: Mapping[str, Any]) -> str:
    data = json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")
    return sha256_bytes(data)


def _require_fields(value: Mapping[str, Any], expected: set[str], label: str) -> None:
    actual = set(value)
    if actual != expected:
        raise ValidationError(
            f"{label} fields are invalid: expected {sorted(expected)}, got {sorted(actual)}"
        )
