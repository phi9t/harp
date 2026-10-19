# Sureal `semantic-layout`: a lead with interactive traecli workers (case study)

| | |
|---|---|
| Subject repo | `/data02/home/philip.yang/workspace/sureal` (origin `git@github.com:phi9t/sureal.git`) |
| Commits read | `phi9t/mainline` at `6d0e97416ca6` ("Add workstream coordinator for semantic-layout tickets"); `work/semantic-layout/integration` at `a0fbeb785036`; ticket branches `8eb02ff5c505` (01) and `a4b3776` (02, still moving) |
| Corenius read | this worktree at `48d71588fff0` (branch `worktree-agent-a74425ca9ef305e92`) |
| Observed | 2026-10-05, 06:18 to 06:22 UTC. Ticket 02's worker was running throughout |
| Mode | Read-only in sureal: file reads, `git log/show/diff/status/reflog/worktree list/branch`, `ps`, `tmux list-windows` and `capture-pane`. `workstream.py` was not run. No worker launched, nothing written outside this file |

Citation form is `path:line` relative to the sureal repo unless a prefix says otherwise
(`corenius:` is this worktree). Labels:

- **[READ]** stated in a file or commit I opened. Unlabeled sentences that carry a citation are [READ].
- **[OBSERVED]** output of a command I ran, with the UTC time where it matters.
- **[TRANSCRIPT n]** the lead's own session log,
  `~/.claude/projects/-data02-home-philip-yang-workspace-sureal/3cfda7be-3a34-4c1c-9dba-7c87df468f6c.jsonl`,
  record `n` (0-based line). It records the commands the lead ran and their output, with timestamps.
  It is a primary record of what the lead did, not of what the worker did.
- **[INFERRED]** my conclusion from the above.
- **[LEAD]** reported by the sureal lead, not verified.

## 1. The setup as run

### Roles

- **Lead:** one interactive Claude Code session in the main checkout on `phi9t/mainline`. It wrote the
  spec, tickets and coordinator, launches, verifies and merges [TRANSCRIPT 535-550, 618-641].
- **Workers:** interactive `traecli` TUI sessions, one per ticket, model `gpt-5.5` (`.scratch/semantic-layout/workstream.py:27-28`).
- **Supervisor:** none. Nothing in `workstream.py` watches a worker's process, paths or commands (whole file, 192 lines).

### How the work was defined

- Decisions: `docs/adr/0001-directories-express-concepts.md`, `docs/adr/0002-bazel-runs-inside-insula.md`,
  `experiments/waymo-perception/docs/adr/0001-source-pins-refer-to-snapshots.md`, `docs/repo-structure.md` (832 lines).
  All exist at `6d0e974`.
- Spec: `.scratch/semantic-layout/spec.md`, 186 lines, 49 numbered user stories (`spec.md:29-94`).
- Tickets: 27 files, `.scratch/semantic-layout/issues/NN-slug.md`, 13 to 18 lines each before work starts.
- Tracker conventions: `AGENTS.md:7,11` points at `docs/agents/issue-tracker.md` and `docs/agents/triage-labels.md`.
- **`.scratch/semantic-layout/` is tracked, not ignored.** `git ls-files .scratch` lists 29 files and
  `.gitignore` has no `scratch` entry [OBSERVED]. It was committed in `3f8b4cc` (spec, tickets, ADRs,
  `docs/agents/*`; 38 files) and `6d0e974` (`workstream.py`). `146d970` does not touch `.scratch`; it adds
  `experiments/waymo-perception/tools/{layers,pins,suites}.py` and their tests, which the worker prompt relies on.
  Because the files are tracked, every ticket worktree contains the spec, all tickets and the coordinator.

### Coordinator

One script, `.scratch/semantic-layout/workstream.py` (standard library, no tests in the tree), with five
commands (`workstream.py:9-13`):

- `status` and `frontier` read every ticket file and print state (`:82-93`). Note that both call
  `integration()`, which creates the integration worktree if it is missing (`:41-46,60`), so even the
  reporting commands can write.
- `up` starts `traecli app-server daemon start` and the `status` and `dashboard` tmux windows (`:102-110`).
- `launch NN...` refuses unless the ticket's state is `ready`, creates the worktree and branch from the
  integration branch, renders the prompt to `~/.cache/sureal/workstream/semantic-layout/NN-slug/prompt.md`,
  and opens a tmux window running the worker (`:148-163`).
