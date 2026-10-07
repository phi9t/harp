# Controller portfolio stabilization and I2 qualification

Status: owner-approved design; independent design and ticket reviews complete.
Implementation has not started.

## 1. Objective

Corenius is the program of record for persistent multi-agent engineering
coordination. This effort first preserves and reconciles Corenius's ongoing
controller work. It then qualifies the retained SQLite controller, I2, through
a recovery-first TraeCLI ladder on `devx_b200_dbg`.

The effort must leave four things true:

1. Every registered worktree and durable effort has a preservation record.
2. Controller work has a complete operational-closure record and no ambiguous
   state is converted into an action.
3. Proven I1 mechanisms enter I2 through focused tickets, not a wholesale branch
   merge.
4. I2 can launch, observe, replace and reconcile real provider workers without
   treating tmux, a provider UI or a process ID as authority.

## 2. Authority and product boundaries

Corenius owns work, intent revisions, acceptance contracts, attempts, provider
bindings, messages, candidates, verification, acceptance, operations, events and
recovery. The controller is the sole execution-state writer.

Kata or the configured work ledger owns human-facing work identity, dependencies
and source revisions. Git owns immutable source and Candidate identity. DevX owns
typed remote access, PTY and tmux control, and developer experience. Insula owns
the declared process and filesystem environment. Provider runtimes execute turns
but do not own work lifecycle. tmux is an attachment and display mechanism.

The current macOS session is the Director and consumes reports from an
independent Observer. It sets portfolio priority, audits evidence and grants the
actions reserved to the human. A distinct GPT-5.5 TraeCLI Lead runs on
`devx_b200_dbg` inside the qualified
developer Insula. The Lead owns day-to-day sequencing, workers, implementation,
tests, reviews, evidence and coherent commits on temporary branches.

The Director retains approval for every pass-two Action Packet, landing, parking,
retirement, push, merge and destructive cleanup. A human performs every merge
into Corenius or a sibling repository. An approved qualification Action Packet
may pre-authorize the authenticated operator command to advance a disposable
Trial Repository's protected ref; that event is fixture evidence, not product
merge authority. The current DevX/Insula implementation Lead remains a separate
role and session.

## 3. Scope

### 3.1 Repository Stabilization Envelope

The envelope covers every registered Corenius worktree and durable effort. Its
record is deliberately small:

- repository and common-Git-directory identity;
- worktree path, Git directory, branch, head and base relation;
- tracked, untracked, staged and conflicted state summaries;
- known owner or explicit unknown owner;
- related durable effort and ticket when established;
- preservation state and next attention point;
- source generation and observation time.

The envelope does not inspect unrelated work deeply and never changes it.

### 3.2 Controller Portfolio

The portfolio includes the SQLite controller I2, the lead-driven I1 code, record
and journal branches, the parked PostgreSQL/Temporal I3 work, worker monitoring,
controller-bench experiments, controller provider adapters, and their runtime
and evidence state.

Portfolio records add:

- durable intent, decision, ticket and status sources, including immutable
  sanitized snapshots of dirty controller-portfolio source files;
- controller database and export identities;
- provider server, session, thread and turn identities without secret values;
- RoleEpoch, ScopeEpoch, WorkspaceGeneration and RuntimeIncarnation;
- tmux server, session, window and pane identity;
- process identity using PID plus start time or pidfd evidence;
- claims, leases, unresolved effects and quarantine state;
- caches, build outputs, Candidates, verification and acceptance artifacts;
- conflicts between prose, Git history, controller state and live observation;
- explicit unknowns and evidence gaps.

### 3.3 Exclusions

Pass one does not land, rebase, retire or delete branches or worktrees. It does
not resume or steer providers, run costly builds, revive I3, change sibling
repositories, or infer abandonment from elapsed age or absent processes.

This effort does not make Armada a workload executor, make DevX or Insula a
controller, add cluster scheduling, deploy a production service, automate merge,
or clean unrelated Corenius work.

## 4. Stabilization protocol

### 4.1 Two passes

Before pass one, RoleEpoch 0 is a narrow Bootstrap Lead. It implements and
qualifies only the Scout Kit in an isolated worktree. The kit contains schemas
and identity code, the source manifest, read-only collectors, redaction, and the
bootstrap journal. The Bootstrap Lead cannot inspect or alter the portfolio.
Independent review and human-authorized landing fence RoleEpoch 0.

