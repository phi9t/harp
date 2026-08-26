from __future__ import annotations

import os
from pathlib import Path
import subprocess
import tempfile
import unittest


REPO = Path(__file__).resolve().parents[3]
MISE = REPO / "mise.toml"
TESTS = REPO / "labs/crouzeix_proof_reproduction/tests"
ATLAS_DEPENDENCY_GUARD = REPO / "scripts/check_atlas_dependencies.sh"


def task_section(text: str, task: str) -> str:
    start = text.index(f"[tasks.{task}]")
    end = text.find("\n[tasks.", start + 1)
    return text[start:] if end == -1 else text[start:end]


class VerifyRegistrationTests(unittest.TestCase):
    def test_verify_rust_discovers_the_full_crouzeix_lab_directory(self) -> None:
        text = MISE.read_text()

        self.assertIn(
            "python3 -m unittest discover -s labs/crouzeix_proof_reproduction/tests -v",
            text,
        )
        self.assertNotIn(
            "python3 -m unittest labs.crouzeix_proof_reproduction.tests.test_",
            text,
        )

    def test_all_current_lab_tests_are_discovered_by_directory_registration(self) -> None:
        registered = {path.name for path in TESTS.glob("test_*.py")}

        self.assertGreaterEqual(
            registered,
            {
                "test_tickets.py",
                "test_expert_contracts.py",
                "test_frontier.py",
                "test_frontier_store.py",
                "test_frontier_provider.py",
                "test_prepare_frontier.py",
                "test_expert_runner.py",
                "test_review_contracts.py",
                "test_review_runner.py",
                "test_formal_receipt.py",
                "test_guided_runner.py",
                "test_verify_registration.py",
            },
        )

    def test_lab_tests_use_fake_tools_and_do_not_invoke_live_provider(self) -> None:
        live_provider_patterns = [
            "subprocess.run([" + "'traecli'",
            'subprocess.run(["' + "traecli" + '"',
            "byted" + "cli",
        ]
        for path in TESTS.glob("test_*.py"):
            text = path.read_text()
            for pattern in live_provider_patterns:
                self.assertNotIn(pattern, text)

    def test_verify_atlas_uses_only_preprovisioned_locked_dependencies(self) -> None:
        text = MISE.read_text()
        verify_atlas = task_section(text, "verify-atlas")
        bootstrap = task_section(text, "bootstrap")
        bin_dir = '$atlas_dir/node_modules/.bin'

        self.assertNotIn('depends = ["build"]', verify_atlas)
        self.assertNotIn("corepack pnpm exec", verify_atlas)
        self.assertNotIn("pnpm install", verify_atlas)
        guard = '"$config_root/scripts/check_atlas_dependencies.sh" "$atlas_dir"'
        self.assertIn(guard, verify_atlas)
        self.assertIn("mise run build", verify_atlas)
        self.assertLess(verify_atlas.index(guard), verify_atlas.index("mise run build"))
        self.assertLess(
            verify_atlas.index(guard),
            verify_atlas.index('"$HARP_TARGET_DIR/size/harp" build'),
        )
        self.assertIn(
            f'"{bin_dir}/eslint" . --ignore-pattern dist --ignore-pattern target',
            verify_atlas,
        )
        self.assertEqual(verify_atlas.count(f'"{bin_dir}/tsc" -b'), 2)
        self.assertIn(f'"{bin_dir}/vitest" run', verify_atlas)
        self.assertIn(f'"{bin_dir}/vite" build', verify_atlas)
        self.assertIn("corepack pnpm install --frozen-lockfile", bootstrap)

    def test_atlas_dependency_guard_checks_executable_local_tools(self) -> None:
        self.assertTrue(ATLAS_DEPENDENCY_GUARD.is_file())
        self.assertTrue(os.access(ATLAS_DEPENDENCY_GUARD, os.X_OK))

        with tempfile.TemporaryDirectory() as directory:
            atlas = Path(directory) / "atlas"
            atlas.mkdir()
            missing = subprocess.run(
                [ATLAS_DEPENDENCY_GUARD, atlas],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(missing.returncode, 1)
            self.assertEqual(missing.stdout, "")
            self.assertEqual(
                missing.stderr,
                "Atlas dependencies are missing; run mise run bootstrap\n",
            )

            bin_dir = atlas / "node_modules/.bin"
            bin_dir.mkdir(parents=True)
            for tool in ("eslint", "tsc", "vitest", "vite"):
                executable = bin_dir / tool
                executable.write_text("#!/bin/sh\nexit 0\n")
                executable.chmod(0o755)

            complete = subprocess.run(
                [ATLAS_DEPENDENCY_GUARD, atlas],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(complete.returncode, 0)
            self.assertEqual(complete.stdout, "")
            self.assertEqual(complete.stderr, "")


if __name__ == "__main__":
    unittest.main()
