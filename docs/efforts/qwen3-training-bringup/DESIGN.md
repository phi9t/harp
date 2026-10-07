# Qwen3 workload bring-up: proposed design

Date: 2026-10-06. Status: approved by user; implementation and bounded live runs authorized.

## Settled intent

The user selected continued pretraining on real text and GRPO matching verl's
TorchTitan PPO-trainer example. The learner backend is TorchTitan and the rollout
backend is SGLang. Both workloads run inside Insula. These runs ground subsequent
Arnold research; adding sensors or recovery policy is not a prerequisite.

## Recommended first workloads

| Property | Continued pretraining | RL |
| --- | --- | --- |
| Owning repo | torchtitan | verl |
| Model initialization | Cached full Qwen3-1.7B HF weights, explicitly loaded | Pinned Qwen/Qwen3-0.6B, matching the example |
| Real data | Existing local FineWeb sample, content/provenance checked | Pinned GSM8K train/test, prepared by verl's dataset path |
| Objective | Causal next-token cross-entropy | GRPO with real answer-based rewards; no learned critic |
| Initial topology | Two B200s, DP shard 2, TP/CP/PP 1 | One B200, learner FSDP/TP/EP 1 and colocated SGLang TP1 |
| Work bound | 20 updates; save at 10; fresh-process resume to 20 | Two initial optimizer steps plus at least one post-resume update, n=4 responses per prompt, small prompt batches; bounded extension only to obtain a genuine update |
| Initial sequence bound | 512 tokens, local batch 1, global batch 2 | 512 prompt + 256 response tokens; truncation recorded |
| Key proof | Loaded pretrained state, real targets, full checkpoint, continued progress | Rollout → valid rewards → nonzero update → weight sync → subsequent rollout |

Different model sizes reuse available local assets and the published RL example.
They do not imply a CPT→RL checkpoint lineage. A shared 1.7B baseline is a useful
later comparison; direct transfer between the two runs is outside this first gate.
The truncated RL context is a bring-up constraint, not a quality configuration.

Approved limits: run workloads serially on at most two explicitly
available local B200s; maximum 15 minutes per GPU attempt and 1 aggregate GPU-hour
for initial qualification. This includes GPU preflights and failed attempts.
Recheck device use before launch and select free devices; visibility or low
utilization does not authorize displacing another job. Asset downloads and CPU
builds use declared writable build/cache paths, not existing runtime mutation.
Stop and report the exact failed gate rather than silently increasing limits.
The user approved these limits and directed live execution. Device availability
still must be checked before each launch.

## Alternatives considered

1. **Recommended: separate pinned Insula environments, native repo entrypoints.**
   TorchTitan CPT can proceed independently while the combined TorchTitan/SGLang
   dependency set is qualified. No requirement that the two environments share
   a torch version. Within the RL environment, both backends must be compatible.
2. **One environment and one model for both workloads.** Easier comparison, but
   it unnecessarily makes CPT depend on the harder RL dependency resolution.
3. **Disaggregated learner and serving environments from the start.** Can separate
   incompatible dependencies, but adds distributed weight transfer, services and
   recovery boundaries. Keep as an explicit fallback decision if a combined RL
   environment cannot be built; do not silently substitute vLLM or another learner.

## Gate 0: source and dependency admission

Preserve dirty work in both sibling repos. Build from isolated candidate source
with exact revision plus any deliberate patch digest. Do not copy untracked
rootfs helpers wholesale or overwrite existing rootfs trees.

Current verl's SGLang extras pin torch 2.11, while its TorchTitan adapter docs need
modern nightly APIs from torch >=2.14.dev20260625. The known-good documented
nightly set covers vLLM, not SGLang. Resolve a concrete package/source build set,
including native extensions, before installing a candidate dependency bundle.
An unconstrained pip override or passing `--no-deps` does not qualify ABI safety.
Use local owning TorchTitan source at a pinned compatible revision in the RL
bundle; if APIs differ, record and test the narrow adapter/version decision.

Insula preflight records actual interpreter and module paths, package/build
identities, CUDA/driver/device exposure, and relevant imports. It must execute
inside the selected environment. Qualify a real model forward/backward in
TorchTitan and SGLang generation/weight-load before the combined RL run.

Reuse established Insula plan/build mechanisms, with explicit code, data,
checkpoint, device and network projections. Existing verl rootfs metadata is not
sufficient: the audited helper mounts the base writable and derives store identity
from bin/bash alone. Prefer an existing qualified rootfs plus sealed dependency
bundle, or repair the necessary local helper under its owning ticket. Whole input
identity must cover the declared closure. No host Python/package fallback.

Downloads occur in an explicit preparation phase. Training uses staged assets
and a declared network policy sufficient for local Ray/NCCL/SGLang communication;
external network is not silently inherited. Runtime writes go to owned outputs
and caches. Insula isolation and hermetic input closure are recorded separately.

