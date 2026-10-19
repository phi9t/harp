# 01 lead-wakeup: main-lead Codex review wakeup

Type: task
Status: resolved
Owner: lead-monitor-wake
Blocked by:
Claimed:
Spec: ../SPEC.md
Route: narrow bootstrap implementation of owner-selected15-minute main-lead wake cadence; product monitoring remains separately gated.

Implement a standalone standard-library Python wake helper in tools/agents/monitor_wake.py, in an isolated author worktree. It supports init, tick, begin, finish, status and stop, with a private SQLite journal and OS writer lock. Tick records durable intent before invoking the pinned installed Codex CLI's existing-thread queue with an argv list and timeout. Exact target is this main lead, not the implementation coordinator. Do not invoke tmux, start a model turn directly, clone credentials, edit Codex storage or manipulate workers.

At most one prepared/queued/unknown/reviewing wake exists. Ambiguous dispatch remains unknown and suppresses duplicate ticks; operator/main lead may inspect and explicitly resolve it. Validate exact lead identity from CODEX_THREAD_ID for begin/finish. A stable wake ID is included in the prompt. Successful CLI queue output must confirm the exact target and an identifier; exit-zero alone is insufficient. No unbounded output, automatic repeated queue attempts or shell interpolation. Initialization refuses replacement of existing configuration.

The prompt names the canonical monitor spec, private assessment ledger, execution coordinator state path and current Codex adapter. It tells the lead to inspect current assignments, review new visible actions in batches against goals/specs/tickets/plan, preserve history gaps, intervene through the coordinator when supported, record evidence/cursors, begin/finish the wake and stay quiet unless meaningful drift/completion/failure/human attention occurs. No financial budget feature or full controller integration.

## Acceptance

- [x] Subprocess-boundary tests prove two racing ticks yield at most one actual queue request, successful delivery receipts, timeout/ambiguous acknowledgment suppression, wrong-thread refusal, pending/reviewing coalescing, completion enabling the next tick, stopped configuration refusal and restart reconciliation without duplicate queue.
- [x] Tests use an actual owned fake CLI process for fault cases, not production credentials.
- [x] Real existing-main-thread queue submission and supervised timer installation have root-owned operational qualification after code review.

Root retains responsibility for docs, systemd timer/service installation, actual delivery, independent review and FF-only integration. Do not edit root's checkout, install a timer, queue any real messages, push/rebase or land main. Final handoff: clean commit, exact test commands/results, risks and state protocol.

## Comments

- 2026-10-04T18:00:00Z sqlite-baseline: owner selected the exact main lead and approximately15-minute cadence. Separate-context author owns codex/monitor-wakeup; root retains installation and qualification. Native queue probe received exact-thread receipt, not a completed review.
- 2026-10-04T18:25:00Z sqlite-baseline: narrow wake-helper acceptance at b034844 with independent Standards/Spec PASS, defect review647 PASS, fresh root9 focused and99 Python checks PASS. Local unit verified/enabled; exact lead queue receipt recorded. Automatic presentation/completed analysis remain separately unqualified; no full production monitor or mainline acceptance implied.
