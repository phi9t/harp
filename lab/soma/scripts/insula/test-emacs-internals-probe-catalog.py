#!/usr/bin/env python3
from __future__ import annotations

import pathlib
import re
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
RUNNER = ROOT / "probes" / "emacs-internals" / "elisp" / "run-probes.el"
PROBE_DOC = ROOT / "docs" / "internals-atlas" / "probes" / "README.md"


class EmacsInternalsProbeCatalogTest(unittest.TestCase):
    def test_capf_exit_function_probe_is_registered_and_documented(self) -> None:
        runner = RUNNER.read_text(encoding="utf-8")
        probe_doc = PROBE_DOC.read_text(encoding="utf-8")

        self.assertIn('"capf-exit-function-acceptance"', runner)
        self.assertIn("soma-probe-capf-exit-function-acceptance", runner)
        self.assertRegex(
            runner,
            re.compile(
                r"soma-probe-capf-exclusivity\s+"
                r"soma-probe-capf-exit-function-acceptance",
                re.MULTILINE,
            ),
        )
        self.assertIn("`capf-exit-function-acceptance`", probe_doc)


if __name__ == "__main__":
    unittest.main()
