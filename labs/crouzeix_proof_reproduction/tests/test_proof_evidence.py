from __future__ import annotations

import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock


LAB = Path(__file__).resolve().parents[1]
REPO = LAB.parents[1]
SCRIPT = LAB / "proof_evidence.py"
sys.path.insert(0, str(LAB))
import ls_contract  # noqa: E402
import ls_receipts  # noqa: E402
import ls_validation  # noqa: E402
import protocol  # noqa: E402
import proof_evidence  # noqa: E402


PINNED_TOOLCHAIN = "leanprover/lean4:v4.32.1"
MANIFEST_SHA256 = "b349bbab303db9a554fafd3f0eafa8a2422d3b843b5e16c6f796debc4d8f4381"
ROUTE_ORDER = ("jin", "lorist-schwenninger", "harp")
ROUTE_TARGETS = {
    "jin": "CrouzeixJin",
    "lorist-schwenninger": "CrouzeixLoristSchwenninger",
    "harp": "CrouzeixHarp",
}
EXPECTED_DEFAULT_TARGETS = (
    "TrainingDynamics",
    "MathematicalFoundations",
    "NNG4Intro",
    "AutodiffGeometry",
    "Crouzeix",
)
EXPECTED_LEAN_LIBS = (
    "TrainingDynamics",
    "MathematicalFoundations",
    "NNG4Intro",
    "AutodiffGeometry",
    "Crouzeix",
    "CrouzeixJin",
    "CrouzeixLoristSchwenninger",
    "CrouzeixHarp",
    "CrouzeixConjecture",
)
MATHLIB_GIT_URL = "https://github.com/leanprover-community/mathlib4.git"
MATHLIB_MANIFEST_REV = "520045ab14e26149ee970e2e617ca04b09bde5d6"
MATHLIB_INPUT_REV = "v4.32.1"


def write_json(path: Path, value: object) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
        + "\n",
        encoding="utf-8",
    )


def write_module(lean_root: Path, module: str, source: str) -> None:
    path = lean_root.joinpath(*module.split(".")).with_suffix(".lean")
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(source, encoding="utf-8")


def write_fake_olean(path: Path, *, valid: bool = True) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(b"olean-fixture\n" if valid else b"broken-fixture\n")


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical_json_sha256(value: object) -> str:
    payload = json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
        allow_nan=False,
    )
    return sha256_bytes((payload + "\n").encode("utf-8"))


