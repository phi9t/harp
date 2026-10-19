#!/usr/bin/env python3
"""Check the acquired workload payload against its reviewed source/disposition map."""
from __future__ import annotations
import hashlib
import json
from pathlib import Path
import sys


def verify(root: Path) -> list[str]:
    manifest = json.loads((root / 'source-map.json').read_text())
    repo = root.parent.parent
    errors = []
    destinations = set()
    for record in manifest['files']:
        if 'destination' not in record:
            if record['disposition'] not in {'excluded-machine-rootfs-selection', 'excluded-transient-host-shadow-executable-copy'}:
                errors.append(f"unreviewed exclusion: {record['path']}")
            continue
        relative = Path(record['destination'])
        target = repo / relative
        if not target.resolve().is_relative_to(root.resolve()) or target.is_symlink() or not target.is_file():
            errors.append(f'missing or unsafe payload: {relative}')
            continue
        if relative.as_posix() in destinations:
            errors.append(f'duplicate destination: {relative}')
        destinations.add(relative.as_posix())
        if hashlib.sha256(target.read_bytes()).hexdigest() != record['destination_sha256']:
            errors.append(f'changed payload requires reviewed mapping: {relative}')
        if oct(target.stat().st_mode & 0o777) != record['destination_mode']:
            errors.append(f'changed payload mode: {relative}')
    for family in ('magit', 'dired', 'org', 'comint', 'jupyter', 'lsp', 'bridge'):
        expected = root / 'benchmarks/emacs-gym' / family / 'elisp/run-benchmark.el'
        if expected.relative_to(repo).as_posix() not in destinations:
            errors.append(f'omitted scenario family: {family}')
    for expected in ('benchmarks/emacs-gym/elisp/soma-gym-bridge.el',
                     'workers/emacs-doom-lifecycle/elisp/run-lifecycle.el',
                     'probes/emacs-internals/elisp/run-probes.el',
                     'ai_lsp/doom/ai-lsp-verifier.el'):
        if (root / expected).relative_to(repo).as_posix() not in destinations:
            errors.append(f'omitted supporting workload: {expected}')
    for name, description in manifest.get('first_party',{}).items():
        target = root/name
        if target.is_symlink() or not target.resolve().is_relative_to(root.resolve()) or not target.is_file():
            errors.append(f'missing or unsafe first-party workload integration: {name}')
        elif hashlib.sha256(target.read_bytes()).hexdigest() != description['sha256']:
            errors.append(f'changed first-party workload integration requires reviewed mapping: {name}')
    return errors


if __name__ == '__main__':
    failures = verify(Path(__file__).resolve().parent)
    for failure in failures:
        print(failure, file=sys.stderr)
    if failures:
        raise SystemExit(1)
    print('Soma acquired payload hashes, modes, scenarios and support closure verified')
