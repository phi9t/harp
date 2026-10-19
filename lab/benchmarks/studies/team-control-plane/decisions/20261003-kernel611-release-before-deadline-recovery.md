# Preserve checkpoint flow and eventual episode recovery

Clarification of the authorized repair611 lifecycle contract, after fullmatrix28026 exit101 exposed premature retirement during normal checkpointing. This does not relax the original goal or require new permission.

Before the canonical deadline, physically completed/released execution may still checkpoint through the existing YieldEpisode contract. After expiry, the same principal must be able to admit a successor when the exact registered execution has independently proven known shutdown/effects/usage and completed release. Both orders must work: deadline then release; release then deadline. Recovery must not depend on a human repeating Release after expiry.

Add public release-before-deadline then expiry then successor proof and predeadline release then yield compatibility. A narrow reconciliation seam under the existing broker lock may retire only associated current-authority admitted/active episodes backed by exact known released execution; preserve yielded/checkpoint/wait state and uncertainty/capacity/budget fences. No unexecuted or unproven expired episode is retired merely due to time.

Preserve current-policy candidate/report error precedence before execution-provenance comparison. All policy relabel/cached receipt refusal controls and full tests remain mandatory.
