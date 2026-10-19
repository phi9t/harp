#!/usr/bin/env python3
import json
import pathlib
import sys
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "scripts" / "insula"))

from rootfs_tree_manifest import (  # noqa: E402
    DEFAULT_CONTRACT_PATH,
    verify_recorded_manifest,
    write_manifest,
    update_contract,
)


class RootfsTreeManifestTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.rootfs = pathlib.Path(self.tmp.name)
        etc = self.rootfs / "etc"
        etc.mkdir()
        (self.rootfs / "bin").mkdir()
        (self.rootfs / "bin" / "tool").write_text("#!/bin/sh\necho tool\n", encoding="utf-8")
        (self.rootfs / "bin" / "tool").chmod(0o755)
        (self.rootfs / "link").symlink_to("/bin/tool")
        (self.rootfs / DEFAULT_CONTRACT_PATH.lstrip("/")).write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "kind": "soma_emacs_rootfs_contract",
                    "forbidden_abi_prefixes": ["/nix", "/data02/home"],
                }
            )
            + "\n",
            encoding="utf-8",
        )

    def tearDown(self):
        self.tmp.cleanup()

    def test_manifest_write_update_and_verify(self):
        summary = write_manifest(self.rootfs)
        update_contract(self.rootfs, summary)

        self.assertEqual(verify_recorded_manifest(self.rootfs), [])
        self.assertEqual(verify_recorded_manifest(self.rootfs, verify_tree=True), [])

        contract = json.loads(
            (self.rootfs / DEFAULT_CONTRACT_PATH.lstrip("/")).read_text(encoding="utf-8")
        )
        record = contract["rootfs_tree_manifest"]
        self.assertEqual(record["kind"], "soma_rootfs_tree_manifest_v1")
        self.assertEqual(record["manifest_path"], "/etc/soma-insula-rootfs-tree-manifest.jsonl")
        self.assertGreater(record["entry_count"], 0)
        self.assertGreater(record["file_count"], 0)

    def test_manifest_verification_rejects_tampered_tree(self):
        summary = write_manifest(self.rootfs)
        update_contract(self.rootfs, summary)
        (self.rootfs / "bin" / "extra").write_text("late mutation\n", encoding="utf-8")

        errors = verify_recorded_manifest(self.rootfs, verify_tree=True)
        self.assertTrue(errors)
        self.assertTrue(any("entry_count" in error for error in errors), errors)


if __name__ == "__main__":
    unittest.main()
