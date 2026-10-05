from pathlib import Path
import hashlib
import io
import json
import sys
import unittest

folder = Path(__file__).parent
sys.path.insert(0, str(folder/'fixture/tools/probes'))
import trae_case_control as control
import test_trae_case_control as tests
origin = Path('/tmp/corenius-trae20-budget-repair-author/.scratch/team-control-plane/trae20-budget-repair/controller-repair1-20261003-v1')
summary = []

def run(label, suite):
    stream = io.StringIO()
    result = unittest.TextTestRunner(stream=stream, verbosity=2).run(suite)
    text = stream.getvalue()
    (folder/(label+'.log.txt')).write_text(text)
    item = {'label':label,'tests':result.testsRun,'failures':len(result.failures),'errors':len(result.errors),'output_sha256':hashlib.sha256(text.encode()).hexdigest()}
    summary.append(item)
    print(json.dumps(item))
    return result

result = run('independent-final-green', unittest.defaultTestLoader.loadTestsFromModule(tests))
assert result.testsRun == 17 and result.wasSuccessful()
source = (origin/'preimages/trae_case_control.py').read_text()
namespace = {'__file__':control.__file__, '__name__':'retained_round0_control'}
exec(compile(source, 'retained_round0_control', 'exec'), namespace)
tests.run_case = namespace['run_case']
result = run('independent-post-reservation-red', unittest.TestSuite([tests.DurableControllerBehavior('test_post_reservation_input_change_retains_hash_bound_refusal')]))
assert result.testsRun == 1 and len(result.failures) == 1 and not result.errors

for entry in json.loads((origin/'semantic-negative-summary.json').read_text()):
    source = (origin/('unsafe-semantic-'+entry['control']+'.py')).read_bytes()
    assert hashlib.sha256(source).hexdigest() == entry['unsafe_source_sha256']
    namespace = {'__file__':control.__file__, '__name__':'retained_unsafe_control'}
    exec(compile(source, 'retained_unsafe_'+entry['control'], 'exec'), namespace)
    tests.run_case = namespace['run_case']
    result = run('independent-semantic-'+entry['control'], unittest.TestSuite([tests.DurableControllerBehavior(entry['test'])]))
    assert result.testsRun == 1 and len(result.failures) == 1 and not result.errors

tests.run_case = control.run_case
class ExistingTerminalRefusal(tests.DurableControllerBehavior):
    def test_existing_terminal_preserved_and_new_refusal_retained(self):
        prior = b'prior-terminal-evidence\n'
        (self.out/'terminal.json').write_bytes(prior)
        result = self.run_controller()
        observation = {'result':result,'files':sorted(p.name for p in self.out.iterdir()),'terminal_unchanged':(self.out/'terminal.json').read_bytes()==prior,'started_nonempty':bool((self.out/'started.json').read_bytes()),'calls':len(self.calls)}
        (folder/'existing-terminal-observation.json').write_text(json.dumps(observation,indent=2)+'\n')
        self.assertFalse(result['executed'])
        self.assertEqual(self.calls, [])
        self.assertEqual((self.out/'terminal.json').read_bytes(), prior)
        self.assertTrue((self.out/'uncertainty.json').exists(), 'healthy storage must retain new hash-bound refusal separately from prior terminal')
result = run('existing-terminal-retention-regression', unittest.TestSuite([ExistingTerminalRefusal('test_existing_terminal_preserved_and_new_refusal_retained')]))
assert result.testsRun == 1 and len(result.failures) == 1 and not result.errors
(folder/'independent-proof-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
