#!/usr/bin/env python3
"""Verify explicit imported-source relocation, retirement and retained ancestry."""
import hashlib
import json
from pathlib import Path
import stat
import subprocess

ROOT=Path(__file__).resolve().parents[1]
MANIFEST=ROOT/'evidence/provenance/absorption.json'


def git(root, *args):
    return subprocess.check_output(['git','--no-replace-objects',*args],cwd=root)


def verify(root=ROOT):
    root=Path(root)
    record=json.loads((root / "evidence/provenance/absorption.json").read_text())
    if record['schema']!='harp-absorption/v2':raise ValueError('unsupported absorption record')
    baseline=record['baseline']
    rows={}
    for entry in git(root, 'ls-tree','-r','-z',baseline,'--','corenius').split(b'\0'):
        if not entry:continue
        fields,name=entry.split(b'\t');mode,kind,blob=fields.decode().split()
        if kind!='blob':raise ValueError('unsupported source tree entry')
        rows[name.decode()]={'blob':blob,'mode':mode}
    if set(rows)!=set(record['paths']):raise ValueError('absorption source path set is incomplete')
    from git_objects import blob_digests
    source_hashes=blob_digests(root, [x["blob"] for x in rows.values()])
    seen=set()
    for old,source in rows.items():
        item=record['paths'][old]
        if item['source_blob']!=source['blob'] or item['source_mode']!=source['mode']:
            raise ValueError('source identity changed: '+old)
        if source_hashes[source["blob"]]!=item['source_sha256']:raise ValueError('source bytes differ: '+old)
        if item['disposition']=='retired':
            if not item.get('reason'):raise ValueError('retirement requires a reason')
            if old.startswith(('corenius/bench/','corenius/docs/efforts/')):
                raise ValueError('benchmark/source study cannot be silently retired: '+old)
            continue
        target=item['destination'];path=root/target
        if target in seen or Path(target).is_absolute() or '..' in Path(target).parts:
            raise ValueError('duplicate or unsafe relocation destination')
        seen.add(target)
        if any(p.is_symlink() for p in [path,*path.parents]):raise ValueError('symlinked relocation destination')
        info=path.stat()
        if not stat.S_ISREG(info.st_mode):raise ValueError('relocation is not an ordinary file')
        mode='100755' if info.st_mode&stat.S_IXUSR else '100644'
        if mode!=item['destination_mode'] or hashlib.sha256(path.read_bytes()).hexdigest()!=item['destination_sha256']:
            raise ValueError('relocated bytes/mode changed without provenance: '+target)
    previous=json.loads(git(root, 'cat-file','blob',record['previous_manifest_blob']))
    source=previous['source_commit']
    tips={x['commit'] for x in previous['branches']}
    tips.update(x.get('snapshot_commit') or x['head'] for x in previous['worktrees'])
    for tip in [source,*tips]:
        subprocess.run(['git','--no-replace-objects','merge-base','--is-ancestor',tip,'HEAD'],cwd=root,check=True)
    if (root/'corenius').exists() and any((root/'corenius').rglob('*')):
        raise ValueError('obsolete imported directory remains')
    print(json.dumps({'source_paths':len(rows),'mapped_paths':len(seen),'retirements':len(rows)-len(seen),
                      'absorbed_tips':len(tips),'bytes_modes_ancestry':'verified'}))

if __name__=='__main__':verify()
