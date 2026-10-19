"""Real process fixtures for the adapter boundary; no live-provider qualification."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest


class CodexAdapterTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name).resolve()
        self.attempt = self.root / "attempt"
        self.attempt.mkdir(mode=0o700)
        self.workspace = self.attempt / "workspace"
        self.workspace.mkdir()
        self.auth = self.root / "selected-auth.json"
        self.auth.write_text('{"tokens":{"access_token":"fixture-selected"}}')
        self.auth.chmod(0o400)
        self.executable = self.root / "codex-fixture"
        self.host_home = self.root / "host-home"
        (self.host_home / ".codex").mkdir(parents=True)
        (self.host_home / ".codex/config.toml").write_text('model = "must-not-load"')

    def tearDown(self):
        self.temporary.cleanup()

    def executable_fixture(self, mode="success"):
        source = r'''import json, os, pathlib, sys
mode = MODE
argv = sys.argv[1:]
final = pathlib.Path(argv[argv.index('--output-last-message') + 1])
home = pathlib.Path(os.environ['CODEX_HOME'])
capture = {
    'argv': argv, 'input': sys.stdin.read(), 'cwd': os.getcwd(),
    'env': {key: os.environ.get(key) for key in (
        'HOME', 'CODEX_HOME', 'XDG_CONFIG_HOME', 'XDG_CACHE_HOME',
        'XDG_STATE_HOME', 'XDG_RUNTIME_DIR', 'TMPDIR', 'GIT_CONFIG_GLOBAL',
        'OPENAI_API_KEY', 'CODEX_CONFIG', 'PYTHONPATH')},
    'provider_files': sorted(path.name for path in home.iterdir()),
    'auth_mode': (home / 'auth.json').stat().st_mode & 0o777,
}
(final.parent / 'invocation.json').write_text(json.dumps(capture))
(home / 'auth.json').write_text('{"refreshed":"fixture-only"}')
print('{"type":"turn.completed","fixture":true}', flush=True)
print('provider stderr sentinel', file=sys.stderr, flush=True)
if mode == 'symlink':
    final.symlink_to(final.parent / 'invocation.json')
elif mode == 'oversize':
    final.write_bytes(b'x' * (256 * 1024 + 1))
elif mode == 'nonutf8':
    final.write_bytes(b'\xff')
elif mode == 'empty':
    final.write_text('  \n')
elif mode != 'missing':
    final.write_text('Completed fixture task.\n')
sys.exit(17 if mode == 'nonzero' else 0)
'''.replace("MODE", repr(mode))
        self.executable.write_text("#!" + sys.executable + "\n" + source)
        self.executable.chmod(0o700)

    def invoke(self, *, task="Frozen task\nQuotes ' \" $() and unicode λ", model=None):
        harness = """import json, pathlib, sys
from harp_durable.providers import codex
args = json.load(sys.stdin)
codex.EXECUTABLE = args['executable']
codex.AUTH = pathlib.Path(args['auth'])
result = codex.run(pathlib.Path(args['workspace']), args['task'],
                   {'provider': 'codex', 'model': args['model']}, pathlib.Path(args['attempt']))
