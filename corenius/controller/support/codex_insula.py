#!/usr/bin/env python3
"""Retired imported orchestration; source history is preserved in Git."""
import sys

RETIREMENT = 'Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.'


def main(argv=None):
    print(RETIREMENT, file=sys.stderr)
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
