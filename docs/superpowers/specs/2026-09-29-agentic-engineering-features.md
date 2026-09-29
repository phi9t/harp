# Agentic-engineering features for corenius

Milestone A of the re-cut plan: a concrete set of features. Milestone B
implements and verifies them in this repository. A feature belongs here only if
it has a named interface and an acceptance check that a command or a review can
run. Guidance topics that are policy only, with nothing to build, stay in
`docs/practice/guideline-map.md`.

Inputs: `docs/practice/guideline-map.md`, `docs/spikes/collab/design-space.md`,
`docs/spikes/collab/interception.md`, `experiments/collab/RESULTS.md`,
`docs/practice/lead-supervisor-worker.md` (trial 1), and
`docs/reference/agentic-inventory/`.

## How to read this

Each feature has: an id; the problem it removes, with the evidence that it is a
real problem; its interface; acceptance checks; dependencies; where it lives;
and its state today. **State** is one of: *exists* (built and tested),
*prototype* (built, not yet verified end to end), *new*. **Tier 1** features are
the smallest set that makes the lead/supervisor/worker loop trustworthy in this
repo. **Tier 2** makes it scale. **Tier 3** is later.

Language: repo tooling is Python under `tools/agents/` (stdlib only, tests
beside it). Anything that has to live inside the agent runtime is Rust under
`src/`.

## Tier 1: the trustworthy loop

### F1 Worker launch and boundary

- **Problem.** A worker needs its own tree, a fixed set of writable paths and
  read-only inputs at a known commit. Trial 1 did this by hand.
- **Interface.** `tools/agents/worker.py launch --task FILE --name N --allow PATH
  [--protect REPO] [--model M --effort E --sandbox S --cap SEC]`. Creates
  `../<repo>-worker-<name>` on branch `worker/<name>`, writes `prompt.md` with
  the standard rules, records `run.json` (allowed paths, base sha, protected
  repos with baseline HEAD and dirty state), starts `traecli exec` detached.
- **Acceptance.** Unit tests (fake `traecli`) pass; a real run with the requested
  model flags produces `events.jsonl`, `last-message.md`, `exit_code` and leaves
  only allowed paths changed.
- **State.** Prototype (`36e61e0`, 10 tests). Not yet run against real `traecli`.

### F2 Deterministic supervisor

- **Problem.** The subagent supervisor in trial 1 sent ten interim reports, did
  not block, and got completion and timing wrong. Checks that read process and
  git state need no model.
- **Interface.** `worker.py supervise RUN_DIR [--interval S --max-wait S]`
  blocks on the process, checks each cycle, writes `status.jsonl` and
  `verdict.json`; exit 0 CLEAN, 1 DEGRADED, 2 VIOLATED, 3 still running.
  Checks: unexpected change, worker commit, protected repo moved or dirtied,
  forbidden commands in the event stream, stall, stderr, non-zero exit, missing
  outputs. `worker.py status RUN_DIR` is the one-shot form.
- **Acceptance.** One test per violation kind; a test that a finished output
  file does not end supervision while the process is alive (both exist).
- **State.** Prototype, same commit.

### F3 Haiku supervisor wrapper

- **Problem.** The owner wants a lightweight subagent as supervisor. Its job is
  to run F2 and relay its verdict, not to judge from raw state.
- **Interface.** `.claude/agents/worker-supervisor.md` (model haiku, tools Bash
  and Read only): run `worker.py supervise RUN_DIR` in the foreground, repeat
  while exit code is 3, never start background work, never touch the worker or
  repo, report `verdict.json` verbatim plus one sentence.
- **Acceptance.** A real end-to-end run in which the subagent returns exactly one
  message, after the process exit, whose verdict equals `verdict.json`.
- **Note.** A project subagent definition is available from the next session
  start; until then the same instructions are passed to a general-purpose
  subagent with model haiku.
- **State.** New.

### F4 Lead procedure (skill) and project settings

- **Problem.** The launch contract, the lead's own PID watch and the
  verify-before-land rule live only in a note.
- **Interface.** `.claude/skills/run-worker/SKILL.md`: write task file; launch;
  start supervisor; watch with `worker.py status`; on verdict read artifacts,
  verify sampled claims or rerun tests, then commit and land. Plus
  `.claude/settings.json` allowing the worker tool, read-only git and offline
  cargo, and denying git push.
