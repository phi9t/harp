# Agentic-engineering guideline map

What a coherent guideline set has to delineate, where each topic stands today,
and what must be decided. This is the outline the guideline set will be
written against, not the guidelines themselves. Conflict ids (C1-C20) refer to
`docs/reference/agentic-inventory/guidance-and-workflows.md` section 3.2.
Sources are the inventories and references under `docs/reference/`, the
collaboration spike under `docs/spikes/collab/`, and the repos' own guidance.

Status labels: **covered** (a clear rule exists and repos agree), **partial**
(rules exist but are scattered, repo-local or conflicting), **gap** (nobody
says). "Recommended default" is a proposal from the evidence, not a decision.

## 1. How the set fits together

Guidance needs layers, or every repo restates everything and drifts:

| Layer | Owns | Changes | Where |
| --- | --- | --- | --- |
| 0 Constitution | Values: honor the request, protect existing work, verify reality | almost never | `CONSTITUTION.md` (upstream) |
| 1 Fleet policy | Workflow, roles, git contract, evidence, vocabulary | versioned, synced | `docs/agents/` in corenius |
| 2 Repo profile | Execution boundary, gates, build/test commands, local exceptions | per repo | the repo's `AGENTS.md` and `docs/agents/` |
| 3 Effort | Spec, tickets, claims, handoffs for one piece of work | per effort | tracker (`.scratch/<effort>/`) |
| 4 Turn | Prompt, tools, permissions for one agent turn | per turn | harness and provider |

Precedence, highest first: direct user instruction, effort, repo profile,
fleet policy, constitution. Today this precedence is only stated in one
sentence in each policy copy. It must state what a lower layer may *not*
override (the constitution, and safety rules in section 11), or "more specific
wins" lets a repo profile switch off protection.

A **repo profile** is a missing concept. Repos differ in kind (a GPU-scarce
build harness like milano, a docs or observatory repo, a product monorepo like
arnold, a vendored upstream like megatronlm), and the fleet policy today
assumes one kind. The set needs a small vocabulary of profiles, each fixing
defaults (worktrees required or not, gates, who merges, what a spike may
touch), with per-repo overrides recorded, not implied.

## 2. Topics to delineate

Each topic gives the question the guideline must answer, its state, and the
decisions needed.

### 2.1 Authority and roles

Must say: who decides what; which agent roles exist and what each may do.

- Roles seen in the repos: human owner, lead, supervisor, executor or worker,
  reviewer, verifier, finalizer, scout, follower (vaso), integrator (milano).
  They are named differently and mean different things in different repos.
- For each role: what it reads, writes, may commit, may merge, may spend
  (GPU-hours, tokens, money), may delegate, and when it must stop and escalate.
- Delegation depth and fan-out limits; who owns the result of a delegated task.
- Human authority points: merge, push, destructive actions, spending, scope
  change, policy change.

State: partial. Constitution and fleet policy put merge with the human; milano
lets an agent integrate to its mainline after gates (C3); vaso is lead/follower.
Recommended default: merge authority is a per-repo setting in the manifest
(human-merge or agent-after-gates), never implicit, and the role table lives in
fleet policy with a fixed vocabulary.

### 2.2 Execution boundary: insula, capsule, rootfs

Must say: where agent-driven code runs and what it may depend on.

- Definitions and their four separate properties: hermetic (dependency
  provenance), isolated (namespaces and binds), offline (no network at run
  time), reproducible (digest from a committed build). The observatory and
  milano define these consistently; soma uses "Insula" differently (C16).
- What the host contributes (driver interface only), what enters through the
  candidate, what is writable (state area), and the path contract inside.
- Capsule identity: what the digest covers. The observatory's covers three
  harness files, not the launcher or build scripts; torchetta's evidence hashes
  the verifier report, not the wheel. The guideline must say what identity
  must cover before a run may claim "reproducible".
- Layer model (base rootfs, foundation, overlay, candidate) and change cost.
- Which agent work must run in the boundary (execution of product code, GPU,
  py-spy) and which may run outside (docs, plain Node/Python tooling).
- The boundary for the *agent itself*: whether the provider process is sandboxed
  (corenius currently spawns it outside the insula; see the interception spike).
- Secrets, tokens and PII in scripts and logs; shared-host privacy (a process
  dump exposes other users).
- Temp and cache placement (C17): megatronlm bans `/tmp`, vaso refuses tmpfs
  roots, flash_attn defaults to `/tmp`.
- Resource leasing: GPUs, output bases, disk; fairness, expiry, stale holders.

State: partial and strong in milano, torchetta, observatory, vaso; absent
elsewhere. Recommended default: adopt the observatory's four-property
vocabulary as fleet vocabulary, require capsule identity to be defined per
profile, and treat "no boundary" as a stated exception, not the silent default.

