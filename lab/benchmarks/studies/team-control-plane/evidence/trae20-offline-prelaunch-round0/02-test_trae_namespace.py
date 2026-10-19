"""Physical owned PID/net namespace fixture; no Trae binary or provider."""
import json
import os
from pathlib import Path
import tempfile
import unittest
from trae_namespace_supervisor import NamespaceSupervisor, NamespaceSetupError

class NamespaceBehavior(unittest.TestCase):
    def test_descendant_inventory_is_exact_and_shutdown_is_empty(self):
        with tempfile.TemporaryDirectory() as tmp:
            ledger=Path(tmp)/'owner-ledger';ledger.write_text('owned sentinel')
            supervisor=NamespaceSupervisor();proof={'kind':'offline_owned_namespace'}
            try:
                proof['observations']=supervisor.start_fixture(Path(tmp)/'owner-ledger')
                proof['inventory_before']=supervisor.inventory()
                self.assertGreaterEqual(len(proof['inventory_before']),2)
                self.assertTrue(all(o['external_connect_denied'] and not o['owner_ledger_visible'] and not o['private_roots_visible'] for o in proof['observations']))
            finally:
                if supervisor.process is not None:proof['shutdown']=supervisor.stop()
                out=os.environ.get('TRAE_OFFLINE_NAMESPACE_PROOF')
                if out:Path(out).write_text(json.dumps(proof,indent=2)+'\n')
            self.assertEqual(proof['shutdown']['inventory_after'],[])
            self.assertFalse(proof['shutdown']['quarantined'])

    def test_injected_setup_noise_stops_exact_namespace_and_descendants(self):
        with tempfile.TemporaryDirectory() as tmp:
            evidence=Path(tmp)/'owned.jsonl'
            ledger=Path(tmp)/'owner-ledger';ledger.write_text('owned sentinel')
            supervisor=NamespaceSupervisor(evidence)
            with self.assertRaises(NamespaceSetupError) as raised:
                supervisor.start_fixture(Path(tmp)/'owner-ledger',noise=True)
            proof=raised.exception.proof
            out=os.environ.get('TRAE_OFFLINE_NAMESPACE_PROOF')
            if out:Path(out).write_text(json.dumps(proof,indent=2)+'\n')
            self.assertTrue(any(o['role']=='setsid-descendant' for o in proof['observations']))
            self.assertIsNotNone(proof['anchor'])
            self.assertEqual(proof['anchor']['inner_pid'],1)
            self.assertEqual(proof['shutdown']['inventory_after'],[])
            self.assertFalse(proof['shutdown']['quarantined'])
            self.assertTrue(proof['shutdown']['launcher_terminal'])
            self.assertIsNotNone(proof['shutdown']['namespace_pin'])
            self.assertEqual(proof['shutdown']['physical_stop'],'pidfd_SIGKILL')
            stages=[json.loads(line)['stage'] for line in evidence.read_text().splitlines()]
            self.assertIn('namespace_owned',stages);self.assertIn('stopped',stages);self.assertIn('setup_failed',stages)
