# Lead-worker-readiness Journal Index

Updated: 2026-10-05T23:54:44Z

## Open Questions

- Should the queue have a phase boundary for "build tickets landed;
  independent review before mainline; live trials later"?
- Should static dependency lists be separated from unresolved blockers in
  monitor rows, so landed dependencies stop reading as active waits?
- Should background acceptance/landing tasks have stale-duration alerts when
  integration has not moved after the lead announces that landing is underway?
- Should the monitor show `lead landing in progress` once the lead starts a
  background acceptance/landing task for a ready ticket?
- Should final `ready for review` marker commits be delayed until after the
  exact-marker full gate passes, or should the protocol name the marker as
  "pending proof" until that gate completes?
- Should repeated `ticket N: ready for review` marker commits be tracked as a
  candidate sequence, with the active final gate tied to the exact marker hash?
- Should owner-facing status and monitor rows record the current long-running
  gate command plus current suite/scenario when a worker is `finishing`?
- Should a rebase-produced `ticket N: ready for review` commit become a
  Candidate immediately, or only after the worker completes the post-rebase
  clean detached gate, report refresh and explicit handoff?
- Should monitor rows distinguish `conflicts resolved; rebase continuation or
  post-rebase gate active` from generic `running` or `finishing`?
- Should landing be shown as two explicit transitions, `integration moved` and
  `ticket resolved/worker closed`, so short gaps do not appear inconsistent?
- Should post-ready lead work, such as a live review probe or acceptance
  inspection, appear as an explicit monitor phase instead of leaving the ticket
  at plain `ready`?
- Should monitors expose a `gate passed; report/marker pending` phase after
  successful clean-checkout validation?
- Should monitors distinguish a quiet but alive long-running gate process from
  an idle or stalled worker?
- Should clean-gate attempts have stable ids and visible final states, so a
  restarted gate is not confused with one long run?
- Should compaction/resume events be first-class monitor events when they reset
  context percentages during a ticket's final validation phase?
- Should clean-gate monitor rows expose current suite, last completed suite
  family and report freshness?
- Should monitors expose "clean implementation commit; clean detached gate
  running" as a state distinct from both dirty rework and ready handoff?
- Should monitors distinguish "focused checks recovered; broad gate running"
  from the earlier debugging loop?
- Should scenario-fixture edits after a ready marker be flagged separately from
  harness and tool edits, since they can change what the proof is proving?
- Should command-chain reporting surface the final failing segment when earlier
  subcommands in the same chain pass?
- Should dirty-path summaries expose top-level areas or path samples when the
  changed surface expands during post-rebase validation?
- Should post-rebase adaptation loops show the last failing focused scenario,
  failure count and active rerun in the monitor row?
- Should "dirty after marker" be a first-class state whenever a worker edits
  after a `ready for review` commit?
- Should ready-marker candidates have a lifecycle that distinguishes tentative
  marker, amended marker and final handoff hash?
- Should ready workers display both ahead and behind counts after integration
  advances, so a stale clean candidate is visible?
- Should monitors show both the latest ready-marker hash and the current branch
  tip when a worker has made later non-marker commits?
- Should "clean but non-marker tip after rebase" be a named handoff state
  distinct from generic running?
- Should the monitor distinguish "content accepted, held for rebase" from
  ordinary ready?
- Should observer/monitor reads move to a unified record-branch API now that
  landing artifacts no longer live in the old cache directory?
- Should lead-side compound commands create typed in-progress events for
  status triage, cleanup closure, report copying, acceptance and landing
  substeps?
- Should marker commits carry an attempt id or metadata so repeated identical
  `ticket N: ready for review` subjects can be distinguished in monitors and
  logs?
- Should monitors show clean-checkout gate subphases and current suite/scenario
  progress?
- Should monitor state include an explicit `conflicted-rebase` value with
  unmerged path names, instead of showing a worker as generic `running`?
- Should clean-checkout gate progress be shown separately from ordinary
  worktree-local validation in the worker table?
- Should collision reporting include unmerged paths as well as dirty paths?
- Should a Candidate become durably "accepted" only after the lead-side gate
  succeeds, or should the protocol explicitly represent "accepted but gate
  failed"?
- Should every worker handoff require a clean detached checkout gate of the
  exact commit before the final marker, rather than trusting the worker's
  possibly dirty or stateful worktree?
- Should `rework` be split into substantive review rework versus
  order-changing reintegration handoffs?
- Should low-context, high-blast-radius workers be highlighted when they touch
  policy docs, skills and core launcher files at once?
- Should lead-launched maintenance workers, such as the disk-cleanup TraeCLI
  session, be first-class actors in the monitor even when they are not ticket
  workers?
- Should manual `tmux send-keys` interventions by the lead create typed audit
  events with target pane, reason and outcome?
- Should no-host-`/tmp` be enforced as a launch/preflight invariant across
  lead, worker, observer and cleanup tooling rather than recorded separately in
  notes, memory and follow-up tickets?
- Should owner-facing status and lead-scheduling status be separate views, with
  branch-behind/rebase facts shown in the latter by default?
- Should gate results distinguish exit-code green with warnings from clean
  green, especially for repeated `ResourceWarning` output?
- Should monitor context percentages include a compaction/resume counter, so a
  jump like ticket 18's 18% to 91% is not read as ordinary task progress?
- Should gate monitors give stable command identifiers and start times for
  repeated broad-suite runs, especially when the worker edits files between
  runs?
- Should real interactive proof runs have structured artifact metadata,
  including run-record path, trust verdict and cleanup status?
- Should the scheduler treat a ready-marker commit as stale as soon as the
  same worker dirties the tree again, even if no replacement marker exists?
- Should monitor rows include the current long-running gate command and last
  gate verdict alongside the generated activity string?
- Should repeated gate attempts be grouped by stable command identity and last
  verdict, so a lead can see whether a worker is rerunning the same failing
  check or making progress through a broader gate?
- How should reviews record a pre-rebase candidate and its later rebased
  equivalent, so a review is not orphaned from the branch head the monitor
  shows?
- Should a worker with a ready-marker commit but post-review dirty changes be
  shown as `rework` or `dirty after ready` instead of simply running?
- Should observer and lead helper tooling avoid `/tmp` by default, given that
  even small pane-capture spools can fail on this host?
- Should monitor dirty counts carry a confidence marker or reconciliation note
  when `ws.py status`, `ws.py workers` and direct git disagree?
- Should review/rework handoffs validate cited fixture paths before they are
  sent to a worker?
- Should readiness and tool tests allocate unique branch names for reruns, or
  clean them deterministically before reuse?
- Should green gates with repeated process/resource warnings be a distinct
  monitor state rather than simply "passed"?
- Should rework state show whether the worker has dirtied the tree, or is still
  only reading/evaluating review feedback?
- Should interactive review probes have a required noninteractive trust/config
  preflight before the lead starts waiting on them?
- Should monitor dirty summaries distinguish newly touched core harness files
  from fixture/report-only changes?
- Should a ticket enter an explicit lead-review/probe phase after a worker
  ready marker, so owner-facing status can show review work in progress?
- Should the worker command recorder flag stderr/error-looking output even when
  shell pipelines return exit `0`?
- Should the protocol define a "green gates but not ready" state for workers
  that have passing checks but continue investigating or broadening scope?
- Should the scheduler keep launching new tickets after a landing when older
  active workers are many commits behind the new integration tip?
- Should landed-worker tmux-window cleanup preserve last-seen pid/context
  snapshots before deleting the panes?
- Should `landed.json` be written for tickets landed through the resolve/log
  path, since tickets 17 and 20 both landed without that state artifact?
- Should the first-level protocol have a typed `needs-info` round distinct from
  review/rework, with a durable answer artifact and monitor history?
- Should monitors show a distinct "validation passed, handoff pending" phase,
  since ticket 20 had broad checks green but no Candidate commit?
- Should worker monitors split tracked dirty files, untracked ticket-owned
  fixtures and generated artifacts, since ticket 18's monitor count hid a new
  readiness scenario directory?
- Should dirty-path monitoring include diff-size deltas, since ticket 18's
  dirty count stayed at four while its patch grew substantially?
- Should monitor context percentages be annotated with compaction/resume events
  so jumps like ticket 20's 24% to 80% are not read as task progress?
- Should observer passes define an explicit cutoff time/offset when active
  transcripts are still growing, so later ticks do not skip or duplicate
  events?
- Should monitors show the active command or suite name, not only a generated
  prose status, when a worker has a long-running shell?
- Should low-context active workers get a queue-level warning before final
  report/marker work, as ticket 20 was already at 24% while still debugging?
- What cadence should independent whole-branch reviews have during a
  lead-driven effort, given that the first one found blocking cross-ticket
  defects after many green per-ticket reviews?
- How should tickets spawned from an independent review be represented as a
  causally linked phase of the run, rather than as ordinary new work?
- How should the monitor show live path collisions now that tickets 18, 19 and
  20 are all editing the agent-tooling area in parallel?
- Should `ws.py tell` become a durable first-class protocol operation, with a
  typed note record distinct from review and re-integration messages?
- Can the lead avoid moving the integration branch while workers are running,
  or does the design need tracker/lead records outside the Candidate branch?
- How should re-integration conflicts and post-rebase marker commits be exposed
  now that tickets 02 and 08 both needed rebases after integration moved?
- How should the system represent a review instruction that asks for fixes but
  explicitly says "do not make the new marker until the lead tells you to
  re-integrate"?
- How should "content accepted; re-integrate only" be represented so it is not
  mixed together with substantive review rounds?
- When parallel workers start, which declared paths overlap and which collisions
  are only visible by inspecting diffs?
- How should the monitor surface active shared-file overlap now that tickets 03
  and 09 both edit `bench/harness/lead_driven.py` and
  `bench/harness/test_lead_driven.py`?
- How should the lead decide when to hold a review or landing because another
  active worker will otherwise pay a second rebase cost?
- Are the monitor columns enough for lead decisions, or does the lead repeatedly
  fall back to direct git, tmux and transcript inspection?
- Does the revised batch-claim launch code prevent future launch-time base skew?
- Where should append-only "for later tickets" findings live so report rewrites
  during rework cannot drop them?
- What policy should apply when a worker spawns its own subagent in the same
  ticket worktree?
- What process-cleanup authority should workers have for scenario-owned
  subprocesses, and how should broad cleanup commands be reported?
- How should context compaction/resume be surfaced, given that worker context
  percentages can jump upward mid-ticket?
- Should worker launch/access profiles explicitly include built-in skills,
  memory files and other home-directory reads?
- How should Codex-worker runtime support expose or normalize Codex session
  records without making workers scrape private transcript internals?
- How should the lead's promise that no other ticket will land before a worker
  rebases be represented as an actual queue/reservation?
- Should review records distinguish fake fixtures, archived sessions and fresh
  real CLI probes when validating runtime behavior?
