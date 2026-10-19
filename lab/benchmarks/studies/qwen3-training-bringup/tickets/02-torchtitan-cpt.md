# 02 torchtitan-cpt: Run and resume real Qwen3 continued pretraining under Insula

Type: task
Status: resolved
Owner: codex
Blocked by: 01
Claimed:

## Acceptance
- [x] Qualified Insula runtime with pinned source/model/data and explicit HF initialization.
- [x] Real CPT reaches 10 updates and writes full checkpoint.
- [x] Fresh process restores and continues through update 20 with evidence.
- [x] Focused behavior checks and independent review complete.

## Comments
- 2026-10-06T23:51:54Z codex: claimed until 2026-10-07T23:51:54Z

## Result

Real CPT milestone passed in isolated TorchTitan worktree, Kata mkqk.20 updates,
20480 global tokens; fresh process restored step10 and continued through20.
47 focused checks passed; independent final review accepted bounded claims.
Native verified result:
/data02/home/philip.yang/workspace/torchtitan/.worktrees/qwen3-cpt-insula/experiments/qwen3_cpt_insula/results/live/verified-result.json

Numerical equivalence/RNG recovery/convergence are not claimed. No commit was
created; owning Kata close remains governed by its commit-evidence requirement.
- 2026-10-07T00:05:38Z codex: resolved
