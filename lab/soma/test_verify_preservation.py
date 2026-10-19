"""Payload preservation checks reject omitted and changed workload dependencies."""
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from verify_preservation import verify


class PreservationTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name) / 'lab/soma'
        self.root.mkdir(parents=True)
        paths = [f'benchmarks/emacs-gym/{name}/elisp/run-benchmark.el'
                 for name in ('magit','dired','org','comint','jupyter','lsp','bridge')]
        paths += ['benchmarks/emacs-gym/elisp/soma-gym-bridge.el',
                  'workers/emacs-doom-lifecycle/elisp/run-lifecycle.el',
                  'probes/emacs-internals/elisp/run-probes.el',
                  'ai_lsp/doom/ai-lsp-verifier.el', 'scripts/fixture-grader.py']
        records = []
        for rel in paths:
            f = self.root / rel
            f.parent.mkdir(parents=True, exist_ok=True)
            f.write_text('fixture\n')
            records.append({'destination': 'lab/soma/' + rel,
                            'destination_sha256': hashlib.sha256(f.read_bytes()).hexdigest(),
                            'destination_mode': oct(f.stat().st_mode & 0o777)})
        self.manifest = {'files': records}
        self.save()

    def save(self):
        (self.root / 'source-map.json').write_text(json.dumps(self.manifest))

    def test_complete_payload_passes(self):
        self.assertEqual([], verify(self.root))

    def test_missing_grader_fails(self):
        (self.root / 'scripts/fixture-grader.py').unlink()
        self.assertTrue(any('missing' in e for e in verify(self.root)))

    def test_changed_grader_fails(self):
        (self.root / 'scripts/fixture-grader.py').write_text('return True\n')
        self.assertTrue(any('changed payload' in e for e in verify(self.root)))

    def test_omitted_family_fails_even_if_mapping_removed(self):
        self.manifest['files'] = [e for e in self.manifest['files']
                                  if '/magit/' not in e['destination']]
        self.save()
        self.assertTrue(any('omitted scenario' in e for e in verify(self.root)))

    def test_escape_rejected(self):
        self.manifest['files'][0]['destination'] = 'lab/soma/../../outside'
        self.save()
        self.assertTrue(any('unsafe' in e for e in verify(self.root)))


if __name__ == '__main__':
    unittest.main()
