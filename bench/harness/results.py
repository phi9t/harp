"""Result records: schema, validation, aggregation.

`provenance` says which groups were measured by the harness and which are unmeasured; nothing an agent
reports about itself is stored as a measurement.
"""

from __future__ import annotations

import json
import math
import statistics
import tomllib
from datetime import datetime, timezone
from pathlib import Path

SCHEMA = "corenius.bench.result.v1"
READINESS_SCHEMA = "corenius.bench.readiness_result.v1"
READINESS_RESULT_FILE = "readiness-results.jsonl"
READINESS_INSTANTIATIONS = ("lead-driven", "local-controller", "workflow-controller")
READINESS_OUTCOMES = ("passes", "passes with intervention", "not applicable by design", "fails")
READINESS_PROOFS = ("deterministic", "live")
CORE_STORIES = frozenset({1, 2, 5, 6, 7, 9, 12, 14, 15, 16, 17, 18, 20, 27, 34, 35})
LEGACY_CONTROLLER_SCHEMA = "corenius.bench.controller.v1"
CONTROLLER_SCHEMA = "corenius.bench.controller.v2"
CONTROLLER_SCHEMAS = {LEGACY_CONTROLLER_SCHEMA, CONTROLLER_SCHEMA}
TASK_VERDICTS = {"pass", "partial_pass", "fail", "unknown", "not_run"}
ORCHESTRATION_VERDICTS = {"pass", "fail", "unknown"}
EFFECT_DISPOSITIONS = {"resolved", "unresolved", "unknown"}
USAGE_STATUSES = {"measured", "partial", "unknown", "not_applicable"}
PREFLIGHT_FAILURE_STAGES = {"missing_prerequisite", "unsupported_task", "preflight"}
DISPATCH_FAILURE_STAGES = {"timeout", "provider_failure"}


def _num(value, name, allow_none=False):
    if value is None and allow_none:
        return
    if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
        raise ValueError(f"{name} must be a finite number")


def _bool(value, name):
    if not isinstance(value, bool):
        raise ValueError(f"{name} must be a bool")


def _str_or_none(value, name):
    if value is not None and not isinstance(value, str):
        raise ValueError(f"{name} must be a string or null")


