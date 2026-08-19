from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import formal_target
import jin_validation
import proof_slice
import protocol


SOURCE_MAP = LAB / "formal_targets/jin-565b6a3/source-map.json"


def descriptor(row_id: str = "jin-max-polynomial-modulus") -> dict[str, object]:
    return {
        "schema_version": "crouzeix-jin-proof-slice-descriptor/v1",
        "route_id": "jin",
        "source_map_row_id": row_id,
        "pinned_source_locator": (
            "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:"
            "Lean/CrouzeixConjecture/Statements.lean#L13-L18"
        ),
        "informal_statement_sha256": (
            "49bb4a5ac09489c9c7642215474b2f099f69d1e6a3e431f7a9dfd2b95ed736d0"
        ),
        "expected_lean_declaration": (
            "CrouzeixConjecture.maxPolynomialModulusOnNumericalRange"
        ),
        "allowed_imports": [],
        "dependency_receipts": [],
        "output_declaration_name": (
            "CrouzeixConjecture.maxPolynomialModulusOnNumericalRange"
        ),
        "build_target": "module/Slice.lean",
        "proof_hole_policy": {"reject_tokens": ["sorry", "admit"]},
        "axiom_policy": {"allowed_axioms": []},
        "attempt_budget": {"timeout_seconds": 3600, "max_output_bytes": 1048576},
    }


def dependent_descriptor() -> dict[str, object]:
    value = descriptor("jin-polynomial-bound")
    value.update(
        {
            "pinned_source_locator": (
                "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:"
                "Lean/CrouzeixConjecture/Statements.lean#L21-L22"
            ),
            "informal_statement_sha256": (
                "4bddb24fd47159593c2223a973569e56d99873f754fa638b75b1f55a93e7167b"
            ),
            "expected_lean_declaration": (
                "CrouzeixConjecture.PolynomialCrouzeixBound"
            ),
            "dependency_receipts": [
                {
                    "row_id": "jin-max-polynomial-modulus",
                    "receipt_sha256": "a" * 64,
                }
            ],
            "output_declaration_name": "CrouzeixConjecture.PolynomialCrouzeixBound",
        }
    )
    return value


def rows() -> tuple[jin_validation.SourceMapRow, ...]:
    return jin_validation.load_source_map(SOURCE_MAP, formal_target.load_lock())


