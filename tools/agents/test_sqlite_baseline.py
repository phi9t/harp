import os
import pathlib
import subprocess
import importlib.util
import json
import os
import shlex
import tempfile
import time
import uuid
import unittest
import http.server
import threading

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


class BaselineEntry(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()

    def tearDown(self):
        self.tmp.cleanup()

    def test_pinned_kata_lookup_survives_retired_worktree_and_preserves_overrides(self):
        repo = pathlib.Path(__file__).resolve().parents[2]
        sha = '8b0f05b1071f5e7a2b8173c9e5ef358d550a601cebd8c9d9cae6e45975e837f6'
        stable = pathlib.Path.home() / '.local/share/corenius/pinned-tools/kata' / sha / 'kata-pinned'
        alternate = pathlib.Path(self.tmp.name) / 'alternate-kata'
        alternate.symlink_to(stable)
        program = r"""
import ast, hashlib, json, os, pathlib, runpy, sys
repo = pathlib.Path(sys.argv[1])
sys.path.insert(0, str(repo / 'bench/harness'))
controller = runpy.run_path(str(repo / 'bench/harness/controller_mode.py'))
baseline = runpy.run_path(str(repo / 'tools/agents/sqlite_baseline.py'))
fixture = repo / 'crates/sqlite-controller/tests/support/kata_protocol.py'
tree = ast.parse(fixture.read_text(), filename=str(fixture))
# Execute the fixture's actual imports, lookup and mandatory hash guard;
# stop before it allocates a run or starts any service.
stop = next(i for i, stmt in enumerate(tree.body) if isinstance(stmt, ast.Assign)
            and any(isinstance(target, ast.Name) and target.id == 'ROOT' for target in stmt.targets))
namespace = {'__file__': str(fixture)}
exec(compile(ast.Module(body=tree.body[:stop], type_ignores=[]), str(fixture), 'exec'), namespace)
paths = [controller['KATA'], baseline['KATA'], namespace['PIN']]
assert all(str(path) == sys.argv[2] for path in paths), paths
assert all(os.access(path, os.X_OK) for path in paths)
assert all(hashlib.sha256(path.read_bytes()).hexdigest() == namespace['SHA'] for path in paths)
print(json.dumps([str(path) for path in paths]))
"""
        env = dict(os.environ)
        env.pop('CORENIUS_KATA_PINNED_BINARY', None)
        env.pop('KATA_PINNED_BINARY', None)
        for expected in (stable, alternate):
            with self.subTest(path=str(expected)):
                if expected == alternate:
                    env.update(CORENIUS_KATA_PINNED_BINARY=str(alternate), KATA_PINNED_BINARY=str(alternate))
                result = subprocess.run(['python3', '-c', program, str(repo), str(expected)],
                                        env=env, capture_output=True, text=True)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(json.loads(result.stdout), [str(expected)] * 3)
        corrupt = pathlib.Path(self.tmp.name) / 'corrupt-kata'
        corrupt.write_bytes(b'not the approved Kata binary')
        env['KATA_PINNED_BINARY'] = str(corrupt)
        rejected = subprocess.run(['python3', '-c', program, str(repo), str(alternate)],
                                  env=env, capture_output=True, text=True)
        self.assertNotEqual(rejected.returncode, 0)
        self.assertIn('AssertionError', rejected.stderr)

    def test_start_accepts_explicit_existing_session_and_pinned_reuse_options(self):
        script = pathlib.Path(__file__).with_name('sqlite_baseline.py')
        result = subprocess.run(['python3', str(script), 'start', '--help'], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0)
        for option in ['--session', '--reuse-kata', '--target-dir', '--model']:
            self.assertIn(option, result.stdout)

    def test_reuse_kata_qualifies_instance_before_copying_private_grant(self):
        # Ignoring the pinned instance would attach the new project to an
        # unrelated ledger and copy its private grant before qualification.
        script = pathlib.Path(__file__).with_name('sqlite_baseline.py')
        spec = importlib.util.spec_from_file_location('baseline', script)
        baseline = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(baseline)
        requests = []
        identity = {'instance_uid': 'expected-instance', 'auth': {'actor': 'corenius-controller'}}
        class Handler(http.server.BaseHTTPRequestHandler):
            def do_GET(self):
                requests.append(self.path)
                body = json.dumps(identity).encode()
                self.send_response(200)
                self.end_headers()
                self.wfile.write(body)
            def log_message(self, *args):
                pass
        server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        try:
            with tempfile.TemporaryDirectory(prefix='cb-reuse-', dir=self.tmp.name) as directory:
                parent = pathlib.Path(directory)
                root = parent / 'new'; root.mkdir(mode=0o700)
                baseline.write(root / 'private.json', {'kata_port': 1})
                token = parent / 'kata.token'; token.write_text('test-private-grant'); token.chmod(0o600)
                config = parent / 'kata.json'
                baseline.write(config, {'origin': 'http://127.0.0.1:' + str(server.server_port),
                                        'origin_instance_uid': 'wrong-instance', 'actor': 'corenius-controller',
                                        'api_schema_version': '0.24.0', 'token_file': str(token)})
                with self.assertRaisesRegex(RuntimeError, 'Kata reuse identity mismatch'):
                    baseline.reuse_kata(root, baseline.pin(config))
                self.assertFalse((root / 'kata.token').exists())
                value = json.loads(config.read_text()); value['origin_instance_uid'] = 'expected-instance'
                baseline.write(config, value)
                baseline.reuse_kata(root, baseline.pin(config))
                self.assertEqual((root / 'kata.token').read_text(), 'test-private-grant')
                self.assertEqual((root / 'kata.token').stat().st_mode & 0o777, 0o600)
                self.assertEqual(json.loads((root / 'private.json').read_text())['kata_port'], server.server_port)
                self.assertEqual(token.read_text(), 'test-private-grant')
                self.assertEqual(requests, ['/api/v1/instance', '/api/v1/instance'])
        finally:
            server.shutdown(); server.server_close(); thread.join()

    def test_retained_entry_point_exposes_start_status_and_literal_input(self):
        script = pathlib.Path(__file__).with_name('sqlite_baseline.py')
        self.assertTrue(script.exists(), 'retained disposable tmux baseline runner missing')
        result = subprocess.run(['python3', str(script), '--help'], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0)
        for command in ['start', 'status', 'input']:
            self.assertIn(command, result.stdout)

    def test_literal_tmux_ingress_preserves_utf8_and_shell_metacharacters(self):
        # Removing literal buffer paste or interpolating a shell command would
        # corrupt this input or execute its command substitutions.
        script = pathlib.Path(__file__).with_name('sqlite_baseline.py')
        spec = importlib.util.spec_from_file_location('baseline', script)
        baseline = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(baseline)
        session = 'corenius-input-test-' + uuid.uuid4().hex[:10]
        with tempfile.TemporaryDirectory(prefix='cb-input-', dir=self.tmp.name) as directory:
            root = pathlib.Path(directory)
            output = root / 'received.bin'
            child = root / 'receiver.py'
            child.write_text('import os,tty,pathlib\ntty.setraw(0)\n'
                             'data=b""\n'
                             'while not data.endswith(b"\\r"):\n'
                             ' data+=os.read(0,65536)\n'
                             'pathlib.Path(' + repr(str(output)) + ').write_bytes(data)\n')
            result = subprocess.run(['tmux','new-session','-d','-P','-F','#{pane_id}',
                                     '-s',session,shlex.join(['python3',str(child)])],
                                    check=True,capture_output=True,text=True)
            pane = result.stdout.strip()
            try:
                (root / 'manifest.json').write_text(json.dumps({'run_id':session,'lead_pane':pane}))
                time.sleep(.2)
                text = 'UTF8 λ literal $(touch ' + str(root / 'canary') + ') `touch nope`; "quotes"'
                baseline.literal(root, text)
                deadline = time.monotonic() + 3
                while not output.exists() and time.monotonic() < deadline:
                    time.sleep(.02)
                self.assertTrue(output.exists())
                self.assertEqual(output.read_bytes(),b'\x1b[200~' + text.encode() + b'\x1b[201~\r')
                self.assertFalse((root / 'canary').exists())
            finally:
                subprocess.run(['tmux','kill-session','-t',session],capture_output=True)

    def test_run_root_uses_tmpdir_or_runs_root_without_host_tmp(self):
        script = pathlib.Path(__file__).with_name('sqlite_baseline.py')
        spec = importlib.util.spec_from_file_location('baseline', script)
        baseline = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(baseline)
        forbidden = pathlib.Path(self.tmp.name) / 'forbidden-host-tmp'
        forbidden.mkdir()
        try:
            old_tmpdir = os.environ.pop('TMPDIR', None)
            old_runs = os.environ.get('CORENIUS_RUNS')
            baseline.tempfile.tempdir = str(forbidden)
            runs_root = pathlib.Path(self.tmp.name) / 'runs'
            os.environ['CORENIUS_RUNS'] = str(runs_root)
            first = baseline.new_run_root('abc123')
            self.assertTrue(first.is_relative_to(runs_root))
            self.assertEqual(list(forbidden.rglob('*')), [])

            explicit = pathlib.Path(self.tmp.name) / 'explicit'
            os.environ['TMPDIR'] = str(explicit)
            second = baseline.new_run_root('def456')
            self.assertTrue(second.is_relative_to(explicit))
        finally:
            baseline.tempfile.tempdir = None
            if old_tmpdir is None:
                os.environ.pop('TMPDIR', None)
            else:
                os.environ['TMPDIR'] = old_tmpdir
            if old_runs is None:
                os.environ.pop('CORENIUS_RUNS', None)
            else:
                os.environ['CORENIUS_RUNS'] = old_runs


if __name__ == '__main__':
    unittest.main()
