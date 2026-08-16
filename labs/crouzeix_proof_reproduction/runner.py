from __future__ import annotations

import json
import os
import signal
import stat
import subprocess
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Callable, Mapping

from protocol import (
    MAX_JSON_BYTES,
    RouteRegistry,
    ValidationError,
    read_run_spec,
    read_strict_json_object,
    sha256_bytes,
)


MAX_PROMPT_BYTES = 384 * 1024
MAX_EVENT_BYTES = 16 * 1024 * 1024
MAX_STDERR_BYTES = 4 * 1024 * 1024
MAX_CANDIDATE_BYTES = 2 * 1024 * 1024
USAGE_FIELDS = (
    "input_tokens",
    "cache_creation_input_tokens",
    "cached_input_tokens",
    "output_tokens",
    "reasoning_output_tokens",
)
CALL_ROLES = frozenset(
    {
        "historical_root",
        "route_worker",
        "controller",
        "redirect",
        "synthesizer",
        "logical_critic",
        "operator_critic",
        "repair",
        "expert",
        "proof_progress_evaluator",
    }
)
HISTORICAL_FIELDS = frozenset(
    {"schema_version", "status", "candidate_path"}
)
ROUTE_FIELDS = frozenset(
    {
        "schema_version",
        "route_id",
        "family",
        "mechanism",
        "proved_statements",
        "unproved_obligations",
        "circularity_risks",
        "candidate_proof",
        "blocker",
        "confidence_basis",
    }
)
CONTROLLER_FIELDS = frozenset(
    {
        "schema_version",
        "decisions",
        "selected_route_ids",
        "family_collapse",
        "redirect_brief",
    }
)
SYNTHESIS_FIELDS = frozenset(
    {"schema_version", "parent_route_ids", "candidate_proof", "obligations"}
)
CRITIC_FIELDS = frozenset(
    {"schema_version", "critic_role", "candidate_sha256", "findings"}
)
REPAIR_FIELDS = frozenset(
    {
        "schema_version",
        "parent_candidate_sha256",
        "candidate_proof",
        "dispositions",
        "unproved_obligations",
    }
)


def build_command(
    spec: Mapping[str, Any],
    *,
    workspace: Path,
    schema: Path,
    final: Path,
    allowed_tools: list[str] | None = None,
) -> list[str]:
    command = [
        str(spec["cli"]["path"]),
        "exec",
        "--ignore-user-config",
        "--ignore-rules",
        "--ephemeral",
        "--sandbox",
        str(spec["sandbox"]),
        "--config",
        f'approval_policy="{spec["approval_policy"]}"',
        "--model",
        str(spec["model"]),
        "--cd",
        str(workspace),
        "--skip-git-repo-check",
        "--output-schema",
        str(schema),
        "--output-last-message",
        str(final),
        "--json",
    ]
    for tool in allowed_tools if allowed_tools is not None else spec["allowed_tools"]:
        command.extend(["--allowed-tool", str(tool)])
    command.append("-")
    return command


