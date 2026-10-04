# 06 — Assembled baseline and permanent support

Status: open
Blocked by: 01, 02, 03, 04, 05
Spec: ../SPEC.md

## Requirements and evidence

Run a real end-to-end two-worker task through Kata, private Git, verification, acceptance and landing. Qualify restart and restore behavior, isolation and live selected runtime separately from fixtures. Provide reproducible setup, operating/recovery docs and regression suite requiring no PostgreSQL/Temporal. Independently review and land relevant accepted work; preserve parked work. No complete claim from unit fixtures alone.

Claim an isolated worktree before implementation. Retain public behavioral red-green evidence, exact source identity and independent Standards/Spec review. No shared branch rewriting.

## Controller-first dedicated tmux trial

The owner requires the goal and executable evidence contract in ../CONTROLLED-TRIAL.md. Implement controller/state-machine and deterministic real integration tests first, then demonstrate one Codex lead and two Codex workers with controller/dashboard in a unique dedicated session. Outer harness drives only lead/operator input through tmux and judges structured controller evidence plus actual process/Git outcomes. Preserve this trial as the baseline regression/dogfood harness.
