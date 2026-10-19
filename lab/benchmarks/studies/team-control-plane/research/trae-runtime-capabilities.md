# Installed TraeCLI runtime capabilities

Read on 2026-10-01. Corenius commit read:
`963f58b31e518a8162dfe719e386ef9d9843321f` (research/trae-runtime).
Installed executable: `/data02/home/philip.yang/.local/bin/traecli`;
`traecli --version`: `0.207.1(internal edition)`.
This is local first-party CLI/schema/source research, not a live integration test.
No model launch, daemon start, authentication read, service mutation, or Git
commit was performed. Only the report and generated scratch schemas were written
in this agent's isolated worktree.

## Evidence and scope

Commands exercised: `--version`, `--help`, `app-server --help`,
`app-server generate-json-schema --help`, `acp --help`, and
`app-server generate-json-schema --experimental --out
.scratch/team-control-plane/trae-schema` (exit 0).
The generated schema bundle SHA-256 is
`22d07defc3fc637f11132ed7aaf15955045e300bbe9e706a829f007e5f2e98d8`.
Schema citations below identify files in that generated bundle, reproducible
with the installed version and command above. Scratch output is not a durable
artifact; the selected contracts are recorded here. Generation included
experimental APIs and does not establish that all endpoints are enabled or
implemented in the running service. No service endpoint was contacted.

Scoped Markdown filename discovery found no Armada/Symphony sibling documents
under the corresponding workspace directories or Corenius reference filenames.
No claims about those systems are made.

## Available contract versus exercised behavior

| Capability | Installed schema/help contract | Corenius source behavior / validation status |
| --- | --- | --- |
| Thread creation | `thread/start`; ThreadStartParams accepts cwd, model, permissions, ephemeral, historyMode and instructions; response contains Thread.id. | ACP implementation sends `initialize`, `session/new`, `session/prompt`. App-server creation not exercised. |
| Recovery / reattachment | `thread/resume` requires threadId. Its description says an already running thread is rejoined; otherwise stored history is loaded, with history > nonempty path > threadId precedence. Prefer threadId. `thread/read`, `thread/list`, `thread/loaded/list`, `thread/turns/list`, `thread/items/list` exist. | ACP starts a fresh child/session for each call; no resume/load call in this adapter. Restart semantics remain untested. |
| Turn start | `turn/start` requires threadId and input; returns a Turn with id and status. Optional userMessageId is described as stable user-message correlation across process restarts. | ACP returns sessionId as provider response id; no app-server turn id is acquired. Schema generation only. |
| Steering | `turn/steer` requires threadId, input, expectedTurnId. Explicit description: request fails if expectedTurnId differs from active turn. Response has turnId. | No steering RPC in Corenius ACP adapter. Runtime timing, delivery order and rejection behavior untested. |
| Cancel | `turn/interrupt` requires threadId and turnId. Background terminals also have separate list/stop/clean APIs. | ACP normal completion kills/waits the child; timeout drops a kill_on_drop child. Neither operation is a proven turn interruption or descendant/effect cancellation guarantee. |
| Observation | Notifications include thread/status/changed, turn/started, turn/completed, item deltas, thread/tokenUsage/updated, thread/goal/updated and cleared. Stored thread/turn/item reads support recovery views. | ACP observes assistant message chunks and optional process snapshots, plus final stopReason/usage. No durable notification cursor/replay contract was established. |
| Runtime incarnation | InitializeResponse reports codexHome (described as TRAE_HOME), platformFamily, platformOs and userAgent. Thread.sessionId denotes a session tree, Thread.id its thread, Turn.id its turn. | No explicit runtime boot/incarnation identity or fencing token in the inspected initialize/thread/turn contracts. PID and elapsed time are observations, not an incarnation guarantee (inference). |
| Goal | `thread/goal/set`, get and clear; ThreadGoal is keyed by threadId and contains objective, status, tokenBudget, tokensUsed, timeUsedSeconds and timestamps. | Isolated ACP profile disables goals feature and get_goal/create_goal/update_goal tools. Normal profile does not add those restrictions. No goal lifecycle call exercised. |

Schema sources: `ClientRequest.json`, `ServerNotification.json`,
`v1/InitializeResponse.json`; `v2/{ThreadStartParams,ThreadStartResponse,
ThreadResumeParams,ThreadResumeResponse,ThreadReadParams,TurnStartParams,
TurnStartResponse,TurnSteerParams,TurnSteerResponse,TurnInterruptParams,
ThreadGoalSetParams,ThreadGoalGetResponse}.json`.
Corenius sources: [ACP adapter](../../../../src/provider/acp.rs), especially
lines 28–105 (isolation), 266–302 (failure classification), 331–457 (lifecycle)
and 516–525 (assistant text projection).

## Unknown start outcomes and idempotency

Neither ThreadStartParams nor TurnStartParams exposes an idempotencyKey or a
caller-selected thread/turn id. JSON-RPC request ids correlate responses; no
inspected schema promises durable duplicate suppression for those ids.
TurnStartParams.userMessageId promises correlation of the user history item,
not idempotent acceptance of a turn. Treating it as a deduplication key would be
an unsupported inference.

**Inference:** losing the response after dispatch leaves an unknown start
outcome. Blindly repeating thread/start can create another thread; repeating
turn/start can submit another input or interact with an existing active turn.
Use stored command intent and reconciliation through read/resume/history before
retrying. The schema does not prove that matching a recovered history item
fully resolves which effects ran or whether acceptance is durable.

