# 03 relocate-controller: Relocate the controller and its tooling

Type: task
Status: open
Owner:
Blocked by: 01
Claimed:

Triage: ready-for-agent

**Parent:** Corenius semantic repository layout specification (same effort).

**What to build:** Controller maintainers and operators find the active Rust service, Python tooling and tests in one semantic component, with the existing Linux public-interface integration still working.

**Source baseline:** 48d71588fff00ac70fa4a29e066a47a6b05f2ebd; execute against the integration revision containing completed blockers.

## Acceptance

- [ ] Inventory controller-owned launchers, relay, operator views, tests, fixtures, SQL assets and runtime support before moves; leave ambiguous shared tooling in place with rationale.
- [ ] Controller-owned code moves together; worktree, tracker, effort, evidence and landing tools retain their engineering ownership.
- [ ] Preserve package/binary identities, SQL inclusion, Python imports, companion adjacency, launch-only and build-and-launch behavior after relocation.
- [ ] Replace obsolete directory-depth assumptions where needed and preserve repository-local scratch/evidence destinations and identity-bound cleanup.
- [ ] Update current commands, documentation, gate registrations and established consumers; retain thin forwarding only for evidenced old script consumers, preserving arguments and exit status.
- [ ] The existing real Linux client integration passes unchanged public assertions for Unix socket, CLI/dashboard/MCP, bounded UTF-8 payloads, pagination, hashes, scoped rejection and idempotent receipt replay.
- [ ] Controller Rust suites retain locked/offline, fault-injection and concurrency intent; affected launcher, relay and operator-view Python suites pass.
- [ ] Qualification caveats and retained historical records are preserved; integration success does not qualify the complete live-worker baseline.

## Exclusions

No protocol/schema changes, admission or permission redesign, real-model trial or final dependency consolidation.

## Verification evidence

Controller cargo test --locked --offline with fault-injection and existing concurrency settings; explicitly run its client integration target; affected Python suites; retain archived client evidence and owned-process exit proof.

## Execution constraints

Linux only. Use an isolated working tree with one writer. Preserve existing work and historical evidence. Implementation requires the subsequent execution phase; ticket publication alone does not authorize commits, pushes, pull requests or merges. Review on repository Standards and parent Spec axes.

## Comments
- 2026-10-07T08:23:48Z codex: Publication normalization preserves ready-for-agent triage, acceptance and blocker graph; implementation remains unstarted.
