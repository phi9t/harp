# Corenius Engineering Profile
## Matt-style skills, broker-managed collaboration, and trunk-based Git

**Date:** 2026-10-11  
**Status:** Proposed design. Not an installed integration or an implementation audit.  
**Baseline:** Corenius RFC of October 2 and agentic-scheduler refinement of October 7. PostgreSQL remains authority; Temporal remains the durable coordinator. Git-backed authoring does not replace either.

## 1. Executive decision

Build an engineering profile on Corenius rather than another agent framework. Use Matt's skills to clarify intent, resolve uncertainty, and describe independently useful outcomes. Use the existing lead/worker/verifier collaboration model for execution. Give agents control over implementation strategy and private iteration; retain machine-enforced authority over workspace allocation, durable claims, verification policy, and trunk promotion.

Choose Git-tracked Markdown as the authoring store. Store live attempts, grants, leases, budgets, operations, and acceptance in PostgreSQL. Preserve Temporal for recoverable coordination. Use native Git, a protected bare integration repository, private attempt repositories with managed worktrees, and a serial final-verification/landing lane. Use Codex app-server as the first native runtime adapter. Reuse Insula for execution and Armada for fleet inspection. Do not implement another SSH launcher, model loop, distributed scheduler, or ticket service.

The important adaptations to Matt's defaults are: transactional claims instead of editing a status line; durable referenced research artifacts instead of automatic branch pushes; a broker-owned task frontier instead of nested subagent orchestration; and incremental trunk landing instead of one feature-sized integration branch.

Harp's accessible contributor guide currently names Kata as its intent ledger and requires explicit permission to push. This profile proposes a different authoring backend for enrolled projects. It must not silently activate beside Kata. For Harp, an explicit migration must select one authoritative authoring backend, preserve issue identity/history, and update its contributor guidance. Until then, retain Kata through the WorkStore adapter. Local landing, remote push, and release are separate permissions.

The term `multi-agent-collab` here means the discussed Corenius/Harp collaboration protocol. A separately deployed implementation bearing that exact name was not established by the source review.

## 2. Concrete components and ownership

| Concern | Selected component | Boundary |
|---|---|---|
| Discovery and engineering guidance | Pinned, selectively adapted `mattpocock/skills` | Guidance, not runtime authority |
| Work authoring | Markdown + YAML front matter in `work/`, ordinary Git history | Definitions, decisions, stable findings |
| Tracker interface | `GitMarkdownWorkStore`, inside Corenius | Parse, validate, identify immutable revisions, propose changes; no new service |
| Runtime authority | Corenius broker + PostgreSQL | Grants, active contract versions, attempts, reservations, acceptance, operations |
| Durable coordination | Existing Temporal layer | Waits, wakeups, bounded retries, reconciliation; not a second task database |
| Native agent runtime | Codex app-server, initially stdio behind a supervised runtime gateway | Threads, turns, tool execution, native context; no terminal scraping |
| Environment | Insula execution contracts; Armada inventory/capability evidence | Physical execution versus fleet inspection remain separate |
| Version control | Native Git CLI, explicit argv, `TaskWorkspace` and `GitIntegrationGate` | Workspace mechanics and protected mutation |
| Evidence | Project-native checks and a separate reviewer using Matt's review criteria | Exact candidate, protected recipe, independent evidence collection |
| Artifacts | SHA-256-addressed filesystem objects on persistent storage, with backup | Logs, bundles, prototypes, checkpoints; no large payloads in Git/Temporal |
| Operator interface | Corenius CLI/JSON, existing tmux/runtime views, editable Markdown | No new dashboard required |

These are logical components, not a service per row. The deployable shape stays a broker, a Temporal worker, PostgreSQL, persistent artifact storage, and the necessary runtime/execution gateways. Keep implementation language consistent with existing components; do not force a rewrite to adopt this profile. The Python code in this bundle is only a Git-mechanics test harness.

Default to one lead, one implementing worker, and independently scheduled verification. Support more workers through the same scheduler when the mission grant allows it and contracts are sufficiently separable. Research and checking can already overlap with one implementation worker. Reserve checking capacity so generation cannot starve completion.

