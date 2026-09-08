# Foundations core execution record

Base: `4a72888fd9b09d86e958da35074e86c08615a257`.
Branch: `codex/harp-mathematics-spec`.
Status: approved foundations core accepted locally after source reviews,
publication and presentation checks, and the full repository gate.

The user authorized autonomous end-to-end completion, including the
prerequisite repairs previously awaiting direction. This execution keeps the
existing feature worktree and leaves merge, push, deployment and external
reviewer outreach out of scope. No Lean dependency update or cache hydration
is authorized or needed.

## Accepted prerequisites

The scoped foundations contract preserves all 72 card and exercise identities
for Chapters 1–12. It accepts a four-chapter core separately from the full
foundations wave. Its fixture tests cover omitted editorial fields, repeated
paragraphs, nested theorem headings, adjacent bold titles and statement labels,
and isolation from the following card's proof. Specification and quality
reviews repaired the two parsing defects and preserved CFT-01-006's existing
worked-example kind. Integrated chapter assertions remain required below.

Chapter 1's arbitrary-basis statements replace the old standard-coordinate
or definitional mappings without deleting the old public helper declarations.
Its norm counterexample now includes sharp uniform squared bounds and
attainment. Both independent reviews passed, and both the complete source
compile and narrow Lake build passed. Its characteristic-polynomial statement
is an explicit preview, so the older whole-chapter issue remains open.
See `chapter01-review.md` for the exact mathematical and execution boundaries.

## Test-first integration

The new compiler-backed Chapters 2–4 test first failed with
`CFT-02-E01 needs a distinct solution`, exit 101. It will require all 18
solution declarations to be theorems, have distinct compiler fingerprints,
and differ from all indexed card statements. It separately checks exact
correspondence, the invariant-restriction definition, and the three Chapter 4
summary previews. Existing publisher fixtures cover stale positions, hashes,
missing assumptions, provider ownership and dependency cycles.

The new PDF rendering test passed Chapter 1 and failed Chapters 2–4 because
their named solution references were missing. It uses the actual canonical
Markdown and maintained renderer. PDF tests require the bundled Node package
path; an initial invocation without it failed to resolve `marked`, and the
corrected six existing edition/supplement tests passed. This was an environment
selection repair, not a package installation or a waived rendering test.

Four website reader tests initially failed against the old generated corpus,
each at a missing named solution reference. They exercise the actual chapter
reader, check each retained card and exercise anchor exactly once, reject
formula-rendering errors, and require the matching Lean chapter source link.
They must pass after the final canonical corpus regeneration.

The fake all-import cache fixture initially rejected the new Chapter 3
`Basis.VectorSpace` and `Isomorphisms` imports before invoking Lake. The
fixture now includes those two synthetic artifacts. Real dependency artifacts
were already present and were not downloaded or changed. This is a test
roster update, not a relaxation of the warm-cache preflight.
The repaired `lean_library` suite passed all 27 tests, including the three
cases that had rejected the incomplete synthetic roster.

## Accepted integration

- Chapters 1–4 passed separate specification and quality reviews and direct
  pinned Lean compilation. The Chapter 4 terminology-only refinement was
  rechecked by its quality reviewer. All 25 PDF rendering/input tests passed.
- The maintained Markdown validator's narrative-format repair passed both
  review stages, including repairs for quote impersonation, malformed provider
  identifiers and interrupted labels. Its final 14 tests and eight independent
  reproduction/control cases passed.
- The official maintained exporter produced 419 declarations. All 312
  registered card and solution metadata rows match that receipt, including
  source positions, type fingerprints and axiom lists. Correspondence was
  regenerated through the existing Rust generator; no receipt was fabricated.
- Publisher and read-only publication checks passed. The six-ledger generation
  is `618f0a2e7e86ef56163f068a6b7d51dbf7e76c4de1813d3f745a050b48c67547`.
  Canonical corpus and offline Atlas exports were regenerated together.
- Actual contract totals are 96 exact correspondences, 89 reconstructible
  proofs, 125 summaries and two not-applicable definition rows. There are
  96 solved exercises and 120 unresolved exercises. Correspondence has 45
  checkpoints and 75 unmapped rows; these actual counts replace estimates.
- Sources verification passed for 481 evidence artifacts and 189 snapshots.
  Selected terminal evidence `9d25d24ee30c404c9b6b6e226c81d1ca` still binds
  current inputs. No refresh or historical evidence rewrite was needed.

