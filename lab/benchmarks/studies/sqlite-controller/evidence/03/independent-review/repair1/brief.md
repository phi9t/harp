Ticket03 consolidated repair1/3. Exclusive writer remains /root/sqlite03_kata_author in corenius-sqlite03-kata, branchsqlite03/kata. BASE40d4805ce34d74521f3473b7f8ad2b9f7451d0c2 retains clean frozen productd201642b36695dbd0f29a7ac857ee9e8de520728. Read same implementation brief and current canonical ticket03/SPEC. Standards PASS (LOW possible captured-source Value/type-design heuristic deferred); SpecFAILone and branch643FAILfive below. Root acceptance is withheld. Verify findings before minimal fixes; one consolidated round, no helpers/native cleanup changes/other tree writes/ref rewrites/production or model calls/tmux.

Retain all existing evidence/source commits. Capture exact complete source/test archive+hashes BEFORE every credited public RED/GREEN command and frozen gate. Setup/oracle failures stay uncredited. Use real pinned supported Kata daemon/API, unique owned fixtures and exact observed process identities; no manufactured HTTP successes or raw Kata SQL. Boundary failures must reach the intended public behavior, not missing setup. Preserve current auth/source authority, immutable admitted versions, finite idempotency/readback, unknown effects and historical failures.

For the six findings require public genuine RED then covering GREEN: checked-path/file replacement cannot expose outside registered source; delayed genuine Kata responses do not block controller socket/control receipts; unresolved taskA publication does not block independent taskB while per-task order holds; maximum accepted progress including escaping/multibyte text retains full local report and publishes bounded valid summary; deterministic fixture is feature-gated and default test/build behavior safe; unchanged source after actual outage/restart clears only current availability uncertainty, retaining failure history and real projection drift. Final affected controller/Kata/public/process/runtime checks, fmt/strictClippy/normal build and a clean frozen source/precommandmanifest/handoff are required. Root decides whether broader tests are warranted by changed source; do not repeatedly run unchanged native25 for Kata-only changes. New native production/cleanup changes need root adjudication first; existing642 remains required04.

Separate independent full-gate failure must be diagnosed, not silently rerun into PASS: root session22957 is terminalexit101, test891.78665s; library4pass, native24pass/1fail, subsequent suites/fmt/clippy/build not run. Failure repair3_adopted_thread_delivers_received_first_turn_notifications in SEPARATE topology, native_registry.py248 wait(bound,30) =>behavior deadline, actualA firstturn/tools/history complete. Failure fixture/tmp/cnr-t94tiojc was disposed; root549retainedidentities0matchinglive/199terminaldisposalreceipts. Raw log/rootgate pins: root reviews/03/root-gate/root-test.log,gate.json,root-resources.json. Relevant source stderr repeatedly native ownership lookup ENOENT then remote executor pending. Root public gate source remains exactlyd201; author priorfull69PASS retained on earlier source with unchanged native/core bytes.

Begin with a focused unchanged-source check of that one inherited native case (source-pinned command, both topologies; if adding diagnostic output do so in a precisely recorded disposable/fixture variant). Obtain current binding/readiness/hold state before failure to identify cause. A passing rerun alone does not resolve the failure. No timeout inflation/identity weakening/ignoring assertion or new02 registry patch. A concrete fixture-ordering fault may be corrected and covered without native production change; if the cause is a native production/architecture issue or remains uncertain, report exact evidence to root for bounded routing while continuing the six independent Kata fixes. Final baseline release remains gated.

Append repair report to docs/efforts/sqlite-controller/evidence/03/handoff.md, retaining initial evidence unchanged and naming new repair1 evidence separately. Return exact clean source+evidence tips, tests/source/process dispositions and remaining concerns. Root owns scoped Standards/Spec/branch rereview and complete final gate. Necessary clean commits authorized; no source landing or user session mutation.

## Exact branch643 findings (verbatim)
## Review Findings