def run_call(
    run_dir: Path,
    spec: Mapping[str, Any],
    *,
    call_id: str,
    role: str,
    prompt: str,
    schema_path: Path,
    parent_digests: Mapping[str, str],
    ticket_binding: Mapping[str, str] | None = None,
    allowed_tools: list[str] | None = None,
    accounting_path: Path | None = None,
    after_receipt: Callable[[], None] | None = None,
    finalize_receipt: Callable[[dict[str, Any], dict[str, Any] | None], None] | None = None,
) -> dict[str, Any]:
    if role not in CALL_ROLES:
        raise ValidationError(f"unknown call role {role}")
    if len(prompt.encode("utf-8")) > MAX_PROMPT_BYTES or "\0" in prompt:
        raise ValidationError("call prompt is invalid or exceeds byte cap")
    _portable_call_id(call_id)
    _verify_cli_identity(spec)
    schema = _regular_file(schema_path, "output schema")
    call_dir = run_dir / "calls" / call_id
    if call_dir.exists() or call_dir.is_symlink():
        raise ValidationError(f"refusing existing call directory: {call_dir}")
    call_dir.mkdir(mode=0o700)
    workspace = call_dir / "workspace"
    workspace.mkdir(mode=0o700)
    final = call_dir / "final.json"
    events = call_dir / "events.jsonl"
    stderr = call_dir / "stderr.txt"
    prompt_path = call_dir / "prompt.md"
    schema_copy = call_dir / "output.schema.json"
    request_path = call_dir / "request.json"
    receipt_path = call_dir / "receipt.json"
    prompt_path.write_text(prompt)
    schema_copy.write_bytes(schema.read_bytes())
    command = build_command(
        spec,
        workspace=workspace,
        schema=schema_copy,
        final=final,
        allowed_tools=allowed_tools,
    )
    tool_names = [
        str(tool)
        for tool in (
            allowed_tools if allowed_tools is not None else spec["allowed_tools"]
        )
    ]
    request = {
        "schema_version": "crouzeix-call-request/v1",
        "call_id": call_id,
        "role": role,
        "cli": dict(spec["cli"]),
        "model": spec["model"],
        "sandbox": spec["sandbox"],
        "approval_policy": spec["approval_policy"],
        "network_access": spec["network_access"],
        "command": command,
        "cwd": str(workspace),
        "allowed_tools": tool_names,
        "prompt_bytes": prompt_path.stat().st_size,
        "prompt_sha256": sha256_bytes(prompt_path.read_bytes()),
        "schema_bytes": schema_copy.stat().st_size,
        "schema_sha256": sha256_bytes(schema_copy.read_bytes()),
        "parent_digests": dict(sorted(parent_digests.items())),
    }
    if ticket_binding is not None:
        request.update(
            {
                "ticket_id": str(ticket_binding["ticket_id"]),
                "ticket_sha256": str(ticket_binding["ticket_sha256"]),
            }
        )
        for field in (
            "ticket_context_sha256",
            "ticket_schema_sha256",
            "ticket_prompt_sha256",
            "ticket_parent_artifact_sha256",
        ):
            if field in ticket_binding:
                request[field] = ticket_binding[field]
    request_path.write_text(json.dumps(request, indent=2, sort_keys=True) + "\n")

    started = _now()
    timed_out = False
    returncode: int | None = None
    child: subprocess.Popen[bytes] | None = None
    try:
        with events.open("wb") as stdout_file, stderr.open("wb") as stderr_file:
            child = subprocess.Popen(
                command,
                stdin=subprocess.PIPE,
                stdout=stdout_file,
                stderr=stderr_file,
                env=_minimal_environment(),
                start_new_session=True,
            )
            try:
                child.communicate(
                    input=prompt.encode("utf-8"),
                    timeout=int(spec["timeout_seconds"]),
                )
            except subprocess.TimeoutExpired:
                timed_out = True
                _terminate_process_group(child)
                child.communicate()
            returncode = child.returncode
    except OSError as error:
        if child is not None and child.poll() is None:
            _terminate_process_group(child)
        stderr.write_text(f"local launch failure: {error}\n")

    completed = _now()
    event_scan: dict[str, Any] | None = None
    final_value: dict[str, Any] | None = None
    parse_error: str | None = None
    try:
        _bounded_regular_file(events, "event log", MAX_EVENT_BYTES)
        _bounded_regular_file(stderr, "stderr", MAX_STDERR_BYTES)
        event_scan = parse_events(events)
        if final.exists():
            final_value = read_strict_json_object(final, "final response")
    except ValidationError as error:
        parse_error = str(error)

    if timed_out:
        status = "timed_out"
    elif returncode not in (0, None):
        status = "failed"
    elif parse_error is not None or final_value is None:
        status = "malformed"
    else:
        status = "completed"

    receipt = {
        "schema_version": "crouzeix-call-receipt/v1",
        "call_id": call_id,
        "role": role,
        "status": status,
        "cli": dict(spec["cli"]),
        "model": spec["model"],
        "sandbox": spec["sandbox"],
        "approval_policy": spec["approval_policy"],
        "network_access": spec["network_access"],
        "allowed_tools": tool_names,
        "blocked_reason": None,
        "started_at_utc": started,
        "completed_at_utc": completed,
        "returncode": returncode,
        "timed_out": timed_out,
        "prompt_bytes": prompt_path.stat().st_size,
        "prompt_sha256": sha256_bytes(prompt_path.read_bytes()),
        "schema_bytes": schema_copy.stat().st_size,
        "schema_sha256": sha256_bytes(schema_copy.read_bytes()),
        "events_bytes": events.stat().st_size if events.exists() else 0,
        "events_sha256": sha256_bytes(events.read_bytes()) if events.exists() else None,
        "stderr_bytes": stderr.stat().st_size if stderr.exists() else 0,
        "stderr_sha256": sha256_bytes(stderr.read_bytes()) if stderr.exists() else None,
        "final_bytes": final.stat().st_size if final.exists() else None,
        "final_sha256": sha256_bytes(final.read_bytes()) if final.exists() else None,
        "session_id": event_scan["session_id"] if event_scan else None,
        "usage": event_scan["usage"] if event_scan else None,
        "event_types": event_scan["event_types"] if event_scan else [],
        "parse_error": parse_error,
        "parent_digests": dict(sorted(parent_digests.items())),
    }
    if ticket_binding is not None:
        receipt.update(
            {
                "ticket_id": str(ticket_binding["ticket_id"]),
                "ticket_sha256": str(ticket_binding["ticket_sha256"]),
            }
        )
        for field in (
            "ticket_context_sha256",
            "ticket_schema_sha256",
            "ticket_prompt_sha256",
            "ticket_parent_artifact_sha256",
        ):
            if field in ticket_binding:
                receipt[field] = ticket_binding[field]
    if finalize_receipt is not None:
        finalize_receipt(receipt, final_value)
    receipt_path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    if accounting_path is not None:
        _append_accounting_receipt(accounting_path, receipt)
    if after_receipt is not None:
        after_receipt()
    return {
        "call_dir": call_dir,
        "receipt": receipt,
        "final": final_value,
    }


def _append_accounting_receipt(path: Path, receipt: Mapping[str, Any]) -> None:
    if path.exists() and not path.is_file():
        raise ValidationError("accounting path must be a regular file")
    line = json.dumps(receipt, sort_keys=True, separators=(",", ":"))
    with path.open("a", encoding="utf-8") as output:
        output.write(line + "\n")


