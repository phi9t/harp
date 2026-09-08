# Round 2: recoverable coding candidates

Status: proposed specification for review, not an implementation claim.
Baseline: `f6feae04` on `codex/workflow-explain`. This commit is not merged into
`master`. The next implementation must include this baseline or reconcile it
explicitly with the selected integration branch.

## Research workload extension

The [frontier research scenario design](frontier-research-scenarios.md) makes
architecture implementation and bounded research repairs concrete consumers of
this workspace/candidate contract. Training-state checkpoints and long-running
job reconciliation are additional contracts; preserving source edits alone does
not resume a training process. Use the [workload/environment matrix](workload-environment-matrix.md) for the
current delivery order.

## Updated sequencing

[plan.md](plan.md) owns the implementation order. This document is the bounded
managed-coding module specification, used by work item C1 and subsequent
agent-authored research. The first W0 campaign trains supplied fixed models and
can qualify independently of this module. Repository-writing agents require
qualified sandbox authority, so E0 direct execution cannot establish this module's
mutation guarantees. Patch review remains a read-only conformance fixture.

## Recap and problem

Round 1 added `workflow inspect` and `workflow explain --task`. They read an
existing SQLite database without migration, artifact initialization, provider
launch, or recovery effects. A consistent snapshot exposes task dependencies,
accepted results, attempt failures, checkpoint references, bounded event pages,
and conservative recovery prerequisites. CLI continuation counts come from the
CLI extension when present. Inspection remains advisory; resume owns authority
checks and effects.

The full gate passed with `RUST_TEST_THREADS=2`, including Atlas, Rust, laboratory,
Lean and repository verification. The first default-parallel gate hit three
fake-provider version-probe timeouts; all passed in the complete rerun. No
production timeout was relaxed. Independent review was completed.

Remaining gaps are substantive:

- `thread_spec` still binds cwd and runtime roots to attempt scratch.
  `GitWorktree` currently changes the sandbox string, not workspace lifecycle.
- There is no durable repository-mutation intent or before/after candidate.
- A valid result envelope can be accepted without candidate-bound checks.
  The scheduler's evaluation artifact currently records `deferred`.
- Checkpoints record progress strings, not a restorable coding state.
- An interrupted agent can leave useful changes without a usable final response.

The coding repair milestone should make those changes recoverable and reviewable while
preserving one Engine. It must not silently treat observation of changed files
as proof that a coding task succeeded.

## Outcome and bounded scope

Deliver one end-to-end managed coding task: start from a pinned repository
snapshot, edit in an isolated workspace, preserve a candidate, run declared
checks against that candidate, and recover after interruption without blindly
repeating completed edits.

The representative fixture edits a parser, adds a regression test, and runs a
local test command. Kill the process after the edit, after candidate publication,
after verification completes, and before task acceptance. A new controller must
identify exactly what survived, recover permitted work, and explain the next
step through the existing inspection commands.

Included:

1. Managed Git workspace identity and ownership.
2. Durable mutation intent, reconciliation and immutable candidate artifacts.
3. Deterministic verification bound to candidate and execution conditions.
4. Engine-built continuation packets at verified boundaries.
5. Review output linking failures, candidates, checks and recovery decisions.

Deferred: automatic test-failure repair loops, agent-authored graph expansion,
selective reuse across changed policies, multi-candidate merging, pushes,
publication, remote service writes, automatic cleanup of unresolved workspaces,
general CopyOnWrite, and arbitrary filesystem rollback. Human acceptance and
merge remain separate from Engine task acceptance.

## Choices and alternatives

Keep the existing Engine and add a workspace/effect module. A richer result
message alone cannot recover edits made before a response was saved. Universal
transactional rollback would overstate what Git and provider processes can
recover. The selected approach is admitted local mutation plus immutable evidence
and explicit reconciliation.

This refines the approved direction in
[the side-effect design](../../superpowers/specs/2026-08-13-dynamic-workflow-side-effect-policy-design.md):

- A status digest is insufficient. It must bind file contents, modes, additions
  and deletions; unchanged porcelain status does not mean unchanged bytes.
- `dispatch_intent_recorded` means launch may have occurred, not that it did.
- An effect key must survive delivery retries and attempt replacement for the
  same logical effect. A new attempt ID cannot itself authorize a new mutation.
- A recorded change can be reusable evidence without being successful work.
- Quiescence must be established before snapshotting or transferring write access.
  A lease expiry does not stop the previous provider process.

