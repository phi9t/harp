# Proposed prerequisite: pinned verifier runtime view

Status: approved by user request to fix this situation on 2026-10-05; execution in ticket06.

## Observed gap

Ticket01 passes real deterministic title-case verification, but the Git backend mounts mutable host `/usr`. Its verifier environment identity hashes executable and script bytes, omitting Python standard library/extensions and shared libraries. Source at the recorded baseline was `crates/sqlite-controller/src/git_workspace.rs` boundary around lines 704–728 and verifier identity around 900–907; after the semantic layout migration the active path is `controller/src/git_workspace.rs`. Baseline d7f674829e9844d011869b711d64fcf55313f7f1. An ambient before/after manifest does not prevent in-run changes/restoration.

## Bounded proposal

Add an optional verifier-only pinned runtime tree to Git backend policy. Mount that private snapshot at `/usr` during verification; keep worker inspection and acceptance semantics unchanged. Include complete runtime digest in verifier environment and proof binding. Validate missing/altered files and escaping symlinks before dispatch and trusted proof validation. Existing policy compatibility remains covered.

Benchmark adapter prepares the independently owned content-addressed tree before admission and retains manifest/digest. Include Python executable, standard library/extensions, dynamic loader and required shared libraries with needed paths. Workers cannot write or load omitted ambient libraries. Reuse existing Insula tree-copy/digest conventions where sound; do not treat the current capsule ambient `/usr` mount as an immutable closure.

## Behavioral acceptance

- Unchanged title-case and clamp tests fail on noop and pass on their respective fixes through the pinned runtime view.
- A controlled change to an isolated ambient-library fixture after snapshot creation cannot alter verifier output.
- Snapshot library tampering invalidates dispatch/proof and cannot authorize acceptance.
- Worker cannot modify runtime tree or import omitted ambient libraries.
- Existing exact-candidate verification, worker inspection and acceptance semantics remain passing.

## Scope and execution

Dedicated prerequisite ticket/worktree and exact requested TraeCLI worker in the existing tmux session. No stale-acceptance redesign, general controller recovery, live model benchmark, pushes, mainline landing or unrelated runtime changes. Ticket01 stays unresolved until this prerequisite and its own review findings pass. No shared host libraries are modified by tests.
