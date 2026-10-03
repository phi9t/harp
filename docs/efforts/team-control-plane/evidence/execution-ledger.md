# Implementation execution ledger

Goal: full PLAN.md persistent team reference; no smaller completion criterion.
Base: `963f58b31e518a8162dfe719e386ef9d9843321f`

## Authority and baseline

Owner explicitly approved recommended baseline. Decision is recorded in
[approved-baseline](../decisions/20261001-approved-baseline.md). Use real PG,
Rust Temporal and Git planning with durable execution authority in PG. Git
commit/push and production deploy are not granted. Live model budgets remain
a separate pre-launch prerequisite. No repeated baseline question is needed.

## Active slice

- Ticket10 claimed to codex-kernel; isolated implementer checkout
  `/data02/home/philip.yang/workspace/corenius-kernel-admission` at fixed base.
- Standalone external broker crate, public command/read/HTTP seam and real
  PostgreSQL behavioral fixtures. Source interface plan is session-local.
- No runtime starts in this slice. Prepared operation intent and reservations
  must commit atomically with authoritative ownership and command result.
- Red/green evidence and separate Standards/Spec reviewer pending.

## Integration rule

A worker owns its checkout. Primary never edits it. After the implementer is
finished, copy only reviewed product paths into the primary isolated checkout,
verify exact hashes, run required checks and retain a fixed review package.
No shared branch rewrite or commit is used as an integration shortcut.
Subsequent workers receive an immutable content manifest of accepted predecessor
files because there is no implicit commit authority. Diverging patches are
reviewed against that exact manifest, rather than HEAD alone.

## Rulings

- Kernel decisions06/08 no longer wait on live probe ticket04: the owner approved
  fault-kernel-first, while actual provider semantics remain gated by04/20/21.
  Cost if wrong: graph rework; no live guarantee or model authority is gained.
- Initial single-host admission uses consistent serialized PG transaction locking.
  Correctness is the acceptance criterion; throughput scalability is not claimed.
  Cost if wrong: storage-lock strategy can change after measured contention,
  preserving public contracts and existing concurrency fixtures.

## Ticket10 independent review — repair round 1 of 3

Frozen candidate: `.scratch/team-control-plane/execution/review10/manifest.json`.
Independent Spec review identified three acceptance blockers: authorized missing-task
assignment rejection is not durable; rejected-command reads bypass narrowed scope;
dependency edits are absent from approved specification digests and reconstructible
history. Standards review identified a separate typed-aggregate event-key collision
risk and two nonblocking readability concerns. The implementer received all findings
and must demonstrate failing regressions, scoped fixes, and fresh required checks.
No candidate has been integrated or accepted. Review concerns about absent runtime
execution are outside ticket10 and do not change its admission-only scope.

## Ticket10 accepted

Round1 re-review passed on both axes. Exact16-file candidate copied into root
isolated checkout and manifest hashes verified. Fresh root integration gate
exit0:16realPG/114Python/71Rust. Accepted manifest retained alongside report.
No runtime/restore claim, Git commit or mainline mutation. Tickets11/12 unblocked.

## Tickets11/12 execution

Separate implementers own codex/artifact-acceptance and codex/gateway-journal,
seeded from the exact ticket10 accepted manifest plus canonical effort docs.
New modules/migrations/tests are assigned separately (artifacts0002/gateway0003);
shared command/storage glue must be reported explicitly for primary integration.
Root tracker claims are codex-artifacts/codex-gateway. Both must retain visible
public behavioral red/green and fresh checks before independent review. Live
models, production runtime changes and Git commits remain unauthorized.

## Ticket17 executable evaluation in progress

Primary owns codex/kata-evaluation after agentthreadlimit rejectedresearch
dispatch. ExactpinnedKata built with checksumverifiedisolatedGo1.27.1.
PublicHTTPfixture scope404,expiry403,claimrace200/409,close200withunexecuted
testclaim,exactclosereplayreceipt,andcoldSQLiterestoreexecuted. Daemons
stopped/listenerclosed/tempstoresremoved. Probe/results/manifest/reportretained.
No adoption; upgrade/PGrestore/brokerprojection evidence still missing.

## Current goal continuation — progress

Previousturn produced pinnedKataAPI/coldrestoreevidence. Thisturn adds actual
forwardSQLitemigrationv0.14.3->pinnedbuild(API0.10.0->0.24.0), retentions
andcredentialread/claimrefusal proven; bothprocesses stopped. Artifact11
candidate frozenandhashed; independentSpecreview dispatched toexisting
design_critiquecontext. Gateway12 addedactualOS SIGKILLboundaryfixtures
perrootrequirement; finalgate/sourcefreeze pending. No fullgoalcompletion.

## Ticket11 review — repair round1 of3

Independent Spec review of frozen5-filecandidate found two blockers. Verifier
uses Lead credential retaining AssignAttempt authority despite distinct-grant
contract; implementer must introduce verifier-specific authority with denial
fixtures. RealGit preparation checks are disconnected from PGacceptance, while
changed-base acceptance uses syntheticIDs/reusedartifact/log. Requireactual
Gitcandidate bytes and executedlogs through publication/submit/verify/accept,
then actualbasechange rejectsoldproof and freshcandidate/freshchecks accepted.
Round1 routed to sameimplementer. Standards review still pending; candidate
notintegrated/accepted. No contractwaiver or ownerapprovalrepeat.

## Ticket12 review — repair round1 of3

IndependentSpecfoundmalformedcreate/observationthreadIDs bypassbindvalidation:
Someemptyorover1024bytes becomesconfirmed/adopted. Implementermustprovered,
thenquarantinewithoutadoption/subsequentStart. ActualsixOS SIGKILLboundaries
anddurableclaim/identityownershipotherwisereviewedcoherent. Standards pending.
Ticket11initialStandards passed; excludesforthcomingrole/Gitproofrepair.
Bothcandidatesremainunaccepted andunintegrated.

## Ticket11 reviewed integration; ticket12 repair2

Ticket11repair1 bothaxespassed: dedicatedVerifierrole/coremigrateentrypoint,
actualretainedGitbundle independentchecks+PGacceptance. Eightreviewedsource
fileshashchecked/copiedintoroot; freshintegrationgate process97063 started.
`roborev review --branch --base 963f58b --local` refused: no commits onbranch.
No commitauthority implied; separatecontextfixedbase+exactcontentreviews are
recordedfallback, not claimedroborevpass.

Ticket12Standards foundfaultfixture native-dedupfalse claimcontradictedby
exclusiveopIDeffectfiles+swallowedsubprocessfailure. Repair2of3 mustmakeevery
invocationobservable/trulynonidempotent andassertcounts acrosscrashmatrix.
No duplicateimplementationbug observed; existingcounts insufficientproof.
Kata17safecheckpoint: ownhashverifiedaccepted10brokerbuilt, no livefixtures,
newprojectionprobe notauthoredyet. Gatewayimplementerresumedownrepair2.

## Ticket11 accepted

Freshrootintegrationgate process97063 terminalexit0/allgreen:10artifactGitPG,
16admissionPG,114Python,71Rust. Bothaxespassedrepair1. Accepted10+11
contentmanifestretained. No commits/push/mainlinechange orbroadercompletionclaim.

## Ticket12 accepted after joint integration

Repair2 bothaxespassed; copiedsixnewexactfiles, mergedexisting11lib/storage.
Primaryinitialglue passedDSNinsteadtransaction; compilegatefailedexit1 and
independentreviewcaught mismatch. Thirdboundedcorrection (2workerrepairs+1
gluefix) useslockedfreshtransactionafterartifactmigration; reviewpassed.
Freshcorrectedgate8215exit0/allgreen:30PGfixtures+114Python+71Rust=215tests.
Fmt/clippyexit0. Exactcombinedaccepted10/11/12manifestretained. Nofurther
repairbudgetimplicitlyextended. No runtime/Temporal/productioncompletionclaim.

## Ticket17 evaluated and deferred

Repair1 added real approved source binding and duplicate event/projection proof.
Repair2 closed inherited unsafe PG targetguard after real public red/green and
identity drift checks; separate Standards/Spec contexts passed. Four owned
fixtures removed; models0. Exact14files captured in ticket17-accepted-manifest.json.
Pinned Kata remains optional; accepted evaluation does not implement18 or add
ledger authority. Full goal remains active with13/15 implementations running.

## jj toolchain and identity preparation

Pinned release0.44.0 source af45d57de7163cc5f17f3df178a31577e4951ea3 built
--locked with Rust1.95 exit0; sourceclean verified. Earlier mainread requires
Rust1.97.1, so release selection and separate identities are explicit. A separate
worker proved stablechange/differentcommit+tree, ignoredfile exclusion, exact
bundle import after rewrite and peer sharedstore rewrite. Root verified frozen
source/evidence hashes and fresh integrated syntax.19 still blocked15/16; no
adoption, conversion, productionartifact, capsule/crash/gate/GC/landing claim.

Temporal13 repair1: both independent axes PASS, exact frozen56-source integration
with no edits, fresh root fullgate/fmt/clippy all0.221 tests pass; one child helper
ignored and exercised by parent. Accepted manifest: ticket13-accepted-manifest.json.
No model call, commit, push or live provider launch.

Supervisor15 finalrepair3 accepted: independentSpec/StandardsPASS on64frozen
files; rootintake preserved11rawlogs matchingauthorhashes. Exact3-sourcecopy
andfivefilejoint13/15glue independentlyPASS; freshroot245testgate/fmt/clippy0.
Accepted100-filemanifest records source andprovenance. No liveprovider/model,
realworktreecommit/push/PR/deployment;16stillblocked14.

## Kernel16 read-only integration preparation

The isolated future kernel-proof tree was seeded from all100 accepted15 files.
The retained independent context produced a source-grounded integration map;
root verified its33 declared input hashes, note digest and provenance digest,
then exact-copied only the two research artifacts. No product source or ticket
status changed. Implementation remains blocked on mailbox14 acceptance.
Role-neutral Execution allocation, actual SDK driver assembly and operator
reconciliation remain explicit16 work; proposed fixture budgets are not approval.

## Live provider configuration source boundary

Separate retained research context located current first-party TOML and legacy
enterprise YAML documentation. Only the latter describes custom model endpoints;
no exact installed0.207.1 routing/protocol or hard ordinary-turn spend limit is
proved. Root read both new artifacts, verified their frozen digests and report
link, and copied exactly2 research files. No runtime/config/provider handshake
ran. Further equivalent help/schema searches stop pending new primary evidence;
04/20 stay open and kernel/mailbox implementation continues independently.

## Restore inventory source preflight retained

Root read the frozen source-grounded report, checked23 input hashes and2 prior
preflight hashes, then exact-copied2 new research artifacts. Accepted15 source
remains unchanged. Current PG-owned registries do not provide discovery/adoption
of post-backup jobs absent an older restored DB; immutable bytes alone cannot
reconstruct erased reference retention. Future24 independent inventory, fresh
epoch/reissue checkpoint and dispatch/admission/GC closure remain explicit.
No restore fixture, service, product mutation or ticket acceptance occurred.

## Mailbox14 actual artifact transport regression observed

Root inspected the terminal public mailbox_artifacts logs: HTTP initially404
versus required401; later MCP returned invalid_argument versus HTTP-aligned
unauthorized for Verifier submission. Author reports owned fixtures cleaned
after terminal exit101 and is implementing the shared ArtifactService MCP
registration. These observed reds are not passing transport/verification proof
or14 acceptance. Exact observed logs are retained in session-local intake;
final frozen evidence and both independent review axes remain required.

Mailbox14 artifact transport focused green observed: terminal8176 exit0,
actual_git_submission_and_independent_verification_share_http_mcp_authority
1passed/0failed. Root read assertions for actual retained Git bundle checkout,
commit/tree/base derivation and executed check/log bytes, Worker HTTP submission,
Verifier MCP report and equal unauthorized submission denial across both paths.
No candidate accepted. Exact observed green log joins session-local red intake;
full14 gate/freeze and independent reviews remain pending.

Mailbox14 peer disclosure public red/green observed: distinct WorkerB/taskB
HTTP401 versus required200 became actual exact-byte success. Root read assertions
for unaccepted task candidate sharing, denied peer task/history, denied direct
foreign-task read and denied registered-but-unlisted digest. Checkpoint fabricated
reference200 versus required401 likewise becomes a focused pass. Four exact
terminal logs retained in session-local intake. Peer MCP byte equality and
conditional-turn/Temporal proof remain author work; no14 acceptance is implied.

Mailbox14 runtime-turn binding red/green observed: Owner publicHTTP400 versus
required200 became1passing fixture; Worker tool credential denied401. Actual
subprocess Start has one retained effect and confirmed operation, bound to exact
Worker episode/input digest. Root inspected terminal logs and public assertions.
This proves the binding slice only; physical conditional delivery/races, Lead/
Verifier execution binding16, Temporal and final review remain unproven. Exact
observed logs join session-local intake; no14 acceptance or live-provider claim.

Mailbox14 completion-before-call physical red/green observed: Gateway Confirmed
versus requiredFailedKnown became focused1pass (terminal83639/exit0). Reported
physical counts are2 invocations (Start+conditional) and1 effect (Start); duplicate
dispatch adds no invocation, presentation/ack arrays remain empty. Root read
terminal logs and current public fixture setup, retaining exact observed logs.
Concurrent post-claim race/lost-reply/unsupported/no-native-dedup proof is a
separate live focused handle7552, not inferred complete from this pass.

Mailbox14 physical matrix observed: terminal42394 exit0/2passed. Root read
actual post-claim gate/PG Dispatching observation, completed/raced/drop-reply/
unsupported/delivered assertions and per-invocation/effect counters. Duplicate
journal dispatch adds no call; successful or uncertain I/O creates no presentation/
ack. Raw same-operation calls create2effects, establishing native-dedup=false.
Initial harness starvation is retained by author; transport isolated in blocking
runtime for concurrent PG observation, no product fix inferred from harness alone.
Pending redelivery after authoritative turn end is separate live56480; real
Temporal wait/replay and final14 acceptance remain outstanding.

### Ticket14 frozen candidate: independent review dispatched

Root verified all138 file hashes and byte lengths from frozen round0, then
copied those bytes without product integration to `.scratch/team-control-plane/review14-round0`.
Manifest SHA256 `6062da2afa6ccab06e957c8fd19fc06ba9e542b50188d5c7c5c74651d120bd78`;
baseline13 patch SHA256 `1063c7c9f538ebf6639c85b97395c7621f95393ad1a6475114b1c0aa269dfc98`.
Separate non-author reviewer contexts received Spec and Standards work against
canonical ticket14 and accepted13. This retained-context fallback is required
by the thread creation cap. Both results remain pending; no acceptance claimed.

### Ticket14 root integration checks started

Both independent round0 axes passed the frozen138-file candidate. Root verified
and backed up all100 accepted15 manifest files before mutation; integrated90
exact candidate source/evidence files plus four root-authored shared files.
Supervisor implementation and Cargo.lock remain preserved. Formatting check
passed. A separate shared-glue review was dispatched to gateway12, covering
root-authored changes rather than author14 source conformance.

Fresh full gate is session28959; strict all-target Clippy is session77007.
Logs are `ticket14-root-gate.log` and `ticket14-root-clippy.log`. Both running
at dispatch; no acceptance or passing-check claim yet.

### Ticket14 accepted; full goal remains incomplete

Gate session28959 exited0, ending `check: all green`:138 Rust and114 Python
passes, one parent-exercised helper ignored. Clippy session77007 exited0; fmt0.
Separate shared-glue reviewer PASS verified Supervisor source/schema/tests and
Cargo.lock preservation. Both conformance axes passed the frozen candidate.
Ticket14 resolved with owner codex-mailbox; tracker lint0. Joint manifest has187
files, SHA256 `ab1370237832f6ac7a105a8c235f556aa3e2b758adab5be1c0e6b8a32306c844`.
Ticket16 assembly, optional evaluations, actual live-provider demonstrations and
upgrade/backup/restore readiness remain required; no full-goal completion claim.

### Ticket16 exact baseline confirmed

Isolated author reports all100 prior accepted15 files unchanged before reseed,
all187 accepted14 joint files hash-matching afterward, and prior16/24 research
notes preserved. Role-neutral additive0007 mapping begins before public red.
Root reconciled stale current map/audit/review summaries; tracker lint and
whitespace checks pass. No ticket16 behavioral acceptance is claimed.

### Ticket16 first public behavioral red reported

Author reports `tests/run-postgres.sh --test assembled_kernel` exit101:
`authenticated_operator_status_is_a_public_cli_view_without_execution_ownership`
failed because actual authenticated CLI status returned invalid_argument against
a unique real PostgreSQL schema. Minimal read-only status green is in progress.
This is author-reported first-slice evidence, not frozen or independently reviewed
ticket16 acceptance. Role-neutral additive0007 mapping binds canonical Episode,
Mission, principal/role/input/epoch and optional genuine Task/Attempt; physical
stop authority remains supervised namespace proof. Full integrated SDK driver
and supervised vertical fixture remain required.

Root directly inspected both first-slice logs: red reports actual CLI
invalid_argument and one failed test; green reports the same named public test
passing against PostgreSQL (one pass). Exact log bytes/digests retained in scratch
`execution/kernel16-observed-status`. This proves only the status slice, not
complete ticket16 or frozen candidate conformance.

### Ticket16 role-neutral allocation public red observed

Root directly read the actualCLI/realPG first failing allocation test: a captured,
admitted Lead episode receives invalid_argument instead of a genuine workspace.
The behavioral oracle requires no synthetic Task/Attempt, unchanged Lead tool
role, and no second capacity charge; physical allocation green is not yet proven.
Root flagged inherited Git configuration in the new disposable fixture helper
to the author; fixture commits/checks must disable inherited hooks/helpers and
use explicit inert configuration. Exact red log copy/digest retained in scratch.
No product integration or ticket16 acceptance.

### Ticket16 first role-neutral allocation green observed

Root directly read the new focused log: both actualCLI status and Lead episode
allocation tests pass against real PostgreSQL. Allocation verifies real Git
workspace bytes, zero synthetic Tasks/Attempts, unchanged Lead role, no double
capacity charge, and public cancel/release returns concurrent capacity while
cumulative episode count stays consumed. This is a Preparing/workspace lifecycle
slice, not actual capsule execution or complete SDK-driven verification.
Fixture Git helper now clears inherited environment and explicitly disables
global/system config, hooks and prompts; no HOME variables changed. Additive0007
exists, but full schema/authorization/recovery conformance is not reviewed.
Exact green log bytes/digest retained beside prior focused observations.

### Ticket16 physical allocation versus yield capacity red-green

Root directly inspected realPG/public mailbox/CLI logs and the behavioral oracle.
Red: conversational Yield changed capacity from1 to0 despite an existing physical
episode binding. Green: the same fixture retains1 on Yield, then public cancel
and completed workspace release return0 while admitted_episodes remains1.
Candidate fix checks durable episode_executions membership in the same Yield
transaction. This addresses new16 assembly; full runtime/Gateway outcome and
actual-running capsule proof remain pending. Exact focused log copies/hashes
retained in scratch; no frozen16 review or acceptance claim.

### Ticket16 allocation authorization before filesystem effects

Root inspected new realPG/actualCLI fixture and both logs. Red: invalid token
rejected but workspace-root directory created. Green: same invalid-token request
fails with workspace root still absent; legitimate Lead allocation and prior
status/yield accounting checks pass. Exact focused logs retained with hashes.
This is a bounded startup/authorization slice; role/generation/epoch concurrent
fences, actual capsule and SDK vertical behavior still need complete16 evidence.

### Ticket16 first physical Lead capsule failure observed

Actual Lead capsule fixture failed: public reconciliation reports Quarantined,
reason launch identity and full membership are unknown. The test requires an
executed read-only planning capsule and authoritative stop before release;
workspace allocation green does not satisfy this requirement. Root observed
the initial failure in tool output and directed bounded diagnosis against the
accepted15 gated PID1/monitor path, preserving unknown quarantine and strict
stop proof. Causal origin and green remain unproven; worker confirmed running.

Author localized initial capsule failure to new fixture ordering: reconcile was
called during Starting before monitor committed init_process. Accepted15 fixture
inspects Starting and waits for state transition before authoritative reconcile.
Changing only this state-conditioned test observation produced focused green;
production namespace stop classifier unchanged. Root directly read green log,
retained it with hashes. The purported initial-red file copy raced the
author overwrite and contains green bytes; raw initial failure was not retained. This proves one actual Lead capsule/read-only planning workload;
startup operator observation semantics and full vertical/recovery proof remain
pending. No fresh joint16 gate or review claimed.

### Evidence correction: initial capsule raw log capture raced overwrite

Author independently checked root's misnamed planning-capsule-initial-red.log:
SHA256 `2fb618b9ed185e05f40b8908af7266d4cbdb17e7c3d8a2c4bd31d60673fb6880`
is the green0.43s log, identical to later green copy. Root's initial tool read
did observe failure/quarantine, but a separate subsequent filesystem copy raced
author overwrite. Therefore no retained raw initial-red log is claimed. The
misnamed scratch file remains for audit with explicit non-red classification.
Root corrected earlier ledger statements and notified author; retrospective
diagnosis must stay labeled. Future fault proofs require immutable per-run paths
and exact bytes captured by the observation rather than a later reopened file.

### Ticket16 explicit startup fault boundary focused proof

Root directly inspected retained passing log and test oracle. With explicit
BeforeMonitor fault, three passive inspections remain Starting; reconciliation
reports Quarantined with launch identity/full membership unknown; release fails
and used_capacity stays1. This fault fixture launches no process and deletes only
its private fixture files after asserting public release denial. Prior normal
Lead capsule still executes read-only scope workload and authoritative stop.
Exact new log bytes were read once, checked as green and copied with SHA256;
no reconstructed initial-red claim. Full assembled SDK/operator proof remains
incomplete and unreviewed.

### Ticket16 authenticated operator pause public red

ActualCLI pause with owner credentials fails invalid_argument against realPG.
New fixture requires durable pause to reject later physical start after allocation,
then explicit versioned resume permits the read-only capsule. Root read and
retained distinct red bytes after asserting failed/pause markers. Full transactional
Gateway dispatch fencing and SDK-driven paused workflow proof remain pending.

### Ticket16 pause/resume focused timeout, not green

Root inspected actual bytes despite filename operator-pause-green.log: pause
succeeds but authenticated resume fails TemporarilyUnavailable after5.42s.
No passing claim. Captured distinct failure bytes once with digest. Root sent
a falsifiable fixture-runtime-starvation hypothesis: blocking child CLI output
on current-thread Tokio could prevent dropped transaction rollback from flushing
while child waits for PG advisory lock. This is a hypothesis, not causal proof;
probe must preserve lock timeout and pause enforcement. Full16 remains unaccepted.

Author retained timeout failure separately, then changed only new fixture runtime
to multi_thread/two workers. Same product SQL/fences and assertions pass2 tests
in0.71s. Root inspected and copied distinct run2 green bytes once. This supports
fixture scheduling hypothesis without changing lock timeout or stop proof.
Blocking CLI calls still need async/spawn_blocking cleanup plus fresh focused
verification; broader assembled SDK/dispatch/operator proof remains pending.

### Ticket16 pause fixture async cleanup verified

Author moved CLI subprocess output to spawn_blocking shared helper and restored
original single-thread Tokio fixture. Root verified actual helper/test attributes
and retained distinct green log:2 passing tests, no failure marker. This closes
fixture-runtime dependence without changing production PG lock timeout/fences.

Author is assembling actual SDK Worker BridgeActivities with GatewayOperationDriver,
canonical Worker Episode→Supervisor Execution and exact prepared gateway journals;
deterministic non-deduplicating OS adapter will call authenticated start-execution
for the recorded handle. Planned public seam is run-temporal.sh assembled_vertical,
followed by yield/resume, retained actual Git submission and supervised independent
verifier. This is implementation direction, not executed vertical evidence.

### Ticket16 first actual SDK vertical run failed before readiness

Root directly inspected actual run-temporal assembled_vertical failure: owned
SDK controller did not publish prepared-journal readiness by5s. Full child causal
origin remains unproven; no SDK dispatch/capsule success inferred. Root retained
distinct failed log bytes/hash and requested strict child-liveness/exit checks and
stdout/stderr preservation. Runtime adapter native-turn proof must originate in
actual immutable runtime records rather than a generic effect alias. Full
yield/resume/independent-verification/acceptance remains required.

### Ticket16 actual SDK reaches recorded Start, supervisor CLI missing

Root inspected retained child log: initial helper failed InvalidArgument during
prepare, explaining missing controller readiness (a setup failure). New distinct
SDK run reaches immutable Start journal through registered OperationDriver:
Operation state FailedKnown, supervisor start-execution CLI rejected InvalidArgument.
This supports actual SDK dispatch reachability, not successful capsule or full
vertical lifecycle. Child startup, prepared dispatch and product CLI failures
remain separately classified. Root retained all three actual failed log byte
sets/digests; no green claim. Independent turn identity and complete
yield/resume/submission/verifier/acceptance remain required.

### Ticket16 actual SDK→Gateway→Supervisor Start focused green

Author reports run-temporal.sh assembled_vertical exit0. Root directly read
and retained distinct run3 passing log bytes/hash. Actual registered driver
confirms Create then Start; fixture asserts capsule src/result bytes,
authoritative Supervisor Stopped, two external invocation/effect records,
strict live SDK worker SIGKILL9 and public release. Root inspected authenticated
start-execution CLI seam and kernel status0007 guard. No full16 acceptance:
yield/resume, exact candidate submission and supervised independent verifier,
operator takeover, unknown-runtime capacity and full fault matrix remain pending.

### Ticket16 native-turn/yield green and continuation accounting red

Root directly inspected distinct logs and public fixture oracles. Actual SDK
Create/Start plus BindRuntimeTurn passes against independently retained native
turn/current packet and exact Start journal; yield retains physical capacity1.
New captured-input Worker continuation exposes capacity1 versus required2.
Author reports narrow owns_capacity=!imported fix: assignment imports first
episode reservation once, continuation owns a new slot. Fresh green pending.
No synthetic Task/Attempt, SQL verdict manufacture or full16 acceptance.

