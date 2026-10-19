"""Oracle command forms cannot resolve candidate code implicitly or explicitly."""
import json
import unittest
from durable.intent.policy import acceptance


class OracleAdmission(unittest.TestCase):
    def test_unqualified_interpreter_and_executable_forms_refuse(self):
        commands=[['/usr/bin/python3','-c','import pathlib'],
                  ['/usr/bin/python3','-I','/proc/self/cwd/checker'],
                  ['/usr/bin/python3','-I','/home/bench/run/workspace/checker'],
                  ['/usr/bin/python3','-I','-m','candidate'],
                  ['/proc/self/cwd/evil'],['/usr/bin/../../home/bench/run/workspace/evil'],
                  ['/usr/bin/env','/proc/self/cwd/evil']]
        for argv in commands:
            with self.subTest(argv=argv),self.assertRaises(ValueError):
                acceptance(json.dumps({'checks':[argv],'allowed_paths':['**']}).encode())
    def test_missing_change_scope_refuses(self):
        with self.assertRaises(ValueError):acceptance(b'{"checks":[["/usr/bin/true"]]}')
    def test_owner_frozen_script_and_inline_code_are_explicit(self):
        for argv,files in [(['/usr/bin/python3','-I','-c','assert True'],[]),
                           (['/usr/bin/python3','-I','/oracle/check'],['check'])]:
            data={'checks':[argv],'allowed_paths':['answer.txt'],'oracle_files':files}
            self.assertEqual(acceptance(json.dumps(data).encode()),data)
