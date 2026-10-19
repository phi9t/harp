#!/usr/bin/env python3
"""Enforce the consolidated Python package ownership graph."""
import ast
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = {'durable', 'insula', 'runner', 'lab', 'corenius', 'harp_durable'}
ALLOWED = {'insula': {'insula'}, 'durable': {'durable', 'insula'},
           'runner': {'runner', 'durable'}}


def check(root=ROOT):
    root = Path(root)
    for retired in ('corenius', 'harp_durable', 'git_workflow'):
        if (root / retired).exists():
            raise ValueError('retired framework directory remains: ' + retired)
    for package, allowed in ALLOWED.items():
        for path in (root / package).rglob('*.py'):
            if path.name.startswith('test_') or path.stem.endswith('_test'):
                continue
            tree = ast.parse(path.read_text(), filename=str(path))
            for node in ast.walk(tree):
                names = [alias.name for alias in node.names] if isinstance(node, ast.Import) else (
                    [node.module] if isinstance(node, ast.ImportFrom) and not node.level and node.module else [])
                for name in names:
                    dependency = name.split('.')[0]
                    if dependency in PACKAGES and dependency not in allowed:
                        raise ValueError(f'{path.relative_to(root)} imports forbidden layer {dependency}')
    print('durable, Insula and runner ownership graph verified')


if __name__ == '__main__':
    check()
