"""claims.py against throwaway reports and git repos."""

from __future__ import annotations

import hashlib
import io
import json
import subprocess
import sys
import tempfile
import textwrap
import unittest
from contextlib import redirect_stderr, redirect_stdout
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import claims  # noqa: E402


def git(cwd, *args):
    return subprocess.run(["git", "-C", str(cwd), *args], check=True, capture_output=True, text=True).stdout


class ClaimsTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Path(self.tmp.name) / "repo"
        self.repo.mkdir()
        git(self.repo, "init", "-q", "-b", "main")
        git(self.repo, "config", "user.email", "a@b")
        git(self.repo, "config", "user.name", "a")
        (self.repo / "source.txt").write_text("alpha\nbravo\ncharlie\n")
        (self.repo / "notes.md").write_text("one\ntwo\nthree\n")
        git(self.repo, "add", ".")
        git(self.repo, "commit", "-qm", "init")
        self.sha = git(self.repo, "rev-parse", "HEAD").strip()
        self.report = self.repo / "report.md"
        self.report.write_text(
            textwrap.dedent(
                f"""\
                # Trial 1 excerpt

                1. [R] The source file's second line is bravo. Evidence: `source.txt:2`.
                - [X] The committed source file can be read with git show. Evidence: `git -C {self.repo} show {self.sha}:source.txt`.
                * [I] The next phase should keep a lead verification sample.
                2) [R] The notes file starts with one. Evidence: `notes.md:1`.
                - [X] The fixture has an executed uncited check.
                """
            )
        )

    def tearDown(self):
        sys.modules.pop("evidence", None)
        self.tmp.cleanup()

    def call(self, argv):
        out, err = io.StringIO(), io.StringIO()
        with redirect_stdout(out), redirect_stderr(err):
            self.code = claims.main(argv)
        self.err = err.getvalue()
        return out.getvalue()

    def json_lines(self, text):
        return [json.loads(line) for line in text.splitlines() if line.strip()]

    def test_trial_1_report_format_with_tags_and_citation_forms(self):
        parsed = claims.parse_claims(self.report)
        self.assertEqual([claim.tag for claim in parsed], ["R", "X", "I", "R", "X"])
        self.assertEqual(parsed[0].command, "sed -n '2,2p' source.txt")
        self.assertEqual(parsed[0].status, "CITED")
        self.assertEqual(parsed[1].command, f"git -C {self.repo} show {self.sha}:source.txt")
        self.assertEqual(parsed[2].status, "UNCITED")
        out = self.call(["sample", str(self.report), "--n", "10", "--seed", "trial", "--tags", "R,X"])
        sampled = self.json_lines(out)
        self.assertEqual(self.code, claims.OK, self.err)
        self.assertEqual({row["tag"] for row in sampled}, {"R", "X"})
        self.assertEqual(len(sampled), 4)

    def test_sampling_is_deterministic_seed_sensitive_bounded_and_marks_uncited(self):
        first = self.json_lines(self.call(["sample", str(self.report), "--n", "2", "--seed", "one"]))
        self.assertEqual(self.code, claims.OK, self.err)
        second = self.json_lines(self.call(["sample", str(self.report), "--n", "2", "--seed", "one"]))
        self.assertEqual([row["id"] for row in first], [row["id"] for row in second])
        # Seed sensitivity: the sampling key hashes the seed with the report digest, and the report here contains
        # a per-run temp path, so any two seeds may order a 2-of-5 sample identically (about 1 in 10 runs). Compare
        # the full order under several seeds instead; all of them agreeing by chance is about 1 in 120**5.
        def order(seed):
            return [row["id"] for row in self.json_lines(self.call(["sample", str(self.report), "--n", "99", "--seed", seed]))]
        base = order("one")
        self.assertTrue(any(order(seed) != base for seed in ("two", "three", "four", "five", "six")))
        all_rows = self.json_lines(self.call(["sample", str(self.report), "--n", "99", "--seed", "one"]))
        self.assertEqual(len(all_rows), 5)
        uncited = [row for row in all_rows if row["status"] == "UNCITED"]
        self.assertEqual(len(uncited), 2)
        self.assertTrue(all(row["command"] is None for row in uncited))

    def test_record_writes_jsonl_and_calls_evidence_with_output_digest(self):
        stub_dir = Path(self.tmp.name) / "stub"
        stub_dir.mkdir()
        (stub_dir / "evidence.py").write_text(
            "calls = []\n"
            "def append_claim_outcome(digest, claim_id, passed, command, output_digest, out_dir=None):\n"
            "    calls.append({'digest': digest, 'claim_id': claim_id, 'passed': passed, "
            "'command': command, 'output_digest': output_digest, 'out_dir': out_dir})\n"
        )
        sys.path.insert(0, str(stub_dir))
        self.addCleanup(lambda: sys.path.remove(str(stub_dir)) if str(stub_dir) in sys.path else None)
        sys.modules.pop("evidence", None)
        claim = claims.parse_claims(self.report)[0]
        output = self.repo / "claim.out"
        output.write_text("bravo\n")
        expected_digest = "sha256:" + hashlib.sha256(output.read_bytes()).hexdigest()
        out = self.call(
            [
                "record",
                str(self.report),
                "--claim",
                claim.claim_id,
                "--passed",
                "--output",
                str(output),
                "--evidence",
                "sha256:run",
            ]
        )
        self.assertEqual(self.code, claims.OK, self.err)
        printed = json.loads(out)
        outcome_file = Path(printed["recorded"])
        rows = [json.loads(line) for line in outcome_file.read_text().splitlines()]
        self.assertEqual(len(rows), 1)
        self.assertEqual(rows[0]["claim_id"], claim.claim_id)
        self.assertTrue(rows[0]["passed"])
        self.assertEqual(rows[0]["output_digest"], expected_digest)
        evidence = sys.modules["evidence"]
        self.assertEqual(
            evidence.calls,
            [
                {
                    "digest": "sha256:run",
                    "claim_id": claim.claim_id,
                    "passed": True,
                    "command": "sed -n '2,2p' source.txt",
                    "output_digest": expected_digest,
                    "out_dir": None,
                }
            ],
        )

    def test_missing_evidence_module_with_evidence_fails_clearly(self):
        claim = claims.parse_claims(self.report)[0]
        output = self.repo / "claim.out"
        output.write_text("bravo\n")
        original = claims.importlib.import_module

        def missing(name):
            if name == "evidence":
                raise ModuleNotFoundError("No module named 'evidence'")
            return original(name)

        claims.importlib.import_module = missing
        self.addCleanup(lambda: setattr(claims.importlib, "import_module", original))
        self.call(
            [
                "record",
                str(self.report),
                "--claim",
                claim.claim_id,
                "--passed",
                "--output",
                str(output),
                "--evidence",
                "sha256:run",
            ]
        )
        self.assertEqual(self.code, claims.ERROR)
        self.assertIn("evidence module is required", self.err)

    def test_path_line_command_reproduces_cited_line(self):
        rows = self.json_lines(self.call(["sample", str(self.report), "--n", "99", "--seed", "line"]))
        row = next(item for item in rows if item["citation"] == "source.txt:2")
        result = subprocess.run(row["command"], cwd=self.repo, shell=True, capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "bravo\n")


if __name__ == "__main__":
    unittest.main()
