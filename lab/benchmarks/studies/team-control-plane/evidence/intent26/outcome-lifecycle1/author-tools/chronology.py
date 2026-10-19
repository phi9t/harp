import pathlib,json,re,hashlib
root=pathlib.Path.cwd();ev=root/'docs/efforts/team-control-plane/evidence/intent26/outcome-lifecycle1';runs=[json.loads(x) for x in (ev/'runs.jsonl').read_text().splitlines()]
notes={
'red-lifecycle':'Genuine baseline RED: accepted research amendment does not fence generation; exact historical receipt stale; terminal mutation and projection gaps. Missing delivery/start IDs are not Owner target proof.',
'red-lifecycle-expanded':'Genuine baseline expanded RED: terminal cached proposal/verification and artifact configure allowed, research wait lacks attention, production historical acceptance receipt refuses. Earlier source-only setup retained.',
'green-lifecycle':'Partial GREEN4/5; production historical accepted receipt still fails nested ArtifactService current-generation check. Failure retained; not final pass.',
'red-research-verifier':'Genuine public RED: exact independent research verifier Episode refused without production candidate after valid current captured input; wrong-role401 is a positive control.',
'green-complete-lifecycle':'24 passed / 2 fixture setup failures: Ready workspace Release attempted without effect Cancel. Full measured two-generation research lifecycle passed. These setup failures are not RED proof. Fixed fixture uses existing Owner Cancel then Release; no terminal semantics change.',
'red-owner-operation-target':'Counterfactual RED: current candidate with ONLY operation-target terminal fence removed. Actual preaccept native runtime/binding/delivery positives; accepted cached binding/delivery wrongly200. Exact guard preimage/delta/restoration retained. Native turn completed physically, usage unknown retained. Not original20-source baseline.',
'red-independent-research-verifier':'Genuine public RED on research route before independence guard: selected verifier principal equals exact current Attempt worker, second verifier credential; Episode admitted200 instead of401. Narrow current-Attempt guard added, no historical ever-worker ban.',
'green-compatibility':'Lib9/artifacts11/gateway4 pass; intent28 pass/1 expectation failure. Source-only old input409 and prepared execution refusal correct; test expected DependencyUnmet although Supervisor::start maps dispatch-current=false to StaleGeneration. Test-only correction, failed fixture retained.',
'green-compatibility-final':'Lib9/artifacts11/gateway4 pass; intent29 pass/1 test observation failure. New concurrency test compared changing snapshot_time. Test-only correction excludes snapshot_time while comparing all stored fields; provisional filename retained unchanged. Not final pass.',
'green-compatibility-freeze':'Final bounded compatibility suite; this record alone supplies the final source-bound pass, if exit0. No further test launches authorized under this allocation.',
'strict-lint':'Failed strict lint: preexisting large command enum/collapsible baseline branches plus one new collapsible condition. Behavior-preserving cleanups and targeted existing enum API lint allowance; later strict lint passed.'}
records=[]
for run in runs:
 p=ev/(run['name']+'.log');raw=p.read_bytes();assert hashlib.sha256(raw).hexdigest()==run['log_sha256'];lines=raw.decode().splitlines();results=[];failures=[]
 for n,line in enumerate(lines,1):
  if line.startswith('test result:'):results.append({'raw_line':n,'text':line})
  if 'panicked at' in line or line.startswith('error:'):failures.append({'raw_line':n,'text':line})
 records.append({k:v for k,v in run.items() if k!='inputs'}|{'interpretation':notes.get(run['name'],'Formatting or strict-lint check; see exact exit/output.'),'raw_results':results,'raw_failure_locations':failures,'input_manifest':run['name']+'.start.json'})
summary={'test_launches':sum(x['kind']=='test' for x in runs),'maximum_test_launches':10,'cumulative_seconds_including_formatting_lint':sum(x['seconds'] for x in runs),'maximum_seconds':3600,'all_commands_within_1800_seconds':all(x['seconds']<1800 for x in runs),'chronology':records,'preservation':'All raw bytes and chronological source input snapshots retained. No logs renamed or rewritten. Original repair3 87-pass log remains rejected-baseline author evidence only.'}
(ev/'run-chronology.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({k:v for k,v in summary.items() if k!='chronology'},indent=2))
