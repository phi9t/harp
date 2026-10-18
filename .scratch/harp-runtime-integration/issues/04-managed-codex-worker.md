# 04: Execute a managed Codex worker

Status: ready-for-agent
Owner: unassigned
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Run a Codex coding worker through the admitted worker-activity contract, with isolated source, immutable inputs, observable progress and a retained result under the selected runtime.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [ ] The public Harp CLI selects Codex explicitly and records executable/version, supported capabilities and the relevant provider execution binding.
- [ ] The worker receives the admitted brief and workspace through the common contract. The selected durable runtime remains the only lifecycle owner.
- [ ] Provider startup failure, malformed output, timeout and ordinary completion produce truthful bounded evidence and outcomes. Missing/incompatible capabilities refuse dispatch.
- [ ] Provider credentials and owner approval/landing capabilities remain outside worker-writable state; the adapter does not broaden native permissions or silently switch providers.
- [ ] The worker result and logs bind to the admitted attempt. A deterministic Codex protocol fixture exercises the actual adapter/process boundary without substituting another scheduler.

## Blocked by

- [03: Admit a pinned task into an isolated workspace](03-admit-pinned-task.md)

## Scope

This ticket delivers the Codex path. It does not by itself establish milestone-one acceptance or authorize a live paid/provider run; final live evidence belongs to ticket 09.

## Verification

Verify observable behavior at the public CLI/process/Git boundary and reuse
existing contract coverage where sufficient. Run the changed-surface gate against
the complete candidate and supplement it for affected consumers. Record exact
source, selected checks, outcomes and limitations here. Historical qualification,
missing prerequisites or unavailable required capabilities are not fresh passes.
