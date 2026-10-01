#!/usr/bin/env python3
"""Model-free Kata remote API probe; isolated home, no shared configuration."""
import argparse, concurrent.futures, hashlib, json, os, secrets, signal, socket
import subprocess, tempfile, time, urllib.error, urllib.request, shutil
from pathlib import Path

def owned_postgres(container, port, expected_id=None):
    """Resolve only the labeled disposable fixture matching the measured DSN."""
    if not container.startswith('corenius-kata17-') or not port or not 0 < port < 65536:
        raise RuntimeError('invalid owned PostgreSQL fixture reference')
    inspected = json.loads(subprocess.check_output(['docker', 'inspect', container]))
    if len(inspected) != 1:
        raise RuntimeError('ambiguous PostgreSQL fixture reference')
    identity = inspected[0]
    labels = identity['Config'].get('Labels') or {}
    config = identity['HostConfig']
    bindings = identity['NetworkSettings']['Ports'].get('5432/tcp')
    if (labels.get('corenius.kata17') != 'true'
            or not identity['State']['Running']
            or config.get('Binds')
            or '/var/lib/postgresql/data' not in (config.get('Tmpfs') or {})
            or bindings != [{'HostIp': '127.0.0.1', 'HostPort': str(port)}]
            or (expected_id is not None and identity['Id'] != expected_id)):
        raise RuntimeError('PostgreSQL fixture ownership, storage or DSN identity mismatch')
    return identity['Id']