def parse_events(path: Path) -> dict[str, Any]:
    session_id = None
    usage = None
    types = []
    for line_number, raw_line in enumerate(path.read_bytes().splitlines(), 1):
        if len(raw_line) > MAX_JSON_BYTES:
            raise ValidationError(f"event log line {line_number} exceeds byte cap")
        try:
            event = json.loads(raw_line)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise ValidationError(f"event log line {line_number} is not JSON") from error
        if not isinstance(event, dict) or not isinstance(event.get("type"), str):
            raise ValidationError(f"event log line {line_number} has no type")
        event_type = event["type"]
        types.append(event_type)
        if event_type == "thread.started":
            thread_id = event.get("thread_id")
            if not isinstance(thread_id, str) or not thread_id or len(thread_id) > 256:
                raise ValidationError("thread.started has invalid thread_id")
            if session_id is not None and session_id != thread_id:
                raise ValidationError("event log has multiple session IDs")
            session_id = thread_id
        if event_type == "turn.completed" and "usage" in event:
            usage = _parse_usage(event["usage"])
    if not types:
        raise ValidationError("event log is empty")
    if session_id is None:
        raise ValidationError("event log has no thread.started")
    if "turn.completed" not in types:
        raise ValidationError("event log has no turn.completed")
    return {"session_id": session_id, "usage": usage, "event_types": types}


def validate_role_output(role: str, value: Mapping[str, Any]) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ValidationError(f"{role} output must be an object")
    validators = {
        "historical_root": _validate_historical_output,
        "route_worker": _validate_route_output,
        "redirect": _validate_route_output,
        "controller": _validate_controller_output,
        "synthesizer": _validate_synthesis_output,
        "logical_critic": _validate_critic_output,
        "operator_critic": _validate_critic_output,
        "repair": _validate_repair_output,
    }
    try:
        validator = validators[role]
    except KeyError as error:
        raise ValidationError(f"unknown call role {role}") from error
    validator(value)
    return dict(value)


def aggregate_usage(receipts: list[Mapping[str, Any]]) -> dict[str, int | None]:
    totals: dict[str, int | None] = {field: None for field in USAGE_FIELDS}
    missing = 0
    for receipt in receipts:
        usage = receipt.get("usage")
        if usage is None:
            missing += 1
            continue
        for field in USAGE_FIELDS:
            value = usage.get(field)
            if value is None:
                continue
            totals[field] = (totals[field] or 0) + value
    totals["calls_without_usage"] = missing
    return totals


def run_experiment(run_dir: Path) -> dict[str, Any]:
    root = run_dir.resolve()
    spec = read_run_spec(root / "run_spec.json")
    receipt_path = root / "run_receipt.json"
    if receipt_path.exists() or receipt_path.is_symlink():
        raise ValidationError("refusing to rerun an experiment with a run receipt")
    if any((root / "calls").iterdir()):
        raise ValidationError("refusing to resume a partially executed experiment")
    failure = None
    try:
        if spec["arm"] == "historical":
            calls, candidate_sha256, promotion = _run_historical(root, spec)
        elif spec["arm"] == "orchestrated":
            calls, candidate_sha256, promotion = _run_orchestrated(root, spec)
        else:
            raise ValidationError("guided arm execution is not implemented")
        call_receipts = [item["receipt"] for item in calls]
        execution_status = "completed"
    except ValidationError as error:
        failure = str(error)
        call_receipts = _load_call_receipts(root / "calls")
        candidate_sha256 = _existing_candidate_digest(root)
        promotion = "not_promoted"
        execution_status = "failed"
    statuses = Counter(str(receipt["status"]) for receipt in call_receipts)
    receipt = {
        "schema_version": "crouzeix-run-receipt/v1",
        "run_id": spec["run_id"],
        "arm": spec["arm"],
        "leakage": spec["leakage"],
        "model": spec["model"],
        "call_count": len(call_receipts),
        "call_status_counts": dict(sorted(statuses.items())),
        "usage": aggregate_usage(call_receipts),
        "candidate_sha256": candidate_sha256,
        "promotion": promotion,
        "execution_status": execution_status,
        "failure": failure,
        "completed_at_utc": _now(),
    }
    receipt_path.write_text(json.dumps(receipt, indent=2, sort_keys=True) + "\n")
    return receipt


def _load_call_receipts(calls_root: Path) -> list[dict[str, Any]]:
    receipts = []
    for call_dir in sorted(calls_root.iterdir()):
        if call_dir.is_symlink() or not call_dir.is_dir():
            raise ValidationError("calls directory contains an invalid entry")
        receipt_path = call_dir / "receipt.json"
        if receipt_path.is_file():
            receipts.append(
                read_strict_json_object(receipt_path, f"{call_dir.name} receipt")
            )
    return receipts


def _existing_candidate_digest(root: Path) -> str | None:
    digest_path = root / "candidate/candidate.sha256"
    if not digest_path.exists():
        return None
    digest = digest_path.read_text().strip()
    _sha256(digest, "candidate digest")
    return digest


def _run_historical(
    root: Path, spec: Mapping[str, Any]
) -> tuple[list[dict[str, Any]], str | None, str]:
    prompt = (root / "inputs/execution_prompt.txt").read_text()
    result = run_call(
        root,
        spec,
        call_id="historical-root",
        role="historical_root",
        prompt=prompt,
        schema_path=root / "schemas/historical_final.schema.json",
        parent_digests={},
    )
    candidate_digest = None
    if result["receipt"]["status"] == "completed":
        final = result["final"]
        if final.get("schema_version") != "crouzeix-historical-final/v1":
            _mark_malformed(result, "historical final schema_version is invalid")
        elif final.get("status") == "candidate_written":
            if final.get("candidate_path") != "candidate.tex":
                _mark_malformed(result, "historical candidate_path must be candidate.tex")
            else:
                try:
                    candidate_digest = _freeze_candidate(
                        result["call_dir"] / "workspace/candidate.tex",
                        root / "candidate/candidate.tex",
                    )
                except ValidationError as error:
                    _mark_malformed(result, str(error))
        elif final.get("status") != "no_candidate" or final.get("candidate_path") is not None:
            _mark_malformed(result, "historical final response is inconsistent")
    return [
        result
    ], candidate_digest, "not_reviewed" if candidate_digest is not None else "not_promoted"


