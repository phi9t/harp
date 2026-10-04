# 01 — Durable local controller storage

Status: open
Blocked by: none
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