class FixtureBuilder:
    def __init__(self) -> None:
        self._directory = tempfile.TemporaryDirectory()
        self.root = Path(self._directory.name).resolve()
        self.repo = self.root / "repo"
        self.repo.mkdir()
        self.lean_root = self.repo / "formalization/lean"
        self.lean_root.mkdir(parents=True)
        self.cache_root = self.root / "approved-cache"
        self.cache_root.mkdir()
        self.cache_alias = self.root / "approved-cache-alias"
        self.cache_alias.symlink_to(self.cache_root, target_is_directory=True)
        (self.lean_root / ".lake").symlink_to(self.cache_root, target_is_directory=True)
        self.manifest = self.valid_manifest()
        self.write_toolchain(PINNED_TOOLCHAIN)
        self.write_manifest(self.manifest)
        self.write_lakefile(self.valid_lakefile())
        self.write_local_sources()
        self.write_mathlib_sources()
        self.write_required_oleans()

    def cleanup(self) -> None:
        self._directory.cleanup()

    def valid_manifest(self) -> dict[str, object]:
        return {
            "version": "1.2.0",
            "packagesDir": ".lake/packages",
            "packages": [
                {
                    "url": MATHLIB_GIT_URL,
                    "type": "git",
                    "subDir": None,
                    "scope": "",
                    "rev": MATHLIB_MANIFEST_REV,
                    "name": "mathlib",
                    "manifestFile": "lake-manifest.json",
                    "inputRev": "v4.32.1",
                    "inherited": False,
                    "configFile": "lakefile.lean",
                },
                {
                    "url": "https://github.com/leanprover-community/plausible",
                    "type": "git",
                    "subDir": None,
                    "scope": "leanprover-community",
                    "rev": "e12c1910fe855cbfc38803cd4e55543906d5fa62",
                    "name": "plausible",
                    "manifestFile": "lake-manifest.json",
                    "inputRev": "main",
                    "inherited": True,
                    "configFile": "lakefile.toml",
                },
                {
                    "url": "https://github.com/leanprover-community/LeanSearchClient",
                    "type": "git",
                    "subDir": None,
                    "scope": "leanprover-community",
                    "rev": "c5d5b8fe6e5158def25cd28eb94e4141ad97c843",
                    "name": "LeanSearchClient",
                    "manifestFile": "lake-manifest.json",
                    "inputRev": "main",
                    "inherited": True,
                    "configFile": "lakefile.toml",
                },
                {
                    "url": "https://github.com/leanprover-community/import-graph",
                    "type": "git",
                    "subDir": None,
                    "scope": "leanprover-community",
                    "rev": "7e9612bf0b9ee66db3cb5b9988a35afc706f5a12",
                    "name": "importGraph",
                    "manifestFile": "lake-manifest.json",
                    "inputRev": "main",
                    "inherited": True,
                    "configFile": "lakefile.toml",
                },
                {
                    "url": "https://github.com/leanprover-community/ProofWidgets4",
                    "type": "git",
                    "subDir": None,
                    "scope": "leanprover-community",
                    "rev": "6e311e2a844da9b2cc3971187df2fe0066947b93",
                    "name": "proofwidgets",
                    "manifestFile": "lake-manifest.json",
                    "inputRev": "main",
                    "inherited": True,
                    "configFile": "lakefile.lean",
                },
                {
                    "url": "https://github.com/leanprover-community/aesop",
                    "type": "git",
                    "subDir": None,
                    "scope": "leanprover-community",
                    "rev": "a7dbf0c63b694e47f425f3dcddbc0e178bb432d3",
                    "name": "aesop",
                    "manifestFile": "lake-manifest.json",
                    "inputRev": "master",
                    "inherited": True,
                    "configFile": "lakefile.toml",
                },
                {
                    "url": "https://github.com/leanprover-community/quote4",
                    "type": "git",
                    "subDir": None,
                    "scope": "leanprover-community",
                    "rev": "38d591e778f100aec9762bb582f9c7f55f50e9dc",
                    "name": "Qq",
                    "manifestFile": "lake-manifest.json",
                    "inputRev": "master",
                    "inherited": True,
                    "configFile": "lakefile.toml",
                },
                {
                    "url": "https://github.com/leanprover-community/batteries",
                    "type": "git",
                    "subDir": None,
                    "scope": "leanprover-community",
                    "rev": "023ce7d62a0531e22a5331e20b587817a80d49ff",
                    "name": "batteries",
                    "manifestFile": "lake-manifest.json",
                    "inputRev": "main",
                    "inherited": True,
                    "configFile": "lakefile.toml",
                },
                {
                    "url": "https://github.com/leanprover/lean4-cli",
                    "type": "git",
                    "subDir": None,
                    "scope": "leanprover",
                    "rev": "88679d088c9720c27ebdf2ba4dafe17341747f94",
                    "name": "Cli",
                    "manifestFile": "lake-manifest.json",
                    "inputRev": "v4.32.0",
                    "inherited": True,
                    "configFile": "lakefile.toml",
                },
            ],
            "name": "harp_formalization",
            "lakeDir": ".lake",
            "fixedToolchain": False,
        }

    def valid_lakefile(self) -> str:
        default_targets = ",\n  ".join(f'"{item}"' for item in EXPECTED_DEFAULT_TARGETS)
        lean_libs = "\n\n".join(
            f'[[lean_lib]]\nname = "{name}"' for name in EXPECTED_LEAN_LIBS
        )
        return f"""name = "harp_formalization"
defaultTargets = [
  {default_targets}
]

{lean_libs}

[[require]]
name = "mathlib"
git = "https://github.com/leanprover-community/mathlib4.git"
rev = "v4.32.1"
"""

    def manifest_hash(self) -> str:
        return canonical_json_sha256(self.manifest)

    def local_module_path(self, module: str) -> Path:
        return (self.lean_root / Path(*module.split("."))).with_suffix(".lean")

    def mathlib_source_path(self, module: str) -> Path:
        if module == "Mathlib":
            return self.cache_root / "packages/mathlib/Mathlib.lean"
        return (self.cache_root / "packages/mathlib" / Path(*module.split("."))).with_suffix(
            ".lean"
        )

    def olean_path(self, module: str) -> Path:
        return (
            self.cache_root
            / "packages/mathlib/.lake/build/lib/lean"
            / Path(*module.split("."))
        ).with_suffix(".olean")

    def write_toolchain(self, value: str) -> None:
        (self.lean_root / "lean-toolchain").write_text(value + "\n", encoding="utf-8")

    def write_manifest(self, value: object) -> None:
        write_json(self.lean_root / "lake-manifest.json", value)

    def write_lakefile(self, value: str) -> None:
        (self.lean_root / "lakefile.toml").write_text(value, encoding="utf-8")

    def write_local_sources(self) -> None:
        write_module(
            self.lean_root,
            "CrouzeixJin",
            "import Crouzeix.Jin.Terminal\nimport CrouzeixConjecture.HilbertSpectralSet\n",
        )
        write_module(
            self.lean_root,
            "Crouzeix.Jin",
            "import Mathlib.Data.Matrix.Basic\n",
        )
        write_module(
            self.lean_root,
            "Crouzeix.Jin.Terminal",
            "import Mathlib.Analysis.Complex.Basic\n",
        )
        write_module(
            self.lean_root,
            "CrouzeixConjecture",
            "import Mathlib.Data.Matrix.Basic\n",
        )
        write_module(
            self.lean_root,
            "CrouzeixConjecture.HilbertSpectralSet",
            "import Mathlib.Data.Matrix.Basic\n",
        )
        write_module(
            self.lean_root,
            "CrouzeixConjecture.NeutralDependency",
            "import Mathlib.Analysis.SpecificLimits.Basic\n",
        )
        write_module(
            self.lean_root,
            "CrouzeixLoristSchwenninger",
            "import Crouzeix.LoristSchwenninger.Consequences\n",
        )
        write_module(
            self.lean_root,
            "Crouzeix.LoristSchwenninger",
            "import Mathlib.Data.Matrix.Basic\n",
        )
        write_module(
            self.lean_root,
            "Crouzeix.LoristSchwenninger.Consequences",
            "import CrouzeixConjecture.NeutralDependency\n",
        )
        write_module(
            self.lean_root,
            "CrouzeixHarp",
            "import Crouzeix.Harp.Consequences\n",
        )
        write_module(
            self.lean_root,
            "Crouzeix.Harp",
            "import Mathlib.Data.Matrix.Basic\n",
        )
        write_module(
            self.lean_root,
            "Crouzeix.Harp.Consequences",
            "import Crouzeix.Harp.MainTheorem\n",
        )
        write_module(
            self.lean_root,
            "Crouzeix.Harp.MainTheorem",
            "import Crouzeix.LoristSchwenninger.Dilation\n",
        )
        write_module(
            self.lean_root,
            "Crouzeix.LoristSchwenninger.Dilation",
            "import Mathlib.Analysis.InnerProductSpace.Adjoint\n",
        )
        for module in sorted(proof_evidence.HARP_ALLOWED_LS_SUPPORT):
            path = self.local_module_path(module)
            if path.exists():
                continue
            write_module(self.lean_root, module, "import Mathlib.Data.Matrix.Basic\n")

    def write_mathlib_sources(self) -> None:
        sources = {
            "Mathlib.Analysis.Complex.Basic": "import Mathlib.Data.Matrix.Basic\n",
            "Mathlib.Analysis.InnerProductSpace.Adjoint": "import Mathlib.Data.Matrix.Basic\n",
            "Mathlib.Analysis.SpecificLimits.Basic": "import Mathlib.Data.Matrix.Basic\n",
            "Mathlib.Data.Matrix.Basic": "\n",
        }
        for module, source in sources.items():
            path = self.mathlib_source_path(module)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(source, encoding="utf-8")

    def write_required_oleans(self) -> None:
        for module in (
            "Mathlib.Analysis.Complex.Basic",
            "Mathlib.Analysis.InnerProductSpace.Adjoint",
            "Mathlib.Analysis.SpecificLimits.Basic",
            "Mathlib.Data.Matrix.Basic",
        ):
            write_fake_olean(self.olean_path(module))

    def evaluate_route(
        self, route: str, *, approved_cache_root: Path | None = None
    ) -> proof_evidence.PreflightResult:
        return proof_evidence.evaluate_route(self.repo, approved_cache_root or self.cache_root, route)

    def evaluate_all_routes(
        self, *, approved_cache_root: Path | None = None
    ) -> tuple[proof_evidence.PreflightResult, ...]:
        return proof_evidence.evaluate_all_routes(
            self.repo, approved_cache_root or self.cache_root
        )

    def blocked_json(
        self, route: str, *, approved_cache_root: Path | None = None
    ) -> dict[str, object]:
        try:
            result = self.evaluate_route(route, approved_cache_root=approved_cache_root)
        except proof_evidence.PreflightError as error:
            return proof_evidence.blocked_result(error).to_json()
        return result.to_json()

    def python_accepts(self, route: str) -> bool:
        try:
            self.evaluate_route(route)
        except proof_evidence.PreflightError:
            return False
        return True

    def fake_shell_result(self, target: str) -> tuple[subprocess.CompletedProcess[str], Path]:
        fake_bin = self.root / "fake-shell-bin"
        fake_bin.mkdir(exist_ok=True)
        marker = self.root / f"{target}-lake-marker"
        fake_lake = fake_bin / "lake"
        fake_lake.write_text(
            "#!/bin/sh\n: > \"$FAKE_LAKE_MARKER\"\nexit 0\n", encoding="utf-8"
        )
        fake_lake.chmod(0o755)
        env = dict(os.environ)
        env["PATH"] = f"{fake_bin}{os.pathsep}{env['PATH']}"
        env["FAKE_LAKE_MARKER"] = str(marker)
        result = subprocess.run(
            [
                "/bin/sh",
                "scripts/check_lean_library.sh",
                target,
                "--project-for-test",
                str(self.lean_root),
            ],
            cwd=REPO,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )
        return result, marker

    def run_cli(self, *args: str) -> subprocess.CompletedProcess[str]:
        env = dict(os.environ)
        env["PYTHONDONTWRITEBYTECODE"] = "1"
        return subprocess.run(
            [sys.executable, str(SCRIPT), *args],
            cwd=REPO,
            env=env,
            text=True,
            capture_output=True,
            check=False,
        )

    def run_real_cli(self, route: str) -> subprocess.CompletedProcess[str]:
        return self.run_cli("preflight", "--route", route)

    def snapshot_tree(self) -> dict[str, tuple[str, str]]:
        snapshot: dict[str, tuple[str, str]] = {}
        for path in sorted(self.repo.rglob("*")):
            relative = path.relative_to(self.repo).as_posix()
            if path.is_symlink():
                snapshot[relative] = ("symlink", os.readlink(path))
            elif path.is_file():
                snapshot[relative] = ("file", sha256_bytes(path.read_bytes()))
            elif path.is_dir():
                snapshot[relative] = ("dir", "")
        return snapshot


