# SQLite controller operating improvements

Status: proposed operating packet; coordinator review and amendment receipts pending.
Recorded: 2026-10-04 UTC. Documentation base: `159dd8b186630211a26d8d44967291f5700350b8`.
Canonical source read: `291f04a5abd9274e4fd0034fd36927b0a4c4d236`; mutable state and brief digests appear in [source findings](evidence/controlled-language.md).
The owner requested these improvements without changing the retained SQLite baseline priority.
This packet records requirements for routing; it does not prove product implementation or authorize additional work.
Matt routing/specification owners are unavailable; this packet applies the repository crosswalk with research and writing-for-agents guidance.

## Current position and authority

Tickets 01 through 04 are accepted within their recorded scope.
Ticket 05 has an active exclusive author: `/root/sqlite05_control_center_author`.
Its thread is `01a10854-c05f-7640-8823-954ceb788816`; its worktree is `corenius-sqlite05-control-center`.
Ticket 06 still requires assembled proof, the dedicated live two-worker trial, and final release checks.
Mainline fast-forward landing remains pending.
Root remains the sole coordinator; the controller remains the execution authority.
Analytical reviewers propose findings; the coordinator adjudicates them.
The existing monitor implementation author owns `monitor_wake.py`, its tests, and worker-monitoring documents.
This packet changes none of those files.
Root Insula isolation remains a proposal; this packet makes no implementation claim.

## Operator actions now

1. Coordinator: record each instruction's exact source, recipient binding, and durable message ID.
2. Coordinator: request an explicit receipt for each pending amendment at the next safe checkpoint.
3. Coordinator: record adopt, pending, or reject separately from transport delivery.
4. Coordinator: replace contradictory current-state sections with one current projection.
5. Coordinator: preserve earlier state in a dated immutable history record.
6. Author: retain each initial failure before another command uses its output path.
7. Verifier: record the tested command's exit status before displaying its log tail.
8. Coordinator: supply a fresh bounded brief before replacement or compaction.
9. Observer owner: keep the failed observer stopped until a fresh parser passes the public-input qualification.
10. Coordinator: show unreviewed backlog while observation remains unavailable.
11. Coordinator: route proposed ticket 05 and 06 refinements through source-bound amendment receipts.
12. Release owner: retain the existing assembled, independent review, live trial, and exact-tip gates.

Completion means recorded receipts, preserved failures, a coherent current projection, and visible observation gaps.
These manual actions do not establish an unattended timer or change a worker's admitted AcceptanceContract.

## Instruction delivery and amendment adoption

Owner: coordinator for current operation; ticket 05 author for admitted product behavior.
The controller records delivery stages independently from a source amendment's disposition.

| Stage | Required evidence |
| --- | --- |
| Persisted | Durable message ID, payload digest, source revision, sender, recipient, and creation timestamp |
| Queued | Pending delivery operation and queue timestamp |
| Presented | Identity-bound transport observation and presentation timestamp |
| Acknowledged | Explicit recipient receipt citing the message ID and current authority binding |
| Applied | Source-bound result, artifact or controller event proving the agreed change |

Terminal submission proves presentation only when the qualified adapter confirms the intended recipient.
Acknowledgement proves receipt; it does not prove adoption, execution, or acceptance.
An amendment records its proposed IntentRevision, AcceptanceContract delta, source digest, authority, and affected tasks.
The coordinator records adopt, pending, or reject with a reason and an exact source binding.
The author acknowledges the adopted revision before new scope enters execution.
Lost replies retain the original message ID; reconciliation precedes another delivery attempt.
Initial tunable thresholds: request receipt after 10 minutes; surface unresolved delivery to the coordinator after 20 minutes.
Measure each interval from its recorded transition timestamp, using the same observation clock.
These thresholds require observed checkpoints; they promise no response time and create no new poller.
The coordinator can tune them with a recorded reason while preserving authority and repair limits.
Negative controls: queued messages cannot appear acknowledged; stale recipients cannot acknowledge or apply an amendment.
Measurement: record queue-to-presentation, presentation-to-acknowledgement, and acknowledgement-to-application latency separately.

