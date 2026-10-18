"""Gated activity guardian; no database access and no shell execution.

The controller sends G only after persisting release intent. EOF on the control
pipe means cancel/owner death. The inherited flock descriptor is retained until
activity cleanup finishes. A stopped guardian therefore prevents takeover.
The default backend assumes trusted children that retain their process group.
Admitted Insula activities may use delegated cgroup v2 ownership instead.
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

from controller.durable.common import atomic_json, digest, durable_mkdir, read_json
from controller.durable.insula import InsulaExecution
from controller.durable.cgroup import Cgroup


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
    group: Cgroup | None = None
    cleanup_proof: dict[str, Any] | None = None
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

    def clean_process() -> bool:
        nonlocal cleanup_proof
        if group is not None:
            try:
                cleanup_proof = group.kill_and_wait()
                if proc is not None:
                    proc.wait(timeout=1)
                return True
            except (OSError, RuntimeError, subprocess.TimeoutExpired) as exc:
                receipt["reason"] = f"cgroup cleanup unproven: {exc}"
                return False
        return _kill_group(proc) if proc is not None else True

    def enter_cgroup() -> None:
        # This fork child retains the ownership fd until membership is durable.
        # A killed guardian must not let replacement recovery race a child
        # paused before its join. A live controller additionally requires this
        # marker before recovering a guardian without a receipt.
        assert group is not None
        group.join_current_process()
        atomic_json(base / "membership.json", {"attempt_id": req["attempt_id"], "cgroup": group.identity()})
        os.close(args.lock_fd)

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
        argv = spec["argv"]
        request_file = base / "request.json"
        result_file = base / "result.json"
        cwd = spec["cwd"] or str(base)
        environment = spec["env"]
        if "execution" in req:
            execution = InsulaExecution(req["execution"])
            # Only payload data is projected writable; receipt and ownership
            # metadata remain private to the host guardian.
            payload_dir = base / "payload"
            durable_mkdir(payload_dir)
            launch, payload_request = execution.payload(payload_dir, req)
            request_file = base / "payload-request.json"
            atomic_json(request_file, payload_request)
            result_file = payload_dir / "result.json"
            argv = launch + ["--"] + spec["argv"]
            cwd = execution.config["code"]
            environment = {"PATH": os.defpath}
            if execution.backend == "cgroup":
                group = Cgroup.resume(req["cgroup"], attempt_id=req["attempt_id"],
                                      root_identity=execution.config["cgroup_identity"])
        with open(request_file, "rb") as inp, \
             open(base / "stdout.log", "ab", buffering=0) as stdout, \
             open(base / "stderr.log", "ab", buffering=0) as stderr:
            proc = subprocess.Popen(
                argv, stdin=inp, stdout=stdout, stderr=stderr,
                cwd=cwd, env=environment,
                start_new_session=True, close_fds=True,
                pass_fds=(args.lock_fd,) if group else (),
                preexec_fn=enter_cgroup if group else None,
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
            cleanup = clean_process()
            proc = None
            # Only read results after the selected backend proves cleanup.
            if exit_ok and cleanup:
                result = read_json(result_file)
                receipt.update(status="SUCCEEDED", reason=None, result=result,
                               result_digest=digest(result))
    except Exception as exc:
        receipt["reason"] = f"{type(exc).__name__}: {exc}"
    finally:
        if proc is not None:
            cleanup = clean_process()
        receipt.update(started=started, cleanup=cleanup, finished_at=time.time())
        if cleanup_proof is not None:
            receipt["cleanup_proof"] = cleanup_proof
        if not cleanup:
            receipt.update(status="FAILED", reason=receipt.get("reason") or "activity cleanup unproven")
        atomic_json(base / "receipt.json", receipt)
        os.close(args.control_fd)
        # No LOCK_UN: inherited descriptors share the flock open description.
        os.close(args.lock_fd)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
