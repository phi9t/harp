# Proposed reference-workflow ticket graph

Status: draft for owner review; individual implementation tickets are published
after approval of granularity and blockers. Parent: ticket 07. Spec: reference-spec.md.

| ID | Slice | Blocked by | Independently checkable result |
| --- | --- | --- | --- |
| 08 | Reconcile current reference truth and authority | None | Effective decisions, benchmark totals and pinned boundary claims agree or explicitly state unresolved evidence |
| 09 | Establish the common launch/lifecycle contract and demonstration fixture | None | Versioned interface, disjoint worker assignments, distinct execution/target baselines and offline containment/shutdown probes support two independent implementations |
| 10 | Codex-led lifecycle implementation and proof | 09 | Two traecli workers produce dispatch/budget and cancellation/retirement controls; Codex reproduces acceptance/adversarial checks and commits its recovery handoff |
| 11 | Claude-led lifecycle implementation and proof | 09 | Independent equivalent effort and committed recovery handoff from the same base, with no access to ticket 10's implementation or results |
| 12 | Select the reviewed implementation and publish verified operating guidance | 08, 10, 11 | Candidate comparison, independent review, shared guide and both adapters agree with selected behavior; no mainline movement |
| 13 | Prove Codex fresh-session recovery | 10 | Codex's committed effort supports one fresh-harness recovery without prior chat; unavailable evidence is reported |
| 14 | Prove Claude fresh-session recovery | 11 | Claude's committed effort supports one fresh-harness recovery without prior chat; unavailable evidence is reported |
| 15 | Verify the reference milestone and prepare human integration | 12, 13, 14 | Requirement coverage, retained evidence, fresh gate and review findings form a complete integration handoff |

## Slice sizing and review

Tickets 10 and 11 intentionally repeat the same effort under different lead
harnesses. Each is a bounded orchestration ticket with two fresh worker tickets,
one for dispatch/budget and one for cancellation/retirement; worker tickets are
created in that lead's own durable demonstration effort. Each worker slice
delivers behavior plus tests and focused guidance, not an isolated code layer.
The two lead efforts share immutable inputs but no implementation branches.

The final candidate-selection ticket does not depend on recovery evidence;
recovery may run independently after each candidate proof. The milestone
handoff depends on both. Ticket 08's source reading need not block baseline
fixtures, but must finish before current operational guidance is published.

Before execution, record local-commit authority for each lead and its recovery
effort, live model budgets, reviewed test seams and complete disjoint worker
ownership. If these are absent, the affected live tickets remain blocked by
authority even when their technical prerequisites are resolved.

Both live tickets are also blocked if the execution baseline's enforced-write,
hard-timeout or owned-emergency-shutdown probes fail. A necessary baseline
repair becomes a separate reviewed prerequisite; it is not silently credited
as either candidate's lifecycle solution.
