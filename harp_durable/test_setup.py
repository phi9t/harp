"""Native admission rejects changed bytes, links and incomplete boundaries."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import stat
import tempfile
import unittest

from harp_durable.setup import native_manifest, prepare, verify_native_manifest

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

    def test_prepare_refuses_public_state_without_mutating_it(self):
        state = self.root / "state"
        state.mkdir(mode=0o755)
        with self.assertRaisesRegex(ValueError, "private directory"):
            prepare(state, self.rootfs, self.root, self.root / "execution.json")
        self.assertEqual(list(state.iterdir()), [])


if __name__ == "__main__":
    unittest.main()
