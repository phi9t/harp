# 05 launch-by-edges-and-triage: Only unblocked, agent-triaged tickets are launched, from the right base, with a pinned brief

Type: feature
Status: resolved
Owner: worker-05
Blocked by: 01
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 18 to 24. Make launching a ticket a single lead command that refuses what must not run: a ticket with an unresolved blocker; a ticket whose blocker entry cannot be parsed; a ticket whose `Triage:` is not `ready-for-agent`; a ticket beyond the declared cap on parallel workers.

The tracker learns to read the `Triage:` line the tracker policy describes and to print the launchable set. The worker's workspace starts from the tip of the effort's integration branch, which holds its blockers' landed work. The brief is assembled from exact revisions of the ticket, the spec and the standing worker instructions, and the launch record holds the commit and digest of each, so a later change to any of them is visible. When a worker stops with a question, the ticket's triage becomes `needs-info` with the question recorded.

## Declared paths
- `tools/agents/tracker.py`, `tools/agents/test_tracker.py`
- `tools/agents/` (a new lead launch command and its tests), `tools/agents/checks.d/`
- `docs/efforts/agentic-engineering/milestone-b/contracts/ticket-format.md` (the `Triage:` line)
- `bench/readiness/` (scenarios)

## Acceptance
- [x] The tracker prints the launchable set: status open, every blocker resolved, triage `ready-for-agent`; ordered by id; with a JSON form.
- [x] Launch is refused, with a distinct message each, for: an unresolved blocker, an unparseable blocker entry, triage other than `ready-for-agent` (including a missing `Triage:` line), and a launch that would exceed the parallel cap.
- [x] A launched worker's branch starts at the current tip of the named integration branch, not at the mainline.
- [x] The launch record holds the commit and content digest of the ticket, the spec and the worker instructions that formed the brief; a test changes the instructions after launch and shows the record still identifies the launched version.
- [x] A worker question moves the ticket's triage to `needs-info` and appends the question to its comments; lint accepts the result.
- [x] Every existing ticket file in the repository still lints clean.
- [x] All scenarios run in the lead-driven mode and match their expectation.

## Comments
- 2026-10-05T09:26:13Z worker-05: claimed until 2026-10-07T09:26:13Z
- 2026-10-05T11:00:19Z worker-05: resolved
