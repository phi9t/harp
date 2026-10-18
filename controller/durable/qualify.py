"""Run both durable backends; missing capabilities or skipped tests never qualify."""
from __future__ import annotations

import argparse
from contextlib import closing
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import signal
import sqlite3
import subprocess
import sys
import tempfile
import time
import unittest
import uuid

from controller.durable.cgroup import Cgroup
from controller.durable.engine import Store

ROOT = Path(__file__).resolve().parents[2]
MODULES = ('controller.durable.runtime_test', 'controller.durable.insula_test',
           'controller.durable.cgroup_test')


class QualificationResult(unittest.TextTestResult):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, **kwargs)
        self.executed = []

    def startTest(self, test):
        self.executed.append(test.id())
        super().startTest(test)


def run_suite(suite, *, stream=sys.stderr):
    count = suite.countTestCases()
    started = time.monotonic()
    result = unittest.TextTestRunner(stream=stream, verbosity=2,
                                    resultclass=QualificationResult).run(suite)
    passed = (count > 0 and result.testsRun == count and result.wasSuccessful()
              and not result.skipped and not result.expectedFailures)
    return {'status': 'passed' if passed else 'failed', 'tests_expected': count,
            'tests_run': result.testsRun, 'executed': result.executed,
            'elapsed_seconds': time.monotonic() - started,
            'skipped': [(test.id(), why) for test, why in result.skipped],
            'errors': [(test.id(), why) for test, why in result.errors],
            'failures': [(test.id(), why) for test, why in result.failures],
            'expected_failures': [test.id() for test, _ in result.expectedFailures],
            'unexpected_successes': [test.id() for test in result.unexpectedSuccesses]}


def source_identity():
    # Bind the exercised runtime, fixtures and Insula implementation, including
    # uncommitted bytes. This is evidence provenance, not a landing Candidate.
    paths = sorted([*ROOT.joinpath('controller/durable').glob('*.py'),
                    *ROOT.joinpath('tools/insula').glob('*.py'),
                    ROOT / 'tools/temp_root.py'])
    return {'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT,
                                            text=True).strip(),
            'files': {str(p.relative_to(ROOT)): hashlib.sha256(p.read_bytes()).hexdigest()
                      for p in paths}}


def preflight():
    delegated = os.environ.get('CORENIUS_TEST_CGROUP_ROOT')
    if not delegated:
        raise RuntimeError('CORENIUS_TEST_CGROUP_ROOT must name an explicitly delegated cgroup v2 root')
    binding = Cgroup.validate_root(delegated)
    # Use production admission with no compatibility override, before tests
    # create disposable stores that permit older SQLite for ordinary CI.
    with tempfile.TemporaryDirectory(prefix='qualify-') as directory:
        with closing(Store(Path(directory) / 'state')) as store:
            if not store.sqlite_fix_verified:
                raise RuntimeError('production SQLite admission failed')
    return {'cgroup': binding, 'production_sqlite_admission': True}


def scope_root(unit, membership):
    if not re.fullmatch(r'corenius-qualification-[a-f0-9]{32}\.scope', unit):
        raise ValueError('invalid qualification scope unit')
    lines = membership.splitlines()
    if len(lines) != 1 or not lines[0].startswith('0::/'):
        raise ValueError('qualification requires unified cgroup v2 membership')
    relative = Path(lines[0][4:])
    if relative.name != unit or '..' in relative.parts:
        raise ValueError('process is outside the requested qualification scope')
    return Path('/sys/fs/cgroup') / relative


