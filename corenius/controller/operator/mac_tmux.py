#!/usr/bin/env python3
"""Maintain explicitly owned MAC operator views, without controlling agent work.

Native views are interactive clients. This tool supplies no prompt or settings,
never sends input, and never closes or replaces an existing window.
"""
import argparse
from contextlib import contextmanager
from datetime import datetime, timezone
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import re
import shlex
import stat
import subprocess
import tempfile
import time
import uuid
from urllib.parse import urlsplit

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)

OWNER = '@corenius_operator_owner'
VIEW_OWNER = '@corenius_view_owner'
TERMINAL_STATES = {'terminal', 'completed', 'quiescent', 'stopped', 'done'}
RUNTIME_AVAILABLE_STATUSES = {'observed', 'available', 'ok'}
RUNTIME_UNAVAILABLE_STATUSES = {'unavailable', 'pending'}


class AlreadyRunning(RuntimeError):
    """Another instance owns the view reconciliation lock."""


def read_object(path):
    value = json.loads(Path(path).read_text())
    if not isinstance(value, dict):
        raise ValueError('expected JSON object')
    return value


def text(value, name):
    if not isinstance(value, str) or not value or any(ord(c) < 32 for c in value):
        raise ValueError(name + ' must be a nonempty single-line string')
    return value


def endpoint(value):
    value = text(value, 'endpoint')
    if not (value.startswith('unix:///') or value.startswith('ws://') or value.startswith('wss://')):
        raise ValueError('endpoint must explicitly identify a Unix or WebSocket daemon')
    return value


def observed_time(value):
    """Accept a timezone-bearing ISO timestamp or a finite numeric Unix epoch."""
    if isinstance(value, bool):
        raise ValueError('timestamp cannot be boolean')
    if isinstance(value, (int, float)):
        if not math.isfinite(value):
            raise ValueError('timestamp must be finite')
        return datetime.fromtimestamp(value, timezone.utc)
    stamp = text(value, 'observed_at')
    observed = datetime.fromisoformat(stamp.replace('Z', '+00:00'))
    if observed.tzinfo is None:
        raise ValueError('timestamp requires timezone')
    return observed


def binding(value, default_endpoint):
    if not isinstance(value, dict):
        raise ValueError('binding must be an object')
    thread = text(value.get('thread'), 'thread')
    if str(uuid.UUID(thread)) != thread:
        raise ValueError('thread must be a canonical UUID')
    return {'role': text(value.get('role'), 'role'), 'thread': thread,
            'cwd': text(value.get('cwd'), 'cwd'),
            'endpoint': endpoint(value.get('endpoint', default_endpoint))}


def public_binding(item, external=False):
    fields = {'role', 'thread', 'cwd', 'endpoint'} | ({'window', 'pane'} if external else {'current'})
    allowed = fields if external else fields | {'observed_at'}
    if not isinstance(item, dict) or not fields <= set(item) or not set(item) <= allowed:
        raise ValueError('invalid public binding fields')
    if any(not isinstance(item[field], str) or len(item[field]) > 4096
           for field in ('role', 'thread', 'cwd', 'endpoint')):
        raise ValueError('invalid binding text')
    view = binding(item, None)
    address = urlsplit(view['endpoint'])
    if address.username is not None or address.password is not None or address.query or address.fragment:
        raise ValueError('credential-bearing endpoints are unsupported')
    if address.scheme in ('ws', 'wss') and not address.hostname:
        raise ValueError('WebSocket endpoint requires a host')
    if external:
        for field, pattern in (('window', r'@[0-9]+'), ('pane', r'%[0-9]+')):
            if not isinstance(item[field], str) or not re.fullmatch(pattern, item[field]):
                raise ValueError('invalid external reference ID')
            view[field] = item[field]
    else:
        if not isinstance(item['current'], bool):
            raise ValueError('current must be boolean')
        view['current'] = item['current']
        if 'observed_at' in item:
            observation = item['observed_at']
            if isinstance(observation, str) and len(observation) > 4096:
                raise ValueError('per-binding observation too large')
            observed_time(observation)  # Evidence metadata; only the envelope gates freshness.
            view['binding_observed_at'] = observation
    return view


