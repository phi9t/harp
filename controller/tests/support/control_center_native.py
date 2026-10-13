"""Actual pinned native/Insula fixture extension. Provider is deterministic and local."""
import concurrent.futures, hashlib, json, os, time, pathlib, sqlite3, uuid

COUNTS = {}

def records(query, collection):
    items=[];cursor=0
    while True:
        page=query('control_records',{'collection':collection,'after':cursor,'limit':20})
        items.extend(page['items'])
        if not page['page']['has_more']:return items
        next_cursor=page['page']['after'];assert next_cursor>cursor
        cursor=next_cursor

def retained_value(query, value):
    if isinstance(value,dict) and value.get('text_reference',{}).get('kind')=='control_input':
        reference=value['text_reference'];chunks=[];offset=0
        while True:
            part=query('control_input',dict(reference,offset=offset))
            assert part['offset']==offset and part['sha256']==reference['sha256'] and part['bytes']==reference['bytes']
            chunks.append(part['text'])
            if part['next_offset'] is None:break
            assert part['next_offset']>offset;offset=part['next_offset']
        content=''.join(chunks)
        assert len(content.encode())==reference['bytes'] and hashlib.sha256(content.encode()).hexdigest()==reference['sha256']
        return content
    if isinstance(value,dict):return {key:retained_value(query,item) for key,item in value.items()}
    if isinstance(value,list):return [retained_value(query,item) for item in value]
    return value

def retained_json(query, value):
    value=retained_value(query,value)
    return json.loads(value) if isinstance(value,str) else value
def provider_item(value, root, request, token):
    raw=json.dumps(value.get('input',[]))
    if 'INTERLEAVING_PEER_TOOL' in raw:
        thread=value.get('client_metadata',{}).get('thread_id','unknown');key=(str(root),'peer-routing',thread);n=COUNTS.get(key,0)+1;COUNTS[key]=n
        if n==1:
            target=json.loads((root/'tool-routing-target.json').read_text())
            if target['topology']=='shared':
                (root/'tool-routing.armed').write_text(json.dumps(target));deadline=time.monotonic()+8
                while not (root/'tool-routing.owner-observed').exists():
                    assert time.monotonic()<deadline,'actual peer tool routing owner RPC deadline';time.sleep(.01)
            return {'type':'custom_tool_call','id':'actual-peer-routing','call_id':'actual-peer-routing','name':'exec','namespace':'functions','input':'text(await tools.corenius_query({"kind":"own_assignment"}));'}
    if 'CORENIUS_QUESTIONS' not in raw and 'CORENIUS_CORRECTION' not in raw:return None
    thread=value.get('client_metadata',{}).get('thread_id','unknown')
    auth={'principal':'operator','token':token}
    center=request(root/'controller.sock',{'auth':auth,'project':'p','query':{'kind':'control_center','args':{}}})['ok']
    actors=[b['attempt'] for b in center['runtime_bindings'] if b['document'].get('thread_id')==thread and b['role']=='attempt']
    corrections=[c for c in center['monitoring']['corrections'] if c['attempt'] in actors and c['state'] in ['prepared','presented'] and 'CORENIUS_CORRECTION '+c['id'] in raw]
    if 'CORENIUS_CORRECTION' in raw and corrections:
        correction=corrections[0];key=(str(root),correction['id']);n=COUNTS.get(key,0)+1;COUNTS[key]=n
        if n>1:return None
        code='text(await tools.corenius_command('+json.dumps({'kind':'ack_correction','operation_id':'fixture-ack-'+correction['id'],'expected_revision':center['revision'],'args':{'correction':correction['id'],'response':'ACTUAL_ADDRESSED_WORKER_ACK'}})+'));'
        return {'type':'custom_tool_call','id':'correction-fixture','call_id':'outer-correction','name':'exec','namespace':'functions','input':code}
    questions=[q for q in center['questions'] if q['answer'] is None and ((isinstance(q.get('text'),str) and q['text'] in raw) or (q['id'] in raw and q.get('delivery') and q['delivery'] in raw))]
    if not questions:return None
    q=questions[0]
    key=(str(root),q['id']);n=COUNTS.get(key,0)+1;COUNTS[key]=n
    if n>3:return None
    if n==1:code='text(await tools.corenius_query('+json.dumps({'kind':'question_input','args':{'question':q['id']}})+'));'
    else:
        kind='ack_question' if n==2 else 'answer_question'
        args={'question':q['id'],'delivery':q['delivery']} if n==2 else {'question':q['id'],'text':'ACTUAL_MODEL_FREE_EXPLICIT_LEAD_ANSWER'}
        code='text(await tools.corenius_command('+json.dumps({'kind':kind,'operation_id':'fixture-'+q['id']+'-'+kind,'expected_revision':center['revision'],'args':args})+'));'
    return {'type':'custom_tool_call','id':'control-fixture-'+str(n),'call_id':'outer-control-'+str(n),'name':'exec','namespace':'functions','input':code}

