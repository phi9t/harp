# Reference-quality workflow specification

Status: draft for owner review; design approved, implementation not authorized.
Triage: ready-for-agent after owner confirms test seams and ticket breakdown.
Source design: `reference-design.md`; accepted decisions in `decisions/20261001-*`.
Repository inspected: `963f58b31e518a8162dfe719e386ef9d9843321f`.

## Problem Statement

Corenius owns the workspace's agentic-engineering practice, but a new lead
must reconcile overlapping guidance, stale claims and manual operational
checks. The exported Claude session demonstrates useful tooling and also
dispatch contradictions, incomplete cancellation, unsafe GC eligibility and
false verification results. Neither a CLEAN verdict nor narrated success is
enough to establish correctness. The owner needs a reference another Codex or
Claude lead can follow and prove through an actual effort and fresh-session
recovery.

## Solution

Maintain one shared lead/worker workflow, executable lifecycle controls,
focused guidance and small harness adapters. Validate the reference with
independent Codex and Claude demonstrations of the same improvement from a
common pinned base. Retain auditable results and clear limitations.

### Requirement IDs

- R1: current guidance and effective decisions agree; historical claims are qualified.
- R2: one structured launch contract is authoritative and validated before effects.
- R3: cancellation terminates owned execution across launcher and capsule sessions.
- R4: finish/GC preserve active or uncertain worktrees, including concurrent launches.
- R5: acceptance verification rejects defective but execution-compliant output.
- R6: both lead adapters follow the same authority, budget and completion contract.
- R7: independent demonstrations and fresh-session recovery produce retained evidence.
- R8: exact-tip gates, independent review and human integration authority are preserved.

## User Stories

1. As the owner, I want one entry point, so I can identify current practice.
2. As a new lead, I want explicit dependencies, so I can distinguish missing setup from failures.
3. As a Codex lead, I want a harness adapter, so I can follow the shared contract.
4. As a Claude lead, I want an equivalent adapter, so completion means the same thing.
5. As a lead, I want immutable recorded launch inputs, so I can inspect what a worker received.
6. As a worker, I want one writable-path declaration, so inconsistent dispatch does not reach me.
7. As a lead, I want bad contracts rejected before effects, so rejected launches consume no model budget.
8. As a lead, I want bounded concurrent launches and attempts, so retries cannot grow spending silently.
9. As a lead, I want owned processes cancelled completely, so workers do not survive cleanup.
10. As another session, I want cancellation to preserve my processes, so shared-host work is protected.
11. As a lead, I want uncertain process identity to cause refusal, so a reused PID is not signalled.
12. As an operator, I want active worktrees protected, so clean git state is not mistaken for inactivity.
13. As a lead, I want execution compliance separate from acceptance, so CLEAN output still receives review.
14. As a verifier, I want negative fixtures, so a weak verifier cannot pass by accepting everything.
15. As a reviewer, I want pinned source and artifact identities, so evidence identifies the inspected result.
16. As a fresh lead session, I want effective decisions and next actions, so I can resume without prior chat.
17. As the owner, I want both leads tested independently, so a guide is not proven by one harness alone.
18. As the owner, I want limits on the demonstration's claims, so usability evidence is not sold as reliability statistics.
19. As a reference reader, I want corrections to supersede prior claims, so historical errors remain traceable.
20. As a maintainer, I want policy changes versioned and tested, so sibling copies have a clear source.

## Implementation Decisions

### Authority and lifecycle terms

Execution compliance concerns the observed launch contract and process behavior.
Artifact acceptance concerns independently reproduced required behavior.
Integration verification concerns the assembled exact tip. These are separate
results and must never be inferred from one another.

The workflow permits inspection, preparation, verification and cancellation of
owned workers within approved authority. Commit and push need explicit session
authorization. Current canonical policy requires human mainline integration.
Prior profile defaults that suggest agent integration must be reconciled with
that policy through a superseding, versioned decision; this spec does not grant
integration authority.

Preparing a gated landing means recording the candidate tip, successful gate
result and reviewer handoff while leaving the real mainline unchanged. The
existing no-push landing command is not a preparation command: it still moves
local mainline. Integration scenarios run against disposable repositories only.
The fresh-session recovery demonstration requires explicit authority for local
implementation and dehydration commits before its live phase starts. Otherwise
the live phase is blocked and reported incomplete.

### Structured dispatch (R2)

