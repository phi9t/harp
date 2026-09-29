# rm-judged-by-target-not-text

Date: 20260929
Supersedes:

## Decision

`worker.py` no longer treats a recursive or forced `rm` as a violation because of its text. `rm_findings()` reads
the targets: a target outside the worktree (outside absolute path, `..`, `~`/`$HOME`, `/`) is a VIOLATION
(`rm-outside-worktree`); anything else is a NOTE (`rm-recursive`) that does not change the verdict. Deleting a
tracked file still fails through the existing git-state check (`unexpected-change`). `curl`/`wget`, `pip install`
and git write commands stay text violations. The worker prompt now asks for a fresh `mktemp -d` per experiment.
Old runs that recorded the legacy `rm` regex are re-judged with the new rule. A re-run keeps the earlier verdict
as `verdict.prev.json`.

## Why

The first research workers (2026-09-29, `research-tasks`, `research-insula`) were marked VIOLATED. Every flagged
command was an `rm -rf` of a scratch directory inside the worker's own evidence folder, or text inside a file it
was reading. `git status` showed only the two allowed output paths. The task files I wrote told workers to build
and reset scratch experiments, so my instructions contradicted the boundary. Text cannot decide `rm -rf "$RUN"`;
the effect can be checked from git and file state.

## Consequences

- `research-tasks` re-judged: VIOLATED -> DEGRADED (5 NOTEs, plus stderr from one command the sandbox blocked).
  The old verdict is in `verdict.prev.json` in its run dir. This is the lead's ruling with evidence, not a silent overwrite.
- Loosening a supervisor rule mid-run changes what later cycles report; it is landed only after the running
  research workers finish.
- `rm` of a path outside the worktree that the sandbox would refuse anyway is still flagged, so an attempt is visible.
- Tests: 5 new cases in `tools/agents/test_worker.py` (16 total, pass).
