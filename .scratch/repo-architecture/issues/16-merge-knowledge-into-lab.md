# 16: Merge maintained knowledge into lab

Status: done
Owner: Codex on codex/lab-knowledge
Parent: [Repository architecture](../spec.md)

## What to build

Remove the tracked knowledge/ directory. Place maintained math and SICP prose beside
those workloads, benchmark notes under lab/benchmarks/, and shared engineering
studies and navigation under lab/. Update all active consumers without changing
document identities, proof source bytes, corpus schema or captured upstream bytes.
Keep the durable framework reorganization outside this slice.

## Acceptance criteria

- All 136 original tracked knowledge files have a documented destination; no tracked
  top-level knowledge directory remains.
- Registered corpus compilation, search, source validation, Obsidian navigation and
  PDF publication resolve the new paths and retain their existing contracts.
- Math, SICP, licenses and captured evidence remain preserved. Existing historical
  receipts retain their original identity rather than claiming new qualification.
- Changed-surface verification and relevant consumer checks pass before landing.

## Blocked by

None.

## Verification

- Baseline: `2da4f4cf2bda80f3f9808846b5851ab7b5b215f8`. The complete-candidate
  changed-surface plan selects 28 checks after adding Lean preflight.
- All 136 original tracked knowledge files have verified original/destination
  digests in `evidence/provenance/knowledge-layout-migration.json`. The 131 corpus
  concept IDs, 11 route IDs, aliases, schema and external source URLs are retained.
- `harp check`, `harp build --check`, search refresh/query, repository verification
  (521 import rows), math relocation and imported-source preservation pass.
  Search has 136 unique indexed paths after removing overlapping topic roots.
- Rust library: 469 passed. Proof tooling: 857 passed with three existing
  conditional skips. Selector: 19 passed. Architecture: five passed. PDF renderer:
  25 passed; all 36 actual chapter bodies render and resolve their Lean modules.
- The initial `mise run verify -- --base 2da4f4cf` invocation exposed the old
  publication fixture layout, four unsmudged LFS payloads and a stale import
  receipt. The fixture was moved; local primary bytes were restored only after
  validating each LFS SHA-256 and size; the receipt was refreshed. CLI: 13 passed.
- Selected checks continued through the actual CLI, style, packet, foundations,
  durable CLI, session and documentation targets. The full publication target
  exercised all 295 cases: 289 passed initially; one required a generated
  publication and five asserted obsolete relative-path spelling. A fresh receipt
  generated six matching publication ledgers. The metadata test passed on rerun.
  The Wave 4 group then passed 27 cases; its remaining exercise locator assertion
  passed after checking the resolved compiler-bound link within that exercise.
  No mathematical or native proof-dependency audit was weakened or skipped.
- Publication checks initially lacked the worktree's warm-cache link. The isolated
  attempt and its children were stopped; the pinned primary cache, toolchain and
  Mathlib artifact were verified before linking and rerunning. The selector now
  runs `mise run lean-env` before publication tests; a regression proves that a
  failed cache preflight prevents launching the native test target.
- No Lean proof source, captured upstream Git payload, Soma source or preserved
  benchmark task was changed. Private ignored notes in the retired local vault
  remain ignored. Existing historical receipts keep their original identities.
- Check logs remain in the task worktree's ignored `.build/lab-knowledge/`.
  The original failed/stopped invocations are retained alongside scoped repair
  logs; these results are not represented as one uninterrupted successful run.
