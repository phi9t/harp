#!/usr/bin/env python3
"""Read-only controller, Kata and workspace panes from explicit public JSON.

No controller connections, native stores, databases, transcripts or tool calls.
Use --render for plain text; 1/2/3 switches panes, j/k scrolls, r refreshes.
"""
import argparse
import curses
import datetime as dt
import pathlib
import re
import textwrap
import time

from mac_dashboard import load_config as dashboard_config, read_json, safe, timestamp

VIEWS = ('controller', 'kata', 'workspaces')
MAX_ITEMS = 40
MAX_LINES = 300
MAX_TEXT = 300
SECRET_KEY = re.compile(r'(secret|credential|password|token|auth|private|reasoning|transcript)', re.I)
# Each nested object has its own allowlist. In particular runtime.document,
# report bodies, operation payloads and visible stream records are never printed.
SCALAR = None
IDENTITY = dict.fromkeys(('id', 'uid', 'name', 'deployment', 'project', 'endpoint',
                          'socket', 'path', 'revision', 'base', 'lead_epoch', 'paused'))
ATTEMPT = dict.fromkeys(('id', 'task', 'slot', 'brief', 'base', 'state', 'status',
                         'scope_epoch', 'cancel_requested', 'workspace', 'worktree',
                         'source', 'epoch'))
RUNTIME = dict.fromkeys(('id', 'role', 'attempt', 'authority_epoch', 'state',
                         'thread', 'endpoint', 'runtime_incarnation'))
NATIVE_SOURCE = dict.fromkeys(('source', 'role', 'current', 'observed_at', 'digest'))
NATIVE_BINDING = dict.fromkeys(('role', 'thread', 'endpoint', 'cwd', 'window', 'current'))
NATIVE_BINDING.update(sources=[NATIVE_SOURCE])
WORKSPACE = dict.fromkeys(('id', 'attempt', 'task', 'workspace', 'worktree', 'path', 'cwd',
                           'source', 'base', 'revision', 'generation', 'workspace_generation',
                           'owner', 'state', 'status', 'head', 'commit_id', 'tree_id',
                           'candidate', 'current_candidate', 'landing', 'disposition',
                           'evidence', 'digest', 'ownership_drift', 'commit', 'kind',
                           'observed_attempt_state', 'cleanup_disposition', 'retention',
                           'sha256', 'bytes', 'coverage', 'verdict', 'report'))
FILESYSTEM = dict.fromkeys(('configured', 'backend', 'status', 'actual_trunk', 'project_base',
                            'ref_drift', 'snapshot_revision', 'current_revision', 'state_drift',
                            'observation_elapsed_ms', 'protected_ref'))
FILESYSTEM.update(workspaces=[WORKSPACE], history=[WORKSPACE])
TASK = dict.fromkeys(('id', 'uid', 'title', 'task', 'state', 'status', 'owner', 'revision',
                      'source_revision', 'brief', 'work_ref', 'digest', 'attempt'))
TASK.update(dependencies=[SCALAR], blocked_by=[SCALAR])
QUESTION = dict.fromkeys(('id', 'task', 'attempt', 'sender', 'text', 'question', 'answer',
                          'answered_by', 'status', 'state', 'delivery_state', 'lead_epoch'))
OPERATION = dict.fromkeys(('id', 'task', 'attempt', 'binding', 'kind', 'state', 'status',
                           'event_seq', 'controller_revision', 'acknowledged_revision'))
SOURCE = dict.fromkeys(('task', 'revision', 'source_revision', 'acknowledged_kata_revision', 'drift'))
SOURCE.update(admitted=TASK, current=TASK)
SYNC = dict.fromkeys(('status', 'state', 'revision', 'configured', 'authority', 'uncertainty',
                      'drift', 'acknowledged_revision', 'source_revision'))
SYNC.update(effects=[OPERATION], history=[OPERATION])
WORKER = dict.fromkeys(('slot', 'task', 'status', 'state', 'epoch', 'authority_epoch', 'staleness'))
WORKER.update(attempt=ATTEMPT, runtime=RUNTIME,
              observed=dict.fromkeys(('state', 'status', 'stop_observed')),
              reported_progress=dict.fromkeys(('status', 'state', 'summary', 'event_cursor')),
              material_evidence=WORKSPACE,
              cancellation=dict.fromkeys(('requested', 'observed_stopped')))
