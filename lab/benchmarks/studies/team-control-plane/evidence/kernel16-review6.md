# Kernel16 review6: final test observation correction

Candidate: `2316e09`, fixed base `1f164156261a3aa927c823919bf73d6e6d11a771`.

Independent Standards and Spec reviewers both returned PASS on the frozen
review6 source manifest. Root independently verified all 48 source hashes;
the only change relative to review5 is `tests/temporal.rs`.

The retained failing root gate2 history proves that killing a worker after a
WorkflowTask starts can require its default 10-second timeout before recovery.
The former approximately 5-second observation window was insufficient. The
correction uses a 30-second monotonic observation deadline and timeout
diagnostics. It preserves the actual incarnation-change predicate and all
subsequent recovery, completed Activity, duplicate answer, checkpoint and
history replay assertions. Production settings and execution budgets are
unchanged. Awaited RPCs can overrun the observation deadline; this is not a
hard total test duration bound or a production recovery SLA.

Fresh complete root gate3 exited 0 and ended `check: all green`. The control
plane matrix reports 129 passed, 4 ignored and zero failures; the repository
matrix also passed. Temporal fixture cleanup reports its listener closed.
Retained gate log: [root-gate3-passed.log](kernel16/root-gate3-passed.log),
SHA256 `7a4d4e6e5a3e40d52b5509f6e6e2b6d2d9e7cc8e53cdcd1bbe44c6ce2af6fa58`.

The earlier inaccessible namespace inventory cause remains unproven. Persistent
inventory failure quarantines execution and retains capacity; committed
quarantine is not reopened. The bounded inventory retry admission window is
not a hard syscall duration bound.

Branch defect review job553 is pending. This record does not accept ticket16
or claim landing, live Trae compatibility, independent lead demonstrations,
Kata/jj adoption, or backup/restore readiness.

## Subsequent branch verification

Branch review553 terminated with CHANGES REQUIRED, independently validated
against candidate2316e09. Findings concern unauthorized repair continuation,
workspace deletion before monitor-result persistence, and historical gateway
observation after task fencing. See retained
[branch review](kernel16/branch-review553.log). Earlier two-axis PASS covers
its reviewed scope and does not override these findings. The implementing
author is producing public regression tests and bounded repairs in its owned
worktree; candidate acceptance and landing remain pending.

## Subsequent verifier freshness correction

Review6 applies to the historical frozen candidate only. Branch553 repairs exposed an additional acceptance boundary: a new Verify key could reuse a successful report predating a newer failure. The author now binds assembled positive re-verification to immutable verifier episode admission revision at or after the latest failed gate event. Independent targeted design assessment supports this rule, but is not a source PASS. A focused actual SDK Complete scenario passed and its exact log/cleanup are retained in `kernel16/branch-repair/fresh-verifier-complete-green.log`. Remaining boundary controls, full matrix, frozen source review and landing remain unproven.