def _run_orchestrated(
    root: Path, spec: Mapping[str, Any]
) -> tuple[list[dict[str, Any]], str | None, str]:
    theorem = (root / "inputs/theorem.txt").read_text()
    calls: list[dict[str, Any]] = []
    routes: list[dict[str, Any]] = []
    registry = RouteRegistry(str(spec["run_id"]))
    route_template = (root / "prompts/route_worker.md").read_text()
    for index in range(1, 4):
        route_id = f"route-{index}"
        result = run_call(
            root,
            spec,
            call_id=f"route-worker-{index}",
            role="route_worker",
            prompt=_phase_prompt(
                "route_worker",
                theorem,
                route_template,
                f"ROUTE_ID: {route_id}",
            ),
            schema_path=root / "schemas/route.schema.json",
            parent_digests={},
        )
        calls.append(result)
        route = _require_completed_final(result, "crouzeix-route/v1")
        if route.get("route_id") != route_id:
            raise ValidationError(f"route worker returned wrong route_id for {route_id}")
        routes.append(route)
        registry.apply(
            _route_event(
                registry,
                spec,
                route,
                to_state="independent",
                reason="independent worker output sealed",
                prompt_sha256=result["receipt"]["prompt_sha256"],
                output_sha256=result["receipt"]["final_sha256"],
            )
        )

    controller_prompt = _phase_prompt(
        "controller",
        theorem,
        (root / "prompts/controller.md").read_text(),
        "SEALED_ROUTE_RECORDS:\n" + _canonical_json(routes),
    )
    controller_result = run_call(
        root,
        spec,
        call_id="controller",
        role="controller",
        prompt=controller_prompt,
        schema_path=root / "schemas/controller.schema.json",
        parent_digests={
            route["route_id"]: calls[index]["receipt"]["final_sha256"]
            for index, route in enumerate(routes)
        },
    )
    calls.append(controller_result)
    controller = _require_completed_final(
        controller_result, "crouzeix-controller/v1"
    )
    decisions = controller.get("decisions")
    if not isinstance(decisions, list) or len(decisions) != 3:
        raise ValidationError("controller must return three decisions")
    decision_by_route = {item["route_id"]: item for item in decisions}
    if set(decision_by_route) != {route["route_id"] for route in routes}:
        raise ValidationError("controller decisions do not cover initial routes")
    for route in routes:
        decision = decision_by_route[route["route_id"]]
        state = decision["state"]
        if state not in {"blocked", "viable"}:
            raise ValidationError("controller returned an invalid route state")
        registry.apply(
            _route_event(
                registry,
                spec,
                route,
                to_state=state,
                reason=decision["reason"],
                prompt_sha256=controller_result["receipt"]["prompt_sha256"],
                output_sha256=controller_result["receipt"]["final_sha256"],
                concrete_artifacts=(
                    route.get("proved_statements", []) if state == "viable" else []
                ),
            )
        )

    if controller.get("family_collapse"):
        brief = controller.get("redirect_brief")
        if not isinstance(brief, str) or not brief.strip():
            raise ValidationError("family collapse requires a redirect brief")
        redirect_result = run_call(
            root,
            spec,
            call_id="redirect",
            role="redirect",
            prompt=_phase_prompt(
                "redirect",
                theorem,
                (root / "prompts/redirect.md").read_text(),
                "ROUTE_ID: route-4\nREDIRECT_BRIEF:\n" + brief,
            ),
            schema_path=root / "schemas/route.schema.json",
            parent_digests={
                "controller": controller_result["receipt"]["final_sha256"]
            },
        )
        calls.append(redirect_result)
        redirect = _require_completed_final(redirect_result, "crouzeix-route/v1")
        if redirect.get("route_id") != "route-4":
            raise ValidationError("redirect must return route-4")
        routes.append(redirect)
        registry.apply(
            _route_event(
                registry,
                spec,
                redirect,
                to_state="independent",
                reason="underexplored redirect output sealed",
                prompt_sha256=redirect_result["receipt"]["prompt_sha256"],
                output_sha256=redirect_result["receipt"]["final_sha256"],
            )
        )
        redirect_state = (
            "blocked"
            if _has_theorem_strength_obligation(redirect)
            or not redirect.get("candidate_proof")
            else "viable"
        )
        registry.apply(
            _route_event(
                registry,
                spec,
                redirect,
                to_state=redirect_state,
                reason="deterministic redirect classification",
                prompt_sha256=redirect_result["receipt"]["prompt_sha256"],
                output_sha256=redirect_result["receipt"]["final_sha256"],
                concrete_artifacts=(
                    redirect.get("proved_statements", [])
                    if redirect_state == "viable"
                    else []
                ),
            )
        )

    selected_ids = controller.get("selected_route_ids")
    if (
        not isinstance(selected_ids, list)
        or len(selected_ids) > 2
        or any(registry.states.get(route_id) != "viable" for route_id in selected_ids)
    ):
        raise ValidationError("controller selected a nonviable or invalid route set")
    viable_routes = [route for route in routes if route["route_id"] in selected_ids]
    if not viable_routes:
        _write_route_events(root, registry)
        return calls, None, "not_promoted"

    synthesis_result = run_call(
        root,
        spec,
        call_id="synthesizer",
        role="synthesizer",
        prompt=_phase_prompt(
            "synthesizer",
            theorem,
            (root / "prompts/synthesizer.md").read_text(),
            "SELECTED_ROUTE_RECORDS:\n" + _canonical_json(viable_routes),
        ),
        schema_path=root / "schemas/synthesis.schema.json",
        parent_digests={
            route["route_id"]: next(
                call["receipt"]["final_sha256"]
                for call in calls
                if call["final"] == route
            )
            for route in viable_routes
        },
    )
    calls.append(synthesis_result)
    synthesis = _require_completed_final(
        synthesis_result, "crouzeix-synthesis/v1"
    )
    if set(synthesis.get("parent_route_ids", [])) != set(selected_ids):
        raise ValidationError("synthesis parent routes do not match controller selection")
    candidate = synthesis.get("candidate_proof")
    if not isinstance(candidate, str) or not candidate.strip():
        raise ValidationError("synthesis returned no candidate proof")
    candidate_path = root / "candidate/candidate.tex"
    candidate_path.write_text(candidate)
    candidate_sha = sha256_bytes(candidate_path.read_bytes())

    for route in viable_routes:
        registry.apply(
            _route_event(
                registry,
                spec,
                route,
                to_state="audited",
                reason="candidate frozen for independent critics",
                prompt_sha256=synthesis_result["receipt"]["prompt_sha256"],
                output_sha256=synthesis_result["receipt"]["final_sha256"],
                candidate_sha256=candidate_sha,
                concrete_artifacts=["frozen standalone candidate"],
                unproved_obligations=_unproved(synthesis.get("obligations", [])),
            )
        )

    critics = []
    for call_id, role, critic_role in (
        ("logical-critic", "logical_critic", "logical"),
        ("operator-critic", "operator_critic", "operator_theory"),
    ):
        critic_result = run_call(
            root,
            spec,
            call_id=call_id,
            role=role,
            prompt=_phase_prompt(
                role,
                theorem,
                (root / "prompts/critic.md").read_text(),
                f"CANDIDATE_SHA256: {candidate_sha}\nCANDIDATE:\n{candidate}",
            ),
            schema_path=root / "schemas/critic.schema.json",
            parent_digests={"candidate": candidate_sha},
        )
        calls.append(critic_result)
        critic = _require_completed_final(critic_result, "crouzeix-critic/v1")
        if (
            critic.get("critic_role") != critic_role
            or critic.get("candidate_sha256") != candidate_sha
        ):
            raise ValidationError("critic identity does not match its assignment")
        critics.append(critic)

    repair_result = run_call(
        root,
        spec,
        call_id="repair",
        role="repair",
        prompt=_phase_prompt(
            "repair",
            theorem,
            (root / "prompts/repair.md").read_text(),
            (
                f"CANDIDATE_SHA256: {candidate_sha}\nCANDIDATE:\n{candidate}\n"
                f"FINDINGS:\n{_canonical_json(critics)}"
            ),
        ),
        schema_path=root / "schemas/repair.schema.json",
        parent_digests={
            "candidate": candidate_sha,
            "logical_critic": calls[-2]["receipt"]["final_sha256"],
            "operator_critic": calls[-1]["receipt"]["final_sha256"],
        },
    )
    calls.append(repair_result)
    repair = _require_completed_final(repair_result, "crouzeix-repair/v1")
    if repair.get("parent_candidate_sha256") != candidate_sha:
        raise ValidationError("repair parent does not match frozen candidate")
    repaired = repair.get("candidate_proof")
    if not isinstance(repaired, str) or not repaired.strip():
        raise ValidationError("repair returned no candidate proof")
    repaired_sha = sha256_bytes(repaired.encode("utf-8"))
    findings = [finding for critic in critics for finding in critic.get("findings", [])]
    dispositions = repair.get("dispositions")
    if not isinstance(dispositions, list):
        raise ValidationError("repair dispositions must be a list")
    disposition_ids = [item.get("finding_id") for item in dispositions]
    finding_ids = [item.get("finding_id") for item in findings]
    if sorted(disposition_ids) != sorted(finding_ids):
        raise ValidationError("repair dispositions do not match critic findings")
    unresolved_critical = sum(
        1
        for finding in findings
        if finding.get("severity") == "critical"
        and next(
            item["disposition"]
            for item in dispositions
            if item["finding_id"] == finding["finding_id"]
        )
        == "unresolved"
    )
    remaining_obligations = repair.get("unproved_obligations")
    if not isinstance(remaining_obligations, list):
        raise ValidationError("repair obligations must be a list")
    changed = repaired_sha != candidate_sha
    promotable = (
        not changed
        and not findings
        and unresolved_critical == 0
        and not _has_theorem_strength(remaining_obligations)
    )
    if changed:
        (root / "candidate/repaired_candidate.tex").write_text(repaired)
    promotion = "promoted" if promotable else "not_promoted"
    for route in viable_routes:
        registry.apply(
            _route_event(
                registry,
                spec,
                route,
                to_state="promoted" if promotable else "rejected",
                reason=(
                    "unchanged candidate survived both critics"
                    if promotable
                    else "candidate requires unresolved or unaudited repair"
                ),
                prompt_sha256=repair_result["receipt"]["prompt_sha256"],
                output_sha256=repair_result["receipt"]["final_sha256"],
                candidate_sha256=candidate_sha,
                concrete_artifacts=["frozen standalone candidate"],
                unproved_obligations=remaining_obligations,
                unresolved_critical_findings=unresolved_critical,
            )
        )
    (root / "review/critic_findings.json").write_text(
        json.dumps(critics, indent=2, sort_keys=True) + "\n"
    )
    (root / "review/repair.json").write_text(
        json.dumps(repair, indent=2, sort_keys=True) + "\n"
    )
    (root / "review/promotion.json").write_text(
        json.dumps(
            {
                "schema_version": "crouzeix-promotion/v1",
                "status": promotion,
                "candidate_sha256": candidate_sha,
                "repair_changed_candidate": changed,
                "unresolved_critical_findings": unresolved_critical,
                "theorem_strength_obligation": _has_theorem_strength(
                    remaining_obligations
                ),
            },
            indent=2,
            sort_keys=True,
        )
        + "\n"
    )
    _write_route_events(root, registry)
    return calls, candidate_sha, promotion


