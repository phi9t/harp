# 15: Finish the repository layout around component ownership

Status: done
Owner: Codex on codex/owned-repo-layout
Parent: [Repository architecture](../spec.md)

## What to build

Move remaining root benchmarks into lab, remove unused RSI web assets, eliminate
content/ by relocating active structured contracts beside their owning components,
and replace crates/ with owned CLI/session components and preserved legacy runtime
benchmark sources. Explain evidence's provenance role. Preserve math, SICP, PDF,
benchmarks, licenses and frozen historical evidence.

## Acceptance criteria

- [x] No top-level benchmarks/, assets/, content/ or crates/ remains.
- [x] Existing structured contracts and generated corpus retain their useful behavior under owned paths; no proof or publication data is discarded.
- [x] CLI and session-index builds resolve from their new homes. Retired Rust execution code has no active product dependency; retained benchmark sources remain buildable in their own workload workspace.
- [x] Current links, selectors, Cargo manifests, tests and provenance agree with the new paths; captured upstream evidence stays byte-identical.
- [x] Relevant selected checks, fresh runtime source-bound qualification where required, and independent review pass before landing.

## Blocked by

None. The previous existing-code migration is complete.

## Verification

Inspect consumers before moving each family. Verify actual CLI/corpus/PDF and
workload behavior, Cargo metadata and builds, source-preservation identities,
private-host exclusion, and the complete-candidate changed-surface plan. Preserve
old qualification identities as historical records; do not relabel them.

## Completion

Implemented in `ec08322f1989489acee918a594581af2e38ebc53`. All 45 complete-candidate checks passed; the
[qualification record](../../../evidence/provenance/owned-layout-qualification.json)
binds the fresh Linux checks to 934 exact source files and records the scoped
repairs, independent review and verification limits. All 252 original root files
have an explicit destination or reviewed unused-RSI retirement.
