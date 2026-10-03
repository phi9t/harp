#!/usr/bin/env python3
"""Real jj regression: verification must precede rewrite and bind immutable proof."""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil


def assert_contract(result):
    events = result['events']
    rewrite = result['observations']['rewrite']
    changed = rewrite['after']
    rewritten_read = next(i for i, event in enumerate(events)
                          if 'log' in event.get('argv', [])
                          and changed['commit'] in event.get('stdout', ''))
    first_fetch = next(i for i, event in enumerate(events)
                       if 'fetch' in event.get('argv', [])
                       and event['cwd'] != changed['workspace'])
    assert first_fetch < rewritten_read, 'first independent verifier fetch occurs after content rewrite'
    rewrite_start = next(i for i, event in enumerate(events)
                         if event.get('journal') == 'content rewrite prepared')
    proofs = result['observations']['verification_proofs']
    assert len(proofs) == 3, 'original, changed and retained original must actually execute verification'
    before, after, retained = proofs
    assert before['candidate']['commit'] == rewrite['before']['commit']
    assert before['candidate']['tree'] == rewrite['before']['tree']
    assert before['command_event'] < rewrite_start < rewritten_read < after['command_event']
    assert events[rewrite_start]['verified_proof'] == before['proof_id']
    assert after['candidate']['commit'] == changed['commit']
    assert after['candidate']['tree'] == changed['tree']
    assert after['proof_id'] != before['proof_id']
    assert retained['candidate']['commit'] == before['candidate']['commit']
    assert retained['candidate']['tree'] == before['candidate']['tree']
    for proof in proofs:
        assert proof['verifier']['workspace'] != proof['candidate']['workspace']
        assert proof['candidate']['base_commit']
        assert proof['specification']['sha256'] and proof['suite']['sha256']
        assert proof['environment']['sha256'] and proof['environment']['values']
        assert proof['environment']['values']['HOME'] == str(Path(proof['verifier']['workspace']) / 'private-home')
        actual = events[proof['command_event']]
        assert actual['argv'] == proof['command'] and actual == proof['result']
        assert actual['exit'] == 0
        measured = json.loads(actual['stdout'])
        assert measured['outcome'] == 'passed'
        assert measured['artifact_sha256'] == hashlib.sha256(
            ('rewritten-source\n' if proof is after else 'verified-original\n').encode()).hexdigest()
    reuse = result['observations']['verification_reuse']
    assert any(r['proof_id'] == before['proof_id'] and r['candidate_commit'] == changed['commit']
               and r['disposition'] == 'refused' for r in reuse), 'recorded old proof accepted changed candidate'
    assert any(r['proof_id'] == after['proof_id'] and r['candidate_commit'] == changed['commit']
               and r['disposition'] == 'accepted' for r in reuse)
    metadata = result['observations']['metadata_rewrite']
    assert any(r['proof_id'] == after['proof_id'] and r['candidate_commit'] == metadata['commit']
               and r['disposition'] == 'refused' for r in reuse), 'same-tree metadata rewrite reused old commit proof'


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, default=Path(__file__).with_name('jj_workspace_evaluation.py'))
    parser.add_argument('--retain-artifacts', type=Path)
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location('jj_fixture', args.fixture)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    evaluation = module.Evaluation(args.binary, False)
    code = 0
    try:
        evaluation.identity_slice()
        evaluation.result['evaluation_outcome'] = 'passed'
        assert_contract(evaluation.result)
        evaluation.result['outcome'] = 'passed'
    except Exception as error:
        evaluation.result.update(outcome='failed', failure=str(error), failure_type=type(error).__name__)
        code = 1
    finally:
        if args.retain_artifacts:
            args.retain_artifacts.mkdir(parents=True, exist_ok=False)
            for p in sorted(evaluation.root.glob('*.bundle')) + sorted(evaluation.root.glob('verification-*.json')):
                shutil.copyfile(p, args.retain_artifacts / p.name)
        evaluation.result['durable_preexecution_journal'] = (evaluation.root / 'operations.jsonl').read_text()
        evaluation.tmp.cleanup()
        evaluation.result.update(slice='identity-verification-contract', fixture_removed=not evaluation.root.exists(),
                                 regression_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
        args.output.write_text(json.dumps(evaluation.result, indent=2) + '\n')
    print(json.dumps({k: evaluation.result[k] for k in ('outcome', 'fixture_sha256', 'fixture_removed', 'failure')
                      if k in evaluation.result}, indent=2))
    return code


if __name__ == '__main__':
    raise SystemExit(main())