Fresh continuation run2 passes actual SDK fixture; root retained passing bytes.
First imported episode stays charged once; continuation raises concurrent slots
1→2; cancel/reconcile/release refunds only its owned slot, cumulative admissions
remain consumed. This proves allocation/accounting, not resumed candidate work.
Next exact-candidate Verifier materialization is explicitly missing: current
create_episode clones approved base rather than submitted candidate bytes.
Author is building the public behavioral red before bound materialization.

### Ticket16 verifier exact-candidate materialization failing seam

Root inspected actualSDK vertical logs and candidate fixture source. Run1 fails
with fixture Option unwrap and is not counted as required behavioral red. Run2
reaches actual capsule-output Git commit/bundle publication, public submission,
Verifier capture/admission/allocation; candidate result path absent in Verifier
workspace, consistent with current admitted-base clone. This is not verified
candidate execution; submitted bundle must be independently fetched/imported
and exact commit/tree/base checked before actual supervised gate. Both failure
logs retained separately with hashes. No task acceptance claim.

### Ticket16 exact submitted candidate materialization green

Root read distinct run3 actualSDK passing log and source seam
create_episode_from_candidate. Scoped snapshot/evidence read fetches immutable
bundle; locked allocation rechecks candidate/spec/generation/digest/current tuple.
Independent workspace Git bundle verify/fetch derives candidate tree and first
parent/base before checkout, then fixture checks actual src/result bytes.
Source is still an unfrozen16 candidate, not independently reviewed acceptance.
Actual Verifier capsule gate/log/verification/acceptance and corrupted-candidate
faults remain required; base clone alone no longer serves as candidate proof.

### Ticket16 assembled candidate/gate/accept fixture and exit-provenance finding

Root inspected and retained actual SDK vertical run1: one passing test, 2.62s;
worker capsule produces candidate bytes, independent Verifier materializes the
submitted Git bundle, runs configured capsule checks, retains output, and public
verification plus Owner acceptance reaches Accepted. Retained log SHA256
`ae6ac9f8b202329201829295b75e6219069efaa9310906bfda69cbe7c350062b`. This is bounded fixture evidence, not ticket16 acceptance.
Root source inspection found run_monitor discards launcher exit results, while
the fixture asserts a printed gate_exit=0 marker and submits exit_code:0.
Actual configured check execution is proven; independent trusted exit/signal
provenance is still required, orthogonal to whole-namespace Stop proof. Author
received the requirement for typed execution-bound status and a nonzero check
with misleading success output negative control. Genuine work resumption,
operator takeover and the complete fault matrix remain pending.

### Ticket16 physical pause and uncertain-runtime capacity regressions

Root inspected distinct public behavioral red/green logs and current fixture
oracles. Paused direct Gateway claim initially invoked runtime once (expected0);
fresh green preserves Prepared with zero RPCs. Lost Start reply independently
records a physical effect while Supervisor proves Stopped; initial public release
incorrectly succeeded. Fresh green denies release with DependencyUnmet, retains
workspace and capacity1. Both release reservation and final refund check
unresolved same-attempt gateway operations. Disposable fixture teardown is
explicitly not product reconciliation or refund. These focused observations do
not establish full ticket16 acceptance. Retained log hashes:

- `paused-public-runtime-claim-red.log`: `f65482dda2488c2078fc4a7b2b6b39b9042ceee4bb13e1ddd90d975822a9d9e2`.
- `paused-public-runtime-claim-run2.log`: `4de8794302171241837cf1879d3957cc19284f417343d9d38f71334647e07b75`.
- `stopped-unknown-runtime-capacity-red.log`: `1455bcd7abcac8d3369ecfc589f041e06ffb2b2df4e4f98363b57cc100d358c9`.
- `stopped-unknown-runtime-capacity-run2.log`: `dfc46e0cc03cf50c9d5c312f0ead2498505fd107f1fb833e8cd3f2e6cb964c9c`.

### Ticket16 delayed monitor exact-issuer expiry regression

Root inspected public monitor red/green: after a one-second issuing Worker grant
expires, another live grant for the same principal must not authorize the delayed
workload. Behavioral red produced forbidden effect; fresh green (1.39s) proves
no effect and DependencyUnmet. Current source binds immutable execution to
credential_id and rechecks that exact credential scope, epoch and expiry before
launch. Focused evidence only; complete role/generation and candidate freshness
matrix plus independent ticket16 review remain pending. Retained hashes:

- `issuing-grant-expiry-monitor-red.log`: `c33ece9e9e57f257ff47c21ce3c3fa0d6de40f445faca5a38310a7f025dec699`.
- `issuing-grant-expiry-monitor-run2.log`: `3f9b379c8593f841c568b0ef1b8ef5070607b6345ba8a8cd4a85a16b35388ebd`.

### Ticket16 trusted wait status independent of misleading output

Root inspected distinct behavioral red and fresh green (0.38s): actual capsule
prints gate_exit=0 but exits17. Trusted monitor now records wait status17, no
signal, immutable argv and separate namespace_stopped fact. Typed result also
binds Execution/candidate/epoch/controller/log digest. This proves observation
independently of stdout, not yet acceptance enforcement: current vertical
verification still supplies fixed0, and negative control does not yet attempt
public acceptance. Author received requirement to bind measured result to gate
evidence and reject contradictory verification/acceptance; Stop/result publication
readiness is explicit. No ticket16 acceptance. Retained hashes:

- `trusted-exit-vs-stdout-red.log`: `3dca904a9d5e10b47c2789897b752d44a027c495ee59e085d5dbdc0ba898d13c`.
- `trusted-exit-vs-stdout-run2.log`: `b3e37c962c3d57034609abd986094f045f2ce1489e78ea67399493cea37e5bd2`.

### Ticket16 measured-exit verification and nonzero acceptance rejection

Root inspected actual SDK run3 passing log (2.91s) and current public fixture.
Successful mandatory gate reports measured monitor exit0 with exact argv,
candidate and log digest, waiting for durable result independently of Stopped.
Separate actual Verifier capsule prints gate_exit=0 but exits17; public Verify
retains measured17, Owner Accept rejects DependencyUnmet and task remains
unaccepted. Separate measured-success report subsequently accepts explicitly.
This closes the bounded fixture stdout/exit evidence gap, not complete assembly
or independent review. Retained log SHA256 `1309b22055457332929987ad5e3db523fd28355d19a78ef6901df371abfcb9f1`.

### Ticket16 report-authority gap remains after honest measured-status fixture

Root current-source inspection: ArtifactService Verify registers referenced log
bytes and caller report; Accept rejects caller-reported nonzero values. Neither
path currently reads kernel_execution_results, and Check has no Execution
binding. Prior run3 honestly supplies measured17: this proves report policy,
not authoritative rejection of fabricated0 contradicting trusted monitor status.
Root requested actual public contradictory-report red and trusted execution-backed
validation of verifier/episode/candidate/command/log/exit/signal at verification
and acceptance. Do not count the honest negative fixture as enforcement against
forged success. Independent assembly acceptance remains pending.

### Ticket16 fabricated-success public behavioral red confirmed

Root inspected distinct initial compiler failure (missing imports; not behavioral
red) and run2 actual SDK public regression (2.90s). Same failed Verifier execution
measured17 is reported as0; public Verify and Owner Accept wrongly accept it.
This directly confirms the authoritative evidence gap; author is implementing
trusted-record comparison and pre-allocation requirement. No green or assembly
acceptance yet. Retained hashes/classifications:

- `fabricated-zero-vs-trusted-seventeen-red.log`: `885a4efdbad7c110fc0f2f6c87d80c11f95213f43c32708b4201a3828c04aa2c`; compiler failure, not behavioral red.
- `fabricated-zero-vs-trusted-seventeen-red-run2.log`: `e7f5da1e1150cd3fcabe4f3f29c3cd811f233a4b51a6fd9290ccffd318729d8c`; decisive public behavioral red: forged zero accepted despite measured seventeen.

### Ticket16 fabricated-zero authoritative comparison focused green

Root inspected fresh run3 passing actual SDK fixture (2.88s), immutable Owner
requirement activated before dispatch, and verify_execution_report DB comparison
of trusted monitor result with task/verifier/credential/episode/candidate/argv/
exit/signal/raw-output digest. Fabricated0 for measured17 is rejected. Requirement
absence retains explicitly legacy unassembled contract. Final assembled entrypoint
must require or atomically enable this policy, with pre-allocation and omission
regressions; fixture opt-in alone cannot prove enforcement for all assembled work.
No frozen16 review or acceptance. Retained log SHA256 `84bc3fb8252c03146b4beff1eb5c7aefe4a7c0dbd8fa057a0be2bc9b075b998c`.

### Ticket16 preallocation report bypass focused green

Root inspected run4 actual SDK passing fixture (3.00s). Before any Verifier
allocation, caller-only success evidence is rejected with DependencyUnmet.
Current role-neutral create_episode source requires enabled supervised policy
for every genuine Task before filesystem allocation, covering both Worker and
Verifier. A distinct omitted-policy public allocation regression is still
required; source guard alone is not behavioral proof. Full resume/operator/
fault matrix and frozen independent review remain incomplete. Retained log
SHA256 `238f3822b922a40f6855953a152bb9200e894d27f539eaa56577b832ae420458`.

### Ticket16 missing-policy allocation negative observed

Root inspected passing public negative (0.24s) and fixture: absent assembled
verification requirement denies create_episode with DependencyUnmet, directory
entries unchanged and zero registered execution rows. Retained log SHA256
`7dbc6f70fe7a67be3869af97a0c25a827ee510518fe5cc7dfd8ecb53274ee111`. This proves the earlier source guard behavior.
Remaining assembly seam: legacy Gateway admission/Attempt uniqueness cannot
honestly represent continuation or role-neutral Lead/Verifier Starts. Author
is designing additive canonical Episode operations under0007, preserving legacy
Worker protocol and no-synthetic-record constraint, with per-Episode Start
identity, prepared journal, current-input/ownership fences and unknown-operation
holds. Actual resumed work and registered SDK role-neutral dispatch remain
unproven; independent takeover quiescence and full fault matrix also pending.

### Ticket16 physical start does not extend admitted deadline

Root inspected distinct public start red/green logs and fixture oracle (0.28s):
start_with_fault BeforeMonitor performs no launch; read-only database comparison
observes execution deadline no later than canonical Episode deadline. Current
transaction uses least(requested launch deadline, admitted Episode deadline).
This proves durable preparation bound, not actual timer cancellation or full
long-running workload shutdown; those remain separate fault requirements.
Retained hashes:

- `episode-deadline-extension-red.log`: `82fa1224e02aeebb155db53ebd38a71d4e43a577c7d55d0316788baaf784e26d`.
- `episode-deadline-extension-run2.log`: `164cd893ba3f6f10f089967d1c2c465895399672b910baaa267eab905f154eba`.

### Ticket16 episode-native public preparation missing seam red

Root inspected actual public CLI regression (0.35s): canonical admitted Worker
Episode and physical Execution exist; prepare-episode-start rejects
invalid_argument because the episode-native journal seam is absent. Fixture
expects Prepared with exact Episode/Execution, no RPC. This red establishes
missing public preparation only, not SDK dispatch or no-synthetic-record proof;
final green must inspect core admission counts and actual role-neutral SDK
effects separately. Retained log SHA256 `0406fdab1db540557284350b2341b89ecc8bef6e640fa37fecd6f0c31c047d2c`.

### Ticket16 episode-native preparation focused green

Root inspected fresh public CLI run2 passing test (0.35s) and current
episode_runtime module. Authenticated preparation persists canonical
Episode/Execution/principal/exact issuing credential/current input and runtime
request before RPC, with per-Episode uniqueness and immutable digest. Current
module exposes preparation only; actual registered SDK routing, cross-legacy
store exclusivity, unknown reconciliation and resumed-work proof remain
incomplete. Test title alone does not prove unchanged admission counts. No
frozen16 review/acceptance. Retained log SHA256 `a4b14337c660ef4545a459b37847a56500be9b44d9e9b1047946ecd60b24fabc`.

### Ticket16 episode preparation no-synthetic-record oracle green

Root inspected run3 passing public preparation fixture (0.38s) and read-only
SQL count oracle: Tasks/Attempts/core operations remain(1,1,1) before/after
canonical Episode journal creation. This now proves the no-synthetic-record
claim for preparation, beyond the prior test title. Physical dispatch/resumption
and role-neutral SDK routing remain separate unproven requirements. Retained
log SHA256 `80901d39c4a398055e62ad0fcbeb0de34656895b575077a6723165d87fa8adbf`.

### Ticket16 canonical Mission progress exposes planning journal

Root inspected actual PG red/green and fixture. Genuine Lead planning Episode
with no Attempt prepares a journal; canonical Mission registry progress now
includes that Episode in waits (fresh green0.35s). Despite test name, no actual
SDK OperationDriver or runtime Start occurs; it proves routing visibility only.
Registered dispatch/physical work, unknown reconciliation and complete takeover
remain pending. Retained hashes:

- `mission-episode-driver-routing-red.log`: `682159fc4a5e2c370b7e29bc512895b66e2f32f5e4fed67484e187315b29ed63`.
- `mission-episode-driver-routing-run2.log`: `e169728524bdc0ddf4b8f2f966251268a1ed2e8370e3b07da8f77263234e1393`.

### Ticket16 episode physical-claim implementation compiles

Root inspected current async EpisodeRuntime dispatch seam and successful
compilation (49.85s). After queue acquisition, PG claim checks original
execution/input/credential/epoch/workflow/pause and shared gateway_stores
controller/busy ownership; commits Dispatching before async runtime call.
Unknown leaves busy held; confirmed/known failure clears it. This is source
inspection plus compilation only, not physical dispatch/recovery proof.
Registered SDK role-neutral and resumed-work tests remain pending. Retained
compile log SHA256 `c96d4468eeac9f97db8b6ae63afc76ea85199aa52ded8081abbb52367b0f675d`.

### Ticket16 actual SDK planning episode dispatch red

Root inspected fresh actual SDK parent/child fixture failure (5.45s) and source.
Genuine Lead Mission planning Episode has prepared journal and physical
workspace; actual registered MissionWorkflow/BridgeActivities worker runs, but
no episode driver is yet attached and trusted Supervisor result never arrives.
Parent confirms live child, explicitly kills/reaps owned worker and cancels/
reconciles/releases unlaunched workspace. This proves missing registered
physical route, not launch success; author implementing composite driver.
Retained log SHA256 `ad531216251aa469e0572a58fd80094ee03d071ee948321e6ed0ba760b52025c`.

### Ticket16 actual SDK Lead planning capsule focused green

Root inspected fresh planning run2 passing log (1.16s) and registered episode
driver fixture. Actual MissionWorkflow SDK child drives prepared Lead Episode
to authenticated supervised capsule; parent obtains trusted exit0 and separate
namespace_stopped. Stronger native invocation/effect count, role/manifest/output
oracles are still being added. This proves a focused planning physical path,
not integrated resumed Worker/Verifier/full lifecycle or takeover/recovery.
Retained log SHA256 `85ff3412759b7846134d93f877ea7d3edfa0e612320479e9608f36e58c2bf212`.

### Ticket16 actual planning capsule captured-context delivery red

Root inspected strengthened actual SDK planning capsule failure (1.51s):
workload attempts to read /episode-input.json and receives FileNotFoundError,
trusted exit1. Prior successful shell launch did not prove captured input was
available to execution. Immutable exact packet materialization/read-only mount
and final launch digest/authority checks are now required before planning or
resumed-work success claim. Root sent this targeted boundary to author.
Retained log SHA256 `3a4880fa65169ec566a5e4be795e48c88843264ac2efe6c0c5ace6ddd86c052a`.

### Ticket16 planning capsule reads captured context focused green

Root inspected actual SDK fresh run2 (1.16s): workload reads episode-input.json
and validates Lead role with null Task/Attempt; trusted exit0, namespaceStop,
exact one native invocation/effect and empty write scope asserted. Current
monitor re-fetches exact captured packet under claim lock, rechecks digest/
524288-byte bound and mounts sealed memfd via bubblewrap ro-bind-data0400.
This closes focused missing input delivery; missing/tampered input and mount
write denial still need negative tests, and full resumed lifecycle remains
incomplete. Retained log SHA256 `63465c5cf63bc42f41ac10ecbb3d1f56744e2f5492496f6ef12b716bc379ee7e`.

### Ticket16 actual restored Worker continuation focused green

Root inspected run2 failure (fixture message body/text mismatch, separate
diagnosis) and run3 actual SDK green3.99s. Continuation independently imports
checkpoint candidate bundle, reads exact src/result and new captured message,
creates src/resumed and records native turn with trusted exit0. This proves
actual continued work, beyond prior allocation/cancel accounting. Current
submission precedes continuation output; later verifier checks original
candidate through direct Supervisor.start. Full lifecycle still must submit
and verify post-resume result through registered episode driver. Root sent
this requirement; recovery/control matrix and independent review remain.
Retained hashes:

- `sdk-actual-restored-worker-continuation-run2.log`: `a3ff82ae341f8fb98c732d27b60e11a553eaaa4ec83da70360be62f23f60302b`.
- `sdk-actual-restored-worker-continuation-run3.log`: `adfcf5406cc451e35679b281027ab6cda10e17ee02c71828801a5dd879e7cfe8`.
- `resume-fixture-text-key-diagnosis.json`: `c26ce345d4ef89402e125db236254a24b10bed4a7e84f4d263143585443108ec`.

### Ticket16 post-resume candidate and SDK Verifier lifecycle focused green

Root inspected fresh passing actualSDK lifecycle run1 (5.95s): candidate tree
includes real src/resumed output, its digest checkpointed/imported for independent
verification, mandatory gate checks both deliverables. Positive and nonzero
misleading-output Verifier capsules launch through prepared Episode journals and
registered SDK driver; authoritative report comparison and explicit Owner
acceptance remain exercised. This is one focused full lifecycle path, not
complete recovery/operator/control matrix, frozen review or kernel acceptance.
Retained log SHA256 `cafb9fbb1207d144a0ddd2c64109cafccd1638227733e12faac991c2a2946b4b`.

### Ticket16 episode-protocol unknown capacity red/green

Root inspected distinct physical episode lost-reply red0.41s/green0.42s logs.
Actual native Start effect survives dropped reply and local capsule Stop;
legacy-only release guard wrongly refunded. Current runtime_release_known now
checks episode/same genuine Attempt unresolved journals at both release phases,
and fresh public negative passes. This extends the focused capacity invariant
to the new episode protocol, not full reconciliation/takeover/crash matrix.
Retained hashes:

- `episode-native-unknown-capacity-red.log`: `fce1a0bf9cfb5f4b22f767f3ccd2355a27e1a9c9ad2a13c6a878ce1a96af8745`.
- `episode-native-unknown-capacity-run2.log`: `fa928be1848751d02256baa6fee95ef98c71df056a539402902f74a623e9ba18`.

### Ticket16 uncertain episode observation missing seam red

Root inspected initial misplaced-oracle compiler failure (not behavioral red)
and actual run2 red0.42s: native Start effect remains OutcomeUnknown when
query-only observation should independently identify it. Current dispatch does
not relaunch, so unsafe retry prevention is retained; authoritative resolution
is missing. Root requested exact correlation and single invocation oracle,
with missing/ambiguous records retaining unknown and no fallback Start.
Retained hashes:

- `episode-native-unknown-observe-red.log`: `15b107eb73a695d5c0fdff2ee16898725a53d987b0d2448c55e9da6e56eae414`.
- `episode-native-unknown-observe-red-run2.log`: `e883c4a178e7538904fd689c2906cfc183fb06b60aca8ea7d30a006c2c72385d`.

### Ticket16 existing-effect query-only resolution focused green

Root inspected fresh run3 green0.42s and fixture oracle: lost Start reply
initially denies release/holds capacity1; repeated dispatch observes existing
native effect, confirms journal and independent invocation count stays1.
Current observe path requires same runtime/store/controller/busy operation;
fixture correlates exact operation/Execution/Episode and rejects unreadable,
malformed or non-unique inventory. This proves same-controller focused
resolution, not replacement-controller takeover, ambiguous inventory negatives
or full crash matrix. Retained log SHA256 `5e7a2e63fc9dcd95d08ec25350fc2f0886df7c65f6ea17c9bb45ebe5da0872fa`.

### Ticket16 missing/ambiguous effect observation negatives and eventual release

Root inspected focused run4 passing bytes and current oracle. Hidden native
effect and duplicate effect both retain OutcomeUnknown; restored unique exact
effect confirms. Independent invocation count remains1 across observations.
Only after confirmation plus proven capsule Stop does public release refund
capacity1→0. Native matching includes runtime/Execution/Episode/packet/digest/
argv. Replacement-controller takeover remains distinct/unproven. Retained
log SHA256 `b1b7bc1dbd9a972101710fd22fa8c885573b6f4362f94ce54bf26862000755d7`.

### Ticket16 operator execution CLI focused red/green

Root inspected public missing inspect red0.33s and fresh green0.48s. Owner
CLI independently inspects canonical ready execution, cancels/reconciles/
releases through Supervisor and refunds capacity; invalid token has no stdout.
Fixture never launches this execution, so it proves operator seams/authority
for Ready only, not running cancellation, crash recovery or positive takeover.
Those remain separate assembly requirements. Retained hashes:

- `operator-execution-commands-red.log`: `1016904ad96bfe8d5a6d6990826dca33e02d99145170640693ac9739ca18c768`.
- `operator-execution-commands-run2.log`: `fafb73e6de5c3b459a2346ef10f138c1c98b3c5034fead6547e2ce7a209dd91f`.

### Ticket16 running capsule operator cancellation

Root inspected terminal focused green0.61s and actual assembled fixture: public
Worker CLI launches foreground and nested setsid/background writers; Owner
CLI cancel/reconcile reaches proven Stopped, both file lengths remain stable,
and public release refunds capacity to0. This extends Ready-only evidence to
a running supervised capsule. It does not establish controller takeover, SDK
crash recovery, or complete failure-matrix acceptance. Retained log SHA256
`c5a12d58ef7d04b95d55ff765fab5486bf18a0b7eecbb8738fc183eda3717ddd`. Ticket16 remains claimed and unfrozen.

### Ticket16 observed execution status

Root inspected decisive missing-execution red0.51s and focused green0.53s.
Public authenticated status exposes Ready execution plus canonical Episode;
source bounds each collection to32 with more flags, filters non-Owner rows,
and includes unresolved journals. This test proves one Owner-visible Ready
row, not pagination, historical-credential isolation, live journal recovery,
or full operator acceptance. Those remain required assembly evidence.

- `operator-observed-status-red.log`: `a3ea1a6b3c4ba9b00725842bb84f647d67111a65d9eb8cb6e6acc5b08842ed1f`.
- `operator-observed-status-run2.log`: `4ec100ddefcb1f50e15c65bf714231b43c7c1cadf066a9dd98a7abea0847f97c`.

### Ticket16 unknown Start blocks continuation before charging

Root inspected decisive red-run3: yielded Worker with prior unknown native
Start incorrectly admitted successor, episodes1→2 and capacity1→2. Focused
run4 green0.52s rejects with DependencyUnmet and unchanged both counters,
while query-only unique-effect observation confirms original Start with only
1 native invocation and final proven Stop permits release. Earlier setup
failures are not substituted for this behavioral red. Scope is one Worker
continuation; full SDK crash/replacement and all-role budget matrix remain
open. Retained evidence:

- `unknown-continuation-admission-red-run3.log`: `9188626b73d295c45a0df643bb030a0e389d32276d931aeb215e2301c6d2fbc7`.
- `unknown-continuation-admission-run4.log`: `c8b7516ae0af8b3a393a3183f042139d853f2495cb4d5c78e61e8ff17f2632c1`.

### Ticket16 takeover registration setup failure

Root inspected terminal failure0.30s: actual SDK helper exits before
registration readiness, so the public takeover command is never reached.
This is setup/seam failure, not a behavioral takeover red. Author diagnosis
identifies host process parser rejecting legitimate pgrp0 processes during
whole-/proc scan before positive target-group filtering; focused correction
and rerun remain pending. No takeover success or unsafe-proof relaxation is
claimed. Retained raw log SHA256 `580333a29d014e069f79f9b3d67b2e8e449e7f5afe205908362a46551dab1637`.

### Ticket16 public takeover behavioral red

Root inspected run2 terminal red1.34s and actual fixture. Registration now
precedes all native effects and binds actual SDK PID/group plus both directory
identities. After a completed supervised planning episode, parent strictly
kills/waits old SDK, pauses Mission, and registers a separate live replacement.
Public takeover-episode-store returns invalid_argument because command is
missing. This is the behavioral red; earlier registration failure is separate.
It does not yet prove safe transfer, unknown-start recovery, replacement
adoption, or negative inventory controls. Retained log SHA256
`afd04783a71afd1c4765a428a6976b5202c48f2c4fae7e9145f2d02c6dd45c48`.

### Ticket16 first focused takeover command green

Root inspected run3 failed temporarily_unavailable1.55s and run4 focused
green1.55s. Actual SDK planning capsule completes with trusted exit0 and
namespace Stop; parent strictly kills/waits original SDK, pauses Mission,
registers live separate replacement, then public CLI returns that controller.
Fixture checks CLI result and pre-transfer invocation/effect counts1/1. It
does not yet assert persisted transfer event/store ownership, post-adoption
no-extra Start, unknown-start crash recovery, or rejecting incomplete/foreign
inventories. Therefore this is a focused seam green, not full safe takeover
or ticket16 acceptance. Retained evidence:

- `public-registered-transport-takeover-run3.log`: `26565f3e322e2085e5be1100cd8ddf4b00da6dbd65ae5e73c05dde425de1f87a`.
- `public-registered-transport-takeover-run4.log`: `8cc2b1e76cb662152ba606a544853afb570a9de19c3942d82441ea2a4cb7c55e`.

### Ticket16 takeover SQL failure diagnosis limit

Root inspected retained diagnosis and corrected inventory query: store is
owned by gateway_bindings, so query now joins operations.binding_id to
bindings.id. The run3 cause is source-confirmed schema mismatch; detailed
server SQLSTATE was not retained after owned runner cleanup. Do not describe
the predicted undefined-column error as measured server evidence. No lock,
identity or inventory classifier relaxation is part of this repair. Retained
diagnosis SHA256 `c4cad94f638b9c95487faecb0a11027158f22bd7067d1364f698d95d9ef78bd3`.

### Ticket16 SDK crash adoption and bounded FIFO rejection

