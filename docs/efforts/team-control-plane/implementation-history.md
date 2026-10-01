# Implementation history — superseded snapshots

This preserves the earlier chronological handoffs. Older status and next-action
claims are historical; [STATE.md](STATE.md) is the current execution boundary.

# State: durable team control plane

Status: implementing approved baseline; full goal not yet achieved.
Repository read: `963f58b31e518a8162dfe719e386ef9d9843321f`.
Working branch: `codex/reference-design` in the isolated design worktree.

## Where we are

The owner's full proposal is preserved as RFC 0001 with a byte-verified body
checksum. Companion analysis maps existing tools, proposed modules, storage and
workflow ownership, recovery hazards and phased acceptance. Two research
reports cite official durability sources and the installed Trae public schema.
Selected public schemas are retained with per-file hashes. An independent
critique was reconciled in the implementation analysis; important unknowns
remain represented by decision tickets rather than assumed solved.

The baseline is now approved and tickets10–12 are accepted; later sections
retain chronological planning history. The earlier reference quality effort
remains preserved; no policy files have changed. All new records
remain uncommitted pending explicit Git authority. This state is a human-authored
planning handoff, not an `effort.py dehydrate` snapshot or a committed checkpoint.

## Historical next actions (superseded by approved baseline)

1. Resolve Choose the broker/workflow stack, Define the live runtime probe
   contract and Reconcile broker/Git authority with the owner. Async preference
   questions are pending in the current session, not yet recorded as decisions.
2. Resolve dispatch/reconciliation, workflow bridge and shared-effect enforcement.
3. Select the first durability milestone, then hand off to a reviewed tracker
   specification and implementation tickets. Approve probes before execution.

## Traps and limits

- Schemas expose interfaces; detach/restart/cancellation guarantees are untested.
- Thread/turn starts have no inspected explicit idempotency key. Experimental
  native workflow idempotency must not be extrapolated to them.
- A gateway epoch, journal record or lease does not stop old filesystem effects.
- Current Markdown intent and human integration rules remain effective.
- PostgreSQL/Temporal atomicity does not extend to app-server or shell effects.
- No shared runtime was started, no paid model experiment ran and no services
  were provisioned. Temporal/Docker are on PATH; PostgreSQL binaries were not
  found on PATH. Service health and deployment availability are unverified.

## Detailed specification update

Owner requested a detailed implementation specification and high-level goal.
[SPEC.md](SPEC.md) and [implementation-breakdown.md](implementation-breakdown.md)
now provide concrete domain/API/durability/acceptance contracts and a proposed
build graph. Kata and jj were independently requested as reuse candidates and
have pinned-source assessments under research/. Neither is a kernel dependency.
Independent review found restore credential fencing and physical-thread alias
ownership gaps; the draft now requires recovery-epoch grants, orphan inventory
and unique physical-thread control with explicit failure fixtures. Architecture
decisions remain proposed; no implementation, migration, commits or deployment.

## Detailed plan handoff

Owner requested the detailed plan and goal. [PLAN.md](PLAN.md) sequences
16 canonical implementation/evaluation tickets (10–25), each with requirements,
exclusions, blockers, interfaces and behavioral evidence. Existing decisions
03–09 remain prerequisites, not assumed resolved. No product implementation or
Git/deployment actions performed.

## Active goal continuation — 2026-10-01T06:45:28Z

Read the owner attachment carrying the full persistent-team objective and
PLAN.md milestones. Full implementation, real runtime evidence, both lead
demonstrations and restore readiness remain required; planning is not completion.
The previous planning turn made progress by creating/reviewing the dependency
graph. This turn starts executable preflight, without assuming unresolved owner
architecture decisions have been approved.

- Primary workspace unchanged: codex/reference-design, base 963f58b.
- Existing repository gate passed: tools/agents/check.sh exit 0, 71 Rust and
  114 Python tests. Worker tests emitted subprocess ResourceWarnings. See
  evidence/20261001-preimplementation-baseline.json; this is baseline proof only.
- Docker daemon reachable (26.1.4); Temporal CLI 1.7.0 reports server 1.31.0
  and UI 2.49.1. Host psql absent. These are availability observations only.
