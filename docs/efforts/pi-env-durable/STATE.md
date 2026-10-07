# Pi Durable / Pi Env research handoff

## Status

Source research and the owner-authorized interrupted-effect probe are recorded
under `docs/spikes/pi-env-durable/`. The probe passed its two recovery scenarios
and negative controls. A fresh integration-worktree reproduction and independent review passed on
2026-10-06. No product
implementation or architecture decision was made.

## Conclusion

Reuse contracts and conformance/failure tests first. Pi Durable is a candidate
for an optional reasoning worker; Pi Env is a candidate for bounded tool I/O
inside an admitted Insula. Keep Corenius's authority ledger, independent
execution supervisor, uncertain-effect reconciliation and acceptance machinery.

## Evidence

- Pi source pinned at `ae92585d3b3e5f1e4b123d14a34314d826d8d9f5`.
- Corenius comparison pinned at `d7f674829e9844d011869b711d64fcf55313f7f1`.
- Durable: 301 selected upstream tests across 11 files passed in a documented
  minimal-dependency source-test setup; raw test output retained under evidence.
- Env: 7 Rust unit tests and a build passed with Rust 1.95 (upstream pins 1.96).
  Protocol/client probes exercised duplicate request IDs, EOF, daemon death,
  escaped descendants, reconnect and stale handles. Scripts and observed outputs
  are embedded in the source review; raw Env tool logs were not saved separately.
- Independent read-only source reviewer found no material inaccuracies in the
  three reports. Out-of-range citation endpoints were corrected before final
  validation. This was research review, not a product security qualification.
- Interrupted-effect probe: two real kill/restart cases passed, including
  revoked authority. Negative controls confirmed payload refusal and fresh-key
  duplication. All six child processes and both proxy threads stopped; private
  fixture directories were removed. Source and sanitized results are retained
  under `experiments/pi-env-durable/` and `evidence/interrupted-effect-probe/`.
- Final file, link, citation and tracker checks: `evidence/document-checks.json`.

## Retained workspaces

- Integrated result: branch `codex/pi-env-deep-dive`, worktree
  `/data02/home/philip.yang/workspace/corenius-codex-pi-env-deep-dive`.
- Env author: `research/pi-env`, worktree
  `/data02/home/philip.yang/workspace/corenius-research-pi-env`.
- Durable author: `research/pi-durable`, worktree
  `/data02/home/philip.yang/workspace/corenius-research-pi-durable`.
- Probe author: `research/pi-effect-probe`, worktree
  `/data02/home/philip.yang/workspace/corenius-research-pi-effect-probe`.

All four start at the Corenius comparison commit. Research drafts/builds remain
retained in the author worktrees; the integrated documents are authoritative for
this spike. The upstream source clone under `.scratch/pi-research/upstream` can
be regenerated from the pinned commit. No active probe workload is retained.

## Git and lifecycle boundary

The owner authorized the branch/worktree and research, not commits, pushes or
landing. New documents remain uncommitted. This is a manually written handoff:
`effort.py dehydrate` was not run because it commits and refuses dirty worktrees;
it has no non-committing mode. Do not interpret this file as a generated clean
checkpoint or resume historical implementation authority from other efforts.

## Remaining uncertainty

A disposable Pi/controller adapter probe was built; no production integration
was built. The effect was a transactional progress event. External execution
recovery and Pi Env confinement remain untested by this probe. No full upstream test/build, live model,
external SSH, cross-platform, power-loss, performance or production qualification
was performed. Both packages are recent and Durable is explicitly experimental;
main includes unreleased changes beyond the version shown in package.json.