MISSION = dict.fromkeys(('project', 'objective', 'goal', 'status', 'state', 'lead_runtime',
                         'lead_epoch', 'role_epoch', 'authority_epoch', 'runtime_incarnation'))
MISSION.update(constraints=SCALAR)
MONITORING = dict.fromkeys(('continuous_runner', 'authority', 'status', 'state'))
MONITOR_RECORD = dict.fromkeys(('id', 'attempt', 'epoch', 'state', 'status', 'stream', 'batch',
                               'gap', 'age_seconds', 'backlog', 'history_gap', 'last_ingested_at',
                               'input_digest', 'output_digest', 'through', 'analyst', 'version'))
MONITORING.update({key: [MONITOR_RECORD] for key in
                   ('streams', 'batches', 'assessments', 'findings', 'corrections', 'hypotheses')})
NEXT = dict.fromkeys(('questions', 'status', 'state'))
NEXT.update(candidate_acceptance=[WORKSPACE], unknown_effects=[OPERATION])
HISTORY = dict.fromkeys(('observed_at', 'state', 'cursor', 'event_cursor', 'revision',
                         'occupied_slots', 'physical_ref', 'digest'))
WORKREF = dict.fromkeys(('origin_instance_uid', 'project_uid', 'item_uid'))
KATA_IDENTITY = dict.fromkeys(('instance_uid', 'project_id', 'project_uid', 'controller_id'))
KATA_ISSUE = dict.fromkeys(('id', 'uid', 'project_id', 'project_uid', 'revision', 'status', 'updated_at'))
KATA_EXECUTION = dict.fromkeys(('state', 'task_state', 'observed_attempt_state', 'attempt',
                                'brief', 'controller_revision', 'event_seq', 'reported_progress',
                                'reported_progress_bytes', 'reported_progress_truncated',
                                'cancel_requested', 'unanswered_questions', 'admitted_version',
                                'controller', 'project', 'operation', 'runtime_cleanup_qualified'))


def text(value):
    """Render a scalar only, never serialize an arbitrary object."""
    if value is None:
        return 'none'
    if not isinstance(value, (str, int, float, bool)):
        return 'unavailable (wrong type)'
    return safe(value)[:MAX_TEXT]


def load_config(path):
    config = dashboard_config(path)
    for key in ('controller_snapshot_path', 'kata_snapshot_path', 'kata_source_path'):
        if key in config and (not isinstance(config[key], str) or not config[key]):
            raise ValueError(key + ' must be a nonempty path')
    return config


def validate(data, kind, config):
    if 'observed_at' in data:
        timestamp(data['observed_at'])
    if kind in ('Product controller', 'Kata', 'Kata source'):
        for key in ('available', 'current'):
            if key in data and not isinstance(data[key], bool):
                raise ValueError('wrong availability flag shape')
    lists = {'Product controller': ('workers', 'tasks', 'attempts', 'operations', 'questions',
                                   'sources', 'candidates', 'verification', 'landing_queue',
                                   'unknown_effects', 'runtime_uncertainty', 'repair_history',
                                   'next_decisions', 'runtime_bindings'),
             'Kata': ('issues', 'items'), 'Manifest': ('bindings',)}.get(kind, ())
    for key in lists:
        if key in data and (not isinstance(data[key], list) or
                            any(not isinstance(item, dict) for item in data[key])):
            raise ValueError('wrong collection shape')
    if kind == 'Manifest' and (data.get('session') != config['session'] or 'bindings' not in data):
        raise ValueError('manifest identity unavailable')
    for key in ('mission', 'lead', 'monitoring', 'synchronization'):
        if key in data and data[key] is not None and not isinstance(data[key], dict):
            raise ValueError('wrong object shape')
    if kind == 'CURRENT':
        for key in ('tickets', 'focused_results'):
            if key in data and not isinstance(data[key], dict):
                raise ValueError('wrong claim shape')
    if kind == 'Product controller' and controller_key(data):
        center = data[controller_key(data)]
        if center is not None:
            if not isinstance(center, dict) or controller_key(center):
                raise ValueError('wrong controller export shape')
            validate(center, kind, config)
        history = data.get('last_successful_observation')
        if history is not None and not isinstance(history, dict):
            raise ValueError('wrong controller history shape')
    if kind in ('Kata', 'Kata source') and data.get('source_identity') is not None:
        if not isinstance(data['source_identity'], dict):
            raise ValueError('wrong Kata source identity shape')
    if kind == 'Kata':
        for item in data.get('items', []):
            for key in ('workref', 'issue', 'corenius.execution'):
                if item.get(key) is not None and not isinstance(item[key], dict):
                    raise ValueError('wrong Kata item shape')


