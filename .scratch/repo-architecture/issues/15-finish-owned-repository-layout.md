# 15: Finish the repository layout around component ownership

Status: in-progress
Owner: Codex on codex/owned-repo-layout
Parent: [Repository architecture](../spec.md)

## What to build

Move remaining root benchmarks into lab, remove unused RSI web assets, eliminate
content/ by relocating active structured contracts beside their owning components,
and replace crates/ with owned CLI/session components and preserved legacy runtime
benchmark sources. Explain evidence's provenance role. Preserve math, SICP, PDF,
benchmarks, licenses and frozen historical evidence.

## Acceptance criteria

- [ ] No top-level benchmarks/, assets/, content/ or crates/ remains.
- [ ] Existing structured contracts and generated corpus retain their useful behavior under owned paths; no proof or publication data is discarded.
- [ ] CLI and session-index builds resolve from their new homes. Retired Rust execution code has no active product dependency; retained benchmark sources remain buildable in their own workload workspace.
- [ ] Current links, selectors, Cargo manifests, tests and provenance agree with the new paths; captured upstream evidence stays byte-identical.
- [ ] Relevant selected checks, fresh runtime source-bound qualification where required, and independent review pass before landing.

## Blocked by

None. The previous existing-code migration is complete.

## Verification

Inspect consumers before moving each family. Verify actual CLI/corpus/PDF and
workload behavior, Cargo metadata and builds, source-preservation identities,
private-host exclusion, and the complete-candidate changed-surface plan. Preserve
old qualification identities as historical records; do not relabel them.
