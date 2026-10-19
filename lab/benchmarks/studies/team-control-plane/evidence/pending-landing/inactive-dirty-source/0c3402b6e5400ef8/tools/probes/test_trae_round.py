import concurrent.futures
import json
import os
from pathlib import Path
import tempfile
import unittest
from trae_synthetic_guard import AttemptGuard

def body():
    return json.dumps({'model':'gpt-5.5','stream':True,'max_output_tokens':32,'input':'owned fixture'}).encode()


def owned_arm(guard,turn):
    arm=guard.arm(turn)
    guard.bind_native_turn(arm['operation'],turn)
    return arm

class RoundBehavior(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.path=Path(self.tmp.name)/'ledger';self.stops=[]
        self.guard=AttemptGuard(self.path,self.stops.append)
        self.guard.start('owned-ns-1');self.arm=owned_arm(self.guard,'turn-1')
    def tearDown(self):self.tmp.cleanup()
    def test_terminal_without_published_case_is_unknown(self):
        self.guard.admit('POST','/v1/responses',body())
        self.assertFalse(self.guard.terminal(self.arm['operation'],'turn-1','failed'))
        self.assertTrue(self.guard.snapshot()['halted'])
    def test_foreign_terminal_and_reused_turn_do_not_change_provenance(self):
        before=self.path.read_bytes()
        with self.assertRaises(ValueError):self.guard.terminal(self.arm['operation'],'foreign-turn','failed')
        self.assertTrue(self.path.read_bytes().startswith(before))
        with self.assertRaises(ValueError):owned_arm(self.guard,'second-active')
        with self.assertRaises(ValueError):self.guard.start('ns-two')
        self.assertEqual(self.guard.snapshot()['incarnations'],1)

    def test_real_process_crash_does_not_refund_reservation(self):
        import subprocess,sys
        # A fresh process opens this owner journal and dies after durable reserve.
        code="import os,sys;from trae_synthetic_guard import AttemptGuard;g=AttemptGuard(sys.argv[1],lambda _:None);g.admit('POST','/v1/responses',sys.argv[2].encode());os._exit(17)"
        result=subprocess.run([sys.executable,'-c',code,str(self.path),body().decode()],cwd=Path(__file__).parent,capture_output=True,timeout=2)
        self.assertEqual(result.returncode,17,result.stderr)
        reopened=AttemptGuard(self.path,self.stops.append)
        self.assertEqual(reopened.snapshot()['admitted'],1);self.assertEqual(reopened.snapshot()['reserved_output_tokens'],32)
        self.assertFalse(reopened.admit('POST','/v1/responses',body())['admitted'])

    def test_corrupt_or_world_readable_journal_is_quarantined_without_reset(self):
        self.guard.admit('POST','/v1/responses',body())
        before=self.path.read_bytes();self.path.chmod(0o644)
        with self.assertRaises(ValueError):self.guard.snapshot()
        self.assertTrue(self.path.read_bytes().startswith(before));self.assertIn('journal_uncertain',self.stops)
        self.path.chmod(0o600)
        with self.path.open('ab') as file:file.write(b'{incomplete')
        corrupt=self.path.read_bytes()
        with self.assertRaises(ValueError):self.guard.snapshot()
        self.assertEqual(self.path.read_bytes(),corrupt)

    def test_symlink_journal_is_refused_without_touching_target(self):
        target=Path(self.tmp.name)/'target';target.write_text('keep')
        linked=Path(self.tmp.name)/'linked';linked.symlink_to(target)
        with self.assertRaises(OSError):AttemptGuard(linked,self.stops.append).snapshot()
        self.assertEqual(target.read_text(),'keep')

    def test_unknown_terminal_ends_round_and_finite_reservation_remains(self):
        from test_trae_synthetic_guard import publish
        self.guard.admit('POST','/v1/responses',body());publish(self.guard,self.arm['operation'])
        self.assertFalse(self.guard.terminal(self.arm['operation'],'turn-1','unknown'))
        self.assertTrue(self.guard.snapshot()['halted']);self.assertEqual(self.guard.snapshot()['reserved_output_tokens'],32)
        with self.assertRaises(ValueError):owned_arm(self.guard,'replacement')

    def test_explicit_terminal_deadline_halts_and_retains_reservation(self):
        self.guard.admit('POST','/v1/responses',body());self.guard.expire(self.arm['operation'])
        self.assertTrue(self.guard.snapshot()['halted']);self.assertEqual(self.guard.snapshot()['admitted'],1)
        self.assertIn('terminal_deadline',self.stops)
    def test_real_owner_watchdog_expires_without_http_or_runtime_cooperation(self):
        import threading
        from trae_synthetic_guard import RoundWatchdog
        stopped=threading.Event();path=Path(self.tmp.name)/'short-deadline'
        guard=AttemptGuard(path,lambda reason:stopped.set(),turn_seconds=.05)
        guard.start('short-owned-ns');owned_arm(guard,'short-turn')
        with RoundWatchdog(guard):self.assertTrue(stopped.wait(.5))
        self.assertTrue(guard.snapshot()['halted'])
        with self.assertRaises(ValueError):AttemptGuard(path,lambda _:None).snapshot()

    def test_deadline_cannot_be_widened_at_constructor(self):
        with self.assertRaises(ValueError):AttemptGuard(Path(self.tmp.name)/'wide',lambda _:None,turn_seconds=121)
        with self.assertRaises(ValueError):AttemptGuard(Path(self.tmp.name)/'wide',lambda _:None,round_seconds=601)
    def test_incarnation_slot_reserved_before_spawn_cannot_be_replenished_after_crash(self):
        path=Path(self.tmp.name)/'pre-spawn';guard=AttemptGuard(path,self.stops.append)
        reservation=guard.reserve_incarnation()
        self.assertEqual(guard.snapshot()['incarnation_slots'],1)
        self.assertEqual(guard.snapshot()['incarnations'],0)
        reopened=AttemptGuard(path,self.stops.append)
        with self.assertRaises(ValueError):reopened.reserve_incarnation()
        self.assertEqual(reopened.snapshot()['incarnation_slots'],1)
        self.assertTrue(reopened.snapshot()['halted'])

    def test_reserved_slot_bound_once_to_owned_namespace_before_arm(self):
        path=Path(self.tmp.name)/'bind';guard=AttemptGuard(path,self.stops.append)
        slot=guard.reserve_incarnation();guard.bind_incarnation(slot,'exact-owned-namespace')
        arm=owned_arm(guard,'owner-turn')
        self.assertEqual(arm['namespace'],'exact-owned-namespace')
        with self.assertRaises(ValueError):guard.bind_incarnation(slot,'other-namespace')
        self.assertEqual(guard.snapshot()['incarnations'],1)
