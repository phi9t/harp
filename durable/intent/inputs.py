"""Publish private immutable input bundles without exposing another run's bytes."""
import hashlib
import os
from pathlib import Path
import tempfile
from durable.common import atomic_json, digest, durable_mkdir, fsync_dir, verify_input_bundle


def freeze(root, files):
    durable_mkdir(root)
    with tempfile.TemporaryDirectory(dir=root, prefix='.freeze-') as temporary:
        bundle = Path(temporary) / 'bundle'
        bundle.mkdir(mode=0o700)
        manifest = {'schema': 1, 'files': {}}
        for name, data in sorted(files.items()):
            path = bundle / name
            path.parent.mkdir(parents=True, exist_ok=True)
            with path.open('xb') as stream:
                stream.write(data); stream.flush(); os.fsync(stream.fileno())
            path.chmod(0o400)
            manifest['files'][name] = {'sha256': hashlib.sha256(data).hexdigest(), 'size': len(data)}
        identity = digest(manifest)
        atomic_json(bundle/'manifest.json', manifest)
        (bundle/'manifest.json').chmod(0o400)
        for path in sorted(bundle.rglob('*'), reverse=True):
            if path.is_dir():fsync_dir(path)
        fsync_dir(bundle)
        target = root/identity
        try:bundle.rename(target)
        except OSError:
            if not target.exists():raise
        fsync_dir(root)
        verify_input_bundle(root, identity)
        return identity