## 3. Repository and storage layout

A project using the new backend has:

```text
AGENTS.md
GLOSSARY.md
docs/adr/
docs/agents/
  issue-tracker.md
  domain.md
  execution.md
.agents/skills/                   # selected pinned/adapted skill payloads
.corenius/project.toml            # declarative profile, not secrets/grants
work/
  M-<id>/
    map.md                       # optional: uncertainty/decision index
    spec.md                      # intent, invariants, acceptance, non-goals
    decisions/D-<id>.md           # a question, resolution, and evidence pointers
    tickets/T-<id>.md             # one independently useful outcome
    outcomes/T-<id>.json          # optional exported acceptance record
```

A bounded fix only needs a ticket; it does not need every document. A map is an index into decisions, not a second specification. A spec owns behavior; a ticket names its slice without copying the entire spec. Preserve the repository's existing canonical documentation locations rather than mechanically imposing this layout on Harp.

Runtime storage is outside tracked source:

```text
/srv/corenius/
  projects/<project-id>/authority.git/
  workspaces/<project-id>/<attempt-id>/
    repo.git/
    worktree/
    scratch/
  verification/<verification-id>/
  artifacts/sha256/<prefix>/<digest>
```

PostgreSQL records workspace identities and lifecycle; a path is not an identity. Local `.harp/work/<id>` views, where retained for ergonomics, are projections/checkpoints rather than another writable authority database.

Keep source and durable work definitions on the same configured trunk. Do not add a synchronization branch or second ticket repository initially. Do not commit heartbeats, task claims, token counters, or streaming logs. Batch useful authoring updates outside the final landing lane.

`outcomes/` is an optional, generated historical export for offline readability. It references the accepted ticket revision and promotion receipt. It is not a permission grant or source for live scheduling. Write it in a subsequent metadata commit: a commit cannot contain its own final hash, and post-verification edits must not mutate the verified candidate. Outcome files do not participate in the original ticket contract digest.

## 4. Work identity and Git/PostgreSQL consistency

A stable ticket ID survives edits, retries, and branch renames. Admission pins the repository identity, source commit, ticket path/blob, referenced spec/decision versions, acceptance recipe, and relevant policy version. Hash the explicit closure of these inputs into a `TaskRevision`; do not resolve acceptance through floating links later.

Git owns authored bytes. PostgreSQL records which exact bytes have been admitted, by whom, under what scope, and whether their outputs have been accepted. This is not two editable copies of the same truth. A working-tree edit or commit is a proposal until admitted. Restarting a controller does not scan arbitrary branches and automatically authorize everything it finds.

Use a small ticket format: ID, type, mission reference, blocking references, expected outcome, relevant invariants, acceptance, and useful context pointers. Authority is inherited from the mission's actual grant, not created by prose. File/path hints for anticipated work are scheduling hints unless a real protected boundary requires an allowlist. Give the model broad read access and reasonable conceptual-module write scope.

For implementation dependencies, `depends_on` means the predecessor's accepted contribution is landed in the chosen trunk. Research/decision dependencies use an explicit `resolved` predicate and any required human approval. A worker returning a commit is not completion. A prototype existing is not a human design decision.

All active claims go through a PostgreSQL transaction. Validate task revision, scope epoch, dependencies, resource/conflict reservations, and uniqueness of an active writer attempt. Insert attempt/operation/outbox records before dispatch. A `Status: claimed` line in two Git branches is not mutual exclusion.

The lead may propose graph changes, splitting tickets, alternative strategies, and new investigations within its envelope. The broker admits graph revisions and checks cycles/dangling references. Completed history is immutable. Changing acceptance or protected scope requires new authorization. Editing a dependency does not erase an already running attempt or retroactively satisfy a join.

Git authoring and PostgreSQL admission are not one transaction. Persist an idempotent admission record keyed by the immutable source revision. A proposal committed but not admitted remains proposed. A referenced source object must be durably retained before an admission is acknowledged. API responses distinguish authored, admitted, dispatched, physically running, verified, and landed.

## 5. Matt skill profile

