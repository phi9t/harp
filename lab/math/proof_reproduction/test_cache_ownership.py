"""The candidate owns proof pins; its Git primary owns only the warm cache."""
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from test_proof_evidence import FixtureBuilder
import proof_evidence


class CacheOwnershipTests(unittest.TestCase):
    def test_unlanded_worktree_uses_its_own_source_and_manifest_with_primary_cache(self):
        fixture = FixtureBuilder()
        self.addCleanup(fixture.cleanup)
        with tempfile.TemporaryDirectory() as directory:
            primary = Path(directory) / 'primary'
            candidate = Path(directory) / 'candidate'
            primary.mkdir()
            def git(*args):
                return subprocess.run(['git', '-C', str(primary), *args], check=True,
                                      capture_output=True, text=True)
            git('init', '-q')
            git('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid',
                'commit', '--allow-empty', '-qm', 'primary has no new proof layout')
            git('worktree', 'add', '--detach', str(candidate), 'HEAD')
            shutil.copytree(fixture.repo, candidate, dirs_exist_ok=True, symlinks=True)
            primary_cache = primary / 'lab/math/lean/.lake'
            primary_cache.parent.mkdir(parents=True)
            primary_cache.symlink_to(fixture.cache_root, target_is_directory=True)
            self.assertFalse((primary_cache.parent / 'lake-manifest.json').exists())
            primary_root, approved_cache = proof_evidence.resolve_primary_checkout_cache(candidate)
            self.assertEqual(primary_root, primary.resolve())
            self.assertEqual(approved_cache, primary.resolve() / 'lab/math/lean/.lake')
            ready = proof_evidence.evaluate_route(candidate, approved_cache, 'jin')
            self.assertEqual(ready.to_json()['status'], 'ready')
            # Primary cache discovery never substitutes primary source for the candidate.
            (candidate / 'lab/math/lean/lake-manifest.json').write_text('{}\n')
            with self.assertRaises(proof_evidence.PreflightError):
                proof_evidence.evaluate_route(candidate, approved_cache, 'jin')
