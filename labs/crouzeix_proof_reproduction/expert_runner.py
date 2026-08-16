from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import stat
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Mapping, Protocol

import expert_contracts
import frontier
import frontier_provider
import frontier_store
import prepare_frontier
import protocol
import tickets


PHASES = ("roots", "evaluate-roots", "generations", "finalize", "check")
PHASE_PREDECESSOR = {
    "roots": None,
    "evaluate-roots": "roots",
    "generations": "evaluate-roots",
    "finalize": "generations",
    "check": "finalize",
}
FORBIDDEN_TOOLS = [
    "Read",
    "Glob",
    "Grep",
    "Bash",
    "Edit",
    "spawn_agent",
    "WebSearch",
    "MCP",
    "Shell",
]
FORBIDDEN_SOURCES = [
    "search",
    "network",
    "mcp",
    "shell",
    "read",
    "delegation",
    "public proof manuscripts",
]
EVALUATOR_PROMPT = "proof_progress_evaluator.md"
EXPERT_PROMPT = "expert.md"
EXPERT_SCHEMA = "expert_result.schema.json"
EVALUATOR_SCHEMA = "node_evaluation_payload.schema.json"
PORTABLE_ID = re.compile(r"[^a-z0-9-]+")


@dataclass(frozen=True)
class ProviderAttempt:
    terminal_status: str
    provider_call: Mapping[str, Any]
    receipt: Mapping[str, Any]
    validated_output: Mapping[str, Any] | None
    reason: str | None = None


@dataclass(frozen=True)
class RunnerLimits:
    max_provider_calls: int = 33
    admitted_node_budget: int = 11
    child_generations: int = 3
    draws_per_generation: int = 2


class FrontierProvider(Protocol):
    def run_expert(
        self,
        *,
        run_dir: Path,
        ticket: dict[str, object],
        context: dict[str, object],
    ) -> ProviderAttempt:
        ...

    def run_evaluator(
        self,
        *,
        run_dir: Path,
        ticket: dict[str, object],
        context: dict[str, object],
    ) -> ProviderAttempt:
        ...


class TraeCliFrontierProvider:
    def run_expert(
        self,
        *,
        run_dir: Path,
        ticket: dict[str, object],
        context: dict[str, object],
    ) -> ProviderAttempt:
        result = frontier_provider.run_frontier_call(
            run_dir,
            ticket=ticket,
            call_id=str(ticket["ticket_id"]),
            role="expert",
            prompt=_read_text(run_dir / "prompts" / EXPERT_PROMPT, "expert prompt"),
            context=context,
            schema_path=run_dir / "schemas" / EXPERT_SCHEMA,
            accounting_path=run_dir / "calls" / "frontier_provider_accounting.jsonl",
        )
        return ProviderAttempt(
            terminal_status=str(result["receipt"]["status"]),
            provider_call=_provider_call_record(result),
            receipt=result["receipt"],
            validated_output=result.get("validated_output"),
            reason=result["receipt"].get("blocked_reason") or result["receipt"].get("parse_error"),
        )

    def run_evaluator(
        self,
        *,
        run_dir: Path,
        ticket: dict[str, object],
        context: dict[str, object],
    ) -> ProviderAttempt:
        result = frontier_provider.run_frontier_call(
            run_dir,
            ticket=ticket,
            call_id=str(ticket["ticket_id"]),
            role="proof_progress_evaluator",
            prompt=_read_text(run_dir / "prompts" / EVALUATOR_PROMPT, "evaluator prompt"),
            context=context,
            schema_path=run_dir / "schemas" / EVALUATOR_SCHEMA,
            accounting_path=run_dir / "calls" / "frontier_provider_accounting.jsonl",
        )
        return ProviderAttempt(
            terminal_status=str(result["receipt"]["status"]),
            provider_call=_provider_call_record(result),
            receipt=result["receipt"],
            validated_output=result.get("validated_output"),
            reason=result["receipt"].get("blocked_reason") or result["receipt"].get("parse_error"),
        )


def run_phase(
    run_dir: Path,
    phase: str,
    *,
    provider: FrontierProvider | None = None,
    limits: RunnerLimits | None = None,
) -> dict[str, object]:
    root = Path(run_dir).resolve()
    if phase not in PHASES:
        raise protocol.ValidationError(f"unknown frontier phase {phase}")
    spec = protocol.read_run_spec(root / "run_spec.json")
    runner_limits = limits or _limits_from_spec(spec)
    engine = provider or TraeCliFrontierProvider()
    if phase != "check":
        _enforce_phase_boundary(root, phase)

    if phase == "roots":
        result = _run_roots(root, spec, engine, runner_limits)
        _mark_phase(root, phase, result)
        return result
    if phase == "evaluate-roots":
        result = _evaluate_roots(root, spec, engine, runner_limits)
        _mark_phase(root, phase, result)
        return result
    if phase == "generations":
        result = _run_generations(root, spec, engine, runner_limits)
        _mark_phase(root, phase, result)
        return result
    if phase == "finalize":
        result = _finalize(root, spec)
        _mark_phase(root, phase, result)
        return result
    return _check(root)


def build_expert_runtime_context(
    *,
    run_dir: Path,
    attempt_id: str,
    ticket_id: str,
    proposed_node_id: str,
    parent_node: Mapping[str, Any] | None,
    generation: int,
    expert_role: str,
    selected_direction: Mapping[str, Any],
) -> dict[str, object]:
    root = Path(run_dir)
    theorem_text = _read_text(root / "inputs" / "theorem.txt", "theorem text")
    context = expert_contracts.build_expert_context(
        run_id=root.name,
        attempt_id=attempt_id,
        ticket_id=ticket_id,
        proposed_node_id=proposed_node_id,
        parent_node=parent_node,
        generation=generation,
        expert_role=expert_role,
        selected_direction=selected_direction,
        theorem_text=theorem_text,
        forbidden_sources=list(_excluded_sources(root)),
    )
    context["forbidden_sources"] = _forbidden_sources(root)
    context["forbidden_tools"] = list(FORBIDDEN_TOOLS)
    context["schema_sha256"] = _file_sha256(root / "schemas" / EXPERT_SCHEMA)
    context["prompt_sha256"] = _file_sha256(root / "prompts" / EXPERT_PROMPT)
    parent_digest = None if parent_node is None else str(parent_node["node_artifact_sha256"])
    context["parent_artifact_sha256"] = parent_digest
    context["allowed_parent_artifacts"] = [] if parent_digest is None else [parent_digest]
    return context


def runtime_ticket(
    *,
    run_dir: Path,
    ticket_id: str,
    task_kind: str,
    node_id: str | None,
    parent_node_id: str | None,
    generation: int,
    direction_id: str | None,
    role: str,
    owner_type: str,
    context: Mapping[str, Any],
    prompt_name: str,
    schema_name: str,
    parent_artifact_sha256: str | None,
    dependency_ticket_ids: list[str],
    objective: str,
    expected_deliverable: str,
) -> dict[str, object]:
    root = Path(run_dir)
    ticket = {
        "schema_version": "crouzeix-runtime-ticket/v1",
        "ticket_id": ticket_id,
        "run_id": root.name,
        "task_kind": task_kind,
        "node_id": node_id,
        "parent_node_id": parent_node_id,
        "generation": generation,
        "direction_id": direction_id,
        "role": role,
        "objective": objective,
        "expected_deliverable": expected_deliverable,
        "dependency_ticket_ids": list(dependency_ticket_ids),
        "context_sha256": _canonical_sha256(context),
        "schema_sha256": _file_sha256(root / "schemas" / schema_name),
        "prompt_sha256": _file_sha256(root / "prompts" / prompt_name),
        "parent_artifact_sha256": parent_artifact_sha256,
        "allowed_tools": ["Write"],
        "forbidden_sources": _forbidden_sources(root),
        "timeout_seconds": 3600,
        "max_output_bytes": 1048576,
        "owner_type": owner_type,
        "created_at_utc": _now(),
        "state": "created",
    }
    return tickets.validate_runtime_ticket(ticket)


