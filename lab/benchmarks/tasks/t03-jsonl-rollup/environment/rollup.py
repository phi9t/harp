#!/usr/bin/env python3
import json
import sys


def summarize(path: str, service: str, status: str) -> tuple[int, int]:
    count = 0
    total = 0
    with open(path, "r", encoding="utf-8") as handle:
        for line in handle:
            if not line.strip():
                continue
            record = json.loads(line)
            count += 1
            total += int(record["amount"])
    return count, total


def main(argv: list[str]) -> int:
    if len(argv) != 4:
        print("usage: rollup.py EVENTS SERVICE STATUS", file=sys.stderr)
        return 2
    count, total = summarize(argv[1], argv[2], argv[3])
    print(f"count={count} total={total}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
