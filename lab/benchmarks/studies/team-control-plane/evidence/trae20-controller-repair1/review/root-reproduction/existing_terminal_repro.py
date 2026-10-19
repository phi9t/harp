from pathlib import Path
import hashlib
import json
import sys
import unittest

folder = Path(__file__).parent
sys.path.insert(0, str(folder/'fixture/tools/probes'))
import trae_case_control as control
import test_trae_case_control as tests

class ExistingTerminalRefusal(tests.DurableControllerBehavior):
    def test_existing_terminal_preserved_and_new_refusal_retained(self):
        prior = b'prior-terminal-evidence\n'
        (self.out/'terminal.json').write_bytes(prior)
        result = self.run_controller()
        retained = folder/'existing-terminal-reproduction'
        retained.mkdir(exist_ok=True)
        mapping = []
        for path in [self.case, self.grant, *sorted(self.out.iterdir())]:
            data = path.read_bytes()
            target = retained/path.name
            target.write_bytes(data)
            mapping.append({'original_path':str(path),'retained_path':str(target),'sha256':hashlib.sha256(data).hexdigest(),'bytes':len(data)})
        source_path = Path(control.__file__)
        observation = {'result':result,'files':sorted(p.name for p in self.out.iterdir()),'terminal_unchanged':(self.out/'terminal.json').read_bytes()==prior,'prior_terminal_sha256':hashlib.sha256(prior).hexdigest(),'started_nonempty':bool((self.out/'started.json').read_bytes()),'executor_calls':len(self.calls),'uncertainty_exists':(self.out/'uncertainty.json').exists(),'input_output_mapping':mapping,'source_path':str(source_path),'source_sha256':hashlib.sha256(source_path.read_bytes()).hexdigest()}
        (retained/'observation.json').write_text(json.dumps(observation,indent=2)+'\n')
        self.assertFalse(result['executed'])
        self.assertEqual(self.calls, [])
        self.assertEqual((self.out/'terminal.json').read_bytes(), prior)
        self.assertTrue((self.out/'uncertainty.json').exists(), 'healthy storage must retain new hash-bound refusal separately from prior terminal')

result = unittest.TextTestRunner(verbosity=2).run(unittest.TestSuite([ExistingTerminalRefusal('test_existing_terminal_preserved_and_new_refusal_retained')]))
sys.exit(0 if result.wasSuccessful() else 1)