- **Acceptance.** The skill is followed for one real run and the run log shows
  each step; settings load without prompts for the allowed commands.
- **State.** New.

### F5 Worktree lifecycle

- **Problem.** One writer per tree is a rule; nothing creates, lists or removes
  agent worktrees consistently. Trial branches (`claude/*`, `worker/*`,
  `traecli/*`) accumulated by hand.
- **Interface.** `tools/agents/wt.py new --agent A --topic T [--base B]`,
  `status` (branch, ahead/behind, dirty, owner, age), `finish BRANCH` (only when
  merged; works from inside the worktree it removes), `gc` (dry run lists merged
  or abandoned trees; removal needs `--apply`).
- **Acceptance.** Tests on throwaway repos including the finish-from-inside case;
  `status` on the real repo lists every current worktree.
- **State.** New. Port the design of `milano.scm` new, status and finish, with its
  known fix for finish-from-inside. The effort-scoped part, `tools/agents/effort.py`
  (`new`, `decide`, `dehydrate`, `rehydrate`, `status`), is implemented and tested.

### F6 Landing with gates

- **Problem.** Landing by hand moved branch refs without a gate and risks
  contaminating other checkouts.
- **Interface.** `wt.py land BRANCH --gate CMD ... [--no-push]`: fetch, rebase,
  refuse merge commits, run each gate on the exact tip in a clean checkout, push
  first if pushing, then fast-forward local mainline only after the push
  succeeds, retry from the rebase on a lost race, under a lock. Refuses when
  the main checkout has tracked changes overlapping the diff. Prints the
  landed sha.
- **Acceptance.** Tests: lost-race retry (gates re-run on the new tip, local
  mainline never strands a commit), failed gate leaves mainline untouched,
  overlap refused, linear history. Reproduces the fix recorded for
  `milano.scm integrate`.
- **State.** New.

### F7 Tracker: tickets and claims

- **Problem.** `.scratch/<effort>/` is the durable tracker by policy, but there
  is no schema check and no claim protocol, so status lives in prose.
- **Interface.** `tools/agents/tracker.py new|claim|resolve|list|lint`.
  Ticket file `.scratch/<effort>/issues/NN-slug.md` with header lines `Type:`,
  `Status: open|claimed|resolved`, `Owner:`, `Blocked by:`, `Claimed:` (time),
  acceptance criteria and a `## Comments` log. `claim` refuses if claimed or
  blocked; claims carry an expiry; `lint` checks schema and blocker ids.
- **Acceptance.** Tests for each transition and each lint error; the current
  roadmap milestones are represented as tickets and `list` shows the frontier.
- **State.** New. Tickets live under `docs/efforts/<effort>/tickets/` (tracked),
  not `.scratch/`, so they survive a fresh clone.

### F8 Messages and handoffs

- **Problem.** Vaso showed that a long-running agent does not re-read an updated
  handoff file; policy and instruction changes need delivery at a defined point.
- **Interface.** `tools/agents/msg.py send|inbox|ack` over append-only files
  `.scratch/<effort>/messages/<to>/<ts>-<from>.md` with a fixed header (`From`,
  `To`, `Type`, `Re`, `Needs-ack`). `inbox` lists unacknowledged messages. Workers
  receive their task file at launch (F1); the lead checks inboxes at each turn
  start (procedure in F4). Runtime turn-start injection is F18.
- **Acceptance.** Tests for ordering, ack, and a message with `Needs-ack` still
  reported until acknowledged; header lint.
- **State.** New.

### F9 Verification and evidence records

- **Problem.** Receipts elsewhere copied declared values and hashed the wrong
  thing; our own trial results live in prose. Run evidence must name what ran
  and separate measured from declared.
- **Interface.** `tools/agents/evidence.py record RUN_DIR` writes a
  content-addressed, append-only record under `.corenius/evidence/` (gitignored)
  and prints its digest: worker config, model flags, prompt digest, base sha,
  worktree diff digest, verdict, event-stream digest, timings, with each field
  tagged `measured` or `declared`. `verify DIGEST` recomputes what it can.
- **Acceptance.** Tests: tamper with the diff and `verify` fails; two identical
  runs produce distinct records; declared and measured are never merged.
- **State.** New. Records are local; committing digests into commit messages is
  the bridge to history.

