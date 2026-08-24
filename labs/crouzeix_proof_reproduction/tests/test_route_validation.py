from __future__ import annotations

import copy
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from labs.crouzeix_proof_reproduction import route_validation


REPO = Path(__file__).resolve().parents[3]
SCRIPT = REPO / "labs/crouzeix_proof_reproduction/proof_evidence.py"
HEX_A = "a" * 64
HEX_B = "b" * 64
HARP_SUPPORT = tuple(sorted(route_validation.HARP_ALLOWED_LS_SUPPORT))


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def write_bytes(path: Path, value: bytes) -> str:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(value)
    return sha256_bytes(value)


def write_json(path: Path, value: object) -> str:
    data = route_validation.canonical_json_bytes(value)
    return write_bytes(path, data)


def write_manifest(path: Path, manifest: route_validation.RouteManifest) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(route_validation.canonical_json_bytes(route_validation.route_manifest_to_dict(manifest)))


class RouteFixture:
    def __init__(self, route_id: str = "jin") -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.repo = Path(self.temporary.name)
        self.lean_root = self.repo / "formalization/lean"
        self.route_id = route_id
        self.aggregate = {
            "jin": "CrouzeixJin",
            "lorist-schwenninger": "CrouzeixLoristSchwenninger",
            "harp": "CrouzeixHarp",
        }[route_id]
        self.claim_kind = "derived" if route_id == "harp" else "source-faithful"
        self.route_dir = (
            "jin-565b6a3" if route_id == "jin" else route_id
        )
        self.manifest_path = Path(
            f"labs/crouzeix_proof_reproduction/formal_targets/{self.route_dir}/route-manifest.json"
        )
        self.receipt_path = Path(f"evidence/crouzeix_conjecture/routes/{route_id}/receipt.json")
        self.review_path = Path(f"evidence/crouzeix_conjecture/reviews/{route_id}.json")
        self.type_paths = {
            "base": Path(f"evidence/crouzeix_conjecture/routes/{route_id}/types/base.txt"),
            "main": Path(f"evidence/crouzeix_conjecture/routes/{route_id}/types/main.txt"),
            "consequence": Path(
                f"evidence/crouzeix_conjecture/routes/{route_id}/types/consequence.txt"
            ),
        }
        self.declarations = {
            "base": f"CrouzeixConjecture.{route_id.replace('-', '')}Base",
            "main": f"CrouzeixConjecture.{route_id.replace('-', '')}Main",
            "consequence": f"CrouzeixConjecture.{route_id.replace('-', '')}Consequence",
        }
        self.modules = self._module_names()
        if route_id == "harp":
            self.type_paths.pop("base")
            for index, module in enumerate(HARP_SUPPORT):
                self.type_paths[f"reuse-{index}"] = Path(
                    f"evidence/crouzeix_conjecture/routes/harp/types/reuse-{index}.txt"
                )
                self.declarations[f"reuse-{index}"] = (
                    f"CrouzeixConjecture.LoristSchwenninger.support{index}"
                )
        self._write_sources()
        self._write_bound_artifacts()
        self.manifest = self._manifest_payload()
        if route_id == "harp":
            self._write_ls_manifest()
        write_json(self.repo / self.manifest_path, self.manifest)
        self._initialize_git()
        self._write_bundle()

    def cleanup(self) -> None:
        self.temporary.cleanup()

    def _module_names(self) -> dict[str, str]:
        stem = {
            "jin": "Jin",
            "lorist-schwenninger": "LoristSchwenninger",
            "harp": "Harp",
        }[self.route_id]
        modules = {
            "aggregate": self.aggregate,
            "foundation": "CrouzeixConjecture.Foundation",
            "base": f"Crouzeix.{stem}.Base",
            "main": f"Crouzeix.{stem}.Main",
            "consequence": f"Crouzeix.{stem}.Consequence",
        }
        if self.route_id == "harp":
            modules.pop("base")
            modules.update(
                {f"reuse-{index}": module for index, module in enumerate(HARP_SUPPORT)}
            )
        return modules

    def module_path(self, module: str) -> Path:
        return Path(*module.split(".")).with_suffix(".lean")

    def _write_module(self, module: str, imports: tuple[str, ...]) -> None:
        source = "".join(f"import {item}\n" for item in imports) + "\n"
        write_bytes(self.lean_root / self.module_path(module), source.encode())

    def _write_sources(self) -> None:
        write_bytes(self.lean_root / "lean-toolchain", b"leanprover/lean4:v4.32.1\n")
        write_json(self.lean_root / "lake-manifest.json", {"name": "fixture", "packages": []})
        self._write_module(
            self.modules["aggregate"],
            (self.modules["consequence"], self.modules["foundation"]),
        )
        self._write_module(self.modules["foundation"], ("Mathlib.Data.Matrix.Basic",))
        if self.route_id == "harp":
            for module in HARP_SUPPORT:
                self._write_module(module, (self.modules["foundation"],))
            self._write_module(self.modules["main"], HARP_SUPPORT)
        else:
            self._write_module(self.modules["base"], (self.modules["foundation"],))
            self._write_module(self.modules["main"], (self.modules["base"],))
        self._write_module(self.modules["consequence"], (self.modules["main"],))
        mathlib_source = self.lean_root / ".lake/packages/mathlib/Mathlib/Data/Matrix/Basic.lean"
        write_bytes(mathlib_source, b"\n")

    def _write_bound_artifacts(self) -> None:
        self.type_text = {
            "base": "Nat →   Prop\n",
            "main": "∀ (n : Nat),   n = n\n",
            "consequence": "∀ (n : Nat), n ≤ n\n",
        }
        if self.route_id == "harp":
            self.type_text.pop("base")
            self.type_text.update(
                {f"reuse-{index}": f"Nat → Prop -- {index}\n" for index in range(len(HARP_SUPPORT))}
            )
        for key, relative in self.type_paths.items():
            write_bytes(self.repo / relative, self.type_text[key].encode())
        self.source_path = Path(
            f"evidence/crouzeix_conjecture/sources/{self.route_id}/source.md"
        )
        write_bytes(self.repo / self.source_path, b"Pinned mathematical source.\n")

    def _node(self, key: str, dependency_ids: list[str]) -> dict[str, object]:
        provenance = "source" if self.claim_kind == "source-faithful" else "derived"
        source_bytes = (self.repo / self.source_path).read_bytes()
        node: dict[str, object] = {
            "node_id": key,
            "role": "terminal" if key == "main" else ("consequence" if key == "consequence" else "load-bearing"),
            "declaration": self.declarations[key],
            "module_path": self.module_path(self.modules[key]).as_posix(),
            "dependency_ids": dependency_ids,
            "provenance_kind": provenance,
            "correspondence_kind": (
                "direct-source" if provenance == "source" else "derived-extraction"
            ),
            "source_locator": (
                f"{self.source_path.as_posix()}#L1-L1"
                if provenance == "source"
                else None
            ),
            "source_archive_sha256": None,
            "source_file_sha256": (
                sha256_bytes(source_bytes) if provenance == "source" else None
            ),
            "source_excerpt_sha256": (
                sha256_bytes(source_bytes) if provenance == "source" else None
            ),
            "source_line_count": 1 if provenance == "source" else None,
            "reused_from_route": None,
            "reused_node_id": None,
            "declaration_type_path": self.type_paths[key].as_posix(),
            "statement_sha256": route_validation.normalized_type_sha256(
                self.type_text[key]
            ),
        }
        if self.route_id == "harp" and key.startswith("reuse-"):
            node["provenance_kind"] = "reused-route"
            node["correspondence_kind"] = "reused-route"
            node["reused_from_route"] = "lorist-schwenninger"
            node["reused_node_id"] = key
        return node

    def _manifest_payload(self) -> dict[str, object]:
        closure = sorted(self.modules.values())
        return {
            "schema_version": "crouzeix-route-proof-manifest/v1",
            "route_id": self.route_id,
            "claim_kind": self.claim_kind,
            "aggregate_module": self.aggregate,
            "build_target": self.aggregate,
            "terminal_declaration": self.declarations["main"],
            "terminal_type_sha256": route_validation.normalized_type_sha256(
                self.type_text["main"]
            ),
            "consequence_declarations": [self.declarations["consequence"]],
            "source_identities": (
                [f"sha256:{sha256_bytes((self.repo / self.source_path).read_bytes())}"]
                if self.claim_kind == "source-faithful"
                else []
            ),
            "shared_foundation_modules": [
                self.modules["aggregate"],
                self.modules["foundation"],
            ],
            "module_closure": closure,
            "module_closure_sha256": route_validation.string_roster_sha256(closure),
            "allowed_axioms": ["Classical.choice", "Quot.sound", "propext"],
            "review_path": self.review_path.as_posix(),
            "review_sha256": None,
            "receipt_path": self.receipt_path.as_posix(),
            "receipt_sha256": None,
            "nodes": (
                [self._node("base", []), self._node("main", ["base"]), self._node("consequence", ["main"])]
                if self.route_id != "harp"
                else [
                    *[self._node(f"reuse-{index}", []) for index in range(len(HARP_SUPPORT))],
                    self._node("main", [f"reuse-{index}" for index in range(len(HARP_SUPPORT))]),
                    self._node("consequence", ["main"]),
                ]
            ),
        }

    def _write_ls_manifest(self) -> None:
        ls_path = route_validation.ROUTE_MANIFEST_PATHS["lorist-schwenninger"]
        nodes = []
        for index, module in enumerate(HARP_SUPPORT):
            key = f"reuse-{index}"
            nodes.append(
                {
                    **self._node(key, []),
                    "provenance_kind": "source",
                    "correspondence_kind": "direct-source",
                    "source_locator": f"{self.source_path.as_posix()}#L1-L1",
                    "source_archive_sha256": None,
                    "source_file_sha256": sha256_bytes(
                        (self.repo / self.source_path).read_bytes()
                    ),
                    "source_excerpt_sha256": sha256_bytes(
                        (self.repo / self.source_path).read_bytes()
                    ),
                    "source_line_count": 1,
                    "reused_from_route": None,
                    "reused_node_id": None,
                    "module_path": self.module_path(module).as_posix(),
                    "role": "terminal" if index == len(HARP_SUPPORT) - 1 else "load-bearing",
                }
            )
        payload = {
            "schema_version": "crouzeix-route-proof-manifest/v1",
            "route_id": "lorist-schwenninger",
            "claim_kind": "source-faithful",
            "aggregate_module": "CrouzeixLoristSchwenninger",
            "build_target": "CrouzeixLoristSchwenninger",
            "terminal_declaration": nodes[-1]["declaration"],
            "terminal_type_sha256": nodes[-1]["statement_sha256"],
            "consequence_declarations": [nodes[-1]["declaration"]],
            "source_identities": self.manifest.get("source_identities") or [
                f"sha256:{sha256_bytes((self.repo / self.source_path).read_bytes())}"
            ],
            "shared_foundation_modules": [],
            "module_closure": list(HARP_SUPPORT),
            "module_closure_sha256": route_validation.string_roster_sha256(HARP_SUPPORT),
            "allowed_axioms": ["Classical.choice", "Quot.sound", "propext"],
            "review_path": "evidence/crouzeix_conjecture/reviews/lorist-schwenninger.json",
            "review_sha256": None,
            "receipt_path": "evidence/crouzeix_conjecture/routes/lorist-schwenninger/receipt.json",
            "receipt_sha256": None,
            "nodes": nodes,
        }
        write_json(self.repo / ls_path, payload)

    def _initialize_git(self) -> None:
        subprocess.run(["git", "init", "-q"], cwd=self.repo, check=True)
        subprocess.run(["git", "add", "."], cwd=self.repo, check=True)
        env = {
            **os.environ,
            "GIT_AUTHOR_NAME": "Fixture",
            "GIT_AUTHOR_EMAIL": "fixture@example.invalid",
            "GIT_COMMITTER_NAME": "Fixture",
            "GIT_COMMITTER_EMAIL": "fixture@example.invalid",
        }
        subprocess.run(["git", "commit", "-q", "-m", "fixture"], cwd=self.repo, env=env, check=True)
        self.commit = subprocess.run(
            ["git", "rev-parse", "HEAD"], cwd=self.repo, text=True, capture_output=True, check=True
        ).stdout.strip()
        self.tree = subprocess.run(
            ["git", "rev-parse", "HEAD^{tree}"], cwd=self.repo, text=True, capture_output=True, check=True
        ).stdout.strip()

    def _write_bundle(self) -> None:
        manifest_digest = route_validation.manifest_contract_sha256(self.manifest)
        command_path = Path(
            f"evidence/crouzeix_conjecture/routes/{self.route_id}/command.json"
        )
        provider_path = Path(
            f"evidence/crouzeix_conjecture/routes/{self.route_id}/provider.json"
        )
        axiom_path = Path(
            f"evidence/crouzeix_conjecture/routes/{self.route_id}/axioms.json"
        )
        stdout_path = Path(
            f"evidence/crouzeix_conjecture/routes/{self.route_id}/stdout.log"
        )
        stderr_path = Path(
            f"evidence/crouzeix_conjecture/routes/{self.route_id}/stderr.log"
        )
        mathlib_module = "Mathlib.Data.Matrix.Basic"
        mathlib_path = route_validation.mathlib_artifact_path(mathlib_module)
        argv = ["scripts/check_lean_library.sh", self.aggregate]
        cache_identity = route_validation.cache_contract_identity(self.repo)
        command = {
            "schema_version": "crouzeix-route-command/v1",
            "argv": argv,
            "working_directory": ".",
            "aggregate_module": self.aggregate,
            "build_target": self.aggregate,
            "cache_identity": cache_identity,
            "toolchain": "leanprover/lean4:v4.32.1",
        }
        command_sha = write_json(self.repo / command_path, command)
        provider = {
            "schema_version": "crouzeix-route-provider-report/v1",
            "route_id": self.route_id,
            "aggregate_module": self.aggregate,
            "modules": self.manifest["module_closure"],
            "module_closure_sha256": self.manifest["module_closure_sha256"],
            "status": "passed",
        }
        provider_sha = write_json(self.repo / provider_path, provider)
        axiom_results = [
            {"declaration": node["declaration"], "axioms": ["propext"]}
            for node in self.manifest["nodes"]
        ]
        axiom = {
            "schema_version": "crouzeix-route-axiom-audit/v1",
            "route_id": self.route_id,
            "allowed_axioms": self.manifest["allowed_axioms"],
            "results": axiom_results,
            "status": "passed",
        }
        axiom_sha = write_json(self.repo / axiom_path, axiom)
        stdout_sha = write_bytes(self.repo / stdout_path, b"Build completed.\n")
        stderr_sha = write_bytes(self.repo / stderr_path, b"")
        mathlib_sha = write_bytes(self.repo / mathlib_path, b"olean-fixture")
        mathlib_roster = [{"module": mathlib_module, "path": mathlib_path.as_posix(), "sha256": mathlib_sha}]
        declaration_types = []
        for node in self.manifest["nodes"]:
            type_path = Path(node["declaration_type_path"])
            declaration_types.append(
                {
                    "declaration": node["declaration"],
                    "type_artifact_path": type_path.as_posix(),
                    "type_artifact_sha256": sha256_bytes(
                        (self.repo / type_path).read_bytes()
                    ),
                    "statement_sha256": node["statement_sha256"],
                }
            )
        self.receipt = {
            "schema_version": "crouzeix-route-proof-receipt/v1",
            "route_id": self.route_id,
            "aggregate_module": self.aggregate,
            "build_target": self.aggregate,
            "manifest_path": self.manifest_path.as_posix(),
            "manifest_sha256": manifest_digest,
            "candidate_commit": self.commit,
            "candidate_tree": self.tree,
            "command_artifact_path": command_path.as_posix(),
            "command_artifact_sha256": command_sha,
            "argv": argv,
            "working_directory": ".",
            "cache_identity": cache_identity,
            "toolchain": "leanprover/lean4:v4.32.1",
            "local_closure_modules": self.manifest["module_closure"],
            "local_closure_sha256": self.manifest["module_closure_sha256"],
            "mathlib_artifacts": mathlib_roster,
            "mathlib_artifacts_sha256": route_validation.record_roster_sha256(
                mathlib_roster
            ),
            "declaration_types": declaration_types,
            "allowed_axioms": self.manifest["allowed_axioms"],
            "axiom_audit_path": axiom_path.as_posix(),
            "axiom_audit_sha256": axiom_sha,
            "axiom_results": axiom_results,
            "provider_report_path": provider_path.as_posix(),
            "provider_report_sha256": provider_sha,
            "stdout_path": stdout_path.as_posix(),
            "stdout_sha256": stdout_sha,
            "stderr_path": stderr_path.as_posix(),
            "stderr_sha256": stderr_sha,
            "exit_code": 0,
            "status": "passed",
            "receipt_sha256": "0" * 64,
        }
        self.receipt["receipt_sha256"] = route_validation.self_digest(
            self.receipt, "receipt_sha256"
        )
        receipt_sha = write_json(self.repo / self.receipt_path, self.receipt)
        self.review = {
            "schema_version": "crouzeix-proof-review/v1",
            "route_id": self.route_id,
            "reviewed_commit": self.commit,
            "reviewed_tree": self.tree,
            "manifest_path": self.manifest_path.as_posix(),
            "manifest_sha256": manifest_digest,
            "terminal_type_sha256": self.manifest["terminal_type_sha256"],
            "review_id": f"review-{self.route_id}-001",
            "reviewer_identity": "independent-reviewer",
            "reviewer_model": "review-model-v1",
            "reviewer_run_id": "run-001",
            "verdict": "complete",
            "outcome": "approved",
            "source_fidelity_check": (
                "passed" if self.claim_kind == "source-faithful" else "not-applicable"
            ),
            "derivation_reuse_check": (
                "passed" if self.claim_kind == "derived" else "not-applicable"
            ),
            "findings": [
                {
                    "finding_id": "F-001",
                    "severity": "Minor",
                    "resolved": True,
                    "locator": self.manifest_path.as_posix(),
                    "statement": "Fixture review note.",
                    "falsifying_test_or_gap": "No remaining gap.",
                }
            ],
            "review_sha256": "0" * 64,
        }
        self.review["review_sha256"] = route_validation.self_digest(
            self.review, "review_sha256"
        )
        review_sha = write_json(self.repo / self.review_path, self.review)
        self.manifest["receipt_sha256"] = receipt_sha
        self.manifest["review_sha256"] = review_sha
        write_json(self.repo / self.manifest_path, self.manifest)

    def rewrite_manifest(self) -> None:
        write_json(self.repo / self.manifest_path, self.manifest)

    def rewrite_receipt(self) -> None:
        self.receipt["receipt_sha256"] = route_validation.self_digest(
            self.receipt, "receipt_sha256"
        )
        digest = write_json(self.repo / self.receipt_path, self.receipt)
        self.manifest["receipt_sha256"] = digest
        self.rewrite_manifest()

    def rewrite_review(self) -> None:
        self.review["review_sha256"] = route_validation.self_digest(
            self.review, "review_sha256"
        )
        digest = write_json(self.repo / self.review_path, self.review)
        self.manifest["review_sha256"] = digest
        self.rewrite_manifest()

    def validate(self, *, allow_unpublished: bool = False) -> route_validation.RouteValidationResult:
        return route_validation.validate_route_bundle(
            self.repo,
            self.manifest_path,
            lean_root=Path("formalization/lean"),
            allow_unpublished=allow_unpublished,
        )