| Skill | Keep | Adapt |
|---|---|---|
| `wayfinder` | Destination, live decision frontier, unresolved uncertainty, sparse context pointers | Broker claims; map/decision files; no automatic pushes; human decisions remain genuinely human |
| `research` | Source-driven investigation and reusable findings | Persist source/capture provenance and artifact digests; record finding summaries in the work record |
| `prototype` | Cheap concrete artifacts answering a question | Private attempt workspace; export a runnable artifact and decision; no production acceptance by implication |
| `grill-with-docs` and domain modeling | Terminology, assumptions, architecture questions | Ask only unresolved human-owned questions; queue them durably when unattended |
| `to-spec` | Outcome and behavioral contract | Write the single canonical spec; leave strategy explicitly provisional |
| `to-tickets` | Tracer-bullet outcomes and dependency edges | Stable IDs, broker publication/admission, only enough frontier to proceed; no giant fixed implementation script |
| `implement` | Agent-owned engineering and tests | Run within an admitted attempt; submit candidate/evidence; never manage protected refs or other attempts |
| `implement-spec` | Spec/ticket awareness and ready-frontier intent | Replace its orchestration body with the Corenius collaboration driver |
| `code-review` | Spec compliance and architectural/standards criteria | Fresh evaluator, frozen candidate, trusted original contract; reviewer cannot accept itself |
| `retro` | Distill repeatable mistakes into reusable improvements | Prefer checks/tests to more prose; changes to active control policy require separate review |

Upstream `implement-spec` creates an integration branch, starts workers from it, asks them to merge its tip, and merges worker results into that branch. Do not invoke it unchanged beneath another coordinator. Upstream wayfinding can push research branches; this profile explicitly disables that effect unless separately granted. Upstream local Markdown uses `.scratch/` and status-line claims; configure the tracker as `other: GitMarkdownWorkStore`, rather than relying on those conventions for concurrency.

Load only one effective implementation of each active skill. Pin upstream content and local adaptation digests in the session manifest. Do not rely on ambiguous shadowing between a global upstream skill and a repository-local override. Skill updates are explicit reviewed changes; running missions keep their admitted versions. Repo documentation records tracker/domain/execution seams, while the compiled session profile supplies role and authority constraints.

The normal path is adaptive, not a mandatory waterfall:

```text
wayfinder -> research / prototype / grilling -> spec frontier -> tickets
                                                             |
                                    Corenius collaboration <-+
                                       |  private engineering
                                       |  candidate + evidence
                                       v
                                verify -> serial trunk landing
```

Agents may return to research when implementation disproves an assumption. Do not mandate one session per ticket, one fixed context size, TDD for every experiment, or a human approval for every mechanical step. Preserve useful skill practices without treating model-specific pacing rules as permanent architecture.

## 6. Collaboration protocol and autonomy

A lead owns the engineering strategy inside an admitted mission. Workers own how to solve their outcomes. A verifier judges a specific contribution against the actual contract. A deterministic integration gate owns protected ref mutation. These roles may use the same model family; separate roles alone do not provide statistical independence or security.

A worker briefing contains: the objective and why it matters; pinned code/context; invariants and non-goals; observable acceptance; available capabilities and budget; relevant discoveries; and what must be escalated. It does not contain a list of guessed edits. A submission contains candidate identity, claims/evidence pointers, material decisions, remaining limitations, and follow-up proposals.

The model-facing operations are a small protocol (proposed names, not existing commands): inspect work, propose work/revision, request an attempt, record a finding, send a contextual message, checkpoint, submit a candidate, request a decision, and propose acceptance. Protected promotion and root-policy edits are not worker tools. Native descendants must register under the same conserved budget; do not run an untracked recursive swarm.

Use messages for questions and references, not duplicated specifications. Distinguish ordinary notification, requested replanning, pause, and cancellation. A lead rehydration does not interrupt every valid worker. A worker waiting for a long build need not repeatedly invoke a model to check a clock.

Re-anchor at material boundaries: changed public semantics, disproven assumptions, proposed acceptance changes, repeated failures without new evidence, candidate submission, and mission completion. The evaluator sees the original contract and actual artifact, not only the worker's success narrative. Require an acceptance-to-evidence mapping and a check for unjustified scope or policy changes.