- If the lead launches a worker without a claim commit to avoid moving
  integration under another rebase, what state should the tracker and monitor
  show during that intentional divergence?
- What should the monitor call a worker that is clean and idle after a
  post-review fix commit whose subject is not `ready for review`?
- Should the harness and worker gates set a worktree-local `TMPDIR` or preflight
  temp-space/inode health before running all scenarios?
- What should be the durable lead-owned home for future-ticket findings that
  are discovered during review and not naturally owned by a single worker
  report?
- Should lead review/landing attempts create an in-progress artifact as soon as
  the lead starts, before a review or landing result exists?
- Should `ws.py land` and `acceptance.py` plus `land.py --ticket` write the same
  landing metadata, so monitors and observers do not depend on which path the
  lead used?
- Should the launcher expose "slot intentionally idle" with a reason, separate
  from no launchable work?
- Should lead-driven scenarios reject relative `--out` paths, or normalize them
  before worker subprocesses change directories?
- Should the monitor expose "dirty after a passing gate" separately from
  ordinary dirty/in-progress state?
- What should the lead do when a worker is near final report/commit work with
  low remaining context, as ticket 04 was at 17%?
- Should live monitor waits distinguish static dependency declarations from
  unresolved blockers, so resolved tickets do not keep appearing as waits?
- Should there be a first-class "note" or "tell" message type distinct from
  review/reintegration handoffs?
- Should `ready` mean a stable handoff waiting on the lead, or only that a
  Candidate existed at least once? Ticket 04 became dirty rework minutes after
  its clean ready marker.
- Should review findings record whether they came from a reproduced failure,
  code inspection, or a process requirement?
- How should record-branch migration preserve legacy effort directories that
  exist without an `EFFORT.md` or `Integration branch` line?
- Should the monitor show "dirty after ready marker" as a distinct active
  state?
- Should overlap reporting compare live dirty paths, declared owned paths, or
  both?
- How should long-running local fake-process test suites be distinguished from
  waits on another agent or the lead?
- How should the lead schedule or preempt when three active workers converge on
  the same harness and launcher files?
- Should context gauges show compaction/resume count so a recovered percentage
  is not mistaken for a short-running task?
- When a ticket changes canonical policy docs, should the lead treat it as a
  higher-risk lane that can block or invalidate other workers?
- Should monitors surface a "post-green, pre-marker" phase separately from
  generic running?
- Should scenario-loop progress include the current scenario name and last
  passed scenario, so the lead can distinguish slow gates from stuck workers?
- Should lead tools resolve effort records consistently from the main checkout,
  integration worktree and worker worktrees, or should the UI hard-require one
  execution root?
- Where do accepted worker reports live after an effort moves tickets and
  evidence onto a record branch?
- Should worker-runtime accounting ingest structured file edit events as well
  as shell command text?
- Can sandboxed workers in git worktrees commit at all under the default
  writable-root policy, given that `.git/worktrees/<name>/index.lock` lives
  under the main checkout?
- How should generic wrapper instructions be reconciled with a review body that
  gives a narrower hand-off protocol?
- Should time-based readiness scenarios expose deterministic thresholds in
  fixture data rather than relying on default wall-clock and quiet-time values?
- Should a worker changing canonical process policy or the workflow manifest be
  automatically highlighted as higher risk than ordinary ticket files?
- Should a ready worker create a durable queue event as soon as it becomes
  clean at a marker, before the lead starts review?
- Should collision reporting include committed but unlanded worker diffs in
  addition to current dirty paths?
- Should ticket amendments be accepted only when their acceptance criteria are
  changed, so workers cannot miss requirements embedded only in prose?
- Should the queue split clean marker commits into "ready and current" versus
  "ready but stale" when the integration branch has moved?
- Should held ready tickets get an explicit durable wait state and dependency,
  rather than staying indistinguishable from actionable ready tickets?
- Should monitors report the difference between a finished test command and a
  worker still doing report/finalization work?
- Should command supervision expose stable command IDs for repeated long gates,
  so lead and observer can distinguish a fresh rerun from continued polling?
- Should the monitor flag changes to durable practice docs and local skill
  files as a special review risk, even when they are in a ticket's declared
  scope?
- Should a lead-started review/check have an explicit durable "review in
  progress" state before the review artifact exists?
- Should ready notifications be event-driven, rather than depending on periodic
  status tables that can miss a worker finishing just after a sample?
- Should ticket-specific permission exceptions be compiled into worker briefs
  with explicit precedence over generic "no model runs" text?
- How should the system represent provider-specific sandbox results when one
  runtime succeeds and another still cannot write `.git` state?
- Should re-integration handoffs include predicted conflict hotspots and a
  merge-base report?
- Should monitors show `behind` or "rebase required" for active workers, not
  only commits ahead of the integration branch?
- Should there be a first-class "gates passed; report/marker pending" phase
  between risky validation and final hand-off?
- Should runtime probe artifacts be recorded with provider, verdict, candidate
  commit and event-stream checksum in the handoff metadata?
- How should the coordinator enforce or display the lead's "nothing will land
  before you" promise during re-integration?
- Should re-integration handoffs be typed as either mechanical rebase or rebase
  plus adaptation to newly landed behavior?
- Should gate runners set workspace-local `TMPDIR`/`TMUX_TMPDIR` by default for
  tmux-heavy tests?
- Should monitors classify generated untracked files separately from
  ticket-owned untracked evidence?
- Should capsule/bwrap gate runners pre-create `/tmp/tmux-$UID` when they bind
  a private `/tmp` for tmux tests?
- Should repeated full-gate attempts carry stable command IDs or phase names so
  observers can distinguish retries from a still-running command?
- Should shared-branch protection explicitly ignore lead-imported peer worker
  branches declared by batch metadata?
- Should generated cleanup artifacts such as `.local` be reported as a separate
  class from ticket-owned dirty paths?
- Should worker monitors show the current long-gate suite or scenario name,
  not only that one shell is running?
- What should `worker resume` guarantee about refreshing the integration ref
  inside a private worker repository?
- After a resumed worker fixes a stale Candidate, which protocol step should
  publish the successor Candidate before the lead tries to land it?
- Should low remaining context during re-integration be elevated in the lead
  monitor before the worker reaches final report/marker work?
- Should stale-base rework support same-session resume as a first-class lead
  path, separate from review rework and fresh-worker fallback?
- Should the worker monitor persist compaction events so context jumps are
  explainable without transcript inspection?
- Should lead-side review and landing gates use the same private temp/tmux
  provisioning that workers use for tmux-heavy tests?
- Should acceptance records be transactional with the landing gate, or clearly
  marked pending/failed when a gate fails?
- Should the monitor show lead-side review/landing attempts and their last
  failure for a ready ticket?
- Should `landed.json` be mandatory for every landed ticket, including tickets
  landed through the acceptance/land.py path rather than `ws.py land`?
- How should lead-sent operational notes, such as temp-directory settings, be
  folded into later worker launch environments instead of relying on manual
  notes?
- Should independent reviewers be represented as first-class run actors when
  the lead starts them outside the ticket worker pool?

## Strongest Findings So Far

- During the 23:54 pass, ticket 22 had landed and integration advanced to
  `7d2f69c`; the ticket 22 worker window was closed and all twenty build
  tickets were now on the integration branch. The lead also launched a second
  independent branch review and recorded a real Codex launch-then-resume proof.
  Tickets 14 and 15 remained blocked/unlaunched, and ticket 14's monitor wait
  list still included ticket 22 even though ticket 22 was landed.
- During the 23:44 pass, nothing observable changed after the lead started
  ticket 22 background landing task `buqu2ydin`: the lead transcript and worker
  transcript were unchanged, integration remained at ticket 18's `3ab9ec4`,
  ticket 22 stayed clean and ready at `7d2f69c`, and the state directory still
  had no `landed.json` or new review artifact. The monitor still showed only
  `ready`, not lead-side landing in progress or stale.
- During the 23:34 pass, ticket 22 completed its worker-side exact-marker gate
  and handed off final commit `7d2f69c`; the coordinator showed it as `ready`,
  clean and two commits ahead. The lead noticed within about a minute and
  started background acceptance/landing task `buqu2ydin`, but integration
  remained at ticket 18's `3ab9ec4` and ticket 22 still had no landing artifact
  by the sample. The monitor had no state between `ready` and landed for that
  lead-side work in progress.
- During the 23:24 pass, ticket 22 remained clean at report-refreshed marker
  `7d2f69c`, two commits ahead of integration, while the exact-marker detached
  gate was still running. The gate had moved past `test_worker.py`, `test_wt.py`
  and `bench/harness/test_fleet.py` into the long
  `bench/harness/test_lead_driven.py` suite. The monitor showed only
  `finishing`, `2` ahead, `0` dirty and one shell running.
- During the 23:14 pass, ticket 22 created a second `ticket 22: ready for
  review` marker, `7d2f69c`, that only refreshed `22-report.md` after the
  prior clean detached gate passed. The worker then started a new exact-marker
  detached gate for `7d2f69c`; by cutoff it was still running in
  `tools/agents/test_worker.py`. The monitor showed ticket 22 as
  `finishing`, `2` ahead and clean, but did not distinguish first marker
  `244f3da` from the later marker under proof.
- During the 23:04 pass, ticket 22 still had clean candidate-shaped commit
  `244f3da`, but its clean detached gate had run from `22:46:16Z` through at
  least `23:04:55Z` without a final verdict. The worker reported all
  `bench/harness/test_*.py` suites done and readiness scenarios in progress,
  including `interactive-launch-and-monitors`; the monitor showed only
  `finishing`, `1` ahead, `0` dirty and an incidental activity string.
- During the 22:54 pass, ticket 22 was clean at rebased marker `244f3da` and
  the monitor showed `finishing`, `1` ahead and `0` dirty, but the worker was
  still inside its required clean detached full gate. The pane showed progress
  through tools suites, `test_worker.py`, `bench/harness/test_fleet.py`, and
  into `bench/harness/test_lead_driven.py`; no final gate result, report
  refresh or explicit handoff existed by the cutoff.
- During the 22:44 pass, ticket 22 resolved the reintegration conflicts from
  `.claude/skills/run-worker/SKILL.md`, `tools/agents/lead_launch.py` and
  `tools/agents/worker.py`, passed focused syntax plus worker/lead-launch
  checks, and completed `rebase --continue` to clean candidate `244f3da`.
  The monitor first showed detached HEAD with `15` dirty paths, then about a
  minute later showed `finishing`, `1` ahead and `0` dirty while the worker was
  preparing the clean detached full gate.
