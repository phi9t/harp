#!/usr/bin/env python3
"""Read-only MAC operator dashboard. JSON projections are data, never instructions.

Run with --config PATH; --render prints all sections without using curses or tmux.
Only an explicit Enter on an agent view selects an existing tmux window.
"""
import argparse
import sys
import curses
import datetime as dt
import hashlib
import json
import math
import pathlib
import re
import subprocess
import textwrap
import time
import unicodedata
from zoneinfo import ZoneInfo

MAX_BYTES = 1024 * 1024
STALE_SECONDS = 300
SECTION_NAMES = ('Overview', 'Tasks', 'Agents', 'Work', 'Evidence', 'VCS', 'Infra')
HUMAN_ZONE = ZoneInfo('America/Los_Angeles')
ANSI = re.compile(r'\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07\x1b]*(?:\x07|\x1b\\))')


def safe(value):
    """Bound display text and remove terminal controls including bidi controls."""
    if isinstance(value, (dict, list)):
        value = json.dumps(value, ensure_ascii=False)
    value = ANSI.sub('', str(value)[:4096])
    return ''.join(' ' if unicodedata.category(c).startswith('C') else c for c in value)


def read_json(path):
    with pathlib.Path(path).open('rb') as handle:
        raw = handle.read(MAX_BYTES + 1)
    if len(raw) > MAX_BYTES:
        raise ValueError('exceeds 1 MiB bound')
    data = json.loads(raw)
    if not isinstance(data, dict):
        raise ValueError('expected JSON object')
    return data, hashlib.sha256(raw).hexdigest()


def load_config(path):
    config, _ = read_json(path)
    if config.get('version', 1) != 1:
        raise ValueError('unsupported config version')
    for key in ('session', 'owner', 'state_dir', 'current_path', 'luna_path',
                'codex_executable', 'endpoint'):
        if not isinstance(config.get(key), str) or not config[key]:
            raise ValueError('config requires nonempty ' + key)
    if not isinstance(config.get('coordinator'), dict):
        raise ValueError('config requires coordinator object')
    if not isinstance(config.get('extra_bindings'), list):
        raise ValueError('config requires extra_bindings list')
    if not isinstance(config.get('dashboard_argv'), list):
        raise ValueError('config requires dashboard_argv list')
    if 'agent_roster_path' in config and (not isinstance(config['agent_roster_path'], str) or not config['agent_roster_path']):
        raise ValueError('agent_roster_path must be a nonempty path')
    if 'goal' in config and not isinstance(config['goal'], str):
        raise ValueError('config goal must be text')
    refresh = config.get('refresh_seconds', 3)
    if isinstance(refresh, bool) or not isinstance(refresh, (int, float)) or not math.isfinite(refresh) or refresh <= 0:
        raise ValueError('refresh_seconds must be a positive number')
    config['refresh_seconds'] = max(0.25, refresh)
    return config


def timestamp(value):
    if not isinstance(value, bool) and isinstance(value, (int, float)):
        try:
            if not math.isfinite(value):
                raise ValueError('timestamp must be finite')
            return dt.datetime.fromtimestamp(value, dt.timezone.utc)
        except (OverflowError, OSError, ValueError):
            raise ValueError('timestamp must be finite and within datetime range') from None
    if not isinstance(value, str):
        raise ValueError('timestamp unavailable')
    parsed = dt.datetime.fromisoformat(value.replace('Z', '+00:00'))
    if parsed.tzinfo is None:
        raise ValueError('timestamp requires timezone')
    return parsed


