from __future__ import annotations

import copy
import hashlib
import json
import sys
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import frontier
import expert_contracts
import protocol


DIGEST_A = "a" * 64
DIGEST_B = "b" * 64
DIGEST_C = "c" * 64
DIGEST_D = "d" * 64
DIGEST_E = "e" * 64
DIGEST_F = "f" * 64

PROBE_IDS = expert_contracts.PROOF_PROGRESS_PROBE_IDS


def digest_payload(value: object) -> str:
    data = json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode(
        "utf-8"
    )
    return hashlib.sha256(data).hexdigest()


def endpoint(kind: str = "blocker", text: str = "bounded endpoint") -> dict[str, str]:
    return {"kind": kind, "text": text}


def payload(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "proof_family": "functional calculus",
        "mechanism": "estimate the numerical range boundary",
        "proved_statements": [
            {
                "statement_id": "s1",
                "statement": "A bounded reduction is available.",
                "justification": "Direct estimate.",
                "depends_on_statement_ids": [],
            }
        ],
        "unproved_obligations": [],
        "circularity_risks": [],
        "proposed_directions": [],
        "endpoint": endpoint(),
        "confidence_basis": "local check only",
    }
    value.update(overrides)
    return value


def direction(**overrides: object) -> dict[str, object]:
    value: dict[str, object] = {
        "direction_id": "root-function-theory",
        "kind": "root_task",
        "statement": "Start an independent function theory route.",
        "strength": "root",
        "recommended_role": "function_theory",
        "source_node_artifact_sha256": None,
        "source_reconciliation_sha256": None,
    }
    value.update(overrides)
    return value


def node(
    *,
    node_id: str,
    generation: int,
    mathematical_payload: dict[str, object] | None = None,
    parent: dict[str, object] | None = None,
    selected_direction: dict[str, object] | None = None,
) -> dict[str, object]:
    base_payload = mathematical_payload or payload()
    if selected_direction is None and parent is not None:
        selected_direction = direction(
            direction_id=f"from-{parent['node_id']}",
            kind="obligation",
            strength="local",
            source_node_artifact_sha256=parent["node_artifact_sha256"],
            source_reconciliation_sha256=None,
        )
    return frontier.build_mathematical_node(
        {
            "schema_version": "crouzeix-mathematical-node/v1",
            "run_id": "expert-frontier-001",
            "node_id": node_id,
            "theorem_sha256": DIGEST_A,
            "leakage_tier": "L0",
            "parent_node_id": None if parent is None else parent["node_id"],
            "parent_node_artifact_sha256": None
            if parent is None
            else parent["node_artifact_sha256"],
            "generation": generation,
            "expert_role": "function_theory",
            "selected_direction": selected_direction or direction(),
            "source_attempt_id": f"attempt-{node_id}",
            "source_ticket_id": f"ticket-{node_id}",
            "source_expert_result_sha256": DIGEST_B,
            "source_context_sha256": DIGEST_C,
            "source_call_receipt_sha256": DIGEST_D,
            "admission_decision_sha256": DIGEST_E,
            "mathematical_payload": base_payload,
        }
    )


def evaluation(
    *,
    evaluator_index: int,
    statuses: dict[str, str],
    node_artifact_sha256: str,
    mathematical_payload_sha256: str,
    findings: list[dict[str, object]] | None = None,
    evaluation_id: str | None = None,
) -> dict[str, object]:
    evaluation_payload = {
        "probes": [
            {"probe_id": probe_id, "status": statuses[probe_id]} for probe_id in PROBE_IDS
        ],
        "findings": findings or [],
    }
    result = {
        "schema_version": "crouzeix-node-evaluation/v1",
        "evaluation_id": evaluation_id or f"eval-{evaluator_index}",
        "evaluator_index": evaluator_index,
        "ticket_id": f"evaluate-node-g0-e{evaluator_index}",
        "node_artifact_sha256": node_artifact_sha256,
        "mathematical_payload_sha256": mathematical_payload_sha256,
        "context_sha256": "1" * 64 if evaluator_index == 1 else "2" * 64,
        "source_kind": "provider_output",
        "call_receipt_sha256": "3" * 64 if evaluator_index == 1 else "4" * 64,
        "terminal_ticket_event_sha256": "5" * 64 if evaluator_index == 1 else "6" * 64,
        "evaluation_payload": evaluation_payload,
        "evaluation_payload_sha256": digest_payload(evaluation_payload),
    }
    result["node_evaluation_sha256"] = digest_payload(result)
    return result