class ProofEvidencePreflightTests(unittest.TestCase):
    maxDiff = None

    def build_fixture(self) -> FixtureBuilder:
        fixture = FixtureBuilder()
        self.addCleanup(fixture.cleanup)
        return fixture

    def test_ready_route_reports_output_contract_fields(self) -> None:
        fixture = self.build_fixture()

        payload = fixture.evaluate_route("jin", approved_cache_root=fixture.cache_alias).to_json()

        self.assertEqual(payload["schema_version"], "crouzeix-proof-preflight/v1")
        self.assertEqual(payload["route_id"], "jin")
        self.assertEqual(payload["status"], "ready")
        self.assertEqual(payload["aggregate_module"], "CrouzeixJin")
        self.assertEqual(payload["toolchain"], PINNED_TOOLCHAIN)
        self.assertEqual(payload["expected_toolchain"], PINNED_TOOLCHAIN)
        self.assertEqual(payload["cache_root"], "formalization/lean/.lake")
        self.assertEqual(payload["cache_identity"], str(fixture.cache_root.resolve()))
        self.assertEqual(
            payload["aggregate_root"],
            str((fixture.lean_root / "CrouzeixJin.lean").resolve()),
        )
        self.assertEqual(
            payload["expected_commands"],
            ["scripts/check_lean_library.sh CrouzeixJin"],
        )
        self.assertEqual(payload["missing_artifacts"], [])

    def test_all_routes_embed_deterministic_contract_fields(self) -> None:
        fixture = self.build_fixture()

        routes = [result.to_json() for result in fixture.evaluate_all_routes()]

        self.assertEqual([route["route_id"] for route in routes], list(ROUTE_ORDER))
        self.assertEqual(
            [route["aggregate_module"] for route in routes],
            [ROUTE_TARGETS[route] for route in ROUTE_ORDER],
        )
        expected_aggregates = [ROUTE_TARGETS[item] for item in ROUTE_ORDER]
        self.assertEqual(len(routes), len(expected_aggregates))
        for route, aggregate in zip(routes, expected_aggregates):
            self.assertEqual(route["cache_root"], "formalization/lean/.lake")
            self.assertEqual(route["cache_identity"], str(fixture.cache_root.resolve()))
            self.assertEqual(
                route["aggregate_root"],
                str((fixture.lean_root / f"{aggregate}.lean").resolve()),
            )
            self.assertEqual(
                route["expected_commands"],
                [f"scripts/check_lean_library.sh {aggregate}"],
            )

    def test_missing_toolchain_reports_null_observed_value(self) -> None:
        fixture = self.build_fixture()
        (fixture.lean_root / "lean-toolchain").unlink()

        payload = fixture.blocked_json("jin")

        self.assertEqual(payload["status"], "blocked")
        self.assertIsNone(payload["toolchain"])
        self.assertEqual(payload["expected_toolchain"], PINNED_TOOLCHAIN)
        self.assertIn("lean-toolchain is missing", payload["reason"])

    def test_wrong_toolchain_reports_observed_value(self) -> None:
        fixture = self.build_fixture()
        fixture.write_toolchain("leanprover/lean4:v4.31.0")

        payload = fixture.blocked_json("jin")

        self.assertEqual(payload["status"], "blocked")
        self.assertEqual(payload["toolchain"], "leanprover/lean4:v4.31.0")
        self.assertEqual(payload["expected_toolchain"], PINNED_TOOLCHAIN)
        self.assertIn("lean-toolchain is not pinned", payload["reason"])

    def test_cli_rejects_removed_override_flags(self) -> None:
        fixture = self.build_fixture()

        result = fixture.run_cli(
            "preflight",
            "--route",
            "jin",
            "--repository-root-for-test",
            str(fixture.repo),
        )

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("unrecognized arguments", result.stderr)

    def test_publish_ls_rejects_forbidden_flags_and_positional_extras(self) -> None:
        fixture = self.build_fixture()
        cases = (
            ("publish-ls", "--path", "override"),
            ("publish-ls", "--command", "override"),
            ("publish-ls", "--env", "override"),
            ("publish-ls", "--row", "override"),
            ("publish-ls", "--output", "override"),
            ("publish-ls", "extra"),
        )
        for argv in cases:
            with self.subTest(argv=argv):
                before = fixture.snapshot_tree()
                result = fixture.run_cli(*argv)
                after = fixture.snapshot_tree()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("unrecognized arguments", result.stderr)
                self.assertEqual(
                    before, after, "publish-ls changed the repository tree"
                )

    def test_publish_ls_emits_closed_success_payload_in_node_order(self) -> None:
        fixture = self.build_fixture()
        before = fixture.snapshot_tree()
        rows = ("row-sentinel",)
        published = {
            node_id: {
                "attempt_path": (
                    "labs/crouzeix_proof_reproduction/formal_targets/"
                    f"lorist-schwenninger/proof-slices/{node_id}/attempt-001"
                ),
                "receipt_sha256": "a" * 64,
                "lean_name": "ignored",
                "status": "passed",
            }
            for node_id in reversed(ls_contract.NODE_ORDER)
        }
        stdout: list[str] = []

        with (
            mock.patch.object(
                proof_evidence, "canonical_repository_root", return_value=fixture.repo
            ),
            mock.patch.object(
                proof_evidence.ls_validation, "load_route_graph", return_value=rows
            ),
            mock.patch.object(
                proof_evidence.ls_receipts,
                "publish_ls_receipts",
                return_value=published,
            ),
            mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock,
        ):
            stdout_mock.write.side_effect = stdout.append
            exit_code = proof_evidence.main(["publish-ls"])

        after = fixture.snapshot_tree()
        payload = json.loads("".join(stdout))
        self.assertEqual(exit_code, 0)
        self.assertEqual(
            set(payload),
            {"schema_version", "route_id", "status", "candidates"},
        )
        self.assertEqual(
            payload["schema_version"], "crouzeix-ls-receipt-publication/v1"
        )
        self.assertEqual(payload["route_id"], "lorist-schwenninger")
        self.assertEqual(payload["status"], "published-unreferenced")
        self.assertEqual(
            [candidate["node_id"] for candidate in payload["candidates"]],
            list(ls_contract.NODE_ORDER),
        )
        for candidate in payload["candidates"]:
            self.assertEqual(
                set(candidate), {"node_id", "attempt_path", "receipt_sha256"}
            )
        self.assertEqual(before, after, "publish-ls changed the repository tree")

    def test_publish_ls_rejects_success_path_that_binds_to_different_node(self) -> None:
        fixture = self.build_fixture()
        rows = ("row-sentinel",)
        published = {
            node_id: {
                "attempt_path": (
                    "labs/crouzeix_proof_reproduction/formal_targets/"
                    f"lorist-schwenninger/proof-slices/{node_id}/attempt-001"
                ),
                "receipt_sha256": "a" * 64,
                "lean_name": "ignored",
                "status": "passed",
            }
            for node_id in ls_contract.NODE_ORDER
        }
        published["ls-equation-one-terminal-bound"] = {
            "attempt_path": (
                "labs/crouzeix_proof_reproduction/formal_targets/"
                "lorist-schwenninger/proof-slices/ls-terminal-crouzeix/attempt-001"
            ),
            "receipt_sha256": "a" * 64,
            "lean_name": "ignored",
            "status": "passed",
        }
        stdout: list[str] = []

        with (
            mock.patch.object(
                proof_evidence, "canonical_repository_root", return_value=fixture.repo
            ),
            mock.patch.object(
                proof_evidence.ls_validation, "load_route_graph", return_value=rows
            ),
            mock.patch.object(
                proof_evidence.ls_receipts,
                "publish_ls_receipts",
                return_value=published,
            ),
            mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock,
        ):
            stdout_mock.write.side_effect = stdout.append
            exit_code = proof_evidence.main(["publish-ls"])

        payload = json.loads("".join(stdout))
        self.assertEqual(exit_code, 1)
        self.assertEqual(payload["status"], "blocked")
        self.assertIn("does not match node_id", payload["reason"])

    def test_publish_ls_rejects_success_path_alias_suffix_tricks(self) -> None:
        fixture = self.build_fixture()
        rows = ("row-sentinel",)
        published = {
            node_id: {
                "attempt_path": (
                    "labs/crouzeix_proof_reproduction/formal_targets/"
                    f"lorist-schwenninger/proof-slices/{node_id}/attempt-001"
                ),
                "receipt_sha256": "a" * 64,
                "lean_name": "ignored",
                "status": "passed",
            }
            for node_id in ls_contract.NODE_ORDER
        }
        published["ls-terminal-crouzeix"] = {
            "attempt_path": (
                "labs/crouzeix_proof_reproduction/formal_targets/"
                "lorist-schwenninger/proof-slices/ls-terminal-crouzeix/attempt-001/suffix"
            ),
            "receipt_sha256": "a" * 64,
            "lean_name": "ignored",
            "status": "passed",
        }
        stdout: list[str] = []

        with (
            mock.patch.object(
                proof_evidence, "canonical_repository_root", return_value=fixture.repo
            ),
            mock.patch.object(
                proof_evidence.ls_validation, "load_route_graph", return_value=rows
            ),
            mock.patch.object(
                proof_evidence.ls_receipts,
                "publish_ls_receipts",
                return_value=published,
            ),
            mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock,
        ):
            stdout_mock.write.side_effect = stdout.append
            exit_code = proof_evidence.main(["publish-ls"])

        payload = json.loads("".join(stdout))
        self.assertEqual(exit_code, 1)
        self.assertEqual(payload["status"], "blocked")
        self.assertIn("canonical LS node", payload["reason"])

    def test_publish_ls_rejects_malformed_publisher_success_result(self) -> None:
        fixture = self.build_fixture()
        rows = ("row-sentinel",)
        valid = {
            node_id: {
                "attempt_path": (
                    "labs/crouzeix_proof_reproduction/formal_targets/"
                    f"lorist-schwenninger/proof-slices/{node_id}/attempt-001"
                ),
                "receipt_sha256": "a" * 64,
                "lean_name": "ignored",
                "status": "passed",
            }
            for node_id in ls_contract.NODE_ORDER
        }
        cases = {
            "missing-node": {
                node_id: value
                for node_id, value in valid.items()
                if node_id != ls_contract.NODE_ORDER[-1]
            },
            "extra-node": {
                **valid,
                "unknown-node": {
                    "attempt_path": (
                        "labs/crouzeix_proof_reproduction/formal_targets/"
                        "lorist-schwenninger/proof-slices/unknown-node/attempt-001"
                    ),
                    "receipt_sha256": "b" * 64,
                },
            },
            "unsafe-path": {
                **valid,
                ls_contract.NODE_ORDER[0]: {
                    "attempt_path": "../escape/attempt-001",
                    "receipt_sha256": "a" * 64,
                },
            },
            "bad-digest": {
                **valid,
                ls_contract.NODE_ORDER[0]: {
                    "attempt_path": valid[ls_contract.NODE_ORDER[0]]["attempt_path"],
                    "receipt_sha256": "not-a-digest",
                },
            },
        }
        for label, published in cases.items():
            with self.subTest(label=label):
                stdout: list[str] = []
                with (
                    mock.patch.object(
                        proof_evidence,
                        "canonical_repository_root",
                        return_value=fixture.repo,
                    ),
                    mock.patch.object(
                        proof_evidence.ls_validation,
                        "load_route_graph",
                        return_value=rows,
                    ),
                    mock.patch.object(
                        proof_evidence.ls_receipts,
                        "publish_ls_receipts",
                        return_value=published,
                    ),
                    mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock,
                ):
                    stdout_mock.write.side_effect = stdout.append
                    exit_code = proof_evidence.main(["publish-ls"])
                payload = json.loads("".join(stdout))
                self.assertEqual(exit_code, 1)
                self.assertEqual(payload["status"], "blocked")
                self.assertIn("publish-ls", payload["reason"])

    def test_publish_ls_normalizes_validation_error_to_blocked_json(self) -> None:
        fixture = self.build_fixture()
        stdout: list[str] = []

        with (
            mock.patch.object(
                proof_evidence, "canonical_repository_root", return_value=fixture.repo
            ),
            mock.patch.object(
                proof_evidence.ls_validation,
                "load_route_graph",
                side_effect=protocol.ValidationError("graph is invalid"),
            ),
            mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock,
        ):
            stdout_mock.write.side_effect = stdout.append
            exit_code = proof_evidence.main(["publish-ls"])

        payload = json.loads("".join(stdout))
        self.assertEqual(exit_code, 1)
        self.assertEqual(
            payload,
            {
                "schema_version": "crouzeix-ls-receipt-publication/v1",
                "route_id": "lorist-schwenninger",
                "status": "blocked",
                "reason": "graph is invalid",
            },
        )

    def test_publish_ls_normalizes_partial_publication_error(self) -> None:
        fixture = self.build_fixture()
        before = fixture.snapshot_tree()
        rows = ("row-sentinel",)
        candidates = tuple(
            {
                "attempt_path": (
                    "labs/crouzeix_proof_reproduction/formal_targets/"
                    f"lorist-schwenninger/proof-slices/{node_id}/attempt-001"
                ),
                "receipt_sha256": digest,
            }
            for node_id, digest in (
                ("ls-terminal-crouzeix", "c" * 64),
                ("ls-equation-one-terminal-bound", "a" * 64),
            )
        )
        stdout: list[str] = []

        with (
            mock.patch.object(
                proof_evidence, "canonical_repository_root", return_value=fixture.repo
            ),
            mock.patch.object(
                proof_evidence.ls_validation, "load_route_graph", return_value=rows
            ),
            mock.patch.object(
                proof_evidence.ls_receipts,
                "publish_ls_receipts",
                side_effect=ls_receipts.PartialPublicationError(
                    "visible candidates remain unreferenced", candidates
                ),
            ),
            mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock,
        ):
            stdout_mock.write.side_effect = stdout.append
            exit_code = proof_evidence.main(["publish-ls"])

        after = fixture.snapshot_tree()
        payload = json.loads("".join(stdout))
        self.assertEqual(exit_code, 1)
        self.assertEqual(
            set(payload),
            {
                "schema_version",
                "route_id",
                "status",
                "reason",
                "recovery_status",
                "invalid_candidate_count",
                "candidates",
            },
        )
        self.assertEqual(
            payload["schema_version"], "crouzeix-ls-receipt-publication/v1"
        )
        self.assertEqual(payload["route_id"], "lorist-schwenninger")
        self.assertEqual(payload["status"], "blocked")
        self.assertIn("visible candidates remain unreferenced", payload["reason"])
        self.assertEqual(payload["recovery_status"], "complete-candidate-set")
        self.assertEqual(payload["invalid_candidate_count"], 0)
        self.assertEqual(
            [candidate["node_id"] for candidate in payload["candidates"]],
            ["ls-equation-one-terminal-bound", "ls-terminal-crouzeix"],
        )
        for candidate in payload["candidates"]:
            self.assertEqual(
                set(candidate), {"node_id", "attempt_path", "receipt_sha256"}
            )
        self.assertEqual(before, after, "publish-ls changed the repository tree")

    def test_publish_ls_rejects_malformed_partial_candidate_metadata(self) -> None:
        fixture = self.build_fixture()
        rows = ("row-sentinel",)
        valid_path = (
            "labs/crouzeix_proof_reproduction/formal_targets/"
            "lorist-schwenninger/proof-slices/ls-terminal-crouzeix/attempt-001"
        )
        cases = (
            (
                {
                    "attempt_path": valid_path,
                    "receipt_sha256": "a" * 64,
                    "extra": "field",
                },
            ),
            (
                {
                    "attempt_path": valid_path,
                    "receipt_sha256": "a" * 64,
                },
                {
                    "attempt_path": valid_path,
                    "receipt_sha256": "b" * 64,
                },
            ),
            (
                {
                    "attempt_path": "../escape/attempt-001",
                    "receipt_sha256": "a" * 64,
                },
            ),
            (
                {
                    "attempt_path": valid_path,
                    "receipt_sha256": "not-a-digest",
                },
            ),
        )
        for candidates in cases:
            with self.subTest(candidates=candidates):
                stdout: list[str] = []
                with (
                    mock.patch.object(
                        proof_evidence,
                        "canonical_repository_root",
                        return_value=fixture.repo,
                    ),
                    mock.patch.object(
                        proof_evidence.ls_validation,
                        "load_route_graph",
                        return_value=rows,
                    ),
                    mock.patch.object(
                        proof_evidence.ls_receipts,
                        "publish_ls_receipts",
                        side_effect=ls_receipts.PartialPublicationError(
                            "visible candidates remain unreferenced", candidates
                        ),
                    ),
                    mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock,
                ):
                    stdout_mock.write.side_effect = stdout.append
                    exit_code = proof_evidence.main(["publish-ls"])
                payload = json.loads("".join(stdout))
                self.assertEqual(exit_code, 1)
                self.assertEqual(payload["status"], "blocked")
                self.assertEqual(payload["recovery_status"], "incomplete-candidate-set")
                expected_invalid = 1 if len(candidates) == 2 else len(candidates)
                self.assertEqual(payload["invalid_candidate_count"], expected_invalid)
                if len(candidates) == 2:
                    self.assertEqual(
                        payload["candidates"],
                        [
                            {
                                "node_id": "ls-terminal-crouzeix",
                                "attempt_path": valid_path,
                                "receipt_sha256": "a" * 64,
                            }
                        ],
                    )
                else:
                    self.assertEqual(payload["candidates"], [])

    def test_publish_ls_partial_preserves_valid_subset_when_entries_are_mixed(self) -> None:
        fixture = self.build_fixture()
        rows = ("row-sentinel",)
        candidates = (
            {
                "attempt_path": (
                    "labs/crouzeix_proof_reproduction/formal_targets/"
                    "lorist-schwenninger/proof-slices/ls-terminal-crouzeix/attempt-001"
                ),
                "receipt_sha256": "c" * 64,
            },
            {
                "attempt_path": "../escape/attempt-001",
                "receipt_sha256": "d" * 64,
            },
            {
                "attempt_path": (
                    "labs/crouzeix_proof_reproduction/formal_targets/"
                    "lorist-schwenninger/proof-slices/ls-equation-one-terminal-bound/attempt-001"
                ),
                "receipt_sha256": "a" * 64,
            },
            {
                "attempt_path": (
                    "labs/crouzeix_proof_reproduction/formal_targets/"
                    "lorist-schwenninger/proof-slices/ls-terminal-crouzeix/attempt-002"
                ),
                "receipt_sha256": "e" * 64,
            },
        )
        stdout: list[str] = []

        with (
            mock.patch.object(
                proof_evidence, "canonical_repository_root", return_value=fixture.repo
            ),
            mock.patch.object(
                proof_evidence.ls_validation, "load_route_graph", return_value=rows
            ),
            mock.patch.object(
                proof_evidence.ls_receipts,
                "publish_ls_receipts",
                side_effect=ls_receipts.PartialPublicationError(
                    "visible candidates remain unreferenced", candidates
                ),
            ),
            mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock,
        ):
            stdout_mock.write.side_effect = stdout.append
            exit_code = proof_evidence.main(["publish-ls"])

        payload = json.loads("".join(stdout))
        self.assertEqual(exit_code, 1)
        self.assertEqual(payload["status"], "blocked")
        self.assertEqual(payload["recovery_status"], "incomplete-candidate-set")
        self.assertEqual(payload["invalid_candidate_count"], 2)
        self.assertEqual(
            [candidate["node_id"] for candidate in payload["candidates"]],
            ["ls-equation-one-terminal-bound", "ls-terminal-crouzeix"],
        )
        self.assertEqual(
            payload["candidates"][1]["receipt_sha256"],
            "c" * 64,
        )

    def test_publish_ls_bounds_and_sanitizes_validation_reasons(self) -> None:
        fixture = self.build_fixture()
        rows = ("row-sentinel",)
        long_reason = ("bad\x00reason\n" * 700)
        stdout: list[str] = []

        with (
            mock.patch.object(
                proof_evidence, "canonical_repository_root", return_value=fixture.repo
            ),
            mock.patch.object(
                proof_evidence.ls_validation, "load_route_graph", return_value=rows
            ),
            mock.patch.object(
                proof_evidence.ls_receipts,
                "publish_ls_receipts",
                side_effect=protocol.ValidationError(long_reason),
            ),
            mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock,
        ):
            stdout_mock.write.side_effect = stdout.append
            exit_code = proof_evidence.main(["publish-ls"])

        payload = json.loads("".join(stdout))
        reason = payload["reason"]
        self.assertEqual(exit_code, 1)
        self.assertEqual(payload["status"], "blocked")
        self.assertLessEqual(len(reason.encode("utf-8")), 4096)
        self.assertNotIn("\x00", reason)
        self.assertNotIn("\n", reason)

    def test_publish_ls_bounds_and_sanitizes_partial_reason(self) -> None:
        fixture = self.build_fixture()
        rows = ("row-sentinel",)
        stdout: list[str] = []
        candidates = (
            {
                "attempt_path": (
                    "labs/crouzeix_proof_reproduction/formal_targets/"
                    "lorist-schwenninger/proof-slices/ls-terminal-crouzeix/attempt-001"
                ),
                "receipt_sha256": "c" * 64,
            },
        )
        long_reason = ("partial\x00reason\n" * 700)

        with (
            mock.patch.object(
                proof_evidence, "canonical_repository_root", return_value=fixture.repo
            ),
            mock.patch.object(
                proof_evidence.ls_validation, "load_route_graph", return_value=rows
            ),
            mock.patch.object(
                proof_evidence.ls_receipts,
                "publish_ls_receipts",
                side_effect=ls_receipts.PartialPublicationError(long_reason, candidates),
            ),
            mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock,
        ):
            stdout_mock.write.side_effect = stdout.append
            exit_code = proof_evidence.main(["publish-ls"])

        payload = json.loads("".join(stdout))
        reason = payload["reason"]
        self.assertEqual(exit_code, 1)
        self.assertEqual(payload["status"], "blocked")
        self.assertLessEqual(len(reason.encode("utf-8")), 4096)
        self.assertNotIn("\x00", reason)
        self.assertNotIn("\n", reason)

    def test_production_source_contains_no_workspace_absolute_paths(self) -> None:
        source = SCRIPT.read_text(encoding="utf-8")

        self.assertNotIn("/Users/", source)
        self.assertNotIn("/home/", source)

    def test_primary_checkout_cache_derives_from_git_common_dir(self) -> None:
        fixture = self.build_fixture()
        expected_primary = fixture.repo.resolve()
        git_common_dir = expected_primary / ".git"
        git_common_dir.mkdir()
        (expected_primary / "scripts").mkdir()
        (expected_primary / "scripts/check_lean_library.sh").write_text("", encoding="utf-8")
        (expected_primary / "crates/harp").mkdir(parents=True)
        (expected_primary / "formalization/lean").mkdir(parents=True, exist_ok=True)
        (expected_primary / "formalization/lean/lake-manifest.json").write_text(
            "{}\n", encoding="utf-8"
        )
        (expected_primary / "formalization/lean/lakefile.toml").write_text(
            'name = "harp_formalization"\n', encoding="utf-8"
        )

        with mock.patch.object(proof_evidence.subprocess, "run") as run_mock:
            run_mock.return_value = subprocess.CompletedProcess(
                args=[
                    "git",
                    "-C",
                    str(fixture.repo),
                    "rev-parse",
                    "--path-format=absolute",
                    "--git-common-dir",
                ],
                returncode=0,
                stdout=str(git_common_dir) + "\n",
                stderr="",
            )

            primary_root, approved_cache = proof_evidence.resolve_primary_checkout_cache(
                fixture.repo
            )

        self.assertEqual(primary_root, expected_primary)
        self.assertEqual(approved_cache, expected_primary / "formalization/lean/.lake")
        run_mock.assert_called_once()

    def test_primary_checkout_cache_fails_closed_when_git_common_dir_is_unavailable(self) -> None:
        with mock.patch.object(proof_evidence.subprocess, "run") as run_mock:
            run_mock.return_value = subprocess.CompletedProcess(
                args=["git"],
                returncode=128,
                stdout="",
                stderr="fatal: not a git repository\n",
            )

            with self.assertRaisesRegex(
                RuntimeError, "git common-dir lookup failed: fatal: not a git repository"
            ):
                proof_evidence.resolve_primary_checkout_cache(REPO)

    def test_cli_git_common_dir_failure_blocks_without_invoking_preflight(self) -> None:
        with mock.patch.object(
            proof_evidence, "resolve_primary_checkout_cache", side_effect=RuntimeError("git common-dir lookup failed: broken metadata")
        ) as resolve_mock, mock.patch.object(
            proof_evidence, "evaluate_route"
        ) as eval_mock:
            stdout = []
            with mock.patch.object(proof_evidence.sys, "stdout") as stdout_mock:
                stdout_mock.write.side_effect = stdout.append
                exit_code = proof_evidence.main(["preflight", "--route", "jin"])

        self.assertEqual(exit_code, 1)
        self.assertIn("git common-dir lookup failed: broken metadata", "".join(stdout))
        resolve_mock.assert_called_once()
        eval_mock.assert_not_called()

    def test_malformed_manifest_blocks(self) -> None:
        fixture = self.build_fixture()
        (fixture.lean_root / "lake-manifest.json").write_text("{bad json\n", encoding="utf-8")

        payload = fixture.blocked_json("jin")

        self.assertIn("lake-manifest.json is invalid JSON", payload["reason"])

    def test_manifest_full_graph_hash_binds_non_mathlib_and_fixed_toolchain(self) -> None:
        fixture = self.build_fixture()
        self.assertEqual(fixture.manifest_hash(), MANIFEST_SHA256)

        manifest = fixture.valid_manifest()
        manifest["packages"][1]["rev"] = "deadbeef"
        fixture.write_manifest(manifest)
        payload = fixture.blocked_json("jin")
        self.assertIn("lake-manifest.json contract hash is invalid", payload["reason"])

        manifest = fixture.valid_manifest()
        manifest["fixedToolchain"] = True
        fixture.write_manifest(manifest)
        payload = fixture.blocked_json("jin")
        self.assertIn("lake-manifest.json contract hash is invalid", payload["reason"])

    def test_lakefile_exact_contract_rejects_missing_duplicate_and_extra_entries(self) -> None:
        fixture = self.build_fixture()
        fixture.write_lakefile(fixture.valid_lakefile().replace('  "Crouzeix"\n', ""))
        payload = fixture.blocked_json("jin")
        self.assertIn("lakefile.toml defaultTargets are invalid", payload["reason"])

        fixture.write_lakefile(
            fixture.valid_lakefile() + '\n[[lean_lib]]\nname = "CrouzeixJin"\n'
        )
        payload = fixture.blocked_json("jin")
        self.assertIn("lakefile.toml lean_lib names are invalid", payload["reason"])

        fixture.write_lakefile(
            fixture.valid_lakefile() + '\n[[lean_lib]]\nname = "ExtraLib"\n'
        )
        payload = fixture.blocked_json("jin")
        self.assertIn("lakefile.toml lean_lib names are invalid", payload["reason"])

        fixture.write_lakefile(
            fixture.valid_lakefile()
            + '\n[[require]]\nname = "other"\ngit = "https://example.invalid/other.git"\nrev = "main"\n'
        )
        payload = fixture.blocked_json("jin")
        self.assertIn("lakefile.toml require entries are invalid", payload["reason"])

    def test_lakefile_exact_contract_rejects_extra_top_level_key_and_library_field(self) -> None:
        fixture = self.build_fixture()
        fixture.write_lakefile('unexpected = true\n' + fixture.valid_lakefile())
        payload = fixture.blocked_json("jin")
        self.assertIn("lakefile.toml contract is invalid", payload["reason"])

        fixture.write_lakefile(
            fixture.valid_lakefile().replace(
                '[[lean_lib]]\nname = "TrainingDynamics"',
                '[[lean_lib]]\nname = "TrainingDynamics"\nsrcDir = "TrainingDynamics"',
                1,
            )
        )
        payload = fixture.blocked_json("jin")
        self.assertIn("lakefile.toml lean_lib names are invalid", payload["reason"])

    def test_transitive_mathlib_olean_missing_blocks(self) -> None:
        fixture = self.build_fixture()
        fixture.olean_path("Mathlib.Data.Matrix.Basic").unlink()

        payload = fixture.blocked_json("jin")

        self.assertEqual(payload["reason"], "missing-mathlib-artifacts")
        self.assertEqual(payload["missing_artifacts"], ["Mathlib/Data/Matrix/Basic.olean"])

    def test_transitive_mathlib_olean_corrupt_blocks(self) -> None:
        fixture = self.build_fixture()
        write_fake_olean(fixture.olean_path("Mathlib.Data.Matrix.Basic"), valid=False)

        payload = fixture.blocked_json("jin")

        self.assertEqual(payload["reason"], "missing-mathlib-artifacts")
        self.assertEqual(payload["missing_artifacts"], ["Mathlib/Data/Matrix/Basic.olean"])

    def test_direct_provider_policy_violation_blocks(self) -> None:
        fixture = self.build_fixture()
        write_module(
            fixture.lean_root,
            "CrouzeixJin",
            "import Crouzeix.LoristSchwenninger.Consequences\n",
        )

        payload = fixture.blocked_json("jin")

        self.assertIn(
            "rejected provider import Crouzeix.LoristSchwenninger.Consequences",
            payload["reason"],
        )

    def test_top_level_aggregate_provider_policy_violations_block(self) -> None:
        cases = (
            ("jin", "CrouzeixJin", "CrouzeixHarp", "rejected provider import CrouzeixHarp"),
            (
                "lorist-schwenninger",
                "CrouzeixLoristSchwenninger",
                "CrouzeixJin",
                "rejected provider import CrouzeixJin",
            ),
            (
                "harp",
                "CrouzeixHarp",
                "CrouzeixLoristSchwenninger",
                "rejected provider import CrouzeixLoristSchwenninger",
            ),
        )

        for route, aggregate, imported, reason in cases:
            fixture = self.build_fixture()
            with self.subTest(route=route, imported=imported):
                write_module(fixture.lean_root, aggregate, f"import {imported}\n")
                payload = fixture.blocked_json(route)
                self.assertEqual(payload["status"], "blocked")
                self.assertEqual(payload["aggregate_module"], aggregate)
                self.assertEqual(payload["reason"], reason)

    def test_harp_accepts_enumerated_ls_support_module(self) -> None:
        fixture = self.build_fixture()

        payload = fixture.evaluate_route("harp").to_json()

        self.assertEqual(payload["status"], "ready")
        self.assertIn("Crouzeix.LoristSchwenninger.Dilation", payload["local_modules"])

    def test_harp_transitive_provider_policy_violation_blocks(self) -> None:
        fixture = self.build_fixture()
        write_module(
            fixture.lean_root,
            "Crouzeix.LoristSchwenninger.Dilation",
            "import Crouzeix.Jin.Terminal\n",
        )

        payload = fixture.blocked_json("harp")

        self.assertIn("rejected provider import Crouzeix.Jin.Terminal", payload["reason"])

    def test_local_source_symlink_escape_blocks(self) -> None:
        fixture = self.build_fixture()
        outside = fixture.root / "outside-local.lean"
        outside.write_text("import Mathlib.Analysis.Complex.Basic\n", encoding="utf-8")
        fixture.local_module_path("CrouzeixJin").unlink()
        fixture.local_module_path("CrouzeixJin").symlink_to(outside)

        payload = fixture.blocked_json("jin")

        self.assertIn("symlink", payload["reason"])

    def test_mathlib_source_symlink_escape_blocks(self) -> None:
        fixture = self.build_fixture()
        outside = fixture.root / "outside-mathlib.lean"
        outside.write_text("\n", encoding="utf-8")
        path = fixture.mathlib_source_path("Mathlib.Analysis.Complex.Basic")
        path.unlink()
        path.symlink_to(outside)

        payload = fixture.blocked_json("jin")

        self.assertIn("symlink", payload["reason"])

    def test_mathlib_source_root_symlink_escape_blocks(self) -> None:
        fixture = self.build_fixture()
        outside = fixture.root / "outside-mathlib-root"
        outside.mkdir()
        (outside / "Mathlib.lean").write_text("\n", encoding="utf-8")
        shutil.rmtree(fixture.cache_root / "packages/mathlib")
        (fixture.cache_root / "packages/mathlib").symlink_to(outside, target_is_directory=True)

        payload = fixture.blocked_json("jin")

        self.assertIn("mathlib source root", payload["reason"])
        self.assertIn("symlink", payload["reason"])

    def test_olean_symlink_escape_blocks(self) -> None:
        fixture = self.build_fixture()
        outside = fixture.root / "outside.olean"
        outside.write_bytes(b"olean-outside\n")
        path = fixture.olean_path("Mathlib.Analysis.Complex.Basic")
        path.unlink()
        path.symlink_to(outside)

        payload = fixture.blocked_json("jin")

        self.assertIn("symlink", payload["reason"])

    def test_mathlib_artifact_root_symlink_escape_blocks(self) -> None:
        fixture = self.build_fixture()
        outside = fixture.root / "outside-artifact-root"
        outside.mkdir()
        shutil.rmtree(fixture.cache_root / "packages/mathlib/.lake/build/lib/lean")
        (fixture.cache_root / "packages/mathlib/.lake/build/lib/lean").symlink_to(
            outside,
            target_is_directory=True,
        )

        payload = fixture.blocked_json("jin")

        self.assertIn("mathlib artifact root", payload["reason"])
        self.assertIn("symlink", payload["reason"])

    def test_preflight_does_not_invoke_lake_or_write_files(self) -> None:
        fixture = self.build_fixture()
        fake_bin = fixture.root / "fake-bin"
        fake_bin.mkdir()
        marker = fixture.root / "lake-marker"
        fake_lake = fake_bin / "lake"
        fake_lake.write_text("#!/bin/sh\n: > \"$FAKE_LAKE_MARKER\"\nexit 99\n", encoding="utf-8")
        fake_lake.chmod(0o755)
        before = fixture.snapshot_tree()
        original_path = os.environ.get("PATH", "")
        original_marker = os.environ.get("FAKE_LAKE_MARKER")
        os.environ["PATH"] = f"{fake_bin}{os.pathsep}{original_path}"
        os.environ["FAKE_LAKE_MARKER"] = str(marker)
        try:
            routes = [result.to_json() for result in fixture.evaluate_all_routes()]
        finally:
            os.environ["PATH"] = original_path
            if original_marker is None:
                os.environ.pop("FAKE_LAKE_MARKER", None)
            else:
                os.environ["FAKE_LAKE_MARKER"] = original_marker
        after = fixture.snapshot_tree()

        self.assertEqual([route["route_id"] for route in routes], list(ROUTE_ORDER))
        self.assertFalse(marker.exists(), "preflight invoked lake")
        self.assertEqual(before, after, "preflight wrote to the repository tree")

    def test_shell_python_provider_policy_parity(self) -> None:
        cases = {
            "jin": (
                ("CrouzeixJin", True),
                ("Crouzeix.Jin", True),
                ("Crouzeix.Jin.Terminal", True),
                ("CrouzeixConjecture", True),
                ("CrouzeixConjecture.NeutralDependency", True),
                ("CrouzeixHarp", False),
                ("Crouzeix.Harp", False),
                ("Crouzeix.Harp.Consequences", False),
                ("CrouzeixLoristSchwenninger", False),
                ("Crouzeix.LoristSchwenninger", False),
                ("Crouzeix.LoristSchwenninger.Consequences", False),
                ("Crouzeix", False),
            ),
            "lorist-schwenninger": (
                ("CrouzeixLoristSchwenninger", True),
                ("Crouzeix.LoristSchwenninger", True),
                ("Crouzeix.LoristSchwenninger.Consequences", True),
                ("CrouzeixConjecture", True),
                ("CrouzeixConjecture.NeutralDependency", True),
                ("CrouzeixJin", False),
                ("Crouzeix.Jin", False),
                ("Crouzeix.Jin.Terminal", False),
                ("CrouzeixHarp", False),
                ("Crouzeix.Harp", False),
                ("Crouzeix.Harp.Consequences", False),
                ("Crouzeix", False),
            ),
            "harp": (
                ("CrouzeixHarp", True),
                ("Crouzeix.Harp", True),
                ("Crouzeix.Harp.Consequences", True),
                ("CrouzeixConjecture", True),
                ("CrouzeixConjecture.NeutralDependency", True),
                *[(module, True) for module in sorted(proof_evidence.HARP_ALLOWED_LS_SUPPORT)],
                ("CrouzeixJin", False),
                ("Crouzeix.Jin", False),
                ("Crouzeix.Jin.Terminal", False),
                ("CrouzeixLoristSchwenninger", False),
                ("Crouzeix.LoristSchwenninger", False),
                ("Crouzeix.LoristSchwenninger.Consequences", False),
                ("Crouzeix.LoristSchwenninger.MainTheorem", False),
                ("Crouzeix.LoristSchwenninger.ConcreteDilation", False),
                ("Crouzeix", False),
            ),
        }
        for route, route_cases in cases.items():
            target = ROUTE_TARGETS[route]
            for imported, expected in route_cases:
                fixture = self.build_fixture()
                with self.subTest(route=route, imported=imported, expected=expected):
                    write_module(fixture.lean_root, target, f"import {imported}\n")
                    if expected and imported != target:
                        path = fixture.local_module_path(imported)
                        if not path.exists():
                            write_module(fixture.lean_root, imported, "import Mathlib.Data.Matrix.Basic\n")
                    python_allowed = fixture.python_accepts(route)
                    shell_result, marker = fixture.fake_shell_result(target)
                    shell_allowed = shell_result.returncode == 0
                    if not expected:
                        self.assertFalse(marker.exists(), "shell hit lake before rejecting")
                    self.assertEqual(python_allowed, expected)
                    self.assertEqual(shell_allowed, expected)

    def test_cli_real_repository_still_supports_route_all(self) -> None:
        result = self.build_fixture().run_real_cli("all")

        self.assertEqual(result.returncode, 0, result.stderr)
        payload = json.loads(result.stdout)
        self.assertEqual(payload["route_id"], "all")


if __name__ == "__main__":
    unittest.main()