- Independent model-free Temporal feasibility spike completed; worker/server stopped.
- Independent isolated PostgreSQL/Rust fixture feasibility spike completed;
  container removed. Both reports distinguish exercised guarantees from docs.
- Owner baseline choice requested: Rust broker/gateway and feasible Rust
  workflow worker (Python fallback); Git-approved planning with PG execution
  authority; real PG/Temporal fault kernel before live Trae. Await response.
- No live paid model probe, mainline edit, commit, push or production deployment.

Next: collect spike and baseline results, record owner decisions, then execute
canonical tickets with visible red/green public behavior and independent review.

## Preflight results

[PostgreSQL feasibility](research/postgres-feasibility.md) proves a pinned Rust
client can connect to a disposable real PG17.6 fixture and observe rollback,
unique-key rejection and capacity locking; container cleanup verified. This
is not admission-policy or crash-durability proof. [Supervisor preflight](research/supervisor-preflight.md) records 21 passing actual capsule tests and
the missing complete-descendant stop proof. Current cgroup is not delegated
writable; containment enforcement remains an explicit implementation concern.

## Temporal feasibility result

[Temporal feasibility](research/temporal-feasibility.md) proves pinned SDK1.0.0
compilation, worker SIGKILL recovery, Continue-As-New and actual offline replay
of both runs. Removing the timer failed replay with nondeterminism. Direct
CLI JSON is incompatible with SDK history parsing; SDK history export worked.
Six hashed toy proof artifacts/lockfile retained under evidence/temporal-feasibility.
Dedicated worker/server cleanup verified, and root independently confirmed
TCP18233 closed. No product workflow or production server-restart acceptance
is implied. Owner stack/authority/first-slice choice remains pending.

## Continuation audit: owner decision unchanged

Previous goal turn: progress (real PostgreSQL/Temporal feasibility and baseline
behavioral evidence). Current attachment and decision tickets were reread.
No owner baseline answer is recorded; ticket 03 still requires the owner stack
choice and ticket 05 still requires the planning/execution authority choice.
The recommended baseline is concrete in SPEC/PLAN; further product execution
cannot be admitted through ticket 10 without those decisions. Model-free spikes
are complete, with resources stopped and retained proof hashes checked. There
is no live process being waited on and no reason to rerun the same probes.
This is the second consecutive goal turn encountering the same owner-decision
blocker. The full implementation objective remains unachieved and active.

## Blocked goal audit

Third consecutive goal turn encounters the same missing owner baseline choice.
The prior continuation made no implementation progress; it revalidated the
blocker and corrected stale spike status. Current attachment, worktree and
tickets 03/05/10 confirm no decision change. Existing authorized model-free
feasibility work is finished; there is no remaining live probe to wait on.
All product tickets still depend on the unresolved architectural prerequisites.
Mark the goal blocked, not complete or paused. Resume when the owner selects
the proposed Rust broker/gateway/workflow stack (Python fallback), Git-approved
planning/PG execution authority and PG/Temporal fault-kernel-first boundary,
or provides alternatives. Keep the complete multi-runtime and restore scope.

## Owner baseline approved — 2026-10-01T07:20:06Z

Owner: “yes take your recommended baseline”. Decision03/05/06/07/08/09
resolved for the kernel contract; live04 remains open. See
[approved baseline](decisions/20261001-approved-baseline.md). Start ticket10
with isolated Rust/PostgreSQL behavioral implementation and independent review.
No repeated baseline approval request is needed.

## Execution handle

Ticket10 implementer: /root/postgres_harness, product worktree
/data02/home/philip.yang/workspace/corenius-kernel-admission, branch
codex/kernel-admission at963f58b. Primary owns this tracker; implementer owns
only its product checkout. Session-local brief and interface micro-plan exist.
Follow up on that same agent/worktree; do not restart from absence of stdout.
Fresh independent review and safe file integration precede ticket resolution.

## Current implementation checkpoint

Baseline approval supersedes the earlier pending-choice next actions and blocked
audit. Ticket10 has a real Rust/PostgreSQL candidate in its own isolated worktree;
it has not been accepted or copied into this worktree. Initial independent review
found three Spec blockers and a typed event-identity collision risk. Repair round
1 of 3 is active with the implementing agent. The durable execution ledger records
the findings and integration rule. Full goal remains incomplete; live probes,
model budgets, subsequent kernel tickets and restore demonstrations remain open.

