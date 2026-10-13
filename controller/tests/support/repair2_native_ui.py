"""Actual pinned native controls, owned deterministic lost-reply and A-to-B replay."""
import json,sqlite3
results=[]
def exercise(root,query,command,request,grants,lead,wait,topology,token):
 auth={'principal':lead['principal'],'token':lead['token']}
 def binding():return next(b for b in query('codex_registry')['bindings'] if b['role']=='lead' and b['authority_epoch']==query('snapshot')['project']['lead_epoch'])
 b=wait(lambda:next((b for b in query('codex_registry')['bindings'] if b['role']=='lead' and b['document'].get('ui_input_ready')),None));bid=b['id'];thread=b['document']['thread_id']
 def current():return next(b for b in query('codex_registry')['bindings'] if b['id']==bid)['document']
 def terminal():return current().get('current_turn',{}).get('state')=='terminal'
 def ui(method,params):return request(root/'controller.sock',{'auth':auth,'project':'p','native_ui':{'method':method,'params':params}})
 def start(key,text):return {'threadId':thread,'clientUserMessageId':key,'input':[{'type':'text','text':text,'text_elements':[]}]}
 wait(terminal,8)
 lost=start('addendum-lost-start','ADDENDUM_LOST_START_NATIVE_ACK');(root/'lose-ack').write_text('owned actual native lost start acknowledgement')
 first=ui('turn/start',lost);assert (root/'lose-ack.observed').exists()
 recovered=wait(lambda:next((o for o in query('codex_registry')['operations'] if o['intent'].get('text')=='ADDENDUM_LOST_START_NATIVE_ACK' and o['state']=='confirmed'),None),12)
 before=len(query('codex_registry')['operations']);replay=ui('turn/start',lost);after=len(query('codex_registry')['operations']);assert before==after and 'ok' in replay
 record={'topology':topology,'lost_ack':{'first_response':first,'confirmed_operation':recovered,'replay':replay,'operations_before':before,'operations_after':after,'contract_pass':bool(replay['ok'].get('turn',{}).get('id'))}}
 wait(terminal,8);hold=root/'control-active-hold';hold.write_text('owned held turn A for exact steer admission')
 try:
  first_a=ui('turn/start',start('addendum-a','CONTROL_ACTIVE_HOLD addendum A'));assert 'ok' in first_a,first_a;A=first_a['ok']['turn']['id'];wait(lambda:current().get('current_turn',{}).get('state')=='active',8)
  steer={'threadId':thread,'expectedTurnId':A,'clientUserMessageId':'addendum-steer','input':[{'type':'text','text':'addendum exact steer','text_elements':[]}]};receipt=ui('turn/steer',steer);assert 'ok' in receipt,receipt
 finally:hold.unlink(missing_ok=True)
 wait(terminal,8);hold.write_text('owned held different current turn B')
 try:
  first_b=ui('turn/start',start('addendum-b','CONTROL_ACTIVE_HOLD addendum B'));assert 'ok' in first_b;B=first_b['ok']['turn']['id'];assert A!=B;wait(lambda:current().get('current_turn',{}).get('state')=='active',8)
  with sqlite3.connect((root/'state.db').as_uri()+'?mode=ro',uri=True) as db:
   saved=[{'parameters':json.loads(a),'request':json.loads(b),'receipt':json.loads(c) if c else None} for a,b,c in db.execute('SELECT parameters,request,receipt FROM control_ui_commands WHERE binding=?',(bid,)) if json.loads(a).get('clientUserMessageId')=='addendum-steer']
  assert len(saved)==1 and saved[0]['receipt'] and saved[0]['request']['command']['kind']=='codex_steer'
  before=len(query('codex_registry')['operations']);old_retry=ui('turn/steer',steer);after=len(query('codex_registry')['operations']);assert before==after
  stale=dict(steer,clientUserMessageId='new-stale-steer');fresh_stale=ui('turn/steer',stale);assert 'error' in fresh_stale and len(query('codex_registry')['operations'])==before
  changed=json.loads(json.dumps(steer));changed['input'][0]['text']='changed exact retained payload'
  changed_reply=ui('turn/steer',changed);assert 'error' in changed_reply and len(query('codex_registry')['operations'])==before
  foreign=dict(steer,threadId='foreign-thread');foreign_reply=ui('turn/steer',foreign);assert 'error' in foreign_reply and len(query('codex_registry')['operations'])==before
  active_replay=ui('turn/start',lost);assert active_replay==replay and len(query('codex_registry')['operations'])==before
  valid=dict(steer,clientUserMessageId='current-valid-steer',expectedTurnId=B);valid_result=ui('turn/steer',valid);assert 'ok' in valid_result
  record['scope_negatives']={'changed_payload':changed_reply,'foreign_thread':foreign_reply,'cached_start_after_later_turn':active_replay,'no_extra_operation':True}
  # Authenticated bounded public/tool history is independent of analyzer review credit.
  for grant in grants:
   worker={k:grant[k] for k in ['principal','token']};wb=next(x for x in query('codex_registry')['bindings'] if x['attempt']==grant['attempt']);cursor=0;items=[]
   while True:
    page=request(root/'controller.sock',{'auth':worker,'project':'p','query':{'kind':'control_records','args':{'collection':'runtime_public_history','binding':wb['id'],'after':cursor,'limit':20}}});assert 'ok' in page,page
    items.extend(page['ok']['items'])
    if not page['ok']['page']['has_more']:break
    assert page['ok']['page']['after']>cursor;cursor=page['ok']['page']['after']
   assert items and all(x['attempt']==grant['attempt'] for x in items)
   item=items[0];reference=item['input_reference'];full=request(root/'controller.sock',{'auth':worker,'project':'p','query':reference});assert 'ok' in full and full['ok']['digest']==reference['args']['sha256'],full
   peer=next(x for x in grants if x['attempt']!=grant['attempt']);peer_auth={k:peer[k] for k in ['principal','token']}
   assert 'error' in request(root/'controller.sock',{'auth':peer_auth,'project':'p','query':reference})
  record['public_history']={'own_worker_complete_input':True,'peer_refused':True,'typed_visible_only':True}

  record['steer']={'original_turn':A,'current_turn':B,'first_response':receipt,'retained':saved[0],'exact_retry':old_retry,'new_stale_refusal':fresh_stale,'current_valid':valid_result,'operations_before':before,'operations_after_exact_retry':after,'contract_pass':old_retry==receipt}
 finally:hold.unlink(missing_ok=True)
 (root/'repair2-native-ui-exact-retries.json').write_text(json.dumps({'auth':auth,'binding':bid,'lost':lost,'steer':steer}))
 wait(terminal,8);results.append(record);(root/'repair2-native-ui-result.json').write_text(json.dumps(results,indent=2));print('actual_repair2_native_ui_saved_recovery',topology,json.dumps(record),flush=True)
 assert record['lost_ack']['contract_pass'] and record['steer']['contract_pass'], record