def _route_event(
    registry: RouteRegistry,
    spec: Mapping[str, Any],
    route: Mapping[str, Any],
    *,
    to_state: str,
    reason: str,
    prompt_sha256: str,
    output_sha256: str,
    concrete_artifacts: list[str] | None = None,
    candidate_sha256: str | None = None,
    unproved_obligations: list[dict[str, str]] | None = None,
    unresolved_critical_findings: int = 0,
) -> dict[str, Any]:
    return {
        "schema_version": "crouzeix-route-event/v1",
        "sequence": len(registry.events) + 1,
        "run_id": spec["run_id"],
        "route_id": route["route_id"],
        "family": route["family"],
        "from_state": registry.states.get(route["route_id"]),
        "to_state": to_state,
        "parent_route_ids": [],
        "prompt_sha256": prompt_sha256,
        "output_sha256": output_sha256,
        "candidate_sha256": candidate_sha256,
        "reason": reason,
        "mechanism": route["mechanism"],
        "concrete_artifacts": concrete_artifacts or [],
        "unproved_obligations": unproved_obligations or [],
        "unresolved_critical_findings": unresolved_critical_findings,
        "occurred_at_utc": _now(),
    }


def _require_completed_final(
    result: Mapping[str, Any], schema_version: str
) -> dict[str, Any]:
    if result["receipt"]["status"] != "completed" or result["final"] is None:
        raise ValidationError(
            f"{result['receipt']['call_id']} did not produce a completed final response"
        )
    final = validate_role_output(result["receipt"]["role"], result["final"])
    if final.get("schema_version") != schema_version:
        raise ValidationError(
            f"{result['receipt']['call_id']} returned the wrong schema version"
        )
    return final


