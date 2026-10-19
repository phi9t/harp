"""Read retained fixture bytes only; never run worker Git or untrusted config."""
from pathlib import Path
import hashlib,json,tarfile
root=Path(__file__).resolve().parent
observations=json.loads((root/'observations.json').read_text());archives={Path(e['original']).name:Path(e['retained']) for e in json.loads((root/'retention.json').read_text()) if e['retained'].endswith('.tar.gz')}
records=[]
for o in observations:
 if o['observation']!='retained_actual_fixture':continue
 a=archives[Path(o['value']['archive']).name]
 with tarfile.open(a) as archive:
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
    record={'archive':str(a),'archive_member':entry.name,'artifact_sha256':hashlib.sha256(data).hexdigest(),'log':o['log'],'mode':o['value']['mode'],'process':value['process']}
    for k in ('argv','args','exit','tool','archive_limit','retained_archive_bytes','os_isolated','environment','suite'):
     if k in value:record[k]=value[k]
    for k in ('stdout','stderr'):
     if k in value:record[k+'_bytes']=len(value[k].encode());record[k+'_sha256']=hashlib.sha256(value[k].encode()).hexdigest()
    records.append(record)
(root/'bounded-process-observations.json').write_text(json.dumps(records,indent=2)+'\n');print('retained bounded process/artifact observations',len(records))
