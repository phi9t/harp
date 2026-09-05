import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


SCRIPT = Path(__file__).resolve().parents[1] / "prepare_pages.py"


class PreparePagesTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.dist = self.root / "atlas/dist"
        self.dist.mkdir(parents=True)
        self.html = b"<!doctype html><title>Harp Atlas</title>"
        self.corpus = b'{"documents":[]}'
        self.receipt = {
            "schema_version": "harp-atlas-export/v1",
            "html_sha256": hashlib.sha256(self.html).hexdigest(),
            "corpus_sha256": hashlib.sha256(self.corpus).hexdigest(),
            "app_inputs_sha256": "a" * 64,
        }
        (self.dist / "harp-atlas.html").write_bytes(self.html)
        corpus_path = self.root / "atlas/src/content/generated/corpus.json"
        corpus_path.parent.mkdir(parents=True)
        corpus_path.write_bytes(self.corpus)
        self.write_receipt()
        self.output = self.root / ".build/pages"

    def write_receipt(self):
        (self.dist / "harp-atlas.receipt.json").write_text(
            json.dumps(self.receipt) + "\n", encoding="utf-8"
        )

    def run_command(self):
        return subprocess.run(
            [sys.executable, str(SCRIPT)], cwd=self.root,
            capture_output=True, text=True, check=False,
        )

    def test_publishes_only_reader_and_receipt_with_original_bytes(self):
        (self.dist / "private.txt").write_text("do not publish", encoding="utf-8")
        result = self.run_command()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(
            sorted(p.name for p in self.output.iterdir()),
            ["harp-atlas.receipt.json", "index.html"],
        )
        self.assertEqual((self.output / "index.html").read_bytes(), self.html)
        self.assertEqual(
            (self.output / "harp-atlas.receipt.json").read_bytes(),
            (self.dist / "harp-atlas.receipt.json").read_bytes(),
        )

    def test_rejects_stale_html_or_corpus_before_writing(self):
        for field in ("html_sha256", "corpus_sha256"):
            with self.subTest(field=field):
                old = self.receipt[field]
                self.receipt[field] = "0" * 64
                self.write_receipt()
                result = self.run_command()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("digest mismatch", result.stderr)
                self.assertFalse(self.output.exists())
                self.receipt[field] = old

    def test_rejects_invalid_receipt_before_writing(self):
        for value in ([], {"schema_version": "unknown"}, {**self.receipt, "html_sha256": 1}):
            with self.subTest(value=value):
                (self.dist / "harp-atlas.receipt.json").write_text(
                    json.dumps(value), encoding="utf-8"
                )
                result = self.run_command()
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("invalid export receipt", result.stderr)
                self.assertFalse(self.output.exists())

    def test_rejects_symlinked_input(self):
        source = self.dist / "harp-atlas.html"
        saved = self.dist / "original.html"
        source.rename(saved)
        source.symlink_to(saved)
        result = self.run_command()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("symlink", result.stderr)
        self.assertFalse(self.output.exists())

    def test_preserves_existing_output(self):
        self.output.mkdir(parents=True)
        sentinel = self.output / "keep"
        sentinel.write_text("untouched", encoding="utf-8")
        result = self.run_command()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("already exists", result.stderr)
        self.assertEqual(sentinel.read_text(), "untouched")
        self.assertEqual(list(self.output.iterdir()), [sentinel])

    def test_rejects_symlinked_output_parent(self):
        outside = self.root / "outside"
        outside.mkdir()
        (self.root / ".build").symlink_to(outside, target_is_directory=True)
        result = self.run_command()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("symlink", result.stderr)
        self.assertEqual(list(outside.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
