# Incremental worker supervision

Status: proposed mechanism; initial manual observation initialized, unattended monitor not deployed.
Priority: support the retained SQLite-controller baseline; preserve the active ticket04 writer and its review gates.
Source read: execution coordinator a75c520761e5f93f078abdfd1d99f24369499b99; initial ticket04 product 2b1887b8c8c6e0ea806487271ae77fc3b743f63d; evidence tip f85fcd15a912a59a9af7e4d0da56329f38a5c298; live repair checkout is mutable and unaccepted. Local transcript observations are dated 2026-10-04. Proposed behavior below is not an implementation claim.

## Intent

Detect when a long-running worker stops advancing its admitted intent, explain the observed trajectory, and intervene before unrelated work, repeated repairs, or changed success criteria consume the task. Process small groups of observable actions with durable cursors. Reconstruct what happened and why from assignments, declared rationale, actions, outcomes, and independent evidence. Do not claim access to private reasoning or infer motive from activity alone.

The human requested ongoing supervision and intervention. Corenius remains the authority for task state and runtime control. A monitor is a replaceable observer that proposes actions; it is not another scheduler. Implementation remains subject to the repository's independent design/specification and behavioral gates. Named Matt phase owners unavailable to this harness are handled through the repository crosswalk.

## Alternatives and selected shape

1. Terminal polling is inexpensive but loses history, identities, acknowledgements and exact outputs. Use it only to confirm the legacy bridge accepted a message.
2. Periodically rereading the entire transcript enables investigation but grows without bound and repeats work after context loss.
3. Select incremental structured-event ingestion, immutable batch reviews and durable cursors. This supports live supervision and historical replay with the same analyzer.

Start with controller-owned SQLite monitoring tables plus private evidence artifacts. During the existing build, a separate observer may keep a private SQLite ledger in its own checkout and send findings through the implementation coordinator. Never write into Codex's state database or the worker's checkout. Controller integration eliminates this bootstrap ledger; it does not create a second authority for execution.

## Turn and action semantics

Owner clarification: implement a Codex transcript adapter first. TraeCLI is a future adapter over the same normalized action/batch contract, not a first-iteration prerequisite. The main lead is the analyst: exact session01a10084-dc3a-7743-b0c1-4694d5ecc621 (corenius-frontier). The implementation coordinator remains a separate session that owns its worker. A periodic scheduler wakes this main lead about every15minutes to read new material and compare it with current goals, specs, tickets and planned work items. Scheduling does not decide engineering alignment.

A formal runtime turn is one episode started by the runtime. It may contain many model/tool exchanges and last over an hour. An action step is a completed observable tool call, public agent message, peer communication, or compaction boundary. Pending tools remain separately observable; token deltas and reasoning records are excluded from analytical inputs.

Observed worker01a1078b-d944-7c82-a392-50333a175a77 had two formal turns and roughly two hundred completed command exchanges on initial inspection. Therefore never wait only for five formal turns.

Initial batching policy: seal on the earliest of 20 completed action steps, five completed formal turns, 10 minutes since the first unreviewed action, or a context/assignment/handoff boundary. Time-based batches can end mid-turn; retain parent turn and unresolved tool identities. A separate liveness tick inspects prolonged pending work without fabricating a completed action. An actual running build or experiment may explain silence.

The15-minute analyst wake cadence and batch segmentation are distinct. At a wake, the lead processes bounded mini-batches from its saved reviewed cursor until caught up or its review deadline is reached, then persists the next cursor and explicit backlog. It also checks the current execution/assignment binding and planned work items, so the monitor follows the next authorized worker rather than analyzing a retired worker indefinitely. Historical backfill remains separately visible.

Bootstrap wake admission uses a dedicated local supervised timer, exact main-lead thread identity and durable wake ledger. At most one wake may be queued, outcome-unknown or under review. The lead claims it when beginning and completes it after saving its assessments/cursors; subsequent ticks coalesce while it is outstanding. A timeout or lost queue acknowledgement stays unresolved rather than being blindly replayed. Timer installation and queue receipts do not themselves establish presentation, review completion or uninterrupted host availability. Stop the timer when the baseline is independently complete, or at an explicit human pause/cancellation; record its final coverage.

