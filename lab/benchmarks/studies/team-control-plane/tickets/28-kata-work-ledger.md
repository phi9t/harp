# 28 kata-work-ledger: qualified first-class ledger service adapter

Type: task
Status: open
Owner:
Blocked by: 17, 26, 27
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Implement Kata as first-class work ledger through version-pinned qualified APIs
and events, with independently owned persistence. Historical17/18 deferral is
preserved; this ticket implements the later owner correction. Follow
[v0.2 reconciliation](../v02-reconciliation.md).
Capture immutable source revisions into admitted intent; claim/close/hierarchy
and projections cannot grant execution or exact acceptance.

Journal each decomposition mutation before I/O; a ticket/edge bundle is
partially completable, not atomic across services. Reconcile uncertain create,
amend/edge/projection replies under measured keys/preconditions before retry.
Qualify cursor identity/order/retention, deduplication, snapshot/high-watermark
resync, bounded backpressure and projection-origin suppression.

## Interface contract

Adapter supports exact work snapshot/revision reads, journaled decomposition,
cursor-based ingestion and broker-event projections. Outcomes distinguish
confirmed, known refusal, outcome_unknown and resync_required. WorkRef includes
Kata deployment/tenant/project identity. Every admitted bundle item/edge pins
source revision and operation result; incomplete bundle blocks admission.

## Exclusions

No direct Kata-table reads/writes, shared transaction claim, repository Markdown
tracker migration/hooks, second execution authority or unqualified API claims.
Kata outage cannot cancel already admitted work or erase retained intent.

## Acceptance

- [ ] Pin actual release/schema/profile and execute real service/API RED/GREEN; compare measured capabilities with17's retained evidence.
- [ ] Crash/lost reply after create and edge mutation reconciles the same bundle; changed key payload conflicts; partial bundle remains visible and cannot launch work.
- [ ] Source revision edits/deletion/tombstones, closed tickets and dependency relationship changes cannot silently change admitted intent or acceptance.
- [ ] Duplicate/reordered events apply once; cursor retention gap recovers consistent snapshot+high-watermark or blocks unsafe ingestion/admission.
- [ ] Outage preserves admitted execution; fresh admission requiring unavailable source evidence refuses explicitly; projections retry without loops or fabricated grants.
- [ ] Flood/backpressure tests retain control and acceptance responsiveness; replay/restart preserves durable cursor and uncertain mutation references.
- [ ] Independent conformance/branch review and exact API/source/evidence manifests accepted; external assertions marked measured or unverified.

## Comments

- 2026-10-03T17:30:00Z lead: First-class target supersedes historical optional adapter18 without reopening its scoped deferred disposition.