Add a strict contract mode to the existing worker-launch CLI. Its versioned
JSON record includes a task identity and body, exact base commit, worker name,
writable paths, model/reasoning configuration, capsule identity, time cap and
effort-budget identity. The task body is supplied through the contract, not a
second competing task file. Preserve the existing launch mode for compatibility
and label it legacy/advisory in guidance; the reference workflow uses strict
mode exclusively.

Contract validation rejects unsupported versions, missing/unknown fields,
duplicate or ambiguous paths, absolute paths, parent traversal, paths resolving
outside the workspace, symlink escapes, invalid limits and unavailable pinned
inputs. Directory permissions are explicit rather than inferred from arbitrary
prose. Legacy command flags that contradict contract fields are rejected;
matching redundant declarations may be accepted only with identical normalized
meaning. Do not claim to detect every contradiction in free-form prose.

Validation and reservation precede branch/worktree creation, output placeholders,
provider execution and charging a model attempt. Store the normalized contract
and its digest with the run; supervision and evidence consume the same record.
If setup fails after reservation, release it, preserve a diagnostic and remove
only setup artifacts whose ownership and unchanged contents are established.

### Budgets and concurrency (R2, R6)

Per lead demonstration: two initial implementation workers, at most two active
workers at any time, cap 1800 seconds per worker, at most three repair rounds
per initial ticket, and at most eight total worker invocations. An invocation
is charged when the provider process is started, including cancelled or failed
invocations. Rejected preflight contracts are recorded but do not consume one.
Replacements count and do not reset the original ticket's repair counter.
Reservations and counters use durable effort identity and survive lead restart.
Concurrent launches cannot both reserve the final slot or exceed active limits.

Persist a launch intent/reservation before provider startup. Fault recovery at
each transition (reservation, process creation, identity registration and
terminal write) must prevent uncounted execution or duplicate charging. An
unknown outcome after possible startup consumes an attempt conservatively and
keeps active capacity reserved until reconciled. Only evidence of no provider
startup permits an attempt refund. A crashed launcher never makes its tree
retirable solely because its own PID disappeared. Inject crashes before and
after spawn and before identity registration in public-CLI fixtures.

Both demonstrations together share a two-worker host concurrency ceiling.
Review budget per lead demonstration: two separate reviewer invocations per
worker ticket (Standards and Spec) and one independent adversarial
lifecycle/assembled-branch review, five invocations per pass. Repeat only
after a fix for at most three repair rounds, at most 20 invocations per lead.
Record actual invocation counts;
do not multiply hidden supervisor or nested-review agents. In-session design
critique and spec review are planning work, separately recorded.

Lead-session budget: one initial implementation session and one fresh recovery
session per harness, cap two hours per implementation session and 30 minutes
per recovery session. Nested lead agents and replacement lead sessions require
additional authorization. A review invocation is capped at 30 minutes. Record
lead, worker, relay and reviewer invocations separately; the worker limit is
not represented as a dollar or whole-program token limit. Optional relay calls
consume reviewer slots instead of receiving a hidden additional allowance.

Increase limits only with explicit authorization. Token usage is measured when
available; missing usage and dollar costs are marked unavailable. Worker model
configuration is pinned in each demonstration record; accepted budget defaults
are not themselves authorization to start paid model processes.

### Owned cancellation (R3)

Add a cancel operation to the worker CLI. A run records immutable execution
identity adequate to distinguish PID reuse and host reboot, and ownership of
the launcher, capsule runner and containment boundary. The repository's capsule
runner starts a separate session; signalling only the outer group is inadequate.
Use the existing PID namespace boundary to contain worker descendants and
verify shutdown of the outer capsule and launcher as well. Track run lifecycle
and resource ownership independently of a last-message file.

Cancellation verifies identity before signalling, requests termination, then
escalates after a bounded grace period (five seconds). Success means owned
non-zombie processes and the contained worker namespace are gone; supervisor
and cancellation records agree on a terminal cancelled outcome. Repeated
cancellation of a confirmed terminal run is safe and idempotent. Cancellation
is not CLEAN completion. Missing identity, inconsistent records or inability to
prove ownership/shutdown cause a non-success outcome and retain resources for
inspection. Never signal an unrelated reused PID or delete its worktree.

The 1800-second cap invokes this same owned-shutdown path, including nested
sessions, instead of merely timing out the launcher. Expiry records timed-out
rather than cancelled/completed. A simultaneous timeout and explicit cancel
may not overwrite terminal evidence or release active slots until shutdown is
confirmed. Failed or uncertain shutdown remains active/unknown for budget
and retirement purposes and requires escalation. Use stable OS process handles
where supported, or refuse unsafe signalling; a start-time check followed by
an unprotected numeric-PID signal does not close the identity-reuse race.

