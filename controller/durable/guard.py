"""Gated activity guardian; no database access and no shell execution.

The controller sends G only after persisting release intent. EOF on the control
pipe means cancel/owner death. The inherited flock descriptor is retained until
process-group cleanup finishes. A stopped guardian therefore prevents takeover.

Scope: trusted children which do not escape their process group. cgroup-backed
containment must replace this backend for daemonizing/arbitrary workloads.
"""
from __future__ import annotations

import argparse
import ctypes
import os
from pathlib import Path
import select
import signal
import subprocess
import sys
import time
from typing import Any

from controller.durable.common import atomic_json, digest, read_json


def _group_members(pgid: int) -> list[int]:
    """Linux-only cleanup evidence, ignoring zombies (which cannot mutate)."""
    members = []
    for path in Path("/proc").glob("[0-9]*/stat"):
        try:
            fields = path.read_text().rsplit(")", 1)[1].split()
            if fields[0] != "Z" and int(fields[2]) == pgid:
                members.append(int(path.parent.name))
        except (FileNotFoundError, ProcessLookupError):
            continue
    return members


def _kill_group(proc: subprocess.Popen[bytes]) -> bool:
    # proc.pid is the new session/process-group identity. Do not persist/reuse a
    # bare PID after restart. This guardian owns it for its complete lifetime.
    for sig, wait_s in [(signal.SIGTERM, 0.15), (signal.SIGKILL, 2.0)]:
        try:
            os.killpg(proc.pid, sig)
        except ProcessLookupError:
            pass
        deadline = time.monotonic() + wait_s
        while time.monotonic() < deadline:
            proc.poll()
            if not _group_members(proc.pid):
                proc.wait(timeout=1)
                # Adopted grandchildren, if any, are reaped opportunistically.
                while True:
                    try:
                        pid, _ = os.waitpid(-1, os.WNOHANG)
                        if pid == 0:
                            break
                    except ChildProcessError:
                        break
                return True
            time.sleep(0.01)
    return False


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("request")
    parser.add_argument("--control-fd", type=int, required=True)
    parser.add_argument("--lock-fd", type=int, required=True)
    args = parser.parse_args()
    req = read_json(Path(args.request))
    base = Path(req["attempt_dir"])
    proc: subprocess.Popen[bytes] | None = None
    started = False
    cleanup = True
    exit_ok = False
    receipt: dict[str, Any] = {
        "protocol": 1,
        "attempt_id": req["attempt_id"],
        "operation_id": req["operation_id"],
        "spec_digest": req["spec_digest"],
        "status": "FAILED",
        "reason": "not released",
    }
    # Signals to this guardian are handled like EOF, so ordinary TERM is safe.
    interrupted = False
    def stop(_sig: int, _frame: Any) -> None:
        nonlocal interrupted
        interrupted = True
    signal.signal(signal.SIGTERM, stop)
    signal.signal(signal.SIGINT, stop)
    try:
        # Linux subreaper helps reap ordinary orphan descendants during cleanup.
        libc = ctypes.CDLL(None, use_errno=True)
        if libc.prctl(36, 1, 0, 0, 0) != 0:  # PR_SET_CHILD_SUBREAPER
            raise OSError(ctypes.get_errno(), "PR_SET_CHILD_SUBREAPER")
        while not interrupted:
            ready, _, _ = select.select([args.control_fd], [], [], 0.05)
            if ready:
                if os.read(args.control_fd, 1) != b"G":
                    return 0
                break
        if interrupted:
            return 0
        spec = req["spec"]
        with open(base / "request.json", "rb") as inp, \
             open(base / "stdout.log", "ab", buffering=0) as stdout, \
             open(base / "stderr.log", "ab", buffering=0) as stderr:
            proc = subprocess.Popen(
                spec["argv"], stdin=inp, stdout=stdout, stderr=stderr,
                cwd=spec["cwd"] or str(base), env=spec["env"],
                start_new_session=True, close_fds=True,
            )
            started = True
            atomic_json(base / "process.json", {"pid": proc.pid, "pgid": proc.pid})
            deadline = time.monotonic() + spec["timeout"]
            while proc.poll() is None:
                ready, _, _ = select.select([args.control_fd], [], [], 0.02)
                if interrupted or (ready and os.read(args.control_fd, 1) != b"G"):
                    receipt["reason"] = "owner lost or cancellation requested"
                    break
                if time.monotonic() >= deadline:
                    receipt["reason"] = "attempt timeout"
                    break
            else:
                if proc.returncode == 0:
                    exit_ok = True
                else:
                    receipt["reason"] = f"exit code {proc.returncode}"
            cleanup = _kill_group(proc)
            proc = None
            # No ordinary child in the owned group may still modify the result.
            if exit_ok and cleanup:
                result = read_json(base / "result.json")
                receipt.update(status="SUCCEEDED", reason=None, result=result,
                               result_digest=digest(result))
    except Exception as exc:
        receipt["reason"] = f"{type(exc).__name__}: {exc}"
    finally:
        if proc is not None:
            cleanup = _kill_group(proc)
        receipt.update(started=started, cleanup=cleanup, finished_at=time.time())
        if not cleanup:
            receipt.update(status="FAILED", reason="process-group cleanup unproven")
        atomic_json(base / "receipt.json", receipt)
        os.close(args.control_fd)
        # No LOCK_UN: inherited descriptors share the flock open description.
        os.close(args.lock_fd)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
