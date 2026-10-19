# 12: Preserve the complete Emacs benchmark work under lab

Status: ready-for-agent
Owner: unassigned
Parent: [Repository architecture](../spec.md)
Inventory: [benchmark coverage](../../../docs/workstream/repo-architecture/benchmark-inventory.md)

## What to build

Acquire the existing Soma Emacs/Doom gym and DevX AI/LSP verification workload
into lab/emacs/gym/ and lab/emacs/ai_lsp/, including their required bridge, worker,
probe, fixture, oracle, profile and documentation code. The imported Corenius
review points to these sibling sources, but their payloads are not yet in Harp.
Capture committed and dirty source identities before migration, preserving both
old evidence and uncommitted work without altering either source checkout.

## Acceptance criteria

- [ ] All six tracked gym scenario families, checked bridge, support dependencies and benchmark traces/reports have a lab entry and source/disposition mapping.
- [ ] Inspect the changed/untracked bridge/API/export work, preserve exact snapshot provenance, and distinguish implemented scenarios from plan-only tickets.
- [ ] Preserve the AI/LSP verifier, adapters, independent expected values, fixtures and required portable configuration. Label offline and live/authenticated checks separately.
- [ ] Required Emacs daemon, kernel, process and language-server execution occurs inside the admitted Insula closure. A namespace-only or host-wrapper run is not labeled rootfs-owned.
- [ ] Port execution and cancellation to the selected durable/Insula/runner boundaries. No sibling checkout dependency or second controller remains.
- [ ] Preserve captured evidence and licensing records. Sanitize public paths/logs and examples without silently rewriting original evidence. Actual credentials and private host/Doom configuration are excluded.
- [ ] A completeness check rejects an omitted scenario, grader, fixture, supporting module or unreviewed benchmark retirement.
- [ ] Run meaningful bridge/scenario and actual supported LSP/Jupyter/process checks; absent capabilities and unimplemented API/export work remain explicitly unqualified.

## Blocked by

- [04-extract-insula](04-extract-insula.md)
- [05-ssh-runner](05-ssh-runner.md)

Read-only source inventory and nonmutating snapshot capture can precede these
execution dependencies. This ticket is part of existing-benchmark preservation,
not one of the deferred new Bazel/Triton/training workload implementations.

## Verification

Verify source HEAD plus staged/unstaged/untracked snapshot identities, byte/mode
and source-to-destination coverage, support closure and sanitized output. Run the
actual migrated bridge/scenario entrypoints with independent negative and positive
oracles. Check daemon/kernel/server cleanup and never substitute historical traces
for current execution. No import, source snapshot or live qualification has been
performed yet.
