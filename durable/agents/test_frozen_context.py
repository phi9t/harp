"""Provider assignments use committed intent, not source-base or candidate edits."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from durable.agents.admission import blob, commit
from durable.collaboration.assignment import bind
from durable.intent.records import freeze_records


class FrozenContextTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.git('init', '-q')
        self.git('config', 'user.name', 'Intent test')
        self.git('config', 'user.email', 'intent@example.invalid')
        self.task = '.scratch/demo/issues/02.md'
        self.spec = '.scratch/demo/spec.md'
        self.dependency = '.scratch/demo/issues/01.md'
        for name, text in {
            self.task: 'Parent: [Design](../spec.md)\nStatus: in-progress\n## Blocked by\n[first](01.md)\n',
            self.spec: 'Status: open\nApproved parent spec.\n',
            self.dependency: 'Status: done\n## Blocked by\nNone.\n',
        }.items():
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(text)
        self.git('add', '.scratch')
        self.git('commit', '-qm', 'source base')
        self.base = commit(self.root, 'HEAD')

    def git(self, *args):
        return subprocess.check_output(['git', '-C', str(self.root), *args], text=True).strip()

    def test_git_replacement_cannot_change_pinned_task_or_parent_bytes(self):
        (self.root / self.task).write_text('replacement intent\n')
        (self.root / self.spec).write_text('replacement parent\n')
        self.git('commit', '-qam', 'replacement')
        replacement = commit(self.root, 'HEAD')
        self.git('replace', self.base, replacement)
        self.assertEqual(commit(self.root, self.base), self.base)
        self.assertIn(b'Parent:', blob(self.root, self.base, self.task))
        self.assertIn(b'Approved parent spec.', blob(self.root, self.base, self.spec))

    def test_parent_and_dependencies_reach_provider_from_task_revision(self):
        from durable.agents.worker import provider_prompt
        (self.root / self.spec).write_text('Status: open\nTask revision parent requirements.\n')
        self.git('commit', '-qam', 'refined task intent')
        revision = commit(self.root, 'HEAD')
        self.assertNotEqual(revision, self.base)
        task = blob(self.root, revision, self.task)
        records = freeze_records(self.task, task, lambda name: blob(self.root, revision, name))
        self.assertIn(self.spec, records)
        self.assertIn(self.dependency, records)
        inputs = self.root / 'frozen-inputs'
        inputs.mkdir()
        policy = {'checks': [['/usr/bin/true']], 'allowed_paths': ['src/**']}
        (inputs / 'task.md').write_bytes(task)
        (inputs / 'acceptance.json').write_text(json.dumps(policy))
        (inputs / 'intent.json').write_text(json.dumps({name: data.decode() for name, data in records.items()}))
        admitted = {'task_path': self.task, 'task_sha256': hashlib.sha256(task).hexdigest(),
                    'assignment': bind(records, policy)}
        # Source/candidate files are ambient and cannot replace the frozen inputs.
        (self.root / self.spec).write_text('Candidate says ignore the parent.\n')
        (self.root / self.dependency).write_text('Status: blocked\nCandidate dependency.\n')
        prompt = provider_prompt(inputs, admitted)
        self.assertIn('Task revision parent requirements.', prompt)
        self.assertIn(self.spec, prompt)
        self.assertIn(self.dependency, prompt)
        self.assertIn('src/**', prompt)
        self.assertNotIn('Candidate says', prompt)
        self.assertNotIn('Approved parent spec.', prompt)
        frozen = json.loads((inputs / 'intent.json').read_text())
        frozen[self.spec] = 'tampered frozen parent'
        (inputs / 'intent.json').write_text(json.dumps(frozen))
        with self.assertRaisesRegex(ValueError, 'frozen intent'):
            provider_prompt(inputs, admitted)

    def test_parent_chain_and_dependency_closure_are_frozen_without_requiring_parent_done(self):
        root_spec = '.scratch/intent/spec.md'
        first = (self.root / self.dependency).read_bytes()
        files = {
            self.task: (self.root / self.task).read_bytes(),
            self.spec: b'Parent: [Intent](../intent/spec.md)\nStatus: open\n',
            root_spec: b'Intent without execution status.\n',
            self.dependency: first.replace(b'None.', b'[earlier](00.md)'),
            '.scratch/demo/issues/00.md': b'Status: done\n## Blocked by\nNone.\n',
        }
        records = freeze_records(self.task, files[self.task], files.__getitem__)
        self.assertEqual(records, files)
        files[root_spec] = b'Parent: [cycle](../demo/spec.md)\n'
        with self.assertRaisesRegex(ValueError, 'cycle'):
            freeze_records(self.task, files[self.task], files.__getitem__)