These refinements supersede the older draft where they differ if this spec is
adopted. They do not change current behavior before implementation.

## Work contract and identities

Add a versioned, pinned `CodingWorkPolicy` at the ExecutionPlan/receipt boundary.
It applies identically to authored workflows and raw TaskGraphs. It contains:

- repository identity and exact base commit/tree, captured at admission;
- an Engine-owned workspace root and writable relative-path scope;
- candidate format and explicit storage/path/file-size limits;
- required verifier specifications, timeout/output limits and environment policy;
- allowed continuation mode and remaining attempt/run budgets.

The compiler lowers authored fields; adapters must not interpret them. The CLI
resolves and pins an optional branch/ref to an exact object before creating the
run. Resume never resolves that mutable ref again. Dirty operator checkouts are
not adopted or cleaned: execution uses the selected committed base. The CLI
must explicitly report that baseline. A request to include uncommitted changes
is unsupported in this round and must be rejected rather than silently omitted.

Use distinct identities:

| Identity | Meaning |
| --- | --- |
| run / task | Existing orchestration identity |
| attempt / activity | Existing execution and delivery history |
| workspace ID | Stable managed workspace allocation |
| ownership epoch | Monotonic grant of write ownership |
| effect key | Run + task + logical effect slot + semantic revision |
| candidate digest | Immutable manifest of exact output bytes |
| verifier key | Candidate + check spec + tool/environment identities |
| recovery decision ID | Persisted prerequisites and selected transition |

Engine allocates effect slots. Reattachment and transport redelivery retain the
same key. A new semantic effect or future repair increments a recorded revision;
callers cannot bypass ambiguous work by changing an attempt ID.

## Records and module boundaries

Persist dedicated workspace/effect records rather than overloading attempt state.
An attempt can fail while retaining a valid candidate or an unresolved effect.

| Record | Required information |
| --- | --- |
| WorkspaceRecord | Repository and administrative-directory identities, root device/inode, base tree, owner/epoch, scope/policy digest |
| MutationIntent | Effect key, attempt/activity links, admitted baseline digest, ownership epoch, prepared/dispatch-intent state |
| MutationReceipt | Intent, before/after manifests, process-quiescence evidence, disposition, observed scope violations |
| CandidateManifest | Base identity, complete path/content/mode mapping, additions/deletions, artifact hashes, producing intent |
| VerificationReceipt | Candidate digest, verifier key, argv/cwd/env/tool identities, process outcome, bounded stdout/stderr references, verdict |
| ContinuationPacket | Original contract, candidate/checkpoint refs, observed facts, agent-reported notes, pending work, remaining authority/budget |
| RecoveryDecision | State version, ownership prerequisites, referenced evidence, selected action and reason |

`harp-engine` owns workspace admission, recovery decisions, candidate acceptance
and verification scheduling. Put Git/process details behind a small module API,
not branches spread throughout scheduler and CLI code. `harp-state` owns typed
rows, compare-and-swap transitions and transactional events. `harp-artifacts`
owns immutable bytes, digest validation and descriptor-safe publication.
`harp-runtime` and `harp-cli-process` execute and observe authorized activities;
they do not decide replay safety.

Initially keep internal policy types in the owning module. Move versioned durable
wire types into `harp-contracts` with migrations and boundary tests when first
used. Reuse the existing supervised process lifecycle for deterministic checks;
do not add a second scheduler or process manager.

Logical interface:

```text
admit(task_claim, pinned_policy) -> admitted_workspace_effect
observe(effect, runtime_control) -> running | quiescent_evidence | unknown
capture(effect, quiescent_evidence) -> candidate_receipt | blocked_receipt
verify(candidate, pinned_checks) -> verification_receipts
assess_recovery(persisted_snapshot) -> recovery_decision
apply_recovery(decision, expected_state_version) -> transition_outcome
```

## Workspace and mutation rules

Allocate one Engine-owned workspace per concurrently writable task. Linked Git
worktrees may share immutable object storage; sibling activities must not share
a mutable index, source tree or administrative state. Workspace allocation itself
is journaled before filesystem effects, with a stable destination and identity
checked on recovery. An existing unknown directory is never adopted or removed.

Only a clean newly allocated baseline is admitted. Record root, Git common-dir
and per-worktree admin identities, HEAD, index and content-manifest digests.
Engine Git operations use bounded output, timeouts, a controlled environment,
no hooks, no optional index refresh and no external diff/filter execution.
Missing objects or unmaterialized LFS content are preflight failures, not implicit
network fetches. Submodules and symlinked source entries are unsupported in this
first format and fail admission with exact paths.

