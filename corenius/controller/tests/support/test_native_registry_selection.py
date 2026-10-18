"""Sparse registry peers must not prevent selection of the observed thread."""
import unittest

from native_registry_selection import binding_for_thread


class RegistrySelectionTest(unittest.TestCase):
    def test_unbound_peer_preceding_target_is_skipped(self):
        peer = {"id": "peer", "state": "preparing", "document": {"server_id": "owned-peer", "executor": {}, "executor_execution": "peer-execution", "ui_input_ready": False}}
        target = {"id": "target", "state": "unknown", "document": {"thread_id": "observed-thread", "ui_input_ready": False}}
        self.assertIs(binding_for_thread([peer, target], "observed-thread"), target)

    def test_missing_target_remains_a_failure(self):
        with self.assertRaises(StopIteration):
            binding_for_thread([{"document": {"thread_id": "other-thread"}}], "observed-thread")


if __name__ == "__main__":
    unittest.main()
