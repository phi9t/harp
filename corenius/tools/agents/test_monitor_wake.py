"""Wake protocol through real helper and owned fake Codex subprocesses."""

from __future__ import annotations

import json
import os
from pathlib import Path
import signal
import subprocess
import sys
import tempfile
import time
import unittest

# Apply the repository temporary-root policy before any host allocation.
import runpy as _temp_policy_runpy
from pathlib import Path as _temp_policy_path
import tempfile as _temp_policy_tempfile
import os as _temp_policy_os
_temp_policy_file = next(parent / "tools" / "temp_root.py" for parent in _temp_policy_path(__file__).resolve().parents if (parent / "tools" / "temp_root.py").is_file())
_temp_policy_root = str(_temp_policy_runpy.run_path(str(_temp_policy_file))["temporary_root"]())
_temp_policy_tempfile.tempdir = _temp_policy_root
_temp_policy_os.environ.update(TMPDIR=_temp_policy_root, TMP=_temp_policy_root, TEMP=_temp_policy_root)

HELPER = Path(__file__).with_name("monitor_wake.py")
LEAD = "01a10084-dc3a-7743-b0c1-4694d5ecc621"
OTHER = "01a1078b-d944-7c82-a392-50333a175a77"
RECEIPT = "01a10815-d7dc-7911-8183-aa49cc389d4c"


class WakeTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        self.state = self.root / "state"
        self.fake = self.root / "fake codex"
        self.fake.write_text(f"#!{sys.executable}\n" + '''
import json, os, pathlib, sqlite3, sys, time
root = pathlib.Path(__file__).parent
args = sys.argv[1:]
if len(args) != 5 or args[:2] != ['queue', '--thread'] or args[3] != '--message':
    sys.exit(64)
# The actual subprocess must observe durable intent before any queue request.
with sqlite3.connect(root / 'state' / 'journal.sqlite') as db:
    wake = db.execute("SELECT id FROM wakes WHERE status='prepared'").fetchone()
if not wake or wake[0] not in args[4]:
    sys.exit(65)
with (root / 'requests').open('a') as out:
    out.write(json.dumps({'argv': args, 'wake': wake[0]}) + '\\n')
(root / 'started').write_text(str(os.getpid()))
mode = (root / 'mode').read_text() if (root / 'mode').exists() else 'ok'
if mode == 'wait':
    time.sleep(30)
elif mode == 'large':
    os.write(1, b'x' * (256 * 1024))
    time.sleep(30)
elif mode == 'ambiguous':
    print('accepted')
    sys.exit(0)
elif mode == 'wrong':
    print('Queued message 01a10815-d7dc-7911-8183-aa49cc389d4c for thread 01a1078b-d944-7c82-a392-50333a175a77.')
    sys.exit(0)
elif mode == 'nonzero':
    print('Queued message 01a10815-d7dc-7911-8183-aa49cc389d4c for thread ' + args[2] + '.')
    sys.exit(3)
elif mode == 'duplicate':
    print('Queued message 01a10815-d7dc-7911-8183-aa49cc389d4c for thread ' + args[2] + '.')
print('ancillary CLI output')
print('Queued message 01a10815-d7dc-7911-8183-aa49cc389d4c for thread ' + args[2] + '.')
''')
        self.fake.chmod(0o700)
        self.env = dict(os.environ, CODEX_THREAD_ID=LEAD)

    def tearDown(self):
        self.tmp.cleanup()

    def argv(self, command, *args):
        return [sys.executable, str(HELPER), command, "--state", str(self.state), *args]

    def call(self, command, *args, thread=LEAD, code=0):
        env = dict(self.env, CODEX_THREAD_ID=thread)
        result = subprocess.run(self.argv(command, *args), env=env, text=True,
                                capture_output=True, timeout=8)
        self.assertEqual(result.returncode, code, result.stdout + result.stderr)
        return json.loads(result.stdout if code == 0 or result.stdout else result.stderr)

    def init(self, timeout="1"):
        return self.call("init", "--lead-thread", LEAD, "--codex", str(self.fake),
                         "--spec", str(self.root / "spec"), "--ledger", str(self.root / "ledger"),
                         "--execution-state", str(self.root / "execution"), "--timeout", timeout)

    def requests(self):
        path = self.root / "requests"
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def test_receipt_and_lifecycle_enable_next_tick(self):
        self.init()
        first = self.call("tick")["outstanding"]
        self.assertEqual(first["status"], "queued")
        self.assertEqual(first["receipt_id"], RECEIPT)
        self.assertEqual(self.requests()[0]["argv"][2], LEAD)
        self.assertEqual(self.call("tick")["outstanding"]["id"], first["id"])
        self.call("begin", "--wake", first["id"])
        self.assertEqual(self.call("tick")["outstanding"]["status"], "reviewing")
        self.call("finish", "--wake", first["id"])
        self.assertIsNone(self.call("status")["outstanding"])
        second = self.call("tick")["outstanding"]
        self.assertNotEqual(first["id"], second["id"])
        self.assertEqual(len(self.requests()), 2)

    def test_two_racing_ticks_make_one_request(self):
        self.init()
        processes = [subprocess.Popen(self.argv("tick"), env=self.env,
                                      stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                     for _ in range(2)]
        outputs = [p.communicate(timeout=8) for p in processes]
        for p, (out, err) in zip(processes, outputs):
            self.assertEqual(p.returncode, 0, out + err)
        self.assertEqual(len(self.requests()), 1)
        self.assertEqual(json.loads(outputs[0][0])["outstanding"]["id"],
                         json.loads(outputs[1][0])["outstanding"]["id"])

    def test_dispatch_brief_requires_one_nonblocking_filtered_observer(self):
        self.init()
        message = self.call("tick")["outstanding"]["prompt"]
        for instruction in (
            "exactly one fresh-context observe-only subagent",
            "fork_turns=none", "Do not wait", "delegate-finish",
            "self-contained", "current execution/assignment binding",
            "one observation-ledger writer", "pending backlog",
            "filter BEFORE any model input or tool output",
            "Do not print raw JSONL", "Reasoning", "allowlist",
            "fresh reviewer context", "no assessment or cursor progress",
            "Main lead owns adjudication", "coordinator alone owns corrections",
            "overrides the retained spec", "Dispatch policy source:", "sha256=",
        ):
            self.assertIn(instruction, message)
        self.assertLess(message.index("filter BEFORE"), message.index("Compare newly visible"))

    def delegate(self, wake, reviewer="/root/wake-observer"):
        return self.call("delegate", "--wake", wake, "--reviewer", reviewer,
                         "--brief", "Source-bound public brief: spec, binding, policy, bounded review.")

    def test_receipt_grants_only_one_reviewer_and_bounded_finish(self):
        self.init()
        wake = self.call("tick")["outstanding"]["id"]
        self.delegate_before_begin(wake)
        self.call("begin", "--wake", wake)
        for thread in (OTHER, ""):
            self.call("delegate", "--wake", wake, "--reviewer", "/root/wake-observer",
                      "--brief", "source-bound brief", thread=thread, code=1)
        result = self.delegate(wake)
        receipt = result["delegation"]
        self.assertEqual(receipt["wake"], wake)
        self.assertEqual(receipt["reviewer"], "/root/wake-observer")
        self.assertEqual(receipt["sources"]["ledger"], str(self.root / "ledger"))
        self.assertEqual(self.delegate(wake)["delegation"], receipt)
        self.call("delegate", "--wake", wake, "--reviewer", "/root/duplicate",
                  "--brief", receipt["brief"], code=1)
        self.call("delegate", "--wake", wake, "--reviewer", receipt["reviewer"],
                  "--brief", "changed brief", code=1)
        self.assertEqual(self.call("tick")["outstanding"]["status"], "reviewing")
        for reviewer, assessment in (("/root/duplicate", "ledger receipt 1"), (receipt["reviewer"], "")):
            self.call("delegate-finish", "--wake", wake, "--reviewer", reviewer,
                      "--assessment", assessment, thread=OTHER, code=1)
        result = self.call("delegate-finish", "--wake", wake, "--reviewer", receipt["reviewer"],
                           "--assessment", "ledger receipt 1; covered cursor 20; backlog 21+",
                           thread=OTHER)
        self.assertIsNone(result["outstanding"])
        self.call("delegate-finish", "--wake", wake, "--reviewer", receipt["reviewer"],
                  "--assessment", "ledger receipt 1; covered cursor 20; backlog 21+", thread=OTHER)
        self.assertNotEqual(self.call("tick")["outstanding"]["id"], wake)
        self.assertEqual(len(self.requests()), 2)

    def delegate_before_begin(self, wake):
        self.call("delegate", "--wake", wake, "--reviewer", "/root/wake-observer",
                  "--brief", "source-bound brief", code=1)

    def test_reviewer_loss_leaves_reviewing_and_private_receipt(self):
        self.init()
        wake = self.call("tick")["outstanding"]["id"]
        self.call("begin", "--wake", wake)
        self.call("delegate-finish", "--wake", wake, "--reviewer", "/root/wake-observer",
                  "--assessment", "unrecorded", thread=OTHER, code=1)
        receipt = self.delegate(wake)["delegation"]
        path = self.state / "delegates" / (wake + ".json")
        self.assertEqual(path.stat().st_mode & 0o777, 0o600)
        self.assertEqual(path.parent.stat().st_mode & 0o777, 0o700)
        self.assertEqual(json.loads(path.read_text()), receipt)
        for _ in range(2):
            self.assertEqual(self.call("tick")["outstanding"]["status"], "reviewing")
        self.assertEqual(len(self.requests()), 1)

    def test_explicit_recovery_preserves_receipt_and_fences_old_finish(self):
        self.init()
        wake = self.call("tick")["outstanding"]["id"]
        self.call("begin", "--wake", wake)
        previous = self.delegate(wake)["delegation"]
        args = ("--wake", wake, "--reviewer", "/root/fresh-observer", "--brief", "fresh public brief",
                "--prior-reviewer", previous["reviewer"])
        self.call("delegate", *args, code=1)
        self.call("delegate", *args, "--recovery-reason", "incident 1; native status confirms interrupted and idle",
                  thread=OTHER, code=1)
        recovered = self.call("delegate", *args, "--recovery-reason",
                              "incident 1; native status confirms interrupted and idle")["delegation"]
        self.assertEqual(recovered["predecessor"], previous)
        self.assertNotEqual(recovered["reviewer"], previous["reviewer"])
        self.call("delegate-finish", "--wake", wake, "--reviewer", previous["reviewer"],
                  "--assessment", "stale publication", thread=OTHER, code=1)
        self.assertEqual(self.call("status")["outstanding"]["status"], "reviewing")
        self.call("delegate-finish", "--wake", wake, "--reviewer", recovered["reviewer"],
                  "--assessment", "fresh receipt 2; cursor 20; remaining backlog", thread=OTHER)

    def test_racing_delegate_receipts_admit_one_identity(self):
        config = self.init()["config"]
        wake = self.call("tick")["outstanding"]["id"]
        self.call("begin", "--wake", wake)
        processes = [subprocess.Popen(self.argv("delegate", "--wake", wake,
                     "--reviewer", reviewer, "--brief", "exact source-bound public brief"),
                     env=self.env, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
                     for reviewer in ("/root/observer-a", "/root/observer-b")]
        outputs = [p.communicate(timeout=8) for p in processes]
        self.assertEqual(sorted(p.returncode for p in processes), [0, 1], outputs)
        status = self.call("status")
        self.assertEqual(status["config"], config)
        self.assertEqual(status["outstanding"]["status"], "reviewing")
        self.assertIn(status["delegation"]["reviewer"], ("/root/observer-a", "/root/observer-b"))
        self.call("stop")
        self.call("delegate", "--wake", wake, "--reviewer", "/root/replacement", "--brief", "new brief",
                  "--prior-reviewer", status["delegation"]["reviewer"],
                  "--recovery-reason", "old reviewer quiescent", code=1)
        self.call("delegate-finish", "--wake", wake, "--reviewer", status["delegation"]["reviewer"],
                  "--assessment", "saved coverage before stop; remaining backlog", thread=OTHER)
        self.call("tick", code=1)

    def test_recovery_cannot_reuse_any_retired_identity(self):
        self.init()
        wake = self.call("tick")["outstanding"]["id"]
        self.call("begin", "--wake", wake)
        first = self.delegate(wake)["delegation"]
        second = self.call("delegate", "--wake", wake, "--reviewer", "/root/observer-b",
                           "--brief", "public brief B", "--prior-reviewer", first["reviewer"],
                           "--recovery-reason", "A quiescent; incident 1")["delegation"]
        self.call("delegate", "--wake", wake, "--reviewer", first["reviewer"],
                  "--brief", "public brief A again", "--prior-reviewer", second["reviewer"],
                  "--recovery-reason", "B quiescent; incident 2", code=1)
        third = self.call("delegate", "--wake", wake, "--reviewer", "/root/observer-c",
                          "--brief", "public brief C", "--prior-reviewer", second["reviewer"],
                          "--recovery-reason", "B quiescent; incident 2")["delegation"]
        for receipt in (first, second):
            self.call("delegate-finish", "--wake", wake, "--reviewer", receipt["reviewer"],
                      "--assessment", "stale publication", thread=OTHER, code=1)
        self.assertEqual(self.call("status")["delegation"], third)
        self.assertEqual(self.call("status")["outstanding"]["status"], "reviewing")

    def test_uncertain_dispatch_survives_restart_and_suppresses_retry(self):
        for mode in ("ambiguous", "wrong", "nonzero", "duplicate", "wait", "large"):
            with self.subTest(mode=mode):
                # Distinct private state per fault; each invocation is a new process.
                if self.state.exists():
                    import shutil
                    shutil.rmtree(self.state)
                (self.root / "mode").write_text(mode)
                before = len(self.requests())
                self.init(timeout="0.2")
                wake = self.call("tick", code=1)["outstanding"]
                self.assertEqual(wake["status"], "unknown")
                self.assertTrue(wake["detail"])
                self.assertLessEqual(len(wake["stdout"].encode()) + len(wake["stderr"].encode()), 65536)
                self.assertEqual(self.call("status")["outstanding"]["id"], wake["id"])
                self.assertEqual(self.call("tick")["outstanding"]["id"], wake["id"])
                self.assertEqual(len(self.requests()), before + 1)

    def test_wrong_or_missing_thread_cannot_begin_or_finish(self):
        self.init()
        wake = self.call("tick")["outstanding"]["id"]
        for thread in (OTHER, ""):
            self.call("begin", "--wake", wake, thread=thread, code=1)
        self.call("finish", "--wake", wake, code=1)
        self.assertEqual(self.call("status")["outstanding"]["status"], "queued")
        self.call("begin", "--wake", wake)
        for thread in (OTHER, ""):
            self.call("finish", "--wake", wake, thread=thread, code=1)
        self.assertEqual(self.call("status")["outstanding"]["status"], "reviewing")
        self.call("finish", "--wake", wake)
        self.call("finish", "--wake", wake)  # Idempotent completion of this wake only.

    def test_interrupted_dispatch_stays_unknown_without_requeue(self):
        self.init(timeout="0.4")
        (self.root / "mode").write_text("wait")
        process = subprocess.Popen(self.argv("tick"), env=self.env,
                                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        try:
            deadline = time.monotonic() + 3
            while not (self.root / "started").exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertTrue((self.root / "started").exists())
            process.kill()
            process.wait(timeout=3)
            wake = self.call("status")["outstanding"]
            self.assertEqual(wake["status"], "unknown")
            self.call("tick")
            self.assertEqual(len(self.requests()), 1)
            # Receipt may be lost while the exact lead still receives the prompt.
            self.call("begin", "--wake", wake["id"])
            self.call("finish", "--wake", wake["id"])
        finally:
            if process.poll() is None:
                process.kill()
                process.wait(timeout=3)
            # Only the PID recorded by this owned fake CLI can outlive the
            # deliberately killed helper. Never touch the real Codex runtime.
            started = self.root / "started"
            if started.exists():
                try:
                    os.kill(int(started.read_text()), signal.SIGKILL)
                except ProcessLookupError:
                    pass

    def test_explicit_unknown_resolution_is_required_before_next_tick(self):
        self.init()
        (self.root / "mode").write_text("ambiguous")
        wake = self.call("tick", code=1)["outstanding"]["id"]
        self.call("resolve", "--wake", wake, "--outcome", "queued", "--reason", "inspected", code=1)
        self.call("resolve", "--wake", wake, "--outcome", "not-queued", "--reason", "", code=1)
        self.call("resolve", "--wake", wake, "--outcome", "queued", "--receipt", RECEIPT,
                  "--reason", "operator confirmed exact target receipt")
        self.call("resolve", "--wake", wake, "--outcome", "not-queued", "--reason", "unsafe", code=1)
        self.call("begin", "--wake", wake)
        self.call("finish", "--wake", wake)
        wake2 = self.call("tick", code=1)["outstanding"]["id"]
        self.call("resolve", "--wake", wake2, "--outcome", "not-queued", "--reason", "confirmed no queue")
        self.assertIsNone(self.call("status")["outstanding"])

    def test_stop_refuses_ticks_and_configuration_replacement(self):
        self.init()
        wake = self.call("tick")["outstanding"]["id"]
        self.call("begin", "--wake", wake)
        self.call("stop")
        self.assertTrue(self.call("status")["config"]["stopped"])
        self.call("tick", code=1)
        self.call("finish", "--wake", wake)
        self.call("tick", code=1)
        self.assertEqual(len(self.requests()), 1)
        self.call("init", "--lead-thread", OTHER, "--codex", str(self.fake),
                  "--spec", "s", "--ledger", "l", "--execution-state", "e", code=1)
        self.assertEqual(self.call("status")["config"]["lead_thread"], LEAD)

    def test_pin_change_is_visible_and_does_not_dispatch(self):
        self.init()
        self.fake.write_text(self.fake.read_text() + "# changed\n")
        wake = self.call("tick", code=1)["outstanding"]
        self.assertEqual(wake["status"], "unknown")
        self.assertIn("binary", wake["detail"])
        self.call("tick")
        self.assertEqual(self.requests(), [])

    def test_state_and_journal_are_private_and_unsafe_state_is_refused(self):
        self.init()
        self.assertEqual(self.state.stat().st_mode & 0o777, 0o700)
        for name in ("journal.sqlite", "writer.lock"):
            self.assertEqual((self.state / name).stat().st_mode & 0o777, 0o600)
        self.state.chmod(0o755)
        self.call("tick", code=1)
        self.assertEqual(self.requests(), [])


if __name__ == "__main__":
    unittest.main()