Pass one belongs to RoleEpoch 1, the Scout Lead. It reads the complete estate and
may launch bounded read-only scouts. Both Observation Rounds write only to a
mode-0700 run root outside every Corenius worktree and Git common directory. The
Scout seals both round bundles before comparing them. Only after sealing may it
copy promoted records into the dedicated stabilization evidence worktree and
commit them. The source manifest classifies that worktree as observer-owned, and
no round treats its later record commits as portfolio drift. The Scout cannot
mutate observed worktrees, resume providers, run costly builds, move refs or
begin I1 or I2 implementation.

Pass two acts through one reviewed Action Packet per controller effort or smaller
conflicting authority unit. Each packet proposes exactly one disposition:

- `preserve`: retain state and block mutation;
- `continue`: resume the existing authority after fencing checks;
- `promote`: admit a proven mechanism into focused I2 work;
- `park`: retain evidence and remove the item from the active frontier;
- `retire`: remove only state proven merged, redundant, effect-free and retained.

The Director approves or rejects each packet. Approval binds exact evidence and
source generations. A changed precondition invalidates approval.

### 4.2 Two-round live inventory

The Scout Lead does not pause ongoing work. It executes two Observation Rounds.
Every source adapter returns facts with its source generation and observation
time. The inventory module compares the rounds.

Stable facts enter promoted records. Changed facts become drift. Unavailable or
ambiguous sources become explicit unknowns. An Action Packet that depends on
drifted, stale or unknown state enters Quarantine until a follow-up observation
closes the uncertainty.

No source may refresh, reconcile, garbage-collect, claim, expire or otherwise
mutate state while being observed. The inventory command allowlist is part of the
evidence. A command with hidden cleanup behavior is not an observation adapter.

## 5. Evidence format

### 5.1 Strict YAML

Pass-one records use YAML 1.2 as storage syntax under a closed schema. Readers
accept only the JSON-compatible YAML data model and reject:

- duplicate keys;
- custom tags;
- anchors, aliases and merge keys;
- non-string map keys;
- unknown fields;
- ambiguous extensions and implementation-specific scalar typing.

Literal block scalars carry multiline commands, excerpts and diagnostics. Secret
values, full credential-bearing environments and unrestricted process arguments
are prohibited. Records use references to admitted secret projections.

Every adapter has a closed output-field allowlist. Process collection records
executable identity, PID/start identity, cwd identity and declared safe metadata;
it never persists raw `/proc/*/cmdline` or environment data. Provider and tmux
collectors persist typed IDs and bounded sanitized excerpts only. Credentials are
represented by projection-reference type and identity, never path contents or
secret-bearing environment values.

Evidence Redaction runs on streams before bytes reach disk. It carries a bounded
overlap across input chunks so split tokens cannot evade detection, applies
versioned literal and structural rules, and emits only rule IDs and counts. It
then scans the complete candidate record and referenced sanitized artifacts. A
remaining secret match returns `secret_detected`, writes no candidate evidence,
and retains only a typed failure record with no matched bytes. Run roots use mode
0700 and evidence files mode 0600. Raw terminal, transcript, log, argv and
environment streams are neither evidence nor retained fallback.

`record_id` is the SHA-256 digest of the RFC 8785 canonical JSON form of the
parsed typed record with the `record_id` field omitted. Comments, indentation and
scalar style do not alter identity. Identity-bearing schemas do not admit binary
floating-point fields unless a later version defines an exact representation.

### 5.2 Record families

The first version defines these closed schemas:

- `corenius.bootstrap-mandate.v1` for one expiring pre-I2 RoleEpoch grant;
- `corenius.bootstrap-directive.v1` for one sequenced Director command;
- `corenius.bootstrap-receipt.v1` for delivery, provider turn and result evidence;
- `corenius.bootstrap-closure.v1` for journal reconciliation, import and fencing;
- `corenius.stabilization-source-manifest.v1` for the authoritative inventory
  registries, selectors and read-only commands;
- `corenius.repository-envelope.v1` for the light repository-wide record;
- `corenius.controller-closure.v1` for one controller authority unit;
- `corenius.observation-drift.v1` for changed or conflicting facts;
- `corenius.trial-template.v1` for Trial Repository definitions;
- `corenius.trial-materialization.v1` for generated repositories;
- `corenius.trial-result.v1` for a qualification run;
- `corenius.cleanup-receipt.v1` for authorized retirement.

