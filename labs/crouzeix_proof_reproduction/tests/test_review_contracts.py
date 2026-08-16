from __future__ import annotations

import hashlib
import json
import sys
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import protocol
import review_contracts


def digest_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def digest_json(value: object) -> str:
    return hashlib.sha256(
        json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
    ).hexdigest()


def candidate_projection(**overrides: object) -> dict[str, object]:
    candidate = "Candidate proof bytes.\n"
    value: dict[str, object] = {
        "schema_version": "crouzeix-candidate-projection/v1",
        "projection_id": "candidate-alpha",
        "source_node_id": "node-g2-d1",
        "source_node_artifact_sha256": "a" * 64,
        "source_reconciliation_sha256": "b" * 64,
        "candidate_sha256": digest_text(candidate),
        "candidate_byte_count": len(candidate.encode("utf-8")),
        "candidate_projection_sha256": "c" * 64,
        "arm": "expert_frontier",
        "model": "hidden-model",
        "cost_usd": "9.99",
        "lineage": ["node-g0", "node-g2-d1"],
        "mechanism_label": "jin-like",
        "prior_findings": [{"finding_id": "leak"}],
        "reviewer_output": {"outcome": "complete"},
    }
    value.update(overrides)
    return value


def finding(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "finding_id": "finding-gap-main",
        "severity": "critical",
        "locator": "candidate.tex#L12-L18",
        "statement": "The proof assumes the theorem-strength conclusion.",
        "falsifying_test_or_gap": "Provide a bound independent of the target theorem.",
    }
    value.update(overrides)
    return value


def review_payload(**overrides: object) -> dict[str, object]:
    candidate = "Candidate proof bytes.\n"
    value: dict[str, object] = {
        "schema_version": "crouzeix-correctness-review-payload/v1",
        "anonymous_candidate_id": "anon-candidate-alpha",
        "candidate_sha256": digest_text(candidate),
        "reviewer_index": 1,
        "outcome": "incomplete",
        "findings": [finding()],
        "theorem_strength_obligations": [
            {
                "obligation_id": "obl-main-gap",
                "locator": "candidate.tex#L12",
                "statement": "Close the theorem-strength gap.",
            }
        ],
    }
    value.update(overrides)
    return value


class CorrectnessContractTests(unittest.TestCase):
    def test_correctness_context_strips_treatment_lineage_and_prior_review_state(self) -> None:
        context = review_contracts.build_correctness_context(
            theorem_text="For every square matrix A, prove the Crouzeix bound.",
            candidate_text="Candidate proof bytes.\n",
            candidate_projection=candidate_projection(),
            reviewer_index=1,
            ticket_id="review-candidate-alpha-r1",
        )

        self.assertEqual(context["schema_version"], "crouzeix-correctness-context/v1")
        self.assertEqual(context["anonymous_candidate_id"], "anon-candidate-alpha")
        self.assertEqual(context["candidate_text"], "Candidate proof bytes.\n")
        self.assertEqual(context["candidate_sha256"], digest_text("Candidate proof bytes.\n"))
        self.assertEqual(context["allowed_tools"], ["Write"])
        self.assertFalse(context["delegation_allowed"])
        serialized = json.dumps(context, sort_keys=True)
        for forbidden in (
            "expert_frontier",
            "hidden-model",
            "cost_usd",
            "source_node",
            "source_reconciliation",
            "lineage",
            "mechanism_label",
            "prior_findings",
            "reviewer_output",
        ):
            self.assertNotIn(forbidden, serialized)

    def test_correctness_payload_requires_closed_outcome_and_exact_finding_shape(self) -> None:
        value = review_contracts.validate_correctness_review_payload(review_payload())

        self.assertEqual(value["outcome"], "incomplete")
        self.assertEqual(value["findings"][0]["severity"], "critical")
        self.assertEqual(value["findings"][0]["locator"], "candidate.tex#L12-L18")
        self.assertIn("correctness_review_payload_sha256", value)

        with self.assertRaisesRegex(protocol.ValidationError, "outcome"):
            review_contracts.validate_correctness_review_payload(
                review_payload(outcome="mostly_complete")
            )
        with self.assertRaisesRegex(protocol.ValidationError, "fields"):
            review_contracts.validate_correctness_review_payload(
                review_payload(findings=[finding(commentary="extra")])
            )
        with self.assertRaisesRegex(protocol.ValidationError, "locator"):
            review_contracts.validate_correctness_review_payload(
                review_payload(findings=[finding(locator="")])
            )


