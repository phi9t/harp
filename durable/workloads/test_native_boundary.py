"""Public native workload lifecycle protects phase isolation and stable artifacts."""
import json
import os
from pathlib import Path
import sys
import subprocess
import unittest

import durable.agents.test_admission as admission_tests


@unittest.skipUnless(sys.platform == 'linux' and all(os.environ.get(name) for name in
                     ('HARP_TEST_CGROUP_ROOT','HARP_TEST_ROOTFS')),
                     'requires native rootfs and disposable delegated Linux cgroup')
class NativeBoundary(unittest.TestCase):
    git = admission_tests.GitInputs.git
    setUp = admission_tests.CodingLifecycle.setUp

    def call(self, *args, ok=True):
        # Full selected-rootfs validation can exceed the small coding fixture's
        # 60-second subprocess bound; activity execution keeps its own deadline.
        result = subprocess.run([*self.command,*args], cwd=admission_tests.ROOT,
                                env=self.env,text=True,capture_output=True,timeout=240)
        self.assertEqual(result.returncode == 0,ok,result.stderr)
        return json.loads(result.stdout if ok else result.stderr)['data' if ok else 'error']

    def test_workload_phase_namespaces_protect_owner_and_accepted_artifacts(self):
        manifest = {'schema':1,'profile':'native','timeout':60,
            'command':['/usr/bin/python3','-I','/workspace/durable/workloads/qualification_fixture.py'],
            'checks':[['/usr/bin/python3','-I','-c',
                "import json,os;from pathlib import Path;r=json.loads((Path(os.environ['HARP_WORKLOAD_OUTPUT'])/'native-boundary.json').read_text());assert r=={'tests':1,'skipped':0,'passed':True}"]],
            'artifacts':['native-boundary.json'],
            'requires':{'executables':['/usr/bin/bwrap']}}
        (self.repo/'workload.json').write_text(json.dumps(manifest))
        writer = "import os,time;from pathlib import Path;time.sleep(1);Path(os.environ['HARP_WORKLOAD_OUTPUT'],'answer.txt').write_text('0')"
        produce = ("import os,subprocess,sys;from pathlib import Path;"
                   "Path(os.environ['HARP_WORKLOAD_OUTPUT'],'answer.txt').write_text('42');"
                   "subprocess.Popen([sys.executable,'-I','-c',"+repr(writer)+"],stdin=subprocess.DEVNULL,"
                   "stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=True)")
        race = {'schema':1,'profile':'native','timeout':30,
            'command':['/usr/bin/python3','-I','-c',produce],
            'checks':[['/usr/bin/python3','-I','-c',
                "import os,time;from pathlib import Path;assert Path(os.environ['HARP_WORKLOAD_OUTPUT'],'answer.txt').read_text()=='42';time.sleep(3)"]],
            'artifacts':['answer.txt']}
        (self.repo/'race.json').write_text(json.dumps(race))
        self.git('add','workload.json','race.json')
        self.git('commit','-qm','Pin workload phase isolation check')
        revision = self.git('rev-parse','HEAD').strip()
        self.call('workload','boundary','--repository',str(self.repo),
                  '--manifest','workload.json','--base',revision,'--task-revision',revision)
        self.call('workload','race','--repository',str(self.repo),
                  '--manifest','race.json','--base',revision,'--task-revision',revision)
        self.call('start','--until-idle','--timeout','90')
        status = self.call('status','boundary')
        self.assertEqual(status['status'],'SUCCEEDED',status)
        self.assertTrue(status['result']['passed'])
        attempt = Path(status['attempts'][0]['directory'])
        report = json.loads((attempt/'payload/artifacts/native-boundary.json').read_text())
        self.assertEqual(report,{'tests':1,'skipped':0,'passed':True})
        receipt = json.loads((attempt/'receipt.json').read_text())
        self.assertTrue(receipt['cleanup'])
        self.assertEqual(receipt['cleanup_proof']['populated'],0)
        race_status = self.call('status','race')
        self.assertEqual(race_status['status'],'SUCCEEDED',race_status)
        accepted = race_status['result']['artifacts'][0]
        # A direct command regression lets the detached writer replace 42 with 0
        # after the checker reads it but before the owner hashes the artifact.
        import hashlib
        self.assertEqual(accepted['sha256'],hashlib.sha256(b'42').hexdigest())
        attempt = Path(race_status['attempts'][0]['directory'])
        self.assertEqual((attempt/'payload/artifacts/answer.txt').read_bytes(),b'42')


if __name__ == '__main__':
    unittest.main()