def _validate_controller_v2(record: dict, controller: dict) -> None:
    window = controller.get("execution_window")
    if not isinstance(window, dict):
        raise ValueError("controller.execution_window must be a dict")
    for key in (
        "deadline_seconds",
        "setup_timeout_seconds",
        "cleanup_timeout_seconds",
        "pinned_verifier_timeout_seconds",
        "durable_run_admission_unix",
        "first_mission_submission_unix",
        "dispatch_cutoff_unix",
    ):
        _num(window.get(key), f"controller.execution_window.{key}")
    if window["deadline_seconds"] != 1800.0:
        raise ValueError("controller.execution_window.deadline_seconds must be 1800.0")
    if abs(window["dispatch_cutoff_unix"] - (window["durable_run_admission_unix"] + window["deadline_seconds"])) > 0.001:
        raise ValueError("controller.execution_window.dispatch_cutoff_unix must be admission plus the deadline")
    if window["first_mission_submission_unix"] < window["durable_run_admission_unix"]:
        raise ValueError("controller.execution_window.first_mission_submission_unix must follow durable admission")
    for key in ("failed_preflight_consumed_model_calls", "automatic_rerun_after_timeout_or_provider_failure"):
        _bool(window.get(key), f"controller.execution_window.{key}")
    if window["automatic_rerun_after_timeout_or_provider_failure"]:
        raise ValueError("controller execution must not automatically rerun after timeout or provider failure")
    for key in ("clock", "live_host_requirement", "platform"):
        if not isinstance(window.get(key), str):
            raise ValueError(f"controller.execution_window.{key} must be a string")
    if window["clock"] != "durable_admission_to_final_verification":
        raise ValueError("controller.execution_window.clock must be durable_admission_to_final_verification")
    if window["platform"] != "linux":
        raise ValueError("controller.execution_window.platform must be linux")
    if window["live_host_requirement"] != "m01":
        raise ValueError("controller.execution_window.live_host_requirement must be m01")

    outcomes = controller.get("outcomes")
    if not isinstance(outcomes, dict):
        raise ValueError("controller.outcomes must be a dict")
    task = outcomes.get("task")
    orchestration = outcomes.get("orchestration")
    effects = outcomes.get("effects")
    if not isinstance(task, dict) or task.get("verdict") not in TASK_VERDICTS:
        raise ValueError("controller.outcomes.task.verdict is unsupported")
    if not isinstance(orchestration, dict) or orchestration.get("verdict") not in ORCHESTRATION_VERDICTS:
        raise ValueError("controller.outcomes.orchestration.verdict is unsupported")
    if not isinstance(effects, dict) or effects.get("disposition") not in EFFECT_DISPOSITIONS:
        raise ValueError("controller.outcomes.effects.disposition is unsupported")
    if record["outcome"].get("task_verdict") != task["verdict"]:
        raise ValueError("outcome.task_verdict must match controller task outcome")
    if record["outcome"].get("orchestration_verdict") != orchestration["verdict"]:
        raise ValueError("outcome.orchestration_verdict must match controller orchestration outcome")
    if record["outcome"].get("effect_disposition") != effects["disposition"]:
        raise ValueError("outcome.effect_disposition must match controller effect disposition")

    failure = controller.get("failure")
    if not isinstance(failure, dict):
        raise ValueError("controller.failure must be a dict")
    for key in ("stage", "kind", "message"):
        _str_or_none(failure.get(key), f"controller.failure.{key}")
    if record["outcome"].get("failure_stage") != failure.get("stage"):
        raise ValueError("outcome.failure_stage must match controller failure stage")
    if failure.get("stage") in PREFLIGHT_FAILURE_STAGES and window["failed_preflight_consumed_model_calls"]:
        raise ValueError("failed preflight must not consume model calls")
    if failure.get("stage") in DISPATCH_FAILURE_STAGES:
        if controller.get("attempts", {}).get("automatic_retries_after_failure") != 0:
            raise ValueError("timeout/provider failure must not automatically retry")
        if effects["disposition"] == "resolved":
            disposition = controller.get("process_disposition", {})
            reconciliation = disposition.get("reconciliation") if isinstance(disposition, dict) else None
            if not isinstance(reconciliation, dict):
                raise ValueError("resolved timeout/provider failure requires cleanup reconciliation proof")
            if reconciliation.get("status") != "resolved":
                raise ValueError("resolved timeout/provider failure requires resolved cleanup reconciliation")
            if reconciliation.get("authority") != "controller.cancel_attempt":
                raise ValueError("resolved timeout/provider failure requires controller cancellation authority")
            if reconciliation.get("bounded") is not True or reconciliation.get("identity_bound") is not True:
                raise ValueError("resolved timeout/provider failure requires bounded identity-bound cleanup")
            if disposition.get("remaining_uncertain"):
                raise ValueError("resolved timeout/provider failure cannot retain uncertain effects")
            if disposition.get("owned_attempts") not in ("stopped", "not_started", "resolved"):
                raise ValueError("resolved timeout/provider failure requires stopped owned attempts")

    autonomous_success = (
        failure.get("stage") is None
        and task["verdict"] in {"pass", "partial_pass"}
        and orchestration["verdict"] == "pass"
        and effects["disposition"] == "resolved"
    )
    if record["outcome"].get("passed") and not autonomous_success:
        raise ValueError("autonomous PASS contradicts controller failure or verdict state")
    if record["outcome"].get("reward") > 0 and not record["outcome"].get("passed"):
        raise ValueError("positive reward requires a passing outcome")

    attempts = controller.get("attempts")
    if not isinstance(attempts, dict):
        raise ValueError("controller.attempts must be a dict")
    for key in (
        "total",
        "fresh_integration",
        "max_fresh_integration_attempts",
        "automatic_retries_after_failure",
        "max_concurrent_worker_executions",
    ):
        _num(attempts.get(key), f"controller.attempts.{key}")
    if attempts["fresh_integration"] > attempts["max_fresh_integration_attempts"]:
        raise ValueError("controller attempts exceed the fresh integration limit")
    if attempts["automatic_retries_after_failure"] != 0:
        raise ValueError("controller automatic failure retries must be zero")
    if record["coordination"].get("attempts") != attempts["total"]:
        raise ValueError("coordination.attempts must match controller attempts")

    interventions = controller.get("interventions")
    if not isinstance(interventions, list) or not all(isinstance(item, dict) for item in interventions):
        raise ValueError("controller.interventions must be a list of dicts")
    manual_interventions = record["coordination"].get("manual_interventions")
    _num(manual_interventions, "coordination.manual_interventions")
    if manual_interventions != len(interventions):
        raise ValueError("coordination.manual_interventions must match controller interventions")
    manual_rescue = any(str(item.get("kind", "")).startswith("manual") for item in interventions)
    if manual_rescue:
        if record["outcome"]["passed"]:
            raise ValueError("manual intervention cannot yield autonomous PASS")
        if controller.get("autonomous_pass_eligible") is not False:
            raise ValueError("manual intervention must mark autonomous pass ineligible")

    usage = controller.get("model_usage")
    if not isinstance(usage, dict) or usage.get("status") not in USAGE_STATUSES:
        raise ValueError("controller.model_usage.status is unsupported")
    _bool(usage.get("complete"), "controller.model_usage.complete")
    for key in ("input_tokens", "output_tokens", "usd"):
        _num(usage.get(key), f"controller.model_usage.{key}", allow_none=True)
    if usage["status"] in ("unknown", "not_applicable"):
        for key in ("input_tokens", "output_tokens", "usd"):
            if usage.get(key) is not None:
                raise ValueError(f"controller.model_usage.{key} must be unknown")
    if usage["status"] == "measured" and (
        usage.get("input_tokens") is None or usage.get("output_tokens") is None or not usage["complete"]
    ):
        raise ValueError("measured controller usage must include complete token counts")

    retained = controller.get("retained_evidence")
    if not isinstance(retained, dict) or not isinstance(retained.get("run_root"), str):
        raise ValueError("controller.retained_evidence.run_root must be a string")
    for key in ("repositories", "manifests", "events", "proofs", "logs"):
        if not isinstance(retained.get(key), list) or not all(isinstance(item, dict) for item in retained[key]):
            raise ValueError(f"controller.retained_evidence.{key} must be a list of dicts")
    if not any(retained[key] for key in ("repositories", "manifests", "events", "proofs", "logs")):
        raise ValueError("controller.retained_evidence must retain at least one identity")

    if bool(controller.get("orchestration_passed")) != (orchestration["verdict"] == "pass"):
        raise ValueError("controller.orchestration_passed must match orchestration verdict")


