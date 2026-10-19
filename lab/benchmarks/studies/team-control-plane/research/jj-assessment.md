# Jujutsu: workspace backend assessment

Status: read-only assessment; recommended placement is inference
Read: 2026-10-01
Repository: https://github.com/jj-vcs/jj
Commit read: `030ecf22c56b98bf6507067dca0df3dffa6de498`
Corenius base read: `963f58b31e518a8162dfe719e386ef9d9843321f`
No jj installation, repository conversion or commands against workspaces performed.

## Observed documentation

Jujutsu uses Git for its production-ready commit backend and supports Git
remotes. Its workspace model differs from Git worktrees; compatibility docs
explicitly do not promise git-worktree support. Hooks, attributes and other
Git features have compatibility limitations in the inspected documentation.
See [Git compatibility](https://github.com/jj-vcs/jj/blob/030ecf22c56b98bf6507067dca0df3dffa6de498/docs/git-compatibility.md).

Most commands snapshot working-copy changes; new nonignored files are implicitly
tracked by default. Multiple workspaces share repository storage. Rewriting a
working-copy commit from another workspace can make that workspace stale.
Snapshots occur at command boundaries, not continuously for every shell write.
See [working copy](https://github.com/jj-vcs/jj/blob/030ecf22c56b98bf6507067dca0df3dffa6de498/docs/working-copy.md).

The operation log records repository views and supports inspection, undo and
restore. Concurrent commands can produce divergent operations/changes that
need resolution. This is version-control concurrency handling, not ownership
or process isolation. See [operation log](https://github.com/jj-vcs/jj/blob/030ecf22c56b98bf6507067dca0df3dffa6de498/docs/operation-log.md).

A change ID generally survives a rewrite; its commit ID changes. A change can
have multiple visible revisions. Verification therefore cannot bind only to a
change ID, bookmark or working-copy alias. See [glossary](https://github.com/jj-vcs/jj/blob/030ecf22c56b98bf6507067dca0df3dffa6de498/docs/glossary.md).

## Recommendation (inference)

Factor jj into the workspace/integration backend design and evaluate it in a
fixture repository after the Git baseline. Keep the durable kernel independent
of VCS choice. PostgreSQL owns team execution state; Temporal owns durable
control flow; Kata may own planning issue ergonomics; jj may improve creation,
checkpointing and composition of code changes. None substitutes for the others.

Use a workspace seam with create_attempt, inspect, checkpoint, freeze_candidate,
prepare_integration and release operations. Each operation is broker-authorized,
journaled and reconciled. Record backend/store/workspace identity, base commit,
operation ID when available, change ID for navigation, exact commit/tree digest
for evidence, conflict status and captured-path manifest. Immutable artifact
publication stays separate from mutable local repository state.

First backend: existing Git worktree mechanisms with lifecycle/GC fixes. Second
candidate: pinned jj release with separate workspace per attempt. Do not convert
the shared checkout, mix Git/jj mutation owners or grant workers unrestricted
shared-store rewrites. Shared jj storage is not a security boundary; use separate
repository stores or a trusted serialized VCS service where cross-attempt
mutation is unacceptable.

A candidate is frozen as an exact conflict-free commit/tree and retained artifact.
Any rewrite, rebase, squash, changed parent or conflict resolution creates a new
candidate requiring applicable verification. A jj operation restore cannot
undo a running process, deployment, remote push or accepted broker decision.
Recovery must distinguish repository repair from external-effect reconciliation.

Evaluation gates: worker edits/checkpoints/replacement; crash between repository
operation and working-copy update; divergent operations and stale workspace;
ignored/private file exclusions; capsule path access; conflict rejection before
Git export; exact commit/tree proof after rewriting; existing gate/landing,
evidence and GC tooling; and a human-approved fast-forward-only shared landing.
Git compatibility is not proof that this repo's tools work unchanged.

Adopt only if the experiment reduces lost-work/recovery or integration friction
while preserving one writer per tree, candidate immutability and current shared
branch policy. Otherwise retain Git. This is an optional experiment, not a new
kernel dependency or permission to rewrite shared history.
