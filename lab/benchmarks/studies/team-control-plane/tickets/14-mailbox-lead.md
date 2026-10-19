# 14 mailbox-lead: Scoped agent tools, durable mailbox and replaceable lead context

Type: task
Status: resolved
Owner: codex-mailbox
Blocked by: 10, 11, 13
Claimed:
Label: implementation:planned
Parent: ../PLAN.md

## Requirements

Expose team context, proposal, messaging, checkpoint, yield, submission and approval operations through the broker and MCP facade. Build a bounded mission view for replacement Codex/Claude leads, and trigger episodes on meaningful coalesced events.

Canonical behavior: [SPEC.md](../SPEC.md). Proposed planning defaults and
unresolved owner decisions remain explicit; open does not mean execution-ready.

## Interface contract

Authenticated sender overrides caller text. Message records separately track persistence, episode presentation, acknowledgment and resolution. Yield names dependencies and external Execution handles. Lead decisions are canonical records before dispatch depends on them.

## Exclusions

No chat-derived state transition, native-goal continuation or unrestricted app-server tools.

## Acceptance

- [x] Establish a failing public behavioral fixture for the required outcome.
- [x] Implement the smallest slice and record fresh passing evidence.
- [x] Race steering with turn completion; redeliver messages; forge sender/role; use stale attempt tokens; lose the lead conversation while questions remain. Pending messages remain deliverable, no synthetic acknowledgment occurs, and replacement gets identical accepted constraints/decisions. Blocked agents spend no polling turns.
- [x] Standards and Spec reviews have no unresolved blocking finding.
- [x] Record artifact/run IDs, limitations and handoff to dependent tickets.

## Execution sequence

1. Rehydrate approved intent, check blockers, claim this ticket and allocate an isolated worktree.
2. Derive session-local exact files/commands from the interfaces; approve no new architectural authority implicitly.
3. Run the acceptance fixture red, implement, then run it green; retain meaningful red/green evidence.
4. Run focused checks, applicable repository gate and independent review; at most three repair/review rounds.
5. Update durable evidence and status only when requirements are proven. Commit/push need explicit authority; human integration remains separate.

## Comments

- 2026-10-01T05:48:16Z lead: Detailed planning task created at the owner's request. No execution or architectural decision is implied.

## Integration preparation

[Read-only accepted-source preflight](../research/mailbox-integration-preflight.md) identifies lead replacement fencing, typed presentation receipts, peer disclosure scopes, atomic yield/answer wakeups and transport integration seams. It does not waive ticket13 or count as behavioral acceptance.
- 2026-10-01T10:45:35Z codex-mailbox: claimed until 2026-10-02T10:45:35Z

- 2026-10-01T10:53:02Z lead: Isolated execution begins against exact accepted13 source;0006 reserved. HTTP real-PG red planned first; supervisor15 remains separately unaccepted.

- 2026-10-01T12:21:14Z lead: Author reports real-PG/public-HTTP green exact single-use approvals, immutable proposals/decisions, scoped durable rejections and coalesced admissions. Frozen evidence, remaining artifact/expected-turn/Temporal proof and independent reviews remain pending; no acceptance or supervisor binding is implied.

- 2026-10-01T12:40:18Z lead: Peer disclosure fixture must use distinct assigned workers/tasks. An addressed message grants only sender-authorized exact artifact references without requiring task/candidate ACCEPTED status; peer task/history and unlisted digests stay denied. Author confirmed the existing accepted-only cross-task restriction was too narrow and is adding the public red/green slice. This clarifies existing SPEC messaging scope, not broader task authority.

## Answer

Implemented and accepted after independent Spec and Standards round0 PASS,
separate root-glue PASS, and fresh252-test joint gate (138 Rust/114 Python),
formatting and strict Clippy. Exact [accepted manifest](../evidence/ticket14-accepted-manifest.json)
binds source and evidence; [candidate report](../evidence/ticket14-mailbox-lead.md)
records artifact/run IDs and limitations. Provider/Supervisor physical role-neutral
assembly and external Execution validation remain ticket16; no live runtime
or restore readiness claim. No commits, pushes or production changes.
- 2026-10-01T16:38:26Z codex-mailbox: resolved
