# 10: Add a checkpointable small transformer training job

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)

Implementation scope: deferred workload follow-up; architecture only in the current pass.

## What to build

Architecture follow-up only in this pass. Later implement lab/training/ with a tiny transformer, fixed miniature dataset, training/evaluation, model/optimizer checkpoints and bounded continuation after process restart. Use TorchTitan-like operational behavior without committing to importing TorchTitan.

## Acceptance criteria

- [ ] A real single-device supported GPU run has finite losses/gradients and improves on the bounded task.
- [ ] Checkpoint/restart preserves model, optimizer, data position and RNG state needed by the declared continuity contract.
- [ ] An interrupted training effect reconciles its checkpoint before retry; replay is not described as model-memory restoration.
- [ ] Results record source, dataset, software/profile, checkpoint and oracle identities. No implicit GPU allocation or multi-device/fleet scheduler.
- [ ] An independent bounded reference checks checkpoint continuation and model outputs, rather than trusting a worker report.

## Blocked by

- [04-extract-insula](04-extract-insula.md)

## Verification

Compare a continuous reference with a checkpoint/restart run under the stated determinism tolerance. Exercise corrupt/mismatched checkpoints and an interrupted write. Qualify one supported device before considering distributed execution.

No implementation or qualification result is recorded yet.
