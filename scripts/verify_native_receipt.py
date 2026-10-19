#!/usr/bin/env python3
"""Require completed Linux qualification for the exact composed source bytes."""
import argparse
import json
import os
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from durable.qualification import source_identity


def verify(report, expected, suite):
    if (report.get('status') != 'passed' or not report.get(suite)
            or report.get('tests_run', 0) <= 0
            or report.get('tests_run') != report.get('tests_expected')
            or any(report.get(key) for key in ('skipped', 'errors', 'failures', 'expected_failures', 'unexpected_successes'))):
        raise ValueError('native qualification is incomplete, failing or contains required skips')
    if report.get('source', {}).get('files') != expected:
        raise ValueError('native qualification source bytes are stale')
    if not report.get('capabilities', {}).get('production_sqlite_admission'):
        raise ValueError('native qualification lacks production SQLite admission')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--suite', choices=('integrated', 'benchmarks', 'supervisor'), required=True)
    parser.add_argument('--receipt', type=Path)
    args = parser.parse_args()
    path = args.receipt or os.environ.get('HARP_NATIVE_' + args.suite.upper() + '_RECEIPT')
    if not path:
        raise ValueError('set HARP_NATIVE_' + args.suite.upper() + '_RECEIPT to a completed source-bound Linux receipt')
    verify(json.loads(Path(path).read_text()), source_identity()['files'], args.suite)
    print(args.suite + ' native source-bound qualification verified')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError) as error:
        print(str(error), file=sys.stderr)
        raise SystemExit(1)
