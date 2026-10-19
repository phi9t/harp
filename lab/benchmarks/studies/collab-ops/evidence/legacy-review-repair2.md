# Legacy push repair: review iteration 2

Read and repair base: `09ced6dfc0cebbeb2edac3e36b3ce9bc295f8d65`.
Original spec base: `c0a2fc0370f18ff28009dbe9b62a59916de0e582`.
Scope: roborev High finding concerning interrupted legacy pushes and premature fence removal.
Status: implementation verified; independent re-review and root acceptance remain pending.

An unchanged remote was previously sufficient to roll back `push_started` or classify a failed push as terminal.
The Git push process did not inherit the landing locks.
After controller death, recovery could clear the fence while the surviving Git process was still waiting to update the remote.

## Red evidence

The new public CLI regression uses a disposable bare remote with a delayed receive-pack.
It kills only the controller after the receiving process starts, then invokes recovery while the remote is still unchanged.
At the repair base, recovery returned immediately instead of remaining fenced behind the live Git child.

A separate receive-pack transport failure leaves the remote unchanged without returning a target-ref rejection.
Before the correction, the journal changed to `push_failed`; the regression requires retained `push_started` uncertainty.
The process-death boundary immediately after recording `push_started` also has a dedicated public failpoint regression.

## Change

Git result children inherit the active `land.lock` descriptor and the initialized collab lane descriptor.
The descriptor list is scoped to the landing context and does not affect gate execution outside that lock.
Kernel locks therefore remain held when a Git child survives its controller.
The initialized-repository test also proves that public collab backup refuses while that lane remains held.

`push_started` remains uncertain when the remote still names the old tip.
Recovery permits rollback only before push start or after an affirmative rejection for the exact target ref plus an unchanged remote.
Generic transport errors retain the fence, even after the failed Git command exits.
Legacy `push_failed` records without affirmative rejection evidence also remain fenced.

Push now requests Git porcelain output.
The parser accepts only the exact target's `[rejected]` or `[remote rejected]` status.
It excludes `[remote failure]` and does not infer rejection from diagnostic text.
The rejection event also binds the exact pushed source tip.
Initialized event export retains the same Candidate and target binding.

A failed command after the remote reaches the exact tip records `pushed` and remains recoverable.
Recovery fast-forwards only that bound tip after the existing preflight and version-2 gate workspace checks.
It never retries a push or moves a remote backward.

## Green evidence

Seven new public CLI regressions pass:

- A delayed push retains the standalone lock after controller death and recovers the exact tip after completion.
- The same delayed push retains the initialized collab lane as well.
- A generic transport failure with an unchanged remote remains uncertain and blocks further landing.
- A recorded push start without an outcome refuses rollback.
- An explicit exact-ref rejection permits recovery without moving refs and allows a subsequent fresh landing.
- The same rejection records and recovers correctly in an initialized repository.
- A transport failure after remote update preserves the `pushed` phase and recovers the exact tip.

The existing 31 landing tests, 15 legacy integration tests, six prior repair tests and seven temporary-root tests also pass unchanged.
Focused `py_compile` and diff checks pass; modified Python lines remain within 120 columns.
The full repository gate was not repeated because the root owns that integration check.

All pushes, process faults and file removals occurred only in disposable test repositories.
No synchronized policy, ticket record, observed checkout, real remote or shared branch was modified.
This is repair iteration 2 of the three-iteration limit; passing tests do not replace independent acceptance.
