# Kernel16 final review disposition

Candidate source commit read: `1f164156261a3aa927c823919bf73d6e6d11a771`, with uncommitted candidate identified by manifest SHA-256 `d1e426646b281154aa26590142b5ac3a2da80f71334dab2d9c4b88daff56ed54`. Root and both independent reviewers validated all 48 frozen file hashes. This candidate is not accepted or landed.

## Standards

Independent reviewer: kernel16_standards_round2, final round-three invocation.
Verdict: PASS, no blocking findings. Complete 29-file baseline change reviewed; 21 files retained previously reviewed bytes and eight received fresh review. Exclusive physical reservation binding now covers both allocation routes, launch and exact release; migration adopts existing legacy bindings. Registered Stop evidence and terminal task dispatch fences were also assessed. No independent dynamic upgrade fixture was executed; historical unexplained EACCES safe quarantine remains an availability limitation.

## Spec

Independent clean-context reviewer: kernel16_spec_round3.
Verdict: CHANGES REQUIRED, one P2 blocker.

Kernel::status at src/kernel.rs:255 selects only episode_runtime_operations. Genuine first-Worker operations in gateway_operations never appear; both CLI status and HTTP status use this view. Existing assembled_vertical.rs legacy fixtures produce outcome_unknown Start or confirmed Start with unknown usage, stopped workspace, refused release and retained capacity. Without an episode runtime journal, an empty unresolved list and more_operations=false are inferred from the query. The exact usage command requires a remembered operation UUID and therefore cannot replace fresh status discovery. Ticket16 explicitly requires status to report unresolved operations and uncertainty to remain visible.

This is source-based inference backed by existing genuine fixtures, not a newly executed status regression. Prior borrowed Stop, terminal dispatch and reservation blockers appear repaired.

## Verification and next boundary

Final full gate: 123 passed, 4 ignored child-process fixtures, 16 groups, zero filtered; wrapper exit 0. Log SHA-256 `10994ca495c27760ac8dc1a30bc4df1218805bd77b7fa4e89d32f931c14fc6f2`. Strict lint, fmt and diff checks passed. These results do not supersede the Spec blocker or prove live Trae recovery.

Three review iterations are exhausted. Root requested one bounded additional round: expose authorized bounded legacy unresolved records, exercise genuine unknown-start/unknown-usage plus access and pagination controls, rerun gates and independent review. Owner approval is pending. No repair, acceptance or landing is implied by this proposed scope.
