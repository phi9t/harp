# Kernel16 review7 — changes required

Frozen manifest SHA-256: `1a225de7e3e73444f26c40517713994cc39579ebb7316d58c1c9a5633dc2231f`; base `1f164156261a3aa927c823919bf73d6e6d11a771`. Root independently verified all48 source hashes, 31 baseline changes and exact author matrix totals: 133 passed, 4 ignored, 16 unfiltered groups, wrapper0 and Temporal cleanup. Strict all-target Clippy passed.

Standards and Spec independently identified one shared P2 evidence finding: `cancelled_sdk_drain_phase` recursively accepts matching base64 JSON from arbitrary history data. Coverage claims it is restricted to a completed Activity result. The frozen code does not implement that restriction. Standards cites CONSTITUTION section5 and evidence-bound acceptance discipline; Spec cites real Temporal lifecycle/replay and public-seam proof requirements.

Both reviewers found no additional actionable product defect in the whole baseline-relative change. Source inspection supports exact bounded repair authorization, immutable verifier admission provenance, monitor-result-before-release, and authenticated historical gateway observation with fenced pending effects. Their review does not waive the oracle finding.

Retained actual histories contain matching progress at `events/6/attributes/ActivityTaskCompletedEventAttributes/result/payloads/0`. Observed completion is supported; the finding concerns what the regression enforces, not proof of a production failure. A narrow predicate correction, fresh relevant execution, updated frozen manifest and independent follow-up review remain required before acceptance.

Controlled late-completing pre-failure verifier and mixed-check physical fixtures remain unexercised. Admission-based freshness is assessed by source inspection and actual old-report refusal/new-verifier positive controls. Later live Trae, lead demonstrations, restore, Kata/jj phases remain incomplete.
