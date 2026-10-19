"""Private inventory cannot be published even when deliberately force-staged."""
from pathlib import Path
import subprocess
import tempfile
import unittest
from runner.privacy import check


class Privacy(unittest.TestCase):
    def test_force_staged_inventory_is_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);(root/'runner').mkdir()
            subprocess.run(['git','init','-q',str(root)],check=True)
            (root/'runner/hosts.local.toml').write_text('[hosts.private]\n')
            subprocess.run(['git','-C',str(root),'add','-f','--','runner/hosts.local.toml'],check=True)
            with self.assertRaisesRegex(ValueError,'private runner inventory'):
                check(root)
