# 12: Bring Soma and its Emacs workloads into lab/soma

Status: done
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

- [x] All six tracked gym scenario families, checked bridge, support dependencies and benchmark traces/reports have a lab entry and source/disposition mapping.
- [x] Inspect the changed/untracked bridge/API/export work, preserve exact snapshot provenance, and distinguish implemented scenarios from plan-only tickets.
- [x] Preserve the AI/LSP verifier, adapters, independent expected values, fixtures and required portable configuration. Label offline and live/authenticated checks separately.
- [x] Required Emacs daemon, kernel, process and language-server execution occurs inside the admitted Insula closure. A namespace-only or host-wrapper run is not labeled rootfs-owned.
- [x] Port execution and cancellation to the selected durable/Insula/runner boundaries. No sibling checkout dependency or second controller remains.
- [x] Preserve captured evidence and licensing records. Sanitize public paths/logs and examples without silently rewriting original evidence. Actual credentials and private host/Doom configuration are excluded.
- [x] A completeness check rejects an omitted scenario, grader, fixture, supporting module or unreviewed benchmark retirement.
- [x] Run meaningful bridge/scenario and actual supported LSP/Jupyter/process checks; absent capabilities and unimplemented API/export work remain explicitly unqualified.

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


### Durable SSH qualification

Implemented the generic committed workload admission/worker path in
`durable/workloads/`, reusing the single journal, Activity, Insula profile,
guardian and manual recovery policy. Soma owns its adapter, manifest and frozen
acceptance checker under `lab/soma/`; there is no framework Soma registry.
`runner/qualification.json` records the native contract, source/profile digests,
artifact hashes and limits.

- Actual admitted selected-rootfs Soma run: 8 suites, 139/139 scenarios;
  9 declared compressed artifacts. Retrieved daemon result over SSH and verified
  its size, SHA-256 and decoded 63-case result.
- Killed SSH transport during a running workload, then queried the same run:
  RUNNING, one attempt. Explicit journal cancellation yielded BLOCKED/manual,
  cgroup.kill evidence and populated=0; all 5 original process identities across
  3 process groups disappeared. Transport loss itself was not treated as cleanup.
- Queued capacity reuse started only after that cleanup receipt, produced the
  independently checked answer and succeeded.
- An adversarial workload rewrote its checkout checker to succeed, but the frozen
  owner checker rejected its bad output: semantic FAILED, check exit1, passed=false.
- Wrong source/profile refused before state creation. After the ephemeral
  delegation was collected, native preflight refused the missing cgroup. An
  unavailable SSH host returned unknown without retry. Private inventory remains
  ignored; OpenSSH host-key checking stays enabled.
- Fixed a real transport problem: a proxy retained stdout/stderr after SSH died.
  Each request now owns its SSH process group and disables multiplexing. Both a
  real child-pipe regression and main-only real SSH kill returned promptly.
- The first admitted Soma attempt exceeded the 16 MiB daemon artifact bound;
  it remains retained and BLOCKED. Soma now compresses reports, leaving framework
  bounds intact. No source checkout or original rootfs was changed.

Focused portable checks and full preservation checks pass. Final composed
repository verification and exact source receipt remain the integration owner's
responsibility; this ticket remains in-progress until that candidate gate.
Authenticated external AI/LSP calls remain outside this qualification.

Agent candidate validation: 10 runner/workload boundary tests and 34 imported
source/preservation tests passed; source-map hashes/modes and staged privacy scan
passed. `mise run verify` and `--plan` refuse the copied shared
`durable/__init__.py` because this isolated base lacks the coordinator's new
verification mapping. No selected check was skipped; repository landing remains
blocked until that mapping and the composed candidate gate pass.

## Completion

Implemented and verified in body commit `c32c6f0f4a2de73e8067be0361121bb841ab4622`. The complete-candidate
changed-surface plan passes, including the bounded verification repairs recorded in
[the repository qualification](../../../evidence/provenance/repository-qualification.json).
Native runtime, benchmark and supervisor receipts bind the composed runtime bytes;
real Codex one/two/four-worker runs and all eight explicitly approved landings are
recorded separately in [Codex qualification](../../../evidence/provenance/codex-qualification.json).
Retained workloads and proof/PDF consumers passed their relevant checks.
