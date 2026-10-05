# effort.py summary

- `effort.py` manages durable efforts: tracked `docs/efforts/<slug>/` plus scratch `.scratch/<slug>/`.
- `cmd_new()` creates an `effort/<slug>` worktree/branch, durable files, keep dirs, scratch, and JSON output.
- `cmd_decide()` appends a dated decision file and rejects duplicate generated filenames.
- `cmd_dehydrate()` validates `STATE.md`, human sections, worktrees, dirty paths, and runs; writes machine JSON and commits effort state.
- `cmd_rehydrate()` validates base/worktree pointers, restores missing branch/worktree/scratch when safe, prints briefing and drift report.
- `cmd_status()` emits one JSON row per effort with status, staleness, and decision count.
- Helpers: `split_state()`, `join_state()`, `human_problems()`, `dirty_paths()`, `registered_worktrees()`, `describe_worktree()`, `read_run()`.

Exit codes:
- `OK = 0`, `WARN = 1`, `ERROR = 2`; `cmd_rehydrate()` uses `WARN` for recoverable drift and `ERROR` for unrecoverable drift.
- `main()` converts raised `WorkerError` validation failures to exit code `64`.

Test coverage in `test_effort.py`:
- `test_new_creates_durable_and_scratch()` covers `cmd_new()` output artifacts.
- `test_dehydrate_requires_human_part()` and `test_dirty_worktree_blocks_dehydrate()` cover dehydrate blockers.
- Round-trip tests cover rehydrate briefing, scratch/worktree restore, missing branch recreation, diverged/advanced branch handling, rewritten base error, and lost-run verdict warning.
- `test_decisions_are_append_only()` covers duplicate decision refusal; `test_status_lists_effort()` covers status JSON.
