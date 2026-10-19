#!/usr/bin/env python3
"""Read-only retained-evidence/bundle audit; never imports or executes controller."""
import hashlib
import json
from pathlib import Path
import struct
import sys
import zlib

P = Path(__file__).resolve().parent


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def load(name):
    return json.loads((P / name).read_bytes())


def encoded(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':')).encode()


def variable(data, pos):
    value, shift = 0, 0
    while True:
        byte = data[pos]
        pos += 1
        value |= (byte & 127) << shift
        if not byte & 128:
            return value, pos
        shift += 7


def delta(base, instructions):
    base_length, pos = variable(instructions, 0)
    result_length, pos = variable(instructions, pos)
    require(base_length == len(base), 'bundle delta base length')
    result = bytearray()
    while pos < len(instructions):
        instruction = instructions[pos]
        pos += 1
        if instruction & 128:
            offset, count = 0, 0
            for bit in range(4):
                if instruction & (1 << bit):
                    offset |= instructions[pos] << (8 * bit)
                    pos += 1
            for bit in range(3):
                if instruction & (1 << (bit + 4)):
                    count |= instructions[pos] << (8 * bit)
                    pos += 1
            count = count or 65536
            require(offset + count <= len(base), 'bundle delta copy bounds')
            result.extend(base[offset:offset + count])
        else:
            require(instruction != 0, 'bundle delta zero instruction')
            result.extend(instructions[pos:pos + instruction])
            pos += instruction
    require(len(result) == result_length, 'bundle delta result length')
    return bytes(result)


def read_bundle():
    raw = (P / 'fixture-only.bundle').read_bytes()
    header, pack = raw.split(b'\n\n', 1)
    lines = header.decode().splitlines()
    require(lines[0] == '# v2 git bundle', 'unexpected bundle version')
    require(not any(line.startswith('-') for line in lines), 'external prerequisite in bundle')
    refs = lines[1:]
    require(pack[:4] == b'PACK', 'bundle PACK missing')
    version, count = struct.unpack('>II', pack[4:12])
    require(version in (2, 3), 'pack version')
    require(hashlib.sha1(pack[:-20]).digest() == pack[-20:], 'pack checksum')
    types = {1: 'commit', 2: 'tree', 3: 'blob', 4: 'tag'}
    pos, entries = 12, {}
    for _ in range(count):
        start = pos
        byte = pack[pos]
        pos += 1
        kind, length, shift = (byte >> 4) & 7, byte & 15, 4
        while byte & 128:
            byte = pack[pos]
            pos += 1
            length |= (byte & 127) << shift
            shift += 7
        base = None
        if kind == 6:
            byte = pack[pos]
            pos += 1
            distance = byte & 127
            while byte & 128:
                byte = pack[pos]
                pos += 1
                distance = ((distance + 1) << 7) | (byte & 127)
            base = start - distance
        elif kind == 7:
            base = pack[pos:pos + 20].hex()
            pos += 20
        else:
            require(kind in types, 'pack object type')
        decompressor = zlib.decompressobj()
        data = decompressor.decompress(pack[pos:])
        require(decompressor.eof, 'pack zlib incomplete')
        pos += len(pack[pos:]) - len(decompressor.unused_data)
        require(len(data) == length, 'pack object length')
        entries[start] = {'kind': kind, 'data': data, 'base': base}
    require(pos == len(pack) - 20, 'extra pack bytes')
    objects = {}
    unresolved = dict(entries)
    while unresolved:
        progress = False
        for offset, entry in list(unresolved.items()):
            kind = entry['kind']
            if kind in types:
                name, data = types[kind], entry['data']
            else:
                base = entries.get(entry['base']) if kind == 6 else objects.get(entry['base'])
                if not base or 'resolved_type' not in base:
                    continue
                name, data = base['resolved_type'], delta(base['resolved_data'], entry['data'])
            oid = hashlib.sha1(name.encode() + b' ' + str(len(data)).encode() + b'\0' + data).hexdigest()
            entry.update({'resolved_type': name, 'resolved_data': data, 'oid': oid})
            objects[oid] = entry
            del unresolved[offset]
            progress = True
        require(progress, 'unresolved pack delta')
    require(len(objects) == count, 'duplicate/unidentified pack objects')
    return refs, objects


