from __future__ import annotations

import copy
import dataclasses
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from labs.crouzeix_proof_reproduction import theorem_graph


REPO = Path(__file__).resolve().parents[3]


class TheoremGraphTests(unittest.TestCase):
    def test_builds_wave_one_harp_proof_obligation_graph(self) -> None:
        graph = theorem_graph.build_proof_obligation_graph(REPO)
        payload = graph.to_json()

        self.assertEqual(
            payload["schema_version"],
            "crouzeix-proof-obligation-graph/v2",
        )
        self.assertEqual(len(payload["nodes"]), 18)
        self.assertEqual(
            payload["frontier"]["open_obligations"],
            [
                "harp-l2-witness-boundary-embedding-data",
                "harp-l2-witness-dimension-count",
                "harp-terminal-witness-to-finite-bound",
            ],
        )
        self.assertEqual(payload["frontier"]["blocked_obligations"], [])
        self.assertEqual(
            payload["frontier"]["missing_obligation_readbacks"],
            ["harp-terminal-witness-to-finite-bound"],
        )
        self.assertEqual(
            payload["frontier"]["oversized_parent_nodes"],
            ["jin-fixed-outer-domain-convergence"],
        )
        parents = {item["parent_node_id"]: item for item in payload["parents"]}
        self.assertEqual(
            {key: value["obligation_status"] for key, value in parents.items()},
            {
                "harp-counting-l2-witness-dimension-bound": "expanded-incomplete",
                "harp-operator-recurrence": "expanded-current",
                "harp-perturbation-endpoint": "expanded-current",
                "harp-terminal-theorem": "expanded-incomplete",
            },
        )
        route_graph = theorem_graph.build_theorem_graph(REPO)
        self.assertEqual(route_graph.frontier["open_nodes"], ())
        self.assertEqual(route_graph.frontier["blocked_nodes"], ())
        by_id = {item["node_id"]: item for item in payload["nodes"]}
        for node_id in payload["frontier"]["open_obligations"]:
            self.assertIsNone(by_id[node_id]["lean_name"])
            self.assertEqual(by_id[node_id]["readback_status"], "missing")
            self.assertTrue(by_id[node_id]["proposed_lean_name"])
            self.assertIsNotNone(by_id[node_id]["extraction_source"])
        for parent in parents.values():
            children = [by_id[node_id] for node_id in parent["child_ids"]]
            completion = [child for child in children if child["covers_parent"]]
            self.assertEqual(len(completion), 1)
            self.assertEqual(
                {child["node_id"] for child in children if not child["covers_parent"]}
                - set(completion[0]["dependency_ids"]),
                set(),
            )

    def test_builds_current_crouzeix_graph_from_route_manifests(self) -> None:
        graph = theorem_graph.build_theorem_graph(REPO)
        payload = graph.to_json()

        self.assertEqual(payload["schema_version"], "crouzeix-theorem-graph/v1")
        self.assertEqual(
            [route["route_id"] for route in payload["routes"]],
            ["jin", "lorist-schwenninger", "harp"],
        )
        self.assertGreaterEqual(len(payload["nodes"]), 30)
        self.assertEqual(payload["frontier"]["open_nodes"], [])
        self.assertEqual(payload["frontier"]["blocked_nodes"], [])
        self.assertIn("ls-perturbation-lemma", payload["frontier"]["external_candidates"])
        self.assertIn("harp-terminal-theorem", payload["frontier"]["external_candidates"])
        self.assertEqual(payload["frontier"]["missing_readbacks"], [])
        by_id = {node["node_id"]: node for node in payload["nodes"]}
        self.assertEqual(by_id["ls-terminal-crouzeix"]["readback_status"], "current")
        self.assertIn(
            "Lorist-Schwenninger",
            by_id["ls-terminal-crouzeix"]["natural_language_statement"],
        )
        self.assertEqual(by_id["harp-closed-range-consequence"]["readback_status"], "current")

    def test_validate_accepts_proof_obligation_builder_output(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)

        graph = theorem_graph.validate_proof_obligation_graph(payload, REPO)

        self.assertEqual(
            graph.schema_version,
            "crouzeix-proof-obligation-graph/v2",
        )

    def test_obligation_validation_rejects_missing_covering_completion(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        completion = next(item for item in payload["nodes"] if item["covers_parent"])
        completion["covers_parent"] = False
        completion["dependency_ids"] = []

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "covering completion"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_incomplete_parent_coverage(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        completion = next(item for item in payload["nodes"] if item["covers_parent"])
        completion["dependency_ids"] = completion["dependency_ids"][1:]

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "does not cover children"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_multiple_covering_completions(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        first_parent = payload["nodes"][0]["parent_node_id"]
        sibling = next(
            item
            for item in payload["nodes"]
            if item["parent_node_id"] == first_parent and not item["covers_parent"]
        )
        sibling["covers_parent"] = True

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "covering completion"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_raw_route_shortcut_from_completion(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        completion = next(item for item in payload["nodes"] if item["covers_parent"])
        completion["dependency_ids"].append("harp-double-layer-application")

        with self.assertRaisesRegex(
            theorem_graph.TheoremGraphError, "invalid covering completion dependency"
        ):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_invented_sibling_dependency(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        node = next(
            item
            for item in payload["nodes"]
            if item["node_id"] == "harp-l2-witness-compression-moments"
        )
        node["dependency_ids"].append("harp-l2-witness-power-cauchy-moments")

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "unjustified obligation dependency"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_planned_obligation_requires_existing_extraction_source(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        node = next(item for item in payload["nodes"] if item["proof_status"] == "open")
        node["extraction_source"] = {
            "module_path": "formalization/lean/Crouzeix/Harp/MainTheorem.lean",
            "declaration": "CrouzeixConjecture.Harp.notADeclaration",
        }

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "extraction source declaration"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_terminal_limit_refinements_are_independent(self) -> None:
        graph = theorem_graph.build_proof_obligation_graph(REPO)
        by_id = {node.node_id: node for node in graph.nodes}

        self.assertNotIn(
            "harp-terminal-inner-limit-transfer",
            by_id["harp-terminal-outer-limit-transfer"].dependency_ids,
        )

    def test_obligation_validation_rejects_unknown_parent(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        payload["nodes"][0]["parent_node_id"] = "missing-parent"

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "unknown obligation parent"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_dangling_dependency(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        payload["nodes"][0]["dependency_ids"] = ["missing-dependency"]

        with self.assertRaisesRegex(
            theorem_graph.TheoremGraphError, "dangling obligation dependency"
        ):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_cycles(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        first = next(
            node for node in payload["nodes"]
            if node["node_id"] == "harp-l2-witness-boundary-embedding-data"
        )
        second = next(
            node for node in payload["nodes"]
            if node["node_id"] == "harp-l2-witness-dimension-count"
        )
        first["dependency_ids"] = [second["node_id"]]
        second["dependency_ids"] = [first["node_id"]]

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "obligation cycle"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_passed_node_without_lean_statement(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        node = next(item for item in payload["nodes"] if item["proof_status"] == "passed-local")
        node["lean_name"] = None

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "lacks Lean statement"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_current_readback_without_lean_statement(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        node = next(item for item in payload["nodes"] if item["proof_status"] == "open")
        node["readback_status"] = "current"

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "unbound current readback"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_passed_node_with_open_dependency(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        passed = next(item for item in payload["nodes"] if item["proof_status"] == "passed-local")
        open_node = next(item for item in payload["nodes"] if item["proof_status"] == "open")
        passed["dependency_ids"] = [open_node["node_id"]]

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "unproved obligation dependency"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_unproved_route_dependency(self) -> None:
        graph = theorem_graph.build_proof_obligation_graph(REPO)
        route_graph = theorem_graph.build_theorem_graph(REPO)
        route_nodes = {node.node_id: node for node in route_graph.nodes}
        route_nodes["harp-finite-measure-cubature"] = dataclasses.replace(
            route_nodes["harp-finite-measure-cubature"],
            proof_status="open",
            readback_status="missing",
        )

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "unproved obligation dependency"):
            theorem_graph._validate_obligation_graph(REPO, route_nodes, graph.nodes)

    def test_obligation_validation_rejects_stale_statement_binding(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        payload["nodes"][0]["statement_sha256"] = "0" * 64

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "statement hash mismatch"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_unknown_kind_role_and_route(self) -> None:
        mutations = (
            ("kind", "nonsense-kind", "invalid obligation kind"),
            ("role", "nonsense-role", "invalid obligation role"),
            ("route_ids", ["not-a-route"], "unknown obligation route"),
        )
        for field, value, message in mutations:
            with self.subTest(field=field):
                payload = theorem_graph.build_proof_obligation_graph_json(REPO)
                payload["nodes"][0][field] = value
                with self.assertRaisesRegex(theorem_graph.TheoremGraphError, message):
                    theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_planned_obligation_statement_is_confined_to_planned_subtree(self) -> None:
        for path in ("docs/import-receipt.md", ".git/HEAD"):
            with self.subTest(path=path):
                payload = theorem_graph.build_proof_obligation_graph_json(REPO)
                node = next(
                    item for item in payload["nodes"] if item["proof_status"] == "open"
                )
                node["statement_text_paths"] = [path]

                with self.assertRaisesRegex(
                    theorem_graph.TheoremGraphError, "planned statement path"
                ):
                    theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_rejects_stale_readback(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        payload["nodes"][0]["readback_status"] = "stale"

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "stale obligation readback"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_obligation_validation_requires_declaration_in_receipt(self) -> None:
        graph = theorem_graph.build_proof_obligation_graph(REPO)
        node = next(item for item in graph.nodes if item.proof_status == "passed-local")
        unreceipted = dataclasses.replace(
            node, lean_name="CrouzeixConjecture.Harp.unreceiptedDeclaration"
        )

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "receipt lacks declaration"):
            theorem_graph._validate_receipt_binding(REPO, unreceipted)

    def test_obligation_validation_rejects_frontier_mismatch(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        payload["frontier"]["open_obligations"] = []

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "frontier"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_prove2me_export_is_deterministic_and_offline(self) -> None:
        first = theorem_graph.build_prove2me_export(REPO)
        second = theorem_graph.build_prove2me_export(REPO)

        self.assertEqual(first, second)
        self.assertEqual(first["schema_version"], "crouzeix-prove2me-export/v2")
        self.assertTrue(first["dry_run"])
        self.assertFalse(first["network_access"])
        graph = theorem_graph.build_proof_obligation_graph(REPO)
        self.assertEqual(
            [card["theorem_id"] for card in first["theorem_cards"]],
            list(graph.frontier["prove2me_candidates"]),
        )
        for card in first["theorem_cards"]:
            self.assertTrue(card["lean_name"])
            self.assertEqual(len(card["statement_sha256"]), 64)
            self.assertTrue(card["statement_text_paths"])
            self.assertTrue(card["natural_language_readback"])
        by_id = {card["theorem_id"]: card for card in first["theorem_cards"]}
        self.assertEqual(
            by_id["harp-l2-witness-cubature-input"]["authorship_status"],
            "harp-authored",
        )
        self.assertNotIn("harp-l2-witness-companion-commutation", by_id)
        scalar = by_id["harp-endpoint-scalar-contradiction"]
        self.assertEqual(
            scalar["claim_ceiling"],
            "source-faithful-local-certification",
        )
        self.assertTrue(scalar["source_locators"])
        card_ids = {card["theorem_id"] for card in first["theorem_cards"]}
        prerequisite_ids = {item["theorem_id"] for item in first["prerequisites"]}
        self.assertTrue(prerequisite_ids)
        for card in first["theorem_cards"]:
            self.assertLessEqual(
                set(card["dependencies"]), card_ids | prerequisite_ids
            )
        for prerequisite in first["prerequisites"]:
            self.assertIn(prerequisite["node_kind"], {"obligation", "route"})
            self.assertTrue(prerequisite["statement_sha256"])
            self.assertTrue(prerequisite["statement_text_paths"])

    def test_prove2me_candidate_requires_source_or_harp_authorship(self) -> None:
        payload = theorem_graph.build_proof_obligation_graph_json(REPO)
        node = next(
            item
            for item in payload["nodes"]
            if item["node_id"] == "harp-l2-witness-companion-commutation"
        )
        node["prove2me_candidate"] = True

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "invalid Prove2Me candidate"):
            theorem_graph.validate_proof_obligation_graph(payload, REPO)

    def test_cli_prints_obligation_graph_and_dry_run_export(self) -> None:
        script = REPO / "labs/crouzeix_proof_reproduction/proof_evidence.py"
        graph_result = subprocess.run(
            [sys.executable, str(script), "proof-obligation-graph"],
            cwd=REPO,
            check=False,
            capture_output=True,
            text=True,
        )
        export_result = subprocess.run(
            [sys.executable, str(script), "prove2me-export", "--dry-run"],
            cwd=REPO,
            check=False,
            capture_output=True,
            text=True,
        )

        self.assertEqual(graph_result.returncode, 0, graph_result.stderr)
        self.assertEqual(
            json.loads(graph_result.stdout)["schema_version"],
            "crouzeix-proof-obligation-graph/v2",
        )
        self.assertEqual(export_result.returncode, 0, export_result.stderr)
        export = json.loads(export_result.stdout)
        self.assertEqual(export["schema_version"], "crouzeix-prove2me-export/v2")
        self.assertFalse(export["network_access"])

    def test_readback_loader_rejects_symlinked_ledger(self) -> None:
        for dangling in (False, True):
            with self.subTest(dangling=dangling), tempfile.TemporaryDirectory() as directory:
                root = Path(directory)
                target = root / "readbacks-target.json"
                if not dangling:
                    target.write_text(
                        json.dumps(
                            {
                                "schema_version": "crouzeix-theorem-readbacks/v1",
                                "readbacks": [],
                            }
                        ),
                        encoding="utf-8",
                    )
                path = root / theorem_graph.READBACK_PATH
                path.parent.mkdir(parents=True)
                path.symlink_to(target)

                with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "symlink"):
                    theorem_graph._load_readbacks(root)

    def test_validate_accepts_builder_output(self) -> None:
        payload = theorem_graph.build_theorem_graph_json(REPO)

        graph = theorem_graph.validate_theorem_graph(payload)

        self.assertEqual(graph.schema_version, "crouzeix-theorem-graph/v1")

    def test_validation_rejects_noncanonical_or_unknown_node_routes(self) -> None:
        graph = theorem_graph.build_theorem_graph_json(REPO)
        shared = next(
            node
            for node in graph["nodes"]
            if node["route_ids"] == ["harp", "lorist-schwenninger"]
        )

        for route_ids, message in (
            (["lorist-schwenninger", "harp"], "graph node routes are not canonical"),
            (["not-a-route"], "unknown graph node route"),
        ):
            with self.subTest(route_ids=route_ids):
                payload = copy.deepcopy(graph)
                node = next(item for item in payload["nodes"] if item["node_id"] == shared["node_id"])
                node["route_ids"] = route_ids

                with self.assertRaisesRegex(theorem_graph.TheoremGraphError, message):
                    theorem_graph.validate_theorem_graph(payload)

    def test_harp_reuse_does_not_hide_lorist_schwenninger_claim_ceiling(self) -> None:
        graph = theorem_graph.build_theorem_graph(REPO)
        by_name = {node.lean_name: node for node in graph.nodes}

        scalar = by_name["CrouzeixConjecture.LoristSchwenninger.scalar_endpoint_le_two"]

        self.assertEqual(scalar.claim_ceiling, "source-faithful-local-certification")
        self.assertEqual(scalar.route_ids, ("harp", "lorist-schwenninger"))
        self.assertIn("harp:harp-reuse-scalar", scalar.route_node_ids)
        self.assertIn("lorist-schwenninger:ls-scalar-contradiction", scalar.route_node_ids)
        self.assertTrue(scalar.source_locators)

    def test_validation_rejects_dangling_frontier(self) -> None:
        payload = theorem_graph.build_theorem_graph_json(REPO)
        payload["frontier"] = copy.deepcopy(payload["frontier"])
        payload["frontier"]["open_nodes"] = ["missing-node"]

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "frontier"):
            theorem_graph.validate_theorem_graph(payload)

    def test_validation_rejects_malformed_frontier_bucket(self) -> None:
        payload = theorem_graph.build_theorem_graph_json(REPO)
        payload["frontier"] = copy.deepcopy(payload["frontier"])
        payload["frontier"]["open_nodes"] = "missing-node"

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "frontier node id"):
            theorem_graph.validate_theorem_graph(payload)

    def test_validation_rejects_cycles(self) -> None:
        payload = theorem_graph.build_theorem_graph_json(REPO)
        first = payload["nodes"][0]
        second = payload["nodes"][1]
        first["dependency_ids"] = [second["node_id"]]
        second["dependency_ids"] = [first["node_id"]]

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "cycle"):
            theorem_graph.validate_theorem_graph(payload)

    def test_validation_rejects_current_node_without_readback_statement(self) -> None:
        payload = theorem_graph.build_theorem_graph_json(REPO)
        node = next(
            item for item in payload["nodes"] if item["node_id"] == "harp-terminal-theorem"
        )
        node["natural_language_statement"] = None

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "current graph node"):
            theorem_graph.validate_theorem_graph(payload)

    def test_readback_binding_rejects_statement_hash_drift(self) -> None:
        graph = theorem_graph.build_theorem_graph(REPO)
        node = next(item for item in graph.nodes if item.node_id == "harp-terminal-theorem")
        readback = theorem_graph._Readback(
            node_id=node.node_id,
            lean_name=node.lean_name,
            statement_sha256="0" * 64,
            statement_text_paths=node.statement_text_paths,
            readback_status="current",
            natural_language_statement="A stale readback should not bind.",
        )

        with self.assertRaisesRegex(theorem_graph.TheoremGraphError, "statement hash"):
            theorem_graph._apply_readbacks((node,), {node.node_id: readback})


if __name__ == "__main__":
    unittest.main()
