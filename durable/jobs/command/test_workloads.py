"""Workload boundaries reject mutable artifacts and false acceptance reports."""
import json
from pathlib import Path
import tempfile
import unittest
from durable.jobs.command.admission import manifest
from durable.jobs.command.worker import artifacts


class Workloads(unittest.TestCase):
    def test_committed_manifest_requires_literal_commands_and_checks(self):
        raw=json.dumps({'schema':1,'profile':'native','timeout':60,'command':['/usr/bin/python3','-I','-c','pass'],
                        'checks':[['/usr/bin/python3','-I','-c','pass']],'artifacts':['result.json']}).encode()
        data=json.loads(raw)
        self.assertEqual(manifest(raw)['profile'],'native')
        for change in [{'command':['echo','unsafe']},{'checks':[]},{'artifacts':['../escape']},
                       {'checks':[['/usr/bin/python3','-I','{source}/checker.py']]},
                       {'checks':[['/bin/sh','-c','true']]}]:
            with self.subTest(change=change),self.assertRaises(ValueError):
                manifest(json.dumps({**data,**change}).encode())

    def test_only_regular_bounded_artifacts_are_published(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d).resolve();(root/'good').write_bytes(b'result')
            self.assertEqual(artifacts(root,['good'])[0]['size'],6)
            (root/'bad').symlink_to(root/'good')
            with self.assertRaisesRegex(ValueError,'symlink'):
                artifacts(root,['bad'])
            with self.assertRaisesRegex(ValueError,'missing'):
                artifacts(root,['missing'])


if __name__=='__main__':
    unittest.main()