Reference launches require capsule containment. Legacy uncontained runs may
receive best-effort cancellation with explicit limitations; no broad guarantee
is made for arbitrary host processes escaping their session or namespace.

The strict reference mode uses an Insula read-only worktree with writable binds
derived from the validated allowed paths. Declared paths are also monitored;
monitoring is not the enforcement mechanism. Capsule evidence records the
measured content identity and the declared request separately. An unmeasured
host-dependent layer remains explicitly unmeasured; no digest label converts
host mounts into a reproducible image claim.

### Safe retirement (R4)

Apply the same active-run guard to finish and GC, including dry-run selection.
An active owned run, matching live process using the worktree, or unknown run
ownership excludes automatic retirement. Observe process use on the supported
Linux host and report visibility/permission limitations instead of treating
incomplete inspection as inactivity. A stale run record with a reused PID
protects unrelated processes and remains unknown until reconciled.

Share lifecycle exclusion between launch registration and retirement. Check
eligibility again while holding the relevant exclusion before removing the
tree. The actual launch operation creates a new worker worktree, so the public
race is the interval between tree creation and active-run registration: GC
must not remove that newly created, clean, merged tree. Reservation protects
the target path before creation and through registration. Reusing an existing
worktree for launch is outside scope. Preserve existing dirty/unmerged/main-worktree protections. Automated
retirement guarantees cover cooperating toolbox launches. Document that an
uncooperative external process can race an OS scan; do not claim universal
host-process exclusion. GC remains dry-run by default.

### Verification and evidence (R5, R7, R8)

Acceptance fixtures exercise the public CLI with real temporary git repos,
fake providers and real child processes. At least one fake provider produces
an allowed output and exits successfully but violates the task's semantic
acceptance; the lead verifier must reject it. A correct fixture passes the
same verifier. Controlled failures run on disposable owned resources.

Retain tracked redacted proof for accepted claims: contracts and digests,
pinned commits/capsules, exact commands and exit results, relevant failure
diagnostics, rejected artifacts or reproducible fixtures, review findings,
corrections and subsequent results. Keep raw event logs locally for at least
30 days after acceptance, recording location and deadline. Deletion is manual
and explicitly authorized. Redaction occurs before Git staging; include no
credentials, private prompt contents or unnecessary user data. Record raw-log
loss without claiming replay from unavailable material. Hashes identify bytes,
not correctness or independently authenticated provenance.

### Guidance and adapters (R1, R6)

Provide a main entry point, shared operating guide, execution-boundary guide,
verification/recovery reference and Codex/Claude adapters. Prefer pointers to
existing canonical rules over duplication. Each practice states its trigger,
action, completion criterion and evidence/limitations. Commands are verified
against installed help and behavioral fixtures; installed help alone does not
prove a live model run. Record CLI versions and host requirements.

Adapters differ only in harness entry, notifications, review invocation and
fresh-session initialization. Deterministic supervision is mandatory; an LLM
relay is optional, read-only and charged to a declared budget. Leads verify
results and retain landing authority according to the explicit session/repo
contract. Existing Rust prototype commands remain discoverable.

Correct small-benchmark totals to 39/40 based on the archived rows. Reconcile
effective feature state and supersede conflicting authority summaries. Milano
claims are resolved by reading pinned source and distinct launch paths, not by
choosing whichever narration is newer. Leave unreproduced claims qualified.

### Independent demonstrations (R7)

Prepare a common target base containing the agreed contracts, requirements, guidance
for bootstrapping and deterministic fixtures, but no lifecycle solution. Each
lead starts independently from that exact commit with two disjoint worker
assignments: dispatch validation/budgets, and cancellation/retirement. Assign
separate implementation/test surfaces; the lead owns CLI integration glue.
Shared interfaces are fixed before dispatch. No worker owns a shared mutable
checkout, and neither lead receives the other implementation or outcome.

Separately pin the execution-tool baseline used to launch these implementation
workers. It includes existing F12 enforced writable binds and PID containment,
but not either candidate's strict contract, cancel or retirement solution.
Before live dispatch, offline probes must verify this baseline's write boundary,
hard timeout and owned emergency shutdown across capsule sessions. The outer
controller retains process identities and protects all demonstration worktrees
from retirement. Manual lead validation, attempt accounting and monitoring are
explicit bootstrap duties, not evidence that the new controls already work.
If the baseline cannot safely contain and stop a worker, live dispatch is
blocked until that prerequisite is repaired separately and reviewed. Pin both
execution and target baselines in evidence. Candidate acceptance invokes the
candidate's public operations, not the outer controller's emergency shutdown.