print('ADAPTER_RESULT:' + json.dumps(result))
"""
        environment = dict(os.environ)
        environment.update(HOME=str(self.host_home), CODEX_HOME=str(self.host_home / ".codex"),
                           CODEX_CONFIG="host-config-must-not-leak", OPENAI_API_KEY="must-not-leak",
                           PYTHONPATH=str(Path(__file__).resolve().parents[2]),
                           PYTHONDONTWRITEBYTECODE="1")
        return subprocess.run([sys.executable, "-c", harness], input=json.dumps({
            "workspace": str(self.workspace), "attempt": str(self.attempt), "auth": str(self.auth),
            "executable": str(self.executable), "task": task, "model": model,
        }), text=True, capture_output=True, env=environment)

    def test_exact_prompt_private_state_and_streamed_logs(self):
        self.executable_fixture()
        task = "Frozen task\nOwn workspace only; acceptance is independently verified.\nλ ' \" $()"
        output = self.invoke(task=task)
        self.assertEqual(output.returncode, 0, output.stderr)
        self.assertIn('"type":"turn.completed"', output.stdout)
        self.assertIn("provider stderr sentinel", output.stderr)
        result = json.loads(output.stdout.split("ADAPTER_RESULT:", 1)[1])
        invocation = json.loads((self.attempt / "invocation.json").read_text())
        self.assertEqual(invocation["input"], task)
        self.assertEqual(invocation["cwd"], str(self.workspace))
        self.assertEqual(invocation["argv"], [
            "--no-daemon", "--enable", "code_mode_host", "--disable", "apps", "--disable", "plugins",
            "exec", "--ignore-user-config", "--ignore-rules", "--ephemeral",
            "--dangerously-bypass-approvals-and-sandbox", "--json", "--color", "never",
            "--cd", str(self.workspace), "--output-last-message", str(self.attempt / "codex-final.txt"), "-",
        ])
        self.assertEqual(invocation["provider_files"], ["auth.json"])
        self.assertEqual(invocation["auth_mode"], 0o600)
        self.assertEqual(invocation["env"]["HOME"], str(self.attempt / "home"))
        self.assertEqual(invocation["env"]["CODEX_HOME"], str(self.attempt / "home/.codex"))
        self.assertEqual(invocation["env"]["GIT_CONFIG_GLOBAL"], "/dev/null")
        for key in ("XDG_CONFIG_HOME", "XDG_CACHE_HOME", "XDG_STATE_HOME", "XDG_RUNTIME_DIR", "TMPDIR"):
            self.assertTrue(Path(invocation["env"][key]).is_relative_to(self.attempt))
        for key in ("OPENAI_API_KEY", "CODEX_CONFIG", "PYTHONPATH"):
            self.assertIsNone(invocation["env"][key])
        self.assertIn("fixture-selected", self.auth.read_text())
        self.assertNotIn("fixture-selected", (self.attempt / "home/.codex/auth.json").read_text())
        final = (self.attempt / "codex-final.txt").read_bytes()
        self.assertEqual(result["final"], {"path": "codex-final.txt", "size": len(final),
                                           "sha256": hashlib.sha256(final).hexdigest()})
        self.assertIsNone(result["model"])

    def test_explicit_model_is_one_argument(self):
        self.executable_fixture()
        model = "explicit model with spaces ' $(not-shell)"
        output = self.invoke(model=model)
        self.assertEqual(output.returncode, 0, output.stderr)
        invocation = json.loads((self.attempt / "invocation.json").read_text())
        self.assertEqual(invocation["argv"][-3:], ["--model", model, "-"])

    def test_nonzero_exit_refuses_even_with_completed_event_and_final_file(self):
        self.executable_fixture("nonzero")
        output = self.invoke()
        self.assertNotEqual(output.returncode, 0)
        self.assertIn('"type":"turn.completed"', output.stdout)
        self.assertTrue((self.attempt / "codex-final.txt").exists())
        self.assertNotIn("ADAPTER_RESULT:", output.stdout)
        self.assertIn("status 17", output.stderr)

    def test_missing_symlink_large_and_invalid_final_artifacts_refuse(self):
        for mode in ("missing", "symlink", "oversize", "nonutf8", "empty"):
            with self.subTest(mode=mode):
                attempt = self.root / mode
                attempt.mkdir(mode=0o700)
                self.attempt = attempt
                self.workspace = attempt / "workspace"
                self.workspace.mkdir()
                self.executable_fixture(mode)
                output = self.invoke()
                self.assertNotEqual(output.returncode, 0)
                self.assertNotIn("ADAPTER_RESULT:", output.stdout)

    def test_missing_or_symlinked_credentials_refuse_before_provider_execution(self):
        self.executable_fixture()
        original = self.auth.read_bytes()
        self.auth.unlink()
        output = self.invoke()
        self.assertNotEqual(output.returncode, 0)
        self.assertFalse((self.attempt / "invocation.json").exists())
        other = self.root / "other-auth.json"
        other.write_bytes(original)
        self.auth.symlink_to(other)
        output = self.invoke()
        self.assertNotEqual(output.returncode, 0)
        self.assertFalse((self.attempt / "invocation.json").exists())


if __name__ == "__main__":
    unittest.main()
