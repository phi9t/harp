from __future__ import annotations

import hashlib
import itertools
import json
import os
import re
import signal
import stat
import subprocess
import tempfile
import threading
import time
from contextlib import contextmanager
from dataclasses import dataclass
from pathlib import Path
from typing import Callable, Iterator

if __package__:
    from . import execution_ledger, local_formalization_validation, route_validation
else:  # pragma: no cover - direct script execution
    import execution_ledger  # type: ignore
    import local_formalization_validation  # type: ignore
    import route_validation  # type: ignore


MAX_GOAL_BYTES = 1024 * 1024
GIT_BINARY = "/usr/bin/git"
GIT_TIMEOUT_SECONDS = 5
GIT_OUTPUT_CAP = 8192
GIT_POLL_INTERVAL_SECONDS = 0.01
PLAN_BYTES_CAP = 1024 * 1024
LEDGER_BYTES_CAP = 256 * 1024
ARTIFACT_BYTES_CAP = 1024 * 1024
FOOTER_ORDER = (
    "goal_status",
    "jin",
    "lorist_schwenninger",
    "harp",
    "local_bundle",
    "reader_surfaces",
    "program_verifier",
    "master_landing",
    "worktree_audit",
    "post_execution_review",
    "plan_revision",
)
FIXED_PLAN_PATH = Path(
    "docs/superpowers/plans/2026-08-23-crouzeix-end-to-end-proof-goal.md"
)
FIXED_LEDGER_PATH = Path(
    "docs/workstream/crouzeix-proof-reproduction/execution-ledger-003.tsv"
)
GIT_ID_RE = re.compile(r"[0-9a-f]{40,64}\Z")
READER_EVIDENCE_PATHS = (
    "atlas/src/content/generated/corpus.json",
    "atlas/dist/harp-atlas.html",
    "atlas/dist/harp-atlas.receipt.json",
)
ATLAS_APP_INPUT_PATHS = (
    "index.html",
    "package.json",
    "scripts/export-static.mjs",
    "tsconfig.json",
    "vite.config.ts",
    "vitest.config.ts",
)
PHASE_ORDER = (
    "cpfr-085",
    "cpfr-086",
    "cpfr-087",
    "cpfr-088",
    "cpfr-089",
    "cpfr-090",
    "cpfr-091",
    "phase-8-plan-review",
)
PROGRAM_PHASES = PHASE_ORDER[:-1]
LANDED_ROW_CONTRACT = {
    "cpfr-085": {
        "ticket": "CPFR-085",
        "verifier": "evidence/crouzeix_conjecture/routes/jin/receipt.json",
    },
    "cpfr-086": {
        "ticket": "CPFR-086",
        "verifier": "evidence/crouzeix_conjecture/routes/lorist-schwenninger/receipt.json",
    },
    "cpfr-087": {
        "ticket": "CPFR-087",
        "verifier": "evidence/crouzeix_conjecture/routes/harp/receipt.json",
    },
    "cpfr-088": {
        "ticket": "CPFR-088",
        "verifier": "evidence/crouzeix_conjecture/local_formalization/manifest.tsv",
    },
    "cpfr-089": {
        "ticket": "CPFR-089",
        "verifier": "atlas/dist/harp-atlas.receipt.json",
    },
    "cpfr-090": {
        "ticket": "CPFR-090",
        "verifier": "docs/workstream/crouzeix-proof-reproduction/program-verification-003.json",
    },
    "cpfr-091": {
        "ticket": "CPFR-091",
        "verifier": "docs/workstream/crouzeix-proof-reproduction/worktree-inventory-002.md",
    },
    "phase-8-plan-review": {
        "ticket": "PHASE-8-PLAN-REVIEW",
        "verifier": "docs/workstream/crouzeix-proof-reproduction/retrospective-003-review.json",
    },
}
PHASE8_REVIEW_FIELDS = frozenset(
    {
        "schema_version",
        "spec_status",
        "standards_status",
        "retrospective_path",
        "retrospective_sha256",
        "plan_path",
        "plan_sha256",
    }
)


@dataclass
class GitSession:
    directory: tempfile.TemporaryDirectory
    env: dict[str, str]
    counter: itertools.count


