#!/usr/bin/env python3
"""Select and supervise repository gate suites. Each suite owns a process group."""
import argparse
import json
import math
import os
from pathlib import Path
import shlex
import signal
import subprocess
import time

ROOT = Path(__file__).resolve().parents[2]


def suites(root, environment):
    entries = [("policy", ["python3", "tools/test_sync_agentic_workflow.py"], True)]
    entries.extend((path.stem, ["sh", str(path.relative_to(root))], True)
                   for path in sorted((root / "tools/agents/checks.d").glob("*.sh")))
    entries.extend(("bench-" + path.stem.removeprefix("test_"),
                    ["python3", str(path.relative_to(root))], environment.get("CHECK_BENCH") == "1")
                   for path in [*sorted((root / "bench/harness").glob("test_*.py")),
                                root / "tools/insula/test_capsule.py"]
                   if path.name != "test_lead_driven.py")
    entries.extend([
        ("sync", ["python3", "tools/sync_agentic_workflow.py", "--check"], environment.get("CHECK_SYNC") == "1"),
        ("cargo", ["cargo", "test", "--offline", "--", "--test-threads=1"], environment.get("CHECK_CARGO", "1") == "1"),
    ])
    return entries


def positive_seconds(value):
    number = float(value)
    if not math.isfinite(number) or number <= 0:
        raise argparse.ArgumentTypeError("must be a finite positive number")
    return number


def stop_group(process):
    """Bound cleanup even when the leader exits before a TERM-ignoring child.

    This is cooperative process-group containment, not a sandbox: descendants
    that call setsid/setpgid can escape. Such workloads require a cgroup owner.
    """
    for sig in (signal.SIGTERM, signal.SIGKILL):
        try:
            os.killpg(process.pid, sig)
        except ProcessLookupError:
            break
        if sig == signal.SIGTERM:
            deadline = time.monotonic() + 0.5
            while time.monotonic() < deadline:
                process.poll()
                try:
                    os.killpg(process.pid, 0)
                except ProcessLookupError:
                    break
                time.sleep(0.02)
    process.wait(timeout=2)


def run_suite(command, timeout, interrupted):
    process = subprocess.Popen(command, cwd=ROOT, start_new_session=True)
    deadline = time.monotonic() + timeout
    status = None
    try:
        while True:
            if interrupted:
                status = "interrupted"
                break
            code = process.poll()
            if code is not None:
                status = "passed" if code == 0 else "failed"
                break
            if time.monotonic() >= deadline:
                status = "timeout"
                break
            time.sleep(0.02)
    finally:
        stop_group(process)
    return status, process.returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--suite", action="append", help="run only this suite (repeatable; see --list)")
    parser.add_argument("--list", action="store_true", help="list suites without executing checks")
    parser.add_argument("--timeout", type=positive_seconds, default=1800,
                        help="seconds per suite (default: 1800; native controller cases exceed fifteen minutes)")
    parser.add_argument("--json", "--json-summary", dest="summary", type=Path,
                        help="write timing and status summary (also on interruption)")
    args = parser.parse_args()
    entries = suites(ROOT, os.environ)
    names = {name for name, _, _ in entries}
    if args.suite and set(args.suite) - names:
        parser.error("unknown suite(s): " + ", ".join(sorted(set(args.suite) - names)))
    if args.list:
        for name, command, default in entries:
            print(f"{name}\t{'default' if default else 'optional'}\t{shlex.join(command)}")
        return 0
    selected = [(name, command) for name, command, default in entries
                if name in args.suite] if args.suite else [(name, command) for name, command, default in entries if default]
    interrupted = []
    def on_signal(signum, frame):
        if not interrupted:
            interrupted.append(signum)
    for sig in (signal.SIGINT, signal.SIGTERM):
        signal.signal(sig, on_signal)
    started = time.monotonic()
    results = []
    for name, command in selected:
        print(f"== {name}: {shlex.join(command)}", flush=True)
        before = time.monotonic()
        if interrupted:
            break
        try:
            status, code = run_suite(command, args.timeout, interrupted)
        except OSError as error:
            print(f"cannot launch {name}: {error}", flush=True)
            status, code = "failed", None
        results.append(dict(suite=name, command=command, status=status, returncode=code,
                            duration_seconds=time.monotonic() - before))
        if status != "passed":
            print(f"FAILED: {name} ({status})", flush=True)
    passed = not interrupted and all(result['status'] == 'passed' for result in results)
    summary = dict(status="interrupted" if interrupted else ("passed" if passed else "failed"),
                   selected_suites=[name for name, _ in selected],
                   remaining_suites=[name for name, _ in selected[len(results):]], duration_seconds=time.monotonic() - started,
                   timeout_seconds=args.timeout, results=results)
    if args.summary:
        args.summary.parent.mkdir(parents=True, exist_ok=True)
        args.summary.write_text(json.dumps(summary, indent=2) + "\n")
    print("check: all green" if passed else "check: FAILED")
    return 128 + interrupted[0] if interrupted else (0 if passed else 1)


if __name__ == "__main__":
    raise SystemExit(main())
