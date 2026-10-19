"""Offline namespace metadata fixtures; these tests never qualify model calls."""
import copy
import json
import os
from pathlib import Path
import sys
import tempfile
import time
import unittest
from unittest.mock import patch

from insula import capsule
import durable.jobs.coding.provider_preflight as preflight


@unittest.skipUnless(sys.platform == "linux" and os.environ.get("HARP_TEST_ROOTFS"),
                     "requires an admitted Linux rootfs and working bubblewrap namespaces")
class ProviderPreflight(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.environment = patch.dict(os.environ, {"INSULA_CAPSULES": str(self.root / "capsules")})
        self.environment.start()
        self.addCleanup(self.environment.stop)
        self.private = self.root / "operator-private"
        self.private.mkdir(mode=0o700)
        (self.private / "owner.json").write_text('{"token":"not-projectable"}')
        self.providers = self.root / "providers"
        self.providers.mkdir()

    def record(self, mode="good"):
        host_net = os.readlink("/proc/self/ns/net")
        for name in ("codex", "trae", "codex-code-mode-host"):
            source = """#!/usr/bin/python3
import os,pathlib,sys,time
name=NAME
mode=MODE
assert os.environ['HOME'].startswith('/home/bench/run/')
assert os.readlink('/proc/self/ns/net') != HOST_NET
assert not pathlib.Path(PRIVATE).exists()
assert not pathlib.Path('/home/bench/credentials').exists()
assert 'LEAK_AUTH' not in os.environ
assert not pathlib.Path(os.environ['HOME'],'.codex','auth.json').exists()
pathlib.Path(os.environ['HOME'],'metadata-only').write_text('private scratch')
try: pathlib.Path('/workspace/unexpected').write_text('write')
except OSError: pass
else: raise RuntimeError('workspace must be readonly')
if name=='codex' and mode=='nonzero': sys.exit(17)
if name=='codex' and mode=='oversize': print('x'*70000);sys.exit(0)
if name=='codex' and mode=='hang': time.sleep(60)
if '--version' in sys.argv:
 print('unsupported' if mode=='badversion' else ('codex-cli 0.162.1' if name=='codex' else 'traecli 0.210.1\\ninstallation: internal'))
else:
 flags=FLAGS
 if mode=='badhelp' and name=='codex': flags.remove('--json')
 print('Options:\\n'+'\\n'.join(flags))
"""
            flags = ["--ignore-user-config", "--ignore-rules", "--ephemeral",
                     "--dangerously-bypass-approvals-and-sandbox", "--json", "--output-last-message",
                     "--cd", "--model", "--color", "--no-daemon", "--disable", "--enable", "-c", "--listen"]
            for key, value in (("NAME", name), ("MODE", mode), ("HOST_NET", host_net),
                               ("PRIVATE", str(self.private)), ("FLAGS", flags)):
                source = source.replace(key, repr(value))
            path = self.providers / name
            path.write_text(source)
            path.chmod(0o755)
        request = self.root / "capsule.json"
        request.write_text(json.dumps({"layers": [
            {"kind": "rootfs", "name": "native", "source": os.environ["HARP_TEST_ROOTFS"]},
            {"kind": "tree", "name": "providers", "source": str(self.providers), "file_binds": [
                {"src": name, "dest": "/cap/providers/" + name}
                for name in ("codex", "trae", "codex-code-mode-host")]},
            {"kind": "host-ro", "name": "provider-credentials", "paths": [], "binds": [
                {"src": str(self.private / "owner.json"), "dest": "/home/bench/credentials/codex-auth.json"}]},
            {"kind": "host-ro", "name": "operator", "paths": [str(self.private)]}],
            "policy": {"net": "host", "gpu": False, "proxy": False}, "env": {"LEAK_AUTH": "must-not-project"}}))
        return capsule.build(str(request))

    def directories(self, label):
        workspace, scratch = self.root / (label + "-workspace"), self.root / (label + "-scratch")
        workspace.mkdir(mode=0o700)
        scratch.mkdir(mode=0o700)
        return workspace, scratch

    def test_metadata_is_versioned_bounded_and_does_not_project_authority(self):
        record = self.record()
        original = copy.deepcopy(record)
        workspace, scratch = self.directories("valid")
        descriptors = preflight.probe_providers(record, workspace, scratch)
        self.assertEqual(descriptors["codex"]["version"], "0.162.1")
        self.assertEqual(descriptors["trae"]["version"], "0.210.1")
        self.assertIn("--json", descriptors["codex"]["capabilities"])
        self.assertEqual(descriptors["codex"]["companions"][0]["path"], "/cap/providers/codex-code-mode-host")
        self.assertLess(len(json.dumps(descriptors)), 4096)
        self.assertNotIn("not-projectable", json.dumps(descriptors))
        self.assertEqual(record, original)
        self.assertEqual(list(workspace.iterdir()), [])
        preflight.verify_providers(record, descriptors)
        changed = copy.deepcopy(descriptors)
        changed["codex"]["binary_sha256"] = "0" * 64
        with self.assertRaises(ValueError):
            preflight.verify_providers(record, changed)
        changed = copy.deepcopy(descriptors)
        changed["trae"]["capabilities"].remove("--json")
        with self.assertRaises(ValueError):
            preflight.verify_providers(record, changed)
        layer = next(item for item in record["layers"] if item.get("name") == "providers")
        sealed = capsule.store_root() / "layers" / layer["digest"] / "tree/codex-code-mode-host"
        sealed.write_bytes(sealed.read_bytes() + b"\n# changed")
        with self.assertRaises(ValueError):
            preflight.verify_providers(record, descriptors)

    def test_incompatible_failed_and_oversized_metadata_are_refused(self):
        for mode in ("badhelp", "badversion", "nonzero", "oversize"):
            with self.subTest(mode=mode), self.assertRaises(ValueError):
                preflight.probe_providers(self.record(mode), *self.directories(mode))

    def test_hanging_metadata_is_bounded_and_launcher_is_cleaned(self):
        record = self.record("hang")
        started = time.monotonic()
        with patch.object(preflight, "TIMEOUT_SECONDS", 0.3), self.assertRaises(ValueError):
            preflight.probe_providers(record, *self.directories("hang"))
        self.assertLess(time.monotonic() - started, 3)
        for path in Path("/proc").glob("[0-9]*/cmdline"):
            try:
                command = path.read_bytes()
            except OSError:
                continue
            self.assertFalse(b"bwrap" in command and os.fsencode(self.root) in command,
                             "owned metadata launcher survived its timeout")

    def test_nonempty_projection_and_missing_companion_refuse(self):
        record = self.record()
        workspace, scratch = self.directories("nonempty")
        (scratch / "owner.json").write_text("must not project existing state")
        with self.assertRaises(ValueError):
            preflight.probe_providers(record, workspace, scratch)
        record = copy.deepcopy(record)
        layer = next(item for item in record["layers"] if item.get("name") == "providers")
        layer["file_binds"] = [item for item in layer["file_binds"] if item["src"] != "codex-code-mode-host"]
        with self.assertRaises(ValueError):
            preflight.probe_providers(record, *self.directories("missing"))


if __name__ == "__main__":
    unittest.main()