## Gate 1: genuine continued pretraining

Use core run_train.sh and the qwen3_1_7b registry, with explicit overrides rather
than the dirty synthetic 4B experiment. The default registry is not automatically
continued pretraining: set HF model loading deliberately, verify loaded tensors,
and do not accept a tokenizer-only load or random initialization.

Pin the cached tokenizer, model config and shard hashes. Verify the FineWeb file
content, source identity, eligible targets and loader path. Use a conservative
CPT learning rate (candidate 1e-5), two warmup steps, and a fixed 20-step scheduler
horizon; validate configuration against the actual registry/API.

Enable synchronous full-state DCP checkpointing and last_save_model_only=false.
Stop the first attempt at step 10 without shortening the 20-step LR schedule.
Resume from that exact DCP with HF-bootstrap disabled and full-state load enabled.
Record model, optimizer, scheduler, loader and trainer progress component coverage.
RNG restoration is unproven in the inspected path and must not be assumed.

Acceptance: successful ranks, actual optimizer updates, finite losses, valid-token
progress, full checkpoint receipt, correct restored step/scheduler/data position,
and further updates to step 20. Compare a short uninterrupted control where
feasible. Only claim numerical continuation equivalence if data identities and
next-update comparisons pass a declared tolerance; otherwise label the supported
resume behavior and missing state precisely.

## Gate 2: genuine GRPO training

Adapt tests/special_e2e/run_ppo_trainer_torchtitan.sh using explicit SGLang override.
Initially use eager flex attention, AC_MODE=none, no pipeline parallelism,
no disaggregation and no delta-sharded sync. The current example's eager/selective
activation-checkpoint defaults conflict with its own warning; validate the resolved
configuration, not just the script invocation.

Prepare GSM8K via the repo's native preprocessing and verify prompt formatting,
answer extraction and reward distribution on actual completions. Preserve train
and evaluation splits. Initial candidate: 8 prompts, 4 responses per prompt,
optimizer minibatch matched to the batch, microbatch 1. Record all-pass/all-fail
groups and truncation. If all advantages are zero, no meaningful RL update has
been proven. Use further ordinary training batches within the approved bound;
never change rewards or fabricate successful completions to satisfy acceptance.

Record rewards, group statistics, finite advantages, policy-gradient/gradient
metrics and parameter-change evidence. Verify TorchTitan HF-name export, SGLang
weight receipt and cache refresh with a new policy version, then generate again.
A changed sampled answer alone does not prove the new weights were used.

Declare a learner/scheduler horizon with room for post-resume updates and keep
that same horizon across attempts. Save an actor/optimizer checkpoint and restart
the driver from the intended boundary. Verify learner step, optimizer/scheduler, actor state, and repopulation
of the rollout engine. Then execute at least one further valid optimizer update,
synchronize that resumed policy into SGLang, and generate a subsequent rollout.
Restoring a checkpoint without this post-resume cycle does not pass. Inventory dataloader/RNG/driver state and describe whether
in-flight rollouts are discarded/reissued. A model-only restart is not full
experiment resume. Any unsupported resume component becomes an explicit fixing
slice or a stated unmet milestone, not a successful completion claim.

## Evidence and validation

Each owning repo supplies one documented entrypoint with preparation, preflight,
run and resume phases, plus a bounded run manifest and result receipt. Receipts
include resolved configs, source/environment/assets digests, run/attempt IDs,
selected devices, command, exit status, losses/rewards, checkpoints, weight sync
and precise failure boundaries. Large artifacts stay outside Git.

Tests exercise behavior: runtime-plan refusal on missing inputs, real dataset and
reward contracts, meaningful optimizer updates, weight-sync identity and actual
checkpoint continuation. Reuse native tests; avoid tests that merely assert CLI
strings match their implementation. Run GPU verification under Insula only.

Two-GPU RL FSDP promotion is a later milestone, outside this initial acceptance
gate and GPU-hour budget; it will require matched global batch/group sampling and
its own approved limits and acceptance. Four-GPU scaling, convergence, SFT, full PPO, actor
MoE/EP, controller integration, fault injection and performance tuning are later
work, not implied by successful bring-up.

## Ownership and implementation handoff

Corenius tracks this shared effort. TorchTitan owns its config/runner/evidence
changes; verl owns its backend environment, RL runner and local Kata work items.
No new training runtime belongs in Ultron. The first repo-specific tickets will
link this effort and declare exact source base, input contracts and verification.
Create/claim those only after design approval and independent critique, using each
repo's tracker rules. No code changes, GPU execution or source commits have
occurred during this design pass.

## Approval receipt

The user approved this design and explicitly said “proceed to the live runs.”
CPT, GRPO, model sizes, topology and run limits are approved. Dependency compatibility remains an implementation gate whose failure
can require a revised design, not an unasked algorithm/backend substitution.
