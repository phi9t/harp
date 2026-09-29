# Lead, supervisor, worker: the working form factor

Decided 2026-09-29 by the owner as the shape multi-agent work follows. First
trial: `traecli exec` as worker, a Haiku subagent as supervisor, the
interactive agent as lead. This note records the form factor and what the
trial exposes. Trial 1 was still running when this was written; its results
go under "Trial log".

## Roles

| Role | Is | Owns | May not |
| --- | --- | --- | --- |
| Lead | The interactive agent working with the owner | Intent, decomposition, the worker's prompt and boundary, verification of results, integration, the decision to land | Land unverified worker output; hand the worker authority the owner has not given |
| Worker | A non-interactive agent process, any provider or model (trial: `traecli exec -m GPT-5.5`, xhigh reasoning) | Doing one bounded task in its own worktree and producing the named artifacts | Commit, push, merge, touch other repos or trees, exceed its allowed outputs |
| Supervisor | A cheap read-only observer (trial: a Haiku subagent) | Liveness, progress, boundary compliance, stall and violation reports | Edit, kill, message the worker or fix anything; it reports, the lead decides |

The lead is the only role the owner talks to. The supervisor has no authority
by design: a monitor that can act is a second lead.

## The launch contract

Every worker launch fixes these in advance, in files, not chat:

1. **Task file** (`prompt.md`): the task, hard rules, allowed outputs, and the
   report format. The worker reads the repo's `AGENTS.md` first.
2. **One worktree and branch** created by the lead from the current mainline.
3. **Allowed outputs**: exact paths the worker may write. Everything else is a
   violation.
4. **Sandbox and limits**: provider sandbox mode (trial: `workspace-write`), a
   shell-tool timeout, and a hard wall-clock cap around the process.
5. **Evidence channels**: an event stream (`--json` JSONL), stderr, and a
   last-message file, all outside the worktree, plus the PID.
6. **Read-only inputs**: sibling repos are read at a named commit
   (`git show <commit>:<path>`), never through the working tree.

## Supervisor checks (per cycle)

Liveness and elapsed time; event-stream growth and last event; stall after N
quiet minutes; new stderr; the worktree's `git status` (only allowed outputs may
appear); protected repos' `HEAD` and dirty state against a baseline; forbidden
commands in the event stream (builds, tests, the Insula, GPU jobs, git writes
elsewhere, network). Output is one line per cycle and a final verdict:
CLEAN, DEGRADED or VIOLATED, with event line numbers as evidence.

## The lead's duties after the worker exits

Read the supervisor's verdict first, then the worker's artifacts. Verify the
artifacts against ground truth yourself (sample claims, rerun tests). Nothing
the worker produced lands until the lead has verified it; the lead, not the
worker, commits and lands, and only where the repo profile permits.

## What this maps to

- The worker is a provider process. Corenius already speaks ACP to `traecli acp
  serve` and has a `ProcessSupervisor` (`src/supervisor.rs`); the runtime version
  of this form factor is corenius spawning the worker and running the
  supervisor's checks itself (roadmap M4).
- Per the interception spike, claims and path limits are advisory at the
  protocol level. The supervisor's worktree-status check is the post-turn
  backstop it recommends; the provider sandbox is the only preventive control
  and its real strength is unverified.
- It is the concrete case for the guideline topics on roles (2.1), messaging
  (2.7) and coordination (2.8) in `guideline-map.md`.

## Open questions this form factor raises

- **Enforcement.** A supervisor that only reports means a violation is seen
  after it happens. Decide which violations warrant a deterministic kill by the
  runtime, not the supervisor.
- **Stall and cost limits.** Thresholds for quiet minutes, tokens and dollars
  per worker, and what happens at the limit.
- **Event-stream schema.** `traecli exec --json` emits its own JSONL; other
  providers differ. A common event shape is needed before supervisors are
  provider-agnostic.
- **Worker-spawned agents.** Whether a worker may run its own subagents, and how
  the supervisor sees them.
- **Second opinion.** The supervisor watches behavior, not correctness; the
  lead's verification is what checks the content. Whether a separate verifier
  role is needed for larger tasks.
- **Multiple workers.** Path claims between concurrent workers (advisory today)
  and GPU leases for workers that need them.

## Trial log

- Trial 1: worker verified `docs/reference/milano.md` against milano at `ed218ae`
  (read-only), writing `docs/reference/milano-verification.md`. Ran 2026-09-29.
  - **Worker.** `traecli exec -m GPT-5.5`, xhigh reasoning, `workspace-write`
    sandbox. About 16 minutes wall clock, 293 events, empty stderr, exit 0, the
    one allowed file (274 lines) plus a last-message file. No boundary
    violations: the worktree held only that file and milano stayed at `ed218ae`
    (lead's own checks). Token and dollar cost were not measured.
  - **Result.** 59 claims: 53 confirmed, 3 wrong, 1 stale, 2 not verifiable from
    source. The lead checked all 3 wrong verdicts against the source and 3 of the
    confirmed ones; all held. The other 50 confirmed claims are unchecked. The
    wrong claims were real errors in the reference and in milano's own
    `AGENTS.md` (the extracted candidate is not read-only); the reference is now
    corrected, milano's `AGENTS.md` is not.
  - **Supervisor.** The Haiku subagent returned eight times, seven of them
    interim, because it delegated to its own background loops and never blocked.
    Its boundary observations matched the lead's, and its final verdict (CLEAN)
    was right. Its timings were not: it reported completion at 07:25:50 while the
    process was still running, and its last reports put the exit at about 07:20
    after 9.5 minutes, while the process exited about 07:26 after roughly 16
    minutes. It inferred completion from the output file instead of the process.
  - **Lessons.** A worker on a bounded read-only task with a fixed output path
    works well. A supervisor needs a blocking contract (wait on the PID, send one
    final report) and should read process state, not artifacts. The lead should
    keep its own PID watch and boundary checks; the supervisor's timings cannot
    be trusted without them. A deterministic supervisor (a script) would do the
    same checks without the drift and the token cost.