def exercise(root, query, command, request, grants, lead, wait, topology, token):
    lead_auth={'principal':lead['principal'],'token':lead['token']}
    binding=wait(lambda:next((b for b in query('codex_registry')['bindings'] if b['role']=='lead' and b['document'].get('ui_input_ready')),None))
    ui=request(root/'controller.sock',{'auth':lead_auth,'project':'p','native_ui':{'method':'thread/list','params':{}}})
    assert 'ok' in ui,('scoped native UI list must exist',ui)
    assert [t['id'] for t in ui['ok']['data']]==[binding['document']['thread_id']]
    for method,params in [('thread/read',{'threadId':'foreign-thread'}),('thread/start',{}),('item/tool/call',{}),('config/value/write',{})]:
        refused=request(root/'controller.sock',{'auth':lead_auth,'project':'p','native_ui':{'method':method,'params':params}})
        assert 'error' in refused,(method,refused)
    if os.environ.get('CORENIUS_TEST_UI_REPAIR') in ['replay','interrupt','both']:
        mode=os.environ['CORENIUS_TEST_UI_REPAIR']
        wait(lambda:next(b for b in query('codex_registry')['bindings'] if b['id']==binding['id'])['document'].get('current_turn',{}).get('state')=='terminal',8)
        hold=root/'control-active-hold';hold.write_text('owned real UI active turn for replay/interrupt negative')
        params={'threadId':binding['document']['thread_id'],'clientUserMessageId':'repair1-ui-'+mode,'input':[{'type':'text','text':'CONTROL_ACTIVE_HOLD repair1 exact UI request','text_elements':[]}]}
        def ui_call(method,params):return request(root/'controller.sock',{'auth':lead_auth,'project':'p','native_ui':{'method':method,'params':params}})
        try:
            first=ui_call('turn/start',params);assert 'ok' in first,first
            turn=first['ok']['turn']['id'];assert turn
            active=wait(lambda:next((b for b in query('codex_registry')['bindings'] if b['id']==binding['id'] and b['document'].get('turn_id')==turn and b['document'].get('current_turn',{}).get('state')=='active'),None),8)
            before=query('codex_registry')['operations']
            if mode in ['replay','both']:
                with sqlite3.connect((root/'state.db').as_uri()+'?mode=ro',uri=True) as db:
                    persisted=json.loads(db.execute('SELECT document FROM codex_bindings WHERE id=?',(binding['id'],)).fetchone()[0])
                    saved=[json.loads(raw)['command']['kind'] for raw, in db.execute('SELECT request FROM control_ui_commands WHERE binding=?',(binding['id'],))]
                metadata={'binding':binding['id'],'current_turn':persisted['current_turn'],'turn_id':persisted['turn_id'],'saved_command_kinds':saved,'owned_hold_exists':hold.exists()}
                print('actual_repair1_ui_immediately_before_replay',topology,json.dumps(metadata),flush=True)
                assert metadata['current_turn']['state']=='active' and metadata['turn_id']==turn and saved==['codex_turn'] and metadata['owned_hold_exists']
                second=ui_call('turn/start',params)
                print('actual_repair1_ui_start_replay',topology,json.dumps({'first':first,'replay':second,'operations_before':len(before),'operations_after':len(query('codex_registry')['operations'])}),flush=True)
                assert second==first,'exact native start replay must retain original response across active state'
                assert len(query('codex_registry')['operations'])==len(before)
                changed=dict(params,input=[{'type':'text','text':'changed payload'}]);assert 'error' in ui_call('turn/start',changed)
            if mode in ['interrupt','both']:
                for bad in [{'threadId':params['threadId'],'turnId':'stale-unrelated-turn'},{'threadId':'foreign-thread','turnId':turn}]:assert 'error' in ui_call('turn/interrupt',bad)
                assert len(query('codex_registry')['operations'])==len(before)
                interrupt={'threadId':params['threadId'],'turnId':turn}
                result=ui_call('turn/interrupt',interrupt)
                print('actual_repair1_ui_exact_native_interrupt',topology,json.dumps({'turn':turn,'result':result}),flush=True)
                assert 'ok' in result,'pinned native interrupt has only threadId+turnId'
                assert ui_call('turn/interrupt',interrupt)==result
                assert len(query('codex_registry')['operations'])==len(before)+1
        finally:
            hold.unlink(missing_ok=True)
        wait(lambda:next(b for b in query('codex_registry')['bindings'] if b['id']==binding['id'])['document'].get('current_turn',{}).get('state')=='terminal',8)
    if os.environ.get('CORENIUS_TEST_UI_DRAIN_RACE')=='1':
        wait(lambda:next(b for b in query('codex_registry')['bindings'] if b['id']==binding['id'])['document'].get('current_turn',{}).get('state')=='terminal',8)
        barrier=root/'ui-drain-barrier';barrier.write_text('declared stop background drain after owned lead document read')
        wait(lambda:(root/'ui-drain-barrier.observed').exists(),8)
        observed=json.loads((root/'ui-drain-barrier.observed').read_text())
        with concurrent.futures.ThreadPoolExecutor(max_workers=1) as pool:
            pending=pool.submit(request,root/'controller.sock',{'auth':lead_auth,'project':'p','native_ui':{'method':'turn/start','params':{'threadId':binding['document']['thread_id'],'clientUserMessageId':'owned-ui-drain-race','input':[{'type':'text','text':'LITERAL_UI_DRAIN_CONSISTENCY','text_elements':[]}]}}})
            deadline=time.monotonic()+3
            while not pending.done() and time.monotonic()<deadline:time.sleep(.01)
            completed_before_release=pending.done();barrier.unlink();result=pending.result(timeout=20)
        registry=query('codex_registry');current=next(b for b in registry['bindings'] if b['id']==binding['id'])
        operations=[o for o in registry['operations'] if o['binding']==binding['id'] and o['kind']=='codex_turn' and o['intent'].get('text')=='LITERAL_UI_DRAIN_CONSISTENCY']
        print('actual_UI_drain_interleaving',topology,json.dumps({'background_before':observed,'completed_before_release':completed_before_release,'result':result,'current_binding':current,'operations':operations}),flush=True)
        assert 'ok' in result and len(operations)==1 and operations[0]['state']=='confirmed'
        assert not completed_before_release,'UI must serialize with the background owned binding read, not operate from a stale document'
        assert current['document']['turn_id']==operations[0]['observation']['turn']['id'],'background drain must preserve the confirmed current UI turn'
    if os.environ.get('CORENIUS_TEST_TOOL_INTERLEAVING')=='1':
        worker=wait(lambda:next((b for b in query('codex_registry')['bindings'] if b['attempt']==grants[0]['attempt'] and b['document'].get('current_turn',{}).get('state')=='terminal'),None),8)
        target={'owner':binding['document']['thread_id'],'peer':worker['document']['thread_id'],'topology':topology}
        (root/'tool-routing-target.json').write_text(json.dumps(target))
        operation=command('codex_turn',{'attempt':grants[0]['attempt'],'text':'INTERLEAVING_PEER_TOOL actual own assignment query'})
        def answered():
            invocations=records(query,'tool_invocations')
            return next((call for call in invocations if call['binding']==worker['id'] and retained_json(query,call['arguments'])=={'tool':'corenius_query','arguments':{'kind':'own_assignment'}} and call['response_retained']),None)
        invocation=wait(answered,18)
        with sqlite3.connect((root/'state.db').as_uri()+'?mode=ro',uri=True) as db:
            reply=json.loads(db.execute('SELECT response FROM control_tool_calls WHERE identity=?',(invocation['identity'],)).fetchone()[0])
        actual=json.loads(reply['contentItems'][0]['text']);assert reply['success'] and {task['id'] for task in actual['tasks']}=={'TASK_A'} and len(actual['workers'])==1 and actual['workers'][0]['attempt']['id']==grants[0]['attempt']
        if topology=='shared':
            queued=json.loads((root/'tool-routing.peer-queued').read_text());assert queued['owner_rpc_thread']==target['owner'] and queued['peer_request']['params']['threadId']==target['peer'] and not queued['reply_sent_by_wrong_binding']
            assert not (root/'tool-routing.armed').exists(),'owned transport barrier is one shot'
        else:assert not (root/'tool-routing.peer-queued').exists();queued={'distinct_server_isolation':True}
        assert worker['document']['server_id']==binding['document']['server_id'] if topology=='shared' else worker['document']['server_id']!=binding['document']['server_id']
        assert query('snapshot')['occupied_slots']==2
        print('actual_native_peer_tool_queued_then_validated_owner_replied',topology,json.dumps({'target':target,'operation':operation,'invocation':invocation,'routing':queued}),flush=True)
    worker={'principal':grants[0]['principal'],'token':grants[0]['token']}
    if os.environ.get('CORENIUS_TEST_CONTROL_RACES')=='1':
        wait(lambda:next(b for b in query('codex_registry')['bindings'] if b['id']==binding['id'])['document']['native_turns'][-1]['status']=='completed',8)
        (root/'early-question').write_text('declared force actual early tool acknowledgement before transport confirmation')
        (root/'lose-tool-reply').write_text('declared discard actual prepared acknowledgement reply before native send')
    revision=query('snapshot')['revision']
    result=request(root/'controller.sock',{'auth':worker,'project':'p','command':{'kind':'ask_question','expected_revision':revision,'idempotency_key':'fixture-question','args':{'task':'TASK_A','text':'CORENIUS_QUESTIONS fixture criterion question'}}})
    assert 'ok' in result,result
    question=result['ok']['result']['question']
    def resolved():
        center=query('control_center');q=next(q for q in center['questions'] if q['id']==question)
        return (q,center) if q['delivery_state']=='resolved' else False
    q,center=wait(resolved,18)
    assert q['attempt']==grants[0]['attempt'] and q['answer']=='ACTUAL_MODEL_FREE_EXPLICIT_LEAD_ANSWER'
    history=retained_json(query,q['history'])
    assert any(x['state']=='prepared' for x in history) and any(x['state']=='acknowledged_by_lead' for x in history) and any(x['state']=='resolved' for x in history),history
    if os.environ.get('CORENIUS_TEST_CONTROL_RACES')=='1':
        assert any(x.get('early') for x in history),('actual early acknowledgement not qualified',history)
        assert (root/'early-question.observed').exists()
        early_transport=json.loads((root/'early-question.observed').read_text());assert early_transport.get('actual_native_reply_deferred_before_adapter_return') or early_transport.get('actual_tool_before_native_start_reply'),early_transport
        assert (root/'lose-tool-reply.observed').exists()
        assert len([x for x in history if x['state']=='acknowledged_by_lead'])==1
        calls=[x for x in records(query,'tool_invocations') if retained_json(query,x['arguments']).get('arguments',{}).get('kind')=='ack_question' and retained_json(query,x['arguments'])['arguments']['args']['question']==question]
        assert len(calls)==1 and calls[0]['response_retained'],calls
        wait(lambda:next(b for b in query('codex_registry')['bindings'] if b['id']==binding['id'])['document'].get('native_turns',[])[-1]['status']=='completed',8)
        (root/'lose-ack').write_text('declared question transport acknowledgement loss')
        second=request(root/'controller.sock',{'auth':worker,'project':'p','command':{'kind':'ask_question','expected_revision':query('snapshot')['revision'],'idempotency_key':'fixture-lost-question','args':{'task':'TASK_A','text':'CORENIUS_QUESTIONS lost native transport reply'}}})
        assert 'ok' in second,second
        second_id=second['ok']['result']['question']
        lost=wait(lambda:next((x for x in query('control_center')['questions'] if x['id']==second_id and x['delivery_state']=='resolved'),None),18)
        assert (root/'lose-ack.observed').exists()
        lost_history=retained_json(query,lost['history'])
        assert any(x.get('observation',{}).get('reconciled_from_native_history') for x in lost_history),lost_history
        operations=query('codex_registry')['operations'];assert len([o for o in operations if any(x.get('id')==second_id for x in o.get('intent',{}).get('questions',[]))])==1
        center=query('control_center')
        print('actual_early_ack_and_lost_question_transport_reconciliation',topology,json.dumps({'early':q,'lost':lost}),flush=True)
        turn_race(root,query,request,worker,lead_auth,binding,wait,token,topology)
        center=query('control_center')
    binding=next(b for b in query('codex_registry')['bindings'] if b['role']=='lead' and b['authority_epoch']==center['lead_epoch'])
    assert any(any(i.get('type')=='userMessage' and 'CORENIUS_QUESTIONS' in json.dumps(i) for i in t['items']) for t in binding['document']['native_turns'])
    assert center['occupied_slots']==2 and len(center['workers'])==2
    assert any(c['binding']==binding['id'] for c in center['tool_invocations'])
    if os.environ.get('CORENIUS_TEST_MONITOR_NATIVE')=='1':
        streams=center['monitoring']['streams']
        assert len({s['attempt'] for s in streams})==2,('actual visible worker ingestion missing',streams)
        assert all(s['reviewed_history']==0 and s['ingested']>0 for s in streams)
        inspect=next(s for s in streams if s['attempt']==grants[0]['attempt'])
        sealed_input=command('seal_monitor_batch',{'stream':inspect['id'],'lane':'history','through':min(inspect['ingested'],20),'boundary':'manual'})
        raw_batch=request(root/'controller.sock',{'auth':{'principal':'operator','token':token},'project':'p','query':{'kind':'monitor_batch','args':{'batch':sealed_input['batch']}}})['ok']
        first_input=json.loads(raw_batch['inputs'])[0]
        observer=command('create_monitor_observer',{'name':'owned-native-bounded-analyzer'})
        inspected=request(root/'controller.sock',{'auth':{'principal':observer['principal'],'token':observer['token']},'project':'p','gateway':{'tool':'corenius_query','arguments':{'kind':'monitor_input','args':{'batch':sealed_input['batch'],'item':first_input['item']}}}})
        assert 'ok' in inspected,inspected
        retained=inspected['ok'];assert len(retained['redaction']['original_source_digest'])==64 and retained['redaction']['redacted_input_digest']==retained['digest']
        assert retained['input'].get('type')!='reasoning' and 'Bearer ' not in json.dumps(retained['input'])
        print('actual_native_allowed_input_retained_scoped_observer_read',topology,json.dumps({'digest':retained['digest'],'redaction':retained['redaction']}),flush=True)
        if os.environ.get('CORENIUS_TEST_CONTROL_RACES')=='1':
            stream=next(s for s in reversed(streams) if s['attempt']==grants[0]['attempt'])
            sealed=command('seal_monitor_batch',{'stream':stream['id'],'lane':'history','through':min(stream['ingested'],20),'boundary':'manual'})
            batch=request(root/'controller.sock',{'auth':{'principal':'operator','token':token},'project':'p','query':{'kind':'monitor_batch','args':{'batch':sealed['batch']}}})
            assert 'ok' in batch,batch
            inputs=json.loads(batch['ok']['inputs']);steps=[{'item':x['item'],'criterion':'fixture-original-intent','rationale':'inspect owned visible result','observation':x['text'],'assessment':'valid_learning','next_test':'bounded diagnostic'} for x in inputs]
            report=command('publish_monitor_assessment',{'batch':sealed['batch'],'analyst':'model-free-fixture','version':'v1','steps':steps,'findings':[{'issue':'fixture-bounded-correction','criterion':'fixture-original-intent','text':'source-bound advisory correction test','items':[inputs[0]['item']]}],'evidence':[]})
            finding=next(f for f in query('control_center')['monitoring']['findings'] if f['issue']=='fixture-bounded-correction')
            command('adjudicate_monitor_finding',{'finding':finding['id'],'decision':'accepted','rationale':'operator independently authorizes fixture action'})
            correction=command('authorize_correction',{'finding':finding['id'],'expected_epoch':finding['epoch'],'criterion':finding['criterion'],'action':'bounded diagnostic only','prohibited_scope':'do not change acceptance or dispatch another actor','next_checkpoint':'one public result','rationale':'source-bound fixture qualification'})
            acknowledged=wait(lambda:next((x for x in query('control_center')['monitoring']['corrections'] if x['id']==correction['correction'] and x['state']=='acknowledged'),None),18)
            assert retained_json(query,acknowledged['acknowledgement'])['principal']==grants[0]['principal']
            command('resolve_correction',{'correction':correction['correction'],'evidence':report['assessment'],'rationale':'actual addressed worker acknowledged; controller retained typed tool receipt'})
            assert query('snapshot')['occupied_slots']==2
            print('actual_advisory_finding_authenticated_correction_worker_ack',topology,json.dumps(acknowledged),flush=True)
    if os.environ.get('CORENIUS_TEST_QUESTION_BYTES')=='1':
        import hashlib
        large_ids=[];expected={};hold=root/'control-active-hold';hold.write_text('hold owned lead while durable questions enqueue')
        try:
            for index in range(10):
                content=('L'*65536) if index==0 else ('M'*16000+str(index)) if index<9 else 'later original question must progress'
                asked=command('ask_question',{'task':'TASK_A','text':content});qid=asked['question'];large_ids.append(qid);expected[qid]=hashlib.sha256(content.encode()).hexdigest()
        finally:hold.unlink(missing_ok=True)
        def all_large_resolved():
            rows=[q for q in query('control_center')['questions'] if q['id'] in large_ids]
            return rows if len(rows)==10 and all(q['delivery_state']=='resolved' for q in rows) else False
        rows=wait(all_large_resolved,40)
        operations=query('codex_registry')['operations'];deliveries=[o for o in operations if any(q['id'] in large_ids for q in o.get('intent',{}).get('questions',[]))]
        descriptors=[q for o in deliveries for q in o['intent']['questions'] if q['id'] in large_ids]
        assert all(len(o['intent']['text'].encode())<=65536 for o in deliveries)
        assert {q['id'] for q in descriptors}==set(large_ids) and all(q['text_digest']==expected[q['id']] for q in descriptors)
        assert any(q.get('text_reference',{}).get('bytes')==65536 for q in descriptors)
        for qid in large_ids:
            exact=request(root/'controller.sock',{'auth':lead_auth,'project':'p','gateway':{'tool':'corenius_query','arguments':{'kind':'question_input','args':{'question':qid}}}})['ok']
            assert exact['sha256']==expected[qid] and hashlib.sha256(exact['text'].encode()).hexdigest()==expected[qid] and exact['bytes']==len(exact['text'].encode())
        assert query('snapshot')['occupied_slots']==2
        print('actual_repair1_large_question_refs_and_later_delivery',topology,json.dumps({'questions':len(rows),'payload_digests':expected,'delivery_count':len(deliveries),'native_text_bytes':[len(o['intent']['text'].encode()) for o in deliveries],'later_resolved':rows[-1]['delivery_state']}),flush=True)
    (root/'control-center-outcome.json').write_text(json.dumps({'topology':topology,'question':q,'projection':center,'binding':binding},indent=2))
    print('actual_product_lead_question_delivered_acknowledged_resolved',topology,json.dumps(q),flush=True)
    if os.environ.get('CORENIUS_TEST_UI_FIXTURE')=='1':
        attach(root,query,request,lead,binding,wait,topology)

