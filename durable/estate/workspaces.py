"""Attempt-bound workspace inventory in the runtime journal.

A lease expires through runtime cleanup proof, never elapsed time. Only this
owner can reclaim a workspace, under the same controller lock as execution.
"""
from __future__ import annotations

from contextlib import contextmanager
import fcntl
import hashlib
import json
import os
from pathlib import Path
import shutil
import socket

from durable.common import digest, durable_mkdir, encode, fsync_dir, read_json
from durable.estate.git import commit, git, tracked_tree, unchanged


class EstateRefused(ValueError):
    pass


def ordinary(path):
    path = Path(path)
    if not path.is_absolute() or '..' in path.parts or any(p.is_symlink() for p in (path, *path.parents)):
        raise EstateRefused('estate paths must be absolute and contain no symlinks or traversal')
    return path


def inode(path):
    info = path.stat()
    return [info.st_dev, info.st_ino]


def host_identity():
    machine = Path('/etc/machine-id')
    value = machine.read_text().strip() if machine.is_file() else socket.gethostname()
    return digest(['estate-host-v1', value])


def materialize(bundle, base, destination):
    """Payload-side clone primitive; conveys no access to the owner journal."""
    destination = Path(destination)
    if destination.is_symlink() or (destination.exists() and (not destination.is_dir() or any(destination.iterdir()))):
        raise EstateRefused('workspace materialization requires a fresh destination')
    git(destination.parent, 'clone', '--no-checkout', '--', str(bundle), str(destination))
    git(destination, 'checkout', '--detach', base)


