#!/usr/bin/env python3
"""Run benchmark tasks under a condition with an agent, inside capsules; write result.json per run.

  run.py TASK... | --all   --condition solo|fleet:N|controller:TICKET   --agent reference|noop|cmd:SHELL|traecli[:MODEL]
         [--repeat K] [--out DIR] [--authorize-model-spend]
  run.py --report DIR       aggregate results under DIR

Agents run inside a capsule (task capsule, offline, for reference/noop/cmd; the agent capsule with network for
traecli). Verification always runs in the task capsule with the network off and the pristine tests restored.
A real model is only launched with --authorize-model-spend.
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
import tempfile
import time
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import controller_mode  # noqa: E402
import fleet  # noqa: E402
import results  # noqa: E402
import tasklib  # noqa: E402

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)

PROMPT_HEADER = """You are an agent working on one task inside a sandbox. Your working directory is /workspace.
Change only what the task needs. Do not modify anything under tests/ (it is restored before grading).
You have no network access to fetch packages or answers. Do not print secrets.
Finish with a short message saying what you changed and what you did not check.

TASK
"""


@dataclass
class Plan:
    """Everything needed to execute one agent unit in a capsule (also serialized as a tmux pane job)."""
    command: str
    env: dict
    net: str
    gpu: bool
    ro: list  # [[host, dest], ...]
    timeout: float
    capsule: str  # capsule request file

    def to_json(self) -> dict:
        return {"command": self.command, "env": self.env, "net": self.net, "gpu": self.gpu, "ro": self.ro,
                "timeout": self.timeout, "capsule": self.capsule}


class Agent:
    def __init__(self, spec: str, authorized: bool = False, timeout: float | None = None):
        self.spec, self.authorized, self.timeout = spec, authorized, timeout
        self.kind = spec.split(":", 1)[0]
        if self.kind not in ("reference", "noop", "cmd", "traecli"):
            raise SystemExit(f"unknown agent {spec!r}")
        if self.kind == "traecli" and not authorized:
            raise SystemExit("agent traecli spends model budget; pass --authorize-model-spend to run it")
        self.arg = spec.split(":", 1)[1] if ":" in spec else ""

    def capsule_request(self, task: tasklib.Task) -> Path:
        if self.kind == "traecli":
            return tasklib.REQUESTS / "agent.json"
        return tasklib.capsule_request_path(task)

    def capsule_record(self, task: tasklib.Task) -> dict:
        return tasklib.capsule.build(str(self.capsule_request(task)))

    def plan(self, instruction: str, task: tasklib.Task, solution_or_ticket: Path | None, workdir: Path) -> Plan:
        """The capsule invocation for one unit. Files the agent needs (instruction, prompt) are written into `workdir`."""
        timeout = self.timeout or float(task.cfg.get("agent", {}).get("timeout_sec", 900))
        request = str(self.capsule_request(task))
        agent_env = dict(task.cfg.get("agent", {}).get("env", {}))
        agent_net = task.cfg.get("agent", {}).get("net", "none")
        if self.kind == "noop":
            return Plan("true", agent_env, "none", False, [], 30.0, request)
        if self.kind == "reference":
            if solution_or_ticket is None:
                return Plan(tasklib.SOLUTION_COMMAND, agent_env, agent_net, False, [[str(task.path / "solution"), "/run/corenius-tmp/solution"]],
                            120.0, request)
            command = ("sh /run/corenius-tmp/ticket/apply.sh" if (solution_or_ticket / "apply.sh").exists()
                       else "cp -r /run/corenius-tmp/ticket/overlay/. /workspace/")
            return Plan(command, agent_env, agent_net, False, [[str(solution_or_ticket), "/run/corenius-tmp/ticket"]], 120.0, request)
        workdir.mkdir(parents=True, exist_ok=True)
        (workdir / "instruction.md").write_text(instruction)
        if self.kind == "cmd":
            return Plan(self.arg, {**agent_env, "TASK_INSTRUCTION": "/run/corenius-tmp/task/instruction.md"}, agent_net, False,
                        [[str(workdir / "instruction.md"), "/run/corenius-tmp/task/instruction.md"]], timeout, request)
        model = self.arg or "GPT-5.5"
        (workdir / "prompt.md").write_text(PROMPT_HEADER + instruction)
        command = (f"traecli exec -m {model} -c model_reasoning_effort=xhigh -c model_reasoning_summary=detailed "
                   f"--json --skip-git-repo-check -s danger-full-access -C /workspace --shell-tool-timeout 120s - < /run/corenius-tmp/task/prompt.md")
        return Plan(command, agent_env, "host", bool(task.bench.get("gpu", False)), [[str(workdir / "prompt.md"), "/run/corenius-tmp/task/prompt.md"]],
                    timeout, request)

    @staticmethod
    def execute(plan: Plan, record: dict, ws: Path, on_line=None, services: list[dict] | None = None) -> dict:
        return tasklib.run_in_capsule(record, ws, plan.command, plan.env, plan.net, plan.timeout,
                                      ro=[tuple(x) for x in plan.ro], gpu=plan.gpu, on_line=on_line,
                                      services=services)

    def run(self, instruction: str, ws: Path, record: dict, solution_or_ticket: Path | None, task: tasklib.Task) -> dict:
        if self.kind == "noop":
            return {"rc": 0, "seconds": 0.0}
        with tempfile.TemporaryDirectory(prefix="agent-", dir=str(tasklib.ROOT / ".scratch")) as tmp:
            services = [service for service in task.services if "agent" in service["phases"]]
            return self.execute(self.plan(instruction, task, solution_or_ticket, Path(tmp)), record, ws,
                                services=services)


def parse_usage(text: str) -> dict | None:
    """Token counts from the traecli event stream (turn.completed events); None when the stream has none."""
    total, seen = {"input_tokens": 0, "output_tokens": 0}, False
    for line in text.splitlines():
        if '"turn.completed"' not in line:
            continue
        try:
            usage = json.loads(line).get("usage") or {}
        except ValueError:
            continue
        seen = True
        total["input_tokens"] += int(usage.get("input_tokens", 0))
        total["output_tokens"] += int(usage.get("output_tokens", 0))
    return total if seen else None


def run_id(task_id: str, condition: str, agent: str, index: int) -> str:
    stamp = datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    return f"{stamp}-{condition.replace(':', '')}-{task_id}-{re.sub(r'[^a-z0-9]+', '', agent.lower())}-{index:03d}"


def base_record(task, condition, agent, record, rid):
    return {
        "schema_version": results.SCHEMA, "run_id": rid, "task_id": task.id, "condition": condition,
        "agent": {"spec": agent.spec},
        "capsule": {"digest": record["capsule_digest"], "net": record["policy"]["net"],
                    "hermetic": record["hermetic"], "non_hermetic_layers": record["non_hermetic_layers"]},
        "cost": {"usd": None, "input_tokens": None, "output_tokens": None},
        "provenance": {"outcome": "measured", "timing": "measured", "coordination": "measured", "cost": "unmeasured"},
    }


def run_solo(task: tasklib.Task, agent: Agent, out: Path, index: int, rid: str | None = None, observer=None) -> dict:
    observer = observer or (lambda event, **fields: None)
    verify_record = tasklib.capsule_for(task)
    agent_record = agent.capsule_record(task)
    rid = rid or run_id(task.id, "solo", agent.spec, index)
    rundir = out / rid
    ws = rundir / "workspace"
    tasklib.materialize(task, ws)
    started = time.time()
    instruction = (task.path / "instruction.md").read_text()
    observer("agents")
    result = agent.run(instruction, ws, agent_record, None, task)
    agent_seconds = time.time() - started
    (rundir / "agent-tail.txt").write_text(str(result.get("tail", "")))
    observer("verify")
    verdict = tasklib.verify(task, ws, verify_record)
    observer("done", passed=verdict["passed"])
    rec = base_record(task, "solo", agent, verify_record, rid)
    rec["outcome"] = {"passed": verdict["passed"], "reward": 1.0 if verdict["passed"] else 0.0,
                      "verifier_rc": verdict["rc"], "timed_out": verdict["timed_out"], "agent_rc": result.get("rc")}
    rec["timing"] = {"wall_seconds": round(time.time() - started, 3), "agent_seconds": round(agent_seconds, 3),
                     "verify_seconds": verdict["seconds"]}
    rec["coordination"] = {"conflicts": 0, "redos": 0, "contamination": 0, "tickets": 1, "tickets_landed": 1}
    usage = parse_usage(str(result.get("tail", "")))
    if usage:
        rec["cost"].update(usage)
        rec["provenance"]["cost"] = "tokens measured from the agent event stream; usd unmeasured"
    (rundir / "verify-tail.txt").write_text(verdict["tail"])
    shutil.rmtree(ws, ignore_errors=True) if not agent.spec.startswith(("cmd", "traecli")) else None
    return rec


def run_fleet_condition(task: tasklib.Task, agent: Agent, out: Path, index: int, n: int, rid: str | None = None,
                        observer=None, log=None) -> dict:
    observer = observer or (lambda event, **fields: None)
    if "multi" not in task.tags:
        raise SystemExit(f"{task.id} is a single-ticket task; fleet:{n} needs a multi task")
    verify_record = tasklib.capsule_for(task)
    agent_record = agent.capsule_record(task)
    rid = rid or run_id(task.id, f"fleet:{n}", agent.spec, index)
    (out / rid).mkdir(parents=True, exist_ok=True)

    usages: list[dict] = []

    def ticket_agent(ticket, ws, _record):
        out = agent.run(ticket.instruction, ws, agent_record, ticket.path, task)
        usage = parse_usage(str(out.get("tail", "")))
        if usage:
            usages.append(usage)
        return out

    observer("agents")
    run = fleet.run_fleet(task, ticket_agent, verify_record, workers=n, log=log, observer=observer)
    observer("done", passed=run["passed"], conflicts=run["conflicts"], redos=run["redos"])
    (out / rid / "verify-tail.txt").write_text(str(run.get("verify_tail", "")))
    rec = base_record(task, f"fleet:{n}", agent, verify_record, rid)
    rec["outcome"] = {"passed": run["passed"], "reward": 1.0 if run["passed"] else 0.0, "verifier_rc": 0 if run["passed"] else 1,
                      "timed_out": False, "agent_rc": None}
    rec["timing"] = {"wall_seconds": run["wall_seconds"], "agent_seconds": round(sum(run["ticket_seconds"].values()), 3),
                     "verify_seconds": run["verify_seconds"]}
    rec["coordination"] = {"conflicts": run["conflicts"], "redos": run["redos"], "contamination": len(run["contamination"]), "contamination_paths": run["contamination"][:50],
                           "tickets": len(run["order"]), "tickets_landed": sum(1 for l in run["landings"] if l["landed"]),
                           "landing_order": run["order"]}
    if usages:
        rec["cost"].update({k: sum(u[k] for u in usages) for k in ("input_tokens", "output_tokens")})
        rec["provenance"]["cost"] = "tokens measured from the agent event stream; usd unmeasured"
    return rec


def persist(rec: dict, out: Path) -> Path:
    """Validate a run record and write it as out/<run_id>/result.json (shared by the direct and the tmux runners)."""
    results.validate(rec)
    path = out / rec["run_id"] / "result.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(rec, indent=2) + "\n")
    return path


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("task", nargs="*")
    p.add_argument("--all", action="store_true")
    p.add_argument("--tasks-root", default=str(tasklib.TASKS), help="directory containing task subdirectories")
    p.add_argument("--condition", default="solo")
    p.add_argument("--agent", default="reference")
    p.add_argument("--repeat", type=int, default=1)
    p.add_argument("--out", default=None)
    p.add_argument("--timeout", type=float)
    p.add_argument("--authorize-model-spend", action="store_true")
    p.add_argument("--controller-lifecycle-fault", choices=sorted(controller_mode.LIFECYCLE_FAULTS),
                   help="deterministic controller lifecycle fault injection for bounded failure tests")
    p.add_argument("--report", metavar="DIR")
    args = p.parse_args(argv)
    if args.report:
        rows = results.summarize(results.load_all(Path(args.report)))
        print(results.table(rows))
        return 0
    (tasklib.ROOT / ".scratch").mkdir(exist_ok=True)
    agent = Agent(args.agent, args.authorize_model_spend, args.timeout)
    tasks = tasklib.load_tasks(root=Path(args.tasks_root), only=None if args.all else args.task)
    if not (args.condition == "solo" or re.fullmatch(r"fleet:\d+", args.condition)
            or re.fullmatch(r"controller:[A-Za-z0-9_.-]+", args.condition)):
        raise SystemExit("condition must be solo, fleet:N, or controller:TICKET")
    if args.condition.startswith("controller:"):
        controller_mode.validate_controller_selection(tasks, args.condition.split(":", 1)[1])
    elif args.controller_lifecycle_fault:
        raise SystemExit("--controller-lifecycle-fault requires a controller condition")
    out = Path(args.out or tasklib.ROOT / "bench" / "results" / datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"))
    out.mkdir(parents=True, exist_ok=True)
    for task in tasks:
        if args.condition.startswith("fleet:") and "multi" not in task.tags:
            continue
        for i in range(args.repeat):
            if args.condition == "solo":
                rec = run_solo(task, agent, out, i)
            elif args.condition.startswith("fleet:"):
                rec = run_fleet_condition(task, agent, out, i, int(args.condition.split(":")[1]))
            else:
                rec = controller_mode.run_controller_condition(
                    task,
                    agent,
                    out,
                    i,
                    args.condition.split(":", 1)[1],
                    fault=args.controller_lifecycle_fault,
                )
            persist(rec, out)
            print(f"{'PASS' if rec['outcome']['passed'] else 'FAIL'} {rec['run_id']} {rec['timing']['wall_seconds']}s")
            if (
                args.condition.startswith("controller:")
                and rec["outcome"].get("failure_stage") in controller_mode.DISPATCH_FAILURE_STAGES
            ):
                break
    print(results.table(results.summarize(results.load_all(out))))
    return 0


if __name__ == "__main__":
    sys.exit(main())
