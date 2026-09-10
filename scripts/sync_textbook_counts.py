#!/usr/bin/env python3
"""Project the Crouzeix textbook contracts onto their derived reader surfaces.

The contract counts, the compiler-receipt identity and the remaining-work
inventory are derived data. They were previously maintained by hand across
roughly twenty-five edit sites in seven files, which is the largest mechanical
cost of landing a chapter and the likeliest source of silent drift.

This script computes every one of them from the two structured contracts and a
compiled receipt, then rewrites the surfaces. `--check` reports drift without
writing, which is what the repository gate runs.

It does not invent content: each rewrite is a targeted substitution inside a
sentence whose shape is fixed by the surrounding prose. A surface whose wording
has changed shape fails loudly rather than being silently reformatted.
"""

from __future__ import annotations

import argparse
import collections
import hashlib
import json
import re
import sys
import textwrap
from pathlib import Path

CHAPTERS = 36
PER_CHAPTER = 6
TOTAL = CHAPTERS * PER_CHAPTER

KATA = {
    7: "3ya3", 8: "tdga", 9: "wstr", 10: "0wc6", 11: "raf4", 12: "r60f",
    13: "fx6b", 14: "2700", 15: "hg37", 16: "92zd", 17: "q81q", 18: "dvm6",
    19: "1r2p", 20: "efcd", 21: "gkgr", 22: "17d3", 23: "0f59", 24: "nz6f",
}


class SurfaceError(RuntimeError):
    """A derived surface no longer has the shape this projection expects."""


def load(root: Path):
    coverage = json.loads((root / "content/crouzeix_textbook/coverage.json").read_text())
    exercises = json.loads((root / "content/crouzeix_textbook/exercises.json").read_text())
    return coverage["items"], exercises["exercises"]


def tally(cards, problems):
    lean = collections.Counter(row["lean_correspondence_status"] for row in cards)
    prose = collections.Counter(row["prose_proof_status"] for row in cards)
    mode = collections.Counter(row["formal_mode"] for row in cards)
    solved = sorted(
        {row["chapter"] for row in problems if row.get("lean_solution")}
    )
    counts = {
        "cards": len(cards),
        "exact": lean["exact"],
        "checkpoint": lean["checkpoint"],
        "unmapped": lean["unmapped"],
        "reconstructible": prose["reconstructible"],
        "summary": prose["summary"],
        "not_applicable": prose["not-applicable"],
        "proved_here": mode["proved-here"],
        "reexported": mode["reexported-proof"],
        "mode_checkpoint": mode["checkpoint"],
        "definition": mode["definition"],
        "solved": sum(1 for row in problems if row.get("lean_solution")),
        "solved_chapters": solved,
    }
    counts["unsolved"] = len(problems) - counts["solved"]
    # The first chapter with no checked solutions is the pending boundary.
    pending = [c for c in range(1, CHAPTERS + 1) if c not in solved]
    counts["first_pending"] = pending[0] if pending else CHAPTERS + 1
    return counts


def receipt_identity(path: Path):
    raw = path.read_bytes()
    return {
        "sha256": hashlib.sha256(raw).hexdigest(),
        "bytes": len(raw),
        "declarations": len(json.loads(raw)["declarations"]),
    }


def substitute(text: str, pattern: str, replacement, label: str) -> str:
    matches = len(re.findall(pattern, text))
    if matches != 1:
        raise SurfaceError(
            f"{label}: expected exactly one match for the derived region; found {matches}. "
            "The surrounding prose has changed shape, or the region is ambiguous; "
            "update this projection rather than letting it rewrite the wrong text."
        )
    return re.sub(pattern, replacement, text, count=1)


def solved_chapters_bullet(counts, *, width: int = 88) -> str:
    """Render the solved-chapter bullet, wrapped the way the surface wraps prose."""
    chapters = counts["solved_chapters"]
    listed = ", ".join(str(c) for c in chapters[:-1]) + f", and {chapters[-1]}"
    sentence = (
        "- Distinct checked exercise solutions: six each in Chapters "
        f"{listed}. The other {counts['unsolved']} exercise rows have no claimed "
        "formal solution yet."
    )
    return textwrap.fill(sentence, width=width, subsequent_indent="  ")