class ProofSliceDescriptorTests(unittest.TestCase):
    def test_selects_first_nonterminal_jin_row(self) -> None:
        target = formal_target.load_lock()

        row = proof_slice.select_first_jin_slice(rows(), target)

        self.assertEqual(row.row_id, "jin-max-polynomial-modulus")
        self.assertNotEqual(row.lean_name, target.target.declaration_name)

    def test_descriptor_accepts_first_nonterminal_jin_row(self) -> None:
        desc = proof_slice.validate_descriptor(
            descriptor(), formal_target.load_lock(), rows()
        )

        self.assertEqual(desc.schema_version, "crouzeix-jin-proof-slice-descriptor/v1")
        self.assertEqual(desc.route_id, "jin")
        self.assertEqual(desc.source_map_row_id, "jin-max-polynomial-modulus")
        self.assertEqual(desc.allowed_imports, ())
        self.assertEqual(desc.dependency_receipts, ())

    def test_descriptor_rejects_unknown_fields(self) -> None:
        with self.assertRaisesRegex(protocol.ValidationError, "unknown"):
            proof_slice.validate_descriptor(
                descriptor() | {"extra": True}, formal_target.load_lock(), rows()
            )

    def test_descriptor_rejects_terminal_row_for_first_slice(self) -> None:
        value = descriptor("jin-terminal-crouzeix")
        value.update(
            {
                "pinned_source_locator": (
                    "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:"
                    "Lean/CrouzeixConjecture/FinalTheorems.lean#L15-L23"
                ),
                "informal_statement_sha256": (
                    "1a2e841ea3af7c41ca815a982e20710e04ca17242aa510b70cbfadbcd17c2bb4"
                ),
                "expected_lean_declaration": (
                    "CrouzeixConjecture.crouzeixConjecture"
                ),
                "dependency_receipts": [
                    {"row_id": "jin-polynomial-bound", "receipt_sha256": "a" * 64}
                ],
                "output_declaration_name": "CrouzeixConjecture.crouzeixConjecture",
            }
        )

        with self.assertRaisesRegex(protocol.ValidationError, "terminal"):
            proof_slice.validate_descriptor(value, formal_target.load_lock(), rows())

    def test_descriptor_rejects_imports_outside_formal_target_allowlist(self) -> None:
        value = descriptor()
        value["allowed_imports"] = ["CrouzeixConjecture.PrivateScratch"]

        with self.assertRaisesRegex(protocol.ValidationError, "allowlist|import"):
            proof_slice.validate_descriptor(value, formal_target.load_lock(), rows())

    def test_descriptor_rejects_terminal_reference_import_for_first_slice(self) -> None:
        value = descriptor()
        value["allowed_imports"] = ["CrouzeixConjecture.FinalTheorems"]

        with self.assertRaisesRegex(protocol.ValidationError, "terminal|reference|import"):
            proof_slice.validate_descriptor(value, formal_target.load_lock(), rows())

    def test_descriptor_rejects_declaration_mismatches(self) -> None:
        for field in ("expected_lean_declaration", "output_declaration_name"):
            with self.subTest(field=field):
                value = descriptor()
                value[field] = "CrouzeixConjecture.other"
                with self.assertRaisesRegex(protocol.ValidationError, "declaration"):
                    proof_slice.validate_descriptor(value, formal_target.load_lock(), rows())

    def test_descriptor_rejects_rows_with_dependencies_without_receipts(self) -> None:
        value = dependent_descriptor()
        value["dependency_receipts"] = []

        with self.assertRaisesRegex(protocol.ValidationError, "dependency"):
            proof_slice.validate_descriptor(value, formal_target.load_lock(), rows())

    def test_descriptor_accepts_exact_dependency_receipt_bindings(self) -> None:
        desc = proof_slice.validate_descriptor(
            dependent_descriptor(), formal_target.load_lock(), rows()
        )

        self.assertEqual(
            desc.dependency_receipts,
            (
                proof_slice.DependencyReceipt(
                    row_id="jin-max-polynomial-modulus",
                    receipt_sha256="a" * 64,
                ),
            ),
        )

    def test_descriptor_rejects_bad_dependency_receipt_bindings(self) -> None:
        cases = (
            ([], "missing"),
            (
                [{"row_id": "jin-max-polynomial-modulus", "receipt_sha256": "b" * 63}],
                "SHA-256",
            ),
            (
                [{"row_id": "jin-terminal-crouzeix", "receipt_sha256": "b" * 64}],
                "missing|extra|dependency",
            ),
            (
                [
                    {
                        "row_id": "jin-max-polynomial-modulus",
                        "receipt_sha256": "b" * 64,
                    },
                    {
                        "row_id": "jin-terminal-crouzeix",
                        "receipt_sha256": "c" * 64,
                    },
                ],
                "extra|dependency",
            ),
            (
                [
                    {
                        "row_id": "jin-max-polynomial-modulus",
                        "receipt_sha256": "b" * 64,
                    },
                    {
                        "row_id": "jin-max-polynomial-modulus",
                        "receipt_sha256": "c" * 64,
                    },
                ],
                "duplicate",
            ),
            (
                [
                    {
                        "row_id": "jin-max-polynomial-modulus",
                        "receipt_sha256": "b" * 64,
                        "extra": True,
                    }
                ],
                "unknown",
            ),
        )
        for dependency_receipts, pattern in cases:
            with self.subTest(dependency_receipts=dependency_receipts):
                value = dependent_descriptor()
                value["dependency_receipts"] = dependency_receipts
                with self.assertRaisesRegex(protocol.ValidationError, pattern):
                    proof_slice.validate_descriptor(value, formal_target.load_lock(), rows())

    def test_descriptor_rejects_duplicate_dependency_receipt_sha256_values(self) -> None:
        target = formal_target.load_lock()
        synthetic_rows = (
            jin_validation.SourceMapRow(
                row_id="jin-first",
                source_locator=(
                    "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:"
                    "Lean/CrouzeixConjecture/Statements.lean#L1-L2"
                ),
                statement_sha256="1" * 64,
                lean_name="CrouzeixConjecture.first",
                dependency_ids=(),
                status="mapped",
            ),
            jin_validation.SourceMapRow(
                row_id="jin-second",
                source_locator=(
                    "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:"
                    "Lean/CrouzeixConjecture/Statements.lean#L3-L4"
                ),
                statement_sha256="2" * 64,
                lean_name="CrouzeixConjecture.second",
                dependency_ids=(),
                status="mapped",
            ),
            jin_validation.SourceMapRow(
                row_id="jin-combined",
                source_locator=(
                    "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:"
                    "Lean/CrouzeixConjecture/Statements.lean#L5-L6"
                ),
                statement_sha256="3" * 64,
                lean_name="CrouzeixConjecture.combined",
                dependency_ids=("jin-first", "jin-second"),
                status="mapped",
            ),
        )
        value = descriptor("jin-combined")
        value.update(
            {
                "pinned_source_locator": synthetic_rows[-1].source_locator,
                "informal_statement_sha256": synthetic_rows[-1].statement_sha256,
                "expected_lean_declaration": synthetic_rows[-1].lean_name,
                "output_declaration_name": synthetic_rows[-1].lean_name,
                "dependency_receipts": [
                    {"row_id": "jin-first", "receipt_sha256": "a" * 64},
                    {"row_id": "jin-second", "receipt_sha256": "a" * 64},
                ],
            }
        )

        with self.assertRaisesRegex(protocol.ValidationError, "duplicate"):
            proof_slice.validate_descriptor(value, target, synthetic_rows)

    def test_descriptor_rejects_bad_policy_shapes(self) -> None:
        cases = (
            ("proof_hole_policy", {"reject_tokens": ["sorry", "admit"], "extra": True}, "unknown"),
            ("proof_hole_policy", {"reject_tokens": ["sorry"]}, "admit"),
            ("proof_hole_policy", {"reject_tokens": ["admit"]}, "sorry"),
            ("axiom_policy", {"allowed_axioms": [], "extra": True}, "unknown"),
            ("axiom_policy", {"allowed_axioms": ["propext", "propext"]}, "duplicate"),
            (
                "attempt_budget",
                {
                    "timeout_seconds": 3600,
                    "max_output_bytes": 1048576,
                    "extra": True,
                },
                "unknown",
            ),
            (
                "attempt_budget",
                {"timeout_seconds": True, "max_output_bytes": 1048576},
                "integer",
            ),
        )
        for field, replacement, pattern in cases:
            with self.subTest(field=field, replacement=json.dumps(replacement)):
                value = descriptor()
                value[field] = replacement
                with self.assertRaisesRegex(protocol.ValidationError, pattern):
                    proof_slice.validate_descriptor(value, formal_target.load_lock(), rows())

    def test_descriptor_rejects_unsafe_or_nonstandard_build_target(self) -> None:
        for build_target, pattern in (
            ("../Slice.lean", "relative"),
            ("module/Other.lean", "module/Slice.lean"),
        ):
            with self.subTest(build_target=build_target):
                value = descriptor()
                value["build_target"] = build_target
                with self.assertRaisesRegex(protocol.ValidationError, pattern):
                    proof_slice.validate_descriptor(value, formal_target.load_lock(), rows())


if __name__ == "__main__":
    unittest.main()
