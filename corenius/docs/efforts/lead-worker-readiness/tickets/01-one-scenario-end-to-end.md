# 01 one-scenario-end-to-end: One readiness scenario runs end to end in a lead-driven mode

Type: feature
Status: resolved
Owner: worker-01
Blocked by:
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (story numbers below refer to it). Decisions: `decisions/` beside it.

Stories 1, 2, 4, 44. This is the tracer bullet: the thinnest path through every layer, so later tickets add scenarios and not plumbing.

Build a lead-driven mode in the benchmark harness, as a new module beside the fleet runner (leave the fleet runner's behavior unchanged). It takes a readiness scenario, materializes a benchmark task into a disposable git repository, launches one worker through the agent toolbox's launcher with a scripted participant standing in for the model, waits for the worker's hand-off, and has an independent verifier judge the scenario from the repository (commits, refs, files) and from the run record. Scripted participants are small programs that take several steps and can misbehave on purpose; they are selected the way the toolbox tests select a fake worker binary.

Change the toolbox's hand-off rule to the decided one: a worker commits on its own branch. A worker's run is ready for review only when its process has exited with code 0, its tree is clean and its branch holds at least one commit on top of the recorded base. The verdict then records the Candidate: commit, tree, base commit and the digest of the brief it was launched with. A worker that exits with a dirty tree, or with no commit, is reported as not ready, never as ready. Moving any branch other than its own, rewriting history below its base, pushing or merging remain violations.

Deliver the hand-off scenario with two scripted participants: one that commits and exits (expected: ready, Candidate recorded and equal to the branch tip), one that announces completion in its final message but leaves the tree dirty (expected: not ready). The scenario result is one of: passes, passes with intervention, not applicable by design, fails.

## Declared paths
- `bench/harness/` (new lead-driven module, its tests, scenario definitions; do not change `fleet.py` behavior)
- `bench/readiness/` (new: scenario definitions and scripted participants)
- `tools/agents/worker.py`, `tools/agents/test_worker.py`
- `docs/practice/lead-supervisor-worker.md` (a dated note recording the changed hand-off rule), `.claude/skills/run-worker/SKILL.md`

## Acceptance
- [x] A command runs the hand-off scenario offline with scripted participants and prints one result per participant; it exits 0 only when both results match their expectation.
- [x] The good participant's run is reported ready, and the recorded Candidate (commit, tree, base, brief digest) equals what `git` shows for the worker branch tip in the disposable repository.
- [x] The participant that leaves a dirty tree, and a participant that exits without any commit, are each reported not ready; neither is ever reported ready.
- [x] A worker that moves a branch other than its own, or whose branch no longer contains its base, is reported as a violation.
- [x] The verifier reads the disposable repository with its git metadata and the run record only; a test shows it does not read the participant's own final message to decide.
- [x] A worker commit on its own branch is no longer a violation in the toolbox; every toolbox worker test still passes, with tests that asserted the old rule rewritten to assert the new one.
- [x] The fleet runner's tests still pass unchanged.
- [x] The practice note and the run-worker skill state the new hand-off rule and cite the decision file.

## Comments
- 2026-10-05T08:04:51Z worker-01: claimed until 2026-10-07T08:04:51Z
- 2026-10-05T09:05:46Z worker-01: resolved
