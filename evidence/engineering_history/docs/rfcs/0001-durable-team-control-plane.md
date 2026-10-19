# RFC 0001: Durable team control plane above Trae

Status: captured proposal; implementation design not yet approved.
Captured: 2026-10-01 from the owner's supplied message.
Owner effort: [Durable team control plane](../efforts/team-control-plane/EFFORT.md).
Wayfinding: [Decision map](../efforts/team-control-plane/map.md).
Implementation analysis: [Feasibility and staged implementation](0001-durable-team-control-plane-implementation.md).

The proposal body below is preserved exactly from the source message, including
diagrams, examples and unresolved source markers. Only the skill invocation
prefix and the final capture/investigation instruction are excluded from the
body. This metadata is editorial; the body states a proposal, not verified
runtime guarantees. The `:chatgpt-content-reference{...}` markers arrived
without URLs and remain unresolved citations. Research and corrections live in
companion documents rather than silently changing this source.

Original user-message UTF-8 SHA-256: `0b4275fb2088aa114a63d3f189ccf632f2f710a28d055f7989a44487af143f22`.
Preserved body UTF-8 SHA-256: `88c9f767012dfe032e586cc40232317b364d9b209ba7f94841f62169fa66955b`.

<!-- preserved-owner-proposal:start -->
**I would build a durable team control plane above Trae—not make the lead agent itself the orchestration server.**

The division should be:

> **The lead decides what work should happen. The broker records and enforces those decisions. Trae executes agent turns. The execution environment holds the actual work. The TUI observes and lets you intervene.**

The most important consequence is that **the lead must be replaceable, just like a worker**. Losing its conversation, its app-server, or its terminal must not lose the team’s assignments, decisions, pending questions, or recovery instructions.

For your intended long-horizon system, my preferred architecture is **an external broker with PostgreSQL, Temporal-backed workflows, runtime adapters, supervised execution environments, and versioned artifacts**. This fits the separation we have been discussing around Symphony/Temporal and Armada/Insula.

I’m taking your description as the observed Trae operating model. I have not inspected your particular build, so its exact detach, cancellation, and recovery guarantees remain things to verify—not assume from another app-server.

## 1. The architecture: separate the team from the agent runtime

I would evolve your diagram into this:

```text
                    Human / tmux TUI / Emacs / Web UI
                                  │
                         commands and observations
                                  │
                                  ▼
                    ┌────────────────────────────┐
                    │      Team control plane    │
                    │                            │
                    │ Goals and task graph       │
                    │ Assignments and permissions│
                    │ Mailboxes and decisions    │
                    │ Budgets and acceptance     │
                    │ Recovery and reconciliation│
                    └─────────────┬──────────────┘
                                  │
                    ┌─────────────┴──────────────┐
                    │                            │
             PostgreSQL + artifacts      Temporal workflows
                    │                            │
                    └─────────────┬──────────────┘
                                  │
                         Runtime adapters
                                  │
                    ┌─────────────┴──────────────┐
                    ▼                            ▼
             Trae app-server A            Trae app-server B
              ├─ Lead thread               ├─ Worker thread
              ├─ Native thread             └─ Verifier thread
              └─ Runner thread
                    │                            │
                    ▼                            ▼
             Supervised execution environments / Insulas
             Workspaces, processes, jobs, snapshots, artifacts
```

The lead and workers also call back into the team control plane through tools. For example:

```text
Lead model
  → team.propose_tasks(...)
  → broker validates and records the proposal
  → workflow dispatches a worker
  → Trae executes that worker
  → worker calls team.submit_result(...)
  → broker records the submission and schedules verification
```

**The major change from your current arrangement is that the lead should stop being the unrestricted primary controller of everyone’s app-server sessions.** It should use team-level operations; the broker’s adapter should own the corresponding runtime mutations.

That gives you a boundary between:

- “Assign Native to implement this interface.”
- “Start turn 812 on thread 417 through server incarnation 29.”

The first is an engineering decision. The second is execution machinery.

I would not initially fork Trae to embed all this inside its app-server. That would couple team semantics to its runtime internals. Nor would I build a free-form agent chat mesh and hope coordination emerges. An external control plane keeps runtime replacement and failure handling explicit.

