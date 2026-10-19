> Historical discovery receipt, before the user selected continued pretraining.
> [The proposed design](../DESIGN.md) supersedes its optional-initialization
> recommendation: verified loading of pretrained HF weights is now required.
> Original findings are preserved below.

# Qwen3 TorchTitan readiness audit (read-only, 2026-10-06)

All relative source paths below are under `/data02/home/philip.yang/workspace/torchtitan`. No runtime probe, install, GPU launch, source edit, or privileged call was performed. Only this requested /tmp report was written.

## Recommended bounded real run

Use core `run_train.sh` -> `torchtitan/train.py` -> `torchtitan/trainer.py`, module `qwen3`, registry `qwen3_1_7b`, on two B200 GPUs with DP shard 2, TP/CP/PP 1, BF16, local batch 1, global batch 2, sequence 512. Use `--dataloader.dataset fineweb_test` and cached `./assets/hf/Qwen3-1.7B`. This gives an actual 1.7B pretrain-style next-token training workload on real local text without acquiring assets or modifying the dirty registry. Random initialization is the existing pretrain default; HF initial loading is an optional explicit continued-pretraining choice, not required.

Use 20 optimizer steps with fixed `lr_scheduler.total_steps=20`, checkpoint enabled, interval 10, full-state last checkpoint, synchronous checkpointing, deterministic seed 42 and TensorBoard. Attempt one stops at 10; attempt two restores step 10 and finishes at 20 with the same run ID and distinct attempt IDs. Use unique output locations, and explicitly point second attempt's initial_load_path at the full step-10 directory with initial_load_model_only false if outputs differ. For a recovery-equivalence claim also run an uninterrupted 20-step control with identical initialization/configuration and compare full-precision step-11..20 loss/grad-norm and final state. Mere successful restore proves resumability, not full recovery equivalence. Do not claim convergence or performance from this bounded workload.

B200 compatibility is supported by historical executions; current allocation, CUDA/driver/NCCL compatibility and free memory are NOT freshly verified. Two H100 80GB GPUs are a plausible alternate for this small configuration, conditional on the selected runtime's actual kernel support; do not assume the old B200 rootfs automatically supports another device.

## Available paths and assets

- `torchtitan/models/qwen3/config_registry.py`: core qwen3_0_6b and qwen3_1_7b configurations; both default to network C4 and checkpoint disabled. Explicit local dataset and checkpoint overrides are needed.
- `torchtitan/hf_datasets/text_datasets.py:49`: registered local non-streaming fineweb_test.
- `experiments/qwen3_fineweb_hsdp_tp/data/fineweb_test/data.json`: present, 9.7MB; documented as 2,000 FineWeb-edu sample-10BT documents. Hash/provenance validation not performed.
- `assets/hf/Qwen3-1.7B`: present tokenizer, config, index and two safetensors shards totaling about 3.8GB. Qwen3-0.6B assets are absent from assets/hf, so 1.7B is the smallest fully cached standard-size family here.
- `torchtitan/components/checkpoint.py`: enable defaults false; native DCP includes model, optimizer, LR scheduler, data loader and passed trainer state. `torchtitan/trainer.py:561` passes train_state; load at line 968. RNG recovery needs explicit inspection/verification rather than an assumed guarantee.
- `run_train.sh`: exports TORCHTITAN_RUN_ID/TORCHTITAN_ATTEMPT_ID; real mode launches torchrun. COMM_MODE fake_backend/local_tensor do not prove real distributed training.
- `torchtitan/observability/run_evidence.py` and Trainer run_evidence configuration: canonical run-attempt evidence already integrated.

## Runtime candidates

- `scripts/rootfs/rootfs`: existing legacy mutable rootfs. On-disk package metadata reports torch 2.13.0+cu132, datasets 4.7.0, tyro 1.0.15, spmd_types 0.2.1, tokenizers 0.22.2, safetensors 0.8.0, tensorboard 2.21.0. No root-level build_manifest.json or .torchtitan ownership marker found by scoped glob. This is not a proven admissible immutable Insula rootfs.
- `.venv-rootfs`: separate repo-local environment contains torch 2.14.0.dev20260805+cu130. Do not mix or silently select this versus the rootfs global environment.
- `scripts/rootfs/enter_rootfs.sh`: authoritative legacy wrapper supports --rootfs and --rootfs-store, offline default, repo bind at /workspace/torchtitan; privileged mode explicitly forbids training.
- `scripts/rootfs/runtime_env.sh`: runtime PATH prioritizes /project/venvs/b200-runtime; default state `.cache/torchtitan-rootfs/modded_nanogpt_b200`, HF cache `.cache/huggingface`. The observed default state's venvs directory is empty; executable selection may fall through to rootfs global /usr/local/bin. Verify actual resolved Python inside admitted runtime.
- `experiments/qwen3_fineweb_hsdp_tp/run.sh` is an old fixed-eight-GPU wrapper without rootfs re-entry. Invoke core run_train inside an admitted Insula/rootfs command; never directly execute this old wrapper on host.
- `/data02/home/philip.yang/workspace/trondheim/insula/rootfs` and sibling trondheim-evidence-aggregation path exist as source directories; not evidence of materialized runtime readiness.

## Prior real execution evidence

- `experiments/qwen3_fineweb_hsdp_tp/results/hsdp_tp_2x2x2.log`: eight-rank debug Qwen3 FineWeb training completed 10 logged steps, loss 8.06982 -> 3.83328. Command explicitly contains --checkpoint.no-enable, so this is NOT a checkpoint/resume receipt. `results/summary.md` summarizes other mesh runs.
- `experiments/qwen3_4b_synthetic/results/latest/run_evidence/09787a72-8a95-4223-a5be-dafac36181eb/8e6c931b-4a45-4538-b28c-c78b89cdbc58/`: all four process outcome files say succeeded, world_size 4, elapsed roughly 35,968 seconds, host n116-077-207. Manifest requests qwen3_4b_synthetic_sft, 10,000 steps, DP2 TP2, runtime torch 2.13.0+cu132, dirty source revision 843948e9f9df21984f026b1abf8f7da8617c0851. Checkpointing false and initial HF loading false. This is substantial prior execution evidence, but not a fresh run or resume proof; no scalar loss series was verified in this audit.
- `experiments/countdown_search_distill/reports/20260811T110113Z-rootfs-full-pilot.md`: reports real Qwen3-1.7B LoRA arms with step 31/62/93/94 checkpoints. Scoped current results search found no .metadata files: treat document as historical report, not proof those checkpoints remain available.

## Preserve existing work / remaining blockers

Working tree is dirty, including qwen3/config_registry.py, rootfs tests, PII checker, Falcon GPU receipts, and untracked experiments/qwen3_4b_synthetic. The synthetic registry uses model 4B with 1.7B tokenizer assets, random initialization and disabled checkpointing; do not relabel it pretrained SFT or reuse it unmodified for recovery proof.

No missing model implementation is evident for the bounded pretrain run. Remaining launch blockers are admission/identity of the selected Insula runtime, actual GPU allocation and health, runtime package/import/driver/NCCL preflight, dataset/checkpoint provenance hashes and a concrete bounded launch/resume/evidence protocol. Legacy rootfs availability alone does not resolve Insula authority. Preserve the existing runtime and dirty source; provision/select approved runtime through Insula rather than mutate this legacy tree on inference.