def snapshot_table(counts) -> str:
    return (
        f"| theorem rows | {counts['cards']} |\n"
        f"| summary prose rows | {counts['summary']} |\n"
        f"| reconstructible prose rows | {counts['reconstructible']} |\n"
        f"| exact correspondence rows | {counts['exact']} |\n"
        f"| unmapped correspondence rows | {counts['unmapped']} |\n"
        f"| solved exercises | {counts['solved']} |\n"
        f"| unresolved exercises | {counts['unsolved']} |"
    )


def render_snapshot(text: str, counts, label: str) -> str:
    return substitute(
        text,
        r"\| theorem rows \| \d+ \|\n(?:\| [a-z ]+ \| \d+ \|\n){5}\| unresolved exercises \| \d+ \|",
        lambda _m: snapshot_table(counts),
        label,
    )


def render_status(text: str, counts, identity) -> str:
    text = render_snapshot(text, counts, "status_and_scope contract snapshot")
    text = substitute(
        text,
        r"- Proof-exposition status: \d+ coverage rows are now `reconstructible`; \d+ remain",
        lambda _m: (
            f"- Proof-exposition status: {counts['reconstructible']} coverage rows are now "
            f"`reconstructible`; {counts['summary']} remain"
        ),
        "status_and_scope proof-exposition line",
    )
    text = substitute(
        text,
        r"- Coverage rows: \d+, comprising \d+ `proved-here`, \d+\n  `reexported-proof`, \d+ `checkpoint`, and \d+ `definition` rows\.",
        lambda _m: (
            f"- Coverage rows: {counts['cards']}, comprising {counts['proved_here']} `proved-here`, "
            f"{counts['reexported']}\n  `reexported-proof`, {counts['mode_checkpoint']} `checkpoint`, "
            f"and {counts['definition']} `definition` rows."
        ),
        "status_and_scope coverage-rows line",
    )
    text = substitute(
        text,
        r"- Exact-correspondence rows: \d+\.",
        lambda _m: f"- Exact-correspondence rows: {counts['exact']}.",
        "status_and_scope exact-correspondence line",
    )
    text = substitute(
        text,
        # The wrap point moves as chapters are added, so match the whole bullet
        # rather than a fixed line break. Non-greedy to the first terminator.
        r"- Distinct checked exercise solutions:[\s\S]*?solution yet\.",
        lambda _m: solved_chapters_bullet(counts),
        "status_and_scope solved-chapters line",
    )
    text = substitute(
        text,
        r"Chapters \d+–24 still hold \d+ unsolved exercises and \d+ rows whose",
        lambda _m: (
            f"Chapters {counts['first_pending']}–24 still hold {counts['unsolved']} unsolved "
            f"exercises and {counts['unsolved']} rows whose"
        ),
        "status_and_scope remaining-work sentence",
    )
    text = substitute(
        text,
        r"- Compiler receipt SHA-256:\n  `[0-9a-f]{64}`\n- Compiler receipt bytes: `\d+`\n- Compiler receipt declarations: `\d+`",
        lambda _m: (
            f"- Compiler receipt SHA-256:\n  `{identity['sha256']}`\n"
            f"- Compiler receipt bytes: `{identity['bytes']}`\n"
            f"- Compiler receipt declarations: `{identity['declarations']}`"
        ),
        "status_and_scope receipt identity",
    )
    providers = identity["declarations"] - counts["cards"] - counts["solved"]
    text = substitute(
        text,
        r"Those \d+ unique declarations comprise \d+ public coverage declarations,\n\d+ distinct exercise solutions, and the \d+ additional underlying proof",
        lambda _m: (
            f"Those {identity['declarations']} unique declarations comprise {counts['cards']} "
            f"public coverage declarations,\n{counts['solved']} distinct exercise solutions, and the "
            f"{providers} additional underlying proof"
        ),
        "status_and_scope declaration decomposition",
    )
    return text


