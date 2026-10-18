# 02: Operate the durable runtime through Harp’s CLI

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Explicitly start, inspect and stop a project-scoped instance of the Corenius-adapted Python durable runtime through Harp. Run a deterministic managed activity and retain its state for a later invocation.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] The public CLI executes a complete deterministic workflow using the existing adapted runtime, records its result, and reports its durable identity and outcome.
- [ ] Stopping and reopening the same state recovers accepted results without repeating completed activities. A concurrent competing runtime for that state is refused.
- [ ] Status distinguishes pending, running, completed and blocked work and exposes useful logs without exposing private runtime credentials.
- [ ] The runtime validates the supported local SQLite/filesystem and execution contract before dispatch. Missing prerequisites are reported before launching an activity.
- [ ] Explicit runtime lifetime requires no installed always-on service and no dependency on another checkout, the supplied ZIP or a private historical environment.

## Blocked by

None (can start immediately).

## Scope

Reuse the existing replay and guardian mechanisms. This slice demonstrates a managed deterministic activity, not a qualified coding provider or an added scheduler.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.
