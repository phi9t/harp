from __future__ import annotations

import json
import os
import sys
import tempfile
import time
import unittest
from dataclasses import replace
from pathlib import Path
from unittest import mock


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import ls_receipts  # noqa: E402
import ls_contract  # noqa: E402
import ls_validation  # noqa: E402
import protocol  # noqa: E402


GRAPH = LAB / "formal_targets/lorist-schwenninger/source-graph.json"
NODE_BINDINGS = {
    "ls-equation-one-terminal-bound": (
        "CrouzeixConjecture.LoristSchwenninger.DilationData."
        "perturbation_mul_target_power_norm_le",
        "formalization/lean/Crouzeix/LoristSchwenninger/Dilation.lean",
    ),
    "ls-power-recurrence": (
        "CrouzeixConjecture.LoristSchwenninger.DilationData.equation_three_lower_bound",
        "formalization/lean/Crouzeix/LoristSchwenninger/OperatorRecurrence.lean",
    ),
    "ls-scalar-contradiction": (
        "CrouzeixConjecture.LoristSchwenninger.scalar_endpoint_le_two",
        "formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean",
    ),
    "ls-perturbation-lemma": (
        "CrouzeixConjecture.LoristSchwenninger.DilationData.norm_target_le_two",
        "formalization/lean/Crouzeix/LoristSchwenninger/PerturbationLemma.lean",
    ),
    "ls-double-layer-realization": (
        "CrouzeixConjecture.LoristSchwenninger."
        "norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary",
        "formalization/lean/Crouzeix/LoristSchwenninger/ConcreteDilation.lean",
    ),
    "ls-terminal-crouzeix": (
        "CrouzeixConjecture.loristSchwenningerMainTheorem",
        "formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean",
    ),
}
DEPENDENCIES = {
    "ls-equation-one-terminal-bound": (),
    "ls-power-recurrence": ("ls-equation-one-terminal-bound",),
    "ls-scalar-contradiction": (),
    "ls-perturbation-lemma": (
        "ls-power-recurrence",
        "ls-scalar-contradiction",
    ),
    "ls-double-layer-realization": ("ls-perturbation-lemma",),
    "ls-terminal-crouzeix": (
        "ls-perturbation-lemma",
        "ls-double-layer-realization",
    ),
}
EXPECTED_MEMBERS = {
    "task.json",
    "source-slice.json",
    "result.json",
    "receipt.json",
    "build/command.json",
    "build/stdout.log",
    "build/stderr.log",
    "build/axioms.txt",
}
EXPECTED_RECEIPT_KEYS = {
    "schema_version",
    "route_id",
    "source_node_id",
    "status",
    "reason",
    "build_target",
    "expected_lean_declaration",
    "allowed_axioms",
    "observed_axioms",
    "task_sha256",
    "source_slice_sha256",
    "command_sha256",
    "stdout_sha256",
    "stderr_sha256",
    "axiom_audit_sha256",
    "result_sha256",
    "module_sha256",
}
EXPECTED_ENV = {
    "ELAN_HOME": "/private/tmp/harp-mathematical-foundations-elan",
    "ELAN_TOOLCHAIN": "leanprover/lean4:v4.32.1",
    "PATH": (
        "/private/tmp/harp-mathematical-foundations-elan/toolchains/"
        "leanprover--lean4---v4.32.1/bin:/usr/bin:/bin"
    ),
}
EXPECTED_COMMAND_KEYS = {
    "schema_version",
    "argv",
    "cache_policy",
    "cwd",
    "elan_toolchain",
    "env",
    "exit_code",
    "output_cap_bytes",
    "timeout_seconds",
    "wrapper_sha256",
    "lake_manifest_sha256",
    "dependency_cache_metadata_sha256",
    "required_mathlib_artifacts_sha256",
    "local_source_closure_sha256",
}
AGGREGATE_STDOUT = (
    b"[lean] target=CrouzeixLoristSchwenninger\n"
    b"[lean] root=formalization/lean\n"
    b"[lean] scan_seconds=0\n"
    b"[lean] cache_seconds=0\n"
    b"[lean] lake_seconds=0\n"
    b"[lean] total_seconds=0\n"
    b"[lean] outcome=passed\n"
)
SAFE_WRAPPER = """#!/bin/sh
if ! has_valid_olean_header "$required_artifact"; then
  printf '%s\n' "Lean dependency cache is missing or invalid: $required_artifact"
fi
if [ -s "$missing_cache_list" ]; then
  printf '%s\n' "Lean verification refuses to rebuild common dependencies."
  exit 1
fi
lake --try-cache build "$1"
"""
REQUIRED_MATHLIB_MODULE = "Mathlib.Algebra.Order.Ring.Defs"


def make_rows() -> tuple[ls_validation.LSGraphRow, ...]:
    return ls_validation.load_route_graph(GRAPH)


def make_workspace(root: Path) -> tuple[Path, Path]:
    repository_root = root / "repository"
    formal_target_root = (
        repository_root
        / "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger"
    )
    (formal_target_root / "proof-slices").mkdir(parents=True)
    lean_root = repository_root / "formalization/lean"
    imports = []
    for node_id, (_, build_target) in NODE_BINDINGS.items():
        module_path = repository_root / build_target
        module_path.parent.mkdir(parents=True, exist_ok=True)
        module_path.write_text(
            f"import {REQUIRED_MATHLIB_MODULE}\n"
            f"-- deterministic module for {node_id}\n",
            encoding="utf-8",
        )
        module = Path(build_target).relative_to("formalization/lean").with_suffix("")
        imports.append("import " + ".".join(module.parts))
    (lean_root / "CrouzeixLoristSchwenninger.lean").write_text(
        "\n".join(imports) + "\n", encoding="utf-8"
    )
    (lean_root / "lake-manifest.json").write_text("{}\n", encoding="utf-8")
    required_artifact = (
        lean_root
        / ".lake/packages/mathlib/.lake/build/lib/lean"
        / Path(*REQUIRED_MATHLIB_MODULE.split(".")).with_suffix(".olean")
    )
    required_artifact.parent.mkdir(parents=True)
    required_artifact.write_bytes(b"olean-required-mathlib-artifact\n")
    dependency_probe = (
        lean_root / ".lake/packages/batteries/.lake/build/lib/lean/Batteries.olean"
    )
    dependency_probe.parent.mkdir(parents=True)
    dependency_probe.write_bytes(b"olean-dependency-cache-probe\n")
    project_build = lean_root / ".lake/build/lib/lean"
    project_build.mkdir(parents=True)
    (project_build / "Project.olean").write_bytes(b"project output\n")
    (repository_root / "scripts").mkdir()
    (repository_root / "scripts/check_lean_library.sh").write_text(
        SAFE_WRAPPER, encoding="utf-8"
    )
    return repository_root, formal_target_root


class FakeExecutor:
    def __init__(
        self,
        *,
        aggregate_exit_code: int = 0,
        audit_failure_number: int | None = None,
        disallowed_axiom_number: int | None = None,
        oversized_aggregate: bool = False,
    ) -> None:
        self.aggregate_exit_code = aggregate_exit_code
        self.audit_failure_number = audit_failure_number
        self.disallowed_axiom_number = disallowed_axiom_number
        self.oversized_aggregate = oversized_aggregate
        self.calls: list[dict[str, object]] = []
        self.audit_sources: list[tuple[Path, str]] = []

    def __call__(
        self,
        argv: list[str],
        cwd: Path,
        env: dict[str, str],
        timeout_seconds: int,
        max_output_bytes: int,
    ) -> ls_receipts.CommandResult:
        self.calls.append(
            {
                "argv": list(argv),
                "cwd": cwd,
                "env": dict(env),
                "timeout_seconds": timeout_seconds,
                "max_output_bytes": max_output_bytes,
            }
        )
        if argv == [
            "scripts/check_lean_library.sh",
            "CrouzeixLoristSchwenninger",
        ]:
            stdout = (
                b"x" * (max_output_bytes + 1)
                if self.oversized_aggregate
                else AGGREGATE_STDOUT
            )
            return ls_receipts.CommandResult(
                self.aggregate_exit_code, stdout, b"aggregate stderr\n"
            )

        audit_number = len(self.audit_sources) + 1
        source_path = cwd / argv[-1]
        source = source_path.read_text(encoding="utf-8")
        self.audit_sources.append((source_path, source))
        if audit_number == self.audit_failure_number:
            return ls_receipts.CommandResult(1, b"", b"audit failed\n")
        declaration = source.split("#print axioms ", 1)[1].strip()
        axioms = (
            "Classical.choice, unsafeAxiom"
            if audit_number == self.disallowed_axiom_number
            else "propext,\n Classical.choice,\n Quot.sound"
        )
        return ls_receipts.CommandResult(
            0, f"'{declaration}' depends on axioms: [{axioms}]\n".encode(), b""
        )


