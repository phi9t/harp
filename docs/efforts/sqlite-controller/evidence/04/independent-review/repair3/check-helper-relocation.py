"""Root's real, independently owned relocated retained-helper qualification."""
from pathlib import Path
import datetime, hashlib, json, os, shutil, subprocess, sys, tempfile

os.umask(0o077)
repo = Path('/data02/home/philip.yang/workspace/corenius-codex-sqlite-controller-execution')
old = '0b97408bf8f7cd8663da181425cb35f230af9c01'
tip = sys.argv[1]
prefix = 'docs/efforts/sqlite-controller/evidence/04/repair1/'
helper_name = prefix + 'artifact-observations.py'
sha = lambda b: hashlib.sha256(b).hexdigest()
def git(*args):
    return subprocess.check_output(['git', '-C', str(repo), *args])
def blob(ref, path):
    return git('show', ref + ':' + path)
changed = git('diff', '--name-only', old, tip).decode().splitlines()
assert changed == [helper_name], changed
gate = json.loads((repo/'.scratch/sqlite-controller/reviews/04/repair2/root-gate/gate.json').read_text())
assert len(gate['source_sha256']) == 34
assert all(sha(blob(tip, p)) == digest for p, digest in gate['source_sha256'].items())
scratch = repo/'.scratch/sqlite-controller/reviews/04/repair3'
scratch.mkdir(exist_ok=True)
fixture = Path(tempfile.mkdtemp(prefix='root-relocated-', dir=scratch))
inputs = fixture/'inputs'
inputs.mkdir(); (inputs/'fixtures').mkdir()
retention = json.loads(blob(tip, prefix+'retention.json'))
names = set()
for entry in retention:
    if entry['retained'].endswith('.tar.gz'):
        name = Path(entry['retained']).name
        assert name not in names
        names.add(name)
        data = blob(tip, prefix+'fixtures/'+name)
        assert len(data) == entry['bytes'] and sha(data) == entry['sha256']
        (inputs/'fixtures'/name).write_bytes(data)
    for key in ['original', 'retained']:
        entry[key] = str(fixture/'nonexistent-origin'/key/Path(entry[key]).name)
(inputs/'retention.json').write_text(json.dumps(retention, indent=2)+'\n')
for name in ['observations.json', 'bounded-process-observations.json']:
    (inputs/name).write_bytes(blob(tip, prefix+name))
def manifest():
    return {str(p.relative_to(inputs)):sha(p.read_bytes()) for p in sorted(inputs.rglob('*')) if p.is_file() and p.name != 'artifact-observations.py'}
before = manifest()
helper = inputs/'artifact-observations.py'
def invoke(name, argv):
    child = subprocess.Popen(argv, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    start = Path('/proc/'+str(child.pid)+'/stat').read_text().rsplit(')',1)[1].split()[19]
    try:
        stdout, stderr = child.communicate(timeout=30)
    except subprocess.TimeoutExpired:
        child.kill(); child.communicate()
        raise
    receipt = {'command':argv, 'exit':child.returncode, 'stdout':stdout.decode(), 'stderr':stderr.decode(), 'process':{'pid':child.pid, 'start_ticks':start, 'waited':True, 'exit':child.returncode}, 'helper_sha256':sha(helper.read_bytes())}
    (fixture/(name+'-receipt.json')).write_text(json.dumps(receipt,indent=2)+'\n')
    return receipt
helper.write_bytes(blob(old, helper_name))
red_output = fixture/'red.json'
red = invoke('old', [sys.executable,str(helper),'--output',str(red_output)])
assert red['exit'] != 0 and 'FileNotFoundError' in red['stderr'] and str(fixture/'nonexistent-origin') in red['stderr']
assert not red_output.exists() and manifest() == before
helper.write_bytes(blob(tip, helper_name))
output = fixture/'fresh.json'
green = invoke('corrected', [sys.executable,str(helper),'--output',str(output)])
assert green['exit'] == 0
records = json.loads(output.read_text())
historical = json.loads((inputs/'bounded-process-observations.json').read_text())
def content(row):
    return {k:v for k,v in row.items() if k not in ['archive','archive_original','archive_retained_original']}
assert list(map(content,records)) == list(map(content,historical))
archive_entries = {Path(e['retained']).name:e for e in retention if e['retained'].endswith('.tar.gz')}
for row in records:
    path = Path(row['archive']); assert path.parent == inputs/'fixtures'
    entry = archive_entries[path.name]
    assert row['archive_original'] == entry['original'] and row['archive_retained_original'] == entry['retained']
output_bytes = output.read_bytes()
overwrite = invoke('overwrite', [sys.executable,str(helper),'--output',str(output)])
assert overwrite['exit'] != 0 and 'FileExistsError' in overwrite['stderr']
assert sha(helper.read_bytes()) == sha(blob(tip,helper_name)) and output.read_bytes() == output_bytes
helper.unlink()
assert manifest() == before and not (fixture/'nonexistent-origin').exists()
result = {'captured_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'base':old,'helper_commit':tip,'product_source':'e31d57bc9ab739d5b4473c387a9320d176adf07b','34_product_files_unchanged':True,'only_changed_file':helper_name,'archives':len(names),'records':len(records),'historical_content_equal':True,'original_paths_provenance_only':True,'inputs_unchanged':True,'existing_output_bytes_unchanged':True,'old_exit':red['exit'],'corrected_exit':green['exit'],'overwrite_exit':overwrite['exit'],'overwrite_error':'FileExistsError','old_helper_sha256':sha(blob(old,helper_name)),'corrected_helper_sha256':sha(blob(tip,helper_name)),'output_sha256':sha(output_bytes),'input_manifest':before,'fixture':str(fixture),'runner_sha256':sha(Path(__file__).read_bytes()),'classification':'REAL_LOCAL_RETAINED_HELPER, no model calls or controller/runtime qualification','verdict':'PASS'}
out = scratch/'root-helper-relocation.json'; assert not out.exists()
out.write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({k:result[k] for k in ['helper_commit','archives','records','old_exit','corrected_exit','overwrite_exit','overwrite_error','34_product_files_unchanged','verdict']}))