### 2.3 Source control: repos, branches, worktrees, landing

Must say: the git contract every agent works under.

- One writer per working tree; worktree-per-task or not, and where they live
  (C6).
- Branch naming (`<agent>/<topic>`), mainline naming (C7: `main`, `master`,
  `byted`, `diziet/mainline`, `ultron/mainline`), and the pointer-branch idea.
- Commit contract: one task per commit, what to stage, message form (what and
  why plus verification), trailers (C9), amend (C5), generated files (C8).
- Integration: rebase versus merge, fast-forward only, force-push limits (C4),
  push (C2), who merges (C3), PR/MR conventions and drafts (C12, C19), a
  landing or staging branch and promotion.
- Gate on the exact tip; retry after a lost race; a red mainline is a stop
  condition.
- Cleanup: merged branches, abandoned worktrees, prunable `/tmp` worktrees.

State: partial; milano's `scm` and vaso's relay are the two implemented
mechanisms, the simulator compares them. Recommended default: worktree per
task, fast-forward-only mainline, gate on the exact tip, landing through one
mechanism (`coord`, phase 4), human promotion where the profile says so. Milano's
retry-after-lost-race bug was found here and landed on `diziet/mainline`
(`f8c0120`, 2026-09-30).

### 2.4 Work definition and tracking

Must say: how work is named, claimed and closed.

- Where the tracker lives (`.scratch/` markdown, Kata, GitLab issues, a ledger)
  and which is authoritative when two exist (torchtitan runs `.scratch/` and
  Kata side by side).
- Ticket schema: id, type, status set (open, claimed, resolved), blocked-by,
  owner, acceptance criteria, evidence, comments. Triage roles and labels.
- Claiming rules: who may claim, claim expiry, takeover, duplicate work.
- Cross-repo efforts: the map, child tickets, links back; where the map lives.
- Where designs and plans live (C11): tracker only, or committed `docs/superpowers`.
- Definition of ready and definition of done, per ticket type.

State: partial (local markdown template is fleet-wide; Kata and Ultron's
control plane are extra). Recommended default: markdown tracker as the durable
floor with a fixed status set and claim line; any richer system is a projection
and never the only copy.

### 2.5 Planning and decomposition

Must say: how much process a task gets and how work is cut.

- Path selection: short, bug, gated; the tests for each (the fleet policy has
  them; only Ultron and a few repos apply them).
