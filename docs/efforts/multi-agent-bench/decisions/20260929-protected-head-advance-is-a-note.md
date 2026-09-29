# protected-head-advance-is-a-note

Date: 20260929
Supersedes:

## Decision

A protected repo whose HEAD advanced by fast-forward since launch (the recorded head is an ancestor of the current head) is a NOTE
(`protected-head-advanced`), not a violation. A HEAD that is not a descendant (rewrite, reset, other branch) stays a VIOLATION
(`protected-head-moved`). A change in the protected repo's dirty state stays a VIOLATION.

## Why

Worker `author-pr` (run `author-pr-20260929T232224Z`) was marked VIOLATED with `protected-head-moved` on the main checkout. The move was
the lead's own fast-forward landing of main at 23:27:55Z (reflog: `merge effort/multi-agent-bench: Fast-forward`). The worker ran one
read-only `git status`, and its worktree contained only its seven allowed task directories. The lead lands into the protected repo
by design while workers run, so an advance cannot be a violation by itself.

## Consequences

- Re-judged `author-pr`: VIOLATED -> CLEAN (one NOTE). The old verdict is in `verdict.prev.json`.
- A worker that fast-forwards a protected repo would now be a NOTE only; that path is still blocked by the forbidden-command regex
  (git commit/push/merge/...) and by the worker's own sandbox. Enforcement stays advisory until workers run inside capsules.
- Two tests added (`test_protected_head_fast_forward_is_a_note`, `test_protected_head_rewrite_is_a_violation`); 18 pass.
- Landed on main while other workers run, so their later verdicts use the new rule.
