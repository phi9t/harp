#!/usr/bin/env python3
"""Bounded evaluation seam: real pinned Kata + accepted broker HTTP + real PG.

This is a disposable fixture, not a production planning adapter. Source events
never authorize assignment: an immutable fixture-owner decision is required.
"""
import argparse
import concurrent.futures
import contextlib
import hashlib
import json
import os
from pathlib import Path
import secrets
import signal
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
import uuid


IMAGE = 'postgres@sha256:f3bd19c606e442c3d7bdfa8002e03fe260a1023351e0ea4598032022b68dd6e3'
PIN = 'b70a4306464e319701df30a3b92860289613b15a'


def canonical(value):
    return json.dumps(value, sort_keys=True, separators=(',', ':'), ensure_ascii=False).encode()


def digest(value):
    return hashlib.sha256(canonical(value)).hexdigest()


def http(origin, token, method, path, value=None, headers=None):
    request_headers = {'Content-Type': 'application/json'}
    if token:
        request_headers['Authorization'] = 'Bearer ' + token
    request_headers.update(headers or {})
    request = urllib.request.Request(origin + path,
        data=None if value is None else canonical(value), headers=request_headers, method=method)
    try:
        with urllib.request.urlopen(request, timeout=5) as reply:
            return reply.status, json.load(reply)
    except urllib.error.HTTPError as error:
        return error.code, json.load(error)


def ephemeral_port():
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        return sock.getsockname()[1]


def checked(arguments, **kwargs):
    return subprocess.check_output(arguments, timeout=30, **kwargs)


def stop_process(proc):
    if proc.poll() is None:
        os.killpg(proc.pid, signal.SIGTERM)
    try:
        proc.wait(timeout=5)
    except subprocess.TimeoutExpired:
        os.killpg(proc.pid, signal.SIGKILL)
        proc.wait(timeout=5)
    return proc.poll() is not None


def wait_ready(proc, callback, ready_status=200):
    for _ in range(100):
        if proc.poll() is not None:
            raise RuntimeError('owned fixture service exited during startup')
        try:
            if callback()[0] == ready_status:
                return
        except (urllib.error.URLError, TimeoutError):
            pass
        time.sleep(0.1)
    raise RuntimeError('owned fixture service readiness timed out')


class EvaluationBridge:
    """Stable source binding, immutable approval and explicit owner decision.

    No Kata owner/title/close/revision field is execution authority. Broker HTTP
    commands and its durable journals remain the only admitted execution store.
    """
    def __init__(self, broker_origin, owner, epoch, binding_path):
        self.origin = broker_origin
        self.owner = owner
        self.epoch = epoch
        self.binding_path = binding_path
        self.binding = json.loads(binding_path.read_text())

    def command(self, key, revision, payload, token=None, generation=None):
        envelope = {'schema_version': 1, 'key': key, 'expected_revision': revision,
                    'expected_generation': generation, 'recovery_epoch': self.epoch,
                    'command': payload}
        return http(self.origin, self.owner if token is None else token,
                    'POST', '/v1/commands', envelope)

    def snapshot(self):
        status, reply = http(self.origin, self.owner, 'GET',
                            '/v1/snapshots/task/' + self.binding['broker_task_id'])
        assert status == 200, (status, reply)
        return reply

    def receive(self, event, assignment_decision=None):
        assert event['origin_instance_uid'] == self.binding['ledger_origin']
        assert event['project_uid'] == self.binding['project_uid']
        assert event['issue_uid'] == self.binding['issue_uid']
        if event['type'] != 'issue.assigned' or assignment_decision is None:
            # A concrete proposal/completion/projection classification crosses
            # the seam, but only the owner command below can mutate the broker.
            disposition = ('completion_claim' if event['type'] == 'issue.closed' else
                           'projection_observation' if event['type'].startswith('issue.metadata') else
                           'source_proposal')
            return {'disposition': disposition, 'event_uid': event['event_uid'],
                    'broker_snapshot': self.snapshot()}
        assert assignment_decision['source_event_uid'] == event['event_uid']
        assert assignment_decision['approved_digest'] == self.binding['approved_digest']
        assert assignment_decision['broker_task_id'] == self.binding['broker_task_id']
        key = 'kata.assign:' + digest((self.binding['ledger_origin'], event['event_uid'],
                                      self.binding['approved_digest'], assignment_decision['decision_id']))
        return self.command(key, assignment_decision['expected_revision'],
            {'kind': 'assign_attempt', 'task_id': self.binding['broker_task_id'],
             'worker_id': assignment_decision['worker_id']}, generation=assignment_decision['expected_generation'])


