from pathlib import Path
import hashlib,json,tarfile,datetime,subprocess,sys
root=Path('.scratch/sqlite-controller/03')
name=sys.argv[1]
paths=sorted(p for p in Path('crates/sqlite-controller').rglob('*') if p.is_file() and 'target' not in p.parts and '__pycache__' not in p.parts)
files={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
with tarfile.open(root/(name+'-source.tar.gz'),'w:gz') as archive:
    for p in paths: archive.add(p,arcname=str(p))
manifest={'captured_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'files':files,'source_archive_sha256':hashlib.sha256((root/(name+'-source.tar.gz')).read_bytes()).hexdigest(),'public_test_sha256':files['crates/sqlite-controller/tests/support/kata_protocol.py']}
(root/(name+'.json')).write_text(json.dumps(manifest,indent=2)+'\n')
print(name,len(files),manifest['public_test_sha256'])
