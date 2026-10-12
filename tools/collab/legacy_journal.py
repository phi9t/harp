"""Durable compatibility landing records without adopting an uninitialized ledger.

Callers hold the landing lane while inspecting or changing uncertain phases.
Initialized repositories also receive the same events in the collab state store.
"""
from __future__ import annotations

import json
import os
from pathlib import Path
import re
import tempfile
import uuid

from common import Refused, digest, now, valid_id
from landing_protocol import gate_workspace_status, process_group_alive, require_no_pending_legacy
from state_store import StateStore

UNCERTAIN = {"prepared", "push_started", "pushed", "local_started", "landed_records_incomplete"}


class LegacyJournal:
    def __init__(self, common: Path):
        self.home = common / "legacy-land"
        home = common / "collab"
        self.store = StateStore(home, home / "state.sqlite3")

    def require_no_pending(self) -> None:
        if self.store.load_config() is not None:
            self.store.no_uncertain_landing()
        require_no_pending_legacy(self.home.parent)

    def confirmed_push_rejection(self, record: dict) -> bool:
        return any(
            event["kind"] == "push_rejection"
            and event.get("source_ref") == record["tip"]
            and event.get("target_ref") == record["target_ref"]
            and isinstance(event.get("summary"), str)
            and event["summary"].startswith(("[rejected]", "[remote rejected]"))
            for event in record["events"]
        )

    def read(self, identity: str) -> dict:
        valid_id(identity)
        path = self.home / (identity + ".json")
        try:
            record = json.loads(path.read_text())
        except (OSError, ValueError) as exc:
            raise Refused(f"Cannot read compatibility landing {identity}: {exc}") from exc
        required = {
            "version", "id", "state", "events", "tip", "target_ref", "target_branch", "local_before", "repo",
            "target_checkout", "pushing", "remote", "remote_before", "record_checkout", "record_branch",
        }
        if (
            not isinstance(record, dict)
            or not required <= record.keys()
            or record["version"] != 1
            or record["id"] != identity
            or not isinstance(record["events"], list)
            or not all(isinstance(event, dict) and isinstance(event.get("kind"), str) for event in record["events"])
            or not isinstance(record["state"], str)
            or not isinstance(record["pushing"], bool)
            or not all(isinstance(record[key], str) for key in ("tip", "local_before", "repo", "target_ref"))
            or not re.fullmatch(r"[a-f0-9]{40,64}", record["tip"])
            or not re.fullmatch(r"[a-f0-9]{40,64}", record["local_before"])
            or not record["target_ref"].startswith("refs/heads/")
            or not Path(record["repo"]).is_absolute()
        ):
            raise Refused("Invalid compatibility landing record; preserve and inspect it")
        return record

    def require_gate_receipt(self, record: dict) -> None:
        receipts = [event for event in record["events"] if event["kind"] == "gate_receipt"]
        if not receipts:
            raise Refused("Compatibility landing has no bound gate receipt")
        receipt = receipts[-1]
        if (
            receipt.get("version") != 2
            or receipt.get("subject") != record["tip"]
            or receipt.get("base") != record["local_before"]
            or receipt.get("outcome") != "passed"
            or receipt.get("clean") is not True
            or receipt.get("same_head") is not True
            or receipt.get("identity") is not True
            or receipt.get("hidden_index") != []
            or receipt.get("quiescent") is not True
            or receipt.get("check_path") != record.get("check_path")
            or receipt.get("check_git_dir") != record.get("check_git_dir")
            or not isinstance(receipt.get("check_path"), str)
            or not isinstance(receipt.get("check_git_dir"), str)
            or not isinstance(receipt.get("process_groups"), list)
            or not all(isinstance(identity, int) and identity > 0 for identity in receipt["process_groups"])
            or digest(Path(receipt["log"]).read_bytes()) != receipt.get("log_sha256")
        ):
            raise Refused("Compatibility gate receipt or log no longer matches the exact subject")
        workspace = gate_workspace_status(
            Path(record["repo"]), Path(receipt["check_path"]), record["tip"], receipt["check_git_dir"],
        )
        if (
            not workspace["identity"]
            or not workspace["same_head"]
            or not workspace["clean"]
            or any(process_group_alive(identity) for identity in receipt["process_groups"])
        ):
            raise Refused("Compatibility gate workspace changed or remains active; preserve and inspect it")

    def begin(self, **plan) -> dict:
        record = {"version": 1, "id": uuid.uuid4().hex, "state": "checking", "events": [], **plan}
        self.change(record, "checking")
        if plan.get("override_owner"):
            self.event(record, "mainline_override", owner=plan["override_owner"], target=plan["target_ref"])
        return record

    def event(self, record: dict, kind: str, **details) -> None:
        event = {"at": now(), "kind": kind, **details}
        record["events"].append(event)
        self._write(record)
        if self.store.load_config() is not None:
            with self.store.tx() as con:
                recorded = {
                    "source": "legacy", "landing": record["id"], "candidate": record["tip"],
                    "target_ref": record["target_ref"], **details,
                }
                self.store.event(con, None, kind, **recorded)

    def change(self, record: dict, state: str, **details) -> None:
        record["state"] = state
        self.event(record, state, **details)

    def _write(self, record: dict) -> None:
        self.home.mkdir(mode=0o700, exist_ok=True)
        path = self.home / (record["id"] + ".json")
        fd, temporary = tempfile.mkstemp(prefix=".record-", dir=self.home)
        try:
            with os.fdopen(fd, "w") as handle:
                json.dump(record, handle, sort_keys=True)
                handle.write("\n")
                handle.flush()
                os.fsync(handle.fileno())
            os.replace(temporary, path)
            directory = os.open(self.home, os.O_RDONLY | os.O_DIRECTORY)
            try:
                os.fsync(directory)
            finally:
                os.close(directory)
        finally:
            if os.path.exists(temporary):
                os.unlink(temporary)
