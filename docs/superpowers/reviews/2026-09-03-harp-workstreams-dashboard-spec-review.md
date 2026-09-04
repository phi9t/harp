# Harp Workstreams dashboard final specification review

## Review identity

- Reviewed commit: `2b97af433de36a4ba5f3cda75f2d51abe2f82abd`
- Reviewed tree: `c30c45af45ae1c21ab10adc0ab6ac58abe0d03e5`
- Base commit: `91e57411939c7a19ac1e694aaaab1f9f3a8b244a`
- Base tree: `85add42212e20d20ee50d2c1683c58536c916679`
- Review date: 2026-09-04
- Reviewer: Independent specification reviewer / TRAE subagent
- Verdict: PASS

There are zero Critical findings and zero Important findings.

## Scope

The cumulative diff adds the offline Workstreams dashboard, its exact
`#workstreams` route, typed repository-reference snapshot, local session queue,
responsive and accessible presentation, offline export guards, regenerated
single-file Atlas artifact, and documentation. It also contains the bounded
XDG Lean-cache and proof-evidence repairs required to run the final repository
gate without hydrating common dependencies.

The candidate does not change Lean theorem sources or proof claims.

## Requirement evidence

### Dashboard and reader boundary

- `atlas/src/app/routes.ts` parses and formats only the exact Workstreams route.
- `atlas/src/app/AtlasApp.tsx` selects the dedicated console before constructing
  reader-only state, while all previous reader routes retain their existing shell.
- `atlas/src/content/workstreams.ts` validates the static snapshot at its
  boundary, including identifiers, timestamps, statuses, stage invariants,
  artifact sets, and watchlist references.
- The dashboard provides the required focus strip, workstream cards, velocity
  panel, insights, watchlist, search, keyboard shortcuts, settings disclosure,
  and bounded tab-local command queue. Command text is stored only; the browser
  receives no process or network capability.

### Offline, responsive, and accessible behavior

- `atlas/tests/static-export.test.mjs` is the primary offline guardrail. It
  rejects external resource URLs, CSS URL assets, browser network APIs,
  `sendBeacon`, and remote dynamic imports in the bounded production surface.
- The checked-in single-file export and its receipt agree. Browser checks at
  1440, 1024, and 390 pixels found no page-level horizontal overflow and zero
  Axe violations. The offline `file://` run made no HTTP, HTTPS, XHR, or fetch
  request.
- The Workstreams stylesheet is scoped under `.workstreams-shell`, uses
  explicit non-color status labels, meets the approved desktop and mobile target
  sizes, and contains the required reduced-motion and print behavior.

### XDG Lean cache boundary

- `scripts/harp_xdg_env.sh`, `mise.toml`, and
  `scripts/check_lean_library.sh` derive and validate the pinned XDG cache and
  toolchain without hydrating dependencies during normal proof gates.
- Python and Rust route validators accept only the exact absolute
  `.lake/packages` link to the configured XDG packages directory. Arbitrary and
  nested links remain rejected.
- The route cache reader binds the selected cache root by device and inode,
  rechecks identity after validation, and verifies the identity on the same
  no-follow-opened descriptor used for descendant reads.

### LS historical and current evidence boundary

- Immutable LS v2 receipts retain their recorded historical wrapper digest and
  remain self-consistent evidence after an infrastructure-only wrapper change.
- New LS graph promotion explicitly requests current-wrapper validation, so a
  stale receipt cannot become newly promoted authority.
- Python and Rust preserve the same boundary. The local aggregate bundle
  independently binds the current wrapper bytes, current source closure, pinned
  toolchain, dependency manifest, required Mathlib artifacts, stdout, and axiom
  audits.

### Refreshed proof evidence

- `evidence/crouzeix_conjecture/local_formalization/manifest.tsv` contains one
  header and six passed rows covering two Harp, two Jin, and two
  Lorist--Schwenninger declarations.
- The refresh changed only `build/command.json`, `build/stdout.log`, and
  `manifest.tsv`; source modules, theorem declarations, route receipts, reviews,
  provider reports, and axiom outputs are unchanged.
- The refreshed bundle passed both the Python local-bundle validator and the
  Rust source verifier. All Jin, Lorist--Schwenninger, and Harp route validators
  report `complete-local`.

### Historical landing evidence

- Goal validation treats only the CPFR-088 local-formalization manifest and the
  CPFR-089 Atlas receipt as evolving artifacts whose immutable landed bytes may
  be recovered from their recorded landing commits. This is a deliberate
  two-phase exception, not a generic historical-evidence escape hatch.
- The fallback still requires the landing commit to exist and be an ancestor of
  `master`, reads the exact registered path, and recomputes the ledger-recorded
  SHA-256 digest before acceptance.
- Ordinary Git commands retain the 8 KiB output cap. Only the historical
  artifact `git show` call uses the existing 1 MiB artifact limit. The regression
  covers a historical CPFR-088 manifest larger than 8 KiB and simultaneous
  current-byte drift in both CPFR-088 and CPFR-089.

## Findings

### Critical

None.

### Important

None.

### Minor

1. The dashboard is intentionally a repository reference snapshot, not live
   monitoring or a production control surface. Future live integration requires
   a separate authority and threat-model review.
2. macOS proof execution in this closeout is a cached local compile, not a
   hermetic Seatbelt-backed proof run. The evidence and review language preserves
   that distinction.
3. Historical landed-byte recovery is intentionally hard-coded to CPFR-088 and
   CPFR-089. Any future evolving landing artifact requires a separate explicit
   policy and review.

## Commands and evidence reviewed

- `git rev-parse HEAD` and `git rev-parse HEAD^{tree}`
- `git diff --name-status 91e57411939c7a19ac1e694aaaab1f9f3a8b244a..2b97af433de36a4ba5f3cda75f2d51abe2f82abd`
- `node atlas/tests/static-export.test.mjs` — 4/4 passed
- `sh scripts/test_harp_xdg_env.sh` — passed
- `sh scripts/test_lean_task_graph.sh` — passed
- `cargo test -p harp --test lean_library -- --test-threads=1` — 23/23 passed
- Focused review of `62ba0200` — spec compliant; no assertion or production-policy weakening
- Atlas gate — 16 files and 116 tests passed
- Python cache and receipt suites — 228 tests passed
- Python route-validation suite — 62 tests passed, one documented skip
- Python route-publication suite — 35 tests passed
- Rust Crouzeix suite — 107 tests passed; formatting and clippy passed
- `proof_evidence.py validate --route all` — all three routes `complete-local`
- Python local-formalization bundle validation — passed
- `cargo run -q -p harp -- sources verify` — 481 evidence artifacts and 189 snapshot files verified
- Goal-validation module — 27/27 passed, including the real complete-goal CLI,
  read-only execution, default Git bounds, and oversized historical manifest
- Browser audits at 1440, 1024, and 390 pixels and offline `file://` reload

## Final assessment

Candidate `2b97af433de36a4ba5f3cda75f2d51abe2f82abd` is specification-compliant and
ready for the final quality/security review and repository gate.
