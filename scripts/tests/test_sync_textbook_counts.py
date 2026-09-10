"""Behavioral tests for the derived-count projection.

These exercise the projection itself against synthetic contracts, not the
repository's real surfaces: the point is that a contract change moves every
derived number, and that a surface whose prose has changed shape fails loudly
instead of being silently reformatted.
"""

import hashlib
import importlib.util
import json
import textwrap
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

REPO = Path(__file__).resolve().parents[2]
SPEC = importlib.util.spec_from_file_location(
    "sync_textbook_counts", REPO / "scripts/sync_textbook_counts.py"
)
sync = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(sync)


def card(chapter, index, *, lean, prose, mode, declared=True):
    row = {
        "item_id": f"CFT-{chapter:02}-{index:03}",
        "chapter": chapter,
        "lean_correspondence_status": lean,
        "prose_proof_status": prose,
        "formal_mode": mode,
    }
    if declared:
        row["lean_declaration"] = {"name": f"CrouzeixTextbook.card_{chapter}_{index}"}
    return row


def problem(chapter, index, *, solved):
    row = {"exercise_id": f"CFT-{chapter:02}-E{index:02}", "chapter": chapter}
    row["lean_solution"] = {"declaration": "x"} if solved else None
    return row


class TallyTests(unittest.TestCase):
    def test_pending_boundary_is_the_first_chapter_without_solutions(self):
        problems = [problem(1, i, solved=True) for i in range(1, 7)]
        problems += [problem(2, i, solved=False) for i in range(1, 7)]
        counts = sync.tally([], problems)
        self.assertEqual(counts["first_pending"], 2)
        self.assertEqual(counts["solved_chapters"], [1])
        self.assertEqual(counts["solved"], 6)
        self.assertEqual(counts["unsolved"], 6)

    def test_counts_follow_the_contract_rather_than_a_pinned_literal(self):
        cards = [
            card(1, 1, lean="exact", prose="reconstructible", mode="proved-here"),
            card(1, 2, lean="checkpoint", prose="summary", mode="checkpoint"),
            card(1, 3, lean="unmapped", prose="summary", mode="reexported-proof"),
        ]
        counts = sync.tally(cards, [])
        self.assertEqual(counts["exact"], 1)
        self.assertEqual(counts["checkpoint"], 1)
        self.assertEqual(counts["unmapped"], 1)
        self.assertEqual(counts["summary"], 2)
        self.assertEqual(counts["reconstructible"], 1)


class InventoryTotalsTests(unittest.TestCase):
    """The regression this projection was written to catch."""

    def test_totals_exclude_chapters_past_the_backlog_span(self):
        # Chapter 28 is complete apart from one summary row. A total taken over
        # "every chapter from the boundary onward" sweeps it in; the backlog
        # spans chapters 10 through 24 only.
        cards = [card(10, i, lean="unmapped", prose="summary", mode="checkpoint") for i in range(1, 7)]
        cards += [card(28, 6, lean="exact", prose="summary", mode="proved-here")]
        problems = [problem(10, i, solved=False) for i in range(1, 7)]
        problems += [problem(28, 6, solved=True)]
        counts = sync.tally(cards, problems)
        table = sync.inventory_tables(cards, problems, counts)
        totals = [line for line in table.splitlines() if line.startswith("| **")]
        self.assertEqual(len(totals), 1)
        # summary column: six from chapter 10, and NOT chapter 28's row.
        self.assertIn("**6**", totals[0])
        self.assertNotIn("**7**", totals[0])


class SubstitutionTests(unittest.TestCase):
    def test_a_reshaped_surface_fails_loudly(self):
        with self.assertRaises(sync.SurfaceError) as caught:
            sync.substitute("nothing to match here", r"- Exact-correspondence rows: \d+\.", "x", "demo")
        self.assertIn("changed shape", str(caught.exception))

    def test_an_ambiguous_surface_is_not_silently_half_updated(self):
        # Two matches means the projection cannot tell which region is derived.
        text = "- Exact-correspondence rows: 1.\n- Exact-correspondence rows: 2.\n"
        with self.assertRaises(sync.SurfaceError):
            sync.substitute(text, r"- Exact-correspondence rows: \d+\.", "x", "demo")


class BulletIdempotenceTests(unittest.TestCase):
    """The projection must be able to re-read what it just wrote."""

    def test_the_bullet_pattern_survives_every_wrap_point(self):
        # The wrap point moves as chapters are added, so a pattern pinned to a
        # particular line break matches its own output only by luck.
        pattern = r"- Distinct checked exercise solutions:[\s\S]*?solution yet\."
        for counts in (
            {"solved_chapters": [1], "unsolved": 210},
            {"solved_chapters": list(range(1, 12)), "unsolved": 150},
            {"solved_chapters": list(range(1, 31)), "unsolved": 36},
        ):
            bullet = sync.solved_chapters_bullet(counts)
            document = f"- Indexed exercises: 216.\n{bullet}\n- Active Lean target: `x`.\n"
            rewritten = sync.substitute(document, pattern, lambda _m: bullet, "bullet")
            self.assertEqual(rewritten, document)


class ReceiptIdentityTests(unittest.TestCase):
    def test_identity_is_the_digest_of_the_exact_bytes(self):
        with TemporaryDirectory() as tmp:
            path = Path(tmp) / "receipt.json"
            payload = json.dumps({"declarations": [{"name": "a"}, {"name": "b"}]}).encode()
            path.write_bytes(payload)
            identity = sync.receipt_identity(path)
            self.assertEqual(identity["sha256"], hashlib.sha256(payload).hexdigest())
            self.assertEqual(identity["bytes"], len(payload))
            self.assertEqual(identity["declarations"], 2)


class SolvedChaptersBulletTests(unittest.TestCase):
    def test_bullet_wraps_and_reports_the_remainder(self):
        counts = {"solved_chapters": [1, 2, 3], "unsolved": 198}
        bullet = sync.solved_chapters_bullet(counts)
        collapsed = " ".join(bullet.split())
        self.assertIn("six each in Chapters 1, 2, and 3.", collapsed)
        self.assertIn("The other 198 exercise rows", collapsed)
        for line in bullet.splitlines()[1:]:
            self.assertTrue(line.startswith("  "), line)


if __name__ == "__main__":
    unittest.main()
