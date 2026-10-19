import datetime,hashlib,json,pathlib,shutil,subprocess,sys
root=pathlib.Path.cwd();scratch=root/'.scratch/sqlite-controller/05';out=root/'docs/efforts/sqlite-controller/evidence/05/repair2'
source=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip();assert source==sys.argv[1]
assert not subprocess.check_output(['git','status','--porcelain'],text=True).strip()
tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],text=True).strip();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
out.mkdir(parents=True,exist_ok=False);(out/'runs').mkdir();(out/'original-reviews').mkdir()
def write(name,value):(out/name).write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
final_names=sys.argv[2:];assert final_names
inventory={}
for path in sorted((root/'crates/sqlite-controller').rglob('*')):
 if not path.is_file() or 'target' in path.parts or '__pycache__' in path.parts:continue
 relative=str(path.relative_to(root));inventory[relative]={'sha256':sha(path),'git_blob':subprocess.check_output(['git','rev-parse',source+':'+relative],text=True).strip()}
for name in final_names:
 data=json.loads((scratch/(name+'.json')).read_text());assert data['exit']==0 and data['before_inputs']==data['after_inputs']
 assert data['after_inputs']=={p:v['sha256'] for p,v in inventory.items()},name
write('source-files.json',{'source_commit':source,'source_tree':tree,'previous_source':'ae4e6fed22d08eab45cdbcddf6e472579a6899a8','previous_evidence':'655c86ee9a212878b1c921c34dee55dee5066c3c','admitted_base':'291f04a5abd9274e4fd0034fd36927b0a4c4d236','input_count':len(inventory),'files':inventory})
notes={
72:'Actual101/BOTH54.44s metadata-only diagnostic productionae4 unchanged: shared last8th tuple reconciling/null/UIfalse at0.504279s; separate29 original2s samples PASS. Original80070 tuple/raw gap unknown. Diagnostic1/2.',
73:'Actual101/BOTH80.11s genuine lost-start-ACK response shape and retained exact A-steer after current B RED; no extra effects. Original current/new-stale controls retained.',
74:'Matched73 actual0/BOTH90.30s native UI lostACK pinned response and exact retained steer after B GREEN, changed arguments/foreign/new stale/currentB/cached reply controls.',
75:'Matched72 original2s assertions/deadlines actual0/BOTH59.51s approved existing empty-creation/incarnation precheck GREEN. Diagnostic2/2 consumed; no historical80070 cause attribution.',
76:'Actual101 accumulated70 valid65536B UTF8 questions: control_center4606190B/events4601159B/dashboard1/MCP-32000 genuine projection RED.',
77:'Compiler undefined number setup failure, no GREEN or behavioral credit.',
78:'Actual0 bounded questions70/4pages/events73/4pages/own35/2pages exact Unicode fulltext and scoped continuations, CLI/dashboard/MCP GREEN. Conditional peer control_input negative not executed; corrected83.',
79:'Public observe_runtime correctly denied lacking trusted runtime evidence; history fixture setup failure, no semantic RED.',
80:'Stopped owned DB seeded26 exited historical rows; fresh two real admissions observed[]/2slots, but fixture asserted wrong response shape. Qualified measurement only, no clean semantic oracle credit.',
81:'Corrected exact original observed response shape actual101 meaningful late-active-slot RED0/2 behind26 stopped-DB seeded histories. Seeded rows are not actual prior executions.',
82:'Matched81 actual0 fresh two supervised workers/heartbeats/cancel0slots using approved operator two-slot exact-journal context. Seed history and fresh physical effects distinguished.',
83:'Actual0 all3client+7public controls; unconditional peer scoped-input refusal, changed digest/bytes and invalid UTF8 offset no-effect negatives; full70questions/73events/35own, duplicate exact receipts.',
84:'Actual0 bounded configured Git inventory/history with25 seeded old rows plus fresh actual late active workspace, two pages26records/25history and exact two large immutable input refs; role negatives/currentcancel. Original slow-I/O control48.65ms. Runtime missing/duplicate-start/duplicate-cancel/foreign journal refusal no journal/event/receipt effect and late-slot positives.',
85:'Actual0/BOTH91.19s pinned native UI lostACK/saved steer A-to-B, changed/foreign/newstale/currentB/cached recovered start after B; worker typed public/tool history full inputs/peer refusal.',
86:'Actual101 Kata7PASS1FAIL: original maxprogress inline-only events consumer fails on bounded exact reference. Runtime bin not run. Retained driver failure, no lost fulltext claim.',
87:'TEST-only reference/page consumer adaptation: original65536B exact progress equality and deadlines unchanged; actual0 required Kata maxprogress plus reported-progress full reference. Product publication/fences unchanged.',
88:'Actual0 units20+runtime16 original pause/currentauth/cancel/tombstone/unknown/no-PID-reconstruction and late-slot semantics PASS.',
89:'Actual101/BOTH82.99s replaced lead public/tool history missing semantic RED despite four retained native turns. Old principal exact native start/steer retries already refused without old UI/journal effects.',
90:'Actual0 units20+client3+public8 after separate typed-visible lead history ledger and documented history-derived recovered response presentation.',
91:'Matched89 actual0/BOTH90.48s old lead12retained typed public/tool items each; replacement/operator exact inputs; worker/observer/revoked grants denied. Empty response items are documented history-derived presentation, never observed lost initial ACK items.',
92:'Strict FI Clippy101 two lint findings raw8args and collapsible-if; no semantic failure credit.',
93:'Mechanical private RecordRange call adaptation missed one compiler call; no GREEN.',
94:'Actual0 final product units21/client3/public8; synthetic typed source fixture proves private/unfinished/credential redaction, exact provenance/digest, immutable lead ledger/dedup and unchanged worker monitor/assessment credit. Actual native positives independently91.',
95:'Actual0 strict all-target FI Clippy -D warnings; product source bytes final, later test-consumer adaptation qualified separately.',
96:'Actual101/43.39s combined native TEST TypeError json.loads(decoded arguments dict), no target RED/success credit. Root audit35135d6 confirms46inputs unchanged.',
97:'Actual101/41.85s combined native TEST TypeError query helper accepts only kind, no target RED/success credit. Subsequent TEST-only optional selectors and large question descriptor match preserve exact assertions/deadlines.',
98:'Actual0/BOTH137.34s combined pinned real native+companion+TUI after TEST consumer adaptation: exact active replay/interrupt, serialized drain, shared peer routing to validated owner, earlyACK/lostreply/finish race, typed monitor input/correction workerACK, ten exact large questions/later delivery, two slots/replacement/cancel controls.',
99:'Actual0 required DEFAULT initial-confirm/BOTH60.64s, no optional diagnostic; exact original turn+clientId wait SAME10s and unchanged final assertion. No historical full-gate causality/05 attribution.',
100:'Actual final-source same-incarnation empty-thread controller-crash fixture, BOTH; original admission/recovery/assertions/deadlines retained. Distinct from consumed changed-incarnation diagnostic2/2.',
101:'Actual final-source later-turn confirmed receipt before binding-crash/adoption fixture BOTH; original assertions/deadlines/native fixtures retained.',
102:'Actual final-source native-history confirmation/lostACK before binding-crash/adoption fixture BOTH; original assertions/deadlines retained.',
103:'Final source cargo fmt --all --check; actual recorded exit authoritative.',

}
records=[];all_dispositions=[]
def present(record,namespace=False):
 try:now=(pathlib.Path('/proc')/str(record['pid'])/'stat').read_text().rsplit(') ',1)[1].split()[19]
 except (FileNotFoundError,ProcessLookupError):return False
 old=str(record['start_ticks']) if namespace else record['start_stat'].rsplit(') ',1)[1].split()[19]
 return now==old
