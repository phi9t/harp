from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

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


class ReconstructionTests(unittest.TestCase):
    def test_materialize_candidate_allows_only_declared_proof_slots(self) -> None:
        sealed = reconstruction.SealedObligation.from_mapping(obligation())

        rendered = reconstruction.materialize_candidate(sealed, {"main": "exact h"})

        self.assertIn("exact h", rendered)
        with self.assertRaisesRegex(protocol.ValidationError, "slot"):
            reconstruction.materialize_candidate(sealed, {"other": "exact h"})
        with self.assertRaisesRegex(protocol.ValidationError, "import"):
            reconstruction.materialize_candidate(sealed, {"main": "import Mathlib\nexact h"})

    def test_classify_results_and_publish_only_compiled_audited_candidate(self) -> None:
        sealed = reconstruction.SealedObligation.from_mapping(obligation())
        history = reconstruction.ReconstructionHistory(sealed)

        history.record({"classification": "syntax", "candidate_sha256": "b" * 64})
        with self.assertRaisesRegex(protocol.ValidationError, "compiled"):
            history.publish_if_compiled({"classification": "syntax", "candidate_sha256": "b" * 64})

        published = history.publish_if_compiled(
            {
                "classification": "compiled",
                "candidate_sha256": "c" * 64,
                "axiom_audit_sha256": "d" * 64,
            }
        )
        self.assertEqual(published["status"], "locally_proved")
        self.assertEqual(published["statement_sha256"], "a" * 64)

    def test_reconstruction_history_retains_failed_candidates_and_budget(self) -> None:
        sealed = reconstruction.SealedObligation.from_mapping(obligation(attempt_budget=1))
        history = reconstruction.ReconstructionHistory(sealed)
        history.record({"classification": "missing_fact", "candidate_sha256": "e" * 64})

        with self.assertRaisesRegex(protocol.ValidationError, "budget"):
            history.record({"classification": "type_mismatch", "candidate_sha256": "f" * 64})
        self.assertEqual(len(history.events), 1)


if __name__ == "__main__":
    unittest.main()
