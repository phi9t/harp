# Consolidate local collaboration operations behind Collab

Corenius uses the modular `tools/collab/` CLI for local ticket admission,
workspaces, checks, landing and manually requested cleanup. Git retains work
contracts and immutable code; SQLite retains attempts and operation receipts.
The tracker remains the writer of Corenius effort tickets. Standalone committed
Markdown tickets remain a supported source through the same admission interface.

This completes the owner-approved Collab Ops specification. Keeping three
independent implementations of these operations would retain conflicting safety
rules and duplicate recovery logic. Replacing the retained product controller
would broaden the change beyond local engineering operations. Collab therefore
owns these local operations while the controller portfolio remains separate.

The check subject is the frozen Integration commit. Trunk movement requires a
new Attempt and fresh verification. Corenius tracker closure is a separate,
attributed bookkeeping commit after code landing; a record failure preserves
the landed outcome and requires repair of that bound operation. An override
name must match trusted operator configuration and every override has an event.

`wt.py` and `land.py` remain compatible entry points for one release. Their
implementation resides in optional Collab modules. Historical expectations that
delete a fresh worktree or accept an unconfigured override name are corrected
because they encode the audited defects. Other argument names and exit codes
remain stable.

Cleanup defaults to a dry run. Dirty and foreign work require explicit flags,
verified recovery archives and current ownership/identity checks. Unknown/live
writers, unresolved operations and externally locked workspaces remain retained.
No elapsed age alone authorizes removal. These cooperative local controls do not
establish a hostile-code boundary or distributed execution guarantee.

The shim retirement checklist is maintained with the Collab Ops completion
evidence. Removal requires caller migration and separate compatibility review.
