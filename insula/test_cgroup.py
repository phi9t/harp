"""Fail-closed boundaries and optional real delegated-cgroup behavior."""
from __future__ import annotations

import json
import os
from pathlib import Path
import select
import subprocess
import sys
import tempfile
import unittest

from insula.cgroup import Cgroup, CgroupUnavailable, validate_root


class RefusalTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)

    def test_ordinary_directory_with_convincing_files_is_not_cgroup(self):
        for name, text in {"cgroup.type": "domain\n", "cgroup.events": "populated 0\n",
                           "cgroup.procs": "", "cgroup.kill": "",
                           "cgroup.subtree_control": ""}.items():
            (self.root / name).write_text(text)
        before = {p.name: p.read_bytes() for p in self.root.iterdir()}
        with self.assertRaisesRegex(CgroupUnavailable, "cgroup v2"):
            Cgroup.create(self.root, "attempt-1")
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.root.iterdir()})
        with self.assertRaisesRegex(CgroupUnavailable, "cgroup v2"):
            Cgroup.validate_root(self.root)
        with self.assertRaisesRegex(CgroupUnavailable, "cgroup v2"):
            validate_root(self.root)
        self.assertEqual(before, {p.name: p.read_bytes() for p in self.root.iterdir()})

    def test_missing_and_relative_roots_do_not_create_directories(self):
        missing = self.root / "missing"
        for root in [missing, "relative", "", None]:
            with self.subTest(root=root), self.assertRaises(CgroupUnavailable):
                Cgroup.create(root, "attempt-1")
        self.assertEqual(list(self.root.iterdir()), [])

    def test_symlink_root_is_refused(self):
        link = self.root / "alias"
        link.symlink_to(self.root, target_is_directory=True)
        with self.assertRaises(CgroupUnavailable):
            Cgroup.create(link, "attempt-1")
        self.assertEqual([p.name for p in self.root.iterdir()], ["alias"])

    def test_attempt_identifier_is_bounded_before_filesystem_changes(self):
        for attempt in ["", "../escape", "a/b", "a\x00b", "x" * 129, None, 7]:
            with self.subTest(attempt=attempt), self.assertRaises(CgroupUnavailable):
                Cgroup.create(self.root, attempt)
        self.assertEqual(list(self.root.iterdir()), [])

    def test_malformed_persisted_identity_never_mutates_a_directory(self):
        identity = {"protocol": 1, "root": str(self.root), "root_device": 1,
                    "root_inode": 1, "group_device": 1, "group_inode": 2,
                    "attempt_id": "attempt-1", "generation": "a" * 32}
        cases = [None, [], {}, identity, dict(identity, attempt_id="other"),
                 dict(identity, generation="../escape"), dict(identity, protocol=True),
                 dict(identity, root_inode=True), dict(identity, group_inode=0),
                 dict(identity, root_device=-1), dict(identity, group_inode=2**64),
                 dict(identity, root="relative"), dict(identity, unexpected="value")]
        for value in cases:
            with self.subTest(value=value), self.assertRaises(CgroupUnavailable):
                Cgroup.resume(value, attempt_id="attempt-1")
        self.assertEqual(list(self.root.iterdir()), [])

    def test_timeout_bounds_and_root_binding_are_checked_without_killing(self):
        identity = {"protocol": 1, "root": str(self.root), "root_device": 1,
                    "root_inode": 1, "group_device": 1, "group_inode": 2,
                    "attempt_id": "attempt-1", "generation": "a" * 32}
        # A syntactically valid binding grants nothing: kernel verification is
        # still mandatory on every operation. Invalid timeouts precede I/O.
        group = Cgroup(identity)
        for value in [0, -1, float("nan"), float("inf"), 61, True, "3"]:
            with self.subTest(value=value), self.assertRaises(ValueError):
                group.kill_and_wait(timeout=value)
        for binding in [{}, None, {"root": str(self.root), "root_device": True, "root_inode": 1}]:
            with self.subTest(binding=binding), self.assertRaises(CgroupUnavailable):
                Cgroup.resume(identity, attempt_id="attempt-1", root_identity=binding)
        with self.assertRaises(CgroupUnavailable):
            group.kill_and_wait()
        self.assertEqual(list(self.root.iterdir()), [])

    def test_unavailable_backend_prevents_payload_executable_release(self):
        identity = {"protocol": 1, "root": str(self.root), "root_device": 1,
                    "root_inode": 1, "group_device": 1, "group_inode": 2,
                    "attempt_id": "attempt-1", "generation": "a" * 32}
        group = Cgroup(identity)
        marker = self.root / "payload-ran"
        with self.assertRaises(subprocess.SubprocessError):
            subprocess.Popen([sys.executable, "-c",
                              f"from pathlib import Path; Path({str(marker)!r}).touch()"],
                             preexec_fn=group.join_current_process, close_fds=True)
        self.assertFalse(marker.exists())
        self.assertEqual(list(self.root.iterdir()), [])