- During the 22:33 pass, ticket 18 landed: the lead reported acceptance and
  landing records for candidate `3ab9ec4`, and integration advanced to that
  commit. The first monitor sample already showed integration at `3ab9ec4`
  while ticket 18 still appeared `claimed`/`ready`; after `ws.py resolve 18`,
  the monitor showed ticket 18 `resolved`/`landed` and its window was closed.
  Ticket 22 then received a content-accepted reintegration note, entered
  `rework`, and hit rebase conflicts in the shared launch/worker surfaces.
- During the 22:23 pass, ticket 18 reached final ready marker `3ab9ec4` after
  refreshing its report and passing `git diff --check` plus `tracker.py lint`.
  The lead noticed within about two minutes, ran status/report checks and a
  throwaway probe, then began longer review/acceptance work; no review record,
  acceptance record or landing existed by the cutoff. The monitor showed
  ticket 18 as simply `ready`, not "lead review in progress".
- During the 22:13 pass, worker 18 reported that `clean-gate.ZDtV04` exited
  `0` for commit `9c4934c`, then moved into report/source-anchor work; the
  branch was still clean at the non-marker implementation commit and no final
  `ticket 18: ready for review` marker existed. The monitor showed generic
  "Considering output approach" rather than `gate passed; report pending`.
- During the 22:03 pass, ticket 18 stayed clean at `9c4934c`; its restarted
  clean-gate attempt `clean-gate.ZDtV04` passed the lead-launch area and all
  required `tools/agents` tests, then sat in the long
  `bench/harness/test_lead_driven.py` run with no final result, report refresh
  or ready marker. The monitor only said "Considering analysis methods".
- During the 21:53 pass, owner status prompted the lead to check tickets 18 and
  22, close stale landed-ticket windows 19 and 21, and report that 18 was still
  in full clean-gate validation. Worker 18 stayed clean at `9c4934c`, but its
  context jumped from 29% to 88% and a new detached gate path
  `clean-gate.ZDtV04` started before the final report or marker.
- During the 21:43 pass, ticket 18 was still clean at non-marker commit
  `9c4934c`, but the detached clean gate had only progressed into full
  `bench/harness/test_lead_driven.py`; no final result, report refresh or
  ready marker existed. The monitor showed a generic "Evaluating testing
  sequence" activity and 29% context left.
- During the 21:33 pass, ticket 18 advanced from dirty broad-gate work to a
  clean non-marker implementation commit `9c4934c`; the worker then launched a
  detached clean-gate worktree and was still running `tools/agents/test_lead_launch.py`.
  The coordinator correctly showed `0` dirty and `5` ahead, but its activity
  text still said "Run narrow tests after conflict resolution...", masking the
  clean-gate phase.
- During the 21:23 pass, ticket 18 showed real gate progress: three focused
  lead-launch tests and two focused harness tests passed at `21:17:44Z`,
  `tools/agents/test_worker.py` passed `86` tests at `21:20:16Z`, and
  `tools/agents/test_lead_launch.py` passed `46` tests at `21:22:08Z`. The
  worker then started a broader `bench/harness/test_lead_driven.py` command,
  still with five dirty files and no fresh ready marker.
- During the 21:13 pass, ticket 18 recovered the focused two-test harness pair
  at `21:09:25Z`, but a broad `tools/agents/test_lead_launch.py` run then
  failed at `21:11:37Z` with two errors. The worker fixed a fake-tmux path
  issue and got a focused three-test lead-launch command green at `21:14:27Z`,
  while the worktree stayed dirty across five files, including the
  `interactive-launch-and-monitors` scripted participant, and no ready marker
  existed.
- During the 21:03 pass, ticket 18 stayed at non-marker `60f6d22` with the
  same four dirty files. The lead-launch status-vocabulary subtest passed, but
  the chained focused harness tests failed again at `20:57:04Z` and
  `21:01:23Z`, each with one failure. The worker then ran the
  `interactive-launch-and-monitors` scenario directly under
  `/data02/home/philip.yang/devx/tmp/lwr-18/inspect-interactive.aJ0xTM` and
  inspected acknowledgement/status code; no fresh marker or broad green gate
  was observed.
- During the 20:53 pass, ticket 18's branch head stayed at non-marker
  `60f6d22`, but its dirty set expanded to `bench/harness/lead_driven.py`,
  `bench/harness/test_lead_driven.py`, `tools/agents/lead_launch.py` and
  `tools/agents/test_lead_launch.py`. A focused two-test harness rerun failed
  at `20:50:53Z`, then a focused lead-launch status-vocabulary test failed at
  `20:53:13Z` and passed at `20:54:26Z`; the monitor showed only four dirty
  files, "Revising session status checks" and one running shell.
- During the 20:43 pass, ticket 18 moved from the prior clean non-marker tip
  back into dirty harness rework. The same combined py_compile plus two focused
  `bench/harness/test_lead_driven.py` methods failed at `20:43:38Z` with two
  failures, then failed again at `20:46:54Z` with one failure after more
  `bench/harness/lead_driven.py` patches. The monitor showed `running`, two
  dirty files and later "Inspecting command errors", but not the failed command
  or rerun history.
- During the 20:33 pass, ticket 18 completed its rebase over landed ticket 21:
  after interim commits `b23d1c4` and `f3acd85`, the rebase finished at
  `60f6d22` (`ticket 18: integrate landed worker behavior`) at `20:32:18Z`.
  Direct git then showed a clean worktree, `0` behind and `4` ahead, but the
  worker remained `running` rather than ready because it was auditing the
  interactive scenario against ticket 21's new rules.
- Ticket 18's current branch tip was not a ready-marker commit even though
  ready-marker commits `b23d1c4`, `f3acd85` and `05b7616` were in its recent
  history. The monitor showed only the tip subject, not the latest marker
  relationship.
- Ticket 22 remained clean at final ready marker `738ead5`, but after ticket
  21 landed direct comparison showed it was `6` behind and `1` ahead of
  integration. The coordinator still displayed only `1` ahead and `0` dirty.
- No lead transcript bytes appeared after the 20:20 public status during this
  pass; the run was effectively waiting on ticket 18's post-rebase validation.
- During the 20:23 pass, ticket 21 landed and integration advanced to
  `94e66ec`. The lead log says the new record-branch flow accepted the second
  candidate, the lander wrote its own landing record, and tracker resolution
  succeeded.
- Ticket 18 committed `514bd31` (`ticket 18: integrate landed worker
  behavior`) at `20:23:40Z`, then rebased onto `effort/lead-worker-readiness`
  at `20:23:47Z` and hit conflicts in `bench/harness/lead_driven.py`,
  `tools/agents/lead_launch.py` and `tools/agents/test_lead_launch.py`. Direct
  git later showed `HEAD` at `94e66ec`, `0` ahead and `20` dirty, including
  the three unmerged files.
- Ticket 22 stayed clean at final marker `738ead5`, and the lead's detached
  checkout review passed the new no-host-`/tmp` check, `test_sqlite_baseline`
  and a fake worker temp-root proof. The lead marked the content accepted but
  held its rebase until ticket 18 lands.
- The old coordinator cache directory had no new ticket 21 `landed.json` even
  though the monitor showed ticket 21 landed; the lead log says landing records
  now belong to the record branch.
- During the 20:13 pass, ticket 21 became clean and ready at marker `94e66ec`
  after completing the second clean-checkout scenario loop, updating its
  report and committing a new marker at `20:09:16Z`; no lead review/landing
  artifact existed by the observed cutoff.
- Ticket 22 became clean and ready at final marker `738ead5` after first
  creating marker `4450f81`, then amending twice to correct duplicate
  co-author trailers. A lead status sample saw the short-lived `4450f81` hash
  before the final handoff hash existed.
- Ticket 18 remained at stale marker `c84af72` but was running with four
  post-marker dirty files. It hit `test_worker.py` and `test_lead_launch.py`
  failures after the marker, patched `test_worker.py`, `lead_launch.py` and
  `test_lead_launch.py`, and only the final focused `test_lead_launch.py` run
  had passed by `20:17:12Z`.
- At `20:10Z` the lead checked status, read the disk-cleanup report, then
  started a compound command that closed the `disk-cleanup` tmux window, copied
  the cleanup report into the records worktree, inspected ticket 21 evidence
  and began acceptance work. No transcript result for that compound command was
  visible by the `20:17:18Z` cutoff.
- Ticket 18 resolved its rebase far enough to create new ready-marker commits
  `d82ac25` and `c84af72` by `20:01:53Z`, then dirtied
  `tools/agents/lead_launch.py` and `tools/agents/worker.py` and started
  `python3 tools/agents/test_worker.py`; the monitor showed `running`, `3`
  ahead and `2` dirty.
- Ticket 21's post-review commit `3df6674` passed the full clean-checkout
  `bench/harness/test_lead_driven.py`, broad tools unittest and remaining
  bench harness gates, then completed one clean-checkout all-scenario loop and
  started a second per-scenario loop by the `20:05Z` cutoff.
- Ticket 22 recovered after its `19:54Z` broad tools failure: focused checks,
  broad tools, `test_sqlite_baseline`, full harness units, all readiness
  scenarios, tracker lint, the new `90-no-host-tmp.sh` check and `py_compile`
  all passed by `20:04:19Z`; it still had 14 dirty paths and no commit.
- Ticket 18 started the required rebase at `19:47:29Z`, the rebase failed, and
  by `19:54Z` direct git showed unmerged conflicts in
  `bench/harness/test_lead_driven.py`, `tools/agents/lead_launch.py`,
  `tools/agents/test_lead_launch.py` and `tools/agents/worker.py`. The monitor
  showed only `running`, `0` ahead and `20` dirty.
- Ticket 21 produced follow-up commit `3df6674` at `19:48:25Z`, passed the
  full `bench/harness/test_lead_driven.py` suite in a clean detached checkout
  by `19:52:51Z`, and was still running the broad tools unittest gate at the
  `19:54Z` cutoff.
- Ticket 22 compacted context during the interval and reached 14 dirty paths.
  Its broad tools unittest gate started at `19:50:10Z` and failed at
  `19:54:06Z`; it then inspected temporary-directory helper usage.
- Ticket 21 reached clean ready marker `b9cfd0b`, but the lead's clean-checkout
  gate failed deterministically: the full `bench/harness/test_lead_driven.py`
  suite reported 35 tests with 1 error, and three focused reruns of the failing
  test also failed. The lead recorded an acceptance commit `e582325` and then
  record-branch commit `5fb3474` requesting rework for the same Candidate.
- Ticket 18 reached clean ready marker `49b6c8f`, then received a
  non-review reintegration handoff at `19:46:27Z` because ticket 21 failed its
  gate and ticket 18 should now land first. The monitor showed ticket 18 as
  generic `rework` even though the artifact explicitly said it was not a review
  round.
- By the `19:47Z` sample, ticket 22 had expanded to launcher, worker, lander,
  SQLite baseline, acceptance tests, harness tests, `AGENTS.md`, a run-worker
  skill and practice docs, with 14 dirty paths and only 17% context left.