@dataclass(frozen=True)
class GoalValidationResult:
    goal_status: str
    earliest_incomplete_phase: str | None
    verified_commits: tuple[str, ...]
    evidence_paths: tuple[str, ...]


def canonical_json(value: object) -> str:
    return (
        json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        )
        + "\n"
    )


def _display_path(repository_root: Path, path: Path) -> str:
    try:
        return Path(path).resolve().relative_to(Path(repository_root).resolve()).as_posix()
    except ValueError:
        return Path(path).as_posix()


def parse_goal_footer(goal_path: Path, *, repository_root: Path | None = None) -> dict[str, str | None]:
    source = _read_trusted_repository_text_artifact(
        goal_path,
        label="goal plan",
        max_bytes=PLAN_BYTES_CAP,
        repository_root=repository_root,
    )
    lines = source.splitlines(keepends=True)
    marker_index = -1
    for index, line in enumerate(lines):
        if line.rstrip("\n") == "## Completion footer":
            marker_index = index
    if marker_index < 0:
        raise ValueError("completion footer is missing")
    open_index = marker_index + 2
    if open_index >= len(lines) or lines[open_index].rstrip("\n") != "```yaml":
        raise ValueError("completion footer fence is missing")
    close_index = None
    payload_lines: list[str] = []
    for index in range(open_index + 1, len(lines)):
        stripped = lines[index].rstrip("\n")
        if stripped == "```":
            close_index = index
            break
        payload_lines.append(lines[index].rstrip("\n"))
    if close_index is None:
        raise ValueError("completion footer fence is unterminated")
    for trailing in lines[close_index + 1 :]:
        if trailing.strip():
            raise ValueError("completion footer has trailing content")
    lines = [line for line in payload_lines if line.strip()]
    observed_keys: list[str] = []
    result: dict[str, str | None] = {}
    for line in lines:
        if ":" not in line:
            raise ValueError("malformed footer line")
        key, raw_value = line.split(":", 1)
        key = key.strip()
        value = raw_value.strip()
        if key in result:
            raise ValueError("duplicate footer key")
        if key not in FOOTER_ORDER:
            raise ValueError("unknown footer key")
        observed_keys.append(key)
        result[key] = None if value == "null" else value
    for key in FOOTER_ORDER:
        if key not in result:
            raise ValueError(f"missing footer key: {key}")
    if tuple(observed_keys) != FOOTER_ORDER:
        raise ValueError("footer keys are out of order")
    return result


def _validate_footer_values(footer: dict[str, str | None]) -> None:
    goal_status = footer["goal_status"]
    if goal_status not in {"in-progress", "complete"}:
        raise ValueError("goal_status value is invalid")
    if goal_status == "in-progress":
        allowed = {
            "jin": {"incomplete", "complete-local"},
            "lorist_schwenninger": {"incomplete", "complete-local"},
            "harp": {"incomplete", "complete-local"},
            "local_bundle": {"absent-or-optional", "required-and-valid"},
            "reader_surfaces": {"unreconciled", "reconciled"},
            "program_verifier": {"pending", "passed"},
            "worktree_audit": {"pending", "passed"},
            "post_execution_review": {"pending", "passed"},
        }
        for key, choices in allowed.items():
            if footer[key] not in choices:
                raise ValueError(f"{key} value is invalid")
        if footer["master_landing"] is not None:
            raise ValueError("master_landing value is invalid")
        if footer["plan_revision"] is not None:
            raise ValueError("plan_revision value is invalid")
        return
    allowed = {
        "jin": {"complete-local"},
        "lorist_schwenninger": {"complete-local"},
        "harp": {"complete-local"},
        "local_bundle": {"required-and-valid"},
        "reader_surfaces": {"reconciled"},
        "program_verifier": {"passed"},
        "worktree_audit": {"passed"},
        "post_execution_review": {"passed"},
    }
    for key, choices in allowed.items():
        if footer[key] not in choices:
            raise ValueError(f"{key} value is invalid")
    if footer["master_landing"] is None:
        raise ValueError("master_landing value is invalid")
    if footer["plan_revision"] is None:
        raise ValueError("plan_revision value is invalid")