def _run_roots(
    run_dir: Path,
    spec: Mapping[str, Any],
    provider: FrontierProvider,
    limits: RunnerLimits,
) -> dict[str, object]:
    budget_exhausted = False
    for role in spec["frontier"]["expert_roles"]:
        if _provider_call_count(run_dir) >= limits.max_provider_calls:
            budget_exhausted = True
            break
        ticket_id = prepare_frontier.root_ticket_id(str(role))
        if _attempt_exists(run_dir, ticket_id):
            continue
        context = _read_json(run_dir / "contexts" / f"{ticket_id}.json", "root context")
        ticket = _read_ticket(run_dir, ticket_id)
        _run_expert_attempt(run_dir, provider, ticket, context)
    receipt = _reconcile_run(run_dir)
    return {
        "phase": "roots",
        "root_attempt_count": _root_attempt_count(run_dir),
        "root_completed": _root_completed_count(run_dir),
        "budget_exhausted": budget_exhausted,
        "run_receipt_sha256": receipt["run_receipt_sha256"],
    }


def _evaluate_roots(
    run_dir: Path,
    spec: Mapping[str, Any],
    provider: FrontierProvider,
    limits: RunnerLimits,
) -> dict[str, object]:
    budget_exhausted = False
    for node in _nodes(run_dir):
        if int(node["generation"]) != 0 or _archive_entry_exists(run_dir, str(node["node_id"])):
            continue
        if _provider_call_count(run_dir) >= limits.max_provider_calls:
            budget_exhausted = True
            break
        if _node_count(run_dir) > limits.admitted_node_budget:
            budget_exhausted = True
            break
        if not _evaluate_and_archive_node(run_dir, provider, node, limits):
            budget_exhausted = True
            break
    receipt = _reconcile_run(run_dir)
    return {
        "phase": "evaluate-roots",
        "archive_entry_count": len(_archive_entries(run_dir)),
        "candidate_projection_count": _candidate_projection_count(run_dir),
        "budget_exhausted": budget_exhausted,
        "run_receipt_sha256": receipt["run_receipt_sha256"],
    }


def _run_generations(
    run_dir: Path,
    spec: Mapping[str, Any],
    provider: FrontierProvider,
    limits: RunnerLimits,
) -> dict[str, object]:
    budget_exhausted = False
    seed = int(spec["frontier"]["selection_seed"])
    for generation in range(1, limits.child_generations + 1):
        if _provider_call_count(run_dir) >= limits.max_provider_calls:
            budget_exhausted = True
            break
        entries = _archive_entries(run_dir)
        nodes = _nodes(run_dir)
        eligible = [entry for entry in entries if int(entry["unanimous_pass_count"]) < 10]
        if not eligible:
            break
        _publish_selection_tickets(
            run_dir,
            generation,
            seed,
            entries,
            nodes,
            draw_count=limits.draws_per_generation,
        )
        try:
            selection = frontier.select_generation(
                entries,
                nodes,
                seed=seed,
                generation=generation,
            )
        except protocol.ValidationError:
            break
        used_by_node: dict[str, set[str]] = {}
        selected_events = list(selection["selection_events"])
        for raw_event in selected_events:
            draw_index = int(raw_event["draw_index"])
            selection_ticket_id = f"select-g{generation}-d{draw_index}"
            _complete_selection_ticket(run_dir, selection_ticket_id, raw_event)
            _append_selection_ledger(run_dir, raw_event, selection_ticket_id)
        for raw_event in selected_events:
            if _provider_call_count(run_dir) >= limits.max_provider_calls:
                budget_exhausted = True
                break
            if _node_count(run_dir) >= limits.admitted_node_budget:
                budget_exhausted = True
                break
            draw_index = int(raw_event["draw_index"])
            selection_ticket_id = f"select-g{generation}-d{draw_index}"
            parent = _node_by_id(run_dir, str(raw_event["selected_node_id"]))
            reconciliation = _reconciliation_for_node(run_dir, str(parent["node_id"]))
            used = used_by_node.setdefault(str(parent["node_id"]), set())
            direction = _select_child_direction(parent, reconciliation, used)
            used.add(str(direction["direction_id"]))
            child_ticket, child_context = _publish_child_expert_ticket(
                run_dir,
                generation=generation,
                draw_index=draw_index,
                parent=parent,
                direction=direction,
                dependency_ticket_id=selection_ticket_id,
            )
            before_nodes = {str(node["node_id"]) for node in _nodes(run_dir)}
            _run_expert_attempt(run_dir, provider, child_ticket, child_context)
            after_nodes = {
                str(node["node_id"]): node
                for node in _nodes(run_dir)
                if str(node["node_id"]) not in before_nodes
            }
            for child in after_nodes.values():
                if _provider_call_count(run_dir) >= limits.max_provider_calls:
                    budget_exhausted = True
                    break
                if not _evaluate_and_archive_node(run_dir, provider, child, limits):
                    budget_exhausted = True
                    break
            if budget_exhausted:
                break
        if budget_exhausted:
            break
    receipt = _reconcile_run(run_dir)
    return {
        "phase": "generations",
        "selection_count": len(_selection_events(run_dir)),
        "archive_entry_count": len(_archive_entries(run_dir)),
        "budget_exhausted": budget_exhausted,
        "run_receipt_sha256": receipt["run_receipt_sha256"],
    }


def _finalize(run_dir: Path, spec: Mapping[str, Any]) -> dict[str, object]:
    entries = _archive_entries(run_dir)
    strongest = _strongest_archive_entry(entries)
    finalization = {
        "schema_version": "crouzeix-frontier-finalization/v1",
        "run_id": run_dir.name,
        "candidate_projection_count": _candidate_projection_count(run_dir),
        "strongest_archive_entry_sha256": None
        if strongest is None
        else strongest["archive_entry_sha256"],
        "strongest_node_id": None if strongest is None else strongest["node_id"],
        "promoted": False,
    }
    finalization["frontier_finalization_sha256"] = _canonical_sha256(finalization)
    _write_json_create_only(
        run_dir / "candidate_projections" / "finalization.json",
        finalization,
        "frontier finalization",
    )
    receipt = _reconcile_run(run_dir)
    return {
        "phase": "finalize",
        "candidate_projection_count": finalization["candidate_projection_count"],
        "strongest_archive_entry_sha256": finalization["strongest_archive_entry_sha256"],
        "run_receipt_sha256": receipt["run_receipt_sha256"],
    }


def _check(run_dir: Path) -> dict[str, object]:
    _require_completed_phase(run_dir, "finalize")
    receipt = _reconcile_run(run_dir)
    return {
        "phase": "check",
        "run_receipt_sha256": receipt["run_receipt_sha256"],
        "ticket_count": receipt["ticket_count"],
        "archive_entry_count": receipt["archive_entry_count"],
    }


