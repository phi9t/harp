from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scripts import migrate_obsidian_links as migration


class ObsidianLinkMigrationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary_directory = tempfile.TemporaryDirectory()
        self.repo_root = Path(self.temporary_directory.name)
        self.knowledge = self.repo_root / "knowledge"
        self.evidence = self.repo_root / "evidence"
        self.knowledge.mkdir()
        self.evidence.mkdir()
        (self.knowledge / "b.md").write_text(
            "# Supporting note\n\n## Result boundary\n\nResult details.\n",
            encoding="utf-8",
        )
        (self.evidence / "source.txt").write_text(
            "line one\nline two\nline three\n",
            encoding="utf-8",
        )
        (self.evidence / "paper.pdf").write_bytes(b"%PDF-1.7\n")
        captured = self.evidence / "artifacts" / "capture.md"
        captured.parent.mkdir()
        captured.write_text("[Captured](../../knowledge/b.md)\n", encoding="utf-8")

    def tearDown(self) -> None:
        self.temporary_directory.cleanup()

    def write_note(self, relative_path: str, text: str) -> Path:
        note = self.knowledge / relative_path
        note.parent.mkdir(parents=True, exist_ok=True)
        note.write_text(text, encoding="utf-8")
        return note

    def run_cli(self, *arguments: str) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [
                sys.executable,
                "scripts/migrate_obsidian_links.py",
                *arguments,
                "--repo-root",
                str(self.repo_root),
            ],
            cwd=Path(__file__).resolve().parents[2],
            check=False,
            capture_output=True,
            text=True,
        )

    def test_migrates_note_and_exact_line_locator_without_touching_code(self) -> None:
        source = self.write_note(
            "a.md",
            "[Result](b.md#result-boundary)\n"
            "[source](../evidence/source.txt#L2-L3)\n"
            "`[Code](b.md)`\n"
            "```md\n"
            "[Code](b.md)\n"
            "```\n",
        )

        migrated = migration.migrate_text(
            source,
            source.read_text(encoding="utf-8"),
            self.repo_root,
        )

        self.assertEqual(
            migrated,
            "[[knowledge/b#Result boundary|Result]]\n"
            "[[evidence/source.txt|source]] ([exact lines 2–3](../evidence/source.txt#L2-L3))\n"
            "`[Code](b.md)`\n"
            "```md\n"
            "[Code](b.md)\n"
            "```\n",
        )

    def test_migrates_root_relative_note_anchor_and_pdf_artifact(self) -> None:
        source = self.write_note(
            "nested/a.md",
            "[Root](../b.md)\n"
            "[Same](#local-anchor)\n"
            "[PDF](../../evidence/paper.pdf)\n"
            "[Outside](https://example.test/reference)\n"
            "[[knowledge/b|Existing wiki]]\n"
            "## Local anchor\n",
        )

        migrated = migration.migrate_text(
            source,
            source.read_text(encoding="utf-8"),
            self.repo_root,
        )

        self.assertEqual(
            migrated,
            "[[knowledge/b|Root]]\n"
            "[[#Local anchor|Same]]\n"
            "[[evidence/paper.pdf|PDF]]\n"
            "[Outside](https://example.test/reference)\n"
            "[[knowledge/b|Existing wiki]]\n"
            "## Local anchor\n",
        )

    def test_rejects_missing_source_before_writing_any_file(self) -> None:
        source = self.write_note("a.md", "[Missing](missing.md)\n")
        original = source.read_text(encoding="utf-8")

        result = self.run_cli("--write")

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("missing.md", result.stderr)
        self.assertEqual(source.read_text(encoding="utf-8"), original)

    def test_rejects_ambiguous_heading(self) -> None:
        (self.knowledge / "b.md").write_text(
            "## Result boundary\n\nOne.\n\n## Result boundary\n\nTwo.\n",
            encoding="utf-8",
        )
        source = self.write_note("a.md", "[Result](b.md#result-boundary)\n")

        with self.assertRaisesRegex(migration.MigrationError, "ambiguous heading"):
            migration.migrate_text(
                source,
                source.read_text(encoding="utf-8"),
                self.repo_root,
            )

    def test_resolves_explicit_heading_id_to_visible_heading_title(self) -> None:
        (self.knowledge / "b.md").write_text(
            "## Result boundary {#stable-result-boundary}\n",
            encoding="utf-8",
        )
        source = self.write_note(
            "a.md", "[Result](b.md#stable-result-boundary)\n"
        )

        migrated = migration.migrate_text(
            source,
            source.read_text(encoding="utf-8"),
            self.repo_root,
        )

        self.assertEqual(migrated, "[[knowledge/b#Result boundary|Result]]\n")

    def test_migrates_safe_directory_target_to_its_regular_readme(self) -> None:
        crate = self.repo_root / "labs" / "sicp-evaluator"
        crate.mkdir(parents=True)
        (crate / "README.md").write_text("# SICP evaluator\n", encoding="utf-8")
        source = self.write_note(
            "rsi/sicp/course/seminars/a.md",
            "[SICP evaluator crate](../../../../../labs/sicp-evaluator)\n",
        )

        migrated = migration.migrate_text(
            source,
            source.read_text(encoding="utf-8"),
            self.repo_root,
        )

        self.assertEqual(
            migrated, "[[labs/sicp-evaluator/README|SICP evaluator crate]]\n"
        )

    def test_migrates_evidence_directory_to_its_regular_provenance(self) -> None:
        bundle = self.evidence / "sicp"
        bundle.mkdir()
        (bundle / "PROVENANCE.md").write_text("captured source\n", encoding="utf-8")
        source = self.write_note(
            "rsi/sicp/a.md", "[evidence/sicp/](../../../evidence/sicp)\n"
        )

        migrated = migration.migrate_text(
            source,
            source.read_text(encoding="utf-8"),
            self.repo_root,
        )

        self.assertEqual(migrated, "[[evidence/sicp/PROVENANCE|evidence/sicp/]]\n")

    def test_rejects_evidence_directory_without_safe_landing_file(self) -> None:
        bundle = self.evidence / "without-landing-file"
        bundle.mkdir()
        source = self.write_note(
            "a.md", "[evidence bundle](../evidence/without-landing-file)\n"
        )

        with self.assertRaisesRegex(migration.MigrationError, "missing local link target"):
            migration.migrate_text(
                source,
                source.read_text(encoding="utf-8"),
                self.repo_root,
            )

    def test_rejects_directory_without_regular_readme(self) -> None:
        directory = self.repo_root / "labs" / "without-readme"
        directory.mkdir(parents=True)
        source = self.write_note("a.md", "[Directory](../labs/without-readme)\n")

        with self.assertRaisesRegex(migration.MigrationError, "missing local link target"):
            migration.migrate_text(
                source,
                source.read_text(encoding="utf-8"),
                self.repo_root,
            )

    def test_rejects_directory_with_symlinked_readme(self) -> None:
        directory = self.repo_root / "labs" / "symlinked-readme"
        directory.mkdir(parents=True)
        target = self.repo_root / "README.md"
        target.write_text("# Outside directory\n", encoding="utf-8")
        (directory / "README.md").symlink_to(target)
        source = self.write_note("a.md", "[Directory](../labs/symlinked-readme)\n")

        with self.assertRaisesRegex(migration.MigrationError, "missing local link target"):
            migration.migrate_text(
                source,
                source.read_text(encoding="utf-8"),
                self.repo_root,
            )

    def test_discovery_excludes_captured_and_local_investigation_markdown(self) -> None:
        source = self.write_note("a.md", "[Result](b.md)\n")
        local_investigation = self.write_note(
            "investigations/local/scratch.md",
            "[Result](../../b.md)\n",
        )
        captured = self.evidence / "artifacts" / "capture.md"

        discovered = migration.discover_managed_notes(self.repo_root)

        self.assertEqual(discovered, [source, self.knowledge / "b.md"])
        self.assertNotIn(local_investigation, discovered)
        self.assertNotIn(captured, discovered)

    def test_cli_write_then_check_is_idempotent_and_reports_deterministically(self) -> None:
        source = self.write_note(
            "a.md",
            "[Result](b.md#result-boundary)\n"
            "[source](../evidence/source.txt#L2-L3)\n",
        )
        report_path = self.repo_root / "target" / "obsidian-link-migration.json"

        pending_check = self.run_cli("--check", "--report", str(report_path))

        self.assertNotEqual(pending_check.returncode, 0)
        self.assertEqual(
            json.loads(report_path.read_text(encoding="utf-8")),
            {
                "converted_count": 2,
                "input_count": 2,
                "remaining_managed_markdown_link_count": 0,
                "retained_line_locator_count": 1,
                "skipped_captured_count": 1,
            },
        )

        write = self.run_cli("--write", "--report", str(report_path))

        self.assertEqual(write.returncode, 0, write.stderr)
        self.assertEqual(
            source.read_text(encoding="utf-8"),
            "[[knowledge/b#Result boundary|Result]]\n"
            "[[evidence/source.txt|source]] ([exact lines 2–3](../evidence/source.txt#L2-L3))\n",
        )
        self.assertEqual(
            json.loads(report_path.read_text(encoding="utf-8")),
            {
                "converted_count": 2,
                "input_count": 2,
                "remaining_managed_markdown_link_count": 0,
                "retained_line_locator_count": 1,
                "skipped_captured_count": 1,
            },
        )

        check = self.run_cli("--check", "--report", str(report_path))

        self.assertEqual(check.returncode, 0, check.stderr)
        self.assertEqual(
            json.loads(report_path.read_text(encoding="utf-8")),
            {
                "converted_count": 0,
                "input_count": 2,
                "remaining_managed_markdown_link_count": 0,
                "retained_line_locator_count": 1,
                "skipped_captured_count": 1,
            },
        )

    def test_cli_rejects_input_outside_managed_knowledge_roots(self) -> None:
        outside = self.repo_root / "notes.md"
        outside.write_text("[Result](knowledge/b.md)\n", encoding="utf-8")

        result = self.run_cli("--check", str(outside))

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("outside managed knowledge roots", result.stderr)


if __name__ == "__main__":
    unittest.main()