def validate(record: dict) -> dict:
    if record.get("schema_version") != SCHEMA:
        raise ValueError("unsupported schema_version")
    for key, typ in (("run_id", str), ("task_id", str), ("condition", str), ("agent", dict), ("capsule", dict),
                     ("outcome", dict), ("timing", dict), ("coordination", dict), ("cost", dict), ("provenance", dict)):
        if not isinstance(record.get(key), typ):
            raise ValueError(f"{key} must be {typ.__name__}")
    condition = record["condition"]
    if not (condition == "solo" or condition.startswith("fleet:") or condition.startswith("controller:")):
        raise ValueError("condition must be solo, fleet:N, or controller:TICKET")
    for key in ("digest", "net"):
        if not isinstance(record["capsule"].get(key), str):
            raise ValueError(f"capsule.{key} must be a string")
    out = record["outcome"]
    if not isinstance(out.get("passed"), bool):
        raise ValueError("outcome.passed must be a bool")
    _num(out.get("reward"), "outcome.reward")
    for key in ("wall_seconds", "agent_seconds", "verify_seconds"):
        _num(record["timing"].get(key), f"timing.{key}")
    for key in ("conflicts", "redos", "contamination", "tickets", "tickets_landed"):
        _num(record["coordination"].get(key), f"coordination.{key}")
    for key in ("usd", "input_tokens", "output_tokens"):
        _num(record["cost"].get(key), f"cost.{key}", allow_none=True)
    if condition.startswith("controller:"):
        controller = record.get("controller")
        if not isinstance(controller, dict):
            raise ValueError("controller must be a dict")
        if controller.get("schema_version") not in CONTROLLER_SCHEMAS:
            raise ValueError("controller.schema_version must be a supported controller schema")
        for key in ("mode", "task", "ticket"):
            if not isinstance(controller.get(key), str):
                raise ValueError(f"controller.{key} must be a string")
        if controller["schema_version"] == LEGACY_CONTROLLER_SCHEMA and controller["mode"] != "deterministic":
            raise ValueError("controller.mode must be deterministic")
        if controller["schema_version"] == CONTROLLER_SCHEMA and controller["mode"] not in ("deterministic", "live"):
            raise ValueError("controller.mode must be deterministic or live")
        expected_ticket = condition.split(":", 1)[1]
        if controller["task"] != record["task_id"] or controller["ticket"] != expected_ticket:
            raise ValueError("controller task/ticket must match the result condition")
        partial = controller.get("partial_contract")
        if not isinstance(partial, dict):
            raise ValueError("controller.partial_contract must be a dict")
        for key in ("suite_id", "suite_digest", "noop_verdict", "fixed_verdict"):
            if not isinstance(partial.get(key), str):
                raise ValueError(f"controller.partial_contract.{key} must be a string")
        if not isinstance(partial.get("selected_tests"), list) or not all(isinstance(item, str) for item in partial["selected_tests"]):
            raise ValueError("controller.partial_contract.selected_tests must be a list of strings")
        landing = controller.get("landing")
        if not isinstance(landing, dict) or not isinstance(landing.get("state"), str):
            raise ValueError("controller.landing.state must be a string")
        if not isinstance(controller.get("process_disposition"), dict):
            raise ValueError("controller.process_disposition must be a dict")
        source = controller.get("source")
        if not isinstance(source, dict):
            raise ValueError("controller.source must be a dict")
        manifest = source.get("content_manifest")
        if not isinstance(manifest, dict) or manifest.get("schema_version") != "corenius.bench.source-manifest.v1":
            raise ValueError("controller.source.content_manifest must be a source manifest")
        if not isinstance(manifest.get("digest"), str):
            raise ValueError("controller.source.content_manifest.digest must be a string")
        if not isinstance(manifest.get("entries"), list):
            raise ValueError("controller.source.content_manifest.entries must be a list")
        closure = controller.get("runtime_closure")
        if not isinstance(closure, dict):
            raise ValueError("controller.runtime_closure must be a dict")
        if closure.get("status") not in ("pinned", "limited"):
            raise ValueError("controller.runtime_closure.status must be pinned or limited")
        if not isinstance(closure.get("pinned"), dict):
            raise ValueError("controller.runtime_closure.pinned must be a dict")
        if closure["status"] == "pinned":
            runtime = closure["pinned"].get("verifier_runtime")
            if not isinstance(runtime, dict):
                raise ValueError("controller.runtime_closure.pinned.verifier_runtime must be a dict")
            manifest = runtime.get("manifest")
            if not isinstance(manifest, dict) or not isinstance(manifest.get("sha256"), str):
                raise ValueError("controller verifier runtime manifest must be pinned")
        for key in ("build_seconds", "setup_seconds", "controller_flow_seconds", "cleanup_seconds"):
            _num(record["timing"].get(key), f"timing.{key}")
        if controller["schema_version"] == CONTROLLER_SCHEMA:
            _num(record["timing"].get("execution_seconds"), "timing.execution_seconds")
            if record["timing"]["execution_seconds"] != record["timing"]["controller_flow_seconds"]:
                raise ValueError("timing.execution_seconds must match controller_flow_seconds")
            _validate_controller_v2(record, controller)
    return record


