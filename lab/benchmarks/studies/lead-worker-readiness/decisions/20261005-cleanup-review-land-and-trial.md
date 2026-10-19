# Decision: disk cleanup by a worker, review and land the integration branch, live trial may go ahead

Date: 2026-10-05, 19:30 UTC.
Authority: the owner, in the lead session, in answer to the lead's status report:
"have a traecli worker clean up disk space; review integration branch thoroughly, fix issues and then land
(we can also leave some TODOs for future); live trial can go ahead".

## Accepted direction

1. A traecli worker cleans up disk space on the data volume.
2. The integration branch `effort/lead-worker-readiness` is reviewed thoroughly, the issues found are fixed,
   and it is then landed. Issues that are not worth fixing now may be left as recorded TODOs.
3. The live trial of the lead-driven instantiation (ticket 14) may go ahead. This is the owner's go-ahead
   that ticket 14's first acceptance item requires, and it covers the model spend of that trial.

## Lead's reading, where the words leave room

- "Land" means fast-forwarding the mainline `main` to the reviewed tip of the integration branch, with the
  owner override the lander now requires for `main`, recorded with the owner's name. It does not include a
  push: the repository has no remote.
- "Clean up disk space" does not name what may be deleted. The worker is given an allowlist (this effort's
  own finished workspaces, temporary directories and caches that are regenerable and not in use) and must
  report everything else with sizes for the owner to decide. It may not delete another effort's or another
  agent's worktrees, branches or evidence.
- "Thoroughly" is taken as: a second independent review by a context that is not the lead, after the last
  two build tickets land, covering the whole range `main..effort/lead-worker-readiness`; every blocking
  finding fixed and re-checked before landing; the rest written into a tracked TODO file.
- The order is review and land first, then the live trial on the landed tools, unless the review shows the
  trial is needed to settle a finding.
