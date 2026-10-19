# 02 interrupted-effect-probe: Probe Pi worker recovery across a lost controller effect reply

Type: task
Status: resolved
Owner: codex
Blocked by:
Claimed:

## Acceptance
- [x] Use pinned Pi Durable with a faux provider and the real Corenius controller
  socket service against disposable databases and fixture credentials.
- [x] Persist the original effect request before sending it. Kill the actual Pi
  process after the controller commits but before the reply reaches Pi.
- [x] Reopen the same Pi store and recover the same tool task. With current
  authority, replay the identical request and observe exactly one progress effect.
- [x] Revoke the Attempt before recovery in a separate case. Observe refusal
  despite the existing receipt, no second effect, and retained occupied capacity.
- [x] Exercise changed-payload refusal and/or a duplicate-producing negative
  control to demonstrate what the adapter's stable request identity prevents.
- [x] Retain runnable source, sanitized results, source identities, test limits
  and proof that all probe-owned processes stopped. Independently review results.

## Scope

The owner approved this bounded probe after the source research. This is a
command-line persistence experiment, not a UI prototype or production adapter.
The effect is a scoped progress record in the real storage service. No external
workload, supervisor, Insula, live model or native runtime will be launched.
No commits, pushes or edits to product source. The author owns the isolated
`research/pi-effect-probe` worktree; the lead integrates artifacts here.

## Result

Both recovery cases passed in the author worktree and in a fresh integration
worktree run. One progress event remained after replay and after revoked-authority
refusal. Changed payload was refused; a fresh key/current revision produced a
second event. All six children and two proxy threads stopped. Credential-bearing
fixtures were removed. No product source changed.

Independent clean-context Standards and Spec review passed after source-provenance
checks were tightened. Evidence: `../evidence/interrupted-effect-probe/result.json`
and `../evidence/interrupted-effect-probe/review.md`. Runnable source:
`experiments/pi-env-durable/`. Findings: `docs/spikes/pi-env-durable/interrupted-effect-probe.md`.

This proves transactional ledger-mutation recovery only. It does not qualify
external execution, Pi Env, Insula, or a production adapter.

## Comments
- 2026-10-06T21:11:09Z codex: claimed until 2026-10-07T21:11:09Z
- 2026-10-06T21:17:39Z codex: resolved
