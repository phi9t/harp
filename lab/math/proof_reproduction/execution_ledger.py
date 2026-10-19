from __future__ import annotations

import re
from dataclasses import dataclass
from pathlib import PurePosixPath, Path


HEADER = (
    "phase\tticket\tstate\tbranch\tworktree\tbase_commit\tcandidate_commit\t"
    "verifier\texit_code\tevidence_digest\tlanding_commit\tnote"
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
STATE_ORDER = (
    "implementing",
    "locally-verified",
    "spec-approved",
    "evidence-published",
    "standards-approved",
    "release-verified",
    "landed",
)
STATE_INDEX = {state: index for index, state in enumerate(STATE_ORDER)}
GIT_ID_RE = re.compile(r"[0-9a-f]{40,64}\Z")
EVIDENCE_DIGEST_RE = re.compile(r"sha256:[0-9a-f]{64}\Z")


@dataclass(frozen=True)
class ExecutionLedgerRow:
    phase: str
    ticket: str
    state: str
    branch: str
    worktree: str
    base_commit: str
    candidate_commit: str
    verifier: str
    exit_code: int | None
    evidence_digest: str
    landing_commit: str
    note: str


@dataclass(frozen=True)
class ExecutionLedger:
    path: Path
    rows: tuple[ExecutionLedgerRow, ...]

    def latest_state(self, phase: str) -> str | None:
        for row in reversed(self.rows):
            if row.phase == phase:
                return row.state
        return None

    def has_landed_phase(self, phase: str) -> bool:
        return self.latest_state(phase) == "landed"


def _decode_field(value: str) -> str:
    parts: list[str] = []
    index = 0
    while index < len(value):
        char = value[index]
        if char != "%":
            parts.append(char)
            index += 1
            continue
        if index + 2 >= len(value):
            raise ValueError("malformed percent-encoding")
        try:
            parts.append(chr(int(value[index + 1 : index + 3], 16)))
        except ValueError as error:
            raise ValueError("malformed percent-encoding") from error
        index += 3
    return "".join(parts)


def _safe_relative_path(value: str) -> str:
    path = PurePosixPath(value)
    if not value or path.is_absolute() or "\\" in value:
        raise ValueError("unsafe path")
    if any(part in {"", ".", ".."} for part in path.parts):
        raise ValueError("unsafe path")
    return path.as_posix()


def _parse_commit(value: str, *, label: str, required: bool) -> str:
    if not value:
        if required:
            raise ValueError(f"{label} commit is required")
        return ""
    if not GIT_ID_RE.fullmatch(value):
        raise ValueError(f"{label} commit is invalid")
    return value


def _parse_exit_code(value: str) -> int | None:
    if not value:
        return None
    try:
        return int(value)
    except ValueError as error:
        raise ValueError("exit code is invalid") from error


def _parse_row(line: str) -> ExecutionLedgerRow:
    columns = [_decode_field(item) for item in line.split("\t")]
    if len(columns) != 12:
        raise ValueError("execution ledger row has wrong field count")
    (
        phase,
        ticket,
        state,
        branch,
        worktree,
        base_commit,
        candidate_commit,
        verifier,
        exit_code,
        evidence_digest,
        landing_commit,
        note,
    ) = columns
    if phase not in PHASE_ORDER:
        raise ValueError("unknown phase")
    if state not in STATE_INDEX:
        raise ValueError("unknown state")
    if evidence_digest and not EVIDENCE_DIGEST_RE.fullmatch(evidence_digest):
        raise ValueError("evidence digest is invalid")
    return ExecutionLedgerRow(
        phase=phase,
        ticket=ticket,
        state=state,
        branch=branch,
        worktree=_safe_relative_path(worktree),
        base_commit=_parse_commit(base_commit, label="base", required=True),
        candidate_commit=_parse_commit(
            candidate_commit,
            label="candidate",
            required=True,
        ),
        verifier=verifier,
        exit_code=_parse_exit_code(exit_code),
        evidence_digest=evidence_digest,
        landing_commit=_parse_commit(landing_commit, label="landing", required=False),
        note=note,
    )


def load_execution_ledger(path: Path) -> ExecutionLedger:
    source = Path(path).read_text(encoding="utf-8")
    lines = source.splitlines()
    if not lines or lines[0] != HEADER:
        raise ValueError("execution ledger header is invalid")
    rows: list[ExecutionLedgerRow] = []
    latest_phase_state: dict[str, int] = {}
    for raw_line in lines[1:]:
        if not raw_line:
            continue
        row = _parse_row(raw_line)
        current_state = STATE_INDEX[row.state]
        previous_state = latest_phase_state.get(row.phase)
        if previous_state is not None:
            if current_state == previous_state:
                raise ValueError("duplicate phase transition")
            if current_state < previous_state:
                raise ValueError("non-monotone state")
        latest_phase_state[row.phase] = current_state
        rows.append(row)
    return ExecutionLedger(Path(path), tuple(rows))