The official receipt has SHA-256
`a7b248d229675d5464bf5be7aad64a049228bce3394e7ae3495a2172109166c1`,
492763 serialized bytes and 419 declarations: 216 public card declarations,
96 solution declarations and 107 additional underlying providers. Its temporary
execution location was `/tmp/harp-foundations-receipt.Cz3zwt/receipt.json`;
the durable identity is recorded in the canonical status surface.

## Presentation acceptance

The three staged-source PDF editions contain 487, 19 and 48 pages. All 25
PDF tests passed; the artifact checker validated 40 canonical source files,
145 linked snapshots, all output hashes, contents destinations and source
links, with zero out-of-page glyphs. The candidate snapshot digest is
`d15fa0f79e34bb9dc496783af3bce044aed31e962a73bdd46652f1ce014b4c22`.
The generated build manifest records exact package, runtime and output identities.

The parent visually inspected 13 pages: book pages 1, 4, 12, 20, 29, 31, 40,
43 and 479; the Chapter 1 cover; and Harp pages 1, 20 and 45. Matrices,
long formulas, proof/provider paragraphs, contents and ML boundaries were
readable without visible overlaps or clipping. This is sampled visual review,
not inspection of all 554 pages. The PDF rasterizer emitted font-configuration
warnings; the inspected rendered pages retained the expected text and formulas.

Actual browser checks passed for all four foundations chapters and both Harp
supplements at widths 1280 and 390. All retained anchors and named solutions
resolved; MathML rendered without error or page-wide overflow. Source links
were checked against candidate files, not claimed as remotely published.
The parent inspected four foundations screenshots and two Harp screenshots.

An existing website test initially hit jsdom's unsupported computed-style
handling for MathML in another heading. The assertion now selects the plain
Opening problem `h2` directly and still checks visibility. All 137 Atlas tests
passed, and the actual browser checks remained the rendering authority.

## Full gate and independent review

The frozen candidate passed `mise run verify` with aggregate exit 0 on
2026-09-07 local time (2026-09-08 UTC). It passed 137 Atlas tests, 502 Rust
library tests, 13 CLI tests, all 296 textbook tests, nine scoped contract
tests, 14 narrative tests and 27 Lean wrapper tests, plus the remaining
workspace and shell suites. The proof-reproduction suite ran 865 tests:
863 passed, with two existing conditional skips. Those were the optional
verified-temporary-Jin-source comparison (source absent) and the absent-route
CLI fixture (the canonical Jin route is already present). Neither is an
executed check or a new waiver.

The aggregate Lean build passed all 8831 jobs in 159 seconds. Git LFS fsck
and final repository verification passed. The execution log was
`/tmp/harp-foundations-full-verify.log`; it is a temporary log, not an immutable
proof authority. Only acceptance records and the self-referential payload
receipt were updated afterward; repository verification is rerun for those
final documentation bytes.

An independent integration reviewer found no critical or important findings.
They reran all 23 scoped/parser tests, checked all 312 registered declaration
records against the official receipt, and verified all three PDF hashes,
40 source hashes, 145 snapshots against canonical and staged bytes, and the
Atlas corpus, HTML and 58-file app-input receipt. Existing public declarations
and identities remain, and the Jin, LS and Harp terminal proof sources are
unchanged.

This execution is local and non-hermetic. Compilation and agent review do not
establish independent human peer review. The previous Jin, LS and Harp terminal
proofs are not being changed by this foundations package.

## Concurrent cache ownership

A separate worktree's full gate overwrote the shared Chapter 1 and Chapter 2
build artifacts after our narrow builds passed. Its source paths appeared in
the canonical Lake trace, and a subsequent metadata probe reported a missing
new Chapter 1 declaration. Source hashes remained unchanged. The failed probe
was not accepted as compiler evidence or used to update the contracts.

For iteration, direct Lean compiles write only to a private temporary output
overlay and read the existing pinned dependencies. The canonical `.lake`
symlink is unchanged. The other task was notified; final shared-output builds
and publication must be serialized after its current gate. This avoids
changing another task's files or interrupting its verification.

The contributor guide now records this shared-output hazard and the bounded
private-output option. Dependency caching remains canonical; isolated compiler
outputs are temporary iteration artifacts, not another dependency cache.