- At `19:26Z`, the owner redirected the lead toward disk cleanup, thorough
  integration review, fixes and landing. The lead launched a separate
  disk-cleanup TraeCLI worker outside the normal ticket rows, recorded a
  no-host-`/tmp` decision and ticket 22, patched scratch `ws.py`, committed
  record-branch commit `9b1d350`, and launched ticket 22 from integration
  `a6d72b6`.
- By the `19:36Z` sample, tickets 18 and 21 were both clean and ready: ticket
  18 at `49b6c8f` after review-2 gates, and ticket 21 at `b9cfd0b` after a
  reintegration fix. Ticket 18 still compared behind integration, while ticket
  21 was current.
- The one-off cleanup worker freed about `1.45G` under
  `/data02/home/philip.yang/devx/tmp` while skipping live `lwr-19` temp state,
  but it was invisible in `ws.py workers`; only pane and lead transcript
  inspection exposed it.
- The live run began with ticket 01 only. It was launched at
  2026-10-05T08:04:52Z from integration commit `3a476b0` into
  `/data02/home/philip.yang/workspace/corenius-lwr-01-one-scenario-end-to-end`.
- The scratch coordinator now includes status/worker monitors, a marker-commit
  hand-off rule, gated fast-forward landing, review sending, an observer launch
  path and a ticker. The observer/ticker support was added during the run.
- The lead identified a design problem before the first Candidate finished:
  lead commits on the integration branch can stale running worker Candidates.
- Worker 01 was still running at 08:18:30 with no commits ahead, seven dirty
  paths and active red-green test progress. No review, rework or landing had
  occurred yet.
- By 08:23, worker 01 had passed focused worker tests, lead-driven/fleet tests,
  all bench harness tests, and hand-off scenario CLI checks, then started the
  required `CHECK_CARGO=0 sh tools/agents/check.sh` gate. It still had no
  commits ahead and no hand-off marker.
- By 08:33, worker 01 was still waiting on the same required gate. Partial
  output showed repeated `checks.d/70-sqlite-controller.sh` native-controller
  failures around missing `/run/attempt-gate`, outside the ticket's changed
  Python/doc paths.
- The launch baseline for ticket 01 was recorded before the worker launch, but
  it is not yet the recursive external-state baseline required by the readiness
  stories.
- Worker 01 disk use grew from 214M at 08:18 to 370M at 08:23 and 928M at
  08:33; the monitor made this visible without extra inspection.
- Ticket 01 produced an initial marker commit at 08:41:19, then rewrote its two
  commits at 08:42:05 to fix duplicate attribution. The final marker was clean,
  but this proved the marker commit alone is not a stable Candidate signal for
  interactive workers.
- The lead reviewed Candidate `c153084`, reran focused tests and scenario
  commands on a detached checkout, sampled six claims, then sent `review-1.md`
  at 08:44:44. Ticket 01 entered rework without landing.
- The first review found two blocking design gaps despite green tests: the
  scenario did not verify the toolbox's own verdict against git, and the
  hand-off rule assumed a worker process that exits.
- `ws.py` received a second live-run patch so marker commits can be reported as
  `finishing` while the pane is busy and `rework` after the Candidate has been
  sent back.
- During rework, worker 01 added marker-mode and toolbox-verdict tests. The
  lead-driven suite failed first on missing marker participants, then passed 9
  tests at 08:53; the worker tree remained dirty, so no new Candidate had been
  handed off.
- Worker 01 produced second Candidate `2bc9124` at `09:02:39Z`, clean and
  without rewriting the previously reviewed commits. The lead reviewed the
  delta and landed that exact Candidate at `09:05:18Z`.
- Ticket 01 completed one full lead-driven loop: about 37 minutes to first
  stable Candidate, about 18 minutes of same-session rework after review, then
  about 3 minutes from second Candidate to landing. The lead's manual log labels
  drifted from artifact timestamps, so auto-stamped events matter.
- The landing artifact for ticket 01 records the exact Candidate, previous
  integration tip and gate command: worker tests, every bench harness test, the
  default handoff scenario, seven optional handoff participants, and tracker
  lint.
- After landing ticket 01, the lead committed resolution/acceptance records and
  launched workers for tickets 02, 08 and 13 around `09:06Z`.
- The launch sequence made ticket 02 start at `e49ce68` and ticket 08 at
  `3c5f682` while integration advanced to `ebc7c9f`; the monitor still showed
  them as `0` ahead and clean. This exposed a missing `behind` or base-skew
  signal.
- The lead patched the coordinator again so future multi-ticket launches claim
  the whole batch in one integration commit before any worktree is cut.
- Rework rewrote the ticket 01 report enough to drop earlier "for later
  tickets" findings. The lead re-recorded them, suggesting that cross-ticket
  hand-offs need an append-only home outside mutable reports.
- By the 09:12 tick, workers 02, 08 and 13 were all running with dirty trees and
  no new commits. All three had uncommitted changes to
  `bench/harness/test_lead_driven.py`; workers 02 and 13 also both touched
  `bench/harness/lead_driven.py`. The coordinator monitors exposed dirty counts
  but not the shared paths.
- Early parallel work followed a red-green pattern across workers: worker 02's
  `test_land.py` failed then passed; worker 08 saw worker and lead-driven test
  failures with a narrow worker test passing; worker 13's lead-driven suite
  failed then passed and `test_run.py` passed before a full harness discovery
  run was still in progress.
- Ticket 13 reached a first marker Candidate `37b0f5b` at `09:17:05Z`; the
  lead reviewed it at `09:17-09:18`, found two should-fix process/reporting
  issues despite passing tests and sampled claims, and sent `review-1.md` at
  `09:18:47Z`.
- Ticket 13 then reworked additively, as instructed: it committed `26652eb`
  (`ticket 13: address review findings`) and second marker Candidate `fcfd998`
  at `09:24:02Z`, without rewriting `37b0f5b`.
- The marker-written-past pattern repeated on ticket 13: at `09:23`, the branch
  head was still first marker `37b0f5b`, but the tree was dirty and the worker
  was active. The coordinator kept it `running` until the tree was clean and
  idle, so the `finishing`/clean/idle rule did useful work.
- By the 09:22 tick, all three active workers had touched both
  `bench/harness/lead_driven.py` and `bench/harness/test_lead_driven.py` during
  the interval. Ticket 13 became ready first; tickets 02 and 08 were still
  dirty and running, so a ticket 13 landing would move shared files under them.
- Ticket 13 landed at `09:25:53Z` as exact Candidate `fcfd998`; the lead then
  committed resolution/acceptance records and launched ticket 05 at
  `09:26:14Z`.
- Ticket 02 produced first Candidate `955059a` at `09:27:22Z`. The lead sent
  `review-1.md` at `09:29:43Z`, requiring a fix for landing onto checked-out
  target branches, re-integration onto the moved integration tip, and scenario
  story metadata.
- Ticket 02 performed the first observed re-integration: `git rebase
  effort/lead-worker-readiness` failed at `09:30:32Z`, conflict inspection and
  py_compile followed, and rebase continuation succeeded at `09:31:56Z`. The
  worker later wrote past the new marker `07028af`, so the monitor moved it
  from `finishing` back to `running`.
- Ticket 05's worker spawned a same-worktree explorer subagent at `09:30:13Z`.
  Selected records showed read-only exploration so far, but this is a new
  process boundary inside the "one worker per ticket" pattern.
- Ticket 02 continued review-driven rework after the rebase marker `07028af`:
  it committed `4f4ad71` (`fix checked-out target landing`) and `1e840cf`
  (`declare landing scenario stories`), then ran a growing gate through
  `test_tasks_tracked.py` by `09:43:50Z`. Because `HEAD` was not a marker and
  the pane was active, the coordinator correctly kept it `running`.
- Ticket 05 remained pre-commit by the 09:42 tick, with nine dirty paths across
  tracker, lead-launch, harness, scenario and ticket-format files. Its tests
  alternated between failures and passes before the latest observed
  `test_lead_launch.py` plus scenario run passed at `09:43:47Z`.
- Ticket 08 remained pre-commit but had passed worker, tools/agents discovery,
  and bench/harness discovery suites by `09:43:27Z`; it was writing its report
  and final marker while still dirty.
- Worker context percentages are not monotonic. Worker 02 and worker 08 showed
  higher context remaining after compaction/resume than in earlier snapshots.
- Ticket 02 completed the full lead-driven loop and landed at `09:50:45Z` as
  exact Candidate `cddfa2087d51644861ed39ee1f76e5ade1aa6fd9`. Its visible
  phases were about 21m20s launch-to-first-Candidate, 2m21s
  first-Candidate-to-review, 18m39s review-to-final-Candidate, 2m23s
  final-Candidate-to-landing, and 44m43s total launch-to-land.
- After ticket 02 landed, the lead recorded ticket 02 acceptance/resolution,
  claimed and launched ticket 03 from integration tip `3c6d325`, then sent the
  held ticket 08 review at `09:51:14Z`.
- Ticket 08's first review required re-integration, scenario-story metadata and
  an explanation for `GIT_OPTIONAL_LOCKS=0`. Its rebase failed at `09:52:03Z`
  and continued successfully at `09:53:34Z`.
- Ticket 08 repeated the marker-written-past behavior after review: it created
  post-rebase marker `beca9c1` at `09:53:33Z`, then committed
  `e5b0f56` (`Declare access-profile scenario stories`) at `09:55:28Z`.
- Worker 05 passed broad local gates by `09:55:01Z` but was still dirty and
  running. Its transcript included `pkill` cleanup commands for scenario-owned
  processes, making process authority and cleanup reporting a design question.
- Ticket 08 landed at `10:03:55Z` as exact Candidate `5b1f23a`; the lead then
  recorded ticket 08 acceptance/resolution and launched ticket 09 from
  integration tip `599a306` at `10:04:15Z`.
- The lead patched scratch `ws.py` at `10:04:13Z` to include
  `GIT_OPTIONAL_LOCKS=0` handling for monitor git calls, matching ticket 08's
  lock-avoidance finding.
- Ticket 05 produced first marker `be64d24` at `10:03:12Z`; review 1 was sent
  at `10:05:05Z` with a blocking real-layout defect, re-integration, hand-off
  line format cleanup and access-profile pass-through. The worker rebased and
  produced new marker `ce06d44` at `10:07:21Z`, while the pane was still
  active.
- The ticket 05 review provides another lead-account discrepancy: the lead log
  says the review took `~9 min`, but selected lead transcript events and the
  review artifact place the visible review window from about `10:04:21Z` to
  `10:05:05Z`.
- Ticket 03 is in active red-green implementation for acceptance records:
  `test_acceptance.py` failed then passed, several land/harness tests cycled
  through failures and passes, and by `10:06:50Z` it had passed multiple local
  suites while still uncommitted and dirty.
