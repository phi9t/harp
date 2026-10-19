import pathlib,subprocess,json,hashlib,tarfile,sys,datetime,os
r=pathlib.Path(__file__).parent
name=sys.argv[1];command=sys.argv[2:];log=r/(name+".log")
assert not log.exists(),"append-only run name required"
files=sorted(p for p in pathlib.Path("crates/sqlite-controller").rglob("*") if p.is_file() and "target" not in p.parts and "__pycache__" not in p.parts)
pins={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in files}
archive=r/(name+"-source.tar.gz")
with tarfile.open(archive,"w:gz") as t:
 for p in files:t.add(p,arcname=str(p))
record={"started_at":datetime.datetime.now(datetime.timezone.utc).isoformat(),"command":command,"before_inputs":pins,"source_archive_sha256":hashlib.sha256(archive.read_bytes()).hexdigest()}
(r/(name+"-start.json")).write_text(json.dumps(record,indent=2)+"\n")
with log.open("wb") as out:result=subprocess.run(command,stdout=out,stderr=subprocess.STDOUT)
record.update(exit=result.returncode,log_sha256=hashlib.sha256(log.read_bytes()).hexdigest(),after_inputs={str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in files},completed_at=datetime.datetime.now(datetime.timezone.utc).isoformat())
(r/(name+".json")).write_text(json.dumps(record,indent=2)+"\n")
print("actual execution exit",result.returncode);print(log.read_text(errors="replace")[-3000:]);sys.exit(result.returncode)
