#!/usr/bin/env python3
"""Rewrite the payload digest in `docs/import-receipt.md` from `repository verify`.

The digest is a SHA-256 over the standalone import payload, pinned in the receipt
and re-derived by `harp repository verify`. Whenever the tracked contracts move,
the pin goes stale and has to be refreshed from the digest the verifier reports.

This existed as an ad-hoc regex typed fresh each time, and that is how it failed.
A run during a *forbidden-reference* failure found no digest in the output, the
capture came back empty, the substitution wrote an empty backtick pair into the
receipt, and the run still reported success — so the broken receipt was committed
and the next verify failed for a new reason. The three guards below are each a
direct response to a step of that sequence:

  * refuse to edit at all unless the failure really is a stale digest, so an
    unrelated verifier failure can never reach the substitution;
  * require a full 64 hex characters from the output, so a partial or absent
    match is an error rather than an empty string;
  * read the digest back out of the rewritten file, so a substitution that
    silently produced nothing usable cannot exit zero.

Usage: python3 scripts/refresh_import_receipt.py [--harp PATH]
Exit status is 0 when the receipt already matches or was refreshed, 1 otherwise.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
RECEIPT = ROOT / "docs/import-receipt.md"
DEFAULT_HARP = ROOT / ".build/harp-target/test-small/harp"

DIGEST = re.compile(r"\b([0-9a-f]{64})\b")
STALE = "payload digest is stale"
# The pinned digest is the last fenced value in the receipt.
SLOT = re.compile(r"`[0-9a-f]{0,64}`(\s*)\Z")


def verify(harp: Path) -> tuple[int, str]:
    result = subprocess.run(
        [str(harp), "repository", "verify"],
        capture_output=True,
        text=True,
        cwd=ROOT,
    )
    return result.returncode, result.stdout + result.stderr


def expected_digest(report: str) -> str:
    """Pull the digest the verifier expects, or explain why there isn't one."""
    if STALE not in report:
        raise SystemExit(
            "repository verify did not report a stale payload digest; "
            "refusing to edit the receipt. It said:\n" + report.strip()
        )
    found = DIGEST.findall(report)
    if not found:
        raise SystemExit(
            "no 64-character digest in the stale-digest report:\n" + report.strip()
        )
    # The report names the expected digest after the recorded one; take the
    # expected side explicitly rather than trusting the ordering.
    match = re.search(r"expected[^0-9a-f]{0,16}([0-9a-f]{64})", report)
    if not match:
        raise SystemExit(
            "could not tell which digest is expected:\n" + report.strip()
        )
    return match.group(1)


def rewrite(digest: str) -> None:
    text = RECEIPT.read_text()
    rewritten, count = SLOT.subn(f"`{digest}`\\1", text)
    if count != 1:
        raise SystemExit(
            f"expected exactly one digest slot at the end of {RECEIPT.name}, "
            f"found {count}"
        )
    RECEIPT.write_text(rewritten)
    if digest not in RECEIPT.read_text():
        raise SystemExit(f"wrote {RECEIPT.name} but the digest is not in it")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--harp", type=Path, default=DEFAULT_HARP)
    args = parser.parse_args()

    if not args.harp.exists():
        raise SystemExit(f"{args.harp} does not exist; build it first")

    code, report = verify(args.harp)
    if code == 0:
        print("receipt already matches; nothing to refresh")
        return 0

    digest = expected_digest(report)
    rewrite(digest)
    print(f"refreshed the payload digest to {digest}")

    code, report = verify(args.harp)
    if code != 0:
        print("repository verify still fails:\n" + report.strip(), file=sys.stderr)
        return 1
    print(report.strip())
    return 0


if __name__ == "__main__":
    sys.exit(main())
