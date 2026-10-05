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


def binding(value, default_endpoint):
    if not isinstance(value, dict):
        raise ValueError('binding must be an object')
    thread = text(value.get('thread'), 'thread')
    if str(uuid.UUID(thread)) != thread:
        raise ValueError('thread must be a canonical UUID')
    return {'role': text(value.get('role'), 'role'), 'thread': thread,
            'cwd': text(value.get('cwd'), 'cwd'),
            'endpoint': endpoint(value.get('endpoint', default_endpoint))}


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
    if 'runtime_bindings_path' in config:
        text(config['runtime_bindings_path'], 'runtime_bindings_path')


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
                or not set(value) <= {'observed_at', 'bindings', 'status', 'reason'}):
            raise ValueError('invalid public roster fields')
        declared_status = value.get('status')
        if 'status' in value and (not isinstance(declared_status, str)
                                 or declared_status not in RUNTIME_AVAILABLE_STATUSES | RUNTIME_UNAVAILABLE_STATUSES):
            raise ValueError('invalid public roster status')
        if 'reason' in value:
            reason = text(value['reason'], 'reason')
            if len(reason) > 4096:
                raise ValueError('reason too large')
        stamp = text(value['observed_at'], 'observed_at')
        observed = datetime.fromisoformat(stamp.replace('Z', '+00:00'))
        if observed.tzinfo is None:
            raise ValueError('timestamp requires timezone')
        age = (datetime.now(timezone.utc) - observed).total_seconds()
        if age < -60:
            raise ValueError('timestamp is in the future')
        items = value['bindings']
        if not isinstance(items, list) or len(items) > 256:
            raise ValueError('invalid bindings list')
        views = []
        for item in items:
            if not isinstance(item, dict) or set(item) != {'role', 'thread', 'cwd', 'endpoint', 'current'}:
                raise ValueError('invalid public binding fields')
            if not isinstance(item['current'], bool):
                raise ValueError('current must be boolean')
            if any(not isinstance(item[field], str) or len(item[field]) > 4096
                   for field in ('role', 'thread', 'cwd', 'endpoint')):
                raise ValueError('invalid binding text')
            view = binding(item, None)  # A runtime endpoint never inherits a default.
            address = urlsplit(view['endpoint'])
            if address.username is not None or address.password is not None or address.query or address.fragment:
                raise ValueError('credential-bearing endpoints are unsupported')
            if address.scheme in ('ws', 'wss') and not address.hostname:
                raise ValueError('WebSocket endpoint requires a host')
            views.append(dict(view, current=item['current']))
        source.update(observed_at=stamp, digest=hashlib.sha256(raw).hexdigest())
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
    wanted.extend(sourced(view, 'runtime_bindings', observed_at=runtime_source['observed_at'],
                          digest=runtime_source['digest']) for view in runtime)
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
                for field in ('observed_at', 'digest'):
                    item[field] = text(source.get(field), field)
                if runtime_source['status'] not in {'ok', 'disabled'}:
                    item['current'] = None
            retained.append(item)
        except ValueError:
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


def adopt(config, tmux, rows, errors):
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
            if not row or row['owner'] == config['owner']:
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
    adopt(config, tmux, rows, errors)
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
            known[key] = dict(view, window=row['window'])
            # Prior source claims enrich an existing validated view; they admit none.
            for prior in previous.get('bindings', []):
                if (isinstance(prior, dict) and prior.get('window') == row['window']
                        and prior.get('endpoint') == view['endpoint'] and prior.get('thread') == view['thread']):
                    if isinstance(prior.get('writer_state'), str):
                        known[key]['writer_state'] = prior['writer_state']
                    known[key]['sources'] = retained_sources(prior, current_known, runtime_source)
        except (ValueError, TypeError):
            errors.append('owned window has invalid binding metadata: ' + row['window'])
    for key, view in wanted.items():
        if key in known or not view['current']:
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
        known[key] = dict(view, window=window)
    bindings = []
    for key, view in known.items():
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