def _validate_historical_output(value: Mapping[str, Any]) -> None:
    _closed_fields(value, HISTORICAL_FIELDS, "historical output")
    if value.get("schema_version") != "crouzeix-historical-final/v1":
        raise ValidationError("historical output has invalid schema_version")
    status = value.get("status")
    candidate = value.get("candidate_path")
    if status == "candidate_written" and candidate == "candidate.tex":
        return
    if status == "no_candidate" and candidate is None:
        return
    raise ValidationError("historical output status and candidate_path are inconsistent")


def _validate_route_output(value: Mapping[str, Any]) -> None:
    _closed_fields(value, ROUTE_FIELDS, "route output")
    if value.get("schema_version") != "crouzeix-route/v1":
        raise ValidationError("route output has invalid schema_version")
    _portable_route_id(value.get("route_id"))
    for field in ("family", "mechanism", "confidence_basis"):
        _nonempty_string(value.get(field), field, 12_000)
    _string_array(value.get("proved_statements"), "proved_statements", 32)
    _obligations(value.get("unproved_obligations"), "unproved_obligations")
    _string_array(value.get("circularity_risks"), "circularity_risks", 32)
    candidate = value.get("candidate_proof")
    blocker = value.get("blocker")
    if candidate is not None:
        _nonempty_string(candidate, "candidate_proof", 250_000)
    if blocker is not None:
        _nonempty_string(blocker, "blocker", 12_000)
    if (candidate is None) == (blocker is None):
        raise ValidationError("route output requires exactly one candidate or blocker")


def _validate_controller_output(value: Mapping[str, Any]) -> None:
    _closed_fields(value, CONTROLLER_FIELDS, "controller output")
    if value.get("schema_version") != "crouzeix-controller/v1":
        raise ValidationError("controller output has invalid schema_version")
    decisions = value.get("decisions")
    if not isinstance(decisions, list) or len(decisions) != 3:
        raise ValidationError("controller decisions must contain exactly three entries")
    route_ids = []
    for decision in decisions:
        _closed_fields(
            decision,
            frozenset({"route_id", "family", "state", "reason"}),
            "controller decision",
        )
        route_ids.append(_portable_route_id(decision.get("route_id")))
        _nonempty_string(decision.get("family"), "decision.family", 256)
        if decision.get("state") not in {"blocked", "viable"}:
            raise ValidationError("controller decision has invalid state")
        _nonempty_string(decision.get("reason"), "decision.reason", 4_000)
    if len(set(route_ids)) != 3:
        raise ValidationError("controller decisions require unique route IDs")
    selected = value.get("selected_route_ids")
    if (
        not isinstance(selected, list)
        or len(selected) > 2
        or len(set(selected)) != len(selected)
        or any(route_id not in route_ids for route_id in selected)
    ):
        raise ValidationError("controller selected_route_ids are invalid")
    if not isinstance(value.get("family_collapse"), bool):
        raise ValidationError("controller family_collapse must be boolean")
    brief = value.get("redirect_brief")
    if brief is not None:
        _nonempty_string(brief, "redirect_brief", 8_000)
    if value["family_collapse"] != (brief is not None):
        raise ValidationError("controller redirect brief is inconsistent")


