# Lab 3: Codex app-server thread, turn, and process lifetimes

An app-server process can complete a model turn and remain alive. The observed
provider turn returned `42` with a separate thread ID and turn ID. A model-free
probe also resumed the same empty stored thread after replacing the process.
Stored-thread persistence does not prove active-turn recovery.

Source revision: `b7bee20e00c822feec48362a1dbd8d62fe9c3641`.
Read `controller/src/native_ws.rs`, `codex.rs`, `native_ui.rs`,
and `support/codex_insula.py`. The WebSocket adapter already observes
turn notifications, handles early notifications, and binds mutations to native
identities. The Insula launcher fences connections with process start identity.
Its current protocol capability pin is 0.159.3; these probes use installed
0.160.1. Do not substitute the new binary for the pin based on schema similarity.

The [official app-server documentation](https://learn.chatgpt.com/docs/app-server)
describes initialization, thread creation/resume, turn admission, streaming,
and terminal notifications. In this lab, local code and installed generated
schemas are the executable anchors; live observations qualify only the cases
listed below.

## Exercise A: generate the actual installed protocol schemas

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/native/run_lab.py schema --timeout 20 --out .scratch/engineering-profile-labs/native/app-schema.json
```

The helper runs `app-server generate-json-schema --experimental` in an offline
private Insula. It records schema counts, selected field sets, and digests,
then removes the generated trees. [Fresh schema evidence](evidence/20261011-schema.json)
contains 440 files and these selected required fields:

| Contract | Required fields | Interpretation |
| --- | --- | --- |
| `ThreadStartParams` | None | Optional/default fields still affect execution policy |
| `ThreadReadParams` | `threadId` | Address an existing conversation |
| `ThreadResumeParams` | `threadId` | Load stored context into a runtime |
| `TurnStartParams` | `threadId`, `input` | Admit model work in that conversation |
| `TurnInterruptParams` | `threadId`, `turnId` | Target a specific active turn |
| `TurnCompletedNotification` | `threadId`, `turn` | Check terminal identity and `turn.status` |

Schema generation is model-free. The presence of `thread/turns/list` or an
`excludeTurns` field does not guarantee the installed backend implements it.

## Exercise B: actual offline lifecycle RPCs

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/native/run_lab.py app-server --timeout 20 --out .scratch/engineering-profile-labs/native/app-lifecycle.json
```

This probe mounts no authentication or user configuration. It creates a
private persistent empty thread, inspects it, requests idle interruption,
replaces the runtime process, and resumes the exact stored thread ID.
It never sends `turn/start`.

Read [the full paginated probe](evidence/20261011-app-server-paginated.json).
The following observed exchanges are abridged:

```json
{"id":1,"method":"initialize","params":{"clientInfo":{"name":"corenius_learning_lab","version":"0.1"},"capabilities":{"experimentalApi":true}}}
{"id":1,"result":{"userAgent":"corenius_learning_lab/0.160.1 (...)"}}
{"method":"initialized"}
{"id":3,"method":"thread/start","params":{"cwd":"/workspace","approvalPolicy":"never","sandbox":"read-only"}}
{"id":3,"result":{"thread":{"id":"01a12c6b-2c6b-7e63-a104-65f9e62303bf","status":{"type":"idle"},"historyMode":"paginated","turns":[]}}}
{"id":4,"method":"thread/read","params":{"threadId":"01a12c6b-2c6b-7e63-a104-65f9e62303bf","includeTurns":true}}
{"id":4,"error":{"code":-32601,"message":"list_turns is not supported yet"}}
```

The response confirms admission of a thread, not model execution. RPC request
IDs correlate responses within a connection. They restart in the new process;
they are not durable thread IDs or turn IDs.

Metadata reads with `includeTurns:false` succeeded. Full reads, same-process
resume, resume with `excludeTurns:true`, and `thread/turns/list` failed with
`list_turns is not supported yet`. A paginated recipe from current documentation
therefore did not repair this installed behavior. The runtime remained alive.

An idle `turn/interrupt` using `lab-no-active-turn` returned `-32600` and
`no active turn to interrupt`. This proves rejection of idle interruption;
it does not qualify cancellation of a running turn.

After the owned first process was disposed, the helper initialized a new
process with the same private store. It resumed the exact thread ID with
`excludeTurns:true`, `approvalPolicy:never`, and `sandbox:read-only`.
That post-restart resume succeeded. In the earlier probe, omitting resume
policy overrides changed `approvalPolicy` to `on-request`.
Reassert and inspect policy on resume; stored identity alone is insufficient.

## Exercise C: one actual app-server provider turn

Prerequisite: existing authentication for the trusted installed Codex runtime.
The helper does not open or copy credential files. This exercise uses a host
stdio server, read-only policy, disabled known tools/features, and an ephemeral
thread. It inherits ambient app-server configuration and may receive startup
notifications. It is not an authenticated hermetic Insula proof.

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/native/run_lab.py live-app-server --model gpt-6.1-sol --timeout 35 --authorize-model-spend --out .scratch/engineering-profile-labs/native/app-provider-live.json
```

The helper completes `initialize`, sends `initialized`, creates a thread,
then sends one `turn/start`. It denies runtime-requested client capabilities.
It matches terminal notifications to both returned native identities. It does
not stop when the RPC admission response arrives. The deadline is finite;
hard token/billing caps remain unestablished.

[Fresh provider evidence](evidence/20261011-codex-app-server-live.json) records:

```text
thread/start response:
  thread_id = 01a12c6d-3024-7aa0-98d9-368a12945dc8
turn/start request input:
  [{"type":"text","text":"Without tools, files, commands, or explanation, answer only: what is 19 + 23?","text_elements":[]}]
turn/start response:
  turn.id = 01a12c6d-308c-7801-936e-066d36c9768d
  turn.status = inProgress
turn/started: same thread and turn
item/agentMessage/delta: same thread and turn; delta matches 42
item/completed: same thread and turn; completed answer matches 42
turn/completed: same thread and turn; turn.status = completed
alive_after_turn_completed = true
```

Startup, warning, account, and token-usage notification **method names** are
retained. Their broad payloads are excluded before persistence. The provider
transcript is an explicit projection, not raw protocol capture. Account/rate
limit events are not credentials, model completion, or controller permission.

The retained provider projection predates the replay runner's item-type field
and stricter rejection of unexpected tool items. It proves the joined arithmetic
completion; it does not independently prove exhaustive tool absence. The replay
runner records item types and rejects items outside its allowed non-tool types.
The current shared allowlist rejects unknown or omitted types on item
notifications. Applying it to this older projection therefore fails tool
qualification; the old `accepted:true` remains the historical arithmetic gate
result. No extra paid turn was used to qualify the final guard or fill that gap.

The server was alive after `turn/completed`. The helper then disposed the owned
process group; its exit code after disposal was 0. Completion was already
established by the native turn outcome. A later process exit cannot change
that observation into active-turn recovery evidence.

## Admission, continuation, and recovery limits

`turn/start` acknowledgment returns an admitted turn, often `inProgress`.
`turn/completed` names a terminal turn; its status can indicate success,
failure, or interruption. Match the exact thread/turn pair and inspect status.
Text deltas, an alive server, and a successful JSON-RPC response are insufficient.

Notifications can precede the admission response. Preserve and route them
instead of discarding unmatched messages. The existing `native_ws.rs` handles
this case; the lab buffers events until it knows the returned turn ID.

After a successful turn, the controller decides whether to admit another
Episode. A saved conversation does not grant continuing tool access, budget,
or authority to a replaced controller. Keep `RuntimeIncarnation`, thread
identity, ExecutionScope ownership, and controller grant epochs separate.

Before production use, qualify active `turn/interrupt`, transport disconnect
during a tool call, lost admission acknowledgment, stale-controller requests,
and runtime restart during a running turn. Record unknown outcomes instead of
automatically starting replacement work. This lab deliberately makes no
exactly-once, active recovery, or production readiness claim.
