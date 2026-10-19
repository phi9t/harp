"""Observable SSH request serialization and fail-closed identity boundaries."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
from .transport import request, source_identity, _exchange


class Transport(unittest.TestCase):
    profile={'ssh_alias':'example.invalid','repository':'/srv/repo with spaces','state':'/srv/state','execution':'/srv/profile.json','execution_contract':'insula-durable-v1:example'}
    def test_literal_argv_and_strict_ssh_trust(self):
        identity=source_identity()
        def exchange(argv,**kwargs):
            self.assertIn('StrictHostKeyChecking=yes',argv)
            data=json.loads(kwargs['input'])
            self.assertEqual(data['arguments'],['status','literal;$(touch x)'])
            return subprocess.CompletedProcess(argv,0,json.dumps({'source':identity,'contract':self.profile['execution_contract'],'result':{},'returncode':0}),'')
        with patch('runner.transport._exchange',side_effect=exchange):
            self.assertEqual(request(self.profile,['status','literal;$(touch x)'])[1],0)
    def test_disconnect_is_unknown_and_does_not_retry(self):
        with patch('runner.transport._exchange',return_value=subprocess.CompletedProcess([],255,'','private host banner')) as run:
            with self.assertRaisesRegex(RuntimeError,'unknown'):request(self.profile,['start'])
            self.assertEqual(run.call_count,1)
    def test_endpoint_refuses_source_mismatch_before_state_creation(self):
        with tempfile.TemporaryDirectory() as d:
            state=Path(d)/'state'
            payload={'schema':1,'source':'0'*64,'contract':'insula-durable-v1:example','state':str(state),'execution':str(Path(d)/'execution'),'arguments':['start']}
            out=subprocess.run([sys.executable,'-m','runner.remote'],input=json.dumps(payload),text=True,capture_output=True)
            self.assertEqual(out.returncode,2)
            self.assertIn('source identity differs',out.stdout)
            self.assertFalse(state.exists())

    def test_endpoint_refuses_profile_mismatch_before_state_creation(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);state=root/'state';profile=root/'execution.json'
            profile.write_text(json.dumps({'contract':'different','state':str(state)}))
            payload={'schema':1,'source':source_identity(),'contract':'insula-durable-v1:example',
                     'state':str(state),'execution':str(profile),'arguments':['start']}
            out=subprocess.run([sys.executable,'-m','runner.remote'],input=json.dumps(payload),text=True,capture_output=True)
            self.assertEqual(out.returncode,2)
            self.assertIn('profile differs',out.stdout)
            self.assertFalse(state.exists())


class TransportProcess(unittest.TestCase):
    def test_exited_transport_cannot_leave_proxy_pipes_open(self):
        import time
        script = "import subprocess; subprocess.Popen(['sleep','30']); raise SystemExit(255)"
        started = time.monotonic()
        result = _exchange([sys.executable, '-c', script], input='', timeout=5)
        self.assertEqual(result.returncode, 255)
        self.assertLess(time.monotonic()-started, 3)
