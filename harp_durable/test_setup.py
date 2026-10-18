"""Native admission rejects changed bytes, links and incomplete boundaries."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import stat
import tempfile
import unittest

from harp_durable.setup import native_manifest, prepare, verify_native_manifest, verify_required_executables

ROOT = Path(__file__).resolve().parents[1]


class NativeAdmission(unittest.TestCase):
    def setUp(self):
        scratch = ROOT / ".scratch" / "durable-setup-tests"
        scratch.mkdir(parents=True, exist_ok=True)
        self.temporary = tempfile.TemporaryDirectory(dir=scratch)
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.rootfs = self.root / "rootfs"
        (self.rootfs / "usr/bin").mkdir(parents=True)
        (self.rootfs / "usr/bin/tool").write_bytes(b"before")
        (self.rootfs / "usr/bin/tool").chmod(0o755)
        (self.rootfs / "bin").symlink_to("usr/bin")
        for name in ("python3.11", "git", "sh"):
            executable = self.rootfs / "usr/bin" / name
            executable.write_bytes(b"fixture-executable")
            executable.chmod(0o755)
        (self.rootfs / "usr/bin/python3").symlink_to("python3.11")
        self.previous_store = os.environ.get("CORENIUS_CAPSULES")
        self.addCleanup(self.restore_environment)

    def restore_environment(self):
        if self.previous_store is None:
            os.environ.pop("CORENIUS_CAPSULES", None)
        else:
            os.environ["CORENIUS_CAPSULES"] = self.previous_store

    def bind(self):
        manifest = native_manifest(self.rootfs)
        path = self.root / "execution.native.json"
        data = json.dumps(manifest, sort_keys=True, separators=(",", ":")).encode()
        path.write_bytes(data)
        return {"path": str(path), "rootfs": str(self.rootfs),
                "sha256": hashlib.sha256(data).hexdigest()}

    def test_file_bytes_are_pinned_even_when_size_mode_and_mtime_are_unchanged(self):
        binding = self.bind()
        verify_native_manifest(binding)
        tool = self.rootfs / "usr/bin/tool"
        before = tool.stat()
        tool.write_bytes(b"after!")
        os.utime(tool, ns=(before.st_atime_ns, before.st_mtime_ns))
        self.assertEqual(tool.stat().st_size, before.st_size)
        self.assertEqual(stat.S_IMODE(tool.stat().st_mode), stat.S_IMODE(before.st_mode))
        with self.assertRaisesRegex(ValueError, "native rootfs changed"):
            verify_native_manifest(binding)

    def test_links_directories_modes_and_new_files_are_part_of_native_closure(self):
        binding = self.bind()
        link = self.rootfs / "bin"
        link.unlink()
        link.symlink_to("/usr/bin")
        with self.assertRaisesRegex(ValueError, "native rootfs changed"):
            verify_native_manifest(binding)
        binding = self.bind()
        (self.rootfs / "usr/bin").chmod(0o700)
        with self.assertRaisesRegex(ValueError, "native rootfs changed"):
            verify_native_manifest(binding)
        binding = self.bind()
        (self.rootfs / "usr/bin/new").write_bytes(b"new")
        with self.assertRaisesRegex(ValueError, "native rootfs changed"):
            verify_native_manifest(binding)

    def test_bytecode_is_included_and_symlinks_are_not_followed_outside_rootfs(self):
        outside = self.root / "outside"
        outside.write_bytes(b"secret-not-an-input")
        (self.rootfs / "external").symlink_to(str(outside))
        (self.rootfs / "cache.pyc").write_bytes(b"compiled")
        binding = self.bind()
        outside.write_bytes(b"changed-outside")
        verify_native_manifest(binding)
        (self.rootfs / "cache.pyc").write_bytes(b"modified")
        with self.assertRaisesRegex(ValueError, "native rootfs changed"):
            verify_native_manifest(binding)

    def test_replaced_manifest_and_special_files_are_refused(self):
        binding = self.bind()
        Path(binding["path"]).write_bytes(b"{}")
        with self.assertRaisesRegex(ValueError, "manifest binding changed"):
            verify_native_manifest(binding)
        os.mkfifo(self.rootfs / "fifo")
        with self.assertRaisesRegex(ValueError, "special file"):
            native_manifest(self.rootfs)

    def test_symlinked_rootfs_is_refused(self):
        alias = self.root / "alias"
        alias.symlink_to(self.rootfs)
        with self.assertRaisesRegex(ValueError, "ordinary directory"):
            native_manifest(alias)

    def test_prepare_refuses_non_kernel_cgroup_without_execution_record(self):
        fake_cgroup = self.root / "not-a-cgroup"
        fake_cgroup.mkdir()
        state = self.root / "state"
        execution = self.root / "execution.json"
        with self.assertRaisesRegex(RuntimeError, "cgroup"):
            prepare(state, self.rootfs, fake_cgroup, execution)
        self.assertFalse(execution.exists())
        self.assertFalse((state / "runtime.sqlite").exists())

    def test_missing_python_alias_refuses_prepare_before_execution_record(self):
        (self.rootfs / "usr/bin/python3").unlink()
        execution = self.root / "execution.json"
        with self.assertRaisesRegex(ValueError, "invalid rootfs executable /usr/bin/python3"):
            prepare(self.root / "state", self.rootfs, self.root, execution)
        self.assertFalse(execution.exists())
        self.assertFalse(execution.with_name("execution.json.native.json").exists())

    def test_executable_absolute_links_resolve_inside_namespace(self):
        alias = self.rootfs / "usr/bin/python3"
        alias.unlink()
        alias.symlink_to("/usr/bin/python3.11")
        verify_required_executables(self.rootfs)
        (self.rootfs / "usr/bin/python3.11").unlink()
        with self.assertRaisesRegex(ValueError, "invalid rootfs executable /usr/bin/python3"):
            verify_required_executables(self.rootfs)

    def test_non_executable_and_directory_targets_are_refused(self):
        git = self.rootfs / "usr/bin/git"
        git.chmod(0o644)
        with self.assertRaisesRegex(ValueError, "executable regular file"):
            verify_required_executables(self.rootfs)
        git.unlink()
        git.mkdir(mode=0o755)
        with self.assertRaisesRegex(ValueError, "executable regular file"):
            verify_required_executables(self.rootfs)

    def test_executable_symlink_escape_and_loop_are_refused(self):
        alias = self.rootfs / "usr/bin/python3"
        alias.unlink()
        alias.symlink_to("../../../outside")
        with self.assertRaisesRegex(ValueError, "escapes the rootfs"):
            verify_required_executables(self.rootfs)
        alias.unlink()
        alias.symlink_to("python3")
        with self.assertRaisesRegex(ValueError, "symlink loop"):
            verify_required_executables(self.rootfs)

    def test_prepare_creates_private_inputs_before_delegation_check(self):
        state = self.root / "state"
        with self.assertRaisesRegex(RuntimeError, "cgroup"):
            prepare(state, self.rootfs, self.root, self.root / "execution.json")
        self.assertEqual(stat.S_IMODE((state / "inputs").stat().st_mode), 0o700)
        self.assertEqual(list((state / "inputs").iterdir()), [])

    def test_prepare_does_not_replace_existing_inputs(self):
        state = self.root / "state"
        state.mkdir(mode=0o700)
        inputs = state / "inputs"
        inputs.write_bytes(b"existing-file")
        with self.assertRaises(OSError):
            prepare(state, self.rootfs, self.root, self.root / "execution.json")
        self.assertEqual(inputs.read_bytes(), b"existing-file")
        inputs.unlink()
        outside = self.root / "outside"
        outside.mkdir(mode=0o700)
        inputs.symlink_to(outside)
        with self.assertRaisesRegex(ValueError, "symlinked ancestry"):
            prepare(state, self.rootfs, self.root, self.root / "execution.json")
        self.assertTrue(inputs.is_symlink())
        self.assertEqual(list(outside.iterdir()), [])

    def test_prepare_rejects_nonprivate_existing_input_directory(self):
        state = self.root / "state"
        state.mkdir(mode=0o700)
        inputs = state / "inputs"
        inputs.mkdir(mode=0o755)
        with self.assertRaisesRegex(ValueError, "private directory"):
            prepare(state, self.rootfs, self.root, self.root / "execution.json")
        self.assertEqual(stat.S_IMODE(inputs.stat().st_mode), 0o755)

    def test_prepare_refuses_public_state_without_mutating_it(self):
        state = self.root / "state"
        state.mkdir(mode=0o755)
        with self.assertRaisesRegex(ValueError, "private directory"):
            prepare(state, self.rootfs, self.root, self.root / "execution.json")
        self.assertEqual(list(state.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
