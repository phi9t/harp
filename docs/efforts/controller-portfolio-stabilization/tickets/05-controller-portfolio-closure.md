# 05 controller-portfolio-closure: Produce complete controller-portfolio closure records

Type: feature
Status: open
Owner:
Blocked by: 04
Claimed:

## Parent

`docs/efforts/controller-portfolio-stabilization/SPEC.md`

## What to build

Extend the repository envelope into complete operational-closure records for I1, I2, I3, worker monitoring and controller-bench authority units, including live bindings, effects, artifacts, conflicts and safe dirty-source preservation.

## Acceptance
- [ ] Closure records bind durable intent and tickets, exact refs, controller stores/exports, full RuntimeBinding fields, claims, effects, caches, Candidates, verification and Quarantine state.
- [ ] Dirty, staged and untracked controller source is captured as sanitized immutable index/worktree artifacts when safe; capture refusal blocks every non-`preserve` Action Packet that depends on that source.
- [ ] Wrong-thread, wrong-store, pane/PID reuse, stale capability and widened-workspace identities remain conflicts rather than inferred matches.
- [ ] Every contradiction among prose, Git history, controller state and live observation is recorded with provenance and freshness.
- [ ] No record contains raw credential values, environments, unrestricted process arguments, provider transcripts or terminal logs.

## Comments