def validate(data, kind, config):
    stamp = 'checked_at' if kind == 'Luna' else 'observed_at'
    timestamp(data.get(stamp))
    shapes = {
        'CURRENT': {'tickets': dict, 'focused_results': dict, 'context': dict,
                    'caps': dict, 'declared_services': dict, 'product_bindings': (dict, list),
                    'unknowns': list, 'deferred': list, 'pending_gates': list,
                    'questions': list, 'progress': (dict, list, str),
                    **dict.fromkeys(('goal', 'next', 'version', 'source', 'base', 'thread',
                                     'writer', 'worktree', 'writer_state', 'ack'), (str, type(None))),
                    'candidate': (dict, str, type(None)), 'landing': (dict, str, type(None))},
        'Luna': {'observed_services': dict, 'source_identities': dict,
                 'actionable_findings': list, 'limitations': list,
                 'progress': (dict, list, str), 'next_checks': list, 'phase': str},
        'Manifest': {'bindings': list, 'errors': list},
        'Roster': {'agents': list},
    }
    for key, shape in shapes[kind].items():
        if key in data and not isinstance(data[key], shape):
            raise ValueError('wrong shape for ' + key)
    if kind == 'Manifest':
        if data.get('session') != config['session']:
            raise ValueError('manifest session mismatch')
        if 'bindings' not in data:
            raise ValueError('bindings unavailable')
        for binding in data['bindings']:
            if not isinstance(binding, dict):
                raise ValueError('binding must be an object')
            for key in ('role', 'thread', 'endpoint', 'cwd', 'window'):
                if not isinstance(binding.get(key), str) or not binding[key]:
                    raise ValueError('binding requires ' + key)
            if 'current' not in binding or (binding['current'] is not None and not isinstance(binding['current'], bool)):
                raise ValueError('binding current must be boolean or null')
            if not re.fullmatch(r'@[0-9]+', binding['window']):
                raise ValueError('invalid tmux window ID')
    if kind == 'Roster':
        if 'agents' not in data:
            raise ValueError('agents unavailable')
        for agent in data['agents']:
            if not isinstance(agent, dict):
                raise ValueError('roster agent must be an object')
            for key in ('name', 'scope', 'status', 'model'):
                if not isinstance(agent.get(key), str) or not agent[key]:
                    raise ValueError('roster agent requires ' + key)
            for key in ('thread', 'endpoint', 'worktree', 'ticket'):
                if key in agent and not isinstance(agent[key], str):
                    raise ValueError('roster agent field must be text: ' + key)


def load_snapshot(config, now=None):
    now = now or dt.datetime.now(dt.timezone.utc)
    sources = {}
    for kind, path in [('CURRENT', config['current_path']), ('Luna', config['luna_path']),
                       ('Manifest', str(pathlib.Path(config['state_dir']) / 'manifest.json')),
                       ('Roster', config.get('agent_roster_path'))]:
        source = {'path': path, 'data': {}, 'digest': '', 'error': ''}
        try:
            if path is None:
                raise ValueError('not configured')
            data, source['digest'] = read_json(path)
            validate(data, kind, config)
            source['data'] = data
            source['time'] = timestamp(data['checked_at' if kind == 'Luna' else 'observed_at'])
        except (OSError, ValueError, TypeError, RecursionError) as error:
            source['error'] = type(error).__name__ + ': ' + safe(error)
        sources[kind] = source
    return {'config': config, 'now': now, 'sources': sources}


def provenance(snapshot, kind):
    source = snapshot['sources'][kind]
    if source['error']:
        return kind + ': unavailable | ' + source['error']
    age = (snapshot['now'] - source['time']).total_seconds()
    status = 'STALE (>5m)' if age > STALE_SECONDS else ('freshness unavailable (future timestamp)' if age < -60 else 'fresh')
    return (f'{kind}: {status} | age {int(age)}s | source timestamp '
            f'{source["data"].get("checked_at", source["data"].get("observed_at"))} | '
            f'LA {source["time"].astimezone(HUMAN_ZONE).isoformat()} | sha256:{source["digest"]}')


def field(label, obj, key):
    value = obj.get(key)
    return label + ': ' + ('unavailable' if key not in obj else 'none' if value is None else safe(value))


def collection(label, obj, key):
    value = obj.get(key)
    if value is None:
        return [label + ': unavailable']
    if not value:
        return [label + ': none']
    if isinstance(value, dict):
        lines = [f'  {safe(k)}: {safe(v)}' for k, v in list(value.items())[:100]]
    elif isinstance(value, list):
        lines = ['  ' + safe(v) for v in value[:100]]
    else:
        lines = ['  ' + safe(value)]
    if isinstance(value, (dict, list)) and len(value) > 100:
        lines.append('  [display limited to 100 entries]')
    return [label + ':'] + lines


def bindings(snapshot):
    return snapshot['sources']['Manifest']['data'].get('bindings', [])


