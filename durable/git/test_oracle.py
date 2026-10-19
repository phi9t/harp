"""Real CLI regressions for candidate-controlled checkers and admitted scope."""
import json
from durable.agents.test_admission import CodingLifecycle


class OracleBoundary(CodingLifecycle):
    def oracle_task(self, allowed):
        (self.repo/'answer.txt').write_text('0\n')
        (self.repo/'check_answer.py').write_text("from pathlib import Path\nassert Path('answer.txt').read_text() == '42\\n'\n")
        (self.repo/'acceptance.json').write_text(json.dumps({
            'allowed_paths': allowed, 'oracle_files': ['check_answer.py'],
            'checks': [['/usr/bin/python3', '-I', '/oracle/check_answer.py']]}))
        self.git('add', 'answer.txt', 'check_answer.py', 'acceptance.json')
        self.git('commit', '-qm', 'Pin external owner oracle and change scope')

    def test_candidate_checker_cannot_replace_frozen_owner_oracle(self):
        self.oracle_task(['**'])
        self.admit('bad', '--fixture-path', 'check_answer.py', '--fixture-content', 'raise SystemExit(0)\n')
        self.call('start', '--until-idle', '--timeout', '30')
        self.call('capture', 'bad')
        verification = self.call('verify', 'bad')
        self.call('start', '--until-idle', '--timeout', '30')
        proof = self.call('status', verification['run_id'])['result']
        self.assertFalse(proof['passed'])
        self.assertNotEqual(proof['checks'][0]['returncode'], 0)

    def test_out_of_scope_checker_change_is_rejected(self):
        self.oracle_task(['answer.txt'])
        self.admit('scope', '--fixture-path', 'check_answer.py', '--fixture-content', 'raise SystemExit(0)\n')
        self.call('start', '--until-idle', '--timeout', '30')
        error = self.call('capture', 'scope', ok=False)
        self.assertIn('change policy', json.dumps(error))

    def test_correct_answer_passes_frozen_oracle(self):
        self.oracle_task(['answer.txt'])
        self.admit('good')
        self.call('start', '--until-idle', '--timeout', '30')
        verification = self.call('verify', 'good')
        self.call('start', '--until-idle', '--timeout', '30')
        proof = self.call('status', verification['run_id'])['result']
        self.assertTrue(proof['passed'], proof)