def canonical_json_bytes(value: object) -> bytes:
    return (
        json.dumps(
            value,
            sort_keys=True,
            separators=(",", ":"),
            ensure_ascii=False,
            allow_nan=False,
        ).encode("utf-8")
        + b"\n"
    )


def exception_group_members(error: BaseException) -> tuple[BaseException, ...]:
    members = getattr(error, "exceptions", ())
    if not isinstance(members, tuple):
        return ()
    return tuple(member for member in members if isinstance(member, BaseException))


def exception_messages(error: BaseException) -> list[str]:
    messages = [str(error)]
    nested_errors = exception_group_members(error)
    if nested_errors:
        for nested in nested_errors:
            messages.extend(exception_messages(nested))
    elif error.__context__ is not None:
        messages.extend(exception_messages(error.__context__))
    return messages


class LSReceiptPublisherTests(unittest.TestCase):
    def setUp(self) -> None:
        self.lock_root = tempfile.TemporaryDirectory()
        self.addCleanup(self.lock_root.cleanup)
        patcher = mock.patch.object(
            ls_receipts,
            "PUBLICATION_LOCK_ROOT",
            Path(self.lock_root.name) / "locks",
        )
        patcher.start()
        self.addCleanup(patcher.stop)

    def assert_candidates_match_visible_attempts(
        self,
        error: ls_receipts.PartialPublicationError,
        repository_root: Path,
        proof_slices: Path,
        *,
        expect_valid_members: bool = True,
    ) -> None:
        expected = tuple(
            {
                "attempt_path": attempt.relative_to(repository_root).as_posix(),
                "receipt_sha256": protocol.sha256_bytes(
                    (attempt / "receipt.json").read_bytes()
                ),
            }
            for row in make_rows()
            if (attempt := proof_slices / row.node_id / "attempt-001").is_dir()
        )
        self.assertEqual(error.candidates, expected)
        for candidate in error.candidates:
            attempt = repository_root / candidate["attempt_path"]
            self.assertEqual(
                {
                    path.relative_to(attempt).as_posix()
                    for path in attempt.rglob("*")
                    if path.is_file()
                },
                EXPECTED_MEMBERS,
            )
            self.assertEqual(
                protocol.sha256_bytes((attempt / "receipt.json").read_bytes()),
                candidate["receipt_sha256"],
            )
            if expect_valid_members:
                receipt = json.loads((attempt / "receipt.json").read_bytes())
                for digest_field, relative_path in ls_validation.MEMBER_PATHS.items():
                    self.assertEqual(
                        protocol.sha256_bytes((attempt / relative_path).read_bytes()),
                        receipt[digest_field],
                    )

    def test_publishes_six_valid_receipts_from_one_aggregate_build(self) -> None:
        rows = make_rows()
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            executor = FakeExecutor()

            published = ls_receipts.publish_ls_receipts(
                rows, repository_root, formal_target_root, executor=executor
            )

            self.assertEqual(list(published), list(NODE_BINDINGS))
            self.assertEqual(len(executor.calls), 7)
            aggregate = executor.calls[0]
            self.assertEqual(ls_contract.AGGREGATE_MODULE, "CrouzeixLoristSchwenninger")
            self.assertEqual(
                aggregate["argv"],
                ["scripts/check_lean_library.sh", "CrouzeixLoristSchwenninger"],
            )
            self.assertEqual(aggregate["cwd"], repository_root)
            for call in executor.calls:
                self.assertEqual(call["env"], EXPECTED_ENV)
                self.assertEqual(call["timeout_seconds"], 3600)
                self.assertEqual(call["max_output_bytes"], 1048576)
                self.assertNotIn("update", call["argv"])
                self.assertNotIn("get", call["argv"])
            audited_calls = executor.calls[1:]
            self.assertEqual(len(audited_calls), len(NODE_BINDINGS))
            for call, (node_id, (declaration, build_target)) in zip(
                audited_calls, NODE_BINDINGS.items()
            ):
                module = (
                    Path(build_target).relative_to("formalization/lean").with_suffix("")
                )
                self.assertEqual(call["argv"][:3], ["lake", "env", "lean"])
                self.assertEqual(
                    executor.audit_sources[list(NODE_BINDINGS).index(node_id)][1],
                    f"import {'.'.join(module.parts)}\n#print axioms {declaration}\n",
                )

            updated_rows = []
            for row in rows:
                publication = published[row.node_id]
                attempt = repository_root / publication["attempt_path"]
                self.assertEqual(attempt.name, "attempt-001")
                self.assertEqual(
                    {
                        path.relative_to(attempt).as_posix()
                        for path in attempt.rglob("*")
                        if path.is_file()
                    },
                    EXPECTED_MEMBERS,
                )
                for json_name in (
                    "task.json",
                    "source-slice.json",
                    "result.json",
                    "receipt.json",
                    "build/command.json",
                ):
                    data = (attempt / json_name).read_bytes()
                    self.assertEqual(data, canonical_json_bytes(json.loads(data)))

                command = json.loads(
                    (attempt / "build/command.json").read_text(encoding="utf-8")
                )
                self.assertEqual(set(command), EXPECTED_COMMAND_KEYS)
                self.assertEqual(
                    command["schema_version"], "crouzeix-ls-lean-command/v2"
                )
                self.assertEqual(
                    command["argv"],
                    ["scripts/check_lean_library.sh", "CrouzeixLoristSchwenninger"],
                )
                self.assertEqual(command["cwd"], ".")
                self.assertEqual(command["env"], EXPECTED_ENV)
                self.assertEqual(
                    command["elan_toolchain"], EXPECTED_ENV["ELAN_TOOLCHAIN"]
                )
                self.assertEqual(command["exit_code"], 0)
                self.assertEqual(command["output_cap_bytes"], 1048576)
                self.assertEqual(command["timeout_seconds"], 3600)
                self.assertEqual(
                    command["wrapper_sha256"],
                    protocol.sha256_bytes(
                        (repository_root / "scripts/check_lean_library.sh").read_bytes()
                    ),
                )
                self.assertEqual(
                    command["lake_manifest_sha256"],
                    protocol.sha256_bytes(
                        (
                            repository_root / "formalization/lean/lake-manifest.json"
                        ).read_bytes()
                    ),
                )
                for field in (
                    "dependency_cache_metadata_sha256",
                    "required_mathlib_artifacts_sha256",
                    "local_source_closure_sha256",
                ):
                    self.assertRegex(command[field], r"\A[0-9a-f]{64}\Z")
                receipt_bytes = (attempt / "receipt.json").read_bytes()
                self.assertEqual(
                    publication["receipt_sha256"],
                    protocol.sha256_bytes(receipt_bytes),
                )
                receipt = json.loads(receipt_bytes)
                self.assertEqual(set(receipt), EXPECTED_RECEIPT_KEYS)
                self.assertEqual(
                    receipt["expected_lean_declaration"],
                    publication["lean_name"],
                )
                self.assertEqual(
                    receipt["allowed_axioms"],
                    ["Classical.choice", "Quot.sound", "propext"],
                )
                self.assertEqual(receipt["observed_axioms"], receipt["allowed_axioms"])
                for digest_field, relative_path in ls_validation.MEMBER_PATHS.items():
                    self.assertEqual(
                        receipt[digest_field],
                        protocol.sha256_bytes((attempt / relative_path).read_bytes()),
                    )
                _, build_target = NODE_BINDINGS[row.node_id]
                self.assertEqual(
                    receipt["module_sha256"],
                    protocol.sha256_bytes(
                        (repository_root / build_target).read_bytes()
                    ),
                )
                updated_rows.append(
                    replace(
                        row,
                        lean_name=publication["lean_name"],
                        status=publication["status"],
                        receipt_sha256=publication["receipt_sha256"],
                        blocked_reason=None,
                        failed_reason=None,
                    )
                )

            self.assertTrue(
                all(not path.exists() for path, _ in executor.audit_sources)
            )
            self.assertEqual(
                list((formal_target_root).glob(".ls-receipts-stage-*")), []
            )
            validated = ls_validation.validate_committed_receipts(
                tuple(updated_rows), formal_target_root, repository_root
            )
            self.assertEqual(set(validated), set(NODE_BINDINGS))

    def test_rejects_aggregate_stdout_without_exact_success_markers(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            executor = FakeExecutor()

            def malformed_stdout(*args: object) -> ls_receipts.CommandResult:
                result = executor(*args)
                if len(executor.calls) == 1:
                    return replace(
                        result, stdout=b"[lean] outcome=passed with warnings\n"
                    )
                return result

            with self.assertRaisesRegex(
                protocol.ValidationError, "aggregate stdout.*success markers"
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=malformed_stdout,
                )

            self.assertEqual(
                list((formal_target_root / "proof-slices").glob("*/attempt-*")), []
            )

    def test_aggregate_failure_or_output_overflow_publishes_nothing(self) -> None:
        for executor in (
            FakeExecutor(aggregate_exit_code=1),
            FakeExecutor(oversized_aggregate=True),
        ):
            with self.subTest(executor=executor):
                with tempfile.TemporaryDirectory() as directory:
                    repository_root, formal_target_root = make_workspace(
                        Path(directory).resolve()
                    )

                    with self.assertRaises(protocol.ValidationError):
                        ls_receipts.publish_ls_receipts(
                            make_rows(),
                            repository_root,
                            formal_target_root,
                            executor=executor,
                        )

                    self.assertEqual(len(executor.calls), 1)
                    self.assertEqual(
                        list((formal_target_root / "proof-slices").iterdir()), []
                    )
                    self.assertEqual(
                        list(formal_target_root.glob(".ls-receipts-stage-*")), []
                    )

    def test_audit_failure_or_disallowed_axiom_publishes_nothing(self) -> None:
        for executor in (
            FakeExecutor(audit_failure_number=3),
            FakeExecutor(disallowed_axiom_number=6),
        ):
            with self.subTest(executor=executor):
                with tempfile.TemporaryDirectory() as directory:
                    repository_root, formal_target_root = make_workspace(
                        Path(directory).resolve()
                    )

                    with self.assertRaises(protocol.ValidationError):
                        ls_receipts.publish_ls_receipts(
                            make_rows(),
                            repository_root,
                            formal_target_root,
                            executor=executor,
                        )

                    self.assertEqual(
                        list((formal_target_root / "proof-slices").iterdir()), []
                    )
                    self.assertEqual(
                        list(formal_target_root.glob(".ls-receipts-stage-*")), []
                    )
                    self.assertTrue(
                        all(not path.exists() for path, _ in executor.audit_sources)
                    )

    def test_post_rename_validation_failure_preserves_visible_candidates(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_read = ls_receipts._read_safe_file_at

            def fail_published_receipt_read(
                directory_descriptor: int, name: str, label: str, maximum: int
            ) -> bytes:
                if name == "receipt.json" and label.startswith("published receipt"):
                    raise protocol.ValidationError("injected published receipt failure")
                return original_read(directory_descriptor, name, label, maximum)

            with mock.patch.object(
                ls_receipts, "_read_safe_file_at", fail_published_receipt_read
            ):
                with self.assertRaisesRegex(
                    protocol.ValidationError,
                    "committed partial publication.*attempt-001",
                ):
                    ls_receipts.publish_ls_receipts(
                        make_rows(),
                        repository_root,
                        formal_target_root,
                        executor=FakeExecutor(),
                    )

            visible = sorted(proof_slices.glob("*/attempt-001"))
            self.assertEqual(len(visible), len(NODE_BINDINGS))
            for attempt in visible:
                self.assertEqual(
                    {
                        path.relative_to(attempt).as_posix()
                        for path in attempt.rglob("*")
                        if path.is_file()
                    },
                    EXPECTED_MEMBERS,
                )
            self.assertEqual(list(proof_slices.glob("*/.attempt-*")), [])

    def test_post_rename_non_receipt_mutation_is_a_partial_publication(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_rename = ls_receipts._rename_directory_no_replace
            mutated_attempt: Path | None = None
            tampered_bytes = b'{"tampered":true}\n'

            def rename_then_mutate_member(
                node_root: ls_receipts._PinnedDirectory,
                source: ls_receipts._PinnedDirectory,
                destination_name: str,
                *,
                candidate: ls_receipts._PublicationCandidate | None = None,
            ) -> ls_receipts._PinnedDirectory:
                nonlocal mutated_attempt
                published = original_rename(
                    node_root, source, destination_name, candidate=candidate
                )
                if mutated_attempt is None:
                    mutated_attempt = published.path
                    (mutated_attempt / "task.json").write_bytes(tampered_bytes)
                return published

            with (
                mock.patch.object(
                    ls_receipts,
                    "_rename_directory_no_replace",
                    rename_then_mutate_member,
                ),
                self.assertRaises(ls_receipts.PartialPublicationError) as caught,
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertIsNotNone(mutated_attempt)
            assert mutated_attempt is not None
            self.assertEqual(
                (mutated_attempt / "task.json").read_bytes(), tampered_bytes
            )
            self.assertEqual(len(caught.exception.candidates), 1)
            self.assert_candidates_match_visible_attempts(
                caught.exception,
                repository_root,
                proof_slices,
                expect_valid_members=False,
            )
            self.assertEqual(list(proof_slices.glob("*/.attempt-*")), [])

    def test_final_bundle_revalidation_catches_late_member_mutation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_require_snapshots = ls_receipts._require_module_snapshots
            snapshot_checks = 0
            mutated_attempt: Path | None = None
            tampered_bytes = b"tampered stdout after all renames\n"

            def mutate_after_all_renames(
                repository: Path,
                plans: tuple[ls_receipts._AttemptPlan, ...],
                expected: dict[str, ls_receipts._FileSnapshot],
            ) -> None:
                nonlocal snapshot_checks, mutated_attempt
                original_require_snapshots(repository, plans, expected)
                snapshot_checks += 1
                if snapshot_checks == 2:
                    mutated_attempt = (
                        proof_slices / "ls-equation-one-terminal-bound" / "attempt-001"
                    )
                    (mutated_attempt / "build/stdout.log").write_bytes(tampered_bytes)

            with (
                mock.patch.object(
                    ls_receipts,
                    "_require_module_snapshots",
                    mutate_after_all_renames,
                ),
                self.assertRaises(ls_receipts.PartialPublicationError) as caught,
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertIsNotNone(mutated_attempt)
            assert mutated_attempt is not None
            self.assertEqual(
                (mutated_attempt / "build/stdout.log").read_bytes(), tampered_bytes
            )
            self.assertEqual(len(caught.exception.candidates), len(NODE_BINDINGS))
            self.assert_candidates_match_visible_attempts(
                caught.exception,
                repository_root,
                proof_slices,
                expect_valid_members=False,
            )

    def test_publication_descriptor_close_failure_retains_candidates(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_rename = ls_receipts._rename_directory_no_replace
            original_close = os.close
            published_descriptors: set[int] = set()
            failed = False

            def capture_published_descriptor(
                node_root: ls_receipts._PinnedDirectory,
                source: ls_receipts._PinnedDirectory,
                destination_name: str,
                *,
                candidate: ls_receipts._PublicationCandidate | None = None,
            ) -> ls_receipts._PinnedDirectory:
                published = original_rename(
                    node_root, source, destination_name, candidate=candidate
                )
                published_descriptors.add(published.descriptor)
                return published

            def close_then_fail_once(descriptor: int) -> None:
                nonlocal failed
                if descriptor in published_descriptors and not failed:
                    failed = True
                    original_close(descriptor)
                    raise OSError("injected published descriptor close failure")
                original_close(descriptor)

            with (
                mock.patch.object(
                    ls_receipts,
                    "_rename_directory_no_replace",
                    capture_published_descriptor,
                ),
                mock.patch.object(ls_receipts.os, "close", close_then_fail_once),
                self.assertRaises(ls_receipts.PartialPublicationError) as caught,
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertTrue(failed)
            self.assertEqual(len(caught.exception.candidates), len(NODE_BINDINGS))
            self.assert_candidates_match_visible_attempts(
                caught.exception, repository_root, proof_slices
            )

    def test_stage_cleanup_failure_after_publication_retains_candidates(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"

            with (
                mock.patch.object(
                    ls_receipts,
                    "_remove_stage",
                    side_effect=protocol.ValidationError(
                        "injected post-publication stage cleanup failure"
                    ),
                ),
                self.assertRaises(ls_receipts.PartialPublicationError) as caught,
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertEqual(len(caught.exception.candidates), len(NODE_BINDINGS))
            self.assert_candidates_match_visible_attempts(
                caught.exception, repository_root, proof_slices
            )

    def test_stage_cleanup_failure_does_not_hide_publication_failure(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_fsync = ls_receipts._fsync_descriptor
            failed = False

            def fail_after_first_rename(descriptor: int, label: str) -> None:
                nonlocal failed
                if label == "LS node root after publication" and not failed:
                    failed = True
                    raise protocol.ValidationError(
                        "injected post-rename publication failure"
                    )
                original_fsync(descriptor, label)

            with (
                mock.patch.object(
                    ls_receipts, "_fsync_descriptor", fail_after_first_rename
                ),
                mock.patch.object(
                    ls_receipts,
                    "_remove_stage",
                    side_effect=protocol.ValidationError(
                        "injected post-publication stage cleanup failure"
                    ),
                ),
                self.assertRaises(ls_receipts.PartialPublicationError) as caught,
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertTrue(failed)
            self.assertIn(
                "injected post-publication stage cleanup failure",
                str(caught.exception),
            )
            self.assertEqual(len(caught.exception.candidates), 1)
            self.assert_candidates_match_visible_attempts(
                caught.exception, repository_root, proof_slices
            )

    def test_lock_release_failure_after_publication_retains_candidates(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_release = ls_receipts._release_publication_lock

            def release_then_fail(lock: ls_receipts._PublicationLock) -> None:
                original_release(lock)
                raise protocol.ValidationError(
                    "injected post-publication lock release failure"
                )

            with (
                mock.patch.object(
                    ls_receipts, "_release_publication_lock", release_then_fail
                ),
                self.assertRaises(ls_receipts.PartialPublicationError) as caught,
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertEqual(len(caught.exception.candidates), len(NODE_BINDINGS))
            self.assert_candidates_match_visible_attempts(
                caught.exception, repository_root, proof_slices
            )

    def test_outer_descriptor_close_failure_after_publication_retains_candidates(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_close_directory = ls_receipts._close_pinned_directory
            failed = False

            def close_then_fail_once(directory: ls_receipts._PinnedDirectory) -> None:
                nonlocal failed
                original_close_directory(directory)
                if directory.path == formal_target_root and not failed:
                    failed = True
                    raise protocol.ValidationError(
                        "injected target descriptor close failure"
                    )

            with (
                mock.patch.object(
                    ls_receipts, "_close_pinned_directory", close_then_fail_once
                ),
                self.assertRaises(ls_receipts.PartialPublicationError) as caught,
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertTrue(failed)
            self.assertEqual(len(caught.exception.candidates), len(NODE_BINDINGS))
            self.assert_candidates_match_visible_attempts(
                caught.exception, repository_root, proof_slices
            )

    def test_parent_fsync_failure_after_rename_preserves_complete_candidate(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_fsync = ls_receipts._fsync_descriptor
            failed = False

            def fail_after_rename(descriptor: int, label: str) -> None:
                nonlocal failed
                if label == "LS node root after publication" and not failed:
                    failed = True
                    raise protocol.ValidationError(
                        "injected destination-parent fsync failure"
                    )
                original_fsync(descriptor, label)

            with (
                mock.patch.object(ls_receipts, "_fsync_descriptor", fail_after_rename),
                self.assertRaisesRegex(
                    protocol.ValidationError,
                    "committed partial publication.*attempt-001",
                ),
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            candidate = proof_slices / "ls-equation-one-terminal-bound" / "attempt-001"
            self.assertTrue(failed)
            self.assertEqual(
                {
                    path.relative_to(candidate).as_posix()
                    for path in candidate.rglob("*")
                    if path.is_file()
                },
                EXPECTED_MEMBERS,
            )
            self.assertEqual(list(proof_slices.glob("*/.attempt-*")), [])

    def test_later_node_rename_failure_preserves_earlier_visible_candidate(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_rename = ls_receipts._rename_directory_no_replace
            rename_count = 0

            def fail_second_rename(
                node_root: ls_receipts._PinnedDirectory,
                source: ls_receipts._PinnedDirectory,
                destination_name: str,
                *,
                candidate: ls_receipts._PublicationCandidate | None = None,
            ) -> ls_receipts._PinnedDirectory:
                nonlocal rename_count
                rename_count += 1
                if rename_count == 2:
                    raise protocol.ValidationError("injected later rename failure")
                return original_rename(
                    node_root, source, destination_name, candidate=candidate
                )

            with (
                mock.patch.object(
                    ls_receipts,
                    "_rename_directory_no_replace",
                    fail_second_rename,
                ),
                self.assertRaisesRegex(
                    protocol.ValidationError,
                    "committed partial publication.*attempt-001",
                ),
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            visible = sorted(proof_slices.glob("*/attempt-001"))
            self.assertEqual(
                [attempt.parent.name for attempt in visible],
                ["ls-equation-one-terminal-bound"],
            )
            self.assertEqual(
                {
                    path.relative_to(visible[0]).as_posix()
                    for path in visible[0].rglob("*")
                    if path.is_file()
                },
                EXPECTED_MEMBERS,
            )
            self.assertEqual(list(proof_slices.glob("*/.attempt-*")), [])

    def test_rename_that_commits_then_raises_preserves_candidate_metadata(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_rename = ls_receipts._rename_directory_no_replace
            raised = False

            def rename_then_raise(
                node_root: ls_receipts._PinnedDirectory,
                source: ls_receipts._PinnedDirectory,
                destination_name: str,
                *,
                candidate: ls_receipts._PublicationCandidate | None = None,
            ) -> ls_receipts._PinnedDirectory:
                nonlocal raised
                published = original_rename(
                    node_root, source, destination_name, candidate=candidate
                )
                if not raised:
                    raised = True
                    raise protocol.ValidationError(
                        "injected exception after committed rename"
                    )
                return published

            with (
                mock.patch.object(
                    ls_receipts, "_rename_directory_no_replace", rename_then_raise
                ),
                self.assertRaises(ls_receipts.PartialPublicationError) as caught,
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertTrue(raised)
            self.assertEqual(len(caught.exception.candidates), 1)
            self.assert_candidates_match_visible_attempts(
                caught.exception, repository_root, proof_slices
            )
            detached = list(proof_slices.glob("*/.abandoned-*"))
            self.assertEqual(len(detached), len(NODE_BINDINGS) - 1)
            for attempt in detached:
                self.assertEqual(
                    {
                        path.relative_to(attempt).as_posix()
                        for path in attempt.rglob("*")
                        if path.is_file()
                    },
                    EXPECTED_MEMBERS,
                )

    def test_rename_that_mutates_commits_then_raises_revalidates_candidate(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_rename = ls_receipts._rename_directory_no_replace
            tampered_bytes = b"tampered after committed rename\n"

            def rename_mutate_then_raise(
                node_root: ls_receipts._PinnedDirectory,
                source: ls_receipts._PinnedDirectory,
                destination_name: str,
                *,
                candidate: ls_receipts._PublicationCandidate | None = None,
            ) -> ls_receipts._PinnedDirectory:
                published = original_rename(
                    node_root, source, destination_name, candidate=candidate
                )
                (published.path / "build/stdout.log").write_bytes(tampered_bytes)
                raise protocol.ValidationError(
                    "injected exception after mutated committed rename"
                )

            with (
                mock.patch.object(
                    ls_receipts,
                    "_rename_directory_no_replace",
                    rename_mutate_then_raise,
                ),
                self.assertRaises(ls_receipts.PartialPublicationError) as caught,
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertIn(
                "member bytes changed: build/stdout.log", str(caught.exception)
            )
            self.assertEqual(len(caught.exception.candidates), 1)
            self.assert_candidates_match_visible_attempts(
                caught.exception,
                repository_root,
                proof_slices,
                expect_valid_members=False,
            )
            attempt = repository_root / caught.exception.candidates[0]["attempt_path"]
            self.assertEqual(
                (attempt / "build/stdout.log").read_bytes(), tampered_bytes
            )

    def test_hidden_candidate_made_visible_during_detach_is_reported(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_rename = ls_receipts._rename_directory_no_replace
            original_detach = ls_receipts._detach_hidden_attempt
            rename_calls = 0
            raced = False

            def fail_second_rename(
                node_root: ls_receipts._PinnedDirectory,
                source: ls_receipts._PinnedDirectory,
                destination_name: str,
                *,
                candidate: ls_receipts._PublicationCandidate | None = None,
            ) -> ls_receipts._PinnedDirectory:
                nonlocal rename_calls
                rename_calls += 1
                if rename_calls == 2:
                    raise protocol.ValidationError("injected later rename failure")
                return original_rename(
                    node_root, source, destination_name, candidate=candidate
                )

            def publish_during_detach(
                candidate: ls_receipts._PublicationCandidate,
            ) -> None:
                nonlocal raced
                if not raced:
                    raced = True
                    published = original_rename(
                        candidate.node_root,
                        candidate.hidden_attempt,
                        candidate.plan.attempt_name,
                        candidate=candidate,
                    )
                    (published.path / "task.json").write_bytes(b"tampered in race\n")
                    raise protocol.ValidationError(
                        "injected visibility change during detach"
                    )
                original_detach(candidate)

            with (
                mock.patch.object(
                    ls_receipts, "_rename_directory_no_replace", fail_second_rename
                ),
                mock.patch.object(
                    ls_receipts, "_detach_hidden_attempt", publish_during_detach
                ),
                self.assertRaises(ls_receipts.PartialPublicationError) as caught,
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertTrue(raced)
            self.assertIn("member bytes changed: task.json", str(caught.exception))
            self.assertEqual(len(caught.exception.candidates), 2)
            self.assert_candidates_match_visible_attempts(
                caught.exception,
                repository_root,
                proof_slices,
                expect_valid_members=False,
            )

    def test_mid_copy_failure_never_exposes_incomplete_attempt_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_write = ls_receipts._write_bytes_create_only_at
            publication_writes = 0
            visible_attempts: list[list[Path]] = []

            def fail_during_publication(
                directory_descriptor: int, name: str, data: bytes, label: str
            ) -> None:
                nonlocal publication_writes
                publication_writes += 1
                visible_attempts.append(list(proof_slices.glob("*/attempt-*")))
                if publication_writes == 3:
                    raise protocol.ValidationError("injected mid-copy failure")
                original_write(directory_descriptor, name, data, label)

            with mock.patch.object(
                ls_receipts, "_write_bytes_create_only_at", fail_during_publication
            ):
                with self.assertRaisesRegex(
                    protocol.ValidationError, "injected mid-copy failure"
                ):
                    ls_receipts.publish_ls_receipts(
                        make_rows(),
                        repository_root,
                        formal_target_root,
                        executor=FakeExecutor(),
                    )

            self.assertTrue(visible_attempts)
            self.assertTrue(all(not paths for paths in visible_attempts))
            self.assertEqual(list(proof_slices.glob("*/attempt-*")), [])
            self.assertEqual(list(proof_slices.glob("*/.attempt-*")), [])

    def test_identical_byte_module_replacement_before_rename_rolls_back(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            _, build_target = NODE_BINDINGS["ls-equation-one-terminal-bound"]
            module_path = repository_root / build_target
            original_copy = ls_receipts._copy_bundle_create_only
            replaced = False

            def copy_then_replace(
                source: Path,
                node_root: ls_receipts._PinnedDirectory,
                destination_name: str,
            ) -> ls_receipts._PinnedDirectory:
                nonlocal replaced
                destination = original_copy(source, node_root, destination_name)
                if not replaced:
                    replacement = module_path.with_suffix(".replacement")
                    replacement.write_bytes(module_path.read_bytes())
                    replacement.replace(module_path)
                    replaced = True
                return destination

            with mock.patch.object(
                ls_receipts, "_copy_bundle_create_only", copy_then_replace
            ):
                with self.assertRaisesRegex(
                    protocol.ValidationError, "module.*changed"
                ):
                    ls_receipts.publish_ls_receipts(
                        make_rows(),
                        repository_root,
                        formal_target_root,
                        executor=FakeExecutor(),
                    )

            self.assertTrue(replaced)
            self.assertEqual(list(proof_slices.glob("*/attempt-*")), [])
            self.assertEqual(list(proof_slices.glob("*/.attempt-*")), [])

    def test_identical_byte_module_replacement_after_final_rename_keeps_candidates(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            _, build_target = NODE_BINDINGS["ls-terminal-crouzeix"]
            module_path = repository_root / build_target
            original_read = ls_receipts._read_safe_file_at
            replaced = False

            def read_then_replace(
                directory_descriptor: int, name: str, label: str, maximum: int
            ) -> bytes:
                nonlocal replaced
                data = original_read(directory_descriptor, name, label, maximum)
                if (
                    not replaced
                    and name == "receipt.json"
                    and "ls-terminal-crouzeix" in label
                ):
                    replacement = module_path.with_suffix(".replacement")
                    replacement.write_bytes(module_path.read_bytes())
                    replacement.replace(module_path)
                    replaced = True
                return data

            with mock.patch.object(
                ls_receipts, "_read_safe_file_at", read_then_replace
            ):
                with self.assertRaisesRegex(
                    protocol.ValidationError, "committed partial publication"
                ):
                    ls_receipts.publish_ls_receipts(
                        make_rows(),
                        repository_root,
                        formal_target_root,
                        executor=FakeExecutor(),
                    )

            self.assertTrue(replaced)
            self.assertEqual(
                len(list(proof_slices.glob("*/attempt-*"))), len(NODE_BINDINGS)
            )
            self.assertEqual(list(proof_slices.glob("*/.attempt-*")), [])

    def test_attempt_numbers_are_monotonic_and_history_is_never_overwritten(
        self,
    ) -> None:
        rows = make_rows()
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            sentinels = {}
            for node_id in NODE_BINDINGS:
                attempt = proof_slices / node_id / "attempt-001"
                attempt.mkdir(parents=True)
                sentinel = attempt / "history.txt"
                sentinel.write_bytes(f"history:{node_id}\n".encode())
                sentinels[sentinel] = sentinel.read_bytes()
            gap_node = proof_slices / "ls-power-recurrence" / "attempt-003"
            gap_node.mkdir()
            gap_sentinel = gap_node / "history.txt"
            gap_sentinel.write_bytes(b"gap history\n")
            sentinels[gap_sentinel] = gap_sentinel.read_bytes()

            first = ls_receipts.publish_ls_receipts(
                rows, repository_root, formal_target_root, executor=FakeExecutor()
            )
            second = ls_receipts.publish_ls_receipts(
                rows, repository_root, formal_target_root, executor=FakeExecutor()
            )

            self.assertTrue(
                first["ls-power-recurrence"]["attempt_path"].endswith("attempt-004")
            )
            self.assertTrue(
                second["ls-power-recurrence"]["attempt_path"].endswith("attempt-005")
            )
            for node_id in NODE_BINDINGS:
                if node_id == "ls-power-recurrence":
                    continue
                self.assertTrue(first[node_id]["attempt_path"].endswith("attempt-002"))
                self.assertTrue(second[node_id]["attempt_path"].endswith("attempt-003"))
            for sentinel, original in sentinels.items():
                self.assertEqual(sentinel.read_bytes(), original)
            historical_digests = {
                row.receipt_sha256 for row in rows if row.receipt_sha256 is not None
            }
            self.assertTrue(
                all(
                    publication["receipt_sha256"] not in historical_digests
                    for publication in (*first.values(), *second.values())
                )
            )

    def test_wrapper_policy_rejects_active_update_and_cache_get_commands(self) -> None:
        unsafe_wrappers = (
            SAFE_WRAPPER + "lake update\n",
            SAFE_WRAPPER + "cache   get\n",
            SAFE_WRAPPER + "lake " + chr(92) + "\n  update\n",
            SAFE_WRAPPER + "cache " + chr(92) + "\n  get\n",
        )
        for wrapper in unsafe_wrappers:
            with self.subTest(wrapper=wrapper):
                with tempfile.TemporaryDirectory() as directory:
                    repository_root, formal_target_root = make_workspace(
                        Path(directory).resolve()
                    )
                    (repository_root / "scripts/check_lean_library.sh").write_text(
                        wrapper, encoding="utf-8"
                    )
                    executor = FakeExecutor()

                    with self.assertRaisesRegex(
                        protocol.ValidationError, "cache policy"
                    ):
                        ls_receipts.publish_ls_receipts(
                            make_rows(),
                            repository_root,
                            formal_target_root,
                            executor=executor,
                        )

                    self.assertEqual(executor.calls, [])

        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            (repository_root / "scripts/check_lean_library.sh").write_text(
                "#!/bin/sh\n# lake update\n# cache get\nexit 0\n",
                encoding="utf-8",
            )
            executor = FakeExecutor()

            ls_receipts.publish_ls_receipts(
                make_rows(),
                repository_root,
                formal_target_root,
                executor=executor,
            )

            self.assertEqual(len(executor.calls), 7)

    def test_wrapper_replacement_during_execution_rolls_back(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            wrapper = repository_root / "scripts/check_lean_library.sh"
            delegate = FakeExecutor()

            def replacing_executor(*args: object) -> ls_receipts.CommandResult:
                result = delegate(*args)
                if len(delegate.calls) == 1:
                    replacement = wrapper.with_suffix(".replacement")
                    replacement.write_bytes(wrapper.read_bytes())
                    replacement.replace(wrapper)
                return result

            with self.assertRaisesRegex(protocol.ValidationError, "wrapper.*changed"):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=replacing_executor,
                )

            self.assertEqual(
                list((formal_target_root / "proof-slices").glob("*/attempt-*")), []
            )

    def test_dependency_cache_mutation_during_execution_rolls_back(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            cache_file = (
                repository_root
                / "formalization/lean/.lake/packages/batteries/.lake/build/lib/lean"
                / "Batteries.olean"
            )
            delegate = FakeExecutor()

            def mutating_executor(*args: object) -> ls_receipts.CommandResult:
                result = delegate(*args)
                if len(delegate.calls) == 1:
                    cache_file.write_bytes(b"changed dependency cache\n")
                return result

            with self.assertRaisesRegex(
                protocol.ValidationError, "dependency cache.*changed"
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=mutating_executor,
                )

            self.assertEqual(
                list((formal_target_root / "proof-slices").glob("*/attempt-*")), []
            )

    def test_required_mathlib_artifact_content_mutation_rolls_back(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            artifact = (
                repository_root
                / "formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean"
                / Path(*REQUIRED_MATHLIB_MODULE.split(".")).with_suffix(".olean")
            )
            delegate = FakeExecutor()

            def mutating_executor(*args: object) -> ls_receipts.CommandResult:
                result = delegate(*args)
                if len(delegate.calls) == 1:
                    data = artifact.read_bytes()
                    artifact.write_bytes(data[:-2] + b"X\n")
                return result

            with self.assertRaisesRegex(
                protocol.ValidationError, "required Mathlib artifacts.*changed"
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=mutating_executor,
                )

            self.assertEqual(
                list((formal_target_root / "proof-slices").glob("*/attempt-*")), []
            )

    def test_rejects_symlink_below_approved_lake_boundary(self) -> None:
        for relative in (
            "formalization/lean/.lake/packages",
            "formalization/lean/.lake/packages/mathlib/.lake/build",
        ):
            with self.subTest(relative=relative):
                with tempfile.TemporaryDirectory() as directory:
                    root = Path(directory).resolve()
                    repository_root, formal_target_root = make_workspace(root)
                    target = repository_root / relative
                    outside = root / "outside-cache"
                    target.rename(outside)
                    target.symlink_to(outside, target_is_directory=True)
                    executor = FakeExecutor()

                    with self.assertRaisesRegex(
                        protocol.ValidationError, "cache.*symlink|cannot be a symlink"
                    ):
                        ls_receipts.publish_ls_receipts(
                            make_rows(),
                            repository_root,
                            formal_target_root,
                            executor=executor,
                        )

                    self.assertEqual(executor.calls, [])

    def test_project_build_output_may_change(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            project_output = (
                repository_root
                / "formalization/lean/.lake/build/lib/lean/Project.olean"
            )
            delegate = FakeExecutor()

            def mutating_executor(*args: object) -> ls_receipts.CommandResult:
                result = delegate(*args)
                if len(delegate.calls) == 1:
                    project_output.write_bytes(b"expected changed project output\n")
                return result

            published = ls_receipts.publish_ls_receipts(
                make_rows(),
                repository_root,
                formal_target_root,
                executor=mutating_executor,
            )

            self.assertEqual(set(published), set(NODE_BINDINGS))

    def test_rejects_direct_and_transitive_provider_leaks_before_execution(
        self,
    ) -> None:
        for leak_kind in ("direct", "transitive"):
            with self.subTest(leak_kind=leak_kind):
                with tempfile.TemporaryDirectory() as directory:
                    repository_root, formal_target_root = make_workspace(
                        Path(directory).resolve()
                    )
                    target = (
                        repository_root
                        / NODE_BINDINGS["ls-equation-one-terminal-bound"][1]
                    )
                    if leak_kind == "direct":
                        target.write_text(
                            "import Crouzeix.Jin.Terminal\n", encoding="utf-8"
                        )
                    else:
                        target.write_text("import Fixture.Helper\n", encoding="utf-8")
                        helper = (
                            repository_root / "formalization/lean/Fixture/Helper.lean"
                        )
                        helper.parent.mkdir(parents=True)
                        helper.write_text(
                            "module\npublic import CrouzeixConjecture.FinalTheorems\n",
                            encoding="utf-8",
                        )
                    executor = FakeExecutor()

                    with self.assertRaisesRegex(
                        protocol.ValidationError, "forbidden module"
                    ):
                        ls_receipts.publish_ls_receipts(
                            make_rows(),
                            repository_root,
                            formal_target_root,
                            executor=executor,
                        )

                    self.assertEqual(executor.calls, [])

    def test_rejects_aggregate_only_provider_leak_before_execution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            aggregate = (
                repository_root / "formalization/lean/CrouzeixLoristSchwenninger.lean"
            )
            aggregate.write_text("import Fixture.AggregateOnly\n", encoding="utf-8")
            helper = repository_root / "formalization/lean/Fixture/AggregateOnly.lean"
            helper.parent.mkdir(parents=True)
            helper.write_text(
                "import CrouzeixConjecture.FinalTheorems\n", encoding="utf-8"
            )
            executor = FakeExecutor()

            with self.assertRaisesRegex(protocol.ValidationError, "forbidden module"):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=executor,
                )

            self.assertEqual(executor.calls, [])

    def test_rejects_missing_transitive_managed_local_import_before_execution(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            target = (
                repository_root / NODE_BINDINGS["ls-equation-one-terminal-bound"][1]
            )
            target.write_text("import Fixture.Helper\n", encoding="utf-8")
            helper = repository_root / "formalization/lean/Fixture/Helper.lean"
            helper.parent.mkdir(parents=True)
            helper.write_text(
                "import CrouzeixConjecture.Missing\n",
                encoding="utf-8",
            )
            executor = FakeExecutor()

            with self.assertRaisesRegex(
                protocol.ValidationError, "local module does not exist"
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=executor,
                )

            self.assertEqual(executor.calls, [])

    def test_rejects_harp_namespace_prefix_before_execution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            target = (
                repository_root / NODE_BINDINGS["ls-equation-one-terminal-bound"][1]
            )
            target.write_text("import Crouzeix.Harp.Support\n", encoding="utf-8")
            executor = FakeExecutor()

            with self.assertRaisesRegex(
                protocol.ValidationError, "forbidden module Crouzeix.Harp.Support"
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=executor,
                )

            self.assertEqual(executor.calls, [])

    def test_imported_local_module_mutation_rolls_back(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            target = (
                repository_root / NODE_BINDINGS["ls-equation-one-terminal-bound"][1]
            )
            target.write_text(
                f"import Fixture.Helper\nimport {REQUIRED_MATHLIB_MODULE}\n",
                encoding="utf-8",
            )
            helper = repository_root / "formalization/lean/Fixture/Helper.lean"
            helper.parent.mkdir(parents=True)
            helper.write_text("def fixtureValue := 1\n", encoding="utf-8")
            delegate = FakeExecutor()

            def mutating_executor(*args: object) -> ls_receipts.CommandResult:
                result = delegate(*args)
                if len(delegate.calls) == 1:
                    helper.write_text("def fixtureValue := 2\n", encoding="utf-8")
                return result

            with self.assertRaisesRegex(
                protocol.ValidationError, "module.*changed|source.*changed"
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=mutating_executor,
                )

            self.assertEqual(
                list((formal_target_root / "proof-slices").glob("*/attempt-*")), []
            )

    def test_required_mathlib_discovery_accepts_public_import(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, _ = make_workspace(Path(directory).resolve())
            for _, build_target in NODE_BINDINGS.values():
                (repository_root / build_target).write_text(
                    "-- no Mathlib import in this fixture module\n",
                    encoding="utf-8",
                )
            target = (
                repository_root / NODE_BINDINGS["ls-equation-one-terminal-bound"][1]
            )
            target.write_text(
                f"module\npublic import {REQUIRED_MATHLIB_MODULE}\n",
                encoding="utf-8",
            )

            self.assertIn(
                REQUIRED_MATHLIB_MODULE,
                ls_receipts._required_mathlib_modules(
                    ls_receipts._local_module_snapshots(
                        repository_root,
                        tuple(
                            ls_receipts._build_target_module(target)
                            for _, target in NODE_BINDINGS.values()
                        ),
                    )
                ),
            )

    def test_required_mathlib_discovery_uses_transitive_closure_outside_crouzeix(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            target = (
                repository_root / NODE_BINDINGS["ls-equation-one-terminal-bound"][1]
            )
            target.write_text("import Fixture.Helper\n", encoding="utf-8")
            helper = repository_root / "formalization/lean/Fixture/Helper.lean"
            helper.parent.mkdir(parents=True)
            helper.write_text(
                f"module\npublic import {REQUIRED_MATHLIB_MODULE}\n",
                encoding="utf-8",
            )

            published = ls_receipts.publish_ls_receipts(
                make_rows(),
                repository_root,
                formal_target_root,
                executor=FakeExecutor(),
            )

            command_digests = set()
            closure_digests = set()
            for publication in published.values():
                attempt = repository_root / publication["attempt_path"]
                receipt = json.loads(
                    (attempt / "receipt.json").read_text(encoding="utf-8")
                )
                command = json.loads(
                    (attempt / "build/command.json").read_text(encoding="utf-8")
                )
                command_digests.add(receipt["command_sha256"])
                closure_digests.add(command["local_source_closure_sha256"])
            self.assertEqual(len(command_digests), 1)
            self.assertEqual(len(closure_digests), 1)

    def test_rejects_hardlinked_local_source_before_execution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            repository_root, formal_target_root = make_workspace(root)
            source = (
                repository_root / NODE_BINDINGS["ls-equation-one-terminal-bound"][1]
            )
            os.link(source, root / "source-alias.lean")
            executor = FakeExecutor()

            with self.assertRaisesRegex(
                protocol.ValidationError, "exactly one hard link"
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=executor,
                )

            self.assertEqual(executor.calls, [])

    def test_atomic_no_replace_race_preserves_competing_destination(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_publish = ls_receipts._rename_directory_no_replace
            injected = False

            def race(
                node_root: ls_receipts._PinnedDirectory,
                source: ls_receipts._PinnedDirectory,
                destination_name: str,
                *,
                candidate: ls_receipts._PublicationCandidate | None = None,
            ) -> ls_receipts._PinnedDirectory:
                nonlocal injected
                if not injected:
                    os.mkdir(destination_name, dir_fd=node_root.descriptor)
                    destination = os.open(
                        destination_name,
                        os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                        dir_fd=node_root.descriptor,
                    )
                    try:
                        competitor = os.open(
                            "competitor.txt",
                            os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                            0o600,
                            dir_fd=destination,
                        )
                        try:
                            os.write(competitor, b"competitor\n")
                        finally:
                            os.close(competitor)
                    finally:
                        os.close(destination)
                    injected = True
                return original_publish(
                    node_root, source, destination_name, candidate=candidate
                )

            with (
                mock.patch.object(ls_receipts, "_rename_directory_no_replace", race),
                self.assertRaisesRegex(
                    protocol.ValidationError, "appeared during publication"
                ),
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            competitor = (
                proof_slices
                / "ls-equation-one-terminal-bound/attempt-001/competitor.txt"
            )
            self.assertTrue(injected)
            self.assertEqual(competitor.read_text(encoding="utf-8"), "competitor\n")
            self.assertEqual(list(proof_slices.glob("*/.attempt-*")), [])

    def test_node_root_swap_after_validation_cannot_redirect_publication(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            repository_root, formal_target_root = make_workspace(root)
            proof_slices = formal_target_root / "proof-slices"
            node_root = proof_slices / "ls-equation-one-terminal-bound"
            displaced = root / "displaced-node-root"
            outside = root / "outside"
            outside.mkdir()
            original_copy = ls_receipts._copy_bundle_create_only
            swapped = False
            wrote_outside = False

            def swap_then_copy(*args: object, **kwargs: object) -> object:
                nonlocal swapped, wrote_outside
                if not swapped:
                    node_root.rename(displaced)
                    node_root.symlink_to(outside, target_is_directory=True)
                    swapped = True
                result = original_copy(*args, **kwargs)
                wrote_outside = any(outside.iterdir())
                return result

            with (
                mock.patch.object(
                    ls_receipts, "_copy_bundle_create_only", swap_then_copy
                ),
                self.assertRaises(protocol.ValidationError),
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertTrue(swapped)
            self.assertFalse(wrote_outside)
            self.assertEqual(list(outside.iterdir()), [])

    def test_stale_hidden_attempt_does_not_block_publication(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            stale = (
                formal_target_root
                / "proof-slices/ls-equation-one-terminal-bound/.attempt-001.staging"
            )
            stale.mkdir(parents=True)
            sentinel = stale / "sentinel"
            sentinel.write_text("abandoned\n", encoding="utf-8")

            published = ls_receipts.publish_ls_receipts(
                make_rows(),
                repository_root,
                formal_target_root,
                executor=FakeExecutor(),
            )

            self.assertEqual(sentinel.read_text(encoding="utf-8"), "abandoned\n")
            self.assertTrue(
                published["ls-equation-one-terminal-bound"]["attempt_path"].endswith(
                    "attempt-001"
                )
            )

    def test_stage_cleanup_cannot_follow_swapped_target_ancestor(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            target = root / "target"
            target.mkdir()
            stage = target / f"{ls_receipts.STAGE_PREFIX}fixture"
            stage.mkdir()
            (stage / "owned").write_text("owned\n", encoding="utf-8")
            displaced = root / "displaced"
            target.rename(displaced)
            outside = root / "outside"
            outside_stage = outside / stage.name
            outside_stage.mkdir(parents=True)
            sentinel = outside_stage / "sentinel"
            sentinel.write_text("outside\n", encoding="utf-8")
            target.symlink_to(outside, target_is_directory=True)

            with self.assertRaises(protocol.ValidationError):
                ls_receipts._remove_stage(stage, target)

            self.assertEqual(sentinel.read_text(encoding="utf-8"), "outside\n")

    def test_cleanup_failure_still_releases_publication_lock(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            delegate = FakeExecutor(aggregate_exit_code=1)

            with mock.patch.object(
                ls_receipts,
                "_remove_stage",
                side_effect=protocol.ValidationError("injected cleanup failure"),
            ):
                with self.assertRaises(BaseException) as caught:
                    ls_receipts.publish_ls_receipts(
                        make_rows(),
                        repository_root,
                        formal_target_root,
                        executor=delegate,
                    )

            messages = exception_messages(caught.exception)
            self.assertTrue(any("aggregate build failed" in item for item in messages))
            self.assertTrue(
                any("injected cleanup failure" in item for item in messages)
            )
            lock = ls_receipts._acquire_publication_lock(proof_slices)
            ls_receipts._release_publication_lock(lock)

    def test_malformed_graph_row_is_rejected_before_lock_or_execution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            rows = list(make_rows())
            rows[0] = replace(rows[0], statement_sha256="not-a-digest")
            executor = FakeExecutor()

            with (
                mock.patch.object(ls_receipts, "_acquire_publication_lock") as acquire,
                self.assertRaisesRegex(protocol.ValidationError, "statement_sha256"),
            ):
                ls_receipts.publish_ls_receipts(
                    tuple(rows),
                    repository_root,
                    formal_target_root,
                    executor=executor,
                )

            acquire.assert_not_called()
            self.assertEqual(executor.calls, [])
            self.assertEqual(list((formal_target_root / "proof-slices").iterdir()), [])

    def test_incomplete_or_reordered_graph_rows_are_rejected_before_side_effects(
        self,
    ) -> None:
        cases = (
            make_rows()[:-1],
            tuple(reversed(make_rows())),
        )
        for rows in cases:
            with self.subTest(rows=rows):
                with tempfile.TemporaryDirectory() as directory:
                    repository_root, formal_target_root = make_workspace(
                        Path(directory).resolve()
                    )
                    executor = FakeExecutor()
                    with (
                        mock.patch.object(
                            ls_receipts, "_acquire_publication_lock"
                        ) as acquire,
                        self.assertRaises(protocol.ValidationError),
                    ):
                        ls_receipts.publish_ls_receipts(
                            rows,
                            repository_root,
                            formal_target_root,
                            executor=executor,
                        )
                    acquire.assert_not_called()
                    self.assertEqual(executor.calls, [])

    def test_external_advisory_lock_rejects_concurrent_publisher(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            executor = FakeExecutor()
            publication_lock = ls_receipts._acquire_publication_lock(proof_slices)

            try:
                with self.assertRaisesRegex(
                    protocol.ValidationError, "publisher is active"
                ):
                    ls_receipts.publish_ls_receipts(
                        make_rows(),
                        repository_root,
                        formal_target_root,
                        executor=executor,
                    )
            finally:
                ls_receipts._release_publication_lock(publication_lock)

            self.assertEqual(executor.calls, [])
            self.assertEqual(list(proof_slices.glob("*lock*")), [])

    def test_advisory_lock_is_persistent_external_and_release_is_close_only(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            proof_slices = Path(directory).resolve()
            alias = proof_slices / "alias"
            alias.symlink_to(".", target_is_directory=True)
            first = ls_receipts._acquire_publication_lock(proof_slices)

            self.assertEqual(
                first.path.parent, ls_receipts.PUBLICATION_LOCK_ROOT.resolve()
            )
            self.assertTrue(first.path.is_file())
            with self.assertRaisesRegex(
                protocol.ValidationError, "publisher is active"
            ):
                ls_receipts._acquire_publication_lock(alias)

            with (
                mock.patch.object(Path, "unlink") as path_unlink,
                mock.patch.object(os, "unlink") as os_unlink,
            ):
                ls_receipts._release_publication_lock(first)
            path_unlink.assert_not_called()
            os_unlink.assert_not_called()

            second = ls_receipts._acquire_publication_lock(alias)
            self.assertEqual(second.path, first.path)
            ls_receipts._release_publication_lock(second)
            self.assertTrue(first.path.is_file())

    def test_all_module_snapshots_are_checked_before_first_final_rename(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            _, build_target = NODE_BINDINGS["ls-terminal-crouzeix"]
            module_path = repository_root / build_target
            original_copy = ls_receipts._copy_bundle_create_only
            rename_calls: list[tuple[str, str]] = []
            replaced = False

            def copy_then_replace(
                source: Path,
                node_root: ls_receipts._PinnedDirectory,
                destination_name: str,
            ) -> ls_receipts._PinnedDirectory:
                nonlocal replaced
                destination = original_copy(source, node_root, destination_name)
                if not replaced:
                    replacement = module_path.with_suffix(".replacement")
                    replacement.write_bytes(module_path.read_bytes())
                    replacement.replace(module_path)
                    replaced = True
                return destination

            original_rename = ls_receipts._rename_directory_no_replace

            def record_rename(
                node_root: ls_receipts._PinnedDirectory,
                source: ls_receipts._PinnedDirectory,
                destination_name: str,
                *,
                candidate: ls_receipts._PublicationCandidate | None = None,
            ) -> ls_receipts._PinnedDirectory:
                rename_calls.append((source.path.name, destination_name))
                return original_rename(
                    node_root, source, destination_name, candidate=candidate
                )

            with (
                mock.patch.object(
                    ls_receipts, "_copy_bundle_create_only", copy_then_replace
                ),
                mock.patch.object(
                    ls_receipts, "_rename_directory_no_replace", record_rename
                ),
            ):
                with self.assertRaisesRegex(
                    protocol.ValidationError, "module.*changed"
                ):
                    ls_receipts.publish_ls_receipts(
                        make_rows(),
                        repository_root,
                        formal_target_root,
                        executor=FakeExecutor(),
                    )

            self.assertTrue(replaced)
            self.assertEqual(rename_calls, [])
            self.assertEqual(list(proof_slices.glob("*/attempt-*")), [])
            self.assertEqual(list(proof_slices.glob("*/.attempt-*")), [])

    def test_new_node_root_fsync_failure_is_cleaned_before_publication(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            proof_slices = formal_target_root / "proof-slices"
            original_fsync = ls_receipts._fsync_descriptor
            labels: list[str] = []

            def fail_proof_slices_fsync(descriptor: int, label: str) -> None:
                labels.append(label)
                if label == "LS proof-slices root after node creation":
                    raise protocol.ValidationError(
                        "injected proof-slices fsync failure"
                    )
                original_fsync(descriptor, label)

            with (
                mock.patch.object(
                    ls_receipts,
                    "_fsync_descriptor",
                    fail_proof_slices_fsync,
                ),
                self.assertRaisesRegex(
                    protocol.ValidationError, "proof-slices fsync failure"
                ),
            ):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=FakeExecutor(),
                )

            self.assertIn("LS proof-slices root after node creation", labels)
            self.assertEqual(list(proof_slices.iterdir()), [])

    def test_subprocess_executor_terminates_descendant_that_retains_pipes(
        self,
    ) -> None:
        with (
            mock.patch.object(ls_receipts, "PIPE_DRAIN_GRACE_SECONDS", 0.05),
            mock.patch.object(
                ls_receipts,
                "_terminate_process_tree",
                wraps=ls_receipts._terminate_process_tree,
            ) as terminate,
        ):
            result = ls_receipts._subprocess_executor(
                ["/bin/sh", "-c", "sleep 2 &"],
                Path("/"),
                {"PATH": "/usr/bin:/bin"},
                1,
                1024,
            )

        self.assertEqual(result.exit_code, 0)
        self.assertIsNone(result.blocked_reason)
        self.assertFalse(result.stdout_truncated or result.stderr_truncated)
        terminate.assert_called_once()

    def test_subprocess_executor_kills_descendants_after_successful_parent_exit(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            marker = Path(directory) / "descendant-survived"
            command = ";".join(
                (
                    "import os,time",
                    "pid=os.fork()",
                    "os._exit(0) if pid else None",
                    "null=os.open('/dev/null',os.O_RDWR)",
                    "os.dup2(null,0)",
                    "os.dup2(null,1)",
                    "os.dup2(null,2)",
                    "time.sleep(0.4)",
                    f"open({os.fspath(marker)!r},'wb').write(b'survived')",
                )
            )
            with mock.patch.object(
                ls_receipts,
                "_terminate_process_tree",
                wraps=ls_receipts._terminate_process_tree,
            ) as terminate:
                result = ls_receipts._subprocess_executor(
                    [sys.executable, "-c", command],
                    Path("/"),
                    {"PATH": "/usr/bin:/bin"},
                    2,
                    1024,
                )
            time.sleep(0.7)
            self.assertEqual(result.exit_code, 0)
            self.assertIsNone(result.blocked_reason)
            terminate.assert_called_once()
            self.assertFalse(marker.exists())

    def test_complete_attempts_are_candidates_until_graph_digest_update(self) -> None:
        rows = make_rows()
        with tempfile.TemporaryDirectory() as directory:
            repository_root, formal_target_root = make_workspace(
                Path(directory).resolve()
            )
            published = ls_receipts.publish_ls_receipts(
                rows,
                repository_root,
                formal_target_root,
                executor=FakeExecutor(),
            )

            original_digests = {row.node_id: row.receipt_sha256 for row in rows}
            self.assertTrue(
                all(
                    published[row.node_id]["receipt_sha256"]
                    != original_digests[row.node_id]
                    for row in rows
                )
            )
            with self.assertRaisesRegex(
                protocol.ValidationError, "exactly one.*receipt_sha256"
            ):
                ls_validation.validate_committed_receipts(
                    rows, formal_target_root, repository_root
                )
            updated = tuple(
                replace(
                    row,
                    lean_name=published[row.node_id]["lean_name"],
                    status=published[row.node_id]["status"],
                    receipt_sha256=published[row.node_id]["receipt_sha256"],
                    blocked_reason=None,
                    failed_reason=None,
                )
                for row in rows
            )
            self.assertEqual(
                set(
                    ls_validation.validate_committed_receipts(
                        updated, formal_target_root, repository_root
                    )
                ),
                set(NODE_BINDINGS),
            )

    def test_rejects_symlinked_output_ancestor_without_running_commands(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            repository_root, formal_target_root = make_workspace(root)
            proof_slices = formal_target_root / "proof-slices"
            outside = root / "outside"
            outside.mkdir()
            node_root = proof_slices / "ls-equation-one-terminal-bound"
            node_root.symlink_to(outside, target_is_directory=True)
            executor = FakeExecutor()

            with self.assertRaisesRegex(protocol.ValidationError, "symlink"):
                ls_receipts.publish_ls_receipts(
                    make_rows(),
                    repository_root,
                    formal_target_root,
                    executor=executor,
                )

            self.assertEqual(executor.calls, [])
            self.assertEqual(list(outside.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
