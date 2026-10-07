# Independent probe review

Date: 2026-10-06. Reviewer: clean-context `effect_review` agent.
Scope: read-only source, evidence, ticket acceptance, and report review.
No test execution by reviewer; the integration owner runs a fresh reproduction.

Standards: PASS. Spec: PASS. No blocking findings remain.

The reviewer checked the actual controller authentication, authorization, receipt
lookup, and commit order. It also checked Pi memo persistence and safe tool
recovery against the pinned source. The crash barrier follows the committed
controller reply, and the tool sends every recovery request through authority.

Findings repaired before final review:

- Verify controller source, manifests, and lockfile against the declared base.
- Verify Pi manifests as well as runtime source before test-time export edits.
- Recreate the four runtime package links to prevent installed packages from
  bypassing the checked source.
- Use the observed refusal string in the documentation.
- Describe receipt equality as JSON value equality, not wire-byte equality.

Reviewed implementation SHA-256:

- Python: `d0dcda08998a7fd7d2de473dce4a556e262fa2467292fde497ab3d33efee641d`.
- TypeScript: `cc55a69f2fef08c16d9b91474f0b63a2360246e8934f261699273f45bbea22b0`.

This review qualifies the bounded ledger-mutation probe. It does not qualify
production adoption, arbitrary external effects, or an Insula boundary.
