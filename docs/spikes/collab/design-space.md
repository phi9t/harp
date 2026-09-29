# Multi-agent collaboration: design space and recommended architecture

Status: research half of the collab spike. Branch `claude/collab-spike`. Not committed.
Companion: the sibling agent's discrete-event, git-backed simulator under `experiments/collab/`
(empty when this was written). Section 9 lists what should be measured there.

## 0. How to read this document

Provenance tags, used throughout, because the brief asks for honesty about sources:

- **[L]** Read in a local file during this spike (path given). Line-level claims marked [L] were
  checked against the source.
- **[K]** From my own background knowledge. **Not verified against a source in this spike.** No URLs
  or citations are given, on purpose. Treat details (version numbers, exact option names, exact
  semantics) as things to confirm before relying on them.
- **[I]** Inference or opinion. This is my judgment, and it is where I am most likely to be wrong.

The document is opinionated. The recommendation is in section 6. If you read one thing, read
sections 2 (what actually breaks today), 6 (architecture) and 9 (what to measure).

---

## 1. The problem, stated precisely

"Multi-agent collaboration" bundles five different scarcity problems. Mechanisms differ mostly in
which of them they address, so name them first.

| # | Scarce thing | Who contends | Cost of a collision |
|---|---|---|---|
| S1 | **Source text** (a file, a hunk, a symbol) | agents editing overlapping code | rebase conflict; or worse, a silent semantic conflict that compiles |
| S2 | **The landing ref** (mainline) | every agent that wants to finish | broken trunk; or serialization latency |
| S3 | **Gate capacity** (Insula runs, mostly GPU-bound: `runtime-test`, `accel-nccl`, `b200-release`) | landing agents plus development runs | hours of queueing; wasted re-gating after retry |
| S4 | **Attention**: the human's review/merge authority, and each agent's context window | human; agents whose context is stale | rubber-stamped merges; agents acting on outdated instructions |
| S5 | **Shared mutable state** (cache, tracker files, roadmap, `Cargo.lock`, registries) | everyone | conflicts on files no one owns; deleted state; disk exhaustion |