def validate_config(config):
    if config.get('version') != 1:
        raise ValueError('unsupported config version')
    for name in ('session', 'owner', 'state_dir', 'current_path', 'codex_executable'):
        text(config.get(name), name)
    if not re.fullmatch(r'[A-Za-z0-9_-]+', config['session']):
        raise ValueError('session must contain only letters, digits, underscore or hyphen')
    endpoint(config.get('endpoint'))
    argv = config.get('dashboard_argv')
    if not isinstance(argv, list) or not argv:
        raise ValueError('dashboard_argv must be a literal argument vector')
    for arg in argv:
        text(arg, 'dashboard argument')
    interval = config.get('refresh_seconds', 3)
    if isinstance(interval, bool) or not isinstance(interval, (int, float)) or not 1 <= interval <= 60:
        raise ValueError('refresh_seconds must be between 1 and 60')
    binding(config.get('coordinator'), config['endpoint'])
    extras = config.get('extra_bindings', [])
    if not isinstance(extras, list):
        raise ValueError('extra_bindings must be a list')
    for extra in extras:
        binding(extra, config['endpoint'])
    for field in ('runtime_bindings_path', 'external_views_path'):
        if field in config:
            text(config[field], field)


@contextmanager
def instance_lock(config):
    state = Path(config['state_dir'])
    state.mkdir(mode=0o700, parents=True, exist_ok=True)
    state.chmod(0o700)
    fd = os.open(state / 'maintainer.lock', os.O_CREAT | os.O_RDWR, 0o600)
    try:
        try:
            fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as exc:
            raise AlreadyRunning('operator view maintainer already running') from exc
        yield
    finally:
        os.close(fd)


def atomic_manifest(path, value):
    fd, temporary = tempfile.mkstemp(prefix='.manifest-', dir=path.parent)
    try:
        with os.fdopen(fd, 'w') as output:
            json.dump(value, output, indent=2)
            output.write('\n')
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
    finally:
        if os.path.exists(temporary):
            os.unlink(temporary)


def resume_argv(config, view):
    return [config['codex_executable'], 'resume', '--remote', view['endpoint'],
            '--include-non-interactive', view['thread']]


class Tmux:
    def __init__(self, argv, runner):
        self.argv = argv
        self.runner = runner

    def run(self, *args, check=True):
        result = self.runner(self.argv + list(args))
        if check and result.returncode:
            raise RuntimeError('tmux ' + args[0] + ' failed: ' + result.stderr.strip())
        return result

    def set_window(self, window, option, value):
        self.run('set-option', '-w', '-t', window, option, value)


