from __future__ import annotations

import sys
import tempfile
import unittest
import json
import os
from pathlib import Path
from unittest import mock


sys.path.insert(0, str(Path(__file__).parents[1]))

import lean_trae_benchmark as benchmark


class LauncherContractTests(unittest.TestCase):
    def test_build_command_uses_only_the_pinned_binary_and_isolated_config(self) -> None:
        command = benchmark.build_command(
            traecli=Path("/sealed/runtime/traecli"),
            workspace=Path("/sealed/run/workspace"),
            prompt_path=Path("/sealed/run/prompt.md"),
            final_message=Path("/sealed/run/candidate.lean"),
        )

        self.assertEqual(command[:2], ["/sealed/runtime/traecli", "exec"])
        self.assertIn("--ephemeral", command)
        self.assertIn("--ignore-user-config", command)
        self.assertIn("--ignore-rules", command)
        self.assertIn("--skip-git-repo-check", command)
        self.assertIn("--json", command)
        self.assertIn("--sandbox", command)
        self.assertEqual(command[command.index("--sandbox") + 1], "workspace-write")
        self.assertIn("--output-last-message", command)
        self.assertEqual(
            command[command.index("--output-last-message") + 1],
            "/sealed/run/candidate.lean",
        )
        self.assertNotIn("--permission-mode", command)
        self.assertNotIn("--dangerously-bypass-approvals-and-sandbox", command)
        allowed_tools = [
            command[index + 1]
            for index, item in enumerate(command)
            if item == "--allowed-tool"
        ]
        self.assertEqual(
            allowed_tools,
            [
                "Read",
                "Glob",
                "Grep",
                "Write",
                "Edit",
                "Bash(lake env lean:*)",
                "Bash(lean:*)",
            ],
        )
        self.assertNotIn("Bash", allowed_tools)

    def test_inspect_cli_accepts_only_the_pinned_trae_version(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            executable = Path(directory) / "traecli"
            executable.write_text("#!/bin/sh\nprintf 'traecli 0.200.19(internal edition)\\n'\n")
            executable.chmod(0o755)

            identity = benchmark.inspect_cli(executable)

        self.assertEqual(identity["version"], "0.200.19")
        self.assertEqual(identity["path"], str(executable))
        self.assertRegex(identity["sha256"], r"^[0-9a-f]{64}$")

    def test_inspect_cli_rejects_a_different_version(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            executable = Path(directory) / "traecli"
            executable.write_text("#!/bin/sh\nprintf 'traecli 0.200.20\\n'\n")
            executable.chmod(0o755)

            with self.assertRaisesRegex(benchmark.ValidationError, "0.200.19"):
                benchmark.inspect_cli(executable)

    def test_inspect_cli_rejects_an_unpinned_binary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            executable = Path(directory) / "traecli"
            executable.write_text("#!/bin/sh\nprintf 'traecli 0.200.19\\n'\n")
            executable.chmod(0o755)

            with self.assertRaisesRegex(benchmark.ValidationError, "SHA-256"):
                benchmark.inspect_cli(executable, expected_sha256="0" * 64)

    def test_run_attempt_captures_the_full_unmodified_jsonl_and_final_message(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            executable = root / "traecli"
            executable.write_text(
                "#!/usr/bin/env python3\n"
                "import os, pathlib, sys\n"
                "if sys.argv[1:] == ['--version']:\n"
                "    print('traecli 0.200.19')\n"
                "    raise SystemExit(0)\n"
                "output = pathlib.Path(sys.argv[sys.argv.index('--output-last-message') + 1])\n"
                "prompt = sys.stdin.read()\n"
                "assert prompt == 'prove the sealed theorem\\n'\n"
                "assert os.environ.get('HARP_UNRELATED_HOST_VALUE') is None\n"
                "assert os.environ['PATH'].split(':')[0] == '/sealed/lean/bin'\n"
                "assert os.environ['ELAN_HOME'] == '/sealed/lean/elan'\n"
                "output.write_text('theorem answer : True := by trivial\\n')\n"
                "print('{\\\"type\\\":\\\"thread.started\\\",\\\"id\\\":\\\"real-session\\\"}')\n"
                "print('{\\\"type\\\":\\\"item.completed\\\",\\\"tool\\\":\\\"Read\\\"}')\n"
            )
            executable.chmod(0o755)
            prompt = root / "prompt.md"
            prompt.write_text("prove the sealed theorem\n")
            run = root / "run"

            with mock.patch.dict(os.environ, {"HARP_UNRELATED_HOST_VALUE": "nope"}), mock.patch.object(
                benchmark, "TRAE_DARWIN_ARM64_SHA256", benchmark.sha256_file(executable)
            ):
                runtime = root / "runtime"
                benchmark.provision_pinned_cli(executable, runtime)
                result = benchmark.run_attempt(
                    runtime_root=runtime,
                    workspace=root,
                    prompt_path=prompt,
                    run_dir=run,
                    lean_bin=Path("/sealed/lean/bin"),
                    elan_home=Path("/sealed/lean/elan"),
                )

            self.assertEqual(result["returncode"], 0)
            self.assertEqual(
                (run / "candidate.lean").read_text(),
                "theorem answer : True := by trivial\n",
            )
            self.assertEqual(
                (run / "events.jsonl").read_text(),
                '{"type":"thread.started","id":"real-session"}\n'
                '{"type":"item.completed","tool":"Read"}\n',
            )
            metadata = json.loads((run / "attempt.json").read_text())
            self.assertEqual(metadata["prompt_sha256"], benchmark.sha256_file(prompt))
            self.assertRegex(metadata["events_sha256"], r"^[0-9a-f]{64}$")
            with self.assertRaisesRegex(benchmark.ValidationError, "existing run"):
                benchmark.run_attempt(
                    runtime_root=root / "runtime",
                    workspace=root,
                    prompt_path=prompt,
                    run_dir=run,
                )

    def test_receipt_is_redacted_but_hash_linked_to_the_full_local_trace(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            run = root / "run"
            run.mkdir()
            (run / "events.jsonl").write_text(
                '{"type":"item.completed","tool":"Read","token":"sk-not-for-export"}\n'
            )
            (run / "stderr.txt").write_text("private stderr\n")
            (run / "candidate.lean").write_text("theorem answer : True := by trivial\n")
            (run / "attempt.json").write_text(
                json.dumps(
                    {
                        "schema_version": "harp-lean-trae-attempt/v1",
                        "returncode": 0,
                        "events_sha256": benchmark.sha256_file(run / "events.jsonl"),
                        "stderr_sha256": benchmark.sha256_file(run / "stderr.txt"),
                        "candidate_sha256": benchmark.sha256_file(run / "candidate.lean"),
                    }
                )
            )
            receipt = root / "receipt.json"

            benchmark.export_receipt(run, receipt)
            value = json.loads(receipt.read_text())

            self.assertEqual(value["events_sha256"], benchmark.sha256_file(run / "events.jsonl"))
            self.assertEqual(value["tool_events"], ["Read"])
            self.assertNotIn("sk-not-for-export", receipt.read_text())
            benchmark.verify_receipt(run, receipt)
            (run / "events.jsonl").write_text('{"type":"tampered"}\n')
            with self.assertRaisesRegex(benchmark.ValidationError, "events digest"):
                benchmark.verify_receipt(run, receipt)

    def test_compile_core_candidate_uses_a_fresh_compiler_process(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            compiler = root / "lean"
            compiler.write_text(
                "#!/usr/bin/env python3\n"
                "import os, pathlib, sys\n"
                "candidate = pathlib.Path(sys.argv[1])\n"
                "assert candidate.read_text() == 'theorem core_and_left (p q : Prop) : p ∧ q → p := by\\n  exact fun h => h.1\\n'\n"
                "assert os.environ['ELAN_HOME'] == '/sealed/lean/elan'\n"
                "print('checked by isolated compiler')\n"
                "#" + "x" * (4 * 1024 * 1024)
            )
            compiler.chmod(0o755)
            candidate = root / "candidate.lean"
            candidate.write_text(
                "theorem core_and_left (p q : Prop) : p ∧ q → p := by\n"
                "  exact fun h => h.1\n"
            )

            result = benchmark.compile_core_candidate(
                compiler,
                candidate,
                elan_home=Path("/sealed/lean/elan"),
            )

        self.assertTrue(result["passed"])
        self.assertEqual(result["returncode"], 0)
        self.assertEqual(result["stdout"], "checked by isolated compiler\n")
        self.assertRegex(result["candidate_sha256"], r"^[0-9a-f]{64}$")

    def test_trace_policy_accepts_only_the_declared_lean_shell_command(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            events = Path(directory) / "events.jsonl"
            events.write_text(
                '{"type":"item.completed","item":{"type":"command_execution",'
                '"command":"lean Scratch.lean","exit_code":0}}\n'
            )
            self.assertEqual(benchmark.validate_trace_policy(events), ["lean Scratch.lean"])
            events.write_text(
                '{"type":"item.completed","item":{"type":"command_execution",'
                '"command":"curl https://example.test","exit_code":0}}\n'
            )
            with self.assertRaisesRegex(benchmark.ValidationError, "not allowlisted"):
                benchmark.validate_trace_policy(events)

    def test_core_candidate_rejects_imports_and_a_different_declaration(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            candidate = Path(directory) / "candidate.lean"
            candidate.write_text("import Mathlib\ntheorem other : True := by trivial\n")

            with self.assertRaisesRegex(benchmark.ValidationError, "Core task"):
                benchmark.validate_core_candidate(candidate)
            candidate.write_text(
                "theorem core_and_left (p q : Prop) : p ∧ q → p := by\n"
                "  exact fun h => h.1\n"
                "theorem unrelated : True := by trivial\n"
            )
            with self.assertRaisesRegex(benchmark.ValidationError, "top-level"):
                benchmark.validate_core_candidate(candidate)
            candidate.write_text(
                "theorem core_and_left (p q : Prop) : p ∧ q → p := by\n"
                "  exact fun h => h.1\n"
                " private theorem smuggled : True := by trivial\n"
            )
            with self.assertRaisesRegex(benchmark.ValidationError, "command"):
                benchmark.validate_core_candidate(candidate)
            candidate.write_text(
                "theorem core_and_left (p q : Prop) : p ∧ q → p := by\n"
                "  exact fun h => h.1\n"
                "private theorem smuggled : True := by trivial\n"
            )
            with self.assertRaisesRegex(benchmark.ValidationError, "top-level"):
                benchmark.validate_core_candidate(candidate)
            candidate.write_text(
                "theorem core_and_left (p q : Prop) : p ∧ q → p := by\n"
                "  exact fun h => h.1\n"
                "inductive Smuggled where | mk\n"
            )
            with self.assertRaisesRegex(benchmark.ValidationError, "top-level"):
                benchmark.validate_core_candidate(candidate)

    def test_receipt_refuses_a_policy_violating_attempt(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            run = root / "run"
            run.mkdir()
            (run / "events.jsonl").write_text(
                '{"type":"item.completed","item":{"type":"command_execution",'
                '"command":"curl https://example.test"}}\n'
            )
            (run / "stderr.txt").write_text("")
            (run / "candidate.lean").write_text(
                "theorem core_and_left (p q : Prop) : p ∧ q → p := by\n  exact fun h => h.1\n"
            )
            (run / "attempt.json").write_text(
                json.dumps(
                    {
                        "schema_version": "harp-lean-trae-attempt/v1",
                        "returncode": 0,
                        "terminal_state": "policy_violation",
                        "events_sha256": benchmark.sha256_file(run / "events.jsonl"),
                        "stderr_sha256": benchmark.sha256_file(run / "stderr.txt"),
                        "candidate_sha256": benchmark.sha256_file(run / "candidate.lean"),
                    }
                )
            )

            with self.assertRaisesRegex(benchmark.ValidationError, "policy"):
                benchmark.export_receipt(run, root / "receipt.json")
            receipt = root / "receipt.json"
            receipt.write_text(
                json.dumps(
                    {
                        "schema_version": "harp-lean-trae-receipt/v1",
                        "attempt_sha256": benchmark.sha256_file(run / "attempt.json"),
                        "events_sha256": benchmark.sha256_file(run / "events.jsonl"),
                        "stderr_sha256": benchmark.sha256_file(run / "stderr.txt"),
                        "candidate_sha256": benchmark.sha256_file(run / "candidate.lean"),
                        "returncode": 0,
                        "tool_events": ["Bash"],
                    }
                )
            )
            with self.assertRaisesRegex(benchmark.ValidationError, "policy"):
                benchmark.verify_receipt(run, receipt)

    def test_timeout_writes_a_terminal_attempt_record(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            executable = root / "traecli"
            executable.write_text(
                "#!/usr/bin/env python3\n"
                "import sys, time\n"
                "if sys.argv[1:] == ['--version']:\n"
                "    print('traecli 0.200.19')\n"
                "    raise SystemExit(0)\n"
                "time.sleep(3)\n"
            )
            executable.chmod(0o755)
            prompt = root / "prompt.md"
            prompt.write_text("prove\n")
            run = root / "run"

            with mock.patch.object(
                benchmark, "TRAE_DARWIN_ARM64_SHA256", benchmark.sha256_file(executable)
            ):
                runtime = root / "runtime"
                benchmark.provision_pinned_cli(executable, runtime)
                result = benchmark.run_attempt(
                    runtime_root=runtime,
                    workspace=root,
                    prompt_path=prompt,
                    run_dir=run,
                    timeout=0.01,
                )

            self.assertEqual(result["terminal_state"], "timed_out")
            self.assertIsNone(result["returncode"])
            self.assertTrue((run / "attempt.json").is_file())
            self.assertRegex(result["events_sha256"], r"^[0-9a-f]{64}$")

    def test_receipt_rejects_a_symlinked_parent(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            target = root / "target"
            target.mkdir()
            link = root / "link"
            link.symlink_to(target, target_is_directory=True)

            with self.assertRaisesRegex(benchmark.ValidationError, "symlink"):
                benchmark._prepare_new_file(link / "receipt.json")

    def test_receipt_rejects_a_symlinked_run_directory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            run = root / "run"
            run.mkdir()
            link = root / "run-link"
            link.symlink_to(run, target_is_directory=True)

            with self.assertRaisesRegex(benchmark.ValidationError, "run directory"):
                benchmark.export_receipt(link, root / "receipt.json")

    def test_provisioned_runtime_resolves_only_the_harp_owned_cli_copy(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "source-traecli"
            source.write_text("#!/bin/sh\nprintf 'traecli 0.200.19\\n'\n")
            source.chmod(0o755)
            runtime = root / "runtime"
            with mock.patch.object(
                benchmark, "TRAE_DARWIN_ARM64_SHA256", benchmark.sha256_file(source)
            ):
                installed = benchmark.provision_pinned_cli(source, runtime)
                resolved = benchmark.resolve_pinned_cli(runtime)

            self.assertEqual(installed, runtime / "traecli")
            self.assertEqual(resolved, installed)
            self.assertNotEqual(resolved, source)
