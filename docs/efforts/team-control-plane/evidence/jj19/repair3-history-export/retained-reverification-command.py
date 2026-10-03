#!/usr/bin/env python3
"""Reimport saved repair3 candidates after their source fixtures have been removed."""
from pathlib import Path
import argparse,hashlib,importlib.util,json

def main():
    parser=argparse.ArgumentParser(); parser.add_argument('--binary',required=True); parser.add_argument('--output',type=Path,required=True); args=parser.parse_args()
    root=Path.cwd(); fixture=root/'docs/efforts/team-control-plane/research/probes/jj_workspace_evaluation.py'
    spec=importlib.util.spec_from_file_location('jj_fixture',fixture); module=importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    evaluation=module.Evaluation(args.binary,False); code=0
    evidence=Path(__file__).resolve().parent; checks=[]; evaluation.result['observations']['retained_reverification']=checks
    try:
        for directory in ('final-history-artifacts','final-identity-artifacts'):
            folder=evidence/directory
            bundles={hashlib.sha256(p.read_bytes()).hexdigest():p for p in folder.glob('*.bundle')}
            for receipt in sorted(folder.glob('verification-*.json')):
                proof=json.loads(receipt.read_text()); identifier=proof.pop('proof_id')
                assert identifier=='sha256:'+hashlib.sha256(json.dumps(proof,sort_keys=True).encode()).hexdigest()
                assert hashlib.sha256(Path(proof['suite']['path']).read_bytes()).hexdigest()==proof['suite']['sha256']
                assert hashlib.sha256(Path(proof['specification']['path']).read_bytes()).hexdigest()==proof['specification']['sha256']
                bundle=bundles[proof['bundle_sha256']]
                heads=evaluation.command(['git','-c','core.hooksPath=/dev/null','bundle','list-heads',str(bundle)]).splitlines()
                assert len(heads)==1
                commit,ref=heads[0].split(); assert commit==proof['candidate']['commit']
                verified=evaluation.verify_candidate(proof['candidate'],bundle,proof['command'][-1],str(len(checks)),ref.removeprefix('refs/fixture/'))
                fresh=json.loads(verified.read_text())
                checks.append({'retained_receipt':str(receipt.relative_to(root)),'retained_receipt_sha256':hashlib.sha256(receipt.read_bytes()).hexdigest(),'original_proof_id':identifier,'bundle':str(bundle.relative_to(root)),'bundle_sha256':proof['bundle_sha256'],'fresh_proof':fresh})
        assert len(checks)==4
        evaluation.result['outcome']='passed'
    except Exception as error:
        evaluation.result.update(outcome='failed',failure=str(error),failure_type=type(error).__name__); code=1
    finally:
        evaluation.result['durable_preexecution_journal']=(evaluation.root/'operations.jsonl').read_text(); evaluation.tmp.cleanup()
        evaluation.result.update(slice='retained-artifact-reverification',fixture_removed=not evaluation.root.exists(),command_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
        args.output.write_text(json.dumps(evaluation.result,indent=2)+'\n')
    print(json.dumps({k:evaluation.result[k] for k in ('outcome','fixture_sha256','fixture_removed','failure') if k in evaluation.result},indent=2)); return code

if __name__=='__main__': raise SystemExit(main())
