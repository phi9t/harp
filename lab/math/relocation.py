"""Verify path migration without relabelling historical proof qualification.

Git objects retain the original identities. This record binds each destination
byte set to its original object and explicitly identifies reviewed adaptations.
It is not a replacement for Lean or a newly approved route receipt/review.
"""
from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import stat
import subprocess


QUALIFIED_SOURCE = "883132bd0249e26f1bf37453796d1dfae1c425f8"

MIGRATION_PREFIXES = [
    {'source': 'formalization/tests/', 'destination': 'lab/math/audit_fixtures/'},
    {'source': 'formalization/', 'destination': 'lab/math/'},
    {'source': 'labs/crouzeix_proof_reproduction/tests/', 'destination': 'lab/math/proof_reproduction/'},
    {'source': 'labs/crouzeix_proof_reproduction/', 'destination': 'lab/math/proof_reproduction/'},
    {'source': 'labs/lean_trae_benchmark/tests/', 'destination': 'lab/math/lean_trae_benchmark/'},
    {'source': 'labs/lean_trae_benchmark/', 'destination': 'lab/math/lean_trae_benchmark/'},
    {'source': 'labs/sicp-evaluator/', 'destination': 'lab/sicp/'},
]


def _destination(source: str) -> str:
    for prefix in MIGRATION_PREFIXES:
        if source.startswith(prefix['source']):
            return prefix['destination'] + source[len(prefix['source']):]
    return source


def _check_mapping(record: dict) -> None:
    if record['prefixes'] != MIGRATION_PREFIXES:
        raise RelocationError('relocation prefixes differ from the approved migration')


class RelocationError(ValueError):
    pass


def _record(root: Path) -> dict:
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise RelocationError(f'duplicate relocation field: {key}')
            result[key] = value
        return result
    return json.loads(_regular(root, 'lab/math/relocation.json').read_text(), object_pairs_hook=unique)


def _regular(root: Path, relative: str) -> Path:
    pure = PurePosixPath(relative)
    if pure.is_absolute() or str(pure) != relative or any(p in {'.', '..'} for p in pure.parts):
        raise RelocationError(f'unsafe relocation path: {relative}')
    cursor = root
    for part in pure.parts:
        cursor /= part
        if cursor.is_symlink():
            raise RelocationError(f'symlinked relocation path: {relative}')
    if not cursor.is_file():
        raise RelocationError(f'missing relocation file: {relative}')
    if cursor.stat().st_nlink != 1:
        raise RelocationError(f'hardlinked relocation file: {relative}')
    return cursor


