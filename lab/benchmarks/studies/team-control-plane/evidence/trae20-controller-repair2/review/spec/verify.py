import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess

author = Path('/tmp/corenius-trae20-budget-repair-author')
policy = Path('/data02/home/philip.yang/workspace/corenius-codex-reference-design')
review = Path(__file__).parent
relative2 = Path('.scratch/team-control-plane/trae20-budget-repair/controller-repair2-20261003-v1')
relative1 = Path('.scratch/team-control-plane/trae20-budget-repair/controller-repair1-20261003-v1')
base = '4b3d2634d85b62f21d97ab9c41b0d50b282d89ae'

def sha(data):
    return hashlib.sha256(data).hexdigest()

def record(path):
    data = path.read_bytes()
    return {'path': str(path), 'sha256': sha(data), 'bytes': len(data)}

checks = []
indexes = {}
for iteration, relative in ((2, relative2), (1, relative1)):
    path = author/relative/'freeze/public-index.json'
    data = path.read_bytes()
    value = json.loads(data)
    indexes[str(iteration)] = {**record(path), 'record_count': len(value['records'])}
    for item in value['records']:
        source = author/item['path']
        if iteration == 1 and item['path'] in ('tools/probes/trae_case_control.py', 'tools/probes/test_trae_case_control.py'):
            source = author/relative2/'preimages'/Path(item['path']).name
        actual = record(source)
        assert actual['sha256'] == item['sha256'], (iteration, source)
        assert actual['bytes'] == item['bytes'], (iteration, source)
        checks.append({'index_iteration': iteration, 'indexed_path': item['path'], 'checked_path': str(source), 'sha256': actual['sha256'], 'bytes': actual['bytes'], 'match': True})
assert indexes['2']['sha256'] == 'ce51e01dcdaee0d0ab8788b2327899679fde66413bc534b635d6d60f1c970523'
assert indexes['1']['sha256'] == '7900c6b77da61c09286061d3a939030722451d6fe09635ad462d07e86e535771'
index2 = json.loads((author/relative2/'freeze/public-index.json').read_bytes())
historical = []
for path, expected in index2['unchanged_historical_sources'].items():
    item = record(author/path)
    assert item['sha256'] == expected
    historical.append(item)

def reconstruct(patch_path, mode):
    # Strict unified-diff application: no offsets, fuzz, or Git mutation.
    lines = patch_path.read_bytes().splitlines(keepends=True)
    results = []
    i = 0
    while i < len(lines):
        if not lines[i].startswith(b'diff --git '):
            i += 1
            continue
        i += 1
        while not lines[i].startswith(b'--- '):
            i += 1
        old = lines[i][4:].strip().decode()
        new = lines[i+1][4:].strip().decode()
        target = new.removeprefix('b/')
        assert target in index2['source_paths'], target
        if mode == 'base':
            assert old == '/dev/null'
            absent = subprocess.run(['git', '-C', str(author), 'cat-file', '-e', base+':'+target], capture_output=True)
            assert absent.returncode != 0, target
            original = b''
        else:
            original = (author/relative2/'preimages'/Path(target).name).read_bytes()
        oldlines = original.splitlines(keepends=True)
        cursor = 0
        output = []
        i += 2
        while i < len(lines) and not lines[i].startswith(b'diff --git '):
            match = re.fullmatch(rb'@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@[^\n]*\n', lines[i])
            assert match, (patch_path, lines[i])
            oldstart = int(match[1]); oldcount = int(match[2] or b'1')
            newcount = int(match[4] or b'1')
            position = oldstart-1 if oldstart else 0
            assert position >= cursor
            output.extend(oldlines[cursor:position]); cursor = position
            removed = added = 0
            i += 1
            while i < len(lines) and not lines[i].startswith((b'@@ ', b'diff --git ')):
                kind, body = lines[i][:1], lines[i][1:]
                assert kind in (b' ', b'+', b'-'), lines[i]
                if kind in (b' ', b'-'):
                    assert cursor < len(oldlines) and oldlines[cursor] == body
                    cursor += 1; removed += 1
                if kind in (b' ', b'+'):
                    output.append(body); added += 1
                i += 1
            assert removed == oldcount and added == newcount
        output.extend(oldlines[cursor:])
        reconstructed = b''.join(output)
        assert reconstructed == (author/target).read_bytes(), target
        destination = review/(mode+'-reconstruction')/target
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(reconstructed)
        results.append({'source_path': target, 'preimage_sha256': sha(original), 'reconstructed_sha256': sha(reconstructed), 'exact': True})
    assert len(results) == 2
    return {'patch': record(patch_path), 'files': results}

