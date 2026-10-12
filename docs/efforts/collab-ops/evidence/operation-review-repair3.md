# Landing operation repair: review iteration 3

Read and repair base: `795b5714dd752d37d26814730db03ce272619eab`.
Original spec base: `c0a2fc0370f18ff28009dbe9b62a59916de0e582`.
Scope: roborev Medium finding concerning historical push intent during a later landing retry.
Status: implementation verified; independent re-review and root acceptance remain pending.

After an affirmative push rejection, the Attempt returned to `verified` but retained its historical push-intent event.
A later retry interrupted at `landing_prepared` could select that earlier event and enter ambiguous pushed recovery.
This blocked recovery even when the later operation used `--no-push` or had not reached its push-intent step.

## Red evidence

Two public CLI regressions use real disposable repositories and an actual rejecting pre-receive hook.
They retry the same verified Attempt and use `before_git_land` to stop after its new preparation event.
Both the local-only and pushing variants failed recovery with exit 2 instead of restoring `verified`.
The failure reported an ambiguous pushed landing using the historical operation's destination.

## Change

`StateStore.latest_event` accepts an optional `after_kind` boundary.
One SQL statement selects only events whose sequence follows the same Attempt's latest boundary event.
If that boundary is absent, the query matches nothing.
The default call retains its existing behavior, and SQLite access stays in the state store.

Landing recovery requests push intent only after the latest `landing_prepared` event.
It passes that bounded destination directly to pushed recovery.
A pushed state without a destination for its current operation refuses instead of consulting current upstream configuration.
Historical events remain durable but cannot classify a later prepared operation as already pushed.

## Green evidence

Three new public CLI regressions pass:

- Rejected push followed by an interrupted `--no-push` retry restores `verified` and then lands locally.
- Rejected push followed by an interrupted pushing retry restores `verified` and then lands on its upstream.
- A retry with a new upstream destination, interrupted after its actual push, recovers that new destination's exact Integration commit.

The positive destination test keeps the old remote main branch at its base and verifies the new staging branch at the Integration commit.
It proves that the boundary still selects current-operation intent when that intent exists.

All six existing core repair tests and nine existing collab recovery tests pass unchanged.
Focused `py_compile` and diff checks pass; modified Python lines remain within 120 columns.
The full repository gate was not repeated because the root owns that integration check.

Work stayed in the assigned isolated repair tree and disposable test repositories.
No policy, ticket record, observed checkout, shared mainline or unrelated implementation path was changed.
This is the final permitted repair iteration, 3 of 3; passing tests do not replace independent acceptance.
