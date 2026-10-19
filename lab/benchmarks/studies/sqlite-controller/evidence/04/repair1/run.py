"""Source-paired, bounded model-free author gates for the frozen repair."""
import datetime,json,os,pathlib,subprocess,sys,time
name=sys.argv[1];command=sys.argv[2:];root=pathlib.Path(__file__).resolve().parent
raw=pathlib.Path('.scratch/sqlite-controller/04/repair1');raw.mkdir(exist_ok=True,parents=True)
seal=root/'source/seal.py'
subprocess.run([sys.executable,str(seal),name+'-before'],check=True)
started=datetime.datetime.now(datetime.timezone.utc).isoformat();clock=time.monotonic()
with (raw/(name+'.log')).open('wb') as log:completed=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT)
subprocess.run([sys.executable,str(seal),name+'-after'],check=True)
record={'name':name,'source':subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip(),'command':command,'started':started,'ended':datetime.datetime.now(datetime.timezone.utc).isoformat(),'elapsed_seconds':time.monotonic()-clock,'exit':completed.returncode,'raw_log':str(raw/(name+'.log')),'environment':{k:v for k,v in os.environ.items() if k.startswith(('CORENIUS_TEST_','KATA_PINNED_','CARGO_','RUST'))}}
(root/(name+'-run.json')).write_text(json.dumps(record,indent=2)+'\n');print(json.dumps(record),flush=True);sys.exit(completed.returncode)