Each record contains `schema`, `record_id`, collector identity, start and finish
time, host and boot identity when applicable, source generations, facts, unknowns,
redactions and referenced artifact digests.

### 5.3 Source manifest and coverage

A reviewed source manifest is the authority for what pass one observes. It pins:

- the canonical Git common directory by path, device and inode;
- the exact `git worktree list --porcelain -z` registry command and its tool
  identity;
- ref namespaces and the exact `for-each-ref` projection used to enumerate them;
- committed effort and ticket roots selected from exact Git trees;
- explicit controller database, export and runtime-state roots;
- provider, tmux, process, claim, cache and artifact sources with adapter names;
- every allowed read-only argv and its collector version;
- exclusions, observer-owned paths and expected unavailable sources.

Filesystem naming is not a registry. A Corenius-looking directory outside the
declared Git or runtime registries is recorded as an unknown discovery and does
not silently expand collection authority. Each Observation Round binds the
manifest ID and records the generation returned by every source. Coverage is
incomplete when a required source is unavailable or when an undeclared source is
needed to explain observed state.

The initial I1 selectors are:

- `refs/heads/effort/lead-worker-readiness`, observed at design time as
  `7d2f69cb274fe548940b1da4794a2ca2e243c43f`;
- `refs/heads/effort/lead-worker-readiness-records`, observed as
  `540f8f7c652ccfec4a283454cd70234c244d9758`;
- `refs/heads/journal/lead-worker-readiness`, observed as
  `7053f39199aa010bb326c14e97f5a17291969c8c`.

These hashes are audit anchors, not freshness claims. Each round resolves and
records current tips and marks movement as drift. I2, I3, worker-monitoring and
controller-bench selectors follow the same rule in the source manifest.

For Controller Portfolio worktrees, the manifest also admits index and working
tree capture. Each round records HEAD, index tree, status class and per-path stat
generation before and after reading. Staged, modified and untracked source under
declared controller roots is captured as a sanitized immutable artifact with
path, status class, source worktree generation, safe content digest, media type,
size and redaction record. Index and working-tree versions remain distinct.
Binary or oversized files use an admitted artifact collector or remain uncaptured.

If a file changes while read, is unreadable, falls outside the allowlist, exceeds
the admitted limit, or triggers secret detection, the record states
`capture_refused` without retaining raw content or a secret-derived digest. That
unknown blocks every non-`preserve` Action Packet whose target, preconditions or
evidence depend on the source.
The original worktree remains the preservation source. Unrelated worktrees keep
the light dirty/staged/untracked summary and are not content-captured.

### 5.4 Action Packets

Action Packets are reviewed Markdown. Strict YAML frontmatter contains:

```yaml
schema: corenius.action-packet.v1
packet_id: sha256:...
body_sha256: sha256:...
target: ...
action: preserve | continue | promote | park | retire
evidence_records: []
expected_generations: []
status: proposed | approved | rejected | executed | invalidated
required_authority: human
```

All authority-bearing fields live in the closed frontmatter schema. The body
states the observed state, conflicts, rationale, consequences, verification,
rollback or recovery behavior, and remaining unknowns. It is valid UTF-8 in NFC,
uses LF line endings and ends with exactly one newline. `body_sha256` covers those
normalized body bytes. `packet_id` covers the RFC 8785 canonical form of the
frontmatter with `packet_id` omitted and `body_sha256` included. Any frontmatter
or body change creates a different packet. Prose cannot override a typed fact. A
record cannot authorize its own action.

## 6. Deep modules and interfaces

### 6.1 Stabilization inventory module

The module presents one primary interface:

```text
collect(plan) -> EvidenceSet | InventoryError
```

It owns both Observation Rounds, adapter invocation, source generation capture,
comparison, Evidence Redaction, schema validation, canonical identity and
evidence output. Callers do not coordinate individual scanners. Internal adapters
cover Git, durable efforts, controller exports, processes, tmux, providers,
claims and artifacts. Each adapter declares its closed source and output field
allowlists. An adapter is admitted only when read-only behavior and pre-persistence
redaction are proven.

### 6.2 Trial Repository module

