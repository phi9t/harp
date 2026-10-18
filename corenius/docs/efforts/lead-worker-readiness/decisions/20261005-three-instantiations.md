# Decision: three instantiations of the idea, judged by the same scenarios

Date: 2026-10-05.
Authority: the owner, in the lead session, after the seam and substrate decision of the same day:
"we should have 3 different instantiation of the idea (the first is what's being done in sureal repo where
claude spins up a bunch of workers and they take up the issue tickets made by /to-issues | the second is a
controller with sqlite/kata/jj as is being designed now | the third is the one with a more complex
controller and temporal dynamic workflow execution)".

## Accepted direction

The readiness scenarios are one yardstick. Three instantiations are measured against it:

1. **Lead-driven.** An interactive lead agent launches workers that take tickets produced by the ticket
   splitting skill; coordination is the lead plus small tools. The sibling repo's coordinator is the worked
   example (case study); in Corenius the agent toolbox is the corresponding material.
2. **Local controller.** A controller with SQLite as execution store, Kata as work ledger and jj as worker
   version control, as designed in the `sqlite-controller` effort.
3. **Workflow controller.** The richer controller with Temporal dynamic workflow execution, as designed in
   the parked `team-control-plane` roadmap.

## Effect on the earlier decision

This supersedes item 2 of `20261005-seam-substrate-runtime-and-network.md` ("the local controller is the
target; the toolbox is changed only for uncovered groups"). The lead-driven form is now an instantiation
in its own right, not a gap filler. Items 1, 3 and 4 of that decision stand: the benchmark harness is the
one seam, Codex workers come before traecli workers, and network is on by default.

## Not decided by the owner's words (lead's reading, to be confirmed)

- Whether every instantiation must pass every scenario, or each declares per scenario: passes, not
  applicable by design, or fails.
- Which instantiation passing counts as "Corenius is good enough".
- The order of work across the three, given the standing instruction to finish the local controller
  baseline first and that the workflow controller is parked.
- Source read note: the `sqlite-controller` spec at `d7f6748` names Git as the required first workspace
  backend and tracks jj qualification separately; the owner's wording names jj for instantiation 2.