Freeze the meaning of success, not the plan. A better algorithm or internal abstraction is generally the agent's decision. Expanded credentials, external communication, protected architecture changes, changed acceptance, or increased budget are authorization decisions. All grants are enforced by runtime controls, not by trusting the agent to interpret a Markdown warning.

## 7. Worktrees and real isolation

**One writable attempt, one workspace generation, one private repository boundary.**

Do not give arbitrary worker shells worktrees directly attached to the protected integration repository. Worktrees share refs and common metadata. Instead materialize an attempt-specific repository through fetch/bundle or a non-local clone, then create its managed worktree. Mount only that attempt's repository metadata and working tree into its capsule. The operator checkout, protected repository, peer workspaces, credentials, and verifier signing material are absent.

Use independent objects initially: no `clone --shared`, writable alternates, or hardlinked object stores across trust boundaries. Immutable/shared caches may be introduced after their safety properties are qualified. A private repository is not sufficient if broad host mounts still expose the gate; the mount/user/process boundary is part of acceptance.

Recommended branch patterns are `work/<ticket>/<attempt>`, `research/<decision>/<attempt>`, and `prototype/<decision>/<attempt>`. Branches are mutable labels; candidate identity is a sealed commit/tree/manifest. New attempts never reuse old mutable paths simply because a lease expired. Resume a workspace only after reconciling its writer and generation; otherwise allocate a new workspace from a retained checkpoint.

The workspace manager owns creation, movement, registration, sealing, archival, and removal. Agents may inspect Git, make private commits, and reorganize unsubmitted local history within their grant. They may not force-update trunk, rewrite a sealed candidate, reset another checkout, clear global state, delete other worktrees, or retarget an admitted attempt to a moving base without a new generation.

At creation, record the exact base, attempt/scope epochs, canonical validated path, repository ID, branch, runtime ID, and required environment. Confirm clean status and base before attaching the runtime. Run `git worktree add --lock` for managed worktrees; use `git worktree list --porcelain -z` for inventory. The Git lock prevents routine removal/pruning; it is not a mutex or permission boundary.

Do not share mutable build outputs by default. Allocate private `.venv`, Rust target directories, Bazel output roots, and Torch extension outputs per attempt. Shared package/source caches need tool-supported concurrency and integrity controls. For Harp's existing Lean arrangement, preserve its documented serialization around shared Harp-owned `.olean` outputs; a symlinked `.lake/build` is not automatically candidate-isolated.

Cleanup is its own journaled operation. Verify process/job references, export required tracked and allowed untracked artifacts, pin candidate/research refs or bundles, classify ignored material, and confirm no unresolved operation holds the workspace. Then unlock and use normal `git worktree remove`. Do not default to force-removal or `rm -rf`. Unknown/dirty/unregistered work is quarantined and surfaced. A successful task with failed cleanup remains successfully landed plus cleanup-pending.

An old timestamp is a warning, not proof a workspace is abandoned. Long jobs may legitimately outlive the agent; their workspace/artifact references prevent cleanup. Prune only after reconciling registry and physical inventory, initially with a dry run. Backup/pin evidence before removing branch labels. Clearing reflogs is not a retention strategy.

## 8. Trunk policy and promotion

Use exactly one configured authoritative local trunk, in a gate-owned bare repository. Preserve the real branch name (`master` for the accessible Harp repository), rather than renaming it to match examples. The operator's checkout is a separate clone/view; synchronizing it is a separate operation that refuses unrelated dirty changes and untracked collisions. GitHub remains the publication remote in this local-first profile, not a competing independently writable integration authority.

All ordinary implementation branches are short-lived outcomes. Default to one coherent integration commit per ticket, while permitting any useful private commit history. Construct the final commit before verification. Large migrations use expand/migrate/contract, compatibility layers, and explicit removal tasks. Feature flags need owners and removal criteria. When a change cannot be made independently green, treat it as one explicitly larger atomic candidate rather than quietly growing a long-lived feature branch.

