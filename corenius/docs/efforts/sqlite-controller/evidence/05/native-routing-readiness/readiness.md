# Native lead tooling and TUI routing, Codex 0.159.3

MODEL-FREE ACTUAL NATIVE PROTOCOL. Source read: a75c520761e5f93f078abdfd1d99f24369499b99, ticket05 brief and canonical CONTROLLED-TRIAL. This follows the retained topology report; it does not supersede its remote-environment restart limit. All Responses/SSE traffic terminates at an owned local fake provider, whose scripted replies cause the real pinned app-server to dispatch tools. No production credential, account, upstream inference, user tmux, repository edit or product gateway implementation. Native CLI authentication here uses only a newly created synthetic fixture capability token.

## Main finding

A bound controller gateway is feasible and necessary. Native authentication grants server-wide access; it does not establish exclusive ownership of dynamic-tool requests. Native TUI resume did not take requests away from the controller: requests were broadcast to both subscribed clients. More strongly, an initialized authenticated connection that never subscribed to the thread successfully answered a tool request using its native JSON-RPC ID. Its result reached the provider input; a later controller response did not replace it. Therefore never hand the native bearer/endpoint to an untrusted viewer or worker.

Actual TUI also automatically requests global thread lists, reads sibling thread histories, and creates ephemeral thread/start + turn/start helpers to generate titles. These are native UI helper turns, not controller task assignments, but they are additional model work and cannot be silently admitted by a supposedly thread-bound viewer. The title helper made real requests to the fake provider in raw-routing runs; no paid calls occurred.

The final bounded recorder/filter fixture kept the actual native TUI usable while rejecting eight automatic helper thread/start requests, filtering thread-list results to the bound thread, withholding eleven dynamic-tool requests, and admitting literal bracketed-paste human input only to the bound lead thread. Its gateway is a diagnostic routing filter, not a production authentication or exhaustive authorization implementation. Its local incoming WebSocket handshake is not an authentication proof. Native upstream connections did authenticate using the fixture capability token.

## Versioned native contract actually exercised

Server: codex app-server --listen ws://127.0.0.1:PORT --ws-auth capability-token --ws-token-file OWNED_FILE. Clients use WebSocket Authorization: Bearer SYNTHETIC_TOKEN. initialize includes clientInfo and capabilities.experimentalApi=true.

thread/start dynamicTools contains function entries:

    {"type":"function","name":"lead_fetch","description":"...","inputSchema":{"type":"object","properties":{},"additionalProperties":false}}

lead_ack and lead_answer add question_id, operation_id and answer fields. These names are fixture operations, not claims of existing Corenius product tools. Model metadata is the pinned bundled gpt-5.5 entry with native ordinary function tools; code_mode and code_mode_host are disabled in this specific fixture. The previous GPT6 companion-host findings still apply to GPT6. Neither model metadata nor fake responses establish account entitlement or reasoning quality.

A scripted Responses function_call item causes the actual server request:

    {"method":"item/tool/call","id":11,"params":{"threadId":"...","turnId":"...","callId":"call20","namespace":null,"tool":"lead_ack","arguments":{"question_id":"Q_LOST","operation_id":"LOST_ACK_STABLE_OP"}}}

Successful response shape:

    {"id":11,"result":{"contentItems":[{"type":"inputText","text":"..."}],"success":true}}

The resulting native function_call_output is retained in subsequent provider input. Native request ID is not a durable controller command identity. Persist server identity/incarnation + thread + turn + callId + native request ID, request digest, scoped principal/role epoch, and controller operation idempotency identity. Integer IDs can be known by other clients and cannot be treated as a secret or permission.

Actual native TUI command: codex --remote ws://127.0.0.1:RECORDER --remote-auth-token-env FIXTURE_NATIVE_TOKEN resume EXACT_THREAD --no-alt-screen, in a new owned PTY. GPT5.5 initially displayed a model-upgrade prompt; fixed down/enter input selected the existing model. No new user session was used. TUI emitted initialize, thread/read and thread/resume with the exact bound thread. Controller re-resume did not remove the TUI subscriber or stop broadcast.