- Ticket 09 began by reading skills, AGENTS/CONSTITUTION/effort context and
  worker code, and searched home-level memory before implementation; by
  `10:07:48Z` it was running with one dirty path.
- From `10:07:50Z` to `10:13:24Z`, no new lead actions or coordinator state
  artifacts appeared; progress was worker-side only.
- Ticket 05 wrote past its post-rebase marker: direct git log showed
  `d400932` (`ticket 05: address lead review`) after marker `ce06d44`, while
  the monitor correctly kept the worker `running` because the pane was active
  and the tip was not the marker.
- Ticket 03 passed additional acceptance, land, claims, lead-driven, run,
  validate, fleet, task-tracking, tracker-lint and diff-check commands but
  remained `0` ahead and `12` dirty.
- Ticket 09 alternated external-protected worker test failures and passes, then
  passed focused external-state lead-driven tests and a combined worker plus
  lead-driven command by `10:12:36Z`; it remained `0` ahead and `5` dirty.
- Ticket 03 produced first Candidate `3128942` at `10:15:12Z`; the lead
  reviewed it by reproducing an acceptance-record location/order defect and
  sent `review-1.md` at `10:17:19Z`.
- Ticket 03's review protocol told the worker to commit fixes and stop without
  a new marker until the lead sends the re-integration message, creating a new
  rework sub-state not covered by earlier marker-only handoffs.
- Ticket 05 wrote another post-marker commit `93dbb4f` (`ticket 05: stabilize
  rebased harness`) at `10:17:52Z`. It was clean but still active and not at a
  ready-marker subject, so the monitor kept it `running`.
- Ticket 09 produced final ready marker `f8588e1` at `10:20:51Z` after a
  commit/amend sequence. The lead began reviewing it by `10:21:51Z` and invoked
  a gate command at `10:23:58Z`, but no ticket 09 review or landing artifact
  existed by the final `10:24:18Z` scan.
- Ticket 09 became the first observed no-rework ticket: it landed exact
  Candidate `f8588e1` at `10:25:21Z`, about 21m06s after launch, and the lead
  recorded three known limits in the acceptance record.
- Ticket 12 launched from integration tip `64a3692` at `10:25:31Z`. Its worker
  immediately inspected Codex/TraeCLI runtime commands and session file shapes,
  then hit initial worker-runtime and lead-driven test failures by `10:33:56Z`.
- Ticket 03 produced a post-review marker `8900967` at `10:27:18Z` despite the
  review protocol saying to commit fixes and stop without a marker until the
  lead sent a re-integration message. No selected lead action or state artifact
  showed that message before the marker.
- Ticket 05 reached final marker `fe86c18` at `10:33:21Z` after multiple
  marker-invalidating commits, broad gate reruns, and process-state inspection;
  the `10:34:23Z` monitor showed it `ready`, clean and idle.
- At `10:34:49Z`, the lead sent ticket 05 a re-integration-only message for
  Candidate `fe86c18`: content accepted, Candidate behind because ticket 09
  landed, rebase once more, and "nothing will land ahead of you this time."
- Ticket 05's rebase failed at `10:35:14Z`, continued successfully at
  `10:36:40Z`, then produced rebased marker `1e8b278`. The worker subsequently
  dirtied `tools/agents/lead_launch.py` and `tools/agents/test_lead_launch.py`,
  so the 10:45 monitor kept it `running` despite a ready-marker subject.
- Ticket 03 stayed clean and idle on marker `8900967`; the lead log says its
  content looked right but re-integration was being held until ticket 05 lands.
- Ticket 12's worker moved from initial failures to passing focused runtime
  worker tests, focused Codex-runtime harness tests, full `test_worker.py`, full
  bench/harness discovery and four lead-driven scenarios by `10:44:38Z`, while
  remaining uncommitted and dirty.
- Ticket 12 reached first Candidate `9548178` at `10:47:08Z` and received
  review 1 at `10:51:58Z`. The lead ran real `codex exec --json` probes before
  review and found the worker's Codex sample did not match the real emitted
  format.
- The lead log says the ticket 12 Codex probe was the first model spend by the
  lead in this run and attributes the bad sample to the lead's ticket wording.
- Ticket 03 remained clean and idle on Candidate `8900967`, but was still held
  behind ticket 05 for re-integration sequencing.
- Ticket 05 remained active after marker `1e8b278`: by `10:53:33Z` it had dirty
  changes in `bench/harness/test_lead_driven.py`, `tools/agents/lead_launch.py`
  and `tools/agents/test_lead_launch.py`.
- Ticket 05 finally landed at `11:00:18Z` as exact Candidate `028d837`, about
  94 minutes after launch. The lead then sent ticket 03 its held
  re-integration message and launched ticket 11 from the new integration tip.
- The lead changed scratch `ws.py` at `11:00:41Z` to launch ticket 11 without a
  claim commit because ticket 03 was mid-rebase. That avoided another immediate
  integration move, but the tracker showed ticket 11 as `open` while its worker
  was running.
- Ticket 03's re-integration rebase failed at `11:01:03Z`, then continued
  successfully at `11:04:09Z`. By `11:06:49Z`, it was clean at rebased marker
  `1e04c59` but still running required checks.
- Ticket 12 committed review fixes at `11:02:57Z` as `f7b794c` (`ticket 12:
  address review 1`) and was clean/idle, but the monitor still only showed
  generic `running` because the tip was not a ready marker.
- Ticket 11 began with read-heavy setup, then moved into test-driven changes:
  its first selected tracker and lead-launch tests failed around `11:05Z`, and
  the worker had dirty files but no commits by the live `11:06Z` monitor.
- Ticket 03 landed at `11:15:36Z` as exact Candidate `dc96ae0`. Before its
  final marker, the worker hit `/tmp` inode exhaustion while running all
  scenarios, then reran successfully with `TMPDIR` under the worktree
  `.scratch`.
- The integration history for ticket 03 contains multiple `ticket 03: ready for
  review` commits (`ef98821`, `1e04c59`, `dc96ae0`), reinforcing that the
  Candidate hash, not the subject, is the hand-off identity.
- Ticket 11 ran for about 15 minutes before the lead recorded its deferred
  claim on integration at `df61ccf`. The lead had intentionally launched it
  without a claim commit to avoid moving integration under ticket 03's rebase.
- Ticket 07 launched at `11:15:56Z` from integration tip `8396875` after ticket
  03 landed. By the 11:23 monitor it was running with dirty files and no
  commits ahead.
- Ticket 11 received review 1 at `11:17:38Z` for Candidate `1aca3a5`. The
  blocking finding corrected the ticket's original source model: this run now
  needs findings collected from worker reports and a lead handoff file, not
  only ticket files.
- Ticket 12 received re-integration permission at `11:15:57Z`, rebased with an
  initial conflict at `11:16:45Z`, continued successfully at `11:18:05Z`, and
  was running expanded checks on rebased head `28fb77e` by the 11:23 monitor.
- Ticket 12 produced a clean final Candidate `536a2b1` at `11:32:07Z` after
  another scenario-loop failure under default temp placement and a successful
  rerun with `TMPDIR` under worktree `.scratch`.
- Ticket 11 committed review fixes at `11:33:05Z` as `850c0a2`, stayed clean
  and idle, and explicitly did not create a new `ready for review` marker
  because the review told it to wait for re-integration after ticket 12.
- At `11:33`, the lead began acting on ticket 12's ready Candidate, but there
  was no review/landing artifact and no selected command result by the
  journal's `11:35:06Z` cutoff.
- Ticket 07 remained in active implementation at the 11:33 monitor: no commits
  ahead, `10` dirty paths, one focused integration-branch scenario passing and
  broader checks still running.
- Ticket 12 landed around `11:35` as Candidate `536a2b1` through the effort's
  own `acceptance.py` and `land.py --ticket` tools, not the scratch
  coordinator `ws.py land` path. Durable `acceptance-12.jsonl` exists, but the
  scratch cache lacked `12-codex-worker-runtime/landed.json` during the 11:42
  pass.
- The lead added ticket 16 (`bookkeeping-off-code-branch`) while landing ticket
  12. It records the repeated root cause that lead bookkeeping commits make
  running code Candidates stale.
- Ticket 04 launched at `11:36:03Z` after ticket 12 landed. Its amended ticket
  now includes the live-run lessons about two worker-session kinds and the
  "fixes committed, waiting for re-integration" state.
- Ticket 11 received re-integration permission at `11:36:04Z`, hit a rebase
  conflict at `11:36:51Z`, continued successfully at `11:37:24Z`, and produced
  new clean Candidate `716019b` at `11:44:18Z`.
- Ticket 07 produced clean Candidate `4891fa9` at `11:44:26Z`, just after the
  11:43 monitor snapshot still showed it dirty/running.
- Ticket 11 landed at `11:48:15Z` as Candidate `716019b` through durable
  acceptance plus `land.py`; `acceptance-11.jsonl` records required checks
  `gate` and `real-findings-collected`, with 25 sampled claims all unrecorded.
- The lead sent ticket 07 review 1 at `11:48:16Z` after reproducing that
  `land.py` and `effort.py status` fail from the main checkout when the effort
  record exists only in the integration worktree. The lead log identifies this
  as the fourth two-worktree defect after tickets 02, 05 and 03.
- The lead changed `ws.py` at `11:48:15Z` to add the three-checkout layout as
  standing brief guidance for future workers.
- The lead held ticket 06 even with a free worker slot because amending and
  claiming it would create integration commits while ticket 07 was reworking.
- Ticket 04 found that `lead_driven.py rework-round --out` failed with a
  relative `.scratch` path but passed with an absolute `mktemp -d` path.
- During the 12:02 pass there were no new integration commits or
  coordinator-cache artifacts; the lead transcript had no new selected records
  after the prior `ws.py wait 04 07`.
- Ticket 04's broad tool gate passed at `11:59:25Z`; the first full harness
  loop failed at `12:00:28Z`, passed at `12:02:37Z` after a harness-test edit,
  and the all-scenario loop passed at `12:03:20Z`. It then made another
  cleanup edit and remained dirty with no Candidate.
- Ticket 04's context dropped to 17% by `12:03` while it still needed report
  and final marker work.
- Ticket 07's broad review gate failed at `11:59:06Z`; after scenario/harness
  corrections, `test_lead_driven.py` passed at `12:03:01Z` and a full review
  gate started at `12:03:56Z`. It remained dirty with no new final marker.
- Ticket 07 produced new clean Candidate `c75abcd` at `12:11:13Z`, passed the
  lead's three-checkout reproduction, and landed around `12:14-12:15`.
- Ticket 07 acceptance required `gate` and `three-checkout-reproduction`; its
  sampled `07-report.md` claims were again all unrecorded (`21` claims).
- After ticket 07 landed, the lead immediately found the new effort-record
  lookup failed on the real repository for older efforts copied across many
  worktrees without an `Integration branch` line.
