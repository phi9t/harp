from pathlib import Path
import json,sqlite3,tomllib,hashlib
R=Path(__file__).parent
records=[json.loads(l) for l in (R/'protocol.jsonl').read_text().splitlines()]
reqs=json.loads((R/'requests.json').read_text());calls=json.loads((R/'tool-calls.json').read_text());cfg=tomllib.loads((R/'codex/config.toml').read_text())
model=next(m for m in json.loads((R/'catalog.json').read_text())['models'] if m['slug']=='gpt-6-sol')
items=[i for r in records for i in r.get('fake_response_items',[]) if i['type']!='message']
outputs=[i for i in reqs[-1]['input'] if i.get('type')=='custom_tool_call_output']
DB=sqlite3.connect('file:'+str(R/'question-fixture.sqlite')+'?mode=ro',uri=True)
checks={
 'pinned_gpt6_metadata_is_code_mode_only':model['tool_mode']=='code_mode_only',
 'actual_requests_use_gpt6':all(r['model']=='gpt-6-sol' for r in reqs),
 'code_mode_and_companion_enabled':cfg['features']['code_mode'] and cfg['features']['code_mode_host'],
 'actual_schema_exposes_all_three_dynamic_methods':all(' '+n+'(args:' in (R/'actual-provider-tool-schema.json').read_text() for n in ['lead_fetch','lead_ack','lead_answer']),
 'only_real_code_mode_custom_calls_emitted':len(items)==3 and all(i['type']=='custom_tool_call' and i['namespace']=='functions' and i['name']=='exec' and 'await tools.lead_' in i['input'] for i in items),
 'native_broker_received_exact_three_methods':[x['params']['tool'] for x in calls]==['lead_fetch','lead_ack','lead_answer'],
 'native_calls_share_thread_and_turn':len({(x['params']['threadId'],x['params']['turnId']) for x in calls})==1,
 'inner_call_ids_distinct_from_outer':all(x['params']['callId'].startswith('exec-') and x['params']['callId'] not in {i['call_id'] for i in items} for x in calls),
 'custom_outputs_contain_all_three_results':len(outputs)==3 and all(v in json.dumps(outputs) for v in ['questions','acknowledged','resolved']),
 'explicit_states_persisted_in_order':[x[0] for x in DB.execute('SELECT state FROM receipts ORDER BY seq')]==['persisted','submitted_to_runtime','acknowledged_by_lead','resolved'],
 'explicit_answer_persisted':DB.execute('SELECT state,answer FROM question').fetchone()==('resolved','MODEL_FREE_EXPLICIT_GPT6_ANSWER'),
}
childs=json.loads((R/'owned-descendants.json').read_text());expected='/data02/home/philip.yang/.codex/packages/standalone/releases/0.159.3-x86_64-unknown-linux-musl/bin/codex-code-mode-host'
checks['actual_pinned_companion_observed']=any(x['exe']==expected for x in childs)
checks['owned_native_processes_gone']=all(not Path('/proc',str(x['pid'])).exists() for x in [*childs,*json.loads((R/'resources.json').read_text())])
(R/'oracle-results.json').write_text(json.dumps(checks,indent=2));print(json.dumps(checks,indent=2));assert all(checks.values())