- Artifacts per path: spec, ADR, plan, ticket; who reviews them.
- Decomposition for disjointness: non-overlapping tickets, path ownership,
  interface freeze and guards as the contract (vaso's rule), what to do when two
  tasks touch a hotspot file (a shared roadmap file is one; milano's
  "tick the roadmap in every commit" rule makes it a conflict source).
- Plan quality: milestones with verifiable exits (this repo's roadmap is the
  pattern), risks, decision points, rollback.
- Timeboxes and stop rules (fix budget, C10: 3 versus 5; when to escalate to
  architectural review).

State: partial. Recommended default: keep Ultron's three paths; one fix budget
constant in fleet policy; plans state exits that a command or a named artifact
can check.

### 2.6 Spike, scout and production

Must say: the rules change when the goal is learning, not shipping.

- Definitions: **scout** (read-only survey), **spike** (time-boxed experiment to
  answer a question), **prototype** (throwaway to show a shape), **production**
  (gated, reviewed, supported). What each may touch, and where it lives (its own
  worktree and branch; a `docs/spikes/` area; never on mainline paths).
- What a spike owes: the question, a verdict, evidence, unverified assumptions
  listed, and a recommendation. This spike's `interception.md` is the model.
- What relaxes (test-first, review depth, naming) and what never relaxes (safety,
  secrets, honesty about verification, isolation).
- Promotion: the criteria and the act that turns a spike into production work
  (re-implement under the gated path; do not merge spike code by default).
- Labels: read versus inferred versus executed; "research is evidence, not
  authority" (observatory).
- Expiry: spikes are archived or deleted, with the finding kept.

State: gap. No fleet policy distinguishes them; the observatory and this
workspace do it by habit. Recommended default: a short fleet document with the
definitions above and a required spike report template.

### 2.7 Communication and message passing

Must say: how agents and humans exchange intent and state.

- Channels: tracker files as the bus (vaso), handoff files, chat relayed by the
  human, harness messages (SendMessage), git itself, ledger events.
- Message types and schema: assignment, claim, status, question, decision,
  handoff, acknowledgement, escalation, completion report.
- Delivery: when a message takes effect. Vaso's finding: a long-running agent
  replays commands from its own context and does not re-read an updated file, so
  policy changes need direct delivery at turn start, and one-shot destructive
  commands must never sit in a handoff.
- Acknowledgement and read receipts; timeouts and what silence means.
- Report format: outcome first, evidence, unverified items, what needs a
  decision (the constitution asks for this; repos vary).
- Human relay: what the human is asked to forward, and how to keep that to one
  message.
- Broadcast versus directed; who may address whom.

State: partial (vaso is the only documented protocol; SDD ledgers define
brief, report, review). Recommended default: append-only message files in the
tracker with a fixed header, plus turn-start delivery in the runtime; the
design-space document covers the event-log alternative.

### 2.8 Multi-agent coordination

Must say: how several agents share a repo and hardware without collision.

- Topologies: lead/follower (vaso), integrator plus workers (milano), supervisor
  tree (Temporal-style, corenius), peer agents behind a queue.
- Claims and leases on paths, tickets and hardware; enforced versus advisory
  (interception spike: advisory until the provider runs in a bind-constrained
  sandbox); expiry and heartbeat.
- Resource scheduling: GPU fairness and starvation (milano's lease releases all
  and sleeps, so an 8-GPU request can starve), stale holder records, priorities,
  preemption, output-base and cache reuse.
- Conflict handling: prediction before work, textual and semantic conflicts, who
  resolves, and what a failed gate means.
- Liveness: detecting a stuck or dead agent, takeover, handover state (vaso's
  `lead.md`), abandoned worktrees.
- Concurrency limits: agents per repo, per GPU, per human reviewer.

State: partial; the simulator gives comparative evidence, with parameters still
uncalibrated. Recommended default: landing queue plus advisory claims first,
fair broker for GPUs, handover file for every long-lived lead.

### 2.9 Verification and evidence

Must say: what "done" and "verified" mean, and what may be claimed.

- Gates per change class and their order; the exact-tip rule ("green means the
  gate ran on the tip that lands").
- Test scope (C13), test-first (C20), formatting (C14), flaky test policy.
- Independent verification: who verifies, with what context, with what budget
  (constitution: verification has its own budget).
- Evidence: content-addressed, append-only, bound to a candidate; measured versus
  declared fields (milano's NCCL receipt copied a declared version); failed gates
  leave a receipt; signed or not; what evidence may be quoted in commits and PRs
  (C12).
- Claim discipline: never claim success without fresh evidence; label verified,
  inferred, assumed. Reference documents carry the commit read.
- Regression and drift checks that run on a schedule.

State: partial and uneven. Recommended default: evidence bound to digests,
declared versus measured separated in every receipt schema, and a rule that a
commit's "Verified:" line names the command and result.

### 2.10 Review and quality

Must say: review roles, loops and standards.

- Review axes (standards and spec), who reviews (independent agent, human,
  roborev-style daemon), severity scale, blocking versus advisory findings.
- Review loop limits and the finalizer step (torchtitan's orchestrator pattern).
- What reviewers may change (nothing, or only findings), and how findings are
  closed.
- PR/MR description content (C12), draft policy (C19).
- Duplicated implementations to converge: bytedcli `mr-reviewer`, roborev,
  Superpowers and Matt review skills.

State: partial and fragmented. Recommended default: two-axis review with a
capped fix loop, findings recorded in the ticket.

### 2.11 Safety and protection of work

Must say: what is never done without authorization, and how agents behave
around others' work.

- Destructive actions: reset, discard, stash, force-push, delete, amend (C4, C5);
  resolve the exact target before acting; prefer recoverable operations.
- Others' work: never edit or clean another agent's tree; untracked notes from
  parallel sessions are left alone; cleaning shared state needs the human.
- Stop-on-correction: when told to stop, stop mutating and report state.
- Secrets and PII: no tokens in repos or logs (arnold ships scripts with hard-coded
  tokens); privacy on shared hosts.
- Permissions: allowlists, hooks and settings; what an agent may run unprompted.
- Provider and network exposure: sending content to external services publishes
  it.
- Incident handling: red mainline, leaked secret, runaway agent, exhausted
  disk or GPUs; who is paged; blameless write-up.

State: partial; the constitution covers the principles, repos add local bans.
There is only one enforcing hook in the fleet. Recommended default: keep the
constitution rules, add a mechanical hook set for the checkable ones.

### 2.12 Knowledge, docs and vocabulary

Must say: what is written down, where, and how it stays true.

- Terminology and glossary (`CONTEXT.md`, domain doc, the observatory's table):
  one glossary with owners; collisions such as "Insula" (C16) resolved.
- Documentation duties per change; ADRs for decisions; research notes as evidence
  not authority.
- Reference documents: commit read, inference labelled, freshness checks.
- Agent memory and handoff state: what lives in the repo versus an agent's
  private memory; nothing decision-critical only in chat.
- Language (C18) and audience.

State: partial. Recommended default: glossary in fleet policy for shared terms,
per-repo glossaries for local ones, freshness date on every reference document.

### 2.13 Skills, hooks and tooling

Must say: how agent capabilities are packaged and governed.

- Skill format: frontmatter schema, validator, tests or evals (about 5 of about
  100 skills ship any), naming, versioning.
- Hooks: which rules are enforced mechanically; how hooks are installed
  (`core.hooksPath`) and tested.
- Subagent definitions and settings: who may add permissions.
- Duplicates to merge (review loops, merge flows, CI repair, bwrap rootfs).
- A shared skills directory and how repos consume it.

State: gap for governance, rich for content. Recommended default: schema plus
validator first (roadmap M5).

### 2.14 Observability, cost and agent evaluation

Must say: how agent work is measured and bounded.

- Run ledger and traces (corenius's durable-execution note); what is recorded.
- Cost and token budgets and gates; what happens at the limit.
- Evaluating the agents and the harness: variance across runs, sample sizes, the
  AIME harness as the template.
- Metrics that show the process working: lead time, wasted gate minutes, conflict
  rate, red-mainline time (the simulator's metrics).

State: gap. Recommended default: adopt the simulator's metric set as the
definition and record them per run.

### 2.15 Lifecycle of the policy itself

Must say: how the guidance changes.

- Versioning, changelog, who may change it, review of policy changes, tests
  (the sync tool has 24).
- Distribution and drift: sync, check, per-repo exceptions recorded in the
  manifest, local files install-once versus managed.
- Deprecation and migration (the `ultron-agentic-workflow` marker and branch
  pointer need one).
- Exceptions: how a repo departs from fleet policy on purpose, and how that is
  recorded and reviewed.

State: partial and improving (manifest, sync, install-once). Recommended
default: every departure is an entry in the repo's profile with a reason.

## 3. Shared vocabulary that must be pinned

Define once, in fleet policy, with "not" columns like the observatory's table:
Insula, capsule, rootfs, candidate, hermetic, isolated, offline, reproducible,
run, evidence, receipt, gate, tip, mainline, landing branch, staging, claim,
lease, ticket, effort, map, spike, scout, prototype, production, lead,
supervisor, worker, integrator, follower, reviewer, verifier, finalizer,
handoff, acknowledgement, profile, policy version, drift.

## 4. Proposed guideline set

One short document per topic, each with the same shape: scope, rules (must and
must not), defaults, exceptions and who approves them, how it is checked, and
the incidents that produced the rules.

| Doc | Covers section | Depends on |
| --- | --- | --- |
| `authority-and-roles.md` | 2.1 | none |
| `execution-boundary.md` | 2.2 | vocabulary |
| `git-and-landing.md` | 2.3 | roles |
| `tracker-and-tickets.md` | 2.4 | roles |
| `planning-and-paths.md` | 2.5 | tracker |
| `spike-scout-production.md` | 2.6 | planning |
| `messaging-and-handoffs.md` | 2.7 | tracker, roles |
| `coordination.md` | 2.8 | git, messaging, boundary |
| `verification-and-evidence.md` | 2.9 | boundary, git |
| `review.md` | 2.10 | verification |
| `safety.md` | 2.11 | roles |
| `knowledge-and-vocabulary.md` | 2.12 | none |
| `skills-and-hooks.md` | 2.13 | safety |
| `observability-and-cost.md` | 2.14 | verification |
| `policy-lifecycle.md` | 2.15 | all |

Write in dependency order: roles and vocabulary, then git, tracker, boundary,
then planning, spike, messaging, then coordination and verification, then the
rest. The 20 conflicts feed straight in: each conflict is settled in the
document that owns its topic (C1-C9 git and landing, C10 and C11 planning,
C12 to C14 verification and review, C15 roles, C16 vocabulary, C17 boundary,
C18 knowledge, C19 review, C20 verification).

## 5. Decisions needed before writing

All seven were decided on 2026-09-30 by adopting the recommended defaults
(repair budget: 3); see `decisions/2026-09-30-policy-defaults.md`. The list
is kept so the guideline documents can cite what each one answers.

1. Repo profiles: adopt the idea and name the initial set.
2. Merge authority per profile (2.1, 2.3).
3. Which tracker is the floor, and how projections are labelled (2.4).
4. Spike and scout definitions and the promotion rule (2.6).
5. Message delivery model: files plus turn-start delivery, or a ledger (2.7).
6. Whether "hermetic" and its siblings become fleet vocabulary as the
   observatory defines them (2.2).
7. One repair budget (3 or 5) (2.5).

## 6. What this map does not cover

Product-specific engineering standards (kernel style, ML numerics, data
handling) and per-repo build instructions stay in the repos. Model and
provider selection, and prompt engineering for particular agents, are a
separate topic. Anything requiring live provider behaviour is unverified until
the interception probe runs.
