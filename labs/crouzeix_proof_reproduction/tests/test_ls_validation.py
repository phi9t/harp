from __future__ import annotations

import json
import os
import shutil
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from unittest import mock


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import ls_validation  # noqa: E402
import ls_receipts  # noqa: E402
import ls_contract  # noqa: E402
import provider_independence  # noqa: E402
import protocol  # noqa: E402


GRAPH = LAB / "formal_targets/lorist-schwenninger/source-graph.json"
INVENTORY = LAB / "formal_targets/lorist-schwenninger/library-inventory.json"
FORMAL_TARGET = LAB / "formal_targets/lorist-schwenninger"
REPOSITORY_ROOT = LAB.parents[1]
V1_FIXTURES = LAB / "fixtures/ls_v1"
VALIDATOR_STDOUT = (
    b"[lean] target=CrouzeixLoristSchwenninger\n"
    b"[lean] root=formalization/lean\n"
    b"[lean] scan_seconds=0\n"
    b"[lean] cache_seconds=0\n"
    b"[lean] lake_seconds=0\n"
    b"[lean] total_seconds=0\n"
    b"[lean] outcome=passed\n"
)
VALIDATOR_MATHLIB_MODULE = "Mathlib.Algebra.Order.Ring.Defs"


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n",
        encoding="utf-8",
    )


def write_json_with_duplicate_key(
    path: Path, value: dict[str, object], key: str
) -> None:
    pairs = ((key, value[key]), *value.items())
    payload = (
        "{"
        + ",".join(
            json.dumps(name, ensure_ascii=False)
            + ":"
            + json.dumps(
                item,
                sort_keys=True,
                separators=(",", ":"),
                ensure_ascii=False,
                allow_nan=False,
            )
            for name, item in pairs
        )
        + "}\n"
    )
    path.write_text(payload, encoding="utf-8")


def copy_committed_receipts(root: Path) -> tuple[Path, Path]:
    repository_root = root / "repository"
    formal_target_root = root / "formal-target"
    shutil.copytree(FORMAL_TARGET, formal_target_root)
    lean_root = REPOSITORY_ROOT / "formalization/lean"
    for source in lean_root.rglob("*.lean"):
        destination = repository_root / source.relative_to(REPOSITORY_ROOT)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(source.read_bytes())
    for node_root in (formal_target_root / "proof-slices").iterdir():
        if not node_root.is_dir():
            continue
        for attempt in node_root.iterdir():
            receipt_path = attempt / "receipt.json"
            if not receipt_path.is_file():
                continue
            receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
            build_target = receipt.get("build_target")
            module_sha256 = receipt.get("module_sha256")
            if not isinstance(build_target, str) or not isinstance(module_sha256, str):
                continue
            module_path = repository_root / build_target
            if not module_path.is_file() or sha256_file(module_path) == module_sha256:
                continue
            fixture = V1_FIXTURES / f"{module_sha256}.lean"
            if not fixture.is_file():
                continue
            historical = fixture.read_bytes()
            if protocol.sha256_bytes(historical) == module_sha256:
                module_path.write_bytes(historical)
    return formal_target_root, repository_root


def historical_passed_rows() -> tuple[ls_validation.LSGraphRow, ...]:
    """Return only rows backed by immutable historical receipt fixtures."""

    return tuple(
        row
        for row in ls_validation.load_route_graph(GRAPH)
        if row.status == "passed" and row.receipt_sha256 is not None
    )


def current_target_overrides(
    formal_target_root: Path,
) -> dict[str, str]:
    targets: dict[str, str] = {}
    for node_root in (formal_target_root / "proof-slices").iterdir():
        if not node_root.is_dir():
            continue
        for attempt in node_root.iterdir():
            receipt_path = attempt / "receipt.json"
            if not receipt_path.is_file():
                continue
            try:
                receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
            except (OSError, UnicodeDecodeError, json.JSONDecodeError):
                continue
            build_target = receipt.get("build_target")
            if isinstance(build_target, str):
                targets[node_root.name] = build_target
    return targets


def validate_fixture_receipts(
    rows: tuple[ls_validation.LSGraphRow, ...],
    formal_target_root: Path,
    repository_root: Path,
) -> dict[str, dict[str, object]]:
    historical_targets = current_target_overrides(formal_target_root)
    aggregate = repository_root / "formalization/lean/Crouzeix.lean"
    historical_imports = []
    for target in historical_targets.values():
        module = Path(target).relative_to("formalization/lean").with_suffix("")
        historical_imports.append("import " + ".".join(module.parts))
    if historical_imports:
        aggregate.write_text(
            aggregate.read_text(encoding="utf-8")
            + "\n"
            + "\n".join(sorted(set(historical_imports)))
            + "\n",
            encoding="utf-8",
        )
    with mock.patch.dict(
        ls_validation.LS_NODE_BUILD_TARGETS,
        historical_targets,
        clear=True,
    ):
        return ls_validation.validate_committed_receipts(
            rows, formal_target_root, repository_root
        )


def validate_receipts(
    rows: tuple[ls_validation.LSGraphRow, ...],
    formal_target_root: Path,
    repository_root: Path,
) -> dict[str, dict[str, object]]:
    """Validate copied immutable attempts against their historical targets."""

    return validate_fixture_receipts(rows, formal_target_root, repository_root)


def replace_graph_row(
    rows: tuple[ls_validation.LSGraphRow, ...],
    node_id: str,
    **changes: object,
) -> tuple[ls_validation.LSGraphRow, ...]:
    return tuple(
        replace(row, **changes) if row.node_id == node_id else row for row in rows
    )


def canonical_rows() -> tuple[ls_validation.LSGraphRow, ...]:
    return tuple(
        ls_validation.LSGraphRow(
            node_id=node.node_id,
            source_locator=node.source_locator,
            statement_sha256=node.statement_sha256,
            lean_name=node.declaration,
            dependencies=node.dependencies,
            role=node.role,
            status="passed",
            receipt_sha256="a" * 64,
        )
        for node in ls_contract.NODES
    )


def write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def write_bytes(path: Path, data: bytes) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)


def make_audit_repo(root: Path, module_bodies: dict[str, bytes | str]) -> Path:
    repository_root = root / "repository"
    for node_id, spec in ls_contract.THEOREM_BODY_AUDITS.items():
        content = module_bodies.get(node_id)
        if content is None:
            raise AssertionError(f"missing audit fixture for {node_id}")
        target = repository_root / str(spec["path"])
        if isinstance(content, bytes):
            write_bytes(target, content)
        else:
            write_text(target, content)
    return repository_root


def valid_theorem_fixture(theorem_name: str, required_calls: tuple[str, ...]) -> str:
    lines = [
        "namespace Fixture",
        f"theorem {theorem_name} : True := by",
        *[f"  have _ := {name}" for name in required_calls],
        "  trivial",
        "",
        "lemma trailing_guard : True := by",
        "  trivial",
        "end Fixture",
        "",
    ]
    return "\n".join(lines)


def valid_audit_sources() -> dict[str, str]:
    return {
        node_id: valid_theorem_fixture(
            str(spec["theorem"]),
            tuple(str(name) for name in spec["required_calls"]),
        )
        for node_id, spec in ls_contract.THEOREM_BODY_AUDITS.items()
    }


def sha256_file(path: Path) -> str:
    return protocol.sha256_bytes(path.read_bytes())


def selected_attempt(
    rows: tuple[ls_validation.LSGraphRow, ...],
    formal_target_root: Path,
    node_id: str,
) -> Path:
    row = next(row for row in rows if row.node_id == node_id)
    if row.receipt_sha256 is None:
        raise AssertionError(f"graph row {node_id} has no committed receipt")
    node_root = formal_target_root / "proof-slices" / node_id
    matches = [
        attempt
        for attempt in node_root.iterdir()
        if not attempt.is_symlink()
        and attempt.is_dir()
        and not (attempt / "receipt.json").is_symlink()
        and (attempt / "receipt.json").is_file()
        and sha256_file(attempt / "receipt.json") == row.receipt_sha256
    ]
    if len(matches) != 1:
        raise AssertionError(
            f"expected one graph-selected attempt for {node_id}, found {len(matches)}"
        )
    return matches[0]


def next_attempt_number(node_root: Path) -> int:
    highest = 0
    for path in node_root.iterdir():
        if not path.is_dir():
            continue
        match = ls_validation.ATTEMPT_NAME.fullmatch(path.name)
        if match is None:
            continue
        highest = max(highest, int(path.name.removeprefix("attempt-")))
    return highest + 1


def rewrite_receipt(
    rows: tuple[ls_validation.LSGraphRow, ...],
    formal_target_root: Path,
    node_id: str,
    receipt: dict[str, object],
) -> tuple[ls_validation.LSGraphRow, ...]:
    receipt_path = selected_attempt(rows, formal_target_root, node_id) / "receipt.json"
    write_json(receipt_path, receipt)
    return replace_graph_row(rows, node_id, receipt_sha256=sha256_file(receipt_path))


