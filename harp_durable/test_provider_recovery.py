"""Real CLI/adapter recovery with controlled executables, never paid providers.

Both adapters execute a metadata-capable fixture inside the admitted namespace.
Only its private workspace contains controls/markers; guardian/cgroup ownership,
Git candidate production, SQLite recovery and public CLI decisions remain real.
"""
import json
import os
from pathlib import Path
import signal
import sys
import types
import unittest

from . import test_admission as admission_tests


def provider_fixture(name):
    version = "codex-cli 0.162.1" if name == "codex" else "traecli 0.210.1"
    return ('''#!/usr/bin/python3
import json, os, pathlib, subprocess, sys, time
args = sys.argv[1:]
if '--version' in args:
    print(VERSION)
    sys.exit(0)
if '--help' in args or 'exec' not in args:
    print('--ignore-user-config --ignore-rules --ephemeral --dangerously-bypass-approvals-and-sandbox --json --output-last-message --cd --model --color --disable --enable --no-daemon -c')
    sys.exit(0)
workspace = pathlib.Path(args[args.index('--cd') + 1])
assert pathlib.Path.cwd() == workspace
model = args[args.index('--model') + 1]
assert model in ('fixture-hold', 'fixture-fast')
assert sys.stdin.read()
marker = workspace / '.provider-launch.json'
assert not marker.exists(), 'provider relaunched in the same workspace'
child = subprocess.Popen(['/usr/bin/python3', '-c', 'import time; time.sleep(120)']) if model == 'fixture-hold' else None
marker.write_text(json.dumps({'provider': NAME, 'pid': os.getpid(), 'child': child.pid if child else None, 'launched_at': time.time()}))
if child:
    time.sleep(120)
(workspace / 'answer.txt').write_text('42\\n')
pathlib.Path(args[args.index('--output-last-message') + 1]).write_text('Controlled adapter fixture completed.\\n')
'''.replace('VERSION', repr(version)).replace('NAME', repr(name))).encode()


@unittest.skipUnless(sys.platform == "linux" and all(os.environ.get(name) for name in
                     ("HARP_TEST_CGROUP_ROOT", "HARP_TEST_ROOTFS", "HARP_DURABLE_BIN")),
                     "requires public Harp binary, native rootfs and disposable delegated Linux cgroup")
