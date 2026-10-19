"""Small, explicit single-host durable runtime. All user side effects are activities.

This reference implements call-journal replay, not Python stack checkpointing.
The durable scheduler is intentionally separate from the Insula admission layer.
"""
from __future__ import annotations

from contextlib import contextmanager
from dataclasses import dataclass, field
import fcntl
import json
import math
import os
from pathlib import Path
import sqlite3
import subprocess
import sys
import time
from typing import Any, Callable, Iterator, Mapping
import uuid

from durable.common import MANAGED_TEMP_ENV, atomic_json, digest, durable_mkdir, encode, input_digest, read_json, temporary_environment
from durable.execution import InsulaExecution


from durable.errors import Park, ReplayMismatch, ActivityFailed, ControllerBusy, CleanupUnproven


_PENDING = object()


@dataclass(frozen=True)
class Activity:
    argv: tuple[str, ...]
    input: Any = None
    cwd: str | None = None
    env: Mapping[str, str] = field(default_factory=dict)
    timeout: float = 60.0
    max_attempts: int = 3
    backoff: float = 0.1
    recovery: str = "manual"  # Opt in to retry_safe only for safe/idempotent effects.
    cpus: int = 1
    gpus: int = 0
    inputs: str | None = None
    offline: bool = False

    def spec(self) -> dict[str, Any]:
        if not self.argv or not os.path.isabs(self.argv[0]):
            raise ValueError("argv[0] must be an absolute executable; no shell")
        if not all(isinstance(a, str) and "\x00" not in a for a in self.argv):
            raise ValueError("argv must contain NUL-free strings")
        if self.cwd is not None and not os.path.isabs(self.cwd):
            raise ValueError("cwd must be absolute")
        if self.recovery not in {"manual", "retry_safe"}:
            raise ValueError("recovery must be manual or retry_safe")
        if not math.isfinite(self.timeout) or self.timeout <= 0:
            raise ValueError("timeout must be finite and positive")
        if not math.isfinite(self.backoff) or self.backoff < 0:
            raise ValueError("backoff must be finite and nonnegative")
        for value, minimum in [(self.max_attempts, 1), (self.cpus, 1), (self.gpus, 0)]:
            if type(value) is not int or value < minimum:
                raise ValueError("invalid retry/resource count")
        env = {"PATH": os.defpath, "LANG": "C.UTF-8", "PYTHONNOUSERSITE": "1",
               "PYTHONDONTWRITEBYTECODE": "1"}
        for k, v in self.env.items():
            if k in MANAGED_TEMP_ENV:
                raise ValueError(f"{k} is a managed temporary environment variable")
            if not isinstance(k, str) or not isinstance(v, str) or "=" in k or "\x00" in k + v:
                raise ValueError("invalid environment entry")
            env[k] = v
        result = dict(argv=list(self.argv), input=self.input, cwd=self.cwd, env=env,
                      timeout=float(self.timeout), max_attempts=self.max_attempts,
                      backoff=float(self.backoff), recovery=self.recovery,
                      cpus=self.cpus, gpus=self.gpus)
        if self.inputs is not None:
            result["inputs"] = input_digest(self.inputs)
        if type(self.offline) is not bool:
            raise ValueError("offline must be a boolean")
        if self.offline:
            result["offline"] = True
        encode(result)
        return result


from durable.state.store import Store


