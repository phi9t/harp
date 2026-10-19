"""Seal complete unchanged product source plus exact helper and real driver."""
from pathlib import Path
import hashlib, json, subprocess, sys, tarfile
root = Path('docs/efforts/sqlite-controller/evidence/04/repair3')
name = sys.argv[1]
frozen = json.loads((root.parent / 'repair2/source/frozen-source.json').read_text())
paths = [Path(p) for p in frozen['files']] + [root.parent / 'repair1/artifact-observations.py', root / 'relocate.py', root / 'old-artifact-observations.py']
files = {str(p): hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
assert all(files[p] == sha for p, sha in frozen['files'].items())
archive = root / (name + '-source.tar.gz')
assert not archive.exists()
with tarfile.open(archive, 'w:gz') as stream:
    for path in paths:
        stream.add(path, arcname=str(path))
record = {'head': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
          'product_source': frozen['head'], 'files': files, 'product34_unchanged': True,
          'archive': archive.name, 'archive_sha256': hashlib.sha256(archive.read_bytes()).hexdigest(),
          'fixture_pin_sha256': hashlib.sha256((root / 'fixture-pin.json').read_bytes()).hexdigest()}
(root / (name + '.json')).write_text(json.dumps(record, indent=2) + '\n')
print(name, len(files), record['archive_sha256'])
