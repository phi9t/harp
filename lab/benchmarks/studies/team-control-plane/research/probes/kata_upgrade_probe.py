#!/usr/bin/env python3
"""Cold, isolated Kata SQLite upgrade probe between two exact binaries."""
import argparse, hashlib, json, os, secrets, signal, socket, subprocess
import tempfile, time, urllib.error, urllib.request
from pathlib import Path

parser = argparse.ArgumentParser()
parser.add_argument('--old-binary', required=True)
parser.add_argument('--new-binary', required=True)
parser.add_argument('--output', required=True)
args = parser.parse_args()
binaries = [str(Path(args.old_binary).resolve()), str(Path(args.new_binary).resolve())]
observations = []
with tempfile.TemporaryDirectory(prefix='corenius-kata17-upgrade-') as temporary:
    home = Path(temporary)
    home.chmod(0o700)
    bootstrap = secrets.token_hex(32)
    config = home / 'config.toml'
    config.write_text('[auth]\ntoken = "' + bootstrap + '"\nrequire_token_identity = true\n')
    config.chmod(0o600)
    env = {key: value for key, value in os.environ.items() if not key.startswith('KATA_')}
    env['KATA_HOME'] = temporary
    with socket.socket() as sock:
        sock.bind(('127.0.0.1', 0))
        port = sock.getsockname()[1]
    origin = 'http://127.0.0.1:' + str(port)
    def call(method, path, token, body=None):
        headers = {'Content-Type': 'application/json'}
        if token:
            headers['Authorization'] = 'Bearer ' + token
        request = urllib.request.Request(origin + '/api/v1' + path,
            data=None if body is None else json.dumps(body).encode(),
            headers=headers, method=method)
        try:
            with urllib.request.urlopen(request, timeout=5) as response:
                return response.status, json.load(response)
        except urllib.error.HTTPError as error:
            return error.code, json.load(error)
    for index, binary in enumerate(binaries):
        with open(home / ('daemon-' + str(index) + '.log'), 'wb') as log:
            proc = subprocess.Popen([binary, 'daemon', 'start', '--foreground',
                '--listen', '127.0.0.1:' + str(port)], cwd=temporary, env=env,
                stdout=log, stderr=log, start_new_session=True)
            try:
                for _ in range(100):
                    if proc.poll() is not None:
                        raise RuntimeError('isolated daemon exited on startup')
                    try:
                        status, health = call('GET', '/health', None)
                        if status == 200:
                            break
                    except urllib.error.URLError:
                        pass
                    time.sleep(0.1)
                else:
                    raise RuntimeError('isolated daemon startup timed out')
                if index == 0:
                    status, grant = call('POST', '/tokens', bootstrap, {'actor': 'fixture-lead'})
                    assert status == 200, status
                    lead = grant['plaintext']
                    status, reply = call('POST', '/projects', lead, {'name': 'upgrade-fixture'})
                    assert status == 200, (status, reply)
                    project_id = reply['project']['id']
                    path = '/projects/' + str(project_id) + '/issues'
                    status, reply = call('POST', path, lead, {'title': 'retained-before-upgrade'})
                    assert status == 200, (status, reply)
                    issue = reply['issue']
                    path += '/' + issue['short_id']
                    status, reply = call('POST', path + '/actions/claim', lead, {'actor': 'fixture-lead'})
                    assert status == 200, (status, reply)
                    observations.append({'phase': 'old-created', 'api_schema_version':
                        health.get('api_schema_version'), 'issue_uid': issue['uid']})
                else:
                    status, reply = call('GET', path, lead)
                    assert status == 200, (status, reply)
                    retained = reply['issue']
                    assert retained['uid'] == issue['uid']
                    assert retained['title'] == 'retained-before-upgrade'
                    assert retained['owner'] == 'fixture-lead'
                    status, reply = call('POST', path + '/actions/claim', lead, {'if_unowned': True})
                    assert status == 409, (status, reply)
                    observations.append({'phase': 'new-read', 'api_schema_version':
                        health.get('api_schema_version'), 'issue_uid_retained': True,
                        'title_retained': True, 'claim_retained': True,
                        'retained_credential_authenticated': True, 'reclaim_refused': status})
            finally:
                if proc.poll() is None:
                    os.killpg(proc.pid, signal.SIGTERM)
                try:
                    proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    os.killpg(proc.pid, signal.SIGKILL)
                    proc.wait(timeout=5)
                with socket.socket() as sock:
                    assert sock.connect_ex(('127.0.0.1', port)) != 0
    observations.append({'phase': 'cleanup', 'all_processes_terminal': True,
                         'listener_closed': True})
result = {'binaries': [{'sha256': hashlib.sha256(Path(binary).read_bytes()).hexdigest(),
    'version': subprocess.check_output([binary, 'version'], text=True)} for binary in binaries],
    'observations': observations}
Path(args.output).write_text(json.dumps(result, indent=2) + '\n')
print(json.dumps(observations, indent=2))
