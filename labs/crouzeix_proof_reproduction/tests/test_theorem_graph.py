from __future__ import annotations

import copy
import unittest
from pathlib import Path

from labs.crouzeix_proof_reproduction import theorem_graph


REPO = Path(__file__).resolve().parents[3]


class TheoremGraphTests(unittest.TestCase):
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

    def test_validate_accepts_builder_output(self) -> None:
        payload = theorem_graph.build_theorem_graph_json(REPO)

        graph = theorem_graph.validate_theorem_graph(payload)

        self.assertEqual(graph.schema_version, "crouzeix-theorem-graph/v1")

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
