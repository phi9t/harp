# Qwen3 training bring-up implementation specification

Status: user approved. Acceptance authority: [DESIGN.md](DESIGN.md).

This specification incorporates the approved design without a second divergent
set of requirements. The design's Gate 0/1/2, evidence and ownership sections are
normative. No backend substitution, host-runtime fallback or budget increase.

## Execution graph

- [01 workload contract](tickets/01-workload-contract.md): resolved by user approval.
- [02 TorchTitan CPT](tickets/02-torchtitan-cpt.md): isolated owning-repo implementation,
  runtime qualification, real train/checkpoint/resume and independent review.
- [03 verl GRPO](tickets/03-verl-grpo.md): isolated owning-repo implementation,
  compatible environment, genuine rollout/update/sync/resume and review.

CPU preparation can overlap; GPU use is serialized by the parent coordinator.
The shared interface is the approved Insula input/output contract and GPU budget,
not a requirement for the two workloads to share package versions or checkpoints.
Native repo work items carry exact entrypoints, meaningful behavior tests, and
implementation details after inspecting the selected runtime.

## Verification boundaries

Real GPU execution is mandatory for workload completion. Unit tests, imports,
config composition and historical results do not substitute for new live receipts.
If a prerequisite blocks execution, retain exact failed command/artifact, tested
alternatives and unmet acceptance; never close a live-run ticket as successful.
