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
