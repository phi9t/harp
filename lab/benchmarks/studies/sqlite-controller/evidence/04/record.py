"""Retain owned local qualification evidence; never execute fixture Git metadata."""
from pathlib import Path
import gzip,hashlib,json,re,shutil,tarfile
root=Path(__file__).resolve().parent
raw=Path('.scratch/sqlite-controller/04')
for name in ['raw','fixtures','observations']:(root/name).mkdir(exist_ok=True)
retained=[];observations=[];tests={}
def keep(path,destination):
 shutil.copyfile(path,destination)
 retained.append({'original':str(path),'retained':str(destination),'bytes':destination.stat().st_size,'sha256':hashlib.sha256(destination.read_bytes()).hexdigest()})
for path in sorted(raw.glob('*.log')):
 data=path.read_bytes();destination=root/'raw'/(path.name+'.gz')
 with destination.open('wb') as stream:
  with gzip.GzipFile(fileobj=stream,mode='wb',mtime=0) as zipped:zipped.write(data)
 retained.append({'original':str(path),'retained':str(destination),'bytes':len(data),'raw_sha256':hashlib.sha256(data).hexdigest(),'sha256':hashlib.sha256(destination.read_bytes()).hexdigest()})
 text=data.decode(errors='replace');tests[path.name]=re.findall(r'test result: (.+)',text)
 for line in text.splitlines():
  if line.startswith(('actual_','owned_fixture_','retained_actual_fixture')) and '{' in line:
   start=line.index('{')
   try:value=json.loads(line[start:])
   except json.JSONDecodeError:continue
   observations.append({'log':path.name,'observation':line[:start].strip(),'value':value})
 for name in re.findall(r'Retained real Kata fixture: (.+)',text):
  fixture=Path(name)
  if not fixture.is_relative_to(Path.cwd()/'.scratch') or not fixture.is_dir():continue
  destination=root/'fixtures'/(fixture.name+'.tar.gz')
  if not destination.exists():
   with tarfile.open(destination,'w:gz') as archive:archive.add(fixture,arcname=fixture.name)
  if not any(x['retained']==str(destination) for x in retained):retained.append({'original':str(fixture),'retained':str(destination),'bytes':destination.stat().st_size,'sha256':hashlib.sha256(destination.read_bytes()).hexdigest()})
for path in sorted(raw.glob('fixtures/*.tar.gz')):keep(path,root/'fixtures'/path.name)
for path in sorted(Path('.scratch/sqlite-controller/native').glob('*.tar.gz')):keep(path,root/'fixtures'/path.name)
for path in sorted(raw.glob('*.json')):keep(path,root/'observations'/path.name)
(root/'retention.json').write_text(json.dumps(retained,indent=2)+'\n')
(root/'observations.json').write_text(json.dumps(observations,indent=2)+'\n')
(root/'test-results.json').write_text(json.dumps(tests,indent=2)+'\n')
pairs=[]
for before in sorted((root/'source').glob('*-before.json')):
 after=before.with_name(before.name.replace('-before.json','-after.json'))
 b=json.loads(before.read_text())
 a=json.loads(after.read_text()) if after.exists() else None
 pairs.append({'before':str(before),'after':str(after) if a else None,'same_files':b['files']==a['files'] if a else None,'head':b['head'],'archive_sha256_before':b['archive_sha256'],'archive_sha256_after':a['archive_sha256'] if a else None})
(root/'source-pairs.json').write_text(json.dumps(pairs,indent=2)+'\n')
print('retained evidence',len(retained),'artifacts',len(observations),'actual JSON observations')
