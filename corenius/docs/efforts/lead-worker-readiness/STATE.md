# State: lead-worker-readiness

<!-- effort:machine:start -->
```json
{
  "base_branch": "main",
  "base_commit": "f4bfd2f7197cba943b02375f8c2f67269659ae47",
  "dehydrated": "2026-10-07T17:31:36Z",
  "effort": "lead-worker-readiness",
  "runs": [],
  "worktrees": []
}
```
<!-- effort:machine:end -->

## Where we are

The reviewed deterministic implementation and selectively imported historical
records are landed on main. Current source read:84cbfeb, with accepted code
candidate36bd01c and its review/component-gate receipts. Historical integration,
record and journal branches/checkouts were completely salvaged and retired under
the user’s broader cleanup goal. Ticket14 live qualification and15 TraeCLI parity
remain open. The repository cleanup does not establish either qualification.

## Next actions

For14/15, establish fresh isolated integration and record branches from current
main using existing effort setup; revalidate current specs, authority and gates.
Consult salvage receipts for historical source recovery rather than recreating
old code or pointing operational tools at retired checkouts.

## Open questions

No live qualification is established by this landing. Active controller benchmark
Ticket03 belongs to its own live owner and still requires independent acceptance.

## Traps

Historical record/journal code is older than accepted main. Retired branch names
are provenance, not routing pointers. Preserve archival evidence and do not
activate rejected or superseded source merely to recover a historical reference.

## Do not redo

Historical journal entries and provider probes retain their dated observations.
Do not replay model calls merely to publish those records.

## Historical initial state (superseded)

Spec draft 2 is written after one independent critique. The owner has confirmed: the benchmark harness as
seam; three instantiations judged by the same scenarios; Codex workers before traecli; network on by
default; the lead-driven instantiation built and run first, agent-heavy, with notes kept. Nothing is
committed. No tickets exist. No worker has been launched under this effort.

### Historical next actions

1. Get the owner's approval of the proposed ticket breakdown.
2. Publish the spec into tickets with blocking edges, lead-driven instantiation first: harness scenarios, then toolbox changes, then the live trial.
3. Ask the owner for authority to commit and land the effort record.

### Historical open questions

### Historical traps

Things a fresh agent would get wrong.

- The controller-backed-bench and semantic-polyglot-layout drafts live uncommitted in another agent's worktrees and change; re-read them before citing.
- The toolbox still forbids worker commits and network; both reversals are decided and neither is implemented.
- The fleet runner in the benchmark is not the seam: it commits for the worker and lands by rebase.
- Sureal is read-only from this effort.

### Historical do not redo
