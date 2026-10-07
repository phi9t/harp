# 04 repository-stabilization-envelope: Produce the repository-wide stabilization envelope

Type: feature
Status: open
Owner:
Blocked by: 03
Claimed:

## Parent

`docs/efforts/controller-portfolio-stabilization/SPEC.md`

## What to build

Apply the sealed two-round inventory to every registered Corenius worktree and durable effort and publish the light Repository Stabilization Envelope while preserving all unrelated work unchanged.

## Acceptance
- [ ] The source manifest derives the complete registered set from the canonical Git common directory, exact worktree/ref registries and committed effort roots.
- [ ] Every observed worktree records identity, owner or unknown owner, branch/head relation, tracked/staged/untracked/conflicted summary, effort relationship, preservation state and next attention point.
- [ ] Observer-owned evidence paths are declared and do not appear as unexplained portfolio drift.
- [ ] Missing, changing or undeclared sources remain explicit unknowns; elapsed age and absent processes never imply abandonment.
- [ ] An independent audit reconciles envelope rows to the declared registries and confirms no unrelated worktree or provider state changed.

## Comments
