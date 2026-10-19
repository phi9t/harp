# 14: Extend the service owner's advertised capabilities

Status: open
Type: follow-up
Parent: [Repository architecture](../spec.md)

## What to build

Future service-owner work, outside the existing-code consolidation. Add a
capability only when a concrete service needs it: non-process readiness/health
probes, secret resolution, privileged identities, Insula destruction, or
controller-independent keep-services lifetime. Current unsupported capabilities
must continue to refuse admission. Preserve the supplied rich supervisor policy
and probe fixtures as reference behavior.

## Acceptance criteria

- [ ] A requested capability has a concrete service use case and explicit owner admission.
- [ ] Real-owner tests cover its namespace access, bounded effects, stale epochs/generations,
  probe/recovery budgets, interrupted stop/restart and unknown outcomes.
- [ ] Secret bytes and control endpoints remain outside workload projections.
- [ ] Any survival claim is proved across both controller and owner failure boundaries.
- [ ] Update capabilities and operator docs only after its native matrix passes.

## Blocked by

- [13: Imported supervisor and bounded owner](13-integrate-insula-supervisor.md)

## Verification

Use the root workspace lock, actual Insula namespaces/cgroups and source-bound
native evidence. Fixture-owner success does not establish production capabilities.
This ticket is not a blocker for the Codex-first repository consolidation.
