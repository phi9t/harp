# 07: Relocate benchmarks and finish imported-tree removal

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)

## What to build

Classify every retained imported source as active first-party code, workload, captured evidence, maintained documentation or history-only material. Move benchmark tasks/validators and applicable root benchmarks into lab/benchmarks/. Replace the fixed-directory import verifier with explicit relocation/disposition provenance and remove the imported wrapper tree.

## Acceptance criteria

- [ ] All absorbed source tips and worktree captures remain ancestors of the delivery candidate; original blob/mode/license identities remain auditable.
- [ ] Every previously tracked imported path has an explicit new destination or reasoned retirement, with adapted retained bytes checked.
- [ ] Retained benchmark materialization and independent grading work; hidden solutions/oracles remain separate from agent workspaces.
- [ ] No legacy agent/controller launch returns as a benchmark fallback. Historical docs do not claim current execution ownership.
- [ ] The old imported directory and old package import paths are absent without compatibility archives. Captured evidence is never silently rewritten.

## Blocked by

- [01-unify-durable-control-plane](01-unify-durable-control-plane.md)
- [04-extract-insula](04-extract-insula.md)

## Verification

Run provenance byte/mode/ancestry and ledger-completeness tests, representative real task materialization/grading, supported-entrypoint checks and changed-path dispatch tests. Review each retirement class, not just file counts.

No implementation or qualification result is recorded yet.