Symphony offers a useful reference for this separation: its specification centralizes dispatch and reconciliation while treating the coding-agent runner as a separate component. But its baseline restart recovery is tracker/filesystem-driven, without restoring exact scheduler state from a durable orchestrator database. Your requirements go beyond that baseline. :chatgpt-content-reference{index="0"}

## 2. Do not equate a worker, a task, and a thread

Your current description says:

> Each worker is a saved thread with its own assignment, goal, and worktree.

That is a useful initial implementation, but I would not preserve that equivalence in the broker’s data model.

Instead:

| Object | Meaning | Lifetime |
|---|---|---|
| **Mission** | The user’s objective, constraints, acceptance criteria, and budget | Potentially days or weeks |
| **Task** | A specific outcome needed for that mission | Until accepted, cancelled, or superseded |
| **Attempt** | One execution attempt against a particular task specification | Until it succeeds, fails, or is abandoned |
| **Worker identity** | A logical team member or role with permissions and a mailbox | Can outlive particular tasks and threads |
| **Runtime binding** | The current mapping to server, thread, environment, and execution | Changes during recovery or relocation |
| **Turn** | One bounded episode of model/tool execution | Usually much shorter than a mission |
| **Artifact** | A versioned output or piece of evidence | Retained independently of the conversation |

For example:

```text
Task: implement reconnect-safe runner lifecycle
    Attempt 1
        worker: runner
        runtime: server-A / thread-123
        workspace: attempt-1
        outcome: interrupted, effects require reconciliation

    Attempt 2
        worker: runner
        runtime: server-B / thread-456
        workspace: attempt-2
        input: reconciled checkpoint from attempt-1
```

You may reuse `thread-123` when that is safe and helpful. But the task must remain intelligible even when you cannot.

**A thread is a context-continuity mechanism, not the identity of the work.**

Likewise, a worktree belongs to a task attempt or a deliberately continued task—not permanently to an agent persona. “Runner” should not accumulate every assignment it has ever received in one giant conversation and one increasingly contaminated directory.

For runtime references, store more than a bare thread ID: include the runtime kind, deployment or storage identity, environment identity, and current server incarnation. A saved thread ID alone is not a migration protocol.

## 3. Build a small hierarchy, but allow scoped peer communication

For your current setup, I would begin with **one lead, a small number of implementing workers, and an independent verifier**. Integration should be a separately authorized capability, although it does not require another permanently running LLM.

### The lead owns engineering judgment

The lead decomposes the mission, defines interfaces, chooses parallelizable work, resolves conflicting assumptions, evaluates findings, and proposes plan changes.

It should not personally maintain retry timers, remember which processes are alive, or poll terminals to discover whether someone finished.

The lead should usually run **in response to meaningful events**, not continuously ask every worker for status.

### Workers own bounded outcomes

Give workers outcome-oriented assignments rather than indefinite responsibilities.

“Implement the runner’s reconnect behavior against contract v3” is better than “You are the runner engineer; keep improving the runner.”

An illustrative task contract could look like:

```yaml
task_id: runner-reconnect
spec_revision: 3
objective: Recover a worker session without duplicating active execution.

inputs:
  interface_contract: artifact:runtime-contract-v3
  repository_base: commit:BASE_SHA

dependencies:
  - runtime-contract-accepted

write_scope:
  - src/runner/
  - tests/runner/

acceptance:
  - Reattaching a view does not create another execution.
  - Lost start acknowledgements enter reconciliation.
  - Existing active executions are adopted rather than relaunched.

deliverables:
  - candidate_commit
  - test_report
  - unresolved_risks

budget:
  max_execution_episodes: 8
  max_repair_attempts: 2
```

The broker validates this structure and enforces the parts it can enforce. The worker chooses implementation tactics.

### The verifier evaluates a particular candidate

The verifier receives the specification, candidate artifact, and required checks. It does not merely receive the implementer’s claim that the work is finished.

For code, verification should refer to an exact commit or workspace snapshot, a particular test-suite revision, and a recorded environment.

