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
import formal_receipt
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

        self.assertEqual(rows[0].row_id, "jin-max-polynomial-modulus")
        self.assertEqual(rows[0].status, "passed")
        self.assertEqual(
            rows[0].receipt_sha256,
            "ba66a41a1bef5a84977161cd5a8a568c95b0bee6d84d705fbd9cd63db3e823ba",
        )
        self.assertIsNone(rows[0].failed_reason)
        self.assertIsNone(rows[0].blocked_reason)
        self.assertEqual(rows[1].row_id, "jin-polynomial-bound")
        self.assertEqual(rows[1].status, "passed")
        self.assertEqual(
            rows[1].receipt_sha256,
            "35459464f1260cdfbde4af756e1dff4e3e37860f81bb56665726f285fb14f913",
        )
        self.assertIsNone(rows[1].failed_reason)
        self.assertIsNone(rows[1].blocked_reason)
        self.assertEqual(rows[-1].lean_name, "CrouzeixConjecture.crouzeixConjecture")
        self.assertEqual(rows[-1].status, "passed")
        self.assertEqual(
            rows[-1].receipt_sha256,
            "5702ceaac0ad5cb6f9456bfcefd37c642c0ed45038125d23c7f463c0cb01e4f3",
        )
        self.assertIsNone(rows[-1].failed_reason)
        self.assertIsNone(rows[-1].blocked_reason)
        self.assertIn("565b6a3e0659b6e0785f783b016c3f6d9f171fa5", rows[-1].source_locator)
        self.assertEqual(rows[-1].statement_sha256, target.target.statement_sha256)

    def test_source_map_accepts_optional_outcome_fields(self) -> None:
        target = formal_target.load_lock()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory).resolve() / "source-map.json"
            write_json(
                path,
                {
                    "schema_version": "crouzeix-jin-source-map/v1",
                    "source_commit": target.source.commit,
                    "rows": [
                        {
                            "row_id": "jin-first-slice",
                            "source_locator": (
                                f"git:{target.source.commit}:"
                                "Lean/CrouzeixConjecture/Statements.lean#L13-L18"
                            ),
                            "statement_sha256": "b" * 64,
                            "lean_name": "CrouzeixConjecture.firstSlice",
                            "dependency_ids": [],
                            "status": "failed",
                            "receipt_sha256": "a" * 64,
                            "failed_reason": "axiom audit output missing",
                        },
                        {
                            "row_id": "jin-terminal",
                            "source_locator": (
                                f"git:{target.source.commit}:"
                                "Lean/CrouzeixConjecture/FinalTheorems.lean#L15-L23"
                            ),
                            "statement_sha256": target.target.statement_sha256,
                            "lean_name": target.target.declaration_name,
                            "dependency_ids": [],
                            "status": "blocked",
                            "receipt_sha256": "c" * 64,
                            "blocked_reason": "missing executable: lake",
                        }
                    ],
                },
            )

            rows = jin_validation.load_source_map(path, target)

            self.assertEqual(rows[0].status, "failed")
            self.assertEqual(rows[0].receipt_sha256, "a" * 64)
            self.assertEqual(rows[0].failed_reason, "axiom audit output missing")
            self.assertIsNone(rows[0].blocked_reason)
            self.assertEqual(rows[1].status, "blocked")
            self.assertEqual(rows[1].receipt_sha256, "c" * 64)
            self.assertEqual(rows[1].blocked_reason, "missing executable: lake")
            self.assertIsNone(rows[1].failed_reason)

    def test_source_map_rejects_unknown_optional_outcome_fields(self) -> None:
        target = formal_target.load_lock()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory).resolve() / "source-map.json"
            write_json(
                path,
                {
                    "schema_version": "crouzeix-jin-source-map/v1",
                    "source_commit": target.source.commit,
                    "rows": [
                        {
                            "row_id": "jin-terminal",
                            "source_locator": (
                                f"git:{target.source.commit}:"
                                "Lean/CrouzeixConjecture/FinalTheorems.lean#L15-L23"
                            ),
                            "statement_sha256": target.target.statement_sha256,
                            "lean_name": target.target.declaration_name,
                            "dependency_ids": [],
                            "status": "passed",
                            "receipt_sha256": "a" * 64,
                            "unexpected_reason": "not allowed",
                        }
                    ],
                },
            )

            with self.assertRaisesRegex(protocol.ValidationError, "unknown"):
                jin_validation.load_source_map(path, target)

    def test_source_map_rejects_invalid_outcome_status(self) -> None:
        target = formal_target.load_lock()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory).resolve() / "source-map.json"
            write_json(
                path,
                {
                    "schema_version": "crouzeix-jin-source-map/v1",
                    "source_commit": target.source.commit,
                    "rows": [
                        {
                            "row_id": "jin-terminal",
                            "source_locator": (
                                f"git:{target.source.commit}:"
                                "Lean/CrouzeixConjecture/FinalTheorems.lean#L15-L23"
                            ),
                            "statement_sha256": target.target.statement_sha256,
                            "lean_name": target.target.declaration_name,
                            "dependency_ids": [],
                            "status": "unknown",
                        }
                    ],
                },
            )

            with self.assertRaisesRegex(protocol.ValidationError, "status"):
                jin_validation.load_source_map(path, target)

    def test_source_map_rejects_status_outcome_field_mismatches(self) -> None:
        target = formal_target.load_lock()
        cases = (
            (
                {"status": "mapped", "receipt_sha256": "a" * 64},
                "mapped.*receipt_sha256",
            ),
            (
                {"status": "mapped", "receipt_sha256": None},
                "mapped.*receipt_sha256",
            ),
            (
                {"status": "mapped", "blocked_reason": "blocked"},
                "mapped.*blocked_reason",
            ),
            (
                {"status": "mapped", "blocked_reason": None},
                "mapped.*blocked_reason",
            ),
            (
                {"status": "mapped", "failed_reason": "failed"},
                "mapped.*failed_reason",
            ),
            (
                {"status": "mapped", "failed_reason": None},
                "mapped.*failed_reason",
            ),
            ({"status": "blocked"}, "blocked.*receipt_sha256"),
            (
                {"status": "blocked", "receipt_sha256": "a" * 64},
                "blocked.*blocked_reason",
            ),
            (
                {
                    "status": "blocked",
                    "receipt_sha256": None,
                    "blocked_reason": "blocked",
                },
                "blocked.*receipt_sha256",
            ),
            (
                {
                    "status": "blocked",
                    "receipt_sha256": "a" * 64,
                    "blocked_reason": None,
                },
                "blocked.*blocked_reason",
            ),
            (
                {
                    "status": "blocked",
                    "receipt_sha256": "a" * 64,
                    "blocked_reason": "blocked",
                    "failed_reason": "failed",
                },
                "blocked.*failed_reason",
            ),
            (
                {
                    "status": "blocked",
                    "receipt_sha256": "a" * 64,
                    "blocked_reason": "blocked",
                    "failed_reason": None,
                },
                "blocked.*failed_reason",
            ),
            ({"status": "failed"}, "failed.*receipt_sha256"),
            (
                {"status": "failed", "receipt_sha256": "a" * 64},
                "failed.*failed_reason",
            ),
            (
                {
                    "status": "failed",
                    "receipt_sha256": None,
                    "failed_reason": "failed",
                },
                "failed.*receipt_sha256",
            ),
            (
                {
                    "status": "failed",
                    "receipt_sha256": "a" * 64,
                    "failed_reason": None,
                },
                "failed.*failed_reason",
            ),
            (
                {
                    "status": "failed",
                    "receipt_sha256": "a" * 64,
                    "failed_reason": "failed",
                    "blocked_reason": "blocked",
                },
                "failed.*blocked_reason",
            ),
            (
                {
                    "status": "failed",
                    "receipt_sha256": "a" * 64,
                    "failed_reason": "failed",
                    "blocked_reason": None,
                },
                "failed.*blocked_reason",
            ),
            ({"status": "passed"}, "passed.*receipt_sha256"),
            (
                {"status": "passed", "receipt_sha256": None},
                "passed.*receipt_sha256",
            ),
            (
                {
                    "status": "passed",
                    "receipt_sha256": "a" * 64,
                    "blocked_reason": "blocked",
                },
                "passed.*blocked_reason",
            ),
            (
                {
                    "status": "passed",
                    "receipt_sha256": "a" * 64,
                    "blocked_reason": None,
                },
                "passed.*blocked_reason",
            ),
            (
                {
                    "status": "passed",
                    "receipt_sha256": "a" * 64,
                    "failed_reason": "failed",
                },
                "passed.*failed_reason",
            ),
            (
                {
                    "status": "passed",
                    "receipt_sha256": "a" * 64,
                    "failed_reason": None,
                },
                "passed.*failed_reason",
            ),
        )
        for overrides, pattern in cases:
            with self.subTest(overrides=overrides):
                with tempfile.TemporaryDirectory() as directory:
                    path = Path(directory).resolve() / "source-map.json"
                    row = {
                        "row_id": "jin-terminal",
                        "source_locator": (
                            f"git:{target.source.commit}:"
                            "Lean/CrouzeixConjecture/FinalTheorems.lean#L15-L23"
                        ),
                        "statement_sha256": target.target.statement_sha256,
                        "lean_name": target.target.declaration_name,
                        "dependency_ids": [],
                    }
                    row.update(overrides)
                    write_json(
                        path,
                        {
                            "schema_version": "crouzeix-jin-source-map/v1",
                            "source_commit": target.source.commit,
                            "rows": [row],
                        },
                    )

                    with self.assertRaisesRegex(protocol.ValidationError, pattern):
                        jin_validation.load_source_map(path, target)

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
        self.assertEqual(task.imports, ("CrouzeixConjecture.FinalTheorems",))

        with tempfile.TemporaryDirectory() as directory:
            bad_target = Path(directory).resolve() / "Target.lean"
            bad_target.write_text(
                "import CrouzeixConjecture.FinalTheorems\n"
                "import CrouzeixConjecture.PrivateScratch\n",
                encoding="utf-8",
            )
            with self.assertRaisesRegex(protocol.ValidationError, "allowlist"):
                jin_validation.make_alignment_task(target, rows, bad_target)

    def make_resolved_target(self, root: Path) -> tuple[formal_target.FormalTargetLock, formal_target.ResolvedTarget]:
        source = root / "input" / "Target.lean"
        source.parent.mkdir(parents=True)
        source.write_bytes(b"target bytes\n")
        target_root = root / "target"
        (target_root / "artifacts").mkdir(parents=True)
        (target_root / "ledger").mkdir()
        (target_root / "artifacts" / "Target.lean").write_bytes(b"target bytes\n")
        lock_value = {
            **{
                "schema_version": "crouzeix-formal-target-lock/v1",
                "source": {
                    "source_id": "JIN-V4-AUDITED",
                    "commit": "565b6a3e0659b6e0785f783b016c3f6d9f171fa5",
                    "tree": "40aafa503bd32762dbf6d1a67ddef3e2b067f0e1",
                    "archive_sha256": "33ee5b75c1037866c4d2bda8eff872cc0640fd42e7c31c58a8e6047405f69542",
                },
                "toolchain": {
                    "lean": "leanprover/lean4:v4.28.0",
                    "mathlib_revision": "8f9d9cff6bd728b17a24e163c9402775d9e6a365",
                },
                "command": {"argv": ["lake", "build"], "cwd": "Lean", "env": {}},
                "target": {
                    "target_id": "crouzeix-main",
                    "declaration_name": "CrouzeixConjecture.crouzeixConjecture",
                    "statement_sha256": "1a2e841ea3af7c41ca815a982e20710e04ca17242aa510b70cbfadbcd17c2bb4",
                    "source_locator": "Harp-authored",
                    "dependency_ids": [],
                },
                "artifacts": [
                    {
                        "artifact_id": "target-lean",
                        "path": "artifacts/Target.lean",
                        "bytes": len(b"target bytes\n"),
                        "sha256": protocol.sha256_bytes(b"target bytes\n"),
                    }
                ],
                "ledger_path": "ledger",
                "import_allowlist": ["CrouzeixConjecture.FinalTheorems"],
            }
        }
        lock = formal_target.FormalTargetLock.from_mapping(lock_value)
        resolved = formal_target.provision(lock, {"target-lean": source}, root / "runtime")
        return lock, resolved

    def test_run_jin_validation_writes_single_create_only_v2_receipt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            lock, resolved = self.make_resolved_target(root)
            task = jin_validation.make_alignment_task(
                lock,
                jin_validation.load_source_map(SOURCE_MAP, formal_target.load_lock()),
                TARGET_LEAN,
            )

            attempt = jin_validation.run_jin_validation(
                resolved,
                task,
                root / "attempt",
                fake_result=jin_validation.FakeLeanResult(
                    exit_code=0,
                    stdout=b"compiled\n",
                    stderr=b"",
                    axioms=[],
                ),
            )

            receipt = formal_receipt.validate_attempt_dir(attempt)
            self.assertEqual(receipt["schema_version"], "crouzeix-formal-attempt-receipt/v2")
            self.assertEqual(receipt["status"], "passed")
            self.assertEqual(receipt["target_type_sha256"], task.target_type_sha256)
            with self.assertRaisesRegex(protocol.ValidationError, "already exists"):
                jin_validation.run_jin_validation(
                    resolved,
                    task,
                    attempt,
                    fake_result=jin_validation.FakeLeanResult(0, b"compiled\n", b"", []),
                )

    def test_run_jin_validation_classifies_failed_blocked_inventory_and_forbidden_axiom(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            lock, resolved = self.make_resolved_target(root)
            task = jin_validation.make_alignment_task(
                lock,
                jin_validation.load_source_map(SOURCE_MAP, formal_target.load_lock()),
                TARGET_LEAN,
            )

            failed_attempt = jin_validation.run_jin_validation(
                resolved,
                task,
                root / "failed",
                fake_result=jin_validation.FakeLeanResult(
                    exit_code=1,
                    stdout=b"",
                    stderr=b"type mismatch\n",
                    axioms=[],
                ),
            )
            self.assertEqual(formal_receipt.validate_attempt_dir(failed_attempt)["status"], "failed")

            blocked_attempt = jin_validation.run_jin_validation(
                resolved,
                task,
                root / "blocked",
                fake_result=jin_validation.FakeLeanResult(
                    exit_code=None,
                    stdout=b"",
                    stderr=b"missing toolchain\n",
                    axioms=[],
                    blocked_reason="missing-pinned-toolchain",
                ),
            )
            self.assertEqual(formal_receipt.validate_attempt_dir(blocked_attempt)["status"], "blocked")

            with self.assertRaisesRegex(protocol.ValidationError, "inventory"):
                jin_validation.run_jin_validation(
                    formal_target.ResolvedTarget(
                        root=resolved.root,
                        inventory_sha256="0" * 64,
                        command=resolved.command,
                        artifacts=resolved.artifacts,
                    ),
                    task,
                    root / "stale",
                    fake_result=jin_validation.FakeLeanResult(0, b"compiled\n", b"", []),
                )

            with self.assertRaisesRegex(protocol.ValidationError, "disallowed axioms"):
                jin_validation.run_jin_validation(
                    resolved,
                    task,
                    root / "axiom",
                    fake_result=jin_validation.FakeLeanResult(
                        exit_code=0,
                        stdout=b"compiled\n",
                        stderr=b"",
                        axioms=["Classical.choice"],
                    ),
                )

    def test_production_jin_validation_is_blocked_by_preflight(self) -> None:
        target = formal_target.load_lock()
        rows = jin_validation.load_source_map(SOURCE_MAP, target)
        task = jin_validation.make_alignment_task(target, rows, TARGET_LEAN)
        preflight = formal_target.load_preflight_receipt(
            LAB / "formal_targets/jin-565b6a3/preflight.json"
        )
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            attempt = jin_validation.record_blocked_preflight_attempt(
                target,
                task,
                root / "blocked-production",
                preflight,
            )
            receipt = formal_receipt.validate_attempt_dir(attempt)
            self.assertEqual(receipt["schema_version"], "crouzeix-formal-attempt-receipt/v2")
            self.assertEqual(receipt["status"], "blocked")
            self.assertIn("insufficient-disk", receipt["reason"])

    def test_rebuild_review_requires_clean_digest_for_proof_claims(self) -> None:
        review = jin_validation.load_rebuild_review(
            LAB / "formal_targets/jin-565b6a3/rebuild-review.json"
        )

        self.assertEqual(review["status"], "blocked")
        self.assertEqual(review["claim"], "no-formal-proof-claim")
        self.assertEqual(review["jin_validation_commit"], "493d922")

        bad = dict(review)
        bad["claim"] = "formal-proof-claim"
        bad["clean_rebuild_sha256"] = None
        with self.assertRaisesRegex(protocol.ValidationError, "clean_rebuild_sha256"):
            jin_validation.validate_rebuild_review(bad)


if __name__ == "__main__":
    unittest.main()
