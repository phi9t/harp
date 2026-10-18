# Effort: Controller portfolio stabilization

Status: owner-approved design and reviewed specification/tickets; no stabilization actions authorized.
Program of record: Corenius.
Base observed for this design: `main` at
`d7f674829e9844d011869b711d64fcf55313f7f1`.

## Intent

Put Corenius's ongoing multi-agent controller work into a coherent, preserved
and reviewable state before adding another controller implementation. Keep the
whole repository safe while focusing active reconciliation on the Controller
Portfolio.

## Approved direction

- Corenius owns the persistent engineering-team control-plane domain. DevX and
  Insula are runtime dependencies and integration surfaces, not competing
  controller authorities.
- The retained SQLite controller (I2) is the product core. The lead-driven line
  (I1) is an operational proving ground and source of validated mechanisms. The
  PostgreSQL/Temporal line (I3) remains parked.
- Stabilization has two passes. Pass one is evidence-only. Pass two uses one
  separately reviewed, human-approved action packet per controller effort.
- Pass one is tiered. Controller work receives a complete operational-closure
  record. Unrelated work receives a lighter preservation record and remains
  otherwise untouched.

The I2 priority reuses the existing owner decision in
`docs/efforts/team-control-plane/decisions/20261004-owner-sqlite-controller-first-supersession.md`.

## Pass-one scope

Every registered worktree and durable effort receives enough identity and
preservation metadata to prevent accidental damage. Controller Portfolio
records additionally cover Git refs and worktrees, effort and ticket state,
provider-session identities, tmux and process bindings, claims, external
effects, caches, artifacts and unresolved quarantines. Secret values are never
captured.

## Non-goals during pass one

- Landing, rebasing, retiring or deleting any branch or worktree.
- Resuming, steering or replacing provider sessions.
- Reconciling unrelated implementation work beyond its preservation record.
- Starting another controller architecture or reviving I3.
- Changing DevX, Insula, Vaso, Sureal or Trondheim.

## Authority boundary

The human approves every pass-two action. An inventory observation, apparent
cleanliness, elapsed age, absent process or passing test does not authorize a
landing, parking or retirement action.

## Operating topology

The Director operates from the current macOS session and observes or directs the
portfolio without sharing Lead authority. A replaceable Lead runs on the B200
host and autonomously sequences tickets, creates isolated worktrees, launches
workers, changes code, runs tests and reviews, records evidence, and makes
coherent commits on temporary branches. Workers receive bounded Attempts and
never share a mutable worktree.

The Director retains approval for every pass-two action, landing, parking,
retirement, push, merge and destructive cleanup. The Lead's durable identity,
RoleEpoch, provider session, RuntimeIncarnation, worktree and admitted objective
must be recorded independently of its tmux pane.

I2's authenticated controller interface is the target Director authority. Before
it qualifies, a Bootstrap Mandate and append-only receipt journal bind each
directive to the current RoleEpoch, provider session, worktree, operation
allowlist, expiry and monotonic sequence. Typed SSH authenticates transport. tmux
may deliver only the exact current directive and never supplies authority by
itself. The bootstrap journal is reconciled into I2 and closed when I2 assumes
the Director channel.

Corenius receives a distinct GPT-5.5 TraeCLI Lead inside the qualified developer
Insula. It has its own RoleEpoch, provider-session identity, Corenius worktree
and named tmux target. The DevX/Insula implementation Lead remains a separate
role and session until its own work is handed off.

RoleEpoch 0 is a narrow Bootstrap Lead. It may implement and qualify only the
Scout Kit in its isolated worktree: strict-YAML schemas and identities, source
manifest, read-only collectors, redaction pipeline and bootstrap journal. It
cannot inspect or alter the observed portfolio. Independent review and
human-authorized landing fence RoleEpoch 0 before observation begins.

RoleEpoch 1 is the Scout Lead. It may inspect the complete estate and launch
bounded read-only scouts. Observation output goes only to a dedicated mode-0700
run root outside every Corenius worktree and Git common directory. Both rounds
are sealed before records are copied into the dedicated stabilization evidence
worktree. It cannot edit observed worktrees, resume providers, run costly builds,
move refs or begin I1/I2 implementation. Pass two requires Director approval.

