# 02 durability-sources: ground the proposed broker durability contracts

Type: task
Status: resolved
Owner: research-control-plane-sources
Blocked by:
Claimed:
Label: wayfinder:research
Parent: ../map.md

## Question

What first-party PostgreSQL, Temporal, Symphony and protocol/runtime sources
support the proposed separation of accepted domain state, durable workflow
control, runtime operations and artifacts? Identify implementation choices,
retry/deduplication limits and relevant SDK support. The supplied source
markers are unresolved and must not be treated as citations.

## Acceptance

- [x] Source-grounded report links opened primary sources, with retrieval date.
- [x] Separate directly supported claims from architectural recommendations.
- [x] Explain outbox/retry limits, deterministic workflow execution and continuation.
- [x] Identify a plausible implementation stack without claiming Trae parity with Codex.

## Answer

See [Durability and protocol sources](../research/durability-sources.md).
The report establishes source-supported mechanisms and explicitly marks its
implementation recommendations as inference. It does not resolve SDK pinning,
live runtime guarantees or the broker/Temporal bridge protocol.

## Comments

- 2026-10-01T04:20:19Z lead: integrated the isolated primary-source report. PostgreSQL, Temporal and protocol capabilities do not individually supply external-effect exactly-once guarantees.
