#!/usr/bin/env python3
"""Pre-review checks for a textbook chapter, run before dispatching reviewers.

Three defect classes have recurred across the independent review passes, all of
them mechanical and all of them missed by the author who introduced them. This
script checks for them so the checking does not depend on remembering to check.

  1. Unresolved cross-references: a `[[wikilink]]` whose target does not exist.
     `harp build` catches these, but only after the prose is otherwise complete.

     A companion check on "Chapter NN does X" claims was written and removed. It
     compared the phrase against the chapter's index title by word overlap and
     produced sixteen false positives on chapter 15 while catching none of its
     real defects; a reference like "Chapter 6's simple-spectrum density" is
     perfectly good against a chapter titled "Eigenvalues and polynomial algebra".
     A check that noisy trains its reader to skip it. Whether a cross-chapter
     claim is apt stays a reviewer's job.

  2. Stale registry rows: contract rows byte-identical to the committed version
     while the prose was rewritten. This produced four wrong `skills` rows in
     Chapter 14, carried over verbatim from a withdrawn sketch. Reported as items
     to confirm rather than as failures, because an unchanged row is often
     correct — chapter 15's prerequisites were right and were deliberately kept.

  3. Prose mode claims. Every sentence naming a formal mode is printed beside the
     registry's actual tally, for comparison. Chapter 12 shipped three sentences
     calling all six of its cards `reexported-proof` after one had been changed to
     `proved-here`; Chapter 15 shipped one. These are surfaced, not adjudicated —
     see the note on `check_mode_claims`.

Usage: python3 scripts/check_chapter_prose.py 15
Exit status is 1 when anything is reported.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
INDEX = ROOT / "knowledge/crouzeix_textbook/crouzeix_textbook_index.md"
COVERAGE = ROOT / "content/crouzeix_textbook/coverage.json"
EXERCISES = ROOT / "content/crouzeix_textbook/exercises.json"

WORD = {
    "one": 1, "two": 2, "three": 3, "four": 4, "five": 5, "six": 6,
    "seven": 7, "eight": 8, "nine": 9, "ten": 10,
}


def load(path: Path):
    return json.loads(path.read_text())


def chapter_rows(chapter: int):
    cards = [r for r in load(COVERAGE)["items"] if r["chapter"] == chapter]
    problems = [e for e in load(EXERCISES)["exercises"] if e.get("chapter") == chapter]
    return cards, problems


def prose_path(cards) -> Path:
    return ROOT / cards[0]["prose_path"]


def index_titles() -> dict[int, str]:
    titles: dict[int, str] = {}
    for line in INDEX.read_text().splitlines():
        m = re.match(r"\s*(\d+)\.\s*\[\[[^|\]]+\|([^\]]+)\]\]", line)
        if m:
            titles[int(m.group(1))] = m.group(2).strip()
    return titles


def strip_code(text: str) -> str:
    """Blank out fenced blocks and inline code spans.

    Without this the link scan reports `[[0,-1],[1,0]]` — a rotation matrix in
    Chapter 12's ML analogy — as an unresolved wikilink.
    """
    text = re.sub(r"```.*?```", "", text, flags=re.S)
    return re.sub(r"`[^`\n]*`", "", text)


def check_links(text: str, problems: list[str]) -> None:
    for m in re.finditer(r"\[\[([^\]|#]+)", strip_code(text)):
        target = m.group(1).strip()
        if not ((ROOT / (target + ".md")).exists() or (ROOT / target).exists()):
            problems.append(f"unresolved wikilink target: {target}")


def check_registry_freshness(chapter: int, prose: Path, confirms: list[str]) -> None:
    def committed(path: Path):
        out = subprocess.run(
            ["git", "show", f"HEAD:{path.relative_to(ROOT)}"],
            capture_output=True, text=True, cwd=ROOT,
        )
        return json.loads(out.stdout) if out.returncode == 0 else None

    prose_changed = subprocess.run(
        ["git", "diff", "--quiet", "HEAD", "--", str(prose.relative_to(ROOT))],
        cwd=ROOT,
    ).returncode != 0
    if not prose_changed:
        return

    old_cov, old_ex = committed(COVERAGE), committed(EXERCISES)
    if old_cov is None or old_ex is None:
        return
    old_cards = {r["item_id"]: r for r in old_cov["items"] if r["chapter"] == chapter}
    old_problems = {e["exercise_id"]: e for e in old_ex["exercises"] if e.get("chapter") == chapter}
    cards, exercises = chapter_rows(chapter)

    for row in cards:
        old = old_cards.get(row["item_id"])
        if old is not None and old.get("pedagogical_prerequisites") == row.get("pedagogical_prerequisites"):
            confirms.append(
                f'{row["item_id"]}: pedagogical_prerequisites unchanged while the prose was rewritten '
                f'({row.get("pedagogical_prerequisites")}) — confirm it is still right, or correct it'
            )
    for row in exercises:
        old = old_problems.get(row["exercise_id"])
        if old is not None and old.get("skills") == row.get("skills"):
            confirms.append(
                f'{row["exercise_id"]}: skills unchanged while the prose was rewritten '
                f'({row.get("skills")}) — confirm it is still right, or correct it'
            )


def check_mode_claims(text: str, cards, confirms: list[str]) -> None:
    """Surface every sentence naming a formal mode, beside the registry counts.

    An earlier version tried to parse the quantity out of such sentences and fail
    when it disagreed. It matched two of the four phrasings the finished chapters
    actually use, missing the one that was Chapter 12's blocking defect ("Every
    card in this chapter is a `reexported-proof` row") because "in this chapter"
    sits between the noun and the verb. Catching only the phrasings the author
    thought of is the defect this whole script exists to compensate for, so the
    check surfaces the sentences instead of adjudicating them: no false failures,
    and no phrasing can slip past.
    """
    counts: dict[str, int] = {}
    for row in cards:
        counts[row["formal_mode"]] = counts.get(row["formal_mode"], 0) + 1
    tally = ", ".join(f"{n} {mode}" for mode, n in sorted(counts.items()))

    modes = ("reexported-proof", "proved-here", "definition", "checkpoint")
    sentences = re.split(r"(?<=[.!?])\s+", text.replace("\n", " "))
    seen: set[str] = set()
    for sentence in sentences:
        if not any(f"`{mode}`" in sentence for mode in modes):
            continue
        if not re.search(r"\bcards?\b|\brows?\b", sentence):
            continue
        trimmed = " ".join(sentence.split())[:150]
        if trimmed in seen:
            continue
        seen.add(trimmed)
        confirms.append(f"mode claim vs registry ({tally}): {trimmed}")


def main(argv: list[str]) -> int:
    if len(argv) != 2 or not argv[1].isdigit():
        print("usage: check_chapter_prose.py <chapter-number>", file=sys.stderr)
        return 2
    chapter = int(argv[1])
    cards, _ = chapter_rows(chapter)
    if not cards:
        print(f"no coverage rows for chapter {chapter}", file=sys.stderr)
        return 2
    prose = prose_path(cards)
    text = prose.read_text()

    problems: list[str] = []
    confirms: list[str] = []
    check_links(text, problems)
    check_registry_freshness(chapter, prose, confirms)
    check_mode_claims(text, cards, confirms)

    if confirms:
        print(f"chapter {chapter}: {len(confirms)} row(s) to confirm (not failures)")
        for item in confirms:
            print(f"  ? {item}")
    if problems:
        print(f"chapter {chapter}: {len(problems)} defect(s) to fix before review")
        for item in problems:
            print(f"  - {item}")
        return 1
    print(f"chapter {chapter}: links resolve; mode claims listed above for comparison")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
