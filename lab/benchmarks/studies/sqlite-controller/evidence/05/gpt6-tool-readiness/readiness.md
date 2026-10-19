# GPT6 code-mode lead tools: pinned native compatibility

PASS: MODEL-FREE ACTUAL NATIVE PROTOCOL, not live model qualification. Read coordination source a75c520761e5f93f078abdfd1d99f24369499b99. This additive check preserves both earlier topology and GPT5.5 dynamic-tool reports unchanged. No product/worktree edits, account/production credential access, upstream inference, user tmux or repeated topology/multiclient matrix.

The pinned Codex 0.159.3 binary with bundled gpt-6-sol metadata (`tool_mode: code_mode_only`) and `features.code_mode=true`, `features.code_mode_host=true` successfully invoked controller dynamic lead_fetch, lead_ack and lead_answer through genuine native code-mode execution. This closes the specific compatibility gap between the earlier ordinary-function fixture and the intended GPT6 companion profile. It supports choosing this configuration for subsequent integrated qualification; it does not establish live account entitlement, model behavior or complete baseline readiness.

## Exact path observed

The fixture starts the actual app-server via stdio in a cleared child environment with private HOME/CODEX_HOME and an owned workspace. Custom Responses/SSE provider points only to an owned local HTTP server, requires_openai_auth=false, supports_websockets=false; catalog and runtime config are retained. Thread startup registers the same three dynamic function specifications as the previous question fixture. `environments:[]` disables execution environments because this check exercises only controller dynamic tools.

The actual first provider request exposes `functions.exec` as a custom tool in `additional_tools`. Its generated tool description contains real declarations for `tools.lead_fetch`, `tools.lead_ack` and `tools.lead_answer`; see actual-provider-tool-schema.json. The fake provider emits these exact forms sequentially:

    {"type":"custom_tool_call","name":"exec","namespace":"functions","call_id":"code1","input":"text(await tools.lead_fetch({}));"}

    text(await tools.lead_ack({"question_id":"Q_GPT6","operation_id":"GPT6_lead_ack"}));
    text(await tools.lead_answer({"question_id":"Q_GPT6","operation_id":"GPT6_lead_answer","answer":"MODEL_FREE_EXPLICIT_GPT6_ANSWER"}));

No direct Responses `function_call` is emitted to bypass code mode. The actual app-server launches its pinned sibling `codex-code-mode-host`; the fixture observes that executable as a descendant of its positively owned app-server, with PID/PPID/start identity retained in owned-descendants.json.

Each code-mode invocation produces an actual controller request:

    {"method":"item/tool/call","id":0,"params":{"threadId":"...","turnId":"...","callId":"exec-<uuid>","namespace":null,"tool":"lead_fetch","arguments":{}}}

The broker checks the actual thread/turn and uses the unchanged reply shape:

    {"id":0,"result":{"contentItems":[{"type":"inputText","text":"..."}],"success":true}}

The companion returns each broker result in a real `custom_tool_call_output` for outer `code1`, `code2` or `code3`. Subsequent provider input contains fetched question/provenance/state, acknowledged receipt and resolved receipt. Same native dynamic-tool contract survives; a separate product adapter for GPT6 dynamic tools is not indicated by this primitive.

## Correlation and journal consequence

The inner `item/tool/call.params.callId` is generated as `exec-<uuid>` and differs from the outer Responses code-mode call IDs `code1`/`code2`/`code3`. Do not infer the inner call ID from model output or use the outer call as controller command identity. Journal the actual server/incarnation + thread + turn + inner native callId/request ID, scoped principal/epoch, operation ID and argument digest. One outer exec cell can potentially invoke multiple nested tools; this fixture deliberately uses one per cell and does not qualify concurrent nested calls.

The diagnostic SQLite question journal (not Corenius's controller implementation) retained, in order: persisted -> submitted_to_runtime -> acknowledged_by_lead -> resolved. Explicit actual nested lead_ack and lead_answer calls caused the latter states; final transcript text did not. `Q_GPT6`, `ATTEMPT_FIXTURE`, native thread/turn/inner-call provenance and stable operation IDs are retained. The answer is `MODEL_FREE_EXPLICIT_GPT6_ANSWER`.

No transport or authority conclusions from the earlier report are weakened: raw native bearer is server-wide, dynamic requests can broadcast and another authenticated connection can answer them, bound TUI filtering remains necessary, and remote-thread resume after server restart requires explicit reconciliation. Those matrices were not repeated here.

## Evidence and limits

`protocol.jsonl`: full actual native requests/replies, fake provider requests and emitted custom-tool items, journal transitions. `actual-provider-tool-schema.json`: exact actual schema surface. `requests.json`: native provider payloads and code-mode outputs. `tool-calls.json`: three actual nested dynamic requests. `question-fixture.sqlite`: journal. `owned-descendants.json` and `resources.json`: executable/process provenance. `tool-pins.json` and `manifest.json`: pins/content hashes.

`python3 verify.py` passes 13 retained behavioral checks, including actual GPT6 requests, code-mode-only metadata, advertised dynamic declarations, exclusively custom code-mode invocations, three nested native requests, shared thread/turn, distinct inner/outer IDs, returned results, ordered explicit journal transitions, actual pinned companion observation and no remaining owned native PIDs.

Codex SHA256: 8bf204b36a2f6dd0dab73aa2f639892e67ef9ac8befccb4a05b1496ebf25c479.
Companion SHA256: fe64bd08c7844ca1fca91d8cb825e62012a03a05ae8040c635bbb61e5be0b4c5.

This fixture uses host processes with sterile child environment/private directories and owned loopback fake provider. It does not repeat or establish Insula, credential relay, private-network, gateway authorization, server-crash idempotency, concurrent nested calls or live model guarantees. All directly owned app-server and observed companion processes exited; fresh oracle confirms their PIDs absent. Root owns canonical retention and integrated qualification.
