from pathlib import Path
import hashlib,json,tarfile,datetime,subprocess,sys
name=sys.argv[1]; root=Path('docs/efforts/sqlite-controller/evidence/04/source')
paths=sorted(p for p in Path('crates/sqlite-controller').rglob('*') if p.is_file() and 'target' not in p.parts and '__pycache__' not in p.parts)
paths += [Path('tools/agents/checks.d/70-sqlite-controller.sh')]
files={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in paths}
a=root/(name+'-source.tar.gz')
with tarfile.open(a,'w:gz') as archive:
 for p in paths: archive.add(p,arcname=str(p))
manifest={'captured_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'head':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'files':files,'archive':str(a),'archive_sha256':hashlib.sha256(a.read_bytes()).hexdigest()}
(root/(name+'.json')).write_text(json.dumps(manifest,indent=2)+'\n'); print(name,len(files),manifest['archive_sha256'])
