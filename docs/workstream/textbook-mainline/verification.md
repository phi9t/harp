# Integration evidence

Candidate branch: `codex/textbook-mainline`, based on `35bfda85`.
Mathematical baseline: `0424501a`, with the PDF exporter derived from `afe523a2`.
This record describes integration milestones, some preceding the final provider
specialization. It is not final acceptance: see `landing-blocker.md`. Final
aggregate gate and landing commit evidence belongs to Kata issue `azr7`.

## Verified during integration

- The configured pinned Lean environment passed its non-hydrating preflight.
  The candidate uses the primary checkout's existing machine-local cache.
- `lean-crouzeix-textbook` completed 3713 jobs. The publication command produced
  six ledgers from a fresh compiler receipt, and its check matched canonical
  inputs. This is local compilation, not an independent mathematical review.
- All 132 Atlas tests and all 14 PDF-tool tests passed.
- The publication task regression exercised different built and cache binaries;
  publish and check both selected the freshly built executable.
- Independent Standards and Spec reviews reported no remaining blocking findings
  after the environment, publication binary and compiler freshness repairs.
- Both PDFs were built with `--staged`. The all-page checker verified 423 book
  pages and 14 Chapter 1 pages, 117 source snapshots, 315 relative source links,
  39 contents destinations and zero outside-page glyphs. The output manifest
  records exact candidate hashes, dependency versions and sampled rendered fonts.
- Rendered cover, chapter openings and square-bracket matrices were inspected.
  Chapter 33 web layouts at 390, 768 and 1440 pixels had no document overflow
  or KaTeX errors. Keyboard traversal exposed a visibly focused skip link;
  720-pixel reflow also had no overflow. Reflow is not a claim that every browser
  zoom implementation was tested.

## Integration repairs

- Added the approved immutable evidence generation selector and refresh command.
  Both independent review axes found the first-refresh cleanup defect; the
  repaired publisher removes only its own empty container before commit and
  preserves all visible evidence. Both reviewers cleared the repair.
- Fresh extension checks passed: 59 Python local-evidence tests, eight Rust
  generation-selector tests, 49 CLI tests and 28 goal-validation tests.
- Repeated textbook publication and Atlas verification passed. The PDF checker
  again matched all 117 source snapshots and found zero outside-page glyphs.
- Corrected the operational README's original-manifest digest against the
  unchanged Git blob. The evidence bytes themselves were not changed.
- Real refresh completed and selected generation
  `bddea3779aa247488755e0889cb53649`, bundle digest
  `7bb428a8386a4313777058baf7b2d1143e31994da5eaa38925ce453d2c5d3668`.
  All six captured declaration audits contain only the allowed axioms
  `Classical.choice`, `Quot.sound` and `propext`. The catalog retains the
  unchanged legacy bundle, digest
  `d89eeb4891f82ee2aab700bd859dd536fc6d2a1dfa3eae9e885edb18c0247a05`.

- Kept mainline XDG tool selection instead of restoring temporary Lean paths.
- Required a successful canonical wrapper build before Wave 4 compiler probes;
  failure is propagated. Removed invalid timestamp-only freshness assumptions.
- Updated Jin's import roster to include mainline's existing RationalBridge
  module: 66 modules. LS and Harp rosters are unchanged.
- Corrected the source-graph locator to line 1 of the existing compact JSON.
- Regenerated corpus data from canonical inputs; the final gate regenerates and
  checks the paired static HTML export.
- Wired the publication binary-selection regression into `test-shell`.

## Full-gate repair pass

The first complete textbook test run finished with 287 passing and five failing
tests. It exposed stale receipt prose, two stale Harp spec locators, the old
synthetic bundle fingerprint, and an actual common-module import violation.
These failures were not waived.

Sharpness, QNumericalRange and QScaledRational now import HolomorphicConsequences
directly. Their rational bounds and Chapter 29's upper-bound step use the same
proved common theorem instead of a terminal-route alias. All public theorem
statements are unchanged. Independent review confirmed that the certified Jin,
LS and Harp import closures are unchanged. The common closure has 66 modules,
digest `f3b82cd7a98454f1ff6d388914e5dde343593fda5f2f71c87b3a8ed4886f344b`.

The repaired textbook compiled and published six ledgers. Its fresh receipt has
383 declarations, 438575 serialized bytes, digest
`a7bd7e504743c4fc7936d5a0b1d618f37581f249fd52afb428ff77b4c2e7e5a3`.
The maintained metadata and Chapter 29's provider explanation now match it.
The focused common-import, provenance and receipt-prose regressions pass.
Both PDFs were rebuilt from staged sources, and all 117 snapshot hashes were
also compared directly with current candidate bytes. Final aggregate gate and
landing evidence remains external to this pre-landing record in Kata azr7.

The post-repair capture selected generation
`0bb8c67c821f4e56b6a6b60e0e8bfe12`, bundle digest
`49f431ada766d461bbdb29692d0fbe003d1e5ba8ffc1631e006c2f5634226aa1`.
The legacy bundle and the first capture remain catalogued and unchanged.
The rebuilt 423-page book and 14-page Chapter 1 edition pass all-page checks
with 316 relative source links, 39 contents destinations and zero outside-page
glyphs. Changed pages passed independent visual review and a lead spot-check.

## Remaining scope

The baseline remains a working draft: 35 chapters and 210 coverage rows do not
mean complete theorem-level correspondence. Whole-book completion remains open
under `00zs`. Uncommitted Chapter 1/13 and Wave 5 work is excluded and preserved.
PDF output is local and ignored; this landing does not publish a downloadable
artifact bundle or resolve distribution licensing. No dependencies were hydrated.
No mainline or remote success is asserted by this pre-landing record.