- `merge NN` refuses unless the state is `done` and the ticket worktree is clean, then runs
  `git merge --no-ff` in the integration worktree (`:166-173`). It runs no test or gate.

State is derived, not stored (`:58-79`):

| State | Rule | Line |
|---|---|---|
| `merged` | the ticket file in the integration worktree says `done` | `:69-70` |
| `running` | a ticket worktree exists, its ticket file still says `ready-for-agent` or `ready-for-human`, and a tmux window with the ticket's name exists | `:72` |
| `stopped` | as above, but no such tmux window | `:72` |
| anything else | the raw `**Status:**` text of the ticket file in the ticket's worktree (`done`, `needs-info`, or any other string; it is not validated) | `:72` |
| `ready` / `blocked` | no worktree; every blocker is / is not `done` on integration | `:68,74` |

Two details differ from how the lead describes it. "Finished" is read from the file in the integration
*worktree's working directory*, not from the branch's committed tree (`:60-61`). And `running` means "the
tmux window exists", not "the process is alive or working"; the window command ends in `exec zsh`
(`:160`), so the window outlives the worker.

### Git layout

- Integration branch `work/semantic-layout/integration`, worktree `.worktrees/semantic-layout-integration`,
  created from mainline `6d0e974` at 05:43:02 [OBSERVED: reflog].
- Per ticket: branch `work/semantic-layout/NN-slug` at `.worktrees/semantic-layout-NN-slug`, created from the
  integration branch at launch (`workstream.py:154-155`; reflog: 01 from `6d0e974` at 05:43:07, 02 from
  `a0fbeb7` at 05:53:35).
- Nothing has gone to mainline or a remote: integration is two commits ahead of `phi9t/mainline`
  (`8eb02ff`, `a0fbeb7`), there is no remote-tracking branch matching `*semantic*`, and `workstream.py`
  has no push or mainline command [OBSERVED].

### tmux and daemon

Session `vision-frontier-workstream`, created 05:43:04, four windows at 06:18:49 [OBSERVED]: `status`
(loop of `workstream.py status` every 30 s), `dashboard` (`traecli dashboard`), `01-cpu-rootfs-with-bazel`,
`02-wrapper-runs-one-test`. A managed daemon `traex app-server --listen unix:// --managed-daemon`
(pid 2731765) started at 05:43:03 [OBSERVED: `ps`; TRANSCRIPT 546]. Whether the workers attach to that
daemon is **[LEAD]**; the worker command line carries no `--remote` flag and I found nothing that shows it.

### Launch command

Read from the tmux window's start command and from `ps` [OBSERVED], matching `workstream.py:27-28,160`:

```
traecli -m gpt-5.5 -c model_reasoning_effort=xhigh -c model_reasoning_summary=detailed --yolo "$(cat <state>/NN-slug/prompt.md)"; exec zsh
```

with the ticket worktree as the window's cwd (`:161`). `~/.trae/traecli.toml:3-4` sets
`default_permissions = ":danger-full-access"` and `approval_policy = "never"`. Both worker panes show
"Full Access" in the status bar [OBSERVED].

### Worker prompt protocol

The prompt is the `PROMPT` constant (`workstream.py:113-145`); the rendered copy for ticket 01 matches it
line for line. It contains:

- Read order: `AGENTS.md` and `docs/agents/`, the ADRs and `docs/repo-structure.md`, the spec, the ticket (`:115-119`).
- Scope: that ticket only (`:121`).
- Boundaries, as instructions: stay in this worktree and branch, no push, merge or history rewrite
  (`:124-125`); outside the repository create new versioned paths, never modify, move or delete existing
  ones (`:126-127`); never modify files under `research/` (`:128`); no `.py` change under
  `experiments/waymo-perception/{pipeline,gpu,tier1,cohort,resources}` unless the ticket says so
  (`:129-131`); run `pins.py check --base <integration>` before each commit and record the result (`:131-133`).
- Method: test-first, narrow tests then the full suite, commit in small steps (`:134-136`).
- Reporting: tick each criterion and commit as it is verified (`:139`); at the end append `## Comments`,
  set `**Status:** done`, commit, stop (`:140-142`); if blocked, write the question, set
  `**Status:** needs-info`, commit, stop (`:143-144`).

The worker commits. The ticket file and the branch are the only channel between lead and worker; no
messages pass.

### How the lead is woken, and what it does on stop

