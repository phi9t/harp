# Lab 1: TraeCLI exec and ACP

TraeCLI exposes a bounded command interface and a bidirectional ACP service.
The interfaces share a runtime, but they give the controller different
lifetime and identity contracts. The current live exec attempts failed; the
offline ACP handshake succeeded. Neither result proves successful ACP inference.

Source revision: `b7bee20e00c822feec48362a1dbd8d62fe9c3641`.
Use `src/provider/acp.rs`, `tests/provider_acp.rs`, and
`tools/agents/worker.py` as local source anchors. `AcpSessionProfile` defines
isolated feature/tool settings. `run_acp_prompt_inner` sends initialize,
session creation, and prompt RPCs; it kills the service after the prompt
response. This adapter is currently turn-scoped even though ACP can retain
a session in a longer-lived client/service relationship.

## Exercise A: inspect the installed interface

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/native/run_lab.py inventory --out .scratch/engineering-profile-labs/native/trae-inventory.json
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/native/run_lab.py catalog --timeout 20 --out .scratch/engineering-profile-labs/native/trae-catalog.json
```

Check the recorded executable digest and version before interpreting events.
Installed help exposes `exec --json`, `exec resume`, `exec fork`, `--ephemeral`,
and `acp serve`. It also advertises a shared app-server daemon and the global
`--no-daemon` option. A frontend PID is therefore insufficient evidence of
backend service ownership. The replay helper uses `--no-daemon` for live exec;
the two retained historical attempts did not pass that option. Its argument
placement was verified with `traecli --no-daemon exec --help`, without inference.

[Fresh catalog evidence](evidence/20261011-trae-model-catalog.json) records
model names only. It advertises `GPT-6.1-Sol`; a catalog entry is not evidence
that the selected account and route can complete inference.

## Exercise B: an actual offline ACP handshake

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/native/run_lab.py acp --timeout 20 --out .scratch/engineering-profile-labs/native/trae-acp.json
```

The helper starts the installed binary in a private model-free Insula. No
authentication, provider turn, global configuration, plugin store, or host
credential store is supplied. Read [the retained exchange](evidence/20261011-acp.json).

The following is an abridged **observed** exchange. Extra capabilities are
omitted; IDs and error values preserve the native response.

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":1,"clientCapabilities":{},"clientInfo":{"name":"corenius_learning_lab","version":"0.1"}}}
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":1,"agentInfo":{"name":"traecli-acp","title":"TRAE CLI","version":"0.209.1"},"agentCapabilities":{"loadSession":true}}}
{"jsonrpc":"2.0","id":2,"method":"session/new","params":{"cwd":"/workspace","mcpServers":[],"_meta":{"trae":{"skills":[],"mcpServers":[]}}}}
{"jsonrpc":"2.0","id":2,"error":{"code":-32000,"message":"Authentication required"}}
```

Request ID 1 correlates the initialize response; it is not a session ID.
The runtime advertises load/resume/fork capabilities, but this lab creates no
authenticated session. The service remains alive after the rejected request.
The helper then disposes it. That disposal is separate from prompt completion.

Once an independently qualified authenticated ACP session exists, the source
adapter sends the following shape. This is a **source-backed recipe**, not a
live exchange from this lab:

```json
{"jsonrpc":"2.0","id":3,"method":"session/prompt","params":{"sessionId":"<returned-sessionId>","prompt":[{"type":"text","text":"Without tools, answer only: what is 19 + 23?"}]}}
{"jsonrpc":"2.0","method":"session/update","params":{"sessionId":"<returned-sessionId>","update":{"sessionUpdate":"agent_message_chunk","content":{"type":"text","text":"42"}}}}
{"jsonrpc":"2.0","id":3,"result":{"stopReason":"end_turn"}}
```

The response to prompt request ID 3 supplies a stop reason. A text chunk alone
does not prove terminal completion. A production adapter must validate the
session identity on updates and interpret all terminal stop reasons. The
current Rust loop extracts text from `session/update`; do not infer complete
cross-session routing or successful cancellation from that implementation.

## Exercise C: one explicitly admitted live CLI turn

Prerequisite: an authenticated trusted installed TraeCLI. This mode uses the
runtime's authentication without opening credential files in the lab helper.
It is a host read-only exercise, not an authenticated Insula qualification.

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/native/run_lab.py live-cli --runtime traecli --model GPT-6.1-Sol --timeout 35 --authorize-model-spend --out .scratch/engineering-profile-labs/native/trae-live.json
```

The fixed prompt is `19 + 23`, with no tools or explanation. The helper
disables known tools, hooks, plugins, memory, and goals. It ignores user exec
configuration/rules, uses an ephemeral session, and rejects unexpected tool
evidence. The finite deadline does not enforce a provider token or billing cap.

Expected acceptance requires one thread identity followed by one `turn.started`,
a completed `42` answer within that interval, and one `turn.completed`.
Every supplied thread identity must match. Unknown item types, terminal
failures, and nonzero exits cause rejection. An existing output path causes
refusal before launch. The retained attempts predate these final guards;
model-free replay through the repaired reducer still rejects both attempts.

The [initial lowercase-model attempt](evidence/20261011-traecli.json) and the
[catalog-model attempt](evidence/20261011-traecli-catalog-model.json) both
recorded this projected sequence:

```text
thread.started -> item.completed(error) -> item.completed(error)
-> turn.started -> error -> item.completed(error) -> turn.failed
exit_code = 1; accepted = false; no arithmetic answer
```

Both attempts finished before the 35-second timeout. Exact error prose was
excluded before persistence. Fixed diagnostic categories did not identify the
cause of the second failure. Authentication, routing, and model entitlement
remain unresolved for this live CLI configuration. Do not label the failure
as an authentication error merely because the separate credential-free ACP
probe required authentication. No further paid retry was made.

## Continuation and failure questions

The native session stores conversation context. The controller owns the next
admitted Episode, its budget, and its durable outcome. `exec resume` requires
a stored exact session ID; this lab's ephemeral attempts cannot establish
resume. Do not use `--last` in a multi-worker controller because another
session can become the latest session.

ACP cancellation uses a session-directed protocol operation rather than
killing the frontend alone. Its active-turn behavior was not exercised here.
A deadline, EOF, or process exit can leave prompt outcome unknown. They do
not justify automatic re-prompting. The current adapter's timeout/stall
classification and retained partial text are useful diagnostics, not effect
reconciliation or exactly-once execution.

Answer these before adoption: Who retains the session ID? Who denies a stale
caller? What proves a stopped tool process? What converts an unknown prompt
result into a safe retry? These remain controller qualification work.