def reconciliation_for(
    mathematical_node: dict[str, object],
    pass_count: int,
    *,
    findings: list[dict[str, object]] | None = None,
) -> dict[str, object]:
    statuses_one = {probe_id: "pass" for probe_id in PROBE_IDS}
    statuses_two = {
        probe_id: "pass" if index < pass_count else "fail"
        for index, probe_id in enumerate(PROBE_IDS)
    }
    second_findings = None
    if pass_count == 10 and findings is None:
        second_findings = [
            {
                "finding_id": "all-pass-note",
                "severity": "minor",
                "statement": "Independent evaluator recorded no blocking issue.",
                "locator": "candidate.tex#L1",
                "recommended_role": "approximation_audit",
            }
        ]
    return frontier.reconcile_node_evaluations(
        "reconcile-" + str(mathematical_node["node_id"]),
        evaluation(
            evaluator_index=1,
            statuses=statuses_one,
            node_artifact_sha256=str(mathematical_node["node_artifact_sha256"]),
            mathematical_payload_sha256=str(mathematical_node["mathematical_payload_sha256"]),
            findings=findings,
        ),
        evaluation(
            evaluator_index=2,
            statuses=statuses_two,
            node_artifact_sha256=str(mathematical_node["node_artifact_sha256"]),
            mathematical_payload_sha256=str(mathematical_node["mathematical_payload_sha256"]),
            findings=second_findings,
        ),
    )


def entry_for(
    mathematical_node: dict[str, object],
    pass_count: int,
    *,
    findings: list[dict[str, object]] | None = None,
) -> tuple[dict[str, object], dict[str, object]]:
    reconciliation = reconciliation_for(mathematical_node, pass_count, findings=findings)
    return frontier.build_archive_entry(mathematical_node, reconciliation), reconciliation