def load_all(root: Path) -> list[dict]:
    records = []
    for path in sorted(Path(root).rglob("result.json")):
        records.append(validate(json.loads(path.read_text())))
    return records


def wilson(k: int, n: int, z: float = 1.96) -> tuple[float, float]:
    if n == 0:
        return (0.0, 1.0)
    p = k / n
    denom = 1 + z * z / n
    centre = (p + z * z / (2 * n)) / denom
    half = z * math.sqrt(p * (1 - p) / n + z * z / (4 * n * n)) / denom
    return (max(0.0, centre - half), min(1.0, centre + half))


def summarize(records: list[dict]) -> list[dict]:
    groups: dict[tuple, list[dict]] = {}
    for r in records:
        groups.setdefault((r["task_id"], r["condition"], r["agent"]["spec"]), []).append(r)
    rows = []
    for (task, condition, agent), rs in sorted(groups.items()):
        passed = sum(r["outcome"]["passed"] for r in rs)
        lo, hi = wilson(passed, len(rs))
        walls = [r["timing"]["wall_seconds"] for r in rs]
        rows.append({
            "task": task, "condition": condition, "agent": agent, "n": len(rs), "passed": passed,
            "pass_rate": round(passed / len(rs), 3), "ci95": [round(lo, 3), round(hi, 3)],
            "wall_median": round(statistics.median(walls), 2),
            "wall_stdev": round(statistics.pstdev(walls), 2) if len(walls) > 1 else 0.0,
            "tokens_out_median": (statistics.median([r["cost"]["output_tokens"] for r in rs if r["cost"]["output_tokens"] is not None])
                                  if any(r["cost"]["output_tokens"] is not None for r in rs) else None),
            "redos": sum(r["coordination"]["redos"] for r in rs),
            "conflicts": sum(r["coordination"]["conflicts"] for r in rs),
            "stale_refusals": sum(r["coordination"].get("stale_refusals", 0) for r in rs),
            "contamination": sum(r["coordination"]["contamination"] for r in rs),
            "capsules": sorted({r["capsule"]["digest"][:12] for r in rs}),
        })
    return rows


