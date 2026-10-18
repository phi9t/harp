**The strongest version of Corenius is a persistent engineering team—not a persistent agent conversation.** Its members can change, its strategy can evolve, and its execution can span machines, without losing the objective, resetting the budget, abandoning useful work, or confusing activity with verified progress.

I consolidated the available material into a proposed RFC:

**corenius-design-v0.1.md[Corenius design RFC v0.1](sandbox:/mnt/data/corenius-design-v0.1.md)** — architecture, domain model, operation protocols, runtime contracts, recovery, verification, delivery sequence, and **32 fault-injection cases**.

The synthesis draws on your current Corenius brief, the earlier Harp Durable Execution design, the Insula execution-contract specification, and the later Armada product-surface design. These are architectural inputs, not proof of a currently implemented system; I did not establish a Corenius implementation to audit.

Here is the design I recommend.

# 1. The product promise: work persists; agents are replaceable

The defining demonstration should be:

> A lead assigns work, a worker starts an expensive job, and verification begins. We disconnect the terminals, replace the lead, and restart the broker. The new lead reconstructs the objective, discovers existing execution, understands unresolved decisions, and continues without blindly repeating work or weakening acceptance.

That is more valuable than “we can launch ten agents.”

Corenius should make six questions answerable without asking an LLM:

| Question | Durable answer |
|---|---|
| What are we trying to accomplish? | An admitted objective and acceptance contract |
| Why are we taking this approach? | Versioned plans and explicit decisions |
| What is actually running? | Independently addressable executions and attributed observations |
| What may happen next? | Current grants, dependencies, budgets, and policy |
| What have we established? | Candidate-bound evidence and qualified claims |
| What requires my attention? | Specific blockers, approval requests, uncertainty, or conflicting decisions |

The earlier HDE proposal already contains much of this foundation: durable objectives across replaceable attempts, independent jobs, evidence, authority, and promotion. We should carry that forward rather than reinventing it under another name. Its earlier local-store and optional-workflow-backend choices are superseded here by your approved **PostgreSQL + Temporal** baseline. harp-durable-execution-design harp-durable-execution-design

My central architectural recommendation is:

**Keep the permission and acceptance core small and strict. Put flexibility into plans, runtime adapters, execution recipes, context compilation, and domain verifiers.**

# 2. The most important refinement: separate the lifetimes

Our previous “Task → Attempt → Runtime” model was useful, but incomplete.

**An attempt must not be the lifetime owner of every job it starts.** Otherwise replaceability quietly breaks: replacing an agent either kills useful work or leaves it orphaned.

The model should instead be:

```text
Mission
  └── Task
        ├── Task revision: outcome + acceptance contract
        ├── Plan revision: current strategy
        └── Durable execution scope
              ├── Attempt A1 → Trae thread X
              ├── Attempt A2 → replacement runtime Y
              ├── Execution E1 → training / build / verification job
              ├── Workspace generation W4
              └── Candidates → evidence → acceptance / promotion
```

An execution records both:

```text
created_by_attempt = A1       # provenance
owned_by_scope     = S17      # lifetime and authority
```

A1 can disappear while E1 remains authorized and supervised. A2 can discover E1, inspect its progress, and use its results without relaunching it.

This develops a distinction already present in HDE: attempts, sessions, jobs, workspaces, and execution scopes are separate records, with different lifetimes. harp-durable-execution-design

## Separate leadership transfer from cancellation

We need several independent generations—not one overloaded “epoch”:

| Identity/version | What it protects |
|---|---|
| `task_revision` | The authorized outcome and acceptance criteria |
| `plan_revision` | The strategy currently being pursued |
| `role_epoch` | Who currently occupies a lead or other controlling role |
| `scope_epoch` | Which execution authority remains valid |
| `workspace_generation` | Which mutable checkout a writer owns |
| `runtime_incarnation` | Which physical runtime instance produced an observation |

**Replacing the lead normally increments its role epoch, not every task’s scope epoch.**