Runtime authority must distinguish the editable tree, read-only Git object
storage and Engine-only metadata/artifacts. A `workspace-write` string or path
check alone does not establish containment. Each adapter must demonstrate the
required boundary with conformance tests on its actual execution path before
managed coding is enabled. Unsupported capabilities fail before launch.
The scope is controlled local file edits, not adversarial arbitrary effects.
Git worktrees alone cannot enforce this boundary.

The provider cannot write the authoritative state DB, candidate store or verifier
receipts. Engine execution does not admit Git commits, index changes, pushes,
repository configuration changes, or shared metadata mutation in this slice.
Changing those identities blocks acceptance and records the observed violation.
Allowed repository commands and tests run under the admitted process authority;
a prompt instruction is not the enforcement mechanism.

Persist intent before launch. Persist the dispatch intent before the external
call, then record the process handle when known. Observe or quiesce the previous
process tree before any new writer, candidate capture or ownership transfer.
If quiescence cannot be established, retain the workspace and block mutation.
Provider terminal output alone is insufficient if surviving descendants can write.

## Candidate capture and publication

Publish exact bytes, not only a diffstat or Git status. A candidate uses a
versioned canonical sorted manifest whose entries name immutable content blobs
and executable mode. Its base is a captured immutable source manifest, so review
and reconstruction do not depend on an alive provider session, temporary worktree,
or later Git garbage collection. A human-readable diff is derived from those
manifests; it is not the authority.

Capture regular files by descriptor without following links. Reject multiply
linked writable source files in this first format; a source hardlink must not
provide another writer outside the admitted tree. Include tracked
files, additions, deletions and executable-bit changes. Apply one explicit,
pinned exclusion policy for disposable build output; do not let a provider edit
.gitignore to hide evidence. Detect changes outside writable scope. Revalidate
file/root identity and the observation before publication. Concurrent mutation,
unsupported entries, identity drift or limits produce a bounded blocked receipt,
not a truncated successful candidate. Preserve the source workspace for review.

Proposed default caps: 100,000 paths, 64 MiB per file, 512 MiB total captured
source bytes, and 1 MiB per manifest. Both path and encoded-manifest limits apply;
a tree can fit the path count and still be rejected for manifest size. Policy may lower these limits. Admission
must also fit the existing run/task storage budget; defaults do not grant extra
storage. Retained base, candidate and log artifacts are accounted for under a
specified deduplication rule: task budgets count all referenced bytes; the run
budget counts each identical blob once, and counts every distinct manifest and
receipt. Sharing a blob does not bypass an individual task's storage limit. Quiescence and artifact verification remain required even
when content hashes are already present in the store.

Publish blobs and manifest durably first, then commit the state reference and
its event in one SQLite transaction. A crash in between leaves an unreferenced
immutable artifact, which is safe to reuse by digest. A DB reference to absent
or corrupt bytes blocks recovery. Do not add garbage collection in this round.

## Verification and task acceptance

Required checks are pinned by the caller and included in the run receipt. Managed
coding requires at least one check; an empty required-check set fails admission.
An agent cannot satisfy or replace a check by reporting that it ran. Checks use structured argv and a relative cwd;
there is no implicit shell interpolation. Pin the executable identity, toolchain
identity where available, approved environment values and relevant dependency
inputs. Secret values must not be placed in receipts or exposed via plain hashes;
a secret-dependent check is outside the deterministic reuse guarantee.

Run checks in a separate reconstruction of the candidate with source read-only
and declared disposable output directories. Verification may write build output,
but cannot modify the candidate or its receipt. A check needing writable source
must fail preflight in this slice. Network-dependent checks are not cacheable
here and are not part of the initial acceptance contract.

Record process completion separately from verdict:

- zero exit under the declared check contract gives `passed`;
- a nonzero test exit gives `failed`;
- launch failure, timeout, lost output or unknown completion gives
  `infrastructure_failure` or `indeterminate`, never `passed`;
- candidate/tool/environment mismatch makes a receipt inapplicable (`stale`).

This establishes declared checks passing, not mathematical or universal code
correctness. Identical text output alone is not a reuse key. Resume revalidates
receipt and artifact identities before reuse. Re-execution is permitted only
for verifiers whose effects are confined to disposable output. Persist verifier
intent before launch and completion evidence before admitting a retry.