class ProviderRecovery(unittest.TestCase):
    git = admission_tests.GitInputs.git
    call = admission_tests.CodingLifecycle.call

    def setUp(self):
        admission_tests.CodingLifecycle.setUp(self)
        # Share native ownership helpers, without inheriting their test cases.
        from .test_recovery import RecoveryLifecycle
        from .test_execution_policy import metadata_fixture
        for name in ("wait_for", "start_process", "started_attempt", "crash", "group", "assert_empty", "guardian"):
            setattr(self, name, types.MethodType(getattr(RecoveryLifecycle, name), self))
        old_state, old_execution = self.state, self.execution
        self.state, self.execution = self.root / "providers-state", self.root / "providers-execution.json"
        self.command = [str(self.state) if item == str(old_state) else
                        str(self.execution) if item == str(old_execution) else item for item in self.command]
        providers, auth = self.root / "providers", self.root / "credentials"
        providers.mkdir(mode=0o700)
        auth.mkdir(mode=0o700)
        for name in ("codex", "trae", "codex-code-mode-host"):
            path = providers / name
            path.write_bytes(metadata_fixture(name) if name == "codex-code-mode-host" else provider_fixture(name))
            path.chmod(0o755)
        for name in ("codex", "trae"):
            path = auth / name
            path.write_text(json.dumps({"fixture": name}))
            path.chmod(0o600)
        self.call("prepare", "--rootfs", os.environ["HARP_TEST_ROOTFS"],
                  "--cgroup-root", os.environ["HARP_TEST_CGROUP_ROOT"], "--network", "host",
                  "--providers-bin", str(providers), "--codex-auth", str(auth / "codex"),
                  "--trae-auth", str(auth / "trae"))

    def admit_provider(self, run_id, provider, hold=False):
        return self.call("admit", run_id, "--repository", str(self.repo), "--task", "task.md",
                         "--acceptance", "acceptance.json", "--provider", provider,
                         "--model", "fixture-hold" if hold else "fixture-fast")

    def launched(self, run_id, provider):
        attempt = self.started_attempt(run_id)
        marker = Path(attempt["directory"]) / "payload/workspace/.provider-launch.json"
        self.wait_for(marker.is_file, "adapter executable never wrote its private launch marker")
        record = json.loads(marker.read_text())
        self.assertEqual(record["provider"], provider)
        self.assertIsInstance(record["child"], int)
        return attempt

    def test_cancel_holds_capacity_until_adapter_descendants_are_cleaned(self):
        for provider in ("codex", "trae"):
            with self.subTest(provider=provider):
                run_id, queued = provider + "-cancel", provider + "-queued"
                self.admit_provider(run_id, provider, hold=True)
                self.admit_provider(queued, provider)
                process = self.start_process()
                attempt = self.launched(run_id, provider)
                guardian = self.guardian(attempt)
                signal.pidfd_send_signal(guardian, signal.SIGSTOP)
                try:
                    self.assertTrue(self.call("cancel", run_id)["cancel_requested"])
                    self.assertEqual(self.call("status", queued)["attempts"], [])
                    events = (self.group(attempt).path / "cgroup.events").read_text()
                    self.assertIn("populated 1", events)
                finally:
                    signal.pidfd_send_signal(guardian, signal.SIGCONT)
                _, error = process.communicate(timeout=40)
                self.assertEqual(process.returncode, 0, error)
                self.assert_empty(attempt)
                cancelled = self.call("status", run_id)
                self.assertEqual(cancelled["status"], "BLOCKED", cancelled)
                self.assertEqual(cancelled["operations"][0]["status"], "UNCERTAIN")
                receipt = json.loads((Path(attempt["directory"]) / "receipt.json").read_text())
                self.assertTrue(receipt["cleanup"])
                self.assertEqual(receipt["cleanup_proof"]["populated"], 0)
                done = self.call("status", queued)
                self.assertEqual(done["status"], "SUCCEEDED", done)
                self.assertEqual(done["result"]["provider_report"]["provider"], provider)
                queued_marker = (Path(done["attempts"][0]["directory"]) /
                                 "payload/workspace/.provider-launch.json")
                self.assertGreaterEqual(json.loads(queued_marker.read_text())["launched_at"],
                                        receipt["finished_at"], "next adapter launched before cleanup receipt")

    def test_active_adapter_restart_requires_explicit_effect_resolution(self):
        for provider in ("codex", "trae"):
            with self.subTest(provider=provider):
                run_id = provider + "-interrupted"
                self.admit_provider(run_id, provider, hold=True)
                process = self.start_process()
                attempt = self.launched(run_id, provider)
                process.kill()
                process.communicate(timeout=10)
                self.wait_for(lambda: not self.call("status")["locked"], "guardian retained ownership after runtime death")
                self.call("start", "--until-idle", "--timeout", "30")
                blocked = self.call("status", run_id)
                self.assertEqual(blocked["status"], "BLOCKED", blocked)
                self.assertEqual(blocked["operations"][0]["status"], "UNCERTAIN")
                self.assertEqual(len(blocked["attempts"]), 1)
                self.assert_empty(attempt)
                self.call("start", "--until-idle", "--timeout", "30")
                self.assertEqual(len(self.call("status", run_id)["attempts"]), 1)
                decision = self.call("resolve", run_id, "coding", "fail", "--evidence",
                                     "Controlled provider was launched; kernel subtree empty; retain interrupted workspace, do not retry")
                self.assertFalse(decision["work_launched"])
                self.call("start", "--until-idle", "--timeout", "30")
                resolved = self.call("status", run_id)
                self.assertEqual(resolved["status"], "FAILED", resolved)
                self.assertEqual(len(resolved["attempts"]), 1)

    def test_completed_adapter_receipt_survives_restart_without_relaunch(self):
        for provider in ("codex", "trae"):
            with self.subTest(provider=provider):
                run_id = provider + "-completed"
                self.admit_provider(run_id, provider)
                self.crash("after_receipt")
                before = self.call("status", run_id)
                self.assertEqual(len(before["attempts"]), 1)
                attempt = before["attempts"][0]
                directory = Path(attempt["directory"])
                receipt_bytes = (directory / "receipt.json").read_bytes()
                receipt = json.loads(receipt_bytes)
                self.assertEqual(receipt["status"], "SUCCEEDED", receipt)
                self.assertEqual(receipt["result"]["provider_report"]["provider"], provider)
                bundle = (directory / "payload/candidate.bundle").read_bytes()
                marker = (directory / "payload/workspace/.provider-launch.json").read_bytes()
                self.assert_empty(attempt)
                for _ in range(2):
                    self.call("start", "--until-idle", "--timeout", "30")
                after = self.call("status", run_id)
                self.assertEqual(after["status"], "SUCCEEDED", after)
                self.assertEqual(after["result"], receipt["result"])
                self.assertEqual(len(after["attempts"]), 1)
                self.assertEqual((directory / "receipt.json").read_bytes(), receipt_bytes)
                self.assertEqual((directory / "payload/candidate.bundle").read_bytes(), bundle)
                self.assertEqual((directory / "payload/workspace/.provider-launch.json").read_bytes(), marker)


if __name__ == "__main__":
    unittest.main()
