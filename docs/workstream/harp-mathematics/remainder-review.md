# Finite-horizon remainder execution record

Base: 2e5d57da339d83b90bb631e0b1939958bd7369a6.
Branch: codex/harp-mathematics-spec. Kata: 5r30, related to 00zs and fq4m.
Authorized implementation date: 2026-09-07. No push, merge or deployment.

## Scalar slice

The implementation agent first compiled clients against missing declarations
and observed the expected failures. It then compiled
Crouzeix.Harp.FiniteHorizonRemainder and HarpRemainderTests successfully.
The seven helper and 21 fixture axiom closures contained only propext,
Classical.choice and Quot.sound. No terminal proof is imported or invoked.

A separate specification reviewer independently compiled both files, exit 0,
and reviewed the scalar prose. A subsequent quality reviewer inspected the
proofs, fixtures and prose. The missing-finite-premise counterexample was
strengthened to exhibit a true terminal premise and a false finite premise
and conclusion. The repaired test module compiled, and the review passed.

The signed estimate does not require a nonnegative coefficient. The absolute
estimate supplies a nonnegative budget. The convergence proof handles the
zero budget before dividing by it. The comparison corollary preserves the
old limiting interface and leaves its provider unchanged.

## One-witness application

The implementation agent observed the two missing-declaration client failures,
then compiled Crouzeix.Harp.FiniteHorizonRemainderApplication and
HarpRemainderApplicationTests, both exit 0. A separate specification reviewer
independently compiled both files, exit 0. The four printed provider/client
axiom closures contained only propext, Classical.choice and Quot.sound.
A subsequent quality reviewer inspected the code and corresponding prose,
with no findings. These are local, non-hermetic executions and agent reviews,
not independent human peer review.

The result accepts one horizon witness. Its unit-vector, norm-identification
and singular-vector hypotheses are explicit. Complete complex inner-product
spaces suffice; finite dimensionality is not required when the singular
vector is supplied. The scalar budget remains implicit in the operator norm
and first moment, not an effective operator certificate or a witness algorithm.

## Integration adjustment

The planned import into CrouzeixHarp made the historical route validator fail
with an unmapped active-closure module. That unused root import was removed:
the terminal aggregate and route manifests remain unchanged. The application
is public through its own module and imported through CrouzeixTextbook, so
the maintained textbook compilation checks all new helpers and fixtures.
The broader source inventory required a selected-evidence refresh, completed
below. No historical receipt was rewritten.

## Publication and presentation checks

Both supplements resolve through the Rust corpus compiler, with 36 chapters
and 47 registered textbook documents. The new resolution test was observed
failing before registration and passing after the real application file
existed. Indexed coverage and exercises are unchanged.

All 21 PDF unit/integration tests passed. The first compiled-artifact check
correctly rejected the old 38-source assumption. The checker now verifies
the exact 40-source set, including both supplements, before checking the
three generated PDFs. The rebuilt artifacts contain 466, 15 and 48 pages;
all page-boundary, destination, source-link and hash checks passed. A local
installed browser was used; no browser or Lean dependency was downloaded.

The parent visually inspected the Harp PDF cover, contents, expanded statement,
recurrence calculation, tolerance proof, examples and Lean-reference table,
plus the full-book supplement contents and a Chapter 1 coordinate calculation.
Those sampled pages have readable formulas and no visible overlaps. This is
sampled visual evidence, not a claim to have inspected all 529 pages.

Website integration initially failed three stale 45-document expectations;
the real corpus now contains 47. The corrected three focused suites passed
all 27 tests, including rendering every registered textbook formula.

The final PDF build also passed staged-source validation. Its candidate
snapshot digest is c9d2dff6fc35f2f2a70f4268a196594bb8cdc8d7823d867b4b9c5cb6a448d190;
the three PDF digests and runtime versions are in the accompanying generated
build-manifest.json. The final checker passed again. The final application
page was rendered and visually inspected after that build.

The actual exported reader passed browser checks at widths 1280 and 390:
Chapter 36 links opened both supplements; 194 audit formulas and 116 remainder
formulas rendered without fallback or page-wide overflow. All 28 audit and
seven remainder source links resolved to candidate files. The parent inspected
desktop and phone screenshots. GitHub availability is not claimed: new sources
remain local until an authorized publication.

A fresh integration reviewer found no blocking source findings and confirmed
unchanged terminal proofs and coverage/exercise contracts. The full gate's
first run passed all 133 Atlas tests and 502 Rust library tests, then correctly
failed the two CLI evidence checks while the selected-bundle refresh and final
payload receipt were pending. That run is not a full-gate pass.

The second run passed the CLI checks after receipt refresh and completed 293
of 294 textbook tests. Its sole failure was the exact support-document roster:
the allowlist omitted the two new supplement names. The allowlist now contains
those exact names in addition to the nine existing support documents; the
36-chapter and 216-row assertions remain unchanged. This run also is not a
full-gate pass.

The maintained refresh subsequently exited 0 with status published-selected,
selecting immutable generation 9d25d24ee30c404c9b6b6e226c81d1ca. Its six rows
bind fresh local aggregate-build and axiom evidence to the three existing
terminal declarations and three closed-range consequences. Historical bundles
and route records were preserved; the status is local selection, not GitHub
publication or human review.

## Accepted local candidate

The final mise run verify completed with aggregate exit 0 on 2026-09-07.
It passed 133 Atlas tests, 502 Rust library tests, all 13 CLI tests and all
294 textbook tests, together with the remaining workspace and shell suites.
The proof-reproduction suite ran 865 tests: 863 passed and two were skipped
by existing environment/state conditions. The skips were the optional
verified-temporary-Jin-source comparison (source absent) and an absent-route
CLI fixture (the canonical Jin route is already present). They are not new
waivers or executed checks. The aggregate Lean target passed 8831 jobs in
70 seconds; Git LFS fsck and final repository verification passed.

The final gate log was /tmp/harp-math-full-verify-accepted.log. This is a
temporary execution log, not an immutable authority. The selected proof bundle
and delivered PDF source snapshots are retained separately. Proof execution
was local and non-hermetic; no human review or stronger operator result is claimed.

The final integration reviewer independently recomputed the selected bundle's
22-member digest, all 54 manifest-linked hashes across six passed rows, all
145 PDF snapshots against candidate files, all three PDF hashes, and the
Atlas HTML/corpus receipt hashes. They matched. The exact support-roster repair
was separately reviewed and its focused test passed before the final full gate.

Reviewed remainder source SHA-256 values:

- FiniteHorizonRemainder.lean: f164196a4f2a8df2b0118cd0adc2ea23c28f1e425f14b774f00e7471a662cd06
- FiniteHorizonRemainderApplication.lean: bb55c871d9483ffd6c7312a58aea715eb98a3d8e8b2ae6de79eb926bca2e9463
- HarpRemainderTests.lean: b4e88e5e59fa0197a6b58a95224b05be0e8e0a1e06aeac16b594ece0ea207d6a
- HarpRemainderApplicationTests.lean: 2ac87bb97dd4d888347c7c4982d4f75014fd35e813946a346006d157b308b00b

PDF check runtimes: pypdf 6.10.0 and pdfplumber 0.11.9. The browser and Node
versions are in the generated build manifest. No proof or rendering source
changed after the accepted gate; final edits record acceptance and refresh
the self-referential repository payload receipt.

The foundations workstream remains blocked by 8mq3 and m4qy; this independent
slice does not close those issues or the whole-book umbrella.
