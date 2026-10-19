from pathlib import Path
import subprocess,hashlib,json,tarfile,time,datetime
root=Path.cwd();out=root/'.scratch/sqlite-controller/02/registry/repair1'
files=sorted(p for p in (root/'crates/sqlite-controller').rglob('*') if p.is_file() and not {'target','__pycache__'}&set(p.relative_to(root).parts))
inventory={str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
(out/'gate-source.json').write_text(json.dumps(inventory,indent=2)+'\n')
with tarfile.open(out/'gate-source.tar.gz','w:gz') as archive:
 for p in files:archive.add(p,arcname=str(p.relative_to(root)))
commands=[('gate-tests.log',['cargo','test','--locked','--offline','--manifest-path','crates/sqlite-controller/Cargo.toml','--features','fault-injection','--','--nocapture','--test-threads=2']),('gate-fmt.log',['cargo','fmt','--manifest-path','crates/sqlite-controller/Cargo.toml','--check']),('gate-clippy.log',['cargo','clippy','--locked','--offline','--manifest-path','crates/sqlite-controller/Cargo.toml','--all-targets','--features','fault-injection','--','-D','warnings']),('gate-normal-build.log',['cargo','build','--locked','--offline','--manifest-path','crates/sqlite-controller/Cargo.toml'])]
report={'source_base':'2dcc4ec34bf7bdd74a6759bcd2f743361083e01c','classification':'MODEL_FREE_ACTUAL_PROTOCOL; no paid/live inference','commands':[]}
for name,command in commands:
 start=time.monotonic();stamp=datetime.datetime.now(datetime.timezone.utc).isoformat()
 with (out/name).open('wb') as logfile: result=subprocess.run(command,stdout=logfile,stderr=subprocess.STDOUT)
 report['commands'].append({'command':' '.join(command),'exit':result.returncode,'log':name,'seconds':round(time.monotonic()-start,2),'started_at':stamp})
 (out/'gate-commands.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report['commands'][-1]),flush=True)
 if result.returncode:raise SystemExit(result.returncode)
report['tested_source_unchanged']=all(hashlib.sha256((root/path).read_bytes()).hexdigest()==digest for path,digest in inventory.items())
(out/'gate-commands.json').write_text(json.dumps(report,indent=2)+'\n')
assert report['tested_source_unchanged']