def _run_expert_attempt(
    run_dir: Path,
    provider: FrontierProvider,
    ticket: dict[str, object],
    context: dict[str, object],
) -> None:
    ticket_id = str(ticket["ticket_id"])
    states = _ticket_states(run_dir, ticket_id)
    current_state = states[-1] if states else "created"
    if current_state == "created":
        _ticket_transition(run_dir, ticket_id, "created", "admitted", "expert admitted", None)
        current_state = "admitted"
    if current_state == "admitted":
        _ticket_transition(
            run_dir,
            ticket_id,
            "admitted",
            "running",
            "expert provider call started",
            _canonical_sha256(context),
        )
        current_state = "running"
    if current_state != "running":
        raise protocol.ValidationError(f"expert ticket {ticket_id} is not resumable")
    provider_attempt = provider.run_expert(run_dir=run_dir, ticket=ticket, context=context)
    terminal_status = _attempt_status(provider_attempt.terminal_status)
    output = provider_attempt.validated_output
    receipt = dict(provider_attempt.receipt)
    provider_call = dict(provider_attempt.provider_call)
    expert_result = None
    terminal_failure = None
    terminal_ticket_state = "completed"
    try:
        if terminal_status == "completed" and output is not None:
            expert_result = expert_contracts.validate_expert_result(output)
        elif terminal_status == "completed":
            terminal_status = "malformed"
    except protocol.ValidationError as error:
        terminal_status = "malformed"
        receipt["parse_error"] = str(error)
        expert_result = None
    if terminal_status != "completed":
        terminal_ticket_state = (
            "timed_out"
            if terminal_status == "timed_out"
            else "blocked_resource"
            if terminal_status == "blocked_resource"
            else "failed"
        )
        terminal_failure = {
            "terminal_status": terminal_status,
            "reason": provider_attempt.reason or receipt.get("parse_error") or terminal_status,
        }
    receipt_sha256 = _canonical_sha256(receipt)
    _ticket_transition(
        run_dir,
        ticket_id,
        "running",
        terminal_ticket_state,
        f"expert terminal status {terminal_status}",
        receipt_sha256,
    )
    store = frontier_store.open_frontier_store(run_dir)
    attempt_dir = store.materialize_attempt(
        attempt_id=str(context["attempt_id"]),
        context=context,
        provider_call=provider_call,
        receipt=receipt,
        expert_result=expert_result,
        terminal_failure=terminal_failure,
    )
    attempt_dir_sha256 = _tree_sha256(attempt_dir)
    store.append_attempt(
        {
            "schema_version": "crouzeix-attempt-ledger/v1",
            "sequence": len(_attempts(run_dir)) + 1,
            "run_id": run_dir.name,
            "attempt_id": str(context["attempt_id"]),
            "ticket_id": ticket_id,
            "proposed_node_id": str(context["proposed_node_id"]),
            "terminal_status": terminal_status,
            "attempt_dir_sha256": attempt_dir_sha256,
            "occurred_at_utc": _now(),
        }
    )
    _append_frontier_event(
        run_dir,
        event_kind="attempt_terminal",
        ticket_id=ticket_id,
        artifact_sha256=attempt_dir_sha256,
    )
    if expert_result is None:
        return

    provenance = _expert_provenance(run_dir, context, receipt)
    decision = expert_contracts.decide_admission(
        {
            "run_id": run_dir.name,
            "attempt_id": context["attempt_id"],
            "ticket_id": ticket_id,
            "proposed_node_id": context["proposed_node_id"],
            "terminal_status": "completed",
        },
        expert_result,
        provenance,
        outcome="accepted",
    )
    store.record_admission(decision)
    _append_frontier_event(
        run_dir,
        event_kind="admission_accepted"
        if decision["outcome"] == "accepted"
        else "admission_rejected",
        ticket_id=ticket_id,
        artifact_sha256=str(decision["admission_decision_sha256"]),
    )
    if decision["outcome"] == "accepted":
        node = expert_contracts.build_mathematical_node(decision, expert_result, provenance)
        store.materialize_mathematical_node(node)
        _ticket_transition(
            run_dir,
            ticket_id,
            "completed",
            "accepted",
            "accepted expert result admitted as mathematical node",
            str(node["node_artifact_sha256"]),
        )
    else:
        _ticket_transition(
            run_dir,
            ticket_id,
            "completed",
            "rejected",
            "strict expert result rejected by admission",
            str(decision["admission_decision_sha256"]),
        )


def _evaluate_and_archive_node(
    run_dir: Path,
    provider: FrontierProvider,
    node: Mapping[str, Any],
    limits: RunnerLimits,
) -> bool:
    evaluations = []
    for evaluator_index in (1, 2):
        if _provider_call_count(run_dir) >= limits.max_provider_calls:
            return False
        ticket, context = _publish_evaluator_ticket(run_dir, node, evaluator_index)
        evaluation = _run_evaluator(run_dir, provider, ticket, context, node)
        frontier_store.open_frontier_store(run_dir).materialize_node_evaluation(evaluation)
        evaluations.append(evaluation)
    reconciliation = frontier.reconcile_node_evaluations(
        f"reconcile-{node['node_id']}",
        evaluations[0],
        evaluations[1],
    )
    entry = frontier.build_archive_entry(node, reconciliation)
    frontier_store.open_frontier_store(run_dir).materialize_archive_entry(entry, reconciliation)
    _append_frontier_event(
        run_dir,
        event_kind="archive_entry_created",
        ticket_id=str(node["source_ticket_id"]),
        artifact_sha256=str(entry["archive_entry_sha256"]),
    )
    if int(entry["unanimous_pass_count"]) == 10 and entry["candidate_proof_sha256"] is not None:
        _project_candidate(run_dir, node, reconciliation)
    return True


def _run_evaluator(
    run_dir: Path,
    provider: FrontierProvider,
    ticket: dict[str, object],
    context: dict[str, object],
    node: Mapping[str, Any],
) -> dict[str, object]:
    ticket_id = str(ticket["ticket_id"])
    _ticket_transition(run_dir, ticket_id, "created", "admitted", "evaluator admitted", None)
    _ticket_transition(
        run_dir,
        ticket_id,
        "admitted",
        "running",
        "evaluator provider call started",
        _canonical_sha256(context),
    )
    provider_context = _provider_visible_evaluator_context(context)
    provider_attempt = provider.run_evaluator(
        run_dir=run_dir,
        ticket=ticket,
        context=context,
    )
    _record_evaluator_call(run_dir, context, provider_context, provider_attempt)
    status = _attempt_status(provider_attempt.terminal_status)
    receipt = dict(provider_attempt.receipt)
    receipt_sha256 = _canonical_sha256(receipt)
    try:
        if status == "completed" and provider_attempt.validated_output is not None:
            payload = _frontier_evaluation_payload(provider_attempt.validated_output)
            terminal_event_sha256 = _ticket_transition(
                run_dir,
                ticket_id,
                "running",
                "completed",
                "evaluator completed",
                receipt_sha256,
            )
            evaluation = _build_frontier_node_evaluation(
                evaluation_id=str(context["evaluation_id"]),
                evaluator_index=int(context["evaluator_index"]),
                ticket_id=ticket_id,
                node=node,
                context_sha256=_canonical_sha256(context),
                source_kind="provider_output",
                call_receipt_sha256=receipt_sha256,
                terminal_ticket_event_sha256=terminal_event_sha256,
                evaluation_payload=payload,
            )
            _ticket_transition(
                run_dir,
                ticket_id,
                "completed",
                "accepted",
                "evaluator output accepted",
                str(evaluation["node_evaluation_sha256"]),
            )
            return evaluation
    except protocol.ValidationError as error:
        status = "malformed"
        receipt["parse_error"] = str(error)
        receipt_sha256 = _canonical_sha256(receipt)
    terminal_state = (
        "timed_out"
        if status == "timed_out"
        else "blocked_resource"
        if status == "blocked_resource"
        else "failed"
    )
    terminal_event_sha256 = _ticket_transition(
        run_dir,
        ticket_id,
        "running",
        terminal_state,
        f"evaluator terminal status {status}",
        receipt_sha256,
    )
    return _build_frontier_node_evaluation(
        evaluation_id=str(context["evaluation_id"]),
        evaluator_index=int(context["evaluator_index"]),
        ticket_id=ticket_id,
        node=node,
        context_sha256=_canonical_sha256(context),
        source_kind="conservative_fallback",
        call_receipt_sha256=receipt_sha256,
        terminal_ticket_event_sha256=terminal_event_sha256,
        evaluation_payload=_fallback_payload(),
    )


def _record_evaluator_call(
    run_dir: Path,
    context: Mapping[str, Any],
    provider_context: Mapping[str, Any],
    provider_attempt: ProviderAttempt,
) -> None:
    root = run_dir / "evaluator_calls"
    root.mkdir(exist_ok=True)
    call_dir = root / str(context["ticket_id"])
    if call_dir.exists():
        raise protocol.ValidationError("evaluator call record already exists")
    call_dir.mkdir()
    _write_json_create_only(call_dir / "context.json", context, "evaluator context")
    _write_json_create_only(
        call_dir / "provider_context.json",
        provider_context,
        "evaluator provider context",
    )
    _write_json_create_only(
        call_dir / "provider_call.json",
        dict(provider_attempt.provider_call),
        "evaluator provider call",
    )
    _write_json_create_only(
        call_dir / "receipt.json",
        dict(provider_attempt.receipt),
        "evaluator receipt",
    )