Root inspected crash-after-effect focused green5.10s and actual public
fixture: native Start effect persisted with Activity reply held and journal
Dispatching; strict original SDK SIGKILL/wait, paused transfer to live
registered replacement, DB controller and immutable evidence assertions, same
Episode/native identifiers confirmed, actual replacement Activity observation
barrier completed, invocation/effect counts remain1/1. This proves one
model-free planning crash boundary, not complete runtime failure matrix.

Root also inspected decisive FIFO red6.36s (owned bounded CLI kill), then
green1.49s with O_NONBLOCK before regular-file classification. Rejected
nonregular inventory leaves original persisted controller unchanged and
returns DependencyUnmet without forced timeout. Other foreign/missing/
conflicting inventories and current cache-authority tests remain pending.

- `public-takeover-native-fifo-red.log`: `d75fcf423af5c83c5a0b53feddfc4d9915fbbe945acf56a95619256a3ff11444`.
- `public-takeover-native-fifo-run2.log`: `1c0a3b1486e0776c625caee1c416e167e8a1ac550f5c680919d853348e7a1308`.
- `public-sdk-crash-existing-effect-takeover-run2.log`: `6fd4523c7cf52a218129bb0952a9e3b2f8652bcd7da675975631f1ec64ee2b9d`.

### Ticket16 takeover negative matrix setup failure

Root inspected aggregate negative-matrix terminal failure9.28s at unfinished
fixture allocation (DependencyUnmet), before that case invokes takeover.
Source attempts new Worker episode after Mission pause and unknown-start
barrier; setup ordering must preserve those guards. No matrix completion or
individual earlier-case acceptance is inferred from this failed aggregate.
Author notified to correct fixture ordering rather than weaken admission.
Retained log SHA256 `5e521113b955da50e3e0b3d6e4bb91c10566aed09ba157f8fd2b55667f070fce`.

### Ticket16 nine takeover inventory/process negative cases

Root inspected run2 terminal green15.30s and explicit begin/completion
markers for foreign effect, missing effect, duplicate/conflicting effect,
symlink record, replaced root, unfinished Ready workspace, surviving foreign
group member, live original controller, and dead replacement. Actual fixtures
invoke bounded public takeover and assert persisted original ownership on
rejection, unknown capacity held and release blocked, with unrelated process
not signaled. Controlled faults are restored before independent inventory
recovery where possible. Unfinished case allocates before Mission pause;
no pause/admission fence relaxed. This is the nine-case model-free takeover
negative matrix, not the full ticket16 runtime/context/budget/replay matrix.
Cache/current-authority and retained raw snapshots remain separate evidence.
Retained log SHA256 `fc792b3b90b1c9d583b543b1dd24c7a4cf787dbf067777854a39614c5a88b327`.

### Ticket16 extended crash/CAN/replay aggregate remains failed

Root inspected run3 compiler-only API argument mismatch and run4 actual
terminal failure11.32s: expected replacement Continue-As-New milestone not
observed within fixture bound. Replay and subsequent current-pause negatives
are not reached; no broad aggregate pass is inferred. Author notified to
inspect actual workflow/SDK evidence and rank causes before changing timeout
or production control flow. Earlier bounded crash/adoption proof is separate.
Retained logs:

- `public-sdk-crash-cache-can-replay-snapshots-run3.log`: `6df7e5b3f45fa12444945cf75df77a455564bcb48c221398b4f9b3d6c7a63f40`.
- `public-sdk-crash-cache-can-replay-snapshots-run4.log`: `09952486f2bf9884c06a41e4666cc88289160c7bbd3a3de0937b8d26863a49bf`.

### Ticket16 CAN diagnostic actual-history discriminator

Root inspected unchanged-bound run5 terminal failure11.38s and retained
actual history: ActivityTaskCompleted event8 followed by WorkflowTaskScheduled
event9 on kind2 sticky queue distinct from normal queue, with no subsequent
WorkflowTaskStarted. Progress is running with no operations. This supports
dead-original sticky routing as the missed-CAN cause, not failed drain or
authorization. Author reports completed replacement attempt2 after heartbeat
timeout; pinned SDK configuration investigation remains pending. No replay,
CAN or production default recovery-time success claim. Retained bytes:

- `public-sdk-can-timeout-diagnostic-run5.log`: `04f3b14ab3a96864df0a8d137bca9d71ebdd7ad38454d1e1f5aeaa7086da7de9`.
- `can-timeout-history.json`: `b2c7ca68c2238aa01e39183cb93f1cea39ecf6e01547295c83af56945519de40`.
- `can-timeout-progress.json`: `37e75fe5979be5aa322e7f4ae4f8fc1d0604bf0bd5e7032928c697d453cf4a3d`.
- `can-timeout-observed.json`: `226c44daedf18bc42a5929a392d240103eb98a328324b33450e5e31f67089004`.

### Ticket16 configured SDK recovery/CAN/replay focused green

Root inspected run7 terminal green10.98s, actual fixture oracles and retained
original52-event history ending ContinuedAsNew plus continued11-event history.
Both histories replay through unchanged Mission/Task workflow code; actual
replacement observation barrier, same native identifiers and invocation/effect
counts1/1 hold. Cached transfer rejects wrong token/Worker, resumed pause state
and changed control revision. The sole fixture configuration change is
supported500ms sticky queue timeout with caching retained and unchanged6s
CAN oracle. This is configured bounded recovery, not default10s sticky SLA.
Full16 budget/context/base/operator matrix, independent review and integration
remain incomplete. Raw native, transport, operation evidence, exit result, SDK
logs and histories retained in14-file manifest SHA256
`b04d8e0a9878a3878ee356dd928ea6ecea842f2f789068d9db4ff9a5584e222f`.

### Ticket16 query-only native receipt symlink regression

Root inspected decisive red0.52s: receipt symlink outside owned runtime
store incorrectly confirmed an unknown Start. Focused green0.54s preserves
OutcomeUnknown on the symlink; restoring actual receipt confirms existing
effect with only1 native invocation, then proven Stop plus confirmation
allows capacity release. Observer now uses bounded descriptor-relative
nofollow/nonblocking file classification, independently of takeover inventory.
This closes this specific query-only path, not full integrity/authority matrix.
Retained logs:

- `native-observe-symlink-red.log`: `11ff7443d347ac33f7f15c21831eacaa5c96cf30734ca03a1d70605dce1c15f8`.
- `native-observe-symlink-green.log`: `b7ce52f1ea2c4f65955e903499f08f80fb4afae32dcefd9a5738c93d4313d437`.

### Ticket16 exact operation inspector focused red/green

Root inspected actual missing-command red0.43s and focused green0.47s:
independent Owner CLI reads exact Prepared operation and immutable execution
binding. Verifier on Worker journal and wrong token fail with empty stdout.
Small/empty evidence_more=false is checked, not actual overflow/pagination.
Exact ID lookup supplements bounded status but full operator views and
historical credential/current-epoch matrix remain open. Compiler-only first
log is excluded as behavioral red. Retained logs:

- `operator-exact-journal-red-run2.log`: `0d3366870f7fc7efb281fca838fa3d704f67e0bb5f12f68e7ba082a3f8a7082a`.
- `operator-exact-journal-green.log`: `fbde4663d58bb8ca760db7175451dda75258ee283ca657814b63f04089f6846a`.


### Ticket16 all-role cumulative admissions and shared runtime fencing

Root retained and inspected terminal evidence. The all-role budget fixture passes
for planning, implementation and verification, each reaching the approved
8-episode cumulative limit with rejected successor leaving accounting unchanged.
Initial failure lacked canonical workflow registration needed by Yield; fixture
setup uses real Mission/Task registration. This is no-physical admission accounting,
not proof of physical concurrency, repairs or unknown usage. Shared legacy/episode
store fixture passes both directions with one invocation/effect each; paused direct
dispatch remains Prepared with no native files before explicit resume. Refreshed
complete lifecycle passes. Runtime registration auth red/green is retained below;
its terminal result must be read independently of filename. These focused checks
do not constitute ticket16 acceptance, independent review or root integration.

- `all-role-finite-budget-run1.log`: `7ec748b36eb1dcc81468ad9d5ed7d0823cd2d8d520246576b987effbc0be4784`.
- `all-role-finite-budget-run2.log`: `d42fcf92358d4152dff07ad0e2a2a423d8271cec0bc6bdc396ada2f9fe0e6f88`.
- `shared-legacy-episode-store-run2.log`: `2def58acf3d0fdad575a62f0686177f91ffb8b9b1e1146b0ef8046054cd14372`.
- `sdk-complete-lifecycle-refresh-run1.log`: `8e32b6c1808374f82a4e2a18758c06848bfaef85e6577c398cf5de6efd3b234e`.
- `runtime-registration-auth-red.log`: `b30801786e335da10edfa1eb0667b9ce4acaf947dfc99c764123b0e7fa4e65ba`.
- `runtime-registration-auth-green.log`: `e1e5f5fd940ef6afad55131d4e638ad7944050867c101e31f326ff5589074652`.


### Ticket16 public native reconciliation missing-command red

Actual public Owner CLI fixture fails with invalid_argument in1.24s for
reconcile-episode-operation after preparing a real uncertain native operation.
This establishes the missing query-only operator command; no green or acceptance
is inferred. Supervisor reconcile-execution remains a distinct operation.
Retained log SHA256 `875662a05c32cd21b7c51c8eb9c853d0cec61a92da30379ca5521cb090144893`.


### Ticket16 public native reconciliation first focused green

Public registered-live-operation fixture passes in1.42s after missing-command
red. This focused positive is not proof of expanded corrupt/missing/symlink
receipt negatives; those remain running separately. No ticket16 acceptance or
root source integration. Retained log SHA256 `86b583821e6f0348e14b8fbe9433298fcc06803ee28fc612f1a266661f357676`.


### Ticket16 native query receipt integrity negatives

Root inspected fixture and terminal0 (1.95s). Missing, corrupt and symlinked
actual native effect receipts each leave the original journal Dispatching,
used capacity2 and release DependencyUnmet. Wrong token and Lead-role caller
fail without stdout disclosure. Restoring original receipt confirms through
query only; original controller and paused Mission persist, native invocation
and effect counts remain1/1. This is registered-live transport reconciliation,
not dead-controller takeover, all evidence corruption, or ticket16 acceptance.
Retained log SHA256 `04d1d07211b68e4c694ab2ddc4e5a4161d1dd5c7a2b8f82e2a363a4558ccf45f`.


### Ticket16 public repair command behavioral red

After actual independent measured failed verification, public HTTP task.repair
returns400 invalid_argument; terminal behavioral red5.99s establishes missing
command. Initial compiler-only log is not behavioral evidence. Root identified
counter-only repeat of one candidate/report as insufficient repair-budget proof;
author agreed each failed report permits one consumable repair authorization,
with distinct supervised repair candidates and measured failures for exhaustion.
No repair enforcement pass yet. Retained red SHA256 `76db48748a728bcf6f23e8bbf23f587724243008fee361f1692f5dbcc76e1ba8`.


### Ticket16 genuine repair lifecycle and restored service capability

Actual run5 passes10.82s after two registered-SDK Worker repair candidates
and independent measured failing gates. Run4 completed repair assertions but
failed duplicate cleanup release; run5 changes that cleanup only. Single failed
candidate/report authorization, duplicate charge protection and finite repair
limit remain scoped proof, not full16 acceptance.

Managed sandbox interruption denied sockets/Docker; source snapshot and13-path
formatting/lint cleanup preserved separately with hashes in /tmp handoff.
Restored full-access root probe now confirms loopback listener, Docker26.1.4
and user/PID bubblewrap execution. Author tasked to verify exact preimages
before copying cleanup and run fresh real-service regression. Unknown usage,
physical budget, integrity and full failure matrix remain incomplete.

- `public-measured-repair-budget-run4.log`: `106382e1290c96f7bb0c5217884c86cbfde52359428311b924fdd1328de317d3`.
- `public-measured-repair-budget-run5.log`: `ec2793a9e281134be0ea56aaac7b20cd5578816af1517ea819e34ffb93edc139`.


### Ticket16 real-service unknown-usage behavioral red

Restored root actual PostgreSQL fixture reaches behavioral red0.42s:
confirmed native effect and trusted stopped workspace incorrectly allow
release without authoritative usage observation. Session65284 terminal101.
This is distinct from unknown native effect. Author now implements separate
usage accounting; no full16 acceptance claim. Retained root log SHA256 `eca381a5f52622f31fc54c8a0e7513aaf2a9b4ce819c47a37a20ed197efc04d5`.


### Ticket16 unknown usage retention focused green

Actual PostgreSQL public regression passes0.47s (session76035terminal0) after
behavioral red0.42s: native Confirmed and proven stopped local execution no
longer suffice to refund unknown usage. This is retention proof only. Registered
measured-zero query-only reconciliation and receipt-integrity controls remain
pending, as does legacy protocol coverage and full16 matrix. Source is the
exclusive author /tmp snapshot; no16 landing or acceptance inferred.

- `unknown-usage-public-red.log`: `eca381a5f52622f31fc54c8a0e7513aaf2a9b4ce819c47a37a20ed197efc04d5`.
- `unknown-usage-public-green-run1.log`: `400658c33c2755ed6c58b2e2fb97615665c8924855b72e96ec287b5ae1979be4`.


### Ticket16 registered SDK measured-zero query proof

Actual PostgreSQL+Temporal fixture passes2.39s (session5619terminal0).
Confirmed native operation initially retains Unknown usage despite stopped
workspace. Missing/corrupt/symlink/wrong-binding/unsupported-profile receipt
controls hold the reservation. Restored exact receipt allows paused Owner
query-only measured_zero evidence, unchanged controller and one native Start;
normal release follows independently proven Stop. Scoped to registered episode
runtime; legacy confirmed-operation usage remains an assembly gap. This is not
full16 acceptance. Retained log SHA256 `519e3b4e60c54cec07b36e79160a110bd5ee25d022ae1121ea9bde18a2a873bd`.


### Ticket16 legacy confirmed usage behavioral red

Root actual PostgreSQL fixture terminal101 (session29671) fails0.51s:
legacy gateway Confirmed Start plus independently stopped supervised process
incorrectly permits public release without measured usage. This establishes
the second protocol accounting gap, distinct from the now-green registered
episode runtime. Additive0007 assembled execution enforcement is next; existing
unassembled ticket12 behavior is a separately scoped baseline. No green yet.
Retained log SHA256 `b071409c2033e2dac923f088626a44ed0208e0a8c164ec2514cd36c77043559f`.


### Ticket16 legacy unknown usage retention focused green

Actual PostgreSQL regression passes0.51s (session30995terminal0) after
legacy Confirmed/Stopped behavioral red0.51s. Canonical task verification
requirements now retain unknown legacy-operation usage before deletion/refund;
caller-supplied Episode omission is not the source of enforcement authority.
This proves retention, not trusted legacy measured-zero reconciliation, whose
positive and integrity fixtures remain pending. Full16 matrix/review incomplete.
Retained log SHA256 `e1babc20377f665991eaa2954ada31892a29054ba6364bac8989fcd9817d5496`.


### Ticket16 assembled lifecycle measurement regression

Actual PostgreSQL+Temporal lifecycle fails3.68s (session84146terminal101)
at public Supervisor.release of continuation_execution, source2455–2459
in the frozen preimage. DependencyUnmet is retained; the tail alone does not
distinguish unknown legacy attempt usage from unregistered continuation
measurement. Author must inspect exact operation/usage evidence before fixing
registration and independent receipt attestation. No full regression pass.
Retained log SHA256 `1f7e282be19451e276d40fc8e46864f1c5200108f01372e41a85246211e330f3`.

### Ticket16 assembled usage discriminator

Root executed the frozen instrumented `real_temporal_assembled_vertical` against actual PostgreSQL and Temporal. Exit 101; test duration 3.63s; continuation release failed with `DependencyUnmet`. Retained log `/tmp/corenius-kernel16-managed-handoff/legacy-measurement-diagnostic-assembled-run2.log`, SHA256 `a2ceb167aaeb915fd993b8ceafd3278689adaf0c51ac7a43243702d9d4a8521d`. The pre-release SQL evidence independently identifies two confirmed legacy operations with unknown usage and no registered observer on the required task attempt. The modern continuation is confirmed, registered, and measured zero with a receipt digest. This establishes the legacy usage fence as the cause in this run; it does not establish acceptance or a successful repair. Source freeze released after terminal completion.

### Ticket16 registered legacy attestation positive regression

Root executed the frozen `real_temporal_assembled_vertical` against real PostgreSQL and Temporal after independently observed legacy transport registration and pinned receipt attestation were added. Exit 0; one test passed in 6.34s. Log `/tmp/corenius-kernel16-managed-handoff/legacy-attestation-positive-assembled-run1.log`, SHA256 `0c4041b77f4a52fb7790c07e936dfe104a9699ff1511b1dd423d0cb812a54369`. Retained pre-release SQL shows both legacy operations confirmed, registered, and measured zero, with exact receipt digests; the modern continuation remains registered and measured zero. Source freeze released after terminal completion. This proves the positive model-free assembled lifecycle only: missing/corrupt/unregistered/foreign-controller receipt controls, the remaining failure matrix, independent reviews, and acceptance remain required. It establishes no Trae usage or recovery guarantee.

### Ticket16 legacy query-only public seam red

Root executed frozen `public_registered_legacy_usage_query_requires_exact_receipt` against actual PostgreSQL and Temporal. Exit 101; test duration 2.61s. Actual failure at the CLI oracle: `authenticated query-only legacy usage reconciliation must exist: {"code":"invalid_argument"}`. The fixture reached the missing command after a genuine registered SDK Start with an unsupported usage profile retained confirmed/unknown usage and rejected release. The failure did not reach the deliberately incomplete later matrix assertion. Retained log `/tmp/corenius-kernel16-managed-handoff/legacy-usage-query-public-red-run1.log`, SHA256 `a30ccfff14c7f99ca9d068739c01e2162ffeffe7a23bb9c12be611f98dd86d38`. Source freeze released after terminal completion. This is a behavioral red for the missing public seam, not proof of completed negative controls or acceptance.

### Ticket16 legacy public query receipt controls green

Root executed frozen `public_registered_legacy_usage_query_requires_exact_receipt` against actual PostgreSQL and Temporal: exit 0, test 7.38s. The public fixture completed unsupported-profile, missing, corrupt, symlink, wrong-binding and wrong-runtime controls; it also checks wrong-token/Worker/Lead/Verifier refusal, retained unknown usage/capacity, exact restored receipt with single observation, unchanged paused store/controller and native execution counts, then the resumed candidate/independent verification/acceptance path. Log `/tmp/corenius-kernel16-managed-handoff/legacy-usage-query-public-green-run1.log`, SHA256 `491298c5b841051f174e557272252ee3f9da5658e389568605869d2ee2b849a2`.

Root then executed `legacy_confirmed_start_without_measured_usage_retains_reservation` against actual PostgreSQL with its added public unregistered-query assertion: exit 0, test 0.52s. Log `/tmp/corenius-kernel16-managed-handoff/legacy-unregistered-usage-query-run1.log`, SHA256 `c6f6fa41a2d08d6cbd2438966790a32ca70bd7570ab9c823cf1890487ba72fe7`. Source freeze released after both terminal completions. Explicit foreign-controller rejection, remaining full failure matrix and independent reviews remain unproven; no ticket16 acceptance or Trae guarantee is claimed.

### Ticket16 legacy foreign-controller authority rejection

Root executed frozen `foreign_legacy_controller_cannot_attest_or_acquire_retained_store` against actual PostgreSQL and Temporal: exit 0, test 2.66s. A genuine registered SDK Start retained confirmed/unknown usage; a distinct controller could not bind the retained physical store, and authenticated Owner usage observation could not attest through quarantined authority. The fixture checks unchanged original store controller, unknown usage with no sample, retained capacity and native counts, then strict owned SDK shutdown after capsule stop. Log `/tmp/corenius-kernel16-managed-handoff/legacy-foreign-controller-run1.log`, SHA256 `678cdc14501defebe9f9cb5e59fc5603b1be443dc19ca38568fa0fd4ec4ad7ea`. Source freeze released after terminal completion. This proves authority rejection, not successful legacy takeover or the full ticket16 matrix.

### Ticket16 remaining proof inventory after usage controls

The active implementer refreshed the remaining full-ticket scope; these items are unproven and are not deferred by the focused usage successes:

- Legacy unknown-usage continuation and physical-claim rejection.
- Two actual live capsules saturating physical capacity, retained/refunded reservation behavior, and deadline enforcement through the operator seam.
- Fresh repair and all-role cumulative-budget regressions after usage changes.
- Sealed-context tampering, current input, scope and generation rejection.
- Corrupt candidate/report/log, missing referenced artifact, and changed integration base requiring a fresh candidate and gate.
- Broker and legacy gateway restart/lost-wakeup recovery beyond the existing SDK crash/replay/rollover subset.
- Full takeover inventory and refusal while unfinished work remains; legacy successful transfer is not established by its current unsupported/quarantine behavior. Canonical safe-quarantine verdicts must be assessed explicitly against scope rather than labeled successful recovery.
- Operator output beyond 32 records, exact lookup and historical/current credential checks.
- Fresh original supervisor matrix, complete PostgreSQL/Temporal suite, formatting/strict Clippy, frozen independent Standards/Spec reviews and final root integration gate.

The next proposed behavioral slice is unknown legacy usage blocking continuation/physical claim. This inventory records missing evidence and sequencing; it is not completed testing or acceptance.

### Ticket16 unknown legacy usage continuation admission red

Root executed frozen `unknown_legacy_usage_blocks_continuation_before_budget_charge` against actual PostgreSQL and Temporal. Exit 101; test duration 2.63s. The actual public successor admission returned an admitted receipt where the oracle required `DependencyUnmet`, after a genuinely registered SDK Worker had confirmed/stopped execution but unknown usage, publicly yielded and captured new input. This establishes the missing continuation usage fence; it is not an effect-uncertainty failure. Retained log `/tmp/corenius-kernel16-managed-handoff/legacy-unknown-continuation-public-red-run1.log`, SHA256 `46a07866463a55dca44e4ebfad3a3be277ed6f18e0b3c2c27455e58998c4208f`. Source freeze released after terminal completion; task-bound admission guard repair and fresh positive/negative regressions remain required.

### Ticket16 unknown legacy usage continuation admission green

Root executed the frozen repaired `unknown_legacy_usage_blocks_continuation_before_budget_charge` against actual PostgreSQL and Temporal. Exit 0; one test passed in 2.67s. Log `/tmp/corenius-kernel16-managed-handoff/legacy-unknown-continuation-public-green-run1.log`, SHA256 `95af130be4efd6f478f2d94777c1cced276083c77b87d8fb56e4785e8d244886`. The previously admitted successor is now rejected by task-bound unknown-usage checks before episode/budget insertion. The implementer also added preparation/final dispatch guards, but this focused test does not by itself prove every physical entrypoint or the previously-admitted direct Supervisor race. Source freeze released after terminal completion. Fresh measured lifecycle/repair/cumulative-role regressions, remaining failure matrix and independent review remain required.

### Ticket16 isolated capacity fixture snapshot

Under an explicit author freeze, root transferred the complete 45-file unpublished control-plane crate into `/tmp/corenius-kernel16-capacity`, based on landed main `1f164156261a3aa927c823919bf73d6e6d11a771`. Source-before/source-after/destination hashes all match, and root Cargo.toml/Cargo.lock match the baseline. Manifest SHA256 `51d629e8ac9ef3f44414768878bc093dbf327a74c8821562f72a3a0e2052f37d`. Freeze released immediately after verification. Clean-context `kernel16_capacity` may edit only new `tests/assembled_capacity.rs` and its own scratch; original author retains sole ownership of the managed source and Supervisor-race work. This records isolation/provenance, not behavioral success or acceptance.

### Ticket16 direct Supervisor race fixture setup failure

Root executed frozen `unknown_legacy_usage_blocks_previously_admitted_verifier_launch` against actual PostgreSQL and Temporal. Exit 101; test duration 0.29s. Failure occurred at candidate `ArtifactCommand::Submit` (frozen line2100), returning `DependencyUnmet`, before Verifier admission, the legacy SDK effect or the intended direct launch oracle. Root inspected the frozen setup: a private Git commit/bundle is produced while the original Worker execution is still Ready. This is a setup failure requiring diagnosis; it does not establish a Supervisor launch vulnerability or a successful fix. The author must use genuine authoritative execution evidence and retain all policy guards. Log `/tmp/corenius-kernel16-managed-handoff/legacy-physical-claim-public-red-run1.log`, SHA256 `92b7aaa10fc29be1dc2b44e5083ca5eec641b48a82d698c0ad20f5c83f8dda53`. Source freeze released after terminal completion.

### Ticket16 direct Supervisor race decisive red

The setup diagnosis is corrected: ArtifactCommand::Submit checks declared required deliverables, and the genuine fixture requires `src/result`. The earlier missing-execution-proof hypothesis was incorrect. Root independently read that guard and the fixture specification. A one-variable real Git path/declaration correction retained all product policy checks.

Root executed the corrected frozen `unknown_legacy_usage_blocks_previously_admitted_verifier_launch` against actual PostgreSQL and Temporal: exit 101, test 2.86s, intended launch oracle received Ok(()) instead of rejection. Root independently inspected retained physical-claim.json: actual candidate-bound trusted exit0, stopped namespace, and capacity before/after2. Retained verifier.log contains the actual unexpected verifier workload output. This proves actual direct Supervisor execution after prior legacy usage became unknown; it is distinct from the first setup failure. Log `/tmp/corenius-kernel16-managed-handoff/legacy-physical-claim-public-red-run2.log`, SHA256 `30fd2d3d79cd9caf6491911505fcd774ef384cca0b0264d01ca8c6505e4071ed`. Source freeze released. Physical-boundary repair and fresh measured-positive regression remain required; no acceptance is claimed.

### Ticket16 direct Supervisor launch guard green

Root executed frozen `unknown_legacy_usage_blocks_previously_admitted_verifier_launch` against actual PostgreSQL and Temporal: exit 0, test 2.79s. The previously launched pre-admitted Verifier is now refused with unknown legacy usage. Log `/tmp/corenius-kernel16-managed-handoff/legacy-physical-claim-public-green-run1.log`, SHA256 `da77bdfc1f96377942fd72b9a7ba7032a65bce1b7afda67a848036f0e278b3b4`. Source freeze released and author resumed remaining scope. The forced monitor post-controller-claim interleaving is still separate and unproven by this direct-start test.

