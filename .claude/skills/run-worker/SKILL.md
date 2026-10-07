---
name: run-worker
description: Launch a traecli worker in its own private git workspace and supervise it (lead/supervisor/worker). Use when delegating a bounded task to a worker agent.
---

# Run a worker

You are the lead. You own intent, the launch contract, verification, and the
decision to land. Design: `docs/practice/lead-supervisor-worker.md`.

1. **Write the task file** outside the repo (scratchpad): the task, exact files
   to read, and the outputs. The launcher adds the standard rules.
2. **Fix the boundary**: choose `--allow <path>` for every path the worker may
   write (repeat the flag), `--protect <repo>` for sibling repos it must not
   change. Everything else is a violation.
3. **Launch** (spends model budget; only with the owner's authorization):
   `python3 tools/agents/worker.py launch --task FILE --name NAME --allow PATH --repo .`
   Defaults: GPT-5.5, xhigh reasoning, `workspace-write`, 2700 s cap, 300 s stall.
   It prints `run_dir`, `worktree`, `branch`, `pid`. The `worktree` is a
   private local clone, not a linked worktree in the shared repository. Normal
   provider `workspace-write` launches put the private git directory under
   `run_dir` and pass it as an additional writable root; `insula` capsule
   launches keep embedded `.git` metadata because only `/workspace` is mounted.
   Both layouts let sandboxed workers commit without exposing the shared
   repository's Git metadata. The launcher also sets `TMPDIR`, `TMUX_TMPDIR`
   and `TMPPREFIX` under `run_dir/tmp`; workers must not use the host's `/tmp`.
   For effort tickets, prefer the lead launcher:
   `python3 tools/agents/lead_launch.py launch EFFORT NN --interactive --tmux-session NAME`.
   Interactive launch starts the provider's terminal command in the private
   clone, one worker per tmux window, and records the tmux target in `run.json`.
   It passes a per-workspace trust override for `traecli` and `codex` so a
   fresh private clone does not stop at the folder-trust prompt, and records the
   override in `run.json`: `-c
   projects={"<worker workspace>"={trust_level="trusted"}}`.
4. **Supervise**: spawn the `worker-supervisor` subagent with the run dir. Keep
   your own check going too: a background
   `python3 tools/agents/worker.py supervise RUN_DIR --max-wait N` blocks on the
   process and is the source of truth; do not trust an LLM's timing or
   completion claim.
5. **Read the verdict first** (`verdict.json`, exit 0 CLEAN / 1 DEGRADED /
   2 VIOLATED). On VIOLATED, do not land; read the findings. Per
   `docs/efforts/lead-worker-readiness/decisions/20261005-eight-answers.md`, a
   worker may commit on its own branch. Exit mode is the default: readiness
   needs process exit 0, a clean tree, and a commit after the recorded base.
   Marker mode is for interactive sessions that may stay alive:
   `--handoff-mode marker --handoff-marker 'ticket NN: ready for review'`.
   There readiness needs the marker subject at the private branch tip, a clean
   tree, and a commit after the recorded base. In both modes the first ready
   observation pins a Candidate with commit, tree, base and brief digest and
   publishes that exact commit into the shared repository under the worker
   branch. Later private branch-tip movement is reported as `candidate-moved`
   and does not replace the Candidate; the shared worker branch stays pinned
   until a later ready Candidate is published.
6. **Verify the artifacts yourself**: read `last-message.md` and the outputs,
   run the checks the task named, sample claims against source. A CLEAN verdict
   means the boundary held, not that the work is right.
7. **Monitor and deliver rework**: use
   `python3 tools/agents/lead_launch.py open-session EFFORT --tmux-session NAME`
   for the ticket and worker monitor windows. Use
   `monitor-tickets EFFORT --json` or `monitor-workers EFFORT --json` for
   one-shot machine-readable state. `monitor-tickets` reports
   `blocked-on-prompt` when a visible runtime prompt is waiting for keyboard
   input; `wait` wakes on that state and includes the prompt text. For an
   interactive worker, `lead_launch.py rework RUN_DIR --findings-file FILE`
   types the findings pointer into that worker's pane and records the sent and
   acknowledged receipt. Runtime transcripts are recorded once an event stream
   reports the session id or, for interactive `traecli` and `codex` sessions,
   once the runtime session store has a session-meta record for the worker
   workspace.
8. **Land** only after verifying the worker's Candidate. The worker still must
   not push, merge, move another branch, or rewrite history below its base.
9. Record the run in the effort: `effort.py dehydrate <effort> --run RUN_DIR`.

The interim coordinator script saved under the lead-worker-readiness evidence
directory was a bootstrap aid. Do not use it for new launches; the supported
paths are `worker.py` and `lead_launch.py`.
