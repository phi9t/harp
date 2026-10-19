import pathlib,subprocess,json,hashlib,difflib,re
root=pathlib.Path.cwd();ev=root/'docs/efforts/team-control-plane/evidence/intent26/outcome-lifecycle1'
h=lambda b:hashlib.sha256(b).hexdigest()
def put(path,b):
 p=ev/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b);return {'path':str(p.relative_to(root)),'sha256':h(b),'bytes':len(b)}
def record(path):
 p=root/path;b=p.read_bytes();return {'path':path,'sha256':h(b),'bytes':len(b)}
old=json.loads((ev.parent/'repair3/source-manifest.json').read_text())
paths=sorted(set(x['path'] for x in old['source_changes'])|{'crates/team-control-plane/src/kernel.rs','crates/team-control-plane/tests/support/outcome_lifecycle.rs'})
launch=json.loads((ev/'green-compatibility-freeze.start.json').read_text())
runs=[json.loads(x) for x in (ev/'runs.jsonl').read_text().splitlines()];final=next(x for x in runs if x['name']=='green-compatibility-freeze');assert final['exit_code']==0
assert all(h(pathlib.Path(p).read_bytes())==v for p,v in launch['inputs'].items())
whole=[];delta=[];entries=[]
for p in paths:
 live=(root/p).read_bytes();base_result=subprocess.run(['git','show','9d6b48238c1aea393d300ff73d6a3a971f3b2de0:'+p],capture_output=True);base=base_result.stdout if base_result.returncode==0 else None
 baseline=ev/'rejected-baseline'/p;prior=baseline.read_bytes() if baseline.exists() else base
 # Preserve exact baseline for newly affected files too; absence represented in manifest.
 pre=put(pathlib.Path('delta-preimages')/p,prior) if prior is not None else None
 source=put(pathlib.Path('source-snapshots')/p,live)
 base_record=put(pathlib.Path('whole-base-preimages')/p,base) if base is not None else None
 entries.append({'path':p,'sha256':h(live),'bytes':len(live),'snapshot':source,'base_preimage':base_record,'rejected_repair3_preimage':pre,'changed_from_base':base!=live,'changed_from_rejected_repair3':prior!=live})
 for original,output in [(base,whole),(prior,delta)]:
  if original!=live:output.extend(difflib.unified_diff((original or b'').decode().splitlines(keepends=True),live.decode().splitlines(keepends=True),fromfile='a/'+p if original is not None else '/dev/null',tofile='b/'+p))
whole_rec=put(pathlib.Path('whole-base.patch'),''.join(whole).encode());delta_rec=put(pathlib.Path('rejected-repair3-delta.patch'),''.join(delta).encode())
# Reconstruct both patches in private scratch and compare every final postimage.
recon=[]
for label,prekey,patch in [('whole-base','base_preimage','whole-base.patch'),('rejected-repair3','rejected_repair3_preimage','rejected-repair3-delta.patch')]:
 dest=root/'.scratch/team-control-plane/outcome-lifecycle1'/('reconstruct-'+label);dest.mkdir(parents=True,exist_ok=False)
 for x in entries:
  p=dest/x['path'];p.parent.mkdir(parents=True,exist_ok=True)
  if x[prekey]:p.write_bytes((root/x[prekey]['path']).read_bytes())
 done=subprocess.run(['patch','-p1','--batch','--forward','-i',str(ev/patch)],cwd=dest,capture_output=True)
 log=put(pathlib.Path('reconstruction-'+label+'.log'),done.stdout+done.stderr);assert done.returncode==0
 assert all(h((dest/x['path']).read_bytes())==x['sha256'] for x in entries)
 recon.append({'base':label,'exit_code':done.returncode,'all_postimages_match':True,'log':log})
executables=[]
for match in re.finditer(r'Running [^\n]*\((/tmp/corenius-target-intent26-outcome-lifecycle1/[^\)]+)\)',(ev/'green-compatibility-freeze.log').read_text()):
 p=pathlib.Path(match.group(1));b=p.read_bytes();executables.append({'path':str(p),'sha256':h(b),'bytes':len(b)})
manifest={'schema_version':1,'base':'9d6b48238c1aea393d300ff73d6a3a971f3b2de0','branch':'codex-intent26/outcome-lifecycle1','implementation_committed':False,'scope':'adopted post-architecture allocation1 only; rejected repair3 baseline preserved','source_changes':entries,'whole_base_patch':whole_rec,'rejected_repair3_delta':delta_rec,'reconstruction':recon,'final_run':{'name':final['name'],'log':record(str((ev/(final['name']+'.log')).relative_to(root))),'start':record(str((ev/(final['name']+'.start.json')).relative_to(root))),'exit_code':final['exit_code'],'all_input_sources_match':True,'executable_outputs':executables},'migrations_0008_0009_unchanged_from_rejected_baseline':all(not x['changed_from_rejected_repair3'] for x in entries if '/migrations/' in x['path'])}
assert manifest['migrations_0008_0009_unchanged_from_rejected_baseline']
put(pathlib.Path('source-manifest.json'),(json.dumps(manifest,indent=2)+'\n').encode())
# Index exact raw observations without rewriting logs; materialize measured artifact bytes as observed.
observations=[];artifacts=[]
for run in runs:
 if run['kind']!='test':continue
 path=ev/(run['name']+'.log')
 for number,line in enumerate(path.read_text().splitlines(),1):
  tags=['public_contract_response','public_mailbox_response','public_artifact_response','lifecycle_','fixture_retained_unknown','fixture_disposal','public_amendment','public_current_gate','postgres_fixture']
  if any(tag in line for tag in tags):
   marker=line.find('{')
   if marker<0:continue
   try:j=json.loads(line[marker:])
   except ValueError:continue
   label=line[:marker].split(' ... ')[-1].strip()
   obs={'run':run['name'],'raw_line':number,'label':label,'key':j.get('key',j.get('body',{}).get('key')),'family':j.get('family',j.get('body',{}).get('command',{}).get('kind')),'status':j.get('status'),'code':j.get('result',{}).get('code') if isinstance(j.get('result'),dict) else None}
   observations.append(obs)
   if label=='lifecycle_measured_research':
    b=bytes(j['artifact_bytes']);assert h(b)==j['artifact'];saved=put(pathlib.Path('measured-artifacts')/j['artifact'],b)
    payload=json.loads(b);logs=payload['output'].encode();assert h(logs)==j['measurement']['logs'];log_saved=put(pathlib.Path('measured-artifacts')/j['measurement']['logs'],logs)
    artifacts.append({'run':run['name'],'raw_line':number,'round':j['round'],'purpose':j['purpose'],'execution':j['execution'],'artifact':saved,'measured_log':log_saved,'manifest':j['manifest'],'measurement':j['measurement'],'released':j['released']})
put(pathlib.Path('observation-index.json'),(json.dumps(observations,indent=2)+'\n').encode())
put(pathlib.Path('measured-artifact-bindings.json'),(json.dumps(artifacts,indent=2)+'\n').encode())
print(json.dumps({'sources':len(entries),'changed_from_base':sum(x['changed_from_base'] for x in entries),'changed_from_repair3':sum(x['changed_from_rejected_repair3'] for x in entries),'executables':len(executables),'observations':len(observations),'measured_artifacts':len(artifacts),'source_manifest':record(str((ev/'source-manifest.json').relative_to(root)))},indent=2))
