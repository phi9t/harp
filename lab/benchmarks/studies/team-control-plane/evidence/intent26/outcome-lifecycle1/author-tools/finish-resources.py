import pathlib,subprocess,json,hashlib,gzip,os,time,re
root=pathlib.Path.cwd();ev=root/'docs/efforts/team-control-plane/evidence/intent26/outcome-lifecycle1';scratch=root/'.scratch/team-control-plane/outcome-lifecycle1';container='corenius-intent26-outcome-lifecycle1-pg'
def save(name,b):
 p=ev/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b);return {'path':str(p.relative_to(root)),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def write(name,v):return save(name,(json.dumps(v,indent=2)+'\n').encode())
invpath=ev/'retained-postgres-inventory.json';rawinv=invpath.read_bytes();inventory=json.loads(rawinv);schemas=[x['schema'] for x in inventory['schemas']];dumps=[]
for i in range(0,len(schemas),10):
 selected=schemas[i:i+10];cmd=['docker','exec',container,'pg_dump','-U','probe','-d','probe','--no-owner','--no-privileges']
 for schema in selected:cmd+=['--schema',schema]
 result=subprocess.run(cmd,capture_output=True,timeout=120)
 if result.returncode:
  save(f'retained-postgres/batch-{i//10:02}.failure.log',result.stdout+result.stderr);raise RuntimeError('bounded dump failure')
 rawpath=scratch/f'retained-postgres-{i//10:02}.raw.sql';rawpath.write_bytes(result.stdout)
 rec=save(f'retained-postgres/batch-{i//10:02}.sql.gz',gzip.compress(result.stdout,mtime=0));rec.update(raw_sha256=hashlib.sha256(result.stdout).hexdigest(),raw_bytes=len(result.stdout),raw_path=str(rawpath),schemas=selected,command=cmd,exit_code=0);dumps.append(rec)
write('retained-postgres-dumps.json',{'scope':'all137 owned retained fixture schemas; stable after tests, no concurrent mutation, per-batch pg_dump snapshots rather than one all-table transaction','initial_whole_dump_failure':'resource-collection.log: max_locks_per_transaction; no config/row mutation; failure retained','batches':dumps})
roots=set(re.findall(r'/data02/home/philip.yang/devx/tmp/kernel16-[0-9a-f-]+',rawinv.decode()))
for log in ev.glob('*.log'):roots.update(re.findall(r'/data02/home/philip.yang/devx/tmp/kernel16-[0-9a-f-]+',log.read_text(errors='replace')))
processes=[]
for proc in pathlib.Path('/proc').iterdir():
 if not proc.name.isdigit():continue
 try:argv=(proc/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace');cwd=os.readlink(proc/'cwd')
 except (OSError,PermissionError):continue
 if any(r in argv or cwd==r or cwd.startswith(r+'/') for r in roots):processes.append({'pid':int(proc.name),'command':argv,'cwd':cwd})
root_states=[]
for r in sorted(roots):
 p=pathlib.Path(r);root_states.append({'path':r,'exists':p.exists(),'turn_states':[{'path':str(q),'bytes':q.read_text()} for q in p.rglob('turn.json')] if p.exists() else []})
inspection=json.loads(subprocess.check_output(['docker','inspect',container]))[0];selected={k:inspection[k] for k in ['Id','Name','Image','State','Mounts','NetworkSettings']}
write('resource-prestop.json',{'observed_at':time.time(),'postgres':selected,'inventory':{'path':str(invpath.relative_to(root)),'sha256':hashlib.sha256(rawinv).hexdigest(),'bytes':len(rawinv)},'retained_dump_batches':len(dumps),'recorded_roots':root_states,'matching_live_processes':processes,'limits':'Read-only enumeration; missing PID/path is not independent measured Stop evidence. See measured artifact bindings and raw supervisor observations.'})
print(json.dumps({'retained_schemas':len(schemas),'dump_batches':len(dumps),'dump_raw_bytes':sum(x['raw_bytes'] for x in dumps),'compressed_bytes':sum(x['bytes'] for x in dumps),'known_roots':len(roots),'matching_live_processes':processes}))
