# Recovery repair after whole-branch review580

Commit read: `dd8e14cff65c00e7936c2d0ce9471aed973c777f`.
Status: bounded existing-contract repair, not acceptance or landing. Author and
independent Spec confirm all three findings after read-only architectural
assessment. This is iteration2 of the bounded repair round established by
`20261002-kernel-interface-repair575.md`, not a reset of its three-iteration
budget. The user's approved baseline, continuation and all-things authority
covers these recovery corrections; no new provider, scheduler, migration or
external effect is introduced. Uncertainty or non-convergence returns to
assessment before further patching.

## Independent capacity claims

Reconcile the original unbound Attempt reservation separately from independently
owned continuation slots. Retained history is not an active consumer. Preserve
Attempt-wide effect, usage and checkpoint uncertainty guards; do not refund an
original physically bound reservation through the unbound path. Refund each
claim atomically once; cumulative admission and repair budgets remain consumed.
Prove imported unallocated Worker Yield, separately charged continuation
allocation, cancellation and release, original refund and replacement admission.
Retain controls for unknown usage, unresolved checkpoints, active siblings,
bound original claims and repeated reconciliation.

## Recovery finalization is not successful verification

Add supervisor recovery finalization through the existing authenticated
reconcile/release lifecycle. Only a current authorized caller and active recovery
epoch may invoke it. Require fresh exact monitor and launcher termination
checks (including PID reuse), complete trustworthy namespace-stop evidence,
readable securely bound retained logs, and existing known-effect/usage guards.
A stopped state string alone is insufficient. Any uncertainty retains capacity.

Persist an immutable recovered result with `exit_code=None` and `signal=None`;
never infer an exit status or reuse it as passing verification. Preserve exact
execution, argv, candidate, original monitor identity and raw log digest.
Distinguish original monitor wait evidence from recovered stop evidence with an
explicit compatible completion-provenance field in the result JSON. Historical
results lacking the additive field deserialize as original monitor results;
new recovery provenance records the actual finalizer identity and current
recovery epoch, keeping original execution identity attributable. No SQL schema
change is required. Verification must explicitly reject recovery provenance in
addition to unknown exit; legacy valid monitor results remain compatible.

Under the existing admission/database lock, conditionally insert only when no
immutable result exists. A committed original result is never overwritten.
Repeated or concurrent finalization returns the existing result without mutation;
the original monitor must also preserve a result already inserted by recovery.
A recovered unknown exit cannot be upgraded by guessing or late overwrite.
Prove a real controlled crash after durable stop but before result persistence,
replacement Owner recovery/release, repeated finalization, immutable result
preservation and Verify/Accept refusal. Control surviving monitor/launcher,
incomplete inventory, unreadable log, unknown usage and stale authority/epoch.
Fault injection may gate the real persistence boundary, not synthesize completion.

## Raw logs and bounded receipts

Separate execution-log validation from native JSON receipt parsing. Keep native
receipts bounded at the existing1MiB limit. Raw logs have no substituted size
limit. Preserve pinned directory-relative openat, no-follow/nonblocking flags,
regular-file, owner, link-count and permission checks. Stream with bounded memory
if contents must be hashed; do not read an entire log simply to classify it.
Existing inventory does not bind log contents into its digest, so do not invent
a new digest protocol merely to remove receipt-size coupling.
Prove actual >1MiB output retained after stopped/released work, stopped old
transport, registered replacement and authenticated takeover. Refuse active
work, orphan entries, symlinks, FIFOs, foreign/unsafe files and malformed or
oversized receipts.

## Isolation and evidence

Fresh sole-writer tree `/tmp/corenius-kernel16-review580-author` at exact dd8e14c.
Production scope: existing supervisor, model-free runtime and necessary result
verification/provenance readers; public assembled fixtures and concise contract
documentation. No Cargo, migration, provider or shared-checkout changes.
Meaningful public red before each product fix, identity-bound cleanup before
oracles, serialized build/PG/Temporal resources, actual terminal codes, full
matrix and strict/fmt/diff checks, immutable48-file source freeze with preimages,
independent two-axis review, root whole gate and fresh committed branch review.
Preserve historical trees, failures and evidence. All of ticket16 and the full
six-phase goal remain unaccepted until their respective requirements are proven.