for path in sorted(scratch.glob('repair2-*.json')):
 if path.name.endswith('-start.json'):continue
 data=json.loads(path.read_text())
 if 'exit' not in data:continue
 number=int(path.name.split('-')[1]);assert data['before_inputs']==data['after_inputs']
 shutil.copyfile(path,out/'runs'/path.name)
 entry={'name':path.stem,'record':'runs/'+path.name,'record_sha256':sha(path),'exit':data['exit'],'source_unchanged':True,'input_count':len(data['before_inputs']),'qualification':notes.get(number,'Final focused qualification: inspect retained actual log/assertions/source/input pins; root independent acceptance pending.'),'ignored_raw':[]}
 for suffix in ['.log','-source.tar.gz','-start.json']:
  raw=scratch/(path.stem+suffix);assert raw.is_file()
  if suffix=='.log':assert sha(raw)==data['log_sha256']
  if suffix=='-source.tar.gz':assert sha(raw)==data['source_archive_sha256']
  entry['ignored_raw'].append({'path':str(raw.relative_to(root)),'bytes':raw.stat().st_size,'sha256':sha(raw)})
 resources=[];namespaces=[]
 for line in (scratch/(path.stem+'.log')).read_text(errors='replace').splitlines():
  for prefix,target in [('owned_fixture_resources ',resources),('owned_ui_resources_terminated ',resources),('owned_projection_resources ',resources),('owned_fixture_namespaces ',namespaces)]:
   if line.startswith(prefix):target.extend(json.loads(line[len(prefix):]))
 for record in resources:record['same_identity_present']=present(record)
 for record in namespaces:record['same_identity_present']=present(record,True)
 assert not any(x['same_identity_present'] for x in resources+namespaces)
 assert all(x['whole_process_terminated'] for x in namespaces)
 if resources or namespaces:all_dispositions.append({'run':path.stem,'resources':resources,'namespaces':namespaces,'namespace_product_cleanup_count':sum(x['terminated_after_service_cleanup'] for x in namespaces),'namespace_fixture_cleanup_count':sum(x['fixture_cleanup_signalled'] for x in namespaces)})
 records.append(entry)
