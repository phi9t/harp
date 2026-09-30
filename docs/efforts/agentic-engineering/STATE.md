# State: agentic-engineering

<!-- effort:machine:start -->
```json
{
  "base_branch": "main",
  "base_commit": "a76603d9d59c5cfef29b69b31747eb5abf85a314",
  "dehydrated": "2026-09-30T08:28:50Z",
  "effort": "agentic-engineering",
  "runs": [
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/mb-claims-20260930T081121Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/mb-evidence-2-20260930T082134Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/mb-evidence-20260930T081121Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/mb-land-20260930T081120Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/mb-msg-20260930T081121Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/mb-tracker-20260930T081121Z",
      "verdict": "CLEAN"
    },
    {
      "exit_code": 0,
      "run_dir": "/data02/home/philip.yang/.cache/corenius/workers/mb-wt-core-20260930T081120Z",
      "verdict": "CLEAN"
    }
  ],
  "worktrees": [
    {
      "branch": "claude/durable-efforts",
      "dirty": [],
      "head": "e0d55e2fbd196ca00365426c7fca55804d86a726",
      "path": "/data02/home/philip.yang/workspace/corenius-efforts",
      "pushed": false
    },
    {
      "branch": "claude/milestone-b",
      "dirty": [],
      "head": "90135138e78ef11b2a7b876d0715909106dee579",
      "path": "/data02/home/philip.yang/workspace/corenius-mb",
      "pushed": false
    },
    {
      "branch": "claude/worker-mechanism",
      "dirty": [],
      "head": "03368f42253dd344c6230d8381fc855f4b7af698",
      "path": "/data02/home/philip.yang/workspace/corenius-worker-mechanism",
      "pushed": false
    }
  ]
}
```
<!-- effort:machine:end -->

## Where we are

Milestone B Tier 1 is implemented: F1-F4 earlier, and F5-F10 built by a fleet of six traecli workers (2026-09-30), each verified by the lead and used on
that run. Tools: `tools/agents/{worker,effort,wt,land,tracker,msg,evidence,claims}.py`, gate `tools/agents/check.sh`. The feature register in
`docs/superpowers/specs/2026-09-29-agentic-engineering-features.md` is updated. Details, per-ticket verification, the rejected evidence round, worker run
evidence digests and limits: `decisions/20260930-milestone-b-fleet.md`. Tier 2 (F11-F17) is not started.

## Next actions

1. Land `claude/milestone-b` (if not already on main) with
   `python3 tools/agents/land.py claude/milestone-b --gate tools/agents/check.sh --no-push --repo <main checkout>`.
2. Tier 2 in the spec's order: F17, F16, F15, F14, F11; F13 and F12 wait on the AIME decision.
3. Fix `wt.py gc` so it skips a worktree that a live process uses (see the decision record limits).
4. Push and merge authority per repo are still the owner's decisions; nothing has been pushed.

## Open questions

- The uncommitted AIME work in the corenius main checkout: commit it or move it
  to its own branch? F13 and the gate command depend on it.
- Is F12 (provider sandbox) required for Milestone B?
- Is F1-F10 the Milestone B minimum?
- Milano: may `claude/scm-retry-after-lost-race` be integrated and pushed?

## Traps

- The corenius main checkout has another agent's uncommitted files (README.md,
  src/cli.rs, src/eval.rs, src/provider/acp.rs, three test files). Never edit,
  stage or move them.
- LLM supervisors do not block; use `worker.py supervise`.
- Only 3 of 53 milano claims were checked against source; the reference docs are
  mostly the agents' reading.

## Do not redo

- The collaboration spike and simulator (`experiments/collab/`), the sibling
  references, and the policy fleet sync are done.
