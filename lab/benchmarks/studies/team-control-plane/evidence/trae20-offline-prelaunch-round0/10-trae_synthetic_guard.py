"""Owner-controlled offline synthetic guard; never forwards to a provider.

No entry in this module launches Trae. Runtime config/retry support is unproved.
"""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import socket
import stat
import time
import threading
import uuid

BODY_LIMIT = 1024 * 1024
RESPONSE_LIMIT = 64 * 1024
CASES = (('http_503', 1, 'failed'), ('truncated', 1, 'failed'), ('success', 2, 'completed'))


class Journal:
    """Bounded owner-only append journal; reservations survive process failure."""
    def __init__(self, path):
        self.path = Path(path)

    def transact(self, action):
        directory = self.path.parent.stat()
        if directory.st_uid != os.getuid() or directory.st_mode & 0o077:
            raise ValueError('journal directory must be owner-only')
        fd = os.open(self.path, os.O_RDWR | os.O_APPEND | os.O_CREAT | os.O_NOFOLLOW, 0o600)
        try:
            fcntl.flock(fd, fcntl.LOCK_EX)
            info = os.fstat(fd)
            if not stat.S_ISREG(info.st_mode) or info.st_uid != os.getuid() or info.st_mode & 0o077 or info.st_nlink != 1:
                raise ValueError('unsafe journal')
            if info.st_size > 65536:
                raise ValueError('journal capacity exhausted')
            raw = os.read(fd, 65537)
            records = []
            previous = ''
            for line in raw.splitlines(keepends=True):
                if not line.endswith(b'\n'):
                    raise ValueError('incomplete journal record; quarantine')
                record = json.loads(line)
                checksum = record.pop('sha256')
                if record['sequence'] != len(records) or record['previous'] != previous or hashlib.sha256(json.dumps(record, sort_keys=True).encode()).hexdigest() != checksum:
                    raise ValueError('invalid journal chain; quarantine')
                record['sha256'] = checksum
                previous = checksum
                records.append(record)
            result, additions = action(records)
            encoded = b''
            for addition in additions:
                record = dict(addition, sequence=len(records), previous=previous)
                previous = hashlib.sha256(json.dumps(record, sort_keys=True).encode()).hexdigest()
                record['sha256'] = previous
                encoded += (json.dumps(record, sort_keys=True) + '\n').encode()
                records.append(record)
            if len(encoded) + info.st_size > 65536:
                raise ValueError('journal capacity exhausted')
            if encoded:
                if os.write(fd, encoded) != len(encoded):
                    raise OSError('short journal write; quarantine')
                os.fsync(fd)
                directory_fd = os.open(self.path.parent, os.O_RDONLY | os.O_DIRECTORY)
                try:
                    os.fsync(directory_fd)
                finally:
                    os.close(directory_fd)
            return result
        finally:
            os.close(fd)


