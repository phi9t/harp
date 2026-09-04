from __future__ import annotations

import inspect
import json
import os
import sys
import tempfile
import unittest
from dataclasses import replace
from pathlib import Path
from typing import Callable
from unittest import mock


LAB = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(LAB))

import local_formalization_evidence  # noqa: E402
import ls_contract  # noqa: E402
import ls_receipts  # noqa: E402
import ls_validation  # noqa: E402
import protocol  # noqa: E402
import route_validation  # noqa: E402


FORMALIZATIONS = (
    (
        "harp-closed-numerical-range",
        "harp",
        "harp-closed-range-consequence",
        "CrouzeixConjecture."
        "harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet",
        "formalization/lean/Crouzeix/Harp/Consequences.lean",
    ),
    (
        "harp-main-theorem",
        "harp",
        "harp-terminal-theorem",
        "CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem",
        "formalization/lean/Crouzeix/Harp/MainTheorem.lean",
    ),
    (
        "jin-closed-numerical-range",
        "jin",
        "jin-hilbert-spectral-set-consequence",
        "CrouzeixConjecture.closedOperatorNumericalRange_isTwoSpectralSet",
        "formalization/lean/CrouzeixConjecture/HilbertSpectralSet.lean",
    ),
    (
        "jin-main-theorem",
        "jin",
        "jin-terminal-crouzeix",
        "CrouzeixConjecture.crouzeixConjecture",
        "formalization/lean/Crouzeix/Jin/Terminal.lean",
    ),
    (
        "ls-closed-numerical-range",
        "lorist-schwenninger",
        "-",
        "CrouzeixConjecture."
        "loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet",
        "formalization/lean/Crouzeix/LoristSchwenninger/Consequences.lean",
    ),
    (
        "ls-main-theorem",
        "lorist-schwenninger",
        "ls-terminal-crouzeix",
        "CrouzeixConjecture.loristSchwenningerMainTheorem",
        "formalization/lean/Crouzeix/LoristSchwenninger/MainTheorem.lean",
    ),
)
REQUIRED_MATHLIB_MODULE = "Mathlib.Algebra.Order.Ring.Defs"
TERMINAL_MEMBER_PATHS = (
    "task.json",
    "source-slice.json",
    "result.json",
    "receipt.json",
    "build/command.json",
    "build/stdout.log",
    "build/stderr.log",
    "build/axioms.txt",
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


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_json_bytes(value))


def write_module(root: Path, module: str, source: str) -> None:
    path = root / "formalization/lean" / Path(*module.split("."))
    path = path.with_suffix(".lean")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(source, encoding="utf-8")


def make_workspace(
    root: Path,
    *,
    all_ls_passed: bool = True,
) -> tuple[Path, dict[str, object], Path]:
    repository = root / "repository"
    evidence = repository / "evidence/crouzeix_conjecture"
    evidence.mkdir(parents=True)
    lean_root = repository / "formalization/lean"
    lean_root.mkdir(parents=True)

    wrapper = repository / "scripts/check_lean_library.sh"
    wrapper.parent.mkdir()
    wrapper.write_text("#!/bin/sh\nset -eu\nexit 0\n", encoding="utf-8")
    (lean_root / "lean-toolchain").write_text(
        "leanprover/lean4:v4.32.1\n", encoding="utf-8"
    )
    write_json(lean_root / "lake-manifest.json", {"packages": [], "version": "1.1.0"})
    (lean_root / "lakefile.toml").write_text(
        'name = "HarpFormalization"\n', encoding="utf-8"
    )
    (lean_root / "Crouzeix.lean").write_text(
        f"import {REQUIRED_MATHLIB_MODULE}\n"
        "import CrouzeixJin\n"
        "import Crouzeix.Harp.Consequences\n"
        "import Crouzeix.LoristSchwenninger.Consequences\n",
        encoding="utf-8",
    )

    modules = {
        "CrouzeixJin": (
            "import Crouzeix.Jin.Terminal\n"
            "import CrouzeixConjecture.HilbertSpectralSet\n"
        ),
        "Crouzeix.Jin.Terminal": (
            "import Crouzeix.Jin.Support\n\n"
            "theorem CrouzeixConjecture.crouzeixConjecture "
            ": True := by\n  trivial\n"
        ),
        "Crouzeix.Jin.Support": "def Jin.support : True := True\n",
        "Crouzeix.Harp.Consequences": (
            "import Crouzeix.Harp.MainTheorem\n\n"
            "theorem CrouzeixConjecture."
            "harpFiniteHorizonClosedOperatorNumericalRange_isTwoSpectralSet "
            ": True := by\n  trivial\n"
        ),
        "Crouzeix.Harp.FiniteAtomicL2Dilation": (
            "import Crouzeix.Harp.Support\n\ndef Harp.atomic : True := True\n"
        ),
        "Crouzeix.Harp.FiniteHorizonPerturbation": (
            "import Crouzeix.Harp.Support\n\ndef Harp.horizon : True := True\n"
        ),
        "Crouzeix.Harp.MainTheorem": (
            "import Crouzeix.Harp.Support\n\n"
            "theorem CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem "
            ": True := by\n  trivial\n"
        ),
        "Crouzeix.Harp.Support": "def Harp.support : True := True\n",
        "Crouzeix.LoristSchwenninger.Consequences": (
            "import Crouzeix.LoristSchwenninger.MainTheorem\n\n"
            "theorem CrouzeixConjecture."
            "loristSchwenningerClosedOperatorNumericalRange_isTwoSpectralSet "
            ": True := by\n  trivial\n"
        ),
        "Crouzeix.LoristSchwenninger.MainTheorem": (
            "import Crouzeix.LoristSchwenninger.Support\n\n"
            "theorem CrouzeixConjecture.loristSchwenningerMainTheorem "
            ": True := by\n  trivial\n"
        ),
        "Crouzeix.LoristSchwenninger.Support": "def LS.support : True := True\n",
        "CrouzeixConjecture.HilbertSpectralSet": (
            "import CrouzeixConjecture.HilbertSpectralSetCore\n\n"
            "theorem CrouzeixConjecture."
            "closedOperatorNumericalRange_isTwoSpectralSet "
            ": True := by\n  trivial\n"
        ),
        "CrouzeixConjecture.HilbertSpectralSetCore": (
            "import Crouzeix.Jin.Terminal\n\n"
            "theorem CrouzeixConjecture."
            "closedOperatorNumericalRange_isTwoSpectralSet_of_mainTheorem "
            ": True := by\n  trivial\n"
        ),
    }
    for contract in ls_contract.NODES:
        module = (
            Path(contract.build_target)
            .relative_to("formalization/lean")
            .with_suffix("")
        )
        module_name = ".".join(module.parts)
        modules.setdefault(
            module_name,
            f"import {REQUIRED_MATHLIB_MODULE}\n-- {contract.node_id}\n",
        )
    for module, source in modules.items():
        write_module(repository, module, source)
    aggregate_imports = [
        "import "
        + ".".join(
            Path(contract.build_target)
            .relative_to("formalization/lean")
            .with_suffix("")
            .parts
        )
        for contract in ls_contract.NODES
    ]
    write_module(
        repository,
        ls_contract.AGGREGATE_MODULE,
        "\n".join(aggregate_imports) + "\n",
    )

    required_artifact = (
        lean_root
        / ".lake/packages/mathlib/.lake/build/lib/lean"
        / Path(*REQUIRED_MATHLIB_MODULE.split("."))
    ).with_suffix(".olean")
    required_artifact.parent.mkdir(parents=True)
    required_artifact.write_bytes(b"olean-required-mathlib-artifact\n")
    dependency_probe = (
        lean_root / ".lake/packages/batteries/.lake/build/lib/lean/Batteries.olean"
    )
    dependency_probe.parent.mkdir(parents=True)
    dependency_probe.write_bytes(b"olean-dependency-cache-probe\n")

    for route_id, manifest_path in route_validation.ROUTE_MANIFEST_PATHS.items():
        receipt_path = Path(
            f"evidence/crouzeix_conjecture/routes/{route_id}/receipt.json"
        )
        review_path = Path(f"evidence/crouzeix_conjecture/reviews/{route_id}.json")
        receipt_bytes = canonical_json_bytes({"route_id": route_id, "kind": "receipt"})
        review_bytes = canonical_json_bytes({"route_id": route_id, "kind": "review"})
        (repository / receipt_path).parent.mkdir(parents=True, exist_ok=True)
        (repository / review_path).parent.mkdir(parents=True, exist_ok=True)
        (repository / receipt_path).write_bytes(receipt_bytes)
        (repository / review_path).write_bytes(review_bytes)
        write_json(
            repository / manifest_path,
            {
                "route_id": route_id,
                "receipt_path": receipt_path.as_posix(),
                "receipt_sha256": protocol.sha256_bytes(receipt_bytes),
                "review_path": review_path.as_posix(),
                "review_sha256": protocol.sha256_bytes(review_bytes),
            },
        )

    target = (
        repository
        / "labs/crouzeix_proof_reproduction/formal_targets/lorist-schwenninger"
    )
    (target / "proof-slices").mkdir(parents=True)
    rows = tuple(
        ls_validation.LSGraphRow(
            node_id=contract.node_id,
            source_locator=contract.source_locator,
            statement_sha256=contract.statement_sha256,
            lean_name=contract.declaration,
            dependencies=contract.dependencies,
            role=contract.role,
            status="mapped",
        )
        for contract in ls_contract.NODES
    )
    published = ls_receipts.publish_ls_receipts(
        rows, repository, target, executor=FakeExecutor()
    )
    passed_rows = tuple(
        replace(
            row,
            lean_name=published[row.node_id]["lean_name"],
            status="passed",
            receipt_sha256=published[row.node_id]["receipt_sha256"],
        )
        for row in rows
    )
    terminal_publication = published["ls-terminal-crouzeix"]
    attempt = repository / terminal_publication["attempt_path"]
    terminal_receipt = json.loads(
        (attempt / "receipt.json").read_text(encoding="utf-8")
    )
    graph_path = target / "source-graph.json"
    graph_nodes: list[dict[str, object]] = []
    for row in passed_rows:
        graph_node: dict[str, object] = {
            "dependencies": list(row.dependencies),
            "lean_name": row.lean_name,
            "node_id": row.node_id,
            "role": row.role,
            "source_locator": row.source_locator,
            "statement_sha256": row.statement_sha256,
            "status": row.status,
        }
        if all_ls_passed or row.node_id == "ls-terminal-crouzeix":
            graph_node["receipt_sha256"] = row.receipt_sha256
        else:
            graph_node["status"] = "mapped"
        graph_nodes.append(graph_node)
    write_json(
        graph_path,
        {
            "nodes": graph_nodes,
            "schema_version": ls_contract.SCHEMA_VERSION,
            "source_id": ls_contract.SOURCE_ID,
            "source_identity": ls_contract.SOURCE_IDENTITY,
        },
    )
    return repository, terminal_receipt, attempt