The isolated capacity worker reports its final actual-service suite exit0 (3 passed, 1 ignored, 30.97s), with test-publication observation race corrected and no product edits. Root inspected its handoff/independent cleanup and independently matched new file SHA256 `8eb9880c3b10e7376eb87a6c39d305bc7ef87d5feefe65bdda8ff4535bd7be15`. Fresh root execution, inherited-file verification, test review and exact new-file integration remain required; worker reports do not establish full16 acceptance.

### Ticket16 forced final-monitor boundary mutation red

Root executed frozen `unknown_legacy_usage_blocks_monitor_after_controller_claim` against actual PostgreSQL and Temporal. Explicit mutation configuration removed only the newly added final monitor physical_usage_known check; allocation/direct-start guards remained. Exit 101; test 3.46s; final monitor oracle returned Ok(()) instead of rejection after the forced interleaving introduced unknown legacy usage following controller claim. This establishes sensitivity to the final guard, not an original baseline failure or a completed repair. Log `/tmp/corenius-kernel16-managed-handoff/legacy-monitor-claim-mutation-red-run1.log`, SHA256 `19313af370759f90810c685611f211579a8c6836c5110b439086e9c8107ccf39`. Source freeze released for exact guard restoration and fresh green verification.

### Primary kernel worker replacement

After actual probes showed unrestricted parent capabilities but stale restricted worker socket/Docker failures, the owner explicitly requested replacement. Old `artifacts11` was terminal, with no local service handles and frozen source. Root preserved it read-only and transferred all45 current crate files into new isolated Git worktree `/tmp/corenius-kernel16-author`, branch `codex/kernel16-author`, based on main1f16415. Before/after/destination hashes all match; source manifest SHA256 `939225e1d509bcc88d360361c731770076041cec49f28b1405d94d3ee5989a6c`. New clean-context `kernel16_author` receives the full remaining16 scope and must verify its actual capabilities, then run the restored final monitor green. Snapshot/dispatch is not proof of permissions, test success or acceptance. Original source/evidence and all branches remain preserved.

Replacement capability result: `kernel16_author` independently verified active danger-full-access filesystem/network-enabled/approval-never policy and default exec exit0 for reversible own-tree write/read/delete, loopback bind, Docker server26.1.4 and HTTPS HEAD200. No overrides, source mutations or paid/live models were involved in these probes. It also independently verified the replacement manifest. The stale worker restriction is resolved for this replacement; product tests/reviews remain incomplete.

### Root independent capacity fixture verification

Root independently verified all45 inherited snapshot hashes and the new assembled_capacity.rs SHA256 `8eb9880c3b10e7376eb87a6c39d305bc7ef87d5feefe65bdda8ff4535bd7be15`, then executed the full new test target against real PostgreSQL/Temporal: exit0, 3passed/0failed/1ignored helper, 30.98s. The parent fixtures actually launch the ignored owned SDK helper. Log `/tmp/corenius-kernel16-capacity/.scratch/team-control-plane-capacity/root-verification-01.log`, SHA256 `f1fe1692ebe548c2eb969a1e0ca331a213c11c7ac03fd740db06806aaf9abb34`; wrapper cleanup records Temporal listener closed. Root requested sole-writer primary author import ONLY that new test file, never old snapshot product/helpers. Fresh integrated tests and independent full16 reviews remain required.

Root also inspected replacement author's restored final-monitor run.log: the exact forced-interleaving test passed in2.86s after restoring the guard. Author reports fresh actual lifecycle, genuine repairs, and all-role budget regression greens; subsequent integrity and current-authority/sealed-context cases remain pending final source freeze and review. Scope remains full16 and the original later phases.

### Changed integration-base materialization investigation

Root verified imported assembled_capacity.rs exact hash and inspected the primary author's actual changed-base run3: exit101, test5.06s, create_episode_from_candidate failed TemporarilyUnavailable after fresh candidate/new Verifier admission. Root inspected the current source guard comparing candidate.base to manifest.base; author is collecting exact manifest/bundle diagnostics before a minimal repair. The immutable admitted Task/repository identity and current authorized artifact policy must remain enforced; simply removing equality is not authorized. Log SHA256 `b90cab09314ecfcd59c0daf7a08cd35c46bdf12089a0b44373ec03ff6d8e182e`. Earlier run1/2 setup failures are distinct. Fresh candidate/gate/acceptance remains incomplete.

The independent capacity worker was assigned a new nonoverlapping tests/assembled_operator.rs slice in its existing private tree: public >32-record summaries/exact lookup and historical/current credential fences. Its completed capacity test and all45 inherited product/helper hashes must remain unchanged. No product-source editing or acceptance authority is delegated. Primary author retains integrity/base/restart scopes.

### Changed integration-base first repair verified

Root independently read the replacement author’s actual changed-base green run3 log and hashed it: test actual_changed_git_base_requires_new_candidate_and_independent_sdk_gate passed, 1passed/0failed, 6.19s; SHA256 `a6ce11c9ebedd1f81ea0e4ee37bd8aece70b9075db15f9d78031cb1ecae0d7da`. The author reports candidate/current policy base validation before allocation and a candidate-bound execution manifest while preserving immutable admitted Task scopes and physical equality. This focused test result is verified; final source inspection, integrated regression and independent Standards/Spec reviews remain required. Earlier directories named green-run1/2 contain failed runs after edit scripts aborted before writing; their names do not indicate passing results. Full16 and later goal phases remain unaccepted.

### Replacement worker recovery focused results

Root independently read exit records and actual test logs in replacement author scratch: active-takeover-run1 exit0, actual running unfinished workload refusal test passed2.13s, log SHA256 `236773b3b230378e03d789a75b50161d9889dbc4629e88dee7a795397ce663ae`; legacy-restart-run2 exit0, no-repeat-start safe quarantine test passed2.64s, SHA256 `7f21a801338d0d68650253bebcba7af5f1b8217044cbdb921e061f5b8d646193`; broker-restart-run2 exit0, owned work and bounded operator exact lookup test passed1.00s, SHA256 `594117ed2317ca546960075452dc19274def53c7f6836a71528feeffa1dfeab7`. Legacy quarantine is not successful controller transfer. These are focused results pending test inspection, integrated frozen gate and independent ticket reviews.

SPEC acceptance and failure matrix require obsolete candidate/base reports cannot accept a changed integration candidate. They do not demand blanket cancellation of an already allocated immutable historical verifier snapshot. The author will prove postallocation changes reject obsolete evidence and require a fresh current candidate gate/acceptance; root will review that precise contract rather than introduce unconditional cancellation.

Root inspected broker_process_restart_preserves_owned_work_and_bounded_operator_exact_lookup fixture body: launches actual CLI HTTP server, drops its HTTP client, kills server process and starts replacement against the same PostgreSQL/epoch; compares public task snapshot/revision and durable pending Bridge hints, observes a separately supervised capsule writer still advancing, then strictly cancels/reconciles and checks no further writes. It additionally admits33 public missions plus original, checks32-row status/more_missions and dynamically selected omitted exact HTTP lookup with scoped denial. This establishes broker API process/view replacement with surviving independent supervisor; it does not establish simultaneous monitor/Temporal runtime loss or database restore. Those retain their own proof requirements.

### Root independent operator fixture execution

Root inspected operator-handoff.json and matched assembled_operator.rs SHA256 `7912b92842224796bd94c888b9da343b74b795e1088bbf26c3aabc803abd9efe`. With the author terminal, root independently executed its complete PG operator target using the recorded public CARGO cache/private capacity target; observed session86360 terminal exit0. Log is retained at `/tmp/corenius-kernel16-capacity/.scratch/team-control-plane-capacity/operator-root-verification.log`. Fresh integrated primary-tree execution and independent full16 review remain required. Operator fixtures do not prove per-operation native evidence overflow, live models, restore or Worker replacement-generation cases.

### Postallocation integration-base focused proof

Replacement author reported actual session56172 terminal0. Root independently inspected postallocation-base-run1/run.log: postallocation_base_change_preserves_snapshot_but_requires_fresh_current_gate passed1/0failed6.15s; SHA256 `ed87cb72c27053ca3b3fae8b4fe5d0bb5fa4682470f905e6e3421ada24bdee17`. Historical immutable verifier snapshot may execute, but changed approved Git base rejects its report; author reports fresh current candidate gate/Owner acceptance in the same finite mission. Root also read context-final-run1 log: physical_monitor_rechecks_current_scope_generation_and_epoch passed2.16s. Exact fixture assertions and full integrated gate remain under review; no ticket acceptance follows these focused logs alone.

### Sealed context focused proof inspection

Root read sealed-final-run1/run.log: actual_sealed_packet_and_write_scope_resist_workload_tampering passed1/0failed0.51s; SHA256 `47307e58f763d299caf2b65c7af171d448409aaf653b08ba0c21e70da97a5827`. Root inspected its fixture: public wrong-digest admission rejects without episode/capacity charge; actual PostgreSQL immutable-input trigger refuses packet rewrite; actual bwrap workload reads captured current mailbox notice, cannot write sealed input or SPEC.md, and writes admitted src/result. Trusted monitor records exit0/stopped namespace; original SPEC bytes unchanged. This scope is narrower than all possible context faults; current-authority and changed-reference/digest tests remain separately mapped. Integrated frozen gate/review still pending.

Replacement author lint progression: original clippy-run1 exit101 and clippy-fix-run1 exit0 with remaining capacity-test warnings are preserved, not passing strict evidence. Root read clippy-strict-run2.exit=0 and clean run2 log10.87s, SHA256 `b0cc320a091ee6e0e951383c9d79f405758abfdcfee74b16659c66b9da543a21`. Final manifest must bind exact all-target/deny-warning command args and source hash; full service gate/review still required.

### Full integrated service gate failure

Root inspected frozen full-gate-run1/run.log: assembled_vertical aborted SIGABRT with stack overflow in public_failed_verification_repairs_are_authorized_idempotent_and_bounded. Log SHA256 `21649149480bf2e326a7d9330ef6f35f8b5f0d7b97e4f92f05c22954a3ee82b5`. Earlier focused repair green does not establish full-suite readiness. Original artifact/capacity/operator and some vertical cases passed before the abort; remaining tests are not proven. Author was instructed to confirm terminal/cleanup and minimize the exact failure before patching. No kernel acceptance, independent-review pass or landing follows this failed full gate.

Full-gate failure scope correction: stack overflow was the final abort, not the only failure. Author reports terminal101 with Temporal listener closed and preceding delayed_monitor authorization failure, legacy/episode namespace release DependencyUnmet, lost-reply release DependencyUnmet, and actual SDK crash takeover Continue-As-New timeout. Post-abort inventory records three owned SDK helper process identities requiring exact pidfd cleanup; cleanup is not yet independently confirmed. Frozen run1 remains failed. Each compatibility/recovery issue must be minimized and repaired without weakening expiry, usage holds or recovery oracles; full suite must be rerun.

Root further inspected complete full-gate log: four additional fixture initialization TemporarilyUnavailable failures at support/assembled.rs78 affect planning dispatch, registered-live takeover, postallocation base and final-monitor unknown usage tests. They belong in the complete failure inventory; shared-service contention is a hypothesis, not a diagnosis. Root read exact PID/start-tick cleanup report for1429595/1429868/1430595, each pidfd SIGKILL terminal recorded. Root also read minimized repair-stack-repro-run1: the single exact repair test independently aborts with stack overflow, so parallelism alone cannot explain that defect. Author retains responsibility for actual cleanup verification and minimal repair.

Root independently inspected repair-stack-size-run1 exit101/log: constructed repair future bytes8440, followed by single-test SIGABRT stack overflow. SHA256 `f862e6be6ea867e66d182adb4caeca3793fb933a02dc26e0051292e7051f8cd8`. Future value size and disassembled poll stack frame size are different measurements; the 8440-byte measurement alone does not prove oversized future allocation causes the abort. Minimal repair and fresh behavioral green remain pending.

Independent read-only stack diagnosis confirms current fixture poll reserves1,402,840 bytes and repair helper poll238,088 bytes, approximately1,640,928 bytes active before child polls. Helper constructor is small; heap-pinning stored future did not change nested poll frames and focused heap probe still aborts. Strong hypothesis is synchronous active poll-chain exhaustion; helper-child-only exhaustion remains possible and recursion is weak/unproven. Recommended one-variable structural experiment returns owned setup state to a thin orchestrator, drops setup future, then runs complete repair phase; no global stack override or removed assertions. Independent diagnosis is not a green repair or whole-ticket review.

### Repair test task-boundary focused green

Root inspected repair-task-boundary-run1/run.log: original public_failed_verification_repairs_are_authorized_idempotent_and_bounded passed11.05s, both genuine SDK repair rounds retained. Author confirms session42314 terminal0 and Temporal listener closed; log SHA256 `db06c0d71bc0a58b89573a14f1ff41e84747ffc3199d0df76e51b74ce4b43c35`. Awaited task boundary separates helper polling from giant parent frame while retaining shared owned fixture until joined. Default stack unchanged. This supports synchronous nested-frame cause; it does not prove future-value allocation was large. Heap probe is failed fix1; second structural fix passes focused test. Other full-gate failures and fresh integrated gate/review remain pending.

Post-repair frame measurement reported by author: fixture poll1,410,232 bytes, helper238,088 bytes. Green follows separation of synchronous polling chains through awaited Tokio task, not shrinking either frame. Arc fixture is recovered by try_unwrap only after JoinHandle completes before original finish; no detached repair workload or authority increase is intended. Root read current lost-reply/shared-store fixtures and flagged their effectConfirmed-only release expectations: confirmed native effect does not establish observed usage. Actual operation diagnostics must decide whether fixtures should assert retained unknown-usage capacity; product refund guards must remain intact.

Root inspected isolated usage-release-diagnostic-run1: original lost-reply test fails0.51s at release with DependencyUnmet; actual operation diagnostic is confirmed with usage.kind=unknown. SHA256 `5231bc44fe9a31e1705d91bdd08d5cb97f097b3aa822dc88ef55d32713718aff`. The unregistered callback fixture old refund expectation contradicts the required unknown-usage hold. Author will preserve effect/query-only/dedup/symlink/continuation assertions and strengthen no-refund assertions without weakening product guards or fabricating measured zero. Fresh corrected fixture and separate registered measured positive remain required.

### Unknown-usage fixture expectation corrections green

Root independently read/hash-verified usage-hold-green-run1: lost-reply test passed0.55s, actual confirmed/unknown operation retained; log SHA256 `f7218cc3cc70eff3d67cefc38f9c847644a1e55d17bcdfea8739fadbe9c41065`. Shared-store-hold-green-run1 passed1.15s, SHA256 `385c654e9ce7d4cc29eb8e1f08f5011fde4880d2da72f2f0b3d97eea7907d057`: legacy-first actual legacyConfirmedUnknown/modernPreparedUnknown; episode-first modernConfirmedUnknown/no legacyrow, each exactly1invocation/1effect. Author reports public unknown usage and capacity1 retained, original effect/dedup/symlink/continuation assertions intact and no product guard edits. These correct stale refund oracles, not product accounting relaxation. Integrated gate/review and remaining timing/init failures remain open.

Root read issuer-expiry-natural-run1 actual test log: passes10.29s, DB observation at02:38:32.076991 confirms issuing grant live before delayed launch boundary; expiry02:38:42.025888 and observation02:38:42.032848 confirm natural expiry before refused monitor. Finite10-second grant remains below30-second episode bound; alternate grant cannot authorize the expired issuing execution. Final session cleanup confirmation and integrated suite still required. Author is adding owned PG server logs/max_connections/activity sampling to diagnose default-parallel failures; resource contention remains unproven until sampled.

### Parallel fixture service saturation diagnosed

Root inspected parallel-pressure-run1 terminal101:33passed4failed2ignored41.26s. Four fixture Broker::connect failures remain; prior stack/CAN/issuer failure classes did not recur. Root independently read owned postgres.log: four FATAL too many clients messages at02:38:57.731/.785/.839/.893; default max_connections100, finalcount6. This proves shared disposable test service saturation for this run; sampled peak92 can miss instantaneous limit. Harness default libtest fixture concurrency2 when caller leaves RUST_TEST_THREADS unset is permitted as measured test-service admission control; explicit override and each fixture’s required internal physical concurrency remain intact. No production/Mission budget change is authorized. Author now runs full-gate-run2 session88415 with every case; source acceptance awaits terminal result, final strict checks and independent reviews.

Full-gate-run2 remains failed: root inspected actual assembled_capacity survivor quarantine PermissionDenied(os13), with real namespace PID1/nested members retained, then test target2passed1failed1ignored9.13s. The strict unknown/quarantine and no-refund contract must remain intact. Author was instructed to retain exact processes and diagnose failed observation path/identity before retrying; focused capacity greens do not establish current integrated capacity readiness. Harness default2 is observed in diff with caller override preserved and no internal physical concurrency change.

Root read isolated capacity-observation-diagnostic-run1: two actual running capsules/third refusal/cancel/deadline/refund case passes30.59s; logSHA256 `1d172cf3dc5afe3e668221d52918dfef8baf3bc248d8e7afe9297f81959244bb`. Observation tracing retains original error classification; visible traces are ENOENT process churn, not EACCES. This failed to reproduce fullgate2 PermissionDenied and does not establish its root cause or clear that gate. Strict quarantine/no-refund safeguards remain required.

Root read capacity-overlap-diagnostic-run1: all3 capacity cases pass with1owned helper ignored32.43s, visible observation traces only ENOENT. Concurrent-fixture diagnostic also did not reproduce prior EACCES. No root-cause repair is established by these two diagnostic greens. Author must preserve tracing for full reproduction and avoid speculative EACCES-as-gone classification; strict safe quarantine remains enforced.

Fullgate3 reached assembled_vertical36passed1failed2ignored66.14s; single failure is takeover CLI stderr JSON parse at1945 after diagnostic identity eprintln was added. Root inspected terminal tail; exact offending stderr must confirm contamination before removal/fix. Original later Supervisor/Temporal targets are not established by this aborted gate. Fullgate2 EACCES remains unexplained despite diagnostic capacity passes; this limitation must remain in independent review.

Author isolated CLI stderr contamination: diagnostic-run1 terminal101/15.67s retained takeover-error-stderr.bin SHA256 `bcb7648f7520dd2cd2806821b10b08fdc00812fa5e86f42e735c9254a64ec718` contains temporary proc-observation prefix before correct dependency_unmet JSON. Temporary trace removed completely while original identity io-error semantics and strict JSON oracle remain; raw stderr capture retained. Full-gate-run4 session40624 is active, all cases retained. EACCESgate2 unexplained safe-quarantine limitation remains explicit for review; no claim of its resolution.

### First complete replacement-author service gate green

Root independently read full-gate-run4/exit=0, final Temporal7passed1ignored11.83s/doc-test terminal summaries, exact log SHA256 `592e65d49b4229e00dee19a8303fa16e9a241e644e026d90753c1139702d3388`, and cleanup temporal_listener_closed=true. Author reports complete114passed4ignored across15summarygroups, including original Supervisor20passed13.17s. Ignored helpers remain parent-executed owned subprocesses. Final strict lint/fmt/source freeze and independent Standards/Spec reviews remain required; this does not self-accept16 or explain previous gate2 EACCES. All later goal phases remain open.

Root independently matched all47 frozen crate files against final-review-manifest.json (23changed baseline1f16415), no byte/hash mismatches. Manifest SHA256 `f4bade885be38245c1170563086b749bf926a7530c3eae162c1b1a385cd12cdb`; baseline-relative diff SHA256 `f3c8867d4af64abb4469af48ded0d76f339b5e6eb6e047c7e58831fc04943d44`. Separate clean-context Standards/defect and Spec reviewers are active against this exact snapshot including untracked source/tests/migration. Acceptance remains pending their findings and root integration gate.

### Independent Standards and defect review, round1

Clean-context kernel16_standards_review independently verified47file manifest/23changes and frozen diff. Verdict: no blocking documented breach or demonstrated material defect in reviewed auth/generation/epoch, serialized claims/idempotency, native evidence/usage holds, physical release, sealed input and execution-backed verification. One P3 heuristic suggestion: duplicated unresolved-operation/unknown-usage predicates across episode_runtime, mailbox and supervisor; scope differences are intentional, no mandatory refactor. Unexplained gate2EACCES safe-quarantine remains an availability/reproducibility limitation, not resolved by later greens. Spec review and root integration gate remain pending; this review alone does not accept16.

### Independent Spec review, round1: changes required

Clean-context kernel16_spec_review source review identified P1 canonical attempt cancellation missing: public cancellation changes workspace desired_cancel only, not attempt desired state or future dispatch authority; episode admission can continue and cancelled task disposition absent. P2 status omits independent kernel control_revision and snapshot_time, so replacement operator cannot discover pause/resume/takeover preconditions after response loss. Root delegated both repairs to the same isolated author with public red-green evidence, no unknown-usage refund relaxation, fresh full gate and new freeze before round2 review. Standards round1 remains no blockers/optionalP3 predicate duplication. Reviewer classifies unexplained EACCES as explicit availability limitation permitted by safe quarantine, not demonstrated safety nonconformance. Ticket16 remains unaccepted; later goal phases/rootgate/landing incomplete.

Review1 P1 public cancellation red: root inspected actual assembled_cancellation public_attempt_cancel_revokes_preadmitted_delayed_and_future_dispatch test0.31s failure at HTTP400 invalid_argument for missing canonical attempt.cancel. Log SHA256 `396a831b8d1b2b3853097a38d4f8cef7d358334afaa5c6c30a64cd36c819e5c6`. This establishes missing public contract, not corrected revocation; guarded implementation/reconciliation and fresh green/fullgate/round2review remain required.

Root inspected review1-cancel-green-run1 actual focused initial test: public_attempt_cancel_revokes_preadmitted_delayed_and_future_dispatch passed0.34s. Unused import warning remains for final lint. This focused result proves only the fixture assertions; running nested-effect shutdown/uncertainty holds, broader authority/artifact fences, P2 public red-green, fresh full service/root gates and round2 independent review remain pending. Original114-test green is pre-review-repair source, not final evidence for modified cancellation implementation.

### Review1 public status and cancellation proof

Root independently inspected review1-status-red-run1: actual fresh public status after a lost pause response omits control_revision (Null versus expected1), test fails0.23s. Log SHA256 `3d6ebedddf01d9976f008b18cc782b1d13e02e2a58665eff60d5eeb335629cc3`. Combined review1-public-green-run2 then passes both public cancellation and fresh-client control revision/database snapshot-time tests, 2passed0.80s, zero filtering; log SHA256 `8b29f90e451bb771a6f91a1d1fad15e5d6fbd50cfedb259ce3033c04794277ec`. Running-process cancellation, uncertain outcome/usage holds, full repaired-source gate and round2 independent reviews remain required. This result does not establish ticket16 acceptance or completion of the six-phase goal.

### Review1 running cancellation fixture: unresolved release failure

Root inspected review1-running-cancel-green1 actual terminal failure0.62s: DependencyUnmet at assembled_cancellation.rs:532, the known-physical release expectation after cancellation. Despite the directory name, this run is failed evidence, SHA256 `ee3e27f8012e785946a59ce8fed96e23151c940098c3739a432e238f29f19159`. Earlier assertions precede the failure but all three modes are not proven. cleanup.json reports temporal_listener_closed=true. Author was asked to diagnose actual runtime/workspace/operation state before altering any guards; no refund relaxation is authorized. Full repaired-source gate and round2 review remain pending.

Running cancellation failure diagnosis: author identified line532 as the second release call; root independently inspected exact fixture lines531–532 and Supervisor::release:812 state guard. The first release succeeded; the second is rejected because Released is outside allowed Stopped/Deleting states. Correcting the fixture to assert Released and repeated release DependencyUnmet with capacity remaining0 preserves exactly-once refund without weakening product guards. All unknown modes and corrected terminal result remain unproven until rerun. This supersedes any interpretation that the initial known-physical release itself failed.

Running cancellation run2 fails1.29s on cancellation_requested versus cancelled, SHA256 `4dc0ed6c3a21a1029101270c5fb44767c8768159de0225c904e2c6125dc9c638`. Root inspected loop: it breaks on a separate later Supervisor::inspect observing Stopped while retaining the earlier command disposition; those observations need not share a transaction. Known-physical retained JSON reports cancelled attempt/task and observed PID1/nested members. Author corrects waiting to require both physical Stop and a terminal domain reconciliation result, rather than weakening cancellation requirements. Unknown modes remain unproven until a complete corrected run.

### Review1 running cancellation focused green

Root independently inspected review1-running-cancel-green3 terminal exit0: public_cancel_stops_actual_nested_writers_and_never_refunds_unknown passes2.13s (1passed,2filtered), SHA256 `8e7c0cc4dae543416a79e43fea14232c72a9e8437e7ee4503ffc65dfe54a5c7d`. Retained three mode JSON records each include four observed process identities and stopped output: known-physical cancelled task/attempt; unknown-usage cancelled task/attempt while accounting remains reserved by test assertions; unknown-effect quarantined attempt with task still running. cleanup reports Temporal listener closed. Corrected fixture awaits physical and domain terminal observations and rejects a duplicate release without a second refund. This focused evidence does not replace the full repaired-source gate, root integration gate or round2 independent reviews.

Measured candidate cancellation run1 terminal1014.98s, SHA256 `4c6f721bef1ce76a563fb5ccbaf52cac714271b5a249dfd392c04a816cf87660`: failure at cleanup Supervisor::release after cancellation/submission/acceptance rejection assertions. Root inspected retained cancelled-report-proof: actual independent gate exit0, namespace_stopped=true, candidate includes original and resumed deliverables; canonical cancellation requested revision8 then cancelled revision9. These preceding facts do not make the whole test green. Author must diagnose exact cleanup receipt/state failure without refund relaxation and rerun; cleanup reports Temporal listener closed.

Measured cancellation cleanup diagnosis confirmed by root: continuation execution already released in preexisting fixture setup before cancellation; failed cleanup attempted duplicate release. Correct cleanup inspects state, releases only Stopped and requires already Released otherwise, preserving receipt guard. Fresh review1-measured-cancel-green2 exit0 passes cancelled_attempt_cannot_submit_cached_report_or_accept_measured_candidate5.04s, 1passed39filtered; log SHA256 `e94bebdf68ce858852c14c64a2eedffcbde45117f854cc3ea54533ecb1891dc5`. Root independently read terminal log and exact fixture paths. Actual measured SDK candidate/Verifier report cannot bypass subsequent canonical cancellation via cached report submission, new submission or Owner acceptance. This remains focused evidence pending full repaired-source gate and independent review.