- The lead added a scratch `ws.py tell` command and sent ticket 16 a note at
  `12:15:46Z` to fix the just-landed lookup defect as part of bookkeeping work.
- Tickets 06 and 16 launched together at `12:15:04Z` and `12:15:05Z` from
  claim commit `253a5d6`, after ticket 07 landed.
- Ticket 04 recovered context after compaction and continued verification:
  worker/harness/scenario checks passed again, then it added a distinct
  `fixes-committed-waiting-for-re-integration` status branch and remained
  dirty with no Candidate.
- Ticket 04 produced first Candidate `e1b111f` at `12:20:06Z`, about 44
  minutes after launch. The lead reviewed it quickly and sent `review-1.md` at
  `12:22:20Z` against that Candidate.
- The ticket 04 review asked for three changes: terminal delivery should not
  treat accepted keystrokes as worker acknowledgement, rework commands should
  use the new effort-record lookup from a main checkout, and the worker should
  re-integrate. The lead log explicitly says one finding was inferred from code
  reading rather than reproduced.
- Ticket 04's re-integration rebase failed at `12:23:02Z` with a conflict in
  `bench/harness/test_lead_driven.py`, then continued successfully at
  `12:23:24Z`. It was later dirty again while adding acknowledgement tests.
- Ticket 06's worker added red tests for launch-time and supervisor overlap
  surfaces. The first `test_lead_launch.py` run failed on missing overlap
  behavior; after production edits, the lead-launch suite passed while the
  worker suite was still running.
- Ticket 16's worker added red tests for moving bookkeeping to a record branch
  and for the ticket 07 real-repo lookup regression. Its early narrow runs
  failed for missing record-branch behavior, then production edits began adding
  local fallback behavior around `resolve_effort_record`.
- By the 12:33 tick, integration had not advanced after `253a5d6`; all visible
  movement was worker-side.
- Ticket 04's first Candidate was superseded by review rework almost
  immediately. It rebased to marker `56a6aa3`, then became dirty again while
  adding acknowledgement and effort-record lookup fixes.
- Ticket 06 showed the target overlap signal in real time: it was dirty in
  `worker.py`/`test_worker.py` at the same time as ticket 04, and dirty in
  `lead_launch.py` at the same time as ticket 16.
- Ticket 16 made effort/tracker/acceptance tests green by about `12:31Z`, but
  land/lead-launch checks were still failing by the `12:33` observation
  window.
- By the 12:43 tick, integration was still at `253a5d6` and the lead
  transcript had no new selected action after ticket 04 review-1.
- Ticket 04's review rework widened from `worker.py` acknowledgement handling
  into acceptance, lead-launch and lead-driven harness changes. It recovered
  worker, acceptance and lead-launch tests to green, then hit a focused
  rework-round harness failure and remained dirty with one shell active.
- Ticket 06 recovered its focused overlap scenario, `test_lead_launch.py`,
  `test_worker.py`, and full `bench/harness/test_lead_driven.py` to green by
  about `12:44Z`, but was still pre-commit and running the direct scenario.
- Ticket 16 fixed land and lead-launch tests, added a
  `bookkeeping-off-code-branch` readiness scenario and policy version bump,
  but its expanded lead-driven gate still failed before a two-test focused
  rerun passed.
- Three active tickets now overlap in the same shared files:
  `bench/harness/lead_driven.py`, `bench/harness/test_lead_driven.py` and
  `tools/agents/lead_launch.py`.
- By the 12:53 tick, integration still had not advanced after `253a5d6`, and
  the lead transcript remained unchanged after ticket 04 review-1.
- Ticket 04 had recovered all observed focused/tool/harness/scenario checks to
  green by about `12:52Z`, including `git diff --check` and `py_compile`, but
  stayed dirty while rewriting `04-report.md`.
- Ticket 06 had a passing direct `parallel-workers-and-overlap` scenario and
  full `bench/harness/test_*.py` loop by `12:47Z`, then moved into diff/anchor
  inspection and report planning without a commit.
- Ticket 16 recovered its touched tool gate, sync-policy check and tracker
  lint by `12:53Z`; broad harness and scenario validation was still active in
  the pane.
- Context percentage recovered upward for tickets 06 and 16 during this tick
  while their elapsed runtime continued increasing, reinforcing that context
  gauges need compaction history.
- Ticket 04 landed by `12:59:32Z` as exact Candidate `a7ba1a7`, after first
  failing from a bad relative integration path and then from the main checkout
  where the effort record was not found. The successful acceptance/land retry
  ran from the integration worktree.
- Ticket 04's durable acceptance record required `gate` and
  `main-checkout-reproduction`; its claim sampling saw 43 claims and 43
  unrecorded, continuing the pattern that acceptance is not yet independent
  verification.
- After ticket 04 landed, the lead recorded that only 4 landed tickets had
  acceptance-tool records and 0 were reviewed by a context other than the lead;
  the lead noted story 11 and the independent-verification gate remain unmet
  before mainline.
- Ticket 10 launched at `13:00:02Z` from integration tip `c92de62`; by the
  `13:08` monitor it was running with four dirty paths and one shell.
- Ticket 16 reached Candidate `54e10bc` at `13:01:14Z`. Review 1 found the
  record-branch move and real-repo lookup fix worked, but blocked on accepted
  worker reports being stranded on the code branch after `move-records`.
- Ticket 06 reached Candidate `4d08d2d` at `13:04:14Z`. Review 1 accepted the
  overlap surfaces but blocked on real runtime `file_change` event accounting
  and recorded that a real sandboxed Codex worker in a git worktree could not
  commit because the main repo `.git/worktrees/.../index.lock` was read-only.
- By the 13:23 pass, no new lead actions were selected after ticket 06
  review-1; integration stayed at `c92de62`.
- Ticket 06 started review rework, added real `file_change` accounting tests,
  recovered the focused review tests by `13:11`, and recovered full harness and
  readiness loops by about `13:23`, but remained dirty and was preparing a
  non-marker review-fix commit per the review protocol.
- Ticket 10 moved from startup to active implementation: worker, lead-launch
  and focused lead-driven tests failed first, then focused
  `watching-limits-accounting`, full `test_worker.py`, full lead-driven tests,
  and the direct scenario passed by `13:23:04Z`; the broad combined gate was
  still running.
- Ticket 16 continued rework after its post-review marker, with repeated red
  acceptance, tracker, lead-launch and scenario tests around accepted report
  copies on moved record branches. `test_acceptance.py` recovered by
  `13:23:53Z`, while tracker edits continued.
- By the 13:33 pass, integration still had no commits after `c92de62`; no new
  coordinator artifacts appeared after ticket 06 review-1.
- Ticket 06 created non-marker review-fix commit `581903c` at `13:31:44Z` and
  was clean and idle, but the monitor still reported generic `running` because
  the head was not a `ready for review` marker. This is the clearest live
  example of "fix committed, waiting for re-integration permission".
- Ticket 10 had no commits by `13:33` but had recovered broad worker,
  lead-launch, lead-driven and direct scenario checks after earlier red runs;
  the broad gate was still active.
- Ticket 16 remained dirty after green narrow checks and had edited canonical
  `docs/agents/issue-tracker.md` plus
  `docs/agents/agentic-workflow-repos.json`, keeping the ticket in a higher
  review-risk lane.
- By the 13:43 pass, integration still had no commits after `c92de62`, no new
  lead transcript actions were selected, and no new coordinator artifacts were
  written after the prior tick.
- Ticket 16 recovered its post-review work and reached new clean Candidate
  `f92a234` at `13:43:00Z` after commit `18c3f5f` imported accepted worker
  reports. The monitor showed it `ready`, but no lead review artifact existed
  by the artifact scan.
- Ticket 06 remained clean and idle at non-marker commit `581903c`; the monitor
  still showed generic `running`, making this waiting-for-reintegration state
  easy to miss.
- Ticket 10 had green worker, lead-launch, lead-driven and direct scenario
  checks by `13:42`, but then edited scenario/harness files again and restarted
  a focused gate after the monitor sample.
- Ticket 16 landed by `13:48Z`; the lead then recorded acceptance, added and
  claimed ticket 17, launched its worker, and told ticket 06 to re-integrate.
  The coordinator state directory for ticket 16 still lacked a per-ticket
  `landed.json` despite the branch/status evidence.
- Ticket 10 reached marker Candidate `2135428` at `13:51:45Z`, but direct
  branch comparison at `13:57Z` showed it already `7` behind and `2` ahead of
  current integration after ticket 16 and ticket 17 bookkeeping.
- Ticket 06's reintegration after ticket 16 caused real conflicts in shared
  launcher and harness files; the worker resolved them and got focused
  `test_lead_launch.py` plus `test_lead_driven.py` green, while the full
  `tools/agents/test_*.py` sweep was still running at the last sample.
- The lead recorded that ticket 10 built its original acceptance items but
  missed most of the lead amendment because the amendment was only prose; the
  lead drafted ticket 18 to carry that work.
- Ticket 17 started from integration `9ab6f26` at `13:48:52Z` and began the
  sandboxed-worker commit fix; by `13:59Z` it had four dirty files and a
  `test_worker.py` failure after earlier green output.
- During the 14:03 pass there were no new lead transcript, lead-log,
  notes or coordinator artifact entries; integration stayed at `9ab6f26`.
- Ticket 06 recovered from its reintegration conflicts enough to pass
  `test_lead_launch.py`, `test_lead_driven.py`, `test_worker.py` and then the
  full `tools/agents/test_*.py` loop by `14:04:02Z`, but it had not produced a
  final ready marker and the pane still showed full-gate work being monitored.
- Ticket 10 stayed clean at ready Candidate `2135428`, still stale behind
  current integration and with no durable artifact indicating it is being held
  behind ticket 06.
- Ticket 17 recovered `test_worker.py`, then `test_lead_launch.py`, then one
  `bench/harness/test_lead_driven.py` run, but a later harness run failed at
  `14:04:56Z`; it remained dirty in five files.
- During the 14:13 pass, no lead transcript, lead-log, notes, coordinator
  artifact or integration-branch changes appeared after the previous tick.
- Ticket 06 kept validating the rebased result: full `tools/agents/test_*.py`
  and `bench/harness/test_*.py` sweeps reached exit code 0, then the worker
  reread the reintegration/verification context and started another tools
  sweep; it remained clean at non-marker head `4bd776c`.
- Ticket 10 remained idle at ready Candidate `2135428`, still a stale hand-off
  behind current integration and without a durable wait-state artifact.
- Ticket 17 recovered `test_worker.py`, `test_lead_launch.py`,
  `test_land.py` and `bench/harness/test_lead_driven.py`, then expanded into
  `docs/practice/lead-supervisor-worker.md` and
  `.claude/skills/run-worker/SKILL.md`; it remained dirty in seven files.