The module presents three operations:

```text
materialize(template, run_context) -> MaterializationReceipt
verify(receipt) -> TrialVerification
retire(receipt, evidence_closure, approval) -> CleanupReceipt
```

The module hides Git object construction, deterministic author and timestamp
selection, work-ledger setup, protected refs, fault setup, storage placement and
teardown ordering. Callers never build repository history through shell text.

### 6.3 Controller and runtime seams

I2 remains in `crates/sqlite-controller`; this effort does not introduce a second
controller crate or ledger. New controller behavior enters through existing
authenticated command, query, runtime, Git, work-ledger and control interfaces.

DevX satisfies a runtime adapter interface for exact argv, PTY, attachment, tmux
and provider launch. Insula supplies the admitted filesystem and process context.
Provider-specific identifiers remain behind provider adapters. I2 stores the
typed binding and lifecycle state. A RuntimeBinding contains:

- provider kind, server identity and binary/version capability identity;
- transport endpoint identity and protected authentication reference;
- host boot ID and RuntimeIncarnation;
- provider thread-store, session or thread, and current turn identities;
- RoleEpoch, ScopeEpoch and WorkspaceGeneration;
- admitted workspace-projection identity and validated cwd;
- process identity using PID plus start time and owned cgroup or pidfd evidence;
- tmux generation, socket device/inode, server process identity, session, window
  and pane IDs when a terminal attaches.

Resume validates the whole binding. Wrong-thread, wrong-store, pane reuse after
tmux restart, PID reuse after host reboot, stale capabilities, widened workspace
or a changed authentication reference fail closed. Labels and pane numbers alone
never select authority.

One adapter does not justify a general plugin system. Add only the TraeCLI adapter
needed for Rungs 0 and 1. Codex becomes the second adapter in Rung 2 and confirms
the seam.

## 7. Remote operating model

The Corenius Lead has its own RoleEpoch, provider-session identity, Corenius
worktree and named tmux target. Losing the terminal or provider process does not
transfer authority. Replacement creates a new binding and fences the old
RoleEpoch before work resumes.

I2's authenticated command and mailbox interface is the target Director channel.
Before it qualifies, the Director admits one strict-YAML Bootstrap Mandate
through typed SSH. Its semantic ID binds RoleEpoch, expected provider session and
worktree, allowed operations, expiry, prior mandate and initial directive
sequence. Each strict-YAML directive cites the mandate ID, expected RoleEpoch,
next sequence, prior journal head, transport identity, allowed operation and
directive digest. The Lead writes one atomic mode-0600 receipt file with delivery
observation, provider turn, result digest, status and prior journal head under the
mode-0700 bootstrap run root. Each directive and receipt ID advances the
hash-chained journal head.

The launcher validates `corenius.bootstrap-mandate.v1`, directive and receipt
records before RoleEpoch 0 starts; it does not depend on unfinished I2. tmux may
carry a directive only while the mandate, RoleEpoch, target, prior head and
sequence match. Replayed, reordered, truncated, tampered, expired or superseded
input is rejected as stale. A missing receipt blocks the next directive until
reconciliation. Emergency input can stop or request attention but cannot expand
the allowlist. Once I2's Director interface qualifies, it validates and imports
the complete chain, records `corenius.bootstrap-closure.v1`, fences the mandate
and rejects later bootstrap directives.

An independent Observer samples controller events, process activity, artifact
freshness and repository progress every five minutes. Two consecutive intervals
without meaningful progress, or an explicit `STUCK` signal, alert the Director.
Lead and workers emit `STUCK: <exact blocker>; NEED: <specific help>`. The
Observer cannot steer, kill, restart, verify, accept or land work.

## 8. I1 evidence harvest

The first approved pass-two Action Packet reconciles the I1 code branch, record
branch and journal branch. It treats commit and ticket history as evidence and
stale status prose as a conflict to record. It does not merge I1.

For each mechanism, the harvest records:

- exact source commit and paths;
- controller requirement served;
- state authority and failure model;
- deterministic tests and independent reviews;
- provider or runtime episodes;
- restart, rework and reconciliation evidence;
- known defects and missing negative paths;
- proposed disposition: `promote`, `re-prove`, `reject` or `I1-only`.

