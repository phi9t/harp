# Independent discovery critique

Date: 2026-10-01.
Reviewer: separate-context `broker_rfc_critique`, read-only, given RFC/analysis
and research paths without the earlier design conversation.

Verdict: feasible as discovery; not yet a build specification.

## Findings and disposition

| Finding | Correction / decision owner |
| --- | --- |
| Database fencing cannot prevent arbitrary old filesystem writes | Qualified the recovery matrix; Define shared-effect enforcement owns the actual containment/credential design |
| Checking ownership before I/O leaves a dispatch race | Added durable exclusive dispatch and uncertainty requirements; Define dispatch reconciliation owns the protocol |
| Markdown and human integration policies conflict with implicit migration | Explicitly retain current policy; Reconcile broker authority owns required versioned changes |
| Lost thread creation may leave no thread ID | Separate thread/start uncertainty from turn/start; Define the live runtime probe contract owns proof |
| Temporal ingress requires a complete bridge protocol | Enumerated Workflow identity, start races, cursors, terminal delivery and rollover; Define the workflow bridge owns resolution |
| Fake acceptance and live recovery evidence were blurred | Added explicit proof class to each recovery-matrix row |
| Decision inventory was not yet concrete | Created one named decision ticket per sharp question, with blockers |

The preserved source RFC was unchanged. These corrections qualify the companion
analysis; they do not assert that the open protocols are now implemented or
approved. Further design critique and adversarial spec review apply when the
first build specification is drafted.

## Locally checked record

- Source body compared exactly with the user's message in the local session
  record, excluding the invocation prefix and capture instruction only.
- Research reports copied unchanged from their owned research worktrees.
- Selected generated public schemas parsed and retained with file hashes.
- No model/runtime experiments, shared-service changes or Git commits ran.

Final artifact checks passed: the ten-section RFC body matches the original
user-message body exactly; all 13 retained schema files match their manifest
hashes and parse as JSON; local Markdown links in the 19 new RFC/effort pages
resolve; both integrated research reports match their source files byte-for-byte;
tracker lint passes for both efforts and whitespace checks report no errors.
These checks establish document/evidence integrity, not runtime correctness.

## Detailed specification review

Independent reviewer `/root/spec_review` read SPEC.md and the build graph.
Two blocking findings were addressed: restore epochs now fence all mutation
and dispatch grants and inventory post-backup orphan jobs; physical runtime
thread identity has unique authoritative control across binding aliases. Both
have explicit observable acceptance fixtures. Optional jj review is based on
pinned primary documentation, not an exercised backend.

The reviewer reread both corrections and the optional jj contract and reported
no remaining material specification blocker. This clears the document review,
not owner decisions or runtime acceptance. Local links, tracker lint and
whitespace checks passed; the preserved RFC body still matches its recorded
SHA-256. No product behavior or live backend guarantees were tested.

## Implementation plan review

Independent review of PLAN.md and tickets found two dependency gaps. Ticket 14
now waits for authoritative submission ticket 11. Rollout ticket 24 now waits
for conditional Kata ticket 18 and explicit jj implementation/acceptance ticket
25. Deferred adapters resolve those tickets with documented exclusions; adoption
requires real backend evidence. No owner-decision bypass was found.

## jj bounded preflight content review

Primary separately read the worker's exact four-artifact frozen package and
verified its hashes. Standards: explicit clean environment, pinned executable,
owned disposable repositories, finite command deadlines, checked errors and
cleanup, no inherited hooks/credentials or shared checkout mutation. Spec for
this limited preparation: actual stable-change/different-commit-and-tree red,
exact-identity green, ignored synthetic file exclusion, independent bundle
import and shared-store metadata rewrite all have command output/IDs in retained
JSON. Fresh integrated Python compilation and tracker lint passed. This is not
independent backend19 acceptance; its capsule/crash/gate/GC/landing requirements
remain untested and no adopt/defer decision is inferred. Deleted fixture bundles
are explicitly historical observations, not available published artifacts.

## Temporal13 repair1 independent review

The frozen56-file package passed both axes against accepted12 plus the ticket
contract. Separate Spec context confirmed family-aware transition identity,
transactional workflow fencing after the runtime mutex, and durable actual
Temporal Run-ID/incarnation observation. Separate Standards context confirmed
scheduler-only wakeup acknowledgement/delivery, strict live-before-SIGKILL and
signal9 evidence, and adjacent authority/history hygiene. Both reviewers
verified every frozen hash. No blocking findings remain in repair1.

At that review boundary, root copied the exact package and started fresh
integration checks. Those later passed and ticket13 was accepted; the joint
current boundary is ticket14's accepted manifest.
The dispatch claim cannot revoke an RPC already in flight; this does not prove
live Trae or multi-host recovery.

## Mailbox14 acceptance and joint integration

[Spec round0](evidence/ticket14-spec-review-round0.md) and
[Standards round0](evidence/ticket14-standards-review-round0.md) independently
verified the frozen138-file candidate against accepted13 and passed.
[Separate shared-glue review](evidence/ticket14-shared-glue-review.md) confirmed
Supervisor behavior/source and lockfile preservation in four root-authored files.
Fresh root gate passed138 Rust and114 Python tests; formatting and strict
all-target Clippy passed. The [joint187-file manifest](evidence/ticket14-accepted-manifest.json)
fixes the accepted boundary. Positional-row proof maintainability is a
nonblocking heuristic concern. Provider/Supervisor physical role-neutral assembly,
actual live behavior and restore readiness remain unproven later milestones.