- Ticket 06 and ticket 17 context gauges jumped sharply upward during active
  work, consistent with compaction/resume rather than actual task simplification.
- Ticket 06 completed reintegration and produced Candidate `3b9baeb` at
  `14:22:43Z`, after rerunning tool-agent suites, harness suites, all readiness
  scenarios, tracker lint and diff-check. Its reintegration phase took about
  34 minutes from the lead's instruction.
- Ticket 17 produced Candidate `25d8d0d` at `14:24:14Z`, about 36 minutes
  after launch, after recovering focused tests, harness tests, all readiness
  scenarios, tracker lint and diff-check.
- The 14:23 monitor missed ticket 17's near-immediate transition: it still
  showed running/dirty at `14:23:45Z`, while direct branch status at
  `14:24:19Z` showed a clean ready marker.
- The lead woke at `14:24Z` for ticket 06 and began a review/check command, but
  no completed review or landing artifact was visible by the pass cursor.
- Ticket 10 continued to wait idle at Candidate `2135428`, stale behind the
  integration branch for another tick.
- Ticket 06 landed by `14:27Z` after the lead reviewed and accepted Candidate
  `3b9baeb`; integration advanced through `bc36daf` to `f57e715`, and ticket
  18 was added for the missed interactive-launch/monitor work.
- Ticket 10 received a content-accepted re-integration handoff at `14:27:47Z`;
  its rebase onto the new integration tip hit shared launcher/test conflicts,
  and by the 14:33 pass it was running with 16 dirty paths at integration head.
- Ticket 17 received review 1 at `14:30:22Z`. The lead's real probes found the
  private-clone approach worked for TraeCLI but not for Codex, where `.git`
  remained read-only under the tested sandbox layout.
- The lead recorded another instruction-precedence failure: ticket 17 allowed
  real runtime probes, but the generic worker brief forbade model runs, so the
  worker needed review rework to run the probes.
- Ticket 10 completed its re-integration round and produced final marker
  `eebb102` at `14:45:21Z`, about 17.5 minutes after the lead handoff. It
  recovered the rebase, reran tool-agent, harness, scenario and tracker checks,
  refreshed the report, and was clean/ready by the `14:46` monitor.
- Ticket 17's review rework got both real runtime probes to pass by `14:42Z`
  and saved public event streams with 19 Codex lines and 16 TraeCLI lines. Its
  worker, tool-agent, harness, readiness scenario, tracker and size checks were
  green by `14:46`, but it was still dirty while updating the report.
- Direct branch comparison showed ticket 17 was `6` behind and `1` ahead of
  integration, while the monitor showed only `1` ahead and `6` dirty. The
  missing behind signal hides an eventual rebase cost.
- Ticket 10 landed by `14:52Z` through `e9b32fa` and `25a03d0`, about
  113 minutes after launch and about 17.5 minutes after its re-integration
  handoff began.
- Ticket 17 reached a clean post-review marker around `14:49Z`, but the lead
  landed ticket 10 first and sent ticket 17 a re-integration request at
  `14:52:19Z`. The worker rebased successfully, then wrote past the marker to
  adapt ticket 06/10 accounting for private worker repositories.
- Ticket 17's accepted rework was the first real end-to-end success of the
  product path for both runtimes: Codex and TraeCLI workers each committed
  under the default sandbox, were judged ready and had Candidates imported
  without touching other branches.
- During the 15:03 pass, ticket 17 remained active after re-integration: it was
  `0` behind/`2` ahead of integration at marker `3d25593`, but dirty in seven
  tracked files plus untracked `.local`.
- Ticket 17's first post-reintegration full tools loop failed, then a rerun
  with workspace-local `TMPDIR`/`TMUX_TMPDIR` isolated a tmux-specific suite
  that passed 35 tests at `15:03:43Z`; the worker then started a full gate
  runner that was still running at the transcript cutoff.
- Ticket 17's context gauge jumped from 22% to 85% between adjacent monitor
  samples while work continued, another compaction/resume visibility issue.
- By the 15:13 pass, ticket 17 was still `0` behind/`2` ahead at marker
  `3d25593` but had written past it in seven tracked files plus generated
  `.local`; the full reintegration gate was still running.
- Ticket 17 diagnosed the failing full gate to tmux private-temp setup: a
  bwrap-mounted `/tmp` needed normal `1777` permissions and a pre-created
  `/tmp/tmux-$UID` directory before `test_sqlite_baseline.py` would pass.
- During the 15:23 pass, ticket 17's private-`/tmp` full gate reached a real
  behavior failure: lead-imported peer worker branches were counted as protected
  shared-branch movement. The worker added a focused regression, fixed it, and
  restarted the full private-`/tmp` gate; it was still running at the cutoff.
- By the 15:33 pass, ticket 17 had cleared the tools suite again but the
  `parallel-workers-and-overlap` scenario still failed. The worker fixed a
  focused private-clone resume-refresh regression, then hit a separate
  `details=landing` failure because the successor Candidate was not yet
  published before landing.
- By the 15:43 pass, ticket 17 had a context compaction at `15:35:52Z`, fixed a
  focused stale-base same-session rework test, made the focused
  `parallel-workers-and-overlap` scenario pass, and got the full tools and
  bench harness unit sweeps to pass under private `/tmp`; readiness scenarios
  had just started and no new marker existed yet.
- Ticket 17 produced final clean marker `7fddf82` at `15:49:48Z`, about
  57.5 minutes after its re-integration handoff. It reran tools, harness,
  readiness scenarios, tracker lint and `git diff --check`, then handed off a
  clean tree.
- The lead began checking ticket 17 around `15:50:53Z`, but a lead-side
  gate/land attempt hit `/tmp` inode exhaustion and then
  `test_sqlite_baseline.py` failure for candidate `7fddf82`; integration did
  not move and an untracked `acceptance-17.jsonl` remained in the integration
  worktree by the cutoff.
- Ticket 17 landed by `15:58Z`: integration advanced through worker marker
  `7fddf82`, `006bf0f` resolving ticket 17, `9336982` recording ticket 17
  acceptance/log evidence, and `0a28a3d` claiming ticket 18. The coordinator
  status reported ticket 17 landed, but the ticket 17 cache directory still had
  no `landed.json` by the `16:05` cutoff.
- Ticket 18 launched at `15:58:18Z` from base `0a28a3d` in
  `/data02/home/philip.yang/workspace/corenius-lwr-18-interactive-launch-and-monitors`.
  The lead sent a temp-directory note at `15:59:04Z`; the worker used
  `/data02/home/philip.yang/devx/tmp/lwr-18` for `TMPDIR`, `TMUX_TMPDIR` and
  `TMPPREFIX`, then began test-first work and had two dirty test files by
  `16:04`.
- The lead started an independent read-only review of the integration branch
  after ticket 17 landed. That actor is not visible in the coordinator worker
  table.
- The independent review completed by `16:09Z` and reported 23 defects, four
  blocking, while every suite and readiness scenario still passed. The lead
  converted the findings into tickets 19, 20 and 21 and made ticket 14 wait on
  them.
- Integration advanced to `375c261` by `16:11Z` through commits recording the
  independent review and claiming tickets 19 and 20. Tickets 19 and 20 launched
  from that base; ticket 18 stayed running from base `0a28a3d`.
- Ticket 18 moved into implementation for interactive launch/monitor behavior,
  adding `lead_launch.py` monitor/open-session surfaces and cycling through
  focused worker and lead-launch test failures/passes. It had no commits ahead
  and four dirty files by the 16:19 cutoff.
- Tickets 19 and 20 began from independent-review findings. Ticket 19 started
  red tests for shared-repository safety and dirty `test_land.py` plus
  `test_worker.py`; ticket 20 started red tests around rework/resume and dirty
  `test_lead_launch.py` plus `test_worker.py`.
- Ticket 17's cache directory still lacked `landed.json` after another
  integration move, even though the monitor reported it landed and the durable
  branch contains acceptance evidence.
- During the 16:23 pass, integration and lead state stayed unchanged while
  tickets 18, 19 and 20 all continued active worker-side work.
- Ticket 18 had focused worker and lead-launch tests green by about `16:21Z`
  but then moved into interactive rework-delivery and readiness-scenario work,
  staying dirty in four agent-tooling files with no Candidate.
- Ticket 19 broadened to eight tracked dirty files plus a new
  `bench/readiness/shared-repository-safe/` scenario; its `test_worker.py`
  run was still active in the pane and later exited `1`.
- Ticket 20 had production edits in `acceptance.py` and `worker.py` plus test
  edits; focused resume/lead-launch checks alternated pass and fail, and the
  monitor showed only 24% context left.
- The active path collision is concrete: tickets 18, 19 and 20 all edit
  `tools/agents/test_worker.py` and `tools/agents/test_lead_launch.py`
  concurrently, but the coordinator shows only dirty counts.
- During the 16:33 pass, the lead and integration branch remained idle while
  tickets 18, 19 and 20 continued active implementation against the same base.
- Ticket 18 recovered focused rework-delivery and shared-brief tests by
  `16:34Z`, but stayed dirty in four files and had no Candidate.
- Ticket 19's narrow land/worker-related test work remained red: `test_land.py`
  exited `1`, a combined acceptance/tool command exited `1`, and the worker
  continued inspecting shared-branch and record-source code.
- Ticket 20 added `lead_launch.py` to its dirty set, recovered several focused
  worker/lead-launch/acceptance checks, but still had a broader worker-suite
  failure before further fixes.
- Ticket 20's context gauge jumped from 24% to 80% while active; tickets 18 and
  19 dropped from high 90s to mid-40s during active implementation.
- During the 16:43 pass, the lead, scratch notes and integration branch stayed
  idle. Ticket 18 added an untracked interactive-launch readiness scenario and
  still had a focused scenario failure by `16:48Z`; ticket 19 recovered compile
  checks but `test_land.py` still failed; ticket 20 recovered broad harness and
  all-scenario checks but had not committed.
- Ticket 19's monitor context jumped from the earlier 19% sample to 93% while
  it remained dirty and active, repeating the compaction/resume visibility gap.
- During the 16:53 pass, the lead and integration branch still did not move.
  Ticket 18 had direct interactive scenario probes passing but the focused
  harness unit test still failed repeatedly. Ticket 19 had a failed combined
  tool suite and a narrower suite still running. Ticket 20 had compile,
  focused harness, all-scenario and tracker checks green, plus six untracked
  real-runtime evidence files, but still no Candidate commit.
- During the 17:03 pass, ticket 20 completed a short needs-info loop:
  `4e67309` (`ticket 20: needs-info`) at `16:57:33Z`, a lead answer sent by
  `ws.py tell 20` around `16:58:42Z`, then ready marker `7538209` at
  `16:59:36Z`. The integration branch had not moved by the `17:03` monitor.
