# State: agentic-engineering

<!-- effort:machine:start -->
```json
{
  "base_branch": "main",
  "base_commit": "5bbcc850a39cdefb3b8891915b944adae5dea88c",
  "dehydrated": "2026-10-07T09:40:29Z",
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
      "branch": "main",
      "dirty": [],
      "head": "5bbcc850a39cdefb3b8891915b944adae5dea88c",
      "path": "/data02/home/philip.yang/workspace/corenius",
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

2026-09-30, later: the owner landed the AIME work (`c1f458b`), so `check.sh` runs cargo by default. The owner then adopted the lead's
recommendations on every open decision (`docs/practice/decisions/2026-09-30-policy-defaults.md`): F12 went to Tier 3 but the spike
answered the question and F12 was built (`worker.py launch --sandbox insula --capsule ...`, 23 worker tests, live probe: escape writes fail
with EROFS); milano `claude/scm-retry-after-lost-race` was integrated through `scm integrate --gate repo` (143 tests) and pushed,
`origin/diziet/mainline` is `f8c0120`, worktree and branch removed with `scm finish`; the marker rename is decided
(`corenius-agentic-workflow`, `corenius/mainline`) but not executed. Corenius has no remote and none exists on the code host.

## Next actions

1. Tier 2 in the spec's order: F17, F16, F15, F14, F11, F13.
2. Fix `wt.py gc` so it skips a worktree that a live process uses (see the decision record limits).
3. Write the adopted policy defaults into `docs/agents/` (version bump plus manifest bump, sync tests), and the marker migration in
   `tools/sync_agentic_workflow.py` (recognise both markers for one version).
4. Assign every repo a profile in the manifest (`harness`, `product`, `docs`, `vendored`) with its merge-authority setting.
5. Find out why `mh3-cxx-engine` fails in the benchmark (fleet 0/3, solo 1/3); the runner now saves the verifier tail.

## Open questions

- Corenius remote: the owner creates `philip.yang/corenius` on `code.byted.org` (none exists) and does the first push by hand; until then
  `land.py --no-push`.

## Traps

- LLM supervisors do not block; use `worker.py supervise`.
- Stale worktrees (`corenius-worker-mechanism`, `corenius-multi-agent-bench`, ...) hold old copies of files; edit only in the main
  checkout or the lead branch's worktree. (The lead once edited `test_worker.py` in a stale worktree by mistake and reverted it.)
- `milano`: the repo gate needs the Insula rootfs in `~/.cache/milano/rootfs`; when it is missing the launcher exits 1 silently. Run
  `run_local_insula.sh -- scripts/rootfs/run_pipeline.sh rootfs-build` first.
- Only 3 of 53 milano claims were checked against source; the reference docs are
  mostly the agents' reading.

## Do not redo

- The collaboration spike and simulator (`experiments/collab/`), the sibling
  references, and the policy fleet sync are done.
