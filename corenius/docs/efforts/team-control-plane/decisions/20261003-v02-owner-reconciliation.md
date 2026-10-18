# Reconcile the owner's RFC0002 correction into implementation

Owner source: RFC0002 capture `768bb943e9a95b9e761e3d1e7de08c0ac8df74b4`, retained on root `942f037800102f3d786ac547e72f11ab928b50ae`.
Accepted kernel baseline: `264756edd550225449202b6f20ac4b560808d14e`.

The owner requires adoption into active implementation, not capture alone.
The [reconciliation](../v02-reconciliation.md), SPEC precedence, updated PLAN
and canonical tickets26–32/revised25 implement that planning change. This
supersedes the target's earlier optional Kata/deferred jj defaults while
preserving their historical accepted evidence. Corenius alone is in scope.

Independent architecture review identified exact dependency-trigger, accounting,
legacy NULL Attempt and caller-authority ambiguities. Each is now explicit:
broker-admitted intent drives dependency suspension; cumulative credits and
reusable capacity conserve independently; original nullable role provenance
is retained; explicit admitted delegation bounds every new mutation. The
revised architectural assessment and separate adversarial Spec gate pass.
Exact original/revised reviews
are retained under evidence/v02-adoption/.

New assembly32 is required before live adapter21. Scope/job adapter31 is
independently testable before interpreter29/protected gate30, preventing a
dependency cycle. Existing lead_generation is reused. Current artifact handoff
is not physical protected integration. External product assertions remain
unverified until pinned capability tests; qualification failures block or
receive explicit disposition rather than silently restoring old deferrals.

No repository policy/tracker migration, shared store conversion, sibling edits,
unbounded provider spend or product readiness follows from this record.
Existing owner full-goal authority covers necessary isolated implementation,
review and gated fast-forward landing; technical admission remains evidence-based.