class ReconciliationAndRepairContractTests(unittest.TestCase):
    def test_reconciliation_requires_one_disposition_for_every_namespaced_finding(self) -> None:
        first = review_contracts.validate_correctness_review_payload(review_payload(reviewer_index=1))
        second = review_contracts.validate_correctness_review_payload(
            review_payload(
                reviewer_index=2,
                findings=[finding(finding_id="finding-gap-secondary", severity="major")],
            )
        )

        ledger = review_contracts.reconcile_correctness_reviews(
            reconciliation_id="reconcile-candidate-alpha",
            candidate_sha256=digest_text("Candidate proof bytes.\n"),
            reviews=[first, second],
            dispositions=[
                {
                    "namespaced_finding_id": "r1:finding-gap-main",
                    "disposition": "unresolved",
                    "rationale": "The gap remains material.",
                },
                {
                    "namespaced_finding_id": "r2:finding-gap-secondary",
                    "disposition": "rejected_with_reason",
                    "rationale": "The locator points to an already proved lemma.",
                },
            ],
        )

        self.assertEqual(ledger["outcome"], "incomplete")
        self.assertEqual(
            [item["namespaced_finding_id"] for item in ledger["finding_dispositions"]],
            ["r1:finding-gap-main", "r2:finding-gap-secondary"],
        )
        self.assertIn("finding_ledger_sha256", ledger)

        with self.assertRaisesRegex(protocol.ValidationError, "exactly one disposition"):
            review_contracts.reconcile_correctness_reviews(
                reconciliation_id="reconcile-candidate-alpha",
                candidate_sha256=digest_text("Candidate proof bytes.\n"),
                reviews=[first, second],
                dispositions=[
                    {
                        "namespaced_finding_id": "r1:finding-gap-main",
                        "disposition": "unresolved",
                        "rationale": "Only one reviewer disposition supplied.",
                    }
                ],
            )

    def test_repair_request_binds_parent_candidate_and_finding_ledger_digests(self) -> None:
        ledger = {
            "schema_version": "crouzeix-correctness-reconciliation/v1",
            "reconciliation_id": "reconcile-candidate-alpha",
            "candidate_sha256": digest_text("Candidate proof bytes.\n"),
            "outcome": "incomplete",
            "finding_dispositions": [
                {
                    "namespaced_finding_id": "r1:finding-gap-main",
                    "disposition": "unresolved",
                    "severity": "critical",
                    "locator": "candidate.tex#L12",
                    "statement": "Gap.",
                    "falsifying_test_or_gap": "Test.",
                    "rationale": "Still open.",
                }
            ],
        }
        ledger["finding_ledger_sha256"] = digest_json(ledger)

        request = review_contracts.build_repair_request(
            repair_id="repair-candidate-alpha",
            parent_candidate_text="Candidate proof bytes.\n",
            finding_ledger=ledger,
            ticket_id="repair-candidate-alpha",
        )

        self.assertEqual(request["parent_candidate_sha256"], digest_text("Candidate proof bytes.\n"))
        self.assertEqual(request["finding_ledger_sha256"], ledger["finding_ledger_sha256"])
        self.assertEqual(request["max_provider_calls"], 1)

        with self.assertRaisesRegex(protocol.ValidationError, "parent candidate"):
            review_contracts.build_repair_request(
                repair_id="repair-candidate-alpha",
                parent_candidate_text="changed\n",
                finding_ledger=ledger,
                ticket_id="repair-candidate-alpha",
            )


class ClassificationContractTests(unittest.TestCase):
    def test_mechanism_classification_binds_frozen_correctness_and_preserves_bytes(self) -> None:
        frozen = {
            "correctness_reconciliation_sha256": "d" * 64,
            "candidate_sha256": digest_text("Candidate proof bytes.\n"),
            "correctness_bytes_sha256": "e" * 64,
        }
        context = review_contracts.build_mechanism_classification_context(
            classification_id="classify-candidate-alpha",
            theorem_text="For every square matrix A, prove the Crouzeix bound.",
            candidate_text="Candidate proof bytes.\n",
            frozen_correctness=frozen,
            reference_cards=[
                {
                    "reference_id": "jin",
                    "mechanism_summary": "Known public mechanism summary.",
                    "reference_sha256": "f" * 64,
                }
            ],
            ticket_id="classify-candidate-alpha",
        )

        self.assertEqual(context["frozen_correctness_sha256"], "d" * 64)
        self.assertEqual(context["candidate_sha256"], digest_text("Candidate proof bytes.\n"))
        self.assertNotIn("correctness_outcome", context)

        payload = review_contracts.validate_mechanism_classification_payload(
            {
                "schema_version": "crouzeix-mechanism-classification-payload/v1",
                "classification_id": "classify-candidate-alpha",
                "candidate_sha256": digest_text("Candidate proof bytes.\n"),
                "frozen_correctness_sha256": "d" * 64,
                "similarities": [
                    {
                        "reference_id": "jin",
                        "similarity": "related",
                        "locator": "candidate.tex#L3",
                        "rationale": "Both use a comparable normalization.",
                    }
                ],
                "candidate_rewrite": None,
                "correctness_outcome": None,
            }
        )

        self.assertEqual(payload["similarities"][0]["similarity"], "related")
        with self.assertRaisesRegex(protocol.ValidationError, "cannot rewrite"):
            review_contracts.validate_mechanism_classification_payload(
                {
                    **payload,
                    "candidate_rewrite": "mutated proof",
                    "mechanism_classification_payload_sha256": payload[
                        "mechanism_classification_payload_sha256"
                    ],
                }
            )
        with self.assertRaisesRegex(protocol.ValidationError, "correctness"):
            review_contracts.validate_mechanism_classification_payload(
                {
                    **payload,
                    "correctness_outcome": "complete",
                    "mechanism_classification_payload_sha256": payload[
                        "mechanism_classification_payload_sha256"
                    ],
                }
            )


if __name__ == "__main__":
    unittest.main()
