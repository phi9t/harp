# Bounded inventory observation recovery

Source commit read: `1f164156261a3aa927c823919bf73d6e6d11a771` with reviewed candidate manifest `0007f56d687897e10601ce277084127e5bc1d1dbb5994efde4d5944a7806da7a`. User approved continuing “for all things”. This records a narrowly scoped availability repair following a failed root integration gate, not a claim about the historical EACCES cause.

## Observed problem and remaining uncertainty

The root physical capacity cancellation fixture quarantined after one incomplete process inventory returned EACCES. Quarantine is terminal; later successful observations do not reopen it. Current recorded PIDs are absent. One externally traced reproduction passed without EACCES, so the exact original syscall/state and cause remain unproven.

## Selected behavior

Before committing quarantine, allow a finite, brief retry of the entire inventory scan while the same verified namespace descriptor remains held. Discard failed scans completely. Only a complete fresh inventory supplies members for existing incarnation-checked signals or establishes an empty pinned namespace. Independent certified original-PID1 finalization remains a separate proof route.

Persistent unreadable processes, missing or mismatched pins, invalid observer frames and exhausted observation budgets remain fail-closed. Never reinterpret EACCES as disappearance, signal partially observed or inferred processes, refund unknown usage, retry native effects or reopen a committed quarantine. Preserve structured error provenance and keep lock-held observation waits short. Concrete attempt/time bounds must be explicit in implementation and verification, with honest limits for individual synchronous proc reads.

## Evidence required before acceptance

Existing NamespaceObserver seam must first show a red transient-error regression, then green fresh nonempty/empty inventory, persistent failure, partial/PID replacement, missing/mismatched pin/frame and terminal-quarantine controls. Retain actual physical cancellation/capacity proof, strict/fmt, full service and root gates, immutable source freeze and independent Standards/Spec review. Successful retry proof demonstrates this recovery behavior, not resolution of an unobserved historical root cause.

Independent read-only architecture reviewer confirmed these safety constraints. Root authorized the bounded implementation under the owner's broad continuation approval. All prior evidence remains retained; no acceptance or landing follows this decision alone.

## Verified timing qualification

Independent Standards and Spec reviews passed the frozen repair. The implementation guarantees at most three scans, but checks elapsed time before observer-frame revalidation; slow synchronous frame/scan reads can cross the20ms interval. It does not promise a hard20ms scan-start, syscall or lock-duration bound. Persistent-error quarantine and capacity holds remain unchanged. Historical EACCES causation remains unproven.