def _publish_evaluator_ticket(
    run_dir: Path,
    node: Mapping[str, Any],
    evaluator_index: int,
) -> tuple[dict[str, object], dict[str, object]]:
    node_id = str(node["node_id"])
    ticket_id = f"evaluate-{node_id}-e{evaluator_index}"
    context = _build_evaluator_runtime_context(
        run_dir=run_dir,
        node=node,
        evaluator_index=evaluator_index,
        ticket_id=ticket_id,
    )
    ticket = runtime_ticket(
        run_dir=run_dir,
        ticket_id=ticket_id,
        task_kind="evaluator",
        node_id=node_id,
        parent_node_id=None,
        generation=int(node["generation"]),
        direction_id=None,
        role="proof_progress_evaluator",
        owner_type="evaluator",
        context=context,
        prompt_name=EVALUATOR_PROMPT,
        schema_name=EVALUATOR_SCHEMA,
        parent_artifact_sha256=str(node["node_artifact_sha256"]),
        dependency_ticket_ids=[str(node["source_ticket_id"])],
        objective=f"Evaluate admitted node {node_id} with evaluator {evaluator_index}.",
        expected_deliverable="Strict node evaluation payload or conservative fallback.",
    )
    tickets.publish_ticket(run_dir / "tickets", ticket)
    return ticket, context


def _build_evaluator_runtime_context(
    *,
    run_dir: Path,
    node: Mapping[str, Any],
    evaluator_index: int,
    ticket_id: str,
) -> dict[str, object]:
    theorem_text = _read_text(run_dir / "inputs" / "theorem.txt", "theorem text")
    return {
        "schema_version": expert_contracts.SCHEMA_VERSION_EVALUATOR_CONTEXT,
        "run_id": run_dir.name,
        "evaluation_id": f"eval-{node['node_id']}-e{evaluator_index}",
        "evaluator_index": evaluator_index,
        "ticket_id": ticket_id,
        "node_id": node["node_id"],
        "node_artifact_sha256": node["node_artifact_sha256"],
        "mathematical_payload_sha256": node["mathematical_payload_sha256"],
        "theorem_text": theorem_text,
        "theorem_sha256": _sha256_text(theorem_text),
        "mathematical_payload": node["mathematical_payload"],
        "probe_ids": list(prepare_frontier.PROOF_PROGRESS_PROBE_IDS),
        "forbidden_sources": _forbidden_sources(run_dir),
        "forbidden_tools": list(FORBIDDEN_TOOLS),
        "allowed_tools": ["Write"],
        "delegation_allowed": False,
        "completion_criteria": "return one strict node_evaluation_payload",
        "result_schema": "node_evaluation_payload",
        "schema_sha256": _file_sha256(run_dir / "schemas" / EVALUATOR_SCHEMA),
        "prompt_sha256": _file_sha256(run_dir / "prompts" / EVALUATOR_PROMPT),
        "parent_artifact_sha256": node["node_artifact_sha256"],
        "allowed_parent_artifacts": [node["node_artifact_sha256"]],
    }


def _provider_visible_evaluator_context(context: Mapping[str, Any]) -> dict[str, object]:
    return {
        "schema_version": "crouzeix-evaluator-provider-context/v1",
        "theorem_text": context["theorem_text"],
        "mathematical_payload": context["mathematical_payload"],
        "probe_ids": context["probe_ids"],
    }


def _publish_selection_tickets(
    run_dir: Path,
    generation: int,
    seed: int,
    entries: list[dict[str, object]],
    nodes: list[dict[str, object]],
    *,
    draw_count: int,
) -> None:
    archive_digest = _canonical_sha256(
        {
            "entries": entries,
            "nodes": [
                {
                    "node_id": node["node_id"],
                    "node_artifact_sha256": node["node_artifact_sha256"],
                }
                for node in nodes
            ],
        }
    )
    for draw_index in range(draw_count):
        ticket_id = f"select-g{generation}-d{draw_index}"
        if (run_dir / "tickets" / ticket_id / "ticket.json").exists():
            continue
        context = {
            "schema_version": "crouzeix-selection-context/v1",
            "run_id": run_dir.name,
            "generation": generation,
            "draw_index": draw_index,
            "selection_seed": seed,
            "archive_digest": archive_digest,
        }
        ticket = runtime_ticket(
            run_dir=run_dir,
            ticket_id=ticket_id,
            task_kind="selection",
            node_id=None,
            parent_node_id=None,
            generation=generation,
            direction_id=None,
            role="dgm_selector",
            owner_type="orchestrator",
            context=context,
            prompt_name=EXPERT_PROMPT,
            schema_name=EXPERT_SCHEMA,
            parent_artifact_sha256=None,
            dependency_ticket_ids=[],
            objective=f"Select parent for generation {generation} draw {draw_index}.",
            expected_deliverable="Deterministic DGM selection event.",
        )
        tickets.publish_ticket(run_dir / "tickets", ticket)
        _ticket_transition(run_dir, ticket_id, "created", "admitted", "selection admitted", None)
        _ticket_transition(
            run_dir,
            ticket_id,
            "admitted",
            "running",
            "selection calculation started",
            _canonical_sha256(context),
        )


def _complete_selection_ticket(
    run_dir: Path,
    ticket_id: str,
    selection_event: Mapping[str, Any],
) -> None:
    artifact = str(selection_event["selection_event_sha256"])
    _ticket_transition(
        run_dir,
        ticket_id,
        "running",
        "completed",
        "selection recorded",
        artifact,
    )
    _ticket_transition(
        run_dir,
        ticket_id,
        "completed",
        "accepted",
        "selection event accepted",
        artifact,
    )


def _publish_child_expert_ticket(
    run_dir: Path,
    *,
    generation: int,
    draw_index: int,
    parent: Mapping[str, Any],
    direction: Mapping[str, Any],
    dependency_ticket_id: str,
) -> tuple[dict[str, object], dict[str, object]]:
    ticket_id = f"expert-g{generation}-d{draw_index}"
    node_id = f"node-g{generation}-d{draw_index}"
    context = build_expert_runtime_context(
        run_dir=run_dir,
        attempt_id=ticket_id,
        ticket_id=ticket_id,
        proposed_node_id=node_id,
        parent_node=parent,
        generation=generation,
        expert_role=str(direction["recommended_role"]),
        selected_direction=direction,
    )
    write_context = run_dir / "contexts" / f"{ticket_id}.json"
    _write_json_create_only(write_context, context, "child expert context")
    ticket = runtime_ticket(
        run_dir=run_dir,
        ticket_id=ticket_id,
        task_kind="expert",
        node_id=node_id,
        parent_node_id=str(parent["node_id"]),
        generation=generation,
        direction_id=str(direction["direction_id"]),
        role=str(direction["recommended_role"]),
        owner_type="expert",
        context=context,
        prompt_name=EXPERT_PROMPT,
        schema_name=EXPERT_SCHEMA,
        parent_artifact_sha256=str(parent["node_artifact_sha256"]),
        dependency_ticket_ids=[dependency_ticket_id],
        objective=f"Extend {parent['node_id']} along {direction['direction_id']}.",
        expected_deliverable="Strict child expert result or terminal failure.",
    )
    tickets.publish_ticket(run_dir / "tickets", ticket)
    return ticket, context


