# Harp mathematics improvement specification

Status: approved for implementation by the user on 2026-09-07.
External review requests, merge, push, and deployment remain unauthorized.

Execution disposition: theorem assurance and the finite-horizon remainder
passed local acceptance. The user subsequently authorized autonomous completion
including scoped prerequisite repairs. The local contract and Chapters 1–4
have passed separate source reviews, coordinated publication, presentation
checks and the full repository gate. All three scoped workstreams are accepted
locally. The older 8mq3/m4qy whole-wave obligations remain distinct
from this core acceptance. This is not whole-program or whole-book completion.
Detailed evidence is in docs/workstream/harp-mathematics/.

Baseline: `2e5d57da339d83b90bb631e0b1939958bd7369a6`.
Draft branch: `codex/harp-mathematics-spec`.

## Outcome and audience

Make the existing Harp mathematics easier to audit, reconstruct, and apply.
The reader has undergraduate mathematics and PhD-level ML research experience.
The narrative is mathematically led, self-contained along its declared reading
path, and uses modern square-bracket matrix notation. Motivation, historical
context, ML analogies, and formal guarantees are explicitly distinguished.

This specification is implemented through three dated plans in
`docs/superpowers/plans/`, each with a bounded acceptance record.

## Chosen approach and alternatives

Recommended: combine a statement-first assurance audit, a foundations slice,
and one quantitative theorem. This improves confidence and reader access while
keeping new mathematical claims small enough to review.

Alternative A, textbook-only, has less research risk but leaves the semantic
audit and finite-horizon quantitative opportunity untouched. Alternative B,
an approximate cubature solver first, requires stability and rounding results
that the current existential construction does not provide. Neither is the
first package.

## Workstreams

1. [Theorem assurance](2026-09-07-harp-theorem-assurance-design.md): expose the
   intended statement and audit its definitions, hypotheses, and proof chain.
2. [Foundations](2026-09-07-harp-foundations-02-04-design.md): deepen Chapters
   2 and 3 and the duality core of Chapter 4, preserving existing identities.
3. [Finite-horizon remainder](2026-09-07-harp-finite-horizon-remainder-design.md):
   prove a quantitative scalar remainder and instantiate it for Harp.

Implementation order: freeze the statement audit first; then develop Chapters
2, 3, and 4 in order. The remainder work may proceed after the audit has
accepted the fixed-core and finite-recurrence contracts. Serialize edits to
shared contracts, compiler receipts, generated ledgers, and website exports.
Review each mathematical slice before beginning the next dependent slice.

## Verified inventory, not new proof verification

The baseline structured contracts contain 216 coverage rows, 72 exact
correspondences, 71 reconstructible expositions, 143 summaries, and two
not-applicable exposition rows. There are 78 formally solved exercises out of
216. These counts describe the checked-in records; no new Lean build was run
to write this specification.

Chapters 2 through 4 contribute 18 coverage rows: 17 theorems and one definition.
At that baseline they had no exact correspondence rows and no formal exercise
solutions. Chapter 4 contains three forward references to scalar invariants;
their statement correspondence and their proof exposition need different
completion criteria.

The prior completion PRD remains the writing and theorem-card standard:
`docs/superpowers/specs/2026-08-26-crouzeix-textbook-completion-prd.md`.
Its historical 35-chapter/210-row roster does not override the current
36-chapter/216-row contracts. This package does not restore whole-book
completion or close the remaining foundations program.

## Non-goals

- No fourth terminal proof, stronger universal constant, or assertion of
  external acceptance or priority.
- No complete-boundedness or matrix-valued functional-calculus extension.
- No certified numerical cubature, floating-point implementation, or claim
  about nonlinear or time-varying neural-network stability.
- No rewrite of Chapters 5 through 24, new chapter number, global notation
  migration, or website redesign.
- No toolchain updates, dependency downloads, external checkout dependencies,
  historical evidence rewrites, or new foundational axioms.
- No unsolicited outreach to an external reviewer.

## Common mathematical and Lean contract

Each displayed theorem has a readable statement with all assumptions, proof
roadmap, complete calculation, boundary example, and a Lean source link.
Formal panels name the public declaration and substantive provider, list
assumptions and recorded axioms, and distinguish reexported proof, proof written
here, definition, checkpoint, and informal material.

Registered metadata comes from the compiler and current Rust publisher.
Never type fabricated source positions, normalized type hashes, or receipts.
Supporting lemmas must also compile and have source links. An unindexed helper
is labeled as such; it does not increase indexed coverage counts. A separate
proof-only compile report may resolve named helpers without becoming a second
publication manifest or a new authority for coverage.

For general module statements, explain the exact specialization to the book's
real or complex vector spaces. Do not pretend the prose proves a more general
semiring theorem. A stronger statement requires a separate proof and explicit
scope. Use `$...$` and `$$...$$` with `bmatrix` for the shared renderer.

An ML bridge names the mathematical object, the exact correspondence, the
assumptions or non-transfer boundary, and a calculation. No analogy acquires
formal verification status by proximity to a theorem. Historical claims require
source attribution; original exposition must not reproduce protected books.

## Acceptance and review

Every slice passes mathematical review and prose-to-Lean review separately.
Record source revision, exact files, reviewer role, findings, repairs, commands,
exit statuses, and any skipped check. An agent review is not external human
peer review. A compiled theorem is not evidence that every surrounding sentence
matches its type.

For implementation, preflight the pinned toolchain and canonical warm cache
without Lake, use narrow chapter or Harp targets while iterating, and freeze
the candidate before `lean-all` and `mise run verify`. No cache hydration is
part of this package. Ordinary local execution is labeled non-hermetic.

Publication acceptance includes the full repository gate, actual browser
rendering at desktop and mobile widths, exact Lean-link resolution, and rebuilt
PDF source snapshots. Preserve Chapter 1 and Chapter 36 standalone editions.
If adding standalone Chapters 2 through 4 later, their edition metadata and
source bundles require separate explicit acceptance; they are not required here.

Run `node --test tools/textbook_pdf/*.test.cjs` and the maintained PDF builder
and checker after final canonical changes. Inspect equation-heavy pages and
relative source links visually; geometry checks alone are insufficient. Read
the current PDF skill and discover runtime dependencies before building PDFs.

If changed formal sources invalidate selected proof evidence, use the maintained
immutable refresh workflow. A textbook-only change does not automatically
require regenerating terminal proof evidence. Check the actual bound inputs.

## Tracking and authority

Reuse Kata issues `8mq3` for the foundations contract, `x5fp` for Chapter 2,
`6bsx` for Chapter 3, and `q8h9` for Chapter 4. The current dependency chain
includes `m4qy`, Chapter 1; inspect its acceptance evidence before executing
Chapter 2. Do not remove blockers or close it merely because some exercise
solutions already exist. `00zs` remains the whole-book umbrella.

Search before creating new assurance or quantitative-theorem issues. Create
those execution issues after the specification is accepted. Do not reopen the
completed Chapter 36 implementation issue `fq4m` as if its prior acceptance
had failed; link new follow-up work to it.

The specification turn performed no implementation or new formal verification.
The subsequent user instruction to proceed authorizes implementation.
External review and repository publication remain separate decisions.
