import argparse
import hashlib
import json
import pathlib
import subprocess
import tarfile
import time
from datetime import datetime, timezone

parser = argparse.ArgumentParser()
parser.add_argument('--repo', required=True)
parser.add_argument('--head', required=True)
parser.add_argument('--output', required=True)
args = parser.parse_args()
repo = pathlib.Path(args.repo).resolve()
out = pathlib.Path(args.output).resolve()
out.mkdir(parents=True, exist_ok=False)
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip()
if head != args.head:
    raise SystemExit('verification HEAD mismatch')
subprocess.run(['git', 'diff', '--quiet', 'HEAD', '--'], cwd=repo, check=True)
paths = ['crates/sqlite-controller', 'tools/agents/checks.d/70-sqlite-controller.sh']
untracked = subprocess.check_output(['git', 'ls-files', '--others', '--exclude-standard', '--', *paths], cwd=repo, text=True)
if untracked:
    raise SystemExit('untracked source is not a frozen source commit')
files = subprocess.check_output(['git', 'ls-files', '--', *paths], cwd=repo, text=True).splitlines()

def hashes():
    return {name: hashlib.sha256((repo / name).read_bytes()).hexdigest() for name in files}

initial = hashes()
archive = out / 'source-before.tar.gz'
with tarfile.open(archive, 'w:gz') as bundle:
    for name in files:
        bundle.add(repo / name, arcname=name, recursive=False)
manifest = 'crates/sqlite-controller/Cargo.toml'
base = ['cargo', 'test', '--locked', '--offline', '--manifest-path', manifest]
checks = [
    ('affected', base + ['--features', 'fault-injection', '--test', 'git', '--test', 'kata', '--test', 'process', '--test', 'public', '--test', 'runtime', '--', '--nocapture', '--test-threads=2']),
    ('native-ticket04', base + ['--features', 'fault-injection', '--test', 'codex', 'ticket04_', '--', '--nocapture', '--test-threads=2']),
    ('default-git-kata', base + ['--test', 'git', '--test', 'kata', '--', '--nocapture', '--test-threads=2']),
    ('fmt', ['cargo', 'fmt', '--manifest-path', manifest, '--', '--check']),
    ('clippy', ['cargo', 'clippy', '--locked', '--offline', '--manifest-path', manifest, '--all-targets', '--features', 'fault-injection', '--', '-D', 'warnings']),
    ('build', ['cargo', 'build', '--locked', '--offline', '--manifest-path', manifest]),
]
record = {
    'started_at': datetime.now(timezone.utc).isoformat(),
    'head': head,
    'repo': str(repo),
    'source_sha256': initial,
    'source_archive_sha256': hashlib.sha256(archive.read_bytes()).hexdigest(),
    'runner_sha256': hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),
    'planned_checks': [{'name': label, 'argv': argv} for label, argv in checks],
    'checks': [],
    'scope': 'Current affected Git/Kata/process/public/runtime FI gate plus four ticket04 native topology/cleanup controls; unchanged native source/fixture proof inherited from root82 repair1 full gate, default Git/Kata feature checks, fmt, strict Clippy, normal build. Deterministic actual local fixture evidence only; not live Codex qualification or final whole repository gate.',
}
(out / 'before.json').write_text(json.dumps(record, indent=2) + '\n')
exit_code = 0
for label, argv in checks:
    log = out / ('root-' + label + '.log')
    start = time.monotonic()
    with log.open('wb') as stream:
        result = subprocess.run(argv, cwd=repo, stdout=stream, stderr=subprocess.STDOUT)
    record['checks'].append({
        'name': label, 'argv': argv, 'returncode': result.returncode,
        'seconds': time.monotonic() - start, 'log': str(log),
        'sha256': hashlib.sha256(log.read_bytes()).hexdigest(),
    })
    (out / 'gate.json').write_text(json.dumps(record, indent=2) + '\n')
    print(label, result.returncode, flush=True)
    if result.returncode:
        exit_code = result.returncode
        break
record['source_after_sha256'] = hashes()
record['head_after'] = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=repo, text=True).strip()
record['source_unchanged'] = initial == record['source_after_sha256'] and head == record['head_after']
record['source_diff_clean'] = subprocess.run(['git', 'diff', '--quiet', 'HEAD', '--'], cwd=repo).returncode == 0
record['verdict'] = 'PASS' if not exit_code and record['source_unchanged'] and record['source_diff_clean'] else 'FAIL'
record['finished_at'] = datetime.now(timezone.utc).isoformat()
(out / 'gate.json').write_text(json.dumps(record, indent=2) + '\n')
if record['verdict'] != 'PASS':
    raise SystemExit(exit_code or 1)