def runtime_bindings(config, errors):
    """Only the bounded public roster can admit runtime clients; errors expose no data.

    Legacy observed_at/bindings rosters remain supported. Optional status admits
    only observed/available/ok; unavailable/pending retains unknown memberships.
    Optional reason is bounded display data and never copied into the manifest.
    """
    source = {'status': 'disabled', 'observed_at': None, 'digest': None}
    path = config.get('runtime_bindings_path')
    if not path:
        return [], source
    source['status'] = 'degraded'
    try:
        with Path(path).open('rb') as input_file:
            raw = input_file.read(1024 * 1024 + 1)
        if len(raw) > 1024 * 1024:
            raise ValueError('runtime roster too large')
        value = json.loads(raw)
        if (not isinstance(value, dict) or not {'observed_at', 'bindings'} <= set(value)
                or not set(value) <= {'observed_at', 'bindings', 'status', 'reason', 'projection_kind'}):
            raise ValueError('invalid public roster fields')
        declared_status = value.get('status')
        if 'status' in value and (not isinstance(declared_status, str)
                                 or declared_status not in RUNTIME_AVAILABLE_STATUSES | RUNTIME_UNAVAILABLE_STATUSES):
            raise ValueError('invalid public roster status')
        if 'reason' in value:
            reason = text(value['reason'], 'reason')
            if len(reason) > 4096:
                raise ValueError('reason too large')
        if 'projection_kind' in value:
            kind = text(value['projection_kind'], 'projection_kind')
            if len(kind) > 4096:
                raise ValueError('projection_kind too large')
        stamp = value['observed_at']
        observed = observed_time(stamp)
        age = (datetime.now(timezone.utc) - observed).total_seconds()
        if age < -60:
            raise ValueError('timestamp is in the future')
        items = value['bindings']
        if not isinstance(items, list) or len(items) > 256:
            raise ValueError('invalid bindings list')
        views = []
        for item in items:
            views.append(public_binding(item))
        source.update(observed_at=stamp, digest=hashlib.sha256(raw).hexdigest())
        if 'projection_kind' in value:
            source['projection_kind'] = value['projection_kind']
        if declared_status is not None:
            source['declared_status'] = declared_status
        if declared_status in RUNTIME_UNAVAILABLE_STATUSES:
            source['status'] = 'unavailable'
            errors.append('runtime bindings unavailable; prior published clients retained with unknown state')
            return [], source
        if age > 300:
            source['status'] = 'stale'
            errors.append('runtime bindings stale; prior published clients retained with unknown state')
            return [], source
        source['status'] = 'ok'
        return views, source
    except FileNotFoundError:
        source['status'] = 'missing'
    except (OSError, ValueError, TypeError, OverflowError, RecursionError):
        pass
    errors.append('runtime bindings ' + source['status'] + '; public roster unavailable or invalid')
    return [], source


def external_panes(tmux, target, errors):
    """Read only IDs and liveness of panes actually in the configured session."""
    try:
        result = tmux.run('list-panes', '-s', '-t', target, '-F', '#{window_id}\t#{pane_id}\t#{pane_dead}')
        panes = {}
        for line in result.stdout.splitlines():
            window, pane, dead = line.split('\t')
            if not re.fullmatch(r'@[0-9]+', window) or not re.fullmatch(r'%[0-9]+', pane) or dead not in {'0', '1'}:
                raise ValueError('invalid pane metadata')
            panes[(window, pane)] = dead == '0'
        return panes
    except (RuntimeError, ValueError):
        errors.append('external observer pending; session pane metadata unavailable')
        return {}


def external_views(config, panes, errors):
    """Operator references observe existing clients; they never authorize launches."""
    source = {'status': 'disabled', 'observed_at': None, 'digest': None}
    path = config.get('external_views_path')
    if not path:
        return [], source
    source['status'] = 'degraded'
    try:
        with Path(path).open('rb') as handle:
            raw = handle.read(1024 * 1024 + 1)
        if len(raw) > 1024 * 1024:
            raise ValueError('external references too large')
        value = json.loads(raw)
        if not isinstance(value, dict) or set(value) != {'observed_at', 'bindings'}:
            raise ValueError('invalid external reference fields')
        observed = observed_time(value['observed_at'])
        age = (datetime.now(timezone.utc) - observed).total_seconds()
        if age < -60 or not isinstance(value['bindings'], list) or len(value['bindings']) > 256:
            raise ValueError('invalid external references')
        views = [public_binding(item, external=True) for item in value['bindings']]
        keys = {(view['endpoint'], view['thread']) for view in views}
        ids = {(view['window'], view['pane']) for view in views}
        if len(keys) != len(views) or len(ids) != len(views):
            raise ValueError('duplicate external references')
        source.update(observed_at=value['observed_at'], digest=hashlib.sha256(raw).hexdigest())
        if age > 300:
            source['status'] = 'stale'
            errors.append('external references stale; observer admission pending')
            return views, source  # Protect these windows, without admitting references.
        if not all(panes.get((view['window'], view['pane'])) for view in views):
            errors.append('external observer pending; referenced pane unavailable in configured session')
            return views, source
        source['status'] = 'ok'
        return views, source
    except FileNotFoundError:
        source['status'] = 'missing'
    except (OSError, ValueError, TypeError, OverflowError, RecursionError):
        pass
    errors.append('external references ' + source['status'] + '; observer admission pending')
    return [], source