A managed coding task is accepted only when its required mutation receipt is
admissible, its candidate bytes are valid, its ResultEnvelope is valid and all
required checks passed for that candidate. The automatic reducer cannot waive
these requirements. Editing source creates a different candidate and invalidates
prior checks for acceptance. Failed verification preserves the candidate and
failure evidence; automatic semantic repair is deferred.

## Recovery table

| Persisted/observed situation | Required behavior |
| --- | --- |
| Allocation prepared, destination absent | Complete the same allocation |
| Mutation prepared, launch definitively absent, baseline matches | Dispatch same logical effect under current ownership |
| Launch outcome unknown or process still live | Observe/reconcile; do not start a replacement writer |
| Process quiescent, edits exist, no candidate receipt | Capture the observed candidate before deciding further work |
| Candidate published, DB reference missing | Revalidate and finish publication for the same intent |
| Candidate retained, final agent result missing | Preserve it; do not invent success or blindly repeat the edit |
| Verifier completed, acceptance missing | Recover its receipt, validate identities, then accept if all gates pass |
| Check failed | Preserve candidate and diagnostics; present repair as future work |
| Workspace/artifact identity drift, missing evidence, unknown writer | Block automatically; preserve all available evidence |
| Cancellation requested | Stop/quiesce, capture when safe, remain cancelled; no implicit continuation |
| Another controller holds ownership | Wait; stale controller decisions must fail CAS |

A content match to the baseline is evidence about current bytes, not proof that
no effect occurred. `unchanged` does not authorize repeating an arbitrary external
action. Authority/policy changes never mutate a pinned run during resume.

Automatic continuation is permitted only at a verified checkpoint boundary with
matching candidate and explicit pending-work intent, after quiescence, within the
existing continuation budget. It is continuation of the same task, not acceptance
of a new objective. An agent's progress note alone cannot establish that boundary.
For partial edits without such a boundary, record `review_required`; expose the
candidate and packet but do not auto-launch another coding activity in round 2.

A continuation packet has at most 64 KiB of canonical JSON plus immutable evidence
references. Facts established by the Engine and notes claimed by the agent occupy
separate fields. It is constructed from pinned inputs and observed receipts, never
by asking a new model to summarize a missing conversation. A new provider session
can consume the same packet when session reuse is unavailable. The current one-CLI-
continuation allowance remains authoritative; this round does not widen it.

## Execution and review interface

Extend existing `workflow run` policy input and `workflow resume`; do not create
an alternative coding executor. Keep `inspect` and `explain` read-only. Add
`workflow review RUN [--task TASK]` as a read-only candidate/verification view with
text and versioned JSON, progressive detail and bounded evidence pagination.

Inspection must show:

- workspace identity and whether any writer is known active or unknown;
- admitted effect and actual recovery disposition;
- retained candidate, changed paths and exact artifact references;
- each required check, verdict and why reuse is or is not applicable;
- root failure, blocked dependents and remaining continuation/budget state;
- the next permitted transition and its persisted prerequisites.

A review can succeed from the DB and artifact store after the runtime/workspace
has disappeared. A missing artifact must be explicit, with metadata still
inspectable; never silently substitute current worktree bytes. Raw provider
transcripts remain optional drill-down evidence. Report version 2 for the new
wire shape. Add `--report-version 1|2` to inspect/explain, defaulting to 1 for
compatibility; version 2 exposes the new records. The new review command defaults
to version 2. Do not silently change an established output contract.

Example target output:

```text
Task parser: review_required
Candidate C7 retained; 2 files changed
Writer: quiescent, interruption receipt Q2
Required check parser-tests: failed on C7, exit 1
Failure: expected 3 records, received 2; evidence V4/stdout
Next: review C7 and V4; no automatic semantic repair admitted
```

Record the recovery decision and the state transition it authorizes atomically.
Execution rechecks state version, ownership epoch and current authority before
applying it. A later observation that invalidates the decision produces a new
blocked decision, not silent fallthrough. Inspection renders these facts; it does
not persist its own inferred permission to execute.

## Compatibility and migration

Existing read-only and scratch runs continue under their pinned semantics.
Introduce a new execution-receipt version for managed coding; do not reinterpret
old receipts or silently convert their scratch directories into repositories.
Old GitWorktree/CopyOnWrite labels are inspectable but do not acquire the new
mutation guarantees. Managed mutation requires the new contract; requesting it
without a supported policy/adapter fails preflight. CopyOnWrite remains unsupported
for managed coding.