class Estate:
    def __init__(self, store):
        self.store = store
        self.host = host_identity()
        store.db.executescript('''
        CREATE TABLE IF NOT EXISTS estate_repositories (
          id TEXT PRIMARY KEY, identity TEXT NOT NULL UNIQUE);
        CREATE TABLE IF NOT EXISTS estate_workspaces (
          id TEXT PRIMARY KEY, handle TEXT NOT NULL UNIQUE,
          attempt_id TEXT NOT NULL UNIQUE REFERENCES attempts(id),
          status TEXT NOT NULL, parent_identity TEXT NOT NULL,
          workspace_identity TEXT, retained TEXT);
        ''')

    @contextmanager
    def exclusive(self):
        fd = os.open(ordinary(self.store.root / 'controller.lock'), os.O_RDWR | os.O_CREAT | os.O_NOFOLLOW, 0o600)
        try:
            try:
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            except BlockingIOError as exc:
                raise EstateRefused('runtime or guardian still owns the estate') from exc
            yield
        finally:
            os.close(fd)

    def register(self, repository):
        root = ordinary(Path(repository).absolute())
        if Path(git(root, 'rev-parse', '--show-toplevel').decode().strip()).resolve() != root:
            raise EstateRefused('repository must name its working-tree root')
        admin = ordinary(Path(git(root, 'rev-parse', '--absolute-git-dir').decode().strip()))
        identity = {'host': self.host, 'path': str(root), 'root': inode(root),
                    'git_directory': str(admin), 'git_identity': inode(admin)}
        rid = digest(identity)
        with self.store.transaction() as db:
            db.execute('INSERT OR IGNORE INTO estate_repositories VALUES(?,?)', (rid, encode(identity)))
        return rid

    def reserve(self, attempt_id, repository_id, base, purpose):
        """Runtime calls while holding its lock, before releasing the guardian."""
        att = self.store.db.execute('''SELECT a.*,o.run_id,o.active_attempt FROM attempts a
            JOIN operations o ON o.id=a.operation_id WHERE a.id=?''', (attempt_id,)).fetchone()
        if att is None or att['status'] != 'ACTIVE' or att['active_attempt'] != attempt_id or att['released']:
            raise EstateRefused('workspace requires the current unreleased attempt lease')
        repository = self.store.db.execute('SELECT identity FROM estate_repositories WHERE id=?', (repository_id,)).fetchone()
        if repository is None or json.loads(repository[0])['host'] != self.host:
            raise EstateRefused('unknown repository or different host')
        parent = ordinary(self.store.root / 'attempts' / attempt_id / 'payload')
        if str(parent.parent) != att['directory']:
            raise EstateRefused('attempt directory differs from managed estate')
        handle = {'schema': 1, 'repository': repository_id, 'base': base,
                  'run': att['run_id'], 'attempt': attempt_id, 'epoch': att['epoch'],
                  'host': self.host, 'path': str(parent / 'workspace'), 'purpose': purpose}
        wid = digest(handle)
        previous = self.store.db.execute('SELECT * FROM estate_workspaces WHERE attempt_id=?', (attempt_id,)).fetchone()
        if previous:
            if previous['handle'] != encode(handle):
                raise EstateRefused('attempt already owns a different workspace lease')
            self._finish_allocation(previous, parent / 'workspace')
            return {'id': wid, **handle}
        # Record the staging directory's inode before publishing it to the
        # guardian. Recovery can finish the atomic rename without adopting an
        # arbitrary directory at the destination.
        stage = ordinary(parent.with_name('estate-allocation'))
        if stage.exists() or parent.exists():
            raise EstateRefused('unknown allocation directory must be preserved')
        durable_mkdir(stage)
        durable_mkdir(stage / 'workspace')
        with self.store.transaction() as db:
            db.execute('INSERT INTO estate_workspaces VALUES(?,?,?,?,?,?,NULL)',
                       (wid, encode(handle), attempt_id, 'ALLOCATING', encode(inode(stage)), encode(inode(stage / 'workspace'))))
            self.store.event('WORKSPACE_RESERVED', att['run_id'], att['operation_id'], attempt_id, workspace=wid)
        row = self.store.db.execute('SELECT * FROM estate_workspaces WHERE id=?', (wid,)).fetchone()
        self._finish_allocation(row, parent / 'workspace')
        return {'id': wid, **handle}

    def _finish_allocation(self, row, path):
        parent = ordinary(path.parent)
        stage = ordinary(parent.with_name('estate-allocation'))
        expected = json.loads(row['parent_identity'])
        if not parent.exists() and stage.exists():
            if inode(stage) != expected:
                raise EstateRefused('allocation staging identity changed')
            stage.rename(parent)
            fsync_dir(parent.parent)
        if not parent.exists() or inode(parent) != expected:
            raise EstateRefused('workspace parent identity changed')
        with self.store.transaction() as db:
            db.execute("UPDATE estate_workspaces SET status='LEASED' WHERE id=? AND status='ALLOCATING'", (row['id'],))

    def _row(self, handle):
        if not isinstance(handle, dict) or 'id' not in handle:
            raise EstateRefused('a complete registered workspace handle is required')
        row = self.store.db.execute('SELECT * FROM estate_workspaces WHERE id=?', (handle['id'],)).fetchone()
        if row is None or handle != {'id': row['id'], **json.loads(row['handle'])} or handle['host'] != self.host:
            raise EstateRefused('workspace handle identity differs from this estate')
        path = ordinary(Path(handle['path']))
        if path != self.store.root / 'attempts' / handle['attempt'] / 'payload/workspace':
            raise EstateRefused('workspace location differs from managed estate')
        if inode(path.parent) != json.loads(row['parent_identity']):
            raise EstateRefused('workspace parent identity changed')
        return row, path

    def handles(self):
        return [{'id': row['id'], **json.loads(row['handle'])} for row in
                self.store.db.execute('SELECT id,handle FROM estate_workspaces ORDER BY id')]

    def _settled(self, handle):
        att = self.store.db.execute('SELECT * FROM attempts WHERE id=?', (handle['attempt'],)).fetchone()
        block = self.store.db.execute('SELECT acknowledged FROM cleanup_blocks WHERE attempt_id=?', (handle['attempt'],)).fetchone()
        if att is None or att['status'] == 'ACTIVE' or (block and not block[0]):
            raise EstateRefused('workspace lease is active or cleanup is unproven')
        if att['released'] and not (block and block[0]):
            receipt = read_json(ordinary(Path(att['directory']) / 'receipt.json'))
            if digest(receipt) != att['receipt_digest'] or receipt.get('cleanup') is not True:
                raise EstateRefused('workspace requires accepted runtime cleanup proof')

    def _checkout(self, row, path):
        if row['workspace_identity'] and inode(path) != json.loads(row['workspace_identity']):
            raise EstateRefused('workspace identity changed')
        admin = ordinary(path / '.git')
        if not admin.is_dir() or (admin / 'objects/info/alternates').exists():
            raise EstateRefused('workspace must have isolated Git administration and objects')
        if (Path(git(path, 'rev-parse', '--show-toplevel').decode().strip()).resolve() != path or
                Path(git(path, 'rev-parse', '--absolute-git-dir').decode().strip()) != admin):
            raise EstateRefused('Git destination identity differs')
        if git(path, 'status', '--porcelain=v1', '--untracked-files=all', '--ignored'):
            raise EstateRefused('dirty workspace must be preserved')
        head = commit(path, 'HEAD')
        if not unchanged(path, tracked_tree(path, head)):
            raise EstateRefused('dirty tracked bytes must be preserved regardless of Git index flags')
        git(path, 'merge-base', '--is-ancestor', json.loads(row['handle'])['base'], head)
        refs = git(path, 'show-ref', '--head').decode()
        return {'commit': head, 'refs': refs}

    def inspect(self, handle):
        row, path = self._row(handle)
        return {'handle': handle, 'status': row['status'], 'exists': path.exists(),
                'retained': json.loads(row['retained']) if row['retained'] else None}

    def retain(self, handle):
        with self.exclusive():
            row, path = self._row(handle)
            self._settled(handle)
            if row['status'] in {'RECLAIMING', 'RECLAIMED'}:
                raise EstateRefused('workspace is already reclaiming or reclaimed')
            subject = self._checkout(row, path)
            root = ordinary(self.store.root / 'estate-retained')
            durable_mkdir(root)
            target = ordinary(root / (handle['id'] + '.bundle'))
            if target.exists():
                if row['retained'] is None:
                    # Interrupted retention is never authority to delete; replace
                    # only this owner's uncommitted archive after validating source.
                    target.unlink()
                else:
                    self._retained(row, subject)
                    return json.loads(row['retained'])
            git(path, 'bundle', 'create', str(target), '--all', 'HEAD')
            with target.open('rb') as stream:
                sha = hashlib.file_digest(stream, 'sha256').hexdigest()
                os.fsync(stream.fileno())
            # Verify the archive's objects independently before granting retention.
            check = ordinary(root / (handle['id'] + '.check'))
            if check.exists():
                raise EstateRefused('interrupted retention check requires inspection')
            git(root, 'clone', '--bare', '--', str(target), str(check))
            try:
                if commit(check, subject['commit']) != subject['commit']:
                    raise EstateRefused('archive does not preserve workspace candidate')
                git(check, 'fsck', '--full')
            finally:
                shutil.rmtree(check)
            fsync_dir(root)
            record = {**subject, 'bundle': str(target), 'sha256': sha}
            with self.store.transaction() as db:
                db.execute("UPDATE estate_workspaces SET status='RETAINED',workspace_identity=?,retained=? WHERE id=?",
                           (encode(inode(path)), encode(record), handle['id']))
                self.store.event('WORKSPACE_RETAINED', handle['run'], attempt=handle['attempt'], workspace=handle['id'])
            return record

    def _retained(self, row, subject):
        if not row['retained']:
            raise EstateRefused('unretained candidate must be preserved')
        record = json.loads(row['retained'])
        target = ordinary(Path(record['bundle']))
        with target.open('rb') as stream:
            sha = hashlib.file_digest(stream, 'sha256').hexdigest()
        if sha != record['sha256'] or any(record[k] != v for k, v in subject.items()):
            raise EstateRefused('retained candidate or workspace changed')

    def reclaim(self, handle, *, failpoint=lambda _point: None):
        with self.exclusive():
            row, path = self._row(handle)
            self._settled(handle)
            if row['status'] == 'RECLAIMED':
                if path.exists():
                    raise EstateRefused('new content occupies a reclaimed workspace')
                return self.inspect(handle)
            trash = ordinary(path.parent / ('.estate-delete-' + handle['id']))
            if row['status'] != 'RECLAIMING':
                if not path.exists():
                    # A claim interrupted before worker release has no candidate.
                    att = self.store.db.execute('SELECT released FROM attempts WHERE id=?', (handle['attempt'],)).fetchone()
                    if att[0]:
                        raise EstateRefused('released workspace is missing; reconcile explicitly')
                    with self.store.transaction() as db:
                        db.execute("UPDATE estate_workspaces SET status='RECLAIMED' WHERE id=?", (handle['id'],))
                    return self.inspect(handle)
                att = self.store.db.execute('SELECT * FROM attempts WHERE id=?', (handle['attempt'],)).fetchone()
                never_started = not att['released']
                if att['receipt_digest']:
                    receipt = read_json(ordinary(Path(att['directory']) / 'receipt.json'))
                    never_started = never_started or (digest(receipt) == att['receipt_digest'] and receipt.get('started') is False)
                if never_started and not any(path.iterdir()):
                    if inode(path) != json.loads(row['workspace_identity']):
                        raise EstateRefused('allocation identity changed')
                    path.rmdir()
                    fsync_dir(path.parent)
                    with self.store.transaction() as db:
                        db.execute("UPDATE estate_workspaces SET status='RECLAIMED' WHERE id=?", (handle['id'],))
                    return self.inspect(handle)
                subject = self._checkout(row, path)
                self._retained(row, subject)
                if trash.exists():
                    raise EstateRefused('unknown cleanup destination exists')
                with self.store.transaction() as db:
                    db.execute("UPDATE estate_workspaces SET status='RECLAIMING',workspace_identity=? WHERE id=?",
                               (encode(inode(path)), handle['id']))
                failpoint('after_intent')
                row, path = self._row(handle)
            expected = json.loads(row['workspace_identity'])
            if path.exists():
                if trash.exists() or inode(path) != expected:
                    raise EstateRefused('cleanup workspace identity changed')
                # Recheck before rename even when replaying a cleanup intent.
                subject = self._checkout(row, path)
                self._retained(row, subject)
                path.rename(trash)
                fsync_dir(path.parent)
                failpoint('after_rename')
            if trash.exists():
                retained = json.loads(row['retained'])
                self._retained(row, {k: retained[k] for k in ('commit', 'refs')})
                if inode(trash) != expected:
                    raise EstateRefused('cleanup destination identity changed')
                shutil.rmtree(trash)
                fsync_dir(path.parent)
            with self.store.transaction() as db:
                db.execute("UPDATE estate_workspaces SET status='RECLAIMED' WHERE id=?", (handle['id'],))
                self.store.event('WORKSPACE_RECLAIMED', handle['run'], attempt=handle['attempt'], workspace=handle['id'])
            return self.inspect(handle)

    def reconcile(self):
        """Resume recorded cleanup only; never infer deletion from an expired lease."""
        results = []
        for handle in self.handles():
            raw = self.store.db.execute('SELECT * FROM estate_workspaces WHERE id=?', (handle['id'],)).fetchone()
            if raw['status'] == 'ALLOCATING':
                with self.exclusive():
                    self._finish_allocation(raw, Path(handle['path']))
            row, path = self._row(handle)
            if row['status'] == 'RECLAIMING':
                results.append(self.reclaim(handle))
            else:
                results.append(self.inspect(handle))
        return results