write('run-index.json',{'source_commit':source,'records':records,'raw_root':str(scratch),'acceptance':'root full regression/two independent axes/branch reviews pending; no self-acceptance'})
write('owned-process-dispositions.json',{'source_commit':source,'checked_at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'runs':all_dispositions,'all_captured_owned_identities_reconciled':True,'qualification':'Only captured owned PID/start identities; actual product and fixture cleanup distinguished. Seeded historical database rows are not prior physical executions. No unrelated signaling, live paid model, powerloss or unmanaged session claim.'})
shutil.copyfile(scratch/'run.py',out/'run-record-driver.py')
for path in sorted((scratch/'repair2').iterdir()):
 if path.is_file():shutil.copyfile(path,out/path.name)
coord=pathlib.Path('/data02/home/philip.yang/workspace/corenius-codex-sqlite-controller-execution')
spec=pathlib.Path('/data02/home/philip.yang/devx/tmp/sqlite05-ae4-spec-addendum-kh0_ijxf');standards=pathlib.Path('/data02/home/philip.yang/devx/tmp/sqlite05-standards-final-ae4e6fed-wfohrlnm')
origins=[
(coord/'.scratch/sqlite-controller/briefs/05-repair2.md','repair2-brief.md','3ac4ca527fdd8bc95e5f2bbb19a747946adef49b2d88851617c112784f5aa0cf'),
(coord/'.scratch/sqlite-controller/monitoring05/05-repair2-sourceae4-admission.json','root-admission.json','a5e5c96ba550e19ef92fb5842c2cef5714c5fac9333ceeff7da7f601a244378e'),
(coord/'.scratch/sqlite-controller/monitoring05/05-repair2-empty-incarnation-guard-adjudication.json','root-empty-guard.json','f6d7bee5b09a93e27a69f7f361e9182500840795c2a63822414af5eadf74049d'),
(coord/'.scratch/sqlite-controller/monitoring05/05-repair2-runtime-context-adjudication.json','root-runtime-context.json','263421f959ddf2eaaebe72f396774f2f333689fe87ed7bffdf55df46dd375057'),
(coord/'.scratch/sqlite-controller/monitoring05/05-repair2-lead-public-history-response-adjudication.json','root-lead-history.json','a8f2fd0f2873857c45512f5b8f178cbfef8c1f26d11f7c3679a32b752ba1888f'),
(coord/'.scratch/sqlite-controller/monitoring05/05-repair2-run96-root-audit.json','root-run96-audit.json','35135d6fe963c9f1a262b64219e9de98094f30f55fd1908544c412c52ade0419'),
(coord/'.scratch/sqlite-controller/monitoring05/05-branch651-result.json','original-reviews/branch651.json','f9aae558d805dc6928d75e982937f5b0acb9adfd35cdd039b38952c3fc5b826a'),
(coord/'.scratch/sqlite-controller/monitoring05/05-sourceae4-branch651-adjudication.json','original-reviews/branch651-adjudication.json','1d18ab9a8462b3d4256b90bc1dec64253bbda39c7c0ef0940f0a91f7ab59d697'),
(coord/'.scratch/sqlite-controller/monitoring05/root-repair1-sourceae4-gate/gate.json','original-reviews/root-sourceae4-gate.json','c240c941752bd977dd7b159b7615bd4998cb35d76350f55c1cf5175e1c7cd7fa'),
(spec/'addendum.md','original-reviews/spec-addendum.md','223c2951617b18f201f64f3656f8800ac83d9bc281d4153ff8632062e32b3917'),
(spec/'addendum.json','original-reviews/spec-addendum.json','0b5f6ca3727102e593a18519e987f738068bca96b8e8ee7e2e61b99028aca5c2'),
(spec/'evidence-index.json','original-reviews/spec-evidence-index.json','75ced68769e9b584ad14f45ab1a916e00bf8dedbb3cb29ca0758f43d294eb915'),
(standards/'report.md','original-reviews/standards.md','68e3580ade600ad4cf55e41f9f25f79621c9670c01149fad9b4a5ef723e77f08'),
(standards/'report.json','original-reviews/standards.json','f3d8ffc4a9a943d1ca54119f2b68959f6c47104d4df3a7590cf202953962d27a'),
(standards/'evidence-sha256-final.json','original-reviews/standards-evidence-index.json','227c9955378f8f9b061fbd88dc20af1cb1a714e468e5b9830084500df10ea49b'),
(standards/'report-location-erratum.md','original-reviews/standards-location-erratum.md','9a3ce448e967a06b5e6d6e587b529cfb123b70489e7aab630e4c92842fd55cae'),
]
retained=[]
for path,relative,expected in origins:
 assert sha(path)==expected,(str(path),sha(path),expected)
 shutil.copyfile(path,out/relative);retained.append({'origin':str(path),'retained':relative,'sha256':expected})
rawroot=scratch/'repair2-original-review-raw';rawroot.mkdir(exist_ok=True)
raworigins=[coord/'.scratch/sqlite-controller/monitoring05/root-repair1-sourceae4-gate/root-complete-fi.log',standards/'independent-edge-probes.log',standards/'independent-edge-probes-receipt.json',standards/'kernel-short-write.strace',standards/'initial-confirm-default.log',spec/'native-addendum.log',spec/'projection-probe.log']
for index,path in enumerate(raworigins):
 assert path.is_file(),path
 copy=rawroot/(str(index)+'-'+path.name);assert not copy.exists() or sha(copy)==sha(path);shutil.copyfile(path,copy)
 retained.append({'ignored_raw_origin':str(path),'retained_ignored':str(copy.relative_to(root)),'sha256':sha(copy),'bytes':copy.stat().st_size})
assert sha(raworigins[0])=='0247aab6b87367b57f4b4b89d2e48e975966e03b73518e25a044c44d1d99afd2'
assert sha(raworigins[1])=='58e705e82195b77ab6a50e0270e96350e8e445ea0cf8792ed52872d6c717ac69'
write('origin-retention.json',{'records':retained,'all_original_failures_preserved':True,'historical_empty_failure':'root80070 tuple/raw unknown; later diagnostic72 shared reconciling is observed only for72; independent29sample PASS does not clear original or establish historical cause','previous_evidence':'../repair1/handoff.md'})
archives=[{'path':str(p.relative_to(root)),'bytes':p.stat().st_size,'sha256':sha(p)} for p in sorted((root/'.scratch/sqlite-controller').rglob('*.tar.gz'))]
write('raw-retention.json',{'source_commit':source,'archives':archives,'raw_logs_retained_ignored':True,'prior_retention':'../repair1/raw-retention.json','initial36_archive_gap_preserved':True,'root80070_original_raw_fixture_gap_preserved':True})
print(json.dumps({'source_commit':source,'tree':tree,'inputs':len(inventory),'runs':len(records),'archives':len(archives),'disposition_runs':len(all_dispositions)}))