### Peer communication should be possible without becoming peer authority

Native should be able to ask Runner a technical question directly. Routing every question through the lead creates unnecessary serialization.

But distinguish:

> “What error code does your adapter return?”

from:

> “I have decided we should change the interface and skip the compatibility tests.”

The first can be ordinary worker-to-worker communication. The second needs an authorized, versioned decision.

**Use a hierarchy for authority, not necessarily for every message.**

## 4. The broker needs commands, events, and mailboxes—not just chat

This is the heart of the exchange mechanism.

I would expose three distinct concepts.

### Commands request authorized state changes

Examples include `task.assign`, `task.pause`, `plan.revise`, `attempt.cancel`, and `result.submit`.

A command has an authenticated principal, an idempotency key, and relevant preconditions such as the expected task revision.

A worker saying “I am now working on task B” must not itself transfer ownership of task B.

### Events record what happened

Examples include `AttemptStarted`, `QuestionRaised`, `CandidateSubmitted`, `VerificationFailed`, and `RuntimeDisconnected`.

Events are useful for audit, materialized views, workflow wakeups, and recovery.

Do not infer these events by searching terminal text for phrases such as “done” or “reconnecting.”

### Mailboxes carry requests and information

A mailbox message should have enough structure to survive retries and changing conversations:

```json
{
  "message_id": "msg-123",
  "mission_id": "mission-7",
  "task_id": "runner-reconnect",
  "task_revision": 3,
  "sender": "native",
  "recipient": "runner",
  "kind": "question",
  "reply_to": null,
  "requires_response": true,
  "attention": "next_safe_point",
  "body": "Should reconnect preserve the original execution ID?",
  "artifact_refs": ["artifact:runtime-contract-v3"]
}
```

The server should derive or validate the sender from authentication. It must not trust a caller-provided `"sender": "lead"` field.

Crucially, track separate message states:

```text
persisted → presented to an execution → explicitly acknowledged → resolved
```

These do not mean the same thing. A successful WebSocket write does not prove that the agent saw the message, understood it, or performed the requested work.

### How messages reach a model

For an idle worker, the controller can start a bounded turn containing a batch of relevant inbox items.

For an active worker, the adapter may steer the current turn when the runtime supports it, or wait until the next safe boundary. Urgent cancellation should use the runtime’s cancellation machinery—not merely inject “please stop.”

Codex’s documented `turn/steer`, for comparison, targets an active turn using `expectedTurnId` and fails when no matching active turn exists. That is the kind of concurrency-sensitive contract your adapter needs to preserve. :chatgpt-content-reference{index="1"}

If the turn finishes between inspection and delivery, the message should return to pending delivery. It should not disappear.

For a blocked worker, provide a **yield mechanism**: record the dependency or question, end the current episode, and wake the worker when the answer arrives. Do not spend model calls repeatedly asking whether something changed.

### How to expose this

I would expose a small team tool surface, such as:

```text
team.get_context
team.list_tasks
team.propose_tasks
team.send_message
team.read_inbox
team.record_decision
team.submit_result
team.request_approval
team.yield
```

MCP is a reasonable agent-facing wrapper because it provides tools and resources while leaving orchestration with the host. It should not become your durable scheduling engine. :chatgpt-content-reference{index="2"}

A2A can be useful later for collaboration with externally operated agents; its task/message/artifact model is relevant. I would put it at the interoperability boundary rather than force every internal runtime operation through it. Its documentation also leaves artifact-version acceptance and linkage with the client, reinforcing the need for your own domain state. :chatgpt-content-reference{index="3"}

## 5. What the central server should persist—and where Temporal fits

I would make PostgreSQL authoritative for **team-domain state**:

```text
missions, task specifications, dependency edges
attempts, ownership generations, runtime bindings
commands, messages, decisions, approvals
artifact manifests, verification reports
budget reservations and usage
operation journal, events, transactional outbox
```

Store large artifacts, transcripts, logs, and snapshots separately, with references and integrity metadata in the database.

### Make every consequential operation durable before dispatch

For example, assigning a worker should follow this shape:

```text
Database transaction:
    validate task revision and authorization
    reserve ownership and budget
    create attempt
    append domain event
    create outbox entry
Commit

Dispatcher:
    read outbox entry
    request execution using stable operation ID
    reconcile acknowledgement and observed state
```

This avoids the particularly dangerous pattern:

```text
launch work
then try to remember that you launched it
```

The outbox lets a dispatcher retry notification after a crash. The receiver still needs deduplication or reconciliation.

For a small team, PostgreSQL plus a transactional outbox is enough for the initial messaging infrastructure. I would not add Kafka merely because agents exchange messages. `LISTEN/NOTIFY` can provide wakeup hints, but the durable records should live in tables. PostgreSQL documents notifications as a signal to listeners and explicitly describes using table records for the associated data. :chatgpt-content-reference{index="4"}

### Use Temporal for durable waiting and lifecycle execution

My suggested workflow arrangement is:

```text
Mission workflow
    waits for relevant domain events
    requests lead planning/review episodes
    coordinates task workflows
    handles mission-level deadlines and escalation

Task workflow
    requests an admitted attempt
    observes execution
    waits for answers or approvals
    requests verification
    applies retry/repair policy
    finishes when the task is accepted or terminated
```

Temporal’s event history supports reconstruction of workflow state after worker failures. That addresses durable control flow, not automatic recovery of arbitrary external shell effects. :chatgpt-content-reference{index="5"}

**Keep workflow code deterministic.** Model calls, app-server RPCs, database access, Git operations, and environment management belong in Activities or services called through Activities. Temporal explicitly places nondeterministic external interactions outside workflow replay. :chatgpt-content-reference{index="6"}

### Avoid two competing sources of truth

Using both Temporal and PostgreSQL needs a clear ownership rule:

> PostgreSQL owns accepted team state. Temporal owns durable execution of waits, timers, and workflow steps. Domain transitions are committed through idempotent broker transactions.

Do not independently maintain an authoritative `task.status` in both places.

For example, a workflow’s “accept result” Activity should call a broker transaction with a stable transition ID and expected revision. A retry receives the already-recorded result rather than accepting the task again.

Ingress follows the same principle: persist the command and outbox record, deliver a wakeup to the workflow, then have the workflow process the durable command. “Notification delivered” and “command applied” should remain distinct states.

### Keep high-volume traces out of workflow history

Temporal should receive milestones and artifact references—not every token delta, terminal byte, or file read.

Use checkpoints and Continue-As-New for sufficiently long-lived workflows. Temporal supports starting a fresh history while retaining the Workflow ID and passing forward relevant state. That is separate from compacting an LLM conversation. :chatgpt-content-reference{index="7"}

## 6. Treat app-server execution as an external system with uncertain outcomes

The runtime adapter is not just an RPC client. It is where your durable task model meets a runtime that may have weaker guarantees.

I would require an adapter contract covering:

```text
inspect runtime and capabilities
create or recover a thread
start an execution episode
observe current and persisted execution state
deliver or steer input
request cancellation
reconcile interrupted operations
```

These are proposed adapter operations, not claims about existing Trae method names.

### The lost-acknowledgement problem

Suppose this happens:

```text
Broker sends “start turn”
Trae starts the turn
Connection drops before the broker receives the reply
```

The broker cannot safely conclude either “nothing happened” or “the turn is definitely running.”

It should record something like:

```text
operation: start-episode-42
status: outcome_unknown
recovery_action: reconcile_before_retry
```

Then it should inspect the thread, execution records, environment, and any correlation metadata.

**A JSON-RPC request ID is not automatically an idempotency key.** An adapter adding `operation_id` to its own database does not make an underlying nondeduplicating start operation safe to repeat.

If the runtime provides neither durable deduplication nor a reliable way to identify the resulting execution, acknowledge that limitation. Ambiguous operations may need quarantine or operator resolution rather than an automatic restart.

Temporal Activities also need this care: retries can repeat external actions, and Temporal recommends idempotent Activities to avoid duplicate effects. Heartbeat details can help resume work, but they do not make every action transactional. :chatgpt-content-reference{index="8"}