def table(rows: list[dict]) -> str:
    condition_width = max([9, *(len(r["condition"]) for r in rows)])
    head = (f"{'task':30s} {'condition':{condition_width}s} {'agent':16s} {'n':>3s} {'pass':>6s} {'ci95':>13s} {'wall':>7s} {'confl':>5s} "
            f"{'stale':>5s} {'redo':>4s} {'contam':>6s} {'tok_out':>8s}")
    lines = [head, "-" * len(head)]
    for r in rows:
        lines.append(f"{r['task']:30s} {r['condition']:{condition_width}s} {r['agent'][:16]:16s} {r['n']:3d} {r['pass_rate']:6.2f} "
                     f"[{r['ci95'][0]:.2f},{r['ci95'][1]:.2f}] {r['wall_median']:7.1f} {r['conflicts']:5d} {r['stale_refusals']:5d} {r['redos']:4d} "
                     f"{r['contamination']:6d} {'-' if r['tokens_out_median'] is None else int(r['tokens_out_median']):>8}")
    return "\n".join(lines)


def validate_readiness_row(row: dict) -> dict:
    if row.get("schema_version") != READINESS_SCHEMA:
        raise ValueError("unsupported readiness schema_version")
    for key in ("recorded_at", "scenario", "instantiation", "runtime", "proof", "outcome"):
        if not isinstance(row.get(key), str) or not row.get(key):
            raise ValueError(f"{key} must be a non-empty string")
    requested_runtime = row.get("requested_runtime")
    if requested_runtime is not None and (not isinstance(requested_runtime, str) or not requested_runtime):
        raise ValueError("requested_runtime must be a non-empty string or null")
    if row["instantiation"] not in READINESS_INSTANTIATIONS:
        raise ValueError("instantiation must be one of the readiness instantiations")
    if row["proof"] not in READINESS_PROOFS:
        raise ValueError("proof must be deterministic or live")
    if row["outcome"] not in READINESS_OUTCOMES:
        raise ValueError("outcome must be a readiness outcome")
    reason = row.get("reason")
    if reason is not None and not isinstance(reason, str):
        raise ValueError("reason must be a string or null")
    if row["outcome"] == "not applicable by design" and not reason:
        raise ValueError("not applicable by design rows require a reason")
    interventions = row.get("interventions")
    if not isinstance(interventions, list):
        raise ValueError("interventions must be a list")
    for intervention in interventions:
        if not isinstance(intervention, dict):
            raise ValueError("intervention must be an object")
        for key in ("who", "what"):
            if not isinstance(intervention.get(key), str) or not intervention.get(key).strip():
                raise ValueError(f"intervention.{key} must be a non-empty string")
    _num(row.get("elapsed_seconds"), "elapsed_seconds")
    usage = row.get("model_usage")
    if usage != "unknown" and not isinstance(usage, dict):
        raise ValueError("model_usage must be unknown or an object")
    participants = row.setdefault("participants", [])
    if not isinstance(participants, list) or any(not isinstance(item, str) or not item for item in participants):
        raise ValueError("participants must be a list of non-empty strings")
    stories = row.setdefault("stories", [])
    if not isinstance(stories, list) or any(isinstance(story, bool) or not isinstance(story, int) for story in stories):
        raise ValueError("stories must be a list of integers")
    return row


