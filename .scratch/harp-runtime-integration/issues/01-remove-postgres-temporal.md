# 01: Remove PostgreSQL/Temporal from active builds and gates

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Build and check the selected local product without PostgreSQL or Temporal services, SDK dependencies or hidden fixture requirements. Preserve recoverable source history and license/evidence records.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] Normal build, bootstrap dependency resolution and selected local checks no longer select the abandoned PostgreSQL/Temporal stack.
- [ ] User-facing entry points and contributor instructions identify the lightweight Python + SQLite runtime as the target; removed experiment commands do not silently start services.
- [ ] Any removal from the imported payload has an explicit provenance disposition. Captured evidence bytes and upstream license records are preserved.
- [ ] Real clean-build/dependency-graph and gate-selection evidence demonstrates the absence of PostgreSQL/Temporal requirements; changing only documentation is insufficient.

## Blocked by

None (can start immediately).

## Scope

This independently useful cleanup removes a parked dependency path. It does not require provider execution, a new database service or a rewrite of retained historical evidence.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.
