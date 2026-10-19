# 01 runtime-capabilities: establish the actual Trae adapter surface

Type: task
Status: resolved
Owner: research-trae-runtime
Blocked by:
Claimed:
Label: wayfinder:research
Parent: ../map.md

## Question

Which installed Trae interfaces can support thread identity, turn start,
observation, steering, detach, cancellation, runtime incarnation and recovery?
What exists in Corenius's ACP integration, process supervision, durable
execution design and capsule toolbox? Which guarantees are not yet exercised?
Identify the safest first adapter and the precise missing probes.

## Acceptance

- [x] Report cites pinned local source, installed CLI versions and safe help/schema observations.
- [x] Distinguish available interfaces from tested persistence/cancellation guarantees.
- [x] Identify idempotency/correlation and ambiguous-start limitations.
- [x] Recommend bounded probes without running paid models or changing services.

## Answer

See [Installed Trae runtime capabilities](../research/trae-runtime-capabilities.md)
and the retained [selected schema snapshot](../research/trae-schema/README.md).
This resolves interface discovery only, not the live runtime proof.

## Comments

- 2026-10-01T04:20:19Z lead: integrated the isolated research report and public schema snapshot; no model/service launch or commit. Actual runtime probe approval remains a separate decision.