def desired_bindings(config, errors):
    def sourced(view, name, **provenance):
        return dict(view, sources=[dict(source=name, role=view['role'],
                                        current=view['current'], **provenance)])
    wanted = [sourced(dict(binding(config['coordinator'], config['endpoint']), current=True), 'coordinator')]
    wanted.extend(sourced(dict(binding(value, config['endpoint']), current=True), 'extra_bindings')
                  for value in config.get('extra_bindings', []))
    current_known = False
    try:
        current = read_object(config['current_path'])
        state = text(current.get('writer_state'), 'writer_state')
        explicitly_empty = ('writer' in current and current['writer'] is None
                            and 'thread' in current and current['thread'] is None
                            and re.match(r'^no active implementation writer(?:;|$)', state.lower()))
        if not explicitly_empty:
            author = binding({'role': 'worker-1', 'thread': current.get('thread'),
                              'cwd': current.get('worktree')}, config['endpoint'])
            wanted.append(sourced(dict(author, current=state.lower() not in TERMINAL_STATES,
                                       writer_state=state), 'implementation'))
        current_known = True
    except (OSError, ValueError, TypeError) as exc:
        errors.append('CURRENT unavailable or invalid: ' + str(exc))
    runtime, runtime_source = runtime_bindings(config, errors)
    runtime_provenance = {key: runtime_source[key] for key in ('observed_at', 'digest', 'projection_kind')
                          if key in runtime_source}
    for view in runtime:
        provenance = dict(runtime_provenance)
        if 'binding_observed_at' in view:
            provenance['binding_observed_at'] = view['binding_observed_at']
        wanted.append(sourced(view, 'runtime_bindings', **provenance))
    # An operator binding can share a view with another role, but never another store.
    unique = {}
    for item in wanted:
        key = (item['endpoint'], item['thread'])
        if key not in unique:
            unique[key] = item
        else:
            unique[key]['current'] = unique[key]['current'] or item['current']
            for source in item['sources']:
                prior = next((old for old in unique[key]['sources']
                              if (old['source'], old['role']) == (source['source'], source['role'])), None)
                if prior is None:
                    unique[key]['sources'].append(source)
                else:
                    prior['current'] = prior['current'] or source['current']
    return unique, current_known, runtime_source


def retained_sources(prior, current_known, runtime_source):
    """Historical memberships preserve provenance, never grant client admission."""
    retained = []
    for source in prior.get('sources', []):
        if not isinstance(source, dict) or source.get('source') not in {
                'coordinator', 'extra_bindings', 'implementation', 'runtime_bindings'}:
            continue
        try:
            item = {'source': source['source'], 'role': text(source.get('role'), 'source role'), 'current': False}
            if item['source'] == 'implementation' and not current_known:
                item['current'] = None
            if item['source'] == 'runtime_bindings':
                observed_time(source.get('observed_at'))
                item['observed_at'] = source['observed_at']
                item['digest'] = text(source.get('digest'), 'digest')
                if 'projection_kind' in source:
                    item['projection_kind'] = text(source['projection_kind'], 'projection_kind')
                if 'binding_observed_at' in source:
                    observation = source['binding_observed_at']
                    if isinstance(observation, str) and len(observation) > 4096:
                        raise ValueError('per-binding observation too large')
                    observed_time(observation)
                    item['binding_observed_at'] = observation
                if runtime_source['status'] not in {'ok', 'disabled'}:
                    item['current'] = None
            retained.append(item)
        except (ValueError, OSError, OverflowError):
            continue
    return retained


