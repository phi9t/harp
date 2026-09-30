# tmux-harness

Date: 20260930
Supersedes:

## Decision

Runs can be watched live: `bench/tmuxh/bench-tmux run ...` puts each run in its own tmux window and each agent in its own pane (a fleet run
also gets a lead pane with landing, conflicts and redos), plus a curses dashboard in window 0. It runs on a dedicated tmux server
(socket `corenius-bench`), never the user's default server. libtmux 0.62 comes from PyPI through a uv project (`bench/tmuxh/pyproject.toml`,
`uv.lock`); the rest of the benchmark stays standard library only.

## Design points that matter

- Files are the interface. The scheduler (the lead), the panes and the dashboard exchange only state files (`runs/*/state.json`,
  `units/*/state.json`, written atomically), so a crashed dashboard or pane cannot corrupt a run and everything is testable without a terminal.
- The seam is `Agent.plan()` (what to run: capsule request, command, env, net, gpu, binds, timeout) versus `Agent.execute()`. `PaneAgent`
  writes the plan as a job, a pane runs `pane.py --job`, and PaneAgent returns the same dict `Agent.run` returns. `run_solo`,
  `run_fleet_condition` and `fleet.run_fleet` are unchanged in behavior (they gained optional `rid`, `observer` and `log` arguments), so tmux
  runs produce the same result records as `run.py` (a test compares a tmux fleet run with a direct one).
- Verification and landing stay in the lead process; panes only run agents, inside capsules as before.
- Slots bound concurrent agent panes (waiting units show `queued`). A pane that dies without a result is marked `crashed` (a test kills a
  running pane). The saved pane log is plain text (ANSI stripped); colors are only in the live pane.
- `capsule.run` and `tasklib.run_in_capsule` gained an optional `on_line` streaming callback (used for the pane stream and state updates).

## Evidence (all run in this session)

- Tests: state and stream 9, pane 8, dashboard 7, scheduler 6 (real tmux on an isolated socket, real capsules), plus 5 new run tests and 1 new
  capsule test; all existing suites green. Scheduler tests: solo run = one window one pane; fleet:2 on m03 = lead + t1 + t2 + redo panes with the
  same pass/conflict/redo counts as the direct runner; slots=1 with two runs shows one running and one queued; a killed pane ends `crashed`,
  not hung; `stop()` kills only the dedicated server and removes its socket file.
- Manual: dashboard rendered in window 0 (rows, slot counts, window markers); keys j, Enter (jumped to the selected run's window), l (log tail),
  q verified through `tmux send-keys`.
- Live, real traecli GPT-5.5 in panes: `p01-slugger` solo PASS 73.5 s (2.6k output tokens); `m02-shared-interface` fleet:2 PASS 46.6 s with both
  ticket agents running in parallel (about 46 s each). The user's own tmux sessions were unchanged.

## Found on the way

- `bench/tasks/t01-log-window/environment/events.log` had never been committed (the repo's `*.log` ignore rule hid it); validation passed only in the
  worktree where the file existed. Fixed and landed separately (commit 0820b5c) with a guard test (`test_tasks_tracked.py`).

## Limits

- The dashboard shows what the agent stream reports; no dollar cost. Pane output is a view; `events.jsonl` and `result.json` are the record.
- The lead pane uses `tail -F`, which polls once a second on this machine (inotify limit), so its last line can lag a second.
- Ticket worktrees expose a `.git` file pointing outside the capsule; agents sometimes run a git command and get "not a git repository" (harmless).
