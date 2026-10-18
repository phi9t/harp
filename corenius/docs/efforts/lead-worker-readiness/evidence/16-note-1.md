# Note for ticket 16: a defect in the effort-record lookup that ticket 07 just landed

Ticket 07 added one lookup for an effort's record (`resolve_effort_record` and friends in
`tools/agents/land.py`, used by the lander, the launch command and `effort.py`). It searches every worktree
of the repository for `docs/efforts/<effort>/EFFORT.md` and refuses when several have it and no single one
is checked out on the record's `Integration branch`. I asked for that rule in review, and it is wrong for
this repository: there are 113 worktrees, and every older effort (`agentic-engineering`, `multi-agent-bench`,
`team-control-plane`, ...) exists in almost all of them with no `Integration branch` line. Run on the real
repository right after landing:

    python3 tools/agents/effort.py --repo /data02/home/philip.yang/workspace/corenius status
    error: ambiguous effort record for 'agentic-engineering': found multiple records and no single worktree
    is checked out on the record's Integration branch: ...

So `effort.py status` fails for the whole repository, and `rehydrate` and `dehydrate` of any older effort
will fail the same way. Other agents use those commands on other efforts; this must be fixed before the
integration branch can go to the mainline. `lead-worker-readiness` itself still resolves, because exactly
one worktree is on its integration branch.

Since your ticket owns where an effort's bookkeeping lives and already touches `effort.py`, `land.py`,
`lead_launch.py` and the tracker, fix the lookup as part of it:
- The copy under the given `--repo` wins when it exists. Only when `--repo` has no copy are other worktrees
  searched, and then the rule from ticket 07 applies (the one on the record's integration branch).
- An effort whose record names no integration branch is never ambiguous: its record is the `--repo` copy,
  and if `--repo` has none it is reported as "not in this checkout", not as an error.
- `effort.py status` never fails as a whole because one effort is ambiguous or unreadable; it prints that
  effort's row with the problem stated and continues.
- Add a test with many worktrees that all contain an older effort without an integration branch, plus one
  effort that lives only on its own branch, and run `status`, `rehydrate` (read-only part) and a ticket
  landing from the main checkout.
- Say in your report that this came from the lead's note and cite this file's name.
