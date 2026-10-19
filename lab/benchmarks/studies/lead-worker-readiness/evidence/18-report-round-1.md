## Built

Interactive effort launch is now part of `lead_launch.py`: it opens each worker in
its own tmux window, uses marker hand-off, records the tmux target, and leaves
the terminal open after the runtime exits. The worker prompt is generated from
the same brief path as non-interactive launch.

The ticket and worker monitors now report live state from the worker's private
clone, run record, process table and visible tmux pane. They handle review
rework, clean post-review successor commits as `waiting for re-integration`,
held marker sessions, runtime trust prompts, transcript discovery, and terminal
delivery acknowledgements.

The `interactive-launch-and-monitors` readiness scenario now runs in the
multi-checkout layout with a main checkout, integration worktree, record
worktree and private worker clones, and it follows the ticket 21 readiness-row
rules.

## Acceptance

- "`lead_launch.py launch` has an interactive mode: it starts the chosen runtime's interactive command (for traecli: `traecli -m <model> -c model_reasoning_effort=<effort> -c model_reasoning_summary=detailed --yolo \"<brief>\"`) in the worker's workspace, in its own window of a named tmux session, with the marker hand-off, and writes the same run record as a non-interactive launch plus the tmux target. The window stays open after the runtime exits. Without tmux the interactive mode is refused with a clear message; the non-interactive mode is unaffected." Proven by the clean detached gate at `9c4934c`: `python3 tools/agents/test_lead_launch.py` ran 46 tests OK, `python3 tools/agents/test_worker.py` ran 86 tests OK, and `python3 bench/harness/lead_driven.py interactive-launch-and-monitors --out ...` passed `full-flow expected=checked observed=checked`.
- "The brief of an interactive launch carries the standing worker rules the interim script's brief carries (read order, boundaries, the gate, the report sections, the marker commit, no rewriting after the marker, hold-the-marker on request, re-integration on request, the multi-checkout layout), generated from one template that both modes share." Proven by `python3 tools/agents/test_lead_launch.py` (46 tests OK), especially the shared-brief interactive launch assertions, and by the passing interactive readiness scenario.
- "Once the runtime's session id is known, the run record holds the path of the runtime's own session transcript; a test with a fake runtime shows it, and the help says for which runtimes this is observed." Proven by `python3 tools/agents/test_worker.py` (86 tests OK), including event-stream transcript and interactive workspace-session-meta transcript tests.
- "A ticket monitor, usable on an interval in a tmux window and as one-shot JSON, prints one row per ticket: worker state, commits ahead of the integration tip, whether the branch contains the tip, dirty count, age of the last commit, what it waits for, whether the tip was rewritten since it was last pinned or reviewed, the count of changed paths outside the ticket's declared paths, and the size of the diff since the last reviewed Candidate." Proven by `python3 tools/agents/test_lead_launch.py` (46 tests OK), including private-clone monitor counts and blocked-prompt rows, plus the passing interactive scenario.
- "A worker monitor, same two forms, prints per running worker: process and uptime, current activity and context remaining where the runtime's pane shows them, disk used by its workspace, and the last delivery with its sent and acknowledged state." Proven by `python3 tools/agents/test_lead_launch.py` (46 tests OK), including worker monitor pane, disk and delivery assertions, plus the passing interactive scenario.
- "Worker states are derived only from git, the process table, the pane and the run record, and are exactly: running; finishing (marker made, session busy); ready (marker, clean tree, session idle, tip not already reviewed); rework (a review was delivered, no new marker since); waiting for re-integration; needs-info; stopped; landed. A test drives a scripted interactive participant through every state." Proven by `python3 tools/agents/test_lead_launch.py` (46 tests OK) and `python3 bench/harness/test_lead_driven.py` (39 tests OK). The scripted readiness scenario asserts the state sequence `running`, `finishing`, `rework`, `waiting for re-integration`, `ready`, `landed`, `needs-info`, `stopped`; review-required prompt detection adds `blocked-on-prompt` for visible runtime key prompts.
- "`wait` returns when any named ticket reaches ready, needs-info, stopped or waiting for re-integration, prints which and why, and never returns on finishing or rework." Proven by `python3 tools/agents/test_lead_launch.py` (46 tests OK), including the rework/no-trigger and waiting-for-re-integration trigger tests, and by the interactive scenario's wait checks.
- "A command opens the effort's session: the two monitor windows, and nothing else." Proven by `python3 tools/agents/test_lead_launch.py` (46 tests OK), including `test_open_session_creates_only_ticket_and_worker_monitor_windows`, and by the interactive scenario's `open_session_windows == [\"tickets\", \"workers\"]` assertion.
- "All of it works for an effort whose records are on a record branch, run from the main checkout, in a repository with a mainline checkout, an integration worktree, the record worktree and several worker workspaces; a scenario shows launch, a rework delivery, a held marker, a re-integration and a landing in that layout with scripted interactive participants." Proven by `python3 bench/harness/lead_driven.py interactive-launch-and-monitors --out ...` passing from the clean detached gate, and by `python3 bench/harness/test_lead_driven.py` (39 tests OK).
- "The run-worker skill and the practice note describe the interactive mode and the monitors, and say the interim script is retired." Proven by source inspection and `python3 tools/agents/tracker.py lint` in the clean detached gate.