class DelegatedKernelTests(unittest.TestCase):
    def setUp(self):
        root = os.environ.get("INSULA_TEST_CGROUP_ROOT")
        if not root:
            self.skipTest("no explicit writable delegation: set INSULA_TEST_CGROUP_ROOT")
        # An explicitly supplied root is a contract: invalid delegation fails the test.
        self.group = Cgroup.create(root, "test-" + os.urandom(8).hex())
        self.addCleanup(self.remove_group)

    def remove_group(self):
        self.group.kill_and_wait()
        self.group.path.rmdir()

    def launch(self, code):
        return subprocess.Popen([sys.executable, "-c", code],
                                preexec_fn=self.group.join_current_process,
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                text=True, close_fds=True)

    def test_membership_precedes_payload_and_kills_detached_descendant(self):
        proc = self.launch("""
import json, os, time
read_fd, write_fd = os.pipe()
child = os.fork()
if child == 0:
    os.close(read_fd)
    os.setsid()
    os.write(write_fd, b'R')
    os.close(write_fd)
    time.sleep(60)
else:
    os.close(write_fd)
    assert os.read(read_fd, 1) == b'R'
    os.close(read_fd)
    controls = []
    for name in os.listdir('/proc/self/fd'):
        try:
            target = os.readlink('/proc/self/fd/' + name)
            if 'cgroup' in target:
                controls.append(target)
        except FileNotFoundError:
            pass
    print(json.dumps({'pid': os.getpid(), 'child': child, 'control_fds': controls}), flush=True)
    time.sleep(60)
""")
        try:
            ready, _, _ = select.select([proc.stdout], [], [], 5)
            self.assertTrue(ready, "payload did not reach its membership probe")
            line = proc.stdout.readline()
            self.assertTrue(line)
            pids = json.loads(line)
            self.assertEqual(pids["control_fds"], [])
            members = {int(p) for p in (self.group.path / "cgroup.procs").read_text().split()}
            self.assertTrue({pids["pid"], pids["child"]} <= members)
            resumed = Cgroup.resume(json.loads(json.dumps(self.group.identity())),
                                    attempt_id=self.group.identity()["attempt_id"])
            proof = resumed.kill_and_wait(timeout=5)
            self.assertEqual(proof["populated"], 0)
            self.assertIn("populated 0", proof["events"])
            self.assertEqual(proc.wait(timeout=5), -9)
            self.assertEqual((self.group.path / "cgroup.procs").read_text(), "")
        finally:
            self.group.kill_and_wait()
            proc.wait(timeout=5)
            proc.stdout.close()
            proc.stderr.close()

    def test_replaced_group_identity_refuses_cleanup_and_launch(self):
        old = self.group.identity()
        wrong = dict(old, group_inode=old["group_inode"] + 1)
        with self.assertRaisesRegex(CgroupUnavailable, "identity"):
            Cgroup.resume(wrong, attempt_id=old["attempt_id"])
        wrong_generation = "b" * 32 if old["generation"] != "b" * 32 else "c" * 32
        with self.assertRaises(CgroupUnavailable):
            Cgroup.resume(dict(old, generation=wrong_generation), attempt_id=old["attempt_id"])
        with self.assertRaises(CgroupUnavailable):
            Cgroup.resume(old, attempt_id=old["attempt_id"],
                          root_identity={"root": old["root"], "root_device": old["root_device"],
                                         "root_inode": old["root_inode"] + 1})
        self.group.path.rmdir()
        with self.assertRaises(CgroupUnavailable):
            Cgroup.resume(old, attempt_id=old["attempt_id"])
        self.group.path.mkdir()
        self.assertNotEqual(self.group.path.stat().st_ino, old["group_inode"])
        with self.assertRaisesRegex(CgroupUnavailable, "identity"):
            Cgroup.resume(old, attempt_id=old["attempt_id"])
        marker = Path(tempfile.gettempdir()) / ("cgroup-release-" + os.urandom(8).hex())
        try:
            with self.assertRaises(subprocess.SubprocessError):
                self.launch(f"from pathlib import Path; Path({str(marker)!r}).touch()")
            self.assertFalse(marker.exists())
        finally:
            marker.unlink(missing_ok=True)
            self.group.path.rmdir()
            # Restore a new identity solely for the fixture cleanup.
            self.group = Cgroup.create(old["root"], old["attempt_id"])


if __name__ == "__main__":
    unittest.main()
