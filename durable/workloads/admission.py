"""Freeze a committed workload manifest and source before admitting an effect."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import tempfile

from durable.agents.admission import blob, commit, export_bundle, git, relative_path
from durable.common import atomic_json, digest, fsync_dir, verify_input_bundle

WORKFLOW_VERSION = 'workload-v1'


def manifest(data: bytes) -> dict:
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise ValueError('duplicate workload field')
            result[key] = value
        return result
    value = json.loads(data, object_pairs_hook=unique)
    required = {'schema', 'profile', 'command', 'checks', 'artifacts', 'timeout'}
    if not isinstance(value, dict) or not required <= set(value) or set(value)-required-{'oracle_files','requires'} or type(value['schema']) is not int or value['schema'] != 1:
        raise ValueError('invalid workload manifest fields')
    if value['profile'] not in {'default', 'native'}:
        raise ValueError('unsupported workload profile')
    if type(value['timeout']) not in {int, float} or not 1 <= value['timeout'] <= 3600:
        raise ValueError('workload timeout must be between 1 and 3600 seconds')
    commands = [value['command']]
    if not isinstance(value['checks'], list) or not 1 <= len(value['checks']) <= 32:
        raise ValueError('workload requires independent acceptance checks')
    commands += value['checks']
    for argv in commands:
        if (not isinstance(argv, list) or not argv or not isinstance(argv[0], str) or
                not argv[0].startswith('/') or any(not isinstance(a, str) or not a or '\x00' in a for a in argv)):
            raise ValueError('workload commands must be literal argv with absolute executables')
    if not isinstance(value['artifacts'], list) or not 1 <= len(value['artifacts']) <= 256:
        raise ValueError('workload must declare one through256 artifact files')
    for path in value['artifacts']:
        relative_path(path)
    if len(set(value['artifacts'])) != len(value['artifacts']):
        raise ValueError('duplicate workload artifact')
    oracles = value.get('oracle_files',[])
    if not isinstance(oracles,list) or len(oracles)>32:
        raise ValueError('invalid workload oracle files')
    for name in oracles:
        relative_path(name)
    from durable.intent.policy import validate_policy
    validate_policy({'checks':value['checks'],'allowed_paths':[],'oracle_files':oracles})
    requirements = value.get('requires',{})
    if not isinstance(requirements,dict) or set(requirements)-{'files','executables'}:
        raise ValueError('invalid workload capability requirements')
    from pathlib import PurePosixPath
    for paths in requirements.values():
        if not isinstance(paths,list) or len(paths)>64:
            raise ValueError('invalid workload capability paths')
        for name in paths:
            if (not isinstance(name,str) or not name.startswith('/') or '\\' in name or '\x00' in name or
                    str(PurePosixPath(name))!=name or '..' in PurePosixPath(name).parts):
                raise ValueError('workload capability paths must be canonical absolute paths')
    return value


def submit(store, execution, *, run_id, repository, manifest_path, base='HEAD', task_revision='HEAD'):
    if execution is None or execution.backend != 'cgroup':
        raise ValueError('workload admission requires an admitted Insula cgroup execution')
    execution.verify()
    inputs_root = Path(execution.config.get('inputs_root', ''))
    if inputs_root != store.root / 'inputs':
        raise ValueError('workload inputs must belong to project state')
    repo = Path(repository).resolve(strict=True)
    if Path(git(repo, 'rev-parse', '--show-toplevel').decode().strip()).resolve() != repo:
        raise ValueError('repository must name its Git working-tree root')
    from durable.authority import protect_owner
    protect_owner(store.root, [repo], db=store.db)
    base_id, revision = commit(repo, base), commit(repo, task_revision)
    raw = blob(repo, revision, manifest_path)
    declared = manifest(raw)
    native = execution.config.get('native_manifest', {})
    rootfs = Path(native.get('rootfs', '/nonexistent')).resolve()
    for category, paths in declared.get('requires', {}).items():
        for name in paths:
            tool = rootfs/name.lstrip('/')
            if (not tool.is_file() or not tool.resolve().is_relative_to(rootfs) or
                    category == 'executables' and not os.access(tool,os.X_OK)):
                raise ValueError('required admitted workload capability is unavailable: '+name)
    request = {'schema':1, 'run_id':run_id, 'base':base_id, 'task_revision':revision,
               'manifest_path':manifest_path, 'manifest_sha256':hashlib.sha256(raw).hexdigest(),
               'workflow_version':WORKFLOW_VERSION, 'execution_contract':execution.contract}
    identity = digest(request)
    old = store.db.execute('SELECT args FROM runs WHERE id=?', (run_id,)).fetchone()
    if old:
        args = json.loads(old[0])
        if args.get('identity') != identity:
            raise ValueError('run ID already binds different immutable workload inputs')
        verify_input_bundle(inputs_root, args['inputs'])
        return {'run_id':run_id, **args, 'reused':True}
    with tempfile.TemporaryDirectory(dir=inputs_root, prefix='.workload-') as temporary:
        bundle = Path(temporary) / 'bundle'
        bundle.mkdir(mode=0o700)
        (bundle / 'workload.json').write_bytes(raw)
        atomic_json(bundle / 'request.json', request)
        export_bundle(repo, base_id, bundle / 'source.bundle', temporary)
        for name in declared.get('oracle_files',[]):
            target = bundle/'oracles'/name
            target.parent.mkdir(parents=True,exist_ok=True)
            target.write_bytes(blob(repo,revision,name))
        files = {}
        for path in sorted(p for p in bundle.rglob('*') if p.is_file()):
            with path.open('rb') as stream:
                sha = hashlib.file_digest(stream, 'sha256').hexdigest()
                os.fsync(stream.fileno())
            files[path.relative_to(bundle).as_posix()] = {'sha256':sha, 'size':path.stat().st_size}
            path.chmod(0o400)
        bundle_manifest = {'schema':1, 'files':files}
        inputs = digest(bundle_manifest)
        atomic_json(bundle / 'manifest.json', bundle_manifest)
        (bundle / 'manifest.json').chmod(0o400)
        for directory in sorted((p for p in bundle.rglob('*') if p.is_dir()),key=lambda p:len(p.parts),reverse=True):
            fsync_dir(directory)
        fsync_dir(bundle)
        destination = inputs_root / inputs
        try:
            bundle.rename(destination)
        except OSError:
            if not destination.exists():
                raise
            verify_input_bundle(inputs_root, inputs)
        fsync_dir(inputs_root)
        verify_input_bundle(inputs_root, inputs)
    args = {'identity':identity, 'inputs':inputs, 'timeout':declared['timeout']}
    store.submit(run_id, 'workload', WORKFLOW_VERSION, args, execution.contract)
    return {'run_id':run_id, **args, 'reused':False}
