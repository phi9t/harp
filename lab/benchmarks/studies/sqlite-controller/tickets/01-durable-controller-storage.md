# 01 durable-controller-storage: Durable local controller storage

Type: task
Status: resolved
Owner: sqlite-controller-01
Blocked by:
Claimed:
Route: gated architectural work
Spec: ../SPEC.md

## Outcome

A public local controller command seam backed solely by SQLite can create pinned tasks, admit at most two worker attempts, retain messages/decisions and recover pending operations after process restart without PostgreSQL or Temporal running.

## Requirements

Implement a fresh-project SQLite schema and migrations; configured durability and foreign keys; authenticated command principals; expected revisions; idempotency with argument mismatch rejection; atomic task/slot/attempt/operation/event transitions; resumable event queries and restart reload. A single controller owns mutations. Keep runtime dispatch as an explicit journal boundary, not in a database transaction. Preserve legacy backend code and existing evidence.

## Exclusions

Live provider execution, physical landing, dashboard UI, automatic import of old PostgreSQL state, financial budgets, multiple controllers or network-filesystem databases.

## Evidence

Visible red-green public API proof against real temporary SQLite databases: two successful slot admissions and third refused; same-task racing claims admit once; duplicate commands replay the same receipt; changed duplicate arguments fail; stale revision fails; interrupted transaction exposes all or none; restart retains pending operation, questions and decisions; no PostgreSQL/Temporal service needed. Record tested source and commands. Independent Standards/Spec review required before acceptance.

## Handoff

Owner implementation lead must claim an isolated worktree before edits, reconcile this new milestone with its existing work, and sequence dependent tickets from SPEC. No inherited worker ownership may be overridden silently.

## Acceptance

- [x] Real SQLite public commands enforce two occupied slots, same-task racing admission, scoped authorization and replay/mismatch/revision checks.
- [x] Interrupted admission is all-or-none; restart retains pending operations, occupied capacity, questions and decisions.
- [x] Effective WAL, synchronous FULL and foreign keys are validated; independent Standards/Spec and branch review pass.

## Comments

- 2026-10-04T03:53:26Z codex-design: Adopted owner commits9d90222/d18e413/f775734; normalized tracker fields without changing required scope. Controller-first sequencing is binding; Matt route fallback uses the repository crosswalk.
- 2026-10-04T03:53:26Z sqlite-controller-01: claimed until 2026-10-05T03:53:26Z

- 2026-10-04T05:14:35Z codex-design: Accepted exact15ae62c after independent Standards/Spec and scoped branch635 repair review, root24-test/fmt/clippy proof. See ../evidence/01/root-verification/acceptance.md; historical rejected candidates retained. Storage qualification only; source mainline unchanged.
- 2026-10-04T05:14:35Z sqlite-controller-01: resolved