parser = argparse.ArgumentParser()
parser.add_argument('--binary', required=True)
parser.add_argument('--output', required=True)
parser.add_argument('--pg-container', help='Owned disposable corenius-kata17-* fixture container')
parser.add_argument('--pg-port', type=int)
args = parser.parse_args()
binary = str(Path(args.binary).resolve())
records = []
with tempfile.TemporaryDirectory(prefix='corenius-kata17-') as temporary:
    home = Path(temporary)
    home.chmod(0o700)
    bootstrap = secrets.token_hex(32)
    config = home / 'config.toml'
    config.write_text('[auth]\ntoken = "' + bootstrap + '"\nrequire_token_identity = true\n')
    config.chmod(0o600)
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    origin = 'http://127.0.0.1:' + str(port)
    env = {k: v for k, v in os.environ.items() if not k.startswith('KATA_')}
    env['KATA_HOME'] = temporary
    if args.pg_container:
        pg_identity = owned_postgres(args.pg_container, args.pg_port)
        env['KATA_DSN'] = 'postgres://probe@127.0.0.1:' + str(args.pg_port) + '/probe?sslmode=disable'
    log = open(home / 'daemon.log', 'wb')
    proc = subprocess.Popen([binary, 'daemon', 'start', '--foreground', '--listen',
                             '127.0.0.1:' + str(port)], env=env, cwd=temporary,
                            stdout=log, stderr=log, start_new_session=True)
    def call(method, path, body=None, token=bootstrap, headers=None):
        request_headers = {'Content-Type': 'application/json'}
        if token:
            request_headers['Authorization'] = 'Bearer ' + token
        request_headers.update(headers or {})
        data = None if body is None else json.dumps(body).encode()
        request = urllib.request.Request(origin + '/api/v1' + path, data=data,
                                         method=method, headers=request_headers)
        try:
            with urllib.request.urlopen(request, timeout=5) as response:
                return response.status, json.load(response)
        except urllib.error.HTTPError as error:
            return error.code, json.load(error)
    def record(name, status, **fields):
        records.append(dict(name=name, status=status, **fields))
    try:
        for _ in range(100):
            if proc.poll() is not None:
                raise RuntimeError('isolated daemon exited during startup')
            try:
                status, health = call('GET', '/health', token=None)
                if status == 200:
                    break
            except (urllib.error.URLError, TimeoutError):
                pass
            time.sleep(0.1)
        else:
            raise RuntimeError('isolated daemon startup timeout')
        record('health', status, api_schema_version=health.get('api_schema_version'))
        status, minted = call('POST', '/tokens', {'actor': 'lead', 'name': 'fixture-lead'})
        assert status == 200, (status, minted)
        lead = minted['plaintext']
        status, project_reply = call('POST', '/projects', {'name': 'corenius-fixture'}, token=lead)
        assert status == 200, (status, project_reply)
        project = project_reply['project']
        base = '/projects/' + str(project['id']) + '/issues'
        def issue(title):
            status, reply = call('POST', base, {'title': title}, token=lead)
            assert status == 200, (status, reply)
            return reply['issue']
        a, b = issue('approved-original'), issue('outside-worker-scope')
        status, minted = call('POST', '/tokens', {'actor': 'worker-a',
             'scope': {'kind': 'issue_subtree', 'project_uid': project['uid'],
                       'root_issue_uid': a['uid']}, 'expires_in_seconds': 4})
        assert status == 200, (status, minted)
        worker = minted['plaintext']
        path_a, path_b = base + '/' + a['short_id'], base + '/' + b['short_id']
        status, reply = call('GET', path_b, token=worker)
        assert status in (403, 404), (status, reply)
        record('scoped-read-outside', status)
        status, reply = call('PATCH', path_a, {'title': 'edited-after-approval'}, token=worker)
        assert status == 200, (status, reply)
        status, edited = call('GET', path_a, token=worker)
        assert status == 200 and edited['issue']['title'] == 'edited-after-approval'
        record('source-edit', status, original_revision=a['revision'],
               resulting_revision=edited['issue']['revision'], title_changed=True)
        status, reply = call('POST', path_a + '/actions/close', {
            'reason': 'done', 'message': 'fixture claimed completion for an unexecuted test command',
            'evidence': [{'type': 'command', 'command': 'true'}]}, token=worker)
        record('direct-close-with-unexecuted-command-evidence', status,
               closed_reason=reply.get('issue', {}).get('closed_reason'))
        close_body = {'retry_protocol': 'close-v1', 'reason': 'done', 'message': 'fixture claimed completion for an unexecuted test command',
                      'evidence': [{'type': 'test', 'command': 'false'}]}
        close_headers = {'Idempotency-Key': 'fixture-close-once'}
        status, closed = call('POST', path_a + '/actions/close', close_body,
                             token=worker, headers=close_headers)
        assert status == 200, (status, closed)
        record('direct-close-with-unexecuted-test-evidence', status,
               closed_reason=closed.get('issue', {}).get('closed_reason'))
        status, replay = call('POST', path_a + '/actions/close', close_body,
                             token=worker, headers=close_headers)
        assert status == 200 and replay.get('original_event'), (status, replay)
        record('duplicate-close-replay', status, has_original_event=True)
        time.sleep(4.1)
        status, reply = call('GET', path_a, token=worker)
        assert status in (401, 403), (status, reply)
        record('expired-token-read', status)
        tokens = []
        for actor in ('claimant-one', 'claimant-two'):
            status, minted = call('POST', '/tokens', {'actor': actor})
            assert status == 200, (status, minted)
            tokens.append(minted['plaintext'])
        def claim(token):
            return call('POST', path_b + '/actions/claim', {'if_unowned': True}, token=token)
        with concurrent.futures.ThreadPoolExecutor(max_workers=2) as executor:
            responses = list(executor.map(claim, tokens))
        statuses = sorted(status for status, _ in responses)
        assert statuses == [200, 409], statuses
        record('claim-race', statuses, successful_claims=1)
        # A stopped-process copy measures cold restore, not online DB backup.
        os.killpg(proc.pid, signal.SIGTERM)
        proc.wait(timeout=5)
        log.close()
        if args.pg_container:
            owned_postgres(args.pg_container, args.pg_port, pg_identity)
            dump = subprocess.check_output(['docker', 'exec', pg_identity,
                'pg_dump', '-U', 'probe', '-d', 'probe', '--format=custom'])
            owned_postgres(args.pg_container, args.pg_port, pg_identity)
            subprocess.check_call(['docker', 'exec', pg_identity,
                'dropdb', '-U', 'probe', 'probe'])
            subprocess.check_call(['docker', 'exec', pg_identity,
                'createdb', '-U', 'probe', 'probe'])
            subprocess.run(['docker', 'exec', '-i', pg_identity,
                'pg_restore', '-U', 'probe', '-d', 'probe', '--exit-on-error'],
                input=dump, check=True)
        with tempfile.TemporaryDirectory(prefix='corenius-kata17-restore-') as restored:
            shutil.copytree(home, Path(restored) / 'home')
            env['KATA_HOME'] = str(Path(restored) / 'home')
            log = open(Path(restored) / 'restored-daemon.log', 'wb')
            proc = subprocess.Popen([binary, 'daemon', 'start', '--foreground', '--listen',
                '127.0.0.1:' + str(port)], env=env, cwd=restored,
                stdout=log, stderr=log, start_new_session=True)
            for _ in range(100):
                if proc.poll() is not None:
                    raise RuntimeError('restored daemon exited during startup')
                try:
                    status, _ = call('GET', '/health', token=None)
                    if status == 200:
                        break
                except urllib.error.URLError:
                    pass
                time.sleep(0.1)
            else:
                raise RuntimeError('restored daemon startup timeout')
            status, restored_a = call('GET', path_a, token=lead)
            assert status == 200 and restored_a['issue']['closed_reason'] == 'done'
            status, restored_b = call('GET', path_b, token=lead)
            assert status == 200 and restored_b['issue']['owner'] in ('claimant-one', 'claimant-two')
            status, _ = call('GET', path_a, token=worker)
            assert status == 403
            record('postgres-logical-restore' if args.pg_container else 'cold-sqlite-restore', 200, closed_issue_preserved=True,
                   winning_claim_preserved=True, expired_token_still_denied=True)
            os.killpg(proc.pid, signal.SIGTERM)
            proc.wait(timeout=5)
            log.close()
    finally:
        if proc.poll() is None:
            os.killpg(proc.pid, signal.SIGTERM)
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            os.killpg(proc.pid, signal.SIGKILL)
            proc.wait(timeout=5)
        log.close()
        cleanup = proc.poll() is not None
        with socket.socket() as sock:
            port_closed = sock.connect_ex(('127.0.0.1', port)) != 0
        records.append({'name': 'cleanup', 'process_terminal': cleanup,
                        'listener_closed': port_closed})
Path(args.output).write_text(json.dumps({'binary_sha256': hashlib.sha256(
    Path(binary).read_bytes()).hexdigest(), 'observations': records}, indent=2) + '\n')
print(json.dumps(records, indent=2))
