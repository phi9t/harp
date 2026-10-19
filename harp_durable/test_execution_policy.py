"""Execution policy boundaries using real capsule records, files and argv.

Preparation fixtures contain inert executable bytes and run no providers. The
Linux preparation case requires real delegation; it does not claim payload or
provider qualification.
"""
import copy
import json
import os
from pathlib import Path
import stat
import sys
import tempfile
import unittest
from unittest.mock import patch

from controller.durable.engine import Activity
from controller.durable.insula import InsulaExecution
from tools.insula import capsule
from .setup import prepare


def metadata_fixture(name):
    """A metadata-only executable; never represents a real provider result."""
    if name == "codex-code-mode-host":
        version, help_text = "host fixture", "Usage: codex-code-mode-host --listen"
    else:
        version = "codex-cli 0.162.1" if name == "codex" else "traecli 0.210.1"
        help_text = ("--ignore-user-config --ignore-rules --ephemeral "
                     "--dangerously-bypass-approvals-and-sandbox --json --output-last-message "
                     "--cd --model --color --disable --enable --no-daemon -c")
    return ("#!/bin/sh\ncase \" $* \" in *' --version '*) printf '%s\\n' '" + version +
            "';; *) printf '%s\\n' '" + help_text + "';; esac\nexit 0\n").encode()


