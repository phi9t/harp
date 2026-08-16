from __future__ import annotations

import argparse
import json
from pathlib import Path

import expert_runner


def main(argv: list[str] | None = None) -> None:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="phase", required=True)
    for phase in (*expert_runner.PHASES, "report-roots"):
        command = subparsers.add_parser(phase)
        command.add_argument("run_dir", type=Path)
    args = parser.parse_args(argv)
    if args.phase == "report-roots":
        result = expert_runner.report_roots_readiness(args.run_dir)
    else:
        result = expert_runner.run_phase(args.run_dir, args.phase)
    print(json.dumps(result, indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
