# 13: Retire the superseded Corenius controller paths

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Retain useful imported provider/Git/verification mechanisms under the selected runtime and retire the old Corenius controller launchers, authority paths and obsolete tracker integration.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] Inventory imported controller, Collab and tracker entry points, distinguishing reusable execution/Git mechanisms from competing lifecycle owners.
- [ ] No supported launcher or fallback starts the old controller to admit, schedule, cancel, recover, verify, approve or land the new path’s workers.
- [ ] Retain required mechanisms as subordinate adapters/libraries with the selected runtime owning authoritative transitions; remove obsolete controller/tracker commands or provide explicit retirement guidance.
- [ ] Git-managed Markdown remains intended work. Kata, a second backlog or sibling-policy synchronization is not introduced as an operational dependency.
- [ ] Update affected imports, build/gate selection, operator guidance and import-preservation dispositions. Preserve recoverable source history and captured evidence/license bytes.
- [ ] CLI/build tests demonstrate retirement and the continued selected-runtime path, without relying on an old daemon or historical private environment.

## Blocked by

- [09: Qualify the complete one-worker milestone](09-qualify-one-worker.md)

## Scope

This batch covers imported Corenius control paths. It does not require the two root Harp caller migrations to finish first; final integration checks all three together.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.
