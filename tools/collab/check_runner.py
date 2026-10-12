"""Check runner that builds an Integration commit and records a Check receipt."""
from __future__ import annotations

import json
import os
from pathlib import Path
import re
import signal
import subprocess
import uuid

from common import Refused, child_environment, digest, failpoint, now
from ticket_source import split_corenius_ticket, ticket_sections
from legacy_journal import LegacyJournal


class CheckRunner:
    def __init__(self, store, workspace, tickets, config):
        self.store = store
        self.workspace = workspace
        self.tickets = tickets
        self.config = config

    def check(self, aid: str) -> dict:
        with self.store.lock("check"):
            attempt, integration, path, log = self._prepare_check(aid)
            failpoint("before_check_command")
            argv = self.config()["check_argv"]
            timeout_seconds = self.config().get("check_timeout_seconds")
            code, timed_out, command_error = self._run_command(aid, argv, path, log, timeout_seconds)
            passed, outcome = self._record_receipt(
                aid,
                attempt,
                integration,
                path,
                log,
                argv,
                code,
                timed_out,
                command_error,
                timeout_seconds,
            )
            if not passed:
                raise Refused(self._failure_message(outcome, log))
        return self.store.row(aid)

    def _prepare_check(self, aid: str) -> tuple[dict, str, Path, Path]:
        # The lane lock serializes check admission with landings, not the gate run.
        with self.store.lock("lane"), self.store.lock("meta"):
            self.store.no_uncertain_landing()
            LegacyJournal(self.workspace.common).require_no_pending()
            attempt = self.store.row(aid)
            if attempt["state"] not in ("submitted", "check_failed", "verified"):
                raise Refused("Submit a candidate first")
            if self.workspace.trunk() != attempt["base"]:
                raise Refused("Trunk moved; stop/abandon, claim fresh, cherry-pick contribution, submit/check again")
            self.tickets.same_contract(attempt, attempt["base"])
            tag = uuid.uuid4().hex[:12]
            config = self.config()
            path = Path(config["workspace_root"]) / f"check-{aid}-{tag}"
            log = self.store.home / f"check-{aid}-{tag}.log"
            self.store.change(aid, "checking", check_path=str(path), log=str(log), integration=None)
            self.workspace.git(
                "worktree",
                "add",
                "--detach",
                "--lock",
                "--reason",
                f"collab-check:{aid}",
                str(path),
                attempt["candidate"],
            )
            # Only the fresh detached verification worktree is rewritten.
            self.workspace.git("reset", "--soft", attempt["base"], cwd=path)
            meta, _ = self.tickets.ticket(attempt["ticket"], attempt["base"])
            if split_corenius_ticket(attempt["ticket"]):
                ticket_path = path / meta["path"]
            else:
                ticket_path = path / ".tickets" / (attempt["ticket"] + ".md")
            raw = ticket_path.read_bytes()
            if digest(raw) != attempt["ticket_hash"]:
                raise Refused("Candidate changed ticket bytes")
            if not split_corenius_ticket(attempt["ticket"]):
                prefix, header, suffix = ticket_sections(raw)
                header, count = re.subn(rb"(?m)^status:[^\r\n]*", b"status: closed", header)
                if count != 1:
                    raise Refused("Expected one status field")
                ticket_path.write_bytes(prefix + header + suffix)
                self.workspace.git("add", "--", ".tickets/" + attempt["ticket"] + ".md", cwd=path)
            message = (
                f"{attempt['ticket']}: accepted contribution\n\n"
                f"Collab-Attempt: {aid}\n"
                f"Worker-Candidate: {attempt['candidate']}\n"
            )
            self.workspace.git("commit", "-m", message, cwd=path)
            integration = self.workspace.git("rev-parse", "HEAD", cwd=path).strip()
            self.workspace.git("update-ref", f"refs/collab/checks/{aid}/{tag}", integration)
            self.store.change(aid, "checking", integration=integration)
        return attempt, integration, path, log

    def _run_command(
        self,
        aid: str,
        argv: list[str],
        path: Path,
        log: Path,
        timeout_seconds: float | None,
    ) -> tuple[int | None, bool, str | None]:
        code = None
        timed_out = False
        command_error = None
        with log.open("x", encoding="utf-8") as stream:
            started = {
                "attempt": aid,
                "subject": self.workspace.git("rev-parse", "HEAD", cwd=path).strip(),
                "argv": argv,
                "timeout_seconds": timeout_seconds,
                "started": now(),
            }
            stream.write(json.dumps(started) + "\n")
            stream.flush()
            try:
                process = subprocess.Popen(
                    argv,
                    cwd=path,
                    stdout=stream,
                    stderr=subprocess.STDOUT,
                    env=child_environment(),
                    pass_fds=tuple(self.store.fds),
                    start_new_session=True,
                )
                try:
                    code = process.wait(timeout=timeout_seconds)
                except subprocess.TimeoutExpired:
                    timed_out = True
                    self._terminate(process)
                    stream.write(f"check timed out after {timeout_seconds:g} seconds\n")
            except OSError as exc:
                command_error = str(exc)
                stream.write(command_error + "\n")
            stream.flush()
            os.fsync(stream.fileno())
        return code, timed_out, command_error

    def _record_receipt(
        self,
        aid: str,
        attempt: dict,
        integration: str,
        path: Path,
        log: Path,
        argv: list[str],
        code: int | None,
        timed_out: bool,
        command_error: str | None,
        timeout_seconds: float | None,
    ) -> tuple[bool, str]:
        with self.store.lock("lane"), self.store.lock("meta"):
            current = self.store.row(aid)
            if current["state"] != "checking":
                raise Refused("Check state changed concurrently")
            clean = False
            same = False
            identity = False
            head_observed = False
            no_landing = True
            errors = []
            try:
                self.store.no_uncertain_landing()
                LegacyJournal(self.workspace.common).require_no_pending()
            except Refused as exc:
                no_landing = False
                errors.append(str(exc))
            try:
                self.workspace.validate_attempt({**attempt, "check_path": str(path)}, "check_path", integration)
                identity = True
            except (Refused, OSError, ValueError) as exc:
                errors.append(str(exc))
            try:
                status = self.workspace.git(
                    "status",
                    "--porcelain=v1",
                    "-z",
                    "--untracked-files=all",
                    cwd=path,
                )
                clean = status == "" and not self.workspace.hidden_index(path)
                head = self.workspace.git("rev-parse", "HEAD", cwd=path).strip()
                head_observed = True
                same = head == integration
            except (Refused, OSError, ValueError) as exc:
                clean = False
                same = False
                identity = False
                errors.append(str(exc))
            target = None
            try:
                target = self.workspace.trunk()
            except (Refused, OSError, ValueError) as exc:
                errors.append(str(exc))
            passed = (
                code == 0
                and clean
                and same
                and identity
                and target == attempt["base"]
                and no_landing
                and not timed_out
                and command_error is None
            )
            outcome = self._outcome(
                timed_out,
                command_error,
                code,
                clean,
                same,
                identity,
                head_observed,
                target,
                attempt,
                no_landing,
            )
            log_hash = digest(log.read_bytes())
            with self.store.tx() as con:
                state = "verified" if passed else "check_failed"
                self.store.set_state_with_event(con, aid, state)
                self.store.event(
                    con,
                    aid,
                    "check_receipt",
                    version=2,
                    subject=integration,
                    argv=argv,
                    outcome=outcome,
                    exit_code=code,
                    timeout_seconds=timeout_seconds,
                    clean=clean,
                    same_head=same,
                    identity=identity,
                    base=attempt["base"],
                    candidate=attempt["candidate"],
                    check_path=str(path),
                    log=str(log),
                    trunk=target,
                    inspection_error="; ".join(errors) if errors else None,
                    log_sha256=log_hash,
                    passed=passed,
                )
                failpoint("before_check_receipt_commit")
            failpoint("after_check_receipt_commit")
        return passed, outcome

    def _outcome(
        self,
        timed_out: bool,
        command_error: str | None,
        code: int | None,
        clean: bool,
        same: bool,
        identity: bool,
        head_observed: bool,
        target: str | None,
        attempt: dict,
        no_landing: bool,
    ) -> str:
        if timed_out:
            return "timeout"
        if command_error is not None:
            return "error"
        if code != 0:
            return "failed"
        if head_observed and not same:
            return "changed_head"
        if not identity:
            return "identity_failed"
        if not clean:
            return "dirty"
        if target != attempt["base"]:
            return "trunk_moved"
        if not no_landing:
            return "unresolved_landing"
        return "passed"

    def _failure_message(self, outcome: str, log: Path) -> str:
        if outcome == "timeout":
            return f"Check timed out; log: {log}"
        if outcome == "trunk_moved":
            return f"Check failed because trunk moved; log: {log}"
        return f"Check did not pass ({outcome}); log: {log}"

    @staticmethod
    def _terminate(process: subprocess.Popen) -> None:
        try:
            os.killpg(process.pid, signal.SIGTERM)
        except ProcessLookupError:
            return
        try:
            process.wait(timeout=1)
        except subprocess.TimeoutExpired:
            try:
                os.killpg(process.pid, signal.SIGKILL)
            except ProcessLookupError:
                return
            process.wait(timeout=1)

