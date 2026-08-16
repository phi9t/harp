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
        graph = ls_validation.load_source_graph(GRAPH)

        self.assertEqual(graph[-1].node_id, "ls-terminal-crouzeix")
        self.assertEqual(graph[-1].role, "terminal")
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
                    }
                ],
            }

            duplicate = root / "duplicate.json"
            write_json(duplicate, base | {"nodes": base["nodes"] + base["nodes"]})
            with self.assertRaisesRegex(protocol.ValidationError, "duplicate"):
                ls_validation.load_source_graph(duplicate)

            unknown = root / "unknown.json"
            unknown_node = dict(base["nodes"][0], dependencies=["missing-node"])
            write_json(unknown, base | {"nodes": [unknown_node]})
            with self.assertRaisesRegex(protocol.ValidationError, "dependency"):
                ls_validation.load_source_graph(unknown)

            cycle = root / "cycle.json"
            node_a = dict(base["nodes"][0], node_id="a", lean_name="LS.a", dependencies=["b"], role="intermediate")
            node_b = dict(base["nodes"][0], node_id="b", lean_name="LS.b", dependencies=["a"], role="terminal")
            write_json(cycle, base | {"nodes": [node_a, node_b]})
            with self.assertRaisesRegex(protocol.ValidationError, "cycle"):
                ls_validation.load_source_graph(cycle)

            jin = root / "jin.json"
            jin_node = dict(
                base["nodes"][0],
                source_locator="git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/private.lean",
            )
            write_json(jin, base | {"nodes": [jin_node]})
            with self.assertRaisesRegex(protocol.ValidationError, "Jin"):
                ls_validation.load_source_graph(jin)

    def test_library_inventory_maps_external_facts_to_mathlib_task_or_blocker(self) -> None:
        inventory = ls_validation.load_library_inventory(INVENTORY)

        self.assertEqual(inventory["source_identity"], "arxiv:2608.03841v1")
        self.assertEqual(
            {item["resolution"] for item in inventory["facts"]},
            {"local_task", "blocked"},
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


if __name__ == "__main__":
    unittest.main()