Only landed accepted dependencies feed normal worker bases. Speculative stacks are not in the initial profile. Independent workers can start from B; after one lands, the other can still reuse its work, but its old green result is not transferable to a new combined tree.

Use one final prepare/verify/land lane per repository:

1. Resolve current trunk B and the active ticket contract.
2. Import sealed worker candidate C under an immutable candidate reference.
3. Build final candidate I on B in an isolated integration workspace. Replaying or squashing happens here, before verification.
4. Run the protected checks and review on exact I; bind the evidence to I, B, contract, recipe, environment, and inputs.
5. Recheck current authorization and record promotion P as prepared.
6. Order commit admission with cancellation and other target mutations through the gate's serialization boundary.
7. Check ancestry and conditionally advance B to I, recording a unique Git receipt ref.
8. Persist/reconcile the PostgreSQL effect receipt. Mark accepted/landed and unblock dependents only now.
9. Schedule cleanup and optional historical outcome export.

The lane is a durable reservation, not a database transaction held across a long test run. Other work and focused tests can proceed in parallel. Metadata commits touching trunk also pass through the lane. Compare-and-set remains mandatory even with a sole normal writer.

For a verified candidate, the protected Git mutation can use:

```text
start
update refs/heads/<trunk> <I> <B>
create refs/corenius/promotions/<P> <I>
prepare
commit
```

This is input to `git update-ref --stdin`. Validate full object IDs/ref names and ancestry separately; `update-ref` does not itself enforce fast-forward policy. The gate owns these refs. The receipt-ref operation is create-only. Git's transaction semantics do not make Git and PostgreSQL atomic and do not synchronize an editable checkout. Concurrent readers can observe a subset of multi-ref updates; reconciliation waits for the gate operation to settle.

If trunk moved before commit, rebuild a new immutable integration candidate and reverify. Never rebase after approval and reuse the old green check. If acknowledgment is lost, reconcile the prepared operation, unique receipt ref, and actual trunk under the sole-writer invariant. Do not blindly replay or declare success solely because a branch happens to equal the expected commit. A conflicting marker or unexplained mutation stays uncertain.

Cancellation prevents future authorized effects; it is not a promise to reverse an already admitted/completed Git update. Commit admission and cancellation must have an explicit ordering in the gate. An unresolved committing operation blocks conflicting target work until reconciled. A controller failover must fence the old gate process/credentials, not merely take over a database lease.

Local landing requires a specific grant. Missions can delegate routine local landing so the operator need not approve every ticket, but Harp's current human-merge policy remains until explicitly changed. Push, publication, release, and deployment require distinct grants.

## 9. Verification, recovery, and drift

Verification has three levels: focused private checks during implementation; protected final checks on the integrated candidate; and an independent contract/architecture review for meaningful changes. The gate evaluates a pinned acceptance recipe obtained from approved state. Candidate edits to tests, CI, `AGENTS.md`, or skills do not redefine the checks governing that candidate. Self-hosted harness improvements are evaluated using the old trusted harness plus separately reviewed migration evidence.

Run candidate tests as untrusted execution with no gate or signing credentials. A trusted wrapper collects outputs and records exit codes, skipped checks, environment identity, toolchain, datasets/seeds, and relevant numerical/performance results. Successful unit tests do not substitute for specified GPU, integration, or fault-injection gates. A unavailable required GPU check means blocked verification, not accepted-with-an-unmentioned-skip. Reviewers can disagree; unresolved material findings are explicit blockers, not automatically majority-voted away.

A cold-start briefing is generated from the mission revision, accepted trunk, relevant decisions/findings, active attempts/jobs, unresolved operations, current budget, and next decision. Preserve evidence pointers and failed hypotheses, not a massive mandatory transcript. A checkpoint includes recoverable source/artifact identities and pending external effects, not just the model's prose summary.

On restart, reconcile before dispatch. Healthy child jobs survive lead replacement. Lease expiry fences result admission but does not prove the old process is stopped. Uncertain workspaces are not reused. Start requests with unknown outcomes are looked up through stable operation identity. PostgreSQL backup rollback requires authority-domain rotation/fencing and reconciliation of external effects; replaying Temporal history is not a fresh authorization grant.

