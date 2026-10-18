# Research spike: Pi Durable and Pi Env

## Intent

Investigate `earendil-works/pi` packages `durable` and `env` against their
implementation and tests, compare them with Corenius, and identify what can be
reused and how. Requested by the owner on 2026-10-06.

## Route and authority

`ask-matt` routes this bounded investigation to standalone `research`, with
parallel primary-source package reviews and a Corenius integration synthesis.
This is research, not implementation or an approved architectural decision.
No product code, dependencies, policy, shared checkouts, or sibling repositories
will be changed. No commits, pushes, deployment, or paid model runs are authorized.
Use the existing `codex/pi-env-deep-dive` worktree. Each background researcher
has its own worktree; only the lead writes the integrated result here.

## Acceptance criteria

- Pin upstream and Corenius commits and link claims to primary source.
- Explain architecture, public interfaces, storage, recovery, tool effects,
  execution lifecycle, security boundary, deployment and relevant tests.
- Distinguish documented guarantees, source observations, exercised behavior,
  inference and unverified claims.
- Compare with the retained SQLite controller and Insula implementation, not
  only the historical PostgreSQL/Temporal roadmap.
- Give ranked reuse options, concrete adapter boundaries, incompatibilities,
  and bounded follow-up probes with acceptance criteria.
- Retain the report under `docs/spikes/pi-env-durable/` and lint this ticket.

## Sources and deliverables

- Corenius baseline: `d7f674829e9844d011869b711d64fcf55313f7f1`.
- Pi snapshot: `ae92585d3b3e5f1e4b123d14a34314d826d8d9f5`.
- Research source checkout: `.scratch/pi-research/upstream/` (regenerable).
- Report: `../../spikes/pi-env-durable/README.md` plus package source reviews.
- Status: source research and ticket 02 complete. The bounded interrupted-effect
  experiment passed independent review and fresh integration reproduction.

## Follow-up authorization

On 2026-10-06, the owner said “give it a try” after the proposed model-free
interrupted-effect experiment. This authorizes a disposable probe of a Pi worker
and the existing Corenius controller, including an actual worker kill/restart.
The probe uses a controller progress record as its effect. It does not qualify
supervised workload execution or require a real model. Retain probe source and
sanitized evidence; leave production code unchanged. Commit, push, deployment,
paid models and migration remain outside this authorization.
