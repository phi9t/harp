"""Relocation verification rejects data loss and altered captured bytes."""
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

from verify_absorption import verify


class PreservationTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.git('init', '-q')
        self.git('config', 'user.name', 'Test')
        self.git('config', 'user.email', 'test@example.invalid')
        self.write('source', b'original')
        self.git('add', 'source'); self.git('commit', '-qm', 'source')
        tip=self.git('rev-parse','HEAD')
        self.write('corenius/bench/task.txt', b'captured benchmark')
        self.write('corenius/HARP_IMPORT.json', json.dumps({'source_commit':tip,'branches':[{'commit':tip}], 'worktrees':[]}).encode())
        self.git('add','corenius'); self.git('commit','-qm','import')
        baseline=self.git('rev-parse','HEAD')
        rows={}
        for name in ('corenius/bench/task.txt','corenius/HARP_IMPORT.json'):
            data=(self.root/name).read_bytes()
            destination='lab/benchmarks/task.txt' if '/bench/' in name else 'evidence/manifest.json'
            self.write(destination,data)
            rows[name]={'source_blob':self.git('rev-parse',baseline+':'+name),'source_mode':'100644', 'source_sha256':hashlib.sha256(data).hexdigest(), 'disposition':'preserved', 'destination':destination,'destination_mode':'100644','destination_sha256':hashlib.sha256(data).hexdigest()}
            (self.root/name).unlink()
        self.record={'schema':'harp-absorption/v2','baseline':baseline,'previous_manifest_blob':rows['corenius/HARP_IMPORT.json']['source_blob'],'paths':rows}
        self.save()
        (self.root/'corenius/bench').rmdir(); (self.root/'corenius').rmdir()

    def write(self,name,data):
        path=self.root/name; path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(data)

    def git(self,*args):
        return subprocess.check_output(['git',*args],cwd=self.root,text=True).strip()

    def save(self):
        self.write('evidence/provenance/absorption.json', json.dumps(self.record).encode())

    def test_current_bytes_and_ancestry_pass(self):
        verify(self.root)

    def test_missing_or_changed_benchmark_is_refused(self):
        self.write('lab/benchmarks/task.txt',b'replaced')
        with self.assertRaisesRegex(ValueError,'bytes/mode'):
            verify(self.root)
        item=self.record['paths']['corenius/bench/task.txt']
        item.update(disposition='retired',reason='not needed')
        self.save()
        with self.assertRaisesRegex(ValueError,'cannot be silently retired'):
            verify(self.root)

    def test_incomplete_inventory_and_symlink_are_refused(self):
        item=self.record['paths'].pop('corenius/bench/task.txt');self.save()
        with self.assertRaisesRegex(ValueError,'incomplete'):
            verify(self.root)
        self.record['paths']['corenius/bench/task.txt']=item;self.save()
        path=self.root/'lab/benchmarks/task.txt';path.unlink();path.symlink_to(self.root/'source')
        with self.assertRaisesRegex(ValueError,'symlinked'):
            verify(self.root)
