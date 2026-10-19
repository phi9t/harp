import pathlib,subprocess,json,hashlib,gzip,os,time,re
root=pathlib.Path.cwd();ev=root/'docs/efforts/team-control-plane/evidence/intent26/outcome-lifecycle1';scratch=root/'.scratch/team-control-plane/outcome-lifecycle1';container='corenius-intent26-outcome-lifecycle1-pg'
def psql(sql):
 p=subprocess.run(['docker','exec',container,'psql','-U','probe','-d','probe','-At','-v','ON_ERROR_STOP=1','-c',sql],capture_output=True);assert p.returncode==0,p.stderr.decode();return p.stdout.decode()
def save(name,b):
 p=ev/name;p.write_bytes(b);return {'path':str(p.relative_to(root)),'sha256':hashlib.sha256(b).hexdigest(),'bytes':len(b)}
def write(name,v):return save(name,(json.dumps(v,indent=2)+'\n').encode())
# This dedicated container contains only this author's own test fixtures.
schemas=psql("SELECT nspname FROM pg_namespace WHERE nspname NOT LIKE 'pg_%' AND nspname NOT IN ('information_schema','public') ORDER BY nspname").splitlines()
rows=[];roots=set()
for schema in schemas:
 assert schema.replace('_','').isalnum()
 tables=psql(f"SELECT tablename FROM pg_tables WHERE schemaname='{schema}' ORDER BY tablename").splitlines();entry={'schema':schema,'table_counts':{},'selected_retained_rows':{}}
 for table in tables:entry['table_counts'][table]=int(psql(f'SELECT count(*) FROM "{schema}"."{table}"'))
 for table in ['tasks','attempts','reservations','episode_executions','supervised_workspaces','gateway_operations','episode_runtime_operations','contract_current','contract_outcomes']:
  if table not in tables:continue
  raw=psql(f'SELECT row_to_json(t) FROM "{schema}"."{table}" t ORDER BY row_to_json(t)::text')
  data=[json.loads(x) for x in raw.splitlines()];entry['selected_retained_rows'][table]=data
  roots.update(re.findall(r'/data02/home/philip.yang/devx/tmp/kernel16-[0-9a-f-]+',raw))
  if table=='supervised_workspaces':
   for row in data:
    manifest=row.get('manifest',{});path=manifest.get('path')
    if path:
     # Known assembled fixture roots are kernel16-* ancestors.
     for ancestor in pathlib.Path(path).parents:
      if ancestor.name.startswith('kernel16-'):roots.add(str(ancestor))
 rows.append(entry)
# Add recorded fixture roots, including those already disposed.
for log in ev.glob('*.log'):
 for line in log.read_text(errors='replace').splitlines():
  pos=line.find('{')
  if pos<0:continue
  try:j=json.loads(line[pos:])
  except ValueError:continue
  for key in ['root','fixture_root']:
   value=j.get(key)
   if isinstance(value,str) and '/kernel16-' in value:roots.add(value)
inv=write('retained-postgres-inventory.json',{'container':container,'schemas':rows,'note':'Final read-only observation before stopping dedicated PostgreSQL. Remaining rows include failed/unknown fixtures; no schema or row cleanup performed here.'})
dump=subprocess.run(['docker','exec',container,'pg_dump','-U','probe','-d','probe','--no-owner','--no-privileges'],capture_output=True);assert dump.returncode==0,dump.stderr.decode()
rawpath=scratch/'retained-postgres.raw.sql';rawpath.write_bytes(dump.stdout)
compressed=gzip.compress(dump.stdout,mtime=0);dump_record=save('retained-postgres.sql.gz',compressed);dump_record['raw_sha256']=hashlib.sha256(dump.stdout).hexdigest();dump_record['raw_bytes']=len(dump.stdout);dump_record['raw_path']=str(rawpath)
processes=[]
for proc in pathlib.Path('/proc').iterdir():
 if not proc.name.isdigit():continue
 try:
  argv=(proc/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace');cwd=os.readlink(proc/'cwd')
 except (OSError,PermissionError):continue
 if any(r in argv or cwd==r or cwd.startswith(r+'/') for r in roots):processes.append({'pid':int(proc.name),'command':argv,'cwd':cwd})
root_states=[]
for r in sorted(roots):
 p=pathlib.Path(r);root_states.append({'path':r,'exists':p.exists(),'turn_states':[{'path':str(q),'bytes':q.read_text()} for q in p.rglob('turn.json')] if p.exists() else []})
inspection=json.loads(subprocess.check_output(['docker','inspect',container]))[0]
selected={k:inspection[k] for k in ['Id','Name','Image','State','Mounts','NetworkSettings']}
write('resource-prestop.json',{'observed_at':time.time(),'postgres':selected,'inventory':inv,'retained_database_dump':dump_record,'recorded_roots':root_states,'matching_live_processes':processes,'limits':'Read-only enumeration; a missing PID/path is not independent measured Stop evidence. See measured-artifact-bindings and raw supervisor observations for that proof.'})
print(json.dumps({'retained_schemas':len(schemas),'dump_raw_bytes':len(dump.stdout),'compressed_bytes':len(compressed),'known_roots':len(roots),'matching_live_processes':len(processes)}))
