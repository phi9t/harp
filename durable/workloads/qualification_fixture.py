"""Native proof that descendants and writes cannot cross workload/check phases.

CLI-only probe inside an admitted Linux workload with bubblewrap installed.
The outer test_native_boundary module invokes it through the public workload
lifecycle. It is deliberately not a test-discovery entry point.
"""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

if __name__ == '__main__':
    sys.path.insert(0, '/workspace')
from durable.workloads.worker import command, sandbox_argv, SOURCE, OUTPUT, ORACLE


class NativeBoundaryProbe(unittest.TestCase):
    def test_detached_writer_dies_and_checks_cannot_change_accepted_bytes(self):
        with tempfile.TemporaryDirectory(dir='/home/bench/run') as directory:
            root = Path(directory)
            source, output, oracle = (root/name for name in ('source','output','oracle'))
            for path in (source,output,oracle): path.mkdir()
            secret = root/'owner-proof.json'
            secret.write_text('owner-only')
            binding = {'execution_contract':os.environ['INSULA_DURABLE_CONTRACT'],
                       'profile':'native','run_id':'native-boundary'}
            writer = "import time;from pathlib import Path;time.sleep(.2);Path("+repr(OUTPUT+'/answer.txt')+").write_text('0')"
            workload = ("import subprocess,sys;from pathlib import Path;"
                        "Path("+repr(OUTPUT+'/answer.txt')+").write_text('42');"
                        "assert not Path("+repr(str(secret))+").exists();"
                        "assert not Path('/home/bench/input/request.json').exists();"
                        "subprocess.Popen([sys.executable,'-I','-c',"+repr(writer)+"],"
                        "stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=True)")
            result = command(sandbox_argv(source,output,['/usr/bin/python3','-I','-c',workload],binding),
                             cwd=root,env={'PATH':'/usr/bin:/bin'})
            self.assertEqual(result['exit_code'],0,result['output_tail'])
            time.sleep(.4)
            self.assertEqual((output/'answer.txt').read_text(),'42')
            checker = """from pathlib import Path
import time
assert Path(OUTPUT+'/answer.txt').read_text()=='42'
for path in [Path(OUTPUT+'/answer.txt'),Path(SOURCE+'/mutation'),Path(ORACLE+'/checker.py')]:
    try: path.write_text('tampered')
    except OSError: pass
    else: raise AssertionError('check projection was writable')
assert not Path(SECRET).exists()
assert not Path('/home/bench/input/request.json').exists()
time.sleep(.4)
assert Path(OUTPUT+'/answer.txt').read_text()=='42'
"""
            checker = 'OUTPUT='+repr(OUTPUT)+'\nSOURCE='+repr(SOURCE)+'\nORACLE='+repr(ORACLE)+'\nSECRET='+repr(str(secret))+'\n'+checker
            (oracle/'checker.py').write_text(checker)
            result = command(sandbox_argv(source,output,['/usr/bin/python3','-I',ORACLE+'/checker.py'],
                                          binding,oracle=oracle),cwd=root,env={'PATH':'/usr/bin:/bin'})
            self.assertEqual(result['exit_code'],0,result['output_tail'])
            self.assertEqual(secret.read_text(),'owner-only')
            self.assertEqual((output/'answer.txt').read_text(),'42')


if __name__ == '__main__':
    if sys.platform != 'linux' or not os.environ.get('INSULA_DURABLE_CONTRACT'):
        raise SystemExit('requires admitted native Insula workload')
    suite=unittest.defaultTestLoader.loadTestsFromTestCase(NativeBoundaryProbe)
    result=unittest.TextTestRunner(verbosity=2).run(suite)
    if target := os.environ.get('HARP_WORKLOAD_OUTPUT'):
        (Path(target)/'native-boundary.json').write_text(json.dumps({
            'tests':result.testsRun,'skipped':len(result.skipped),'passed':result.wasSuccessful()}))
    raise SystemExit(0 if result.wasSuccessful() and not result.skipped else 1)
