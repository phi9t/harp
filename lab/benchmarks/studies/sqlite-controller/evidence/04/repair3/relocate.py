"""Real retained-helper relocation reproducer; output only to owned fresh scratch."""
from pathlib import Path
import argparse, hashlib, json, shutil, subprocess, sys, tempfile, time

parser = argparse.ArgumentParser()
parser.add_argument('phase', choices=['setup', 'red', 'green'])
parser.add_argument('--fixture', type=Path)
parser.add_argument('--helper', type=Path)
parser.add_argument('--label')
args = parser.parse_args()
repo = Path.cwd()
evidence = repo / 'docs/efforts/sqlite-controller/evidence/04'
source = evidence / 'repair1'
scratch = repo / '.scratch/sqlite-controller/04/repair3'
digest = lambda data: hashlib.sha256(data).hexdigest()

def manifest(root):
    return {str(p.relative_to(root)): digest(p.read_bytes())
            for p in sorted(root.rglob('*')) if p.is_file()}

def invoke(command):
    child = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    start = Path(f'/proc/{child.pid}/stat').read_text().rsplit(')', 1)[1].split()[19]
    stdout, stderr = child.communicate()
    return subprocess.CompletedProcess(command, child.returncode, stdout, stderr), {
        'pid': child.pid, 'start_ticks': start, 'waited': True, 'exit': child.returncode}

if args.phase == 'setup':
    fixture = Path(tempfile.mkdtemp(prefix='relocated-', dir=scratch))
    inputs = fixture / 'inputs'
    inputs.mkdir()
    shutil.copytree(source / 'fixtures', inputs / 'fixtures')
    for name in ['observations.json', 'bounded-process-observations.json']:
        shutil.copyfile(source / name, inputs / name)
    retention = json.loads((source / 'retention.json').read_text())
    sentinel = fixture / 'nonexistent-original-checkout'
    assert not sentinel.exists()
    for entry in retention:
        for key in ['original', 'retained']:
            entry[key] = str(sentinel / key / Path(entry[key]).name)
    (inputs / 'retention.json').write_text(json.dumps(retention, indent=2) + '\n')
    (fixture / 'input-manifest.json').write_text(json.dumps(manifest(inputs), indent=2) + '\n')
    print(fixture)
    sys.exit(0)

fixture = args.fixture.resolve()
assert fixture.is_relative_to(scratch.resolve()) and fixture.is_dir()
inputs = fixture / 'inputs'
before = manifest(inputs)
assert before == json.loads((fixture / 'input-manifest.json').read_text())
helper = inputs / 'artifact-observations.py'
helper_source = args.helper or source / 'artifact-observations.py'
shutil.copyfile(helper_source, helper)
label = args.label or args.phase
assert label and Path(label).name == label
output = fixture / (label + '-output.json')
receipt_path = fixture / (label + '-receipt.json')
assert not output.exists()
assert not receipt_path.exists()
started = time.time()
result, process = invoke([sys.executable, str(helper), '--output', str(output)])
receipt = {'phase': args.phase, 'label': label, 'helper_sha256': digest(helper.read_bytes()),
           'driver_sha256': digest(Path(__file__).read_bytes()), 'command': result.args,
           'exit': result.returncode, 'stdout': result.stdout.decode(), 'stderr': result.stderr.decode(),
           'elapsed_seconds': time.time() - started, 'fixture': str(fixture), 'process': process,
           'input_manifest_sha256': digest((fixture / 'input-manifest.json').read_bytes()),
           'original_sentinel_nonexistent': not (fixture / 'nonexistent-original-checkout').exists()}
helper.unlink()
receipt['inputs_unchanged'] = manifest(inputs) == before
receipt_path.write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps(receipt))
assert receipt['inputs_unchanged'] and receipt['original_sentinel_nonexistent']
assert result.returncode == 0, 'helper cannot read relocated retained fixtures'
records = json.loads(output.read_text())
historical = json.loads((inputs / 'bounded-process-observations.json').read_text())
def content(record):
    return {k: v for k, v in record.items() if k not in ['archive', 'archive_original', 'archive_retained_original']}
assert list(map(content, records)) == list(map(content, historical))
retention = {Path(e['retained']).name: e for e in json.loads((inputs / 'retention.json').read_text())}
for record in records:
    path = Path(record['archive'])
    assert path.parent == inputs / 'fixtures'
    entry = retention[path.name]
    assert record['archive_original'] == entry['original']
    assert record['archive_retained_original'] == entry['retained']
original_output = output.read_bytes()
shutil.copyfile(helper_source, helper)
again, overwrite_process = invoke(result.args)
assert digest(helper.read_bytes()) == receipt['helper_sha256']
assert again.returncode != 0 and b'FileExistsError' in again.stderr and output.read_bytes() == original_output
helper.unlink()
receipt.update({'records': len(records), 'same_historical_content': True,
                'original_paths_provenance_only': True, 'output_sha256': digest(original_output),
                'overwrite_exit': again.returncode, 'overwrite_stderr': again.stderr.decode(),
                'overwrite_bytes_unchanged': True, 'overwrite_process': overwrite_process})
controls = []
for mode in ['duplicate', 'digest']:
    control = fixture / mode
    shutil.copytree(inputs, control)
    values = json.loads((control / 'retention.json').read_text())
    archive = next(e for e in values if e['retained'].endswith('.tar.gz'))
    if mode == 'duplicate':
        values.append(dict(archive))
    else:
        archive['sha256'] = '0' * 64
    (control / 'retention.json').write_text(json.dumps(values, indent=2) + '\n')
    shutil.copyfile(source / 'artifact-observations.py', control / 'artifact-observations.py')
    target = fixture / (mode + '-output.json')
    command = [sys.executable, str(control / 'artifact-observations.py'), '--output', str(target)]
    rejected, control_process = invoke(command)
    assert rejected.returncode != 0 and not target.exists()
    controls.append({'mode': mode, 'command': command, 'exit': rejected.returncode,
                     'stderr': rejected.stderr.decode(), 'output_absent': True, 'process': control_process})
receipt['validation_controls'] = controls
receipt['inputs_unchanged'] = manifest(inputs) == before
assert receipt['inputs_unchanged']
receipt_path.write_text(json.dumps(receipt, indent=2) + '\n')
print('actual_relocated_retained_helper_PASS', len(records), 'records; overwrite/duplicate/digest refused')
