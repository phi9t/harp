from pathlib import Path
import json,os,select
out=Path('.scratch/sqlite-controller/02/registry/repair3');identities={};dispositions=[];roots=set()
def add(pid,start,kind,**extra):
 if pid and start:identities.setdefault((int(pid),str(start)),{'pid':int(pid),'start_ticks':str(start),'kind':kind,**extra})
def visit(value):
 if isinstance(value,dict):
  add(value.get('namespace_init'),value.get('init_start_ticks'),'retained_owned_namespace_identity',handle=value.get('handle'),pid_namespace=value.get('pid_namespace'))
  add(value.get('runtime_pid'),value.get('runtime_start'),'qualified_native_child',pid_namespace=value.get('pid_namespace'))
  for child in value.values():visit(child)
 elif isinstance(value,list):
  for child in value:visit(child)
for logfile in sorted(out.glob('*.log')):
 for line in logfile.read_text(errors='replace').splitlines():
  pos=line.find('{');array=line.find('[')
  if line.startswith('owned_fixture_resources '):
   for record in json.loads(line.split(' ',1)[1]):
    stat=record['start_stat'].rsplit(') ',1)[1].split();add(record['pid'],stat[19],'fixture_owned_service',name=record['name'])
    for arg in record['argv']:
     if arg.startswith('/tmp/cnr-'):roots.add('/'.join(arg.split('/')[:3]))
  if line.startswith('owned_fixture_namespaces '):
   records=json.loads(line.split(' ',1)[1]);dispositions.extend({'log':logfile.name,**record} for record in records)
   for record in records:add(record['pid'],record['start_ticks'],'retained_owned_namespace_identity',handle=record['execution_handle'],pid_namespace=record['pid_namespace'])
  if pos>=0:
   try:visit(json.JSONDecoder().raw_decode(line[pos:])[0])
   except (ValueError,KeyError):pass
observations=[]
for (pid,start),identity in sorted(identities.items()):
 record=dict(identity)
 try:
  fd=os.pidfd_open(pid)
  try:
   actual=Path('/proc/'+str(pid)+'/stat').read_text().rsplit(') ',1)[1].split()[19]
   poll=select.poll();poll.register(fd,select.POLLIN)
   record['state']='identity_replaced' if actual!=start else 'whole_process_terminated' if poll.poll(0) else 'live'
  finally:os.close(fd)
 except (FileNotFoundError,ProcessLookupError):record['state']='absent'
 observations.append(record)
report={'read_only_observations':observations,'retained_owned_identities':len(observations),'matching_live':sum(r['state']=='live' for r in observations),'whole_process_disposal_receipts':len(dispositions),'all_disposal_receipts_terminated':all(r['whole_process_terminated'] for r in dispositions),'disposal_receipts':dispositions,'temporary_roots':[{'path':r,'exists':Path(r).exists()} for r in sorted(roots)],'provenance_limit':'Interrupted cargo runs without emitted fixture resource identities are not included or claimed as complete inventories. This read-only audit inspects only positively retained owned identities; no foreign process enumeration/signalling.'}
(out/'gate-resources.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({k:report[k] for k in ['retained_owned_identities','matching_live','whole_process_disposal_receipts','all_disposal_receipts_terminated']}))
assert report['matching_live']==0 and report['all_disposal_receipts_terminated'] and not any(r['exists'] for r in report['temporary_roots'])
