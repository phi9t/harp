"""Remote retrieval exposes only exact artifacts from guardian-accepted workloads."""
import hashlib
from pathlib import Path
import tempfile
import unittest
from runner.remote import fetch_artifact


class Artifacts(unittest.TestCase):
    def test_retrieval_rejects_tampering_and_undeclared_files(self):
        with tempfile.TemporaryDirectory() as d:
            state=Path(d).resolve();root=state/'attempts'/('a'*32)/'payload'/'artifacts';root.mkdir(parents=True)
            (root/'summary.json').write_bytes(b'42')
            status={'workflow':'workload','status':'SUCCEEDED','result':{'attempt_id':('a'*32),
              'artifacts':[{'path':'summary.json','size':2,'sha256':hashlib.sha256(b'42').hexdigest()}]}}
            self.assertEqual(fetch_artifact({'state':str(state)},status,'summary.json')['content_base64'],'NDI=')
            with self.assertRaisesRegex(ValueError,'not declared'):
                fetch_artifact({'state':str(state)},status,'private.json')
            (root/'summary.json').write_bytes(b'00')
            with self.assertRaisesRegex(ValueError,'bytes differ'):
                fetch_artifact({'state':str(state)},status,'summary.json')


if __name__=='__main__':
    unittest.main()