def verify(root: Path, *, qualified_revision: str = QUALIFIED_SOURCE) -> dict:
    root = root.resolve(strict=True)
    record = _record(root)
    if record['schema_version'] != 'harp-math-relocation/v1':
        raise RelocationError('unsupported relocation schema')
    revision = record['source_commit']
    if revision != qualified_revision:
        raise RelocationError('source is not the qualified migration revision')
    if len(revision) != 40 or any(c not in '0123456789abcdef' for c in revision):
        raise RelocationError('source revision must be an exact Git commit')
    subprocess.run(['git', '--no-replace-objects', '-C', str(root), 'merge-base', '--is-ancestor', revision, 'HEAD'], check=True, capture_output=True)
    tree = {}
    for line in subprocess.check_output(['git', '--no-replace-objects', '-C', str(root), 'ls-tree', '-r', revision], text=True).splitlines():
        header, path = line.split('\t', 1)
        mode, kind, blob = header.split()
        tree[path] = (mode, kind, blob)
    _check_mapping(record)
    prefixes = MIGRATION_PREFIXES
    expected = {p for p in tree if any(p.startswith(v['source']) for v in prefixes) or p.startswith('evidence/crouzeix_conjecture/')}
    rows = record['entries']
    if len(rows) != len(expected) or {r['source'] for r in rows} != expected:
        raise RelocationError('relocation source coverage differs from the original Git tree')
    destinations = set()
    retained = adapted = retired = 0
    # One batch reads original blobs, avoiding hundreds of Git subprocesses.
    batch = subprocess.check_output(['git', '--no-replace-objects', '-C', str(root), 'cat-file', '--batch'], input=''.join(r['source_blob']+'\n' for r in rows).encode())
    offset = 0
    for row in rows:
        source = row['source']
        mode, kind, blob = tree[source]
        if kind != 'blob' or (mode, blob) != (row['mode'], row['source_blob']):
            raise RelocationError(f'original Git identity mismatch: {source}')
        end = batch.index(b'\n', offset)
        _, blob_kind, size = batch[offset:end].split()
        offset = end+1
        original = batch[offset:offset+int(size)]
        offset += int(size)+1
        if blob_kind != b'blob' or hashlib.sha256(original).hexdigest() != row['source_sha256']:
            raise RelocationError(f'original byte identity mismatch: {source}')
        expected_destination = _destination(source)
        destination = row['destination']
        if destination != expected_destination or destination in destinations:
            raise RelocationError(f'destination mapping mismatch: {source}')
        destinations.add(destination)
        disposition = row['disposition']
        if disposition == 'retired':
            if source != 'labs/crouzeix_proof_reproduction/tests/test_verify_registration.py' or not row.get('reason'):
                raise RelocationError(f'unapproved retirement: {source}')
            retired += 1
            continue
        path = _regular(root, destination)
        data = path.read_bytes()
        if hashlib.sha256(data).hexdigest() != row['destination_sha256']:
            raise RelocationError(f'destination byte identity mismatch: {destination}')
        executable = bool(path.stat().st_mode & stat.S_IXUSR)
        if executable != (mode == '100755'):
            raise RelocationError(f'destination executable mode mismatch: {destination}')
        immutable = source.endswith('.lean') or source.startswith('evidence/') or '/proof-slices/' in source or '/formal_targets/' in source and not source.endswith(('proof-obligations.json', 'theorem-readbacks.json'))
        if source == 'formalization/lean/CrouzeixTextbook/ExportReceipt.lean' and data == original.replace(b'formalization/lean/', b'lab/math/lean/'):
            adapted += 1
        elif disposition == 'retained' or immutable:
            if data != original:
                raise RelocationError(f'preserved source bytes changed: {destination}')
            retained += 1
        elif disposition == 'adapted':
            adapted += 1
        else:
            raise RelocationError(f'unknown disposition: {source}')
    # Extra proof or evidence inputs must not inherit the old qualification.
    import os
    for directory in ('lab/math/lean', 'lab/math/proof_reproduction/formal_targets', 'evidence/crouzeix_conjecture'):
        base = root / directory
        if not base.exists():
            continue
        for current, folders, files in os.walk(base, followlinks=False):
            if Path(current) == root / 'lab/math/lean':
                folders[:] = [name for name in folders if name != '.lake']
            for name in (*folders, *files):
                candidate = Path(current) / name
                if candidate.is_symlink():
                    raise RelocationError(f'symlinked proof authority: {candidate.relative_to(root)}')
            for name in files:
                relative = (Path(current) / name).relative_to(root).as_posix()
                if relative not in destinations:
                    raise RelocationError(f'unqualified new proof authority: {relative}')
    return {'schema_version':record['schema_version'], 'source_commit':revision,
            'qualification':record['qualification'], 'retained':retained, 'adapted':adapted,
            'retired':retired, 'status':'preserved'}


def resolve(root: Path, relative: str) -> str:
    """Resolve an archived location only through its digest-checked mapping."""
    record_path = root / 'lab/math/relocation.json'
    if not record_path.is_file():
        return relative
    record = _record(root)
    _check_mapping(record)
    for row in record['entries']:
        if row['source'] == relative and row['disposition'] != 'retired':
            destination = row['destination']
            if destination != _destination(relative):
                raise RelocationError(f'destination mapping mismatch: {relative}')
            data = _regular(root, destination).read_bytes()
            if hashlib.sha256(data).hexdigest() != row['destination_sha256']:
                raise RelocationError(f'destination byte identity mismatch: {destination}')
            return destination
    return relative


def project_locations(root: Path, value):
    """An explicit reader view; the persisted record and its digests stay original."""
    if isinstance(value, str):
        return resolve(root, value)
    if isinstance(value, list):
        return [project_locations(root, item) for item in value]
    if isinstance(value, dict):
        return {key: project_locations(root, item) for key, item in value.items()}
    return value


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root', type=Path, default=Path(__file__).resolve().parents[2] if '__file__' in globals() else None)
    args = parser.parse_args()
    if args.root is None:
        parser.error("--root is required for the embedded verifier")
    try:
        print(json.dumps(verify(args.root), sort_keys=True))
    except (OSError, ValueError, KeyError, subprocess.CalledProcessError) as error:
        print(json.dumps({'status':'invalid', 'reason':str(error)}))
        return 1
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