## Admission accepted; next slices active

Ticket10 resolved after repair round1 passed independent Standards/Spec review
and root's fresh integration gate: 16 real PostgreSQL, 114 Python, 71 Rust tests.
Exact reviewed content is integrated only in codex/reference-design; accepted
manifest retained. No commits, pushes or human mainline integration performed.
Tickets11/12 claimed and assigned to separate isolated worktrees seeded from
that manifest: codex/artifact-acceptance and codex/gateway-journal. Artifact
publication/verified acceptance and fault-runtime journal/reconciliation are
next; no live provider or Temporal lifecycle claim yet.

## Kata evaluation progress

Ticket17 claimed; isolated pinned build and real remote API/coldSQLite restore
probe completed, with cleanup verified. Report and hashed evidence retained.
It is not accepted: upgrade, PG restore and broker projection evidence remain
missing; no adoption decision or tracker migration occurred. Tickets11/12
implementers remain active.

## Current accepted boundary

Tickets10/11 resolved. Artifact11 repair1 passed both independentaxes androot
fresh211-testintegrationgate; exactaccepted10+11contentmanifest retained.
Gateway12repair2 corrects faultfixture hiddennative dedup/invocation counts;
notaccepted. Kata17Spec review retains concreteID/digest/revision mapping and
brokerduplicateeventauthority proof as requirements. Gatewayimplementerowns
Kataworktree now, safelycheckpointedaccepted10ownedbrokerbuildwithnoliveservices
whilefixingownGateway. No commits/push/mainlineintegration; fullgoalactive.

## Next kernel implementation boundary

Accepted10/11/12 source is frozen in evidence/ticket12-accepted-manifest.json.
Fresh joint gate passed 215 tests; formatting and clippy passed. Ticket13
Temporal bridge is assigned in its own hash-seeded worktree. Ticket15 supervision
is claimed and seeded, awaiting its implementer’s independent Kata review handoff.
Kata17 repair1 adds real broker mapping/replay/projection proof and is under
separate Standards/Spec review. No Kata adoption or broader completion claim.
New agent threads are unavailable; retained separate contexts are reused with
bounded briefs and isolated trees. No commits, push, live model calls or production
deployment authorized or performed. Full goal remains active.

## Kata evaluation accepted; next durability proofs active

Ticket17 is resolved with exact14-file accepted manifest. Real source/broker
identity/replay/projection and restore evidence survived review; repair2 added
exact Docker ownership/storage/port/ID checks before destructive fixture restore.
Fresh negative and positive probes passed; all four owned containers removed.
Both axes passed. Kata is explicitly deferred within the approved baseline,
with conditional18 disposition recorded pending kernel16.

Temporal13 has a realPG failing registry fixture and is implementing deterministic
workflows/atomic command dispositions. Supervisor15 is active in its own tree,
implementing durable registration and PIDnamespace membership/stop proof.
A read-only [live Trae contract](research/live-trae-probe-contract.md) now records
proposed private entry, bounded probe matrix and launch prerequisites; no live
server/model launched and04/20 remain open. Full objective remains incomplete.

## Parallel kernel and optional-backend preparation

Temporal13 and supervisor15 have actual realPG red fixtures and active isolated
implementations. Separate read-only mailbox preparation records required Lead
generation fencing and atomic presentation/yield/wakeup seams before14 starts.
A source-pinned jj0.44.0 executable built successfully with hostRust1.95; the
PATH name is an unrelatedSSHhelper and is excluded. Pinned release differs from
the earlier staticread's Rust1.97.1-requiring source; both identities are explicit.
Model-free jj identity preflight runs in its own tree;19 remains blocked on15/16
for full backend evidence. No provider or production launch, Git commit or push.

## jj limited executable evidence retained

Actual0.44 fixture shows changeID stays while commit/tree rewrite, synthetic
ignoredfile stays excluded, independent Gitbundle import retains oldbytes, and
peer workspace can rewrite another change's metadata. Exactmanifest retained;
root reviewed scoped source/evidence and fresh syntax. These are historical
fixture observations; bundles were removed, no backend19 acceptance or adoption.
Temporal13 now offers an explicit operationdriver extension that uses stable
gateway intents and retains unknown outcomes; unconfigured driver is a named
milestone, never providercompletion. Both13/15 remain pending independent review
and root integration despite focused worker greens.