def _validate_synthesis_output(value: Mapping[str, Any]) -> None:
    _closed_fields(value, SYNTHESIS_FIELDS, "synthesis output")
    if value.get("schema_version") != "crouzeix-synthesis/v1":
        raise ValidationError("synthesis output has invalid schema_version")
    parents = value.get("parent_route_ids")
    if (
        not isinstance(parents, list)
        or not 1 <= len(parents) <= 3
        or len(set(parents)) != len(parents)
    ):
        raise ValidationError("synthesis parent_route_ids are invalid")
    for route_id in parents:
        _portable_route_id(route_id)
    _nonempty_string(value.get("candidate_proof"), "candidate_proof", 250_000)
    obligations = value.get("obligations")
    if not isinstance(obligations, list) or len(obligations) > 64:
        raise ValidationError("synthesis obligations must be a bounded list")
    for obligation in obligations:
        _closed_fields(
            obligation,
            frozenset({"statement", "strength", "status"}),
            "synthesis obligation",
        )
        _one_obligation(obligation)
        if obligation.get("status") not in {"proved", "unproved"}:
            raise ValidationError("synthesis obligation status is invalid")


def _validate_critic_output(value: Mapping[str, Any]) -> None:
    _closed_fields(value, CRITIC_FIELDS, "critic output")
    if value.get("schema_version") != "crouzeix-critic/v1":
        raise ValidationError("critic output has invalid schema_version")
    if value.get("critic_role") not in {"logical", "operator_theory"}:
        raise ValidationError("critic_role is invalid")
    _sha256(value.get("candidate_sha256"), "candidate_sha256")
    findings = value.get("findings")
    if not isinstance(findings, list) or len(findings) > 64:
        raise ValidationError("critic findings must be a bounded list")
    seen = set()
    for finding in findings:
        _closed_fields(
            finding,
            frozenset({"finding_id", "severity", "locator", "statement", "test"}),
            "critic finding",
        )
        finding_id = _nonempty_string(finding.get("finding_id"), "finding_id", 64)
        if finding_id in seen:
            raise ValidationError("critic finding IDs must be unique")
        seen.add(finding_id)
        if finding.get("severity") not in {"critical", "major", "minor"}:
            raise ValidationError("finding severity is invalid")
        for field in ("locator", "statement", "test"):
            _nonempty_string(finding.get(field), field, 8_000)


def _validate_repair_output(value: Mapping[str, Any]) -> None:
    _closed_fields(value, REPAIR_FIELDS, "repair output")
    if value.get("schema_version") != "crouzeix-repair/v1":
        raise ValidationError("repair output has invalid schema_version")
    _sha256(value.get("parent_candidate_sha256"), "parent_candidate_sha256")
    _nonempty_string(value.get("candidate_proof"), "candidate_proof", 250_000)
    dispositions = value.get("dispositions")
    if not isinstance(dispositions, list) or len(dispositions) > 128:
        raise ValidationError("repair dispositions must be a bounded list")
    seen = set()
    for disposition in dispositions:
        _closed_fields(
            disposition,
            frozenset({"finding_id", "disposition", "reason"}),
            "repair disposition",
        )
        finding_id = _nonempty_string(
            disposition.get("finding_id"), "finding_id", 64
        )
        if finding_id in seen:
            raise ValidationError("repair disposition IDs must be unique")
        seen.add(finding_id)
        if disposition.get("disposition") not in {
            "fixed",
            "rejected_with_reason",
            "unresolved",
        }:
            raise ValidationError("repair disposition is invalid")
        _nonempty_string(disposition.get("reason"), "disposition.reason", 8_000)
    _obligations(value.get("unproved_obligations"), "unproved_obligations")


def _closed_fields(
    value: Any, expected: frozenset[str], label: str
) -> None:
    if not isinstance(value, dict) or frozenset(value) != expected:
        actual = sorted(value) if isinstance(value, dict) else type(value).__name__
        raise ValidationError(
            f"{label} fields must be {sorted(expected)}, got {actual}"
        )


def _nonempty_string(value: Any, label: str, maximum: int) -> str:
    if (
        not isinstance(value, str)
        or not value.strip()
        or len(value) > maximum
        or "\0" in value
    ):
        raise ValidationError(f"{label} must be a bounded nonempty string")
    return value


def _string_array(value: Any, label: str, maximum: int) -> list[str]:
    if not isinstance(value, list) or len(value) > maximum:
        raise ValidationError(f"{label} must be a bounded list")
    return [_nonempty_string(item, f"{label} item", 8_000) for item in value]


def _portable_route_id(value: Any) -> str:
    route_id = _nonempty_string(value, "route_id", 64)
    if not route_id.startswith("route-") or not route_id[6:].isdigit():
        raise ValidationError("route_id must match route-N")
    return route_id


