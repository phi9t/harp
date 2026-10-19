"""A real admitted payload must start with provider mounts on a readonly rootfs."""
import json
import os
from pathlib import Path
import sys
import unittest

from . import test_admission as admission_tests
from .test_execution_policy import metadata_fixture


@unittest.skipUnless(sys.platform == "linux" and all(os.environ.get(name) for name in
                     ("HARP_TEST_CGROUP_ROOT", "HARP_TEST_ROOTFS", "HARP_DURABLE_BIN")),
                     "requires public Harp binary, admitted native rootfs and delegated Linux cgroup")
class ProviderMounts(unittest.TestCase):
    setUp = admission_tests.CodingLifecycle.setUp
    git = admission_tests.GitInputs.git
    call = admission_tests.CodingLifecycle.call
    admit = admission_tests.CodingLifecycle.admit

    def test_sealed_provider_mounts_start_without_mutating_native_rootfs(self):
        original_state, original_execution = self.state, self.execution
        self.state = self.root / "provider-state"
        self.execution = self.root / "provider-execution.json"
        self.command = [str(self.state) if item == str(original_state) else
                        str(self.execution) if item == str(original_execution) else item for item in self.command]
        providers = self.root / "provider-bin"
        providers.mkdir()
        auth = self.root / "private-auth"
        auth.mkdir(mode=0o700)
        for name in ("codex", "codex-code-mode-host", "trae"):
            (providers / name).write_bytes(metadata_fixture(name))
            (providers / name).chmod(0o755)
        for name in ("codex", "trae"):
            (auth / name).write_text(json.dumps({"fixture": name}))
            (auth / name).chmod(0o600)
        profile = self.call("prepare", "--rootfs", os.environ["HARP_TEST_ROOTFS"],
                            "--cgroup-root", os.environ["HARP_TEST_CGROUP_ROOT"],
                            "--network", "host", "--providers-bin", str(providers),
                            "--codex-auth", str(auth / "codex"), "--trae-auth", str(auth / "trae"))
        checks = [["/usr/bin/python3", "-c",
                   "from pathlib import Path; "
                   "assert Path('/cap/providers/codex').read_bytes().startswith(b'#!/bin/sh'); "
                   "assert Path('/cap/providers/trae').read_bytes().startswith(b'#!/bin/sh'); "
                   "assert not Path('/home/bench/credentials/codex-auth.json').exists(); "
                   "assert not Path('/home/bench/credentials/trae-auth.json').exists()"]]
        (self.repo / "acceptance.json").write_text(json.dumps({"checks": checks}))
        self.git("add", "acceptance.json")
        self.git("commit", "-qm", "Pin provider mount smoke checks")
        self.admit("mounts")
        self.call("start", "--capacity", "1", "--until-idle", "--timeout", "30")
        coding = self.call("status", "mounts")
        self.assertEqual(coding["status"], "SUCCEEDED", coding)
        verification = self.call("verify", "mounts")
        self.call("start", "--capacity", "1", "--until-idle", "--timeout", "30")
        result = self.call("status", verification["run_id"])
        self.assertEqual(result["status"], "SUCCEEDED", result)
        self.assertTrue(result["result"]["passed"], result)
        # All native bytes still agree with the admitted closure after mounting.
        from .setup import verify_native_manifest
        verify_native_manifest(profile["native_manifest"])
