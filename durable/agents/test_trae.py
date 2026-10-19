"""Process fixtures validate the Trae invocation contract, not live inference."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import stat
import subprocess
import sys
import tempfile
import unittest

ROOT = next(p for p in Path(__file__).resolve().parents if (p / "Cargo.toml").is_file() and (p / "durable").is_dir())
FIXTURE = '''#!{python}
import hashlib,json,os,pathlib,sys
args=sys.argv[1:]
home=pathlib.Path(os.environ["TRAE_HOME"])
auth=pathlib.Path(os.environ["TRAECLI_HOME"])/"auth.json"
record={{"argv":args,"task":sys.stdin.read(),"cwd":os.getcwd(),
        "home":os.environ["HOME"],"trae_home":str(home),
        "cli_home":os.environ["TRAECLI_HOME"],"codex_home":os.environ["CODEX_HOME"],
        "auth_sha256":hashlib.sha256(auth.read_bytes()).hexdigest(),
        "auth_mode":auth.stat().st_mode & 0o777,
        "copied_user_config":(home/"traecli.toml").exists()}}
pathlib.Path(os.environ["HARP_TRAE_RECORD"]).write_text(json.dumps(record))
auth.write_text('{{"refreshed":true}}')
print("provider-json-event")
print("provider-stderr-event",file=sys.stderr)
if os.environ.get("HARP_TRAE_FINAL","yes")=="yes":
 kind=os.environ.get("HARP_TRAE_OUTPUT_KIND","text")
 content={{"text":b"completed fixture task\\n","empty":b"","oversize":b"x"*(256*1024+1),
          "binary":b"\\xff","whitespace":b" \\n"}}[kind]
 pathlib.Path(args[args.index("--output-last-message")+1]).write_bytes(content)
sys.exit(int(os.environ.get("HARP_TRAE_EXIT","0")))
'''
WRAPPER = '''import json,os,pathlib,sys
from durable.agents.providers import trae
trae.EXECUTABLE=pathlib.Path(sys.argv[1]); trae.AUTH=pathlib.Path(sys.argv[2])
a=pathlib.Path(sys.argv[3]); model=json.loads(sys.argv[4])
result=trae.run(a/"workspace",sys.argv[5],{"model":model,"provider":os.environ.get("HARP_TRAE_PROVIDER","trae")},a)
(a/"completion.json").write_text(json.dumps(result))
'''


class TraeInvocation(unittest.TestCase):
    def setUp(self):
        scratch = ROOT / ".scratch" / "durable-trae-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.attempt = self.root / "attempt"
        self.attempt.mkdir(mode=0o700)
        (self.attempt / "workspace").mkdir()
        self.auth = self.root / "selected-auth.json"
        self.secret = "fixture-secret-should-not-appear-in-output"
        self.auth.write_text(json.dumps({"trae": {"token": self.secret}}))
        self.source_auth = self.auth.read_bytes()
        self.binary = self.root / "trae-fixture"
        self.binary.write_text(FIXTURE.format(python=sys.executable))
        self.binary.chmod(0o755)
        self.record = self.root / "invocation.json"
        self.task = "Implement the task.\nKeep literal $(not-a-shell) and `text`.\n"
        self.env = {**os.environ, "PYTHONPATH": str(ROOT),
                    "PYTHONDONTWRITEBYTECODE": "1", "PYTHONNOUSERSITE": "1",
                    "HARP_TRAE_RECORD": str(self.record), "TRAE_HOME": "/not-the-attempt",
                    "TRAECLI_HOME": "/not-the-attempt/cli", "CODEX_HOME": "/unrelated-config"}

    def call(self, model=None, **environment):
        return subprocess.run([sys.executable, "-c", WRAPPER, str(self.binary), str(self.auth),
                               str(self.attempt), json.dumps(model), self.task],
                              env={**self.env, **environment}, cwd=ROOT,
                              text=True, capture_output=True, timeout=20)

    def test_fresh_process_receives_exact_stdin_private_auth_and_completion_artifact(self):
        result = self.call()
        self.assertEqual(result.returncode, 0, result.stderr)
        record = json.loads(self.record.read_text())
        self.assertEqual(record["task"], self.task)
        self.assertEqual(record["cwd"], str(self.attempt / "workspace"))
        self.assertEqual(record["home"], str(self.attempt / "home"))
        self.assertEqual(record["trae_home"], str(self.attempt / "home/.trae"))
        self.assertEqual(record["cli_home"], str(self.attempt / "home/.trae/cli"))
        self.assertEqual(record["codex_home"], record["trae_home"])
        self.assertEqual(record["auth_mode"], 0o600)
        self.assertFalse(record["copied_user_config"])
        self.assertEqual(self.auth.read_bytes(), self.source_auth)
        self.assertEqual(json.loads((self.attempt / "home/.trae/cli/auth.json").read_text()),
                         {"refreshed": True})
        self.assertIn("provider-json-event", result.stdout)
        self.assertIn("provider-stderr-event", result.stderr)
        self.assertNotIn(self.secret, result.stdout + result.stderr)
        final = self.attempt / "trae-final.txt"
        completion = json.loads((self.attempt / "completion.json").read_text())
        self.assertEqual(completion["final_sha256"], hashlib.sha256(final.read_bytes()).hexdigest())
        self.assertEqual(completion["final_size"], final.stat().st_size)
        self.assertEqual(completion["provider"], "trae")
        args = record["argv"]
        self.assertEqual(args[0], "exec")
        self.assertEqual(args[-1], "-")
        self.assertNotIn("--model", args)
        for flag in ("--ephemeral", "--json", "--ignore-user-config", "--ignore-rules",
                     "--dangerously-bypass-approvals-and-sandbox"):
            self.assertIn(flag, args)
        self.assertIn('model_provider="trae"', args)
        self.assertNotIn(self.task, args)
        self.assertEqual(stat.S_IMODE((self.attempt / "home").stat().st_mode), 0o700)

    def test_admitted_model_is_one_literal_argument(self):
        model = "fixture model; $(not-executed)"
        result = self.call(model)
        self.assertEqual(result.returncode, 0, result.stderr)
        args = json.loads(self.record.read_text())["argv"]
        self.assertEqual(args[args.index("--model") + 1], model)

    def test_provider_failure_refuses_completion_even_with_final_file(self):
        result = self.call(HARP_TRAE_EXIT="19")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("exited with status 19", result.stderr)
        self.assertFalse((self.attempt / "completion.json").exists())
        self.assertTrue((self.attempt / "trae-final.txt").exists())
        self.assertNotIn(self.secret, result.stdout + result.stderr)

    def test_success_without_final_artifact_refuses_completion(self):
        result = self.call(HARP_TRAE_FINAL="no")
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse((self.attempt / "completion.json").exists())

    def test_empty_oversized_and_binary_finals_refuse_completion(self):
        for kind in ("empty", "oversize", "binary", "whitespace"):
            with self.subTest(kind=kind):
                # Each case is a new attempt; adapters never resume an old home.
                original_attempt = self.attempt
                self.attempt = self.root / ("attempt-" + kind)
                self.attempt.mkdir(mode=0o700)
                (self.attempt / "workspace").mkdir()
                try:
                    result = self.call(HARP_TRAE_OUTPUT_KIND=kind)
                    self.assertNotEqual(result.returncode, 0)
                    self.assertIn("Trae final artifact", result.stderr)
                    self.assertFalse((self.attempt / "completion.json").exists())
                    self.assertNotIn(self.secret, result.stdout + result.stderr)
                finally:
                    self.attempt = original_attempt

    def test_mismatched_provider_refuses_before_creating_home_or_forking(self):
        result = self.call(HARP_TRAE_PROVIDER="codex")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("requires a Trae admission", result.stderr)
        self.assertFalse((self.attempt / "home").exists())
        self.assertFalse(self.record.exists())

    def test_second_invocation_refuses_provider_resume(self):
        self.assertEqual(self.call().returncode, 0)
        self.record.unlink()
        result = self.call()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("resume is unsupported", result.stderr)
        self.assertFalse(self.record.exists())

    def test_symlinked_private_home_refused_without_changing_target(self):
        outside = self.root / "outside"
        outside.mkdir(mode=0o700)
        (self.attempt / "home").symlink_to(outside)
        result = self.call()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("must not be symlinked", result.stderr)
        self.assertEqual(list(outside.iterdir()), [])
        self.assertFalse(self.record.exists())


if __name__ == "__main__":
    unittest.main()