## Supervisor15 initial review boundary

Candidate froze with focused10 and fullgate green, but Spec review found five
blocking authority/path/launch/file-scope/operator-recovery gaps. Exact findings
retained in evidence/ticket15-review-initial.md. No supervisor acceptance or root
product integration. Standards review precedes consolidated repair1; three rounds
maximum, none consumed yet. Temporal13 candidate final gate/evidence still pending.

## Temporal13 frozen for independent review

Hash-verified43-file package freezes accepted12 baseline plus declared13 paths.
ActualSDK/PG evidence includes replyloss retryattempt2 with one admission,
SIGKILL/replacement, unsignaled timer recovery, CAN and three offline-replayed
histories. Registered gatewaydriver queries stableintentjournals and keeps
OutcomeUnknown withoutanotherphysicalinvocation. Fresh workerfullgate/fmt/clippy
reported0; independentSpec/Standardscontexts nowreview exactfrozencontent.
No rootproductintegration/13acceptance. Supervisor15 is awaiting consolidated
reviewfindings beforeboundedrepair1. Bothgoalrequirements remainunproven.

## Consolidated kernel repair1 assigned

Both review axes completed for13 and15.13 has five blockers (family identity,
queued dispatch workflow fence, durableRunID, scheduler-only relay, exactSIGKILL
proof);15 has six including the five Spec gaps plus failclosed schema startup.
Both authors received one consolidated repair1of3 brief, actual regression and
fullgate/freshfreeze requirements. Initial exact source/history evidence retained;
no rootproductintegration or kernelcompletion inferred from earliergreen checks.

## Preparation-survivor gap included in repair1

Root plus separate Spec review confirmed seventh15 source gap: creator loss
during actualGit preparation can orphan a writing child while registered state
misclassifies stopped/refundable. Author includes durable Preparing intent and
actual child-boundary crash regression in the existing consolidated repair1.
Unknown prep quarantines, retaining capacity/no deletion. Initial source proof
is explicitly not an experimental reproduction; upcoming fixture must prove it.

## Temporal13 repair1 reviewed; root integration underway

Both independent axes PASS on the exact56-file frozen package. Root hash-copied
it into its owned tree; fresh integration checks are running, acceptance pending.
Supervisor15 resolved its short-command race from a measured Linux ESRCH at
process-membership observation; focused19 and bounded32-command concurrent
regression pass, author finalgate running before fresh freeze/reviews. Full
objective remains active, uncommitted, with no provider/model launch.

## Temporal13 accepted after fresh root checks

Exact56-file reviewed source integrated without glue edits. Root fullgate0
(221 passing tests plus one ignored OS child helper), fmt0, clippy0. Accepted
manifest binds source hashes, review axes and fresh evidence.13 resolved;14
unblocked. Single-host model-free SDK/PG proof does not establish provider
completion, multi-host failover or in-flight RPC revocation.15 remains pending
author freeze and independent review. Full goal remains active.

## Supervisor15 repair1 frozen; mailbox14 started

39-file hash-verified accepted12-relative supervisor package retained in root
scratch. Author finalfocused19/fullgate/fmt/clippy0; actual32-command concurrent
regression fixes measured ESRCH only, preserving unknown/permission quarantine.
Independent Standards and Spec re-review assigned; no15integration/acceptance.
14 claimed codex-mailbox and isolated new tree seeded exactaccepted13. Gateway
context begins14 only after its bounded read-only15Standardsreview. New schema0006
reserved, supervisor0005 stays separately owned. Retained-context fallback
continues because fresh agent contexts cannot be allocated. No commits/models.

Supervisor15repair1 independent Spec PASS with all39hashes verified. Standards
review remains pending; no15rootproductcopy or acceptance. Pending joint13/15
glue plan preserves Temporal dependencies, execute transaction seam and all
migrations; separate review/fresh gate mandatory.

## Supervisor15 repair2 required; mailbox14 executing

