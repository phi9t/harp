---
name: run-worker
description: Launch a traecli worker in its own worktree and supervise it (lead/supervisor/worker). Use when delegating a bounded task to a worker agent.
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
   It prints `run_dir`, `worktree`, `branch`, `pid`.
4. **Supervise**: spawn the `worker-supervisor` subagent with the run dir. Keep
   your own check going too: a background
   `python3 tools/agents/worker.py supervise RUN_DIR --max-wait N` blocks on the
   process and is the source of truth; do not trust an LLM's timing or
   completion claim.
5. **Read the verdict first** (`verdict.json`, exit 0 CLEAN / 1 DEGRADED /
   2 VIOLATED). On VIOLATED, do not land; read the findings.
6. **Verify the artifacts yourself**: read `last-message.md` and the outputs,
   run the checks the task named, sample claims against source. A CLEAN verdict
   means the boundary held, not that the work is right.
7. **Land** only after verifying: copy the allowed outputs into your own
   worktree, commit, fast-forward. The worker never commits.
8. Record the run in the effort: `effort.py dehydrate <effort> --run RUN_DIR`.