class ReconciliationTests(unittest.TestCase):
    def test_frontier_probe_ids_are_the_semantic_contract_ids(self) -> None:
        self.assertEqual(frontier.PROBE_IDS, expert_contracts.PROOF_PROGRESS_PROBE_IDS)
        self.assertNotIn("probe-01", frontier.PROBE_IDS)

    def test_reconciliation_uses_two_evaluation_digests_and_integer_pass_count(self) -> None:
        mathematical_node = node(node_id="node-a", generation=0)
        findings = [
            {
                "finding_id": "gap",
                "severity": "critical",
                "statement": "The boundary estimate is unproved.",
                "locator": "candidate.tex#L12",
                "recommended_role": "matrix_extremal",
            }
        ]

        left = evaluation(
            evaluator_index=1,
            statuses={probe_id: "pass" for probe_id in PROBE_IDS},
            node_artifact_sha256=str(mathematical_node["node_artifact_sha256"]),
            mathematical_payload_sha256=str(mathematical_node["mathematical_payload_sha256"]),
            findings=findings,
        )
        right = evaluation(
            evaluator_index=2,
            statuses={
                probe_id: "pass" if index < 7 else "fail"
                for index, probe_id in enumerate(PROBE_IDS)
            },
            node_artifact_sha256=str(mathematical_node["node_artifact_sha256"]),
            mathematical_payload_sha256=str(mathematical_node["mathematical_payload_sha256"]),
        )

        reconciliation = frontier.reconcile_node_evaluations("reconcile-node-a", left, right)

        self.assertEqual(reconciliation["unanimous_pass_count"], 7)
        self.assertEqual(reconciliation["disagreement_count"], 3)
        self.assertEqual(
            [probe["probe_id"] for probe in reconciliation["probes"]],
            list(PROBE_IDS),
        )
        self.assertEqual(
            reconciliation["probes"][0]["probe_id"],
            "p01_theorem_statement_preserved",
        )
        self.assertEqual(
            reconciliation["node_evaluation_sha256s"],
            [left["node_evaluation_sha256"], right["node_evaluation_sha256"]],
        )
        self.assertEqual(reconciliation["evaluator_findings"][0]["finding_id"], "e1:gap")
        self.assertEqual(reconciliation["evaluator_findings"][0]["source_finding_id"], "gap")
        self.assertEqual(reconciliation["evaluator_findings"][0]["source_evaluator_index"], 1)
        self.assertNotIn("alpha", reconciliation)

        duplicate_probe = evaluation(
            evaluator_index=1,
            statuses={probe_id: "pass" for probe_id in PROBE_IDS},
            node_artifact_sha256=str(mathematical_node["node_artifact_sha256"]),
            mathematical_payload_sha256=str(mathematical_node["mathematical_payload_sha256"]),
        )
        duplicate_probe["evaluation_payload"]["probes"][1]["probe_id"] = PROBE_IDS[0]
        duplicate_probe["evaluation_payload_sha256"] = digest_payload(
            duplicate_probe["evaluation_payload"]
        )
        duplicate_probe["node_evaluation_sha256"] = digest_payload(
            {k: v for k, v in duplicate_probe.items() if k != "node_evaluation_sha256"}
        )
        with self.assertRaisesRegex(protocol.ValidationError, "ten probe IDs"):
            frontier.reconcile_node_evaluations(
                "bad",
                duplicate_probe,
                evaluation(
                    evaluator_index=2,
                    statuses={probe_id: "pass" for probe_id in PROBE_IDS},
                    node_artifact_sha256=str(mathematical_node["node_artifact_sha256"]),
                    mathematical_payload_sha256=str(mathematical_node["mathematical_payload_sha256"]),
                ),
            )

        missing_probe = evaluation(
            evaluator_index=1,
            statuses={probe_id: "pass" for probe_id in PROBE_IDS},
            node_artifact_sha256=str(mathematical_node["node_artifact_sha256"]),
            mathematical_payload_sha256=str(mathematical_node["mathematical_payload_sha256"]),
        )
        missing_probe["evaluation_payload"]["probes"].pop()
        missing_probe["evaluation_payload_sha256"] = digest_payload(
            missing_probe["evaluation_payload"]
        )
        missing_probe["node_evaluation_sha256"] = digest_payload(
            {k: v for k, v in missing_probe.items() if k != "node_evaluation_sha256"}
        )
        with self.assertRaisesRegex(protocol.ValidationError, "ten probe IDs"):
            frontier.reconcile_node_evaluations(
                "missing",
                missing_probe,
                evaluation(
                    evaluator_index=2,
                    statuses={probe_id: "pass" for probe_id in PROBE_IDS},
                    node_artifact_sha256=str(mathematical_node["node_artifact_sha256"]),
                    mathematical_payload_sha256=str(mathematical_node["mathematical_payload_sha256"]),
                ),
            )

        out_of_order = evaluation(
            evaluator_index=1,
            statuses={probe_id: "pass" for probe_id in PROBE_IDS},
            node_artifact_sha256=str(mathematical_node["node_artifact_sha256"]),
            mathematical_payload_sha256=str(mathematical_node["mathematical_payload_sha256"]),
        )
        out_of_order["evaluation_payload"]["probes"] = list(
            reversed(out_of_order["evaluation_payload"]["probes"])
        )
        out_of_order["evaluation_payload_sha256"] = digest_payload(
            out_of_order["evaluation_payload"]
        )
        out_of_order["node_evaluation_sha256"] = digest_payload(
            {k: v for k, v in out_of_order.items() if k != "node_evaluation_sha256"}
        )
        with self.assertRaisesRegex(protocol.ValidationError, "probe order"):
            frontier.reconcile_node_evaluations(
                "out-of-order",
                out_of_order,
                evaluation(
                    evaluator_index=2,
                    statuses={probe_id: "pass" for probe_id in PROBE_IDS},
                    node_artifact_sha256=str(mathematical_node["node_artifact_sha256"]),
                    mathematical_payload_sha256=str(mathematical_node["mathematical_payload_sha256"]),
                ),
            )

    def test_reconciliation_rejects_mismatched_nodes_and_duplicate_evaluator_artifacts(self) -> None:
        first = node(node_id="node-a", generation=0)
        second = node(node_id="node-b", generation=0)
        statuses = {probe_id: "pass" for probe_id in PROBE_IDS}
        left = evaluation(
            evaluator_index=1,
            statuses=statuses,
            node_artifact_sha256=str(first["node_artifact_sha256"]),
            mathematical_payload_sha256=str(first["mathematical_payload_sha256"]),
        )
        right = evaluation(
            evaluator_index=2,
            statuses=statuses,
            node_artifact_sha256=str(second["node_artifact_sha256"]),
            mathematical_payload_sha256=str(second["mathematical_payload_sha256"]),
        )
        with self.assertRaisesRegex(protocol.ValidationError, "same node_artifact_sha256"):
            frontier.reconcile_node_evaluations("mismatch", left, right)

        duplicate = evaluation(
            evaluator_index=2,
            statuses=statuses,
            node_artifact_sha256=str(first["node_artifact_sha256"]),
            mathematical_payload_sha256=str(first["mathematical_payload_sha256"]),
            findings=[
                {
                    "finding_id": "note",
                    "severity": "minor",
                    "statement": "Independent note.",
                    "locator": "candidate.tex#L1",
                    "recommended_role": "approximation_audit",
                }
            ],
        )
        duplicate["ticket_id"] = left["ticket_id"]
        duplicate["node_evaluation_sha256"] = digest_payload(
            {k: v for k, v in duplicate.items() if k != "node_evaluation_sha256"}
        )
        with self.assertRaisesRegex(protocol.ValidationError, "distinct"):
            frontier.reconcile_node_evaluations("duplicate", left, duplicate)


