# 04 rework-round: A review can send a Candidate back for rework

Type: feature
Status: resolved
Owner: worker-04
Blocked by: 03
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 12, 13 (live part noted, not proven here), 14, 15, 16. Add the rework path for the lead-driven instantiation: the lead writes review findings to a file, delivers them to the worker and gets a receipt; the worker's session is resumed (not started fresh) when it is still available; the worker commits and hands over a new Candidate; the earlier Candidate and its review stay on record; the new Candidate needs its own acceptance record before it can land.

Review-driven rounds are capped at three per ticket. A fourth request is refused and the ticket is marked for the owner. Re-doing a Candidate because the base moved (ticket 02) does not count against the cap.

Scenarios: one rework round ending in acceptance; reworked Candidate landed without a second review (refused); three rounds then a fourth (refused, ticket marked for the owner); stale-base redo not counted.

Amended by the lead 2026-10-05 from what the run has shown (read `evidence/lead/notes.md` and every `- to 04:` line under `evidence/`):
- There are two kinds of worker session and rework must serve both. A run started non-interactively is resumed with `worker.py resume` (ticket 12); that resume must be started detached and supervised like a launch (wall-clock cap, quiet time, hand-off detection), not run in the foreground. A run that is an interactive session in a terminal (marker hand-off) receives the findings as one typed line that points at the findings file; the tool does the typing and writes the receipt.
- A request to re-integrate onto a moved base is delivered the same way but is not a review round and does not count against the cap of three.
- After a review is delivered, the pinned Candidate is superseded only by a new marker commit made after the delivery; that successor is recorded as round N. A tip change with no delivered review stays the violation `candidate-moved`.
- The lead may tell a worker to commit its fixes and hold the marker commit until told to re-integrate. The record must show that state as "fixes committed, waiting for re-integration", distinct from ready and from running.
- A worker's report is extended each round, never rewritten; findings for later tickets are never dropped.

## Declared paths
- `tools/agents/worker.py` (deliver findings, resume), `tools/agents/acceptance.py`, `tools/agents/lead_launch.py`, their tests
- `bench/readiness/` (scenarios and a scripted participant that fixes its work when resumed with findings)

## Acceptance
- [x] Delivering findings records the findings file's digest, the time and the worker run it was delivered to; the worker side acknowledges and the receipt is recorded.
- [x] A resumed scripted participant receives the findings and produces a second commit; the run record lists both Candidates in order with the review that rejected the first.
- [x] The second Candidate cannot land on the first one's acceptance record.
- [x] A fourth review-driven round is refused and the ticket's triage becomes `ready-for-human` with a comment; a stale-base redo leaves the round count unchanged.
- [x] When the worker's session can no longer be resumed, the tool says so and starts a fresh worker with the findings and the earlier Candidate in its brief; the record says which happened.
- [x] All scenarios run in the lead-driven mode and match their expectation.

## Comments
- 2026-10-05T11:36:02Z worker-04: claimed until 2026-10-07T11:36:02Z
- 2026-10-05T12:59:58Z worker-04: resolved
