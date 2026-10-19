#!/usr/bin/env python3
"""Public CLI portability tests; adversarial candidates are inputs, not run history."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import unittest

P = Path(__file__).resolve().parent
ARCHIVE = Path('/data02/home/philip.yang/workspace/corenius-codex-reference-design/docs/efforts/team-control-plane/evidence/trae20-controller-repair2/git-reader')
ORIGIN = '/tmp/corenius-controller-git-reader-proof-20261003'
parser = argparse.ArgumentParser()
parser.add_argument('--legacy', action='store_true')
ARGS = parser.parse_args()
CALLS = []


def invoke(label, *extra):
    if ARGS.legacy:
        argv = ['python3', '-B', str(ARCHIVE / 'audit.py'), '--check-index', '--check-freeze']
    else:
        argv = ['python3', '-B', str(P / 'portable_audit.py'), '--artifact-root', str(ARCHIVE), *extra]
    result = subprocess.run(argv, capture_output=True, check=False)
    prefix = ('legacy-' if ARGS.legacy else 'portable-') + label
    stdout, stderr = P / 'rawlogs' / (prefix + '.stdout'), P / 'rawlogs' / (prefix + '.stderr')
    # Records have exclusive names; rerunning must not overwrite prior evidence.
    for path, data in ((stdout, result.stdout), (stderr, result.stderr)):
        with path.open('xb') as fd:
            fd.write(data)
    event = {'label': label, 'argv': argv, 'exit_code': result.returncode,
             'stdout': stdout.relative_to(P).as_posix(), 'stdout_sha256': hashlib.sha256(result.stdout).hexdigest(),
             'stderr': stderr.relative_to(P).as_posix(), 'stderr_sha256': hashlib.sha256(result.stderr).hexdigest(),
             'purpose': 'read-only audit invocation; no controller execution'}
    with (P / ('legacy-commands.jsonl' if ARGS.legacy else 'portable-commands.jsonl')).open('ab') as fd:
        fd.write(json.dumps(event, sort_keys=True).encode() + b'\n')
    CALLS.append(event)
    return result


class ArtifactPortability(unittest.TestCase):
    def test_relocated_archive_passes_exact_retained_bindings(self):
        result = invoke('relocated-positive')
        self.assertEqual(result.returncode, 0, result.stderr.decode())
        output = json.loads(result.stdout)
        self.assertEqual(output['public_index_artifacts_verified'], 176)
        self.assertEqual(output['command_rawlogs_verified'], 132)
        self.assertEqual(output['bundle_objects'], 10)
        self.assertEqual(output['actual_reader_git_commands_verified'], 44)
        self.assertEqual(output['recorded_execution_origin'], ORIGIN)
        self.assertEqual(output['retained_artifact_root'], str(ARCHIVE))
        self.assertFalse(output['controller_imported_or_run_by_audit'])

    @unittest.skipIf(ARGS.legacy, 'candidate controls apply only to new auditor')
    def test_contradictory_origin_expectation_refuses(self):
        result = invoke('contradictory-origin', '--expected-execution-origin', ORIGIN + '-CONTRADICTORY')
        self.assertEqual(result.returncode, 1)
        self.assertIn(b'recorded execution origin contradicts expectation', result.stderr)

    @unittest.skipIf(ARGS.legacy, 'candidate controls apply only to new auditor')
    def test_candidate_recorded_origin_metadata_refuses(self):
        value = json.loads((ARCHIVE / 'proof-run.json').read_bytes())
        value['argv'][2] = ORIGIN + '-CONTRADICTORY/proof.py'
        candidate = P / 'controls' / 'contradictory-proof-run.json'
        with candidate.open('xb') as fd:
            fd.write(json.dumps(value, sort_keys=True).encode() + b'\n')
        result = invoke('candidate-origin-metadata', '--candidate-proof-run', str(candidate))
        self.assertEqual(result.returncode, 1)
        self.assertIn(b'frozen artifact mismatch: proof-run.json', result.stderr)

    @unittest.skipIf(ARGS.legacy, 'candidate controls apply only to new auditor')
    def test_candidate_index_refuses(self):
        value = json.loads((ARCHIVE / 'public-index.json').read_bytes())
        value['fixture_grant_commit_G'] = '0' * 40
        candidate = P / 'controls' / 'contradictory-index.json'
        with candidate.open('xb') as fd:
            fd.write(json.dumps(value, sort_keys=True).encode() + b'\n')
        result = invoke('candidate-index', '--candidate-index', str(candidate))
        self.assertEqual(result.returncode, 1)
        self.assertIn(b'frozen artifact mismatch: public-index.json', result.stderr)

    @unittest.skipIf(ARGS.legacy, 'candidate controls apply only to new auditor')
    def test_candidate_bundle_ref_refuses(self):
        raw = (ARCHIVE / 'fixture-only.bundle').read_bytes()
        raw = raw.replace(b'refs/heads/fixture-only', b'refs/heads/FOREIGN-only', 1)
        candidate = P / 'controls' / 'contradictory-bundle.bundle'
        with candidate.open('xb') as fd:
            fd.write(raw)
        result = invoke('candidate-bundle-ref', '--candidate-bundle', str(candidate))
        self.assertEqual(result.returncode, 1)
        self.assertIn(b'frozen artifact mismatch: fixture-only.bundle', result.stderr)

    def test_original_archive_remains_byte_exact(self):
        snapshot = json.loads((P / 'original-archive-before.json').read_bytes())
        self.assertEqual(len(snapshot), 181)
        self.assertEqual({name: hashlib.sha256((ARCHIVE / name).read_bytes()).hexdigest()
                          for name in snapshot}, snapshot)


if __name__ == '__main__':
    unittest.main(argv=['regression.py'], verbosity=2)