Promotion uses a risk-tiered proof floor. Pure state and data logic needs
deterministic behavioral tests and independent review. Provider and runtime
mechanisms need a real B200 episode with disconnect, restart and reconciliation.
Authority and external-effect mechanisms additionally need model-free fault
injection, stale-generation rejection, idempotent replay and retained Quarantine
evidence. A promoted mechanism cannot introduce a second execution-state
authority.

## 9. Trial Repository factory

A Trial Template is declarative and immutable. It defines:

- files, modes and content blobs;
- deterministic commits, parents, authors, timestamps and messages;
- branches, tags and protected refs;
- work-ledger project, work items, dependencies and acceptance contracts;
- verification commands with exact argv;
- controlled fault hooks and allowed activation points;
- expected seed tree, commit, ref and work-ledger identities;
- retention and cleanup requirements.

Materialization happens inside an admitted Insula with pinned Git, Python and
work-ledger tools. The factory owns `GIT_CONFIG_NOSYSTEM`, a private HOME, Git
object format and configuration, fixed author/committer identity and timestamp,
UTC timezone, admitted locale, umask, sorted filesystem traversal, file modes and
`PYTHONHASHSEED`. Work-ledger IDs derive from the template ID and stable item key,
not ambient allocation order. Verification uses an absolute interpreter path,
version and digest.

The receipt binds the template ID, all deterministic environment inputs, tool
identities, run namespace, Git object IDs, refs, work items, filesystem tree and
storage location. Two materializations of the same template under the same
admitted toolchain must have the same semantic seed identity. Run-specific paths
and namespaces remain distinct. An existing namespace or destination fails
closed. Construction stages outside the final path and publishes atomically; a
partial staging tree is never a Trial Repository.

The first template is a small dependency-free Python task-board package. Rung 0
uses one test-first behavior ticket and `python3 -m unittest -v`. Rung 1 adds two
independent tickets created at the same base, so the second Candidate becomes
stale after the first protected-ref advance even if the source changes do not
textually conflict.

The factory retains Candidate objects, controller events, verification output,
process observations and cleanup evidence before retirement. It refuses cleanup
when any effect, Candidate, evidence digest or runtime binding is unresolved.
Cleanup is a state machine with a stable operation ID. Repeating a completed
cleanup returns the same receipt. A partial cleanup reconciles each owned path,
namespace and process before continuing and never treats absence alone as proof
of prior success.

## 10. Candidate and verification identity

A Candidate is a closed typed record. It contains:

- schema and Candidate ID;
- WorkRef, IntentRevision and AcceptanceContract digests;
- Attempt ID, ScopeEpoch and WorkspaceGeneration;
- base commit and tree;
- candidate commit and tree;
- patch or source-archive digest;
- clean index/worktree proof and captured status digest;
- explicit submodule and Git LFS policies;
- produced artifact record IDs;
- producer RuntimeBinding ID and creation time.

Rungs 0 and 1 reject submodules and Git LFS. Later templates must declare and
verify either policy before admission. Candidate identity is immutable. A moved
branch, amended commit, changed artifact or new base creates a new Candidate and
requires new verification.

A Verification binds Candidate ID, verifier identity, suite and command digests,
rootfs and dependency-bundle IDs, environment plan, start/finish times, bounded
outputs and verdict. It materializes the exact Candidate separately and never
executes from the producer's writable worktree. Acceptance binds the Verification
ID and expected controller and protected-ref generations.

## 11. Qualification ladder

### 11.1 Prerequisite

The developer Insula and DevX attachment/control path must pass their complete
B200 qualification. A successful rootfs build alone does not satisfy this gate.
The distinct Corenius Lead starts after qualification and after this effort's
specification and ticket graph are approved.

### 11.2 Rung 0: one TraeCLI worker and recovery

Rung 0 uses a GPT-5.5 TraeCLI Lead and one GPT-5.5 TraeCLI worker. It must:

1. Admit one exact WorkRef, IntentRevision and AcceptanceContract.
2. Materialize one Trial Repository and work-ledger namespace.
3. Create one fenced Attempt and isolated worktree inside the developer Insula.
4. Deliver one bounded assignment and record provider acknowledgement.
5. Bind provider session, RuntimeIncarnation, RoleEpoch, ScopeEpoch, tmux and
   process identities.
