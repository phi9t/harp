# Ticket17 broker-seam behavioral proof, evaluation repair1

Read base `963f58b31e518a8162dfe719e386ef9d9843321f` plus accepted10 source
manifest. Candidate only; independent review and ticket disposition remain
root-owned. Own worktree `corenius-codex-kata-evaluation`. Gateway candidate
remained frozen throughout this evaluation work.

## Reproduction

Hash-verified accepted10 source was copied to
`.scratch/team-control-plane/kata17/broker-source` and built with:

```
cargo build --manifest-path .scratch/team-control-plane/kata17/broker-source/Cargo.toml --locked
python3 -m py_compile docs/efforts/team-control-plane/research/probes/kata_broker_probe.py
python3 docs/efforts/team-control-plane/research/probes/kata_broker_probe.py \
  --kata-binary .scratch/team-control-plane/kata17/kata-pinned \
  --broker-binary .scratch/team-control-plane/kata17/broker-source/target/debug/team-control-plane \
  --broker-source .scratch/team-control-plane/kata17/broker-source \
  --accepted-manifest .scratch/team-control-plane/kata17/broker-accepted-manifest.json \
  --output docs/efforts/team-control-plane/evidence/ticket17-kata-broker-results.json
```

Build, syntax check and final public fixture each exited0. The executable probe
rechecks accepted source hashes, records both binary hashes, and drives real
Kata HTTP, actual compiled Rust broker HTTP and PostgreSQL. No routine old spike
or kernel suite was restarted. No production Rust code changed.

## Red → green

Meaningful red (`.scratch/team-control-plane/kata17/bridge-red.log`, exit1):
actual Kata claim event received twice with delivery-specific broker keys.
First request returned200 with an admitted attempt/operation/reservation receipt;
duplicate returned409 `stale_revision`. Assertion failed: “duplicate source event
must return the same durable assignment receipt.” No nonexistent/mocked broker
was used. Fixture services/container were stopped by its finally cleanup.

Green (`bridge-final.log`, exit0): key derives from persisted source origin,
actual event UID, approved source digest and explicit owner decision ID.
Concurrent delivery returns the same durable original receipt. A newly
constructed reader reloads the mapping and decision; the replay remains exact.
The public task snapshot reports one attempt, one operation, one reservation
and one outbox entry. Mission read model reports one admitted attempt/episode
and one capacity unit. The distinct source claim before owner authorization
reports zero attempts.

The approved title/body/input digest remain unchanged after actual title edit,
actual issue close claiming an unexecuted test, repeated source event delivery,
actual broker-to-Kata projection, repeated HTTP metadata commands and projection
loop replay. The broker remains running and its admitted spec/revision remains
exact; a Kata token cannot authenticate to broker HTTP (401).

Stopping Kata and logically restoring only its owned database uses actual
`pg_dump`/drop/recreate/`pg_restore`. Immediately before drop, the uniquely named
container's ownership label and bound loopback port are rechecked. Broker remains
running in a separate database. Restored source origin/event UID and projection
are preserved; the same event/decision returns the same original receipt and
all four broker counts remain1. Final JSON retains the full public snapshots,
source/broker event IDs, immutable approval mapping and decision.

Final cleanup assertions: all foreground services terminal; all listeners
closed; owned tmpfs PostgreSQL container removed; temporary home/Git files
removed only after service shutdown. Zero model calls, no hooks/adoption/tracker
migration, shared installation changes, production mutations, commit or push.

## Limits

This is an evaluation fixture, not a installed remote Rust adapter or durable
production outbox worker. Provider starts, Temporal, production delivery/reset/
outage behavior, supervisor guarantees and production backup/PITR are separate.
Existing Kata upgrade/SQLite/PG restore probes were preserved unchanged. The
new bridge restore proves its evaluated authority seam, not all restore cases.
Qualitative adapter/service costs and measured fixture size/runtime support
proposed defer; production engineering effort remains unmeasured. Independent
Standards/Spec review is still required; no ticket acceptance is asserted.
