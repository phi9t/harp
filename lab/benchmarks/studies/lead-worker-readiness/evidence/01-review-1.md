# Lead review 1 of ticket 01: changes required before it can land

Reviewed Candidate: c153084 (tree 02c5439). I ran your suites on a separate checkout of that exact commit:
worker tests 27 OK, lead-driven tests 5 OK, fleet tests 9 OK, both scenario commands exit 0. Six sampled
claims from your report held. The toolbox rule change, the Candidate record and the practice-note update
are good and stay. Four things must change.

## 1. The scenario never judges the toolbox's own verdict (blocking)

`launch_and_supervise` sets the outcome from `verification["ready_for_review"] == expected` only. The
toolbox's verdict (`toolbox_verdict`, with its `ready_for_review` and `candidate`) is recorded but never
compared with anything. So if the toolbox reported a dirty run as ready, or recorded a wrong Candidate,
every scenario would still print `passes`. The instantiation under test is the toolbox; the verifier's job
is to check what the toolbox claimed against what git shows.

Required: a scenario passes only when (a) the toolbox's `ready_for_review` equals the expectation, (b) the
independent verifier's observation agrees with the toolbox, and (c) for a ready run the toolbox's recorded
Candidate (commit, tree, base, brief digest) equals what the verifier reads from git and the launch record.
A disagreement is `fails` with the differing field named. Add a test that feeds a doctored toolbox verdict
(ready on a dirty tree; a Candidate with the wrong commit) and shows the scenario fails. Acceptance item 2
of the ticket is about the toolbox's recorded Candidate, and the current test at
`bench/harness/test_lead_driven.py:50` checks only the verifier's own.

## 2. The hand-off rule only fits a worker process that exits (blocking; this is a correction to the ticket, from the owner's direction after it was written)

The first level the owner wants runs workers as interactive sessions that do not exit: you are one. Your
own run shows the gap. You signalled ready at 08:41:19 with commit 88baa03, then reset and recommitted, and
the branch tip became c153084 at 08:42:05 (same tree, different commits). A lead that acted on the first
signal would have reviewed a commit that no longer exists on the branch.

Required, in the toolbox and in the scenario:
- A second, explicit hand-off mode chosen at launch, for sessions that do not exit: the run is ready when the
  branch tip is a commit whose subject equals a marker declared at launch (for example
  `ticket 01: ready for review`), the tree is clean, and the branch has at least one commit on the base. The
  process-exit mode stays as it is and stays the default.
- In both modes the Candidate is pinned when readiness is first observed. If the branch tip later differs
  from the pinned Candidate, `status` (and a later `supervise`) report `candidate-moved` with both commits,
  and the earlier Candidate stays in the record; it is not silently replaced.
- New scripted participants: one that makes the marker commit and stays alive (expected: ready, Candidate
  pinned, without waiting for exit); one that makes the marker commit with a dirty tree (expected: not
  ready); one that becomes ready and then rewrites its tip (expected: ready, then `candidate-moved` reported).
- Update the practice note and the skill text to describe both modes.

## 3. The scenario does not use the benchmark's task library (should fix)

The ticket says the mode "materializes a benchmark task into a disposable git repository", and the spec's
seam is the benchmark's task library, materialization and verifier. `lead_driven.py` copies a private
`environment/` directory and never touches `tasklib`. Required: a scenario may name a benchmark task id;
when it does, the repository is materialized through the task library's existing function, so later
scenarios can use real multi-ticket tasks. Keep the tiny private environment as the other option, and make
the hand-off scenario run once against one small existing benchmark task (materialization only; no capsule
and no task verifier run are needed for this ticket). If the task library cannot do that without a
capsule, stop and report exactly what blocks it instead of working around it.

## 4. Small things

- The verifier's foreign-branch check misses a branch created during the run; the toolbox catches it
  (`<new>`). Make the verifier see it too and add the participant.
- Two report claims cite the line before the statement they describe (`worker.py:33` is the tuple's opening
  line; `:466` is the `if`, the append is the next line). Cite the line that shows the claim.
- Your report says `check.sh` runs cargo despite `CHECK_CARGO=0`. Confirmed: `checks.d/70-sqlite-controller.sh`
  calls cargo unconditionally. That instruction in your brief was the lead's mistake. Do not run
  `check.sh`; the gate for this ticket is the worker suite, the three bench harness suites you ran, both
  scenario commands and `tracker.py lint`.

## Protocol

- Do not rewrite or reset commits you have already handed over. Add new commits on top of c153084.
- Update the report (acceptance evidence, claims, anything outside declared paths) for what you change.
- Finish with a new final commit whose subject is exactly `ticket 01: ready for review`, on a clean tree,
  and then make no further commits or amendments. If item 3 or anything else cannot be done as stated,
  commit with `ticket 01: needs-info` and the exact reason in the report.
