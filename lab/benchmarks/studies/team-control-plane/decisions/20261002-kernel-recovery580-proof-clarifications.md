# Recovery580 proof clarification

Commit read: `dd8e14cff65c00e7936c2d0ce9471aed973c777f`.
This adds precise proof requirements to
[the bounded recovery contract](20261002-kernel-recovery-repair580.md), following
independent Spec critique. It does not reset iteration2 or expand architecture.

Preserve the result's original execution recovery epoch. `RecoveredStop`
records the current finalization epoch separately. Recovery does not silently
adopt an orphan from an earlier restore or relabel original execution evidence.

Stop proof must bind the exact namespace incarnation and complete empty
observation. If its namespace pin cannot be obtained, require retained
trustworthy completion evidence plus exact monitor and launcher termination.
Namespace disappearance or a stopped state string alone cannot establish
complete coverage. Uncertain observation retains capacity.

Hash logs through a securely opened execution-bound descriptor. Preserve
pinned directory and file identity, no-follow/nonblocking, regular-file, owner,
link and mode checks. Check identity and relevant metadata before and after
reading so uncertain replacement or mutation during finalization is refused.
A safe pathname alone does not establish stable bytes.

Both monitor and recovery persistence paths conditionally insert under the
existing lock, reread the immutable stored winner on conflict, and never update
its contents. Unknown recovered exit cannot become successful verification.
Missing historical provenance defaults to original monitor evidence.

These are required implementation details within existing approved recovery
and evidence contracts. Independent critique identified no blocking design
ambiguity; no fresh runtime behavior or acceptance is claimed.
