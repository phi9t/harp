import gzip
import hashlib
import io
import json
import pathlib
import subprocess
import tarfile
from datetime import datetime, timezone

author = pathlib.Path('/data02/home/philip.yang/workspace/corenius-sqlite04-git')
tip = 'f85fcd15a912a59a9af7e4d0da56329f38a5c298'
prefix = 'docs/efforts/sqlite-controller/evidence/04/'
root = pathlib.Path(__file__).resolve().parents[2]
output = root / '.scratch/sqlite-controller/reviews/04/initial/retention-audit.json'

def blob(name):
    return subprocess.check_output(['git', 'show', tip + ':' + name], cwd=author)

def sha(data):
    return hashlib.sha256(data).hexdigest()

retention = json.loads(blob(prefix + 'retention.json'))
observations = json.loads(blob(prefix + 'observations.json'))
identities = {}
terminal_records = 0

def add(rows, namespace=False):
    global terminal_records
    for row in rows:
        pid = row.get('pid')
        ticks = row.get('start_ticks')
        if ticks is None and isinstance(row.get('start_stat'), str):
            ticks = row['start_stat'].rsplit(') ', 1)[1].split()[19]
        if isinstance(pid, int) and ticks is not None:
            identities[(pid, str(ticks))] = True
        if namespace and row.get('whole_process_terminated') is True:
            terminal_records += 1
        if not namespace and row.get('exit') is not None:
            terminal_records += 1

def logs(data):
    for line in data.decode(errors='replace').splitlines():
        for label, namespace in [('owned_fixture_namespaces', True), ('owned_fixture_resources', False)]:
            if line.startswith(label + ' '):
                add(json.loads(line[len(label) + 1:]), namespace)
        if line.startswith('retained_actual_fixture '):
            value = json.loads(line[len('retained_actual_fixture ') :])
            add(value['processes'])
            add(value['namespaces'], True)

raw_logs = 0
for item in retention:
    name = str(pathlib.Path(item['retained']).relative_to(author))
    assert name.startswith(prefix) and '..' not in pathlib.Path(name).parts
    data = blob(name)
    assert sha(data) == item['sha256'], name
    if 'raw_sha256' in item:
        with gzip.GzipFile(fileobj=io.BytesIO(data)) as stream:
            raw = stream.read(128 * 1024 * 1024 + 1)
        assert len(raw) <= 128 * 1024 * 1024
        assert len(raw) == item['bytes'] and sha(raw) == item['raw_sha256'], name
        logs(raw)
        raw_logs += 1
    else:
        assert len(data) == item['bytes'], name

archives = 0
source_files = 0
names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', tip, '--', prefix + 'source'], cwd=author, text=True).splitlines()
for name in names:
    if not name.endswith('.json'):
        continue
    manifest = json.loads(blob(name))
    if not isinstance(manifest, dict) or not all(key in manifest for key in ['files', 'archive', 'archive_sha256']):
        continue
    data = blob(manifest['archive'])
    assert sha(data) == manifest['archive_sha256'], name
    with tarfile.open(fileobj=io.BytesIO(data), mode='r:gz') as archive:
        members = archive.getmembers()
        assert len(members) == len(manifest['files']), name
        for member in members:
            assert member.isfile() and member.name in manifest['files'], member.name
            assert sha(archive.extractfile(member).read()) == manifest['files'][member.name], member.name
    archives += 1
    source_files += len(manifest['files'])

for item in observations:
    if item['observation'] == 'retained_actual_fixture':
        add(item['value']['processes'])
        add(item['value']['namespaces'], True)

root_log = root / '.scratch/sqlite-controller/reviews/04/initial/root-gate/root-assembled.log'
logs(root_log.read_bytes())
live = []
unknown = []
for pid, ticks in identities:
    try:
        current = pathlib.Path('/proc', str(pid), 'stat').read_text().rsplit(') ', 1)[1].split()[19]
    except (FileNotFoundError, ProcessLookupError):
        continue
    except OSError:
        unknown.append({'pid': pid, 'start_ticks': ticks})
        continue
    if current == ticks:
        live.append({'pid': pid, 'start_ticks': ticks})

report = {
    'captured_at': datetime.now(timezone.utc).isoformat(),
    'source': '2b1887b8c8c6e0ea806487271ae77fc3b743f63d', 'evidence_tip': tip,
    'retention_records_verified': len(retention), 'raw_logs_verified': raw_logs,
    'source_archives_verified': archives, 'archived_source_files_verified': source_files,
    'retained_fixture_identities_checked': len(identities),
    'recorded_terminal_receipts_including_duplicates': terminal_records,
    'matching_current_kernel_identities': live, 'unresolved_kernel_reads': unknown,
    'scope': 'Immutable initial04 evidence plus root initial gate. Only explicit fixture process/namespace receipts scanned; exact PID/start checks, no signals or broad process discovery. Historical unknown domain rows remain unknown. Does not qualify repair1 or live Codex.',
    'runner_sha256': sha(pathlib.Path(__file__).read_bytes()),
    'verdict': 'PASS' if not live and not unknown else 'ATTENTION_REQUIRED',
}
output.write_text(json.dumps(report, indent=2) + '\n')
print(json.dumps(report))
