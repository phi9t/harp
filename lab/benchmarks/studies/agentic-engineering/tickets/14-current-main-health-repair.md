# 14 current-main-health-repair: Repair current state, gate scheduling and controller deadline findings

Type: bug
Status: claimed
Owner: codex-health-repair
Blocked by: 
Claimed: 2026-10-07T17:54:46Z expires 2026-10-08T17:54:46Z

Triage: ready-for-agent

## Description

User requested diagnosis of all three findings and then said proceed. This admits one narrow controller repair round for C1 peer-wait and relay S2 without resetting previous repair allowances. No live provider calls, deployment, SQLite05/06 acceptance, commits, push or landing.

Preserve historical snapshots and reconcile current projections to archived/quiescent. Deduplicate real gate scheduling and make parked integration explicit. Correct acknowledged peer polling and relay deadline handling using actual caller regressions. Separate sole-writer worktrees implement each slice; root integrates and independent review follows.

## Acceptance

- [x] Each reproduced defect has behavioral RED/GREEN evidence at its actual call site.
- [x] Historical state and all prior repair allowances remain preserved; current projections give no obsolete launch authority.
- [x] Gate schedules each suite once and retains explicitly selectable parked integration.
- [x] Focused checks, repository checks and independent Standards/Spec review pass or limitations are named.

## Comments
- 2026-10-07T17:54:46Z codex-health-repair: claimed until 2026-10-08T17:54:46Z

- 2026-10-07T00:00:00Z codex-health-repair: date-only summary; exact author/probe times remain in tool records. Three repairs integrated on isolated uncommitted candidate; independent reviews PASS. Root71 tests pass serially; focused regressions pass; fullnative42 resource-failed and isolated affected native1 PASS. Broader qualification remains uncredited. Evidence: ../evidence/health-repair-20261007.md. No commits, push or landing.
- 2026-10-07T00:00:00Z codex-health-repair: date-only disposition; tracker resolution correctly refused because candidate is not accepted and landed. Ticket remains claimed pending publication, not a claim of mainline completion.
