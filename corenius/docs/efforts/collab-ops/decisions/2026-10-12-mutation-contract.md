# Decision: shared mutation contract for Collab Ops follow-up

Date: 2026-10-12
Status: approved for tickets 20-25
Read commit: `de806765f3e141a2fa4622b8a4f4b91f5983afc8`

## Decision

The follow-up uses `tools/collab/landing_protocol.py` as the shared repository
mutation module for every managed trunk writer covered by Collab Ops. Public CLI
entry points and compatibility shims remain stable; callers route their existing
claim, release, publication and record-repair effects through the shared guard.

The lock order is:

1. compatibility `land.lock`, when the legacy route participates;
2. Collab `lane.lock`;
3. Collab `meta.lock`.

Callers that already require a check lock acquire that check lock before the
publication locks, as in the existing check-runner contract. Gate execution stays
outside publication locks, followed by a fresh mutation guard and preflight
immediately before effects.

The approved scope is all managed trunk writers listed in `FOLLOWUP.md`: Collab
admission, allocation recovery, landing, recovery, abandonment and claim release;
legacy landing and recovery; record-bookkeeping commits; effort record commits;
and the retained `lead_launch` claim and release writers. This remains a
cooperative local guarantee and does not fence arbitrary external Git commands,
controller redesigns or migration rewrites.

Liveness evidence is a decision, not a boolean. The shared states are
`running`, `stopped`, `unknown` and `unbound`. Supplied but unusable worker
evidence is `unknown` and blocks inspect/recover/abandon/refresh/GC/sweep and
retention cleanup. Valid stopped workers and explicit unbound confirmation keep
working.

Recovery treats remote publication, local publication and record repair as
separate effects. When exact attributed local success is already established,
recovery may repair only the bound local records/state offline and idempotently.
It must not repeat a push, repeat an Integration fast-forward or duplicate
acceptance records. Outstanding local fast-forward with offline remote remains
uncertain unless exact remote acknowledgement is available.

## Budget

The current implementation request authorizes this bounded follow-up: implement
tickets 20, 21, 22, 23 and 25; run ticket 24 qualification; perform up to three
focused repair/review iterations for blockers found by that qualification. Any
broader scope, controller redesign, migration rewrite, remote publication or PR
creation requires separate human authority.
