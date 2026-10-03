#!/usr/bin/env python3
"""Read-only closure check for frozen repair3 bytes and normal Git retention."""
from pathlib import Path
import hashlib,json,re,subprocess

def main():
    root=Path.cwd(); evidence=Path(__file__).resolve().parent
    digest=lambda path:hashlib.sha256(path.read_bytes()).hexdigest()
    handoff=json.loads((evidence/'final-handoff.json').read_text())
    for field in ['source_freeze','public_proof_index','patch']:
        assert digest(root/handoff[field]['path'])==handoff[field]['sha256'],field
    freeze=json.loads((root/handoff['source_freeze']['path']).read_text())
    for record in freeze['source_records']+freeze['preimages']:
        assert digest(root/record['path'])==record['sha256'],record['path']
    index=json.loads((root/handoff['public_proof_index']['path']).read_text())
    selected=set(subprocess.check_output(['git','ls-files','--cached','--others','--exclude-standard','-z'],text=True).split('\0'))
    for record in index['records']:
        path=root/record['path']; assert path.is_file() and path.stat().st_size==record['size'] and digest(path)==record['sha256'],record['path']
        assert record['path'] in selected,'not normally Git-selectable: '+record['path']
    for path in ['public-proof-index.json','final-handoff.json']:
        assert str((evidence/path).relative_to(root)) in selected
    prior=json.loads((evidence/'prior-evidence-inventory.json').read_text())['unchanged_historical_files']
    for record in prior: assert digest(root/record['path'])==record['sha256'],record['path']
    report=root/'docs/efforts/team-control-plane/research/jj-workspace-evaluation.md'
    links=[]
    for target in re.findall(r'\]\(([^)]+)\)',report.read_text()):
        if '://' in target or target.startswith('#'): continue
        local=(report.parent/target.split('#')[0]).resolve(); assert local.is_file(),target; links.append(target)
    diff=subprocess.run(['git','diff','--check'],capture_output=True,text=True); assert diff.returncode==0,diff.stderr
    print(json.dumps({'outcome':'passed','indexed_records':len(index['records']),'normal_git_selectable':len(index['records'])+2,'historical_unchanged':len(prior),'report_links_resolved':len(links),'fixture_sha256':handoff['final_fixture_sha256'],'source_freeze_sha256':handoff['source_freeze']['sha256'],'public_proof_index_sha256':handoff['public_proof_index']['sha256'],'handoff_sha256':digest(evidence/'final-handoff.json')},indent=2))

if __name__=='__main__': main()
