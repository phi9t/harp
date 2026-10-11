"""Relay boundary tests. The fixture upstream is never live-model evidence."""
import http.client
import importlib.util
import io
import json
import pathlib
import socket
import tempfile
import threading
import unittest
import secrets
import time
import types

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)


class RelayBoundary(unittest.TestCase):
    def deadline_request(self, response_factory, limit):
        path = pathlib.Path(__file__).with_name('sqlite_responses_relay.py')
        spec = importlib.util.spec_from_file_location('deadline_relay', path)
        relay = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(relay)
        relay.STREAM_SECONDS = limit
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            evidence = root / 'metadata.jsonl'
            response = response_factory(relay)
            server = relay.Server(root / 'provider.sock', lambda *_: response, evidence)
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            started = time.monotonic()
            try:
                c = http.client.HTTPConnection('localhost', timeout=2)
                c.sock = socket.socket(socket.AF_UNIX)
                c.sock.connect(str(server.server_address))
                c.request('POST', '/v1/responses', body=json.dumps({
                    'stream': True, 'store': False, 'input': []}))
                c.getresponse().read()
                elapsed = time.monotonic() - started
                c.close()
            finally:
                server.shutdown()
                server.server_close()
                thread.join()
            return json.loads(evidence.read_text()), elapsed

    def test_eof_after_deadline_is_not_recorded_complete(self):
        def response_factory(relay):
            clock = [100.0]
            relay.time = types.SimpleNamespace(monotonic=lambda: clock[0], time=lambda: clock[0])
            class Response(io.BytesIO):
                status = 200
                def read1(self, size):
                    clock[0] += 0.06
                    return b''
            return Response(b'')
        record, _ = self.deadline_request(response_factory, 0.02)
        self.assertFalse(record.get('stream_complete', False))
        self.assertIn('failure_class', record)

    def test_stalled_http_response_read_is_bounded_by_remaining_deadline(self):
        peers = []
        def response_factory(relay):
            local, peer = socket.socketpair()
            peers.extend([local, peer])
            peer.sendall(b'HTTP/1.0 200 OK\r\nContent-Type: text/event-stream\r\n\r\n')
            response = http.client.HTTPResponse(local)
            response.begin()
            def delayed_eof():
                time.sleep(0.3)
                peer.close()
            threading.Thread(target=delayed_eof, daemon=True).start()
            return response
        try:
            record, elapsed = self.deadline_request(response_factory, 0.02)
            self.assertLess(elapsed, 0.15, 'socket read escaped stream deadline')
            self.assertFalse(record.get('stream_complete', False))
            self.assertIn('failure_class', record)
        finally:
            for peer in peers:
                peer.close()

    def test_trickling_chunk_header_cannot_extend_absolute_deadline(self):
        peers = []
        senders = []
        def response_factory(relay):
            local, peer = socket.socketpair()
            peers.extend([local, peer])
            peer.sendall(b'HTTP/1.0 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n')
            response = http.client.HTTPResponse(local)
            response.begin()
            def trickle():
                try:
                    for _ in range(30):
                        peer.sendall(b'1')
                        time.sleep(0.01)
                except OSError:
                    pass
                finally:
                    peer.close()
            sender = threading.Thread(target=trickle, daemon=True)
            senders.append(sender)
            sender.start()
            return response
        try:
            record, elapsed = self.deadline_request(response_factory, 0.04)
            self.assertLess(elapsed, 0.15, 'chunk framing kept extending socket inactivity timeout')
            self.assertFalse(record.get('stream_complete', False))
            self.assertIn('failure_class', record)
        finally:
            for peer in peers:
                peer.close()
            for sender in senders:
                sender.join(timeout=1)

    def test_runtime_generated_body_and_header_canaries_absent_from_evidence(self):
        path = pathlib.Path(__file__).with_name('sqlite_responses_relay.py')
        spec = importlib.util.spec_from_file_location('relay_privacy', path)
        relay = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(relay)
        body_canary, header_canary = secrets.token_hex(32), secrets.token_hex(32)

        class Response(io.BytesIO):
            status = 200

        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            evidence = root / 'metadata.jsonl'
            server = relay.Server(root / 'provider.sock', lambda *_:Response(b'data: {}\n\n'), evidence)
            thread = threading.Thread(target=server.serve_forever,daemon=True)
            thread.start()
            try:
                c = http.client.HTTPConnection('localhost',timeout=2)
                c.sock = socket.socket(socket.AF_UNIX)
                c.sock.connect(str(server.server_address))
                c.request('POST','/v1/responses',body=json.dumps({
                    'model':{'untrusted':body_canary},'stream':True,'store':False,'input':[]}),
                    headers={'session_id':header_canary})
                response = c.getresponse()
                self.assertEqual(response.status,200)
                response.read()
                c.close()
            finally:
                server.shutdown()
                server.server_close()
            retained = evidence.read_text()
            self.assertFalse(body_canary in retained, 'body canary reached evidence')
            self.assertFalse(header_canary in retained, 'header canary reached evidence')

    def test_fixed_route_strips_caller_authority_and_streams(self):
        path = pathlib.Path(__file__).with_name('sqlite_responses_relay.py')
        self.assertTrue(path.exists(), 'retained Responses Unix relay is missing')
        spec = importlib.util.spec_from_file_location('relay', path)
        relay = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(relay)
        calls = []

        class Response(io.BytesIO):
            status = 200

        def upstream(body, headers):
            calls.append((body, headers))
            return Response(b'data: {"type":"response.completed"}\n\n')

        with tempfile.TemporaryDirectory() as directory:
            server = relay.Server(pathlib.Path(directory) / 'provider.sock', upstream)
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            try:
                def request(path, body, headers=None):
                    c = http.client.HTTPConnection('localhost', timeout=2)
                    c.sock = socket.socket(socket.AF_UNIX)
                    c.sock.connect(str(server.server_address))
                    c.request('POST', path, body=body, headers=headers or {})
                    response = c.getresponse()
                    result = response.status, response.read()
                    c.close()
                    return result

                body = json.dumps({'model':'gpt-6-sol', 'store':False, 'stream':True, 'input':[]})
                status, data = request('/v1/responses', body, {
                    'Authorization':'Bearer foreign', 'ChatGPT-Account-Id':'foreign',
                    'Host':'foreign.invalid', 'Content-Type':'application/json',
                    'session_id':'scoped-session'})
                self.assertEqual(status, 200)
                self.assertIn(b'response.completed', data)
                self.assertEqual(len(calls), 1)
                self.assertEqual(calls[0][1], {'session_id':'scoped-session'})
                self.assertEqual(request('/arbitrary', body)[0], 404)
                self.assertEqual(request('/v1/responses', body,
                                         {'Content-Type':'x' * 20000})[0], 431)
                self.assertEqual(request('/v1/responses', b'',
                                         {'Content-Length':str(relay.REQUEST_LIMIT + 1)})[0], 413)
                self.assertEqual(len(calls), 1)
            finally:
                server.shutdown()
                server.server_close()


if __name__ == '__main__':
    unittest.main()