def make_failed_receipt(
    rows: tuple[ls_validation.LSGraphRow, ...],
    formal_target_root: Path,
    node_id: str,
) -> tuple[ls_validation.LSGraphRow, ...]:
    attempt = selected_attempt(rows, formal_target_root, node_id)
    result_path = attempt / "result.json"
    result = json.loads(result_path.read_text(encoding="utf-8"))
    result.update({"status": "failed", "reason": "Lean command exited 1."})
    write_json(result_path, result)

    command_path = attempt / "build/command.json"
    command = json.loads(command_path.read_text(encoding="utf-8"))
    command["exit_code"] = 1
    write_json(command_path, command)

    axiom_path = attempt / "build/axioms.txt"
    axiom_path.write_bytes(b"")

    receipt = json.loads((attempt / "receipt.json").read_text(encoding="utf-8"))
    receipt.update(
        {
            "status": "failed",
            "reason": "Lean command failed.",
            "observed_axioms": [],
            "command_sha256": sha256_file(command_path),
            "axiom_audit_sha256": sha256_file(axiom_path),
            "result_sha256": sha256_file(result_path),
        }
    )
    failed_rows = replace_graph_row(
        rows, node_id, status="failed", failed_reason="Lean command failed."
    )
    return rewrite_receipt(failed_rows, formal_target_root, node_id, receipt)


class ValidatorExecutor:
    def __call__(
        self,
        argv: list[str],
        cwd: Path,
        env: dict[str, str],
        timeout_seconds: int,
        max_output_bytes: int,
    ) -> ls_receipts.CommandResult:
        del env, timeout_seconds, max_output_bytes
        if argv == [
            "scripts/check_lean_library.sh",
            "CrouzeixLoristSchwenninger",
        ]:
            return ls_receipts.CommandResult(0, VALIDATOR_STDOUT, b"aggregate stderr\n")
        source = (cwd / argv[-1]).read_text(encoding="utf-8")
        declaration = source.split("#print axioms ", 1)[1].strip()
        return ls_receipts.CommandResult(
            0,
            (
                f"'{declaration}' depends on axioms: "
                "[propext, Classical.choice, Quot.sound]\n"
            ).encode(),
            b"",
        )


def make_v2_receipts(
    root: Path,
) -> tuple[tuple[ls_validation.LSGraphRow, ...], Path, Path]:
    repository_root = root / "repository"
    formal_target_root = (
        repository_root
        / "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger"
    )
    (formal_target_root / "proof-slices").mkdir(parents=True)
    lean_root = repository_root / "formalization/lean"
    imports = []
    for node_id, (_, build_target) in ls_receipts.LS_RECEIPT_BINDINGS.items():
        module_path = repository_root / build_target
        module_path.parent.mkdir(parents=True, exist_ok=True)
        module_path.write_text(
            f"import {VALIDATOR_MATHLIB_MODULE}\n-- {node_id}\n",
            encoding="utf-8",
        )
        module = Path(build_target).relative_to("formalization/lean").with_suffix("")
        imports.append("import " + ".".join(module.parts))
    (lean_root / "CrouzeixLoristSchwenninger.lean").write_text(
        "\n".join(imports) + "\n", encoding="utf-8"
    )
    (lean_root / "lake-manifest.json").write_text("{}\n", encoding="utf-8")
    artifact = (
        lean_root
        / ".lake/packages/mathlib/.lake/build/lib/lean"
        / Path(*VALIDATOR_MATHLIB_MODULE.split(".")).with_suffix(".olean")
    )
    artifact.parent.mkdir(parents=True)
    artifact.write_bytes(b"olean-validator-artifact\n")
    dependency = (
        lean_root / ".lake/packages/batteries/.lake/build/lib/lean/Batteries.olean"
    )
    dependency.parent.mkdir(parents=True)
    dependency.write_bytes(b"olean-validator-dependency\n")
    (repository_root / "scripts").mkdir()
    (repository_root / "scripts/check_lean_library.sh").write_text(
        '#!/bin/sh\nlake --try-cache build "$1"\n', encoding="utf-8"
    )
    rows = ls_validation.load_route_graph(GRAPH)
    published = ls_receipts.publish_ls_receipts(
        rows, repository_root, formal_target_root, executor=ValidatorExecutor()
    )
    current_rows = tuple(
        replace(
            canonical_row,
            lean_name=published[row.node_id]["lean_name"],
            status="passed",
            receipt_sha256=published[row.node_id]["receipt_sha256"],
            blocked_reason=None,
            failed_reason=None,
        )
        for row, canonical_row in zip(rows, canonical_rows(), strict=True)
    )
    return current_rows, formal_target_root, repository_root


