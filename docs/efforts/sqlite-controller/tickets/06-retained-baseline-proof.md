# 06 retained-baseline-proof: Assembled baseline and permanent support

Type: task
Status: open
Owner:
Blocked by: 01, 02, 03, 04, 05
Claimed:
Spec: ../SPEC.md

## Requirements and evidence

Run a real end-to-end two-worker task through Kata, private Git, verification, acceptance and landing. Qualify restart and restore behavior, isolation and live selected runtime separately from fixtures. Provide reproducible setup, operating/recovery docs and regression suite requiring no PostgreSQL/Temporal. Independently review and land relevant accepted work; preserve parked work. No complete claim from unit fixtures alone.

Claim an isolated worktree before implementation. Retain public behavioral red-green evidence, exact source identity and independent Standards/Spec review. No shared branch rewriting.

## Controller-first dedicated tmux trial

The owner requires the goal and executable evidence contract in ../CONTROLLED-TRIAL.md. Implement controller/state-machine and deterministic real integration tests first, then demonstrate one Codex lead and two Codex workers with controller/dashboard in a unique dedicated session. Outer harness drives only lead/operator input through tmux and judges structured controller evidence plus actual process/Git outcomes. Preserve this trial as the baseline regression/dogfood harness.

## Acceptance

- [ ] Deterministic real SQLite/Git/Kata/runtime fixtures qualify the controller before any live Codex trial.
- [ ] The independent outer harness drives the lead with literal tmux input in a NEW dedicated session; one Codex lead and TWO Codex workers prove overlap, stale integration, recovery and isolation with structured and physical evidence.
- [ ] One-command harness, operating/recovery docs and regression suite require no PostgreSQL/Temporal; supported backup/restore refuses unreconciled effects.
- [ ] Independent assembled and branch reviews plus fresh exact-tip gates pass before authorized FF-only landing; the runnable baseline remains supported.

## Comments

- 2026-10-04T03:53:26Z codex-design: Adopted owner commits9d90222/d18e413/f775734; normalized tracker fields without changing required scope. Controller-first sequencing is binding; Matt route fallback uses the repository crosswalk.

- 2026-10-04T18:12:52Z codex-design: Owner832e769 monitoring refinement adopted. Qualify retained supervised recurring monitoring runner and frozen-corpus/recovery/duplicate live-backfill overlap/source rotation-truncation/compaction/retention/private-data/injected-authority negatives. Prove20-action/10-minute mid-turn checkpoints and no cap reset; no unattended claim before tested runner. See SPEC Owner832e769 and retained exact observer design. Existing status/ownership/dependencies unchanged.
