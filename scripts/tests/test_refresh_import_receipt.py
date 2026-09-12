"""Behavioral tests for the import-receipt digest refresh.

Every case here is a step of the sequence that actually broke the receipt: a
verifier failure that was not about the digest, a report with no digest in it,
and a substitution that produced an empty pin. The test that matters is not that
a good report rewrites the file, but that a bad one leaves it exactly as it was.
"""

import importlib.util
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

REPO = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "refresh_import_receipt", REPO / "scripts/refresh_import_receipt.py"
)
refresh = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(refresh)

GOOD = "a" * 64
OTHER = "b" * 64

STALE_REPORT = (
    "error: repository.payload_digest\n"
    f"  the standalone payload digest is stale: recorded `{OTHER}`, "
    f"expected `{GOOD}`\n"
)

FORBIDDEN_REPORT = (
    "error: repository.forbidden_reference\n"
    '  docs/plans/notes.md contains "/Users/someone/workspace"\n'
)

RECEIPT_BODY = (
    "# Import receipt\n\n"
    "## Standalone payload digest\n\n"
    f"`{OTHER}`\n"
)


class ExpectedDigestTests(unittest.TestCase):
    def test_it_reads_the_expected_digest_not_the_recorded_one(self):
        self.assertEqual(refresh.expected_digest(STALE_REPORT), GOOD)

    def test_a_failure_that_is_not_about_the_digest_is_refused(self):
        with self.assertRaises(SystemExit) as caught:
            refresh.expected_digest(FORBIDDEN_REPORT)
        self.assertIn("did not report a stale payload digest", str(caught.exception))

    def test_a_stale_report_with_no_digest_is_refused(self):
        with self.assertRaises(SystemExit):
            refresh.expected_digest(
                "error: the standalone payload digest is stale\n"
            )


class RewriteTests(unittest.TestCase):
    def setUp(self):
        self.tmp = TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.receipt = Path(self.tmp.name) / "import-receipt.md"
        self.receipt.write_text(RECEIPT_BODY)
        self._saved = refresh.RECEIPT
        refresh.RECEIPT = self.receipt
        self.addCleanup(lambda: setattr(refresh, "RECEIPT", self._saved))

    def test_the_pin_is_replaced_and_the_rest_of_the_receipt_survives(self):
        refresh.rewrite(GOOD)
        text = self.receipt.read_text()
        self.assertIn(f"`{GOOD}`", text)
        self.assertNotIn(OTHER, text)
        self.assertTrue(text.startswith("# Import receipt"))

    def test_a_receipt_with_no_pin_is_refused_rather_than_appended_to(self):
        self.receipt.write_text("# Import receipt\n\nno digest here\n")
        with self.assertRaises(SystemExit):
            refresh.rewrite(GOOD)
        self.assertNotIn(GOOD, self.receipt.read_text())

    def test_an_already_emptied_pin_is_repaired(self):
        """The state the broken run left behind must still be recoverable."""
        self.receipt.write_text("# Import receipt\n\n``\n")
        refresh.rewrite(GOOD)
        self.assertIn(f"`{GOOD}`", self.receipt.read_text())


if __name__ == "__main__":
    unittest.main()
