from __future__ import annotations

import argparse
from pathlib import Path

from meta_harness_trae import safe_extract_archive


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("archive", type=Path)
    parser.add_argument("inventory", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    safe_extract_archive(args.archive, args.inventory, args.output)


if __name__ == "__main__":
    main()