@contextmanager
def git_session() -> Iterator[GitSession]:
    directory = tempfile.TemporaryDirectory(prefix="harp-goal-validation-git-")
    env = {
        "HOME": directory.name,
        "GIT_CONFIG_NOSYSTEM": "1",
        "GIT_CONFIG_GLOBAL": str(Path(directory.name) / "gitconfig"),
        "LC_ALL": "C",
        "PATH": "/usr/bin:/bin",
    }
    env["GIT_CONFIG_NOSYSTEM"] = "1"
    session = GitSession(directory=directory, env=env, counter=itertools.count())
    try:
        yield session
    finally:
        directory.cleanup()


class _BoundedPipe:
    def __init__(self, maximum: int) -> None:
        self._maximum = maximum
        self._chunks: list[bytes] = []
        self._size = 0
        self.truncated = False
        self._thread: threading.Thread | None = None

    @property
    def data(self) -> bytes:
        return b"".join(self._chunks)

    def start(self, stream: object) -> None:
        if stream is None:
            return
        self._thread = threading.Thread(target=self._read, args=(stream,), daemon=True)
        self._thread.start()

    def join(self, timeout: float | None = None) -> bool:
        if self._thread is None:
            return True
        self._thread.join(timeout)
        return not self._thread.is_alive()

    def _read(self, stream: object) -> None:
        with stream:  # type: ignore[arg-type]
            reader = getattr(stream, "read1", None)
            while True:
                if reader is not None:
                    chunk = reader(4096)
                else:
                    chunk = stream.read(4096)  # type: ignore[attr-defined]
                if not chunk:
                    return
                if self._size < self._maximum + 1:
                    keep = min(len(chunk), self._maximum + 1 - self._size)
                    if keep:
                        self._chunks.append(chunk[:keep])
                        self._size += keep
                if self._size > self._maximum:
                    self.truncated = True


def _terminate_process_tree(process: subprocess.Popen[bytes]) -> None:
    try:
        os.killpg(process.pid, signal.SIGKILL)
    except ProcessLookupError:
        return
    except (AttributeError, OSError):
        try:
            process.kill()
        except ProcessLookupError:
            return


def _ensure_process_reaped(process: subprocess.Popen[bytes]) -> None:
    try:
        process.wait(timeout=1.0)
    except subprocess.TimeoutExpired:
        _terminate_process_tree(process)
        try:
            process.wait(timeout=1.0)
        except subprocess.TimeoutExpired as error:
            raise ValueError("git command failed to terminate cleanly") from error


