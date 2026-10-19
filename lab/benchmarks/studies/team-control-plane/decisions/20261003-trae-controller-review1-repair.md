# Repair the offline controller's reviewed retention gap

Commit read: `a21316f74672f55f3c22eff94f93ce1511b51822`.
Root retained all11 round0 inputs and both independent reviews under
`evidence/trae20-controller-round0/`; no active controller source was integrated.
Standards blocks incomplete slice RED evidence and post-reservation error
retention; Spec reports partial fault/changed-command coverage. Root inspected
the source and confirmed that post-reservation input checks can return a reason
without writing the empty terminal file even while storage remains healthy.

Authorize one bounded offline repair/review round under existing full-goal
authority. Preserve the complete round0 freeze and reports. First record the
actual public failing post-reservation case; retain a terminal refusal or
uncertainty, bound to the admitted operation and input hashes, whenever storage
permits. Keep the spent started record and fail closed on retention faults.
Add explicit changed-command, later file/directory fsync, executor exception,
and terminal write/fsync controls. Distinguish failure before dispatch from
unknown effects after dispatch. Existing evidence must never be overwritten.

Complete meaningful baseline/semantic negative controls for previously unproven
test slices and label their timing truthfully. They are post-review verification
controls, not retroactive proof of the original development order. Preserve the
original process gap for independent review; do not invent an earlier RED.
Production Git-reader validation can be tested read-only against committed
inputs after integration; fixture-reader positives remain explicitly limited.

Only the sole-writer author may change controller/test sources. No Trae,
namespace, service, provider, model, Cargo/PG/Temporal, Git mutation or historical
case rerun is admitted. Freeze the new sources, preimages, patch and retained
public evidence for independent conformance. A source pass still does not
qualify a future executor's deadline/descendant containment or admit a physical
case. Prior PARTIAL/quarantine and all spent budgets remain unchanged.