6. Produce one clean immutable Candidate.
7. Verify the exact Candidate and base in an independent materialization.
8. Disconnect and reconnect terminal views without changing execution.
9. Restart the controller and replace the Lead at controlled pre-Candidate and
   post-Candidate points.
10. Prove no duplicate worker, effect, Candidate or authority appears.
11. Prepare a human-approved landing Action Packet without automatic merge.

Lost acknowledgements and ambiguous effects enter Quarantine. The controller does
not infer success from terminal text or provider completion. Rung 0 injects loss
or replacement at every state transition:

| Transition | Injected uncertainty | Required result |
| --- | --- | --- |
| assignment prepared | Lead exits before delivery | no worker starts; prepared intent is safely retryable |
| assignment delivered | delivery acknowledgement lost | query exact provider binding; never blind-resubmit |
| provider acknowledged | Lead replaced before first tool effect | same Attempt resumes or closes; no second worker |
| first tool effect | effect receipt lost | retain capacity and Quarantine until reconciled |
| Candidate submitted | controller receipt lost | recover exact Candidate by Attempt; never create a substitute |
| verification started | verifier exits | idempotently resume or rerun against the same Candidate |
| acceptance recorded | reply lost | expected revisions deduplicate the acceptance |
| trial ref advanced | confirmation lost | reconcile operation journal and exact ref before retry |
| cleanup started | controller or helper exits | repeated cleanup is idempotent; unresolved effects refuse completion |

Each row runs with controller restart and, where applicable, Lead replacement.
The retained record proves the old RoleEpoch is fenced before a replacement can
issue commands.

### 11.3 Rung 1: two TraeCLI workers

Rung 1 keeps a GPT-5.5 TraeCLI Lead and admits two overlapping TraeCLI workers.
It proves two-slot capacity, rejection of a third worker, independent worktrees,
Candidate pinning, stale-base rejection after the first protected-ref advance, a
new rework Attempt at the new base, independent verification, serialized landing
preparation and review-driven rework.

Rung 1 is a TraeCLI analogue of the existing controlled-trial scenarios. This
spec supersedes that document's Codex-only provider ordering, not its concurrency,
stale-Candidate, fault, evidence or acceptance requirements. The original Codex
lead-plus-two-workers PASS condition moves to Rung 2. No prior Codex evidence is
relabeled as TraeCLI qualification.

### 11.4 Rung 2: Codex and mixed-provider parity

Rung 2 replays Rungs 0 and 1 through the Codex adapter, then runs one mixed pair
under the TraeCLI Lead. It proves that provider differences stay behind adapters
and do not change controller identities, lifecycle, Candidate rules, evidence or
recovery. It then runs the broader controlled fault matrix.

## 12. Failure behavior

All failures are typed and fail closed. Required classes include:

- `source_changed`;
- `source_unavailable`;
- `identity_ambiguous`;
- `schema_violation`;
- `secret_detected`;
- `runtime_binding_unknown`;
- `delivery_unknown`;
- `effect_unresolved`;
- `candidate_moved`;
- `precondition_changed`;
- `cleanup_refused`.

Unknown delivery never triggers blind resubmission. Absent PID never proves an
effect stopped. A tmux pane, branch name, ticket status or provider UI string
never proves ownership. Candidate movement creates a new Candidate or Attempt; it
does not mutate the verified identity. Cleanup refuses unresolved evidence or
effects.

## 13. Verification

### 13.1 Deterministic gates

- Strict-YAML parser and schema fixtures include duplicate-key, alias, tag, merge,
  unknown-field, type-confusion and canonical-digest negatives.
- Bootstrap fixtures cover valid mandate/directive/receipt/closure chains and
  reject replayed, expired, superseded, truncated, reordered, tampered and
  missing-receipt journals before I2 import.
- Source-manifest fixtures prove complete registry coverage, exact command
  allowlists, observer-owned exclusions, missing-source failure and round binding.
- Dirty-source fixtures distinguish index and working-tree versions, capture safe
  tracked and untracked files, detect during-read drift, and block action for
  unreadable, oversized, binary-unadmitted or secret-bearing content.
- Action Packet fixtures prove frontmatter and normalized-body identity, reject
  unknown authority fields and invalidate approval after any body change.
- Evidence-redaction fixtures place fake tokens and credential values in env,
  argv, tmux text, provider transcripts, logs, structured fields, input chunk
  boundaries and capture truncation boundaries. No matched bytes reach disk, and
  a post-redaction match fails closed.
