# 05: Execute a managed Trae worker

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Run a Trae coding worker through the same admitted worker-activity contract and durable authority as Codex, with isolated source and retained observable results.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] The public Harp CLI selects Trae explicitly and records executable/version, supported capabilities and the relevant provider execution binding.
- [ ] The adapter supplies immutable task/context inputs and its isolated workspace through the common contract; provider conversation identity does not replace task or attempt identity.
- [ ] Provider startup failure, malformed output, timeout and ordinary completion produce truthful bounded evidence and outcomes. Missing/incompatible capabilities refuse dispatch.
- [ ] The adapter preserves native permission enforcement, keeps private owner capabilities outside worker control and does not silently fall back to Codex.
- [ ] The worker result and logs bind to the admitted attempt. A deterministic Trae protocol fixture exercises the actual adapter/process boundary using the same observable contract as Codex.

## Blocked by

- [03: Admit a pinned task into an isolated workspace](03-admit-pinned-task.md)

## Scope

This ticket is independent of the Codex adapter once the shared activity contract exists. Final real-provider acceptance belongs to ticket 09.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.
