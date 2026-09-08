# Scoped foundations acceptance contract

This contract freezes the foundations interface needed by the approved
[mathematics package](../../superpowers/specs/2026-09-07-harp-mathematics-program-design.md).
It covers Chapter 1 prerequisite repairs and the
[Chapters 2–4 core](../../superpowers/specs/2026-09-07-harp-foundations-02-04-design.md).
It does not declare the original
[Wave 6 plan](../../superpowers/plans/2026-08-23-crouzeix-textbook-wave-6-foundations.md)
complete, authorize Chapters 5–24 expansion, or replace the existing coverage
and exercise contracts. Canonical mathematical exposition remains under
`knowledge/crouzeix_textbook/`.

## Stable interface and completion boundary

Chapters 1–12 retain exactly six coverage rows and six exercise rows each:
`CFT-CC-001` through `CFT-CC-006` and `CFT-CC-E01` through `CFT-CC-E06`.
Their 72 card anchors are `cft-cc-nnn`; their 72 exercise anchors are
`exercise-cft-cc-enn`. Each anchor resolves once in its registered chapter.
The current whole-book roster remains 36 chapters and 216 rows in each
contract; the old plan's 35/210 counts are not publication authority.

| Surface | Acceptance in this package | Remaining obligation |
| --- | --- | --- |
| Chapters 1–4, all 24 cards | Active, exact Lean statement correspondence | Compilation and accurate provider, hypothesis, and source metadata remain required |
| CFT-01-001 and CFT-03-005 | Definitions; prose proof not applicable | Do not count definitions as reconstructed theorem proofs |
| CFT-04-004 through 006 | Exact statements, explicitly labeled previews, prose proof summary | Full proof exposition belongs to later scalar-invariant foundations work |
| CFT-01-005 | Exact characteristic-polynomial statement; an explicitly labeled summary preview is allowed | Reconstructible status requires a prerequisite-safe proof, not a forward citation |
| Other Chapters 1–4 theorems | Explicit statement cards and reconstructible proofs | Independent review must establish that the argument can be recovered from the reading path |
| Chapters 1–4 exercises | 24 distinct local solution theorem statements | Compile and compare against the written tasks; names and hashes alone do not establish semantic agreement |
| Chapters 5–12 | Preserve all 48 card and 48 exercise identities; correspondence remains non-exact, proofs summary, solutions null | Future approved foundations waves supply the missing proofs, editorial depth, and distinct solutions |

Preserve existing public declaration names, including Chapter 1's six
`CrouzeixTextbook.Part01.Exercises.Chapter01.exercise_YY_solution` names.
Chapters 2–4 use the corresponding `Exercises.ChapterCC.exercise_YY_solution`
names. A strengthened card may cite a more complete supporting declaration
without deleting its existing public declaration or changing the mathematical
identity of the CFT item. Declaration names and anchor labels are not evidence
that two theorem statements agree.

Every completed exercise has a different compiler statement fingerprint from
the other completed exercises and from public cards. An exercise may reuse
legitimate proof lemmas; it may not merely rename a checkpoint or restate its
parent card. The existing compiler-backed publication checks remain responsible
for declaration kind, source bytes and positions, hypotheses, and fingerprint
freshness. Do not hand-author a compiler receipt to satisfy this contract.

## Editorial interface and review

Each accepted chapter carries explicit `Motivation` and `Historical context`
labels. The ML bridge labels its `Mathematical object`, `Exact transfer`,
`Non-transfer`, and `Calculation`. Markdown headings and bold labels ending
in a period or colon are acceptable. Each field contains its own explanation;
there is no word-count threshold. Historical context must distinguish
pedagogical framing from attributed historical claims, which need the exact
edition and locator used by the source registry.

Motivation and analogy paragraphs must be specific to their chapters. Repeated
paragraphs across these fields fail the mechanical fixture even when wrapped
differently. A different sentence expressing the same generic filler is still
an editorial defect; the fixture does not pretend to judge that semantic
question. Shared mathematical notation and a legitimately reused running
example are expected.

Each accepted CFT card states its object, scalar assumptions, quantified data,
hypotheses, and conclusion explicitly. An identifiable `Statement` label may
be part of the bold theorem title. Reconstructible theorem cards have a
`Proof` exposition. Definitions explain the construction; summary cards
visibly say `preview` or `forward reference`. Exact Lean correspondence and
self-contained exposition are separate acceptance judgments. Chapter 4's
scalar-invariant previews cannot become prerequisites of its duality core.