- Permission fixtures require mode 0700 evidence roots and mode 0600 files.
- Inventory adapters prove no mutation against disposable sources and refuse
  commands with cleanup or reconciliation behavior.
- Two-round fixtures cover stable, drifted, unavailable and conflicting sources.
- Trial materialization runs twice and compares semantic seed identity, Git
  objects, refs, work items and verification commands.
- Trial factory negatives cover work-ledger ID allocation, Git configuration,
  locale, timezone, umask, filesystem ordering, Python identity, namespace
  collision, partial materialization, changed toolchain and repeated cleanup.
- Cleanup tests cover unresolved effects, missing evidence, changed generations
  and idempotent completed cleanup.
- I1 harvest fixtures ensure every promoted mechanism has evidence at the required
  risk tier and one disposition.
- Controller tests cover revisions, generations, duplicate commands, lost
  acknowledgement, stale Candidate, replacement fencing and Quarantine.
- RuntimeBinding negatives cover wrong-thread and wrong-store resume, pane ID reuse
  after tmux restart, PID reuse after host reboot, stale capability versions,
  widened workspace scope and changed authentication references.
- Candidate fixtures cover dirty state, changed base/tree, amended commit,
  unspecified submodule/LFS policy, mismatched artifact digests, producer-worktree
  verification refusal and environment drift.
- Recovery fixtures inject failure at every transition in the Rung 0 table and
  prove idempotent replay or retained Quarantine.

### 13.2 Real B200 evidence

The public DevX client drives the real SSH to developer-Insula path. Receipts bind
exact Corenius, DevX and Insula commits, rootfs and bundle IDs, host boot ID,
DeveloperWorkspaceId, InsulaId, tmux generation, provider session, Attempt and
Candidate. Evidence distinguishes deterministic fixtures, simulated faults, live
provider runs and physical-terminal input.

No secret value, SSH agent, host home, kubeconfig, Docker socket, provider token,
full credential-bearing environment or unrestricted process argument list enters
the evidence bundle.

## 14. Delivery sequence

1. Qualify the developer Insula and DevX attachment path.
2. Approve and commit this effort's glossary, decisions and specification.
3. Create blocker-linked tickets and their acceptance evidence.
4. Admit a Bootstrap Mandate and launch RoleEpoch 0, the distinct B200 Bootstrap
   Lead, to implement and qualify the Scout Kit only.
5. Independently review and human-land the Scout Kit, close its bootstrap
   journal and fence RoleEpoch 0.
6. Launch RoleEpoch 1, the B200 Scout Lead, with a new Bootstrap Mandate.
7. Produce and review the sealed two-round Repository Stabilization Envelope and
   Controller Portfolio closure.
8. Approve the bounded I1 evidence-harvest Action Packet.
9. Create and review focused I2 Mechanism Promotion tickets.
10. Build the Trial Repository factory and remaining Rung 0 I2 behavior.
11. Complete Rung 0 with the real TraeCLI Lead and worker, reconcile and close
    the bootstrap authority channel, and use I2 thereafter.
12. Complete Rung 1 with two TraeCLI workers.
13. Complete Rung 2 with Codex and mixed-provider parity.
14. Use reviewed recovery records to migrate Sureal, Trondheim and Vaso without
    importing their local coordinators as new controller authorities.

Each implementation ticket uses its own worktree and clean-context worker. The
Lead may commit coherent ticket branches. The Director approves every pass-two
Action Packet and Git landing action. A human performs every merge.

## 15. Acceptance

This effort is complete only when:

- the repository envelope covers every registered worktree and durable effort;
- controller closure records contain no unclassified conflict or silent unknown;
- all Action Packets bind exact evidence and generations;
- the I1 harvest assigns every selected mechanism one reviewed disposition;
- I2 remains the only execution-state authority;
- the Trial Repository factory passes deterministic and negative-path gates;
- Rungs 0, 1 and 2 pass at exact recorded commits;
- controller and Lead replacement do not duplicate authority or effects;
- all retained claims are scoped to evidence actually exercised;
- independent review has no unresolved blocking finding;
- the human receives reviewed branches, fresh evidence and remaining risks;
- no automatic merge, destructive cleanup or sibling-repository mutation occurs.
