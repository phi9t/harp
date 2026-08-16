from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
REPO = LAB.parents[1]
sys.path.insert(0, str(LAB))

import formal_target
import jin_validation
import protocol


SOURCE_MAP = LAB / "formal_targets/jin-565b6a3/source-map.json"
TARGET_LEAN = LAB / "formal_targets/jin-565b6a3/Target.lean"


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


class JinValidationTests(unittest.TestCase):
    def test_source_map_covers_terminal_theorem_and_pinned_locator(self) -> None:
        target = formal_target.load_lock()
        rows = jin_validation.load_source_map(SOURCE_MAP, target)

        self.assertEqual(rows[-1].lean_name, "CrouzeixConjecture.crouzeixConjecture")
        self.assertEqual(rows[-1].status, "mapped")
        self.assertIn("565b6a3e0659b6e0785f783b016c3f6d9f171fa5", rows[-1].source_locator)
        self.assertEqual(rows[-1].statement_sha256, target.target.statement_sha256)

    def test_source_map_rejects_duplicates_wrong_revision_and_missing_terminal(self) -> None:
        target = formal_target.load_lock()
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            base = {
                "schema_version": "crouzeix-jin-source-map/v1",
                "source_commit": target.source.commit,
                "rows": [
                    {
                        "row_id": "jin-terminal",
                        "source_locator": f"git:{target.source.commit}:Lean/CrouzeixConjecture/FinalTheorems.lean#L15-L23",
                        "statement_sha256": target.target.statement_sha256,
                        "lean_name": "CrouzeixConjecture.crouzeixConjecture",
                        "dependency_ids": [],
                        "status": "mapped",
                    }
                ],
            }

            duplicate = root / "duplicate.json"
            write_json(duplicate, base | {"rows": base["rows"] + base["rows"]})
            with self.assertRaisesRegex(protocol.ValidationError, "duplicate"):
                jin_validation.load_source_map(duplicate, target)

            wrong_revision = root / "wrong-revision.json"
            wrong = dict(base)
            wrong["rows"] = [
                dict(
                    base["rows"][0],
                    source_locator="git:9df07838327b988e3924453daa29c8cd726d34b0:Lean/CrouzeixConjecture/FinalTheorems.lean#L15-L23",
                )
            ]
            write_json(wrong_revision, wrong)
            with self.assertRaisesRegex(protocol.ValidationError, "pinned"):
                jin_validation.load_source_map(wrong_revision, target)

            missing_terminal = root / "missing-terminal.json"
            write_json(
                missing_terminal,
                base | {"rows": [dict(base["rows"][0], lean_name="CrouzeixConjecture.other")]},
            )
            with self.assertRaisesRegex(protocol.ValidationError, "terminal"):
                jin_validation.load_source_map(missing_terminal, target)

    def test_target_alignment_rejects_imports_outside_allowlist_and_builds_task(self) -> None:
        target = formal_target.load_lock()
        rows = jin_validation.load_source_map(SOURCE_MAP, target)

        task = jin_validation.make_alignment_task(target, rows, TARGET_LEAN)

        self.assertEqual(task.task_id, "jin-target-alignment")
        self.assertEqual(task.expected_declaration, target.target.declaration_name)
        self.assertEqual(task.target_type_sha256, target.target.statement_sha256)
        self.assertEqual(task.imports, tuple(target.import_allowlist))

        with tempfile.TemporaryDirectory() as directory:
            bad_target = Path(directory).resolve() / "Target.lean"
            bad_target.write_text(
                "import CrouzeixConjecture.FinalTheorems\n"
                "import CrouzeixConjecture.PrivateScratch\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(protocol.ValidationError, "allowlist"):
                jin_validation.make_alignment_task(target, rows, bad_target)


if __name__ == "__main__":
    unittest.main()