def load_snapshot(config, now=None):
    sources = {}
    for kind, path in [('CURRENT', config['current_path']),
                       ('Manifest', str(pathlib.Path(config['state_dir']) / 'manifest.json')),
                       ('Product controller', config.get('controller_snapshot_path')),
                       ('Kata', config.get('kata_snapshot_path')),
                       ('Kata source', config.get('kata_source_path'))]:
        source = {'path': path, 'data': {}, 'digest': '', 'error': '', 'time': None}
        if path is None:
            source['error'] = 'not configured'
        else:
            try:
                data, source['digest'] = read_json(path)
                validate(data, kind, config)
                source['time'] = timestamp(data['observed_at']) if 'observed_at' in data else None
                source['data'] = data
            except FileNotFoundError:
                source['error'] = 'missing'
            except OSError:
                source['error'] = 'unreadable'
            except (ValueError, TypeError, RecursionError, OverflowError):
                source['error'] = 'malformed or exceeds 1 MiB input bound'
        sources[kind] = source
    return {'config': config, 'sources': sources,
            'now': now or dt.datetime.now(dt.timezone.utc)}


def provenance(snapshot, kind):
    source = snapshot['sources'][kind]
    status = 'unavailable | ' + source['error'] if source['error'] else 'freshness unavailable'
    if not source['error'] and source['time'] is not None:
        age = (snapshot['now'] - source['time']).total_seconds()
        status = 'STALE (>5m)' if age > 300 else 'future timestamp; freshness unavailable' if age < -60 else 'fresh'
        status += ' | age ' + str(int(age)) + 's'
    data = source['data']
    if not source['error'] and (data.get('status') == 'unavailable' or data.get('available') is False or
            (kind == 'Product controller' and controller_key(data) and data[controller_key(data)] is None)):
        status = 'unavailable (export observation) | ' + status
    elif not source['error'] and data.get('current') is False:
        status = 'observed snapshot (current:false; no ongoing liveness) | ' + status
    return (kind + ': ' + status + ' | path ' + text(source['path']) +
            ' | source timestamp ' + text(source['data'].get('observed_at', 'unavailable')) +
            (' | UTC ' + source['time'].isoformat() if source['time'] is not None else '') +
            ' | sha256:' + (source['digest'] or 'unavailable'))


def record(label, value, schema, depth=0):
    """Project only schema fields, with bounded collection count and nesting."""
    prefix = '  ' * depth
    if schema is None:
        return [prefix + label + ': ' + text(value)]
    if value is None:
        return [prefix + label + ': none']
    if isinstance(schema, list):
        if not isinstance(value, list):
            return [prefix + label + ': unavailable (wrong type)']
        if not value:
            return [prefix + label + ': none']
        lines = [prefix + label + ':']
        for index, item in enumerate(value[:MAX_ITEMS]):
            lines += record(str(index + 1), item, schema[0], depth + 1)
        if len(value) > MAX_ITEMS:
            lines.append(prefix + '  [display limited to 40 entries]')
        return lines
    if not isinstance(value, dict):
        # Older/simple projections have scalar attempt/runtime identities.
        return [prefix + label + ': ' + text(value)]
    lines = [prefix + label + ':']
    for key, subschema in schema.items():
        if key in value:
            lines += record(key, value[key], subschema, depth + 1)
    if len(lines) == 1:
        lines.append(prefix + '  allowlisted fields: unavailable')
    return lines