def render_claim_ledger(text: str, counts, identity) -> str:
    text = render_snapshot(text, counts, "claim_evidence_ledger contract snapshot")
    return substitute(
        text,
        r"- Statement: The version-two contract has \d+ theorem rows: \d+ are\n  `proved-here`, \d+ are `reexported-proof`, \d+ are `checkpoint`, and six are\n  `definition`\. A fresh \d+-row Lean receipt checks the declarations required\n  by that contract under Lean 4\.32\.1\. The contract has \d+ distinct exercise\n  solutions; \d+ exercises remain correspondence-incomplete\. The theorem\n  correspondence axis records \d+ exact rows, \d+ checkpoints, and \d+ unmapped\n  rows\.",
        lambda _m: (
            f"- Statement: The version-two contract has {counts['cards']} theorem rows: "
            f"{counts['proved_here']} are\n  `proved-here`, {counts['reexported']} are "
            f"`reexported-proof`, {counts['mode_checkpoint']} are `checkpoint`, and six are\n"
            f"  `definition`. A fresh {identity['declarations']}-row Lean receipt checks the "
            f"declarations required\n  by that contract under Lean 4.32.1. The contract has "
            f"{counts['solved']} distinct exercise\n  solutions; {counts['unsolved']} exercises "
            f"remain correspondence-incomplete. The theorem\n  correspondence axis records "
            f"{counts['exact']} exact rows, {counts['mode_checkpoint']} checkpoints, and "
            f"{counts['unmapped']} unmapped\n  rows."
        ),
        "claim_evidence_ledger CFT-CL-002 statement",
    )


def render_chapter_33(text: str, identity) -> str:
    return substitute(
        text,
        r"- Compiler receipt SHA-256:\n  `[0-9a-f]{64}`;\n- serialized bytes: `\d+`; and\n- declaration count: `\d+`\.",
        lambda _m: (
            f"- Compiler receipt SHA-256:\n  `{identity['sha256']}`;\n"
            f"- serialized bytes: `{identity['bytes']}`; and\n"
            f"- declaration count: `{identity['declarations']}`."
        ),
        "chapter 33 receipt identity",
    )


def inventory_tables(cards, problems, counts) -> str:
    first = counts["first_pending"]
    by_card = collections.defaultdict(list)
    by_problem = collections.defaultdict(list)
    for row in cards:
        by_card[row["chapter"]].append(row)
    for row in problems:
        by_problem[row["chapter"]].append(row)

    lines = [
        "| Ch | Kata | cards `checkpoint` | cards `unmapped` | summary proofs | "
        "unsolved exercises | declared providers already named |",
        "| ---: | --- | ---: | ---: | ---: | ---: | ---: |",
    ]
    for chapter in range(first, 25):
        chapter_cards = by_card[chapter]
        lean = collections.Counter(r["lean_correspondence_status"] for r in chapter_cards)
        prose = collections.Counter(r["prose_proof_status"] for r in chapter_cards)
        unsolved = sum(1 for r in by_problem[chapter] if not r.get("lean_solution"))
        named = sum(1 for r in chapter_cards if r.get("lean_declaration"))
        lines.append(
            f"| {chapter} | `{KATA.get(chapter, chr(8212))}` | {lean['checkpoint']} | {lean['unmapped']} | "
            f"{prose['summary']} | {unsolved} | {named} |"
        )
    span = range(first, 25)
    totals = {
        "checkpoint": sum(1 for r in cards if r["chapter"] in span and r["lean_correspondence_status"] == "checkpoint"),
        "unmapped": sum(1 for r in cards if r["chapter"] in span and r["lean_correspondence_status"] == "unmapped"),
        "summary": sum(1 for r in cards if r["chapter"] in span and r["prose_proof_status"] == "summary"),
        "unsolved": sum(1 for r in problems if r["chapter"] in span and not r.get("lean_solution")),
        "named": sum(1 for r in cards if r["chapter"] in span and r.get("lean_declaration")),
    }
    lines.append(
        f"| **{first}–24** | | **{totals['checkpoint']}** | **{totals['unmapped']}** | "
        f"**{totals['summary']}** | **{totals['unsolved']}** | **{totals['named']}** |"
    )
    lines += [
        "",
        f"### Per-identity backlog, Chapters {first}–24",
        "",
        "| ID | correspondence | proof | formal mode | named declaration | exercise |",
        "| --- | --- | --- | --- | --- | --- |",
    ]
    for chapter in range(first, 25):
        for row in by_card[chapter]:
            declaration = row.get("lean_declaration")
            name = declaration["name"].replace("CrouzeixTextbook.", "") if declaration else "—"
            index = int(row["item_id"].split("-")[-1])
            exercise_id = f"CFT-{chapter:02}-E{index:02}"
            exercise = next(r for r in by_problem[chapter] if r["exercise_id"] == exercise_id)
            state = "solved" if exercise.get("lean_solution") else "unsolved"
            lines.append(
                f"| {row['item_id']} | {row['lean_correspondence_status']} | "
                f"{row['prose_proof_status']} | {row['formal_mode']} | `{name}` | "
                f"{exercise_id} {state} |"
            )
    return "\n".join(lines) + "\n"


