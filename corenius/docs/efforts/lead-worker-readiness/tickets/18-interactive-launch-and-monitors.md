# 18 interactive-launch-and-monitors: The real tools launch interactive workers in tmux and replace the interim coordinator

Type: feature
Status: resolved
Owner: worker-18
Blocked by: 06, 10, 17
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (stories 32, 34, 37, 38 and the hand-off stories). Decisions: `decisions/` beside it.

The owner runs workers as interactive sessions he can watch and type into: one tmux window per worker in a named session, plus a ticket monitor and a worker monitor. Through this effort the lead did that with an interim script, `evidence/lead/ws.py.txt` (about 500 lines, no tests); `evidence/lead/notes.md` and `evidence/lead/lead-log.md` say why each part exists. Ticket 10 was amended to absorb it, but the amendment was written into its description and not into its acceptance items, and it was not built. This ticket is that work, with acceptance items.

Build on what has landed: the marker hand-off and Candidate pinning (01), launch by edges and triage with a pinned brief (05), the access profile (08), findings in briefs (11), runtimes (12), rework deliveries with sent and acknowledged (04), overlap (06), accounting and wait (10), the record branch (16) and the worker workspace kind from ticket 17.

## Declared paths
- `tools/agents/lead_launch.py`, `tools/agents/worker.py`, a new monitor module if you want one, their tests
- `bench/harness/lead_driven.py`, `bench/readiness/` (scenarios)
- `.claude/skills/run-worker/SKILL.md`, `docs/practice/lead-supervisor-worker.md`

## Acceptance
- [x] `lead_launch.py launch` has an interactive mode: it starts the chosen runtime's interactive command (for traecli: `traecli -m <model> -c model_reasoning_effort=<effort> -c model_reasoning_summary=detailed --yolo "<brief>"`) in the worker's workspace, in its own window of a named tmux session, with the marker hand-off, and writes the same run record as a non-interactive launch plus the tmux target. The window stays open after the runtime exits. Without tmux the interactive mode is refused with a clear message; the non-interactive mode is unaffected.
- [x] The brief of an interactive launch carries the standing worker rules the interim script's brief carries (read order, boundaries, the gate, the report sections, the marker commit, no rewriting after the marker, hold-the-marker on request, re-integration on request, the multi-checkout layout), generated from one template that both modes share.
- [x] Once the runtime's session id is known, the run record holds the path of the runtime's own session transcript; a test with a fake runtime shows it, and the help says for which runtimes this is observed.
- [x] A ticket monitor, usable on an interval in a tmux window and as one-shot JSON, prints one row per ticket: worker state, commits ahead of the integration tip, whether the branch contains the tip, dirty count, age of the last commit, what it waits for, whether the tip was rewritten since it was last pinned or reviewed, the count of changed paths outside the ticket's declared paths, and the size of the diff since the last reviewed Candidate.
- [x] A worker monitor, same two forms, prints per running worker: process and uptime, current activity and context remaining where the runtime's pane shows them, disk used by its workspace, and the last delivery with its sent and acknowledged state.
- [x] Worker states are derived only from git, the process table, the pane and the run record, and are exactly: running; finishing (marker made, session busy); ready (marker, clean tree, session idle, tip not already reviewed); rework (a review was delivered, no new marker since); waiting for re-integration; needs-info; stopped; landed. A test drives a scripted interactive participant through every state.
- [x] `wait` returns when any named ticket reaches ready, needs-info, stopped or waiting for re-integration, prints which and why, and never returns on finishing or rework.
- [x] A command opens the effort's session: the two monitor windows, and nothing else.
- [x] All of it works for an effort whose records are on a record branch, run from the main checkout, in a repository with a mainline checkout, an integration worktree, the record worktree and several worker workspaces; a scenario shows launch, a rework delivery, a held marker, a re-integration and a landing in that layout with scripted interactive participants.
- [x] The run-worker skill and the practice note describe the interactive mode and the monitors, and say the interim script is retired.

## Comments
- 2026-10-05T15:58:17Z worker-18: claimed until 2026-10-07T15:58:17Z
- 2026-10-05T22:34:43Z worker-18: resolved
