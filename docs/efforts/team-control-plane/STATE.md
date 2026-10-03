# State: durable team control plane

Status: implementing the approved baseline; full goal active.
Updated: 2026-10-02T21:42:28.765927+00:00
Local main: `1f164156261a3aa927c823919bf73d6e6d11a771`.
Root worktree: `/data02/home/philip.yang/workspace/corenius-codex-reference-design`.
Branch: `codex/reference-design`; committed candidate `dd8e14cff65c00e7936c2d0ce9471aed973c777f`.

## Goal and authority

Make Corenius the executable reference for persistent replaceable Codex/Claude leads and TraeCLI workers. Preserve assignments, decisions and evidence through failures, prevent unsafe retries, and accept only verified candidates. The complete six-phase scope remains in [PLAN.md](PLAN.md) and [SPEC.md](SPEC.md).

The owner approved the recommended Rust/PostgreSQL/Temporal baseline and authorized continuation, worker replacement and gated landing across worktrees. [Continuation authority](decisions/20261002-continuation-authority.md) and [landing authority](decisions/20261001-landing-authority.md) preserve this scope. No additional permission is currently required for the kernel work. No remote is configured. Necessary technical provider-isolation and bounded-execution prerequisites remain to prove before live work.

## Accepted and landed baseline

Tickets10–15 and17 are accepted within their explicit scopes and landed to local main. Current main includes subsequent accepted baseline updates through `1f164156261a3aa927c823919bf73d6e6d11a771`. Kata remains deferred for the first kernel; conditional ticket18 closure awaits16. jj has a pinned executable preflight, not ticket19 acceptance or backend adoption.

## Current kernel candidate

Ticket16 remains claimed, unaccepted and unlanded. Final repair source is committed as `c4599e8be5720ff7da74ce886069ff0da0560b80`. Freeze3 has independent Standards and Spec PASS; root verified all48 integrated and committed blobs and passed whole repository gate7 with exit0 and Temporal cleanup. Author matrix passed159 tests with0 failures and7 explicit ignores; final strict lint, formatting, diff and affected native test passed. Committed branch review575 (`main..c4599e8`) finished CHANGES REQUIRED with four new operator-visibility and log-evidence findings. Independent architectural assessment confirmed all four gaps in operator projections and byte-stream evidence. The [bounded interface contract](decisions/20261002-kernel-interface-repair575.md) is recorded; a new isolated author at `/tmp/corenius-kernel16-review575-author`, base `c4599e8`, has established meaningful public reds and focused greens for all four findings. All seven operator fixtures pass, including exact omitted-Mission pause/resume, legacy running/quarantined discovery, foreign-scope refusal and unreadable-log refusal. Real invalid-UTF8 verifier output supports Verify/Accept; a128MiB raw log finalizes under measured64MiB monitor headroom. The full real PostgreSQL/Temporal matrix passed on author session78915:162 tests,0 failures,7 explicit ignores,16 groups. Root verified all48 frozen files and7 Git preimages; independent Standards and Spec reviews both passed against immutable manifest `6ad5e6e759050e7e2ca2d7150c2141b0d55336d2411d44db1ce0b1aac59ac51b`. Strict lint, formatting and diff checks passed. Root integrated seven reviewed files and verified all48 postimages; whole repository gate8 passed on session13946. Repair is committed as `dd8e14cff65c00e7936c2d0ce9471aed973c777f`; root verified all48 committed Git blobs against the reviewed freeze. Fresh whole-branch review580 on session36530 terminated1 with three Medium recovery findings: original unused reservation refund after continuation cancellation, monitor-crash recovery finalization, and retained logs over1MiB blocking takeover. Author and independent Spec confirmed all three existing-contract gaps. The [recovery contract](decisions/20261002-kernel-recovery-repair580.md) records provenance, immutable race handling and secure log rules; iteration2 of the existing three-iteration round is assigned to a fresh isolated author at `/tmp/corenius-kernel16-review580-author`, exactbase dd8e14c. The first public capacity regression reproduced used_capacity1 after cancellation/release; its focused green now proves capacity2→0, duplicate reconciliation, replacement admission and unchanged cumulative budgets. Real monitor-crash focused green now proves replacementOwner finalization/release with unknownexit and immutable repeated result. LegitimateVerifier refusal, same-sequence capacity/recovery guards and CAS controls remain pending; actual large-log takeover regression is being run. Acceptance and landing remain pending. Earlier SIGABRT setup failure left an owned SDK orphan; identity-bound termination and three exact owned-root removals are retained, superseding the incorrect initial no-spawn claim. Acceptance, final evidence commit and exact-tip gated landing remain pending. [Candidate evidence](evidence/kernel16-review571.json).