## 10. Initial delivery slices and conformance

The first delivered slice should be a complete scripted-worker round trip: admitted Git ticket -> private workspace -> candidate -> protected verification -> conditional local landing -> receipt -> safe cleanup. Use a deterministic fake agent initially to isolate substrate bugs. Then add native Codex execution, restart recovery, and a second independent worker. Do not begin with a general purpose DSL or a complex UI.

| Slice | Independently verifiable result |
|---|---|
| 1. Single-ticket vertical path | Exact accepted source revision lands through the gate; failure blocks landing |
| 2. Matt/native runtime profile | Selected skills find one tracker and one coordinator; runtime has no protected-ref access |
| 3. Durable continuation | Restart lead/broker; adopt known jobs and recover without duplicate protected effects |
| 4. Concurrent frontier | Two independent workers; stale base rebuilt/reverified; landed dependency gating |
| 5. Research/prototype loop | Captured artifacts, human-owned decisions, revised spec, no implicit push |
| 6. Operational hardening | Crash points, budget pressure, storage failure, safe cleanup, backup/fencing exercises |

Required conformance cases include: simultaneous claims; stale result after replacement; candidate tests that attempt to alter verifier policy; changes to upstream spec while a worker runs; trunk moving after verification; lost promotion acknowledgment; old gate resuming after failover; dirty operator checkout; live jobs during cleanup; missing required hardware; conflicting interface changes across disjoint files; and a supposedly complete mission missing a requirement.

The bundle's seven executable tests cover only native Git mechanics: successful conditional promotion with receipt; stale-base rejection; receipt conflict blocking target update; observing a lost acknowledgment without replay; separate attempt refs; locked-worktree removal refusal; and preservation of an untracked file under normal removal. They passed with Git 2.47.3 and Python 3.13.5. They do not test PostgreSQL/Temporal integration, OS isolation, malicious Git objects, crash durability, provider control, or end-to-end autonomy.

Judge success by accepted useful outcomes per unit of wall-clock, model budget, and operator attention; measure rework, escaped defects, stale-candidate churn, verifier wait, and successful recovery. Agent count, ticket count, and generated documentation are not success metrics.

## Sources and scope notes

Project inputs read for this design:

- `corenius-design-v0.1.md`, October 2, 2026 (Library). Proposed RFC, explicitly not implementation-verified; PostgreSQL/Temporal, native runtime, immutable candidates and protected promotion.
- `DESIGN.md`, October 7, 2026 (Library scheduler lab). Refines that RFC; corrects Armada to fleet management and Insula to execution; SQLite is a learning oracle only.
- `phi9t/harp/AGENTS.md`, accessible `master`. Existing Kata, local landing/no-push, worktree and Lean-cache rules. https://github.com/phi9t/harp/blob/master/AGENTS.md

Primary external sources checked:

- Matt's skills: https://github.com/mattpocock/skills
- Wayfinder: https://github.com/mattpocock/skills/blob/main/skills/engineering/wayfinder/SKILL.md
- Implement-spec: https://github.com/mattpocock/skills/blob/main/skills/engineering/implement-spec/SKILL.md
- To-tickets: https://github.com/mattpocock/skills/blob/main/skills/engineering/to-tickets/SKILL.md
- Local tracker: https://github.com/mattpocock/skills/blob/main/skills/engineering/setup-matt-pocock-skills/issue-tracker-local.md
- Prototype: https://github.com/mattpocock/skills/blob/main/skills/engineering/prototype/SKILL.md
- Codex app-server: https://developers.openai.com/codex/app-server
- Git worktree: https://git-scm.com/docs/git-worktree
- Git clone: https://git-scm.com/docs/git-clone
- Git update-ref: https://git-scm.com/docs/git-update-ref
- PostgreSQL locking: https://www.postgresql.org/docs/current/explicit-locking.html
- Temporal Python: https://docs.temporal.io/develop/python

These rolling sources describe upstream behavior, not a pinned installation. The profile must pin and qualify the versions actually installed. No repository, library source, tracker, service, agent setting, or remote was modified by preparing this bundle.
