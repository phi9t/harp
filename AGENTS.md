# AGENTS.md

Corenius is a standalone Rust terminal agent prototype (turn-scoped runtime,
Codex and TraeCLI ACP providers, permissioned tools, an `insula` bubblewrap
boundary) and the owner of agentic-engineering practice for the workspace.

## Where things are

- `README.md`: commands and the AIME eval harness.
- `docs/agents/`: the canonical agentic-engineering policy that sibling repos
  copy (`agentic-engineering.md`, `skill-orchestration.md`, the manifest
  `agentic-workflow-repos.json`). `tools/sync_agentic_workflow.py --check` reports
  drift; `--apply` repairs it. Edit policy here, never in a sibling copy.
- `docs/practice/`: charter, imports of milano, vaso and observatory practice,
  and the open decisions.
- `docs/reference/`: what each sibling repo is for and how it is designed, plus
  `agentic-inventory/` (policy, skills, products, and the rules agents follow).
- `tools/agents/`: the lead/worker toolbox. `worker.py` (launch a traecli worker, `--capsule` to run it in a capsule; `supervise`), `effort.py`
  (durable efforts), `wt.py` (new/status/finish/gc agent worktrees), `land.py` (gated landing: rebase, gate the exact tip, push first, then
  fast-forward mainline; also `wt.py land`), `tracker.py` (tickets and claims under `docs/efforts/*/tickets/`), `msg.py` (handoff messages),
  `evidence.py` (content-addressed run records), `claims.py` (sample and record claim checks). The repo gate is `tools/agents/check.sh`.
- `docs/efforts/`: durable efforts (charter, decisions, state). Session start:
  `python3 tools/agents/effort.py rehydrate <effort>`; session end: `dehydrate`.
  Working detail goes in gitignored `.scratch/<effort>/`.
- `bench/` and `tools/insula/`: the multi-agent benchmark (validated tasks, harness, results) and the layered
  capsule builder every step runs in. Start at `bench/README.md`; validate with `python3 bench/harness/validate.py`;
  watch runs live with `bench/tmuxh/bench-tmux` (a tmux window per run, a pane per agent, a dashboard).
- `docs/spikes/collab/` and `experiments/collab/`: the multi-agent collaboration
  design space and the simulator that measures it.

## Rules for working here

- One writer per working tree. Do your work in your own git worktree; do not
  edit, stage or move files in a checkout another agent is using.
- Land by fast-forward only; never rebase, force-push or delete a shared branch.
- Policy changes bump the version header in the edited file and the manifest
  version together, and `tools/test_sync_agentic_workflow.py` must pass.
- Reference and inventory documents state a commit read and mark inference; keep
  that discipline when editing them.

<!-- ultron-agentic-workflow:start -->
## Agentic engineering workflow

**Mandatory:** Read and follow `CONSTITUTION.md` before acting. Before planning,
building, fixing, or changing code, read and follow
`docs/agents/agentic-engineering.md`. Direct user instructions and more specific
repository guidance take precedence.
<!-- ultron-agentic-workflow:end -->
