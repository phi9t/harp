import pathlib,hashlib,json,tarfile,subprocess,time,datetime,sys
root=pathlib.Path.cwd();out=root/'.scratch/sqlite-controller/02/registry/repair3';name=sys.argv[1];args=sys.argv[2:]
files=sorted(p for p in (root/'crates/sqlite-controller').rglob('*') if p.is_file() and 'target' not in p.parts and '__pycache__' not in p.parts)
manifest={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in files};(out/(name+'-source.json')).write_text(json.dumps(manifest,indent=2)+'\n')
with tarfile.open(out/(name+'-source.tar.gz'),'w:gz') as tar:
 for p in files:tar.add(p,arcname=str(p.relative_to(root)))
start=time.monotonic();stamp=datetime.datetime.now(datetime.timezone.utc).isoformat()
with open(out/(name+'.log'),'w') as log:result=subprocess.run(args,stdout=log,stderr=subprocess.STDOUT)
record={'command':args,'exit':result.returncode,'seconds':round(time.monotonic()-start,2),'started_at':stamp,'source_unchanged':all(hashlib.sha256((root/p).read_bytes()).hexdigest()==h for p,h in manifest.items())}
(out/(name+'-command.json')).write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record),flush=True)
