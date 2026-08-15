from __future__ import annotations

import hashlib
import json
import sys
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import expert_contracts
import protocol


def digest_json(value: object) -> str:
    data = json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    ).encode("utf-8")
    return hashlib.sha256(data).hexdigest()


def selected_direction(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "direction_id": "root-function-theory",
        "kind": "root_task",
        "statement": "Develop an independent route to the theorem.",
        "strength": "root",
        "recommended_role": "function_theory",
        "source_node_artifact_sha256": None,
        "source_reconciliation_sha256": None,
    }
    value.update(overrides)
    return value


def payload(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "proof_family": "functional calculus route",
        "mechanism": "Reduce the bound to a local convexity estimate.",
        "proved_statements": [
            {
                "statement_id": "stmt-local-bound",
                "statement": "A local bound follows from the normalized estimate.",
                "justification": "Direct estimate in the proposed normalization.",
                "depends_on_statement_ids": [],
            }
        ],
        "unproved_obligations": [
            {
                "obligation_id": "obl-global-extension",
                "statement": "Extend the local estimate globally.",
                "strength": "major",
            }
        ],
        "circularity_risks": [
            {
                "risk_id": "risk-hidden-theorem",
                "statement": "The estimate must not assume the target theorem.",
                "locator": "mechanism paragraph 2",
            }
        ],
        "proposed_directions": [
            {
                "direction_id": "dir-check-extension",
                "statement": "Audit the global extension step.",
                "strength": "major",
                "recommended_role": "approximation_audit",
            }
        ],
        "endpoint": {
            "kind": "candidate_proof",
            "text": "Candidate proof text with explicit remaining checks.",
        },
        "confidence_basis": "One proved statement and one explicit obligation.",
    }
    value.update(overrides)
    return value


def nonfunctioning_payload(**overrides: object) -> dict[str, object]:
    value = payload(
        mechanism="No concrete mechanism supplied.",
        proved_statements=[],
        proposed_directions=[],
        endpoint={"kind": "candidate_proof", "text": "No concrete proof content."},
    )
    value.update(overrides)
    return value


def expert_result(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "schema_version": "crouzeix-expert-result/v1",
        "run_id": "expert-frontier-001",
        "attempt_id": "attempt-g0-function-theory",
        "ticket_id": "expert-g0-function-theory",
        "proposed_node_id": "node-g0-function-theory",
        "parent_node_id": None,
        "parent_node_artifact_sha256": None,
        "generation": 0,
        "expert_role": "function_theory",
        "selected_direction_id": "root-function-theory",
        "mathematical_payload": payload(),
    }
    value.update(overrides)
    return value


def attempt(status: str = "completed", **overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "run_id": "expert-frontier-001",
        "attempt_id": "attempt-g0-function-theory",
        "ticket_id": "expert-g0-function-theory",
        "proposed_node_id": "node-g0-function-theory",
        "terminal_status": status,
    }
    value.update(overrides)
    return value


def provenance(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "theorem_sha256": "a" * 64,
        "leakage_tier": "L1",
        "context_sha256": "b" * 64,
        "call_receipt_sha256": "c" * 64,
        "leakage_audit_sha256": "d" * 64,
        "selected_direction": selected_direction(),
    }
    value.update(overrides)
    return value


def evaluation_payload(status: str = "pass", *, candidate_digest: str | None = None) -> dict[str, object]:
    probes = [
        {"probe_id": probe_id, "status": status, "rationale": f"{probe_id} rationale"}
        for probe_id in expert_contracts.PROOF_PROGRESS_PROBE_IDS
    ]
    if candidate_digest is not None:
        probes[-1]["candidate_proof_sha256"] = candidate_digest
    return {
        "schema_version": "crouzeix-node-evaluation-payload/v1",
        "probes": probes,
        "findings": [
            {
                "finding_id": "finding-e1-critical-gap",
                "probe_id": expert_contracts.PROOF_PROGRESS_PROBE_IDS[0],
                "severity": "critical",
                "locator": "payload.proved_statements[0]",
                "statement": "The local estimate is not justified.",
                "test": "Supply the missing inequality.",
            }
        ],
    }