### F10 Claim-checking for lead verification

- **Problem.** Trial 1: the lead verified a sample of the worker's verdicts by
  hand. That step is the value of the lead and needs a small tool.
- **Interface.** `tools/agents/claims.py sample FILE --n N` picks claims from a
  verification report deterministically and prints the cited `git show` command
  for each, so the lead runs and records pass or fail; `record` appends the
  outcome to the run's evidence.
- **Acceptance.** Test on the trial 1 report format; outcomes appear in the
  evidence record.
- **State.** New, small.

## Tier 2: scale and safety

### F11 Advisory path claims

- **Problem.** Concurrent workers can touch the same files. Enforcement is not
  available yet (interception spike), but early warning is.
- **Interface.** `tools/agents/claims.py path claim|release|check` over files in
  `.scratch/<effort>/claims/` with expiry; `check` lists overlaps between
  workers' declared allowed paths at launch; F1 refuses a launch that overlaps
  a live claim unless `--force-overlap` is given and logged.
- **Acceptance.** Tests for overlap, expiry, release; a simulated two-worker
  overlap is refused.
- **State.** New. Advisory only, by design.

### F12 Provider sandbox with bind-derived paths

- **Problem.** The workspace-write sandbox is the only preventive control and
  its strength is unverified; corenius spawns providers outside its insula.
- **Interface.** Run the worker under bubblewrap with a read-only view of the
  repo and writable binds for the allowed paths only (interception spike, mode
  "sandbox path allowlist"). Flag `--sandbox insula` on F1.
- **Acceptance.** A test worker that writes outside its allowed paths gets a
  permission error, not just a supervisor violation; the escape attempts in the
  interception fixtures fail. Needs the network and credential paths the
  provider requires; document them.
- **State.** New. Depends on what `traecli` needs inside the sandbox; the first
  step is finding out.

### F13 ACP client correctness

- **Problem.** Corenius advertises no client capabilities and drops agent
  requests; an agent request that reuses id 3 ends the turn early (bug pinned
  by `tests/interception.rs`).
- **Interface.** Rust, `src/provider/acp.rs`: distinguish responses from
  requests by the presence of `method`; answer or refuse `fs/*`,
  `session/request_permission` and terminal requests according to policy;
  advertise only what is implemented.
- **Acceptance.** `cargo test --offline`: the id-collision test flips from
  documenting the bug to asserting the fix; a scripted agent's write request is
  seen and denied.
- **State.** New. Touches the AIME work in the main checkout, so it must wait
  for that work to be committed or move to a worktree.

### F14 Provider-neutral event stream

- **Problem.** `traecli exec --json` events differ from other providers; the
  supervisor's command extraction and stall logic depend on one shape.
- **Interface.** `tools/agents/events.py normalize` maps provider events to
  `{ts, kind, command?, path?, text?}`; F2 consumes it.
- **Acceptance.** Fixtures from real traecli output (trial 1 stream) normalize;
  a second provider adapter added without changing F2.
- **State.** New.

### F15 Budgets

- **Problem.** No token or cost gate exists anywhere in the fleet; only the
  wall-clock cap.
- **Interface.** `--cap` (exists), `--max-events`, `--max-commands`; supervisor
  reports `BUDGET` findings and, for hard budgets, the runtime kills the process
  group. Token counts recorded when the provider reports them.
- **Acceptance.** A test worker exceeding `--max-commands` is killed and the
  verdict is DEGRADED with a budget finding.
- **State.** New. Dollar cost needs provider data; record it as `declared` if
  taken from provider output.

### F16 Spike and reference tooling

- **Problem.** Spikes have no template; reference documents go stale silently.
- **Interface.** `tools/agents/spike.py lint FILE` checks the required sections
  (question, verdict, evidence with sources, unverified assumptions,
  recommendation). `tools/agents/refs.py check` reads the commit recorded at the
  top of each `docs/reference/*.md`, compares it to the repo's current HEAD, and
  prints stale documents and the changed-file count.
- **Acceptance.** Tests on good and bad documents; `refs.py check` on the real
  reference set reports drift for milano (its HEAD moved from `ed218ae`).
- **State.** New.

### F17 Policy gates for this repo

