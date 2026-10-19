from pathlib import Path
import json
r=Path(__file__).parent;d=json.loads((r/'result.json').read_text());sid=d['a_session_result']['session_id']
def shell(rows):return next(x for x in rows if 'read -r -t 45 answer' in x['cmdline'] and x['cmdline'].startswith('/usr/bin/bash'))
before=shell(d['a_processes_before_b']);after=shell(d['a_processes_after_b'])
def start(x):return x['stat'][x['stat'].rfind(')')+2:].split()[19]
checks={
 'a_actual_native_session_created':isinstance(sid,int) and 'A_READY'in d['a_session_result']['output'],
 'b_exact_peer_session_refused':f'Unknown process id {sid}'in json.dumps(d['b_foreign_session_result']),
 'b_guessed_session_refused':'Unknown process id 999999999'in json.dumps(d['b_guessed_session_result']),
 'a_process_identity_survived_b_attempt':before['pid']==after['pid'] and start(before)==start(after) and before['pid_namespace']==after['pid_namespace'] and before['mnt_namespace']==after['mnt_namespace'],
 'a_marker_absent_after_b':d['a_marker_after_b'] is None,
 'b_marker_absent_after_b':d['b_marker_after_b'] is None,
 'same_a_thread_own_session_succeeded':'A_RECEIVED:A_AUTHORIZED'in json.dumps(d['a_authorized_session_result']) and '"exit_code":0' in json.dumps(d['a_authorized_session_result']).replace('\\"','"'),
 'independent_a_marker_only_authorized':d['a_marker_after_owner']=='A_AUTHORIZED' and (r/'a/workspace/received-identity-run').read_text()=='A_AUTHORIZED',
 'a_read_process_exited_after_own_input':not any(x['pid']==before['pid'] for x in d['a_processes_after_owner']),
 'all_recorded_processes_gone':all(x['terminated'] and not Path('/proc',str(x['pid'])).exists() for x in json.loads((r/'resources.json').read_text())) and all(not Path('/proc',str(x['pid'])).exists() for x in d['a_processes_before_b']),
}
(r/'oracle-results.json').write_text(json.dumps(checks,indent=2));print(json.dumps(checks,indent=2));assert all(checks.values())
