# Collab Lite state

<!-- effort:machine:start -->
```json
{
  "base_branch": "main",
  "base_commit": "be9de21f3f5287bc7c8ab3cb01e581328eb20b1a",
  "dehydrated": "2026-10-11T22:30:09Z",
  "effort": "collab-lite",
  "runs": [],
  "worktrees": [
    {
      "branch": "codex/collab-lite-mainline",
      "dirty": [],
      "head": "240ca4817970fda7caec23f77b35afa7764b34f6",
      "path": "/data02/home/philip.yang/workspace/corenius-collab-lite-mainline",
      "pushed": false
    }
  ]
}
```
<!-- effort:machine:end -->

Status: accepted; ready for authorized local mainline landing
Updated: 2026-10-11
Base read: d31c3df460132e9ab87c0a52bffd949c387543e8
Mainline preparation base: be9de21f3f5287bc7c8ab3cb01e581328eb20b1a
Integration branch: codex/collab-lite-mainline
Worktree: /data02/home/philip.yang/workspace/corenius-collab-lite-mainline

## Where we are

The user approved the standalone CLI and public CLI tests, then requested
landing on Corenius mainline. Reviewed source f6ba6c8 was preserved on its
original branch and cherry-picked onto the independently landed health repair
be9de21 as Candidate 240ca48. Product hashes remain identical to review.
Acceptance is recorded through acceptance.py and ticket01 resolved through
tracker.py on the integration branch. Mainline landing remains pending.

All48 CLI tests and the updated gate-dispatch, operational-state consistency,
temporary-path, global tracker-lint and whitespace checks passed on the new
base. The prior full repository suite passed every other stage; its historical
report-placement lint failure was corrected and fresh global lint passes.
Evidence: implementation.md, review.md, verification.md, mainline-preflight.json
and acceptance-01.jsonl under evidence/. No live Collab adoption occurred.

## Next actions

Land the exact committed candidate through land.py on local main under the
shared landing lock. There is no configured remote. Record the observed
mainline transition and refresh this state after landing. The user request
explicitly delegates the required local Git actions; no PR merge is involved.

## Open questions

None for this standalone implementation. Deployment and migration are separate.

## Traps

Unattributed started landings require manual reconciliation. The CLI is local
cooperative Linux tooling; it has no hostile-code boundary, process supervision,
automatic retries, distributed execution or complete power-loss guarantee.
The retained live two-worker controller baseline remains unqualified.

## Do not redo

Do not rebase, delete preserved branches/worktrees, migrate ledgers, or repeat
unchanged native-runtime suites solely for bookkeeping. Preserve original
review and verification history and the health-repair landing.