def sections(snapshot):
    cur = snapshot['sources']['CURRENT']['data']
    luna = snapshot['sources']['Luna']['data']
    manifest = snapshot['sources']['Manifest']['data']
    overview = [field('Goal (CURRENT)', cur, 'goal'), field('Next (CURRENT)', cur, 'next'),
                field('Phase (Luna observation)', luna, 'phase')]
    if cur.get('goal') is None and snapshot['config'].get('goal'):
        overview.insert(1, field('Goal (operator config)', snapshot['config'], 'goal'))
    overview += collection('Blockers / unknowns (CURRENT)', cur, 'unknowns')
    overview += [provenance(snapshot, kind) for kind in ('CURRENT', 'Luna', 'Manifest')]
    tasks = [field('Projection version (CURRENT)', cur, 'version'),
             field('Pinned source', cur, 'source')] + collection('Tickets (CURRENT claims)', cur, 'tickets')
    tasks += collection('Repair caps (CURRENT)', cur, 'caps')
    agents = ['Bindings describe native views; implementation roles retain their source labels.',
              'Product controller worker slots: unavailable unless explicitly declared by CURRENT.']
    if not bindings(snapshot):
        agents.append('Native agent views: unavailable' if not manifest else 'Native agent views: none')
    for i, binding in enumerate(bindings(snapshot), 1):
        author = (binding['thread'] == cur.get('thread') and binding['cwd'] == cur.get('worktree')
                  and binding['endpoint'] == snapshot['config']['endpoint'])
        state = 'unknown' if binding['current'] is None else 'current' if binding['current'] else 'historical'
        label = ' | implementation author (CURRENT binding)' if author else ''
        agents += [f'[{i}] {safe(binding["role"])} | {state} | window {binding["window"]}{label}',
                   f'  thread {safe(binding["thread"])} | endpoint {safe(binding["endpoint"])}',
                   '  cwd ' + safe(binding['cwd'])]
    agents += [field('Writer (CURRENT)', cur, 'writer'), field('Writer state (CURRENT)', cur, 'writer_state')]
    agents += collection('Product bindings (CURRENT)', cur, 'product_bindings')
    agents += ['', 'Operator roster (lead-owned) | separate from product assignments', provenance(snapshot, 'Roster')]
    for agent in snapshot['sources']['Roster']['data'].get('agents', []):
        view = next((b for b in bindings(snapshot) if agent.get('thread') == b['thread']
                     and agent.get('endpoint') == b['endpoint']), None)
        agents += [f'  {safe(agent["name"])} | scope {safe(agent["scope"])} | '
                   f'status {safe(agent["status"])} | model {safe(agent["model"])}',
                   '    native view: ' + (view['window'] if view else 'unavailable')]
        for key in ('ticket', 'thread', 'endpoint', 'worktree'):
            if key in agent:
                agents.append('    ' + field(key, agent, key))
    work = collection('Progress (CURRENT)', cur, 'progress')
    work += collection('Progress (Luna observation)', luna, 'progress')
    work += collection('Questions (CURRENT)', cur, 'questions')
    work += [field('Acknowledgment (CURRENT)', cur, 'ack')]
    evidence = collection('Focused results (CURRENT reports)', cur, 'focused_results')
    evidence += collection('Remaining gates (CURRENT)', cur, 'pending_gates')
    evidence += collection('Findings (Luna)', luna, 'actionable_findings')
    evidence += collection('Deferred (CURRENT)', cur, 'deferred')
    evidence += collection('Observation limitations (Luna)', luna, 'limitations')
    vcs = [field(label + ' (CURRENT)', cur, key) for label, key in
           [('Workspace', 'worktree'), ('Base', 'base'), ('Source', 'source'),
            ('Candidate', 'candidate'), ('Landing', 'landing')]]
    vcs += collection('Source identities (Luna observation)', luna, 'source_identities')
    infra = collection('Declared services (CURRENT)', cur, 'declared_services')
    infra += collection('Observed services (Luna)', luna, 'observed_services')
    infra += collection('Context (CURRENT; counters retain source names)', cur, 'context')
    context = cur.get('context', {})
    if 'author_latest_observed_input' in context:
        infra.append(field('Author latest observed input (not cumulative usage)', context, 'author_latest_observed_input'))
    infra += [field('Manifest health (manager report)', manifest, 'health')]
    infra += collection('Manifest errors (manager report)', manifest, 'errors')
    infra += collection('Next checks (Luna)', luna, 'next_checks')
    infra += [provenance(snapshot, kind) for kind in ('CURRENT', 'Luna', 'Manifest')]
    return dict(zip(SECTION_NAMES, (overview, tasks, agents, work, evidence, vcs, infra)))


def render(snapshot):
    lines = ['CORENIUS MAC | read-only operator projection',
             'Observed now ' + snapshot['now'].astimezone(HUMAN_ZONE).isoformat() + ' America/Los_Angeles',
             'CURRENT claims / Luna observations / manager native-view metadata']
    for name, body in sections(snapshot).items():
        lines += ['', '=== ' + name + ' ==='] + body
    return lines


def jump_view(snapshot, index):
    raise RuntimeError('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.')