class ExpertContextTests(unittest.TestCase):
    def test_root_and_child_contexts_expose_only_declared_expert_inputs(self) -> None:
        theorem = "For every square matrix A, prove the required norm bound."

        root = expert_contracts.build_expert_context(
            run_id="expert-frontier-001",
            attempt_id="attempt-g0-function-theory",
            ticket_id="expert-g0-function-theory",
            proposed_node_id="node-g0-function-theory",
            parent_node=None,
            generation=0,
            expert_role="function_theory",
            selected_direction=selected_direction(),
            theorem_text=theorem,
            forbidden_sources=["public proof manuscripts"],
        )

        self.assertEqual(root["parent"], None)
        self.assertEqual(root["theorem_text"], theorem)
        self.assertEqual(root["theorem_sha256"], hashlib.sha256(theorem.encode("utf-8")).hexdigest())
        self.assertEqual(root["allowed_tools"], ["Write"])
        self.assertFalse(root["delegation_allowed"])
        self.assertNotIn("peer_artifacts", root)
        self.assertEqual(
            root["limits"],
            {"timeout_seconds": 3600, "max_output_bytes": 1048576},
        )
        self.assertIn("functioning_criteria", root)
        self.assertIn("completion_criteria", root)

        result = expert_contracts.validate_expert_result(expert_result())
        decision = expert_contracts.decide_admission(
            attempt(),
            result,
            provenance(),
            outcome="accepted",
        )
        node = expert_contracts.build_mathematical_node(decision, result, provenance())

        child_direction = selected_direction(
            direction_id="child-obligation",
            kind="obligation",
            statement="Close the global extension obligation.",
            strength="major",
            recommended_role="approximation_audit",
            source_node_artifact_sha256=node["node_artifact_sha256"],
            source_reconciliation_sha256=None,
        )
        child = expert_contracts.build_expert_context(
            run_id="expert-frontier-001",
            attempt_id="attempt-g1-d0",
            ticket_id="expert-g1-d0",
            proposed_node_id="node-g1-d0",
            parent_node=node,
            generation=1,
            expert_role="approximation_audit",
            selected_direction=child_direction,
            theorem_text=theorem,
            forbidden_sources=["public proof manuscripts"],
        )

        self.assertEqual(set(child["parent"]), {"node_id", "node_artifact_sha256", "mathematical_payload"})
        self.assertEqual(child["parent"]["node_id"], node["node_id"])
        self.assertEqual(child["parent"]["node_artifact_sha256"], node["node_artifact_sha256"])
        self.assertNotIn("source_attempt_id", child["parent"])
        self.assertNotIn("admission_decision_sha256", child["parent"])

    def test_evaluator_context_exposes_theorem_and_payload_only(self) -> None:
        result = expert_contracts.validate_expert_result(expert_result())
        decision = expert_contracts.decide_admission(attempt(), result, provenance(), outcome="accepted")
        node = expert_contracts.build_mathematical_node(decision, result, provenance())

        context = expert_contracts.build_evaluator_context(
            theorem_text="For every square matrix A, prove the required norm bound.",
            node=node,
        )

        self.assertEqual(
            set(context),
            {
                "schema_version",
                "theorem_text",
                "mathematical_payload",
                "probe_ids",
            },
        )
        for forbidden in (
            "run_id",
            "node_id",
            "node_artifact_sha256",
            "ticket_id",
            "evaluator_index",
            "allowed_tools",
            "delegation_allowed",
            "parent_node_id",
            "generation",
            "expert_role",
            "selected_direction",
            "score",
            "treatment",
        ):
            self.assertNotIn(forbidden, context)