def _run_git_bounded(
    repository_root: Path,
    session: GitSession,
    *args: str,
    executable: str | Path = GIT_BINARY,
) -> subprocess.CompletedProcess[str]:
    argv = [str(executable), "-C", str(Path(repository_root).resolve()), *args]
    stdout = _BoundedPipe(GIT_OUTPUT_CAP)
    stderr = _BoundedPipe(GIT_OUTPUT_CAP)
    try:
        process = subprocess.Popen(
            argv,
            cwd=repository_root,
            env=session.env,
            stdin=subprocess.DEVNULL,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
    except (FileNotFoundError, OSError) as error:
        raise ValueError(f"git command failed: {error}") from error
    stdout.start(process.stdout)
    stderr.start(process.stderr)
    termination_error: ValueError | None = None
    run_error: ValueError | None = None
    returncode: int | None = None
    try:
        deadline = time.monotonic() + GIT_TIMEOUT_SECONDS
        while True:
            returncode = process.poll()
            if returncode is not None:
                break
            if stdout.truncated:
                run_error = ValueError("git stdout exceeds cap")
                break
            if stderr.truncated:
                run_error = ValueError("git stderr exceeds cap")
                break
            if time.monotonic() >= deadline:
                run_error = ValueError("git command timed out")
                break
            time.sleep(GIT_POLL_INTERVAL_SECONDS)
    finally:
        if process.poll() is None:
            _terminate_process_tree(process)
        try:
            _ensure_process_reaped(process)
        except ValueError as error:
            termination_error = error
        stdout.join(1.0)
        stderr.join(1.0)
        if process.stdout is not None:
            process.stdout.close()
        if process.stderr is not None:
            process.stderr.close()
    if termination_error is not None:
        raise termination_error
    if run_error is not None:
        raise run_error
    if stdout.truncated:
        raise ValueError("git stdout exceeds cap")
    if stderr.truncated:
        raise ValueError("git stderr exceeds cap")
    try:
        stdout_text = stdout.data.decode("utf-8")
        stderr_text = stderr.data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ValueError("git output is not valid UTF-8") from error
    return subprocess.CompletedProcess(argv, returncode, stdout_text, stderr_text)


def _run_git_checked(
    repository_root: Path,
    session: GitSession,
    runner: Callable[..., subprocess.CompletedProcess[str]] | None,
    *args: str,
    executable: str | Path = GIT_BINARY,
) -> subprocess.CompletedProcess[str]:
    argv = [str(executable), "-C", str(Path(repository_root).resolve()), *args]
    if runner is None:
        return _run_git_bounded(repository_root, session, *args, executable=executable)
    try:
        result = runner(
            argv,
            cwd=str(Path(repository_root).resolve()),
            env=session.env,
            timeout=GIT_TIMEOUT_SECONDS,
            capture_output=True,
            text=True,
            check=False,
        )
    except subprocess.TimeoutExpired as error:
        raise ValueError("git command timed out") from error
    stdout = result.stdout or ""
    stderr = result.stderr or ""
    if len(stdout.encode("utf-8")) > GIT_OUTPUT_CAP:
        raise ValueError("git stdout exceeds cap")
    if len(stderr.encode("utf-8")) > GIT_OUTPUT_CAP:
        raise ValueError("git stderr exceeds cap")
    return subprocess.CompletedProcess(argv, result.returncode, stdout, stderr)


def _git_commit_exists(
    repository_root: Path,
    commit_id: str,
    *,
    session: GitSession | None = None,
    runner: Callable[..., subprocess.CompletedProcess[str]] | None = None,
) -> bool:
    if not GIT_ID_RE.fullmatch(commit_id):
        return False
    if session is None:
        with git_session() as managed_session:
            return _git_commit_exists(
                repository_root,
                commit_id,
                session=managed_session,
                runner=runner,
            )
    result = _run_git_checked(
        repository_root,
        session,
        runner,
        "rev-parse",
        "--verify",
        f"{commit_id}^{{commit}}",
    )
    if result.returncode != 0:
        return False
    output = result.stdout.strip()
    if not GIT_ID_RE.fullmatch(output):
        raise ValueError("git output is malformed")
    return output.startswith(commit_id)


def _git_is_ancestor(
    repository_root: Path,
    older: str,
    newer_ref: str,
    *,
    session: GitSession | None = None,
    runner: Callable[..., subprocess.CompletedProcess[str]] | None = None,
) -> bool:
    if session is None:
        with git_session() as managed_session:
            return _git_is_ancestor(
                repository_root,
                older,
                newer_ref,
                session=managed_session,
                runner=runner,
            )
    result = _run_git_checked(
        repository_root,
        session,
        runner,
        "merge-base",
        "--is-ancestor",
        older,
        newer_ref,
    )
    if result.returncode == 0:
        return True
    if result.returncode == 1:
        return False
    stderr = result.stderr.strip() or "unknown git error"
    raise ValueError(f"git command failed: {stderr}")


def _validate_commit_reference(
    repository_root: Path,
    commit_id: str | None,
    *,
    label: str,
    session: GitSession,
) -> str:
    if commit_id is None or not GIT_ID_RE.fullmatch(commit_id):
        raise ValueError(f"{label} is invalid")
    if not _git_commit_exists(repository_root, commit_id, session=session):
        raise ValueError(f"{label} does not exist")
    if not _git_is_ancestor(repository_root, commit_id, "master", session=session):
        raise ValueError(f"{label} is not an ancestor of master")
    return commit_id


def _validate_phase_progression(ledger: execution_ledger.ExecutionLedger) -> None:
    earlier_phases_landed = True
    for phase in PHASE_ORDER:
        latest_state = ledger.latest_state(phase)
        if latest_state is not None and not earlier_phases_landed:
            raise ValueError("later phase appears before earlier landed phase")
        if latest_state != "landed":
            earlier_phases_landed = False


def _earliest_incomplete_phase(ledger: execution_ledger.ExecutionLedger) -> str | None:
    for phase in PHASE_ORDER:
        if ledger.latest_state(phase) != "landed":
            return phase
    return None


def _validate_landed_rows(ledger: execution_ledger.ExecutionLedger) -> None:
    for row in ledger.rows:
        if row.state != "landed":
            continue
        contract = LANDED_ROW_CONTRACT[row.phase]
        if row.ticket != contract["ticket"]:
            raise ValueError(
                "Phase 8 plan-review row is invalid"
                if row.phase == "phase-8-plan-review"
                else "landed row ticket is invalid"
            )
        if not row.verifier:
            raise ValueError("landed row requires verifier")
        if row.verifier != contract["verifier"]:
            raise ValueError("landed row verifier path is invalid")
        if row.exit_code != 0:
            raise ValueError("landed row requires zero exit code")
        if not row.evidence_digest:
            raise ValueError("landed row requires evidence digest")
        if not row.landing_commit:
            raise ValueError("landed row requires landing commit")


def _validate_landed_row_evidence(
    repository_root: Path,
    ledger: execution_ledger.ExecutionLedger,
    *,
    session: GitSession,
) -> tuple[str, ...]:
    evidence_paths: list[str] = []
    landed_rows: list[execution_ledger.ExecutionLedgerRow] = []
    for row in ledger.rows:
        if row.state != "landed":
            continue
        landed_rows.append(row)
        evidence_bytes = _read_bounded_artifact(repository_root, row.verifier)
        if row.evidence_digest != f"sha256:{_sha256_bytes(evidence_bytes)}":
            raise ValueError("landed row evidence digest mismatch")
        if row.phase == "phase-8-plan-review":
            _validate_phase8_review_artifact(repository_root, row.verifier)
        evidence_paths.append(_display_path(repository_root, Path(repository_root) / row.verifier))
    for row in landed_rows:
        if not _git_commit_exists(repository_root, row.landing_commit, session=session):
            raise ValueError("landed row landing_commit does not exist")
        if not _git_is_ancestor(repository_root, row.landing_commit, "master", session=session):
            raise ValueError("landed row landing_commit is not an ancestor of master")
    return tuple(evidence_paths)


def _require_landed_program_phases(ledger: execution_ledger.ExecutionLedger) -> None:
    missing = [phase for phase in PROGRAM_PHASES if not ledger.has_landed_phase(phase)]
    if missing:
        raise ValueError("complete goal requires landed rows for CPFR-085 through CPFR-091")
    if not ledger.has_landed_phase("phase-8-plan-review"):
        raise ValueError("complete goal requires a passed Phase 8 plan-review row")


def _validate_route_claims(repository_root: Path) -> None:
    for route_id in ("jin", "lorist-schwenninger", "harp"):
        result = route_validation.inspect_route(repository_root, route_id)
        if result.claim_level != "complete-local" or result.status != "complete":
            raise ValueError(f"{route_id} route is not complete-local")


def _validate_local_bundle(repository_root: Path) -> tuple[str, ...]:
    validator = local_formalization_validation
    if validator is None:
        raise ValueError("local formalization bundle is required")
    try:
        result = validator.validate_local_formalization_bundle(Path(repository_root))
    except Exception as error:
        raise ValueError(
            f"local formalization bundle is required: {error}"
        ) from error
    if getattr(result, "status", None) != "passed":
        raise ValueError("local formalization bundle is required")
    manifest = (
        Path(repository_root)
        / "evidence/crouzeix_conjecture/local_formalization/manifest.tsv"
    )
    return (_display_path(repository_root, manifest),)


def _sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _safe_relative_artifact_path(value: str) -> str:
    path = Path(value)
    if not value or path.is_absolute() or "\\" in value:
        raise ValueError("landed row verifier path is invalid")
    normalized = path.as_posix()
    parts = normalized.split("/")
    if any(part in {"", ".", ".."} for part in parts):
        raise ValueError("landed row verifier path is invalid")
    return normalized


def _read_repository_artifact(
    repository_root: Path,
    relative: str | Path,
    *,
    label: str,
    max_bytes: int,
    missing_message: str,
) -> bytes:
    relative_path = _safe_relative_artifact_path(Path(relative).as_posix())
    root_fd = os.open(Path(repository_root).resolve(), os.O_RDONLY)
    current_fd = root_fd
    try:
        parts = Path(relative_path).parts
        for index, part in enumerate(parts):
            is_last = index == len(parts) - 1
            flags = os.O_RDONLY | getattr(os, "O_NOFOLLOW", 0)
            try:
                next_fd = os.open(part, flags, dir_fd=current_fd)
            except OSError as error:
                if error.errno in {getattr(os, "ELOOP", 40), getattr(os, "EMLINK", 31), 40, 62}:
                    raise ValueError(f"{label} path is invalid") from error
                raise
            if current_fd != root_fd:
                os.close(current_fd)
            current_fd = next_fd
            stat_result = os.fstat(current_fd)
            if is_last:
                if not stat.S_ISREG(stat_result.st_mode) or stat_result.st_nlink != 1:
                    raise ValueError(f"{label} path is invalid")
                if stat_result.st_size > max_bytes:
                    raise ValueError(f"{label} exceeds cap")
            else:
                if not stat.S_ISDIR(stat_result.st_mode):
                    raise ValueError(f"{label} path is invalid")
        identity_before = os.fstat(current_fd)
        size = identity_before.st_size
        data = b""
        while len(data) < size:
            chunk = os.read(current_fd, min(4096, size - len(data)))
            if not chunk:
                break
            data += chunk
            if len(data) > max_bytes:
                raise ValueError(f"{label} exceeds cap")
        identity_after = os.fstat(current_fd)
        if identity_after.st_ino != identity_before.st_ino or identity_after.st_dev != identity_before.st_dev:
            raise ValueError(f"{label} path is invalid")
        return data
    except FileNotFoundError as error:
        raise ValueError(missing_message) from error
    finally:
        os.close(current_fd)
        if root_fd != current_fd:
            os.close(root_fd)


def _trusted_relative_path(path: Path, *, repository_root: Path) -> str:
    root = Path(repository_root)
    target = Path(path)
    if target.is_absolute():
        try:
            relative = target.relative_to(root)
        except ValueError as error:
            raise ValueError("unsafe path") from error
    else:
        relative = target
    if not relative.parts:
        raise ValueError("unsafe path")
    try:
        return _safe_relative_artifact_path(relative.as_posix())
    except ValueError as error:
        raise ValueError("unsafe path") from error


def _read_trusted_repository_text_artifact(
    path: Path,
    *,
    label: str,
    max_bytes: int,
    repository_root: Path | None,
) -> str:
    if repository_root is None:
        raise ValueError(f"{label} path is invalid")
    try:
        relative = _trusted_relative_path(path, repository_root=Path(repository_root))
        data = _read_repository_artifact(
            Path(repository_root),
            relative,
            label=label,
            max_bytes=max_bytes,
            missing_message=f"{label} is missing",
        )
    except ValueError as error:
        message = str(error)
        if message == f"{label} is missing":
            raise
        raise ValueError(f"{label} path is invalid") from error
    try:
        return data.decode("utf-8")
    except UnicodeDecodeError as error:
        raise ValueError(f"{label} path is invalid") from error


def _load_execution_ledger_checked(
    path: Path,
    *,
    repository_root: Path,
) -> execution_ledger.ExecutionLedger:
    source = _read_trusted_repository_text_artifact(
        path,
        label="execution ledger",
        max_bytes=LEDGER_BYTES_CAP,
        repository_root=repository_root,
    )
    lines = source.splitlines()
    if not lines or lines[0] != execution_ledger.HEADER:
        raise ValueError("execution ledger header is invalid")
    rows: list[execution_ledger.ExecutionLedgerRow] = []
    latest_phase_state: dict[str, int] = {}
    for raw_line in lines[1:]:
        if not raw_line:
            continue
        row = execution_ledger._parse_row(raw_line)
        current_state = execution_ledger.STATE_INDEX[row.state]
        previous_state = latest_phase_state.get(row.phase)
        if previous_state is not None:
            if current_state == previous_state:
                raise ValueError("duplicate phase transition")
            if current_state < previous_state:
                raise ValueError("non-monotone state")
        latest_phase_state[row.phase] = current_state
        rows.append(row)
    return execution_ledger.ExecutionLedger(Path(path), tuple(rows))


def _read_bounded_artifact(repository_root: Path, relative: str) -> bytes:
    return _read_repository_artifact(
        repository_root,
        relative,
        label="landed row verifier",
        max_bytes=ARTIFACT_BYTES_CAP,
        missing_message="landed row evidence is missing",
    )


def _validate_phase8_review_artifact(repository_root: Path, relative: str) -> None:
    try:
        payload = json.loads(_read_bounded_artifact(repository_root, relative).decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise ValueError("Phase 8 plan-review row is invalid") from error
    if not isinstance(payload, dict) or set(payload) != PHASE8_REVIEW_FIELDS:
        raise ValueError("Phase 8 plan-review row is invalid")
    if payload.get("schema_version") != "crouzeix-plan-review/v1":
        raise ValueError("Phase 8 plan-review row is invalid")
    if payload.get("spec_status") != "passed" or payload.get("standards_status") != "passed":
        raise ValueError("Phase 8 plan-review row is invalid")
    retrospective_path = payload.get("retrospective_path")
    plan_path = payload.get("plan_path")
    if not isinstance(retrospective_path, str) or not isinstance(plan_path, str):
        raise ValueError("Phase 8 plan-review row is invalid")
    retrospective_bytes = _read_bounded_artifact(repository_root, retrospective_path)
    plan_bytes = _read_bounded_artifact(repository_root, plan_path)
    if payload.get("retrospective_sha256") != _sha256_bytes(retrospective_bytes):
        raise ValueError("Phase 8 plan-review row is invalid")
    if payload.get("plan_sha256") != _sha256_bytes(plan_bytes):
        raise ValueError("Phase 8 plan-review row is invalid")


def _atlas_input_digest(repository_root: Path) -> str | None:
    atlas_root = Path(repository_root) / "atlas"
    source_root = atlas_root / "src"
    if not atlas_root.exists() or not source_root.exists():
        return None
    if any(not (atlas_root / relative).exists() for relative in ATLAS_APP_INPUT_PATHS):
        return None
    inputs: list[tuple[str, bytes]] = []
    for relative in ATLAS_APP_INPUT_PATHS:
        inputs.append(
            (
                relative,
                _read_repository_artifact(
                    atlas_root,
                    relative,
                    label="atlas artifact",
                    max_bytes=ARTIFACT_BYTES_CAP,
                    missing_message="generated-reader receipt is required",
                ),
            )
        )
    for path in sorted(source_root.rglob("*")):
        if not path.is_file() or path.suffix not in {".css", ".ts", ".tsx"}:
            continue
        relative = path.relative_to(atlas_root).as_posix()
        inputs.append(
            (
                relative,
                _read_repository_artifact(
                    atlas_root,
                    relative,
                    label="atlas artifact",
                    max_bytes=ARTIFACT_BYTES_CAP,
                    missing_message="generated-reader receipt is required",
                ),
            )
        )
    inputs.sort(key=lambda item: item[0])
    digest = hashlib.sha256()
    for relative, content in inputs:
        digest.update(relative.encode("utf-8"))
        digest.update(b"\0")
        digest.update(content)
    return digest.hexdigest()


def _validate_reader_surfaces(repository_root: Path) -> tuple[str, ...]:
    missing = [
        relative
        for relative in READER_EVIDENCE_PATHS
        if not (Path(repository_root) / relative).exists()
    ]
    if missing:
        raise ValueError("generated-reader receipt is required")
    corpus_path = Path(repository_root) / READER_EVIDENCE_PATHS[0]
    html_path = Path(repository_root) / READER_EVIDENCE_PATHS[1]
    receipt_path = Path(repository_root) / READER_EVIDENCE_PATHS[2]
    try:
        receipt = json.loads(
            _read_repository_artifact(
                repository_root,
                READER_EVIDENCE_PATHS[2],
                label="atlas artifact",
                max_bytes=ARTIFACT_BYTES_CAP,
                missing_message="generated-reader receipt is required",
            ).decode("utf-8")
        )
    except (UnicodeDecodeError, json.JSONDecodeError, ValueError) as error:
        if isinstance(error, ValueError) and str(error) == "atlas artifact exceeds cap":
            raise
        raise ValueError("generated-reader receipt is invalid") from error
    if not isinstance(receipt, dict) or receipt.get("schema_version") != "harp-atlas-export/v1":
        raise ValueError("generated-reader receipt is invalid")
    corpus_bytes = _read_repository_artifact(
        repository_root,
        READER_EVIDENCE_PATHS[0],
        label="atlas artifact",
        max_bytes=ARTIFACT_BYTES_CAP,
        missing_message="generated-reader receipt is required",
    )
    html_bytes = _read_repository_artifact(
        repository_root,
        READER_EVIDENCE_PATHS[1],
        label="atlas artifact",
        max_bytes=ARTIFACT_BYTES_CAP,
        missing_message="generated-reader receipt is required",
    )
    if receipt.get("corpus_sha256") != _sha256_bytes(corpus_bytes):
        raise ValueError("atlas receipt corpus_sha256 mismatch")
    if receipt.get("html_sha256") != _sha256_bytes(html_bytes):
        raise ValueError("atlas receipt html_sha256 mismatch")
    app_inputs_sha256 = _atlas_input_digest(repository_root)
    if (
        app_inputs_sha256 is not None
        and receipt.get("app_inputs_sha256") != app_inputs_sha256
    ):
        raise ValueError("atlas receipt app_inputs_sha256 mismatch")
    return READER_EVIDENCE_PATHS


def validate_goal(
    goal_path: Path,
    *,
    execution_ledger_path: Path,
    repository_root: Path,
) -> GoalValidationResult:
    footer = parse_goal_footer(goal_path, repository_root=repository_root)
    _validate_footer_values(footer)
    ledger = _load_execution_ledger_checked(
        execution_ledger_path,
        repository_root=repository_root,
    )
    _validate_phase_progression(ledger)
    evidence_paths = [_display_path(repository_root, execution_ledger_path)]
    if footer["goal_status"] != "complete":
        return GoalValidationResult(
            goal_status=str(footer["goal_status"]),
            earliest_incomplete_phase=_earliest_incomplete_phase(ledger),
            verified_commits=(),
            evidence_paths=tuple(evidence_paths),
        )

    _validate_landed_rows(ledger)
    _require_landed_program_phases(ledger)
    with git_session() as session:
        evidence_paths.extend(_validate_landed_row_evidence(repository_root, ledger, session=session))
        master_landing = _validate_commit_reference(
            repository_root,
            footer["master_landing"],
            label="master_landing",
            session=session,
        )
        plan_revision = _validate_commit_reference(
            repository_root,
            footer["plan_revision"],
            label="plan_revision",
            session=session,
        )
        if not _git_is_ancestor(
            repository_root,
            master_landing,
            plan_revision,
            session=session,
        ):
            raise ValueError("plan_revision is older than master_landing")
    _validate_route_claims(repository_root)
    evidence_paths.extend(_validate_local_bundle(repository_root))
    evidence_paths.extend(_validate_reader_surfaces(repository_root))
    return GoalValidationResult(
        goal_status="complete",
        earliest_incomplete_phase=None,
        verified_commits=(master_landing, plan_revision),
        evidence_paths=tuple(dict.fromkeys(evidence_paths)),
    )


def validate_goal_payload(
    repository_root: Path,
    *,
    goal_path: Path | None = None,
    execution_ledger_path: Path | None = None,
) -> tuple[int, dict[str, object]]:
    repo_root = Path(repository_root)
    target_goal = goal_path or repo_root / FIXED_PLAN_PATH
    target_ledger = execution_ledger_path or repo_root / FIXED_LEDGER_PATH
    try:
        result = validate_goal(
            target_goal,
            execution_ledger_path=target_ledger,
            repository_root=repo_root,
        )
    except ValueError as error:
        message = str(error)
        payload = {
            "schema_version": "crouzeix-goal-validation/v1",
            "status": "invalid",
            "goal_status": "invalid",
            "earliest_incomplete_phase": None,
            "verified_commits": [],
            "evidence_paths": [_display_path(repo_root, target_ledger)],
            "reason": message,
        }
        if (
            "CPFR-085 through CPFR-091" in message
            or "passed Phase 8 plan-review row" in message
        ):
            payload["status"] = "incomplete"
            payload["goal_status"] = "complete"
        return 1, payload
    payload = {
        "schema_version": "crouzeix-goal-validation/v1",
        "status": "complete" if result.goal_status == "complete" else "incomplete",
        "goal_status": result.goal_status,
        "earliest_incomplete_phase": result.earliest_incomplete_phase,
        "verified_commits": list(result.verified_commits),
        "evidence_paths": list(result.evidence_paths),
    }
    return (0 if payload["status"] == "complete" else 1), payload