def readiness_row(
    *,
    scenario: str,
    instantiation: str,
    runtime: str,
    proof: str,
    outcome: str,
    interventions: list[dict] | None = None,
    elapsed_seconds: float = 0.0,
    model_usage="unknown",
    reason: str | None = None,
    recorded_at: str | None = None,
    participants: list[str] | None = None,
    stories: list[int] | None = None,
    requested_runtime: str | None = None,
) -> dict:
    return validate_readiness_row({
        "schema_version": READINESS_SCHEMA,
        "recorded_at": recorded_at or datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "scenario": scenario,
        "instantiation": instantiation,
        "runtime": runtime,
        "proof": proof,
        "outcome": outcome,
        "reason": reason,
        "interventions": interventions or [],
        "elapsed_seconds": round(float(elapsed_seconds), 3),
        "model_usage": model_usage,
        "participants": participants or [],
        "stories": sorted(set(stories or [])),
        "requested_runtime": requested_runtime,
    })


def append_readiness_row(root: Path, row: dict) -> Path:
    root.mkdir(parents=True, exist_ok=True)
    path = root / READINESS_RESULT_FILE
    with path.open("a") as fh:
        fh.write(json.dumps(validate_readiness_row(row), sort_keys=True) + "\n")
    return path


def load_readiness_rows(root: Path) -> list[dict]:
    root = Path(root)
    paths = [root] if root.is_file() else sorted(root.rglob(READINESS_RESULT_FILE))
    rows = []
    for path in paths:
        if not path.exists():
            continue
        for index, line in enumerate(path.read_text().splitlines(), start=1):
            if line.strip():
                try:
                    rows.append(validate_readiness_row(json.loads(line)))
                except (json.JSONDecodeError, ValueError) as error:
                    raise ValueError(f"{path}:{index}: {error}") from error
    return rows


def readiness_scenarios(readiness_dir: Path) -> list[dict]:
    scenarios = []
    for path in sorted(Path(readiness_dir).glob("*/scenario.toml")):
        cfg = tomllib.loads(path.read_text())
        sid = cfg.get("id") or path.parent.name
        stories = cfg.get("stories")
        if stories is not None:
            if not isinstance(stories, list) or any(isinstance(story, bool) or not isinstance(story, int) for story in stories):
                raise ValueError(f"{path}: stories must be a list of integers")
        scenarios.append({"id": sid, "stories": stories})
    return scenarios


def _display(value) -> str:
    if value in (None, ""):
        return "-"
    if isinstance(value, dict):
        return json.dumps(value, sort_keys=True)
    return str(value)


def _format_interventions(interventions: list[dict]) -> str:
    if not interventions:
        return "-"
    return "; ".join(f"{item['who']}: {item['what']}" for item in interventions)


def _latest_readiness(rows: list[dict]) -> dict[tuple[str, str, str], dict]:
    latest = {}
    for row in rows:
        latest[(row["scenario"], row["proof"], row["instantiation"])] = row
    return latest


def _row_stories(row: dict, scenario: dict) -> set[int]:
    if "stories" in row:
        return set(row.get("stories") or [])
    return set(scenario["stories"] or [])


