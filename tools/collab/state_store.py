"""SQLite-backed collab state store.

This module is the only collab module that imports or calls SQLite directly.
"""
from __future__ import annotations

from contextlib import contextmanager
import fcntl
import json
import os
from pathlib import Path
import sqlite3

from common import Refused, failpoint, now, output_path, valid_id

StoreError = sqlite3.Error

SCHEMA = """
CREATE TABLE config (id INTEGER PRIMARY KEY CHECK(id=1), value TEXT NOT NULL);
CREATE TABLE attempts (
 id TEXT PRIMARY KEY, ticket TEXT NOT NULL, owner TEXT NOT NULL,
 state TEXT NOT NULL, base TEXT NOT NULL, ticket_hash TEXT NOT NULL,
 path TEXT NOT NULL UNIQUE, branch TEXT NOT NULL UNIQUE,
 worker_pid INTEGER, worker_pid_start_ticks TEXT, worker_run_dir TEXT,
 candidate TEXT, integration TEXT, check_path TEXT, log TEXT,
 created TEXT NOT NULL, updated TEXT NOT NULL
);
CREATE UNIQUE INDEX one_active_ticket ON attempts(ticket)
 WHERE state NOT IN ('landed','abandoned');
CREATE TABLE events (
 seq INTEGER PRIMARY KEY AUTOINCREMENT, at TEXT NOT NULL,
 attempt TEXT, kind TEXT NOT NULL, details TEXT NOT NULL
);
"""

UNRESOLVED_LANDING = (
    "landing_prepared",
    "push_intended",
    "push_completed",
    "landing_started",
    "landing",
    "landed_records_incomplete",
)
ATTEMPT_LIVENESS_COLUMNS = {
    "worker_pid": "INTEGER",
    "worker_pid_start_ticks": "TEXT",
    "worker_run_dir": "TEXT",
}


