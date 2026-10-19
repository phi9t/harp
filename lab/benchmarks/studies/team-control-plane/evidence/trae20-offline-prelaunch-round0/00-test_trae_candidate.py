import json
from pathlib import Path
import subprocess
import sys
import unittest

class CandidateBoundary(unittest.TestCase):
    def test_runtime_refusal_precedes_configuration_or_process_entry(self):
        script=Path(__file__).with_name('trae_synthetic_candidate.py')
        result=subprocess.run([sys.executable,str(script),'--launch-trae','--offline-check'],capture_output=True,timeout=2)
        self.assertEqual(result.returncode,78)
        self.assertIn(b'runtime admission withheld',result.stderr)
        self.assertNotIn(b'Ran ',result.stderr)

    def test_public_configuration_is_parseable_but_explicitly_unverified(self):
        script=Path(__file__).with_name('trae_synthetic_candidate.py')
        result=subprocess.run([sys.executable,str(script),'--render-config'],capture_output=True,timeout=2)
        self.assertEqual(result.returncode,0,result.stderr)
        rendered=json.loads(result.stdout);self.assertEqual(rendered['status'],'unverified_pinned_binary_hypothesis')
        config=rendered['configuration'];provider=config['model_providers']['corenius-synthetic']
        self.assertEqual(provider['request_max_retries'],0);self.assertEqual(provider['stream_max_retries'],0)
        self.assertEqual(provider['request_overrides']['extra_body']['max_output_tokens'],32)
        self.assertFalse(provider['requires_openai_auth'])