## Raw routing evidence

run3/protocol.jsonl retains the native controller-only and transparent TUI tests. Controller-only tool call reached controller. After TUI resume, identical item/tool/call JSON-RPC requests reached controller and TUI; after controller re-resume, both still received requests. TUI sent no tool response in these observed runs; controller responded successfully. Do not generalize this to exclusive ownership: run4 and current protocol.jsonl show the deliberately unsubscribed authenticated client's FOREIGN_CONNECTION_RESPONSE in actual provider tool output. The fixture shares the known request ID deliberately to test the server boundary; no unrelated server was accessed.

run4 also retains unfiltered TUI helper creation and provider title-generation requests. The fake responder originally used a global response sequence; helper turns perturbed it and an answer call went to a helper without tools, returning unsupported call. This failed fixture version is preserved. Final fixture classifies requests by actual client_metadata.thread_id and filters helper creation, so the question oracle uses only the bound lead thread. No failed variant is counted as successful question resolution.

## Question delivery and explicit state evidence

question-fixture.sqlite is a small independent diagnostic journal, not Corenius's controller database or implementation. It records question ID, attempt provenance, text, state, answer and receipts with native correlation. Multiple fixture generations are preserved in the journal; current protocol's thread ID identifies final-run receipts. This journal and the actual native protocol establish the primitive plumbing only.

Q_IDLE from ATTEMPT_A starts persisted. The controller sends its question text through turn/start. The returned turn receipt is recorded as submitted_to_runtime. The deterministic fake model explicitly calls lead_fetch, then lead_ack, then lead_answer. Actual item/tool/call requests drive acknowledged_by_lead and resolved receipts. Fetch returns the persisted ID, provenance, question text and current state. Final answer is MODEL_FREE_EXPLICIT_LEAD_ANSWER.

Q_ACTIVE from ATTEMPT_B is persisted while an actual native turn awaits its held fake-provider response. A wrong expectedTurnId receives an exact mismatch rejection. A correct turn/steer with the active ID accepts question text and returns that same turn ID; only this marks submitted_to_runtime. The held stream then invokes fetch/ack/answer, completing the explicit state progression. Steering the same already-completed turn returns no active turn to steer. The corresponding delivery must remain pending rather than silently starting another turn.

Actual thread/items/list history contains the idle question text, active question text and literal UI_BOUND_INPUT. This proves native retained text transport, not just terminal rendering. Native turn completion alone does not mark a question acknowledged or resolved. The fake model is intentionally scripted to invoke the explicit tools; no claim about a live model reliably choosing those tools is made.

The fixture holds its first fake response until the submission receipt is recorded, solely to separate observed states. It does not qualify an acknowledgement racing ahead of submission confirmation. Product logic must correlate an early explicit ack with the prepared delivery operation, atomically reconcile the delivery evidence, and avoid dropping valid acknowledgement merely because the transport receipt is still unknown.

## Lost tool response and reconnect

The fixture commits Q_LOST acknowledgement on the actual lead_ack request, then intentionally omits the native response and closes only the controller connection. TUI remains attached behind the filter, which does not answer the request. A new authenticated controller initializes and thread/resume rejoins the same thread, returning active state. Native server replays the identical pending item/tool/call: same request ID 11, thread, turn, callId call20, operation ID and arguments. The new fixture controller returns a response without recording the acknowledgement effect again; the original turn completes. The final journal contains exactly one Q_LOST acknowledgement effect.

This is observed pending-request replay across connection loss in the same server incarnation. It is not an exactly-once native tool guarantee or server-crash recovery. The fixture's replacement reply is a transport success payload after checking the retained test setup, not a complete product idempotency implementation. In the product, first reconcile the durable operation/arguments/authority, then resend its recorded result. Do not reapply the effect because a native request was replayed, and do not reuse a stale response after runtime incarnation changes.

