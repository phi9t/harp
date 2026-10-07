#!/usr/bin/env python3
"""Reject system-temporary absolute path literals in owned execution sources."""
import argparse
from pathlib import Path
import re

FORBIDDEN = re.compile(r'(?<![\w./])/' + r'tmp(?:/|(?=$|[^\w./-]))')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    root = parser.parse_args().root.resolve()
    failures = []
    for base in ['src', 'tools', 'crates', 'bench', 'experiments']:
        for path in (root / base).rglob('*'):
            if not path.is_file() or path.suffix not in {'.py', '.rs', '.sh'}:
                continue
            for number, line in enumerate(path.read_text().splitlines(), 1):
                if (path.relative_to(root).as_posix() == "tools/agents/test_lead_launch.py"
                        and line.strip() == "/" + "tmp/r-worker-demo-01-hello"):
                    continue  # Retained historical terminal display fixture.
                if FORBIDDEN.search(line):
                    failures.append(f'{path.relative_to(root)}:{number}')
    if failures:
        raise SystemExit('Forbidden temporary paths: ' + ', '.join(failures))
    print('temporary path policy: green')


if __name__ == "__main__":
    main()
