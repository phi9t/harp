#!/usr/bin/env python3
"""Host-only fixed Codex Responses relay; participants never receive host auth.

This uses the current installed CLI ChatGPT login, without copying or refreshing
it. The pinned 0.159.3 binary embeds this backend origin and account header.
No configurable upstream, redirects, credential logging, or executor TCP route.
"""
import argparse
import base64
import hashlib
import http.client
import http.server
import json
import os
import pathlib
import socketserver
import ssl
import stat
import threading
import time

ORIGIN = 'chatgpt.com'
UPSTREAM_PATH = '/backend-api/codex/responses'
REQUEST_LIMIT = 2 * 1024 * 1024
RESPONSE_LIMIT = 16 * 1024 * 1024
HEADER_LIMIT = 16384
STREAM_SECONDS = 180


def host_headers():
    path = pathlib.Path.home() / '.codex/auth.json'
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        info = os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_size > 32768:
            raise ValueError('host authentication file identity refused')
        with os.fdopen(fd, 'rb', closefd=False) as source:
            value = json.load(source)
        tokens = value.get('tokens', {})
        access, account = tokens.get('access_token'), tokens.get('account_id')
        if value.get('auth_mode') != 'chatgpt' or not access or not account:
            raise ValueError('current host ChatGPT login unavailable')
        claims = json.loads(base64.urlsafe_b64decode(access.split('.')[1] + '==='))
        if claims.get('exp', 0) <= time.time():
            raise ValueError('current host ChatGPT access expired; relay does not refresh it')
        return {'Authorization':'Bearer ' + access, 'ChatGPT-Account-Id':account,
                'Content-Type':'application/json', 'Accept':'text/event-stream',
                'originator':'codex_cli_rs', 'User-Agent':'codex_cli_rs/0.159.3'}
    finally:
        os.close(fd)


def upstream(body, headers):
    connection = http.client.HTTPSConnection(ORIGIN, timeout=30, context=ssl.create_default_context())
    trusted = host_headers()
    trusted.update(headers)
    connection.request('POST', UPSTREAM_PATH, body=body, headers=trusted)
    response = connection.getresponse()
    response._owned_connection = connection
    return response


class Server(socketserver.ThreadingMixIn, socketserver.UnixStreamServer):
    daemon_threads = True
    block_on_close = False

    def __init__(self, path, transport=upstream, evidence=None):
        self.transport, self.evidence = transport, evidence
        self.slots = threading.BoundedSemaphore(6)
        self.evidence_lock = threading.Lock()
        super().__init__(str(path), Handler)
        os.chmod(path, 0o600)

    def record(self, value):
        if self.evidence:
            with self.evidence_lock, open(self.evidence, 'a') as destination:
                destination.write(json.dumps(value) + '\n')


class Handler(http.server.BaseHTTPRequestHandler):
    protocol_version = 'HTTP/1.0'
    server_version = 'CoreniusRelay/1'

    def setup(self):
        self.request.settimeout(10)
        super().setup()

    def log_message(self, *_):
        pass

    def error(self, status):
        body = json.dumps({'error':{'code':'relay_' + str(status)}}).encode()
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def do_POST(self):
        self.connection.settimeout(30)
        started = time.time()
        record = {'started':started, 'path':'/v1/responses'}
        response = None
        acquired = False
        try:
            if self.path != '/v1/responses':
                return self.error(404)
            if sum(len(k) + len(v) + 4 for k, v in self.headers.items()) > HEADER_LIMIT:
                return self.error(431)
            lengths = self.headers.get_all('Content-Length', [])
            if len(lengths) != 1 or self.headers.get('Transfer-Encoding'):
                return self.error(400)
            length = int(lengths[0])
            if length < 0 or length > REQUEST_LIMIT:
                return self.error(413)
            deadline = time.monotonic() + 30
            body = bytearray()
            while len(body) < length:
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise TimeoutError('request body deadline')
                self.connection.settimeout(remaining)
                chunk = self.rfile.read1(min(65536, length - len(body)))
                if not chunk:
                    break
                body.extend(chunk)
            body = bytes(body)
            if len(body) != length:
                return self.error(400)
            value = json.loads(body)
            if not isinstance(value, dict) or value.get('stream') is not True or value.get('store') is not False:
                return self.error(400)
            acquired = self.server.slots.acquire(blocking=False)
            if not acquired:
                return self.error(429)
            # Attribution only. Caller authentication, Host, redirects and routes
            # never enter the host transport.
            allowed = {'session_id', 'conversation_id', 'x-codex-turn-metadata'}
            headers = {k.lower():v for k, v in self.headers.items() if k.lower() in allowed}
            record.update(request_bytes=length, request_sha256=hashlib.sha256(body).hexdigest())
            response = self.server.transport(body, headers)
            record['upstream_status'] = response.status
            if response.status != 200:
                return self.error(response.status if 400 <= response.status < 600 else 502)
            self.send_response(200)
            self.send_header('Content-Type', 'text/event-stream')
            self.end_headers()
            total = 0
            deadline = time.monotonic() + STREAM_SECONDS
            while time.monotonic() < deadline:
                chunk = response.read1(65536) if hasattr(response, 'read1') else response.read(65536)
                if not chunk:
                    record['stream_complete'] = True
                    break
                total += len(chunk)
                if total > RESPONSE_LIMIT:
                    raise ValueError('response byte limit')
                self.wfile.write(chunk)
                self.wfile.flush()
            else:
                raise ValueError('response wall deadline')
            record['response_bytes'] = total
        except Exception as error:
            # Exception messages, request/response bodies and headers may contain
            # upstream secrets or reasoning; retain only the safe class.
            record['failure_class'] = type(error).__name__
            if response is None:
                try:
                    self.error(502)
                except OSError:
                    pass
        finally:
            if response is not None:
                response.close()
                connection = getattr(response, '_owned_connection', None)
                if connection:
                    connection.close()
            if acquired:
                self.server.slots.release()
            record['finished'] = time.time()
            self.server.record(record)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('socket', type=pathlib.Path)
    parser.add_argument('--evidence', type=pathlib.Path, required=True)
    args = parser.parse_args()
    parent = args.socket.parent
    if parent.resolve() != parent or parent.stat().st_uid != os.getuid() or stat.S_IMODE(parent.stat().st_mode) != 0o700:
        parser.error('socket parent must be an owned canonical 0700 directory')
    host_headers()  # Fail before readiness; never write these returned headers.
    with Server(args.socket, evidence=args.evidence) as server:
        print('Host Responses relay ready', flush=True)
        server.serve_forever()


if __name__ == '__main__':
    main()
