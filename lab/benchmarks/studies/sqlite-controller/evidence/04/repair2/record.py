"""Retain only this repair's owned logs/fixtures, preserving initial evidence."""
from pathlib import Path
import gzip,hashlib,json,re,shutil,tarfile
root=Path(__file__).resolve().parent;raw=Path('.scratch/sqlite-controller/04/repair2')
for name in ('raw','fixtures','observations'):(root/name).mkdir(exist_ok=True)
retained=[];observations=[];results={};selected=set()
def keep(path,destination):
 if destination.exists():assert destination.read_bytes()==path.read_bytes(),('immutable retention differs',destination)
 else:shutil.copyfile(path,destination)
 retained.append({'original':str(path),'retained':str(destination),'bytes':destination.stat().st_size,'sha256':hashlib.sha256(destination.read_bytes()).hexdigest()})
for path in sorted(raw.glob('*.log')):
 data=path.read_bytes();destination=root/'raw'/(path.name+'.gz')
 with destination.open('wb') as stream:
  with gzip.GzipFile(fileobj=stream,mode='wb',mtime=0) as zipped:zipped.write(data)
 retained.append({'original':str(path),'retained':str(destination),'bytes':len(data),'raw_sha256':hashlib.sha256(data).hexdigest(),'sha256':hashlib.sha256(destination.read_bytes()).hexdigest()})
 text=data.decode(errors='replace');results[path.name]=re.findall(r'test result: (.+)',text)
 for line in text.splitlines():
  if line.startswith(('actual_','owned_fixture_','retained_actual_fixture')) and '{' in line:
   start=line.index('{')
   try:value=json.loads(line[start:])
   except json.JSONDecodeError:continue
   observations.append({'log':path.name,'observation':line[:start].strip(),'value':value})
   if line.startswith('retained_actual_fixture'):selected.add(Path(value['archive']))
 for name in re.findall(r'Retained real Kata fixture: (.+)',text):
  fixture=Path(name)
  assert fixture.is_relative_to(Path.cwd()/'.scratch') and fixture.is_dir()
  destination=root/'fixtures'/(fixture.name+'.tar.gz')
  if not destination.exists():
   with tarfile.open(destination,'w:gz') as archive:archive.add(fixture,arcname=fixture.name)
  if not any(x['retained']==str(destination) for x in retained):retained.append({'original':str(fixture),'retained':str(destination),'bytes':destination.stat().st_size,'sha256':hashlib.sha256(destination.read_bytes()).hexdigest()})
initial={entry['original'] for record in [root.parent/'retention.json',root.parent/'repair1/retention.json'] for entry in json.loads(record.read_text())}
selected.update(path for path in Path('.scratch/sqlite-controller/native').glob('*.tar.gz') if str(path) not in initial)
for path in sorted(selected):keep(path,root/'fixtures'/path.name)
(root/'retention.json').write_text(json.dumps(retained,indent=2)+'\n');(root/'observations.json').write_text(json.dumps(observations,indent=2)+'\n');(root/'test-results.json').write_text(json.dumps(results,indent=2)+'\n')
pairs=[]
for before in sorted((root/'source').glob('*-before.json')):
 after=before.with_name(before.name.replace('-before.json','-after.json'));b=json.loads(before.read_text());a=json.loads(after.read_text()) if after.exists() else None
 pairs.append({'before':str(before),'after':str(after) if a else None,'same_files':b['files']==a['files'] if a else None,'head':b['head'],'source_files':len(b['files']),'archive_sha256_before':b['archive_sha256'],'archive_sha256_after':a['archive_sha256'] if a else None})
(root/'source-pairs.json').write_text(json.dumps(pairs,indent=2)+'\n');print('retained',len(retained),'artifacts',len(observations),'observations',len(pairs),'source pairs')