Final focused public cancellation/status suite review1-public-final-run1 exit0:3passed2.57s, zero filtering, log SHA256 `69d8075cfbff68811575d07a83646a61a657a81d9f24c648320ba3a1bdfc9b60`; root independently read terminal log/exit/cleanup and current added fixture. Coverage adds genuine same-task foreign Worker/controller denial, actual epoch rotation rejecting old cached cancellation, and current Owner historical reconciliation without renewing dispatch. Includes prior fresh-client status and three running cancellation modes. Temporal listener closed. Full repaired-source gate/new freeze/round2 reviews/rootgate remain pending; ticket16 not accepted.

Repaired-source strict lint run1 rejects items_after_test_module: new reconcile_attempt_cancel helper appears after supervisor observation_tests. Root read retained review1-clippy-strict-run1.log SHA256 `dae7d838e220d2d5f7f9f25225e2e7107dbd4db4761f0fb241f0823b67277bed`. Author requested behavior-preserving helper relocation before tests, no lint suppression, fresh exact locked/offline all-targets -Dwarnings run before full gate/freeze. This is a lint failure, not a passing verification result.

Root inspected repaired-source review1-clippy-strict-run2.log: Checking then Finished dev profile7.26s with no diagnostics, SHA256 `d8d74e232e019e63525136328696b74bc51f4157c86583ac113ab3688600ad5f`. Author terminal exit confirmation requested. This lint evidence does not replace behavioral full gate or independent review; full six-phase objective remains active.

Author confirms strict lint run2 terminal0/7.26s, matching root-read log. Full repaired-source all-case service gate handle12887 is now active, review1-full-gate1/run.log; root observed current compilation output, no terminal claim. New prelaunch cancellation regression covers a durable accepted MonitorGate controller claim followed by canonical cancellation and rejected resume without workload result. Final strict lint must include this last test addition before distinct round2 snapshot. Prior frozen review artifacts remain historical evidence.

Full repaired-source review1-full-gate1 terminal101, root-read log SHA256 `1f0b8ac69f3c957d9944ca1397e881b508f19c19ae2ae1163597e27ebfb18657`. New assembled cancellation4passed2.55s; gate later stops on assembled_operator2passed1failed5.76s. Root inspected whole-status equality assertions396/403 against independently sampled views after adding required DB snapshot_time; timestamp volatility must be validated separately while durable scoped fields remain equal. Author asked to prove exact diff and correct stale oracle, preserving required timestamp/control_revision and all bound/lookup checks. No full-suite pass or source acceptance claim.

Exact full-gate operator failure location clarified: root read retained panic at assembled_operator.rs:313, comparing CLI status to a separately sampled Kernel::status. Earlier noted396/403 whole-status comparisons are additional volatility-sensitive assertions, not the actual first failure location. Timestamp correction must preserve independently validated snapshot_time and equality of durable authorized view. Author remains active; no replacement run or acceptance inferred from elapsed observation.

Author exact operator diagnosis: independently sampled DB timestamp .724047 versus CLI .751243; correction validates both via database timestamptz parsing/nondecreasing ordering then excludes only top snapshot_time from durable equality. Root previous suggestion about396/403 volatility was overbroad: those are immutable exact-operation inspections, retained unchanged. More material P1 reconciliation concern discovered before freeze: legacy Confirmed Start with no associated physical workspace can satisfy empty all-stopped set. Author investigating an actual public regression; root requires absent/ambiguous effect-to-workspace binding remain quarantined with reserved capacity, never claim physical Stop from a receipt alone. Full gate remains failed and fresh freeze/review waits for this boundary proof.

### P1 cancellation completeness: actual unbound legacy Start red

Root independently inspected review1-legacy-unbound-red1: confirmed_legacy_start_without_bound_physical_stop_remains_quarantined fails0.37s, actual cancelled versus required quarantined. Log SHA256 `c1a4be3aa2c1e108424fea145f699026cc2c207a1917d2b92adc7be78de8acf2`. Author reports actual zero-model Python native runtime subprocess Create/Start effects through public Gateway binding/prepare/dispatch, with no supervised workspace. This confirms empty-workspace all-stopped proof is insufficient for effect-bearing legacy Start. Minimal binding-completeness repair/focused positive and negative/fullgate/round2 review remain required; no refund relaxation or acceptance allowed.

### Cancellation physical-binding completeness focused green

Root independently inspected review1-public-boundary-green1 exit0:5passed2.95s, zero filtering; log SHA256 `89be642b0c82102c7ef21a7fdd4fbb69ac3bd3aab02c7aed2b3c2482d0e30d0d`. Includes actual confirmed legacy Start without physical binding remaining quarantined, delayed MonitorGate cancellation, controller/epoch revocation, running nested effects and fresh status recovery. Root read minimal reconciliation predicate: each confirmed legacy Start must name an exact supervised execution bound to the same Attempt; absent/malformed/foreign binding quarantines, independently requiring all bound workspaces Stopped/Released. Receipt and usage guards remain separate. Genuine registered SDK measured-candidate positive must rerun unchanged for compatibility, then full gate/lint/new freeze/round2 review.

Post-binding-fix genuine SDK compatibility green: root independently inspected review1-bound-sdk-positive2 exit0, cancelled_attempt_cannot_submit_cached_report_or_accept_measured_candidate passes4.97s (1passed39filtered), log SHA256 `30752a0bb6a0de2d58f44383267be0b41449685c4e1a076b4ea6e1c7c1f5a705`. This preserves real registered SDK Worker/resume/independent measured Verifier cancellation behavior alongside unbound legacy quarantine negative. Operator focused suite, complete repaired-source gate, final strict lint and distinct frozen round2 reviews remain required.

Corrected operator focused suite green: root read review1-operator-final1.log,3passed6.32s zero filtering, SHA256 `311872e155eac85c3561e10fff70313a26ac257d031f2d08b542b41f7309dd82`; author confirms terminal0. Historical authority, natural issuer expiry,33-record bounded summaries/exact lookup remain tested. Complete repaired-source review1-full-gate2 is active handle37659; root observed cancellation5passed3.01s and capacity suite underway. No full terminal result or acceptance inferred.

### Complete repaired-source service gate green

Root independently inspected review1-full-gate2 exit0 and parsed all16 terminal summary groups:120passed0failed4ignored0filtered. Log SHA256 `1c84b20966e2821a84a5e042f546f4e2f292336af170d2222a47ca105cbb789b`; cleanup Temporal listener closed=true. Original Supervisor20passed13.62s and final Temporal7passed1ignored11.89s retained; deliberate unexpected-exit panic is contained by passing sigkill_fixture_rejects_unexpected_prior_exit, not gate failure. This proves standalone control-plane gate on repaired source; final strict lint/new exact freeze, independent round2 reviews and separate root integration gate remain required. It does not establish full six-phase completion or accept16 yet.

### Independent round2 frozen snapshot verified and dispatched

Root independently compared all48 source files against review2-source-manifest.json:29 baseline changes, zero byte/hash mismatches. Manifest SHA256 `6e009b273d834b18257ba62416ddd4022231134ed8149e1a539e97f0f15f2c36`; complete baseline-relative diff SHA256 `153f5911794ba66024966dbb68cbc697e5d426f7cbf557abaaf8e994291fc9d7`; coverage SHA256 `50220d298de58c9a3549a645e0019fb560fb20a9002f4c2de28c9f013939cd25`. Final strict lint terminal0/5.77s log SHA256 `41cc3d6824cd0f977e10de73235bab885f1ae3714f8f81644dcf96232b21662d`, fmt/diff0 author confirmed. Implementer explicitly frozen/terminal. Independent kernel16_spec_review and clean-context kernel16_standards_round2 both confirmed running against full snapshot; no builds/mutations authorized.120pass standalone gate does not replace these reviews or root integration gate. Historical EACCES remains explicit availability limitation. Ticket16 unaccepted; all later phases remain in scope.

Round2 independent reviewers remain active and have not issued final verdicts. Spec confirms P2 repair and traces accepted-task dispatch closure plus whether caller execution arguments are sufficient legacy Stop evidence. Standards traces possible mixed legacy/imported workspace allocation against one reservation. These are investigation leads, not proven findings or acceptance evidence. Root requested exact canonical requirement and reachable authorized public paths before classification; frozen source remains unchanged.

### Independent round2 verdicts: changes required

Spec reviewer confirmed frozen48file snapshot and prior P2 status repaired. New P1: cancellation physical binding trusts caller request.arguments.execution and can borrow stopped same-attempt workspace evidence for unrelated confirmed legacy native effect (supervisor1919; SPEC160–162,194–198). New P2: accepted status does not revoke Worker capture/admission/allocation/physical launch (artifacts712–725; mailbox450–462,1242–1275,1790–1801; supervisor216–227,328–340; SPEC151–152). Standards reviewer confirmed static P1 mixed legacy/imported first-Worker execution can allocate/start two physical workspaces against one reservation, and first imported admission can borrow already released legacy reservation (supervisor442/569; mailbox1323; SPEC315). No dynamic reproduction performed by reviewers. Root delegated actual public red regressions before minimal fixes, both allocation orders/postrelease reuse, genuine acceptedcandidate prepared/fresh dispatch fences, and borrowed-workspace cancellation negative with independently bound SDK positive. Both reviewers terminal; author resumed second repair iteration. New full gate/lint/distinct round3 freeze and independent reviews required. Preserve round2 snapshot/history; ticket16 remains unaccepted and broader goal unchanged.

### Review2 borrowed physical Stop evidence reproduced

Root independently inspected review2-borrowed-stop-red1 terminal failure0.47s (author confirms101): confirmed_legacy_start_cannot_borrow_stopped_workspace_from_arguments returns cancelled instead of quarantined. Log SHA256 `05f7c41bef16a27bbda3e52d48fd2b9f0cf93f85b10478af9be413f2c8681de5`. Fixture first genuinely executes/stops same-attempt workspace, then unrelated actual Python native Create/Start caller-names its execution. This dynamically confirms Spec P1 that supplied arguments are insufficient effect correspondence. Author remains preparing accepted-task/physical-reservation reproductions before product changes; immutable registered observation evidence may supply trusted correspondence only if exact bindings are independently established. No repaired result or new source acceptance.

### Review2 physical reservation violation dynamically reproduced

Root inspected review2-reservation-red1 terminal failure1.74s (author101), SHA256 `ef4066449a5062430a5d3a68d090ba2bf3775641c80e1c5d6136b961f661b4a7`. Retained reservation-sharing.json independently read: legacy-first bothfirst/secondRunning used_capacity1; episode-first bothRunning used_capacity1; released-legacy firstReleased secondRunning used_capacity0. All three forbidden actual physical paths reproduced, not merely static allocation claims. Author reports exact observed namespace members absent after strict fixture cancellation; no survivor claim independently checked by root here. Accepted-task terminal regression pending before product fixes; no acceptance/landing.

Root independently checked all15 recorded namespace member identities from review2-reservation-red1/reservation-sharing.json against current /proc stat PID/start_ticks/state: no matching live non-zombie processes. This strengthens cleanup evidence for recorded members only, without broad kill or claims about unrecorded descendants. Accepted-task regression remains pending; author owns ongoing isolated repairs.

Accepted-task reproduction review2-accepted-red1 aborts SIGABRT stack overflow before semantic assertions, log SHA256 `2d3fec604a79592932f60ead860b99b0b23d78de95638944899f0c528b393e50`. Root independently read log and requested previously established awaited owned-task poll-chain boundary with retained cleanup/default stack unchanged. This is fixture failure, not behavioral confirmation of postacceptance authority finding. Actual semantic red remains required before product repair; preserve exact cleanup evidence.

Accepted-task red2 fixture fails5.70s DependencyUnmet at existing verifier release line3875, SHA256 `fcfe042e8c8f545ef612f60ace11310a6f8bf4f38d2218bfe24c32cf04f8569a`. Root inspected release location and requested exact state diagnosis: fixture previously frees measured verifier to allocate pending execution, so subsequent existing release may be a duplicate. This failure does not prove accepted-task dispatch finding; no product guard relaxation authorized. Await corrected actual semantic reproduction.

### Review2 accepted-task authority defect dynamically reproduced

Root independently read review2-accepted-red3 terminal failure6.45s: genuinely_accepted_task_revokes_cached_new_and_prepared_worker_dispatch detects five permitted dispatch mutations after genuine acceptance. Log SHA256 `c3b7b81a3de558a4abb8d799b5cf5a14f23b80a2f6e0dc8cb5c8516d6ed73e46`; actual accepted-terminal-dispatch.json retained under kernel16-accepted-terminal fixture. Author confirms101. Prior stack/duplicate-release setup failures are not this semantic proof. All three round2 findings now have public dynamic reds; minimal repairs and corresponding positives/fullgate/round3 review remain necessary.

Borrowed physical Stop evidence focused repaired green: root independently read review2-borrowed-stop-green1 exit0, confirmed_legacy_start_cannot_borrow_stopped_workspace_from_arguments passes0.48s (1passed6filtered), log SHA256 `ad7e726532239ea1f0dcb5c0e2fb5dc6e0a884c7eba86ed4a9c05ae488b77183`. Author changed correspondence to independently registered immutable operation observation, rather than caller argument alone; actual bound SDK compatibility positive still required. Accepted-terminal artifact contains accepted/before/after/execution/forbidden_output/violations and remains retained from semantic red. Two other repairs, complete gate and round3 review remain open.

Independent bound SDK compatibility after trusted-correspondence repair: root read review2-bound-positive1 exit0, actual cancelled_attempt_cannot_submit_cached_report_or_accept_measured_candidate passes5.03s (1passed40filtered), log SHA256 `dc444195b82b04a814ec897e0756148699846570536ae321461884092f68aea9`. Genuine registered Worker/resume/independent measured Verifier cancellation remains compatible alongside borrowed-workspace rejection. Accepted-terminal and exclusive reservation repairs plus full gate/lint/round3 review remain required.

Accepted-task terminal fencing focused green: root independently read review2-accepted-green1 exit0, genuinely_accepted_task_revokes_cached_new_and_prepared_worker_dispatch passes6.56s (1passed40filtered), log SHA256 `80a9701ea7f72941b1a6857341b5e14b575cc385eafeb78076cbbc310222882f`. Current attempt dispatch guard joins task terminal status; fixture genuine acceptance then cached/new/prepared boundaries reject. This is scoped regression proof, not full-source acceptance. Exclusive physical-reservation repair, full gate/final lint and round3 review remain pending.

Exclusive physical reservation focused green: root independently inspected review2-reservation-green1 exit0, legacy_and_imported_episode_cannot_share_one_physical_reservation passes1.23s (1passed6filtered), log SHA256 `5b2666ed5b5c1ec36e865ad26dc6f7aa32621d28691a5cfd8fb9698765e3d4d8`. Current unaccepted additive0007 includes unique reservation.execution_id FK and immutable non-null/released binding trigger; author reports atomic same-lock still-reserved claim for legacy/imported-first allocation and launch/release validation. Focused three violation cases now reject; ordinary imported/independent charged capacity, exactly-once refund, genuine SDK full path and fullgate remain required for compatibility. All3round2 findings have focused repaired greens, not yet source acceptance.

Post-round2 complete gate review2-full-gate1 now active handle86571; root observed compilation output, no terminal claim. Author strengthened capacity regression exact DependencyUnmet/counters/Stop/one refund and terminal accepted-task counters/noeffects; additive0007 now adopts existing legacy workspace reservation binding without allocating new budget (root inspected UPDATE binding). Historical Owner inspection/release and immutable cached Accept receipt intentionally remain available. All-case terminal gate, final lint/fmt/frozen round3 review remain required.

### Round-two repairs: complete behavioral gate, lint still pending

Root independently inspected the terminal `review2-full-gate1/run.log`: 123 passed, 4 ignored child-process fixtures, 16 result groups, zero failures and zero filtered tests. SHA-256: `aaf708173ab253070f9869246d0c8c59785f58f9cab41a71cae69315d650685c`. This covers the repaired borrowed-stop proof, exclusive reservation binding, and accepted-task dispatch fences alongside the full existing suites. Author reported session 86571 exit 0 and closed Temporal listener. Root observed `review3-strict-run1.log` failing with two `collapsible_if` lint errors; this is not a passing final source check. Author is applying behavior-preserving condition cleanup and will provide fresh strict/full-gate results before freezing the third and final review snapshot. Ticket 16 remains unaccepted and unlanded.

### Final lint repair verification

Root read `review3-strict-run2.exit` as 0 and the associated log as a completed strict check (9.18 seconds), SHA-256 `424daedb48e28d25aa69e720b13a45081ae9a06137bf7870ab9fef0a90e0f88b`. The earlier lint failure remains retained. Final full behavioral rerun and frozen round-three independent Standards/Spec reviews remain pending; this strict result alone does not accept ticket 16. Root also reread conditional ticket 18 and ticket 19: Kata remains deferred with Git retained, and jj evaluation still waits for kernel16 acceptance. No downstream ticket was prematurely closed.

### Final cleaned source: complete behavioral rerun observed

Root followed the same final rerun (author session 81097, observed Cargo PID 4061814) to terminal output; the PID subsequently disappeared and the log ends with successful real Temporal recovery/replay and doc tests. Root independently summed all 16 result groups: 123 passed, zero failed, 4 ignored subprocess fixtures, zero measured and zero filtered. Log `review2-full-gate2/run.log` SHA-256 `10994ca495c27760ac8dc1a30bc4df1218805bd77b7fa4e89d32f931c14fc6f2`. Wrapper exit/cleanup confirmation and frozen final review snapshot are being collected by its owner. Third-round Standards and Spec reviews, root repository gate and exact-tip landing remain pending. No live provider guarantees follow from these model-free results.

### Third and final frozen review dispatched

Author confirmed final wrapper exit 0 and completed cleanup. Root independently validated all 48 manifest file bytes, unchanged accepted root Cargo files and migrations 0005/0006, and exact accepted HEAD. Final manifest SHA-256 `d1e426646b281154aa26590142b5ac3a2da80f71334dab2d9c4b88daff56ed54`; complete 29-file baseline diff `2648776770cca01dd882ba0a3d13e2048e322c6af8e995f44d7022deaa11c8b7`; coverage `f4e002d73ec934dd740522161ae03d956b810982491a13eb2b8ab78bef11721a`. Fresh clean-context Spec reviewer `kernel16_spec_round3` started. A second new thread hit the manager thread limit; root resumed the independent Standards reviewer `kernel16_standards_round2` against the new hashes instead, without sharing author context. Both reviews are read-only, cover the entire baseline-relative candidate, and are the third/final allotted iteration. No verdict is inferred from passing tests.

### Final Standards verdict

Independent Standards reviewer returned PASS/no blocking findings against round-three hashes and all 48 source files. The reviewer assessed the complete 29-file baseline change, including exclusive reservation allocation/launch/release, additive legacy binding migration, registered Stop evidence and terminal dispatch guards. Review was read-only; migration upgrade assessment is static, and the historical EACCES availability limitation remains unresolved. Independent Spec review remains active and is investigating a status visibility gap: the current status query reads episode runtime operations but not the separate legacy gateway journal. Root independently inspected kernel.rs status lines 236–266 and confirmed that query structure; final semantic verdict remains pending. No source mutation, acceptance or landing follows the Standards result alone.

### Final Spec verdict and bounded-budget escalation

Independent final Spec review returned CHANGES REQUIRED: P2 status omits unresolved legacy gateway operations. Both CLI and HTTP call kernel.rs status, whose operation query only joins episode_runtime_operations; existing genuine legacy fixtures retain unknown Start/usage and occupied capacity without such an episode journal row. The empty status list is a source-based inference, not a newly executed regression. Standards passed; all prior three blockers appear repaired. Root requests one additional bounded repair/review round: authorized bounded legacy operation visibility, genuine unknown-start/unknown-usage regression and authorization/pagination controls, full gates and independent review. Repository three-iteration budget is exhausted; no further source changes or landing are authorized by this request alone. Await owner response; retain exact frozen source and all evidence.

### Blocking audit: pending repair-budget decision

Same condition has persisted across three consecutive goal turns since final review: exhausted three-iteration budget, confirmed P2 unresolved legacy status visibility, and no owner response to the bounded extension request. Root revalidated all 48 frozen candidate hashes and the canonical downstream blocker graph: 18/19/20 depend on16;21 depends on20/16;22/23 on21;24/25 likewise remain downstream. Review evidence and completion audit are durable; no live verification job remains to await and no remaining independent execution ticket is ready. The full goal is not achieved. Stop automatic goal continuation pending the owner decision rather than silently exceeding the repair budget or repeating equivalent audits.

### Owner approved continuation

Owner explicitly approved “for all things” after the bounded extension request. Root recorded decisions/20261002-continuation-authority.md and resumed the sole-writer author for the additional status repair/review round. Frozen review3 evidence remains immutable; actual public red is required before implementation. Planned private live verification is now authorized within the finite proposed envelope, but technical isolation and hard spend/provider prerequisites remain unproven. No model traffic or landing occurred in this resumption step.

### Authorized status repair: reproduced, repaired and frozen

Root inspected the actual public red log: both genuine legacy modes completed before CLI/HTTP missing unresolved visibility asserted, terminal101/1.21s, SHA `a240f759a7d0fe85d6878c5df5cba6d4f7aee8f3ef4d7ccfe798b22135280522`. Both-mode focused green SHA `1e657c74badc3dbefec6e3165fa3b5ee24dce60093be977995db93235613e8d4`; merged operator controls SHA `f50a2858ba0804432080de2d3c2217bc81e35b7e6cc96fa3d3ea909ce5d4445d`. Root verified these hashes independently. Final full gate has 126 passed/4 ignored/16groups/zero failures or filtered, SHA `d52b965d01b774280689570b580e186d0d83e444da54de1ae8e3b10d0a2ac559`; author reports wrapper93191 exit0 and cleaned services. Strict/fmt/diff pass.

Root validated all48 frozen source bytes and new artifacts: review4 manifest `0007f56d687897e10601ce277084127e5bc1d1dbb5994efde4d5944a7806da7a`, full baseline diff `ecfac3ac71d764d68e998dc16df07eeedd52b4975fe028d1b4b6469a5f156371`, coverage `b22030a7b32244826e50508aed9d4e9aa8afede477986ae345a19651e6f07613`. Independent Spec and Standards reviewers resumed against this exact snapshot under the explicit owner extension. Prior frozen snapshots remain preserved. Acceptance/root integration/landing still pending verdicts.

### Round-four reviews passed; root integrated exact source

Both independent axes returned PASS/no blocking findings, with all48filebytes verified. Root copied only29 manifest changes, verified the entire48-file destination hash set, and retained review4 artifacts under evidence/kernel16/. Fresh root repository gate started as session63483; it includes checks.d/70 real PostgreSQL/Temporal integration, not merely Python/root Rust tests. Branch defect gate and acceptance/landing are still pending. Detailed verdicts are preserved in evidence/kernel16-review4.md.

### Root gate1 failure retained; no landing

Root session63483 exited1. Exact integrated source remains the reviewed48-file snapshot. Tracker lint rejected the root-added approval comment date format; root corrected its own timestamp to the required ISO form and fresh tracker lint now exits0. The product gate also failed in two_running_capsules_refuse_third_and_cancel_deadline_refund_once: cancellation quarantined execution0814d200-2e8b-471e-abdf-a3d60bf5a467 after process2316113 membership observation returned EACCES. This matches the historical unresolved availability limitation; no stopped/refund/acceptance claim follows. Failure log retained as evidence/kernel16/root-gate1-failed.log SHA256 `e9c76e33aebe1b6ed90e84106a1d920cf4857351e0dab14f937e0644f070c362`. Author resumed for bounded diagnosis of exact owned fixture and survivor inventory; source edits/retries/guard weakening are not inferred from a transient proc observation. Root Rust suites passed but do not override this complete-gate failure.

### Root-gate availability diagnosis: current cleanup, cause unresolved

Root independently observed every exact recorded init/member/controller/launcher PID from the failed execution absent in current /proc; no signal sent. Author independently saved proc-inventory.json under root-gate-eacces-diagnosis1, also finding failedPID2316113 absent and no current same-UID matching/unreadable nested namespace, plus no owned PostgreSQL container. These observations establish current absence of recorded identities, not complete historical effect cessation or the EACCES cause. Root traced inventory to metadata/status/identity reads sharing a collapsed error string; namespace-link access during transient process exit/exec or an unrelated nondumpable same-UID nested task are hypotheses, not verified causes. Original failed syscall/state cannot be reconstructed from retained log. Reviewed source remains frozen; bounded instrumented reproduction is the next evidence boundary. No retry, refund, acceptance or landing occurred.

### Bounded instrumented reproduction1

Root verified focused fixture terminal exit0, one passed/three filtered,31.28s; log SHA256 `3cacd3869a18776eace422090fa960543b103e6adcf41500a356a605a1a0fb0e`. Owned Temporal listener closed; failed-PID observer output is empty. Failed-readlink trace contains no EACCES/EPERM and tracerPID is absent. This does not reproduce or explain the prior failure; strace may alter timing/dumpability. Source remains frozen. Root requested independent read-only architecture assessment of bounded complete-inventory retry while the verified namespace pin remains held; no error may be ignored, inferred process signaled or uncertain capacity refunded. This is a proposed diagnostic direction, not an implemented fix or acceptance claim.

### Independent inventory-retry assessment and selected bounded repair

Independent reviewer confirmed complete-scan retry can preserve failclosed only under the held verified namespace descriptor, finite observation bounds, no partial signals, persistent-error quarantine/holds and no reopening committed quarantine. Author independently confirmed terminal quarantine behavior. Root recorded decisions/20261002-inventory-observation-recovery.md and authorized deterministic red/green plus focused physical proof before new freeze/full gates/reviews. Historical EACCES cause remains unproven; no guard-relaxation or native retry is authorized.

### Bounded instrumented reproduction2 remains inconclusive