def run(args):
    start = time.monotonic()
    kata = str(Path(args.kata_binary).resolve())
    broker = str(Path(args.broker_binary).resolve())
    expected = json.loads(Path(args.accepted_manifest).read_text())
    source = Path(args.broker_source)
    for item in expected['files']:
        prefix = 'crates/team-control-plane/'
        if item['path'].startswith(prefix):
            assert hashlib.sha256((source / item['path'][len(prefix):]).read_bytes()).hexdigest() == item['sha256'], item['path']
    version = checked([kata, 'version'], text=True)
    assert 'b70a430' in version, 'fixture requires the exact pinned Kata build'
    records = {'repository_base': expected['base'], 'kata_pin': PIN,
               'kata_binary_sha256': hashlib.sha256(Path(kata).read_bytes()).hexdigest(),
               'broker_binary_sha256': hashlib.sha256(Path(broker).read_bytes()).hexdigest(),
               'broker_content_manifest_sha256': hashlib.sha256(Path(args.accepted_manifest).read_bytes()).hexdigest(),
               'observations': []}
    fixture = 'corenius-kata17-bridge-' + str(uuid.uuid4())
    container_started = False
    processes = []
    ports = []
    logs = []
    temporary_context = tempfile.TemporaryDirectory(prefix='corenius-kata17-bridge-')
    try:
        checked(['docker', 'run', '--rm', '-d', '--name', fixture,
                 '--label', 'corenius.kata17.bridge=true', '-p', '127.0.0.1::5432',
                 '--tmpfs', '/var/lib/postgresql/data:rw', '-e', 'POSTGRES_USER=probe',
                 '-e', 'POSTGRES_DB=probe', '-e', 'POSTGRES_HOST_AUTH_METHOD=trust', IMAGE])
        container_started = True
        for _ in range(60):
            if subprocess.run(['docker', 'exec', fixture, 'pg_isready', '-U', 'probe'],
                              stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=5).returncode == 0:
                break
            time.sleep(0.2)
        else:
            raise RuntimeError('owned PostgreSQL fixture failed readiness')
        port_reply = checked(['docker', 'port', fixture, '5432/tcp'], text=True).strip()
        assert port_reply.startswith('127.0.0.1:'), port_reply
        pg_port = int(port_reply.split(':')[1])
        owned = json.loads(checked(['docker', 'inspect', fixture], text=True))[0]
        assert owned['Config']['Labels']['corenius.kata17.bridge'] == 'true'
        assert owned['HostConfig']['PortBindings']['5432/tcp'][0]['HostIp'] == '127.0.0.1'
        for db in ['kata_fixture', 'broker_fixture']:
            checked(['docker', 'exec', fixture, 'createdb', '-U', 'probe', db])
        with contextlib.nullcontext(temporary_context.name) as temporary:
            root = Path(temporary)
            home = root / 'kata'; home.mkdir(mode=0o700)
            bootstrap = secrets.token_hex(32)
            config = home / 'config.toml'
            config.write_text('[auth]\ntoken="' + bootstrap + '"\nrequire_token_identity=true\n')
            config.chmod(0o600)
            kata_env = {key: value for key, value in os.environ.items() if not key.startswith('KATA_')}
            kata_env.update(KATA_HOME=str(home),
                KATA_DSN=f'postgres://probe@127.0.0.1:{pg_port}/kata_fixture?sslmode=disable')
            kata_port = ephemeral_port(); ports.append(kata_port)
            kata_origin = 'http://127.0.0.1:' + str(kata_port)
            log = open(root / 'kata.log', 'wb'); logs.append(log)
            proc = subprocess.Popen([kata, 'daemon', 'start', '--foreground', '--listen',
                '127.0.0.1:' + str(kata_port)], env=kata_env, cwd=root,
                stdout=log, stderr=log, start_new_session=True); processes.append(proc)
            def call(method, path, value=None, token=bootstrap, headers=None):
                return http(kata_origin, token, method, '/api/v1' + path, value, headers)
            kata_process = proc
            wait_ready(proc, lambda: call('GET', '/health', token=None))
            status, granted = call('POST', '/tokens', {'actor': 'fixture-ledger-lead'})
            assert status == 200; lead = granted['plaintext']
            status, project_reply = call('POST', '/projects', {'name': 'broker-boundary'}, token=lead)
            assert status == 200; project = project_reply['project']
            project_path = '/projects/' + str(project['id'])
            status, created = call('POST', project_path + '/issues',
                {'title': 'explicitly approved objective', 'body': 'immutable source body'}, token=lead)
            assert status == 200; issue = created['issue']
            issue_path = project_path + '/issues/' + issue['short_id']
            status, events = call('GET', project_path + '/events?after_id=0', token=lead)
            assert status == 200
            created_event = next(event for event in events['events']
                if event.get('issue_uid') == issue['uid'] and event['type'] == 'issue.created')
            approved = {'schema_version': 1, 'ledger_origin': created_event['origin_instance_uid'],
                        'project_uid': project['uid'], 'issue_uid': issue['uid'],
                        'title': issue['title'], 'body': issue.get('body', ''),
                        'acceptance': ['one admitted execution authority'],
                        'deliverables': ['fixture result'], 'write_scope': ['fixture']}
            approved_digest = digest(approved)
            git_root = root / 'planning'; git_root.mkdir()
            approved_path = git_root / 'approved.json'
            with approved_path.open('xb') as output:
                output.write(canonical(approved)); output.flush(); os.fsync(output.fileno())
            approved_path.chmod(0o400)
            def git(*arguments):
                return checked(['git', '-C', str(git_root), *arguments], text=True).strip()
            git('init', '-b', 'fixture'); git('config', 'user.name', 'Kata evaluation fixture')
            git('config', 'user.email', 'fixture@example.invalid')
            git('add', 'approved.json'); git('commit', '-m', 'Immutable fixture approval')
            commit = git('rev-parse', 'HEAD')
            ledger_key = (approved['ledger_origin'], approved['project_uid'])
            binding_key = (*ledger_key, approved['issue_uid'], approved_digest)
            mission = str(uuid.uuid5(uuid.NAMESPACE_URL, canonical(ledger_key).decode()))
            task = str(uuid.uuid5(uuid.NAMESPACE_URL, canonical(binding_key).decode()))
            binding = dict(approved, approved_digest=approved_digest,
                           broker_mission_id=mission, broker_task_id=task)
            binding_path = root / 'binding.json'
            with binding_path.open('xb') as output:
                output.write(canonical(binding)); output.flush(); os.fsync(output.fileno())
            binding_path.chmod(0o400)
            epoch = str(uuid.uuid4())
            broker_env = dict(os.environ, TEAM_CONTROL_PLANE_DSN=
                f'host=127.0.0.1 port={pg_port} user=probe dbname=broker_fixture',
                TEAM_CONTROL_PLANE_RECOVERY_EPOCH=epoch)
            checked([broker, 'migrate'], env=broker_env)
            owner = checked([broker, 'bootstrap'], env=broker_env, text=True).strip()
            broker_port = ephemeral_port(); ports.append(broker_port)
            broker_origin = 'http://127.0.0.1:' + str(broker_port)
            log = open(root / 'broker.log', 'wb'); logs.append(log)
            proc = subprocess.Popen([broker, 'serve', '127.0.0.1:' + str(broker_port)],
                env=broker_env, cwd=root, stdout=log, stderr=log, start_new_session=True)
            processes.append(proc)
            wait_ready(proc, lambda: http(broker_origin, owner, 'GET', '/v1/snapshots/task/' + task), ready_status=404)
            bridge = EvaluationBridge(broker_origin, owner, epoch, binding_path)
            input_ref = {'repository': str(git_root), 'commit': commit,
                         'path': 'approved.json', 'digest': approved_digest}
            status, _ = bridge.command('mission:' + digest(ledger_key), 0,
                {'kind': 'create_mission', 'id': mission, 'objective': 'Kata bounded evaluation',
                 'input': input_ref, 'policy': {'capacity': 2, 'max_attempts': 8,
                 'max_episodes': 8, 'max_repairs': 3, 'episode_deadline_seconds': 30}})
            assert status == 200
            task_command = {'kind': 'create_task', 'id': task, 'mission_id': mission,
                'spec': {'dependencies': [], 'objective': approved['title'], 'input': input_ref,
                         'acceptance': approved['acceptance'], 'deliverables': approved['deliverables'],
                         'write_scope': approved['write_scope']}}
            for _ in range(2):
                status, _ = bridge.command('source:' + digest(binding_key), 0, task_command)
                assert status == 200
            initial = bridge.snapshot(); spec_digest = initial['data']['spec_digest']
            for _ in range(2):
                status, admitted = bridge.command('approval:' + approved_digest, 1,
                    {'kind': 'admit_task', 'task_id': task, 'spec_digest': spec_digest})
                assert status == 200
            worker = str(uuid.uuid4()); worker_token = secrets.token_hex(32)
            status, _ = bridge.command('fixture-worker-grant', 0, {'kind': 'grant_credential',
                'credential_id': str(uuid.uuid4()), 'principal_id': worker, 'role': 'worker',
                'mission_id': mission, 'task_id': task, 'generation': 1,
                'token_hash': hashlib.sha256(worker_token.encode()).hexdigest(), 'expires_in_seconds': 60})
            assert status == 200
            status, claimed = call('POST', issue_path + '/actions/claim', {'if_unowned': True}, token=lead)
            assert status == 200
            status, poll = call('GET', project_path + '/events?after_id=0', token=lead)
            assert status == 200
            claim_event = next(event for event in poll['events']
                if event.get('issue_uid') == issue['uid'] and event['type'] == 'issue.assigned')
            observed = bridge.receive(claim_event)
            assert observed['broker_snapshot']['data']['attempt_count'] == 0, 'ledger claim must not authorize assignment'
            decision = {'decision_id': str(uuid.uuid4()), 'source_event_uid': claim_event['event_uid'],
                        'approved_digest': approved_digest, 'broker_task_id': task,
                        'worker_id': worker, 'expected_revision': 2, 'expected_generation': 0}
            decision_path = root / 'assignment-decision.json'
            with decision_path.open('xb') as output:
                output.write(canonical(decision)); output.flush(); os.fsync(output.fileno())
            decision_path.chmod(0o400)
            # Reload the binding and owner decision, so memory is not deduplication authority.
            bridge = EvaluationBridge(broker_origin, owner, epoch, binding_path)
            decision = json.loads(decision_path.read_text())
            with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
                first, duplicate = list(executor.map(lambda _: bridge.receive(claim_event, decision), range(2)))
            assert first[0] == duplicate[0] == 200 and first[1] == duplicate[1], \
                ('duplicate source event must return the same durable assignment receipt', first, duplicate)
            snapshot = bridge.snapshot()
            assert [snapshot['data'][name] for name in ['attempt_count', 'operation_count', 'reservation_count', 'outbox_count']] == [1, 1, 1, 1]
            assert snapshot['data']['spec_digest'] == spec_digest
            records['binding'] = dict(binding, broker_spec_digest=spec_digest,
                                      broker_admitted_revision=admitted['revision'], git_commit=commit)
            records['assignment_decision'] = decision
            records['source_claim_event'] = claim_event
            records['original_assignment_receipt'] = first[1]
            records['duplicate_assignment_receipt'] = duplicate[1]
            records['observations'].append({'name': 'actual-source-event-duplicate', 'counts': [1, 1, 1, 1]})
            status, _ = call('PATCH', issue_path, {'title': 'edited after approval'}, token=lead)
            assert status == 200
            status, _ = call('POST', issue_path + '/actions/close', {'retry_protocol': 'close-v1',
                'reason': 'done', 'message': 'fixture completion claim for an unexecuted test', 'evidence': [{'type': 'test', 'command': 'false'}]}, token=lead,
                headers={'Idempotency-Key': 'fixture-close'})
            assert status == 200
            status, source_events = call('GET', project_path + '/events?after_id=0', token=lead)
            assert status == 200
            delivered = []
            for event in source_events['events']:
                if event.get('issue_uid') == issue['uid'] and event['event_uid'] != claim_event['event_uid']:
                    for _ in range(2):
                        observation = bridge.receive(event)
                        delivered.append({'event_uid': observation['event_uid'], 'type': event['type'], 'disposition': observation['disposition']})
            after = bridge.snapshot()
            assert (after['revision'], after['data']) == (snapshot['revision'], snapshot['data']), 'source edit/close replay must not mutate broker execution'
            assert after['data']['approved_digest'] == spec_digest and after['data']['status'] == 'running'
            status, current = call('GET', issue_path, token=lead)
            assert status == 200 and current['issue']['closed_reason'] == 'done'
            assert current['issue']['title'] == 'edited after approval'
            assert hashlib.sha256(approved_path.read_bytes()).hexdigest() == approved_digest
            unauthorized, rejection = bridge.command('ledger-token-cannot-admit', 3,
                {'kind': 'assign_attempt', 'task_id': task, 'worker_id': worker}, token=lead)
            assert unauthorized == 401 and rejection['code'] == 'unauthorized'
            records['post_source_replay_snapshot'] = after
            records['source_events_replayed'] = delivered
            records['observations'].append({'name': 'edit-close-cannot-mutate-admitted-execution',
                'source_closed': True, 'broker_status': after['data']['status'],
                'approved_snapshot_digest_unchanged': True, 'ledger_token_broker_status': unauthorized})
            # Project an actual immutable broker event into a namespaced ledger field.
            # Repeated projection reads its stable marker; it cannot loop into admission.
            broker_event = next(event for event in after['data']['events'] if event['kind'] == 'attempt.assign')
            projection = {'broker_task_id': task, 'broker_event_id': broker_event['id'],
                          'broker_revision': broker_event['sequence'], 'state': after['data']['status'],
                          'attempt_id': first[1]['attempt_id'], 'generation': first[1]['generation']}
            status, before_metadata = call('GET', issue_path + '/metadata', token=lead)
            assert status == 200
            status, projected = call('POST', issue_path + '/metadata',
                {'patch': {'corenius_execution': projection},
                 'guard': {'key': 'corenius_execution', 'if_absent': True}}, token=lead)
            assert status == 200 and projected['changed']
            projection_event = projected['event']
            for _ in range(2):
                status, retained = call('GET', issue_path + '/metadata', token=lead)
                assert status == 200 and retained['issue']['metadata']['corenius_execution'] == projection
                # Exercise an actual duplicate HTTP metadata delivery too, not only local lookup.
                status, replayed = call('POST', issue_path + '/metadata',
                    {'patch': {'corenius_execution': projection},
                     'guard': {'key': 'corenius_execution', 'if_value': canonical(projection).decode()}}, token=lead)
                assert status == 200 and not replayed['changed'] and not replayed.get('event')
            status, replay_events = call('GET', project_path + '/events?after_id=0', token=lead)
            assert status == 200
            projected_events = [event for event in replay_events['events']
                if event.get('issue_uid') == issue['uid'] and event['type'].startswith('issue.metadata')]
            assert len(projected_events) == 1, projected_events
            for _ in range(2):
                received = bridge.receive(projected_events[0])
                assert received['disposition'] == 'projection_observation'
            after_projection = bridge.snapshot()
            assert (after_projection['revision'], after_projection['data']) == (after['revision'], after['data'])
            status, mission_snapshot = http(broker_origin, owner, 'GET', '/v1/snapshots/mission/' + mission)
            assert status == 200
            assert [mission_snapshot['data'][key] for key in ['admitted_attempts', 'admitted_episodes', 'used_capacity']] == [1, 1, 1]
            # A freshly constructed reader reloads the stable binding after ledger mutation.
            rebuilt = EvaluationBridge(broker_origin, owner, epoch, binding_path)
            status, reused = rebuilt.command('source:' + digest(binding_key), 0, task_command)
            assert status == 200 and reused['revision'] == 1
            assert rebuilt.snapshot()['data']['spec_digest'] == spec_digest
            assert hashlib.sha256(checked(['git', '-C', str(git_root), 'show', commit + ':approved.json'])).hexdigest() == approved_digest
            records['broker_projection_event'] = broker_event
            records['kata_projection_event'] = projection_event
            records['post_projection_snapshot'] = after_projection
            records['mission_snapshot'] = mission_snapshot
            records['observations'].append({'name': 'actual-broker-projection-duplicate-and-loop',
                'projection_event_count': len(projected_events), 'broker_counts': [1, 1, 1, 1],
                'mission_admissions_capacity': [1, 1, 1], 'stable_binding_reloaded': True})
            # Restore only the owned ledger database; the broker stays running.
            assert stop_process(kata_process)
            owned = json.loads(checked(['docker', 'inspect', fixture], text=True))[0]
            assert owned['Config']['Labels']['corenius.kata17.bridge'] == 'true'
            assert checked(['docker', 'port', fixture, '5432/tcp'], text=True).strip() == port_reply
            dump = checked(['docker', 'exec', fixture, 'pg_dump', '-U', 'probe',
                            '-d', 'kata_fixture', '--format=custom'])
            checked(['docker', 'exec', fixture, 'dropdb', '-U', 'probe', 'kata_fixture'])
            checked(['docker', 'exec', fixture, 'createdb', '-U', 'probe', 'kata_fixture'])
            subprocess.run(['docker', 'exec', '-i', fixture, 'pg_restore', '-U', 'probe',
                            '-d', 'kata_fixture', '--exit-on-error'], input=dump, check=True, timeout=30)
            log = open(root / 'kata-restored.log', 'wb'); logs.append(log)
            kata_restored = subprocess.Popen([kata, 'daemon', 'start', '--foreground', '--listen',
                '127.0.0.1:' + str(kata_port)], env=kata_env, cwd=root,
                stdout=log, stderr=log, start_new_session=True); processes.append(kata_restored)
            wait_ready(kata_restored, lambda: call('GET', '/health', token=None))
            status, restored_events = call('GET', project_path + '/events?after_id=0', token=lead)
            assert status == 200
            restored_claim = next(event for event in restored_events['events']
                if event['event_uid'] == claim_event['event_uid'])
            assert restored_claim == claim_event
            restored_bridge = EvaluationBridge(broker_origin, owner, epoch, binding_path)
            status, restored_receipt = restored_bridge.receive(restored_claim, json.loads(decision_path.read_text()))
            assert status == 200 and restored_receipt == first[1]
            after_restore = restored_bridge.snapshot()
            assert (after_restore['revision'], after_restore['data']) == (after_projection['revision'], after_projection['data'])
            status, restored_metadata = call('GET', issue_path + '/metadata', token=lead)
            assert status == 200 and restored_metadata['issue']['metadata']['corenius_execution'] == projection
            records['post_ledger_restore_snapshot'] = after_restore
            records['restored_assignment_receipt'] = restored_receipt
            records['observations'].append({'name': 'ledger-pg-restore-replayed-event-one-broker-authority',
                'source_event_uid_preserved': True, 'source_origin_preserved': True,
                'approved_binding_reloaded': True, 'same_assignment_receipt': True,
                'broker_counts': [1, 1, 1, 1], 'projection_preserved': True})
            records['elapsed_seconds'] = round(time.monotonic() - start, 3)
    finally:
        records['cleanup'] = {'services_terminal': all([stop_process(proc) for proc in processes])}
        for log in logs:
            log.close()
        records['cleanup']['listeners_closed'] = all(not socket_open(port) for port in ports)
        if container_started:
            checked(['docker', 'stop', fixture])
        records['cleanup']['postgres_removed'] = subprocess.run(['docker', 'inspect', fixture],
            stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=5).returncode != 0
        temporary_context.cleanup()
        records['cleanup']['temporary_home_removed'] = not Path(temporary_context.name).exists()
    Path(args.output).write_text(json.dumps(records, indent=2) + '\n')
    return records


def socket_open(port):
    with socket.socket() as sock:
        return sock.connect_ex(('127.0.0.1', port)) == 0


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--kata-binary', required=True)
    parser.add_argument('--broker-binary', required=True)
    parser.add_argument('--broker-source', required=True)
    parser.add_argument('--accepted-manifest', required=True)
    parser.add_argument('--output', required=True)
    result = run(parser.parse_args())
    print(json.dumps({'observations': result['observations'], 'cleanup': result['cleanup']}, indent=2))
