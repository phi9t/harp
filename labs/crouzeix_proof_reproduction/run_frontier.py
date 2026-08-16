from __future__ import annotations

import argparse
import json
from pathlib import Path

import expert_runner


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="phase", required=True)
    for phase in expert_runner.PHASES:
        command = subparsers.add_parser(phase)
        command.add_argument("run_dir", type=Path)
    args = parser.parse_args(argv)
    print(json.dumps(expert_runner.run_phase(args.run_dir, args.phase), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