Root verified second focused run exit0, one passed/three filtered30.96s, SHA `cfa5f91b81668640fbb3580414bb963529f03dab09cce8af88d084b9e3d62d84`. Temporal listener closed, preattached failed-PID observer empty and failed-readlink trace contains no EACCES/EPERM. A passing traced run does not explain or resolve the historical failure. Root directed transition to deterministic NamespaceObserver red for the independently assessed bounded whole-scan retry; the three-run maximum is not a requirement to consume a third inconclusive tracing attempt. Source remains unchanged pending the regression.

### Deterministic inventory availability red established

Root inspected namespace-scan-repair/red1.log: approved NamespaceObserver seam injects PermissionDenied then permits a complete empty inventory while the verified held pin remains supplied. Unmodified production decide_namespace returns errno13 rather than taking the fresh scan; the new test fails as required (onefailed/fourfiltered). Log SHA256 `b81d61c1a18d33c6cdbca813d72ddf46b83a3e8c3360d7c3912beaf488323c57`. This proves the selected transient-observation behavior is absent, not the original historical syscall cause. Exact review4 supervisor preimage retained by author. Product retry implementation and persistent-error/membership/terminal-quarantine controls remain next; no stopped/refund or landing claim follows.

### First inventory-retry behavior green

Root inspected green1.log: the previously failing injected PermissionDenied/complete-empty-scan case now passes, onepassed/fourfiltered. SHA256 `88d5ec6bcc7b9aa7ddfbacab04bd1e8a50727bb25253f9175a1b73fbbbee3007`. Author selected at most3 complete attempts,1ms between attempts and20ms admission window for another attempt; individual synchronous proc reads/scans are not claimed hard-time-bounded. PermissionDenied preserves typed pid/operation/source provenance; non-permission errors remain immediate failures. Same verified descriptor retained, partial scans discarded, frame revalidated, committed quarantine remains terminal. Remaining negative controls, actual physical cancellation, final service/root gates and independent review are still required. The narrow green is not ticket acceptance or proof of the historical error cause.

### Historical kernel review evidence retained in root checkout

Root copied55 immutable historical author snapshot/diff/coverage and terminal console-log artifacts (2,127,130bytes) into evidence/kernel16/author/, validating exact preexisting destinations and recording per-file sizes/hashes. Archive manifest SHA256 `163a1db56664db7bf8803bb44ebb43af51dd0e6117b1b1ac12118c2332dc7d1e`. Runtime stores, SQLite, credential/configuration files and active repair output were excluded. This preserves failed and passed reviews/gates for fresh-clone investigation; it does not accept the current repair or substitute for fresh verification. A separate bounded read-only Trae prerequisite researcher is checking installed primary provider-routing/spend contracts while the sole-writer author finishes inventory controls; no services/model traffic authorized by that research assignment.

### Trae provider prerequisite research captured

Independent bounded read-only researcher verified installed0.207.1/hash and help plus candidate embedded OSS keys/protocol descriptions. Root preserved findings and explicit unknowns in research/live-trae-provider-preflight.md. Embedded strings do not prove app-server routing or a spend ceiling. Planned synthetic owned-endpoint proof still requires kernel16 and executable isolation; no provider requests or services occurred. Owner launch approval is recorded, with technical enforcement remaining a prerequisite.

### Physical inventory repair proof observed

Root observed all7 canonical cancellation tests plus3 real capacity tests pass (one ignored child fixture) in namespace-scan-physical1; original two-running-capsules/third-rejection/cancel/deadline/refund case now passed32.39s. Cargo process no longer present and cleanup record exists. Root retained focused red/green, all7 observer controls and physical run/cleanup records with per-file hashes under evidence/kernel16/inventory-repair/. Author terminal wrapper confirmation and strict/fullgate/frozen independent review remain next. Passing physical proof does not establish historical EACCES causation or guarantee every future proc scan succeeds. No acceptance or landing yet.

### Inventory repair strict check and physical cleanup verified

Root read strict-run1.exit0 and terminal strict log10.94s; retained its exact bytes, SHA256 `9b991f4e43ca011717d82a673d33a6f9d8bab7e5ee7547309fc9cf008f4e3d1e`. Root also read physical known/cleanup.json: both executions stopped/released, owned fixture removed and SDK waited. These are physical-fixture cleanup claims supported by retained records, not a resolution of the historical unknown EACCES cause. Full-service exact-source gate/freeze/review and root gate remain required.

### Inventory repair review5 passed and exact source integrated

Root independently verified all48files and complete gate129passed4ignored16groupszero failures/filtered, SHA967e70a9d55e7ee26db126767d610a3ad7c84a3ec50f4c1be6e5211d82855f86. Both independent axes PASS/no blocking findings. They explicitly qualify20ms as not a hard syscall/scan-start/lock duration bound; max3 scans holds. Root appended that qualification to the recovery decision, copied only supervisor.rs from frozenreview5, verified all48destinationhashes and retained new manifest/diff/coverage/fullservice log. New complete root gate follows; branch defect review and landing remain pending. Historical failed rootgate retained, not erased.

### Root gate2: new Temporal failure retained

Root session44219 exited1. Tracker/Python/root Rust and new physical cancellation/capacity checks passed, but real_temporal_mailbox_wait_survives_worker_death_and_lost_answer_hint failed at temporal.rs1375: canonical Continue-As-New incarnation did not advance within its100x50ms observation loop. Root retained exact failed gate log SHA256 `7b902793c6d31b83a0c59afc9965e3a40464bf058aa4fbc813c3d78a35b43ab8` as evidence/kernel16/root-gate2-failed.log; full owned service evidence/Temporal SQLite remain at .scratch/team-control-plane/root-gate2-service. This is a new failure, not the EACCES failure. Author resumed bounded history/registry/timing diagnosis; no whole-gate retry or assertion weakening was authorized. Reviewed source remains unaccepted/unlanded and no commit was made.

### Temporal root gate2 failure: history confirms observation deadline mismatch

Root independently checked retained raw protobuf history/execution hashes and event timestamps: blocked Activity completion event8 at1790958515.442124511, doomed-worker WorkflowTaskStarted event10 at1790958515.447289054, WorkflowTaskTimedOut event11 at1790958525.449655590, approximately10.002s later; event12 reschedules. This in-flight task cannot recover inside the test100x50ms incarnation loop. Sticky schedule-to-start1s does not shorten already-started task timeout10s. Actual registry at assertion time is unavailable because wrapper removed PostgreSQL; that limitation remains explicit. Raw artifacts retained under evidence/kernel16/temporal-diagnosis/ (history SHA dd2c434263144070159d25f687283f80eb2059541f88f61a879056d638fc85e5; execution SHA1074cd508fd4bb2b825cdd1ccb3a943b0c18d7a3f438643ea91266dedba57cdb).

Root authorized a TEST-ONLY30s monotonic incarnation observation deadline and timeout diagnostics. Preserve all production/sticky/task/mission/activity/episode budgets and actual completed-history/replay/duplicateanswer/nohint recovery assertions. Existing failed root fixture is the public red; focused fresh run/fullgates/freeze and independent review remain required. No new artificial worker gate or product recovery patch is needed for this verified minimal correction. User status answered: kernel remains unlanded and full goal unfinished.

### Temporal observation correction: focused proof and live full gate

Root read genuine mailbox/worker-death/lost-hint fixture focused run:1passed7filtered6.85s, SHA `df5c8ab5cf828ed759f5e1204b2a08567fa0554c3d2872e6ffd0b0219ce1904a`. Author confirms wrapper62745 exit0 and closed Temporal listener. Root preserved focused log/cleanup and strict log under evidence/kernel16/temporal-diagnosis/. The focused pass does not independently reenact the original10s timeout; retained rootfailure history provides that causal evidence. Test-only30s observation deadline preserves product settings and all recovery assertions. Fresh complete service gate67162 is live; root confirmed CargoPID2675144 and progressive cancellation/capacity output. No new native/runtime behavior, acceptance or landing claim follows yet.

### Test-only Temporal correction full gate frozen and independently validated

Root verified all48review6source hashes and sole delta tests/temporal.rs; full service log SHA9fb8f831786034fc3fa8b28969c20a60098636bbc8eb44d789030a452718b241 sums129passed4ignored16groupszero failures/filtered. Author confirms67162exit0/cleanup. Manifest55b0ffbd1aa5062e2f556f5002cd396e54f25843ce30a3c25a16886175605fdf; diff33b026155a552a382a91dd609af5545b6c93d756eaea2c1fc13840f8a86d1325; coverage56c48b61e98869ccbed7f37a78b75effa1d6980ee689289d39c5b26f98ed1826. Independent Standards review6PASS; confirms unchanged incarnation predicate and allsubsequent history/replay/nohint/duplicateanswer/checkpoint/secret/no-fabricated-gateway assertions, no production setting/budget changes. Deadline does not hard-bound awaitedRPC runtime. Spec review remains active; no rootintegration/commit/landing follows single-axisPASS.

### Review6 both axes PASS; root exact integration

Spec alsoPASS/no blocker, confirming actualnewincarnation/completedActivity/nohint/duplicateanswer/checkpoint/replay predicates preserved and no recoverySLA claimed. Root copied solely tests/temporal.rs and verified all48reviewed sourcebytes, retained manifest/diff/coverage/finalservice log. Fresh complete root gate3 follows; no acceptance/commit/landing until terminalrequiredchecks and branchdefect gate. Earlierfailedrootgates and exactcausalhistory remain retained.

### Root gate3 passed; committed candidate enters branch verification

Root session22595 exited0; complete repository gate ended `check: all green`, including129 control-plane tests passed/4 ignored and the root test matrix. Temporal fixture cleanup records listener closed. Retained `kernel16/root-gate3-passed.log` SHA256 `7a4d4e6e5a3e40d52b5509f6e6e2b6d2d9e7cc8e53cdcd1bbe44c6ce2af6fa58`. Root revalidated all48review6 source hashes, staged exactly30 baseline-changed source paths, and committed candidate `2316e09` under explicit owner authority. `roborev review --branch --base 1f164156261a3aa927c823919bf73d6e6d11a771 --wait` enqueued Codex job553; session77379 observes that job. Acceptance and landing remain pending its verdict. Main remains1f164156; the full live/lead/restore scope remains incomplete.

### jj prerequisite recheck correction

While branch job553 runs, root mistakenly invoked PATH `jj --version` before consulting the retained preflight. `/opt/tiger/ss_bin/jj` is an unrelated SSH helper; it attempted hostname `10.6.23.--version` and failed DNS resolution. No VCS operation occurred. Read-only inspection confirms the shell wrapper. Earlier preflight already documents this collision and the owned source-built `.scratch/team-control-plane/jj-preflight/jj-pinned` release0.44.0. Future evaluation uses that explicit pinned path and existing fixture evidence; this incident does not establish or change any jj capability.

### Renewed kernel claim and actual jj executable revalidation

At16:49UTC the prior kernel claim was expired. Root renewed ticket16 for codex-kernel24h through tracker.py; tracker lint passed. Reviewed source2316e09 and main1f164156 remain unchanged. The historical jj-pinned convenience path is absent. Actual owned source/target/release/jj exists and independently reports0.44.0-af45d57de7163cc5f17f3df178a31577e4951ea3; SHA256 `15fe28e44398552177ce01ba69dd17b52151e8e604ccf82fe587acd40ba13be4` matches retained build evidence, and source HEAD matches that pin. Use the verified concrete absolute executable for later ticket19; no rebuild, install or shared workspace mutation occurred. Branch job553 remains running per authoritative daemon status.

### Branch defect review553 requires repair

Review session77379 terminated exit1 with three findings: repair authorization/budget bypass after failed verification (High), stopped-before-result-persistence workspace-release race (Medium), and dispatch fence blocking historical gateway observation/Temporal drain (Medium). Retained `kernel16/branch-review553.log` SHA256 `e98cb1bb5c7b5fed7fe0ae34eef79d627d5f64fd23f99db86a9d78663e71f7c2`. Candidate2316e09 is not accepted or landed. Root delegated independent diagnosis, public red regressions and targeted fixes to the existing author in its owned isolated tree under owner continuation authority. No failing verdict is waived; fresh conformance and branch verification follow any repair.

### Independent assessment of branch553 contracts

Root and independent Spec context inspected fixed candidate2316e09 and confirm allthree findings. Repairs must retain initial yield/resume, exact authenticated command replay, independent current-candidate re-verification, epoch/scope/generation/workflow guards, genuine prelaunch cancellation release and legacy non-episode workspaces. After failed verification, exact charged authorization gates candidate replacement and all fresh/prepared Worker effects; budgets/counters stay unchanged on rejection. Launched episode deletion requires immutable monitor result persistence bound to durable launch identity, not controller absence. Existing gateway journals remain observable without runtime I/O after terminal-task fencing; Prepared fenced work stays visible without whole-drain failure or fabricated stop/refund. Author develops deterministic public regressions before product changes. SPEC failed-verification state wording was explicitly flagged for reconciliation with the existing Repair transition. No future patch acceptance is inferred.

### Branch repair regression harness evidence

Root retained first two terminal regression logs under kernel16/branch-repair. Red1 failed compilation(E0507) and red2 compiled but aborted with stack overflow(SIGABRT) before the contract assertion. Neither proves the intended behavioral regression. Root notified author to correct the harness before production repair and preserve physical cleanup; no product patch or acceptance follows these failures.

### Abort cleanup and ongoing stack diagnosis

Author independently identified SDK children left by harness aborts via exactPID/start/executable/arguments and stopped only those pidfd-pinned processes. Retained red2/red3 sdk-cleanup.json report terminalpidfds; root inspected them and each Temporalcleanup listenerclosed record. These do not prove complete namespace inventory cleanup. Read-only disassembly diagnosis is active; no fourth fixture or production patch was launched at this boundary.

### Stack diagnosis authorizes a structural harness correction

Root independently inspected emitted parent-poll prologue: stack probe0x176000 plus remainder0x3d8 totals1,532,888bytes; retained stack-diagnosis.txt SHA256 `91ba309eabde303e7c5359b17b83e7428148de338c1032885c7c98720899fae0`. This measures emitted frame allocation, not future-value size or complete peak stack. Author compares prior1,402,840byte measurement; added inline spawned-future construction increases parent stack. External helper construction of the subcheck task is authorized as a test-only correction preserving default stack and every assertion. Reaching the behavioral regression remains required; no production repair or PASS claim follows disassembly alone.

### Repair budget bypass reproduced through public interfaces

Red4 terminal101,6.19s, defaultstack: actual independent measured failing verification followed by uncharged Worker input Capture admitted and same-candidate/new-key HTTP Submit200 revision9. Assertion rejects both successful bypasses. Root independently checked log/hash `4df6f3b221ee1eff53fecefcdca931555164f1cb1bdf90a14afbdf8bec86bf9c` and preserved log/Temporalclosedcleanup under branch-repair. Author reports no uniquelyownedPG or exactmatchingSDK/supervisor processes remain aftersemanticpanic. This is genuine regression evidence, unlike priorcompile/stack failures. Production patch remains pending; gateway historical/pending public red is active.

### Historical journal fencing regression reproduced

Root inspected actual compiled public cancellation fixture historyred2: after canonical cancellation, retained workflow journal observation fails DependencyUnmet;0passed1failed7filtered0.33s. Retained history-public-red.log SHA256 `32dd50316b8b0b94fe81cabf89c1d4f49d45867da118ac19e02a8e875b8b4920`; cleanup records Temporallistenerclosed. This is direct workflow-operation-driver regression evidence, not yet a fresh successful wholeSDK workflow recovery proof. The earlier historyred1 alias compilefailure is not behavioral evidence. Third finalization-race regression remains pending.

### Allthree branch findings now have genuine behavioral reds

Finalizationred1 terminal101/test5.45s: actual launched episode canonicalStop followed by release succeeded when immutable monitor result was still absent; fixture expected rejection. Root inspected exactlog/hash `a5e4708f4c6029b17fe2ee98a4d1f2d6f4808113de6c895988a74dafd141a7f1` and preservedTemporalclosedcleanup. Author reports authenticatedempty membership, release deletingworkspace and resultremainingnull after5s, with exactpidfd-pinned trustedlauncher pause/resume and finishedfixture beforeassertion. No matchingSDK/supervisorprocess remained. Allthree public regressions now reproduced; author begins minimalproductfixes and focusedgreen checks before anyfullgate.

### Focused boundary repairs green, fullcandidate still unaccepted

Root read boundariesgreen2 complete libtest summary9passed0failed0ignored0filtered4.78s; retained log SHA256 `a370caf2a0388807878459254cc9477711928429973f4708841513f5c45c2f12` andclosedTemporalcleanup. Tests include historicaljournal observation aftercancel and stoppedlaunchedepisode workspace retention untilresultpublication, plus existing cancellation/unknownusage/capacity/ownership controls. Gateway fallback nowuses read_guarded with authenticatedepoch/scope/generation and exactWorkflowRef guard in serializedtransaction; no fallbackauthorization is implicit. This focused evidence doesnot replace remainingrepairbudgetfix, stale/closedWorkflowRef and realSDKdrain evidence, frozenmanifest/twoaxisreview/rootgate/branchreview. No sourceintegration orlanding occurred.

### Repair guard provisional review: invalid evidence must not grant work

Root inspected the author live current_failed_verification/implementation_enabled helpers. Exact current candidate/report, failed eventsequence, currentepoch/generation and subsequent Repair charge are now represented. Provisional concern sent to author: validation failure of an existing report currently returns no failed gate, implicitly enabling implementation. Policy/base/generation changes may make earlier failed evidence invalid; this needs explicit authorized-revision semantics or fail-closed regression proof, distinct from an initial task with no report. Latest focusedrepairgreen1 aborted on stack overflow beforebehavior; no green or sourceintegration claim follows.

### Budget guard harness diagnosis and invalid-report disposition

Root verified emitted fixturepoll stack subtraction0x175000+0x648=1,529,416bytes; retained budget-stack-diagnosis.txt SHA256 `89d5fe06a3aa10185f7b28618b2711ff497256ae5a384d1dff1a0632d4b3c15e`. Author measured smaller producthelper frames; exact overflow callee remains unproven. Test-only named owned-task boundary around the artifact/publiccall phase is authorized to preserve defaultstack and every behavioral assertion/order without adding productionretry or scheduling behavior. Author agrees invalidexistingreport must failclosed rather than count as absent evidence; OwnerConfigure cannot silentlyclear failedgate/repairbudget. Focused proof remains pending.

### Repair budget focused fixture green

Root independently read repairgreen2 complete result1passed0failed41filtered11.33s, including two genuineSDKrepairrounds(newcandidate/Worker0/Verifier1), andTemporalclosedcleanup. Retained log SHA256 `ed0960f359abf29cf5b4ee97d7a78e257ae433e6627a8fff2d3264fd29560118`. Author currentfailedgate helper now classifies intrinsic failure without dropping it when policy/spec/generation invalidate priorproof; successful priorproof remains distinct. Remaining retry/authorization controls, actualSDKfenceddrain, fullmatrix/frozenmanifest/reviews/rootgate/branchgate are pending. Focusedpass doesnot accept orlandcandidate2316e09 or its unpublishedrepairs.

### Focused repair terminal confirmation and cycle binding

Author confirms repairgreen2 wrapperexit0; root preservedlog independently matches SHA256 `ed0960f359abf29cf5b4ee97d7a78e257ae433e6627a8fff2d3264fd29560118`. Test-only ownedtask boundaries preserve23realArtifactService calls/order/defaultstack; no productionspawn. Author validates repeatedcontent-addressed failure canreuse reportID but constitutesa laterverificationevent; unique candidate/reportcharge alonewoulddeadlock. Additiveunaccepted0007 needs verification_revision chargeidentity with cumulativefinitebudget unchanged. Those controls and actualSDKdrain precedefullcandidatefreeze/review. Main and rootcandidate remainunchanged.

### Exact repeated-report repair cycle control passed

Root read cycle-green2 pass1/10filtered0.42s and inspected fixture: declared shell gate actuallyexecutesexit17 eachcycle; samecandidate/output reportcontent hash remains identical. ExactVerify replay produces no newevent; later genuinefailure rejects oldchargedpermission; exactRepair replay counts once; threecycles authorized then fourthRepair BudgetExhausted, consumed_repairs3. This proves ordinary artifact-service cycle semantics; registeredsupervisedSDK evidence remains separatelyscoped. Log SHA256 `e33cdfafa18d3744cfd0e1cfc8cc32413ae9ede89ee7f7754e4ecb543a618037` andclosedTemporalcleanup retained. Fullcandidate/review/landing stillpending.

### ActualSDK cancellation drain focused proof

Root read sdk-green1 complete1passed0failed41filtered6.18s/Temporalclosed. Extended actualSDK measured-candidate cancellation fixture requires completedActivityhistory containing retainedjournals and gateway_dispatch_fenced PreparedSteer without nativeeffect. Retained log SHA256 `469b49f37f41c95b94f876cd29a72fc0719c49d69eccb05072a9ca2365ccb148`; actualhistorypredicate is in tests/assembled_vertical.rs. Author reported handle44315; terminalwrapperconfirmation/frozenfullsource remainpending. No fullmatrix orcandidateacceptance inferred from thisfocusedproof.

### Preadmitted worker failed-gate control green

Root read prepared-green2 complete1passed0failed42filtered6.46s andclosedTemporalcleanup; retained log SHA256 `4c7a84e6500cde28bac7372a2c3c9269afbb750d376608c84869feea8e2c2416`. Actualindependentlymeasuredfailedgate fences alreadyallocatedWorkerReady physicalstart and furtherimplementation withoutchargingnewwork. Priorpreparedgreen1 oracleexpectedDependencyUnmet whileexistingSupervisorcontract refusedStaleGeneration; correctedexactoracle preservednooutput/counters/allocation/admission refusal assertions. Fullcandidatepolicycontrols/fullgate/freeze/reviews/landing remainpending.

### Branch repair final control suite

Author run `branch-review-controls-green1` terminated with wrapper exit 0. Independently checked log SHA-256 `1b001d5058b0e16fc34f695f30e6d1a56479c9f3b074f5dc22e503f4d4e8dec8` and retained `kernel16/branch-repair/final-controls-green.log` and cleanup record. Artifacts: 11 passed; assembled cancellation: 9 passed; no filtering. This proves those controls only; fresh-verifier report fencing, full matrix, independent review and acceptance remain pending.

### Fresh verifier Complete scenario

Independently verified author fresh-green1 wrapper exit 0, Temporal listener cleanup and log SHA-256 `ca9a78484a7a890da50094a880332fe3cf2640ef88c1c1f4ad9a37664e7bf9a0`. Actual SDK Complete scenario passed (1 passed, 42 filtered, 7.36 s), rejecting old successful report reuse and executing a fresh same-candidate verifier. This is focused evidence only. Independent design assessment supports immutable admission revision >= latest failed gate revision; pre-failure late completion, mixed proofs and later-failure controls remain to inspect in frozen source. Full matrix and acceptance remain pending.

### Verifier admission freshness contract (provisional source)

Read-only author source inspection confirms `mailbox_episodes.admitted_task_revision` is populated from task revision during admission, with an additive migration trigger forbidding later mutation. Assembled positive Verify clearing a failed current gate checks every report execution against its registered verifier episode and requires admission revision >= the failed verification event sequence; absent/NULL provenance fails the SQL comparison. This is a provisional implementation observation, not frozen-source acceptance. Author reports admission and Verify use the existing serialized PostgreSQL transaction boundary. Independent Spec design assessment agrees with this contract. The focused old-success rejection and fresh execution evidence exist; no separate physical late-completion or mixed-check fixture is claimed. Accepted-terminal compatibility and fresh full matrix remain pending.

### Accepted-terminal duplicate-release helper failure

Author accepted-green1 terminated 101; independently retained log SHA-256 `c8a84a88099444a7dc3e1c937d8ac911a973a9c65e2d10b0765f692ee8db46fb` and Temporal cleanup. Fresh-verifier helper tried to release a negative verifier already released by pending-worker setup and received DependencyUnmet. This is compatibility fixture failure, not a product behavior red. Author changed only the helper to inspect actual Released state before skipping duplicate release; accepted-green2 is the rerun, not yet accepted.

### Accepted-terminal actual SDK compatibility pass

Author confirms accepted-green2 wrapper terminal exit 0; independently checked hash `e69c4c63882c8fe48d25b3a46c3a21234f7d78fdc89ec7adc7bb8a4378708079`, retained log and Temporal listener cleanup. Actual SDK acceptance/terminal dispatch scenario passed: 1 passed, 42 filtered, 7.52 s. Full matrix, independent frozen-source review and landing remain pending. Strict all-target warnings check launched by author as session93362, branch-review-strict-run1.log.

### Branch repair strict all-target checks

Retained strict-run1 diagnostics (13 needless-borrow fixture warnings) and strict-run2 clean log. Author confirmed strict-run2 terminal 0 for locked/offline all-target Clippy with -D warnings; log confirms completion in 5.47 s. Correction removes redundant borrowing in fixture-owned task calls; no guard or runtime contract changed. Full real-service matrix is independently observed live (cargo PID3636098; assembled vertical child3681056), not yet a complete pass.

### Review7 frozen source and full author matrix independently checked

Root independently verified all 48 frozen source hashes, manifest/diff/coverage/log hashes, wrapper run.exit 0, Temporal cleanup, and all 16 unfiltered group summaries: 133 passed, 4 ignored. Exact artifacts retained under evidence/kernel16/review7-*. Independent clean-context Standards and Spec reviews launched against this frozen set. Source integration, fresh root gate, committed branch review and acceptance/landing remain pending.

### Review7 completed-Activity predicate mismatch

Independent Standards review found frozen cancelled_sdk_drain_phase recursively scans every history data payload despite coverage claiming restriction to ActivityTaskCompleted result. Root independently read and confirmed the predicate. Reviewer inspected retained actual histories and found matching progress at events/6/attributes/ActivityTaskCompletedEventAttributes/result/payloads/0, supporting observed completion but not the claimed implemented oracle restriction. Narrow predicate/coverage repair is required; no production defect inferred from this finding. Independent reviews continue against frozen review7, which remains unaccepted.

### Review8 completed-Activity oracle focused pass

Root read the narrow test-only predicate selecting event_type12 and ActivityTaskCompletedEventAttributes.result.payloads. Independently verified focused actual SDK run.exit0, log SHA `7607f44cfbfd8bcecafdf681f70f989362649bd71940c573961359ce68b70874`, Temporal cleanup and 1 passed/42 filtered (6.10s). Strict all-target check exit0 confirmed; root retained both logs. No production change claimed. New frozen manifest, independent follow-up reviews and fresh root whole gate remain required.