## One current projection and immutable history

Owner: coordinator now; ticket 05 author for controller-backed views.
The current projection contains one status per task, current owner, admitted revision, source tip, pending gates, and unknown effects.
It includes the projection's event cursor and observation timestamp.
Historical acceptance, repair and ownership records remain immutable and separately linked.
STATE.md currently mixes the new ticket 05 ownership with an older active ticket 04 ownership section.
That source observation establishes a documentation conflict; it does not revoke an accepted ticket or change worker ownership.
The coordinator archives the old projection before publishing a replacement current section.
Dashboard, CLI, lead brief and STATE.md identify the same current projection version.
After replay or reconnect, the view either catches up to that version or displays an explicit stale marker.
Negative controls: an old ownership event cannot overwrite a newer revision; conflicting current claims prevent a healthy status.
Measurement: record projection lag and count unresolved current-state conflicts.

## Product behavior, test validity and retained evidence

Owner: author produces evidence; independent verifier checks it; coordinator adjudicates disputed assertions.
Each failure receives separate dispositions for product behavior, test-driver validity, and artifact retention.
A faulty driver can invalidate an inference while the original failed run remains retained.
A corrected driver receives a new immutable run identity and a reason that cites the old run.
Each run records exact source, admitted requirement, suite/environment, command, timestamps, actual exit, and artifact digests.
Before publishing a receipt, the producer stores and verifies the referenced immutable artifacts.
The producer preserves original failure bytes, source snapshots and negative controls independently of later successful runs.
For shell wrappers, the producer records the cargo exit before running `tail` or another display command.
For pipelines, the driver records the tested process's exit explicitly; a successful final pipeline command supplies no test result.
Focused reruns follow each changed behavior or verified finding.
Broad required gates run at candidate freeze, after source-changing integration or repairs that invalidate their coverage, and before final release.
A narrow rerun records the changed scope and exact unchanged source identities supporting inherited coverage.
Narrow reruns preserve every required gate; they cannot turn an unrun gate into a pass.
The verifier checks source stability before and after each gate.
Negative controls: masked cargo failure, overwritten initial logs, mutated source, and old-candidate evidence all prevent credited success.
Measurements: record useful discriminating results, gate duration, review waiting time, and retention verification failures.

## Bounded briefs and context boundaries

Owner: coordinator supplies the brief; each agent reports its measured context and pending effects.
A fresh brief states current intent, exact source, ownership, constraints, findings, unresolved effects, permitted next action, and evidence pointers.
Agents retrieve exact referenced files and bounded excerpts when a question requires them.
Compaction carries public constraints and evidence pointers; it creates no replacement historical evidence.
Initial operator thresholds use measured current input tokens and the runtime's known context window.
At 100,000 input tokens, the coordinator requests a context warning and a brief refresh.
At 120,000 tokens or 50 percent of the known window, the coordinator requests compaction at a safe checkpoint.
At 150,000 tokens or 60 percent, the coordinator prepares a fresh context and a bounded handoff.
The lower applicable threshold triggers the action; unknown window size leaves absolute thresholds available.
The operator records actual compaction or handoff acknowledgement and pending effects before claiming the boundary occurred.
These are tunable operating thresholds, not financial budgets, cost counters, or automatic runtime guarantees.
Negative controls: cumulative token totals cannot stand in for current input size; a compaction summary cannot erase unresolved effects.
Measurements: record measured context, warning time, completed boundary time, and missing constraints after the handoff.

## Observer qualification and visible coverage

