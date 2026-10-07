# Qwen3 verl / TorchTitan / SGLang readiness (read-only, 2026-10-06)

## Source identity and limits

Owning checkout `/data02/home/philip.yang/workspace/verl`, HEAD `3f395aef288b59921923453b131e4e42d7672ed2` (`[example] feat: single-file from-scratch PPO for a tiny LM`), branch main. Existing dirty `ppo_single_file.py`; untracked `.trae/` and `scripts/rootfs/`. No repository edits, installs, GPU execution, production calls, builds, environment imports, or network package resolves performed. Read AGENTS.md, CONSTITUTION.md, workflow/commands; CONTEXT.md absent in this checkout. Kata quickstart read; searches found no TorchTitan issue. Report is an inspection, not runtime qualification.

The user-linked https://verl.readthedocs.io/en/latest/workers/torchtitan_workers.html#ppo-example was fetched and agrees with the checked-in doc on the algorithm, nightly requirements, and example. Local source is the authority for the observations below.

## Smallest honest target

Real Qwen/Qwen3-0.6B GRPO on prepared GSM8K parquet, launched by `verl.trainer.main_ppo` with `model_engine=torchtitan`, SGLang rollout, one GPU, all parallel degrees one, two short optimizer steps, then explicit checkpoint/restart. This preserves the documented algorithm and real model/backend while reducing resource and failure surface. A later four-GPU FSDP run is closer to the displayed docs command.

`tests/special_e2e/run_ppo_trainer_torchtitan.sh` is **GRPO via the PPO driver**, not full PPO/GAE: `algorithm.adv_estimator=grpo`; script defaults rollout.name=vllm, train batch 256, minibatch 64, microbatch 4, rollout.n=5, length 512+256, 100 steps. It accepts trailing overrides, so explicit `actor_rollout_ref.rollout.name=sglang`, train batch 4 or 8, minibatch matched to it, microbatch 1, rollout.n=2, two steps and save frequency one are reasonable bounded candidate settings, not validated commands. Preserve enough group samples for GRPO (n >= 2), sample/reward inspection, and nonzero observed gradients/parameter changes; a zero-reward/zero-advantage smoke does not prove a meaningful policy update.

Set `AC_MODE=none` for the initial eager `spmd_types` candidate: script explicitly warns eager selective AC recompute leaves the SPMD mesh context and crashes, yet defaults AC_MODE=selective with compile=False. This inconsistency needs resolution before simply executing the documented defaults. `ATTN_TYPE=flex` avoids an additional external attention dependency; throughput may be poor in eager mode.

Full PPO is a separate feature/qualification target. `verl/trainer/ppo/utils.py:96` disables critic unless GAE or explicitly enabled; critic YAML fields in the script do not prove a running critic. `verl/workers/engine/torchtitan/transformer_impl.py:717` registers only `language_model`; no TorchTitan `value_model` registry implementation was found. Switching adv_estimator to gae alone is not an established supported path.

## Present runtime seams

- TorchTitan adapter imports `Trainer`, modern `ParallelismConfig`, optimizer configs, and AC classes; imports at `transformer_impl.py:29-40` impose a concrete API floor.
- HF checkpoint bootstrap exists through trainer/checkpointer construction and `self.checkpointer.load()` around lines 280-282.
- Full weight export exists: `_to_hf_named_params` around line 536 converts adapter names and restores tied lm_head; `get_per_tensor_param` around line 633 gathers DTensors. Pipeline parallelism is unsupported.
- `verl/workers/rollout/sglang_rollout/sglang_rollout.py:304` implements async update_weights. Its ordinary hybrid path exhausts the generator on all ranks (necessary for DTensor all_gather), buckets tensors, calls SGLang's weight-sync utility, flushes cache, and updates global-step identity. Delta-sharded/disaggregated machinery exists too but is unnecessary complexity for the first run.
- TorchTitan checkpoint save/load at `transformer_impl.py:479-529` delegates to its checkpointer, overrides parent folder, and extracts global_step_N on reload. Presence is not proof of verl trainer resume correctness: validate actual output paths, saved optimizer/scheduler/RNG/loader state, step advancement, and restored actor weights with an interrupted/resumed run.