def window_rows(tmux, target):
    fields = ['window', 'name', 'owner', 'role', 'thread', 'endpoint', 'cwd', 'current']
    fmt = '#{window_id}\t#{window_name}\t#{' + VIEW_OWNER + '}\t#{@corenius_role}\t#{@corenius_thread}\t#{@corenius_endpoint}\t#{@corenius_cwd}\t#{@corenius_current}'
    rows = []
    for line in tmux.run('list-windows', '-t', target, '-F', fmt).stdout.splitlines():
        parts = line.split('\t')
        if len(parts) != len(fields):
            raise ValueError('invalid tmux window metadata')
        rows.append(dict(zip(fields, parts)))
    return rows


def adopt(config, tmux, rows, errors, protected=()):
    path = config.get('adopt_manifest_path')
    if not path:
        return
    try:
        old = read_object(path)
        if old.get('session') != config['session']:
            raise ValueError('adoption manifest session differs')
        old_bindings = old.get('bindings', [])
        if not isinstance(old_bindings, list) or any(not isinstance(item, dict) for item in old_bindings):
            raise ValueError('adoption bindings must be a list of objects')
        for item in old_bindings:
            row = next((r for r in rows if r['window'] == item.get('window')), None)
            if not row or row['owner'] == config['owner'] or row['window'] in protected:
                continue
            if row['owner'] or row['role'] != item.get('role') or row['thread'] != item.get('thread'):
                raise ValueError('adoption window identity differs')
            view = binding({'role': item['role'], 'thread': item['thread'],
                            'cwd': item.get('cwd', item.get('workspace')),
                            'endpoint': item.get('endpoint', old.get('daemon_endpoint'))}, config['endpoint'])
            expected = resume_argv(config, view)
            if item.get('argv') != expected:
                raise ValueError('adoption manifest lacks exact endpoint resume evidence')
            commands = tmux.run('list-panes', '-t', row['window'], '-F', '#{pane_start_command}').stdout.splitlines()
            actual = shlex.split(commands[0]) if len(commands) == 1 else []
            # tmux quotes its single shell-command argument as one string.
            if len(actual) == 1:
                actual = shlex.split(actual[0])
            if actual != expected:
                raise ValueError('adoption live launch metadata differs')
            for option, value in ((VIEW_OWNER, config['owner']), ('@corenius_endpoint', view['endpoint']), ('@corenius_cwd', view['cwd'])):
                tmux.set_window(row['window'], option, value)
            row.update(owner=config['owner'], endpoint=view['endpoint'], cwd=view['cwd'])
    except (OSError, ValueError, TypeError, KeyError) as exc:
        errors.append('adoption pending: ' + str(exc))


def launch_problem(config, view):
    if not Path(view['cwd']).is_dir():
        return 'binding cwd unavailable'
    if not os.access(config['codex_executable'], os.X_OK) or not Path(config['codex_executable']).is_file():
        return 'configured Codex executable unavailable'
    if view['endpoint'].startswith('unix://'):
        try:
            if not stat.S_ISSOCK(Path(view['endpoint'][7:]).stat().st_mode):
                return 'existing daemon endpoint is not a socket'
        except OSError:
            return 'existing daemon socket unavailable'
    return None