Branch571 rejected the preceding `da7ff9b` candidate with three confirmed lifecycle findings. The approved [repair contract](decisions/20261002-kernel-lifecycle-repair571.md) now has public proof for once-only unallocated Worker/Verifier capacity recovery without cumulative budget refunds, genuine old-binary Yield compatibility, authenticated bound Cancel after work revocation, and historical verifier evidence surviving producer expiry while live authority remains enforced. The first repair freeze failed Spec for imported first Worker unresolved Yield; genuine public regression showed premature refund while an actual job remained live. The final guard quarantines both imported and independently reserved episodes and holds their capacity. Earlier failures, candidates and reviews remain retained historical evidence, not current acceptance.

The [independent requirement audit](evidence/kernel16-acceptance-audit-review571.json) identifies no additional required kernel gap. Historical namespace observation EACCES cause remains unproven; persistent inaccessible inventory quarantines and holds capacity. Controlled late-completing pre-failure verifier and mixed-check physical fixtures remain unexercised. Colliding-v2 rollover is proven at payload level, not forced actual Continue-As-New. Successful registered SDK takeover is demonstrated; unsupported legacy gateway recovery quarantines safely. No live-provider or rollback-restore property is inferred from model-free kernel evidence.

## Remaining full-goal work

After16 acceptance: explicitly close deferred18; complete jj evaluation19 and conditional backend25; prove live Trae probes20 and adapter21; independently demonstrate Codex22 and Claude23 leads; verify upgrades, backups and restore24. Current restart recovery does not prove rollback restore. No live Trae/model-provider demonstration or complete restore has run.

The [completion audit](completion-audit.md) tracks requirements without shrinking the full objective. Historical implementation chronology, including the superseded state snapshot, is preserved in [implementation-history.md](implementation-history.md). This is a manual durable state record; `effort.py dehydrate` is not invoked while reviewed evidence changes are pending because it auto-commits.

### Recovery580 verification update

The final focused run `42143` exited zero: nine public parent tests plus the owned singleton child passed. Proof includes capacity refunds and unknown-usage retention, delayed/fast real monitor crash, surviving-thread termination refusal, exact-candidate recovered-Verifier refusal, historical missing-provenance monitor Verify/Accept compatibility, actual large-log takeover and seven security cases. Full PostgreSQL/Temporal matrix `20645` is in progress; no immutable source freeze, independent conformance verdict, new root gate or landing exists yet. Earlier aborted run and exact process cleanup remain retained. The whole six-phase goal remains incomplete.

Full matrix `20645` is now terminal `101`, with vertical process stack overflow in a historical two-scenario verifier wrapper. The author identified shared future growth and is applying a consistent heap boundary at fixture entry rather than changing stack limits or narrowing scenarios. Exact orphan cleanup is required before rerunning. The previous final focused pass remains valid at its tested source, but does not establish a full-suite pass.

Corrected-source matrix `25437` uses the shared boxed fixture entry across all scenarios and `matrix-corrected-heap.log`. The accidental unchanged-source run is terminal101 and excluded; cleanup is verified with retained author reference scans and root process checks. Corrected matrix cancellation group has 27 passed/3 ignored and capacity group is in progress, not full acceptance. Ticket19 jj evaluation remains blocked by16; no dependency bypass or live provider run was launched.