Valid worker grants should be rooted in durable task scopes, not in the continued existence of the lead process. Otherwise a leadership change unnecessarily invalidates the whole team.

A returned old lead can contribute historical observations, but its obsolete role assignment cannot issue new authoritative commands.

## Avoid a giant status enum

The following should be expressible simultaneously:

```text
Task: awaiting acceptance
Scope: paused for new work
Lead: disconnected
GPU execution: healthy and explicitly retained
Candidate: sealed
Verification: running
Cleanup: not yet due
```

Trying to compress this into `task.status = RUNNING` loses the information needed for recovery.

This also preserves the Insula principle that workload outcome, conformance, cleanup, and resource release are separate facts. A successful computation followed by failed cleanup must not become an undifferentiated “failure.” 2026-09-16-insula-execution-pro…

# 3. PostgreSQL and Temporal: one authority, durable coordination

The division should be precise:

**PostgreSQL records authorized intent and domain state. Temporal records coordination progress. Runtime gateways report physical observations.**

None should impersonate the others.

The normal mutation path is:

```text
Authenticated command
    ↓
PostgreSQL transaction
    validate authority and revisions
    reserve budget/resources
    create or deduplicate operation
    update domain state
    append audit event
    write outbox record
    ↓ commit
Outbox relay
    ↓
Temporal starts or wakes coordination
    ↓
Activity invokes an authenticated broker operation
    ↓
Runtime / execution / integration gateway
    ↓
Observation or receipt returns to the broker
```

The state change, reservation, operation identity, audit event, and outbox entry commit together. The external side effect happens afterward.

That gives us a recoverable handoff without pretending PostgreSQL, Temporal, Git, and remote execution share a transaction.

## Correct one detail from my earlier sketch