- **Problem.** The sync tool exists but nothing runs it; commit rules are prose.
- **Interface.** `.githooks/pre-commit` (whitespace check, tracker lint, spike
  lint on changed docs) and `pre-push` (refuse force-push and deletion of the
  mainline, merge commits), installed by `tools/agents/wt.py setup` via
  `core.hooksPath`. `sync_agentic_workflow.py --check` runs in the gate list of
  F6.
- **Acceptance.** Hook tests on a throwaway repo; the repo's own gate command
  (below) passes.
- **State.** New.

## Tier 3: later

- **F18 Turn-start message injection** in the corenius runtime (Rust), so
  messages reach an agent at a defined point without the human relaying them.
- **F19 Landing queue and staging branch** with speculative batching, from the
  design space; only if landing contention shows up (the simulator says it
  matters at 8 or more agents).
- **F20 GPU broker** with fair scheduling in front of flock leases; only
  meaningful once a GPU repo adopts the mechanism.
- **F21 Ledger replay** (rebuild all state from an event log) and
  tamper-evident chaining, once F9 records exist and are used.
- **F22 Run metrics** (lead time, conflicts, wasted gate minutes) computed from
  F9 records.
- **F23 Calibrated simulator** as a regression harness for coordination
  policy changes (`experiments/collab/`, exists; needs real durations).

## Cross-cutting acceptance

The repo gate for Milestone B is one command, `tools/agents/check.sh`, that runs
every Tier 1 and Tier 2 test suite, `sync_agentic_workflow.py --check`, the
tracker and spike linters on the repo's own documents, and `cargo test
--offline`. Each feature is done when: its acceptance checks pass in that gate;
it is documented in `AGENTS.md`; and the lead has used it on a real task and
recorded the result in the feature's row below.

## Feature register

| Id | Feature | Tier | State | Verified by | Dogfooded |
| --- | --- | --- | --- | --- | --- |
| F1 | Worker launch and boundary | 1 | prototype | 10 unit tests (fake provider) | trial 1 by hand |
| F2 | Deterministic supervisor | 1 | prototype | same | no |
| F3 | Haiku supervisor wrapper | 1 | new | | no |
| F4 | Lead skill and settings | 1 | new | | no |
| F5 | Worktree lifecycle | 1 | new | | no |
| F6 | Landing with gates | 1 | new | | no |
| F7 | Tracker | 1 | new | | no |
| F8 | Messages | 1 | new | | no |
| F9 | Evidence records | 1 | new | | no |
| F10 | Claim checking | 1 | new | | trial 1 by hand |
| F11 | Advisory path claims | 2 | new | | no |
| F12 | Provider sandbox | 2 | new | | no |
| F13 | ACP client correctness | 2 | new | | no |
| F14 | Event normalization | 2 | new | | no |
| F15 | Budgets | 2 | new | | no |
| F16 | Spike and reference tooling | 2 | new | | no |
| F17 | Policy gates | 2 | new | | no |

## Implementation order for Milestone B

1. Finish the prototype: run F1 and F2 against the real `traecli`; add F3 and F4
   and do one supervised end-to-end run (the acceptance for F1 to F4).
2. F5 and F6, so that every later feature lands through the repo's own
   mechanism instead of by hand.
3. F7, F8, F9, F10 (the tracker, messages and evidence the loop reports into).
4. Tier 2 in this order: F17, F16, F15, F14, F11, then F13 and F12, which
   need the AIME work committed and a decision on provider sandboxing.
5. `tools/agents/check.sh` grows with each feature and is green before each
   landing.

## Decisions needed

1. ~~Where durable efforts live~~ Resolved: tracked `docs/efforts/<slug>/`
   (durable) plus gitignored `.scratch/<slug>/` (execution), with
   `tools/agents/effort.py dehydrate|rehydrate`. See
   `docs/efforts/agentic-engineering/decisions/`.
2. Whether the corenius main checkout, which carries uncommitted AIME work, gets
   that work committed or moved to its own branch. F13 and the gate command
   depend on it.
3. Whether F12 (sandboxing the provider) is required for Milestone B or a Tier 3
   follow-up if `traecli` cannot run inside bubblewrap without extra binds.
4. The Tier 1 boundary: agree that F1 to F10 is the Milestone B minimum.

## Not features

Policy that needs a decision but no code (merge authority, fix-round budget,
repo profiles, vocabulary) is in `docs/practice/guideline-map.md`. A feature that
would enforce one of those waits until the decision is made.
