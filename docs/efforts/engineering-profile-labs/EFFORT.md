# Engineering profile learning labs

Status: independently reviewed; user authorized local landing
Base: main
Integration branch: main
Source: `b7bee20e00c822feec48362a1dbd8d62fe9c3641`
Branch: `codex/engineering-postgres-temporal-labs`

## Intent

Experiment in Corenius before any Trondheim integration. Produce detailed,
executable PostgreSQL, Temporal, TraeCLI, Codex CLI and Codex app-server learning
labs using existing native fixtures and installed runtimes.
Explain transactional claims, idempotency, revision checks, fencing, durable
waits, worker restart, rollover and retained-history replay. Compare CLI process
completion with persistent app-server thread and turn events.

## Scope and authority

This private clone owns lab documentation and a bounded fixture runner. Reuse
`crates/team-control-plane` at the recorded source revision. The source checkout
was untouched during development. No product semantics, user database migrations,
deployment, GPU run or secret-bearing environment capture is authorized. The
user's subsequent request, "land these labs first", authorizes committing and
landing this reviewed lab slice on local main. No remote is configured.
The existing team-control-plane is parked; passing these labs is not production
readiness. Manual STATE updates override the auto-committing dehydrate command.

## Engineering route

Bounded learning task under the gated effort record: distributed and concurrency
invariants exclude the short path. The user authorized Corenius-first experiments;
the implementation reuses existing public fixtures as the learning seam.
No new production architecture decision is introduced. The root read the
explicitly requested `ask-matt` router; command-only phases use the repository
crosswalk without claiming slash-command invocation. The parent coordinates
independent review. Existing behavior tests are reused without modifying product
code. Native event acceptance uses focused model-free red/green fixtures.

## Acceptance

- Document exact source anchors and runnable commands for all five labs.
- Run real disposable loopback PostgreSQL and Temporal fixtures with a private
  Cargo target directory, locked dependencies and offline Cargo resolution.
- Retain fresh exit status, assertions, source identity, versions, artifact hashes,
  cleanup evidence and explicit limitations.
- Integrate a detailed lab only after its fresh selected tests pass.
- Keep resumable STATE without committing and present changes for human review.

## Milestones

1. PostgreSQL transactional kernel labs and fresh native fixture evidence.
2. Temporal wait, restart, rollover and replay labs and fresh native evidence.
3. Native CLI/protocol labs, bounded live observations and model-free acceptance tests.
4. Evidence reduction, independent review handoff and manual STATE.

## Native runtime integration

The separately developed native slice is integrated at
[native/README.md](native/README.md). Its evidence classes and qualification gaps
remain independent of passing database tests.