Standardsrepair1 found boolean alive() hides EACCES/EIO/malformed secondstat
failures asabsence, allowing inventoryempty/Stopped/refund. Root confirms exact
source; author receives one consolidatedrepair2of3 for fallible authoritative
observations and minimized meaningfulregression. Specrepair1PASS does not waive
thisblocker. No15integration/acceptance. Gateway completesreview then starts
14 independently in seededaccepted13 mailboxworktree;0006reserved. Previous
goalturn classifiedprogress:13accepted withfresh221testgate and14unblocked.

## Live preparation advanced without launch

Separate context retained20complete initialization/config/read/list/background
contracts and installedTRAE_HOME/privateUnix/daemonavoidance evidence. Root
read report and hashverified22files beforecopy; separate manifestretained.
No server/model/provider credential or globalconfigaccess. Exact synthetic
provider routing and hardordinaryturnspend ceiling remainunproven;04staysopen.
Supervisorrepair2classifierred/green retainsEACCES/EIO/malformed uncertainty;
firstfocused18/19 exposes namespaceproofrace, diagnosisactive/no gatefreeze.
Mailbox14 realPG HTTP red401expected/404actual retained beforeimplementation.

## Supervisor namespace reuse measured

Previous goalturn classifiedprogress: Standards blockerchanged repairaction,
mailbox publicHTTPrealPG redretained, and22hashverified livepreflightartifacts
integrated. Current author discriminator shows oldinit1733023/start26777873
namespace4026539861 absent while newerPID1733061/start26777881 reportsNSpid1.
Root readexactdiagnostic3; a newerPID1 is not originaldescendant, confirming
namespace inode reuse ratherthan simple stale scan. Verified gatednamespaceFD
pin repairauthorized withinround2; nopatch/gateacceptance inferred. Current
identity/fallibleobservation and monitorlossquarantine requirements remain.

## Supervisor15 finalrepair3 assigned

Round2 frozen46-file package: StandardsPASS, SpecBLOCKING externalreconcile
cachedoldproof mayoutlive solemonitorFD, allowingnewnamespace signals. Root
confirmssourceordering; notexperimentalraceclaim. Author finalrepair3of3 must
pin currentrecordedincarnation independently through externalinventory/signals
orquarantine; explicitmonitorinternalpin mayproveits own shutdown. Deterministic
boundaryregression andretainedactual19/32capsulematrix needed. No15integration.
Anyremainingblockingfinding after3 requires architecturalattention, nofourthpatch.
Mailbox14 duplicatebudget planningepisode publicHTTPredgreen nowretained; role
credentialsremainseparate. Fullgoalactive withmultipleavailablein-scope actions.

## Finalrepair3 recovery contract settled within approved baseline

Independent primaryLinuxv5.15 source +3boundedactualbwraplaunches (one mount
setupfail,twosuccesses) support finalized ORIGINAL PID1/group exit certificate.
GatedactualchildNSpid1; controllerSIGKILL followed boundpidfdreadiness/statgone
andstoppedwrites. Vendor5.15.152 caveat explicit; no Dstate/thread/permission
race claimed. Rootreadprimarykernel sources; nslink disappearance is early,
leaderZ alone insufficient. Finalrepair3 approvedimplementationdirection now:
persist verifiedgatedPID1origin, separatelyprovefinalgroup/reapedexit ORhold
independentverifiednamespaceFD through externalinventory/signals. No saved
inode/cachedbool substitutes; unknownquarantines, controllerlive maydefer
without effects. Strictcontrollerloss/32tests retained, no smallerrecoverygoal.
Author sharedproductiondecision regression alreadyred (replacement signal1
expected0); no repairedcandidate/gate/acceptance inferred.

## Recovery diagnosis retained and identity assumptions explicit

Rootread/hashcopied three frozenPID1diagnosis artifacts+manifest; freshprobe
syntax0 without rerun. Finalexit proof must retain same observerPIDframe,
bootID and correct /proc/getpid relationship, separately from namespace-link
errors. Savedcontroller namespace binds original observerframe; changed/unproven
frame mustnot turn hiddenoriginalPID into falseStopped. Author receives these
constraints withinfinalrepair3 before implementationfreeze. Fullgoal remains
unproven; remainingkernel/live/demos/restore tasks are not waived.
