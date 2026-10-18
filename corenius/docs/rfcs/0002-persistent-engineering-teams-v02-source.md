Yes. Those corrections materially improve the architecture.

I would now change two positions from the earlier design:

- **Kata (`kenn-io/kata`) should become a first-class component of Corenius, not an optional “maybe.”** It already implements much of the durable human/agent work ledger we would otherwise rebuild.
- **jj should be treated as a serious candidate for the default agent workspace VCS**, while keeping the final acceptance subject immutable and keeping Git-compatible protected integration initially.

And we should make Temporal much less hand-wavy.

The revised mental model is:

```text
                         CORIENIUS

 Human / TUI / CLI / Lead
             │
             ▼
┌────────────────────────────────────────────────────────────┐
│                    CORIENIUS BROKER                        │
│                                                            │
│ authority · admission · attempts · budgets · decisions     │
│ operations · candidates · evidence · reconciliation        │
└───────┬────────────────┬──────────────────┬────────────────┘
        │                │                  │
        ▼                ▼                  ▼
      Kata           PostgreSQL          Temporal
   WORK GRAPH      AUTHORITY GRAPH    COORDINATION GRAPH
        │                │                  │
        │                │                  ▼
        │                │           runtime activities
        │                │                  │
        └──────────┬─────┘                  ▼
                   │                 Runtime adapters
                   │             Codex · Claude · Trae
                   │                       │
                   │                       ▼
                   │                Armada / Insula
                   │                       │
                   ▼                       ▼
                  jj                 executions/jobs
              CHANGE GRAPH                │
                   │                      │
                   └──── candidate ───────┘
                            │
                            ▼
                  verifier / integration
```

The useful phrase here is:

> **Kata owns the work graph. Corenius owns the authority graph. Temporal owns the coordination graph. jj owns the change graph. Armada owns the execution graph.**

Those are related graphs, but deliberately **not one database pretending to be all five things**.

---

# 1. Kata's role should be much bigger

