from pathlib import Path
import json
r=Path(__file__).parent
old=json.loads((r/'proxy-first/proxy-upstream-seen.json').read_text());new=json.loads((r/'proxy-upstream-seen.json').read_text());out1=json.loads((r/'proxy-first/proxy-outcomes.json').read_text());out2={x['case']:x for x in json.loads((r/'proxy-outcomes.json').read_text())};sink=json.loads((r/'proxy-redirect-sink.json').read_text());summary=json.loads((r/'retained-evidence-summary.json').read_text())
checks={
 'retained_native_tui_no_account_required':summary['actual_native_tui_account_read']['message']['result']=={'account':None,'requiresOpenaiAuth':False,'workspaceRouting':None},
 'owned_native_fixtures_have_no_auth_file':not any(summary['native_auth_files_exist_in_owned_fixtures'].values()),
 'proxy_overrides_authorization':old[1]['headers']['authorization']=='Bearer sk-owned-fixture-not-real',
 'proxy_forwards_untrusted_account_header':old[1]['headers']['chatgpt-account-id']=='UNTRUSTED_ACCOUNT',
 'proxy_forwards_unqualified_model':'UNQUALIFIED_MODEL' in old[2]['body'],
 'proxy_rejects_unqualified_paths_and_connect':all(x['status']==403 and x['upstream_requests_added']==0 for x in out1[3:]),
 'proxy_accepts_opaque_synthetic_bearer':new[0]['headers']['authorization']=='Bearer opaque-fixture-oauth-not-real',
 'proxy_follows_redirect':out2['redirect']['status']==200 and len(sink)==1,
 'redirect_drops_bearer_but_forwards_account_and_body':'authorization' not in sink[0]['headers'] and sink[0]['headers']['chatgpt-account-id']=='SYNTHETIC_ACCOUNT' and 'gpt-6-sol' in sink[0]['body'],
 'small_event_buffered_in_probe':out2['normal']['first_byte_elapsed']>.25,
 'large_event_streamed_before_completion':out2['large-first']['first_byte_elapsed']<.15 and out2['large-first']['completion_elapsed']>.25,
 'proxy_preserves_synthetic401':out2['unauthorized']['status']==401 and 'invalid_api_key'in out2['unauthorized']['body_excerpt'],
 'owned_proxy_processes_gone':all(x['terminated'] and not Path('/proc',str(x['pid'])).exists() for p in [r/'proxy-resources.json',r/'proxy-first/proxy-resources.json'] for x in json.loads(p.read_text()))}
(r/'oracle-results.json').write_text(json.dumps(checks,indent=2));print(json.dumps(checks,indent=2));assert all(checks.values())