def turn_race(root,query,request,worker,lead_auth,binding,wait,token,topology):
    from wsutil import WS
    wait(lambda:next(b for b in query('codex_registry')['bindings'] if b['id']==binding['id'])['document']['native_turns'][-1]['status']=='completed',8)
    (root/'control-active-hold').write_text('declared block owned local lead response')
    started=request(root/'controller.sock',{'auth':lead_auth,'project':'p','native_ui':{'method':'turn/start','params':{'threadId':binding['document']['thread_id'],'clientUserMessageId':'race-owned-literal','input':[{'type':'text','text':'CONTROL_ACTIVE_HOLD owned deterministic lead'}]}}})
    assert 'ok' in started,started
    wait(lambda:next(b for b in query('codex_registry')['bindings'] if b['id']==binding['id'])['document']['native_turns'][-1]['status']=='inProgress',8)
    (root/'question-race').write_text('declared finish actual turn after durable steer preparation')
    asked=request(root/'controller.sock',{'auth':worker,'project':'p','command':{'kind':'ask_question','expected_revision':query('snapshot')['revision'],'idempotency_key':'fixture-race-question','args':{'task':'TASK_A','text':'CORENIUS_QUESTIONS actual finish-before-steer race'}}})
    assert 'ok' in asked,asked
    qid=asked['ok']['result']['question'];wait(lambda:(root/'question-race.observed').exists(),8)
    prepared=next(q for q in query('control_center')['questions'] if q['id']==qid);assert prepared['delivery_state']=='prepared'
    (root/'control-active-hold').unlink()
    current=next(b for b in query('codex_registry')['bindings'] if b['id']==binding['id'])['document']
    app=request(root/'controlplanes.sock',{'token':'native-controlplane-secret','action':'native_lookup','binding':current['server_execution']['binding']})['ok']
    runtime=app['runtime'];ws=WS.connect('localhost',8788,app['capability'],unix=runtime['endpoint'],path='/?corenius_start='+runtime['runtime_start']);ws.s.settimeout(8)
    def rpc(method,params):
        identifier='race-read-'+str(uuid.uuid4());ws.send({'id':identifier,'method':method,'params':params})
        while True:
            message=ws.recv()
            if message.get('id')==identifier:assert 'result' in message,message;return message['result']
    try:
        rpc('initialize',{'clientInfo':{'name':'owned-exact-race-read-only','version':'1'},'capabilities':{'experimentalApi':True}});ws.send({'method':'initialized','params':{}})
        terminal=wait(lambda:(lambda x:x if x['thread']['turns'][-1]['status']=='completed' else False)(rpc('thread/read',{'threadId':current['thread_id'],'includeTurns':True})),8)
        (root/'race-actual-terminal.json').write_text(json.dumps(terminal))
    finally:ws.close()
    (root/'question-race').unlink();wait(lambda:(root/'question-race.failed').exists(),8)
    pending=next(q for q in query('control_center')['questions'] if q['id']==qid);assert pending['delivery_state']=='persisted' and pending['delivery']==prepared['delivery'],pending
    old=next(o for o in query('codex_registry')['operations'] if o['id']==prepared['delivery']);assert old['state']=='failed' and 'native RPC refused:' in old['observation']['error'],old
    (root/'question-race.continue').write_text('pending state independently observed; allow next current native history reconciliation')
    resolved=wait(lambda:next((q for q in query('control_center')['questions'] if q['id']==qid and q['delivery_state']=='resolved'),None),18)
    assert resolved['delivery']!=prepared['delivery']
    print('actual_finish_before_steer_refusal_retained_pending_then_resolved',topology,json.dumps({'prepared':prepared,'pending':pending,'failed':old,'resolved':resolved}),flush=True)