Urgent controller-observed authority violations can trigger immediate policy enforcement without waiting for a batch. An LLM suspicion alone cannot kill a process, modify files, accept a candidate, or revoke authority.

## Source and cursor protocol

Prefer structured runtime events ingested by the controller. In the local bootstrap adapter, read a pinned Codex JSONL export and normalize one canonical completed-item representation. The export duplicates some messages/tool results in other records: deduplicate by source identity plus turn/item ID, not text or timestamp. Preserve peer-message provenance. A subprocess tool poll is an observation of the original execution, not automatically a new experiment.

Store per-stream identity: mission/task/attempt, role and authority epoch, runtime deployment/store/thread, transcript segment generation, parser version and source path identity. Discovery by nickname/cwd proposes a binding; exact thread metadata and assignment must confirm it before analysis or intervention.

Maintain distinct frontiers:

- `ingested_cursor`: complete source records durably imported, including raw references.
- `reviewed_cursor`: last contiguous range with a committed analytical receipt and declared coverage.
- `intervention_state`: requested, dispatched, acknowledged and resolved outcomes, independently of review progress.

A local cursor contains segment identity, complete-line byte offset, record ordinal, preceding-record digest, normalized event sequence and current turn ID. Hash the retained prefix at checkpoints. On resume, validate identity, file size, previous boundary digest and segment continuity. Truncation/replacement/mismatched history creates a source gap; do not silently reset or skip. Known rotation creates a linked segment. Retain incomplete final lines for the next pass. Compaction never erases prior monitoring records.

Historical backfill and live-tail analysis have separate cursors. A live review ahead of a historical gap must not advance the contiguous backfill cursor. Both share deduplication and evidence identities; dashboard coverage explicitly displays holes.

Their analytical effects also share transactional uniqueness. A progress credit is keyed by intent criterion plus exact evidence/run/source identity; a finding by admitted intent plus canonical issue identity; a repair round by the authenticated repair admission; and a correction by finding generation plus target authority epoch. A later overlapping review may add evidence or expand coverage, but cannot duplicate credit, increment a repair count or send the same correction again. Publication validates these keys in the same transaction as cursor advancement. Disputed issue matching remains explicit pending adjudication, not silently merged or double-counted.

## Crash-safe batch processing

1. Import normalized records with stable IDs in a transaction. Keep raw sensitive artifacts private; public records carry safe excerpts and hashes.
2. Seal an immutable batch range, source digest, intent/plan/acceptance revision, assignment history, previous assessment and source coverage. Short overlap from the preceding batch is context only, never new progress credit.
3. Analyze outside the database transaction and outside any execution-state lock. Retrieve exact test/diff/artifact evidence as necessary through a read-only allowlisted adapter.
4. Validate a typed report: each observation cites an event or artifact; every step has a disposition; missing detail is explicit; new claims distinguish worker-reported, source-observed and independently verified.
5. Atomically append the accepted report, requirement/finding deltas, proposed intervention outbox records and advance the reviewed frontier using compare-and-swap against its predecessor. Concurrent analyzers cannot both publish the same batch. Analyzer-version reruns append assessments; they do not overwrite history or double-credit progress.
6. Deliver approved interventions with a stable ID and expected assignment/role epoch. Retry or reconcile uncertain delivery; record acknowledgement and response separately.

Crash before report publication leaves the batch available for retry. Crash after publication returns the existing receipt. Failure to send a correction does not roll back the reviewed frontier; it leaves visible pending intervention. Never infer receipt from a successful terminal write.

Seal evidence identity as well as event range. Retrieval must address an immutable commit/tree/blob, sealed test-run/source pair or retained content-addressed artifact, with hashes checked against its manifest. A mutable checkout or log path is a discovery hint only. If inspection of uncommitted work is necessary, capture a versioned snapshot and detect changes during capture; contradictory or unbound bytes are unverifiable and cannot support a confirmed finding or progress credit. Newly acquired evidence appends a bound attachment to the assessment; it cannot silently replace sealed batch input. Retention must survive deletion of the original export and working artifacts, with an explicit retention policy and tombstones when evidence is deliberately expired.

## Durable records