class AttemptGuard:
    """One round, two namespace incarnations, three fixed sequential cases.

Only the trusted owner calls start/arm/terminal. HTTP cannot supply attribution.
Stop callback requests immediate physical stop for violations/unknowns. Third
reservation revokes admission, but physical stop waits for terminal/deadline.
"""
    def __init__(self, path, stop, turn_seconds=120, round_seconds=600):
        if not 0 < turn_seconds <= 120 or not 0 < round_seconds <= 600:
            raise ValueError('deadline widening refused')
        self.journal = Journal(path)
        self.stop = stop
        self.turn_seconds = turn_seconds
        self.round_seconds = round_seconds

    def _transaction(self, action):
        try:
            def checked(records):
                for record in records:
                    if record['kind'] in ('incarnation', 'incarnation_reserved') and (record['turn_seconds'] != self.turn_seconds or record['round_seconds'] != self.round_seconds):
                        raise ValueError('durable deadline policy mismatch')
                return action(records)
            return self.journal.transact(checked)
        except (OSError, ValueError, KeyError, TypeError):
            try:
                self.journal.transact(lambda records: (None, [{'kind': 'halt', 'reason': 'owner_operation_or_journal_uncertain'}]))
            except (OSError, ValueError, KeyError, TypeError):
                pass
            self.stop('journal_uncertain')
            raise

    @staticmethod
    def _state(records):
        attempts = [r for r in records if r['kind'] == 'attempt']
        reservations = [r for r in records if r['kind'] == 'reservation']
        arms = [r for r in records if r['kind'] == 'arm']
        terminals = [r for r in records if r['kind'] == 'terminal']
        bindings = [r for r in records if r['kind'] == 'native_turn']
        active = arms[-1] if len(arms)>len(terminals) else None
        binding = next((r for r in bindings if active and r['operation']==active['operation']), None)
        incarnations = [r for r in records if r['kind'] == 'incarnation']
        slots = [r for r in records if r['kind'] == 'incarnation_reserved']
        halted = any(r['kind'] == 'halt' for r in records)
        return {'incoming_attempts': len(attempts), 'admitted': len(reservations),
                'reserved_output_tokens': sum(r['output_cap'] for r in reservations),
                'envelope_violation': len(attempts) > 3,
                'admission_revoked': len(attempts) >= 3 or halted,
                'halted': halted, 'active': active, 'native_turn': binding['native_turn'] if binding else None,
                'terminals': len(terminals), 'incarnations': len(incarnations),
                'incarnation_slots': len(slots), 'unbound_slot': slots[-1] if len(slots)>len(incarnations) else None,
                'namespace': incarnations[-1]['namespace'] if incarnations else None,
                'round_deadline': slots[0]['round_deadline'] if slots else None,
                'transport_uncertain': any(r['kind'] == 'transport_uncertain' for r in records),
                'connections': sum(r['kind'] == 'connection' for r in records)}

    def snapshot(self):
        return self._transaction(lambda records: (self._state(records), []))

    def reserve_incarnation(self):
        """Owner must durably reserve BEFORE any runtime process creation."""
        self.check_deadlines()
        def action(records):
            state = self._state(records)
            if state['active'] or state['admission_revoked'] or state['incarnation_slots'] >= 2 or state['unbound_slot']:
                raise ValueError('incarnation reservation refused')
            n = state['incarnation_slots'] + 1
            if (n == 1 and state['terminals'] != 0) or (n == 2 and state['terminals'] != 2):
                raise ValueError('case-to-incarnation mapping refused')
            record = {'kind': 'incarnation_reserved', 'incarnation': n, 'launch_operation': str(uuid.uuid4()),
                      'round_deadline': state['round_deadline'] or time.monotonic()+self.round_seconds,
                      'turn_seconds': self.turn_seconds, 'round_seconds': self.round_seconds,
                      'boot_id': Path('/proc/sys/kernel/random/boot_id').read_text().strip()}
            return record, [record]
        return self._transaction(action)

    def bind_incarnation(self, reservation, namespace):
        def action(records):
            state = self._state(records)
            pending = state['unbound_slot']
            if state['halted'] or not pending or pending['launch_operation'] != reservation['launch_operation'] or not isinstance(namespace, str) or not namespace or any(r.get('namespace') == namespace for r in records):
                raise ValueError('incarnation binding refused')
            record = {key: pending[key] for key in ('incarnation','launch_operation','round_deadline','turn_seconds','round_seconds','boot_id')}
            record.update(kind='incarnation',namespace=namespace)
            return pending['incarnation'], [record]
        return self._transaction(action)

    def start(self, namespace):
        """Offline convenience; runtime owner separates reserve/spawn/bind."""
        return self.bind_incarnation(self.reserve_incarnation(), namespace)

    def arm(self, turn):
        self.check_deadlines()
        def action(records):
            state = self._state(records)
            if not isinstance(turn, str) or not turn or state['active'] or state['admission_revoked'] or state['terminals'] >= 3 or not state['incarnations'] or any(r.get('turn') == turn for r in records):
                raise ValueError('turn admission refused')
            case, incarnation, _ = CASES[state['terminals']]
            if incarnation != state['incarnations']:
                raise ValueError('wrong case incarnation')
            record = {'kind': 'arm', 'operation': str(uuid.uuid4()), 'turn': turn, 'case': case,
                      'incarnation': incarnation, 'namespace': state['namespace'], 'turn_deadline': time.monotonic()+self.turn_seconds}
            return record, [record]
        return self._transaction(action)

    def bind_native_turn(self, operation, native_turn):
        """Trusted owner appends native ID from exact turn/start response, once."""
        def action(records):
            state = self._state(records)
            active = state['active']
            if state['halted'] or not active or active['operation'] != operation or state['native_turn'] or not isinstance(native_turn, str) or not native_turn or any(r.get('native_turn') == native_turn for r in records):
                raise ValueError('native turn binding refused')
            return None, [{'kind': 'native_turn', 'operation': operation, 'owner_turn': active['turn'],
                           'native_turn': native_turn, 'namespace': active['namespace'],
                           'incarnation': active['incarnation']}]
        return self._transaction(action)

    def connection(self):
        return self._transaction(lambda records: (None, [{'kind': 'connection'}]))

    def check_deadlines(self):
        def action(records):
            state = self._state(records)
            boot = Path('/proc/sys/kernel/random/boot_id').read_text().strip()
            invalid_boot = any(r['kind'] in ('incarnation', 'incarnation_reserved') and r['boot_id'] != boot for r in records)
            expired = invalid_boot or bool(state['round_deadline'] and time.monotonic() >= state['round_deadline']) or bool(state['active'] and time.monotonic() >= state['active']['turn_deadline'])
            return expired, [{'kind': 'halt', 'reason': 'deadline_or_boot_uncertain'}] if expired and not state['halted'] else []
        expired = self._transaction(action)
        if expired:
            self.stop('deadline_or_boot_uncertain')
        return expired

    def begin(self):
        """Called immediately upon first request byte, before parsing any field."""
        def action(records):
            state = self._state(records)
            active = state['active']
            duplicate = bool(active and any(r['kind'] == 'attempt' and r.get('operation') == active['operation'] for r in records))
            allowed = bool(active and not state['admission_revoked'] and not duplicate)
            record = {'kind': 'attempt', 'attempt': str(uuid.uuid4()), 'ordinal': state['incoming_attempts'] + 1,
                      'eligible': allowed, 'operation': active['operation'] if active else None,
                      'turn': active['turn'] if active else None, 'native_turn': state['native_turn'],
                      'incarnation': active['incarnation'] if active else state['incarnations'],
                      'namespace': active['namespace'] if active else state['namespace']}
            additions = [record]
            if not allowed:
                additions.append({'kind': 'halt', 'reason': 'unexpected_or_extra_attempt'})
            return record, additions
        self.check_deadlines()
        result = self._transaction(action)
        if not result['eligible']:
            self.stop('envelope_violation' if result['ordinal'] > 3 else 'unexpected_or_extra_attempt')
        return result

    def reject(self, attempt, reason):
        def action(records):
            if not any(r['kind'] == 'attempt' and r['attempt'] == attempt['attempt'] for r in records):
                raise ValueError('unknown attempt')
            return {'admitted': False, 'operation': attempt['operation']}, [{'kind': 'halt', 'reason': reason, 'attempt': attempt['attempt']}]
        result = self._transaction(action)
        self.stop(reason)
        return result

    def reserve(self, attempt, method, route, body):
        try:
            obj = json.loads(body) if len(body) <= BODY_LIMIT else None
        except (ValueError, UnicodeError):
            obj = None
        allowed = (attempt['eligible'] and method == 'POST' and route == '/v1/responses'
                   and isinstance(obj, dict) and obj.get('model') == 'gpt-5.5' and obj.get('stream') is True
                   and type(obj.get('max_output_tokens')) is int and 0 < obj['max_output_tokens'] <= 32)
        if not allowed:
            return self.reject(attempt, 'request_rejected')
        def action(records):
            state = self._state(records)
            if state['halted'] or not state['native_turn'] or not any(r['kind'] == 'attempt' and r == dict(attempt, sequence=r['sequence'], previous=r['previous'], sha256=r['sha256']) for r in records) or any(r['kind'] == 'reservation' and r['attempt'] == attempt['attempt'] for r in records):
                raise ValueError('reservation refused')
            record = {'kind': 'reservation', 'attempt': attempt['attempt'], 'operation': attempt['operation'],
                      'native_turn': state['native_turn'], 'output_cap': obj['max_output_tokens'], 'body_bytes': len(body),
                      'body_sha256': hashlib.sha256(body).hexdigest(), 'model': obj['model'],
                      'stream': True, 'method': method, 'route': route,
                      'admission_revoked_before_response': state['incoming_attempts'] >= 3}
            return {'admitted': True, 'operation': attempt['operation'], 'case': state['active']['case']}, [record]
        return self._transaction(action)

    def admit(self, method, route, body):
        """Offline buffered convenience seam; production transport uses begin first."""
        return self.reserve(self.begin(), method, route, body)

    def send_response(self, connection, operation):
        """Reserve immutable response intent, write bounded bytes, record publication."""
        response_id = 'resp_' + operation
        item_id = 'msg_' + operation
        def action(records):
            state = self._state(records)
            active = state['active']
            if not active or active['operation'] != operation or state['halted'] or not any(r['kind'] == 'reservation' and r['operation'] == operation for r in records) or any(r['kind'] == 'response_intent' and r['operation'] == operation for r in records):
                raise ValueError('response publication refused')
            case = active['case']
            if case == 'http_503':
                status = '503 Service Unavailable'
                payload = b'{"error":{"message":"owned synthetic retryable failure","type":"server_error"}}'
                mime = 'application/json'
            else:
                status = '200 OK'
                payload = synthetic_sse(response_id, item_id, truncate=case == 'truncated')
                mime = 'text/event-stream'
            wire = ('HTTP/1.1 ' + status + '\r\nContent-Type: ' + mime + '\r\nContent-Length: ' + str(len(payload)) + '\r\nConnection: close\r\n\r\n').encode() + payload
            if len(wire) > RESPONSE_LIMIT:
                raise ValueError('response capacity exceeded')
            record = {'kind': 'response_intent', 'operation': operation, 'case': case,
                      'bytes': len(wire), 'sha256_wire': hashlib.sha256(wire).hexdigest(),
                      'response_id': response_id, 'item_id': item_id,
                      'truncate_boundary': 'after response.output_text.delta' if case == 'truncated' else None}
            return wire, [record]
        wire = self._transaction(action)
        try:
            connection.settimeout(1)
            connection.sendall(wire)
            connection.shutdown(socket.SHUT_WR)
        except OSError:
            self.expire(operation)
            raise
        self._transaction(lambda records: (None, [{'kind': 'published', 'operation': operation}]))
        return len(wire)

    def terminal(self, operation, turn, outcome, content=None):
        def action(records):
            state = self._state(records)
            active = state['active']
            if not active or active['operation'] != operation or state['native_turn'] != turn:
                raise ValueError('terminal attribution refused')
            attempted = [r for r in records if r['kind'] == 'attempt' and r['operation'] == operation]
            reserved = [r for r in records if r['kind'] == 'reservation' and r['operation'] == operation]
            expected = CASES[state['terminals']][2]
            published = [r for r in records if r['kind'] == 'published' and r['operation'] == operation]
            known = not state['halted'] and len(attempted) == len(reserved) == len(published) == 1 and outcome == expected and (outcome != 'completed' or content == 'preflight-ok')
            record = {'kind': 'terminal', 'operation': operation, 'turn': turn, 'outcome': outcome, 'known': known}
            additions = [record]
            if not known:
                additions.append({'kind': 'halt', 'reason': 'unknown_terminal'})
            return known, additions
        self.check_deadlines()
        known = self._transaction(action)
        if not known or self.snapshot()['admission_revoked']:
            self.stop('round_complete' if known else 'unknown_terminal')
        return known

    def halt(self, reason):
        self._transaction(lambda records: (None, [{'kind': 'halt', 'reason': reason}]))
        self.stop(reason)

    def expire(self, operation):
        self._transaction(lambda records: (None, [{'kind': 'halt', 'reason': 'terminal_deadline', 'operation': operation}]))
        self.stop('terminal_deadline')