- Ticket 18 was still running with 18% context left, eight tracked dirty files
  plus a new scenario directory, and edits to `.claude/skills` and
  `docs/practice`. Ticket 19 was still dirty in the shared agent-tooling area
  with a broad gate running.
- During the 17:13 pass, ticket 20 was reviewed, re-marked at `62b50ec`, landed
  through resolution `e77022d` and acceptance/log commit `c25920c`, and then
  ticket 21 launched at `17:14:23Z` from `660b3b7`.
- After ticket 20 landed, active tickets 18 and 19 were respectively `8` and
  `6` commits behind integration while the monitor still showed only `0 ahead`
  and dirty counts.
- The lead closed landed-worker tmux windows during ticket 20 cleanup; the
  worker monitor then showed old landed tickets with `pid -` and
  `context: unknown`, improving live clutter but losing last-seen context
  visibility.
- During the 17:23 pass, ticket 18 became ready at `901a770` at `17:22:52Z`
  after green checks and a clean tree. Direct branch comparison still showed it
  `8` behind integration while the monitor showed only `1` ahead and clean.
- The lead began ticket 18 review by reading its report/diff and launching a
  throwaway real interactive-worker probe at `17:25Z`; the probe wait had no
  result by the cursor boundary.
- Ticket 19's broad gate exited `0` by `17:26:47Z`, followed by a successful
  `py_compile`, but the worker continued inspecting shared-repository/base
  behavior and remained dirty in fourteen paths without a ready marker.
- Ticket 21 was in early red/green work: focused lead-driven scenario tests
  failed at `17:20:47Z`, and the direct snapshot showed two dirty harness files
  with no commit.
- During the 17:33 pass, ticket 18 stayed ready at `901a770`, still `8`
  behind integration by direct branch comparison and with no review artifact in
  its state directory.
- The lead's ticket 18 live review probe waited about seven minutes on a
  project trust/config prompt, then the lead sent input to the throwaway probe
  tmux session and started another wait with no result by the cursor boundary.
- Ticket 19's broad gate was no longer simply green: a later chained gate
  failed at `17:33:52Z` in
  `test_failed_batch_launch_releases_tickets_that_did_not_launch_with_bad_tmpdir`,
  after which the worker read debugging guidance and inspected the failing
  lead-launch paths.
- Ticket 21 expanded its dirty set to include `bench/harness/lead_driven.py`
  alongside `results.py` and `test_lead_driven.py`; compile checks over those
  files passed, but there was still no commit.
- During the 17:43 pass, ticket 18 moved from ready to rework. The lead sent
  review at `17:39:24Z` for candidate `901a770` after preserving probe pane and
  run-record artifacts from a real interactive review.
- Ticket 18's worker resumed in the same session, read the review and relevant
  skills/source paths, and stayed clean at `901a770` while investigating.
- Ticket 19 completed a broad chained gate with exit `0` by `17:45:09Z`, but
  the output included repeated `ResourceWarning: subprocess ... is still
  running` lines and the branch remained dirty without a candidate.
- Ticket 21's dirty set expanded to fifteen files across harness, readiness
  scenarios, acceptance and lander code while compile checks continued to pass.
- During the 18:03 pass, ticket 19 committed implementation `914e038` and ready
  marker `357a8c7`, the lead sent review at `18:01:18Z`, and the worker
  rebased successfully to current commits `30dd5b4` and `572128d` before
  starting review-follow-up tests. The review artifact still names the
  pre-rebase candidate `357a8c7`.
- Ticket 18 recovered full worker and lead-launch tests after several focused
  failures, but the full lead-driven harness failed at `18:04:45Z`; the
  focused interactive-launch scenario passed at `18:05:22Z` and another full
  harness run was started.
- Ticket 21 recovered focused effort, lead-launch and worker tests, then the
  full lead-driven harness failed at `18:04:17Z`; the dirty set reached
  twenty-seven paths and included staged executable check fragments.
- An observer-side attempt to spool pane captures under `/tmp` failed with
  `No space left on device`; this reinforces that temp-space policy affects
  monitoring as well as lead/worker validation.
- During the 18:13 pass, ticket 18 completed broad harness, py_compile,
  auxiliary harness, lead-launch and readiness scenario checks successfully and
  moved into report/final-marker work, while still dirty after its earlier
  ready marker.
- Ticket 19 reran the same chained land/acceptance/harness gate three times
  with exit `1` before it passed at `18:14:04Z`, then started a broader tools
  gate without committing a replacement marker.
- Ticket 21 continued scenario-failure repair, with focused mutation,
  bookkeeping direct-run and bookkeeping unit checks failing before later
  inspection/compile work; it still had twenty-seven dirty paths and no commit.
- No selected lead transcript records appeared after the ticket 19 review send
  recorded in the previous pass; integration stayed at `660b3b7`.
- During the 18:23 pass, ticket 18 was still at ready marker `901a770` but
  dirty in twelve paths; a py_compile/check command failed at `18:23:23Z` and
  the worker restarted focused interactive scenario reproduction after context
  compaction.
- Ticket 19 remained at rebased ready marker `572128d` but dirty in eight
  tracked paths; a full tools gate failed at `18:19:50Z`, then a targeted
  worker/land/acceptance/harness rerun had worker, land and acceptance passing
  while the harness suite was still running at the pane snapshot.
- Ticket 21's combined tool unittest failed at `18:18:29Z`, then a rerun
  completed `OK` with ResourceWarning noise before the worker started the full
  lead-driven harness at `18:24:15Z`.
- There were no selected lead transcript records and no new coordinator
  review/landing artifacts during the 18:23 pass.
- During the 18:33 pass, integration remained at `660b3b7`; tickets 18, 19 and
  21 were still running, and there were no selected lead transcript records or
  new coordinator review/landing artifacts.
- Ticket 18 fixed another interactive scenario timing issue after focused
  failures: it patched the scenario participant wait, then the fake-worker cap,
  and the focused interactive lead-driven test passed at `18:34:37Z`, but the
  branch stayed dirty at stale ready marker `901a770`.
- Ticket 19 recovered from the prior full-tools failure: targeted
  worker/land/acceptance/harness, full tools, bench harness and readiness
  scenario gates all completed successfully by the `18:37` transcript cutoff,
  but no replacement marker had been committed and the session compacted at
  18% context.
- Ticket 21 passed standalone readiness and tracker gates, patched
  `checks.d/30-tracker.sh`, and had a direct 30-tracker gate pass before
  starting a full scenario run; direct git showed staged additions plus
  unstaged edits, which the monitor reduced to a single `dirty 27` count.
- During the 18:43 pass, ticket 19 became clean and ready at replacement
  marker `bdda651` on top of implementation commit `8ff9423`; the lead saw it
  ready and started a ticket 19 acceptance/review command at `18:42:56Z`, but
  no result was in the lead transcript by byte `5119867`.
- Ticket 18 reported broad gates green after review-code changes, including
  all harness tests and all seventeen readiness scenarios, but it stayed
  running with twelve dirty paths and no replacement marker.
- Ticket 21 context compacted at `18:41:38Z`; monitor context later showed
  77%, while direct git showed twenty-eight dirty paths including an untracked
  report and staged check-script/scenario-participant changes.
- The worker monitor showed ticket 19 ready and idle, but it did not expose the
  lead-side review/acceptance command that had already begun.
- During the 18:53 pass, ticket 19 completed review/accept/land: integration
  advanced from `660b3b7` to `a6d72b6`, with ticket 19 commits plus resolve,
  acceptance/log and cleanup commits.
- Immediately after ticket 19 landed, the lead changed `ws.py` so effort
  ticket/evidence bookkeeping reads and writes from a record worktree while
  code still lands on the integration branch; the lead notes call this the end
  of the morning's code-branch bookkeeping workarounds.
- Ticket 21 reached first ready marker `8e754d2` after implementation
  `f29a2b7`, then the lead reviewed it, sent `reintegrate-1.md`, and the
  worker returned to dirty rework with three harness files changed.
- Ticket 18 reached replacement ready marker `f127c06`; the lead's real
  interactive probe showed prompt detection and ready detection worked, but the
  real runtime ignored the trust override that the worker had only tested as a
  command-line string, so review-2 was sent and the ticket returned to rework.
- During this pass, ticket 21's dirty count changed between samples: an earlier
  `ws.py status` sample showed `2` dirty, while later `ws.py status`,
  `ws.py workers` and direct git showed `3`.
- During the 19:03 pass, integration remained at `a6d72b6`, ticket 19 stayed
  landed, and no selected lead tool calls occurred after the previous ticket
  18 review-2 send.
- Ticket 18 remained at replacement marker `f127c06` but accumulated seven
  dirty files while working review-2; it hit a `test_worker.py` exit `2` at
  `19:02:28Z`, recovered focused worker tests, and completed a lead-driven
  scenario by `19:03:56Z` without committing a new marker.
- Ticket 21 remained at marker `8e754d2` with three dirty harness files after
  review-1/reintegration; a lander-related check failed at `18:58:17Z`, then
  focused checks continued, a full `bench.harness.test_lead_driven` unittest
  was still running at the pane sample, and it completed successfully at
  `19:05:59Z`.
- Both active workers were near low context while dirty: ticket 18 showed 30%
  context and ticket 21 showed 31% context in the 19:04 worker monitor.
- During the 19:13 pass, integration remained at `a6d72b6`, ticket 19 stayed
  landed, and no new lead transcript bytes appeared after the prior review-2
  handoff.
- Ticket 18 continued review-2 rework at stale marker `f127c06`: a real
  interactive launch proof and focused worker/lead-launch/trust tests passed,
  then the worker context compacted and the full lead-driven harness was still
  running with seven dirty files at the cutoff.
- Ticket 21 completed a broad tools unittest and a full
  `bench.harness.test_lead_driven` run successfully, then restarted the broad
  tools unittest while remaining dirty at marker `8e754d2`.
- Ticket 18's monitor context jumped from 18% to 91% during the same observer
  pass because of compaction/resume, while the monitor row did not preserve
  that transition as a first-class event.
- During the 19:23 pass, the lead answered a public `status` prompt after
  checking `ATTENTION.md`, `ws.py status`, `ws.py workers` and direct branch
  ancestry. The reply matched the record: seventeen tickets landed, ticket 21
  current with integration, ticket 18 behind the tip, and tickets 14/15 still
  blocked.
- Ticket 18 passed the full lead-driven harness and several smaller harness
  suites after review-2, then started a full `tools/agents/test_*.py` sweep
  that was still running in `test_worker.py`; it remained dirty at stale marker
  `f127c06`.
- Ticket 21 completed broad tools and broad harness gates, compacted context at
  `19:23:45Z`, then restarted by rereading guidance and began a 17-scenario
  readiness loop; it remained dirty at marker `8e754d2`.
- Both active workers produced green commands with repeated
  `ResourceWarning` output, which the monitor still collapsed into a generic
  running or passed state.