def dashboard(screen, config):
    try:
        curses.curs_set(0)
    except curses.error:
        pass
    screen.timeout(200)
    tab, top, selected = 0, 0, 0
    next_read, help_on, message = 0, False, ''
    snapshot = None
    while True:
        if time.monotonic() >= next_read:
            snapshot = load_snapshot(config)
            next_read = time.monotonic() + config['refresh_seconds']
        views = bindings(snapshot)
        selected = max(0, min(selected, len(views) - 1))
        name = SECTION_NAMES[tab]
        body = sections(snapshot)[name]
        if name == 'Agents':
            body = [('> ' if line.startswith(f'[{selected + 1}] ') else '  ') + line for line in body]
        if help_on:
            body = ['1..7: Overview, Tasks, Agents, Work, Evidence, VCS, Infra.',
                    'Tab / left / right: change section. Up/down or j/k: scroll.',
                    'PageUp/PageDown: page. Home/End: start/end. r: refresh now.',
                    'Agents: n/p select native view; Enter selects its tmux window.',
                    'View selection sends no model input and changes no task state.',
                    '? returns. q exits only this dashboard.',
                    'Tmux Ctrl-b d detaches while clients and work keep running.',
                    'Sources are read-only; stale is over five minutes. Missing differs from none.']
        screen.erase()
        height, width = screen.getmaxyx()
        width = max(1, width - 1)
        wrapped = [part for line in body for part in (textwrap.wrap(safe(line), width=width,
                    replace_whitespace=False) or [''])]
        header_rows = min(3, max(1, height - 2))
        body_height = max(0, height - header_rows - 1)
        top = max(0, min(top, max(0, len(wrapped) - body_height)))

        def draw(y, text, attr=0):
            if y < height:
                try:
                    screen.addnstr(y, 0, safe(text), width, attr)
                except curses.error:
                    pass

        draw(0, 'CORENIUS MAC | ' + snapshot['now'].astimezone(HUMAN_ZONE).strftime('%H:%M:%S LA'), curses.A_BOLD)
        if header_rows > 1:
            draw(1, f'[{tab + 1}/7] {name} | 1-7 tabs', curses.A_BOLD)
        if header_rows > 2:
            source = snapshot['sources']['CURRENT']
            freshness = ('unavailable' if source['error'] else
                         ('STALE' if (snapshot['now'] - source['time']).total_seconds() > STALE_SECONDS else 'loaded'))
            draw(2, f'CURRENT {freshness} | {top + 1}/{len(wrapped)}')
        for y, line in enumerate(wrapped[top:top + body_height], header_rows):
            draw(y, line)
        draw(height - 1, message or 'j/k scroll r refresh ? help q quit')
        screen.refresh()
        key = screen.getch()
        if key == ord('q'):
            return
        if key == ord('r'):
            next_read = 0
        elif key == ord('?'):
            help_on, top = not help_on, 0
        elif ord('1') <= key <= ord('7'):
            tab, top, help_on, message = key - ord('1'), 0, False, ''
        elif key in (9, curses.KEY_RIGHT, curses.KEY_LEFT, curses.KEY_BTAB):
            delta = -1 if key in (curses.KEY_LEFT, curses.KEY_BTAB) else 1
            tab, top, help_on, message = (tab + delta) % len(SECTION_NAMES), 0, False, ''
        elif key in (curses.KEY_DOWN, ord('j')):
            top += 1
        elif key in (curses.KEY_UP, ord('k')):
            top -= 1
        elif key in (curses.KEY_NPAGE, curses.KEY_PPAGE):
            top += max(1, body_height) * (1 if key == curses.KEY_NPAGE else -1)
        elif key in (curses.KEY_HOME, curses.KEY_END):
            top = 0 if key == curses.KEY_HOME else len(wrapped)
        elif name == 'Agents' and not help_on:
            if key in (ord('n'), ord('p')) and views:
                selected = (selected + (1 if key == ord('n') else -1)) % len(views)
                top = 0
                message = f'View {selected + 1}/{len(views)} {safe(views[selected]["role"])}; Enter jumps'
            elif key in (10, 13, curses.KEY_ENTER):
                message = jump_view(snapshot, selected)


def main(argv=None):
    print('Retired imported controller entrypoint. Use harp durable --help for prepare/admit/start/status/cancel/resolve/approve/land. Intent belongs in .scratch/<feature>/spec.md and issues/*.md; legacy state is not migrated.', file=sys.stderr)
    return 2


if __name__ == '__main__':
    raise SystemExit(main())
