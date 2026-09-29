import argparse
import csv
import json


STATES = ("ok", "failed", "queued")


def read_counts(path: str) -> dict[str, int]:
    counts = {state: 0 for state in STATES}
    with open(path, newline="") as handle:
        for row in csv.DictReader(handle):
            state = (row.get("state") or "").strip().lower()
            if state in counts:
                counts[state] += 1
    return counts


def format_text(counts: dict[str, int]) -> str:
    return "\n".join(f"{state}: {counts[state]}" for state in STATES)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(prog="deployctl")
    subcommands = parser.add_subparsers(dest="command", required=True)
    status = subcommands.add_parser("status")
    status.add_argument("path")
    status.add_argument("--format", choices=("text", "json"), default="text")

    args = parser.parse_args(argv)
    if args.command == "status":
        counts = read_counts(args.path)
        if args.format == "json":
            print(json.dumps(counts, sort_keys=True, separators=(",", ":")))
        else:
            print(format_text(counts))
        return 0
    parser.error("unknown command")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
