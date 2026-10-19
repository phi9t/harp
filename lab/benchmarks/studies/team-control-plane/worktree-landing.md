# Worktree landing disposition

Inventory read against Corenius main963f58b31e518a8162dfe719e386ef9d9843321f. The owner authorized gated landing across all worktrees. Registered51 trees,49 committed tips already ancestors of main,38 trees containing uncommitted paths at inventory time. Counts describe the original snapshot, not a completion claim.

Historical worker benchmark source and toolbox contributions largely match current main byte-for-byte. Ten Blade task manifests are superseded by stronger protected paths and documented fallback fidelity; the old claims test is superseded by the current seed-flakiness correction. Both evidence branches at4ed6397a41b7acdcc743ade886f182eebd25a7b7 contain a known untracked-diff-digest defect; main1715155 contains the correction, and evidence-2 dirty source exactly matches main. Landing those old tips would regress the accepted implementation.

Missing44 Blade raw logs cited by existing research are preserved under the original evidence path with an immutable hash manifest. Historical trial-2 effort summary is preserved with source/current commit provenance. Smoke text and generated research caches/build/run copies remain local execution artifacts; their presence is not a missing product contribution. Private Kata/Trae/Codex spike configuration remains local and is not incorporated into repository source. No worktree or branch is deleted or rewritten by this disposition.

Current team-control-plane work is consolidated in reference-design: accepted10–15 and17 evidence are distinct from incomplete16. Kata/jj preflights and live/restore readiness documents are preparatory evidence, not completed live capability. Kernel16 is landing-blocked until its full behavioral matrix, independent reviews and joint integration gate pass. The /tmp managed snapshot has explicit original/post-cleanup manifests; new source is transferred only after exact-preimage verification.

The consolidated accepted baseline and preserved historical evidence landed locally at `15a5ead8622d36d06723d8f11edfa769898e3b5c` after independent Standards/Spec reviews and the exact-tip repository gate. Two additional manifest-backed Temporal replay logs and the landing state record subsequently passed the same review/gate path and landed at `1f164156261a3aa927c823919bf73d6e6d11a771`. Current local `main` and reference-design HEAD both resolve to that latter commit; no remote is configured. See [baseline review](evidence/landing-baseline-review.md) and [state](STATE.md) for evidence and limitations.

Remaining landing queue: complete and independently verify kernel16 before its subsequent landing. Continue remaining full-goal phases and land each verified increment. Final audit must account for unique remaining contributions, not merely count branch ancestry. The historical dispositions above preserve source worktrees and branches; they do not claim that unfinished kernel or later phases are complete.

A subsequent isolated capacity-test tree was created at `/tmp/corenius-kernel16-capacity`, branch `codex/kernel16-capacity`, starting from landed `1f164156261a3aa927c823919bf73d6e6d11a771`. It is reserved for a new `tests/assembled_capacity.rs` fixture, with no shared-helper or product-source edits. The unpublished kernel snapshot was subsequently transferred under the primary implementer's explicit freeze: all 45 crate files matched before/after source hashes and destination hashes, and root Cargo files matched the landed baseline. Manifest `.scratch/team-control-plane-capacity/source-manifest.json` in that isolated tree has SHA256 `51d629e8ac9ef3f44414768878bc093dbf327a74c8821562f72a3a0e2052f37d`. Clean-context agent `kernel16_capacity` owns the new test file; neither snapshot transfer nor delegation establishes test success or acceptance. Its future unique test contribution must be included in the final landing audit.

The owner subsequently requested replacement of the stale restricted primary worker. The old worker is terminal and its managed source/evidence remains read-only. Replacement `kernel16_author` owns `/tmp/corenius-kernel16-author`, branch `codex/kernel16-author`, starting at landed `1f164156261a3aa927c823919bf73d6e6d11a771`. All 45 current crate files, including the restored final monitor guard, matched frozen source-before/source-after/destination hashes; root Cargo files remain identical to baseline. Its source manifest SHA256 is `939225e1d509bcc88d360361c731770076041cec49f28b1405d94d3ee5989a6c`. Replacement independently reported default-execution capability probes exit0: reversible tree write/read/delete, loopback bind, Docker daemon26.1.4 and HTTPS200, with active danger-full-access/network-enabled/approval-never policy and no overrides. Source manifest was independently verified. Fresh monitor proof remains required for behavioral completion. No old branch/tree was deleted or rewritten.

## Refreshed committed-tip inventory, 2026-10-02

Read-only inventory at17:26UTC against main1f164156 records53registered
worktrees. Fifty committed tips are ancestors of main. The three exceptions
are candidate2316e09 in reference-design and the two historical evidence
worktrees at4ed6397 already classified above as superseded by the correction
in main. See [inventory](evidence/kernel16/current-worktree-tips.json).
No other new committed contribution is waiting for landing at this snapshot.
This does not dispose of dirty source contributions by ancestry: retained
content classifications and kernel source/evidence manifests remain required.
The author tree repairs are unpublished; source integration awaits a frozen,
verified candidate. No tree or branch was deleted, rewritten or mutated by
this inventory.

## Final candidate landing mechanism

The current source candidate is `c4599e8be5720ff7da74ce886069ff0da0560b80`;
committed review575 remains pending. Final evidence records must be packaged
before landing. The stock `land.py` path unconditionally calls rebase and may
reset its branch on gate failure. To preserve this effort's fast-forward-only
rule, the prepared session-local landing command uses its lock, detached
exact-tip gate and fast-forward primitives directly. It checks the expected
candidate, main ancestry, merge-free history, tracked overlap and unchanged
refs before updating main. It performs no rebase, reset or push; a failed gate
leaves refs unchanged. It has not been executed. Existing worktrees and the
main checkout's untracked transcript remain preserved.