Both independent repair580 conformance axes PASS. Exact seven-file reviewed patch integrated into isolated root tree on dd8 base, with all48 preimages/postimages verified. Full repository gate9 session3092 is live. Root source remains uncommitted pending gate; main is unchanged and six-phase goal still incomplete.

Current source HEAD is `480d7c22bdfd926e1534c4680988d63d50618d2e`: exact reviewed recovery580 repair, all48 Git blobs verified. Root gate9 passed terminal0 with14 retained cleanup records. Whole-branch job584 (`1f16415..480d7c2`, session94656) is live; acceptance and mainline landing remain pending. Main stays1f16415. The full six-phase goal is active.

Wholebranch584 rejected480d7c2 with two Medium findings: historical stopped crash recovery afterepochrotation and cancelled currentWorker inspection/cleanup authority. Independent assessments requested. Prior green checks remain evidence but not acceptance. No mainline movement; final allowed repair would iteration3/3, no budget reset.

Final repair3/3 now assigned after both assessments: fresh isolated kernel16_review584_author at480d7c2. Canonical cleanup contract20261002-kernel-cleanup-repair584.md; author owns exclusive runtime slot, root source unchanged. No public proof yet; full goal and ticket16 remain incomplete.

Final repair584 has full author matrix173/0/8 and strict/fmt/diff0, verified48-file freeze plus4exact480preimages. Both independent conformance axes PASS after allauthorityconsumer tracing. Exactfourfilepatch integrated roottree on480base with48postimages verified; root fullgate10 session32923 polledlive, sourceuncommitted. Selected37publicproof/terminal/cleanuprecords retained separately from broadruntimeinventory. No sourceacceptance/landing yet; fullsixphasegoal remainsactive.

Current source9055b9976d429229e5d42b2e86eb04dbc30da5bf has exactfinalrepair3/3; author173/0/8 strict0 bothconformancePASS rootgate10terminal0/17cleanuprecords/all48Gitblobmatches. Wholebranch591 session99070 live; no acceptance or landing. Currenthandoff/completion row refreshed; historicalrejections retained. Fullgoalstillactive.

Finalwholebranch591 rejected9055b99 with oneMedium role-isolation finding in episode status summaries. Existing3/3repairbudget exhausted; read-only validity assessment assigned, no furtherpatch/landing authorized by this budget. Mainunchanged; fullgoalincomplete.

Owner explicitly approved oneadditionalbounded591repair/reviewround. Approval recorded20261003-kernel-role-isolation591-approved-extension.md; prior3/3historynotreset. Fresh isolatedkernel16_review591_author solewrites /tmp/corenius-kernel16-review591-author at3e2622e source905same; ownsCargo/PG/Temporal. Fullgoalincomplete; no mainline movement.


### Repair591 frozen candidate verified

Root verified all 48 current source files, all 130 frozen files, the exact patch and two Git-base preimages. Retained 76 selected public proof files plus four metadata/patch records. The full matrix records 174 parent-suite passes, zero failures and eight owned subprocess entrypoints ignored; its separately nested child records one pass. Independent Standards and Spec reviews are pending. No source integration or landing has occurred. The owner’s full-goal authorization supersedes routine repair permission stops; see `decisions/20261003-owner-authorizes-full-goal-execution.md`.


### Whole-branch611 rejection and authorized repair

Review611 session86801 terminal exit1: High verification execution provenance omits immutable suite/environment policy digests; Medium deadline-stopped released lead episode remains live and prevents successor admission. Earlier conformance/gate passes do not override this rejection. Fresh isolated repair assigned to kernel16_review591_author in a new review611 worktree at587aca8 under full-goal owner authorization; source integration and landing withheld. Review and repair contract retained in decisions/20261003-kernel-policy-lifecycle611-repair.md and evidence/kernel16/branch-repair/branch611/.