- **Severity**: High
- **Location**: `crates/sqlite-controller/src/kata.rs:414`
- **Problem**: Snapshot symlink checks and file reads operate on separate pathname lookups. A concurrent writer can replace a checked file or ancestor with a symlink before `std::fs::read`, causing the controller to capture files outside the registered repository. Captured bytes are subsequently exposed through participant context.
- **Fix**: Resolve paths relative to an opened repository directory using descriptor-based traversal that refuses symlinks, and validate/read the same opened file.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/service.rs:73`
- **Problem**: Autonomous Kata reconciliation blocks the sole socket-serving loop. Each drain processes up to 100 effects and then polls every bound source sequentially. Slow responses or many unavailable sources can delay all participant requests, including cancellation, for seconds or longer.
- **Fix**: Run reconciliation independently of socket acceptance, or give each drain a bounded work/time budget and resume it across iterations. Verify socket responsiveness while genuine upstream responses are delayed.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/kata.rs:1033`
- **Problem**: Any failed effect stops the entire publication batch. Persistent projection drift or an expired ambiguous comment for task A therefore prevents healthy task B from publishing progress or cancellation indefinitely, including across projects during autonomous reconciliation.
- **Fix**: Preserve ordering within each task while allowing unrelated tasks to continue. Add coverage showing that one task’s unresolved effect does not block another task’s synchronization.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/src/kata.rs:875`
- **Problem**: `report_progress` accepts text up to 65,536 bytes, but the generated comment adds framing and JSON string escaping. `apply_effect` subsequently validates that comment through `text`, which has the same 65,536-byte limit. A valid progress report can therefore create a permanently unpublishable effect; later synchronization also reuses that progress text.
- **Fix**: Bound or truncate the visible progress excerpt before queuing the comment, accounting for formatting overhead, while retaining the full report locally. Test a maximum-length report and text requiring escaping.

---

- **Severity**: Medium
- **Location**: `crates/sqlite-controller/tests/kata.rs:3`
- **Problem**: This test runs without `fault-injection`, although its Python fixture relies on `CORENIUS_TEST_MANUAL_RUNTIME=1`. The service only honors that setting with the feature enabled. Default test builds consequently reconcile autonomously, racing expected revisions and consuming effects before the fixture’s acknowledgement-loss assertions.
- **Fix**: Gate the test on `fault-injection`, matching the documented invocation, or make the fixture work reliably with autonomous reconciliation.

## Summary

The change adds pinned Kata source capture and admission, journaled publication, and scoped participant context and followup proposals.
## Exact independent Spec report (verbatim)
Spec axis: **FAIL** for `b55bd9c3e6e44bbb03b8e1ca026e9a3210d8829f..d201642b36695dbd0f29a7ac857ee9e8de520728`. Evidence-only tip `40d4805ce34d74521f3473b7f8ad2b9f7451d0c2` has identical product bytes.

**P2 — recovered unchanged sources retain a false outage.** [kata.rs:598](/data02/home/philip.yang/workspace/corenius-sqlite03-kata/crates/sqlite-controller/src/kata.rs:598) returns when a successful capture equals the retained current version. A prior [source failure](/data02/home/philip.yang/workspace/corenius-sqlite03-kata/crates/sqlite-controller/src/kata.rs:574) has independently stored `source observation unavailable` in `kata_sources.drift`. After connectivity returns without a Kata/filesystem change and with no queued publication, autonomous/operator reconciliation keeps taking this early return; [participant context](/data02/home/philip.yang/workspace/corenius-sqlite03-kata/crates/sqlite-controller/src/kata.rs:1193) still reports the obsolete outage. An explicit `refresh_kata_source` or later revision/content change can clear it, but ordinary unchanged-source recovery cannot.

Canonical [ticket03:12](/data02/home/philip.yang/workspace/corenius-codex-sqlite-controller-execution/docs/efforts/sqlite-controller/tickets/03-kata-participant-protocol.md:12) requires “Prove duplicate delivery, offline recovery, changed task refusal, unauthorized claims/closure and replacement-lead reconstruction.” [SPEC84](/data02/home/philip.yang/workspace/corenius-codex-sqlite-controller-execution/docs/efforts/sqlite-controller/SPEC.md:84) requires “retain controller event/revision and acknowledged Kata revision, and show pending synchronization or drift.” Reconcile availability independently of content equality, retaining historical failures and genuine projection drift. The retained offline GREEN queues publication and changes the remote revision ([fixture:300](/data02/home/philip.yang/workspace/corenius-sqlite03-kata/crates/sqlite-controller/tests/support/kata_protocol.py:300)); it omits this boundary. Finding is source-traced, not newly reproduced.

Otherwise, the delta implements immutable selected filesystem/Kata versions, global actual WorkRef ownership, scoped participants/context and journaled supported metadata/comment readback with finite idempotency retention. Explicit file selection—not automatic reference closure—is the capture contract. The additive schema retains version1/2; matching configuration is required, while old-binary downgrade fencing is expressly unsupported. Native642 uncertainty remains exposed and belongs to ticket04.

Evidence: eight 29-file archives match manifests; frozen 29 match source and final covering manifests. Sealed RED 25-file archive equals base. Actual pinned Kata 0.24/source b70a430 fixtures retain public source/authorization/effect/offline proof. Historical 69 tests plus final affected 40/clean-parent/fmt/Clippy/build are honestly composed; root’s final full gate remains separate and running. Declared regression variants/setup failures remain distinguished.

No tests, checkout edits, credentials/upstream calls, process launches/cleanup, tmux or helpers. Prior ticket02 reports unchanged. Root owns acceptance; no assembled/live qualification inferred.

## Authorized bounded native fixture correction (2026-10-04)

Independent adjudication at canonical evidence/03/independent-review/initial/native-fixture-adjudication (original report /data02/home/philip.yang/devx/tmp/corenius-03-native-fixture-adjudication-0ts0apno). Source-proven adoption-fixture circular wait; exact root failure attribution remains incomplete and real B list_turns refusal is retained separately. This is a fixture reliability correction within03 consolidated repair1, not native production/642 cleanup or another02 round.

Execute the report's precise declared scheduling deferral against old global-hold behavior, with complete pre-command source/driver/binary pins. Defer exact admitted B before native binding RPC while A lacks first projection, then once more in A's projection pass and disarm. Preserve real A post-confirmation crash, first-turn tools/history/received frames. Genuine RED must reach A immutable snapshot/globalhold+B nonready+deferralmarker+no retry after disarm inside unchanged30s deadline; missing prerequisites remain inconclusive. No fabricated native replies or DB writes.

Then minimal FI-only per-actor snapshot freeze: preserve A first exact projection and prevent only this binding's subsequent fixture reconciliation. Leave B eligible to retry normally, retain B first qualified actual projection/creation-event control. Global hold only once both exact actors qualify. No synchronous callback wait, widened production permission, timeout inflation or weakened original assertions. Both actual shared/separate topologies must GREEN, including exact one thread/turn/client/frame evidence and controlled peer deferral. Persistent actual native refusal is still a failed capability and must be reported, never waived. All code changes strictly cfg(fault-injection)/adoption-fixture mode; native production source behavior unchanged.

Retain existing49 affected Kata/default/fmt/Clippy/build proof; after fixture change run exact native adoption case both modes, affected feature-off/fmt/lint/build checks as needed. Freeze clean source and full29-file archive+test inventory; root runs fresh appropriate scoped gate and independent conformance. Do not repeat full unchanged native25 without a new unresolved concern. All prior failures/historical source tips/process evidence remain preserved.
