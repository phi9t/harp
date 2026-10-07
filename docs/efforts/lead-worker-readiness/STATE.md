# State: lead-worker-readiness

<!-- effort:machine:start -->
<!-- effort:machine:end -->

## Where we are

As of 2026-10-07, code tip7d2f69cb and records540f8f7 are being integrated on
codex/landing-queue-20261007 atop main9a92c95. The specs and tickets exist;
implementation and worker probe evidence are retained. Current independent review
found two validation defects, being repaired before landing. Ticket14 remains
open: recorded small provider probes do not qualify the complete live trial.

## Next actions

Finish bounded review repairs, run exact-candidate gates and locally fast-forward
main. Complete live-trial work separately under its effort's execution authority.

## Open questions

No live qualification is established by this landing. The controller benchmark's
active implementation worktree is a separate candidate and must be reviewed.

## Traps

The record and journal branches contain older versions of code and state. Import
only their intent, decisions, tickets and historical evidence; preserve newer
code reports. Existing branches are preserved and are not rebased or deleted.

## Do not redo

Historical journal entries and provider probes remain evidence of their dated
observations. Do not replay model calls merely to publish those records.

## Historical initial state (superseded)

Spec draft 2 is written after one independent critique. The owner has confirmed: the benchmark harness as
seam; three instantiations judged by the same scenarios; Codex workers before traecli; network on by
default; the lead-driven instantiation built and run first, agent-heavy, with notes kept. Nothing is
committed. No tickets exist. No worker has been launched under this effort.

## Next actions

1. Get the owner's approval of the proposed ticket breakdown.
2. Publish the spec into tickets with blocking edges, lead-driven instantiation first: harness scenarios, then toolbox changes, then the live trial.
3. Ask the owner for authority to commit and land the effort record.

## Open questions

## Traps

Things a fresh agent would get wrong.

- The controller-backed-bench and semantic-polyglot-layout drafts live uncommitted in another agent's worktrees and change; re-read them before citing.
- The toolbox still forbids worker commits and network; both reversals are decided and neither is implemented.
- The fleet runner in the benchmark is not the seam: it commits for the worker and lands by rebase.
- Sureal is read-only from this effort.

## Do not redo