class ExpertResultAdmissionAndNodeTests(unittest.TestCase):
    def test_expert_result_is_not_a_node_and_rejects_nullable_endpoint_siblings(self) -> None:
        result = expert_contracts.validate_expert_result(expert_result())

        self.assertIn("expert_result_sha256", result)
        self.assertNotIn("node_artifact_sha256", result)
        self.assertEqual(result["mathematical_payload"]["endpoint"]["kind"], "candidate_proof")

        malformed_payload = payload(endpoint={"kind": "candidate_proof", "text": "proof"}, blocker=None)
        with self.assertRaisesRegex(protocol.ValidationError, "fields"):
            expert_contracts.validate_expert_result(
                expert_result(mathematical_payload=malformed_payload)
            )

        blocker = expert_contracts.validate_expert_result(
            expert_result(
                mathematical_payload=payload(
                    endpoint={"kind": "blocker", "text": "A precise obstruction."}
                )
            )
        )
        self.assertEqual(blocker["mathematical_payload"]["endpoint"]["kind"], "blocker")

    def test_only_strict_results_receive_admission_and_only_acceptance_creates_node(self) -> None:
        result = expert_contracts.validate_expert_result(expert_result())

        rejected = expert_contracts.decide_admission(
            attempt(),
            result,
            provenance(),
            outcome="rejected_nonfunctioning",
            diagnostic_detail="No materially new mechanism.",
        )

        self.assertEqual(rejected["outcome"], "rejected_nonfunctioning")
        self.assertNotIn("node_artifact_sha256", rejected)
        with self.assertRaisesRegex(protocol.ValidationError, "accepted"):
            expert_contracts.build_mathematical_node(rejected, result, provenance())

        for terminal_status in ("failed", "malformed", "timed_out", "blocked_resource"):
            with self.assertRaisesRegex(protocol.ValidationError, "terminal"):
                expert_contracts.decide_admission(
                    attempt(terminal_status),
                    result,
                    provenance(),
                    outcome="accepted",
                )

    def test_admission_derives_nonfunctioning_candidate_and_blocker_rejections(self) -> None:
        empty_candidate = expert_contracts.validate_expert_result(
            expert_result(mathematical_payload=nonfunctioning_payload())
        )

        rejected_candidate = expert_contracts.decide_admission(
            attempt(),
            empty_candidate,
            provenance(),
            outcome="accepted",
        )

        self.assertEqual(rejected_candidate["outcome"], "rejected_nonfunctioning")
        self.assertEqual(rejected_candidate["reason_code"], "nonfunctioning_result")
        with self.assertRaisesRegex(protocol.ValidationError, "accepted"):
            expert_contracts.build_mathematical_node(
                rejected_candidate,
                empty_candidate,
                provenance(),
            )

        empty_blocker = expert_contracts.validate_expert_result(
            expert_result(
                mathematical_payload=nonfunctioning_payload(
                    mechanism="No precise blocker supplied.",
                    endpoint={"kind": "blocker", "text": "Blocked."},
                    proposed_directions=[],
                )
            )
        )
        rejected_blocker = expert_contracts.decide_admission(
            attempt(),
            empty_blocker,
            provenance(),
            outcome="accepted",
        )

        self.assertEqual(rejected_blocker["outcome"], "rejected_nonfunctioning")

    def test_mathematical_node_has_acyclic_admission_and_stable_digest_boundaries(self) -> None:
        result = expert_contracts.validate_expert_result(expert_result())
        decision = expert_contracts.decide_admission(attempt(), result, provenance(), outcome="accepted")
        node = expert_contracts.build_mathematical_node(decision, result, provenance())

        self.assertEqual(decision["proposed_node_id"], node["node_id"])
        self.assertNotIn("node_artifact_sha256", decision)
        self.assertEqual(
            node["mathematical_payload_sha256"],
            digest_json(node["mathematical_payload"]),
        )
        without_artifact_digest = dict(node)
        without_artifact_digest.pop("node_artifact_sha256")
        self.assertEqual(node["node_artifact_sha256"], digest_json(without_artifact_digest))

        for forbidden in (
            "evaluation",
            "score",
            "child_count",
            "selection",
            "review",
            "timestamp",
            "path",
        ):
            self.assertNotIn(forbidden, json.dumps(node, sort_keys=True))

    def test_node_payload_requires_unique_acyclic_stable_ids(self) -> None:
        duplicated = payload(
            proved_statements=[
                {
                    "statement_id": "stmt-a",
                    "statement": "A",
                    "justification": "because A",
                    "depends_on_statement_ids": [],
                },
                {
                    "statement_id": "stmt-a",
                    "statement": "B",
                    "justification": "because B",
                    "depends_on_statement_ids": [],
                },
            ]
        )
        with self.assertRaisesRegex(protocol.ValidationError, "duplicate"):
            expert_contracts.validate_expert_result(expert_result(mathematical_payload=duplicated))

        cyclic = payload(
            proved_statements=[
                {
                    "statement_id": "stmt-a",
                    "statement": "A",
                    "justification": "because B",
                    "depends_on_statement_ids": ["stmt-b"],
                },
                {
                    "statement_id": "stmt-b",
                    "statement": "B",
                    "justification": "because A",
                    "depends_on_statement_ids": ["stmt-a"],
                },
            ]
        )
        with self.assertRaisesRegex(protocol.ValidationError, "acyclic"):
            expert_contracts.validate_expert_result(expert_result(mathematical_payload=cyclic))

    def test_parent_identity_is_pairwise_null_or_pairwise_present(self) -> None:
        with self.assertRaisesRegex(protocol.ValidationError, "parent"):
            expert_contracts.validate_expert_result(expert_result(parent_node_id="node-parent"))

        child_result = expert_contracts.validate_expert_result(
            expert_result(
                parent_node_id="node-parent",
                parent_node_artifact_sha256="f" * 64,
                generation=1,
                selected_direction_id="child-obligation",
            )
        )
        child_provenance = provenance(
            selected_direction=selected_direction(
                direction_id="child-obligation",
                kind="obligation",
                strength="major",
                recommended_role="approximation_audit",
                source_node_artifact_sha256="f" * 64,
            )
        )
        decision = expert_contracts.decide_admission(attempt(), child_result, child_provenance, outcome="accepted")
        node = expert_contracts.build_mathematical_node(decision, child_result, child_provenance)
        self.assertEqual(node["parent_node_id"], "node-parent")
        self.assertEqual(node["parent_node_artifact_sha256"], "f" * 64)

        mismatch = provenance(
            selected_direction=selected_direction(
                direction_id="child-obligation",
                kind="obligation",
                strength="major",
                recommended_role="approximation_audit",
                source_node_artifact_sha256="e" * 64,
            )
        )
        with self.assertRaisesRegex(protocol.ValidationError, "parent artifact"):
            expert_contracts.decide_admission(
                attempt(),
                child_result,
                mismatch,
                outcome="accepted",
            )


