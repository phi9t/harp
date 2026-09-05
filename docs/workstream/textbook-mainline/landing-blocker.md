# Resolved architecture boundary discovered by the initial gate

The initial dual-edition candidate failed the repository-wide gate because
local proof evidence pinned older build inputs. The owner approved versioned
refresh, which is now implemented and reviewed. A real fresh generation has
been captured. Final gate and landing evidence is recorded in Kata issue azr7;
this document retains the reason for the architecture extension.

## Evidence

The first substantive failure was `sources.crouzeix.route_contracts`: imported
Harp provider bytes differed from the immutable candidate commit recorded in
the existing route receipt. The four Harp provider files have now been restored
byte-for-byte to mainline. CFT-35-006 uses the provider's existing `n : Type`
boundary; all finite matrix dimensions remain representable by `Fin d`.
Its fresh compiler fingerprint is
`5ce10ccd8b2e5f6a0a93932256d6df60556b450817f4324eb4bac53cd444962b`.
The specialized textbook built 3713 jobs and passed six-ledger publication/check.

Source verification then rejected the changed aggregate wrapper digest.
Temporarily restoring that wrapper exposed the next binding: the original
`formalization/lean/lakefile.toml` digest. The local evidence also binds the
active proof-source tree, which includes Sharpness. A wrapper-only split is
therefore not sufficient. The working extended wrapper and its callers were
restored; no caller points to an absent adapter.

The maintained `publish_local_formalization_evidence` operation is create-only:
it requires the fixed destination to be absent and uses no-replace publication.
Existing evidence was not deleted, edited, moved or relabeled. No failed gate
has been waived and no commit or push has occurred.

## Approved design decision

Two approaches go beyond the approved integration design:

1. Add versioned local evidence refresh, preserving previous immutable bundles
   and capturing a new real build against the current wrapper/configuration.
2. Isolate textbook compilation with a separate Lake configuration and separate
   sharpness support, leaving every evidence-bound input unchanged.

The integration lead recommends a separately specified, versioned refresh
workflow: future proof/build changes need an honest renewal path, not hand-edited
hashes. This must retain fail-closed validation, old bundle reachability and the
distinction between local compilation and independently reviewed proof evidence.
The owner approved the versioned refresh extension. Its wire contract,
failure behavior and acceptance checks are specified in evidence-generations.md.
Implementation and independent review are complete. Both review axes cleared
the first-refresh cleanup repair. Approval does not waive the final repository
gate.

The remaining landing sequence is to verify the selected evidence, refresh the
import receipt last, run the complete textbook and repository gates, then commit,
fast-forward and push. Whole-book completion remains open.
