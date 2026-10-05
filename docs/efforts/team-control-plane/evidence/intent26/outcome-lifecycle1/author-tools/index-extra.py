import json,pathlib,hashlib
root=pathlib.Path.cwd();ev=root/'docs/efforts/team-control-plane/evidence/intent26/outcome-lifecycle1';p=ev/'observation-index.json';rows=json.loads(p.read_text());seen={(x['run'],x['raw_line']) for x in rows};extra=[]
for f in ev.glob('*.log'):
 for n,line in enumerate(f.read_text().splitlines(),1):
  pos=line.find('{')
  if pos<0:continue
  try:j=json.loads(line[pos:])
  except ValueError:continue
  label=line[:pos].split(' ... ')[-1].strip()
  if (f.stem,n) not in seen:
   rows.append({'run':f.stem,'raw_line':n,'label':label,'fields':sorted(j.keys())})
  if label=='measured_gate_bytes' and f.stem=='green-compatibility-freeze':
   b=bytes(j['artifact_bytes']);digest=hashlib.sha256(b).hexdigest();assert digest==j['logs'];(ev/'measured-artifacts'/digest).write_bytes(b)
   payload=json.loads(b);output=payload['output'].encode();logs=hashlib.sha256(output).hexdigest();assert logs==payload['trusted_monitor_result']['logs'];(ev/'measured-artifacts'/logs).write_bytes(output)
   extra.append({'run':f.stem,'raw_line':n,'execution':j['execution'],'artifact':{'path':str((ev/'measured-artifacts'/digest).relative_to(root)),'sha256':digest,'bytes':len(b)},'measured_log':{'path':str((ev/'measured-artifacts'/logs).relative_to(root)),'sha256':logs,'bytes':len(output)},'trusted_monitor_result':payload['trusted_monitor_result']})
rows.sort(key=lambda x:(x['run'],x['raw_line']));p.write_text(json.dumps(rows,indent=2)+'\n');(ev/'production-artifact-bindings.json').write_text(json.dumps(extra,indent=2)+'\n')
print(json.dumps({'indexed':len(rows),'production_bindings':len(extra)}))
