"""Wrong-way framework dependencies and resurrected wrappers fail the gate."""
from pathlib import Path
import tempfile
import unittest
from check_architecture import check


class OwnershipTests(unittest.TestCase):
    def test_allowed_dependency_and_wrong_way_nested_import(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);(root/'durable').mkdir();(root/'insula').mkdir()
            (root/'durable/worker.py').write_text('from insula import capsule\n')
            check(root)
            (root/'insula/capsule.py').write_text('def launch():\n    from durable import execution\n')
            with self.assertRaisesRegex(ValueError,'forbidden layer durable'):
                check(root)

    def test_retired_package_cannot_reappear(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);(root/'harp_durable').mkdir()
            with self.assertRaisesRegex(ValueError,'retired framework'):
                check(root)
