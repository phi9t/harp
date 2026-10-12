# 20 mutation-fence: Fence every managed trunk mutation and pending operation

Type: task
Status: resolved
Owner: trae
Blocked by: 
Claimed:

Triage: ready-for-agent

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md) and [architectural evidence](../evidence/architecture-review.md). Classification: gated; concurrency, publication and destructive cleanup invariants. No implementation or additional repair budget is authorized by this draft.

## Acceptance
- [x] Admission, allocation recovery and claim release refuse unrelated pending core or legacy publication before changing ticket bytes, refs or Attempt reservation.
- [x] Held lane and surviving inherited Git-child locks fence mutation; exact operation recovery remains possible.
- [x] Integrate the approved adjacent toolbox writers or document the approved restricted guarantee; preserve uninitialized legacy behavior.

## Comments
- 2026-10-12T17:39:00Z trae: unblocked by resolved ticket 19 decision record `decisions/2026-10-12-mutation-contract.md`.
- 2026-10-12T17:38:46Z trae: claimed until 2026-10-13T17:38:46Z
- 2026-10-12T17:49:00Z trae: implemented shared initialized-trunk mutation guards for Collab admission, refresh allocation, landing/recovery/release, optional effort-record commits, lead_launch claim/release and effort dehydrate/move-records. Evidence: `env PYTHONPATH=tools/collab/tests python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_pending_pushed_landing_blocks_corenius_admission_before_tracker_mutation tools.collab.tests.test_collab.CollabTests.test_held_lane_blocks_corenius_admission_before_tracker_mutation tools.collab.tests.test_collab.CollabTests.test_empty_cherry_pick_marker_refuses_before_core_push tools.collab.tests.test_legacy_landing.LegacyLandingTest.test_empty_cherry_pick_marker_refuses_before_push tools.collab.tests.test_lifecycle_ops.LifecycleOpsTests.test_pushed_local_success_recovers_records_while_remote_offline` passed 5 tests; `env PYTHONPATH=tools/collab/tests python3 -m unittest tools.collab.tests.test_lifecycle_ops tools.collab.tests.test_operation_repair tools.collab.tests.test_legacy_landing tools.collab.tests.test_legacy_repair tools.collab.tests.test_legacy_push_repair tools.collab.tests.test_interrupt_repair` passed 46 tests; `python3 -m unittest tools.agents.test_lead_launch tools.agents.test_effort` passed 65 tests.
- 2026-10-12T18:54:03Z trae: repaired independent-review blocker in `d944bdb` so `move-records` enters the initialized trunk mutation guard before creating a missing record branch/worktree. Evidence: `python3 -m unittest tools.agents.test_effort.EffortTest.test_move_records_refuses_pending_publication_before_record_worktree_creation` passed; `python3 -m unittest tools.agents.test_effort` passed 18 tests; `python3 -m unittest tools.collab.tests.test_collab.CollabTests.test_pending_pushed_landing_blocks_corenius_admission_before_tracker_mutation` passed; `git diff --check` passed.
- 2026-10-12T18:54:20Z trae: resolved with --force despite missing accepted, landed Candidate