patches = [reconstruct(author/relative2/'freeze/git-base-full.patch', 'base'), reconstruct(author/relative2/'freeze/repair1-to-repair2.patch', 'repair1')]

commands = []
def tests(folder_name, source, selections):
    root = review/folder_name
    (root/'tools/probes').mkdir(parents=True, exist_ok=True)
    (root/'.scratch').mkdir(exist_ok=True)
    (root/'tools/probes/trae_case_control.py').write_bytes(source.read_bytes())
    (root/'tools/probes/test_trae_case_control.py').write_bytes((author/'tools/probes/test_trae_case_control.py').read_bytes())
    argv = ['/usr/bin/python3', '-m', 'unittest', '-v', *selections]
    env = os.environ.copy(); env['PYTHONDONTWRITEBYTECODE'] = '1'; env['PYTHONPATH'] = 'tools/probes'
    completed = subprocess.run(argv, cwd=root, env=env, capture_output=True)
    command = {'argv': argv, 'cwd': str(root), 'env_controls': {'PYTHONDONTWRITEBYTECODE': '1', 'PYTHONPATH': 'tools/probes'}, 'source': record(root/'tools/probes/trae_case_control.py'), 'tests': record(root/'tools/probes/test_trae_case_control.py'), 'terminal_exit_code': completed.returncode}
    (review/(folder_name+'-command.json')).write_text(json.dumps(command, indent=2)+'\n')
    (review/(folder_name+'-output.txt')).write_bytes(completed.stdout+completed.stderr)
    command['output'] = record(review/(folder_name+'-output.txt'))
    commands.append(command)
    return completed

green = tests('final-fixture', author/'tools/probes/trae_case_control.py', ['test_trae_case_control'])
assert green.returncode == 0
red = tests('repair1-fixture', author/relative2/'preimages/trae_case_control.py', ['test_trae_case_control.DurableControllerBehavior.test_prior_terminal_without_start_retains_new_refusal_and_spends_start'])
assert red.returncode == 1 and b'FAILED (failures=1)' in red.stderr and b'spent reservation must retain new refusal' in red.stderr

# Verify retained root review copies, without mutating their checkout.
root_disposition = json.loads((author/relative2/'freeze/review-inputs/2-root-disposition.json').read_bytes())
prior_reviews = []
for item in root_disposition['records']:
    actual = record(policy/item['retained_path'])
    assert actual['sha256'] == item['sha256'] and actual['bytes'] == item['bytes']
    prior_reviews.append(actual)
spec_paths = [policy/'docs/efforts/team-control-plane/decisions'/name for name in ('20261003-trae-launch-controller-provenance-repair.md', '20261003-trae-controller-review1-repair.md', '20261003-trae-controller-review2-repair.md')]
assert spec_paths[-1].read_bytes() == (author/relative2/'freeze/review-inputs/0-20261003-trae-controller-review2-repair.md').read_bytes()

summary = {'base_read': base, 'recorded_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'indexes': indexes, 'hash_checks': checks, 'historical_sources': historical, 'patch_reconstruction': patches, 'commands': commands, 'prior_review_artifacts': prior_reviews, 'specs': [record(p) for p in spec_paths], 'physical_launch_count': 0, 'mutated_author_or_root_checkout': False, 'limitations': ['Tests use injected committed-byte readers and recording executors only.', 'Independent repair1 RED is a post-review sensitivity reproduction, not proof of the author chronology.', 'Intermediate source for occupied-uncertainty RED is not frozen; its retained log and timing classification do not supply exact source identity.', 'No production Git-positive validation, runtime CLI/executor/deadline/descendant qualification, hostile source/ancestor mutation guarantee, or physical admission.', 'Original chronological gaps, physical result1 PARTIAL, spent budgets and quarantines remain unchanged.']}
(review/'verification.json').write_text(json.dumps(summary, indent=2)+'\n')
print(json.dumps({'indexes': indexes, 'matched_indexed_records': len(checks), 'root_prior_review_artifacts_matched': len(prior_reviews), 'patches_exact': len(patches), 'independent_final_exit': green.returncode, 'independent_exact_preimage_red_exit': red.returncode, 'physical_launches': 0}, indent=2))
