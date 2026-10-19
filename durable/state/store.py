"""Single SQLite workflow journal and operational transition owner."""
from __future__ import annotations
from contextlib import contextmanager
import json
from pathlib import Path
import sqlite3
import time
from typing import Any, Iterator
from durable.common import digest, durable_mkdir, encode
from durable.errors import CleanupUnproven

_SCHEMA = """
CREATE TABLE IF NOT EXISTS meta (key TEXT PRIMARY KEY, value TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS runs (
 id TEXT PRIMARY KEY, workflow TEXT NOT NULL, version TEXT NOT NULL,
 contract TEXT NOT NULL, args TEXT NOT NULL, fingerprint TEXT NOT NULL,
 status TEXT NOT NULL DEFAULT 'RUNNING', cancel_requested INTEGER NOT NULL DEFAULT 0,
 result TEXT, error TEXT, created REAL NOT NULL
);
CREATE TABLE IF NOT EXISTS operations (
 id TEXT PRIMARY KEY, run_id TEXT NOT NULL REFERENCES runs(id),
 seq INTEGER NOT NULL, key TEXT NOT NULL, kind TEXT NOT NULL,
 spec TEXT NOT NULL, spec_digest TEXT NOT NULL,
 status TEXT NOT NULL DEFAULT 'PENDING', active_attempt TEXT,
 attempt_count INTEGER NOT NULL DEFAULT 0, ready_at REAL NOT NULL DEFAULT 0,
 due REAL, result TEXT, error TEXT,
 UNIQUE(run_id, seq), UNIQUE(run_id, key)
);
CREATE TABLE IF NOT EXISTS attempts (
 id TEXT PRIMARY KEY, operation_id TEXT NOT NULL REFERENCES operations(id),
 number INTEGER NOT NULL, epoch INTEGER NOT NULL, spec_digest TEXT NOT NULL,
 directory TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'ACTIVE',
 released INTEGER NOT NULL DEFAULT 0, created REAL NOT NULL, receipt_digest TEXT,
 UNIQUE(operation_id, number)
);
CREATE TABLE IF NOT EXISTS signals (
 run_id TEXT NOT NULL REFERENCES runs(id), key TEXT NOT NULL,
 payload TEXT NOT NULL, PRIMARY KEY(run_id, key)
);
CREATE TABLE IF NOT EXISTS cleanup_blocks (
 attempt_id TEXT PRIMARY KEY REFERENCES attempts(id), reason TEXT NOT NULL,
 acknowledged TEXT
);
CREATE TABLE IF NOT EXISTS events (
 id INTEGER PRIMARY KEY AUTOINCREMENT, at REAL NOT NULL, kind TEXT NOT NULL,
 run_id TEXT, operation_id TEXT, attempt_id TEXT, detail TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS ready_operations ON operations(status, ready_at);
PRAGMA user_version=1;
"""


