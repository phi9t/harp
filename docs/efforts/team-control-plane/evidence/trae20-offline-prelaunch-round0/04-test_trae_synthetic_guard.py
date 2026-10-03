#!/usr/bin/env python3
"""Offline owner/guard seam only. Frozen predecessor retains its six-test proof."""
import concurrent.futures
import json
from pathlib import Path
import socket
import tempfile
import unittest
from trae_synthetic_guard import AttemptGuard, synthetic_sse


def request(**changes):
    obj={'model':'gpt-5.5','stream':True,'max_output_tokens':32,'input':'owned fixture'}
    obj.update(changes)
    return json.dumps(obj).encode()


def publish(guard,operation):
    client,server=socket.socketpair()
    try:
        guard.send_response(server,operation)
        data=b''
        while True:
            chunk=client.recv(65536)
            if not chunk:return data
            data+=chunk
    finally:client.close();server.close()


def advance(guard,turn):
    arm=owned_arm(guard,turn)
    decision=guard.admit('POST','/v1/responses',request())
    assert decision['admitted']
    wire=publish(guard,arm['operation'])
    assert guard.terminal(arm['operation'],turn,'completed' if arm['case']=='success' else 'failed',content='preflight-ok' if arm['case']=='success' else None)
    return wire



def owned_arm(guard,turn):
    arm=guard.arm(turn)
    guard.bind_native_turn(arm['operation'],turn)
    return arm

class GuardBehavior(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.path=Path(self.tmp.name)/'attempts.jsonl';self.stops=[]
        self.guard=AttemptGuard(self.path,self.stops.append);self.guard.start('namespace-one')
    def tearDown(self):self.tmp.cleanup()

    def test_third_request_revokes_admission_before_response_then_waits_for_terminal(self):
        advance(self.guard,'turn-one');advance(self.guard,'turn-two')
        self.guard.start('namespace-two');arm=owned_arm(self.guard,'turn-three')
        self.assertTrue(self.guard.admit('POST','/v1/responses',request())['admitted'])
        self.assertTrue(self.guard.snapshot()['admission_revoked']);self.assertEqual(self.stops,[])
        wire=publish(self.guard,arm['operation']);self.assertIn(b'response.completed',wire)
        self.assertEqual(self.stops,[])
        self.assertTrue(self.guard.terminal(arm['operation'],'turn-three','completed','preflight-ok'))
        self.assertEqual(self.stops,['round_complete'])
        self.assertFalse(self.guard.admit('POST','/v1/responses',request())['admitted'])
        self.assertEqual(self.guard.snapshot()['incoming_attempts'],4)
        self.assertEqual(self.guard.snapshot()['admitted'],3)
        self.assertTrue(self.guard.snapshot()['envelope_violation'])

    def test_concurrent_last_slot_retry_is_counted_and_never_forwarded(self):
        advance(self.guard,'turn-one');advance(self.guard,'turn-two');self.guard.start('namespace-two');owned_arm(self.guard,'turn-three')
        with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
            attempts=list(pool.map(lambda _:AttemptGuard(self.path,self.stops.append).begin(),range(3)))
        self.assertEqual(sum(a['eligible'] for a in attempts),1)
        self.assertEqual(self.guard.snapshot()['incoming_attempts'],5)
        self.assertEqual(self.guard.snapshot()['admitted'],2)
        self.assertEqual(len({a['attempt'] for a in attempts}),3)
        self.assertTrue(self.guard.snapshot()['halted'])

    def test_restart_preserves_uncertain_reservation_and_refuses_retry(self):
        owned_arm(self.guard,'turn-one');self.guard.admit('POST','/v1/responses',request())
        later=AttemptGuard(self.path,self.stops.append)
        self.assertFalse(later.admit('POST','/v1/responses',request())['admitted'])
        self.assertEqual(later.snapshot()['incoming_attempts'],2)
        self.assertEqual(later.snapshot()['admitted'],1)
        self.assertEqual(later.snapshot()['reserved_output_tokens'],32)

    def test_unbounded_model_route_and_body_requests_count_then_stop(self):
        bad=[('GET','/v1/models',b''),('POST','/v1/responses?route=alternate',request()),('POST','/v1/responses',request(model='other')),('POST','/v1/responses',request(max_output_tokens=None)),('POST','/v1/responses',request(max_output_tokens=True)),('POST','/v1/responses',request(max_output_tokens=33)),('POST','/v1/responses',b'x'*(1024*1024+1))]
        for i,(method,path,body) in enumerate(bad):
            with self.subTest(i=i):
                g=AttemptGuard(Path(self.tmp.name)/f'bad-{i}.jsonl',self.stops.append);g.start('owned');owned_arm(g,'turn')
                self.assertFalse(g.admit(method,path,body)['admitted'])
                self.assertEqual(g.snapshot()['incoming_attempts'],1)
                self.assertEqual(g.snapshot()['reserved_output_tokens'],0)
                self.assertTrue(g.snapshot()['halted'])

    def test_append_only_terminal_does_not_accept_reused_operation(self):
        owned_arm(self.guard,'turn-one');decision=self.guard.admit('POST','/v1/responses',request());publish(self.guard,decision['operation'])
        self.guard.terminal(decision['operation'],'turn-one','failed');before=self.path.read_bytes()
        with self.assertRaises(ValueError):self.guard.terminal(decision['operation'],'turn-one','failed')
        self.assertTrue(self.path.read_bytes().startswith(before))

    def test_stream_is_coherent_bounded_and_truncation_has_no_completion(self):
        stream=synthetic_sse('owned-response','owned-item');self.assertLess(len(stream),64*1024)
        events=[json.loads(p[6:]) for p in stream.decode().splitlines() if p.startswith('data: ')]
        self.assertEqual([e['sequence_number'] for e in events],list(range(len(events))))
        self.assertEqual(events[-1]['type'],'response.completed')
        final=events[-1]['response'];self.assertEqual(final['id'],'owned-response')
        self.assertEqual(final['output'][0]['id'],'owned-item');self.assertEqual(final['output'][0]['content'][0]['text'],'preflight-ok')
        truncated=synthetic_sse('owned-response','owned-item',truncate=True)
        self.assertNotIn(b'response.completed',truncated);self.assertIn(b'response.output_text.delta',truncated)
        with self.assertRaises(ValueError):synthetic_sse('x'*65536,'item')

if __name__=='__main__':unittest.main()