def _core_counts(scenarios: list[dict], latest: dict[tuple[str, str, str], dict], proof: str, instantiation: str) -> dict[str, int]:
    counts = {outcome: 0 for outcome in READINESS_OUTCOMES}
    counts["not yet run"] = 0
    core = [scenario for scenario in scenarios if scenario["stories"] is not None and set(scenario["stories"]) & CORE_STORIES]
    for scenario in core:
        row = latest.get((scenario["id"], proof, instantiation))
        if row is None or not (_row_stories(row, scenario) & CORE_STORIES):
            counts["not yet run"] += 1
        else:
            counts[row["outcome"]] += 1
    return counts


def _core_story_counts(scenarios: list[dict], latest: dict[tuple[str, str, str], dict], proof: str, instantiation: str) -> dict[str, list[int]]:
    status: dict[int, str] = {story: "not yet run" for story in sorted(CORE_STORIES)}
    rank = {
        "not yet run": 0,
        "not applicable by design": 1,
        "passes": 2,
        "passes with intervention": 3,
        "fails": 4,
    }
    for scenario in scenarios:
        row = latest.get((scenario["id"], proof, instantiation))
        if row is None:
            continue
        for story in _row_stories(row, scenario) & CORE_STORIES:
            current = status[story]
            outcome = row["outcome"]
            if rank[outcome] >= rank[current]:
                status[story] = outcome
    out = {key: [] for key in ["passes", "passes with intervention", "fails", "not yet run", "not applicable by design"]}
    for story, outcome in sorted(status.items()):
        out[outcome].append(story)
    return out


def _list_numbers(values: list[int]) -> str:
    return ",".join(str(value) for value in values) or "-"


def readiness_report(root: Path, readiness_dir: Path) -> str:
    scenarios = readiness_scenarios(readiness_dir)
    rows = load_readiness_rows(root)
    latest = _latest_readiness(rows)
    proofs = sorted({"deterministic", *(row["proof"] for row in rows)}, key=READINESS_PROOFS.index)
    lines = []
    for scenario in scenarios:
        for proof in proofs:
            lines.append(f"Scenario: {scenario['id']}")
            lines.append(
                "Stories: "
                + ("not declared" if scenario["stories"] is None else ", ".join(str(story) for story in scenario["stories"]))
            )
            lines.append(f"Proof: {proof}")
            lines.append("| instantiation | runtime | requested runtime | outcome | elapsed | model usage | interventions | reason | participants | stories run |")
            lines.append("| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |")
            for instantiation in READINESS_INSTANTIATIONS:
                row = latest.get((scenario["id"], proof, instantiation))
                if row is None:
                    lines.append(f"| {instantiation} | - | - | not yet run | - | - | - | - | - | - |")
                    continue
                elapsed = f"{row['elapsed_seconds']:.3f}s"
                lines.append(
                    f"| {instantiation} | {row['runtime']} | {_display(row.get('requested_runtime'))} | {row['outcome']} | {elapsed} | "
                    f"{_display(row['model_usage'])} | {_format_interventions(row['interventions'])} | "
                    f"{_display(row.get('reason'))} | {', '.join(row.get('participants') or []) or '-'} | "
                    f"{_list_numbers(row.get('stories') or [])} |"
                )
            lines.append("")
    for proof in proofs:
        for instantiation in READINESS_INSTANTIATIONS:
            counts = _core_counts(scenarios, latest, proof, instantiation)
            lines.append(
                f"Core set ({proof}, {instantiation}): passes={counts['passes']} "
                f"passes with intervention={counts['passes with intervention']} "
                f"fails={counts['fails']} not yet run={counts['not yet run']} "
                f"not applicable by design={counts['not applicable by design']}"
            )
            story_counts = _core_story_counts(scenarios, latest, proof, instantiation)
            lines.append(
                f"Core stories ({proof}, {instantiation}): passes={_list_numbers(story_counts['passes'])} "
                f"passes with intervention={_list_numbers(story_counts['passes with intervention'])} "
                f"fails={_list_numbers(story_counts['fails'])} not yet run={_list_numbers(story_counts['not yet run'])} "
                f"not applicable by design={_list_numbers(story_counts['not applicable by design'])}"
            )
    return "\n".join(lines).rstrip() + "\n"