def tree_paths(objects, tree, prefix=''):
    obj = objects[tree]
    require(obj['resolved_type'] == 'tree', 'referenced tree type')
    data, pos, result, closure = obj['resolved_data'], 0, {}, {tree}
    while pos < len(data):
        space = data.index(b' ', pos)
        nul = data.index(b'\0', space)
        mode = data[pos:space]
        name = data[space + 1:nul].decode()
        oid = data[nul + 1:nul + 21].hex()
        pos = nul + 21
        closure.add(oid)
        path = prefix + name
        if mode == b'40000':
            nested, nested_closure = tree_paths(objects, oid, path + '/')
            result.update(nested)
            closure.update(nested_closure)
        else:
            require(mode == b'100644', 'unexpected fixture tree mode')
            require(objects[oid]['resolved_type'] == 'blob', 'fixture file not blob')
            result[path] = objects[oid]['resolved_data']
    return result, closure


def main():
    ids = load('fixture-identities.json')
    result = load('proof-result.json')
    run = load('proof-run.json')
    freeze_before = load('pre-run-freeze.json')
    require(run['exit_code'] == 0 and result['status'] == 'PASS', 'proof execution failed')
    require(sha((P / 'proof.py').read_bytes()) == freeze_before['proof_script_sha256'] == run['proof_script_sha256'],
            'prewritten proof bytes changed')
    for channel in ('stdout', 'stderr'):
        require(sha((P / run[channel]).read_bytes()) == run[channel + '_sha256'], 'proof rawlog hash')
    source = (P / 'fixture/tools/probes/trae_case_control.py').read_bytes()
    require(sha(source) == ids['source_sha256'] and source == (P / 'preimages/reviewed-source.py').read_bytes(),
            'source identity')
    require(sha((P / 'preimages/reviewed-tests.py').read_bytes()) == ids['reviewed_tests_sha256'], 'test identity')
    case_raw = (P / 'fixture/fixtures/case.json').read_bytes()
    grant_raw = (P / 'fixture/fixtures/grant.json').read_bytes()
    case, grant = json.loads(case_raw), json.loads(grant_raw)
    require(sha(case_raw) == ids['case_sha256'] == grant['case_sha256'], 'case hash binding')
    require(sha(grant_raw) == ids['grant_sha256'], 'grant hash binding')
    require(grant['input_commit'] == ids['source_commit_C'], 'source commit binding')
    require(grant['fixture_only'] is True and grant['physical_admission'] is False, 'fixture authority marker')
    require(sha(encoded(case['command'])) == ids['command_sha256'] == grant['command_sha256'], 'command binding')
    require(case['sources'] == {'tools/probes/trae_case_control.py': ids['source_sha256']}, 'source map')
    started = load('records/operation/started.json')
    terminal = load('records/operation/terminal.json')
    require(started['provenance'] == 'committed-git', 'reader provenance')
    for retained in (started, terminal):
        require(retained['grant_revision'] == ids['grant_commit_G'], 'grant revision record')
        require(retained['grant_sha256'] == sha(grant_raw) and retained['case_sha256'] == sha(case_raw),
                'record input references')
        require(retained['operation_id'] == grant['operation_id'] and retained['sources'] == case['sources'],
                'record operation/source references')
    require(terminal['command_sha256'] == sha(encoded(started['command'])), 'terminal command reference')
    require(terminal['status'] == 'terminal' and terminal['exit_code'] == 0 and terminal['executed'] is True,
            'terminal state')
    before, after = load('record-hashes-before-duplicate.json'), load('record-hashes-after-duplicate.json')
    actual_records = {p.name: sha(p.read_bytes()) for p in (P / 'records/operation').iterdir()}
    require(before == after == actual_records, 'duplicate record preservation')
    require(set(actual_records) == {'started.json', 'terminal.json', 'stdout.log.txt', 'stderr.log.txt'},
            'unexpected retained record')
    outcomes = [json.loads(p.read_bytes()) for p in sorted((P / 'outcomes').glob('*.json'))]
    require(len(outcomes) == 11 and sum(o['result']['executed'] for o in outcomes) == 1, 'outcome execution count')
    require(all(o['read_committed_argument'] == 'OMITTED' for o in outcomes), 'reader injection evidence')
    require(load('outcomes/positive-default-git-reader.json')['actual_reader_git_commands'] == 12, 'positive real reader calls')
    require(load('outcomes/duplicate-default-git-reader.json')['callbacks_total'] == 1, 'duplicate callback count')
    commands = [json.loads(line) for line in (P / 'commands.jsonl').read_bytes().splitlines()]
    require(len(commands) == 66, 'command count')
    reader_commands = [c for c in commands if c['purpose'] == 'actual-default-committed-reader']
    require(len(reader_commands) == 44, 'real reader total command count')
    for c in commands:
        for channel in ('stdout', 'stderr'):
            require(sha((P / c[channel]).read_bytes()) == c[channel + '_sha256'], 'command rawlog reference')
        if c['purpose'].startswith('read-only-reviewed'):
            require(c['argv'][3] == 'show', 'shared repository mutation command')
        else:
            require(c['argv'][1:3] == ['-C', str(P / 'fixture')], 'Git command outside fixture')
    require(sum(c['exit_code'] != 0 for c in commands) == 2, 'expected Git error controls')
    require(all(c['exit_code'] == 128 for c in commands if c['exit_code'] != 0), 'Git error exit values')
    refs, objects = read_bundle()
    require(sha((P / 'fixture-only.bundle').read_bytes()) == ids['bundle_sha256'], 'bundle hash')
    require(sorted(refs) == sorted(ids['bundle_refs']), 'bundle refs')
    require(sorted(refs) == sorted([ids['grant_commit_G'] + ' HEAD', ids['grant_commit_G'] + ' refs/heads/fixture-only']),
            'bundle contains foreign refs')
    closure = set()
    for label, expected_parent, expected_paths in (
        ('source_commit_C', [], {'tools/probes/trae_case_control.py': source}),
        ('grant_commit_G', [ids['source_commit_C']], {'tools/probes/trae_case_control.py': source,
         'fixtures/case.json': case_raw, 'fixtures/grant.json': grant_raw})):
        oid = ids[label]
        commit = objects[oid]
        require(commit['resolved_type'] == 'commit', 'bundle commit type')
        header = commit['resolved_data'].split(b'\n\n', 1)[0].decode().splitlines()
        parents = [line.split(' ', 1)[1] for line in header if line.startswith('parent ')]
        require(parents == expected_parent, 'bundle contains shared/foreign commit parent')
        tree = header[0].split(' ', 1)[1]
        paths, tree_closure = tree_paths(objects, tree)
        require(paths == expected_paths, 'bundle tree paths/bytes mismatch')
        closure.add(oid)
        closure.update(tree_closure)
    require(set(objects) == closure, 'bundle contains extra objects outside synthetic closure')
    advertised = {line.split(' ', 1)[0] for line in (P / 'fixture-objects.txt').read_text().splitlines()}
    require(advertised == set(objects), 'bundle and repository object closures differ')
    index_count = None
    if '--check-index' in sys.argv:
        index = load('public-index.json')
        for artifact in index['artifacts']:
            path = P / artifact['path']
            require(path.is_file(), 'indexed artifact missing')
            data = path.read_bytes()
            require(len(data) == artifact['bytes'] and sha(data) == artifact['sha256'], 'indexed artifact hash/size mismatch')
        index_count = len(index['artifacts'])
    if '--check-freeze' in sys.argv:
        freeze = load('freeze.json')
        for name, expected_sha in freeze['hashes'].items():
            require(sha((P / name).read_bytes()) == expected_sha, 'freeze sidecar hash mismatch')
        require(freeze['record_hashes'] == actual_records, 'freeze record hashes')
    print(json.dumps({'status': 'PASS', 'proof_script_sha256': sha((P / 'proof.py').read_bytes()),
          'bundle_sha256': ids['bundle_sha256'], 'bundle_objects': len(objects),
          'bundle_object_types': {kind: sum(o['resolved_type'] == kind for o in objects.values())
                                  for kind in ('commit', 'tree', 'blob', 'tag')},
          'complete_owned_synthetic_closure': True, 'bundle_refs': refs,
          'record_hashes': actual_records, 'command_rawlogs_verified': len(commands) * 2,
          'actual_reader_git_commands_verified': len(reader_commands),
          'controller_invocations_verified': len(outcomes),
          'public_index_artifacts_verified': index_count,
          'controller_imported_or_run_by_audit': False}, sort_keys=True))


if __name__ == '__main__':
    main()
