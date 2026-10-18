# 03 verl-grpo: Run and resume Qwen3 GRPO with TorchTitan and SGLang under Insula

Type: task
Status: resolved
Owner: codex
Blocked by: 01
Claimed:

## Acceptance
- [x] Compatible TorchTitan/SGLang dependency bundle qualified inside Insula.
- [x] Real GSM8K rollout and nonzero GRPO updates with verified weight sync.
- [x] Full supported checkpoint restored; further update and subsequent rollout verified.
- [x] Focused behavior checks and independent review complete.

## Comments
- 2026-10-06T23:51:54Z codex: claimed until 2026-10-07T23:51:54Z

## Verified result

Qualified Qwen3-0.6B/GSM8K under Insula with TorchTitan default SPMD,
torch2.13.0+cu132 and SGLang0.5.21. Two initial and two resumed GRPO updates
had nonzero advantages and gradients. Every update changed the exported
311-tensor policy digest and received actual serving session/bucket/flush ACKs,
followed by generation. The resumed step2 digest matched the saved digest.
All254 optimizer counters, scheduler and trainer advanced from2 to4.
All128 training and16 validation rewards replayed under the native scorer.
18 focused CPU tests and independent code/evidence review passed.

Review: ../evidence/rl-final-review.md. Owning native result evidence:
/data02/home/philip.yang/workspace/verl.worktrees/eb6b-qwen3-grpo/.runtime/acceptance.json

Resume is qualified at completed epoch boundary only. No RNG, exact future
sample sequence, numerical equivalence, independent serving tensor readback,
quality or scaling claim. Full fresh installation via final prepare script was
not repeated. RL trainer ntokens_seen is not valid token accounting.
RL total1230 GPU-seconds includes failed attempts. Native Kata eb6b was closed done with reviewed evidence; files remain uncommitted.
- 2026-10-07T00:54:00Z codex: resolved

Native result: /data02/home/philip.yang/workspace/verl.worktrees/eb6b-qwen3-grpo/examples/qwen3_grpo_insula/RESULT.md