class Context:
    def __init__(self, store: Store, run_id: str):
        self.store, self.run_id, self.seq = store, run_id, 0

    def _op(self, key: str, kind: str, spec: Any) -> sqlite3.Row:
        if not key:
            raise ValueError("operation key is required")
        seq, self.seq = self.seq, self.seq + 1
        h = digest(spec)
        with self.store.transaction() as db:
            row = db.execute("SELECT * FROM operations WHERE run_id=? AND seq=?", (self.run_id, seq)).fetchone()
            if row:
                if (row["key"], row["kind"], row["spec_digest"]) != (key, kind, h):
                    raise ReplayMismatch(f"operation {seq}: recorded {row['key']}/{row['kind']}, replayed {key}/{kind}; definition/input changed")
            else:
                if db.execute("SELECT 1 FROM operations WHERE run_id=? AND key=?", (self.run_id, key)).fetchone():
                    raise ReplayMismatch(f"duplicate operation key {key}")
                oid = uuid.uuid4().hex
                due = time.time() + spec["seconds"] if kind == "TIMER" else None
                db.execute("INSERT INTO operations(id,run_id,seq,key,kind,spec,spec_digest,due) VALUES(?,?,?,?,?,?,?,?)",
                           (oid, self.run_id, seq, key, kind, encode(spec), h, due))
                self.store.event("OPERATION_RECORDED", self.run_id, oid, key=key, operation_kind=kind)
                row = db.execute("SELECT * FROM operations WHERE id=?", (oid,)).fetchone()
        return row

    def _activity(self, key: str, activity: Activity) -> Any:
        row = self._op(key, "ACTIVITY", activity.spec())
        if row["status"] == "SUCCEEDED":
            return json.loads(row["result"])
        if row["status"] == "FAILED":
            raise ActivityFailed(f"{key}: {row['error']}")
        return _PENDING

    def activity(self, key: str, activity: Activity) -> Any:
        result = self._activity(key, activity)
        if result is _PENDING:
            raise Park()
        return result

    def parallel(self, items: list[tuple[str, Activity]]) -> list[Any]:
        # Record operations in a stable order, not completion order.
        results = [self._activity(key, activity) for key, activity in items]
        if any(value is _PENDING for value in results):
            raise Park()
        return results

    def sleep(self, key: str, seconds: float) -> None:
        if not math.isfinite(seconds) or seconds < 0:
            raise ValueError("sleep must be finite and nonnegative")
        row = self._op(key, "TIMER", {"seconds": float(seconds)})
        if row["status"] == "SUCCEEDED":
            return
        if time.time() < row["due"]:
            raise Park()
        with self.store.transaction() as db:
            db.execute("UPDATE operations SET status='SUCCEEDED',result='null' WHERE id=?", (row["id"],))
            self.store.event("TIMER_FIRED", self.run_id, row["id"])

    def wait_signal(self, key: str) -> Any:
        row = self._op(key, "SIGNAL", {"name": key})
        if row["status"] == "SUCCEEDED":
            return json.loads(row["result"])
        value = self.store.db.execute("SELECT payload FROM signals WHERE run_id=? AND key=?", (self.run_id, key)).fetchone()
        if value is None:
            raise Park()
        with self.store.transaction() as db:
            db.execute("UPDATE operations SET status='SUCCEEDED',result=? WHERE id=?", (value[0], row["id"]))
            self.store.event("SIGNAL_CONSUMED", self.run_id, row["id"])
        return json.loads(value[0])


Workflow = Callable[[Context, Any], Any]


@dataclass
class _Live:
    process: subprocess.Popen[bytes]
    control: int
    attempt: str
    op: str
    run: str
    cpus: int
    devices: list[str]
    closing: bool = False


