from pathlib import Path
import json,sqlite3,collections
r=Path(__file__).parent
def records(p):return [json.loads(l) for l in p.read_text().splitlines()]
d=records(r/'protocol.jsonl');raw=records(r/'run3/protocol.jsonl');checks={}
requests=[x['recv'] for x in raw if x.get('connection')=='controller' and x.get('recv',{}).get('method')=='item/tool/call']
tui_requests=[x['message'] for x in raw if x.get('connection')=='tui' and x.get('direction')=='recv' and x.get('message',{}).get('method')=='item/tool/call']
checks['same_native_tool_request_broadcast_to_controller_and_tui']=any(a==b for a in requests for b in tui_requests)
checks['unsubscribed_authenticated_client_reply_used_by_native_provider']=any('FOREIGN_CONNECTION_RESPONSE' in str(x.get('body',{}).get('input')) for x in d if 'fake_request_number'in x)
checks['gateway_denied_tui_helper_thread_creation']=any(x.get('gateway_denied',{}).get('method')=='thread/start' for x in d)
checks['gateway_withheld_dynamic_requests']=any('gateway_withheld_tool_request'in x for x in d)
checks['bound_native_tui_input_forwarded']=any('UI_BOUND_INPUT'in str(x.get('gateway_bound_human_input',{})) for x in d)
h=(r/'native-items.json').read_text()
for marker in ['Question Q_IDLE from ATTEMPT_A','Question Q_ACTIVE from ATTEMPT_B','UI_BOUND_INPUT']:checks['native_retained_history:'+marker]=marker in h
checks['wrong_expected_turn_rejected']=any('expected active turn id `STALE_TURN`' in str(x.get('recv',{}).get('error')) for x in d)
checks['completed_turn_steer_rejected']=any(x.get('recv',{}).get('error',{}).get('message')=='no active turn to steer' for x in d)
original=[x['recv'] for x in d if x.get('connection')=='controller' and x.get('recv',{}).get('method')=='item/tool/call' and x['recv']['params']['arguments'].get('question_id')=='Q_LOST']
replay=[x['recv'] for x in d if x.get('connection')=='controller-reconnected' and x.get('recv',{}).get('method')=='item/tool/call']
checks['pending_request_replayed_identically_after_reconnect']=bool(original and replay and original[-1]==replay[-1])
db=sqlite3.connect('file:'+str(r/'question-fixture.sqlite')+'?mode=ro',uri=True)
for q in ['Q_IDLE','Q_ACTIVE']:
 checks[q+'_explicitly_resolved']=db.execute('SELECT state,answer FROM questions WHERE id=?',(q,)).fetchone()==('resolved','MODEL_FREE_EXPLICIT_LEAD_ANSWER')
checks['lost_ack_fixture_effect_recorded_once']=db.execute("SELECT count(*) FROM receipts WHERE question='Q_LOST' AND state='acknowledged_by_lead'").fetchone()[0]==1
checks['latest_fixture_owned_processes_terminated']=all(x['terminated'] and not Path('/proc',str(x['pid'])).exists() for x in json.loads((r/'resources.json').read_text()))
(r/'oracle-results.json').write_text(json.dumps(checks,indent=2));print(json.dumps(checks,indent=2));assert all(checks.values())
