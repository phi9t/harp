from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import kernel_validation
import protocol


ADMISSIONS = LAB / "formal_targets/shared-kernel/admissions.json"


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


class KernelValidationTests(unittest.TestCase):
    def test_empty_kernel_admissions_are_valid_until_two_consumers_exist(self) -> None:
        admissions = kernel_validation.load_admissions(ADMISSIONS)

        self.assertEqual(admissions["status"], "no-admitted-kernel-lemmas")
        self.assertEqual(admissions["admissions"], [])

    def test_kernel_admission_rejects_one_consumer_and_terminal_dependency(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            one_consumer = root / "one.json"
            write_json(
                one_consumer,
                {
                    "schema_version": "crouzeix-shared-kernel-admissions/v1",
                    "status": "admissions-present",
                    "admissions": [
                        {
                            "admission_id": "kernel-bridge",
                            "statement_sha256": "a" * 64,
                            "consumer_ids": ["jin-terminal"],
                            "allowed_imports": ["Mathlib"],
                            "prohibited_dependencies": [],
                            "status": "admitted",
                        }
                    ],
                },
            )
            with self.assertRaisesRegex(protocol.ValidationError, "two consumers"):
                kernel_validation.load_admissions(one_consumer)

            terminal = root / "terminal.json"
            write_json(
                terminal,
                {
                    "schema_version": "crouzeix-shared-kernel-admissions/v1",
                    "status": "admissions-present",
                    "admissions": [
                        {
                            "admission_id": "kernel-bridge",
                            "statement_sha256": "a" * 64,
                            "consumer_ids": ["jin-node", "ls-node"],
                            "allowed_imports": ["Mathlib"],
                            "prohibited_dependencies": ["terminal-crouzeix-theorem"],
                            "status": "admitted",
                        }
                    ],
                },
            )
            with self.assertRaisesRegex(protocol.ValidationError, "terminal"):
                kernel_validation.load_admissions(terminal)


if __name__ == "__main__":
    unittest.main()
