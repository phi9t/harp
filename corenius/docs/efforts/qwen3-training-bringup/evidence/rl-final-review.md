# Independent final GRPO review

Date: 2026-10-07

Decision: approve the bounded single-GPU GRPO and completed-epoch checkpoint continuation milestone. No remaining critical or important issue found in the reviewed code path and live evidence. This is not approval of numerical recovery equivalence, general interruption recovery, scaling, or quality improvement.

Reviewed checkout: `/data02/home/philip.yang/workspace/verl.worktrees/eb6b-qwen3-grpo`.

## Reviewed implementation

Read the native Insula preparation/launch/training scripts and asset identity code, TorchTitan engine checkpoint/horizon changes, SGLang schema/bootstrap compatibility changes, real HTTP weight-update session handling, exported-tensor evidence, and focused tests. Inspected existing evidence only; this reviewer ran no GPU workloads, framework tests, builds, installs, or large checkpoint scans.

Material findings were resolved before successful qualification:

- Per-attempt limits now reserve termination time: preflight 380+20 seconds, training/resume 680+20 seconds.
- Runtime source exclusions have matching masks for verl's .runtime and .git; projected runtime/toolkit/driver/assets are admitted through the recorded identity manifest.
- Explicit verl saves bypass TorchTitan's independent interval, force full-state save, synchronize trainer.step, and fail when native save declines. Loads reject false returns and unexpected restored steps.
- The configured four-step horizon reaches native TrainingConfig and its scheduler fallback. Initial epoch exhaustion stops at two updates; resumed epoch completes steps three and four.
- Current SGLang msgspec arguments and SchedulerInitResult are handled using the installed contracts. Required begin/end weight-update calls are real HTTP operations with checked acknowledgements, wrapping the full ordinary tensor stream. Cache flush success is checked from the installed server's actual reply. Failed transfers cannot publish successful sync evidence. The out-of-scope delta path was preserved.
- CPU checkpoint tests now perturb optimizer step/moments, model, scheduler, and trainer before restoration. Existing final18.log reports 18 tests passed, including protocol success/failure tests.

## Evidence inspected

- `.runtime/run/preflight.json`: strict pretrained TorchTitan load, finite forward/backward, nonzero gradient and parameter change; SGLang generation and disk weight reload succeeded on torch2.13.0+cu132/SGLang0.5.21 with default TorchTitan SPMD.
- `.runtime/prep/dataset-contract.json`: all 7,473 train and 1,319 test prepared rows match pinned raw prompt/reward fields. Pinned model/raw-data digests and native preprocessing are documented.
- `.runtime/run/train-attempt4.log` and `resume.log`, with exit receipts: initial steps 1–2 and resumed steps 3–4 succeeded. Gradient norms are approximately 1.8698, 2.4124, 1.7524, and 1.7191. Each step has positive and negative finite advantage extrema, genuine mixed-outcome GRPO groups, and 32 recorded completions. Four validation completions follow each update/sync.
- `.runtime/run/sync/`: six receipts at steps [0,1,2,2,3,4], each covering 311 exported tensors with genuine begin, tensor-load, end, and cache-flush acknowledgements. Hashes change on every optimizer update. The resumed step-2 export digest exactly matches the first attempt's step-2 digest: `e474521c95abb433b132b321b76b2b73ac5920b97d5b3a4af4a3f7daaea78ab0`.
- `.runtime/prep/checkpoint2.json` and `checkpoint4.json`, plus the scalar-only native DCP reader source: each checkpoint has 4,124 metadata entries; all 254 optimizer step tensors, scheduler.last_epoch, and trainer.step advance from 2 to 4. This complements the matched resumed policy digest and native full-state load path.
- `.runtime/prep/reward-replay-final.json`: all 128 training and 16 validation rewards reproduce under the native scorer.
- `.runtime/postcheck-identity-v9.log`: final recorded runtime identity verification passed. `.runtime/acceptance.json` consolidates outcomes, source diff identity, sync, checkpoint, reward, and budget evidence.

## Limits that must travel with the result

Weight synchronization proves exported tensor bytes plus successful serving-engine protocol acknowledgements and subsequent generation. It is not independent readback of serving-resident tensors.

Resume is qualified at a completed epoch boundary. The native log explicitly skips the exhausted saved dataloader state at step 2 and starts the next epoch. Actor/optimizer/scheduler/trainer continuation is supported; RNG restoration, uninterrupted-control equivalence, exact future sample-sequence equivalence, and in-flight rollout recovery were not established. Trainer ntokens_seen remains zero in the RL adapter and must not be presented as valid RL token accounting.

Four validation examples do not establish quality improvement; observed accuracy is 0.25, 0.25, 0, 0. Only default-SPMD, TP1, ordinary full-weight GRPO is qualified. Other SPMD modes, LoRA, delta transfer, and multi-GPU configurations remain unqualified.

Reproducibility artifacts include the package lock, pinned model/data revisions and digests, preparation scripts, declared source/runtime identity, native entrypoints, logs and large local checkpoints. Existing-bundle checks and original CPU preparation ran; a fresh duplicate installation through the final preparation script was not executed. Large artifacts remain local and ignored by Git.

The coordinator records 1,230 GPU-seconds for RL, including failed attempts, and 1,526.53090452 GPU-seconds for CPT plus RL. One failed preflight envelope is reconstructed from recorded timestamps; accounting is wall-envelope based, not fine-grained kernel time. Allocation and free-device snapshots were coordinator-owned. No GPU time was consumed by this review.
