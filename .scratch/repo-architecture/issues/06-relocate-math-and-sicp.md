# 06: Put retained executable mathematics and SICP under lab

Status: in-progress
Owner: math migration agent (`codex/math-migration`)
Parent: [Repository architecture](../spec.md)

## What to build

Move formalization and proof companion programs into lab/math/ and the evaluator/executable education into lab/sicp/. Keep shared knowledge, contracts and captured evidence canonical, linked through explicit workload inputs. Update proof routes, Cargo membership, publication paths and documentation without changing mathematical claims.

## Acceptance criteria

- [x] Existing Crouzeix theorem and route identities remain authoritative, with fresh relevant Lean build/receipt evidence.
- [x] SICP evaluator behavior and existing educational consumers continue to work after Cargo/path migration.
- [x] The PDF publication mechanism builds retained source through the new proof/program paths.
- [ ] Move canonical cache ownership deliberately; verify the existing warm-cache target and serialize shared Harp outputs. Never hydrate dependencies incidentally.
- [x] No duplicate proof/source authorities or stale managed links; regenerated corpus agrees with canonical inputs.

## Blocked by

- [01-unify-durable-control-plane](01-unify-durable-control-plane.md)

## Verification

Preflight the pinned toolchain and existing cache without Lake, then run relevant Lean route/receipt checks, evaluator tests, managed link/corpus checks and PDF consumer checks. Use the selected gate, adding consumers affected by moved paths.

Implemented on `codex/math-migration` in its isolated worktree. Executable sources moved to `lab/math/` and `lab/sicp/`; Python tests and historical test fixtures are adjacent to their source. Cargo membership, Rust proof/publication consumers, managed knowledge links, corpus roots, PDF discovery and Lean scripts use the new paths. Proof declarations and module/route identities are unchanged. The receipt exporter changes only two source-path literals.

`lab/math/relocation.json` binds all 667 original paths to source Git objects, modes, SHA-256 identities, explicit destinations and dispositions. It preserves 611 byte-identical sources/artifacts and captures 55 reviewed adaptations; the only retirement is the old literal task-registration/retired Atlas assertion suite. Original route manifests, receipts, reviews and proof-slice commands are untouched. The executable relocation verifier rejects missing coverage, new proof inputs, symlinked sources, altered proof/evidence bytes and an unqualified source revision. Preserved source qualification is explicitly historical; it does not relabel an old proof run or review as a new one. New proof changes require fresh qualification rather than editing historical artifacts to satisfy this record.

Fresh checks:

- Lean 4.32.1: warm packages link and ancestry inspected without Lake; `check_lean_library.sh CrouzeixTextbook --receipt-output ...` built 3726 jobs and exported a new-path receipt in 47 seconds. `harp crouzeix-textbook publish` generated six ledgers; `check` passed. Derived count check: 174 exact declarations, 174 solved exercises, 513 exported declarations.
- SICP: six Rust evaluator tests passed with the pinned Rust toolchain and existing macOS 26.5 SDK. The default 27 SDK cannot be read by the configured classic linker; no toolchain changes or downloads were made.
- Lean benchmark: 42 behavioral tests passed. Crouzeix Rust source/route tests: 115 passed. Corpus Rust tests: 45 passed. Managed Obsidian link migration: 15 passed. Generated correspondence and corpus checks passed.
- PDF renderer: 25 tests passed using the pinned packages, materialized into this isolated checkout from the existing npm cache with `npm ci --ignore-scripts --offline` under the earlier pinned PDF-dependency authorization. No browser was downloaded. All three editions built through the new paths using installed Chrome; PDF inspection validated 145 source snapshots, internal destinations and links, with zero outside-page glyphs. Generated publications remain ignored.
- Colocated relocation tests: seven behavioral checks, including rejection of hardlink aliases to proof/evidence sources. Full proof suite is 855 cases; the final matrix (850 passed, two existing unavailable/state-dependent cases skipped, three failures subsequently repaired and rerun successfully) and focused repair reruns are recorded in `/tmp/harp-math-proof-final-modules/` and `/tmp/harp-math-repaired-tests.log`. The original Phase 8 goal receipt is already invalid on the unchanged primary; its old current-tree green assertion was retired, while read-only behavior and fixture validation remain tested.

Integration still owns the selected gate and canonical cache transition. `mise run verify -- --plan` on this isolated slice refuses unmapped `lab/math/audit_fixtures/...` until the coordinator's selector/mise changes are integrated. The worktree's verified `.lake` link currently targets the primary's original `formalization/lean/.lake` directory. At operator-facing landing move that ignored canonical directory to `lab/math/lean/.lake`, then relink feature worktrees and rerun route preflight/gates. Do not copy or hydrate the dependency cache. No other Lean writer ran during the fresh receipt gate. The primary has not been changed by this ticket.

Detailed local evidence: `/tmp/harp-math-new-lean-receipt.log`, `/tmp/harp-math-new-receipt.b9YNew/receipt.json`, `/tmp/harp-math-textbook-{publish,check}.log`, `/tmp/harp-math-rust-{crouzeix,corpus}.log`, `/tmp/harp-math-pdf-{build,check}.log`, and `/tmp/harp-math-relocation-test.log`.

Final trust checks: the continuous 855-case proof suite passed (two existing expected skips) in `/tmp/harp-math-855-frozen-final.log`. Added counterexamples ensure inspected repositories cannot execute their own relocation checker through either the Rust source verifier or Python route reader. The verifier ignores Git replacement refs, pins the approved path map, and rejects hardlink aliases; nine relocation checks pass. Historical route identities remain historical; no proof bytes or original receipts were rewritten.