Every chapter in the eventual twelve-chapter route must calculate with
`A_{\lambda,\alpha}` and explain the calculation through that chapter's
mathematical object. The current package accepts this requirement only for
Chapters 1–4. Review the following cumulative progression in canonical prose:

| Chapter | Required mathematical use of the family |
| --- | --- |
| 1 | Representation, basis change, and the effect of a nonunitary coordinate change on Euclidean lengths |
| 2 | Invariant subspace, generalized eigenspace, and quotient action; distinguish a complement from an invariant complement |
| 3 | Kernels and ranges of the zero-eigenvalue specialization and its powers, plus invariant restriction |
| 4 | Primal action and algebraic dual pullback, with transpose distinguished from a metric adjoint |

The redundant feature map from the approved Chapters 2–4 design is a second
continuous example, not a replacement for this matrix-family requirement.
Later chapters retain the Wave 6 obligations: determinant/trace (5), polynomial
response (6), projections and adjoints (7), Gram geometry (8), singular values
(9), tensor contraction (10), differentiation (11), and orientation (12).
None is claimed completed by the current core slice.

## Verification seam

`crates/harp/tests/foundations_contract.rs` reads canonical chapter Markdown and
the existing JSON contracts. It freezes the roster, current core statuses,
distinct registered solution fingerprints, and explicitly pending remainder.
Its temporary fixtures demonstrate that a missing analogy label and repeated
motivation/ML paragraphs are rejected. It checks that the cumulative family is
present; the mathematical relevance and correctness of the calculation require
chapter-specific behavioral tests, compiled support, and independent review.
It adds neither a runtime validator nor another public manifest.

The initial RED run had four passing tests and three integration failures:
Chapter 1 still registered checkpoint correspondence, Chapters 2–4 lacked
solution records, and chapter editorial fields were absent. Fixture development
also observed RED before implementing each rejection. These are development
observations, not current publication status. The integrated package must run:

```sh
MISE_TRUSTED_CONFIG_PATHS="$PWD/mise.toml" mise exec -- sh -c \
  'CARGO_TARGET_DIR="$HARP_TARGET_DIR" cargo test --locked --offline --profile test-small -p harp --test foundations_contract -- --test-threads=1'
```

Run the focused compiler-backed chapter tests and publication checks separately;
the editorial test does not repeatedly invoke Lean. The final package requires
the repository's `mise run verify` gate after canonical inputs, correspondence,
ledgers, corpus, Atlas, and the requested rendered artifacts are integrated.
Local Rust checks are not hermetic Lean proof-execution receipts. No dependency
cache hydration is authorized by this contract.

## Truthful issue disposition

Record this scoped contract freeze as verified evidence only after its
integrated acceptance tests pass. The original `8mq3` issue has whole-wave
conditions and the unresolved `fsg8` analysis-wave dependency. Do not remove
that dependency, close `fsg8`, or claim all 72 exercise solutions to make the
scoped freeze appear complete. The scoped prerequisite is sufficient to proceed
with the approved core; completing unrelated whole-wave work is not required
to record that narrower fact.

Evaluate Chapter 1 issue `m4qy` against its actual proof requirements. If its
characteristic-polynomial exposition remains a preview, record the accepted
core and remaining proof obligation explicitly. Chapter 4 likewise remains
incomplete under full-chapter acceptance while CFT-04-004 through 006 are
summaries. Close Chapter 2 or Chapter 3 issues only with their full applicable
chapter evidence. The pending Chapters 5–12 and larger textbook program stay
open. This document itself closes no issue and changes no dependency.

## Independent contract review

The specification review found that card extraction truncated nested theorem
headings. The implementation now preserves child headings while stopping at
peer headings or the next card. A regression also prevents borrowing a proof
from the following card. The quality review then found that adjacent bold
title and statement labels, used by Chapter 1, were not recognized. A second
positive fixture and missing-statement mutation cover that format. The same
quality reviewer rechecked both fixes and independently ran the two focused
theorem-card regressions successfully. No blocking parser findings remained.

During integration, promoting Chapter 1 exposed one further compatibility
expectation: CFT-01-006 is a retained `worked-example`, not a `theorem` kind.
Its exact correspondence and reconstructible proof requirements are unchanged.
The test must preserve that existing kind rather than mutate the public card
to satisfy a generic assertion. This is a bounded fixture repair, not an
extension to the mathematical writing scope.