The candidate solutions are competing results; the selected reviewed solution
alone becomes the integration candidate. Final operational guidance is verified
against that selected implementation. Bootstrap guidance names limitations of
the legacy launcher until its replacement passes checks. Lead-controlled
watching and cleanup may be needed before the new controls are available; these
actions are recorded rather than represented as proof of the new controls.

Compare acceptance coverage, safety findings and reproducibility of evidence;
do not rank general model quality or performance from one demonstration.
Evidence is labeled with its lead and candidate identity, including the
unselected candidate, so results from competing implementations are not mixed.

Each lead plans, dispatches two real traecli workers, independently verifies,
rejects a controlled defective artifact, exercises dispatch refusal and complete
cancellation, proves retirement protection, prepares an exact-tip gated handoff,
dehydrates and resumes in a fresh harness session. Implementation tickets own
candidate proof plus the committed dehydration handoff; two separate recovery
tickets each own the one fresh-session check for their lead. Recovery runs
once per lead, and depends only on that lead's proof. The fresh session receives
only repository entry instructions and the durable effort identifier, not prior
conversation or another lead's result. It must correctly identify effective
decisions, candidate identity, completed checks, rejected artifacts, remaining
authority and next action. No transcript-resume shortcut satisfies this check.

One complete run per lead demonstrates initial usability, not statistical
reliability, general fleet superiority or proven containment against all attacks.
Unavailable authentication or a required permission produces a blocked result,
never a simulated live pass. No real mainline movement or remote push is needed
to demonstrate preparation; disposable repos test landing safety.

## Testing Decisions

Proposed primary seam for owner confirmation: the existing command-line tools
operating on disposable real git repos, real subprocesses and a fake provider.
This extends the existing worker, worktree, landing and effort tests. Avoid
tests that inspect source text or mock the safety rule being tested.

- Dispatch: invalid contracts have no branch/tree/provider side effects; valid
  contracts produce exact outputs and recorded identities; two simultaneous
  reservations cannot exceed limits; restarted leads preserve attempts;
  fault injection proves conservative recovery before/after spawn and identity
  registration without uncounted execution or unsafe capacity release.
- Cancellation: nested launcher/capsule sessions, TERM-resistant descendants,
  cap expiry through owned shutdown, timeout/cancel races,
  natural exit/cancel races, repeated cancel, stale identity, PID reuse, missing
  metadata and permission failures. Assert unrelated processes survive.
  Exercise an identity change between validation and signalling and at least
  one real bubblewrap PID-namespace run; a fake session alone is insufficient.
- Retirement: clean merged active tree, dirty/unmerged tree, unknown ownership,
  delayed registration after worktree creation racing GC, process visibility
  failure and terminal run.
- Acceptance: correct artifact passes and deliberately defective CLEAN artifact
  fails; tracked and untracked output tampering are detected.
- Landing: disposable bare remote exercises failed gate, rejected push and
  push-first ordering. Real mainline and other agents' files remain unchanged.
- Recovery: committed durable context survives scratch loss; new sessions
  correctly reconstruct next actions and report unavailable raw evidence.
- Guidance: locally verify examples and adapter entry behavior; live checks
  stay explicitly separated from offline fixtures.

Focused suites precede the repository gate. Include benchmark/capsule suites
when relevant and policy-sync tests for policy edits. External sibling drift
does not silently change the scope to sibling repairs. Independently review
security/concurrency/lifecycle behavior and review each ticket on Standards
and Spec; record findings, repairs and fresh results.

## Out of Scope

Sibling rollout, remote creation/push, automatic human integration, unrelated
Rust-runtime expansion, GPU scheduling, cross-host portability, exhaustive
adversarial containment, automatic log deletion and comparative performance
claims. Do not execute live leads/workers or auto-committing dehydration until
the required session authority is granted.

Execution authorization must also cover creating the common pinned target-base
commit and any reviewed execution-tool baseline repair.

## Further Notes

Publication is in the existing tracked effort. This draft awaits owner review
of the behavioral seam and proposed ticket graph. Independent critique and
adversarial spec review must be reconciled before execution readiness.
