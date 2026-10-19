#!/usr/bin/env python3
"""Read-only audit of a relocated sealed archive, retaining historical path scope.

Artifact root selects where immutable bytes are read. Recorded execution origin
is derived from sealed observations and never changed to match artifact root.
Candidate files are explicit negative-control inputs, not rewritten history.
The original auditor's hash-pinned pure bundle helpers are reused; its main and
the controller are never called. No processes, Git writes, or records are made.
"""
import argparse
import hashlib
import json
from pathlib import Path, PurePosixPath
import sys

ORIGINAL_SEAL_SHA256 = 'f3cf98a41b724453c3fa3beebdecf5b667a96ce4ed105bcd7f05c727f7115fdb'


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def audit_archive(artifact_root, *, candidates=None, expected_execution_origin=None):
    root = Path(artifact_root).absolute()
    candidates = candidates or {}
    require(set(candidates) <= {'proof-run.json', 'public-index.json', 'fixture-only.bundle'},
            'unsupported candidate input')

    def read(relative):
        path = PurePosixPath(relative)
        require(not path.is_absolute() and '..' not in path.parts and path.parts,
                'unsafe artifact-relative path')
        return Path(candidates[relative]).read_bytes() if relative in candidates else (root / relative).read_bytes()

    def load(relative):
        return json.loads(read(relative))

    # A fixed externally published seal prevents self-consistent candidate
    # metadata, index or bundle changes from becoming historical observations.
    require(sha(read('freeze.json')) == ORIGINAL_SEAL_SHA256, 'original seal mismatch')
    seal = load('freeze.json')
    for name, expected in seal['hashes'].items():
        require(sha(read(name)) == expected, 'frozen artifact mismatch: ' + name)
    index = load('public-index.json')
    require(len(index['artifacts']) == 176, 'original artifact count')
    names = set()
    for artifact in index['artifacts']:
        name = artifact['path']
        require(name not in names, 'duplicate indexed artifact')
        names.add(name)
        data = read(name)
        require(len(data) == artifact['bytes'] and sha(data) == artifact['sha256'],
                'indexed artifact mismatch: ' + name)
    ids, result = load('fixture-identities.json'), load('proof-result.json')
    run, before_run = load('proof-run.json'), load('pre-run-freeze.json')
    commands = [json.loads(line) for line in read('commands.jsonl').splitlines()]
    require(ids == seal['fixture_identities'] and result == seal['results'], 'seal metadata binding')
    require(ids['reviewed_root_commit'] == index['source_read_commit'], 'reviewed source commit binding')
    require(ids['source_commit_C'] == index['fixture_source_commit_C'] and
            ids['grant_commit_G'] == index['fixture_grant_commit_G'], 'index commit binding')
    require(run['exit_code'] == 0 and result['status'] == 'PASS', 'original proof failed')
    require(run['argv'] == before_run['command'] and run['argv'][:2] == ['python3', '-B'] and len(run['argv']) == 3,
            'pre-run and recorded invocation disagree')
    script = PurePosixPath(run['argv'][2])
    require(script.is_absolute() and '..' not in script.parts and script.name == 'proof.py',
            'invalid recorded execution script path')
    origin = script.parent
    if expected_execution_origin is not None:
        require(str(origin) == str(PurePosixPath(expected_execution_origin)),
                'recorded execution origin contradicts expectation')
    fixture_origin = str(origin / 'fixture')
    require(sha(read('proof.py')) == run['proof_script_sha256'] == before_run['proof_script_sha256'],
            'prewritten proof binding')
    for channel in ('stdout', 'stderr'):
        require(sha(read(run[channel])) == run[channel + '_sha256'], 'proof rawlog reference')
    source = read('fixture/tools/probes/trae_case_control.py')
    require(sha(source) == ids['source_sha256'] == index['source_sha256'] and
            source == read('preimages/reviewed-source.py'), 'source identity')
    require(sha(read('preimages/reviewed-tests.py')) == ids['reviewed_tests_sha256'], 'test identity')
    case_raw, grant_raw = read('fixture/fixtures/case.json'), read('fixture/fixtures/grant.json')
    case, grant = json.loads(case_raw), json.loads(grant_raw)
    require(sha(case_raw) == ids['case_sha256'] == grant['case_sha256'], 'case hash binding')
    require(sha(grant_raw) == ids['grant_sha256'], 'grant hash binding')
    require(grant['input_commit'] == ids['source_commit_C'], 'source commit binding')
    require(grant['fixture_only'] is True and grant['physical_admission'] is False and
            case['fixture_only'] is True and case['physical_admission'] is False, 'fixture authority markers')
    require(case['command']['cwd'] == fixture_origin, 'case contradicts recorded fixture origin')
    require(grant['output_directory'] == str(origin / 'records/operation'), 'grant contradicts recorded output origin')
    require(sha(encoded(case['command'])) == ids['command_sha256'] == grant['command_sha256'], 'command binding')
    require(case['sources'] == {'tools/probes/trae_case_control.py': ids['source_sha256']}, 'source map')
    callback = load('recording-callback.json')
    require(callback['command'] == case['command'] and callback['operation_id'] == grant['operation_id'] and
            callback['timeout_seconds'] == 30, 'recorded callback binding')
    started, terminal = load('records/operation/started.json'), load('records/operation/terminal.json')
    require(started['provenance'] == 'committed-git' and started['command'] == case['command'], 'started binding/provenance')
    for record in (started, terminal):
        require(record['grant_revision'] == ids['grant_commit_G'], 'record grant revision')
        require(record['grant_sha256'] == sha(grant_raw) and record['case_sha256'] == sha(case_raw), 'record input references')
        require(record['operation_id'] == grant['operation_id'] and record['sources'] == case['sources'], 'record operation/source references')
    require(terminal['command_sha256'] == sha(encoded(started['command'])) and
            terminal['status'] == 'terminal' and terminal['exit_code'] == 0 and terminal['executed'] is True, 'terminal state')
    record_names = {'started.json', 'terminal.json', 'stdout.log.txt', 'stderr.log.txt'}
    require({p.name for p in (root / 'records/operation').iterdir()} == record_names, 'unexpected operation record')
    actual_records = {name: sha(read('records/operation/' + name)) for name in sorted(record_names)}
    require(actual_records == load('record-hashes-before-duplicate.json') == load('record-hashes-after-duplicate.json') ==
            seal['record_hashes'] == index['record_hashes'], 'duplicate/freeze record preservation')
    outcome_names = sorted(name for name in names if name.startswith('outcomes/') and name.endswith('.json'))
    outcomes = [load(name) for name in outcome_names]
    require(len(outcomes) == 11 and sum(o['result']['executed'] for o in outcomes) == 1, 'historical invocation count')
    require(all(o['read_committed_argument'] == 'OMITTED' for o in outcomes), 'reader injection evidence')
    require(load('outcomes/positive-default-git-reader.json')['actual_reader_git_commands'] == 12, 'positive reader calls')
    require(load('outcomes/duplicate-default-git-reader.json')['callbacks_total'] == 1, 'historical duplicate callback count')
    require(len(commands) == 66, 'historical Git command count')
    reader_commands = [c for c in commands if c['purpose'] == 'actual-default-committed-reader']
    require(len(reader_commands) == 44, 'actual reader command count')
    reviewed_paths = {'tools/probes/trae_case_control.py', 'tools/probes/test_trae_case_control.py'}
    seen_reviewed = set()
    for command in commands:
        for channel in ('stdout', 'stderr'):
            require(sha(read(command[channel])) == command[channel + '_sha256'], 'command rawlog reference')
        argv = command['argv']
        require(argv[0] == 'git', 'historical non-Git process')
        if command['purpose'].startswith('read-only-reviewed'):
            require(len(argv) == 5 and argv[1] == '-C' and argv[3] == 'show', 'historical shared mutation command')
            revision, relative = argv[4].split(':', 1)
            require(revision == ids['reviewed_root_commit'] and relative in reviewed_paths, 'historical reviewed source scope')
            seen_reviewed.add(relative)
        else:
            require(argv[1:3] == ['-C', fixture_origin], 'historical Git command outside recorded fixture')
        if command in reader_commands:
            require(argv[3] in ('cat-file', 'show'), 'historical reader command kind')
    require(seen_reviewed == reviewed_paths, 'reviewed source/test read coverage')
    require(sum(c['exit_code'] != 0 for c in commands) == 2 and
            all(c['exit_code'] == 128 for c in commands if c['exit_code'] != 0), 'historical expected Git errors')
    sync_events = load('fsync-events.json')
    require(all(PurePosixPath(e['path']).is_relative_to(origin / 'records/operation') for e in sync_events),
            'fsync observations contradict recorded output origin')
    require(result['recording_callbacks'] == 1 and result['fixture_repositories'] == 1 and
            all(result[name] == 0 for name in ('namespace_starts', 'runtime_starts', 'provider_requests', 'service_starts',
                                             'case_argv_executions', 'root_or_shared_mutations')), 'historical resource scope')
    # Reuse only exact, sealed original pure decoding functions. Its main has
    # the portability defect and is deliberately never called.
    namespace = {'__name__': 'sealed_bundle_helpers', '__file__': str(root / 'audit.py')}
    exec(compile(read('audit.py'), str(root / 'audit.py'), 'exec'), namespace)
    namespace['P'] = root
    refs, objects = namespace['read_bundle']()
    require(sha(read('fixture-only.bundle')) == ids['bundle_sha256'] == index['bundle_sha256'], 'bundle identity')
    expected_refs = [ids['grant_commit_G'] + ' HEAD', ids['grant_commit_G'] + ' refs/heads/fixture-only']
    require(sorted(refs) == sorted(ids['bundle_refs']) == sorted(expected_refs), 'bundle refs')
    closure = set()
    for label, parents, expected_paths in (
        ('source_commit_C', [], {'tools/probes/trae_case_control.py': source}),
        ('grant_commit_G', [ids['source_commit_C']], {'tools/probes/trae_case_control.py': source,
         'fixtures/case.json': case_raw, 'fixtures/grant.json': grant_raw})):
        oid = ids[label]
        commit = objects[oid]
        require(commit['resolved_type'] == 'commit', 'bundle commit type')
        header = commit['resolved_data'].split(b'\n\n', 1)[0].decode().splitlines()
        require([line.split(' ', 1)[1] for line in header if line.startswith('parent ')] == parents, 'bundle contains shared parent')
        paths, tree_closure = namespace['tree_paths'](objects, header[0].split(' ', 1)[1])
        require(paths == expected_paths, 'bundle paths/bytes')
        closure.add(oid)
        closure.update(tree_closure)
    require(set(objects) == closure == {line.split(' ', 1)[0] for line in read('fixture-objects.txt').decode().splitlines()},
            'bundle object closure')
    return {'status': 'PASS', 'retained_artifact_root': str(root), 'recorded_execution_origin': str(origin),
            'original_seal_sha256': ORIGINAL_SEAL_SHA256, 'original_index_sha256': sha(read('public-index.json')),
            'public_index_artifacts_verified': len(names), 'command_rawlogs_verified': len(commands) * 2,
            'actual_reader_git_commands_verified': len(reader_commands), 'controller_invocations_verified': len(outcomes),
            'bundle_objects': len(objects), 'bundle_refs': refs, 'bundle_sha256': ids['bundle_sha256'],
            'record_hashes': actual_records, 'complete_owned_synthetic_closure': True,
            'controller_imported_or_run_by_audit': False, 'new_controller_calls': 0,
            'new_recording_callbacks': 0, 'prior_recording_callbacks_remains': 1,
            'new_fixture_repositories': 0, 'physical_admission': False, 'acceptance_authority': False,
            'candidate_inputs': sorted(candidates)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--artifact-root', required=True)
    parser.add_argument('--expected-execution-origin')
    parser.add_argument('--candidate-proof-run', help='Negative-control input; immutable seal must still match.')
    parser.add_argument('--candidate-index', help='Negative-control input; immutable seal must still match.')
    parser.add_argument('--candidate-bundle', help='Negative-control input; immutable seal must still match.')
    args = parser.parse_args()
    candidates = {name: value for name, value in (
        ('proof-run.json', args.candidate_proof_run), ('public-index.json', args.candidate_index),
        ('fixture-only.bundle', args.candidate_bundle)) if value is not None}
    try:
        result = audit_archive(args.artifact_root, candidates=candidates,
                               expected_execution_origin=args.expected_execution_origin)
    except (AssertionError, KeyError, ValueError, OSError) as error:
        print('REFUSED: ' + str(error), file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
