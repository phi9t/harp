from __future__ import annotations

import contextlib
import io
import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import formal_target
import jin_validation
import proof_slice
import protocol


SOURCE_MAP = LAB / "formal_targets/jin-565b6a3/source-map.json"
EXPECTED_SHARED_PATH = (
    "/private/tmp/harp-mathematical-foundations-elan/toolchains/"
    "leanprover--lean4---v4.32.1/bin:/usr/bin:/bin"
)


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
                    "receipt_sha256": "1" * 64,
                }
            ],
            "output_declaration_name": "CrouzeixConjecture.PolynomialCrouzeixBound",
        }
    )
    return value


def rows() -> tuple[jin_validation.SourceMapRow, ...]:
    return jin_validation.load_source_map(SOURCE_MAP, formal_target.load_lock())


def rows_with_passed_first_receipt(
    receipt_sha256: str = "1" * 64,
) -> tuple[jin_validation.SourceMapRow, ...]:
    return tuple(
        jin_validation.SourceMapRow(
            row_id=row.row_id,
            source_locator=row.source_locator,
            statement_sha256=row.statement_sha256,
            lean_name=row.lean_name,
            dependency_ids=row.dependency_ids,
            status="passed" if row.row_id == "jin-max-polynomial-modulus" else row.status,
            receipt_sha256=(
                receipt_sha256
                if row.row_id == "jin-max-polynomial-modulus"
                else row.receipt_sha256
            ),
            blocked_reason=(
                None
                if row.row_id == "jin-max-polynomial-modulus"
                else row.blocked_reason
            ),
            failed_reason=row.failed_reason,
        )
        for row in rows()
    )


class FakeExecutor:
    def __init__(self, result: object) -> None:
        self.result = result
        self.calls: list[tuple[list[str], Path, int, int]] = []

    def __call__(
        self,
        argv: list[str],
        cwd: Path,
        timeout_seconds: int,
        max_output_bytes: int,
    ) -> object:
        self.calls.append((argv, cwd, timeout_seconds, max_output_bytes))
        return self.result