class RoundWatchdog:
    """Trusted owner timer; physical stop proof still comes from supervisor."""
    def __init__(self, guard):
        self.guard = guard
        self.closed = threading.Event()
        self.thread = threading.Thread(target=self._watch, name='synthetic-owner-deadline', daemon=True)

    def _watch(self):
        while not self.closed.wait(.01):
            try:
                if self.guard.check_deadlines() or self.guard.snapshot()['halted']:
                    return
            except (OSError, ValueError, KeyError, TypeError):
                return  # Guard already requested stop and retained uncertainty.

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *unused):
        self.closed.set()
        self.thread.join(timeout=1)
        if self.thread.is_alive():
            self.guard.stop('watchdog_terminal_unknown')
            raise RuntimeError('owner watchdog failed to terminate')


def receive_http(connection, guard, timeout=1.0, record_connection=True):
    """One request per connection, bounded reads; connection always closed by caller.

First byte is the counting point. A connection with zero bytes is separately
recorded. Pipelined bytes are rejected (no keepalive/discovery exemptions).
"""
    if record_connection:
        guard.connection()
    deadline = time.monotonic() + timeout
    def read(size):
        connection.settimeout(max(.001, deadline - time.monotonic()))
        if time.monotonic() >= deadline:
            raise TimeoutError('read deadline')
        return connection.recv(size)
    try:
        first = read(1)
    except TimeoutError:
        return None
    if not first:
        return None
    attempt = guard.begin()
    try:
        headers = bytearray(first)
        while not headers.endswith(b'\r\n\r\n'):
            if len(headers) >= 8192:
                raise ValueError('headers too large')
            byte = read(1)
            if not byte:
                raise ValueError('short headers')
            headers += byte
        lines = bytes(headers[:-4]).decode('ascii').split('\r\n')
        request = lines[0].split(' ')
        if len(request) != 3 or request[2] != 'HTTP/1.1':
            raise ValueError('request line')
        lengths = []
        for line in lines[1:]:
            name, separator, value = line.partition(':')
            if not separator or not name or name.strip() != name or any(c.isspace() for c in name):
                raise ValueError('header syntax')
            if name.lower() == 'transfer-encoding':
                raise ValueError('chunked/transfer encoding refused')
            if name.lower() == 'content-length':
                lengths.append(value.strip())
        if len(lengths) != 1 or not lengths[0].isascii() or not lengths[0].isdigit():
            raise ValueError('ambiguous length')
        length = int(lengths[0])
        if length > BODY_LIMIT:
            raise ValueError('body too large')
        body = bytearray()
        while len(body) < length:
            chunk = read(min(65536, length - len(body)))
            if not chunk:
                raise ValueError('body length mismatch')
            body += chunk
        return guard.reserve(attempt, request[0], request[1], bytes(body))
    except (ValueError, UnicodeError, TimeoutError, OSError) as error:
        result = guard.reject(attempt, type(error).__name__ + ': ' + str(error))
        guard._transaction(lambda records: (None, [{'kind': 'transport_uncertain', 'attempt': attempt['attempt'], 'reason': 'ambiguous_or_incomplete_framing'}]))
        result['framing_uncertain'] = True
        return result


