# 24 safety-qualification: Qualify the follow-up against real Git and independent reviews

Type: task
Status: claimed
Owner: trae
Blocked by:
Claimed: 2026-10-12T18:02:04Z expires 2026-10-13T18:02:04Z

Triage: ready-for-agent

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md) and [architectural evidence](../evidence/architecture-review.md). Classification: gated; concurrency, publication and destructive cleanup invariants. No implementation or additional repair budget is authorized by this draft.

## Acceptance
- [x] Public regressions cover the important mutation, preflight, liveness, local-record recovery and archive compatibility surfaces from architecture-review.md without requiring every retained reproduction to become a separate long-running test.
- [x] Focused unit/toolbox coverage and a small live/e2e set pass, with lint/diff checks; the full default gate is not required for this follow-up qualification.
- [x] Fresh Standards and Spec reviews plus independent branch defect review have no unresolved blockers before declaring Collab Ops accepted.

## Comments
- 2026-10-12T17:57:00Z trae: unblocked after integration branch merged ticket 20/21/23 mutation-recovery branch, ticket 22 liveness branch and ticket 25 archive compatibility branch with focused affected suites passing.
- 2026-10-12T18:02:04Z trae: claimed until 2026-10-13T18:02:04Z
- 2026-10-12T19:36:00Z trae: user narrowed qualification strategy to reasonable unit coverage for important surface area plus a few live/e2e checks; interrupted full default gate exits are not acceptance evidence.
- 2026-10-12T20:22:00Z trae: focused qualification passed on current tip after review-found repairs. Evidence: `env PYTHONPATH=tools/collab/tests python3 -m unittest tools.agents.test_effort tools.collab.tests.test_lifecycle_ops tools.collab.tests.test_core_repair tools.collab.tests.test_cleanup tools.collab.tests.test_legacy_landing tools.collab.tests.test_collab.CollabTests.test_pending_pushed_landing_blocks_corenius_admission_before_tracker_mutation tools.collab.tests.test_sweep_repair` passed 72 tests; `python3 tools/agents/test_operational_state.py` passed 8 tests; `python3 tools/agents/operational_state.py` passed; `python3 -m unittest bench.harness.test_lead_driven.LeadDrivenTest.test_exact_candidate_landing_scenario_lands_candidate_and_binds_gate bench.harness.test_lead_driven.LeadDrivenTest.test_parallel_workers_and_overlap_scenario_runs_through_lead_driven_mode` passed 2 tests; `git diff --check main...HEAD`, `git diff --check`, and `python3 tools/agents/tracker.py lint collab-ops` passed. Fresh review agents: Standards `01a131af-61bd-7710-a3fd-f32cc613550b`, Spec `01a131af-5e19-71b2-8731-df6e388f87fd`, independent defect `01a131af-673a-7173-8784-5e5da9897656`; all reported no unresolved blockers.
