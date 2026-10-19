# 04: Execute a managed Codex worker

Status: done
Owner: Codex
Branch: codex/harp-durable-runtime
Parent: [Harp durable coding-worker integration](../spec.md)

## What to build

Run a Codex coding worker through the admitted worker-activity contract, with isolated source, immutable inputs, observable progress and a retained result under the selected runtime.

Use the Corenius-adapted Python + SQLite durable runtime as the sole execution
authority. Git-managed Markdown owns intent. Target Linux at devx_b200_dbg.
Both providers are required by milestone one; qualify one worker, then two,
then four. This ticket does not authorize implementation or live setup by itself.

## Acceptance criteria

- [x] The public Harp CLI selects Codex explicitly and records executable/version, supported capabilities and the relevant provider execution binding.
- [x] The worker receives the admitted brief and workspace through the common contract. The selected durable runtime remains the only lifecycle owner.
- [x] Provider startup failure, malformed output, timeout and ordinary completion produce truthful bounded evidence and outcomes. Missing/incompatible capabilities refuse dispatch.
- [x] Provider credentials and owner approval/landing capabilities remain outside worker-writable state; the adapter does not broaden native permissions or silently switch providers.
- [x] The worker result and logs bind to the admitted attempt. A deterministic Codex protocol fixture exercises the actual adapter/process boundary without substituting another scheduler.

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

Live qualification completed in `runs/live-one-04` using the frozen source at
`runs/provider-complete-03` (the Python payload committed in `45997f25`).
Provider codex produced a useful code change, captured as commit
`330797ab90b8e1b3b5b7054914a7b46754c1b492`, tree
`07d839f241e62194e3506cb302ae72798b7f270d`. Independent verification run
`verification-ca1aed2df85b415f6fced0e5b7178ecf9a1a2f37ca81bbf5134ec086b2b4b0da` passed all frozen checks
without modifying tracked candidate bytes. A fresh runtime invocation retained
the same coding attempt and result. Exact provider versions, sealed binary/helper
hashes, SQLite/Python versions, contract and receipts are retained in the private
run artifacts. The original source checkout remains unchanged. This completes
the adapter ticket; human approval/physical landing and integrated fault evidence
remain the ticket 09 milestone gate. No live approval has been inferred.
