"""Committed tracker dependencies, not ambient working-tree edits, own readiness."""
import subprocess
import tempfile
from pathlib import Path
import unittest

from durable.agents.admission import blob, commit
from durable.intent.records import freeze_records, linked_records
from durable.collaboration.assignment import bind


class RecordTests(unittest.TestCase):
    def test_freezes_committed_dependency_and_assignment(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['git', 'init', '-q', directory], check=True)
            issues = root / '.scratch/demo/issues'
            issues.mkdir(parents=True)
            (issues / '01.md').write_text('Status: done\n## Blocked by\nNone.\n')
            task = '.scratch/demo/issues/02.md'
            (root / task).write_text('Status: in-progress\n## Blocked by\n- [first](01.md)\n## Verification\n')
            subprocess.run(['git', '-C', directory, 'add', '.scratch'], check=True)
            subprocess.run(['git', '-C', directory, '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '-qm', 'intent'], check=True)
            revision = commit(root, 'HEAD')
            (issues / '01.md').write_text('Status: blocked\n')
            records = freeze_records(task, blob(root, revision, task), lambda name: blob(root, revision, name))
            self.assertIn(b'Status: done', records['.scratch/demo/issues/01.md'])
            assignment = bind(records, {'allowed_paths': ['src/**']})
            self.assertEqual(assignment['allowed_paths'], ['src/**'])
            self.assertEqual(len(assignment['records']), 2)

    def test_refuses_unfinished_dependency_and_cycle(self):
        path = '.scratch/demo/issues/02.md'
        task = b'Status: in-progress\n## Blocked by\n[first](01.md)\n'
        with self.assertRaisesRegex(ValueError, 'unfinished'):
            freeze_records(path, task, lambda _: b'Status: open\n## Blocked by\nNone.\n')
        with self.assertRaisesRegex(ValueError, 'cycle'):
            freeze_records(path, task, lambda _: b'Status: done\n## Blocked by\n[back](02.md)\n')

    def test_rejects_external_or_escaping_authority(self):
        for target in ('https://example.invalid/issue', '../../../../outside.md'):
            with self.subTest(target=target), self.assertRaises(ValueError):
                freeze_records('.scratch/demo/issues/01.md', ('## Blocked by\n[x](' + target + ')\n').encode(), lambda _: b'')

    def test_freezes_transitive_done_dependency_chain(self):
        task = '.scratch/demo/issues/03.md'
        records = {
            task: b'Status: in-progress\n## Blocked by\n[second](02.md)\n',
            '.scratch/demo/issues/02.md': b'Status: done\n## Blocked by\n[first](01.md)\n',
            '.scratch/demo/issues/01.md': b'Status: done\n## Blocked by\nNone.\n',
        }
        self.assertEqual(freeze_records(task, records[task], records.__getitem__), records)

    def test_freezes_diamond_with_shared_done_dependency(self):
        task = '.scratch/demo/issues/04.md'
        records = {
            task: b'Status: in-progress\n## Blocked by\n[left](02.md)\n[right](03.md)\n',
            '.scratch/demo/issues/02.md': b'Status: done\n## Blocked by\n[shared](01.md)\n',
            '.scratch/demo/issues/03.md': b'Status: done\n## Blocked by\n[shared](01.md)\n',
            '.scratch/demo/issues/01.md': b'Status: done\n## Blocked by\nNone.\n',
        }
        self.assertEqual(freeze_records(task, records[task], records.__getitem__), records)

    def test_anchor_resolves_to_committed_markdown_dependency(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['git', 'init', '-q', directory], check=True)
            issues = root / '.scratch/demo/issues'
            issues.mkdir(parents=True)
            dependency = '.scratch/demo/issues/01.md'
            done = b'Status: done\n## Blocked by\nNone.\n## Verification\nPassed.\n'
            (root / dependency).write_bytes(done)
            task = '.scratch/demo/issues/02.md'
            data = b'Status: in-progress\n## Blocked by\n[first](01.md#verification)\n'
            (root / task).write_bytes(data)
            subprocess.run(['git', '-C', directory, 'add', '.scratch'], check=True)
            subprocess.run(['git', '-C', directory, '-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '-qm', 'intent'], check=True)
            revision = commit(root, 'HEAD')
            (root / dependency).write_text('Status: open\n')
            committed_task = blob(root, revision, task)
            self.assertEqual(linked_records(task, committed_task), [dependency])
            self.assertEqual(
                freeze_records(task, committed_task, lambda name: blob(root, revision, name)),
                {task: data, dependency: done},
            )

    def test_issue_without_blocked_by_section_refuses_readiness(self):
        task = '.scratch/demo/issues/01.md'
        data = b'Status: in-progress\n## Verification\nPending.\n'
        with self.assertRaisesRegex(ValueError, 'missing.*Blocked by'):
            freeze_records(task, data, {}.__getitem__)

    def test_non_issue_markdown_does_not_infer_dependencies_from_prose(self):
        task = '.scratch/demo/task.md'
        data = b'See [unfinished issue](issues/01.md) for background.\n'
        self.assertEqual(linked_records(task, data), [])
        self.assertEqual(freeze_records(task, data, {}.__getitem__), {task: data})