Clean detached gate:

- Worktree: `/data02/home/philip.yang/devx/tmp/lwr-18/clean-gate.ZDtV04/wt`
- Log: `/data02/home/philip.yang/devx/tmp/lwr-18/clean-gate.ZDtV04/gate.log`
- HEAD: `9c4934c`
- Result: exit 0; `GATE COMPLETE`
- Suites: all `tools/agents/test_*.py` except `test_sqlite_baseline.py`, all `bench/harness/test_*.py`, every `bench/readiness/*` scenario, and `python3 tools/agents/tracker.py lint`
- Scenario result for this ticket: `passes interactive-launch-and-monitors full-flow expected=checked observed=checked`

## Claims

- [R] Runtime help says interactive `traecli` transcript discovery reads session metadata from the runtime store `tools/agents/worker.py:80`
- [R] Runtime help says interactive launches pass the inline-table per-worker workspace trust override for `traecli` and `codex` `tools/agents/worker.py:84`
- [R] The runtime trust override config is `projects={"<workspace>"={trust_level="trusted"}}` `tools/agents/worker.py:468`
- [R] Interactive runtime argv includes the workspace trust config `tools/agents/worker.py:1043`
- [R] Interactive runtime argv passes the brief prompt after `--yolo` `tools/agents/worker.py:1045`
- [R] Worker launch computes runtime trust only for interactive launches `tools/agents/worker.py:1405`
- [R] Worker interactive launch writes a foreground timeout command into the tmux script `tools/agents/worker.py:1445`
- [R] Worker interactive launch keeps the window open after runtime exit by execing a shell `tools/agents/worker.py:1449`
- [R] Worker run records store private-clone workspace kind `tools/agents/worker.py:1481`
- [R] Worker run records store the interactive command `tools/agents/worker.py:1496`
- [R] Worker run records store the runtime trust record `tools/agents/worker.py:1497`
- [R] Worker run records store the tmux window record `tools/agents/worker.py:1499`
- [R] Worker tests assert both supported runtimes record the inline-table workspace trust override `tools/agents/test_worker.py:644`
- [R] Worker tests assert the dotted workspace trust override is absent from the interactive command `tools/agents/test_worker.py:667`
- [R] Worker tests assert event-stream session ids record a runtime transcript path `tools/agents/test_worker.py:639`
- [R] Worker tests assert interactive transcript discovery returns the matching session id from session metadata `tools/agents/test_worker.py:708`
- [R] Workspace transcript discovery requires the first session record's cwd to match the worker workspace `tools/agents/worker.py:530`
- [R] Workspace transcript discovery skips runtime session files older than launch `tools/agents/worker.py:533`
- [R] Workspace transcript discovery returns the newest matching session id and path `tools/agents/worker.py:540`
- [R] `worker.py status --update-run-record` is documented as the command that writes observed session and Candidate state `tools/agents/worker.py:2690`
- [R] `lead_launch.py launch --interactive` passes interactive tmux flags through to `worker.py launch` `tools/agents/lead_launch.py:660`
- [R] Interactive lead launch records the worker-reported tmux window in `lead_launch` run metadata `tools/agents/lead_launch.py:688`
- [R] Interactive lead launch refuses a missing tmux session argument `tools/agents/lead_launch.py:730`
- [R] Interactive lead launch verifies tmux availability before launching workers `tools/agents/lead_launch.py:731`
- [R] The state machine returns `landed` for resolved tickets `tools/agents/lead_launch.py:1205`
- [R] The state machine returns `blocked-on-prompt` when a live pane shows a blocking runtime prompt `tools/agents/lead_launch.py:1213`
- [R] The state machine returns `needs-info` for a clean needs-info marker commit `tools/agents/lead_launch.py:1225`
- [R] The state machine returns `waiting for re-integration` for a clean post-review successor commit `tools/agents/lead_launch.py:1230`
- [R] The state machine returns `finishing` for a marker commit while the interactive session is busy `tools/agents/lead_launch.py:1234`
- [R] The state machine returns `ready` for an unreviewed clean marker when the session is idle `tools/agents/lead_launch.py:1236`
- [R] The state machine returns `running` while the worker process is alive and no terminal state applies `tools/agents/lead_launch.py:1247`
- [R] The state machine returns `stopped` when no terminal state applies and the worker is not alive `tools/agents/lead_launch.py:1248`
- [R] The pane prompt detector recognizes the folder-trust prompt and key-wait hint `tools/agents/lead_launch.py:1280`
- [R] TraeCLI interactive busy detection uses the visible `esc to interrupt` marker `tools/agents/lead_launch.py:1294`
- [R] Ticket monitor rows include ahead count `tools/agents/lead_launch.py:1335`
- [R] Ticket monitor rows include branch-contains-integration state `tools/agents/lead_launch.py:1336`
- [R] Ticket monitor rows include dirty count `tools/agents/lead_launch.py:1337`
- [R] Ticket monitor rows include waits-for blockers `tools/agents/lead_launch.py:1339`
- [R] Ticket monitor rows include tip rewrite state `tools/agents/lead_launch.py:1341`
- [R] Ticket monitor rows include outside-declared count `tools/agents/lead_launch.py:1342`
- [R] Ticket monitor rows include diff size since reviewed Candidate `tools/agents/lead_launch.py:1343`
- [R] Ticket monitor rows include blocked prompt text `tools/agents/lead_launch.py:1346`
- [R] Ticket monitor ahead and branch-containment checks are computed in the worker's own worktree `tools/agents/lead_launch.py:1360`
- [R] Ticket monitor dirty count is computed from the worker's own worktree `tools/agents/lead_launch.py:1365`
- [R] Ticket monitor outside-declared paths are derived from the diff since the reviewed Candidate `tools/agents/lead_launch.py:1370`
- [R] `monitor-tickets --json` prints the ticket rows as JSON `tools/agents/lead_launch.py:1388`
- [R] Worker monitor rows include process uptime `tools/agents/lead_launch.py:1462`
- [R] Worker monitor rows include current pane activity `tools/agents/lead_launch.py:1464`
- [R] Worker monitor rows include context remaining `tools/agents/lead_launch.py:1465`
- [R] Worker monitor rows include workspace disk usage `tools/agents/lead_launch.py:1467`
- [R] Worker monitor rows include the last delivery record `tools/agents/lead_launch.py:1468`
- [R] `wait` treats only `running`, `finishing` and `rework` as non-triggering running states `tools/agents/lead_launch.py:1542`
- [R] `wait` triggers when a named ticket leaves the running-state set `tools/agents/lead_launch.py:1550`
- [R] `wait` includes blocked prompt text in the trigger payload when present `tools/agents/lead_launch.py:1554`
- [R] `open-session` creates only the `tickets` and `workers` tmux windows in its JSON payload `tools/agents/lead_launch.py:1593`
- [R] The scripted participant's trust config helper emits the inline-table form `bench/readiness/interactive-launch-and-monitors/scripted_participant.py:62`
- [R] The scripted participant rejects any dotted `projects.` config key before accepting the inline form `bench/readiness/interactive-launch-and-monitors/scripted_participant.py:67`
- [R] The scripted participant prints the folder-trust prompt when the trust override would not apply `bench/readiness/interactive-launch-and-monitors/scripted_participant.py:117`
- [R] The scripted participant waits for input after printing that trust prompt `bench/readiness/interactive-launch-and-monitors/scripted_participant.py:220`
- [R] The scripted participant's ticket 04 commits a ready marker when trust applies `bench/readiness/interactive-launch-and-monitors/scripted_participant.py:204`
- [R] The interactive scenario declares the ticket-21 story list `bench/readiness/interactive-launch-and-monitors/scenario.toml:2`
- [R] Readiness rows record effective runtime `bench/harness/lead_driven.py:1410`
- [R] Readiness rows record proof `bench/harness/lead_driven.py:1411`
- [R] Readiness rows record participants `bench/harness/lead_driven.py:1417`
- [R] Readiness rows record stories `bench/harness/lead_driven.py:1418`
- [R] The interactive scenario uses `tool_cmd` when invoking `lead_launch` so judged tool overrides apply `bench/harness/lead_driven.py:1444`
- [R] The interactive scenario accepts documented no-trigger wait JSON with return code 1 `bench/harness/lead_driven.py:1476`
- [R] The interactive scenario creates a record-branch worktree before launch `bench/harness/lead_driven.py:1515`
- [R] The interactive scenario launches tickets 01 through 04 with `--interactive` `bench/harness/lead_driven.py:1562`
- [R] The interactive scenario records four worker workspaces `bench/harness/lead_driven.py:1591`
- [R] The interactive scenario records the trust-override state, prompt field and wait trigger `bench/harness/lead_driven.py:1607`
- [R] The interactive scenario verifies `wait` does not trigger while ticket 01 is in rework `bench/harness/lead_driven.py:1658`
- [R] The interactive scenario verifies `wait` triggers on `waiting for re-integration` `bench/harness/lead_driven.py:1665`
- [R] The interactive scenario lands the accepted Candidate through the judged land tool `bench/harness/lead_driven.py:1708`
- [R] The interactive scenario records monitor data from ticket 01's private worker clone `bench/harness/lead_driven.py:1744`
- [R] The interactive scenario verifies effort records are absent from the main checkout `bench/harness/lead_driven.py:1764`
- [R] The interactive scenario verifies effort records are absent from the integration worktree `bench/harness/lead_driven.py:1765`
- [R] The interactive scenario verifies effort records exist in the record worktree `bench/harness/lead_driven.py:1766`
- [R] The interactive scenario expects `running`, `finishing`, `rework`, `waiting for re-integration`, `ready`, `landed`, `needs-info` and `stopped` `bench/harness/lead_driven.py:1768`
- [R] The interactive scenario fails if the private-clone monitor lacks clean/ahead state `bench/harness/lead_driven.py:1780`
- [R] The interactive scenario returns case stories in its scenario record `bench/harness/lead_driven.py:1797`
- [R] The mutation check covers the interactive scenario when `lead_launch` is stubbed `bench/harness/test_lead_driven.py:329`
- [R] The mutation check covers the interactive scenario when `land` is stubbed `bench/harness/test_lead_driven.py:330`
- [R] The scenario unit test asserts the multi-checkout layout `bench/harness/test_lead_driven.py:793`
- [R] The scenario unit test asserts four worker workspaces `bench/harness/test_lead_driven.py:794`
- [R] The scenario unit test asserts the full state sequence `bench/harness/test_lead_driven.py:795`
- [R] The scenario unit test asserts the trust override reaches ready with no prompt `bench/harness/test_lead_driven.py:806`
- [R] The scenario unit test asserts the record exists in the record worktree `bench/harness/test_lead_driven.py:816`
- [R] The scenario unit test asserts scripted runtime rows are recorded as `scripted` `bench/harness/test_lead_driven.py:818`
- [R] The scenario unit test asserts scripted proof rows are recorded as `deterministic` `bench/harness/test_lead_driven.py:819`
- [R] The scenario unit test asserts participant rows include `full-flow` `bench/harness/test_lead_driven.py:820`
- [R] The scenario unit test asserts the ticket-21 story list is recorded `bench/harness/test_lead_driven.py:821`
- [R] The scenario unit test asserts the old dotted project key does not apply `bench/harness/test_lead_driven.py:830`
- [R] The scenario unit test asserts the inline-table project config does apply `bench/harness/test_lead_driven.py:836`
- [R] The scenario unit test asserts the old dotted-form process prints the folder-trust prompt `bench/harness/test_lead_driven.py:877`
- [R] The scenario unit test asserts the old dotted-form process remains waiting for input `bench/harness/test_lead_driven.py:878`
- [R] Lead tests assert interactive launch records the tmux target `tools/agents/test_lead_launch.py:754`
- [R] Lead tests assert the interactive launch script passes `--yolo` `tools/agents/test_lead_launch.py:763`
- [R] Lead tests assert the interactive launch script retains the terminal with `exec ${SHELL:-sh}` `tools/agents/test_lead_launch.py:764`
- [R] Lead tests assert clean post-review successor commits become `waiting for re-integration` `tools/agents/test_lead_launch.py:1015`
- [R] Lead tests assert dirty post-review work remains `rework` `tools/agents/test_lead_launch.py:1029`
- [R] Lead tests assert needs-info, stopped and landed states `tools/agents/test_lead_launch.py:1037`
- [R] Lead tests assert `wait` does not return for `rework` `tools/agents/test_lead_launch.py:1076`
- [R] Lead tests assert `wait` returns for `waiting for re-integration` `tools/agents/test_lead_launch.py:1084`
- [R] Lead tests assert worker monitor reports pane activity `tools/agents/test_lead_launch.py:1166`
- [R] Lead tests assert worker monitor reports context remaining `tools/agents/test_lead_launch.py:1167`
- [R] Lead tests assert worker monitor reports disk usage `tools/agents/test_lead_launch.py:1168`
- [R] Lead tests assert worker monitor reports last delivery acknowledgement state `tools/agents/test_lead_launch.py:1170`
- [R] Lead tests assert prompt panes become `blocked-on-prompt` `tools/agents/test_lead_launch.py:1188`
- [R] Lead tests assert prompt panes wake `wait` with prompt text `tools/agents/test_lead_launch.py:1192`
- [R] Lead tests assert a real idle TraeCLI pane tail is `ready` `tools/agents/test_lead_launch.py:1216`
- [R] Lead tests assert a real idle TraeCLI pane tail reports context `95%` `tools/agents/test_lead_launch.py:1218`
- [R] Lead tests assert `open-session` creates exactly the `tickets` and `workers` windows `tools/agents/test_lead_launch.py:1243`
- [R] The run-worker skill documents the interactive lead launch command `./.claude/skills/run-worker/SKILL.md:27`
- [R] The run-worker skill documents one worker per tmux window and run record tmux target storage `./.claude/skills/run-worker/SKILL.md:29`
- [R] The run-worker skill documents the inline-table trust override `./.claude/skills/run-worker/SKILL.md:33`
- [R] The run-worker skill documents monitor commands and one-shot JSON forms `./.claude/skills/run-worker/SKILL.md:60`
- [R] The run-worker skill documents `blocked-on-prompt` for visible runtime prompts `./.claude/skills/run-worker/SKILL.md:62`
- [R] The run-worker skill documents runtime transcript discovery from session metadata `./.claude/skills/run-worker/SKILL.md:67`
- [R] The run-worker skill says the interim coordinator script is retired `./.claude/skills/run-worker/SKILL.md:74`
- [R] The practice note documents interactive launch with tmux session and private clone `docs/practice/lead-supervisor-worker.md:84`
- [R] The practice note documents the inline-table interactive trust override `docs/practice/lead-supervisor-worker.md:88`
- [R] The practice note documents prompt-blocked monitor state `docs/practice/lead-supervisor-worker.md:98`
- [R] The practice note documents TraeCLI busy detection and percent-only context `docs/practice/lead-supervisor-worker.md:103`
- [R] The practice note documents transcript discovery from session metadata `docs/practice/lead-supervisor-worker.md:107`
- [R] The practice note says the interim coordinator script is retired as an operator path `docs/practice/lead-supervisor-worker.md:118`

## Outside declared paths

- `docs/efforts/lead-worker-readiness/evidence/18-report.md`: required hand-off report for this ticket.

## For later tickets

none

## Not checked

- I did not run `tools/agents/check.sh` or cargo; the ticket explicitly forbids both.
- I did not run `tools/agents/test_sqlite_baseline.py` in the final clean gate because the lead note explicitly excluded it.
- I did not launch real model workers or real `traecli`/Codex sessions in the final gate; tests and readiness scenarios use fake/scripted runtimes.
- The passing Python suites emitted existing `ResourceWarning` noise from subprocess and `bench/harness/tasklib.py:90` paths.
