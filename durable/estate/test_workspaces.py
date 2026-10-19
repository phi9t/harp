"""Real Git/journal contracts for workspace ownership and lossless cleanup."""
import fcntl
import json
import os
from pathlib import Path
import tempfile
import unittest

from durable.common import atomic_json, digest
from durable.estate.git import commit, git
from durable.estate.workspaces import Estate, EstateRefused, materialize
from durable.runtime.store import Store


class EstateTests(unittest.TestCase):
    def setUp(self):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        self.root = Path(temp.name).resolve()
        self.repo = self.root / 'repository'
        self.repo.mkdir()
        git(self.repo, 'init', '-q')
        git(self.repo, 'config', 'user.name', 'Estate qualification')
        git(self.repo, 'config', 'user.email', 'qualification@harp.local')
        (self.repo / 'source.txt').write_text('base\n')
        git(self.repo, 'add', 'source.txt')
        git(self.repo, 'commit', '-qm', 'base')
        self.base = commit(self.repo, 'HEAD')
        self.bundle = self.root / 'source.bundle'
        git(self.repo, 'bundle', 'create', str(self.bundle), 'HEAD')
        self.store = Store(self.root / 'state', allow_unpatched_sqlite=True)
        self.addCleanup(self.store.close)
        self.estate = Estate(self.store)
        self.rid = self.estate.register(self.repo)
        self.store.submit('job', 'coding', 'v1', {}, 'test')
        self.aid = 'a' * 32
        self.directory = self.store.root / 'attempts' / self.aid
        self.directory.mkdir(parents=True)
        with self.store.transaction() as db:
            db.execute("INSERT INTO operations(id,run_id,seq,key,kind,spec,spec_digest,status,active_attempt) VALUES('op','job',0,'coding','ACTIVITY','{}','spec','RUNNING',?)", (self.aid,))
            db.execute("INSERT INTO attempts(id,operation_id,number,epoch,spec_digest,directory,created) VALUES(?,'op',1,1,'spec',?,0)", (self.aid, str(self.directory)))
        self.handle = self.estate.reserve(self.aid, self.rid, self.base, 'coding')
        self.workspace = Path(self.handle['path'])

    def materialize(self):
        materialize(self.bundle, self.base, self.workspace)

    def settle(self, *, released=True):
        receipt = {'cleanup': True, 'attempt_id': self.aid}
        atomic_json(self.directory / 'receipt.json', receipt)
        with self.store.transaction() as db:
            db.execute("UPDATE attempts SET status='SUCCEEDED',released=?,receipt_digest=? WHERE id=?", (int(released), digest(receipt), self.aid))
            db.execute("UPDATE operations SET status='SUCCEEDED',active_attempt=NULL WHERE id='op'")

    def test_exact_handle_archive_and_reclamation_survive_restart(self):
        self.materialize()
        self.settle()
        record = self.estate.retain(self.handle)
        self.assertEqual(record['commit'], self.base)
        restarted = Estate(self.store)
        self.assertEqual(restarted.handles(), [self.handle])
        self.assertEqual(restarted.reclaim(self.handle)['status'], 'RECLAIMED')
        self.assertEqual(restarted.reclaim(self.handle)['status'], 'RECLAIMED')
        recovered = self.root / 'restored'
        git(self.root, 'clone', str(record['bundle']), str(recovered))
        self.assertEqual(commit(recovered, 'HEAD'), self.base)
        self.assertEqual((recovered / 'source.txt').read_text(), 'base\n')

    def test_path_forgery_unknown_ownership_and_destination_replacement_refused(self):
        self.materialize()
        self.settle()
        for forged in (str(self.workspace), {'id':'unknown'}, {**self.handle, 'path':str(self.repo)}, {**self.handle, 'host':'other-host'}, {**self.handle, 'epoch':2}):
            with self.assertRaises(EstateRefused):
                self.estate.reclaim(forged)
        backup = self.workspace.with_name('original')
        self.workspace.rename(backup)
        self.materialize()
        with self.assertRaisesRegex(EstateRefused, 'identity'):
            self.estate.retain(self.handle)
        self.assertTrue((backup / 'source.txt').is_file())
        self.assertTrue((self.workspace / 'source.txt').is_file())

    def test_active_stale_lease_and_cleanup_without_proof_never_authorize_deletion(self):
        self.materialize()
        # A newer runtime epoch does not revoke a live lease.
        self.store.db.execute("INSERT INTO meta VALUES('epoch','999')")
        with self.assertRaisesRegex(EstateRefused, 'active'):
            self.estate.reclaim(self.handle)
        self.settle()
        self.store.db.execute("INSERT INTO cleanup_blocks VALUES(?, 'lost guardian', NULL)", (self.aid,))
        with self.assertRaisesRegex(EstateRefused, 'unproven'):
            self.estate.reclaim(self.handle)
        self.assertTrue(self.workspace.exists())

    def test_dirty_untracked_ignored_and_unretained_candidates_preserved(self):
        self.materialize()
        self.settle()
        with self.assertRaisesRegex(EstateRefused, 'unretained'):
            self.estate.reclaim(self.handle)
        for path in ('source.txt', 'untracked.txt', '.gitignore'):
            target = self.workspace / path
            old = target.read_bytes() if target.exists() else None
            target.write_text('local changes\n')
            with self.assertRaisesRegex(EstateRefused, 'dirty'):
                self.estate.retain(self.handle)
            self.assertEqual(target.read_text(), 'local changes\n')
            if old is None: target.unlink()
            else: target.write_bytes(old)
        (self.workspace / '.git/info/exclude').write_text('ignored\n')
        (self.workspace / 'ignored').write_text('private work')
        with self.assertRaisesRegex(EstateRefused, 'dirty'):
            self.estate.reclaim(self.handle)

    def test_cleanup_intent_and_rename_replay_are_idempotent(self):
        self.materialize()
        self.settle()
        self.estate.retain(self.handle)
        class Crash(Exception): pass
        for point in ('after_intent', 'after_rename'):
            def crash(actual):
                if actual == point: raise Crash()
            if point == 'after_rename':
                # Resume the existing intent and interrupt immediately after rename.
                pass
            with self.assertRaises(Crash):
                self.estate.reclaim(self.handle, failpoint=crash)
        self.assertFalse(self.workspace.exists())
        outcome = Estate(self.store).reconcile()
        self.assertEqual(outcome[0]['status'], 'RECLAIMED')

    def test_interrupted_allocation_reconciles_without_touching_other_paths(self):
        self.settle(released=False)
        self.assertEqual(Estate(self.store).reclaim(self.handle)['status'], 'RECLAIMED')
        self.assertTrue((self.repo / 'source.txt').exists())

    def test_retention_tampering_and_post_retention_changes_refused(self):
        self.materialize()
        self.settle()
        record = self.estate.retain(self.handle)
        target = Path(record['bundle'])
        original = target.read_bytes()
        target.write_bytes(b'corrupt')
        with self.assertRaisesRegex(EstateRefused, 'changed'):
            self.estate.reclaim(self.handle)
        target.write_bytes(original)
        git(self.workspace, '-c', 'user.name=Qualification', '-c', 'user.email=q@harp.local', 'commit', '--allow-empty', '-qm', 'new candidate')
        with self.assertRaisesRegex(EstateRefused, 'changed'):
            self.estate.reclaim(self.handle)

    def test_runtime_lock_and_symlink_replacement_refused(self):
        self.materialize()
        self.settle()
        fd = os.open(self.store.root / 'controller.lock', os.O_CREAT | os.O_RDWR, 0o600)
        try:
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            with self.assertRaisesRegex(EstateRefused, 'guardian'):
                self.estate.reclaim(self.handle)
        finally:
            os.close(fd)
        old = self.workspace.with_name('old')
        self.workspace.rename(old)
        self.workspace.symlink_to(self.repo, target_is_directory=True)
        with self.assertRaisesRegex(EstateRefused, 'symlinks'):
            self.estate.reclaim(self.handle)
        self.assertTrue((self.repo / 'source.txt').exists())

    def test_registered_allocation_rename_recovers_and_conflicting_lease_refuses(self):
        parent = self.workspace.parent
        stage = parent.with_name('estate-allocation')
        parent.rename(stage)
        self.store.db.execute("UPDATE estate_workspaces SET status='ALLOCATING' WHERE id=?", (self.handle['id'],))
        result = Estate(self.store).reconcile()
        self.assertTrue(self.workspace.is_dir())
        self.assertEqual(result[0]['status'], 'LEASED')
        self.assertEqual(self.estate.reserve(self.aid, self.rid, self.base, 'coding'), self.handle)
        with self.assertRaisesRegex(EstateRefused, 'different workspace'):
            self.estate.reserve(self.aid, self.rid, self.base, 'workload')

    def test_cleanup_recovery_requires_the_retained_archive_to_still_exist(self):
        self.materialize()
        self.settle()
        record = self.estate.retain(self.handle)
        class Crash(Exception): pass
        def crash(point):
            if point == 'after_rename': raise Crash()
        with self.assertRaises(Crash):
            self.estate.reclaim(self.handle, failpoint=crash)
        archive = Path(record['bundle'])
        original = archive.read_bytes()
        archive.write_bytes(b'corrupted after cleanup intent')
        with self.assertRaisesRegex(EstateRefused, 'changed'):
            Estate(self.store).reconcile()
        trash = self.workspace.parent / ('.estate-delete-' + self.handle['id'])
        self.assertEqual((trash / 'source.txt').read_text(), 'base\n')
        archive.write_bytes(original)
        self.assertEqual(Estate(self.store).reconcile()[0]['status'], 'RECLAIMED')

    def test_index_flags_cannot_hide_dirty_bytes_from_retention_or_cleanup(self):
        self.materialize()
        self.settle()
        self.estate.retain(self.handle)
        git(self.workspace, 'update-index', '--assume-unchanged', 'source.txt')
        (self.workspace / 'source.txt').write_text('valuable local changes\n')
        self.assertEqual(git(self.workspace, 'status', '--porcelain'), b'')
        for operation in (self.estate.retain, self.estate.reclaim):
            with self.assertRaisesRegex(EstateRefused, 'dirty tracked bytes'):
                operation(self.handle)
        self.assertEqual((self.workspace / 'source.txt').read_text(), 'valuable local changes\n')

    def test_released_but_never_started_allocation_can_be_reclaimed(self):
        receipt = {'cleanup': True, 'started': False, 'attempt_id': self.aid}
        atomic_json(self.directory / 'receipt.json', receipt)
        with self.store.transaction() as db:
            db.execute("UPDATE attempts SET status='FAILED',released=1,receipt_digest=? WHERE id=?", (digest(receipt), self.aid))
            db.execute("UPDATE operations SET status='PENDING',active_attempt=NULL WHERE id='op'")
        self.assertEqual(self.estate.reclaim(self.handle)['status'], 'RECLAIMED')
        self.assertFalse(self.workspace.exists())
