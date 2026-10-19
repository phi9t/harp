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

    def test_retired_container_roots_cannot_return(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for name in ('benchmarks', 'assets', 'content', 'crates'):
                with self.subTest(name=name):
                    (root/name).mkdir()
                    with self.assertRaisesRegex(ValueError, 'retired top-level'):
                        check(root)
                    (root/name).rmdir()

    def test_product_cannot_depend_on_the_retired_workload(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root/'cli').mkdir()
            (root/'cli/Cargo.toml').write_text(
                '[dev-dependencies]\nold = { path = "../lab/benchmarks/legacy_runtime/harp-engine" }\n')
            with self.assertRaisesRegex(ValueError, 'retired runtime workload'):
                check(root)