class ArchiveEntryTests(unittest.TestCase):
    def test_archive_entry_binds_node_reconciliation_and_candidate_digest(self) -> None:
        mathematical_node = node(
            node_id="node-proof",
            generation=0,
            mathematical_payload=payload(
                endpoint=endpoint("candidate_proof", "\\begin{proof}ok\\end{proof}")
            ),
        )
        archive_entry, reconciliation = entry_for(mathematical_node, 10)

        self.assertEqual(archive_entry["node_id"], "node-proof")
        self.assertEqual(
            archive_entry["candidate_proof_sha256"],
            hashlib.sha256("\\begin{proof}ok\\end{proof}".encode("utf-8")).hexdigest(),
        )
        self.assertEqual(archive_entry["unanimous_pass_count"], 10)
        self.assertEqual(
            archive_entry["reconciliation_sha256"],
            reconciliation["reconciliation_sha256"],
        )
        self.assertIn("archive_entry_sha256", archive_entry)
        self.assertNotIn("child_count", archive_entry)
        self.assertNotIn("selected", archive_entry)

        broken = dict(reconciliation)
        broken["node_artifact_sha256"] = DIGEST_F
        broken["reconciliation_sha256"] = digest_payload(
            {key: value for key, value in broken.items() if key != "reconciliation_sha256"}
        )
        with self.assertRaisesRegex(protocol.ValidationError, "node_artifact_sha256"):
            frontier.build_archive_entry(mathematical_node, broken)

    def test_keep_all_archive_retains_lower_scoring_children(self) -> None:
        root = node(node_id="node-root", generation=0)
        child = node(node_id="node-child", generation=1, parent=root)
        root_entry, _ = entry_for(root, 8)
        child_entry, _ = entry_for(child, 2)

        archive = frontier.keep_all_archive([root_entry, child_entry])

        self.assertEqual([entry["node_id"] for entry in archive], ["node-child", "node-root"])
        self.assertLess(child_entry["unanimous_pass_count"], root_entry["unanimous_pass_count"])

    def test_archive_validation_rejects_stale_digest_after_score_forgery(self) -> None:
        mathematical_node = node(
            node_id="node-complete",
            generation=0,
            mathematical_payload=payload(endpoint=endpoint("candidate_proof", "complete")),
        )
        archive_entry, _ = entry_for(mathematical_node, 10)

        forged = dict(archive_entry)
        forged["unanimous_pass_count"] = 9

        with self.assertRaisesRegex(protocol.ValidationError, "archive_entry_sha256"):
            frontier.keep_all_archive([forged])
        with self.assertRaisesRegex(protocol.ValidationError, "archive_entry_sha256"):
            frontier.select_generation([forged], [mathematical_node], seed=20260814, generation=1)

    def test_archive_entry_construction_rejects_forged_reconciliation_score(self) -> None:
        mathematical_node = node(
            node_id="node-complete",
            generation=0,
            mathematical_payload=payload(endpoint=endpoint("candidate_proof", "complete")),
        )
        _, reconciliation = entry_for(mathematical_node, 10)

        forged = dict(reconciliation)
        forged["unanimous_pass_count"] = 9

        with self.assertRaisesRegex(
            protocol.ValidationError, "reconciliation_sha256|unanimous_pass_count"
        ):
            frontier.build_archive_entry(mathematical_node, forged)

    def test_archive_entry_construction_rejects_digest_consistent_forged_probe_score(self) -> None:
        mathematical_node = node(
            node_id="node-complete",
            generation=0,
            mathematical_payload=payload(endpoint=endpoint("candidate_proof", "complete")),
        )
        _, reconciliation = entry_for(mathematical_node, 10)

        forged = dict(reconciliation)
        forged["unanimous_pass_count"] = 9
        forged["reconciliation_sha256"] = digest_payload(
            {key: value for key, value in forged.items() if key != "reconciliation_sha256"}
        )

        with self.assertRaisesRegex(protocol.ValidationError, "unanimous_pass_count"):
            frontier.build_archive_entry(mathematical_node, forged)