### Root review8 exact transfer and fresh gate4

Both independent review8 axes PASS, no remaining actionable finding. Root verified all48 review6 preimages and review8 author hashes, copied exactly nine changed paths, then checked all48 integration hashes against review8. Fresh whole repository gate launched session71714, exact log `.scratch/team-control-plane/kernel16-root-gate4.log`; original root candidate2316e09 remains committed historical source until gate succeeds and repair commit is made. New gate is observed live; no terminal PASS yet.

### Fresh root gate4 PASS and exact repair commit

Root session71714 terminated0; exact whole repository gate ends check: all green. Retained full log SHA-256 `af5cb405906b60249323e760cdfe25940e4ccee8dffc7401258740ead4eeccd2` and Temporal cleanup record (listener closed). All48 integrated source hashes still match review8. Staged exactly nine changed source paths and committed `66e221a97465bfc4e17917a76b2db18fc657e97f`. Mandatory committed-branch review against base1f164156261a3aa927c823919bf73d6e6d11a771 launched root session19125, roborev job555. Review result, final acceptance documentation and exact-tip gated landing remain pending.

### Committed branch review555 CHANGES REQUIRED

Root session19125 terminal exit1. Exact review retained at kernel16/branch-review555.log, SHA256 `c87c558dcc1c1cc205af1a74985531522a5d60a9329c176152d5b1c1e3b36ad2`. Four findings require validation: unbound confirmed episode effects during cancellation; paused Prepared episode causing nonretryable workflow failure; untyped episode/admission UUID collisions; PostgreSQL-only artifact fixture missing Temporal evidence environment. Author and independent Spec reviewer resumed for concrete validation and public red/green repairs. Candidate66e221a remains unaccepted/unlanded; no earlier PASS overrides branch findings.

### Independent review555 validity confirmed

Independent Spec read-only assessment and root inspected source confirm all four findings. Exact bounded repair constraints retained in kernel16-review555-assessment.md, including independent stop binding, authenticated pause/resume progress, typed wait/replay semantics and PostgreSQL-only export compatibility. No new fixtures or future-source PASS claimed. Author is preparing meaningful public regressions.

### Review555 PostgreSQL-only public red

Author run review555-pg-red1 terminal101; root read actual NotPresent panic at artifacts.rs2098 after repeated failed-report behavioral assertions. Exact PG-only runner with TEMPORAL_FIXTURE_EVIDENCE unset reproduces supported-runner failure. Log SHA256 `3a873c03f7edcaf607318c50d872d361332983d154cbf86203e81fbf183cb657` independently checked and retained; author reports owned Docker wrapper cleanup terminal. Product unchanged for this red; remaining three public regressions and green repairs pending.

### Review555 unbound confirmed episode public red

Root independently checked actual cancellation regression exit101, exact log SHA `688551aeb08a05f8b26b97e07eace20dbbd8417659331269a95061fbd4e698c7`, Temporal cleanup and retained public snapshot. Unregistered adapter confirmed an actual native effect without launching allocated workspace; after public cancel/reconcile, an owned external child remained alive but disposition was cancelled. Snapshot retains unknown usage and held capacity: false completed-cancellation status is proven, not a claim of refunded capacity. Exact child is fixture-owned and RAII cleanup kills/waits only that child. Earlier red1 was compile-only and is not behavioral proof. No product repair PASS yet.

### Review555 paused episode driver public red

Authenticated current workflow driver against an Owner-paused Prepared episode returned DependencyUnmet instead of observable fenced journal progress. Root checked actual assertion failure, run.exit101, Temporal cleanup and log SHA `af431287eb8ec941adee1fee77ff918ff2de84cdc6ca55802730ff64fbc14f06`; exact log retained. Registered adapter is uninvoked and genuine prelaunch allocation cleaned before assertion. This directly proves driver refusal; actual SDK workflow durable pause/resume remains to test after repair, not inferred from this narrow red.

### Review555 typed wait replay compatibility constraint

Independent reviewer inspected accepted13 fixed-base wire behavior and identified admission-only raw UUID history compatibility. Explicit v1/v2 semantics and recorded-result transition preserve deterministic replay; constraints retained in kernel16-review555-assessment.md and sent author. No new version implementation or passing compatibility test is claimed yet.

### Correction: pause-red1 was registration setup failure

Author traced pause-red1 and red2 to singleton transport registration setup before the intended driver behavioral assertion. Root earlier classified pause-red1 too broadly from the terminal DependencyUnmet output. That classification is withdrawn: retained log proves setup failure only, not paused-driver behavior. No meaningful pause behavioral red is yet claimed. Author pause-red3 removes inappropriate direct-process registration while retaining real native adapter behind paused dispatch; actual registered SDK positive control remains pending. Evidence file preserved rather than deleted.

### Review555 pause-red3 genuine driver refusal

Root read complete fixture sequence: current WorkflowRef registration and public Owner pause succeed, driver.drive returns DependencyUnmet, native directory remains empty, genuine prelaunch allocation cancel/release and fixture cleanup complete, then intended behavioral assertion fails. Exact snapshot confirms driver Err dependency_unmet. Root verified log SHA `a29211114c20d842c8fbae978ca6864db20c639ced86f62a2708f236139274b3`, run.exit101 and Temporal cleanup; retained all three records. This proves the paused-driver behavior; actual SDK durable resume is still pending. It replaces withdrawn red1 classification.

### Review555 same-UUID dual-protocol public red

Root inspected complete public fixture: genuine admission UUID reused as caller Episode ID, both protocols Prepared and actual legacy fallback configured; every Bridge progress wait driven, but legacy journal remained Prepared. Snapshot confirms two identical raw waits; terminal intended assertion distinguishes Prepared from expected Confirmed. Log SHA `6228354c003c87c84a8ae8570ddb4887968be614f9dab969fb9e9d1543bbc535`, run.exit101 and Temporal cleanup independently checked and records retained. All four findings now have meaningful public reds; author begins narrow repairs. Actual SDK pause/resume/collision positives and old-history replay remain pending.

### Review555 first repaired boundary run — existing control failure

Root inspected provisional typed WaitRef serialization and exact registered effect-binding implementation. First repaired cancellation suite exit101: 11 passed/1 failed, 5.75 s; three new regressions pass but existing nested-writer cancellation expects cancelled and receives quarantined. Exact log SHA `add8031361fd5e49bd718f2bc23ee2ea1c9bdb1c3d97c712f2fd42a2b346923e` and cleanup verified/retained. This is not a complete boundary PASS. Author must distinguish unregistered fixture provenance from incorrectly rejected genuine cancellation; exact binding proof cannot be waived. Actual registered stopped/unknown-usage control and deterministic replay remain pending.

### Review555 PostgreSQL-only artifact suite green

Author confirms evidence environment explicitly unset. Root verified run.exit0 and exact log SHA `5d1f221232a4b13b646a86a7477c308194a222816eb1ff277b9f7975e4437499`: all11 artifact tests passed, no filtering, 3.13 s. Optional export guard preserves behavioral assertions. Exact PG-only log retained. Actual SDK combined paused/colliding/registered-unknown-usage control launched author98125, review555-sdk-green1/run.log; no terminal outcome claimed.

### Review555 SDK reconciliation fixture stale revision

Combined actual SDK control terminal101, log SHA `6b083045657ba5c9b9ee288397b5c2a5acb4ae655ba791c8e2d65940f0f19c94`, Temporal cleanup verified. Root inspected failure line6792: fresh unique Reconcile commands repeatedly use original cancel.revision after an earlier reconciliation increments task revision. StaleRevision enforces CAS; this is fixture-loop correction, not justification to weaken product concurrency. Pause/resume and collisions precede failure, but no overall SDK positive PASS is claimed. Exact failure retained for rerun diagnosis.

### Review555 registered bound cancellation positive unresolved

SDK-green2 terminal101 after10.93s; actual assertion that independently registered stopped effect completes cancellation without claiming known usage timed out. Root verified terminal/cleanup and retained exact log SHA `316e0959045dad01ddb980c427cb0f51b42e5c4a6884a4acd6e50623e197a5ef`. Unlike green1 stale-CAS harness issue, this reaches intended positive control and requires causal diagnosis of actual effect binding, launch identity and stop observations. No longer duration or weakened proof is authorized as an unsupported fix. Whole SDK control remains unproven.

### Review555 registered effect binding diagnostic evidence

Diagnostic-only actual SDK run terminal101, cleanup verified; root retained full log and before/final public snapshots. Before cancel: workspace running with persisted init identity, journal evidence only dispatch_claimed/native_turn_confirmed and no registered_native_execution_bound. Final: strict workspace Stop and immutable namespace_stopped result exit137 exist, but Attempt remains quarantined. This isolates missing independent binding as the refusal condition; timing at earlier confirmation is still a hypothesis, not proven from these later snapshots. No timeout/proof weakening or product fix is claimed.

### Review555 at-confirmation launch gap observed

Diagnostic2 terminal101, cleanup verified. Root independently decoded isolated confirmation JSONL: binding helper reaches launch_snapshot for exact operation with init=null, after registered/native receipt validation. Later diagnostic snapshots already establish persisted launch identity and strict stopped result. Thus the binding helper can run before monitor publishes launch identity, skip binding, and leave a later genuine stopped effect quarantined. Exact diagnostic log/record retained. Instrumentation is temporary, preimage must be restored before freeze. No proof weakening or timeout change claimed.

### Review555 actual SDK combined control PASS

Root independently verified SDK-green3 run.exit0, log SHA `0fbcbaed50c4506d8423fd810af61ff97caaa3df8372d9c517ef532e39d3261a`, cleanup and owned SDK waited record. Decoded actual event_type12 ActivityTaskCompleted result contains both episode_dispatch_fenced and gateway_dispatch_fenced progress. Final public snapshot proves registered effect cancellation completed, namespace_stopped immutable result, used_capacity1 and unknown-usage capacity_held. Focused test passed2.74s; exact records retained. Broader control/replay suite and final full matrix/review/integration remain pending.

### Review555 controls and retained replay PASS

Root verified run.exit0, cleanup and log SHA `f3af45e9cee138670e2616d5a21ace1465d188c892f810bebf1a08d45d482486`. Same run: library9passed (includes raw-v1 wire-shape roundtrip and same-UUID typed rollover payload); cancellation12passed; actual Temporal7passed/1 ignored childhelper, including retained history/replay/rollover. Unfiltered scopes retained. This resolves first boundary suite failure under strengthened unregistered quarantine assertions while separate actual registered unknown-usage SDK positive passes. Frozen source review, final full matrix, root integration and landing remain pending.

### Review555 strict all-target PASS

Root verified strict1.exit0 and exact Clippy log SHA `007699d45e2fa68e3e131e5a4d477abbb3e85731a0c8ef8d8c74feb99fce1dcd`, completed14.65s. Author reports all-target warnings-denied scope; exact log retained. Root source search finds no temporary binding-confirmation diagnostic seam remaining. Final full matrix and frozen reviewed manifest remain pending.

### Review555 foreign collision scope fixture setup failure

Root verified scope-green2 terminal101, cleanup, 12passed/1failed and exact log SHA `90d6e42a72974672dd805103a0f50439c351df4282ce826fa62800f2777c0460`. Full fixture source at failure shows foreign-worker create_episode allocation returns DependencyUnmet before typed routing assertions. This is setup failure, not collision behavior proof; existing12 controls pass narrowly. Allocation authorization/proof/budget checks remain intact. Author corrects foreign fixture setup before rerun.

### Review555 foreign scope and workflow guard controls PASS

Root verified scope-green3 terminal0, Temporal cleanup, 13passed/no filtering7.90s and log SHA `5a19f8953a23e86c89baaf985f7198402d0ec08c008c8eafabd9dee868edb2e2`. Foreign Task fixture now uses the same required public Owner supervised-verification activation as original fixture. Typed legitimate admission collision executes correctly; explicit foreign Episode route remains unauthorized; missing Episode references still reject closed/superseded WorkflowRef. No production guard relaxation. Final strict/full matrix/frozen review pending.

### Review555 final strict PASS and full matrix live

Root independently checked final strict2.exit0/log SHA `ae40722ab2d7fcfbc50a63fd5cf2e29385dc73316bc1a15422a4349da39c112b`, clean7.59s. Final unfiltered PG+Temporal matrix launched author59962 review555-full-gate1/run.log; root observed exact cargo PID776002 and owned Temporal PID774995 live. Existing concurrency2 and mission budgets preserved. Frozen review9 and source acceptance remain unproven pending terminal matrix and independent review.

### Review555 full matrix1 release fixture failure

Full matrix1 terminal101 before completion: cancellation suite12pass1fail; reservation-sharing fixture release1134 DependencyUnmet after waiting only for Stop. Root verified exact log SHA `05b35716dca81c41af85319537d633193fa7214af68203c1eb6c477eaf172875`, terminal and cleanup. Launched Episode release requires durable result in addition to physical Stop. Historical exact release-time result was not captured; timing cause remains a hypothesis. Author strengthens fixture to observe/retain immutable finalization and early release refusal rather than changing product guard. Full matrix PASS remains pending; prior focused scopes stay limited.

### Review555 reservation finalization focused pass and oracle race review

Focused reservation-sharing test terminal0, 1passed/12filtered1.45s; root verified cleanup and retained log SHA `7468da0ceecaf432af99c3bfdbfb0d38285d7f5d4540434d78706fc49929b711`. Root source inspection identified a remaining test-only nonatomic oracle: result absent read followed by mandatory release denial can race legitimate result publication. Author notified to permit either refusal or successful release backed by immutable result; deterministic early-denial proof remains in separate controlled stopped-monitor test. This passing run is focused evidence, not final-source full acceptance.

### Review9 frozen full author matrix verified

Root independently verified all48 frozen source hashes, manifest/baseline/review8-relative/coverage digests, full matrix log ff2aa353854a36598615d95a4918ad4bdec5ee1e84a03517471f9945ec2a8f2f, terminal0, cleanup and140passed4ignored16unfilteredgroups. Initial relative-diff filename assumption was corrected to actual review9-relative-to-review8.diff; no source mutated. Exact records retained. Independent Standards/Spec reviews active; root source remains committed66e221a until reviewed transfer.

### Review9 exact root transfer and gate5 live

Both independent review9 axes PASS. Root verified all48review8preimages and frozenreview9hashes, transferred exactlyeightchangedpaths, rechecked all48sourcehashes and launched fresh root whole gate5 session4948. Exact pending source paths are uncommitted; source commit66e221a remains historical candidate until gatepasses/newcommit. Audit reconciled to current review9 scope; no acceptance or landing claimed.

### Fresh root gate5 PASS and exact second repair commit

Root session4948 terminal0; whole repository gate check: all green, Temporal cleanup verified. Retained exact log SHA `dbacf7b8fb7427067f17bc0d62f007addfa4b48307313e791cace3af55f7b87d`. All48sourcehashes still matchreview9. Staged and committed exactlyeightrepairpaths as `58c96816ecfd8224d4e2b6f8f01e59eda1eff249`. Mandatory committed branch review launched root95418 againstfixedbase1f164156; final acceptance/docs commit and exact-tip landing await review result.

## Current candidate records reconciled while branch560 is live

Updated STATE and completion audit to committed candidate58c9681 and fresh root gate5 PASS. Refreshed the retained evidence manifest over 185 files (6943415 bytes), excluding the manifest itself. This is a retention inventory, not task acceptance. Root independently polled session95418 and roborev status: job560 remained running and daemon healthy. No execution was restarted; ticket16 remains unaccepted and unlanded.

## Review560 terminal findings

2026-10-02T19:15:59.092053+00:00 — roborev status reports560 done6m14s; show560 retained. Root wrapper95418 exit1. Independent committed-candidate Spec assessment confirms all3 findings. Acceptance withheld; read-only architectural repair assessment delegated before implementation. STATE and completion audit updated; source remains58c9681.

## Architectural assessment complete; isolated repair author established

2026-10-02T19:18:13.295768+00:00 — Independent author confirms all3 findings and proposes no new stack or mandatory schema. Root recorded explicit recovery/dispatch/unused-capacity transition decision. Sole-writer newtree `/tmp/corenius-kernel16-review560-author` HEAD58c96816ecfd8224d4e2b6f8f01e59eda1eff249 and clean source verified read-only. Author preparing public reds; exclusive cargo/fixture build slot assigned. Root performed only later-ticket source/prerequisite reads, no provider requests or competing services. Later restore remains incomplete: source epoch rotation is not dispatch-disabled rollback restore or orphan accounting.

## Review560 unbound reservation genuine public red retained

Author37530 terminal101 confirmed. Root inspected exact test: Fixture.finish executes before intended used_capacity assertion; retained run.exit101, full log, cleanup, public before/after/reconciliation JSON and PG final cleanup. Failure shows used_capacity1 expected0 after initial and repeated reconciliation. Cumulative counter assertions are after failing assertion, so this red does not prove them; they remain required green controls. Initial fixture does not prove capacity-one replacement. Raw Temporal SQLite and provider/private configuration excluded.

## Review560 cancellation driver red retained

Root inspected terminal exit101/Temporal listener closed and public JSON progress Errstale_generation. Exact log and snapshot retained with digests under branch-repair/review560-cancel-driver-red. Fixture final assertion executes after cleanup. This is genuine direct-driver regression, not actual SDK Activity proof. Source remains author-local test changes; root committed candidate unchanged and unaccepted.

## Cancellation-red exact terminal handle confirmed

Author session93930 terminal101 confirmed. Current/closed/superseded Workflow controls execute before intended direct-driver final assertion. Root clarified public physical-launch test must retain explicit worker journal uncertainty and before/after reservations; launch-result evidence alone cannot prove budget preservation. Final monitor barrier and actual SDK history remain required, with no competing root build.

## Third review560 public red: physical verifier launched

Author47605 terminal101; root observed finalized log/exit/Temporalcleanup and actual verifier result. Actual native Worker unknown-usage public outcome was established in test setup; initial retained claim lacks explicit journal/reservation snapshot, so strengthened evidence remains required. Original red retained without overwriting. Author applied minimal patches after respective reds and reports focused cancellation suite running88036; this is not acceptance. Root performs no competing build or source integration.

## Initial review560 focused cancellation green observed

Root read finalized focused-green/run.exit0 and unfiltered15passed/0failed/0ignored in6.57s. This includes new refund/current-workflow controls and existing cancellation regressions. Retained log SHA256 c6b706a9a8b351e8fd81ee804b5f15580fcceeff04b591ea87b9e7ea43da76a4. Source is mutable author-local and this is an intermediate green, not frozen final acceptance; actual SDK/monitor usage race, full matrix, own legacy launch, budget negatives and independent reviews remain required.

## Physical launch second-boundary red retained

Root inspected actual MonitorGate ordering in author fixture and finalized monitor-red exit101/Temporalcleanuptrue. Monitor claims, genuine sibling native effect develops unknown usage, monitor resumes and verifier result exits0. Intended refusal oracle fails after fixture cleanup. Selected log/result snapshot retained with hashes under branch-repair/review560-monitor-red. Exact author handle/preimage confirmation pending; usage/reservation fields must still be strengthened before final green acceptance. No production code transferred.

## Episode launch-fence architecture clarified

Independent Spec assessment confirms preserving accepted legacy predicate and adding episode uncertainty checks for all actual Attempt-bound executions suffices for560. Own execution mapping is not effect-origin proof: own unresolved/confirmedUnknown cannot receive blanket exemption. Recorded explicit repair direction and required positive/negative controls; author instructed to report unresolved dispatching-origin ambiguity. Monitor-red exact author73200 terminal101 and restored physical predicate preimage confirmed. No root source mutation or candidate acceptance.

## Own-operation timing ambiguity escalated before patch

Actual start/monitor asynchronous ordering invalidates a blanket dispatching-only exemption. Root and author stopped that patch direction and requested independent Ready admission/original continuation provenance assessment. Actual SDK public-red author72666 currently live per author handle report; authoritative outcome pending. No weakened exemption, schema change, provider traffic or root source integration performed.

## Actual SDK cancellation red and original admission provenance direction

Author72666 terminal101,20.68s; root independently inspected retained history/eventcontainer and terminal JSON: completed_fenced_activityfalse,no_effecttrue,SDKSIGKILLwaitedtrue; Temporalcleanuptrue. Retained selected exactpublichistory/log/terminal with hashes, noSQLite. Actual SDKgreen18522 reportedlive, no restart or competingfixture. Independent architecture final confirmed atomically recorded registered-origin launchadmission certificate in existingepisodeevidence table; explicit decision written and author instructed to implement proof-bound originalmonitorcontinuation, never blanketownID or usage refund.

## Actual SDK cancellation green independently decoded

Author18522 terminal0,1passed/46filtered1.37s. Root decoded only actual event12 ActivityTaskCompleted result payloads and verified both native episode_dispatch_fenced and legacy gateway_dispatch_fenced in one completed progress. Exact history/log/terminal/cleanup retained with hashes and decoded payload record; SDKwaited/no-effect and Temporalclosed observed. This does not prove failed-verification gate branch, physical usage fences, finalsource matrix or acceptance; author source mutable and physical predicate still baseline pending provenance implementation.

## Ready own unknown effect regression observed and retained

Root inspected new public fixture establishing genuine unregistered external effect and live owned child, confirmedUnknown journal, then Ready workspace physicalstartOk plus actualresult. Final expected-refusal assertion follows drop(runtime)/Fixture.finish; root retained exactsnapshot/log/exit101/Temporalcleanuptrue with hashes. Exact author terminal handle and child kill/wait confirmation pending. This directly proves ownexecutionID blanketexemption unsafe and motivates recorded original launchadmission provenance; no root source mutation.

## Durable tracker sanity during launch-provenance repair

2026-10-02T19:33:52.474519+00:00 — Root ran `python3 tools/agents/tracker.py lint team-control-plane`, terminal exit0 with no diagnostics. Ticket16 remains claimed and acceptance boxes unchecked; dependent later milestones remain unresolved. Source inspection confirms public Start persists Starting before spawning monitor, reinforcing original-launch certificate transaction requirement. No competing builds/services or authority/tracker resolution mutations.

## Launch-admission implementation present; intermediate source review

Author mutable supervisor now distinguishes public first argv from monitor persisted argv; checks own Dispatching plus actual registeredprocess at Ready admission, inserts immutable original_launch_admitted evidence, and requires exact certificate for Starting continuation. Root read-only inspection confirms insert and Starting occur in same transaction. Root requested registered/pinned workspace/native directory correspondence in addition to process/runtime/store binding before final proof. No frozen-source/fullmatrix result or acceptance inferred from code inspection. Root source remains58c9681.

## Certificate fixture registration setup refused; not behavioral green

Author83464 terminal101; root log confirms both usage cases refuse register_runtime before dispatch because libtest process is not exclusive process-group leader. This is fixture setup failure, not a production fix verdict. Author adapting existing owned subprocess/process_group(0) harness rather than weakening origin proof. Failed setup log retained separately (SHA256 320c6f5344f12d0c51a33e4f901bc0b073fc48df4562dfdc80a636453be3012f). Ready ownUnknown author1878 terminal101 and exact child kill/wait before cleanup confirmed; retention record updated. Pinned roots/current directory checks remain required.

## Both actual physical-launch boundaries green

Root independently inspected usage-green2 exit0,2passed46filtered1.01s,Temporalcleanuptrue; both public claim snapshots show launchErrdependency_unmet and verifierresultnull. Owned child terminal records retained along with exactlogs/snapshots and hashes. This is focused intermediate author-source evidence; explicit before/after reservations/usage provenance, ownreply-beforemonitor controlled positive, corruption/foreignorigin negatives, fullmatrix and independent review still required. No rootsource integrated.

## Focused launch green terminal handle confirmed; budget fixture seam reviewed

Author12910 terminal0 and owned child waits confirmed; retained record updated. Root read-only reviewed with_limits fixture support: preserves ordinary2/8/8 defaults while allowing capacity-one/cumulative budget regressions; no product policy ceiling changed. STATE reconciled to actual SDK and both physical launch greens without acceptance. Remaining controls/fullmatrix/reviews explicit. Root did not start competing builds.

## Capacity controls pass inside non-green cancellation suite

Root observed cancellation-green2 full17cases:16passed1failed6.81s. New capacity-one replacement/cumulative BudgetExhausted and ReadyownUnknown controls passed; selected exact publicsnapshots/log/cleanup retained. Existing nestedwriter/cancel positive expectedConfirmed but actualFailedKnown atline476; complete suite is NOTgreen. Root flagged author to diagnose genuine registration-origin requirement, preserve boundary and report exact cause. Intermediate source not accepted or integrated.

## Registered-origin nested cancellation corrected and passing

Root observes cancellation-green3 run.exit0,17passed1owned-childignored0filtered7.15s. Author67200 givenhandle. Source changed fixture to real registered exclusiveprocessgroup while retaining physicalmembers/nestedwritersStop and usedcapacity1 assertions. Registered bound confirmedUnknown may cancel while holdingcapacity; OutcomeUnknown remainsquarantined. Unregisterednegative preserved. Selected three-mode evidence, budget/Readynegative snapshots/log/cleanup retained with hashes. Parentchildterminalrecord requested; no finalsource acceptance, fullmatrix or rootintegration.

## Deterministic original-launch proof drafted; first run compile-only

Author4135 provenance-green started. Root currentlog sees missing UsageObservation type qualification atthree new assertions, so this run is compile failure with no behavior verdict. Retained log separately (SHA256 6a884c2dfceb1e0edcd0010054c4a0c7c81a481bed1cec8cab9c30706de67e35). Read test draft: actual registered Readyadmission under ownDispatching then BeforeMonitor hold, Unknownreply before actualmonitor; expects exactlyone real output and heldcapacity; wrongargv/foreignCLIorigin/storedargv negatives retain zeroresult. Root did not infer proof from source. Cancellationgreen3 session67200terminal0 confirmed and record updated.

## Original registered launch and provenance negatives pass

Root observes provenance-green2 exit0,2passed48filtered1.63s. Independently checked four public launch-proof snapshots: original has1certificate and actual exit0 despite ownUnknown; wrongargv/foreignorigin/storedargv have noresult; all heldcapacity1,usageUnknown,releaseDependencyUnmet. Source positive asserts actual outputonce, parent children waited. Exact logs/snapshots/terminal/cleanup retained with hashes. These are focusedmutable-source greens, not wholekernel acceptance. SDKfailedgate/strengthenedusage/fullmatrix/independentreviews remain.

