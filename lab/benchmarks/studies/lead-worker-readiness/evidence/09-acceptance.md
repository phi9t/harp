# Acceptance record: ticket 09

Candidate: the tip landed on `effort/lead-worker-readiness` by fast-forward 2026-10-05T10:25Z (base 599a306ad, the integration tip at launch; no re-integration needed).
Reviewer: the lead (not an independent context).
Review rounds: 0 of 3. Accepted on the first hand-off, 17 minutes after launch.

| Axis or check | Result | Evidence |
|---|---|---|
| Ticket and spec | pass | Seven acceptance items read against the diff |
| Repository standards | pass | Standard library only; two files outside declared paths (the lead-driven harness module and its test), reported |
| Gate on the exact Candidate | pass | lander and worker suites (42 tests); every bench harness suite; five scenarios; tracker lint |
| Lead's probes | pass | In a temp repository a missing, a relative and an in-repository protected path are each refused with exit 64 and no branch or worktree is created |

Known limits accepted, carried in `lead-handoffs.md`:
- A protected path that contains the worker's worktree or the run directory is accepted, and the run's own writes would then be reported as changes.
- Files over 1 MiB are compared by size and modification time only.
- The baseline and every supervision cycle walk the whole protected tree; cost on a large cache is unmeasured.
