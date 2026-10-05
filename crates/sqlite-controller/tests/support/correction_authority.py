"""Public/native regression for stale correction authorization and fresh grants."""
import json,sqlite3,time,uuid,pathlib
def exercise(root,query,command,request,grants,lead,wait,topology,token):
 out=root
 operator={'principal':'operator','token':token};oldlead={k:lead[k] for k in ['principal','token']}
 def cmd(kind,args,actor=operator):
  value=request(root/'controller.sock',{'auth':actor,'project':'p','command':{'kind':kind,'idempotency_key':str(uuid.uuid4()),'expected_revision':query('snapshot')['revision'],'args':args}})
  assert 'ok' in value,value
  return value['ok']['result']
 def correction_rows():
  return query('monitor_records',{'collection':'corrections','limit':20})['items']
 def byid(id):return next(x for x in correction_rows() if x['id']==id)
 for grant in grants:
  wait(lambda:any(b['attempt']==grant['attempt'] and b['document'].get('ui_input_ready') and b['document'].get('current_turn',{}).get('state')=='terminal' for b in query('codex_registry')['bindings']))
 (root/'hold-codex').write_text('owned focused correction admission ordering; no product state edits')
 try:
  findings=[]
  for j,g in enumerate(grants):
   stream=cmd('register_monitor_stream',{'attempt':g['attempt'],'source':{'deployment':'independent-public-fixture','store':'retained','thread':'review-'+str(j),'segment':'one','parser':'visible-v1','identity':'explicit-fixture-observation'},'live_baseline':0})['stream']
   cmd('ingest_monitor_events',{'stream':stream,'expected_ordinal':0,'previous_digest':'','events':[{'item':'one','turn':'declared-observation','kind':'public_message','text':'independent public protocol input; not claimed model analysis','ordinal':1,'byte_end':20,'digest':'d1'}]})
   batch=cmd('seal_monitor_batch',{'stream':stream,'lane':'history','through':1,'boundary':'manual'})['batch']
   cmd('publish_monitor_assessment',{'batch':batch,'analyst':'independent-fixture','version':'1','steps':[{'item':'one','criterion':'admitted-intent','hypothesis':'test authority transition','rationale':'declared API test','observation':'public fixture input','assessment':'valid_learning','next_test':'exact correction admission'}],'findings':[{'issue':'independent-'+str(j)+'-'+str(n),'criterion':'admitted-intent','text':'fixture finding requires authenticated decision','items':['one']} for n in range(2 if j==0 else 1)],'evidence':[]})
   fs=[x for x in query('monitor_context')['findings'] if x['attempt']==g['attempt'] and x['issue'].startswith('independent-')];assert len(fs)==(2 if j==0 else 1);findings.append(fs)
  def args(f):return {'finding':f['id'],'expected_epoch':f['epoch'],'criterion':'admitted-intent','action':'Independent bounded correction delivery','prohibited_scope':'no expanded authority','next_checkpoint':'explicit actor response','rationale':'independent regression test'}
  old=findings[0][0];cmd('adjudicate_monitor_finding',{'finding':old['id'],'decision':'accepted','rationale':'old lead authenticated decision'},oldlead)
  first=cmd('authorize_correction',args(old),oldlead)
  assert byid(first['correction'])['state']=='requested'
  replacement=cmd('replace_lead',{})
  currentlead={k:replacement[k] for k in ['principal','token']}
  others=[]
  for f,actor in [(findings[0][1],currentlead),(findings[1][0],operator)]:
   cmd('adjudicate_monitor_finding',{'finding':f['id'],'decision':'accepted','rationale':'current authenticated decision'},actor)
   others.append(cmd('authorize_correction',args(f),actor))
  before=correction_rows()
 finally:(root/'hold-codex').unlink(missing_ok=True)
 # Peer correction proves actual autonomous native dispatch is progressing, without allowing stale authority.
 wait(lambda:byid(others[1]['correction'])['state']=='presented',20)
 try:wait(lambda:byid(others[0]['correction'])['state'] in ('presented','acknowledged'),12)
 except AssertionError:pass
 registry=query('codex_registry');after=correction_rows()
 positive=byid(others[1]['correction']);blocked=byid(others[0]['correction']);stale=byid(first['correction'])
 original_author=next(x for x in query('control_center')['decisions'] if x['id']==stale['decision'])['author']
 with sqlite3.connect((root/'state.db').as_uri()+'?mode=ro',uri=True) as db:
  ops=db.execute("SELECT id,state,intent FROM codex_operations WHERE json_extract(intent,'$.correction') IN (?,?,?)",(first['correction'],others[0]['correction'],others[1]['correction'])).fetchall()
 old_refusal=request(root/'controller.sock',{'auth':oldlead,'project':'p','query':{'kind':'monitor_context','args':{}}});assert 'error' in old_refusal
 result={'topology':topology,'old_lead':oldlead['principal'],'current_lead':currentlead['principal'],'original_author_retained':original_author,'reauthorization':'not attempted before fairness discriminator','before':before,'after':after,'native_correction_operations':ops,'peer_positive':positive,'current_authorized_behind_stale':blocked,'stale':stale,'old_lead_refused':old_refusal,'contract_pass':blocked['state'] in ('presented','acknowledged'),'qualification':'Actual native shared runtime; public fixture observations/findings explicitly synthetic; all auth/replacement/correction requests real. Peer presented proves dispatch progress; no old-grant execution allowed.'}
 (out/'correction-result.json').write_text(json.dumps(result,indent=2));(root/'independent-correction-result.json').write_text(json.dumps(result,indent=2));print('independent_correction_authority_queue',json.dumps(result),flush=True)
 assert positive['state'] in ('presented','acknowledged'), positive
 assert result['contract_pass'], 'superseded authorization starves later current correction'
 assert stale['state']=='authorization-stale' and stale['operation'] is None, stale
 reauthorized=cmd('authorize_correction',args(old),operator)
 assert reauthorized['correction']==first['correction'] and reauthorized['decision']!=stale['decision'], reauthorized
 wait(lambda:byid(first['correction'])['state'] in ('presented','acknowledged'),20)
 history=query('control_center')['decisions']
 assert any(d['id']==stale['decision'] and d['author']==oldlead['principal'] for d in history)
 assert any(d['id']==reauthorized['decision'] and d['author']=='operator' for d in history)
 again=cmd('authorize_correction',args(old),operator)
 assert again['existing'] and again['correction']==first['correction']
 result['reauthorization']=reauthorized;result['final']=correction_rows();result['decision_history']=history
 (out/'correction-result.json').write_text(json.dumps(result,indent=2));(root/'independent-correction-result.json').write_text(json.dumps(result,indent=2));print('fresh_authenticated_regrant',json.dumps(result),flush=True)
 return