### Exactly one owner of turn continuation

Your native `/goal` behavior creates another possible race:

```text
Runtime goal loop sees an idle thread → starts another turn
Broker sees an idle thread           → starts another turn
```

I would initially let the broker own explicit episode continuation.

Later, native goal continuation can be delegated for an attempt, with a bounded budget and a clear rule that the broker must not simultaneously schedule turns on that thread.

The outer broker still owns task acceptance, ownership, cancellation, and cumulative budget. A native goal reporting completion is evidence to inspect—not permission to declare the mission finished.

### Pin and test runtime capabilities

Record whether a particular runtime supports steering, history replay, durable goals, cancellation, and operation correlation.

Codex’s current documentation illustrates why this matters: it exposes version-specific schemas, labels its WebSocket transport experimental, and documents thread unloading after the last subscriber’s inactivity grace period. An attached UI and a durable executor are not interchangeable concepts. :chatgpt-content-reference{index="9"}

For Trae, test the actual behavior. Keep a broker-owned connection or subscription where required, rather than making execution depend on a human’s terminal staying attached.

## 7. Recovery requires ownership fencing and effect reconciliation

This is where a useful demo becomes a dependable system.

### Lease ownership, but do not confuse leases with process termination

Each admitted attempt should receive an ownership generation:

```text
task: runner-reconnect
attempt: attempt-4
ownership_generation: 12
```

The broker rejects state-changing requests and result submissions from superseded generations.

However, rejecting an old worker’s messages does **not** stop its shell process from editing files.

To handle that, I would combine:

**Separate attempt workspaces.** A replacement attempt should not write into the same directory while an old executor might still be alive.

**Restricted shared effects.** Landing changes, publishing artifacts as accepted, provisioning expensive resources, and other shared mutations should go through services that check current authority.

**Explicit environment reconciliation.** Before reusing an old workspace, establish whether its processes and jobs have stopped, survived, or become unreachable.

This is also why Git worktrees are useful collaboration isolation but not a security boundary: Git documents that linked worktrees share repository data except for particular per-worktree files. Use stronger environment or repository isolation when workers must not affect each other’s state. :chatgpt-content-reference{index="10"}

### Recover each failure according to what actually failed

| Failure | Intended recovery behavior |
|---|---|
| TUI closes | Execution continues under broker ownership; the view can reattach |
| Broker restarts | Reload durable commands and attempts; reconcile before new dispatch |
| Lead thread is lost | Start a replacement lead with the accepted plan, decisions, outstanding questions, and artifact references |
| Trae server dies | Restore the runtime, inspect saved threads and surviving environment effects, then decide whether to resume |
| Start reply is lost | Look up the existing operation; do not blindly launch another |
| Long-running job outlives its agent | Recover the job by its execution handle and inspect it |
| Old worker reports after reassignment | Retain the report for audit, but reject it as an authoritative completion |
| Verification targets an obsolete candidate | Require verification of the candidate that will actually be integrated |

For your earlier server replacement incident, the desired recovery is therefore not just “fix the endpoint.”

It is:

```text
discover replacement server and its incarnation
recover the thread-store binding
reconcile every affected attempt and external execution
restore observation and message delivery
resume only the attempts that are safe to resume
reattach views to the current binding
```

A stable gateway or connection registry helps eliminate stale endpoint handling, but it must route to the correct runtime and thread storage—not arbitrarily load-balance a saved thread across unrelated servers.

Human intervention should use the same command path. Where a native TUI cannot respect broker control ownership, attaching it for mutation should be an explicit operator takeover, not an invisible second scheduler.

## 8. Long-horizon work needs durable working state, not an immortal conversation

I would require a checkpoint at significant milestones and before deliberate context replacement.

A useful checkpoint contains:

```text
accepted task specification and revision
current candidate commit or workspace snapshot
verified facts, with evidence references
hypotheses that remain unverified
decisions and their rationale
pending questions and dependencies
active external execution handles
operations whose effects remain uncertain
next intended action
remaining budget and permissions
```

A prose summary alone is insufficient. It can omit a side effect or silently promote a guess into a fact.

