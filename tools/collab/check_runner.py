"""Check runner that builds an Integration commit and records a Check receipt."""
from __future__ import annotations

import json
from pathlib import Path
import re
import subprocess
import uuid

from common import Refused, child_environment, digest, failpoint, now
from ticket_source import ticket_sections


class CheckRunner:
    def __init__(self, store, workspace, tickets, config):
        self.store = store
        self.workspace = workspace
        self.tickets = tickets
        self.config = config

    def check(self, aid: str) -> dict:
        # The lane lock serializes final checks, not all implementation or DB writes.
        with self.store.lock("lane"):
            with self.store.lock("meta"):
                self.store.no_uncertain_landing()
                attempt = self.store.row(aid)
                if attempt["state"] not in ("submitted", "check_failed", "verified"):
                    raise Refused("Submit a candidate first")
                if self.workspace.trunk() != attempt["base"]:
                    raise Refused(
                        "Trunk moved; stop/abandon, claim fresh, cherry-pick contribution, submit/check again"
                    )
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
                ticket_path = path / ".tickets" / (attempt["ticket"] + ".md")
                raw = ticket_path.read_bytes()
                if digest(raw) != attempt["ticket_hash"]:
                    raise Refused("Candidate changed ticket bytes")
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
            failpoint("before_check_command")
            argv = self.config()["check_argv"]
            code = -1
            with log.open("x") as stream:
                started = {
                    "attempt": aid,
                    "subject": integration,
                    "argv": argv,
                    "started": now(),
                }
                stream.write(json.dumps(started) + "\n")
                stream.flush()
                try:
                    process = subprocess.run(
                        argv,
                        cwd=path,
                        stdout=stream,
                        stderr=subprocess.STDOUT,
                        env=child_environment(),
                        pass_fds=tuple(self.store.fds),
                    )
                    code = process.returncode
                except OSError as exc:
                    stream.write(str(exc) + "\n")
                stream.flush()
                os_fsync(stream.fileno())
            with self.store.lock("meta"):
                current = self.store.row(aid)
                if current["state"] != "checking":
                    raise Refused("Check state changed concurrently")
                clean = False
                same = False
                identity = False
                errors = []
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
                    same = self.workspace.git("rev-parse", "HEAD", cwd=path).strip() == integration
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
                passed = code == 0 and clean and same and identity and target == attempt["base"]
                log_hash = digest(log.read_bytes())
                with self.store.tx() as con:
                    state = "verified" if passed else "check_failed"
                    self.store.set_state_with_event(con, aid, state)
                    self.store.event(
                        con,
                        aid,
                        "check_receipt",
                        subject=integration,
                        argv=argv,
                        exit_code=code,
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
            if not passed:
                raise Refused(f"Check failed, changed its workspace, or trunk moved; log: {log}")
        return self.store.row(aid)


def os_fsync(fd: int) -> None:
    import os

    os.fsync(fd)
