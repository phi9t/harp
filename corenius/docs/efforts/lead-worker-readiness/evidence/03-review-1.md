# Lead review 1 of ticket 03: changes required before it can land

Reviewed Candidate: 3128942. The record format, the validation, the landing precondition with its own exit
code, the waiver handling and the five scenarios are right and stay. Do items 1 to 3 now. Do NOT rebase
yet: another ticket that changes the same harness module is about to land, and I will tell you when to
re-integrate so you only do it once.

## 1. The lander looks for the acceptance record in the wrong place when the target has its own worktree (blocking)

`land.py` reads the record under `repo_root(--repo)`. The effort's record lives with the effort, on the
branch being landed onto, and that branch is normally checked out in its own worktree (mainline in the main
checkout, integration branch in the lead's worktree). I reproduced it: main checkout on `main`, `integ` in
a second worktree, the record written there with your command. `land.py <candidate> --repo <main checkout>
--mainline integ --effort demo --ticket 01` answers `acceptance refused: acceptance record missing`, exit 7.
Run from the integration worktree it lands. The outcome must not depend on which worktree the lead stands in.

Required: read the record from the worktree that has the target branch checked out (the lander already
finds that worktree for the fast-forward); fall back to `--repo` only when the target is checked out
nowhere. Add this two-worktree case to a test and to one scenario.

## 2. Say how the record and the landing fit together, and test it (blocking)

The record names the Candidate, and it is a tracked file on the branch the Candidate must fast-forward. If
the lead commits the record before landing, the target moves and the Candidate is stale. So the only order
that works today is: write the record uncommitted in the target's checkout, land, then commit the record.
That is fine, but it must be stated and held by a test:
- the help text of `acceptance.py record` and of the lander's `--ticket` option says the record is written
  before landing and committed after;
- a test shows a landing with the uncommitted record present leaves the record file untouched and the
  checkout otherwise clean;
- a test shows that committing the record first makes the lander refuse the Candidate as stale (exit 6),
  not as an acceptance failure, and the message says a new Candidate on the new base is required.

## 3. Small things

- `claim_sampling.report` and `.outcomes` are stored as absolute paths. Store them relative to the
  repository root when they are inside it.
- Under `## For later tickets` add: `- to 14: required checks are named by the reviewer when recording; nothing
  yet ties them to the ticket. The live trial should name each ticket's required checks in the ticket.`

## Protocol

New commits on top of 3128942; no rewriting. Update the report. When items 1 to 3 are done and your gate
passes, commit them and stop WITHOUT the marker commit; wait for my message telling you to re-integrate.
After that rebase and gate rerun, make the final commit `ticket 03: ready for review` on a clean tree.