def serve_connection(connection, guard, timeout=1.0):
    """Consume complete pipelined requests without concealing buffered retries.

The entire connection has a finite deadline. Once framing becomes ambiguous,
record the malformed request and halt; remaining unparsed bytes are uncertainty,
not fabricated request counts. No persistent connections are advertised.
"""
    guard.connection()
    deadline = time.monotonic() + timeout
    decisions = []
    try:
        while time.monotonic() < deadline:
            decision = receive_http(connection, guard, timeout=max(.001, deadline-time.monotonic()), record_connection=False)
            if decision is None:
                break
            decisions.append(decision)
            if decision['admitted']:
                guard.send_response(connection, decision['operation'])
            if decision.get('framing_uncertain'):
                break
    finally:
        connection.close()
    return decisions


def synthetic_sse(response_id, item_id, truncate=False):
    if not all(isinstance(value, str) and 0 < len(value) <= 128 for value in (response_id, item_id)):
        raise ValueError('bounded response identity required')
    item = {'id': item_id, 'type': 'message', 'role': 'assistant', 'status': 'completed',
            'content': [{'type': 'output_text', 'text': 'preflight-ok', 'annotations': []}]}
    response = {'id': response_id, 'object': 'response', 'created_at': 0, 'status': 'completed', 'model': 'gpt-5.5',
                'output': [item], 'usage': {'input_tokens': 1, 'output_tokens': 1, 'total_tokens': 2}, 'error': None, 'incomplete_details': None}
    events = [{'type': 'response.created', 'response': dict(response, status='in_progress', output=[])},
              {'type': 'response.in_progress', 'response': dict(response, status='in_progress', output=[])},
              {'type': 'response.output_item.added', 'output_index': 0, 'item': dict(item, status='in_progress', content=[])},
              {'type': 'response.content_part.added', 'output_index': 0, 'item_id': item_id, 'content_index': 0,
               'part': {'type': 'output_text', 'text': '', 'annotations': []}},
              {'type': 'response.output_text.delta', 'output_index': 0, 'item_id': item_id, 'content_index': 0, 'delta': 'preflight-ok'}]
    if not truncate:
        events += [{'type': 'response.output_text.done', 'output_index': 0, 'item_id': item_id, 'content_index': 0, 'text': 'preflight-ok'},
                   {'type': 'response.content_part.done', 'output_index': 0, 'item_id': item_id, 'content_index': 0, 'part': item['content'][0]},
                   {'type': 'response.output_item.done', 'output_index': 0, 'item': item},
                   {'type': 'response.completed', 'response': response}]
    encoded = b''.join(('event: ' + e['type'] + '\ndata: ' + json.dumps(dict(e, sequence_number=n)) + '\n\n').encode() for n, e in enumerate(events))
    if len(encoded) > RESPONSE_LIMIT:
        raise ValueError('response capacity exceeded')
    return encoded
