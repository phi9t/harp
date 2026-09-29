# State: agentic-engineering

<!-- effort:machine:start -->
<!-- effort:machine:end -->

## Where we are

Milestone A (feature set) is landed as 03368f4. Milestone B has started: the
effort mechanism (`tools/agents/effort.py`, 11 tests) and policy 2026.09.29.3
exist on branch `claude/durable-efforts`. `worker.py` (F1, F2) has 10 offline
tests but has never run against the real `traecli`.

## Next actions

1. Land `claude/durable-efforts` by fast-forward once the owner has read it.
2. Run `worker.py launch` and `supervise` against the real `traecli`; then write
   F3 (`.claude/agents/worker-supervisor.md`, Haiku) and F4 (run-worker skill,
   `.claude/settings.json`); one supervised end-to-end run.
3. F5 and F6 (worktree lifecycle, landing with gates), then F7-F10.

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
