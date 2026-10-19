"""Check this repair's retained bytes and complete source archives, read-only."""
from pathlib import Path
import gzip, hashlib, json, tarfile

root = Path(__file__).resolve().parent
digest = lambda data: hashlib.sha256(data).hexdigest()
retained = json.loads((root / 'retention.json').read_text())
for entry in retained:
    path = root / ('raw' if 'raw_sha256' in entry else 'fixtures') / Path(entry['retained']).name
    data = path.read_bytes()
    assert digest(data) == entry['sha256'], path
    if 'raw_sha256' in entry:
        raw = gzip.decompress(data)
        assert digest(raw) == entry['raw_sha256'] and len(raw) == entry['bytes'], path
    else:
        assert len(data) == entry['bytes'], path
members = 0
manifests = []
for path in sorted((root / 'source').glob('*.json')):
    value = json.loads(path.read_text())
    if 'files' not in value or 'archive_sha256' not in value:
        continue
    archive = root / 'source' / Path(value['archive']).name
    assert digest(archive.read_bytes()) == value['archive_sha256'], archive
    found = {}
    with tarfile.open(archive) as stream:
        for member in stream:
            if member.isfile():
                assert member.name not in found, member.name
                found[member.name] = digest(stream.extractfile(member).read())
    assert found == value['files'], path
    manifests.append(path.name)
    members += len(found)
pairs = json.loads((root / 'source-pairs.json').read_text())
assert all(value['same_files'] and value['source_files'] == 34 for value in pairs)
result = {'retained_records': len(retained), 'raw_logs': sum('raw_sha256' in x for x in retained),
          'source_archives': len(manifests), 'source_members': members,
          'source_pairs': len(pairs), 'all_pairs_identical': True,
          'all_artifact_and_source_hashes_verified': True}
(root / 'retention-check.json').write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(result))
