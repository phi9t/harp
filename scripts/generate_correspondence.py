#!/usr/bin/env python3
"""Regenerate `formalization/lean/CrouzeixTextbook/Correspondence.lean`.

Correspondence.lean is the generated Lean surface the receipt exporter reads: one
`#check` and one `ReceiptMarker` definition per declaration named by the contracts.
Registering a chapter's cards or exercise solutions changes it, and the file has to
be regenerated before a receipt will contain the new declarations at all.

The authority is the Rust `generate_correspondence` in
`crates/harp/src/crouzeix_textbook/lean.rs`, which is reachable only from the test
`deletion_generated_lean_surface_is_the_contract_correspondence`. That test asserts
the committed file equals what the Rust renderer produces, so this script cannot
drift undetected: if the two disagree, the gate fails on that test. What it saves
is a build-and-test cycle per chapter to discover a file the contracts already
determine.

Mirrored behaviour, which must match the Rust renderer exactly:

  * modules come from `lean_declaration.source_path` on theorem rows and
    `lean_solution.source_path` on exercise rows, restricted to rows whose
    `verification_target` is `CrouzeixTextbook`, mapped from a path under
    `formalization/lean/` to a dotted module name;
  * declarations are the card `name`, its `underlying_declaration` when present,
    and the exercise `declaration`;
  * both sets are `BTreeSet<String>`, so both are sorted by code point, and the
    marker index is the declaration's 1-based position in that sorted order —
    which is why adding one declaration renumbers every later marker.

Usage:
  python3 scripts/generate_correspondence.py            # rewrite the file
  python3 scripts/generate_correspondence.py --check    # exit 1 if it is stale
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
COVERAGE = ROOT / "content/crouzeix_textbook/coverage.json"
EXERCISES = ROOT / "content/crouzeix_textbook/exercises.json"
OUTPUT = ROOT / "formalization/lean/CrouzeixTextbook/Correspondence.lean"
TARGET = "CrouzeixTextbook"
PREFIX = ("formalization", "lean")


def module_from_path(path: str) -> str | None:
    parts = path.split("/")
    if tuple(parts[:2]) != PREFIX:
        return None
    relative = "/".join(parts[2:])
    if not relative.endswith(".lean"):
        return None
    return relative[: -len(".lean")].replace("/", ".")


def render() -> str:
    coverage = json.loads(COVERAGE.read_text())
    exercises = json.loads(EXERCISES.read_text())

    modules: set[str] = set()
    declarations: set[str] = set()

    for row in coverage["items"]:
        declaration = row.get("lean_declaration")
        if not declaration or declaration.get("verification_target") != TARGET:
            continue
        module = module_from_path(declaration["source_path"])
        if module:
            modules.add(module)
        declarations.add(declaration["name"])
        underlying = declaration.get("underlying_declaration")
        if underlying:
            declarations.add(underlying)

    for exercise in exercises["exercises"]:
        solution = exercise.get("lean_solution")
        if not solution or solution.get("verification_target") != TARGET:
            continue
        module = module_from_path(solution["source_path"])
        if module:
            modules.add(module)
        declarations.add(solution["declaration"])

    out = ["-- GENERATED from the Crouzeix textbook contracts. DO NOT EDIT.\n\n"]
    out += [f"import {module}\n" for module in sorted(modules)]
    out.append("\nset_option linter.defProp false\n\n")
    for index, declaration in enumerate(sorted(declarations), start=1):
        out.append(f"#check {declaration}\n")
        out.append(
            f"noncomputable def CrouzeixTextbook.ReceiptMarker.declaration{index:04} :=\n"
            f"  @{declaration}\n"
        )
    return "".join(out)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="report staleness, do not write")
    args = parser.parse_args()

    rendered = render()
    current = OUTPUT.read_text() if OUTPUT.exists() else None

    if args.check:
        if current == rendered:
            print("Correspondence.lean matches the contracts")
            return 0
        print("Correspondence.lean is stale; run without --check", file=sys.stderr)
        return 1

    if current == rendered:
        print("Correspondence.lean already matches the contracts")
        return 0
    OUTPUT.write_text(rendered)
    print(f"regenerated Correspondence.lean ({rendered.count('#check ')} declarations)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
