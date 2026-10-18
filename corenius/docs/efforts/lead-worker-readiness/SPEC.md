# Lead and worker readiness

Status: draft 2, rewritten after an independent critique and the owner's three-instantiations direction.
All open decisions answered by the owner (`decisions/`). Ticket breakdown proposed, awaiting approval.
Triage: ready-for-agent
Source read: Corenius `main` at `d7f6748`; the uncommitted drafts of `controller-backed-bench` (its
`SPEC.md` and `EFFORT.md`) and `semantic-polyglot-layout` in their Codex worktrees, read 2026-10-05 07:40 to
08:10 UTC and liable to change; the case study `docs/practice/imported/sureal-semantic-layout.md`
including section 8. "By design" marks a statement read from another effort's spec, not observed in
running code.

## Problem Statement

The owner wants to hand Corenius a multi-ticket effort and have a lead run it with parallel workers,
without watching every step and without the outcome depending on luck. Today that cannot be claimed for
any of the ways Corenius could do it, and there is no common measure to compare them.

A sibling repository ran such an effort with an improvised coordinator. The case study records what went
wrong and what caught it:

- A worker declared a ticket done before committing, and the lead tried to commit the worker's work itself
  (§5, §8).
- Verification ran against an uncommitted tree; the merge followed with no re-check (§5).
- A ticket with every acceptance item ticked and all tests passing violated its spec in four ways. A
  review against the spec and decision records caught it; no command gate would have (§8).
- The rework was done by the same worker, with its context, after the review reached it. The merge came
  about a minute after the second "done", and no second review is on record (§8).
- A recorded failing check was merged, because merging had no gate (§5).
- Shared state outside the repository was fingerprinted by hand, once, after the worker had started (§5, §8).
- Workers ran with full access, no time limit, no stall detection and no durable record of actions (§5).
- Two parallel tickets edited files a third had just delivered, with nothing watching for overlap (§8).
- Finished worker processes stayed alive, tickets meant for a human could be launched, a blocker line in
  an unexpected format would be read as no blocker, and findings later tickets needed had nowhere to go
  (§2, §5, §8).

## Solution

One yardstick and three instantiations.

The yardstick is a fixed set of readiness scenarios, each stated as behavior an outside observer can check.
Each instantiation is run against the same scenarios and declares, per scenario, one of: passes, passes
with intervention (a supervising agent stepped in; what it did is recorded), not applicable by design (with
the reason), or fails. The same measures are recorded for each: owner
interventions, elapsed time, model usage where the runtime exposes it, and setup cost.

| | Instantiation | Coordination | Ticket source | Worker result | Lander | State today |
|---|---|---|---|---|---|---|
| I1 | Lead-driven | An interactive lead agent plus the agent toolbox | Tracked tickets in the effort directory, produced by the ticket-splitting skill | A branch in a worktree | The toolbox's gated lander | Toolbox exists; worked example in the case study; never run on a multi-ticket effort in Corenius |
| I2 | Local controller | A controller with SQLite as execution store | Kata work items, admitted at an exact revision (by design) | A Candidate from a private repository (by design) | The controller's serialized landing (by design) | In implementation; one ticket unaccepted; live qualification pending |
| I3 | Workflow controller | The richer controller with Temporal workflow execution | As I2 | As I2 | As I2 | Parked roadmap; preserved, unfinished |

The owner named jj as the worker version control of I2. The controller spec at the commit read makes Git
the required first backend and qualifies jj separately; I2 is measured with whichever backend that effort
has qualified at the time, and the record says which.

The instantiations differ in how much they automate. I1 comes first and is agent-heavy: supervising agents
watch it, are cautious and step in, and the notes on what that took are an output of this effort. I2 and I3
come later and exist to remove those interventions, so that most work runs without an active agent fixing
things. The count and kind of interventions is therefore the main measure that separates the three.

From the owner's side a ready instantiation behaves like this: the lead launches every ticket whose
blockers have landed and that is meant for an agent; each worker hands over one exact result; the lead
reviews that exact result against the ticket and the spec, records the decision, and accepts it or returns
findings for a bounded number of rounds; accepted results land one at a time through a gate; anything a
worker was not allowed to touch is shown unchanged or the change is reported; and the lead is told when a
worker stalls, overruns or outlives its ticket.

## User Stories

Tags: **D** can be judged deterministically with a scripted participant; **L** needs a live model worker.
Observation surface: **R** the disposable repository (commits, refs, files); **P** the run's process and
filesystem observer; **Q** the instantiation's recorded decisions and events (for I1 the tracked effort
files and run records; for I2 and I3 the controller's query interface).

Hand-off