## Feasible ticket05 gateway contract (design inference)

1. Native endpoint and token remain controller-side. Give the TUI a separate gateway capability bound to project, lead/attempt role, authority epoch, registered server/store/incarnation and exact thread. Operator input may use that view; workers cannot acquire the raw native bearer or controller tool authority. Keep gateway identity distinct from provider and worker grants.
2. Use a controller-owned native upstream connection for dynamic tools, with a persisted request/result journal. Intercept item/tool/call before any UI forwarding. Reject UI-originated JSON-RPC responses and unrecognized requests, even when an attacker guesses a native request ID. Subscribe and reconcile replay on the controller connection. Do not use last-resume ownership assumptions.
3. Provide a pinned allowlist for TUI bootstrap/read requests. In this pin the native TUI asked for account/read, model/list, configRequirements/read, collaborationMode/list, hooks/list, thread/read, thread/resume, thread/turns/list, thread/items/list, thread/loaded/list, thread/list, thread/goal/get, skills/list and config/read. Some contain paths/cursors or expose server-wide data: sanitize to the registered cwd and thread, validate cursor ownership, filter list payloads and native notifications, and return bounded compatible responses where unrestricted operations are unnecessary. The diagnostic filter only exercises some of these controls; it is not the complete allowlist implementation.
4. Deny TUI thread/start/fork/import, arbitrary history/path resume, environment mutation, unrelated thread queries, config writes, host fs/process/command APIs, account changes and arbitrary tool replies. Native TUI title helpers can receive a structured rejection; the fixture proved the existing thread remained usable afterward. Filter notifications as well as responses; thread identity restrictions cannot be a UI convention.
5. Route human input through the normal durable controller command path. Bind to the current registered thread and authority; drop caller model/provider/cwd/permissions/environment overrides. At idle, prepare the delivery then start with the validated binding. During an active turn, steer with its expected turn identity; on a race return pending. The final fixture sanitized idle TUI turn/start to bound thread + input and observed completion. Product durable journaling, active human-input translation, authority revocation and all denial cases remain implementation/qualification work.
6. Questions carry immutable question ID, attempt provenance, delivery ID/content digest and stable operation IDs. Transport acceptance advances submitted_to_runtime only; explicit scoped lead_ack establishes acknowledgement, and explicit lead_answer records answer/resolution. Scope and current epoch are checked at every call. Tool parameters cannot select another principal, project, controller database, native server or workspace. Deduplicate controller effects by operation ID+argument digest and reconcile before responding to native replay.
7. Preserve the prior restart guard: app-server restart can resume a remote thread with local environment. Gateway blocks input and runtime mutation until supervisor/store/incarnation/environment/workspace bindings are reconciled. Every worker turn includes validated explicit environments. Sharing native server infrastructure never grants peer/controller filesystem or process access.

## Evidence, limits and cleanup

Run `python3 verify.py` from this owned root to check 15 retained behavioral oracles. oracle-results.json records all true: broadcast, foreign reply, helper rejection, withheld requests, native bound input, three retained text markers, wrong/completed turn refusal, identical pending replay, explicit resolution of two questions, one lost-ack effect and terminated owned processes. tool-pins.json records exact binaries and catalog/config. manifest.json hashes retained files. resources.json records Popen argv/PID/start stat/exit; fresh checks found each latest owned PID absent. Earlier generations' resource manifests are retained.

These processes used sterile child HOME/CODEX_HOME and an owned loopback fake provider, not private-network or Insula qualification. This task did not rerun ticket02 boundaries. The gateway fixture is not production auth/authorization and should not be installed as a product service. No real model completion, user credentials, model account eligibility, production permissions or full controlled-trial readiness was established. Parent owns canonical retention and ticket05/06 implementation.
