# Ticket 05: content accepted; re-integrate once more

Reviewed Candidate: fe86c18. I re-ran my two-worktree reproduction against it: launching from the main
checkout now claims on the integration branch in its own worktree, cuts the worker from the post-claim tip
and leaves the mainline and the main checkout untouched; an integration branch checked out nowhere is
refused. The access-profile pass-through and the hand-off lines are fixed. Nothing else to change.

While you were reworking, ticket 09 landed on `effort/lead-worker-readiness` and changed
`bench/harness/lead_driven.py`, its test, and `tools/agents/worker.py` (external protected paths). Your
Candidate is behind the tip again. This is not a review round.

Do: rebase your own branch onto the current tip of `effort/lead-worker-readiness`, resolve conflicts by
intent (keep the `external-state-report` scenario and the external-protect option; keep your `lead-launch`
kind), pass `--external-protect-path` through the launch command the same way you pass the other profile
options, rerun the whole gate including all six scenario commands, and finish with a new final commit
`ticket 05: ready for review` on a clean tree. Nothing will land ahead of you this time.
