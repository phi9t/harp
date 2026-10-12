# Ticket 06: configured override policy

## Built
Initialization accepts `--override-owners-json`; only configured owners can request an override. The durable event binds owner and Integration subject. Effort exports become attributed committed records. Uninitialized legacy repositories use trusted Git configuration.

## Verification
The initial public test failed because the init flag was unrecognized. `test_configured_owner_override_is_recorded_and_unknown_owner_is_refused` now passes, rejecting the unknown owner before any landing. `test_override_with_effort_exports_attributed_committed_record` passes and verifies the committed export.

## Claims
The public CLI is tested in disposable real Git repositories. Integration preserves existing source workers.

## Outside declared paths
Tracker `resolve --mark-reviewed` is the sole ticket mutation seam needed for Corenius closure.

## For later tickets
none

## Not checked
Full repository and independent review acceptance are recorded separately after the final source freeze.
