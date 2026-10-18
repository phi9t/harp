# 21 scenarios-that-can-fail: Every readiness scenario depends on the tool it judges, and results do not overstate

Type: bug
Status: resolved
Owner: worker-21
Blocked by:
Claimed:

## Description
Triage: ready-for-agent
Spec: `docs/efforts/lead-worker-readiness/SPEC.md` (stories 44 to 46, and the core set). Source: `evidence/independent-review-1.md`, findings 9, 12, 13, 15, 16, 17, 19, 20, 21, 23 (read each in full).

The owner judges "good enough" from the core-set line of the readiness report. The independent reviewer showed scenarios that pass with the tool under test removed, scenarios whose worker never ran, and result rows labelled live and Codex that came from a scripted binary. Fix the scenarios and the report so that line rests on observations. Do not change product behaviour here except where an item says so; if a corrected scenario then fails because the product is wrong, report it under `## For later tickets` and leave the scenario failing with the reason.

You may not run real model workers.

## Declared paths
- `bench/harness/lead_driven.py`, `bench/harness/results.py`, their tests, `bench/readiness/`
- `tools/agents/checks.d/`, `tools/agents/lead_launch.py` (finding 9 and the `wait` exit code only), `tools/agents/acceptance.py` (finding 23 only), `tools/agents/effort.py` (finding 23 only), `tools/agents/land.py` (the unused option only), their tests

## Acceptance
- [x] A mutation check exists and runs in the harness tests: for every scenario, replacing the tool it judges (the lander, the acceptance command, the launch command, the worker launcher, the tracker) with a stub that does nothing makes the scenario report `fails`. Findings 15 and 16 are the first two cases it catches.
- [x] Finding 16: a scenario whose worker process did not start, exited non-zero unexpectedly, or wrote to `stderr.log` an error the scenario did not expect reports `fails`; scripted participants are executable; no scenario commits or imports on the worker's behalf.
- [x] Finding 17: a result row's `proof` is `live` only when the run used a real runtime binary (decided by the harness, not by a flag); a run that errors writes a `fails` row; an empty record list is `fails`; a row lists only the stories whose participants actually ran, and the report shows which participants ran.
- [x] Finding 13: the wall-clock notice is proven by a worker that really outlives a short cap, not by editing the run record after launch; if the notice cannot occur because the wrapper kills the process at the cap, change the product so that the notice is written when the cap is reached and say so.
- [x] Finding 12: the access-profile scenario's participants perform the acts (a real loopback fetch, a real write under the declared add path, a real command longer than the old limit); the help and the launch record say for each runtime which parts of the profile are enforced, which are only declared, and which are only detected from command text.
- [x] Finding 9: `monitor-tickets` computes commits ahead and whether the branch contains the integration tip from the integration commit recorded at launch and the worker's own repository; its worker state never says ready when the supervisor's verdict is not ready.
- [x] Finding 19: a ticket counts as landed for launching its dependants only when a landing record or the integration branch's history shows its accepted Candidate; a ticket merely marked resolved does not unblock them. `tracker.py resolve` refuses when no accepted, landed Candidate exists unless forced, and a forced resolve is logged.
- [x] Finding 20: the repository gate's `checks.d` runs the worker, effort, lead-launch, lander and acceptance suites, lints every effort, and has an entry for the lead-driven harness tests.
- [x] Finding 21 and 23: `land.py --attempts` is removed or used; `lead_launch.py wait` exits non-zero on timeout; a failed check passed to `acceptance.py record` is stored and blocks acceptance unless waived, whether or not it was named required; `effort.py move-records` can be rerun after an interruption between its two commits.
- [x] The readiness report, run after all of the above, prints the core-set line for the lead-driven instantiation; the report and your hand-off say plainly which core stories now pass, which fail, and why.

## Comments
- 2026-10-05T17:14:22Z worker-21: claimed until 2026-10-07T17:14:22Z
- 2026-10-05T20:19:42Z worker-21: resolved
