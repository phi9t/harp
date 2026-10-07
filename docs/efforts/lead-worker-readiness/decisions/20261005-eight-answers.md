# Decision: the eight open items of spec draft 2

Date: 2026-10-05.
Authority: the owner answered "sure" to the lead's message listing eight questions, each with a
recommendation. The recommendations are therefore accepted as put. The questions, as put:

1. "Worker commits. May a worker commit in its own workspace and hand over an exact commit, reversing the
   29 September rule? I recommend yes."
2. "The lead never writes in a worker's workspace. I recommend yes, for all three."
3. "The core set. You said 'a good amount of tests'. I proposed sixteen stories covering hand-off,
   acceptance, rework, launch by blocking edges, gated landing and the external-state report. Say if that
   is the right bar."
4. "Park exemption. I read 'get 1 up and running first' as exempting lead-driven work from the controller
   spec's instruction to park unrelated work. Is that right?"
5. "Network default. I propose it applies to the lead-driven form, and the controllers record what their
   design grants."
6. "Container builds and shared-cache writes. I propose off by default, declared per launch."
7. "Integration branch. May a lead land accepted work on an effort's integration branch without you, with
   only the move to mainline being yours?"
8. "Live trial shape. I propose a real small Corenius effort: at least three tickets, one blocking edge,
   two parallel workers and one rework round."

## Accepted direction

1. A worker commits in its own workspace and hands over an exact Candidate. This supersedes, for this
   effort's instantiations, the rule in `docs/practice/lead-supervisor-worker.md` (decided 2026-09-29)
   that the lead, not the worker, commits. A worker still cannot move a shared branch, merge or push.
2. The lead never writes in a worker's workspace.
3. The core set is stories 1, 2, 5 to 7, 9, 12, 14 to 18, 20, 27, 34 and 35 of `SPEC.md` draft 2.
4. Lead-driven work under this effort is exempt from the controller spec's instruction to park unrelated
   work. Nothing here changes the controller, its tickets or its caps.
5. Network on by default applies to the lead-driven instantiation; for the controllers the scenario
   records what their design grants.
6. Container builds and writes to a shared cache are off by default and declared per launch.
7. A lead may land accepted Candidates on an effort's integration branch without the owner. Moving the
   mainline remains the owner's.
8. The lead-driven live trial is a real small Corenius effort: at least three tickets, one blocking edge,
   two parallel workers and one review-driven rework round.

## Limits of this answer

A one-word answer to eight questions. Items 3, 4 and 7 were phrased as questions with the lead's reading
attached, not only as recommendations; they are taken as accepted in that reading. Not covered: authority
to commit or land this effort's record, and authority to launch workers or spend model budget.
