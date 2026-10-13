# 03 relocate-controller: Relocate the controller and its tooling

Type: task
Status: resolved
Owner: TRAE
Blocked by: 01
Claimed:

Triage: ready-for-agent

**Parent:** Corenius semantic repository layout specification (same effort).

**What to build:** Controller maintainers and operators find the active Rust service, Python tooling and tests in one semantic component, with the existing Linux public-interface integration still working.

**Source baseline:** 48d71588fff00ac70fa4a29e066a47a6b05f2ebd; execute against the integration revision containing completed blockers.

## Acceptance

- [x] Inventory controller-owned launchers, relay, operator views, tests, fixtures, SQL assets and runtime support before moves; leave ambiguous shared tooling in place with rationale.
- [x] Controller-owned code moves together; worktree, tracker, effort, evidence and landing tools retain their engineering ownership.
- [x] Preserve package/binary identities, SQL inclusion, Python imports, companion adjacency, launch-only and build-and-launch behavior after relocation.
- [x] Replace obsolete directory-depth assumptions where needed and preserve repository-local scratch/evidence destinations and identity-bound cleanup.
- [x] Update current commands, documentation, gate registrations and established consumers; retain thin forwarding only for evidenced old script consumers, preserving arguments and exit status.
- [x] The existing real Linux client integration passes unchanged public assertions for Unix socket, CLI/dashboard/MCP, bounded UTF-8 payloads, pagination, hashes, scoped rejection and idempotent receipt replay.
- [x] Controller Rust suites retain locked/offline, fault-injection and concurrency intent; affected launcher, relay and operator-view Python suites pass.
- [x] Qualification caveats and retained historical records are preserved; integration success does not qualify the complete live-worker baseline.

## Exclusions

No protocol/schema changes, admission or permission redesign, real-model trial or final dependency consolidation.

## Verification evidence

Controller cargo test --locked --offline with fault-injection and existing concurrency settings; explicitly run its client integration target; affected Python suites; retain archived client evidence and owned-process exit proof.

## Execution constraints

Linux only. Use an isolated working tree with one writer. Preserve existing work and historical evidence. Implementation requires the subsequent execution phase; ticket publication alone does not authorize commits, pushes, pull requests or merges. Review on repository Standards and parent Spec axes.

## Comments
- 2026-10-07T08:23:48Z codex: Publication normalization preserves ready-for-agent triage, acceptance and blocker graph; implementation remains unstarted.
- 2026-10-13T00:10:32Z TRAE: moved the active controller and controller-owned Python launch/operator tooling to `controller/`; kept generic worktree, tracker, evidence and landing tools under `tools/agents/`. Verified `python3 tools/temp_root.py -- sh tools/agents/checks.d/75-controller-python.sh` and full `python3 tools/temp_root.py -- sh tools/agents/checks.d/70-sqlite-controller.sh`, both exit 0.