Pass-one facts are stored as immutable strict-YAML evidence records under a
closed schema. A reviewed source manifest declares every registry, selector,
read-only command and expected generation used for coverage. Human
interpretation, conflict resolution and each proposed Action Packet are reviewed
Markdown with strict YAML frontmatter bound to exact record identities and a
digest of the normalized body. Prose cannot override facts, and unfinished I2
state is not the sole copy of preservation evidence.

The YAML reader accepts only the JSON-compatible YAML 1.2 data model and rejects
duplicate keys, custom tags, anchors, aliases, merge keys and non-string map
keys. Multiline scalars are allowed. Record identity is the SHA-256 digest of
the RFC 8785 canonical JSON form of the parsed typed data, not the YAML bytes,
so comments and formatting do not change identity.

Every collector uses a closed field allowlist and applies streaming Evidence
Redaction before persistence. Evidence roots are mode 0700 and files mode 0600.
Records store credential projection references, never credential values, raw
environments, unrestricted argv or unsanitized terminal/provider text.

Pass one observes the live estate twice without pausing ongoing work. Every fact
retains its source generation and observation time. Facts stable across both
Observation Rounds are promoted; changed facts are recorded as drift. An Action
Packet that depends on drifted state is quarantined until a stable follow-up.

The first pass-two Action Packet is a bounded I1 evidence harvest. It reconciles
the I1 code, records and journal branches, maps mechanisms to implementation,
tests, live evidence and gaps, and assigns each mechanism one disposition:
`promote`, `re-prove`, `reject` or `I1-only`. Approved promotions become focused
I2 tickets. The I1 branches are never landed wholesale.

Mechanism Promotion uses risk-tiered proof. Pure state and data mechanisms need
deterministic tests and independent review. Provider or runtime mechanisms need
a real B200 episode plus disconnect, restart and reconciliation evidence.
Authority and external-effect mechanisms additionally need model-free fault
injection, stale-generation rejection, idempotent replay and retained quarantine
evidence. No promotion may introduce a second execution-state authority.

The first I2 product tracer is one recovery-first worker flow: admit exact work,
intent and acceptance; create one fenced Attempt and isolated worktree inside the
developer Insula; launch one GPT-5.5 TraeCLI worker through DevX; bind delivery,
provider, runtime, role, tmux and process identities; capture and independently
verify one exact Candidate; exercise controlled controller or Lead replacement;
and prepare a human-approved landing Action Packet without automatic merge.

The distinct Corenius Lead on `devx_b200_dbg` executes this tracer. It starts only
after the developer Insula qualifies, pass one completes, and the I1 harvest is
approved. The current DevX/Insula Lead is not repurposed.

This tracer is Qualification Rung 0. It does not replace the existing controlled
trial. Rung 1 retains the lead-plus-two-overlapping-workers contract, capacity
enforcement, stale-base rejection, rework and serialized landing. Rung 2 adds
provider parity and the broader fault matrix. Each rung is cumulative and its
claim is limited to the evidence it actually exercises.

Rungs 0 and 1 use a GPT-5.5 TraeCLI Lead and TraeCLI workers on the B200 host.
Rung 2 replays the same contracts with Codex, then tests mixed-provider workers.
Controller schemas and lifecycle rules remain provider-neutral.

Rung 0 changes a disposable Trial Repository rather than Corenius or a sibling.
The repository is a real private Git and work-ledger scenario generated from a
versioned deterministic Trial Template. Its seed identity, generated history,
work items, tests, acceptance contract, fault hooks and teardown evidence are
part of the run record. It is not an ad-hoc mock or ambient `/tmp` checkout.

Trial Repositories are produced by a schema-driven factory. A Trial Template
declares files and modes, deterministic commit history, refs and protection,
work-ledger items and blockers, tests, acceptance, fault hooks and expected
identities. Materialization emits a content-addressed private repository and a
receipt binding the template, tools, generated objects and run namespace.

An independent Observer samples controller events, process activity, artifact
freshness and repository progress every five minutes. Two consecutive intervals
without meaningful progress, or an explicit `STUCK` signal, alert the Director.
Lead and workers report `STUCK: <blocker>; NEED: <help>` instead of waiting
silently. The Observer cannot steer, kill, restart or accept work.

## Resolved design frontier

`SPEC.md` now defines the recovery-first tracer and public proof seam, typed
Action Packet identity and invalidation, risk-tiered I1 promotion criteria,
source coverage, bootstrap authority, and stabilization ordering. Implementation
tickets must cite those sections and may not reopen them implicitly. A material
change returns to owner design review and records a superseding decision.
