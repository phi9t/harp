# Delegated bootstrap review dispatch

Date: 2026-10-04. Source read: author checkout base 159dd8b186630211a26d8d44967291f5700350b8. This evidence covers the operational wake helper, not product monitor integration or an end-to-end native background cycle.

The owner requires routine review in one fresh-context observe-only subagent. Main lead claims the wake, launches through the receiving harness's native tool, saves the returned identity and exact public source-bound brief, then releases its turn. Observer publishes bounded assessments/cursors/backlog and finishes that recorded wake. Main lead retains adjudication/user interaction; coordinator alone owns worker corrections/control.

Receipts are private durable files under the wake-state directory. Existing journal schema/configuration and original exact-thread begin/finish protocol are retained. Explicit replacement records old-writer quiescence/recovery evidence and preserves its predecessor. All retired identities remain fenced from reuse and delegate-finish. Missing tool support, uncertain launch/receipt or lost reviewer leaves reviewing/pending; ticks coalesce. The generated prompt binds current owner-selected dispatch policy to its helper path and digest, overriding the retained old spec snapshot's inline analyst role.

## Verification and independent review

Author observed failing prompt/absent-delegation contract checks, then green implementation. Independent reviewer `/root/monitor_design_critique` reproduced an A-to-B-to-A recovery defect: checking only the immediate predecessor allowed an older retired identity to return. Author added a real subprocess negative, observed it fail, then checked the full retained predecessor chain. The new test rejects A-to-B-to-A, allows fresh C, and rejects finish from retired A and B while retaining the active reviewing receipt.

Independent Standards PASS and Spec PASS cover the final helper/test hashes below; reviewer independently ran all 15 focused tests and checked the repaired negative control. Fresh author verification:

- `python3 tools/agents/test_monitor_wake.py`: 15 PASS, including all original 9 lifecycle/unknown-delivery/racing-tick/wrong-thread/stop/idempotency tests.
- `python3 tools/agents/tracker.py lint worker-monitoring`: PASS.
- `python3 -m py_compile tools/agents/monitor_wake.py tools/agents/test_monitor_wake.py`: PASS.
- `git diff --check`: PASS.

Final helper SHA-256: `3600a04ac1017955b2c3d0cd03921c127c62569048abb7bbcf3e2f14cb90e62f`.
Final focused test SHA-256: `acc627ea5cb2e5200520d101ea2f6fb30199bd77d6d56c694229bcdc34b040f9`.
Unchanged reviewed SPEC SHA-256: `56e88c6610cc9bca331e2a554e5cc3d55dd0ecb4ebd4a8299f314d964c6a27e4`.
Unchanged reviewed OPERATIONS SHA-256: `71fb6e57c04dc6000806d34fb34a50b5ed79d08b14df2e4a0f969fc0aed54ee3`.

## Installation evidence

Installed current tick/delegation helper: `/data02/home/philip.yang/.local/lib/corenius/monitor-wake/3600a04ac1017955b2c3d0cd03921c127c62569048abb7bbcf3e2f14cb90e62f/monitor_wake.py`, mode 0600 in mode 0700 directory, bytes verified against the reviewed digest. Existing `corenius-lead-monitor-01a10084.service` ExecStart now selects that helper with the existing wake-state path. Only that service path changed; user daemon-reload succeeded. No new timer, start, restart, stop or reset was performed.

Read-only before/after journal queries matched: stopped=0, original configured begin/finish helper 501e060e retained; wakes 2d80dd33 and 3a6c35ca remained finished; existing wake d3ffa87c-7737-40e4-b138-317c1f70a13f remained queued. Its old queued prompt is unchanged and needs explicit main-lead handoff before delegation. Old tick helper ea554311 remains installed unchanged.

Timer before/after matched exactly: active/waiting; NextElapseUSecMonotonic=`6d 11h 5min 36.828182s`; LastTriggerUSecMonotonic=`6d 10h 50min 36.825073s`. The author neither opened the observation ledger nor dispatched a live reviewer, changed live wake status, or touched worker/coordinator trees.

## Limits

Native actor identity and termination are local lead attestations, not runtime-authenticated guarantees. Delegate-finish trusts the recorded identity and nonempty durable-publication reference; it does not verify ledger contents. Original main-lead finish remains available for explicit reconciliation. One-writer safety depends on the lead respecting the grant and confirming old-writer quiescence.

The prompt requires deterministic public-event filtering and redaction before model input/tool output. Its contract test checks those instructions and order; this slice provides no executable sanitizer or sanitizer qualification. The separately interrupted reviewer printed excluded private Reasoning before filtering; it is a failed/pending incident, not completed coverage. Grant no assessment/cursor progress from that contaminated review; require fresh-context recovery. Incident publication/reconciliation is separately owned.

Native spawning must exist in the receiving harness. Timer/queue installation does not prove autonomous app dispatch or a completed asynchronous review/publication/finish cycle. No broad unchanged-repository gate, product integration, native dispatch or full live-cycle qualification is claimed. No mainline landing was performed.