class LSValidationTests(unittest.TestCase):
    def test_ls_graph_has_terminal_node_and_no_jin_private_paths(self) -> None:
        graph = ls_validation.load_route_graph(GRAPH)

        self.assertEqual(
            tuple(row.node_id for row in graph),
            (
                "ls-equation-one-terminal-bound",
                "ls-power-recurrence",
                "ls-scalar-contradiction",
                "ls-perturbation-lemma",
                "ls-double-layer-realization",
                "ls-terminal-crouzeix",
            ),
        )
        self.assertEqual(graph[-1].node_id, "ls-terminal-crouzeix")
        self.assertEqual(graph[-1].role, "terminal")
        self.assertEqual(
            {row.node_id for row in graph if row.status == "passed"},
            {"ls-equation-one-terminal-bound", "ls-scalar-contradiction"},
        )
        self.assertEqual(
            {row.node_id for row in graph if row.status == "blocked"},
            {
                "ls-power-recurrence",
                "ls-perturbation-lemma",
                "ls-double-layer-realization",
                "ls-terminal-crouzeix",
            },
        )
        self.assertTrue(
            all(
                row.receipt_sha256 is not None
                for row in graph
                if row.status == "passed"
            )
        )
        self.assertIn("ls-perturbation-lemma", graph[-1].dependencies)
        self.assertIn("ls-double-layer-realization", graph[-1].dependencies)
        self.assertTrue(all("565b6a3" not in row.source_locator for row in graph))
        self.assertTrue(all("JIN" not in row.source_locator for row in graph))

    def test_ls_node_build_targets_match_the_six_source_faithful_bindings(self) -> None:
        self.assertEqual(
            ls_validation.LS_NODE_BUILD_TARGETS,
            {
                "ls-equation-one-terminal-bound": (
                    "formalization/lean/Crouzeix/LoristSchwenninger/Dilation.lean"
                ),
                "ls-power-recurrence": (
                    "formalization/lean/Crouzeix/LoristSchwenninger/"
                    "OperatorRecurrence.lean"
                ),
                "ls-scalar-contradiction": (
                    "formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean"
                ),
                "ls-perturbation-lemma": (
                    "formalization/lean/Crouzeix/LoristSchwenninger/"
                    "PerturbationLemma.lean"
                ),
                "ls-double-layer-realization": (
                    "formalization/lean/Crouzeix/LoristSchwenninger/"
                    "ConcreteDilation.lean"
                ),
                "ls-terminal-crouzeix": (
                    "formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean"
                ),
            },
        )

    def test_route_graph_rejects_noncanonical_shape_and_identity_fields(self) -> None:
        graph = json.loads(GRAPH.read_text(encoding="utf-8"))
        mutations = []
        missing = json.loads(json.dumps(graph))
        missing["nodes"].pop()
        mutations.append(missing)
        reordered = json.loads(json.dumps(graph))
        reordered["nodes"][0], reordered["nodes"][1] = (
            reordered["nodes"][1],
            reordered["nodes"][0],
        )
        mutations.append(reordered)
        for field, value in (
            ("source_locator", "arxiv:2608.03841v1:other.tex#L1-L2"),
            ("statement_sha256", "f" * 64),
            ("dependencies", ["ls-scalar-contradiction"]),
            ("role", "adapter"),
            ("lean_name", "CrouzeixConjecture.ValidButWrong"),
        ):
            changed = json.loads(json.dumps(graph))
            changed["nodes"][0][field] = value
            mutations.append(changed)

        for index, mutation in enumerate(mutations):
            with self.subTest(index=index):
                with tempfile.TemporaryDirectory() as directory:
                    path = Path(directory) / "source-graph.json"
                    write_json(path, mutation)
                    with self.assertRaisesRegex(
                        protocol.ValidationError, "canonical LS graph"
                    ):
                        ls_validation.load_route_graph(path)

    def test_live_graph_is_accepted_only_as_exact_legacy_contract(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        self.assertEqual(
            tuple(row.lean_name for row in rows),
            tuple(
                (node.legacy_graph_lean_name or node.declaration)
                for node in ls_contract.NODES
            ),
        )
        self.assertEqual(
            tuple(row.dependencies for row in rows),
            tuple(node.legacy_dependencies for node in ls_contract.NODES),
        )
        ls_validation._validate_graph_contract(rows, allow_legacy=True)
        with self.assertRaisesRegex(protocol.ValidationError, "canonical LS graph"):
            ls_validation._validate_graph_contract(rows, allow_legacy=False)

    def test_graph_contract_rejects_legacy_declaration_with_canonical_dependencies(
        self,
    ) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        hybrid = replace_graph_row(
            rows,
            "ls-terminal-crouzeix",
            dependencies=ls_contract.BY_ID["ls-terminal-crouzeix"].dependencies,
        )
        with self.assertRaisesRegex(
            protocol.ValidationError, "canonical LS graph"
        ):
            ls_validation._validate_graph_contract(hybrid, allow_legacy=True)

    def test_graph_contract_rejects_canonical_declaration_with_legacy_dependencies(
        self,
    ) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        hybrid = replace_graph_row(
            rows,
            "ls-terminal-crouzeix",
            lean_name=ls_contract.BY_ID["ls-terminal-crouzeix"].declaration,
        )
        with self.assertRaisesRegex(
            protocol.ValidationError, "canonical LS graph"
        ):
            ls_validation._validate_graph_contract(hybrid, allow_legacy=True)

    def test_canonical_rows_use_exact_terminal_dependency_without_redundant_edge(
        self,
    ) -> None:
        rows = canonical_rows()
        ls_validation._validate_graph_contract(rows, allow_legacy=False)
        self.assertEqual(rows[-1].dependencies, ("ls-double-layer-realization",))

    def test_validate_committed_receipts_accepts_bound_passed_bundles(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )

            rows = historical_passed_rows()
            receipts = validate_fixture_receipts(
                rows, formal_target_root, repository_root
            )
            result_reason = json.loads(
                (
                    selected_attempt(
                        rows, formal_target_root, "ls-equation-one-terminal-bound"
                    )
                    / "result.json"
                ).read_text(encoding="utf-8")
            )["reason"]

        self.assertEqual(set(receipts), {row.node_id for row in rows})
        self.assertTrue(
            all(receipt["status"] == "passed" for receipt in receipts.values())
        )
        self.assertEqual(
            receipts["ls-scalar-contradiction"]["expected_lean_declaration"],
            "CrouzeixConjecture.LoristSchwenninger.scalar_endpoint_le_two",
        )
        self.assertNotEqual(
            receipts["ls-equation-one-terminal-bound"]["reason"],
            result_reason,
            "v1 binds detailed result prose by hash, not by reason equality",
        )

    def test_validate_committed_receipts_validates_published_failed_bundle(
        self,
    ) -> None:
        node_id = "ls-equation-one-terminal-bound"
        rows = tuple(
            row
            for row in ls_validation.load_route_graph(GRAPH)
            if row.node_id == node_id
        )
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            failed_rows = make_failed_receipt(rows, formal_target_root, node_id)

            receipts = validate_receipts(
                failed_rows, formal_target_root, repository_root
            )
            self.assertEqual(receipts[node_id]["status"], "failed")

            task_path = (
                selected_attempt(failed_rows, formal_target_root, node_id) / "task.json"
            )
            task_path.write_bytes(task_path.read_bytes() + b"tampered\n")
            with self.assertRaisesRegex(
                protocol.ValidationError, "task_sha256.*mismatch"
            ):
                validate_receipts(failed_rows, formal_target_root, repository_root)

    def test_failed_receipt_requires_failed_execution_evidence(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            failed_rows = make_failed_receipt(rows, formal_target_root, node_id)
            attempt = selected_attempt(failed_rows, formal_target_root, node_id)
            command_path = attempt / "build/command.json"
            command = json.loads(command_path.read_text(encoding="utf-8"))
            command["exit_code"] = 0
            write_json(command_path, command)
            receipt = json.loads((attempt / "receipt.json").read_text(encoding="utf-8"))
            receipt["command_sha256"] = sha256_file(command_path)
            failed_rows = rewrite_receipt(
                failed_rows, formal_target_root, node_id, receipt
            )

            with self.assertRaisesRegex(
                protocol.ValidationError, "failed receipt.*failure evidence"
            ):
                validate_receipts(failed_rows, formal_target_root, repository_root)

    def test_command_schema_accepts_historical_v1_and_strict_v2(self) -> None:
        historical = {
            "schema_version": "crouzeix-ls-lean-command/v1",
            "argv": ["scripts/check_lean_library.sh", "Crouzeix"],
            "cache_policy": "historical policy",
            "cwd": ".",
            "elan_home": "/private/tmp/harp-mathematical-foundations-elan",
            "exit_code": 0,
        }
        current = {
            "schema_version": "crouzeix-ls-lean-command/v2",
            "argv": [
                "scripts/check_lean_library.sh",
                "CrouzeixLoristSchwenninger",
            ],
            "cache_policy": "snapshot-bound cached wrapper",
            "cwd": ".",
            "exit_code": 0,
            "elan_toolchain": "leanprover/lean4:v4.32.1",
            "env": {
                "ELAN_HOME": "/private/tmp/harp-mathematical-foundations-elan",
                "ELAN_TOOLCHAIN": "leanprover/lean4:v4.32.1",
                "PATH": (
                    "/private/tmp/harp-mathematical-foundations-elan/toolchains/"
                    "leanprover--lean4---v4.32.1/bin:/usr/bin:/bin"
                ),
            },
            "timeout_seconds": 3600,
            "output_cap_bytes": 1048576,
            "wrapper_sha256": "0" * 64,
            "lake_manifest_sha256": "1" * 64,
            "dependency_cache_metadata_sha256": "2" * 64,
            "required_mathlib_artifacts_sha256": protocol.sha256_bytes(
                b'{"artifacts":[]}\n'
            ),
            "local_source_closure_sha256": "4" * 64,
        }

        with tempfile.TemporaryDirectory() as directory:
            repository_root = Path(directory).resolve()
            wrapper = repository_root / "scripts/check_lean_library.sh"
            manifest = repository_root / "formalization/lean/lake-manifest.json"
            wrapper.parent.mkdir(parents=True)
            manifest.parent.mkdir(parents=True)
            wrapper.write_bytes(b"#!/bin/sh\n")
            manifest.write_bytes(b"{}\n")
            current["wrapper_sha256"] = sha256_file(wrapper)
            current["lake_manifest_sha256"] = sha256_file(manifest)
            closure = ls_validation._LocalSourceClosure(
                modules=frozenset(), sources=(), sha256="4" * 64
            )

            self.assertEqual(
                ls_validation._validate_command_object(historical, repository_root), 0
            )
            self.assertEqual(
                ls_validation._validate_command_object(
                    current, repository_root, closure
                ),
                0,
            )
            with self.assertRaisesRegex(protocol.ValidationError, "argv"):
                ls_validation._validate_command_object(
                    {
                        **current,
                        "argv": ["scripts/check_lean_library.sh", "Crouzeix"],
                    },
                    repository_root,
                    closure,
                )
            for mutation in (
                {**current, "wrapper_sha256": "invalid"},
                {**current, "timeout_seconds": 3600.0},
                {**current, "output_cap_bytes": True},
                {**current, "exit_code": 0.0},
                {**current, "unexpected": True},
                {
                    key: value
                    for key, value in current.items()
                    if key != "elan_toolchain"
                },
            ):
                with self.subTest(mutation=mutation):
                    with self.assertRaises(protocol.ValidationError):
                        ls_validation._validate_command_object(
                            mutation, repository_root, closure
                        )

            wrapper.write_bytes(b"#!/bin/sh\n# changed\n")
            with self.assertRaisesRegex(protocol.ValidationError, "wrapper_sha256"):
                ls_validation._validate_command_object(
                    current, repository_root, closure
                )

    def test_failed_receipt_rejects_successful_axiom_evidence(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            failed_rows = make_failed_receipt(rows, formal_target_root, node_id)
            attempt = selected_attempt(failed_rows, formal_target_root, node_id)
            receipt = json.loads((attempt / "receipt.json").read_text(encoding="utf-8"))
            axiom_path = attempt / "build/axioms.txt"
            axiom_path.write_text(
                f"'{receipt['expected_lean_declaration']}' depends on axioms: "
                "[propext]\n",
                encoding="utf-8",
            )
            receipt["observed_axioms"] = ["propext"]
            receipt["axiom_audit_sha256"] = sha256_file(axiom_path)
            failed_rows = rewrite_receipt(
                failed_rows, formal_target_root, node_id, receipt
            )

            with self.assertRaisesRegex(
                protocol.ValidationError, "failed receipt.*failure evidence"
            ):
                validate_receipts(failed_rows, formal_target_root, repository_root)

    def test_committed_receipts_reject_published_mapped_or_blocked_rows(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        source = next(
            row for row in rows if row.node_id == "ls-equation-one-terminal-bound"
        )
        for status, reason_changes in (
            ("mapped", {}),
            ("blocked", {"blocked_reason": "fixture blocker"}),
        ):
            with self.subTest(status=status):
                with tempfile.TemporaryDirectory() as directory:
                    formal_target_root, repository_root = copy_committed_receipts(
                        Path(directory).resolve()
                    )
                    bad_row = replace(
                        source,
                        status=status,
                        failed_reason=None,
                        **reason_changes,
                    )
                    bad_rows = tuple(
                        bad_row if row.node_id == source.node_id else row
                        for row in rows
                    )

                    with self.assertRaisesRegex(
                        protocol.ValidationError, f"{status}.*receipt_sha256"
                    ):
                        validate_receipts(bad_rows, formal_target_root, repository_root)

    def test_committed_receipt_build_target_has_exact_node_mapping(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            historical_targets = current_target_overrides(formal_target_root)
            attempt = selected_attempt(rows, formal_target_root, node_id)
            task_path = attempt / "task.json"
            task = json.loads(task_path.read_text(encoding="utf-8"))
            substituted_target = (
                "formalization/lean/Crouzeix/LoristSchwenninger/Scalar.lean"
            )
            task["build_target"] = substituted_target
            write_json(task_path, task)
            receipt = json.loads((attempt / "receipt.json").read_text(encoding="utf-8"))
            receipt["build_target"] = substituted_target
            receipt["task_sha256"] = sha256_file(task_path)
            receipt["module_sha256"] = sha256_file(repository_root / substituted_target)
            bad_rows = rewrite_receipt(rows, formal_target_root, node_id, receipt)

            with mock.patch.dict(
                ls_validation.LS_NODE_BUILD_TARGETS,
                historical_targets,
                clear=True,
            ):
                with self.assertRaisesRegex(
                    protocol.ValidationError,
                    "build_target does not match LS node",
                ):
                    ls_validation.validate_committed_receipts(
                        bad_rows, formal_target_root, repository_root
                    )

    def test_committed_receipt_build_target_is_in_active_import_closure(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            historical_targets = current_target_overrides(formal_target_root)
            root_module = repository_root / "formalization/lean/Crouzeix.lean"
            source = root_module.read_text(encoding="utf-8")
            root_module.write_text(
                source.replace(
                    "import Crouzeix.Jin.Terminal\n",
                    "-- import Crouzeix.Jin.Terminal\n",
                ),
                encoding="utf-8",
            )

            receipts = validate_receipts(rows, formal_target_root, repository_root)
            self.assertIn("ls-equation-one-terminal-bound", receipts)

            root_module.write_text(
                "import Crouzeix.Jin.Terminal\n",
                encoding="utf-8",
            )
            with mock.patch.dict(
                ls_validation.LS_NODE_BUILD_TARGETS,
                historical_targets,
                clear=True,
            ):
                with self.assertRaisesRegex(
                    protocol.ValidationError, "not reachable.*Crouzeix"
                ):
                    ls_validation.validate_committed_receipts(
                        rows, formal_target_root, repository_root
                    )

    def test_active_import_parser_ignores_line_and_nested_block_comments(
        self,
    ) -> None:
        source = b"""-- import Hidden.Line
/- outer
import Hidden.Block
/- import Hidden.Nested -/
-/
import Visible.Direct
/- inline comment -/ import Visible.AfterComment
namespace StopsHeaderParsing
import Hidden.Late
"""

        self.assertEqual(
            ls_validation._active_lean_imports(source, "Fixture"),
            ("Visible.Direct", "Visible.AfterComment"),
        )

    def test_active_import_parser_accepts_module_and_public_import_syntax(
        self,
    ) -> None:
        source = b"""/- module and public import inside comments are inactive -/
module
public import Visible.Public
import Visible.One
import Visible.Two
def marker := "import Hidden.String"
import Hidden.AfterHeader
"""

        self.assertEqual(
            ls_validation._active_lean_imports(source, "Fixture"),
            ("Visible.Public", "Visible.One", "Visible.Two"),
        )

    def test_committed_receipts_require_exact_selected_attempt_tree(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        for extra_kind in ("file", "directory", "fifo", "symlink"):
            with self.subTest(extra_kind=extra_kind):
                with tempfile.TemporaryDirectory() as directory:
                    formal_target_root, repository_root = copy_committed_receipts(
                        Path(directory).resolve()
                    )
                    attempt = selected_attempt(rows, formal_target_root, node_id)
                    extra = attempt / f"unexpected-{extra_kind}"
                    if extra_kind == "file":
                        extra.write_text("unbound\n", encoding="utf-8")
                    elif extra_kind == "directory":
                        extra.mkdir()
                    elif extra_kind == "fifo":
                        os.mkfifo(extra)
                    else:
                        extra.symlink_to("task.json")

                    with self.assertRaisesRegex(
                        protocol.ValidationError, "attempt member"
                    ):
                        validate_receipts(rows, formal_target_root, repository_root)

    def test_committed_receipts_enforce_each_output_log_budget(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        for log_name, digest_field in (
            ("stdout.log", "stdout_sha256"),
            ("stderr.log", "stderr_sha256"),
        ):
            with self.subTest(log_name=log_name):
                with tempfile.TemporaryDirectory() as directory:
                    formal_target_root, repository_root = copy_committed_receipts(
                        Path(directory).resolve()
                    )
                    attempt = selected_attempt(rows, formal_target_root, node_id)
                    log_path = attempt / "build" / log_name
                    log_path.write_bytes(b"x" * (1024 * 1024 + 1))
                    receipt = json.loads(
                        (attempt / "receipt.json").read_text(encoding="utf-8")
                    )
                    receipt[digest_field] = sha256_file(log_path)
                    bad_rows = rewrite_receipt(
                        rows, formal_target_root, node_id, receipt
                    )

                    with self.assertRaisesRegex(
                        protocol.ValidationError, f"{log_name}.*max_output_bytes"
                    ):
                        validate_receipts(bad_rows, formal_target_root, repository_root)

    def test_v2_receipts_require_exact_stdout_markers(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            rows, formal_target_root, repository_root = make_v2_receipts(
                Path(directory).resolve()
            )
            node_id = rows[0].node_id
            attempt = selected_attempt(rows, formal_target_root, node_id)
            stdout = attempt / "build/stdout.log"
            stdout.write_bytes(b"[lean] outcome=passed with warnings\n")
            receipt = json.loads((attempt / "receipt.json").read_text())
            receipt["stdout_sha256"] = sha256_file(stdout)
            bad_rows = rewrite_receipt(rows, formal_target_root, node_id, receipt)

            with self.assertRaisesRegex(
                protocol.ValidationError, "stdout.*exact success markers"
            ):
                ls_validation.validate_committed_receipts(
                    bad_rows, formal_target_root, repository_root
                )

    def test_aggregate_stdout_target_is_selected_by_command_schema(self) -> None:
        v1_stdout = VALIDATOR_STDOUT.replace(
            b"target=CrouzeixLoristSchwenninger", b"target=Crouzeix"
        )
        ls_validation._validate_aggregate_stdout(
            v1_stdout,
            "legacy-node",
            "crouzeix-ls-lean-command/v1",
        )
        ls_validation._validate_aggregate_stdout(
            VALIDATOR_STDOUT,
            "current-node",
            "crouzeix-ls-lean-command/v2",
        )
        with self.assertRaisesRegex(protocol.ValidationError, "success markers"):
            ls_validation._validate_aggregate_stdout(
                v1_stdout,
                "current-node",
                "crouzeix-ls-lean-command/v2",
            )

    def test_v2_receipts_require_ordered_allowed_axioms(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            rows, formal_target_root, repository_root = make_v2_receipts(
                Path(directory).resolve()
            )
            node_id = rows[0].node_id
            attempt = selected_attempt(rows, formal_target_root, node_id)
            receipt = json.loads((attempt / "receipt.json").read_text())
            receipt["allowed_axioms"] = list(reversed(receipt["allowed_axioms"]))
            bad_rows = rewrite_receipt(rows, formal_target_root, node_id, receipt)

            with self.assertRaisesRegex(
                protocol.ValidationError, "allowed axioms.*policy"
            ):
                ls_validation.validate_committed_receipts(
                    bad_rows, formal_target_root, repository_root
                )

    def test_v2_canonical_six_reject_graph_drift_before_attempt_validation(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            rows, formal_target_root, repository_root = make_v2_receipts(
                Path(directory).resolve()
            )
            first = rows[0].node_id
            mutations = (
                ("order", (rows[1], rows[0], *rows[2:])),
                (
                    "dependency edge",
                    replace_graph_row(rows, first, dependencies=(rows[2].node_id,)),
                ),
                ("role", replace_graph_row(rows, first, role="adapter")),
                (
                    "source locator",
                    replace_graph_row(
                        rows,
                        first,
                        source_locator=(
                            "arxiv:2608.03841v1:CrouzeixConjecturev2.tex#L1-L2"
                        ),
                    ),
                ),
                (
                    "statement digest",
                    replace_graph_row(rows, first, statement_sha256="f" * 64),
                ),
                (
                    "declaration",
                    replace_graph_row(
                        rows,
                        first,
                        lean_name="CrouzeixConjecture.ValidButWrong",
                    ),
                ),
            )

            for field, bad_rows in mutations:
                with self.subTest(field=field):
                    with mock.patch.object(
                        ls_validation,
                        "_validate_committed_receipt",
                        side_effect=AssertionError(
                            "attempt validation reached before graph rejection"
                        ),
                    ):
                        with self.assertRaisesRegex(
                            protocol.ValidationError, "canonical LS graph"
                        ):
                            ls_validation.validate_committed_receipts(
                                bad_rows, formal_target_root, repository_root
                            )

    def test_v2_canonical_six_require_shared_aggregate_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            rows, formal_target_root, repository_root = make_v2_receipts(
                Path(directory).resolve()
            )
            node_id = rows[-1].node_id
            attempt = selected_attempt(rows, formal_target_root, node_id)
            stderr = attempt / "build/stderr.log"
            stderr.write_bytes(b"different aggregate stderr\n")
            receipt = json.loads((attempt / "receipt.json").read_text())
            receipt["stderr_sha256"] = sha256_file(stderr)
            bad_rows = rewrite_receipt(rows, formal_target_root, node_id, receipt)

            with self.assertRaisesRegex(
                protocol.ValidationError, "canonical six.*identical aggregate"
            ):
                ls_validation.validate_committed_receipts(
                    bad_rows, formal_target_root, repository_root
                )

    def test_v2_rejects_hardlinked_evidence_member(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            rows, formal_target_root, repository_root = make_v2_receipts(root)
            attempt = selected_attempt(rows, formal_target_root, rows[0].node_id)
            os.link(attempt / "task.json", root / "task-alias.json")

            with self.assertRaisesRegex(
                protocol.ValidationError, "exactly one hard link"
            ):
                ls_validation.validate_committed_receipts(
                    rows, formal_target_root, repository_root
                )

    def test_v2_recomputes_local_closure_and_mathlib_artifact_content(self) -> None:
        for mutation in ("closure", "mathlib"):
            with self.subTest(mutation=mutation):
                with tempfile.TemporaryDirectory() as directory:
                    rows, formal_target_root, repository_root = make_v2_receipts(
                        Path(directory).resolve()
                    )
                    if mutation == "closure":
                        target = (
                            repository_root
                            / ls_receipts.LS_RECEIPT_BINDINGS[rows[-1].node_id][1]
                        )
                        target.write_bytes(target.read_bytes() + b"-- changed\n")
                        expected = "local_source_closure_sha256"
                    else:
                        target = (
                            repository_root
                            / "formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean"
                            / Path(*VALIDATOR_MATHLIB_MODULE.split(".")).with_suffix(
                                ".olean"
                            )
                        )
                        target.write_bytes(b"changed artifact\n")
                        expected = "required_mathlib_artifacts_sha256"

                    with self.assertRaisesRegex(protocol.ValidationError, expected):
                        ls_validation.validate_committed_receipts(
                            rows, formal_target_root, repository_root
                        )

    def test_v2_rejects_missing_transitive_managed_local_import(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            rows, formal_target_root, repository_root = make_v2_receipts(
                Path(directory).resolve()
            )
            target = (
                repository_root / ls_receipts.LS_RECEIPT_BINDINGS[rows[0].node_id][1]
            )
            target.write_text("import Fixture.Helper\n", encoding="utf-8")
            helper = repository_root / "formalization/lean/Fixture/Helper.lean"
            helper.parent.mkdir(parents=True)
            helper.write_text(
                "import CrouzeixConjecture.Missing\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                protocol.ValidationError, "local module does not exist"
            ):
                ls_validation.validate_committed_receipts(
                    rows, formal_target_root, repository_root
                )

    def test_v2_rejects_harp_namespace_prefix(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            rows, formal_target_root, repository_root = make_v2_receipts(
                Path(directory).resolve()
            )
            target = (
                repository_root / ls_receipts.LS_RECEIPT_BINDINGS[rows[0].node_id][1]
            )
            target.write_text("import Crouzeix.Harp.Support\n", encoding="utf-8")

            with self.assertRaisesRegex(
                protocol.ValidationError, "forbidden module Crouzeix.Harp.Support"
            ):
                ls_validation.validate_committed_receipts(
                    rows, formal_target_root, repository_root
                )

    def test_v2_rejects_symlink_below_approved_lake_boundary(self) -> None:
        for relative in (
            "formalization/lean/.lake/packages",
            "formalization/lean/.lake/packages/mathlib/.lake/build",
        ):
            with self.subTest(relative=relative):
                with tempfile.TemporaryDirectory() as directory:
                    root = Path(directory).resolve()
                    rows, formal_target_root, repository_root = make_v2_receipts(root)
                    target = repository_root / relative
                    outside = root / "outside-cache"
                    target.rename(outside)
                    target.symlink_to(outside, target_is_directory=True)

                    with self.assertRaisesRegex(
                        protocol.ValidationError, "cannot be a symlink|cache.*symlink"
                    ):
                        ls_validation.validate_committed_receipts(
                            rows, formal_target_root, repository_root
                        )

    def test_theorem_body_audit_accepts_active_exact_body(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository_root = make_audit_repo(
                Path(directory).resolve(), valid_audit_sources()
            )
            audits = ls_validation.audit_required_theorem_provider_calls(repository_root)
        self.assertEqual(
            audits["ls-terminal-crouzeix"],
            (
                "norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary",
                "norm_polynomialEval_le_of_tendsto",
                "tendsto_maxPolynomialModulusOnSet_of_outerApproximation",
            ),
        )

    def test_theorem_body_audit_rejects_required_names_only_in_comments(self) -> None:
        sources = valid_audit_sources()
        sources["ls-power-recurrence"] = "\n".join(
            [
                "namespace Fixture",
                "theorem equation_three_lower_bound : True := by",
                "  -- recurrence_lower_bound",
                "  /- recurrence_difference_lower_bound -/",
                "  trivial",
                "end Fixture",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            repository_root = make_audit_repo(Path(directory).resolve(), sources)
            with self.assertRaisesRegex(
                protocol.ValidationError,
                "missing required provider calls",
            ):
                ls_validation.audit_required_theorem_provider_calls(repository_root)

    def test_theorem_body_audit_rejects_required_names_only_in_strings(self) -> None:
        sources = valid_audit_sources()
        sources["ls-perturbation-lemma"] = "\n".join(
            [
                "namespace Fixture",
                "theorem norm_target_le_two : True := by",
                '  let _ := "equation_three_lower_bound displacementSq_le"',
                '  let _ := s!"scalar_endpoint_le_two"',
                '  let _ := r#"equation_three_lower_bound"#',
                "  trivial",
                "end Fixture",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            repository_root = make_audit_repo(Path(directory).resolve(), sources)
            with self.assertRaisesRegex(
                protocol.ValidationError,
                "missing required provider calls",
            ):
                ls_validation.audit_required_theorem_provider_calls(repository_root)

    def test_theorem_body_audit_rejects_names_only_in_following_lemma(self) -> None:
        sources = valid_audit_sources()
        sources["ls-double-layer-realization"] = "\n".join(
            [
                "namespace Fixture",
                "theorem norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary : True := by",
                "  trivial",
                "",
                "lemma trailing_guard : True := by",
                "  have _ := dilationDataOfParametricPolynomial",
                "  have _ := norm_target_le_two",
                "  trivial",
                "end Fixture",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            repository_root = make_audit_repo(Path(directory).resolve(), sources)
            with self.assertRaisesRegex(
                protocol.ValidationError,
                "missing required provider calls",
            ):
                ls_validation.audit_required_theorem_provider_calls(repository_root)

    def test_theorem_body_audit_rejects_missing_duplicate_malformed_and_nonby_theorem(
        self,
    ) -> None:
        cases = {
            "missing": "namespace Fixture\nlemma other : True := by\n  trivial\nend Fixture\n",
            "duplicate": "\n".join(
                [
                    "namespace Fixture",
                    "theorem equation_three_lower_bound : True := by",
                    "  have _ := recurrence_lower_bound",
                    "  have _ := recurrence_difference_lower_bound",
                    "  trivial",
                    "theorem equation_three_lower_bound : True := by",
                    "  have _ := recurrence_lower_bound",
                    "  have _ := recurrence_difference_lower_bound",
                    "  trivial",
                    "end Fixture",
                    "",
                ]
            ),
            "malformed": "namespace Fixture\ntheorem equation_three_lower_bound\nend Fixture\n",
            "nonby": (
                "namespace Fixture\n"
                "theorem equation_three_lower_bound : True := True.intro\n"
                "end Fixture\n"
            ),
        }
        expected = {
            "missing": "missing theorem declaration",
            "duplicate": "duplicate theorem declaration",
            "malformed": "malformed theorem declaration",
            "nonby": "non-by theorem declaration",
        }
        for case, content in cases.items():
            with self.subTest(case=case), tempfile.TemporaryDirectory() as directory:
                sources = valid_audit_sources()
                sources["ls-power-recurrence"] = content
                repository_root = make_audit_repo(Path(directory).resolve(), sources)
                with self.assertRaisesRegex(protocol.ValidationError, expected[case]):
                    ls_validation.audit_required_theorem_provider_calls(repository_root)

    def test_theorem_body_audit_rejects_empty_theorem_body(self) -> None:
        sources = valid_audit_sources()
        sources["ls-power-recurrence"] = (
            "namespace Fixture\n"
            "theorem equation_three_lower_bound : True := by\n"
            "lemma next_command : True := by\n"
            "  trivial\n"
            "end Fixture\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            repository_root = make_audit_repo(Path(directory).resolve(), sources)
            with self.assertRaisesRegex(protocol.ValidationError, "empty theorem body"):
                ls_validation.audit_required_theorem_provider_calls(repository_root)

    def test_theorem_body_audit_rejects_symlinked_parent_or_file(self) -> None:
        for case in ("parent", "file"):
            with self.subTest(case=case), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                repository_root = make_audit_repo(root, valid_audit_sources())
                target = repository_root / str(
                    ls_contract.THEOREM_BODY_AUDITS["ls-power-recurrence"]["path"]
                )
                outside = root / "outside.lean"
                write_text(outside, valid_audit_sources()["ls-power-recurrence"])
                if case == "parent":
                    parent = target.parent
                    moved = root / "moved-parent"
                    parent.rename(moved)
                    parent.symlink_to(moved, target_is_directory=True)
                else:
                    target.unlink()
                    target.symlink_to(outside)
                with self.assertRaisesRegex(
                    protocol.ValidationError, "symlink|regular file"
                ):
                    ls_validation.audit_required_theorem_provider_calls(repository_root)

    def test_theorem_body_audit_rejects_hardlinked_final_file(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            repository_root = make_audit_repo(root, valid_audit_sources())
            target = repository_root / str(
                ls_contract.THEOREM_BODY_AUDITS["ls-power-recurrence"]["path"]
            )
            alias = root / "alias.lean"
            os.link(target, alias)
            with self.assertRaisesRegex(
                protocol.ValidationError, "exactly one hard link"
            ):
                ls_validation.audit_required_theorem_provider_calls(repository_root)

    def test_theorem_body_audit_rejects_oversize_and_invalid_utf8(self) -> None:
        cases = {
            "oversize": b"x" * (provider_independence.MAX_SOURCE_BYTES + 1),
            "utf8": b"\xff\xfe\xfd",
        }
        for case, payload in cases.items():
            with self.subTest(case=case), tempfile.TemporaryDirectory() as directory:
                sources = valid_audit_sources()
                sources["ls-power-recurrence"] = payload
                repository_root = make_audit_repo(Path(directory).resolve(), sources)
                with self.assertRaisesRegex(
                    protocol.ValidationError,
                    "exceeds byte cap|cannot decode",
                ):
                    ls_validation.audit_required_theorem_provider_calls(repository_root)

    def test_theorem_body_audit_forbidden_terminal_provider_in_active_code_fails(
        self,
    ) -> None:
        sources = valid_audit_sources()
        sources["ls-terminal-crouzeix"] = "\n".join(
            [
                "namespace Fixture",
                "theorem loristSchwenningerMainTheorem : True := by",
                "  have _ := norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary",
                "  have _ := norm_polynomialEval_le_of_tendsto",
                "  have _ := tendsto_maxPolynomialModulusOnSet_of_outerApproximation",
                "  have _ := harpFiniteHorizonMainTheorem",
                "  trivial",
                "end Fixture",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            repository_root = make_audit_repo(Path(directory).resolve(), sources)
            with self.assertRaisesRegex(
                protocol.ValidationError,
                "forbidden provider calls",
            ):
                ls_validation.audit_required_theorem_provider_calls(repository_root)

    def test_theorem_body_audit_forbidden_name_in_comment_or_string_does_not_trip(
        self,
    ) -> None:
        sources = valid_audit_sources()
        sources["ls-terminal-crouzeix"] = "\n".join(
            [
                "namespace Fixture",
                "theorem loristSchwenningerMainTheorem : True := by",
                "  -- harpFiniteHorizonMainTheorem",
                '  let _ := "harpFiniteHorizonMainTheorem"',
                "  have _ := norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary",
                "  have _ := norm_polynomialEval_le_of_tendsto",
                "  have _ := tendsto_maxPolynomialModulusOnSet_of_outerApproximation",
                "  trivial",
                "end Fixture",
                "",
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            repository_root = make_audit_repo(Path(directory).resolve(), sources)
            audits = ls_validation.audit_required_theorem_provider_calls(repository_root)
        self.assertEqual(
            audits["ls-terminal-crouzeix"],
            (
                "norm_euclideanOperator_polynomialEval_le_two_of_parametricBoundary",
                "norm_polynomialEval_le_of_tendsto",
                "tendsto_maxPolynomialModulusOnSet_of_outerApproximation",
            ),
        )

    def test_committed_receipts_accept_zero_axioms_and_bind_declaration(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            attempt = selected_attempt(rows, formal_target_root, node_id)
            receipt_path = attempt / "receipt.json"
            receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
            axiom_path = attempt / "build/axioms.txt"
            axiom_path.write_text(
                f"'{receipt['expected_lean_declaration']}' does not depend on any axioms\n",
                encoding="utf-8",
            )
            receipt["observed_axioms"] = []
            receipt["axiom_audit_sha256"] = sha256_file(axiom_path)
            updated_rows = rewrite_receipt(rows, formal_target_root, node_id, receipt)

            validated = validate_receipts(
                updated_rows, formal_target_root, repository_root
            )
            self.assertEqual(validated[node_id]["observed_axioms"], [])

        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            attempt = selected_attempt(rows, formal_target_root, node_id)
            receipt_path = attempt / "receipt.json"
            receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
            axiom_path = attempt / "build/axioms.txt"
            axiom_path.write_text(
                "'CrouzeixConjecture.Wrong' does not depend on any axioms\n",
                encoding="utf-8",
            )
            receipt["observed_axioms"] = []
            receipt["axiom_audit_sha256"] = sha256_file(axiom_path)
            bad_rows = rewrite_receipt(rows, formal_target_root, node_id, receipt)

            with self.assertRaisesRegex(
                protocol.ValidationError, "axiom audit declaration.*expected"
            ):
                validate_receipts(bad_rows, formal_target_root, repository_root)

    def test_committed_receipts_reject_duplicate_json_keys(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        cases = (
            ("receipt.json", None),
            ("task.json", "task_sha256"),
            ("source-slice.json", "source_slice_sha256"),
            ("result.json", "result_sha256"),
            ("build/command.json", "command_sha256"),
        )
        for relative_path, digest_field in cases:
            with self.subTest(relative_path=relative_path):
                with tempfile.TemporaryDirectory() as directory:
                    formal_target_root, repository_root = copy_committed_receipts(
                        Path(directory).resolve()
                    )
                    attempt = selected_attempt(rows, formal_target_root, node_id)
                    member_path = attempt / relative_path
                    member = json.loads(member_path.read_text(encoding="utf-8"))
                    write_json_with_duplicate_key(member_path, member, "schema_version")
                    if digest_field is None:
                        bad_rows = replace_graph_row(
                            rows, node_id, receipt_sha256=sha256_file(member_path)
                        )
                    else:
                        receipt = json.loads(
                            (attempt / "receipt.json").read_text(encoding="utf-8")
                        )
                        receipt[digest_field] = sha256_file(member_path)
                        bad_rows = rewrite_receipt(
                            rows, formal_target_root, node_id, receipt
                        )

                    with self.assertRaisesRegex(
                        protocol.ValidationError, "duplicate JSON key"
                    ):
                        validate_receipts(bad_rows, formal_target_root, repository_root)

    def test_committed_receipt_discovery_bounds_attempt_count(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            node_root = formal_target_root / "proof-slices" / node_id
            start = next_attempt_number(node_root)
            for attempt_number in range(start, start + 256):
                historical = node_root / f"attempt-{attempt_number:03d}"
                historical.mkdir()
                (historical / "receipt.json").write_text("{}\n", encoding="utf-8")

            with self.assertRaisesRegex(
                protocol.ValidationError, "attempt count exceeds cap"
            ):
                validate_receipts(rows, formal_target_root, repository_root)

    def test_committed_receipt_discovery_bounds_all_node_entries(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            node_root = formal_target_root / "proof-slices" / node_id
            for entry_number in range(ls_validation.MAX_ATTEMPTS):
                (node_root / f"unrelated-{entry_number:03d}").touch()

            with self.assertRaisesRegex(
                protocol.ValidationError, "attempt count exceeds cap"
            ):
                validate_receipts(rows, formal_target_root, repository_root)

    def test_committed_receipt_selected_attempt_walk_fails_early(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            attempt = selected_attempt(rows, formal_target_root, node_id)
            unexpected = attempt / "unexpected"
            unexpected.touch()
            original_scandir = os.scandir

            class GuardedScandir:
                def __init__(self, entries: list[os.DirEntry[str]]) -> None:
                    self.iterator = iter(entries)

                def __enter__(self) -> "GuardedScandir":
                    return self

                def __exit__(self, *args: object) -> None:
                    return None

                def __iter__(self) -> "GuardedScandir":
                    return self

                def __next__(self) -> os.DirEntry[str]:
                    return next(self.iterator)

            def guarded_scandir(target: object):
                iterator = original_scandir(target)
                if isinstance(target, int):
                    entries = list(iterator)
                    names = {entry.name for entry in entries}
                    if "unexpected" in names and "task.json" in names:
                        unexpected_entry = next(
                            entry for entry in entries if entry.name == "unexpected"
                        )
                        return GuardedScandir([unexpected_entry])
                    return GuardedScandir(entries)
                return iterator

            with mock.patch.object(os, "scandir", side_effect=guarded_scandir):
                with self.assertRaisesRegex(
                    protocol.ValidationError, "unexpected.*attempt member"
                ):
                    validate_receipts(rows, formal_target_root, repository_root)

    def test_source_discovery_rejects_non_lake_symlink(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            formal_target_root, repository_root = copy_committed_receipts(root)
            outside = root / "outside-source"
            outside.write_text("def outside := 1\n", encoding="utf-8")
            (repository_root / "formalization/lean/Unexpected.lean").symlink_to(outside)

            with self.assertRaisesRegex(
                protocol.ValidationError, "Lean source.*symlink"
            ):
                validate_receipts(rows, formal_target_root, repository_root)

    def test_committed_receipt_import_discovery_bounds_all_source_entries(
        self,
    ) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            with mock.patch.object(ls_validation, "MAX_IMPORT_MODULES", 3):
                with self.assertRaisesRegex(
                    protocol.ValidationError, "Lean source entry count exceeds cap"
                ):
                    validate_receipts(rows, formal_target_root, repository_root)

    def test_committed_receipt_discovery_bounds_aggregate_receipt_bytes(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            node_root = formal_target_root / "proof-slices" / node_id
            start = next_attempt_number(node_root)
            for attempt_number in range(start, start + 4):
                historical = node_root / f"attempt-{attempt_number:03d}"
                historical.mkdir()
                (historical / "receipt.json").write_bytes(b"x" * (1024 * 1024))

            with self.assertRaisesRegex(
                protocol.ValidationError, "aggregate receipt bytes exceed cap"
            ):
                validate_receipts(rows, formal_target_root, repository_root)

    def test_next_attempt_number_uses_monotone_slot_after_tracked_history(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, _ = copy_committed_receipts(Path(directory).resolve())
            node_root = (
                formal_target_root
                / "proof-slices"
                / "ls-equation-one-terminal-bound"
            )

            self.assertEqual(next_attempt_number(node_root), 3)

            (node_root / "attempt-003").mkdir()
            self.assertEqual(next_attempt_number(node_root), 4)

    def test_committed_receipts_reject_duplicate_dependency_ids(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-scalar-contradiction"
        dependency_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            attempt = selected_attempt(rows, formal_target_root, node_id)
            source_slice_path = attempt / "source-slice.json"
            source_slice = json.loads(source_slice_path.read_text(encoding="utf-8"))
            source_slice["dependency_ids"] = [dependency_id, dependency_id]
            write_json(source_slice_path, source_slice)
            receipt = json.loads((attempt / "receipt.json").read_text(encoding="utf-8"))
            receipt["source_slice_sha256"] = sha256_file(source_slice_path)
            bad_rows = replace_graph_row(
                rows, node_id, dependencies=(dependency_id, dependency_id)
            )
            bad_rows = rewrite_receipt(bad_rows, formal_target_root, node_id, receipt)

            with self.assertRaisesRegex(
                protocol.ValidationError, "dependencies.*duplicates"
            ):
                validate_receipts(bad_rows, formal_target_root, repository_root)

    def test_committed_receipts_recompute_every_embedded_member_hash(self) -> None:
        member_cases = (
            ("task.json", "task_sha256"),
            ("source-slice.json", "source_slice_sha256"),
            ("result.json", "result_sha256"),
            ("build/command.json", "command_sha256"),
            ("build/stdout.log", "stdout_sha256"),
            ("build/stderr.log", "stderr_sha256"),
            ("build/axioms.txt", "axiom_audit_sha256"),
        )
        for relative_path, digest_field in member_cases:
            with self.subTest(member=relative_path):
                with tempfile.TemporaryDirectory() as directory:
                    formal_target_root, repository_root = copy_committed_receipts(
                        Path(directory).resolve()
                    )
                    member = (
                        selected_attempt(
                            ls_validation.load_route_graph(GRAPH),
                            formal_target_root,
                            "ls-equation-one-terminal-bound",
                        )
                        / relative_path
                    )
                    member.write_bytes(member.read_bytes() + b"tampered\n")

                    with self.assertRaisesRegex(
                        protocol.ValidationError, f"{digest_field}.*mismatch"
                    ):
                        validate_receipts(
                            ls_validation.load_route_graph(GRAPH),
                            formal_target_root,
                            repository_root,
                        )

        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            attempt = selected_attempt(
                ls_validation.load_route_graph(GRAPH),
                formal_target_root,
                "ls-equation-one-terminal-bound",
            )
            receipt = json.loads((attempt / "receipt.json").read_text(encoding="utf-8"))
            module = repository_root / receipt["build_target"]
            module.write_text(module.read_text(encoding="utf-8") + "\n-- tampered\n")

            with self.assertRaisesRegex(
                protocol.ValidationError, "module_sha256.*mismatch"
            ):
                validate_receipts(
                    ls_validation.load_route_graph(GRAPH),
                    formal_target_root,
                    repository_root,
                )

    def test_committed_receipts_reject_graph_receipt_and_status_drift(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            bad_rows = replace_graph_row(rows, node_id, receipt_sha256="0" * 64)

            with self.assertRaisesRegex(
                protocol.ValidationError, "receipt_sha256.*match"
            ):
                validate_receipts(bad_rows, formal_target_root, repository_root)

        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            receipt_path = (
                selected_attempt(rows, formal_target_root, node_id) / "receipt.json"
            )
            receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
            receipt["status"] = "failed"
            bad_rows = rewrite_receipt(rows, formal_target_root, node_id, receipt)

            with self.assertRaisesRegex(
                protocol.ValidationError, "receipt status.*graph node"
            ):
                validate_receipts(bad_rows, formal_target_root, repository_root)

        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            attempt = selected_attempt(rows, formal_target_root, node_id)
            result_path = attempt / "result.json"
            result = json.loads(result_path.read_text(encoding="utf-8"))
            result["status"] = "failed"
            write_json(result_path, result)
            receipt = json.loads((attempt / "receipt.json").read_text(encoding="utf-8"))
            receipt["result_sha256"] = sha256_file(result_path)
            bad_rows = rewrite_receipt(rows, formal_target_root, node_id, receipt)

            with self.assertRaisesRegex(
                protocol.ValidationError, "result status.*receipt status"
            ):
                validate_receipts(bad_rows, formal_target_root, repository_root)

    def test_committed_receipts_reject_axiom_and_declaration_drift(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            attempt = selected_attempt(rows, formal_target_root, node_id)
            axiom_path = attempt / "build/axioms.txt"
            axiom_path.write_text(
                axiom_path.read_text(encoding="utf-8").replace(
                    row_name := next(
                        row.lean_name for row in rows if row.node_id == node_id
                    ),
                    f"{row_name}.tampered",
                ),
                encoding="utf-8",
            )
            receipt = json.loads((attempt / "receipt.json").read_text(encoding="utf-8"))
            receipt["axiom_audit_sha256"] = sha256_file(axiom_path)
            bad_rows = rewrite_receipt(rows, formal_target_root, node_id, receipt)

            with self.assertRaisesRegex(
                protocol.ValidationError, "axiom audit declaration.*expected"
            ):
                validate_receipts(bad_rows, formal_target_root, repository_root)

        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            attempt = selected_attempt(rows, formal_target_root, node_id)
            task_path = attempt / "task.json"
            task = json.loads(task_path.read_text(encoding="utf-8"))
            task["expected_lean_declaration"] = "LS.WrongDeclaration"
            write_json(task_path, task)
            receipt = json.loads((attempt / "receipt.json").read_text(encoding="utf-8"))
            receipt["task_sha256"] = sha256_file(task_path)
            bad_rows = rewrite_receipt(rows, formal_target_root, node_id, receipt)

            with self.assertRaisesRegex(
                protocol.ValidationError, "task declaration.*receipt declaration"
            ):
                validate_receipts(bad_rows, formal_target_root, repository_root)

    def test_committed_receipts_reject_missing_symlinked_and_unsafe_files(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            receipt_path = (
                selected_attempt(rows, formal_target_root, node_id) / "receipt.json"
            )
            receipt_path.unlink()

            with self.assertRaisesRegex(
                protocol.ValidationError, "receipt_sha256.*match"
            ):
                validate_receipts(rows, formal_target_root, repository_root)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            formal_target_root, repository_root = copy_committed_receipts(root)
            task_path = (
                selected_attempt(rows, formal_target_root, node_id) / "task.json"
            )
            outside = root / "outside-task.json"
            outside.write_bytes(task_path.read_bytes())
            task_path.unlink()
            task_path.symlink_to(outside)

            with self.assertRaisesRegex(protocol.ValidationError, "task.*symlink"):
                validate_receipts(rows, formal_target_root, repository_root)

        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            formal_target_root, repository_root = copy_committed_receipts(root)
            attempt = selected_attempt(rows, formal_target_root, node_id)
            outside = root / "outside-attempt"
            attempt.rename(outside)
            attempt.symlink_to(outside, target_is_directory=True)

            with self.assertRaisesRegex(protocol.ValidationError, "attempt.*symlink"):
                validate_receipts(rows, formal_target_root, repository_root)

        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            historical_targets = current_target_overrides(formal_target_root)
            attempt = selected_attempt(rows, formal_target_root, node_id)
            task_path = attempt / "task.json"
            task = json.loads(task_path.read_text(encoding="utf-8"))
            task["build_target"] = "../../outside.lean"
            write_json(task_path, task)
            receipt = json.loads((attempt / "receipt.json").read_text(encoding="utf-8"))
            receipt["build_target"] = task["build_target"]
            receipt["task_sha256"] = sha256_file(task_path)
            bad_rows = rewrite_receipt(rows, formal_target_root, node_id, receipt)

            with mock.patch.dict(
                ls_validation.LS_NODE_BUILD_TARGETS,
                historical_targets,
                clear=True,
            ):
                with self.assertRaisesRegex(
                    protocol.ValidationError,
                    "build_target.*normalized.*formalization/lean",
                ):
                    ls_validation.validate_committed_receipts(
                        bad_rows, formal_target_root, repository_root
                    )

    def test_passed_node_requires_passed_mechanically_validated_predecessors(
        self,
    ) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-scalar-contradiction"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            bad_rows = replace_graph_row(
                rows, node_id, dependencies=("ls-power-recurrence",)
            )

            with self.assertRaisesRegex(
                protocol.ValidationError, "predecessor.*ls-power-recurrence.*not passed"
            ):
                validate_receipts(bad_rows, formal_target_root, repository_root)

        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            dependency_id = "ls-equation-one-terminal-bound"
            attempt = selected_attempt(rows, formal_target_root, node_id)
            source_slice_path = attempt / "source-slice.json"
            source_slice = json.loads(source_slice_path.read_text(encoding="utf-8"))
            source_slice["dependency_ids"] = [dependency_id]
            write_json(source_slice_path, source_slice)
            receipt = json.loads((attempt / "receipt.json").read_text(encoding="utf-8"))
            receipt["source_slice_sha256"] = sha256_file(source_slice_path)
            dependent_rows = replace_graph_row(
                rows, node_id, dependencies=(dependency_id,)
            )
            dependent_rows = rewrite_receipt(
                dependent_rows, formal_target_root, node_id, receipt
            )
            dependency_task = (
                selected_attempt(dependent_rows, formal_target_root, dependency_id)
                / "task.json"
            )
            dependency_task.write_bytes(dependency_task.read_bytes() + b"tampered\n")
            dependent_first = tuple(
                sorted(
                    dependent_rows,
                    key=lambda row: 0 if row.node_id == node_id else 1,
                )
            )

            with self.assertRaisesRegex(
                protocol.ValidationError, "task_sha256.*mismatch"
            ):
                validate_receipts(dependent_first, formal_target_root, repository_root)

    def test_committed_receipt_selects_one_matching_attempt_among_history(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            formal_target_root, repository_root = copy_committed_receipts(
                Path(directory).resolve()
            )
            node_root = formal_target_root / "proof-slices" / node_id
            committed = selected_attempt(rows, formal_target_root, node_id)
            historical_receipts = {
                attempt.name: (attempt / "receipt.json").read_bytes()
                for attempt in node_root.iterdir()
                if attempt.is_dir() and (attempt / "receipt.json").is_file()
            }
            next_number = (
                max(
                    int(attempt.name.removeprefix("attempt-"))
                    for attempt in node_root.iterdir()
                    if attempt.is_dir() and attempt.name.startswith("attempt-")
                )
                + 1
            )
            newer = node_root / f"attempt-{next_number:03d}"
            shutil.copytree(committed, newer)
            newer_receipt = json.loads(
                (newer / "receipt.json").read_text(encoding="utf-8")
            )
            newer_receipt["reason"] = "graph-selected newer receipt"
            write_json(newer / "receipt.json", newer_receipt)
            newer_rows = replace_graph_row(
                rows,
                node_id,
                receipt_sha256=sha256_file(newer / "receipt.json"),
            )

            receipts = validate_fixture_receipts(
                newer_rows, formal_target_root, repository_root
            )
            self.assertEqual(
                receipts[node_id]["reason"], "graph-selected newer receipt"
            )
            self.assertEqual(
                historical_receipts,
                {
                    name: (node_root / name / "receipt.json").read_bytes()
                    for name in historical_receipts
                },
            )

            duplicate = node_root / f"attempt-{next_number + 1:03d}"
            shutil.copytree(newer, duplicate)
            with self.assertRaisesRegex(
                protocol.ValidationError, "exactly one.*receipt_sha256"
            ):
                validate_fixture_receipts(
                    newer_rows, formal_target_root, repository_root
                )

    def test_committed_receipt_ignores_missing_nonmatching_history(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            formal_target_root, repository_root = copy_committed_receipts(root)
            node_root = formal_target_root / "proof-slices" / node_id
            (node_root / "attempt-000").mkdir()

            receipts = validate_receipts(rows, formal_target_root, repository_root)

            self.assertIn(node_id, receipts)

    def test_committed_receipt_rejects_symlinked_historical_entries(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        for historical_state in ("attempt", "receipt"):
            with self.subTest(historical_state=historical_state):
                with tempfile.TemporaryDirectory() as directory:
                    root = Path(directory).resolve()
                    formal_target_root, repository_root = copy_committed_receipts(root)
                    node_root = formal_target_root / "proof-slices" / node_id
                    historical = node_root / "attempt-000"
                    outside = root / "historical"
                    outside.mkdir()
                    (outside / "receipt.json").write_text("{}\n", encoding="utf-8")
                    if historical_state == "attempt":
                        historical.symlink_to(outside, target_is_directory=True)
                    else:
                        historical.mkdir()
                        (historical / "receipt.json").symlink_to(
                            outside / "receipt.json"
                        )

                    with self.assertRaisesRegex(
                        protocol.ValidationError,
                        f"{historical_state}.*symlink",
                    ):
                        validate_receipts(rows, formal_target_root, repository_root)

    def test_selected_build_swap_after_tree_check_is_rejected(self) -> None:
        rows = ls_validation.load_route_graph(GRAPH)
        node_id = "ls-equation-one-terminal-bound"
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            formal_target_root, repository_root = copy_committed_receipts(root)
            attempt = selected_attempt(rows, formal_target_root, node_id)
            replacement = root / "replacement-build"
            shutil.copytree(attempt / "build", replacement)
            displaced = root / "displaced-build"
            original = ls_validation._validate_attempt_members
            swapped = False

            def swap_after_tree_check(pinned_attempt, observed_node_id):
                nonlocal swapped
                build = original(pinned_attempt, observed_node_id)
                if observed_node_id == node_id and not swapped:
                    (attempt / "build").rename(displaced)
                    replacement.rename(attempt / "build")
                    swapped = True
                return build

            with mock.patch.object(
                ls_validation,
                "_validate_attempt_members",
                side_effect=swap_after_tree_check,
            ):
                with self.assertRaisesRegex(
                    protocol.ValidationError, "parent identity changed"
                ):
                    validate_receipts(rows, formal_target_root, repository_root)

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
            node_a = dict(
                base["nodes"][0],
                node_id="a",
                lean_name="LS.a",
                dependencies=["b"],
                role="intermediate",
            )
            node_b = dict(
                base["nodes"][0],
                node_id="b",
                lean_name="LS.b",
                dependencies=["a"],
                role="terminal",
            )
            write_json(cycle, base | {"nodes": [node_a, node_b]})
            with self.assertRaisesRegex(
                protocol.ValidationError, "canonical LS graph|cycle"
            ):
                ls_validation.load_route_graph(cycle)

            jin = root / "jin.json"
            jin_node = dict(
                base["nodes"][0],
                source_locator="git:565b6a3e0659b6e0785f783b016c3f6d9f171fa5:Lean/private.lean",
            )
            write_json(jin, base | {"nodes": [jin_node]})
            with self.assertRaisesRegex(protocol.ValidationError, "Jin"):
                ls_validation.load_route_graph(jin)

    def test_library_inventory_maps_external_facts_to_mathlib_task_or_blocker(
        self,
    ) -> None:
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

    def test_materialize_ls_tasks_records_blocked_nodes_and_terminal_assembly(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            graph = ls_validation.load_route_graph(GRAPH)

            summary = ls_validation.materialize_tasks(graph, root)

            self.assertEqual(summary["status"], "blocked")
            self.assertEqual(summary["terminal_node_id"], "ls-terminal-crouzeix")
            self.assertTrue(
                (root / "ls-equation-one-terminal-bound" / "task.json").is_file()
            )
            first_result = json.loads(
                (root / "ls-equation-one-terminal-bound" / "result.json").read_text()
            )
            self.assertEqual(first_result["status"], "passed")
            self.assertTrue((root / "ls-perturbation-lemma" / "task.json").is_file())
            self.assertTrue(
                (root / "ls-double-layer-realization" / "task.json").is_file()
            )
            self.assertTrue((root / "assembly" / "result.json").is_file())
            terminal_result = json.loads(
                (root / "assembly" / "result.json").read_text()
            )
            self.assertEqual(terminal_result["status"], "blocked")
            self.assertIn("ls-perturbation-lemma", terminal_result["blocked_by"])
            self.assertIn("ls-double-layer-realization", terminal_result["blocked_by"])

    def test_materialize_ls_tasks_rejects_unpublished_predecessor_and_jin_imports(
        self,
    ) -> None:
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
                ls_validation.materialize_tasks(
                    tuple(tampered), Path(directory).resolve()
                )

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
                    "receipt_sha256": "a" * 64,
                },
                "blocked.*receipt_sha256",
            ),
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
