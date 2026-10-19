"""Wrong-way framework dependencies and resurrected wrappers fail the gate."""
from pathlib import Path
import tempfile
import subprocess
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
            for name in ('benchmarks', 'assets', 'content', 'crates', 'knowledge'):
                with self.subTest(name=name):
                    (root/name).mkdir()
                    with self.assertRaisesRegex(ValueError, 'retired top-level'):
                        check(root)
                    (root/name).rmdir()

    def test_ignored_operator_notes_are_preserved_but_new_knowledge_source_is_refused(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            subprocess.run(['git', 'init', '-q', str(root)], check=True)
            (root / '.gitignore').write_text('knowledge/private/\n')
            (root / 'knowledge/private').mkdir(parents=True)
            (root / 'knowledge/private/note.md').write_text('operator note\n')
            check(root)
            (root / 'knowledge/new.md').write_text('new canonical source\n')
            with self.assertRaisesRegex(ValueError, 'retired top-level'):
                check(root)

    def test_product_cannot_depend_on_the_retired_workload(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root/'cli').mkdir()
            (root/'cli/Cargo.toml').write_text(
                '[dev-dependencies]\nold = { path = "../lab/benchmarks/legacy_runtime/harp-engine" }\n')
            with self.assertRaisesRegex(ValueError, 'retired runtime workload'):
                check(root)

    def test_estate_and_intent_cannot_depend_on_job_implementation(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for owner in ('estate', 'intent'):
                package = root / 'durable' / owner
                package.mkdir(parents=True, exist_ok=True)
                source = package / 'policy.py'
                source.write_text('from durable.jobs.coding.admission import git\n')
                with self.assertRaisesRegex(ValueError, 'job-owned'):
                    check(root)
                source.write_text('from durable.estate.git import git\n')
            check(root)

    def test_production_registry_cannot_import_demo_jobs(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / 'durable/jobs').mkdir(parents=True)
            (root / 'durable/jobs/registry.py').write_text('from durable.fixtures.workflows import REGISTRY\n')
            with self.assertRaisesRegex(ValueError, 'qualification fixtures'):
                check(root)