There is a distinct experimental workflow/start API with mandatory threadId
and idempotencyKey, and workflow/revise with mandatory threadId, runId, source,
expectedVersion and idempotencyKey. Related list/status/events/nodes/pause/
resume/cancel APIs exist. These are potentially useful but do **not** confer
idempotency on thread/start or turn/start. No deduplication scope, retention,
conflict response, crash consistency, or execution safety was exercised.
Sources: `v2/{WorkflowStartParams,WorkflowStartResponse,WorkflowReviseParams}.json`.

Recovered history is not a complete effect journal: generated
ThreadRollbackResponse's Thread description explicitly notes lossy persisted
ThreadItems and absent interactions such as command executions, consistent
with thread/resume. Thread.turns is populated only by resume/rollback/fork/read
(includeTurns), otherwise empty. Do not mistake an empty start response for
proof of no turns. Source: `v2/ThreadRollbackResponse.json` and Thread definition
in `v2/ThreadStartResponse.json`.

## Runtime, worker and authority boundaries

[Process supervisor](../../../../src/supervisor.rs) snapshots ps PID/PPID,
state, elapsed time and command and traverses descendants. Missing root,
zombie and stopped processes produce health findings. It does not issue
thread/turn control or establish stable identity across PID reuse.

[worker.py](../../../../tools/agents/worker.py) lines 196–317 launch a bounded
traecli exec process through a timeout shell wrapper and write run metadata
including pid/start time. Lines 430–495 observe exit_code/pid liveness, check
outputs/boundaries, and persist verdicts; max_wait returns RUNNING rather than
cancelling. This is a process/artifact supervisor, not a persistent thread
reconciliation controller. Neither launch path explicitly adds --no-daemon;
CLI help describes --no-daemon as avoiding shared-daemon probing/start.
**Inference:** current wrappers must not be assumed to own an exclusively
embedded runtime merely because they own a PID.

[Capsule builder](../../../../tools/insula/capsule.py) lines 371–435 constructs
bubblewrap PID/IPC/UTS isolation, clearenv, read-only layers, workspace and
explicit writable binds; worker --sandbox insula enforces allowed output paths.
The worker's capsule launch explicitly requests host networking. Namespace and
filesystem confinement do not establish logical turn identity, remote service
shutdown, goal ownership or exact-once effects (inference). Capsule run timeout
kills its process group (lines 444–480); live behavior was not re-probed.

[effort.py](../../../../tools/agents/effort.py) records durable pointers and
worker verdicts in STATE.md; dehydrate rejects inappropriate dirty state and
commits its changes. Rehydrate checks pointers and can repair worktrees. Neither
is a live runtime lease or thread recovery API. They were read only, not invoked.
[land.py](../../../../tools/agents/land.py) lines 200–247 gates an exact rebased
tip, pushes first when configured, then fast-forwards mainline. It is a Git
landing boundary, not authorization for a team controller to merge or commit.
This investigation does not change human-owned architecture/merge authority.

[Durable execution note](../../../observability/durable-execution.md) describes
a future event-sourced workflow, stable activity idempotency keys, replay reuse,
and crash/resume tests. Its wording is design direction and immediate slices;
this note is not evidence these guarantees are implemented.

## Goal ownership conclusion

The installed protocol locates a goal on a runtime thread. No inspected goal
schema includes team/effort/ticket identity, owner lease, revision precondition,
controller epoch or delegation authority. Mapping `/goal` to a durable team
objective is therefore a policy/design choice, not an existing API guarantee
(inference). Preserve an external durable effort/ticket as authority; use a
thread goal only as a subordinate runtime execution mechanism if explicitly
chosen. Completion, budget limits and thread shutdown need independent
reconciliation. The schema alone does not prove `/goal` TUI semantics,
automatic continuation, restart persistence or ownership enforcement.

## Bounded future probes (not performed)

1. In a private temporary TRAE_HOME, an isolated worktree and explicit embedded
   stdio app-server, impose a 30-second wall limit and no model requests. Execute
   initialize, thread/start, thread/read and thread/resume; capture effective
   permissions and ids. Repeat using a fresh server process to test stored
   recovery. Confirm initialization never touches the shared daemon first.
2. With an approved synthetic/fake model provider and disabled tools, spend at
   most two turns/60 seconds: start, wrong-id steer, correct-id steer,
   interrupt, then read. Verify acknowledgements against terminal notifications
   and persisted history, including steer arriving near completion.
3. Through a transport fault injector, cut after request dispatch but before
   turn/start response. Reconnect/read by known threadId and userMessageId;
   repeat that userMessageId only under the fake provider. Record whether
   duplicate acceptance occurs. Do the analogous thread/start probe and report
   whether creation can be unambiguously reconciled without a caller key.
4. In that private runtime, set/get/pause/clear a goal without model work, restart
   and get again. Separately approve a one-turn fake continuation test to
   establish whether active goals autonomously create further turns. Assert
   runtime goal state cannot grant repository landing authority.
5. Test experimental workflow/start repeated keys with identical and differing
   payloads, then restart and repeat, under fake effects with at most three
   submissions. Establish scope and durability before choosing it as a safety
   primitive. Never extrapolate its result to turn/start.
6. Capture initialize/read responses across two private server incarnations;
   determine whether any non-schema metadata supplies an epoch. Otherwise make
   controller-generated epochs and stale-connection fencing explicit design
   requirements. Test child and background tool termination independently of
   turn/interrupt using bounded harmless subprocesses inside a capsule.