class RouteValidationTests(unittest.TestCase):
    maxDiff = None

    def fixture(self, route_id: str = "jin") -> RouteFixture:
        fixture = RouteFixture(route_id)
        self.addCleanup(fixture.cleanup)
        return fixture

    def assert_invalid(self, fixture: RouteFixture, pattern: str) -> None:
        with self.assertRaisesRegex(route_validation.RouteValidationError, pattern):
            fixture.validate()

    def test_concrete_producer_validator_round_trip(self) -> None:
        fixture = self.fixture()
        loaded = route_validation.load_route_manifest(
            fixture.repo, fixture.manifest_path
        )
        output = fixture.repo / fixture.manifest_path.with_name("round-trip.json")

        write_manifest(output, loaded)
        result = fixture.validate(allow_unpublished=True)

        self.assertEqual(output.read_bytes(), (fixture.repo / fixture.manifest_path).read_bytes())
        self.assertEqual(result.route_id, "jin")
        self.assertEqual(result.claim_level, "complete-local")
        self.assertIsInstance(loaded.nodes[0], route_validation.RouteNode)
        self.assertIsInstance(loaded, route_validation.RouteManifest)

    def test_duplicate_json_keys_and_unknown_fields_fail_closed(self) -> None:
        fixture = self.fixture()
        path = fixture.repo / fixture.manifest_path
        text = path.read_text()
        path.write_text(text.replace("{", '{"route_id":"jin",', 1))
        self.assert_invalid(fixture, "duplicate JSON key.*route_id")

        fixture = self.fixture()
        fixture.manifest["surprise"] = True
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "unknown field.*surprise")

    def test_duplicate_nodes_modules_unknown_dependencies_self_edges_and_cycles(self) -> None:
        mutations = [
            (lambda f: f.manifest["nodes"].append(copy.deepcopy(f.manifest["nodes"][0])), "duplicate node_id"),
            (lambda f: f.manifest["nodes"][1].__setitem__("module_path", f.manifest["nodes"][0]["module_path"]), "duplicate node module"),
            (lambda f: f.manifest["nodes"][1].__setitem__("dependency_ids", ["absent"]), "unknown dependency"),
            (lambda f: f.manifest["nodes"][0].__setitem__("dependency_ids", ["base"]), "self dependency"),
            (lambda f: f.manifest["nodes"][0].__setitem__("dependency_ids", ["main"]), "cycle|stable topological order"),
        ]
        for mutate, pattern in mutations:
            with self.subTest(pattern=pattern):
                fixture = self.fixture()
                mutate(fixture)
                fixture.rewrite_manifest()
                self.assert_invalid(fixture, pattern)

    def test_terminal_and_consequence_invariants(self) -> None:
        mutations = [
            (lambda f: f.manifest["nodes"][1].__setitem__("role", "load-bearing"), "exactly one terminal node"),
            (lambda f: f.manifest["nodes"][0].__setitem__("role", "terminal"), "exactly one terminal node"),
            (lambda f: f.manifest.__setitem__("terminal_declaration", "Wrong.main"), "terminal declaration mismatch"),
            (lambda f: f.manifest.__setitem__("consequence_declarations", ["Wrong.consequence"]), "consequence declaration is not represented"),
        ]
        for mutate, pattern in mutations:
            with self.subTest(pattern=pattern):
                fixture = self.fixture()
                mutate(fixture)
                fixture.rewrite_manifest()
                self.assert_invalid(fixture, pattern)

    def test_active_closure_requires_exact_nonoverlapping_coverage(self) -> None:
        fixture = self.fixture()
        fixture.manifest["nodes"] = fixture.manifest["nodes"][1:]
        fixture.manifest["nodes"][0]["dependency_ids"] = []
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "unmapped active closure module")

        fixture = self.fixture()
        fixture.manifest["shared_foundation_modules"].append(fixture.modules["base"])
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "covered by both a node and shared foundation")

    def test_paths_reject_escape_symlinks_and_hardlink_aliases(self) -> None:
        fixture = self.fixture()
        fixture.manifest["nodes"][0]["source_locator"] = "../outside.md#L1-L1"
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "safe repository-relative path")

        fixture = self.fixture()
        original = fixture.repo / fixture.type_paths["base"]
        alias = original.with_name("alias.txt")
        alias.symlink_to(original.name)
        fixture.manifest["nodes"][0]["declaration_type_path"] = alias.relative_to(fixture.repo).as_posix()
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "symlink")

        fixture = self.fixture()
        original = fixture.repo / fixture.type_paths["base"]
        alias = original.with_name("hardlink.txt")
        os.link(original, alias)
        fixture.manifest["nodes"][0]["declaration_type_path"] = alias.relative_to(fixture.repo).as_posix()
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "hardlink")

    def test_source_routes_require_pinned_source_nodes_and_identities(self) -> None:
        fixture = self.fixture()
        fixture.manifest["nodes"][0]["source_locator"] = None
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "source node requires a pinned source locator")

        fixture = self.fixture()
        fixture.manifest["source_identities"] = []
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "source-faithful route requires source identities")

        fixture = self.fixture()
        fixture.manifest["source_identities"] = [f"sha256:{HEX_B}"]
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "source locator digest is not declared")

    def test_source_faithful_route_allows_explicit_derived_nodes_only_without_source_data(self) -> None:
        fixture = self.fixture()
        (fixture.repo / fixture.receipt_path).unlink()
        (fixture.repo / fixture.review_path).unlink()
        fixture.manifest["receipt_sha256"] = None
        fixture.manifest["review_sha256"] = None
        node = fixture.manifest["nodes"][0]
        node["provenance_kind"] = "derived"
        node["correspondence_kind"] = "derived-extraction"
        node["source_locator"] = None
        node["source_archive_sha256"] = None
        node["source_file_sha256"] = None
        node["source_excerpt_sha256"] = None
        node["source_line_count"] = None
        fixture.rewrite_manifest()

        self.assertEqual(fixture.validate(allow_unpublished=True).claim_level, "mapped")

        node["source_file_sha256"] = HEX_A
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "derived node cannot claim source or reuse provenance")

        node["source_file_sha256"] = None
        node["provenance_kind"] = "reused-route"
        node["correspondence_kind"] = "reused-route"
        node["reused_from_route"] = "harp"
        node["reused_node_id"] = "foreign"
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "source-faithful route cannot reuse another route")

    def test_local_source_locator_recomputes_file_excerpt_and_line_metadata(self) -> None:
        cases = (
            ("source_file_sha256", HEX_B, "source file digest"),
            ("source_excerpt_sha256", HEX_B, "source excerpt digest"),
            ("source_line_count", 2, "source line count"),
            ("source_archive_sha256", HEX_A, "local source locator cannot declare an archive"),
        )
        for field, value, pattern in cases:
            with self.subTest(field=field):
                fixture = self.fixture()
                fixture.manifest["nodes"][0][field] = value
                fixture.rewrite_manifest()
                self.assert_invalid(fixture, pattern)

    def test_reuse_is_declared_cross_route_and_required_for_harp(self) -> None:
        fixture = self.fixture("harp")
        fixture.manifest["nodes"][0]["reused_node_id"] = None
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "reused-route node requires exact route and node identities")

        fixture = self.fixture("harp")
        fixture.manifest["nodes"][0]["reused_from_route"] = "jin"
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "Harp may reuse only lorist-schwenninger")

        fixture = self.fixture("harp")
        fixture.manifest["nodes"][0]["provenance_kind"] = "derived"
        fixture.manifest["nodes"][0]["correspondence_kind"] = "derived-extraction"
        fixture.manifest["nodes"][0]["reused_from_route"] = None
        fixture.manifest["nodes"][0]["reused_node_id"] = None
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "Harp route must explicitly record LS reuse|LS support reuse coverage mismatch")

    def test_correspondence_kind_matrix_is_exact_and_checked_during_parse(self) -> None:
        allowed = {
            "source": ("direct-source", "compatibility-port", "structural-refactor"),
            "derived": ("derived-extraction",),
            "shared-foundation": ("shared-foundation",),
            "reused-route": ("reused-route",),
        }
        all_kinds = tuple(
            kind for kinds in allowed.values() for kind in kinds
        )
        for provenance, kinds in allowed.items():
            for correspondence in kinds:
                with self.subTest(valid=(provenance, correspondence)):
                    fixture = self.fixture()
                    node = fixture.manifest["nodes"][0]
                    node["provenance_kind"] = provenance
                    node["correspondence_kind"] = correspondence
                    if provenance != "source":
                        for field in (
                            "source_locator", "source_archive_sha256",
                            "source_file_sha256", "source_excerpt_sha256",
                            "source_line_count",
                        ):
                            node[field] = None
                    if provenance == "reused-route":
                        node["reused_from_route"] = "lorist-schwenninger"
                        node["reused_node_id"] = "support"
                    route_validation.parse_route_manifest(fixture.manifest)
            invalid = next(kind for kind in all_kinds if kind not in kinds)
            with self.subTest(invalid=(provenance, invalid)):
                fixture = self.fixture()
                node = fixture.manifest["nodes"][0]
                node["provenance_kind"] = provenance
                node["correspondence_kind"] = invalid
                with self.assertRaisesRegex(
                    route_validation.RouteValidationError, "incompatible"
                ):
                    route_validation.parse_route_manifest(fixture.manifest)

    def test_harp_resolves_all_eleven_support_modules_against_ls_manifest(self) -> None:
        fixture = self.fixture("harp")
        reused = [
            node for node in fixture.manifest["nodes"]
            if node["provenance_kind"] == "reused-route"
        ]
        self.assertEqual({fixture.modules[node["node_id"]] for node in reused}, set(HARP_SUPPORT))
        self.assertEqual(len(reused), 11)
        self.assertEqual(fixture.validate().claim_level, "complete-local")

        for index in range(len(HARP_SUPPORT)):
            with self.subTest(omitted=HARP_SUPPORT[index]):
                candidate = self.fixture("harp")
                candidate.manifest["nodes"].pop(index)
                candidate.manifest["nodes"][len(HARP_SUPPORT) - 1]["dependency_ids"].remove(
                    f"reuse-{index}"
                )
                candidate.rewrite_manifest()
                self.assert_invalid(candidate, "LS support reuse coverage mismatch|unmapped active closure module")

    def test_harp_reuse_rejects_missing_dangling_and_mismatched_ls_manifest(self) -> None:
        ls_path = route_validation.ROUTE_MANIFEST_PATHS["lorist-schwenninger"]
        cases = ("missing", "dangling", "module", "declaration", "digest")
        for case in cases:
            with self.subTest(case=case):
                fixture = self.fixture("harp")
                if case == "missing":
                    (fixture.repo / ls_path).unlink()
                elif case == "dangling":
                    fixture.manifest["nodes"][0]["reused_node_id"] = "absent"
                    fixture.rewrite_manifest()
                else:
                    payload = json.loads((fixture.repo / ls_path).read_text())
                    field = {
                        "module": "module_path",
                        "declaration": "declaration",
                        "digest": "statement_sha256",
                    }[case]
                    payload["nodes"][0][field] = (
                        "Crouzeix/LoristSchwenninger/Wrong.lean"
                        if case == "module"
                        else ("Wrong.declaration" if case == "declaration" else HEX_B)
                    )
                    write_json(fixture.repo / ls_path, payload)
                self.assert_invalid(fixture, "LS reuse|LS route manifest")

    def test_provider_policy_rejects_direct_transitive_and_allowlist_drift(self) -> None:
        cases = [
            ("jin", "Crouzeix.LoristSchwenninger.MainTheorem"),
            ("lorist-schwenninger", "Crouzeix.Jin.MainTheorem"),
            ("harp", "Crouzeix.LoristSchwenninger.MainTheorem"),
        ]
        for route_id, forbidden in cases:
            with self.subTest(route_id=route_id):
                fixture = self.fixture(route_id)
                fixture._write_module(forbidden, ("Mathlib.Data.Matrix.Basic",))
                target = (
                    fixture.modules["base"]
                    if "base" in fixture.modules
                    else fixture.modules["reuse-0"]
                )
                fixture._write_module(target, (forbidden,))
                self.assert_invalid(fixture, "forbidden module|provider policy")

        fixture = self.fixture("jin")
        bridge = "CrouzeixConjecture.Bridge"
        forbidden = "Crouzeix.Harp.Main"
        fixture._write_module(forbidden, ("Mathlib.Data.Matrix.Basic",))
        fixture._write_module(bridge, (forbidden,))
        fixture._write_module(fixture.modules["base"], (bridge,))
        self.assert_invalid(fixture, "forbidden module|provider policy")

    def test_provider_policy_rejects_top_level_cross_route_aggregates(self) -> None:
        cases = {
            "jin": ("CrouzeixLoristSchwenninger", "CrouzeixHarp", "Crouzeix"),
            "lorist-schwenninger": ("CrouzeixJin", "CrouzeixHarp", "Crouzeix"),
            "harp": ("CrouzeixJin", "CrouzeixLoristSchwenninger", "Crouzeix"),
        }
        for route_id, forbidden_modules in cases.items():
            for forbidden in forbidden_modules:
                with self.subTest(route_id=route_id, forbidden=forbidden):
                    fixture = self.fixture(route_id)
                    fixture._write_module(forbidden, ())
                    fixture._write_module(fixture.aggregate, (forbidden,))
                    self.assert_invalid(fixture, "forbidden module|provider policy")

    def test_linked_worktree_cache_accepts_only_approved_primary_cache(self) -> None:
        fixture = self.fixture()
        primary = fixture.repo
        linked = Path(tempfile.mkdtemp()) / "linked"
        self.addCleanup(lambda: subprocess.run(["git", "worktree", "remove", "--force", str(linked)], cwd=primary, check=False, capture_output=True))
        subprocess.run(["git", "worktree", "add", "-q", "--detach", str(linked), "HEAD"], cwd=primary, check=True)
        approved = primary / "formalization/lean/.lake"
        linked_cache = linked / "formalization/lean/.lake"
        linked_cache.parent.mkdir(parents=True, exist_ok=True)
        if linked_cache.is_symlink():
            linked_cache.unlink()
        elif linked_cache.exists():
            shutil.rmtree(linked_cache)
        linked_cache.symlink_to(approved, target_is_directory=True)
        self.assertEqual(route_validation.resolve_approved_cache_root(linked), approved.resolve())

        linked_cache.unlink()
        hostile = primary / "hostile-cache"
        hostile.mkdir()
        linked_cache.symlink_to(hostile, target_is_directory=True)
        with self.assertRaisesRegex(route_validation.RouteValidationError, "approved primary cache"):
            route_validation.resolve_approved_cache_root(linked)

        linked_cache.unlink()
        linked_cache.symlink_to(approved, target_is_directory=True)
        mathlib = approved / "packages/mathlib"
        moved = approved / "packages/mathlib-real"
        mathlib.rename(moved)
        mathlib.symlink_to(moved, target_is_directory=True)
        with self.assertRaisesRegex(route_validation.RouteValidationError, "nested symlink"):
            route_validation.active_mathlib_closure(linked, fixture.manifest["module_closure"])

    def test_declaration_type_digest_drift_is_rejected(self) -> None:
        fixture = self.fixture()
        write_bytes(fixture.repo / fixture.type_paths["main"], b"False\n")
        self.assert_invalid(fixture, "declaration type digest mismatch")

    def test_axioms_are_sorted_unique_and_allowlisted(self) -> None:
        for axioms, pattern in [
            (["propext", "Classical.choice"], "sorted"),
            (["propext", "propext"], "duplicate"),
            (["False.elim"], "forbidden axiom"),
        ]:
            with self.subTest(axioms=axioms):
                fixture = self.fixture()
                fixture.receipt["axiom_results"][0]["axioms"] = axioms
                axiom_path = Path(fixture.receipt["axiom_audit_path"])
                axiom = json.loads((fixture.repo / axiom_path).read_text())
                axiom["results"][0]["axioms"] = axioms
                fixture.receipt["axiom_audit_sha256"] = write_json(
                    fixture.repo / axiom_path, axiom
                )
                fixture.rewrite_receipt()
                self.assert_invalid(fixture, pattern)

    def test_receipt_cross_artifact_mismatches_fail_closed(self) -> None:
        cases = [
            (lambda f: f.receipt.__setitem__("aggregate_module", "Wrong"), "receipt aggregate mismatch"),
            (lambda f: f.receipt.__setitem__("manifest_sha256", HEX_B), "receipt manifest digest mismatch"),
            (lambda f: f.receipt.__setitem__("argv", ["true"]), "receipt command mismatch"),
            (lambda f: f.receipt.__setitem__("local_closure_modules", [f.aggregate]), "receipt local closure mismatch"),
            (lambda f: f.receipt.__setitem__("mathlib_artifacts_sha256", HEX_B), "Mathlib roster digest mismatch"),
            (lambda f: f.receipt.__setitem__("provider_report_sha256", HEX_B), "provider report digest mismatch"),
            (lambda f: f.receipt.__setitem__("stdout_sha256", HEX_B), "stdout digest mismatch"),
            (lambda f: f.receipt.__setitem__("status", "failed"), "receipt status/exit mismatch"),
        ]
        for mutate, pattern in cases:
            with self.subTest(pattern=pattern):
                fixture = self.fixture()
                mutate(fixture)
                fixture.rewrite_receipt()
                self.assert_invalid(fixture, pattern)

        fixture = self.fixture()
        fixture.receipt["mathlib_artifacts"][0]["sha256"] = HEX_B
        fixture.receipt["mathlib_artifacts_sha256"] = route_validation.record_roster_sha256(
            fixture.receipt["mathlib_artifacts"]
        )
        fixture.rewrite_receipt()
        self.assert_invalid(fixture, "Mathlib artifact digest mismatch")

    def test_receipt_requires_exact_declaration_and_mathlib_rosters(self) -> None:
        fixture = self.fixture()
        fixture.receipt["declaration_types"][1] = copy.deepcopy(
            fixture.receipt["declaration_types"][0]
        )
        fixture.rewrite_receipt()
        self.assert_invalid(fixture, "declaration type roster")

        mutations = (
            lambda items: items.clear(),
            lambda items: items.append({"module": "Mathlib.Extra", "path": "formalization/lean/.lake/packages/mathlib/.lake/build/lib/lean/Mathlib/Extra.olean", "sha256": HEX_A}),
            lambda items: items[0].__setitem__("module", "Mathlib.Wrong"),
            lambda items: items[0].__setitem__("path", "evidence/wrong.olean"),
        )
        for mutate in mutations:
            with self.subTest(mutate=mutate):
                fixture = self.fixture()
                mutate(fixture.receipt["mathlib_artifacts"])
                fixture.receipt["mathlib_artifacts_sha256"] = route_validation.record_roster_sha256(
                    fixture.receipt["mathlib_artifacts"]
                )
                fixture.rewrite_receipt()
                self.assert_invalid(fixture, "Mathlib artifact roster")

    def test_review_binding_severity_and_complete_verdict_invariants(self) -> None:
        cases = [
            (lambda f: f.review.__setitem__("reviewed_commit", "3" * 40), "review commit mismatch"),
            (lambda f: f.review.__setitem__("manifest_sha256", HEX_B), "review manifest digest mismatch"),
            (lambda f: f.review.__setitem__("terminal_type_sha256", HEX_B), "review terminal type mismatch"),
            (lambda f: f.review["findings"][0].__setitem__("severity", "major"), "invalid finding severity"),
        ]
        for mutate, pattern in cases:
            with self.subTest(pattern=pattern):
                fixture = self.fixture()
                mutate(fixture)
                fixture.rewrite_review()
                self.assert_invalid(fixture, pattern)

        for severity in ("Critical", "Important"):
            with self.subTest(severity=severity):
                fixture = self.fixture()
                fixture.review["findings"][0]["severity"] = severity
                fixture.review["findings"][0]["resolved"] = False
                fixture.rewrite_review()
                self.assert_invalid(fixture, "complete review has unresolved Critical/Important finding")

    def test_roles_consequence_mapping_and_exact_active_closure(self) -> None:
        fixture = self.fixture()
        fixture.manifest["nodes"][0]["role"] = "helper"
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "invalid node role")

        fixture = self.fixture()
        fixture.manifest["nodes"][2]["role"] = "load-bearing"
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "consequence role mismatch")

        fixture = self.fixture()
        fixture.manifest["nodes"][0]["role"] = "consequence"
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "undeclared consequence-role node")

        fixture = self.fixture()
        extra = copy.deepcopy(fixture.manifest["nodes"][0])
        extra.update({"node_id": "inactive", "declaration": "CrouzeixConjecture.inactive", "module_path": "Crouzeix/Jin/Inactive.lean", "dependency_ids": []})
        fixture.manifest["nodes"].insert(1, extra)
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "inactive node/shared module|closure coverage")

        fixture = self.fixture()
        fixture.manifest["shared_foundation_modules"].append("CrouzeixConjecture.Inactive")
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "inactive node/shared module|closure coverage")

    def test_source_locator_requires_ordered_in_range_lines_and_digest(self) -> None:
        for locator, pattern in (
            ("#L2-L1", "reversed source locator span"),
            ("#L1-L2", "source locator span exceeds file"),
        ):
            with self.subTest(locator=locator):
                fixture = self.fixture()
                fixture.manifest["nodes"][0]["source_locator"] = (
                    fixture.source_path.as_posix() + locator
                )
                fixture.rewrite_manifest()
                self.assert_invalid(fixture, pattern)

    def test_unpublished_present_artifacts_are_validated_without_claim_promotion(self) -> None:
        fixture = self.fixture()
        (fixture.repo / fixture.review_path).unlink()
        fixture.manifest["review_sha256"] = None
        fixture.rewrite_manifest()

        result = fixture.validate(allow_unpublished=True)

        self.assertEqual(result.claim_level, "receipt-backed")
        self.assertEqual(result.status, "incomplete")

        fixture.review["findings"][0]["severity"] = "unknown"
        fixture.rewrite_review()
        self.assert_invalid(fixture, "invalid finding severity")

    def test_publication_state_machine_rejects_incoherent_presence_and_digests(self) -> None:
        fixture = self.fixture()
        receipt = fixture.repo / fixture.receipt_path
        review = fixture.repo / fixture.review_path
        receipt.unlink()
        review.unlink()
        fixture.manifest["receipt_sha256"] = None
        fixture.manifest["review_sha256"] = None
        fixture.rewrite_manifest()
        self.assertEqual(fixture.validate(allow_unpublished=True).claim_level, "mapped")

        for state in ("receipt-present-null", "receipt-digest-absent", "review-without-receipt", "review-present-null", "review-digest-absent"):
            with self.subTest(state=state):
                candidate = self.fixture()
                receipt = candidate.repo / candidate.receipt_path
                review = candidate.repo / candidate.review_path
                if state == "receipt-present-null":
                    candidate.manifest["receipt_sha256"] = None
                    review.unlink()
                    candidate.manifest["review_sha256"] = None
                elif state == "receipt-digest-absent":
                    receipt.unlink(); review.unlink(); candidate.manifest["review_sha256"] = None
                elif state == "review-without-receipt":
                    receipt.unlink(); candidate.manifest["receipt_sha256"] = None
                elif state == "review-present-null":
                    candidate.manifest["review_sha256"] = None
                else:
                    review.unlink()
                candidate.rewrite_manifest()
                self.assert_invalid(candidate, "publication|digest|review.*receipt|missing")

    def test_manifest_only_rejects_source_type_and_harp_identity_drift(self) -> None:
        fixture = self.fixture()
        (fixture.repo / fixture.receipt_path).unlink(); (fixture.repo / fixture.review_path).unlink()
        fixture.manifest["receipt_sha256"] = None; fixture.manifest["review_sha256"] = None
        fixture.manifest["nodes"][0]["source_locator"] = f"{fixture.source_path.as_posix()}#L2-L1"
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "reversed source locator")

        fixture = self.fixture()
        (fixture.repo / fixture.receipt_path).unlink(); (fixture.repo / fixture.review_path).unlink()
        fixture.manifest["receipt_sha256"] = None; fixture.manifest["review_sha256"] = None
        write_bytes(fixture.repo / fixture.type_paths["main"], b"False\n")
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "declaration type digest mismatch")

        fixture = self.fixture("harp")
        (fixture.repo / fixture.receipt_path).unlink(); (fixture.repo / fixture.review_path).unlink()
        fixture.manifest["receipt_sha256"] = None; fixture.manifest["review_sha256"] = None
        fixture.manifest["source_identities"] = [f"sha256:{HEX_A}"]
        fixture.rewrite_manifest()
        self.assert_invalid(fixture, "Harp.*source identities")

    def test_invocation_contract_rejects_schema_cwd_toolchain_and_cache_identity(self) -> None:
        cases = (
            ("schema_version", "wrong/v1", "command artifact schema"),
            ("working_directory", "formalization/lean", "working directory"),
            ("toolchain", "leanprover/lean4:v0.0.0", "toolchain"),
            ("cache_identity", f"sha256:{HEX_B}", "cache identity"),
        )
        for field, value, pattern in cases:
            with self.subTest(field=field):
                fixture = self.fixture()
                command_path = fixture.repo / fixture.receipt["command_artifact_path"]
                command = json.loads(command_path.read_text())
                command[field] = value
                if field != "schema_version":
                    fixture.receipt[field] = value
                fixture.receipt["command_artifact_sha256"] = write_json(command_path, command)
                fixture.rewrite_receipt()
                self.assert_invalid(fixture, pattern)

    def test_command_and_axiom_artifacts_reject_unknown_rehashed_fields(self) -> None:
        fixture = self.fixture()
        command_path = fixture.repo / fixture.receipt["command_artifact_path"]
        command = json.loads(command_path.read_text()); command["surprise"] = True
        fixture.receipt["command_artifact_sha256"] = write_json(command_path, command)
        fixture.rewrite_receipt()
        self.assert_invalid(fixture, "command artifact has unknown field")

        fixture = self.fixture()
        audit_path = fixture.repo / fixture.receipt["axiom_audit_path"]
        audit = json.loads(audit_path.read_text()); audit["surprise"] = True
        fixture.receipt["axiom_audit_sha256"] = write_json(audit_path, audit)
        fixture.rewrite_receipt()
        self.assert_invalid(fixture, "axiom audit has unknown field")

    def test_production_module_has_no_manifest_writer(self) -> None:
        self.assertFalse(hasattr(route_validation, "write_route_manifest"))

    def test_allow_unpublished_controls_absent_and_partial_publication(self) -> None:
        empty = tempfile.TemporaryDirectory()
        self.addCleanup(empty.cleanup)
        root = Path(empty.name)
        (root / "formalization/lean").mkdir(parents=True)
        write_bytes(root / "formalization/lean/CrouzeixJin.lean", b"\n")
        with self.assertRaisesRegex(route_validation.RouteValidationError, "unpublished"):
            route_validation.inspect_route(root, "jin")
        result = route_validation.inspect_route(root, "jin", allow_unpublished=True)
        self.assertEqual(result.claim_level, "authored")

        fixture = self.fixture()
        (fixture.repo / fixture.receipt_path).unlink()
        (fixture.repo / fixture.review_path).unlink()
        fixture.manifest["receipt_sha256"] = None
        fixture.manifest["review_sha256"] = None
        fixture.rewrite_manifest()
        with self.assertRaisesRegex(route_validation.RouteValidationError, "partial publication"):
            route_validation.inspect_route(fixture.repo, "jin")
        self.assertEqual(
            route_validation.inspect_route(fixture.repo, "jin", allow_unpublished=True).claim_level,
            "mapped",
        )

        fixture = self.fixture()
        (fixture.repo / fixture.review_path).unlink()
        fixture.manifest["review_sha256"] = None
        fixture.rewrite_manifest()
        with self.assertRaisesRegex(route_validation.RouteValidationError, "partial publication"):
            route_validation.inspect_route(fixture.repo, "jin")
        self.assertEqual(
            route_validation.inspect_route(fixture.repo, "jin", allow_unpublished=True).claim_level,
            "receipt-backed",
        )

        fixture = self.fixture()
        self.assertEqual(route_validation.inspect_route(fixture.repo, "jin").claim_level, "complete-local")

    def test_fake_or_mismatched_git_identity_is_rejected(self) -> None:
        fixture = self.fixture()
        fixture.receipt["candidate_commit"] = "1" * 40
        fixture.review["reviewed_commit"] = "1" * 40
        fixture.rewrite_receipt()
        fixture.rewrite_review()
        self.assert_invalid(fixture, "candidate commit.*Git")

        fixture = self.fixture()
        fixture.receipt["candidate_tree"] = "2" * 40
        fixture.review["reviewed_tree"] = "2" * 40
        fixture.rewrite_receipt()
        fixture.rewrite_review()
        self.assert_invalid(fixture, "candidate tree mismatch")

    def test_dangling_commit_object_is_not_accepted_as_reachable_history(self) -> None:
        fixture = self.fixture()
        dangling = subprocess.run(
            ["git", "commit-tree", fixture.tree],
            cwd=fixture.repo,
            input="dangling fixture\n",
            text=True,
            capture_output=True,
            check=True,
            env={
                **os.environ,
                "GIT_AUTHOR_NAME": "Fixture",
                "GIT_AUTHOR_EMAIL": "fixture@example.invalid",
                "GIT_COMMITTER_NAME": "Fixture",
                "GIT_COMMITTER_EMAIL": "fixture@example.invalid",
            },
        ).stdout.strip()
        fixture.receipt["candidate_commit"] = dangling
        fixture.review["reviewed_commit"] = dangling
        fixture.rewrite_receipt()
        fixture.rewrite_review()
        self.assert_invalid(fixture, "candidate commit is not an ancestor of HEAD")

    def test_ancestor_commit_rejects_current_route_source_blob_drift(self) -> None:
        fixture = self.fixture()
        route_source = fixture.lean_root / fixture.module_path(fixture.aggregate)
        route_source.write_text(route_source.read_text() + "-- drift after candidate\n")
        subprocess.run(["git", "add", route_source.relative_to(fixture.repo)], cwd=fixture.repo, check=True)
        subprocess.run(
            ["git", "-c", "user.name=Fixture", "-c", "user.email=fixture@example.invalid", "commit", "-q", "-m", "source drift"],
            cwd=fixture.repo,
            check=True,
        )
        self.assert_invalid(fixture, "candidate route source blob mismatch")

    def test_schema_files_are_draft_2020_12_and_closed_recursively(self) -> None:
        schema_root = REPO / "labs/crouzeix_proof_reproduction/schemas"
        for name in (
            "route_manifest.schema.json",
            "route_receipt.schema.json",
            "proof_review.schema.json",
        ):
            with self.subTest(name=name):
                payload = json.loads((schema_root / name).read_text())
                self.assertEqual(payload["$schema"], "https://json-schema.org/draft/2020-12/schema")
                self._assert_closed_objects(payload)

    def _assert_closed_objects(self, value: object) -> None:
        if isinstance(value, dict):
            if value.get("type") == "object":
                self.assertIs(value.get("additionalProperties"), False)
            for child in value.values():
                self._assert_closed_objects(child)
        elif isinstance(value, list):
            for child in value:
                self._assert_closed_objects(child)

    def test_cli_absent_artifacts_and_allow_unpublished_are_non_successful_and_read_only(self) -> None:
        def source_snapshot() -> dict[str, str]:
            snapshot: dict[str, str] = {}
            for root_name in ("labs", "formalization", "evidence"):
                for path in (REPO / root_name).rglob("*"):
                    if "__pycache__" in path.parts:
                        continue
                    if path.is_file() or path.is_symlink():
                        relative = path.relative_to(REPO).as_posix()
                        snapshot[relative] = (
                            "link" if path.is_symlink() else sha256_bytes(path.read_bytes())
                        )
            return snapshot

        before = source_snapshot()
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        fake_bin = Path(temporary.name)
        markers = []
        for executable in ("lean", "lake"):
            marker = fake_bin / f"{executable}.called"
            markers.append(marker)
            script = fake_bin / executable
            script.write_text(f"#!/bin/sh\n: > '{marker}'\nexit 99\n")
            script.chmod(0o755)
        env = dict(os.environ)
        env["PYTHONDONTWRITEBYTECODE"] = "1"
        env["PATH"] = f"{fake_bin}{os.pathsep}{env['PATH']}"

        ordinary = subprocess.run(
            [sys.executable, str(SCRIPT), "validate", "--route", "jin"],
            cwd=REPO, env=env, text=True, capture_output=True, check=False,
        )
        allowed = subprocess.run(
            [sys.executable, str(SCRIPT), "validate", "--route", "jin", "--allow-unpublished"],
            cwd=REPO, env=env, text=True, capture_output=True, check=False,
        )

        self.assertNotEqual(ordinary.returncode, 0)
        self.assertNotEqual(allowed.returncode, 0)
        ordinary_payload = json.loads(ordinary.stdout)
        allowed_payload = json.loads(allowed.stdout)
        for payload in (ordinary_payload, allowed_payload):
            self.assertEqual(payload["route_id"], "jin")
        self.assertEqual(ordinary_payload["claim_level"], "authored")
        self.assertEqual(allowed_payload["claim_level"], "mapped")
        self.assertEqual(ordinary_payload["status"], "invalid")
        self.assertEqual(allowed_payload["status"], "incomplete")
        self.assertTrue(all(not marker.exists() for marker in markers))
        after = source_snapshot()
        self.assertEqual(after, before)


if __name__ == "__main__":
    unittest.main()