def materialized_task(directory: Path, value: dict[str, object] | None = None) -> Path:
    target = formal_target.load_lock()
    source_rows = jin_validation.load_source_map(SOURCE_MAP, target)
    desc = proof_slice.validate_descriptor(value or descriptor(), target, source_rows)
    return proof_slice.materialize_task(directory / "task", desc, source_rows)


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
            dependent_descriptor(),
            formal_target.load_lock(),
            rows_with_passed_first_receipt(),
        )

        self.assertEqual(
            desc.dependency_receipts,
            (
                proof_slice.DependencyReceipt(
                    row_id="jin-max-polynomial-modulus",
                    receipt_sha256="1" * 64,
                ),
            ),
        )

    def test_descriptor_rejects_dependency_receipt_for_blocked_row(self) -> None:
        with self.assertRaisesRegex(protocol.ValidationError, "not passed"):
            proof_slice.validate_descriptor(
                dependent_descriptor(), formal_target.load_lock(), rows()
            )

    def test_descriptor_rejects_dependency_receipt_digest_mismatch(self) -> None:
        value = dependent_descriptor()
        value["dependency_receipts"] = [
            {"row_id": "jin-max-polynomial-modulus", "receipt_sha256": "2" * 64}
        ]

        with self.assertRaisesRegex(protocol.ValidationError, "digest mismatch"):
            proof_slice.validate_descriptor(
                value,
                formal_target.load_lock(),
                rows_with_passed_first_receipt(),
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

    def test_materialize_task_writes_create_only_tree_and_slice_module(self) -> None:
        target = formal_target.load_lock()
        source_rows = jin_validation.load_source_map(SOURCE_MAP, target)
        desc = proof_slice.validate_descriptor(descriptor(), target, source_rows)
        with tempfile.TemporaryDirectory() as directory:
            task_dir = Path(directory).resolve() / "task"

            result = proof_slice.materialize_task(task_dir, desc, source_rows)

            self.assertEqual(result, task_dir)
            for relative in (
                "task.json",
                "source-slice.json",
                "module/Slice.lean",
                "build/command.json",
                "build/stdout.log",
                "build/stderr.log",
                "build/axioms.json",
                "result.json",
                "receipt.json",
            ):
                self.assertTrue((task_dir / relative).exists(), relative)

            source = (task_dir / "module/Slice.lean").read_text(encoding="utf-8")
            self.assertNotIn("import ", source)
            self.assertNotIn("CrouzeixConjecture.FinalTheorems", source)
            self.assertIn(
                "#check CrouzeixConjecture.maxPolynomialModulusOnNumericalRange",
                source,
            )
            self.assertNotIn("crouzeixConjecture", source)

            task = json.loads((task_dir / "task.json").read_text(encoding="utf-8"))
            self.assertEqual(task["descriptor"], desc.to_json())
            self.assertEqual(task["descriptor"]["dependency_receipts"], [])

            command = json.loads(
                (task_dir / "build/command.json").read_text(encoding="utf-8")
            )
            self.assertEqual(
                command["argv"],
                [
                    "lake",
                    "env",
                    "lean",
                    os.path.relpath(
                        task_dir / "module/Slice.lean",
                        proof_slice.SHARED_LEAN_ROOT,
                    ),
                ],
            )
            self.assertEqual(command["cwd"], "formalization/lean")
            env = command["env"]
            self.assertEqual(
                env.get("ELAN_HOME"),
                "/private/tmp/harp-mathematical-foundations-elan",
            )
            self.assertEqual(env.get("ELAN_TOOLCHAIN"), "leanprover/lean4:v4.32.1")
            self.assertEqual(
                env.get("PATH"),
                EXPECTED_SHARED_PATH,
            )
            self.assertNotIn("HOME", env)

            result_json = json.loads(
                (task_dir / "result.json").read_text(encoding="utf-8")
            )
            receipt = json.loads((task_dir / "receipt.json").read_text(encoding="utf-8"))
            axioms = json.loads(
                (task_dir / "build/axioms.json").read_text(encoding="utf-8")
            )
            self.assertEqual(result_json["status"], "not_attempted")
            self.assertEqual(receipt["status"], "not_attempted")
            self.assertEqual(axioms["status"], "not_attempted")
            self.assertFalse(axioms["scan_performed"])

            for relative in (
                "task.json",
                "source-slice.json",
                "build/command.json",
                "build/axioms.json",
                "result.json",
                "receipt.json",
            ):
                text = (task_dir / relative).read_text(encoding="utf-8")
                self.assertTrue(text.endswith("\n"), relative)

            with self.assertRaisesRegex(protocol.ValidationError, "already exists"):
                proof_slice.materialize_task(task_dir, desc, source_rows)

    def test_materialize_task_rejects_symlink_destination_or_ancestor(self) -> None:
        target = formal_target.load_lock()
        source_rows = jin_validation.load_source_map(SOURCE_MAP, target)
        desc = proof_slice.validate_descriptor(descriptor(), target, source_rows)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            real = root / "real"
            real.mkdir()
            link = root / "link"
            link.symlink_to(real)

            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                proof_slice.materialize_task(link, desc, source_rows)
            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                proof_slice.materialize_task(link / "task", desc, source_rows)

    def test_materialize_task_preserves_row_bound_dependency_receipts(self) -> None:
        target = formal_target.load_lock()
        source_rows = rows_with_passed_first_receipt()
        desc = proof_slice.validate_descriptor(dependent_descriptor(), target, source_rows)
        with tempfile.TemporaryDirectory() as directory:
            task_dir = Path(directory).resolve() / "task"

            proof_slice.materialize_task(task_dir, desc, source_rows)

            task = json.loads((task_dir / "task.json").read_text(encoding="utf-8"))
            self.assertEqual(
                task["descriptor"]["dependency_receipts"],
                [
                    {
                        "row_id": "jin-max-polynomial-modulus",
                        "receipt_sha256": "1" * 64,
                    }
                ],
            )

    def test_materialize_task_rejects_stale_source_row_before_creation(self) -> None:
        target = formal_target.load_lock()
        source_rows = rows_with_passed_first_receipt()
        desc = proof_slice.validate_descriptor(dependent_descriptor(), target, source_rows)
        stale_rows = tuple(
            jin_validation.SourceMapRow(
                row_id=row.row_id,
                source_locator=(
                    row.source_locator + "-stale"
                    if row.row_id == desc.source_map_row_id
                    else row.source_locator
                ),
                statement_sha256=(
                    "f" * 64 if row.row_id == desc.source_map_row_id else row.statement_sha256
                ),
                lean_name=(
                    "CrouzeixConjecture.Stale"
                    if row.row_id == desc.source_map_row_id
                    else row.lean_name
                ),
                dependency_ids=(
                    ()
                    if row.row_id == desc.source_map_row_id
                    else row.dependency_ids
                ),
                status=row.status,
            )
            for row in source_rows
        )
        with tempfile.TemporaryDirectory() as directory:
            task_dir = Path(directory).resolve() / "task"

            with self.assertRaisesRegex(protocol.ValidationError, "source-map row"):
                proof_slice.materialize_task(task_dir, desc, stale_rows)

            self.assertFalse(task_dir.exists())

    def test_materialize_task_removes_created_directory_after_midway_failure(self) -> None:
        target = formal_target.load_lock()
        source_rows = jin_validation.load_source_map(SOURCE_MAP, target)
        desc = proof_slice.validate_descriptor(descriptor(), target, source_rows)
        original = proof_slice._write_json_create_only

        def fail_after_source_slice(
            path: Path, value: object, label: str
        ) -> None:
            original(path, value, label)
            if label == "source slice":
                raise RuntimeError("forced materialization failure")

        with tempfile.TemporaryDirectory() as directory:
            task_dir = Path(directory).resolve() / "task"
            proof_slice._write_json_create_only = fail_after_source_slice
            try:
                with self.assertRaisesRegex(RuntimeError, "forced"):
                    proof_slice.materialize_task(task_dir, desc, source_rows)
            finally:
                proof_slice._write_json_create_only = original

            self.assertFalse(task_dir.exists())


class ProofSliceCliTests(unittest.TestCase):
    def test_main_json_select_returns_first_slice_identity(self) -> None:
        result = proof_slice.main_json(["select", "--source-map", str(SOURCE_MAP)])

        self.assertEqual(result["row_id"], "jin-max-polynomial-modulus")
        self.assertEqual(
            result["lean_name"],
            "CrouzeixConjecture.maxPolynomialModulusOnNumericalRange",
        )
        self.assertEqual(
            result["source_locator"],
            (
                "git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:"
                "Lean/CrouzeixConjecture/Statements.lean#L13-L18"
            ),
        )

    def test_main_json_materialize_creates_default_task(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = Path(directory).resolve() / "attempt-001"

            result = proof_slice.main_json(
                ["materialize", "--task-dir", str(task_dir)]
            )

            self.assertEqual(result["task_dir"], str(task_dir))
            self.assertEqual(result["row_id"], "jin-max-polynomial-modulus")
            self.assertTrue((task_dir / "task.json").exists())
            task = json.loads((task_dir / "task.json").read_text(encoding="utf-8"))
            self.assertEqual(task["descriptor"]["allowed_imports"], [])
            self.assertEqual(task["descriptor"]["dependency_receipts"], [])

    def test_main_json_run_returns_run_task_result_shape(self) -> None:
        expected = {
            "schema_version": "crouzeix-jin-proof-slice-result/v1",
            "status": "failed",
            "reason": "axiom audit output missing",
        }
        calls: list[Path] = []
        original_run_task = proof_slice.run_task

        def fake_run_task(task_dir: Path) -> dict[str, object]:
            calls.append(task_dir)
            return dict(expected)

        try:
            proof_slice.run_task = fake_run_task
            with tempfile.TemporaryDirectory() as directory:
                task_dir = Path(directory).resolve() / "attempt-001"
                result = proof_slice.main_json(["run", "--task-dir", str(task_dir)])
        finally:
            proof_slice.run_task = original_run_task

        self.assertEqual(result, expected)
        self.assertEqual(calls, [task_dir])

    def test_main_prints_sorted_json(self) -> None:
        original_main_json = proof_slice.main_json
        original_argv = sys.argv

        def fake_main_json(argv: list[str]) -> dict[str, object]:
            self.assertEqual(argv, ["select"])
            return {"z": 1, "a": 2}

        try:
            proof_slice.main_json = fake_main_json
            sys.argv = ["proof_slice.py", "select"]
            stdout = io.StringIO()
            with contextlib.redirect_stdout(stdout):
                proof_slice.main()
        finally:
            proof_slice.main_json = original_main_json
            sys.argv = original_argv

        self.assertEqual(stdout.getvalue(), '{"a": 2, "z": 1}\n')

    def test_script_entrypoint_prints_json(self) -> None:
        result = subprocess.run(
            [
                sys.executable,
                str(LAB / "proof_slice.py"),
                "select",
                "--source-map",
                str(SOURCE_MAP),
            ],
            check=True,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )

        payload = json.loads(result.stdout)
        self.assertEqual(payload["row_id"], "jin-max-polynomial-modulus")


class ProofSliceRunTaskTests(unittest.TestCase):
    def test_run_task_passes_with_clean_exit_and_axiom_audit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    0,
                    b"ok\n",
                    b"",
                    axiom_audit_output=b"axioms: none\n",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "passed")
            self.assertEqual(len(executor.calls), 1)
            argv, cwd, timeout_seconds, max_output_bytes = executor.calls[0]
            self.assertEqual(
                argv,
                [
                    "lake",
                    "env",
                    "lean",
                    os.path.relpath(
                        task_dir / "module/Slice.lean",
                        proof_slice.SHARED_LEAN_ROOT,
                    ),
                ],
            )
            self.assertEqual(cwd, proof_slice.SHARED_LEAN_ROOT)
            self.assertEqual(timeout_seconds, 3600)
            self.assertEqual(max_output_bytes, 1048576)
            self.assertEqual(
                (task_dir / "build/stdout.log").read_bytes(),
                b"ok\n",
            )
            self.assertEqual((task_dir / "build/stderr.log").read_bytes(), b"")

            result_json = json.loads(
                (task_dir / "result.json").read_text(encoding="utf-8")
            )
            receipt = json.loads((task_dir / "receipt.json").read_text(encoding="utf-8"))
            command = json.loads(
                (task_dir / "build/command.json").read_text(encoding="utf-8")
            )
            axioms = json.loads(
                (task_dir / "build/axioms.json").read_text(encoding="utf-8")
            )
            self.assertEqual(result_json["status"], "passed")
            self.assertEqual(receipt["status"], "passed")
            self.assertEqual(
                receipt["command_sha256"],
                proof_slice._canonical_sha256(command),
            )
            self.assertEqual(axioms["status"], "passed")
            self.assertTrue(axioms["scan_performed"])
            self.assertEqual(axioms["observed_axioms"], [])

    def test_run_task_classifies_ordinary_lean_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            executor = FakeExecutor(
                proof_slice.CommandResult(1, b"", b"type mismatch\n")
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("exit code 1", str(result["reason"]))
            self.assertEqual((task_dir / "build/stderr.log").read_bytes(), b"type mismatch\n")

    def test_run_task_classifies_blocker(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    None,
                    b"",
                    b"",
                    blocked_reason="missing pinned toolchain",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "blocked")
            self.assertIn("missing pinned toolchain", str(result["reason"]))
            receipt = json.loads((task_dir / "receipt.json").read_text(encoding="utf-8"))
            self.assertEqual(receipt["status"], "blocked")
            command = json.loads(
                (task_dir / "build/command.json").read_text(encoding="utf-8")
            )
            self.assertEqual(
                receipt["command_sha256"],
                proof_slice._canonical_sha256(command),
            )

    def test_run_task_blocker_precedes_output_cap_policy(self) -> None:
        value = descriptor()
        value["attempt_budget"] = {"timeout_seconds": 3600, "max_output_bytes": 1}
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve(), value)
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    None,
                    b"oversized",
                    b"",
                    blocked_reason="missing pinned toolchain",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "blocked")
            self.assertIn("missing pinned toolchain", str(result["reason"]))
            receipt = json.loads((task_dir / "receipt.json").read_text(encoding="utf-8"))
            self.assertEqual(receipt["status"], "blocked")

    def test_run_task_rejects_stale_empty_command_environment(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            command_path = task_dir / "build/command.json"
            command = json.loads(command_path.read_text(encoding="utf-8"))
            command["env"] = {}
            command_path.write_text(
                json.dumps(command, indent=2, sort_keys=True) + "\n",
                encoding="utf-8",
            )
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    0,
                    b"",
                    b"",
                    axiom_audit_output=b"axioms: none\n",
                )
            )

            with self.assertRaisesRegex(protocol.ValidationError, "command env"):
                proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(executor.calls, [])

    def test_run_task_requires_active_expected_declaration_check(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            (task_dir / "module/Slice.lean").write_text(
                "def unrelated : Nat := 1\n",
                encoding="utf-8",
            )
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    0,
                    b"",
                    b"",
                    axiom_audit_output=b"axioms: none\n",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("expected declaration check", str(result["reason"]))
            self.assertEqual(executor.calls, [])
            receipt = json.loads((task_dir / "receipt.json").read_text(encoding="utf-8"))
            self.assertIsNone(receipt["command_exit_code"])

    def test_run_task_rejects_expected_check_inside_comment(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            (task_dir / "module/Slice.lean").write_text(
                "/- #check CrouzeixConjecture.maxPolynomialModulusOnNumericalRange -/\n",
                encoding="utf-8",
            )
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    0,
                    b"",
                    b"",
                    axiom_audit_output=b"axioms: none\n",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("expected declaration check", str(result["reason"]))
            self.assertEqual(executor.calls, [])

    def test_run_task_fails_clean_compile_with_forbidden_axiom(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    0,
                    b"",
                    b"",
                    axiom_audit_output=b"axioms: Classical.choice\n",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("forbidden axiom", str(result["reason"]))
            axioms = json.loads(
                (task_dir / "build/axioms.json").read_text(encoding="utf-8")
            )
            self.assertEqual(axioms["status"], "failed")
            self.assertEqual(axioms["observed_axioms"], ["Classical.choice"])

    def test_run_task_allows_configured_axiom(self) -> None:
        value = descriptor()
        value["axiom_policy"] = {"allowed_axioms": ["Classical.choice"]}
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve(), value)
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    0,
                    b"",
                    b"",
                    axiom_audit_output=b"axioms: Classical.choice\n",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "passed")
            axioms = json.loads(
                (task_dir / "build/axioms.json").read_text(encoding="utf-8")
            )
            self.assertEqual(axioms["status"], "passed")
            self.assertEqual(axioms["observed_axioms"], ["Classical.choice"])

    def test_run_task_exit_zero_output_cap_still_performs_axiom_audit(self) -> None:
        value = descriptor()
        value["attempt_budget"] = {"timeout_seconds": 3600, "max_output_bytes": 1}
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve(), value)
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    0,
                    b"over-cap",
                    b"",
                    axiom_audit_output=b"axioms: none\n",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("output cap", str(result["reason"]))
            axioms = json.loads(
                (task_dir / "build/axioms.json").read_text(encoding="utf-8")
            )
            self.assertTrue(axioms["scan_performed"])
            self.assertEqual(axioms["status"], "passed")

    def test_run_task_exit_zero_truncated_output_fails_after_axiom_audit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    0,
                    b"axioms: none",
                    b"",
                    stdout_truncated=True,
                    axiom_audit_output=b"axioms: none\n",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("output cap", str(result["reason"]))
            axioms = json.loads(
                (task_dir / "build/axioms.json").read_text(encoding="utf-8")
            )
            self.assertTrue(axioms["scan_performed"])
            self.assertEqual(axioms["status"], "passed")
            receipt = json.loads((task_dir / "receipt.json").read_text(encoding="utf-8"))
            self.assertTrue(receipt["stdout_truncated"])
            self.assertFalse(receipt["stderr_truncated"])
            self.assertEqual(receipt["max_output_bytes"], 1048576)

    def test_run_task_bounds_oversized_executor_output_before_persisting(self) -> None:
        value = descriptor()
        value["attempt_budget"] = {"timeout_seconds": 3600, "max_output_bytes": 4}
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve(), value)
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    0,
                    b"abcde",
                    b"vwxyz",
                    axiom_audit_output=b"axioms: none\n",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("output cap", str(result["reason"]))
            self.assertEqual((task_dir / "build/stdout.log").read_bytes(), b"abcd")
            self.assertEqual((task_dir / "build/stderr.log").read_bytes(), b"vwxy")
            receipt = json.loads((task_dir / "receipt.json").read_text(encoding="utf-8"))
            self.assertTrue(receipt["stdout_truncated"])
            self.assertTrue(receipt["stderr_truncated"])
            self.assertEqual(receipt["max_output_bytes"], 4)
            axioms = json.loads(
                (task_dir / "build/axioms.json").read_text(encoding="utf-8")
            )
            self.assertTrue(axioms["scan_performed"])
            self.assertEqual(axioms["status"], "passed")

    def test_run_task_scrubs_workspace_path_from_command_logs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            workspace_path = proof_slice.REPO_ROOT.as_posix().encode("utf-8")
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    1,
                    b"",
                    b"error at " + workspace_path + b"/formalization/lean\n",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertEqual(
                (task_dir / "build/stderr.log").read_bytes(),
                b"error at <harp-workspace>/formalization/lean\n",
            )
            receipt = json.loads((task_dir / "receipt.json").read_text(encoding="utf-8"))
            self.assertEqual(
                receipt["stderr_sha256"],
                proof_slice.protocol.sha256_bytes(
                    b"error at <harp-workspace>/formalization/lean\n"
                ),
            )

    def test_run_task_exit_zero_without_axiom_audit_line_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            executor = FakeExecutor(proof_slice.CommandResult(0, b"ok\n", b""))

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("axiom audit output missing", str(result["reason"]))
            axioms = json.loads(
                (task_dir / "build/axioms.json").read_text(encoding="utf-8")
            )
            self.assertFalse(axioms["scan_performed"])
            self.assertEqual(axioms["status"], "failed")

    def test_run_task_exit_zero_with_malformed_audit_channel_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    0,
                    b"ok\n",
                    b"",
                    axiom_audit_output=b"candidate text\naxioms: none\n",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("axiom audit output malformed", str(result["reason"]))
            axioms = json.loads(
                (task_dir / "build/axioms.json").read_text(encoding="utf-8")
            )
            self.assertFalse(axioms["scan_performed"])
            self.assertEqual(axioms["status"], "failed")

    def test_run_task_ignores_candidate_controlled_stdout_axiom_line(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            executor = FakeExecutor(
                proof_slice.CommandResult(0, b"ok\naxioms: none\n", b"")
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("axiom audit output missing", str(result["reason"]))
            axioms = json.loads(
                (task_dir / "build/axioms.json").read_text(encoding="utf-8")
            )
            self.assertFalse(axioms["scan_performed"])
            self.assertEqual(axioms["status"], "failed")

    def test_run_task_fails_proof_hole_token_without_executor_call(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            (task_dir / "module/Slice.lean").write_text(
                (
                    "#check CrouzeixConjecture.maxPolynomialModulusOnNumericalRange\n"
                    "theorem bad : True := by\n"
                    "  sorry\n"
                ),
                encoding="utf-8",
            )
            executor = FakeExecutor(proof_slice.CommandResult(0, b"ok\n", b""))

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("proof-hole token", str(result["reason"]))
            self.assertEqual(executor.calls, [])
            self.assertEqual((task_dir / "build/stdout.log").read_bytes(), b"")
            receipt = json.loads((task_dir / "receipt.json").read_text(encoding="utf-8"))
            self.assertIsNone(receipt["command_exit_code"])

    def test_run_task_rejects_expected_check_inside_nested_block_comment(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            task_dir = materialized_task(Path(directory).resolve())
            (task_dir / "module/Slice.lean").write_text(
                (
                    "/- outer comment\n"
                    "  /- nested comment -/\n"
                    "  #check CrouzeixConjecture.maxPolynomialModulusOnNumericalRange\n"
                    "-/\n"
                ),
                encoding="utf-8",
            )
            executor = FakeExecutor(
                proof_slice.CommandResult(
                    0,
                    b"",
                    b"",
                    axiom_audit_output=b"axioms: none\n",
                )
            )

            result = proof_slice.run_task(task_dir, executor=executor)

            self.assertEqual(result["status"], "failed")
            self.assertIn("expected declaration check", str(result["reason"]))
            self.assertEqual(executor.calls, [])

    def test_run_task_rejects_symlinked_result_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            task_dir = materialized_task(root)
            outside = root / "outside-result.json"
            outside.write_text("{}", encoding="utf-8")
            (task_dir / "result.json").unlink()
            (task_dir / "result.json").symlink_to(outside)
            executor = FakeExecutor(proof_slice.CommandResult(0, b"ok\n", b""))

            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                proof_slice.run_task(task_dir, executor=executor)

    def test_subprocess_executor_uses_shared_lean_environment(self) -> None:
        captured: dict[str, object] = {}
        original_popen = proof_slice.subprocess.Popen
        original_which = proof_slice.shutil.which

        def fake_which(executable: str, path: str | None = None) -> str | None:
            self.assertEqual(executable, "lake")
            self.assertIsNotNone(path)
            return "/private/tmp/harp-mathematical-foundations-elan/toolchains/leanprover--lean4---v4.32.1/bin/lake"

        class FakePopen:
            def __init__(self, *args: object, **kwargs: object) -> None:
                captured["args"] = args
                captured["kwargs"] = kwargs
                self.stdout = io.BytesIO(b"ok\n")
                self.stderr = io.BytesIO(b"")

            def wait(self, timeout: int) -> int:
                captured["timeout"] = timeout
                return 0

        def fake_popen(*args: object, **kwargs: object) -> object:
            captured["args"] = args
            captured["kwargs"] = kwargs
            return FakePopen(*args, **kwargs)

        try:
            proof_slice.shutil.which = fake_which
            proof_slice.subprocess.Popen = fake_popen
            result = proof_slice._subprocess_executor(
                ["lake", "env", "lean", "module/Slice.lean"],
                proof_slice.SHARED_LEAN_ROOT,
                3600,
                1024,
            )
        finally:
            proof_slice.subprocess.Popen = original_popen
            proof_slice.shutil.which = original_which

        self.assertEqual(result.exit_code, 0)
        env = captured["kwargs"]["env"]
        self.assertEqual(
            env["ELAN_HOME"],
            "/private/tmp/harp-mathematical-foundations-elan",
        )
        self.assertEqual(env["ELAN_TOOLCHAIN"], "leanprover/lean4:v4.32.1")
        self.assertEqual(env["PATH"], EXPECTED_SHARED_PATH)
        self.assertNotIn("HOME", env)

    def test_subprocess_executor_resolves_executable_before_passing_environment(self) -> None:
        captured: dict[str, object] = {}
        original_popen = proof_slice.subprocess.Popen
        original_which = proof_slice.shutil.which

        def fake_which(executable: str, path: str | None = None) -> str | None:
            captured["which"] = executable
            captured["which_path"] = path
            return "/opt/toolchains/lake"

        class FakePopen:
            def __init__(self, *args: object, **kwargs: object) -> None:
                captured["args"] = args
                captured["kwargs"] = kwargs
                self.stdout = io.BytesIO(b"ok\n")
                self.stderr = io.BytesIO(b"")

            def wait(self, timeout: int) -> int:
                captured["timeout"] = timeout
                return 0

        def fake_popen(*args: object, **kwargs: object) -> object:
            captured["args"] = args
            captured["kwargs"] = kwargs
            return FakePopen(*args, **kwargs)

        try:
            proof_slice.shutil.which = fake_which
            proof_slice.subprocess.Popen = fake_popen
            result = proof_slice._subprocess_executor(
                ["lake", "env", "lean", "module/Slice.lean"],
                Path("/tmp"),
                3600,
                1024,
            )
        finally:
            proof_slice.subprocess.Popen = original_popen
            proof_slice.shutil.which = original_which

        self.assertEqual(result.exit_code, 0)
        self.assertEqual(captured["which"], "lake")
        self.assertIsNotNone(captured["which_path"])
        self.assertEqual(
            captured["args"][0],
            ["/opt/toolchains/lake", "env", "lean", "module/Slice.lean"],
        )
        self.assertEqual(
            captured["kwargs"]["env"],
            proof_slice._shared_lean_environment(),
        )

    def test_subprocess_executor_blocks_when_executable_cannot_be_resolved(self) -> None:
        original_which = proof_slice.shutil.which

        def fake_which(executable: str, path: str | None = None) -> str | None:
            self.assertEqual(executable, "missing-tool")
            self.assertIsNotNone(path)
            return None

        try:
            proof_slice.shutil.which = fake_which
            result = proof_slice._subprocess_executor(
                ["missing-tool", "arg"],
                Path("/tmp"),
                3600,
                1024,
            )
        finally:
            proof_slice.shutil.which = original_which

        self.assertIsNone(result.exit_code)
        self.assertIn("missing executable", str(result.blocked_reason))
        self.assertIn("missing-tool", str(result.blocked_reason))

    def test_subprocess_executor_timeout_kills_group_and_bounds_pipe_join(self) -> None:
        captured: dict[str, object] = {}
        pipes: list[object] = []
        original_popen = proof_slice.subprocess.Popen
        original_which = proof_slice.shutil.which
        original_killpg = proof_slice.os.killpg
        original_bounded_pipe = proof_slice._BoundedPipe

        def fake_which(executable: str, path: str | None = None) -> str | None:
            self.assertEqual(executable, "lake")
            self.assertIsNotNone(path)
            return "/opt/toolchains/lake"

        class FakePopen:
            pid = 4242

            def __init__(self, *args: object, **kwargs: object) -> None:
                captured["args"] = args
                captured["kwargs"] = kwargs
                self.stdout = object()
                self.stderr = object()

            def wait(self, timeout: int) -> int:
                captured["timeout"] = timeout
                raise proof_slice.subprocess.TimeoutExpired(
                    cmd=["lake", "env", "lean"], timeout=timeout
                )

            def kill(self) -> None:
                captured["fallback_kill"] = True

        class FakeBoundedPipe:
            data = b""
            truncated = False

            def __init__(self, max_bytes: int) -> None:
                self.max_bytes = max_bytes
                self.join_timeout: float | None = None
                pipes.append(self)

            def start(self, stream: object) -> None:
                self.stream = stream

            def join(self, timeout: float | None = None) -> None:
                self.join_timeout = timeout

        def fake_killpg(pid: int, sig: int) -> None:
            captured["killpg"] = (pid, sig)

        try:
            proof_slice.shutil.which = fake_which
            proof_slice.subprocess.Popen = FakePopen
            proof_slice.os.killpg = fake_killpg
            proof_slice._BoundedPipe = FakeBoundedPipe
            result = proof_slice._subprocess_executor(
                ["lake", "env", "lean", "module/Slice.lean"],
                Path("/tmp"),
                7,
                1024,
            )
        finally:
            proof_slice.subprocess.Popen = original_popen
            proof_slice.shutil.which = original_which
            proof_slice.os.killpg = original_killpg
            proof_slice._BoundedPipe = original_bounded_pipe

        self.assertIsNone(result.exit_code)
        self.assertIn("timed out", str(result.blocked_reason))
        self.assertEqual(captured["killpg"], (4242, proof_slice.signal.SIGKILL))
        self.assertNotIn("fallback_kill", captured)
        self.assertEqual(captured["kwargs"]["start_new_session"], True)
        self.assertEqual(captured["timeout"], 7)
        self.assertEqual(
            [pipe.join_timeout for pipe in pipes],
            [proof_slice.PIPE_DRAIN_GRACE_SECONDS] * 2,
        )

    def test_subprocess_executor_truncates_returned_output(self) -> None:
        original_popen = proof_slice.subprocess.Popen
        original_which = proof_slice.shutil.which

        def fake_which(executable: str, path: str | None = None) -> str | None:
            self.assertEqual(executable, "lake")
            self.assertIsNotNone(path)
            return "/opt/toolchains/lake"

        class FakePopen:
            def __init__(self, *args: object, **kwargs: object) -> None:
                self.stdout = io.BytesIO(b"abcdef")
                self.stderr = io.BytesIO(b"ghijkl")

            def wait(self, timeout: int) -> int:
                return 1

        try:
            proof_slice.shutil.which = fake_which
            proof_slice.subprocess.Popen = FakePopen
            result = proof_slice._subprocess_executor(
                ["lake", "env", "lean", "module/Slice.lean"],
                Path("/tmp"),
                3600,
                3,
            )
        finally:
            proof_slice.subprocess.Popen = original_popen
            proof_slice.shutil.which = original_which

        self.assertEqual(result.exit_code, 1)
        self.assertEqual(result.stdout, b"abc")
        self.assertEqual(result.stderr, b"ghi")
        self.assertTrue(result.stdout_truncated)
        self.assertTrue(result.stderr_truncated)
        self.assertIsNone(result.axiom_audit_output)


if __name__ == "__main__":
    unittest.main()
