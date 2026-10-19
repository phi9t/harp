"""Audit actual retained helper receipts and immutable bytes; never execute fixtures."""
from pathlib import Path
import gzip, hashlib, json, tarfile

root = Path(__file__).resolve().parent
repo = Path.cwd()
digest = lambda data: hashlib.sha256(data).hexdigest()
retention = json.loads((root / 'retention.json').read_text())
for entry in retention:
    data = (root / entry['retained']).read_bytes()
    assert digest(data) == entry['sha256']
    if 'raw_sha256' in entry:
        assert digest(gzip.decompress(data)) == entry['raw_sha256']
history = json.loads((root / 'historical-before.json').read_text())
changed = [p for p, sha in history['files'].items() if digest((repo / p).read_bytes()) != sha]
assert changed == ['docs/efforts/sqlite-controller/evidence/04/repair1/artifact-observations.py']
assert all(digest((repo / p).read_bytes()) == sha for p, sha in history['product_files'].items())
assert digest((root / 'old-artifact-observations.py').read_bytes()) == history['files'][changed[0]]
archives = 0
members = 0
for path in sorted(root.glob('*.json')):
    value = json.loads(path.read_text())
    if not isinstance(value, dict) or 'archive_sha256' not in value:
        continue
    archive = root / value['archive']
    assert digest(archive.read_bytes()) == value['archive_sha256']
    with tarfile.open(archive) as stream:
        actual = {m.name: digest(stream.extractfile(m).read()) for m in stream if m.isfile()}
    assert actual == value['files']
    archives += 1
    members += len(actual)
pin = json.loads((root / 'fixture-pin.json').read_text())
with tarfile.open(root / pin['archive']) as stream:
    actual = {m.name: digest(stream.extractfile(m).read()) for m in stream if m.isfile()}
assert actual.pop('input-manifest.json') == digest((root / 'input-manifest.json').read_bytes())
assert {p.removeprefix('inputs/'): sha for p, sha in actual.items()} == pin['files']
red = json.loads((root / 'paired-red-receipt.json').read_text())
green = json.loads((root / 'paired-green-receipt.json').read_text())
assert red['driver_sha256'] == green['driver_sha256'] == digest((root / 'relocate.py').read_bytes())
assert red['input_manifest_sha256'] == green['input_manifest_sha256']
assert red['exit'] == 1 and 'FileNotFoundError' in red['stderr']
assert red['helper_sha256'] == digest((root / 'old-artifact-observations.py').read_bytes())
assert green['helper_sha256'] == digest((root.parent / 'repair1/artifact-observations.py').read_bytes())
assert green['exit'] == 0 and green['records'] == 59 and green['same_historical_content']
assert green['overwrite_exit'] != 0 and 'FileExistsError' in green['overwrite_stderr']
assert green['overwrite_bytes_unchanged']
assert digest((root / 'paired-green-output.json').read_bytes()) == green['output_sha256']
for control in green['validation_controls']:
    expected = 'duplicate retained archive' if control['mode'] == 'duplicate' else 'manifest digest/size mismatch'
    assert control['exit'] != 0 and expected in control['stderr'] and control['output_absent']
for name in ['red', 'paired-red', 'paired-green']:
    receipt = json.loads((root / (name + '-receipt.json')).read_text())
    assert receipt['inputs_unchanged'] and receipt['original_sentinel_nonexistent']
processes = [json.loads((root / 'red-receipt.json').read_text())['process'], red['process'],
             green['process'], green['overwrite_process']] + [x['process'] for x in green['validation_controls']]
for process in processes:
    assert process['pid'] > 0 and int(process['start_ticks']) > 0 and process['waited']
    path = Path(f"/proc/{process['pid']}/stat")
    try:
        current = path.read_text().rsplit(')', 1)[1].split()[19]
    except FileNotFoundError:
        continue
    assert current != process['start_ticks'], process
result = {'status': 'author helper controls PASS; root independent helper/branch PASS reported; scoped reviews/committed retention pending',
          'repair_round': 'ticket04 final3/3, ticket02 cap3/3 unchanged',
          'helper_commit': 'f439e9a1bbab958c0ced1a49026cac4207d7db64',
          'product_source': 'e31d57bc9ab739d5b4473c387a9320d176adf07b',
          'product34_unchanged': True, 'historical_unchanged_files': len(history['files']) - 1,
          'retained_records': len(retention), 'raw_logs': sum('raw_sha256' in e for e in retention),
          'source_archives': archives, 'source_members': members, 'fixture_input_members': len(actual),
          'records_matching_historical_content': green['records'], 'explicit_owned_process_identities': len(processes),
          'matching_live': 0, 'unknown_process_reads': 0,
          'matched_driver_and_inputs': True, 'all_artifact_source_fixture_hashes_verified': True,
          'independent_root_checkpoint': 'Root reported own relocated89-archive old/new helper RED1/GREEN0,59 identical-content/provenance records, FileExistsError overwrite refusal, unchanged inputs/output/product34; exact helper branch649 PASS. Scoped Standards/Spec and committed retention pending.',
          'no_product_rust_gate_repeat': True, 'acceptance_withheld': True}
(root / 'verification.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