State migration adds dedicated tables and typed event payloads. Existing
inspection's read-only opener never performs this migration. Normal write-capable
opening performs a validated migration; tests cover old-run recovery and explicit
errors from inspection of unmigrated state. Unknown versions fail closed.

## Implementation sequence and owned boundaries

| Slice | Files/modules | Deliverable |
| --- | --- | --- |
| 1. Policy and persisted identities | harp-contracts, ExecutionPlan/receipt, harp-state migration | Validated policy, workspace/effect rows, CAS/events; no runtime launch without admission |
| 2. Managed workspace | new harp-engine workspace module, runtime authority and CLI process validation | Real isolated cwd, journaled allocation, enforced authority, quiescence contract |
| 3. Candidate publication | harp-artifacts and Engine reconciliation | Bounded reconstructible candidates, mutation receipts, crash-safe publication |
| 4. Verification | Engine check orchestration and supervised activity adapter | Exact candidate-bound checks and acceptance gating |
| 5. Recovery and review | recovery.rs, inspection.rs, checkpoint projection, CLI | Resume table, continuation packet, read-only review, actionable failure evidence |
| 6. Freeze and land | tests, maintained guide, generated corpus/Atlas, import receipt | Independent review, full gate, coherent commits |

Slices are implementation order inside this round. Do not land an intermediate
behavior that advertises managed coding before its authority and recovery checks
exist. Keep new behavior unavailable until the complete vertical slice passes.
Search/update Kata before creating implementation issues; this specification is
not a record that implementation has been authorized or completed.

## Acceptance tests

Use real disposable Git repositories, actual SQLite reopen, real file mutations,
and supervised fake provider processes. Inject abrupt process exit at named
boundaries. Assert persisted state and file contents, not source-text patterns.

1. Two sibling writers have different source and index/admin authorities.
2. Provider cwd is the admitted repository, not attempt scratch.
3. Unsupported adapter authority is rejected before any provider process starts.
4. Dirty-operator-baseline requests, missing objects, unsupported entries and
   out-of-scope paths fail with actionable diagnostics and no operator mutation.
5. Crash before dispatch repeats the same admitted intent only when non-launch
   and baseline identity are established.
6. Crash after a file write preserves exact additions/deletions/modes; no second
   edit activity starts before quiescence and reconciliation.
7. Equal Git status with different file bytes yields different candidates.
8. Lost launch response and live descendants never produce two concurrent writers.
9. Crash between artifact publication and DB reference reuses the same candidate.
10. Crash after verification completion preserves or safely reconciles the check;
    accepted results are not duplicated and unnecessary editing is not repeated.
11. A passing check for C1 cannot accept edited C2 or a changed verifier environment.
12. Timeouts, malformed output and unknown check completion never count as success.
13. Matching verified checkpoints can continue in a fresh session within the
    existing allowance; partial edits without one remain review-required.
14. Missing/corrupt candidate blobs, root swaps, hardlinks and concurrent mutation
    produce bounded blocked evidence without cleanup or false acceptance.
15. Stale ownership epochs and stale recovery decisions cannot launch work.
16. Cancellation preserves safe-to-capture work and never implicitly resumes it.
17. Review works after runtime/workspace loss, identifies missing artifacts, pages
    evidence and does not change DB or artifact bytes.
18. An out-of-band successful process/result without required check receipts cannot
    bypass the managed-coding acceptance gate.
19. Existing read-only/scratch runs, old receipt parsing and inspection v1 remain
    covered by regression tests.

The final demonstration starts with the parser fixture and presents the same
reviewable candidate/check evidence for uninterrupted and crash-recovered runs.
Compare candidate bytes and accepted results; record provider edit invocations,
verifier invocations, lost work and recovery disposition. Do not claim exactly-once
external effects, broad adversarial isolation, model quality gains, or test-driven
correctness beyond the specified checks.

Focused verification targets are the new workspace/effect tests, artifact/state
recovery tests and CLI fixtures. After freezing the candidate, run independent
review and `RUST_TEST_THREADS=2 mise run verify`. This changes test concurrency,
not the checks. Follow the pinned Lean cache discipline; do not hydrate caches.
Refresh the import receipt after all canonical/derived payload is settled, and
rerun the relevant repository verifier. No push or merge is part of this spec.