1. As a lead, I want a worker's result to be one immutable Candidate (content plus the base and brief it was made against), so that I always know what I am reviewing. [D, R+Q]
2. As a lead, I want the "ready for review" event to be impossible before that Candidate exists, so that I am never woken on an unfinished tree. [D, R+Q]
3. As a lead, I want my own write into a worker's workspace refused or reported as a violation, so that I cannot pre-empt a worker that is still writing. [D, R+P]
4. As an owner, I want a worker to be unable to move any shared branch, so that worker freedom does not put shared history at risk. [D, R]

Verification and acceptance

5. As a lead, I want verification bound to the exact Candidate, so that what was checked is what lands. [D, R+Q]
6. As an owner, I want acceptance recorded as its own decision, naming who decided and the Candidate, so that passing tests are never mistaken for acceptance. [D, Q]
7. As an owner, I want landing refused when no review record on both axes (repository standards, ticket and spec) exists for the exact Candidate, so that review cannot be skipped. [D, Q]
8. As an owner, I want a Candidate that passes its tests but breaks its spec to be rejected at review, so that self-consistent but wrong work does not land. [L, Q]
9. As an owner, I want landing refused when a required check is recorded as failing, unless my waiver is on record, so that known failures are not merged by default. [D, Q]
10. As a lead, I want to sample a worker's cited claims and record each outcome, so that my verification effort is visible. [D, Q]
11. As an owner, I want the review of at least one ticket per effort done by a context that is not the lead, so that the lead's reading is not the only check. [D, Q]

Rework

12. As a lead, I want review findings delivered to the worker as a durable artifact with a receipt, so that delivery does not depend on typing into a terminal. [D, Q]
13. As a lead, I want the rework done by the worker that produced the Candidate when it is still available, so that the rework uses the context it already has. [L, Q]
14. As a lead, I want each rework to produce a new Candidate while the earlier one and its review stay on record, so that a ticket's history can be read later. [D, R+Q]
15. As an owner, I want a reworked Candidate reviewed again before landing, so that "fixed" is checked and not assumed. [D, Q]
16. As an owner, I want review-driven rework capped at three rounds per ticket, counted separately from re-integration after a moved base, after which the ticket returns to me, so that a ticket that does not converge stops consuming budget. [D, Q]
17. As a lead, I want a worker's change to a ticket's status or acceptance items to have no effect on whether it is accepted, so that a worker cannot revise what "done" meant. [D, R+Q]

Scheduling

18. As a lead, I want the launchable set computed from blocking edges, so that I never launch a ticket whose blockers have not landed. [D, Q]
19. As a lead, I want a blocker entry that cannot be parsed to stop the launch with an error, so that a malformed edge is never read as "no blocker". [D, Q]
20. As an owner, I want a ticket not triaged for an agent refused at launch, so that decisions and credentialed steps stay with me. [D, Q]
21. As a lead, I want a new worker's workspace to start from the tip that contains its blockers' landed work, so that it builds on what it depends on. [D, R]
22. As a worker, I want my brief to be an exact recorded revision of the ticket, the spec and the standing instructions, so that a change to the instructions mid-effort is visible and not silent. [D, Q]
23. As an owner, I want a stated cap on parallel workers, so that load and spend are bounded. [D, P+Q]
24. As a lead, I want a worker's question to put its ticket in the `needs-info` triage state with the question recorded, so that a blocked worker does not guess. [D, Q]

Parallel work and landing

25. As a lead, I want two workers to run at the same time and both submit, so that independent tickets do not wait on each other. [D, P+Q]
26. As a lead, I want to be told when a running worker changes a path outside its ticket's declared paths, or a path another running ticket has declared, so that overlap is seen before review. [D, R+Q]
27. As an owner, I want the gate to run a real test command on the exact Candidate that lands, so that the landed commit is the verified commit. [D, R+Q]
28. As a lead, I want a Candidate whose base has moved to require a new Candidate on the new base with its own verification and acceptance, so that nothing lands on a check made against an older base. [D, R+Q]
29. As an owner, I want an effort able to land on an integration branch that only I move to the mainline, so that an effort accumulates before I take it. [D, R]
30. As an owner, I want shared branches never rebased, force-moved or deleted by any participant, so that other agents' work is safe. [D, R]

Boundaries

31. As an owner, I want workers to have network access by default, with the launch record stating whether it was on, so that build and fetch steps work and the choice is visible. [D, P+Q]
32. As a lead, I want to turn the network off for one launch, so that a task that must be network-isolated can be. [D, P]
33. As a lead, I want to declare paths outside the repository that a worker may add to, so that a legitimate build can write a new versioned output. [D, P]
34. As an owner, I want declared protected paths outside the repository recorded before the worker starts, at the depth of every entry beneath them (kind, size, modification time, content digest for regular files up to a stated size), so that the baseline cannot be late, shallow or forgotten. [D, P]
35. As a lead, I want a report of each protected entry modified or removed during the run and each addition outside the declared places, stating that it changed during the run without naming who changed it, so that damage to shared state is seen at review. [D, P]
36. As a lead, I want a single worker command allowed to run for as long as the launch declares, so that a build that takes longer than two minutes is not cut off. [D, P]