- A background shell loop in the lead's harness polls `workstream.py status` every 60 s and exits when the
  ticket's state is no longer `running` [TRANSCRIPT 555]. For ticket 02 the loop has a `sleep 90` before
  it and a `sleep 60` after it, and also prints `git status --short` of the worktree; it was a live
  process (pid 3313080, started 05:53:41) when I looked [OBSERVED: `ps`; TRANSCRIPT 652].
- On ticket 01's stop the lead ran, in order [TRANSCRIPT 618-641]: status, branch log, `git status` and
  the ticket's comments; the diff of the build script and Dockerfile; its own checks (existing lock digests
  and directory mtimes against the baseline, `bazel --version` in the new rootfs, `pip freeze` old vs new,
  rootfs identity recomputed against both locks, the new tests); the upstream Bazel checksum; then
  `workstream.py merge 01` and `launch 02`.

## 2. Tickets and the blocking graph

Format, identical across all 27 files (shown for `issues/01-cpu-rootfs-with-bazel.md` at `6d0e974`):
line 1 `# NN: Title`; line 3 `**What to build:** ...`; line 5 `**Blocked by:** NN (title), ...` or
`None (can start immediately)`; line 7 `**Status:** ...`; from line 9 `- [ ]` acceptance items.

- Count: 27 tickets; 186 acceptance items in total, 5 to 10 per ticket [OBSERVED: counted].
- Status values at `6d0e974`: 25 `ready-for-agent`, 2 `ready-for-human` (10 `snapshots-in-hdfs`, 27 `readmit-balanced16`).
- Edges: 42 `Blocked by` references [OBSERVED: computed from the files].

```
01 -> 02 -> 03 -> 04           05 <- 02      06 <- 05
07 <- 03, 04     08 <- 06, 07  09 <- 07      10, 11, 12 <- 09
13 <- 11, 12     14, 15, 22 <- 13
16, 18, 20, 21, 23 <- 14, 15   17, 19 <- 16
24 <- 17, 18, 19   25 <- 22, 24   26 <- 20, 21, 23, 25   27 <- 04, 10, 26
```

- **Roots:** one, ticket 01. **Leaves:** 08 and 27.
- **Longest chain:** 15 tickets: 01, 02, 03, 04, 07, 09, 11 (or 12), 13, 14 (or 15), 16, 17 (or 19), 24, 25, 26, 27.
  With every ready ticket started and finished together the effort takes 15 rounds.
- **Width:** in that round-by-round schedule the rounds hold 1, 1, 2, 2, 1, 2, 3, 1, 3, 5, 2, 1, 1, 1, 1
  tickets; the widest is round 10 (16, 18, 20, 21, 23). The largest set of tickets with no dependency between
  any two is 9 (08, 10, 17, 18, 19, 20, 21, 22, 23), reachable only if earlier tickets are left unfinished
  while later ones proceed. The first six rounds never exceed two parallel tickets.

