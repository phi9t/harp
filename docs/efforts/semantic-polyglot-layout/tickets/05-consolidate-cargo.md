# 05 consolidate-cargo: Consolidate Cargo dependency management

Type: task
Status: open
Owner:
Blocked by: 02, 03, 04
Claimed:

Triage: ready-for-agent

**Parent:** Corenius semantic repository layout specification (same effort).

**What to build:** Developers use coherent Cargo workspace dependency management where compatible while preserving native commands, explicit component checks and an evidenced fallback for incompatible packages.

**Source baseline:** 48d71588fff00ac70fa4a29e066a47a6b05f2ebd; execute against the integration revision containing completed blockers.

## Acceptance

- [ ] Attempt all three packages as root workspace members, with only the terminal agent as default member and unchanged package/binary identities.
- [ ] In the successful outcome the root lockfile is authoritative; remove redundant member locks only after verifying builds and required locked/offline behavior.
- [ ] Preserve controller feature selection, experimental service-backed gate participation, component-local usage and explicit custom target directories.
- [ ] Update executable output discovery and documentation for shared target locations; do not break adjacent supervisor/gate installation contracts.
- [ ] If consolidation requires unrelated dependency changes or breaks required locked/offline behavior, diagnose and record the exact resolver/cache incompatibility; incorrect workspace wiring does not qualify as a blocker.
- [ ] In the fallback outcome use explicit membership/exclusions and retain authoritative independent workspace declarations/locks; record exclusions and bounded follow-up work without blocking the semantic layout.
- [ ] Native test/build evidence proves the selected outcome; metadata success alone is insufficient.
- [ ] No ambient dependency assumptions, unrelated upgrades or implicit default builds of the parked product are introduced.

## Exclusions

No Bazel adoption, mandatory uv workspace, unrelated dependency upgrades or removal of experimental checks.

## Verification evidence

Cargo metadata for membership/default selection plus actual locked/offline root and controller builds/tests; experimental service runner as available; record lock ownership, resulting commands, source revision and any fallback evidence.

## Execution constraints

Linux only. Use an isolated working tree with one writer. Preserve existing work and historical evidence. Implementation requires the subsequent execution phase; ticket publication alone does not authorize commits, pushes, pull requests or merges. Review on repository Standards and parent Spec axes.

## Comments
- 2026-10-07T08:23:48Z codex: Publication normalization preserves ready-for-agent triage, acceptance and blocker graph; implementation remains unstarted.
