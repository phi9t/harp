"""Public transport seam: real owned socketpair, never Trae/external endpoints."""
import json
from pathlib import Path
import socket
import tempfile
import threading
import time
import unittest
from trae_synthetic_guard import AttemptGuard, receive_http, serve_connection


def owned_arm(guard,turn):
    arm=guard.arm(turn)
    guard.bind_native_turn(arm['operation'],turn)
    return arm

class TransportBehavior(unittest.TestCase):
    def test_partial_request_is_counted_before_waiting_for_headers(self):
        with tempfile.TemporaryDirectory() as tmp:
            guard=AttemptGuard(Path(tmp)/'ledger',lambda _:None)
            guard.start('owned-namespace-1');owned_arm(guard,'owned-turn-1')
            client,server=socket.socketpair()
            thread=threading.Thread(target=receive_http,args=(server,guard),daemon=True)
            thread.start()
            try:
                client.sendall(b'P')
                end=time.monotonic()+1
                while time.monotonic()<end and guard.snapshot()['incoming_attempts']==0:
                    time.sleep(.01)
                self.assertEqual(guard.snapshot()['incoming_attempts'],1)
            finally:
                client.close();thread.join(2);server.close()

    def exercise(self, wire, expected_count=1):
        with tempfile.TemporaryDirectory() as tmp:
            stops=[];guard=AttemptGuard(Path(tmp)/'ledger',stops.append)
            guard.start('owned');owned_arm(guard,'turn')
            client,server=socket.socketpair()
            try:
                client.sendall(wire);client.shutdown(socket.SHUT_WR)
                results=serve_connection(server,guard,timeout=.2) if expected_count>1 else [receive_http(server,guard,timeout=.2)]
                result=results[-1]
                self.assertFalse(result['admitted']);self.assertEqual(guard.snapshot()['incoming_attempts'],expected_count)
                self.assertEqual(guard.snapshot()['admitted'],0);self.assertTrue(guard.snapshot()['halted'])
                self.assertTrue(stops)
            finally:client.close();server.close()

    def test_rejected_framing_is_bounded_and_counted(self):
        examples=[b'BROKEN\r\n\r\n',b'POST /v1/responses HTTP/1.1\r\n\r\n',
          b'POST /v1/responses HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n',
          b'POST /v1/responses HTTP/1.1\r\nContent-Length: 2\r\nContent-Length: 2\r\n\r\n{}',
          b'POST /v1/responses HTTP/1.1\r\nContent-Length: 2,3\r\n\r\n{}',
          b'POST /v1/responses HTTP/1.1\r\nContent-Length: -1\r\n\r\n',
          b'POST /v1/responses HTTP/1.1\r\nContent-Length: 1048577\r\n\r\n',
          b'POST /v1/responses HTTP/1.1\r\nContent-Length: 3\r\n\r\n{}',
          b'POST /v1/responses HTTP/1.1\r\nX: '+b'x'*8192+b'\r\n\r\n']
        for wire in examples:
            with self.subTest(wire=wire[:100]):self.exercise(wire)

    def test_trailing_body_mismatch_is_not_an_uncounted_request(self):
        self.exercise(b'POST /v1/responses HTTP/1.1\r\nContent-Length: 2\r\n\r\n{}GET /v1/models HTTP/1.1\r\n\r\n',expected_count=2)

    def test_stalled_body_deadline_preserves_attempt(self):
        with tempfile.TemporaryDirectory() as tmp:
            guard=AttemptGuard(Path(tmp)/'ledger',lambda _:None);guard.start('owned');owned_arm(guard,'turn')
            client,server=socket.socketpair()
            try:
                client.sendall(b'POST /v1/responses HTTP/1.1\r\nContent-Length: 100\r\n\r\nx')
                start=time.monotonic();result=receive_http(server,guard,timeout=.05)
                self.assertFalse(result['admitted']);self.assertLess(time.monotonic()-start,.5)
                self.assertEqual(guard.snapshot()['incoming_attempts'],1)
            finally:client.close();server.close()

    def test_real_transport_three_cases_exact_response_and_terminal_order(self):
        from test_trae_synthetic_guard import request
        with tempfile.TemporaryDirectory() as tmp:
            stops=[];guard=AttemptGuard(Path(tmp)/'ledger',stops.append);guard.start('ns-one')
            for number in range(3):
                if number==2:guard.start('ns-two')
                arm=owned_arm(guard,'turn-'+str(number));client,server=socket.socketpair()
                body=request();wire=b'POST /v1/responses HTTP/1.1\r\nContent-Length: '+str(len(body)).encode()+b'\r\n\r\n'+body
                try:
                    client.sendall(wire);decision=receive_http(server,guard)
                    self.assertTrue(decision['admitted']);guard.send_response(server,decision['operation'])
                    response=b''
                    while True:
                        chunk=client.recv(65536)
                        if not chunk:break
                        response+=chunk
                    self.assertLessEqual(len(response),65536)
                    if number==0:self.assertTrue(response.startswith(b'HTTP/1.1 503'))
                    elif number==1:self.assertIn(b'response.output_text.delta',response);self.assertNotIn(b'response.completed',response)
                    else:self.assertIn(b'response.completed',response);self.assertIn(b'preflight-ok',response);self.assertEqual(stops,[])
                    self.assertTrue(guard.terminal(arm['operation'],arm['turn'],'completed' if number==2 else 'failed','preflight-ok' if number==2 else None))
                finally:client.close();server.close()
            self.assertEqual(stops,['round_complete']);self.assertEqual(guard.snapshot()['incoming_attempts'],3)

    def test_four_complete_pipelined_requests_are_all_observed_not_one_extra(self):
        from test_trae_synthetic_guard import request
        with tempfile.TemporaryDirectory() as tmp:
            guard=AttemptGuard(Path(tmp)/'ledger',lambda _:None);guard.start('owned');owned_arm(guard,'turn')
            client,server=socket.socketpair();body=request()
            wire=b'POST /v1/responses HTTP/1.1\r\nContent-Length: '+str(len(body)).encode()+b'\r\n\r\n'+body
            try:
                client.sendall(wire*4);client.shutdown(socket.SHUT_WR)
                results=serve_connection(server,guard)
                self.assertEqual(len(results),4)
                self.assertEqual(guard.snapshot()['incoming_attempts'],4)
                self.assertEqual(guard.snapshot()['admitted'],1)
                self.assertTrue(guard.snapshot()['envelope_violation'])
            finally:client.close();server.close()

    def test_ambiguous_framing_never_claims_complete_request_count_coverage(self):
        with tempfile.TemporaryDirectory() as tmp:
            guard=AttemptGuard(Path(tmp)/'ledger',lambda _:None);guard.start('owned');owned_arm(guard,'turn')
            client,server=socket.socketpair()
            try:
                client.sendall(b'POST /v1/responses HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n');client.shutdown(socket.SHUT_WR)
                self.assertFalse(receive_http(server,guard)['admitted'])
                self.assertTrue(guard.snapshot()['transport_uncertain'])
                self.assertTrue(guard.snapshot()['halted'])
            finally:client.close();server.close()
