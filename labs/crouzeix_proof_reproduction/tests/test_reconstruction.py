from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import formal_receipt
import protocol
import reconstruction


def obligation(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-reconstruction-obligation/v1",
        "obligation_id": "recon-jin-gap",
        "statement_sha256": "a" * 64,
        "source_locator": "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/CrouzeixConjecture/FinalTheorems.lean#L18-L23",
        "imports": ["CrouzeixConjecture.FinalTheorems"],
        "local_context": ["A : SquareMatrix n", "p : Polynomial C"],
        "proof_slots": ["main"],
        "attempt_budget": 1,
        "decomposition_budget": 1,
    }
    value.update(overrides)
    return value


def passed_receipt(*, candidate_sha256: str = "c" * 64) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-formal-attempt-receipt/v2",
        "attempt_id": "recon-attempt",
        "run_id": "reconstruction",
        "ticket_id": "recon-ticket",
        "ticket_sha256": "1" * 64,
        "candidate": {
            "outcome": "candidate",
            "candidate_id": "candidate",
            "candidate_sha256": candidate_sha256,
            "candidate_bytes": 1,
        },
        "toolchain": {
            "name": "lean",
            "version": "4.28.0",
            "platform": "fixture",
            "toolchain_sha256": "2" * 64,
        },
        "source": {
            "path": "source/Target.lean",
            "sha256": "3" * 64,
            "bytes": 1,
        },
        "command": {
            "argv": ["lake", "build"],
            "cwd": ".",
            "env_sha256": "4" * 64,
            "exit_code": 0,
        },
        "logs": {
            "stdout_path": "logs/stdout.txt",
            "stdout_sha256": "5" * 64,
            "stderr_path": "logs/stderr.txt",
            "stderr_sha256": "6" * 64,
            "complete": True,
        },
        "resource_receipt": {
            "path": "resource_receipt.json",
            "sha256": "7" * 64,
        },
        "axioms": {
            "scan_performed": True,
            "scanner": "fixture",
            "allowed_axioms": [],
            "observed_axioms": [],
            "scan_log_path": "logs/axioms.txt",
            "scan_log_sha256": "d" * 64,
        },
        "status": "passed",
        "reason": "fixture compile passed",
        "started_at_utc": "2026-08-15T12:00:01Z",
        "completed_at_utc": "2026-08-15T12:00:02Z",
        "formal_target": {
            "path": "formal_target.lock.json",
            "sha256": "8" * 64,
        },
        "runtime_inventory_sha256": "9" * 64,
        "target_type_sha256": "a" * 64,
    }
    value["formal_attempt_sha256"] = formal_receipt.canonical_sha256_without_self(value)
    return value


class ReconstructionTests(unittest.TestCase):
    def test_materialize_candidate_allows_only_declared_proof_slots(self) -> None:
        sealed = reconstruction.SealedObligation.from_mapping(obligation())

        rendered = reconstruction.materialize_candidate(sealed, {"main": "exact h"})

        self.assertIn("exact h", rendered)
        with self.assertRaisesRegex(protocol.ValidationError, "slot"):
            reconstruction.materialize_candidate(sealed, {"other": "exact h"})
        with self.assertRaisesRegex(protocol.ValidationError, "import"):
            reconstruction.materialize_candidate(sealed, {"main": "import Mathlib\nexact h"})

    def test_publish_requires_compiled_candidate_backed_by_passed_v2_receipt(self) -> None:
        sealed = reconstruction.SealedObligation.from_mapping(obligation())
        history = reconstruction.ReconstructionHistory(sealed)

        history.record({"classification": "syntax", "candidate_sha256": "b" * 64})
        with self.assertRaisesRegex(protocol.ValidationError, "compiled"):
            history.publish_if_compiled({"classification": "syntax", "candidate_sha256": "b" * 64})
        with self.assertRaisesRegex(protocol.ValidationError, "formal receipt"):
            history.publish_if_compiled(
                {
                    "classification": "compiled",
                    "candidate_sha256": "c" * 64,
                    "axiom_audit_sha256": "d" * 64,
                }
            )

        published = history.publish_if_compiled(
            {
                "classification": "compiled",
                "candidate_sha256": "c" * 64,
                "axiom_audit_sha256": "d" * 64,
                "formal_receipt": passed_receipt(),
            }
        )
        self.assertEqual(published["status"], "locally_proved")
        self.assertEqual(published["statement_sha256"], "a" * 64)

        failed = passed_receipt()
        failed["status"] = "failed"
        failed["command"] = dict(failed["command"], exit_code=1)
        failed["formal_attempt_sha256"] = formal_receipt.canonical_sha256_without_self(failed)
        with self.assertRaisesRegex(protocol.ValidationError, "passed"):
            history.publish_if_compiled(
                {
                    "classification": "compiled",
                    "candidate_sha256": "c" * 64,
                    "axiom_audit_sha256": "d" * 64,
                    "formal_receipt": failed,
                }
            )

    def test_reconstruction_history_retains_failed_candidates_and_budget(self) -> None:
        sealed = reconstruction.SealedObligation.from_mapping(obligation(attempt_budget=1))
        history = reconstruction.ReconstructionHistory(sealed)
        history.record({"classification": "missing_fact", "candidate_sha256": "e" * 64})

        with self.assertRaisesRegex(protocol.ValidationError, "budget"):
            history.record({"classification": "type_mismatch", "candidate_sha256": "f" * 64})
        self.assertEqual(len(history.events), 1)


if __name__ == "__main__":
    unittest.main()