def field(label, data, key, schema=SCALAR):
    if key not in data:
        return [label + ': unavailable']
    return record(label, data[key], schema)


def claims(label, data, key):
    value = data.get(key)
    if not isinstance(value, dict):
        return [label + ': unavailable']
    lines = [label + ':' if value else label + ': none']
    for name, item in list(value.items())[:MAX_ITEMS]:
        if SECRET_KEY.search(name):
            continue
        lines += record(text(name), item, dict.fromkeys(('state', 'status', 'result', 'source',
                                                       'evidence', 'digest')), 1)
    if len(value) > MAX_ITEMS:
        lines.append('  [display limited to 40 entries]')
    return lines


def controller_key(export):
    if 'control_center' in export:
        return 'control_center'
    if 'data' in export and 'available' in export:
        return 'data'
    return None


def controller_data(snapshot):
    """Select current public state only; historical success never fills missing state."""
    export = snapshot['sources']['Product controller']['data']
    key = controller_key(export)
    if key is None:
        return export
    if export.get('status') == 'unavailable' or export.get('available') is False:
        return {}
    return export[key] or {}


def availability(data):
    if 'available' not in data:
        return []
    return field('Available (export claim)', data, 'available') + field('Current (export claim)', data, 'current')


def controller_export(snapshot):
    export = snapshot['sources']['Product controller']['data']
    key = controller_key(export)
    if key is None:
        return []
    if key == 'data':
        return availability(export) + field('source_seam', export, 'source_seam') + field('source_commit', export, 'source_commit')
    lines = field('Export status', export, 'status') + field('Export source', export, 'source')
    lines += field('deployment', export, 'deployment') + field('project', export, 'project')
    lines += field('Acceptance (export claim)', export, 'acceptance')
    lines += field('Last successful controller observation (history; no current liveness)',
                   export, 'last_successful_observation', HISTORY)
    history = export.get('last_successful_observation')
    if isinstance(history, dict):
        lines += claims('Historical task state claims', history, 'tasks')
    return lines


def controller(snapshot):
    data = controller_data(snapshot)
    cur = snapshot['sources']['CURRENT']['data']
    lines = [provenance(snapshot, 'Product controller')] + controller_export(snapshot)
    for key, schema in [('deployment', IDENTITY), ('project', IDENTITY), ('revision', SCALAR),
                        ('event_cursor', SCALAR), ('occupied_slots', SCALAR), ('lead_epoch', SCALAR),
                        ('mission', MISSION), ('lead', RUNTIME)]:
        lines += field(key, data, key, schema)
    # The official projection orders workers by occupied slot, but does not
    # necessarily export physical slot IDs. Never infer an ID from array order.
    workers = data.get('workers', [])

    def physical_slot(worker):
        attempt = worker.get('attempt')
        return worker.get('slot', attempt.get('slot') if isinstance(attempt, dict) else None)

    mapped = set()
    for slot in (1, 2):
        worker = next((w for w in workers if type(physical_slot(w)) is int and physical_slot(w) == slot), None)
        if worker is not None:
            mapped.add(id(worker))
        lines += record('Worker slot ' + str(slot), worker, WORKER) if worker else ['Worker slot ' + str(slot) + ': unknown']
    unassigned = [w for w in workers if physical_slot(w) is None]
    if unassigned:
        lines += record('Occupied workers (projection order; physical slot IDs unavailable)', unassigned, [WORKER])
    unmapped = [w for w in workers if physical_slot(w) is not None and id(w) not in mapped]
    if unmapped:
        lines += record('Unmapped workers (unknown or duplicate physical slot IDs)', unmapped, [WORKER])
    lines += ['', provenance(snapshot, 'CURRENT')]
    lines += field('Implementation writer (CURRENT)', cur, 'writer')
    lines += field('Implementation writer state (CURRENT)', cur, 'writer_state')
    lines += claims('Tasks (CURRENT claims)', cur, 'tickets')
    lines += native(snapshot)
    for key, schema in [('tasks', [TASK]), ('attempts', [ATTEMPT]), ('operations', [OPERATION]),
                        ('questions', [QUESTION]),
                        ('unknown_effects', [OPERATION]), ('runtime_uncertainty', [OPERATION]),
                        ('runtime_bindings', [RUNTIME]), ('sources', [SOURCE]),
                        ('synchronization', SYNC), ('candidates', [WORKSPACE]),
                        ('verification', [WORKSPACE]), ('landing_queue', [WORKSPACE]),
                        ('monitoring', MONITORING), ('repair_history', [WORKSPACE]), ('next_decisions', [NEXT])]:
        lines += field(key, data, key, schema)
    return lines