Owner: observer implementation author; coordinator owns delivery and intervention decisions.
The retained operator incident receipt reports private items reaching observer context before the parent stopped the observer.
That receipt records zero ledger writes; this packet grants no cursor credit for that incident.
This author inspected no worker transcript or private body.
A fresh observer must filter a strict public event allowlist before any model input, tool output, or report output.
Accepted inputs contain completed public messages/tool results, attributed peer messages, and boundary metadata.
The parser excludes Reasoning records, credentials, unknown item types, and compaction replacement histories containing private material.
Public tool outputs still require credential filtering; public event classification alone cannot establish safe content.
An unqualified or ambiguous parser fails closed and publishes only a safe coverage/error marker.
The observer preserves the last qualified cursors and exposes backlog, gaps, parser identity, and failure time.
Qualification uses retained synthetic fixtures with private/credential canaries and unknown schema variants.
The verifier checks that excluded canaries never reach model requests, stdout/stderr, reports, or analytical receipts.
A fresh observer resumes only after those controls pass against the pinned parser and source schema.
Separate ingestion, contiguous historical review, live-tail review, and intervention receipts remain visible.
Overlapping live/backfill batches may expand coverage but cannot duplicate progress credit, finding counts, or corrections.
Long single turns still seal at 20 visible completed actions, 10 minutes, or a context/assignment/handoff boundary.
Pending jobs require separate liveness observations; a quiet worker receives no presumed healthy assessment.
Measurements: record qualified coverage, backlog age, parser rejection counts, report latency, and intervention outcomes.

## Finding identity and repair history

Owner: coordinator maintains finding lineage; ticket 05 displays it; ticket 06 verifies it.
A finding retains one lifetime identity, invariant, initial evidence, and cumulative repair history across tickets, attempts, and contexts.
Routing a finding into another ticket carries its original identity and consumed repair allowance.
Equivalent failures and overlapping reviews reference the existing finding; they do not create a fresh allowance.
The coordinator retains the existing three-round cap and escalates exhaustion to the authorized architecture owner.
An authorized supersession records the old identity, reason, changed invariant, new intent, and any explicitly approved additional allowance.
Supersession preserves old failures and repair counts; it never silently resets them.
Negative controls: new ticket IDs, replacement workers, compaction, and duplicate reports cannot renew a consumed cap.
Measurement: report cumulative repair rounds and the exact authorization for any changed allowance.

## Ticket routing and acceptance

The coordinator must record a source-bound amendment receipt before admitting any new product scope from this packet.
Existing ticket requirements remain canonical until that receipt records adoption.

| Ticket and owner | Requirement after admission | Acceptance evidence and negative controls |
| --- | --- | --- |
| 05 author; coordinator approves | Separate delivery, acknowledgement, application and amendment disposition | Public API receipts; replay and lost-reply tests; queued/stale messages cannot claim acknowledgement |
| 05 author; coordinator owns current state | One versioned projection with linked immutable history | CLI/dashboard/lead agree; reconnect shows lag; historical owners cannot become current |
| 05 author; observer author owns parser | Coverage, finding lineage and qualified public-input views | Allowlist precedes outputs; canaries stay excluded; overlap cannot duplicate credit or reset caps |
| 06 verifier; coordinator admits run | Retained assembled gates and driver/evidence qualification | Actual exit capture, stable exact source, immutable failures, independent review and retention checks |
| 06 live-trial owner; controller schedules | Actual dedicated Codex lead and two workers | Process/event evidence proves overlap, stale-base reintegration, recovery and isolation; model-free fixtures remain separately labeled |

The live trial starts only after deterministic real SQLite/Git/Kata/runtime integrations qualify.
The outer harness submits literal input to the registered lead; the controller admits both workers through normal commands.
The run retains actual overlapping process intervals and distinct private repositories.
After the first candidate lands, the stale candidate requires a fresh integration candidate, verification, and acceptance.
Recovery records surviving jobs, revoked grants, unknown effects, and reconciliation before conflicting dispatch.
Isolation probes must fail against other workspaces, secrets, controller storage, protected refs, and unauthorized APIs.
The trial records acknowledgement latency, useful progress, review delay, and independently adjudicated intervention quality.
Useful progress means a source-bound result that advances a requirement or rules out a recorded hypothesis.
Review delay runs from candidate freeze to the first independent disposition; ongoing waits remain visible.
The evaluator records useful corrections, mistaken corrections, missed drift, and unresolved assessments with cited evidence.
The final packet separates fixture proof, live proof, independent acceptance, and authorized fast-forward landing.
