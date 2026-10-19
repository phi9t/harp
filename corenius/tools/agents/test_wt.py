#!/usr/bin/env python3
"""Retired launchers refuse before reading legacy state or starting effects."""
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]


class RetiredLaunchers(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="retired-authority-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repository"
        self.repo.mkdir()
        subprocess.run(["git", "init", "-q", str(self.repo)], check=True)
        for name in ("legacy.sqlite3", "legacy.sock", "run.json", "owner.json"):
            (self.repo / name).write_bytes(b"preserve existing operator state\n")
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.trace = self.root / "launched"
        source = "#!" + sys.executable + "\nimport os\nfrom pathlib import Path\nPath(os.environ['RETIREMENT_TRACE']).write_text('launched')\nraise SystemExit(97)\n"
        for name in ("git", "tmux", "codex", "traecli", "traex", "cargo", "kata", "curl"):
            path = self.bin / name
            path.write_text(source)
            path.chmod(0o700)
        self.home = self.root / "home"
        self.home.mkdir()
        self.environment = {**os.environ, "PATH": str(self.bin) + os.pathsep + os.environ["PATH"],
                            "HOME": str(self.home), "PYTHONDONTWRITEBYTECODE": "1",
                            "RETIREMENT_TRACE": str(self.trace)}

    def snapshot(self):
        return {str(p.relative_to(self.root)): hashlib.sha256(p.read_bytes()).hexdigest()
                for p in self.root.rglob("*") if p.is_file()}

    def refusal(self, command):
        before = self.snapshot()
        result = subprocess.run(command, cwd=self.repo, env=self.environment,
                                capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 2, result.stdout + result.stderr)
        self.assertIn("Retired", result.stderr)
        self.assertIn("harp durable", result.stderr)
        self.assertEqual(result.stdout, "")
        self.assertEqual(self.snapshot(), before, command)

    def test_python_commands_and_direct_compatibility_modules_have_no_effects(self):
        cases = {
            "bench/harness/lead_driven.py": ["handoff", "--runtime", "codex"],
            "controller/launch/sqlite_baseline.py": ["start", "--run-id", "old"],
            "controller/launch/sqlite_responses_relay.py": ["legacy.sock"],
            "controller/support/codex_insula.py": [],
            "controller/operator/mac_dashboard.py": ["--config", "owner.json"],
            "controller/operator/mac_tmux.py": ["--config", "owner.json"],
            "controller/operator/mac_state_view.py": ["--config", "owner.json"],
            "tools/collab/collab.py": ["init", "--trunk", "main"],
            "tools/collab/cli.py": ["land", "attempt"],
            "tools/collab/worktree_compat.py": ["new", "ticket"],
            "tools/collab/legacy_landing.py": ["--push", "ticket"],
            "tools/agents/wt.py": ["land", "ticket"],
            "tools/agents/land.py": ["ticket", "--no-push"],
            "tools/agents/worker.py": ["launch", "--runtime", "codex", "--task", "task.md"],
            "tools/agents/lead_launch.py": ["launch", "effort"],
            "tools/agents/monitor_wake.py": ["--config", "owner.json"],
            "tools/agents/msg.py": ["send", "worker", "hello"],
            "tools/agents/effort.py": ["rehydrate", "effort"],
            "tools/agents/tracker.py": ["claim", "effort", "01", "--owner", "agent"],
            "tools/agents/claims.py": ["record", "report.md", "--passed"],
            "tools/agents/acceptance.py": ["record", "effort", "01"],
        }
        for path, args in cases.items():
            with self.subTest(path=path):
                self.refusal([sys.executable, str(ROOT / path), *args])

    def test_old_benchmark_provider_command_and_scheduler_modes_refuse(self):
        for args in (["task", "--agent", "traecli", "--authorize-model-spend"],
                     ["task", "--agent=cmd:touch unauthorized"],
                     ["task", "--condition", "controller:t1"],
                     ["task", "--condition=fleet:2"]):
            with self.subTest(args=args):
                self.refusal([sys.executable, str(ROOT / "bench/harness/run.py"), *args])

    def test_imported_library_authority_refuses_before_opening_or_launching(self):
        code = """import json,sys
from types import SimpleNamespace
sys.path[:0]=[AGENTS,COLLAB]
import worker
from core import Collab
from state_store import StateStore
sentinel = SimpleNamespace(run_dir='.', findings_file='run.json')
actions = [
 ('launch',(None,),{}),('supervise',(None,),{}),
 ('create_private_worker_clone',(None,None,None,None),{}),
 ('import_candidate_to_shared_repo',(None,None),{}),('refresh_integration_branch',(None,),{}),
 ('resume',(sentinel,),{}),('resume_locked',(sentinel,None),{}),
 ('deliver_findings',(sentinel,),{}),('deliver_findings_locked',(sentinel,None),{}),
 ('tmux_interactive_window',(None,None,None,None),{}),('run_tmux',('new-session',),{}),
 ('terminal_delivery_receipt',(None,None,None,None,None,None),{}),
 ('resume_command',(None,None,None,None),{}),
 ('run_record_lock',('.',),{}),('locked_run',('.',),{}),('save_run',(None,),{}),
 ('new_run_temp',(None,),{}),('ensure_run_temp',(None,),{}),('cleanup_run_temp',(None,),{}),
 ('temp_parent',(),{}),('temporary_directory',(None,),{}),
 ('sandbox_binds',(None,None),{}),('remove_untouched',(None,),{}),
 ('write_external_baseline',(None,None,None),{}),('record_shared_rework',(None,None,None),{}),
 ('acknowledge_delivery',(None,None,None),{}),('acknowledge_pending_rework_deliveries',(None,),{}),
 ('status',(sentinel,),{}),('status_verdict',(None,),{'persist':True}),
 ('handoff_state',(None,None,None),{'persist':True}),('update_session_id_from_events',(None,),{}),
 ('Checker',({},),{}),
]
for name,args,kwargs in actions:
 try: getattr(worker,name)(*args,**kwargs)
 except Exception as error:
  assert 'Retired' in str(error),(name,repr(error))
 else: raise AssertionError('old authority was callable: '+name)
for action in (lambda: Collab('.'),lambda: StateStore(None,None)):
 try: action()
 except RuntimeError as error: assert 'Retired' in str(error),repr(error)
 else: raise AssertionError('old store authority was callable')
# Retained descriptions and parsers must not start tools or create worker state.
assert json.loads(worker.toml_array(['one','two'])) == ['one','two']
assert worker.runtime_temp_env({'tmpdir':'owned','tmux_tmpdir':'terminal','tmpprefix':'prefix'}) == {'TMPDIR':'owned','TMUX_TMPDIR':'terminal','TMPPREFIX':'prefix'}
assert worker.porcelain_paths(' M answer.txt') == ['answer.txt']
""".replace("AGENTS", repr(str(ROOT / "tools/agents"))).replace("COLLAB", repr(str(ROOT / "tools/collab")))
        before = self.snapshot()
        result = subprocess.run([sys.executable, "-c", code], cwd=self.repo,
                                env=self.environment, capture_output=True, text=True, timeout=10)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertEqual(self.snapshot(), before)

    def test_rust_explicit_targets_and_legacy_modes_refuse_without_effects(self):
        cargo = shutil.which("cargo")
        self.assertIsNotNone(cargo, "Rust retirement targets require the pinned Rust toolchain")
        target = self.root / "rust-target"
        subprocess.run([cargo, "build", "--locked", "--offline", "--bins", "--manifest-path",
                        str(ROOT / "Cargo.toml"), "--target-dir", str(target)],
                       cwd=ROOT, check=True, text=True, timeout=60)
        for binary in ("corenius", "corenius-sqlite-controller", "corenius-attempt-supervisor", "corenius-attempt-gate"):
            for args in (["legacy.sqlite3", "legacy.sock"], ["run", "--provider", "codex"],
                         ["supervise", "--provider", "traecli"], ["eval", "--help"], ["--help"]):
                with self.subTest(binary=binary,args=args):
                    self.refusal([str(target / "debug" / binary), *args])
        metadata = subprocess.check_output([cargo, "metadata", "--locked", "--offline", "--no-deps",
                                            "--format-version", "1", "--manifest-path", str(ROOT / "Cargo.toml")],
                                           cwd=ROOT, text=True, timeout=10)
        self.assertEqual({tuple(target["kind"]) for package in json.loads(metadata)["packages"]
                          for target in package["targets"]}, {("bin",)})


if __name__ == "__main__":
    unittest.main()
