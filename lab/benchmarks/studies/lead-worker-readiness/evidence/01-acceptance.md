# Acceptance record: ticket 01

Candidate: 2bc91245e (base 3a476b06c). Landed on `effort/lead-worker-readiness` by fast-forward 2026-10-05T09:09Z.
Reviewer: the lead (not an independent context).
Review rounds: 1 of 3. Round 1 rejected Candidate c153084 (`01-review-1.md`); the same worker session reworked it.

| Axis or check | Result | Evidence |
|---|---|---|
| Ticket and spec | pass | All eight acceptance items read against the diff; review items 1 to 4 addressed in 4a2aa63 |
| Repository standards | pass | Standard library only, tests beside code, fleet runner untouched, no change outside declared paths except the report |
| Gate on the exact Candidate | pass | worker suite 30 tests; every bench harness suite; nine hand-off participants match expectation; tracker lint |
| Claim sample | pass | 6 of 21 claims of the first report checked against their citations; all held, two cited one line early |
| Repository gate `check.sh` | not run | It runs cargo regardless of `CHECK_CARGO=0`; not usable per ticket |

Accepted with these known limits, carried to later tickets in `lead-handoffs.md`:
- `status` now pins the Candidate (writes the run record) although its help says it changes nothing.
- After a Candidate is pinned, a legitimate rework commit is reported as `candidate-moved`.