def _project_candidate(
    run_dir: Path,
    node: Mapping[str, Any],
    reconciliation: Mapping[str, Any],
) -> None:
    ticket_id = f"candidate-freeze-{node['node_id']}"
    if (run_dir / "tickets" / ticket_id / "ticket.json").exists():
        return
    context = {
        "schema_version": "crouzeix-candidate-freeze-context/v1",
        "run_id": run_dir.name,
        "node_id": node["node_id"],
        "node_artifact_sha256": node["node_artifact_sha256"],
        "reconciliation_sha256": reconciliation["reconciliation_sha256"],
    }
    ticket = runtime_ticket(
        run_dir=run_dir,
        ticket_id=ticket_id,
        task_kind="candidate_freeze",
        node_id=str(node["node_id"]),
        parent_node_id=None,
        generation=int(node["generation"]),
        direction_id=None,
        role="candidate_freeze",
        owner_type="orchestrator",
        context=context,
        prompt_name=EXPERT_PROMPT,
        schema_name=EXPERT_SCHEMA,
        parent_artifact_sha256=str(node["node_artifact_sha256"]),
        dependency_ticket_ids=[str(node["source_ticket_id"])],
        objective=f"Freeze score-complete candidate from {node['node_id']}.",
        expected_deliverable="Create-only candidate projection.",
    )
    tickets.publish_ticket(run_dir / "tickets", ticket)
    _ticket_transition(run_dir, ticket_id, "created", "admitted", "candidate freeze admitted", None)
    _ticket_transition(
        run_dir,
        ticket_id,
        "admitted",
        "running",
        "candidate freeze started",
        _canonical_sha256(context),
    )
    candidate_bytes = str(node["mathematical_payload"]["endpoint"]["text"]).encode("utf-8")
    projection = _freeze_candidate_projection(
        run_dir,
        projection_id=f"candidate-{node['node_id']}",
        node=node,
        reconciliation=reconciliation,
        candidate_bytes=candidate_bytes,
    )
    _ticket_transition(
        run_dir,
        ticket_id,
        "running",
        "completed",
        "candidate projection written",
        str(projection["candidate_projection_sha256"]),
    )
    _ticket_transition(
        run_dir,
        ticket_id,
        "completed",
        "accepted",
        "candidate projection accepted",
        str(projection["candidate_projection_sha256"]),
    )
    _append_frontier_event(
        run_dir,
        event_kind="candidate_projected",
        ticket_id=ticket_id,
        artifact_sha256=str(projection["candidate_projection_sha256"]),
    )


def _freeze_candidate_projection(
    run_dir: Path,
    *,
    projection_id: str,
    node: Mapping[str, Any],
    reconciliation: Mapping[str, Any],
    candidate_bytes: bytes,
) -> dict[str, object]:
    entry = _archive_entry_for_node(run_dir, str(node["node_id"]))
    candidate_sha256 = hashlib.sha256(candidate_bytes).hexdigest()
    if entry["candidate_proof_sha256"] != candidate_sha256:
        raise protocol.ValidationError("candidate proof digest does not match archive entry")
    root = run_dir / "candidate_projections"
    _ensure_directory(root, "candidate projections directory", create=True)
    projection = {
        "schema_version": "crouzeix-candidate-projection/v1",
        "projection_id": projection_id,
        "source_node_id": node["node_id"],
        "source_node_artifact_sha256": node["node_artifact_sha256"],
        "source_reconciliation_sha256": reconciliation["reconciliation_sha256"],
        "candidate_sha256": candidate_sha256,
        "candidate_byte_count": len(candidate_bytes),
    }
    projection["candidate_projection_sha256"] = _canonical_sha256(projection)
    candidate_path = root / f"{candidate_sha256}.tex"
    projection_path = root / f"{projection_id}.json"
    _reject_symlink(candidate_path, "candidate projection")
    _reject_symlink(projection_path, "candidate projection")
    if candidate_path.exists() or projection_path.exists():
        raise protocol.ValidationError("candidate projection already exists")
    _write_bytes_create_only(candidate_path, candidate_bytes, "candidate proof")
    _write_json_create_only(projection_path, projection, "candidate projection")
    _write_candidate_index(run_dir)
    return projection


def _select_child_direction(
    node: Mapping[str, Any],
    reconciliation: Mapping[str, Any],
    used_direction_ids: set[str],
) -> dict[str, object]:
    node_digest = str(node["node_artifact_sha256"])
    reconciliation_digest = str(reconciliation["reconciliation_sha256"])
    candidates: list[tuple[tuple[int, str], dict[str, object]]] = []
    payload = node["mathematical_payload"]
    for obligation in payload.get("unproved_obligations", []):
        strength = str(obligation.get("strength"))
        priority = {"theorem_strength": 0, "major": 2, "local": 4}.get(strength)
        if priority is None:
            continue
        obligation_id = _portable(str(obligation.get("obligation_id", "obligation")))
        direction = {
            "direction_id": f"obligation-{obligation_id}",
            "kind": "obligation",
            "statement": str(obligation["statement"]),
            "strength": strength,
            "recommended_role": "approximation_audit",
            "source_parent_node_id": node["node_id"],
            "source_node_artifact_sha256": node_digest,
            "source_reconciliation_sha256": None,
        }
        candidates.append(((priority, str(direction["direction_id"])), direction))
    for finding in reconciliation.get("evaluator_findings", []):
        severity = str(finding.get("severity"))
        priority = {"critical": 1, "major": 3}.get(severity)
        if priority is None:
            continue
        finding_id = _portable(str(finding["finding_id"]))
        direction = {
            "direction_id": f"finding-{finding_id}",
            "kind": "evaluator_finding",
            "statement": str(finding["statement"]),
            "strength": severity,
            "recommended_role": str(finding["recommended_role"]),
            "source_parent_node_id": node["node_id"],
            "source_node_artifact_sha256": node_digest,
            "source_reconciliation_sha256": reconciliation_digest,
        }
        candidates.append(((priority, str(direction["direction_id"])), direction))
    for proposed in payload.get("proposed_directions", []):
        proposed_id = _portable(str(proposed["direction_id"]))
        direction = {
            "direction_id": f"proposed-{proposed_id}",
            "kind": "proposed_direction",
            "statement": str(proposed["statement"]),
            "strength": "proposed",
            "recommended_role": str(proposed["recommended_role"]),
            "source_parent_node_id": node["node_id"],
            "source_node_artifact_sha256": node_digest,
            "source_reconciliation_sha256": None,
        }
        candidates.append(((5, str(direction["direction_id"])), direction))
    candidates.sort(key=lambda item: item[0])
    if not candidates:
        raise protocol.ValidationError("selected parent has no open directions")
    for _, direction in candidates:
        if str(direction["direction_id"]) not in used_direction_ids:
            return direction
    return candidates[0][1]