`kenn-io/kata` is almost exactly the work-ledger primitive Corenius needs. It is explicitly designed as a durable issue ledger for humans and coding agents, with stable references, relationships, atomic claims, events, idempotent mutations, and evidence-bearing closes. It is also explicitly *not* an agent worker pool or VCS. [GitHub](https://github.com/kenn-io/kata?utm_source=chatgpt.com)

So I would **stop building a second issue tracker inside Corenius**.

## Kata owns the durable work graph

For example:

```text
corenius#7fd2    Implement deterministic recovery
    ├── blocked-by corenius#1ac8
    ├── child corenius#91ba  Trae runtime reconciliation
    ├── child corenius#a261  cancellation race tests
    └── related corenius#77c0 Temporal worker versioning
```

Kata owns:

```text
issue identity
title/body
parent/child
blocks/blocked-by
related
priority
human/agent owner
comments
open/closed disposition
ticket event history
completion claim + ticket evidence
```

Kata already has atomic `claim`, revision-conditional mutations, idempotent create/close, event cursors, and evidence-based close semantics. [GitHub](https://github.com/kenn-io/kata/blob/main/docs/guide/concepts.md?utm_source=chatgpt.com)

That is substantial functionality we should reuse.

## But a Kata claim is NOT a Corenius grant

This boundary is critical.

```text
kata claim abc4
```

means approximately:

> “This actor currently owns/coordinates this ticket.”

It must **not** mean:

> “This process can now allocate eight B200s, push Git refs, spawn unrestricted agents, or publish artifacts.”

So:

```text
Kata ownership
    ≠ execution authority
    ≠ credential
    ≠ resource reservation
```

Likewise:

```text
Kata CLOSED
    ≠ verified candidate
    ≠ Corenius acceptance
```

Kata itself describes closing as a completion claim with evidence. That is excellent for the work ledger, but Corenius still needs stronger exact-subject verification semantics. [GitHub](https://github.com/kenn-io/kata/blob/main/docs/guide/concepts.md?utm_source=chatgpt.com)

---

# 2. Don't duplicate `Task` inside Corenius unnecessarily

I'd change the data model from:

```text
Corenius Task
  title
  body
  dependencies
  priority
  ...
```

to:

```text
WorkRef {
    tracker: "kata"
    project_id
    issue_id
}
```

plus:

```text
IntentRevision {
    work_ref
    kata_revision

    objective_snapshot
    acceptance_contract
    non_goals
    permitted_effects
    resource_envelope

    admitted_at
    admitted_by
}
```

This is subtle but extremely important.

Suppose Kata issue `abc4` says:

> Implement resumable SGLang serving.

Corenius admits revision 17.

Later someone edits the issue to:

> Implement resumable SGLang serving and deploy it to production.

The existing worker **does not suddenly gain deployment authority**.

We get:

```text
Kata:
    abc4 revision 18

Corenius:
    active intent = abc4@revision17
```

UI:

```text
⚠ ticket changed after admission
active execution uses kata#abc4@rev17
current ticket is rev18

[inspect diff] [admit revision 18] [keep current]
```

That's exactly the separation we want between **editable planning** and **admitted execution intent**.

---

# 3. Corenius should integrate through Kata's API/events, not its tables

Kata can run with PostgreSQL, including a shared daemon configuration. [GitHub](https://github.com/kenn-io/kata?utm_source=chatgpt.com)

I would deploy:

```text
Postgres cluster

  schema/database: kata
      owned by Kata

  schema/database: corenius
      owned by Corenius

Temporal:
      its own supported persistence database/schema
```

Do **not** have Corenius SELECT/UPDATE Kata's internal tables.

Instead:

```text
Corenius ← HTTP/MCP/API → Kata daemon
Corenius ← event cursor ─ Kata event stream
```

Kata already exposes durable events with cursors and live tailing, explicitly for hooks, audit, and automation. [GitHub](https://github.com/kenn-io/kata/blob/main/docs/guide/concepts.md?utm_source=chatgpt.com)

So a Kata event might wake Corenius:

```text
issue.changed
issue.claimed
issue.closed
relationship.changed
```

but the event is a **notification**.

Corenius subsequently asks Kata:

```text
give me authoritative abc4 revision 18
```

rather than assuming the event payload is the universe.

---

# 4. Lead decomposition should create actual Kata tickets

This becomes really nice.

Lead decides:

```text
Goal:
  make runtime launch reconciliation robust

Proposed decomposition:
  A. named runtime identity
  B. ambiguous-launch reconciliation
  C. cancellation tests
```

Broker checks its delegation authority.

Then it creates:

```text
kata create A --parent root --idempotency-key corenius/op-...
kata create B --parent root --blocked-by A ...
kata create C --parent root ...
```

Kata's create idempotency support is exactly what we need if Corenius crashes after the request succeeds but before receiving the response. [GitHub](https://github.com/kenn-io/kata/blob/main/docs/reference/cli.md?utm_source=chatgpt.com)

Then Kata becomes the surface both humans and agents naturally inspect:

```text
kata ready
kata show abc4
kata claim abc4
```

while Corenius determines which of those ready tickets are actually executable:

```text
Kata-ready
AND admitted
AND dependencies accepted
AND resource allocation available
AND conflict domain available
AND authority valid
= Corenius-admissible
```

That's a beautiful division.

---

# 5. jj should move much closer to the center

I undersold jj before.

For agentic development, several of jj's fundamental choices are unusually useful:

- the working copy itself is a commit;
- a **change ID survives rewrites** such as edits/rebases;
- the operation log records repository operations;
- multiple workspaces are first-class;
- Git-backed repositories produce ordinary Git-compatible commits. [GitHub](https://github.com/jj-vcs/jj?utm_source=chatgpt.com)

That's a much better match for:

```text
agent starts change
agent edits
lead changes base
change rebases
agent edits again
verification candidate eventually freezes
```

than reasoning primarily in branch names.

## The key Corenius representation

During active work:

```text
WorkChange {
    vcs = jj
    change_id = "kxq..."
    workspace_generation = 7
}
```

The `change_id` is the **logical evolving work identity**.

When the worker says:

> I have a candidate.

Corenius seals:

```text
Candidate {
    candidate_id: C31

    vcs: jj

    change_id: kxq...          # lineage / human navigation
    commit_id: 8f4c...         # immutable revision
    tree_digest: ...
    jj_operation_id: ...

    base_commit: ...
    task_revision: ...
}
```

Evidence binds to:

```text
commit_id / tree_digest
```

**not** merely the jj change ID.

Because jj's change ID deliberately remains stable while the underlying commit changes. [GitHub](https://github.com/jj-vcs/jj/blob/main/docs/tutorial.md?utm_source=chatgpt.com)

That gives us both identities we need:

```text
Change ID
    "this is still the same engineering change"

Commit ID
    "these are exactly the bytes we verified"
```

That's almost ideal for Corenius.

---

# 6. jj can substantially improve the rebase problem

Consider:

```text
worker candidate
       C
       │
main ─ A ─ B
```

Another candidate lands:

```text
main ─ A ─ B ─ D
```

With Git-centric worker semantics we start talking about:

```text
rebase branch
force update branch
candidate SHA changed
which branch owns what?
```

With jj:

```text
logical change X survives rewriting

X:
   old commit C1
       ↓ automatic rebase
   new commit C2
```

Corenius knows:

```text
change_id X is still the work item

but:

C1 != C2

therefore evidence(C1)
does not automatically verify C2
```

That is exactly the right model.

jj also automatically rebases descendants when an ancestor changes, which is attractive for stacks of dependent agent changes. [GitHub](https://github.com/jj-vcs/jj/blob/main/README.md?utm_source=chatgpt.com)

---

# 7. But don't give every agent unrestricted access to one shared jj repo

There is an important caveat.

jj's operation log is repository-wide, and it supports concurrent operations. Multiple workspaces can share one repo, but operations such as `jj undo` act against the shared operation history rather than a workspace-private history. The jj maintainers explicitly note that there is currently no workspace-specific operation log/undo. [GitHub](https://github.com/jj-vcs/jj/blob/main/docs/operation-log.md?utm_source=chatgpt.com)

That means:

```text
worker A: jj undo
```

could undo an operation produced from another workspace.

That's not acceptable as an isolation primitive.

So v1 should probably be:

```text
attempt/workspace
    ↓
private jj repository / controlled workspace generation
```

rather than:

```text
every untrusted worker
    ↓
one giant shared .jj repo
```

We can still exploit jj's local semantics.

Later, if we develop a properly restricted VCS gateway, shared jj repositories become more interesting.

Also note that workspaces can become stale if another workspace rewrites their checked-out commits; jj can detect and repair this, but that is another reason not to equate “jj supports multiple workspaces” with Corenius-level writer isolation. [GitHub](https://github.com/jj-vcs/jj/blob/main/docs/working-copy.md?utm_source=chatgpt.com)

---

# 8. I would make Corenius jj-first, Git-compatible

My v1 recommendation becomes:

```text
Workers:
    jj

Live engineering identity:
    jj change ID

Sealed candidate:
    immutable commit ID + tree digest

Remote interoperability:
    Git

Protected integration:
    initially Git ref CAS
```

Because jj's production backend is Git-compatible, jj-created commits can participate in existing Git remote workflows. [GitHub](https://github.com/jj-vcs/jj/blob/main/docs/git-compatibility.md?utm_source=chatgpt.com)

So underneath:

```text
              Corenius
                  │
           change C / commit H
                  │
                jj
                  │
           Git object model
                  │
              GitHub/etc.
```

This means we aren't locked into:

> jj versus Git.

For the first version it's closer to:

> **jj as the agent-facing VCS semantics, Git as the interoperability and protected-ref substrate.**

Eventually a fully qualified jj integration adapter could own protected bookmarks too.

But we shouldn't require that to get most of jj's benefits.

---

# 9. Now Temporal: precisely what is it doing?

I would narrow Temporal's responsibility considerably.

Temporal is **not**:

```text
the task DB
the agent DB
the authority system
the resource scheduler
the runtime supervisor
the evidence store
```

Temporal is:

> **The durable interpreter for coordination procedures that contain waits, timers, retries, and reactions to external events.**

Temporal workflows reconstruct their local state by replaying event history. Actual I/O belongs in Activities. Workflow Tasks are re-run deterministically after failures, while previously completed Activities/timers resolve from recorded history. [Temporal Docs](https://docs.temporal.io/tasks?utm_source=chatgpt.com)

That maps extremely well to Corenius if we use it carefully.

---

# 10. The primary Temporal entity should be a Scope Workflow

I would **not** create:

```text
one Temporal Workflow per agent
```

and probably not:

```text
one giant Workflow for the entire project
```

Instead:

```text
one Temporal Workflow per active Corenius execution scope
```

For example:

```text
Workflow ID:
corenius/scope/S17

Workflow:
TaskScopeWorkflow
```

A scope normally corresponds to one admitted Kata work item revision.

So:

```text
Kata issue abc4
      ↓ admit rev 17
Corenius scope S17
      ↓
Temporal workflow corenius/scope/S17
```

The workflow lasts as long as the **coordination problem**, rather than as long as one agent conversation.

That means:

```text
Trae dies
Codex replaces Trae
Claude replaces lead
broker restarts
```

but:

```text
TaskScopeWorkflow(S17)
```

continues.

---

# 11. Temporal Workflow code should mostly be a durable state machine

Something approximately like:

```python
@workflow.defn
class TaskScopeWorkflow:
    def __init__(self):
        self.changed = False

    @workflow.signal
    def state_changed(self, seq: int):
        self.changed = True

    @workflow.run
    async def run(self, scope_id: str):
        while True:
            state = await workflow.execute_activity(
                refresh_scope,
                scope_id,
                start_to_close_timeout=timedelta(seconds=30),
            )

            if state.terminal:
                return

            action = determine_next_coordination_action(state)

            if action:
                await workflow.execute_activity(
                    ensure_operation,
                    action,
                    start_to_close_timeout=timedelta(minutes=1),
                )

            self.changed = False

            await workflow.wait_condition(
                lambda: self.changed,
                timeout=next_reconcile_delay(state),
            )
```

This isn't meant as production code, but the primitives are real Temporal patterns:

- execute external work through Activities;
- use Signals to wake durable Workflows;
- use `wait_condition` to sleep without polling a model;
- use timers/timeouts for eventual reconciliation. The official Python samples demonstrate Activity invocation, Signals, `wait_condition`, retries, cancellation, children, and Continue-As-New. [GitHub](https://github.com/temporalio/samples-python?utm_source=chatgpt.com)

And most importantly:

```text
determine_next_coordination_action()
```

is operating over **broker state fetched through an Activity**.

The Workflow does not SELECT PostgreSQL directly and does not decide whether an old grant is valid from stale history.

---

# 12. Why the PG lookup has to be an Activity

Suppose Workflow history says:

```text
scope_epoch = 12
worker A authorized
```

Meanwhile, the human cancels it through Corenius:

```text
Postgres:
scope_epoch = 13
scope = REVOKED
```

Temporal may replay the Workflow's historical local state after a Worker restart.

It must **not** conclude:

> My replay says epoch 12 was valid, therefore continue.

Instead:

```text
Workflow wakes
      ↓
Activity: broker.refresh_scope(S17)
      ↓
broker returns current epoch 13 / REVOKED
      ↓
Workflow coordinates shutdown/reconciliation
```

Temporal's Event History tells us:

> where the coordination program got to.

Postgres tells us:

> what is authorized now.

That's the boundary.

---

# 13. Concrete Temporal example #1: start an agent safely

Lead decides worker execution is needed.

Broker transaction:

```text
POST /attempts

checks:
    Kata revision still admitted
    dependency ready
    scope epoch current
    worker slot available
    token budget available

writes atomically:
    attempt A17
    reservation R91
    operation O63
    event 1421
    outbox notification
```

Then Temporal wakes.

Workflow executes:

```text
Activity:
ensure_runtime_started(O63)
```

The Activity calls the Corenius runtime gateway.

Adapter does:

```text
operation O63
    ↓
Trae app-server
```

Network fails.

Temporal retries the Activity.

**This is where people often misunderstand Temporal.**

The Temporal retry does *not* magically make:

```text
start Trae
```

exactly once.

The Activity must call:

```text
ensure runtime for operation O63
```

not:

```text
spawn another runtime
```

The adapter therefore does:

```text
lookup runtime by operation O63

exists:
    return existing runtime

definitely absent:
    start one

unknown:
    return UNKNOWN / reconciliation needed
```

Temporal's own samples demonstrate automatic Activity retries; the application still has to make the Activity's external effect idempotent or reconcilable. [GitHub](https://github.com/temporalio/samples-go?utm_source=chatgpt.com)

This is one of Corenius's core protocols.

---

# 14. Concrete Temporal example #2: a 14-hour B200 training job

This is where Temporal is particularly useful—but **do not run the training process itself as a 14-hour Activity**.

Instead:

### Step 1

Workflow:

```text
Activity:
armada.submit(O77, ExecutionContract)
```

Armada returns:

```text
execution = exec-932
```

in seconds.

### Step 2

The physical job:

```text
exec-932
```

is owned and supervised by Armada/Insula independently.

### Step 3

Task Workflow sleeps:

```text
wait for:
    execution observation signal
OR
    durable 5-minute reconciliation timer
OR
    cancellation/scope-change signal
```

No LLM is consuming tokens.

No Temporal Worker thread is sitting there for 14 hours.

No SSH connection needs to stay alive.

### Step 4

Armada observer sees:

```text
checkpoint 27000
loss ...
job healthy
```

Broker records the observation.

Then the broker's outbox sends:

```text
Signal:
state_changed(sequence=1662)
```

to `corenius/scope/S17`.

### Step 5

Workflow wakes and asks through an Activity:

```text
broker.refresh_scope(S17)
```

If job finished, coordinate candidate collection.

If still running:

```text
wait again
```

If notification is lost:

```text
timer fires
→ refresh
```

That's exactly the kind of “wait days, survive failure, resume” workload Temporal is designed for. [Temporal Docs](https://docs.temporal.io/temporal?utm_source=chatgpt.com)

---

# 15. Why not model the long job itself as a Temporal Activity?

Temporal absolutely supports long-running Activities with heartbeats, and its samples recommend heartbeat timeouts for such Activities. [GitHub](https://github.com/temporalio/samples-python/blob/main/hello/hello_activity_heartbeat.py?utm_source=chatgpt.com)

But our physical GPU job already has a superior independent identity:

```text
Armada Execution ID
```

We explicitly want:

```text
Temporal dies
Corenius dies
agent dies
```

without:

```text
training dies
```

Therefore:

```text
short Activity:
    submit execution

durable external entity:
    Armada job

short Activities:
    inspect / reconcile / cancel
```

is cleaner than making Temporal itself the process supervisor.

It preserves the layering.

---

# 16. Concrete Temporal example #3: human approval

Say candidate C52 passes tests, but landing requires human approval.

Temporal gets to:

```text
WAITING_FOR_APPROVAL
```

and does nothing.

Hours later, TUI:

```text
approve promotion P13
```

Broker transaction checks:

```text
candidate still C52
target still appropriate
acceptance contract satisfied
human has authority
```

and records:

```text
Approval A9
event 1927
```

Then it signals:

```text
state_changed(1927)
```

Temporal wakes.

It does **not** trust:

```text
"approved"
```

as a Temporal signal payload.

It reloads:

```text
Approval A9
candidate C52
current target
current scope epoch
```

through the broker Activity.

Then coordination continues.

This is safer than making the Temporal Signal itself the permission token.

---

# 17. Concrete Temporal example #4: exact-candidate landing

Workflow determines from fresh broker state:

```text
promotion P91 = PREPARED
```

It executes:

```text
Activity:
apply_promotion(P91)
```

Inputs ultimately contain:

```text
operation_id = O991
candidate = commit 8f4c...
expected_target = 713b...
target = main
```

Integration gateway performs conditional mutation.

Now imagine:

```text
Git ref update succeeds
↓
machine loses network
↓
Activity result never reaches Temporal
```

Temporal retries.

Again:

**we do not call an unsafe `git merge` twice.**

The Activity calls:

```text
reconcile_or_apply(O991)
```

Integration gateway asks:

```text
did O991 already land?
```

using operation journals plus target state.

Possible result:

```text
APPLIED
```

or:

```text
target unchanged → safe to retry
```

or:

```text
AMBIGUOUS → promotion RECONCILING
```

Temporal then waits.

Temporal's Saga example demonstrates durable orchestration across failure/retry boundaries, but Corenius should not pretend every integration effect has a valid automatic compensation. A revert is a new candidate/effect, not merely “undo the Activity.” [GitHub](https://github.com/temporalio/samples-go/blob/main/saga/workflow.go?utm_source=chatgpt.com)

---

# 18. Signals, Updates, Queries: how I would use each

### Signals

Use as **wakeups / asynchronous notifications**:

```text
state_changed(seq)
execution_observed(exec, seq)
shutdown_requested(seq)
external_dependency_changed(seq)
```

Temporal's official samples show signals updating workflow-local state and waking `wait_condition`. [GitHub](https://github.com/temporalio/samples-python/blob/main/hello/hello_signal.py?utm_source=chatgpt.com)

Signal payloads should usually contain identifiers/cursors, not authoritative business state.

### Queries

Use for:

```text
debugging
Temporal UI
operational introspection
```

Never:

```text
Corenius truth
```

If:

```text
Temporal Query says RUNNING
PG says REVOKED
```

PG wins.

### Updates

Use sparingly where we want:

```text
request
+
durable workflow processing
+
synchronous response
```

For example:

```text
reconcile_now()
```

or a workflow-local administrative operation.

I would **not** make Updates the main authority API.

The broker transaction needs to happen first.

---

# 19. Temporal timers have very specific jobs

Timers are excellent for:

```text
retry observation in 30s
escalate if verifier hasn't started in 10m
refresh resource state in 5m
attention threshold after 20m
wake at budget review deadline
```

They're not:

```text
agent polling loop
```

So instead of:

```text
LLM:
"check again"
"still running?"
"check again"
```

we have:

```text
workflow sleeps durably
timer/event wakes workflow
agent invoked only if judgment is needed
```

This is potentially an enormous efficiency win.

---

# 20. We should mostly avoid Temporal child workflows for Kata task hierarchy

This is another revision I feel strongly about.

It is tempting to do:

```text
MissionWorkflow
    ├── child Task A
    ├── child Task B
    └── child Task C
```

But now Temporal's lifecycle semantics start defining our task ownership.

Temporal explicitly documents that parent close policy affects children, and that children are not automatically carried across a parent's Continue-As-New. Temporal also recommends not using child workflows merely for code organization. [Temporal Docs](https://docs.temporal.io/child-workflows?utm_source=chatgpt.com)

But our semantic ownership already exists:

```text
Kata parent/child graph
Corenius scope ownership
```

Therefore I'd use independently addressable:

```text
corenius/scope/S17
corenius/scope/S18
corenius/scope/S19
```

Temporal Workflows.

Their dependency relationship comes from Corenius/Kata, not from Temporal's parent-child lifetime semantics.

Use Child Workflows only when Temporal parent-child semantics are actually what we want.

---

# 21. Continue-As-New is still very useful

A Corenius scope could live for weeks and receive thousands of:

```text
state_changed
execution_observed
plan_updated
worker_changed
```

events.

Temporal history will grow.

So occasionally:

```text
TaskScopeWorkflow
    ↓
Continue-As-New
    ↓
same logical Workflow ID
new Run / small history
```

carrying only workflow-local coordination state such as:

```text
last_broker_sequence
current waiting mode
timer metadata
```

The real task state stays in PG/Kata.

Temporal's current agent integrations use Continue-As-New specifically to keep long-running agent workflow histories bounded. [Temporal Docs](https://docs.temporal.io/develop/python/integrations/strands-agents?trk=article-ssr-frontend-pulse_little-text-block\&utm_source=chatgpt.com)

That is precisely applicable.

---

# 22. This also changes how we should approach agent-generated workflows

Earlier we discussed agents generating Temporal workflow code.

I now think the **default should be declarative plan generation**, not arbitrary workflow source generation.

Why?

Temporal Workflows replay deterministically. Changing command-producing Workflow code can make existing histories nondeterministic unless properly versioned. Temporal explicitly requires versioning discipline around changes that schedule Activities, timers, children, etc. [Temporal Docs](https://docs.temporal.io/workflow-definition?trk=article-ssr-frontend-pulse_little-text-block\&utm_source=chatgpt.com)

So build:

```text
stable Temporal interpreter

        +

versioned Corenius Plan IR
```

Example:

```yaml
plan: 17

steps:
  - explore:
      roles: 2
      join: all

  - decide:
      actor: lead

  - implement:
      ticket: kata#42ab

  - verify:
      profile: cuda-unit-v3

  - if_failed:
      max_repair_rounds: 2
      action: diagnose

  - await_acceptance: {}
```

Agent changes:

```text
Plan R17 → R18
```

without rewriting the Temporal Workflow implementation.

That gives us dynamic agency while keeping workflow replay tractable.

Later:

```text
agent-authored custom workflow
```

can be treated exactly like code:

```text
candidate
→ tests
→ versioning compatibility
→ review
→ promotion
```

not `eval()`ed inside the control plane.

---

# 23. A global reconciliation workflow is also worthwhile

Besides per-scope workflows, I'd have something like:

```text
ReconciliationSweep
```

started by a Temporal Schedule.

Periodically:

```text
find scopes with:
    UNKNOWN operation
    stale runtime observation
    missing Temporal coordination workflow
    stuck promotion
    missed notification
```

Then invoke bounded reconciliation.

This is the anti-entropy path.

The rule becomes:

```text
events give low latency
reconciliation gives eventual correctness
```

This is exactly the same design philosophy as Kubernetes controllers.

---

# 24. The revised authority map

I would now make this explicit in the spec:

| System | Authoritative for | Explicitly NOT authoritative for |
|---|---|---|
| **Kata** | Work items, hierarchy, dependencies, planning ownership, comments, work-level close history | Runtime authority, compute budgets, process truth, exact candidate acceptance |
| **Corenius/Postgres** | Admitted intent, scopes, grants, attempts, operations, budgets, decisions, candidate/evidence bindings, promotion | Editable ticket prose, actual OS process existence |
| **Temporal** | Durable progress of coordination procedures | Task truth, permissions, process truth |
| **jj** | Mutable engineering change evolution and local VCS provenance | Whether work is authorized or accepted |
| **Armada/Insula** | Concrete execution identity, environment, runtime observations, lifecycle evidence | Why execution was authorized or whether result satisfies task |
| **Agent runtime** | Conversation/session state and proposed actions | Team truth or promotion authority |
| **Verifier** | Predicate results against exact candidates under a named environment | Permission to land |
| **Integration gate** | Conditional mutation of protected targets | Defining acceptance |
| **Artifacts** | Immutable bytes and integrity | Meaning/trust of those bytes |

And I would rename the graphs explicitly:

```text
Kata       → Work Graph
Corenius   → Authority / Provenance Graph
Temporal   → Coordination History
jj         → Change Graph
Armada     → Execution Graph
Evidence   → Verification Graph
```

That makes Corenius much easier to reason about.

---

# 25. Revised end-to-end flow

The whole system then becomes:

```text
Human defines goal
       │
       ▼
Kata issue G
       │
       ▼
Lead proposes decomposition
       │
       ▼
Broker authorizes ticket mutations
       │
       ├── Kata issues A/B/C
       │
       ▼
Corenius admits A@revision7
       │
       ├── Scope S17
       ├── budget allocation
       ├── permissions
       └── Temporal Workflow corenius/scope/S17
                    │
                    ▼
              start Attempt A1
                    │
                    ▼
              Codex / Trae / Claude
                    │
                    ▼
              private jj workspace
                    │
                    ├── evolving change X
                    │
                    └── Armada executions E1/E2
                              │
                              ▼
                         observations
                              │
                    PG ← Broker ── signal → Temporal
                              │
                              ▼
                    immutable candidate C
                              │
                    commit ID + tree digest
                              │
                              ▼
                         verification
                              │
                              ▼
                     acceptance contract
                              │
                              ▼
                      promotion prepared
                              │
                              ▼
                   protected Git/jj gate
                              │
                              ▼
                          landed C
                              │
                              ▼
                   Kata closed with evidence
```

Notice that **closing Kata is nearly the final step**, rather than starting the chain of execution semantics.

And if a human closes it independently:

```text
Kata CLOSED
Corenius scope still active
```

Corenius surfaces the discrepancy rather than silently inventing acceptance.

---

# 26. Where I'd land the three decisions now

### Kata

**Promote from “optional experiment” to a first-class dependency/interface.**

Don't reimplement its issue tracker.

Use it as the durable work graph, with Corenius admission pinning an exact ticket revision.

### jj

**Promote from “interesting optional VCS” to the preferred agent-working VCS candidate.**

Use:

```text
change ID = evolving engineering identity
commit ID = immutable verification identity
```

Initially isolate worker repositories/workspaces and retain a Git-level protected integration primitive.

### Temporal

**Keep as core infrastructure, but sharply constrain its semantics.**

Use it for:

```text
durable waits
timers
coordination retries
long-lived state-machine continuation
wakeups
bounded fan-out/fan-in
reconciliation procedures
```

Do not use it as a shadow database or permission system.

And I think that gives us a notably stronger architecture than v0.1:

> **Kata says what work exists. Corenius says what work is authorized. Temporal makes authorized coordination survive time and failure. jj gives agents a first-class model of evolving changes. Armada makes computation independently durable. Verification establishes what exact result counts.**

That is the decomposition I'd now consider the basis for **Corenius v0.2**.