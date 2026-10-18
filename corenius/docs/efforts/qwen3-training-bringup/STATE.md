# Qwen3 training bring-up state

Date: 2026-10-06 (America/Los_Angeles). Approved live milestone complete.

## Where we are

Both bounded workloads passed live execution and independent review. Tickets01–03
are resolved for the approved bring-up milestone. CPT completed20 updates on
Qwen3-1.7B/FineWeb across checkpoint10; GRPO completed2 initial and2 resumed
updates on Qwen3-0.6B/GSM8K with TorchTitan learner and SGLang serving.
Initial and resumed runs exited0. GPUs0/1 are released. Aggregate consumed:
1526.53090452 GPU-seconds (0.42403636 GPU-hours), including failed attempts.
See evidence/gpu-budget.md and both final review receipts.

## Next actions

Human review/integration of the uncommitted native changes remains. No further
GPU run is needed for this milestone. Any longer training campaign or broader
recovery/scaling qualification is separate work. TorchTitan Kata mkqk remains needs-review pending its commit gate; verl Kata
eb6b is closed done with evidence. No Git authorization was inferred.

## Claim limits

CPT proves state restoration and continued updates, not RNG restoration or
uninterrupted equivalence. RL qualifies a completed-epoch boundary; exhausted
saved loader state is skipped for the next epoch. No future-sample equivalence,
in-flight rollout recovery, or RNG proof. Sync evidence combines exported tensor
digests with real engine ACKs and subsequent generation; no independent serving
readback. RL trainer.ntokens_seen remains0 and is not token accounting. Four
validation samples do not establish quality gains. Fresh duplicate installation
via the final preparation script was not repeated.

## Do not redo

Read native RESULT/README, acceptance receipts and independent reviews before
further exploration. Do not repeat CPT/SFT or GRPO/PPO questions. The user chose
CPT and GRPO and approved the bounded runs. Arnold research remains in Ultron.

## Workspace and preservation

Corenius worktree: /data02/home/philip.yang/workspace/corenius-qwen3-training-bringup
Branch: codex/qwen3-training-bringup
Base: d7f674829e9844d011869b711d64fcf55313f7f1

Files remain uncommitted. Automatic effort dehydration was not invoked because
it creates a commit and no commit authorization was supplied. This is an explicit
human-readable checkpoint, not a claim that machine dehydration/rehydration ran.

## User-requested artifact cleanup

2026-10-07 UTC: Deleted only generated CPT and GRPO training checkpoint trees
after recording per-file hashes and byte counts in evidence/checkpoint-deletion.json.
Released55.2858505 GiB of allocated checkpoint files. Existing receipts and logs
remain; these historical checkpoints are no longer available for resume.
