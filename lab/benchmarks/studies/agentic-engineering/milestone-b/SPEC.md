# Milestone B Tier 1 (F5-F10), built by a fleet

Source of the features: `docs/superpowers/specs/2026-09-29-agentic-engineering-features.md` (F5 to F10). This document is the work
breakdown for a fleet of traecli workers, the rules they follow, and the done criteria the lead checks. Decided with the owner on
2026-09-30: scope is Tier 1; the lead verifies, then lands locally by fast-forward; nothing is pushed.

## Roles

- **Lead** (interactive agent): owns contracts, tickets, shared files, the gate script, verification and landing.
- **Workers** (traecli GPT-5.5, xhigh, inside the `agent` capsule): one per ticket, each in its own worktree, each allowed to write only the
  files its ticket names. Workers never commit, never land, never edit shared files.
- **Supervisors** (Haiku): relay the deterministic `worker.py supervise` verdict. The verdict says the boundary held, not that the work is right.

## Tickets

| Id | Slug | Feature | Files the worker may write |
| --- | --- | --- | --- |
| 01 | wt-core | F5 worktree lifecycle | `tools/agents/wt.py`, `test_wt.py`, `checks.d/10-wt.sh` |
| 02 | land | F6 landing with gates | `tools/agents/land.py`, `test_land.py`, `checks.d/20-land.sh` |
| 03 | tracker | F7 tickets and claims | `tools/agents/tracker.py`, `test_tracker.py`, `checks.d/30-tracker.sh` |
| 04 | msg | F8 messages and handoffs | `tools/agents/msg.py`, `test_msg.py`, `checks.d/40-msg.sh` |
| 05 | evidence | F9 evidence records | `tools/agents/evidence.py`, `test_evidence.py`, `checks.d/50-evidence.sh` |
| 06 | claims | F10 claim checking | `tools/agents/claims.py`, `test_claims.py`, `checks.d/60-claims.sh` |

The ticket files are `docs/efforts/agentic-engineering/tickets/NN-slug.md` (format: `contracts/ticket-format.md`). They are the worker task text.

## Contracts (written first, so workers build against a fixed interface)

- `contracts/wt-dispatch.md`: how `wt.py` loads `land` and how `land.py` plugs in (tickets 01, 02).
- `contracts/evidence-record.md`: the evidence record schema and Python API (tickets 05, 06).
- `contracts/ticket-format.md`: the tracked ticket file format (ticket 03, and every ticket file here).

## Rules for every worker

1. Write only the files your ticket lists. Shared files (`AGENTS.md`, `check.sh`, specs, other tickets, other tools) are lead-owned.
2. Standard library only, Python 3.11, no network. Match the style of `tools/agents/worker.py` and `effort.py`. Import helpers from `worker.py`
   (`git`, `main_worktree`, `now`, `WorkerError`) instead of copying them.
3. Tests are `unittest`, run with `python3 tools/agents/test_<name>.py`, and use throwaway git repos under a temp directory (your worktree's own `.git`
   is not visible inside the sandbox, so never run git against the real repo; set `user.name`/`user.email` with `-c` in test repos).
4. `checks.d/NN-name.sh` is one small `sh` script that runs your tool's acceptance checks (it is run by `tools/agents/check.sh` from the repo root).
5. Do not commit, push, merge or switch branches. Finish with a short message: what you built, how to run it, what you did not check.
6. Every acceptance item in your ticket must be covered by a test that fails without the feature. Say which test covers which item.

## Done criteria (lead)

A ticket is done when: the verdict is CLEAN (or DEGRADED with stderr only) and the changed files are exactly the allowed ones; the worker's tests and `tools/agents/check.sh`
pass; the lead has reproduced the acceptance scenarios outside the sandbox (including the ones a worker cannot run against the real repo); sampled claims in the worker's final
message match the source; it is landed by fast-forward. After landing, each tool is used on this very run (evidence records for the worker runs, tracker list/lint over these
tickets, claim sampling of the lead's verification notes, handoff messages), and the feature register in the features spec is updated.
