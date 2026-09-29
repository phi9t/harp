#!/usr/bin/env python3
"""Run benchmark tasks under a condition with an agent, inside capsules; write result.json per run.

  run.py TASK... | --all   --condition solo|fleet:N   --agent reference|noop|cmd:SHELL|traecli[:MODEL]
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
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fleet  # noqa: E402
import results  # noqa: E402
import tasklib  # noqa: E402

PROMPT_HEADER = """You are an agent working on one task inside a sandbox. Your working directory is /workspace.
Change only what the task needs. Do not modify anything under tests/ (it is restored before grading).
You have no network access to fetch packages or answers. Do not print secrets.
Finish with a short message saying what you changed and what you did not check.

TASK
"""


class Agent:
    def __init__(self, spec: str, authorized: bool = False, timeout: float | None = None):
        self.spec, self.authorized, self.timeout = spec, authorized, timeout
        self.kind = spec.split(":", 1)[0]
        if self.kind not in ("reference", "noop", "cmd", "traecli"):
            raise SystemExit(f"unknown agent {spec!r}")
        if self.kind == "traecli" and not authorized:
            raise SystemExit("agent traecli spends model budget; pass --authorize-model-spend to run it")
        self.arg = spec.split(":", 1)[1] if ":" in spec else ""

    def capsule_record(self, task: tasklib.Task) -> dict:
        if self.kind == "traecli":
            return tasklib.capsule.build(str(tasklib.REQUESTS / "agent.json"))
        return tasklib.capsule_for(task)

    def run(self, instruction: str, ws: Path, record: dict, solution_or_ticket: Path | None, task: tasklib.Task) -> dict:
        timeout = self.timeout or float(task.cfg.get("agent", {}).get("timeout_sec", 900))
        if self.kind == "noop":
            return {"rc": 0, "seconds": 0.0}
        if self.kind == "reference":
            if solution_or_ticket is None:
                return tasklib.apply_solution(task, ws, record)
            command = ("sh /tmp/ticket/apply.sh" if (solution_or_ticket / "apply.sh").exists()
                       else "cp -r /tmp/ticket/overlay/. /workspace/")
            return tasklib.run_in_capsule(record, ws, command, {}, "none", 120.0, ro=[(str(solution_or_ticket), "/tmp/ticket")])
        with tempfile.TemporaryDirectory(prefix="agent-", dir=str(tasklib.ROOT / ".scratch")) as tmp:
            (Path(tmp) / "instruction.md").write_text(instruction)
            ro = [(str(Path(tmp) / "instruction.md"), "/tmp/task/instruction.md")]
            if self.kind == "cmd":
                return tasklib.run_in_capsule(record, ws, self.arg, {"TASK_INSTRUCTION": "/tmp/task/instruction.md"},
                                              "none", timeout, ro=ro)
            model = self.arg or "GPT-5.5"
            prompt = Path(tmp) / "prompt.md"
            prompt.write_text(PROMPT_HEADER + instruction)
            ro = [(str(prompt), "/tmp/task/prompt.md")]
            command = (f"traecli exec -m {model} -c model_reasoning_effort=xhigh -c model_reasoning_summary=detailed "
                       f"--json --skip-git-repo-check -s danger-full-access -C /workspace --shell-tool-timeout 120s - < /tmp/task/prompt.md")
            return tasklib.run_in_capsule(record, ws, command, {}, "host", timeout, ro=ro,
                                          gpu=bool(task.bench.get("gpu", False)))


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


def run_solo(task: tasklib.Task, agent: Agent, out: Path, index: int) -> dict:
    verify_record = tasklib.capsule_for(task)
    agent_record = agent.capsule_record(task)
    rid = run_id(task.id, "solo", agent.spec, index)
    rundir = out / rid
    ws = rundir / "workspace"
    tasklib.materialize(task, ws)
    started = time.time()
    instruction = (task.path / "instruction.md").read_text()
    result = agent.run(instruction, ws, agent_record, None, task)
    agent_seconds = time.time() - started
    (rundir / "agent-tail.txt").write_text(str(result.get("tail", "")))
    verdict = tasklib.verify(task, ws, verify_record)
    rec = base_record(task, "solo", agent, verify_record, rid)
    rec["outcome"] = {"passed": verdict["passed"], "reward": 1.0 if verdict["passed"] else 0.0,
                      "verifier_rc": verdict["rc"], "timed_out": verdict["timed_out"], "agent_rc": result.get("rc")}
    rec["timing"] = {"wall_seconds": round(time.time() - started, 3), "agent_seconds": round(agent_seconds, 3),
                     "verify_seconds": verdict["seconds"]}
    rec["coordination"] = {"conflicts": 0, "redos": 0, "contamination": 0, "tickets": 1, "tickets_landed": 1}
    (rundir / "verify-tail.txt").write_text(verdict["tail"])
    shutil.rmtree(ws, ignore_errors=True) if not agent.spec.startswith(("cmd", "traecli")) else None
    return rec


def run_fleet_condition(task: tasklib.Task, agent: Agent, out: Path, index: int, n: int) -> dict:
    if "multi" not in task.tags:
        raise SystemExit(f"{task.id} is a single-ticket task; fleet:{n} needs a multi task")
    verify_record = tasklib.capsule_for(task)
    agent_record = agent.capsule_record(task)
    rid = run_id(task.id, f"fleet:{n}", agent.spec, index)
    (out / rid).mkdir(parents=True, exist_ok=True)

    def ticket_agent(ticket, ws, _record):
        return agent.run(ticket.instruction, ws, agent_record, ticket.path, task)

    run = fleet.run_fleet(task, ticket_agent, verify_record)
    rec = base_record(task, f"fleet:{n}", agent, verify_record, rid)
    rec["outcome"] = {"passed": run["passed"], "reward": 1.0 if run["passed"] else 0.0, "verifier_rc": 0 if run["passed"] else 1,
                      "timed_out": False, "agent_rc": None}
    rec["timing"] = {"wall_seconds": run["wall_seconds"], "agent_seconds": round(sum(run["ticket_seconds"].values()), 3),
                     "verify_seconds": run["verify_seconds"]}
    rec["coordination"] = {"conflicts": run["conflicts"], "redos": run["redos"], "contamination": len(run["contamination"]),
                           "tickets": len(run["order"]), "tickets_landed": sum(1 for l in run["landings"] if l["landed"]),
                           "landing_order": run["order"]}
    return rec


def main(argv=None) -> int:
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    p.add_argument("task", nargs="*")
    p.add_argument("--all", action="store_true")
    p.add_argument("--condition", default="solo")
    p.add_argument("--agent", default="reference")
    p.add_argument("--repeat", type=int, default=1)
    p.add_argument("--out", default=None)
    p.add_argument("--timeout", type=float)
    p.add_argument("--authorize-model-spend", action="store_true")
    p.add_argument("--report", metavar="DIR")
    args = p.parse_args(argv)
    if args.report:
        rows = results.summarize(results.load_all(Path(args.report)))
        print(results.table(rows))
        return 0
    (tasklib.ROOT / ".scratch").mkdir(exist_ok=True)
    agent = Agent(args.agent, args.authorize_model_spend, args.timeout)
    tasks = tasklib.load_tasks(only=None if args.all else args.task)
    if not (args.condition == "solo" or re.fullmatch(r"fleet:\d+", args.condition)):
        raise SystemExit("condition must be solo or fleet:N")
    out = Path(args.out or tasklib.ROOT / "bench" / "results" / datetime.now(timezone.utc).strftime("%Y%m%dT%H%M%SZ"))
    out.mkdir(parents=True, exist_ok=True)
    for task in tasks:
        if args.condition != "solo" and "multi" not in task.tags:
            continue
        for i in range(args.repeat):
            rec = (run_solo(task, agent, out, i) if args.condition == "solo"
                   else run_fleet_condition(task, agent, out, i, int(args.condition.split(":")[1])))
            results.validate(rec)
            (out / rec["run_id"]).mkdir(parents=True, exist_ok=True)
            (out / rec["run_id"] / "result.json").write_text(json.dumps(rec, indent=2) + "\n")
            print(f"{'PASS' if rec['outcome']['passed'] else 'FAIL'} {rec['run_id']} {rec['timing']['wall_seconds']}s")
    print(results.table(results.summarize(results.load_all(out))))
    return 0


if __name__ == "__main__":
    sys.exit(main())