class NodeEvaluationTests(unittest.TestCase):
    def test_evaluator_payload_and_harness_envelope_are_separate(self) -> None:
        result = expert_contracts.validate_expert_result(expert_result())
        decision = expert_contracts.decide_admission(attempt(), result, provenance(), outcome="accepted")
        node = expert_contracts.build_mathematical_node(decision, result, provenance())
        evaluation = evaluation_payload()
        context = expert_contracts.build_evaluator_context(
            theorem_text="For every square matrix A, prove the required norm bound.",
            node=node,
        )
        context_sha256 = digest_json(context)

        envelope = expert_contracts.build_node_evaluation(
            evaluation_id="eval-node-g0-function-theory-e1",
            evaluator_index=1,
            ticket_id="evaluate-node-g0-function-theory-e1",
            node=node,
            context_sha256=context_sha256,
            source_kind="provider_output",
            call_receipt_sha256="1" * 64,
            terminal_ticket_event_sha256="2" * 64,
            evaluation_payload=evaluation,
        )

        self.assertEqual(envelope["ticket_id"], "evaluate-node-g0-function-theory-e1")
        self.assertEqual(envelope["evaluator_index"], 1)
        self.assertEqual(envelope["node_artifact_sha256"], node["node_artifact_sha256"])
        self.assertEqual(envelope["mathematical_payload_sha256"], node["mathematical_payload_sha256"])
        self.assertEqual(envelope["evaluation_payload"], evaluation)
        self.assertEqual(envelope["evaluation_payload_sha256"], digest_json(evaluation))
        without_digest = dict(envelope)
        without_digest.pop("node_evaluation_sha256")
        self.assertEqual(envelope["node_evaluation_sha256"], digest_json(without_digest))
        self.assertNotIn("run_id", json.dumps(envelope["evaluation_payload"], sort_keys=True))
        self.assertNotIn("node_artifact_sha256", json.dumps(envelope["evaluation_payload"], sort_keys=True))

    def test_evaluation_rejects_duplicate_probe_ids_and_structural_p10_without_candidate(self) -> None:
        result = expert_contracts.validate_expert_result(
            expert_result(mathematical_payload=payload(endpoint={"kind": "blocker", "text": "not a candidate"}))
        )
        decision = expert_contracts.decide_admission(attempt(), result, provenance(), outcome="accepted")
        node = expert_contracts.build_mathematical_node(decision, result, provenance())

        duplicate = evaluation_payload()
        duplicate["probes"][1]["probe_id"] = duplicate["probes"][0]["probe_id"]
        with self.assertRaisesRegex(protocol.ValidationError, "duplicate"):
            expert_contracts.build_node_evaluation(
                evaluation_id="eval-node-g0-function-theory-e1",
                evaluator_index=1,
                ticket_id="evaluate-node-g0-function-theory-e1",
                node=node,
                context_sha256="3" * 64,
                source_kind="provider_output",
                call_receipt_sha256="4" * 64,
                terminal_ticket_event_sha256="5" * 64,
                evaluation_payload=duplicate,
            )

        with self.assertRaisesRegex(protocol.ValidationError, "candidate proof"):
            expert_contracts.build_node_evaluation(
                evaluation_id="eval-node-g0-function-theory-e1",
                evaluator_index=1,
                ticket_id="evaluate-node-g0-function-theory-e1",
                node=node,
                context_sha256="3" * 64,
                source_kind="provider_output",
                call_receipt_sha256="4" * 64,
                terminal_ticket_event_sha256="5" * 64,
                evaluation_payload=evaluation_payload("pass"),
            )

    def test_conservative_fallback_is_harness_owned_insufficient_evidence(self) -> None:
        result = expert_contracts.validate_expert_result(expert_result())
        decision = expert_contracts.decide_admission(attempt(), result, provenance(), outcome="accepted")
        node = expert_contracts.build_mathematical_node(decision, result, provenance())

        fallback = expert_contracts.build_conservative_fallback_evaluation(
            evaluation_id="eval-node-g0-function-theory-e2",
            evaluator_index=2,
            ticket_id="evaluate-node-g0-function-theory-e2",
            node=node,
            context_sha256="6" * 64,
            terminal_ticket_event_sha256="7" * 64,
            call_receipt_sha256=None,
        )

        self.assertEqual(fallback["source_kind"], "conservative_fallback")
        self.assertIsNone(fallback["call_receipt_sha256"])
        self.assertEqual(
            {probe["status"] for probe in fallback["evaluation_payload"]["probes"]},
            {"insufficient_evidence"},
        )
        self.assertNotEqual(fallback["evaluation_payload"]["schema_version"], "crouzeix-node-evaluation/v1")


