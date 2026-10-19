from pathlib import Path
import datetime,hashlib,json,os,subprocess,sys
root=Path.cwd(); out=root/'docs/efforts/team-control-plane/evidence/jj19/repair3-history-export'
label=sys.argv[1]; expected=int(sys.argv[2]); argv=sys.argv[3:]
fixture=Path(argv[argv.index('--fixture')+1]) if '--fixture' in argv else root/'docs/efforts/team-control-plane/research/probes/jj_workspace_evaluation.py'
record={'label':label,'argv':argv,'cwd':str(root),'started_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'fixture_path':str(fixture),'fixture_sha256':hashlib.sha256(fixture.read_bytes()).hexdigest(),'command_source_sha256':hashlib.sha256(Path(argv[1]).read_bytes()).hexdigest(),'expected_exit':expected}
env=os.environ.copy(); env['PYTHONDONTWRITEBYTECODE']='1'
p=subprocess.run(argv,capture_output=True,text=True,env=env,timeout=600)
record.update(exit=p.returncode,stdout=p.stdout,stderr=p.stderr,finished_utc=datetime.datetime.now(datetime.timezone.utc).isoformat())
if '--output' in argv:
    output=Path(argv[argv.index('--output')+1]).resolve(); record.update(output=str(output.relative_to(root)),output_sha256=hashlib.sha256(output.read_bytes()).hexdigest())
(out/(label+'-terminal.json')).write_text(json.dumps(record,indent=2)+'\n')
ledger=out/'run-ledger.json'; records=json.loads(ledger.read_text()) if ledger.exists() else []; records.append(record); ledger.write_text(json.dumps(records,indent=2)+'\n')
print(json.dumps({'label':label,'exit':p.returncode,'expected_exit':expected,'stdout':p.stdout[-1500:],'stderr':p.stderr[-1500:]},indent=2))
if p.returncode!=expected: raise SystemExit('unexpected exit')
