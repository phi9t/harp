# Final ticket05 Spec review — FAIL

SOURCE `92429710c422075332f9d5d4f7a5eaa9b583da2b`, tree `917018ee0d6fe5cd35847ea569cf37894454e229`; base `291f04a5abd9274e4fd0034fd36927b0a4c4d236`. Evidence TIP `0fccc193949476e9b7f04da1b0c9513f3d0a055c`, tree `46d03f269edbaeeb41287c8454d7d682d3f9edd2`. All43 source hashes and64 committed implementation files verified.

- **P1 — Atomic delivery remains incomplete.** `src/codex.rs:1450,1474` commits `applying` before `fail_delivery`. A later SQLite failure rolls back only the helper transaction, leaving operation`applying`/question`prepared`; `src/control.rs:223` blocks reprepare. SPEC78 requires “A turn race returns delivery to pending.” Exact caller inspection and the retained independent Standards dispatcher/reopen probe confirm this. Move local rejection before applying admission or provide explicit recovery; test the real caller, failed second write and reopen. The passing helper test does not cover this boundary.
- **P2 — Missing full inputs receive reviewed coverage.** `src/monitor.rs:887–924` validates report evidence, but omits sealed native-input dependencies before cursor publication at715–720. Owner SPEC112 requires complete retained inputs before review credit. My test-only fixture retained a40KB item, sealed its placeholder, removed its full artifact and observed `monitor_input` refusal; publication nevertheless succeeded, advancing history0→2 with backlog0/gapnull. Actual negative test failed as intended. Validate complete dependencies and preserve pending/explicitly limited coverage when unavailable.

Original findings: observer authority, turn admission, peer routing and pending-argument replay are resolved in scoped checks. Atomic delivery remains open; bounded descriptor reads are repaired, but their publication acceptance control remains incomplete.

Independent results:14 unit+7 control+2 client tests pass; original observer probe and exhaustive Git authority/authorized-effect controls pass; combined actual native shared/separate UI, routing, question and monitoring/correction test passes108.80s. **Original default INITIAL_CONFIRM fails**: shared one-shot turn/clientId assertion, exit101/54.77s; separate recovery passed its assertions. Both raw archives retained. Cause/05 attribution remains unproved.

All owned process identities are terminal. Preserve gate36 failures/archive gap,52 FIRST-read limitation,28 source drift,26 binary collision and historical cleanup distinctions. No02/04 cap reset or acceptance. Model-free proof does not satisfy06 live/recurring-harness requirements. Root’s full gates, branch review and final acceptance remain separate.

`spec.json` contains six dispositions and exact evidence; `evidence-index.json` pins reports, tests, source and raw archives.
