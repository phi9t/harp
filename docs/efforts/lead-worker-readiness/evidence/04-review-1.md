# Lead review 1 of ticket 04: three changes, and re-integrate in the same pass

Reviewed Candidate: e1b111f. The Candidate history with rounds, the successor pinned after a delivered
review instead of `candidate-moved`, the separate count for stale-base redos, the cap of three, the
detached resume with its own event stream, the fresh-worker fallback and the
`fixes-committed-waiting-for-re-integration` state are right and stay.

## 1. A terminal delivery is recorded as acknowledged when nothing acknowledged it (blocking)

`terminal_delivery_receipt` returns `"acknowledged": True` as soon as `tmux send-keys` exits 0 (and
unconditionally for an input file). That records that keystrokes were accepted by a terminal, not that a
worker received anything: the pane may be showing a dead shell, or the session may be mid-turn with the
text sitting unsent in its input box. The spec's rule is that an uncertain effect is never recorded as a
confirmed one, and the ticket says "the worker side acknowledges and the receipt is recorded".

Required: the receipt records `sent` (true when the transport accepted the message) and, separately,
`acknowledged` with what was observed and when. For the terminal transport the acknowledgement is an
observation made after sending, within a stated wait: a new worker event or a new commit on the worker
branch after the delivery time, or (for a plain terminal with no event stream) the findings file's path
appearing in the pane followed by activity. If nothing is observed in the wait, the receipt says
`acknowledged: false` with the reason, the delivery is reported as unconfirmed with a non-zero exit, and
the round is not counted until an acknowledgement is recorded (a second command, or the next supervise
cycle, may record it later). Tests: delivery to a target that swallows the text gives `sent: true,
acknowledged: false`; delivery followed by a worker event gives `acknowledged: true` with the event cited.

## 2. The rework commands must work from the main checkout (blocking)

Ticket 07, now landed, added one lookup for an effort's record across the repository's worktrees
(`resolve_effort_record` in `tools/agents/land.py`), because an effort's directory lives on its own branch
in its own worktree and not in the main checkout. `acceptance.py` (`record`, `review`, `request-rework`,
`summary`) and `lead_launch.py rework` still read the effort under `--repo` only. After the rebase, resolve
the effort through that shared lookup in every one of them, and add tests in the layout your brief did not
yet describe: mainline in the main checkout, the effort's integration branch in its own worktree holding
the effort directory, one worktree per worker, commands run from the main checkout. Add that layout to
your scenario's `one-round-accepted` case.

## 3. Re-integrate in the same pass

`effort/lead-worker-readiness` has moved: tickets 11 and 07 landed since your base. They changed
`tools/agents/lead_launch.py` (findings from three sources in the brief; the integration branch read from
the effort record), `tools/agents/land.py` (effort record lookup, integration branch as default target,
owner override), `tools/agents/tracker.py`, `tools/agents/effort.py` and the harness module. Rebase your
own branch onto the current tip first, resolve conflicts by intent, then do items 1 and 2 as new commits.

## 4. Small things

- Your two `- to 13:` lines are addressed to a ticket that has already landed. Address them to 14 (the live
  trial) and keep the text.
- Confirm in the report, with the test name, that the refused fourth review round sets the ticket's triage
  to `ready-for-human` with a comment (the ticket's acceptance item 4).

## Protocol

Rebase, then the fixes, then the whole gate (the worker, acceptance, lead-launch, lander, tracker and effort
suites; every bench harness suite; every scenario under `bench/readiness/`; `tracker.py lint`). Update the
report. Finish with a new final commit `ticket 04: ready for review` on a clean tree. Tickets 06 and 16
started a few minutes ago and will not land before you.