def _build_frontier_node_evaluation(
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
    payload = _frontier_evaluation_payload(evaluation_payload)
    evaluation = {
        "schema_version": "crouzeix-node-evaluation/v1",
        "evaluation_id": evaluation_id,
        "evaluator_index": evaluator_index,
        "ticket_id": ticket_id,
        "node_artifact_sha256": node["node_artifact_sha256"],
        "mathematical_payload_sha256": node["mathematical_payload_sha256"],
        "context_sha256": context_sha256,
        "source_kind": source_kind,
        "call_receipt_sha256": call_receipt_sha256,
        "terminal_ticket_event_sha256": terminal_ticket_event_sha256,
        "evaluation_payload": payload,
        "evaluation_payload_sha256": frontier.canonical_sha256(payload),
    }
    evaluation["node_evaluation_sha256"] = frontier.canonical_sha256(evaluation)
    return evaluation


def _frontier_evaluation_payload(value: Mapping[str, Any]) -> dict[str, object]:
    probes = list(value.get("probes", []))
    if len(probes) != 10:
        raise protocol.ValidationError("evaluation payload must contain ten probes")
    normalized = []
    for probe in probes:
        probe_id = str(probe["probe_id"])
        if probe_id not in frontier.PROBE_IDS:
            raise protocol.ValidationError("evaluation probe ID is invalid")
        status = str(probe["status"])
        if status not in {"pass", "fail", "insufficient_evidence"}:
            raise protocol.ValidationError("evaluation probe status is invalid")
        normalized.append({"probe_id": probe_id, "status": status})
    if tuple(probe["probe_id"] for probe in normalized) != frontier.PROBE_IDS:
        raise protocol.ValidationError("evaluation probe order must match closed probe IDs")
    findings = []
    for finding in value.get("findings", []):
        findings.append(
            {
                "finding_id": _portable(str(finding["finding_id"])),
                "severity": str(finding["severity"]),
                "statement": str(finding["statement"]),
                "locator": str(finding["locator"]),
                "recommended_role": str(finding["recommended_role"]),
            }
        )
    return {"probes": normalized, "findings": findings}


def _fallback_payload() -> dict[str, object]:
    return {
        "probes": [
            {"probe_id": probe_id, "status": "insufficient_evidence"}
            for probe_id in frontier.PROBE_IDS
        ],
        "findings": [],
    }


def _expert_provenance(
    run_dir: Path,
    context: Mapping[str, Any],
    receipt: Mapping[str, Any],
) -> dict[str, object]:
    return {
        "theorem_sha256": context["theorem_sha256"],
        "leakage_tier": protocol.read_run_spec(run_dir / "run_spec.json")["leakage"],
        "context_sha256": _canonical_sha256(context),
        "call_receipt_sha256": _canonical_sha256(receipt),
        "leakage_audit_sha256": _canonical_sha256(
            {
                "schema_version": "crouzeix-leakage-audit/v1",
                "ticket_id": context["ticket_id"],
                "forbidden_sources": context["forbidden_sources"],
                "status": "not_detected_by_harness",
            }
        ),
        "selected_direction": context["selected_direction"],
    }


def _append_selection_ledger(
    run_dir: Path,
    selection_event: Mapping[str, Any],
    ticket_id: str,
) -> None:
    frontier_store.open_frontier_store(run_dir).append_selection_event(
        {
            "schema_version": "crouzeix-selection-event/v1",
            "sequence": len(_selection_events(run_dir)) + 1,
            "run_id": run_dir.name,
            "event_kind": "selection_recorded",
            "ticket_id": ticket_id,
            "artifact_sha256": selection_event["selection_event_sha256"],
            "occurred_at_utc": _now(),
            "generation": selection_event["generation"],
            "draw_index": selection_event["draw_index"],
            "selected_node_id": selection_event["selected_node_id"],
            "archive_snapshot_sha256": selection_event["archive_snapshot_sha256"],
        }
    )
    _append_frontier_event(
        run_dir,
        event_kind="selection_recorded",
        ticket_id=ticket_id,
        artifact_sha256=str(selection_event["selection_event_sha256"]),
    )


def _append_frontier_event(
    run_dir: Path,
    *,
    event_kind: str,
    ticket_id: str,
    artifact_sha256: str,
) -> None:
    frontier_store.open_frontier_store(run_dir).append_frontier_event(
        {
            "schema_version": "crouzeix-frontier-event/v1",
            "sequence": len(_frontier_events(run_dir)) + 1,
            "run_id": run_dir.name,
            "event_kind": event_kind,
            "ticket_id": ticket_id,
            "artifact_sha256": artifact_sha256,
            "occurred_at_utc": _now(),
        }
    )


def _ticket_transition(
    run_dir: Path,
    ticket_id: str,
    from_state: str,
    to_state: str,
    reason: str,
    artifact_sha256: str | None,
) -> str:
    event = _ticket_event(
        run_dir,
        ticket_id,
        from_state,
        to_state,
        reason,
        artifact_sha256,
    )
    tickets.append_ticket_event(run_dir / "tickets", ticket_id, event)
    return _canonical_sha256(event)


def _ticket_event(
    run_dir: Path,
    ticket_id: str,
    from_state: str,
    to_state: str,
    reason: str,
    artifact_sha256: str | None,
) -> dict[str, object]:
    event_path = run_dir / "tickets" / ticket_id / "ticket_events.jsonl"
    return {
        "schema_version": "crouzeix-runtime-ticket-event/v1",
        "sequence": len(_read_jsonl(event_path)) + 1,
        "ticket_id": ticket_id,
        "from_state": from_state,
        "to_state": to_state,
        "reason": reason,
        "occurred_at_utc": _now(),
        "artifact_sha256": artifact_sha256,
    }


def _reconcile_run(run_dir: Path) -> dict[str, object]:
    _validate_all_runtime_tickets_terminal(run_dir)
    _validate_evidence_ticket_links(run_dir)
    attempts = _attempts(run_dir)
    selection_events = _selection_events(run_dir)
    evidence_ticket_count = len(
        {str(item["ticket_id"]) for item in attempts}
        | {str(item["ticket_id"]) for item in selection_events}
    )
    base = frontier_store.open_frontier_store(run_dir).reconcile_run(
        ticket_count=evidence_ticket_count
    )
    receipt = dict(base)
    receipt["evidence_ticket_count"] = evidence_ticket_count
    receipt["ticket_count"] = _ticket_count(run_dir)
    receipt["provider_call_count"] = _provider_call_count(run_dir)
    receipt["admission_count"] = len(list((run_dir / "admissions").glob("*.json")))
    receipt["node_evaluation_count"] = len(
        list((run_dir / "node_evaluations").glob("*/evaluator-*/evaluation.json"))
    )
    receipt["reconciliation_count"] = len(
        list((run_dir / "node_evaluations").glob("*/reconciliation.json"))
    )
    receipt["candidate_projection_count"] = _candidate_projection_count(run_dir)
    receipt["frontier_event_count"] = len(_frontier_events(run_dir))
    receipt["generation_count"] = len({item["generation"] for item in selection_events})
    receipt["archive_size"] = receipt["archive_entry_count"]
    receipt["selected_parent_frequencies"] = _selected_parent_frequencies(selection_events)
    receipt["evaluator_disagreement_count"] = sum(
        int(item["disagreement_count"]) for item in _reconciliations(run_dir)
    )
    receipt["score_distribution"] = _score_distribution(_archive_entries(run_dir))
    receipt.pop("run_receipt_sha256", None)
    receipt["run_receipt_sha256"] = _canonical_sha256(receipt)
    _write_json_replace(run_dir / "run_receipt.json", receipt)
    return receipt


def _limits_from_spec(spec: Mapping[str, Any]) -> RunnerLimits:
    frontier_config = spec["frontier"]
    return RunnerLimits(
        max_provider_calls=int(frontier_config["total_call_budget"]),
        admitted_node_budget=int(frontier_config["admitted_mathematical_node_budget"]),
        child_generations=int(frontier_config["child_generations"]),
        draws_per_generation=int(frontier_config["draws_per_generation"]),
    )


def _validate_all_runtime_tickets_terminal(
    run_dir: Path,
    *,
    allow_unstarted_roots: bool = False,
) -> None:
    ticket_root = run_dir / "tickets"
    if not ticket_root.exists():
        return
    for ticket_dir in sorted(ticket_root.iterdir()):
        if not ticket_dir.is_dir():
            continue
        ticket = tickets.validate_runtime_ticket(
            _read_json(ticket_dir / "ticket.json", "runtime ticket")
        )
        states = _ticket_states(run_dir, str(ticket["ticket_id"]))
        if not states:
            if (
                allow_unstarted_roots
                and ticket["task_kind"] == "expert"
                and int(ticket["generation"]) == 0
                and (
                    not _attempt_exists(run_dir, str(ticket["ticket_id"]))
                    or _ticket_has_no_terminal_evidence(run_dir, str(ticket["ticket_id"]))
                )
            ):
                continue
            if _ticket_has_evidence(run_dir, str(ticket["ticket_id"]), str(ticket["task_kind"])):
                raise protocol.ValidationError(
                    f"runtime ticket {ticket['ticket_id']} has evidence without terminal event"
                )
            raise protocol.ValidationError(
                f"runtime ticket {ticket['ticket_id']} is missing terminal event"
            )
            continue
        if states[-1] not in tickets.TERMINAL_TICKET_STATES:
            if (
                allow_unstarted_roots
                and ticket["task_kind"] == "expert"
                and int(ticket["generation"]) == 0
                and not _attempt_exists(run_dir, str(ticket["ticket_id"]))
                and states[-1] in {"admitted", "running"}
            ):
                continue
            raise protocol.ValidationError(
                f"runtime ticket {ticket['ticket_id']} is missing terminal event"
            )
        if _ticket_has_evidence(run_dir, str(ticket["ticket_id"]), str(ticket["task_kind"])) is False:
            raise protocol.ValidationError(
                f"runtime ticket {ticket['ticket_id']} has terminal event without evidence"
            )


def _validate_evidence_ticket_links(run_dir: Path) -> None:
    ticket_ids = _runtime_ticket_ids(run_dir)
    for call_dir in sorted((run_dir / "evaluator_calls").glob("*")):
        if not call_dir.is_dir():
            continue
        ticket_id = call_dir.name
        if ticket_id not in ticket_ids:
            raise protocol.ValidationError(f"orphan evaluator evidence for {ticket_id}")
        receipt = _read_json(call_dir / "receipt.json", "evaluator receipt")
        if receipt.get("ticket_id") != ticket_id:
            raise protocol.ValidationError("evaluator receipt ticket_id mismatch")
        if not (
            run_dir
            / "tickets"
            / ticket_id
            / "ticket_events.jsonl"
        ).exists():
            raise protocol.ValidationError(f"orphan evaluator evidence for {ticket_id}")
    for projection in _candidate_projections(run_dir):
        ticket_id = f"candidate-freeze-{projection['source_node_id']}"
        if ticket_id not in ticket_ids:
            raise protocol.ValidationError(f"orphan candidate evidence for {ticket_id}")
        if not (
            run_dir
            / "tickets"
            / ticket_id
            / "ticket_events.jsonl"
        ).exists():
            raise protocol.ValidationError(f"orphan candidate evidence for {ticket_id}")


def _ticket_has_evidence(run_dir: Path, ticket_id: str, task_kind: str) -> bool:
    if task_kind == "expert":
        return any(item["ticket_id"] == ticket_id for item in _attempts(run_dir))
    if task_kind == "evaluator":
        return (run_dir / "evaluator_calls" / ticket_id / "receipt.json").is_file()
    if task_kind == "selection":
        return any(item["ticket_id"] == ticket_id for item in _selection_events(run_dir))
    if task_kind == "candidate_freeze":
        return any(
            str(item.get("projection_id")) == ticket_id.replace("candidate-freeze-", "candidate-")
            for item in _candidate_projections(run_dir)
        )
    return any(
        event["ticket_id"] == ticket_id
        and event["event_kind"]
        in {
            "attempt_terminal",
            "admission_accepted",
            "admission_rejected",
            "archive_entry_created",
            "selection_recorded",
            "candidate_projected",
        }
        for event in _frontier_events(run_dir)
    )


def _ticket_has_no_terminal_evidence(run_dir: Path, ticket_id: str) -> bool:
    return not _attempt_exists(run_dir, ticket_id) and not _ticket_states(run_dir, ticket_id)


def _ticket_states(run_dir: Path, ticket_id: str) -> list[str]:
    event_path = run_dir / "tickets" / ticket_id / "ticket_events.jsonl"
    return [str(event["to_state"]) for event in _read_jsonl(event_path)]


def _ticket_is_terminal(run_dir: Path, ticket_id: str) -> bool:
    states = _ticket_states(run_dir, ticket_id)
    return bool(states) and states[-1] in tickets.TERMINAL_TICKET_STATES


def _reject_partial_later_phase_artifacts(run_dir: Path, phase: str) -> None:
    if phase in {"roots", "evaluate-roots"}:
        if _selection_events(run_dir):
            raise protocol.ValidationError("partial later phase selection evidence exists")
        if (run_dir / "candidate_projections" / "finalization.json").exists():
            raise protocol.ValidationError("partial later phase finalization artifact exists")
    if phase == "generations" and (run_dir / "candidate_projections" / "finalization.json").exists():
        raise protocol.ValidationError("partial later phase finalization artifact exists")


def _validate_roots_evidence(run_dir: Path) -> None:
    root_attempts = [
        item for item in _attempts(run_dir) if str(item["attempt_id"]).startswith("expert-g0-")
    ]
    if len(root_attempts) != len(prepare_frontier.EXPERT_ROLES):
        raise protocol.ValidationError("roots evidence is incomplete")
    for role in prepare_frontier.EXPERT_ROLES:
        ticket_id = prepare_frontier.root_ticket_id(role)
        if not _ticket_is_terminal(run_dir, ticket_id):
            raise protocol.ValidationError("roots evidence is missing terminal ticket events")


def _validate_evaluate_roots_evidence(run_dir: Path) -> None:
    root_nodes = [node for node in _nodes(run_dir) if int(node["generation"]) == 0]
    archived_roots = {
        str(entry["node_id"]) for entry in _archive_entries(run_dir)
    }
    missing = [node["node_id"] for node in root_nodes if node["node_id"] not in archived_roots]
    if missing:
        raise protocol.ValidationError("evaluate-roots evidence is incomplete")


def _enforce_phase_boundary(run_dir: Path, phase: str) -> None:
    _reject_partial_later_phase_artifacts(run_dir, phase)
    phases = _phase_state(run_dir)
    if phase in phases:
        raise protocol.ValidationError(f"frontier phase repeat rejected: {phase}")
    predecessor = PHASE_PREDECESSOR[phase]
    if predecessor is not None and predecessor not in phases:
        raise protocol.ValidationError(
            f"frontier phase {phase} requires completed {predecessor}"
        )
    if predecessor == "roots":
        _validate_roots_evidence(run_dir)
    if predecessor == "evaluate-roots":
        _validate_evaluate_roots_evidence(run_dir)
    _validate_all_runtime_tickets_terminal(
        run_dir,
        allow_unstarted_roots=phase == "roots",
    )


def _require_completed_phase(run_dir: Path, phase: str) -> None:
    if phase not in _phase_state(run_dir):
        raise protocol.ValidationError(f"frontier check requires completed {phase}")


def _mark_phase(run_dir: Path, phase: str, result: Mapping[str, Any]) -> None:
    phases = _phase_state(run_dir)
    phases[phase] = {
        "completed_at_utc": _now(),
        "result_sha256": _canonical_sha256(result),
    }
    _write_json_replace(run_dir / "frontier_phase_state.json", phases)


def _phase_state(run_dir: Path) -> dict[str, object]:
    path = run_dir / "frontier_phase_state.json"
    if not path.exists():
        return {}
    return _read_json(path, "frontier phase state")


def _provider_call_count(run_dir: Path) -> int:
    evaluator_root = run_dir / "evaluator_calls"
    evaluator_calls = 0
    if evaluator_root.exists():
        evaluator_calls = sum(1 for path in evaluator_root.iterdir() if path.is_dir())
    return len(_attempts(run_dir)) + evaluator_calls


def _root_attempt_count(run_dir: Path) -> int:
    return sum(1 for item in _attempts(run_dir) if str(item["attempt_id"]).startswith("expert-g0-"))


def _root_completed_count(run_dir: Path) -> int:
    return sum(
        1
        for item in _attempts(run_dir)
        if str(item["attempt_id"]).startswith("expert-g0-")
        and item["terminal_status"] == "completed"
    )


def _attempt_status(status: str) -> str:
    if status not in {"completed", "failed", "malformed", "timed_out", "blocked_resource"}:
        raise protocol.ValidationError("provider attempt terminal status is invalid")
    return status


def _ticket_count(run_dir: Path) -> int:
    ticket_root = run_dir / "tickets"
    if not ticket_root.exists():
        return 0
    return sum(1 for path in ticket_root.iterdir() if path.is_dir())


def _runtime_ticket_ids(run_dir: Path) -> set[str]:
    ticket_root = run_dir / "tickets"
    if not ticket_root.exists():
        return set()
    ticket_ids = set()
    for path in sorted(ticket_root.iterdir()):
        if path.is_dir():
            ticket_ids.add(path.name)
    return ticket_ids


def _candidate_projection_count(run_dir: Path) -> int:
    return len(_candidate_projections(run_dir))


def _candidate_projections(run_dir: Path) -> list[dict[str, object]]:
    root = run_dir / "candidate_projections"
    if not root.exists():
        return []
    projections = []
    for path in sorted(root.glob("candidate-*.json")):
        value = _read_json(path, "candidate projection")
        if value.get("schema_version") == "crouzeix-candidate-projection/v1":
            projections.append(value)
    return projections


def _write_candidate_index(run_dir: Path) -> None:
    projections = _candidate_projections(run_dir)
    index = {
        "schema_version": "crouzeix-candidate-projection-index/v1",
        "projection_count": len(projections),
        "projections": sorted(
            projections,
            key=lambda item: str(item["source_node_id"]),
        ),
    }
    index["candidate_projection_index_sha256"] = _canonical_sha256(index)
    _write_json_replace(run_dir / "candidate_projections" / "index.json", index)


def _archive_entry_for_node(run_dir: Path, node_id: str) -> dict[str, object]:
    for entry in _archive_entries(run_dir):
        if entry["node_id"] == node_id:
            return entry
    raise protocol.ValidationError(f"archive entry missing for {node_id}")


def _node_count(run_dir: Path) -> int:
    return len(_nodes(run_dir))


def _nodes(run_dir: Path) -> list[dict[str, object]]:
    root = run_dir / "mathematical_nodes"
    if not root.exists():
        return []
    return [
        expert_contracts.validate_mathematical_node(_read_json(path / "node.json", "node"))
        for path in sorted(root.iterdir())
        if path.is_dir()
    ]


def _node_by_id(run_dir: Path, node_id: str) -> dict[str, object]:
    for node in _nodes(run_dir):
        if node["node_id"] == node_id:
            return node
    raise protocol.ValidationError(f"unknown node {node_id}")


def _archive_entries(run_dir: Path) -> list[dict[str, object]]:
    root = run_dir / "archive_entries"
    if not root.exists():
        return []
    return [
        frontier._validate_archive_entry(_read_json(path, "archive entry"))
        for path in sorted(root.glob("*.json"))
    ]


def _archive_entry_exists(run_dir: Path, node_id: str) -> bool:
    return (run_dir / "archive_entries" / f"{node_id}.json").exists()


def _reconciliation_for_node(run_dir: Path, node_id: str) -> dict[str, object]:
    return frontier._validate_reconciliation(
        _read_json(
            run_dir / "node_evaluations" / node_id / "reconciliation.json",
            "reconciliation",
        )
    )


def _reconciliations(run_dir: Path) -> list[dict[str, object]]:
    return [
        frontier._validate_reconciliation(_read_json(path, "reconciliation"))
        for path in sorted((run_dir / "node_evaluations").glob("*/reconciliation.json"))
    ]


def _attempts(run_dir: Path) -> list[dict[str, object]]:
    return _read_jsonl(run_dir / "attempt_ledger.jsonl")


def _attempt_exists(run_dir: Path, attempt_id: str) -> bool:
    return any(item["attempt_id"] == attempt_id for item in _attempts(run_dir))


def _selection_events(run_dir: Path) -> list[dict[str, object]]:
    return _read_jsonl(run_dir / "selection_events.jsonl")


def _frontier_events(run_dir: Path) -> list[dict[str, object]]:
    return _read_jsonl(run_dir / "frontier_events.jsonl")


def _read_ticket(run_dir: Path, ticket_id: str) -> dict[str, object]:
    return tickets.validate_runtime_ticket(
        _read_json(run_dir / "tickets" / ticket_id / "ticket.json", "runtime ticket")
    )


def _strongest_archive_entry(entries: list[dict[str, object]]) -> dict[str, object] | None:
    if not entries:
        return None
    return sorted(
        entries,
        key=lambda entry: (-int(entry["unanimous_pass_count"]), str(entry["node_id"])),
    )[0]


def _selected_parent_frequencies(selection_events: list[dict[str, object]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for event in selection_events:
        node_id = str(event["selected_node_id"])
        counts[node_id] = counts.get(node_id, 0) + 1
    return dict(sorted(counts.items()))


def _score_distribution(entries: list[dict[str, object]]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for entry in entries:
        score = str(entry["unanimous_pass_count"])
        counts[score] = counts.get(score, 0) + 1
    return dict(sorted(counts.items(), key=lambda item: int(item[0])))


def _forbidden_sources(run_dir: Path) -> list[str]:
    return sorted(set(FORBIDDEN_SOURCES) | set(_excluded_sources(run_dir)))


def _excluded_sources(run_dir: Path) -> list[str]:
    try:
        spec = protocol.read_run_spec(run_dir / "run_spec.json")
    except Exception:
        return ["public proof manuscripts"]
    return [str(item) for item in spec.get("generation_excluded_classes", [])]


def _provider_call_record(result: Mapping[str, Any]) -> dict[str, object]:
    call_dir = result.get("call_dir")
    return {
        "call_dir": None if call_dir is None else str(call_dir),
        "receipt_sha256": _canonical_sha256(result["receipt"]),
    }


def _tree_sha256(path: Path) -> str:
    records = []
    for item in sorted(path.rglob("*")):
        if item.is_file():
            records.append(
                {
                    "path": str(item.relative_to(path)),
                    "sha256": _file_sha256(item),
                }
            )
    return _canonical_sha256({"files": records})


def _file_sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _sha256_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def _canonical_sha256(value: object) -> str:
    return hashlib.sha256(_canonical_json_bytes(value)).hexdigest()


def _canonical_json_bytes(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")


def _read_json(path: Path, label: str) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        raise protocol.ValidationError(f"cannot parse {label}: {error}") from error
    if not isinstance(value, dict):
        raise protocol.ValidationError(f"{label} must be a JSON object")
    return value


def _read_jsonl(path: Path) -> list[dict[str, object]]:
    if not path.exists():
        return []
    values = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line:
            continue
        value = json.loads(line)
        if not isinstance(value, dict):
            raise protocol.ValidationError("JSONL row must be an object")
        values.append(value)
    return values


def _read_text(path: Path, label: str) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as error:
        raise protocol.ValidationError(f"cannot read {label}: {error}") from error


def _write_json_create_only(path: Path, value: Mapping[str, Any], label: str) -> None:
    _reject_symlink(path, label)
    _ensure_directory(path.parent, f"{label} parent", create=True)
    try:
        with path.open("x", encoding="utf-8") as handle:
            json.dump(value, handle, indent=2, sort_keys=True)
            handle.write("\n")
    except FileExistsError as error:
        raise protocol.ValidationError(f"{label} already exists: {path}") from error


def _write_json_replace(path: Path, value: Mapping[str, Any]) -> None:
    _reject_symlink(path, path.name)
    _ensure_directory(path.parent, f"{path.name} parent", create=True)
    tmp = path.with_name(f".{path.name}.tmp")
    _reject_symlink(tmp, tmp.name)
    tmp.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    os.replace(tmp, path)


def _write_bytes_create_only(path: Path, data: bytes, label: str) -> None:
    _reject_symlink(path, label)
    _ensure_directory(path.parent, f"{label} parent", create=True)
    try:
        with path.open("xb") as handle:
            handle.write(data)
    except FileExistsError as error:
        raise protocol.ValidationError(f"{label} already exists: {path}") from error


def _ensure_directory(path: Path, label: str, *, create: bool) -> None:
    current = Path(path.anchor) if path.is_absolute() else Path(".")
    parts = path.parts[1:] if path.is_absolute() else path.parts
    for part in parts:
        current = current / part
        try:
            metadata = current.lstat()
        except FileNotFoundError:
            if not create:
                raise protocol.ValidationError(f"{label} does not exist: {current}")
            current.mkdir(mode=0o700)
            metadata = current.lstat()
        if stat.S_ISLNK(metadata.st_mode):
            raise protocol.ValidationError(f"{label} contains symlink: {current}")
        if not stat.S_ISDIR(metadata.st_mode):
            raise protocol.ValidationError(f"{label} must be a directory: {current}")


def _reject_symlink(path: Path, label: str) -> None:
    try:
        metadata = path.lstat()
    except FileNotFoundError:
        return
    if stat.S_ISLNK(metadata.st_mode):
        raise protocol.ValidationError(f"{label} cannot be a symlink")


def _portable(value: str) -> str:
    text = PORTABLE_ID.sub("-", value.lower()).strip("-")
    text = re.sub(r"-+", "-", text)
    if not text or not text[0].isalpha():
        text = "id-" + text
    return text[:96]


def _now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("phase", choices=PHASES)
    parser.add_argument("run_dir", type=Path)
    args = parser.parse_args(argv)
    print(json.dumps(run_phase(args.run_dir, args.phase), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