def attach(root, query, request, lead, binding, wait, topology):
    import subprocess,sys,pty,fcntl,termios,struct,threading,select,hashlib
    from wsutil import WS
    token='owned-ui-fixture-capability'
    gateway_env={'PATH':'/usr/bin:/bin','CORENIUS_CLIENT_PRINCIPAL':lead['principal'],'CORENIUS_CLIENT_TOKEN':lead['token'],'CORENIUS_UI_TOKEN':token,'CORENIUS_UI_AUDIT':str(root/'gateway-audit.jsonl')}
    gateway_log=open(root/'gateway.stdout','wb');gateway_err=open(root/'gateway.stderr','wb')
    gateway=subprocess.Popen([sys.argv[1],'--native-ui',str(root/'controller.sock'),'p','127.0.0.1:0'],env=gateway_env,stdout=gateway_log,stderr=gateway_err)
    tui=None;master=None;reader=None;resources=[{'name':'gateway','pid':gateway.pid,'start_stat':pathlib.Path('/proc/'+str(gateway.pid)+'/stat').read_text()}]
    try:
        def listening():
            assert gateway.poll() is None,(root/'gateway.stderr').read_text()
            raw=(root/'gateway.stdout').read_text()
            return int(raw.split('Gateway listening 127.0.0.1:',1)[1].splitlines()[0]) if 'Gateway listening' in raw else False
        port=wait(listening,10)
        auth_ws=WS.connect('localhost',port,token)
        auth_ws.send({'id':'forged-tool-reply','result':{'success':True}})
        assert 'error' in auth_ws.recv();auth_ws.close()
        home=root/'ui-home';home.mkdir();(home/'codex').mkdir()
        master,slave=pty.openpty();fcntl.ioctl(slave,termios.TIOCSWINSZ,struct.pack('HHHH',42,132,0,0))
        env={'PATH':'/usr/bin:/bin','HOME':str(home),'CODEX_HOME':str(home/'codex'),'TERM':'xterm-256color','LANG':'C.UTF-8','CORENIUS_VIEW_TOKEN':token}
        binary=pathlib.Path('/data02/home/philip.yang/.codex/packages/standalone/releases/0.159.3-x86_64-unknown-linux-musl/bin/codex')
        argv=[str(binary),'--remote','ws://127.0.0.1:'+str(port),'--remote-auth-token-env','CORENIUS_VIEW_TOKEN','resume',binding['document']['thread_id'],'--no-alt-screen']
        tui=subprocess.Popen(argv,env=env,stdin=slave,stdout=slave,stderr=slave,start_new_session=True);os.close(slave)
        resources.append({'name':'native-tui','pid':tui.pid,'start_stat':pathlib.Path('/proc/'+str(tui.pid)+'/stat').read_text(),'argv':argv,'binary_sha256':hashlib.sha256(binary.read_bytes()).hexdigest()})
        def drain():
            with open(root/'native-tui.pty','wb') as out:
                while tui.poll() is None:
                    if select.select([master],[],[],.1)[0]:
                        try:data=os.read(master,65536)
                        except OSError:break
                        if not data:break
                        out.write(data);out.flush()
        reader=threading.Thread(target=drain,daemon=True);reader.start()
        def attached():
            assert tui.poll() is None,('native TUI exited', (root/'native-tui.pty').read_bytes()[-2500:])
            lines=(root/'gateway-audit.jsonl').read_text().splitlines()
            return any(json.loads(line).get('method')=='thread/resume' and json.loads(line).get('accepted') for line in lines)
        wait(attached,15)
        prompt='UI_BOUND_INPUT literal owned fixture; answer once using the registered environment.'
        (root/'literal-ui-input.txt').write_text(prompt)
        os.write(master,b'\x1b[200~'+prompt.encode()+b'\x1b[201~');os.write(master,b'\r')
        def delivered():
            assert tui.poll() is None,('native TUI exited',(root/'native-tui.pty').read_bytes()[-2500:])
            current=next(b for b in query('codex_registry')['bindings'] if b['id']==binding['id'])
            return current if any('UI_BOUND_INPUT' in json.dumps(t.get('items',[])) for t in current['document'].get('native_turns',[])) else False
        observed=wait(delivered,18)
        assert query('snapshot')['occupied_slots']==2
        audit=[json.loads(x) for x in (root/'gateway-audit.jsonl').read_text().splitlines()]
        assert any(x.get('method')=='thread/list' and x.get('accepted') for x in audit)
        assert any(x.get('request_id')=='forged-tool-reply' and not x.get('accepted') for x in audit)
        assert not any(x.get('method') in ['thread/start','item/tool/call','config/value/write'] and x.get('accepted') for x in audit)
        print('actual_native_scoped_tui_literal_input_accepted_no_worker_slot',topology,json.dumps({'binding':observed['id'],'thread':observed['document']['thread_id'],'audit':audit}),flush=True)
        (root/'ui-outcome.json').write_text(json.dumps({'binding':observed,'audit':audit,'resources':resources},indent=2))
    finally:
        for name,p in [('native-tui',tui),('gateway',gateway)]:
            if p is not None:
                if p.poll() is None:p.kill()
                p.wait(timeout=5)
                next(x for x in resources if x['name']==name)['exit']=p.returncode
        if reader is not None:reader.join(2)
        if master is not None:os.close(master)
        gateway_log.close();gateway_err.close()
        (root/'ui-resources.json').write_text(json.dumps(resources,indent=2))
        print('owned_ui_resources_terminated',json.dumps(resources),flush=True)