Sureal's own tracker document does not describe this format. `docs/agents/issue-tracker.md:9` asks for a
plain `Status:` line and `:27` for `Blocked by: NN, NN`; `docs/agents/triage-labels.md:5-11` lists five
status strings (`needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, `wontfix`). The tickets
use bold labels and `NN (title)`, and `done` exists only in the worker prompt and the coordinator.
`workstream.py:54` finds blockers with the pattern `(\d\d) \(`, so a blocker written as the tracker
document specifies would be read as no blocker [INFERRED from the regex].

## 3. Ticket 01 as data

| Time (UTC, 2026-10-05) | Event | Source |
|---|---|---|
| 05:43:07 | branch created; worker process started | reflog; `ps` lstart of pid 2736012 |
| 05:50:38 / 05:50:41 | `rootfs-v3` directory / `rootfs-v3.lock.json` mtimes under `~/.cache/waystone/waymo-perception/insula/` | `ls --time-style=full-iso` |
| 05:50:40 | lead writes `baseline-01.txt` | file line 1; TRANSCRIPT 600 |
| 05:53:00 | lead's watcher exits: state `done`, criteria 5/5, commits 0 | TRANSCRIPT 617, 619 |
| 05:53:04 | worktree dirty: 3 modified, 1 untracked | TRANSCRIPT 619 |
| 05:53:21-24 | lead's verification runs against the uncommitted tree | TRANSCRIPT 633-634 |
| 05:53:28 | worker commit `8eb02ff` | commit date; reflog |
| 05:53:31 | lead runs `git add -A && git commit` in the worker's worktree; "nothing to commit" | TRANSCRIPT 638-639 |
| 05:53:35 | merge `a0fbeb7` (parents `6d0e974`, `8eb02ff`); ticket 02 branch created and launched | commit; reflog |

- Ticket: build a new CPU rootfs version with Bazel 9.2; 5 acceptance items, all ticked on integration.
- Diff `6d0e974..8eb02ff`: 4 files, +468 -14 (ticket file +147, `build.sh`, `insula/Dockerfile`, a new
  233-line test module). One commit, authored `phi9t` with a `Co-authored-by: TRAE CLI` trailer.
- The worker pane's last line reads "Worked for 10m 25s · 5:54 AM" [OBSERVED].
- Work outside the repo: a docker build that downloads the Bazel binary with `urllib` inside a `RUN` step
  (Dockerfile in `8eb02ff`), an export to the new directory `rootfs-v3` and a new lock file beside the
  existing ones [READ: ticket file lines 20-28 on integration; OBSERVED: the paths exist].
- `pins.py check` reported `FAIL: 2 changed file(s) cited by retained receipts` (`build.sh` cited by 4
  receipts, `insula/Dockerfile` by 745); the worker recorded it with a reason, as the prompt asks, and the
  ticket was merged (ticket file lines 51-56 on integration). The prompt asks for a record, not a pass.
- The worker's full-suite run failed on the host: 190 tests, 5 failures, 98 errors, attributed to packages
  and sources that exist only in the rootfs (lines 57-60). The worker's attribution is unverified.
- The ticket's comments end with two lists for later re-admission: 67 files naming `rootfs-v2` and 16 that
  compare runtime locks (lines 64-148).
- The lead's own checks all passed [TRANSCRIPT 634]. They ran before the worker's commit existed; nothing
  in the transcript re-checks the committed tree against what was verified before the merge 7 s later
  [TRANSCRIPT 641].

## 4. Ticket 02 as observed

| Time (UTC) | Observation |
|---|---|
| 05:53:35 | launched from `a0fbeb7`; prompt identical to ticket 01's except for names (both files are 2326 bytes) |
| 06:18:39 | status window: 01 `merged` 5/5; 02 `running` 0/8, 0 commits; 25 tickets `blocked`, none `ready` |
| 06:18:54 | pane: 24 m 51 s elapsed, 66.4K tokens shown, "Context 36% left", plan 3 of 6 steps done |
| 06:19:16 | worktree: 11 paths staged or modified, no commit, `**Status:** ready-for-agent` |
| 06:19:57 | first commit `a4b3776` "Add Insula Bazel wrapper", 11 files, +328 |
| 06:21:40 | status window: `running` 0/8, 1 commit; pane 27 m 41 s, "Context 27% left", still working |

No acceptance item was ticked in the first 28 minutes. The worker's own plan lists "Update ticket, commit
small steps, final closeout commit" as its last step [OBSERVED: pane at 06:18:54].

## 5. What went wrong or is weak

| # | Item | Evidence |
|---|---|---|
| 1 | **Done before commit.** The worker set `Status: done` in the working file before committing. The coordinator reads the working file (`workstream.py:67`), so the watcher fired on a dirty tree. The commit followed 28 s after the notification. | Verified: TRANSCRIPT 617-619 against commit time 05:53:28. The only commit carries the status change, so the window exists by construction. |
| 2 | **The lead tried to commit the worker's work.** `git add -A && git commit` with the lead's own message ran in the worker's worktree 3 s after the worker's commit and found nothing to commit. | Verified: TRANSCRIPT 638-639. The lead's note describes this as "nearly committed". That an earlier attempt would have succeeded is [INFERRED]. |
| 3 | **The interim fix is a delay.** Ticket 02's watcher sleeps 60 s after the state change and prints `git status`; `merge` refuses a dirty tree (`workstream.py:170-171`). Neither ties "done" to a commit. | Verified: live process command line; code. |
| 4 | **The external-state baseline was taken mid-run.** It was written 7 m 33 s after the worker started and within three seconds of the new rootfs's file mtimes; it does not list `rootfs-v3`. It cannot show what changed in the first seven and a half minutes. The lead's report to the owner and its note both say it was recorded before the worker started. | Verified: `baseline-01.txt:1`, TRANSCRIPT 600 vs 550; lead's statements at TRANSCRIPT 655, 687. |
| 5 | **The baseline is shallow and manual.** It holds sha256 of nine lock files, the top-level mtime of four rootfs directories and a directory listing (`baseline-01.txt:3-19`). A top-level mtime does not change when a file deeper in the tree is edited. The lead also recomputed the `rootfs-v2` identity against its lock, which is the stronger check. No coordinator command takes or compares a baseline, and none was recorded for ticket 02 (the state directory holds only `baseline-01.txt`). | Verified: file, TRANSCRIPT 634, directory listing. The mtime limitation is [INFERRED]. |
| 6 | **Boundaries are instructions only.** The worker runs with full access and no approval, and no process observes it. | Verified: `workstream.py:27-28,124-133`; `~/.trae/traecli.toml:3-4`. |
| 7 | **No cap, stall detection or event stream.** No timeout wraps the worker, nothing measures quiet time, and no output is captured to a file; the record of a run is tmux scrollback and whatever traecli keeps. | Verified by absence in `workstream.py`; the state directory holds only `prompt.md` per ticket. What traecli itself stores was not examined. |
| 8 | **`running` cannot tell working from idle.** The interactive TUI does not exit: ticket 01's `traecli` process was still alive at 06:19, 26 minutes after its commit. A worker that stops without changing the status line stays `running` and the watcher never fires. | Verified: `ps`; `workstream.py:72`. The stuck-worker case is [INFERRED]; it has not happened. |
| 9 | **Progress columns lag.** Ticket 01 showed 0/5 and 0 commits until the end; ticket 02 showed 0/8 and no commit for 26 minutes, then one commit with 0/8. The prompt asks for incremental ticks and commits. | Verified: TRANSCRIPT 619; section 4. |
| 10 | **Merge has no gate.** `merge` checks state and cleanliness only; verification is whatever the lead runs by hand. The failing pin check and the failing host suite did not block the merge. | Verified: `workstream.py:166-173`; section 3. |
| 11 | **`ready-for-human` tickets are launchable.** `launch` checks only `state == 'ready'`; the `[human]` marker is display only. | Verified: `workstream.py:78,87,152`. Tickets 10 and 27 are not yet ready, so untested. |
| 12 | **Parallel work is untested.** One worker has run at a time; the first fork is after ticket 02 (03 and 05). Conflicts between parallel tickets and a ticket branch falling behind a moved integration branch have not occurred. | Verified: graph; reflog. |
| 13 | **Lead and worker both write in the worker's worktree.** The lead ran tests and `git add -A` there. | Verified: TRANSCRIPT 633, 638. |
| 14 | **Hand-off is prose.** The 83-file list for a later ticket lives in ticket 01's comments; nothing routes it to a later ticket's prompt. | Verified: ticket file; `PROMPT` has no such slot. |
| 15 | **No independent review of a merged ticket yet.** | [LEAD]. |

## 6. Sureal as run versus the Corenius toolbox

Corenius citations are to this worktree at `48d7158`. Nothing here was run against sureal; the right-hand
column describes code and documents, and its application to sureal is [INFERRED] unless a line is cited.

| Topic | Sureal as run | Corenius toolbox as it exists |
|---|---|---|
| Who commits | The worker (`workstream.py:136,139-144`). | The lead. The prompt header forbids worker commits (`corenius:tools/agents/worker.py:133`), a moved `HEAD` is a `worker-committed` violation (`:374-376`), and the skill says "The worker never commits" (`corenius:.claude/skills/run-worker/SKILL.md:30-31`). |
| Worker process | Interactive TUI in a tmux window; stays alive after finishing. | `traecli exec ... --json ... -` detached in its own session, prompt on stdin (`worker.py:268-295`). No interactive mode exists in `worker.py`. |
| Sandbox | None: `--yolo`, full access. | Default `-s workspace-write` (`worker.py:273,527`); optional capsule with only the worktree mounted (`:231-233`), and `--sandbox insula` making it read-only except allowed paths (`:260-265`). The practice note says the provider sandbox's real strength is unverified (`corenius:docs/practice/lead-supervisor-worker.md:58-61`). |
| Network | Used (image build downloads Bazel). | Header: "No network access and no package installs" (`worker.py:135`). `curl`, `wget`, `pip install` in a command are violations (`:34-35`). Capsule mode passes `--net host` (`:256`). |
| Could ticket 01 have run under the default contract? | It ran. | Not as written. (a) The header forbids network and commits (`worker.py:133,135`). (b) Allowed paths are relative to the worktree (`:131,519`; absolute paths are rejected in insula mode, `:155-156`), so `~/.cache/waystone/.../rootfs-v3` cannot be declared as an output. (c) The worker's commit would be a violation (`:374-376`). (d) Each shell call is limited to 120 s (`:276`). (e) In a capsule only `/workspace` is visible (`:233`), so the shared cache is not there. Whether `workspace-write` would block the docker socket or the cache write was not tested. The download itself happens inside `docker build` through `urllib`, so the forbidden-command patterns would not have matched it [INFERRED from `:32-36` and the Dockerfile]. |
| Supervision | None. | `supervise` re-checks each cycle: worktree status against allowed paths, protected repos' head and dirty state, forbidden commands in the event stream (`worker.py:366-417`); writes `verdict.json` CLEAN, DEGRADED or VIOLATED (`:471-496`). It reports and does not stop the worker. |
| Wall-clock cap | None. | `timeout <cap>` around the process, default 2700 s (`worker.py:280,529`). Ticket 02 had run 1 690 s when last observed. |
| Stall | None. | A STALL finding after 300 s without events (`worker.py:419-422,530`); a finding, not a kill. |
| Event stream | None captured. | `events.jsonl`, `stderr.log`, `last-message.md`, `exit_code`, `run.json` in a run directory outside the worktree (`worker.py:246,275,279-317`). |
| Done signal | A status line in a working file, polled every 60 s; raced the commit. | Process exit: `exit_code` file or dead pid (`worker.py:429-433`), read from process state "never artifacts" (`:13-14`). It requires a process that exits. |
| Ticket location | `.scratch/<effort>/issues/NN-slug.md`, tracked. | `docs/efforts/<effort>/tickets/NN-slug.md` (`corenius:tools/agents/tracker.py:4,116-117`). |
| Ticket layout | `# NN: Title`, bold `**Label:**` lines, free order, items directly under the header. | `# NN slug: title`, then exactly `Type`, `Status`, `Owner`, `Blocked by`, `Claimed` in order, then `## Acceptance` and `## Comments` (`corenius:docs/efforts/agentic-engineering/milestone-b/contracts/ticket-format.md:6-23`; `tracker.py:34,41,205-211`). A sureal ticket fails `lint` on its title, header and missing sections [INFERRED from the regexes]. |
| Status vocabulary | `ready-for-agent`, `ready-for-human`, `done`, `needs-info`, plus derived `blocked`, `ready`, `running`, `stopped`, `merged`. | `open`, `claimed`, `resolved` only (`tracker.py:36`); any other value is a lint problem (`:221-222`). There is no `needs-info`, no agent/human distinction, and no state for "worker finished, not yet landed". Claims carry an owner and an expiry (`ticket-format.md:12,24,26`), which sureal lacks. |
| Blockers | `NN (title)` list; a ticket is ready when every blocker is `done` on integration (`workstream.py:68`). | Comma-separated ids only (`tracker.py:120-130`); cycles and unknown ids are refused (`ticket-format.md:25`); `claim` refuses while a blocker is unresolved (`tracker.py:435`). |
| Frontier | `frontier` prints ready tickets; `launch` enforces it. | `tracker.py list` orders open, claimed, blocked, resolved (`tracker.py:508-515`). No tool connects the frontier to `worker.py launch`; the lead does that by hand. |
| Worktree and branch | Inside the repo: `.worktrees/<effort>-NN-slug`, branch `work/<effort>/NN-slug`, based on the integration branch. | Beside the repo: `../<repo>-worker-<name>`, branch `worker/<name>`, base default `main` (`worker.py:199-203,524`). Sureal has a local `main` that is not its mainline, so the default base would be wrong there [OBSERVED: `git branch`]. |
| Integration and merge | `git merge --no-ff` into an integration branch, one merge commit per ticket; nothing reaches mainline. | `land.py` rebases the branch onto the target (`corenius:tools/agents/land.py:222`), refuses any merge commit in the landed range with exit 4 (`:223-229`), runs gates on the exact tip (`:230`), pushes, then fast-forwards (`:234-244`). The target is `--mainline`, default `main` (`:257`), so an integration branch could be named as the target. Sureal's integration branch already contains merge commit `a0fbeb7`, which `land.py` would refuse on the way to mainline [INFERRED, not run]. |
| Gate at merge | None. | `--gate` commands, repeatable (`land.py:253`); the repo gate is `tools/agents/check.sh` (`corenius:AGENTS.md:21`). |
| State outside the repo | A prompt rule (`workstream.py:126-127`) and a hand-made baseline. | Not covered. `--protect` takes git repositories and records `HEAD` and dirty state (`worker.py:207-215,520`); a cache directory that is not a repository cannot be protected. Recursive `rm` aimed outside the worktree is a violation by command text (`:53-85`). |
| Claim checking | The lead re-derives facts by hand. | `claims.py` samples report lines tagged `[R]`, `[X]` or `[I]` (`corenius:tools/agents/claims.py:35`). Sureal's ticket comments carry no such tags, so nothing would be sampled [INFERRED]. |
| Coordination | One script holds state derivation, launch, tmux and merge. | Separate tools: `worker.py`, `tracker.py`, `wt.py`, `land.py`, `claims.py`, `msg.py`, `evidence.py`, `effort.py` (`corenius:AGENTS.md:17-21`). No single command spans ticket to launch to land. |
| Visibility | A tmux window per worker, a status loop and `traecli dashboard`. | `worker.py status` and `status.jsonl` (`worker.py:442-455,499-510`). `bench/tmuxh/bench-tmux` exists for benchmark runs (`corenius:AGENTS.md:27`); whether it serves `worker.py` runs was not checked. |
| Hand-offs between tickets | Prose in the finished ticket's `## Comments`. | `msg.py` has a `handoff` message type (`corenius:tools/agents/msg.py:15`). Ticket comments are single stamped lines (`tracker.py:44`; `ticket-format.md:19,29`), so a list like ticket 01's would not fit there [INFERRED]. |

## 7. Open decisions this raises

1. Whether a worker may commit on its own branch, or whether the lead remains the only committer; and, in
   either case, what the single event is that means "ready for the lead".
2. Whether the toolbox supports an interactive, watchable worker in addition to `traecli exec`, and what
   then replaces process exit as the done signal and the event stream as evidence.
3. How a task that legitimately needs network, a container build or writes to a shared cache is declared in
   the launch contract, and who or what enforces that declaration.
4. Whether state outside any repository can be named as protected, how it is fingerprinted, and when the
   baseline is taken.
5. Which ticket layout and location is canonical for a repo that already has tickets, and whether the other
   is converted, imported or tolerated.
6. Whether the status vocabulary gains states for "needs a decision", "for a human" and "finished, not yet
   landed", or whether those are expressed some other way.
7. Whether landing into an integration branch is a supported target, and whether per-ticket merge commits
   are permitted there or history stays linear.
8. What the gate is at ticket merge, and whether a recorded failing check (the pin check here) blocks or
   only informs.
9. Whether one command should drive frontier, launch, supervise and land for an effort, or the tools stay
   separate with the lead as the glue.
10. Where structured hand-offs from one ticket to later tickets live.
11. What limits apply to a worker (wall clock, quiet time, tokens, context remaining) and what happens at
    each limit.
12. Whether the lead may run commands inside a worker's worktree while the worker may still be writing.

## 8. Later observations: ticket 02's rework and the first parallel launch

Added 2026-10-05 by a second read, 06:27 to 07:43 UTC, in the same read-only mode and with the same labels as
above. Commits read: `phi9t/mainline` at `eb6e412`; `work/semantic-layout/integration` at `417664f`; ticket
branches `b2b3a4b` (02) and `3c71fb3` (03). Corenius read at `d7f6748`. Sections 1 to 7 are unchanged and
describe the state at 06:22.

### Ticket 02: done, sent back, done again

| Time (UTC) | Event | Source |
|---|---|---|
| 06:31:34 | third commit `2186fe5` "Use legacy Bazel workspace mode" | commit |
| 06:36:02 | `4d43113` "Close semantic layout wrapper ticket": `**Status:** done`, 8 of 8 items ticked | commit |
| 06:37:00 | the lead reviews the branch against the spec and ADR 0002 instead of merging | [TRANSCRIPT] |
| 06:37:41 | the lead writes `review-1.md` (3385 bytes) into the ticket's state directory under `~/.cache/sureal/workstream/` | [OBSERVED] file time |
| 06:37:44 | the lead types one line into the worker's tmux window (`tmux send-keys`) pointing at that file | [TRANSCRIPT] |
| 06:38:35 | the worker commits `b6d7ac5` "Reopen ... after review": status back to `ready-for-agent`, three items unticked | commit |
| 07:05:40 | `0eb7eae` "Use Bzlmod Python wrapper for one test" | commit |
| 07:06:19 | `b2b3a4b` "Mark wrapper ticket done", a commit that changes only the ticket | commit |
| 07:07:22 | merge `417664f` into the integration branch (12 files, +832 −10 against `a0fbeb7`) | commit |

The review found four deviations [TRANSCRIPT; the lead's statement of its own findings, not re-verified here]:
Bzlmod switched off with a hand-written empty lock file, where ADR 0002 and the spec require Bzlmod with a
committed lock; a hand-written test rule where the spec names `rules_python`; the whole experiment placed on
`PYTHONPATH` inside the sandbox; and 3.5 GB of probe caches left in the worktree. The worker's tests passed
and every acceptance item was ticked at the first `done` [READ: `4d43113`].

What this shows:

- A ticked acceptance list and passing tests did not mean the ticket met its spec. The defect was found by a
  review against the spec and the ADR, not by a gate command. [INFERRED from the two rows above]
- The rework was done by the same worker session, which still held its context, 27 minutes after the review
  was delivered. The delivery channel was keystrokes into an interactive terminal. [OBSERVED, TRANSCRIPT]
- The branch history records the close, the reopen and the fix as separate commits. The reopen was made by
  the worker, which also reset the status and unticked the items itself. [READ: `b6d7ac5`]
- The lead's second watcher waited for "finish with a clean tree" [TRANSCRIPT: the background command's
  description], and the worker's final `done` was a separate commit on a clean tree. The race of section 5
  did not recur.

### Prompt change

`eb6e412` on `phi9t/mainline` (07:07:22) changes the worker prompt: commit the work and the Comments first, set
`**Status:** done` only in a final separate commit, never write `done` on a dirty tree; meet stated spec and
ADR requirements or stop with `needs-info` and the exact error; run Bazel only through `./bazelw`, keep one
cache layout and delete probe caches because "the disk is nearly full" [READ: `eb6e412`]. The integration
branch does not contain this commit; ticket worktrees are cut from the integration branch, and the prompt is
rendered from the mainline copy of the coordinator [INFERRED from the branch graph and `workstream.py`].

### Tickets 03 and 05 in parallel

- Both branches were created from `417664f` at 07:07:23 and 07:07:25 [OBSERVED: reflog, state directory
  times]. This is the first time two workers ran at once.
- Ticket 03 finished at 07:41:18 with three commits in 73 seconds: the work (`8a0cdf3`), the verification
  record (`233dc6f`), then the status (`3c71fb3`); 7 files, +484 −39; tree clean; 6 of 6 ticked. Until
  07:40:05 it had no commit and showed 0 of 6. [OBSERVED]
- Ticket 03 commits two executables named `git` and `curl` under `experiments/waymo-perception/tools/`; the
  `git` file describes itself as a "Git command double for Bazel tests running in the minimal rootfs"
  [READ: `3c71fb3`]. Whether that is acceptable was not judged.
- Ticket 05 was still running at 07:42 with 8 uncommitted paths. They include the wrapper `bazelw` and its
  test, which ticket 02 delivered and ticket 03 relies on, and a new rootfs build script and lock inputs for
  the second research program. [OBSERVED]
- Neither parallel ticket had been merged at 07:43, so a merge with a moved integration branch is still
  untested.

### Still absent

- No external-state baseline was recorded for tickets 02, 03 or 05: the state directory holds only
  `baseline-01.txt` [OBSERVED 07:41]. Ticket 05 builds a rootfs.
- The traecli processes of tickets 01 and 02 were still alive at 07:37, 104 and 44 minutes after their
  tickets were merged [OBSERVED: `ps`].
- The data volume was 94% full with 191 GB free; three ticket worktrees held 1.6 GB of Bazel cache
  [OBSERVED 07:38].
- One finding from ticket 02 that later tickets depend on (the CPU rootfs has no C++ compiler, and tickets
  that build C++ evaluators need one) exists only in the lead's report to the owner [TRANSCRIPT].

### Effect on the open decisions in section 7

Decision 1 (who commits) now has a worked answer on the sureal side: the worker commits, and the status
change is its own final commit on a clean tree. Decision 2 (interactive workers) gains a cost on the
non-interactive side: a worker process that exits cannot receive a review and rework with its context
intact. Decision 8 (the gate at merge) gains a case where no command gate would have caught the defect.
Two decisions are added:

13. How review findings reach a worker and how many rework rounds are allowed before the ticket goes back
    to the lead or the owner.
14. Whether a worker may reopen its own ticket and untick acceptance items, or whether only the reviewer
    changes that state.
