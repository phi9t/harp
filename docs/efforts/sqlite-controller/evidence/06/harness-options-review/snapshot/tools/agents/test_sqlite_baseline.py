import pathlib
import subprocess
import importlib.util
import json
import shlex
import tempfile
import time
import uuid
import unittest
import http.server
import threading


class BaselineEntry(unittest.TestCase):
    def test_start_accepts_explicit_existing_session_and_pinned_reuse_options(self):
        script = pathlib.Path(__file__).with_name('sqlite_baseline.py')
        result = subprocess.run(['python3', str(script), 'start', '--help'], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0)
        for option in ['--session', '--reuse-kata', '--target-dir']:
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
            with tempfile.TemporaryDirectory(prefix='cb-reuse-', dir='/tmp') as directory:
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
        with tempfile.TemporaryDirectory(prefix='cb-input-', dir='/tmp') as directory:
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


if __name__ == '__main__':
    unittest.main()