def run_scope(command, unit):
    """Own the waiter and exact transient unit through cancellation and cleanup."""
    interrupted = []
    old_handlers = {}
    for sig in (signal.SIGINT, signal.SIGTERM):
        old_handlers[sig] = signal.signal(sig, lambda number, frame: interrupted.append(number))
    process = None
    cleanup_error = None
    try:
        process = subprocess.Popen(command, cwd=ROOT, start_new_session=True)
        while process.poll() is None and not interrupted:
            time.sleep(0.02)
    finally:
        try:
            if process is not None:
                # Stop the requester before stopping the unit, so it cannot
                # create the scope after the cleanup request has completed.
                if process.poll() is None:
                    try:
                        os.killpg(process.pid, signal.SIGTERM)
                    except ProcessLookupError:
                        pass
                    try:
                        process.wait(timeout=2)
                    except subprocess.TimeoutExpired:
                        try:
                            os.killpg(process.pid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                        process.wait(timeout=2)
        except (OSError, subprocess.SubprocessError) as error:
            cleanup_error = str(error)
        finally:
            try:
                # Kernel containment cleanup is independent of waiter cleanup.
                # A raced exit or wait error must not bypass stopping the unit.
                stopped = subprocess.run(['systemctl', '--user', 'stop', unit],
                                         capture_output=True, text=True, timeout=10)
                if stopped.returncode:
                    state = subprocess.run(['systemctl', '--user', 'show', '--property=LoadState',
                                            '--value', unit], capture_output=True, text=True, timeout=5)
                    if state.stdout.strip() != 'not-found':
                        cleanup_error = 'could not prove qualification scope stopped: ' + stopped.stderr.strip()
            except (OSError, subprocess.SubprocessError) as error:
                cleanup_error = str(error)
            finally:
                for sig, handler in old_handlers.items():
                    signal.signal(sig, handler)
    if cleanup_error:
        print(cleanup_error, file=sys.stderr)
        return 2
    return 128 + interrupted[0] if interrupted else process.returncode


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--user-scope', action='store_true',
                        help='run in a fresh systemd user scope with delegation and a 10 minute limit')
    parser.add_argument('--scope-unit', help=argparse.SUPPRESS)
    args = parser.parse_args(argv)
    if args.user_scope:
        if args.scope_unit:
            parser.error('--user-scope cannot be nested')
        if args.output.exists():
            parser.error('output already exists; choose a fresh evidence path')
        unit = f'corenius-qualification-{uuid.uuid4().hex}.scope'
        command = ['systemd-run', '--user', '--scope', '--quiet', '--collect',
                   '--property=Delegate=yes', '--property=RuntimeMaxSec=600',
                   '--property=TimeoutStopSec=5', f'--unit={unit}',
                   sys.executable, '-m', 'controller.durable.qualify',
                   '--scope-unit', unit, '--output', str(args.output.resolve())]
        status = run_scope(command, unit)
        if status and args.output.is_file():
            report = json.loads(args.output.read_text())
            if report.get('scope_unit') == unit:
                report.update(status='incomplete', reason='scope execution or cleanup did not complete',
                              scope_returncode=status)
                args.output.write_text(json.dumps(report, indent=2) + '\n')
        return status
    args.output.parent.mkdir(parents=True, exist_ok=True)
    # Exclusive creation avoids overwriting older qualification evidence.
    report = {'status': 'incomplete', 'tests_run': 0, 'command': sys.argv,
              'python': sys.version, 'kernel': platform.release(),
              'sqlite': sqlite3.sqlite_version, 'modules': MODULES,
              'source': source_identity(), 'scope_unit': args.scope_unit}
    with args.output.open('x') as output:
        json.dump(report, output, indent=2)
    status = 2
    try:
        if args.scope_unit:
            os.environ['CORENIUS_TEST_CGROUP_ROOT'] = str(scope_root(
                args.scope_unit, Path('/proc/self/cgroup').read_text()))
        report['capabilities'] = preflight()
        loader = unittest.TestLoader()
        suites = [loader.loadTestsFromName(name) for name in MODULES]
        if loader.errors or any(s.countTestCases() == 0 for s in suites):
            raise RuntimeError('qualification modules are missing or contain no tests')
        # These groups exercise real kernel containment and runtime integration.
        for name in ('controller.durable.cgroup_test.DelegatedKernelTests',
                     'controller.durable.insula_test.DelegatedRuntimeTests'):
            if loader.loadTestsFromName(name).countTestCases() < 2 or loader.errors:
                raise RuntimeError(f'required qualification cases missing: {name}')
        test_report = run_suite(unittest.TestSuite(suites))
        source_after = source_identity()
        report.update(test_report)
        if source_after != report['source']:
            report.update(status='failed', reason='source changed during qualification')
        status = 0 if report['status'] == 'passed' else 1
    except (OSError, RuntimeError, ValueError, subprocess.SubprocessError) as error:
        report.update(status='unavailable', reason=str(error))
        print(f'qualification unavailable: {error}', file=sys.stderr)
    finally:
        args.output.write_text(json.dumps(report, indent=2) + '\n')
    return status


if __name__ == '__main__':
    raise SystemExit(main())
