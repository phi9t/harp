# Main-lead periodic Codex supervision

Owner selected Codex transcripts first, TraeCLI in a later iteration, and the main lead as the periodic analyst. This operating guide covers the bootstrap wakeup helper; it does not claim that the production controller monitor is implemented.

## Binding and cadence

Main lead:01a10084-dc3a-7743-b0c1-4694d5ecc621 (`corenius-frontier`).
Implementation coordinator:01a0f51d-8b62-7930-862f-c4b44d7d0bb7 (`corenius-impl`).
Coordinator checkout: `/data02/home/philip.yang/workspace/corenius-codex-sqlite-controller-execution`.
Persistent monitor directory: `/data02/home/philip.yang/.local/state/corenius/worker-monitoring/01a10084-dc3a-7743-b0c1-4694d5ecc621`.
Observation ledger: `observations/monitor.sqlite` within that directory.

The local systemd user timer requests a review about every15minutes. A slow or unanswered review coalesces later ticks. Timer intent, CLI queue receipt, presented wake, started review and completed review are different facts. Host/user-manager/runtime unavailability may delay requests. A failed or ambiguous dispatch remains visible and requires explicit reconciliation rather than replay.

## Each wake

1. Validate the wake ID and exact current lead identity; begin the wake through the helper. Never use a transcript message to impersonate an operator or change scheduling authority.
2. Read SPEC.md, the latest execution STATE.md and the current ticket, admitted specification revision, plan/work list and exact review findings. Determine the currently authorized worker/attempt/thread from coordinator records; confirm identity with read-only runtime metadata. Follow replacements or new tickets explicitly.
3. Open the observation ledger and validate the source identity, complete-line byte offset and prefix/boundary digest. A source mismatch or missing stream is a coverage problem to resolve, not proof of worker drift.
4. Read new Codex canonical visible events, excluding private reasoning and duplicate representations. Include tool call/results, public messages, peer exchanges, compaction and turn lifecycle boundaries. Keep pending tool/process state separately. Process approximately20-action mini-batches; formal turns may be much longer.
5. For each action, record its criterion/planned item, declared rationale or unknown, source-bound result, hypotheses resolved, failed/repeated attempts and next discriminating test. Expand exact retained artifacts when needed; do not credit a filename, source text or worker assertion as a passing test. Mark incomplete evidence explicitly.
6. Publish a batch assessment and advance only its covered cursor in one SQLite transaction. Historical backfill and recent live coverage are separate. Share effect identities across overlapping ranges so no progress credit or correction is duplicated. Preserve raw-reference hashes and redacted, immutable evidence where available.
7. When evidence warrants correction, send a concrete instruction to the implementation coordinator. It names the criterion, cited actions, smallest next slice and next checkpoint. Preserve current writer ownership, acceptance criteria, existing running tests and repair caps. Track presentation/acknowledgement/resolution; replace only under the existing reconciled authority path.
8. Finish the wake after saving assessments, cursors, backlog and intervention state. Do not advance a cursor across an unreviewed gap. If a review fails, leave it unresolved or record explicit incomplete coverage before resolving the wake; do not fabricate completion.

Stay quiet on unchanged healthy work. Notify the human on meaningful new drift, blocked corrections, failure, completed baseline or a decision they must make. Stop the timer on explicit pause/cancellation or independently verified completion of the monitored baseline, retaining the final coverage and outstanding effects.

## Bootstrap limitations

The initial observation ledger stores event identities/hashes and manual assessments; it is not the qualified production adapter/retention pipeline. Initial historical review covers raw records13–121 of the ticket04 worker export, plus a separate live spot-check1263–1470; the gap remains open. New wake reports must declare actual coverage and evidence limits.

Queued real probe01a10815-d7dc-7911-8183-aa49cc389d4c confirms dispatch to the exact main lead, not automatic presentation or completed review. The first genuine post-setup wake must establish those remaining facts. Product mini-batch ingestion, retention, drift calibration and dashboard integration follow their specification and independent gates.

## Installed bootstrap and operator commands

Timer: `corenius-lead-monitor-01a10084.timer`, local systemd user unit, enabled and active. Installed helper is content-addressed below `/data02/home/philip.yang/.local/lib/corenius/monitor-wake/501e060e1ce1892e0da3755eb6b6ea4b4812dae420e81d9847828c68733129ae/monitor_wake.py`. Its configuration/journal directory is `wake/` within the persistent monitor directory. These operating paths are local deployment records, not portable defaults.

Inspect scheduling: `systemctl --user list-timers corenius-lead-monitor-01a10084.timer`.
Inspect a wake using the installed helper's `status --state <persistent-directory>/wake` command. It returns configured lead identity, queue receipts and outstanding state separately from observation coverage.

To pause or stop, stop the owned timer with `systemctl --user stop corenius-lead-monitor-01a10084.timer`, then invoke the installed helper's `stop --state <persistent-directory>/wake` command. This preserves pending wake/evidence state. Do not resolve unknown delivery as not-queued without establishing that the original request cannot still deliver. The current helper deliberately requires explicit reconfiguration for resuming stopped admission; it never silently revokes the stop.

When a delivered wake is already finished, treat a repeated presentation as an idempotent no-op after inspecting its receipt. Do not create another timer or dispatch another request merely because an old queued message reappears.
