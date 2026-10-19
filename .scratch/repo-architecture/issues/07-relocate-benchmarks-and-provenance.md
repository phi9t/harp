# 07: Relocate benchmarks and finish imported-tree removal

Status: in-progress
Owner: Codex on codex/repo-architecture
Parent: [Repository architecture](../spec.md)

## What to build

Classify every retained imported source as active first-party code, workload, captured evidence, maintained documentation or history-only material. Move benchmark tasks/validators and applicable root benchmarks into lab/benchmarks/. Replace the fixed-directory import verifier with explicit relocation/disposition provenance and remove the imported wrapper tree.

Use the [explicit benchmark inventory](../../../docs/workstream/repo-architecture/benchmark-inventory.md).
The scope includes effort-embedded fixtures/prototypes, inventories, tmux work,
results and branch/worktree exemplars, not only bench/tasks. Preserve every
benchmark collection under a lab entry; retire competing execution infrastructure
without retiring its benchmark payload. Emacs source acquisition is covered by
[ticket 12](12-preserve-emacs-benchmarks.md).

## Acceptance criteria

- [ ] All absorbed source tips and worktree captures remain ancestors of the delivery candidate; original blob/mode/license identities remain auditable.
- [ ] Every previously tracked imported path has an explicit new destination or reasoned retirement, with adapted retained bytes checked.
- [ ] Source-to-destination completeness covers all 42 current task directories, 18 readiness scenarios and additional benchmark assets outside bench/, including dirty worktree captures; unexplained benchmark omissions fail.
- [ ] Retained benchmark materialization and independent grading work; hidden solutions/oracles remain separate from agent workspaces.
- [ ] No legacy agent/controller launch returns as a benchmark fallback. Historical docs do not claim current execution ownership.
- [ ] The old imported directory and old package import paths are absent without compatibility archives. Captured evidence is never silently rewritten.

## Blocked by

- [01-unify-durable-control-plane](01-unify-durable-control-plane.md)
- [04-extract-insula](04-extract-insula.md)

## Verification

Run provenance byte/mode/ancestry and ledger-completeness tests, representative real task materialization/grading, supported-entrypoint checks and changed-path dispatch tests. Review each retirement class, not just file counts.

See the implementation progress below.

## Implementation progress

All 42 task fixtures and all 18 study trees are relocated under lab/benchmarks/. The executable absorption record verifies 8,088 original paths and 30 retained branch/worktree tips; benchmark/study retirement is prohibited. All 6,960 study files and four archived results remain byte-identical. Linux materialization (54), grader validation (20), and three red/green/repeated real-capsule reference tasks passed. Final source-bound benchmark receipt remains pending.
