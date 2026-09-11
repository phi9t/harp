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
import re
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
        row["lean_declaration"] = {
            "name": f"CrouzeixTextbook.card_{chapter}_{index}",
            "underlying_declaration": None,
            "type_sha256": f"{chapter:02}{index:02}" + "0" * 60,
        }
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


class ModePartitionTests(unittest.TestCase):
    """The four formal modes partition the roster, so the ledger sentence must add up.

    The definition count was a hardcoded word in the projector's replacement for
    as long as it happened to be six. When a seventh definition row landed, the
    published ledger kept claiming six and its four parts summed to 215 of 216
    rows, silently, because nothing compared the parts against the total.
    """

    def test_the_rendered_ledger_names_the_actual_definition_count(self):
        """This is the test that would have caught it: render, then read back."""
        cards = [card(1, 1, lean="exact", prose="reconstructible", mode="proved-here")]
        cards += [
            card(1, i, lean="exact", prose="reconstructible", mode="definition")
            for i in range(2, 11)
        ]
        counts = sync.tally(cards, [])
        text = (
            "| theorem rows | 216 |\n"
            "| summary prose rows | 65 |\n"
            "| reconstructible prose rows | 149 |\n"
            "| exact correspondence rows | 156 |\n"
            "| unmapped correspondence rows | 37 |\n"
            "| solved exercises | 156 |\n"
            "| unresolved exercises | 60 |\n"
            "\n"
            "- Statement: The version-two contract has 216 theorem rows: 69 are\n"
            "  `proved-here`, 115 are `reexported-proof`, 23 are `checkpoint`, and six are\n"
            "  `definition`. A fresh 486-row Lean receipt checks the declarations required\n"
            "  by that contract under Lean 4.32.1. The contract has 156 distinct exercise\n"
            "  solutions; 60 exercises remain correspondence-incomplete. The theorem\n"
            "  correspondence axis records 156 exact rows, 23 checkpoints, and 37 unmapped\n"
            "  rows.\n"
        )
        identity = {"sha256": "0" * 64, "bytes": 1, "declarations": 486}
        rendered = sync.render_claim_ledger(text, counts, identity)
        self.assertIn("and 9 are\n  `definition`.", rendered)
        self.assertNotIn("and six are", rendered)
        parts = re.search(
            r"has (\d+) theorem rows: (\d+) are\n  `proved-here`, (\d+) are "
            r"`reexported-proof`, (\d+) are `checkpoint`, and (\d+) are",
            rendered,
        )
        total, *modes = (int(g) for g in parts.groups())
        self.assertEqual(sum(modes), total)

    def test_the_four_modes_sum_to_the_card_count(self):
        cards = [
            card(1, 1, lean="exact", prose="reconstructible", mode="proved-here"),
            card(1, 2, lean="exact", prose="reconstructible", mode="reexported-proof"),
            card(1, 3, lean="checkpoint", prose="summary", mode="checkpoint"),
            card(1, 4, lean="exact", prose="reconstructible", mode="definition"),
            card(1, 5, lean="exact", prose="reconstructible", mode="definition"),
        ]
        counts = sync.tally(cards, [])
        parts = (
            counts["proved_here"]
            + counts["reexported"]
            + counts["mode_checkpoint"]
            + counts["definition"]
        )
        self.assertEqual(parts, counts["cards"])
        self.assertEqual(counts["definition"], 2)


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


class ProviderIdentityTests(unittest.TestCase):
    """The identity key is the provider, and the type fingerprint is not.

    Both failure modes below are present in the real roster, so keying on the
    fingerprint would be wrong in both directions at once.
    """

    def test_two_aliases_of_one_theorem_count_once_despite_differing_fingerprints(self):
        # A pretty-printed type carries universe metavariable names, so two
        # aliases of the same theorem hash differently. They are still one proof.
        alias_a = card(6, 1, lean="exact", prose="reconstructible", mode="reexported-proof")
        alias_a["lean_declaration"]["underlying_declaration"] = "Lib.theorem_x"
        alias_a["lean_declaration"]["type_sha256"] = "a" * 64
        alias_b = card(19, 6, lean="unmapped", prose="summary", mode="reexported-proof")
        alias_b["lean_declaration"]["underlying_declaration"] = "Lib.theorem_x"
        alias_b["lean_declaration"]["type_sha256"] = "b" * 64
        counts = sync.tally([alias_a, alias_b], [])
        self.assertEqual(counts["distinct_proofs"], 1)
        self.assertEqual(counts["restating_cards"], 1)
        self.assertEqual(list(sync.shared_providers([alias_a, alias_b])), ["Lib.theorem_x"])

    def test_independent_proofs_of_one_statement_count_separately(self):
        # Two routes reaching the same theorem share a type. Reporting them as
        # duplication would invert the book's independence claim.
        route_a = card(35, 5, lean="exact", prose="reconstructible", mode="proved-here")
        route_b = card(36, 6, lean="exact", prose="reconstructible", mode="proved-here")
        for row, fingerprint in ((route_a, "c" * 64), (route_b, "c" * 64)):
            row["lean_declaration"]["type_sha256"] = fingerprint
            row["lean_declaration"]["underlying_declaration"] = None
        counts = sync.tally([route_a, route_b], [])
        self.assertEqual(counts["distinct_proofs"], 2)
        self.assertEqual(counts["restating_cards"], 0)
        self.assertEqual(sync.shared_providers([route_a, route_b]), {})


class CompletedPrefixTests(unittest.TestCase):
    def test_prefix_stops_at_the_pending_boundary(self):
        cards = [card(1, 1, lean="exact", prose="reconstructible", mode="proved-here")]
        cards += [card(3, 1, lean="exact", prose="reconstructible", mode="proved-here")]
        problems = [problem(1, 1, solved=True), problem(3, 1, solved=True)]
        counts = sync.tally(cards, problems)
        # Chapter 2 has no solutions, so the completed prefix is Chapter 1 alone
        # and Chapter 3's exact row must not be counted into it.
        self.assertEqual(counts["first_pending"], 2)
        self.assertEqual(counts["prefix_last"], 1)
        self.assertEqual(counts["prefix_exact"], 1)
        self.assertEqual(counts["prefix_solved"], 1)


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