def native(snapshot):
    data = snapshot['sources']['Manifest']['data']
    return ['', provenance(snapshot, 'Manifest')] + field('Native bindings (manager)', data, 'bindings', [NATIVE_BINDING])


def kata(snapshot):
    data = snapshot['sources']['Kata']['data']
    lines = [provenance(snapshot, 'Kata')]
    if snapshot['config'].get('kata_source_path'):
        source = snapshot['sources']['Kata source']['data']
        lines += ['', 'Kata source provenance', provenance(snapshot, 'Kata source')]
        lines += field('Source identity', source, 'source_identity', KATA_IDENTITY)
    if 'items' in data:
        lines += availability(data)
        if 'available' in data:
            lines += field('source_seam', data, 'source_seam')
            lines += field('Source identity', data, 'source_identity', KATA_IDENTITY)
        else:
            for label, key in [('Export status', 'status'), ('Export source', 'source'),
                           ('deployment', 'deployment'), ('origin_instance_uid', 'origin_instance_uid'),
                           ('project_uid', 'project_uid'), ('Controller authority (export claim)', 'controller_authority')]:
                lines += field(label, data, key)
        lines += ['Execution summaries may lag the controller; no runtime liveness claim.',
                  'Reported progress is a public source claim; display text limited to 300 chars.']
        if data.get('available') is False:
            return lines + ['Kata items: unavailable']
        items = data['items']
        if not items:
            lines.append('Kata items: none')
        for index, item in enumerate(items[:MAX_ITEMS], 1):
            lines += ['', 'Kata item ' + str(index)]
            lines += field('Task', item, 'task')
            if 'issue' in item:
                lines += field('Native Kata issue', item, 'issue', KATA_ISSUE)
                issue = item['issue'] or {}
                lines += field('Source revision', issue, 'revision') + field('Native Kata status', issue, 'status')
            else:
                lines += field('WorkRef', item, 'workref', WORKREF)
                lines += field('Source revision', item, 'source_revision')
                lines += field('Native Kata status', item, 'status')
            lines += field('Synchronized execution (corenius.execution; may lag; no runtime liveness)',
                           item, 'corenius.execution', KATA_EXECUTION)
        if len(items) > MAX_ITEMS:
            lines.append('[display limited to 40 entries]')
        return lines
    for key, schema in [('deployment', IDENTITY), ('project', IDENTITY), ('issues', [TASK]),
                        ('synchronization', SYNC)]:
        lines += field(key, data, key, schema)
    return lines


def workspaces(snapshot):
    cur = snapshot['sources']['CURRENT']['data']
    data = controller_data(snapshot)
    lines = [provenance(snapshot, 'CURRENT'), 'CURRENT claims:']
    for key in ('worktree', 'base', 'source', 'candidate', 'landing'):
        lines += field(key, cur, key, WORKSPACE if key in ('candidate', 'landing') else SCALAR)
    lines += claims('Evidence (CURRENT claims)', cur, 'focused_results')
    lines += ['', provenance(snapshot, 'Product controller')]
    lines += controller_export(snapshot)
    lines += field('Attempts (controller published claims)', data, 'attempts', [ATTEMPT])
    lines += field('Worker attempts (controller published claims)', data, 'workers',
                   [{'slot': SCALAR, 'attempt': ATTEMPT}])
    filesystem_schema = [WORKSPACE] if isinstance(data.get('filesystem'), list) else FILESYSTEM
    for label, key, schema in [('Controller filesystem (published inventory)', 'filesystem', filesystem_schema),
                               ('Controller candidates', 'candidates', [WORKSPACE]),
                               ('Controller landing queue', 'landing_queue', [WORKSPACE])]:
        lines += field(label, data, key, schema)
    manifest = snapshot['sources']['Manifest']['data']
    lines += ['', provenance(snapshot, 'Manifest')]
    lines += field('Native view cwd (manager)', manifest, 'bindings', [NATIVE_BINDING])
    return lines


