#!/usr/bin/env python3
"""Read-only verifier for follow-up evidence and the unchanged original archive."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path, PurePosixPath


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--followup-root', default=str(Path(__file__).resolve().parent))
    parser.add_argument('--original-root', required=True)
    parser.add_argument('--check-seal', action='store_true')
    args = parser.parse_args()
    p, original = Path(args.followup_root).absolute(), Path(args.original_root).absolute()

    def digest(data):
        return hashlib.sha256(data).hexdigest()

    def load(relative):
        return json.loads((p / relative).read_bytes())

    index = load('public-index.json')
    for artifact in index['artifacts']:
        relative = PurePosixPath(artifact['path'])
        assert not relative.is_absolute() and '..' not in relative.parts
        data = (p / relative).read_bytes()
        assert len(data) == artifact['bytes'] and digest(data) == artifact['sha256'], artifact['path']
    before, after = load('original-archive-before.json'), load('original-archive-after.json')
    assert before == after and len(before) == 181
    assert {name: digest((original / name).read_bytes()) for name in before} == before
    assert load('red-run.json')['exit_code'] == 1
    assert load('green-run.json')['exit_code'] == 0
    commands = []
    for relative in ('legacy-commands.jsonl', 'portable-commands.jsonl'):
        for line in (p / relative).read_bytes().splitlines():
            command = json.loads(line)
            for channel in ('stdout', 'stderr'):
                assert digest((p / command[channel]).read_bytes()) == command[channel + '_sha256']
            commands.append(command)
    assert len(commands) == 6
    assert commands[0]['exit_code'] == 1
    assert b'Git command outside fixture' in (p / commands[0]['stderr']).read_bytes()
    portable = commands[1:]
    assert sum(c['exit_code'] == 0 for c in portable) == 1
    assert sum(c['exit_code'] == 1 for c in portable) == 4
    spec = importlib.util.spec_from_file_location('verified_portability_auditor', p / 'portable_audit.py')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    current = module.audit_archive(original)
    assert current['public_index_artifacts_verified'] == 176
    assert current['command_rawlogs_verified'] == 132
    assert current['bundle_objects'] == 10
    assert current['new_controller_calls'] == current['new_recording_callbacks'] == 0
    if args.check_seal:
        for name, expected in load('freeze.json')['hashes'].items():
            assert digest((p / name).read_bytes()) == expected, name
    print(json.dumps({'status': 'PASS', 'followup_artifacts_verified': len(index['artifacts']),
          'original_files_unchanged': 181, 'legacy_relocated_audit_exit': 1,
          'portable_cli_controls_refused': 4, 'portable_cli_positive_exits_zero': 1,
          'regression_tests_passed': 6, 'original_artifact_audit': current,
          'new_controller_calls': 0, 'new_recording_callbacks': 0}, sort_keys=True))


if __name__ == '__main__':
    main()
