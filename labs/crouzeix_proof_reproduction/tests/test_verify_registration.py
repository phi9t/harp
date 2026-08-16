from __future__ import annotations

from pathlib import Path
import unittest


REPO = Path(__file__).resolve().parents[3]
MISE = REPO / "mise.toml"
TESTS = REPO / "labs/crouzeix_proof_reproduction/tests"


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


if __name__ == "__main__":
    unittest.main()
