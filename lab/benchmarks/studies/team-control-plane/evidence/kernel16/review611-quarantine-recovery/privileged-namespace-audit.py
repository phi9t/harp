import os,pathlib,json,time,errno
start=time.time(); targets={'ipc':'4026539790','mnt':'4026539788','net':'4026539792','pid':'4026539791','uts':'4026539789'}; counts={};errors=[];matches=[];holders=[];processes=threads=nsreads=fdreads=0
boot=pathlib.Path('/proc/sys/kernel/random/boot_id').read_text().strip()
def failure(e,kind,p):
 key=kind+':'+str(e.errno);counts[key]=counts.get(key,0)+1
 if e.errno not in (2,3):errors.append({'path':str(p),'kind':kind,'errno':e.errno})
for p in pathlib.Path('/proc').iterdir():
 if not p.name.isdigit():continue
 processes+=1
 try:ts=list((p/'task').iterdir())
 except OSError as e:failure(e,'tasklist',p);continue
 for t in ts:
  threads+=1
  for typ,inode in targets.items():
   q=t/'ns'/typ
   try:v=os.readlink(q);nsreads+=1
   except OSError as e:failure(e,'ns',q);continue
   if v==typ+':['+inode+']':matches.append({'pid':int(p.name),'tid':int(t.name),'namespace':v})
 try:fs=list((p/'fd').iterdir())
 except OSError as e:failure(e,'fdlist',p);continue
 for f in fs:
  try:v=os.readlink(f);fdreads+=1
  except OSError as e:failure(e,'fd',f);continue
  if any(v==typ+':['+inode+']' for typ,inode in targets.items()):holders.append({'pid':int(p.name),'fd':f.name,'namespace':v})
child={'pid':4188604}
try:
 s=pathlib.Path('/proc/4188604/stat').read_text();parts=s[s.rfind(')')+2:].split();child.update(present=True,state=parts[0],start_ticks=int(parts[19]),namespaces={typ:os.readlink('/proc/4188604/ns/'+typ) for typ in targets})
except OSError as e:child.update(present=False if e.errno in (2,3) else None,errno=e.errno)
print(json.dumps({'started_unix':start,'ended_unix':time.time(),'effective_uid':os.geteuid(),'boot_id':boot,'targets':targets,'child':child,'matching_task_namespaces':matches,'matching_namespace_fd_holders':holders,'counts':{'processes':processes,'threads':threads,'nsreads':nsreads,'fdreads':fdreads},'errors_by_kind_errno':counts,'non_disappearance_errors':errors,'limitations':['non-atomic single proc sweep; concurrent creation/exit possible','proc visibility bounded to this host proc mount','original boot/starttick identity absent; no historical shutdown certificate','read-only; no signals/deletion/refund/reuse']},indent=2))
