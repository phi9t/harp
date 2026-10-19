"""The migration preserves exact proof bytes, modes and complete Git coverage."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

import relocation


class RelocationTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.git('init', '-q')
        self.git('config', 'user.name', 'Fixture')
        self.git('config', 'user.email', 'fixture@example.invalid')
        source = self.root / 'formalization/lean/Proof.lean'
        source.parent.mkdir(parents=True)
        source.write_text('theorem identity (x : Nat) : x = x := rfl\n')
        self.git('add', 'formalization/lean/Proof.lean')
        self.git('commit', '-qm', 'proof')
        revision = self.git('rev-parse', 'HEAD')
        blob = self.git('rev-parse', 'HEAD:formalization/lean/Proof.lean')
        self.destination = self.root / 'lab/math/lean/Proof.lean'
        self.destination.parent.mkdir(parents=True)
        source.rename(self.destination)
        digest = hashlib.sha256(self.destination.read_bytes()).hexdigest()
        self.record = {'schema_version':'harp-math-relocation/v1', 'source_commit':revision,
            'qualification':'historical', 'prefixes':[{'source':'formalization/', 'destination':'lab/math/'}],
            'entries':[{'source':'formalization/lean/Proof.lean', 'destination':'lab/math/lean/Proof.lean',
                'source_blob':blob, 'mode':'100644', 'source_sha256':digest,
                'destination_sha256':digest, 'disposition':'retained'}]}
        self.save()

    def git(self, *args):
        return subprocess.check_output(['git', '-C', str(self.root), *args], text=True).strip()

    def save(self):
        (self.root / 'lab/math/relocation.json').write_text(json.dumps(self.record))

    def test_exact_move_is_verified_against_original_git_object(self):
        self.assertEqual(relocation.verify(self.root, qualified_revision=self.record["source_commit"])['retained'], 1)

    def test_proof_edits_cannot_be_hidden_by_adaptation_and_new_destination_digest(self):
        self.destination.write_text('axiom identity : False\n')
        row = self.record['entries'][0]
        row['destination_sha256'] = hashlib.sha256(self.destination.read_bytes()).hexdigest()
        row['disposition'] = 'adapted'
        self.save()
        with self.assertRaisesRegex(relocation.RelocationError, 'preserved source bytes changed'):
            relocation.verify(self.root, qualified_revision=self.record["source_commit"])

    def test_source_cannot_be_omitted_from_migration_record(self):
        self.record['entries'] = []
        self.save()
        with self.assertRaisesRegex(relocation.RelocationError, 'coverage'):
            relocation.verify(self.root, qualified_revision=self.record["source_commit"])

    def test_destination_cannot_redirect_through_a_symlink(self):
        target = self.root / 'external.lean'
        self.destination.rename(target)
        self.destination.symlink_to(target)
        with self.assertRaisesRegex(relocation.RelocationError, 'symlinked'):
            relocation.verify(self.root, qualified_revision=self.record["source_commit"])

    def test_new_proof_input_cannot_inherit_historical_qualification(self):
        (self.destination.parent / 'Unqualified.lean').write_text('axiom invalid : False\n')
        with self.assertRaisesRegex(relocation.RelocationError, 'unqualified new proof authority'):
            relocation.verify(self.root, qualified_revision=self.record["source_commit"])

    def test_record_cannot_nominate_an_unqualified_history_revision(self):
        with self.assertRaisesRegex(relocation.RelocationError, 'qualified migration revision'):
            relocation.verify(self.root)