Observation and limits

37. As an owner, I want each running worker visible in its own terminal window, so that I can see what the fleet is doing without asking. [L, P]
38. As a lead, I want a durable record of each worker's commands and messages that survives the terminal closing, so that I can investigate afterwards. [D, P+Q]
39. As a lead, I want a notice when a worker passes its declared wall-clock limit or its declared quiet time, so that I can act before the run is lost. [D, P+Q]
40. As an owner, I want every worker process shown as stopped, or explicitly retained for rework and counted against the parallel cap, when its ticket lands or is abandoned, so that finished workers do not linger unaccounted. [D, P+Q]
41. As an owner, I want the disk used by each worker's workspace and caches reported at the end of its run, so that growth is seen. [D, P]

Continuity

42. As a worker, I want to leave a finding addressed to a named later ticket, so that what I learn is not lost in prose. [D, Q]
43. As a lead, I want findings addressed to a ticket included in that ticket's brief at launch, so that the later worker starts with them. [D, Q]

Proof

44. As an owner, I want each D story proven with scripted participants and real git, process and file effects, so that the mechanics are shown without model spend. [D]
45. As an owner, I want one live trial per instantiation that is run, reported separately from the deterministic results, with every intervention by a supervising agent or by me recorded against the scenario it rescued, so that the claim is honest. [L]
47. As an owner, I want a running log of what it took to get the lead-driven instantiation working well (each obstacle, each intervention, each rule added), so that the controllers are built to remove exactly those. [L]
46. As an owner, I want the three instantiations' results in one table per scenario with the common measures, so that I can compare them. [D]

## Implementation Decisions

Confirmed by the owner (his words are quoted in `decisions/`)

- The benchmark harness is the test seam.
- There are three instantiations, as in the table above, judged by the same scenarios.
- Codex workers first, then traecli workers.
- Network is on by default.
- The focus is Corenius itself; the case study is background.
- I1 is built and run first and is expected to pass a good share of the scenarios, not all. It is
  agent-heavy, and interventions by supervising agents are expected and recorded. Notes on what it takes
  are kept. I2 and I3 follow and aim to automate most work without an active agent.

Following from repository policy already in force (not new decisions)

- Fast-forward only; no participant rebases, force-moves or deletes a shared branch.
- Review covers both axes, repository standards and ticket and spec, and an independent review is a
  separate gate.
- The repair budget is three rounds; extending it is the owner's decision.
- Tickets and the spec are tracked in the effort directory; lifecycle status and triage role are separate
  fields.
- Only a human merges to the mainline. Under this spec a lead may land accepted Candidates on an effort's
  integration branch (confirmed below); the move to the mainline is his.

Confirmed by the owner on 2026-10-05 (`decisions/20261005-eight-answers.md`)

1. **Worker commits.** A worker commits in its own workspace and hands over an exact Candidate. This
   supersedes, for this effort, the 2026-09-29 rule that the lead commits. A worker still cannot move a
   shared branch, merge or push.
2. **The lead never writes in a worker's workspace**, in any instantiation.
3. **The core set** that I1 must pass, with or without intervention, before its live trial counts:
   stories 1, 2, 5 to 7, 9, 12, 14 to 18, 20, 27, 34, 35.
4. **Park exemption.** Lead-driven work under this effort (toolbox changes and a lead-driven mode in the
   harness) is exempt from the controller spec's instruction to park unrelated work.
5. **Network default.** On by default for I1. For I2 and I3 the scenario records what their design grants,
   and stories 31 to 33 may be "not applicable by design" there.