class StateStore:
    def __init__(self, home: Path, db_path: Path):
        self.home = home
        self.db_path = db_path
        self.fds: list[int] = []

    def load_config(self) -> dict | None:
        if not self.home.exists():
            return None
        if not self.db_path.is_file():
            raise Refused("Partial initialization; inspect, do not overwrite")
        with self.db() as con:
            self._ensure_schema(con)
            row = con.execute("SELECT value FROM config WHERE id=1").fetchone()
        if row:
            return json.loads(row[0])
        return None

    def _ensure_schema(self, con) -> None:
        if not con.execute("SELECT name FROM sqlite_master WHERE type='table' AND name='attempts'").fetchone():
            return
        columns = {row["name"] for row in con.execute("PRAGMA table_info(attempts)")}
        for name, kind in ATTEMPT_LIVENESS_COLUMNS.items():
            if name not in columns:
                con.execute(f"ALTER TABLE attempts ADD COLUMN {name} {kind}")

    def require_initialized(self) -> None:
        if not self.db_path.is_file():
            raise Refused("Run init first; never delete state.sqlite3 to reset active work")

    @contextmanager
    def db(self):
        con = sqlite3.connect(self.db_path, timeout=5, isolation_level=None)
        con.row_factory = sqlite3.Row
        con.execute("PRAGMA foreign_keys=ON")
        con.execute("PRAGMA busy_timeout=5000")
        con.execute("PRAGMA synchronous=FULL")
        try:
            yield con
        finally:
            con.close()

    @contextmanager
    def tx(self):
        with self.db() as con:
            con.execute("BEGIN IMMEDIATE")
            try:
                yield con
                con.commit()
            except BaseException:
                con.rollback()
                raise

    @contextmanager
    def lock(self, name: str):
        if not self.home.exists():
            raise Refused("Run init first")
        with (self.home / (name + ".lock")).open("a+") as lockfile:
            try:
                fcntl.flock(lockfile.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as exc:
                raise Refused(f"{name} operation busy; retry later") from exc
            self.fds.append(lockfile.fileno())
            try:
                yield
            finally:
                self.fds.remove(lockfile.fileno())
                # Close only this descriptor. LOCK_UN would also release surviving
                # children's inherited references to the same open file description.

    def event(self, con, attempt: str | None, kind: str, **details) -> None:
        con.execute(
            "INSERT INTO events(at,attempt,kind,details) VALUES(?,?,?,?)",
            (now(), attempt, kind, json.dumps(details, sort_keys=True)),
        )

    def initialize(self, config: dict) -> None:
        if self.db_path.exists():
            raise Refused("Already initialized or partial initialization; inspect, do not overwrite")
        self.home.mkdir(mode=0o700, parents=False, exist_ok=False)
        with self.lock("meta"):
            with self.db() as con:
                # Conservative rollback journal for the tiny prototype. WAL is optional,
                # not required; all writes are short and concurrency is low.
                con.execute("PRAGMA journal_mode=DELETE")
                con.executescript(SCHEMA)
                con.execute("INSERT INTO config VALUES(1,?)", (json.dumps(config),))
                self.event(con, None, "initialized", **config)

    def row(self, aid: str) -> dict:
        self.require_initialized()
        valid_id(aid)
        with self.db() as con:
            attempt = con.execute("SELECT * FROM attempts WHERE id=?", (aid,)).fetchone()
            receipt = con.execute(
                "SELECT details FROM events WHERE attempt=? AND kind='check_receipt' ORDER BY seq DESC LIMIT 1",
                (aid,),
            ).fetchone()
            events = con.execute(
                "SELECT seq,at,kind,details FROM events WHERE attempt=? ORDER BY seq", (aid,)
            ).fetchall()
        if attempt is None:
            raise Refused("Unknown attempt")
        return {
            **dict(attempt),
            "check_receipt": json.loads(receipt[0]) if receipt else None,
            "events": [{**dict(event), "details": json.loads(event["details"])} for event in events],
        }

    def insert_attempt(
        self,
        con,
        aid: str,
        ticket: str,
        owner: str,
        base: str,
        ticket_hash: str,
        path: Path,
        branch: str,
        worker_pid: int | None = None,
        worker_pid_start_ticks: str | None = None,
        worker_run_dir: str | None = None,
    ) -> None:
        try:
            con.execute(
                """
                INSERT INTO attempts(
                    id,ticket,owner,state,base,ticket_hash,path,branch,
                    worker_pid,worker_pid_start_ticks,worker_run_dir,created,updated
                )
                VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?)
                """,
                (
                    aid,
                    ticket,
                    owner,
                    "allocating",
                    base,
                    ticket_hash,
                    str(path),
                    branch,
                    worker_pid,
                    worker_pid_start_ticks,
                    worker_run_dir,
                    now(),
                    now(),
                ),
            )
        except sqlite3.IntegrityError as exc:
            raise Refused("Ticket already has an active claim") from exc

    def reserve_refresh(
        self, previous: dict, aid: str, base: str, path: Path, branch: str, head: str, commits: list[str],
    ) -> dict:
        """Transfer the active reservation and its journal in one durable transaction."""
        with self.tx() as con:
            current = con.execute("SELECT * FROM attempts WHERE id=?", (previous["id"],)).fetchone()
            identity = ("state", "ticket", "owner", "base", "ticket_hash", "path", "branch")
            if current is None or any(current[key] != previous[key] for key in identity):
                raise Refused("Refresh admission changed; preserving the original Attempt")
            self.set_state_with_event(con, previous["id"], "abandoned")
            self.event(con, previous["id"], "refresh_requested", trunk=base, head=head, commits=commits)
            failpoint("after_refresh_abandon")
            self.insert_attempt(
                con, aid, previous["ticket"], previous["owner"], base, previous["ticket_hash"], path, branch
            )
            self.event(con, aid, "allocating", previous=previous["id"], claim_transferred=True)
            self.event(con, aid, "refresh_started", previous=previous["id"], commits=commits)
        return self.row(aid)

    def busy_locks(self, names: tuple[str, ...] = ("check", "lane", "meta")) -> list[dict]:
        blockers = []
        for name in names:
            path = self.home / (name + ".lock")
            if not path.exists():
                continue
            with path.open("a+") as lockfile:
                try:
                    fcntl.flock(lockfile.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                except BlockingIOError:
                    blockers.append({"kind": "lock", "name": name, "path": str(path)})
                else:
                    fcntl.flock(lockfile.fileno(), fcntl.LOCK_UN)
        return blockers

    def change(self, aid: str, state: str, **fields) -> None:
        self.change_with_event(aid, state, event_kind=state, event_details=fields, **fields)

    def change_with_event(
        self,
        aid: str,
        state: str,
        event_kind: str,
        event_details: dict | None = None,
        **fields,
    ) -> None:
        permitted = {"base", "candidate", "integration", "check_path", "log"}
        if not set(fields) <= permitted:
            raise Refused("Invalid state update")
        with self.tx() as con:
            values = {"state": state, "updated": now(), **fields}
            assignments = ",".join(key + "=?" for key in values)
            con.execute(
                "UPDATE attempts SET " + assignments + " WHERE id=?",
                (*values.values(), aid),
            )
            self.event(con, aid, event_kind, **(event_details or {}))

    def record_event(self, aid: str, kind: str, **details) -> None:
        with self.tx() as con:
            self.event(con, aid, kind, **details)

    def latest_event(self, aid: str, kind: str, *, after_kind: str | None = None) -> dict | None:
        """Restrict an event to the latest boundary; a missing boundary matches nothing."""
        self.require_initialized()
        with self.db() as con:
            if after_kind is None:
                row = con.execute(
                    "SELECT details FROM events WHERE attempt=? AND kind=? ORDER BY seq DESC LIMIT 1",
                    (aid, kind),
                ).fetchone()
            else:
                row = con.execute(
                    """
                    SELECT details FROM events WHERE attempt=? AND kind=? AND seq > (
                        SELECT MAX(seq) FROM events WHERE attempt=? AND kind=?
                    ) ORDER BY seq DESC LIMIT 1
                    """,
                    (aid, kind, aid, after_kind),
                ).fetchone()
        return json.loads(row[0]) if row else None

    def records_incomplete(self, aid: str, error: str) -> None:
        with self.tx() as con:
            con.execute(
                "UPDATE attempts SET state=?,updated=? WHERE id=?",
                ("landed_records_incomplete", now(), aid),
            )
            self.event(
                con,
                aid,
                "landed_records_incomplete",
                error=error,
                repair="fix landing records, then run recover --confirm-stopped",
            )

    def update_claim_basis(self, aid: str, base: str, ticket_hash: str) -> None:
        with self.tx() as con:
            con.execute(
                "UPDATE attempts SET base=?,ticket_hash=?,updated=? WHERE id=?",
                (base, ticket_hash, now(), aid),
            )
            self.event(con, aid, "tracker_claimed", base=base, ticket_hash=ticket_hash)

    def set_state_with_event(self, con, aid: str, state: str) -> None:
        con.execute("UPDATE attempts SET state=?,updated=? WHERE id=?", (state, now(), aid))
        self.event(con, aid, state)

    def no_uncertain_landing(self) -> None:
        self.require_initialized()
        with self.db() as con:
            placeholders = ",".join("?" for _ in UNRESOLVED_LANDING)
            row = con.execute(
                f"SELECT id FROM attempts WHERE state IN ({placeholders}) LIMIT 1",
                UNRESOLVED_LANDING,
            ).fetchone()
        if row:
            raise Refused(f"Unresolved landing {row['id']}; reconcile first")

    def attempts(self) -> list[dict]:
        self.require_initialized()
        with self.db() as con:
            return [dict(row) for row in con.execute("SELECT * FROM attempts ORDER BY created")]

    def backup(self, destination: str) -> dict:
        requested = Path(destination).expanduser()
        if requested.is_symlink():
            raise Refused("Backup destination exists as a symlink")
        target = output_path(destination)
        with self.lock("lane"), self.lock("meta"), self.db() as source:
            # O_EXCL arbitrates even backups from independent repositories.
            fd = os.open(target, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
            os.close(fd)
            with sqlite3.connect(target) as dest:
                source.backup(dest)
        return {
            "database_backup": str(target),
            "note": "Also preserve Git refs, workspaces and logs; not a full-project checkpoint",
        }