class PolicyFixture(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.rootfs = self.root / "native"
        (self.rootfs / "usr/bin").mkdir(parents=True)
        (self.rootfs / "bin").symlink_to("usr/bin")
        for name in ("python3", "git", "sh"):
            path = self.rootfs / "usr/bin" / name
            path.write_bytes(b"inert preparation fixture; never executed")
            path.chmod(0o755)
        (self.rootfs / "etc").mkdir()
        (self.rootfs / "etc/resolv.conf").write_text("nameserver 127.0.0.1\n")
        self.providers = self.root / "providers"
        self.providers.mkdir(mode=0o700)
        for name in ("codex", "codex-code-mode-host", "trae"):
            path = self.providers / name
            path.write_bytes(metadata_fixture(name) + b"#" + b"x" * (1 << 20) + b"tail")
            path.chmod(0o755)
        self.credentials = {}
        for name in ("codex", "trae"):
            path = self.root / (name + "-auth.json")
            path.write_text(json.dumps({"fixture": name}))
            path.chmod(0o600)
            self.credentials[name] = path
        self.state = self.root / "state"
        self.execution_file = self.root / "execution.json"
        self.store = self.root / "capsules"
        self.store_environment = patch.dict(os.environ, {"CORENIUS_CAPSULES": str(self.store)})
        self.store_environment.start()
        self.addCleanup(self.store_environment.stop)

    def execution(self):
        request = self.root / "capsule.json"
        request.write_text(json.dumps({"layers": [
            {"kind": "rootfs", "name": "native", "source": str(self.rootfs)},
            {"kind": "tree", "name": "providers", "copy": [
                {"from": str(self.providers / name), "to": name} for name in ("codex", "trae")],
             "file_binds": [{"src": name, "dest": "/cap/providers/" + name}
                            for name in ("codex", "trae")]},
            {"kind": "host-ro", "name": "provider-credentials", "paths": [], "binds": [
                {"src": str(path), "dest": f"/home/bench/credentials/{name}-auth.json"}
                for name, path in self.credentials.items()]},
        ], "policy": {"net": "host", "gpu": False, "proxy": False}}))
        record = capsule.build(str(request))
        code = self.root / "code"
        code.mkdir()
        (code / "registry.py").write_text("REGISTRY = {}\n")
        return InsulaExecution.create(record, code, self.state)

    def payload(self, execution, provider=None, *, offline=False):
        directory = self.root / "payload"
        directory.mkdir(exist_ok=True)
        activity = Activity(argv=("/usr/bin/true",), offline=offline)
        request = {"attempt_dir": str(directory), "input": {"provider": provider}, "spec": activity.spec()}
        return execution.payload(directory, request)


class ExecutionPolicy(PolicyFixture):
    def test_offline_activity_narrows_host_network_and_preserves_legacy_spec(self):
        execution = self.execution()
        ordinary, _ = self.payload(execution)
        offline, projected = self.payload(execution, offline=True)
        self.assertNotIn("--unshare-net", ordinary)
        self.assertIn("--unshare-net", offline)
        self.assertTrue(projected["spec"]["offline"])
        self.assertEqual(execution.config["capsule"]["policy"]["net"], "host")
        self.assertNotIn("offline", Activity(argv=("/usr/bin/true",)).spec())
        for invalid in ("true", 1, None):
            with self.subTest(value=invalid), self.assertRaises(ValueError):
                Activity(argv=("/usr/bin/true",), offline=invalid).spec()

    def test_selected_credentials_exclude_the_other_provider_and_verifier_has_none(self):
        execution = self.execution()
        original = copy.deepcopy(execution.config)
        for provider in ("codex", "trae", None, "unrecognized"):
            with self.subTest(provider=provider):
                argv, request = self.payload(execution, provider, offline=provider is None)
                bindings = [(argv[i + 1], argv[i + 2]) for i, value in enumerate(argv) if value == "--ro-bind"]
                for name, path in self.credentials.items():
                    expected = (str(path), f"/home/bench/credentials/{name}-auth.json")
                    if provider == name:
                        self.assertIn(expected, bindings)
                    else:
                        self.assertNotIn(expected, bindings)
                        self.assertNotIn(str(path), argv)
                self.assertNotIn("execution", request)
                self.assertEqual(execution.config, original)

    def test_changed_sealed_provider_bytes_beyond_shallow_prefix_are_refused(self):
        execution = self.execution()
        execution.verify()
        layer = next(layer for layer in execution.config["capsule"]["layers"] if layer["name"] == "providers")
        pinned = self.store / "layers" / layer["digest"] / "tree/codex"
        original = pinned.read_bytes()
        info = pinned.stat()
        pinned.write_bytes(original[:-1] + b"!")
        os.utime(pinned, ns=(info.st_atime_ns, info.st_mtime_ns))
        self.assertEqual(pinned.read_bytes()[:1 << 20], original[:1 << 20])
        self.assertEqual(pinned.stat().st_size, info.st_size)
        self.assertEqual((self.providers / "codex").read_bytes(), original)
        for require_backend in (True, False):
            with self.subTest(inspection=not require_backend), self.assertRaises(ValueError):
                execution.verify(require_backend=require_backend)

    def test_provider_setup_requires_explicit_network_and_private_regular_credentials(self):
        def refuse(**options):
            with self.assertRaises(ValueError):
                prepare(self.state, self.rootfs, self.root / "not-delegated", self.execution_file, **options)
            self.assertFalse(self.execution_file.exists())
            self.assertFalse(self.execution_file.with_name("execution.json.native.json").exists())
            self.assertFalse((self.state / "runtime.sqlite").exists())
        refuse(providers_bin=self.providers)
        refuse(credentials={"codex": self.credentials["codex"]})
        refuse(network="host", credentials={"codex": self.credentials["codex"]})
        self.credentials["codex"].chmod(0o644)
        refuse(network="host", providers_bin=self.providers, credentials=self.credentials)
        self.credentials["codex"].chmod(0o600)
        alias = self.root / "credential-link"
        alias.symlink_to(self.credentials["codex"])
        refuse(network="host", providers_bin=self.providers, credentials={"codex": alias})
        refuse(network="host", providers_bin=self.providers, credentials={"unknown": self.credentials["codex"]})


@unittest.skipUnless(sys.platform == "linux" and os.environ.get("HARP_TEST_CGROUP_ROOT"),
                     "requires an explicit disposable delegated Linux cgroup for prepare")
class PreparedProviderPolicy(PolicyFixture):
    def setUp(self):
        super().setUp()
        if not os.environ.get("HARP_TEST_ROOTFS"):
            self.skipTest("metadata probes require an admitted executable native rootfs")
        self.rootfs = Path(os.environ["HARP_TEST_ROOTFS"])

    def test_exact_prepare_reuses_record_but_changed_source_binary_or_mode_does_not(self):
        options = {"network": "host", "providers_bin": self.providers, "credentials": self.credentials}
        args = (self.state, self.rootfs, Path(os.environ["HARP_TEST_CGROUP_ROOT"]), self.execution_file)
        config = prepare(*args, **options)
        record = self.execution_file.read_bytes()
        self.assertEqual(prepare(*args, **options), config)
        self.assertEqual(self.execution_file.read_bytes(), record)
        source = self.providers / "codex"
        original, info = source.read_bytes(), source.stat()
        source.write_bytes(original[:-1] + b"!")
        os.utime(source, ns=(info.st_atime_ns, info.st_mtime_ns))
        with self.assertRaises(ValueError):
            prepare(*args, **options)
        self.assertEqual(self.execution_file.read_bytes(), record)
        source.write_bytes(original)
        source.chmod(stat.S_IMODE(info.st_mode) ^ stat.S_IXGRP)
        with self.assertRaises(ValueError):
            prepare(*args, **options)
        self.assertEqual(self.execution_file.read_bytes(), record)


if __name__ == "__main__":
    unittest.main()