def render(snapshot, view):
    lines = ['CORENIUS MAC | ' + view.title() + ' | read-only public projection',
             'Observed now ' + snapshot['now'].isoformat()]
    lines += {'controller': controller, 'kata': kata, 'workspaces': workspaces}[view](snapshot)
    if len(lines) > MAX_LINES:
        lines = lines[:MAX_LINES - 1] + ['[display limited to 300 lines; source may contain more records]']
    return [safe(line)[:1200] for line in lines]


def dashboard(screen, config, initial):
    try:
        curses.curs_set(0)
    except curses.error:
        pass
    screen.timeout(150)
    selected, top, next_read, help_on = VIEWS.index(initial), 0, 0, False
    while True:
        if time.monotonic() >= next_read:
            snapshot = load_snapshot(config)
            next_read = time.monotonic() + config['refresh_seconds']
        height, width = screen.getmaxyx()
        width = max(1, width - 1)
        body = render(snapshot, VIEWS[selected])
        if help_on:
            body = ['1 Controller / 2 Kata / 3 Workspaces', 'Tab or arrows changes pane.',
                    'j/k scroll; PgUp/PgDn page; Home/End.', 'r refresh; ? help; q quit.',
                    'Public files only. Unknown slots remain unknown.', 'Stale >5m; no source timestamp means unknown freshness.']
        wrapped = [part for line in body for part in (textwrap.wrap(line, width=width) or [''])]
        rows = max(0, height - 2)
        top = max(0, min(top, max(0, len(wrapped) - rows)))
        screen.erase()

        def draw(y, line):
            if y < height:
                try:
                    screen.addnstr(y, 0, safe(line), width)
                except curses.error:
                    pass

        draw(0, VIEWS[selected].title() + ' | ' + str(top + 1) + '/' + str(len(wrapped)))
        for y, line in enumerate(wrapped[top:top + rows], 1):
            draw(y, line)
        draw(height - 1, '1/2/3 j/k r ? q')
        screen.refresh()
        key = screen.getch()
        if key == ord('q'):
            return
        if key == ord('r'):
            next_read = 0
        elif key == ord('?'):
            help_on, top = not help_on, 0
        elif ord('1') <= key <= ord('3'):
            selected, top, help_on = key - ord('1'), 0, False
        elif key in (9, curses.KEY_RIGHT, curses.KEY_LEFT, curses.KEY_BTAB):
            selected = (selected + (-1 if key in (curses.KEY_LEFT, curses.KEY_BTAB) else 1)) % 3
            top, help_on = 0, False
        elif key in (ord('j'), curses.KEY_DOWN):
            top += 1
        elif key in (ord('k'), curses.KEY_UP):
            top -= 1
        elif key in (curses.KEY_NPAGE, curses.KEY_PPAGE):
            top += max(1, rows) * (1 if key == curses.KEY_NPAGE else -1)
        elif key in (curses.KEY_HOME, curses.KEY_END):
            top = 0 if key == curses.KEY_HOME else len(wrapped)


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--config', required=True, type=pathlib.Path)
    parser.add_argument('--view', choices=VIEWS, default='controller')
    parser.add_argument('--render', action='store_true')
    args = parser.parse_args(argv)
    try:
        config = load_config(args.config)
    except (OSError, ValueError, TypeError, RecursionError) as error:
        parser.exit(2, 'Config unavailable: ' + safe(error) + '\n')
    if args.render:
        print('\n'.join(render(load_snapshot(config), args.view)))
    else:
        curses.wrapper(dashboard, config, args.view)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