def _obligations(value: Any, label: str) -> None:
    if not isinstance(value, list) or len(value) > 64:
        raise ValidationError(f"{label} must be a bounded list")
    for obligation in value:
        _closed_fields(
            obligation,
            frozenset({"statement", "strength"}),
            "unproved obligation",
        )
        _one_obligation(obligation)


def _one_obligation(value: Mapping[str, Any]) -> None:
    _nonempty_string(value.get("statement"), "obligation.statement", 4_000)
    if value.get("strength") not in {"local", "major", "theorem_strength"}:
        raise ValidationError("obligation strength is invalid")


def _sha256(value: Any, label: str) -> str:
    if (
        not isinstance(value, str)
        or len(value) != 64
        or any(character not in "0123456789abcdef" for character in value)
    ):
        raise ValidationError(f"{label} must be a lowercase SHA-256")
    return value


def _phase_prompt(role: str, theorem: str, contract: str, payload: str) -> str:
    return (
        f"CALL_ROLE: {role}\n\n"
        f"THEOREM:\n{theorem}\n\n"
        f"PHASE_CONTRACT:\n{contract}\n\n"
        f"{payload}\n"
    )


def _canonical_json(value: Any) -> str:
    return json.dumps(value, sort_keys=True, separators=(",", ":"))


def _freeze_candidate(source: Path, destination: Path) -> str:
    path = _bounded_regular_file(source, "candidate", MAX_CANDIDATE_BYTES)
    data = path.read_bytes()
    if destination.exists() or destination.is_symlink():
        raise ValidationError("candidate destination already exists")
    destination.write_bytes(data)
    (destination.parent / "candidate.sha256").write_text(sha256_bytes(data) + "\n")
    return sha256_bytes(data)


def _mark_malformed(result: Mapping[str, Any], message: str) -> None:
    receipt = dict(result["receipt"])
    receipt["status"] = "malformed"
    receipt["parse_error"] = message
    result["receipt"].clear()
    result["receipt"].update(receipt)
    (result["call_dir"] / "receipt.json").write_text(
        json.dumps(receipt, indent=2, sort_keys=True) + "\n"
    )


def _unproved(obligations: Any) -> list[dict[str, str]]:
    if not isinstance(obligations, list):
        raise ValidationError("obligations must be a list")
    return [
        {"statement": item["statement"], "strength": item["strength"]}
        for item in obligations
        if item.get("status") == "unproved"
    ]


def _has_theorem_strength_obligation(route: Mapping[str, Any]) -> bool:
    return _has_theorem_strength(route.get("unproved_obligations", []))


def _has_theorem_strength(obligations: Any) -> bool:
    return isinstance(obligations, list) and any(
        isinstance(item, dict) and item.get("strength") == "theorem_strength"
        for item in obligations
    )


def _write_route_events(root: Path, registry: RouteRegistry) -> None:
    (root / "route_events.jsonl").write_text(
        "".join(_canonical_json(event) + "\n" for event in registry.events)
    )


def _parse_usage(raw: Any) -> dict[str, int | None]:
    if not isinstance(raw, dict):
        raise ValidationError("turn usage must be an object")
    usage = {}
    for field in USAGE_FIELDS:
        value = raw.get(field)
        if value is not None and (
            not isinstance(value, int) or isinstance(value, bool) or value < 0
        ):
            raise ValidationError(f"usage field {field} is invalid")
        usage[field] = value
    return usage


def _verify_cli_identity(spec: Mapping[str, Any]) -> None:
    path = _regular_file(Path(str(spec["cli"]["path"])), "TRAE CLI executable")
    if not os.access(path, os.X_OK):
        raise ValidationError("TRAE CLI executable is not executable")
    observed = sha256_bytes(path.read_bytes())
    if observed != spec["cli"]["sha256"]:
        raise ValidationError("TRAE CLI executable identity changed after preparation")


def _regular_file(path: Path, label: str) -> Path:
    try:
        metadata = path.lstat()
    except OSError as error:
        raise ValidationError(f"cannot inspect {label}: {error}") from error
    if stat.S_ISLNK(metadata.st_mode) or not stat.S_ISREG(metadata.st_mode):
        raise ValidationError(f"{label} must be a regular non-symlink file")
    return path.resolve()


def _bounded_regular_file(path: Path, label: str, maximum: int) -> Path:
    regular = _regular_file(path, label)
    if regular.stat().st_size > maximum:
        raise ValidationError(f"{label} exceeds byte cap")
    return regular


def _portable_call_id(value: str) -> str:
    if (
        not value
        or len(value) > 128
        or not value[0].islower()
        or any(character not in "abcdefghijklmnopqrstuvwxyz0123456789-" for character in value)
    ):
        raise ValidationError("call_id must be a portable lowercase ID")
    return value


def _terminate_process_group(child: subprocess.Popen[bytes]) -> None:
    try:
        os.killpg(child.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        child.wait(timeout=2)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(child.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        child.wait()


def _minimal_environment() -> dict[str, str]:
    allowed = (
        "HOME",
        "PATH",
        "TMPDIR",
        "LANG",
        "LC_ALL",
        "SSL_CERT_FILE",
        "SSL_CERT_DIR",
        "TRAE_HOME",
    )
    environment = {key: os.environ[key] for key in allowed if key in os.environ}
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    return environment


def _now() -> str:
    return datetime.now(timezone.utc).isoformat().replace("+00:00", "Z")