class NodeConstructionTests(unittest.TestCase):
    def test_mathematical_node_validates_parent_generation_and_direction_source(self) -> None:
        root = node(node_id="node-root", generation=0)
        child_direction = direction(
            direction_id="from-finding",
            kind="evaluator_finding",
            strength="critical",
            source_node_artifact_sha256=root["node_artifact_sha256"],
            source_reconciliation_sha256=DIGEST_F,
        )
        child = node(
            node_id="node-child",
            generation=1,
            parent=root,
            selected_direction=child_direction,
        )
        self.assertEqual(child["parent_node_artifact_sha256"], root["node_artifact_sha256"])
        self.assertEqual(child["selected_direction_sha256"], digest_payload(child_direction))

        with self.assertRaisesRegex(protocol.ValidationError, "parent"):
            frontier.build_mathematical_node(
                {
                    "schema_version": "crouzeix-mathematical-node/v1",
                    "run_id": "expert-frontier-001",
                    "node_id": "bad-child",
                    "theorem_sha256": DIGEST_A,
                    "leakage_tier": "L0",
                    "parent_node_id": "node-root",
                    "parent_node_artifact_sha256": None,
                    "generation": 1,
                    "expert_role": "function_theory",
                    "selected_direction": child_direction,
                    "source_attempt_id": "attempt-bad",
                    "source_ticket_id": "ticket-bad",
                    "source_expert_result_sha256": DIGEST_B,
                    "source_context_sha256": DIGEST_C,
                    "source_call_receipt_sha256": DIGEST_D,
                    "admission_decision_sha256": DIGEST_E,
                    "mathematical_payload": payload(),
                }
            )

    def test_mathematical_node_rejects_unknown_selected_direction_role(self) -> None:
        bad_direction = direction(recommended_role="unknown-role")

        with self.assertRaisesRegex(protocol.ValidationError, "recommended_role"):
            node(node_id="node-bad-role", generation=0, selected_direction=bad_direction)