class FakeExecutor:
    def __init__(
        self,
        *,
        fail_declaration: str | None = None,
        aggregate_result: ls_receipts.CommandResult | None = None,
        observed_axioms: str = "propext, Classical.choice, Quot.sound",
        after_call: Callable[[int, list[str]], None] | None = None,
    ) -> None:
        self.fail_declaration = fail_declaration
        self.aggregate_result = aggregate_result
        self.observed_axioms = observed_axioms
        self.after_call = after_call
        self.calls: list[dict[str, object]] = []

    def __call__(
        self,
        argv: list[str],
        cwd: Path,
        environment: dict[str, str],
        timeout_seconds: int,
        max_output_bytes: int,
    ) -> ls_receipts.CommandResult:
        self.calls.append(
            {
                "argv": list(argv),
                "cwd": cwd,
                "environment": dict(environment),
                "timeout_seconds": timeout_seconds,
                "max_output_bytes": max_output_bytes,
            }
        )
        if argv in (
            ["scripts/check_lean_library.sh", "Crouzeix"],
            ["scripts/check_lean_library.sh", ls_contract.AGGREGATE_MODULE],
        ):
            target = argv[-1]
            result = self.aggregate_result or ls_receipts.CommandResult(
                0,
                (
                    f"[lean] target={target}\n"
                    "[lean] root=formalization/lean\n"
                    "[lean] outcome=passed\n"
                ).encode(),
                b"",
            )
        else:
            audit_source = (cwd / argv[-1]).read_text(encoding="utf-8")
            declaration = audit_source.split("#print axioms ", 1)[1].strip()
            if declaration == self.fail_declaration:
                result = ls_receipts.CommandResult(1, b"", b"audit failed\n")
            else:
                result = ls_receipts.CommandResult(
                    0,
                    (
                        f"'{declaration}' depends on axioms: [{self.observed_axioms}]\n"
                    ).encode(),
                    b"",
                )
        if self.after_call is not None:
            self.after_call(len(self.calls), argv)
        return result


def stage_directories(repository: Path) -> list[Path]:
    stage_root = repository / ".build"
    if not stage_root.exists():
        return []
    return sorted(
        path
        for path in stage_root.iterdir()
        if path.name.startswith(".local-formalization-stage-")
    )


