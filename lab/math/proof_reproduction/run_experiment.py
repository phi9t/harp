from __future__ import annotations

import argparse
import json
from pathlib import Path

from runner import run_experiment


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("run_dir", type=Path)
    args = parser.parse_args()
    print(json.dumps(run_experiment(args.run_dir), indent=2, sort_keys=True))


if __name__ == "__main__":
    main()
