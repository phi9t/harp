# 22 liveness-evidence: Retain workspaces with unusable worker evidence

Type: task
Status: resolved
Owner: trae
Blocked by:
Claimed:

Triage: ready-for-agent

## Context

Draft follow-up requested by the user on 2026-10-12. See [FOLLOWUP.md](../FOLLOWUP.md) and [architectural evidence](../evidence/architecture-review.md). Classification: gated; concurrency, publication and destructive cleanup invariants. No implementation or additional repair budget is authorized by this draft.

## Acceptance
- [x] Missing, malformed, non-mapping, nonpositive or invalid PID/session and malformed completion produce unknown with an explicit reason.
- [x] Inspect, abandon, recover, refresh, GC, sweep and retention consume the same conservative decision and retain unknown workspaces/refs.
- [x] Valid stopped workers permit cleanup; live sessions override completion; explicitly unbound confirmation still works.

## Comments
- 2026-10-12T17:39:00Z trae: unblocked by resolved ticket 19 decision record `decisions/2026-10-12-mutation-contract.md`.
- 2026-10-12T17:38:46Z trae: claimed until 2026-10-13T17:38:46Z
- 2026-10-12T17:52:00Z trae: replaced boolean worker liveness with shared `running|stopped|unknown|unbound` decisions and routed inspect/recover/abandon/refresh/GC/sweep/retention through the same conservative gate. Evidence: focused 7-test liveness command passed; expanded liveness/lifecycle/refresh/cleanup command passed 32 tests; `py_compile` and `git diff --check` passed.
- 2026-10-12T18:54:20Z trae: resolved with --force despite missing accepted, landed Candidate