class LocalFormalizationEvidenceTests(unittest.TestCase):
    def setUp(self) -> None:
        for name, side_effect in (
            (
                "inspect_route",
                lambda _root, route_id: route_validation.RouteValidationResult(
                    route_id,
                    "complete-local",
                    "complete",
                    route_validation.ROUTE_MANIFEST_PATHS[route_id].as_posix(),
                ),
            ),
            (
                "_read_json",
                lambda root, path, _label: json.loads(
                    (Path(root) / path).read_text(encoding="utf-8")
                ),
            ),
        ):
            patcher = mock.patch.object(
                local_formalization_evidence.route_validation,
                name,
                side_effect=side_effect,
            )
            patcher.start()
            self.addCleanup(patcher.stop)

    def test_uses_a_local_active_source_scan_bound(self) -> None:
        self.assertEqual(
            local_formalization_evidence.MAX_ACTIVE_SOURCE_FILES,
            4096,
        )

    def test_shared_xdg_environment_uses_existing_pinned_toolchain(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            toolchain_bin = (
                root
                / "harp/lean/elan/toolchains/leanprover--lean4---v4.32.1/bin"
            )
            toolchain_bin.mkdir(parents=True)
            for name in ("lake", "lean"):
                executable = toolchain_bin / name
                executable.write_text("#!/bin/sh\n", encoding="utf-8")
                executable.chmod(0o755)

            with mock.patch.dict(
                os.environ,
                {"HOME": "/unused", "XDG_CACHE_HOME": str(root)},
                clear=True,
            ):
                environment = local_formalization_evidence._shared_xdg_environment()

            self.assertEqual(
                environment,
                {
                    "ELAN_HOME": (root / "harp/lean/elan").as_posix(),
                    "ELAN_TOOLCHAIN": "leanprover/lean4:v4.32.1",
                    "HARP_ELAN_HOME": (root / "harp/lean/elan").as_posix(),
                    "HARP_LEAN_CACHE_ROOT": (
                        root / "harp/lean/lean-4.32.1"
                    ).as_posix(),
                    "PATH": os.pathsep.join(
                        (toolchain_bin.as_posix(), "/usr/bin", "/bin")
                    ),
                },
            )

    def test_shared_xdg_environment_rejects_missing_or_relative_roots(self) -> None:
        with mock.patch.dict(os.environ, {}, clear=True):
            with self.assertRaisesRegex(protocol.ValidationError, "HOME must be set"):
                local_formalization_evidence._shared_xdg_environment()

        with mock.patch.dict(
            os.environ,
            {"HOME": "/unused", "XDG_CACHE_HOME": "relative"},
            clear=True,
        ):
            with self.assertRaisesRegex(
                protocol.ValidationError, "XDG_CACHE_HOME must be absolute"
            ):
                local_formalization_evidence._shared_xdg_environment()

    def test_ls_policy_matches_shell_forbidden_prefixes(self) -> None:
        self.assertEqual(
            ls_contract.FORBIDDEN_PREFIXES,
            ("Crouzeix.Harp", "Crouzeix.Jin"),
        )

    def test_mathlib_artifact_snapshot_rejects_nested_symlink(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            repository, _, _ = make_workspace(root)
            lean_root = repository / "formalization/lean"
            sources = local_formalization_evidence._active_source_snapshots(repository)
            target = lean_root / ".lake/packages/mathlib/.lake/build/lib"
            outside = root / "outside-mathlib-lib"
            target.rename(outside)
            target.symlink_to(outside, target_is_directory=True)

            with self.assertRaisesRegex(
                protocol.ValidationError, "cannot be a symlink|contains symlink"
            ):
                local_formalization_evidence._required_mathlib_artifact_snapshots(
                    lean_root, sources
                )

    def test_provider_reports_reject_missing_transitive_managed_local_import(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, _, _ = make_workspace(Path(directory).resolve())
            lean_root = repository / "formalization/lean"
            write_module(
                repository,
                "Crouzeix.LoristSchwenninger.MainTheorem",
                "import Fixture.Helper\n",
            )
            write_module(
                repository,
                "Fixture.Helper",
                "import CrouzeixConjecture.Missing\n",
            )

            with self.assertRaisesRegex(
                protocol.ValidationError, "local module does not exist"
            ):
                local_formalization_evidence._provider_reports(lean_root)

    def test_provider_reports_reject_ls_harp_namespace_prefix_but_preserve_harp_ls_reuse(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, _, _ = make_workspace(Path(directory).resolve())
            lean_root = repository / "formalization/lean"
            write_module(
                repository,
                ls_contract.AGGREGATE_MODULE,
                "import Crouzeix.Harp.Support\n",
            )

            with self.assertRaisesRegex(
                protocol.ValidationError, "forbidden module Crouzeix.Harp.Support"
            ):
                local_formalization_evidence._provider_reports(lean_root)

        with tempfile.TemporaryDirectory() as directory:
            repository, _, _ = make_workspace(Path(directory).resolve())
            lean_root = repository / "formalization/lean"
            write_module(
                repository,
                "Crouzeix.Harp.MainTheorem",
                "import Crouzeix.LoristSchwenninger.Scalar\n\n"
                "theorem CrouzeixConjecture.Harp.harpFiniteHorizonMainTheorem "
                ": True := by\n  trivial\n",
            )

            reports = local_formalization_evidence._provider_reports(lean_root)

            harp_modules = {item["module_name"] for item in reports["harp"]["modules"]}
            self.assertIn(
                "Crouzeix.LoristSchwenninger.Scalar",
                harp_modules,
            )
            self.assertNotIn(
                "Crouzeix.Harp.Support",
                {
                    item["module_name"]
                    for item in reports["lorist-schwenninger"]["modules"]
                },
            )

    def publish(
        self,
        repository: Path,
        terminal_receipt: dict[str, object],
        *,
        executor: FakeExecutor,
        validate_real: bool = False,
    ) -> local_formalization_evidence.Publication:
        if validate_real:
            return local_formalization_evidence.publish_local_formalization_evidence(
                repository, executor=executor
            )
        with mock.patch.object(
            local_formalization_evidence.ls_validation,
            "validate_committed_receipts",
            return_value={"ls-terminal-crouzeix": terminal_receipt},
        ) as validate:
            publication = (
                local_formalization_evidence.publish_local_formalization_evidence(
                    repository, executor=executor
                )
            )
        self.assertEqual(validate.call_count, 3)
        return publication

    def test_publication_roster_is_the_exact_sorted_six_row_contract(self) -> None:
        self.assertEqual(
            [item[0] for item in FORMALIZATIONS],
            [
                "harp-closed-numerical-range",
                "harp-main-theorem",
                "jin-closed-numerical-range",
                "jin-main-theorem",
                "ls-closed-numerical-range",
                "ls-main-theorem",
            ],
        )

    def test_publishes_one_exact_bundle_from_one_aggregate_and_six_audits(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, attempt = make_workspace(
                Path(directory).resolve()
            )
            import_receipt = repository / "docs/import-receipt.md"
            import_receipt.parent.mkdir()
            import_receipt.write_text("do not mutate\n", encoding="utf-8")
            graph_path = attempt.parents[2] / "source-graph.json"
            graph_before = graph_path.read_bytes()
            legacy = repository / "evidence/crouzeix_conjecture/legacy.txt"
            legacy.write_text("preserve\n", encoding="utf-8")
            legacy_identity = (legacy.stat().st_dev, legacy.stat().st_ino)
            executor = FakeExecutor()

            publication = self.publish(
                repository,
                terminal_receipt,
                executor=executor,
                validate_real=True,
            )

            self.assertEqual(len(executor.calls), 7)
            self.assertEqual(
                executor.calls[0]["argv"],
                ["scripts/check_lean_library.sh", "Crouzeix"],
            )
            self.assertTrue(
                all(
                    call["argv"][:3] == ["lake", "env", "lean"]
                    for call in executor.calls[1:]
                )
            )
            self.assertEqual(
                [(call["cwd"] / call["argv"][-1]).name for call in executor.calls[1:]],
                ["Audit.lean"] * 6,
            )
            self.assertEqual(graph_path.read_bytes(), graph_before)
            self.assertEqual(
                import_receipt.read_text(encoding="utf-8"), "do not mutate\n"
            )
            self.assertEqual(legacy.read_text(encoding="utf-8"), "preserve\n")
            self.assertEqual(
                (legacy.stat().st_dev, legacy.stat().st_ino), legacy_identity
            )

            expected_root = (
                repository / "evidence/crouzeix_conjecture/local_formalization"
            )
            self.assertEqual(publication.artifact_root, expected_root)
            self.assertEqual(publication.manifest_path, expected_root / "manifest.tsv")
            expected_files = {
                "manifest.tsv",
                "build/command.json",
                "build/stdout.log",
                "build/stderr.log",
                "providers/harp.json",
                "providers/jin.json",
                "providers/lorist-schwenninger.json",
                *(
                    f"routes/{route_id}.{kind}.json"
                    for route_id in route_validation.ROUTE_IDS
                    for kind in ("manifest", "receipt", "review")
                ),
                *(f"axioms/{item[0]}.txt" for item in FORMALIZATIONS),
            }
            self.assertEqual(
                {
                    path.relative_to(publication.artifact_root).as_posix()
                    for path in publication.artifact_root.rglob("*")
                    if path.is_file()
                },
                expected_files,
            )
            self.assertFalse(
                (
                    repository
                    / "evidence/crouzeix_conjecture/local_formalization_manifest.tsv"
                ).exists()
            )
            self.assertEqual(stage_directories(repository), [])

            manifest_lines = publication.manifest_path.read_text(
                encoding="utf-8"
            ).splitlines()
            self.assertEqual(manifest_lines[0], local_formalization_evidence.HEADER)
            rows = [line.split("\t") for line in manifest_lines[1:]]
            self.assertEqual(len(rows), 6)
            self.assertTrue(all(len(row) == 29 for row in rows))
            self.assertEqual(
                [row[1] for row in rows], [item[0] for item in FORMALIZATIONS]
            )
            self.assertEqual(
                [row[2] for row in rows], [item[1] for item in FORMALIZATIONS]
            )
            self.assertEqual(
                [row[3] for row in rows], [item[2] for item in FORMALIZATIONS]
            )
            self.assertTrue(
                all(row[21] == "Classical.choice,Quot.sound,propext" for row in rows)
            )
            self.assertTrue(
                all(row[22] == "Classical.choice,Quot.sound,propext" for row in rows)
            )
            for row in rows:
                for path_index, digest_index in ((7, 8), (9, 10), (11, 12)):
                    self.assertEqual(
                        row[digest_index],
                        protocol.sha256_bytes((repository / row[path_index]).read_bytes()),
                    )
            for row in rows:
                for path_index, digest_index in (
                    (13, 14),
                    (15, 16),
                    (17, 18),
                    (19, 20),
                    (23, 24),
                ):
                    self.assertTrue(
                        row[path_index].startswith(
                            "evidence/crouzeix_conjecture/local_formalization/"
                        ),
                        row[path_index],
                    )
                    self.assertEqual(
                        row[digest_index],
                        protocol.sha256_bytes(
                            (repository / row[path_index]).read_bytes()
                        ),
                    )

            command = json.loads(
                (publication.artifact_root / "build/command.json").read_text()
            )
            lean_root = repository / "formalization/lean"
            active_sources = [lean_root / "Crouzeix.lean"]
            for source_root in (
                lean_root / "Crouzeix",
                lean_root / "CrouzeixConjecture",
            ):
                active_sources.extend(sorted(source_root.rglob("*.lean")))
            active_sources.sort()
            source_records = [
                {
                    "path": path.relative_to(repository).as_posix(),
                    "sha256": protocol.sha256_bytes(path.read_bytes()),
                }
                for path in active_sources
            ]
            artifact_records = [
                {
                    "module": REQUIRED_MATHLIB_MODULE,
                    "sha256": protocol.sha256_bytes(
                        (
                            lean_root
                            / ".lake/packages/mathlib/.lake/build/lib/lean"
                            / Path(*REQUIRED_MATHLIB_MODULE.split("."))
                        )
                        .with_suffix(".olean")
                        .read_bytes()
                    ),
                }
            ]
            cache_digest = ls_receipts._snapshot_digest(
                ls_receipts._dependency_cache_metadata_snapshot(lean_root)
            )
            self.assertEqual(
                set(command),
                {
                    "schema_version",
                    "argv",
                    "cwd",
                    "lean_toolchain",
                    "lean_toolchain_sha256",
                    "lake_manifest_sha256",
                    "wrapper_sha256",
                    "lakefile_sha256",
                    "active_source_closure_sha256",
                    "dependency_cache_metadata_sha256",
                    "required_mathlib_artifacts_sha256",
                    "ls_graph_sha256",
                    "exit_code",
                    "status",
                    "stdout_path",
                    "stdout_sha256",
                    "stderr_path",
                    "stderr_sha256",
                },
            )
            self.assertEqual(
                command["wrapper_sha256"],
                protocol.sha256_bytes(
                    (repository / "scripts/check_lean_library.sh").read_bytes()
                ),
            )
            self.assertEqual(
                command["lakefile_sha256"],
                protocol.sha256_bytes((lean_root / "lakefile.toml").read_bytes()),
            )
            self.assertEqual(
                command["active_source_closure_sha256"],
                protocol.sha256_bytes(
                    canonical_json_bytes({"sources": source_records})
                ),
            )
            self.assertEqual(command["dependency_cache_metadata_sha256"], cache_digest)
            self.assertEqual(
                command["required_mathlib_artifacts_sha256"],
                protocol.sha256_bytes(
                    canonical_json_bytes({"artifacts": artifact_records})
                ),
            )
            self.assertEqual(
                command["ls_graph_sha256"],
                protocol.sha256_bytes(graph_before),
            )
            self.assertEqual(
                command["stdout_path"],
                "evidence/crouzeix_conjecture/local_formalization/build/stdout.log",
            )
            self.assertEqual(
                command["stderr_path"],
                "evidence/crouzeix_conjecture/local_formalization/build/stderr.log",
            )

    def test_public_api_does_not_accept_precomputed_build_evidence(self) -> None:
        parameters = inspect.signature(
            local_formalization_evidence.publish_local_formalization_evidence
        ).parameters
        self.assertNotIn("aggregate_build", parameters)
        self.assertFalse(hasattr(local_formalization_evidence, "SuccessfulBuildPaths"))

    def test_route_snapshot_rejects_unsafe_manifest_path_before_read(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, _, _ = make_workspace(Path(directory).resolve())
            manifest_path = route_validation.ROUTE_MANIFEST_PATHS["harp"]
            manifest = json.loads((repository / manifest_path).read_text(encoding="utf-8"))
            manifest["receipt_path"] = "../outside/receipt.json"
            write_json(repository / manifest_path, manifest)

            with self.assertRaisesRegex(
                protocol.ValidationError, "manifest is invalid|unsafe|path"
            ):
                local_formalization_evidence._snapshot_route_evidence(repository)

    def test_rejects_executed_build_without_exact_success_markers(self) -> None:
        bad_outputs = (
            b"[lean] target=Crouzeix\n[lean] root=formalization/lean\n[lean] outcome=skipped\n",
            b"[lean] target=Crouzeix\n[lean] root=formalization/lean\n[lean] outcome=passed with warnings\n",
            b"[lean] target=Crouzeix\n[lean] root=formalization/lean\n[lean] outcome=passed\n[lean] outcome=failed\n",
            b"[lean] target=Crouzeix\n[lean] root=formalization/lean\n[lean] outcome=passed\n[lean] outcome=passed\n",
        )
        for output in bad_outputs:
            with (
                self.subTest(output=output),
                tempfile.TemporaryDirectory() as directory,
            ):
                repository, terminal_receipt, _ = make_workspace(
                    Path(directory).resolve()
                )
                executor = FakeExecutor(
                    aggregate_result=ls_receipts.CommandResult(0, output, b"")
                )
                with (
                    mock.patch.object(
                        local_formalization_evidence.ls_validation,
                        "validate_committed_receipts",
                        return_value={"ls-terminal-crouzeix": terminal_receipt},
                    ),
                    self.assertRaisesRegex(
                        protocol.ValidationError, "successful Crouzeix build report"
                    ),
                ):
                    local_formalization_evidence.publish_local_formalization_evidence(
                        repository, executor=executor
                    )
                self.assertEqual(len(executor.calls), 1)
                self.assertFalse(
                    (
                        repository / "evidence/crouzeix_conjecture/local_formalization"
                    ).exists()
                )
                self.assertEqual(stage_directories(repository), [])

    def test_rejects_build_state_changes_before_publication(self) -> None:
        mutations = {
            "wrapper": lambda repository: (
                repository / "scripts/check_lean_library.sh"
            ).write_bytes(b"#!/bin/sh\nset -eu\n# changed\nexit 0\n"),
            "toolchain": lambda repository: (
                repository / "formalization/lean/lean-toolchain"
            ).write_bytes(b"leanprover/lean4:v4.32.2\n"),
            "lake manifest": lambda repository: (
                repository / "formalization/lean/lake-manifest.json"
            ).write_bytes(b'{"packages":["changed"],"version":"1.1.0"}\n'),
            "lakefile": lambda repository: (
                repository / "formalization/lean/lakefile.toml"
            ).write_bytes(b'name = "Changed"\n'),
            "source": lambda repository: (
                repository / "formalization/lean/Crouzeix/Harp/Support.lean"
            ).write_bytes(b"def Harp.support : True := by trivial\n"),
            "dependency cache": lambda repository: (
                repository
                / "formalization/lean/.lake/packages/batteries/.lake/build/lib/lean/Batteries.olean"
            ).write_bytes(b"changed dependency cache\n"),
            "required Mathlib artifacts": lambda repository: (
                repository
                / "formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean"
                / Path(*REQUIRED_MATHLIB_MODULE.split("."))
            )
            .with_suffix(".olean")
            .write_bytes(b"changed Mathlib artifact\n"),
        }
        for label, mutate in mutations.items():
            with self.subTest(label=label), tempfile.TemporaryDirectory() as directory:
                repository, terminal_receipt, _ = make_workspace(
                    Path(directory).resolve()
                )

                def after_call(number: int, _: list[str]) -> None:
                    if number == 1:
                        mutate(repository)

                with (
                    mock.patch.object(
                        local_formalization_evidence.ls_validation,
                        "validate_committed_receipts",
                        return_value={"ls-terminal-crouzeix": terminal_receipt},
                    ),
                    self.assertRaisesRegex(
                        protocol.ValidationError, "changed during evidence generation"
                    ),
                ):
                    local_formalization_evidence.publish_local_formalization_evidence(
                        repository, executor=FakeExecutor(after_call=after_call)
                    )
                self.assertFalse(
                    (
                        repository / "evidence/crouzeix_conjecture/local_formalization"
                    ).exists()
                )
                self.assertEqual(stage_directories(repository), [])

    def test_rejects_same_byte_lakefile_replacement(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, _ = make_workspace(Path(directory).resolve())
            lakefile = repository / "formalization/lean/lakefile.toml"

            def after_call(number: int, _: list[str]) -> None:
                if number == 1:
                    replacement = lakefile.with_suffix(".replacement")
                    replacement.write_bytes(lakefile.read_bytes())
                    replacement.replace(lakefile)

            with (
                mock.patch.object(
                    local_formalization_evidence.ls_validation,
                    "validate_committed_receipts",
                    return_value={"ls-terminal-crouzeix": terminal_receipt},
                ),
                self.assertRaisesRegex(protocol.ValidationError, "build state changed"),
            ):
                local_formalization_evidence.publish_local_formalization_evidence(
                    repository, executor=FakeExecutor(after_call=after_call)
                )

    def test_rejects_terminal_receipt_member_or_graph_change_before_commit(
        self,
    ) -> None:
        for label, relative in (
            ("member", "build/stdout.log"),
            ("graph", None),
        ):
            with self.subTest(label=label), tempfile.TemporaryDirectory() as directory:
                repository, terminal_receipt, attempt = make_workspace(
                    Path(directory).resolve()
                )
                graph = attempt.parents[2] / "source-graph.json"

                def after_call(number: int, _: list[str]) -> None:
                    if number != 1:
                        return
                    target = graph if relative is None else attempt / relative
                    target.write_bytes(target.read_bytes() + b"changed\n")

                with (
                    mock.patch.object(
                        local_formalization_evidence.ls_validation,
                        "validate_committed_receipts",
                        return_value={"ls-terminal-crouzeix": terminal_receipt},
                    ),
                    self.assertRaisesRegex(
                        protocol.ValidationError, "LS terminal .* changed"
                    ),
                ):
                    local_formalization_evidence.publish_local_formalization_evidence(
                        repository, executor=FakeExecutor(after_call=after_call)
                    )
                self.assertFalse(
                    (
                        repository / "evidence/crouzeix_conjecture/local_formalization"
                    ).exists()
                )

    def test_rejects_terminal_only_passed_ls_graph_before_receipt_validation(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, _, _ = make_workspace(
                Path(directory).resolve(), all_ls_passed=False
            )
            module = repository / FORMALIZATIONS[-1][4]

            with self.assertRaisesRegex(
                protocol.ValidationError,
                "all canonical LS graph rows must be passed and receipt-bound",
            ):
                local_formalization_evidence._snapshot_terminal_evidence(
                    repository, protocol.sha256_bytes(module.read_bytes())
                )

    def test_failed_or_disallowed_audit_leaves_no_artifacts(self) -> None:
        executors = (
            FakeExecutor(fail_declaration=FORMALIZATIONS[1][3]),
            FakeExecutor(observed_axioms="Classical.choice, Fixture.axiom"),
        )
        for executor in executors:
            with (
                self.subTest(executor=executor),
                tempfile.TemporaryDirectory() as directory,
            ):
                repository, terminal_receipt, _ = make_workspace(
                    Path(directory).resolve()
                )
                with (
                    mock.patch.object(
                        local_formalization_evidence.ls_validation,
                        "validate_committed_receipts",
                        return_value={"ls-terminal-crouzeix": terminal_receipt},
                    ),
                    self.assertRaises(protocol.ValidationError),
                ):
                    local_formalization_evidence.publish_local_formalization_evidence(
                        repository, executor=executor
                    )
                self.assertFalse(
                    (
                        repository / "evidence/crouzeix_conjecture/local_formalization"
                    ).exists()
                )
                self.assertEqual(stage_directories(repository), [])

    def test_refuses_existing_or_symlinked_destination_before_commands(self) -> None:
        for symlink in (False, True):
            with (
                self.subTest(symlink=symlink),
                tempfile.TemporaryDirectory() as directory,
            ):
                root = Path(directory).resolve()
                repository, terminal_receipt, _ = make_workspace(root)
                destination = (
                    repository / "evidence/crouzeix_conjecture/local_formalization"
                )
                if symlink:
                    outside = root / "outside"
                    outside.mkdir()
                    destination.symlink_to(outside, target_is_directory=True)
                else:
                    destination.mkdir()
                    (destination / "owned.txt").write_text(
                        "preserve\n", encoding="utf-8"
                    )
                executor = FakeExecutor()

                with (
                    mock.patch.object(
                        local_formalization_evidence.ls_validation,
                        "validate_committed_receipts",
                        return_value={"ls-terminal-crouzeix": terminal_receipt},
                    ),
                    self.assertRaisesRegex(protocol.ValidationError, "already exists"),
                ):
                    local_formalization_evidence.publish_local_formalization_evidence(
                        repository, executor=executor
                    )

                self.assertEqual(executor.calls, [])
                if symlink:
                    self.assertTrue(destination.is_symlink())
                else:
                    self.assertEqual(
                        (destination / "owned.txt").read_text(encoding="utf-8"),
                        "preserve\n",
                    )

    def test_external_publication_lock_rejects_concurrent_publisher(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, _ = make_workspace(Path(directory).resolve())
            evidence = repository / "evidence/crouzeix_conjecture"
            executor = FakeExecutor()
            publication_lock = ls_receipts._acquire_publication_lock(evidence)
            try:
                with (
                    mock.patch.object(
                        local_formalization_evidence.ls_validation,
                        "validate_committed_receipts",
                        return_value={"ls-terminal-crouzeix": terminal_receipt},
                    ),
                    self.assertRaisesRegex(
                        protocol.ValidationError, "publisher is active"
                    ),
                ):
                    local_formalization_evidence.publish_local_formalization_evidence(
                        repository, executor=executor
                    )
            finally:
                ls_receipts._release_publication_lock(publication_lock)
            self.assertEqual(executor.calls, [])

    def test_no_replace_race_preserves_competing_destination_and_cleans_stage(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, _ = make_workspace(Path(directory).resolve())
            evidence = repository / "evidence/crouzeix_conjecture"
            destination = evidence / "local_formalization"
            original = local_formalization_evidence._rename_directory_no_replace
            injected = False

            def race(
                source_parent: ls_receipts._PinnedDirectory,
                source: ls_receipts._PinnedDirectory,
                destination_parent: ls_receipts._PinnedDirectory,
                destination_name: str,
            ) -> ls_receipts._PinnedDirectory:
                nonlocal injected
                os.mkdir(
                    destination_name, mode=0o700, dir_fd=destination_parent.descriptor
                )
                competitor = os.open(
                    "competitor.txt",
                    os.O_WRONLY | os.O_CREAT | os.O_EXCL,
                    0o600,
                    dir_fd=os.open(
                        destination_name,
                        os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW,
                        dir_fd=destination_parent.descriptor,
                    ),
                )
                os.write(competitor, b"competitor\n")
                os.close(competitor)
                injected = True
                return original(
                    source_parent, source, destination_parent, destination_name
                )

            with (
                mock.patch.object(
                    local_formalization_evidence,
                    "_rename_directory_no_replace",
                    side_effect=race,
                ),
                mock.patch.object(
                    local_formalization_evidence.ls_validation,
                    "validate_committed_receipts",
                    return_value={"ls-terminal-crouzeix": terminal_receipt},
                ),
                self.assertRaisesRegex(
                    protocol.ValidationError, "appeared during publication"
                ),
            ):
                local_formalization_evidence.publish_local_formalization_evidence(
                    repository, executor=FakeExecutor()
                )

            self.assertTrue(injected)
            self.assertEqual(
                (destination / "competitor.txt").read_text(encoding="utf-8"),
                "competitor\n",
            )
            self.assertEqual(stage_directories(repository), [])

    def test_destination_parent_swap_during_execution_cannot_redirect_publication(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            repository, terminal_receipt, _ = make_workspace(root)
            evidence = repository / "evidence/crouzeix_conjecture"
            displaced = root / "displaced-evidence"
            marker = evidence / "replacement-marker.txt"

            def after_call(number: int, _: list[str]) -> None:
                if number == 1:
                    evidence.rename(displaced)
                    evidence.mkdir()
                    marker.write_text("replacement\n", encoding="utf-8")

            with (
                mock.patch.object(
                    local_formalization_evidence.ls_validation,
                    "validate_committed_receipts",
                    return_value={"ls-terminal-crouzeix": terminal_receipt},
                ),
                self.assertRaisesRegex(
                    protocol.ValidationError, "evidence bundle parent identity changed"
                ),
            ):
                local_formalization_evidence.publish_local_formalization_evidence(
                    repository, executor=FakeExecutor(after_call=after_call)
                )

            self.assertEqual(marker.read_text(encoding="utf-8"), "replacement\n")
            self.assertFalse((evidence / "local_formalization").exists())
            self.assertFalse((displaced / "local_formalization").exists())
            self.assertEqual(stage_directories(repository), [])

    def test_symlink_destination_race_is_not_followed_or_overwritten(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            repository, terminal_receipt, _ = make_workspace(root)
            evidence = repository / "evidence/crouzeix_conjecture"
            destination = evidence / "local_formalization"
            outside = root / "outside"
            outside.mkdir()
            original = local_formalization_evidence._rename_directory_no_replace

            def race(
                source_parent: ls_receipts._PinnedDirectory,
                source: ls_receipts._PinnedDirectory,
                destination_parent: ls_receipts._PinnedDirectory,
                destination_name: str,
            ) -> ls_receipts._PinnedDirectory:
                os.symlink(
                    outside,
                    destination_name,
                    target_is_directory=True,
                    dir_fd=destination_parent.descriptor,
                )
                return original(
                    source_parent, source, destination_parent, destination_name
                )

            with (
                mock.patch.object(
                    local_formalization_evidence,
                    "_rename_directory_no_replace",
                    side_effect=race,
                ),
                mock.patch.object(
                    local_formalization_evidence.ls_validation,
                    "validate_committed_receipts",
                    return_value={"ls-terminal-crouzeix": terminal_receipt},
                ),
                self.assertRaisesRegex(
                    protocol.ValidationError, "appeared during publication"
                ),
            ):
                local_formalization_evidence.publish_local_formalization_evidence(
                    repository, executor=FakeExecutor()
                )

            self.assertTrue(destination.is_symlink())
            self.assertEqual(list(outside.iterdir()), [])
            self.assertEqual(stage_directories(repository), [])

    def test_final_recursive_fsync_precedes_rename_and_both_parents_are_synced(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, _ = make_workspace(Path(directory).resolve())
            events: list[tuple[str, str]] = []
            original_tree = local_formalization_evidence._fsync_tree
            original_rename = local_formalization_evidence._rename_directory_no_replace
            original_parent = local_formalization_evidence._fsync_pinned_directory

            def record_tree(directory: ls_receipts._PinnedDirectory) -> None:
                original_tree(directory)
                events.append(("tree", directory.path.name))

            def record_rename(
                source_parent: ls_receipts._PinnedDirectory,
                source: ls_receipts._PinnedDirectory,
                destination_parent: ls_receipts._PinnedDirectory,
                destination_name: str,
            ) -> ls_receipts._PinnedDirectory:
                events.append(("rename", destination_name))
                return original_rename(
                    source_parent, source, destination_parent, destination_name
                )

            def record_parent(
                directory: ls_receipts._PinnedDirectory, label: str
            ) -> None:
                events.append(("parent", directory.path.as_posix()))
                original_parent(directory, label)

            with (
                mock.patch.object(
                    local_formalization_evidence, "_fsync_tree", side_effect=record_tree
                ),
                mock.patch.object(
                    local_formalization_evidence,
                    "_rename_directory_no_replace",
                    side_effect=record_rename,
                ),
                mock.patch.object(
                    local_formalization_evidence,
                    "_fsync_pinned_directory",
                    side_effect=record_parent,
                ),
            ):
                self.publish(repository, terminal_receipt, executor=FakeExecutor())

            tree_index = next(
                index for index, event in enumerate(events) if event[0] == "tree"
            )
            rename_index = next(
                index for index, event in enumerate(events) if event[0] == "rename"
            )
            parent_events = [event[1] for event in events if event[0] == "parent"]
            self.assertEqual(rename_index, tree_index + 1)
            self.assertEqual(
                parent_events,
                [
                    (repository / ".build").as_posix(),
                    (repository / "evidence/crouzeix_conjecture").as_posix(),
                ],
            )

    def test_build_and_terminal_state_are_rechecked_immediately_before_fsync(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, _ = make_workspace(Path(directory).resolve())
            events: list[str] = []
            original_build = local_formalization_evidence._require_build_state
            original_terminal = local_formalization_evidence._require_terminal_evidence
            original_tree = local_formalization_evidence._fsync_tree
            original_rename = local_formalization_evidence._rename_directory_no_replace

            def record_build(*args: object) -> None:
                original_build(*args)
                events.append("build")

            def record_terminal(*args: object) -> None:
                original_terminal(*args)
                events.append("terminal")

            def record_tree(*args: object) -> None:
                original_tree(*args)
                events.append("fsync-tree")

            def record_rename(*args: object) -> ls_receipts._PinnedDirectory:
                events.append("rename")
                return original_rename(*args)

            with (
                mock.patch.object(
                    local_formalization_evidence,
                    "_require_build_state",
                    side_effect=record_build,
                ),
                mock.patch.object(
                    local_formalization_evidence,
                    "_require_terminal_evidence",
                    side_effect=record_terminal,
                ),
                mock.patch.object(
                    local_formalization_evidence, "_fsync_tree", side_effect=record_tree
                ),
                mock.patch.object(
                    local_formalization_evidence,
                    "_rename_directory_no_replace",
                    side_effect=record_rename,
                ),
            ):
                self.publish(repository, terminal_receipt, executor=FakeExecutor())

            self.assertEqual(
                events[-6:],
                [
                    "build",
                    "terminal",
                    "fsync-tree",
                    "build",
                    "terminal",
                    "rename",
                ],
            )

    def test_rejects_bound_input_or_staged_member_changed_after_fsync(self) -> None:
        for label in ("input", "staged member"):
            with self.subTest(label=label), tempfile.TemporaryDirectory() as directory:
                repository, terminal_receipt, _ = make_workspace(
                    Path(directory).resolve()
                )
                original_tree = local_formalization_evidence._fsync_tree

                def mutate_after_fsync(bundle: ls_receipts._PinnedDirectory) -> None:
                    original_tree(bundle)
                    if label == "input":
                        target = (
                            repository / "formalization/lean/Crouzeix/Harp/Support.lean"
                        )
                    else:
                        target = bundle.path / "build/stdout.log"
                    target.write_bytes(target.read_bytes() + b"changed after fsync\n")

                with (
                    mock.patch.object(
                        local_formalization_evidence,
                        "_fsync_tree",
                        side_effect=mutate_after_fsync,
                    ),
                    mock.patch.object(
                        local_formalization_evidence.ls_validation,
                        "validate_committed_receipts",
                        return_value={"ls-terminal-crouzeix": terminal_receipt},
                    ),
                    self.assertRaisesRegex(
                        protocol.ValidationError, "changed|bundle member"
                    ),
                ):
                    local_formalization_evidence.publish_local_formalization_evidence(
                        repository, executor=FakeExecutor()
                    )
                self.assertFalse(
                    (
                        repository / "evidence/crouzeix_conjecture/local_formalization"
                    ).exists()
                )

    def test_rejects_hard_linked_staged_or_published_member(self) -> None:
        for when in ("staged", "published"):
            with self.subTest(when=when), tempfile.TemporaryDirectory() as directory:
                root = Path(directory).resolve()
                repository, terminal_receipt, _ = make_workspace(root)
                outside = root / f"{when}-command-link.json"
                if when == "staged":
                    original = local_formalization_evidence._fsync_tree

                    def inject(bundle: ls_receipts._PinnedDirectory) -> None:
                        os.link(bundle.path / "build/command.json", outside)
                        original(bundle)

                    patcher = mock.patch.object(
                        local_formalization_evidence,
                        "_fsync_tree",
                        side_effect=inject,
                    )
                    expected_error = protocol.ValidationError
                else:
                    original_rename = (
                        local_formalization_evidence._rename_directory_no_replace
                    )

                    def inject_after_rename(
                        *args: object,
                    ) -> ls_receipts._PinnedDirectory:
                        published = original_rename(*args)
                        os.link(published.path / "build/command.json", outside)
                        return published

                    patcher = mock.patch.object(
                        local_formalization_evidence,
                        "_rename_directory_no_replace",
                        side_effect=inject_after_rename,
                    )
                    expected_error = (
                        local_formalization_evidence.PublicationCommittedError
                    )
                with (
                    patcher,
                    mock.patch.object(
                        local_formalization_evidence.ls_validation,
                        "validate_committed_receipts",
                        return_value={"ls-terminal-crouzeix": terminal_receipt},
                    ),
                    self.assertRaises(expected_error),
                ):
                    local_formalization_evidence.publish_local_formalization_evidence(
                        repository, executor=FakeExecutor()
                    )

    def test_stage_writes_cannot_follow_swapped_nested_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            repository, terminal_receipt, _ = make_workspace(root)
            outside = root / "outside"
            outside.mkdir()
            original = local_formalization_evidence._write_bytes_create_only_at
            injected = False
            displaced = root / "displaced-build"

            def swap_then_write(
                directory_descriptor: int, name: str, data: bytes, label: str
            ) -> None:
                nonlocal injected
                if not injected and name == "command.json":
                    stage = stage_directories(repository)[0]
                    build = stage / "build"
                    build.rename(displaced)
                    build.symlink_to(outside, target_is_directory=True)
                    injected = True
                original(directory_descriptor, name, data, label)

            with (
                mock.patch.object(
                    local_formalization_evidence,
                    "_write_bytes_create_only_at",
                    side_effect=swap_then_write,
                ),
                mock.patch.object(
                    local_formalization_evidence.ls_validation,
                    "validate_committed_receipts",
                    return_value={"ls-terminal-crouzeix": terminal_receipt},
                ),
            ):
                try:
                    local_formalization_evidence.publish_local_formalization_evidence(
                        repository, executor=FakeExecutor()
                    )
                except protocol.ValidationError:
                    pass
            self.assertTrue(injected)
            self.assertEqual(list(outside.iterdir()), [])
            self.assertFalse(
                (
                    repository / "evidence/crouzeix_conjecture/local_formalization"
                ).exists()
            )

    def test_bounded_scanner_and_cleanup_stop_at_the_cap(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for index in range(3):
                (root / f"member-{index}").write_text("x", encoding="utf-8")
            descriptor = os.open(root, os.O_RDONLY | os.O_DIRECTORY)
            try:
                with (
                    mock.patch.object(
                        local_formalization_evidence, "MAX_BUNDLE_ENTRIES", 2
                    ),
                    self.assertRaisesRegex(protocol.ValidationError, "entry cap"),
                ):
                    local_formalization_evidence._remove_tree_contents_at(descriptor)
            finally:
                os.close(descriptor)
            self.assertEqual(len(list(root.iterdir())), 3)

    def test_production_traversals_do_not_use_unbounded_globs(self) -> None:
        source = inspect.getsource(local_formalization_evidence)
        self.assertNotIn(".rglob(", source)
        self.assertNotIn(".glob(", source)
        self.assertNotIn("sorted(os.scandir(", source)

    def test_closes_pinned_directories_with_shared_close_helper(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, _ = make_workspace(Path(directory).resolve())
            closed: list[ls_receipts._PinnedDirectory] = []
            original = ls_receipts._close_pinned_directory

            def record_close(directory: ls_receipts._PinnedDirectory) -> None:
                closed.append(directory)
                original(directory)

            with mock.patch.object(
                local_formalization_evidence.ls_receipts,
                "_close_pinned_directory",
                side_effect=record_close,
            ):
                self.publish(repository, terminal_receipt, executor=FakeExecutor())
            self.assertTrue(closed)
            self.assertTrue(
                any(directory.owned_ancestor_descriptors for directory in closed)
            )

    def test_precommit_failure_cleans_only_owned_stage_and_preserves_primary_error(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, _ = make_workspace(Path(directory).resolve())
            removed: list[tuple[Path, Path]] = []
            original_remove = local_formalization_evidence._remove_stage

            def record_remove(stage: Path, parent: Path) -> None:
                removed.append((stage, parent))
                original_remove(stage, parent)

            with (
                mock.patch.object(
                    local_formalization_evidence,
                    "_rename_directory_no_replace",
                    side_effect=protocol.ValidationError("injected precommit failure"),
                ),
                mock.patch.object(
                    local_formalization_evidence,
                    "_remove_stage",
                    side_effect=record_remove,
                ),
                mock.patch.object(
                    local_formalization_evidence.ls_validation,
                    "validate_committed_receipts",
                    return_value={"ls-terminal-crouzeix": terminal_receipt},
                ),
                self.assertRaisesRegex(
                    protocol.ValidationError, "injected precommit failure"
                ),
            ):
                local_formalization_evidence.publish_local_formalization_evidence(
                    repository, executor=FakeExecutor()
                )

            self.assertEqual(len(removed), 1)
            self.assertEqual(removed[0][1], repository / ".build")
            self.assertTrue(
                removed[0][0].name.startswith(".local-formalization-stage-")
            )
            self.assertEqual(stage_directories(repository), [])
            self.assertFalse(
                (
                    repository / "evidence/crouzeix_conjecture/local_formalization"
                ).exists()
            )

    def test_precommit_cleanup_failure_reports_primary_and_cleanup_errors(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, _ = make_workspace(Path(directory).resolve())
            original_remove = local_formalization_evidence._remove_stage

            def remove_then_fail(stage: Path, parent: Path) -> None:
                original_remove(stage, parent)
                raise protocol.ValidationError("injected stage cleanup failure")

            with (
                mock.patch.object(
                    local_formalization_evidence,
                    "_rename_directory_no_replace",
                    side_effect=protocol.ValidationError("injected precommit failure"),
                ),
                mock.patch.object(
                    local_formalization_evidence,
                    "_remove_stage",
                    side_effect=remove_then_fail,
                ),
                mock.patch.object(
                    local_formalization_evidence.ls_validation,
                    "validate_committed_receipts",
                    return_value={"ls-terminal-crouzeix": terminal_receipt},
                ),
                self.assertRaises(
                    local_formalization_evidence.PublicationCleanupError
                ) as raised,
            ):
                local_formalization_evidence.publish_local_formalization_evidence(
                    repository, executor=FakeExecutor()
                )

            self.assertIn(
                "injected precommit failure", str(raised.exception.primary_error)
            )
            self.assertTrue(
                any(
                    "injected stage cleanup failure" in str(error)
                    for error in raised.exception.cleanup_errors
                )
            )
            self.assertEqual(stage_directories(repository), [])

    def test_precommit_cleanup_cannot_delete_a_replacement_stage_path(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            repository, terminal_receipt, _ = make_workspace(root)
            build_root = repository / ".build"
            displaced = root / "displaced-build"
            replacement_stage: Path | None = None
            original_tree = local_formalization_evidence._fsync_tree

            def swap_after_fsync(bundle: ls_receipts._PinnedDirectory) -> None:
                nonlocal replacement_stage
                original_tree(bundle)
                build_root.rename(displaced)
                build_root.mkdir()
                replacement_stage = build_root / bundle.path.name
                replacement_stage.mkdir()
                (replacement_stage / "competitor.txt").write_text(
                    "preserve\n", encoding="utf-8"
                )

            with (
                mock.patch.object(
                    local_formalization_evidence,
                    "_fsync_tree",
                    side_effect=swap_after_fsync,
                ),
                mock.patch.object(
                    local_formalization_evidence.ls_validation,
                    "validate_committed_receipts",
                    return_value={"ls-terminal-crouzeix": terminal_receipt},
                ),
                self.assertRaisesRegex(
                    protocol.ValidationError, "staging parent identity changed"
                ),
            ):
                local_formalization_evidence.publish_local_formalization_evidence(
                    repository, executor=FakeExecutor()
                )

            assert replacement_stage is not None
            self.assertEqual(
                (replacement_stage / "competitor.txt").read_text(encoding="utf-8"),
                "preserve\n",
            )
            self.assertFalse((displaced / replacement_stage.name).exists())

    def test_postcommit_cleanup_failure_reports_committed_publication(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, _ = make_workspace(Path(directory).resolve())
            original_release = (
                local_formalization_evidence.ls_receipts._release_publication_lock
            )

            def release_then_fail(lock: ls_receipts._PublicationLock) -> None:
                original_release(lock)
                raise protocol.ValidationError("injected lock cleanup failure")

            with (
                mock.patch.object(
                    local_formalization_evidence.ls_receipts,
                    "_release_publication_lock",
                    side_effect=release_then_fail,
                ),
                mock.patch.object(
                    local_formalization_evidence.ls_validation,
                    "validate_committed_receipts",
                    return_value={"ls-terminal-crouzeix": terminal_receipt},
                ),
                self.assertRaises(
                    local_formalization_evidence.PublicationCommittedError
                ) as raised,
            ):
                local_formalization_evidence.publish_local_formalization_evidence(
                    repository, executor=FakeExecutor()
                )

            destination = (
                repository / "evidence/crouzeix_conjecture/local_formalization"
            )
            self.assertEqual(raised.exception.publication.artifact_root, destination)
            self.assertEqual(raised.exception.recovery_path, destination)
            self.assertTrue((destination / "manifest.tsv").is_file())
            self.assertIn("committed", str(raised.exception))
            self.assertEqual(stage_directories(repository), [])

    def test_postrename_validation_failure_is_committed_and_preserves_bundle(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, _ = make_workspace(Path(directory).resolve())
            destination = (
                repository / "evidence/crouzeix_conjecture/local_formalization"
            )
            original = ls_receipts._require_descriptor_identity
            injected = False

            def fail_after_rename(
                pinned: ls_receipts._PinnedDirectory, label: str
            ) -> None:
                nonlocal injected
                if (
                    not injected
                    and label == "local formalization staging parent"
                    and destination.is_dir()
                ):
                    injected = True
                    raise protocol.ValidationError(
                        "injected post-rename descriptor validation failure"
                    )
                original(pinned, label)

            with (
                mock.patch.object(
                    local_formalization_evidence.ls_receipts,
                    "_require_descriptor_identity",
                    side_effect=fail_after_rename,
                ),
                mock.patch.object(
                    local_formalization_evidence.ls_validation,
                    "validate_committed_receipts",
                    return_value={"ls-terminal-crouzeix": terminal_receipt},
                ),
                self.assertRaises(
                    local_formalization_evidence.PublicationCommittedError
                ) as raised,
            ):
                local_formalization_evidence.publish_local_formalization_evidence(
                    repository, executor=FakeExecutor()
                )

            self.assertTrue(injected)
            self.assertEqual(raised.exception.recovery_path, destination)
            expected_files = {
                "manifest.tsv",
                "build/command.json",
                "build/stdout.log",
                "build/stderr.log",
                "providers/harp.json",
                "providers/jin.json",
                "providers/lorist-schwenninger.json",
                *(
                    f"routes/{route_id}.{kind}.json"
                    for route_id in route_validation.ROUTE_IDS
                    for kind in ("manifest", "receipt", "review")
                ),
                *(f"axioms/{item[0]}.txt" for item in FORMALIZATIONS),
            }
            self.assertEqual(
                {
                    path.relative_to(destination).as_posix()
                    for path in destination.rglob("*")
                    if path.is_file()
                },
                expected_files,
            )

    def test_parent_fsync_failure_is_committed_and_still_attempts_both_parents(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository, terminal_receipt, _ = make_workspace(Path(directory).resolve())
            calls: list[Path] = []

            def fail_first(directory: ls_receipts._PinnedDirectory, _: str) -> None:
                calls.append(directory.path)
                if len(calls) == 1:
                    raise protocol.ValidationError(
                        "injected source-parent fsync failure"
                    )

            with (
                mock.patch.object(
                    local_formalization_evidence,
                    "_fsync_pinned_directory",
                    side_effect=fail_first,
                ),
                mock.patch.object(
                    local_formalization_evidence.ls_validation,
                    "validate_committed_receipts",
                    return_value={"ls-terminal-crouzeix": terminal_receipt},
                ),
                self.assertRaises(
                    local_formalization_evidence.PublicationCommittedError
                ),
            ):
                local_formalization_evidence.publish_local_formalization_evidence(
                    repository, executor=FakeExecutor()
                )

            self.assertEqual(
                calls,
                [
                    repository / ".build",
                    repository / "evidence/crouzeix_conjecture",
                ],
            )
            self.assertTrue(
                (
                    repository
                    / "evidence/crouzeix_conjecture/local_formalization/manifest.tsv"
                ).is_file()
            )


if __name__ == "__main__":
    unittest.main()