6. **Container builds and shared-cache writes** are off by default and declared per launch.
7. **Integration branch.** A lead may land accepted Candidates on an effort's integration branch without
   the owner; moving the mainline is the owner's. When the mainline moves while an integration branch
   accumulates, the integration branch is re-created on the new mainline and every Candidate re-verified
   (lead's elaboration; fast-forward only leaves no other path).
8. **Live trial for I1.** A real small Corenius effort of at least three tickets with one blocking edge,
   two parallel workers and one review-driven rework, landed on an integration branch.

Not yet authorized: commits or landing of this effort's record, worker launches, model spend.

What each instantiation needs, as read today

| Story group | I1 lead-driven (toolbox) | I2 local controller | I3 |
|---|---|---|---|
| Hand-off | Missing: worker commits are a violation; the lander rebases the branch in the worker's worktree, so the landed commit is not the reviewed one | Exact commit, tree, base and report retained at submission (by design). Refusing the lead's write is not stated | As I2, parked |
| Verification, acceptance | Gate command exists; claim sampling exists; no acceptance record, no review precondition, no waiver | Acceptance is a durable decision separate from tests (by design). Review as a precondition and the owner waiver are not stated | |
| Rework | Missing: the worker process exits and nothing delivers findings | Request repair and a repair-required state (by design). Same-worker delivery not stated; traecli excluded from its trial | |
| Scheduling | Blocking edges and frontier ordering exist; triage is not read at all; no brief pinning | Kata owns items and dependencies; sources admitted at exact revisions; two worker slots (by design). Dependent tickets are a later milestone of `controller-backed-bench`, which refuses dependency graphs in its first | |
| Parallel, landing | Serialized gated landing exists; an integration branch can be named as target; overlap between running tickets is not watched | Serialized landing; a moved base needs a new integration Attempt (by design); proof pending | |
| Boundaries | Missing: the launch contract forbids network, limits each command to two minutes, protects git repositories only | Excluded by design: no shared mutable stores, provider connections only | |
| Observation, limits | Event stream, wall-clock cap, quiet-time finding and a live terminal view for the benchmark exist; nothing accounts for a retained worker or disk | Process disposition, workspace inventory and bounded disk are stated (by design); incremental supervision reads Codex transcripts first | |
| Continuity | A hand-off message type exists; nothing puts it in a later brief | Follow-up proposals and durable worker messages (by design); inclusion in a later brief not stated | |

This effort does not change the controller, its tickets or its repair caps. Where I2 lacks a stated
behavior, this effort's contribution is the scenario and its result, handed to the owning effort.

## Testing Decisions

- A good test drives an instantiation only through the commands a lead or worker actually uses and judges
  only what an outside observer can see. It never reads a participant's own claim of success.
- **Seam.** The benchmark's task library, materialization and independent verifier, extended with a run
  observer. `controller-backed-bench` reuses the same task and verifier seam together with the controller's
  command and query interface. The existing fleet runner is not the seam: it commits on the worker's
  behalf, discards out-of-bounds changes, lands by rebase, treats blocking edges as advisory, and verifies
  a copy with no git metadata.
- **Harness work this needs, none of which exists today:**
  - a scripted participant that can take several steps and misbehave on purpose (today's is one command, offline);
  - a Codex participant kind (today: reference, no-op, command, traecli);
  - a reviewer role and a rework round;
  - a verifier that sees commits, refs and the recorded decisions;
  - a run observer for processes, worker disk use and a run-owned directory standing in for external state;
  - a mode per instantiation: lead-driven (built under this effort) and controller (being specified in `controller-backed-bench`).
- Scripted participants reproduce the case study's failures on purpose: mark done before committing; leave
  the tree dirty; tick every item; modify a protected external entry; write outside declared paths; go
  silent; outlive the ticket; ask a question; submit on a stale base; edit their own ticket's status.
- Stories 8, 13, 37 and 45 are judged only in a live trial. Story 8 is judged by seeding a ticket whose
  obvious implementation breaks a stated spec requirement and observing whether review rejects it.
- Prior art: the benchmark's multi-ticket tasks with dependencies; the toolbox's worker tests, which use a
  fake worker binary against a throwaway repository; the landing tests, which race two landings.
- Live trials: I1 as in item 8 above. I2 under the limits its own effort sets (a disposable repository,
  two tickets, bounded Attempts), which means I2's first live result will not cover dependent tickets or
  review-driven rework; the comparison table shows those as not yet run, not as passes.

## Out of Scope

- Running Corenius tools in a sibling repository, importing its tickets, or replacing its coordinator.
- A runbook for other repositories; that follows a passed live trial.
- Building or un-parking I3. It is described and its expected coverage is stated from its roadmap.
- Changing the accepted status, ownership or repair caps of any controller ticket.
- Context-window supervision of a worker, which the worker-monitoring effort owns.
- Cost and token budgets beyond reporting what the runtime exposes.
- The repository layout work in `semantic-polyglot-layout`.

## Further Notes

- The critique that produced this draft is summarized in `evidence/critique-1.md`. It was made by a
  separate context of the same model family; the workflow prefers another family when one is available.
- The case study is evidence for the problem, not a design to copy. Its coordinator reads mutable ticket
  files and types into terminals; the scenarios keep what that achieved without those mechanisms.
- A worker retained for rework is the same thing as the lingering process the case study criticises.
  Story 40 resolves the tension by making retention explicit and counted.
- The first plan from this session (three toolbox tickets) is superseded.
