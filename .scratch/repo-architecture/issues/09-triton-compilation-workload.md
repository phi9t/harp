# 09: Add a bounded Triton compilation workload

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)

Implementation scope: deferred workload follow-up; architecture only in the current pass.

## What to build

Architecture follow-up only in this pass. Later implement lab/triton_compile/ with a small fixed kernel set, controlled compilation caches, phase timing and independent execution checks. Bind compiler, target, inputs and cache conditions to each result.

## Acceptance criteria

- [ ] Required GPU/toolchain capability is explicit; missing support is not a passing skip or CPU substitute.
- [ ] Cold and warm runs use declared cache conditions and record compilation phases separately from kernel execution.
- [ ] Compiled kernels agree with an independent reference on representative boundary shapes and numeric tolerances.
- [ ] Runner executes the workload under Insula and returns sanitized, identity-bound results; timings alone cannot establish correctness.

## Blocked by

- [04-extract-insula](04-extract-insula.md)
- [05-ssh-runner](05-ssh-runner.md)

## Verification

Run actual supported GPU compilation/execution with controlled inputs and fresh/warm caches. Preserve failed attempts and compare outputs independently. Avoid performance thresholds until a stable measured baseline exists.

No implementation or qualification result is recorded yet.
