#!/usr/bin/env python3
"""Retired tmux orchestration; original implementation remains in Git history."""
import sys

RETIREMENT = 'Retired imported tmux harness. Use harp durable --help for coding lifecycle and human-approved landing; pure offline reference/noop replay remains in lab/benchmarks/harness/run.py.'


def main(argv=None):
    print(RETIREMENT, file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
