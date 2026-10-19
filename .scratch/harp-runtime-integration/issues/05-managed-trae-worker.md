# 05: Execute a managed Trae worker

Status: in-progress
Owner: Codex
Branch: codex/harp-durable-runtime
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

2026-10-18 integration: the managed process adapter, provider-specific credential
projection, explicit network policy and full sealed-binary integrity checks are
wired into the public lifecycle. A real namespace launch exposed a mount target
under readonly `/opt`; no provider process ran. The failed attempt's empty-kernel
cleanup receipt was inspected and its effect explicitly resolved as failed, with
no retry of that admission. Provider bindings now use capsule-owned
`/cap/providers`. A fresh Linux suite passed 19 cases, zero skips, including an
actual payload plus independent check with the provider profile, both adapters'
process contracts and policy boundaries. Evidence is under
`/data03/harp-runtime-integration-philip-yang/runs/provider-mounts-01/`.
Live provider qualification is in progress in the separately pinned
`runs/live-one-02` profile; this ticket is not yet complete.

The first networked profile used an offline native closure with an empty
resolver. It was stopped, its unchanged private workspace and guardian cleanup
evidence inspected, and the effect explicitly resolved as failed without retry.
A separate `rootfs-network` copy preserves the original snapshot and adds pinned
resolver bytes and default TLS trust-path aliases. Real namespace DNS/TLS probes
pass; a 34-case Linux provider/native suite also passes with zero skips
(`runs/provider-ready-01`). Preparation now rejects missing/empty/invalid resolver
configuration before dispatch. The `runs/live-one-03` Codex process returned unchanged source because disabling
its code-mode host disabled coding tools. Independent verification rejected the
proposal; no approval was recorded. The current adapter enables the matching
packaged host while disabling optional apps and plugins. Setup now runs bounded
offline version/help probes against all three sealed executables before publishing
the profile, and admission retains the resulting version/capability/hash binding.
The integrated Linux suite at `runs/provider-complete-03` passed 38 cases with zero
skips (16.414 seconds). The fresh live qualification is `runs/live-one-04`; its
results and exact-candidate approval remain pending.
