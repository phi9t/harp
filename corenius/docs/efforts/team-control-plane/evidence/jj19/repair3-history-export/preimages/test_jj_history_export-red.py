#!/usr/bin/env python3
"""Public export regression: deleting a private path does not authorize its history."""
from __future__ import annotations
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import shutil

SYNTHETIC = 'SYNTHETIC_PRIVATE_HISTORY_DO_NOT_EXPORT\n'


def private_history(evaluation, module):
    repo = evaluation.create_attempt('private-history')
    (repo / '.gitignore').write_text('')
    (repo / 'private.env').write_text(SYNTHETIC)
    ancestor = evaluation.checkpoint(repo, True)
    evaluation.jj(repo, 'new', '-m', 'descendant of synthetic private checkpoint')
    (repo / 'private.env').unlink()
    (repo / '.gitignore').write_text('private.env\n')
    (repo / 'artifact.txt').write_text('verified-original\n')
    evaluation.checkpoint(repo, True)
    evaluation.jj(repo, 'bookmark', 'create', 'private-history-bookmark', '-r', ancestor['commit'])
    candidate = evaluation.freeze_candidate(repo)
    assert candidate['captured_paths'] == ['.gitignore', 'artifact.txt']
    store = evaluation.store(repo)
    before = evaluation.git(store, 'for-each-ref', '--format=%(refname) %(objectname)', bare=True)
    observed = {'ancestor': ancestor, 'clean_candidate': candidate, 'refs_before': before}
    evaluation.result['observations']['private_history'] = observed
    try:
        bundle = evaluation.prepare_integration(repo, candidate, 'private-history')
    except module.Refused as error:
        observed['refusal'] = str(error)
        after = evaluation.git(store, 'for-each-ref', '--format=%(refname) %(objectname)', bare=True)
        observed['refs_after'] = after
        assert before == after, 'refusal published a Git ref'
        assert not (evaluation.root / 'private-history.bundle').exists(), 'refusal published a bundle'
        journal = [json.loads(line) for line in (evaluation.root / 'operations.jsonl').read_text().splitlines()]
        assert any(entry.get('phase') == 'integration refused' and entry.get('candidate') == candidate
                   for entry in journal), 'refusal did not journal the exact candidate'
        return
    imported = evaluation.root / 'private-history-import'; imported.mkdir()
    evaluation.git(imported, 'init', '-q')
    evaluation.git(imported, 'fetch', str(bundle), 'refs/fixture/private-history')
    recovered = evaluation.git(imported, 'show', ancestor['commit'] + ':private.env')
    assert recovered == SYNTHETIC.strip(), 'RED setup did not recover the synthetic ancestor content'
    observed.update(bundle_sha256=hashlib.sha256(bundle.read_bytes()).hexdigest(),
                    recovered_private_content=recovered, recoverable=True)
    raise AssertionError('private ancestor bytes remain recoverable from the exported clean-tip bundle')


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--fixture', type=Path, default=Path(__file__).with_name('jj_workspace_evaluation.py'))
    parser.add_argument('--retain-artifacts', type=Path)
    args = parser.parse_args()
    spec = importlib.util.spec_from_file_location('jj_fixture', args.fixture)
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    evaluation = module.Evaluation(args.binary, False)
    code = 0
    try:
        private_history(evaluation, module)
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
        evaluation.result.update(slice='history-export-contract', fixture_removed=not evaluation.root.exists(),
                                 regression_sha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
        args.output.write_text(json.dumps(evaluation.result, indent=2) + '\n')
    print(json.dumps({key: evaluation.result[key] for key in ('outcome', 'fixture_sha256', 'fixture_removed', 'failure')
                      if key in evaluation.result}, indent=2))
    return code


if __name__ == '__main__':
    raise SystemExit(main())