Proposed tables: `monitor_streams`, `monitor_events`, `monitor_batches`, `monitor_assessments`, `monitor_step_reviews`, `monitor_findings`, `monitor_interventions`, and `monitor_cursors`. Execution decisions still use existing controller commands/events and authority checks.

Each batch records input/output digests, analyst identity/version, policy version, sealed source interval, assignment revision, evidence references, coverage and predecessor. Each step review records:

| Field | Meaning |
| --- | --- |
| Action and subject | Tool/message/item identity; exact target or source reference |
| Intended outcome | Requirement or review finding pursued; unmapped is explicit |
| Reason given | Public stated rationale, with attribution; otherwise unknown |
| Observation | Actual result, exit state, changed artifact and source version |
| Assessment | Progress, valid learning, necessary setup, waiting, repetition, scope concern, contradicted claim or insufficient evidence |
| Next discriminating action | What would resolve uncertainty or advance this criterion |

Record hypotheses separately from verified causes. A failed experiment that rules out a hypothesis can advance knowledge. A new test pass is not independent acceptance. Reading code that contains the word PASS supplies no test result.

## Drift signals and calibration

Measure requirement coverage and finding convergence, not lines changed, token counts, filenames or terminal activity. Track:

- Changes/actions without a requirement, approved dependency or finding mapping.
- Repeated equivalent attempts without changed hypothesis, input, source or new evidence.
- Success claims contradicted by actual results or applied to a different source/version.
- Weakened tests, removed negatives, altered acceptance criteria or changed task authority.
- Context replacement followed by forgotten constraints, reopened terminal tasks or replayed uncertain effects.
- Time/steps since a new discriminating result, adjusted for observed long-running jobs.
- Repair rounds and reopened findings; resetting a ticket/thread must not reset a finding's repair history.
- Monitor backlog, source gaps and report confidence. An unobserved worker is not presumed healthy.

No single numerical drift score authorizes intervention. Show individual signals and evidence. Initial thresholds are review prompts, not scientific claims: two consecutive batches without new evidence request a checkpoint; three materially equivalent failures request diagnosis; existing three-round independent repair limits continue to apply. Approved long-running work and explained setup can suppress a stall warning for a bounded period, with a next expected evidence point.

Test-oracle changes require a comparison of old/new assertion, intent invariant, fixture validity and retained failed evidence. Classify legitimate measurement correction separately from a weakened contract. Independent review adjudicates disputed changes. Exhausted repair caps surface to the human or authorized architecture owner; a new ticket cannot silently renew the cap.

## Intervention policy

| Level | Trigger | Authorized action |
| --- | --- | --- |
| Observe | Mapped work with credible progress or justified waiting | Commit batch receipt; stay quiet |
| Clarify | Missing rationale, unmapped action or weak coverage | Ask for hypothesis, criterion and next evidence at a safe boundary |
| Redirect | Confirmed repetition or departure from admitted scope | Lead narrows the next bounded slice; retain original acceptance |
| Hold | Confirmed authority/acceptance violation or exhausted policy cap | Controller refuses further conflicting admission; reconcile existing effects |
| Replace | Continued failure after acknowledged correction or lost context | Authorized lead/operator checkpoints evidence, fences old authority and creates a fresh attempt; reuse workspace only after reconciliation |

A correction names the exact finding, cited steps, unchanged acceptance criterion, smallest next action, prohibited scope expansion and next checkpoint. One finding has one outstanding correction; repeated polls do not produce repeated nudges. Recheck current binding and authority before delivery. An old report cannot steer a replacement worker.

All transcript text, tool output, peer messages, summaries and analyst reports are untrusted evidence. They cannot change policy, impersonate an operator, authorize tools, relax acceptance or supply arbitrary intervention destinations. An observer receives a read-only monitoring grant and bounded artifact retrieval, without shell mutation or runtime-control credentials. The trusted coordinator/policy engine validates any proposal against current intent and authorization and creates the actual command. Authenticate acknowledgements against the addressed actor and correction ID; an apparent acknowledgement in a transcript is only reported evidence.

