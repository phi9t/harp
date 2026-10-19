"""Read retained fixture bytes only; never run worker Git or untrusted config."""
from pathlib import Path
import argparse,hashlib,json,os,re,stat,tarfile
root=Path(__file__).resolve().parent
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('--output',required=True,type=Path,help='fresh explicit JSON output; existing paths are refused')
output=parser.parse_args().output
if output.exists() or output.is_symlink():raise FileExistsError(output)
fixtures=root/'fixtures'
if fixtures.is_symlink():raise ValueError('fixtures directory must be local')
observations=json.loads((root/'observations.json').read_text());archives={};names=set()
def validated_archive(entry):
 name=Path(entry['retained']).name
 if not re.fullmatch(r'[A-Za-z0-9][A-Za-z0-9_.-]*\.tar\.gz',name):raise ValueError('invalid retained archive filename')
 path=fixtures/name
 stream=os.fdopen(os.open(path,os.O_RDONLY|os.O_NOFOLLOW),'rb')
 try:
  if not stat.S_ISREG(os.fstat(stream.fileno()).st_mode):raise ValueError('retained archive must be a regular file')
  digest=hashlib.sha256();size=0
  while block:=stream.read(1024*1024):digest.update(block);size+=len(block)
  if size!=entry['bytes'] or digest.hexdigest()!=entry['sha256']:raise ValueError('retained archive manifest digest/size mismatch')
  stream.seek(0)
  return path,stream
 except BaseException:
  stream.close()
  raise
for entry in json.loads((root/'retention.json').read_text()):
 if not entry['retained'].endswith('.tar.gz'):continue
 key=Path(entry['original']).name;name=Path(entry['retained']).name
 if key in archives or name in names:raise ValueError('duplicate retained archive filename or provenance key')
 _,stream=validated_archive(entry);stream.close()
 archives[key]=entry;names.add(name)
records=[]
for o in observations:
 if o['observation']!='retained_actual_fixture':continue
 retained=archives[Path(o['value']['archive']).name]
 a,stream=validated_archive(retained)
 with stream,tarfile.open(fileobj=stream) as archive:
  for entry in archive:
   if not entry.isfile():continue
   parts=Path(entry.name).parts
   if 'artifacts' not in parts:continue
   stream=archive.extractfile(entry)
   if entry.size>32*1024*1024:continue
   data=stream.read()
   try:value=json.loads(data)
   except (ValueError,UnicodeDecodeError):continue
   if not isinstance(value,dict):continue
   if 'process' in value and ('stdout' in value or 'retained_archive_bytes' in value):
    record={'archive':str(a),'archive_original':retained['original'],'archive_retained_original':retained['retained'],'archive_member':entry.name,'artifact_sha256':hashlib.sha256(data).hexdigest(),'log':o['log'],'mode':o['value']['mode'],'process':value['process']}
    for k in ('argv','args','exit','tool','archive_limit','retained_archive_bytes','os_isolated','environment','suite'):
     if k in value:record[k]=value[k]
    for k in ('stdout','stderr'):
     if k in value:record[k+'_bytes']=len(value[k].encode());record[k+'_sha256']=hashlib.sha256(value[k].encode()).hexdigest()
    records.append(record)
with output.open('x',encoding='utf-8') as destination:destination.write(json.dumps(records,indent=2)+'\n')
print('retained bounded process/artifact observations',len(records))