## Blocking compatibility mismatch

`pyproject.toml:103` SGLang extra pins torch==2.11.0, torchvision==0.26.0, torchaudio==2.11.0, sglang==0.5.12, transformers==5.3.0; comments identify native kernels as torch-2.11/cu130 builds. No torchtitan extra was found.

`docs/workers/torchtitan_workers.rst:19-38` instead requires recent TorchTitan plus torch >=2.14.0.dev20260625 for spmd_types. Its known-good set is torch 2.14.0.dev20260625+cu130, torchvision 0.29.0.dev20260626+cu130, torchtitan 0.1.0.dev20260701+cu130 and **vLLM** 1.0.0.dev20260620+cu130. Those pins do not establish a SGLang-compatible build. `uv sync --frozen --extra sglang` alone therefore cannot reproduce the advertised TorchTitan environment. Do not force-install the nightly over ABI-bound native dependencies and call it qualified. Need one explicit immutable dependency solution and import/ABI evidence for TorchTitan+SGLang, or a deliberately designed separate-runtime/disaggregated boundary (larger scope). A different SPMD setting does not by itself prove old torch supports the adapter's modern API imports.

## Rootfs inventory and provenance limits

`/data02/home/philip.yang/workspace/verl/scripts/rootfs/rootfs` exists, with owner marker verl-rootfs and build_manifest.json. Manifest declares CUDA 13.0.0 cuDNN Ubuntu 22.04; store_id `rootfs-59474588a312b6b6e73e5a42a59bf71e62b55416b6c9d5e4a6e1c630c2a9ecd4`; immutable false flag (`mutable_rootfs_allowed=false`). This is a candidate base only: opt contains nvidia, not a prequalified TorchTitan/SGLang environment.

Untracked builder `scripts/rootfs/build_rootfs.sh:31-33,256` deliberately bakes base/system/uv only and resolves Python deps at launch from uv.lock. Current builder around line 308 derives store_id from sha256 of bin/bash, so that identifier does not hash the complete rootfs. Manifest image_id is the mutable `verl-rootfs:local` tag, not an immutable OCI digest. Launcher `scripts/rootfs/enter_rootfs.sh:304` uses `--bind ROOTFS /` (writable), even when manifest/store mode sets mutable_rootfs_allowed=false. It defaults networked (line 294), projects repository writable, and directs Python/cache state to `/project`. Do not promote its manifest claim into a verified immutable Insula execution contract.

Existing root `.agentic` files are September CPU/agentic check logs. Worktree `.trae/worktrees/h2rq-sc-insula-harness` has score-centering CPU sweeps/invariance/replay receipts, including rootfs-emulated names; these do not demonstrate Qwen3 training, TorchTitan, or SGLang. `kata search rootfs` found closed h2rq plus open review findings (yabd, kn8y, cjy0, 51vp, 5d3e) and ky1d dry-run manifests. No qualifying combined-backend receipt identified in this bounded scan.

Default `${HOME}/models/Qwen/Qwen3-0.6B` and `${HOME}/data/gsm8k` were not found at the host home path inspected. This does not exclude assets in alternate model stores. Model/tokenizer revision and parquet content digests must be resolved and staged explicitly, with Insula-visible paths instead of host defaults.

## Gates before any GPU attempt

1. Freeze dedicated clean source/worktree and exact dependency environment compatible with both backends; validate modern TorchTitan imports and SGLang native ABI inside the intended rootfs.
2. Reuse an Insula-qualified image/rootfs with actual whole-content identity, declared mounts/devices/network/write surfaces; verl's existing rootfs alone supplies neither proof nor authorization.
3. Stage exact Qwen3 model/tokenizer and real GSM8K parquet; resolve reward function and prompt formatting, finite group advantages, and run bounds.
4. Compose config and validate topology without starting workers; override SGLang explicitly, disable problematic eager AC, and bound total updates.
5. Prove first rollout -> rewards -> TorchTitan backward/optimizer update -> SGLang weight sync -> second rollout; record digests/version IDs and update metrics.
6. Save/restart checkpoint and verify resumed optimizer step and policy identity. Treat quality improvement as unproven at this tiny budget.
