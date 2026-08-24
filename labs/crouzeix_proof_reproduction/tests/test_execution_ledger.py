from __future__ import annotations

import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
REPO = LAB.parents[1]

from labs.crouzeix_proof_reproduction import execution_ledger  # noqa: E402


HEADER = (
    "phase\tticket\tstate\tbranch\tworktree\tbase_commit\tcandidate_commit\t"
    "verifier\texit_code\tevidence_digest\tlanding_commit\tnote\n"
)
BASE_ROW = (
    "cpfr-085\tCPFR-085\timplementing\tfeat/cpfr-085-jin-certification\t"
    ".worktrees/cpfr-085-jin-certification\t"
    "f89cb21b6484956740c187dda48250be8f3e67c7\t"
    "8723d1c01bb6aae1e6eab133a3614c3056cdd002\t\t\t\t\trecovered-existing-worktree\n"
)


def write_ledger(root: Path, *rows: str) -> Path:
    path = root / "execution-ledger.tsv"
    path.write_text(HEADER + "".join(rows), encoding="utf-8")
    return path


class ExecutionLedgerTests(unittest.TestCase):
    def test_parses_recovered_cpfr085_row(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = write_ledger(Path(directory), BASE_ROW)

            ledger = execution_ledger.load_execution_ledger(path)

        self.assertEqual(ledger.path, path)
        self.assertEqual(len(ledger.rows), 1)
        row = ledger.rows[0]
        self.assertEqual(row.phase, "cpfr-085")
        self.assertEqual(row.ticket, "CPFR-085")
        self.assertEqual(row.state, "implementing")
        self.assertEqual(row.base_commit, "f89cb21b6484956740c187dda48250be8f3e67c7")
        self.assertEqual(
            row.candidate_commit,
            "8723d1c01bb6aae1e6eab133a3614c3056cdd002",
        )
        self.assertEqual(row.note, "recovered-existing-worktree")

    def test_rejects_unknown_state(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = write_ledger(Path(directory), BASE_ROW.replace("implementing", "started", 1))

            with self.assertRaisesRegex(ValueError, "unknown state"):
                execution_ledger.load_execution_ledger(path)

    def test_rejects_unknown_phase(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = write_ledger(Path(directory), BASE_ROW.replace("cpfr-085", "cpfr-999", 1))

            with self.assertRaisesRegex(ValueError, "unknown phase"):
                execution_ledger.load_execution_ledger(path)

    def test_rejects_duplicate_phase_transition(self) -> None:
        duplicate = (
            "cpfr-085\tCPFR-085\timplementing\tfeat/cpfr-085-jin-certification\t"
            ".worktrees/cpfr-085-jin-certification\t"
            "f89cb21b6484956740c187dda48250be8f3e67c7\t"
            "8723d1c01bb6aae1e6eab133a3614c3056cdd002\t"
            "\t\t\t\tworker-rerun\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            path = write_ledger(Path(directory), BASE_ROW, duplicate)

            with self.assertRaisesRegex(ValueError, "duplicate phase transition"):
                execution_ledger.load_execution_ledger(path)

    def test_accepts_strictly_increasing_phase_history_and_tracks_latest_state(self) -> None:
        second = (
            "cpfr-085\tCPFR-085\tlocally-verified\tfeat/cpfr-085-jin-certification\t"
            ".worktrees/cpfr-085-jin-certification\t"
            "f89cb21b6484956740c187dda48250be8f3e67c7\t"
            "8723d1c01bb6aae1e6eab133a3614c3056cdd002\t"
            "python3 -m unittest\t0\tsha256:" + ("a" * 64) + "\t\t\n"
        )
        third = (
            "cpfr-085\tCPFR-085\tspec-approved\tfeat/cpfr-085-jin-certification\t"
            ".worktrees/cpfr-085-jin-certification\t"
            "f89cb21b6484956740c187dda48250be8f3e67c7\t"
            "8723d1c01bb6aae1e6eab133a3614c3056cdd002\t"
            "spec review\t0\tsha256:" + ("b" * 64) + "\t\t\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            path = write_ledger(Path(directory), BASE_ROW, second, third)

            ledger = execution_ledger.load_execution_ledger(path)

        self.assertEqual(len(ledger.rows), 3)
        self.assertEqual(ledger.latest_state("cpfr-085"), "spec-approved")
        self.assertEqual(
            [row.state for row in ledger.rows if row.phase == "cpfr-085"],
            ["implementing", "locally-verified", "spec-approved"],
        )

    def test_rejects_same_state_as_duplicate(self) -> None:
        duplicate = (
            "cpfr-085\tCPFR-085\timplementing\tfeat/cpfr-085-jin-certification\t"
            ".worktrees/cpfr-085-jin-certification\t"
            "f89cb21b6484956740c187dda48250be8f3e67c7\t"
            "8723d1c01bb6aae1e6eab133a3614c3056cdd002\t\t\t\t\tworker-rerun\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            path = write_ledger(Path(directory), BASE_ROW, duplicate)

            with self.assertRaisesRegex(ValueError, "duplicate phase transition"):
                execution_ledger.load_execution_ledger(path)

    def test_rejects_non_monotone_state(self) -> None:
        later = (
            "cpfr-085\tCPFR-085\tlocally-verified\tfeat/cpfr-085-jin-certification\t"
            ".worktrees/cpfr-085-jin-certification\t"
            "f89cb21b6484956740c187dda48250be8f3e67c7\t"
            "8723d1c01bb6aae1e6eab133a3614c3056cdd002\t"
            "python3 -m unittest\t0\tsha256:" + ("a" * 64) + "\t\t\n"
        )
        earlier = (
            "cpfr-085\tCPFR-085\tspec-approved\tfeat/cpfr-085-jin-certification\t"
            ".worktrees/cpfr-085-jin-certification\t"
            "f89cb21b6484956740c187dda48250be8f3e67c7\t"
            "8723d1c01bb6aae1e6eab133a3614c3056cdd002\t"
            "spec review\t0\tsha256:" + ("b" * 64) + "\t\t\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            path = write_ledger(Path(directory), earlier, later)

            with self.assertRaisesRegex(ValueError, "non-monotone state"):
                execution_ledger.load_execution_ledger(path)

    def test_rejects_backward_transition_after_progress(self) -> None:
        second = (
            "cpfr-085\tCPFR-085\tlocally-verified\tfeat/cpfr-085-jin-certification\t"
            ".worktrees/cpfr-085-jin-certification\t"
            "f89cb21b6484956740c187dda48250be8f3e67c7\t"
            "8723d1c01bb6aae1e6eab133a3614c3056cdd002\t"
            "python3 -m unittest\t0\tsha256:" + ("a" * 64) + "\t\t\n"
        )
        backward = (
            "cpfr-085\tCPFR-085\timplementing\tfeat/cpfr-085-jin-certification\t"
            ".worktrees/cpfr-085-jin-certification\t"
            "f89cb21b6484956740c187dda48250be8f3e67c7\t"
            "8723d1c01bb6aae1e6eab133a3614c3056cdd002\t\t\t\t\tbackward\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            path = write_ledger(Path(directory), BASE_ROW, second, backward)

            with self.assertRaisesRegex(ValueError, "non-monotone state"):
                execution_ledger.load_execution_ledger(path)

    def test_rejects_unsafe_worktree_path(self) -> None:
        unsafe = BASE_ROW.replace(
            ".worktrees/cpfr-085-jin-certification",
            "../cpfr-085-jin-certification",
            1,
        )
        with tempfile.TemporaryDirectory() as directory:
            path = write_ledger(Path(directory), unsafe)

            with self.assertRaisesRegex(ValueError, "unsafe path"):
                execution_ledger.load_execution_ledger(path)

    def test_rejects_malformed_commit_id(self) -> None:
        bad_commit = BASE_ROW.replace(
            "8723d1c01bb6aae1e6eab133a3614c3056cdd002",
            "not-a-commit",
            1,
        )
        with tempfile.TemporaryDirectory() as directory:
            path = write_ledger(Path(directory), bad_commit)

            with self.assertRaisesRegex(ValueError, "commit"):
                execution_ledger.load_execution_ledger(path)