def _reconcile(config, tmux_argv, runner):
    tmux = Tmux(tmux_argv, runner)
    target = '=' + config['session']
    exists = tmux.run('has-session', '-t', target, check=False).returncode == 0
    if exists:
        owner = tmux.run('show-options', '-v', '-t', target + ':', OWNER, check=False).stdout.strip()
        if owner != config['owner']:
            raise ValueError('session owner differs; refusing unrelated session')
    else:
        tmux.run('new-session', '-d', '-s', config['session'], '-n', 'overview',
                 shlex.join(config['dashboard_argv']))
        tmux.run('set-option', '-t', target + ':', OWNER, config['owner'])
    errors = []
    manifest_path = Path(config['state_dir']) / 'manifest.json'
    previous = {}
    if manifest_path.exists():
        try:
            previous = read_object(manifest_path)
        except (OSError, ValueError) as exc:
            errors.append('previous manifest unavailable or invalid: ' + str(exc))
        if previous and (previous.get('session') != config['session'] or previous.get('owner') != config['owner']):
            raise ValueError('manifest owner or session differs; refusing unrelated state')
    wanted, current_known, runtime_source = desired_bindings(config, errors)
    rows = window_rows(tmux, target)
    prior_external = [view for view in previous.get('bindings', [])
                      if isinstance(view, dict) and view.get('managed') is False]
    panes = external_panes(tmux, target, errors) if config.get('external_views_path') or prior_external else {}
    references, external_source = external_views(config, panes, errors)
    protected = {view['window'] for view in references}
    protected.update(view.get('window') for view in prior_external if isinstance(view.get('window'), str))
    if not config.get('external_views_path'):
        adopt(config, tmux, rows, errors, protected)
    # External observer mode never upgrades core windows via legacy launch probing.
    if not any(r['name'] == 'overview' for r in rows):
        errors.append('overview unavailable; dashboard deployment needed')
    # Metadata is evidence for owned views only; unowned windows are never guessed.
    known = {}
    for row in rows:
        if row['owner'] != config['owner']:
            continue
        try:
            view = binding(row, config['endpoint'])
            key = (view['endpoint'], view['thread'])
            if key in known:
                errors.append('duplicate owned view retained: ' + view['thread'])
                continue
            known[key] = dict(view, window=row['window'], managed=True)
            # Prior source claims enrich an existing validated view; they admit none.
            for prior in previous.get('bindings', []):
                if (isinstance(prior, dict) and prior.get('window') == row['window']
                        and prior.get('endpoint') == view['endpoint'] and prior.get('thread') == view['thread']):
                    if isinstance(prior.get('writer_state'), str):
                        known[key]['writer_state'] = prior['writer_state']
                    known[key]['sources'] = retained_sources(prior, current_known, runtime_source)
        except (ValueError, TypeError):
            errors.append('owned window has invalid binding metadata: ' + row['window'])
    # Keep previously admitted external references visible even when a source or
    # pane disappears. They never become owned windows or replacement launches.
    for prior in prior_external:
        try:
            view = public_binding({field: prior.get(field) for field in
                                   ('role', 'thread', 'endpoint', 'cwd', 'window', 'pane')}, external=True)
            key = (view['endpoint'], view['thread'])
            if key in known:
                continue  # A validated owned view takes precedence.
            membership_source = runtime_source if external_source['status'] == 'ok' else {'status': 'degraded'}
            sources = retained_sources(prior, current_known, membership_source)
            live = panes.get((view['window'], view['pane']), False)
            current = None if not live or external_source['status'] != 'ok' else (
                None if any(source['current'] is None for source in sources) else False)
            known[key] = dict(view, managed=False, current=current, sources=sources,
                              external_reference=prior.get('external_reference', {}))
            if not live:
                errors.append('external observer pending; prior referenced pane unavailable')
        except (ValueError, TypeError):
            errors.append('external observer pending; prior reference metadata invalid')
    if runtime_source['status'] == 'ok' and external_source['status'] == 'ok':
        for reference in references:
            key = (reference['endpoint'], reference['thread'])
            desired = wanted.get(key)
            if key in known and known[key].get('managed') is not False:
                continue
            if not desired or not any(source['source'] == 'runtime_bindings' and source['current'] is True
                                      for source in desired['sources']):
                continue
            known[key] = dict(reference, role=desired['role'], managed=False, current=True,
                              sources=desired['sources'], external_reference={
                                  'observed_at': external_source['observed_at'], 'digest': external_source['digest'],
                                  'role': reference['role']})
    for key, view in wanted.items():
        if key in known or not view['current']:
            continue
        if config.get('external_views_path') and any(source['source'] == 'runtime_bindings'
                                                     for source in view['sources']):
            errors.append('external observer pending; validated existing runtime reference required')
            continue
        # A legacy-looking unowned UUID is ambiguous, not permission to duplicate it.
        if any(r['thread'] == view['thread'] and r['owner'] != config['owner']
               and r['endpoint'] in ('', view['endpoint']) for r in rows):
            errors.append('view pending validated adoption: ' + view['thread'])
            continue
        problem = launch_problem(config, view)
        if problem:
            errors.append('view pending ' + view['thread'] + ': ' + problem)
            continue
        # Set retention in the same tmux command queue, before it handles an
        # immediately exiting client. A unique temporary name targets this window.
        temporary_name = 'mac-view-' + uuid.uuid4().hex
        window = tmux.run('new-window', '-d', '-P', '-F', '#{window_id}', '-t', target + ':',
                          '-n', temporary_name, '-c', view['cwd'], shlex.join(resume_argv(config, view)),
                          ';', 'set-option', '-w', '-t', target + ':=' + temporary_name,
                          'remain-on-exit', 'on').stdout.strip()
        for option, value in ((VIEW_OWNER, config['owner']), ('@corenius_thread', view['thread']),
                              ('@corenius_role', view['role']), ('@corenius_endpoint', view['endpoint']),
                              ('@corenius_cwd', view['cwd']), ('automatic-rename', 'off')):
            tmux.set_window(window, option, value)
        known[key] = dict(view, window=window, managed=True)
    bindings = []
    for key, view in known.items():
        if view.get('managed') is False:
            bindings.append(view)
            continue  # Never tag, rename, respawn, select, or send input to external windows.
        dead = tmux.run('list-panes', '-t', view['window'], '-F', '#{pane_dead}').stdout.splitlines()
        if not dead or any(value != '0' for value in dead):
            errors.append('native view exited; retained without replacement: ' + view['window'])
        desired = wanted.get(key)
        sources = view.get('sources', [])
        if desired:
            view.update(role=desired['role'], current=desired['current'])
            for source in desired['sources']:
                sources = [old for old in sources if (old['source'], old['role']) != (source['source'], source['role'])]
                sources.append(source)
            if 'writer_state' in desired:
                view['writer_state'] = desired['writer_state']
        else:
            view['current'] = None if not current_known and view['role'] == 'worker-1' else False
        if sources:
            view['sources'] = sources
            states = [source['current'] for source in sources]
            view['current'] = True if True in states else (None if None in states else False)
        label = 'current' if view['current'] else ('unknown' if view['current'] is None else 'historical')
        tmux.set_window(view['window'], '@corenius_current', label)
        tmux.set_window(view['window'], '@corenius_role', view['role'])
        tmux.run('rename-window', '-t', view['window'], view['role'] + (' [' + label + ']' if label != 'current' else ''))
        bindings.append(view)
    report = {'session': config['session'], 'owner': config['owner'],
              'observed_at': datetime.now(timezone.utc).isoformat(), 'bindings': bindings,
              'runtime_bindings_source': runtime_source,
              'external_views_source': external_source,
              'health': 'degraded' if errors else 'ok', 'errors': errors}
    atomic_manifest(manifest_path, report)
    return report


def reconcile(config, *, tmux_argv=None, runner=None):
    """One bounded reconciliation; tmux command injection is the isolated test seam."""
    validate_config(config)
    with instance_lock(config):
        return _reconcile(config, tmux_argv or ['tmux'], runner or
                          (lambda argv: subprocess.run(argv, capture_output=True, text=True)))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', required=True)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument('--once', action='store_true')
    mode.add_argument('--watch', action='store_true')
    args = parser.parse_args()
    try:
        config = read_object(args.config)
        validate_config(config)
        if args.once:
            print(json.dumps(reconcile(config)))
        else:
            with instance_lock(config):
                while True:
                    report = _reconcile(config, ['tmux'], lambda argv: subprocess.run(argv, capture_output=True, text=True))
                    print(json.dumps({'observed_at': report['observed_at'], 'health': report['health'], 'errors': report['errors']}), flush=True)
                    time.sleep(config.get('refresh_seconds', 3))
    except KeyboardInterrupt:
        return 0
    except (OSError, ValueError, RuntimeError) as exc:
        parser.exit(1, 'mac_tmux: ' + str(exc) + '\n')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
