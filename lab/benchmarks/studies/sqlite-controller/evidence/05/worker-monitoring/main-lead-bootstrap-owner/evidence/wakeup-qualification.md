# Bootstrap main-lead wake qualification

Qualified source: b034844ef108580ad9ba3b3d05b77bb859a18077, base b5d901b.
Installed helper SHA256:501e060e1ce1892e0da3755eb6b6ea4b4812dae420e81d9847828c68733129ae.
Pinned Codex0.159.3 binary SHA256:8bf204b36a2f6dd0dab73aa2f639892e67ef9ac8befccb4a05b1496ebf25c479.

Independent Standards and Spec reviewers found no blocking defects. Standards separately ran the nine focused tests. Independent branch defect review647 returned no issues. Root FF-integrated the fixed source into its owned documentation/tool checkout, then freshly observed nine focused subprocess tests PASS, all99 agent-tool Python tests PASS, py_compile and diff check PASS. Existing worker tests emit ResourceWarnings. The broader shell gate is not claimed green: inherited SQLite-ticket formatting is invalid in this documentation fork, and its unrelated PostgreSQL gate starts Rust builds even with CHECK_CARGO=0; the author interrupted only its owned gate process. The new worker-monitoring ticket is normalized and its scoped tracker lint passes.

Root prepared private persistent observation/wake state, retained document copies, and a content-addressed installed helper. Existing main-thread identity was confirmed from CODEX_THREAD_ID plus read-only Codex thread metadata (`corenius-frontier`). No Codex state database was modified, credential copied or new model/runtime started by this helper.

Systemd user service/timer verification passed. The new unique timer was enabled and reports active/waiting. A manual start of its actual service dispatched exactly one tracked wake:

- Wake:2d80dd33-f95f-4290-a3e0-63970a1cea43.
- Exact lead:01a10084-dc3a-7743-b0c1-4694d5ecc621.
- Native queue receipt:01a10827-dc93-7881-951e-8ddbfc9e70b1.
- Status:queued, with receipt validation, not presented/reviewed/completed.
- First next timer deadline observed:2026-10-04T18:38:12UTC (11:38:12 America/Los_Angeles); cadence approximately15minutes,30-second timer accuracy.

The outstanding wake coalesces later ticks until it is claimed/completed or explicitly reconciled. The one-shot delivery probe is a diagnostic request, not an additional tracked recurring wake. Automatic presentation and analysis after the current turn exits are not yet established. Local host/user-manager/runtime availability is required; this is not native hosted Codex automation.

Bootstrap limits: completion records the lead's attestation that assessments/cursors were saved; dispatch holds a writer lock for its bounded deadline; binary pinning assumes a trusted local operator. The helper does not independently evaluate worker alignment, perform controller enforcement or implement full production transcript retention/calibration. Those remain governed by SPEC.md and future qualification.