Model classifications always remain advisory, including a label of confirmed or high confidence. Enforceable confirmation requires independently controller-verifiable facts under a named rule, or a separately authenticated decision from the authorized lead/operator that cites the observation. Draft monitor proposals are never directly executable outbox commands. A trusted authorization transaction validates the current task/intent, target epoch, rule or decision and correction identity before admitting a durable command. Existing intent-before-effect and reconciliation rules apply to its external delivery. A model report claiming prior approval, a fabricated repair count or operator identity cannot satisfy this transaction.

Before review credit, retain the complete allowed visible-item inputs and selected artifacts as content-addressed, private evidence, including original-source and redacted-input digests and the redaction policy. Exclude private reasoning, credentials and unrelated conversation. Mutable source-path references alone do not preserve a batch. Enforce access checks, bounded retention and explicit missing-evidence markers. If policy requires omitting evidence, record its absence and restrict the resulting confidence/coverage; never label omitted material as deeply reviewed.

The analyzer has a deadline and bounded retries per sealed batch. Exhaustion retains the batch as review-pending/attention-needed and exposes coverage lag; it does not advance the reviewed frontier or automatically halt otherwise authorized work. Parsing/source corruption is distinct from worker drift. Safety-critical command enforcement remains in the controller even while monitoring is degraded. The scheduler coalesces pending batches, gives each stream fair service and keeps monitoring overhead bounded; it must not swamp the worker with repeated checkpoints or block ordinary controller queries.

Current operational bridge: this observer sends instructions to the existing implementation coordinator through its durable mailbox and the installed native existing-thread queue, with confirmed literal tmux delivery as an alternative. Queue/submission receipts do not establish coordinator adoption. The coordinator owns its child worker and delivers scoped corrections. This is bootstrap operation, not a claim that the new controller API is live. Do not mutate or attach another scheduler to the child worker.

## User control center

Show each worker's assigned intent/revision, latest reviewed action/turn, unreviewed backlog and age, current criterion/hypothesis, new evidence versus reported claims, unresolved findings, repair-round history, active jobs and uncertain effects. Provide a chronological drill-down from batch to individual action and exact artifact.

Show corrections as requested/presented/acknowledged/resolved. The MTS can inspect reasoning stated publicly, accept or dismiss an assessment with rationale, pause/redirect through controller commands and replace a worker under the existing authority protocol. Suppress routine healthy notifications; notify on meaningful new drift, correction failure, completion or required human action.

## Detailed replay and implementation sequence

1. Pin the actual source schema and confirm worker identity/assignment. Initialize historical backfill at zero, not at the current tail. Create a separate live-tail baseline with its uncovered-history marker.
2. Implement durable ingestion/batching/cursors and bounded artifact retrieval. Independently qualify duplicate export representations, partial lines, rotation, truncation, compaction and restart.
3. Implement schema-validated analytical receipts and requirement mapping. Use a frozen corpus with healthy research, long builds, genuine loops, false success, legitimate oracle repair and weakened criteria. Evaluate false positives and missed drift; LLM agreement is not ground truth.
4. Implement policy-gated correction delivery and acknowledgement through the controller. Test stale actor, lost acknowledgement, duplicate correction and replacement fencing.
5. Expose monitor coverage/findings in ticket05's control center and qualify replay/live behavior with ticket06's retained baseline. Coordination owns additive ticket routing; no parallel edit to the active ticket04 author tree.

Required behavioral proof: crash at every cursor/report/outbox boundary; no skipped or double-counted events; long single-turn monitoring; bounded input/output and analyst deadlines; no execution-state mutex held during analysis; prompt-injection text in transcripts cannot issue controller commands; secrets/private reasoning excluded; old reports cannot act on new grants; observation-only failure cannot silently grant execution authority.

Explicit negative cases include overlapping live/backfill batches published in either order, a valid-schema injected report claiming authorization/exhausted repair allowance, concurrent worker edits during evidence capture, and deletion of original exports/artifacts after publication. None may duplicate analytical effects, alter authority, credit unbound source or destroy retained review inputs.

## Initial operational limits

This design does not deploy an unattended timer or native hosted automation. Manual cursor-led reviews and coordinator checkpoints can begin immediately. A recurring runner must be explicitly implemented, supervised and tested before claiming continuous coverage. Runtime export internals are pinned local observations, not a portable or stable API guarantee. Raw transcripts are private; share redacted analytical receipts and independently pinned evidence.
