# 12: Bring Soma and its Emacs workloads into lab/soma

Status: in-progress
Owner: soma-import agent (codex/soma-migration)
Parent: [Repository architecture](../spec.md)
Inventory: [benchmark coverage](../../../docs/workstream/repo-architecture/benchmark-inventory.md)

## What to build

Acquire the existing Soma Emacs/Doom gym and DevX AI/LSP verification workload
into lab/soma/ and lab/soma/ai_lsp/, respectively. Preserve Soma as a coherent
project, including its gym, bridge, lifecycle workers, mechanism probes and
maintained source documentation, rather than extracting only benchmark folders.
Include the required
probe, fixture, oracle, profile and documentation code. Shared isolation/service
mechanisms use insula/ and its supervisor; agent execution/collaboration uses
durable/, with no duplicate source of lifecycle authority. The imported Corenius
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
- [13-integrate-insula-supervisor](13-integrate-insula-supervisor.md)

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

## Acquisition and migration evidence

The coherent source project is now in `lab/soma/`, including six gym families,
checked bridge, dirty API changes, daemon client, lifecycle worker,36 probes,
source-grounded docs and plan-only API/export tickets. Separately acquired AI/LSP
adapters, independent expected values and fixture generators are under
`lab/soma/ai_lsp/`, with tests colocated with adapters and launcher.

- Soma source HEAD: `0bfd646a1e2263b53f723796ab2945db325aacbf`.
- AI/LSP source HEAD: `2724406183236616badef061138208eb20417903`.
- `lab/soma/source-map.json`:822 source/disposition records,442 destination
  payloads. This includes historical tracked files deleted in the dirty source.
- Exact committed archives, dirty binary patches, all untracked bytes (including
  transient executable shadow copies), file hashes and source status retained in
  task-private acquisition evidence. Post-acquisition source hashes/status/HEAD
  matched with zero mismatches. Original source checkouts were never mutated.
- Published source and traces are declared sanitized derivatives. Private host,
  operator/storage destination and source-root configuration are excluded.
- No upstream repository license was present; licensing remains explicit.
- Rootfs profile/plan schema remains Soma-specific; namespace entry delegates
  to shared `insula.capsule.execute_prepared` with an explicit minimal environment.
  The old host-bind namespace-only wrapper now requires selected-rootfs entry.
  The optional sibling-repository HDFS introspection/ticket writer is retired;
  sanitized historical source remains under `history/retired-external-storage/`.

### Fresh checks

- Portable Python source checks:29 tests, plus5 preservation contract tests;
  all passed. These cover selected-rootfs boundary policies, real temporary-tree
  manifests, aggregation identity rejection and omission/mutation checks.
- All shell wrappers passed Bash parsing; actual checked bridge passed Emacs
  `check-parens` on the existing Emacs30.2 host toolchain.
- Fresh Linux selected-rootfs runs via shared Insula:8 suites,139/139 scenarios
  as expected; real Python/C++/Rust language servers, Jupyter continuity,
  comint submission/interrupt, Magit/Dired/Org and daemon/bridge transport.
- Fresh selected-rootfs mechanism probes:36/36; cold/warm lifecycle:4/4 children,
  summary pass. This uses a read-only pre-existing selected rootfs and fresh
  task-owned output/cache state, not the source checkout's historical traces.
- AI/LSP launcher fixture runs on Linux:3 diagnostic and13 lifecycle cases passed
  with exit0. Fixed inherited fixture problems: async-shell ignored SIGINT,
  reaped-PID cleanup under `set -e`, and a race-dependent private-file assertion.
  Public redaction, bounded output, ownership, signal and cleanup checks remain.
- Aggregation wrapper now honors explicit rootfs selection consistently; macOS
  temp-root test fixtures resolve `/var` aliases before recording identity.

`lab/soma/qualification.json` records executed source hashes and exact result
scope. That foreground qualification does not claim durable admission/recovery
or supervisor-backed cancellation. Final integrated candidate qualification must
run through the shared durable/runner owner. Authenticated Copilot completion
checks remain explicitly unqualified; no credentials or external service access
were imported. New API/export tickets remain plan-only. Source preservation is
complete; final owner integration acceptance remains coordinated with tickets04,
05 and13.