## Provenance green exact terminal confirmed; failed-gate proof oracle bounded

Author7830 terminal0 confirmed; retainedprovenance record updated. Author extending existing actualSDK failedgate fixture, preserving Prepared/noeffect and requiring a newly completed event12 after actual failed-report history boundary. Root explicitly rejects old paused-result reuse and requested retained taskrevision/report/newActivitypayload plus subsequent recovery. This is verification scope clarification, not an executed failedgate proof or new acceptance. No duplicate fixture launch/root builds.

## Actual SDK failed-gate revocation red independently decoded

Root observes sdk-failedgate-red exit101/1failed49filtered26.43s/Temporalcleanuptrue, author27527 handle. Postfailedreport boundary maxevent11 includes historicalcompleted7; later finalhistoryevent18 is actualActivityTaskFailed brokerStaleGeneration nonRetryabletrue, event22 Workflowfailed. This proves more than absence/timeout. Source ownedSDKkill and fixturefinish precede finaloracle. Selected exact histories/boundary/preallocatedrefusal/log retained with hashes; rawSQLite excluded. No final acceptance/root integration.

## Failed-gate SDK focused proof passes within live non-green breadth

Root vertical-green8147 partiallog marks failedgate fixtureok. Independently decoded boundarymaxevent11 and latercompletedActivity18: nativejournalPrepared with episode_dispatch_fenced. Exactpublicboundary/history+decodedrecord retained. Broader suite still live and already has3existingcasefailures; no whole-suitePASS or finalsourceacceptance claimed. Root instructed author not torestartlivehandle, diagnose terminalfailures separately, preserve requiredpublicbehavior.

## Assembled breadth terminal: three old physical positives require diagnosis

Root vertical-green run.exit101/fullsummary44passed3failed3ignored0filtered85.92s; author8147 handle. Exact failures: expectedactualnativeeffect1 observed0, lostreply expectedOutcomeUnknown gotFailedKnown, sharedstoreepisodefirst expectedConfirmed gotFailedKnown. Actual SDK cancellation/failedgate, registeredprovenance and strengthenedusage tests pass. Fullfailedlog/cleanup/finalusagepublicsnapshots retained with hashes. Root requires diagnosis preserving actualeffects/lostreply/storeexclusion assertions, no downgrading oracle or weakening registration. No accepted/frozenfinalsource.

## Two prior physical positives and authority controls pass focused run

Root observes compatibility-green run.exit0,3passed47filtered3.67s. Actual selected tests: lostreply physicalcapacity, legacy/episode storeexclusion, physicalmonitor scope/generation/epoch. Third priorfailing confirmednativeunknownusage case NOTselected; root explicitly requested coverage and did not infer repair. Exact logs/ownedchildterminal/refusalpublicsnapshots/cleanup retained with hashes. No finalfreeze/sourceintegration/acceptance.

## Compatibility terminal confirmed; fixture consolidation awaits final gate

Author28842 terminal0 confirmed exactselected3cases. Root readonly observes duplicatephysical/monitor fixture cases consolidated (five changedpaths,978added26removed atcurrent read); production bytes remain priorpatch. Updated proofmap to actualfailedgate newActivity and independentlychecked usage/reservation snapshots. No finalsourcefreeze/matrix success inferred. Third priorpositive still needs finalmatrix; all48rootcommittedsource remains58c9681.

### Review560 intermediate full matrix and Lead scope clarification

Author handle50593 terminated with exit0. Root independently matched log SHA256 `81bb54642b034995db543b21be37f33947df3a17663cc393b637669cccd86c7e`, parsed all16 result groups and counted149passed/0failed/6ignored/0filtered. Selected public log, terminal exit and cleanup record retained in `kernel16/branch-repair/review560-full-matrix1/`. This is intermediate evidence, not acceptance.

Source inspection found `physical_usage_known` returning early for task-free episodes. Directed the isolated author to apply own-operation provenance to all roles while grouping siblings only by an available Attempt, and to prove task-free negative and registered original-launch controls before a fresh full gate. Durable clarification: `../decisions/20261002-original-launch-provenance-all-roles.md`. No source integration or landing performed.

### Review571 focused recovery proof

Replacement author public fixtures prove unallocated Worker continuation and
Verifier capacity release once, capacity reuse without resetting cumulative
admissions, native Cancel before and after work revocation, and completed
verification acceptance/replay after producer expiry while expired current
calls remain refused. Setup compilation failures and a zero-selected-test run
are not passing behavioral evidence.

Cross-version proof used the exact archived `da7ff9b` crate: root compared all
48 tracked files to Git. Old executable SHA256 is
`d31c71231eac53d932cc7695faab38ae3f7f636021a80e0c8894286d7e1a4720`.
Its public Yield persisted the old outstanding flag after refund; repaired
cancellation reconciled the same database twice without a second refund.
This external-binary test passes separately and remains explicitly ignored in
the ordinary matrix. It proves this compatibility case, not general restore.

The uncertain allocated sibling control (author session35749) terminated0,
one selected pass: quarantine retains both capacity claims. Public evidence is
retained under `kernel16/branch-repair/review571-uncertain/`.
Focused final session87013 terminated0 with four passes and one explicit ignored
external-binary case; native controls session38307 terminated0 with one pass.
Full real PostgreSQL/Temporal matrix session99634 is pending terminal outcome;
independent review, root gate, branch review, acceptance and landing remain open.


### Review575 bounded interface repair: focused byte and memory evidence

The previous review571 pending outcomes above are historical: source candidate
`c4599e8be5720ff7da74ce886069ff0da0560b80` passed root gate7, but independent
branch review575 rejected it with four interface findings. The current isolated
repair implements the decision `20261002-kernel-interface-repair575.md`.

Raw-byte verifier output: author session68954 (`red3e`) terminated101 after a
real successful verifier emitted byte255, its monitor recorded trusted exit0,
and cleanup finished before Verify rejected the encoded-byte representation.
Session65336 (`green3`) terminated0: one selected public fixture passed valid
Verify/Accept and the refusal controls. Public logs, proof and cleanup records
are retained under `kernel16/branch-repair/review575-binary-output-{red,green}/`.
Earlier binary fixture compilation/stack failures are setup failures, not red
behavioral evidence.

Bounded monitor finalization: author session10228 (`red4`) terminated101 at the
intended trusted-result assertion. A calibrated1MiB log finalized under the
owned monitor's measured address-space constraint; a134217728-byte log was
complete on disk and independently hashed, but the old monitor left no trusted
result, refused release and retained capacity1. Both cases recorded monitor
termination, empty namespace membership and fixture removal before the oracle.
Selected logs, measured limits, results and cleanup records are retained under
`kernel16/branch-repair/review575-monitor-memory-red/`. The streaming fix, its
green evidence, full matrix and acceptance remain pending. These focused proofs
do not establish live providers, backup/restore, landing or the full goal.


Monitor streaming green session37056 terminated0: one selected public test
passed the1MiB calibration and128MiB log under measured64MiB monitor headroom,
with trusted raw digest, durable result and capacity release. Root independently
read its terminal exit and retained nine hashed public artifacts under
`kernel16/branch-repair/review575-monitor-memory-green/`.

Focused operator session50554 terminated0 with all seven tests passing, including
exact omitted-Mission pause/resume, running and quarantined legacy status,
foreign-scope controls and real unreadable-log refusal. Root independently read
the terminal log and retained22 hashed public artifacts under
`kernel16/branch-repair/review575-operator-final/`. Full matrix, strict checks,
frozen two-axis review, root gate and independent branch review remain required.


Review575 full author matrix session78915 terminated0:162 passed,0 failed,
7 explicit helper/compatibility ignores across16 groups. Strict session34361,
format and diff checks terminated0. Root verified immutable manifest
`6ad5e6e759050e7e2ca2d7150c2141b0d55336d2411d44db1ce0b1aac59ac51b`,
all48 sources and seven preimages against `c4599e8`. Independent clean-context
Standards and Spec reviews both PASS; their retained verdicts explicitly limit
scope to the bounded repair. Root integrated seven files only, verified all48
postimages, and launched full repository gate8 on session13946. Terminal gate
outcome, committed branch review, acceptance and landing remain pending.


Root gate8 session13946 terminated0 with `check: all green`. Retained full log,
exit, classification and18 selected cleanup JSON records under
`kernel16/branch-repair/root-gate8/`. Exact reviewed repair committed as
`dd8e14cff65c00e7936c2d0ce9471aed973c777f`; root verified all48 Git blobs.
Whole-branch review580 covers `1f16415..dd8e14c` (six commits) and is running
on session36530. Full-ticket independent audit found no required behavioral gap;
its corrected evidence assessment directly verifies all retained focused hashes
and the actual162/0/7 matrix log. Acceptance and landing remain pending.


While review580 remains live, root refreshed and verified the public retained
inventory:626 files,9671700 bytes, candidate dd8e14c, pendingreview580. No SQLite,
DB, PEM or key artifact was included. `roborev status` confirms job580 running
with healthy database/workers; session36530 remains live. A `show580` call found
no completed review, which is consistent with pending state and is not a reason
to restart or enqueue another review.


Whole-branch review580 session36530 terminated1, CHANGES REQUIRED: three Medium
findings on original unbound reservation refund after allocated continuation,
monitor-crash/result-persistence recovery finalization, and retained large logs
blocking takeover. Retained terminal review under `kernel16/branch-repair/branch580/`.
Prior focused Standards/Spec PASS and full gates do not override this rejection.
Read-only independent validity and architecture assessments requested before
further patching; candidate dd8e14c remains unaccepted and unlanded.


Both independent read-only assessments confirmed all three review580 gaps.
Root recorded bounded recovery decision and additive proof clarifications,
including original/finalization epoch separation, complete exact-namespace stop
coverage, descriptor-bound stable bytes and immutable CAS-winner preservation.
This is iteration2 of the existing three-iteration round, not a budget reset.
Fresh sole-writer author580 initialized exact dd8e14c worktree and owns serialized
Cargo/PG/Temporal fixtures. No red or product fix yet; first slice is original
unbound capacity recovery across independently charged continuation cancellation.


Recovery580 original-claim slice: author red session46335 terminated101 at the
intended post-cleanup oracle (used_capacity1 vs required0). Focused green
session35284 terminated0, one pass, after narrow original-binding and known-usage
checks: before2/after0, repeat reconciliation, replacement admission and consumed
cumulative budgets. Root directly read terminal logs/exits and retained both
public proof/cleanup sets under review580-capacity-{red,green}. Remaining same-
sequence uncertainty controls, monitor-crash recovery, large retained-log takeover,
full checks/review and acceptance remain pending. No root source integration yet.


Recovery580 monitor crash red2 terminated101 at intended post-cleanup release
oracle. The fixture blocks actual resultINSERT with an owned PostgreSQL trigger,
observes committed stopped state, kills the exact monitor (PID/start/boot bound),
then unlocks/removes its trigger. Missing result is confirmed; replacement Owner
reconcile/release still refuses. Root retained full log, exit, real crash record
and cleanup under review580-monitor-crash-red. Actual author handle and final
recovered provenance/stopcertificate/digest, legitimate Verifier refusal controls,
new recovery guard/CAS coverage remain pending. No passing recovery claimed.


Recovery580 monitor slice focused green session60011 terminated0, one selected
pass. Root retained full log/exit/crash/cleanup and inspected the actual result:
recovered_stop provenance records finalizer, current epoch, exact original PID1
identity and stop observation UUID; original monitor/epoch remain separate,
exit/signal null, release succeeds, capacity0 and repeated result identical.
This is physical recovery evidence, not successful verifier evidence. Legitimate
Verifier refusal and authority/uncertainty/CAS/compatibility controls remain
pending. Large retained-log takeover regression is now written against actual
2097153-byte workload output; its terminal red and product fix remain pending.


Fast crash recovery session25683 terminated101 at the post-cleanup release
oracle. Actual immediate workload and monitor precommit crash have no gateway
operation/usage ambiguity. Root retained full log/exit/crash/cleanup under
review580-fast-crash-red. Initial launch persisted init/members/running without
an append-only running observation; the strict recovery chain therefore lacked
a predecessor for fast completion. Authorized correction records exact PID1
running observation in the same gated launch transaction; it does not weaken
complete stop proof. Green and legitimate Verifier refusal remain pending.
A same-sequence unknown native usage/refund control is now added by the author.

### Review580 focused recovery and thread-group boundary

Focused session `98900` exited `0`: independent capacity refund, unknown-usage hold and receipt resolution, delayed and fast monitor crash recovery, exact-candidate recovered-Verifier Verify/Accept refusal, and actual large-log takeover passed. Public JSON proof and cleanup records plus log/exit are retained under `kernel16/branch-repair/review580-focused-green/`, with SHA256 classification. This is focused evidence, not full ticket acceptance.

Session `77895` exited `101` with a meaningful whole-thread-group regression: an owned zombie leader with a surviving thread was incorrectly considered terminated during recovery. Cleanup preceded the result-none assertion. The author reused the existing identity-fenced whole-group termination seam for monitor and launcher, then started final focused session `96409`; its outcome remains unverified. Red evidence is retained under `kernel16/branch-repair/review580-thread-group-red/`. The repair remains iteration 2 of the existing three-iteration round.

Final focused session `96409` exited `101`: cancellation two and operator three tests passed, including the new surviving-thread recovery refusal. The vertical binary aborted with stack overflow before completion. This is not a green suite. Log/exit and classification are retained in `kernel16/branch-repair/review580-final-focused-abort/`; exact orphan cleanup and fixture-stack correction were requested before another run.

Final boxed focused session `42143` exited `0`: cancellation2, operator3 and vertical4 parent tests passed, plus the owned child run. This includes missing-provenance historical monitor Verify/Accept compatibility, exact surviving-thread refusal, and seven takeover security cases. Source formatting preceded full real PostgreSQL/Temporal matrix session `20645`, whose outcome is pending. Public JSON/log/exit SHA256 records are retained in `kernel16/branch-repair/review580-final-focused-green/`. No ticket acceptance or landing is claimed.

Full matrix `20645` reached assembled_vertical but its process aborted with stack overflow in historical `review571_completed_verifier_provenance_survives_producer_expiry`. Earlier group passes remain partial evidence; no full matrix pass. The root requested shared fixture future-size diagnosis and a consistent heap boundary, preserving all scenarios, plus exact orphan cleanup before another runtime run. Current log is retained in `kernel16/branch-repair/review580-matrix-abort/`; wrapper terminal outcome and cleanup remain to verify.

Matrix20645 abort cleanup retained: author identity-bound SDK77666 ticks39854847 was stopped, roots preserved and references empty; root independently confirmed PID absent. A subsequent edit failed because its cwd-relative path was wrong, yet the shell launched unchanged-source session62239. This is invalid validation; root requested stop/collection, descendant cleanup and fail-closed edit/verification/launch commands before the corrected shared-fixture run. No green outcome or source freeze is claimed.

Accidental unchanged-source session `62239` produced `matrix-heap.log`/exit `101`; the misleading filename does not make it corrected-source validation. The owned operator process113383 was sent SIGTERM and the runner reports that signal. Stop record and log/exit retained in `kernel16/branch-repair/review580-mislaunch/`; complete descendant/reference cleanup remains to confirm. The author source now contains a single thin `Pin<Box<dyn Future<Output=()>>>` fixture entry delegating to the shared body, statically consistent with the intended all-scenario heap boundary. Fresh runtime proof of that source is pending.

Mislaunch cleanup complete: Cargo94856 and test113383 absent on independent root check; author zero-program/root-reference record retained. Shared heap entry is applied and formatting passed. Genuine corrected-source full matrix is distinct session `25437`, using `matrix-corrected-heap.log`; outcome remains unverified. Edit/cleanup/format/confirmation now fail closed, with terminal-code capture only around validation.

Corrected matrix25437 directly inspected: operator10 passed; vertical54 passed/0failed/3ignored, including historical two-scenario producer expiry and all four new580 vertical cases. The shared heap boundary therefore passes the previously aborting path in full-suite context. Later groups are still running; no terminal full matrix claim.

Corrected full matrix25437 is terminal0: 16 summary lines total172 passed/0failed/8ignored including owned child runs. Final Temporal7 passed/1ignored. Public log/exit/cleanup/PG-final SHA-indexed evidence retained in `kernel16/branch-repair/review580-matrix-green/`. Strict locked all-targets clippy with warnings denied, fmt/diff and freeze sequence39389 is pending. Mislaunch cleanup path filtering was explicitly amended to remove ledger JSON strings mistaken for absolute roots; root retained prior and amended records plus explanation, with process/exit facts unchanged.

Strict/fmt/diff/freeze39389 terminal0. Root independently verified manifest3cea8def... and patchf83dbd29..., all48 current/frozen source hashes and seven exact dd8 Git preimages. Manifest/patch retained under `review580-freeze`, validation records under `review580-validation`. Independent clean-context Standards and Spec reviews dispatched against this frozen candidate; no integration/acceptance/landing yet.

Independent Standards review580_standards returned PASS/no blocking findings for the seven-file bounded repair. Reviewer directly checked44 public-proof SHA256 values and actual behavioral/cleanup records, excluding invalid runs. Spec reviewer independently verified48source/7Gitbase and all selected behavioral proof, final verdict pending. Root gate9 fail-closed wrapper prepared and syntax checked; not executed or integrated yet.

Independent Spec returned PASS/no blocking findings; its scope is bounded repair, with some guard coverage from existing matrix/source, not fullticket16. Both review verdicts retained under review580-freeze. Root verified48 Git preimages, applied exact reviewed patch and verified48 frozen postimages. Full repository gate9 launched as session3092 and polled live; no source commit or landing yet.

Root full repository gate9 session3092 terminal0;14 cleanup JSON records retained. Exact seven-file source committed as `480d7c22bdfd926e1534c4680988d63d50618d2e`, with all48 committed Git blobs verified against frozen manifest. Fresh whole-branch roborev job584 covers seven commits `1f16415..480d7c2`, session94656 polled live. No landing or ticket16 acceptance yet.

Independent full-ticket acceptance580 audit found no concrete kernel behavioral blocker after checking48source/Git blobs,44proof hashes, actual matrix172/0/8, rootgate9 exit0 and mapping all18 public failure-matrix rows (optional Kata/jj/restore rows outside16). Remaining: branch584 verdict, final packaging/current handoff, formal acceptance and gated landing. Audit limits preserved in kernel16-acceptance-audit-review580.json; no fullgoal/liveTrae/restore acceptance.

Wholebranch584 session94656 terminal1 CHANGES REQUIRED: epoch-rotated stopped crashed-monitor cleanup strands result/reservation; canonical cancellation fences current Worker episode inspection/cleanup. Original log/exit retained branch584. Independent author/Spec read-only validity/contract assessments requested before final iteration3/3. Tests/conformance/kernel audit do not supersede rejection; no acceptance or landing.

Both independent author and Spec static assessments confirm584 lifecycle findings. Final iteration3/3 assigned to fresh clean-context kernel16_review584_author in /tmp/corenius-kernel16-review584-author, branchcodex/kernel16-review584-author at exact480d7c2. Sole writer owns Cargo/PG/Temporal; no schema/provider or authority expansion. Public red/green, fullmatrix/freeze/reviews/gates required; no budget reset or acceptance.

Rotationred19570 terminal101; greenrotation exits0 established by fail-closed sequence reaching workerred76532. Workerred terminal101 with genuine currentWorker inspect StaleGeneration after canonicalCancel and physical fixture cleanup, ownedchild waited101. PublicJSON/stderr/log retained review584-red-worker. Correction: earlier author cleanup.json claim was wrong; actual rotationcleanup is f.finish/rootabsence before oracle plus run-postgres trap. No invented file retained; explicit ownedcontainer observation requested, no metadata-only rerun.

Focused68791 terminal101 setupmissingTEMPORAL_ADDRESS at existing singletonfixture;26individualcancellationpasses including currentWorkerknown/unknowncleanup, but no suitegreen. No productfix/skips for environment. RealTemporal96963 terminal0 proves cancelledcached/newreport/accept refusal, listenercleanup retained. Full realPG/Temporal alltargets26814 active; log/evidence retained in review584-green-focused and review584-green-runtime.

Finaliteration matrix26814 cancellation27pass/0fail/3ignored, operator progressing. Author read-only Docker inventory finds sole currentmatrix PGfixturea0c027a761cf; prior terminated auto-remove fixtures absent. Observation retained review584-cleanup-observation, no kill/rerun. Fail-closed rootgate10 prepared/syntaxchecked only; waits immutable freeze and conformance.

Final repair matrix26814 terminal0 directly verified:16summaries173passed/0failed/8ignored including nestedchild. Temporal listenerclosed and zero remaining PGfixtures records retained review584-matrix-green. Strict/fmt/diff79620 reportedterminal0; author freeze/preimage/patch verification index pending. No acceptance/landing.

Final584freeze independently verified48source and4Git480preimages; manifestf33924b2... patchf615f7ab... retained with terminal/strict index. Authorreleasedruntime slot. Standards freshreview active; freshSpecs pawn rejected by threadlimit, so existing independent acceptance580 reviewer handles Spec followup with exactfrozencontract. Broad878-entry author index includesruntimeDB/service logs and is not treated as publicproof package; root retains selected records only.

Finalrepair584 Standards+Spec bothPASS after all authorityconsumer/lifecycle tracing and physicalproof inspection. Exactfourfilepatch integrated root480base, all48preimages/postimages verified; source remains uncommitted pending rootfullgate10 session32923 launchedlive. No mainline movement or acceptance.

Root finalrepair package retains37selectedpublicproof/terminal/cleanuprecords with repo-relative SHA index (review584-portable-proofs); databases/service logs excluded. Gate10session32923 polledlive into vertical cases; sourceunchanged since exactreviewedintegration. Finalwholebranchwrapper ready but unexecuted pendinggate+exactcommit.

Rootfullgate10session32923 terminal0,17cleanuprecords retained. Exactfourfilefinalrepair committed9055b9976d429229e5d42b2e86eb04dbc30da5bf; all48Gitblobs matchfreeze. Finaliteration wholebranchreview launchedsession99070, verdictpending. No acceptance or mainlinelanding; further nonconvergence requires escalation.

Wholebranch591session99070terminal1CHANGESREQUIRED: kernelstatus summaries omitrolefilter for sameprincipal crossrole grants. Rejection retainedbranch591. Repairbudget3/3exhausted; authorread-only assessment assigned. No patch/test/newreviewround or landing; concreteextension scope followsassessment.

Confirmed591 role-isolation finding: sameprincipal grants permitted, summarySQL omitsrole while exactinspect requiresit. Concreteone-round queryfilter/publicsameprincipal+paginationproof proposal recorded20261003-kernel-role-isolation591-proposal.md. Explicitbudgetextension request pending; automaticgoalcontinuation is not an approval. No patch/runtime/landing launched.

User approved requestedone-round extension. Concreteapprovedcontract recorded, fresh cleancontextauthorassigned kernel16_review591_author at3e2622e. Publicsameprincipalrole/paginationredgreen andfullgates required; no implicitunlimitedbudget or landing.


### Repair591 frozen candidate verified

Root verified all 48 current source files, all 130 frozen files, the exact patch and two Git-base preimages. Retained 76 selected public proof files plus four metadata/patch records. The full matrix records 174 parent-suite passes, zero failures and eight owned subprocess entrypoints ignored; its separately nested child records one pass. Independent Standards and Spec reviews are pending. No source integration or landing has occurred. The owner’s full-goal authorization supersedes routine repair permission stops; see `decisions/20261003-owner-authorizes-full-goal-execution.md`.


Repair591 conformance: Standards PASS (one nonblocking fixture duplication heuristic), Spec PASS (76 public hashes independently checked). Exact frozen patch integrated into root; all48 source hashes match. Root gate11 launched as session13545 and observed live in the required real PostgreSQL/Temporal gate. Source commit, branch review and landing remain pending.


Root gate11 session13545 terminal exit0; all repository checks green. Retained19 cleanupJSON records and exact raw run/exit. All48 source hashes reverified before sourcecommit e6676bf6c6d05ab850ddde17aa71f6932be495f5 and again against committed Git blobs. Whole-branch review session86801 launched against main; pending. Main remains unchanged; ticket16 acceptance and landing pending.


### Whole-branch611 rejection and authorized repair

Review611 session86801 terminal exit1: High verification execution provenance omits immutable suite/environment policy digests; Medium deadline-stopped released lead episode remains live and prevents successor admission. Earlier conformance/gate passes do not override this rejection. Fresh isolated repair assigned to kernel16_review591_author in a new review611 worktree at587aca8 under full-goal owner authorization; source integration and landing withheld. Review and repair contract retained in decisions/20261003-kernel-policy-lifecycle611-repair.md and evidence/kernel16/branch-repair/branch611/.


Repair611 author confirmed fresh solewriter tree at587aca8821d352b73463dff2ffd02cbdf55e19e5. Ranked policy and lifecycle hypotheses recorded in kernel16-review611-assessment.json. Public regressions under construction; no new red/green evidence or services reported yet. Root requested separate suite/environment changes, fresh-policy positive, cached report/acceptance revalidation and preservation of historical cleanup/checkpoints/waits.


Repair611 first focused green10605 terminal0: five public tests passed, including separate suite/environment changes with a fresh actual verification execution and acceptance, deadline successor admission, yielded wait preservation and unknown-native-usage refusal. Author consumer trace found cached Verify bypasses execution-policy revalidation; cached Accept already rechecks. Additional same-scope test-first cachedVerify slice is required before final-source fullmatrix/freeze. Earlier focused green is not proof of this subsequent source.


Repair611 finalfocused83199 terminal0:6/6 public tests passed against final four-file source, including cached unchanged-policy idempotency and changed-policy refusal. Author launched full realPG/Temporal alltargets session28026; source writes stopped pending checks, exclusive build/service slot retained. Fullmatrix/freeze/reviews/rootintegration remain pending.


Repair611 lazy-red60492 terminal101; reconciled-focused34565 terminal0 with7lifecycle+3policy cases passing. Author confirmed all authority/provenrelease/uncertainty checks are under broker transaction. Final fullmatrix launching, includes three previous regressions; handle/finalresult pending. No root sourceintegration or concurrent service use.
