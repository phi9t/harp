# 20 trae-probes: Measure actual Trae control and recovery capabilities

Type: task
Status: open
Owner:
Blocked by: 04, 16
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Run authorized bounded live probes against the pinned installed Trae version and schema in dedicated environments. Measure thread-store identity, create/resume/start/observe/steer/cancel, detach, restart, background jobs and goal-loop ownership.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Capability report records actual tested version, deployment/store identity, observed guarantees, unsupported paths, uncertainty and redacted evidence. Correlation fields are not called deduplication without proof.

## Exclusions

No unbudgeted model probes, real user workspace, provider fork or assumption from Codex APIs.

## Acceptance

- [ ] Establish a failing public behavioral fixture for the required outcome.
- [ ] Implement the smallest slice and record fresh passing evidence.
- [ ] Lost start acknowledgment and server restart either establish adoptable execution with evidence or precise quarantine. Interrupted thread creation is tested separately. UI detach does not substitute for executor durability. Process cancellation includes independent supervisor observations.
- [ ] Standards and Spec reviews have no unresolved blocking finding.
- [ ] Record artifact/run IDs, limitations and handoff to dependent tickets.

## Execution sequence

1. Rehydrate approved intent, check blockers, claim this ticket and allocate an isolated worktree.
2. Derive session-local exact files/commands from the interfaces; approve no new architectural authority implicitly.
3. Run the acceptance fixture red, implement, then run it green; retain meaningful red/green evidence.
4. Run focused checks, applicable repository gate and independent review; at most three repair/review rounds.
5. Update durable evidence and status only when requirements are proven. Commit/push need explicit authority; human integration remains separate.

## Comments

- 2026-10-01T05:48:16Z lead: Detailed planning task created at the owner's request. No execution or architectural decision is implied.
- 2026-10-03T18:33:56Z lead: One concretely admitted offline setup-noise case ran once. Independent adjudication passes exact physical cleanup measurements; full conformance remains partial because original prelaunch controller ordering is unverified. All12 frozen records, raw events, exact grant and independent review are retained in ../evidence/trae20-offline-result1/. No rerun, budget reset or actual Trae/provider admission follows; see ../decisions/20261003-trae-offline-result1-partial.md.
