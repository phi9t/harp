# 04: Make Insula a first-party isolation package

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)

## What to build

Move capsule, rootfs, temporary storage and confinement mechanisms into top-level insula/. Move runtime-specific admission/binding into durable/execution.py. Migrate every caller and frozen-code manifest; remove imported tools package dependencies.

## Acceptance criteria

- [ ] Insula rootfs/capsule construction and execution work without importing durable, git_workflow, runner or a workload package.
- [ ] Durable attempt admission, guardian ownership and receipt acceptance stay outside Insula.
- [ ] Projected inputs remain immutable; state, credentials, capability files and authoritative receipts stay outside worker namespaces.
- [ ] Missing cgroup delegation refuses execution without fallback. Namespace, network and descendant-cleanup tests pass on the moved code.

## Blocked by

- [01-unify-durable-control-plane](01-unify-durable-control-plane.md)

## Verification

Run real capsule/rootfs tests, public prepare/start/cancel/recovery tests and required Linux cgroup/namespace tests. Verify new sealed profiles use the moved source and old profiles remain unchanged.

No implementation or qualification result is recorded yet.
