"""The frozen workload oracle rejects missing scenarios and false closure claims."""
import json
import gzip
from pathlib import Path
import tempfile
import unittest
from lab.soma.check_workload import check, EXPECTED


class WorkloadOracle(unittest.TestCase):
    def test_independent_oracle_rejects_scenario_and_closure_failures(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d)
            for name,count in EXPECTED.items():
                (root/(name+'.json.gz')).write_bytes(gzip.compress(json.dumps({'total_scenarios':count,
                    'scenarios_as_expected':count,'insula_checks_passed':True}).encode()))
            check(root)
            p=root/'magit.json.gz';data=json.loads(gzip.decompress(p.read_bytes()));data['scenarios_as_expected']-=1
            p.write_bytes(gzip.compress(json.dumps(data).encode()))
            with self.assertRaisesRegex(ValueError,'magit'):
                check(root)
            data['scenarios_as_expected']+=1;data['insula_checks_passed']=False
            p.write_bytes(gzip.compress(json.dumps(data).encode()))
            with self.assertRaisesRegex(ValueError,'magit'):
                check(root)


if __name__=='__main__':
    unittest.main()