The earlier illustrative workflow directly called `broker.get_task_state()`. In a real Temporal implementation, that database/network access belongs in an **Activity**, not directly in workflow code. Workflow code must preserve deterministic replay behavior; effect execution still needs application-level idempotency and reconciliation. [GitHub](https://github.com/temporalio/temporal/blob/main/docs/architecture/README.md)

A workflow should therefore coordinate a small set of operations:

- Refresh the relevant broker state through an Activity.
- Request an admitted operation.
- Wait for a durable notification or timer.
- Reconcile when an observation is missing or ambiguous.

It should not independently decide that an old assignment remains authorized because its history says so.

**A replayed result is historical knowledge, not a fresh permission grant.**

## Do not keep agents busy while the system is waiting

An expensive training or verification job should be submitted through a short operation that returns a durable execution ID. Coordination then waits for job observations or deadlines.

There is no reason to keep making model calls to ask, “Has it finished?”

Use versioned scope notifications, deduplicated delivery, and periodic reconciliation. A notification means “inspect newer state,” not “trust this message as authority.”

Keep raw token streams, large logs, datasets, and checkpoints outside PostgreSQL transactions and Temporal histories. Store their immutable references instead. This follows HDE’s useful distinction between transactional state, audit events, and bulk artifacts. harp-durable-execution-design

# 4. Dynamic workflows: combine durable plans with bounded judgment

This is where Corenius can become substantially more capable than a reliable task queue.

We discussed having agents generate Temporal workflows and execute them. I would preserve that ambition, but introduce **two paths**.

## Ordinary path: typed plans with explicit decision points

Start with a trusted, versioned interpreter over a small set of operations:

```text
Investigate
    → propose implementation plan
    → admit independent work
    → await candidates
    → construct integration candidate
    → verify
    → accept and land

On failure:
    → classify evidence
    → request bounded diagnosis
    → admit a revised plan
```

The plan can contain parallelism, joins, bounded repair loops, waits, and agent decisions.

The crucial separation is:

**The acceptance contract defines what counts as success. The plan defines how we are currently trying to get there.**

A lead can change an algorithm or split a task differently within its authority. It cannot quietly lower the numerical tolerance, remove a required test, increase the compute budget, or authorize publication just because the current approach is failing.

Each consequential agent decision becomes a record containing the question, selected alternative, supporting evidence, deciding role, and applicable authority.

That makes agent reasoning operationally useful without requiring the broker to recover semantics from chat.

## Advanced path: agent-authored workflow bundles

An agent can also author real workflow code. But that code should become an immutable **workflow candidate** that is tested, reviewed, and promoted into a registered recipe.

It should not be evaluated directly inside a privileged control-plane worker.

This distinction matters because Temporal’s Python sandbox is explicitly a determinism helper, **not a security sandbox for arbitrary untrusted code**. [GitHub](https://github.com/temporalio/sdk-python)

A promoted workflow bundle should pin its implementation, dependencies, allowed operations, resource limits, and compatibility requirements. Changes go through the same candidate/evidence/promotion discipline as other engineering work.

This gives us a progression:

**One-off agent strategy → repeatable typed plan → qualified workflow recipe.**

Useful procedures become reusable without becoming privileged merely because an agent wrote them.

## Where Pi Durable fits

I would place harness-level durability below Corenius, not in competition with it. Pi Durable describes persistence around agent/task execution, including model, tool, and compaction boundaries, with explicit interrupted-operation behavior. That can improve an individual runtime adapter. It does not eliminate the need for team authority, independent execution ownership, budgets, or protected integration. [Earendil](https://earendil.com/posts/pi-durable/)

The design question is not “Temporal or a durable agent harness?”

It is **which layer owns which continuation**.

# 5. Native runtimes should remain native

We should not rebuild Codex, Claude, or TraeCLI inside Corenius.

Their native loops should continue to own inference, conversation handling, and supported tool behavior. Corenius wraps them through a tested semantic contract.

The adapter needs more than `start()` and `stop()`. It should report whether the concrete runtime supports inspection, steering, interruption, reconnect, discovery after an uncertain start, descendant enumeration, checkpoints, usage reporting, and enforceable limits.

**Capability support must be demonstrated, not inferred from a provider name.**

Current Codex app-server documentation provides useful native distinctions: threads, turns, items, reading persisted threads versus resuming them, and steering active turns. It also documents that replacing a native goal objective resets that goal’s usage accounting. Corenius therefore needs independent scope-level accounting that survives native goal changes. [OpenAI Developers](https://developers.openai.com/codex/app-server)

## Exactly one continuation owner per attempt

An attempt should explicitly select:

```text
continuation_owner = broker
```

or:

```text
continuation_owner = native_goal_loop
```

In the second case, the native runtime advances within a broker-admitted envelope. Corenius must not simultaneously keep submitting new turns because it thinks it owns advancement.

Otherwise we create a subtle two-controller system inside each worker.

## Integration quality is a spectrum

An adapter might be observation-only, able to supervise a process, or able to mediate protected effects and account for descendants.

Those are different guarantees.

Launching a CLI does not establish exact per-request accounting. Seeing a terminal does not establish job ownership. A persisted thread does not establish ongoing inference. Unregistered native subagents cannot be silently included in a claim of complete lifecycle control.

This preserves HDE’s explicit integration tiers and its warning against fabricating guarantees from an opaque harness. harp-durable-execution-design

The human-facing runtime TUI remains useful—but as an attached view, not the team’s control plane.

# 6. Corenius coordinates execution; Armada and Insula implement it

We should use the substrate work you have already done.

The valuable Insula abstraction is:

> A typed profile and recipe compile into an immutable Execution Contract; generic execution machinery owns the lifecycle afterward.

That contract already separates preparation and workload specialization from placement, launch, supervision, shutdown, cleanup, and evidence capture. Corenius should call this seam rather than invent another SSH launcher. 2026-09-16-insula-execution-pro…

The higher-level division becomes:

**Corenius:** coordinate a campaign, wait, retry over time, fan out, manage dependencies, allocate budgets, interpret outcomes.

**Armada:** admit and manage a concrete execution on a concrete target.

**Insula:** provide the selected environment boundary and its conformance evidence.

The later Armada design explicitly keeps a profile scoped to one execution rather than embedding multi-host orchestration inside it. Corenius is the natural place for that higher-level coordination. 2026-09-17-armada-product-surfa…

For ML systems work, a durable execution should carry a progress contract: startup deadline, maximum admitted runtime, telemetry expectations, checkpoint support, termination method, partition behavior, and recovery capability.

A quiet compilation, numerical checker, training job, and interactive agent should not share one generic idle timeout.

Diagnostics also need first-class treatment. A diagnostic attempt may require the workload’s namespace visibility, but it still has its own authority, resource allocation, and termination policy. Access to debugging capabilities must be observed on the actual target, not assumed because it is reachable over SSH.

**Kata Containers stays optional.** Qualify it for the relevant target and workload rather than treating its installation as proof that all required isolation, accelerator, and debugging behavior is available. The project’s VM-based isolation model is relevant, but Corenius’s guarantees must depend on the configuration actually deployed. [Kata Containers](https://katacontainers.io/)

# 7. Recovery should preserve useful work without preserving stale authority

The reconciliation loop compares:

**Authorized desired state → recorded operation state → observed physical state.**

These are deliberately not the same thing.

| Situation | Corenius response |
|---|---|
| Lead disappears; job remains healthy | Replace the lead and adopt existing work |
| Launch response is lost | Discover the existing operation or preserve `UNKNOWN` |
| Old lead returns | Reject obsolete controlling commands; retain useful historical observations |
| Old writer may still be alive | Give the replacement a different writable generation |
| Cancel races with promotion | Serialize the grant decision and report effects already committed |
| Cancelled work returns a useful artifact | Quarantine it; require current authority before use |

A useful refinement to our earlier discussion:

**“Do not blindly replace uncertain execution” does not mean “block all progress forever.”**

A policy can authorize isolated replacement work with a new workspace, separate reservation, and no conflicting protected effects. The original possible execution remains outstanding and accounted for.

That is controlled redundancy, not accidental duplication.

## Give human control precise meanings

**Pause** stops new ordinary admission, with an explicit policy for existing jobs.

**Quiesce** drains work to a consistency boundary.

**Cancel** revokes future authority and requests termination.

**Take over** replaces a controlling participant while retaining work that remains authorized.

Cancellation must return both an authority receipt and actual stop status. An effect already granted may complete afterward; a partitioned process may remain unconfirmed.

And a lease timeout alone is not sufficient fencing: the protected resource or its gateway must reject stale operations. For writable files still mounted in an old process, allocating a new isolated workspace generation is often the safer handoff. [Martin Kleppmann](https://martin.kleppmann.com/2016/02/08/how-to-do-distributed-locking.html)

Backup restoration needs the same rigor. Restoring an old database must not resurrect old grants while remote jobs still exist. Recovery should close admission, establish fresh authority, reconcile surviving execution, and explicitly re-admit retained work. HDE already identifies this as a separate recovery problem from restoring bytes. harp-durable-execution-design

# 8. Verification and integration are a protocol, not a “review agent”

This is the other place where correctness must be stronger than the surrounding agent behavior.

A candidate should be an immutable manifest identifying the exact code/artifacts, relevant input identities, task revision, and producer.

An evidence receipt should bind that candidate to the acceptance contract, verifier recipe, actual environment, collected checks, outcome, coverage, skips, and limitations.

The trusted receipt producer must sit outside the candidate’s writable environment. A worker-generated `PASS.json` is not equivalent to a verifier receipt. The earlier HDE design makes this trust boundary explicit. harp-durable-execution-design

## Verify the integration candidate, not merely the feature branch

The protocol should be:

```text
Protected target is B
    ↓
Construct exact integration candidate C based on B
    ↓
Verify C
    ↓
Prepare promotion: operation O, expected B, candidate C, evidence E
    ↓
Recheck authority and serialize commit admission
    ↓
Conditionally fast-forward B → C
    ↓
Record or reconcile the actual effect
```

The critical rule:

**No rebase, merge, cherry-pick, or “tiny fix” after verification followed by reuse of the old green status.**

A changed subject requires a new applicability decision, and initially that should mean exact-subject verification.

Git’s expected-old-value ref update supplies a useful compare-and-set primitive. It does not make Git and PostgreSQL atomic together, and it does not synchronize a checked-out worktree. [Git](https://git-scm.com/docs/git-update-ref)

## Protect the integration repository itself

I recommend a gate-owned integration repository, preferably bare for the initial protected target. Ordinary workers should not have write access to its common metadata.

Separate worktrees are useful for collaboration, but they share repository state; they are not by themselves a protected integration boundary. [Git](https://git-scm.com/docs/git-worktree)

After a crash between the Git mutation and the database receipt, the gate must reconcile the operation before another conflicting landing. A matching ref is sufficient only when the exclusive-writer history establishes attribution; otherwise preserve uncertainty.

Local landing, remote push, publication, deployment, and deletion remain separate effects.

## jj is a workspace improvement, not an acceptance primitive

jj’s stable change identity can follow evolving work while commit identity changes. That makes it potentially useful for agent editing and navigation—but acceptance should still bind an immutable revision or manifest, not an unqualified change ID. [GitHub](https://github.com/jj-vcs/jj/blob/main/docs/tutorial.md)

Thus neither jj nor Kata needs to become part of Corenius’s conceptual core.

# 9. Durable knowledge should be compiled, not dumped into context

A replacement lead should receive a **recovery manifest and briefing**, not an entire transcript.

The manifest should contain a consistent state cutoff and authoritative references for:

- The objective, non-goals, acceptance contract, and current task graph.
- Approved decisions, open questions, relevant findings, and superseded assumptions.
- Exact workspace/checkpoint/candidate identities, running jobs, pending effects, and observation freshness.
- Current permissions as a summary—not reusable credentials—and settled/reserved budget.

This develops HDE’s distinction between authoritative context, current working material, and historical context. It also preserves the rule that exact constraints and unresolved blockers must not survive only as model-generated paraphrases. harp-durable-execution-design

Three important design choices follow.

**First, a briefing is a snapshot, not a grant.** Actions revalidate current state.

**Second, warm continuation is an optimization; cold continuation is the portability contract.** Preserve a useful native conversation when possible, but make no correctness dependency on hidden reasoning or provider-specific state.

**Third, shared memory needs promotion.** An agent can propose a lesson, but durable team knowledge should record supporting evidence, conditions of applicability, contradictions, and supersession.

A failed experiment should be remembered as:

> This approach failed under these conditions, with this evidence.

Not:

> Never try this approach.

That distinction is essential for research.

# 10. The interface should organize attention, not display a wall of agents

This is where the system can feel genuinely excellent.

The main screen should not begin with ten scrolling terminals. It should begin with:

**What changed? What is blocked? What needs my decision? What is still running? What is ready to accept?**

An illustrative task card:

```text
Task T17: reduce attention backward memory

Contract R3 · Plan P6 · Scope active

Candidate C31
  Numerical checks: passed
  Performance check: running
  Integration: not landed

Execution E42
  Healthy · last observation 8 seconds ago
  Checkpoint CP9
  Independent of the disconnected lead

Budget
  Settled 46% · reserved 18%
  One usage record unresolved

Human decision: none
Action available: replace lead without restarting E42
```

The native terminal is one drill-down view. Other equally important views are the task graph, execution inspector, candidate comparison, evidence bundle, budget ledger, and causal timeline.

## “Why?” should be a first-class operation

Every important status should explain its provenance:

```text
Why is this blocked?
    → Required evidence missing
    → Verification job could not start
    → Target lacks required capability
    → Last observation and exact failed predicate
    → Safe available actions
```

That explanation should be machine-derived where possible.

For your editor/trace-explorer direction, the same event model can support hierarchical entries, Dired-like marking, previews, and sending selected events into a diagnostic attempt. A curated event selection becomes a referenced context artifact—not a copied blob whose provenance is lost.

All clients should share the same IDs and command semantics. An editor, TUI, CLI, or model-facing tool interface must not become another state store.

## Protect the operator from unnecessary interruptions

Batch nonurgent observations. Wake the lead when a meaningful decision is ready. Escalate a bounded blocker with the relevant evidence, alternatives, and cost implications—not a stream of vague “still working” messages.

Human attention should be spent on scope, consequential tradeoffs, and acceptance—not on remembering which tmux pane owns the training run.

# 11. Make autonomy economically accountable—and research honest

Budget accounting should live at the durable scope level:

\[
\text{available}
=
\text{allocation}
-
\text{settled usage}
-
\text{outstanding reservations}.
\]

Child allocations are transfers, not newly created funds. Uncertain operations retain an appropriate reservation. A disconnected conversation, new native goal, or replacement provider does not reset the ledger.

A hard ceiling should be promised only where the underlying runtime or provider can enforce it. Otherwise label the budget soft and expose the uncertainty. This follows HDE’s distinction between observed usage, outstanding commitments, and actual enforceability. harp-durable-execution-design

Scheduling should reserve capacity for verification, reconciliation, and control actions. Generation must not consume every slot while verification and cancellation wait behind it.

Measure **accepted quality, latency, cost, human attention, rework, recovery success, and uncertain-runtime exposure** separately. Agent count is not an objective.

For ML and systems research, use domain contracts that identify the question, baseline, dataset/model/code, hardware/toolchain, metrics, trial-selection policy, and checkpoint lineage.

Then distinguish:

**The investigation completed**  
from  
**The hypothesis was supported.**

A useful negative result can complete a research task. A failed or censored trial must remain in the record rather than disappearing from an attractive summary.

Finally, keep harness improvement separate from production execution. New skills, context policies, routing strategies, and workflow templates should be evaluated as versioned candidates under frozen acceptance rules. The evaluated agent cannot modify its own evaluator and then declare improvement. harp-durable-execution-design

# 12. Build a small system that earns its guarantees

I would not begin with a distributed swarm, a large plugin ecosystem, or a custom scheduler.

Begin with one logical authority and a narrow vertical slice. The RFC’s delivery sequence is:

| Slice | Deliverable | Qualification gate |
|---|---|---|
| Durable core | PostgreSQL operations/outbox, Temporal coordination, scope/role epochs, fake runtime | Duplicates, stale commands, cancellation races, and budgets behave correctly |
| One engineering task | Real lead and worker, isolated workspace, candidate, verifier, local integration | Goal-to-landing survives specified crashes without bypassing acceptance |
| Durable remote jobs | Armada execution integration, observation, adoption, partition policy | Replacement discovers existing work without duplicate launch |
| Controlled parallelism | Typed plan revisions, allocations, joins, conflict domains, attention UI | Higher throughput without weaker control or higher hidden rework |
| Research and improvement | Experiment contracts, claim evidence, memory promotion, harness evaluation | Results include uncertainty, failures, cost, and interventions |

The important early tests are adversarial: lose a launch acknowledgment; return an old lead; resume a stale writer; move the target branch; change the candidate after PASS; kill the broker after landing but before receipt; restore a backup while remote jobs survive.

The RFC specifies 32 such cases.

For formal assurance, model-check the small authority/operation state machine first: two role epochs, two workspace generations, duplicated messages, cancellation, and promotion. Then separately test the actual database, runtime, execution, and integration adapters. A proof about the abstract state machine is not proof of the operating system integration.

## The first demonstration that would convince me

Run one meaningful engineering task with a lead, worker, independent verifier, and an externally supervised job.

Then deliberately remove the lead’s conversation, restart the control plane, and attach a fresh lead—potentially using another provider.

The replacement must identify the active contract, existing job, current candidate, unresolved evidence, budget commitments, and safe next action. It must preserve useful execution, reject stale control, and land only the exact verified candidate.

**That is the foundation worth building: agency becomes replaceable without making work disposable.** Everything more ambitious—larger teams, agent-authored workflows, richer research campaigns, better memory, and improved harnesses—can then grow without changing what is authorized or what counts as success.