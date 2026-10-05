"""Post-review semantic controls, NOT retroactive development RED evidence.

Pure unsafe variants of retained round0 source run through the same public
recording-executor tests. No subprocess, namespace, provider or runtime launch.
"""
import hashlib
import io
import json
from pathlib import Path
import unittest
import test_trae_case_control as tests
import trae_case_control as control

folder = Path(__file__).parent
baseline = (folder/'preimages/trae_case_control.py').read_text()
write_start = baseline.index('def write_all(')
write_end = baseline.index('\ndef exclusive(', write_start)
unsafe_write = 'def write_all(fd, data):\n    try:\n        os.write(fd, data)\n    except OSError:\n        pass\n\n'
short_write = 'def write_all(fd, data):\n    os.write(fd, data)\n\n'
variants = [
 ('duplicate', baseline.replace('os.O_EXCL', 'os.O_TRUNC'), 'test_durable_boundary_and_duplicate_refusal'),
 ('concurrent', baseline.replace('os.O_EXCL', 'os.O_TRUNC'), 'test_concurrent_operation_executes_once'),
 ('changed-input', baseline.replace('if actual != reader(revision, relative):', 'if False:'), 'test_changed_case_grant_and_source_refuse'),
 ('write-fault', baseline[:write_start]+unsafe_write+baseline[write_end:], 'test_failed_write_spends_reservation_without_execution'),
 ('fsync-fault', baseline.replace('os.fsync(', 'unsafe_sync(')+'\ndef unsafe_sync(fd):\n    try:\n        os.fsync(fd)\n    except OSError:\n        pass\n', 'test_fsync_failure_never_execute'),
 ('short-write', baseline[:write_start]+short_write+baseline[write_end:], 'test_short_writes_complete_and_fsync_precedes_executor'),
 ('exception', baseline.replace("'status': 'unknown', 'error':", "'status': 'terminal', 'error':"), 'test_timeout_and_exception_are_retained_unknown_without_retry'),
]
summary = []
for name, source, case in variants:
    (folder/('unsafe-semantic-'+name+'.py')).write_text(source)
    namespace = {'__file__':control.__file__, '__name__':'unsafe_semantic_control'}
    exec(compile(source, 'unsafe-semantic-'+name+'.py', 'exec'), namespace)
    tests.run_case = namespace['run_case']
    output = io.StringIO()
    result = unittest.TextTestRunner(stream=output, verbosity=2).run(unittest.TestSuite([tests.DurableControllerBehavior(case)]))
    (folder/('semantic-'+name+'-negative.log.txt')).write_text(output.getvalue())
    assert result.testsRun == 1 and len(result.failures) == 1 and not result.errors, (name, output.getvalue())
    summary.append({'control':name,'test':case,'failures':len(result.failures),'errors':len(result.errors),'unsafe_source_sha256':hashlib.sha256(source.encode()).hexdigest(),'classification':'post-review-semantic-negative-not-original-TDD-red'})
tests.run_case = control.run_case
(folder/'semantic-negative-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print('7 semantic unsafe controls detected by assertion failures; zero runtime launches')
