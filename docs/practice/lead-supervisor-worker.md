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
| Worker | A non-interactive agent process, any provider or model (trial: `traecli exec -m GPT-5.5`, xhigh reasoning) | Doing one bounded task in its own private git workspace and producing the named artifacts as a commit on its worker branch | Push, merge, move shared branches, rewrite history below its base, touch other repos or trees, exceed its allowed outputs |
| Supervisor | A cheap read-only observer (trial: a Haiku subagent) | Liveness, progress, boundary compliance, stall and violation reports | Edit, kill, message the worker or fix anything; it reports, the lead decides |

The lead is the only role the owner talks to. The supervisor has no authority
by design: a monitor that can act is a second lead.

## The launch contract

Every worker launch fixes these in advance, in files, not chat:

1. **Task file** (`prompt.md`): the task, hard rules, allowed outputs, and the
   report format. The worker reads the repo's `AGENTS.md` first.
2. **One private local clone and branch** created by the lead from the current mainline.
3. **Allowed outputs**: exact paths the worker may write. Everything else is a
   violation.
4. **Sandbox and limits**: provider sandbox mode (trial: `workspace-write`), a
   shell-tool timeout, and a hard wall-clock cap around the process.
5. **Evidence channels**: an event stream (`--json` JSONL), stderr, and a
   last-message file, all outside the worktree, plus the PID.
6. **Read-only inputs**: sibling repos are read at a named commit
   (`git show <commit>:<path>`), never through the working tree.
7. **Temp directory**: the worker receives `TMPDIR`, `TMUX_TMPDIR` and
   `TMPPREFIX` under its run directory (`RUN_DIR/tmp`, `RUN_DIR/tmp/tmux`,
   `RUN_DIR/tmp/zsh`) and must not use the host's `/tmp`.

## Supervisor checks (per cycle)

Liveness and elapsed time; event-stream growth and last event; stall after N
quiet minutes; new stderr; the worktree's `git status` (only allowed outputs may
appear); protected repos' `HEAD` and dirty state against a baseline; forbidden
commands in the event stream (builds, tests, the Insula, GPU jobs, git writes
elsewhere, network). Output is one line per cycle and a final verdict:
CLEAN, DEGRADED or VIOLATED, with event line numbers as evidence.

## Hand-off rule update (2026-10-05)

`docs/efforts/lead-worker-readiness/decisions/20261005-eight-answers.md`
changes the hand-off rule for the lead-worker-readiness instantiations: a
worker commits in its own workspace and hands over an exact Candidate. The
worker still cannot push, merge, move a branch other than its own, or rewrite
history below its recorded base.

The launcher gives each worker a private local clone, not a linked worktree in
the shared repository. For normal provider `workspace-write` launches, the
clone's `.git` is a pointer to a private git directory under the run directory,
and that private git directory is passed to the runtime as an additional
writable root. For `insula` capsule launches, the private clone keeps embedded
`.git` metadata because only `/workspace` is mounted in the capsule. In both
layouts, sandboxed workers write only private Git metadata and the main
repository's `.git` directory stays outside the sandbox.

The same run directory owns the worker's temporary files. The launcher records
the temp directory in `run.json`, passes it to the runtime process, and removes
it when a stopped run is accounted unless the launch requested retention.

The worker branch inside the shared repository is only a publication target:
when the supervisor first pins a ready Candidate, it imports that exact commit
from the private clone into the shared repository under the worker branch.

The launcher has two hand-off modes. In exit mode, the supervisor reports a run
as ready for review only after the process exits with code 0, the private worker
tree is clean, and the private worker branch contains at least one commit on top
of the launch base. In marker mode, used for interactive sessions that may
remain alive, the supervisor reports readiness when the private branch tip's
commit subject equals the declared marker, the worker tree is clean, and the
branch contains at least one commit on top of the launch base.