Keep canonical structured state and immutable evidence references alongside the human-readable summary. Preserve raw traces for investigation, but do not inject all of them into every worker.

### Make the lead context reconstructible

A replacement lead should receive a compact mission view:

> What are we trying to accomplish? What is accepted? What is currently running? What needs a decision? What evidence exists? What constraints cannot be changed?

That view should be generated from the broker’s records.

The lead’s old conversation may be useful supporting material. It should not be the only place where the plan exists.

### Distinguish liveness from progress

I would expose separate timestamps for runtime heartbeat, last model activity, last tool activity, and last meaningful task evidence.

A worker producing tokens is not necessarily making progress. A silent worker may be waiting on a legitimate compilation or experiment.

Use task-specific progress policies: repeated identical failures, repeated abandoned plans, excessive repair attempts, or spending without new evidence should trigger review—not an unconditional “continue.”

Budgets should be cumulative across replacement threads and attempts. Reserve budget before dispatch; reconcile usage afterward. Unknown usage should remain reserved rather than being silently refunded after a failure.

### Make external work independently addressable

This is where your Armada model fits naturally.

An agent starting a training run or a service should obtain an **Execution** handle, not just leave an unmanaged shell command running.

The broker should persist the concrete **Target**, **Insula**, and **Execution** identities. A **Recipe** expresses requirements, while placement uses capabilities observed on the concrete target.

Then agent recovery becomes:

> “Execution X is still running in Insula Y; inspect its logs and continue monitoring.”

—not:

> “The conversation restarted, so launch training again.”

Agent-thread recovery and workload recovery should be different operations.

## 9. Completion should be a controlled acceptance process

I would use a task lifecycle resembling:

```text
READY → RUNNING → SUBMITTED → VERIFYING → ACCEPTED
            │                       │
            └→ BLOCKED              └→ REPAIR_REQUIRED
```

Keep cancellation, approval waits, and uncertain execution outcomes explicit rather than forcing everything into “running” or “failed.”

The important transition is **SUBMITTED → ACCEPTED**.

An implementer submits a candidate. A verifier produces evidence. The lead evaluates unresolved engineering questions. The broker applies the acceptance policy.

For code integration, bind the evidence to the candidate and integration base:

```text
task specification revision
candidate commit
integration base commit
test-suite revision
environment identity
test commands, results, and logs
review decision
```

If the integration base changes, construct and verify the new integration candidate as required. Do not assume two independently passing branches necessarily pass together.

I would give only the integration service authority to update the protected landing branch. Workers may propose changes; they should not bypass verification by pushing directly.

Similarly, approval records should bind to the operation and arguments being approved. A changed command or changed artifact should not inherit an unrelated earlier approval.

## 10. What I would build first

The first deployment can remain small: **one broker service, PostgreSQL, Temporal, an artifact directory or object store, one supervised runtime gateway, and your existing lead/Native/Runner/Verifier threads.** Logical separation does not require a fleet of microservices.

First, import the existing thread and workspace bindings into durable records. Route new assignments, steering, submissions, and cancellations through the broker. Give every attempt an operation journal and ownership generation.

Next, add durable inboxes and event-triggered lead episodes. Make a worker able to block, yield, receive an answer, and resume without a human copying text between windows.

Then prove recovery before adding more agents or machines. The acceptance test should include closing every TUI, killing the lead, restarting the broker, killing the app-server during a tool operation, dropping a start acknowledgement, duplicating message delivery, and changing the integration base before landing.

For each test, the system must either recover safely **or report a precise unresolved state and stop the unsafe action**. “It eventually printed success” is not sufficient.

Only after those paths work would I expand placement through Armada, add heterogeneous agent runtimes, or introduce broader peer collaboration.

The central design principle is:

> **Build a persistent task system with replaceable reasoning processes—not a collection of persistent conversations that happen to send messages.**

That gives you the team behavior you want while preserving the properties that matter over a long horizon: assignments remain known, decisions remain attributable, messages remain recoverable, effects remain inspectable, and success remains tied to evidence rather than to whichever terminal currently looks healthy.
<!-- preserved-owner-proposal:end -->