Safety properties I think the mechanism must protect, whatever else it does (these become the
simulator's invariants in section 9):

- **I1** The landing ref only ever points at commits whose exact tree passed the required gates.
  (Already an invariant in both milano and vaso [L].)
- **I2** One writer per working tree. (Both baselines [L].)
- **I3** A scarce device is never held by two parties; a dead holder's device is eventually
  reclaimed.
- **I4** Every coordination fact is attributable, append-only and replayable.
- **I5** The human can always veto, reorder, or take over, and the system says clearly what
  requires their decision.
- **L1** (liveness) Every request made by a live agent is eventually served, including big requests
  such as an 8-GPU release gate.

Scale targets to keep in mind: (a) 2-4 agents, where overhead must approach zero; (b) 30+ agents,
where coordination cost, decomposition quality and human attention dominate; (c) GPU-scarce gates,
where one 8-GPU qualification blocks the host; (d) the human keeps merge authority.

---

## 2. What the internal baselines actually do, and where they break

### 2.1 milano (`AGENTS.md`, `milano/scm.py`, `scripts/lib/gpu_lease.sh`) [L]

Design: one worktree per task on `<agent>/<topic>`; an integration flock in the git common dir;
`integrate` = fetch, rebase, gate the exact tip in the Insula, fast-forward, push, retry up to 3
times; a pre-push hook rejecting non-ff and merge commits on mainline; GPU leases as exclusive
flocks on `$MILANO_CACHE_ROOT/locks/gpu/gpu-N`, with `.holder` files as advisory metadata.

What I verified by reading the code, and think matters:

1. **The integration lock is held across the gates.** `integrate` takes `integration_lock` around
   the whole retry loop including `run_gate` (`scm.py:168-190`). Landing throughput is therefore
   `1 / (sum of gate times)`, and gate time includes GPU lease waits of up to 900 s each
   (`MILANO_GPU_LEASE_WAIT`). Every other agent's `integrate` blocks on `flock(LOCK_EX)` with no
   timeout, no queue position, and no visibility of who is ahead.
2. **The real serialization point is the remote ref, not the lock.** `git push tip:refs/heads/...`
   is a compare-and-swap: a non-ff push is rejected. The lock only avoids wasted gating. It is a
   host-local optimization: it does not order two hosts. [I]
3. **A rejected push re-runs every gate** (loop restarts at fetch/rebase/gate). Under contention
   this multiplies GPU demand exactly when GPUs are scarcest.
4. **Rebase conflict returns one line** (`does not rebase cleanly ... resolve and retry`) after
   `rebase --abort`. No file list, no "who else touched this", no task that caused it. The agent
   rediscovers the conflict from scratch.
5. **The GPU lease retry loop is release-all-and-sleep-10** (`gpu_lease.sh:29-53`). It never holds
   partial sets, so it cannot deadlock, but it also cannot serve a large request while small
   requests keep arriving: an 8-GPU request needs all 8 free in one 10-second polling instant.
   `AGENTS.md` says qualification "holds all 8"; there is no drain or reservation mechanism to make
   that possible. Small jobs starve big ones. **This is a liveness (L1) bug by construction**, not
   a tuning problem.
6. **`.holder` metadata lies after failed partial attempts.** The holder file is written
   immediately after each successful `flock -n` (line 34-35), before the loop knows it will keep
   the whole set. If the attempt then fails and releases, the file still names that process, so
   the "who holds this" message printed to waiters can name a process that holds nothing. Cosmetic
   today; misleading once humans or agents act on it.
7. **`MILANO_GPU_LEASE` is a forgeable capability.** Any child that inherits the env var skips
   acquisition (`gpu_lease.sh:13-15`). It is correct for the intended nesting (qualification gates
   reuse the parent's lease) but nothing checks that the holder is still alive, that
   `CUDA_VISIBLE_DEVICES` still matches, or that the process tree is the lessee's.
8. **Nothing records intent.** `scm new` creates a worktree, but no artifact says what files the
   task intends to touch, so nothing can be predicted before the first conflict. `status` reports
   ahead/behind and dirty, not overlap.
9. **The roadmap is a conflict magnet by policy.** `AGENTS.md`: "Tick the task's roadmap boxes in
   the same commit. If the plan conflicts on rebase, keep both sides." Every landing edits one
   shared file. That is S5 by design, and "keep both sides" is manual conflict resolution by an
   LLM on the hot path. [I: this is the highest-yield cheap fix; see 7.6.]
10. **The strong parts, worth keeping:** exact-tip gating; content-addressed candidates; the kernel
    releasing flocks when the holder dies (no lease-expiry logic needed on one host); GPU health
    selection; `GIT_OPTIONAL_LOCKS=0`; evidence being append-only and content-addressed; "guards
    are the contract" style tests. All are inputs to the recommended design, not things to replace.

### 2.2 vaso lead/follower relay (`docs/agents/cross-agent-collaboration.md`, `scripts/agents/relay.py`) [L]

Design: one agent (the follower) owns a long-running environment and the primary checkout; the lead
works in worktrees and maintains a landing branch the follower fast-forwards into the target at
turn boundaries. Messages are files in `.scratch/<effort>/`; the human relays only the first
message. `relay.py watch` polls and prints state-change lines (target moved, follower busy/idle,
tracker changed, tracked files deleted). `land` refuses on unexpected dirt.

The table of "failure modes seen in practice" is the single most valuable empirical artifact I
found. Reading it as evidence for a mechanism, not just for vaso:

| Observed | What it says about mechanism design |
|---|---|
| Lead waited for follower idle so it could land into follower's tree; turns ran for hours | Coordination that needs the *other* agent to reach a boundary has unbounded latency. The party with the turn boundary must be the one that acts. |
| Two agents edited the same JSON field; rebase conflicted | Shared files need an owner or an append-only discipline. Ownership by convention, discovered after the fact. |
| Rebase moved the landing ref to a commit known to need a follow-up; follower landed it within 30 s | Verified refs and work-in-progress refs must be different refs (vaso invariant 6). |
| Verify wrote into the checkout; `git add -A` committed 109,075 files | Gate execution must not have write access to the candidate. (Milano's read-only `/workspace/milano` candidate mount already prevents this. [L]) |
| Follower replayed a stale "delete these untracked files" instruction from its own context and deleted the ODR gate's checker | **Prose instructions in files do not update an agent's context.** Durable protocol must be enforced by tools, and changes delivered as messages at a turn boundary. Never one-shot destructive commands in handoffs. |
| Unquoted zsh variable silently no-op | Not coordination; a reminder that harness scripts should assert expected output. |
| 13 Bazel output bases exhausted `/tmp` inodes | Shared host resources need per-agent quotas and an owner, not just "be careful". |

Structural limits, in my judgment [I]:

- It is **inherently two-party.** Roles are named leader/follower and the protocol is symmetric only
  by re-assignment ("Lead handover"). Fan-out to 30 agents needs a tree of leads; the doc has no
  such notion.
- The **human is a message bus** for the bootstrap step, and a required decision-maker for landing
  authorization. That does not scale, but it does keep authority explicit, which is a real virtue.
- `watch` **polls** git state and processes. The busy probe is provider-specific (`systemd-inhibit`
  child for TRAE CLI). This is exactly what a runtime that owns the turn loop knows natively.

### 2.3 robust-training-observatory `AGENTS.md` Rules [L]

Relevant policy content is small but instructive. "We do not push to `mlsys/arnold`"; discoveries go
to `docs/research/` and code changes stay on local worktree feature branches: **authority is by
namespace** (product vs research vs experimental), not by locks. "Research notes from parallel
sessions may be untracked in `docs/research/`; leave them alone": **coexistence by an untouchable
rule on other agents' untracked files**. Never commit `.cache`, `out/`: generated state stays out of
the tree. Note the repo also fixes vocabulary (Insula, capsule, candidate, evidence). A coordination
runtime should reuse that vocabulary (gate receipts as evidence, candidate digest as identity) rather
than invent a parallel one.

### 2.4 corenius (worktree `corenius-collab-spike`) [L]

What exists that coordination can build on, and what does not:

- `core/runner.rs`: `Coordinator` is **session- and turn-scoped**: `start_user_turn` spawns a tokio
  task per turn with a fresh `turn_id`; `TurnRunner` emits `RuntimeEvent`s into a `TranscriptStore`.
  Consequence: **cross-agent state cannot live in `Coordinator`.** It needs a home outside any one
  session.
- `core/policy.rs`: `PolicyGate::evaluate` is a **stateless unit struct** returning
  `Allow | Ask(ApprovalRequest)` from tool capability flags (`read_only`, `writes`, `shell`) only.
  There is no `Deny`, no per-path scoping, and no access to any shared view. Claims would need a
  stateful gate.
- `core/types.rs`: `ToolUse {read_only, writes, shell}` is the only capability vocabulary. Tool calls
  carry `arguments: Value`, so path arguments are inspectable for the corenius-native tools.
- `supervisor.rs`: `ProcessSupervisor` snapshots a process subtree via `ps` and reports zombie or
  stopped children (`healthy`, `issues`). That is a liveness signal, and the same signal vaso's
  busy-probe reconstructs by hand.
- `provider/acp.rs`: spawns a provider CLI (`traecli` by default) as an ACP subprocess, sends
  prompts, streams updates, and captures supervision snapshots through a `SupervisorSink`. The
  provider owns its own tool loop; what is interposable depends on ACP delegating fs/terminal to the
  client [K: ACP defines client-side file and terminal methods, not verified here]. I did not
  determine how much of the provider's native file writing corenius can intercept. **Treat that as
  the largest unknown for enforcement**, see section 8.
- `insula.rs`: builds bubblewrap plans from a selection/marker file (`build_command`, `run`,
  `doctor`). I only skimmed signatures. Bind-mount granularity is the natural enforcement
  mechanism for write scopes.
- `docs/observability/durable-execution.md`: already commits to an **event-sourced, Temporal-shaped
  run model**: replayable decision layer from persisted events only; effects as activities with
  idempotency key `run_id + turn_id + step_id + activity_id + input_hash`; append-only trace with
  content-addressed `payload_ref`; a `TraceStore` next to `TranscriptStore`. The collaboration
  ledger below is deliberately the same shape, so the two can share storage, hashing, blob store,
  and replay tooling. It also gives the benchmark template (fresh child process per sample, one JSON
  report) reused in section 9.

Other checkouts consulted, bounded: `~/workspace/jj` (a jujutsu source checkout; `docs/conflicts.md`
and `docs/operation-log.md` read, consistent with the jj description in F3); `~/workspace/ultron`
(controller/policy repo: its invariants "every destructive action is backed by causal evidence or a
conservative safety rule" and typed evidence is the same discipline wanted here; not otherwise
about agent collaboration); `corenius/.scratch/research/temporal` is a Temporal Go checkout (used
only through the summary in `durable-execution.md`, not re-read). `milano.worktrees` was empty at
the time of reading, so no live worktree data was available; the simulator should therefore be fed
from git history rather than from live worktree state.

---

## 3. The mechanism families

Each entry: mechanism, what it guarantees, failure modes, cost, and fit for **(a)** 2-4 agents,
**(b)** 30+ agents, **(c)** GPU-scarce gates, **(d)** human keeps merge authority. Ratings are
`++ / + / o / - / --` and are my judgment [I], not measurements. Prior art is from memory [K]
unless marked.

### F1. Worktree-per-task (the milano baseline)

- **Mechanism.** Every task gets its own checkout and branch from the shared mainline. Agents never
  share a working tree. Integration is rebase then fast-forward.
- **Guarantees.** Removes working-tree collisions (I2) entirely. Linear history. Cheap; git-native.
- **Failure modes.** Says nothing about *semantic* or *textual* overlap between branches until
  integration time: conflicts are discovered late, at the moment of maximum sunk cost. Long-lived
  branches drift from mainline. Disk and inode use scale with worktrees. Does not address S2-S5 by
  itself.
- **Cost.** Near zero.
- **Fit.** (a) `++` (b) `o`: fine as a substrate, insufficient alone (c) `--` irrelevant (d) `+`
  neutral, humans can inspect branches.
- Provenance: [L] milano `scm.py`; git worktrees themselves [K].

### F2. Merge queues (GitHub merge queue, Bors, Zuul, Gerrit submit, and similar)

- **Mechanism.** Changes do not merge themselves. They enter a queue; a queue runner builds a
  candidate = trunk + changes ahead + this change, runs the gates on that candidate, and advances
  trunk only on success. Variants: serial (Bors style "never break master" rule [K]); batched (GitHub
  merge queue can group several PRs into one test run [K]; Kubernetes' Tide batches [K]);
  **speculative** (Zuul's gating pipelines optimistically assume changes ahead will pass and test
  in parallel on stacked candidates, re-queuing those behind a failure [K]); Gerrit submit rules
  and submit types such as rebase-always with required labels (Code-Review, Verified) [K].
- **Guarantees.** Trunk is never red *at the tested tip* (I1), independent of which agent forgot to
  rebase. Serial fairness. Batching and speculation raise throughput. A rejection carries a
  concrete cause.
- **Failure modes.** Throughput bounded by gate time unless batched or speculative. Speculation
  multiplies resource demand by the pipeline depth, and a flaky test invalidates everything behind
  it. Batching needs bisection when a batch fails (cost `O(log k)` gate runs per culprit [I]).
  Tests must be reliable, or the queue becomes a random rejector. The queue itself is a single
  point of failure and a policy hotspot.
- **Cost.** A queue service plus reliable gates. Milano's `integrate` already *is* a serial merge
  queue with a lock instead of a queue; the missing pieces are ordering visibility, batching, the
  lock not spanning GPU waits, and structured rejections.
- **Fit.** (a) `+` (b) `++` (c) `o`: speculation is expensive when a gate needs GPUs, so speculate
  only the CPU gates and serialize/batch the GPU ones (d) `++`: a queue is a natural place for
  human approval as a gate.
- Provenance: [K] all of it; milano `integrate` [L].

### F3. Stacked diffs and jujutsu-style change-ids

- **Mechanism.** A "change" has a stable id that survives rewriting; descendants are automatically
  rebased when a change is amended (jj's auto-rebase); conflicts are *recorded in the commit* as
  first-class data rather than blocking the rebase; an operation log records every repo-modifying
  operation so any state can be undone or restored. Gerrit's Change-Id and Phabricator-style stacks
  are the older pattern [K].
- **Guarantees.** [L, jj `docs/conflicts.md`, `docs/operation-log.md`] Rebase never "fails"; a
  conflicted commit can be further rebased, merged or backed out; resolution can be postponed;
  all repo operations are logged and undoable. For agents: no `rebase --continue` state machine, no
  half-rebased working tree (the vaso "tests on a half-finished rebase" failure), and undo for
  mistakes.
- **Failure modes.** Conflicted commits can be *pushed downstream* if nothing forbids it (needs a
  gate that rejects conflicted commits). Agent training data is dominated by git commands; jj
  commands may be used wrongly [I]. Git compatibility exists (colocated repos [K]) but the pre-push
  hook and worktree tooling in milano assume plain git. Change-ids need to be preserved through
  export to a git remote.
- **Cost.** A tool the agents must learn, or a wrapper that hides it. Low runtime cost.
- **Fit.** (a) `+` (b) `+`: auto-rebase of stacks helps most when agents produce dependent work
  (c) `--` irrelevant (d) `+`: change-ids make review of a moving change stable.
- **Take-away.** Adopt the *ideas* in the coordination layer even if we never adopt the tool: a
  stable **task/change id** independent of commit sha, **conflicts as recorded state** (not an
  aborted command), and an **operation log**, which the ledger in section 6 is.

### F4. Pijul / patch theory (also Darcs)

- **Mechanism.** History is a set of patches with explicit dependencies; patches that do not depend
  on each other commute, so merge is order-independent; conflicts are first-class objects that
  stay until a patch resolves them [K].
- **Guarantees.** Cherry-pick and reorder without rewriting identity; merge result independent of
  order (by construction of the category-theory model, [K] not verified).
- **Failure modes.** Small ecosystem, no integration with code.byted.org GitLab, CI, or our
  `git archive` candidate flow [I]. Commutation is syntactic (line-level); it does not know about
  semantic dependencies, so commuting two patches that must not commute (rename vs use) is still
  possible. Not an option for a repo that already ships git hooks and a git-based candidate digest.
- **Cost.** Total replacement of the VCS: prohibitive.
- **Fit.** (a) `-` (b) `o` (c) `--` (d) `o`.
- **Take-away.** The transferable idea is to **track patch-level dependencies**: "task B builds on
  task A's commit", so that landing B before A is rejected up front. The ledger records
  `depends_on` between tasks; git plus jj-style change-ids realize it.

### F5. CRDT / operational-transform live co-editing

- **Mechanism.** Multiple writers edit one document concurrently; character- or operation-level
  merge functions guarantee convergence without central coordination (CRDTs such as Automerge/Yjs;
  OT in Google-Docs style editors) [K].
- **Guarantees.** Strong eventual consistency of the *text*. No lost edits.
- **Failure modes.** Convergence is not correctness: two agents' interleaved edits converge to code
  that does not compile or that violates an invariant. Agent edits are coarse and turn-based, so the
  fine-grained conflict resolution CRDTs excel at solves a problem agents do not have, while
  hiding the one they do (semantic conflict) until later. Live buffers also fight the "gate the
  exact tip" invariant, because there is no stable tip. [I]
- **Cost.** A shared editing service and editor integration; large.
- **Fit.** (a) `-` (b) `-` (c) `--` (d) `-` (no clean commit boundary to review).
- **Take-away.** Only defensible use: *plans and tracker text* (notes, roadmap), where an
  append-only ledger with derived views is a simpler answer than a CRDT document. Recommended: do
  not adopt.

### F6. File/region leases and advisory locks

- **Mechanism.** An agent acquires a lease on a path, glob, directory or symbol before editing;
  others see it held, and wait, negotiate, or work elsewhere. Leases expire (TTL + heartbeat) so a
  dead holder does not block forever. Hierarchical variants use intention locks (IS/IX/X) so a lock
  on a directory conflicts appropriately with locks on its children [K, database lock manager
  literature]. Perforce-style exclusive checkout and Git LFS file locking are real-world instances
  [K].
- **Guarantees.** If enforced: no concurrent edits to a leased scope, so no textual conflicts there.
  If advisory: a conflict *signal* before work is wasted.
- **Failure modes.** Over-locking idles agents (lock the world, no parallelism); under-locking
  misses semantic coupling (interface in file A, callers in B). Deadlock when agents hold one scope
  and wait for another: needs an ordering rule or wait-die/wound-wait style timeouts [K]. Lease
  expiry vs a slow but live agent (an agent turn can last hours, per vaso [L]): expiry must be tied
  to *liveness of the process*, not a wall clock TTL, or extensions must be automatic. Stale
  holder after crash: needs fencing tokens so a zombie cannot act on an expired lease [K, standard
  distributed-locking lore].
- **Cost.** Low for a lock table; moderate to get scopes right and to enforce them.
- **Fit.** (a) `o`: for 2-4 agents, a shared convention beats a lock table (b) `+` for hot files and
  registries; `-` as a blanket policy (c) `++` for devices, which are naturally exclusive and
  countable (d) `+`.
- **Take-away.** Split the family in two: **resource leases** (GPUs, integration slot) must be hard,
  fenced and fair. **Path claims** should be *soft by default* (announce, detect overlap,
  negotiate) and hard only for a short list of hot or dangerous paths. [I; test in section 9, E4]

### F7. Ownership maps (CODEOWNERS-style path ownership per agent)

- **Mechanism.** A committed map assigns each path prefix to an owner (agent role or human). Writes
  outside your regions are proposals to the owner, or need the owner's approval. GitHub CODEOWNERS
  [K] and Chromium OWNERS-style files [K] are prior art; vaso's "give each shared file an owner and
  have the other agent append only" is an instance [L].
- **Guarantees.** Static, cheap, zero-negotiation disjointness for stable areas. Review routing.
- **Failure modes.** Stale maps; cross-cutting changes (rename an interface) touch every owner;
  owner agents become bottlenecks (30 agents all asking the `csrc/runtime` owner); ownership by
  *agent identity* is wrong because agents are ephemeral, so ownership should be by *role/area* and
  assigned per task. Encourages Conway's-law partitioning, which may or may not match the real
  dependency graph. [K/I]
- **Cost.** Low: a file plus a check in the gate.
- **Fit.** (a) `o` (b) `+`: the most scalable *static* mechanism (c) `--` (d) `++`: humans own the
  map and can be the owner of the dangerous areas.
- **Take-away.** Use ownership as the **default claim policy**: a task's initial footprint is its
  area; the map decides which areas are hard-claimed. It also supplies the hot-file list.

### F8. Lead/follower relay (vaso)

- **Mechanism.** One agent leads (specs, tickets, guards, landing branch, review), another follows
  (owns the environment, integrates at turn boundaries). Communication by files; human relays
  the first message. [L: `cross-agent-collaboration.md`, `relay.py`]
- **Guarantees.** Clear single authority per concern; verified landing ref (invariant 6); proven
  discard via manifest; human authority explicit.
- **Failure modes.** See 2.2. Two-party; polling; stale-context replays; the human as bus; lead can
  become a bottleneck for follower throughput; no place for a third agent.
- **Cost.** Low tooling, high protocol discipline. The doc itself is 200 lines of hard-won rules.
- **Fit.** (a) `+` (b) `--` as a flat pattern; `o` as a tree of leads (c) `o` (d) `++`.
- **Take-away.** Its *invariants* (1, 2, 6, 7) survive into the design. Its *transport* (files,
  polling, human relay) should be replaced by the ledger and turn-boundary message delivery.

### F9. Blackboard / tracker-as-message-bus

- **Mechanism.** Agents communicate by reading and writing a shared artifact (tracker files,
  issues, `.scratch/` notes) instead of direct messages. Classic blackboard architecture [K].
- **Guarantees.** Decoupling in time and identity; durable; human-readable; git-versioned if kept in
  the repo.
- **Failure modes.** No delivery guarantee (nobody has to read it); no notification, so it depends
  on polling; the shared file itself becomes a conflict hot spot (vaso's JSON `front` field [L]);
  context bloat as it grows; ambiguous authority (who may edit what). Vaso's stale-instruction
  incident shows it fails silently in exactly the case where a protocol changes. [L]
- **Cost.** None to start.
- **Fit.** (a) `+` (b) `-` (c) `--` (d) `+`.
- **Take-away.** Keep a human-readable *projection* of the ledger as markdown for people and
  agents to read, generated, never hand-edited. Do not use it as the bus. See also F13.

### F10. Actor/mailbox model with supervisor trees (Erlang/OTP, Akka, Temporal)

- **Mechanism.** Each participant is an actor with a mailbox; interaction is asynchronous message
  passing; supervisors restart failed children by declared strategies (one_for_one, one_for_all,
  restart intensity limits) [K, OTP]. Temporal adds durable workflows: deterministic orchestration
  code replayed from an event history; **signals** deliver messages into a running workflow;
  **activities** carry the side effects, with retries, heartbeats and idempotency; child workflows
  form a tree; sagas/compensation undo partial work [K]. Corenius already adopts the activity and
  idempotency-key parts. [L `durable-execution.md`]
- **Guarantees.** Failure isolation and structured restart; hierarchical delegation with clear
  responsibility; with durability, crash-resumable coordination and exactly-once *effects* (via
  idempotency keys) on top of at-least-once execution.
- **Failure modes.** Complexity: determinism constraints on the orchestration layer (no clock or RNG
  in decision code), event history growth, versioning of workflow logic over long-running runs.
  Mailboxes with no back-pressure. Supervisors that restart an LLM agent restart it with a *lost
  context*: "restart" is not a cheap operation for agents the way it is for Erlang processes, so
  the restart strategy is closer to "escalate to a lead or a human". [I]
- **Cost.** Moderate to high if built from scratch. Much of the substrate (event log, replay,
  activity idempotency, supervisor snapshots) is on corenius's roadmap already.
- **Fit.** (a) `o` overkill (b) `++`: hierarchy is the only structure that scales attention (c) `+`
  activities with heartbeats model long GPU jobs naturally (d) `+` an approval is a signal.
- **Take-away.** Model tasks as workflows over the ledger; model messages as signals delivered at
  turn boundaries; model gates as activities; model an agent's death as an event the parent
  decides on.

### F11. Contract-first parallelism (interface freeze plus guards)

- **Mechanism.** Before agents fan out, a lead fixes the interfaces between their areas (headers,
  schemas, function signatures, wire formats) and encodes them as executable guards (tests that fail
  on any agent's commit that violates them). Agents then work independently against the contract.
  Vaso's "guards are the contract" invariant [L]; consumer-driven contract tests and
  interface-first design in general [K]; milano's "missing operators must stay missing" negative
  tests [L] are the same idea applied to a registry.
- **Guarantees.** Converts semantic conflicts into red tests that land in the same gate as the
  code, not into arguments. Enables real parallelism where file-level disjointness is impossible.
- **Failure modes.** A wrong contract wastes all agents' work; contracts calcify; the lead becomes
  a single point of quality; contract amendment mid-flight needs a protocol (who must be told, which
  in-flight work becomes invalid); guards can have coverage holes (vaso's false green where shared
  guards never saw a capture's files [L]).
- **Cost.** Up-front design effort; guard authoring.
- **Fit.** (a) `+` (b) `++`: the mechanism that makes 30-way parallelism work on shared code
  (c) `o` (d) `++`: humans approve the contract.
- **Take-away.** Represent contracts as first-class ledger objects with **subscribers** (agents who
  declared a dependency); amending a contract emits an event that invalidates dependents' read-sets
  (see F16).

### F12. Speculative execution with commit/abort (STM, optimistic concurrency)

- **Mechanism.** Work proceeds without locks against a snapshot; at commit a validation step checks
  that nothing the transaction read or wrote has changed; on conflict abort and retry. Kung-Robinson
  optimistic concurrency control and software transactional memory [K]. Git push with
  compare-and-swap is already this at the granularity of the whole ref.
- **Guarantees.** No lock overhead in the conflict-free case; serializability when validation is
  read-set aware.
- **Failure modes.** High contention makes aborts dominate ("livelock" by repeated aborts); an
  abort of an agent's hours of work is far more expensive than an abort of a transaction; git's
  ff-check validates only *ancestry*, not the read set, so it misses semantic staleness.
- **Cost.** Low to build; cost is paid in wasted agent work.
- **Fit.** (a) `++` (b) `o` when disjointness is high, `--` when hot (c) `-` GPUs make abort
  expensive (d) `o`.
- **Take-away.** Two concrete ideas. (1) **Read-set validation** [I, my proposal, untested]: record
  which files an agent *read* during a task (from tool-call logs) and, when mainline advances,
  compute the intersection between files changed on mainline and the agent's read/write set. A
  non-empty intersection is a staleness signal delivered at the next turn boundary, before any
  textual conflict exists. (2) A **two-tier trunk** (`staging` then `mainline`), so optimistic
  landing is cheap while promotion remains deliberate (section 7.5).

### F13. Event-sourced coordination log as source of truth

- **Mechanism.** All coordination state (claims, leases, queue, messages, decisions, gate receipts)
  is derived by folding an append-only, ordered, content-addressed event log. Agents and the
  human act by appending events through validated commands; nothing edits state in place.
  Event sourcing and CQRS [K]; Temporal event histories [K/L]; jj's operation log [L]; milano's
  content-addressed evidence records [L `AGENTS.md`, `records.py`].
- **Guarantees.** Auditability (who decided what, on which evidence); replayability (rebuild any
  view; test the coordinator deterministically; simulate); one source of truth so projections
  cannot disagree; crash recovery by refolding; time-travel debugging of a coordination failure.
- **Failure modes.** Schema evolution over a long-lived log; log growth (needs snapshots and
  compaction); a single writer or a consensus story if it must be multi-host; folding must be
  deterministic; "audit trail" is only as strong as the writer's identity (milano's receipts are
  "not yet signed" [L]); over-engineering risk.
- **Cost.** Moderate; but corenius has committed to building the same substrate for run traces.
- **Fit.** (a) `o` overhead is mostly amortized if it is also the trace (b) `++` (c) `++` (d) `++`.
- **Take-away.** This is the **spine** of the recommended design. Every other family becomes a
  *policy over the log*, which is what makes them comparable and swappable in the simulator.

### F14. Resource schedulers for GPUs

- **Mechanism.** Slurm-like queues with priorities, gang allocation of N devices, backfill of small
  jobs into gaps, fair-share accounting, and preemption or checkpoint-kill; Kubernetes device
  plugins plus gang schedulers/queueing layers (Volcano/Kueue style) for the same problems [K].
  The minimal form is milano's per-device flock [L].
- **Guarantees.** Depending on policy: exclusive device use; bounded wait (FIFO with reservation);
  no starvation of large jobs; fairness across agents; utilization.
- **Failure modes.** Fairness policies are a source of surprise; preemption of a gate corrupts the
  gate's meaning, so only preempt dev runs; over-strict fairness reduces utilization; idle
  leases (agent holds GPUs while thinking) waste the scarcest thing. A full scheduler is heavy for
  one 8-GPU host. [I]
- **Cost.** Low for the four policy pieces on one host: (1) ticket queue with head-of-line
  reservation, (2) gang requests, (3) backfill with a bounded delay, (4) per-agent GPU-minute
  budgets. High if it is a real cluster scheduler.
- **Fit.** (a) `o` (b) `++` (c) `++` (d) `+` the human's `b200-release` should be a priority lane.
- **Take-away.** Keep the kernel-released flock as the *enforcement* primitive (it is right and
  needs no expiry logic on one host) and put a **fair queue in front of it**. Details in 6.3.

### F15. Review and verification roles (independent verifier agents, two-key merge)

- **Mechanism.** The agent that wrote the change is not the one that decides it is good. Options: a
  machine gate (tests, receipts); an independent verifier agent with fresh context and read-only
  access, given the diff and receipts; a human; and *two-key* rules (author + verifier, or verifier +
  human, both required). Gerrit's Code-Review/Verified split and 4-eyes principles [K]; Vaso's
  lead reviewing follower commits in `reviews.md` [L]; the milano rule that "green means the lead
  ran it on the exact tip" [L invariant 7].
- **Guarantees.** Catches spec misreads and unverified claims; separates "I say it passes" from "a
  receipt says it passed"; gives the human a summary they can trust more than the author's.
- **Failure modes.** Same-model verifiers share blind spots [I]; verifier cost in tokens and
  latency; verifiers rubber-stamp when given the author's narrative; unclear who breaks ties. The
  effectiveness is an empirical question we have not measured.
- **Cost.** Tokens plus one more agent slot per landing. Machine receipts are near free given
  milano's evidence layer.
- **Fit.** (a) `o` (b) `+` (c) `o` (d) `++`.
- **Take-away.** Machine-verified receipts are mandatory; a verifier agent is selective, applied to
  risky footprints (guards, release scripts, `.githooks`, `csrc/runtime`), and the human
  key is required on promotion to mainline in the default policy.

### F16. Conflict prediction (static and dynamic, before the conflict exists)

- **Mechanism.** Tiers, cheapest first: (1) declared-footprint overlap at task start; (2)
  historical co-change coupling mined from `git log` (files that change together); (3) symbol-level
  overlap using a parser or ctags-like index; (4) **dry-run merges** between all active branch tips
  and mainline on a timer, using `git merge-tree` to compute a merge without touching any working
  tree [K: `merge-tree --write-tree` exists in recent git; verify the exact flag on our version];
  (5) read-set staleness from F12; (6) semantic: run the fast CPU gate on the trial merge.
- **Guarantees.** None absolute. Turns "conflict at landing" into "warning N minutes before it
  matters", which is only valuable if the agent can act on it.
- **Failure modes.** False positives train agents to ignore warnings; semantic conflicts need tests;
  trial merges of 30 branches pairwise are `O(n^2)` (mitigate: only against mainline and only
  branches with overlapping footprints, [I]); warnings delivered mid-turn may not be acted on.
- **Cost.** Low for tiers 1, 4, 5; medium for tiers 3 and 6.
- **Fit.** (a) `+` (b) `++` (c) `-` (d) `+`.
- **Take-away.** Cheap enough to do continuously. The unknown is *lead time and precision*, which the
  simulator can estimate from real history (E5, E14).

### F17. Capability and permission models for agents

- **Mechanism.** An agent holds *capabilities*, not ambient authority: tokens naming exactly which
  coordination actions and resources it may use (claim scope, GPU count, land, message peers).
  Capabilities can be attenuated when delegated (a lead hands a sub-agent a subset), as in
  object-capability systems and macaroons-style caveats [K]. In corenius the hooks already exist:
  `ToolUse` flags and `PolicyGate` [L].
- **Guarantees.** Least privilege; a compromised or confused agent's blast radius is bounded by what
  it holds; delegation cannot widen authority; policy is a function of the ledger, so it is
  explainable.
- **Failure modes.** Permission prompt fatigue on the human; capabilities that are too coarse;
  enforcement gaps for tools the runtime cannot see (provider-native file writes); capability
  leakage through environment variables (milano's `MILANO_GPU_LEASE`, 2.1 point 7).
- **Cost.** Moderate; mostly in extending `PolicyGate`.
- **Fit.** (a) `o` (b) `++` (c) `+` (d) `++`.
- **Take-away.** Represent claims and leases as capabilities checked by a stateful policy gate, and
  make delegation (`parent_claim`) an attenuation.

### F18. Task decomposition for disjointness

- **Mechanism.** A planner (human or agent) partitions the goal into tasks with minimal footprint
  overlap: by module boundary, by dependency-graph cut, by "one op per file" registries, and by
  reserving hot shared files to a single *integrator* task. Footprints are declared up front and
  compared with observed diffs afterward. Conway's-law alignment, modular design [K].
- **Guarantees.** If the decomposition is good, nothing else in this document is needed. If it is
  bad, no coordination mechanism recovers the lost work.
- **Failure modes.** The planner's footprint estimates are wrong (especially for refactors); tasks
  that look independent share a hidden invariant; over-decomposition creates coordination cost that
  exceeds the parallel speedup (Amdahl-style ceiling from the serial landing and review steps [I]).
- **Cost.** Planner tokens plus attention; recurring.
- **Fit.** (a) `+` (b) `++`: the highest-leverage lever at scale (c) `o` (d) `++`.
- **Take-away.** Capture **declared vs observed footprint** for every task in the ledger. The gap is
  the planner's error signal and the only honest measure of whether decomposition is working.

### F19. Hedged / redundant execution (brief)

- **Mechanism.** Give the same task to two agents, land the first that passes. Tail-latency hedging
  [K].
- **Guarantees.** Lower latency and variance on uncertain tasks.
- **Failure modes.** Double token spend, double GPU demand, and a wasted branch to garbage-collect.
- **Fit.** (a) `o` (b) `-` (c) `--` (d) `o`. Listed for completeness; **not recommended** where GPUs
  are the constraint.

---

## 4. Comparison matrix

Legend: `++` strong, `+` good, `o` neutral/conditional, `-` weak, `--` unsuitable. "Effort" is
implementation effort in corenius, given what exists today. Columns (a)-(d) as defined in section 1.
Ratings are opinions [I].

| Family | (a) 2-4 | (b) 30+ | (c) GPU gates | (d) human authority | Protects | Effort | Main risk |
|---|---|---|---|---|---|---|---|
| F1 worktree-per-task | ++ | o | -- | + | I2 | none (exists) | late conflict discovery |
| F2 merge queue | + | ++ | o | ++ | I1, S2 | M | throughput vs GPU cost |
| F3 change-ids / jj ideas | + | + | -- | + | conflict handling | S as ideas, L as tool | agent familiarity |
| F4 Pijul / patch theory | - | o | -- | o | commutation | prohibitive | ecosystem |
| F5 CRDT / OT | - | - | -- | - | text convergence | L | convergence != correctness |
| F6 leases: resources | o | + | ++ | + | I3, S3 | S | starvation without fair queue |
| F6 leases: path claims | o | o (soft +, hard -) | -- | + | S1 | S-M | over-locking, deadlock |
| F7 ownership maps | o | + | -- | ++ | S1, S5 | S | stale map, bottleneck owners |
| F8 lead/follower | + | -- (o as tree) | o | ++ | S2, S4 | exists | two-party, human as bus |
| F9 blackboard | + | - | -- | + | S4 | none | silent non-delivery |
| F10 actors + supervision + durable workflows | o | ++ | + | + | crash safety, delegation | L | complexity, restart = lost context |
| F11 contract-first + guards | + | ++ | o | ++ | S1 semantic | M per effort | wrong contract |
| F12 optimistic + read-set validation | ++ | o / -- when hot | - | o | S1, staleness | S-M | abort waste |
| F13 event-sourced ledger | o | ++ | ++ | ++ | I4, replay | M (shared with traces) | schema evolution, scope creep |
| F14 GPU scheduling policies | o | ++ | ++ | + | I3, L1 | S-M | fairness surprises |
| F15 verifier roles / two-key | o | + | o | ++ | quality | M (tokens) | correlated blind spots |
| F16 conflict prediction | + | ++ | - | + | S1 early | S-M | false positives |
| F17 capabilities / permissions | o | ++ | + | ++ | blast radius | M | enforcement gaps |
| F18 decomposition | + | ++ | o | ++ | everything | recurring | wrong estimates |
| F19 hedged execution | o | - | -- | o | latency | S | cost |

Reading the matrix: no single family scores well across the row. The best generalists (F13, F2,
F14, F18) are *infrastructure or discipline*, and the best specialists (F6 resources, F16, F12) plug
into them. That is why the candidates below are composites.

---

## 5. Candidate composite designs

Each composite says what it is, what it costs, and where it breaks. All assume git remains the
code substrate. (Indices refer to the families above.)

### C1. Hardened baseline (status quo plus repairs)

F1 + F2-lite + F6 resources, no new runtime.
- Changes: release the integration lock before GPU gates (queue first, gate later); print a queue
  and ordering; return structured rebase conflicts (files, other branches touching them);
  ticketed GPU queue with head-of-line reservation; fix `.holder` writes; per-task roadmap
  fragments and `merge=union` for append-only files [K git attribute; verify].
- Cost: days. Fits (a) and mostly (c).
- Breaks: no claims/prediction, no hierarchy, still a human as message bus. Good for 2-6 agents.
  Everything is bash/Python and local to milano; it does not help other repos.

### C2. Ledger plus landing queue plus GPU broker (recommended core)

F13 spine + F2 + F14 + F6 (soft claims) + F16 (cheap tiers) + F17 (stateful policy) + F1 substrate.
- All coordination as events in an append-only ledger owned by corenius; a queue runner batches and
  speculates CPU gates, serializes GPU gates through the broker; soft path claims with conflict
  prediction; permissioned coordination tools; two-tier trunk.
- Cost: medium. Fits (a) with low overhead if defaults are permissive, (b) if paired with C4 for
  decomposition, (c) strongly, (d) strongly.
- Breaks: needs enforcement to be more than advice (see risk R1); single-host authority for the
  flock layer; ledger schema design must be right early.

### C3. Ownership and contract-first (organizational, minimal runtime)

F7 + F11 + F18 + F1, with F9 as the bus.
- A planner writes a CODEOWNERS-like area map and a contract freeze with guards; each agent is
  assigned an area; landing goes through the existing `integrate`.
- Cost: mostly human/planner effort; almost no runtime. Scales further than C1 for cheap, because
  it attacks the cause of conflicts.
- Breaks: needs excellent up-front planning; a wrong contract is expensive; no help for GPU
  contention; silent-failure blackboard problems reappear at 30 agents.

### C4. Hierarchical workflows (Temporal-shaped tree of leads)

F10 + F13 + F18 + F17 + F8-as-tree.
- A root lead decomposes; sub-leads own areas and delegate attenuated capabilities; children
  report by signals; parents decide on child failure; the ledger is each workflow's history.
- Cost: large; it is C2 plus a decision layer. Only worth building after C2 shows that flat
  coordination is the bottleneck. Fits (b) best.
- Breaks: orchestration agents are themselves LLMs with context limits; a lead's misjudgment
  is amplified; more tokens on coordination than on coding at small scale.

### C5. Optimistic staging train (throughput first)

F12 + F2 (batched) + two-tier trunk + F15 (selective verification).
- Agents land into `staging` quickly after cheap CPU gates. GPU gates run periodically on the
  staging tip in batches. Promotion of a green staging tip to `mainline` is a human decision (or a
  policy for low-risk footprints). A red batch is bisected, and the culprit is reverted from
  staging, not from mainline.
- Cost: medium; it is a policy in C2 rather than a separate system.
- Breaks: `staging` is sometimes red at its tip between GPU gates, so agents that base work on it
  inherit failures; mitigation is basing new work on the last *promoted* or *green* commit. Violates
  vaso invariant 6 unless `staging` is explicitly a different ref from the "landing ref that only
  points at verified commits". Risky for a hardware-qualified repo unless promotion requires the
  full receipts.

### C6. jj-native workspace model

F3 + F2 + F13-lite.
- Each agent uses a jj workspace; changes have stable ids; auto-rebase keeps stacks fresh; conflicts
  are recorded, and a gate rejects a conflicted change from landing. The operation log replaces
  part of the ledger for VCS events.
- Cost: medium plus training of agents and hooks; high uncertainty about agent fluency [I].
- Breaks: milano's hooks and `--rev` launcher assume git; two logs (jj's and ours) to reconcile;
  does not address GPU or claims.

**Which composites are compatible.** C2 is a base. C5 is a policy setting of C2. C4 builds on C2.
C3 is orthogonal and cheap and should happen regardless. C1's repairs are a subset of C2's first
phase and can ship immediately without waiting. C6 is an independent client-side experiment, and I
would run it only as an optional adapter after C2, since the ledger's `change_id` abstraction is
what makes jj pluggable rather than required.

---

## 6. Recommended architecture: a coordination runtime hosted in corenius

Recommendation: **build C2, ship in the order C1 repairs, then C2 phases, keep C3 as standing
practice, decide about C4 and C5 from simulator and pilot evidence.** Keep git as the substrate; keep
milano's Insula, exact-tip gating and flock as enforcement primitives; add a ledger and a policy
layer above them. Name used below: `coord` (module `src/coord/`, CLI `corenius coord ...`).

### 6.1 Placement in corenius and why

Constraints from the code [L]:

1. `Coordinator`/`TurnRunner` are per-session and per-turn, so coordination state lives **outside**
   them, in an on-disk ledger keyed to a coordination root
   (`$CORENIUS_COORD_ROOT`, default under the same cache-root convention as milano, so leases and
   evidence continue to be shared across sandboxes).
2. Turn-scoped processes are short-lived. So the **ledger is operated daemonless by default**: each
   `coord` command takes an exclusive `flock` on the ledger head, folds the current state (from the
   latest snapshot plus tail), validates, appends, releases. This matches milano's idiom and is
   crash-safe because the kernel drops the lock. A separate, restartable **queue runner** process
   advances the landing queue and expires leases; it is a pure function of the ledger plus
   idempotent effects, so it can crash anywhere.
3. `PolicyGate` gets a **ledger view** and a `Deny` variant; coordination and edit tools declare
   the resources they need (extend `ToolUse` with optional `scope`/`resource` fields, and add
   `coordination: bool`).
4. Turn boundaries are corenius's to own. `Courier` injects pending inbox messages and
   staleness notices at **turn start**, as a system-attributed section of the prompt. This directly
   fixes the vaso failure of protocol changes not reaching a running thread. [L, I]
5. Liveness comes from `ProcessSupervisor` snapshots (already taken in the ACP path), not a
   provider-specific busy probe. A lease's heartbeat is "supervisor sees the agent's process
   subtree healthy", and a `stall` event is emitted on `healthy=false` or lack of progress.
6. Enforcement layering for writes (prevent, detect, repair), because I do not know how much of
   a provider's native writing corenius can intercept:
   - **Prevent:** run the provider in an Insula plan where the worktree is bound read-only except
     for claimed paths (bubblewrap bind granularity: directory or file). Strongest, coarsest.
   - **Detect:** at every turn end, diff the worktree against `base_rev`; paths outside the
     footprint emit `footprint.violation` and open a claim request the human/lead or the policy
     resolves.
   - **Repair:** violations either widen the claim (soft claims) or fail the landing request
     (hard claims).

Components:

| Component | Role | Corenius home |
|---|---|---|
| **Ledger** | append-only, hash-chained event log + reducer | `src/coord/ledger.rs`, shares blob store and hashing with `TraceStore` |
| **Broker** | claims, GPU/integration leases, fair queue, budgets | `src/coord/broker.rs` |
| **Lander** | landing queue runner, gate execution in the Insula, receipts | `src/coord/lander.rs`, calls the existing Insula launcher |
| **Scout** | conflict prediction: overlap, `merge-tree` probes, read-set staleness | `src/coord/scout.rs` |
| **Courier** | inbox and notices delivered at turn start; outbox from tools | integrates with `TurnRunner` |
| **Board** | TUI/markdown projection for the human; the only human write path is decisions | `tui.rs` view + generated `board.md` |
| **Tools** | `coord.claim`, `coord.release`, `coord.gpu` (run under a lease), `coord.land`, `coord.status`, `coord.send`, `coord.inbox`, `coord.handoff` | `src/tools.rs`, registered behind capabilities |

### 6.2 Data model

Design rules: all times are ledger-assigned (`seq` is the order; wall time is metadata);
everything referenced by content hash where possible; commands validate against the folded state and
produce events; state is never edited.

**Event envelope** (one JSON object per line; large payloads by `payload_ref`, same convention as
the trace design [L]):

```jsonc
{
  "schema_version": 1,
  "seq": 18234,                       // dense, assigned under the ledger lock
  "ts": "2026-09-29T10:14:03.221Z",   // metadata only; never used by the reducer
  "prev": "sha256:...",               // hash of previous event (chain)
  "id": "sha256:...",                 // hash of this envelope minus id
  "actor": { "kind": "agent|human|runner|broker", "id": "claude-3", "session": "...", "role": "worker|lead|verifier" },
  "subject": { "task": "t-0042", "run": "...", "turn": "..." },  // correlation with RuntimeEvent/turn ids
  "kind": "claim.granted",
  "idempotency_key": "run+turn+step+activity+input_hash",       // same key convention as durable-execution.md
  "payload": { },
  "payload_ref": null,                // blob address for large data (gate logs, diffs)
  "cap": "cap-7f3a"                   // capability that authorized it (for audit)
}
```

**Event kinds** (closed set, versioned):

- Tasks: `task.created`, `task.decomposed{children}`, `task.assigned`, `task.state{from,to,why}`,
  `task.footprint.declared`, `task.footprint.observed`, `task.depends{on}`, `task.abandoned`.
- Claims: `claim.requested`, `claim.granted{epoch}`, `claim.denied{conflicts_with}`,
  `claim.widened`, `claim.released`, `claim.expired`, `footprint.violation`.
- Leases: `lease.requested`, `lease.queued{position}`, `lease.granted{epoch, devices}`,
  `lease.heartbeat`, `lease.released`, `lease.expired`, `lease.preempted`, `lease.idle_warned`.
- Landing: `land.enqueued`, `land.batch_formed{members}`, `land.rebased{tip, tree}`,
  `land.gate_started{gate, epoch}`, `land.gate_finished{gate, receipt}`, `land.bisect{culprit}`,
  `land.staged{sha}`, `land.rejected{reason}`, `land.promoted{sha, by}`.
- Prediction: `conflict.predicted{tasks, files, tier}`, `stale.read{task, files, since}`.
- Messages: `msg.sent{to, body_ref, kind}`, `msg.delivered{turn}`, `msg.acked`.
- Contracts: `contract.registered{id, guard}`, `contract.amended`, `contract.subscribed`.
- Verification: `verify.requested`, `verify.attested{verdict, evidence}`.
- Human: `human.decision{on, decision, note}`, `human.override{action}`.
- Runtime: `agent.spawned`, `agent.stalled`, `agent.exited{class}`, `policy.denied{tool, why}`.

**Task**

```jsonc
{ "id": "t-0042", "parent": "t-0040", "goal_ref": "blob:...",
  "base_rev": "<sha>", "branch": "claude/m4-orchestrator", "worktree": "...",
  "state": "working",
  "footprint": { "declared": ["torch_accel/csrc/ops/add.*"], "observed": ["..."], "reads": ["..."] },
  "depends_on": ["t-0039"], "contracts": ["c-abi-v2"], "priority": "normal",
  "owner": "claude-3", "budget": { "gpu_minutes": 60, "tokens": 2000000 } }
```

**Claim** (a capability; delegation is attenuation)

```jsonc
{ "id": "cl-91", "task": "t-0042", "scope": { "kind": "path|dir|symbol", "pattern": "torch_accel/csrc/ops/**" },
  "mode": "write|append|read-contract|intent",       // intent = hierarchical intention lock on a directory
  "strength": "soft|hard",                           // hard only for paths on the hot list
  "epoch": 311,                                      // fencing token, increments per grant on this scope
  "parent": "cl-70",                                 // child scope must be a subset of the parent's
  "granted_at_seq": 18201, "ttl": "liveness" }       // expiry is tied to agent liveness, not wall time
```

Compatibility: `write` conflicts with `write`/`append` on overlapping scope; `append` is compatible
with `append` (registries, roadmap fragments, using a union-merge driver); `read-contract` is
shared and only creates a subscription; `intent` on a directory conflicts with `write` on it but
not with `write` on a child claimed under it.

**Lease** (resource; enforced by flock, ordered by the broker)

```jsonc
{ "resource": "gpu", "request": { "count": 2, "class": "gate|dev|release", "min_healthy": true },
  "holder": { "task": "t-0042", "run": "...", "pid_tree_root": 12345 },
  "epoch": 88, "devices": [3,5], "ticket": 4412, "priority": 2,
  "preemptible": false, "max_duration": "45m", "idle_policy": "warn@10m,reclaim@30m" }
```

`resource` also covers `integration` (the landing slot) and `cache-gc` (mutual exclusion for
destructive maintenance). The broker grants tickets in policy order; the *holder* then takes the
real per-device flock, so the kernel is still the arbiter of "alive". The epoch is written into
receipts so a zombie holder's receipt is rejected.

**Landing request and receipt**

```jsonc
{ "id": "lr-207", "task": "t-0042", "head": "<sha>", "base": "<sha>",
  "gates": [ {"name":"repo","cost":"cpu"}, {"name":"runtime-test","cost":"gpu:1"},
             {"name":"accel-nccl --world-size 2 --collectives-only --ddp","cost":"gpu:2"} ],
  "policy": {"promote": "human"}, "state": "queued", "batch": "b-33" }
// Receipt: written by the Lander (never by the agent), content-addressed:
{ "gate": "runtime-test", "tip": "<sha>", "tree": "<tree sha>", "capsule_digest": "...",
  "lease_epoch": 88, "result": "pass|fail|infra", "evidence_digest": "sha256:...", "log_ref": "blob:..." }
```

Receipts are keyed by `(tree, gate, capsule_digest)`. A rebase that leaves the tree identical (or a
batch member whose merge result was already gated) reuses the receipt, which is the same
argument milano's build cache makes for native inputs. [I: unmeasured]

### 6.3 Protocol state machines

**Task lifecycle**

```
proposed -> claimed -> working -> ready -> queued -> gating -> staged -> promoted -> finished
    |          |          |         |        |          |         |
    |          |          +-> blocked(claim|lease|contract|dependency) -> working
    |          +-> denied(hard claim conflict) -> proposed
    |                     working -> stale(read-set overlap | base moved)  -> working (after rebase)
                          queued/gating -> rejected(conflict | gate_fail | culprit) -> working
    any -> abandoned (agent exit without handoff) -> [lead|human decides: reassign | close]
```

Transition rules:
- `working -> ready`: only through `coord.land`, which requires clean worktree, no
  `footprint.violation` unresolved, and `depends_on` all `staged` or later.
- `queued -> gating`: only by the Lander, only holding the lease epochs it needs.
- `gating -> staged`: only if every gate has a `pass` receipt whose `tree` equals the tested
  tree, and the batch base equals current `staging` tip (else re-form the batch; receipts are reused
  when the tree is identical).
- `staged -> promoted`: fast-forward of `mainline` to a staged sha; requires a `human.decision` unless
  the task's footprint is on the auto-promote list AND all release-tier receipts exist.
- A `rejected` landing always carries a structured reason, delivered to the agent at its next turn.

**Lease lifecycle**

```
requested -> queued -> granted -> (heartbeat)* -> released
                          |                    -> expired   (holder dead per supervisor + flock free)
                          |                    -> preempted (only class=dev, cooperative SIGTERM + grace)
                          +-> revoked by human.override
```

Head-of-line rule (the fix for milano's starvation): the broker orders requests by
`(priority class, ticket)`. The head request **reserves** devices as they become free, for a bounded
reservation time, while small requests are **backfilled** only onto devices that will not delay the
head beyond its estimated wait. Release-class requests (8 GPU) therefore drain the host instead of
waiting for a lucky instant. Idle detection (lease held, near-zero GPU utilization, agent process
idle) triggers `lease.idle_warned`, then reclaim. Fair-share budgets (`gpu_minutes` per task/agent)
are an option, not a default.

**Landing protocol (per batch)**

1. `land.enqueued` (agent-invoked, capability-checked).
2. The Lander forms a batch from queue head: up to `k` compatible requests (disjoint footprints and
   no dependency edges between them are compatible; `k` adaptive from recent failure rate).
3. Build the speculative tip: `staging` + batch members in order, by rebasing (never a merge commit;
   the linear-history rule is preserved [L]). A conflict rejects that member alone with hunks and the
   conflicting task ids.
4. Run CPU gates (`repo`) for the whole batch in parallel with others (no lease needed).
5. Request the GPU gates' leases as one gang, in the priority class `gate`. Run them on the batch tip
   in the Insula against the exact tree.
6. Pass: `land.staged`, fast-forward `staging` with compare-and-swap on the ref (the actual
   linearization point, same as milano's push). Fail: bisect (split the batch, reuse receipts of
   halves already gated), reject the culprit, re-queue the rest.
7. Promotion to `mainline` is separate (section 7.5).

**Human authority points** (explicit events, never implicit): `promote`, `reorder`, `veto`,
`release-lane` priority, `force-release lease`, `assign/reassign task`, `contract amend approve`,
`widen hard claim`. The Board shows what is waiting on a human, oldest first, with the receipts
attached. Everything else runs without asking.

### 6.4 Why these choices (short)

- **Ledger over files** because vaso's failures (stale instructions, tracker conflicts, unread
  notes) are all consequences of using mutable, undelivered, unowned text as the bus. [L, I]
- **Flock kept, queue added** because flock's kernel-level death handling is exactly right, and its
  only defect (no ordering, no drain) is a policy defect fixable in front of it.
- **Soft claims by default** because false blocking makes agents idle, which at 30 agents is the
  most expensive failure. Hard claims on a short list because some conflicts (release scripts,
  guards, `.githooks`, target TOML, Spack lock) are costly enough to prevent. [I, to be measured]
- **Landing at batch granularity** because a strict serial queue caps throughput at
  `1/gate_time`, and a speculative-parallel queue multiplies GPU demand. Batching amortizes a
  scarce-resource gate across `k` changes; its price is bisection on failure. [I, modeled in E2]
- **Two-tier trunk** because it makes *agent landing* cheap and *human promotion* deliberate, which
  is the only way I see to reconcile 30 agents with a human who keeps merge authority.
- **Receipts written by the Lander, not the agent** because "green means the lead ran it on the exact
  tip" [L] is the invariant, and it is only reliably true when the party claiming green is not the
  party who benefits.

### 6.5 Migration from what exists

| Phase | Deliverable | Risk | Value |
|---|---|---|---|
| 0 | No code. Roadmap boxes as per-task fragments; union merge for append-only files; per-agent I/O roots and quotas (vaso relay's rule) | none | removes the known conflict magnet |
| 1 | Ledger library + `corenius coord emit`; wrap `scm.py` and `gpu_lease.sh` to emit events (read-only mirror); Board view | low | observability, real data for the simulator |
| 2 | Broker: ticketed fair GPU queue in front of flock; correct `.holder`; drain for release lane; integration lock released before gates (queue first) | medium | fixes L1 |
| 3 | Lander: batched landing, receipts by tree, structured rejections, `staging` ref | medium | throughput, feedback |
| 4 | Claims + Scout + stateful PolicyGate + Courier | medium-high | prevention and early warning |
| 5 | Roles, delegation, hierarchical tasks, verifier agents | high | scale beyond ~10 agents (unproven) |

Phases 0-2 are worthwhile even if everything after is dropped.

---

## 7. Positions and why, on the contested points

### 7.1 Should agents be told about each other at all?

Yes, but through structured notices and only what changes their next action: a claim overlap, a
landed commit touching their read-set, a rejected batch they were in, a contract amendment. Free-form
peer chat between workers is a cost with no invariant behind it. Route worker-to-worker
negotiation through a lead or the ledger's `msg.*` with a size cap. [I]

### 7.2 Enforced or advisory claims?

Both, keyed by path. Advisory (soft) for the long tail, enforced (hard) for the hot list. Detection
at turn end covers whatever prevention misses. The expected weakness: agents will treat soft warnings
as noise; E4 and E5 exist to find out.

### 7.3 Single host or multi-host?

Everything above works on one host, which is today's reality (milano is pinned to `n116-077-207`
[L]). If multi-host is needed: the **remote ref CAS** stays the linearization point for landing; the
ledger can move to a git ref (append-only commits under `refs/coord/ledger`, advanced by
compare-and-swap) [I, untested], or to a small server; per-device flock cannot span hosts and must
become broker-granted leases with liveness via heartbeat plus fencing epochs. Design the `Ledger`
behind a trait so this is a swap, not a rewrite.

### 7.4 Where jj fits

As an optional client adapter after Phase 3, not a dependency. The ledger's task id and
`change_id` field are what make it possible. The single most useful borrowed behavior, "conflicts as
recorded state, never an aborted rebase", can be approximated in git by having the Lander record the
conflicting `merge-tree` result as a blob and hand it to the agent.

### 7.5 Human merge authority, concretely

`mainline` moves only by `land.promoted`, which requires a `human.decision` in the default policy.
Agents land into `staging`. Auto-promotion is a per-footprint policy that the human sets, and each
auto-promotion emits the same event with `by: policy:<id>` so it is auditable. This differs from
milano today, where `integrate` pushes mainline directly [L], and from vaso, where the follower
lands after the lead's verification and the human authorizes landing [L]. It is the smallest change
that keeps the human decisive at 30 agents, provided the Board makes each decision a one-glance
review.

### 7.6 The roadmap file

Regardless of anything else: a shared plan file that every landing must edit guarantees serial
conflicts. Replace with `docs/roadmap/<task>.md` fragments (or ledger-derived status), aggregated
by a generated view. [I; highest ratio of benefit to effort in this document.]

---

## 8. Open questions and risks

| # | Risk / question | Why it matters | Mitigation or way to find out |
|---|---|---|---|
| R1 | **Enforcement reach over provider-native tools** (ACP CLIs write files themselves) | claims are advice if they cannot be enforced | read `acp.rs` and the ACP client-side fs/terminal methods; prototype the Insula read-only-except-claimed plan; measure detect-at-turn-end delay |
| R2 | Flat file/path claims miss semantic coupling | main source of "clean rebase, red build" | contracts+guards (F11), read-set staleness, CPU gate on trial merges; measure escape rate |
| R3 | Batching hurts under flaky gates | flaky GPU tests turn batching into random rejection | receipts distinguish `fail` vs `infra`; quarantine/rerun policy; E2 with injected flakiness |
| R4 | Human attention is the true bottleneck at 30+ agents | promotion queue grows unboundedly | measure approvals/hour and time-in-queue (E12); auto-promotion policy; batch review UI |
| R5 | Stale agent context (vaso's replay incident) | protocol changes do not propagate | Courier at turn start; tools that verify state before destructive action; pilot test (E11) |
| R6 | Ledger schema and semantics ossify early | the log is long-lived | version every event; keep the reducer pure; migration test using the `trace-schema-migration` benchmark pattern [L] |
| R7 | Restart of an LLM agent loses context | Erlang-style restart is not cheap | prefer handoff records (`coord.handoff`) and escalate to the lead/human; measure resume cost |
| R8 | Verifier agents may not add signal | token cost without benefit | E8 with seeded defects and correlated-model comparison |
| R9 | GPU idle-lease reclaim false positives | kills a long compile or an agent thinking | warn-then-reclaim with generous thresholds; only reclaim if the process subtree is idle too |
| R10 | Dependence on git internals (`merge-tree`, union merge, ref CAS semantics) | version differences | pin behavior with tests against the repo's git version [K, unverified here] |
| R11 | Hierarchy amplifies bad decomposition | whole subtree wasted | require declared footprints and reviewed decomposition for high fan-out; record declared vs observed |
| R12 | The ledger becomes the new shared hot file | single flock contention at high event rates | measure append latency and lock hold time (E10); segment the log per subject with a merged index if needed |
| R13 | Scope creep: this becomes a workflow engine | large surface for little marginal value | phase gates in 6.5; keep C4 conditional on evidence |
| R14 | Receipts are not signed (milano says so [L]) | a compromised agent could forge a receipt | Lander is the only receipt writer, running outside the agent's Insula; hash-chain anchoring into git notes/trailers; signing is a separate item |
| R15 | Simulator realism | conclusions only as good as the workload model | drive it from mined git history (E14) and from Phase 1 live event logs, not from synthetic Poisson alone |

Things I explicitly do not know: how often milano agents actually conflict (never measured);
distribution of gate durations and GPU wait times (the `.holder` and evidence data exist but were
not analyzed in this spike); how well the current ACP provider surfaces file edits; whether agents
follow soft warnings.

---

## 9. Proposed experiments

Home: the sibling agent's discrete-event, git-backed simulator under `experiments/collab/`. The
simulator should model: agents as state machines with turn boundaries and per-turn work; a real git
repo (so rebases and `merge-tree` conflicts are real, not probabilistic); a GPU pool with gang
requests; gate durations from distributions; and the ledger as a real reducer, so the same code
runs in simulation and in the runtime (I4/I5 hold by construction).

The first block is **not** simulator work: it is cheap data mining that anchors the simulator's
parameters and should happen first.

| ID | Uncertainty | Experiment | Metric | Decision it drives |
|---|---|---|---|---|
| E14 | How often do concurrent tasks really overlap? | Mine `git log` of milano, vaso and corenius for branches that were concurrent; compute footprint overlap, hot-file share (roadmap, `Cargo.lock`, registries), and conflict frequency using `git merge-tree` replay of actual concurrent pairs | overlap rate, conflicts per landing, top-N hot files | sets simulator workload; how much C3/roadmap fix is worth |
| E0 | Gate and GPU-wait distributions | Parse milano evidence and `.holder`/lease logs (or add Phase 1 instrumentation) | gate durations by mode, lease wait, utilization | parameters for E2 and E3 |
| E1 | Does declared footprint predict observed footprint? | Simulator with a noisy planner; live pilot: log declared vs observed on real tasks | precision/recall of declared footprint | whether soft claims can be trusted, F18 value |
| E2 | Landing throughput and GPU cost: serial vs batched vs speculative | Simulate the four landing policies (milano lock, FIFO serial, batch size k, Zuul-like speculation) under GPU cap 8 and gate mixes (1,2,8 GPUs), failure probability p, flake rate f | landings/hour, agent idle time, GPU-hours wasted per landing, red-tip time, bisection cost | batching factor `k`, whether speculation is ever worth it |
| E3 | GPU broker policy | Compare release-and-retry (milano), FIFO ticket, FIFO plus reservation, backfill, fair-share budgets | p50/p99 wait, utilization, starvation events (L1) for the 8-GPU release request | ships the fix for 2.1 point 5; proves it |
| E4 | Soft vs hard claims | Vary claim strictness (none, soft, hard-all, hard-hot) and hot-list size | conflicts at landing, agent blocked time, throughput | default strictness |
| E5 | Conflict prediction value | Replay E14 history with each predictor tier (declared overlap, co-change, symbol, `merge-tree` probe, read-set staleness); vary probe interval | precision, recall, lead time between warning and conflict, false-alarm rate per agent-hour | which Scout tiers to build |
| E6 | Message delivery cadence | Turn-boundary only vs mid-turn interrupt vs periodic poll; agents have turn lengths from minutes to hours | wasted work after a relevant event, time-to-acknowledge, interruptions per turn | Courier design; validates the "turn start" choice |
| E7 | Flat vs hierarchical at 30 agents | Simulate flat, tree with fan-out 4/6/8, and lead-per-area; lead throughput as a limited resource | landings/hour, lead queue length, orphaned work, rework rate | whether C4 is needed, span of control |
| E8 | Do verifier agents add signal? | **Live**, not simulated: seed known defects into diffs (mutation-style); compare gate-only, verifier-agent, verifier with author's narrative hidden, human | detection rate, false rejects, tokens per detection, correlation with author model | scope of F15 |
| E9 | Safety and recovery under failure | Simulator fault injection: kill agents, queue runner and broker at random ledger positions; kill during flock hold; clock skew | invariants I1-I4 and L1 hold; lost or duplicate landings; orphaned leases; time to reclaim | correctness gate for the runtime; run as a property test in CI |
| E10 | Ledger overhead | Reuse the `cold-open-trace` / `resume-agent` child-process benchmark shape [L]: synthetic logs of 30 agents x days | append latency, lock hold time, fold time, snapshot size, log growth | snapshot policy; single flock vs sharded log |
| E11 | Stale-instruction replay | **Live pilot** (small): change a protocol mid-run; compare instructions in files vs turn-start injection vs tool-enforced check | fraction of agents that violate the new protocol; time to comply | validates the Courier and enforcement-by-tool |
| E12 | Human attention | Simulate promotion policies (all human, per-footprint auto, batch review) with a human service-time model; live pilot on one week of real promotions | promotion queue length and age, approvals/hour, auto-promote share, post-promotion regressions | default policy for promotion; Board design |
| E13 | Idle-lease reclaim | Replay lease-holder utilization traces (Phase 1 data) under thresholds | wasted GPU-minutes, false reclaim rate | R9 thresholds |
| E15 | Decomposition quality | Take real multi-task efforts, decompose by planner, compute achievable parallel speedup vs serial-landing ceiling | overlap rate, critical-path length, speedup | limit on useful N, decomposition guidance |
| E16 | Enforcement reach (R1) | Live: run the ACP provider under the read-only-except-claimed Insula plan and under detect-only | fraction of writes blocked/detected; detection delay; agent failure rate from denied writes | whether `Prevent` is viable for ACP providers |

Suggested simulator invariants to assert on every run (property-test style): (I1) `staging` and
`mainline` only ever point at commits with pass receipts for the required gates on the identical
tree; (I2) no two agents write to one worktree; (I3) at most one live holder per device and epochs
strictly increase; (I3b) a receipt with a stale epoch is never accepted; (I4/I5) `fold(log)` equals
the live state and replay is deterministic; (L1) under fair policies every request is served within a
bound in the absence of failures; and "no landing with an unresolved `footprint.violation`".

Order of work, if only some can be done: E14 and E0 (mining, days), then E3 and E9 on the simulator
(they validate the phase 2 broker and the safety story), then E2, then E5, E4 and E12. E8, E11 and
E16 need live agents, and E16 blocks any commitment to enforcement, so schedule it early in the
Phase 1-2 window.

---

## 10. Summary of stance

1. The unit of design is the **ledger**; every other mechanism is a policy over it.
2. Fix the real, verified defects first, even before any new runtime: GPU starvation of large
   requests, lock held across gates, opaque conflicts, and the shared roadmap file.
3. Use **soft claims + early prediction** for text, **hard fenced leases** for devices, **contracts
   and guards** for semantics, and **decomposition discipline** as the lever that dominates at scale.
4. Land into `staging` cheaply, in gated batches; **promote to mainline by human decision** unless
   policy says otherwise.
5. Move protocol from prose to **tools and turn-start messages**, since the observed failures were
   all context-staleness or ownership failures rather than git failures.
6. Decide on hierarchy, verifier agents and jj from evidence: the simulator and two small live
   pilots, not from enthusiasm.
