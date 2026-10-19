# Lab 2: Codex CLI as a bounded Episode executor

`codex exec --json` exposes a native event stream for one command execution.
The observed arithmetic turn returned `42`, emitted `turn.completed`, and
exited with code 0. These are three separate facts; the lab requires all three.

Source revision: `b7bee20e00c822feec48362a1dbd8d62fe9c3641`.
Installed version: 0.160.1. The source anchors are
`tools/agents/worker.py::event_session_id`, `event_usage`,
`RuntimeEventAdapter`, and `tools/agents/test_worker.py`.
The worker adapter normalizes native events and tracks started/completed tool
items without reporting the same command twice.

`src/provider/codex.rs` is a different integration: it sends Responses requests
directly and parses SSE. It is not an implementation of `codex exec` or the
app-server protocol. Keep evidence from these interfaces distinct.

## Exercise A: inspect options without model spend

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/native/run_lab.py inventory --out .scratch/engineering-profile-labs/native/codex-inventory.json
```

Inspect the actual help for `exec`, `exec resume`, and app-server. The installed
CLI exposes `--json`, `--ephemeral`, `--ignore-user-config`, `--ignore-rules`,
read-only sandbox selection, and last-message/schema outputs. Those are
installed interface observations. For the general automation contract, see
[official non-interactive documentation](https://learn.chatgpt.com/docs/non-interactive-mode).
The native event stream and process status should both be inspected.

## Exercise B: one actual provider turn

Prerequisite: the trusted installed Codex CLI already has usable authentication.
The helper never reads or copies its authentication material. This explicit
learning attempt runs on the host with a disposable working directory.

```bash
python3 tools/temp_root.py -- python3 docs/efforts/engineering-profile-labs/native/run_lab.py live-cli --runtime codex --model gpt-6.1-sol --timeout 35 --authorize-model-spend --out .scratch/engineering-profile-labs/native/codex-cli-live.json
```

The helper requests one arithmetic response, disables known context/tool
features, ignores user exec configuration/rules, and uses read-only/ephemeral
execution. A deadline stops the owned process group. Provider-side billing and
ordinary-turn token caps are not established by this command.

[Fresh provider evidence](evidence/20261011-codex-cli.json) contains this
abridged observed exchange:

```json
{"type":"thread.started","thread_id":"01a12c6a-a3fb-76e0-91e0-9a5089d86d8f"}
{"type":"turn.started"}
{"type":"item.completed","item_type":"agent_message","answer":"42","arithmetic_answer_matches":true}
{"type":"turn.completed","usage":{"input_tokens":13822,"cached_input_tokens":0,"output_tokens":5}}
```

These rows are a persisted allowlist projection, not the complete raw native
stream. An earlier `item.completed` of type `error` was also observed. It did
not prevent the successful terminal turn. The exact warning was not retained.
Do not treat every diagnostic item as a terminal failure. Conversely, a
native `error` event or `turn.failed` does invalidate this lab's acceptance.

The thread ID identifies the native conversation. The CLI event interval
defines this observed turn; these projected CLI rows do not expose a separate
native turn ID. The prompt, binary digest, model selection, process outcome,
and native usage are therefore retained beside the events.

The CLI exited 0 after structured completion. The model output used five
tokens, but harness input used 13,822 tokens. An arithmetic prompt does not
make total token usage equal to the visible prompt length.

## Exercise C: completion loss and identity ambiguity

```bash
python3 tools/temp_root.py -- python3 -m unittest discover -s docs/efforts/engineering-profile-labs/native -p 'test_*.py' -v
```

These are explicitly **fake conformance fixtures**. They verify the lab's
acceptance seam, not the native runtime. The separate retained-evidence test
also reduces three actual historical CLI artifacts without launching a native
runtime. The final suite contains 21 tests. Remove `turn.completed` while keeping
exit code 0: acceptance fails. Change the answer item to `item.started`:
acceptance fails. Remove the thread ID or add a different thread ID:
acceptance fails. Move the completed answer before `turn.started` or after
`turn.completed`: acceptance fails. Unknown item categories and native tool
items cause rejection through the same shared allowlist used by tool flags.

The historical Codex CLI attempt predates the final reducer and temporary-root
guards. Its retained rows pass the stricter ordered reducer without a provider
replay. Both retained TraeCLI attempts still fail. No historical file was changed.

This makes the contract reviewable without using another paid request. A
completed artifact and process exit cannot replace a missing turn outcome.

## Resume, interruption, and controller ownership

`codex exec resume <SESSION_ID> <PROMPT>` is an installed CLI interface for
continuing a saved conversation. It creates another process/turn attempt.
Because this lab uses `--ephemeral`, it deliberately does not qualify that
saved-session path. The exact resume option set is recorded in the inventory;
do not assume every `exec` flag exists on `exec resume`.

Use an explicitly recorded session ID. `--last` is unsuitable for concurrent
worker ownership. A PID identifies a process lifetime, not a conversation,
ExecutionScope, or authority grant. A controller replacement must reconstruct
its owned jobs from its ledger rather than infer them from native session order.

SIGTERM/SIGKILL cleanup proves disposal of the helper's owned process group.
It does not prove provider cancellation, reconciliation of external effects,
or termination of a pre-existing shared service. Interruption/resume during
active native tool execution remains unqualified.

When a timeout or pipe failure removes the terminal event, preserve an unknown
outcome. A Temporal Activity retry that launches another CLI process can repeat
work. PostgreSQL can record admission and ambiguity, but it cannot infer
native completion from a process exit. The controller must join exact native
evidence before admitting continuation.