class Runtime:
    """One controller per state directory; guardians retain its lock during cleanup."""
    def __init__(self, store: Store, registry: dict[str, tuple[str, Workflow]],
                 contract: str, max_active: int = 4, cpu_slots: int = 4,
                 gpu_devices: tuple[str, ...] = (), lock_timeout: float = 3.0,
                 failpoint: Callable[[str, dict[str, Any]], None] | None = None,
                 execution: InsulaExecution | None = None):
        if sys.platform != "linux":
            raise RuntimeError("This process-group backend requires Linux /proc")
        if max_active < 1 or cpu_slots < 1 or len(set(gpu_devices)) != len(gpu_devices):
            raise ValueError("invalid capacity")
        self.store, self.registry, self.contract = store, registry, contract
        self.execution = execution
        if execution is not None:
            execution.verify()
            if execution.contract != contract or execution.config["state"] != str(store.root):
                raise ValueError("runtime contract/state disagree with admitted execution")
        self.max_active, self.cpu_slots = max_active, cpu_slots
        self.gpu_devices, self.lock_timeout = list(gpu_devices), lock_timeout
        self.failpoint = failpoint or (lambda _point, _context: None)
        self.live: dict[str, _Live] = {}
        self.lock_fd: int | None = None
        self.epoch = 0

    def __enter__(self) -> Runtime:
        self.lock_fd = os.open(self.store.root / "controller.lock", os.O_RDWR | os.O_CREAT, 0o600)
        deadline = time.monotonic() + self.lock_timeout
        while True:
            try:
                fcntl.flock(self.lock_fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
                break
            except BlockingIOError:
                if time.monotonic() >= deadline:
                    os.close(self.lock_fd)
                    self.lock_fd = None
                    raise ControllerBusy("controller or activity cleanup still holds the state lock")
                time.sleep(0.02)
        try:
            # Contract/version validation happens before recovery or launching work.
            for run in self.store.db.execute("SELECT * FROM runs WHERE status IN ('RUNNING','BLOCKED')"):
                if run["contract"] != self.contract:
                    raise RuntimeError("capsule contract mismatch; resume under the admitted contract")
                if run["workflow"] not in self.registry or self.registry[run["workflow"]][0] != run["version"]:
                    raise RuntimeError("workflow version unavailable; no automatic migration")
            with self.store.transaction() as db:
                row = db.execute("SELECT value FROM meta WHERE key='epoch'").fetchone()
                self.epoch = int(row[0]) + 1 if row else 1
                db.execute("INSERT OR REPLACE INTO meta VALUES('epoch',?)", (str(self.epoch),))
                self.store.event("CONTROLLER_STARTED", epoch=self.epoch)
            self._assert_clean()
            for attempt in self.store.db.execute("SELECT * FROM attempts WHERE status='ACTIVE'").fetchall():
                receipt = Path(attempt["directory"]) / "receipt.json"
                if receipt.exists():
                    try:
                        self._accept(attempt["id"], read_json(receipt))
                    except (ValueError, OSError) as exc:
                        self._block(attempt["id"], f"invalid recovery receipt: {exc}")
                elif not attempt["released"]:
                    self._accept(attempt["id"], self._synthetic(attempt, started=False, cleanup=True,
                                                               reason="interrupted before release"))
                else:
                    self._recover_missing_receipt(attempt)
            self._assert_clean()
            return self
        except BaseException:
            os.close(self.lock_fd)
            self.lock_fd = None
            raise

    def _assert_clean(self) -> None:
        rows = self.store.db.execute("SELECT attempt_id FROM cleanup_blocks WHERE acknowledged IS NULL").fetchall()
        if rows:
            raise CleanupUnproven("cleanup unproven for " + ", ".join(r[0] for r in rows))

    def _recover_missing_receipt(self, attempt: sqlite3.Row) -> None:
        """Kernel ownership can prove cleanup; it cannot prove external effects."""
        if self.execution is None or self.execution.backend != "cgroup":
            self._block(attempt["id"], "released attempt has no receipt; external cleanup evidence required")
            return
        from insula.cgroup import Cgroup
        try:
            identity = read_json(Path(attempt["directory"]) / "cgroup.json")
            membership = read_json(Path(attempt["directory"]) / "membership.json")
            if membership != {"attempt_id": attempt["id"], "cgroup": identity}:
                raise ValueError("cgroup payload membership was not durably confirmed")
            group = Cgroup.resume(identity, attempt_id=attempt["id"],
                                  root_identity=self.execution.config["cgroup_identity"])
            proof = group.kill_and_wait()
            receipt = self._synthetic(attempt, started=bool(attempt["released"]), cleanup=True,
                                      reason="guardian lost; cgroup cleanup verified", cleanup_proof=proof)
            atomic_json(Path(attempt["directory"]) / "receipt.json", receipt)
            self._accept(attempt["id"], receipt)
        except (ValueError, OSError, RuntimeError) as exc:
            self._block(attempt["id"], f"cgroup recovery cleanup unproven: {exc}")

    @staticmethod
    def _synthetic(attempt: sqlite3.Row, **fields: Any) -> dict[str, Any]:
        return dict(protocol=1, attempt_id=attempt["id"], operation_id=attempt["operation_id"],
                    spec_digest=attempt["spec_digest"], status="FAILED", **fields)

    def _block(self, aid: str, reason: str) -> None:
        with self.store.transaction() as db:
            att = db.execute("SELECT * FROM attempts WHERE id=?", (aid,)).fetchone()
            op = db.execute("SELECT * FROM operations WHERE id=?", (att["operation_id"],)).fetchone()
            db.execute("INSERT OR IGNORE INTO cleanup_blocks VALUES(?,?,NULL)", (aid, reason))
            db.execute("UPDATE attempts SET status='LOST' WHERE id=?", (aid,))
            db.execute("UPDATE operations SET status='UNCERTAIN',error=? WHERE id=?", (reason, op["id"]))
            db.execute("UPDATE runs SET status='BLOCKED',error=? WHERE id=?", (reason, op["run_id"]))
            self.store.event("CLEANUP_UNPROVEN", op["run_id"], op["id"], aid, reason=reason)

    def _accept(self, aid: str, receipt: Any) -> bool:
        """Validate first, then fence result acceptance in the same DB transaction."""
        att = self.store.db.execute("SELECT * FROM attempts WHERE id=?", (aid,)).fetchone()
        if att is None:
            return False
        current = self.store.db.execute("SELECT status,active_attempt FROM operations WHERE id=?", (att["operation_id"],)).fetchone()
        if att["status"] != "ACTIVE" or current["status"] != "RUNNING" or current["active_attempt"] != aid:
            return False
        if not isinstance(receipt, dict) or type(receipt.get("protocol")) is not int:
            raise ValueError("invalid receipt object/protocol")
        identity = (receipt.get("protocol"), receipt.get("attempt_id"), receipt.get("operation_id"), receipt.get("spec_digest"))
        expected = (1, aid, att["operation_id"], att["spec_digest"])
        if identity != expected:
            raise ValueError("receipt identity mismatch")
        if receipt.get("status") not in ("SUCCEEDED", "FAILED") or type(receipt.get("started")) is not bool or type(receipt.get("cleanup")) is not bool:
            raise ValueError("invalid receipt envelope")
        if receipt["status"] == "SUCCEEDED":
            if not receipt["started"] or "result" not in receipt or receipt.get("reason") is not None:
                raise ValueError("invalid successful receipt")
            if digest(receipt["result"]) != receipt.get("result_digest"):
                raise ValueError("receipt result digest mismatch")
        elif not isinstance(receipt.get("reason"), str) or not receipt["reason"].strip():
            raise ValueError("failed receipt requires a reason")
        if not receipt["cleanup"]:
            self._block(aid, "guardian could not prove activity cleanup")
            return False
        with self.store.transaction() as db:
            op = db.execute("SELECT * FROM operations WHERE id=?", (att["operation_id"],)).fetchone()
            # Fencing check: stale/duplicate receipts are not authoritative.
            if op["status"] != "RUNNING" or op["active_attempt"] != aid or att["status"] != "ACTIVE":
                return False
            run = db.execute("SELECT * FROM runs WHERE id=?", (op["run_id"],)).fetchone()
            db.execute("UPDATE attempts SET status=?,receipt_digest=? WHERE id=?", (receipt["status"], digest(receipt), aid))
            if receipt["status"] == "SUCCEEDED":
                db.execute("UPDATE operations SET status='SUCCEEDED',result=?,error=NULL,active_attempt=NULL WHERE id=? AND active_attempt=?",
                           (encode(receipt["result"]), op["id"], aid))
                self.store.event("ACTIVITY_COMMITTED", op["run_id"], op["id"], aid)
            else:
                spec = json.loads(op["spec"])
                # Cancellation and sibling failure prove cleanup, not what a
                # started external effect did. Preserve the reconciliation gate.
                if receipt["started"] and spec["recovery"] == "manual":
                    db.execute("UPDATE operations SET status='UNCERTAIN',active_attempt=NULL,error=? WHERE id=?", (receipt["reason"], op["id"]))
                    db.execute("UPDATE runs SET status='BLOCKED',error=? WHERE id=?", (receipt["reason"], op["run_id"]))
                    self.store.event("EFFECT_UNCERTAIN", op["run_id"], op["id"], aid)
                elif run["cancel_requested"] or run["status"] in {"FAILED", "CANCELLED"}:
                    db.execute("UPDATE operations SET status='CANCELLED',active_attempt=NULL,error=? WHERE id=?", (receipt["reason"], op["id"]))
                elif op["attempt_count"] < spec["max_attempts"]:
                    delay = min(60.0, spec["backoff"] * 2 ** min(op["attempt_count"] - 1, 20))
                    db.execute("UPDATE operations SET status='PENDING',active_attempt=NULL,ready_at=?,error=? WHERE id=?",
                               (time.time() + delay, receipt["reason"], op["id"]))
                    self.store.event("RETRY_SCHEDULED", op["run_id"], op["id"], aid, delay=delay)
                else:
                    db.execute("UPDATE operations SET status='FAILED',active_attempt=NULL,error=? WHERE id=?", (receipt["reason"], op["id"]))
                    self.store.event("RETRIES_EXHAUSTED", op["run_id"], op["id"], aid)
        return True

    def _replay(self) -> None:
        for run in self.store.db.execute("SELECT * FROM runs WHERE status='RUNNING' AND cancel_requested=0 ORDER BY created").fetchall():
            # Newly submitted work must pass the same version/contract checks.
            definition = self.registry.get(run["workflow"])
            if run["contract"] != self.contract or definition is None or definition[0] != run["version"]:
                raise RuntimeError("new workflow contract/version mismatch")
            ctx = Context(self.store, run["id"])
            try:
                result = definition[1](ctx, json.loads(run["args"]))
                count = self.store.db.execute("SELECT count(*) FROM operations WHERE run_id=?", (run["id"],)).fetchone()[0]
                if count != ctx.seq:
                    raise ReplayMismatch("workflow returned before consuming its recorded operation tail")
                output = encode(result)
            except Park:
                continue
            except Exception as exc:
                with self.store.transaction() as db:
                    db.execute("UPDATE runs SET status='FAILED',error=? WHERE id=?", (f"{type(exc).__name__}: {exc}", run["id"]))
                    self.store.event("RUN_FAILED", run["id"], reason=str(exc))
            else:
                with self.store.transaction() as db:
                    db.execute("UPDATE runs SET status='SUCCEEDED',result=? WHERE id=?", (output, run["id"]))
                    self.store.event("RUN_SUCCEEDED", run["id"])

    def _launch(self, op: sqlite3.Row, devices: list[str]) -> None:
        spec = json.loads(op["spec"])
        if "inputs" in spec:
            if self.execution is None:
                raise ValueError("activity inputs require an admitted Insula execution")
            self.execution.input_bundle(spec["inputs"])
        aid, number = uuid.uuid4().hex, op["attempt_count"] + 1
        directory = self.store.root / "attempts" / aid
        durable_mkdir(directory)
        with self.store.transaction() as db:
            db.execute("INSERT INTO attempts(id,operation_id,number,epoch,spec_digest,directory,created) VALUES(?,?,?,?,?,?,?)",
                       (aid, op["id"], number, self.epoch, op["spec_digest"], str(directory), time.time()))
            cur = db.execute("UPDATE operations SET status='RUNNING',active_attempt=?,attempt_count=? WHERE id=? AND status='PENDING'",
                             (aid, number, op["id"]))
            if cur.rowcount != 1:
                raise RuntimeError("activity claim lost")
            self.store.event("ATTEMPT_CLAIMED", op["run_id"], op["id"], aid, number=number, devices=devices)
        ctx = dict(attempt_id=aid, operation_id=op["id"], run_id=op["run_id"])
        self.failpoint("after_claim", ctx)
        effective = {**spec, "env": {**spec["env"], **temporary_environment(),
                                   "CUDA_VISIBLE_DEVICES": ",".join(devices)}}
        req = dict(protocol=1, workflow_id=op["run_id"], operation_id=op["id"],
                   operation_key=op["key"], attempt_id=aid, attempt_number=number,
                   spec_digest=op["spec_digest"], attempt_dir=str(directory), input=spec["input"],
                   idempotency_key=digest([op["run_id"], op["key"], op["spec_digest"]]), spec=effective)
        if self.execution is not None:
            self.execution.verify()
            req["execution"] = self.execution.config
            if self.execution.backend == "cgroup":
                from insula.cgroup import Cgroup
                group = Cgroup.create(self.execution.config["cgroup_root"], aid,
                                      root_identity=self.execution.config["cgroup_identity"])
                req["cgroup"] = group.identity()
                atomic_json(directory / "cgroup.json", req["cgroup"])
        atomic_json(directory / "request.json", req)
        read_fd, write_fd = os.pipe()
        try:
            guard = subprocess.Popen(
                [sys.executable, "-m", "durable.runtime.guard", str(directory / "request.json"),
                 "--control-fd", str(read_fd), "--lock-fd", str(self.lock_fd)],
                pass_fds=(read_fd, self.lock_fd), stdin=subprocess.DEVNULL,
                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, close_fds=True,
                cwd=Path(__file__).resolve().parents[2],
            )
        except BaseException:
            os.close(write_fd)
            raise
        finally:
            os.close(read_fd)
        self.live[aid] = _Live(guard, write_fd, aid, op["id"], op["run_id"], spec["cpus"], devices)
        self.failpoint("after_guard_spawn", ctx)
        with self.store.transaction() as db:
            db.execute("UPDATE attempts SET released=1 WHERE id=?", (aid,))
            self.store.event("RELEASE_INTENT", op["run_id"], op["id"], aid)
        self.failpoint("before_release", ctx)
        os.write(write_fd, b"G")
        self.failpoint("after_release", ctx)

    def _dispatch(self) -> None:
        if len(self.live) >= self.max_active:
            return
        rows = self.store.db.execute("SELECT o.* FROM operations o JOIN runs r ON r.id=o.run_id WHERE o.kind='ACTIVITY' AND o.status='PENDING' AND o.ready_at<=? AND r.status='RUNNING' AND r.cancel_requested=0 ORDER BY r.created,o.seq", (time.time(),)).fetchall()
        for op in rows:
            if len(self.live) >= self.max_active:
                break
            spec = json.loads(op["spec"])
            if spec["cpus"] > self.cpu_slots or spec["gpus"] > len(self.gpu_devices):
                with self.store.transaction() as db:
                    db.execute("UPDATE operations SET status='FAILED',error='request exceeds admitted capacity' WHERE id=?", (op["id"],))
                continue
            free = [d for d in self.gpu_devices if all(d not in job.devices for job in self.live.values())]
            cpus = sum(job.cpus for job in self.live.values())
            if cpus + spec["cpus"] <= self.cpu_slots and len(free) >= spec["gpus"]:
                self._launch(op, free[:spec["gpus"]])

    @staticmethod
    def _stop(job: _Live) -> None:
        if not job.closing:
            os.close(job.control)  # EOF wakes guardian, which owns process cleanup.
            job.closing = True

    def _collect(self) -> None:
        for aid, job in list(self.live.items()):
            if job.process.poll() is None:
                continue
            self._stop(job)
            del self.live[aid]
            att = self.store.db.execute("SELECT * FROM attempts WHERE id=?", (aid,)).fetchone()
            path = Path(att["directory"]) / "receipt.json"
            if not path.exists():
                self._recover_missing_receipt(att)
                continue
            try:
                receipt = read_json(path)
            except (ValueError, OSError) as exc:
                self._block(aid, f"invalid guardian receipt: {exc}")
                continue
            self.failpoint("after_receipt", dict(attempt_id=aid, operation_id=job.op, run_id=job.run))
            try:
                self._accept(aid, receipt)
            except (ValueError, OSError) as exc:
                self._block(aid, f"invalid guardian receipt: {exc}")
                continue
            self.failpoint("after_commit", dict(attempt_id=aid, operation_id=job.op, run_id=job.run))

    def _cancellations(self) -> None:
        for job in self.live.values():
            row = self.store.db.execute("SELECT status,cancel_requested FROM runs WHERE id=?", (job.run,)).fetchone()
            if row[0] != "RUNNING" or row[1]:
                self._stop(job)
        for run in self.store.db.execute("SELECT * FROM runs WHERE cancel_requested=1 OR status='FAILED'").fetchall():
            with self.store.transaction() as db:
                db.execute("UPDATE operations SET status='CANCELLED' WHERE run_id=? AND status='PENDING'", (run["id"],))
                if run["cancel_requested"] and not any(j.run == run["id"] for j in self.live.values()):
                    # A blocked/uncertain effect remains visible even after cancellation.
                    uncertain = db.execute("SELECT 1 FROM operations WHERE run_id=? AND status='UNCERTAIN'", (run["id"],)).fetchone()
                    if not uncertain and run["status"] not in {"SUCCEEDED", "CANCELLED"}:
                        db.execute("UPDATE runs SET status='CANCELLED' WHERE id=?", (run["id"],))
                        self.store.event("RUN_CANCELLED", run["id"])

    def tick(self) -> None:
        self._assert_clean()
        self._collect()
        self._assert_clean()
        self._cancellations()
        self._replay()
        self._cancellations()
        self._dispatch()

    def drive(self, timeout: float = 30.0, serve: bool = False) -> dict[str, str]:
        deadline = time.monotonic() + timeout
        while True:
            self.tick()
            running = self.store.db.execute("SELECT count(*) FROM runs WHERE status='RUNNING'").fetchone()[0]
            runnable = self.store.db.execute("SELECT count(*) FROM operations o JOIN runs r ON r.id=o.run_id WHERE r.status='RUNNING' AND o.status!='SUCCEEDED' AND o.kind!='SIGNAL'").fetchone()[0]
            if not serve and not self.live and (not running or not runnable):
                break  # Settled, or parked only on externally delivered signals.
            if time.monotonic() >= deadline:
                raise TimeoutError("drive deadline reached; active guardians will be stopped")
            time.sleep(0.01)
        return {r[0]: r[1] for r in self.store.db.execute("SELECT id,status FROM runs")}

    def __exit__(self, *_exc: Any) -> None:
        for job in self.live.values():
            self._stop(job)
        # Never force-kill guardians here: they retain the lock until cleanup.
        deadline = time.monotonic() + 5
        try:
            while self.live and time.monotonic() < deadline:
                self._collect()
                time.sleep(0.01)
        finally:
            if self.lock_fd is not None:
                os.close(self.lock_fd)
                self.lock_fd = None
        # Remaining guardians hold inherited lock descriptors: fail closed.
        if self.live:
            raise CleanupUnproven("guardian cleanup still running; state remains locked")