class Store:
    """Operational state only. Git/source/rootfs remain the code/spec authority."""
    def __init__(self, state: str | Path, *, allow_unpatched_sqlite: bool = False):
        v = sqlite3.sqlite_version_info
        self.sqlite_fix_verified = (v >= (3, 51, 3) or
                                    (v[:2] == (3, 50) and v >= (3, 50, 7)) or
                                    (v[:2] == (3, 44) and v >= (3, 44, 6)))
        if not self.sqlite_fix_verified and not allow_unpatched_sqlite:
            raise RuntimeError(
                f"SQLite {sqlite3.sqlite_version}: WAL-reset fix not verified. "
                "Use SQLite >=3.51.3 or documented 3.50.7/3.44.6 backport. "
                "The allow_unpatched_sqlite override is for disposable tests only."
            )
        self.root = Path(state).resolve()
        durable_mkdir(self.root)
        self.db = sqlite3.connect(self.root / "runtime.sqlite", isolation_level=None, timeout=5)
        self.db.row_factory = sqlite3.Row
        self.db.execute("PRAGMA foreign_keys=ON")
        self.db.execute("PRAGMA busy_timeout=5000")
        mode = self.db.execute("PRAGMA journal_mode=WAL").fetchone()[0]
        if mode != "wal":
            raise RuntimeError("SQLite WAL unavailable")
        self.db.execute("PRAGMA synchronous=FULL")
        version = self.db.execute("PRAGMA user_version").fetchone()[0]
        if version not in {0, 1}:
            raise RuntimeError(f"Unsupported database schema: {version}")
        self.db.executescript(_SCHEMA)

    def close(self) -> None:
        self.db.close()

    @contextmanager
    def transaction(self) -> Iterator[sqlite3.Connection]:
        self.db.execute("BEGIN IMMEDIATE")
        try:
            yield self.db
            self.db.execute("COMMIT")
        except BaseException:
            self.db.execute("ROLLBACK")
            raise

    def event(self, kind: str, run: str | None = None, op: str | None = None,
              attempt: str | None = None, **detail: Any) -> None:
        self.db.execute("INSERT INTO events(at,kind,run_id,operation_id,attempt_id,detail) VALUES(?,?,?,?,?,?)",
                        (time.time(), kind, run, op, attempt, encode(detail)))

    def submit(self, run_id: str, workflow: str, version: str, args: Any,
               contract: str) -> str:
        if not run_id or not workflow or not version or not contract:
            raise ValueError("run ID, workflow, version and contract must be nonempty")
        payload = encode(args)
        fingerprint = digest([workflow, version, contract, args])
        with self.transaction() as db:
            old = db.execute("SELECT fingerprint FROM runs WHERE id=?", (run_id,)).fetchone()
            if old:
                if old[0] != fingerprint:
                    raise ValueError("run ID already binds different immutable inputs")
                return run_id
            db.execute("INSERT INTO runs(id,workflow,version,contract,args,fingerprint,created) VALUES(?,?,?,?,?,?,?)",
                       (run_id, workflow, version, contract, payload, fingerprint, time.time()))
            self.event("RUN_SUBMITTED", run_id)
        return run_id

    def signal(self, run_id: str, key: str, payload: Any) -> None:
        value = encode(payload)
        with self.transaction() as db:
            old = db.execute("SELECT payload FROM signals WHERE run_id=? AND key=?", (run_id, key)).fetchone()
            if old:
                if old[0] != value:
                    raise ValueError("signal key already has a different value")
                return
            db.execute("INSERT INTO signals VALUES(?,?,?)", (run_id, key, value))
            self.event("SIGNAL_RECORDED", run_id, key=key)

    def cancel(self, run_id: str) -> None:
        with self.transaction() as db:
            row = db.execute("SELECT status FROM runs WHERE id=?", (run_id,)).fetchone()
            if row is None:
                raise KeyError(run_id)
            if row[0] in {"RUNNING", "BLOCKED"}:
                db.execute("UPDATE runs SET cancel_requested=1 WHERE id=?", (run_id,))
                self.event("CANCEL_REQUESTED", run_id)

    def acknowledge_cleanup(self, attempt_id: str, evidence: str) -> None:
        """Trusted operator/Insula evidence only. Does not retry or launch anything."""
        if not evidence.strip():
            raise ValueError("cleanup evidence is required")
        with self.transaction() as db:
            cur = db.execute("UPDATE cleanup_blocks SET acknowledged=? WHERE attempt_id=?",
                             (evidence, attempt_id))
            if cur.rowcount != 1:
                raise KeyError(attempt_id)
            self.event("CLEANUP_ACKNOWLEDGED", attempt=attempt_id, evidence=evidence)

    def resolve(self, run_id: str, key: str, decision: str, evidence: str,
                result: Any = None) -> None:
        """Resolve uncertain effects explicitly; reconciliation itself never runs work."""
        if decision not in {"retry", "fail", "succeed"} or not evidence.strip():
            raise ValueError("decision and evidence required")
        with self.transaction() as db:
            if db.execute("SELECT 1 FROM cleanup_blocks WHERE acknowledged IS NULL").fetchone():
                raise CleanupUnproven("acknowledge cleanup before resolving effects")
            op = db.execute("SELECT * FROM operations WHERE run_id=? AND key=?", (run_id, key)).fetchone()
            if op is None or op["status"] != "UNCERTAIN":
                raise ValueError("operation is not uncertain")
            status = {"retry": "PENDING", "fail": "FAILED", "succeed": "SUCCEEDED"}[decision]
            db.execute("UPDATE operations SET status=?,active_attempt=NULL,result=?,error=NULL,ready_at=0 WHERE id=?",
                       (status, encode(result) if decision == "succeed" else None, op["id"]))
            uncertain = db.execute("SELECT 1 FROM operations WHERE run_id=? AND status='UNCERTAIN'", (run_id,)).fetchone()
            if not uncertain:
                db.execute("UPDATE runs SET status='RUNNING',error=NULL WHERE id=?", (run_id,))
            self.event("EFFECT_RESOLVED", run_id, op["id"], decision=decision, evidence=evidence)

    def status(self, run_id: str) -> dict[str, Any]:
        run = self.db.execute("SELECT * FROM runs WHERE id=?", (run_id,)).fetchone()
        if run is None:
            raise KeyError(run_id)
        data = dict(run)
        for key in ["args", "result"]:
            data[key] = json.loads(data[key]) if data[key] is not None else None
        data["operations"] = [dict(r) for r in self.db.execute(
            "SELECT id,seq,key,kind,status,active_attempt,attempt_count,ready_at,due,result,error FROM operations WHERE run_id=? ORDER BY seq",
            (run_id,))]
        data["attempts"] = [dict(r) for r in self.db.execute(
            "SELECT a.* FROM attempts a JOIN operations o ON a.operation_id=o.id WHERE o.run_id=? ORDER BY a.created", (run_id,))]
        return data
