# Harp runtime integration decisions

Status: resolved
Owner: Codex and project owner
Base: 1418455d
Scope: design and specification only; implementation has not been authorized.

## Settled first milestone

The owner chose one real coding task end to end: an isolated worker produces
an exact candidate, verification checks that candidate, and the owner explicitly
approves local landing. Restart recovery and cancellation are part of this
milestone. Concurrent workers follow later.

## Existing constraints

Harp is the destination product. Imported Corenius branches/worktree captures
remain preserved. PostgreSQL/Temporal adoption is excluded. Git-managed Markdown
owns work intent; retained runtime state does not introduce another backlog.
Math, proofs, SICP, education and PDF publication remain functional. Atlas and
the RSI article archive remain retired. Study relocation and Cargo/toolchain
unification are separate follow-ups.

## Settled runtime ownership (updated by owner)

The owner replaced the earlier Rust-controller selection with Corenius's existing
lightweight Python + SQLite durable runtime. It is a call-journal replay engine,
not a Temporal SDK/server. Extend that foundation to own the complete integrated
worker lifecycle, including the responsibilities currently split across older
controllers. Those other controller efforts are superseded as callers migrate.
The PostgreSQL-backed version is dropped from the active product.

Harp provides a CLI-first operator interface. The runtime and workers execute on
Linux at devx_b200_dbg. Both Codex and Trae are required from milestone one.
Qualify one worker, then two concurrent workers, then up to four. Other Harp
controller workflows are migration inputs, not permanent parallel authorities.
The exact phase of each caller retirement belongs in the specification.

Git-managed Markdown retains work intent. SQLite holds admitted immutable input
identities and operational state. Human approval remains distinct from agent
roles and must bind the exact verified candidate.

## Source facts constraining the design

- The selected lightweight runtime has configurable/default capacity four, but
  four live coding workers and dual-provider adapters are not yet qualified.
- It presently runs demonstration activities. Provider adapters, task intake,
  Git verification/approval/landing and migration of old callers are new work.
- On runtime death, guardians clean running children. Completed operations replay
  from receipts; a started manual-recovery activity becomes uncertain rather than
  silently being rerun. Automatic live conversation resumption is not implemented.
- The imported Rust Git backend moves a ref but does not synchronize a checked-out
  primary worktree. A successful local landing must include the actual destination
  checkout state, with dirty/stale targets preserved.

## Settled recovery, acceptance and lifetime

The owner selected safe replay of completed steps, cleanup of interrupted workers
and explicit reconciliation of ambiguous effects before another attempt. Live
provider-session resumption is not required. The public Harp CLI is the acceptance
seam: real Git/state/process tests, deterministic failure injection and real coding
tasks with each provider, then mixed two- and four-worker runs.

The CLI explicitly starts and stops a project-scoped runtime. Durable state remains
for later recovery; no installed always-on service is part of this milestone.
The owner subsequently instructed writing the spec with these settled decisions.
See [the ready specification](spec.md).

## Supplied reference inspection

The supplied insula_durable_reference archive has SHA-256
30f55eef3f5aa10b84df0c9b8d31aa0ec28300dc08da7acfd53ccd8be88f180b,
matching the Corenius provenance. All 16 manifest entries matched and all nine
Python files parsed. Source comparison confirmed direct lineage plus substantive
Corenius safety/Insula/cgroup improvements. No archive code or tests were executed
during this inspection; recorded 35/73-case results remain historical evidence.
DBOS was an alternative, not an implemented dependency. A separate Collab Lite
Rust archive mentioned in pasted provenance was not supplied or verified.
