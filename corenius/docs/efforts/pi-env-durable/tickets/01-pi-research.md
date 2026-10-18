# 01 pi-research: Research Pi Durable and Pi Env reuse in Corenius

Type: task
Status: resolved
Owner: codex
Blocked by:
Claimed:

## Acceptance
- [x] Pin both repositories and distinguish main from published package versions.
- [x] Source-review both packages, including APIs, storage/recovery, effects,
  security, deployment, tests, license and limitations.
- [x] Compare against current Corenius source and qualification status.
- [x] Publish cited source reviews and a ranked integration recommendation with
  bounded next-probe acceptance criteria.
- [x] Record actual verification evidence and lint the research ticket.

## Scope and route

Standalone research selected by the owner's explicit `ask-matt` invocation.
No product implementation or architectural approval is implied. See
`../EFFORT.md` for acceptance scope and authority boundaries. Research agents
write only their isolated `research/pi-env` and `research/pi-durable` worktrees;
the lead integrates their reports into `codex/pi-env-deep-dive`.

## Result

Complete: `docs/spikes/pi-env-durable/README.md` and two package source reviews.
The report pins sources, distinguishes main/release and implemented/tested/inferred
claims, compares current Corenius code and qualification, and proposes concrete
adapter boundaries and six bounded follow-up probes.

Verification: 301 selected Durable tests, 7 Env Rust tests, Linux protocol/client
probes, and independent source review. Setup differences and untested platforms
are explicit in the reports. Citation range corrections are integrated. Final
tracker/link/diff checks are retained in `../evidence/document-checks.json`.

## Comments
- 2026-10-06T19:14:48Z codex: claimed until 2026-10-07T19:14:48Z
- 2026-10-06T19:23:04Z codex: resolved
