# Kata: planning ledger reuse assessment

Status: read-only assessment; integration recommendation is inference
Read: 2026-10-01
Repository: https://github.com/kenn-io/kata
Commit read: `b70a4306464e319701df30a3b92860289613b15a`
Corenius base read: `963f58b31e518a8162dfe719e386ef9d9843321f`
No installation, hooks, daemon, database or code execution performed.

## Observed documentation

Kata presents itself as a durable issue ledger for coding agents and humans,
not an agent worker pool or Git workflow engine. It has CLI/TUI/browser UI,
issue relationships, claims, evidence-based closes, attention requests, MCP
and issue-scoped credentials. SQLite is its local default; a shared daemon
supports PostgreSQL. See the pinned [README](https://github.com/kenn-io/kata/blob/b70a4306464e319701df30a3b92860289613b15a/README.md).

The documented issue model has open/closed status, owner, relationships and
history. A workspace points to a project independently of its database. These
are useful planning concepts; they do not establish our attempt/episode/runtime
or accepted-candidate model. See [concepts](https://github.com/kenn-io/kata/blob/b70a4306464e319701df30a3b92860289613b15a/docs/guide/concepts.md).

Go hosts can embed a listener-free service. Host access control derives actor
identity; transaction fences run inside writable storage transactions, with
separate background-worker fences and a restricted embedding profile. This is
relevant existing authority machinery, rather than a claim that Kata lacks
fencing. It is not proof of external provider or process fencing. See
[embedding](https://github.com/kenn-io/kata/blob/b70a4306464e319701df30a3b92860289613b15a/docs/development/embedding.md).

PostgreSQL operation documentation uses a dedicated schema and separate
schema-owner/runtime roles, validation-only serving, offline upgrades and
backup/restore ceremonies. Reuse comes with operational obligations. See
[PostgreSQL operations](https://github.com/kenn-io/kata/blob/b70a4306464e319701df30a3b92860289613b15a/docs/operations/postgres.md).

## Recommendation (inference)

Factor Kata into the reuse decision. PostgreSQL and Kata are different layers:
PostgreSQL supplies storage transactions; Kata supplies an issue product that
can use those transactions. PostgreSQL alone is sufficient storage but does
not provide our orchestration or a ready-made issue interface. Temporal and
the broker still supply the durable execution contracts in the RFC.

Prefer an optional planning adapter first: Kata issues carry proposals,
dependencies, attention and user-facing summaries; the broker admits immutable
approved revisions and owns attempts, runtime bindings, reservations,
reconciliation, messages and exact-candidate acceptance. Current Git tracker
policy remains effective. Sharing a server does not make separate API mutations
atomic. Do not independently make both an issue claim and broker ownership
canonical. Treat issue close as a completion claim unless accepted proof is
explicitly bound through the broker.

Before adopting, exercise a pinned release for immutable revision capture,
claim races, stale credential mutations, event replay and duplication, close
policy, upgrade/restore, projection feedback loops and supported integration
API. Rust cannot assume the Go embedding hooks are available over the remote
API. A future Go broker choice could make embedding worth investigating, but
this assessment does not justify changing languages solely for that option.

Accept Kata if the adapter preserves one execution authority and materially
avoids duplicating ledger/UI work. Defer it if satisfying our authority boundary
requires a fork or dual authoritative state. Either outcome preserves the
PostgreSQL + Temporal kernel. These are proposed decisions, not exercised Kata
runtime guarantees.
