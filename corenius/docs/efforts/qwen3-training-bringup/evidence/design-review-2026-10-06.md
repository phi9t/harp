# Independent workload design review

Date: 2026-10-06

Reviewed `docs/efforts/qwen3-training-bringup/DESIGN.md` and both readiness receipts in `/data02/home/philip.yang/workspace/corenius-qwen3-training-bringup`, with a narrow read of local verl entrypoint/checkpoint code. Follow-up verification re-read only changed design passages and the TorchTitan receipt header.

Result: no remaining blocking findings in this bounded design review.

Original findings and verified resolutions:

1. RL restart acceptance previously stopped at restored state and rollout-engine repopulation. The revised workload table and Gate 2 require at least one post-resume optimizer update, SGLang synchronization, and a subsequent rollout. The declared learner/scheduler horizon must remain unchanged across attempts.
2. Two-GPU RL promotion had ambiguous completion/budget scope. It is now explicitly a later milestone outside initial acceptance and the initial GPU-hour budget, requiring its own limits and acceptance.
3. The historical TorchTitan readiness receipt allowed random initialization, conflicting with the subsequently selected CPT objective. A supersession header now explicitly requires verified pretrained HF loading under the proposed design while preserving historical findings.

The scheme supports the stated initial CPT and GRPO milestones and explicitly preserves existing dirty sources and runtimes. Separate CPT/RL environments and the unresolved combined TorchTitan/SGLang dependency gate are appropriately distinguished. No false numerical-equivalence, RNG-restoration, or runtime-readiness claim was identified.

Limits: this is design acceptance review, not a full source audit or runtime qualification. Dependency compatibility, asset provenance, Insula runtime closure, actual GPU availability, meaningful RL updates, and checkpoint continuation remain implementation gates. No repository edits, builds, installs, or GPU execution were performed by this reviewer; only this requested receipt was written.
