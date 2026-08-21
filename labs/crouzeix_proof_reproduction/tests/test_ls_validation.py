from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import ls_validation
import protocol


GRAPH = LAB / "formal_targets/lorist-schwenninger/source-graph.json"
INVENTORY = LAB / "formal_targets/lorist-schwenninger/library-inventory.json"


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


class LSValidationTests(unittest.TestCase):
    def test_ls_graph_has_terminal_node_and_no_jin_private_paths(self) -> None:
        graph = ls_validation.load_route_graph(GRAPH)

        self.assertEqual(graph[-1].node_id, "ls-terminal-crouzeix")
        self.assertEqual(graph[-1].role, "terminal")
        self.assertEqual(graph[0].node_id, "ls-equation-one-terminal-bound")
        self.assertEqual(graph[0].status, "passed")
        self.assertIsNotNone(graph[0].receipt_sha256)
        self.assertEqual(graph[1].status, "blocked")
        self.assertIsNotNone(graph[1].blocked_reason)
        self.assertTrue(all("565b6a3" not in row.source_locator for row in graph))
        self.assertTrue(all("JIN" not in row.source_locator for row in graph))

    def test_ls_graph_rejects_duplicate_unknown_cycle_and_jin_leakage(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            base = {
                "schema_version": "crouzeix-ls-source-graph/v1",
                "source_id": "LS-ARXIV-V1",
                "source_identity": "arxiv:2608.03841v1",
                "nodes": [
                    {
                        "node_id": "ls-terminal-crouzeix",
                        "source_locator": "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128",
                        "statement_sha256": "a" * 64,
                        "lean_name": "LS.CrouzeixTerminal",
                        "dependencies": [],
                        "role": "terminal",
                        "status": "blocked",
                        "blocked_reason": "fixture blocker",
                    }
                ],
            }

            duplicate = root / "duplicate.json"
            write_json(duplicate, base | {"nodes": base["nodes"] + base["nodes"]})
            with self.assertRaisesRegex(protocol.ValidationError, "duplicate"):
                ls_validation.load_route_graph(duplicate)

            unknown = root / "unknown.json"
            unknown_node = dict(base["nodes"][0], dependencies=["missing-node"])
            write_json(unknown, base | {"nodes": [unknown_node]})
            with self.assertRaisesRegex(protocol.ValidationError, "dependency"):
                ls_validation.load_route_graph(unknown)

            cycle = root / "cycle.json"
            node_a = dict(base["nodes"][0], node_id="a", lean_name="LS.a", dependencies=["b"], role="intermediate")
            node_b = dict(base["nodes"][0], node_id="b", lean_name="LS.b", dependencies=["a"], role="terminal")
            write_json(cycle, base | {"nodes": [node_a, node_b]})
            with self.assertRaisesRegex(protocol.ValidationError, "cycle"):
                ls_validation.load_route_graph(cycle)

            jin = root / "jin.json"
            jin_node = dict(
                base["nodes"][0],
                source_locator="git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/private.lean",
            )
            write_json(jin, base | {"nodes": [jin_node]})
            with self.assertRaisesRegex(protocol.ValidationError, "Jin"):
                ls_validation.load_route_graph(jin)

    def test_library_inventory_maps_external_facts_to_mathlib_task_or_blocker(self) -> None:
        inventory = ls_validation.load_library_inventory(INVENTORY)

        self.assertEqual(inventory["source_identity"], "arxiv:2608.03841v1")
        self.assertEqual(
            {item["resolution"] for item in inventory["facts"]},
            {"local_compiled", "local_task", "blocked"},
        )

        with tempfile.TemporaryDirectory() as directory:
            bad = Path(directory).resolve() / "inventory.json"
            write_json(
                bad,
                {
                    "schema_version": "crouzeix-ls-library-inventory/v1",
                    "source_identity": "arxiv:2608.03841v1",
                    "facts": [
                        {
                            "fact_id": "bad",
                            "statement_sha256": "b" * 64,
                            "source_locator": "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L1",
                            "resolution": "Jin private file",
                        }
                    ],
                },
            )
            with self.assertRaisesRegex(protocol.ValidationError, "resolution"):
                ls_validation.load_library_inventory(bad)

    def test_materialize_ls_tasks_records_blocked_nodes_and_terminal_assembly(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            graph = ls_validation.load_route_graph(GRAPH)

            summary = ls_validation.materialize_tasks(graph, root)

            self.assertEqual(summary["status"], "blocked")
            self.assertEqual(summary["terminal_node_id"], "ls-terminal-crouzeix")
            self.assertTrue((root / "ls-equation-one-terminal-bound" / "task.json").is_file())
            first_result = json.loads(
                (root / "ls-equation-one-terminal-bound" / "result.json").read_text()
            )
            self.assertEqual(first_result["status"], "passed")
            self.assertTrue((root / "ls-perturbation-lemma" / "task.json").is_file())
            self.assertTrue((root / "assembly" / "result.json").is_file())
            terminal_result = json.loads((root / "assembly" / "result.json").read_text())
            self.assertEqual(terminal_result["status"], "blocked")
            self.assertIn("ls-perturbation-lemma", terminal_result["blocked_by"])

    def test_materialize_ls_tasks_rejects_unpublished_predecessor_and_jin_imports(self) -> None:
        graph = ls_validation.load_route_graph(GRAPH)
        tampered = list(graph)
        tampered[1] = ls_validation.LSGraphRow(
            node_id=tampered[1].node_id,
            source_locator=tampered[1].source_locator,
            statement_sha256=tampered[1].statement_sha256,
            lean_name=tampered[1].lean_name,
            dependencies=("not-published",),
            role=tampered[1].role,
            status=tampered[1].status,
        )
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(protocol.ValidationError, "predecessor"):
                ls_validation.materialize_tasks(tuple(tampered), Path(directory).resolve())

        leaky = list(graph)
        leaky[0] = ls_validation.LSGraphRow(
            node_id=leaky[0].node_id,
            source_locator=leaky[0].source_locator,
            statement_sha256=leaky[0].statement_sha256,
            lean_name="JIN.PrivateLeak",
            dependencies=leaky[0].dependencies,
            role=leaky[0].role,
            status=leaky[0].status,
        )
        with tempfile.TemporaryDirectory() as directory:
            with self.assertRaisesRegex(protocol.ValidationError, "Jin"):
                ls_validation.materialize_tasks(tuple(leaky), Path(directory).resolve())

    def test_ls_graph_rejects_status_outcome_field_mismatches(self) -> None:
        cases = (
            (
                {"status": "mapped", "receipt_sha256": "a" * 64},
                "mapped.*receipt_sha256",
            ),
            (
                {"status": "mapped", "blocked_reason": "blocked"},
                "mapped.*blocked_reason",
            ),
            ({"status": "blocked"}, "blocked.*blocked_reason"),
            (
                {
                    "status": "blocked",
                    "blocked_reason": "blocked",
                    "failed_reason": "failed",
                },
                "blocked.*failed_reason",
            ),
            ({"status": "failed"}, "failed.*receipt_sha256"),
            (
                {"status": "failed", "receipt_sha256": "a" * 64},
                "failed.*failed_reason",
            ),
            ({"status": "passed"}, "passed.*receipt_sha256"),
            (
                {
                    "status": "passed",
                    "receipt_sha256": "a" * 64,
                    "blocked_reason": "blocked",
                },
                "passed.*blocked_reason",
            ),
        )
        for overrides, pattern in cases:
            with self.subTest(overrides=overrides):
                with tempfile.TemporaryDirectory() as directory:
                    path = Path(directory).resolve() / "source-graph.json"
                    node = {
                        "node_id": "ls-terminal-crouzeix",
                        "source_locator": "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L125-L128",
                        "statement_sha256": "a" * 64,
                        "lean_name": "LS.CrouzeixTerminal",
                        "dependencies": [],
                        "role": "terminal",
                    }
                    node.update(overrides)
                    write_json(
                        path,
                        {
                            "schema_version": "crouzeix-ls-source-graph/v1",
                            "source_id": "LS-ARXIV-V1",
                            "source_identity": "arxiv:2608.03841v1",
                            "nodes": [node],
                        },
                    )

                    with self.assertRaisesRegex(protocol.ValidationError, pattern):
                        ls_validation.load_route_graph(path)


if __name__ == "__main__":
    unittest.main()
