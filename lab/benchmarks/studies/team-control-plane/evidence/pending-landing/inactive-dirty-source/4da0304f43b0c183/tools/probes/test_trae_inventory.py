"""Read-only kernel/profile and owned /proc-snapshot fixtures; no process launch."""
import os
from pathlib import Path
import unittest
from trae_namespace_supervisor import NamespaceSupervisor

class HostInventoryBehavior(unittest.TestCase):
    def test_readable_host_nspid_excludes_actual_hidden_host_namespace_link(self):
        status=Path('/proc/32161/status').read_text()
        values=[line.split()[1:] for line in status.splitlines() if line.startswith('NSpid:')]
        self.assertEqual(values,[['32161']])
        self.assertEqual(Path('/proc/32161').stat().st_uid,os.getuid())
        with self.assertRaises(PermissionError):os.readlink('/proc/32161/ns/pid')
        error=None
        try:NamespaceSupervisor.members('pid:[0]')
        except (OSError,RuntimeError) as failure:error=failure
        self.assertIsNone(error,'host-profile exclusion must precede namespace-link read: '+repr(error))

class SnapshotInventoryBehavior(unittest.TestCase):
    def setUp(self):
        import tempfile
        self.tmp=tempfile.TemporaryDirectory();self.root=Path(self.tmp.name)
        (self.root/'self').mkdir();(self.root/'self/status').write_text('NSpid:\t'+str(os.getpid())+'\n')
        self.process=self.root/'101';self.process.mkdir()
        (self.process/'status').write_text('NSpid:\t101\n')
    def tearDown(self):
        ns=self.process/'ns'
        if ns.exists():ns.chmod(0o700)
        self.tmp.cleanup()
    def test_single_host_component_excluded_without_any_namespace_link(self):
        self.assertEqual(NamespaceSupervisor.members('pid:[777]',self.root),[])
    def test_missing_empty_duplicate_or_invalid_nspid_is_uncertain(self):
        for status in ['Name:\towned\n','NSpid:\t\n','NSpid:\t101\nNSpid:\t101\n','NSpid:\t101 unknown\n','NSpid:\t102\n']:
            with self.subTest(status=status):
                (self.process/'status').write_text(status)
                with self.assertRaises(RuntimeError):NamespaceSupervisor.members('pid:[777]',self.root)
    def test_nested_nspid_with_unreadable_identity_is_not_skipped(self):
        (self.process/'status').write_text('NSpid:\t101 1\n')
        ns=self.process/'ns';ns.mkdir();(ns/'pid').symlink_to('pid:[777]');ns.chmod(0o000)
        with self.assertRaises(PermissionError):NamespaceSupervisor.members('pid:[777]',self.root)
    def test_nested_readable_identity_is_included(self):
        (self.process/'status').write_text('NSpid:\t101 1\n')
        ns=self.process/'ns';ns.mkdir();(ns/'pid').symlink_to('pid:[777]')
        (self.process/'stat').write_text('101 (owned snapshot) S '+'0 '*18+'456 0\n')
        self.assertEqual(NamespaceSupervisor.members('pid:[777]',self.root),[{'pid':101,'start_ticks':456,'namespace':'pid:[777]','inner_pid':1}])
    def test_nested_observer_frame_does_not_claim_host_exclusion(self):
        (self.root/'self/status').write_text('NSpid:\t'+str(os.getpid())+' 1\n')
        with self.assertRaises(RuntimeError):NamespaceSupervisor.members('pid:[777]',self.root)

class ProcessFenceBehavior(unittest.TestCase):
    def test_actual_existing_observer_namespace_can_be_pinned_without_launch_or_signal(self):
        from trae_namespace_supervisor import ProcessFence
        anchor=NamespaceSupervisor.identity(os.getpid());fence=ProcessFence(anchor)
        try:
            fence.validate()
            self.assertEqual(anchor['namespace'],'pid:['+str(fence.inode)+']')
            self.assertTrue(hasattr(os,'pidfd_open'))
            descriptor=os.pidfd_open(os.getpid(),0)
            os.close(descriptor)
        finally:fence.close()
        with self.assertRaises(RuntimeError):fence.validate()
    def test_recorded_owner_identity_mismatch_refuses_without_signaling(self):
        from trae_namespace_supervisor import ProcessFence
        anchor=NamespaceSupervisor.identity(os.getpid());wrong=dict(anchor,start_ticks=anchor['start_ticks']-1)
        with self.assertRaises(RuntimeError):ProcessFence(wrong)
        fence=ProcessFence(anchor)
        try:
            with self.assertRaises(RuntimeError):fence.signal_recorded(wrong)
            fence.validate()
        finally:fence.close()