class SelectionTests(unittest.TestCase):
    def test_exact_dgm_draws_bind_one_snapshot_and_exclude_complete_nodes(self) -> None:
        root = node(node_id="node-root", generation=0)
        middle = node(node_id="node-middle", generation=0)
        complete = node(
            node_id="node-complete",
            generation=0,
            mathematical_payload=payload(endpoint=endpoint("candidate_proof", "complete")),
        )
        child = node(node_id="node-child", generation=1, parent=root)
        entries_and_reconciliations = [
            entry_for(root, 6),
            entry_for(middle, 8),
            entry_for(complete, 10),
            entry_for(child, 3),
        ]
        entries = [pair[0] for pair in entries_and_reconciliations]
        nodes = [root, middle, complete, child]

        golden = {
            1: {
                "snapshot": "1128ac359ca043e213cc13a7e5d14f17be520eb3d9432609db3835cf8b96a4fc",
                "events": [
                    (
                        "63726f757a6569782d66726f6e746965722f763100323032363038313400310030",
                        "69b498ba05b369d5cb9f426be251d02750106ebaf4f11f563dce942a1387ccb4",
                        47811935653498240072031748538651364671312772502834154747365072972128455216308,
                        "0.41291193524894045700971252331675301381789478073510814423354530244601255925717784",
                        "node-middle",
                        "0.082934944943438368713751346808053027677479305241142223520084224675312735506156301",
                        "0.74568449354849669978531104947170421685868359401397264248961559483844300031956078",
                    ),
                    (
                        "63726f757a6569782d66726f6e746965722f763100323032363038313400310031",
                        "7f15d45f3ae097e35bcce65a93fcacefe69adc4a1dfe560c7538343c45aad3f0",
                        57482301296056135619951154671669852338659841806284819290658368769563734758384,
                        "0.4964268429274646383680890225027770679873268897741964703544014776525268294682548",
                        "node-middle",
                        "0.082934944943438368713751346808053027677479305241142223520084224675312735506156301",
                        "0.74568449354849669978531104947170421685868359401397264248961559483844300031956078",
                    ),
                ],
            },
            2: {
                "snapshot": "94bb01c384d597a827d500b7d25a4059ec59c1beaa2e14fd7e035af2a0b3e7b5",
                "events": [
                    (
                        "63726f757a6569782d66726f6e746965722f763100323032363038313400320030",
                        "f9cae00ef069552dd2c83988ffcac4436938772a9068f28163a70efb26ac9fca",
                        112984348798257780138385993803865617987980581250755348956145201637729724309450,
                        "0.97575188030933664411200503846536838083689684037687824277867243096291234382126794",
                        "node-root",
                        "0.74568449354849669978531104947170421685868359401397264248961559483844300031956078",
                        "1",
                    ),
                    (
                        "63726f757a6569782d66726f6e746965722f763100323032363038313400320031",
                        "649b0f9c3210741d73dd268fb9ac2b3065fd79b814a0b3d60b5f06bc5a40c99d",
                        45505253890586553533721648966570490850987488675246190929056911739332673456541,
                        "0.39299104274147273079055831461421233019670895122984498357011907528136430330508538",
                        "node-middle",
                        "0.082934944943438368713751346808053027677479305241142223520084224675312735506156301",
                        "0.74568449354849669978531104947170421685868359401397264248961559483844300031956078",
                    ),
                ],
            },
            3: {
                "snapshot": "3ef9d7ac282666790c3a202b41e06400ad8fa3c3cf6a864d51b013f9ca96717f",
                "events": [
                    (
                        "63726f757a6569782d66726f6e746965722f763100323032363038313400330030",
                        "0edd0a6044e1370fec82a9b171354b3b7eb5991b0a7f1e409790da77ddd0ae3b",
                        6722924694353984272523126614831948659477101672073665482612712080697610776123,
                        "0.058060310843647809117587692722300084872129993286257154677328002214340111383809218",
                        "node-child",
                        "0",
                        "0.082934944943438368713751346808053027677479305241142223520084224675312735506156301",
                    ),
                    (
                        "63726f757a6569782d66726f6e746965722f763100323032363038313400330031",
                        "d87bbc5995e56d03529a4210cd66600740e5cb6d7efbc93bdfa437ab38b197ec",
                        97918197426487642935200942090426683582054087079063908269931294046254280054764,
                        "0.84563805758616235348750174716964863575226592271168713145870722020649168655462287",
                        "node-root",
                        "0.74568449354849669978531104947170421685868359401397264248961559483844300031956078",
                        "1",
                    ),
                ],
            },
        }

        for generation, expected in golden.items():
            result = frontier.select_generation(entries, nodes, seed=20260814, generation=generation)
            self.assertEqual(len(result["selection_events"]), 2)
            self.assertEqual(result["snapshot"]["archive_snapshot_sha256"], expected["snapshot"])
            self.assertEqual(
                result["snapshot"]["eligible_node_ids"],
                ["node-child", "node-middle", "node-root"],
            )
            self.assertNotIn("node-complete", result["snapshot"]["eligible_node_ids"])
            self.assertEqual(result["snapshot"]["child_counts_by_node_id"]["node-root"], 1)
            self.assertEqual(result["snapshot"]["child_counts_by_node_id"]["node-child"], 0)
            self.assertEqual(
                {event["archive_snapshot_sha256"] for event in result["selection_events"]},
                {result["snapshot"]["archive_snapshot_sha256"]},
            )
            for event, expected_event in zip(result["selection_events"], expected["events"]):
                (
                    preimage_hex,
                    draw_sha256,
                    unsigned_integer,
                    uniform_decimal,
                    selected_node_id,
                    interval_start,
                    interval_end,
                ) = expected_event
                self.assertEqual(event["draw_preimage_hex"], preimage_hex)
                self.assertEqual(event["draw_sha256"], draw_sha256)
                self.assertEqual(event["draw_unsigned_integer"], unsigned_integer)
                self.assertEqual(event["uniform_decimal"], uniform_decimal)
                self.assertEqual(event["selected_node_id"], selected_node_id)
                self.assertEqual(event["selected_interval_start_decimal"], interval_start)
                self.assertEqual(event["selected_interval_end_decimal"], interval_end)

        terms = frontier.select_generation(entries, nodes, seed=20260814, generation=1)[
            "selection_events"
        ][0]["terms"]
        self.assertEqual([term["node_id"] for term in terms], ["node-child", "node-middle", "node-root"])
        self.assertEqual(
            [term["alpha_decimal"] for term in terms],
            ["0.3", "0.8", "0.6"],
        )
        self.assertEqual(terms[2]["underexploration_decimal"], "0.5")
        self.assertEqual(
            [term["probability_decimal"] for term in terms],
            [
                "0.082934944943438368713751346808053027677479305241142223520084224675312735506156301",
                "0.66274954860505833107155970266365118918120428877283041896953137016313026481340448",
                "0.25431550645150330021468895052829578314131640598602735751038440516155699968043922",
            ],
        )
        self.assertEqual(terms[-1]["interval_end_decimal"], "1")

    def test_selection_rejects_empty_eligible_set_and_invalid_pass_counts(self) -> None:
        complete = node(
            node_id="node-complete",
            generation=0,
            mathematical_payload=payload(endpoint=endpoint("candidate_proof", "complete")),
        )
        complete_entry, _ = entry_for(complete, 10)
        with self.assertRaisesRegex(protocol.ValidationError, "eligible"):
            frontier.select_generation([complete_entry], [complete], seed=20260814, generation=1)

        broken = dict(complete_entry)
        broken["unanimous_pass_count"] = 11
        with self.assertRaisesRegex(protocol.ValidationError, "unanimous_pass_count"):
            frontier.select_generation([broken], [complete], seed=20260814, generation=1)