In both modes the first ready observation pins the Candidate: commit, tree, base
commit, and digest of the brief. If the private branch tip later differs from
that Candidate, the supervisor reports `candidate-moved` with the pinned and
current commits; it does not replace the Candidate, and the shared worker branch
stays at the pinned commit until a later ready Candidate is published. A dirty
tree, no commit after the base, a moved foreign branch, or a branch that no
longer contains the base is not ready.

## Interactive launch and monitors

For effort tickets, `tools/agents/lead_launch.py launch EFFORT NN --interactive
--tmux-session NAME` starts the chosen runtime's interactive command in the
worker's private clone and records the tmux window and pane target in `run.json`.
Interactive launches for `traecli` and `codex` also pass and record
`-c projects={"<worker workspace>"={trust_level="trusted"}}` so a fresh private
clone does not stop at the runtime's folder-trust prompt.
The command refuses interactive launch when tmux is unavailable; non-interactive
launch may still request a best-effort tmux watch window. Interactive windows
stay open after the runtime exits for review or rework.

`tools/agents/lead_launch.py open-session EFFORT --tmux-session NAME` opens only
the ticket monitor and worker monitor windows. `monitor-tickets EFFORT --json`
reports one row per ticket, using the worker's private clone for ahead, dirty
state and tip checks, and marks a visible runtime key prompt as
`blocked-on-prompt`. `wait` wakes on ready, needs-info, stopped,
waiting-for-re-integration, and blocked-on-prompt; the prompt case includes the
prompt text in JSON output. `monitor-workers EFFORT --json` reports each
worker's process state and uptime, pane activity and context where visible,
workspace disk usage, and the last rework delivery with sent and acknowledged
state. For TraeCLI panes, the busy signal is the visible `esc to interrupt`
runtime hint; idle footers such as `Context 95% left` are reported as `95%`.

When an event stream reports the runtime session id, `run.json` records the
provider's transcript path. Interactive `traecli` and `codex` sessions can also
be discovered from the runtime session store by finding the newest session-meta
record whose working directory is the worker workspace and whose timestamp is
after launch.

For interactive runs, `lead_launch.py rework RUN_DIR --findings-file FILE`
types a one-line pointer to the findings file into the worker pane and records a
terminal delivery receipt. Once the worker acknowledges the delivery by pane
activity, event-stream activity, or a new commit, review rework counts against
the review cap; stale-base re-integration does not.

The interim coordinator script kept as lead-worker-readiness evidence is retired
as an operator path. New launches and monitors use `worker.py` and
`lead_launch.py`.

## The lead's duties after the worker exits

Read the supervisor's verdict first, then the worker's artifacts. Verify the
artifacts against ground truth yourself (sample claims, rerun tests). Nothing
the worker produced lands until the lead has verified it; the worker's commit is
the review subject, and only the repo profile and effort decisions decide where
it may be landed.

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

### Trial 2 (2026-09-29): real `traecli`, deterministic supervisor, Haiku relay

Task: summarize `tools/agents/effort.py` in `docs/practice/trial-2-effort-summary.md`
(one allowed output). Launched with `worker.py launch` (GPT-5.5, xhigh,
`workspace-write`), supervised by `worker.py supervise` run by the lead in the
background and, in parallel, by a Haiku subagent following
`.claude/agents/worker-supervisor.md`.

- **Worker:** exit 0 after 139 s, 43 events, verdict CLEAN, no findings, one
  output written and nothing else changed. The lead read the summary against the
  source: function names, exit codes (0, 1, 2, and 64 for `WorkerError`) and the
  test list match. The output was not landed; it was a trial artifact.
- **Haiku supervisor:** made one tool call, blocked for 114 s on the command, and
  relayed exit code and JSON verbatim with no interim reports and no timing
  claims. The trial 1 failures (no blocking, early completion claims) did not
  recur once the checks moved into the script.
- **Defect found:** two supervisors on one run both write `verdict.json`; the
  later write wins (finish time differed by 2 s between the two copies). Run one
  supervisor per run, or make the write idempotent.
- **Defect fixed:** a missing task file was a traceback; it is now a clean
  error (exit 64) that creates no worktree.
