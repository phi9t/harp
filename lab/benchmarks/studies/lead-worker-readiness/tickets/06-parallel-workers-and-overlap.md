# 06 parallel-workers-and-overlap: Two workers run at once and overlap is reported

Type: feature
Status: resolved
Owner: worker-06
Blocked by: 05
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 3, 25, 26. Show two workers of one effort running at the same time, both handing over Candidates, both landing (the second after a redo on the moved base). While workers run, report: a change to a path outside the ticket's declared paths; a change to a path that another running ticket has declared; and any write into a worker's workspace that the worker did not make (the lead must not write there).

Amended by the lead 2026-10-05 from what the run has shown (read `evidence/lead/notes.md` and every `- to 06:` line the launch brief gives you):
- Declared paths did not keep parallel tickets apart in this run. Ten tickets landed; every one that added a scenario edited the shared lead-driven harness module and its test, which most had not declared, and each worker's report sits outside its declared paths by design. Overlap reporting must therefore separate three cases: a path both running tickets declared (reported at launch, expected); a path one ticket declared and another changed (reported while running); and a path nobody declared that two running tickets both changed (reported while running, the case that actually happened). The worker's own report file is never an overlap.
- The lead's landing order around a shared file was the most frequent decision of the run. Add a read-only command that lists, for the running tickets of an effort, which changed paths they have in common right now and which of them is behind the integration tip, so the lead can decide who lands first. It changes nothing.
- Probes of a worker's worktree must not take git locks (see the finding from ticket 08), and a write "not made by the worker" has to be judged without the lead's own read-only probes counting as writes.

## Declared paths
- `tools/agents/worker.py`, `tools/agents/lead_launch.py`, their tests
- `bench/readiness/` (scenarios with two scripted participants)

## Acceptance
- [x] A scenario runs two scripted workers whose running intervals overlap (shown from recorded start and end times) and both Candidates land; the second lands as a new Candidate on the moved base.
- [x] A worker that writes outside its declared paths produces a finding naming the path while it is still running.
- [x] Two running tickets that declare a common path are reported at launch; a worker writing a path declared by the other running ticket produces a finding naming both tickets.
- [x] A file changed in a worker's workspace by a process outside the worker's process tree is reported as a violation by the supervisor, or, where the process cannot be identified, as a change the worker's event record does not account for; the limits of this detection are stated in the tool's help.
- [x] All scenarios run in the lead-driven mode and match their expectation.

## Comments
- 2026-10-05T12:15:03Z worker-06: claimed until 2026-10-07T12:15:03Z
- 2026-10-05T14:27:45Z worker-06: resolved