class DirectionTests(unittest.TestCase):
    def test_direction_priority_joins_reconciliation_findings_without_mutating_node(self) -> None:
        original_payload = payload(
            unproved_obligations=[
                {
                    "obligation_id": "minor-cleanup",
                    "statement": "Local compactness detail.",
                    "strength": "local",
                    "recommended_role": "approximation_audit",
                }
            ],
            proposed_directions=[
                {
                    "direction_id": "try-approximation",
                    "statement": "Try approximation by rational functions.",
                    "recommended_role": "approximation_audit",
                }
            ],
        )
        mathematical_node = node(node_id="node-with-directions", generation=0, mathematical_payload=original_payload)
        before = copy.deepcopy(mathematical_node)
        archive_entry, reconciliation = entry_for(
            mathematical_node,
            5,
            findings=[
                {
                    "finding_id": "critical-gap",
                    "severity": "critical",
                    "statement": "The dilation step is unsupported.",
                    "locator": "candidate.tex#L20",
                    "recommended_role": "operator_dilation",
                }
            ],
        )

        selected = frontier.select_frontier_direction(mathematical_node, reconciliation)

        self.assertEqual(selected["direction_id"], "finding-e1:critical-gap")
        self.assertEqual(selected["kind"], "evaluator_finding")
        self.assertEqual(selected["strength"], "critical")
        self.assertEqual(selected["recommended_role"], "operator_dilation")
        self.assertEqual(selected["source_node_artifact_sha256"], archive_entry["node_artifact_sha256"])
        self.assertEqual(selected["source_reconciliation_sha256"], reconciliation["reconciliation_sha256"])
        self.assertEqual(mathematical_node, before)

    def test_duplicate_parent_draws_receive_stable_distinct_directions_when_available(self) -> None:
        mathematical_node = node(
            node_id="node-parent",
            generation=0,
            mathematical_payload=payload(
                unproved_obligations=[
                    {
                        "obligation_id": "theorem-gap",
                        "statement": "Close theorem-strength gap.",
                        "strength": "theorem_strength",
                        "recommended_role": "matrix_extremal",
                    },
                    {
                        "obligation_id": "local-gap",
                        "statement": "Close local gap.",
                        "strength": "local",
                        "recommended_role": "completion_positivity",
                    },
                ]
            ),
        )
        _, reconciliation = entry_for(mathematical_node, 5)
        events = [
            {"draw_index": 0, "selected_node_id": "node-parent"},
            {"draw_index": 1, "selected_node_id": "node-parent"},
        ]

        assignments = frontier.assign_directions_to_selection_events(
            events,
            {"node-parent": mathematical_node},
            {str(reconciliation["node_artifact_sha256"]): reconciliation},
        )

        self.assertEqual(
            [assignment["selected_direction"]["direction_id"] for assignment in assignments],
            ["obligation-theorem-gap", "obligation-local-gap"],
        )
        self.assertEqual(assignments[1]["selected_direction"]["recommended_role"], "completion_positivity")
        self.assertNotIn("selected_direction", events[0])

    def test_direction_projection_rejects_unknown_roles(self) -> None:
        mathematical_node = node(
            node_id="node-parent",
            generation=0,
            mathematical_payload=payload(
                unproved_obligations=[
                    {
                        "obligation_id": "local-gap",
                        "statement": "Close local gap.",
                        "strength": "local",
                        "recommended_role": "unknown-role",
                    },
                ]
            ),
        )
        _, reconciliation = entry_for(mathematical_node, 5)

        with self.assertRaisesRegex(protocol.ValidationError, "recommended_role"):
            frontier.select_frontier_direction(mathematical_node, reconciliation)


if __name__ == "__main__":
    unittest.main()
