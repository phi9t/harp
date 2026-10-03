# Proof-backed historical cleanup and cancelled Worker recovery

Commit read: `480d7c22bdfd926e1534c4680988d63d50618d2e`.

Whole-branch584 rejected the candidate with two Medium lifecycle findings. Independent Spec assessment confirms both; author assessment is pending. This records the minimal existing-contract reconciliation before implementation. It is final iteration3 of the existing three-iteration repair round, not a new budget or architecture.

A currently authenticated Owner may finalize an already registered stopped execution after epoch rotation when exact retained launch/PID1/complete-stop provenance, fresh monitor and launcher whole-group termination, known effects/usage and secure stable log bytes establish cleanup safety. Preserve original execution epoch and record current finalizer epoch separately. Do not adopt unrecorded restore orphans, renew old credentials, transfer ownership, admit or dispatch work. Uncertain inventory/provenance/effects/usage retains capacity.

Desired cancellation fences new execution and final monitor launch. It must not remove otherwise-current Worker observation and bounded cleanup authority. Keep authenticated principal, scope, credential expiry, issuing identity, generation and epoch requirements. Inspection/log/result/history and cancel/reconcile/release remain available when those requirements hold; stale/foreign/expired callers stay refused.

Public proof must establish meaningful reds then greens for real monitor crash followed by rotation/current Owner finalization and once-only release, and canonical Worker cancellation followed by observation and release. Include unknown-usage/effect retention, exact provenance and process guards, dispatch refusal after cancellation, and stale/expired/wrong-generation authority negatives. Existing immutable result and recovered unknown Verify/Accept refusal remain required.

Scope is supervisor authority/finalization predicates, public behavioral fixtures and concise reference clarification. No schema, provider, workflow or permission expansion. Implement only after author validity assessment and assignment in a fresh isolated worktree; freeze and independently review the exact candidate, run the full repository gate and fresh whole-branch review. Non-convergence after this iteration requires escalation under the existing budget rule.
