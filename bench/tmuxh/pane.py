#!/usr/bin/env python3
"""The command a tmux pane runs: execute one agent unit in its capsule and record what happens.

  pane.py --job JOB.json

The job (written by the scheduler) holds the plan (capsule request, command, env, net, gpu, binds, timeout), the workspace and where
to write state. The pane shows a readable stream (stream.py), keeps the raw stream in events.jsonl, updates state.json (phase, elapsed,
tokens, last event) and writes result.json. Standard library only; the capsule stays the boundary for the agent.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import threading
import time
import traceback
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
for extra in (HERE, ROOT / "bench" / "harness", ROOT / "tools" / "insula"):
    if str(extra) not in sys.path:
        sys.path.insert(0, str(extra))

import state  # noqa: E402
import stream  # noqa: E402


ANSI = re.compile(r"\x1b\[[0-9;]*m")


def run_job(job: dict, out=sys.stdout, color: bool = True) -> int:
    import tasklib  # deferred: importing it needs the harness on sys.path

    unit_dir = Path(job["unit_dir"])
    unit_dir.mkdir(parents=True, exist_ok=True)
    spath = unit_dir / "state.json"
    plan = job["plan"]
    started = time.time()
    lock = threading.Lock()
    current = {**(state.read(spath) or {}), **job.get("meta", {}), "unit": job["unit"], "run_id": job["run_id"],
               "phase": "agent", "started": started, "pid": os.getpid(), "tokens_in": 0, "tokens_out": 0, "commands": 0,
               "files": 0, "errors": 0, "last": "starting capsule"}
    current.setdefault("created", started)
    state.write_atomic(spath, {**current, "updated": started})
    stop = threading.Event()

    def flush(extra: dict | None = None) -> None:
        with lock:
            if extra:
                current.update(extra)
            current["elapsed"] = round(time.time() - started, 1)
            state.write_atomic(spath, {**current, "updated": time.time()})

    def heartbeat() -> None:
        while not stop.wait(2.0):
            flush()

    threading.Thread(target=heartbeat, daemon=True).start()
    events = (unit_dir / "events.jsonl").open("a", buffering=1)
    log = (unit_dir / "pane.log").open("a", buffering=1)

    def say(text: str) -> None:
        print(text, file=out, flush=True)
        log.write(ANSI.sub("", text) + "\n")  # the saved log stays plain text; colors are only for the live pane

    def on_line(line: str) -> None:
        events.write(line + "\n")
        text, update = stream.format_event(line, color=color)
        if text is not None:
            say(text)
        if update:
            with lock:
                merged = stream.merge_update(current, update)
                current.clear()
                current.update(merged)

    header = f"{job['meta'].get('task', '?')}  {job['meta'].get('condition', '')}  {job['meta'].get('ticket', '') or ''}  " \
             f"agent={job['meta'].get('agent', '')}  capsule={Path(plan['capsule']).stem}  net={plan['net']}" \
             f"{'  gpu' if plan['gpu'] else ''}"
    say(("\033[1m" if color else "") + "══ " + header + (" ══\033[0m" if color else " ══"))
    code = 255
    try:
        record = tasklib.capsule.load_capsule(plan["capsule"])
        flush({"capsule_digest": record["capsule_digest"][:12], "last": "agent running"})
        result = tasklib.run_in_capsule(record, Path(job["workspace"]), plan["command"], plan["env"], plan["net"],
                                        float(plan["timeout"]), ro=[tuple(x) for x in plan["ro"]], gpu=plan["gpu"],
                                        on_line=on_line)
        code = int(result["rc"])
        (unit_dir / "result.json").write_text(json.dumps(result, indent=1) + "\n")
        flush({"phase": "done", "rc": code, "timed_out": bool(result.get("timed_out")),
               "last": f"finished rc={code} in {result['seconds']}s"})
        say(("\033[2m" if color else "") + f"══ finished rc={code} in {result['seconds']}s" +
            ("  TIMED OUT" if result.get("timed_out") else "") + (" ══\033[0m" if color else " ══"))
    except BaseException as exc:  # a crash must leave a state the scheduler and dashboard can act on
        tb = traceback.format_exc()
        say(f"!! pane crashed: {exc}")
        (unit_dir / "result.json").write_text(json.dumps({"rc": 255, "timed_out": False, "seconds": round(time.time() - started, 3),
                                                          "tail": tb[-1500:], "crashed": True}, indent=1) + "\n")
        flush({"phase": "crashed", "rc": 255, "last": f"crashed: {exc}"[:100]})
        code = 255
    finally:
        stop.set()
        events.close()
        log.close()
    return min(code, 255)


def main(argv: list[str] | None = None) -> int:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--job", required=True)
    p.add_argument("--no-color", action="store_true")
    args = p.parse_args(argv)
    job = json.loads(Path(args.job).read_text())
    return run_job(job, color=not args.no_color and sys.stdout.isatty())


if __name__ == "__main__":
    sys.exit(main())