def render_inventory(text: str, cards, problems, counts) -> str:
    first = counts["first_pending"]
    heading = re.search(r"## Chapters \d+–24: the exact remaining backlog", text)
    if heading is None:
        raise SurfaceError("backlog inventory: remaining-backlog heading is missing")
    ownership = text.index("## Ownership")
    body = inventory_tables(cards, problems, counts)
    return (
        text[: heading.start()]
        + f"## Chapters {first}–24: the exact remaining backlog\n\n"
        + body
        + "\n"
        + text[ownership:]
    )


SURFACES = (
    ("knowledge/crouzeix_textbook/status_and_scope.md", "status"),
    ("knowledge/crouzeix_textbook/claim_evidence_ledger.md", "claim"),
    (
        "knowledge/crouzeix_textbook/part_06_constant_two_routes/"
        "35_comparison_verification_and_boundaries.md",
        "snapshot",
    ),
    (
        "knowledge/crouzeix_textbook/part_06_constant_two_routes/"
        "33_lorist_schwenninger_perturbation_lemma.md",
        "identity",
    ),
    ("docs/workstream/harp-mathematics/textbook-backlog-inventory.md", "inventory"),
)


def project(root: Path, receipt: Path):
    cards, problems = load(root)
    counts = tally(cards, problems)
    identity = receipt_identity(receipt)
    rendered = {}
    for relative, kind in SURFACES:
        path = root / relative
        text = path.read_text()
        if kind == "status":
            text = render_status(text, counts, identity)
        elif kind == "claim":
            text = render_claim_ledger(text, counts, identity)
        elif kind == "snapshot":
            text = render_snapshot(text, counts, relative)
        elif kind == "identity":
            text = render_chapter_33(text, identity)
        elif kind == "inventory":
            text = render_inventory(text, cards, problems, counts)
        rendered[path] = text
    return counts, identity, rendered


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".", type=Path)
    parser.add_argument("--receipt", required=True, type=Path)
    parser.add_argument("--check", action="store_true", help="report drift without writing")
    args = parser.parse_args(argv)

    try:
        counts, identity, rendered = project(args.root.resolve(), args.receipt)
    except SurfaceError as error:
        print(f"textbook count projection failed: {error}", file=sys.stderr)
        return 2

    stale = [path for path, text in rendered.items() if path.read_text() != text]
    if args.check:
        for path in stale:
            print(f"stale derived surface: {path}", file=sys.stderr)
        if stale:
            print(
                "run `mise run textbook-counts` to reproject them from the contracts",
                file=sys.stderr,
            )
            return 1
        print(
            f"derived surfaces match the contracts: {counts['exact']} exact, "
            f"{counts['solved']} solved, {identity['declarations']} declarations"
        )
        return 0

    for path in stale:
        path.write_text(rendered[path])
        print(f"reprojected {path}")
    if not stale:
        print("derived surfaces already match the contracts")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