class SchemaAndPromptTests(unittest.TestCase):
    def test_new_schema_files_exist_and_old_overloaded_names_are_absent(self) -> None:
        schema_dir = LAB / "schemas"
        expected = {
            "expert_result.schema.json",
            "admission_decision.schema.json",
            "mathematical_node.schema.json",
            "node_evaluation.schema.json",
        }
        self.assertTrue(expected.issubset({path.name for path in schema_dir.glob("*.schema.json")}))
        self.assertFalse((schema_dir / "expert_node.schema.json").exists())
        self.assertFalse((schema_dir / "proof_progress_evaluation.schema.json").exists())

    def test_expert_result_schema_requires_canonical_result_digest(self) -> None:
        schema = json.loads((LAB / "schemas" / "expert_result.schema.json").read_text())

        self.assertIn("expert_result_sha256", schema["required"])
        self.assertEqual(
            schema["properties"]["expert_result_sha256"],
            {"$ref": "#/$defs/sha256"},
        )

    def test_prompts_use_contract_vocabulary_without_answer_bearing_mechanisms(self) -> None:
        prompt_dir = LAB / "prompts"
        expert_prompt = (prompt_dir / "expert.md").read_text()
        evaluator_prompt = (prompt_dir / "proof_progress_evaluator.md").read_text()

        self.assertIn("expert_result", expert_prompt)
        self.assertIn("candidate_proof", expert_prompt)
        self.assertIn("blocker", expert_prompt)
        self.assertIn("evaluation payload", evaluator_prompt)
        self.assertNotIn("node_artifact_sha256", evaluator_prompt)

        forbidden_terms = (
            "Jin",
            "Lorist",
            "Schwenninger",
            "positive kernel",
            "annulus",
            "spectral set",
        )
        joined = f"{expert_prompt}\n{evaluator_prompt}".lower()
        for term in forbidden_terms:
            self.assertNotIn(term.lower(), joined)


if __name__ == "__main__":
    unittest.main()
