"""Behavioral contracts for publishing and verifying filesystem input bundles."""
import os
from pathlib import Path
import tempfile
import unittest

from durable.common import verify_input_bundle
from durable.intent.inputs import freeze


class InputBundles(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve() / "inputs"

    def assert_bundle(self, identity, files):
        bundle = verify_input_bundle(self.root, identity)
        self.assertEqual(bundle, self.root / identity)
        self.assertTrue(bundle.is_dir())
        for name, data in files.items():
            self.assertEqual((bundle / name).read_bytes(), data)
        return bundle

    def test_nested_files_converge_with_reversed_input_order(self):
        files = {"nested/deep/data.bin": b"\x00\xffpayload", "notes.txt": b"notes\n"}
        first = freeze(self.root, files)
        original = self.assert_bundle(first, files)
        manifest = (original / "manifest.json").read_bytes()

        second = freeze(self.root, dict(reversed(list(files.items()))))

        self.assertEqual(second, first)
        self.assertEqual(self.assert_bundle(second, files), original)
        self.assertEqual((original / "manifest.json").read_bytes(), manifest)

    def test_changed_bytes_publish_new_identity_and_preserve_previous_bundle(self):
        before = {"nested/data.bin": b"old", "stable.txt": b"unchanged"}
        after = {**before, "nested/data.bin": b"new"}
        first = freeze(self.root, before)
        original = self.assert_bundle(first, before)
        manifest = (original / "manifest.json").read_bytes()

        second = freeze(self.root, after)

        self.assertNotEqual(second, first)
        self.assert_bundle(second, after)
        self.assertEqual(self.assert_bundle(first, before), original)
        self.assertEqual((original / "manifest.json").read_bytes(), manifest)

    def test_empty_bundle_verifies(self):
        identity = freeze(self.root, {})
        bundle = self.assert_bundle(identity, {})
        self.assertEqual({path.name for path in bundle.iterdir()}, {"manifest.json"})

    def test_tampered_published_bytes_refuse_verification(self):
        files = {"nested/data.bin": b"original"}
        identity = freeze(self.root, files)
        path = self.assert_bundle(identity, files) / "nested/data.bin"
        mode = path.stat().st_mode & 0o777
        try:
            path.chmod(0o600)
            path.write_bytes(b"tampered")
        finally:
            path.chmod(mode)

        with self.assertRaises(ValueError):
            verify_input_bundle(self.root, identity)

    def test_symlink_replacement_refuses_verification(self):
        files = {"nested/data.bin": b"original"}
        identity = freeze(self.root, files)
        path = self.assert_bundle(identity, files) / "nested/data.bin"
        target = self.root.parent / "replacement.bin"
        target.write_bytes(files["nested/data.bin"])
        path.unlink()
        path.symlink_to(target)

        with self.assertRaises(ValueError):
            verify_input_bundle(self.root, identity)
        self.assertEqual(target.read_bytes(), files["nested/data.bin"])

    def test_fifo_replacement_refuses_verification(self):
        files = {"nested/data.bin": b"original"}
        identity = freeze(self.root, files)
        path = self.assert_bundle(identity, files) / "nested/data.bin"
        path.unlink()
        os.mkfifo(path, 0o600)

        with self.assertRaises(ValueError):
            verify_input_bundle(self.root, identity)


if __name__ == "__main__":
    unittest.main()
