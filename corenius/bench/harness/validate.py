#!/usr/bin/env python3
"""Offline validation of the benchmark itself: tasks and verifiers are sound before any agent runs.

For every task: the starting state fails the verifier; the reference solution passes; cheats
(deleted tests, stubbed tests) do not turn a failing state into a pass; the passing verdict is
deterministic; the capsule probe is clean. Every step runs inside the task's capsule.
"""

from __future__ import annotations

import argparse
import json
import shutil
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fleet  # noqa: E402
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


def cheat_paths(task: tasklib.Task) -> list[str]:
    """Where an agent would tamper: the declared protected paths plus the conventional tests/ directory."""
    return sorted(set(task.protected) | {"tests"})


def stub_out(ws: Path, task: tasklib.Task) -> None:
    for rel in cheat_paths(task):
        for path in (ws / rel).rglob("*") if (ws / rel).is_dir() else [ws / rel]:
            if path.is_file():
                path.write_text("#!/bin/sh\nexit 0\n" if path.suffix in ("", ".sh") else "")


def check_task(task: tasklib.Task, record: dict) -> dict:
    checks: dict[str, dict] = {}
    with tempfile.TemporaryDirectory(prefix=f"bench-{task.id}-", dir=str(tasklib.ROOT / ".scratch")) as tmp:
        base = Path(tmp)

        def fresh(name: str) -> Path:
            ws = base / name
            tasklib.materialize(task, ws)
            return ws

        start = tasklib.verify(task, fresh("start"), record)
        checks["start-fails"] = {"ok": not start["passed"] and not start["timed_out"], "rc": start["rc"], "seconds": start["seconds"]}

        ws = fresh("solved")
        applied = tasklib.apply_solution(task, ws, record)
        solved = tasklib.verify(task, ws, record)
        checks["solution-applies"] = {"ok": applied["rc"] == 0, "rc": applied["rc"]}
        checks["solution-passes"] = {"ok": solved["passed"], "rc": solved["rc"], "seconds": solved["seconds"],
                                     "tail": "" if solved["passed"] else solved["tail"]}
        again = tasklib.verify(task, ws, record)
        checks["deterministic"] = {"ok": again["passed"] == solved["passed"] and again["rc"] == solved["rc"]}

        deleted = fresh("cheat-delete")
        for rel in cheat_paths(task):
            target = deleted / rel
            shutil.rmtree(target) if target.is_dir() else target.unlink(missing_ok=True)
        r = tasklib.verify(task, deleted, record)
        checks["cheat-delete-tests"] = {"ok": not r["passed"]}

        stubbed = fresh("cheat-stub")
        stub_out(stubbed, task)
        r = tasklib.verify(task, stubbed, record)
        checks["cheat-stub-tests"] = {"ok": not r["passed"]}

        extra = [r for r in task.protected if r != "tests"]
        if extra:
            tampered = fresh("tampered")
            tasklib.apply_solution(task, tampered, record)
            for rel in extra:
                target = tampered / rel
                for path in (target.rglob("*") if target.is_dir() else [target]):
                    if path.is_file():
                        path.write_text("// tampered\n")
            r = tasklib.verify(task, tampered, record)
            checks["tamper-protected-restored"] = {"ok": r["passed"], "paths": extra, "tail": "" if r["passed"] else r["tail"]}

        empty = fresh("empty")
        checks["empty-patch-fails"] = {"ok": not tasklib.verify(task, empty, record)["passed"]}
    if "multi" in task.tags:
        checks.update(check_multi(task, record))
    return checks


def check_multi(task: tasklib.Task, record: dict) -> dict:
    """Every ticket is necessary, and every landing order ends in a pass with the declared conflict count."""
    expected = task.cfg.get("multi", {}).get("expected_conflicts")
    ids = [t.id for t in fleet.load_tickets(task)]
    checks = {}
    necessary = {}
    for skip in ids:
        run = fleet.run_fleet(task, fleet.reference_agent, record, include=[i for i in ids if i != skip])
        necessary[skip] = not run["passed"]
    checks["tickets-necessary"] = {"ok": all(necessary.values()) and len(ids) > 1, "detail": necessary}
    bad = []
    for order in fleet.all_orders(task):
        run = fleet.run_fleet(task, fleet.reference_agent, record, order=order)
        if not run["passed"] or run["contamination"] or (expected is not None and run["conflicts"] != expected):
            bad.append({"order": order, "passed": run["passed"], "conflicts": run["conflicts"],
                        "contamination": run["contamination"]})
    checks["orders-land"] = {"ok": not bad, "orders": len(fleet.all_orders(task)), "bad": bad}
    return checks


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--task", action="append", default=[])
    p.add_argument("--tasks-root", default=str(tasklib.TASKS), help="directory containing task subdirectories")
    p.add_argument("--json", help="write the full report here")
    args = p.parse_args(argv)
    (tasklib.ROOT / ".scratch").mkdir(exist_ok=True)
    tasks = tasklib.load_tasks(root=Path(args.tasks_root), only=args.task)
    report, failed = {}, 0
    records: dict[str, dict] = {}
    started = time.time()
    for task in tasks:
        request_path = str(tasklib.capsule_request_path(task).resolve())
        name = task.bench.get("capsule", "toolchains")
        if request_path not in records:
            records[request_path] = tasklib.capsule_for(task)
            probe = tasklib.capsule.probe(records[request_path])
            report.setdefault("_capsules", {})[request_path] = {
                "name": name,
                "digest": records[request_path]["capsule_digest"],
                "probe": probe,
            }
            if not probe["ok"]:
                print(f"capsule {name}: probe failed: {probe}")
                failed += 1
        checks = check_task(task, records[request_path])
        report[task.id] = checks
        bad = [k for k, v in checks.items() if not v["ok"]]
        failed += bool(bad)
        print(f"{'FAIL' if bad else 'ok  '} {task.id:28s} " + (", ".join(bad) if bad else f"{len(checks)} checks"))
        for k in bad:
            tail = checks[k].get("tail")
            if tail:
                print("    " + tail.replace("\n", "\n    "))
    print(f"{len(tasks)} task(s), {failed} failing, {time.time() - started:.1f}s")
    if args.json:
        Path(args.json).write_text(json.dumps(report, indent=2) + "\n")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
